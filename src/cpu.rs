
#![allow(non_snake_case)]

use std::fs::File;
use std::io::Read;

//use std::{thread, time};

pub struct Cpu {
    A: u8, //Accumulator
    X: u8, //X index
    Y: u8, //Y index
    pc: usize, //Program counter (16 bits)
    sp: usize, //Stack pointer (8 bits)

    N: bool, //Negative flag
    V: bool, //Overflow flag
    I: bool, //Interrupt inhibit
    Z: bool, //Zero flag
    C: bool, //Carry flag
    B: bool, //Break flag
    D: bool, //BCD flag

    mem: [u8; 0x10000],
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            A: 0,
            X: 0,
            Y: 0,
            pc: 0,
            sp: 0xFD,

            N: false,
            V: false,
            I: true,
            Z: false,
            C: false,
            B: false,
            D: false,

            mem: [0; 0x10000],
        };

        //cpu.load_to_memory(&mut File::open("SMB.nes").unwrap());
        cpu.load_to_memory(&mut File::open("nestest.nes").unwrap());
        cpu.mem[0x180] = 0x33;
        cpu.mem[0x17F] = 0x69;
        cpu
    }

    pub fn load_to_memory(&mut self, file: &mut File) {
        self.mem = [0; 0x10000];
        self.pc = 0x8000;
        let mut bytes = file.bytes();

        for _ in 0..16 {
            bytes.next();
        }

        for _ in 0..0x4000 {
            let b = bytes.next().unwrap().unwrap();
            self.mem[self.pc - 0x7FF0];
            self.mem[self.pc] = b;
            self.mem[self.pc + 0x4000] = b;
            self.pc += 1;
        }

        self.pc = 0xC000;
    }

    //pub fn load_to_memory(&mut self, file: &mut File) {
    //    //TODO: Create separate module for mappers and loaders
    //    self.mem = [0; 0x10000];
    //    self.pc = 0x8000;
//
    //    let mut bytes = file.bytes();
    //    for _ in 0..16 {
    //        bytes.next();
    //    }
//
    //    for _ in 0..0x8000 {
    //        let b = bytes.next().unwrap().unwrap();
    //        self.mem[self.pc] = b;
    //        self.pc += 1;
    //    }
