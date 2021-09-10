use egui::Label;

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
                        let header = nes.get_ines_header();

                        egui::Grid::new("iNes Header Grid")
                            .striped(true)
                            .spacing([20., 5.])
                            .show(ui, |ui| {
                                ui.add(Label::new("Field").heading().strong());
                                ui.add(Label::new("Value").heading().strong());
                                ui.end_row();

                                ui.label("Mapper");
                                ui.add(Label::new(format!("{}", header.mapper_id)).monospace());
                                ui.end_row();

                                ui.label("PRG ROM count");
                                ui.add(
                                    Label::new(format!("{} * 16KB", header.prg_rom_count))
                                        .monospace(),
                                );
                                ui.end_row();

                                ui.label("CHR ROM count");
                                ui.add(
                                    Label::new(format!("{} * 8KB", header.chr_rom_count))
                                        .monospace(),
                                )
                                .on_hover_text("0 CHR ROM count means that the board uses CHR RAM");
                                ui.end_row();

                                ui.label("PRG RAM count");
                                ui.add(
                                    Label::new(format!("{} * 8KB", header.prg_ram_count))
                                        .monospace(),
                                );
                                ui.end_row();

                                ui.label("Mirroring");
                                ui.add(Label::new(format!("{:?}", header.mirroring)));
                                ui.end_row();

                                ui.label("Has Battery");
                                ui.add(Label::new(format!("{}", header.has_battery)));
                                ui.end_row();
                            });
                    });
            }
            _ => (),
        }
    }
}
