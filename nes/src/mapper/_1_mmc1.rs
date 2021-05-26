use super::{
    super::{
        cartridge::{BankSize, Cartridge, Mirroring},
        Nes,
    },
    Mapper,
};

impl Nes {
    pub(crate) fn _1_mmc1_initialize(cartridge: Cartridge) -> Mapper {
        let mut mapper = Mapper::new(cartridge);

        let chr_mask = if mapper.cartridge.header.chr_rom_count <= 1 {
            1
        } else {
            0xFF
        };

        mapper.shift = 0x10;

        mapper.mirroring = Mirroring::Horizontal;
        mapper.prg_mode = 3;
        mapper.chr_mode = 0;
        mapper.chr_mask = chr_mask;
        mapper.enable_ram = false;

        mapper.ignore_write = 0;

        mapper.prg_rom_indices = vec![0, mapper.cartridge.header.prg_rom_count - 1];
        mapper.chr_indices = vec![0, 0];

        mapper.cpu_read.ptr = Nes::_1_mmc1_cpu_read;
        mapper.cpu_peek.ptr = Nes::_1_mmc1_cpu_peek;
        mapper.cpu_write.ptr = Nes::_1_mmc1_cpu_write;
        mapper.read_chr.ptr = Nes::_1_mmc1_read_chr;
        mapper.write_chr.ptr = Nes::_1_mmc1_write_chr;
        mapper.read_nametable.ptr = Nes::_1_mmc1_read_nametable;
        mapper.write_nametable.ptr = Nes::_1_mmc1_write_nametable;

        mapper
    }

    pub(crate) fn _1_mmc1_reload(&mut self) {
        self.mapper.cpu_read.ptr = Nes::_1_mmc1_cpu_read;
        self.mapper.cpu_peek.ptr = Nes::_1_mmc1_cpu_peek;
        self.mapper.cpu_write.ptr = Nes::_1_mmc1_cpu_write;
        self.mapper.read_chr.ptr = Nes::_1_mmc1_read_chr;
        self.mapper.write_chr.ptr = Nes::_1_mmc1_write_chr;
        self.mapper.read_nametable.ptr = Nes::_1_mmc1_read_nametable;
        self.mapper.write_nametable.ptr = Nes::_1_mmc1_write_nametable;
    }

    //Control (internal, $8000-$9FFF)
    //4bit0
    //-----
    //CPPMM
    //|||||
    //|||++- Mirroring (0: one-screen, lower bank; 1: one-screen, upper bank;
    //|||               2: vertical; 3: horizontal)
    //|++--- PRG ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
    //|                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
    //|                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
    //+----- CHR ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks)
    #[inline]
    pub(crate) fn _1_mmc1_write_control(&mut self, val: u8) {
        self.mapper.mirroring = match val & 3 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };

        self.mapper.prg_mode = (val & 0xC) >> 2;
        match self.mapper.prg_mode {
            0 | 1 => (),
            2 => self.mapper.prg_rom_indices[0] = 0,
            3 => {
                self.mapper.prg_rom_indices[1] =
                    self.mapper.cartridge.header.prg_rom_count - 1
            }
            _ => unreachable!(),
        }

