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
1. Cargo
2. Macroquad dependencies

```sh
$ git clone git@github.com:TomasKralCZ/fearless-NES.git
$ cd fearless-NES
$ cargo run --release
```

# Accuracy
For accuracy tests, see TESTS.md.
Run these with 'cargo test'.

# TODO
- [ ] rest of the APU
- [ ] advanced mappers such as MMC3, MMC5
- [ ] various accuracy tests
- [x] save states
- [x] controller support

# Controls
### Keyboard
| NES controller  | Keyboard |
| ------------- | ------------- |
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
