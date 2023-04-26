use egui::Ui;
use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};
use vec_drain_where::VecDrainWhereExt;

use crate::{
    l3x::{Direction, L3XCommand, L3X},
    traveler::{Registers, Traveler},
};

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

    queues: HashMap<UVec2, VecDeque<Registers>>,
    travelers: Vec<Traveler>,
    single_input_next_frame_focus: bool,
    single_input_text: String,
    single_input: Option<Registers>,
    stream_input_text: Option<String>,
    stream_input: Vec<Registers>,

    simulating: bool,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            storage: Default::default(),
            dims: uvec2(1, 1),
            editing: Default::default(),
            editing_text: Default::default(),
            queues: Default::default(),
            travelers: Default::default(),
            single_input_next_frame_focus: false,
            single_input_text: Default::default(),
            single_input: Default::default(),
            stream_input_text: Default::default(),
            stream_input: Default::default(),
            simulating: false,
        }
    }
}

impl Matrix {
    pub fn draw(&self, offset: Vec2, cell_size: f32, scale: f32) {
        let cell_size = cell_size * scale;
        let font_size = 32.0 * scale;
        let text_offset = vec2(0.05, 0.67) * cell_size;

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
            let lower = (uvec2(x, y).as_vec2() * cell_size + offset) * scale;
            draw_rectangle_lines(lower.x, lower.y, cell_size, cell_size, 2.0, WHITE);

            // TODO represent cell contents graphically
            if let Some(l3x) = self.storage.get(&uvec2(x, y)) {
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

    /// Forces the streaming input square to be a queue when the matrix is in l3x mode
    fn force_queue_l3x(&mut self) {
        if self.mode == MatrixMode::L3X {
            self.storage
                .entry(uvec2(1, 0))
                .and_modify(|e| e.command = L3XCommand::Queue)
                .or_insert(L3X {
                    direction: Direction::Down,
                    command: L3XCommand::Queue,
                });
        }
    }

    fn is_editing_input_stream(&self) -> bool {
        self.mode == MatrixMode::L3X && self.editing == Some(uvec2(1, 0))
    }

    pub fn set_dims(&mut self, dims: IVec2) {
        if !self.simulating && self.mode.minimum_size().as_ivec2().cmple(dims).all() {
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

    fn init_simulation(&mut self) {
        self.simulating = true;
    }

    fn cleanup_simulation(&mut self) {
        self.simulating = false;
    }

    pub fn config_ui(&mut self, ui: &mut Ui) {
        ui.label("Simulation");
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                if ui.button("Start").clicked() {
                    self.init_simulation()
                }
            });
            ui.scope(|ui| {
                ui.set_enabled(self.simulating);
                ui.button("▶").on_hover_text("play (step automatically)");
                ui.button("⏸").on_hover_text("pause (stop autostepping");
                ui.button("⏭").on_hover_text("step by one cycle");
                if ui
                    .button("⏹")
                    .on_hover_text("exit the simulation")
                    .clicked()
                {
                    self.cleanup_simulation();
                }
            });
        });

        ui.scope(|ui| {
            ui.set_enabled(!self.simulating);
            ui.separator();
            ui.label("L3 Mode");
            ui.horizontal(|ui| {
                let l3_radio = ui.radio_value(&mut self.mode, MatrixMode::L3, "L3");
                let l3x_radio = ui.radio_value(&mut self.mode, MatrixMode::L3X, "L3X");
                if l3_radio.union(l3x_radio).changed() {
                    self.dims = self.dims.max(self.mode.minimum_size());
                    self.force_queue_l3x()
                }
            });

            ui.separator();
            ui.label("Single input (L3)");
            if ui
                .text_edit_singleline(&mut self.single_input_text)
                .lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                if let Ok(registers) = self.single_input_text.parse() {
                    self.single_input = Some(registers);
                }
            }

            ui.separator();
            ui.label("Multi input (L3X)");

            self.stream_input
                .e_drain_where(|registers| ui.button(registers.to_string()).clicked())
                .for_each(drop);
            if let Some(ref mut text) = self.stream_input_text {
                let textedit = ui.text_edit_singleline(text);
                if self.single_input_next_frame_focus {
                    textedit.request_focus();
                    self.single_input_next_frame_focus = false;
                }
                if textedit.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(registers) = text.parse() {
                            self.stream_input.push(registers);
                        }
                        text.clear();
                        self.single_input_next_frame_focus = true;
                    } else {
                        self.stream_input_text = None;
                    }
                }
            } else if ui.button("Add to stream").clicked() {
                self.stream_input_text = Some(String::new());
                self.single_input_next_frame_focus = true;
            }
        });

        if let Some(location) = self.editing {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                ui.separator();
                ui.label(format!("Cell value @ {location}"));
                ui.horizontal(|ui| {
                    let textedit = ui.text_edit_singleline(&mut self.editing_text);
                    if textedit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(serialize_success) = L3X::try_from(self.editing_text.as_str()) {
                            if self.is_editing_input_stream()
                                && serialize_success.command != L3XCommand::Queue
                            {
                                log::warn!("In L3X mode, edited square *must* be a queue!")
                            } else {
                                self.storage.insert(location, serialize_success);
                            }
                        } else {
                            log::warn!("Serialization failure")
                        }
                    }
                    if ui.button("Clear").clicked() {
                        self.editing_text.clear();
                        self.storage.remove(&location);
                        self.force_queue_l3x();
                    }
                });
            });
        }
    }
}
