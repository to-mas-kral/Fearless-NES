#![feature(test)]
#![feature(nll)]

mod tests;
mod nes;

fn main() {
    let mut nes = nes::Nes::<nes::mapper::Nrom>::new();
    nes.run();
}
