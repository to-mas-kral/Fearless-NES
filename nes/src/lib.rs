mod apu;
mod cartridge;
mod controller;
mod cpu;
mod mapper;
mod ppu;

#[cfg(test)]
mod tests;

use apu::Apu;
use cartridge::InesHeader;
use controller::Controller;
use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;

use serde::{Deserialize, Serialize};

pub use controller::Button;
pub use ppu::PALETTE;

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

/*
    Public API
*/
impl Nes {
    pub fn new(rom: &[u8]) -> Result<Nes, NesError> {
        let cartridge = cartridge::parse_rom(rom)?;

        let mut nes = Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),

            mapper: Nes::initialize_mapper(cartridge)?,

            controller: Controller::new(),

            frame_ready: false,
            cycle_count: 0,
        };

        nes.cpu_gen_reset();
        nes.cpu_reset_routine();
        Ok(nes)
    }

    pub fn set_button_state(&mut self, button: controller::Button, state: bool) {
        self.controller.set_button(button, state);
    }

    pub fn run_one_frame(&mut self) {
        while !self.frame_ready {
            self.cpu_tick();
        }
        self.frame_ready = false;
    }

    pub fn run_cpu_cycle(&mut self) {
        self.cpu_tick();
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.ppu.output_buffer
    }

    pub fn load_state(&mut self, save: &[u8]) -> Result<(), Box<bincode::ErrorKind>> {
        let nes: Nes = bincode::deserialize(&save)?;

        *self = nes;
        self.reaload_mapper_pointers();

        Ok(())
    }

    pub fn save_state(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }

    pub fn get_cpu_state(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn get_ines_header(&mut self) -> &InesHeader {
        &self.mapper.cartridge.header
    }
}

impl Nes {
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
    Ines2Unsupported,
    TrainerUnsupported,
    PalUnsupported,
    UnSupportedMapper(u32),
    InvalidInesFormat,
    InvalidRomSize,
}