//
    //    self.pc = (((self.mem[0xFFFD] as u16) << 8) | self.mem[0xFFFC] as u16) as usize;
    //}

    pub fn step(&mut self) {
        let opcode = self.mem[self.pc];
        let pc = self.pc;
        let (adr, mut cycles): (usize, u8) = match opcode {
            0x69 | 0x29 | 0xC9 | 0xE0 | 0xC0 | 0x49 | 0xA9 | 0xA2 | 0xA0 | 0x09 | 0xE9 |
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 | 0xEB => self.imm(),
            0x65 | 0x25 | 0x06 | 0x24 | 0xC5 | 0xE4 | 0xC4 | 0xC6 | 0x45 | 0xE6 | 0xA5 |
            0xA6 | 0xA4 | 0x46 | 0x05 | 0x26 | 0x66 | 0xE5 | 0x85 | 0x86 | 0x84 | 0x04 |
            0x44 | 0x64 | 0xA7 | 0x87 | 0xC7 | 0xE7 | 0x07 | 0x27 | 0x47 | 0x67 => self.zpg(),
            0x75 | 0x35 | 0x16 | 0xD5 | 0xD6 | 0x55 | 0xF6 | 0xB5 | 0xB4 | 0x56 | 0x15 |
            0x36 | 0x76 | 0xF5 | 0x95 | 0x94 | 0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 |
            0xD7 | 0xF7 | 0x17 | 0x37 | 0x57 | 0x77 => self.zpgx(),
            0xB6 | 0x96 | 0xB7 | 0x97 => self.zpgy(),
            0x6D | 0x2D | 0x0E | 0x2C | 0xCD | 0xEC | 0xCC | 0xCE | 0x4D | 0xEE | 0x4C |
            0x20 | 0xAD | 0xAE | 0xAC | 0x4E | 0x0D | 0x2E | 0x6E | 0xED | 0x8D | 0x8E |
            0x8C | 0x0C | 0xAF | 0x8F | 0xCF | 0xEF | 0x0F | 0x2F | 0x4F | 0x6F => self.abs(),
            0x7D | 0x3D | 0x1E | 0xDD | 0xDE | 0x5D | 0xFE | 0xBD | 0xBC | 0x5E | 0x1D |
            0x3E | 0x7E | 0xFD | 0x9D | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC | 0xDF | 
            0xFF | 0x1F | 0x3F | 0x5F | 0x7F => self.absx(),
            0x79 | 0x39 | 0xD9 | 0x59 | 0xB9 | 0xBE | 0x19 | 0xF9 | 0x99 | 0xBF | 0xDB |
            0xFB | 0x1B | 0x3B | 0x5B | 0x7B => self.absy(),
            0x90 | 0xB0 | 0xF0 | 0x30 | 0xD0 | 0x10 | 0x50 | 0x70 => self.rel(),
            0x6C => self.ind(),
            0x61 | 0x21 | 0xC1 | 0x41 | 0xA1 | 0x01 | 0xE1 | 0x81 | 0xA3 | 0x83 | 0xC3 |
            0xE3 | 0x03 | 0x23 | 0x43 | 0x63 => self.indx(),
            0x71 | 0x31 | 0xD1 | 0x51 | 0xB1 | 0x11 | 0xF1 | 0x91 | 0xB3 | 0xD3 | 0xF3 |
            0x13 | 0x33 | 0x53 | 0x73 => self.indy(),
            0x0A | 0x00 | 0x18 | 0xD8 | 0x58 | 0xB8 | 0xCA | 0x88 | 0xE8 | 0xC8 | 0x4A |
            0xEA | 0x48 | 0x08 | 0x68 | 0x28 | 0x2A | 0x6A | 0x40 | 0x60 | 0x38 | 0xF8 |
            0x78 | 0xAA | 0xA8 | 0xBA | 0x8A | 0x9A | 0x98 | 0x1A | 0x3A | 0x5A | 0x7A |
            0xDA | 0xFA => (70000, 0),
            _ => {
                println!("ERROR CODE: {:X}", self.mem[0x02]);
                println!("ERROR CODE: {:X}", self.mem[0x03]);
                println!("ADDRESS: {:X}", self.pc);
                panic!("Illegal opcode: {:X}", opcode);
            },
        };

    //TODO: clean this
        let mut status: u8 = 1 << 5;
        status |= (if self.N {1} else {0}) << 7;
        status |= (if self.V {1} else {0}) << 6;
        status |= (if self.B {1} else {0}) << 4;
        status |= (if self.D {1} else {0}) << 3;
        status |= (if self.I {1} else {0}) << 2;
        status |= (if self.Z {1} else {0}) << 1;
        status |= if self.C {1} else {0};

        print!("{:X} {:X} {:X} A:{:X} X:{:X} Y:{:X} P:{:X} S:{:X} {} ", pc, opcode, adr, self.A, self.X, self.Y, status, self.sp, cycles);
        print!("{}", cycles);

        cycles += match opcode {
            0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.ADC(adr),
            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 | 0xEB => self.SBC(adr),
            0x0A | 0x06 | 0x16 | 0x0E | 0x1E => self.ASL(adr),
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E => self.LSR(adr),
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E => self.ROL(adr),
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E => self.ROR(adr),
            0xE6 | 0xF6 | 0xEE | 0xFE => self.INC(adr),
            0xE8 => self.INX(),
            0xC8 => self.INY(),
            0xC6 | 0xD6 | 0xCE | 0xDE => self.DEC(adr),
            0xCA => self.DEX(),
            0x88 => self.DEY(),
            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.AND(adr),
            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.EOR(adr),
            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.ORA(adr),
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => self.CMP(adr),
            0xE0 | 0xE4 | 0xEC => self.CPX(adr),
            0xC0 | 0xC4 | 0xCC => self.CPY(adr),
            0x24 | 0x2C => self.BIT(adr),
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.LDA(adr),
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.LDX(adr),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.LDY(adr),
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.STA(adr),
            0x86 | 0x96 | 0x8E => self.STX(adr),
            0x84 | 0x94 | 0x8C => self.STY(adr),
            0x4C | 0x6C => self.JMP(adr),
            0x20 => self.JSR(adr),
            0x40 => self.RTI(),
            0x60 => self.RTS(),
            0x90 => self.BCC(adr),
            0xB0 => self.BCS(adr),
            0xF0 => self.BEQ(adr),
            0x30 => self.BMI(adr),
            0xD0 => self.BNE(adr),
            0x10 => self.BPL(adr),
            0x50 => self.BVC(adr),
            0x70 => self.BVS(adr),
            0x78 => self.SEI(),
            0xF8 => self.SED(),
            0x38 => self.SEC(),
            0x18 => self.CLC(),
            0xD8 => self.CLD(),
            0x58 => self.CLI(),
            0xB8 => self.CLV(),
            0xAA => self.TAX(),
            0xA8 => self.TAY(),
            0xBA => self.TSX(),
            0x8A => self.TXA(),
            0x9A => self.TXS(),
            0x98 => self.TYA(),
            0xEA | 0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89 |
            0xC2 | 0xD4 | 0xE2 | 0xF4 | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => 2,
            0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
            0x00 => self.BRK(),
            0x48 => self.PHA(),
            0x08 => self.PHP(),
            0x68 => self.PLA(),
            0x28 => self.PLP(),
            //AAC
            0x87 | 0x97 | 0x83 | 0x8F => self.AAX(adr),
            //ARR
            //ASR
            //ATX
            //AXA
            //AXS
            0xC7 | 0xD7 | 0xCF | 0xDF | 0xDB | 0xC3 | 0xD3 => self.DCP(adr),
            0xE7 | 0xF7 | 0xEF | 0xFF | 0xFB | 0xE3 | 0xF3 => self.ISC(adr),
            //KIL
            //LAR
            0xA7 | 0xB7 | 0xAF | 0xBF | 0xA3 | 0xB3 => self.LAX(adr),
            0x27 | 0x37 | 0x2F | 0x3F | 0x3B | 0x23 | 0x33 => self.RLA(adr),
            0x67 | 0x77 | 0x6F | 0x7F | 0x7B | 0x63 | 0x73 => self.PRA(adr),
            0x07 | 0x17 | 0x0F | 0x1F | 0x1B | 0x03 | 0x13 => self.SLO(adr),
            0x47 | 0x57 | 0x4F | 0x5F | 0x5B | 0x43 | 0x53 => self.SRE(adr),
            //SXA
            //SYA
            //XAA
            //XAS
            _ => panic!("Opcode not implemented"),
        };

        self.pc += 1;
    }

    fn ADC(&mut self, adr: usize) -> u8 {
        let mut num = self.mem[adr];
        let carry = (num as u16 + self.A as u16 + (if self.C {1} else {0})) & (1 << 8) != 0;
        let (num, v1): (i8, bool) = (num as i8).overflowing_add(if self.C {1} else {0});
        let (num, v2): (i8, bool) = (num as i8).overflowing_add(self.A as i8);
        self.A = num as u8;
        self.V = v1 || v2;
        self.C = carry;
        let a = self.A;
        self.Z(a);
        self.N(a);
        1
    }

    fn SBC(&mut self, adr: usize) -> u8 {
        self.mem[adr] = !self.mem[adr];
        self.ADC(adr);
        self.mem[adr] = !self.mem[adr];
        1
    }

    fn ASL(&mut self, adr: usize) -> u8 {
        let tmp = {
            let target = if adr == 70000 {&mut self.A} else {&mut self.mem[adr]};
            let tmp = (*target as u16) << 1;
            *target = tmp as u8;
            tmp
        };

        self.C = tmp & (1 << 8) != 0;
        self.Z(tmp as u8);
        self.N(tmp as u8);

        if adr == 70000 {2} else {3}
    }

    fn LSR(&mut self, adr: usize) -> u8 {
        let tmp = {
            let target = if adr == 70000 {&mut self.A} else {&mut self.mem[adr]};
            self.C = (*target & 1) != 0;
            *target >>= 1;
            let tmp = *target;
            tmp
        };

        self.Z(tmp);
        self.N(tmp);

        if adr == 70000 {2} else {3}
    }

    fn ROL(&mut self, adr: usize) -> u8 {
        let tmp = {
            let target = if adr == 70000 {&mut self.A} else {&mut self.mem[adr]};
            let C = *target & (1 << 7) != 0;
            *target <<= 1;
            *target |= if self.C {1} else {0};
            self.C = C;
            let tmp = *target;
            tmp
        };

        self.Z(tmp);
        self.N(tmp);

        if adr == 70000 {2} else {3}
    }

    fn ROR(&mut self, adr: usize) -> u8 {
        let tmp = {
            let target = if adr == 70000 {&mut self.A} else {&mut self.mem[adr]};
            let C = *target & 1 != 0;
            *target >>= 1;
            *target |= (if self.C {1} else {0}) << 7;
            self.C = C;
            let tmp = *target;
            tmp
        };

        self.Z(tmp);
        self.N(tmp);

        if adr == 70000 {2} else {3}
    }

    fn INC(&mut self, adr: usize) -> u8 {
         self.mem[adr] = self.mem[adr].wrapping_add(1);
         let n = self.mem[adr];
         self.Z(n);
         self.N(n);

         3
    }

    fn INX(&mut self) -> u8 {
        self.X = self.X.wrapping_add(1);
        let x = self.X;
        self.Z(x);
        self.N(x);

        2
    }

    fn INY(&mut self) -> u8 {
        self.Y = self.Y.wrapping_add(1);
        let y = self.Y;
        self.Z(y);
        self.N(y);

        2
    }

    fn DEC(&mut self, adr: usize) -> u8 {
         self.mem[adr] = self.mem[adr].wrapping_sub(1);
         let n = self.mem[adr];
         self.Z(n);
         self.N(n);

         3
    }

    fn DEX(&mut self) -> u8 {
        self.X = self.X.wrapping_sub(1);
        let x = self.X;
        self.Z(x);
        self.N(x);

        2
    }

    fn DEY(&mut self) -> u8 {
        self.Y = self.Y.wrapping_sub(1);
        let y = self.Y;
        self.Z(y);
        self.N(y);
        
        2
    }

    fn AND(&mut self, adr: usize) -> u8 {
        self.A &= self.mem[adr];

        let a = self.A;
        self.Z(a);
        self.N(a);

        1
    }

    fn EOR(&mut self, adr: usize) -> u8 {
         self.A ^= self.mem[adr];
         let a = self.A;
         self.Z(a);
         self.N(a);

         1
    }

    fn ORA(&mut self, adr: usize) -> u8 {
         self.A |= self.mem[adr];
         let a = self.A;
         self.Z(a);
         self.N(a);

         1
    }

    fn CMP(&mut self, adr: usize) -> u8 {
        let a = self.A;
        self.compare(adr, a)
    }

    fn CPX(&mut self, adr: usize) -> u8 {
        let x = self.X;
        self.compare(adr, x)
    }

    fn CPY(&mut self, adr: usize) -> u8 {
        let y = self.Y;
        self.compare(adr, y)
    }

    fn BIT(&mut self, adr: usize) -> u8 {
         let mut byte = self.mem[adr];
         self.Z = (byte & self.A) == 0;
         self.V = (byte >> 6) & 1 != 0;
         self.N = (byte >> 7) & 1 != 0;

         1
    }

    fn LDA(&mut self, adr: usize) -> u8 {
        self.A = self.mem[adr];
        let a = self.A;
        self.Z(a);
        self.N(a);

        1
    }

    fn LDX(&mut self, adr: usize) -> u8 {
        self.X = self.mem[adr];
        
        let x = self.X;
        self.Z(x);
        self.N(x);

        1
    }

    fn LDY(&mut self, adr: usize) -> u8 {
        self.Y = self.mem[adr];

        let y = self.Y;
        self.Z(y);
        self.N(y);

        1
    }

    fn STA(&mut self, adr: usize) -> u8 {
        self.mem[adr] = self.A;
        1
    }

    fn STX(&mut self, adr: usize) -> u8 {
        self.mem[adr] = self.X;
        1
    }

    fn STY(&mut self, adr: usize) -> u8 {
        self.mem[adr] = self.Y;
        1
    }

    fn JMP(&mut self, adr: usize) -> u8 {
        self.pc = adr - 1;
        0
    }

    fn JSR(&mut self, adr: usize) -> u8 {
        let pc = self.pc;
        self.push((pc >> 8) as u8);
        self.push(pc as u8);
        self.pc = adr - 1;

        6
    }

    fn RTI(&mut self) -> u8 {
         self.pull_status();
         self.pc = ((self.pop() as u16) | ((self.pop() as u16) << 8)) as usize - 1;
         6
    }

    fn RTS(&mut self) -> u8 {
        self.pc = ((self.pop() as u16) | (self.pop() as u16) << 8) as usize;
        6
    }

    fn BCC(&mut self, adr: usize) -> u8 {
         let c = !self.C;
         self.branch(adr, c)
    }

    fn BCS(&mut self, adr: usize) -> u8 {
        let c = self.C;
        self.branch(adr, c)
    }

    fn BEQ(&mut self, adr: usize) -> u8 {
        let z = self.Z;
        self.branch(adr, z)
    }

    fn BMI(&mut self, adr: usize) -> u8 {
         let n = self.N;
         self.branch(adr, n)
    }

    fn BNE(&mut self, adr: usize) -> u8 {
        let z = !self.Z;
        self.branch(adr, z)
    }

    fn BPL(&mut self, adr: usize) -> u8 {
        let n = !self.N;
        self.branch(adr, n)
    }

    fn BVC(&mut self, adr: usize) -> u8 {
         let v = !self.V;
         self.branch(adr, v)
    }

    fn BVS(&mut self, adr: usize) -> u8 {
         let v = self.V;
         self.branch(adr, v)
    }

    fn SEI(&mut self) -> u8 {
        self.I = true;
        2
    }

    fn SED(&mut self) -> u8 {
        self.D = true;
        2
    }

    fn SEC(&mut self) -> u8 {
        self.C = true;
        2
    }

    fn CLC(&mut self) -> u8 {
        self.C = false;
        2
    }

    fn CLD(&mut self) -> u8 {
        self.D = false;
        2
    }

    fn CLI(&mut self) -> u8 {
        self.I = false;
        2
    }

    fn CLV(&mut self) -> u8 {
        self.V = false;
        2
    }

    fn TAX(&mut self) -> u8 {
        self.X = self.A;

        let x  = self.X;
        self.Z(x);
        self.N(x);

        2
    }

    fn TAY(&mut self) -> u8 {
        self.Y = self.A;
        let y = self.Y;
        self.Z(y);
        self.N(y);

        2
    }

    fn TSX(&mut self) -> u8 {
         self.X = self.sp as u8;
         let x = self.X;
         self.Z(x);
         self.N(x);

         2
    }

    fn TXA(&mut self) -> u8 {
        self.A = self.X;

        let a = self.A;
        self.Z(a);
        self.N(a);

        2
    }

    fn TXS(&mut self) -> u8 {
        self.sp = self.X as usize;
        2
    }

    fn TYA(&mut self) -> u8 {
        self.A = self.Y;

        let a = self.A;
        self.Z(a);
        self.N(a);

        2
    }

    fn BRK(&mut self) -> u8 {
        self.pc += 1;
        let tmp = self.pc + 1;
        self.push((tmp >> 8) as u8);
        self.push(tmp as u8);
        self.push_status(true);
        self.pc = (((self.mem[0xFFFF] as u16) << 8) | self.mem[0xFFFE] as u16) as usize;
        self.B = true;
        7
    }

    fn PHA(&mut self) -> u8 {
         let a = self.A;
         self.push(a);

         3
    }

    fn PHP(&mut self) -> u8 {
        self.push_status(true);
        3
    }

    fn PLA(&mut self) -> u8 {
         self.A = self.pop();
         let a = self.A;
         self.Z(a);
         self.N(a);

         4
    }

    fn PLP(&mut self) -> u8 {
        self.pull_status();
        4
    }

    //Undocumented instructions
    //TODO: complete other undocumented instructions

    fn AAX(&mut self, adr: usize) -> u8 {
        let res = self.X & self.A;
        self.mem[adr] = res;
        1
    }

    fn DCP(&mut self, adr: usize) -> u8 {
        self.DEC(adr);
        self.CMP(adr);
        3
    }

    fn ISC(&mut self, adr: usize) -> u8 {
        self.INC(adr);
        self.SBC(adr);
        3
    }

    fn LAX(&mut self, adr: usize) -> u8 {
        self.A = self.mem[adr];
        self.X = self.A;
        let x = self.X;
        self.N(x);
        self.Z(x);
        1
    }

    fn RLA(&mut self, adr: usize) -> u8 {
        self.ROL(adr);
        self.AND(adr);
        3
    }

    fn PRA(&mut self, adr: usize) -> u8 {
        self.ROR(adr);
        self.ADC(adr);
        3
    }

    fn SLO(&mut self, adr: usize) -> u8 {
        self.ASL(adr);
        self.ORA(adr);
        3
    }

    fn SRE(&mut self, adr: usize) -> u8 {
        self.LSR(adr);
        self.EOR(adr);
        3
    }

    //Helper functions

    fn compare(&mut self, adr: usize, mut b: u8) -> u8 {
        let num = self.mem[adr];
        self.C = b >= num;
        b = b.wrapping_sub(num);
        self.Z = b == 0;
        self.N = b & (1 << 7) != 0;
        1
    }

    fn branch(&mut self, adr: usize, cond: bool) -> u8 {
        let mut cycles = 1;
        if cond {
            let diff = adr as i8 as isize;
            let before = self.pc;
            if diff > 0 { self.pc += diff as usize } else { self.pc -= diff.abs() as usize };
            cycles += 1 + Cpu::crosses(before, self.pc);
        }

        cycles
    }

    fn push(&mut self, b: u8) {
        self.mem[self.sp] = b;
        self.sp -= 1;
    }

    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.mem[self.sp]
    }

    fn push_status(&mut self, brk_php: bool) {
        let mut status: u8 = 1 << 5;
        status |= (if self.N {1} else {0}) << 7;
        status |= (if self.V {1} else {0}) << 6;
        status |= (if brk_php {1} else {0}) << 4;
        status |= (if self.D {1} else {0}) << 3;
        status |= (if self.I {1} else {0}) << 2;
        status |= (if self.Z {1} else {0}) << 1;
        status |= if self.C {1} else {0};
        self.push(status);
    }

    fn pull_status(&mut self) {
        let status = self.pop();
        self.N = status >> 7 != 0;
        self.V = (status >> 6) & 1 != 0;
        self.I = (status >> 2) & 1 != 0;
        self.D = (status >> 3) & 1 != 0;
        self.Z = (status >> 1) & 1 != 0;
        self.C = status & 1 != 0;
    }

    //Addressing modes

    fn imm(&mut self) -> (usize, u8) {
        self.pc += 1;
        (self.pc, 1)
    }

    fn zpg(&mut self) -> (usize, u8) {
        self.pc += 1;
        (self.mem[self.pc] as usize, 2)
    }

    fn zpgx(&mut self) -> (usize, u8) {
        self.pc += 1;
        (self.mem[self.pc].wrapping_add(self.X) as usize, 3)
    }

    fn zpgy(&mut self) -> (usize, u8) {
        self.pc += 1;
        ((self.mem[self.pc].wrapping_add(self.Y)) as usize, 3)
    }

    fn rel(&mut self) -> (usize, u8) {
        self.pc += 1;
        let mut cycles: u8 = 1;
        (self.mem[self.pc] as usize, cycles)
    }

    fn abs(&mut self) -> (usize, u8) {
        self.pc += 2;
        (((self.mem[self.pc] as usize) << 8) | (self.mem[self.pc - 1] as usize), 3)
    }

    fn absx_y(&mut self, offset: u8) -> (usize, u8) {
        let mut cycles: u8 = 3;
        let mut base = self.abs().0;
        let before = base;
        base = (base as u16).wrapping_add(offset as u16) as usize;
        cycles += Cpu::crosses(before, base);

        (base as usize, cycles)
    }

    fn absx(&mut self) -> (usize, u8) {
        let x = self.X;
        self.absx_y(x)
    }

    fn absy(&mut self) -> (usize, u8) {
        let y = self.Y;
        self.absx_y(y)
    }

    fn ind(&mut self) -> (usize, u8) {
        let base = self.abs().0;
        let addr = if (base as u8) == 0xFF {
            self.mem[base] as usize | ((self.mem[base & 0xFF00] as usize) << 8)
        } else {
            self.mem[base] as usize | ((self.mem[(base.wrapping_add(1))] as usize) << 8)
        };

        (addr, 5)
    }

    fn indx(&mut self) -> (usize, u8) {
        self.pc += 1;
        let base = self.mem[self.pc].wrapping_add(self.X);
        let addr = self.mem[base as usize] as usize | ((self.mem[(base.wrapping_add(1)) as usize] as usize) << 8);
        (addr, 5)
    }

    fn indy(&mut self) -> (usize, u8) {
        self.pc += 1;
        let mut cycles = 4;
        let mut base = self.mem[self.pc] as usize;
        base = self.mem[base] as usize | ((self.mem[((base as u8).wrapping_add(1)) as usize] as usize) << 8);
        let before = base;
        base = (base as u16).wrapping_add(self.Y as u16) as usize;
        cycles += Cpu::crosses(before, base);
        (base, cycles)
    }

    //Flag checking

    fn Z(&mut self, b: u8) {
        self.Z = b == 0
    }

    fn N(&mut self, b: u8) {
        self.N = (b & 0b1000_0000) > 0
    }

    fn crosses(before: usize, after: usize) -> u8 {
        if ((before as u16) & 0xFF00) != ((after as u16) & 0xFF00) {1} else {0}
    }
}
