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

const CELL_SIZE: f32 = 60.0;
const SCALE_RATE: f32 = 0.02;

struct Model<'a> {
    matrix: Matrix<'a>,
    offset: Vec2,
    scale: f32,
    input_driver: InputDriver,
}

impl<'a> Default for Model<'a> {
    fn default() -> Self {
        Self {
            matrix: Default::default(),
            input_driver: Default::default(),
            offset: Vec2::splat(100.0),
            scale: 1.0,
        }
    }
}

#[macroquad::main("L3X IDE")]
async fn main() {
    if let Err(e) = simple_logger::SimpleLogger::default()
        .with_level(log::LevelFilter::Debug)
        .init()
    {
        println!("Failed to init logging, you are on your own");
        println!("simple-logger failed with error {e}")
    }

    let mut state = Model::default();

    loop {
        clear_background(BEIGE);

        state.input_driver.update();
        let logical =
            (Vec2::from(mouse_position()) - state.offset * state.scale) / (CELL_SIZE * state.scale);
        if state.input_driver.lmb_hold().is_some() {
            state.matrix.set_dims((logical + Vec2::splat(0.5)).as_ivec2())
        }

        // panning
        if is_mouse_button_down(MouseButton::Right) {
            state.offset += state.input_driver.mouse_delta();
        }

        if state.input_driver.lmb_doubleclick() {
            state.matrix.edit(logical.as_ivec2());
        }
        if is_key_pressed(KeyCode::Escape) {
            state.matrix.stop_edit();
        }

        state.scale += mouse_wheel().1 * SCALE_RATE;

        egui_macroquad::ui(|ctx| {
            egui::Window::new("Menu")
                .title_bar(false)
                .anchor(Align2::RIGHT_TOP, (-50.0, 50.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        state.matrix.config_ui(ui);
                    })
                });
        });

        state.matrix.update();

        state.matrix.draw(state.offset, CELL_SIZE, state.scale);
        egui_macroquad::draw();
        next_frame().await
    }
}
