use crate::app::Gui;

pub struct Settings {
    pub overscan: OverscanUi,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            overscan: OverscanUi::new(),
        }
    }
}

impl Gui for Settings {
    fn gui_window(app: &mut crate::App, egui_ctx: &egui::CtxRef) {
        OverscanUi::gui_window(app, egui_ctx);
    }
}

pub struct OverscanUi {
    pub window_shown: bool,
}

impl OverscanUi {
    pub fn new() -> Self {
        Self {
            window_shown: false,
        }
    }
}

impl Gui for OverscanUi {
    fn gui_window(app: &mut crate::App, egui_ctx: &egui::CtxRef) {
        let window_shown = &mut app.settings.overscan.window_shown;
        let top = &mut app.config.overscan.top;
        let right = &mut app.config.overscan.right;
        let bottom = &mut app.config.overscan.bottom;
        let left = &mut app.config.overscan.left;

        egui::Window::new("Overscan")
            .open(window_shown)
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.add(
                    egui::Slider::new(top, 0..=16)
                        .text("Top")
                        .clamp_to_range(true)
                        .integer(),
                );

                ui.add(
                    egui::Slider::new(right, 0..=16)
                        .text("Right")
                        .clamp_to_range(true)
                        .integer(),
                );

                ui.add(
                    egui::Slider::new(bottom, 0..=16)
                        .text("Bottom")
                        .clamp_to_range(true)
                        .integer(),
                );

                ui.add(
                    egui::Slider::new(left, 0..=16)
                        .text("Left")
                        .clamp_to_range(true)
                        .integer(),
                );
            });
    }
}
