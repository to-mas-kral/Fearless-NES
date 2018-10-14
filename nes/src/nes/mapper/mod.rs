use super::cartridge::Cartridge;
use super::cartridge::Mirroring;
use super::NesError;

mod _0_nrom;
mod _2_uxrom;
mod _3_cnrom;

pub trait Mapper {
    fn cpu_read(&mut self, addr: usize) -> Option<u8>;
    fn cpu_read_direct(&mut self, addr: usize) -> u8;
    fn cpu_write(&mut self, addr: usize, val: u8);
    fn read_chr(&mut self, addr: usize) -> u8;
    fn write_chr(&mut self, addr: usize, val: u8);
    fn read_nametable(&mut self, addr: usize) -> u8;
    fn write_nametable(&mut self, addr: usize, val: u8);
    fn mirroring(&self) -> Mirroring;
}

pub fn create_mapper(cartridge: Cartridge) -> Result<Box<Mapper>, NesError> {
    match cartridge.header.mapper {
        0 => Ok(Box::new(_0_nrom::Nrom::new(cartridge))),
        2 => Ok(Box::new(_2_uxrom::Uxrom::new(cartridge))),
        3 => Ok(Box::new(_3_cnrom::Cnrom::new(cartridge))),
        _ => {
            println!("mapper number {} is unsupported", cartridge.header.mapper);
            Err(NesError::UnsupportedMapper)
        }
    }
}
