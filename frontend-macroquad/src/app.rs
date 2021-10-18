use std::path::{Path, PathBuf};

use egui::{CtxRef, FontDefinitions, FontFamily, Ui};
use gilrs::{Axis, EventType, Gilrs};

use fearless_nes::{Button as NesButton, Nes};

mod config;
mod debug;
mod nesrender;
mod replays;
mod saves;
mod settings;

pub use config::Config;
use debug::Debug;
use macroquad::prelude::{is_key_pressed, is_key_released, mouse_position, show_mouse};
use native_dialog::FileDialog;
use nesrender::NesRender;
pub use replays::{Recording, Replays};
pub use saves::Saves;
use settings::Settings;

use crate::create_nes;

pub struct App {
    pub config: Config,

    pub nes: Option<Nes>,

    pub render: NesRender,

    pub paused: bool,

    pub saves: Saves,
    pub debug: Debug,
    pub replays: Replays,
    pub settings: Settings,

    gilrs: Option<Gilrs>,

    last_mouse_pos: (f32, f32),
    /// Frame count
    mouse_not_moved: u32,
}

impl App {
    pub fn new(config: Config) -> Self {
        let gilrs = Gilrs::new().map(|g| Some(g)).unwrap_or_else(|_| {
            report_error("Couldn't initialize gamepad input library");
            None
        });

        let app = Self {
            config,

            nes: None,
            paused: false,

            render: NesRender::new(),
            saves: Saves::new(),
            debug: Debug::new(),
            replays: Replays::new(),
            settings: Settings::new(),

            gilrs,

            last_mouse_pos: (0., 0.),
            mouse_not_moved: 0,
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

            self.render
                .update_frame(nes.get_frame_buffer(), &self.config.overscan);
        }
    }

    pub fn draw_nes(&mut self) {
        self.render.draw_nes();
    }

    const HIDE_MOUSE_FRAMES: u32 = 180;

    #[rustfmt::skip]
    pub fn handle_input(&mut self) {
        let current_mouse_pos = mouse_position();
        if current_mouse_pos != self.last_mouse_pos {
            self.last_mouse_pos = current_mouse_pos;
            self.mouse_not_moved = 0;
            show_mouse(true);
        } else {
            self.mouse_not_moved += 1;
        }

        if self.mouse_not_moved >= Self::HIDE_MOUSE_FRAMES {
            show_mouse(false);
        }

        let gilrs = &mut self.gilrs;
        let nes = &mut self.nes;
        let replays = &mut self.replays;

        let binds = &self.config.keybinds;

        if let Some(gilrs) = gilrs {
            while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(b, ..) => {
                        if b == binds.a.ctrl { Self::set_button(nes, replays, NesButton::A, true) }
                        if b == binds.b.ctrl { Self::set_button(nes, replays, NesButton::B, true) }
                        if b == binds.start.ctrl { Self::set_button(nes, replays, NesButton::Start, true) }
                        if b == binds.select.ctrl { Self::set_button(nes, replays, NesButton::Select, true) }
                        if b == binds.up.ctrl { Self::set_button(nes, replays, NesButton::Up, true) }
                        if b == binds.right.ctrl { Self::set_button(nes, replays, NesButton::Right, true) }
                        if b == binds.down.ctrl { Self::set_button(nes, replays, NesButton::Down, true) }
                        if b == binds.left.ctrl { Self::set_button(nes, replays, NesButton::Left, true) }
                    }
                    EventType::ButtonReleased(b, ..) => {
                        if b == binds.a.ctrl { Self::set_button(nes, replays, NesButton::A, false) }
                        if b == binds.b.ctrl { Self::set_button(nes, replays, NesButton::B, false) }
                        if b == binds.start.ctrl { Self::set_button(nes, replays, NesButton::Start, false) }
                        if b == binds.select.ctrl { Self::set_button(nes, replays, NesButton::Select, false) }
                        if b == binds.up.ctrl { Self::set_button(nes, replays, NesButton::Up, false) }
                        if b == binds.right.ctrl { Self::set_button(nes, replays, NesButton::Right, false) }
                        if b == binds.down.ctrl { Self::set_button(nes, replays, NesButton::Down, false) }
                        if b == binds.left.ctrl { Self::set_button(nes, replays, NesButton::Left, false) }
                    }
                    EventType::AxisChanged(Axis::LeftStickX, val, ..) => {
                        if val > 0.5 {
                            Self::set_button(nes, replays, NesButton::Right, true);
                        } else if val < -0.5 {
                            Self::set_button(nes, replays, NesButton::Left, true);
                        } else {
                            Self::set_button(nes, replays, NesButton::Right, false);
                            Self::set_button(nes, replays, NesButton::Left, false);
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickY, val, ..) => {
                        if val > 0.5 {
                            Self::set_button(nes, replays, NesButton::Up, true);
                        } else if val < -0.5 {
                            Self::set_button(nes, replays, NesButton::Down, true);
                        } else {
                            Self::set_button(nes, replays, NesButton::Up, false);
                            Self::set_button(nes, replays, NesButton::Down, false);
                        }
                    }
                    _ => (),
                }
            }
        }

