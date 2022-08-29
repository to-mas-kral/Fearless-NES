use egui_glium::egui_winit::egui;
use std::{fs, io::Write};

use fearless_nes::ReplayInputs;

use crate::app::get_save_named_path;
use crate::dialog::{report_error, DialogReport};

use super::RuntimeNes;

pub enum Recording {
    On { replay_inputs: ReplayInputs },
    Off,
}

pub struct Replays {
    pub recording: Recording,
}

impl Replays {
    pub fn new() -> Self {
        Self {
            recording: Recording::Off,
        }
    }

    pub fn start_recording(&mut self) {
        self.recording = Recording::On {
            replay_inputs: ReplayInputs::new(),
        };
    }

    pub fn stop_recording(&mut self, nes: &RuntimeNes) {
        // We can unwrap app.nes, because recording can only exist if we have
        // a Nes instance
        let nes = nes.as_ref().unwrap();
        let nes = nes.lock().unwrap();

        let save_path =
            match get_save_named_path(None, "FearLess-NES recorded inputs", "fnesinputs") {
                Some(p) => p,
                None => return,
            };

        match &mut self.recording {
            Recording::On { replay_inputs } => {
                let inputs = match replay_inputs
                    .save_with_end_frame(nes.get_frame_count())
                    .report_dialog_msg("Couldn't save the recording")
                {
                    Ok(i) => i,
                    Err(_) => return,
                };

                let mut save_file = fs::File::create(save_path).unwrap();
                save_file
                    .write_all(&inputs)
                    .report_dialog_msg("Couldn't save the recording")
                    .ok();

                self.recording = Recording::Off;
            }
            Recording::Off => report_error("No started recording to save"),
        }
    }
}

impl Replays {
    pub fn gui_embed(app: &mut super::App, ui: &mut egui::Ui) {
        match &mut app.replays.recording {
            Recording::On { .. } => {
                if ui.button("Stop recording").clicked() {
                    app.replays.stop_recording(&app.nes);
                }
            }
            Recording::Off => {
                if ui.button("Start recording").clicked() {
                    app.replays.start_recording();
                }
            }
        }
    }
}
