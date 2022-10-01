use std::{env, fs, hash::Hasher, path::Path};

use siphasher::sip::SipHasher13;

use fearless_nes::Nes;

pub fn blargg_test(rom_path: &str, pass_text: &str) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_path = "/tests/";
    let rom_path = base_dir + test_path + rom_path;

    let rom = fs::read(Path::new(&rom_path)).unwrap();

    let mut nes = Nes::new(&rom).expect("error when creating test NES instance");

    let s = nes.run_blargg_test();

    assert_eq!(s, pass_text);
}

#[allow(dead_code)]
pub fn hash_test(rom_path: &str, frames_to_run: u64, expected_hash: u64) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_path = "/tests/";
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
