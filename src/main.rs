#![warn(clippy::all)]

use egui::Align2;
use macroquad::prelude::*;
use macroquad::window::next_frame;

use crate::input::InputDriver;
use crate::matrix::Matrix;

mod input;
mod l3x;
mod matrix;
mod swapbuffer;
mod traveler;

#[macroquad::main("L3X IDE")]
async fn main() {
    if let Err(e) = simple_logger::SimpleLogger::default()
        .with_level(log::LevelFilter::Debug)
        .init()
    {
        println!("Failed to init logging, you are on your own");
        println!("simple-logger failed with error {e}")
    }

    let mut matrix = Matrix::default();
    const CELL_SIZE: f32 = 60.0;
    const SCALE_RATE: f32 = 0.02;
    let mut offset = Vec2 { x: 100.0, y: 100.0 };
    let mut scale = 1.0;

    let mut input_driver = InputDriver::default();
    loop {
        clear_background(BEIGE);

        input_driver.update();
        let logical = (Vec2::from(mouse_position()) - offset * scale) / (CELL_SIZE * scale);
        if input_driver.lmb_hold().is_some() {
            matrix.set_dims((logical + Vec2::splat(0.5)).as_ivec2())
        }

        // panning
        if is_mouse_button_down(MouseButton::Right) {
            offset += input_driver.mouse_delta();
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
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        matrix.config_ui(ui);
                    })
                });
        });

        matrix.update();

        matrix.draw(offset, CELL_SIZE, scale);
        egui_macroquad::draw();
        next_frame().await
    }
}
