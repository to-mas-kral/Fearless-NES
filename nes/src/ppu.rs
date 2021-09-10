use serde::{Deserialize, Serialize};

use super::Nes;

/// This pallete maps the PPU output to RGB (24 bits RGB format)
pub static PALETTE: [u8; 192] = [
    84, 84, 84, 0, 30, 116, 8, 16, 144, 48, 0, 136, 68, 0, 100, 92, 0, 48, 84, 4, 0, 60, 24, 0, 32,
    42, 0, 8, 58, 0, 0, 64, 0, 0, 60, 0, 0, 50, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 152, 150, 152, 8,
    76, 196, 48, 50, 236, 92, 30, 228, 136, 20, 176, 160, 20, 100, 152, 34, 32, 120, 60, 0, 84, 90,
    0, 40, 114, 0, 8, 124, 0, 0, 118, 40, 0, 102, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 236, 238, 236,
    76, 154, 236, 120, 124, 236, 176, 98, 236, 228, 84, 236, 236, 88, 180, 236, 106, 100, 212, 136,
    32, 160, 170, 0, 116, 196, 0, 76, 208, 32, 56, 204, 108, 56, 180, 204, 60, 60, 60, 0, 0, 0, 0,
    0, 0, 236, 238, 236, 168, 204, 236, 188, 188, 236, 212, 178, 236, 236, 174, 236, 236, 174, 212,
    236, 180, 176, 228, 196, 144, 204, 210, 120, 180, 222, 120, 168, 226, 144, 152, 226, 180, 160,
    214, 228, 160, 162, 160, 0, 0, 0, 0, 0, 0,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenLow,
    SingleScreenHigh,
    FourScreen,
    Obscure,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct Sprite {
    y: u8,
    x: u8,
    index: u16,
    palette: u8,
    ///O - in front, 1 - behind background
    priority: bool,
    horizontal_flip: bool,
    vertical_flip: bool,
    tile_low: u8,
    tile_high: u8,
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            y: 0,
            x: 0,
            index: 0,
            palette: 0,
            priority: false,
            horizontal_flip: false,
            vertical_flip: false,
            tile_low: 0,
            tile_high: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Ppu {
    pub output_buffer: Vec<u8>,

    pub oam: Vec<u8>,
    secondary_oam: Vec<u8>,
    palettes: Vec<u8>,

    oamdata_buffer: u8,
    sprite_eval_count: u8,
    secondary_oam_addr: u8,
    sprite_in_range: bool,
    sprite_fetch_step: u8,
    sprite_0_added: bool,
    sprite_0_visible: bool,
    oam_copy_done: bool,
    sprite_count: u8,

    sprite_index: u8,
    sprite_buffer: Vec<Sprite>,
    sprite_cache: Vec<bool>,

    vram_addr: usize,
    temp_vram_addr: usize,
    x_fine_scroll: u8,

    pub xpos: u16,
    pub scanline: u16,
    pub cycle_count: u32,
    odd_frame: bool,

    nametable_byte: u8,
    attribute: u8,
    tile_addr: usize,
    tile_lb: u8,
    tile_hb: u8,
    shift_low: u16,
    shift_high: u16,

    ignore_writes: bool,

    ppustatus: u8,
    pub oamaddr: u8,
    write_toggle: bool,
    latch: u8,
    read_buffer: u8,

    nt_base_addr: usize,
    addr_increment: usize,
    sp_pattern_table_addr: usize,
    bg_pattern_table_addr: usize,
    sp_size: u8,

    /// https://wiki.nesdev.com/w/index.php?title=NMI#Operation
    nmi_on_vblank: bool,
    /// By toggling NMI_output (PPUCTRL.7) during vertical blank without reading PPUSTATUS,
    /// a program can cause /NMI to be pulled low multiple times, causing multiple NMIs to be generated.
    prev_nmi: bool,
    suppress_nmi: bool,

    greyscale: bool,
    bg_left_clip: u8,
    sp_left_clip: u8,
    show_bg: bool,
    show_sp: bool,
    rendering_enabled: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl Ppu {
    pub(crate) fn new() -> Ppu {
        let palettes = vec![
            0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00,
            0x04, 0x2C, 0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02,
            0x00, 0x20, 0x2C, 0x08,
        ];

        Ppu {
            output_buffer: vec![0; 256 * 240],

            oam: vec![0; 0x100],
            secondary_oam: vec![0; 0x20],
            palettes,

            oamdata_buffer: 0,
            sprite_eval_count: 0,
            secondary_oam_addr: 0,
            sprite_in_range: false,
            sprite_fetch_step: 0,
            sprite_0_added: false,
            sprite_0_visible: false,
            oam_copy_done: false,
            sprite_count: 0,

            sprite_index: 0,
            sprite_buffer: vec![Sprite::new(); 8],
            sprite_cache: vec![false; 0x101],

            vram_addr: 0,
            temp_vram_addr: 0,
            x_fine_scroll: 0,

            xpos: 0,
            scanline: 0,
            cycle_count: 0,
            odd_frame: false,

            nametable_byte: 0,
            attribute: 0,
            tile_addr: 0,
            tile_lb: 0,
            tile_hb: 0,
            shift_low: 0,
            shift_high: 0,

            ignore_writes: true,

            ppustatus: 0,
            oamaddr: 0,
            write_toggle: false,
            latch: 0,
            read_buffer: 0,

            nt_base_addr: 0x2000,
            addr_increment: 1,
            sp_pattern_table_addr: 0,
            bg_pattern_table_addr: 0,
            sp_size: 8,

            nmi_on_vblank: false,
            prev_nmi: false,
            suppress_nmi: false,

            greyscale: false,
            bg_left_clip: 0,
            sp_left_clip: 0,
            show_bg: false,
            show_sp: false,
            rendering_enabled: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,
        }
    }
}

impl Nes {
    #[inline]
    fn ppu_write(&mut self, mut addr: usize, val: u8) {
        addr &= 0x3FFF;

        self.mapper
            .notify_a12(addr, self.ppu.cycle_count, &mut self.cpu.irq_signal);

        match addr {
            0..=0x1FFF => self.mapper.write_chr(addr, val),
            0x2000..=0x3EFF => self.write_nametable(addr & 0xFFF, val),
            0x3F00..=0x3FFF => self.palette_write(addr, val),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn write_nametable(&mut self, addr: usize, val: u8) {
        match self.mapper.mirroring() {
            Mirroring::Vertical => match addr {
                0..=0x3FF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr + 0x800, val);
                }
                0x400..=0x7FF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr + 0x800, val);
                }
                0x800..=0xBFF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr - 0x400, val);
                }
                0xC00..=0xFFF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr - 0x800, val);
                }
                _ => unreachable!(),
            },
            Mirroring::Horizontal => match addr {
                0..=0x3FF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr + 0x400, val);
                }
                0x400..=0x7FF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr - 0x400, val);
                }
                0x800..=0xBFF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr + 0x400, val);
                }
                0xC00..=0xFFF => {
                    self.mapper.write_nametable(addr, val);
                    self.mapper.write_nametable(addr - 0x400, val);
                }
                _ => unreachable!(),
            },
            Mirroring::SingleScreenLow => match addr {
                0..=0x3FF => self.mapper.write_nametable(addr, val),
                0x400..=0x7FF => self.mapper.write_nametable(addr - 0x400, val),
                0x800..=0xBFF => self.mapper.write_nametable(addr - 0x800, val),
                0xC00..=0xFFF => self.mapper.write_nametable(addr - 0xC00, val),
                _ => unreachable!(),
            },
            Mirroring::SingleScreenHigh => match addr {
                0..=0x3FF => self.mapper.write_nametable(addr + 0x400, val),
                0x400..=0x7FF => self.mapper.write_nametable(addr, val),
                0x800..=0xBFF => self.mapper.write_nametable(addr - 0x400, val),
                0xC00..=0xFFF => self.mapper.write_nametable(addr - 0x800, val),
                _ => unreachable!(),
            },
            Mirroring::FourScreen => self.mapper.write_nametable(addr, val),
            m => unimplemented!("{:?} mirroring is unimplemented", m),
        }
    }

    #[inline]
    fn ppu_read(&mut self, mut addr: usize) -> u8 {
        addr &= 0x3FFF;

        self.mapper
            .notify_a12(addr, self.ppu.cycle_count, &mut self.cpu.irq_signal);

        match addr {
            0..=0x1FFF => self.mapper.read_chr(addr),
            0x2000..=0x3EFF => self.read_nametable(addr & 0xFFF),
            0x3F00..=0x3FFF => self.palette_read(addr),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn read_nametable(&mut self, addr: usize) -> u8 {
        match self.mapper.mirroring() {
            Mirroring::Vertical => self.mapper.read_nametable(addr),
            Mirroring::Horizontal => self.mapper.read_nametable(addr),
            Mirroring::SingleScreenLow => match addr {
                0..=0x3FF => self.mapper.read_nametable(addr),
                0x400..=0x7FF => self.mapper.read_nametable(addr - 0x400),
                0x800..=0xBFF => self.mapper.read_nametable(addr - 0x800),
                0xC00..=0xFFF => self.mapper.read_nametable(addr - 0xC00),
                _ => unreachable!(),
            },
            Mirroring::SingleScreenHigh => match addr {
                0..=0x3FF => self.mapper.read_nametable(addr + 0x400),
                0x400..=0x7FF => self.mapper.read_nametable(addr),
                0x800..=0xBFF => self.mapper.read_nametable(addr - 0x400),
                0xC00..=0xFFF => self.mapper.read_nametable(addr - 0x800),
                _ => unreachable!(),
            },
            Mirroring::FourScreen => self.mapper.read_nametable(addr),
            m => unimplemented!("{:?} mirroring is unimplemented", m),
        }
    }

    #[inline]
    fn palette_write(&mut self, mut addr: usize, mut val: u8) {
        addr &= 0x1F;
        val &= 0x3F;

        match addr {
            0x0 | 0x10 => {
                self.ppu.palettes[0] = val;
                self.ppu.palettes[0x10] = val;
            }
            0x4 | 0x14 => {
                self.ppu.palettes[0x4] = val;
                self.ppu.palettes[0x14] = val;
            }
            0x8 | 0x18 => {
                self.ppu.palettes[0x8] = val;
                self.ppu.palettes[0x18] = val;
            }
            0xC | 0x1C => {
                self.ppu.palettes[0xC] = val;
                self.ppu.palettes[0x1C] = val;
            }
            _ => self.ppu.palettes[addr] = val,
        }
    }

    #[inline]
    fn palette_read(&mut self, mut addr: usize) -> u8 {
        addr &= 0x1F;
        if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
            addr &= !0x10;
        }

        let mut index = self.ppu.palettes[addr];

        if self.ppu.greyscale {
            index &= 0x30;
        }

        index
    }

    //TODO: latch decay ?
    //TODO: open bus masks
    #[inline]
    pub(crate) fn ppu_read_reg(&mut self, addr: usize) -> u8 {
        match addr & 7 {
            0 | 1 | 3 | 5 | 6 => (),
            2 => self.read_ppustatus(),
            4 => self.read_oamdata(),
            7 => self.read_ppudata(),
            _ => unreachable!(),
        };

        self.ppu.latch
    }

    #[inline]
    pub(crate) fn ppu_write_reg(&mut self, addr: usize, val: u8) {
        self.ppu.latch = val;
        match addr & 7 {
            0 if !self.ppu.ignore_writes => self.write_ppuctrl(),
            1 if !self.ppu.ignore_writes => self.write_ppumask(),
            3 => self.write_oamaddr(),
            4 => self.write_oamdata(),
            5 if !self.ppu.ignore_writes => self.write_ppuscroll(),
            6 if !self.ppu.ignore_writes => self.write_ppuaddr(),
            7 => self.write_ppudata(),
            _ => (),
        }
    }

    #[inline]
    pub(crate) fn ppu_enable_writes(&mut self) {
        self.ppu.ignore_writes = false;
    }

    /** Ppuctrl
    N -- 00000011 -- Name table address (0 = 0x2000; 1 = 0x2400; 2 = 0x2800; 3 = 0x2C00)
    I -- 00000100 -- PPU address increment (0: add 1, going across; 1: add 32, going down)
    S -- 00001000 -- Sprite pattern table address (0: 0x0000; 1: 0x1000; ignored in 8x16 mode)
    B -- 00010000 -- Background pattern table address (0: 0x0000; 1: 0x1000)
    H -- 00100000 -- Sprite size (0: 8x8, 1: 8x16)
    P -- 01000000 -- PPU master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)
    V -- 10000000 -- Execute NMI on vblank **/
    #[inline]
    fn write_ppuctrl(&mut self) {
        //TODO: bit 0 bus conflict
        let val = self.ppu.latch;
        self.ppu.temp_vram_addr &= !0xC00;
        self.ppu.temp_vram_addr |= ((val as usize) & 3) << 10;

        self.ppu.nt_base_addr = match val & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        };
        self.ppu.addr_increment = if val & (1 << 2) == 0 { 1 } else { 32 };
        self.ppu.sp_pattern_table_addr = if val & (1 << 3) == 0 { 0 } else { 0x1000 };
        self.ppu.bg_pattern_table_addr = if val & (1 << 4) == 0 { 0 } else { 0x1000 };
        self.ppu.sp_size = if val & (1 << 5) == 0 { 8 } else { 16 };

        self.ppu.nmi_on_vblank = val & (1 << 7) != 0;
    }

    /** Ppumask
    g -- 00000001 -- Greyscale (0: normal color, 1: produce a greyscale display)
    m -- 00000010 -- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    M -- 00000100 -- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    b -- 00001000 -- 1: Show background, 0: Hide
    s -- 00010000 -- 1: Show sprites, 0: Hide
    R -- 00100000 -- Emphasize red
    G -- 01000000 -- Emphasize green
    B -- 10000000 -- Emphasize blue **/
    #[inline]
    fn write_ppumask(&mut self) {
        let val = self.ppu.latch;
        self.ppu.greyscale = val & 1 != 0;
        self.ppu.bg_left_clip = if val & (1 << 1) != 0 { 0 } else { 8 };
        self.ppu.sp_left_clip = if val & (1 << 2) != 0 { 0 } else { 8 };
        self.ppu.show_bg = val & (1 << 3) != 0;
        self.ppu.show_sp = val & (1 << 4) != 0;
        self.ppu.emphasize_red = val & (1 << 5) != 0;
        self.ppu.emphasize_green = val & (1 << 6) != 0;
        self.ppu.emphasize_blue = val & (1 << 7) != 0;
        self.ppu.rendering_enabled = self.ppu.show_bg || self.ppu.show_sp;
    }

    /** Ppustatus
    O -- 00100000 -- Sprite overflow
    S -- 01000000 -- Sprite 0 hit
    V -- 10000000 -- Vertical blank has started (0: not in vblank; 1: in vblank) **/
    #[inline]
    fn read_ppustatus(&mut self) {
        self.ppu.write_toggle = false;
        self.ppu.latch = self.ppu.ppustatus;
        self.ppu.ppustatus &= 0x7F;

        // https://wiki.nesdev.com/w/index.php?title=PPU_frame_timing#VBL_Flag_Timing
        // Reading $2002 within a few PPU clocks of when VBL is set results in special-case behavior.
        // Reading one PPU clock before reads it as clear and never sets the flag or generates
        // NMI for that frame. Reading on the same PPU clock or one later reads it as set,
        // clears it, and suppresses the NMI for that frame.

        if self.ppu.scanline == 241 && (self.ppu.xpos == 2 || self.ppu.xpos == 3) {
            self.ppu.latch |= 0x80;
            self.ppu.suppress_nmi = true;
            self.cpu.nmi_signal = false;
        } else if self.ppu.scanline == 241 && self.ppu.xpos == 1 {
            self.ppu.latch &= 0x7F;
            self.ppu.suppress_nmi = true;
        }
    }

    #[inline]
    fn write_oamaddr(&mut self) {
        self.ppu.oamaddr = self.ppu.latch;
    }

    #[inline]
    fn read_oamdata(&mut self) {
        if self.ppu.scanline <= 239 && self.ppu.rendering_enabled {
            self.ppu.latch = self.ppu.oamdata_buffer;
        } else {
            self.ppu.latch = self.ppu.oam[self.ppu.oamaddr as usize];
        }
    }

    #[inline]
    fn write_oamdata(&mut self) {
        if self.ppu.rendering_enabled && (self.ppu.scanline <= 239 || self.ppu.scanline == 261) {
            self.ppu.oamaddr = self.ppu.oamaddr.wrapping_add(4);
        } else {
            let val = if self.ppu.oamaddr & 3 == 2 {
                self.ppu.latch & 0xE3
            } else {
                self.ppu.latch
            };

            self.ppu.oam[self.ppu.oamaddr as usize] = val;
            self.ppu.oamaddr = self.ppu.oamaddr.wrapping_add(1);
        }
    }

    #[inline]
    fn write_ppuscroll(&mut self) {
        let val = self.ppu.latch;
        if self.ppu.write_toggle {
            self.ppu.temp_vram_addr = (self.ppu.temp_vram_addr & !0x73E0)
                | ((val as usize & 0xF8) << 2)
                | ((val as usize & 7) << 12);
        } else {
            self.ppu.temp_vram_addr = (self.ppu.temp_vram_addr & !0x1F) | (val as usize >> 3);
            self.ppu.x_fine_scroll = val & 7;
        }

        self.ppu.write_toggle = !self.ppu.write_toggle;
    }

    #[inline]
    fn write_ppuaddr(&mut self) {
        let val = self.ppu.latch;

        if self.ppu.write_toggle {
            self.ppu.temp_vram_addr = (self.ppu.temp_vram_addr & !0xFF) | val as usize;
            //TODO: add 2-3 cycle delay to the update
            self.ppu.vram_addr = self.ppu.temp_vram_addr;

            self.mapper.notify_a12(
                self.ppu.vram_addr,
                self.ppu.cycle_count,
                &mut self.cpu.irq_signal,
            );
        } else {
            self.ppu.temp_vram_addr =
                (self.ppu.temp_vram_addr & !0xFF00) | ((val as usize & 0x3F) << 8);
        }

        self.ppu.write_toggle = !self.ppu.write_toggle;
    }

    #[inline]
    fn read_ppudata(&mut self) {
        self.ppu.latch = self.ppu.read_buffer;
        self.ppu.read_buffer = self.ppu_read(self.ppu.vram_addr);

        if (self.ppu.vram_addr & 0x3FFF) >= 0x3F00 {
            self.ppu.latch = self.palette_read(self.ppu.vram_addr);
            self.ppu.read_buffer = self
                .mapper
                .read_nametable((self.ppu.vram_addr & 0x3FFF) - 0x3000);
        }

        if self.ppu.rendering_enabled && (self.ppu.scanline < 240 || self.ppu.scanline == 261) {
            self.coarse_x_increment();
            self.y_increment();
        } else {
            self.ppu.vram_addr += self.ppu.addr_increment;
        }

        self.mapper.notify_a12(
            self.ppu.vram_addr,
            self.ppu.cycle_count,
            &mut self.cpu.irq_signal,
        );
    }

    #[inline]
    fn write_ppudata(&mut self) {
        self.ppu_write(self.ppu.vram_addr, self.ppu.latch);

        if self.ppu.rendering_enabled && (self.ppu.scanline < 240 || self.ppu.scanline == 261) {
            self.coarse_x_increment();
            self.y_increment();
        } else {
            self.ppu.vram_addr += self.ppu.addr_increment;
        };

        self.mapper.notify_a12(
            self.ppu.vram_addr,
            self.ppu.cycle_count,
            &mut self.cpu.irq_signal,
        );
    }

    #[inline]
    fn attr_table_addr(&self) -> usize {
        0x23C0
            | (self.ppu.vram_addr & 0xC00)
            | ((self.ppu.vram_addr >> 4) & 0x38)
            | ((self.ppu.vram_addr >> 2) & 7)
    }

    #[inline]
    fn nametable_addr(&self) -> usize {
        0x2000 | (self.ppu.vram_addr & 0xFFF)
    }

    #[inline]
    pub(crate) fn ppu_tick(&mut self) {
        self.ppu_scanline_tick();

        self.ppu.xpos += 1;
        if self.ppu.xpos > 340 {
            self.ppu.xpos = 0;
            self.ppu.scanline += 1;

            if self.ppu.scanline > 261 {
                self.ppu.scanline = 0;
            }
        }

        self.ppu.cycle_count = self.ppu.cycle_count.wrapping_add(1);
    }

    /// https://wiki.nesdev.com/w/index.php/PPU_rendering
    /// https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
    #[inline]
    fn ppu_scanline_tick(&mut self) {
        match self.ppu.scanline {
            // TODO: correct operation when rendering is disabled
            0..=239 => match self.ppu.xpos {
                2..=256 => {
                    self.shift_tile_registers();
                    self.fetch_bg();

                    if self.ppu.xpos >= 65 {
                        self.sprite_evaluation();

                        if self.ppu.xpos == 256 {
                            self.y_increment();
                        }
                    }

                    self.draw_pixel();
                }
                321..=336 => {
                    self.fetch_bg();
                    self.shift_tile_registers();
                }
                258..=320 => self.fetch_sprites(),
                0 => {
                    // Uncompleted tile fetch needed for proper MMC3 emulation
                    let addr = (usize::from(self.ppu.nametable_byte) << 4)
                        | (self.ppu.vram_addr >> 12)
                        | self.ppu.bg_pattern_table_addr;
                    self.mapper
                        .notify_a12(addr, self.ppu.cycle_count, &mut self.cpu.irq_signal);
                }
                1 => {
                    self.fetch_nt();
                    self.draw_pixel();
                    self.ppu.secondary_oam.fill(0xFF);
                }
                257 => {
                    self.t_to_v();
                    self.shift_tile_registers();
                    self.fetch_sprites();
                }
                337 | 339 => {
                    self.ppu_read(self.nametable_addr());
                }
                _ => (),
            },
            240 => {
                if self.ppu.xpos == 1 {
                    self.mapper.notify_a12(
                        self.ppu.vram_addr,
                        self.ppu.cycle_count,
                        &mut self.cpu.irq_signal,
                    );
                }
            }
            241..=260 => self.vblank(),
            261 => {
                match self.ppu.xpos {
                    2..=256 => {
                        self.fetch_bg();
                        if self.ppu.xpos < 9 {
                            self.oam_refresh_bug();
                        }
                        if self.ppu.xpos == 256 {
                            self.y_increment();
                        }
                    }
                    280..=304 => {
                        self.v_from_t();
                        self.fetch_sprites();
                    }
                    258..=279 => self.fetch_sprites(),
                    305..=320 => self.fetch_sprites(),
                    321..=336 => {
                        self.fetch_bg();
                        self.shift_tile_registers();
                    }
                    0 => {
                        // Uncompleted tile fetch needed for proper MMC3 emulation
                        let addr = (usize::from(self.ppu.nametable_byte) << 4)
                            | (self.ppu.vram_addr >> 12)
                            | self.ppu.bg_pattern_table_addr;
                        self.mapper.notify_a12(
                            addr,
                            self.ppu.cycle_count,
                            &mut self.cpu.irq_signal,
                        );
                    }
                    1 => {
                        self.ppu.ppustatus &= !0xE0;
                        self.cpu.nmi_signal = false;
                        self.ppu.suppress_nmi = false;
                        self.ppu.prev_nmi = false;
                        self.fetch_nt();
                        self.oam_refresh_bug();
                    }
                    257 => {
                        self.t_to_v();
                        self.fetch_sprites();
                    }
                    337 => {
                        self.ppu_read(self.nametable_addr());
                    }
                    339 => {
                        self.ppu.sprite_cache.fill(false);
                        self.frame_ready = true;
                        self.ppu_read(self.nametable_addr());

                        // The skipped tick is implemented by jumping directly from (339, 261)
                        // to (0, 0), meaning the last tick of the last NT fetch takes place at (0, 0)
                        // on odd frames replacing the idle tick
                        if self.ppu.odd_frame & self.ppu.rendering_enabled {
                            self.ppu.xpos = 340;
                        }

                        self.ppu.odd_frame = !self.ppu.odd_frame;
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    #[inline]
    fn vblank(&mut self) {
        match (self.ppu.scanline, self.ppu.xpos) {
            (241, 1) => {
                if !self.ppu.suppress_nmi {
                    self.ppu.ppustatus |= 0x80;
                }

                if self.ppu.nmi_on_vblank && (self.ppu.ppustatus & 0x80) != 0 {
                    self.ppu.prev_nmi = true;
                    self.cpu.nmi_signal = true;
                }
            }
            (241, 0) => (),
            _ => {
                let current_nmi = self.ppu.nmi_on_vblank
                    && ((self.ppu.ppustatus & 0x80) != 0)
                    && !self.ppu.suppress_nmi;

                match (self.ppu.prev_nmi, current_nmi) {
                    (false, true) => self.cpu.nmi_signal = true,
                    _ => (),
                }

                self.ppu.prev_nmi = current_nmi;
            }
        }
    }

    /** Tile and attribute fetching
    http://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching

    The high bits of v are used for fine Y during rendering,
    and addressing nametable data only requires 12 bits,
    with the high 2 CHR addres lines fixed to the 0x2000 region.
    The address to be fetched during rendering can be deduced from v in the following way:

    nametable address = 0x2000 | (v & 0x0FFF)
    attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    tile address low  = (nametable address << 4) | (v >> 12) | background patter table address
    tile address high = tile address low + 8

    The low 12 bits of the attribute address are composed in the following way:

    NN 1111 YYY XXX
    || |||| ||| +++-- high 3 bits of coarse X (x/4)
    || |||| +++------ high 3 bits of coarse Y (y/4)
    || ++++---------- attribute offset (960 bytes)
    ++--------------- nametable select

    The 15 bit registers t and v are composed this way during rendering:
    yyy NN YYYYY XXXXX
    ||| || ||||| +++++-- coarse X scroll
    ||| || +++++-------- coarse Y scroll
    ||| ++-------------- nametable select
    +++----------------- fine Y scroll **/
    #[inline(always)]
    fn fetch_bg(&mut self) {
        if self.ppu.rendering_enabled {
            match self.ppu.xpos & 7 {
                0 => self.coarse_x_increment(),
                1 => self.fetch_nt(),
                3 => self.fetch_at(),
                5 => self.fetch_bg_low(),
                7 => self.fetch_bg_high(),
                _ => (),
            }
        }
    }

    #[inline(always)]
    fn fetch_nt(&mut self) {
        // The 2-bit 1-of-4 selector" is used to shift the attribute byte right
        // by 0, 2, 4, or 6 bits depending on bit 4 of the X and Y pixel position.
        // Roughly: if (v & 0x40) attrbyte >>= 4; if (v & 0x02) attrbyte >>= 2.
        if self.ppu.rendering_enabled {
            self.shift_attrbutes();
            self.ppu.shift_low |= u16::from(self.ppu.tile_lb);
            self.ppu.shift_high |= u16::from(self.ppu.tile_hb);

            self.ppu.nametable_byte = self.ppu_read(self.nametable_addr());
        }
    }

    #[inline]
    fn fetch_at(&mut self) {
        let shift = ((self.ppu.vram_addr >> 4) & 4) | (self.ppu.vram_addr & 2);
        let attr = (self.ppu_read(self.attr_table_addr()) >> shift) & 3;
        self.ppu.attribute |= attr << 6;
    }

    #[inline]
    fn fetch_bg_low(&mut self) {
        self.ppu.tile_addr = (usize::from(self.ppu.nametable_byte) << 4)
            | (self.ppu.vram_addr >> 12)
            | self.ppu.bg_pattern_table_addr;
        self.ppu.tile_lb = self.ppu_read(self.ppu.tile_addr);
    }

    #[inline]
    fn fetch_bg_high(&mut self) {
        self.ppu.tile_hb = self.ppu_read(self.ppu.tile_addr + 8);
    }

    #[inline]
    fn fetch_sprites(&mut self) {
        if self.ppu.xpos == 257 {
            self.ppu.sprite_cache.fill(false);
            self.ppu.sprite_index = 0;
        }

        if self.ppu.rendering_enabled {
            self.ppu.oamaddr = 0;
            match (self.ppu.xpos - 1) & 7 {
                0 => {
                    self.ppu_read(self.nametable_addr());
                }
                2 => {
                    self.ppu_read(self.attr_table_addr());
                }
                3 => self.load_sprite(),
                _ => (),
            };
        }
    }

    #[inline]
    fn load_sprite(&mut self) {
        let sprite_addr = 4 * self.ppu.sprite_index as usize;
        let sprite = &mut self.ppu.sprite_buffer[self.ppu.sprite_index as usize];
        sprite.y = self.ppu.secondary_oam[sprite_addr];
        sprite.x = self.ppu.secondary_oam[sprite_addr + 3];

        for i in 1..9 {
            if sprite.x as usize + i < 257 {
                self.ppu.sprite_cache[sprite.x as usize + i] = true;
            }
        }

        let attributes = self.ppu.secondary_oam[sprite_addr + 2];
        sprite.vertical_flip = attributes & 0x80 != 0;
        sprite.horizontal_flip = attributes & 0x40 != 0;
        sprite.priority = attributes & 0x20 != 0;
        sprite.palette = ((attributes & 3) << 2) | 0x10;

        let scanline = if self.ppu.scanline == 261 {
            -1
        } else {
            self.ppu.scanline as i16
        };

        let mut y_offset = if sprite.vertical_flip {
            (self.ppu.sp_size - 1) as i16 - (scanline - sprite.y as i16)
        } else {
            scanline - sprite.y as i16
        };

        let index = self.ppu.secondary_oam[sprite_addr + 1];
        sprite.index = if self.ppu.sp_size == 8 {
            (self.ppu.sp_pattern_table_addr as u16 | (u16::from(index) << 4))
                .wrapping_add(y_offset as u16)
        } else {
            if y_offset >= 8 {
                y_offset += 8;
            }

            let pattern_table_addr = if index & 1 != 0 { 0x1000 } else { 0 };
            pattern_table_addr | (u16::from(index & !1) << 4).wrapping_add(y_offset as u16)
        };

        let index = sprite.index as usize;
        self.ppu.sprite_buffer[self.ppu.sprite_index as usize].tile_low = self.ppu_read(index);
        // The second read is performed 2 cycles later, but it shouldn't have any efect
        self.ppu.sprite_buffer[self.ppu.sprite_index as usize].tile_high = self.ppu_read(index + 8);

        self.ppu.sprite_index = (self.ppu.sprite_index + 1) & 7;
    }

    /// http://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
    #[inline]
    fn sprite_evaluation(&mut self) {
        if !self.ppu.rendering_enabled {
            return;
        }

        if self.ppu.xpos == 65 {
            self.ppu.sprite_eval_count = 0;
            self.ppu.oamdata_buffer = 0;
            self.ppu.sprite_fetch_step = 0;
            self.ppu.secondary_oam_addr = 0;
            self.ppu.sprite_0_added = false;
            self.ppu.sprite_in_range = false;
            self.ppu.oam_copy_done = false;
        } else if self.ppu.xpos == 256 {
            self.ppu.sprite_0_visible = self.ppu.sprite_0_added;
            self.ppu.sprite_count = self.ppu.secondary_oam_addr >> 2;
        }

        if self.ppu.xpos & 1 != 0 {
            self.ppu.oamdata_buffer = self.ppu.oam[self.ppu.oamaddr as usize];
        } else {
            if self.ppu.oam_copy_done {
                self.ppu.sprite_eval_count = (self.ppu.sprite_eval_count + 1) & 0x3F;
                if self.ppu.secondary_oam_addr >= 0x20 {
                    self.ppu.oamdata_buffer =
                        self.ppu.secondary_oam[self.ppu.secondary_oam_addr as usize & 0x1F];
                }
            } else {
                //1. Starting at n = 0, read a sprite's Y-coordinate (OAM[n][0], copying it
                //to the next open slot in secondary OAM (unless 8 sprites have been
                //found, in which case the write is ignored).

                if !self.ppu.sprite_in_range
                    && (self.ppu.scanline >= self.ppu.oamdata_buffer as u16)
                    && (self.ppu.scanline
                        < (self.ppu.oamdata_buffer as u16 + self.ppu.sp_size as u16))
                {
                    self.ppu.sprite_in_range = true;
                }

                if self.ppu.secondary_oam_addr < 0x20 {
                    self.ppu.secondary_oam[self.ppu.secondary_oam_addr as usize] =
                        self.ppu.oamdata_buffer;

                    if self.ppu.sprite_in_range {
                        //1a. If Y-coordinate is in range, copy remaining bytes of sprite
                        //data (OAM[n][1] thru OAM[n][3]) into secondary OAM.
                        self.ppu.secondary_oam_addr += 1;
                        self.ppu.sprite_fetch_step += 1;

                        if self.ppu.sprite_eval_count == 0 {
                            self.ppu.sprite_0_added = true;
                        }

                        if self.ppu.sprite_fetch_step == 4 {
                            self.ppu.sprite_in_range = false;
                            self.ppu.sprite_fetch_step = 0;
                            self.ppu.sprite_eval_count = (self.ppu.sprite_eval_count + 1) & 0x3F;
                            if self.ppu.sprite_eval_count == 0 {
                                self.ppu.oam_copy_done = true;
                            }
                        }
                    } else {
                        self.ppu.sprite_eval_count = (self.ppu.sprite_eval_count + 1) & 0x3F;
                        if self.ppu.sprite_eval_count == 0 {
                            self.ppu.oam_copy_done = true;
                        }
                    }
                } else {
                    self.ppu.oamdata_buffer =
                        self.ppu.secondary_oam[self.ppu.secondary_oam_addr as usize & 0x1F];

                    if self.ppu.sprite_in_range {
                        self.ppu.ppustatus |= 0x20;
                        self.ppu.sprite_fetch_step += 1;
                        if self.ppu.sprite_fetch_step == 4 {
                            self.ppu.sprite_eval_count = (self.ppu.sprite_eval_count + 1) & 0x3F;
                            self.ppu.sprite_fetch_step = 0;
                        }
                    } else {
                        self.ppu.sprite_eval_count = (self.ppu.sprite_eval_count + 1) & 0x3F;
                        self.ppu.sprite_fetch_step = (self.ppu.sprite_fetch_step + 1) & 3;

                        if self.ppu.sprite_eval_count == 0 {
                            self.ppu.oam_copy_done = true;
                        }
                    }
                }
            }
            self.ppu.oamaddr = 4 * self.ppu.sprite_eval_count + self.ppu.sprite_fetch_step;
        }
    }

    /// Taken from: http://wiki.nesdev.com/w/index.php/PPU_scrolling
    #[inline]
    fn y_increment(&mut self) {
        if self.ppu.rendering_enabled {
            if (self.ppu.vram_addr & 0x7000) != 0x7000 {
                self.ppu.vram_addr += 0x1000;
            } else {
                self.ppu.vram_addr &= !0x7000;
                let mut y = (self.ppu.vram_addr & 0x3E0) >> 5;
                if y == 29 {
                    y = 0;
                    self.ppu.vram_addr ^= 0x800;
                } else if y == 31 {
                    y = 0;
                } else {
                    y += 1;
                }

                self.ppu.vram_addr = (self.ppu.vram_addr & !0x3E0) | (y << 5);
            }
        }
    }

    /// Taken from: http://wiki.nesdev.com/w/index.php/PPU_scrolling
    #[inline]
    fn coarse_x_increment(&mut self) {
        if (self.ppu.vram_addr & 0x1F) == 31 {
            self.ppu.vram_addr &= !0x1F;
            self.ppu.vram_addr ^= 0x400
        } else {
            self.ppu.vram_addr += 1;
        }
    }

    /// At dot 257 of each scanline
    /// If rendering is enabled, the PPU copies all bits related to horizontal position from t to v:
    /// v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
    #[inline]
    fn t_to_v(&mut self) {
        if self.ppu.rendering_enabled {
            self.ppu.vram_addr = (self.ppu.vram_addr & !0x41F) | (self.ppu.temp_vram_addr & 0x41F);
        }
    }

    /// During dots 280 to 304 of the pre-render scanline (end of vblank)
    /// If rendering is enabled, at the end of vblank, shortly after the horizontal
    /// bits are copied from t to v at dot 257, the PPU will repeatedly copy the
    /// vertical bits from t to v from dots 280 to 304, completing the full initialization of v from t:
    /// v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
    #[inline]
    fn v_from_t(&mut self) {
        if self.ppu.rendering_enabled {
            self.ppu.vram_addr =
                (self.ppu.vram_addr & !0x7BE0) | (self.ppu.temp_vram_addr & 0x7BE0);
        }
    }

    /// If the sprite address (OAMADDR, $2003) is not zero at the beginning of
    /// the pre-render scanline,on the 2C02 an OAM hardware refresh bug will cause the
    /// first 8 bytes of OAM to be overwritten by the 8 bytes beginning at OAMADDR & $F8
    /// before sprite evaluation begins.
    #[inline]
    fn oam_refresh_bug(&mut self) {
        if self.ppu.oamaddr >= 8 && self.ppu.rendering_enabled {
            self.ppu.oam[self.ppu.xpos as usize - 1] = self.ppu.oam
                [(self.ppu.oamaddr as usize & 0xF8).wrapping_add(self.ppu.xpos as usize - 1)];
        }
    }

    #[inline]
    fn shift_tile_registers(&mut self) {
        self.ppu.shift_low <<= 1;
        self.ppu.shift_high <<= 1;
    }

    #[inline]
    fn shift_attrbutes(&mut self) {
        self.ppu.attribute >>= 2;
    }

    #[inline(always)]
    fn draw_pixel(&mut self) {
        let addr = (usize::from(self.ppu.scanline) << 8) + usize::from(self.ppu.xpos - 1);
        let color_index = self.pixel_color();
        self.ppu.output_buffer[addr] = self.ppu.palettes[color_index];
    }

    #[inline(always)]
    fn pixel_color(&mut self) -> usize {
        if !self.ppu.rendering_enabled && (self.ppu.vram_addr & 0x3F00) == 0x3F00 {
            return self.ppu.vram_addr & 0x1F;
        }

        let bg_index = if self.ppu.show_bg && self.ppu.xpos > self.ppu.bg_left_clip as u16 {
            let tile_h_bit =
                ((self.ppu.shift_high << u16::from(self.ppu.x_fine_scroll)) & 0x8000) >> 14;
            let tile_l_bit =
                ((self.ppu.shift_low << u16::from(self.ppu.x_fine_scroll)) & 0x8000) >> 15;
            let attribute = if self.ppu.x_fine_scroll as u16 + ((self.ppu.xpos - 1) & 7) < 8 {
                self.ppu.attribute & 0xC
            } else {
                (self.ppu.attribute & 0x30) >> 2
            };
            (attribute as u16 | tile_h_bit | tile_l_bit) as usize
        } else {
            0
        };

        if self.ppu.sprite_cache[self.ppu.xpos as usize]
            && self.ppu.scanline != 1
            && self.ppu.show_sp
            && self.ppu.xpos > self.ppu.sp_left_clip as u16
        {
            for i in 0..self.ppu.sprite_count {
                let spr = &mut self.ppu.sprite_buffer[i as usize];
                let shift = self.ppu.xpos as i32 - spr.x as i32 - 1;
                if shift >= 0 && shift <= 7 {
                    let sp_color = if spr.horizontal_flip {
                        ((spr.tile_low >> shift) & 1) | (((spr.tile_high >> shift) & 1) << 1)
                    } else {
                        (((spr.tile_low << shift) & 0x80) >> 7)
                            | (((spr.tile_high << shift) & 0x80) >> 6)
                    };

                    if sp_color != 0 {
                        if self.ppu.sprite_0_visible
                            && i == 0
                            && bg_index != 0
                            && self.ppu.xpos != 256
                            && self.ppu.ppustatus & 0x40 == 0
                        {
                            self.ppu.ppustatus |= 0x40;
                        }

                        if bg_index == 0 || !spr.priority {
                            return usize::from(sp_color | spr.palette);
                        }
                    };
                }
            }
        }

        if bg_index & 3 > 0 {
            bg_index
        } else {
            0
        }
    }
}
