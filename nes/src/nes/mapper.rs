use std::fs::File;
use std::io::Read;

use super::ines::InesHeader;
use super::NesError;

pub trait Mapper {
    fn read_prg(&mut self, adr: usize) -> u8;
    fn read_prg_direct(&mut self, adr: usize) -> u8;
    fn write_prg(&mut self, adr: usize, val: u8);
    fn write_prg_direct(&mut self, adr: usize, val: u8);
    fn read_chr(&mut self, adr: usize) -> u8;
    fn write_chr(&mut self, adr: usize, val: u8);
    fn load_cartridge(&mut self, header: InesHeader, file: &mut File) -> Result<bool, NesError>;
}

pub struct Nrom {
    pub prg: [u8; 0xBFE0],
    pub chr: [u8; 0x2000],
}

impl Nrom {
    pub fn new() -> Nrom {
        Nrom {
            prg: [0; 0xBFE0],
            chr: [0; 0x2000],
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

    //TODO: generalize this later
    fn load_cartridge(&mut self, header: InesHeader, file: &mut File) -> Result<bool, NesError> {
        let _bytes: Result<Vec<u8>, _> = file.bytes().collect();
        let bytes = _bytes?;

        match header.prg_rom_size {
            1 => {
                self.prg[0x3FDF..=(0x4000 + 0x3FDF)].clone_from_slice(&bytes[15..=(0x4000 + 15)]);
                self.prg[(0x4000 + 0x3FDF)..=(0x8000 + 0x3FDF)]
                    .clone_from_slice(&bytes[15..=(0x4000 + 15)]);
                self.chr.clone_from_slice(&bytes[0x4000 + 15..0x6000 + 15]);
            }
            2 => {
                self.prg[0x3FDF..=0xBFDF].clone_from_slice(&bytes[(15)..=(0x8000 + 15)]);
                self.chr
                    .clone_from_slice(&bytes[(0x8001 + 15)..(0x8001 + 0x2000 + 15)]);
            }
            _ => unreachable!(),
        }

        Ok(true)
    }
}

pub fn get_mapper(id: u32) -> Box<Mapper> {
    match id {
        0 => Box::new(Nrom::new()),
        _ => panic!("mapper number {} is unsupported", id),
    }
}
