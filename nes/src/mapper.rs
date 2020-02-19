use serde::{Deserialize, Serialize};

use super::{cartridge::Cartridge, ppu::Mirroring, Nes, NesError};

mod _0_nrom;
mod _1_mmc1;
mod _2_uxrom;
mod _3_cnrom;
mod _7_axrom;

impl Nes {
    pub(crate) fn initialize_mapper(cartridge: Cartridge) -> Result<Mapper, NesError> {
        match cartridge.header.mapper_id {
            0 => Ok(Nes::_0_nrom_initialize(cartridge)),
            1 => Ok(Nes::_1_mmc1_initialize(cartridge)),
            2 => Ok(Nes::_2_uxrom_initialize(cartridge)),
            3 => Ok(Nes::_3_cnrom_initialize(cartridge)),
            7 => Ok(Nes::_7_axrom_initialize(cartridge)),
            mapper_id => Err(NesError::UnSupportedMapper(mapper_id)),
        }
    }

    pub(crate) fn reaload_mapper_pointers(&mut self) {
        match self.mapper.cartridge.header.mapper_id {
            0 => Nes::_0_nrom_reload(self),
            1 => Nes::_1_mmc1_reload(self),
            3 => Nes::_3_cnrom_reload(self),
            2 => Nes::_2_uxrom_reload(self),
            7 => Nes::_7_axrom_reload(self),
            mapper_id => panic!("Loaded unsupported mapper {} from save file", mapper_id),
        }
    }
}

/*
    Using functions rather than Trait objects provides more versatility.
    It's also necessary to implement more complicated mappers such as
    MMC3 / MMC5 which require access to nes state.
*/

#[derive(Serialize, Deserialize)]
pub struct Mapper {
    // Function pointers
    #[serde(skip)]
    pub cpu_read: MapperRead,
    #[serde(skip)]
    pub cpu_peek: MapperRead,
    #[serde(skip)]
    pub cpu_write: MapperWrite,
    #[serde(skip)]
    pub read_chr: MapperRead,
    #[serde(skip)]
    pub write_chr: MapperWrite,
    #[serde(skip)]
    pub read_nametable: MapperRead,
    #[serde(skip)]
    pub write_nametable: MapperWrite,

    pub nt_ram: Vec<u8>,

    pub mirroring: Mirroring,
    pub cartridge: Cartridge,

    prg_rom_count: u8,
    chr_count: u8,

    prg_rom_indices: Vec<u8>,
    chr_indices: Vec<u8>,

    // State for specific mappers
    // 0 - NROM
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
            cpu_read: MapperRead::default(),
            cpu_peek: MapperRead::default(),
            cpu_write: MapperWrite::default(),
            read_chr: MapperRead::default(),
            write_chr: MapperWrite::default(),
            read_nametable: MapperRead::default(),
            write_nametable: MapperWrite::default(),

            nt_ram: vec![0; 0x1000],

            mirroring: cartridge.header.mirroring,

            prg_rom_count: cartridge.header.prg_rom_count,
            chr_count: cartridge.header.chr_rom_count,

            // Must be initialized for specific mappers
            prg_rom_indices: vec![],
            chr_indices: vec![],

            cartridge,

            // State for specific mappers, this is set in initialze functions
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

// Structs have to be used here for implementing Default, which is used for Serde

pub struct MapperRead {
    pub ptr: fn(_: &mut Nes, _: usize) -> u8,
}

pub struct MapperWrite {
    pub ptr: fn(_: &mut Nes, _: usize, _: u8),
}

impl Default for MapperRead {
    fn default() -> Self {
        MapperRead {
            ptr: |_: &mut Nes, _: usize| {
                panic!("Mapper function pointers haven't been initialised")
            },
        }
    }
}

impl Default for MapperWrite {
    fn default() -> Self {
        MapperWrite {
            ptr: |_: &mut Nes, _: usize, _: u8| {
                panic!("Mapper function pointers haven't been initialised")
            },
        }
    }
}
