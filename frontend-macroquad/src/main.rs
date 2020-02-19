use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use egui::{Label, TextureId};
use gilrs::{Axis, Button as GButton, EventType, Gilrs};
use macroquad::prelude::*;
use native_dialog::FileDialog;

use fearless_nes::{Button as NesButton, Nes, PALETTE};
use zip::write::FileOptions;

const NES_WIDTH: usize = 256;
const NES_HEIGHT: usize = 240;

struct App {
    nes: Option<Nes>,

    render: NesRender,

    paused: bool,
    saves: Saves,
    debug: Debug,
}

impl App {
    pub fn new() -> Self {
        Self {
            nes: None,
            paused: false,

            render: NesRender::new(),
            saves: Saves::new(),
            debug: Debug::new(),
        }
    }
}

struct Saves {
    saves: Vec<Save>,

    window_shown: bool,
    folder_path: PathBuf,
    folder_changed: bool,
}

impl Saves {
    fn new() -> Self {
        Self {
            saves: Vec::new(),

            window_shown: false,
            folder_path: PathBuf::new(),
            folder_changed: false,
        }
    }
}

const SCREENSHOT_PATH: &str = "screenshot.png";
const SAVESTATE_PATH: &str = "savestate.fnes";

struct Save {
    name: String,
    file: File,
    screnshot_texture: Texture2D,
}

impl Save {
    fn new(name: String, file: File, screnshot_texture: Texture2D) -> Self {
        Self {
            name,
            file,
            screnshot_texture,
        }
    }
}

struct Debug {
    show_controls: bool,
    show_cpu: bool,
    show_ines_header: bool,
}

impl Debug {
    fn new() -> Self {
        Self {
            show_controls: false,
            show_cpu: false,
            show_ines_header: false,
        }
    }
}

struct NesRender {
    image: Image,
    texture: Texture2D,
    scale: f32,
    draw_pos: f32,
}

