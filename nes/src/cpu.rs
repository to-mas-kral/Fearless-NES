use super::Nes;

mod instructions;
mod state_machine;

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
    pub fn cpu_debug_info(&mut self) -> String {
        format!(
            "A: 0x{:X}, X: 0x{:X}, Y: 0x{:X}, pc: 0x{:X}, sp: 0x{:X}, ab: 0x{:X}",
            self.cpu.a, self.cpu.x, self.cpu.y, self.cpu.pc, self.cpu.sp, self.cpu.ab
        )
    }
    pub fn cpu_gen_reset(&mut self) {
        self.cpu.state = 0;
        self.cpu.take_interrupt = true;
        self.cpu.reset_signal = true;
        self.cpu.interrupt_type = InterruptType::Reset;
        self.cpu_write(0x4015, 0);
        self.cpu.reset_signal = false;
    }

    #[inline]
    fn cpu_check_interrupts(&mut self) {
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
    fn cpu_interrupt_address(&mut self) -> usize {
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
    pub fn cpu_read(&mut self, index: usize) {
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
    pub fn cpu_write(&mut self, index: usize, val: u8) {
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
    pub fn cpu_peek(&mut self, index: usize) -> u8 {
        match index {
            0..=0x1FFF => self.cpu.ram[index & 0x7FF],
            0x4020..=0xFFFF => (self.mapper.cpu_peek)(self, index),
            _ => panic!("Error: memory access into unmapped address: 0x{:X}", index),
        }
    }

    //forums.nesdev.com/viewtopic.php?f=3&t=14120
    #[inline]
    pub fn cpu_dma(&mut self) {
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

    #[inline]
    fn cpu_compare(&mut self, num: u8, mut reg: u8) {
        self.cpu.c = reg >= num;
        reg = reg.wrapping_sub(num);
        self.cpu.z = reg == 0;
        self.cpu.n = reg & (1 << 7) != 0;
    }

    #[inline]
    fn cpu_take_branch(&mut self) {
        let diff = self.cpu.temp as i8 as isize;
        let pc_before = self.cpu.pc;
        if diff > 0 {
            //self.cpu.pc += diff as usize
            self.cpu.pc = (self.cpu.pc as u16).wrapping_add(diff as u16) as usize;
        } else {
            //self.cpu.pc -= diff.abs() as usize
            self.cpu.pc = (self.cpu.pc as u16).wrapping_sub(diff.abs() as u16) as usize;
        };
        let crosses = (pc_before & 0xFF00) != (self.cpu.pc & 0xFF00);
        self.cpu.temp = if crosses { 1 } else { 0 };
    }

    #[inline]
    fn cpu_push_status(&mut self, brk_php: bool) {
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
    fn cpu_pull_status(&mut self, status: u8) {
        self.cpu.n = status >> 7 != 0;
        self.cpu.v = (status >> 6) & 1 != 0;
        self.cpu.d = (status >> 3) & 1 != 0;
        self.cpu.i = (status >> 2) & 1 != 0;
        self.cpu.z = (status >> 1) & 1 != 0;
        self.cpu.c = status & 1 != 0;
    }

    #[inline]
    fn cpu_set_z_n(&mut self, num: u8) {
        self.cpu.z = num == 0;
        self.cpu.n = (num & 0b1000_0000) != 0;
    }
}
