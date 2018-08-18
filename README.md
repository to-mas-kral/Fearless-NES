# Fearless-NES
Fearless-NES is a work-in-progress Nintendo enterntainment system emulator written in Rust.

# Building from source
1. Install Rustup
2. Install Cargo
3. Install rustfmt

```sh
$ git clone git@github.com:TomasKralCZ/fearless-NES.git
$ cd fearless-NES
$ rustup default nightly
$ cargo run --release -p -frontend-sdl
```

# Roadmap
- [x] cycle-accurate CPU