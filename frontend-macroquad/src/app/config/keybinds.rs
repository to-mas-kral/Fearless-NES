use std::ops::{Index, IndexMut};

use gilrs::ev::Button as GButton;
use macroquad::prelude::KeyCode;
use serde::{Deserialize, Serialize};

use fearless_nes::Button as NesButton;

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
            a: Keys::new(GButton::East, KeyCode::F),
            b: Keys::new(GButton::West, KeyCode::D),
            start: Keys::new(GButton::Start, KeyCode::Enter),
            select: Keys::new(GButton::Select, KeyCode::Space),
            up: Keys::new(GButton::DPadUp, KeyCode::Up),
            right: Keys::new(GButton::DPadRight, KeyCode::Right),
            down: Keys::new(GButton::DPadDown, KeyCode::Down),
            left: Keys::new(GButton::DPadLeft, KeyCode::Left),
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

    pub fn key_used(&self, key: KeyCode) -> bool {
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
    #[serde(with = "KeyCodeDef")]
    /// Keyboard keybind
    pub kbd: KeyCode,
}

impl Keys {
    pub fn new(ctrl: GButton, kbd: KeyCode) -> Self {
        Self { ctrl, kbd }
    }
}

/* Serde workaround - Macroquad's Keycode doesn't implement Serialize... */

#[derive(Serialize, Deserialize)]
#[serde(remote = "KeyCode")]
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
#[repr(u32)]
pub enum KeyCodeDef {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    Unknown,
}
