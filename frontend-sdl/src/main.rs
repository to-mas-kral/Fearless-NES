extern crate fearless_nes;
extern crate sdl2;

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
        .window("rust-sdl2 demo", 256, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let sdl_texture_creator = canvas.texture_creator();
    let mut sdl_texture = sdl_texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    //TODO: unwraps
    let mut nes = fearless_nes::nes::Nes::new(Path::new(
        "/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/blargg_instr/rom_singles/01-basics.nes",
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/palette/palette.nes",
        //"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/nestest/nestest.nes",
        //"/home/tomas/Documents/Programovani/fearless-nes/SMB.nes",
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
                    keycode: Some(Keycode::L),
                    ..
                } => nes.run_one_ppu_cycle(),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => nes.run_one_frame(),
                _ => {}
            }
        }

        
        nes.run_one_frame();

         buffer_to_texture(framebuffer.clone(), &mut sdl_texture);

        canvas.clear();
        canvas
            .copy(&sdl_texture, None, Some(Rect::new(0, -8, 256, 240)))
            .unwrap();
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_u32));
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
        })
        .unwrap();
}
