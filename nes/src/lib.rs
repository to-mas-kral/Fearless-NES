mod tests;

pub mod apu;
pub mod cartridge;
pub mod controller;
pub mod cpu;
pub mod mapper;
pub mod ppu;

use mapper::Mapper;

use apu::Apu;
use controller::Controller;
use cpu::Cpu;
use ppu::Ppu;

use std::{
    fs::{self, File},
    io,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    apu: Apu,

    mapper: Mapper,

    controller: controller::Controller,

    frame_ready: bool,
    cycle_count: u64,
}

impl Nes {
    /*
        Public API
    */
    pub fn new(rom_path: &str) -> Result<Nes, NesError> {
        // TODO: remove file loading logic, accept vector of bytes
        // TODO: error handling
        let mut rom = File::open(Path::new(rom_path))?;
        let cartridge = cartridge::parse_rom(&mut rom)?;

        let mut nes = Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),

            mapper: Nes::initialize_mapper(cartridge),

            controller: Controller::new(),

            frame_ready: false,
            cycle_count: 0,
        };

        nes.cpu_gen_reset();
        nes.cpu_reset_routine();
        Ok(nes)
    }

    pub fn set_controller_state(&mut self, keycode: controller::Keycode, state: bool) {
        self.controller.set_button(keycode, state);
    }

    pub fn run_one_frame(&mut self) {
        while !self.frame_ready {
            self.cpu_tick();
        }
        self.frame_ready = false;
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.ppu.output_buffer
    }

    pub fn load_state(&mut self, save_path: &str) {
        let save = fs::read(save_path).unwrap();

        let nes: Nes = bincode::deserialize(&save).unwrap();

        *self = nes;
        self.reaload_mapper_pointers();
    }

    pub fn save_state(&mut self, save_path: &str) {
        let nes = bincode::serialize(self).unwrap();

        fs::write(save_path, nes).unwrap();
    }

    fn clock_ppu_apu(&mut self) {
        self.cpu.odd_cycle = !self.cpu.odd_cycle;
        self.cycle_count += 1;
        if self.cycle_count == 29658 {
            self.ppu_enable_writes();
        }

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
