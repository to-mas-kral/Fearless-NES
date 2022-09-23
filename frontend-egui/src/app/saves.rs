use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use egui_glium::egui_winit::egui::{self, Color32, ColorImage, RichText, TextureHandle};
use eyre::{eyre, Result, WrapErr};
use fearless_nes::{Nes, NES_HEIGHT, NES_WIDTH};
use zip::write::FileOptions;

use super::RuntimeNes;
use crate::app::{get_save_named_path, nesrender::NesRender};
use crate::{app::App, dialog::DialogReport};

pub struct Saves {
    pub saves: Vec<Save>,

    pub window_shown: bool,
    pub folder_path: PathBuf,
}

impl Saves {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "Fearless-NES", "Fearless-NES")
            .ok_or(eyre!("Couldn't locate project-dirs"))?;

        let folder_path = proj_dirs.data_dir().to_path_buf();
        fs::create_dir_all(&folder_path)?;

        Ok(Self {
            saves: Vec::new(),

            window_shown: false,
            folder_path,
        })
    }

    fn gather_saves(&mut self, egui_ctx: &egui::Context) -> Result<()> {
        self.saves.clear();

        let saves_paths: Vec<PathBuf> = fs::read_dir(&self.folder_path)
            .wrap_err("couldn't open save directory")?
            .filter_map(|e| e.ok())
            .filter(|de| de.path().extension() == Some(OsStr::new("fnes")))
            .map(|de| de.path())
            .collect();

        for path in saves_paths {
            self.load_save(&path, egui_ctx)
                .report_dialog_msg(&format!("Could't load the save file:\n{:?}", path))?;
        }

        Ok(())
    }

    fn load_save(&mut self, save_path: &Path, egui_ctx: &egui::Context) -> Result<()> {
        let save_file = std::fs::File::open(&save_path)?;
        let mut save_archive = zip::ZipArchive::new(&save_file)?;

        let screenshot = {
            let mut screenshot_zip = save_archive.by_name(SCREENSHOT_PATH)?;
            let decoder = png::Decoder::new(&mut screenshot_zip);
            let mut reader = decoder.read_info()?;

            let mut buf = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buf)?;
            buf.resize(info.buffer_size(), 0);

            buf
        };

        let save_name = save_path
            .file_stem()
            .ok_or(eyre!(""))?
            .to_string_lossy()
            .into_owned();

        let mut img = ColorImage::new([NES_WIDTH, NES_HEIGHT], Color32::BLACK);
        img.pixels = bytemuck::cast_slice(&screenshot).to_vec();

        let texture_handle = egui_ctx.load_texture(&save_name, img, egui::TextureFilter::Nearest);

        self.saves
            .push(Save::new(save_name, save_file, texture_handle));

        Ok(())
    }

    pub fn create_save(&mut self, nes: &RuntimeNes, nesrender: &mut NesRender) -> Result<()> {
        let path = match get_save_named_path(Some(&self.folder_path), "Fearless-NES save", "fnes") {
            Some(p) => p,
            None => return Ok(()),
        };

        if let Some(nes) = nes {
            let nes = nes.lock().unwrap();
            let save_data = nes.save_state()?;

            let mut screenshot: Vec<u8> = Vec::with_capacity(NES_WIDTH * NES_HEIGHT);
            {
                let mut encoder =
                    png::Encoder::new(&mut screenshot, NES_WIDTH as u32, NES_HEIGHT as u32);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header()?;

                writer.write_image_data(bytemuck::cast_slice(nesrender.image.pixels.as_slice()))?;
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

    /// GUI: add a new save image button to the saves window
    pub fn push_save_view(save: &Save, ui: &mut egui::Ui) -> Option<Nes> {
        let mut loaded_save = None;

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(&save.name).heading().strong());

            if ui
                .add(egui::ImageButton::new(
                    &save.texture_handle,
                    &[NES_WIDTH as f32, NES_HEIGHT as f32],
                ))
                .clicked()
            {
                if let Ok(n) = Self::load_zipped_save(&save.file)
                    .report_dialog_with(|e| format!("Couldn't load the save file. Error: {}", e))
                {
                    loaded_save = Some(n);
                }
            };

            ui.separator();
        });

        loaded_save
    }
}

impl Saves {
    pub fn gui_window(app: &mut App, egui_ctx: &egui::Context) {
        if app.saves.window_shown {
            let saves = &mut app.saves.saves;
            let window_shown = &mut app.saves.window_shown;

            let mut loaded_nes = None;

            egui::Window::new("Saves")
                .open(window_shown)
                .show(egui_ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .show(ui, |ui| {
                            for save in saves {
                                if let Some(n) = Saves::push_save_view(save, ui) {
                                    loaded_nes = Some(n);
                                }
                            }
                        });
                });

            if let Some(loaded_nes) = loaded_nes {
                app.replace_nes(loaded_nes);
            }
        }
    }

    pub fn gui_embed(app: &mut App, ui: &mut egui::Ui) {
        if app.nes.is_some() && ui.button("Save").clicked() {
            app.saves
                .create_save(&app.nes, &mut app.render)
                .report_dialog_with(|e| format!("Couldn't create the save file. Error: {}", e))
                .ok();
        }

        if ui.button("Load saves from folder").clicked() {
            app.saves.gather_saves(ui.ctx()).report_dialog().ok();
            app.saves.window_shown = true;
        }
    }
}

const SCREENSHOT_PATH: &str = "screenshot.png";
const SAVESTATE_PATH: &str = "savestate.fnes";

pub struct Save {
    name: String,
    file: File,
    texture_handle: TextureHandle,
}

impl Save {
    fn new(name: String, file: File, texture_handle: TextureHandle) -> Self {
        Self {
            name,
            file,
            texture_handle,
        }
    }
}
