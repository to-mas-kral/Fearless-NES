#![allow(non_snake_case)]

mod cpu;

//use std::env;
use std::io;
use std::fs::File;
use std::io::Read;

use std::io::BufReader;
use std::io::BufRead;

fn main() {
    let mut cpu = cpu::Cpu::new();

    let mut stdin = io::stdin();

    let f = File::open("nestest.log").unwrap();
    let mut file = BufReader::new(&f);
    let mut lines = file.lines();

    let mut i: u64 = 1;

    while i < 180_000_000 {
        print!("{} ", i);
        i += 1;
        cpu.step();
        if i > 8900 {
            let _ = stdin.read(&mut [0u8]).unwrap();
        }
        println!("         {:?}", lines.next().unwrap().unwrap());
    }
}
