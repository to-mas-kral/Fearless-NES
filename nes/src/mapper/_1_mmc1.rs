use super::super::cartridge::Cartridge;
use super::super::cartridge::Mirroring;
use super::super::Nes;
use super::Mapper;

pub struct Mmc1 {
    pub prg_1: usize,
    pub prg_2: usize,
    pub chr_1: usize,
    pub chr_2: usize,

    pub shift: u8,

    mirroring: Mirroring,
    prg_mode: u8,
    chr_mode: u8,
    chr_mask: u8,
    enable_ram: bool,

    pub nt_ram: [u8; 0x1000],
    pub cartridge: Cartridge,

    ignore_write: u64,

    pub nes: *mut Nes,
}

impl Mmc1 {
    pub fn new(cartridge: Cartridge) -> Mmc1 {
        let prg_count = cartridge.header.prg_rom_size as usize;
        let prg_2 = 0x4000 * (prg_count - 1);

        let chr_mask = if cartridge.header.chr_rom_size <= 1 {
            1
        } else {
            0xFF
        };

        Mmc1 {
            prg_1: 0,
            prg_2,
            chr_1: 0,
            chr_2: 0,

            shift: 0x10,

            mirroring: Mirroring::Horizontal,
            prg_mode: 3,
            chr_mode: 0,
            chr_mask,
            enable_ram: false,

            nt_ram: [0; 0x1000],
            cartridge,

            ignore_write: 0,

            nes: 0 as *mut Nes,
        }
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
            2 => self.prg_1 = 0,
            3 => self.prg_2 = (self.cartridge.header.prg_rom_size as usize - 1) * 0x4000,
            _ => unreachable!(),
        }

        self.chr_mode = (val & 0x10) >> 4;

        //println!("val: {}, mode: {}", val, self.prg_mode);
        //println!("prg_1: 0x{:X}, prg_2: 0x{:X}", self.prg_1, self.prg_2);
    }

    //CHR bank 0 (internal, $A000-$BFFF)
    //4bit0
    //-----
    //CCCCC
    //|||||
    //+++++- Select 4 KB or 8 KB CHR bank at PPU $0000 (low bit ignored in 8 KB mode)
    //For carts with 8 KiB of CHR (be it ROM or RAM), MMC1 follows the common
    //behavior of using only the low-order bits: the bank number is in effect ANDed with 1.
    #[inline]
    fn write_chr_0(&mut self, val: u8) {
        if self.chr_mode == 1 {
            self.chr_1 = 0x1000 * (val & self.chr_mask) as usize;
        } else {
            self.chr_1 = 0x1000 * (val as usize & 0xFE);
            self.chr_2 = self.chr_1 + 0x1000;
        }
        //println!(
        //    "switching chr banks, mode: {}, chr_1: 0x{:X}, chr_2: 0x{:X}",
        //    self.chr_mode, self.chr_1, self.chr_2
        //);
    }

    //CHR bank 1 (internal, $C000-$DFFF)
    //4bit0
    //-----
    //CCCCC
    //|||||
    //+++++- Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode)
    #[inline]
    fn write_chr_1(&mut self, val: u8) {
        if self.chr_mode == 1 {
            self.chr_2 = 0x1000 * (val & self.chr_mask) as usize;
        }
        //println!(
        //    "switching chr banks, mode: {}, chr_1: 0x{:X}, chr_2: 0x{:X}",
        //    self.chr_mode, self.chr_1, self.chr_2
        //);
    }

    //PRG bank (internal, $E000-$FFFF)
    //4bit0
    //-----
    //RPPPP
    //|||||
    //|++++- Select 16 KB PRG ROM bank (low bit ignored in 32 KB mode)
    //+----- PRG RAM chip enable (0: enabled; 1: disabled; ignored on MMC1A)
    #[inline]
    fn write_prg(&mut self, val: u8) {
        self.enable_ram = val & 0x10 == 0;
        match self.prg_mode {
            0 | 1 => {
                self.prg_1 = 0x4000 * (val as usize & 0xE);
                self.prg_2 = self.prg_1 + 0x4000;
            }
            2 => self.prg_2 = 0x4000 * (val as usize & 0xF),
            3 => self.prg_1 = 0x4000 * (val as usize & 0xF),
            _ => unreachable!(),
        }

        //println!("size: {:?}", self.cartridge.header.prg_rom_size);

        //println!("val: 0x{:X}, mode: {}", val, self.prg_mode);
        //println!(
        //    "new prg_1: 0x{:X}, new prg_2: 0x{:X}",
        //    self.prg_1, self.prg_2
        //);
    }
}

