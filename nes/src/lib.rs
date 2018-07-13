#![feature(test)]
#![feature(nll)]

#[macro_escape]
macro_rules! debug_log {
    ($msg:expr, $($vars:tt),*) => {
        #[cfg(feature = "debug_log")]
        println!($msg, $($vars),*);
    };
}

use std::path::Path;

mod nes;
mod tests;

pub fn run() {
    let mut nes = nes::Nes::new(&Path::new(
        "/home/tomas/Documents/Programovani/fearless-nes/donkey kong.nes",
    )).unwrap();
    nes.run();
}
