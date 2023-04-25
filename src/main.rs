use egui::Align2;
use macroquad::prelude::*;
use macroquad::window::next_frame;

use crate::input::InputDriver;
use crate::matrix::Matrix;

mod input;
mod l3x;
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
    if simple_logger::SimpleLogger::default().init().is_err() {
        println!("Failed to init logging, you are on your own");
    }

    let mut matrix = Matrix::default();

    const CELL_SIZE: f32 = 60.0;
    const SCALE_RATE: f32 = 0.02;
    let mut offset = Vec2 { x: 100.0, y: 100.0 };
    let mut scale = 1.0;

    let mut rmb_position = None;
    let mut input_driver = InputDriver::default();

    loop {
        clear_background(BLACK);

        input_driver.update();
        let logical = (mouse() - offset) / (CELL_SIZE * scale);
        if input_driver.lmb_hold().is_some() {
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

        if input_driver.lmb_doubleclick() {
            matrix.edit(logical.as_ivec2());
        }
        if is_key_pressed(KeyCode::Escape) {
            matrix.stop_edit();
        }

        scale += mouse_wheel().1 * SCALE_RATE;

        egui_macroquad::ui(|ctx| {
            egui::Window::new("Menu")
                .title_bar(false)
                .anchor(Align2::RIGHT_TOP, (-50.0, 50.0))
                .show(ctx, |ui| {
                    matrix.config_ui(ui);
                });
        });

        matrix.draw(offset, CELL_SIZE, scale);
        egui_macroquad::draw();

        next_frame().await
    }
}
