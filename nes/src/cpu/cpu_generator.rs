use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

mod opcodes;
use self::opcodes::{Timing, OPCODES};

pub fn main() {
    println!("cargo:rerun-if-changed=self");
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("src/cpu/state_machine.rs");

    let mut generator = Generator::new();
    generator.generate_machine();
    generator.optimize_machine();
    generator.output_machine(&out_path);

    Command::new("rustfmt").args(&[out_path]).status().unwrap();
}

type Opcodes = Vec<Opcode>;
type Opcode = (String, usize);

#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    code: String,
    opcodes: Opcodes,
}

struct Generator {
    state: usize,
    state_machine: BTreeMap<(usize, Vec<usize>), String>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            state: 0x100,
            state_machine: BTreeMap::new(),
        }
    }

    pub fn optimize_machine(&mut self) {
        //TODO: optimize CPU state machine
        let mut optimized = BTreeMap::new();

        for (key, val) in &mut self.state_machine {
            let mut code = val.clone();
            if key.1.len() > 1 {
                let state_num_2 = format!("0x{:X}", key.1[1]);
                code = code.replace("?<>", &state_num_2);
            }
            let state_num = format!("0x{:X}", key.1[0]);
            optimized.insert(key.clone(), code.replace("<>", &state_num));
        }

        self.state_machine = optimized;
    }

    pub fn output_machine(&mut self, path: &PathBuf) {
        let mut file = File::create(path).unwrap();

        let mut s = String::new();

        s.push_str("use super::Nes;");
        s.push_str("impl Nes {");
        s.push_str("#[allow(unused_variables)]");
        s.push_str("pub fn cpu_tick(&mut self) {");
        s.push_str("if self.cpu.halt {return}");
        s.push_str("self.cpu.odd_cycle = !self.cpu.odd_cycle;");
        s.push_str(
            "if self.cpu.dma.oam || self.cpu.dma.dmc { self.cpu_dma(); if self.cpu.dma.cycles != 0 {return;} }",
        );
        s.push_str(
            "macro_rules! cache_interrupts {($self:ident) => {self.cpu.cached_irq = self.cpu.irq_signal; self.cpu.cached_nmi = self.cpu.nmi_signal; /*self.irq_signal = false; self.nmi_signal = false;*/}};",
        );
        s.push_str(
            "macro_rules! check_dma {($self:ident) => {if $self.cpu.dma.hijack_read {self.cpu.dma.cycles = 1; return; }}}",
        );
        s.push_str(
            "macro_rules! read {($self:ident, $addr: expr) => {$self.cpu_read($addr)};}",
        );
        s.push_str("macro_rules! read_ab {() => {read!(self, self.cpu.ab)};}");
        s.push_str(
            "macro_rules! sp_to_ab {($self:ident) => {$self.cpu.ab = $self.cpu.sp | 0x100};}",
        );
        s.push_str("match self.cpu.state {");

        for (key, val) in self.state_machine.iter() {
            let st = format!("0x{:X} => {{ {} }},", key.0, val);
            s.push_str(st.as_str());
        }

        s.push_str("op => unreachable!(\"invalid state, opcode: 0x{:X}\", op),");
        s.push_str("} } }");

        file.write(s.as_bytes())
            .expect("error writing to a file while generating the cpu");
    }

    pub fn generate_machine(&mut self) {
        for (opcode, opinfo) in OPCODES.iter().enumerate() {
            match opinfo.0 {
                Timing::Implied => self.implied(opinfo.1, opcode),
                Timing::Rti => self.rti(opcode),
                Timing::Rts => self.rts(opcode),
                Timing::Jsr => self.jsr(opcode),
                Timing::Brk => self.brk(opcode),
                Timing::Pha => self.pha(opcode),
                Timing::Php => self.php(opcode),
                Timing::Plp => self.plp(opcode),
                Timing::Pla => self.pla(opcode),
                Timing::Immediate => self.immediate(opinfo.1, opcode),
                Timing::Accumulator => self.accumulator(opinfo.1, opcode),
                Timing::ZeroPage => self.zero_page(opinfo.1, opcode),
                Timing::ZeroPageX => self.zero_page_x(opinfo.1, opcode),
                Timing::ZeroPageY => self.zero_page_y(opinfo.1, opcode),
                Timing::Relative => self.relative(opinfo.1, opcode),
                Timing::Absolute => self.absolute(opinfo.1, opcode),
                Timing::AbsoluteX => self.absolute_x_or_y(opinfo.1, opcode, "x"),
                Timing::AbsoluteY => self.absolute_x_or_y(opinfo.1, opcode, "y"),
                Timing::Indirect => self.indirect(opcode),
                Timing::AbsoluteJmp => self.absolute_jmp(opcode),
                Timing::IndirectX => self.indirect_x(opinfo.1, opcode),
                Timing::IndirectY => self.indirect_y(opinfo.1, opcode),
                Timing::ZeroPageRmw => self.zero_page_rmw(opinfo.1, opcode),
                Timing::ZeroPageXRmw => self.zero_page_x_rmw(opinfo.1, opcode),
                Timing::AbsoluteRmw => self.absolute_rmw(opinfo.1, opcode),
                Timing::AbsoluteXRmw => self.absolute_x_rmw(opinfo.1, opcode),
                Timing::ZeroPageSt => self.zero_page_st(opinfo.1, opcode),
                Timing::ZeroPageXSt => self.zero_page_x_or_y_st(opinfo.1, opcode, "x"),
                Timing::ZeroPageYSt => self.zero_page_x_or_y_st(opinfo.1, opcode, "y"),
                Timing::AbsoluteSt => self.absolute_st(opinfo.1, opcode),
                Timing::AbsoluteXSt => self.absolute_x_or_y_st(opinfo.1, opcode, "x"),
                Timing::AbsoluteYSt => self.absolute_x_or_y_st(opinfo.1, opcode, "y"),
                Timing::IndirectXSt => self.indirect_x_st(opinfo.1, opcode),
                Timing::IndirectYSt => self.indirect_y_st(opinfo.1, opcode),
                Timing::IndirectXIllegal => self.indirect_x_illegal(opinfo.1, opcode),
                Timing::IndirectYIllegal => self.indirect_y_illegal(opinfo.1, opcode),
                Timing::AbsoluteYIllegal => self.absolute_y_illegal(opinfo.1, opcode),
            };
        }

        let fetch_next = String::from("cache_interrupts!(self); let int = if self.cpu.take_interrupt {0} else {1}; read_ab!(); check_dma!(self); self.cpu.state = u16::from(int * self.cpu.db); self.cpu.pc = (self.cpu.pc as u16).wrapping_add(int as u16) as usize; self.cpu.ab = self.cpu.pc");
        self.state_machine.insert((0x100, vec![0]), fetch_next);
    }

    fn add_single(&mut self, s: String, op: usize) {
        self.state_machine.insert((op, vec![0x100]), s);
    }

    fn add_entry(&mut self, s: String, op: usize) {
        let next = self.update_state();
        self.state_machine.insert((op, vec![next]), s);
    }

    fn add_middle(&mut self, s: String) {
        let prev = self.state;
        let next = self.update_state();
        self.state_machine.insert((prev, vec![next]), s);
    }

    fn add_exit(&mut self, s: String) {
        let prev = self.state;
        self.state_machine.insert((prev, vec![0x100]), s);
    }

    fn plp(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.cpu.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;"
                .to_string(),
        );
        self.add_exit(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu_pull_status(self.cpu.db); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;"
                .to_string(),
        );
    }

    fn pla(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.cpu.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;"
                .to_string(),
        );
        self.add_exit("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu_lda(self.cpu.db); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100".to_string());
    }

    fn php(&mut self, op: usize) {
        self.add_entry(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); sp_to_ab!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(
            "self.cpu_check_interrupts(); self.cpu_push_status(true); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;"
                .to_string(),
        );
    }

    fn pha(&mut self, op: usize) {
        self.add_entry(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); sp_to_ab!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(
            "self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.a); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;"
                .to_string(),
        );
    }

    fn jsr(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle("self.cpu_write(self.cpu.ab, (self.cpu.pc >> 8) as u8); sp_to_ab!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;".to_string());
        self.add_middle("cache_interrupts!(self); self.cpu_write(self.cpu.ab, (self.cpu.pc & 0xFF) as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = <>;".to_string());
        self.add_exit("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp; check_dma!(self); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn brk(&mut self, op: usize) {
        self.add_entry("read_ab!(); check_dma!(self); let int = if self.cpu.take_interrupt {0} else {1}; self.cpu.pc += int; sp_to_ab!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;".to_string(), op);
        self.add_middle("if !(self.cpu.take_interrupt && self.cpu.reset_signal) {self.cpu_write(self.cpu.ab, (self.cpu.pc >> 8) as u8);} sp_to_ab!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;".to_string());
        self.add_middle("if !(self.cpu.take_interrupt && self.cpu.reset_signal) {self.cpu_write(self.cpu.ab, (self.cpu.pc & 0xFF) as u8);} sp_to_ab!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_sub(1) as usize; self.cpu.state = <>;".to_string());
        self.add_middle("if !(self.cpu.take_interrupt && self.cpu.reset_signal) {self.cpu_push_status(true);} self.cpu.ab = self.cpu_interrupt_address(); self.cpu.take_interrupt = false; self.cpu.interrupt_type = super::InterruptType::None; self.cpu.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab += 1; self.cpu.i = true; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_exit("read_ab!(); check_dma!(self); self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn rti(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.cpu.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;".to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu_pull_status(self.cpu.db); self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;".to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;".to_string());
        self.add_exit("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn rts(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.cpu.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>".to_string(),
        );
        self.add_middle("read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.sp = (self.cpu.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.cpu.state = <>;".to_string());
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;".to_string());
        self.add_exit("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn implied(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); {} self.cpu.state = 0x100;",
            i_code
        );
        self.add_single(code, op);
    }

    fn immediate(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        );
        self.add_single(code, op);
    }

    fn accumulator(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); {} self.cpu.state = 0x100;",
            i_code
        );
        self.add_single(code, op);
    }

    fn zero_page(&mut self, i_code: &str, op: usize) {
        let cycle_0 = String::from(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;",
        );
        let cycle_1 = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        );

        self.add_entry(cycle_0, op);
        self.add_exit(cycle_1);
    }

    fn zero_page_x(&mut self, i_code: &str, op: usize) {
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;",
        );
        let cycle_1 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF; self.cpu.state = <>;");
        let cycle_2 = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        );

        self.add_entry(cycle_0, op);
        self.add_middle(cycle_1);
        self.add_exit(cycle_2);
    }

    fn zero_page_y(&mut self, i_code: &str, op: usize) {
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;",
        );
        let cycle_1 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.y as usize) & 0xFF; self.cpu.state = <>;");
        let cycle_2 = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        );

        self.add_entry(cycle_0, op);
        self.add_middle(cycle_1);
        self.add_exit(cycle_2);
    }

    fn relative(&mut self, i_code: &str, op: usize) {
        self.add_entry(format!("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = if {} {{<>}} else {{0x100}}", i_code), op);
        self.add_middle("cache_interrupts!(self); self.cpu.take_interrupt = false; read_ab!(); check_dma!(self); self.cpu_take_branch(); self.cpu.ab = self.cpu.pc; self.cpu.state = if self.cpu.temp != 0 {<>} else {0x100};".to_string());
        self.add_exit(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100"
                .to_string(),
        );
    }

    fn absolute(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>".to_string());
        self.add_exit(format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100",
            i_code
        ));
    }

    fn absolute_x_or_y(&mut self, i_code: &str, op: usize, reg: &str) {
        let cycle_1_state = self.update_state();
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;",
        );

        let cycle_2_state = self.update_state();
        let cycle_3_state = self.update_state();
        let cycle_1 = format!("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.{} as usize) & 0xFF); self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = if (self.cpu.temp + self.cpu.{} as usize) < 0x100 {{?<>}} else {{<>}};", reg, reg);
        let cycle_2 =
            String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize; self.cpu.state = <>");
        let cycle_3 = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        );

        self.state_machine
            .insert((op, vec![cycle_1_state]), cycle_0);
        self.state_machine
            .insert((cycle_1_state, vec![cycle_2_state, cycle_3_state]), cycle_1);
        self.state_machine
            .insert((cycle_2_state, vec![cycle_3_state]), cycle_2);
        self.state_machine
            .insert((cycle_3_state, vec![0x100]), cycle_3);
    }

    fn indirect(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.state = <>"
                .to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab & 0xFF00) | ((self.cpu.ab + 1) & 0xFF); self.cpu.state = <>".to_string());
        self.add_exit("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100".to_string());
    }

    fn absolute_jmp(&mut self, op: usize) {
        self.add_entry("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>".to_string(), op);
        self.add_exit("self.cpu_check_interrupts(); read_ab!(); check_dma!(self); self.cpu.pc = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100".to_string());
    }

    fn indirect_x(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF; self.cpu.state = <>".to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab + 1) & 0xFF; self.cpu.state = <>"
                .to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.state = <>;".to_string());
        self.add_exit(format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100",
            i_code
        ));
    }

    fn indirect_y(&mut self, i_code: &str, op: usize) {
        let cycle_1_state = self.update_state();
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;",
        );

        let cycle_2_state = self.update_state();
        let cycle_1 = String::from("read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab + 1usize) & 0xFF; self.cpu.state = <>;");

        let cycle_3_state = self.update_state();
        let cycle_4_state = self.update_state();
        let cycle_2 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = (((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.y as usize) & 0xFF)) as usize; self.cpu.state = if (self.cpu.temp + self.cpu.y as usize) < 0x100 {?<>} else {<>};");
        let cycle_3 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize; self.cpu.state = <>;");

        self.state += 1;
        let cycle_4 = format!(
            "self.cpu_check_interrupts(); read_ab!(); check_dma!(self); let val = self.cpu.db; {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        );

        self.state_machine
            .insert((op, vec![cycle_1_state]), cycle_0);
        self.state_machine
            .insert((cycle_1_state, vec![cycle_2_state]), cycle_1);
        self.state_machine
            .insert((cycle_2_state, vec![cycle_3_state, cycle_4_state]), cycle_2);
        self.state_machine
            .insert((cycle_3_state, vec![cycle_4_state]), cycle_3);
        self.state_machine
            .insert((cycle_4_state, vec![0x100]), cycle_4);
    }

    fn zero_page_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); let val = self.cpu.temp as u8; {} self.cpu.state = <>;", i_code));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn zero_page_x_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); let val = self.cpu.temp as u8; {} self.cpu.state = <>;", i_code));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn absolute_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); let val = self.cpu.temp as u8; {} self.cpu.state = <>;", i_code));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string())
    }

    fn absolute_x_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.x as usize) & 0xFF); self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;".to_string());
        self.add_middle("read_ab!(); check_dma!(self); if (self.cpu.temp + self.cpu.x as usize) >= 0x100 {self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize}; self.cpu.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); let val = self.cpu.temp as u8; {} self.cpu.state = <>;", i_code));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn zero_page_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(format!(
            "self.cpu_check_interrupts(); {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        ));
    }

    fn zero_page_x_or_y_st(&mut self, i_code: &str, op: usize, reg: &str) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(format!("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.{} as usize) & 0xFF; self.cpu.state = <>;", reg));
        self.add_exit(format!(
            "self.cpu_check_interrupts(); {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        ));
    }

    fn absolute_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>".to_string());
        self.add_exit(format!(
            "self.cpu_check_interrupts(); {} self.cpu.ab = self.cpu.pc; self.cpu.state = <>;",
            i_code
        ));
    }

    fn absolute_x_or_y_st(&mut self, i_code: &str, op: usize, reg: &str) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(format!("read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.{} as usize) & 0xFF); self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;", reg));
        self.add_middle(format!("cache_interrupts!(self); read_ab!(); check_dma!(self); if (self.cpu.temp + self.cpu.{} as usize) >= 0x100 {{self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize}}; self.cpu.state = <>;", reg));
        self.add_exit(format!(
            "self.cpu_check_interrupts(); {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        ));
    }

    fn indirect_x_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab + 1) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.state = <>;".to_string());
        self.add_exit(format!(
            "self.cpu_check_interrupts(); {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        ));
    }

    fn indirect_y_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab + 1) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle("read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.y as usize) & 0xFF); self.cpu.state = <>;".to_string());
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); if self.cpu.temp + self.cpu.y as usize >= 0x100 {self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;} self.cpu.state = <>;".to_string());
        self.add_exit(format!(
            "self.cpu_check_interrupts(); {} self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;",
            i_code
        ));
    }

    fn indirect_x_illegal(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = (self.cpu.ab + self.cpu.x as usize) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab + 1) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | self.cpu.temp; self.cpu.state = <>;".to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); let val = self.cpu.temp as u8; {} self.cpu.state = <>;", i_code));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn indirect_y_illegal(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.ab = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.ab = (self.cpu.ab + 1) & 0xFF; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle("read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.y as usize) & 0xFF); self.cpu.state = <>;".to_string());
        self.add_middle("read_ab!(); check_dma!(self); if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize}; self.cpu.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!(
            "cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); {} self.cpu.state = <>;",
            i_code
        ));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn absolute_y_illegal(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.ab = self.cpu.pc; self.cpu.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("read_ab!(); check_dma!(self); self.cpu.ab = ((self.cpu.db as usize) << 8) | ((self.cpu.temp + self.cpu.y as usize) & 0xFF); self.cpu.pc = (self.cpu.pc as u16).wrapping_add(1) as usize; self.cpu.state = <>;".to_string());
        self.add_middle("read_ab!(); check_dma!(self); if (self.cpu.temp + self.cpu.y as usize) >= 0x100 {self.cpu.ab = (self.cpu.ab as u16).wrapping_add(0x100) as usize;}; self.cpu.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.cpu.temp = self.cpu.db as usize; self.cpu.state = <>;"
                .to_string(),
        );
        self.add_middle(format!(
            "cache_interrupts!(self); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); {} self.cpu.state = <>;",
            i_code
        ));
        self.add_exit("self.cpu_check_interrupts(); self.cpu_write(self.cpu.ab, self.cpu.temp as u8); self.cpu.ab = self.cpu.pc; self.cpu.state = 0x100;".to_string());
    }

    fn update_state(&mut self) -> usize {
        self.state += 1;
        self.state
    }
}
