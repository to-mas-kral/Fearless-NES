# Fearless-NES
Fearless-NES is a work-in-progress Nintendo enterntainment system emulator written in Rust.

# Screenshots

![Kirby's Adventure](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/Kirby'sAdventure.png)
![Super Mario Bros. 3](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/SuperMarioBros3.png)
![Legend Of Zelda](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/LegendOfZelda.png)
![Mega Man III](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/MegaManIII.png)
![Battletoads](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/Battletoads.png)
![Super Mario Bros](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/screenshots/SuperMarioBros.png)


# Build instructions:
1. Install [Macroquad dependencies](https://github.com/not-fl3/macroquad#linux) if you use Linux.
2. Build with `cargo run --release` and enjoy !

# Accuracy
For accuracy tests, see TESTS.md.
Run these with 'cargo test'.

# Supported mappers
| Mapper | Example Games |
| -------| ------------- |
| 0 (NROM) | Super Mario Bros, Donkey Kong, Balloon fight |
| 1 (MMC1) | Final Fantasy, Legend of Zelda, Mega Man 2 |
| 2 (UxROM) | Castlevania, Mega Man, Contra |
| 3 (CNROM) | Solomon's Key, Arkista's Ring |
| 4 (MMC3)* | Kirby's Adventure, Mega Man 3-6, Ninja Gaiden II: ... |
| 7 (AxROM) | Battletoads, Jeopardy! |

* Some MMC3 games like Mega Man III have major graphical issues, but SMB3 a Kirby's adventures do work.

With these mappers, Fearless-NES supports 84 % of commercial NES games.

# TODO (with descending priority)
- [ ] (frontend) use the NES 2.0 XML game database in addition to iNES headers
- [ ] (frontend) controllable overscan
- [ ] (frontend) memory editor
- [ ] (frontend) PPU viewer
- [ ] (core) rest of the APU
- [ ] (dev) change the zip dependency to something pure rust
- [ ] (frontend) user-defined RGB palettes
- [ ] (core) various accuracy tests
- [ ] (core) advanced mappers such as MMC5, VRC2/4...
- [ ] (dev) more integration tests
- [x] (frontend) persistent configuration
- [x] (frontend) save states
- [x] (frontend) controller support

# Controls
### Keyboard
| NES controller | Keyboard |
| -------------- | -------- |
| A  | F  |
| B  | D  |
| Select  | Space |
| Start  | Enter  |
| Up  | ArrowUp  |
| Down  | ArrowDown  |
| Right  | ArrowRight  |
| Left  | ArrowLeft  |

### Gamepad
![Gamepad Layout](https://raw.githubusercontent.com/TomasKralCZ/Fearless-NES/master/controller.svg)
