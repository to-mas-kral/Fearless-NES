pub mod cpu;
use self::cpu::Tick;

pub mod memory;
//mod ines;
pub mod ppu;

use std::cell::Cell;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

pub fn run() {
    let int_bus_ref = Rc::new(Cell::new(InterruptBus::new()));

    let ppu_ref = Rc::new(RefCell::new(ppu::Ppu::new(int_bus_ref.clone())));
    let mem = Rc::new(RefCell::new(memory::Memory::new(ppu_ref)));
    mem.borrow_mut()
        .load_mapper_0_1(&mut File::open("donkey kong.nes").unwrap());
    let mut cpu = cpu::Cpu::new(mem.clone(), int_bus_ref.clone());

    //let header = ines::parse_header(&mut File::open("donkey kong.nes").unwrap());

    cpu.gen_reset();

    while !cpu.halt {
        //TODO: stall CPU on DMA
        cpu.print_debug_info();
        if mem.borrow().dma_cycles > 0 {
            mem.borrow_mut().dma_cycles -= 1;
        } else {
            cpu.tick();
        }
        //let mut s: String = String::new();
        //stdin.read_line(&mut s);
    }
}

#[derive(Clone, Copy)]
pub struct InterruptBus {
    pub irq_signal: bool,
    pub nmi_signal: bool,
    pub reset_signal: bool,
}

impl InterruptBus {
    pub fn new() -> InterruptBus {
        InterruptBus {
            irq_signal: false,
            nmi_signal: false,
            reset_signal: false,
        }
    }
}
