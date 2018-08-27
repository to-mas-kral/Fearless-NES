use super::Tick;
impl Tick for super::Cpu {
    #[allow(unused_variables)]
    fn tick(&mut self) {
        if self.halt {
            return;
        }
        macro_rules! cache_interrupts {
            ($self: ident) => {
                self.cached_irq = self.interrupt_bus.borrow().irq_signal;
                self.cached_nmi = self.interrupt_bus.borrow().nmi_signal;
            };
        }
        macro_rules! read_ab {
            ($self: ident) => {
                $self.read($self.ab)
            };
        }
        macro_rules! sp_to_ab {
            ($self: ident) => {
                $self.ab = $self.sp | 0x100
            };
        }
        debug_log!("executing opcode 0x{:X}", (self.state));
        debug_log!("CPU state: {}", (self.debug_info()));
        self.odd_cycle = !self.odd_cycle;
        match self.state {
            0x0 => {
                read_ab!(self);
                let int = if self.take_interrupt { 0 } else { 1 };
                self.pc += int;
                sp_to_ab!(self);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x102;
            }
            0x1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x107
            }
            0x2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x10B;
            }
            0x4 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x111;
            }
            0x5 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x112;
            }
            0x6 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x113;
            }
            0x7 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x116;
            }
            0x8 => {
                cache_interrupts!(self);
                read_ab!(self);
                sp_to_ab!(self);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x119;
            }
            0x9 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xA => {
                self.check_interrupts();
                read_ab!(self);
                self.asl_a();
                self.state = 0x100;
            }
            0xB => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.anc(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x11A;
            }
            0xD => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x11C;
            }
            0xE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x11E;
            }
            0xF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x122;
            }
            0x10 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if !self.n { 0x126 } else { 0x100 }
            }
            0x11 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x128;
            }
            0x12 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x13 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x12D;
            }
            0x14 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x133;
            }
            0x15 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x135;
            }
            0x16 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x137;
            }
            0x17 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x13B;
            }
            0x18 => {
                self.check_interrupts();
                read_ab!(self);
                self.c = false;
                self.state = 0x100;
            }
            0x19 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x13F;
            }
            0x1A => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0x1B => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x142;
            }
            0x1C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x147;
            }
            0x1D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x14A;
            }
            0x1E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x14D;
            }
            0x1F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x152;
            }
            0x20 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                sp_to_ab!(self);
                self.state = 0x157;
            }
            0x21 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x15B
            }
            0x22 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x23 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x15F;
            }
            0x24 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x165;
            }
            0x25 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x166;
            }
            0x26 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x167;
            }
            0x27 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x16A;
            }
            0x28 => {
                read_ab!(self);
                sp_to_ab!(self);
                self.state = 0x16D;
            }
            0x29 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2A => {
                self.check_interrupts();
                read_ab!(self);
                self.rol_a();
                self.state = 0x100;
            }
            0x2B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.anc(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x16F;
            }
            0x2D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x171;
            }
            0x2E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x173;
            }
            0x2F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x177;
            }
            0x30 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if self.n { 0x17B } else { 0x100 }
            }
            0x31 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x17D;
            }
            0x32 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x33 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x182;
            }
            0x34 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x188;
            }
            0x35 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x18A;
            }
            0x36 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x18C;
            }
            0x37 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x190;
            }
            0x38 => {
                self.check_interrupts();
                read_ab!(self);
                self.c = true;
                self.state = 0x100;
            }
            0x39 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x194;
            }
            0x3A => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0x3B => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x197;
            }
            0x3C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x19C;
            }
            0x3D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x19F;
            }
            0x3E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1A2;
            }
            0x3F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1A7;
            }
            0x40 => {
                read_ab!(self);
                sp_to_ab!(self);
                self.state = 0x1AC;
            }
            0x41 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1B0
            }
            0x42 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x43 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1B4;
            }
            0x44 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1BA;
            }
            0x45 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1BB;
            }
            0x46 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1BC;
            }
            0x47 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1BF;
            }
            0x48 => {
                cache_interrupts!(self);
                read_ab!(self);
                sp_to_ab!(self);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x1C2;
            }
            0x49 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x4A => {
                self.check_interrupts();
                read_ab!(self);
                self.lsr_a();
                self.state = 0x100;
            }
            0x4B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.alr(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x4C => {
                cache_interrupts!(self);
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1C3
            }
            0x4D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1C4;
            }
            0x4E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1C6;
            }
            0x4F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1CA;
            }
            0x50 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if !self.v { 0x1CE } else { 0x100 }
            }
            0x51 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1D0;
            }
            0x52 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x53 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1D5;
            }
            0x54 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1DB;
            }
            0x55 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1DD;
            }
            0x56 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1DF;
            }
            0x57 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x1E3;
            }
            0x58 => {
                self.check_interrupts();
                read_ab!(self);
                self.i = false;
                self.state = 0x100;
            }
            0x59 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1E7;
            }
            0x5A => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0x5B => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1EA;
            }
            0x5C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1EF;
            }
            0x5D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1F2;
            }
            0x5E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1F5;
            }
            0x5F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x1FA;
            }
            0x60 => {
                read_ab!(self);
                sp_to_ab!(self);
                self.state = 0x1FF;
            }
            0x61 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x203
            }
            0x62 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x63 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x207;
            }
            0x64 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x20D;
            }
            0x65 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x20E;
            }
            0x66 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x20F;
            }
            0x67 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x212;
            }
            0x68 => {
                read_ab!(self);
                sp_to_ab!(self);
                self.state = 0x215;
            }
            0x69 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x6A => {
                self.check_interrupts();
                read_ab!(self);
                self.ror_a();
                self.state = 0x100;
            }
            0x6B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.arr(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x6C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x217
            }
            0x6D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x21A;
            }
            0x6E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x21C;
            }
            0x6F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x220;
            }
            0x70 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if self.v { 0x224 } else { 0x100 }
            }
            0x71 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x226;
            }
            0x72 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x73 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x22B;
            }
            0x74 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x231;
            }
            0x75 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x233;
            }
            0x76 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x235;
            }
            0x77 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x239;
            }
            0x78 => {
                self.check_interrupts();
                read_ab!(self);
                self.i = true;
                self.state = 0x100;
            }
            0x79 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x23D;
            }
            0x7A => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0x7B => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x240;
            }
            0x7C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x245;
            }
            0x7D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x248;
            }
            0x7E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x24B;
            }
            0x7F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x250;
            }
            0x80 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x81 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x255;
            }
            0x82 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x83 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x259
            }
            0x84 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x25D;
            }
            0x85 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x25E;
            }
            0x86 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x25F;
            }
            0x87 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x260;
            }
            0x88 => {
                self.check_interrupts();
                read_ab!(self);
                self.dey();
                self.state = 0x100;
            }
            0x89 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x8A => {
                self.check_interrupts();
                read_ab!(self);
                self.txa();
                self.state = 0x100;
            }
            0x8B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.xaa(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x8C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x261;
            }
            0x8D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x263;
            }
            0x8E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x265;
            }
            0x8F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x267;
            }
            0x90 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if !self.c { 0x269 } else { 0x100 }
            }
            0x91 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x26B;
            }
            0x92 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x93 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x26F;
            }
            0x94 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x273;
            }
            0x95 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x275;
            }
            0x96 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x277;
            }
            0x97 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x279;
            }
            0x98 => {
                self.check_interrupts();
                read_ab!(self);
                self.tya();
                self.state = 0x100;
            }
            0x99 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x27B;
            }
            0x9A => {
                self.check_interrupts();
                read_ab!(self);
                self.txs();
                self.state = 0x100;
            }
            0x9B => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x27E;
            }
            0x9C => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x281;
            }
            0x9D => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x284;
            }
            0x9E => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x287;
            }
            0x9F => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x28A;
            }
            0xA0 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldy(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xA1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x28D
            }
            0xA2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldx(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xA3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x291
            }
            0xA4 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x295;
            }
            0xA5 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x296;
            }
            0xA6 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x297;
            }
            0xA7 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x298;
            }
            0xA8 => {
                self.check_interrupts();
                read_ab!(self);
                self.tay();
                self.state = 0x100;
            }
            0xA9 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xAA => {
                self.check_interrupts();
                read_ab!(self);
                self.tax();
                self.state = 0x100;
            }
            0xAB => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xAC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x299;
            }
            0xAD => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x29B;
            }
            0xAE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x29D;
            }
            0xAF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x29F;
            }
            0xB0 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if self.c { 0x2A1 } else { 0x100 }
            }
            0xB1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2A3;
            }
            0xB2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xB3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2A8;
            }
            0xB4 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2AD;
            }
            0xB5 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2AF;
            }
            0xB6 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2B1;
            }
            0xB7 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2B3;
            }
            0xB8 => {
                self.check_interrupts();
                read_ab!(self);
                self.v = false;
                self.state = 0x100;
            }
            0xB9 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2B5;
            }
            0xBA => {
                self.check_interrupts();
                read_ab!(self);
                self.tsx();
                self.state = 0x100;
            }
            0xBB => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2B8;
            }
            0xBC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2BB;
            }
            0xBD => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2BE;
            }
            0xBE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2C1;
            }
            0xBF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2C4;
            }
            0xC0 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cpy(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xC1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2C7
            }
            0xC2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xC3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2CB;
            }
            0xC4 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2D1;
            }
            0xC5 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2D2;
            }
            0xC6 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2D3;
            }
            0xC7 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2D6;
            }
            0xC8 => {
                self.check_interrupts();
                read_ab!(self);
                self.iny();
                self.state = 0x100;
            }
            0xC9 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xCA => {
                self.check_interrupts();
                read_ab!(self);
                self.dex();
                self.state = 0x100;
            }
            0xCB => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.axs(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xCC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2D9;
            }
            0xCD => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2DB;
            }
            0xCE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2DD;
            }
            0xCF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2E1;
            }
            0xD0 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if !self.z { 0x2E5 } else { 0x100 }
            }
            0xD1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2E7;
            }
            0xD2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xD3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2EC;
            }
            0xD4 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2F2;
            }
            0xD5 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2F4;
            }
            0xD6 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2F6;
            }
            0xD7 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x2FA;
            }
            0xD8 => {
                self.check_interrupts();
                read_ab!(self);
                self.d = false;
                self.state = 0x100;
            }
            0xD9 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x2FE;
            }
            0xDA => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0xDB => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x301;
            }
            0xDC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x306;
            }
            0xDD => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x309;
            }
            0xDE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x30C;
            }
            0xDF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x311;
            }
            0xE0 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cpx(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xE1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x316
            }
            0xE2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xE3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x31A;
            }
            0xE4 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x320;
            }
            0xE5 => {
                cache_interrupts!(self);
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x321;
            }
            0xE6 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x322;
            }
            0xE7 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x325;
            }
            0xE8 => {
                self.check_interrupts();
                read_ab!(self);
                self.inx();
                self.state = 0x100;
            }
            0xE9 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xEA => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0xEB => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xEC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x328;
            }
            0xED => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x32A;
            }
            0xEE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x32C;
            }
            0xEF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x330;
            }
            0xF0 => {
                self.check_interrupts();
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = if self.z { 0x334 } else { 0x100 }
            }
            0xF1 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x336;
            }
            0xF2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.halt = true;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0xF3 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x33B;
            }
            0xF4 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x341;
            }
            0xF5 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x343;
            }
            0xF6 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x345;
            }
            0xF7 => {
                self.ab = read_ab!(self) as usize;
                self.pc += 1;
                self.state = 0x349;
            }
            0xF8 => {
                self.check_interrupts();
                read_ab!(self);
                self.d = true;
                self.state = 0x100;
            }
            0xF9 => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x34D;
            }
            0xFA => {
                self.check_interrupts();
                read_ab!(self);
                self.state = 0x100;
            }
            0xFB => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x350;
            }
            0xFC => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x355;
            }
            0xFD => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x358;
            }
            0xFE => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x35B;
            }
            0xFF => {
                self.temp = read_ab!(self) as usize;
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x360;
            }
            0x100 => {
                cache_interrupts!(self);
                let int = if self.take_interrupt { 0 } else { 1 };
                self.state = u16::from(int * read_ab!(self));
                self.pc += int as usize;
                self.ab = self.pc
            }
            0x102 => {
                if !(self.take_interrupt && self.pending_reset) {
                    self.push(self.ab, (self.pc >> 8) as u8);
                }
                sp_to_ab!(self);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x103;
            }
            0x103 => {
                if !(self.take_interrupt && self.pending_reset) {
                    self.push(self.ab, (self.pc & 0xFF) as u8);
                }
                sp_to_ab!(self);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x104;
            }
            0x104 => {
                if !(self.take_interrupt && self.pending_reset) {
                    self.push_status(true);
                }
                self.ab = self.interrupt_address();
                self.take_interrupt = false;
                self.interrupt_type = super::InterruptType::None;
                self.state = 0x105;
            }
            0x105 => {
                self.temp = read_ab!(self) as usize;
                self.ab += 1;
                self.i = true;
                self.state = 0x106;
            }
            0x106 => {
                self.pc = ((read_ab!(self) as usize) << 8) | self.temp;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x107 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x108
            }
            0x108 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x109
            }
            0x109 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x10A;
            }
            0x10A => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x10B => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x10C;
            }
            0x10C => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x10D;
            }
            0x10D => {
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x10E;
            }
            0x10E => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x10F;
            }
            0x10F => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.slo();
                self.state = 0x110;
            }
            0x110 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x111 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x112 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x113 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x114;
            }
            0x114 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.asl(val);
                self.state = 0x115;
            }
            0x115 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x116 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x117;
            }
            0x117 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.slo();
                self.state = 0x118;
            }
            0x118 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x119 => {
                self.check_interrupts();
                self.push_status(true);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x11A => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x11B
            }
            0x11B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x11C => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x11D
            }
            0x11D => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x11E => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x11F;
            }
            0x11F => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x120;
            }
            0x120 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.asl(val);
                self.state = 0x121;
            }
            0x121 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x122 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x123;
            }
            0x123 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x124;
            }
            0x124 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.slo();
                self.state = 0x125;
            }
            0x125 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x126 => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x127 } else { 0x100 };
            }
            0x127 => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x128 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x129;
            }
            0x129 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x12B
                } else {
                    0x12A
                };
            }
            0x12A => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x12B;
            }
            0x12B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x12D => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x12E;
            }
            0x12E => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x12F;
            }
            0x12F => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x130;
            }
            0x130 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x131;
            }
            0x131 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.slo();
                self.state = 0x132;
            }
            0x132 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x133 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x134;
            }
            0x134 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x135 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x136;
            }
            0x136 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x137 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x138;
            }
            0x138 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x139;
            }
            0x139 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.asl(val);
                self.state = 0x13A;
            }
            0x13A => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x13B => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x13C;
            }
            0x13C => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x13D;
            }
            0x13D => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.slo();
                self.state = 0x13E;
            }
            0x13E => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x13F => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x141
                } else {
                    0x140
                };
            }
            0x140 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x141
            }
            0x141 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x142 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x143;
            }
            0x143 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                };
                self.state = 0x144;
            }
            0x144 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x145;
            }
            0x145 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.slo();
                self.state = 0x146;
            }
            0x146 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x147 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x149
                } else {
                    0x148
                };
            }
            0x148 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x149
            }
            0x149 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x14A => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x14C
                } else {
                    0x14B
                };
            }
            0x14B => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x14C
            }
            0x14C => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ora(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x14D => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x14E;
            }
            0x14E => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x14F;
            }
            0x14F => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x150;
            }
            0x150 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.asl(val);
                self.state = 0x151;
            }
            0x151 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x152 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x153;
            }
            0x153 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x154;
            }
            0x154 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x155;
            }
            0x155 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.slo();
                self.state = 0x156;
            }
            0x156 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x157 => {
                self.pop(self.ab);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x158;
            }
            0x158 => {
                self.push(self.ab, (self.pc >> 8) as u8);
                sp_to_ab!(self);
                self.sp = (self.sp as u8).wrapping_sub(1) as usize;
                self.state = 0x159;
            }
            0x159 => {
                cache_interrupts!(self);
                self.push(self.ab, (self.pc & 0xFF) as u8);
                self.ab = self.pc;
                self.state = 0x15A;
            }
            0x15A => {
                self.check_interrupts();
                self.pc = ((read_ab!(self) as usize) << 8) | self.temp;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x15B => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x15C
            }
            0x15C => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x15D
            }
            0x15D => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x15E;
            }
            0x15E => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x15F => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x160;
            }
            0x160 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x161;
            }
            0x161 => {
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x162;
            }
            0x162 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x163;
            }
            0x163 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rla();
                self.state = 0x164;
            }
            0x164 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x165 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.bit(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x166 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x167 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x168;
            }
            0x168 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rol(val);
                self.state = 0x169;
            }
            0x169 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x16A => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x16B;
            }
            0x16B => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rla();
                self.state = 0x16C;
            }
            0x16C => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x16D => {
                cache_interrupts!(self);
                self.pop(self.ab);
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x16E;
            }
            0x16E => {
                self.check_interrupts();
                self.pull_status();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x16F => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x170
            }
            0x170 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.bit(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x171 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x172
            }
            0x172 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x173 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x174;
            }
            0x174 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x175;
            }
            0x175 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rol(val);
                self.state = 0x176;
            }
            0x176 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x177 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x178;
            }
            0x178 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x179;
            }
            0x179 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rla();
                self.state = 0x17A;
            }
            0x17A => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x17B => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x17C } else { 0x100 };
            }
            0x17C => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x17D => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x17E;
            }
            0x17E => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x180
                } else {
                    0x17F
                };
            }
            0x17F => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x180;
            }
            0x180 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x182 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x183;
            }
            0x183 => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x184;
            }
            0x184 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x185;
            }
            0x185 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x186;
            }
            0x186 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.rla();
                self.state = 0x187;
            }
            0x187 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x188 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x189;
            }
            0x189 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x18A => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x18B;
            }
            0x18B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x18C => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x18D;
            }
            0x18D => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x18E;
            }
            0x18E => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rol(val);
                self.state = 0x18F;
            }
            0x18F => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x190 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x191;
            }
            0x191 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x192;
            }
            0x192 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rla();
                self.state = 0x193;
            }
            0x193 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x194 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x196
                } else {
                    0x195
                };
            }
            0x195 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x196
            }
            0x196 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x197 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x198;
            }
            0x198 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                };
                self.state = 0x199;
            }
            0x199 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x19A;
            }
            0x19A => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.rla();
                self.state = 0x19B;
            }
            0x19B => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x19C => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x19E
                } else {
                    0x19D
                };
            }
            0x19D => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x19E
            }
            0x19E => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x19F => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x1A1
                } else {
                    0x1A0
                };
            }
            0x1A0 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x1A1
            }
            0x1A1 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.and(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1A2 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x1A3;
            }
            0x1A3 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x1A4;
            }
            0x1A4 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1A5;
            }
            0x1A5 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rol(val);
                self.state = 0x1A6;
            }
            0x1A6 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1A7 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x1A8;
            }
            0x1A8 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x1A9;
            }
            0x1A9 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1AA;
            }
            0x1AA => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rla();
                self.state = 0x1AB;
            }
            0x1AB => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1AC => {
                self.pop(self.ab);
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x1AD;
            }
            0x1AD => {
                self.pull_status();
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x1AE;
            }
            0x1AE => {
                cache_interrupts!(self);
                self.temp = self.pop(self.ab) as usize;
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x1AF;
            }
            0x1AF => {
                self.check_interrupts();
                self.pc = ((read_ab!(self) as usize) << 8) | self.temp;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1B0 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x1B1
            }
            0x1B1 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x1B2
            }
            0x1B2 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x1B3;
            }
            0x1B3 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x1B4 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x1B5;
            }
            0x1B5 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x1B6;
            }
            0x1B6 => {
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x1B7;
            }
            0x1B7 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1B8;
            }
            0x1B8 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.sre();
                self.state = 0x1B9;
            }
            0x1B9 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1BA => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1BB => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1BC => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x1BD;
            }
            0x1BD => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.lsr(val);
                self.state = 0x1BE;
            }
            0x1BE => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1BF => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x1C0;
            }
            0x1C0 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.sre();
                self.state = 0x1C1;
            }
            0x1C1 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1C2 => {
                self.check_interrupts();
                self.push(self.ab, self.a);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1C3 => {
                self.check_interrupts();
                self.pc = ((read_ab!(self) as usize) << 8) | self.temp;
                self.ab = self.pc;
                self.state = 0x100
            }
            0x1C4 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x1C5
            }
            0x1C5 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x1C6 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x1C7;
            }
            0x1C7 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1C8;
            }
            0x1C8 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.lsr(val);
                self.state = 0x1C9;
            }
            0x1C9 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1CA => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x1CB;
            }
            0x1CB => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1CC;
            }
            0x1CC => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.sre();
                self.state = 0x1CD;
            }
            0x1CD => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1CE => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x1CF } else { 0x100 };
            }
            0x1CF => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x1D0 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x1D1;
            }
            0x1D1 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x1D3
                } else {
                    0x1D2
                };
            }
            0x1D2 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x1D3;
            }
            0x1D3 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1D5 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x1D6;
            }
            0x1D6 => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x1D7;
            }
            0x1D7 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x1D8;
            }
            0x1D8 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1D9;
            }
            0x1D9 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.sre();
                self.state = 0x1DA;
            }
            0x1DA => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1DB => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x1DC;
            }
            0x1DC => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1DD => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x1DE;
            }
            0x1DE => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1DF => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x1E0;
            }
            0x1E0 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x1E1;
            }
            0x1E1 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.lsr(val);
                self.state = 0x1E2;
            }
            0x1E2 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1E3 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x1E4;
            }
            0x1E4 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x1E5;
            }
            0x1E5 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.sre();
                self.state = 0x1E6;
            }
            0x1E6 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1E7 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x1E9
                } else {
                    0x1E8
                };
            }
            0x1E8 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x1E9
            }
            0x1E9 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1EA => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x1EB;
            }
            0x1EB => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                };
                self.state = 0x1EC;
            }
            0x1EC => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1ED;
            }
            0x1ED => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.sre();
                self.state = 0x1EE;
            }
            0x1EE => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1EF => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x1F1
                } else {
                    0x1F0
                };
            }
            0x1F0 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x1F1
            }
            0x1F1 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1F2 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x1F4
                } else {
                    0x1F3
                };
            }
            0x1F3 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x1F4
            }
            0x1F4 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.eor(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1F5 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x1F6;
            }
            0x1F6 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x1F7;
            }
            0x1F7 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1F8;
            }
            0x1F8 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.lsr(val);
                self.state = 0x1F9;
            }
            0x1F9 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1FA => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x1FB;
            }
            0x1FB => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x1FC;
            }
            0x1FC => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x1FD;
            }
            0x1FD => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.sre();
                self.state = 0x1FE;
            }
            0x1FE => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x1FF => {
                self.pop(self.ab);
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x200
            }
            0x200 => {
                self.temp = self.pop(self.ab) as usize;
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x201;
            }
            0x201 => {
                cache_interrupts!(self);
                self.pc = ((self.pop(self.ab) as usize) << 8) | self.temp;
                self.ab = self.pc;
                self.state = 0x202;
            }
            0x202 => {
                self.check_interrupts();
                read_ab!(self);
                self.pc += 1;
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x203 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x204
            }
            0x204 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x205
            }
            0x205 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x206;
            }
            0x206 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x207 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x208;
            }
            0x208 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x209;
            }
            0x209 => {
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x20A;
            }
            0x20A => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x20B;
            }
            0x20B => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rra();
                self.state = 0x20C;
            }
            0x20C => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x20D => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x20E => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x20F => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x210;
            }
            0x210 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.ror(val);
                self.state = 0x211;
            }
            0x211 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x212 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x213;
            }
            0x213 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rra();
                self.state = 0x214;
            }
            0x214 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x215 => {
                cache_interrupts!(self);
                self.pop(self.ab);
                self.sp = (self.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.state = 0x216;
            }
            0x216 => {
                self.check_interrupts();
                let a = self.pop(self.ab);
                self.lda(a);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x217 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.state = 0x218
            }
            0x218 => {
                cache_interrupts!(self);
                self.temp = read_ab!(self) as usize;
                self.ab = (self.ab & 0xFF00) | ((self.ab + 1) & 0xFF);
                self.state = 0x219
            }
            0x219 => {
                self.check_interrupts();
                self.pc = ((read_ab!(self) as usize) << 8) | self.temp;
                self.ab = self.pc;
                self.state = 0x100
            }
            0x21A => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x21B
            }
            0x21B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x21C => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x21D;
            }
            0x21D => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x21E;
            }
            0x21E => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.ror(val);
                self.state = 0x21F;
            }
            0x21F => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x220 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x221;
            }
            0x221 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x222;
            }
            0x222 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rra();
                self.state = 0x223;
            }
            0x223 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x224 => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x225 } else { 0x100 };
            }
            0x225 => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x226 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x227;
            }
            0x227 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x229
                } else {
                    0x228
                };
            }
            0x228 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x229;
            }
            0x229 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x22B => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x22C;
            }
            0x22C => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x22D;
            }
            0x22D => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x22E;
            }
            0x22E => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x22F;
            }
            0x22F => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.rra();
                self.state = 0x230;
            }
            0x230 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x231 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x232;
            }
            0x232 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x233 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x234;
            }
            0x234 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x235 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x236;
            }
            0x236 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x237;
            }
            0x237 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.ror(val);
                self.state = 0x238;
            }
            0x238 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x239 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x23A;
            }
            0x23A => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x23B;
            }
            0x23B => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rra();
                self.state = 0x23C;
            }
            0x23C => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x23D => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x23F
                } else {
                    0x23E
                };
            }
            0x23E => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x23F
            }
            0x23F => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x240 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x241;
            }
            0x241 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                };
                self.state = 0x242;
            }
            0x242 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x243;
            }
            0x243 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.rra();
                self.state = 0x244;
            }
            0x244 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x245 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x247
                } else {
                    0x246
                };
            }
            0x246 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x247
            }
            0x247 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x248 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x24A
                } else {
                    0x249
                };
            }
            0x249 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x24A
            }
            0x24A => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.adc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x24B => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x24C;
            }
            0x24C => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x24D;
            }
            0x24D => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x24E;
            }
            0x24E => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.ror(val);
                self.state = 0x24F;
            }
            0x24F => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x250 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x251;
            }
            0x251 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x252;
            }
            0x252 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x253;
            }
            0x253 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.rra();
                self.state = 0x254;
            }
            0x254 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x255 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x256;
            }
            0x256 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x257;
            }
            0x257 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x258;
            }
            0x258 => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x259 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x25A
            }
            0x25A => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x25B
            }
            0x25B => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x25C;
            }
            0x25C => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.aax();
                self.ab = self.pc;
                self.state = 0x100
            }
            0x25D => {
                self.check_interrupts();
                self.sty();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x25E => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x25F => {
                self.check_interrupts();
                self.stx();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x260 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.aax();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x261 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x262
            }
            0x262 => {
                self.check_interrupts();
                self.sty();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x263 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x264
            }
            0x264 => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x265 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x266
            }
            0x266 => {
                self.check_interrupts();
                self.stx();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x267 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x268
            }
            0x268 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.aax();
                self.ab = self.pc;
                self.state = 0x100
            }
            0x269 => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x26A } else { 0x100 };
            }
            0x26A => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x26B => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x26C;
            }
            0x26C => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x26D;
            }
            0x26D => {
                cache_interrupts!(self);
                read_ab!(self);
                if self.temp + self.y as usize >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                }
                self.state = 0x26E;
            }
            0x26E => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x26F => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x270;
            }
            0x270 => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x271;
            }
            0x271 => {
                cache_interrupts!(self);
                read_ab!(self);
                if self.temp + self.y as usize >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                }
                self.state = 0x272;
            }
            0x272 => {
                self.check_interrupts();
                self.ahx();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x273 => {
                cache_interrupts!(self);
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x274;
            }
            0x274 => {
                self.check_interrupts();
                self.sty();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x275 => {
                cache_interrupts!(self);
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x276;
            }
            0x276 => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x277 => {
                cache_interrupts!(self);
                self.ram[self.ab];
                self.ab = (self.ab + self.y as usize) & 0xFF;
                self.state = 0x278;
            }
            0x278 => {
                self.check_interrupts();
                self.stx();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x279 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.y as usize) & 0xFF;
                self.state = 0x27A;
            }
            0x27A => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.aax();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x27B => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x27C;
            }
            0x27C => {
                cache_interrupts!(self);
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x27D;
            }
            0x27D => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x27E => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x27F;
            }
            0x27F => {
                cache_interrupts!(self);
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x280;
            }
            0x280 => {
                self.check_interrupts();
                self.tas();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x281 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x282;
            }
            0x282 => {
                cache_interrupts!(self);
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x283;
            }
            0x283 => {
                self.check_interrupts();
                self.shy();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x284 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x285;
            }
            0x285 => {
                cache_interrupts!(self);
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x286;
            }
            0x286 => {
                self.check_interrupts();
                self.sta();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x287 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x288;
            }
            0x288 => {
                cache_interrupts!(self);
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x289;
            }
            0x289 => {
                self.check_interrupts();
                self.shx();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x28A => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x28B;
            }
            0x28B => {
                cache_interrupts!(self);
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x28C;
            }
            0x28C => {
                self.check_interrupts();
                self.ahx();
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x28D => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x28E
            }
            0x28E => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x28F
            }
            0x28F => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x290;
            }
            0x290 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x291 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x292
            }
            0x292 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x293
            }
            0x293 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x294;
            }
            0x294 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x295 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldy(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x296 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x297 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldx(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x298 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x299 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x29A
            }
            0x29A => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldy(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x29B => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x29C
            }
            0x29C => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x29D => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x29E
            }
            0x29E => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldx(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x29F => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x2A0
            }
            0x2A0 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x2A1 => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x2A2 } else { 0x100 };
            }
            0x2A2 => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x2A3 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x2A4;
            }
            0x2A4 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2A6
                } else {
                    0x2A5
                };
            }
            0x2A5 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2A6;
            }
            0x2A6 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2A8 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x2A9;
            }
            0x2A9 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2AB
                } else {
                    0x2AA
                };
            }
            0x2AA => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2AB;
            }
            0x2AB => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2AD => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2AE;
            }
            0x2AE => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldy(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2AF => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2B0;
            }
            0x2B0 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2B1 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.y as usize) & 0xFF;
                self.state = 0x2B2;
            }
            0x2B2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldx(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2B3 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.y as usize) & 0xFF;
                self.state = 0x2B4;
            }
            0x2B4 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2B5 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2B7
                } else {
                    0x2B6
                };
            }
            0x2B6 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2B7
            }
            0x2B7 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2B8 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2BA
                } else {
                    0x2B9
                };
            }
            0x2B9 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2BA
            }
            0x2BA => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.las(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2BB => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x2BD
                } else {
                    0x2BC
                };
            }
            0x2BC => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2BD
            }
            0x2BD => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldy(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2BE => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x2C0
                } else {
                    0x2BF
                };
            }
            0x2BF => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2C0
            }
            0x2C0 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lda(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2C1 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2C3
                } else {
                    0x2C2
                };
            }
            0x2C2 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2C3
            }
            0x2C3 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ldx(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2C4 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2C6
                } else {
                    0x2C5
                };
            }
            0x2C5 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2C6
            }
            0x2C6 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.lax(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2C7 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2C8
            }
            0x2C8 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x2C9
            }
            0x2C9 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x2CA;
            }
            0x2CA => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x2CB => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2CC;
            }
            0x2CC => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x2CD;
            }
            0x2CD => {
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x2CE;
            }
            0x2CE => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x2CF;
            }
            0x2CF => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dcp();
                self.state = 0x2D0;
            }
            0x2D0 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2D1 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cpy(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2D2 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2D3 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x2D4;
            }
            0x2D4 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dec(val);
                self.state = 0x2D5;
            }
            0x2D5 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2D6 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x2D7;
            }
            0x2D7 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dcp();
                self.state = 0x2D8;
            }
            0x2D8 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2D9 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x2DA
            }
            0x2DA => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cpy(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x2DB => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x2DC
            }
            0x2DC => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x2DD => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x2DE;
            }
            0x2DE => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x2DF;
            }
            0x2DF => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dec(val);
                self.state = 0x2E0;
            }
            0x2E0 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2E1 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x2E2;
            }
            0x2E2 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x2E3;
            }
            0x2E3 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dcp();
                self.state = 0x2E4;
            }
            0x2E4 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2E5 => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x2E6 } else { 0x100 };
            }
            0x2E6 => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x2E7 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x2E8;
            }
            0x2E8 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x2EA
                } else {
                    0x2E9
                };
            }
            0x2E9 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x2EA;
            }
            0x2EA => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2EC => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x2ED;
            }
            0x2ED => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x2EE;
            }
            0x2EE => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x2EF;
            }
            0x2EF => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x2F0;
            }
            0x2F0 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.dcp();
                self.state = 0x2F1;
            }
            0x2F1 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2F2 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2F3;
            }
            0x2F3 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2F4 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2F5;
            }
            0x2F5 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2F6 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2F7;
            }
            0x2F7 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x2F8;
            }
            0x2F8 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dec(val);
                self.state = 0x2F9;
            }
            0x2F9 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2FA => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x2FB;
            }
            0x2FB => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x2FC;
            }
            0x2FC => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dcp();
                self.state = 0x2FD;
            }
            0x2FD => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x2FE => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x300
                } else {
                    0x2FF
                };
            }
            0x2FF => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x300
            }
            0x300 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x301 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x302;
            }
            0x302 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                };
                self.state = 0x303;
            }
            0x303 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x304;
            }
            0x304 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.dcp();
                self.state = 0x305;
            }
            0x305 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x306 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x308
                } else {
                    0x307
                };
            }
            0x307 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x308
            }
            0x308 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x309 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x30B
                } else {
                    0x30A
                };
            }
            0x30A => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x30B
            }
            0x30B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cmp(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x30C => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x30D;
            }
            0x30D => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x30E;
            }
            0x30E => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x30F;
            }
            0x30F => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dec(val);
                self.state = 0x310;
            }
            0x310 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x311 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x312;
            }
            0x312 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x313;
            }
            0x313 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x314;
            }
            0x314 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.dcp();
                self.state = 0x315;
            }
            0x315 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x316 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x317
            }
            0x317 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x318
            }
            0x318 => {
                cache_interrupts!(self);
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x319;
            }
            0x319 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x31A => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x31B;
            }
            0x31B => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x31C;
            }
            0x31C => {
                self.ab = ((self.ram[self.ab] as usize) << 8) | self.temp;
                self.state = 0x31D;
            }
            0x31D => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x31E;
            }
            0x31E => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.isc();
                self.state = 0x31F;
            }
            0x31F => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x320 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cpx(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x321 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x322 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x323;
            }
            0x323 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.inc(val);
                self.state = 0x324;
            }
            0x324 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x325 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x326;
            }
            0x326 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.isc();
                self.state = 0x327;
            }
            0x327 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x328 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x329
            }
            0x329 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.cpx(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x32A => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x32B
            }
            0x32B => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x32C => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x32D;
            }
            0x32D => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x32E;
            }
            0x32E => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.inc(val);
                self.state = 0x32F;
            }
            0x32F => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x330 => {
                self.ab = ((read_ab!(self) as usize) << 8) | self.temp;
                self.pc += 1;
                self.state = 0x331;
            }
            0x331 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x332;
            }
            0x332 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.isc();
                self.state = 0x333;
            }
            0x333 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x334 => {
                cache_interrupts!(self);
                self.take_interrupt = false;
                read_ab!(self);
                self.take_branch();
                self.ab = self.pc;
                self.state = if self.temp != 0 { 0x335 } else { 0x100 };
            }
            0x335 => {
                self.check_interrupts();
                read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100
            }
            0x336 => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1usize) & 0xFF;
                self.state = 0x337;
            }
            0x337 => {
                cache_interrupts!(self);
                self.ab = (((self.ram[self.ab] as usize) << 8)
                    | ((self.temp + self.y as usize) & 0xFF)) as usize;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x339
                } else {
                    0x338
                };
            }
            0x338 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x339;
            }
            0x339 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x33B => {
                self.temp = self.ram[self.ab] as usize;
                self.ab = (self.ab + 1) & 0xFF;
                self.state = 0x33C;
            }
            0x33C => {
                self.ab =
                    ((self.ram[self.ab] as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.state = 0x33D;
            }
            0x33D => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x33E;
            }
            0x33E => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x33F;
            }
            0x33F => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.isc();
                self.state = 0x340;
            }
            0x340 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x341 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x342;
            }
            0x342 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x343 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x344;
            }
            0x344 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x345 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x346;
            }
            0x346 => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x347;
            }
            0x347 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.inc(val);
                self.state = 0x348;
            }
            0x348 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x349 => {
                self.ram[self.ab];
                self.ab = (self.ab + self.x as usize) & 0xFF;
                self.state = 0x34A;
            }
            0x34A => {
                self.temp = self.ram[self.ab] as usize;
                self.state = 0x34B;
            }
            0x34B => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.isc();
                self.state = 0x34C;
            }
            0x34C => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x34D => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.y as usize) < 0x100 {
                    0x34F
                } else {
                    0x34E
                };
            }
            0x34E => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x34F
            }
            0x34F => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x350 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x351;
            }
            0x351 => {
                read_ab!(self);
                if (self.temp + self.y as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                };
                self.state = 0x352;
            }
            0x352 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x353;
            }
            0x353 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                self.isc();
                self.state = 0x354;
            }
            0x354 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x355 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x357
                } else {
                    0x356
                };
            }
            0x356 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x357
            }
            0x357 => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x358 => {
                cache_interrupts!(self);
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = if (self.temp + self.x as usize) < 0x100 {
                    0x35A
                } else {
                    0x359
                };
            }
            0x359 => {
                cache_interrupts!(self);
                read_ab!(self);
                self.ab = (self.ab as u16).wrapping_add(0x100) as usize;
                self.state = 0x35A
            }
            0x35A => {
                self.check_interrupts();
                let val = read_ab!(self);
                self.sbc(val);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x35B => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x35C;
            }
            0x35C => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x35D;
            }
            0x35D => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x35E;
            }
            0x35E => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.inc(val);
                self.state = 0x35F;
            }
            0x35F => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            0x360 => {
                self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF);
                self.pc += 1;
                self.state = 0x361;
            }
            0x361 => {
                read_ab!(self);
                if (self.temp + self.x as usize) >= 0x100 {
                    self.ab = (self.ab as u16).wrapping_add(0x100) as usize
                };
                self.state = 0x362;
            }
            0x362 => {
                self.temp = read_ab!(self) as usize;
                self.state = 0x363;
            }
            0x363 => {
                cache_interrupts!(self);
                self.write(self.ab, self.temp as u8);
                let val = self.temp as u8;
                self.isc();
                self.state = 0x364;
            }
            0x364 => {
                self.check_interrupts();
                self.write(self.ab, self.temp as u8);
                self.ab = self.pc;
                self.state = 0x100;
            }
            op => unreachable!("propably a Rust compiler error, opcode: 0x{:X}", op),
        }
    }
}
