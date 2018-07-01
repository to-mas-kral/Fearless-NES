use super::ppu::Ppu;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

static MAPPING_ERROR_MSG: &str = "internal emulator error in memory mapping";

pub trait MemoryOps {
    fn read(&self, _index: usize) -> u8;
    fn read_direct(&self, _index: usize) -> u8;
    fn read_zp(&self, _index: usize) -> u8;
    fn write(&self, _index: usize, value: u8);
    fn write_direct(&self, _index: usize, value: u8);
}

pub struct Memory {
    mem: [u8; 0x10000],
    ppu: Rc<RefCell<Ppu>>,
}

impl Memory {
    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Memory {
        Memory {
            mem: [0; 0x10000],
            ppu,
        }
    }

    pub fn clear(&mut self) {
        self.mem = [0; 0x10000];
    }

    pub fn load_mapper_0_1(&mut self, f: &mut File) {
        let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
        let bytes = _bytes.expect("error while reading the file");

        for i in 0..16384 {
            self.mem[0x7FFF + i] = bytes[15 + i];
            self.mem[0x7FFF + 0x4000 + i] = bytes[15 + i];
        }
    }

    pub fn load_mapper_0(&mut self, f: &mut File) {
        let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
        let bytes = _bytes.expect("error while reading the file");
        self.mem[32767..(32769 + 32767)].clone_from_slice(&bytes[15..(32769 + 15)])
    }
}

impl MemoryOps for Rc<RefCell<Memory>> {
    #[inline]
    fn read(&self, index: usize) -> u8 {
        /* println!(
            ":::: reading 0x{:X} from 0x{:X}",
            self.borrow_mut().mem[index],
            index
        ); */

        match index {
            0..=0x1FFF => self.borrow_mut().mem[index],
            adr @ 0x2000..=0x3FFF => {
                let base = adr & 0x8;
                match 0x2000 + base {
                    0x2000 => 0,
                    0x2001 => 0,
                    0x2002 => self.borrow_mut().ppu.borrow_mut().regs.read_ppustatus(),
                    0x2003 => 0,
                    0x2004 => self.borrow_mut().ppu.borrow_mut().regs.read_oamdata(),
                    0x2005 => 0,
                    0x2006 => 0,
                    0x2007 => self.borrow_mut().ppu.borrow_mut().regs.read_ppudata(),
                    _ => panic!(MAPPING_ERROR_MSG),
                }
            }
            0x4000..=0x4017 => {
                //TODO: APU and stuff
                0
            }
            0x4018..=0x401F => 0,
            0x4020..=0xFFFF => {
                //TODO: some mapper register shenanigans here
                self.borrow_mut().mem[index]
            }
            _ => panic!(MAPPING_ERROR_MSG),
        }
    }

    #[inline]
    fn write(&self, index: usize, value: u8) {
        //println!(":::: writing 0x{:X} to 0x{:X}", value, index);

        match index {
            adr @ 0..=0x1FFF => {
                let base = (adr & 0x0FFF) % 0x800;
                self.borrow_mut().mem[base] = value;
                self.borrow_mut().mem[0x800 + base] = value;
                self.borrow_mut().mem[0x1000 + base] = value;
                self.borrow_mut().mem[0x1800 + base] = value;
            }
            adr @ 0x2000..=0x3FFF => {
                let base = adr & 0x8;
                match 0x2000 + base {
                    0x2000 => self.borrow_mut().ppu.borrow_mut().regs.write_ppuctrl(value),
                    0x2001 => self.borrow_mut().ppu.borrow_mut().regs.write_ppumask(value),
                    0x2002 => (),
                    0x2003 => self.borrow_mut().ppu.borrow_mut().regs.write_oamaddr(value),
                    0x2004 => self.borrow_mut().ppu.borrow_mut().regs.write_oamdata(value),
                    0x2005 => self.borrow_mut().ppu.borrow_mut().regs.write_ppuscroll(value),
                    0x2006 => self.borrow_mut().ppu.borrow_mut().regs.write_ppuaddr(value),
                    0x2007 => self.borrow_mut().ppu.borrow_mut().regs.write_ppudata(value),
                    _ => panic!(MAPPING_ERROR_MSG),
                }
            }
            0x4000..=0x4017 => {
                //TODO: APU and stuff
            }
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => {
                //TODO: some mapper register shenanigans here
                self.borrow_mut().mem[index] = value;
            }
            _ => panic!(MAPPING_ERROR_MSG),
        }

        self.borrow_mut().mem[index] = value;
    }

    #[inline]
    fn read_zp(&self, index: usize) -> u8 {
        //TODO: how does this differ ??
        /* println!(
            ":::: reading 0x{:X} from 0x{:X}",
            self.borrow_mut().mem[index],
            index
        ); */
        self.borrow_mut().mem[index]
    }

    #[inline]
    fn read_direct(&self, index: usize) -> u8 {
        self.borrow_mut().mem[index]
    }

    #[inline]
    fn write_direct(&self, index: usize, value: u8) {
        self.borrow_mut().mem[index] = value;
    }
}
