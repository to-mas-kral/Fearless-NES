use super::super::cartridge::Cartridge;
use super::super::cartridge::Mirroring;
use super::Mapper;

pub struct Axrom {
    pub prg_1: usize,
    pub nt_ram: [u8; 0x1000],
    pub mirroring: Mirroring,
    pub cartridge: Cartridge,
}

impl Axrom {
    pub fn new(cartridge: Cartridge) -> Axrom {
        Axrom {
            prg_1: 0,
            nt_ram: [0; 0x1000],
            mirroring: Mirroring::SingleScreenLow,
            cartridge,
        }
    }
}

impl Mapper for Axrom {
    fn cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xFFFF => self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)],
            _ => 0,
        }
    }

    fn cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => Some(self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)]),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.prg_1 = 0x8000 * (val & 7) as usize;

            self.mirroring = if val & 0x10 != 0 {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            }
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
        self.mirroring
    }
}
