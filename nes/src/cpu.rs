use bincode::{Decode, Encode};

use super::Nes;

#[derive(Decode, Encode)]
enum InterruptType {
    Nmi,
    Irq,
    Reset,
    None,
}

#[derive(Decode, Encode)]
enum DmaHijack {
    Request,
    Hijacked,
    None,
}

const RAM_SIZE: usize = 0x800;

/**
    Most of the documentation for the 6502 can be found on nesdev:
    http://nesdev.org/6502_cpu.txt
    However, a few illegal instructions (LAX and XAA) are basically undefined.
    Some information about those can be found in the visual6502.org wiki:
    http://visual6502.org/wiki/index.php?title=6502_Unsupported_Opcodes
    The 6502 has many quirks, some of them (such as the branch behavior) are
    described on visual6502.org wiki. Some of them are described in numerous
    random documents found in the hidden corners of the internet.
**/
#[derive(Decode, Encode)]
pub struct Cpu {
    // registers
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,

    // flags
    pub n: bool,
    pub v: bool,
    pub i: bool,
    pub z: bool,
    pub c: bool,
    /// BCD flag, BCD mode doesn't work on the NES CPU, but we still need to
    /// keep the state of this flag to return the correct value in pull_status()
    pub d: bool,

    // state needed for cycle-accuracy
    pub current_instruction: u8,
    pub odd_cycle: bool,
    /// Address bus
    pub ab: u16,
    /// Data bus
    db: u8,
    temp: u16,
    pub open_bus: u8,

    pub irq_mapper_signal: bool,
    pub irq_apu_signal: bool,
    pub nmi_signal: bool,
    /// status of the IRQ line sampled at the end of the penultimate cycle of an instruction
    cached_irq: bool,
    /// status of the NMI line sampled at the end of the penultimate cycle of an instruction
    cached_nmi: bool,
    reset_signal: bool,
    take_interrupt: bool,
    interrupt_type: InterruptType,

    dma_addr: u16,
    hijack_read: DmaHijack,
    copy_buffer: u8,
    dma_cycles: u16,

    ram: [u8; RAM_SIZE],
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD,

            n: false,
            v: false,
            d: false,
            i: true,
            z: false,
            c: false,

            current_instruction: 0,
            odd_cycle: false,

            ab: 0,
            db: 0,
            temp: 0,
            open_bus: 0,

            cached_irq: false,
            irq_mapper_signal: false,
            irq_apu_signal: false,
            cached_nmi: false,
            nmi_signal: false,
            reset_signal: false,
            take_interrupt: false,
            interrupt_type: InterruptType::None,

            dma_cycles: 0,
            hijack_read: DmaHijack::None,
            copy_buffer: 0,
            dma_addr: 0,

            ram: [0; RAM_SIZE],
        }
    }
}

impl Nes {
    pub(crate) fn cpu_gen_reset(&mut self) {
        // FIXME: cpu reset is broken...
        self.cpu.current_instruction = 0;
        self.cpu.take_interrupt = true;
        self.cpu.reset_signal = true;
        self.cpu.interrupt_type = InterruptType::Reset;
        self.cpu_write(0x4015, 0);
        self.cpu.reset_signal = false;
    }

    #[inline]
    pub(crate) fn cpu_read(&mut self, index: usize) -> u8 {
        self.cpu.open_bus = match index {
            0x4020..=0xFFFF => self.mapper.cpu_read(index).unwrap_or(self.cpu.open_bus),
            0..=0x1FFF => self.cpu.ram[index & 0x7FF],
            0x2000..=0x3FFF => self.ppu_read_reg(index),
            0x4000..=0x4014 | 0x4017..=0x401F => self.cpu.open_bus,
            0x4016 => (self.cpu.open_bus & 0xE0) | self.controller.read_reg(),
            0x4015 => self.apu_read_status(),
            _ => unreachable!("memory access into unmapped address: 0x{:X}", index),
        };

        self.cpu.db = self.cpu.open_bus;
        if let DmaHijack::Request = self.cpu.hijack_read {
            self.cpu.hijack_read = DmaHijack::Hijacked;
        }

        self.cpu.open_bus
    }

