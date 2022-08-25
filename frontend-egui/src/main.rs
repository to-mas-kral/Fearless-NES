#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui_glium::egui_winit::egui;
use eyre::Result;
use fearless_nes::Nes;
use gilrs::Gilrs;
use glium::{glutin, Surface};
use std::{env, fs, path::PathBuf};

mod app;
mod dialog;

use app::App;
use dialog::DialogReport;

const NES_WIDTH: usize = 256;
const NES_HEIGHT: usize = 240;

fn create_nes(rom_path: PathBuf) -> Result<Nes> {
    let rom = fs::read(rom_path).report_dialog_msg("Error while reading the ROM file")?;
    Ok(Nes::new(&rom).report_dialog_with(|e| format!("Error while loading the ROM: {:?}", e))?)
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let display = create_display(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(&display, &event_loop);

    let mut gilrs = Gilrs::new()
        .report_dialog_msg("Error while initializing gamepad input library")
        .ok();

    let config = app::Config::new();

    let mut app = App::new(config);
    egui_glium.run(&display, |egui_ctx| {
        app.init_egui_style(&egui_ctx);
    });

    if let Some(p) = env::args().nth(1) {
        app.nes = create_nes(PathBuf::from(p)).ok()
    }

    event_loop.run(move |event, _, control_flow| {
        if let Some(gilrs) = gilrs.as_mut() {
            app.handle_gamepad_input(gilrs);
        }

        let mut redraw = || {
            app.run_nes_frame();

            let should_repaint = egui_glium.run(&display, |egui_ctx| {
                app.draw_gui(egui_ctx);
                app.draw_nes(egui_ctx);
            });

            //if should_repaint {
            display.gl_window().window().request_redraw();
            //}

            {
                let mut target = display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

                egui_glium.paint(&display, &mut target);

                target.finish().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    app.config
                        .save()
                        .report_dialog_msg("Error while saving the config file")
                        .unwrap();

                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                egui_glium.on_event(&event);

                match event {
                    WindowEvent::KeyboardInput { input, .. } => app.handle_keyboard_input(input),
                    // TODO(rewrite): mouse auto-hiding
                    /* WindowEvent::CursorMoved { .. } => app.handle_mouse_input(&mut display),
                    WindowEvent::MouseInput { .. } => app.handle_mouse_input(&mut display), */
                    // TODO: use winit gampead support or gilrs ?
                    //WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
                    _ => (),
                };

                display.gl_window().window().request_redraw();
            }
            glutin::event::Event::NewEvents(glutin::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    });
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 1280.0,
            height: 720.0,
        })
        .with_title("Fearless-NES");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}
