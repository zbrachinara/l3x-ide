use itertools::Itertools;
use macroquad::prelude::*;
use macroquad::window::next_frame;

#[macroquad::main("L3X IDE")]
async fn main() {
    let mut matrix_rows = 1;
    let mut matrix_cols = 1;

    const CELL_SIZE: usize = 40;
    let mut offset_x = 300.0;
    let mut offset_y = 300.0;

    let mut rmb_position = None;

    loop {
        if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let logical_x = (mouse_x - offset_x) / CELL_SIZE as f32 + 0.5;
            let logical_y = (mouse_y - offset_y) / CELL_SIZE as f32 + 0.5;

            if logical_x > 0.0 && logical_y > 0.0 {
                matrix_rows = std::cmp::max(logical_x as usize, 1);
                matrix_cols = std::cmp::max(logical_y as usize, 1);
            }
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

                offset_x += difference_x;
                offset_y += difference_y;
                rmb_position = Some(mouse_position());
            } else {
                rmb_position = Some(mouse_position());
            }
        }

        for (i, j) in (0..matrix_rows).cartesian_product(0..matrix_cols) {
            draw_rectangle_lines(
                (i * CELL_SIZE) as f32 + offset_x,
                (j * CELL_SIZE) as f32 + offset_y,
                CELL_SIZE as f32,
                CELL_SIZE as f32,
                2.0,
                WHITE,
            )
        }

        next_frame().await
    }
}
