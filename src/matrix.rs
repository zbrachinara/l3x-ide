use egui::{Ui};
use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Matrix {
    storage: HashMap<UVec2, String>,
    dims: UVec2,
    editing: Option<UVec2>,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
            dims: UVec2 { x: 1, y: 1 },
            editing: None,
        }
    }
}

impl Matrix {
    pub fn draw(&self, offset: Vec2, cell_size: f32, scale: f32) {
        for (i, j) in (0..self.dims.x).cartesian_product(0..self.dims.y) {
            let lower = (Vec2::new(i as f32, j as f32) * cell_size + offset) * scale;
            let size = cell_size * scale;
            draw_rectangle_lines(lower.x, lower.y, size, size, 2.0, WHITE);
        }
    }

    pub fn set_dims(&mut self, dims: IVec2) {
        if dims.x >= 1 && dims.y >= 1 {
            self.dims = dims.as_uvec2();
        }
    }

    pub fn edit(&mut self, location: IVec2) {
        if location.x > 0 && location.y > 0 {
            self.editing = Some(location.as_uvec2());
        }
    }

    pub fn stop_edit(&mut self) {
        self.editing = None;
    }

    pub fn ui(&self, ui: &mut Ui) {
        if let Some(location) = self.editing {
            ui.label("Editing");
        }
    }
}
