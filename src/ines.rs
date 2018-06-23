use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct InesHeader {
    prg_rom_size: u8,
    chr_rom_size: u8,
    flags_6: u8,
    flags_7: u8,
    prg_ram_size: u8,
    flags_9: u8,
}

impl InesHeader {
    pub fn get_mapper(&self) -> u8 {
        (self.flags_6 >> 4) | (self.flags_7 & 0b11110000)
    }

    pub fn get_mirroring(&self) -> u8 {
        self.flags_6 & 0b00000001
    }
}

static NES_CONSTANT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

pub fn parse_header(f: &mut File) -> InesHeader {
    let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
    let bytes = _bytes.expect("error while reading the file");
    if bytes.len() < 16 {
        panic!("invalid file");
    }
    
    for i in 0..3 {
        if bytes[i] != NES_CONSTANT[i] {
            panic!("not a NES file")
        }
    }

    for i in 10..15 {
        if bytes[i] != 0 {
            panic!("different .nes format generation")
        }
    }

    InesHeader {
        prg_rom_size: bytes[4],
        chr_rom_size: bytes[5],
        flags_6: bytes[6],
        flags_7: bytes[7],
        prg_ram_size: bytes[8],
        flags_9: bytes[9],
    }
}
