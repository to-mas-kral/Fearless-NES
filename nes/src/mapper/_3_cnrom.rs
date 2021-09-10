use serde::{Deserialize, Serialize};

use crate::cartridge::{BankSize, Cartridge};

#[derive(Serialize, Deserialize)]
pub struct _3Cnrom {
    prg_1: usize,

    chr_0: usize,
}

impl _3Cnrom {
    pub fn new(cartridge: &Cartridge) -> Self {
        let prg_count = cartridge.header.prg_rom_count as usize;

        let prg_1 = match prg_count {
            1 => Cartridge::map_bank(0, BankSize::Kb16),
            2 => Cartridge::map_bank(1, BankSize::Kb16),
            // TODO: should return an error here
            _ => panic!("Cnrom shouldn't have more than 2 PRG banks"),
        };

        Self { prg_1, chr_0: 0 }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize, open_bus: u8) -> u8 {
        match addr {
            0x8000..=0xBFFF => cartridge.read_prg_rom(addr - 0x8000),
            0xC000..=0xFFFF => cartridge.read_prg_rom(self.prg_1 + addr - 0xC000),
            _ => open_bus,
        }
    }

    pub fn cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.chr_0 = Cartridge::map_bank(val & 3, BankSize::Kb8);
        }
    }

    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        cartridge.read_chr(self.chr_0 + addr)
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        if cartridge.header.chr_rom_count == 0 {
            cartridge.write_chr(self.chr_0 + addr, val)
        }
    }
}
