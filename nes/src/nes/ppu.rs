use std::cell::RefCell;
use std::rc::Rc;

use super::mapper::Mapper;
use super::Frame;
use super::InterruptBus;

static PALETTE: [(u8, u8, u8); 64] = [
    (0x7C, 0x7C, 0x7C),
    (0x00, 0x00, 0xFC),
    (0x00, 0x00, 0xBC),
    (0x44, 0x28, 0xBC),
    (0x94, 0x00, 0x84),
    (0xA8, 0x00, 0x20),
    (0xA8, 0x10, 0x00),
    (0x88, 0x14, 0x00),
    (0x50, 0x30, 0x00),
    (0x00, 0x78, 0x00),
    (0x00, 0x68, 0x00),
    (0x00, 0x58, 0x00),
    (0x00, 0x40, 0x58),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xBC, 0xBC, 0xBC),
    (0x00, 0x78, 0xF8),
    (0x00, 0x58, 0xF8),
    (0x68, 0x44, 0xFC),
    (0xD8, 0x00, 0xCC),
    (0xE4, 0x00, 0x58),
    (0xF8, 0x38, 0x00),
    (0xE4, 0x5C, 0x10),
    (0xAC, 0x7C, 0x00),
    (0x00, 0xB8, 0x00),
    (0x00, 0xA8, 0x00),
    (0x00, 0xA8, 0x44),
    (0x00, 0x88, 0x88),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xF8, 0xF8, 0xF8),
    (0x3C, 0xBC, 0xFC),
    (0x68, 0x88, 0xFC),
    (0x98, 0x78, 0xF8),
    (0xF8, 0x78, 0xF8),
    (0xF8, 0x58, 0x98),
    (0xF8, 0x78, 0x58),
    (0xFC, 0xA0, 0x44),
    (0xF8, 0xB8, 0x00),
    (0xB8, 0xF8, 0x18),
    (0x58, 0xD8, 0x54),
    (0x58, 0xF8, 0x98),
    (0x00, 0xE8, 0xD8),
    (0x78, 0x78, 0x78),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xFC, 0xFC, 0xFC),
    (0xA4, 0xE4, 0xFC),
    (0xB8, 0xB8, 0xF8),
    (0xD8, 0xB8, 0xF8),
    (0xF8, 0xB8, 0xF8),
    (0xF8, 0xA4, 0xC0),
    (0xF0, 0xD0, 0xB0),
    (0xFC, 0xE0, 0xA8),
    (0xF8, 0xD8, 0x78),
    (0xD8, 0xF8, 0x78),
    (0xB8, 0xF8, 0xB8),
    (0xB8, 0xF8, 0xD8),
    (0x00, 0xFC, 0xFC),
    (0xF8, 0xD8, 0xF8),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
];

enum RenderState {
    PreRender,
    Render,
    PostRender,
    VBlank,
}

#[derive(Clone, Copy)]
struct Sprite {
    y: u8,
    x: u8,
    index: u16,
    palette: u8,
    priority: bool, //O - in front, 1 - behind background
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

pub struct Ppu {
    output_buffer: [u8; 256 * 240],
    frame: Rc<RefCell<Frame>>,

    interrupt_bus: Rc<RefCell<InterruptBus>>,
    prev_nmi: bool,

    pub oam: [u8; 0x100],
    secondary_oam: [u8; 0x20],
    palettes: [u8; 0x20],

    oamdata_buffer: u8,
    sprite_eval_count: u8,
    sprite_eval_state: u8,
    sprite_fetch_step: u8,
    sprite_in_range: bool,
    sprites_found: u8,

    sprite_index: u8,
    sprite_buffer: [Sprite; 8],

    mapper: Rc<RefCell<Box<Mapper>>>,

    vram_addr: usize,
    temp_vram_addr: usize,
    x_fine_scroll: u8,

    xpos: u16,
    scanline: u16,
    odd_frame: bool,

    nametable_byte: u8,
    attr_table_byte: u8,
    next_attr_table_byte: u8,
    tile_addr: usize,
    tile_lb: u8,
    tile_hb: u8,
    shift_low: u16,
    shift_high: u16,

