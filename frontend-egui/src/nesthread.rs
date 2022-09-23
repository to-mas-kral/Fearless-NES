use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use blip_buf::BlipBuf;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate,
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
const SAMPLE_RATE: u32 = 44100;

pub fn run_nes_thread(nes: Arc<Mutex<Nes>>, channel: Receiver<NesMsg>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut state = State::new();

        let (audio_send, audio_recv) = crossbeam::channel::bounded::<i16>(4096);
        let stream = setup_audio(audio_recv);

        let mut blip_buf = BlipBuf::new(4096);
        blip_buf.set_rates(1789772.7272, SAMPLE_RATE as f64);
        let mut blip_time = 0;
        let mut last_sample = 0i32;

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

                    handle_audio(
                        &mut n,
                        &mut last_sample,
                        &mut blip_buf,
                        &mut blip_time,
                        &audio_send,
                    );
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

// TODO: rewrite blip_buf in Rust and refactor
fn handle_audio(
    n: &mut std::sync::MutexGuard<Nes>,
    last_sample: &mut i32,
    blip_buf: &mut BlipBuf,
    blip_time: &mut u32,
    audio_send: &crossbeam::channel::Sender<i16>,
) {
    for sample in n.apu_samples() {
        let delta = (*sample) - *last_sample;
        *last_sample = *sample;

        if delta != 0 {
            blip_buf.add_delta(*blip_time, delta);
        }

        *blip_time += 1;
    }
    blip_buf.end_frame(*blip_time);
    *blip_time = 0;
    while blip_buf.samples_avail() > 0 {
        let temp = &mut [0i16; 1024];
        let count = blip_buf.read_samples(temp, false);

        for s in 0..count {
            audio_send.try_send(temp[s]).ok();
        }
    }
    n.apu_samples().clear();
}

fn setup_audio(audio_recv: Receiver<i16>) -> cpal::Stream {
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
        .with_sample_rate(SampleRate(SAMPLE_RATE))
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
                        "Audio: not enough samples. Requestred: {}, got: {}",
                        requested, len
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
