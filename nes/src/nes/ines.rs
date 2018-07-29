use std::fs::File;
use std::io::Read;

use super::NesError;

//An iNES file consists of the following sections, in order:

//Header (16 bytes)
//Trainer, if present (0 or 512 bytes)
//PRG ROM data (16384 * x bytes)
//CHR ROM data, if present (8192 * y bytes)
//PlayChoice INST-ROM, if present (0 or 8192 bytes)
//PlayChoice PROM, if present (16 bytes Data, 16 bytes CounterOut) (this is often missing, see PC10 ROM-Images for details)
//Some ROM-Images additionally contain a 128-byte (or sometimes 127-byte) title at the end of the file.

//The format of the header is as follows:

//0-3: Constant $4E $45 $53 $1A ("NES" followed by MS-DOS end-of-file)
//4: Size of PRG ROM in 16 KB units
//5: Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM)
//6: Flags 6
//7: Flags 7
//8: Size of PRG RAM in 8 KB units (Value 0 infers 8 KB for compatibility; see PRG RAM circuit)
//9: Flags 9
//10: Flags 10 (unofficial)
//11-15: Zero filled

//Flags 6
//76543210
//||||||||
//|||||||+- Mirroring: 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
//|||||||              1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
//||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
//|||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
//||||+---- 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
//++++----- Lower nybble of mapper number

//Flags 7
//76543210
//||||||||
//|||||||+- VS Unisystems
//||||||+-- PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
//||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
//++++----- Upper nybble of mapper number

//Flags 9
//76543210
//||||||||
//|||||||+- TV system (0: NTSC; 1: PAL)
//+++++++-- Reserved, set to zero

//TODO: have some Cartridge struct that holds the data henceforth
//TODO: support iNES 2.0

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

pub fn parse_header(f: &mut File) -> Result<InesHeader, NesError> {
    let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
    let header = _bytes?;
    if header.len() < 16 {
        return Err(NesError::InvalidFile);
    }

    for i in 0..3 {
        if header[i] != NES_CONSTANT[i] {
            return Err(NesError::NesConstantMissing);
        }
    }

    for i in 10..15 {
        if header[i] != 0 {
            //TODO: handle iNES 2.0
        }
    }

    let mirroring = if header[6] & (1 << 3) != 0 {
        Mirroring::FourScreen
    } else if header[6] & 1 != 0 {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };

    let has_prg_ram = header[6] & 2 != 0;
    let has_trainer = header[6] & (1 << 2) != 0;

    let mapper = ((header[6] >> 4) | (header[7] & 0xF0)) as u32;

    Ok(InesHeader {
        prg_rom_size: header[4],
        chr_rom_size: header[5],
        prg_ram_size: header[8],
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
