use std::{fs, io::Write};

use fearless_nes::{Nes, ReplayInputs};

use crate::app::{get_save_named_path, report_error};

use super::Gui;

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

    pub fn stop_recording(&mut self, paused: &mut bool, nes: &Nes) {
        let save_path =
            match get_save_named_path(None, "FearLess-NES recorded inputs", "fnesinputs") {
                Ok(Some(p)) => p,
                Ok(None) | Err(_) => return,
            };

        match &mut self.recording {
            Recording::On { replay_inputs } => {
                let inputs = match replay_inputs.save_with_end_frame(nes.get_frame_count()) {
                    Ok(i) => i,
                    Err(_) => {
                        report_error("Couldn't save the recording");
                        return;
                    }
                };

                let mut save_file = fs::File::create(save_path).unwrap();
                if let Err(_) = save_file.write_all(&inputs) {
                    report_error("Couldn't save the recording");
                };

                *paused = true;
                self.recording = Recording::Off;
            }
            Recording::Off => report_error("No started recording to save"),
        }
    }
}

impl Gui for Replays {
    fn gui_embed(app: &mut super::App, ui: &mut egui::Ui) {
        match &mut app.replays.recording {
            Recording::On { .. } => {
                if ui.button("Stop recording").clicked() {
                    // We can unwrap app.nes, because recording can only exist if we have
                    // a Nes instance
                    app.replays
                        .stop_recording(&mut app.paused, app.nes.as_ref().unwrap());
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
