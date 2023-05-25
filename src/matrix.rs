use if_chain::if_chain;
use itertools::Itertools;
use macroquad::prelude::*;
use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, HashSet, VecDeque};
use vec_drain_where::VecDrainWhereExt;

mod file;
#[cfg(not(target_arch = "wasm32"))]
mod future_states;
mod ui;

use crate::{
    l3x::{Direction, L3XCommand, L3X},
    registers::Registers,
    sound::Chord,
    swapbuffer::SwapBuffer,
    traveler::Traveler,
};

use self::ui::{UiSingleInput, UiStreamInput};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum MatrixMode {
    #[default]
    L3,
    L3X,
}

bitflags::bitflags! {
    #[derive(Copy, Clone)]
    struct Alignments: u8 {
        const ALIGNED = 0b01;
        const UNALIGNED = 0b10;
    }
}

impl Alignments {
    fn aligned(one: Direction, the_other: Direction) -> Self {
        if one == the_other {
            Self::ALIGNED
        } else {
            Self::UNALIGNED
        }
    }
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
    instructions: HashMap<IVec2, L3X>,
    dims: UVec2,
    selecting: Option<IVec2>,
    selecting_text: String,
    period: usize,
    stepping: bool,

    time: usize,

    queues: HashMap<IVec2, VecDeque<Registers>>,
    waiting_for_queue: Vec<(Traveler, Registers)>,
    travelers: SwapBuffer<Traveler>,
    output: Option<Registers>,
    output_stream: Vec<Registers>,

    focus_editing: u8,

    single_input: UiSingleInput,
    stream_input: UiStreamInput,

    simulating: bool,
    gridlines: bool,

    // rust async moments
    #[cfg(not(target_arch = "wasm32"))]
    future_states: future_states::FutureStates,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            instructions: Default::default(),
            dims: uvec2(1, 1),
            selecting: Default::default(),
            selecting_text: Default::default(),
            period: 10,
            stepping: false,
            queues: Default::default(),
            waiting_for_queue: Default::default(),
            travelers: Default::default(),
            output: Default::default(),
            output_stream: Default::default(),
            focus_editing: 0,
            single_input: Default::default(),
            stream_input: Default::default(),
            simulating: false,
            gridlines: false,
            #[cfg(not(target_arch = "wasm32"))]
            future_states: Default::default(),
            time: 0,
        }
    }
}

