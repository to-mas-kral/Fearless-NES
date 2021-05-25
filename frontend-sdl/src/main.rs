use sdl2::{
    controller::{Axis, Button},
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Error, EventPump, GameControllerSubsystem,
};

use std::{
    env, thread,
    time::{Duration, Instant},
};

use fearless_nes::{controller, ppu::PALETTE};

// The NTSC frame rate is 60,0988 FPPS
const NTSC_FRAME_DURATION: Duration = Duration::from_nanos(16639267);

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1);

    let rom_path = &args[1];

    let scale = if args.len() > 1 {
        args[2].parse::<f32>().unwrap()
    } else {
        4.0
    };

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

    let mut nes = match fearless_nes::Nes::new(rom_path) {
        Ok(nes) => nes,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    sdl.controller.set_event_state(true);
    if sdl.controller.num_joysticks().unwrap() > 0 {}
    let _controller = sdl.controller.open(0).unwrap();

    let mut pause = false;

    let mut durs: Vec<Duration> = vec![];

    'running: loop {
        let start = Instant::now();

        for event in sdl.event_pump.poll_iter() {
            match event {
                /*
                    Keyboard  controller events
                */
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => nes.set_controller_state(controller::Keycode::A, true),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => nes.set_controller_state(controller::Keycode::B, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    ..
                } => nes.set_controller_state(controller::Keycode::Select, true),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => nes.set_controller_state(controller::Keycode::Start, true),
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
                } => nes.set_controller_state(controller::Keycode::B, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Y),
                    ..
                } => nes.set_controller_state(controller::Keycode::Select, false),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => nes.set_controller_state(controller::Keycode::Start, false),
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
                /*
                    Gamepad controller events
                */
                Event::ControllerButtonDown {
                    button: Button::B, ..
                } => nes.set_controller_state(controller::Keycode::A, true),
                Event::ControllerButtonDown {
                    button: Button::X, ..
                } => nes.set_controller_state(controller::Keycode::B, true),
                Event::ControllerButtonDown {
                    button: Button::Start,
                    ..
                } => nes.set_controller_state(controller::Keycode::Start, true),
                Event::ControllerButtonDown {
                    button: Button::Back,
                    ..
                } => nes.set_controller_state(controller::Keycode::Select, true),
                Event::ControllerButtonDown {
                    button: Button::DPadUp,
                    ..
                } => nes.set_controller_state(controller::Keycode::Up, true),
                Event::ControllerButtonDown {
                    button: Button::DPadDown,
                    ..
                } => nes.set_controller_state(controller::Keycode::Down, true),
                Event::ControllerButtonDown {
                    button: Button::DPadLeft,
                    ..
                } => nes.set_controller_state(controller::Keycode::Left, true),
                Event::ControllerButtonDown {
                    button: Button::DPadRight,
                    ..
                } => nes.set_controller_state(controller::Keycode::Right, true),
                Event::ControllerButtonUp {
                    button: Button::B, ..
                } => nes.set_controller_state(controller::Keycode::A, false),
                Event::ControllerButtonUp {
                    button: Button::X, ..
                } => nes.set_controller_state(controller::Keycode::B, false),
                Event::ControllerButtonUp {
                    button: Button::Start,
                    ..
                } => nes.set_controller_state(controller::Keycode::Start, false),
                Event::ControllerButtonUp {
                    button: Button::Back,
                    ..
                } => nes.set_controller_state(controller::Keycode::Select, false),
                Event::ControllerButtonUp {
                    button: Button::DPadUp,
                    ..
                } => nes.set_controller_state(controller::Keycode::Up, false),
                Event::ControllerButtonUp {
                    button: Button::DPadDown,
                    ..
                } => nes.set_controller_state(controller::Keycode::Down, false),
                Event::ControllerButtonUp {
                    button: Button::DPadLeft,
                    ..
                } => nes.set_controller_state(controller::Keycode::Left, false),
                Event::ControllerButtonUp {
                    button: Button::DPadRight,
                    ..
                } => nes.set_controller_state(controller::Keycode::Right, false),
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => match axis {
                    Axis::LeftX => {
                        if value < -25000 {
                            nes.set_controller_state(controller::Keycode::Left, true);
                        } else if value > 25000 {
                            nes.set_controller_state(controller::Keycode::Right, true);
                        } else {
                            nes.set_controller_state(controller::Keycode::Right, false);
                            nes.set_controller_state(controller::Keycode::Left, false);
                        }
                    }
                    Axis::LeftY => {
                        if value < -25000 {
                            nes.set_controller_state(controller::Keycode::Up, true);
                        } else if value > 25000 {
                            nes.set_controller_state(controller::Keycode::Down, true);
                        } else {
                            nes.set_controller_state(controller::Keycode::Down, false);
                            nes.set_controller_state(controller::Keycode::Up, false);
                        }
                    }
                    _ => (),
                },
                /*
                   Settings
                */
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
                    keycode: Some(Keycode::PrintScreen),
                    ..
                }
                | Event::ControllerButtonDown {
                    button: Button::Guide,
                    ..
                } => nes.save_state("fearlessNES-save.fnes"),
                Event::ControllerButtonDown {
                    button: Button::RightStick,
                    ..
                } => nes.load_state("fearlessNES-save.fnes"),
                _ => (),
            }
        }

        let emu_start = Instant::now();
        if !pause {
            nes.run_one_frame();
        }
        let emu_end = Instant::now();

        durs.push(emu_end.duration_since(emu_start));

        buffer_to_texture(&nes, &mut texture);

        sdl.canvas.clear();
        sdl.canvas
            .copy(&texture, None, Some(Rect::new(0, 0, 256, 240)))
            .unwrap();
        sdl.canvas.set_clip_rect(Rect::new(0, 0, 256, 240));
        sdl.canvas.set_scale(scale, scale).unwrap();
        sdl.canvas.present();

        let end = Instant::now();
        let elapsed = end.duration_since(start);

        if let Some(d) = NTSC_FRAME_DURATION.checked_sub(elapsed) {
            thread::sleep(d);
        }
    }

    println!(
        "Average frame time: {:?}",
        durs.iter().sum::<Duration>() / durs.len() as u32
    );
}

struct SdlSystem {
    canvas: Canvas<Window>,
    event_pump: EventPump,

    texture_creator: TextureCreator<WindowContext>,

    controller: GameControllerSubsystem,
}

impl SdlSystem {
    pub fn new(scale: f32) -> Result<SdlSystem, Error> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let controller = sdl_context.game_controller().unwrap();

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

            controller,
            //audio_device,
        })
    }
}

fn buffer_to_texture(nes: &fearless_nes::Nes, texture: &mut Texture) {
    texture
        .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            for (i, pixel_color) in nes.get_frame_buffer().iter().enumerate() {
                let rgb_address = (pixel_color * 3) as usize;
                let buffer_addr = i * 3;

                // This speeds things up a little bit and should be totally safe
                buffer[buffer_addr] = PALETTE[rgb_address];

                buffer[buffer_addr + 1] = PALETTE[rgb_address + 1];

                buffer[buffer_addr + 2] = PALETTE[rgb_address + 2];
            }
        })
        .unwrap();
}
