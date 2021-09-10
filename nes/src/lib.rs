mod apu;
mod cartridge;
mod controller;
mod cpu;
mod mapper;
mod ppu;
mod replay;

#[cfg(test)]
mod tests;

use apu::Apu;
use cartridge::InesHeader;
use controller::Controller;
use cpu::Cpu;
use mapper::BaseMapper;
use ppu::Ppu;

use serde::{Deserialize, Serialize};

pub use controller::Button;
pub use ppu::PALETTE;
pub use replay::ReplayInputs;

#[derive(Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    apu: Apu,

    mapper: BaseMapper,

    controller: controller::Controller,

    frame_ready: bool,
    /// CPU cycle count
    cycle_count: u64,
    frame_count: u64,
}

// TODO: wrap inner NES into some Console struct

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

            mapper: BaseMapper::new(cartridge)?,

            controller: Controller::new(),

            frame_ready: false,
            cycle_count: 0,

            frame_count: 0,
        };

        nes.cpu_gen_reset();
        Ok(nes)
    }

    pub fn set_button_state(&mut self, button: controller::Button, state: bool) {
        self.controller.set_button(button, state);
    }

    pub fn reset(&mut self) {
        self.cpu_gen_reset();
    }

    pub fn run_one_frame(&mut self) {
        while !self.frame_ready {
            self.cpu_tick();
        }
        self.frame_ready = false;

        self.frame_count += 1;
    }

    pub fn run_cpu_cycle(&mut self) {
        self.cpu_tick();
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.ppu.output_buffer
    }

    pub fn save_state(&self) -> Result<Vec<u8>, NesError> {
        bincode::serialize(self).map_err(|_| NesError::InvalidSaveState)
    }

    pub fn load_state(save: &[u8]) -> Result<Nes, NesError> {
        let nes: Nes = bincode::deserialize(save).map_err(|_| NesError::InvalidSaveState)?;

        Ok(nes)
    }

    pub fn drive_replay_inputs(&mut self, inputs: &ReplayInputs) {
        self._drive_replay_inputs(inputs)
    }

    pub fn get_cpu_state(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn get_ines_header(&mut self) -> &InesHeader {
        &self.mapper.cartridge.header
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn get_cycle_count(&self) -> u64 {
        self.cycle_count
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
    InvalidSaveState,
}
