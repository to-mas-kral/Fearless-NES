use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use crate::app::Gui;

pub struct Ppu {
    pub window_active: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            window_active: false,
        }
    }
}

impl Gui for Ppu {
    fn gui_window(app: &mut crate::app::App, egui_ctx: &egui::CtxRef) {
        let ppu_window_active = &mut app.debug.ppu.window_active;
        let nes = &app.nes;

        if *ppu_window_active {
            egui::Window::new("PPU")
                .open(ppu_window_active)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    if let Some(ref nes) = nes {
                        let mut hasher = DefaultHasher::new();
                        hasher.write(nes.get_frame_buffer());
                        let hash = hasher.finish();

                        ui.text_edit_singleline(&mut format!("Display hash: {}", hash));
                    }
                });
        }
    }
}
