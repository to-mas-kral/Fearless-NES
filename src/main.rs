#![feature(test)]
#![feature(nll)]

#[macro_use]
extern crate bitflags;

mod tests;
mod nes;

fn main() {
    nes::run();
}
