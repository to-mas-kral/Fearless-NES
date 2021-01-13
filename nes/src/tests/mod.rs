#![allow(unused_imports)]
extern crate test;

use std::env;
use std::path::Path;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::BufReader;

use super::Nes;

mod cpu;
mod ppu;

use test::black_box;
use test::Bencher;

#[macro_export]
macro_rules! blargg_test {
    ($test_name: ident, $path: expr, $pass_text: expr) => {
        #[test]
        fn $test_name() {
            let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let test_path = "/src/tests/";

            let mut nes = Nes::new((base_dir + test_path + $path).as_str())
                .expect("error when creating test NES instance");

            let mut test_running = false;

            loop {
                nes.cpu_tick();

                let test_state = nes.cpu_peek(0x6000);
                if test_state == 0x80 {
                    test_running = true;
                }

                if test_running && test_state <= 81 {
                    break;
                }
            }

            let mut s = String::new();
            let mut p: usize = 0x6004;
            while nes.cpu_peek(p) != 0 {
                s.push(nes.cpu_peek(p) as char);
                p += 1;
            }

            assert_eq!(s, $pass_text);
        }
    };
}

#[macro_export]
macro_rules! hash_test {
    ($test_name:ident ,$path:expr, $frames:expr, $hash:expr) => {
        #[test]
        fn $test_name() {
            let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let test_path = "/src/tests/";

            let mut nes = Nes::new((base_dir + test_path + $path).as_str())
                .expect("error when creating test NES instance");

            for _ in 0..$frames {
                nes.run_one_frame();
            }

            let mut hasher = DefaultHasher::new();

            for addr in 0..0xFFF {
                hasher.write_u8((nes.mapper.read_nametable.ptr)(&mut nes, addr));
            }

            assert_eq!(hasher.finish(), $hash);
        }
    };
}

#[bench]
fn nes_bencher(b: &mut Bencher) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bench_path = base_dir + "/src/tests/SPRITE.NES";

    let mut nes =
        Nes::new(&bench_path).expect("error when creating bencher NES instance");

    b.iter(|| {
        black_box(nes.run_one_frame());
    });
}

#[bench]
fn test_bencher_1(b: &mut Bencher) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bench_path = base_dir + "/src/tests/ppu/scanline/scanline.nes";

    let mut nes =
        Nes::new(&bench_path).expect("error when creating bencher NES instance");

    b.iter(|| {
        black_box(nes.run_one_frame());
    });
}

#[bench]
fn test_bencher_2(b: &mut Bencher) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bench_path = base_dir + "/src/tests/cpu/blargg_instr/all_instrs.nes";

    let mut nes =
        Nes::new(&bench_path).expect("error when creating bencher NES instance");

    b.iter(|| {
        black_box(nes.run_one_frame());
    });
}
