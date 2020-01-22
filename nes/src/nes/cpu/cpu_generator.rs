#[allow(deprecated)]

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

mod opcodes;
use self::opcodes::{Timing, OPCODES};

fn main() {
    println!("cargo:rerun-if-changed=self");
    generate_cpu();
}

pub fn generate_cpu() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("src/nes/cpu/state_machine.rs");

    let mut generator = Generator::new();
    generator.generate_machine();
    generator.optimize_machine();
    generator.output_machine(&out_path);

    Command::new("rustfmt").args(&[out_path]).status().unwrap();
}

#[derive(Debug)]
struct Node {
    id: usize,
    code: String,
    opcode: usize,
    previous_nodes: Vec<usize>,
}

impl Node {
    fn new_node(id: usize, code: String) -> Node {
        Node {
            id,
            code,
            opcode: usize::max_value(),
            previous_nodes: Vec::new(),
        }
    }

    fn new_start_node(id: usize, code: String, opcode: usize) -> Node {
        Node {
            id,
            code,
            opcode: opcode,
            previous_nodes: Vec::new(),
        }
    }
}

struct Generator {
    state: usize,
    state_machine: BTreeMap<(usize, Vec<usize>), String>,

    nodes: Vec<Node>,
    end_nodes: Vec<usize>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            state: 0x100,
            state_machine: BTreeMap::new(),

