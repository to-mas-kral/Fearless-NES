use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, StreamConfig, StreamError,
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
        let (stream, sample_rate) = setup_audio(audio_recv);

        {
            let mut n = nes.lock().unwrap();
            n.set_sample_rate(sample_rate.0 as f64)
        }

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

fn setup_audio(audio_recv: Receiver<i16>) -> (cpal::Stream, SampleRate) {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let default_config: StreamConfig = device.default_output_config().unwrap().into();

    let sample_rate = default_config.sample_rate;
    let channels = default_config.channels;

    let stream = device
        .build_output_stream(
            &default_config,
            move |buf: &mut [i16], _: &cpal::OutputCallbackInfo| {
                stream_callback(buf, audio_recv.clone(), channels);
            },
            stream_err,
        )
        .unwrap();

    stream.play().unwrap();
    (stream, sample_rate)
}

fn stream_callback(buf: &mut [i16], audio_recv: Receiver<i16>, channels: u16) {
    let requested = buf.len();
    let got = audio_recv.len();

    if got < requested / channels as usize {
        // INVESTIGATE: is underrun necessarily a problem here ?
        /* eprintln!(
            "Audio: not enough samples. Requested: {}, got: {}",
            requested, got
        ); */
    }

    let sample_iter = audio_recv.try_iter().take(requested / channels as usize);

    let mut i = 0;
    for sample in sample_iter {
        buf[i..(i + channels as usize)].fill(sample);
        i += channels as usize;
    }
}

fn stream_err(e: StreamError) {
    dbg!(e);
}
