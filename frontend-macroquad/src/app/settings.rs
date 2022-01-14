use egui::{Button, Color32, Label, Ui};
use gilrs::{EventType, Gilrs};
use macroquad::prelude::get_last_key_pressed;

use crate::app::config::Keybinds;
use crate::app::Gui;

use fearless_nes::Button as NesButton;

use super::App;

pub struct Settings {
    pub overscan: OverscanUi,
    pub keybinds: KeybindsUi,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            overscan: OverscanUi::new(),
            keybinds: KeybindsUi::new(),
        }
    }
}

impl Gui for Settings {
    fn gui_window(app: &mut crate::App, egui_ctx: &egui::CtxRef) {
        OverscanUi::gui_window(app, egui_ctx);
        KeybindsUi::gui_window(app, egui_ctx);
    }

    fn gui_embed(app: &mut App, ui: &mut Ui) {
        let mode_text = if app.config.dark_mode {
            "Light mode"
        } else {
            "Dark mode"
        };

        if ui.button(mode_text).clicked() {
            app.config.dark_mode = !app.config.dark_mode;
        }

        if ui.button("Overscan").clicked() {
            app.settings.overscan.window_shown = true;
        }

        if ui.button("Key bindings").clicked() {
            app.settings.keybinds.window_shown = true;
        }
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

pub struct KeybindsUi {
    pub window_shown: bool,
    selected: SelectedButton,
}

impl KeybindsUi {
    pub fn new() -> Self {
        Self {
            window_shown: false,
            selected: SelectedButton::None,
        }
    }

    fn display_keybind(
        binds: &mut Keybinds,
        btn: NesButton,
        ui: &mut Ui,
        selected: &mut SelectedButton,
    ) {
        let bind = &mut binds[btn];

        ui.label(btn.name());

        let (ctrl_color, kbd_color) = match *selected {
            SelectedButton::Controller(sbtn) if sbtn == btn => (Some(Color32::RED), None),
            SelectedButton::Keyboard(sbtn) if sbtn == btn => (None, Some(Color32::RED)),
            _ => (None, None),
        };

        if ui
            .add(Button::new(format!("{:?}", bind.ctrl)).text_color_opt(ctrl_color))
            .clicked()
        {
            if *selected == SelectedButton::Controller(btn) {
                *selected = SelectedButton::None;
            } else {
                *selected = SelectedButton::Controller(btn);
            }
        }

        if ui
            .add(Button::new(format!("{:?}", bind.kbd)).text_color_opt(kbd_color))
            .clicked()
        {
            if *selected == SelectedButton::Keyboard(btn) {
                *selected = SelectedButton::None;
            } else {
                *selected = SelectedButton::Keyboard(btn);
            }
        }

        ui.end_row();
    }

    fn change_keybind(
        gilrs: &mut Option<Gilrs>,
        binds: &mut Keybinds,
        selected: &mut SelectedButton,
    ) {
        match *selected {
            SelectedButton::None => (),
            SelectedButton::Controller(btn) => {
                if let Some(gilrs) = gilrs {
                    while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                        match event {
                            EventType::ButtonPressed(b, ..) => {
                                if !binds.ctrl_btn_used(b) {
                                    binds[btn].ctrl = b;
                                    *selected = SelectedButton::None;
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            SelectedButton::Keyboard(btn) => {
                if let Some(key) = get_last_key_pressed() {
                    if !binds.key_used(key) {
                        binds[btn].kbd = key;
                        *selected = SelectedButton::None;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq)]
enum SelectedButton {
    Controller(NesButton),
    Keyboard(NesButton),
    None,
}

impl Gui for KeybindsUi {
    fn gui_window(app: &mut App, egui_ctx: &egui::CtxRef) {
        let window_shown = &mut app.settings.keybinds.window_shown;
        let selected = &mut app.settings.keybinds.selected;

        let binds = &mut app.config.keybinds;
        let gilrs = &mut app.gilrs;

        egui::Window::new("Key bindings")
            .open(window_shown)
            .resizable(false)
            .show(egui_ctx, |ui| {
                egui::Grid::new("Key Bindingss Grid")
                    .striped(true)
                    .spacing([20., 5.])
                    .show(ui, |ui| {
                        ui.add(Label::new("NES button").heading().strong());
                        ui.add(Label::new("Controller").heading().strong());
                        ui.add(Label::new("Keyboard").heading().strong());
                        ui.end_row();

                        KeybindsUi::display_keybind(binds, NesButton::A, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::B, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::Start, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::Select, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::Up, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::Right, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::Down, ui, selected);
                        KeybindsUi::display_keybind(binds, NesButton::Left, ui, selected);
                    })
            });

        KeybindsUi::change_keybind(gilrs, binds, selected);
    }
}
