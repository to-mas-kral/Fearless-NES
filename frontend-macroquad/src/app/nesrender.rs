use macroquad::prelude::*;

use crate::{NES_HEIGHT, NES_WIDTH};
use fearless_nes::PALETTE;

pub struct NesRender {
    pub image: Image,
    texture: Texture2D,
    scale: f32,
    draw_pos: f32,
}

impl NesRender {
    pub fn new() -> Self {
        let image = Image::gen_image_color(NES_WIDTH as u16, NES_HEIGHT as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        let mut s = Self {
            image,
            texture,
            scale: 0.,
            draw_pos: 0.,
        };

        s.recalc_draw_pos();

        s
    }

    pub fn update_frame(&mut self, nes_framebuffer: &[u8]) {
        for (i, pixel_color) in nes_framebuffer.iter().enumerate() {
            let palette_addr = (pixel_color * 3) as usize;

            let r = PALETTE[palette_addr];
            let g = PALETTE[palette_addr + 1];
            let b = PALETTE[palette_addr + 2];

            // TOOD: filling the image could be slow
            self.image.set_pixel(
                (i % NES_WIDTH) as u32,
                (i / NES_WIDTH) as u32,
                Color::from_rgba(r, g, b, u8::MAX),
            );
        }
    }

    pub fn draw_nes(&mut self) {
        self.recalc_draw_pos();

        self.texture.update(&self.image);

        draw_texture_ex(
            self.texture,
            self.draw_pos,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    NES_WIDTH as f32 * self.scale,
                    NES_HEIGHT as f32 * self.scale,
                )),
                ..Default::default()
            },
        );
    }

    fn recalc_draw_pos(&mut self) {
        let x_scale = screen_width() / NES_WIDTH as f32;
        let y_scale = screen_height() / NES_HEIGHT as f32;

        self.scale = x_scale.min(y_scale) as f32;
        self.draw_pos = (screen_width() - NES_WIDTH as f32 * self.scale) / 2.0;
    }
}
