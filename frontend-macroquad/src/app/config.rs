use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::Serialize;
use toml::Value;

use std::{
    convert::TryInto,
    fs::{create_dir_all, File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::{report_error, NES_HEIGHT, NES_WIDTH};

use super::nesrender::Overscan;

#[derive(Serialize)]
pub struct Config {
    // TODO: actually use these (and add position...) when Macroquad gets support for chaning window position
    pub window_width: i32,
    pub window_height: i32,

    pub save_folder_path: PathBuf,
    pub rom_folder_path: PathBuf,

    pub dark_mode: bool,

    /* TOML docs: "Note that the TOML format has a restriction that if a table itself contains tables,
    all keys with non-table values must be emitted first." */
    pub overscan: Overscan,
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
        }
    }
}

const CONFIG_FILENAME: &str = "Fearless-NES.toml";

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();

        if let Err(e) = config.load_config_file() {
            report_error(&format!(
                "Couldn't load the configuration file. Relying on default configuration. Error: {}",
                e
            ));
        }

        return config;
    }

    fn load_config_file(&mut self) -> Result<()> {
        let proj_dirs = ProjectDirs::from("com", "Fearless-NES", "Fearless-NES")
            .ok_or(anyhow!("Couldn't locate project-dirs"))?;
        let config_path = proj_dirs.config_dir().join(CONFIG_FILENAME);

        let mut config_file = File::open(config_path)?;

        let mut config_contents = String::new();
        if let Err(err) = config_file.read_to_string(&mut config_contents) {
            match err.kind() {
                // Config file doesn't exist, probably the first time using the program
                io::ErrorKind::NotFound => return Ok(()),
                _ => return Err(anyhow!("")),
            }
        }

        if let Err(_) = self.add_config_from_str(&config_contents) {
            report_error(&format!("The configuration file contains invalid data"));
        }

        Ok(())
    }

    fn add_config_from_str(&mut self, config_contents: &str) -> Result<()> {
        let value = config_contents.parse::<Value>()?;

        let fields = value.as_table().ok_or(anyhow!("parse error"))?;

        self.dark_mode = fields
            .get("dark_mode")
            .and_then(|v| v.as_bool())
            .ok_or(anyhow!("parse error"))?;

        self.rom_folder_path = PathBuf::from(
            fields
                .get("rom_folder_path")
                .and_then(|v| v.as_str())
                .ok_or(anyhow!("parse error"))?,
        );

        self.save_folder_path = PathBuf::from(
            fields
                .get("save_folder_path")
                .and_then(|v| v.as_str())
                .ok_or(anyhow!("parse error"))?,
        );

        let overscan = fields
            .get("overscan")
            .and_then(|v| v.as_table())
            .ok_or(anyhow!("parse error"))?;

        self.overscan.top = overscan
            .get("top")
            .and_then(|v| v.as_integer())
            .ok_or(anyhow!("parse error"))?
            .try_into()
            .map_err(|_| anyhow!("parse error"))?;

        self.overscan.right = overscan
            .get("right")
            .and_then(|v| v.as_integer())
            .ok_or(anyhow!("parse error"))?
            .try_into()
            .map_err(|_| anyhow!("parse error"))?;

        self.overscan.bottom = overscan
            .get("bottom")
            .and_then(|v| v.as_integer())
            .ok_or(anyhow!("parse error"))?
            .try_into()
            .map_err(|_| anyhow!("parse error"))?;

        self.overscan.left = overscan
            .get("left")
            .and_then(|v| v.as_integer())
            .ok_or(anyhow!("parse error"))?
            .try_into()
            .map_err(|_| anyhow!("parse error"))?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let contents = toml::to_string(self)?;

        let proj_dirs = ProjectDirs::from("com", "Fearless-NES", "Fearless-NES")
            .ok_or(anyhow!("Couldn't locate project-dirs"))?;

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