        if is_key_pressed(binds.up.kbd) { Self::set_button(nes, replays, NesButton::Up, true) }
        if is_key_pressed(binds.right.kbd) { Self::set_button(nes, replays, NesButton::Right, true) }
        if is_key_pressed(binds.down.kbd) { Self::set_button(nes, replays, NesButton::Down, true) }
        if is_key_pressed(binds.left.kbd) { Self::set_button(nes, replays, NesButton::Left, true) }
        if is_key_pressed(binds.start.kbd) { Self::set_button(nes, replays, NesButton::Start, true) }
        if is_key_pressed(binds.select.kbd) { Self::set_button(nes, replays, NesButton::Select, true) }
        if is_key_pressed(binds.b.kbd) { Self::set_button(nes, replays, NesButton::B, true) }
        if is_key_pressed(binds.a.kbd) { Self::set_button(nes, replays, NesButton::A, true) }

        if is_key_released(binds.up.kbd) { Self::set_button(nes, replays, NesButton::Up, false) }
        if is_key_released(binds.right.kbd) { Self::set_button(nes, replays, NesButton::Right, false) }
        if is_key_released(binds.down.kbd) { Self::set_button(nes, replays, NesButton::Down, false) }
        if is_key_released(binds.left.kbd) { Self::set_button(nes, replays, NesButton::Left, false) }
        if is_key_released(binds.start.kbd) { Self::set_button(nes, replays, NesButton::Start, false) }
        if is_key_released(binds.select.kbd) { Self::set_button(nes, replays, NesButton::Select, false) }
        if is_key_released(binds.b.kbd) { Self::set_button(nes, replays, NesButton::B, false) }
        if is_key_released(binds.a.kbd) { Self::set_button(nes, replays, NesButton::A, false) }
    }

    fn set_button(nes: &mut Option<Nes>, replays: &mut Replays, button: NesButton, state: bool) {
        if let Some(nes) = nes {
            nes.set_button_state(button, state);

            if let Recording::On { replay_inputs, .. } = &mut replays.recording {
                replay_inputs.add_input_change(nes.get_frame_count(), button, state);
            }
        };
    }

    pub fn draw_gui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            App::gui_window(self, egui_ctx);
            Saves::gui_window(self, egui_ctx);
            Debug::gui_window(self, egui_ctx);
            Settings::gui_window(self, egui_ctx);
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
                "normal".to_owned(),
                std::borrow::Cow::Borrowed(include_bytes!("Quicksand-Bold.ttf")),
            );

            fonts.font_data.insert(
                "monospace".to_owned(),
                std::borrow::Cow::Borrowed(include_bytes!("SourceCodePro-Bold.ttf")),
            );

            fonts
                .fonts_for_family
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "normal".to_owned());

            fonts
                .fonts_for_family
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .insert(0, "monospace".to_owned());

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
                            report_error(&format!("Couldn't create the save file. Error: {}", e));
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

                if app.nes.is_some() {
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
                                        app.nes.as_ref().unwrap(),
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

                        if ui.button("Overscan").clicked() {
                            app.settings.overscan.window_shown = true;
                        }

                        if ui.button("Key bindings").clicked() {
                            app.settings.keybinds.window_shown = true;
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
        .set_location(location.unwrap_or_else(|| Path::new("~/")))
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
        .set_location(location.unwrap_or_else(|| Path::new("~/")))
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
        .set_location(location.unwrap_or_else(|| Path::new("~/")))
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
