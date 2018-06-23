#![allow(non_snake_case)]

use super::memory;
use std::cell::RefCell;
use std::rc::Rc;

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



struct Registers {
    ppuctrl: Ppuctrl,
    ppumask: Ppumask,
    ppustatus: Ppustatus,
    oamaddr: u8,
    oamdata: u8,
    ppuscroll: u8,
    scroll_latch: u8,
    ppuadr: u8,
    adr_latch: u8,
    ppudata: u8,
    oamdma: u8,
}

impl Registers {
    pub fn write_ctrl(&mut self, value: u8) {
        self.ppuctrl.bits = value;
    }

    pub fn write_mask(&mut self, value: u8) {
        self.ppumask.bits = value;
    }

    pub fn read_status(&mut self) -> u8 {
        //TODO: clear D7 ???
        self.scroll_latch = 0;
        self.adr_latch = 0;
        self.ppustatus.bits
    }

    pub fn write_oamaddr(&mut self, value: u8) {
        self.oamaddr = value;
    }

    pub fn read_oamdata(&mut self) -> u8 {
        self.oamdata
    }

    pub fn write_oamdata(&mut self, value: u8) {
        //TODO: ignore writes during rendering
        self.oamdata = value;
        self.oamaddr.wrapping_add(1);
    }

    pub fn write_scrool(&mut self, value: u8) {
        
    }
}

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
    pub mem: Rc<RefCell<memory::Memory>>,
    pub ppuctrl: Ppuctrl,

    pub xpos: u16,
    pub scanline: u16,
    pub odd_frame: bool,
}

impl Ppu {
    pub fn new(mem: Rc<RefCell<memory::Memory>>) -> Ppu {
        Ppu {
            mem,
            regs: Registers,

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
