use std::fs::File;
use std::io::Read;

use super::ines::InesHeader;
use super::ines::Mirroring;
use super::NesError;

pub trait Mapper {
    fn read_prg(&mut self, adr: usize) -> u8;
    fn read_prg_direct(&mut self, adr: usize) -> u8;
    fn write_prg(&mut self, adr: usize, val: u8);
    fn write_prg_direct(&mut self, adr: usize, val: u8);
    fn read_chr(&mut self, adr: usize) -> u8;
    fn write_chr(&mut self, adr: usize, val: u8);
    fn read_nametable(&mut self, adr: usize) -> u8;
    fn write_nametable(&mut self, adr: usize, val: u8);
    fn load_cartridge(&mut self, file: &mut File) -> Result<bool, NesError>;
}

pub struct Nrom {
    pub prg: [u8; 0xBFE0],
    pub chr: [u8; 0x2000],
    pub nt_ram: [u8; 0x1000],
    pub header: InesHeader,
}

impl Nrom {
    pub fn new(header: InesHeader) -> Nrom {
        Nrom {
            prg: [0; 0xBFE0],
            chr: [0; 0x2000],
            nt_ram: [0; 0x1000],
            header,
        }
    }
}

impl Mapper for Nrom {
    fn read_prg_direct(&mut self, adr: usize) -> u8 {
        self.prg[adr]
    }

    fn read_prg(&mut self, adr: usize) -> u8 {
        self.prg[adr]
    }

    fn write_prg_direct(&mut self, adr: usize, val: u8) {
        self.prg[adr] = val;
    }

    fn write_prg(&mut self, adr: usize, val: u8) {
        self.prg[adr] = val;
    }

    fn read_chr(&mut self, adr: usize) -> u8 {
        self.chr[adr]
    }

    fn write_chr(&mut self, adr: usize, val: u8) {
        self.chr[adr] = val;
    }

    fn read_nametable(&mut self, adr: usize) -> u8 {
        self.nt_ram[adr]
    }

    fn write_nametable(&mut self, adr: usize, val: u8) {
        match self.header.mirroring {
            Mirroring::Vertical => match adr {
                0..=0x3FF => {
                    self.nt_ram[adr] = val;
                    self.nt_ram[adr + 0x800] = val;
                }
                0x400..=0x7FF => {
                    self.nt_ram[adr] = val;
                    self.nt_ram[adr + 0x800] = val;
                }
                0x800..=0xBFF => {
                    self.nt_ram[adr] = val;
                    self.nt_ram[adr - 0x400] = val;
                }
                0xC00..=0xFFF => {
                    self.nt_ram[adr] = val;
                    self.nt_ram[adr - 0x800] = val;
                }
                _ => unreachable!(),
            },
            _ => (),
        }
    }

    //TODO: generalize this later
    fn load_cartridge(&mut self, file: &mut File) -> Result<bool, NesError> {
        let _bytes: Result<Vec<u8>, _> = file.bytes().collect();
        let bytes = _bytes?;

        match self.header.prg_rom_size {
            1 => {
                self.prg[0x3FDF..=(0x4000 + 0x3FDF)].clone_from_slice(&bytes[15..=(0x4000 + 15)]);
                self.prg[(0x4000 + 0x3FDF)..=(0x8000 + 0x3FDF)]
                    .clone_from_slice(&bytes[15..=(0x4000 + 15)]);
                if self.header.chr_rom_size != 0 {
                    self.chr
                        .clone_from_slice(&bytes[(0x4001 + 15)..(0x6001 + 15)]);
                }
            }
            2 => {
                self.prg[0x3FDF..=0xBFDF].clone_from_slice(&bytes[(15)..=(0x8000 + 15)]);
                if self.header.chr_rom_size != 0 {
                    self.chr
                        .clone_from_slice(&bytes[(0x8001 + 15)..(0x8001 + 0x2000 + 15)]);
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
