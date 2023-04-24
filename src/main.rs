use macroquad::prelude::*;
use macroquad::window::next_frame;

use crate::matrix::Matrix;

mod matrix;

fn mouse() -> Vec2 {
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

    loop {
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

        matrix.draw(offset, CELL_SIZE, 1.0);

        next_frame().await
    }
}
