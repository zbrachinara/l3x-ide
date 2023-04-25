use egui::Ui;
use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Matrix {
    storage: HashMap<UVec2, String>,
    dims: UVec2,
    editing: Option<UVec2>,
    editing_text: String,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
            dims: UVec2 { x: 1, y: 1 },
            editing: None,
            editing_text: "".to_string(),
        }
    }
}

impl Matrix {
    pub fn draw(&self, offset: Vec2, cell_size: f32, scale: f32) {
        for (x, y) in (0..self.dims.x).cartesian_product(0..self.dims.y) {
            let lower = (Vec2::new(x as f32, y as f32) * cell_size + offset) * scale;
            let size = cell_size * scale;
            draw_rectangle_lines(lower.x, lower.y, size, size, 2.0, WHITE);

            let text_offset = lower + Vec2::new(size * 0.05, size * 0.67);
            if let Some(text) = self.storage.get(&UVec2 { x, y }) {
                draw_text(text, text_offset.x, text_offset.y, 32.0, WHITE)
            }
        }
    }

    pub fn set_dims(&mut self, dims: IVec2) {
        if dims.x >= 1 && dims.y >= 1 {
            self.dims = dims.as_uvec2();
        }
    }

    pub fn edit(&mut self, location: IVec2) {
        if location.x > 0
            && location.y > 0
            && location.x < self.dims.x as i32
            && location.y < self.dims.y as i32
        {
            let location = location.as_uvec2();
            self.editing = Some(location);
            self.editing_text = self
                .storage
                .get(&location)
                .cloned()
                .unwrap_or("".to_string());
        }
    }

    pub fn stop_edit(&mut self) {
        self.editing = None;
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if let Some(location) = self.editing {
            ui.label("Editing");
            ui.text_edit_singleline(&mut self.editing_text);

            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.storage.insert(location, self.editing_text.clone());
            }
        }
    }
}
