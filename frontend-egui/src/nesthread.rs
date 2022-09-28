use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamConfig,
};
use crossbeam::channel::Receiver;
use fearless_nes::Nes;

pub enum NesMsg {
    Pause,
    Unpause,
    Exit,
}

struct State {
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

        let (audio_send, audio_recv) = crossbeam::channel::bounded::<i16>(2048);
        let stream = setup_audio(audio_recv);

        let mut samples = Vec::with_capacity(16);

        loop {
            let deadline = Instant::now() + FRAME_DURATION;

            for msg in channel.try_iter() {
                match msg {
                    NesMsg::Exit => return,
                    NesMsg::Pause => {
                        state.paused = true;
                        stream.pause().ok();
                    }
                    NesMsg::Unpause => {
                        state.paused = false;
                        stream.play().ok();
                    }
                }
            }

            // TODO(rewrite): performance monitoring
            if !state.paused {
                let mut n = nes.lock().unwrap();

                while !n.frame_ready_reset() {
                    n.run_scanline();

                    n.apu_samples(&mut samples);

                    for s in samples.iter() {
                        audio_send.try_send(*s).ok();
                    }
                    samples.clear();
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

fn setup_audio(audio_recv: Receiver<i16>) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let supported_config: StreamConfig = supported_configs_range
        .find(|r| {
            r.sample_format() == SampleFormat::I16
                && r.min_sample_rate() <= SampleRate(48000)
                && r.max_sample_rate() >= SampleRate(48000)
        })
        .expect("no supported config?!")
        .with_sample_rate(SampleRate(48000))
        .into();

    let stream = device
        .build_output_stream(
            &supported_config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let requested = data.len();
                let got = audio_recv.len();

                if got < requested {
                    // INVESTIGATE: is underrun necessarily a problem here ?
                    /* eprintln!(
                        "Audio: not enough samples. Requested: {}, got: {}",
                        requested, got
                    ); */
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
    stream
}
