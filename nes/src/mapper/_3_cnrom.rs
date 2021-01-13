use super::super::cartridge::Cartridge;
use super::super::Nes;
use super::Mapper;

impl Nes {
    pub(crate) fn _3_cnrom_initialize(cartridge: Cartridge) -> Mapper {
        let mut mapper = Mapper::new(cartridge);

        let prg_count = mapper.cartridge.header.prg_rom_count as usize;
        let prg_2 = match prg_count {
            1 => 0,
            2 => 0x4000,
            _ => panic!("Cnrom shouldn't have more than 2 PRG banks"),
        };

        mapper.prg_2 = prg_2;
        mapper.chr_1 = 0;
        mapper.nt_ram = vec![0; 0x1000];
        mapper.mirroring = mapper.cartridge.header.mirroring;

        mapper.cpu_read.ptr = Nes::_3_cnrom_cpu_read;
        mapper.cpu_peek.ptr = Nes::_3_cnrom_cpu_peek;
        mapper.cpu_write.ptr = Nes::_3_cnrom_cpu_write;
        mapper.read_chr.ptr = Nes::_3_cnrom_read_chr;
        mapper.write_chr.ptr = Nes::_3_cnrom_write_chr;
        mapper.read_nametable.ptr = Nes::_3_cnrom_read_nametable;
        mapper.write_nametable.ptr = Nes::_3_cnrom_write_nametable;

        mapper
    }

    pub(crate) fn _3_cnrom_reload(&mut self) {
        self.mapper.cpu_read.ptr = Nes::_3_cnrom_cpu_read;
        self.mapper.cpu_peek.ptr = Nes::_3_cnrom_cpu_peek;
        self.mapper.cpu_write.ptr = Nes::_3_cnrom_cpu_write;
        self.mapper.read_chr.ptr = Nes::_3_cnrom_read_chr;
        self.mapper.write_chr.ptr = Nes::_3_cnrom_write_chr;
        self.mapper.read_nametable.ptr = Nes::_3_cnrom_read_nametable;
        self.mapper.write_nametable.ptr = Nes::_3_cnrom_write_nametable;
    }

    pub(crate) fn _3_cnrom_cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => self.mapper.cartridge.prg_rom[addr - 0x8000],
            0xC000..=0xFFFF => {
                self.mapper.cartridge.prg_rom[self.mapper.prg_2 + (addr - 0xC000)]
            }
            _ => 0,
        }
    }

    pub(crate) fn _3_cnrom_cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => self.mapper.cartridge.prg_rom[addr - 0x8000],
            0xC000..=0xFFFF => {
                self.mapper.cartridge.prg_rom[self.mapper.prg_2 + (addr - 0xC000)]
            }
            _ => self.cpu.open_bus,
        }
    }

    pub(crate) fn _3_cnrom_cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.mapper.chr_1 = 0x2000 * ((val & 3) as usize);
        }
    }

    pub(crate) fn _3_cnrom_read_chr(&mut self, addr: usize) -> u8 {
        self.mapper.cartridge.chr[self.mapper.chr_1 + addr]
    }

    pub(crate) fn _3_cnrom_write_chr(&mut self, addr: usize, val: u8) {
        if self.mapper.cartridge.header.chr_rom_count == 0 {
            self.mapper.cartridge.chr[self.mapper.chr_1 + addr] = val;
        }
    }

    pub(crate) fn _3_cnrom_read_nametable(&mut self, addr: usize) -> u8 {
        self.mapper.nt_ram[addr]
    }

    pub(crate) fn _3_cnrom_write_nametable(&mut self, addr: usize, val: u8) {
        self.mapper.nt_ram[addr] = val;
    }
}
