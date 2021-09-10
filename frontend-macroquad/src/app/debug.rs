use crate::app::Gui;

mod cartridge_info;
mod ppu;

use cartridge_info::CartridgeInfo;
use ppu::Ppu;

use super::App;

pub struct Debug {
    pub show_controls: bool,
    pub show_cpu: bool,
    pub cartridge_info: CartridgeInfo,
    pub ppu: Ppu,
    pub perf: Perf,
}

impl Debug {
    pub fn new() -> Self {
        Self {
            show_controls: false,
            show_cpu: false,
            cartridge_info: CartridgeInfo::new(),
            ppu: Ppu::new(),
            perf: Perf::new(),
        }
    }
}

impl Gui for Debug {
    fn gui_window(app: &mut super::App, egui_ctx: &egui::CtxRef) {
        if let Some(ref mut nes) = app.nes {
            if app.debug.show_cpu {
                egui::Window::new("CPU State")
                    .open(&mut app.debug.show_cpu)
                    .resizable(false)
                    .default_width(0.)
                    .show(egui_ctx, |ui| {
                        let cpu = nes.get_cpu_state();

                        ui.columns(2, |columns| {
                            columns[0].group(|ui| {
                                egui::Grid::new("Registers")
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.add(egui::Label::new("Registers").heading().strong());
                                        ui.add(egui::Label::new("Value").heading().strong());
                                        ui.end_row();

                                        ui.label("A").on_hover_text("accumulator");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.a))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("X").on_hover_text("X index");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.x))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("Y").on_hover_text("Y index");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.y))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("PC").on_hover_text("program counter");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.pc))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("SP").on_hover_text("stack pointer");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.sp))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();
                                    });
                            });

                            columns[1].group(|ui| {
                                egui::Grid::new("Flags").striped(true).show(ui, |ui| {
                                    ui.add(egui::Label::new("Flag").heading().strong());
                                    ui.add(egui::Label::new("Value").heading().strong());
                                    ui.end_row();

                                    ui.label("N").on_hover_text("negative");
                                    ui.checkbox(&mut cpu.n, "");
                                    ui.end_row();

                                    ui.label("V").on_hover_text("overflow");
                                    ui.checkbox(&mut cpu.v, "");
                                    ui.end_row();

                                    ui.label("I").on_hover_text("irq inhibit");
                                    ui.checkbox(&mut cpu.i, "");
                                    ui.end_row();

                                    ui.label("Z").on_hover_text("zero");
                                    ui.checkbox(&mut cpu.z, "");
                                    ui.end_row();

                                    ui.label("C").on_hover_text("carry");
                                    ui.checkbox(&mut cpu.c, "");
                                    ui.end_row();
                                });
                            });
                        });
                    });
            }

            let paused = &mut app.paused;

            if app.debug.show_controls {
                egui::Window::new("Controls and Status")
                    .open(&mut app.debug.show_controls)
                    .resizable(false)
                    .default_width(0.)
                    .show(egui_ctx, |ui| {
                        ui.checkbox(paused, "Paused");
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

            CartridgeInfo::gui_window(app, egui_ctx);
            Ppu::gui_window(app, egui_ctx);
            Perf::gui_window(app, egui_ctx);
        }
    }
}

pub struct Perf {
    pub window_active: bool,
    pub measuring: bool,
    /// In millis
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

    pub fn add_frame_time(&mut self, frame_time: u128) {
        if self.measuring {
            self.total_frame_time += frame_time;
            self.total_frames += 1;
        }
    }
}

impl Gui for Perf {
    fn gui_window(app: &mut App, egui_ctx: &egui::CtxRef) {
        let perf = &mut app.debug.perf;
        let measuring = &mut perf.measuring;
        let total_frame_time = &mut perf.total_frame_time;
        let total_frames = &mut perf.total_frames;
        let perf_window_active = &mut perf.window_active;

        egui::Window::new("PPU")
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

                let avg = *total_frame_time as f64 / *total_frames as f64;
                ui.label(format!("Average frame time: {:.2}ms", avg));
            });
    }
}
