use bincode::{Decode, Encode};

use crate::{ppu::Mirroring, BankSize, Cartridge};

#[derive(Decode, Encode)]
pub struct _69Fme7 {
    prg_0: usize,
    prg_1: usize,
    prg_2: usize,
    prg_3: usize,
    prg_4: usize,

    enable_prg_ram: bool,
    prg_ram_selected: bool,

    chr_0: usize,
    chr_1: usize,
    chr_2: usize,
    chr_3: usize,
    chr_4: usize,
    chr_5: usize,
    chr_6: usize,
    chr_7: usize,

    mirroring: Mirroring,

    irq_enable: bool,
    irq_counter_enable: bool,
    irq_counter: u16,
    fire_irq: bool,

    cmd: u8,
}

impl _69Fme7 {
    pub fn new(cartridge: &Cartridge) -> Self {
        let prg_banks = cartridge.prg_rom_count(BankSize::Kb8) as u8;

        Self {
            prg_0: 0,
            prg_1: 0,
            prg_2: 0,
            prg_3: 0,
            prg_4: Cartridge::map_bank_prg_wrap(cartridge, prg_banks - 1, BankSize::Kb8),

            enable_prg_ram: false,
            prg_ram_selected: false,

            chr_0: 0,
            chr_1: 0,
            chr_2: 0,
            chr_3: 0,
            chr_4: 0,
            chr_5: 0,
            chr_6: 0,
            chr_7: 0,

            mirroring: Mirroring::Vertical,

            irq_enable: false,
            irq_counter_enable: false,
            irq_counter: 0,
            fire_irq: false,

            cmd: 0,
        }
    }

