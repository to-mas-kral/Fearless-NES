use egui::Label;
use fearless_nes::BankSize;

use crate::app::Gui;

pub struct CartridgeInfo {
    pub window_active: bool,
}

impl CartridgeInfo {
    pub fn new() -> Self {
        Self {
            window_active: false,
        }
    }
}

impl Gui for CartridgeInfo {
    fn gui_window(app: &mut crate::app::App, egui_ctx: &egui::CtxRef) {
        match (&mut app.nes, app.debug.cartridge_info.window_active) {
            (Some(nes), true) => {
                egui::Window::new("Cartridge Info")
                    .open(&mut app.debug.cartridge_info.window_active)
                    .resizable(false)
                    .default_width(0.)
                    .show(egui_ctx, |ui| {
                        let cartridge = nes.get_cartridge();
                        let header = &cartridge.header;

                        egui::Grid::new("Header Grid")
                            .striped(true)
                            .spacing([20., 5.])
                            .show(ui, |ui| {
                                ui.add(Label::new("Field").heading().strong());
                                ui.add(Label::new("Value").heading().strong());
                                ui.end_row();

                                ui.label("Header source");
                                ui.add(Label::new(format!("{}", header.source)));
                                ui.end_row();

                                ui.label("Name");
                                ui.add(Label::new(format!("{}", header.name)));
                                ui.end_row();

                                ui.label("Mapper : Submapper");
                                ui.add(
                                    Label::new(format!("{} : {}", header.mapper, header.submapper))
                                        .monospace(),
                                );
                                ui.end_row();

                                ui.label("PRG ROM count");
                                ui.add(
                                    Label::new(format!(
                                        "{} KB",
                                        cartridge.prg_rom_count(BankSize::Kb1)
                                    ))
                                    .monospace(),
                                );
                                ui.end_row();

                                ui.label("PRG RAM count");
                                ui.add(
                                    Label::new(format!(
                                        "{} KB",
                                        cartridge.prg_ram_count(BankSize::Kb1).unwrap_or(0)
                                    ))
                                    .monospace(),
                                );
                                ui.end_row();

                                ui.label("PRG NVRAM count");
                                ui.add(
                                    Label::new(format!(
                                        "{} KB",
                                        cartridge.prg_nvram_count(BankSize::Kb1).unwrap_or(0)
                                    ))
                                    .monospace(),
                                );
                                ui.end_row();

                                ui.label("CHR ROM count");
                                ui.add(
                                    Label::new(format!(
                                        "{} KB",
                                        cartridge.chr_rom_count(BankSize::Kb1).unwrap_or(0)
                                    ))
                                    .monospace(),
                                );
                                ui.end_row();

                                ui.label("CHR RAM count");
                                ui.add(
                                    Label::new(format!(
                                        "{} KB",
                                        cartridge.chr_ram_count(BankSize::Kb1).unwrap_or(0)
                                    ))
                                    .monospace(),
                                );
                                ui.end_row();

                                ui.label("Mirroring");
                                ui.add(Label::new(format!("{:?}", header.mirroring)));
                                ui.end_row();

                                ui.label("Has Battery");
                                ui.add(Label::new(format!("{}", header.battery)));
                                ui.end_row();

                                ui.label("Console Type");
                                ui.add(Label::new(format!("{}", header.console_typ)));
                                ui.end_row();

                                ui.label("Region");
                                ui.add(Label::new(format!("{}", header.region)));
                                ui.end_row();

                                ui.label("Expansion device");
                                ui.add(Label::new(format!("{}", header.expansion)));
                                ui.end_row();
                            });
                    });
            }
            _ => (),
        }
    }
}
