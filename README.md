# Fearless-NES
Fearless-NES is a work-in-progress Nintendo enterntainment system emulator written in Rust.

# Screenshots

![Castlevania](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/Castlevania.png)
![Legend Of Zelda](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/LegendOfZelda.png)
![Mega Man II](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/MegaManII.png)

![Metal Gear](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/MetalGear.png)
![Solomon's key](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/SolomonsKey.png)
![Super Mario Bros](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/SuperMarioBros.png)


# Building from source requires:
1. Rustup
2. Cargo
3. rustfmt
4. SDL 2

```sh
$ git clone git@github.com:TomasKralCZ/fearless-NES.git
$ cd fearless-NES
$ rustup default nightly
$ cargo run --release -p -frontend-sdl -- -r 'path to ROM'
```

# Accuracy
For accuracy tests, see TESTS.md.
Run these with 'cargo test'.

# What still needs to be done (see TODO.md)
- [ ] majority of the APU
- [ ] advanced mappers such as MMC3, MMC5
- [ ] various accuracy tests

# Controls
| NES controller  | Keyboard |
| ------------- | ------------- |
| A  | A  |
| B  | S  |
| Select  | Y/Z |
| Start  | X  |
| Up  | ArrowUp  |
| Down  | ArrowDown  |
| Right  | ArrowRight  |
| Left  | ArrowLeft  |
