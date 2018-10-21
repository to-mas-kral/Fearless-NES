use super::super::cartridge::Cartridge;
use super::super::cartridge::Mirroring;

use super::Mapper;

pub struct Uxrom {
    pub prg_1: usize,
    pub prg_2: usize,
    pub nt_ram: [u8; 0x1000],
    pub cartridge: Cartridge,
}

impl Uxrom {
    pub fn new(cartridge: Cartridge) -> Uxrom {
        let prg_count = cartridge.header.prg_rom_size as usize;
        let prg_2 = 0x4000 * (prg_count - 1);

        Uxrom {
            prg_1: 0,
            prg_2,
            nt_ram: [0; 0x1000],
            cartridge,
        }
    }
}

impl Mapper for Uxrom {
    fn cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)],
            0xC000..=0xFFFF => self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)],
            _ => 0,
        }
    }

    fn cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x8000..=0xBFFF => Some(self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)]),
            0xC000..=0xFFFF => Some(self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)]),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.prg_1 = 0x4000 * val as usize
        }
    }

    fn read_chr(&mut self, addr: usize) -> u8 {
        self.cartridge.chr[addr]
    }

    fn write_chr(&mut self, addr: usize, val: u8) {
        if self.cartridge.header.chr_rom_size == 0 {
            self.cartridge.chr[addr] = val;
        }
    }

    fn read_nametable(&mut self, addr: usize) -> u8 {
        self.nt_ram[addr]
    }

    fn write_nametable(&mut self, addr: usize, val: u8) {
        self.nt_ram[addr] = val;
    }

    fn mirroring(&self) -> Mirroring {
        self.cartridge.header.mirroring
    }
}
