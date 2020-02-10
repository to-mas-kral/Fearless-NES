use super::Nes;
impl Nes {
    #[allow(unused_variables)]
    pub fn cpu_tick(&mut self) {
        if self.cpu.halt {
            return;
        }
        self.cpu.odd_cycle = !self.cpu.odd_cycle;
        if self.cpu.dma.oam || self.cpu.dma.dmc {
            self.cpu_dma();
            if self.cpu.dma.cycles != 0 {
                return;
            }
        }
        macro_rules! cache_interrupts {($self:ident) => {self.cpu.cached_irq = self.cpu.irq_signal; self.cpu.cached_nmi = self.cpu.nmi_signal; /*self.irq_signal = false; self.nmi_signal = false;*/}};
        macro_rules! check_dma {
            ($self:ident) => {
                if $self.cpu.dma.hijack_read {
                    self.cpu.dma.cycles = 1;
                    return;
                }
            };
        }
        macro_rules! read {
            ($self:ident, $addr: expr) => {
                $self.cpu_read($addr)
            };
        }
        macro_rules! read_ab {
            () => {
                read!(self, self.cpu.ab)
            };
        }
        macro_rules! sp_to_ab {
            ($self:ident) => {
                $self.cpu.ab = $self.cpu.sp | 0x100
            };
        }
        match self.cpu.state {
            0x0 => {
                read_ab!();
                check_dma!(self);
                let int = if self.cpu.take_interrupt { 0 } else { 1 };
                self.cpu.pc += int;
                sp_to_ab!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x101;
            }
            0x1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x106
            }
            0x2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x10A;
            }
            0x4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x110;
            }
            0x5 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x111;
            }
            0x6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x112;
            }
            0x7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x115;
            }
            0x8 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                sp_to_ab!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x118;
            }
            0x9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.asl_a();
                self.cpu.state = 0x100;
            }
            0xB => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.anc(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x119;
            }
            0xD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x11B;
            }
            0xE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x11D;
            }
            0xF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x121;
            }
            0x10 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if !self.cpu.n { 0x125 } else { 0x100 }
            }
            0x11 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x127;
            }
            0x12 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x13 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x12C;
            }
            0x14 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x132;
            }
            0x15 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x134;
            }
            0x16 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x136;
            }
            0x17 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x13A;
            }
            0x18 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.c = false;
                self.cpu.state = 0x100;
            }
            0x19 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x13E;
            }
            0x1A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0x1B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x141;
            }
            0x1C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x146;
            }
            0x1D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x149;
            }
            0x1E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x14C;
            }
            0x1F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x151;
            }
            0x20 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x156;
            }
            0x21 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x15A
            }
            0x22 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x23 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x15E;
            }
            0x24 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x164;
            }
            0x25 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x165;
            }
            0x26 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x166;
            }
            0x27 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x169;
            }
            0x28 => {
                read_ab!();
                check_dma!(self);
                sp_to_ab!(self);
                self.cpu.state = 0x16C;
            }
            0x29 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.rol_a();
                self.cpu.state = 0x100;
            }
            0x2B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.anc(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x16E;
            }
            0x2D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x170;
            }
            0x2E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x172;
            }
            0x2F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x176;
            }
            0x30 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.n { 0x17A } else { 0x100 }
            }
            0x31 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x17C;
            }
            0x32 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x33 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x181;
            }
            0x34 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x187;
            }
            0x35 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x189;
            }
            0x36 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x18B;
            }
            0x37 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x18F;
            }
            0x38 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.c = true;
                self.cpu.state = 0x100;
            }
            0x39 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x193;
            }
            0x3A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0x3B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x196;
            }
            0x3C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x19B;
            }
            0x3D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x19E;
            }
            0x3E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1A1;
            }
            0x3F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1A6;
            }
            0x40 => {
                read_ab!();
                check_dma!(self);
                sp_to_ab!(self);
                self.cpu.state = 0x1AB;
            }
            0x41 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1AF
            }
            0x42 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x43 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1B3;
            }
            0x44 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1B9;
            }
            0x45 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1BA;
            }
            0x46 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1BB;
            }
            0x47 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1BE;
            }
            0x48 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                sp_to_ab!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x1C1;
            }
            0x49 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x4A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.lsr_a();
                self.cpu.state = 0x100;
            }
            0x4B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.alr(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x4C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1C2
            }
            0x4D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1C3;
            }
            0x4E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1C5;
            }
            0x4F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1C9;
            }
            0x50 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if !self.cpu.v { 0x1CD } else { 0x100 }
            }
            0x51 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1CF;
            }
            0x52 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x53 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1D4;
            }
            0x54 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1DA;
            }
            0x55 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1DC;
            }
            0x56 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1DE;
            }
            0x57 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1E2;
            }
            0x58 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.i = false;
                self.cpu.state = 0x100;
            }
            0x59 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1E6;
            }
            0x5A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0x5B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1E9;
            }
            0x5C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1EE;
            }
            0x5D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1F1;
            }
            0x5E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1F4;
            }
            0x5F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x1F9;
            }
            0x60 => {
                read_ab!();
                check_dma!(self);
                sp_to_ab!(self);
                self.cpu.state = 0x1FE;
            }
            0x61 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x202
            }
            0x62 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x63 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x206;
            }
            0x64 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x20C;
            }
            0x65 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x20D;
            }
            0x66 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x20E;
            }
            0x67 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x211;
            }
            0x68 => {
                read_ab!();
                check_dma!(self);
                sp_to_ab!(self);
                self.cpu.state = 0x214;
            }
            0x69 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x6A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.ror_a();
                self.cpu.state = 0x100;
            }
            0x6B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.arr(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x6C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x216
            }
            0x6D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x219;
            }
            0x6E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x21B;
            }
            0x6F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x21F;
            }
            0x70 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.v { 0x223 } else { 0x100 }
            }
            0x71 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x225;
            }
            0x72 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x73 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x22A;
            }
            0x74 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x230;
            }
            0x75 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x232;
            }
            0x76 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x234;
            }
            0x77 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x238;
            }
            0x78 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.i = true;
                self.cpu.state = 0x100;
            }
            0x79 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x23C;
            }
            0x7A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0x7B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x23F;
            }
            0x7C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x244;
            }
            0x7D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x247;
            }
            0x7E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x24A;
            }
            0x7F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x24F;
            }
            0x80 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x81 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x254;
            }
            0x82 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x83 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x258
            }
            0x84 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x25C;
            }
            0x85 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x25D;
            }
            0x86 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x25E;
            }
            0x87 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x25F;
            }
            0x88 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.dey();
                self.cpu.state = 0x100;
            }
            0x89 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x8A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.txa();
                self.cpu.state = 0x100;
            }
            0x8B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.xaa(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x8C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x260;
            }
            0x8D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x262;
            }
            0x8E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x264;
            }
            0x8F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x266;
            }
            0x90 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if !self.cpu.c { 0x268 } else { 0x100 }
            }
            0x91 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x26A;
            }
            0x92 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x93 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x26E;
            }
            0x94 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x272;
            }
            0x95 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x274;
            }
            0x96 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x276;
            }
            0x97 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x278;
            }
            0x98 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.tya();
                self.cpu.state = 0x100;
            }
            0x99 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x27A;
            }
            0x9A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.txs();
                self.cpu.state = 0x100;
            }
            0x9B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x27D;
            }
            0x9C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x280;
            }
            0x9D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x283;
            }
            0x9E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x286;
            }
            0x9F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x289;
            }
            0xA0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldy(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xA1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x28C
            }
            0xA2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldx(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xA3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x290
            }
            0xA4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x294;
            }
            0xA5 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x295;
            }
            0xA6 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x296;
            }
            0xA7 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x297;
            }
            0xA8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.tay();
                self.cpu.state = 0x100;
            }
            0xA9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xAA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.tax();
                self.cpu.state = 0x100;
            }
            0xAB => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xAC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x298;
            }
            0xAD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x29A;
            }
            0xAE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x29C;
            }
            0xAF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x29E;
            }
            0xB0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.c { 0x2A0 } else { 0x100 }
            }
            0xB1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2A2;
            }
            0xB2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xB3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2A7;
            }
            0xB4 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2AC;
            }
            0xB5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2AE;
            }
            0xB6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2B0;
            }
            0xB7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2B2;
            }
            0xB8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.v = false;
                self.cpu.state = 0x100;
            }
            0xB9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2B4;
            }
            0xBA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.tsx();
                self.cpu.state = 0x100;
            }
            0xBB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2B7;
            }
            0xBC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2BA;
            }
            0xBD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2BD;
            }
            0xBE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2C0;
            }
            0xBF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2C3;
            }
            0xC0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpy(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xC1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2C6
            }
            0xC2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xC3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2CA;
            }
            0xC4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2D0;
            }
            0xC5 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2D1;
            }
            0xC6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2D2;
            }
            0xC7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2D5;
            }
            0xC8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.iny();
                self.cpu.state = 0x100;
            }
            0xC9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xCA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.dex();
                self.cpu.state = 0x100;
            }
            0xCB => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.axs(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xCC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2D8;
            }
            0xCD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2DA;
            }
            0xCE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2DC;
            }
            0xCF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2E0;
            }
            0xD0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if !self.cpu.z { 0x2E4 } else { 0x100 }
            }
            0xD1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2E6;
            }
            0xD2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xD3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2EB;
            }
            0xD4 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2F1;
            }
            0xD5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2F3;
            }
            0xD6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2F5;
            }
            0xD7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2F9;
            }
            0xD8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.d = false;
                self.cpu.state = 0x100;
            }
            0xD9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x2FD;
            }
            0xDA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0xDB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x300;
            }
            0xDC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x305;
            }
            0xDD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x308;
            }
            0xDE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x30B;
            }
            0xDF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x310;
            }
            0xE0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpx(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xE1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x315
            }
            0xE2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xE3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x319;
            }
            0xE4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x31F;
            }
            0xE5 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x320;
            }
            0xE6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x321;
            }
            0xE7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x324;
            }
            0xE8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.inx();
                self.cpu.state = 0x100;
            }
            0xE9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xEA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0xEB => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xEC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x327;
            }
            0xED => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x329;
            }
            0xEE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x32B;
            }
            0xEF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x32F;
            }
            0xF0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.z { 0x333 } else { 0x100 }
            }
            0xF1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x335;
            }
            0xF2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.halt = true;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0xF3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x33A;
            }
            0xF4 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x340;
            }
            0xF5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x342;
            }
            0xF6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x344;
            }
            0xF7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x348;
            }
            0xF8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.d = true;
                self.cpu.state = 0x100;
            }
            0xF9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x34C;
            }
            0xFA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.state = 0x100;
            }
            0xFB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x34F;
            }
            0xFC => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x354;
            }
            0xFD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x357;
            }
            0xFE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x35A;
            }
            0xFF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x35F;
            }
            0x100 => {
                cache_interrupts!(self);
                let int = if self.cpu.take_interrupt { 0 } else { 1 };
                read_ab!();
                check_dma!(self);
                self.cpu.state = u16::from(int * self.cpu.db);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(int as u16) as usize;
                self.cpu.ab = self.cpu.pc
            }
            0x101 => {
                if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
                    self.cpu_write(self.cpu.ab, (self.cpu.pc >> 8) as u8);
                }
                sp_to_ab!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x102;
            }
            0x102 => {
                if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
                    self.cpu_write(self.cpu.ab, (self.cpu.pc & 0xFF) as u8);
                }
                sp_to_ab!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x103;
            }
            0x103 => {
                if !(self.cpu.take_interrupt && self.cpu.reset_signal) {
                    self.cpu_push_status(true);
                }
                self.cpu.ab = self.cpu_interrupt_address();
                self.cpu.take_interrupt = false;
                self.cpu.interrupt_type = super::InterruptType::None;
                self.cpu.state = 0x104;
            }
            0x104 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab += 1;
                self.cpu.i = true;
                self.cpu.state = 0x105;
            }
            0x105 => {
                read_ab!();
                check_dma!(self);
                self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x106 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x107
            }
            0x107 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x108
            }
            0x108 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x109;
            }
            0x109 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x10A => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x10B;
            }
            0x10B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x10C;
            }
            0x10C => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x10D;
            }
            0x10D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x10E;
            }
            0x10E => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.slo();
                self.cpu.state = 0x10F;
            }
            0x10F => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x110 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x111 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x112 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x113;
            }
            0x113 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.asl(val);
                self.cpu.state = 0x114;
            }
            0x114 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x115 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x116;
            }
            0x116 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.slo();
                self.cpu.state = 0x117;
            }
            0x117 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x118 => {
                self.cpu_check_interrupts();
                self.cpu_push_status(true);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x119 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x11A
            }
            0x11A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x11B => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x11C
            }
            0x11C => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x11D => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x11E;
            }
            0x11E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x11F;
            }
            0x11F => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.asl(val);
                self.cpu.state = 0x120;
            }
            0x120 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x121 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x122;
            }
            0x122 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x123;
            }
            0x123 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.slo();
                self.cpu.state = 0x124;
            }
            0x124 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x125 => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x126 } else { 0x100 };
            }
            0x126 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x127 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x128;
            }
            0x128 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x12A
                } else {
                    0x129
                };
            }
            0x129 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x12A;
            }
            0x12A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x12C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x12D;
            }
            0x12D => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x12E;
            }
            0x12E => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x12F;
            }
            0x12F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x130;
            }
            0x130 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.slo();
                self.cpu.state = 0x131;
            }
            0x131 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x132 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x133;
            }
            0x133 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x134 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x135;
            }
            0x135 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x136 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x137;
            }
            0x137 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x138;
            }
            0x138 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.asl(val);
                self.cpu.state = 0x139;
            }
            0x139 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x13A => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x13B;
            }
            0x13B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x13C;
            }
            0x13C => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.slo();
                self.cpu.state = 0x13D;
            }
            0x13D => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x13E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x140
                } else {
                    0x13F
                };
            }
            0x13F => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x140
            }
            0x140 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x141 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x142;
            }
            0x142 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                };
                self.cpu.state = 0x143;
            }
            0x143 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x144;
            }
            0x144 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.slo();
                self.cpu.state = 0x145;
            }
            0x145 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x146 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x148
                } else {
                    0x147
                };
            }
            0x147 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x148
            }
            0x148 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x149 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x14B
                } else {
                    0x14A
                };
            }
            0x14A => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x14B
            }
            0x14B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ora(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x14C => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x14D;
            }
            0x14D => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x14E;
            }
            0x14E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x14F;
            }
            0x14F => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.asl(val);
                self.cpu.state = 0x150;
            }
            0x150 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x151 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x152;
            }
            0x152 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x153;
            }
            0x153 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x154;
            }
            0x154 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.slo();
                self.cpu.state = 0x155;
            }
            0x155 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x156 => {
                read_ab!();
                check_dma!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x157;
            }
            0x157 => {
                self.cpu_write(self.cpu.ab, (self.cpu.pc >> 8) as u8);
                sp_to_ab!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize;
                self.cpu.state = 0x158;
            }
            0x158 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, (self.cpu.pc & 0xFF) as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x159;
            }
            0x159 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x15A => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x15B
            }
            0x15B => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x15C
            }
            0x15C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x15D;
            }
            0x15D => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x15E => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x15F;
            }
            0x15F => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x160;
            }
            0x160 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x161;
            }
            0x161 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x162;
            }
            0x162 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rla();
                self.cpu.state = 0x163;
            }
            0x163 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x164 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.bit(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x165 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x166 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x167;
            }
            0x167 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rol(val);
                self.cpu.state = 0x168;
            }
            0x168 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x169 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x16A;
            }
            0x16A => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rla();
                self.cpu.state = 0x16B;
            }
            0x16B => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x16C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x16D;
            }
            0x16D => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu_pull_status(self.cpu.db);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x16E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x16F
            }
            0x16F => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.bit(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x170 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x171
            }
            0x171 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x172 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x173;
            }
            0x173 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x174;
            }
            0x174 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rol(val);
                self.cpu.state = 0x175;
            }
            0x175 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x176 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x177;
            }
            0x177 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x178;
            }
            0x178 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rla();
                self.cpu.state = 0x179;
            }
            0x179 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x17A => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x17B } else { 0x100 };
            }
            0x17B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x17C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x17D;
            }
            0x17D => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x17F
                } else {
                    0x17E
                };
            }
            0x17E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x17F;
            }
            0x17F => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x181 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x182;
            }
            0x182 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x183;
            }
            0x183 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x184;
            }
            0x184 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x185;
            }
            0x185 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.rla();
                self.cpu.state = 0x186;
            }
            0x186 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x187 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x188;
            }
            0x188 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x189 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x18A;
            }
            0x18A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x18B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x18C;
            }
            0x18C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x18D;
            }
            0x18D => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rol(val);
                self.cpu.state = 0x18E;
            }
            0x18E => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x18F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x190;
            }
            0x190 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x191;
            }
            0x191 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rla();
                self.cpu.state = 0x192;
            }
            0x192 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x193 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x195
                } else {
                    0x194
                };
            }
            0x194 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x195
            }
            0x195 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x196 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x197;
            }
            0x197 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                };
                self.cpu.state = 0x198;
            }
            0x198 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x199;
            }
            0x199 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.rla();
                self.cpu.state = 0x19A;
            }
            0x19A => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x19B => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x19D
                } else {
                    0x19C
                };
            }
            0x19C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x19D
            }
            0x19D => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x19E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x1A0
                } else {
                    0x19F
                };
            }
            0x19F => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x1A0
            }
            0x1A0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.and(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1A1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1A2;
            }
            0x1A2 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x1A3;
            }
            0x1A3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1A4;
            }
            0x1A4 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rol(val);
                self.cpu.state = 0x1A5;
            }
            0x1A5 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1A6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1A7;
            }
            0x1A7 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x1A8;
            }
            0x1A8 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1A9;
            }
            0x1A9 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rla();
                self.cpu.state = 0x1AA;
            }
            0x1AA => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1AB => {
                read_ab!();
                check_dma!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x1AC;
            }
            0x1AC => {
                read_ab!();
                check_dma!(self);
                self.cpu_pull_status(self.cpu.db);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x1AD;
            }
            0x1AD => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x1AE;
            }
            0x1AE => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1AF => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x1B0
            }
            0x1B0 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x1B1
            }
            0x1B1 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x1B2;
            }
            0x1B2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x1B3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x1B4;
            }
            0x1B4 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x1B5;
            }
            0x1B5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x1B6;
            }
            0x1B6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1B7;
            }
            0x1B7 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.sre(val);
                self.cpu.state = 0x1B8;
            }
            0x1B8 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1B9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1BA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1BB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1BC;
            }
            0x1BC => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.lsr(val);
                self.cpu.state = 0x1BD;
            }
            0x1BD => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1BE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1BF;
            }
            0x1BF => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.sre(val);
                self.cpu.state = 0x1C0;
            }
            0x1C0 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1C1 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.a);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1C2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x1C3 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1C4
            }
            0x1C4 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x1C5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1C6;
            }
            0x1C6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1C7;
            }
            0x1C7 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.lsr(val);
                self.cpu.state = 0x1C8;
            }
            0x1C8 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1C9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1CA;
            }
            0x1CA => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1CB;
            }
            0x1CB => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.sre(val);
                self.cpu.state = 0x1CC;
            }
            0x1CC => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1CD => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x1CE } else { 0x100 };
            }
            0x1CE => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x1CF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x1D0;
            }
            0x1D0 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x1D2
                } else {
                    0x1D1
                };
            }
            0x1D1 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x1D2;
            }
            0x1D2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1D4 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x1D5;
            }
            0x1D5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x1D6;
            }
            0x1D6 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x1D7;
            }
            0x1D7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1D8;
            }
            0x1D8 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.sre(val);
                self.cpu.state = 0x1D9;
            }
            0x1D9 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1DA => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x1DB;
            }
            0x1DB => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1DC => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x1DD;
            }
            0x1DD => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1DE => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x1DF;
            }
            0x1DF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1E0;
            }
            0x1E0 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.lsr(val);
                self.cpu.state = 0x1E1;
            }
            0x1E1 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1E2 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x1E3;
            }
            0x1E3 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1E4;
            }
            0x1E4 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.sre(val);
                self.cpu.state = 0x1E5;
            }
            0x1E5 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1E6 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x1E8
                } else {
                    0x1E7
                };
            }
            0x1E7 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x1E8
            }
            0x1E8 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1E9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1EA;
            }
            0x1EA => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                };
                self.cpu.state = 0x1EB;
            }
            0x1EB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1EC;
            }
            0x1EC => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.sre(val);
                self.cpu.state = 0x1ED;
            }
            0x1ED => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1EE => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x1F0
                } else {
                    0x1EF
                };
            }
            0x1EF => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x1F0
            }
            0x1F0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1F1 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x1F3
                } else {
                    0x1F2
                };
            }
            0x1F2 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x1F3
            }
            0x1F3 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.eor(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1F4 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1F5;
            }
            0x1F5 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x1F6;
            }
            0x1F6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1F7;
            }
            0x1F7 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.lsr(val);
                self.cpu.state = 0x1F8;
            }
            0x1F8 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1F9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x1FA;
            }
            0x1FA => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x1FB;
            }
            0x1FB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x1FC;
            }
            0x1FC => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.sre(val);
                self.cpu.state = 0x1FD;
            }
            0x1FD => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x1FE => {
                read_ab!();
                check_dma!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x1FF
            }
            0x1FF => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x200;
            }
            0x200 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x201;
            }
            0x201 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x202 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x203
            }
            0x203 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x204
            }
            0x204 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x205;
            }
            0x205 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x206 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x207;
            }
            0x207 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x208;
            }
            0x208 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x209;
            }
            0x209 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x20A;
            }
            0x20A => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rra(val);
                self.cpu.state = 0x20B;
            }
            0x20B => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x20C => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x20D => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x20E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x20F;
            }
            0x20F => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.ror(val);
                self.cpu.state = 0x210;
            }
            0x210 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x211 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x212;
            }
            0x212 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rra(val);
                self.cpu.state = 0x213;
            }
            0x213 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x214 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize;
                sp_to_ab!(self);
                self.cpu.state = 0x215;
            }
            0x215 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.lda(self.cpu.db);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x216 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x217
            }
            0x217 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab & 0xFF00) | ((self.cpu.ab + 1) & 0xFF);
                self.cpu.state = 0x218
            }
            0x218 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x219 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x21A
            }
            0x21A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x21B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x21C;
            }
            0x21C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x21D;
            }
            0x21D => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.ror(val);
                self.cpu.state = 0x21E;
            }
            0x21E => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x21F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x220;
            }
            0x220 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x221;
            }
            0x221 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rra(val);
                self.cpu.state = 0x222;
            }
            0x222 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x223 => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x224 } else { 0x100 };
            }
            0x224 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x225 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x226;
            }
            0x226 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x228
                } else {
                    0x227
                };
            }
            0x227 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x228;
            }
            0x228 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x22A => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x22B;
            }
            0x22B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x22C;
            }
            0x22C => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x22D;
            }
            0x22D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x22E;
            }
            0x22E => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.rra(val);
                self.cpu.state = 0x22F;
            }
            0x22F => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x230 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x231;
            }
            0x231 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x232 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x233;
            }
            0x233 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x234 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x235;
            }
            0x235 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x236;
            }
            0x236 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.ror(val);
                self.cpu.state = 0x237;
            }
            0x237 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x238 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x239;
            }
            0x239 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x23A;
            }
            0x23A => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rra(val);
                self.cpu.state = 0x23B;
            }
            0x23B => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x23C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x23E
                } else {
                    0x23D
                };
            }
            0x23D => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x23E
            }
            0x23E => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x23F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x240;
            }
            0x240 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                };
                self.cpu.state = 0x241;
            }
            0x241 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x242;
            }
            0x242 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.rra(val);
                self.cpu.state = 0x243;
            }
            0x243 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x244 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x246
                } else {
                    0x245
                };
            }
            0x245 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x246
            }
            0x246 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x247 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x249
                } else {
                    0x248
                };
            }
            0x248 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x249
            }
            0x249 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.adc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x24A => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x24B;
            }
            0x24B => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x24C;
            }
            0x24C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x24D;
            }
            0x24D => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.ror(val);
                self.cpu.state = 0x24E;
            }
            0x24E => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x24F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x250;
            }
            0x250 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x251;
            }
            0x251 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x252;
            }
            0x252 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.rra(val);
                self.cpu.state = 0x253;
            }
            0x253 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x254 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x255;
            }
            0x255 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x256;
            }
            0x256 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x257;
            }
            0x257 => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x258 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x259
            }
            0x259 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x25A
            }
            0x25A => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x25B;
            }
            0x25B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.aax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x25C => {
                self.cpu_check_interrupts();
                self.sty();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x25D => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x25E => {
                self.cpu_check_interrupts();
                self.stx();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x25F => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.aax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x260 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x261
            }
            0x261 => {
                self.cpu_check_interrupts();
                self.sty();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x262 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x263
            }
            0x263 => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x264 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x265
            }
            0x265 => {
                self.cpu_check_interrupts();
                self.stx();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x266 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x267
            }
            0x267 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.aax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x268 => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x269 } else { 0x100 };
            }
            0x269 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x26A => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x26B;
            }
            0x26B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x26C;
            }
            0x26C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if self.cpu.temp + self.cpu.y as usize >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                }
                self.cpu.state = 0x26D;
            }
            0x26D => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x26E => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x26F;
            }
            0x26F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x270;
            }
            0x270 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if self.cpu.temp + self.cpu.y as usize >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                }
                self.cpu.state = 0x271;
            }
            0x271 => {
                self.cpu_check_interrupts();
                self.ahx();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x272 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x273;
            }
            0x273 => {
                self.cpu_check_interrupts();
                self.sty();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x274 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x275;
            }
            0x275 => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x276 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF;
                self.cpu.state = 0x277;
            }
            0x277 => {
                self.cpu_check_interrupts();
                self.stx();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x278 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF;
                self.cpu.state = 0x279;
            }
            0x279 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.aax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x27A => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x27B;
            }
            0x27B => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x27C;
            }
            0x27C => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x27D => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x27E;
            }
            0x27E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x27F;
            }
            0x27F => {
                self.cpu_check_interrupts();
                self.tas();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x280 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x281;
            }
            0x281 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x282;
            }
            0x282 => {
                self.cpu_check_interrupts();
                self.shy();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x283 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x284;
            }
            0x284 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x285;
            }
            0x285 => {
                self.cpu_check_interrupts();
                self.sta();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x286 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x287;
            }
            0x287 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x288;
            }
            0x288 => {
                self.cpu_check_interrupts();
                self.shx();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x289 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x28A;
            }
            0x28A => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x28B;
            }
            0x28B => {
                self.cpu_check_interrupts();
                self.ahx();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x28C => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x28D
            }
            0x28D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x28E
            }
            0x28E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x28F;
            }
            0x28F => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x290 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x291
            }
            0x291 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x292
            }
            0x292 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x293;
            }
            0x293 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x294 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldy(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x295 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x296 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldx(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x297 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x298 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x299
            }
            0x299 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldy(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x29A => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x29B
            }
            0x29B => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x29C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x29D
            }
            0x29D => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldx(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x29E => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x29F
            }
            0x29F => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x2A0 => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x2A1 } else { 0x100 };
            }
            0x2A1 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x2A2 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x2A3;
            }
            0x2A3 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2A5
                } else {
                    0x2A4
                };
            }
            0x2A4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2A5;
            }
            0x2A5 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2A7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x2A8;
            }
            0x2A8 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2AA
                } else {
                    0x2A9
                };
            }
            0x2A9 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2AA;
            }
            0x2AA => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2AC => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2AD;
            }
            0x2AD => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldy(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2AE => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2AF;
            }
            0x2AF => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2B0 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF;
                self.cpu.state = 0x2B1;
            }
            0x2B1 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldx(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2B2 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF;
                self.cpu.state = 0x2B3;
            }
            0x2B3 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2B4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2B6
                } else {
                    0x2B5
                };
            }
            0x2B5 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2B6
            }
            0x2B6 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2B7 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2B9
                } else {
                    0x2B8
                };
            }
            0x2B8 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2B9
            }
            0x2B9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.las(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2BA => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x2BC
                } else {
                    0x2BB
                };
            }
            0x2BB => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2BC
            }
            0x2BC => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldy(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2BD => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x2BF
                } else {
                    0x2BE
                };
            }
            0x2BE => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2BF
            }
            0x2BF => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lda(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2C0 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2C2
                } else {
                    0x2C1
                };
            }
            0x2C1 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2C2
            }
            0x2C2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.ldx(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2C3 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2C5
                } else {
                    0x2C4
                };
            }
            0x2C4 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2C5
            }
            0x2C5 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.lax(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2C6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2C7
            }
            0x2C7 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x2C8
            }
            0x2C8 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x2C9;
            }
            0x2C9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x2CA => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2CB;
            }
            0x2CB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x2CC;
            }
            0x2CC => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x2CD;
            }
            0x2CD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2CE;
            }
            0x2CE => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dcp(val);
                self.cpu.state = 0x2CF;
            }
            0x2CF => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2D0 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpy(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2D1 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2D2 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2D3;
            }
            0x2D3 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dec(val);
                self.cpu.state = 0x2D4;
            }
            0x2D4 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2D5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2D6;
            }
            0x2D6 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dcp(val);
                self.cpu.state = 0x2D7;
            }
            0x2D7 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2D8 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2D9
            }
            0x2D9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpy(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x2DA => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2DB
            }
            0x2DB => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x2DC => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2DD;
            }
            0x2DD => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2DE;
            }
            0x2DE => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dec(val);
                self.cpu.state = 0x2DF;
            }
            0x2DF => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2E0 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x2E1;
            }
            0x2E1 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2E2;
            }
            0x2E2 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dcp(val);
                self.cpu.state = 0x2E3;
            }
            0x2E3 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2E4 => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x2E5 } else { 0x100 };
            }
            0x2E5 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x2E6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x2E7;
            }
            0x2E7 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2E9
                } else {
                    0x2E8
                };
            }
            0x2E8 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2E9;
            }
            0x2E9 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2EB => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x2EC;
            }
            0x2EC => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x2ED;
            }
            0x2ED => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x2EE;
            }
            0x2EE => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2EF;
            }
            0x2EF => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.dcp(val);
                self.cpu.state = 0x2F0;
            }
            0x2F0 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2F1 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2F2;
            }
            0x2F2 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2F3 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2F4;
            }
            0x2F4 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2F5 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2F6;
            }
            0x2F6 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2F7;
            }
            0x2F7 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dec(val);
                self.cpu.state = 0x2F8;
            }
            0x2F8 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2F9 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x2FA;
            }
            0x2FA => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x2FB;
            }
            0x2FB => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dcp(val);
                self.cpu.state = 0x2FC;
            }
            0x2FC => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x2FD => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x2FF
                } else {
                    0x2FE
                };
            }
            0x2FE => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x2FF
            }
            0x2FF => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x300 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x301;
            }
            0x301 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                };
                self.cpu.state = 0x302;
            }
            0x302 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x303;
            }
            0x303 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.dcp(val);
                self.cpu.state = 0x304;
            }
            0x304 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x305 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x307
                } else {
                    0x306
                };
            }
            0x306 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x307
            }
            0x307 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x308 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x30A
                } else {
                    0x309
                };
            }
            0x309 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x30A
            }
            0x30A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cmp(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x30B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x30C;
            }
            0x30C => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x30D;
            }
            0x30D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x30E;
            }
            0x30E => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dec(val);
                self.cpu.state = 0x30F;
            }
            0x30F => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x310 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x311;
            }
            0x311 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x312;
            }
            0x312 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x313;
            }
            0x313 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.dcp(val);
                self.cpu.state = 0x314;
            }
            0x314 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x315 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x316
            }
            0x316 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x317
            }
            0x317 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x318;
            }
            0x318 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x319 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x31A;
            }
            0x31A => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x31B;
            }
            0x31B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.state = 0x31C;
            }
            0x31C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x31D;
            }
            0x31D => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.isc(val);
                self.cpu.state = 0x31E;
            }
            0x31E => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x31F => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpx(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x320 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x321 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x322;
            }
            0x322 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.inc(val);
                self.cpu.state = 0x323;
            }
            0x323 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x324 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x325;
            }
            0x325 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.isc(val);
                self.cpu.state = 0x326;
            }
            0x326 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x327 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x328
            }
            0x328 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpx(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x329 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x32A
            }
            0x32A => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x32B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x32C;
            }
            0x32C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x32D;
            }
            0x32D => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.inc(val);
                self.cpu.state = 0x32E;
            }
            0x32E => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x32F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp;
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x330;
            }
            0x330 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x331;
            }
            0x331 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.isc(val);
                self.cpu.state = 0x332;
            }
            0x332 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x333 => {
                cache_interrupts!(self);
                self.cpu.take_interrupt = false;
                read_ab!();
                check_dma!(self);
                self.cpu_take_branch();
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = if self.cpu.temp != 0 { 0x334 } else { 0x100 };
            }
            0x334 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100
            }
            0x335 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF;
                self.cpu.state = 0x336;
            }
            0x336 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF))
                    as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x338
                } else {
                    0x337
                };
            }
            0x337 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x338;
            }
            0x338 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x33A => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.ab = (self.cpu.ab + 1) & 0xFF;
                self.cpu.state = 0x33B;
            }
            0x33B => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.state = 0x33C;
            }
            0x33C => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x33D;
            }
            0x33D => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x33E;
            }
            0x33E => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.isc(val);
                self.cpu.state = 0x33F;
            }
            0x33F => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x340 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x341;
            }
            0x341 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x342 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x343;
            }
            0x343 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x344 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x345;
            }
            0x345 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x346;
            }
            0x346 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.inc(val);
                self.cpu.state = 0x347;
            }
            0x347 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x348 => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF;
                self.cpu.state = 0x349;
            }
            0x349 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x34A;
            }
            0x34A => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.isc(val);
                self.cpu.state = 0x34B;
            }
            0x34B => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x34C => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {
                    0x34E
                } else {
                    0x34D
                };
            }
            0x34D => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x34E
            }
            0x34E => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x34F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.y as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x350;
            }
            0x350 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                };
                self.cpu.state = 0x351;
            }
            0x351 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x352;
            }
            0x352 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.isc(val);
                self.cpu.state = 0x353;
            }
            0x353 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x354 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x356
                } else {
                    0x355
                };
            }
            0x355 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x356
            }
            0x356 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x357 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = if (self.cpu.temp + self.cpu.x as usize) < 0x100 {
                    0x359
                } else {
                    0x358
                };
            }
            0x358 => {
                cache_interrupts!(self);
                read_ab!();
                check_dma!(self);
                self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;
                self.cpu.state = 0x359
            }
            0x359 => {
                self.cpu_check_interrupts();
                read_ab!();
                check_dma!(self);
                let val = self.cpu.db;
                self.sbc(val);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x35A => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x35B;
            }
            0x35B => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x35C;
            }
            0x35C => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x35D;
            }
            0x35D => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.inc(val);
                self.cpu.state = 0x35E;
            }
            0x35E => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            0x35F => {
                read_ab!();
                check_dma!(self);
                self.cpu.ab = ((self.cpu.db as usize) << 8)
                    | ((self.cpu.temp + self.cpu.x as usize) & 0xFF);
                self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize;
                self.cpu.state = 0x360;
            }
            0x360 => {
                read_ab!();
                check_dma!(self);
                if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {
                    self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize
                };
                self.cpu.state = 0x361;
            }
            0x361 => {
                read_ab!();
                check_dma!(self);
                self.cpu.temp = self.cpu.db as usize;
                self.cpu.state = 0x362;
            }
            0x362 => {
                cache_interrupts!(self);
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                let val = self.cpu.temp as u8;
                self.isc(val);
                self.cpu.state = 0x363;
            }
            0x363 => {
                self.cpu_check_interrupts();
                self.cpu_write(self.cpu.ab, self.cpu.temp as u8);
                self.cpu.ab = self.cpu.pc;
                self.cpu.state = 0x100;
            }
            op => unreachable!("invalid state, opcode: 0x{:X}", op),
        }
    }
}
