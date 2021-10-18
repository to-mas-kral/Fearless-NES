use std::{convert::TryFrom, fmt::Display};

use crate::{ppu::Mirroring, NesError};

use serde::{Deserialize, Serialize};

mod gamedb;

const HEADER_SIZE: usize = 16;

#[derive(Serialize, Deserialize)]
pub struct Cartridge {
    pub header: Header,
    prg_rom: Vec<u8>,
    prg_wram: Option<Vec<u8>>,
    chr: Vec<u8>,
}

impl Cartridge {
    /// First parse the iNES header, then try to find information in the NEs 2.0 XML Game Database
    pub fn from_rom(rom: &[u8]) -> Result<Cartridge, NesError> {
        if rom.len() < HEADER_SIZE {
            return Err(NesError::InvalidInesFormat);
        }
        let (header, rom) = rom.split_at(HEADER_SIZE);

        let header = Header::from_ines(header)?;

        let prg_end = header.prg_rom_size as usize;
        let prg_portion = rom.get(0..prg_end).ok_or(NesError::RomCorrupted)?;

        let chr_portion = if let Some(chr_rom_size) = header.chr_rom_size {
            let chr_end = prg_end + chr_rom_size as usize;
            Some(rom.get(prg_end..chr_end).ok_or(NesError::RomCorrupted)?)
        } else {
            None
        };

        let header = Header::from_prg_chr(prg_portion, chr_portion)?.unwrap_or(header);

        let prg_rom = Vec::from(rom.get(0..prg_end).ok_or(NesError::RomCorrupted)?);
        let prg_wram = match (header.prg_ram_size, header.prg_nvram_size) {
            (Some(size), None) | (None, Some(size)) => Some(vec![0; size as usize]),
            (None, None) => None,
            (Some(_), Some(_)) => return Err(NesError::ChrRomAndRamUnsupported),
        };

        // TODO: handle games with both CHR RAM and ROM (also fix has_chr_ram()...)
        let chr = match (header.chr_ram_size, header.chr_rom_size) {
            // Unwrap should be safe because header.chr_rom_size is Some(_)...
            (None, Some(_)) => Vec::from(chr_portion.unwrap()),
            (Some(size), None) => vec![0; size as usize],
            (Some(_), Some(_)) => return Err(NesError::ChrRomAndRamUnsupported),
            (None, None) => return Err(NesError::RomCorrupted),
        };

        if header.console_typ != ConsoleType::Standard {
            return Err(NesError::ConsoleUnsupported(header.console_typ));
        };

        if header.region != Region::Ntsc && header.region != Region::Multi {
            return Err(NesError::RegionUnsupported(header.region));
        };

        Ok(Cartridge {
            header,

            prg_rom,
            prg_wram,
            chr,
        })
    }

    /// Banks are indexed from 0
    pub fn map_bank(bank: u8, bank_size: BankSize) -> usize {
        bank as usize * bank_size as usize
    }

    #[inline]
    pub fn read_prg_rom(&self, addr: usize) -> u8 {
        self.prg_rom[addr]
    }

    #[inline]
    pub fn read_prg_ram(&self, addr: usize) -> Option<u8> {
        self.prg_wram.as_ref().map(|prg_ram| prg_ram[addr])
    }

    #[inline]
    pub fn write_prg_ram(&mut self, addr: usize, val: u8) {
        if let Some(ref mut prg_ram) = self.prg_wram {
            prg_ram[addr] = val
        }
    }

    #[inline]
    pub fn read_chr(&self, addr: usize) -> u8 {
        self.chr[addr]
    }

    #[inline]
    pub fn write_chr(&mut self, addr: usize, val: u8) {
        self.chr[addr] = val
    }

    // FIXME: should probably ceil() these "count" calculations...
    #[inline]
    pub fn prg_rom_count(&self, unit: BankSize) -> u32 {
        self.header.prg_rom_size / unit as u32
    }

    #[inline]
    pub fn prg_ram_count(&self, unit: BankSize) -> Option<u32> {
        self.header.prg_ram_size.map(|size| size / unit as u32)
    }

    #[inline]
    pub fn prg_nvram_count(&self, unit: BankSize) -> Option<u32> {
        self.header.prg_nvram_size.map(|size| size / unit as u32)
    }

    #[inline]
    pub fn chr_rom_count(&self, unit: BankSize) -> Option<u32> {
        self.header.chr_rom_size.map(|size| size / unit as u32)
    }

    #[inline]
    pub fn chr_ram_count(&self, unit: BankSize) -> Option<u32> {
        self.header.chr_ram_size.map(|size| size / unit as u32)
    }

    #[inline]
    pub fn has_chr_ram(&self) -> bool {
        self.header.chr_ram_size.is_some()
    }
}

pub enum BankSize {
    Kb1 = 0x400,
    #[allow(dead_code)]
    Kb2 = 0x800,
    Kb4 = 0x1000,
    Kb8 = 0x2000,
    Kb16 = 0x4000,
    Kb32 = 0x8000,
}

/** https://wiki.nesdev.org/w/index.php?title=NES_2.0 **/
#[derive(Serialize, Deserialize)]
pub struct Header {
    /// Whether this information comes from iNES 1/2 or the 2.0 Game Database
    pub source: HeaderSource,

