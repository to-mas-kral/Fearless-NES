use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use cpal::SampleRate;
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

        let (audio_send, audio_recv) = crossbeam::channel::unbounded::<f32>();

        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_sample_rate(SampleRate(44100))
            .into();

        let stream = device
            .build_output_stream(
                &supported_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let requested = data.len();
                    let len = audio_recv.len();

                    if len < requested {
                        eprintln!("Audio: not enough samples");
                    }

                    let sample_iter = audio_recv.try_iter().take(requested);
                    for (i, sample) in sample_iter.enumerate() {
                        data[i] = sample;
                    }
                },
                move |err| {
                    dbg!(err);
                },
            )
            .unwrap();

        stream.play().unwrap();

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

                while !n.frame_ready_reset() {
                    n.run_scanline();

                    for sample in n.apu_samples() {
                        audio_send.send(*sample).unwrap();
                    }

                    n.apu_samples().clear();
                }
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
