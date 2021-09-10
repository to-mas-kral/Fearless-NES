use std::{collections::hash_map::DefaultHasher, env, fs, hash::Hasher, path::Path};

use crate::Nes;

/// Loads the game, performs recorded inputs and compares the hash of the framebuffer at the end
fn game_hash_test(rom_path: &str, inputs_path: &str, result_hash: u64) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_path = "/../roms/";
    let rom_path = base_dir.clone() + test_path + rom_path;

    let rom = fs::read(Path::new(&rom_path)).unwrap();
    let mut nes = Nes::new(&rom).expect("error when creating test NES instance");

    let inputs_path = base_dir + "/src/tests/integration/" + inputs_path;
    let inputs = fs::read(Path::new(&inputs_path)).unwrap();
    let inputs = crate::ReplayInputs::load_state(&inputs).unwrap();

    nes.drive_replay_inputs(&inputs);

    let mut hasher = DefaultHasher::new();
    hasher.write(nes.get_frame_buffer());

    assert_eq!(hasher.finish(), result_hash);
}

// Mapper 0

#[test]
fn smb() {
    game_hash_test("Super Mario Bros..nes", "SMB.inputs", 9966675680041268468);
}

// Mapper 1

#[test]
fn mega_man_2() {
    game_hash_test("Mega Man II.nes", "Mega Man II.inputs", 2283362443600069136);
}

// Mapper 2

#[test]
fn castlevania() {
    game_hash_test("Castlevania.nes", "Castlevania.inputs", 555372468362883018);
}

// Mapper 3
#[test]
fn solomons_key() {
    game_hash_test(
        "Solomon's Key.nes",
        "Solomon's Key.inputs",
        7547833452040878089,
    );
}

// Mapper 4
#[test]
fn adventure_island_2() {
    game_hash_test(
        "Adventure Island II.nes",
        "Adventure Island II.inputs",
        17528018295609052337,
    );
}

// Mapper 7
#[test]
fn battletoads() {
    game_hash_test("Battletoads.nes", "Battletoads.inputs", 414186645752951874);
}
