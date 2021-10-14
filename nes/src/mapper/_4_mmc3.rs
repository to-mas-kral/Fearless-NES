use serde::{Deserialize, Serialize};

use crate::{
    cartridge::{BankSize, Cartridge},
    ppu::Mirroring,
};

#[derive(Serialize, Deserialize)]
pub struct _4Mmc3 {
    prg_bank_mode: u8,
    chr_bank_mode: u8,
    bank_update: u8,
    irq_latch: u8,
    irq_counter: u8,
    irq_should_reload: bool,
    irqs_enabled: bool,
    a12_state: bool,
    /// PPU cycle of the last falling edge of A12 before filtering
    a12_falling_cycle: u32,
    mirroring: Mirroring,

    /// 8KB units
    prg_rom_count: u8,
    prg_0: usize,
    prg_1: usize,
    prg_end_1: usize,
    prg_end_2: usize,

    /// 1KB units
    chr_count: u16,
    chr_0: usize,
    chr_1: usize,
    chr_2: usize,
    chr_3: usize,
    chr_4: usize,
    chr_5: usize,
}

impl _4Mmc3 {
    pub fn new(cartridge: &Cartridge) -> Self {
        // TODO: some MMC3 games use both CHR ROM and CHR RAM
        let chr_count = match (
            cartridge.chr_rom_count(BankSize::Kb1),
            cartridge.chr_ram_count(BankSize::Kb1),
        ) {
            (Some(count), None) => count as u16,
            (None, Some(count)) => count as u16,
            _ => todo!("handle games with both CHR ROM and CHR RAM"),
        };

        let prg_rom_count = cartridge.prg_rom_count(BankSize::Kb8) as u8;

        Self {
            // MMC3 initial state is unspecified
            prg_bank_mode: 0,
            chr_bank_mode: 0,
            bank_update: 0,
            irq_latch: 0,
            irq_counter: 0,
            irq_should_reload: false,
            irqs_enabled: false,
            a12_state: false,
            a12_falling_cycle: 0,
            mirroring: cartridge.header.mirroring,

            prg_rom_count,
            prg_0: 0,
            prg_1: 0,
            prg_end_1: Cartridge::map_bank(prg_rom_count - 1, BankSize::Kb8),
            prg_end_2: Cartridge::map_bank(prg_rom_count - 2, BankSize::Kb8),

            chr_count,
            chr_0: 0,
            chr_1: 0,
            chr_2: 0,
            chr_3: 0,
            chr_4: 0,
            chr_5: 0,
        }
    }

    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => cartridge.read_prg_ram(addr - 0x6000),
            0xA000..=0xBFFF => Some(cartridge.read_prg_rom(self.prg_1 + addr - 0xA000)),
            0xE000..=0xFFFF => Some(cartridge.read_prg_rom(self.prg_end_1 + addr - 0xE000)),
            _ => match self.prg_bank_mode {
                0 => match addr {
                    0x8000..=0x9FFF => Some(cartridge.read_prg_rom(self.prg_0 + addr - 0x8000)),
                    0xC000..=0xDFFF => Some(cartridge.read_prg_rom(self.prg_end_2 + addr - 0xC000)),
                    _ => None,
                },
                1 => match addr {
                    0x8000..=0x9FFF => Some(cartridge.read_prg_rom(self.prg_end_2 + addr - 0x8000)),
                    0xC000..=0xDFFF => Some(cartridge.read_prg_rom(self.prg_0 + addr - 0xC000)),
                    _ => None,
                },
                _ => unreachable!(),
            },
        }
    }

    pub fn cpu_write(
        &mut self,
        cartridge: &mut Cartridge,
        addr: usize,
        val: u8,
        cpu_irq: &mut bool,
    ) {
        match addr {
            0x6000..=0x7FFF => cartridge.write_prg_ram(addr - 0x6000, val),
            0x8000..=0x9FFE if addr % 2 == 0 => {
                self.bank_update = val & 0x7;
                self.prg_bank_mode = (val & 0x40) >> 6;
                self.chr_bank_mode = (val & 0x80) >> 7;
            }
            0x8001..=0x9FFF if addr % 2 == 1 => match self.bank_update {
                // This isn't mentioned in the docs, but some games (e.g. Super C) use banks higher
                // than (prg / chr)_rom_count and expect wrap-around

                // R0 and R1 ignore the bottom bit, as the value written still counts
                // banks in 1KB units but odd numbered banks can't be selected.
                0b000 => {
                    self.chr_0 = Cartridge::map_bank(
                        ((val & 0xFE) as u16 % self.chr_count) as u8,
                        BankSize::Kb1,
                    )
                }
                0b001 => {
                    self.chr_1 = Cartridge::map_bank(
                        ((val & 0xFE) as u16 % self.chr_count) as u8,
                        BankSize::Kb1,
                    )
                }
                0b010 => {
                    self.chr_2 =
                        Cartridge::map_bank(((val) as u16 % self.chr_count) as u8, BankSize::Kb1)
                }
                0b011 => {
                    self.chr_3 =
                        Cartridge::map_bank(((val) as u16 % self.chr_count) as u8, BankSize::Kb1)
                }
                0b100 => {
                    self.chr_4 =
                        Cartridge::map_bank(((val) as u16 % self.chr_count) as u8, BankSize::Kb1)
                }
                0b101 => {
                    self.chr_5 =
                        Cartridge::map_bank(((val) as u16 % self.chr_count) as u8, BankSize::Kb1)
                }
                0b110 => {
                    self.prg_0 =
                        Cartridge::map_bank((val & 0x3F) % self.prg_rom_count, BankSize::Kb8)
                }
                0b111 => {
                    self.prg_1 =
                        Cartridge::map_bank((val & 0x3F) % self.prg_rom_count, BankSize::Kb8)
                }
                _ => unreachable!(),
            },
            0xA000..=0xBFFE if addr % 2 == 0 => {
                self.mirroring = if val & 1 == 1 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                }
            }
            // Do not emulate RAM protect for better compatibility with MMC6
            0xA001..=0xBFFF if addr % 2 == 1 => (),
            0xC000..=0xDFFE if addr % 2 == 0 => {
                self.irq_latch = val;
            }
            0xC001..=0xDFFF if addr % 2 == 1 => {
                self.irq_should_reload = true;
            }
            0xE000..=0xFFFE if addr % 2 == 0 => {
                self.irqs_enabled = false;
                *cpu_irq = false;
            }
            0xE001..=0xFFFF if addr % 2 == 1 => {
                self.irqs_enabled = true;
            }
            _ => unreachable!(),
        }
    }

    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        match self.chr_bank_mode {
            0 => match addr {
                0x000..=0x7FF => cartridge.read_chr(self.chr_0 + addr),
                0x800..=0xFFF => cartridge.read_chr(self.chr_1 + addr - 0x800),
                0x1000..=0x13FF => cartridge.read_chr(self.chr_2 + addr - 0x1000),
                0x1400..=0x17FF => cartridge.read_chr(self.chr_3 + addr - 0x1400),
                0x1800..=0x1BFF => cartridge.read_chr(self.chr_4 + addr - 0x1800),
                0x1C00..=0x1FFF => cartridge.read_chr(self.chr_5 + addr - 0x1C00),
                _ => unreachable!(),
            },
            1 => match addr {
                0x000..=0x3FF => cartridge.read_chr(self.chr_2 + addr),
                0x400..=0x7FF => cartridge.read_chr(self.chr_3 + addr - 0x400),
                0x800..=0xBFF => cartridge.read_chr(self.chr_4 + addr - 0x800),
                0xC00..=0xFFF => cartridge.read_chr(self.chr_5 + addr - 0xC00),
                0x1000..=0x17FF => cartridge.read_chr(self.chr_0 + addr - 0x1000),
                0x1800..=0x1FFF => cartridge.read_chr(self.chr_1 + addr - 0x1800),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        match self.chr_bank_mode {
            0 => match addr {
                0x000..=0x7FF => cartridge.write_chr(self.chr_0 + addr, val),
                0x800..=0xFFF => cartridge.write_chr(self.chr_1 + addr - 0x800, val),
                0x1000..=0x13FF => cartridge.write_chr(self.chr_2 + addr - 0x1000, val),
                0x1400..=0x17FF => cartridge.write_chr(self.chr_3 + addr - 0x1400, val),
                0x1800..=0x1BFF => cartridge.write_chr(self.chr_4 + addr - 0x1800, val),
                0x1C00..=0x1FFF => cartridge.write_chr(self.chr_5 + addr - 0x1C00, val),
                _ => unreachable!(),
            },
            1 => match addr {
                0x000..=0x3FF => cartridge.write_chr(self.chr_2 + addr, val),
                0x400..=0x7FF => cartridge.write_chr(self.chr_3 + addr - 0x400, val),
                0x800..=0xBFF => cartridge.write_chr(self.chr_4 + addr - 0x800, val),
                0xC00..=0xFFF => cartridge.write_chr(self.chr_5 + addr - 0xC00, val),
                0x1000..=0x17FF => cartridge.write_chr(self.chr_0 + addr - 0x1000, val),
                0x1800..=0x1FFF => cartridge.write_chr(self.chr_1 + addr - 0x1800, val),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    pub fn notify_a12(&mut self, a12: bool, ppu_cycle: u32, cpu_irq: &mut bool) {
        /* http://archive.nes.science/nesdev-forums/f3/t1100.xhtml#p9558
        In addition, the MMC3 will not detect a rising edge on PPU A12 if it was low for less than ~2 CPU
        cycles (OR if the last rising edge was less than ~3 CPU cycles ago - the exact behavior is not known)
        - during sprite fetches, the PPU rapidly alternates between $1xxx and $2xxx, and the MMC3 does not see A13
        - as such, the PPU will send 8 rising edges on A12 during the sprite fetch portion of the scanline
        (with 8 pixel clocks, or 2.67 CPU cycles between them), but the MMC3 will only see the first one.

        NOTE: Mesen clocks the IRQ counter after 10 PPU cycles have elapsed after a falling edge */

        let prev_a12 = self.a12_state;
        self.a12_state = a12;

        let cycles_elapsed = if self.a12_falling_cycle > 0 {
            ppu_cycle.wrapping_sub(self.a12_falling_cycle)
        } else {
            0
        };

        match (prev_a12, a12) {
            (true, false) => self.a12_falling_cycle = ppu_cycle,
            (false, true) => {
                if cycles_elapsed <= 8 {
                    self.a12_falling_cycle = 0;
                } else {
                    // IRQ counter clock

                    if self.irq_counter == 0 || self.irq_should_reload {
                        self.irq_counter = self.irq_latch;
                        self.irq_should_reload = false;
                    } else {
                        self.irq_counter -= 1;
                    }

                    // "Normal" MMC3 IRQ behavior
                    if self.irq_counter == 0 && self.irqs_enabled {
                        *cpu_irq = true;
                    }
                }
            }
            _ => (),
        }
    }
}
