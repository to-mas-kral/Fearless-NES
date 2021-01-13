use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Controller {
    strobe: bool,
    shifter: u8,
    state: u8,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            strobe: false,
            shifter: 0,
            state: 0,
        }
    }

    #[inline]
    pub(crate) fn write_reg(&mut self, val: u8) {
        if self.strobe && (val & 1) == 0 {
            self.shifter = self.state;
        }

        self.strobe = (val & 1) != 0;
    }

    #[inline]
    pub(crate) fn read_reg(&mut self) -> u8 {
        if self.strobe {
            return self.state & 1;
        }

        let key = self.shifter & 1;
        self.shifter = 0x80 | (self.shifter >> 1);

        key
    }

    #[inline]
    pub fn set_button(&mut self, keycode: Keycode, state: bool) {
        self.state = match keycode {
            Keycode::A => (self.state & !1) | (state as u8),
            Keycode::B => (self.state & !(1 << 1)) | (state as u8) << 1,
            Keycode::Select => (self.state & !(1 << 2)) | (state as u8) << 2,
            Keycode::Start => (self.state & !(1 << 3)) | (state as u8) << 3,
            Keycode::Up => (self.state & !(1 << 4)) | (state as u8) << 4,
            Keycode::Down => (self.state & !(1 << 5)) | (state as u8) << 5,
            Keycode::Left => (self.state & !(1 << 6)) | (state as u8) << 6,
            Keycode::Right => (self.state & !(1 << 7)) | (state as u8) << 7,
        }
    }
}

pub enum Keycode {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}
