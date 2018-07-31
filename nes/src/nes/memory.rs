use super::mapper::Mapper;
use super::ppu::Ppu;

use std::cell::RefCell;
use std::rc::Rc;

static MAPPING_ERROR_MSG: &str = "internal emulator error in memory mapping";

pub trait MemoryOps {
    fn read(&self, _index: usize) -> u8;
    fn read_direct(&self, _index: usize) -> u8;
    fn read_zp(&self, _index: usize) -> u8;
    fn write_zp(&self, _index: usize, value: u8);
    fn write(&self, _index: usize, value: u8);
}

pub struct Memory {
    cpu_ram: [u8; 0x800],
    mapper: Rc<RefCell<Box<Mapper>>>,
    ppu: Rc<RefCell<Ppu>>,
    pub dma_cycles: u16,
}

impl Memory {
    pub fn new(ppu: Rc<RefCell<Ppu>>, mapper: Rc<RefCell<Box<Mapper>>>) -> Memory {
        Memory {
            cpu_ram: [0; 0x800],
            mapper,
            ppu,
            dma_cycles: 0,
        }
    }
}

impl MemoryOps for Rc<RefCell<Memory>> {
    #[inline]
    fn read(&self, index: usize) -> u8 {
        let result = match index {
            0..=0x1FFF => self.borrow_mut().cpu_ram[index & 0x7FF],
            0x2000..=0x3FFF => self.borrow_mut().ppu.borrow_mut().read_reg(index),
            0x4000..=0x4017 => {
                //TODO: APU and stuff
                0
            }
            0x4018..=0x401F => 0,
            0x4020..=0xFFFF => self
                .borrow_mut()
                .mapper
                .borrow_mut()
                .read_prg(index - 0x4020),
            _ => panic!(MAPPING_ERROR_MSG),
        };

        debug_log!("memory map - reading 0x{:X} from 0x{:X}", result, index);

        result
    }

    #[inline]
    fn write(&self, index: usize, value: u8) {
        debug_log!("memory map - writing 0x{:X} to 0x{:X}", value, index);

        match index {
            0..=0x1FFF => self.borrow_mut().cpu_ram[index & 0x7FF] = value,
            0x2000..=0x3FFF => self.borrow_mut().ppu.borrow_mut().write_reg(index, value),
            0x4000..=0x4017 => {
                //TODO: APU and stuff
                match index {
                    0x4014 => {
                        //TODO: add 1 cycle on odd cpu cycle
                        //TODO: make this cycle accurate
                        //TODO: implement DMA in the state machine
                        self.borrow_mut().dma_cycles = 513;
                        for i in 0..256 {
                            let oamaddr = self.borrow_mut().ppu.borrow_mut().oamaddr as usize;
                            self.borrow_mut().ppu.borrow_mut().oam[oamaddr.wrapping_add(i)] =
                                self.read((usize::from(value) << 8) + i);
                        }
                    }
                    _ => (),
                }
            }
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => {
                self.borrow_mut()
                    .mapper
                    .borrow_mut()
                    .write_prg(index - 0x4020, value);
            }
            _ => panic!(MAPPING_ERROR_MSG),
        }
    }

    #[inline]
    fn read_zp(&self, index: usize) -> u8 {
        debug_log!("memory map - reading from zero page 0x{:X}", index);
        self.borrow_mut().cpu_ram[index]
    }

    #[inline]
    fn write_zp(&self, index: usize, val: u8) {
        debug_log!(
            "memory map - writing 0x{:X} to zero page 0x{:X}",
            val,
            index
        );
        self.borrow_mut().cpu_ram[index] = val;
    }

    #[inline]
    fn read_direct(&self, index: usize) -> u8 {
        debug_log!("memory map - reading direct from 0x{:X}", index);
        match index {
            0..=0x1FFF => self.borrow_mut().cpu_ram[index & 0x7FF],
            0x4020..=0xFFFF => self
                .borrow_mut()
                .mapper
                .borrow_mut()
                .read_prg_direct(index - 0x4020),
            _ => panic!("internal emulator error that should never ever ever happen"),
        }
    }
}
