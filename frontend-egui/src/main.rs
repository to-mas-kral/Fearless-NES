#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use gilrs::Gilrs;
use glium::glutin;
use glutin::event::WindowEvent;
use std::{env, path::PathBuf};

mod app;
mod dialog;
mod nesthread;

use app::{App, Config};
use dialog::DialogReport;

fn main() {
    let config = app::Config::new();

    let event_loop = glutin::event_loop::EventLoop::new();
    let display = create_display(&event_loop, &config);

    let mut egui_glium = egui_glium::EguiGlium::new(&display, &event_loop);

    let mut gilrs = Gilrs::new()
        .report_dialog_msg("Error while initializing gamepad input library")
        .ok();

    let mut app = App::new(config).unwrap();
    egui_glium.run(&display, |egui_ctx| {
        app.init_egui_style(egui_ctx);
    });

    if let Some(p) = env::args().nth(1) {
        app.create_nes_with_file(PathBuf::from(p)).ok();
    }

    event_loop.run(move |event, _, control_flow| {
        if let Some(gilrs) = gilrs.as_mut() {
            app.handle_gamepad_input(gilrs);
        }

        let mut redraw = || {
            egui_glium.run(&display, |egui_ctx| {
                app.draw_gui(egui_ctx);
            });

            display.gl_window().window().request_redraw();

            {
                let mut target = display.draw();
                egui_glium.paint(&display, &mut target);
                target.finish().unwrap();
            }
        };

        display.gl_window().window().request_redraw();

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    on_exit(&display, &mut app);
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

fn on_exit(display: &glium::Display, app: &mut App) {
    let size = display.gl_window().window().inner_size();

    app.config.window_height = size.height;
    app.config.window_width = size.width;
    app.config
        .save()
        .report_dialog_msg("Error while saving the config file")
        .unwrap();
}

fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
    config: &Config,
) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::PhysicalSize {
            width: config.window_width,
            height: config.window_height,
        })
        .with_title("Fearless-NES");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}
