use bincode::{
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

use crate::{controller::Button, Nes};

#[derive(Encode, Decode)]
pub struct ReplayInputs {
    pub inputs: Vec<InputChange>,
    pub end_frame: u64,
}

impl ReplayInputs {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            end_frame: 0,
        }
    }

    pub fn add_input_change(&mut self, frame: u64, button: Button, state: bool) {
        self.inputs.push(InputChange {
            frame,
            button,
            state,
        });
    }

    pub fn save_with_end_frame(&mut self, end_frame: u64) -> Result<Vec<u8>, EncodeError> {
        self.end_frame = end_frame;
        bincode::encode_to_vec(&*self, crate::BINCODE_CONFIG)
    }

    pub fn load_state(save: &[u8]) -> Result<ReplayInputs, DecodeError> {
        let (replay_inputs, _) = bincode::decode_from_slice(save, crate::BINCODE_CONFIG)?;
        Ok(replay_inputs)
    }
}

#[derive(Encode, Decode)]
pub struct InputChange {
    pub frame: u64,
    pub button: Button,
    pub state: bool,
}

impl Nes {
    pub(crate) fn _drive_replay_inputs(&mut self, inputs: &ReplayInputs) {
        for ic in &inputs.inputs {
            while self.frame_count < ic.frame {
                self.run_frame();
            }

            self.set_button_state(ic.button, ic.state);
        }

        while self.frame_count < inputs.end_frame {
            self.run_frame();
        }
    }
}