            nodes: Vec::new(),
            end_nodes: Vec::new(),
        }
    }

    pub fn optimize_machine(&mut self) {
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
        s.push_str("impl Tick for super::Cpu {");
        s.push_str("#[allow(unused_variables)]");
        s.push_str("fn tick(&mut self) {");
        s.push_str("if self.halt {return}");
        s.push_str("self.odd_cycle = !self.odd_cycle;");
        s.push_str(
            "if self.dma.oam || self.dma.dmc { self.dma(); if self.dma.cycles != 0 {return;} }",
        );
        s.push_str(
            "macro_rules! cache_interrupts {($self:ident) => {self.cached_irq = self.irq_signal; self.cached_nmi = self.nmi_signal; /*self.irq_signal = false; self.nmi_signal = false;*/}};",
        );
        s.push_str(
            "macro_rules! check_dma {($self:ident) => {if $self.dma.hijack_read {self.dma.cycles = 1; return; }}}",
        );
        s.push_str("macro_rules! read {($self:ident, $addr: expr) => {$self.read($addr)};}");
        s.push_str("macro_rules! read_ab {() => {read!(self, self.ab)};}");
        s.push_str("macro_rules! sp_to_ab {($self:ident) => {$self.ab = $self.sp | 0x100};}");
        s.push_str("match self.state {");

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
        for (opcode, (addressing_mode, code)) in OPCODES.iter().enumerate() {
            match addressing_mode {
                Timing::Implied => self.implied(code, opcode),
                Timing::Rti => self.rti(opcode),
                Timing::Rts => self.rts(opcode),
                Timing::Jsr => self.jsr(opcode),
                Timing::Brk => self.brk(opcode),
                Timing::Pha => self.pha(opcode),
                Timing::Php => self.php(opcode),
                Timing::Plp => self.plp(opcode),
                Timing::Pla => self.pla(opcode),
                Timing::Immediate => self.immediate(code, opcode),
                Timing::Accumulator => self.accumulator(code, opcode),
                Timing::ZeroPage => self.zero_page(code, opcode),
                Timing::ZeroPageX => self.zero_page_x(code, opcode),
                Timing::ZeroPageY => self.zero_page_y(code, opcode),
                Timing::Relative => self.relative(code, opcode),
                Timing::Absolute => self.absolute(code, opcode),
                Timing::AbsoluteX => self.absolute_x_or_y(code, opcode, "x"),
                Timing::AbsoluteY => self.absolute_x_or_y(code, opcode, "y"),
                Timing::Indirect => self.indirect(opcode),
                Timing::AbsoluteJmp => self.absolute_jmp(opcode),
                Timing::IndirectX => self.indirect_x(code, opcode),
                Timing::IndirectY => self.indirect_y(code, opcode),
                Timing::ZeroPageRmw => self.zero_page_rmw(code, opcode),
                Timing::ZeroPageXRmw => self.zero_page_x_rmw(code, opcode),
                Timing::AbsoluteRmw => self.absolute_rmw(code, opcode),
                Timing::AbsoluteXRmw => self.absolute_x_rmw(code, opcode),
                Timing::ZeroPageSt => self.zero_page_st(code, opcode),
                Timing::ZeroPageXSt => self.zero_page_x_or_y_st(code, opcode, "x"),
                Timing::ZeroPageYSt => self.zero_page_x_or_y_st(code, opcode, "y"),
                Timing::AbsoluteSt => self.absolute_st(code, opcode),
                Timing::AbsoluteXSt => self.absolute_x_or_y_st(code, opcode, "x"),
                Timing::AbsoluteYSt => self.absolute_x_or_y_st(code, opcode, "y"),
                Timing::IndirectXSt => self.indirect_x_st(code, opcode),
                Timing::IndirectYSt => self.indirect_y_st(code, opcode),
                Timing::IndirectXIllegal => self.indirect_x_illegal(code, opcode),
                Timing::IndirectYIllegal => self.indirect_y_illegal(code, opcode),
                Timing::AbsoluteYIllegal => self.absolute_y_illegal(code, opcode),
            };
        }

        let fetch_next = String::from("cache_interrupts!(self); let int = if self.take_interrupt {0} else {1}; read_ab!(); check_dma!(self); self.state = u16::from(int * self.db); self.pc = (self.pc as u16).wrapping_add(int as u16) as usize; self.ab = self.pc");
        self.state_machine.insert((0x100, vec![0]), fetch_next);

        println!("{}", self.nodes.len());
        for n in self.nodes.iter() {
            println!("{:?}", n);
        }
        panic!();
    }

    //#[deprecated]
    fn add_single(&mut self, s: String, op: usize) {
        self.state_machine.insert((op, vec![0x100]), s);
    }

    //#[deprecated]
    fn add_entry(&mut self, s: String, op: usize) {
        let next = self.update_state();
        self.state_machine.insert((op, vec![next]), s);
    }

    //#[deprecated]
    fn add_middle(&mut self, s: String) {
        let prev = self.state;
        let next = self.update_state();
        self.state_machine.insert((prev, vec![next]), s);
    }

    //#[deprecated]
    fn add_exit(&mut self, s: String) {
        let prev = self.state;
        self.state_machine.insert((prev, vec![0x100]), s);
    }

    fn add_node(&mut self, code: &str) -> usize {
        let tmp = self.nodes.len();
        self.nodes.push(Node::new_node(tmp, code.to_string()));
        tmp
    }

    fn add_end_node(&mut self, code: &str) -> usize {
        for id in self.end_nodes.iter() {
            if self.nodes[*id].code == code {
                return *id;
            }
        }

        let id = self.add_node(code);
        self.end_nodes.push(id);
        id
    }

    fn add_middle_node(&mut self, next_node: usize, code: &str) -> usize {
        for id in self.nodes[next_node].previous_nodes.iter() {
            if self.nodes[*id].code == code {
                return *id;
            }
        }

        let id = self.add_node(code);
        self.nodes[next_node].previous_nodes.push(id);
        id
    }

    fn add_start_node(&mut self, next_node: usize, op: usize, code: &str) {
        let tmp = self.nodes.len();
        self.nodes.push(Node::new_start_node(tmp, code.to_string(), op));
        self.nodes[next_node].previous_nodes.push(tmp);
    }

    fn add_single_node(&mut self, op: usize, code: &str) {
        let tmp = self.nodes.len();
        self.nodes.push(Node::new_start_node(tmp, code.to_string(), op));
    }

    fn plp(&mut self, op: usize) {
        /* let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.pull_status(self.db); self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");
        self.add_start_node(
            next_node,
            op,
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;",
        );



        /* self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;"
                .to_string(),
        );
        self.add_exit(
            "self.check_interrupts(); read_ab!();  check_dma!(self); self.pull_status(self.db); self.ab = self.pc; self.state = 0x100;"
                .to_string(),
        ); */ */


        
    }

    fn pla(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.lda(self.db); self.ab = self.pc; self.state = 0x100");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;");




        /* self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;"
                .to_string(),
        );
        self.add_exit("self.check_interrupts(); read_ab!(); check_dma!(self); self.lda(self.db); self.ab = self.pc; self.state = 0x100".to_string()); */
    }

    fn php(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); self.push_status(true); self.ab = self.pc; self.state = 0x100;");
        self.add_start_node(next_node, op, "cache_interrupts!(self); read_ab!(); check_dma!(self); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");



        /* self.add_entry(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(
            "self.check_interrupts(); self.push_status(true); self.ab = self.pc; self.state = 0x100;"
                .to_string(),
        ); */
    }

    fn pha(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); self.write(self.ab, self.a); self.ab = self.pc; self.state = 0x100;");
        self.add_start_node(next_node, op, "cache_interrupts!(self); read_ab!(); check_dma!(self); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");



        /* self.add_entry(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(
            "self.check_interrupts(); self.write(self.ab, self.a); self.ab = self.pc; self.state = 0x100;"
                .to_string(),
        ); */
    }

    fn jsr(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; check_dma!(self); self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); self.write(self.ab, (self.pc & 0xFF) as u8); self.ab = self.pc; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "self.write(self.ab, (self.pc >> 8) as u8); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");



        /* self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle("self.write(self.ab, (self.pc >> 8) as u8); sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string());
        self.add_middle("cache_interrupts!(self); self.write(self.ab, (self.pc & 0xFF) as u8); self.ab = self.pc; self.state = <>;".to_string());
        self.add_exit("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; check_dma!(self); self.ab = self.pc; self.state = 0x100;".to_string()); */
    }

    fn brk(&mut self, op: usize) {
        let next_node = self.add_end_node("read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab += 1; self.i = true; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "if !(self.take_interrupt && self.reset_signal) {self.push_status(true);} self.ab = self.interrupt_address(); self.take_interrupt = false; self.interrupt_type = super::InterruptType::None; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "if !(self.take_interrupt && self.reset_signal) {self.write(self.ab, (self.pc & 0xFF) as u8);} sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "if !(self.take_interrupt && self.reset_signal) {self.write(self.ab, (self.pc >> 8) as u8);} sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); let int = if self.take_interrupt {0} else {1}; self.pc += int; sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;");



        /* self.add_entry("read_ab!(); check_dma!(self); let int = if self.take_interrupt {0} else {1}; self.pc += int; sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string(), op);
        self.add_middle("if !(self.take_interrupt && self.reset_signal) {self.write(self.ab, (self.pc >> 8) as u8);} sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string());
        self.add_middle("if !(self.take_interrupt && self.reset_signal) {self.write(self.ab, (self.pc & 0xFF) as u8);} sp_to_ab!(self); self.sp = (self.sp as u8).wrapping_sub(1) as usize; self.state = <>;".to_string());
        self.add_middle("if !(self.take_interrupt && self.reset_signal) {self.push_status(true);} self.ab = self.interrupt_address(); self.take_interrupt = false; self.interrupt_type = super::InterruptType::None; self.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab += 1; self.i = true; self.state = <>;"
                .to_string(),
        );
        self.add_exit("read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;".to_string()); */
    }

    fn rti(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.pull_status(self.db); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;");



        /* self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.pull_status(self.db); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string());
        self.add_exit("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100;".to_string()); */
    }

    fn rts(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;");




        /* self.add_entry(
            "read_ab!(); check_dma!(self); sp_to_ab!(self); self.state = <>;".to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>".to_string(),
        );
        self.add_middle("read_ab!(); check_dma!(self); self.temp = self.db as usize; self.sp = (self.sp as u8).wrapping_add(1) as usize; sp_to_ab!(self); self.state = <>;".to_string());
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = <>;".to_string());
        self.add_exit("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = 0x100;".to_string()); */
    }

    fn implied(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); {} self.state = 0x100;",
            i_code
        );

        self.add_single_node(op, &code);



        /* self.add_single(code, op); */
    }

    fn immediate(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = 0x100;",
            i_code
        );

        self.add_single_node(op, &code);



        /* self.add_single(code, op); */
    }

    fn accumulator(&mut self, i_code: &str, op: usize) {
        let code = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); {} self.state = 0x100;",
            i_code
        );

        self.add_single_node(op, &code);



        /* self.add_single(code, op); */
    }

    fn zero_page(&mut self, i_code: &str, op: usize) {
        let cycle_0 = "cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;";
        let cycle_1 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100;",
            i_code
        );

        let next_node = self.add_end_node(&cycle_1);
        self.add_start_node(next_node, op, cycle_0);


        /* self.add_entry(cycle_0.to_string(), op);
        self.add_exit(cycle_1); */
    }

    fn zero_page_x(&mut self, i_code: &str, op: usize) {
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;",
        );
        let cycle_1 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;");
        let cycle_2 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100;",
            i_code
        );


        let next_node = self.add_end_node(&cycle_2);
        let next_node = self.add_middle_node(next_node, &cycle_1);
        self.add_start_node(next_node, op, &cycle_0);



        /* self.add_entry(cycle_0, op);
        self.add_middle(cycle_1);
        self.add_exit(cycle_2); */
    }

    fn zero_page_y(&mut self, i_code: &str, op: usize) {
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;",
        );
        let cycle_1 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = (self.ab + self.y as usize) & 0xFF; self.state = <>;");
        let cycle_2 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100;",
            i_code
        );

        let next_node = self.add_end_node(&cycle_2);
        let next_node = self.add_middle_node(next_node, &cycle_1);
        self.add_start_node(next_node, op, &cycle_0);

        /* self.add_entry(cycle_0, op);
        self.add_middle(cycle_1);
        self.add_exit(cycle_2); */
    }

    fn relative(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.ab = self.pc; self.state = 0x100");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); self.take_interrupt = false; read_ab!(); check_dma!(self); self.take_branch(); self.ab = self.pc; self.state = if self.temp != 0 {<>} else {0x100};");
        self.add_start_node(next_node, op, "self.check_interrupts(); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = if {} {{<>}} else {{0x100}}");


        /* self.add_entry(format!("self.check_interrupts(); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = if {} {{<>}} else {{0x100}}", i_code), op);
        self.add_middle("cache_interrupts!(self); self.take_interrupt = false; read_ab!(); check_dma!(self); self.take_branch(); self.ab = self.pc; self.state = if self.temp != 0 {<>} else {0x100};".to_string());
        self.add_exit(
            "self.check_interrupts(); read_ab!(); check_dma!(self); self.ab = self.pc; self.state = 0x100"
                .to_string(),
        ); */
    }

    fn absolute(&mut self, i_code: &str, op: usize) {
        let cycle_2 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100",
            i_code
        );


        let next_node = self.add_end_node(&cycle_2);
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;");

        /* self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100",
            i_code
        )); */
    }

    fn absolute_x_or_y(&mut self, i_code: &str, op: usize, reg: &str) {
        //TODO: page crossing
        let cycle_1_state = self.update_state();
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;",
        );

        let cycle_2_state = self.update_state();
        let cycle_3_state = self.update_state();
        let cycle_1 = format!("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | ((self.temp + self.{} as usize) & 0xFF); self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = if (self.temp + self.{} as usize) < 0x100 {{?<>}} else {{<>}};", reg, reg);
        let cycle_2 =
            String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = (self.ab as u16).wrapping_add(0x100) as usize; self.state = <>");
        let cycle_3 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100;",
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
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100");
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab & 0xFF00) | ((self.ab + 1) & 0xFF); self.state = <>");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.state = <>");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>");
/* 
        self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.state = <>"
                .to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab & 0xFF00) | ((self.ab + 1) & 0xFF); self.state = <>".to_string());
        self.add_exit("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100".to_string()); */
    }

    fn absolute_jmp(&mut self, op: usize) {
        let next_node = self.add_end_node("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100");
        self.add_start_node(next_node, op, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>");

        /* self.add_entry("cache_interrupts!(self); read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>".to_string(), op);
        self.add_exit("self.check_interrupts(); read_ab!(); check_dma!(self); self.pc = ((self.db as usize) << 8) | self.temp; self.ab = self.pc; self.state = 0x100".to_string()); */
    }

    fn indirect_x(&mut self, i_code: &str, op: usize) {
        let cycle_4 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100",
            i_code
        );

        let next_node = self.add_end_node(&cycle_4);
        let next_node = self.add_middle_node(next_node, "cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>");

/*         self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>".to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>"
                .to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.state = <>;".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100",
            i_code
        )); */
    }

    fn indirect_y(&mut self, i_code: &str, op: usize) {
        //TODO: page crossing
        let cycle_1_state = self.update_state();
        let cycle_0 = String::from(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;",
        );

        let cycle_2_state = self.update_state();
        let cycle_1 = String::from("read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1usize) & 0xFF; self.state = <>;");

        let cycle_3_state = self.update_state();
        let cycle_4_state = self.update_state();
        let cycle_2 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = (((self.db as usize) << 8) | ((self.temp + self.y as usize) & 0xFF)) as usize; self.state = if (self.temp + self.y as usize) < 0x100 {?<>} else {<>};");
        let cycle_3 = String::from("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = (self.ab as u16).wrapping_add(0x100) as usize; self.state = <>;");

        self.state += 1;
        let cycle_4 = format!(
            "self.check_interrupts(); read_ab!(); check_dma!(self); let val = self.db; {} self.ab = self.pc; self.state = 0x100;",
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
        let cycle_2 = format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code);

        let next_node = self.add_end_node("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, &cycle_2);
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;");

        /* self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string()); */
    }

    fn zero_page_x_rmw(&mut self, i_code: &str, op: usize) {
        let cycle_3 = format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code);

        let next_node = self.add_end_node("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, &cycle_3);
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;");

/*         self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string()); */
    }

    fn absolute_rmw(&mut self, i_code: &str, op: usize) {
        let cycle_3 = format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code);

        let next_node = self.add_end_node("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;");
        let next_node = self.add_middle_node(next_node, &cycle_3);
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;");
        let next_node = self.add_middle_node(next_node, "read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;");
        self.add_start_node(next_node, op, "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;");
/* 
        self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string()) */
    }

    fn absolute_x_rmw(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | ((self.temp + self.x as usize) & 0xFF); self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;".to_string());
        self.add_middle("read_ab!(); check_dma!(self); if (self.temp + self.x as usize) >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize}; self.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn zero_page_st(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn zero_page_x_or_y_st(&mut self, i_code: &str, op: usize, reg: &str) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(format!("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = (self.ab + self.{} as usize) & 0xFF; self.state = <>;", reg));
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn absolute_st(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = <>;",
            i_code
        ));
    }

    fn absolute_x_or_y_st(&mut self, i_code: &str, op: usize, reg: &str) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(format!("read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | ((self.temp + self.{} as usize) & 0xFF); self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;", reg));
        self.add_middle(format!("cache_interrupts!(self); read_ab!(); check_dma!(self); if (self.temp + self.{} as usize) >= 0x100 {{self.ab = (self.ab as u16).wrapping_add(0x100) as usize}}; self.state = <>;", reg));
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn indirect_x_st(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.state = <>;".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn indirect_y_st(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle("read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | ((self.temp + self.y as usize) & 0xFF); self.state = <>;".to_string());
        self.add_middle("cache_interrupts!(self); read_ab!(); check_dma!(self); if self.temp + self.y as usize >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize;} self.state = <>;".to_string());
        self.add_exit(format!(
            "self.check_interrupts(); {} self.ab = self.pc; self.state = 0x100;",
            i_code
        ));
    }

    fn indirect_x_illegal(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = (self.ab + self.x as usize) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | self.temp; self.state = <>;".to_string(),
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!("cache_interrupts!(self); self.write(self.ab, self.temp as u8); let val = self.temp as u8; {} self.state = <>;", i_code));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn indirect_y_illegal(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.ab = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.ab = (self.ab + 1) & 0xFF; self.state = <>;"
                .to_string(),
        );
        self.add_middle("read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | ((self.temp + self.y as usize) & 0xFF); self.state = <>;".to_string());
        self.add_middle("read_ab!(); check_dma!(self); if (self.temp + self.y as usize) >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize}; self.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!(
            "cache_interrupts!(self); self.write(self.ab, self.temp as u8); {} self.state = <>;",
            i_code
        ));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn absolute_y_illegal(&mut self, i_code: &str, op: usize) {
        let next_node = self.add_end_node();
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        let next_node = self.add_middle_node(next_node, );
        self.add_start_node(next_node, op, );

        self.add_entry(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.pc = (self.pc as u16).wrapping_add(1) as usize; self.ab = self.pc; self.state = <>;"
                .to_string(),
            op,
        );
        self.add_middle("read_ab!(); check_dma!(self); self.ab = ((self.db as usize) << 8) | ((self.temp + self.y as usize) & 0xFF); self.pc = (self.pc as u16).wrapping_add(1) as usize; self.state = <>;".to_string());
        self.add_middle("read_ab!(); check_dma!(self); if (self.temp + self.y as usize) >= 0x100 {self.ab = (self.ab as u16).wrapping_add(0x100) as usize;}; self.state = <>;".to_string());
        self.add_middle(
            "read_ab!(); check_dma!(self); self.temp = self.db as usize; self.state = <>;"
                .to_string(),
        );
        self.add_middle(format!(
            "cache_interrupts!(self); self.write(self.ab, self.temp as u8); {} self.state = <>;",
            i_code
        ));
        self.add_exit("self.check_interrupts(); self.write(self.ab, self.temp as u8); self.ab = self.pc; self.state = 0x100;".to_string());
    }

    fn update_state(&mut self) -> usize {
        self.state += 1;
        self.state
    }
}
