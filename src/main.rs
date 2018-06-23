#![feature(test)]
#![feature(nll)]

//#[macro_use]
//extern crate bitflags;

mod cpu;
use cpu::Tick;

mod memory;
//mod ines;
//mod ppu;

mod tests;

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

fn main() {
    let mem = Rc::new(RefCell::new(memory::Memory::new()));
    mem.borrow_mut()
        .load_mapper_0(&mut File::open("donkey kong.nes").unwrap());
    let mut cpu = cpu::Cpu::new(mem.clone());
    //let mut ppu = ppu::Ppu::new(mem.clone());

    //cpu.load_to_memory(&mut File::open("nestest.nes").unwrap());

    //let header = ines::parse_header(&mut File::open("donkey kong.nes").unwrap());

    cpu.gen_reset();

    while !cpu.halt {
        cpu.print_debug_info();
        cpu.tick();
        //let mut s: String = String::new();
        //stdin.read_line(&mut s);
    }
}
