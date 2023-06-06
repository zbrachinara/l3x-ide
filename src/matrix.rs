use itertools::Itertools;
use macroquad::prelude::*;
use smallvec::{smallvec, SmallVec};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Index,
};
use tap::Tap;
use vec_drain_where::VecDrainWhereExt;

mod file;
mod ui;
use core::cmp::{max,min};
use crate::{
    l3x::{Direction, L3XCommand, MaybeL3X, L3X},
    registers::Registers,
    sound::chord::Chord,
    swapbuffer::SwapBuffer,
    traveler::Traveler,
    wasync::AsyncContext,
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
#[derive(Clone)]
pub struct L3XData {
    data: Vec<Vec<MaybeL3X>>,
    dims: UVec2,
}
impl Index<UVec2> for L3XData {
    type Output = MaybeL3X;
    fn index(&self, index: UVec2) -> &MaybeL3X {
        &self.data[index.y as usize][index.x as usize]
    }
}
impl std::ops::IndexMut<UVec2> for L3XData {
    fn index_mut(&mut self, index: UVec2) -> &mut MaybeL3X {
        &mut self.data[index.y as usize][index.x as usize]
    }
}
//all should be copy except paste - I don't think this is possible
pub enum MatrixAction {
    Resize(UVec2),
    Swap(Selection, IVec2),
    ReflectH(Selection),
    ReflectV(Selection),
    Paste(IVec2, L3XData),
    Transpose(Selection),
}
use crate::matrix::MatrixAction::*;

impl MatrixAction {
    fn inverse(&self, current_state: &mut Matrix) -> MatrixAction {
        match self {
            //put the size back to what it was
            Resize(_) => Resize(current_state.dims),
            //paste back what used to be in that spot
            Paste(start, data) => Paste(
                *start,
                current_state.snip(Selection {
                    starts: *start,
                    ends: *start + data.dims.as_ivec2()-ivec2(1,1),
                }),
            ),
            //all other transformations are self-inverting
            //but we can't do a match all because we have to manually copy these
            Swap(s, i) => Swap(*s, *i),
            ReflectH(s) => ReflectH(*s),
            ReflectV(s) => ReflectV(*s),
            Transpose(s) => Transpose(*s),
        }
    }
}

/// Defines the selected area on which to operate, start and end inclusive
#[derive(Clone, Copy)]
pub struct Selection {
    starts: IVec2,
    ends: IVec2,
}

impl From<IVec2> for Selection {
    fn from(value: IVec2) -> Self {
        Self {
            starts: value,
            ends: value,
        }
    }
}

impl Selection {
    fn rect(&self, offset: Vec2, cell_size: f32, scale: f32) -> Rect {
        let starts = (self.starts.as_vec2() * cell_size + offset) * scale;
        let size = ((self.ends + IVec2::ONE - self.starts).as_vec2() * cell_size) * scale;
        Rect::new(starts.x, starts.y, size.x, size.y)
    }

    fn transpose(self) -> Self {
        Self {
            starts: self.starts,
            ends: (self.ends-self.starts).yx()+self.starts,
        }
    }

    fn contains(&self, location: IVec2) -> bool {
        self.starts.cmple(location).all() && self.ends.cmpge(location).all()
    }
    fn to(self,newStart:IVec2)->Self {
        Self {
            starts: newStart,
            ends: newStart+self.ends-self.starts
        }
    }
    fn width(&self) -> i32{
        self.ends.x-self.starts.x+1
    }
    fn height(&self) -> i32{
        self.ends.y-self.starts.y+1
    }
}

pub struct Matrix {
    mode: MatrixMode,
    instructions: HashMap<IVec2, L3X>,
    pub dims: UVec2,
    selecting: Option<Selection>,
    selecting_text: String,
    period: usize,
    stepping: bool,

    time: usize,

    queues: HashMap<IVec2, VecDeque<Registers>>,
    waiting_for_queue: Vec<(Traveler, Registers)>,
    travelers: SwapBuffer<Traveler>,
    output: Option<Registers>,
    output_stream: Vec<Registers>,

    focus_editing: bool,

    single_input: UiSingleInput,
    stream_input: UiStreamInput,

    simulating: bool,
    sound_follows_cursor: bool,
    global_volume: u8,
    gridlines: bool,
    history:Vec<MatrixAction>,
    copy_data:Option<L3XData>
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
            focus_editing: false,
            single_input: Default::default(),
            stream_input: Default::default(),
            simulating: false,
            sound_follows_cursor: false,
            global_volume: 80,
            gridlines: false,
            time: 0,
            history:vec![],
            copy_data:None
        }
    }
}

impl Matrix {
    pub fn update(&mut self, ctx: &mut AsyncContext) {
        self.time += 1;
        if self.time > self.period {
            self.time %= self.period;
            if self.stepping {
                self.step();
            }
        }
        self.try_import_data(ctx);
        ctx.try_export_file();
    }

