use std::path::{Path, PathBuf};

use egui::{CtxRef, FontDefinitions, FontFamily, Ui};
use gilrs::{Axis, Button as GButton, EventType, Gilrs};

use fearless_nes::{Button as NesButton, Nes};

mod config;
mod debug;
mod nesrender;
mod replays;
mod saves;

pub use config::Config;
use debug::Debug;
use macroquad::prelude::{is_key_pressed, is_key_released, KeyCode};
use native_dialog::FileDialog;
use nesrender::NesRender;
pub use replays::{Recording, Replays};
pub use saves::Saves;

use crate::create_nes;

pub struct App {
    pub config: Config,

    pub nes: Option<Nes>,

    pub render: NesRender,

    pub paused: bool,

    pub saves: Saves,
    pub debug: Debug,
    pub replays: Replays,

    _last_mouse_pos: (f32, f32),
    /// Frame count
    _mouse_not_moved: u32,
}

impl App {
    pub fn new(config: Config) -> Self {
        let app = Self {
            config,

            nes: None,
            paused: false,

            render: NesRender::new(),
            saves: Saves::new(),
            debug: Debug::new(),
            replays: Replays::new(),

            _last_mouse_pos: (0., 0.),
            _mouse_not_moved: 0,
        };

        app.init_egui_style();

        app
    }

    pub fn run_nes_frame(&mut self) {
        if let Some(nes) = &mut self.nes {
            if !self.paused {
                use std::time::Instant;

                let start = Instant::now();
                nes.run_one_frame();
                let duration = start.elapsed();

                self.debug.perf.add_frame_time(duration.as_millis());
            }

            self.render.update_frame(nes.get_frame_buffer());
        }
    }

    pub fn draw_nes(&mut self) {
        self.render.draw_nes();
    }

    #[rustfmt::skip]
    pub fn handle_input(&mut self, gilrs: &mut Option<Gilrs>) {
        // TODO: auto-hide mouse cursor and GUI - show_mouse(false) isn't working...
        /* let current_mouse_pos = mouse_position();
        if current_mouse_pos != self.last_mouse_pos {
            self.last_mouse_pos = current_mouse_pos;
            self.mouse_not_moved = 0;
            //show_mouse(true);
        } else {
            self.mouse_not_moved += 1;
        }

        if self.mouse_not_moved >= 300 {
            show_mouse(false);
        } */

        if let Some(gilrs) = gilrs {
            while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(button, ..) => match button {
                        GButton::East => self.set_button_state(NesButton::A, true),
                        GButton::West => self.set_button_state(NesButton::B, true),
                        GButton::Start => self.set_button_state(NesButton::Start, true),
                        GButton::Select => self.set_button_state(NesButton::Select, true),
                        GButton::DPadUp => self.set_button_state(NesButton::Up, true),
                        GButton::DPadRight => self.set_button_state(NesButton::Right, true),
                        GButton::DPadDown => self.set_button_state(NesButton::Down, true),
                        GButton::DPadLeft => self.set_button_state(NesButton::Left, true),
                        GButton::Mode => {
                            self.paused = !self.paused;
                        }
                        _ => (),
                    },
                    EventType::ButtonReleased(button, ..) => match button {
                        GButton::East => self.set_button_state(NesButton::A, false),
                        GButton::West => self.set_button_state(NesButton::B, false),
                        GButton::Start => self.set_button_state(NesButton::Start, false),
                        GButton::Select => self.set_button_state(NesButton::Select, false),
                        GButton::DPadUp => self.set_button_state(NesButton::Up, false),
                        GButton::DPadRight => self.set_button_state(NesButton::Right, false),
                        GButton::DPadDown => self.set_button_state(NesButton::Down, false),
                        GButton::DPadLeft => self.set_button_state(NesButton::Left, false),
                        _ => (),
                    },
                    EventType::AxisChanged(Axis::LeftStickX, val, ..) => {
                        if val > 0.5 {
                            self.set_button_state(NesButton::Right, true);
                        } else if val < -0.5 {
                            self.set_button_state(NesButton::Left, true);
                        } else {
                            self.set_button_state(NesButton::Right, false);
                            self.set_button_state(NesButton::Left, false);
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickY, val, ..) => {
                        if val > 0.5 {
                            self.set_button_state(NesButton::Up, true);
                        } else if val < -0.5 {
                            self.set_button_state(NesButton::Down, true);
                        } else {
                            self.set_button_state(NesButton::Up, false);
                            self.set_button_state(NesButton::Down, false);
                        }
                    }
                    _ => (),
                }
            }
        }

