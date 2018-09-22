use std::fs::File;
use std::io::Read;

use super::ines::InesHeader;
use super::ines::Mirroring;
use super::NesError;

pub trait Mapper {
    fn cpu_read(&mut self, addr: usize) -> Option<u8>;
    fn cpu_read_direct(&mut self, addr: usize) -> u8;
    fn cpu_write(&mut self, addr: usize, val: u8);
    fn read_chr(&mut self, addr: usize) -> u8;
    fn write_chr(&mut self, addr: usize, val: u8);
    fn read_nametable(&mut self, addr: usize) -> u8;
    fn write_nametable(&mut self, addr: usize, val: u8);
    fn load_cartridge(&mut self, file: &mut File) -> Result<bool, NesError>;
}

pub struct Nrom {
    pub prg: [u8; 0x8000],
    pub chr: [u8; 0x2000],
    pub ram: [u8; 0x2000],
    pub nt_ram: [u8; 0x1000],
    pub header: InesHeader,
}

impl Nrom {
    pub fn new(header: InesHeader) -> Nrom {
        Nrom {
            prg: [0; 0x8000],
            chr: [0; 0x2000],
            ram: [0; 0x2000],
            nt_ram: [0; 0x1000],
            header,
        }
    }
}

impl Mapper for Nrom {
    fn cpu_read_direct(&mut self, addr: usize) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.ram[addr - 0x6000],
            0x8000..=0xFFFF => self.prg[addr - 0x8000],
            _ => 0,
        }
    }

    fn cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => Some(self.ram[addr - 0x6000]),
            0x8000..=0xFFFF => Some(self.prg[addr - 0x8000]),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: usize, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.ram[addr - 0x6000] = val,
            _ => (),
        }
    }

    fn read_chr(&mut self, addr: usize) -> u8 {
        self.chr[addr]
    }

    fn write_chr(&mut self, addr: usize, val: u8) {
        self.chr[addr] = val;
    }

    fn read_nametable(&mut self, addr: usize) -> u8 {
        self.nt_ram[addr]
    }

    fn write_nametable(&mut self, addr: usize, val: u8) {
        match self.header.mirroring {
            Mirroring::Vertical => match addr {
                0..=0x3FF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr + 0x800] = val;
                }
                0x400..=0x7FF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr + 0x800] = val;
                }
                0x800..=0xBFF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr - 0x400] = val;
                }
                0xC00..=0xFFF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr - 0x800] = val;
                }
                _ => unreachable!(),
            },
            Mirroring::Horizontal => match addr {
                0..=0x3FF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr + 0x400] = val;
                }
                0x400..=0x7FF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr - 0x400] = val;
                }
                0x800..=0xBFF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr + 0x400] = val;
                }
                0xC00..=0xFFF => {
                    self.nt_ram[addr] = val;
                    self.nt_ram[addr - 0x400] = val;
                }
                _ => unreachable!(),
            },
            _ => (),
        }
    }

    //TODO: generalize cartridge loading later
    fn load_cartridge(&mut self, file: &mut File) -> Result<bool, NesError> {
        let _bytes: Result<Vec<u8>, _> = file.bytes().collect();
        let bytes = _bytes?;

        match self.header.prg_rom_size {
            1 => {
                self.prg[0..0x4000].clone_from_slice(&bytes[16..(0x4000 + 16)]);
                self.prg[0x4000..0x8000].clone_from_slice(&bytes[16..(0x4000 + 16)]);
                if self.header.chr_rom_size != 0 {
                    self.chr
                        .clone_from_slice(&bytes[(0x4000 + 16)..(0x6000 + 16)]);
                }
            }
            2 => {
                self.prg[0..0x8000].clone_from_slice(&bytes[(16)..(0x8000 + 16)]);
                if self.header.chr_rom_size != 0 {
                    self.chr
                        .clone_from_slice(&bytes[(0x8000 + 16)..(0x8000 + 0x2000 + 16)]);
                }
            }
            _ => unreachable!(),
        }

        Ok(true)
    }
}

pub fn get_mapper(header: InesHeader) -> Box<Mapper> {
    match header.mapper {
        0 => Box::new(Nrom::new(header)),
        _ => panic!("mapper number {} is unsupported", header.mapper),
    }
}
