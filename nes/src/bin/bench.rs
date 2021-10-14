use std::{collections::hash_map::DefaultHasher, env, fs, hash::Hasher, path::Path, time::Instant};

use fearless_nes::{Nes, ReplayInputs};

/// Loads the game, performs recorded inputs and compares the hash of the framebuffer at the end
fn game_bench(rom_path: &str, inputs_path: &str) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_path = "/../roms/";
    let rom_path = base_dir.clone() + test_path + rom_path;

    let rom = fs::read(Path::new(&rom_path)).unwrap();
    let mut nes = Nes::new(&rom).expect("error when creating test NES instance");

    let inputs_path = base_dir + "/src/tests/integration/" + inputs_path;
    let inputs = fs::read(Path::new(&inputs_path)).unwrap();
    let inputs = crate::ReplayInputs::load_state(&inputs).unwrap();

    let mut max_frame_time = 0;
    let mut total_time = 0u128;
    let mut frames = 0;

    for ic in &inputs.inputs {
        while nes.get_frame_count() < ic.frame {
            let start = Instant::now();
            nes.run_one_frame();
            let duration = start.elapsed();

            // The first few frames are ususally unproportionally fast
            if nes.get_frame_count() > 50 {
                total_time += duration.as_millis();
                frames += 1;

                if duration.as_millis() > max_frame_time {
                    max_frame_time = duration.as_millis();
                }
            }
        }

        nes.set_button_state(ic.button.clone(), ic.state);
    }

    while nes.get_frame_count() < inputs.end_frame {
        nes.run_one_frame();
    }

    let mut hasher = DefaultHasher::new();
    hasher.write(nes.get_frame_buffer());

    println!(
        "{} - Average frame time: {:.2}ms, Max frame time: {:.2}ms",
        rom_path,
        total_time as f64 / frames as f64,
        max_frame_time
    );
}

fn main() {
    game_bench("Super Mario Bros..nes", "SMB.inputs");

    game_bench("Mega Man II.nes", "Mega Man II.inputs");

    game_bench("Castlevania.nes", "Castlevania.inputs");

    game_bench("Solomon's Key.nes", "Solomon's Key.inputs");

    game_bench("Adventure Island II.nes", "Adventure Island II.inputs");

    game_bench("Battletoads.nes", "Battletoads.inputs");
}
