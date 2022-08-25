use bincode::{config::Configuration, Decode, Encode};
use thiserror::Error;

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
use cartridge::{ConsoleType, Region};
use controller::Controller;
use cpu::Cpu;
use mapper::BaseMapper;
use ppu::Ppu;

pub use cartridge::{BankSize, Cartridge, Header};
pub use controller::Button;
pub use ppu::PALETTE;
pub use replay::ReplayInputs;

#[derive(Encode, Decode)]
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

impl Nes {
    pub fn new(rom: &[u8]) -> Result<Nes, NesError> {
        let cartridge = Cartridge::from_rom(rom)?;

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
        bincode::encode_to_vec(self, BINCODE_CONFIG).map_err(|_| NesError::InvalidSaveState)
    }

    pub fn load_state(save: &[u8]) -> Result<Nes, NesError> {
        let (nes, _): (Nes, usize) = bincode::decode_from_slice(save, BINCODE_CONFIG)
            .map_err(|_| NesError::InvalidSaveState)?;

        Ok(nes)
    }

    pub fn drive_replay_inputs(&mut self, inputs: &ReplayInputs) {
        self._drive_replay_inputs(inputs)
    }

    pub fn get_cartridge(&mut self) -> &Cartridge {
        &self.mapper.cartridge
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

#[derive(Error, Debug)]
pub enum NesError {
    #[error("iNES 2.0 binary format is not supported")]
    Ines2Unsupported,
    #[error("iNES trainers are not supported")]
    TrainerUnsupported,
    #[error("mapper {0} is not supported")]
    UnSupportedMapper(u32),
    #[error("console type {0} is not supported")]
    ConsoleUnsupported(ConsoleType),
    #[error("the {0} region is not supported")]
    RegionUnsupported(Region),
    #[error("the provided file is not a valid iNES ROM")]
    InvalidInesFormat,
    #[error("games with both CHR RAM and ROM are not supported")]
    ChrRomAndRamUnsupported,
    #[error("corrupted ROM file")]
    RomCorrupted,
    #[error("the provided savestate is corrupted, or it has been created by a incompatible version of Fearless-NES")]
    InvalidSaveState,
    #[error("the NES 2.0 XML Game Database contains invalid data")]
    GameDbFormat,
}

const BINCODE_CONFIG: Configuration = bincode::config::standard();
