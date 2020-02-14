use super::Nes;

enum InterruptType {
    Nmi,
    Irq,
    Reset,
    None,
}

struct Dma {
    cycles: u16,
    oam: bool,
    dmc: bool,
    hijack_read: bool,
    copy_buffer: u8,
    addr: usize,
}

impl Dma {
    pub fn new() -> Dma {
        Dma {
            cycles: 0,
            oam: false,
            dmc: false,
            hijack_read: false,
            copy_buffer: 0,
            addr: 0,
        }
    }
}

pub struct Cpu {
    a: u8,         // Accumulator
    x: u8,         // X index
    y: u8,         // Y index
    pub pc: usize, // Program counter (16 bits)
    pub sp: usize, // Stack pointer (8 bits)

    n: bool, // Negative flag
    v: bool, // Overflow flag
    i: bool, // Interrupt inhibit
    z: bool, // Zero flag
    c: bool, // Carry flag
    d: bool, // BCD flag, this doesn't do anything on the NES CPU

    pub halt: bool,
    pub state: u16,
    odd_cycle: bool,

    pub ab: usize, //Address bus
    db: u8,        //Data bus
    temp: usize,
    pub open_bus: u8,

    cached_irq: bool,
    pub irq_signal: bool,
    cached_nmi: bool,
    pub nmi_signal: bool,
    reset_signal: bool,
    take_interrupt: bool,
    interrupt_type: InterruptType,

    ram: Vec<u8>,
    dma: Dma,
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

            halt: false,
            state: 0x100,
            odd_cycle: false,

            ab: 0,
            db: 0,
            temp: 0,

            cached_irq: false,
            irq_signal: false,
            cached_nmi: false,
            nmi_signal: false,
            reset_signal: false,
            take_interrupt: false,
            interrupt_type: InterruptType::None,

            ram: vec![0; 0x800],
            dma: Dma::new(),
            open_bus: 0,
        }
    }
}

impl Nes {
    pub(crate) fn cpu_debug_info(&mut self) -> String {
        format!(
            "A: 0x{:X}, X: 0x{:X}, Y: 0x{:X}, pc: 0x{:X}, sp: 0x{:X}, ab: 0x{:X}",
            self.cpu.a, self.cpu.x, self.cpu.y, self.cpu.pc, self.cpu.sp, self.cpu.ab
        )
    }

    pub(crate) fn cpu_gen_reset(&mut self) {
        self.cpu.state = 0;
        self.cpu.take_interrupt = true;
        self.cpu.reset_signal = true;
        self.cpu.interrupt_type = InterruptType::Reset;
        self.cpu_write(0x4015, 0);
        self.cpu.reset_signal = false;
    }

    pub(crate) fn cpu_reset_routine(&mut self) {
        //self.cache_interrupts();
        //let int = if self.cpu.take_interrupt { 0 } else { 1 };
        //self.cpu_read(self.cpu.ab);
        //self.check_dma();
        //self.cpu.state = u16::from(int * self.cpu.db);
        //self.cpu.pc = (self.cpu.pc as u16).wrapping_add(int as u16) as usize;
        //self.cpu.ab = self.cpu.pc;

        //self.clock_ppu_apu();

        self.cpu.state = 0;
        self.cpu_tick_new();
    }

    #[inline]
    pub(crate) fn cpu_read(&mut self, index: usize) {
        self.cpu.open_bus = match index {
            0x4020..=0xFFFF => (self.mapper.cpu_read)(self, index),
            0..=0x1FFF => self.cpu.ram[index & 0x7FF],
            0x2000..=0x3FFF => self.ppu_read_reg(index),
            0x4000..=0x4014 | 0x4017..=0x401F => self.cpu.open_bus,
            0x4016 => {
                let tmp = self.controller.read_reg();
                (self.cpu.open_bus & 0xE0) | tmp
            }
            0x4015 => self.apu_read_status(),
            _ => unreachable!("memory access into unmapped address: 0x{:X}", index),
        };

        self.cpu.db = self.cpu.open_bus;
    }