    pub fn draw(&self, offset: Vec2, cell_size: f32, scale: f32) {
        let primary_color = DARKBROWN;

        let cell_size = cell_size * scale;
        let offset = offset * scale;
        let font_size = 32.0 * scale;

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
        if let Some(range) = self.selecting {
            let r = range.rect(offset, cell_size, scale); // TODO restrict rect to bounds of matrix
            draw_rectangle(r.x, r.y, r.w, r.h, LIGHTGRAY)
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

    pub fn global_volume(&self) -> f32 {
        self.global_volume as f32 / 100.
    }

    pub fn update_sound(&self, logical_mouse: Vec2) -> Option<Chord> {
        if self.sound_follows_cursor {
            self.travelers
                .iter()
                .map(|traveler| {
                    let dist = traveler.location.as_vec2().distance(logical_mouse);
                    (traveler, dist)
                })
                .filter(|(_, dist)| *dist < 1.5)
                .min_by_key(|(_, dist)| (dist * 1000.) as usize)
                .map(|(traveler, distance)| {
                    let volume = (distance).clamp(0.0, 1.0) * self.global_volume();
                    Chord {
                        volume,
                        pitches: traveler.pitches(),
                    }
                })
        } else {
            Some(
                self.travelers
                    .iter()
                    .map(|traveler| Chord {
                        volume: self.global_volume(),
                        pitches: traveler.pitches(),
                    })
                    .sum(),
            )
        }
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
        self.mode == MatrixMode::L3X
            && self
                .selecting
                .map(|u| u.contains(ivec2(1, 0)))
                .unwrap_or(false)
    }

    pub fn set_dims(&mut self, dims: IVec2) {
        if !self.simulating && self.mode.minimum_size().as_ivec2().cmple(dims).all() {
            self.dims = dims.as_uvec2();
        }
    }

    pub fn set_selection_end(&mut self, end: IVec2) {
        if let Some(sel) = self.selecting.as_mut() {
            if sel.starts.cmple(end).all() {
                sel.ends = end;
            }
        }
    }

    pub fn edit(&mut self, location: Selection) {
        if location.starts.cmpge(IVec2::ZERO).all()
            && location.ends.cmplt(self.dims.as_ivec2()).all()
        {
            let location = location;
            self.focus_editing = true;
            self.selecting = Some(location);
            self.selecting_text = self
                .instructions
                .get(&location.starts)
                .map(|l3x| l3x.to_string())
                .unwrap_or("".to_string()); // TODO only do this when selecting a single cell
        }
    }

    pub fn transpose(&mut self) {
        self.dims = self.dims.yx();
        let instructions_new: HashMap<_, _> = self
            .instructions
            .drain()
            .map(|(k, v)| (k.yx(), v.tap_mut(|v| v.direction = v.direction.opposite())))
            .collect();
        self.instructions = instructions_new;
        if let Some(selecting) = self.selecting {
            self.edit(selecting.transpose())
        }
    }

    pub fn stop_edit(&mut self) {
        self.selecting = None;
    }

    pub fn finalize_resize(&mut self) {
        self.apply(MatrixAction::Resize(self.dims));
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
    fn snip(&mut self, range: Selection) -> L3XData {
        let mut res=vec![];
        for i in range.starts.y..range.ends.y+1 {
            let mut row = vec![];
            for j in range.starts.x..range.ends.x+1 {
                row.push(MaybeL3X::from(
                    self.instructions.remove(&IVec2::from((j,i)))
                ));
            }
            res.push(row);
        }
        L3XData {
            data: res,
            dims: UVec2::from((
                range.width() as u32,
                range.height() as u32
            ))
        }
    }
    fn apply_raw(&mut self, a: MatrixAction) {
        match a {
            Resize(dims) => {
                self.dims=dims;
            },
            Swap(selection,target) => {
                {
                    for i in 0..selection.height() {
                        for j in 0..selection.width() {
                            self.swap_and_map(selection.starts+ivec2(j,i), target+ivec2(j,i),|c| c);
                        }
                    }
                };
            },
            ReflectH(selection) => {
                {
                    for i in 0..selection.height() {
                        for j in 0..(selection.width()+1)/2 {
                            self.swap_and_map(selection.starts+ivec2(j,i), selection.starts+ivec2(selection.width()-j-1, i),|mut c| {c.direction=c.direction.reflectH();c});
                        }
                    }
                };
            },
            ReflectV(selection) => {
                {
                    for i in 0..(selection.height()+1)/2 {
                        for j in 0..selection.width() {
                            self.swap_and_map(selection.starts+ivec2(j,i), selection.starts+ivec2(j, selection.height()-i-1),|mut c| {c.direction=c.direction.reflectV();c});
                        }
                    }
                };
            },
            Transpose(selection) => {
                {
                    let width=max(selection.width(),selection.height());
                    let height=min(selection.width(),selection.height());
                    println!("{:?}",(width,height));
                    for i in 0..height {
                        for j in i..width {
                            println!("{:?}",(i,j));
                            self.swap_and_map(selection.starts+ivec2(j,i), selection.starts+ivec2(i,j),|mut c| {c.direction=c.direction.transpose();c});
                        }
                    }
                };
            },
            Paste(target, mut data) => {
                for i in 0..data.dims.y {
                    for j in 0..data.dims.x {
                        Option::<L3X>::from(data[uvec2(j,i)].optionalTake()).map_or(self.instructions.remove(&(target+ivec2(j as i32,i as i32))), |c| self.instructions.insert(target+ivec2(j as i32,i as i32),c));
                    }
                }
            }
        };
    }
    pub fn apply(&mut self, a: MatrixAction) {
        let inverse=a.inverse(self);
        self.apply_raw(a);
        self.history.push(inverse);
    }
    
    pub fn undo(&mut self) {
        let t=self.history.pop();
        t.map_or((), |f| self.apply_raw(f))
    }
    fn swap_and_map<F: Fn(L3X)->L3X>(&mut self,k1:IVec2,k2:IVec2,f:F) {
        let e1=self.instructions.remove_entry(&k1);
        let e2=self.instructions.remove_entry(&k2);
        e1.and_then(|(k,v)| self.instructions.insert(k2,f(v)));
        e2.and_then(|(k,v)| self.instructions.insert(k1,f(v)));
    }
}
