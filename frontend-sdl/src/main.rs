extern crate fearless_nes;
extern crate sdl2;

use fearless_nes::nes::controller;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use fearless_nes::nes::Frame;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 256 * 4, 240 * 4)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let sdl_texture_creator = canvas.texture_creator();
    let mut sdl_texture = sdl_texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256 * 4, 240 * 4)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut nes = fearless_nes::nes::Nes::new(Path::new(
        "/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/04-nmi_control.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/05-nmi_timing.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/06-suppression.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes"

        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_open_bus/ppu_open_bus.nes"

        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/cpu/branch_timing_tests/1.Branch_Basics.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/cpu/branch_timing_tests/3.Forward_Branch.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/cpu/cpu_timing_test6/cpu_timing_test.nes"

        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/palette/palette.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/full_palette/full_palette.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/cpu/nestest/nestest.nes"

        //"/home/tomas/Documents/Programovani/fearless-nes/donkey_kong.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/arkanoid.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/Clu_Clu_Land.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/Pinball.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/SMB.nes"
        //"/home/tomas/Documents/Programovani/fearless-nes/popeye.nes",
    )).unwrap();
    let framebuffer = nes.get_framebuffer();

    'running: loop {
        for event in event_pump.poll_iter() {
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

        nes.run_one_frame();

        buffer_to_texture(framebuffer.clone(), &mut sdl_texture);

        canvas.clear();
        canvas.set_scale(4f32, 4f32).unwrap();
        canvas
            .copy(&sdl_texture, None, Some(Rect::new(0, 0, 256 * 4, 240 * 4)))
            .unwrap();
        canvas.present();

        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32));
    }
}

#[inline]
fn buffer_to_texture(frame: Rc<RefCell<Frame>>, texture: &mut Texture) {
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..240 {
                for x in 0..256 {
                    let texture_address = (y * pitch) + (x * 3);
                    let framebuffer_address = (y << 8) + x;

                    let pixel_colour = frame.borrow_mut().output_buffer[framebuffer_address];

                    buffer[texture_address] = pixel_colour.0;
                    buffer[texture_address + 1] = pixel_colour.1;
                    buffer[texture_address + 2] = pixel_colour.2;
                }
            }
        }).unwrap();
}