impl NesRender {
    fn new() -> Self {
        let image = Image::gen_image_color(NES_WIDTH as u16, NES_HEIGHT as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        let (scale, draw_pos) = {
            let x_scale = screen_width() as usize / NES_WIDTH;
            let y_scale = screen_height() as usize / NES_HEIGHT;
            let scale = x_scale.min(y_scale) as f32;

            let draw_pos = (screen_width() - NES_WIDTH as f32 * scale) / 2.0;

            (scale, draw_pos)
        };

        Self {
            image,
            texture,
            scale,
            draw_pos,
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Fearless-NES".to_owned(),
        fullscreen: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
    if let Some(p) = env::args().skip(1).next() {
        app.nes = create_nes(PathBuf::from(p)).ok()
    }

    let mut gilrs = match Gilrs::new() {
        Ok(g) => Some(g),
        Err(_) => {
            // TODO: implement my own logging (with egui ?)
            eprintln!("Couldn't initialize gamepad input library");
            None
        }
    };

    loop {
        handle_input(&mut app, &mut gilrs);

        if let Some(nes) = &mut app.nes {
            if !app.paused {
                nes.run_one_frame();
            }

            update_frame(nes.get_frame_buffer(), &mut app.render.image);
        }

        handle_gui(&mut app);

        // Draw things before egui

        draw_nes(&mut app);

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}

#[rustfmt::skip]
fn handle_input(app: &mut App, gilrs: &mut Option<Gilrs>) {
    if app.nes.is_none() {
        return;
    }

    let nes = app.nes.as_mut().unwrap();

    // TODO: hopefully macroquad gets gamepad support
    if let Some(gilrs) = gilrs {
        while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(button, ..) => match button {
                    GButton::East => nes.set_button_state(NesButton::A, true),
                    GButton::West => nes.set_button_state(NesButton::B, true),
                    GButton::Start => nes.set_button_state(NesButton::Start, true),
                    GButton::Select => nes.set_button_state(NesButton::Select, true),
                    GButton::DPadUp => nes.set_button_state(NesButton::Up, true),
                    GButton::DPadRight => nes.set_button_state(NesButton::Right, true),
                    GButton::DPadDown => nes.set_button_state(NesButton::Down, true),
                    GButton::DPadLeft => nes.set_button_state(NesButton::Left, true),
                    GButton::Mode => {
                        app.paused = !app.paused;
                    }
                    _ => (),
                },
                EventType::ButtonReleased(button, ..) => match button {
                    GButton::East => nes.set_button_state(NesButton::A, false),
                    GButton::West => nes.set_button_state(NesButton::B, false),
                    GButton::Start => nes.set_button_state(NesButton::Start, false),
                    GButton::Select => nes.set_button_state(NesButton::Select, false),
                    GButton::DPadUp => nes.set_button_state(NesButton::Up, false),
                    GButton::DPadRight => nes.set_button_state(NesButton::Right, false),
                    GButton::DPadDown => nes.set_button_state(NesButton::Down, false),
                    GButton::DPadLeft => nes.set_button_state(NesButton::Left, false),
                    _ => (),
                },
                EventType::AxisChanged(Axis::LeftStickX, val, ..) => {
                    if val > 0.5 {
                        nes.set_button_state(NesButton::Right, true);
                    } else if val < -0.5 {
                        nes.set_button_state(NesButton::Left, true);
                    } else {
                        nes.set_button_state(NesButton::Right, false);
                        nes.set_button_state(NesButton::Left, false);
                    }
                }
                EventType::AxisChanged(Axis::LeftStickY, val, ..) => {
                    if val > 0.5 {
                        nes.set_button_state(NesButton::Up, true);
                    } else if val < -0.5 {
                        nes.set_button_state(NesButton::Down, true);
                    } else {
                        nes.set_button_state(NesButton::Up, false);
                        nes.set_button_state(NesButton::Down, false);
                    }
                }
                _ => (),
            }
        }
    }

    if is_key_pressed(KeyCode::Up) { nes.set_button_state(NesButton::Up, true) }
    if is_key_pressed(KeyCode::Right) { nes.set_button_state(NesButton::Right, true) }
    if is_key_pressed(KeyCode::Down) { nes.set_button_state(NesButton::Down, true) }
    if is_key_pressed(KeyCode::Left) { nes.set_button_state(NesButton::Left, true) }
    if is_key_pressed(KeyCode::Enter) { nes.set_button_state(NesButton::Start, true) }
    if is_key_pressed(KeyCode::Space) { nes.set_button_state(NesButton::Select, true) }
    if is_key_pressed(KeyCode::D) { nes.set_button_state(NesButton::B, true) }
    if is_key_pressed(KeyCode::F) { nes.set_button_state(NesButton::A, true) }

    if is_key_released(KeyCode::Up) { nes.set_button_state(NesButton::Up, false) }
    if is_key_released(KeyCode::Right) { nes.set_button_state(NesButton::Right, false) }
    if is_key_released(KeyCode::Down) { nes.set_button_state(NesButton::Down, false) }
    if is_key_released(KeyCode::Left) { nes.set_button_state(NesButton::Left, false) }
    if is_key_released(KeyCode::Enter) { nes.set_button_state(NesButton::Start, false) }
    if is_key_released(KeyCode::Space) { nes.set_button_state(NesButton::Select, false) }
    if is_key_released(KeyCode::D) { nes.set_button_state(NesButton::B, false) }
    if is_key_released(KeyCode::F) { nes.set_button_state(NesButton::A, false) }
}

// TODO: refactor into nicer functions
fn handle_gui(app: &mut App) {
    clear_background(BLACK);
    egui_macroquad::ui(|egui_ctx| {
        egui::Window::new("Fearless-NES")
            .resizable(false)
            .anchor(egui::Align2::LEFT_TOP, [0., 0.])
            .show(egui_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    if ui.button("Load ROM").clicked() {
                        loop {
                            let rom_path = match get_rom_path() {
                                Ok(Some(p)) => p,
                                Ok(None) => break,
                                Err(_) => return,
                            };

                            match create_nes(rom_path) {
                                Ok(n) => {
                                    app.nes = Some(n);
                                    break;
                                }
                                Err(_) => continue,
                            }
                        }
                    }

                    egui::menu::menu(ui, "Controls", |ui| {
                        let button_text = match app.paused {
                            true => "Resume",
                            false => "Pause",
                        };

                        if ui.button(button_text).clicked() {
                            app.paused = !app.paused;
                        }

                        if ui.button("Reset").clicked() {
                            // TODO: resetting
                        }
                    });

                    egui::menu::menu(ui, "Saves", |ui| {
                        if ui.button("Save").clicked() {
                            create_save(app);
                        }

                        if ui.button("Load saves from folder").clicked() {
                            match get_save_folder() {
                                Ok(Some(p)) => {
                                    app.saves.window_shown = true;
                                    app.saves.folder_path = p;
                                    app.saves.folder_changed = true;
                                }
                                Ok(None) => (),
                                Err(_) => return,
                            }
                        }
                    });

                    egui::menu::menu(ui, "Debugger", |ui| {
                        if ui.button("Controls and Breakpoints").clicked() {
                            app.debug.show_controls = true;
                        }

                        if ui.button("CPU State").clicked() {
                            app.debug.show_cpu = true;
                        }

                        if ui.button("iNES Header").clicked() {
                            app.debug.show_ines_header = true;
                        }
                    });
                });
            });

        if app.saves.window_shown {
            if app.saves.folder_changed {
                app.saves.folder_changed = false;
                app.saves.saves = build_saves(app);
            }

            let nes = &mut app.nes;
            let saves = &app.saves;

            egui::Window::new("Saves").show(egui_ctx, |ui| {
                egui::ScrollArea::from_max_height(f32::INFINITY).show(ui, |ui| {
                    for save in &saves.saves {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Label::new(&save.name)
                                    .text_style(egui::TextStyle::Heading)
                                    .strong(),
                            );

                            if ui
                                .add(egui::ImageButton::new(
                                    TextureId::User(
                                        save.screnshot_texture
                                            .raw_miniquad_texture_handle()
                                            .gl_internal_id()
                                            as u64,
                                    ),
                                    &[NES_WIDTH as f32, NES_HEIGHT as f32],
                                ))
                                .clicked()
                            {
                                let mut save_archive =
                                    zip::ZipArchive::new(&save.file).unwrap();
                                let mut savestate_zip =
                                    save_archive.by_name(SAVESTATE_PATH).unwrap();
                                let mut savestate =
                                    Vec::with_capacity(savestate_zip.size() as usize);
                                savestate_zip.read_to_end(&mut savestate).unwrap();

                                if let Some(nes) = nes {
                                    nes.load_state(&savestate).unwrap();
                                }
                            };

                            ui.separator();
                        });
                    }
                });
            });
        }

        if let Some(ref mut nes) = app.nes {
            if app.debug.show_cpu {
                egui::Window::new("CPU State")
                    .open(&mut app.debug.show_cpu)
                    .resizable(false)
                    .default_width(0.)
                    .show(egui_ctx, |ui| {
                        let cpu = nes.get_cpu_state();

                        ui.columns(2, |columns| {
                            columns[0].group(|ui| {
                                egui::Grid::new("Registers")
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new("Registers")
                                                .heading()
                                                .strong(),
                                        );
                                        ui.add(
                                            egui::Label::new("Value").heading().strong(),
                                        );
                                        ui.end_row();

                                        ui.label("A").on_hover_text("accumulator");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.a))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("X").on_hover_text("X index");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.x))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("Y").on_hover_text("Y index");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.y))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("PC").on_hover_text("program counter");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.pc))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();

                                        ui.label("SP").on_hover_text("stack pointer");
                                        ui.add(
                                            egui::Label::new(format!("{:#x}", cpu.sp))
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                        ui.end_row();
                                    });
                            });

                            columns[1].group(|ui| {
                                egui::Grid::new("Flags").striped(true).show(ui, |ui| {
                                    ui.add(egui::Label::new("Flag").heading().strong());
                                    ui.add(egui::Label::new("Value").heading().strong());
                                    ui.end_row();

                                    ui.label("N").on_hover_text("negative");
                                    ui.checkbox(&mut cpu.n, "");
                                    ui.end_row();

                                    ui.label("V").on_hover_text("overflow");
                                    ui.checkbox(&mut cpu.v, "");
                                    ui.end_row();

                                    ui.label("I").on_hover_text("irq inhibit");
                                    ui.checkbox(&mut cpu.i, "");
                                    ui.end_row();

                                    ui.label("Z").on_hover_text("zero");
                                    ui.checkbox(&mut cpu.z, "");
                                    ui.end_row();

                                    ui.label("C").on_hover_text("carry");
                                    ui.checkbox(&mut cpu.c, "");
                                    ui.end_row();
                                });
                            });
                        });
                    });
            }
        }

        let paused = &mut app.paused;

        if let Some(ref mut nes) = app.nes {
            if app.debug.show_controls {
                egui::Window::new("Controls and Breakpoints")
                    .open(&mut app.debug.show_controls)
                    .resizable(false)
                    .default_width(0.)
                    .show(egui_ctx, |ui| {
                        ui.checkbox(paused, "Paused");
                        if ui.button("Step frame").is_pointer_button_down_on() {
                            nes.run_one_frame();
                        }

                        if ui.button("Step CPU cycle").is_pointer_button_down_on() {
                            nes.run_cpu_cycle();
                        }
                    });
            }
        };

        match (&mut app.nes, app.debug.show_ines_header) {
            (Some(nes), true) => {
                egui::Window::new("iNes Header")
                    .open(&mut app.debug.show_ines_header)
                    .resizable(false)
                    .default_width(0.)
                    .show(egui_ctx, |ui| {
                        let header = nes.get_ines_header();

                        egui::Grid::new("iNes Header Grid")
                            .striped(true)
                            .spacing([20., 5.])
                            .show(ui, |ui| {
                                ui.add(Label::new("Field").heading().strong());
                                ui.add(Label::new("Value").heading().strong());
                                ui.end_row();

                                ui.label("Mapper");
                                ui.add(
                                    Label::new(format!("{}", header.mapper_id))
                                        .monospace(),
                                );
                                ui.end_row();

                                ui.label("PRG ROM count");
                                ui.add(
                                    Label::new(format!(
                                        "{} * 16KB",
                                        header.prg_rom_count
                                    ))
                                    .monospace(),
                                );
                                ui.end_row();

                                ui.label("CHR ROM count");
                                ui.add(
                                    Label::new(format!("{} * 8KB", header.chr_rom_count))
                                        .monospace(),
                                )
                                .on_hover_text(
                                    "0 CHR ROM count means that the board uses CHR RAM",
                                );
                                ui.end_row();

                                ui.label("PRG RAM count");
                                ui.add(
                                    Label::new(format!("{} * 8KB", header.prg_ram_count))
                                        .monospace(),
                                );
                                ui.end_row();

                                ui.label("Mirroring");
                                ui.add(Label::new(format!("{:?}", header.mirroring)));
                                ui.end_row();

                                ui.label("Has Battery");
                                ui.add(Label::new(format!("{}", header.has_battery)));
                                ui.end_row();
                            });
                    });
            }
            _ => (),
        }
    });
}

