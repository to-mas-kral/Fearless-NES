use std::fs::File;
use std::io::Read;

use super::NesError;

//TODO: FDS, UNIF, NFS file formats

pub struct Cartridge {
    pub header: InesHeader,

    pub rom: Vec<u8>,
    //pub prg_ram: Vec<u8>,
}

pub fn parse_rom(f: &mut File) -> Result<Cartridge, NesError> {
    let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
    let rom = _bytes?;

    let header = parse_header(&rom)?;

    let rom: Vec<u8> = rom.iter().cloned().skip(16).collect();

    Ok(Cartridge {
        header,

        rom,
        //prg_ram: Vec::new(),
    })
}

#[derive(Debug)]
pub struct InesHeader {
    pub prg_rom_size: u8,
    pub prg_ram_size: u8,
    pub chr_rom_size: u8,
    pub mirroring: Mirroring,
    pub has_prg_ram: bool,
    pub has_trainer: bool,
    pub mapper: u32,
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

    let has_prg_ram = rom[6] & 2 != 0;
    let has_trainer = rom[6] & (1 << 2) != 0;
    if has_trainer {
        panic!("ROM has trainer, unsupported");
    }

    let mapper = u32::from((rom[6] >> 4) | (rom[7] & 0xF0));

    if rom[9] != 0 {
        return Err(NesError::PalRom);
    }

    Ok(InesHeader {
        prg_rom_size: rom[4],
        chr_rom_size: rom[5],
        prg_ram_size: rom[8],
        mirroring,
        has_prg_ram,
        has_trainer,
        mapper,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen,
    FourScreen,
    Obscure,
}
