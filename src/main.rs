use egui::Align2;
use macroquad::prelude::*;
use macroquad::window::next_frame;

use crate::input::InputDriver;
use crate::matrix::Matrix;

mod input;
mod matrix;

pub fn mouse() -> Vec2 {
    let (mouse_x, mouse_y) = mouse_position();
    Vec2 {
        x: mouse_x,
        y: mouse_y,
    }
}

#[macroquad::main("L3X IDE")]
async fn main() {
    let mut matrix = Matrix::default();

    const CELL_SIZE: f32 = 60.0;
    let mut offset = Vec2 { x: 100.0, y: 100.0 };

    let mut rmb_position = None;
    let mut input_driver = InputDriver::default();

    loop {
        clear_background(BLACK);

        input_driver.update();
        let logical = (mouse() - offset) / CELL_SIZE;
        if is_mouse_button_down(MouseButton::Left) {
            matrix.set_dims((logical + Vec2::splat(0.5)).as_ivec2())
        }

        // panning
        if is_mouse_button_released(MouseButton::Right) {
            rmb_position = None;
        }
        if is_mouse_button_down(MouseButton::Right) {
            if let Some((pos_x, pos_y)) = rmb_position {
                let (new_x, new_y) = mouse_position();
                let difference_x = new_x - pos_x;
                let difference_y = new_y - pos_y;

                offset.x += difference_x;
                offset.y += difference_y;
                rmb_position = Some(mouse_position());
            } else {
                rmb_position = Some(mouse_position());
            }
        }

        egui_macroquad::ui(|ctx| {
            egui::Window::new("Menu")
                .title_bar(false)
                .anchor(Align2::RIGHT_TOP, (-50.0, 50.0))
                .show(ctx, |ui| {
                    ui.label("Hello");
                });
        });

        matrix.draw(offset, CELL_SIZE, 1.0);
        egui_macroquad::draw();

        next_frame().await
    }
}