        if is_key_pressed(KeyCode::Up) { self.set_button_state(NesButton::Up, true) }
        if is_key_pressed(KeyCode::Right) { self.set_button_state(NesButton::Right, true) }
        if is_key_pressed(KeyCode::Down) { self.set_button_state(NesButton::Down, true) }
        if is_key_pressed(KeyCode::Left) { self.set_button_state(NesButton::Left, true) }
        if is_key_pressed(KeyCode::Enter) { self.set_button_state(NesButton::Start, true) }
        if is_key_pressed(KeyCode::Space) { self.set_button_state(NesButton::Select, true) }
        if is_key_pressed(KeyCode::D) { self.set_button_state(NesButton::B, true) }
        if is_key_pressed(KeyCode::F) { self.set_button_state(NesButton::A, true) }

        if is_key_released(KeyCode::Up) { self.set_button_state(NesButton::Up, false) }
        if is_key_released(KeyCode::Right) { self.set_button_state(NesButton::Right, false) }
        if is_key_released(KeyCode::Down) { self.set_button_state(NesButton::Down, false) }
        if is_key_released(KeyCode::Left) { self.set_button_state(NesButton::Left, false) }
        if is_key_released(KeyCode::Enter) { self.set_button_state(NesButton::Start, false) }
        if is_key_released(KeyCode::Space) { self.set_button_state(NesButton::Select, false) }
        if is_key_released(KeyCode::D) { self.set_button_state(NesButton::B, false) }
        if is_key_released(KeyCode::F) { self.set_button_state(NesButton::A, false) }
    }

    fn set_button_state(&mut self, button: NesButton, state: bool) {
        if let Some(nes) = &mut self.nes {
            nes.set_button_state(button.clone(), state);

            if let Recording::On { replay_inputs, .. } = &mut self.replays.recording {
                replay_inputs.add_input_change(nes.get_frame_count(), button, state);
            }
        };
    }

    pub fn draw_gui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            App::gui_window(self, egui_ctx);
            Saves::gui_window(self, egui_ctx);
            Debug::gui_window(self, egui_ctx);
        });
    }

    pub fn init_egui_style(&self) {
        egui_macroquad::cfg(|egui_ctx| {
            if self.config.dark_mode {
                egui_ctx.set_visuals(egui::Visuals::dark());
            } else {
                egui_ctx.set_visuals(egui::Visuals::light());
            };

            let mut fonts = FontDefinitions::default();

            fonts.font_data.insert(
                "bold".to_owned(),
                std::borrow::Cow::Borrowed(include_bytes!("Quicksand-Bold.ttf")),
            );

            // Put my font first (highest priority):
            fonts
                .fonts_for_family
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "bold".to_owned());

            // Put my font as last fallback for monospace:
            fonts
                .fonts_for_family
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .push("bold".to_owned());

            egui_ctx.set_fonts(fonts);
        });
    }
}

impl Gui for App {
    fn gui_window(app: &mut App, egui_ctx: &CtxRef) {
        egui::TopBottomPanel::top("TopPanel").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Load ROM").clicked() {
                    loop {
                        let rom_path = match get_open_file_path(Some(&app.config.rom_folder_path)) {
                            Ok(Some(p)) => {
                                app.config.rom_folder_path = p.clone();
                                app.config.rom_folder_path.pop(); // Remove the file, we only want the folder
                                p
                            }
                            Ok(None) => break,
                            Err(_) => return,
                        };

                        match create_nes(rom_path) {
                            Ok(n) => {
                                app.nes = Some(n);
                                break;
                            }
                            Err(_) => continue,
                        }
                    }
                }

