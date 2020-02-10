use super::super::Nes;

impl Nes {
    // Table of instructions - index is opcode, 1st field is addressing mode,
    // 2nd field is function name
    pub fn cpu_tick_new(&mut self) {
        match self.cpu.state {
            //0x00 => (Brk, ""),
            //0x01 => (IndirectX, "self.cpu_ora(val);"),
            0x02 => self.cpu_immediate(Nes::cpu_halt),
            //0x03 => (IndirectXIllegal, "self.cpu_slo();"),
            0x04 => self.cpu_zero_page(|_, _| ()),
            0x05 => self.cpu_zero_page(Nes::cpu_ora),
            //0x06 => (ZeroPageRmw, "self.cpu_asl(val);"),
            //0x07 => (ZeroPageRmw, "self.cpu_slo();"),
            //0x08 => (Php, ""),
            0x09 => self.cpu_immediate(Nes::cpu_ora),
            0x0A => self.cpu_accumulator(Nes::cpu_asl_a),
            0x0B => self.cpu_immediate(Nes::cpu_anc),
            //0x0C => (Absolute, ""),
            //0x0D => (Absolute, "self.cpu_ora(val);"),
            //0x0E => (AbsoluteRmw, "self.cpu_asl(val);"),
            //0x0F => (AbsoluteRmw, "self.cpu_slo();"),
            //0x10 => (Relative, "!self.cpu.n"),
            //0x11 => (IndirectY, "self.cpu_ora(val);"),
            0x12 => self.cpu_immediate(Nes::cpu_halt),
            //0x13 => (IndirectYIllegal, "self.cpu_slo();"),
            //0x14 => (ZeroPageX, ""),
            //0x15 => (ZeroPageX, "self.cpu_ora(val);"),
            //0x16 => (ZeroPageXRmw, "self.cpu_asl(val);"),
            //0x17 => (ZeroPageXRmw, "self.cpu_slo();"),
            0x18 => self.cpu_implied(Nes::cpu_clc),
            //0x19 => (AbsoluteY, "self.cpu_ora(val);"),
            0x1A => self.cpu_implied(|_| ()),
            //0x1B => (AbsoluteYIllegal, "self.cpu_slo();"),
            //0x1C => (AbsoluteX, ""),
            //0x1D => (AbsoluteX, "self.cpu_ora(val);"),
            //0x1E => (AbsoluteXRmw, "self.cpu_asl(val);"),
            //0x1F => (AbsoluteXRmw, "self.cpu_slo();"),
            //0x20 => (Jsr, "self.cpu_jsr(val);"),
            //0x21 => (IndirectX, "self.cpu_and(val);"),
            0x22 => self.cpu_immediate(Nes::cpu_halt),
            //0x23 => (IndirectXIllegal, "self.cpu_rla();"),
            0x24 => self.cpu_zero_page(Nes::cpu_bit),
            0x25 => self.cpu_zero_page(Nes::cpu_and),
            //0x26 => (ZeroPageRmw, "self.cpu_rol(val);"),
            //0x27 => (ZeroPageRmw, "self.cpu_rla();"),
            //0x28 => (Plp, ""),
            0x29 => self.cpu_immediate(Nes::cpu_and),
            0x2A => self.cpu_accumulator(Nes::cpu_rol_a),
            0x2B => self.cpu_immediate(Nes::cpu_anc),
            //0x2C => (Absolute, "self.cpu_bit(val);"),
            //0x2D => (Absolute, "self.cpu_and(val);"),
            //0x2E => (AbsoluteRmw, "self.cpu_rol(val);"),
            //0x2F => (AbsoluteRmw, "self.cpu_rla();"),
            //0x30 => (Relative, "self.cpu.n"),
            //0x31 => (IndirectY, "self.cpu_and(val);"),
            0x32 => self.cpu_immediate(Nes::cpu_halt),
            //0x33 => (IndirectYIllegal, "self.cpu_rla();"),
            //0x34 => (ZeroPageX, ""),
            //0x35 => (ZeroPageX, "self.cpu_and(val);"),
            //0x36 => (ZeroPageXRmw, "self.cpu_rol(val);"),
            //0x37 => (ZeroPageXRmw, "self.cpu_rla();"),
            0x38 => self.cpu_implied(Nes::cpu_sec),
            //0x39 => (AbsoluteY, "self.cpu_and(val);"),
            0x3A => self.cpu_implied(|_| ()),
            //0x3B => (AbsoluteYIllegal, "self.cpu_rla();"),
            //0x3C => (AbsoluteX, ""),
            //0x3D => (AbsoluteX, "self.cpu_and(val);"),
            //0x3E => (AbsoluteXRmw, "self.cpu_rol(val);"),
            //0x3F => (AbsoluteXRmw, "self.cpu_rla();"),
            //0x40 => (Rti, "self.cpu_rti();"),
            //0x41 => (IndirectX, "self.cpu_eor(val);"),
            0x42 => self.cpu_immediate(Nes::cpu_halt),
            //0x43 => (IndirectXIllegal, "self.cpu_sre();"),
            0x44 => self.cpu_zero_page(|_, _| ()),
            0x45 => self.cpu_zero_page(Nes::cpu_eor),
            //0x46 => (ZeroPageRmw, "self.cpu_lsr(val);"),
            //0x47 => (ZeroPageRmw, "self.cpu_sre();"),
            //0x48 => (Pha, ""),
            0x49 => self.cpu_immediate(Nes::cpu_eor),
            0x4A => self.cpu_accumulator(Nes::cpu_lsr_a),
            0x4B => self.cpu_immediate(Nes::cpu_alr),
            //0x4C => (AbsoluteJmp, ""),
            //0x4D => (Absolute, "self.cpu_eor(val);"),
            //0x4E => (AbsoluteRmw, "self.cpu_lsr(val);"),
            //0x4F => (AbsoluteRmw, "self.cpu_sre();"),
            //0x50 => (Relative, "!self.cpu.v"),
            //0x51 => (IndirectY, "self.cpu_eor(val);"),
            0x52 => self.cpu_immediate(Nes::cpu_halt),
            //0x53 => (IndirectYIllegal, "self.cpu_sre();"),
            //0x54 => (ZeroPageX, ""),
            //0x55 => (ZeroPageX, "self.cpu_eor(val);"),
            //0x56 => (ZeroPageXRmw, "self.cpu_lsr(val);"),
            //0x57 => (ZeroPageXRmw, "self.cpu_sre();"),
            0x58 => self.cpu_implied(Nes::cpu_cli),
            //0x59 => (AbsoluteY, "self.cpu_eor(val);"),
            0x5A => self.cpu_implied(|_| ()),
            //0x5B => (AbsoluteYIllegal, "self.cpu_sre();"),
            //0x5C => (AbsoluteX, ""),
            //0x5D => (AbsoluteX, "self.cpu_eor(val);"),
            //0x5E => (AbsoluteXRmw, "self.cpu_lsr(val);"),
            //0x5F => (AbsoluteXRmw, "self.cpu_sre();"),
            //0x60 => (Rts, "self.cpu_rts();"),
            //0x61 => (IndirectX, "self.cpu_adc(val);"),
            0x62 => self.cpu_immediate(Nes::cpu_halt),
            //0x63 => (IndirectXIllegal, "self.cpu_rra();"),
            0x64 => self.cpu_zero_page(|_, _| ()),
            0x65 => self.cpu_zero_page(Nes::cpu_adc),
            //0x66 => (ZeroPageRmw, "self.cpu_ror(val);"),
            //0x67 => (ZeroPageRmw, "self.cpu_rra();"),
            //0x68 => (Pla, ""),
            0x69 => self.cpu_immediate(Nes::cpu_adc),
            0x6A => self.cpu_accumulator(Nes::cpu_ror_a),
            0x6B => self.cpu_immediate(Nes::cpu_arr),
            //0x6C => (Indirect, ""),
            //0x6D => (Absolute, "self.cpu_adc(val);"),
            //0x6E => (AbsoluteRmw, "self.cpu_ror(val);"),
            //0x6F => (AbsoluteRmw, "self.cpu_rra();"),
            //0x70 => (Relative, "self.cpu.v"),
            //0x71 => (IndirectY, "self.cpu_adc(val);"),
            0x72 => self.cpu_immediate(Nes::cpu_halt),
            //0x73 => (IndirectYIllegal, "self.cpu_rra();"),
            //0x74 => (ZeroPageX, ""),
            //0x75 => (ZeroPageX, "self.cpu_adc(val);"),
            //0x76 => (ZeroPageXRmw, "self.cpu_ror(val);"),
            //0x77 => (ZeroPageXRmw, "self.cpu_rra();"),
            0x78 => self.cpu_implied(Nes::cpu_sei),
            //0x79 => (AbsoluteY, "self.cpu_adc(val);"),
            0x7A => self.cpu_implied(|_| ()),
            //0x7B => (AbsoluteYIllegal, "self.cpu_rra();"),
            //0x7C => (AbsoluteX, ""),
            //0x7D => (AbsoluteX, "self.cpu_adc(val);"),
            //0x7E => (AbsoluteXRmw, "self.cpu_ror(val);"),
            //0x7F => (AbsoluteXRmw, "self.cpu_rra();"),
            0x80 => self.cpu_immediate(|_, _| ()),
            //0x81 => (IndirectXSt, "self.cpu_sta();"),
            0x82 => self.cpu_immediate(|_, _| ()),
            //0x83 => (IndirectX, "self.cpu_aax();"),
            //0x84 => (ZeroPageSt, "self.cpu_sty();"),
            //0x85 => (ZeroPageSt, "self.cpu_sta();"),
            //0x86 => (ZeroPageSt, "self.cpu_stx();"),
            0x87 => self.cpu_zero_page(Nes::cpu_aax),
            0x88 => self.cpu_implied(Nes::cpu_dey),
            0x89 => self.cpu_immediate(|_, _| ()),
            0x8A => self.cpu_implied(Nes::cpu_txa),
            0x8B => self.cpu_immediate(Nes::cpu_xaa),
            //0x8C => (AbsoluteSt, "self.cpu_sty();"),
            //0x8D => (AbsoluteSt, "self.cpu_sta();"),
            //0x8E => (AbsoluteSt, "self.cpu_stx();"),
            //0x8F => (Absolute, "self.cpu_aax();"),
            //0x90 => (Relative, "!self.cpu.c"),
            //0x91 => (IndirectYSt, "self.cpu_sta();"),
            0x92 => self.cpu_immediate(Nes::cpu_halt),
            //0x93 => (IndirectYSt, "self.cpu_ahx();"),
            //0x94 => (ZeroPageXSt, "self.cpu_sty();"),
            //0x95 => (ZeroPageXSt, "self.cpu_sta();"),
            //0x96 => (ZeroPageYSt, "self.cpu_stx();"),
            //0x97 => (ZeroPageY, "self.cpu_aax();"),
            0x98 => self.cpu_implied(Nes::cpu_tya),
            //0x99 => (AbsoluteYSt, "self.cpu_sta();"),
            0x9A => self.cpu_implied(Nes::cpu_txs),
            //0x9B => (AbsoluteYSt, "self.cpu_tas();"),
            //0x9C => (AbsoluteXSt, "self.cpu_shy();"),
            //0x9D => (AbsoluteXSt, "self.cpu_sta();"),
            //0x9E => (AbsoluteYSt, "self.cpu_shx();"),
            //0x9F => (AbsoluteYSt, "self.cpu_ahx();"),
            0xA0 => self.cpu_immediate(Nes::cpu_ldy),
            //0xA1 => (IndirectX, "self.cpu_lda(val);"),
            0xA2 => self.cpu_immediate(Nes::cpu_ldx),
            //0xA3 => (IndirectX, "self.cpu_lax(val);"),
            0xA4 => self.cpu_zero_page(Nes::cpu_ldy),
            0xA5 => self.cpu_zero_page(Nes::cpu_lda),
            0xA6 => self.cpu_zero_page(Nes::cpu_ldx),
            0xA7 => self.cpu_zero_page(Nes::cpu_lax),
            0xA8 => self.cpu_implied(Nes::cpu_tay),
            0xA9 => self.cpu_immediate(Nes::cpu_lda),
            0xAA => self.cpu_implied(Nes::cpu_tax),
            0xAB => self.cpu_immediate(Nes::cpu_lax),
            //0xAC => (Absolute, "self.cpu_ldy(val);"),
            //0xAD => (Absolute, "self.cpu_lda(val);"),
            //0xAE => (Absolute, "self.cpu_ldx(val);"),
            //0xAF => (Absolute, "self.cpu_lax(val);"),
            //0xB0 => (Relative, "self.cpu.c"),
            //0xB1 => (IndirectY, "self.cpu_lda(val);"),
            0xB2 => self.cpu_immediate(Nes::cpu_halt),
            //0xB3 => (IndirectY, "self.cpu_lax(val);"),
            //0xB4 => (ZeroPageX, "self.cpu_ldy(val);"),
            //0xB5 => (ZeroPageX, "self.cpu_lda(val);"),
            //0xB6 => (ZeroPageY, "self.cpu_ldx(val);"),
            //0xB7 => (ZeroPageY, "self.cpu_lax(val);"),
            0xB8 => self.cpu_implied(Nes::cpu_clv),
            //0xB9 => (AbsoluteY, "self.cpu_lda(val);"),
            0xBA => self.cpu_implied(Nes::cpu_tsx),
            //0xBB => (AbsoluteY, "self.cpu_las(val);"),
            //0xBC => (AbsoluteX, "self.cpu_ldy(val);"),
            //0xBD => (AbsoluteX, "self.cpu_lda(val);"),
            //0xBE => (AbsoluteY, "self.cpu_ldx(val);"),
            //0xBF => (AbsoluteY, "self.cpu_lax(val);"),
            0xC0 => self.cpu_immediate(Nes::cpu_cpy),
            //0xC1 => (IndirectX, "self.cpu_cmp(val);"),
            0xC2 => self.cpu_immediate(|_, _| ()),
            //0xC3 => (IndirectXIllegal, "self.cpu_dcp();"),
            0xC4 => self.cpu_zero_page(Nes::cpu_cpy),
            0xC5 => self.cpu_zero_page(Nes::cpu_cmp),
            //0xC6 => (ZeroPageRmw, "self.cpu_dec(val);"),
            //0xC7 => (ZeroPageRmw, "self.cpu_dcp();"),
            0xC8 => self.cpu_implied(Nes::cpu_iny),
            0xC9 => self.cpu_immediate(Nes::cpu_cmp),
            0xCA => self.cpu_implied(Nes::cpu_dex),
            0xCB => self.cpu_immediate(Nes::cpu_axs),
            //0xCC => (Absolute, "self.cpu_cpy(val);"),
            //0xCD => (Absolute, "self.cpu_cmp(val);"),
            //0xCE => (AbsoluteRmw, "self.cpu_dec(val);"),
            //0xCF => (AbsoluteRmw, "self.cpu_dcp();"),
            //0xD0 => (Relative, "!self.cpu.z"),
            //0xD1 => (IndirectY, "self.cpu_cmp(val);"),
            0xD2 => self.cpu_immediate(Nes::cpu_halt),
            //0xD3 => (IndirectYIllegal, "self.cpu_dcp();"),
            //0xD4 => (ZeroPageX, ""),
            //0xD5 => (ZeroPageX, "self.cpu_cmp(val);"),
            //0xD6 => (ZeroPageXRmw, "self.cpu_dec(val);"),
            //0xD7 => (ZeroPageXRmw, "self.cpu_dcp();"),
            0xD8 => self.cpu_implied(Nes::cpu_cld),
            //0xD9 => (AbsoluteY, "self.cpu_cmp(val);"),
            0xDA => self.cpu_implied(|_| ()),
            //0xDB => (AbsoluteYIllegal, "self.cpu_dcp();"),
            //0xDC => (AbsoluteX, ""),
            //0xDD => (AbsoluteX, "self.cpu_cmp(val);"),
            //0xDE => (AbsoluteXRmw, "self.cpu_dec(val);"),
            //0xDF => (AbsoluteXRmw, "self.cpu_dcp();"),
            0xE0 => self.cpu_immediate(Nes::cpu_cpx),
            //0xE1 => (IndirectX, "self.cpu_sbc(val);"),
            0xE2 => self.cpu_immediate(|_, _| ()),
            //0xE3 => (IndirectXIllegal, "self.cpu_isc();"),
            0xE4 => self.cpu_zero_page(Nes::cpu_cpx),
            0xE5 => self.cpu_zero_page(Nes::cpu_sbc),
            //0xE6 => (ZeroPageRmw, "self.cpu_inc(val);"),
            //0xE7 => (ZeroPageRmw, "self.cpu_isc();"),
            0xE8 => self.cpu_implied(Nes::cpu_inx),
            0xE9 => self.cpu_immediate(Nes::cpu_sbc),
            0xEA => self.cpu_implied(|_| ()),
            0xEB => self.cpu_immediate(Nes::cpu_sbc),
            //0xEC => (Absolute, "self.cpu_cpx(val);"),
            //0xED => (Absolute, "self.cpu_sbc(val);"),
            //0xEE => (AbsoluteRmw, "self.cpu_inc(val);"),
            //0xEF => (AbsoluteRmw, "self.cpu_isc();"),
            //0xF0 => (Relative, "self.cpu.z"),
            //0xF1 => (IndirectY, "self.cpu_sbc(val);"),
            0xF2 => self.cpu_immediate(Nes::cpu_halt),
            //0xF3 => (IndirectYIllegal, "self.cpu_isc();"),
            //0xF4 => (ZeroPageX, ""),
            //0xF5 => (ZeroPageX, "self.cpu_sbc(val);"),
            //0xF6 => (ZeroPageXRmw, "self.cpu_inc(val);"),
            //0xF7 => (ZeroPageXRmw, "self.cpu_isc();"),
            0xF8 => self.cpu_implied(Nes::cpu_sed),
            //0xF9 => (AbsoluteY, "self.cpu_sbc(val);"),
            0xFA => self.cpu_implied(|_| ()),
            //0xFB => (AbsoluteYIllegal, "self.cpu_isc();"),
            //0xFC => (AbsoluteX, ""),
            //0xFD => (AbsoluteX, "self.cpu_sbc(val);"),
            //0xFE => (AbsoluteXRmw, "self.cpu_inc(val);"),
            //0xFF => (AbsoluteXRmw, "self.cpu_isc();"),
            _ => (),
        }
    }
}

