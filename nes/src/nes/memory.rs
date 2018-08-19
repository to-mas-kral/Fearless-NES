use super::apu::Apu;
use super::controller::Controller;
use super::mapper::Mapper;
use super::ppu::Ppu;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Memory {
    cpu_ram: [u8; 0x800],
    open_bus: u8,
    pub apu: Apu,
    controller: Rc<RefCell<Controller>>,
    mapper: Rc<RefCell<Box<Mapper>>>,
    pub ppu: Ppu,
    pub dma_cycles: u16,
}

impl Memory {
    pub fn new(
        apu: Apu,
        controller: Rc<RefCell<Controller>>,
        mapper: Rc<RefCell<Box<Mapper>>>,
        ppu: Ppu,
    ) -> Memory {
        Memory {
            cpu_ram: [0; 0x800],
            open_bus: 0,
            apu,
            controller,
            mapper,
            ppu,
            dma_cycles: 0,
        }
    }

    #[inline]
    pub fn read(&mut self, index: usize) -> u8 {
        self.open_bus = match index {
            0..=0x1FFF => self.cpu_ram[index & 0x7FF],
            0x2000..=0x3FFF => self.ppu.read_reg(index),
            0x4000..=0x4014 => self.open_bus,
            0x4015 => self.apu.read_status(),
            0x4016 | 0x4017 => {
                let tmp = self.controller.borrow_mut().read_reg();
                (self.open_bus & 0xE0) | tmp
            }
            0x4018..=0x401F => self.open_bus,
            0x4020..=0xFFFF => self.mapper.borrow_mut().read_prg(index - 0x4020),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        };

        debug_log!(
            "memory map - reading 0x{:X} from 0x{:X}",
            (self.open_bus),
            index
        );

        self.open_bus
    }

    #[inline]
    pub fn write(&mut self, index: usize, val: u8) {
        debug_log!("memory map - writing 0x{:X} to 0x{:X}", val, index);

        match index {
            0..=0x1FFF => self.cpu_ram[index & 0x7FF] = val,
            0x2000..=0x3FFF => self.ppu.write_reg(index, val),
            0x4000..=0x4015 => {
                match index {
                    0x4000..=0x4013 | 0x4015 => self.apu.write_reg(index, val),
                    0x4014 => {
                        //TODO: add 1 cycle on odd cpu cycle, make this cycle accurate, implement DMA in the state machine
                        self.dma_cycles = 513;
                        for i in 0..256 {
                            let oamaddr = self.ppu.oamaddr as usize;
                            self.ppu.oam[oamaddr.wrapping_add(i)] =
                                self.read((usize::from(val) << 8) + i);
                        }
                    }
                    _ => (),
                }
            }
            0x4016 => self.controller.borrow_mut().write_reg(val),
            0x4017 => self.apu.write_reg(index, val),
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => self.mapper.borrow_mut().write_prg(index - 0x4020, val),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    #[inline]
    pub fn read_zp(&mut self, index: usize) -> u8 {
        debug_log!("memory map - reading from zero page 0x{:X}", index);
        self.cpu_ram[index]
    }

    #[inline]
    pub fn write_zp(&mut self, index: usize, val: u8) {
        debug_log!(
            "memory map - writing 0x{:X} to zero page 0x{:X}",
            val,
            index
        );
        self.cpu_ram[index] = val;
    }

    #[inline]
    pub fn read_direct(&mut self, index: usize) -> u8 {
        debug_log!("memory map - reading direct from 0x{:X}", index);
        match index {
            0..=0x1FFF => self.cpu_ram[index & 0x7FF],
            0x4020..=0xFFFF => self.mapper.borrow_mut().read_prg_direct(index - 0x4020),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }
}
