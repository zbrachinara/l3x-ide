#![warn(clippy::all)]

use egui::Align2;
use macroquad::prelude::*;
use macroquad::window::next_frame;
use sound::chord::Chord;
use sound::signal::Updater;
use wasync::AsyncContext;

use crate::input::InputDriver;
use crate::matrix::Matrix;

mod input;
mod l3x;
mod matrix;
mod polygon;
mod registers;
mod sound;
mod swapbuffer;
mod traveler;
#[cfg(target_arch = "wasm32")]
mod wasm_log;
mod wasync;

const CELL_SIZE: f32 = 60.0;
const SCALE_RATE: f32 = 0.02;

struct Model<'a> {
    matrix: Matrix,
    offset: Vec2,
    scale: f32,
    input_driver: InputDriver,
    sound_handle: Updater,
    sound_needs_killing: bool,
    ctx: AsyncContext<'a>,
}

impl<'a> Default for Model<'a> {
    fn default() -> Self {
        Self {
            matrix: Default::default(),
            input_driver: Default::default(),
            offset: Vec2::splat(100.0),
            scale: 1.0,
            sound_needs_killing: false,
            sound_handle: Updater::default(),
            ctx: Default::default(),
        }
    }
}

#[macroquad::main("L3X IDE")]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = simple_logger::SimpleLogger::default()
            .with_level(log::LevelFilter::Debug)
            .init()
        {
            println!("Failed to init logging, you are on your own");
            println!("simple-logger failed with error {e}")
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        #[allow(clippy::redundant_pattern_matching)]
        if let Err(_) = wasm_log::WasmLogger::default()
            .with_level(log::LevelFilter::Debug)
            .init()
        {
            // TODO not sure if it's possible to communicate to the user here, maybe use an alert or something?
        }
    }

    log::debug!("If you see this message, logging is enabled (Debug level)");

    // #[cfg(not(target_arch = "wasm32"))]
    // let mut executor = async_executor::LocalExecutor::default();

    let mut state = Model::default();

    loop {
        clear_background(BEIGE);

        let mut egui_hovered = false;
        egui_macroquad::ui(|ctx| {
            egui_hovered = ctx.is_pointer_over_area();
            state.input_driver.update(ctx);
            egui::Window::new("Menu")
                .title_bar(false)
                .anchor(Align2::RIGHT_TOP, (-50.0, 50.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        state.ctx.tick();
                        state.matrix.config_ui(ui, &mut state.ctx);
                    })
                });
        });

        let logical =
            (Vec2::from(mouse_position()) - state.offset * state.scale) / (CELL_SIZE * state.scale);

        // panning
        if is_mouse_button_down(MouseButton::Right) {
            state.offset += state.input_driver.mouse_delta();
        }

        if state.input_driver.lmb().double_clicked() {
            state.matrix.edit(logical.as_ivec2());
        }
        if is_key_pressed(KeyCode::Escape) {
            state.matrix.stop_edit();
        }

        if let Some(chord) = state.matrix.update_sound(logical) {
            state.sound_needs_killing = true;
            state.sound_handle.update(chord).unwrap();
        } else if state.sound_needs_killing {
            state.sound_needs_killing = false;
            state.sound_handle.update(Chord::default()).unwrap()
        }

        if state.input_driver.lmb().held().is_some() {
            state
                .matrix
                .set_dims((logical + Vec2::splat(0.5)).as_ivec2())
        }

        if !egui_hovered {
            state.scale += mouse_wheel().1 * SCALE_RATE;
        }

        state.matrix.update(&mut state.ctx);

        state.matrix.draw(state.offset, CELL_SIZE, state.scale);
        egui_macroquad::draw();
        next_frame().await
    }
}
