#![feature(test)]

mod tests;

pub mod cpu;

pub mod apu;
pub mod cartridge;
pub mod controller;
pub mod mapper;
pub mod ppu;

use mapper::Mapper;

use std::fs::File;
use std::io;
use std::path::Path;

pub struct Nes {
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Ppu,
    pub apu: apu::Apu,

    pub mapper: Mapper,

    pub controller: controller::Controller,

    pub frame_ready: bool,
    cycle_count: u64,
}

impl Nes {
    pub fn new(rom_path: &Path) -> Result<Nes, NesError> {
        let mut rom = File::open(rom_path)?;
        let cartridge = cartridge::parse_rom(&mut rom)?;

        let mut nes = Nes {
            cpu: cpu::Cpu::new(),
            ppu: ppu::Ppu::new(),
            apu: apu::Apu::new(),

            mapper: Nes::initialize_mapper(cartridge),

            controller: controller::Controller::new(),

            frame_ready: false,
            cycle_count: 0,
        };

        use std::mem::size_of;
        println!("Size of Nes in bytes: {}", size_of::<Nes>());

        nes.cpu_gen_reset();
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
        self.cycle_count += 1;
        if self.cycle_count == 29658 {
            self.ppu_enable_writes();
        }

        self.cpu_tick();
        for _ in 0..3 {
            self.ppu_tick();
        }

        self.apu_tick();
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
