use serde::{Deserialize, Serialize};

use crate::{
    cartridge::{BankSize, Cartridge},
    ppu::Mirroring,
};

#[derive(Serialize, Deserialize)]
pub struct _7Axrom {
    mirroring: Mirroring,

    prg_0: usize,
}

impl _7Axrom {
    pub fn new(_cartridge: &Cartridge) -> Self {
        Self {
            mirroring: Mirroring::SingleScreenLow,

            prg_0: 0,
        }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => Some(cartridge.read_prg_rom(self.prg_0 + addr - 0x8000)),
            _ => None,
        }
    }

    pub fn cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.prg_0 = Cartridge::map_bank(val & 7, BankSize::Kb32);

            self.mirroring = if val & 0x10 != 0 {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            }
        }
    }

    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        cartridge.read_chr(addr)
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        if cartridge.has_chr_ram() {
            cartridge.write_chr(addr, val);
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
