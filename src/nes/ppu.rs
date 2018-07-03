#![allow(unused_variables)]

use std::cell::Cell;
use std::rc::Rc;

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

bitflags! {
    struct Ppuctrl: u8 {
        const N = 0b00000011; //Name table address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const I = 0b00000100; //PPU address increment (0: add 1, going across; 1: add 32, going down)
        const S = 0b00001000; //Sprite pattern table address (0: $0000; 1: $1000; ignored in 8x16 mode)
        const B = 0b00010000; //Background pattern table address (0: $0000; 1: $1000)
        const H = 0b00100000; //Sprite size (0: 8x8, 1: 8x16)
        const P = 0b01000000; //PPU master/slave select (unused)
        const V = 0b10000000; //Execute NMI on vblank
    }
}

bitflags! {
    struct Ppumask: u8 {
        const g = 0b00000001; //Greyscale (0: normal color, 1: produce a greyscale display)
        const m = 0b00000010; //1: Show background in leftmost 8 pixels of screen, 0: Hide
        const M = 0b00000100; //1: Show sprites in leftmost 8 pixels of screen, 0: Hide
        const b = 0b00001000; //1: Show background, 0: Hide
        const s = 0b00010000; //1: Show sprites, 0: Hide
        const R = 0b00100000; //Emphasize red
        const G = 0b01000000; //Emphasize green
        const B = 0b10000000; //Emphasize blue
    }
}

bitflags! {
    struct Ppustatus: u8 {
        const O = 0b00100000; //Sprite overflow
        const S = 0b01000000; //Sprite 0 hit
        const V = 0b10000000; //Vertical blank has started (0: not in vblank; 1: in vblank)
    }
}

pub struct Memory {
    ram: [u8; 0x3FFF],
    pub oam: [u8; 0x100],

    vram_addr: usize,      //15 bits
    temp_vram_addr: usize, //15 bits
    x_fine_scroll: u8,     //3 bits

    ppuctrl: Ppuctrl,
    ppumask: Ppumask,
    ppustatus: Ppustatus,
    pub oamaddr: u8,
    oamdata: u8,
    ppuscroll: u8,
    scroll_latch: u8,
    addr_latch: u8,
    ppudata: u8,
    oamdma: u8,
    write_toggle: bool,
}

impl Memory {
    pub fn new() -> Memory {
        //TODO: set startup state
        Memory {
            ram: [0; 0x3FFF],
            oam: [0; 0x100],

            vram_addr: 0,
            temp_vram_addr: 0,
            x_fine_scroll: 0,

            ppuctrl: Ppuctrl::from_bits(0).unwrap(),
            ppumask: Ppumask::from_bits(0).unwrap(),
            ppustatus: Ppustatus::from_bits(0).unwrap(),
            oamaddr: 0,
            oamdata: 0,
            ppuscroll: 0,
            scroll_latch: 0,
            addr_latch: 0,
            ppudata: 0,
            oamdma: 0,
            write_toggle: false,
        }
    }

    #[inline]
    pub fn write_ram(&mut self, val: u8) {
        let addr = self.vram_addr & 0x4000;
    }

    #[inline]
    pub fn read_ram(&mut self) -> u8 {
        let addr = self.vram_addr & 0x4000;
        0
    }

    #[inline]
    pub fn write_ppuctrl(&mut self, val: u8) {
        //TODO: ignore writes after reset (30000 cycles)
        //TODO: bit 0 bus conflict
        self.ppuctrl.bits = val;
    }

    #[inline]
    pub fn write_ppumask(&mut self, val: u8) {
        self.ppumask.bits = val;
    }

    #[inline]
    pub fn read_ppustatus(&mut self) -> u8 {
        self.scroll_latch = 0;
        self.addr_latch = 0;
        self.ppustatus.bits &= 0xFF >> 1;
        self.ppustatus.bits
    }

    #[inline]
    pub fn write_oamaddr(&mut self, val: u8) {
        self.oamaddr = val;
    }

    #[inline]
    pub fn read_oamdata(&mut self) -> u8 {
        self.oamdata
    }

    #[inline]
    pub fn write_oamdata(&mut self, val: u8) {
        //TODO: ignore writes during rendering
        //TODO: implement other trickery involved with this register
        self.oamdata = val;
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }

    #[inline]
    pub fn write_ppuscroll(&mut self, val: u8) {
        if !self.write_toggle {
            self.scroll_latch = val;
        } else {
            self.ppuscroll = val;
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
        let increment = if self.ppuctrl.bits & (1 << 2) == 0 {
            1
        } else {
            32
        };
        let ret = self.read_ram();
        //TODO: buffered reads ?
        self.vram_addr = self.vram_addr.wrapping_add(increment);
        ret
    }

    #[inline]
    pub fn write_ppudata(&mut self, val: u8) {
        self.write_ram(val);
        let increment = if self.ppuctrl.bits & (1 << 2) == 0 {
            1
        } else {
            32
        };
        self.vram_addr = self.vram_addr.wrapping_add(increment);
    }

    #[inline]
    pub fn write_oamdma(&mut self, val: u8) {
        //TODO: wtf
    }
}

#[derive(Clone, Copy)]
enum Render_state {
    pre_render,
    render,
    post_render,
    v_blank,
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

pub struct Ppu {
    pub mem: Memory,

    interrupt_bus: Rc<Cell<InterruptBus>>,

    pub xpos: u16,
    pub scanline: u16,
    pub odd_frame: bool,
}

impl Ppu {
    pub fn new(interrupt_bus: Rc<Cell<InterruptBus>>) -> Ppu {
        Ppu {
            mem: Memory::new(),

            interrupt_bus,

            xpos: 0,
            scanline: 0,
            odd_frame: true,
        }
    }

    pub fn reset(&mut self) {}

    pub fn step(&mut self, cycles: u8) {
        let state = match self.scanline {
            261 => Render_state::pre_render,
            0...239 => Render_state::render,
            240 => Render_state::post_render,
            241...260 => Render_state::v_blank,
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

    fn scanline_cycle(&mut self, state: Render_state) {
        //if (s == NMI and dot == 1) { status.vBlank = true; if (ctrl.nmi) CPU::set_nmi(); }

    }
}