    pub name: String,

    pub prg_rom_size: u32,
    pub chr_rom_size: Option<u32>,

    // TODO(low): Some unlicensed and Playchoice games have "miscrom" field, but we probably don't care
    // TODO(low): Some unlicensed games have "chrnvram" field, but we probably don't care
    pub chr_ram_size: Option<u32>,
    pub prg_ram_size: Option<u32>,
    pub prg_nvram_size: Option<u32>,

    pub mapper: u32,
    pub submapper: u32,
    pub mirroring: Mirroring,
    /// "Battery" means that a cartridge has either a battery or non-volatile RAM
    pub battery: bool,

    pub console_typ: ConsoleType,
    pub region: Region,

    // TODO(low): https://wiki.nesdev.org/w/index.php?title=NES_2.0#Default_Expansion_Device
    pub expansion: u32,
    // TODO(low): 10 mostly homebrew or compatibility hacks games have a trainer
}

const NES_CONSTANT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

/** https://wiki.nesdev.org/w/index.php?title=INES **/
impl Header {
    pub fn from_ines(ines: &[u8]) -> Result<Self, NesError> {
        if ines[0..=3] != NES_CONSTANT {
            return Err(NesError::InvalidInesFormat);
        }

        let mirroring = if ines[6] & (1 << 3) != 0 {
            Mirroring::FourScreen
        } else if ines[6] & 1 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let battery = ines[6] & 2 != 0;
        if ines[6] & (1 << 2) != 0 {
            return Err(NesError::TrainerUnsupported);
        }

        let mapper = u32::from((ines[6] >> 4) | (ines[7] & 0xF0));

        if ines[7] & 0xC == 0x8 {
            return Err(NesError::Ines2Unsupported);
        }

        let (chr_rom_size, chr_ram_size) = match ines[5] {
            0 => ((None, Some(1 * BankSize::Kb8 as u32))),
            cnt => ((Some(cnt as u32 * BankSize::Kb8 as u32), None)),
        };

        let prg_ram_size = match ines[8] {
            0 => Some(1 * BankSize::Kb8 as u32),
            cnt => (Some(cnt as u32 * BankSize::Kb8 as u32)),
        };

        let console_typ = match ines[7] & 3 {
            0 => ConsoleType::Standard,
            1 => ConsoleType::VsSystem,
            2 => ConsoleType::Playchoice,
            3 => return Err(NesError::InvalidInesFormat),
            _ => unreachable!(),
        };

        Ok(Header {
            mirroring,
            source: HeaderSource::Ines1,
            name: String::from(""),
            prg_rom_size: ines[4] as u32 * BankSize::Kb16 as u32,
            chr_rom_size,
            chr_ram_size,
            prg_ram_size,
            prg_nvram_size: None,
            mapper,
            submapper: 0,
            battery,
            console_typ,
            region: Region::try_from(ines[9])?,
            expansion: 1,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub enum HeaderSource {
    Ines1,
    Ines2,
    GameDb,
}

impl Display for HeaderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderSource::Ines1 => write!(f, "iNES 1. header"),
            HeaderSource::Ines2 => write!(f, "iNES 2. header"),
            HeaderSource::GameDb => write!(f, "NES 2.0 XML Database"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ConsoleType {
    /// NES / Famicom
    Standard = 0,
    VsSystem = 1,
    Playchoice = 2,
    Extended = 3,
}

impl Display for ConsoleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleType::Standard => write!(f, "NES / Famicom"),
            ConsoleType::VsSystem => write!(f, "Vs. System"),
            ConsoleType::Playchoice => write!(f, "Playchoice 10"),
            ConsoleType::Extended => write!(f, "Extended console type..."),
        }
    }
}

impl TryFrom<&str> for ConsoleType {
    type Error = NesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "0" => Ok(ConsoleType::Standard),
            "1" => Ok(ConsoleType::VsSystem),
            "2" => Ok(ConsoleType::Playchoice),
            "3" => Ok(ConsoleType::Extended),
            _ => Err(NesError::GameDbFormat),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Region {
    Ntsc = 0,
    Pal = 1,
    Multi = 2,
    Dendy = 3,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::Ntsc => write!(f, "NTSC"),
            Region::Pal => write!(f, "PAL"),
            Region::Multi => write!(f, "Multi-region"),
            Region::Dendy => write!(f, "Dendy"),
        }
    }
}

impl TryFrom<&str> for Region {
    type Error = NesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "0" => Ok(Region::Ntsc),
            "1" => Ok(Region::Pal),
            "2" => Ok(Region::Multi),
            "3" => Ok(Region::Dendy),
            _ => Err(NesError::GameDbFormat),
        }
    }
}

impl TryFrom<u8> for Region {
    type Error = NesError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Region::Ntsc),
            1 => Ok(Region::Pal),
            _ => Err(NesError::InvalidInesFormat),
        }
    }
}

impl TryFrom<&str> for Mirroring {
    type Error = NesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "V" => Ok(Mirroring::Vertical),
            "H" => Ok(Mirroring::Horizontal),
            "4" => Ok(Mirroring::FourScreen),
            _ => Err(NesError::GameDbFormat),
        }
    }
}