    #[inline]
    pub(crate) fn cpu_write(&mut self, index: usize, val: u8) {
        match index {
            0..=0x1FFF => self.cpu.ram[index & 0x7FF] = val,
            0x2000..=0x3FFF => self.ppu_write_reg(index, val),
            0x4000..=0x4013 => self.apu_write_reg(index, val),
            0x4014 => {
                self.cpu.hijack_read = DmaHijack::Request;
                self.cpu.dma_addr = (val as u16) << 8;
            }
            0x4015 => self.apu_write_reg(index, val),
            0x4016 => self.controller.write_reg(val),
            0x4017 => self.apu_write_reg(index, val),
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => self.mapper.cpu_write(
                index,
                val,
                self.cycle_count,
                &mut self.cpu.irq_mapper_signal,
            ),
            _ => unreachable!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    //https://forums.nesdev.org/viewtopic.php?f=3&t=14120
    #[inline]
    fn dma(&mut self) {
        if let DmaHijack::Hijacked = self.cpu.hijack_read {
            self.cpu.hijack_read = DmaHijack::None;
            self.cpu.dma_cycles = 1;
            self.clock_components();
        }

        if self.cpu.dma_cycles == 1 && self.cpu.odd_cycle {
            self.cpu_read(self.cpu.ab as usize);
        } else if self.cpu.dma_cycles >= 1 {
            if self.cpu.dma_cycles & 1 != 0 {
                self.cpu_read(self.cpu.dma_addr as usize);
                self.cpu.dma_addr = self.cpu.dma_addr.wrapping_add(1);
                self.cpu.copy_buffer = self.cpu.db
            } else {
                self.cpu_write(0x2004, self.cpu.copy_buffer);
            }
            self.cpu.dma_cycles += 1;

            if self.cpu.dma_cycles == 0x201 {
                self.cpu.dma_cycles = 0;

                self.load_next_instruction();
                self.clock_components();
            }
        }
    }
}

// Helper macros
macro_rules! check_read_hijack {
    ($self:ident) => {
        if let DmaHijack::Hijacked = $self.cpu.hijack_read {
            return;
        }
    };
}

macro_rules! cycle {
    ($self:ident) => {
        $self.cpu_read($self.cpu.ab as usize);
        check_read_hijack!($self);
    };
}

macro_rules! penultimate_cycle {
    ($self:ident) => {
        $self.cache_interrupts();
        $self.cpu_read($self.cpu.ab as usize);
        check_read_hijack!($self);
    };
}

macro_rules! last_cycle {
    ($self:ident) => {
        $self.check_interrupts();
        $self.cpu_read($self.cpu.ab as usize);
        check_read_hijack!($self);
    };
}

impl Nes {
    pub(crate) fn cpu_tick(&mut self) {
        self.dma();
        if self.cpu.dma_cycles != 0 {
            self.clock_components();
            return;
        }

        match self.cpu.current_instruction {
            0x00 => self.brk(),
            0x01 => self.indirect_x(Nes::ora),
            0x02 => self.immediate(Nes::halt),
            0x03 => self.indirect_x_illegal(Nes::slo),
            0x04 => self.zero_page(|_, _| ()),
            0x05 => self.zero_page(Nes::ora),
            0x06 => self.zero_page_rmw(Nes::asl),
            0x07 => self.zero_page_rmw(Nes::slo),
            0x08 => self.php(),
            0x09 => self.immediate(Nes::ora),
            0x0A => self.accumulator(Nes::asl_a),
            0x0B => self.immediate(Nes::anc),
            0x0C => self.absolute(|_, _| ()),
            0x0D => self.absolute(Nes::ora),
            0x0E => self.absolute_rmw(Nes::asl),
            0x0F => self.absolute_rmw(Nes::slo),
            0x10 => self.relative(!self.cpu.n),
            0x11 => self.indirect_y(Nes::ora),
            0x12 => self.immediate(Nes::halt),
            0x13 => self.indirect_y_illegal(Nes::slo),
            0x14 => self.zero_page_x(|_, _| ()),
            0x15 => self.zero_page_x(Nes::ora),
            0x16 => self.zero_page_x_rmw(Nes::asl),
            0x17 => self.zero_page_x_rmw(Nes::slo),
            0x18 => self.implied(Nes::clc),
            0x19 => self.absolute_y(Nes::ora),
            0x1A => self.implied(|_| ()),
            0x1B => self.absolute_y_illegal(Nes::slo),
            0x1C => self.absolute_x(|_, _| ()),
            0x1D => self.absolute_x(Nes::ora),
            0x1E => self.absolute_x_rmw(Nes::asl),
            0x1F => self.absolute_x_rmw(Nes::slo),
            0x20 => self.jsr(),
            0x21 => self.indirect_x(Nes::and),
            0x22 => self.immediate(Nes::halt),
            0x23 => self.indirect_x_illegal(Nes::rla),
            0x24 => self.zero_page(Nes::bit),
            0x25 => self.zero_page(Nes::and),
            0x26 => self.zero_page_rmw(Nes::rol),
            0x27 => self.zero_page_rmw(Nes::rla),
            0x28 => self.plp(),
            0x29 => self.immediate(Nes::and),
            0x2A => self.accumulator(Nes::rol_a),
            0x2B => self.immediate(Nes::anc),
            0x2C => self.absolute(Nes::bit),
            0x2D => self.absolute(Nes::and),
            0x2E => self.absolute_rmw(Nes::rol),
            0x2F => self.absolute_rmw(Nes::rla),
            0x30 => self.relative(self.cpu.n),
            0x31 => self.indirect_y(Nes::and),
            0x32 => self.immediate(Nes::halt),
            0x33 => self.indirect_y_illegal(Nes::rla),
            0x34 => self.zero_page_x(|_, _| ()),
            0x35 => self.zero_page_x(Nes::and),
            0x36 => self.zero_page_x_rmw(Nes::rol),
            0x37 => self.zero_page_x_rmw(Nes::rla),
            0x38 => self.implied(Nes::sec),
            0x39 => self.absolute_y(Nes::and),
            0x3A => self.implied(|_| ()),
            0x3B => self.absolute_y_illegal(Nes::rla),
            0x3C => self.absolute_x(|_, _| ()),
            0x3D => self.absolute_x(Nes::and),
            0x3E => self.absolute_x_rmw(Nes::rol),
            0x3F => self.absolute_x_rmw(Nes::rla),
            0x40 => self.rti(),
            0x41 => self.indirect_x(Nes::eor),
            0x42 => self.immediate(Nes::halt),
            0x43 => self.indirect_x_illegal(Nes::sre),
            0x44 => self.zero_page(|_, _| ()),
            0x45 => self.zero_page(Nes::eor),
            0x46 => self.zero_page_rmw(Nes::lsr),
            0x47 => self.zero_page_rmw(Nes::sre),
            0x48 => self.pha(),
            0x49 => self.immediate(Nes::eor),
            0x4A => self.accumulator(Nes::lsr_a),
            0x4B => self.immediate(Nes::alr),
            0x4C => self.absolute_jmp(),
            0x4D => self.absolute(Nes::eor),
            0x4E => self.absolute_rmw(Nes::lsr),
            0x4F => self.absolute_rmw(Nes::sre),
            0x50 => self.relative(!self.cpu.v),
            0x51 => self.indirect_y(Nes::eor),
            0x52 => self.immediate(Nes::halt),
            0x53 => self.indirect_y_illegal(Nes::sre),
            0x54 => self.zero_page_x(|_, _| ()),
            0x55 => self.zero_page_x(Nes::eor),
            0x56 => self.zero_page_x_rmw(Nes::lsr),
            0x57 => self.zero_page_x_rmw(Nes::sre),
            0x58 => self.implied(Nes::cli),
            0x59 => self.absolute_y(Nes::eor),
            0x5A => self.implied(|_| ()),
            0x5B => self.absolute_y_illegal(Nes::sre),
            0x5C => self.absolute_x(|_, _| ()),
            0x5D => self.absolute_x(Nes::eor),
            0x5E => self.absolute_x_rmw(Nes::lsr),
            0x5F => self.absolute_x_rmw(Nes::sre),
            0x60 => self.rts(),
            0x61 => self.indirect_x(Nes::adc),
            0x62 => self.immediate(Nes::halt),
            0x63 => self.indirect_x_illegal(Nes::rra),
            0x64 => self.zero_page(|_, _| ()),
            0x65 => self.zero_page(Nes::adc),
            0x66 => self.zero_page_rmw(Nes::ror),
            0x67 => self.zero_page_rmw(Nes::rra),
            0x68 => self.pla(),
            0x69 => self.immediate(Nes::adc),
            0x6A => self.accumulator(Nes::ror_a),
            0x6B => self.immediate(Nes::arr),
            0x6C => self.indirect(),
            0x6D => self.absolute(Nes::adc),
            0x6E => self.absolute_rmw(Nes::ror),
            0x6F => self.absolute_rmw(Nes::rra),
            0x70 => self.relative(self.cpu.v),
            0x71 => self.indirect_y(Nes::adc),
            0x72 => self.immediate(Nes::halt),
            0x73 => self.indirect_y_illegal(Nes::rra),
            0x74 => self.zero_page_x(|_, _| ()),
            0x75 => self.zero_page_x(Nes::adc),
            0x76 => self.zero_page_x_rmw(Nes::ror),
            0x77 => self.zero_page_x_rmw(Nes::rra),
            0x78 => self.implied(Nes::sei),
            0x79 => self.absolute_y(Nes::adc),
            0x7A => self.implied(|_| ()),
            0x7B => self.absolute_y_illegal(Nes::rra),
            0x7C => self.absolute_x(|_, _| ()),
            0x7D => self.absolute_x(Nes::adc),
            0x7E => self.absolute_x_rmw(Nes::ror),
            0x7F => self.absolute_x_rmw(Nes::rra),
            0x80 => self.immediate(|_, _| ()),
            0x81 => self.indirect_x_st(Nes::sta),
            0x82 => self.immediate(|_, _| ()),
            0x83 => self.indirect_x(Nes::aax),
            0x84 => self.zero_page_st(Nes::sty),
            0x85 => self.zero_page_st(Nes::sta),
            0x86 => self.zero_page_st(Nes::stx),
            0x87 => self.zero_page(Nes::aax),
            0x88 => self.implied(Nes::dey),
            0x89 => self.immediate(|_, _| ()),
            0x8A => self.implied(Nes::txa),
            0x8B => self.immediate(Nes::xaa),
            0x8C => self.absolute_st(Nes::sty),
            0x8D => self.absolute_st(Nes::sta),
            0x8E => self.absolute_st(Nes::stx),
            0x8F => self.absolute(Nes::aax),
            0x90 => self.relative(!self.cpu.c),
            0x91 => self.indirect_y_st(Nes::sta),
            0x92 => self.immediate(Nes::halt),
            0x93 => self.indirect_y_st(Nes::ahx),
            0x94 => self.zero_page_x_st(Nes::sty),
            0x95 => self.zero_page_x_st(Nes::sta),
            0x96 => self.zero_page_y_st(Nes::stx),
            0x97 => self.zero_page_y(Nes::aax),
            0x98 => self.implied(Nes::tya),
            0x99 => self.absolute_y_st(Nes::sta),
            0x9A => self.implied(Nes::txs),
            0x9B => self.absolute_y_st(Nes::tas),
            0x9C => self.absolute_x_st(Nes::shy),
            0x9D => self.absolute_x_st(Nes::sta),
            0x9E => self.absolute_y_st(Nes::shx),
            0x9F => self.absolute_y_st(Nes::ahx),
            0xA0 => self.immediate(Nes::ldy),
            0xA1 => self.indirect_x(Nes::lda),
            0xA2 => self.immediate(Nes::ldx),
            0xA3 => self.indirect_x(Nes::lax),
            0xA4 => self.zero_page(Nes::ldy),
            0xA5 => self.zero_page(Nes::lda),
            0xA6 => self.zero_page(Nes::ldx),
            0xA7 => self.zero_page(Nes::lax),
            0xA8 => self.implied(Nes::tay),
            0xA9 => self.immediate(Nes::lda),
            0xAA => self.implied(Nes::tax),
            0xAB => self.immediate(Nes::lax),
            0xAC => self.absolute(Nes::ldy),
            0xAD => self.absolute(Nes::lda),
            0xAE => self.absolute(Nes::ldx),
            0xAF => self.absolute(Nes::lax),
            0xB0 => self.relative(self.cpu.c),
            0xB1 => self.indirect_y(Nes::lda),
            0xB2 => self.immediate(Nes::halt),
            0xB3 => self.indirect_y(Nes::lax),
            0xB4 => self.zero_page_x(Nes::ldy),
            0xB5 => self.zero_page_x(Nes::lda),
            0xB6 => self.zero_page_y(Nes::ldx),
            0xB7 => self.zero_page_y(Nes::lax),
            0xB8 => self.implied(Nes::clv),
            0xB9 => self.absolute_y(Nes::lda),
            0xBA => self.implied(Nes::tsx),
            0xBB => self.absolute_y(Nes::las),
            0xBC => self.absolute_x(Nes::ldy),
            0xBD => self.absolute_x(Nes::lda),
            0xBE => self.absolute_y(Nes::ldx),
            0xBF => self.absolute_y(Nes::lax),
            0xC0 => self.immediate(Nes::cpy),
            0xC1 => self.indirect_x(Nes::cmp),
            0xC2 => self.immediate(|_, _| ()),
            0xC3 => self.indirect_x_illegal(Nes::dcp),
            0xC4 => self.zero_page(Nes::cpy),
            0xC5 => self.zero_page(Nes::cmp),
            0xC6 => self.zero_page_rmw(Nes::dec),
            0xC7 => self.zero_page_rmw(Nes::dcp),
            0xC8 => self.implied(Nes::iny),
            0xC9 => self.immediate(Nes::cmp),
            0xCA => self.implied(Nes::dex),
            0xCB => self.immediate(Nes::axs),
            0xCC => self.absolute(Nes::cpy),
            0xCD => self.absolute(Nes::cmp),
            0xCE => self.absolute_rmw(Nes::dec),
            0xCF => self.absolute_rmw(Nes::dcp),
            0xD0 => self.relative(!self.cpu.z),
            0xD1 => self.indirect_y(Nes::cmp),
            0xD2 => self.immediate(Nes::halt),
            0xD3 => self.indirect_y_illegal(Nes::dcp),
            0xD4 => self.zero_page_x(|_, _| ()),
            0xD5 => self.zero_page_x(Nes::cmp),
            0xD6 => self.zero_page_x_rmw(Nes::dec),
            0xD7 => self.zero_page_x_rmw(Nes::dcp),
            0xD8 => self.implied(Nes::cld),
            0xD9 => self.absolute_y(Nes::cmp),
            0xDA => self.implied(|_| ()),
            0xDB => self.absolute_y_illegal(Nes::dcp),
            0xDC => self.absolute_x(|_, _| ()),
            0xDD => self.absolute_x(Nes::cmp),
            0xDE => self.absolute_x_rmw(Nes::dec),
            0xDF => self.absolute_x_rmw(Nes::dcp),
            0xE0 => self.immediate(Nes::cpx),
            0xE1 => self.indirect_x(Nes::sbc),
            0xE2 => self.immediate(|_, _| ()),
            0xE3 => self.indirect_x_illegal(Nes::isc),
            0xE4 => self.zero_page(Nes::cpx),
            0xE5 => self.zero_page(Nes::sbc),
            0xE6 => self.zero_page_rmw(Nes::inc),
            0xE7 => self.zero_page_rmw(Nes::isc),
            0xE8 => self.implied(Nes::inx),
            0xE9 => self.immediate(Nes::sbc),
            0xEA => self.implied(|_| ()),
            0xEB => self.immediate(Nes::sbc),
            0xEC => self.absolute(Nes::cpx),
            0xED => self.absolute(Nes::sbc),
            0xEE => self.absolute_rmw(Nes::inc),
            0xEF => self.absolute_rmw(Nes::isc),
            0xF0 => self.relative(self.cpu.z),
            0xF1 => self.indirect_y(Nes::sbc),
            0xF2 => self.immediate(Nes::halt),
            0xF3 => self.indirect_y_illegal(Nes::isc),
            0xF4 => self.zero_page_x(|_, _| ()),
            0xF5 => self.zero_page_x(Nes::sbc),
            0xF6 => self.zero_page_x_rmw(Nes::inc),
            0xF7 => self.zero_page_x_rmw(Nes::isc),
            0xF8 => self.implied(Nes::sed),
            0xF9 => self.absolute_y(Nes::sbc),
            0xFA => self.implied(|_| ()),
            0xFB => self.absolute_y_illegal(Nes::isc),
            0xFC => self.absolute_x(|_, _| ()),
            0xFD => self.absolute_x(Nes::sbc),
            0xFE => self.absolute_x_rmw(Nes::inc),
            0xFF => self.absolute_x_rmw(Nes::isc),
        };

        self.load_next_instruction();
        self.clock_components();
    }
}

// Addressing modes with proper timings

impl Nes {
    #[inline]
    fn implied(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        last_cycle!(self);
        op_instruction(self);

        self.clock_components();
    }

    #[inline]
    fn immediate(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        last_cycle!(self);
        op_instruction(self, self.cpu.db);

        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn accumulator(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        last_cycle!(self);
        op_instruction(self);

        self.clock_components();
    }

    #[inline]
    fn zero_page(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        penultimate_cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_x(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        //Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.x as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_y(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        //Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.y as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        penultimate_cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_x_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.x as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_y_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.y as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 2
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 3
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn zero_page_x_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.x as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 3
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 4
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn relative(&mut self, branch: bool) {
        // Cycle 0
        last_cycle!(self);

        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        if !branch {
            return;
        }

        // Cycle 1b
        // https://wiki.nesdev.org/w/index.php?title=CPU_interrupts#Branch_instructions_and_interrupts
        penultimate_cycle!(self);

        self.take_branch();
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        if self.cpu.temp == 0 {
            return;
        }

        // Cycle 2b
        last_cycle!(self);

        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_jmp(&mut self) {
        // Cycle 0
        penultimate_cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        last_cycle!(self);
        self.cpu.pc = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 3
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 4
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_x(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab =
            ((self.cpu.db as u16) << 8) | ((self.cpu.temp as u16 + self.cpu.x as u16) & 0xFF);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        if (self.cpu.temp + self.cpu.x as u16) >= 0x100 {
            // Cycle extra if page boundary was crossed
            penultimate_cycle!(self);
            self.cpu.ab = (self.cpu.ab).wrapping_add(0x100);

            self.clock_components();
        }

        // Cycle 2
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_y(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab =
            ((self.cpu.db as u16) << 8) | ((self.cpu.temp as u16 + self.cpu.y as u16) & 0xFF);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        if (self.cpu.temp as u16 + self.cpu.y as u16) >= 0x100 {
            // Cycle extra if page boundary was crossed
            penultimate_cycle!(self);
            self.cpu.ab = (self.cpu.ab).wrapping_add(0x100);

            self.clock_components();
        }

        // Cycle 2
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_x_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.x as u16) & 0xFF);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        penultimate_cycle!(self);
        if (self.cpu.temp + self.cpu.x as u16) >= 0x100 {
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100);
        };

        self.clock_components();

        // Cycle 3
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_y_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.y as u16) & 0xFF);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        penultimate_cycle!(self);
        if (self.cpu.temp + self.cpu.y as u16) >= 0x100 {
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100);
        };

        self.clock_components();

        // Cycle 3
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_y_illegal(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.y as u16) & 0xFF);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        cycle!(self);
        if (self.cpu.temp + self.cpu.y as u16) >= 0x100 {
            self.cpu.ab = (self.cpu.ab).wrapping_add(0x100);
        };

        self.clock_components();

        // Cycle 3
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 4
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 5
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect(&mut self) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;

        self.clock_components();

        // Cycle 2
        penultimate_cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab & 0xFF00) | ((self.cpu.ab + 1) & 0xFF);

        self.clock_components();

        // Cycle 3
        last_cycle!(self);
        self.cpu.pc = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect_x(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.x as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab + 1) & 0xFF;

        self.clock_components();

        // Cycle 3
        penultimate_cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;

        self.clock_components();

        // Cycle 4
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect_x_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.x as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab + 1) & 0xFF;