fn build_saves(app: &mut App) -> Vec<Save> {
    let mut saves = Vec::new();

    let saves_paths: Vec<PathBuf> = fs::read_dir(&app.saves.folder_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|de| de.path().extension() == Some(OsStr::new("fnes")))
        .map(|de| de.path())
        .collect();

    for path in saves_paths {
        // TODO: error handling...
        let save_file = std::fs::File::open(&path).unwrap();
        let mut save_archive = zip::ZipArchive::new(&save_file).unwrap();

        let screenshot = {
            let mut screenshot_zip = save_archive.by_name(SCREENSHOT_PATH).unwrap();
            let mut screenshot = Vec::with_capacity(screenshot_zip.size() as usize);
            screenshot_zip.read_to_end(&mut screenshot).unwrap();

            screenshot
        };

        let texture =
            Texture2D::from_file_with_format(&screenshot, Some(ImageFormat::Png));

        saves.push(Save::new(
            path.file_stem().unwrap().to_string_lossy().into_owned(),
            save_file,
            texture,
        ));
    }

    saves
}

fn create_save(app: &App) {
    let path = match FileDialog::new()
        .set_location("~/")
        .add_filter("Fearless-NES save", &["fnes"])
        .show_save_single_file()
    {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(_) => {
            report_error("Error while getting the save path");
            return;
        }
    };

    if let Some(ref nes) = app.nes {
        let save_data = match nes.save_state() {
            Ok(s) => s,
            Err(e) => {
                report_error(&format!("Couldnt create save file, error: {}", e));
                return;
            }
        };

        // TODO Error checking

        // TODO: check typical size
        let mut screenshot: Vec<u8> = Vec::with_capacity(NES_WIDTH * NES_HEIGHT);
        {
            let mut encoder =
                png::Encoder::new(&mut screenshot, NES_WIDTH as u32, NES_HEIGHT as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();

            // TODO: create issue on Macroquad Github...
            let image_data: Vec<u8> = app
                .render
                .image
                .get_image_data()
                .iter()
                .flat_map(|e| *e)
                .collect();
            writer.write_image_data(&image_data).unwrap();
        };

        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options =
            FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);
        zip.start_file("screenshot.png", options).unwrap();
        zip.write_all(&screenshot).unwrap();

        zip.start_file("savestate.fnes", options).unwrap();
        zip.write_all(&save_data).unwrap();

        zip.finish().unwrap();
    };
}

