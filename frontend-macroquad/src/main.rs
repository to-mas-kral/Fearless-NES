use std::{env, fs, path::PathBuf};

use gilrs::Gilrs;
use macroquad::prelude::*;

use fearless_nes::Nes;

mod app;

use app::{report_error, App};

const NES_WIDTH: usize = 256;
const NES_HEIGHT: usize = 240;

fn window_conf() -> Conf {
    Conf {
        window_title: "Fearless-NES".to_owned(),
        fullscreen: true,
        high_dpi: true,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = app::Config::new();

    let mut app = App::new(config);
    if let Some(p) = env::args().nth(1) {
        app.nes = create_nes(PathBuf::from(p)).ok()
    }

    let mut gilrs = match Gilrs::new() {
        Ok(g) => Some(g),
        Err(_) => {
            report_error("Couldn't initialize gamepad input library");
            None
        }
    };

    loop {
        app.handle_input(&mut gilrs);

        app.run_nes_frame();

        app.draw_gui();

        app.draw_nes();

        egui_macroquad::draw();

        prevent_quit();
        if is_quit_requested() {
            if let Err(_) = app.config.save() {
                report_error(&format!("Couldn't save the configuration file"));
            }
            break;
        }

        next_frame().await;
    }
}

fn create_nes(rom_path: PathBuf) -> Result<Nes, ()> {
    let rom = match fs::read(rom_path) {
        Ok(r) => r,
        Err(_) => {
            report_error("Error while reading from the ROM file");
            return Err(());
        }
    };

    match Nes::new(&rom) {
        Ok(n) => return Ok(n),
        Err(e) => {
            report_error(&format!("Error while loading the ROM: {:?}", e));
            return Err(());
        }
    };
}
