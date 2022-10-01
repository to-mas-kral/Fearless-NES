use egui_glium::egui_winit::egui::{self, RichText};

use super::App;

pub struct Events {
    pub window_active: bool,
}

impl Events {
    pub fn new() -> Self {
        Self {
            window_active: false,
        }
    }

    pub fn gui_window(app: &mut App, egui_ctx: &egui::Context) {
        if let Some(n) = &mut app.nes {
            let n = n.lock().unwrap();
            let events = n.debug_events();

            egui::Window::new("Events")
                .open(&mut app.debug.events.window_active)
                .resizable(true)
                .show(egui_ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("events_grid").show(ui, |ui| {
                            ui.label(RichText::new("Event").heading().strong());
                            ui.label(RichText::new("Scanline").heading().strong());
                            ui.label(RichText::new("Xpos").heading().strong());
                            ui.end_row();

                            for e in events {
                                ui.label(e.kind.to_string());
                                ui.label(e.scanline.to_string());
                                ui.label(e.xpos.to_string());
                                ui.end_row();
                            }
                        });
                    });
                });
        }
    }
}
