pub mod cpu;
use self::cpu::Tick;

pub mod ines;
pub mod mapper;
pub mod memory;
pub mod ppu;

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::rc::Rc;

pub struct Nes {
    pub cpu: cpu::Cpu,
    pub mem: Rc<RefCell<memory::Memory>>,
    ppu: Rc<RefCell<ppu::Ppu>>,
    int_bus: Rc<RefCell<InterruptBus>>,
}

impl Nes {
    pub fn new(rom_path: &Path) -> Result<Nes, NesError> {
        let mut file = File::open(rom_path)?;
        let header = ines::parse_header(&mut file)?;

        let mapper = Rc::new(RefCell::new(mapper::get_mapper(header.mapper())));

        let mut file = File::open(rom_path)?;
        mapper.borrow_mut().load_cartridge(header, &mut file)?;

        let int_bus = Rc::new(RefCell::new(InterruptBus::new()));
        let ppu = Rc::new(RefCell::new(ppu::Ppu::new(int_bus.clone(), mapper.clone())));
        let mem = Rc::new(RefCell::new(memory::Memory::new(
            ppu.clone(),
            mapper.clone(),
        )));

        let mut cpu = cpu::Cpu::new(mem.clone(), int_bus.clone());
        cpu.gen_reset();

        Ok(Nes {
            cpu,
            mem,
            ppu,
            int_bus,
        })
    }

    pub fn run(&mut self) {
        let mut ppu_cycle = 0;
        self.cpu.gen_reset();

        while !self.cpu.halt {
            ppu_cycle += 1;
            if self.mem.borrow().dma_cycles > 0 {
                self.mem.borrow_mut().dma_cycles -= 1;
            } else {
                self.cpu.tick();
            }

            self.ppu.borrow_mut().tick();
            self.ppu.borrow_mut().tick();
            self.ppu.borrow_mut().tick();
        }
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

#[derive(Debug)]
pub enum NesError {
    IoError(io::Error),
    NesConstantMissing,
    InvalidFile,
}

impl From<io::Error> for NesError {
    fn from(error: io::Error) -> Self {
        NesError::IoError(error)
    }
}
