use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use pyo3::{
    exceptions::{PyIOError, PyValueError},
    prelude::*,
};
use std::path::Path;

#[pyclass]
pub struct FenrirTiff {
    path: String,
    width: u32,
    height: u32,
    pages: usize,
    current_page: usize,
    is_big_tiff: bool,
}

#[pymethods]
impl FenrirTiff {
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(PyIOError::new_err(format!(
                "Arquivo não encontrado: {}",
                path
            )));
        }

        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if ext != "tif" && ext != "tiff" {
            return Err(PyValueError::new_err(
                "Arquivo deve ser TIFF (.tif ou .tiff)",
            ));
        }

        let file = std::fs::File::open(path)
            .map_err(|e| PyIOError::new_err(format!("Erro abrindo TIFF: {}", e)))?;

        let mut decoder = tiff::decoder::Decoder::new(file)
            .map_err(|e| PyIOError::new_err(format!("Erro decodificando TIFF: {}", e)))?;

        let (width, height) = decoder
            .dimensions()
            .map_err(|e| PyIOError::new_err(format!("Erro obtendo dimensões: {}", e)))?;

        let pages = 1;
        let is_big_tiff = false;

        Ok(Self {
            path: path.to_string(),
            width,
            height,
            pages,
            current_page: 0,
            is_big_tiff,
        })
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_page_count(&self) -> usize {
        self.pages
    }

    pub fn is_big_tiff(&self) -> bool {
        self.is_big_tiff
    }

    pub fn set_page(&mut self, page: usize) -> PyResult<()> {
        if page >= self.pages {
            return Err(PyValueError::new_err("Página inválida"));
        }
        self.current_page = page;
        Ok(())
    }

    pub fn get_current_page(&self) -> usize {
        self.current_page
    }

    pub fn to_fenrir_image(&self) -> PyResult<super::image::FenrirImage> {
        let img = image::open(&self.path)
            .map_err(|e| PyIOError::new_err(format!("Erro abrindo TIFF: {}", e)))?;

        Ok(super::image::FenrirImage::from_dynamic(img))
    }

    pub fn load_region(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> PyResult<super::image::FenrirImage> {
        if x >= self.width || y >= self.height {
            return Err(PyValueError::new_err("Coordenadas fora dos limites"));
        }

        if width == 0 || height == 0 {
            return Err(PyValueError::new_err("Dimensões inválidas"));
        }

        let img = image::open(&self.path)
            .map_err(|e| PyIOError::new_err(format!("Erro abrindo TIFF: {}", e)))?;

        let actual_width = width.min(self.width - x);
        let actual_height = height.min(self.height - y);

        let cropped = img.crop_imm(x, y, actual_width, actual_height);

        Ok(super::image::FenrirImage::from_dynamic(cropped))
    }
}

#[pyclass]
pub struct FenrirTiffWriter {
    path: String,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[pymethods]
impl FenrirTiffWriter {
    #[new]
    pub fn new(path: &str, width: u32, height: u32) -> PyResult<Self> {
        if width == 0 || height == 0 {
            return Err(PyValueError::new_err(
                "Dimensões devem ser maiores que zero",
            ));
        }

        Ok(Self {
            path: path.to_string(),
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize],
        })
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: (u8, u8, u8, u8)) -> PyResult<()> {
        if x >= self.width || y >= self.height {
            return Err(PyValueError::new_err("Pixel fora dos limites"));
        }

        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = color.0;
        self.data[idx + 1] = color.1;
        self.data[idx + 2] = color.2;
        self.data[idx + 3] = color.3;

        Ok(())
    }

    pub fn fill(&mut self, color: (u8, u8, u8, u8)) -> PyResult<()> {
        for i in (0..self.data.len()).step_by(4) {
            self.data[i] = color.0;
            self.data[i + 1] = color.1;
            self.data[i + 2] = color.2;
            self.data[i + 3] = color.3;
        }
        Ok(())
    }

    pub fn save(&self) -> PyResult<()> {
        let img: RgbaImage = ImageBuffer::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| PyIOError::new_err("Erro criando imagem"))?;

        img.save(&self.path)
            .map_err(|e| PyIOError::new_err(format!("Erro salvando TIFF: {}", e)))
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
