use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use std::ops::{Index, IndexMut};

pub trait MemoryOps {
    fn read(&self, _index: usize) -> u8;
    fn read_zp(&self, _index: usize) -> u8;
    fn write(&self, _index: usize, value: u8);
}

pub struct Memory {
    mem: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { mem: [0; 0x10000] }
    }

    pub fn clear(&mut self) {
        self.mem = [0; 0x10000];
    }

    pub fn load_mapper_0(&mut self, f: &mut File) {
        let _bytes: Result<Vec<u8>, _> = f.bytes().collect();
        let bytes = _bytes.expect("error while reading the file");

        for i in 0..16384 {
            self.mem[0x7FFF + i] = bytes[15 + i];
            self.mem[0x7FFF + 0x4000 + i] = bytes[15 + i];
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;
    fn index(&self, _index: usize) -> &u8 {
        if _index >= 0x2000 && _index <= 0x2007 {
            println!("Accessing PPU control registers");
        }

        &self.mem[_index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        if _index >= 0x2000 && _index <= 0x2007 {
            println!("Accessing PPU control registers");
        }

        &mut self.mem[_index]
    }
}

impl MemoryOps for Rc<RefCell<Memory>> {
    fn read(&self, index: usize) -> u8 {
        /* println!(
            ":::: reading 0x{:X} from 0x{:X}",
            self.borrow_mut().mem[index],
            index
        ); */
        self.borrow_mut().mem[index]
    }

    fn read_zp(&self, index: usize) -> u8 {
        //TODO: how does this differ ??
        /* println!(
            ":::: reading 0x{:X} from 0x{:X}",
            self.borrow_mut().mem[index],
            index
        ); */
        self.borrow_mut().mem[index]
    }

    fn write(&self, index: usize, value: u8) {
        //println!(":::: writing 0x{:X} to 0x{:X}", value, index);

        self.borrow_mut().mem[index] = value;
    }
}
