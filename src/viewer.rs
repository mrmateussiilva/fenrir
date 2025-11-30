use crate::image::FenrirImage;
use eframe::{
    egui::{self, Color32, ColorImage, TextureOptions, Vec2},
    App, CreationContext, Frame, NativeOptions,
};
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};
use pyo3::PyResult;

pub fn show_image(image: FenrirImage) {
    if env::var("FENRIR_DISABLE_VIEWER")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return;
    }

    let shared = Arc::new(Mutex::new(image));

    let _ = thread::Builder::new()
        .name("fenrir-viewer".to_string())
        .spawn({
            let shared = Arc::clone(&shared);
            move || {
                let options = viewer_native_options();
                let _ = eframe::run_native(
                    "Fenrir Viewer",
                    options,
                    Box::new(move |cc| {
                        Ok(Box::new(FenrirViewerApp::new(
                            cc,
                            Arc::clone(&shared),
                        )))
                    }),
                );
            }
        });
}

fn base_native_options() -> NativeOptions {
    NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(900.0, 600.0))
            .with_min_inner_size(Vec2::new(480.0, 360.0)),
        follow_system_theme: true,
        ..Default::default()
    }
}

#[cfg(target_os = "linux")]
fn viewer_native_options() -> NativeOptions {
    let mut options = base_native_options();
    options.event_loop_builder = Some(Box::new(|builder| {
        {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            builder.with_any_thread(true);
        }
        {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_any_thread(true);
        }
    }));
    options
}

#[cfg(not(target_os = "linux"))]
fn viewer_native_options() -> NativeOptions {
    base_native_options()
}

struct FenrirViewerApp {
    texture: egui::TextureHandle,
    shared: Arc<Mutex<FenrirImage>>,
    image_size: Vec2,
    zoom: f32,
    offset: Vec2,
    active_tool: Tool,
    crop: CropState,
    resize: ResizeState,
    split: SplitState,
    draw_pixel: DrawPixelState,
    draw_line: DrawLineState,
    draw_rect: DrawRectState,
    fill_color: ColorInput,
    gradient: GradientState,
    ascii: AsciiState,
    status: Option<StatusMessage>,
}

impl FenrirViewerApp {
    fn new(cc: &CreationContext<'_>, shared: Arc<Mutex<FenrirImage>>) -> Self {
        let (color_image, size) = snapshot_color_image(&shared);
        let texture = cc.egui_ctx.load_texture("fenrir-image", color_image, TextureOptions::LINEAR);

        Self {
            texture,
            shared,
            image_size: size,
            zoom: 1.0,
            offset: Vec2::ZERO,
            active_tool: Tool::None,
            crop: CropState::default(),
            resize: ResizeState::default(),
            split: SplitState::default(),
            draw_pixel: DrawPixelState::default(),
            draw_line: DrawLineState::default(),
            draw_rect: DrawRectState::default(),
            fill_color: ColorInput::solid(255, 255, 255, 255),
            gradient: GradientState::default(),
            ascii: AsciiState::default(),
            status: None,
        }
    }

    fn update_texture_from_pixels(&mut self, pixels: Vec<u8>, w: u32, h: u32) {
        let color_image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
        self.texture.set(color_image, TextureOptions::LINEAR);
        self.image_size = Vec2::new(w as f32, h as f32);
    }

    fn apply_and_refresh<F>(&mut self, ctx: &egui::Context, mut op: F) -> Result<(), String>
    where
        F: FnMut(&mut FenrirImage) -> PyResult<()>,
    {
        let mut guard = self
            .shared
            .lock()
            .map_err(|_| "Falha ao acessar a imagem".to_string())?;
        op(&mut guard).map_err(|e| e.to_string())?;
        let (pixels, w, h) = guard.snapshot_rgba();
        drop(guard);
        self.update_texture_from_pixels(pixels, w, h);
        ctx.request_repaint();
        Ok(())
    }

