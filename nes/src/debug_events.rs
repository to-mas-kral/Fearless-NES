use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct DebugEvents {
    events: [Vec<DebugEvent>; 2],
    current_frame: u8,
}

impl DebugEvents {
    pub fn new() -> Self {
        Self {
            events: [Vec::with_capacity(128), Vec::with_capacity(128)],
            current_frame: 0,
        }
    }

    pub fn add(&mut self, kind: EventKind, scanline: u16, xpos: u16) {
        let e = DebugEvent {
            kind,
            scanline,
            xpos,
        };

        // TODO: have some sort of filter
        self.events[self.current_frame as usize].push(e);
    }

    pub fn on_frame_ended(&mut self) {
        if self.current_frame == 0 {
            self.current_frame = 1;
        } else {
            self.current_frame = 0;
        }

        self.events[self.current_frame as usize].clear();
    }
}

#[derive(Encode, Decode)]
pub struct DebugEvent {
    kind: EventKind,
    scanline: u16,
    xpos: u16,
}

#[derive(Encode, Decode, Debug)]
pub enum EventKind {
    Irq,
}
