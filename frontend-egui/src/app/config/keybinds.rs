use fearless_nes::Button as NesButton;
use gilrs::ev::Button as GButton;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

// Need to use winit as a dependency because egui_winit doesnt compile winit with serde features
use winit::event::VirtualKeyCode;

#[derive(Serialize, Deserialize)]
pub struct Keybinds {
    pub a: Keys,
    pub b: Keys,
    pub start: Keys,
    pub select: Keys,
    pub up: Keys,
    pub right: Keys,
    pub down: Keys,
    pub left: Keys,
}

impl Keybinds {
    pub fn new() -> Self {
        Self {
            a: Keys::new(GButton::East, VirtualKeyCode::F),
            b: Keys::new(GButton::West, VirtualKeyCode::D),
            start: Keys::new(GButton::Start, VirtualKeyCode::Return),
            select: Keys::new(GButton::Select, VirtualKeyCode::Space),
            up: Keys::new(GButton::DPadUp, VirtualKeyCode::Up),
            right: Keys::new(GButton::DPadRight, VirtualKeyCode::Right),
            down: Keys::new(GButton::DPadDown, VirtualKeyCode::Down),
            left: Keys::new(GButton::DPadLeft, VirtualKeyCode::Left),
        }
    }

    pub fn ctrl_btn_used(&self, btn: GButton) -> bool {
        self.a.ctrl == btn
            || self.b.ctrl == btn
            || self.start.ctrl == btn
            || self.select.ctrl == btn
            || self.up.ctrl == btn
            || self.right.ctrl == btn
            || self.down.ctrl == btn
            || self.left.ctrl == btn
    }

    pub fn key_used(&self, key: VirtualKeyCode) -> bool {
        self.a.kbd == key
            || self.b.kbd == key
            || self.start.kbd == key
            || self.select.kbd == key
            || self.up.kbd == key
            || self.right.kbd == key
            || self.down.kbd == key
            || self.left.kbd == key
    }
}

impl Index<NesButton> for Keybinds {
    type Output = Keys;

    fn index(&self, btn: NesButton) -> &Self::Output {
        match btn {
            NesButton::A => &self.a,
            NesButton::B => &self.b,
            NesButton::Start => &self.start,
            NesButton::Select => &self.select,
            NesButton::Up => &self.up,
            NesButton::Right => &self.right,
            NesButton::Down => &self.down,
            NesButton::Left => &self.left,
        }
    }
}

impl IndexMut<NesButton> for Keybinds {
    fn index_mut(&mut self, btn: NesButton) -> &mut Self::Output {
        match btn {
            NesButton::A => &mut self.a,
            NesButton::B => &mut self.b,
            NesButton::Start => &mut self.start,
            NesButton::Select => &mut self.select,
            NesButton::Up => &mut self.up,
            NesButton::Right => &mut self.right,
            NesButton::Down => &mut self.down,
            NesButton::Left => &mut self.left,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Keys {
    /// Gamepad keybind
    pub ctrl: GButton,
    /// Keyboard keybind
    pub kbd: VirtualKeyCode,
}

impl Keys {
    pub fn new(ctrl: GButton, kbd: VirtualKeyCode) -> Self {
        Self { ctrl, kbd }
    }
}