        self.mapper.chr_mode = (val & 0x10) >> 4;
    }

    //CHR bank 0 (internal, $A000-$BFFF)
    //4bit0
    //-----
    //CCCCC
    //|||||
    //+++++- Select 4 KB or 8 KB CHR bank at PPU 000 (low bit ignored in 8 KB mode)
    //For carts with 8 KiB of CHR (be it ROM or RAM), MMC1 follows the common
    //behavior of using only the low-order bits: the bank number is in effect ANDed with 1.
    #[inline]
    pub(crate) fn _1_mmc1_write_chr_0(&mut self, val: u8) {
        if self.mapper.chr_mode == 1 {
            self.mapper.chr_indices[0] = val & self.mapper.chr_mask;
        } else {
            self.mapper.chr_indices[0] = val & 0xFE;
            self.mapper.chr_indices[1] = self.mapper.chr_indices[0] + 1;
        }
    }

    //CHR bank 1 (internal, $C000-$DFFF)
    //4bit0
    //-----
    //CCCCC
    //|||||
    //+++++- Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode)
    #[inline]
    pub(crate) fn _1_mmc1_write_chr_1(&mut self, val: u8) {
        if self.mapper.chr_mode == 1 {
            self.mapper.chr_indices[1] = val & self.mapper.chr_mask;
        }
    }

    //PRG bank (internal, $E000-$FFFF)
    //4bit0
    //-----
    //RPPPP
    //|||||
    //|++++- Select 16 KB PRG ROM bank (low bit ignored in 32 KB mode)
    //+----- PRG RAM chip enable (0: enabled; 1: disabled; ignored on MMC1A)
    #[inline]
    pub(crate) fn _1_mmc1_write_prg(&mut self, val: u8) {
        self.mapper.enable_ram = val & 0x10 == 0;
        match self.mapper.prg_mode {
            0 | 1 => {
                self.mapper.prg_rom_indices[0] = val & 0xE;
                self.mapper.prg_rom_indices[1] = self.mapper.prg_rom_indices[0] + 1;
            }
            2 => self.mapper.prg_rom_indices[1] = val & 0xF,
            3 => self.mapper.prg_rom_indices[0] = val & 0xF,
            _ => unreachable!(),
        }
    }

    pub(crate) fn _1_mmc1_cpu_peek(&mut self, addr: usize) -> u8 {
        self._1_mmc1_cpu_read(addr)
    }

    pub(crate) fn _1_mmc1_cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            0x6000..=0x7FFF if self.mapper.enable_ram => self
                .mapper
                .cartridge
                .read_prg_ram(addr - 0x6000, 0, BankSize::Kb8),
            0x8000..=0xBFFF => self.mapper.cartridge.read_prg_rom(
                addr - 0x8000,
                self.mapper.prg_rom_indices[0],
                BankSize::Kb16,
            ),
            0xC000..=0xFFFF => self.mapper.cartridge.read_prg_rom(
                addr - 0xC000,
                self.mapper.prg_rom_indices[1],
                BankSize::Kb16,
            ),
            _ => self.cpu.open_bus,
        }
    }

    pub(crate) fn _1_mmc1_cpu_write(&mut self, addr: usize, val: u8) {
        match addr {
            0x6000..=0x7FFF if self.mapper.enable_ram => {
                self.mapper
                    .cartridge
                    .write_prg_ram(addr - 0x6000, 0, BankSize::Kb8, val);
            }
            0x8000..=0xFFFF => {
                //When the CPU writes to the serial port on consecutive cycles, the MMC1 ignores
                //all writes but the first. This happens when the 6502 executes read-modify-write
                //(RMW) instructions, such as DEC and ROR, by writing back the old value and then
                //writing the new value on the next cycle.
                if self.cycle_count == self.mapper.ignore_write {
                    self.mapper.ignore_write = 0;
                    return;
                } else {
                    self.mapper.ignore_write = self.cycle_count + 1;
                }

                //Writing a value with bit 7 set to any address in $8000-$FFFF clears the shift register
                //to its initial state.
                if val & 0x80 != 0 {
                    self.mapper.shift = 0x10;
                    self.mapper.prg_mode = 3;
                } else {
                    //To change a register's value, the CPU writes five times with bit 7 clear and
                    //a bit of the desired value in bit 0. On the first four writes, the MMC1 shifts
                    //bit 0 into a shift register. On the fifth write, the MMC1 copies bit 0 and the
                    //shift register contents into an internal register selected by bits 14 and 13 of
                    //the address, and then it clears the shift register.
                    let shift = ((val & 1) << 4) | self.mapper.shift >> 1;

                    if self.mapper.shift & 1 != 0 {
                        match addr & 0x6000 {
                            0 => self._1_mmc1_write_control(shift),
                            0x2000 => self._1_mmc1_write_chr_0(shift),
                            0x4000 => self._1_mmc1_write_chr_1(shift),
                            0x6000 => self._1_mmc1_write_prg(shift),
                            _ => unreachable!(),
                        }

                        //After the fifth write, the shift register is cleared automatically,
                        //so a write to the shift register with bit 7 on to reset it is not needed.

                        self.mapper.shift = 0x10;
                    } else {
                        self.mapper.shift = shift;
                    }
                }
            }
            _ => (),
        }
    }

    pub(crate) fn _1_mmc1_read_chr(&mut self, addr: usize) -> u8 {
        match addr {
            0..=0xFFF => self.mapper.cartridge.read_chr(
                addr,
                self.mapper.chr_indices[0],
                BankSize::Kb4,
            ),
            0x1000..=0x1FFF => self.mapper.cartridge.read_chr(
                addr & 0xFFF,
                self.mapper.chr_indices[1],
                BankSize::Kb4,
            ),
            _ => unreachable!(),
        }
    }

    pub(crate) fn _1_mmc1_write_chr(&mut self, addr: usize, val: u8) {
        match addr {
            0..=0xFFF => self.mapper.cartridge.write_chr(
                addr,
                self.mapper.chr_indices[0],
                BankSize::Kb4,
                val,
            ),
            0x1000..=0x1FFF => self.mapper.cartridge.write_chr(
                addr & 0xFFF,
                self.mapper.chr_indices[1],
                BankSize::Kb4,
                val,
            ),
            _ => unreachable!(),
        }
    }

    pub(crate) fn _1_mmc1_read_nametable(&mut self, addr: usize) -> u8 {
        self.mapper.nt_ram[addr]
    }

    pub(crate) fn _1_mmc1_write_nametable(&mut self, addr: usize, val: u8) {
        self.mapper.nt_ram[addr] = val;
    }
}