// Addressing modes with proper timings

impl Nes {
    #[inline]
    pub fn cpu_implied(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.cpu_check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.cpu_check_dma();

        op_instruction(self);

        self.clock_ppu_apu();

        // Cycle 1
        self.cpu_load_next_instruction();

        self.clock_ppu_apu();
    }

    #[inline]
    pub fn cpu_immediate(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.cpu_check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.cpu_check_dma();

        op_instruction(self, self.cpu.db);

        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 1
        self.cpu_load_next_instruction();

        self.clock_ppu_apu();
    }

    #[inline]
    pub fn cpu_accumulator(&mut self, op_instruction: fn(&mut Nes)) {
        // Cycle 0
        self.cpu_check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.cpu_check_dma();
        op_instruction(self);

        self.clock_ppu_apu();

        // Cycle 1

        self.cpu_load_next_instruction();

        self.clock_ppu_apu();
    }

    #[inline]
    pub fn cpu_zero_page(&mut self, op_instruction: fn(&mut Nes, val: u8)) {
        // Cycle 0
        self.cpu_cache_interrupts();
        self.cpu_read(self.cpu.ab);
        self.cpu_check_dma();

        self.cpu.ab = self.cpu.db as usize;
        self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;

        self.clock_ppu_apu();

        // Cycle 1
        self.cpu_check_interrupts();
        self.cpu_read(self.cpu.ab);
        self.cpu_check_dma();

        op_instruction(self, self.cpu.db);
        self.cpu.ab = self.cpu.pc;

        self.clock_ppu_apu();

        // Cycle 2
        self.cpu_load_next_instruction();

        self.clock_ppu_apu();
    }
}

