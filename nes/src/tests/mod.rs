use std::{env, fs, hash::Hasher, path::Path};

use siphasher::sip::SipHasher13;

use super::Nes;

mod cpu;

#[cfg(feature = "integration_tests")]
mod integration;
mod ppu;

fn blargg_test(rom_path: &str, pass_text: &str) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_path = "/src/tests/";
    let rom_path = base_dir + test_path + rom_path;

    let rom = fs::read(Path::new(&rom_path)).unwrap();

    let mut nes = Nes::new(&rom).expect("error when creating test NES instance");

    let mut test_running = false;

    loop {
        nes.cpu_tick();

        let test_state = nes.cpu_read(0x6000);
        if test_state == 0x80 {
            test_running = true;
        }

        if test_running && test_state <= 81 {
            break;
        }
    }

    let mut s = String::new();
    let mut p: usize = 0x6004;
    while nes.cpu_read(p) != 0 {
        s.push(nes.cpu_read(p) as char);
        p += 1;
    }

    assert_eq!(s, pass_text);
}

fn hash_test(rom_path: &str, frames_to_run: u64, expected_hash: u64) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_path = "/src/tests/";
    let rom_path = base_dir + test_path + rom_path;

    let rom = fs::read(Path::new(&rom_path)).unwrap();

    let mut nes = Nes::new(&rom).expect("error when creating test NES instance");

    for _ in 0..frames_to_run {
        nes.run_one_frame();
    }

    let mut hasher = SipHasher13::new();

    hasher.write(nes.frame_buffer());
    assert_eq!(hasher.finish(), expected_hash);
}
