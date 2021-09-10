use directories::ProjectDirs;
use serde::Serialize;
use toml::{de::Error as TomlError, Value};

use std::{
    fmt::Display,
    fs::{create_dir_all, File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::{report_error, NES_HEIGHT, NES_WIDTH};

#[derive(Serialize)]
pub struct Config {
    // TODO: actually use these (and add position...) when Macroquad gets support for chaning window position
    pub window_width: i32,
    pub window_height: i32,

    pub save_folder_path: PathBuf,
    pub rom_folder_path: PathBuf,

    pub dark_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: NES_WIDTH as i32 * 2,
            window_height: NES_HEIGHT as i32 * 2,

            save_folder_path: PathBuf::from("~"),
            rom_folder_path: PathBuf::from("~"),

            dark_mode: true,
        }
    }
}

const CONFIG_FILENAME: &str = "Fearless-NES.toml";

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();

        if let Err(e) = config.load_config_file() {
            report_error(&format!("{}", e));
        }

        return config;
    }

    fn load_config_file(&mut self) -> Result<(), ConfigErr> {
        let proj_dirs =
            ProjectDirs::from("com", "Fearless-NES", "Fearless-NES").ok_or(ConfigErr::NoDir)?;
        let config_path = proj_dirs.config_dir().join(CONFIG_FILENAME);

        let mut config_file = File::open(config_path)?;

        let mut config_contents = String::new();
        if let Err(err) = config_file.read_to_string(&mut config_contents) {
            match err.kind() {
                // Config file doesn't exist, probably the first time using the program
                io::ErrorKind::NotFound => return Ok(()),
                _ => return Err(ConfigErr::FileReadErr),
            }
        }

        self.add_config_from_str(&config_contents)?;

        Ok(())
    }

    fn add_config_from_str(&mut self, config_contents: &str) -> Result<(), ConfigErr> {
        let value = config_contents.parse::<Value>()?;

        let fields = value.as_table().ok_or(ConfigErr::ParseErr)?;

        self.dark_mode = fields
            .get("dark_mode")
            .and_then(|v| v.as_bool())
            .ok_or(ConfigErr::ParseErr)?;

        self.rom_folder_path = PathBuf::from(
            fields
                .get("rom_folder_path")
                .and_then(|v| v.as_str())
                .ok_or(ConfigErr::ParseErr)?,
        );

        self.save_folder_path = PathBuf::from(
            fields
                .get("save_folder_path")
                .and_then(|v| v.as_str())
                .ok_or(ConfigErr::ParseErr)?,
        );

        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigErr> {
        let contents = toml::to_string(self).map_err(|_| ConfigErr::SaveErr)?;

        let proj_dirs =
            ProjectDirs::from("com", "Fearless-NES", "Fearless-NES").ok_or(ConfigErr::SaveErr)?;

        create_dir_all(proj_dirs.config_dir()).map_err(|_| ConfigErr::SaveErr)?;

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

pub enum ConfigErr {
    NoDir,
    FileReadErr,
    ParseErr,
    SaveErr,
}

impl From<TomlError> for ConfigErr {
    fn from(_: TomlError) -> Self {
        ConfigErr::ParseErr
    }
}

impl From<io::Error> for ConfigErr {
    fn from(_: io::Error) -> Self {
        ConfigErr::FileReadErr
    }
}

impl Display for ConfigErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigErr::NoDir => write!(f, "Couldn't find the configuration folder"),
            ConfigErr::FileReadErr => write!(f, "Couldn't read from the configuration file"),
            ConfigErr::ParseErr => write!(f, "Config file has invalid format"),
            ConfigErr::SaveErr => write!(f, "Couldn't save the configuration file"),
        }
    }
}
