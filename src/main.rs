#![allow(non_snake_case)]

mod cpu;

//use std::env;
//use std::io;
//use std::fs::File;
//use std::io::Read;

//use std::io::BufReader;
//use std::io::BufRead;

fn main() {
    let mut cpu = cpu::Cpu::new();

    //let mut stdin = io::stdin();

    //let f = File::open("nestest.log").unwrap();
    //let file = BufReader::new(&f);
    //let mut lines = file.lines();

    let mut i: u64 = 1;

    while !cpu.halt {
        print!("{:X} ", i);
        i += 1;
        cpu.print_debug_info();
        cpu.step();
        //println!("         {:?}", lines.next().unwrap().unwrap());
        //let mut s: String = String::new();
        //stdin.read_line(&mut s);
    }
}