impl Mapper for Mmc1 {
    fn cpu_peek(&mut self, addr: usize) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.cartridge.prg_ram[addr - 0x6000],
            0x8000..=0xBFFF => self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)],
            0xC000..=0xFFFF => self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)],
            _ => 0,
        }
    }

    fn cpu_read(&mut self, addr: usize) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF if self.enable_ram => {
                Some(self.cartridge.prg_ram[addr - 0x6000])
            }
            0x8000..=0xBFFF => Some(self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)]),
            0xC000..=0xFFFF => Some(self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)]),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: usize, val: u8) {
        match addr {
            0x6000..=0x7FFF if self.enable_ram => {
                self.cartridge.prg_ram[addr - 0x6000] = val
            }
            0x8000..=0xFFFF => {
                //When the CPU writes to the serial port on consecutive cycles, the MMC1 ignores
                //all writes but the first. This happens when the 6502 executes read-modify-write
                //(RMW) instructions, such as DEC and ROR, by writing back the old value and then
                //writing the new value on the next cycle.
                if nes!(self.nes).cycle_count == self.ignore_write {
                    self.ignore_write = 0;
                    return;
                } else {
                    self.ignore_write = nes!(self.nes).cycle_count + 1;
                }
                //println!("cpu write, val: 0x{:X}", val);

                //Writing a value with bit 7 set to any address in $8000-$FFFF clears the shift register
                //to its initial state.
                if val & 0x80 != 0 {
                    self.shift = 0x10;
                    self.prg_mode = 3;
                } else {
                    //To change a register's value, the CPU writes five times with bit 7 clear and
                    //a bit of the desired value in bit 0. On the first four writes, the MMC1 shifts
                    //bit 0 into a shift register. On the fifth write, the MMC1 copies bit 0 and the
                    //shift register contents into an internal register selected by bits 14 and 13 of
                    //the address, and then it clears the shift register.
                    let shift = ((val & 1) << 4) | self.shift >> 1;

                    //println!("shifter after: {:b}", shift);
                    if self.shift & 1 != 0 {
                        match addr & 0x6000 {
                            0 => self.write_control(shift),
                            0x2000 => self.write_chr_0(shift),
                            0x4000 => self.write_chr_1(shift),
                            0x6000 => self.write_prg(shift),
                            _ => unreachable!(),
                        }

                        //After the fifth write, the shift register is cleared automatically,
                        //so a write to the shift register with bit 7 on to reset it is not needed.

                        self.shift = 0x10;
                    } else {
                        self.shift = shift;
                    }
                }
            }
            _ => (),
        }
    }

    fn read_chr(&mut self, addr: usize) -> u8 {
        //println!("chr reading, addr 0x{:X}", addr);
        //println!("chr_1: 0x{:X}, chr_2 0x{:X}", self.chr_1, self.chr_2);
        match addr {
            0..=0xFFF => self.cartridge.chr[self.chr_1 + addr],
            0x1000..=0x1FFF => self.cartridge.chr[self.chr_2 + (addr & 0xFFF)],
            _ => unreachable!(),
        }
    }

    fn write_chr(&mut self, addr: usize, val: u8) {
        //println!("chr writing, addr 0x{:X}", addr);
        //println!("chr_1: 0x{:X}, chr_2 0x{:X}", self.chr_1, self.chr_2);
        match addr {
            0..=0xFFF => self.cartridge.chr[self.chr_1 + addr] = val,
            0x1000..=0x1FFF => self.cartridge.chr[self.chr_2 + (addr & 0xFFF)] = val,
            _ => unreachable!(),
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

    fn update_nes_ptr(&mut self, ptr: *mut Nes) {
        self.nes = ptr;
    }
}
