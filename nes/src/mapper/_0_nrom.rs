use super::{
    super::{
        cartridge::{BankSize, Cartridge},
        Nes,
    },
    Mapper,
};

impl Nes {
    pub(crate) fn _0_nrom_initialize(cartridge: Cartridge) -> Mapper {
        let mut mapper = Mapper::new(cartridge);

        mapper.cpu_read.ptr = Nes::_0_nrom_cpu_read;
        mapper.cpu_peek.ptr = Nes::_0_nrom_cpu_peek;
        mapper.cpu_write.ptr = Nes::_0_nrom_cpu_write;
        mapper.read_chr.ptr = Nes::_0_nrom_read_chr;
        mapper.write_chr.ptr = Nes::_0_nrom_write_chr;
        mapper.read_nametable.ptr = Nes::_0_nrom_read_nametable;
        mapper.write_nametable.ptr = Nes::_0_nrom_write_nametable;

        mapper
    }

    pub(crate) fn _0_nrom_reload(&mut self) {
        self.mapper.cpu_read.ptr = Nes::_0_nrom_cpu_read;
        self.mapper.cpu_peek.ptr = Nes::_0_nrom_cpu_peek;
        self.mapper.cpu_write.ptr = Nes::_0_nrom_cpu_write;
        self.mapper.read_chr.ptr = Nes::_0_nrom_read_chr;
        self.mapper.write_chr.ptr = Nes::_0_nrom_write_chr;
        self.mapper.read_nametable.ptr = Nes::_0_nrom_read_nametable;
        self.mapper.write_nametable.ptr = Nes::_0_nrom_write_nametable;
    }

    pub(crate) fn _0_nrom_cpu_peek(&mut self, addr: usize) -> u8 {
        self._0_nrom_cpu_read(addr)
    }

    pub(crate) fn _0_nrom_cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.mapper
                    .cartridge
                    .read_prg_ram(addr - 0x6000, 0, BankSize::Kb8)
            }
            0x8000..=0xBFFF => {
                self.mapper
                    .cartridge
                    .read_prg_rom(addr - 0x8000, 0, BankSize::Kb16)
            }
            0xC000..=0xFFFF => self.mapper.cartridge.read_prg_rom(
                addr - 0xC000,
                self.mapper.prg_rom_count - 1,
                BankSize::Kb16,
            ),
            _ => self.cpu.open_bus,
        }
    }

    pub(crate) fn _0_nrom_cpu_write(&mut self, addr: usize, val: u8) {
        if let 0x6000..=0x7FFF = addr {
            self.mapper
                .cartridge
                .write_prg_ram(addr - 0x6000, 0, BankSize::Kb8, val);
        }
    }

    pub(crate) fn _0_nrom_read_chr(&mut self, addr: usize) -> u8 {
        self.mapper.cartridge.read_chr(addr, 0, BankSize::Kb8)
    }

    pub(crate) fn _0_nrom_write_chr(&mut self, addr: usize, val: u8) {
        if self.mapper.cartridge.header.chr_rom_count == 0 {
            self.mapper.cartridge.write_chr(addr, 0, BankSize::Kb8, val);
        }
    }

    pub(crate) fn _0_nrom_read_nametable(&mut self, addr: usize) -> u8 {
        self.mapper.nt_ram[addr]
    }

    pub(crate) fn _0_nrom_write_nametable(&mut self, addr: usize, val: u8) {
        self.mapper.nt_ram[addr] = val;
    }
}
