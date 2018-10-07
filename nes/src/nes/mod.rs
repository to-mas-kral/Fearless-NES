#[macro_export]
macro_rules! nes {
    ($ptr:expr) => {{
        let ret = unsafe { &mut *$ptr };
        ret
    }};
}

pub mod cpu;
use self::cpu::Tick;

pub mod apu;
pub mod controller;
pub mod ines;
pub mod mapper;
pub mod ppu;

use std::fs::File;
use std::io;
use std::path::Path;

pub struct Nes {
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Ppu,
    pub apu: apu::Apu,

    pub mapper: Box<mapper::Mapper>,
    pub interrupt_bus: InterruptBus,
    pub controller: controller::Controller,

    pub frame: Frame,
    cycle_count: u64,
}

impl Nes {
    pub fn new(rom_path: &Path) -> Result<Nes, NesError> {
        let mut file = File::open(rom_path)?;
        let header = ines::parse_header(&mut file)?;

        let mut mapper = mapper::get_mapper(header);

        let mut file = File::open(rom_path)?;
        mapper.load_cartridge(&mut file)?;

        let ppu = ppu::Ppu::new();
        let apu = apu::Apu::new();
        let cpu = cpu::Cpu::new();

        let frame = Frame::new();

        let interrupt_bus = InterruptBus::new();
        let controller = controller::Controller::new();

        let mut nes = Nes {
            cpu,
            ppu,
            apu,

            mapper,
            interrupt_bus,
            controller,

            frame,
            cycle_count: 0,
        };

        //Update new pointer
        let ptr: *mut _ = &mut nes;

        nes.cpu.nes = ptr;
        nes.ppu.nes = ptr;
        nes.apu.nes = ptr;

        nes.cpu.gen_reset();
        for _ in 0..6 {
            nes.run_one_cycle();
        }

        Ok(nes)
    }

    pub fn set_controller_state(&mut self, keycode: controller::Keycode, state: bool) {
        self.controller.set_button(keycode, state);
    }

    pub fn run_one_frame(&mut self) {
        while !self.frame.ready {
            self.run_one_cycle();
        }
        self.frame.ready = false;
    }

    pub fn run_one_cycle(&mut self) {
        let ptr: *mut Nes = self;
        if ptr != self.cpu.nes || ptr != self.ppu.nes || ptr != self.apu.nes {
            let ptr: *mut Nes = self;

            self.cpu.nes = ptr;
            self.ppu.nes = ptr;
            self.apu.nes = ptr;
        }

        self.cpu.tick();
        self.cycle_count += 1;
        if self.cycle_count == 29658 {
            self.ppu.enable_writes();
        }

        for _ in 0..3 {
            self.ppu.tick();
        }

        self.apu.tick();
    }
}

#[derive(Clone)]
pub struct Frame {
    pub ready: bool,
}

impl Frame {
    pub fn new() -> Frame {
        Frame { ready: false }
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
    PalRom,
    InvalidFile,
}

impl From<io::Error> for NesError {
    fn from(error: io::Error) -> Self {
        NesError::IoError(error)
    }
}
