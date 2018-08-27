pub mod cpu;
use self::cpu::Tick;

pub mod apu;
pub mod controller;
pub mod ines;
pub mod mapper;
pub mod ppu;

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::rc::Rc;

pub struct Nes {
    pub frame: Rc<RefCell<Frame>>,
    pub cpu: cpu::Cpu,
    pub controller: Rc<RefCell<controller::Controller>>,
    int_bus: Rc<RefCell<InterruptBus>>,
    apu_cycle: bool,
    cycle_count: u64,
}

impl Nes {
    pub fn new(rom_path: &Path) -> Result<Nes, NesError> {
        let mut file = File::open(rom_path)?;
        let header = ines::parse_header(&mut file)?;

        let mapper = Rc::new(RefCell::new(mapper::get_mapper(header)));

        let mut file = File::open(rom_path)?;
        mapper.borrow_mut().load_cartridge(&mut file)?;

        let frame = Rc::new(RefCell::new(Frame::new()));

        let int_bus = Rc::new(RefCell::new(InterruptBus::new()));
        let ppu = ppu::Ppu::new(int_bus.clone(), mapper.clone(), frame.clone());

        let apu = apu::Apu::new(int_bus.clone());

        let controller = Rc::new(RefCell::new(controller::Controller::new()));

        let mut cpu = cpu::Cpu::new(
            int_bus.clone(),
            apu,
            controller.clone(),
            mapper.clone(),
            ppu,
        );

        let mut nes = Nes {
            frame,
            cpu,
            controller,
            int_bus,
            apu_cycle: false,
            cycle_count: 0,
        };

        nes.cpu.gen_reset();
        for _ in 0..6 {
            nes.run_one_cpu_cycle();
        }

        Ok(nes)
    }

    pub fn set_controller_state(&mut self, keycode: controller::Keycode, state: bool) {
        self.controller.borrow_mut().set_button(keycode, state);
    }

    pub fn get_framebuffer(&self) -> Rc<RefCell<Frame>> {
        self.frame.clone()
    }

    pub fn run_one_frame(&mut self) {
        while !self.frame.borrow().frame_ready {
            self.run_one_cpu_cycle();
        }
        self.frame.borrow_mut().frame_ready = false;
    }

    pub fn run_one_cpu_cycle(&mut self) {
        self.cpu.tick();
        self.cycle_count += 1;
        if self.cycle_count == 29658 {
            self.cpu.ppu.enable_writes();
        }

        for _ in 0..3 {
            self.cpu.ppu.tick();
        }

        if self.apu_cycle {
            self.cpu.apu.tick();
        }
        self.apu_cycle = !self.apu_cycle;
    }
}

#[derive(Clone)]
pub struct Frame {
    pub output_buffer: [(u8, u8, u8); 256 * 240],
    pub frame_ready: bool,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            output_buffer: [(0, 0, 0); 256 * 240],
            frame_ready: false,
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
    PalRom,
    InvalidFile,
}

impl From<io::Error> for NesError {
    fn from(error: io::Error) -> Self {
        NesError::IoError(error)
    }
}
