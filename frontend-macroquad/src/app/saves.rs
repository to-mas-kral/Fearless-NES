use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use egui::{CtxRef, TextureId};
use fearless_nes::Nes;
use macroquad::prelude::{ImageFormat, Texture2D};
use zip::write::FileOptions;

use crate::{
    app::{report_error, App, Gui},
    NES_HEIGHT, NES_WIDTH,
};

use crate::app::{get_save_named_path, nesrender::NesRender};

use super::get_folder_path;

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

    fn gather_saves(&mut self) -> Result<()> {
        self.saves.clear();

        let saves_paths: Vec<PathBuf> = fs::read_dir(&self.folder_path)
            .context("couldn't open save directory")?
            .filter_map(|e| e.ok())
            .filter(|de| de.path().extension() == Some(OsStr::new("fnes")))
            .map(|de| de.path())
            .collect();

        for path in saves_paths {
            if let Err(_) = self.load_save(&path) {
                report_error(&format!("Could't load the save file:\n{:?}", path));
            }
        }

        Ok(())
    }

    fn load_save(&mut self, save_path: &Path) -> Result<()> {
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
                .ok_or(anyhow!(""))?
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
    ) -> Result<()> {
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
                let mut writer = encoder.write_header()?;

                // TODO: get_image_data() - create PR in Macroquad...
                let image_data: Vec<u8> = nesrender
                    .image
                    .get_image_data()
                    .iter()
                    .flat_map(|e| *e)
                    .collect();

                writer.write_image_data(&image_data)?;
            };

            let file = std::fs::File::create(&path)?;
            let mut zip = zip::ZipWriter::new(file);

            let options =
                FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("screenshot.png", options)?;
            zip.write_all(&screenshot)?;

            zip.start_file("savestate.fnes", options)?;
            zip.write_all(&save_data)?;

            zip.finish()?;
        };

        Ok(())
    }

    pub fn load_zipped_save(savefile: &File) -> Result<Nes> {
        let mut save_archive = zip::ZipArchive::new(savefile)?;
        let mut savestate_zip = save_archive.by_name(SAVESTATE_PATH)?;
        let mut savestate = Vec::with_capacity(savestate_zip.size() as usize);
        savestate_zip.read_to_end(&mut savestate)?;

        Ok(Nes::load_state(&savestate)?)
    }

    /// GUI: adds a new save imagebutton to the saves window
    pub fn push_save_view(
        nes: &mut Option<Nes>,
        save: &Save,
        ui: &mut egui::Ui,
    ) -> egui::InnerResponse<()> {
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
                            .gl_internal_id() as u64,
                    ),
                    &[NES_WIDTH as f32, NES_HEIGHT as f32],
                ))
                .clicked()
            {
                match Self::load_zipped_save(&save.file) {
                    Ok(n) => *nes = Some(n),
                    Err(e) => report_error(&format!("Couldn't load the save file. Error: {}", e)),
                }
            };

            ui.separator();
        })
    }
}

impl Gui for Saves {
    fn gui_window(app: &mut App, egui_ctx: &CtxRef) {
        if app.saves.window_shown {
            if app.saves.folder_changed {
                app.saves.folder_changed = false;

                if let Err(e) = app.saves.gather_saves() {
                    report_error(&format!("{}", e))
                }
            }

            let saves = &mut app.saves.saves;
            let window_shown = &mut app.saves.window_shown;
            let nes = &mut app.nes;

            egui::Window::new("Saves")
                .open(window_shown)
                .show(egui_ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .show(ui, |ui| {
                            for save in saves {
                                Saves::push_save_view(nes, save, ui);
                            }
                        });
                });
        }
    }

    fn gui_embed(app: &mut App, ui: &mut egui::Ui) {
        if app.nes.is_some() && ui.button("Save").clicked() {
            if let Err(e) =
                app.saves
                    .create_save(&app.nes, &mut app.render, &app.config.save_folder_path)
            {
                report_error(&format!("Couldn't create the save file. Error: {}", e));
            }
        }

        if ui.button("Load saves from folder").clicked() {
            match get_folder_path(Some(app.config.save_folder_path.as_ref())) {
                Ok(Some(p)) => {
                    app.saves.window_shown = true;
                    app.saves.folder_path = p.clone();
                    app.saves.folder_changed = true;

                    app.config.save_folder_path = p;
                }
                Ok(None) => (),
                Err(_) => return,
            }
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
