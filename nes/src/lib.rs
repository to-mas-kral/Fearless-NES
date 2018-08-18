#![feature(test)]
#![feature(nll)]

#[macro_escape]
macro_rules! debug_log {
    ($msg:expr, $($vars:tt),*) => {
        #[cfg(feature = "debug_log")]
        println!($msg, $($vars),*);
    };
}

pub mod nes;
mod tests;