fn get_save_folder() -> Result<Option<PathBuf>, ()> {
    match FileDialog::new().set_location("~/").show_open_single_dir() {
        Ok(Some(p)) => return Ok(Some(p)),
        Ok(None) => return Ok(None),
        Err(_) => {
            report_error("Error while getting the ROM path, please specify the ROM path using the command line");
            return Err(());
        }
    }
}

fn draw_nes(app: &mut App) {
    // TODO: update nes_draw_pos when resizing the window
    let nes_render = &mut app.render;

    nes_render.texture.update(&nes_render.image);

    draw_texture_ex(
        nes_render.texture,
        nes_render.draw_pos,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                NES_WIDTH as f32 * nes_render.scale,
                NES_HEIGHT as f32 * nes_render.scale,
            )),
            ..Default::default()
        },
    );
}

fn update_frame(nes_framebuffer: &[u8], output: &mut Image) {
    for (i, pixel_color) in nes_framebuffer.iter().enumerate() {
        let palette_addr = (pixel_color * 3) as usize;

        let r = PALETTE[palette_addr];
        let g = PALETTE[palette_addr + 1];
        let b = PALETTE[palette_addr + 2];

        // TOOD: filling the image could be slow
        output.set_pixel(
            (i % NES_WIDTH) as u32,
            (i / NES_WIDTH) as u32,
            Color::from_rgba(r, g, b, u8::MAX),
        );
    }
}

fn create_nes(rom_path: PathBuf) -> Result<Nes, ()> {
    let rom = match fs::read(rom_path) {
        Ok(r) => r,
        Err(_) => {
            report_error("Error while reading from the ROM file");
            return Err(());
        }
    };

    match Nes::new(&rom) {
        Ok(n) => return Ok(n),
        Err(e) => {
            report_error(&format!("Error while loading the ROM: {:?}", e));
            return Err(());
        }
    };
}

fn get_rom_path() -> Result<Option<PathBuf>, ()> {
    loop {
        match FileDialog::new().set_location("~/").show_open_single_file() {
            Ok(Some(p)) => return Ok(Some(p)),
            Ok(None) => return Ok(None),
            Err(_) => {
                report_error("Error while getting the ROM path, please specify the ROM path using the command line");
                return Err(());
            }
        }
    }
}

fn report_error(msg: &str) {
    native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Info)
        .set_title("Error")
        .set_text(msg)
        .show_alert()
        .expect("Error while displaying an error message box (needs KDialog)");
}
