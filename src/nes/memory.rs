use super::mapper::Mapper;
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
    fn write_zp(&self, _index: usize, value: u8);
    fn write(&self, _index: usize, value: u8);
}

pub struct Memory<M>
where
    M: Mapper,
{
    cpu_ram: [u8; 0x800],
    cartridge_space: [u8; 0xBFE0],
    mapper: Rc<RefCell<M>>,
    ppu: Rc<RefCell<Ppu<M>>>,
    pub dma_cycles: u16,
}

impl<M> Memory<M>
where
    M: Mapper,
{
    pub fn new(ppu: Rc<RefCell<Ppu<M>>>, mapper: Rc<RefCell<M>>) -> Memory<M> {
        Memory {
            cpu_ram: [0; 0x800],
            cartridge_space: [0; 0xBFE0],
            mapper,
            ppu,
            dma_cycles: 0,
        }
    }

    pub fn clear(&mut self) {
        self.cpu_ram = [0; 0x800];
        self.cartridge_space = [0; 0xBFE0];
    }

    pub fn load_mapper_0_1(&mut self, f: &mut File) {
        let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
        let bytes = _bytes.expect("error while reading the file");

        for i in 0..0x4000 {
            self.cartridge_space[i] = bytes[15 + i];
            self.cartridge_space[0x4000 + i] = bytes[15 + i];
        }

        for i in 0..0x2000 {
            self.mapper.borrow_mut().write_chr(i, bytes[0x4001 + i]);
        }
    }

    pub fn load_mapper_0(&mut self, f: &mut File) {
        let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
        let bytes = _bytes.expect("error while reading the file");
        self.cartridge_space[0x3FDF..(0x3FDF + 0x8001)].clone_from_slice(&bytes[15..(0x8001 + 15)]);
        for i in (0x8001 + 15)..(0x8001 + 0x2000 + 15) {
            self.mapper
                .borrow_mut()
                .write_chr(i - (0x8001 + 15), bytes[i])
        }
    }
}

impl<M> MemoryOps for Rc<RefCell<Memory<M>>>
where
    M: Mapper,
{
    #[inline]
    fn read(&self, index: usize) -> u8 {
        /* println!(
            ":::: reading from 0x{:X}",
            index
        ); */

        match index {
            adr @ 0..=0x1FFF => {
                let base = (adr & 0x0FFF) % 0x800;
                self.borrow_mut().cpu_ram[base]
            }
            adr @ 0x2000..=0x3FFF => {
                let base = adr & 0x8;
                match 0x2000 + base {
                    0x2000 => 0,
                    0x2001 => 0,
                    0x2002 => self.borrow_mut().ppu.borrow_mut().mem.read_ppustatus(),
                    0x2003 => 0,
                    0x2004 => self.borrow_mut().ppu.borrow_mut().mem.read_oamdata(),
                    0x2005 => 0,
                    0x2006 => 0,
                    0x2007 => self.borrow_mut().ppu.borrow_mut().mem.read_ppudata(),
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
                self.borrow_mut().cartridge_space[index - 0x4020]
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
                self.borrow_mut().cpu_ram[base] = value;
            }
            adr @ 0x2000..=0x3FFF => {
                let base = adr & 0x8;
                match 0x2000 + base {
                    0x2000 => self.borrow_mut().ppu.borrow_mut().mem.write_ppuctrl(value),
                    0x2001 => self.borrow_mut().ppu.borrow_mut().mem.write_ppumask(value),
                    0x2002 => (),
                    0x2003 => self.borrow_mut().ppu.borrow_mut().mem.write_oamaddr(value),
                    0x2004 => self.borrow_mut().ppu.borrow_mut().mem.write_oamdata(value),
                    0x2005 => self
                        .borrow_mut()
                        .ppu
                        .borrow_mut()
                        .mem
                        .write_ppuscroll(value),
                    0x2006 => self.borrow_mut().ppu.borrow_mut().mem.write_ppuaddr(value),
                    0x2007 => self.borrow_mut().ppu.borrow_mut().mem.write_ppudata(value),
                    _ => panic!(MAPPING_ERROR_MSG),
                }
            }
            adr @ 0x4000..=0x4017 => {
                //TODO: APU and stuff
                match adr {
                    0x4014 => {
                        //TODO: add 1 cycle on odd cpu cycle
                        self.borrow_mut().dma_cycles = 513;
                        for i in 0..256 {
                            let oamaddr = self.borrow_mut().ppu.borrow_mut().mem.oamaddr as usize;
                            self.borrow_mut().ppu.borrow_mut().mem.oam[oamaddr.wrapping_add(i)] =
                                self.read((usize::from(value) << 8) + i);
                        }
                    }
                    _ => (),
                }
            }
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => {
                //TODO: some mapper register shenanigans here
                self.borrow_mut().cartridge_space[index - 0x4020] = value;
            }
            _ => panic!(MAPPING_ERROR_MSG),
        }
    }

    #[inline]
    fn read_zp(&self, index: usize) -> u8 {
        self.borrow_mut().cpu_ram[index]
    }

    #[inline]
    fn write_zp(&self, index: usize, val: u8) {
        self.borrow_mut().cpu_ram[index] = val;
    }

    #[inline]
    fn read_direct(&self, index: usize) -> u8 {
        match index {
            adr @ 0..=0x1FFF => {
                let base = (adr & 0x0FFF) % 0x800;
                self.borrow_mut().cpu_ram[base]
            }
            0x4020..=0xFFFF => self.borrow_mut().cartridge_space[index - 0x4020],
            _ => panic!("internal emulator error that should never ever ever happen"),
        }
    }
}
