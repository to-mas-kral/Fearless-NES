use directories::ProjectDirs;
use eyre::{eyre, Result};
use fearless_nes::{NES_HEIGHT, NES_WIDTH};
use serde::{Deserialize, Serialize};

use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::dialog::DialogReport;

use super::nesrender::Overscan;

mod keybinds;

pub use keybinds::Keybinds;

#[derive(Serialize, Deserialize)]
pub struct Config {
    // TODO: window config
    pub window_width: i32,
    pub window_height: i32,

    pub save_folder_path: PathBuf,
    pub rom_folder_path: PathBuf,

    pub dark_mode: bool,

    /* TOML docs: "Note that the TOML format has a restriction that if a table itself contains tables,
    all keys with non-table values must be emitted first." */
    pub overscan: Overscan,
    pub keybinds: Keybinds,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: NES_WIDTH as i32 * 2,
            window_height: NES_HEIGHT as i32 * 2,

            save_folder_path: PathBuf::from("~"),
            rom_folder_path: PathBuf::from("~"),

            dark_mode: true,

            overscan: Overscan::new(),
            keybinds: Keybinds::new(),
        }
    }
}

const CONFIG_FILENAME: &str = "Fearless-NES.toml";

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();

        config
            .load_config_file()
            .report_dialog_with(|e| {
                format!(
                "Couldn't load the configuration file. Relying on default configuration. Error: {}",
                e
            )
            })
            .ok();

        config
    }

    fn load_config_file(&mut self) -> Result<()> {
        let proj_dirs = ProjectDirs::from("com", "Fearless-NES", "Fearless-NES")
            .ok_or(eyre!("Couldn't locate project-dirs"))?;
        let config_path = proj_dirs.config_dir().join(CONFIG_FILENAME);

        let mut config_file = File::open(config_path)?;
        let mut config_contents = String::new();
        if let Err(err) = config_file.read_to_string(&mut config_contents) {
            match err.kind() {
                // Config file doesn't exist, probably the first time using the program
                io::ErrorKind::NotFound => return Ok(()),
                _ => return Err(eyre!("")),
            }
        }

        *self = toml::from_str(&config_contents)?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let contents = toml::to_string(self)?;

        let proj_dirs = ProjectDirs::from("com", "Fearless-NES", "Fearless-NES")
            .ok_or(eyre!("Couldn't locate project-dirs"))?;

        create_dir_all(proj_dirs.config_dir())?;

        let config_path = proj_dirs.config_dir().join(CONFIG_FILENAME);

        let mut config_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path)?;
        config_file.write_all(contents.as_bytes())?;

        Ok(())
    }
}
