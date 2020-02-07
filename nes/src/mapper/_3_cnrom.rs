use super::super::cartridge::Cartridge;
use super::super::Nes;
use super::Mapper;

impl Nes {
    pub fn _3_cnrom_initialize(cartridge: Cartridge) -> Mapper {
        let mut mapper = Mapper::new(cartridge);

        let prg_count = mapper.cartridge.header.prg_rom_size as usize;
        let prg_2 = match prg_count {
            1 => 0,
            2 => 0x4000,
            _ => panic!("Cnrom shouldn't have more than 2 PRG banks"),
        };

        mapper.prg_2 = prg_2;
        mapper.chr_1 = 0;
        mapper.nt_ram = vec![0; 0x1000];
        mapper.mirroring = mapper.cartridge.header.mirroring;

        mapper.cpu_read = Nes::_3_cnrom_cpu_read;
        mapper.cpu_peek = Nes::_3_cnrom_cpu_peek;
        mapper.cpu_write = Nes::_3_cnrom_cpu_write;
        mapper.read_chr = Nes::_3_cnrom_read_chr;
        mapper.write_chr = Nes::_3_cnrom_write_chr;
        mapper.read_nametable = Nes::_3_cnrom_read_nametable;
        mapper.write_nametable = Nes::_3_cnrom_write_nametable;

        mapper
    }

    pub fn _3_cnrom_cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => self.mapper.cartridge.prg_rom[addr - 0x8000],
            0xC000..=0xFFFF => {
                self.mapper.cartridge.prg_rom[self.mapper.prg_2 + (addr - 0xC000)]
            }
            _ => 0,
        }
    }

    pub fn _3_cnrom_cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x8000..=0xBFFF => Some(self.mapper.cartridge.prg_rom[addr - 0x8000]),
            0xC000..=0xFFFF => {
                Some(self.mapper.cartridge.prg_rom[self.mapper.prg_2 + (addr - 0xC000)])
            }
            _ => None,
        }
    }

    pub fn _3_cnrom_cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.mapper.chr_1 = 0x2000 * ((val & 3) as usize);
        }
    }

    pub fn _3_cnrom_read_chr(&mut self, addr: usize) -> u8 {
        self.mapper.cartridge.chr[self.mapper.chr_1 + addr]
    }

    pub fn _3_cnrom_write_chr(&mut self, addr: usize, val: u8) {
        if self.mapper.cartridge.header.chr_rom_size == 0 {
            self.mapper.cartridge.chr[self.mapper.chr_1 + addr] = val;
        }
    }

    pub fn _3_cnrom_read_nametable(&mut self, addr: usize) -> u8 {
        self.mapper.nt_ram[addr]
    }

    pub fn _3_cnrom_write_nametable(&mut self, addr: usize, val: u8) {
        self.mapper.nt_ram[addr] = val;
    }
}
