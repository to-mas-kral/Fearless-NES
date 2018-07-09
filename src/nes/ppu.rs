#![allow(unused_variables)]

use std::cell::RefCell;
use std::rc::Rc;

use super::mapper::Mapper;
use super::InterruptBus;

static PALETTE: [u8; 192] = [
    124, 124, 124, 0, 0, 252, 0, 0, 188, 68, 40, 188, 148, 0, 132, 168, 0, 32, 168, 16, 0, 136, 20,
    0, 80, 48, 0, 0, 120, 0, 0, 104, 0, 0, 88, 0, 0, 64, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 188,
    188, 0, 120, 248, 0, 88, 248, 104, 68, 252, 216, 0, 204, 228, 0, 88, 248, 56, 0, 228, 92, 16,
    172, 124, 0, 0, 184, 0, 0, 168, 0, 0, 168, 68, 0, 136, 136, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248,
    248, 248, 60, 188, 252, 104, 136, 252, 152, 120, 248, 248, 120, 248, 248, 88, 152, 248, 120,
    88, 252, 160, 68, 248, 184, 0, 184, 248, 24, 88, 216, 84, 88, 248, 152, 0, 232, 216, 120, 120,
    120, 0, 0, 0, 0, 0, 0, 252, 252, 252, 164, 228, 252, 184, 184, 248, 216, 184, 248, 248, 184,
    248, 248, 164, 192, 240, 208, 176, 252, 224, 168, 248, 216, 120, 216, 248, 120, 184, 248, 184,
    184, 248, 216, 0, 252, 252, 248, 216, 248, 0, 0, 0, 0, 0, 0,
];

pub struct PpuMemory<M>
where
    M: Mapper,
{
    pub ram: [u8; 0x1000],
    pub oam: [u8; 0x100],
    pub palettes: [u8; 0x20],

    mapper: Rc<RefCell<M>>,

    vram_addr: usize,      //15 bits
    temp_vram_addr: usize, //15 bits
    x_fine_scroll: u8,     //3 bits

    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    pub oamaddr: u8,
    write_toggle: bool,
}

