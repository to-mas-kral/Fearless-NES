use super::Nes;

mod state_machine;

pub trait Tick {
    fn tick(&mut self);
}

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
    a: u8,         //Accumulator
    x: u8,         //X index
    y: u8,         //Y index
    pub pc: usize, //Program counter (16 bits)
    pub sp: usize, //Stack pointer (8 bits)

    n: bool, //Negative flag
    v: bool, //Overflow flag
    i: bool, //Interrupt inhibit
    z: bool, //Zero flag
    c: bool, //Carry flag
    d: bool, //BCD flag, this doesn't do anything on the NES CPU

    pub halt: bool,
    pub state: u16,
    odd_cycle: bool,

    pub ab: usize, //Address bus
    db: u8,        //Data bus
    temp: usize,
    open_bus: u8,

    cached_irq: bool,
    pub irq_signal: bool,
    cached_nmi: bool,
    pub nmi_signal: bool,
    reset_signal: bool,
    take_interrupt: bool,
    interrupt_type: InterruptType,

    ram: [u8; 0x800],
    dma: Dma,

    pub nes: *mut Nes,
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

            ram: [0; 0x800],
            dma: Dma::new(),
            open_bus: 0,

            nes: 0 as *mut Nes,
        }
    }

    pub fn debug_info(&mut self) -> String {
        format!(
            "A: 0x{:X}, X: 0x{:X}, Y: 0x{:X}, pc: 0x{:X}, sp: 0x{:X}, ab: 0x{:X}",
            self.a, self.x, self.y, self.pc, self.sp, self.ab
        )
    }
    pub fn gen_reset(&mut self) {
        self.state = 0;
        self.take_interrupt = true;
        self.reset_signal = true;
        self.interrupt_type = InterruptType::Reset;
        self.write(0x4015, 0);
        self.reset_signal = false;
    }

    #[inline]
    fn check_interrupts(&mut self) {
        if !self.i && self.cached_irq {
            self.cached_irq = false;
            self.nmi_signal = false;
            self.take_interrupt = true;
            self.interrupt_type = InterruptType::Irq;
        }

        if self.cached_nmi {
            self.cached_nmi = false;
            self.nmi_signal = false;
            self.take_interrupt = true;
            self.interrupt_type = InterruptType::Nmi;
        }

        //TODO: resets
    }

    #[inline]
    fn interrupt_address(&mut self) -> usize {
        if self.nmi_signal {
            return 0xFFFA;
        }

        match self.interrupt_type {
            InterruptType::Irq | InterruptType::None => 0xFFFE,
            InterruptType::Nmi => 0xFFFA,
            InterruptType::Reset => 0xFFFC,
        }
    }

    #[inline]
    pub fn read(&mut self, index: usize) {
        self.open_bus = match index {
            0..=0x1FFF => self.ram[index & 0x7FF],
            0x2000..=0x3FFF => nes!(self.nes).ppu.read_reg(index),
            0x4000..=0x4014 => self.open_bus,
            0x4015 => nes!(self.nes).apu.read_status(),
            0x4016 => {
                let tmp = nes!(self.nes).controller.read_reg();
                (self.open_bus & 0xE0) | tmp
            }
            0x4017..=0x401F => self.open_bus,
            0x4020..=0xFFFF => {
                if let Some(val) = nes!(self.nes).mapper.cpu_read(index) {
                    val
                } else {
                    self.open_bus
                }
            }
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        };

        debug_log!(
            "memory map - reading 0x{:X} from 0x{:X}",
            (self.open_bus),
            index
        );

        self.db = self.open_bus;
    }

    #[inline]
    pub fn write(&mut self, index: usize, val: u8) {
        debug_log!("memory map - writing 0x{:X} to 0x{:X}", val, index);

        match index {
            0..=0x1FFF => self.ram[index & 0x7FF] = val,
            0x2000..=0x3FFF => nes!(self.nes).ppu.write_reg(index, val),
            0x4000..=0x4013 => nes!(self.nes).apu.write_reg(index, val),
            0x4014 => {
                self.dma.oam = true;
                self.dma.addr = (val as usize) << 8;
            }
            0x4015 => nes!(self.nes).apu.write_reg(index, val),
            0x4016 => nes!(self.nes).controller.write_reg(val),
            0x4017 => nes!(self.nes).apu.write_reg(index, val),
            0x4018..=0x401F => (),
            0x4020..=0xFFFF => nes!(self.nes).mapper.cpu_write(index, val),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    #[inline]
    pub fn peek(&mut self, index: usize) -> u8 {
        debug_log!("memory map - reading direct from 0x{:X}", index);
        match index {
            0..=0x1FFF => self.ram[index & 0x7FF],
            0x4020..=0xFFFF => nes!(self.nes).mapper.cpu_peek(index),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    //forums.nesdev.com/viewtopic.php?f=3&t=14120
    #[inline]
    pub fn dma(&mut self) {
        if self.dma.cycles == 0 {
            self.dma.hijack_read = true;
        } else {
            self.dma.hijack_read = false;
            if self.dma.oam {
                if self.dma.cycles == 1 && self.odd_cycle {
                    self.read(self.ab);
                } else {
                    if self.dma.cycles & 1 != 0 {
                        self.read(self.dma.addr);
                        self.dma.addr += 1;
                        self.dma.copy_buffer = self.db
                    } else {
                        self.write(0x2004, self.dma.copy_buffer);
                    }
                    self.dma.cycles += 1;

                    if self.dma.cycles == 0x201 {
                        self.dma.oam = false;
                        self.dma.cycles = 0;
                    }
                }
            }
            if self.dma.dmc {
                unimplemented!("DMC DMA is unimplemented");
            }
        }
    }

    #[inline]
    fn adc(&mut self, num: u8) {
        let a = self.a;
        let b = num;
        let carry =
            (u16::from(num) + u16::from(self.a) + (if self.c { 1 } else { 0 })) & (1 << 8) != 0;
        let num: i8 = (num as i8).wrapping_add(if self.c { 1 } else { 0 });
        let num: i8 = (num as i8).wrapping_add(self.a as i8);
        self.a = num as u8;
        self.v = (a ^ b) & (0x80) == 0 && (a ^ self.a) & 0x80 != 0;
        self.c = carry;
        self.set_z_n(self.a);
    }

    #[inline]
    fn sbc(&mut self, num: u8) {
        self.adc(!num);
    }

    #[inline]
    fn asl(&mut self, mut num: u8) {
        self.c = num & (1 << 7) != 0;
        num <<= 1;
        self.set_z_n(num);
        self.temp = num as usize;
    }

    #[inline]
    fn asl_a(&mut self) {
        self.c = self.a & (1 << 7) != 0;
        self.a <<= 1;
        self.set_z_n(self.a);
    }

    #[inline]
    fn lsr(&mut self, mut num: u8) {
        self.c = (num & 1) != 0;
        num >>= 1;
        self.set_z_n(num);
        self.temp = num as usize;
    }

    #[inline]
    fn lsr_a(&mut self) {
        self.c = (self.a & 1) != 0;
        self.a >>= 1;
        self.set_z_n(self.a);
    }

    #[inline]
    fn rol(&mut self, mut num: u8) {
        let c = self.c;
        self.c = num & (1 << 7) != 0;
        num <<= 1;
        num |= if c { 1 } else { 0 };
        self.set_z_n(num);
        self.temp = num as usize;
    }

    #[inline]
    fn rol_a(&mut self) {
        let c = self.c;
        self.c = self.a & (1 << 7) != 0;
        self.a <<= 1;
        self.a |= if c { 1 } else { 0 };
        self.set_z_n(self.a);
    }

    #[inline]
    fn ror(&mut self, mut num: u8) {
        let c = self.c;
        self.c = num & 1 != 0;
        num >>= 1;
        num |= if c { 1 } else { 0 } << 7;
        self.set_z_n(num);
        self.temp = num as usize;
    }

    #[inline]
    fn ror_a(&mut self) {
        let c = self.c;
        self.c = self.a & 1 != 0;
        self.a >>= 1;
        self.a |= if c { 1 } else { 0 } << 7;
        self.set_z_n(self.a);
    }

    #[inline]
    fn inc(&mut self, mut num: u8) {
        num = num.wrapping_add(1);
        self.set_z_n(num);
        self.temp = num as usize;
    }

    #[inline]
    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_z_n(self.x);
    }

    #[inline]
    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_z_n(self.y);
    }

    #[inline]
    fn dec(&mut self, mut num: u8) {
        num = num.wrapping_sub(1);
        self.set_z_n(num);
        self.temp = num as usize;
    }

    #[inline]
    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_z_n(self.x);
    }

    #[inline]
    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_z_n(self.y);
    }

    #[inline]
    fn and(&mut self, num: u8) {
        self.a &= num;
        self.set_z_n(self.a);
    }

    #[inline]
    fn eor(&mut self, num: u8) {
        self.a ^= num;
        self.set_z_n(self.a);
    }

    #[inline]
    fn ora(&mut self, num: u8) {
        self.a |= num;
        self.set_z_n(self.a);
    }

    #[inline]
    fn cmp(&mut self, num: u8) {
        self.compare(num, self.a);
    }

    #[inline]
    fn cpx(&mut self, num: u8) {
        self.compare(num, self.x);
    }

    #[inline]
    fn cpy(&mut self, num: u8) {
        self.compare(num, self.y);
    }

    #[inline]
    fn bit(&mut self, byte: u8) {
        self.z = (byte & self.a) == 0;
        self.v = (byte >> 6) & 1 != 0;
        self.n = (byte >> 7) & 1 != 0;
    }

    #[inline]
    fn lda(&mut self, num: u8) {
        self.a = num;
        self.set_z_n(self.a);
    }

    #[inline]
    fn ldx(&mut self, num: u8) {
        self.x = num;
        self.set_z_n(self.x);
    }

    #[inline]
    fn ldy(&mut self, num: u8) {
        self.y = num;
        self.set_z_n(self.y);
    }

    #[inline]
    fn sta(&mut self) {
        self.write(self.ab, self.a);
    }

    #[inline]
    fn stx(&mut self) {
        self.write(self.ab, self.x);
    }

    #[inline]
    fn sty(&mut self) {
        self.write(self.ab, self.y);
    }

    #[inline]
    fn tax(&mut self) {
        self.x = self.a;
        self.set_z_n(self.x);
    }

    #[inline]
    fn tay(&mut self) {
        self.y = self.a;
        self.set_z_n(self.y);
    }

    #[inline]
    fn tsx(&mut self) {
        self.x = self.sp as u8;
        self.set_z_n(self.x);
    }

    #[inline]
    fn txa(&mut self) {
        self.a = self.x;
        self.set_z_n(self.a);
    }

    #[inline]
    fn txs(&mut self) {
        self.sp = self.x as usize;
    }

    #[inline]
    fn tya(&mut self) {
        self.a = self.y;
        self.set_z_n(self.a);
    }

    #[inline]
    fn aax(&mut self) {
        let res = self.x & self.a;
        self.write(self.ab, res);
    }

    #[inline]
    fn dcp(&mut self) {
        self.dec(self.temp as u8);
        self.cmp(self.temp as u8);
    }

    #[inline]
    fn isc(&mut self) {
        self.inc(self.temp as u8);
        self.sbc(self.temp as u8);
    }

    #[inline]
    fn lax(&mut self, num: u8) {
        self.a = num;
        self.x = self.a;
        self.set_z_n(self.x);
    }

    #[inline]
    fn rla(&mut self) {
        self.rol(self.temp as u8);
        self.and(self.temp as u8);
    }

    #[inline]
    fn rra(&mut self) {
        self.ror(self.temp as u8);
        self.adc(self.temp as u8);
    }

    #[inline]
    fn slo(&mut self) {
        self.asl(self.temp as u8);
        self.ora(self.temp as u8);
    }

    #[inline]
    fn sre(&mut self) {
        self.lsr(self.temp as u8);
        self.eor(self.temp as u8);
    }

    #[inline]
    fn anc(&mut self, num: u8) {
        self.and(num);
        self.c = self.n;
    }

    #[inline]
    fn alr(&mut self, num: u8) {
        self.a &= num;
        self.c = self.a & 1 != 0;
        self.a >>= 1;
        self.set_z_n(self.a);
    }

    #[inline]
    fn axs(&mut self, num: u8) {
        self.x &= self.a;
        self.c = self.x >= num;
        self.x = self.x.wrapping_sub(num);
        self.set_z_n(self.x);
    }

    #[inline]
    fn xaa(&mut self, num: u8) {
        self.a = (self.a | 0xEE) & self.x & num;
        self.set_z_n(self.a);
    }

    #[inline]
    fn ahx(&mut self) {
        let result = self.a & self.x & ((self.ab >> 8) + 1) as u8;
        self.write(self.ab, result);
    }

    #[inline]
    fn shx(&mut self) {
        let result = ((self.ab >> 8) as u8).wrapping_add(1) & self.x;
        self.write((usize::from(result) << 8) | (self.ab & 0xFF), self.x);
    }

    #[inline]
    fn shy(&mut self) {
        let result = ((self.ab >> 8) as u8).wrapping_add(1) & self.y;
        self.write((usize::from(result) << 8) | (self.ab & 0xFF), self.y);
    }

    #[inline]
    fn las(&mut self, val: u8) {
        self.sp &= val as usize;
        self.a = self.sp as u8;
        self.x = self.sp as u8;
        self.set_z_n(self.a);
    }

    #[inline]
    fn tas(&mut self) {
        self.sp = (self.a & self.x) as usize;
        let result = self.sp & ((self.ab >> 8) + 1);
        self.write(self.ab, result as u8);
    }

    #[inline]
    fn arr(&mut self, num: u8) {
        self.and(num);
        self.ror_a();

        self.c = (self.a >> 6) & 1 == 1;
        self.v = self.c != ((self.a >> 5) & 1 == 1);
    }

    #[inline]
    fn compare(&mut self, num: u8, mut reg: u8) {
        self.c = reg >= num;
        reg = reg.wrapping_sub(num);
        self.z = reg == 0;
        self.n = reg & (1 << 7) != 0;
    }

    #[inline]
    fn take_branch(&mut self) {
        let diff = self.temp as i8 as isize;
        let pc_before = self.pc;
        if diff > 0 {
            //self.pc += diff as usize
            self.pc = (self.pc as u16).wrapping_add(diff as u16) as usize;
        } else {
            //self.pc -= diff.abs() as usize
            self.pc = (self.pc as u16).wrapping_sub(diff.abs() as u16) as usize;
        };
        let crosses = (pc_before & 0xFF00) != (self.pc & 0xFF00);
        self.temp = if crosses { 1 } else { 0 };
    }

    #[inline]
    fn push_status(&mut self, brk_php: bool) {
        let mut status: u8 = 1 << 5;
        status |= (if self.n { 1 } else { 0 }) << 7;
        status |= (if self.v { 1 } else { 0 }) << 6;
        status |= (if brk_php { 1 } else { 0 }) << 4;
        status |= (if self.d { 1 } else { 0 }) << 3;
        status |= (if self.i { 1 } else { 0 }) << 2;
        status |= (if self.z { 1 } else { 0 }) << 1;
        status |= if self.c { 1 } else { 0 };
        self.write(self.ab, status);
    }

    #[inline]
    fn pull_status(&mut self, status: u8) {
        self.n = status >> 7 != 0;
        self.v = (status >> 6) & 1 != 0;
        self.d = (status >> 3) & 1 != 0;
        self.i = (status >> 2) & 1 != 0;
        self.z = (status >> 1) & 1 != 0;
        self.c = status & 1 != 0;
    }

    #[inline]
    fn set_z_n(&mut self, num: u8) {
        self.z = num == 0;
        self.n = (num & 0b1000_0000) != 0;
    }
}