    #[inline]
    pub(crate) fn cpu_write(&mut self, index: usize, val: u8) {
        match index {
            0..=0x1FFF => self.cpu.ram[index & 0x7FF] = val,
            0x2000..=0x3FFF => self.ppu_write_reg(index, val),
            0x4000..=0x4013 => self.apu_write_reg(index, val),
            0x4014 => {
                self.cpu.dma.oam = true;
                self.cpu.dma.addr = (val as usize) << 8;
            }
            0x4015 => self.apu_write_reg(index, val),
            0x4016 => self.controller.write_reg(val),
            0x4017 => self.apu_write_reg(index, val),
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => (self.mapper.cpu_write)(self, index, val),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    #[inline]
    pub(crate) fn cpu_peek(&mut self, index: usize) -> u8 {
        match index {
            0..=0x1FFF => self.cpu.ram[index & 0x7FF],
            0x4020..=0xFFFF => (self.mapper.cpu_peek)(self, index),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    //forums.nesdev.com/viewtopic.php?f=3&t=14120
    #[inline]
    pub(crate) fn cpu_dma(&mut self) {
        if self.cpu.dma.cycles == 0 {
            self.cpu.dma.hijack_read = true;
        } else {
            self.cpu.dma.hijack_read = false;
            if self.cpu.dma.oam {
                if self.cpu.dma.cycles == 1 && self.cpu.odd_cycle {
                    self.cpu_read(self.cpu.ab);
                } else {
                    if self.cpu.dma.cycles & 1 != 0 {
                        self.cpu_read(self.cpu.dma.addr);
                        self.cpu.dma.addr += 1;
                        self.cpu.dma.copy_buffer = self.cpu.db
                    } else {
                        self.cpu_write(0x2004, self.cpu.dma.copy_buffer);
                    }
                    self.cpu.dma.cycles += 1;

                    if self.cpu.dma.cycles == 0x201 {
                        self.cpu.dma.oam = false;
                        self.cpu.dma.cycles = 0;
                    }
                }
            }
            if self.cpu.dma.dmc {
                unimplemented!("DMC DMA is unimplemented");
            }
        }
    }
}

/* Most of the documentation for the 6502 can be found on nesdev:
http://nesdev.com/6502_cpu.txt
However, a few illegal instructions (LAX and XAA) are basically undefined.
Some information about those can be found in the visual6502.org wiki:
http://visual6502.org/wiki/index.php?title=6502_Unsupported_Opcodes
The 6502 has many quirks, some of them (such as the branch behavior) are
described on visual6502.org wiki. Some of them are described in numerous
random documents found in the hidden corners of the internet.*/

impl Nes {
    pub(crate) fn cpu_tick_new(&mut self) {
        //println!("OP: 0x{:X}", self.cpu.state);

        match self.cpu.state {
            0x00 => self.brk(),
            0x01 => self.indirect_x(Nes::ora),
            0x02 => self.immediate(Nes::halt),
            //0x03 => (IndirectXIllegal, "self.cpu_slo();"),
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
            //0x0E => (AbsoluteRmw, "self.cpu_asl(val);"),
            //0x0F => (AbsoluteRmw, "self.cpu_slo();"),
            0x10 => self.relative(!self.cpu.n),
            0x11 => self.indirect_y(Nes::ora),
            0x12 => self.immediate(Nes::halt),
            //0x13 => (IndirectYIllegal, "self.cpu_slo();"),
            0x14 => self.zero_page_x(|_, _| ()),
            0x15 => self.zero_page_x(Nes::ora),
            0x16 => self.zero_page_x_rmw(Nes::asl),
            0x17 => self.zero_page_x_rmw(Nes::slo),
            0x18 => self.implied(Nes::clc),
            0x19 => self.absolute_y(Nes::ora),
            0x1A => self.implied(|_| ()),
            //0x1B => (AbsoluteYIllegal, "self.cpu_slo();"),
            0x1C => self.absolute_x(|_, _| ()),
            0x1D => self.absolute_x(Nes::ora),
            0x1E => self.absolute_x_rmw(Nes::asl),
            0x1F => self.absolute_x_rmw(Nes::slo),
            0x20 => self.jsr(),
            0x21 => self.indirect_x(Nes::and),
            0x22 => self.immediate(Nes::halt),
            //0x23 => (IndirectXIllegal, "self.cpu_rla();"),
            0x24 => self.zero_page(Nes::bit),
            0x25 => self.zero_page(Nes::and),
            0x26 => self.zero_page_rmw(Nes::rol),
            0x27 => self.zero_page_rmw(Nes::rla),
            //0x28 => (Plp, ""),
            0x29 => self.immediate(Nes::and),
            0x2A => self.accumulator(Nes::rol_a),
            0x2B => self.immediate(Nes::anc),
            0x2C => self.absolute(Nes::bit),
            0x2D => self.absolute(Nes::and),
            //0x2E => (AbsoluteRmw, "self.cpu_rol(val);"),
            //0x2F => (AbsoluteRmw, "self.cpu_rla();"),
            0x30 => self.relative(self.cpu.n),
            0x31 => self.indirect_y(Nes::and),
            0x32 => self.immediate(Nes::halt),
            //0x33 => (IndirectYIllegal, "self.cpu_rla();"),
            0x34 => self.zero_page_x(|_, _| ()),
            0x35 => self.zero_page_x(Nes::and),
            0x36 => self.zero_page_x_rmw(Nes::rol),
            0x37 => self.zero_page_x_rmw(Nes::rla),
            0x38 => self.implied(Nes::sec),
            0x39 => self.absolute_y(Nes::and),
            0x3A => self.implied(|_| ()),
            //0x3B => (AbsoluteYIllegal, "self.cpu_rla();"),
            0x3C => self.absolute_x(|_, _| ()),
            0x3D => self.absolute_x(Nes::and),
            0x3E => self.absolute_x_rmw(Nes::rol),
            0x3F => self.absolute_x_rmw(Nes::rla),
            0x40 => self.rti(),
            0x41 => self.indirect_x(Nes::eor),
            0x42 => self.immediate(Nes::halt),
            //0x43 => (IndirectXIllegal, "self.cpu_sre();"),
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
            //0x4E => (AbsoluteRmw, "self.cpu_lsr(val);"),
            //0x4F => (AbsoluteRmw, "self.cpu_sre();"),
            0x50 => self.relative(!self.cpu.v),
            0x51 => self.indirect_y(Nes::eor),
            0x52 => self.immediate(Nes::halt),
            //0x53 => (IndirectYIllegal, "self.cpu_sre();"),
            0x54 => self.zero_page_x(|_, _| ()),
            0x55 => self.zero_page_x(Nes::eor),
            0x56 => self.zero_page_x_rmw(Nes::lsr),
            0x57 => self.zero_page_x_rmw(Nes::sre),
            0x58 => self.implied(Nes::cli),
            0x59 => self.absolute_y(Nes::eor),
            0x5A => self.implied(|_| ()),
            //0x5B => (AbsoluteYIllegal, "self.cpu_sre();"),
            0x5C => self.absolute_x(|_, _| ()),
            0x5D => self.absolute_x(Nes::eor),
            0x5E => self.absolute_x_rmw(Nes::lsr),
            0x5F => self.absolute_x_rmw(Nes::sre),
            0x60 => self.rts(),
            0x61 => self.indirect_x(Nes::adc),
            0x62 => self.immediate(Nes::halt),
            //0x63 => (IndirectXIllegal, "self.cpu_rra();"),
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
            //0x6E => (AbsoluteRmw, "self.cpu_ror(val);"),
            //0x6F => (AbsoluteRmw, "self.cpu_rra();"),
            0x70 => self.relative(self.cpu.v),
            0x71 => self.indirect_y(Nes::adc),
            0x72 => self.immediate(Nes::halt),
            //0x73 => (IndirectYIllegal, "self.cpu_rra();"),
            0x74 => self.zero_page_x(|_, _| ()),
            0x75 => self.zero_page_x(Nes::adc),
            0x76 => self.zero_page_x_rmw(Nes::ror),
            0x77 => self.zero_page_x_rmw(Nes::rra),
            0x78 => self.implied(Nes::sei),
            0x79 => self.absolute_y(Nes::adc),
            0x7A => self.implied(|_| ()),
            //0x7B => (AbsoluteYIllegal, "self.cpu_rra();"),
            0x7C => self.absolute_x(|_, _| ()),
            0x7D => self.absolute_x(Nes::adc),
            0x7E => self.absolute_x_rmw(Nes::ror),
            0x7F => self.absolute_x_rmw(Nes::rra),
            0x80 => self.immediate(|_, _| ()),
            //0x81 => (IndirectXSt, "self.cpu_sta();"),
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
            //0x91 => (IndirectYSt, "self.cpu_sta();"),
            0x92 => self.immediate(Nes::halt),
            //0x93 => (IndirectYSt, "self.cpu_ahx();"),
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
            //0xC3 => (IndirectXIllegal, "self.cpu_dcp();"),
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
            //0xCE => (AbsoluteRmw, "self.cpu_dec(val);"),
            //0xCF => (AbsoluteRmw, "self.cpu_dcp();"),
            0xD0 => self.relative(!self.cpu.z),
            0xD1 => self.indirect_y(Nes::cmp),
            0xD2 => self.immediate(Nes::halt),
            //0xD3 => (IndirectYIllegal, "self.cpu_dcp();"),
            0xD4 => self.zero_page_x(|_, _| ()),
            0xD5 => self.zero_page_x(Nes::cmp),
            0xD6 => self.zero_page_x_rmw(Nes::dec),
            0xD7 => self.zero_page_x_rmw(Nes::dcp),
            0xD8 => self.implied(Nes::cld),
            0xD9 => self.absolute_y(Nes::cmp),
            0xDA => self.implied(|_| ()),
            //0xDB => (AbsoluteYIllegal, "self.cpu_dcp();"),
            0xDC => self.absolute_x(|_, _| ()),
            0xDD => self.absolute_x(Nes::cmp),
            0xDE => self.absolute_x_rmw(Nes::dec),
            0xDF => self.absolute_x_rmw(Nes::dcp),
            0xE0 => self.immediate(Nes::cpx),
            0xE1 => self.indirect_x(Nes::sbc),
            0xE2 => self.immediate(|_, _| ()),
            //0xE3 => (IndirectXIllegal, "self.cpu_isc();"),
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
            //0xEE => (AbsoluteRmw, "self.cpu_inc(val);"),
            //0xEF => (AbsoluteRmw, "self.cpu_isc();"),
            0xF0 => self.relative(self.cpu.z),
            0xF1 => self.indirect_y(Nes::sbc),
            0xF2 => self.immediate(Nes::halt),
            //0xF3 => (IndirectYIllegal, "self.cpu_isc();"),
            0xF4 => self.zero_page_x(|_, _| ()),
            0xF5 => self.zero_page_x(Nes::sbc),
            0xF6 => self.zero_page_x_rmw(Nes::inc),
            0xF7 => self.zero_page_x_rmw(Nes::isc),
            0xF8 => self.implied(Nes::sed),
            0xF9 => self.absolute_y(Nes::sbc),
            0xFA => self.implied(|_| ()),
            //0xFB => (AbsoluteYIllegal, "self.cpu_isc();"),
            0xFC => self.absolute_x(|_, _| ()),
            0xFD => self.absolute_x(Nes::sbc),
            0xFE => self.absolute_x_rmw(Nes::inc),
            0xFF => self.absolute_x_rmw(Nes::isc),
            opcode => unimplemented!("Opcode 0x{:X} is unimplemented", opcode),
        }

        self.load_next_instruction();

        self.clock_ppu_apu();
    }
}

// Addressing modes with proper timings

impl Nes {
    #[inline]
    pub(in cpu) fn implied(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.last_cycle();
        op_instruction(self);

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn immediate(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.last_cycle();
        op_instruction(self, self.cpu.db);

        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn accumulator(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.last_cycle();
        op_instruction(self);

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.penultimate_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_x(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        //Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 2
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_y(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        //Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 2
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.penultimate_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_x_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 2
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_y_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 2
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_ppu_apu();

        // Cycle 3
        self.check_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn zero_page_x_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 2
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;

        self.clock_ppu_apu();

        // Cycle 3
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_ppu_apu();

        // Cycle 4
        self.check_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn relative(&mut self, branch: bool) {
        // Cycle 0
        self.check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.check_dma();

        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        if !branch {
            // Cycle 1a
            self.load_next_instruction();

            self.clock_ppu_apu();
        }

        // Cycle 1b
        self.cache_interrupts();
        self.cpu.take_interrupt = false;
        self.cpu_read(self.cpu.ab);
        self.check_dma();

        self.take_branch();
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        if self.cpu.temp == 0 {
            // Cycle 2a
            self.load_next_instruction();

            self.clock_ppu_apu();
        }

        // Cycle 2b
        self.check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.check_dma();

        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn absolute(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn absolute_x(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8)
            | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
            // Cycle extra if page boundary was crossed
            self.penultimate_cycle();
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;

            self.clock_ppu_apu();
        }

        // Cycle 2
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn absolute_y(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8)
            | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
            // Cycle extra if page boundary was crossed
            self.cache_interrupts();
            self.cpu_read(self.cpu.ab);
            self.check_dma();
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;

            self.clock_ppu_apu();
        }

        // Cycle 2
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn absolute_x_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8)
            | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.penultimate_cycle();
        if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
            {
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
            }
        };

        self.clock_ppu_apu();

        // Cycle 3
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn absolute_y_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8)
            | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.penultimate_cycle();
        if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
            {
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
            }
        };

        self.clock_ppu_apu();

        // Cycle 3
        self.check_interrupts();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn indirect(&mut self) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;

        self.clock_ppu_apu();

        // Cycle 2
        self.penultimate_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.ab = (self.cpu.ab & 0xFF00) | ((self.cpu.ab + 1) & 0xFF);

        self.clock_ppu_apu();

        // Cycle 3
        self.last_cycle();
        self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.ab = self.cpu.pc;
    }

    #[inline]
    pub(in cpu) fn absolute_jmp(&mut self) {
        // Cycle 0
        self.penultimate_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.last_cycle();
        self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn indirect_x(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 2
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.ab = (self.cpu.ab + 1) & 0xFF;

        self.clock_ppu_apu();

        // Cycle 3
        self.penultimate_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;

        self.clock_ppu_apu();

        // Cycle 4
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn indirect_y(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        //Cycle 1
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;

        self.clock_ppu_apu();

        //Cycle 2
        self.penultimate_cycle();
        self.cpu.ab = (((self.cpu.db as usize) << 8)
            | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
            as usize;

        self.clock_ppu_apu();

        if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
            //Cycle extra if page boundary was crossed
            self.penultimate_cycle();
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;

            self.clock_ppu_apu();
        }

        //Cycle 3
        self.penultimate_cycle();
        self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;

        self.clock_ppu_apu();

        //Cycle 4
        self.last_cycle();
        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn absolute_st(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.last_cycle();
        op_instruction(self);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    // 5 "self.check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
    // self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;"

    #[inline]
    pub(in cpu) fn absolute_x_rmw(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.ab = ((self.cpu.db as usize) << 8)
            | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.start_cycle();
        if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
            self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
        };

        self.clock_ppu_apu();

        // Cycle 3
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;

        self.clock_ppu_apu();

        // Cycle 4
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
        op_instruction(self, self.cpu.temp as u8);

        self.clock_ppu_apu();

        // Cycle 5
        self.check_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn jsr(&mut self) {
        // Cycle 0
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        self.cpu_write(self.cpu.ab, (self.cpu.pc >> 8) as u8);
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 3
        self.cache_interrupts();
        self.cpu_write(self.cpu.ab, (self.cpu.pc & 0xFF) as u8);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 4
        self.last_cycle();
        self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn brk(&mut self) {
        // Cycle 0
        self.start_cycle();
        let int = if self.cpu.take_interrupt { 0 } else { 1 };
        self.cpu.pc += int;
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
            self.cpu_write(self.cpu.ab, (self.cpu.pc >> 8) as u8);
        }
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 2
        if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
            self.cpu_write(self.cpu.ab, (self.cpu.pc & 0xFF) as u8);
        }
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 3
        if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
            self.push_status(true);
        }
        self.cpu.ab = self.interrupt_address();
        self.cpu.take_interrupt = false;
        self.cpu.interrupt_type = InterruptType::None;

        self.clock_ppu_apu();

        // Cycle 4
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.ab += 1;
        self.cpu.i = true;

        self.clock_ppu_apu();

        // Cycle 5
        self.start_cycle();
        self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn rti(&mut self) {
        // Cycle 0
        self.start_cycle();
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 2
        self.start_cycle();
        self.pull_status(self.cpu.db);
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 3
        self.penultimate_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 4
        self.last_cycle();
        self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn rts(&mut self) {
        // Cycle 0
        self.start_cycle();
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 1
        self.start_cycle();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 2
        self.start_cycle();
        self.cpu.temp = self.cpu.db as usize;
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 3
        self.penultimate_cycle();
        self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 4

        self.last_cycle();
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn pha(&mut self) {
        // Cycle 0
        self.penultimate_cycle();
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.check_interrupts();
        self.cpu_write(self.cpu.ab, self.cpu.a);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn php(&mut self) {
        // Cycle 0
        self.penultimate_cycle();
        self.sp_to_ab();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.check_interrupts();
        self.push_status(true);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }

    #[inline]
    pub(in cpu) fn pla(&mut self) {
        // Cycle 0
        self.start_cycle();
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 1
        self.penultimate_cycle();
        self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
        self.sp_to_ab();

        self.clock_ppu_apu();

        // Cycle 2
        self.last_cycle();
        self.lda(self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();
    }
}

// The instruction logic without timings

impl Nes {
    #[inline]
    pub(in cpu) fn adc(&mut self, num: u8) {
        let a = self.cpu.a;
        let b = num;
        let carry =
            (u16::from(num) + u16::from(self.cpu.a) + (if self.cpu.c { 1 } else { 0 }))
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
    pub(in cpu) fn sbc(&mut self, num: u8) {
        self.adc(!num);
    }

    #[inline]
    pub(in cpu) fn asl(&mut self, mut num: u8) {
        self.cpu.c = num & (1 << 7) != 0;
        num <<= 1;
        self.set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub(in cpu) fn asl_a(&mut self) {
        self.cpu.c = self.cpu.a & (1 << 7) != 0;
        self.cpu.a <<= 1;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn lsr(&mut self, mut num: u8) {
        self.cpu.c = (num & 1) != 0;
        num >>= 1;
        self.set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub(in cpu) fn lsr_a(&mut self) {
        self.cpu.c = (self.cpu.a & 1) != 0;
        self.cpu.a >>= 1;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn rol(&mut self, mut num: u8) {
        let c = self.cpu.c;
        self.cpu.c = num & (1 << 7) != 0;
        num <<= 1;
        num |= if c { 1 } else { 0 };
        self.set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub(in cpu) fn rol_a(&mut self) {
        let c = self.cpu.c;
        self.cpu.c = self.cpu.a & (1 << 7) != 0;
        self.cpu.a <<= 1;
        self.cpu.a |= if c { 1 } else { 0 };
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn ror(&mut self, mut num: u8) {
        let c = self.cpu.c;
        self.cpu.c = num & 1 != 0;
        num >>= 1;
        num |= if c { 1 } else { 0 } << 7;
        self.set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub(in cpu) fn ror_a(&mut self) {
        let c = self.cpu.c;
        self.cpu.c = self.cpu.a & 1 != 0;
        self.cpu.a >>= 1;
        self.cpu.a |= if c { 1 } else { 0 } << 7;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn inc(&mut self, mut num: u8) {
        num = num.wrapping_add(1);
        self.set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub(in cpu) fn inx(&mut self) {
        self.cpu.x = self.cpu.x.wrapping_add(1);
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn iny(&mut self) {
        self.cpu.y = self.cpu.y.wrapping_add(1);
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    pub(in cpu) fn dec(&mut self, mut num: u8) {
        num = num.wrapping_sub(1);
        self.set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub(in cpu) fn dex(&mut self) {
        self.cpu.x = self.cpu.x.wrapping_sub(1);
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn dey(&mut self) {
        self.cpu.y = self.cpu.y.wrapping_sub(1);
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    pub(in cpu) fn and(&mut self, num: u8) {
        self.cpu.a &= num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn eor(&mut self, num: u8) {
        self.cpu.a ^= num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn ora(&mut self, num: u8) {
        self.cpu.a |= num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn cmp(&mut self, num: u8) {
        self.compare(num, self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn cpx(&mut self, num: u8) {
        self.compare(num, self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn cpy(&mut self, num: u8) {
        self.compare(num, self.cpu.y);
    }

    #[inline]
    pub(in cpu) fn bit(&mut self, byte: u8) {
        self.cpu.z = (byte & self.cpu.a) == 0;
        self.cpu.v = (byte >> 6) & 1 != 0;
        self.cpu.n = (byte >> 7) & 1 != 0;
    }

    #[inline]
    pub(in cpu) fn clc(&mut self) {
        self.cpu.c = false;
    }

    #[inline]
    pub(in cpu) fn sec(&mut self) {
        self.cpu.c = true;
    }

    #[inline]
    pub(in cpu) fn cli(&mut self) {
        self.cpu.i = false;
    }

    #[inline]
    pub(in cpu) fn sei(&mut self) {
        self.cpu.i = true;
    }

    #[inline]
    pub(in cpu) fn cld(&mut self) {
        self.cpu.d = false;
    }

    #[inline]
    pub(in cpu) fn sed(&mut self) {
        self.cpu.d = true;
    }

    #[inline]
    pub(in cpu) fn clv(&mut self) {
        self.cpu.v = false;
    }

    #[inline]
    pub(in cpu) fn lda(&mut self, num: u8) {
        self.cpu.a = num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn ldx(&mut self, num: u8) {
        self.cpu.x = num;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn ldy(&mut self, num: u8) {
        self.cpu.y = num;
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    pub(in cpu) fn sta(&mut self) {
        self.cpu_write(self.cpu.ab, self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn stx(&mut self) {
        self.cpu_write(self.cpu.ab, self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn sty(&mut self) {
        self.cpu_write(self.cpu.ab, self.cpu.y);
    }

    #[inline]
    pub(in cpu) fn tax(&mut self) {
        self.cpu.x = self.cpu.a;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn tay(&mut self) {
        self.cpu.y = self.cpu.a;
        self.set_z_n(self.cpu.y);
    }

    #[inline]
    pub(in cpu) fn tsx(&mut self) {
        self.cpu.x = self.cpu.sp as u8;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn txa(&mut self) {
        self.cpu.a = self.cpu.x;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn txs(&mut self) {
        self.cpu.sp = self.cpu.x as usize;
    }

    #[inline]
    pub(in cpu) fn tya(&mut self) {
        self.cpu.a = self.cpu.y;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn aax(&mut self, _: u8) {
        let res = self.cpu.x & self.cpu.a;
        self.cpu_write(self.cpu.ab, res);
    }

    #[inline]
    pub(in cpu) fn dcp(&mut self, val: u8) {
        self.dec(val);
        self.cmp(val);
    }

    #[inline]
    pub(in cpu) fn isc(&mut self, val: u8) {
        self.inc(val);
        self.sbc(val);
    }

    #[inline]
    pub(in cpu) fn lax(&mut self, num: u8) {
        self.cpu.a = num;
        self.cpu.x = self.cpu.a;
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn rla(&mut self, val: u8) {
        self.rol(val);
        self.and(val);
    }

    #[inline]
    pub(in cpu) fn rra(&mut self, val: u8) {
        self.ror(val);
        self.adc(val);
    }

    #[inline]
    pub(in cpu) fn slo(&mut self, val: u8) {
        self.asl(val);
        self.ora(val);
    }

    #[inline]
    pub(in cpu) fn sre(&mut self, val: u8) {
        self.lsr(val);
        self.eor(val);
    }

    #[inline]
    pub(in cpu) fn anc(&mut self, num: u8) {
        self.and(num);
        self.cpu.c = self.cpu.n;
    }

    #[inline]
    pub(in cpu) fn alr(&mut self, num: u8) {
        self.cpu.a &= num;
        self.cpu.c = self.cpu.a & 1 != 0;
        self.cpu.a >>= 1;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn axs(&mut self, num: u8) {
        self.cpu.x &= self.cpu.a;
        self.cpu.c = self.cpu.x >= num;
        self.cpu.x = self.cpu.x.wrapping_sub(num);
        self.set_z_n(self.cpu.x);
    }

    #[inline]
    pub(in cpu) fn xaa(&mut self, num: u8) {
        self.cpu.a = (self.cpu.a | 0xEE) & self.cpu.x & num;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn ahx(&mut self) {
        let result = self.cpu.a & self.cpu.x & ((self.cpu.ab >> 8) + 1) as u8;
        self.cpu_write(self.cpu.ab, result);
    }

    #[inline]
    pub(in cpu) fn shx(&mut self) {
        let result = ((self.cpu.ab >> 8) as u8).wrapping_add(1) & self.cpu.x;
        self.cpu_write(
            (usize::from(result) << 8) | (self.cpu.ab & 0xFF),
            self.cpu.x,
        );
    }

    #[inline]
    pub(in cpu) fn shy(&mut self) {
        let result = ((self.cpu.ab >> 8) as u8).wrapping_add(1) & self.cpu.y;
        self.cpu_write(
            (usize::from(result) << 8) | (self.cpu.ab & 0xFF),
            self.cpu.y,
        );
    }

    #[inline]
    pub(in cpu) fn las(&mut self, val: u8) {
        self.cpu.sp &= val as usize;
        self.cpu.a = self.cpu.sp as u8;
        self.cpu.x = self.cpu.sp as u8;
        self.set_z_n(self.cpu.a);
    }

    #[inline]
    pub(in cpu) fn tas(&mut self) {
        self.cpu.sp = (self.cpu.a & self.cpu.x) as usize;
        let result = self.cpu.sp & ((self.cpu.ab >> 8) + 1);
        self.cpu_write(self.cpu.ab, result as u8);
    }

    #[inline]
    pub(in cpu) fn arr(&mut self, num: u8) {
        self.and(num);
        self.ror_a();

        self.cpu.c = (self.cpu.a >> 6) & 1 == 1;
        self.cpu.v = self.cpu.c != ((self.cpu.a >> 5) & 1 == 1);
    }

    pub(in cpu) fn halt(&mut self, _: u8) {
        self.cpu.halt = true;
    }
}

// Helper functions
impl Nes {
    #[inline]
    pub(in cpu) fn load_next_instruction(&mut self) {
        self.cache_interrupts();
        let int: u8 = if self.cpu.take_interrupt { 0 } else { 1 };
        self.cpu_read(self.cpu.ab);
        self.check_dma();
        self.cpu.state = u16::from(int * self.cpu.db);
        self.cpu.pc = (self.cpu.pc).wrapping_add(int as usize);
        self.cpu.ab = self.cpu.pc
    }

    #[inline]
    pub(in cpu) fn cache_interrupts(&mut self) {
        self.cpu.cached_irq = self.cpu.irq_signal;
        self.cpu.cached_nmi = self.cpu.nmi_signal;
    }

    #[inline]
    pub(in cpu) fn check_interrupts(&mut self) {
        if !self.cpu.i && self.cpu.cached_irq {
            self.cpu.cached_irq = false;
            self.cpu.nmi_signal = false;
            self.cpu.take_interrupt = true;
            self.cpu.interrupt_type = InterruptType::Irq;
        }

        if self.cpu.cached_nmi {
            self.cpu.cached_nmi = false;
            self.cpu.nmi_signal = false;
            self.cpu.take_interrupt = true;
            self.cpu.interrupt_type = InterruptType::Nmi;
        }

        //TODO: resets
    }

    #[inline]
    pub(in cpu) fn interrupt_address(&mut self) -> usize {
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
    pub(in cpu) fn check_dma(&mut self) {
        if self.cpu.dma.hijack_read {
            self.cpu.dma.cycles = 1;
            return;
        }
    }

    #[inline]
    pub(in cpu) fn sp_to_ab(&mut self) {
        self.cpu.ab = self.cpu.sp | 0x100;
    }

    // Only if the instructions has 3 or more cycles (without next instruction fetch)
    #[inline]
    pub(in cpu) fn start_cycle(&mut self) {
        self.cpu_read(self.cpu.ab);
        self.check_dma();
    }

    #[inline]
    pub(in cpu) fn penultimate_cycle(&mut self) {
        self.cache_interrupts();
        self.cpu_read(self.cpu.ab);
        self.check_dma();
    }

    // Always executed on the last instruction cycle (before next instruction fetch)
    #[inline]
    pub(in cpu) fn last_cycle(&mut self) {
        self.check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.check_dma();
    }

    #[inline]
    pub(in cpu) fn compare(&mut self, num: u8, mut reg: u8) {
        self.cpu.c = reg >= num;
        reg = reg.wrapping_sub(num);
        self.cpu.z = reg == 0;
        self.cpu.n = reg & (1 << 7) != 0;
    }

    #[inline]
    pub(in cpu) fn take_branch(&mut self) {
        let diff = self.cpu.temp as i8 as isize;
        let pc_before = self.cpu.pc;
        if diff > 0 {
            self.cpu.pc = (self.cpu.pc as u16).wrapping_add(diff as u16) as usize;
        } else {
            self.cpu.pc = (self.cpu.pc as u16).wrapping_sub(diff.abs() as u16) as usize;
        };
        let crosses = (pc_before & 0xFF00) != (self.cpu.pc & 0xFF00);
        self.cpu.temp = if crosses { 1 } else { 0 };
    }

    #[inline]
    pub(in cpu) fn push_status(&mut self, brk_php: bool) {
        let mut status: u8 = 1 << 5;
        status |= (if self.cpu.n { 1 } else { 0 }) << 7;
        status |= (if self.cpu.v { 1 } else { 0 }) << 6;
        status |= (if brk_php { 1 } else { 0 }) << 4;
        status |= (if self.cpu.d { 1 } else { 0 }) << 3;
        status |= (if self.cpu.i { 1 } else { 0 }) << 2;
        status |= (if self.cpu.z { 1 } else { 0 }) << 1;
        status |= if self.cpu.c { 1 } else { 0 };
        self.cpu_write(self.cpu.ab, status);
    }

    #[inline]
    pub(in cpu) fn pull_status(&mut self, status: u8) {
        self.cpu.n = status >> 7 != 0;
        self.cpu.v = (status >> 6) & 1 != 0;
        self.cpu.d = (status >> 3) & 1 != 0;
        self.cpu.i = (status >> 2) & 1 != 0;
        self.cpu.z = (status >> 1) & 1 != 0;
        self.cpu.c = status & 1 != 0;
    }

    #[inline]
    pub(in cpu) fn set_z_n(&mut self, num: u8) {
        self.cpu.z = num == 0;
        self.cpu.n = (num & 0b1000_0000) != 0;
    }
}
