use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;

mod parser;

pub fn generate_cpu() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("src/nes/cpu/state_machine.rs");

    let source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let opcode_path = Path::new(&source_dir).join("cpu_generator/opcodes.txt");

    let mut src = String::new();
    let mut f = File::open(opcode_path).unwrap();
    f.read_to_string(&mut src).unwrap();

    let mut parser = parser::Parser::new(&src);
    let instructions = parser.parse_file();

    let mut generator = Generator::new();
    generator.generate_machine(instructions);
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
        //TODO: implement optimization
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

        s.push_str("use super::Tick;");
        s.push_str("use nes::memory::MemoryOps;");
        s.push_str("impl Tick for super::Cpu {");
        s.push_str("#[allow(unused_variables)]");
        s.push_str("fn tick(&mut self) {");
        s.push_str("if self.halt {return}");
        s.push_str(
            "macro_rules! cache_irq {($self:ident) => {self.cached_irq = self.interrupt_bus.borrow().irq_signal;};}",
        );
        s.push_str("macro_rules! read_ab {($self:ident) => {$self.mem.read($self.ab)};}");
        s.push_str("macro_rules! sp_to_ab {($self:ident) => {$self.ab = $self.sp | 0x100};}");
        s.push_str("debug_log!(\"executing opcode 0x{:X}\", (self.state));");
        s.push_str("debug_log!(\"CPU state: {}\", (self.debug_info()));");
        s.push_str("match self.state {");

        for (key, val) in self.state_machine.iter() {
            let st = format!("0x{:X} => {{ {} }},", key.0, val);
            s.push_str(st.as_str());
        }

        s.push_str("op => unreachable!(\"propably a Rust compiler error, opcode: 0x{:X}\", op),");
        s.push_str("} } }");

        file.write(s.as_bytes())
            .expect("error writing to a file while generating the cpu");
    }

    pub fn generate_machine(&mut self, instructions: Vec<ParsedInstruction>) {
        for i in instructions {
            for op in &i.opcodes {
                match op.0.as_str() {
                    "implied" => match op.1 {
                        0x28 => self.plp(op.1),
                        0x68 => self.pla(op.1),
                        0x08 => self.php(op.1),
                        0x48 => self.pha(op.1),
                        0x20 => self.jsr(op.1),
                        0x00 => self.brk(op.1),
                        0x40 => self.rti(op.1),
                        0x60 => self.rts(op.1),
                        _ => self.implied(&i.code, op.1),
                    },
                    "immediate" => self.immediate(&i.code, op.1),
                    "accumulator" => self.accumulator(&i.code, op.1),
                    "zero_page" => self.zero_page(&i.code, op.1),
                    "zero_page_x" => self.zero_page_x(&i.code, op.1),
                    "zero_page_y" => self.zero_page_y(&i.code, op.1),
                    "relative" => self.relative(&i.code, op.1),
                    "absolute" => self.absolute(&i.code, op.1),
                    "absolute_x" => self.absolute_x_or_y(&i.code, op.1, "x"),
                    "absolute_y" => self.absolute_x_or_y(&i.code, op.1, "y"),
                    "indirect" => self.indirect(op.1),
                    "absolute_jmp" => self.absolute_jmp(op.1),
                    "indirect_x" => self.indirect_x(&i.code, op.1),
                    "indirect_y" => self.indirect_y(&i.code, op.1),
                    "zero_page_rmw" => self.zero_page_rmw(&i.code, op.1),
                    "zero_page_x_rmw" => self.zero_page_x_rmw(&i.code, op.1),
                    "absolute_rmw" => self.absolute_rmw(&i.code, op.1),
                    "absolute_x_rmw" => self.absolute_x_rmw(&i.code, op.1),
                    "zero_page_st" => self.zero_page_st(&i.code, op.1),
                    "zero_page_x_st" => self.zero_page_x_or_y_st(&i.code, op.1, "x"),
                    "zero_page_y_st" => self.zero_page_x_or_y_st(&i.code, op.1, "y"),
                    "absolute_st" => self.absolute_st(&i.code, op.1),
                    "absolute_x_st" => self.absolute_x_or_y_st(&i.code, op.1, "x"),
                    "absolute_y_st" => self.absolute_x_or_y_st(&i.code, op.1, "y"),
                    "indirect_x_st" => self.indirect_x_st(&i.code, op.1),
                    "indirect_y_st" => self.indirect_y_st(&i.code, op.1),
                    "indirect_x_illegal" => self.indirect_x_illegal(&i.code, op.1),
                    "indirect_y_illegal" => self.indirect_y_illegal(&i.code, op.1),
                    "absolute_y_illegal" => self.absolute_y_illegal(&i.code, op.1),
                    am => panic!("unknown addressing mode: {}", am),
                };
            }
        }

        let next = String::from("cache_irq!(self); let int = if self.take_interrupt {0} else {1}; self.state = u16::from(int * read_ab!(self)); self.pc += int as usize; self.ab = self.pc");
        self.state_machine.insert((0x100, vec![0]), next);
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
            "read_ab!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "cache_irq!(self); self.pop(self.ab); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;"
                .to_string(),
        );
        self.add_exit(
            "self.check_interrupts(); self.pull_status(); self.ab = self.pc; self.state = 0x100;"
                .to_string(),
        );
    }

    fn pla(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "cache_irq!(self); self.pop(self.ab); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;"
                .to_string(),
        );
        self.add_exit("self.check_interrupts(); let a = self.pop(self.ab); self.lda(a); self.ab = self.pc; self.state = 0x100".to_string());
    }

    fn php(&mut self, op: usize) {
        self.add_entry(
            "cache_irq!(self); read_ab!(self); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(
            "self.check_interrupts(); self.push_status(true); self.ab = self.pc; self.state = 0x100;"
                .to_string(),
        );
    }

    fn pha(&mut self, op: usize) {
        self.add_entry(
            "cache_irq!(self); read_ab!(self); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(
            "self.check_interrupts(); self.push(self.ab, self.a); self.ab = self.pc; self.state = 0x100;"
                .to_string(),
        );
    }

    fn jsr(&mut self, op: usize) {
        self.add_entry(
            "self.temp = read_ab!(self) as usize; self.pc += 1; sp_to_ab!(self); self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("self.pop(self.ab); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string());
        self.add_middle("self.push(self.ab, (self.pc >> 8) as u8); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string());
        self.add_middle("cache_irq!(self); self.push(self.ab, (self.pc & 0xFF) as u8); self.ab = self.pc; self.state = <>;".to_string());
        self.add_exit("self.check_interrupts(); self.pc = ((read_ab!(self) as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn brk(&mut self, op: usize) {
        self.add_entry("read_ab!(self); let int = if self.take_interrupt {0} else {1}; self.pc += int; sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string(), op);
        self.add_middle("if !(self.take_interrupt && self.pending_reset) {self.push(self.ab, (self.pc >> 8) as u8);} sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string());
        self.add_middle("if !(self.take_interrupt && self.pending_reset) {self.push(self.ab, (self.pc & 0xFF) as u8); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;}".to_string());
        self.add_middle("if !(self.take_interrupt && self.pending_reset) {self.push_status(true);} self.ab = self.interrupt_address(); self.take_interrupt = false; self.interrupt_type = super::InterruptType::None; self.state = <>;".to_string());
        self.add_middle(
            "self.temp = read_ab!(self) as usize; self.ab += 1; self.i = true; self.state = <>;"
                .to_string(),
        );
        self.add_exit("self.pc = ((read_ab!(self) as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn rti(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "self.pop(self.ab); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string(),
        );
        self.add_middle(
            "self.pull_status(); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string(),
        );
        self.add_middle("cache_irq!(self); self.temp = self.pop(self.ab) as usize; self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string());
        self.add_exit("self.check_interrupts(); self.pc = ((read_ab!(self) as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn rts(&mut self, op: usize) {
        self.add_entry(
            "read_ab!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "self.pop(self.ab); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>".to_string(),
        );
        self.add_middle("self.temp = self.pop(self.ab) as usize; self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string());
        self.add_middle("cache_irq!(self); self.pc = ((self.pop(self.ab) as usize) << 8) | self.temp; self.ab = self.pc; self.state = <>;".to_string());
        self.add_exit("self.check_interrupts(); read_ab!(self); self.pc += 1; self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn implied(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.check_interrupts(); read_ab!(self); {} self.state = 0x100;",
            i_code
        );
        self.add_single(code, op);
    }

    fn immediate(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.pc += 1; self.ab = self.pc; self.state = 0x100;",
            i_code
        );
        self.add_single(code, op);
    }

    fn accumulator(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.check_interrupts(); read_ab!(self); {} self.state = 0x100;",
            i_code
        );
        self.add_single(code, op);
    }

    fn zero_page(&mut self, i_code: &str, op: usize) {
        let cycle_0 = String::from(
            "cache_irq!(self); self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;",
        );
        let cycle_1 = format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        );

        self.add_entry(cycle_0, op);
        self.add_exit(cycle_1);
    }

    fn zero_page_x(&mut self, i_code: &str, op: usize) {
        let cycle_0 =
            String::from("self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;");
        let cycle_1 = String::from("cache_irq!(self); read_ab!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;");
        let cycle_2 = format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        );

        self.add_entry(cycle_0, op);
        self.add_middle(cycle_1);
        self.add_exit(cycle_2);
    }

    fn zero_page_y(&mut self, i_code: &str, op: usize) {
        let cycle_0 =
            String::from("self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;");
        let cycle_1 = String::from("cache_irq!(self); read_ab!(self); self.ab = (self.ab + self.y as usize) & 0xFF; self.state = <>;");
        let cycle_2 = format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        );

        self.add_entry(cycle_0, op);
        self.add_middle(cycle_1);
        self.add_exit(cycle_2);
    }

    fn relative(&mut self, i_code: &str, op: usize) {
        self.add_entry(format!("self.check_interrupts(); self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = if {} {{<>}} else {{0x100}}", i_code), op);
        self.add_middle("cache_irq!(self); read_ab!(self); self.take_branch(); self.ab = self.pc; self.state = if self.temp > 0 {<>} else {0x100};".to_string());
        self.add_exit("self.check_interrupts(); read_ab!(self); self.pc += ((self.temp << 8) as u8) as usize; self.ab = self.pc; self.state = 0x100".to_string());
    }

    fn absolute(&mut self, i_code: &str, op: usize) {
        self.add_entry("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;".to_string(), op);
        self.add_middle("cache_irq!(self); self.ab = ((read_ab!(self) as usize) << 8) | self.temp; self.pc += 1; self.state = <>".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100",
            i_code
        ));
    }

    fn absolute_x_or_y(&mut self, i_code: &str, op: usize, reg: &str) {
        let cycle_1_state = self.update_state();
        let cycle_0 = String::from("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;");

        let cycle_2_state = self.update_state();
        let cycle_3_state = self.update_state();
        let cycle_1 = format!("cache_irq!(self); self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.{} as usize) & 0xFF); self.pc += 1; self.state = if (self.temp + self.{} as usize) < 0x100 {{?<>}} else {{<>}};", reg, reg);
        let cycle_2 =
            String::from("cache_irq!(self); read_ab!(self); self.ab = (self.ab as u16).wrapping_add(0x100) as usize; self.state = <>");
        let cycle_3 = format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100;",
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
            "self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>"
                .to_string(),
            op,
        );
        self.add_middle(
            "self.ab = ((read_ab!(self) as usize) << 8) | self.temp; self.state = <>".to_string(),
        );
        self.add_middle("cache_irq!(self); self.temp = read_ab!(self) as usize; self.ab = (self.ab & 0xFF00) | ((self.ab + 1) & 0xFF); self.state = <>".to_string());
        self.add_exit("self.check_interrupts(); self.pc = ((read_ab!(self) as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100".to_string());
    }

    fn absolute_jmp(&mut self, op: usize) {
        self.add_entry("cache_irq!(self); self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>".to_string(), op);
        self.add_exit("self.check_interrupts(); self.pc = ((read_ab!(self) as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100".to_string());
    }

    fn indirect_x(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>".to_string(),
            op,
        );
        self.add_middle("self.mem.read_zp(self.ab); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>".to_string());
        self.add_middle("self.temp = self.mem.read_zp(self.ab) as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>".to_string());
        self.add_middle("cache_irq!(self); self.ab = ((self.mem.read_zp(self.ab) as usize) << 8) | self.temp; self.state = <>;".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100",
            i_code
        ));
    }

    fn indirect_y(&mut self, i_code: &str, op: usize) {
        let cycle_1_state = self.update_state();
        let cycle_0 =
            String::from("self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;");

        let cycle_2_state = self.update_state();
        let cycle_1 = String::from("self.temp = self.mem.read_zp(self.ab) as usize; self.ab = (self.ab + 1usize) & 0xFF; self.state = <>;");

        let cycle_3_state = self.update_state();
        let cycle_4_state = self.update_state();
        let cycle_2 = String::from("cache_irq!(self); self.ab = (((self.mem.read_zp(self.ab) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF)) as usize; self.state = if (self.temp + self.y as usize) < 0x100 {?<>} else {<>};");
        let cycle_3 = String::from("cache_irq!(self); read_ab!(self); self.ab = (self.ab as u16).wrapping_add(0x100) as usize; self.state = <>;");

        self.state += 1;
        let cycle_4 = format!(
            "self.check_interrupts(); let val = read_ab!(self); {} self.ab = self.pc; self.state = 0x100;",
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
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "self.temp = self.mem.read_zp(self.ab) as usize; self.state = <>;".to_string(),
        );
        self.add_middle(format!("cache_irq!(self); self.mem.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn zero_page_x_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle("self.mem.read_zp(self.ab); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;".to_string());
        self.add_middle(
            "self.temp = self.mem.read_zp(self.ab) as usize; self.state = <>;".to_string(),
        );
        self.add_middle(format!("cache_irq!(self); self.mem.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn absolute_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;".to_string(), op);
        self.add_middle("self.ab = ((read_ab!(self) as usize) << 8) | self.temp; self.pc += 1; self.state = <>;".to_string());
        self.add_middle("self.temp = read_ab!(self) as usize; self.state = <>;".to_string());
        self.add_middle(format!("cache_irq!(self); self.mem.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string())
    }

    fn absolute_x_rmw(&mut self, i_code: &str, op: usize) {
        self.add_entry("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;".to_string(), op);
        self.add_middle("self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.x as usize) & 0xFF); self.pc += 1; self.state = <>;".to_string());
        self.add_middle("read_ab!(self); if (self.temp + self.x as usize) >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize}; self.state = <>;".to_string());
        self.add_middle("self.temp = read_ab!(self) as usize; self.state = <>;".to_string());
        self.add_middle(format!("cache_irq!(self); self.mem.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn zero_page_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "cache_irq!(self); self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn zero_page_x_or_y_st(&mut self, i_code: &str, op: usize, reg: &str) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle(format!("cache_irq!(self); self.mem.read_zp(self.ab); self.ab = (self.ab + self.{} as usize) & 0xFF; self.state = <>;", reg));
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn absolute_st(&mut self, i_code: &str, op: usize) {
        self.add_entry("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;".to_string(), op);
        self.add_middle("cache_irq!(self); self.ab = ((read_ab!(self) as usize) << 8) | self.temp; self.pc += 1; self.state = <>".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = <>;",
            i_code
        ));
    }

    fn absolute_x_or_y_st(&mut self, i_code: &str, op: usize, reg: &str) {
        self.add_entry("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;".to_string(), op);
        self.add_middle(format!("self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.{} as usize) & 0xFF); self.pc += 1; self.state = <>;", reg));
        self.add_middle(format!("cache_irq!(self); read_ab!(self); if (self.temp + self.{} as usize) >= 0x100 {{self.ab = (self.ab as u16).wrapping_add(0x100) as usize}}; self.state = <>;", reg));
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn indirect_x_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle("self.mem.read_zp(self.ab); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;".to_string());
        self.add_middle("self.temp = self.mem.read_zp(self.ab) as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;".to_string());
        self.add_middle("cache_irq!(self); self.ab = ((self.mem.read_zp(self.ab) as usize) << 8) | self.temp; self.state = <>;".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn indirect_y_st(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle("self.temp = self.mem.read_zp(self.ab) as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;".to_string());
        self.add_middle("self.ab = ((self.mem.read_zp(self.ab) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF); self.state = <>;".to_string());
        self.add_middle("cache_irq!(self); read_ab!(self); if self.temp + self.y as usize >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize;} self.state = <>;".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn indirect_x_illegal(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle("self.mem.read_zp(self.ab); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;".to_string());
        self.add_middle("self.temp = self.mem.read_zp(self.ab) as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;".to_string());
        self.add_middle(
            "self.ab = ((self.mem.read_zp(self.ab) as usize) << 8) | self.temp; self.state = <>;"
                .to_string(),
        );
        self.add_middle("self.temp = read_ab!(self) as usize; self.state = <>;".to_string());
        self.add_middle(format!("cache_irq!(self); self.mem.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn indirect_y_illegal(&mut self, i_code: &str, op: usize) {
        self.add_entry(
            "self.ab = read_ab!(self) as usize; self.pc += 1; self.state = <>;".to_string(),
            op,
        );
        self.add_middle("self.temp = self.mem.read_zp(self.ab) as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;".to_string());
        self.add_middle("self.ab = ((self.mem.read_zp(self.ab) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF); self.state = <>;".to_string());
        self.add_middle("read_ab!(self); if (self.temp + self.y as usize) >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize}; self.state = <>;".to_string());
        self.add_middle("self.temp = read_ab!(self) as usize; self.state = <>;".to_string());
        self.add_middle(format!(
            "cache_irq!(self); self.mem.write(self.ab, self.temp as u8); {} self.state = <>;",
            i_code
        ));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn absolute_y_illegal(&mut self, i_code: &str, op: usize) {
        self.add_entry("self.temp = read_ab!(self) as usize; self.pc += 1; self.ab = self.pc; self.state = <>;".to_string(), op);
        self.add_middle("self.ab = ((read_ab!(self) as usize) << 8) | ((self.temp + self.y as usize) & 0xFF); self.pc += 1; self.state = <>;".to_string());
        self.add_middle("read_ab!(self); if (self.temp + self.y as usize) >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize;}; self.state = <>;".to_string());
        self.add_middle("self.temp = read_ab!(self) as usize; self.state = <>;".to_string());
        self.add_middle(format!(
            "cache_irq!(self); self.mem.write(self.ab, self.temp as u8); {} self.state = <>;",
            i_code
        ));
        self.add_exit("self.check_interrupts(); self.mem.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn update_state(&mut self) -> usize {
        self.state += 1;
        self.state
    }
}