    fn apply_replace<F>(&mut self, ctx: &egui::Context, mut op: F) -> Result<(), String>
    where
        F: FnMut(&mut FenrirImage) -> PyResult<FenrirImage>,
    {
        let mut guard = self
            .shared
            .lock()
            .map_err(|_| "Falha ao acessar a imagem".to_string())?;
        let new_img = op(&mut guard).map_err(|e| e.to_string())?;
        *guard = new_img;
        let (pixels, w, h) = guard.snapshot_rgba();
        drop(guard);
        self.update_texture_from_pixels(pixels, w, h);
        ctx.request_repaint();
        Ok(())
    }

    fn set_status_ok(&mut self, msg: impl Into<String>) {
        self.status = Some(StatusMessage::Info(msg.into()));
    }

    fn set_status_err(&mut self, msg: impl Into<String>) {
        self.status = Some(StatusMessage::Error(msg.into()));
    }

    fn crop_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        numeric_input(ui, "X", &mut self.crop.x);
        numeric_input(ui, "Y", &mut self.crop.y);
        numeric_input(ui, "Largura", &mut self.crop.width);
        numeric_input(ui, "Altura", &mut self.crop.height);
        if ui.button("Aplicar corte").clicked() {
            let parsed = (|| {
                let x = parse_u32(&self.crop.x, "X")?;
                let y = parse_u32(&self.crop.y, "Y")?;
                let w = parse_u32(&self.crop.width, "largura")?;
                let h = parse_u32(&self.crop.height, "altura")?;
                self.apply_replace(ctx, |img| img.crop(x, y, w, h))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Crop aplicado"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn resize_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        numeric_input(ui, "Nova largura", &mut self.resize.width);
        numeric_input(ui, "Nova altura", &mut self.resize.height);
        if ui.button("Redimensionar").clicked() {
            let parsed = (|| {
                let w = parse_u32(&self.resize.width, "largura")?;
                let h = parse_u32(&self.resize.height, "altura")?;
                self.apply_and_refresh(ctx, |img| img.resize(w, h))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Resize aplicado"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn split_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.split.axis, SplitAxis::Vertical, "Vertical");
            ui.selectable_value(&mut self.split.axis, SplitAxis::Horizontal, "Horizontal");
        });
        ui.label("Cortes (valores separados por vírgula)");
        ui.text_edit_singleline(&mut self.split.cuts);
        if ui.button("Aplicar split").clicked() {
            let parsed = (|| {
                let mut cuts = Vec::new();
                for value in self.split.cuts.split(',') {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let pos = trimmed
                        .parse::<u32>()
                        .map_err(|_| format!("Valor inválido de corte: {trimmed}"))?;
                    cuts.push(pos);
                }
                if cuts.is_empty() {
                    return Err("Informe pelo menos um corte".to_string());
                }
                let mut payload = Vec::with_capacity(cuts.len() + 1);
                payload.push(self.split.axis.code());
                payload.extend(cuts);
                let (first, total) = {
                    let guard = self
                        .shared
                        .lock()
                        .map_err(|_| "Falha ao acessar a imagem".to_string())?;
                    let segments = guard.split(payload).map_err(|e| e.to_string())?;
                    let total = segments.len();
                    let first = segments
                        .into_iter()
                        .next()
                        .ok_or_else(|| "Nenhum segmento gerado".to_string())?;
                    (first, total)
                };
                let mut guard = self
                    .shared
                    .lock()
                    .map_err(|_| "Falha ao atualizar imagem".to_string())?;
                *guard = first;
                let (pixels, w, h) = guard.snapshot_rgba();
                drop(guard);
                self.update_texture_from_pixels(pixels, w, h);
                ctx.request_repaint();
                Ok(total)
            })();
            match parsed {
                Ok(total) => self.set_status_ok(format!("Split gerou {total} partes (mostrando a primeira)")),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn rotate_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label("Rotacionar:");
        ui.horizontal(|ui| {
            if ui.button("90°").clicked() {
                self.handle_simple(ctx, |img| img.rotate_90(), "Imagem rotacionada em 90°");
            }
            if ui.button("180°").clicked() {
                self.handle_simple(ctx, |img| img.rotate_180(), "Imagem rotacionada em 180°");
            }
            if ui.button("270°").clicked() {
                self.handle_simple(ctx, |img| img.rotate_270(), "Imagem rotacionada em 270°");
            }
        });
    }

    fn draw_pixel_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        numeric_input(ui, "X", &mut self.draw_pixel.x);
        numeric_input(ui, "Y", &mut self.draw_pixel.y);
        color_inputs(ui, &mut self.draw_pixel.color);
        if ui.button("Desenhar pixel").clicked() {
            let parsed = (|| {
                let x = parse_u32(&self.draw_pixel.x, "X")?;
                let y = parse_u32(&self.draw_pixel.y, "Y")?;
                let color = self.draw_pixel.color.to_tuple()?;
                self.apply_and_refresh(ctx, |img| img.draw_pixel(x, y, color))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Pixel desenhado"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn draw_line_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        numeric_input(ui, "X1", &mut self.draw_line.x1);
        numeric_input(ui, "Y1", &mut self.draw_line.y1);
        numeric_input(ui, "X2", &mut self.draw_line.x2);
        numeric_input(ui, "Y2", &mut self.draw_line.y2);
        color_inputs(ui, &mut self.draw_line.color);
        if ui.button("Desenhar linha").clicked() {
            let parsed = (|| {
                let x1 = parse_i32(&self.draw_line.x1, "X1")?;
                let y1 = parse_i32(&self.draw_line.y1, "Y1")?;
                let x2 = parse_i32(&self.draw_line.x2, "X2")?;
                let y2 = parse_i32(&self.draw_line.y2, "Y2")?;
                let color = self.draw_line.color.to_tuple()?;
                self.apply_and_refresh(ctx, |img| img.draw_line(x1, y1, x2, y2, color))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Linha desenhada"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn draw_rect_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        numeric_input(ui, "X", &mut self.draw_rect.x);
        numeric_input(ui, "Y", &mut self.draw_rect.y);
        numeric_input(ui, "Largura", &mut self.draw_rect.width);
        numeric_input(ui, "Altura", &mut self.draw_rect.height);
        ui.checkbox(&mut self.draw_rect.fill, "Preenchido");
        color_inputs(ui, &mut self.draw_rect.color);
        if ui.button("Desenhar retângulo").clicked() {
            let parsed = (|| {
                let x = parse_u32(&self.draw_rect.x, "X")?;
                let y = parse_u32(&self.draw_rect.y, "Y")?;
                let w = parse_u32(&self.draw_rect.width, "largura")?;
                let h = parse_u32(&self.draw_rect.height, "altura")?;
                let color = self.draw_rect.color.to_tuple()?;
                let fill = self.draw_rect.fill;
                self.apply_and_refresh(ctx, |img| img.draw_rect(x, y, w, h, color, fill))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Retângulo aplicado"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn fill_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        color_inputs(ui, &mut self.fill_color);
        if ui.button("Preencher imagem").clicked() {
            let parsed = (|| {
                let color = self.fill_color.to_tuple()?;
                self.apply_and_refresh(ctx, |img| img.fill(color))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Imagem preenchida"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn gradient_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.gradient.horizontal, true, "Horizontal");
            ui.selectable_value(&mut self.gradient.horizontal, false, "Vertical");
        });
        ui.label("Cor inicial");
        color_inputs(ui, &mut self.gradient.start);
        ui.label("Cor final");
        color_inputs(ui, &mut self.gradient.end);
        if ui.button("Aplicar gradiente").clicked() {
            let parsed = (|| {
                let dir = if self.gradient.horizontal {
                    "horizontal"
                } else {
                    "vertical"
                };
                let start = self.gradient.start.to_tuple()?;
                let end = self.gradient.end.to_tuple()?;
                self.apply_and_refresh(ctx, |img| img.linear_gradient(dir, start, end))
            })();
            match parsed {
                Ok(_) => self.set_status_ok("Gradiente aplicado"),
                Err(err) => self.set_status_err(err),
            }
        }
    }

    fn ascii_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Largura alvo");
        ui.text_edit_singleline(&mut self.ascii.width);
        if ui.button("Gerar ASCII").clicked() {
            let parsed = (|| {
                let width = parse_u32(&self.ascii.width, "largura")?;
                let guard = self
                    .shared
                    .lock()
                    .map_err(|_| "Falha ao acessar a imagem".to_string())?;
                guard.to_ascii(width).map_err(|e| e.to_string())
            })();
            match parsed {
                Ok(result) => {
                    self.ascii.result = Some(result);
                    self.set_status_ok("ASCII gerado");
                }
                Err(err) => self.set_status_err(err),
            }
        }
        if let Some(text) = &self.ascii.result {
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.monospace(text);
                });
        }
    }

    fn handle_simple<F>(&mut self, ctx: &egui::Context, op: F, ok_msg: &str)
    where
        F: FnMut(&mut FenrirImage) -> PyResult<()>,
    {
        match self.apply_and_refresh(ctx, op) {
            Ok(_) => self.set_status_ok(ok_msg),
            Err(err) => self.set_status_err(err),
        }
    }
}

impl App for FenrirViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.request_repaint();

        egui::SidePanel::left("fenrir-sidebar")
            .resizable(false)
            .exact_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Fenrir Tools");
                ui.separator();
                for (tool, label) in TOOL_BUTTONS {
                    if ui.button(*label).clicked() {
                        self.active_tool = *tool;
                    }
                }
                ui.separator();
                match self.active_tool {
                    Tool::None => {
                        ui.label("Selecione uma ferramenta");
                    }
                    Tool::Crop => self.crop_ui(ui, ctx),
                    Tool::Resize => self.resize_ui(ui, ctx),
                    Tool::Split => self.split_ui(ui, ctx),
                    Tool::Rotate => self.rotate_ui(ui, ctx),
                    Tool::DrawPixel => self.draw_pixel_ui(ui, ctx),
                    Tool::DrawLine => self.draw_line_ui(ui, ctx),
                    Tool::DrawRect => self.draw_rect_ui(ui, ctx),
                    Tool::Fill => self.fill_ui(ui, ctx),
                    Tool::Gradient => self.gradient_ui(ui, ctx),
                    Tool::Ascii => self.ascii_ui(ui),
                }
                if let Some(status) = &self.status {
                    ui.separator();
                    match status {
                        StatusMessage::Info(msg) => {
                            ui.colored_label(Color32::LIGHT_GREEN, msg);
                        }
                        StatusMessage::Error(msg) => {
                            ui.colored_label(Color32::LIGHT_RED, msg);
                        }
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::drag());

            if response.dragged() {
                self.offset += response.drag_delta();
            }

            let scroll = ui.input(|i| i.raw_scroll_delta.y + i.smooth_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                let factor = 1.0 + (scroll * 0.001);
                self.zoom = (self.zoom * factor).clamp(0.1, 20.0);
            }

            draw_checker(&painter, response.rect);

            let scaled_size = self.image_size * self.zoom;
            let center = response.rect.center() + self.offset;
            let top_left = center - scaled_size * 0.5;
            let image_rect = egui::Rect::from_min_size(top_left, scaled_size);

            painter.image(
                self.texture.id(),
                image_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            let fps = ui.input(|i| if i.stable_dt > 0.0 { 1.0 / i.stable_dt } else { 0.0 });
            let overlay = format!("Zoom: {:>4.0}% | FPS: {:>4.0}", self.zoom * 100.0, fps);
            painter.text(
                response.rect.left_top() + egui::vec2(8.0, 8.0),
                egui::Align2::LEFT_TOP,
                overlay,
                egui::FontId::monospace(14.0),
                Color32::WHITE,
            );
        });
    }
}

fn draw_checker(painter: &egui::Painter, rect: egui::Rect) {
    let tile = 24.0;
    let mut y = rect.top();
    let mut row = 0;
    while y < rect.bottom() {
        let mut x = rect.left();
        let mut col = 0;
        while x < rect.right() {
            let color = if (row + col) % 2 == 0 {
                Color32::from_gray(70)
            } else {
                Color32::from_gray(100)
            };
            let tile_rect = egui::Rect::from_min_max(
                egui::pos2(x, y),
                egui::pos2((x + tile).min(rect.right()), (y + tile).min(rect.bottom())),
            );
            painter.rect_filled(tile_rect, 0.0, color);
            x += tile;
            col += 1;
        }
        y += tile;
        row += 1;
    }
}

fn snapshot_color_image(shared: &Arc<Mutex<FenrirImage>>) -> (ColorImage, Vec2) {
    if let Ok(guard) = shared.lock() {
        let (pixels, w, h) = guard.snapshot_rgba();
        let image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
        (image, Vec2::new(w as f32, h as f32))
    } else {
        (
            ColorImage::from_rgba_unmultiplied([1, 1], &[255, 255, 255, 255]),
            Vec2::new(1.0, 1.0),
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    None,
    Crop,
    Resize,
    Split,
    Rotate,
    DrawPixel,
    DrawLine,
    DrawRect,
    Fill,
    Gradient,
    Ascii,
}

const TOOL_BUTTONS: &[(Tool, &str)] = &[
    (Tool::Crop, "Crop"),
    (Tool::Resize, "Resize"),
    (Tool::Split, "Split"),
    (Tool::Rotate, "Rotate"),
    (Tool::DrawPixel, "Draw Pixel"),
    (Tool::DrawLine, "Draw Line"),
    (Tool::DrawRect, "Draw Rect"),
    (Tool::Fill, "Fill"),
    (Tool::Gradient, "Gradient"),
    (Tool::Ascii, "ASCII Preview"),
];

#[derive(Default)]
struct CropState {
    x: String,
    y: String,
    width: String,
    height: String,
}

#[derive(Default)]
struct ResizeState {
    width: String,
    height: String,
}

#[derive(Clone, Copy, PartialEq)]
enum SplitAxis {
    Vertical,
    Horizontal,
}

impl Default for SplitAxis {
    fn default() -> Self {
        SplitAxis::Vertical
    }
}

impl SplitAxis {
    fn code(self) -> u32 {
        match self {
            SplitAxis::Vertical => 0,
            SplitAxis::Horizontal => 1,
        }
    }
}

#[derive(Default)]
struct SplitState {
    axis: SplitAxis,
    cuts: String,
}

#[derive(Default)]
struct DrawPixelState {
    x: String,
    y: String,
    color: ColorInput,
}

#[derive(Default)]
struct DrawLineState {
    x1: String,
    y1: String,
    x2: String,
    y2: String,
    color: ColorInput,
}

#[derive(Default)]
struct DrawRectState {
    x: String,
    y: String,
    width: String,
    height: String,
    color: ColorInput,
    fill: bool,
}

#[derive(Clone)]
struct ColorInput {
    r: String,
    g: String,
    b: String,
    a: String,
}

impl Default for ColorInput {
    fn default() -> Self {
        Self::solid(255, 0, 0, 255)
    }
}

impl ColorInput {
    fn solid(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r.to_string(),
            g: g.to_string(),
            b: b.to_string(),
            a: a.to_string(),
        }
    }

    fn to_tuple(&self) -> Result<(u8, u8, u8, u8), String> {
        Ok((
            parse_u8(&self.r, "R")?,
            parse_u8(&self.g, "G")?,
            parse_u8(&self.b, "B")?,
            parse_u8(&self.a, "A")?,
        ))
    }
}

#[derive(Default)]
struct GradientState {
    horizontal: bool,
    start: ColorInput,
    end: ColorInput,
}

#[derive(Default)]
struct AsciiState {
    width: String,
    result: Option<String>,
}

enum StatusMessage {
    Info(String),
    Error(String),
}

fn numeric_input(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(value);
    });
}

fn color_inputs(ui: &mut egui::Ui, input: &mut ColorInput) {
    ui.horizontal(|ui| {
        ui.label("R");
        ui.text_edit_singleline(&mut input.r);
        ui.label("G");
        ui.text_edit_singleline(&mut input.g);
    });
    ui.horizontal(|ui| {
        ui.label("B");
        ui.text_edit_singleline(&mut input.b);
        ui.label("A");
        ui.text_edit_singleline(&mut input.a);
    });
}

fn parse_u32(value: &str, name: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("Valor inválido para {name}"))
}

fn parse_i32(value: &str, name: &str) -> Result<i32, String> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|_| format!("Valor inválido para {name}"))
}

fn parse_u8(value: &str, name: &str) -> Result<u8, String> {
    value
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Valor inválido para {name}"))
}
