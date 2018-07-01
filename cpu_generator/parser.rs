use std::iter::Peekable;
use std::str::Chars;

use super::ParsedInstruction;

pub struct Parser<'s> {
    source: Peekable<Chars<'s>>,
}

impl<'s> Parser<'s> {
    pub fn new(src: &'s String) -> Parser<'s> {
        Parser {
            source: src.chars().peekable(),
        }
    }

    pub fn parse_file(&mut self) -> Vec<ParsedInstruction> {
        let mut instructions = Vec::new();
        while let Some(_) = self.source.peek() {
            let inst = self.parse_instruction();
            instructions.push(inst);
        }

        instructions
    }

    fn parse_instruction(&mut self) -> ParsedInstruction {
        self.skip_newline();

        for _ in 1..=4 {
            self.source.next();
        }

        self.skip_whitespace();

        let mut code = String::new();
        while self.source.peek() != Some(&'\n') {
            code.push(self.source.next().unwrap());
        }
        self.source.next();
        self.source.next();

        let opcodes = self.parse_opcodes();

        ParsedInstruction { code, opcodes }
    }

    fn parse_opcodes(&mut self) -> Vec<(String, usize)> {
        let mut opcodes = Vec::new();
        while self.source.peek() == Some(&' ') {
            let opcode = self.parse_opcode();
            opcodes.push(opcode);
        }

        opcodes
    }

    fn parse_opcode(&mut self) -> (String, usize) {
        self.skip_whitespace();
        let mut addressing_mode = String::new();
        while self.source.peek() != Some(&' ') {
            addressing_mode.push(self.source.next().unwrap());
        }
        self.source.next();

        let mut opcode = String::new();
        while self.source.peek() != Some(&'\n') {
            opcode.push(self.source.next().unwrap());
        }
        self.source.next();

        (
            addressing_mode,
            usize::from_str_radix(&opcode[2..=3], 16).unwrap(),
        )
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ') = self.source.peek() {
            self.source.next();
        }
    }

    fn skip_newline(&mut self) {
        if let Some('\n') = self.source.peek() {
            self.source.next();
        }
    }
}
