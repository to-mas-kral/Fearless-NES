use egui_glium::egui_winit::egui::{self, Button, RichText, Ui};
use gilrs::Button as GButton;
use winit::event::VirtualKeyCode;

use crate::app::config::Keybinds;

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

    pub fn gui_window(app: &mut crate::App, egui_ctx: &egui::Context) {
        OverscanUi::gui_window(app, egui_ctx);
        KeybindsUi::gui_window(app, egui_ctx);
    }

    pub fn gui_embed(app: &mut App, ui: &mut Ui) {
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

    pub fn gui_window(app: &mut crate::App, egui_ctx: &egui::Context) {
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
    pub selected_nesbtn: SelectedButton,
    pub input_btn: Option<InputButton>,
}

impl KeybindsUi {
    pub fn new() -> Self {
        Self {
            window_shown: false,
            selected_nesbtn: SelectedButton::None,
            input_btn: None,
        }
    }

    fn display_keybind(
        binds: &mut Keybinds,
        btn: NesButton,
        ui: &mut Ui,
        selected: &mut SelectedButton,
    ) {
        let bind = &mut binds[btn];

        ui.label(btn.to_string());

        let txt_color = ui.style().visuals.text_color();
        let highlight_color = ui.style().visuals.warn_fg_color;

        let (ctrl_color, kbd_color) = match *selected {
            SelectedButton::Controller(sbtn) if sbtn == btn => (highlight_color, txt_color),
            SelectedButton::Keyboard(sbtn) if sbtn == btn => (txt_color, highlight_color),
            _ => (txt_color, txt_color),
        };

        if ui
            .add(Button::new(
                RichText::new(format!("{:?}", bind.ctrl)).color(ctrl_color),
            ))
            .clicked()
        {
            if *selected == SelectedButton::Controller(btn) {
                *selected = SelectedButton::None;
            } else {
                *selected = SelectedButton::Controller(btn);
            }
        }

        if ui
            .add(Button::new(
                RichText::new(format!("{:?}", bind.kbd)).color(kbd_color),
            ))
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
        binds: &mut Keybinds,
        selected: &mut SelectedButton,
        input_button: &mut Option<InputButton>,
    ) {
        match *selected {
            SelectedButton::None => (),
            SelectedButton::Controller(btn) => {
                if let Some(InputButton::Gamepad(b)) = input_button {
                    if !binds.ctrl_btn_used(*b) {
                        binds[btn].ctrl = *b;
                        *selected = SelectedButton::None;
                        *input_button = None;
                    }
                }
            }
            SelectedButton::Keyboard(btn) => {
                if let Some(InputButton::Keyboard(key)) = input_button {
                    if !binds.key_used(*key) {
                        binds[btn].kbd = *key;
                        *selected = SelectedButton::None;
                        *input_button = None;
                    }
                }
            }
        }
    }

    pub fn gui_window(app: &mut App, egui_ctx: &egui::Context) {
        let window_shown = &mut app.settings.keybinds.window_shown;
        let selected = &mut app.settings.keybinds.selected_nesbtn;
        let input_button = &mut app.settings.keybinds.input_btn;
        let binds = &mut app.config.keybinds;

        egui::Window::new("Key bindings")
            .open(window_shown)
            .resizable(false)
            .show(egui_ctx, |ui| {
                egui::Grid::new("Key Bindingss Grid")
                    .striped(true)
                    .spacing([20., 5.])
                    .show(ui, |ui| {
                        ui.label(RichText::new("NES button").heading().strong());
                        ui.label(RichText::new("Controller").heading().strong());
                        ui.label(RichText::new("Keyboard").heading().strong());
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

        KeybindsUi::change_keybind(binds, selected, input_button);
    }
}

#[derive(PartialEq)]
pub enum SelectedButton {
    Controller(NesButton),
    Keyboard(NesButton),
    None,
}

pub enum InputButton {
    Gamepad(GButton),
    Keyboard(VirtualKeyCode),
}
