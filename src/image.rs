use crate::viewer;
use image::{
    self, imageops, DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Rgb, RgbImage,
    Rgba, RgbaImage,
};
use pyo3::{
    exceptions::{PyIOError, PyValueError},
    prelude::*,
};
use std::{env, path::Path, process::Command};
use tempfile::Builder;

#[pyclass]
pub struct FenrirImage {
    width: u32,
    height: u32,
    buffer: DynamicImage,
}

enum ImageBufferMut<'a> {
    Rgb(&'a mut RgbImage),
    Rgba(&'a mut RgbaImage),
    Luma(&'a mut GrayImage),
}

enum ImageBufferRef<'a> {
    Rgb(&'a RgbImage),
    Rgba(&'a RgbaImage),
    Luma(&'a GrayImage),
}

impl FenrirImage {
    fn normalize_dynamic(image: DynamicImage) -> DynamicImage {
        match image {
            DynamicImage::ImageRgb8(_)
            | DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageLuma8(_) => image,
            other => DynamicImage::ImageRgba8(other.to_rgba8()),
        }
    }

    fn from_dynamic(image: DynamicImage) -> Self {
        let normalized = FenrirImage::normalize_dynamic(image);
        let (width, height) = normalized.dimensions();
        Self {
            width,
            height,
            buffer: normalized,
        }
    }

    fn set_from_dynamic(&mut self, image: DynamicImage) {
        let normalized = FenrirImage::normalize_dynamic(image);
        let (width, height) = normalized.dimensions();
        self.width = width;
        self.height = height;
        self.buffer = normalized;
    }

    pub(crate) fn duplicate(&self) -> Self {
        FenrirImage::from_dynamic(self.buffer.clone())
    }

    pub(crate) fn snapshot_rgba(&self) -> (Vec<u8>, u32, u32) {
        let rgba = self.buffer.to_rgba8();
        let (w, h) = rgba.dimensions();
        (rgba.into_vec(), w, h)
    }

    fn buffer_mut(&mut self) -> ImageBufferMut<'_> {
        match self.buffer {
            DynamicImage::ImageRgb8(ref mut img) => ImageBufferMut::Rgb(img),
            DynamicImage::ImageRgba8(ref mut img) => ImageBufferMut::Rgba(img),
            DynamicImage::ImageLuma8(ref mut img) => ImageBufferMut::Luma(img),
            _ => {
                // Should be unreachable because we always normalize,
                // but convert defensively to keep PyO3 happy.
                let converted = self.buffer.to_rgba8();
                self.buffer = DynamicImage::ImageRgba8(converted);
                self.buffer_mut()
            }
        }
    }

    fn buffer_ref(&self) -> ImageBufferRef<'_> {
        match &self.buffer {
            DynamicImage::ImageRgb8(img) => ImageBufferRef::Rgb(img),
            DynamicImage::ImageRgba8(img) => ImageBufferRef::Rgba(img),
            DynamicImage::ImageLuma8(img) => ImageBufferRef::Luma(img),
            _ => panic!("FenrirImage buffer in unsupported format"),
        }
    }

    fn validate_point(&self, x: u32, y: u32) -> PyResult<()> {
        if x >= self.width || y >= self.height {
            return Err(PyValueError::new_err("Point outside image bounds"));
        }
        Ok(())
    }

    fn validate_rect(&self, x: u32, y: u32, w: u32, h: u32) -> PyResult<()> {
        if w == 0 || h == 0 {
            return Err(PyValueError::new_err(
                "Rectangle width and height must be greater than zero",
            ));
        }
        if x >= self.width || y >= self.height {
            return Err(PyValueError::new_err(
                "Rectangle origin outside the image bounds",
            ));
        }

        let max_x = x
            .checked_add(w)
            .ok_or_else(|| PyValueError::new_err("Rectangle width overflow"))?;
        let max_y = y
            .checked_add(h)
            .ok_or_else(|| PyValueError::new_err("Rectangle height overflow"))?;

        if max_x > self.width || max_y > self.height {
            return Err(PyValueError::new_err(
                "Rectangle exceeds image dimensions",
            ));
        }

        Ok(())
    }
}

fn luma_from_rgba(color: (u8, u8, u8, u8)) -> u8 {
    let (r, g, b, _) = color;
    let value = 0.299f32 * (r as f32) + 0.587f32 * (g as f32) + 0.114f32 * (b as f32);
    value.round().clamp(0.0, 255.0) as u8
}

fn lerp_channel(start: u8, end: u8, t: f32) -> u8 {
    let start_f = start as f32;
    let end_f = end as f32;
    (start_f + (end_f - start_f) * t).round().clamp(0.0, 255.0) as u8
}

const ASCII_GRADIENT: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

#[pymethods]
impl FenrirImage {
    /// Cria uma nova imagem preenchida com a cor especificada.
    #[staticmethod]
    pub fn new(width: u32, height: u32, mode: &str, color: (u8, u8, u8, u8)) -> PyResult<Self> {
        if width == 0 || height == 0 {
            return Err(PyValueError::new_err(
                "Image dimensions must be greater than zero",
            ));
        }

        let mode_upper = mode.to_ascii_uppercase();
        let buffer = match mode_upper.as_str() {
            "RGB" => {
                let pixel = Rgb([color.0, color.1, color.2]);
                DynamicImage::ImageRgb8(ImageBuffer::from_pixel(width, height, pixel))
            }
            "RGBA" => {
                let pixel = Rgba([color.0, color.1, color.2, color.3]);
                DynamicImage::ImageRgba8(ImageBuffer::from_pixel(width, height, pixel))
            }
            "L" => {
                let value = luma_from_rgba(color);
                let pixel = Luma([value]);
                DynamicImage::ImageLuma8(ImageBuffer::from_pixel(width, height, pixel))
            }
            _ => {
                return Err(PyValueError::new_err(
                    "Mode must be one of: RGB, RGBA ou L",
                ))
            }
        };

        Ok(Self {
            width,
            height,
            buffer,
        })
    }

    /// Abre uma imagem do disco.
    #[staticmethod]
    pub fn open(path: &str) -> PyResult<Self> {
        let img = image::open(path)
            .map_err(|e| PyIOError::new_err(format!("Erro abrindo imagem: {}", e)))?;
        Ok(FenrirImage::from_dynamic(img))
    }

    /// Retorna (largura, altura).
    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Retorna o valor RGBA de um pixel.
    pub fn get_pixel(&self, x: u32, y: u32) -> PyResult<(u8, u8, u8, u8)> {
        self.validate_point(x, y)?;
        let color = match self.buffer_ref() {
            ImageBufferRef::Rgb(img) => {
                let pixel = img.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                (r, g, b, 255)
            }
            ImageBufferRef::Rgba(img) => {
                let pixel = img.get_pixel(x, y);
                let [r, g, b, a] = pixel.0;
                (r, g, b, a)
            }
            ImageBufferRef::Luma(img) => {
                let pixel = img.get_pixel(x, y);
                let value = pixel.0[0];
                (value, value, value, 255)
            }
        };

        Ok(color)
    }

    /// Salva a imagem em disco.
    pub fn save(&self, path: &str) -> PyResult<()> {
        self.buffer
            .save(path)
            .map_err(|e| PyIOError::new_err(format!("Erro salvando imagem: {}", e)))
    }

    /// Corta a imagem para uma região e retorna o recorte.
    pub fn crop(&mut self, x: u32, y: u32, w: u32, h: u32) -> PyResult<Self> {
        self.validate_rect(x, y, w, h)?;
        let cropped = self.buffer.crop_imm(x, y, w, h);
        let normalized = FenrirImage::normalize_dynamic(cropped);
        let (nw, nh) = normalized.dimensions();
        self.width = nw;
        self.height = nh;
        self.buffer = normalized.clone();
        Ok(FenrirImage {
            width: nw,
            height: nh,
            buffer: normalized,
        })
    }

    /// Redimensiona usando filtro Lanczos3.
    pub fn resize(&mut self, w: u32, h: u32) -> PyResult<()> {
        if w == 0 || h == 0 {
            return Err(PyValueError::new_err(
                "Resize dimensions must be greater than zero",
            ));
        }
        let resized =
            self.buffer
                .resize(w, h, image::imageops::FilterType::Lanczos3);

        let normalized = FenrirImage::normalize_dynamic(resized);
        self.width = w;
        self.height = h;
        self.buffer = normalized;
        Ok(())
    }

    /// Divide a imagem em segmentos verticais/horizontais.
    pub fn split(&self, cuts: Vec<u32>) -> PyResult<Vec<FenrirImage>> {
        if cuts.len() < 2 {
            return Err(PyValueError::new_err(
                "Split requires at least the axis and one cut point",
            ));
        }

        let axis = cuts[0];
        if axis != 0 && axis != 1 {
            return Err(PyValueError::new_err(
                "First value must be 0 (vertical) ou 1 (horizontal)",
            ));
        }

        let mut points: Vec<u32> = cuts[1..].to_vec();
        points.sort_unstable();

        let limit = if axis == 0 { self.width } else { self.height };
        let mut segments = Vec::new();
        let mut start = 0u32;

        for point in points {
            if point <= start || point > limit {
                continue;
            }

            let (x, y, w, h) = if axis == 0 {
                (start, 0, point - start, self.height)
            } else {
                (0, start, self.width, point - start)
            };

            if w == 0 || h == 0 {
                continue;
            }

            let piece = self.buffer.crop_imm(x, y, w, h);
            segments.push(FenrirImage::from_dynamic(piece));
            start = point;
        }

        if start < limit {
            let (x, y, w, h) = if axis == 0 {
                (start, 0, limit - start, self.height)
            } else {
                (0, start, self.width, limit - start)
            };

            if w > 0 && h > 0 {
                let piece = self.buffer.crop_imm(x, y, w, h);
                segments.push(FenrirImage::from_dynamic(piece));
            }
        }

        Ok(segments)
    }

    /// Desenha um pixel na posição especificada.
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: (u8, u8, u8, u8)) -> PyResult<()> {
        self.validate_point(x, y)?;
        match self.buffer_mut() {
            ImageBufferMut::Rgb(img) => {
                img.put_pixel(x, y, Rgb([color.0, color.1, color.2]));
            }
            ImageBufferMut::Rgba(img) => {
                img.put_pixel(x, y, Rgba([color.0, color.1, color.2, color.3]));
            }
            ImageBufferMut::Luma(img) => {
                let value = luma_from_rgba(color);
                img.put_pixel(x, y, Luma([value]));
            }
        }
        Ok(())
    }

    /// Desenha uma linha usando Bresenham.
    pub fn draw_line(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: (u8, u8, u8, u8),
    ) -> PyResult<()> {
        let width = self.width as i32;
        let height = self.height as i32;

        for (x, y) in [(x1, y1), (x2, y2)] {
            if x < 0 || y < 0 || x >= width || y >= height {
                return Err(PyValueError::new_err(
                    "Line endpoints must be inside the image bounds",
                ));
            }
        }

        let mut x0 = x1;
        let mut y0 = y1;
        let dx = (x2 - x1).abs();
        let sx = if x0 < x2 { 1 } else { -1 };
        let dy = -(y2 - y1).abs();
        let sy = if y0 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.draw_pixel(x0 as u32, y0 as u32, color)?;
            if x0 == x2 && y0 == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }

        Ok(())
    }

    /// Desenha um retângulo (preenchido ou apenas bordas).
    #[pyo3(signature = (x, y, w, h, color, fill=true))]
    pub fn draw_rect(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        color: (u8, u8, u8, u8),
        fill: bool,
    ) -> PyResult<()> {
        self.validate_rect(x, y, w, h)?;

        if fill {
            for yy in y..(y + h) {
                for xx in x..(x + w) {
                    self.draw_pixel(xx, yy, color)?;
                }
            }
            return Ok(());
        }

        let x2 = (x + w - 1) as i32;
        let y2 = (y + h - 1) as i32;
        let x = x as i32;
        let y = y as i32;

        self.draw_line(x, y, x2, y, color)?;
        self.draw_line(x, y2, x2, y2, color)?;
        if h > 1 {
            self.draw_line(x, y, x, y2, color)?;
            self.draw_line(x2, y, x2, y2, color)?;
        }

        Ok(())
    }

    /// Preenche a imagem inteira com uma cor.
    pub fn fill(&mut self, color: (u8, u8, u8, u8)) -> PyResult<()> {
        match self.buffer_mut() {
            ImageBufferMut::Rgb(img) => {
                for pixel in img.pixels_mut() {
                    *pixel = Rgb([color.0, color.1, color.2]);
                }
            }
            ImageBufferMut::Rgba(img) => {
                for pixel in img.pixels_mut() {
                    *pixel = Rgba([color.0, color.1, color.2, color.3]);
                }
            }
            ImageBufferMut::Luma(img) => {
                let value = luma_from_rgba(color);
                for pixel in img.pixels_mut() {
                    *pixel = Luma([value]);
                }
            }
        }
        Ok(())
    }

    /// Aplica um gradiente linear horizontal ou vertical.
    pub fn linear_gradient(
        &mut self,
        direction: &str,
        color_start: (u8, u8, u8, u8),
        color_end: (u8, u8, u8, u8),
    ) -> PyResult<()> {
        let dir_lower = direction.to_ascii_lowercase();
        let horizontal = match dir_lower.as_str() {
            "horizontal" => true,
            "vertical" => false,
            _ => {
                return Err(PyValueError::new_err(
                    "Direction must be 'horizontal' or 'vertical'",
                ))
            }
        };

        let width = self.width;
        let height = self.height;
        let den = if horizontal {
            if width <= 1 {
                1.0
            } else {
                (width - 1) as f32
            }
        } else if height <= 1 {
            1.0
        } else {
            (height - 1) as f32
        };

        match self.buffer_mut() {
            ImageBufferMut::Rgb(img) => {
                for y in 0..height {
                    for x in 0..width {
                        let t = if horizontal {
                            x as f32 / den
                        } else {
                            y as f32 / den
                        };
                        let r = lerp_channel(color_start.0, color_end.0, t);
                        let g = lerp_channel(color_start.1, color_end.1, t);
                        let b = lerp_channel(color_start.2, color_end.2, t);
                        img.put_pixel(x, y, Rgb([r, g, b]));
                    }
                }
            }
            ImageBufferMut::Rgba(img) => {
                for y in 0..height {
                    for x in 0..width {
                        let t = if horizontal {
                            x as f32 / den
                        } else {
                            y as f32 / den
                        };
                        let r = lerp_channel(color_start.0, color_end.0, t);
                        let g = lerp_channel(color_start.1, color_end.1, t);
                        let b = lerp_channel(color_start.2, color_end.2, t);
                        let a = lerp_channel(color_start.3, color_end.3, t);
                        img.put_pixel(x, y, Rgba([r, g, b, a]));
                    }
                }
            }
            ImageBufferMut::Luma(img) => {
                let start = luma_from_rgba(color_start);
                let end = luma_from_rgba(color_end);
                for y in 0..height {
                    for x in 0..width {
                        let t = if horizontal {
                            x as f32 / den
                        } else {
                            y as f32 / den
                        };
                        let value = lerp_channel(start, end, t);
                        img.put_pixel(x, y, Luma([value]));
                    }
                }
            }
        }

        Ok(())
    }

    /// Rotaciona a imagem em 90 graus.
    pub fn rotate_90(&mut self) -> PyResult<()> {
        let rotated = self.buffer.rotate90();
        self.set_from_dynamic(rotated);
        Ok(())
    }

    /// Rotaciona a imagem em 180 graus.
    pub fn rotate_180(&mut self) -> PyResult<()> {
        let rotated = self.buffer.rotate180();
        self.set_from_dynamic(rotated);
        Ok(())
    }

    /// Rotaciona a imagem em 270 graus.
    pub fn rotate_270(&mut self) -> PyResult<()> {
        let rotated = self.buffer.rotate270();
        self.set_from_dynamic(rotated);
        Ok(())
    }

    /// Salva a imagem em um arquivo temporário e abre no visualizador do sistema.
    pub fn show(&self) -> PyResult<()> {
        let temp_file = Builder::new()
            .prefix("fenrir-preview-")
            .suffix(".png")
            .tempfile()
            .map_err(|e| PyIOError::new_err(format!("Erro criando arquivo temporário: {}", e)))?;

        self.buffer
            .save(temp_file.path())
            .map_err(|e| PyIOError::new_err(format!("Erro salvando imagem temporária: {}", e)))?;

        let (_, path) = temp_file
            .keep()
            .map_err(|e| PyIOError::new_err(format!("Erro preservando arquivo temporário: {}", e)))?;

        launch_viewer(&path)
    }

    /// Converte a imagem para arte ASCII proporcional.
    pub fn to_ascii(&self, width: u32) -> PyResult<String> {
        if width == 0 {
            return Err(PyValueError::new_err("Width must be greater than zero"));
        }
        let (orig_w, orig_h) = self.get_size();
        if orig_w == 0 || orig_h == 0 {
            return Err(PyValueError::new_err("Imagem inválida"));
        }

        let aspect = orig_h as f64 / orig_w as f64;
        let mut target_height = (width as f64 * aspect).round() as u32;
        if target_height == 0 {
            target_height = 1;
        }

        let gray = self.buffer.to_luma8();
        let resized = imageops::resize(
            &gray,
            width,
            target_height,
            imageops::FilterType::Nearest,
        );

        let mut output = String::with_capacity(((width + 1) * target_height) as usize);
        for y in 0..target_height {
            for x in 0..width {
                let pixel = resized.get_pixel(x, y);
                let value = pixel[0];
                let gradient_index =
                    (value as usize * (ASCII_GRADIENT.len() - 1)) / 255;
                output.push(ASCII_GRADIENT[gradient_index]);
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Abre o viewer interativo estilo Photoshop.
    pub fn show_viewer(&self) -> PyResult<()> {
        let clone = self.duplicate();
        viewer::show_image(clone);
        Ok(())
    }
}

fn launch_viewer(path: &Path) -> PyResult<()> {
    if let Ok(command) = env::var("FENRIR_SHOW_COMMAND") {
        return spawn_command(&command, path);
    }
    spawn_default_viewer(path)
}

fn spawn_command(command: &str, path: &Path) -> PyResult<()> {
    Command::new(command)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| PyIOError::new_err(format!("Erro abrindo visualizador: {}", e)))
}

#[cfg(target_os = "linux")]
fn spawn_default_viewer(path: &Path) -> PyResult<()> {
    spawn_command("xdg-open", path)
}

#[cfg(target_os = "macos")]
fn spawn_default_viewer(path: &Path) -> PyResult<()> {
    spawn_command("open", path)
}

#[cfg(target_os = "windows")]
fn spawn_default_viewer(path: &Path) -> PyResult<()> {
    let mut command = Command::new("cmd");
    command.arg("/C").arg("start").arg("").arg(path);
    command
        .spawn()
        .map(|_| ())
        .map_err(|e| PyIOError::new_err(format!("Erro abrindo visualizador: {}", e)))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn spawn_default_viewer(_path: &Path) -> PyResult<()> {
    Err(PyIOError::new_err(
        "Visualização não suportada nesta plataforma",
    ))
}
