use serde::{Deserialize, Serialize};

use std::{fs::File, io::Read};

use super::NesError;

pub(crate) fn parse_rom(f: &mut File) -> Result<Cartridge, NesError> {
    let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
    let rom = _bytes?;

    let header = parse_header(&rom)?;

    let rom: Vec<u8> = rom.iter().cloned().skip(16).collect();

    let prg_end = 0x4000 * header.prg_rom_count as usize;

    let mut prg_rom = Vec::new();
    prg_rom.extend_from_slice(&rom[0..prg_end]);

    let chr = if header.chr_rom_count != 0 {
        let chr_end = prg_end + 0x2000 * header.chr_rom_count as usize;
        let mut chr = Vec::new();
        chr.extend_from_slice(&rom[prg_end..chr_end]);
        chr
    } else {
        let chr = vec![0; 0x2000];
        chr
    };

    let prg_ram = if header.prg_ram_count != 0 {
        vec![0; 0x2000 * header.prg_ram_count as usize]
    } else {
        vec![0; 0x2000]
    };

    Ok(Cartridge {
        header,

        prg_rom,
        prg_ram,
        chr,
    })
}

// TODO cleanup cartridge api

#[derive(Serialize, Deserialize)]
pub struct Cartridge {
    pub header: InesHeader,

    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr: Vec<u8>,
}

// Banks are indexed from 0
impl Cartridge {
    pub fn read_prg_rom(&self, addr: usize, bank: u8, bank_size: BankSize) -> u8 {
        self.prg_rom[addr + bank as usize * bank_size as usize]
    }

    pub fn read_prg_ram(&self, addr: usize, bank: u8, bank_size: BankSize) -> u8 {
        self.prg_ram[addr + bank as usize * bank_size as usize]
    }

    pub fn write_prg_ram(&mut self, addr: usize, bank: u8, bank_size: BankSize, val: u8) {
        self.prg_ram[addr + bank as usize * bank_size as usize] = val;
    }

    pub fn read_chr(&self, addr: usize, bank: u8, bank_size: BankSize) -> u8 {
        self.chr[addr + bank as usize * bank_size as usize]
    }

    pub fn write_chr(&mut self, addr: usize, bank: u8, bank_size: BankSize, val: u8) {
        self.chr[addr + bank as usize * bank_size as usize] = val
    }

    pub fn prg_rom_bank_count(&self, bank_size: BankSize) -> u8 {
        ((self.header.prg_rom_count as usize * 0x4000) / bank_size as usize) as u8
    }
}

pub enum BankSize {
    Kb1 = 0x400,
    Kb2 = 0x800,
    Kb4 = 0x1000,
    Kb8 = 0x2000,
    Kb16 = 0x4000,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InesHeader {
    pub prg_rom_count: u8, // 16 KB units
    pub prg_ram_count: u8, // 8 KB units
    pub chr_rom_count: u8, // 8 KB units
    pub mirroring: Mirroring,
    pub has_battery: bool,
    pub has_trainer: bool,
    pub mapper_id: u32,
}

static NES_CONSTANT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

fn parse_header(rom: &Vec<u8>) -> Result<InesHeader, NesError> {
    if rom.len() < 16 {
        return Err(NesError::InvalidFile);
    }

    for i in 0..3 {
        if rom[i] != NES_CONSTANT[i] {
            return Err(NesError::InvalidFile);
        }
    }

    for i in 10..15 {
        if rom[i] != 0 {
            println!("iNES 2.0 rom file");
            break;
            //TODO: handle iNES 2.0
        }
    }

    let mirroring = if rom[6] & (1 << 3) != 0 {
        Mirroring::FourScreen
    } else if rom[6] & 1 != 0 {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };

    let has_battery = rom[6] & 2 != 0;
    let has_trainer = rom[6] & (1 << 2) != 0;
    if has_trainer {
        panic!("ROM has trainer, unsupported");
    }

    let mapper = u32::from((rom[6] >> 4) | (rom[7] & 0xF0));

    if rom[9] != 0 {
        return Err(NesError::PalRom);
    }

    Ok(InesHeader {
        prg_rom_count: rom[4],
        chr_rom_count: rom[5],
        prg_ram_count: rom[8],
        mirroring,
        has_battery,
        has_trainer,
        mapper_id: mapper,
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenLow,
    SingleScreenHigh,
    FourScreen,
    Obscure,
}
