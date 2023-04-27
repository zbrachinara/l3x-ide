use egui::Ui;
use itertools::Itertools;
use macroquad::prelude::*;
use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, VecDeque};
use vec_drain_where::VecDrainWhereExt;

use crate::{
    l3x::{Direction, L3XCommand, L3X},
    swapbuffer::SwapBuffer,
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
    storage: HashMap<IVec2, L3X>,
    dims: UVec2,
    selecting: Option<IVec2>,
    selecting_text: String,

    queues: HashMap<IVec2, VecDeque<Registers>>,
    waiting_for_queue: Vec<(Traveler, Registers)>,
    travelers: SwapBuffer<Traveler>,
    output: Option<Registers>,
    output_stream: Vec<Registers>,

    focus_editing: u8,
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
            selecting: Default::default(),
            selecting_text: Default::default(),
            queues: Default::default(),
            waiting_for_queue: Default::default(),
            travelers: Default::default(),
            output: Default::default(),
            output_stream: Default::default(),
            focus_editing: 0,
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
            if matches!(self.selecting, Some(vec) if vec == uvec2(x, y).as_ivec2()) {
                draw_rectangle(lower.x, lower.y, cell_size, cell_size, GRAY);
            }
            draw_rectangle_lines(lower.x, lower.y, cell_size, cell_size, 2.0, WHITE);

            // TODO represent cell contents graphically
            if let Some(l3x) = self.storage.get(&uvec2(x, y).as_ivec2()) {
                draw_text(
                    &l3x.to_string(),
                    (lower + text_offset).x,
                    (lower + text_offset).y,
                    font_size,
                    WHITE,
                )
            }
        }

        // draw travelers
        for traveler in &**self.travelers {
            let pos =
                ((traveler.location.as_vec2() + Vec2::splat(0.5)) * cell_size + offset) * scale;
            draw_circle(pos.x, pos.y, 10.0 * scale, BLUE);
        }
    }

    /// Forces the streaming input square to be a queue when the matrix is in l3x mode
    fn force_queue_l3x(&mut self) {
        if self.mode == MatrixMode::L3X {
            self.storage
                .entry(ivec2(1, 0))
                .and_modify(|e| e.command = L3XCommand::Queue)
                .or_insert(L3X {
                    direction: Direction::Down,
                    command: L3XCommand::Queue,
                });
        }
    }

    fn is_editing_input_stream(&self) -> bool {
        self.mode == MatrixMode::L3X && self.selecting == Some(ivec2(1, 0))
    }

    pub fn set_dims(&mut self, dims: IVec2) {
        if !self.simulating && self.mode.minimum_size().as_ivec2().cmple(dims).all() {
            self.dims = dims.as_uvec2();
        }
    }

    pub fn edit(&mut self, location: IVec2) {
        if location.cmpge(IVec2::ZERO).all() && location.cmplt(self.dims.as_ivec2()).all() {
            let location = location;
            self.focus_editing = 4;
            self.selecting = Some(location);
            self.selecting_text = self
                .storage
                .get(&location)
                .map(|l3x| l3x.to_string())
                .unwrap_or("".to_string());
        }
    }

    pub fn stop_edit(&mut self) {
        self.selecting = None;
    }

    fn init_simulation_inner(&mut self) -> Option<()> {
        self.travelers.push(Traveler {
            value: self.single_input.clone()?,
            location: IVec2::ZERO,
            direction: Direction::Down,
        });

        self.queues
            .insert(ivec2(1, 0), self.stream_input.clone().into());

        Some(())
    }

    fn init_simulation(&mut self) {
        self.simulating = self.init_simulation_inner().is_some()
    }

    fn cleanup_simulation(&mut self) {
        self.simulating = false;
        self.travelers.clear();
        self.queues.clear();
        self.waiting_for_queue.clear();
        self.output = None;
        self.output_stream.clear();
    }

    fn step(&mut self) {
        // TODO check errors and cleanup output travelers
        self.step_travelers();
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
                ui.button("⏸").on_hover_text("pause (stop autostepping)");
                if ui.button("⏭").on_hover_text("step by one cycle").clicked() {
                    self.step()
                }
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
            if let Some(ref value) = self.single_input {
                ui.label(format!("Current value: {}", value));
            }
            if ui
                .text_edit_singleline(&mut self.single_input_text)
                .lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                if let Ok(registers) = self.single_input_text.parse() {
                    self.single_input = Some(registers);
                }
            }

            if self.mode == MatrixMode::L3X {
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
            }
        });

        if let Some(location) = self.selecting {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                ui.separator();
                ui.label(format!("Cell value @ {location}"));
                ui.horizontal(|ui| {
                    let textedit = ui.text_edit_singleline(&mut self.selecting_text);
                    match self.focus_editing {
                        x if x > 1 => self.focus_editing -= 1,
                        1 => {
                            textedit.request_focus();
                            self.focus_editing -= 1;
                        }
                        _ => (),
                    }
                    if textedit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(serialize_success) = L3X::try_from(self.selecting_text.as_str()) {
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
                        self.selecting_text.clear();
                        self.storage.remove(&location);
                        self.force_queue_l3x();
                    }
                });
            });

            if self.simulating {
                ui.separator();
                ui.label("Travelers on this cell");
                self.travelers
                    .iter()
                    .filter(|&&Traveler { location: loc, .. }| loc == location)
                    .for_each(|traveler| {
                        ui.label(traveler.to_string());
                    });

                if let Some(queue) = self.queues.get(&location) {
                    ui.separator();
                    ui.label("Queue on this cell");
                    for register in queue {
                        ui.label(register.to_string());
                    }
                }
            }

            ui.separator();
            ui.label("Output");
            if let Some(ref register) = self.output {
                ui.label(register.to_string());
            }

            ui.separator();
            ui.label("Output stream");
            for register in &self.output_stream {
                ui.label(register.to_string());
            }
        }
    }

    fn step_travelers(&mut self) -> Result<(), ()> {
        self.travelers.try_swap(|mut traveler| 'a: {
            let instruction = if traveler.location.cmplt(self.dims.as_ivec2()).all() {
                self.storage.get(&traveler.location).ok_or(())?
            } else if traveler.location == self.dims.as_ivec2() - ivec2(1, 0) {
                break 'a if self.output.is_none() {
                    self.output = Some(traveler.value);
                    Ok(smallvec![])
                } else {
                    Err(()) // will be a different type of error than out-of-bounds
                };
            } else if traveler.location == self.dims.as_ivec2() - ivec2(2, 0) {
                self.output_stream.push(traveler.value);
                break 'a Ok(smallvec![]);
            } else {
                break 'a Err(());
            };

            let aligned = traveler.direction == instruction.direction;

            let out: SmallVec<[_; 2]> = match &instruction.command {
                L3XCommand::Multiply(with) => {
                    if aligned {
                        traveler.value *= with;
                        traveler.direct(instruction.direction);
                    } else if let Some(div) = traveler.value.try_div(with) {
                        traveler.value = div;
                        traveler.direct(instruction.direction);
                    } else {
                        traveler.direct(instruction.direction.opposite());
                    }

                    smallvec![traveler]
                }
                L3XCommand::Duplicate => {
                    let mut new_traveler = traveler.clone();
                    traveler.direct(instruction.direction);
                    new_traveler.direct(instruction.direction.opposite());

                    smallvec![traveler, new_traveler]
                }
                L3XCommand::Queue => {
                    if aligned {
                        self.queues
                            .entry(traveler.location)
                            .and_modify(|q| q.push_back(traveler.value.clone()))
                            .or_insert_with(|| vec![traveler.value.clone()].into());
                    } else {
                        traveler.direction = instruction.direction;
                        self.waiting_for_queue.push((traveler, Registers::ONE));
                    }
                    smallvec![]
                }
                L3XCommand::Annihilate => {
                    traveler.value = Registers::ONE;
                    traveler.direct(instruction.direction);
                    smallvec![traveler]
                }
            };
            Ok(out)
        })?;

        self.waiting_for_queue
            .e_drain_where(|(traveler, u)| {
                self.queues
                    .get_mut(&traveler.location)
                    .and_then(|q| q.pop_front())
                    .map(|register| *u = register)
                    .is_some()
            })
            .for_each(|(mut traveler, multiplier)| {
                traveler.value *= multiplier;
                traveler.direct(traveler.direction); // needs to step off the queue cell
                self.travelers.push(traveler)
            });

        Ok(())
    }
}
