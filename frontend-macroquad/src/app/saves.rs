use std::{
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use egui::{CtxRef, TextureId};
use fearless_nes::{Nes, NesError};
use macroquad::prelude::{ImageFormat, Texture2D};
use zip::{result::ZipError, write::FileOptions};

use crate::{
    app::{report_error, App, Gui},
    NES_HEIGHT, NES_WIDTH,
};

use crate::app::{get_save_named_path, nesrender::NesRender};

pub struct Saves {
    pub saves: Vec<Save>,

    pub window_shown: bool,
    pub folder_path: PathBuf,
    pub folder_changed: bool,
}

impl Saves {
    pub fn new() -> Self {
        Self {
            saves: Vec::new(),

            window_shown: false,
            folder_path: PathBuf::new(),
            folder_changed: false,
        }
    }

    fn build_saves_view(&mut self) -> Result<(), SaveErr> {
        self.saves.clear();

        let saves_paths: Vec<PathBuf> = fs::read_dir(&self.folder_path)
            .map_err(|_| SaveErr::DirOpenErr)?
            .filter_map(|e| e.ok())
            .filter(|de| de.path().extension() == Some(OsStr::new("fnes")))
            .map(|de| de.path())
            .collect();

        for path in saves_paths {
            if let Err(e) = self.push_save_view(&path) {
                report_error(&format!("{}\n{:?}", e, path));
            }
        }

        Ok(())
    }

    fn push_save_view(&mut self, save_path: &Path) -> Result<(), SaveErr> {
        let save_file = std::fs::File::open(&save_path)?;
        let mut save_archive = zip::ZipArchive::new(&save_file)?;

        let screenshot = {
            let mut screenshot_zip = save_archive.by_name(SCREENSHOT_PATH)?;
            let mut screenshot = Vec::with_capacity(screenshot_zip.size() as usize);
            screenshot_zip.read_to_end(&mut screenshot)?;

            screenshot
        };

        let texture = Texture2D::from_file_with_format(&screenshot, Some(ImageFormat::Png));

        self.saves.push(Save::new(
            save_path
                .file_stem()
                .ok_or(SaveErr::DirContentsErr)?
                .to_string_lossy()
                .into_owned(),
            save_file,
            texture,
        ));

        Ok(())
    }

    pub fn create_save(
        &mut self,
        nes: &Option<Nes>,
        nesrender: &mut NesRender,
        save_folder_path: &Path,
    ) -> Result<(), SaveErr> {
        let path = match get_save_named_path(Some(save_folder_path), "Fearless-NES save", "fnes") {
            Ok(Some(p)) => p,
            Ok(None) | Err(_) => return Ok(()),
        };

        if let Some(ref nes) = nes {
            let save_data = nes.save_state()?;

            let mut screenshot: Vec<u8> = Vec::with_capacity(NES_WIDTH * NES_HEIGHT);
            {
                let mut encoder =
                    png::Encoder::new(&mut screenshot, NES_WIDTH as u32, NES_HEIGHT as u32);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().map_err(|_| SaveErr::SaveErr)?;

                // TODO: get_image_data() - create PR in Macroquad...
                let image_data: Vec<u8> = nesrender
                    .image
                    .get_image_data()
                    .iter()
                    .flat_map(|e| *e)
                    .collect();

                writer
                    .write_image_data(&image_data)
                    .map_err(|_| SaveErr::SaveErr)?;
            };

            let file = std::fs::File::create(&path)?;
            let mut zip = zip::ZipWriter::new(file);

            let options = FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);
            zip.start_file("screenshot.png", options)?;
            zip.write_all(&screenshot)?;

            zip.start_file("savestate.fnes", options)?;
            zip.write_all(&save_data)?;

            zip.finish()?;
        };

        Ok(())
    }

    pub fn load_zipped_save(savefile: &File) -> Result<Nes, SaveErr> {
        let mut save_archive = zip::ZipArchive::new(savefile)?;
        let mut savestate_zip = save_archive.by_name(SAVESTATE_PATH)?;
        let mut savestate = Vec::with_capacity(savestate_zip.size() as usize);
        savestate_zip.read_to_end(&mut savestate)?;

        Ok(Nes::load_state(&savestate)?)
    }
}

impl Gui for Saves {
    fn gui_window(app: &mut App, egui_ctx: &CtxRef) {
        if app.saves.window_shown {
            if app.saves.folder_changed {
                app.saves.folder_changed = false;

                if let Err(e) = app.saves.build_saves_view() {
                    report_error(&format!("{}", e))
                }
            }

            let saves = &mut app.saves.saves;
            let window_shown = &mut app.saves.window_shown;
            let nes = &mut app.nes;

            egui::Window::new("Saves")
                .open(window_shown)
                .show(egui_ctx, |ui| {
                    egui::ScrollArea::from_max_height(f32::INFINITY).show(ui, |ui| {
                        for save in saves {
                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(&save.name)
                                        .text_style(egui::TextStyle::Heading)
                                        .strong(),
                                );

                                if ui
                                    .add(egui::ImageButton::new(
                                        TextureId::User(
                                            save.screenshot_texture
                                                .raw_miniquad_texture_handle()
                                                .gl_internal_id()
                                                as u64,
                                        ),
                                        &[NES_WIDTH as f32, NES_HEIGHT as f32],
                                    ))
                                    .clicked()
                                {
                                    match Self::load_zipped_save(&save.file) {
                                        Ok(n) => *nes = Some(n),
                                        Err(e) => report_error(&format!("{}", e)),
                                    }
                                };

                                ui.separator();
                            });
                        }
                    });
                });
        }
    }
}

const SCREENSHOT_PATH: &str = "screenshot.png";
const SAVESTATE_PATH: &str = "savestate.fnes";

pub struct Save {
    name: String,
    file: File,
    screenshot_texture: Texture2D,
}

impl Save {
    fn new(name: String, file: File, screnshot_texture: Texture2D) -> Self {
        Self {
            name,
            file,
            screenshot_texture: screnshot_texture,
        }
    }
}

pub enum SaveErr {
    DirOpenErr,
    DirContentsErr,
    FileLoadErr,
    InvalidSaveState,
    SaveErr,
}

impl From<ZipError> for SaveErr {
    fn from(_: ZipError) -> Self {
        SaveErr::FileLoadErr
    }
}

impl From<std::io::Error> for SaveErr {
    fn from(_: std::io::Error) -> Self {
        SaveErr::FileLoadErr
    }
}

impl From<NesError> for SaveErr {
    fn from(_: NesError) -> Self {
        SaveErr::InvalidSaveState
    }
}

impl Display for SaveErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveErr::FileLoadErr => write!(f, "Couldn't load the savefile"),
            SaveErr::InvalidSaveState => write!(f, "Savefile contained incompatible save version"),
            SaveErr::DirOpenErr => write!(f, "Couldn't open the save folder"),
            SaveErr::DirContentsErr => write!(f, "Couldn't process the saves"),
            SaveErr::SaveErr => write!(f, "Couldn't create the savefile"),
        }
    }
}
