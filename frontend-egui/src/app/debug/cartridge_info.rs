use egui_glium::egui_winit::egui::{self, Label, RichText};
use fearless_nes::{BankSize, Cartridge, Header};

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

impl CartridgeInfo {
    pub fn gui_window(app: &mut crate::app::App, egui_ctx: &egui::Context) {
        if let (Some(nes), true) = (&mut app.nes, app.debug.cartridge_info.window_active) {
            egui::Window::new("Cartridge Info")
                .open(&mut app.debug.cartridge_info.window_active)
                .resizable(false)
                .default_width(0.)
                .show(egui_ctx, |ui| {
                    let mut nes = nes.lock().unwrap();
                    let cartridge = nes.get_cartridge();
                    let header = &cartridge.header;

                    egui::Grid::new("Cartridge Header Grid")
                        .striped(true)
                        .spacing([20., 5.])
                        .show(ui, |ui| {
                            display_cartridge_info(ui, header, cartridge);
                        });
                });
        }
    }
}

fn display_cartridge_info(ui: &mut egui::Ui, header: &Header, cartridge: &Cartridge) {
    ui.label(RichText::new("Field").heading().strong());
    ui.label(RichText::new("Value").heading().strong());

    ui.end_row();

    ui.label("Header source");
    ui.add(Label::new(format!("{}", header.source)));
    ui.end_row();

    ui.label("Name");
    ui.add(Label::new(&header.name));
    ui.end_row();

    ui.label("Mapper : Submapper");
    ui.label(RichText::new(format!("{} : {}", header.mapper, header.submapper)).monospace());
    ui.end_row();

    ui.label("PRG ROM size");
    ui.label(RichText::new(format!("{} KB", cartridge.prg_rom_count(BankSize::Kb1))).monospace());
    ui.end_row();

    ui.label("PRG RAM size");
    ui.label(
        RichText::new(format!(
            "{} KB",
            cartridge.prg_ram_count(BankSize::Kb1).unwrap_or(0)
        ))
        .monospace(),
    );
    ui.end_row();

    ui.label("PRG NVRAM size");
    ui.label(
        RichText::new(format!(
            "{} KB",
            cartridge.prg_nvram_count(BankSize::Kb1).unwrap_or(0)
        ))
        .monospace(),
    );
    ui.end_row();

    ui.label("CHR ROM size");
    ui.label(
        RichText::new(format!(
            "{} KB",
            cartridge.chr_rom_count(BankSize::Kb1).unwrap_or(0)
        ))
        .monospace(),
    );
    ui.end_row();

    ui.label("CHR RAM size");
    ui.label(
        RichText::new(format!(
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
}
