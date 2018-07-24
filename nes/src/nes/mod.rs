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
    pub frame: Rc<RefCell<Frame>>,
    pub cpu: cpu::Cpu,
    pub mem: Rc<RefCell<memory::Memory>>,
    ppu: Rc<RefCell<ppu::Ppu>>,
    int_bus: Rc<RefCell<InterruptBus>>,
    cycle_counter: u64,
}

impl Nes {
    pub fn new(rom_path: &Path) -> Result<Nes, NesError> {
        let mut file = File::open(rom_path)?;
        let header = ines::parse_header(&mut file)?;

        let mapper = Rc::new(RefCell::new(mapper::get_mapper(header.mapper())));

        let mut file = File::open(rom_path)?;
        mapper.borrow_mut().load_cartridge(header, &mut file)?;

        let frame = Rc::new(RefCell::new(Frame::new()));

        let int_bus = Rc::new(RefCell::new(InterruptBus::new()));
        let ppu = Rc::new(RefCell::new(ppu::Ppu::new(
            int_bus.clone(),
            mapper.clone(),
            frame.clone(),
        )));
        let mem = Rc::new(RefCell::new(memory::Memory::new(
            ppu.clone(),
            mapper.clone(),
        )));

        let mut cpu = cpu::Cpu::new(mem.clone(), int_bus.clone());
        cpu.gen_reset();

        Ok(Nes {
            frame,
            cpu,
            mem,
            ppu,
            int_bus,
            cycle_counter: 3,
        })
    }

    pub fn get_framebuffer(&self) -> Rc<RefCell<Frame>> {
        self.frame.clone()
    }

    pub fn run(&mut self) {
        while !self.cpu.halt {
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

    pub fn run_one_frame(&mut self) {
        for _ in 0..=29780 {
            self.cpu.tick();
            for _ in 0..3 {
                self.ppu.borrow_mut().tick();
            }
        }
        self.ppu.borrow_mut().tick();
    }

    pub fn run_one_cpu_cycle(&mut self) {
        self.cpu.tick();
        for _ in 0..3 {
            self.ppu.borrow_mut().tick();
        }
    }

    pub fn run_one_ppu_cycle(&mut self) {
        if self.cycle_counter == 3 {
            self.cpu.tick();
            self.cycle_counter = 0;
        }
        self.ppu.borrow_mut().tick();
        self.cycle_counter += 1;
    }
}

#[derive(Clone)]
pub struct Frame {
    pub output_buffer: [(u8, u8, u8); 256 * 240],
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            output_buffer: [(0, 0, 0); 256 * 240],
        }
    }

    pub fn clear(&mut self) {
        self.output_buffer = [(0, 0, 0); 256 * 240];
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