// The instruction logic without timings

impl Nes {
    #[inline]
    pub fn cpu_adc(&mut self, num: u8) {
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
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_sbc(&mut self, num: u8) {
        self.cpu_adc(!num);
    }

    #[inline]
    pub fn cpu_asl(&mut self, mut num: u8) {
        self.cpu.c = num & (1 << 7) != 0;
        num <<= 1;
        self.cpu_set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub fn cpu_asl_a(&mut self) {
        self.cpu.c = self.cpu.a & (1 << 7) != 0;
        self.cpu.a <<= 1;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_lsr(&mut self, mut num: u8) {
        self.cpu.c = (num & 1) != 0;
        num >>= 1;
        self.cpu_set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub fn cpu_lsr_a(&mut self) {
        self.cpu.c = (self.cpu.a & 1) != 0;
        self.cpu.a >>= 1;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_rol(&mut self, mut num: u8) {
        let c = self.cpu.c;
        self.cpu.c = num & (1 << 7) != 0;
        num <<= 1;
        num |= if c { 1 } else { 0 };
        self.cpu_set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub fn cpu_rol_a(&mut self) {
        let c = self.cpu.c;
        self.cpu.c = self.cpu.a & (1 << 7) != 0;
        self.cpu.a <<= 1;
        self.cpu.a |= if c { 1 } else { 0 };
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_ror(&mut self, mut num: u8) {
        let c = self.cpu.c;
        self.cpu.c = num & 1 != 0;
        num >>= 1;
        num |= if c { 1 } else { 0 } << 7;
        self.cpu_set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub fn cpu_ror_a(&mut self) {
        let c = self.cpu.c;
        self.cpu.c = self.cpu.a & 1 != 0;
        self.cpu.a >>= 1;
        self.cpu.a |= if c { 1 } else { 0 } << 7;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_inc(&mut self, mut num: u8) {
        num = num.wrapping_add(1);
        self.cpu_set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub fn cpu_inx(&mut self) {
        self.cpu.x = self.cpu.x.wrapping_add(1);
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_iny(&mut self) {
        self.cpu.y = self.cpu.y.wrapping_add(1);
        self.cpu_set_z_n(self.cpu.y);
    }

    #[inline]
    pub fn cpu_dec(&mut self, mut num: u8) {
        num = num.wrapping_sub(1);
        self.cpu_set_z_n(num);
        self.cpu.temp = num as usize;
    }

    #[inline]
    pub fn cpu_dex(&mut self) {
        self.cpu.x = self.cpu.x.wrapping_sub(1);
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_dey(&mut self) {
        self.cpu.y = self.cpu.y.wrapping_sub(1);
        self.cpu_set_z_n(self.cpu.y);
    }

    #[inline]
    pub fn cpu_and(&mut self, num: u8) {
        self.cpu.a &= num;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_eor(&mut self, num: u8) {
        self.cpu.a ^= num;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_ora(&mut self, num: u8) {
        self.cpu.a |= num;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_cmp(&mut self, num: u8) {
        self.cpu_compare(num, self.cpu.a);
    }

    #[inline]
    pub fn cpu_cpx(&mut self, num: u8) {
        self.cpu_compare(num, self.cpu.x);
    }

    #[inline]
    pub fn cpu_cpy(&mut self, num: u8) {
        self.cpu_compare(num, self.cpu.y);
    }

    #[inline]
    pub fn cpu_bit(&mut self, byte: u8) {
        self.cpu.z = (byte & self.cpu.a) == 0;
        self.cpu.v = (byte >> 6) & 1 != 0;
        self.cpu.n = (byte >> 7) & 1 != 0;
    }

    #[inline]
    pub fn cpu_clc(&mut self) {
        self.cpu.c = false;
    }

    #[inline]
    pub fn cpu_sec(&mut self) {
        self.cpu.c = true;
    }

    #[inline]
    pub fn cpu_cli(&mut self) {
        self.cpu.i = false;
    }

    #[inline]
    pub fn cpu_sei(&mut self) {
        self.cpu.i = true;
    }

    #[inline]
    pub fn cpu_cld(&mut self) {
        self.cpu.d = false;
    }

    #[inline]
    pub fn cpu_sed(&mut self) {
        self.cpu.d = true;
    }

    #[inline]
    pub fn cpu_clv(&mut self) {
        self.cpu.v = false;
    }

    #[inline]
    pub fn cpu_lda(&mut self, num: u8) {
        self.cpu.a = num;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_ldx(&mut self, num: u8) {
        self.cpu.x = num;
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_ldy(&mut self, num: u8) {
        self.cpu.y = num;
        self.cpu_set_z_n(self.cpu.y);
    }

    #[inline]
    pub fn cpu_sta(&mut self) {
        self.cpu_write(self.cpu.ab, self.cpu.a);
    }

    #[inline]
    pub fn cpu_stx(&mut self) {
        self.cpu_write(self.cpu.ab, self.cpu.x);
    }

    #[inline]
    pub fn cpu_sty(&mut self) {
        self.cpu_write(self.cpu.ab, self.cpu.y);
    }

    #[inline]
    pub fn cpu_tax(&mut self) {
        self.cpu.x = self.cpu.a;
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_tay(&mut self) {
        self.cpu.y = self.cpu.a;
        self.cpu_set_z_n(self.cpu.y);
    }

    #[inline]
    pub fn cpu_tsx(&mut self) {
        self.cpu.x = self.cpu.sp as u8;
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_txa(&mut self) {
        self.cpu.a = self.cpu.x;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_txs(&mut self) {
        self.cpu.sp = self.cpu.x as usize;
    }

    #[inline]
    pub fn cpu_tya(&mut self) {
        self.cpu.a = self.cpu.y;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_aax(&mut self, _: u8) {
        let res = self.cpu.x & self.cpu.a;
        self.cpu_write(self.cpu.ab, res);
    }

    #[inline]
    pub fn cpu_dcp(&mut self) {
        self.cpu_dec(self.cpu.temp as u8);
        self.cpu_cmp(self.cpu.temp as u8);
    }

    #[inline]
    pub fn cpu_isc(&mut self) {
        self.cpu_inc(self.cpu.temp as u8);
        self.cpu_sbc(self.cpu.temp as u8);
    }

    #[inline]
    pub fn cpu_lax(&mut self, num: u8) {
        self.cpu.a = num;
        self.cpu.x = self.cpu.a;
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_rla(&mut self) {
        self.cpu_rol(self.cpu.temp as u8);
        self.cpu_and(self.cpu.temp as u8);
    }

    #[inline]
    pub fn cpu_rra(&mut self) {
        self.cpu_ror(self.cpu.temp as u8);
        self.cpu_adc(self.cpu.temp as u8);
    }

    #[inline]
    pub fn cpu_slo(&mut self) {
        self.cpu_asl(self.cpu.temp as u8);
        self.cpu_ora(self.cpu.temp as u8);
    }

    #[inline]
    pub fn cpu_sre(&mut self) {
        self.cpu_lsr(self.cpu.temp as u8);
        self.cpu_eor(self.cpu.temp as u8);
    }

    #[inline]
    pub fn cpu_anc(&mut self, num: u8) {
        self.cpu_and(num);
        self.cpu.c = self.cpu.n;
    }

    #[inline]
    pub fn cpu_alr(&mut self, num: u8) {
        self.cpu.a &= num;
        self.cpu.c = self.cpu.a & 1 != 0;
        self.cpu.a >>= 1;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_axs(&mut self, num: u8) {
        self.cpu.x &= self.cpu.a;
        self.cpu.c = self.cpu.x >= num;
        self.cpu.x = self.cpu.x.wrapping_sub(num);
        self.cpu_set_z_n(self.cpu.x);
    }

    #[inline]
    pub fn cpu_xaa(&mut self, num: u8) {
        self.cpu.a = (self.cpu.a | 0xEE) & self.cpu.x & num;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_ahx(&mut self) {
        let result = self.cpu.a & self.cpu.x & ((self.cpu.ab >> 8) + 1) as u8;
        self.cpu_write(self.cpu.ab, result);
    }

    #[inline]
    pub fn cpu_shx(&mut self) {
        let result = ((self.cpu.ab >> 8) as u8).wrapping_add(1) & self.cpu.x;
        self.cpu_write(
            (usize::from(result) << 8) | (self.cpu.ab & 0xFF),
            self.cpu.x,
        );
    }

    #[inline]
    pub fn cpu_shy(&mut self) {
        let result = ((self.cpu.ab >> 8) as u8).wrapping_add(1) & self.cpu.y;
        self.cpu_write(
            (usize::from(result) << 8) | (self.cpu.ab & 0xFF),
            self.cpu.y,
        );
    }

    #[inline]
    pub fn cpu_las(&mut self, val: u8) {
        self.cpu.sp &= val as usize;
        self.cpu.a = self.cpu.sp as u8;
        self.cpu.x = self.cpu.sp as u8;
        self.cpu_set_z_n(self.cpu.a);
    }

    #[inline]
    pub fn cpu_tas(&mut self) {
        self.cpu.sp = (self.cpu.a & self.cpu.x) as usize;
        let result = self.cpu.sp & ((self.cpu.ab >> 8) + 1);
        self.cpu_write(self.cpu.ab, result as u8);
    }

    #[inline]
    pub fn cpu_arr(&mut self, num: u8) {
        self.cpu_and(num);
        self.cpu_ror_a();

        self.cpu.c = (self.cpu.a >> 6) & 1 == 1;
        self.cpu.v = self.cpu.c != ((self.cpu.a >> 5) & 1 == 1);
    }

    pub fn cpu_halt(&mut self, _: u8) {
        self.cpu.halt = true;
    }
}

// Helper functions
impl Nes {
    pub fn cpu_load_next_instruction(&mut self) {
        self.cpu_cache_interrupts();
        let int: u8 = if self.cpu.take_interrupt { 0 } else { 1 };
        self.cpu_read(self.cpu.ab);
        self.cpu_check_dma();
        self.cpu.state = u16::from(int * self.cpu.db);
        self.cpu.pc = (self.cpu.pc).wrapping_add(int as usize);
        self.cpu.ab = self.cpu.pc
    }

    pub fn cpu_cache_interrupts(&mut self) {
        self.cpu.cached_irq = self.cpu.irq_signal;
        self.cpu.cached_nmi = self.cpu.nmi_signal;
    }

    pub fn cpu_check_dma(&mut self) {
        if self.cpu.dma.hijack_read {
            self.cpu.dma.cycles = 1;
            return;
        }
    }

    pub fn cpu_sp_to_ab(&mut self) {
        self.cpu.ab = self.cpu.sp | 0x100;
    }
}