impl<M> PpuMemory<M>
where
    M: Mapper,
{
    pub fn new(mapper: Rc<RefCell<M>>) -> PpuMemory<M> {
        //TODO: set startup state
        let palettes = [
            0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00,
            0x04, 0x2C, 0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02,
            0x00, 0x20, 0x2C, 0x08,
        ];

        PpuMemory {
            ram: [0; 0x1000],
            oam: [0; 0x100],
            palettes,

            mapper,

            vram_addr: 0,
            temp_vram_addr: 0,
            x_fine_scroll: 0,

            ppuctrl: 0,
            ppumask: 0,
            ppustatus: 0,
            oamaddr: 0,
            write_toggle: false,
        }
    }

    #[inline]
    pub fn write(&mut self, addr: usize, val: u8) {
        let addr = addr & 0x4000;
        match addr {
            0..=0x1FFF => self.mapper.borrow_mut().write_chr(addr, val),
            0x2000..=0x2FFF => self.ram[addr - 0x2000] = val, //TODO: nametable mirrorring in mapper
            0x3000..=0x3EFF => self.ram[addr - 0x3000] = val,
            0x3F00..=0x3FFF => self.palettes[(addr - 0x3F00) % 0x20] = val,
            _ => panic!("internal error in PPU memory mapping"),
        }
    }

    #[inline]
    pub fn read(&mut self, addr: usize) -> u8 {
        let addr = addr & 0x4000;
        match addr {
            0..=0x1FFF => self.mapper.borrow_mut().read_chr(addr),
            0x2000..=0x2FFF => self.ram[addr - 0x2000], //TODO: nametable mirrorring in mapper
            0x3000..=0x3EFF => self.ram[addr - 0x3000],
            0x3F00..=0x3FFF => self.palettes[(addr - 0x3F00) % 0x20],
            _ => panic!("internal error in PPU memory mapping"),
        }
    }

    #[inline]
    pub fn write_ppuctrl(&mut self, val: u8) {
        //TODO: ignore writes after reset (30000 cycles)
        //TODO: bit 0 bus conflict
        self.ppuctrl = val;
    }

    #[inline]
    pub fn write_ppumask(&mut self, val: u8) {
        self.ppumask = val;
    }

    #[inline]
    pub fn read_ppustatus(&mut self) -> u8 {
        //TODO: clear the latch; is this the way ?
        self.temp_vram_addr = 0;
        self.ppustatus &= 0xFF >> 1;
        self.ppustatus
    }

    #[inline]
    pub fn write_oamaddr(&mut self, val: u8) {
        self.oamaddr = val;
    }

    #[inline]
    pub fn read_oamdata(&mut self) -> u8 {
        self.oam[self.oamaddr as usize]
        //TODO: increment in some cases
    }

    #[inline]
    pub fn write_oamdata(&mut self, val: u8) {
        //TODO: ignore writes during rendering
        //TODO: implement other trickery involved with this register
        self.oam[self.oamaddr as usize] = val;
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }

    #[inline]
    pub fn write_ppuscroll(&mut self, val: u8) {
        if !self.write_toggle {
            self.temp_vram_addr = (self.temp_vram_addr & 0xFFE0) | (usize::from(val) >> 3);
            self.x_fine_scroll = val & 0x7;
        } else {
            self.temp_vram_addr = (self.temp_vram_addr & 0x8FFF) | ((usize::from(val) & 0x7) << 12);
            self.temp_vram_addr = (self.temp_vram_addr & 0xFC1F) | ((usize::from(val) & 0xF8) << 2);
        }

        self.write_toggle = !self.write_toggle;
    }

    #[inline]
    pub fn write_ppuaddr(&mut self, val: u8) {
        if !self.write_toggle {
            self.temp_vram_addr = (self.temp_vram_addr & 0x80FF) | ((usize::from(val) & 0x3F) << 8)
        } else {
            self.temp_vram_addr = (self.temp_vram_addr & 0xFF00) | usize::from(val);
            self.vram_addr = self.temp_vram_addr;
        }

        self.write_toggle = !self.write_toggle;
    }

    #[inline]
    pub fn read_ppudata(&mut self) -> u8 {
        let increment = self.addr_increment();
        let ret = self.read(self.vram_addr);
        //TODO: buffered reads ?
        self.vram_addr = self.vram_addr.wrapping_add(increment);
        ret
    }

    #[inline]
    pub fn write_ppudata(&mut self, val: u8) {
        self.write(self.vram_addr, val);
        let increment = self.addr_increment();
        self.vram_addr = self.vram_addr.wrapping_add(increment);
    }

    //Ppuctrl
    //    N -- 00000011 -- Name table address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    //    I -- 00000100 -- PPU address increment (0: add 1, going across; 1: add 32, going down)
    //    S -- 00001000 -- Sprite pattern table address (0: $0000; 1: $1000; ignored in 8x16 mode)
    //    B -- 00010000 -- Background pattern table address (0: $0000; 1: $1000)
    //    H -- 00100000 -- Sprite size (0: 8x8, 1: 8x16)
    //    P -- 01000000 -- PPU master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)
    //    V -- 10000000 -- Execute NMI on vblank

    #[inline]
    pub fn name_table_base_addr(&self) -> usize {
        match self.ppuctrl & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn addr_increment(&self) -> usize {
        if self.ppuctrl & (1 << 2) == 0 {
            1
        } else {
            32
        }
    }

    #[inline]
    pub fn sp_pattern_table_addr(&self) -> usize {
        //TODO: like this ?
        if let SpriteSize::_8x16 = self.sp_size() {
            return 0;
        }

        if self.ppuctrl & (1 << 3) == 0 {
            0
        } else {
            0x1000
        }
    }

    #[inline]
    pub fn bg_pattern_table_addr(&self) -> usize {
        if self.ppuctrl & (1 << 4) == 0 {
            0
        } else {
            0x1000
        }
    }

    #[inline]
    pub fn sp_size(&self) -> SpriteSize {
        if self.ppuctrl & (1 << 5) == 0 {
            SpriteSize::_8x8
        } else {
            SpriteSize::_8x16
        }
    }

    #[inline]
    pub fn nmi_on_vblank(&self) -> bool {
        self.ppuctrl & (1 << 7) != 0
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
    pub fn greyscale(&self) -> bool {
        self.ppumask & 1 != 0
    }

    #[inline]
    pub fn bg_leftmost_8(&self) -> bool {
        self.ppumask & (1 << 1) != 0
    }

    #[inline]
    pub fn sp_leftmost_8(&self) -> bool {
        self.ppumask & (1 << 2) != 0
    }

    #[inline]
    pub fn show_bg(&self) -> bool {
        self.ppumask & (1 << 3) != 0
    }

    #[inline]
    pub fn show_sp(&self) -> bool {
        self.ppumask & (1 << 4) != 0
    }

    #[inline]
    pub fn rendering_enabled(&self) -> bool {
        self.show_bg() || self.show_sp()
    }

    #[inline]
    pub fn emphasize_red(&self) -> bool {
        self.ppumask & (1 << 5) != 0
    }

    #[inline]
    pub fn emphasize_green(&self) -> bool {
        self.ppumask & (1 << 6) != 0
    }

    #[inline]
    pub fn emphasize_blue(&self) -> bool {
        self.ppumask & (1 << 7) != 0
    }

    //Ppustatus
    //    O -- 00100000 -- Sprite overflow
    //    S -- 01000000 -- Sprite 0 hit
    //    V -- 10000000 -- Vertical blank has started (0: not in vblank; 1: in vblank)

    #[inline]
    pub fn sp_overflow(&self) -> bool {
        self.ppustatus & (1 << 5) != 0
    }

    #[inline]
    pub fn sp_0_hit(&self) -> bool {
        self.ppustatus & (1 << 7) != 0
    }

    #[inline]
    pub fn in_vblank(&self) -> bool {
        self.ppustatus & (1 << 7) != 0
    }
}

pub enum SpriteSize {
    _8x8,
    _8x16,
}

struct Sprite {
    addr: u8, //Index in OAM (object attribute memory)
    x: u8,
    y: u8,
    tile: u8,
    attr: u8,
    data_l: u8, //Tile data (low)
    data_h: u8, //Tile data (high)
}

#[derive(Clone, Copy)]
enum RenderState {
    PreRender,
    Render,
    PostRender,
    VBlank,
}

pub struct Ppu<M>
where
    M: Mapper,
{
    pub output_buffer: [u16; 256 * 240],

    pub mem: PpuMemory<M>,

    interrupt_bus: Rc<RefCell<InterruptBus>>,
    nmi_reset: bool,

    pub xpos: u16,
    pub scanline: u16,
    pub odd_frame: bool,

    nametable_byte: u8,
    attr_table_byte: u8,
    offset_y: usize,
    tile_addr: usize,
    tile_lb: u8,
    tile_hb: u8,
    shift_low: u16,
    shift_high: u16,
}

impl<M> Ppu<M>
where
    M: Mapper,
{
    pub fn new(interrupt_bus: Rc<RefCell<InterruptBus>>, mapper: Rc<RefCell<M>>) -> Ppu<M> {
        Ppu {
            output_buffer: [0; 256 * 240],

            mem: PpuMemory::new(mapper),

            interrupt_bus,
            nmi_reset: false,

            xpos: 0,
            scanline: 0,
            odd_frame: true,

            nametable_byte: 0,
            attr_table_byte: 0,
            offset_y: 0,
            tile_addr: 0,
            tile_lb: 0,
            tile_hb: 0,
            shift_low: 0,
            shift_high: 0,
        }
    }

    //pub fn reset(&mut self) {}

    pub fn tick(&mut self) {
        let state = match self.scanline {
            0...239 => RenderState::Render,
            240 => RenderState::PostRender,
            241...260 => RenderState::VBlank,
            261 => RenderState::PreRender,
            _ => panic!("Invalid render state"),
        };

        self.scanline_cycle(state);

        self.xpos += 1;
        if self.xpos > 340 {
            self.xpos %= 341;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.odd_frame ^= true;
            }
        }
    }

    #[inline]
    fn scanline_cycle(&mut self, state: RenderState) {
        //TODO: check for self.mem.rendering_enabled()
        match state {
            RenderState::PreRender => {
                self.output_buffer = [0; 256 * 240];
                match self.xpos {
                    1..=256 => {
                        if self.xpos == 1 {
                            //TODO: Sprite 0 overflow
                            self.mem.ppustatus &= !(1 << 7);
                        }
                        self.fetch_bg();
                        if self.xpos == 256 {
                            self.y_increment();
                        }
                    }
                    257 => {
                        self.t_to_v();
                    }
                    280..=304 => {
                        self.v_from_t();
                    }
                    321..=339 => {
                        self.fetch_bg();
                        //TODO: unused NT fetches ?
                    }
                    340 => {
                        //The skipped tick is implemented by jumping directly from (339, 261)
                        //to (0, 0), meaning the last tick of the last NT fetch takes place at (0, 0)
                        //on odd frames replacing the idle tick
                        self.fetch_bg();
                        if self.scanline == 0 && self.odd_frame {
                            self.scanline = 0;
                            self.xpos = 0;
                            self.tick();
                        }
                    }
                    _ => (),
                }
            }
            RenderState::Render => match self.xpos {
                1..=256 => {
                    self.fetch_bg();
                    if self.xpos == 256 {
                        self.y_increment();
                    }
                    //TODO: drawing
                    self.draw_pixel();
                }
                257 => {
                    self.t_to_v();
                }
                321..=340 => {
                    self.fetch_bg();
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
    //tile address low  = (nametable address << 4) | (v >> 12) | (nametable base address (bits 0 a 1 of ppuctrl))
    //tile address high = tile address low + 8

    //The low 12 bits of the attribute address are composed in the following way:

    //NN 1111 YYY XXX
    //|| |||| ||| +++-- high 3 bits of coarse X (x/4)
    //|| |||| +++------ high 3 bits of coarse Y (y/4)
    //|| ++++---------- attribute offset (960 bytes)
    //++--------------- nametable select

    #[inline]
    fn fetch_bg(&mut self) {
        match self.xpos % 8 {
            0 => self.coarse_x_increment(),
            1 => {
                self.shift_low <<= 1;
                self.shift_high <<= 1;

                self.shift_low = self.tile_lb as u16;
                self.shift_high = self.tile_hb as u16;

                let addr = 0x2000 | (self.mem.vram_addr & 0xFFF);
                self.nametable_byte = self.mem.read(addr);
                self.tile_addr = (usize::from(self.nametable_byte) << 4)
                    | (self.mem.vram_addr >> 12)
                    | self.mem.name_table_base_addr();
                self.offset_y = self.mem.vram_addr >> 12;
            }
            3 => {
                let shift = ((self.mem.vram_addr >> 4) & 0x04) | (self.mem.vram_addr & 0x02);
                let attr_table_addr = 0x23C0
                    | (self.mem.vram_addr & 0x0C00)
                    | ((self.mem.vram_addr >> 4) & 0x38)
                    | ((self.mem.vram_addr >> 2) & 0x07);
                self.attr_table_byte = (self.mem.read(attr_table_addr >> shift) & 0x03) << 2;
            }
            5 => self.tile_lb = self.mem.read(self.tile_addr),
            7 => self.tile_hb = self.mem.read(self.tile_addr + 8),
            _ => (),
        }
    }

    #[inline]
    fn handle_vblank(&mut self) {
        //So the NMI signal is high for the whole Vblank period ?

        //Yes, $2002 bit 7 is high during the whole vblank period if $2002 isn't read during that period.
        //The NMI signal is active low (0 = on, 1 = off), produced as $2002 bit 7 NAND $2000 bit 7.
        //The CPU calls the NMI handler when the NMI signal goes from high to low.
        //It's actually possible to make the NMI signal go from high to low twice within one vblank by
        //turning $2000 bit 7 off and then on without reading $2002
        //and one of the Bases Loaded games relies on that.
        match self.scanline {
            241 => {
                if self.xpos == 1 {
                    self.mem.ppustatus |= 1 << 7;
                    if self.mem.nmi_on_vblank() {
                        self.interrupt_bus.borrow_mut().nmi_signal = true;
                    }
                }
            }
            _ => {
                if !self.mem.nmi_on_vblank() {
                    self.nmi_reset = true;
                } else if self.nmi_reset {
                    self.nmi_reset = false;
                    self.interrupt_bus.borrow_mut().nmi_signal = true;
                }
            }
        }
    }

    //Taken from: http://wiki.nesdev.com/w/index.php/PPU_scrolling

    #[inline]
    fn y_increment(&mut self) {
        if (self.mem.vram_addr & 0x7000) != 0x7000 {
            self.mem.vram_addr += 0x1000;
        } else {
            self.mem.vram_addr &= !0x7000;
            let mut y = (self.mem.vram_addr & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.mem.vram_addr ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }

            self.mem.vram_addr = (self.mem.vram_addr & !0x03E0) | (y << 5);
        }
    }

    //Taken from: http://wiki.nesdev.com/w/index.php/PPU_scrolling

    #[inline]
    fn coarse_x_increment(&mut self) {
        if (self.mem.vram_addr & 0x001F) == 31 {
            self.mem.vram_addr &= !0x001F;
            self.mem.vram_addr ^= 0x0400
        } else {
            self.mem.vram_addr += 1;
        }
    }

    //At dot 257 of each scanline
    //If rendering is enabled, the PPU copies all bits related to horizontal position from t to v:
    //v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
    //TODO: rename this method
    #[inline]
    fn t_to_v(&mut self) {
        let mask = 0b11111 | (1 << 10);
        self.mem.vram_addr &= self.mem.temp_vram_addr & mask;
    }

    //During dots 280 to 304 of the pre-render scanline (end of vblank)
    //If rendering is enabled, at the end of vblank, shortly after the horizontal
    //bits are copied from t to v at dot 257, the PPU will repeatedly copy the
    //vertical bits from t to v from dots 280 to 304, completing the full initialization of v from t:
    //v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
    //TODO: rename this method
    #[inline]
    fn v_from_t(&mut self) {
        let mask = 0b11110_11111 << 5;
        self.mem.vram_addr &= self.mem.temp_vram_addr & mask;
    }

    #[inline]
    fn draw_pixel(&mut self) {}
}
