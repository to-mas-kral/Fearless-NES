#[macro_use]
extern crate clap;
extern crate fearless_nes;
extern crate sdl2;

use clap::{App, Arg};

use fearless_nes::nes::controller;
use fearless_nes::nes::ppu::PALETTE;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::Error;
use sdl2::EventPump;

use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

static ROM_PATH: &str = {
    //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/vbl_nmi_timing/5.nmi_suppression.nes"
    //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/vbl_nmi_timing/6.nmi_disable.nes"
    //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/vbl_nmi_timing/7.nmi_timing.nes"

    //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_open_bus/ppu_open_bus.nes"

    //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/cpu/cpu_timing_test6/cpu_timing_test.nes"
    //Ox9D, 0x43, 0x46 - probably inaccurate due to NMI

    //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/cpu/nestest/nestest.nes"

    "/home/tomas/Documents/Programovani/fearless-nes/Nintendo/Super Mario Bros..nes"
};

fn main() {
    let matches = App::new("Fearless-NES")
        .version("0.1.0")
        .author("Tomáš Král <tomas@kral.hk>")
        .about("A NES emulator written in Rust")
        .arg(
            Arg::with_name("rom")
                .short("r")
                .long("rom")
                .help("Sets the ROM input file to use")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("scale")
                .short("s")
                .long("scale")
                .help("Sets the screen size scaling")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let rom_path = matches.value_of("rom").unwrap_or(ROM_PATH);
    let scale = value_t!(matches, "scale", f32).unwrap_or(4f32);

    let mut sdl = match SdlSystem::new(scale) {
        Ok(sdl) => sdl,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    let mut texture = sdl
        .texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut nes = match fearless_nes::nes::Nes::new(Path::new(rom_path)) {
        Ok(nes) => nes,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    //Update new pointer
    let ptr: *mut _ = &mut nes;

    nes.cpu.nes = ptr;
    nes.ppu.nes = ptr;
    nes.apu.nes = ptr;

    let mut pause = false;

    'running: loop {
        for event in sdl.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => nes.run_one_frame(),
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => pause = !pause,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => nes.set_controller_state(controller::Keycode::A, true),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => nes.set_controller_state(controller::Keycode::S, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    ..
                } => nes.set_controller_state(controller::Keycode::Z, true),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => nes.set_controller_state(controller::Keycode::X, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => nes.set_controller_state(controller::Keycode::Up, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => nes.set_controller_state(controller::Keycode::Down, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => nes.set_controller_state(controller::Keycode::Left, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => nes.set_controller_state(controller::Keycode::Right, true),
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => nes.set_controller_state(controller::Keycode::A, false),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => nes.set_controller_state(controller::Keycode::S, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Y),
                    ..
                } => nes.set_controller_state(controller::Keycode::Z, false),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => nes.set_controller_state(controller::Keycode::X, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => nes.set_controller_state(controller::Keycode::Up, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => nes.set_controller_state(controller::Keycode::Down, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => nes.set_controller_state(controller::Keycode::Left, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => nes.set_controller_state(controller::Keycode::Right, false),
                _ => {}
            }
        }

        let start = Instant::now();
        if !pause {
            nes.run_one_frame();
        }
        buffer_to_texture(&nes, &mut texture);
        let end = Instant::now();

        let elapsed = end.duration_since(start);
        let expected = Duration::from_millis(16);

        if let Some(d) = expected.checked_sub(elapsed) {
            thread::sleep(d);
        }

        sdl.canvas.clear();
        sdl.canvas
            .copy(&texture, None, Some(Rect::new(0, 0, 256, 240)))
            .unwrap();
        sdl.canvas.set_scale(scale, scale).unwrap();
        sdl.canvas.present();
    }
}

struct SdlSystem {
    canvas: Canvas<Window>,
    event_pump: EventPump,

    texture_creator: TextureCreator<WindowContext>,
}

impl SdlSystem {
    pub fn new(scale: f32) -> Result<SdlSystem, Error> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "Fearless-NES",
                (256f32 * scale) as u32,
                (240f32 * scale) as u32,
            )
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();

        let event_pump = sdl_context.event_pump().unwrap();

        Ok(SdlSystem {
            canvas,
            event_pump,
            texture_creator,
        })
    }
}

#[inline]
fn buffer_to_texture(nes: &fearless_nes::nes::Nes, texture: &mut Texture) {
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..240 {
                for x in 0..256 {
                    let texture_address = (y * pitch) + (x * 3);
                    let framebuffer_address = (y << 8) + x;

                    let pixel_colour = nes.ppu.output_buffer[framebuffer_address];

                    let palette_address = (pixel_colour * 3) as usize;

                    buffer[texture_address] = PALETTE[palette_address];
                    buffer[texture_address + 1] = PALETTE[palette_address + 1];
                    buffer[texture_address + 2] = PALETTE[palette_address + 2];
                }
            }
        })
        .unwrap();
}
