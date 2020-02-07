#![feature(test)]

//pub mod nes;
mod tests;

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
pub mod cartridge;
pub mod controller;
pub mod mapper;
pub mod ppu;

use std::fs::File;
use std::io;
use std::path::Path;

pub struct Nes {
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Ppu,
    pub apu: apu::Apu,

    pub mapper: Box<dyn mapper::Mapper>,

    pub controller: controller::Controller,

    pub frame_ready: bool,
    cycle_count: u64,
}

impl Nes {
    pub fn new(rom_path: &Path) -> Result<Nes, NesError> {
        let mut rom = File::open(rom_path)?;

        let cartridge = cartridge::parse_rom(&mut rom)?;

        let mapper = mapper::create_mapper(cartridge)?;

        let ppu = ppu::Ppu::new();
        let apu = apu::Apu::new();
        let cpu = cpu::Cpu::new();

        let controller = controller::Controller::new();

        let mut nes = Nes {
            cpu,
            ppu,
            apu,

            mapper,
            controller,

            frame_ready: false,
            cycle_count: 0,
        };

        //Update new pointer
        let ptr: *mut _ = &mut nes;

        nes.cpu.nes = ptr;
        nes.ppu.nes = ptr;
        nes.apu.nes = ptr;
        nes.mapper.update_nes_ptr(ptr);

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
        while !self.frame_ready {
            self.run_one_cycle();
        }
        self.frame_ready = false;
    }

    pub fn run_one_cycle(&mut self) {
        let ptr: *mut Nes = self;
        if ptr != self.cpu.nes || ptr != self.ppu.nes || ptr != self.apu.nes {
            let ptr: *mut Nes = self;

            self.cpu.nes = ptr;
            self.ppu.nes = ptr;
            self.apu.nes = ptr;
            self.mapper.update_nes_ptr(ptr);
        }

        self.cycle_count += 1;
        if self.cycle_count == 29658 {
            self.ppu.enable_writes();
        }

        self.cpu.tick();
        for _ in 0..3 {
            self.ppu.tick();
        }

        self.apu.tick();
    }
}

#[derive(Debug)]
pub enum NesError {
    IoError(io::Error),
    PalRom,
    InvalidFile,
    UnsupportedMapper,
}

impl From<io::Error> for NesError {
    fn from(error: io::Error) -> Self {
        NesError::IoError(error)
    }
}
