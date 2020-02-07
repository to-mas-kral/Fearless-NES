use super::super::cartridge::Cartridge;
use super::super::cartridge::Mirroring;
use super::Mapper;

pub struct Nrom {
    pub prg_2: usize,
    pub nt_ram: [u8; 0x1000],
    pub cartridge: Cartridge,
}

impl Nrom {
    pub fn new(cartridge: Cartridge) -> Nrom {
        let prg_count = cartridge.header.prg_rom_size as usize;
        let prg_2 = match prg_count {
            1 => 0,
            2 => 0x4000,
            _ => panic!("NROM shouldn't have more than 2 PRG banks"),
        };

        Nrom {
            prg_2,
            nt_ram: [0; 0x1000],
            cartridge,
        }
    }
}

impl Mapper for Nrom {
    fn cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.cartridge.prg_ram[addr - 0x6000],
            0x8000..=0xBFFF => self.cartridge.prg_rom[addr - 0x8000],
            0xC000..=0xFFFF => self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)],
            _ => 0,
        }
    }

    fn cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => Some(self.cartridge.prg_ram[addr - 0x6000]),
            0x8000..=0xBFFF => Some(self.cartridge.prg_rom[addr - 0x8000]),
            0xC000..=0xFFFF => Some(self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)]),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x6000..=0x7FFF = addr {
            self.cartridge.prg_ram[addr - 0x6000] = val
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