impl Matrix {
    pub fn update(&mut self) {
        self.time += 1;
        if self.time > self.period {
            self.time %= self.period;
            if self.stepping {
                self.step();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.try_open_file();
            self.try_export_file();
        }
    }

    pub fn draw(&self, offset: Vec2, cell_size: f32, scale: f32) {
        let primary_color = DARKBROWN;

        let cell_size = cell_size * scale;
        let offset = offset * scale;
        let font_size = 32.0 * scale;
        let text_offset = vec2(0.05, 0.67) * cell_size;

        // annotate input and output
        let io_text_offset = vec2(0.4, 0.67) * cell_size;
        let i_single = offset + vec2(0.0, -cell_size) + io_text_offset;
        let o_single = offset + (self.dims - uvec2(1, 0)).as_vec2() * cell_size + io_text_offset;
        draw_text("I", i_single.x, i_single.y, font_size, primary_color);
        draw_text("O", o_single.x, o_single.y, font_size, primary_color);
        if self.mode == MatrixMode::L3X {
            let i_stream = offset + vec2(cell_size, -cell_size) + io_text_offset;
            let o_stream =
                offset + (self.dims - uvec2(2, 0)).as_vec2() * cell_size + io_text_offset;
            draw_text("I_s", i_stream.x, i_stream.y, font_size, primary_color);
            draw_text("O_s", o_stream.x, o_stream.y, font_size, primary_color);
        }

        // highlight selected square
        if_chain! {
            if let Some(location) = self.selecting;
            if location.cmplt(self.dims.as_ivec2()).all();
            then {
                let lower = (location.as_vec2() * cell_size) + offset;
                draw_rectangle(lower.x, lower.y, cell_size, cell_size, LIGHTGRAY);
            }
        }

        // box around the matrix
        draw_rectangle_lines(
            offset.x,
            offset.y,
            self.dims.x as f32 * cell_size,
            self.dims.y as f32 * cell_size,
            4.0,
            primary_color,
        );

        if self.gridlines {
            for column in 1..self.dims.x {
                let lower = vec2(column as f32, 0.) * cell_size + offset;
                let upper = lower + vec2(0., self.dims.y as f32) * cell_size;
                draw_line(lower.x, lower.y, upper.x, upper.y, 2.0, primary_color)
            }
            for row in 1..self.dims.y {
                let lower = vec2(0., row as f32) * cell_size + offset;
                let upper = lower + vec2(self.dims.x as f32, 0.) * cell_size;
                draw_line(lower.x, lower.y, upper.x, upper.y, 2.0, primary_color)
            }
        }

        for (location, instruction) in &self.instructions {
            if location.cmplt(self.dims.as_ivec2()).all() {
                instruction.draw(
                    &self.instructions,
                    self.dims,
                    *location,
                    cell_size,
                    offset,
                    font_size,
                    primary_color,
                )
            }
        }

        // draw travelers
        for traveler in &**self.travelers {
            let pos = (traveler.location.as_vec2() + Vec2::splat(0.5)) * cell_size + offset;
            draw_circle(pos.x, pos.y, 10.0 * scale, BLUE);
        }
    }

    pub fn update_sound(&self, logical_mouse: Vec2) -> Option<Chord> {
        self.travelers
            .iter()
            .map(|traveler| {
                let dist = traveler.location.as_vec2().distance(logical_mouse);
                (traveler, dist)
            })
            .filter(|(_, dist)| *dist < 1.5) 
            .min_by_key(|(_, dist)| (dist * 1000.) as usize)
            .map(|(traveler, distance)| {
                let volume = (distance).clamp(0.0, 1.0);
                Chord {
                    volume,
                    pitches: traveler.pitches(),
                }
            })
    }

    /// Forces the streaming input square to be a queue when the matrix is in l3x mode
    fn force_queue_l3x(&mut self) {
        if self.mode == MatrixMode::L3X {
            self.instructions
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
                .instructions
                .get(&location)
                .map(|l3x| l3x.to_string())
                .unwrap_or("".to_string());
        }
    }

    pub fn transpose(&mut self) {
        self.dims = self.dims.yx();
        let instructions_new: HashMap<_, _> = self
            .instructions
            .drain()
            .map(|(mut k, mut v)| {
                k = k.yx();
                v.direction = match v.direction {
                    Direction::Up => Direction::Left,
                    Direction::Down => Direction::Right,
                    Direction::Left => Direction::Up,
                    Direction::Right => Direction::Down,
                };

                (k, v)
            })
            .collect();
        self.instructions = instructions_new;
        if let Some(selecting) = self.selecting {
            self.edit(selecting.yx())
        }
    }

    pub fn stop_edit(&mut self) {
        self.selecting = None;
    }

    fn init_simulation_inner(&mut self) -> Option<()> {
        self.travelers.push(Traveler {
            value: self.single_input.value().clone(),
            location: IVec2::ZERO,
            direction: Direction::Down,
        });

        self.queues
            .insert(ivec2(1, 0), self.stream_input.value().clone().into());

        Some(())
    }

    fn init_simulation(&mut self) {
        if self.mode == MatrixMode::L3
            || self.instructions[&ivec2(1, 0)].command == L3XCommand::Queue
        {
            self.simulating = self.init_simulation_inner().is_some()
        } else {
            log::warn!("Could not start simulation: There is no queue on the queue input square!");
        }
    }

    fn cleanup_simulation(&mut self) {
        self.simulating = false;
        self.travelers.clear();
        self.queues.clear();
        self.waiting_for_queue.clear();
        self.output = None;
        self.output_stream.clear();
    }

    pub fn step(&mut self) {
        if self.collision_free() {
            self.step_travelers();
        } else {
            log::error!("Collision detected");
        }
    }
    fn is_output_cell(&self, location: IVec2) -> bool {
        location == self.dims.as_ivec2() - ivec2(1, 0)
            || self.mode == MatrixMode::L3X && location == self.dims.as_ivec2() - ivec2(2, 0)
    }

    /// Iterates through the travelers stored in this matrix and checks whether they collide.
    /// Ignores collisions on a queue *&&* between a traveler aligned and one not aligned with the
    /// queue.
    fn collision_free(&self) -> bool {
        let mut collision_check = HashSet::new();
        let mut queue_collision_check = HashMap::<_, Alignments>::new();
        self.travelers.iter().all(|traveler| {
            self.instructions
                .get(&traveler.location)
                .map(|l3x| {
                    if l3x.command == L3XCommand::Queue {
                        let aligned = Alignments::aligned(l3x.direction, traveler.direction);
                        if let Some(alignments) = queue_collision_check.get_mut(&traveler.location)
                        {
                            if alignments.contains(aligned) {
                                false
                            } else {
                                *alignments &= aligned;
                                true
                            }
                        } else {
                            queue_collision_check.insert(traveler.location, aligned);
                            true
                        }
                    } else {
                        collision_check.insert(traveler.location)
                    }
                })
                .unwrap_or_else(|| self.is_output_cell(traveler.location))
        })
    }

    fn step_travelers(&mut self) -> Result<(), ()> {
        self.travelers.try_swap(|mut traveler| {
            let instruction = if traveler.location.cmplt(self.dims.as_ivec2()).all() {
                self.instructions.get(&traveler.location).ok_or(())?
            } else if traveler.location == self.dims.as_ivec2() - ivec2(1, 0) {
                return if self.output.is_none() {
                    self.output = Some(traveler.value);
                    Ok(smallvec![])
                } else {
                    Err(()) // will be a different type of error than out-of-bounds
                };
            } else if traveler.location == self.dims.as_ivec2() - ivec2(2, 0) {
                self.output_stream.push(traveler.value);
                return Ok(smallvec![]);
            } else {
                return Err(());
            };

            let aligned = traveler.direction == instruction.direction;

            let out: SmallVec<[_; 2]> = match &instruction.command {
                L3XCommand::Multiply(with) => {
                    smallvec![if aligned {
                        traveler.mul(with).direct(instruction.direction)
                    } else if let Some(div) = traveler.value.try_div(with) {
                        traveler.value(div).direct(instruction.direction)
                    } else {
                        traveler.direct(instruction.direction.opposite())
                    }]
                }
                L3XCommand::Duplicate => {
                    smallvec![
                        traveler.clone().direct(instruction.direction),
                        traveler.direct(instruction.direction.opposite())
                    ]
                }
                L3XCommand::Queue => {
                    if aligned {
                        self.queues
                            .entry(traveler.location)
                            .and_modify(|q| q.push_back(traveler.value.clone()))
                            .or_insert_with(|| vec![traveler.value.clone()].into());
                        smallvec![]
                    } else {
                        traveler.direction = instruction.direction;
                        if let Some(queued) = self
                            .queues
                            .get_mut(&traveler.location)
                            .and_then(|q| q.pop_front())
                        {
                            smallvec![traveler.mul(&queued).step()]
                        } else {
                            self.waiting_for_queue.push((traveler, Registers::ONE));
                            smallvec![]
                        }
                    }
                }
                L3XCommand::Annihilate => {
                    smallvec![traveler.value(Registers::ONE).direct(instruction.direction)]
                }
            };
            Ok(out)
        })?;

        let dequeued_travelers = self
            .waiting_for_queue
            .e_drain_where(|(traveler, u)| {
                let queued_traveler = self
                    .travelers
                    .iter()
                    .position(|e| e.location == traveler.location)
                    .map(|ix| self.travelers.swap_remove(ix).value);

                queued_traveler.map(|register| *u = register).is_some()
            })
            .map(|(traveler, multiplier)| traveler.mul(&multiplier).step())
            .collect_vec();

        self.travelers.extend(dequeued_travelers);

        Ok(())
    }
}
