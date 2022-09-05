use egui_glium::egui_winit::egui::{self, Color32, ColorImage, Pos2, TextureHandle};
use serde::{Deserialize, Serialize};

use fearless_nes::{FRAMEBUFFER_SIZE, NES_HEIGHT, NES_WIDTH, PALETTE};

pub struct NesRender {
    pub image: ColorImage,
    texture: Option<TextureHandle>,
}

impl NesRender {
    pub fn new() -> Self {
        Self {
            image: ColorImage::new([NES_WIDTH, NES_HEIGHT], Color32::BLACK),
            texture: None,
        }
    }

    pub fn draw_nes(
        &mut self,
        nes_framebuffer: &[u8; FRAMEBUFFER_SIZE],
        egui_context: &egui::Context,
        overscan: &Overscan,
    ) {
        for y in 0..NES_HEIGHT {
            for x in 0..NES_WIDTH {
                let palette_index = nes_framebuffer[y * NES_WIDTH + x] as usize * 3;
                let r = PALETTE[palette_index];
                let g = PALETTE[palette_index + 1];
                let b = PALETTE[palette_index + 2];

                self.image[(x, y)] = Color32::from_rgb(r, g, b);
            }
        }

        match &mut self.texture {
            Some(t) => t.set(self.image.clone(), egui::TextureFilter::Nearest),
            None => {
                self.texture = Some(egui_context.load_texture(
                    "NES-screen",
                    self.image.clone(),
                    egui::TextureFilter::Nearest,
                ))
            }
        };

        let uv = Self::calculate_uv(overscan);

        egui::CentralPanel::default().show(egui_context, |ui| {
            let size = Self::calculate_nes_size(overscan);

            let img = egui::Image::new(self.texture.as_ref().unwrap(), size).uv(uv);

            let available = egui_context.available_rect();
            let rect = Self::calculate_nes_rect(available);

            img.paint_at(ui, rect);
        });
    }

    fn calculate_nes_size(overscan: &Overscan) -> [f32; 2] {
        let width = NES_WIDTH as f32 - overscan.left as f32 - overscan.right as f32;
        let height = NES_HEIGHT as f32 - overscan.top as f32 - overscan.bottom as f32;

        [width, height]
    }

    fn calculate_uv(overscan: &Overscan) -> [Pos2; 2] {
        let left = overscan.left as f32 / NES_WIDTH as f32;
        let top = overscan.top as f32 / NES_HEIGHT as f32;
        let right = overscan.right as f32 / NES_WIDTH as f32;
        let bottom = overscan.bottom as f32 / NES_HEIGHT as f32;

        [Pos2::new(left, top), Pos2::new(1.0 - right, 1.0 - bottom)]
    }

    // FIXME: overscan still stretches the image
    fn calculate_nes_rect(available: egui::Rect) -> egui::Rect {
        let rect_side = f32::min(available.width(), available.height());
        let center = available.center();

        let topx = center.x - rect_side / 2.;
        let topy = center.y - rect_side / 2.;
        let botx = center.x + rect_side / 2.;
        let boty = center.y + rect_side / 2.;

        egui::Rect {
            min: egui::pos2(topx, topy),
            max: egui::pos2(botx, boty),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Overscan {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Overscan {
    pub fn new() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }
}
