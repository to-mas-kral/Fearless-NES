use std::ops::Add;

use egui::{Color32, Id, Pos2, Rect, Stroke};
use egui_glium::egui_winit::egui::{self, epaint::PathShape};

const TRIANGLE_SIZE: f32 = 40.0;

///draws a paused overlay over the given `Rect`
pub fn pause_overlay(egui_ctx: &egui::Context, rect: Rect, paused: bool) {
    let painter = egui_ctx.layer_painter(egui::LayerId {
        order: egui::Order::Middle,
        id: Id::new("pause_overlay"),
    });

    let trans = egui_ctx.animate_bool(Id::new("transition"), paused);
    let bg = egui_ctx.style().noninteractive().bg_fill;
    let color = if egui_ctx.style().visuals.dark_mode {
        Color32::from_rgba_premultiplied(bg.r(), bg.g(), bg.b(), (150.0 * trans) as u8)
    } else {
        Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), (50.0 * trans) as u8)
    };
    painter.rect_filled(rect, 0.5, color);

    //the play symbol
    let mut triangle = PathShape::closed_line(
        vec![
            rect.center()
                .add(Pos2::new(-TRIANGLE_SIZE / 1.6, -TRIANGLE_SIZE).to_vec2()),
            rect.center()
                .add(Pos2::new(-TRIANGLE_SIZE / 1.6, TRIANGLE_SIZE).to_vec2()),
            rect.center()
                .add(Pos2::new(TRIANGLE_SIZE / 1.6, 0.0).to_vec2()),
        ],
        Stroke::none(),
    );
    triangle.fill = if egui_ctx.style().visuals.dark_mode {
        Color32::from_rgba_premultiplied(bg.r(), bg.g(), bg.b(), (255.0 * trans) as u8)
    } else {
        Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), (255.0 * trans) as u8)
    };
    painter.add(triangle);
}
