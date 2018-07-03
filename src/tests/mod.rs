#![allow(unused_imports)]
extern crate test;

use std::cell::RefCell;
use std::cell::Cell;
use std::rc::Rc;
use std::env;
use std::fs::File;
use std::path::Path;

use self::test::Bencher;
#[cfg(test)]
use super::nes::{cpu, memory, ppu, InterruptBus};
use nes::cpu::Tick;

use std::io::BufRead;
use std::io::BufReader;

use nes::memory::MemoryOps;

#[bench]
fn nestest_bencher(b: &mut Bencher) {
    let nestest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let nestest_path = Path::new(&nestest_dir).join("src/tests/nestest/nestest.nes");

    let int_bus_ref = Rc::new(Cell::new(InterruptBus::new()));
    let ppu_ref = Rc::new(RefCell::new(ppu::Ppu::new(int_bus_ref.clone())));
    let mem = Rc::new(RefCell::new(memory::Memory::new(ppu_ref)));
    let mut cpu = cpu::Cpu::new(mem.clone(), int_bus_ref);
    cpu.load_to_memory(&mut File::open(nestest_path).unwrap());

    b.iter(|| {
        cpu.tick();
    });
}

#[bench]
fn blargg_bencher(b: &mut Bencher) {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bench_path = Path::new(&base_dir).join("src/tests/blargg_instr/rom_singles/07-abs_xy.nes");

    let int_bus_ref = Rc::new(Cell::new(InterruptBus::new()));
    let ppu_ref = Rc::new(RefCell::new(ppu::Ppu::new(int_bus_ref.clone())));
    let mem = Rc::new(RefCell::new(memory::Memory::new(ppu_ref)));
    let mut cpu = cpu::Cpu::new(mem.clone(), int_bus_ref);
    mem.borrow_mut()
        .load_mapper_0(&mut File::open(bench_path).unwrap());
    cpu.gen_reset();

    b.iter(|| {
        cpu.tick();
    });
}

#[test]
fn nestest() {
    let nestest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let nestest_log_path = Path::new(&nestest_dir).join("src/tests/nestest/nestest_formatted.log");
    let nestest_path = Path::new(&nestest_dir).join("src/tests/nestest/nestest.nes");

    let int_bus_ref = Rc::new(Cell::new(InterruptBus::new()));
    let ppu_ref = Rc::new(RefCell::new(ppu::Ppu::new(int_bus_ref.clone())));
    let mem = Rc::new(RefCell::new(memory::Memory::new(ppu_ref)));
    let mut cpu = cpu::Cpu::new(mem.clone(), int_bus_ref);
    cpu.load_to_memory(&mut File::open(nestest_path).unwrap());

    let f = File::open(nestest_log_path).unwrap();
    let file = BufReader::new(&f);
    let mut lines = file.lines();

    for _ in 0..8991 {
        assert_eq!(cpu.debug_info(), lines.next().unwrap().unwrap());
        cpu.print_debug_info();
        cpu.tick();
        while cpu.state != 0x100 {
            cpu.print_debug_info();
            cpu.tick();
        }
    }
}

macro_rules! blargg_instr_test {
    ($test_name:ident, $path:expr, $pass_text:expr) => {
        #[test]
        fn $test_name() {
            let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let test_path = Path::new(&base_dir)
                .join("src/tests/blargg_instr/rom_singles/")
                .join($path);

            let int_bus_ref = Rc::new(Cell::new(InterruptBus::new()));
            let ppu_ref = Rc::new(RefCell::new(ppu::Ppu::new(int_bus_ref.clone())));
            let mem = Rc::new(RefCell::new(memory::Memory::new(ppu_ref)));
            let mut cpu = cpu::Cpu::new(mem.clone(), int_bus_ref);
            mem.borrow_mut()
                .load_mapper_0(&mut File::open(test_path).unwrap());
            cpu.gen_reset();

            let mut test_running = false;

            loop {
                cpu.tick();
                while cpu.state != 0x100 {
                    cpu.tick();
                }

                let test_state = mem.read(0x6000);
                if test_state == 0x80 {
                    test_running = true;
                }

                if test_running && test_state <= 81 {
                    break;
                }
            }

            let mut s = String::new();
            let mut p: usize = 0x6004;
            while mem.read(p) != 0 {
                s.push(mem.read(p) as char);
                p += 1;
            }

            assert_eq!(s, $pass_text);
        }
    };
}

blargg_instr_test!(
    blargg_instr_basics,
    "01-basics.nes",
    "\n01-basics\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_implied,
    "02-implied.nes",
    "\n02-implied\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_immediate,
    "03-immediate.nes",
    "\n03-immediate\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_zero_page,
    "04-zero_page.nes",
    "\n04-zero_page\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_zero_page_xy,
    "05-zp_xy.nes",
    "\n05-zp_xy\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_absolute,
    "06-absolute.nes",
    "\n06-absolute\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_absolute_xy,
    "07-abs_xy.nes",
    "\n07-abs_xy\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_indirect_x,
    "08-ind_x.nes",
    "\n08-ind_x\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_indirect_y,
    "09-ind_y.nes",
    "\n09-ind_y\n\nPassed\n"
);
blargg_instr_test!(
    blargg_instr_branches,
    "10-branches.nes",
    "\n10-branches\n\nPassed\n"
);
blargg_instr_test!(blargg_instr_stack, "11-stack.nes", "\n11-stack\n\nPassed\n");
blargg_instr_test!(
    blargg_instr_jmp_jsr,
    "12-jmp_jsr.nes",
    "\n12-jmp_jsr\n\nPassed\n"
);
blargg_instr_test!(blargg_instr_rts, "13-rts.nes", "\n13-rts\n\nPassed\n");
blargg_instr_test!(blargg_instr_rti, "14-rti.nes", "\n14-rti\n\nPassed\n");
blargg_instr_test!(blargg_instr_brk, "15-brk.nes", "\n15-brk\n\nPassed\n");
blargg_instr_test!(
    blargg_instr_special,
    "16-special.nes",
    "\n16-special\n\nPassed\n"
);
