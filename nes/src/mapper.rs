use serde::{Deserialize, Serialize};

use super::{cartridge::Cartridge, ppu::Mirroring, NesError};

mod _0_nrom;
mod _1_mmc1;
mod _2_uxrom;
mod _3_cnrom;
mod _4_mmc3;
mod _7_axrom;

use _0_nrom::_0Nrom;
use _1_mmc1::_1Mmc1;
use _2_uxrom::_2Uxrom;
use _3_cnrom::_3Cnrom;
use _4_mmc3::_4Mmc3;
use _7_axrom::_7Axrom;

#[derive(Serialize, Deserialize)]
pub struct BaseMapper {
    nt_ram: Vec<u8>,
    pub cartridge: Cartridge,

    chip: MapperChip,
}

impl BaseMapper {
    pub fn new(cartridge: Cartridge) -> Result<Self, NesError> {
        let chip = match cartridge.header.mapper_id {
            0 => MapperChip::_0Nrom(_0Nrom::new(&cartridge)),
            1 => MapperChip::_1Mmc1(_1Mmc1::new(&cartridge)),
            2 => MapperChip::_2Uxrom(_2Uxrom::new(&cartridge)),
            3 => MapperChip::_3Cnrom(_3Cnrom::new(&cartridge)),
            4 => MapperChip::_4Mmc3(_4Mmc3::new(&cartridge)),
            7 => MapperChip::_7Axrom(_7Axrom::new(&cartridge)),
            mapper_id => return Err(NesError::UnSupportedMapper(mapper_id)),
        };

        Ok(BaseMapper {
            nt_ram: vec![0; 0x1000],
            cartridge,
            chip,
        })
    }

    #[inline]
    pub fn cpu_read(&self, addr: usize, open_bus: u8) -> u8 {
        match &self.chip {
            MapperChip::_0Nrom(nrom) => nrom.cpu_read(&self.cartridge, addr, open_bus),
            MapperChip::_1Mmc1(mmc1) => mmc1.cpu_read(&self.cartridge, addr, open_bus),
            MapperChip::_2Uxrom(uxrom) => uxrom.cpu_read(&self.cartridge, addr, open_bus),
            MapperChip::_3Cnrom(cnrom) => cnrom.cpu_read(&self.cartridge, addr, open_bus),
            MapperChip::_4Mmc3(mmc3) => mmc3.cpu_read(&self.cartridge, addr, open_bus),
            MapperChip::_7Axrom(axrom) => axrom.cpu_read(&self.cartridge, addr, open_bus),
        }
    }

    #[inline]
    pub fn cpu_write(&mut self, addr: usize, val: u8, cpu_cycle: u64, cpu_irq: &mut bool) {
        match &mut self.chip {
            MapperChip::_0Nrom(nrom) => nrom.cpu_write(&mut self.cartridge, addr, val),
            MapperChip::_1Mmc1(mmc1) => mmc1.cpu_write(&mut self.cartridge, addr, val, cpu_cycle),
            MapperChip::_2Uxrom(uxrom) => uxrom.cpu_write(addr, val),
            MapperChip::_3Cnrom(cnrom) => cnrom.cpu_write(addr, val),
            MapperChip::_4Mmc3(mmc3) => mmc3.cpu_write(&mut self.cartridge, addr, val, cpu_irq),
            MapperChip::_7Axrom(axrom) => axrom.cpu_write(addr, val),
        }
    }

    #[inline]
    pub fn read_chr(&self, addr: usize) -> u8 {
        match &self.chip {
            MapperChip::_0Nrom(nrom) => nrom.read_chr(&self.cartridge, addr),
            MapperChip::_1Mmc1(mmc1) => mmc1.read_chr(&self.cartridge, addr),
            MapperChip::_2Uxrom(uxrom) => uxrom.read_chr(&self.cartridge, addr),
            MapperChip::_3Cnrom(cnrom) => cnrom.read_chr(&self.cartridge, addr),
            MapperChip::_4Mmc3(mmc3) => mmc3.read_chr(&self.cartridge, addr),
            MapperChip::_7Axrom(axrom) => axrom.read_chr(&self.cartridge, addr),
        }
    }

    #[inline]
    pub fn write_chr(&mut self, addr: usize, val: u8) {
        match &mut self.chip {
            MapperChip::_0Nrom(nrom) => nrom.write_chr(&mut self.cartridge, addr, val),
            MapperChip::_1Mmc1(mmc1) => mmc1.write_chr(&mut self.cartridge, addr, val),
            MapperChip::_2Uxrom(uxrom) => uxrom.write_chr(&mut self.cartridge, addr, val),
            MapperChip::_3Cnrom(cnrom) => cnrom.write_chr(&mut self.cartridge, addr, val),
            MapperChip::_4Mmc3(mmc3) => mmc3.write_chr(&mut self.cartridge, addr, val),
            MapperChip::_7Axrom(axrom) => axrom.write_chr(&mut self.cartridge, addr, val),
        }
    }

    #[inline]
    pub fn read_nametable(&self, addr: usize) -> u8 {
        self.nt_ram[addr]
    }

    #[inline]
    pub fn write_nametable(&mut self, addr: usize, val: u8) {
        self.nt_ram[addr] = val;
    }

    #[inline]
    pub fn mirroring(&self) -> Mirroring {
        match &self.chip {
            MapperChip::_0Nrom(_) | MapperChip::_2Uxrom(_) | MapperChip::_3Cnrom(_) => {
                self.cartridge.header.mirroring
            }
            MapperChip::_1Mmc1(mmc1) => mmc1.mirroring(),
            MapperChip::_4Mmc3(mmc3) => mmc3.mirroring(),
            MapperChip::_7Axrom(axrom) => axrom.mirroring(),
        }
    }

    #[inline]
    pub fn notify_a12(&mut self, addr: usize, ppu_cycle: u32, cpu_irq: &mut bool) {
        let a12 = (addr & 0x1000) != 0;

        match &mut self.chip {
            MapperChip::_0Nrom(_)
            | MapperChip::_1Mmc1(_)
            | MapperChip::_2Uxrom(_)
            | MapperChip::_3Cnrom(_)
            | MapperChip::_7Axrom(_) => (),
            MapperChip::_4Mmc3(mmc3) => mmc3.notify_a12(a12, ppu_cycle, cpu_irq),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum MapperChip {
    _0Nrom(_0Nrom),
    _1Mmc1(_1Mmc1),
    _2Uxrom(_2Uxrom),
    _3Cnrom(_3Cnrom),
    _4Mmc3(_4Mmc3),
    _7Axrom(_7Axrom),
}
