use serde::{Deserialize, Serialize};

use crate::cartridge::{BankSize, Cartridge};

#[derive(Serialize, Deserialize)]
pub struct _2Uxrom {
    prg_0: usize,
    prg_1: usize,
}

impl _2Uxrom {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            prg_0: 0,
            prg_1: Cartridge::map_bank(cartridge.header.prg_rom_count - 1, BankSize::Kb16),
        }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize, open_bus: u8) -> u8 {
        match addr {
            0x8000..=0xBFFF => cartridge.read_prg_rom(self.prg_0 + addr - 0x8000),
            0xC000..=0xFFFF => cartridge.read_prg_rom(self.prg_1 + addr - 0xC000),
            _ => open_bus,
        }
    }

    pub fn cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.prg_0 = Cartridge::map_bank(val, BankSize::Kb16);
        }
    }

    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        cartridge.read_chr(addr)
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        if cartridge.header.chr_rom_count == 0 {
            cartridge.write_chr(addr, val);
        }
    }
}
