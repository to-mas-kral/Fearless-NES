mod cartridge_info;
mod ppu;

use cartridge_info::CartridgeInfo;
use egui_glium::egui_winit::egui;
use ppu::Ppu;

use crate::App;

pub struct Debug {
    pub show_controls: bool,
    pub cartridge_info: CartridgeInfo,
    pub ppu: Ppu,
    pub perf: Perf,
}

impl Debug {
    pub fn new() -> Self {
        Self {
            show_controls: false,
            cartridge_info: CartridgeInfo::new(),
            ppu: Ppu::new(),
            perf: Perf::new(),
        }
    }
}

impl Debug {
    pub fn gui_window(app: &mut super::App, egui_ctx: &egui::Context) {
        if let Some(nes) = &app.nes {
            {
                let mut nes = nes.lock().unwrap();

                if app.debug.show_controls {
                    egui::Window::new("Controls and Status")
                        .open(&mut app.debug.show_controls)
                        .resizable(false)
                        .default_width(0.)
                        .show(egui_ctx, |ui| {
                            if ui.button("Step frame").clicked() {
                                nes.run_one_frame();
                            }

                            if ui.button("Step CPU cycle").clicked() {
                                nes.run_cpu_cycle();
                            }

                            ui.label(format!("Frame count: {}", nes.get_frame_count()));
                            ui.label(format!("CPU cycle count: {}", nes.get_cycle_count()));
                        });
                }
            }

            CartridgeInfo::gui_window(app, egui_ctx);
            Ppu::gui_window(app, egui_ctx);
            Perf::gui_window(app, egui_ctx);
        }
    }

    pub fn gui_embed(app: &mut App, ui: &mut egui::Ui) {
        if ui.button("Controls and Status").clicked() {
            app.debug.show_controls = true;
        }

        if ui.button("PPU").clicked() {
            app.debug.ppu.window_active = true;
        }

        if ui.button("Cartridge Info").clicked() {
            app.debug.cartridge_info.window_active = true;
        }

        if ui.button("Performance").clicked() {
            app.debug.perf.window_active = true;
        }
    }
}

pub struct Perf {
    pub window_active: bool,
    pub measuring: bool,
    /// In nanos
    pub total_frame_time: u128,
    total_frames: u64,
}

impl Perf {
    fn new() -> Self {
        Self {
            window_active: false,
            measuring: false,
            total_frame_time: 0,
            total_frames: 0,
        }
    }

    // TODO(rewrite): performance monitoring
    /* pub fn add_frame_time(&mut self, frame_time: u128) {
        if self.measuring {
            self.total_frame_time += frame_time;
            self.total_frames += 1;
        }
    } */
}

impl Perf {
    pub fn gui_window(app: &mut App, egui_ctx: &egui::Context) {
        let perf = &mut app.debug.perf;
        let measuring = &mut perf.measuring;
        let total_frame_time = &mut perf.total_frame_time;
        let total_frames = &mut perf.total_frames;
        let perf_window_active = &mut perf.window_active;

        egui::Window::new("Performance")
            .open(perf_window_active)
            .resizable(false)
            .show(egui_ctx, |ui| {
                if ui.button("Toggle measuring").clicked() {
                    if *measuring {
                        *total_frame_time = 0;
                        *total_frames = 0;
                    }

                    *measuring = !*measuring;
                };

                let avg = (*total_frame_time as f64 / *total_frames as f64) / (1000. * 1000.);
                ui.label(format!("Average frame time: {:.2}ms", avg));
            });
    }
}