    /*
    CPU $6000-$7FFF: 8 KB Bankable PRG ROM or PRG RAM
    CPU $8000-$9FFF: 8 KB Bankable PRG ROM
    CPU $A000-$BFFF: 8 KB Bankable PRG ROM
    CPU $C000-$DFFF: 8 KB Bankable PRG ROM
    CPU $E000-$FFFF: 8 KB PRG ROM, fixed to the last bank of ROM
    */
    pub fn cpu_read(&self, cartridge: &Cartridge, addr: usize) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => {
                match self.prg_ram_selected {
                    true => match self.enable_prg_ram {
                        true => cartridge.read_prg_ram(addr - 0x6000),
                        /*
                        Open bus occurs if the RAM / ROM Select Bit is 1 (RAM selected),
                        but the RAM Enable Bit is 0 (disabled);
                        */
                        false => None,
                    },
                    false => Some(cartridge.read_prg_rom(self.prg_0 + addr - 0x6000)),
                }
            }
            0x8000..=0x9FFF => Some(cartridge.read_prg_rom(self.prg_1 + addr - 0x8000)),
            0xA000..=0xBFFF => Some(cartridge.read_prg_rom(self.prg_2 + addr - 0xA000)),
            0xC000..=0xDFFF => Some(cartridge.read_prg_rom(self.prg_3 + addr - 0xC000)),
            0xE000..=0xFFFF => Some(cartridge.read_prg_rom(self.prg_4 + addr - 0xE000)),
            _ => unreachable!(),
        }
    }

    /*
    Command Register ($8000-$9FFF)
    7  bit  0
    ---- ----
    .... CCCC
         ||||
         ++++- The command number to invoke when writing to the Parameter Register

    Parameter Register ($A000-$BFFF)

    7  bit  0
    ---- ----
    PPPP PPPP
    |||| ||||
    ++++-++++- The parameter to use for this command.

    Writing to this register invokes the command in the Command Register.
    */
    pub fn cpu_write(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                if self.prg_ram_selected && self.enable_prg_ram {
                    cartridge.write_prg_ram(addr - 0x6000, val);
                }
            }
            0x8000..=0x9FFF => self.cmd = val & 0xF,
            0xA000..=0xBFFF => match self.cmd {
                0..=7 => self.select_chr(val),
                8..=0xB => self.select_prg(val, cartridge),
                0xC => self.select_mirroring(val),
                0xD..=0xF => self.set_irq(val),
                _ => unreachable!(),
            },
            _ => (),
        }
    }

    fn select_chr(&mut self, cmd_param: u8) {
        match self.cmd {
            0 => self.chr_0 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            1 => self.chr_1 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            2 => self.chr_2 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            3 => self.chr_3 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            4 => self.chr_4 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            5 => self.chr_5 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            6 => self.chr_6 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            7 => self.chr_7 = Cartridge::map_bank(cmd_param, BankSize::Kb1),
            _ => unreachable!(),
        }
    }

    /*
    7  bit  0
    ---- ----
    ERbB BBBB
    |||| ||||
    ||++-++++- The bank number to select at CPU $6000 - $7FFF
    |+------- RAM / ROM Select Bit
    |           0 = PRG ROM
    |           1 = PRG RAM
    +-------- RAM Enable Bit (6264 +CE line)
                0 = PRG RAM Disabled
                1 = PRG RAM Enabled
    */
    fn select_prg(&mut self, cmd_param: u8, cartridge: &mut Cartridge) {
        /*
        The FME-7 has up to 6 bits for PRG banking (512 KiB), though this was never used in a game.
        The 5A and 5B, however, support only 5 (256 KiB)—hence the lowercase 'b' above.
        */

        // Games expect bank numbers to wrap around the total number of banks...
        match self.cmd {
            8 => {
                self.enable_prg_ram = (cmd_param & 0x80) != 0;
                self.prg_ram_selected = (cmd_param & 0x40) != 0;
                self.prg_0 = cartridge.map_bank_prg_wrap(cmd_param & 0x3F, BankSize::Kb8);
            }
            9 => self.prg_1 = cartridge.map_bank_prg_wrap(cmd_param & 0x3F, BankSize::Kb8),
            0xA => self.prg_2 = cartridge.map_bank_prg_wrap(cmd_param & 0x3F, BankSize::Kb8),
            0xB => self.prg_3 = cartridge.map_bank_prg_wrap(cmd_param & 0x3F, BankSize::Kb8),
            _ => unreachable!(),
        }
    }

    /*
    7  bit  0
    ---- ----
    .... ..MM
           ||
           ++- Mirroring Mode
                0 = Vertical
                1 = Horizontal
                2 = One Screen Mirroring from $2000 ("1ScA")
                3 = One Screen Mirroring from $2400 ("1ScB")
    */
    fn select_mirroring(&mut self, cmd_param: u8) {
        self.mirroring = match cmd_param & 0b11 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLow,
            3 => Mirroring::SingleScreenHigh,
            _ => unreachable!(),
        };
    }

    /*
    IRQ Control ($D)
    7  bit  0
    ---- ----
    C... ...T
    |       |
    |       +- IRQ Enable
    |           0 = Do not generate IRQs
    |           1 = Do generate IRQs
    +-------- IRQ Counter Enable
                0 = Disable Counter Decrement
                1 = Enable Counter Decrement

    All writes to this register acknowledge an active IRQ.

    IRQ Counter Low Byte ($E)
    7  bit  0
    ---- ----
    LLLL LLLL
    |||| ||||
    ++++-++++- The low eight bits of the IRQ counter

    IRQ Counter High Byte ($F)
    7  bit  0
    ---- ----
    HHHH HHHH
    |||| ||||
    ++++-++++- The high eight bits of the IRQ counter
    */
    fn set_irq(&mut self, cmd_param: u8) {
        match self.cmd {
            0xD => {
                self.irq_enable = (cmd_param & 1) != 0;
                self.irq_counter_enable = (cmd_param & 0x80) != 0;
                self.fire_irq = false;
            }
            0xE => {
                self.irq_counter = (self.irq_counter & 0xFF00) | cmd_param as u16;
            }
            0xF => {
                self.irq_counter = (self.irq_counter & 0x00FF) | ((cmd_param as u16) << 8);
            }
            _ => unreachable!(),
        }
    }

    /*
    PPU $0000-$03FF: 1 KB Bankable CHR ROM
    PPU $0400-$07FF: 1 KB Bankable CHR ROM
    PPU $0800-$0BFF: 1 KB Bankable CHR ROM
    PPU $0C00-$0FFF: 1 KB Bankable CHR ROM
    PPU $1000-$13FF: 1 KB Bankable CHR ROM
    PPU $1400-$17FF: 1 KB Bankable CHR ROM
    PPU $1800-$1BFF: 1 KB Bankable CHR ROM
    PPU $1C00-$1FFF: 1 KB Bankable CHR ROM
    */
    pub fn read_chr(&self, cartridge: &Cartridge, addr: usize) -> u8 {
        match addr {
            0x0000..=0x03FF => cartridge.read_chr(self.chr_0 + addr),
            0x0400..=0x07FF => cartridge.read_chr(self.chr_1 + addr - 0x0400),
            0x0800..=0x0BFF => cartridge.read_chr(self.chr_2 + addr - 0x0800),
            0x0C00..=0x0FFF => cartridge.read_chr(self.chr_3 + addr - 0x0C00),
            0x1000..=0x13FF => cartridge.read_chr(self.chr_4 + addr - 0x1000),
            0x1400..=0x17FF => cartridge.read_chr(self.chr_5 + addr - 0x1400),
            0x1800..=0x1BFF => cartridge.read_chr(self.chr_6 + addr - 0x1800),
            0x1C00..=0x1FFF => cartridge.read_chr(self.chr_7 + addr - 0x1C00),
            _ => unreachable!(),
        }
    }

    pub fn write_chr(&mut self, cartridge: &mut Cartridge, addr: usize, val: u8) {
        match addr {
            0x0000..=0x03FF => cartridge.write_chr(self.chr_0 + addr, val),
            0x0400..=0x07FF => cartridge.write_chr(self.chr_1 + addr - 0x0400, val),
            0x0800..=0x0BFF => cartridge.write_chr(self.chr_2 + addr - 0x0800, val),
            0x0C00..=0x0FFF => cartridge.write_chr(self.chr_3 + addr - 0x0C00, val),
            0x1000..=0x13FF => cartridge.write_chr(self.chr_4 + addr - 0x1000, val),
            0x1400..=0x17FF => cartridge.write_chr(self.chr_5 + addr - 0x1400, val),
            0x1800..=0x1BFF => cartridge.write_chr(self.chr_6 + addr - 0x1800, val),
            0x1C00..=0x1FFF => cartridge.write_chr(self.chr_7 + addr - 0x1C00, val),
            _ => unreachable!(),
        };
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    /*
    The IRQ feature of FME-7 is a CPU cycle counting IRQ generator.
    When enabled the 16-bit IRQ counter is decremented once per CPU cycle.
    When the IRQ counter is decremented from $0000 to $FFFF an IRQ is generated.
    The IRQ line is held low until it is acknowledged.
    */
    pub fn clock(&mut self, irq_signal: &mut bool) {
        if self.irq_counter_enable {
            if self.irq_counter == 0 {
                self.fire_irq = true;
                self.irq_counter = 0xFFFF;
            } else {
                self.irq_counter -= 1;
            }
        }

        if self.irq_enable && self.fire_irq {
            *irq_signal = true;
        }
    }
}
