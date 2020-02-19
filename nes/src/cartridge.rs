use crate::{ppu::Mirroring, NesError};

use serde::{Deserialize, Serialize};

const HEADER_SIZE: usize = 16;

pub(crate) fn parse_rom(rom: &[u8]) -> Result<Cartridge, NesError> {
    if rom.len() < HEADER_SIZE {
        return Err(NesError::InvalidInesFormat);
    }

    let (header, rom) = rom.split_at(HEADER_SIZE);
    let header = parse_header(header)?;

    let prg_end = (BankSize::Kb16 as usize) * header.prg_rom_count as usize;
    let mut prg_rom = Vec::with_capacity(prg_end);
    prg_rom.extend_from_slice(&rom.get(0..prg_end).ok_or(NesError::InvalidRomSize)?);

    let chr = if header.chr_rom_count != 0 {
        let chr_end = prg_end + (BankSize::Kb8 as usize) * header.chr_rom_count as usize;
        let mut chr = Vec::with_capacity(chr_end - prg_end);
        chr.extend_from_slice(
            &rom.get(prg_end..chr_end).ok_or(NesError::InvalidRomSize)?,
        );
        chr
    } else {
        vec![0; BankSize::Kb8 as usize]
    };

    let prg_ram = if header.prg_ram_count != 0 {
        vec![0; BankSize::Kb8 as usize * header.prg_ram_count as usize]
    } else {
        vec![0; BankSize::Kb8 as usize]
    };

    Ok(Cartridge {
        header,

        prg_rom,
        prg_ram,
        chr,
    })
}

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
}

pub enum BankSize {
    #[allow(dead_code)]
    Kb1 = 0x400,
    #[allow(dead_code)]
    Kb2 = 0x800,
    Kb4 = 0x1000,
    Kb8 = 0x2000,
    Kb16 = 0x4000,
    Kb32 = 0x8000,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InesHeader {
    pub prg_rom_count: u8, // 16 KB units
    pub prg_ram_count: u8, // 8 KB units
    pub chr_rom_count: u8, // 8 KB units
    pub mirroring: Mirroring,
    pub has_battery: bool,
    pub mapper_id: u32,
}

static NES_CONSTANT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

/*
    https://wiki.nesdev.com/w/index.php?title=INES
*/
fn parse_header(rom: &[u8]) -> Result<InesHeader, NesError> {
    if rom[0..=3] != NES_CONSTANT {
        return Err(NesError::InvalidInesFormat);
    }

    let mirroring = if rom[6] & (1 << 3) != 0 {
        Mirroring::FourScreen
    } else if rom[6] & 1 != 0 {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };

    let has_battery = rom[6] & 2 != 0;
    if rom[6] & (1 << 2) != 0 {
        return Err(NesError::TrainerUnsupported);
    }

    let mapper = u32::from((rom[6] >> 4) | (rom[7] & 0xF0));

    if rom[7] & 0xC == 0x8 {
        return Err(NesError::Ines2Unsupported);
    }

    if rom[9] != 0 {
        return Err(NesError::PalUnsupported);
    }

    Ok(InesHeader {
        prg_rom_count: rom[4],
        chr_rom_count: rom[5],
        prg_ram_count: rom[8],
        mirroring,
        has_battery,
        mapper_id: mapper,
    })
}
