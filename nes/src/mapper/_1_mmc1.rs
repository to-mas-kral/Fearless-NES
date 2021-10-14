use crate::{
    cartridge::{BankSize, Cartridge},
    ppu::Mirroring,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct _1Mmc1 {
    shift: u8,
    prg_mode: u8,
    chr_mask: u8,
    chr_mode: u8,
    enable_ram: bool,
    ignore_write_cycle: u64,
    mirroring: Mirroring,

    prg_banks: u8,
    prg_0: usize,
    prg_1: usize,

    chr_0: usize,
    chr_1: usize,
}

impl _1Mmc1 {
    pub fn new(cartridge: &Cartridge) -> Self {
        // FIXME: only 8KB CHR RAM, emulate submappers (chr select...)
        // https://wiki.nesdev.org/w/index.php?title=NES_2.0_submappers#001:_MMC1
        let chr_mask = if cartridge.has_chr_ram()
            && cartridge.header.chr_ram_size == Some(BankSize::Kb8 as u32)
        {
            1
        } else {
            0xFF
        };

        let prg_banks = cartridge.prg_rom_count(BankSize::Kb16) as u8;

        Self {
            shift: 0x10,
            prg_mode: 3,
            chr_mode: 0,
            chr_mask,
            enable_ram: false,
            ignore_write_cycle: 0,
            mirroring: Mirroring::Horizontal,

            prg_banks,
            prg_0: 0,
            prg_1: Cartridge::map_bank(prg_banks - 1, BankSize::Kb16),

            chr_0: 0,
            chr_1: 0,
        }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF if self.enable_ram => cartridge.read_prg_ram(addr - 0x6000),
            0x8000..=0xBFFF => Some(cartridge.read_prg_rom(self.prg_0 + addr - 0x8000)),
            0xC000..=0xFFFF => Some(cartridge.read_prg_rom(self.prg_1 + addr - 0xC000)),
            _ => None,
        }
    }

    pub fn cpu_write(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8, cpu_cycle: u64) {
        match addr {
            0x6000..=0x7FFF if self.enable_ram => cartridge.write_prg_ram(addr - 0x6000, val),
            0x8000..=0xFFFF => {
                /* When the CPU writes to the serial port on consecutive cycles, the MMC1 ignores
                all writes but the first. This happens when the 6502 executes read-modify-write
                (RMW) instructions, such as DEC and ROR, by writing back the old value and then
                writing the new value on the next cycle. */
                if cpu_cycle == self.ignore_write_cycle {
                    self.ignore_write_cycle = 0;
                    return;
                } else {
                    self.ignore_write_cycle = cpu_cycle + 1;
                }

                /* Writing a value with bit 7 set to any address in $8000-$FFFF clears the shift register
                to its initial state. */
                if val & 0x80 != 0 {
                    self.shift = 0x10;
                    self.prg_mode = 3;
                } else {
                    /* To change a register's value, the CPU writes five times with bit 7 clear and
                    a bit of the desired value in bit 0. On the first four writes, the MMC1 shifts
                    bit 0 into a shift register. On the fifth write, the MMC1 copies bit 0 and the
                    shift register contents into an internal register selected by bits 14 and 13 of
                    the address, and then it clears the shift register. */
                    let shift = ((val & 1) << 4) | self.shift >> 1;

                    if self.shift & 1 != 0 {
                        match addr & 0x6000 {
                            0 => self.write_control(shift),
                            0x2000 => self.select_chr_0(shift),
                            0x4000 => self.select_chr_1(shift),
                            0x6000 => self.select_prg(shift),
                            _ => unreachable!(),
                        }

                        /* After the fifth write, the shift register is cleared automatically,
                        so a write to the shift register with bit 7 on to reset it is not needed. */
                        self.shift = 0x10;
                    } else {
                        self.shift = shift;
                    }
                }
            }
            _ => (),
        }
    }

    /* Control (internal, $8000-$9FFF)
    4bit0
    -----
    CPPMM
    |||||
    |||++- Mirroring (0: one-screen, lower bank; 1: one-screen, upper bank;
    |||               2: vertical; 3: horizontal)
    |++--- PRG ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
    |                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
    |                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
    +----- CHR ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks) */
    #[inline]
    fn write_control(&mut self, val: u8) {
        self.mirroring = match val & 3 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };

        self.prg_mode = (val & 0xC) >> 2;
        match self.prg_mode {
            0 | 1 => (),
            2 => self.prg_0 = 0,
            3 => self.prg_1 = Cartridge::map_bank(self.prg_banks - 1, BankSize::Kb16),
            _ => unreachable!(),
        }

        self.chr_mode = (val & 0x10) >> 4;
    }

    /* PRG bank (internal, $E000-$FFFF)
    4bit0
    -----
    RPPPP
    |||||
    |++++- Select 16 KB PRG ROM bank (low bit ignored in 32 KB mode)
    +----- PRG RAM chip enable (0: enabled; 1: disabled; ignored on MMC1A) */
    #[inline]
    fn select_prg(&mut self, val: u8) {
        self.enable_ram = val & 0x10 == 0;
        match self.prg_mode {
            0 | 1 => {
                self.prg_0 = Cartridge::map_bank(val & 0xE, BankSize::Kb16);
                self.prg_1 = Cartridge::map_bank((val & 0xE) + 1, BankSize::Kb16);
            }
            2 => self.prg_1 = Cartridge::map_bank(val & 0xF, BankSize::Kb16),
            3 => self.prg_0 = Cartridge::map_bank(val & 0xF, BankSize::Kb16),
            _ => unreachable!(),
        }
    }

    /* CHR bank 0 (internal, $A000-$BFFF)
    4bit0
    -----
    CCCCC
    |||||
    +++++- Select 4 KB or 8 KB CHR bank at PPU 000 (low bit ignored in 8 KB mode)
    For carts with 8 KiB of CHR (be it ROM or RAM), MMC1 follows the common
    behavior of using only the low-order bits: the bank number is in effect ANDed with 1. */
    #[inline]
    fn select_chr_0(&mut self, val: u8) {
        if self.chr_mode == 1 {
            self.chr_0 = Cartridge::map_bank(val & self.chr_mask, BankSize::Kb4);
        } else {
            self.chr_0 = Cartridge::map_bank(val & 0xFE, BankSize::Kb4);
            self.chr_1 = Cartridge::map_bank((val & 0xFE) + 1, BankSize::Kb4);
        }
    }

    /* CHR bank 1 (internal, $C000-$DFFF)
    4bit0
    -----
    CCCCC
    |||||
    +++++- Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode) */
    #[inline]
    fn select_chr_1(&mut self, val: u8) {
        if self.chr_mode == 1 {
            self.chr_1 = Cartridge::map_bank(val & self.chr_mask, BankSize::Kb4);
        }
    }

    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        match addr {
            0..=0xFFF => cartridge.read_chr(self.chr_0 + addr),
            0x1000..=0x1FFF => cartridge.read_chr(self.chr_1 + (addr & 0xFFF)),
            _ => unreachable!(),
        }
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        match addr {
            0..=0xFFF => cartridge.write_chr(self.chr_0 + addr, val),
            0x1000..=0x1FFF => {
                cartridge.write_chr(self.chr_1 + (addr & 0xFFF), val);
            }
            _ => unreachable!(),
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
