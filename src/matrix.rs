use egui::Ui;
use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::l3x::L3X;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum MatrixMode {
    #[default]
    L3,
    L3X,
}

impl MatrixMode {
    fn minimum_size(&self) -> UVec2 {
        match self {
            MatrixMode::L3 => uvec2(1, 1),
            MatrixMode::L3X => uvec2(2, 2),
        }
    }
}

pub struct Matrix {
    mode: MatrixMode,
    storage: HashMap<UVec2, L3X>,
    dims: UVec2,
    editing: Option<UVec2>,
    editing_text: String,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            storage: Default::default(),
            dims: UVec2 { x: 1, y: 1 },
            editing: Default::default(),
            editing_text: Default::default(),
        }
    }
}

impl Matrix {
    pub fn draw(&self, offset: Vec2, cell_size: f32, scale: f32) {
        let cell_size = cell_size * scale;
        let font_size = 32.0 * scale;
        let text_offset = Vec2::new(cell_size * 0.05, cell_size * 0.67);

        // annotate input and output
        let io_text_offset = vec2(0.4, 0.67) * cell_size;
        let i_single_text = offset + vec2(0.0, -cell_size) + io_text_offset;
        let o_single_text =
            offset + (self.dims - uvec2(1, 0)).as_vec2() * cell_size + io_text_offset;
        draw_text("I", i_single_text.x, i_single_text.y, font_size, WHITE);
        draw_text("O", o_single_text.x, o_single_text.y, font_size, WHITE);
        if self.mode == MatrixMode::L3X {
            let i_stream_text = offset + vec2(cell_size, -cell_size) + io_text_offset;
            let o_stream_text =
                offset + (self.dims - uvec2(2, 0)).as_vec2() * cell_size + io_text_offset;
            draw_text("I_s", i_stream_text.x, i_stream_text.y, font_size, WHITE);
            draw_text("O_s", o_stream_text.x, o_stream_text.y, font_size, WHITE);
        }

        for (x, y) in (0..self.dims.x).cartesian_product(0..self.dims.y) {
            let lower = (Vec2::new(x as f32, y as f32) * cell_size + offset) * scale;
            draw_rectangle_lines(lower.x, lower.y, cell_size, cell_size, 2.0, WHITE);

            // TODO represent cell contents graphically
            if let Some(l3x) = self.storage.get(&UVec2 { x, y }) {
                draw_text(
                    &l3x.to_string(),
                    (lower + text_offset).x,
                    (lower + text_offset).y,
                    font_size,
                    WHITE,
                )
            }
        }
    }

    pub fn set_dims(&mut self, dims: IVec2) {
        if self.mode.minimum_size().as_ivec2().cmple(dims).all() {
            self.dims = dims.as_uvec2();
        }
    }

    pub fn edit(&mut self, location: IVec2) {
        if location.cmpge(IVec2::ZERO).all() && location.cmplt(self.dims.as_ivec2()).all() {
            let location = location.as_uvec2();
            self.editing = Some(location);
            self.editing_text = self
                .storage
                .get(&location)
                .map(|l3x| l3x.to_string())
                .unwrap_or("".to_string());
        }
    }

    pub fn stop_edit(&mut self) {
        self.editing = None;
    }

    pub fn config_ui(&mut self, ui: &mut Ui) {
        ui.label("L3 Mode");
        ui.horizontal(|ui| {
            let l3_radio = ui.radio_value(&mut self.mode, MatrixMode::L3, "L3");
            let l3x_radio = ui.radio_value(&mut self.mode, MatrixMode::L3X, "L3X");
            if l3_radio.union(l3x_radio).changed() {
                self.dims = self.dims.max(self.mode.minimum_size());
            }
        });

        if let Some(location) = self.editing {
            ui.label("Editing");
            ui.text_edit_singleline(&mut self.editing_text);

            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(serialize_success) = L3X::try_from(self.editing_text.as_str()) {
                    self.storage.insert(location, serialize_success);
                } else {
                    log::debug!("Serialization failure")
                }
            }
        }
    }
}
