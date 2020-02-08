use super::cartridge::Cartridge;
use super::cartridge::Mirroring;
use super::Nes;

mod _0_nrom;
mod _1_mmc1;
//mod _2_uxrom;
//mod _3_cnrom;
//mod _7_axrom;

impl Nes {
    pub fn initialize_mapper(cartridge: Cartridge) -> Mapper {
        match cartridge.header.mapper {
            0 => Nes::_0_nrom_initialize(cartridge),
            1 => Nes::_1_mmc1_initialize(cartridge),
            //2 => Ok(Box::new(_2_uxrom::Uxrom::new(cartridge))),
            //3 => Ok(Box::new(_3_cnrom::Cnrom::new(cartridge))),
            //7 => Ok(Box::new(_7_axrom::Axrom::new(cartridge))),
            _ => {
                panic!("mapper number {} is unsupported", cartridge.header.mapper);
                //Err(NesError::UnsupportedMapper)
            }
        }
    }
}

pub struct Mapper {
    // Function pointers
    pub cpu_read: fn(&mut Nes, usize) -> Option<u8>,
    pub cpu_peek: fn(&mut Nes, usize) -> u8,
    pub cpu_write: fn(&mut Nes, addr: usize, val: u8),
    pub read_chr: fn(&mut Nes, addr: usize) -> u8,
    pub write_chr: fn(&mut Nes, addr: usize, val: u8),
    pub read_nametable: fn(&mut Nes, addr: usize) -> u8,
    pub write_nametable: fn(&mut Nes, addr: usize, val: u8),

    // General mapper state
    pub prg_1: usize,
    pub prg_2: usize,
    pub chr_1: usize,
    pub chr_2: usize,

    pub nt_ram: Vec<u8>,

    pub mirroring: Mirroring,

    pub cartridge: Cartridge,

    // State for specific mappers
    // 0
    // 1 - MMC1
    pub shift: u8,
    prg_mode: u8,
    chr_mode: u8,
    chr_mask: u8,
    enable_ram: bool,
    ignore_write: u64,
    // 2 - UxROM
    // 3 - CnROM
    // 7 - AxROM
}

impl Mapper {
    pub fn new(cartridge: Cartridge) -> Mapper {
        Mapper {
            // Function pointers
            cpu_read: mock_cpu_read,
            cpu_peek: mock_cpu_peek,
            cpu_write: mock_cpu_write,
            read_chr: mock_read_chr,
            write_chr: mock_write_chr,
            read_nametable: mock_read_nametable,
            write_nametable: mock_write_nametable,

            // General mapper state
            prg_1: 0,
            prg_2: 0,
            chr_1: 0,
            chr_2: 0,

            nt_ram: vec![0; 0x1000],

            mirroring: Mirroring::Obscure,

            cartridge,

            // State for specific mappers
            // 0
            // 1 - MMC1
            shift: 0,
            prg_mode: 0,
            chr_mode: 0,
            chr_mask: 0,
            enable_ram: false,
            ignore_write: 0,
            // 2 - UxROM
            // 3 - CnROM
            // 7 - AxROM
        }
    }
}

fn mock_cpu_read(_: &mut Nes, _: usize) -> Option<u8> {
    unimplemented!("Mapper function pointer have not been set")
}
fn mock_cpu_peek(_: &mut Nes, _: usize) -> u8 {
    unimplemented!("Mapper function pointer have not been set")
}
fn mock_cpu_write(_: &mut Nes, _: usize, _: u8) {
    unimplemented!("Mapper function pointer have not been set")
}
fn mock_read_chr(_: &mut Nes, _: usize) -> u8 {
    unimplemented!("Mapper function pointer have not been set")
}
fn mock_write_chr(_: &mut Nes, _: usize, _: u8) {
    unimplemented!("Mapper function pointer have not been set")
}
fn mock_read_nametable(_: &mut Nes, _: usize) -> u8 {
    unimplemented!("Mapper function pointer have not been set")
}
fn mock_write_nametable(_: &mut Nes, _: usize, _: u8) {
    unimplemented!("Mapper function pointer have not been set")
}
