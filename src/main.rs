use itertools::Itertools;
use macroquad::prelude::*;
use macroquad::window::next_frame;

#[macroquad::main("L3X IDE")]
async fn main() {
    let matrix_rows = 1;
    let matrix_cols = 1;

    const CELL_SIZE: usize = 40;
    const CELL_OFFSET_X: usize = 300;
    const CELL_OFFSET_Y: usize = 300;

    loop {
        if is_mouse_button_down(MouseButton::Left) {}

        for (i, j) in (0..matrix_rows).cartesian_product(0..matrix_cols) {
            draw_rectangle_lines(
                (i * CELL_SIZE + CELL_OFFSET_X) as f32,
                (j * CELL_SIZE + CELL_OFFSET_Y) as f32,
                CELL_SIZE as f32,
                CELL_SIZE as f32,
                2.0,
                WHITE,
            )
        }

        next_frame().await
    }
}
