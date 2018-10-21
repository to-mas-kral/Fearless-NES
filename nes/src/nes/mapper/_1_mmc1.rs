use super::super::cartridge::Cartridge;
use super::super::cartridge::Mirroring;

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
    enable_ram: bool,

    pub nt_ram: [u8; 0x1000],
    pub cartridge: Cartridge,
}

impl Mmc1 {
    pub fn new(cartridge: Cartridge) -> Mmc1 {
        let prg_count = cartridge.header.prg_rom_size as usize;
        let prg_2 = 0x4000 * (prg_count - 1);

        Mmc1 {
            prg_1: 0,
            prg_2,
            chr_1: 0,
            chr_2: 0,

            shift: 0x10,

            mirroring: Mirroring::Horizontal,
            prg_mode: 3,
            chr_mode: 0,
            enable_ram: false,

            nt_ram: [0; 0x1000],
            cartridge,
        }
    }

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

    #[inline]
    fn write_chr_0(&mut self, val: u8) {
        if self.chr_mode == 1 {
            self.chr_1 = 0x1000 * (val & 1) as usize;
        } else {
            self.chr_1 = 1000 * (val as usize & 0xFE);
            self.chr_2 = self.chr_1 + 0x1000;
        }
    }

    #[inline]
    fn write_chr_1(&mut self, val: u8) {
        if self.chr_mode == 1 {
            self.chr_2 = 0x1000 * (val & 1) as usize;
        }
    }

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

        println!("size: {:?}", self.cartridge.header.prg_rom_size);

        println!("val: 0x{:X}, mode: {}", val, self.prg_mode);
        println!("new prg_1: 0x{:X}, new prg_2: 0x{:X}", self.prg_1, self.prg_2);
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
            0x6000..=0x7FFF if self.enable_ram => Some(self.cartridge.prg_ram[addr - 0x6000]),
            0x8000..=0xBFFF => Some(self.cartridge.prg_rom[self.prg_1 + (addr - 0x8000)]),
            0xC000..=0xFFFF => Some(self.cartridge.prg_rom[self.prg_2 + (addr - 0xC000)]),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: usize, val: u8) {
        //TODO: ignore consecutive writes
        match addr {
            0x6000..=0x7FFF if self.enable_ram => self.cartridge.prg_ram[addr - 0x6000] = val,
            0x8000..=0xFFFF => {
                println!("cpu write, val: 0x{:X}", val);
                if val & 0x80 != 0 {
                    self.shift = 0x10;
                    self.prg_mode = 3;
                } else {
                    let shift = ((val & 1) << 4) | self.shift >> 1;

                    println!("shifter after: {:b}", shift);
                    if self.shift & 1 != 0 {
                        match addr & 0x6000 {
                            0 => self.write_control(shift),
                            0x2000 => self.write_chr_0(shift),
                            0x4000 => self.write_chr_1(shift),
                            0x6000 => self.write_prg(shift),
                            _ => unreachable!(),
                        }

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
        match addr {
            0..=0xFFF => self.cartridge.chr[self.chr_1 + addr],
            0x1000..=0x1FFF => self.cartridge.chr[self.chr_2 + (addr & 0xFFF)],
            _ => unreachable!(),
        }
    }

    fn write_chr(&mut self, addr: usize, val: u8) {
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
}
