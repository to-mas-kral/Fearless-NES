use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crossbeam::channel::Receiver;
use fearless_nes::Nes;

pub enum NesMsg {
    Pause,
    Unpause,
    Exit,
}

struct State {
    // TODO(rewrite): pause the emulator while dialogs are open...
    paused: bool,
}

impl State {
    fn new() -> Self {
        Self { paused: false }
    }
}

// The NTSC PPU runs at 60.0988 Hz
const FRAME_DURATION: Duration = Duration::from_nanos(16639267);

pub fn run_nes_thread(nes: Arc<Mutex<Nes>>, channel: Receiver<NesMsg>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut state = State::new();

        loop {
            let deadline = Instant::now() + FRAME_DURATION;

            for msg in channel.try_iter() {
                match msg {
                    NesMsg::Exit => return,
                    NesMsg::Pause => state.paused = true,
                    NesMsg::Unpause => state.paused = false,
                }
            }

            // TODO(rewrite): performance monitoring
            if !state.paused {
                let mut n = nes.lock().unwrap();
                n.run_one_frame();
            }

            let now = Instant::now();
            if now > deadline {
                println!("WARNING: missed a NES frame deadline");
            } else {
                let delta = deadline.duration_since(now);
                spin_sleep::sleep(delta);
            }
        }
    })
}
