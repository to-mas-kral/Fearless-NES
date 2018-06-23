#[allow(unused_imports)]
mod tests {
    extern crate test;

    use std::cell::RefCell;
    use std::env;
    use std::fs::File;
    use std::path::Path;
    use std::rc::Rc;

    use self::test::bench;
    use self::test::Bencher;
    #[cfg(test)]
    use super::super::{cpu, memory};
    use cpu::Tick;

    use std::io::BufRead;
    use std::io::BufReader;

    #[bench]
    fn nestest_bencher(b: &mut Bencher) {
        let nestest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let nestest_path = Path::new(&nestest_dir).join("src/tests/nestest.nes");

        let mem = Rc::new(RefCell::new(memory::Memory::new()));
        let mut cpu = cpu::Cpu::new(mem.clone());
        cpu.load_to_memory(&mut File::open(nestest_path).unwrap());

        b.iter(|| {
            cpu.tick();
        });
    }

    #[test]
    fn nestest() {
        let nestest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let nestest_log_path = Path::new(&nestest_dir).join("src/tests/nestest_formatted.log");
        let nestest_path = Path::new(&nestest_dir).join("src/tests/nestest.nes");

        let mem = Rc::new(RefCell::new(memory::Memory::new()));
        let mut cpu = cpu::Cpu::new(mem.clone());
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

    #[test]
    fn nestest() {
        let nestest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let nestest_log_path = Path::new(&nestest_dir).join("src/tests/nestest_formatted.log");
        let nestest_path = Path::new(&nestest_dir).join("src/tests/nestest.nes");

        let mem = Rc::new(RefCell::new(memory::Memory::new()));
        let mut cpu = cpu::Cpu::new(mem.clone());
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
}
