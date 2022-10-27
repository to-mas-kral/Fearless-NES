use std::ops::Add;

use egui::{Id, Pos2, Rect, Stroke};
use egui_glium::egui_winit::egui::{self, epaint::PathShape};

const TRIANGLE_SIZE: f32 = 40.0;

///draws a paused overlay over the given `Rect`
pub fn pause_overlay(egui_ctx: &egui::Context, rect: Rect, paused: bool) {
    let painter = egui_ctx.layer_painter(egui::LayerId {
        order: egui::Order::Middle,
        id: Id::new("pause_overlay"),
    });
    let mut bg = egui_ctx.style().noninteractive().bg_fill;
    let trans = egui_ctx.animate_bool(Id::new("transition"), paused);

    //transparent overlay
    bg[3] = (150.0 * trans) as u8;
    let (color, contrast_bg) = if egui_ctx.style().visuals.dark_mode {
        (bg, egui::Visuals::light().noninteractive().bg_fill)
    } else {
        (
            bg.linear_multiply(0.1),
            egui::Visuals::dark().noninteractive().bg_fill,
        )
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
    bg[3] = (255.0 * trans) as u8;
    triangle.fill = bg;
    triangle.stroke = Stroke::new(5.0, contrast_bg);
    painter.add(triangle);
}
