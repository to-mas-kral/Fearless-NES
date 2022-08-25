use bincode::{Decode, Encode};

use crate::cartridge::{BankSize, Cartridge};

#[derive(Decode, Encode)]
pub struct _3Cnrom {
    prg_1: usize,

    chr_0: usize,
}

impl _3Cnrom {
    pub fn new(cartridge: &Cartridge) -> Self {
        let prg_count = cartridge.prg_rom_count(BankSize::Kb16);

        let prg_1 = match prg_count {
            1 => Cartridge::map_bank(0, BankSize::Kb16),
            // Simply ignore the other banks
            _ => Cartridge::map_bank(1, BankSize::Kb16),
        };

        Self { prg_1, chr_0: 0 }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize) -> Option<u8> {
        match addr {
            0x8000..=0xBFFF => Some(cartridge.read_prg_rom(addr - 0x8000)),
            0xC000..=0xFFFF => Some(cartridge.read_prg_rom(self.prg_1 + addr - 0xC000)),
            _ => None,
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
        if cartridge.has_chr_ram() {
            cartridge.write_chr(self.chr_0 + addr, val)
        }
    }
}