                egui::menu::menu(ui, "Saves", |ui| {
                    if app.nes.is_some() && ui.button("Save").clicked() {
                        if let Err(e) = app.saves.create_save(
                            &app.nes,
                            &mut app.render,
                            &app.config.save_folder_path,
                        ) {
                            report_error(&format!("{}", e));
                        }
                    }

                    if ui.button("Load saves from folder").clicked() {
                        match get_folder_path(Some(app.config.save_folder_path.as_ref())) {
                            Ok(Some(p)) => {
                                app.saves.window_shown = true;
                                app.saves.folder_path = p.clone();
                                app.saves.folder_changed = true;

                                app.config.save_folder_path = p;
                            }
                            Ok(None) => (),
                            Err(_) => return,
                        }
                    }
                });

                egui::menu::menu(ui, "Settings", |ui| {
                    let mode_text = if app.config.dark_mode {
                        "Light mode"
                    } else {
                        "Dark mode"
                    };

                    if ui.button(mode_text).clicked() {
                        app.config.dark_mode = !app.config.dark_mode;
                        if app.config.dark_mode {
                            egui_ctx.set_visuals(egui::Visuals::dark());
                        } else {
                            egui_ctx.set_visuals(egui::Visuals::light());
                        };
                    }
                });

                if let Some(_) = app.nes {
                    egui::menu::menu(ui, "Controls", |ui| {
                        let button_text = match app.paused {
                            true => "Resume",
                            false => "Pause",
                        };

                        if ui.button(button_text).clicked() {
                            app.paused = !app.paused;
                        }

                        if ui.button("Reset").clicked() {
                            // Satisfy the borrow checker...
                            app.nes.as_mut().unwrap().reset();
                        }
                    });

                    egui::menu::menu(ui, "Debug", |ui| {
                        if ui.button("Controls and Status").clicked() {
                            app.debug.show_controls = true;
                        }

                        if ui.button("CPU State").clicked() {
                            app.debug.show_cpu = true;
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
                    });

                    egui::menu::menu(ui, "Record inputs", |ui| {
                        match &mut app.replays.recording {
                            Recording::On { .. } => {
                                if ui.button("Stop recording").clicked() {
                                    // We can unwrap app.nes, because recording can only exist if we have
                                    // a Nes instance
                                    app.replays.stop_recording(
                                        &mut app.paused,
                                        &app.nes.as_ref().unwrap(),
                                    );
                                }
                            }
                            Recording::Off => {
                                if ui.button("Start recording").clicked() {
                                    app.replays.start_recording();
                                }
                            }
                        }
                    });
                }
            });
        });
    }
}

/// Display given component using Egui
pub trait Gui {
    fn gui_window(_app: &mut App, _egui_ctx: &CtxRef) {}
    fn gui_embed(_app: &mut App, _ui: &mut Ui) {}
}

fn get_folder_path(location: Option<&Path>) -> Result<Option<PathBuf>, ()> {
    match FileDialog::new()
        .set_location(location.unwrap_or(Path::new("~/")))
        .show_open_single_dir()
    {
        Ok(Some(p)) => return Ok(Some(p)),
        Ok(None) => return Ok(None),
        Err(_) => {
            report_error("Error while getting path from the user (needs KDialog on Linux");
            return Err(());
        }
    }
}

fn get_save_named_path(
    location: Option<&Path>,
    filter_desc: &str,
    filter_ext: &str,
) -> Result<Option<PathBuf>, ()> {
    match FileDialog::new()
        .set_location(location.unwrap_or(Path::new("~/")))
        .add_filter(filter_desc, &[filter_ext])
        .show_save_single_file()
    {
        Ok(Some(p)) => return Ok(Some(p)),
        Ok(None) => return Ok(None),
        Err(_) => {
            report_error("Error while getting path from the user (needs KDialog on Linux");
            return Err(());
        }
    }
}

fn get_open_file_path(location: Option<&Path>) -> Result<Option<PathBuf>, ()> {
    match FileDialog::new()
        .set_location(location.unwrap_or(Path::new("~/")))
        .show_open_single_file()
    {
        Ok(Some(p)) => return Ok(Some(p)),
        Ok(None) => return Ok(None),
        Err(_) => {
            report_error("Error while getting path from the user (needs KDialog on Linux");
            return Err(());
        }
    }
}

pub fn report_error(msg: &str) {
    native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Info)
        .set_title("Error")
        .set_text(msg)
        .show_alert()
        .expect("Error while displaying an error message box (needs KDialog on Linux)");
}
