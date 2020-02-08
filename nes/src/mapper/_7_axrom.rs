use super::super::cartridge::Cartridge;
use super::super::cartridge::Mirroring;
use super::super::Nes;
use super::Mapper;

impl Nes {
    pub fn _7_axrom_initialize(cartridge: Cartridge) -> Mapper {
        let mut mapper = Mapper::new(cartridge);

        mapper.prg_1 = 0;
        mapper.nt_ram = vec![0; 0x1000];
        mapper.mirroring = Mirroring::SingleScreenLow;

        mapper.cpu_read = Nes::_7_axrom_cpu_read;
        mapper.cpu_peek = Nes::_7_axrom_cpu_peek;
        mapper.cpu_write = Nes::_7_axrom_cpu_write;
        mapper.read_chr = Nes::_7_axrom_read_chr;
        mapper.write_chr = Nes::_7_axrom_write_chr;
        mapper.read_nametable = Nes::_7_axrom_read_nametable;
        mapper.write_nametable = Nes::_7_axrom_write_nametable;

        mapper
    }

    pub fn _7_axrom_cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                self.mapper.cartridge.prg_rom[self.mapper.prg_1 + (addr - 0x8000)]
            }
            _ => 0,
        }
    }

    pub fn _7_axrom_cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => {
                Some(self.mapper.cartridge.prg_rom[self.mapper.prg_1 + (addr - 0x8000)])
            }
            _ => None,
        }
    }

    pub fn _7_axrom_cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.mapper.prg_1 = 0x8000 * (val & 7) as usize;

            self.mapper.mirroring = if val & 0x10 != 0 {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            }
        }
    }

    pub fn _7_axrom_read_chr(&mut self, addr: usize) -> u8 {
        self.mapper.cartridge.chr[addr]
    }

    pub fn _7_axrom_write_chr(&mut self, addr: usize, val: u8) {
        if self.mapper.cartridge.header.chr_rom_size == 0 {
            self.mapper.cartridge.chr[addr] = val;
        }
    }

    pub fn _7_axrom_read_nametable(&mut self, addr: usize) -> u8 {
        self.mapper.nt_ram[addr]
    }

    pub fn _7_axrom_write_nametable(&mut self, addr: usize, val: u8) {
        self.mapper.nt_ram[addr] = val;
    }
}
