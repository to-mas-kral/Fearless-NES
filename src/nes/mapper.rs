pub trait Mapper {
    fn write_prg();
    fn read_prrg();
    fn read_chr(&mut self, adr: usize) -> u8;
    fn write_chr(&mut self, adr: usize, val: u8);
}

pub struct Nrom {
    pub chr: [u8; 0x2000],
}

impl Nrom {
    pub fn new() -> Nrom {
        Nrom { chr: [0; 0x2000] }
    }
}

impl Mapper for Nrom {
    fn write_prg() {}
    fn read_prrg() {}
    fn read_chr(&mut self, adr: usize) -> u8 {
        self.chr[adr]
    }
    fn write_chr(&mut self, adr: usize, val: u8) {
        self.chr[adr] = val;
    }
}
