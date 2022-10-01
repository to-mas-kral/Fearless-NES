use bincode::{config::Configuration, Decode, Encode};
use debug_events::DebugEvent;
use thiserror::Error;

mod apu;
mod cartridge;
mod controller;
mod cpu;
#[cfg(feature = "debug_tools")]
mod debug_events;
mod mapper;
mod ppu;
mod replay;

use apu::Apu;
use cartridge::{ConsoleType, Region};
use controller::Controller;
use cpu::Cpu;
use mapper::BaseMapper;
use ppu::Ppu;

pub use cartridge::{BankSize, Cartridge, Header};
pub use controller::Button;
#[cfg(feature = "debug_tools")]
pub use debug_events::{DebugEvents, EventKind};
pub use ppu::{FRAMEBUFFER_SIZE, NES_HEIGHT, NES_WIDTH, PALETTE};
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

    #[cfg(feature = "debug_tools")]
    debug_events: DebugEvents,
}

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

            #[cfg(feature = "debug_tools")]
            debug_events: DebugEvents::new(),
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

    pub fn run_frame(&mut self) {
        while !self.frame_ready {
            self.cpu_tick();
        }

        self.on_frame_ended();
    }

    /// Runs a scanline and returns true if the next frame is ready
    pub fn run_scanline(&mut self) -> bool {
        for _ in 0..114 {
            self.cpu_tick();
        }

        let frame_ready = self.frame_ready;
        if frame_ready {
            self.on_frame_ended();
        }
        frame_ready
    }

    pub fn run_cpu_cycle(&mut self) {
        self.cpu_tick();
    }

    pub fn frame_buffer(&self) -> &[u8; ppu::FRAMEBUFFER_SIZE] {
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

    pub fn cartridge(&mut self) -> &Cartridge {
        &self.mapper.cartridge
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    pub fn apu_samples(&mut self, samples: &mut Vec<i16>) {
        self.apu.blip_buf.end_frame(samples)
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.apu
            .blip_buf
            .set_rates(apu::Apu::CLOCK_RATE, sample_rate);
    }

    #[cfg(feature = "debug_tools")]
    pub fn debug_events(&self) -> &[DebugEvent] {
        self.debug_events.events()
    }

    // TODO: Move this back to tests and have some public API for debug cpu_reads...
    pub fn run_blargg_test(&mut self) -> String {
        let mut test_running = false;

        loop {
            self.cpu_tick();

            let test_state = self.cpu_read(0x6000);
            if test_state == 0x80 {
                test_running = true;
            }

            if test_running && test_state <= 81 {
                break;
            }
        }

        let mut s = String::new();
        let mut p: usize = 0x6004;
        while self.cpu_read(p) != 0 {
            s.push(self.cpu_read(p) as char);
            p += 1;
        }

        s
    }
}

impl Nes {
    fn clock_components(&mut self) {
        self.cpu.odd_cycle = !self.cpu.odd_cycle;
        self.cycle_count += 1;
        if self.cycle_count == 29658 {
            self.ppu_enable_writes();
        }

        for _ in 0..3 {
            self.ppu_tick();
        }

        self.apu_tick();

        self.mapper.cpu_clock(&mut self.cpu.irq_mapper_signal);
    }

    fn on_frame_ended(&mut self) {
        self.frame_ready = false;
        self.frame_count += 1;

        #[cfg(feature = "debug_tools")]
        self.debug_events.on_frame_ended();
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
