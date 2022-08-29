use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use crossbeam::channel::Sender;
use egui_glium::egui_winit::egui::{self, FontDefinitions, FontFamily};
use eyre::Result;
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
use native_dialog::FileDialog;
use nesrender::NesRender;
pub use replays::{Recording, Replays};
pub use saves::Saves;
use settings::Settings;
use winit::event::{ElementState, KeyboardInput};

use crate::{
    dialog::{report_error, DialogReport},
    nesthread::{self, NesMsg},
};

use self::settings::{InputButton, SelectedButton};

type RuntimeNes = Option<Arc<Mutex<Nes>>>;

const CHANNEL_TIMEOUT: Duration = Duration::from_secs(1);

pub struct App {
    pub config: Config,

    nes: RuntimeNes,
    nes_channel: Option<Sender<NesMsg>>,

    paused: bool,

    render: NesRender,
    saves: Saves,
    debug: Debug,
    replays: Replays,
    settings: Settings,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,

            nes: None,
            nes_channel: None,

            paused: false,

            render: NesRender::new(),
            saves: Saves::new(),
            debug: Debug::new(),
            replays: Replays::new(),
            settings: Settings::new(),
        }
    }

    // TODO(high): refactor this... need to handle this in the core instead
    pub fn create_nes_with_file(&mut self, rom_path: PathBuf) -> Result<()> {
        let rom = fs::read(rom_path).report_dialog_msg("Error while reading the ROM file")?;
        let new_nes = Nes::new(&rom)
            .report_dialog_with(|e| format!("Error while loading the ROM: {:?}", e))?;

        self.replace_nes(new_nes);

        Ok(())
    }

    pub fn replace_nes(&mut self, new_nes: Nes) {
        if let Some(nes) = &self.nes {
            let mut nes = nes.lock().unwrap();
            *nes = new_nes;
        } else {
            let nes = Arc::new(Mutex::new(new_nes));
            let (send, recv) = crossbeam::channel::bounded::<NesMsg>(1024);

            self.nes = Some(Arc::clone(&nes));
            self.nes_channel = Some(send);

            std::thread::spawn(move || {
                nesthread::run_nes_thread(Arc::clone(&nes), recv);
            });
        }
    }

    #[rustfmt::skip]
    pub fn handle_keyboard_input(&mut self, input: KeyboardInput) {
        let state = input.state == ElementState::Pressed;

        if let SelectedButton::Keyboard(_) = self.settings.keybinds.selected_nesbtn {
            if let Some(keycode) = input.virtual_keycode {
                if input.state == ElementState::Pressed {
                    self.settings.keybinds.input_btn = Some(InputButton::Keyboard(keycode));
                    return;
                }
            }
        }

        if let Some(keycode) = input.virtual_keycode {
            if keycode == self.config.keybinds.up.kbd { self.set_button(NesButton::Up, state) }
            if keycode == self.config.keybinds.right.kbd { self.set_button(NesButton::Right, state) }
            if keycode == self.config.keybinds.down.kbd { self.set_button(NesButton::Down, state) }
            if keycode == self.config.keybinds.left.kbd { self.set_button(NesButton::Left, state) }
            if keycode == self.config.keybinds.start.kbd { self.set_button(NesButton::Start, state) }
            if keycode == self.config.keybinds.select.kbd { self.set_button(NesButton::Select, state) }
            if keycode == self.config.keybinds.b.kbd { self.set_button(NesButton::B, state) }
            if keycode == self.config.keybinds.a.kbd { self.set_button(NesButton::A, state) }
        }
    }

    #[rustfmt::skip]
    pub fn handle_gamepad_input(&mut self, gilrs: &mut Gilrs) {
        while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(b, ..) => {
                    if let SelectedButton::Controller(_) = self.settings.keybinds.selected_nesbtn {
                        self.settings.keybinds.input_btn = Some(InputButton::Gamepad(b));
                        continue;
                    }

                    if b == self.config.keybinds.a.ctrl { self.set_button(NesButton::A, true) }
                    if b == self.config.keybinds.b.ctrl { self.set_button(NesButton::B, true) }
                    if b == self.config.keybinds.start.ctrl { self.set_button(NesButton::Start, true) }
                    if b == self.config.keybinds.select.ctrl { self.set_button(NesButton::Select, true) }
                    if b == self.config.keybinds.up.ctrl { self.set_button(NesButton::Up, true) }
                    if b == self.config.keybinds.right.ctrl { self.set_button(NesButton::Right, true) }
                    if b == self.config.keybinds.down.ctrl { self.set_button(NesButton::Down, true) }
                    if b == self.config.keybinds.left.ctrl { self.set_button(NesButton::Left, true) }
                }
                EventType::ButtonReleased(b, ..) => {
                    if b == self.config.keybinds.a.ctrl { self.set_button(NesButton::A, false) }
                    if b == self.config.keybinds.b.ctrl { self.set_button(NesButton::B, false) }
                    if b == self.config.keybinds.start.ctrl { self.set_button(NesButton::Start, false) }
                    if b == self.config.keybinds.select.ctrl { self.set_button(NesButton::Select, false) }
                    if b == self.config.keybinds.up.ctrl { self.set_button(NesButton::Up, false) }
                    if b == self.config.keybinds.right.ctrl { self.set_button(NesButton::Right, false) }
                    if b == self.config.keybinds.down.ctrl { self.set_button(NesButton::Down, false) }
                    if b == self.config.keybinds.left.ctrl { self.set_button(NesButton::Left, false) }
                }
                EventType::AxisChanged(Axis::LeftStickX, val, ..) => {
                    if val > 0.5 {
                        self.set_button(NesButton::Right, true);
                    } else if val < -0.5 {
                        self.set_button(NesButton::Left, true);
                    } else {
                        self.set_button(NesButton::Right, false);
                        self.set_button(NesButton::Left, false);
                    }
                }
                EventType::AxisChanged(Axis::LeftStickY, val, ..) => {
                    if val > 0.5 {
                        self.set_button(NesButton::Up, true);
                    } else if val < -0.5 {
                        self.set_button(NesButton::Down, true);
                    } else {
                        self.set_button(NesButton::Up, false);
                        self.set_button(NesButton::Down, false);
                    }
                }
                _ => (),
            }
        }
    }

    fn set_button(&mut self, button: NesButton, state: bool) {
        if let Some(nes) = &mut self.nes {
            let mut nes = nes.lock().unwrap();

            nes.set_button_state(button, state);

            if let Recording::On { replay_inputs, .. } = &mut self.replays.recording {
                replay_inputs.add_input_change(nes.get_frame_count(), button, state);
            }
        };
    }

    pub fn draw_gui(&mut self, egui_ctx: &egui::Context) {
        App::gui(self, egui_ctx);
        Saves::gui_window(self, egui_ctx);
        Debug::gui_window(self, egui_ctx);
        Settings::gui_window(self, egui_ctx);

        if let Some(nes) = &self.nes {
            let nes = nes.lock().unwrap();

            let nes_framebuffer = nes.get_frame_buffer();
            self.render
                .draw_nes(nes_framebuffer, egui_ctx, &self.config.overscan);
        }
    }

    pub fn init_egui_style(&self, egui_ctx: &egui::Context) {
        if self.config.dark_mode {
            egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            egui_ctx.set_visuals(egui::Visuals::light());
        };

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "normal".to_owned(),
            egui::FontData::from_static(include_bytes!("Quicksand-Bold.ttf")),
        );

        fonts.font_data.insert(
            "monospace".to_owned(),
            egui::FontData::from_static(include_bytes!("SourceCodePro-Bold.ttf")),
        );

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "normal".to_owned());

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "monospace".to_owned());

        egui_ctx.set_fonts(fonts);
    }

    fn load_rom(&mut self) {
        loop {
            let rom_path = match get_open_file_path(Some(&self.config.rom_folder_path)) {
                Some(p) => {
                    self.config.rom_folder_path = p.clone();
                    self.config.rom_folder_path.pop(); // Remove the file, we only want the folder
                    p
                }
                None => break,
            };

            match self.create_nes_with_file(rom_path) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    fn gui(app: &mut App, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("TopPanel").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Load ROM").clicked() {
                    app.load_rom();
                }

                egui::menu::menu_button(ui, "Saves", |ui| {
                    Saves::gui_embed(app, ui);
                });

                if let Some(nes) = &app.nes {
                    egui::menu::menu_button(ui, "Controls", |ui| {
                        let button_text = match app.paused {
                            true => "Resume",
                            false => "Pause",
                        };

                        if ui.button(button_text).clicked() {
                            app.paused = !app.paused;
                            if let Some(channel) = &app.nes_channel {
                                let msg = match app.paused {
                                    true => NesMsg::Pause,
                                    false => NesMsg::Unpause,
                                };

                                channel
                                    .send_timeout(msg, CHANNEL_TIMEOUT)
                                    .report_dialog()
                                    .ok();
                            }
                        }

                        if ui.button("Reset").clicked() {
                            // Satisfy the borrow checker...
                            let mut nes = nes.lock().unwrap();
                            nes.reset();
                        }
                    });

                    egui::menu::menu_button(ui, "Debug", |ui| {
                        Debug::gui_embed(app, ui);
                    });

                    egui::menu::menu_button(ui, "Record inputs", |ui| {
                        Replays::gui_embed(app, ui);
                    });

                    egui::menu::menu_button(ui, "Settings", |ui| {
                        Settings::gui_embed(app, ui);
                        if app.config.dark_mode {
                            egui_ctx.set_visuals(egui::Visuals::dark());
                        } else {
                            egui_ctx.set_visuals(egui::Visuals::light());
                        };
                    });
                }
            });
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if self.nes.is_some() {
            if let Some(channel) = &self.nes_channel {
                channel.send(NesMsg::Exit).unwrap();
            }
        }
    }
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
) -> Option<PathBuf> {
    match FileDialog::new()
        .set_location(location.unwrap_or_else(|| Path::new("~/")))
        .add_filter(filter_desc, &[filter_ext])
        .show_save_single_file()
    {
        Ok(Some(p)) => return Some(p),
        Ok(None) => return None,
        Err(_) => {
            report_error("Error while getting path from the user (needs KDialog on Linux");
            return None;
        }
    }
}

fn get_open_file_path(location: Option<&Path>) -> Option<PathBuf> {
    match FileDialog::new()
        .set_location(location.unwrap_or_else(|| Path::new("~/")))
        .show_open_single_file()
    {
        Ok(Some(p)) => return Some(p),
        Ok(None) => return None,
        Err(_) => {
            report_error("Error while getting path from the user (needs KDialog on Linux");
            return None;
        }
    }
}