        self.clock_components();

        // Cycle 3
        penultimate_cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;

        self.clock_components();

        // Cycle 4
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect_x_illegal(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = (self.cpu.ab + self.cpu.x as u16) & 0xFF;

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab + 1) & 0xFF;

        self.clock_components();

        // Cycle 3
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;

        self.clock_components();

        // Cycle 4
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 5
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 6
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect_y(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        //Cycle 1
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab + 1u16) & 0xFF;

        self.clock_components();

        //Cycle 2
        penultimate_cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.y as u16) & 0xFF);

        self.clock_components();

        if (self.cpu.temp + self.cpu.y as u16) >= 0x100 {
            //Cycle extra if page boundary was crossed
            penultimate_cycle!(self);
            self.cpu.ab = (self.cpu.ab).wrapping_add(0x100);

            self.clock_components();
        }

        //Cycle 4
        last_cycle!(self);
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect_y_illegal(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab + 1) & 0xFF;

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.y as u16) & 0xFF);

        self.clock_components();

        // Cycle 3
        cycle!(self);
        if (self.cpu.temp + self.cpu.y as u16) >= 0x100 {
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100);
        };

        self.clock_components();

        // Cycle 4
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 5
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 6
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn indirect_y_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.ab = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab = (self.cpu.ab + 1) & 0xFF;

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.y as u16) & 0xFF);

        self.clock_components();

        // Cycle 3
        penultimate_cycle!(self);
        if self.cpu.temp + self.cpu.y as u16 >= 0x100 {
            self.cpu.ab = (self.cpu.ab).wrapping_add(0x100);
        }

        self.clock_components();

        // Cycle 4
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn absolute_x_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.ab = ((self.cpu.db as u16) << 8) | ((self.cpu.temp + self.cpu.x as u16) & 0xFF);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);

        self.clock_components();

        // Cycle 2
        cycle!(self);
        if (self.cpu.temp + self.cpu.x as u16) >= 0x100 {
            self.cpu.ab = (self.cpu.ab).wrapping_add(0x100);
        };

        self.clock_components();

        // Cycle 3
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;

        self.clock_components();

        // Cycle 4
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_components();

        // Cycle 5
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn jsr(&mut self) {
        // Cycle 0
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);

        self.clock_components();

        // Cycle 2
        self.cpu_write(self.cpu.ab as usize, (self.cpu.pc >> 8) as u8);
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);

        self.clock_components();

        // Cycle 3
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab as usize, (self.cpu.pc & 0xFF) as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 4
        last_cycle!(self);
        self.cpu.pc = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn brk(&mut self) {
        // Cycle 0
        cycle!(self);
        let int = if self.cpu.take_interrupt { 0 } else { 1 };
        self.cpu.pc += int;
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);

        self.clock_components();

        // Cycle 1
        if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
            self.cpu_write(self.cpu.ab as usize, (self.cpu.pc >> 8) as u8);
        }
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);

        self.clock_components();

        // Cycle 2
        if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
            self.cpu_write(self.cpu.ab as usize, (self.cpu.pc & 0xFF) as u8);
        }
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);

        self.clock_components();

        // Cycle 3
        if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
            self.push_status(true);
        }
        self.cpu.ab = self.interrupt_address();
        self.cpu.take_interrupt = false;
        self.cpu.interrupt_type = InterruptType::None;

        self.clock_components();

        // Cycle 4
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.ab += 1;
        self.cpu.i = true;

        self.clock_components();

        // Cycle 5
        cycle!(self);
        self.cpu.pc = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn rti(&mut self) {
        // Cycle 0
        cycle!(self);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);

        self.sp_to_ab();

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.pull_status(self.cpu.db);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);

        self.sp_to_ab();

        self.clock_components();

        // Cycle 3
        penultimate_cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);

        self.sp_to_ab();

        self.clock_components();

        // Cycle 4
        last_cycle!(self);
        self.cpu.pc = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn rts(&mut self) {
        // Cycle 0
        cycle!(self);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 1
        cycle!(self);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 2
        cycle!(self);
        self.cpu.temp = self.cpu.db as u16;
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 3
        penultimate_cycle!(self);
        self.cpu.pc = ((self.cpu.db as u16) << 8) | self.cpu.temp as u16;
        self.cpu.ab = self.cpu.pc;

        self.clock_components();

        // Cycle 4

        last_cycle!(self);
        self.cpu.pc = (self.cpu.pc).wrapping_add(1);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn pha(&mut self) {
        // Cycle 0
        penultimate_cycle!(self);
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);
        self.clock_components();

        // Cycle 1
        self.check_interrupts();
        self.cpu_write(self.cpu.ab as usize, self.cpu.a);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn php(&mut self) {
        // Cycle 0
        penultimate_cycle!(self);
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1);
        self.clock_components();

        // Cycle 1
        self.check_interrupts();
        self.push_status(true);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn pla(&mut self) {
        // Cycle 0
        cycle!(self);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 2
        last_cycle!(self);
        self.lda(self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }

    #[inline]
    fn plp(&mut self) {
        // Cycle 0
        cycle!(self);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 1
        penultimate_cycle!(self);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1);
        self.sp_to_ab();

        self.clock_components();

        // Cycle 2
        last_cycle!(self);
        self.pull_status(self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_components();
    }
}

// The instruction logic without timings

impl Nes {
    #[inline]
    fn adc(&mut self, num: u8) {
        let a = self.cpu.a;
        let b = num;
        let carry = (u16::from(num) + u16::from(self.cpu.a) + (if self.cpu.c { 1 } else { 0 }))
            & (1 << 8)
            != 0;
        let num: i8 = (num as i8).wrapping_add(if self.cpu.c { 1 } else { 0 });
        let num: i8 = (num as i8).wrapping_add(self.cpu.a as i8);
        self.cpu.a = num as u8;
        self.cpu.v = (a ^ b) & (0x80) == 0 && (a ^ self.cpu.a) & 0x80 != 0;
        self.cpu.c = carry;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn sbc(&mut self, num: u8) {
        self.adc(!num);
    }

    #[inline]
    fn asl(&mut self, mut num: u8) {
        self.cpu.c = num & (1 << 7) != 0;
        num <<= 1;
        self.set_z_n(num);
        self.cpu.temp = num as u16;
    }

    #[inline]
    fn asl_a(&mut self) {
        self.cpu.c = self.cpu.a & (1 << 7) != 0;
        self.cpu.a <<= 1;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn lsr(&mut self, mut num: u8) {
        self.cpu.c = (num & 1) != 0;
        num >>= 1;
        self.set_z_n(num);
        self.cpu.temp = num as u16;
    }

    #[inline]
    fn lsr_a(&mut self) {
        self.cpu.c = (self.cpu.a & 1) != 0;
        self.cpu.a >>= 1;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn rol(&mut self, mut num: u8) {
        let c = self.cpu.c;
        self.cpu.c = num & (1 << 7) != 0;
        num <<= 1;
        num |= if c { 1 } else { 0 };
        self.set_z_n(num);
        self.cpu.temp = num as u16;
    }

    #[inline]
    fn rol_a(&mut self) {
        let c = self.cpu.c;
        self.cpu.c = self.cpu.a & (1 << 7) != 0;
        self.cpu.a <<= 1;
        self.cpu.a |= if c { 1 } else { 0 };
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn ror(&mut self, mut num: u8) {
        let c = self.cpu.c;
        self.cpu.c = num & 1 != 0;
        num >>= 1;
        num |= if c { 1 } else { 0 } << 7;
        self.set_z_n(num);
        self.cpu.temp = num as u16;
    }

    #[inline]
    fn ror_a(&mut self) {
        let c = self.cpu.c;
        self.cpu.c = self.cpu.a & 1 != 0;
        self.cpu.a >>= 1;
        self.cpu.a |= if c { 1 } else { 0 } << 7;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn inc(&mut self, mut num: u8) {
        num = num.wrapping_add(1);
        self.set_z_n(num);
        self.cpu.temp = num as u16;
    }

    #[inline]
    fn inx(&mut self) {
        self.cpu.x = self.cpu.x.wrapping_add(1);
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn iny(&mut self) {
        self.cpu.y = self.cpu.y.wrapping_add(1);
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    fn dec(&mut self, mut num: u8) {
        num = num.wrapping_sub(1);
        self.set_z_n(num);
        self.cpu.temp = num as u16;
    }

    #[inline]
    fn dex(&mut self) {
        self.cpu.x = self.cpu.x.wrapping_sub(1);
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn dey(&mut self) {
        self.cpu.y = self.cpu.y.wrapping_sub(1);
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    fn and(&mut self, num: u8) {
        self.cpu.a &= num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn eor(&mut self, num: u8) {
        self.cpu.a ^= num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn ora(&mut self, num: u8) {
        self.cpu.a |= num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn cmp(&mut self, num: u8) {
        self.compare(num, self.cpu.a);
    }

    #[inline]
    fn cpx(&mut self, num: u8) {
        self.compare(num, self.cpu.x);
    }

    #[inline]
    fn cpy(&mut self, num: u8) {
        self.compare(num, self.cpu.y);
    }

    #[inline]
    fn bit(&mut self, byte: u8) {
        self.cpu.z = (byte & self.cpu.a) == 0;
        self.cpu.v = (byte >> 6) & 1 != 0;
        self.cpu.n = (byte >> 7) & 1 != 0;
    }

    #[inline]
    fn clc(&mut self) {
        self.cpu.c = false;
    }

    #[inline]
    fn sec(&mut self) {
        self.cpu.c = true;
    }

    #[inline]
    fn cli(&mut self) {
        self.cpu.i = false;
    }

    #[inline]
    fn sei(&mut self) {
        self.cpu.i = true;
    }

    #[inline]
    fn cld(&mut self) {
        self.cpu.d = false;
    }

    #[inline]
    fn sed(&mut self) {
        self.cpu.d = true;
    }

    #[inline]
    fn clv(&mut self) {
        self.cpu.v = false;
    }

    #[inline]
    fn lda(&mut self, num: u8) {
        self.cpu.a = num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn ldx(&mut self, num: u8) {
        self.cpu.x = num;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn ldy(&mut self, num: u8) {
        self.cpu.y = num;
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    fn sta(&mut self) {
        self.cpu_write(self.cpu.ab as usize, self.cpu.a);
    }

    #[inline]
    fn stx(&mut self) {
        self.cpu_write(self.cpu.ab as usize, self.cpu.x);
    }

    #[inline]
    fn sty(&mut self) {
        self.cpu_write(self.cpu.ab as usize, self.cpu.y);
    }

    #[inline]
    fn tax(&mut self) {
        self.cpu.x = self.cpu.a;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn tay(&mut self) {
        self.cpu.y = self.cpu.a;
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    fn tsx(&mut self) {
        self.cpu.x = self.cpu.sp as u8;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn txa(&mut self) {
        self.cpu.a = self.cpu.x;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn txs(&mut self) {
        self.cpu.sp = self.cpu.x;
    }

    #[inline]
    fn tya(&mut self) {
        self.cpu.a = self.cpu.y;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn aax(&mut self, _: u8) {
        let res = self.cpu.x & self.cpu.a;
        self.cpu_write(self.cpu.ab as usize, res);
    }

    #[inline]
    fn dcp(&mut self, _: u8) {
        self.dec(self.cpu.temp as u8);
        self.cmp(self.cpu.temp as u8);
    }

    #[inline]
    fn isc(&mut self, _: u8) {
        self.inc(self.cpu.temp as u8);
        self.sbc(self.cpu.temp as u8);
    }

    #[inline]
    fn lax(&mut self, num: u8) {
        self.cpu.a = num;
        self.cpu.x = self.cpu.a;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn rla(&mut self, _: u8) {
        self.rol(self.cpu.temp as u8);
        self.and(self.cpu.temp as u8);
    }

    #[inline]
    fn rra(&mut self, _: u8) {
        self.ror(self.cpu.temp as u8);
        self.adc(self.cpu.temp as u8);
    }

    #[inline]
    fn slo(&mut self, _: u8) {
        self.asl(self.cpu.temp as u8);
        self.ora(self.cpu.temp as u8);
    }

    #[inline]
    fn sre(&mut self, _: u8) {
        self.lsr(self.cpu.temp as u8);
        self.eor(self.cpu.temp as u8);
    }

    #[inline]
    fn anc(&mut self, num: u8) {
        self.and(num);
        self.cpu.c = self.cpu.n;
    }

    #[inline]
    fn alr(&mut self, num: u8) {
        self.cpu.a &= num;
        self.cpu.c = self.cpu.a & 1 != 0;
        self.cpu.a >>= 1;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn axs(&mut self, num: u8) {
        self.cpu.x &= self.cpu.a;
        self.cpu.c = self.cpu.x >= num;
        self.cpu.x = self.cpu.x.wrapping_sub(num);
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    fn xaa(&mut self, num: u8) {
        self.cpu.a = (self.cpu.a | 0xEE) & self.cpu.x & num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn ahx(&mut self) {
        let result = self.cpu.a & self.cpu.x & ((self.cpu.ab >> 8) + 1) as u8;
        self.cpu_write(self.cpu.ab as usize, result);
    }

    #[inline]
    fn shx(&mut self) {
        let result = ((self.cpu.ab >> 8) as u8).wrapping_add(1) & self.cpu.x;
        self.cpu_write(
            (usize::from(result) << 8) | (self.cpu.ab as usize & 0xFF),
            self.cpu.x,
        );
    }

    #[inline]
    fn shy(&mut self) {
        let result = ((self.cpu.ab >> 8) as u8).wrapping_add(1) & self.cpu.y;
        self.cpu_write(
            (usize::from(result) << 8) | (self.cpu.ab as usize & 0xFF),
            self.cpu.y,
        );
    }

    #[inline]
    fn las(&mut self, val: u8) {
        self.cpu.sp &= val;
        self.cpu.a = self.cpu.sp as u8;
        self.cpu.x = self.cpu.sp as u8;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    fn tas(&mut self) {
        self.cpu.sp = self.cpu.a & self.cpu.x;
        let result = self.cpu.sp as u16 & ((self.cpu.ab >> 8) + 1);
        self.cpu_write(self.cpu.ab as usize, result as u8);
    }

    #[inline]
    fn arr(&mut self, num: u8) {
        self.and(num);
        self.ror_a();

        self.cpu.c = (self.cpu.a >> 6) & 1 == 1;
        self.cpu.v = self.cpu.c != ((self.cpu.a >> 5) & 1 == 1);
    }

    fn halt(&mut self, _: u8) {
        // TODO: shouldn't panic here
        panic!("The CPU executed a halt instruction, this implies either an emulator bug, or a game bug.");
    }
}

// Helper functions
impl Nes {
    #[inline]
    fn load_next_instruction(&mut self) {
        self.cache_interrupts();
        let int = if self.cpu.take_interrupt { 0 } else { 1 };
        self.cpu_read(self.cpu.ab as usize);
        check_read_hijack!(self);
        self.cpu.current_instruction = int * self.cpu.db;
        self.cpu.pc = (self.cpu.pc).wrapping_add(int as u16);
        self.cpu.ab = self.cpu.pc
    }

    #[inline]
    fn cache_interrupts(&mut self) {
        self.cpu.cached_irq = self.cpu.irq_mapper_signal || self.cpu.irq_apu_signal;
        self.cpu.cached_nmi = self.cpu.nmi_signal;
    }

    #[inline]
    fn check_interrupts(&mut self) {
        if !self.cpu.i && self.cpu.cached_irq {
            self.cpu.cached_irq = false;
            self.cpu.take_interrupt = true;
            self.cpu.interrupt_type = InterruptType::Irq;
        }

        if self.cpu.cached_nmi {
            self.cpu.cached_nmi = false;
            self.cpu.nmi_signal = false;
            self.cpu.take_interrupt = true;
            self.cpu.interrupt_type = InterruptType::Nmi;
        }
    }

    #[inline]
    fn interrupt_address(&mut self) -> u16 {
        // https://wiki.nesdev.org/w/index.php?title=CPU_interrupts#Interrupt_hijacking
        // For example, if NMI is asserted during the first four ticks of a BRK instruction,
        // the BRK instruction will execute normally at first (PC increments will occur and
        // the status word will be pushed with the B flag set), but execution will branch to
        // the NMI vector instead of the IRQ/BRK vector
        if self.cpu.nmi_signal {
            return 0xFFFA;
        }

        match self.cpu.interrupt_type {
            InterruptType::Irq | InterruptType::None => 0xFFFE,
            InterruptType::Nmi => 0xFFFA,
            InterruptType::Reset => 0xFFFC,
        }
    }

    #[inline]
    fn sp_to_ab(&mut self) {
        self.cpu.ab = self.cpu.sp as u16 | 0x100;
    }

    #[inline]
    fn compare(&mut self, num: u8, mut reg: u8) {
        self.cpu.c = reg >= num;
        reg = reg.wrapping_sub(num);
        self.cpu.z = reg == 0;
        self.cpu.n = reg & (1 << 7) != 0;
    }

    #[inline]
    fn take_branch(&mut self) {
        // Truncating to 8 bits is important !
        let diff = self.cpu.temp as i8 as i16;
        let pc_before = self.cpu.pc;
        if diff > 0 {
            self.cpu.pc = (self.cpu.pc).wrapping_add(diff as u16);
        } else {
            self.cpu.pc = (self.cpu.pc).wrapping_sub(diff.unsigned_abs());
        };
        let crosses = (pc_before & 0xFF00) != (self.cpu.pc & 0xFF00);
        self.cpu.temp = if crosses { 1 } else { 0 };
    }

    #[inline]
    fn push_status(&mut self, brk_php: bool) {
        let mut status: u8 = 1 << 5;
        status |= (if self.cpu.n { 1 } else { 0 }) << 7;
        status |= (if self.cpu.v { 1 } else { 0 }) << 6;
        status |= (if brk_php { 1 } else { 0 }) << 4;
        status |= (if self.cpu.d { 1 } else { 0 }) << 3;
        status |= (if self.cpu.i { 1 } else { 0 }) << 2;
        status |= (if self.cpu.z { 1 } else { 0 }) << 1;
        status |= if self.cpu.c { 1 } else { 0 };
        self.cpu_write(self.cpu.ab as usize, status);
    }

    #[inline]
    fn pull_status(&mut self, status: u8) {
        self.cpu.n = status >> 7 != 0;
        self.cpu.v = (status >> 6) & 1 != 0;
        self.cpu.d = (status >> 3) & 1 != 0;
        self.cpu.i = (status >> 2) & 1 != 0;
        self.cpu.z = (status >> 1) & 1 != 0;
        self.cpu.c = status & 1 != 0;
    }

    #[inline]
    fn set_z_n(&mut self, num: u8) {
        self.cpu.z = num == 0;
        self.cpu.n = (num & 0b1000_0000) != 0;
    }
}
