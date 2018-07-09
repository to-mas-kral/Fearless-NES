use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use super::mapper::Mapper;
use super::memory::*;
use super::InterruptBus;

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

//A little macro for checking and setting zero and negative flags
macro_rules! set_z_n {
    ($var:expr, $self:ident) => {{
        let tmp = $var;
        $self.z(tmp);
        $self.n(tmp);
    }};
}

pub struct Cpu<M>
where
    M: Mapper,
{
    a: u8,     //Accumulator
    x: u8,     //X index
    y: u8,     //Y index
    pc: usize, //Program counter (16 bits)
    sp: usize, //Stack pointer (8 bits)

    n: bool, //Negative flag
    v: bool, //Overflow flag
    i: bool, //Interrupt inhibit
    z: bool, //Zero flag
    c: bool, //Carry flag
    d: bool, //BCD flag, this doesn't do anything on the NES CPU

    pub halt: bool,
    pub state: u16,

    ab: usize, //Address bus
    temp: usize,

    cached_irq: bool,
    take_interrupt: bool,
    interrupt_type: InterruptType,
    interrupt_bus: Rc<RefCell<InterruptBus>>,

    pending_reset: bool,

    pub mem: Rc<RefCell<Memory<M>>>, //reference to a memory map shared by other components
}

impl<M> Cpu<M>
where
    M: Mapper,
{
    pub fn new(mem: Rc<RefCell<Memory<M>>>, interrupt_bus: Rc<RefCell<InterruptBus>>) -> Cpu<M> {
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

            ab: 0,
            temp: 0,

            cached_irq: false,
            take_interrupt: false,
            interrupt_type: InterruptType::None,
            interrupt_bus,

            pending_reset: false,

            mem,
        }
    }

    pub fn load_to_memory(&mut self, file: &mut File) {
        self.mem.borrow_mut().clear();
        self.pc = 0x8000;
        let mut bytes = file.bytes();

        for _ in 0..16 {
            bytes.next();
        }

        for _ in 0..0x4000 {
            let b = bytes.next().unwrap().unwrap();
            self.mem.write(self.pc, b);
            self.mem.write(self.pc + 0x4000, b);
            self.pc += 1;
        }

        self.pc = 0xC000;
        self.ab = self.pc;
    }

    pub fn debug_info(&mut self) -> String {
        let mut status: u8 = 1 << 5;
        status |= (if self.n { 1 } else { 0 }) << 7;
        status |= (if self.v { 1 } else { 0 }) << 6;
        status |= (if self.d { 1 } else { 0 }) << 3;
        status |= (if self.i { 1 } else { 0 }) << 2;
        status |= (if self.z { 1 } else { 0 }) << 1;
        status |= if self.c { 1 } else { 0 };

        format!(
            "{:X} {:X} A:{:X} X:{:X} Y:{:X} P:{:X} SP:{:X}",
            self.pc,
            self.mem.read(self.pc),
            self.a,
            self.x,
            self.y,
            status,
            self.sp,
        )
    }

    pub fn print_debug_info(&mut self) {
        println!("{}", self.debug_info());
    }

    pub fn gen_reset(&mut self) {
        self.pc = ((u16::from(self.mem.read_direct(0xFFFD)) << 8)
            | u16::from(self.mem.read_direct(0xFFFC))) as usize;
        self.ab = self.pc;
    }

    //TODO: what is pending_IRQ ?
    #[inline]
    fn check_interrupts(&mut self) {
        if !self.i && self.cached_irq {
            self.take_interrupt = true;
            self.interrupt_type = InterruptType::Irq;
        }

        if self.interrupt_bus.borrow().nmi_signal {
            self.take_interrupt = true;
            self.interrupt_type = InterruptType::Nmi;
            self.interrupt_bus.borrow_mut().nmi_signal = false;
        }

        if self.interrupt_bus.borrow().reset_signal {
            //FIXME: make this cycle-accurate
            self.gen_reset();
        }
    }

    #[inline]
    fn interrupt_address(&mut self) -> usize {
        //TODO: verify this procedure

        if self.interrupt_bus.borrow().nmi_signal {
            return 0xFFFA;
        }

        match self.interrupt_type {
            InterruptType::Irq => 0xFFFE,
            InterruptType::Nmi => 0xFFFA,
            InterruptType::Reset => 0xFFFC,
            InterruptType::None => 0,
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
        set_z_n!(self.a, self);
    }

    #[inline]
    fn sbc(&mut self, num: u8) {
        self.adc(!num);
    }

    #[inline]
    fn asl(&mut self, mut num: u8) {
        self.c = num & (1 << 7) != 0;
        num <<= 1;
        set_z_n!(num, self);
        self.temp = num as usize;
    }

    #[inline]
    fn asl_a(&mut self) {
        self.c = self.a & (1 << 7) != 0;
        self.a <<= 1;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn lsr(&mut self, mut num: u8) {
        self.c = (num & 1) != 0;
        num >>= 1;
        set_z_n!(num, self);
        self.temp = num as usize;
    }

    #[inline]
    fn lsr_a(&mut self) {
        self.c = (self.a & 1) != 0;
        self.a >>= 1;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn rol(&mut self, mut num: u8) {
        let c = self.c;
        self.c = num & (1 << 7) != 0;
        num <<= 1;
        num |= if c { 1 } else { 0 };
        set_z_n!(num, self);
        self.temp = num as usize;
    }

    #[inline]
    fn rol_a(&mut self) {
        let c = self.c;
        self.c = self.a & (1 << 7) != 0;
        self.a <<= 1;
        self.a |= if c { 1 } else { 0 };
        set_z_n!(self.a, self);
    }

    #[inline]
    fn ror(&mut self, mut num: u8) {
        let c = self.c;
        self.c = num & 1 != 0;
        num >>= 1;
        num |= if c { 1 } else { 0 } << 7;
        set_z_n!(num, self);
        self.temp = num as usize;
    }

    #[inline]
    fn ror_a(&mut self) {
        let c = self.c;
        self.c = self.a & 1 != 0;
        self.a >>= 1;
        self.a |= if c { 1 } else { 0 } << 7;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn inc(&mut self, mut num: u8) {
        num = num.wrapping_add(1);
        set_z_n!(num, self);
        self.temp = num as usize;
    }

    #[inline]
    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        set_z_n!(self.x, self);
    }

    #[inline]
    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        set_z_n!(self.y, self);
    }

    #[inline]
    fn dec(&mut self, mut num: u8) {
        num = num.wrapping_sub(1);
        set_z_n!(num, self);
        self.temp = num as usize;
    }

    #[inline]
    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        set_z_n!(self.x, self);
    }

    #[inline]
    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        set_z_n!(self.y, self);
    }

    #[inline]
    fn and(&mut self, num: u8) {
        self.a &= num;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn eor(&mut self, num: u8) {
        self.a ^= num;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn ora(&mut self, num: u8) {
        self.a |= num;
        set_z_n!(self.a, self);
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
        set_z_n!(self.a, self);
    }

    #[inline]
    fn ldx(&mut self, num: u8) {
        self.x = num;
        set_z_n!(self.x, self);
    }

    #[inline]
    fn ldy(&mut self, num: u8) {
        self.y = num;
        set_z_n!(self.y, self);
    }

    #[inline]
    fn sta(&mut self) {
        self.mem.write(self.ab, self.a);
    }

    #[inline]
    fn stx(&mut self) {
        self.mem.write(self.ab, self.x);
    }

    #[inline]
    fn sty(&mut self) {
        self.mem.write(self.ab, self.y);
    }

    #[inline]
    fn tax(&mut self) {
        self.x = self.a;
        set_z_n!(self.x, self);
    }

    #[inline]
    fn tay(&mut self) {
        self.y = self.a;
        set_z_n!(self.y, self);
    }

    #[inline]
    fn tsx(&mut self) {
        self.x = self.sp as u8;
        set_z_n!(self.x, self);
    }

    #[inline]
    fn txa(&mut self) {
        self.a = self.x;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn txs(&mut self) {
        self.sp = self.x as usize;
    }

    #[inline]
    fn tya(&mut self) {
        self.a = self.y;
        set_z_n!(self.a, self);
    }

    #[inline]
    fn aax(&mut self) {
        let res = self.x & self.a;
        self.mem.write(self.ab, res);
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
        set_z_n!(self.x, self);
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
    fn compare(&mut self, num: u8, mut reg: u8) {
        self.c = reg >= num;
        reg = reg.wrapping_sub(num);
        self.z = reg == 0;
        self.n = reg & (1 << 7) != 0;
    }

    #[inline]
    fn take_branch(&mut self) {
        let diff = self.temp as i8 as isize;
        if diff > 0 {
            self.pc += diff as usize
        } else {
            self.pc -= diff.abs() as usize
        };
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
        set_z_n!(self.a, self);
    }

    #[inline]
    fn axs(&mut self, num: u8) {
        self.x &= self.a;
        self.c = self.x >= num;
        self.x = self.x.wrapping_sub(num);
        set_z_n!(self.x, self);
    }

    //TODO: complete these
    #[inline]
    fn xaa(&mut self) {}
    #[inline]
    fn ahx(&mut self) {}
    #[inline]
    fn shx(&mut self) {
        let result = ((self.ab >> 8) as u8).wrapping_add(1) & self.x;
        self.mem
            .write((usize::from(result) << 8) | (self.ab & 0xFF), self.x);
    }
    #[inline]
    fn shy(&mut self) {
        let result = ((self.ab >> 8) as u8).wrapping_add(1) & self.y;
        self.mem
            .write((usize::from(result) << 8) | (self.ab & 0xFF), self.y);
    }
    #[inline]
    fn las(&mut self) {}
    #[inline]
    fn tas(&mut self) {}
    #[inline]
    fn arr(&mut self, num: u8) {
        self.and(num);
        self.ror_a();

        let bit5 = (self.a >> 5) & 1 == 1;
        let bit6 = (self.a >> 6) & 1 == 1;

        if bit5 {
            if bit6 {
                self.c = true;
                self.v = false;
            } else {
                self.c = false;
                self.v = true;
            }
        } else if bit6 {
            self.c = true;
            self.v = true;
        } else {
            self.c = false;
            self.v = false;
        }
    }

    #[inline]
    fn push(&mut self, adr: usize, data: u8) {
        self.mem.write_zp(adr, data);
    }

    #[inline]
    fn pop(&mut self, adr: usize) -> u8 {
        self.mem.read_zp(adr)
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
        self.push(self.ab, status);
    }

    #[inline]
    fn pull_status(&mut self) {
        let status = self.pop(self.ab);
        self.n = status >> 7 != 0;
        self.v = (status >> 6) & 1 != 0;
        self.d = (status >> 3) & 1 != 0;
        self.i = (status >> 2) & 1 != 0;
        self.z = (status >> 1) & 1 != 0;
        self.c = status & 1 != 0;
    }

    #[inline]
    fn z(&mut self, b: u8) {
        self.z = b == 0
    }

    #[inline]
    fn n(&mut self, b: u8) {
        self.n = (b & 0b1000_0000) > 0
    }
}