    ignore_writes: bool,
    suppress_vbl: bool,

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
    nmi_on_vblank: bool,

    greyscale: bool,
    bg_leftmost_8: bool,
    sp_leftmost_8: bool,
    show_bg: bool,
    show_sp: bool,
    rendering_enabled: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl Ppu {
    pub fn new(
        interrupt_bus: Rc<RefCell<InterruptBus>>,
        mapper: Rc<RefCell<Box<Mapper>>>,
        frame: Rc<RefCell<Frame>>,
    ) -> Ppu {
        let palettes = [
            0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00,
            0x04, 0x2C, 0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02,
            0x00, 0x20, 0x2C, 0x08,
        ];

        Ppu {
            output_buffer: [0; 256 * 240],
            frame,

            interrupt_bus,
            prev_nmi: false,

            xpos: 0,
            scanline: 0,
            odd_frame: false,

            oam: [0; 0x100],
            secondary_oam: [0; 0x20],
            palettes,

            oamdata_buffer: 0,
            sprite_eval_count: 0,
            sprite_eval_state: 1,
            sprite_fetch_step: 0,
            sprite_in_range: false,
            sprites_found: 0,

            sprite_index: 0,
            sprite_buffer: [Sprite::new(); 8],

            mapper,

            nametable_byte: 0,
            attr_table_byte: 0,
            next_attr_table_byte: 0,
            tile_addr: 0,
            tile_lb: 0,
            tile_hb: 0,
            shift_low: 0,
            shift_high: 0,

            vram_addr: 0,
            temp_vram_addr: 0,
            x_fine_scroll: 0,

            ignore_writes: true,
            suppress_vbl: false,

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

            greyscale: false,
            bg_leftmost_8: false,
            sp_leftmost_8: false,
            show_bg: false,
            show_sp: false,
            rendering_enabled: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,
        }
    }

    //TODO: latch decay ?
    //TODO: open bus masks
    #[inline]
    pub fn read_reg(&mut self, addr: usize) -> u8 {
        match addr & 7 {
            0 | 1 | 3 | 5 | 6 => (),
            2 => self.read_ppustatus(),
            4 => self.read_oamdata(),
            7 => self.read_ppudata(),
            _ => unreachable!(),
        };

        self.latch
    }

    #[inline]
    pub fn write_reg(&mut self, addr: usize, val: u8) {
        self.latch = val;
        match addr & 7 {
            0 if !self.ignore_writes => self.write_ppuctrl(),
            1 if !self.ignore_writes => self.write_ppumask(),
            3 => self.write_oamaddr(),
            4 => self.write_oamdata(),
            5 if !self.ignore_writes => self.write_ppuscroll(),
            6 if !self.ignore_writes => self.write_ppuaddr(),
            7 => self.write_ppudata(),
            _ => (),
        }
    }

    #[inline]
    pub fn enable_writes(&mut self) {
        self.ignore_writes = false;
    }

    #[inline]
    fn write(&mut self, mut addr: usize, val: u8) {
        addr &= 0x3FFF;
        match addr {
            0..=0x1FFF => (),
            0x2000..=0x2FFF => self.mapper.borrow_mut().write_nametable(addr - 0x2000, val),
            0x3000..=0x3EFF => self.mapper.borrow_mut().write_nametable(addr - 0x3000, val),
            0x3F00..=0x3FFF => self.palette_write(addr, val),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn read(&mut self, mut addr: usize) -> u8 {
        addr &= 0x3FFF;
        match addr {
            0..=0x1FFF => self.mapper.borrow_mut().read_chr(addr),
            0x2000..=0x2FFF => self.mapper.borrow_mut().read_nametable(addr - 0x2000),
            0x3000..=0x3EFF => self.mapper.borrow_mut().read_nametable(addr - 0x3000),
            0x3F00..=0x3FFF => self.palette_read(addr),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn palette_write(&mut self, mut addr: usize, mut val: u8) {
        addr &= 0x1f;
        val &= 0x3f;

        match addr {
            0x0 | 0x10 => {
                self.palettes[0] = val;
                self.palettes[0x10] = val;
            }
            0x4 | 0x14 => {
                self.palettes[0x4] = val;
                self.palettes[0x14] = val;
            }
            0x8 | 0x18 => {
                self.palettes[0x8] = val;
                self.palettes[0x18] = val;
            }
            0xC | 0x1C => {
                self.palettes[0xC] = val;
                self.palettes[0x1C] = val;
            }
            _ => self.palettes[addr] = val,
        }
    }

    #[inline]
    fn palette_read(&mut self, mut addr: usize) -> u8 {
        addr &= 0x1F;
        if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
            addr &= !0x10;
        }

        let mut index = self.palettes[addr];

        if self.greyscale {
            index &= 0x30;
        }

        index
    }

    //Ppuctrl
    //    N -- 00000011 -- Name table address (0 = 0x2000; 1 = 0x2400; 2 = 0x2800; 3 = 0x2C00)
    //    I -- 00000100 -- PPU address increment (0: add 1, going across; 1: add 32, going down)
    //    S -- 00001000 -- Sprite pattern table address (0: 0x0000; 1: 0x1000; ignored in 8x16 mode)
    //    B -- 00010000 -- Background pattern table address (0: 0x0000; 1: 0x1000)
    //    H -- 00100000 -- Sprite size (0: 8x8, 1: 8x16)
    //    P -- 01000000 -- PPU master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)
    //    V -- 10000000 -- Execute NMI on vblank
    #[inline]
    fn write_ppuctrl(&mut self) {
        //TODO: bit 0 bus conflict
        let val = self.latch;
        self.temp_vram_addr &= !0xC00;
        self.temp_vram_addr |= ((val as usize) & 3) << 10;

        self.nt_base_addr = match val & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        };
        self.addr_increment = if val & (1 << 2) == 0 { 1 } else { 32 };
        self.sp_pattern_table_addr = if val & (1 << 3) == 0 { 0 } else { 0x1000 };
        self.bg_pattern_table_addr = if val & (1 << 4) == 0 { 0 } else { 0x1000 };
        self.sp_size = if val & (1 << 5) == 0 { 8 } else { 16 };
        self.nmi_on_vblank = val & (1 << 7) != 0;
    }

    //Ppumask
    //    g -- 00000001 -- Greyscale (0: normal color, 1: produce a greyscale display)
    //    m -- 00000010 -- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    //    M -- 00000100 -- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    //    b -- 00001000 -- 1: Show background, 0: Hide
    //    s -- 00010000 -- 1: Show sprites, 0: Hide
    //    R -- 00100000 -- Emphasize red
    //    G -- 01000000 -- Emphasize green
    //    B -- 10000000 -- Emphasize blue
    #[inline]
    fn write_ppumask(&mut self) {
        let val = self.latch;
        self.greyscale = val & 1 != 0;
        self.bg_leftmost_8 = val & (1 << 1) != 0;
        self.sp_leftmost_8 = val & (1 << 2) != 0;
        self.show_bg = val & (1 << 3) != 0;
        self.show_sp = val & (1 << 4) != 0;
        self.emphasize_red = val & (1 << 5) != 0;
        self.emphasize_green = val & (1 << 6) != 0;
        self.emphasize_blue = val & (1 << 7) != 0;
        self.rendering_enabled = self.show_bg || self.show_sp;
    }

    //Ppustatus
    //    O -- 00100000 -- Sprite overflow
    //    S -- 01000000 -- Sprite 0 hit
    //    V -- 10000000 -- Vertical blank has started (0: not in vblank; 1: in vblank)
    #[inline]
    fn read_ppustatus(&mut self) {
        self.write_toggle = false;
        self.latch = self.ppustatus;
        self.ppustatus &= 0x7F;

        //Reading $2002 within a few PPU clocks of when VBL is set results in special-case behavior.
        //Reading one PPU clock before reads it as clear and never sets the flag or generates
        //NMI for that frame. Reading on the same PPU clock or one later reads it as set,
        //clears it, and suppresses the NMI for that frame.
        if self.scanline == 241 && (self.xpos == 2 || self.xpos == 3) {
            self.latch |= 0x80;
            self.interrupt_bus.borrow_mut().nmi_signal = false;
        } else if self.scanline == 241 && self.xpos == 1 {
            self.latch &= 0x7F;
            self.suppress_vbl = true;
        }
    }

    #[inline]
    fn write_oamaddr(&mut self) {
        self.oamaddr = self.latch;
    }

    #[inline]
    fn read_oamdata(&mut self) {
        if self.scanline <= 239 && self.rendering_enabled {
            self.latch = self.oamdata_buffer;
        } else {
            self.latch = self.oam[self.oamaddr as usize];
        }
    }

    #[inline]
    fn write_oamdata(&mut self) {
        if self.rendering_enabled && (self.scanline <= 239 || self.scanline == 261) {
            self.oamaddr = self.oamaddr.wrapping_add(4);
        } else {
            let val = if self.oamaddr & 3 == 2 {
                self.latch & 0xE3
            } else {
                self.latch
            };

            self.oam[self.oamaddr as usize] = val;
            self.oamaddr = self.oamaddr.wrapping_add(1);
        }
    }

    #[inline]
    fn write_ppuscroll(&mut self) {
        let val = self.latch;
        if self.write_toggle {
            self.temp_vram_addr = (self.temp_vram_addr & !0x73E0)
                | ((val as usize & 0xF8) << 2)
                | ((val as usize & 7) << 12);
        } else {
            self.temp_vram_addr = (self.temp_vram_addr & !0x1F) | (val as usize >> 3);
            self.x_fine_scroll = val & 7;
        }

        self.write_toggle = !self.write_toggle;
    }

    #[inline]
    fn write_ppuaddr(&mut self) {
        let val = self.latch;
        if self.write_toggle {
            self.temp_vram_addr = (self.temp_vram_addr & !0xFF) | val as usize;
            //TODO: add 2-3 cycle delay to the update
            self.vram_addr = self.temp_vram_addr;
        } else {
            self.temp_vram_addr = (self.temp_vram_addr & !0xFF00) | ((val as usize & 0x3F) << 8);
        }

        self.write_toggle = !self.write_toggle;
    }

    #[inline]
    fn read_ppudata(&mut self) {
        self.latch = self.read_buffer;
        self.read_buffer = self.read(self.vram_addr);

        if (self.vram_addr & 0x3FFF) >= 0x3F00 {
            self.latch = self.read(self.vram_addr);
            self.read_buffer = self
                .mapper
                .borrow_mut()
                .read_nametable(self.vram_addr - 0x3000);
        }

        //TODO: buffered reads ?
        if self.rendering_enabled && self.scanline < 240 {
            self.coarse_x_increment();
            self.y_increment();
        } else {
            //TODO: trigger some memory read
            self.vram_addr += self.addr_increment;
        }
    }

    #[inline]
    fn write_ppudata(&mut self) {
        self.write(self.vram_addr, self.latch);

        if self.rendering_enabled && self.scanline < 240 {
            self.coarse_x_increment();
            self.y_increment();
        } else {
            //TODO: trigger some memory read
            self.vram_addr += self.addr_increment;
        }
    }

    #[inline]
    fn attr_table_addr(&self) -> usize {
        0x23C0
            | (self.vram_addr & 0xC00)
            | ((self.vram_addr >> 4) & 0x38)
            | ((self.vram_addr >> 2) & 7)
    }

    #[inline]
    fn nametable_addr(&self) -> usize {
        0x2000 | (self.vram_addr & 0xFFF)
    }

    #[inline]
    pub fn tick(&mut self) {
        let state = match self.scanline {
            0...239 => RenderState::Render,
            240 => RenderState::PostRender,
            241...260 => RenderState::VBlank,
            261 => RenderState::PreRender,
            _ => unreachable!("Invalid render state"),
        };

        debug_log!("at scanline {} cycle {}", (self.scanline), (self.xpos));
        debug_log!("vram address: 0x{:X}", (self.vram_addr));

        self.scanline_tick(state);

        self.xpos += 1;
        if self.xpos > 340 {
            self.xpos = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
            }
        }
    }

    //https://wiki.nesdev.com/w/index.php/PPU_rendering
    //http://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching
    //https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
    #[inline]
    fn scanline_tick(&mut self, state: RenderState) {
        match state {
            RenderState::PreRender => {
                match self.xpos {
                    1 => {
                        self.ppustatus &= !(1 << 7);
                        self.fetch_bg();
                    }
                    2..=256 => {
                        self.fetch_bg();
                        if self.xpos == 256 {
                            self.y_increment();
                        }
                    }
                    257 => {
                        self.t_to_v();
                        self.fetch_sprites();
                    }
                    258..=279 => self.fetch_sprites(),
                    280..=304 => {
                        self.v_from_t();
                        self.fetch_sprites();
                    }
                    305..=320 => self.fetch_sprites(),
                    321 => {
                        self.fetch_bg();
                    }
                    322..=337 => {
                        self.shift_tile_registers();
                        self.fetch_bg();
                    }
                    339 => {
                        (*self.frame).borrow_mut().frame_ready = true;
                        for (index, color_index) in self.output_buffer.iter().enumerate() {
                            (*self.frame).borrow_mut().output_buffer[index] =
                                PALETTE[*color_index as usize];
                        }

                        self.read(self.nametable_addr());

                        //The skipped tick is implemented by jumping directly from (339, 261)
                        //to (0, 0), meaning the last tick of the last NT fetch takes place at (0, 0)
                        //on odd frames replacing the idle tick
                        if self.odd_frame & self.rendering_enabled {
                            self.scanline = 0;
                            self.xpos = 0;
                        }

                        self.odd_frame = !self.odd_frame;
                    }
                    _ => (),
                }
            }
            RenderState::Render => match self.xpos {
                1 => {
                    self.fetch_bg();
                    self.draw_pixel();
                    //TODO: secondary clear cycle-accurate ?
                    self.secondary_oam = [0xFF; 0x20];
                }
                2..=256 => {
                    self.shift_tile_registers();
                    self.fetch_bg();
                    if self.xpos == 256 {
                        self.y_increment();
                        self.sprite_evaluation();
                    } else if self.xpos >= 65 {
                        self.sprite_evaluation();
                    }

                    self.draw_pixel();
                }
                257 => {
                    self.t_to_v();
                    self.shift_tile_registers();
                    self.fetch_sprites();
                }
                258..=320 => self.fetch_sprites(),
                321 => self.fetch_bg(),
                322..=337 => {
                    self.shift_tile_registers();
                    self.fetch_bg();
                }
                339 => {
                    self.read(self.nametable_addr());
                }
                _ => (),
            },
            RenderState::VBlank => self.handle_vblank(),
            _ => (),
        }
    }

    //Tile and attribute fetching

    //The high bits of v are used for fine Y during rendering,
    //and addressing nametable data only requires 12 bits,
    //with the high 2 CHR addres lines fixed to the 0x2000 region.
    //The address to be fetched during rendering can be deduced from v in the following way:

    //nametable address = 0x2000 | (v & 0x0FFF)
    //attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    //tile address low  = (nametable address << 4) | (v >> 12) | background patter table address
    //tile address high = tile address low + 8

    //The low 12 bits of the attribute address are composed in the following way:

    //NN 1111 YYY XXX
    //|| |||| ||| +++-- high 3 bits of coarse X (x/4)
    //|| |||| +++------ high 3 bits of coarse Y (y/4)
    //|| ++++---------- attribute offset (960 bytes)
    //++--------------- nametable select

    //The 15 bit registers t and v are composed this way during rendering:
    //yyy NN YYYYY XXXXX
    //||| || ||||| +++++-- coarse X scroll
    //||| || +++++-------- coarse Y scroll
    //||| ++-------------- nametable select
    //+++----------------- fine Y scroll
    #[inline]
    fn fetch_bg(&mut self) {
        if self.rendering_enabled {
            match self.xpos & 7 {
                0 => self.coarse_x_increment(),
                1 => {
                    self.shift_low |= u16::from(self.tile_lb);
                    self.shift_high |= u16::from(self.tile_hb);
                    self.attr_table_byte = self.next_attr_table_byte;

                    self.nametable_byte = self.read(self.nametable_addr());

                    debug_log!("---- fetching data for a new tile ----",);
                    debug_log!("shift_high {:b}", (self.shift_high));
                    debug_log!("shift_low {:b}", (self.shift_low));
                    debug_log!("VRAM addr = 0x{:X}", (self.vram_addr));
                    debug_log!("Nametable byte = 0x{:X}", (self.nametable_byte));
                }

                //The 2-bit 1-of-4 selector" is used to shift the attribute byte right
                //by 0, 2, 4, or 6 bits depending on bit 4 of the X and Y pixel position.
                //Roughly: if (v & 0x40) attrbyte >>= 4; if (v & 0x02) attrbyte >>= 2.
                3 => {
                    let shift = ((self.vram_addr >> 4) & 0x04) | (self.vram_addr & 0x02);
                    self.next_attr_table_byte =
                        ((self.read(self.attr_table_addr()) >> shift) & 0x03) << 2;
                }
                5 => {
                    self.tile_addr = (usize::from(self.nametable_byte) << 4)
                        | (self.vram_addr >> 12)
                        | self.bg_pattern_table_addr;
                    self.tile_lb = self.read(self.tile_addr);
                    debug_log!("Tile addr low = 0x{:X}", (self.tile_addr));
                    debug_log!("Tile lb = {:b}", (self.tile_lb));
                }
                7 => {
                    self.tile_hb = self.read(self.tile_addr + 8);
                    debug_log!("Tile hb = {:b}", (self.tile_hb));
                }
                _ => (),
            }
        }
    }

    #[inline]
    fn fetch_sprites(&mut self) {
        if self.rendering_enabled {
            self.oamaddr = 0;
            match (self.xpos - 1) & 7 {
                0 => {
                    self.read(self.nametable_addr());
                }
                2 => {
                    self.read(self.attr_table_addr());
                }
                3 => self.load_sprite(),
                4 => {
                    let index = self.sprite_buffer[self.sprite_index as usize].index;
                    self.sprite_buffer[self.sprite_index as usize].tile_low =
                        self.read(index as usize);
                }
                6 => {
                    let index = self.sprite_buffer[self.sprite_index as usize].index;
                    self.sprite_buffer[self.sprite_index as usize].tile_low =
                        self.read(index as usize + 8);
                    self.sprite_index = self.sprite_index & 7;
                }
                _ => (),
            }
        }
    }

    #[inline]
    fn load_sprite(&mut self) {
        let sprite_addr = 4 * self.sprite_index as usize;
        let sprite = &mut self.sprite_buffer[self.sprite_index as usize];
        sprite.y = self.secondary_oam[sprite_addr];
        sprite.x = self.secondary_oam[sprite_addr + 3];

        let attributes = self.secondary_oam[sprite_addr + 2];
        sprite.vertical_flip = attributes & 0x80 != 0;
        sprite.horizontal_flip = attributes & 0x40 != 0;
        sprite.priority = attributes & 0x20 != 0;
        sprite.palette = ((attributes & 3) << 2) | 0x10;

        let scanline = if self.scanline == 261 {
            -1
        } else {
            self.scanline as i16
        };
        let mut y_offset = if sprite.vertical_flip {
            (self.sp_size - 1) as i16 - (scanline - sprite.y as i16)
        } else {
            scanline - sprite.y as i16
        };

        let index = self.secondary_oam[sprite_addr + 1];
        sprite.index = if self.sp_size == 8 {
            self.sp_pattern_table_addr as u16 | (u16::from(index) << 4) | y_offset as u16
        } else {
            if y_offset >= 8 {
                y_offset += 8;
            }
            let pattern_table_addr = if index & 1 != 0 { 0x1000 } else { 0 };
            pattern_table_addr | (u16::from(index & !1) << 4) + y_offset as u16
        };
    }

    //http://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
    #[inline]
    fn sprite_evaluation(&mut self) {
        //TODO: If the sprite address (OAMADDR, $2003) is not zero at the beginning of
        //the pre-render scanline,on the 2C02 an OAM hardware refresh bug will cause the
        //first 8 bytes of OAM to be overwritten by the 8 bytes beginning at OAMADDR & $F8 before sprite evaluation begins.[1][2]
        if self.rendering_enabled {
            if self.xpos == 65 {
                self.sprite_eval_count = 0;
                self.oamdata_buffer = 0;
                self.sprites_found = 0;
                self.sprite_fetch_step = 0;
                self.sprite_eval_state = 1;
            }

            if self.xpos & 1 != 0 {
                self.oamdata_buffer = self.oam[self.oamaddr as usize];
            } else {
                match self.sprite_eval_state {
                    1 => {
                        //1. Starting at n = 0, read a sprite's Y-coordinate (OAM[n][0], copying it to the next open slot
                        //in secondary OAM (unless 8 sprites have been found, in which case the write is ignored).
                        if self.sprites_found < 8 {
                            self.secondary_oam[4 * self.sprites_found as usize] =
                                self.oamdata_buffer;

                            self.sprite_in_range = self.scanline >= self.oamdata_buffer as u16
                                && self.scanline < self.oamdata_buffer as u16 + self.sp_size as u16;

                            if !self.sprite_in_range {
                                self.sprite_eval_state = 2;
                            } else {
                                self.sprite_fetch_step = 1;
                                self.sprite_eval_state = 5;
                            }
                        }
                    }
                    2 => {
                        //2. Increment n.
                        self.sprite_eval_count += 1;
                        if self.sprite_eval_count == 65 {
                            self.sprite_eval_count = 0;
                            //2a. If n has overflowed back to zero (all 64 sprites evaluated), go to 4.
                            self.sprite_eval_state = 4;
                        } else if self.sprites_found < 8 {
                            //2b. If less than 8 sprites have been found, go to 1.
                            self.sprite_eval_state = 1;
                        } else if self.sprites_found == 8 {
                            //2c. If exactly 8 sprites have been found, disable writes to secondary OAM because it is full.
                            //This causes sprites in back to drop out.
                            self.sprite_eval_state = 3;
                        }
                    }
                    3 => {
                        //3. Starting at m = 0, evaluate OAM[n][m] as a Y-coordinate.
                        if self.scanline >= self.oamdata_buffer as u16
                            && self.scanline < (self.oamdata_buffer + self.sp_size) as u16
                        {
                            self.ppustatus |= 0x20;
                            self.sprite_fetch_step = 1;
                            self.sprite_eval_state = 8;
                        } else {
                            //3b. If the value is not in range, increment n and m (without carry).
                            //If n overflows to 0, go to 4; otherwise go to 3.
                            self.sprite_eval_count += 1;
                            self.sprite_fetch_step += 1;
                            if self.sprite_eval_count == 65 {
                                self.sprite_eval_count = 0;
                                self.sprite_eval_state = 4;
                            }
                        }
                    }
                    4 => {
                        //4. Attempt (and fail) to copy OAM[n][0] into the next free slot in secondary OAM,
                        //and increment n (repeat until HBLANK is reached).
                        self.sprite_eval_count += 1;
                    }
                    5 => {
                        //1a. If Y-coordinate is in range, copy remaining bytes of sprite data (OAM[n][1] thru OAM[n][3]) into secondary OAM.
                        self.secondary_oam
                            [(4 * self.sprites_found + self.sprite_fetch_step) as usize] =
                            self.oamdata_buffer;
                        self.sprite_fetch_step += 1;

                        if self.sprite_fetch_step == 4 {
                            self.sprite_fetch_step = 0;
                            self.sprites_found += 1;
                            self.sprite_eval_state = 2;
                        }
                    }
                    6 => {
                        //3a. If the value is in range, set the sprite overflow flag in $2002 and read the next 3 entries of
                        //OAM (incrementing 'm' after each byte and incrementing 'n' when 'm' overflows); if m = 3, increment n.
                        self.sprite_fetch_step += 1;

                        if self.sprite_fetch_step == 4 {
                            self.sprite_fetch_step = 0;
                            self.sprite_eval_count += 1;
                            self.sprite_eval_state = 4;
                        }
                    }
                    _ => (),
                }
                self.oamaddr = 4 * self.sprite_eval_count + self.sprite_fetch_step;
            }
        }
    }

    #[inline]
    fn handle_vblank(&mut self) {
        match (self.scanline, self.xpos) {
            (241, 0) => (),
            (241, 1) => {
                if !self.suppress_vbl {
                    self.ppustatus |= 0x80
                };
                self.suppress_vbl = false;
                if self.nmi_on_vblank && self.ppustatus & 0x80 != 0 {
                    self.interrupt_bus.borrow_mut().nmi_signal = true;
                }
            }
            (260, 340) => self.interrupt_bus.borrow_mut().nmi_signal = false,
            (241..=260, _) => {
                let current_nmi = self.nmi_on_vblank && (self.ppustatus & 0x80) != 0;
                if self.prev_nmi && current_nmi {
                    self.interrupt_bus.borrow_mut().nmi_signal = true;
                } else if !self.prev_nmi {
                    self.interrupt_bus.borrow_mut().nmi_signal = false;
                }
                self.prev_nmi = current_nmi;
            }
            _ => (),
        }
    }

    //Taken from: http://wiki.nesdev.com/w/index.php/PPU_scrolling
    #[inline]
    fn y_increment(&mut self) {
        if self.rendering_enabled {
            if (self.vram_addr & 0x7000) != 0x7000 {
                self.vram_addr += 0x1000;
            } else {
                self.vram_addr &= !0x7000;
                let mut y = (self.vram_addr & 0x3E0) >> 5;
                if y == 29 {
                    y = 0;
                    self.vram_addr ^= 0x800;
                } else if y == 31 {
                    y = 0;
                } else {
                    y += 1;
                }

                self.vram_addr = (self.vram_addr & !0x3E0) | (y << 5);
            }
        }
    }

    //Taken from: http://wiki.nesdev.com/w/index.php/PPU_scrolling
    #[inline]
    fn coarse_x_increment(&mut self) {
        if (self.vram_addr & 0x1F) == 31 {
            self.vram_addr &= !0x1F;
            self.vram_addr ^= 0x400
        } else {
            self.vram_addr += 1;
        }
    }

    //At dot 257 of each scanline
    //If rendering is enabled, the PPU copies all bits related to horizontal position from t to v:
    //v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
    #[inline]
    fn t_to_v(&mut self) {
        if self.rendering_enabled {
            self.vram_addr = (self.vram_addr & !0x41F) | (self.temp_vram_addr & 0x41F);
        }
    }

    //During dots 280 to 304 of the pre-render scanline (end of vblank)
    //If rendering is enabled, at the end of vblank, shortly after the horizontal
    //bits are copied from t to v at dot 257, the PPU will repeatedly copy the
    //vertical bits from t to v from dots 280 to 304, completing the full initialization of v from t:
    //v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
    #[inline]
    fn v_from_t(&mut self) {
        if self.rendering_enabled {
            self.vram_addr = (self.vram_addr & !0x7BE0) | (self.temp_vram_addr & 0x7BE0);
        }
    }

    #[inline]
    fn shift_tile_registers(&mut self) {
        self.shift_low <<= 1;
        self.shift_high <<= 1;
    }

    #[inline]
    fn draw_pixel(&mut self) {
        let addr = (usize::from(self.scanline) << 8) + usize::from(self.xpos - 1);
        let color_index = self.priority_mux();
        self.output_buffer[addr] = self.palette_read(color_index);
    }

    #[inline]
    fn priority_mux(&mut self) -> usize {
        if !self.rendering_enabled && (self.vram_addr & 0x3F00) == 0x3F00 {
            return self.vram_addr & 0x1F;
        }

        let mut color_index = 0;
        if self.show_bg {
            color_index = self.bg_color();
        }

        //TODO: sprites

        color_index
    }

    #[inline]
    fn bg_color(&mut self) -> usize {
        let tile_h_bit = ((self.shift_high << u16::from(self.x_fine_scroll)) & 0x8000) >> 14;
        let tile_l_bit = ((self.shift_low << u16::from(self.x_fine_scroll)) & 0x8000) >> 15;
        (u16::from(self.attr_table_byte) | tile_h_bit | tile_l_bit) as usize
    }
}
