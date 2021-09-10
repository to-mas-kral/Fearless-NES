use serde::{Deserialize, Serialize};

use crate::cartridge::{BankSize, Cartridge};

#[derive(Serialize, Deserialize)]
pub struct _0Nrom {
    prg_1: usize,
}

impl _0Nrom {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            prg_1: Cartridge::map_bank(cartridge.header.prg_rom_count - 1, BankSize::Kb16),
        }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize, open_bus: u8) -> u8 {
        match addr {
            0x6000..=0x7FFF => cartridge.read_prg_ram(addr - 0x6000),
            0x8000..=0xBFFF => cartridge.read_prg_rom(addr - 0x8000),
            0xC000..=0xFFFF => cartridge.read_prg_rom(self.prg_1 + addr - 0xC000),
            _ => open_bus,
        }
    }

    pub fn cpu_write(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        if let 0x6000..=0x7FFF = addr {
            cartridge.write_prg_ram(addr - 0x6000, val);
        }
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        if cartridge.header.chr_rom_count == 0 {
            cartridge.write_chr(addr, val);
        }
    }

    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        cartridge.read_chr(addr)
    }
}
