use std::collections::HashMap;

use arrayvec::ArrayVec;
use itertools::Itertools;
use macroquad::prelude::*;
use smallvec::{smallvec, SmallVec};
use strum::IntoEnumIterator;
use tap::Pipe;

use crate::polygon::triangulate_indices;
use crate::registers::Registers;

macro_rules! arrayvec {
    () => (
        arrayvec::ArrayVec::new()
    );
    ($elem:expr; $n:expr) => (
        [$elem; $n].as_slice().try_into().unwrap()
    );
    ($($x:expr),+ $(,)?) => (
        [$($x),+].as_slice().try_into().unwrap()
    );
}

#[derive(PartialEq, Eq, Debug,Clone)]
pub struct L3X {
    // TODO support watch points
    pub direction: Direction,
    pub command: L3XCommand,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, strum_macros::EnumIter)]
#[rustfmt::skip]
pub enum Direction {
    Up, Down, Left, Right,
}

impl DirectionIter {
    fn with_offsets(self, from: IVec2) -> impl Iterator<Item = (Direction, IVec2)> {
        self.map(move |item| (item, IVec2::from(item) + from))
    }
}

#[derive(PartialEq, Eq, Debug,Clone)]
pub enum L3XCommand {
    Multiply(Registers),
    Duplicate,
    Queue,
    Annihilate,
}

#[derive(PartialEq, Eq, Debug)]
pub enum L3XParseError {
    BadDirection,
    BadCommand,
    ZeroCommand,
    NumberOverflow,
    WrongLength,
    UnaccountedCharacters,
    Empty,
}

#[derive(Default)]
pub enum MaybeL3X {
    Some(L3X),
    #[default]
    None,
}

impl From<MaybeL3X> for Option<L3X> {
    fn from(value: MaybeL3X) -> Self {
        match value {
            MaybeL3X::Some(v) => Some(v),
            MaybeL3X::None => None,
        }
    }
}
impl From<Option<L3X>> for MaybeL3X {
    fn from(value:Option<L3X>) -> Self {
        match value {
            None=>MaybeL3X::None,
            Some(v)=>MaybeL3X::Some(v)
        }
    }
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn horizontal(&self) -> bool {
        matches!(self, Direction::Left | Direction::Right)
    }

    /// If the direction is pointing in a positive axis. For macroquad this means down or right
    pub fn positive(&self) -> bool {
        matches!(self, Direction::Down | Direction::Right)
    }
}

impl From<Direction> for IVec2 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => ivec2(0, -1),
            Direction::Down => ivec2(0, 1),
            Direction::Left => ivec2(-1, 0),
            Direction::Right => ivec2(1, 0),
        }
    }
}

impl From<Direction> for Mat2 {
    /// Gives the transformation needed for a vector pointing downward to be rotated to point in the
    /// given direction
    fn from(value: Direction) -> Self {
        let rotate = mat2(vec2(0., 1.), vec2(-1., 0.));
        match value {
            Direction::Up => -Mat2::IDENTITY,
            Direction::Down => Mat2::IDENTITY,
            Direction::Left => rotate,
            Direction::Right => rotate * rotate * rotate,
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = L3XParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'u' | 'U' | 'n' | 'N' => Ok(Direction::Up),
            'd' | 'D' | 's' | 'S' => Ok(Direction::Down),
            'l' | 'L' | 'w' | 'W' => Ok(Direction::Left),
            'r' | 'R' | 'e' | 'E' => Ok(Direction::Right),
            _ => Err(L3XParseError::BadDirection),
        }
    }
}

impl TryFrom<&str> for L3X {
    type Error = L3XParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Option::from(MaybeL3X::try_from(value)?).ok_or(L3XParseError::Empty)
    }
}

impl TryFrom<&str> for MaybeL3X {
    type Error = L3XParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();

        if value.is_empty() {
            return Ok(Self::None);
        }

        let direction_char = value.chars().last().ok_or(L3XParseError::WrongLength)?;
        let direction: Direction = direction_char.try_into()?;

        let mut command_is_numeric = true;
        let command_str = value
            .chars()
            .take_while(|c| !c.is_alphabetic())
            .inspect(|i| command_is_numeric &= i.is_numeric())
            .collect::<String>();
        if command_str.len() + 1 != value.len() {
            // there are some unaccounted-for characters, don't accept this string
            return Err(L3XParseError::UnaccountedCharacters);
        }
        let command = if command_is_numeric {
            L3XCommand::Multiply(
                command_str
                    .parse()
                    .map_err(|_| L3XParseError::NumberOverflow)
                    .and_then(|i: u64| {
                        Registers::try_from(i).map_err(|_| L3XParseError::ZeroCommand)
                    })?,
            )
        } else {
            match command_str
                .chars()
                .next()
                .ok_or(L3XParseError::WrongLength)?
            {
                '%' => L3XCommand::Duplicate,
                '&' => L3XCommand::Queue,
                '~' => L3XCommand::Annihilate,
                _ => return Err(L3XParseError::BadCommand),
            }
        };

        Ok(Self::Some(L3X { direction, command }))
    }
}

impl ToString for L3X {
    fn to_string(&self) -> String {
        let mut out = match self.command {
            L3XCommand::Multiply(ref n) => format!("{n}"),
            L3XCommand::Duplicate => "%".to_string(),
            L3XCommand::Queue => "&".to_string(),
            L3XCommand::Annihilate => '~'.to_string(),
        };
        out.push(match self.direction {
            Direction::Up => 'U',
            Direction::Down => 'D',
            Direction::Left => 'L',
            Direction::Right => 'R',
        });
        out
    }
}

#[derive(Copy, Clone)]
pub enum Output {
    Major(Direction),
    Minor(Direction),
}

impl Output {
    fn direction(self) -> Direction {
        match self {
            Output::Major(di) | Output::Minor(di) => di,
        }
    }

    fn is_major(self) -> bool {
        matches!(self, Self::Major(_))
    }
}

#[derive(Clone, Copy, Eq, strum_macros::EnumDiscriminants)]
#[strum_discriminants(name(DIDiscriminants), vis())]
pub enum DrawInstructions {
    /// A minor line connecting the input (left) to the minor output (right)
    ToMinor(Direction, Direction),
    /// A major line connecting the input (left) to the major output (right)
    ToMajor(Direction, Direction),
    /// A major/minor line connecting the major side to the minor side (which is opposite to the
    /// given direction)
    MajorMinor(Direction),
    /// Feeds an input from the direction into the counterclockwise loop
    IntoLoop(Direction),
    /// Constructs a counterclockwise roundabout which feeds into this direction
    Loop(Direction),
}

impl PartialEq for DrawInstructions {
    fn eq(&self, other: &Self) -> bool {
        // std::mem::discriminant(self) == std::mem::discriminant(other)
        DIDiscriminants::from(self) == DIDiscriminants::from(other)
    }
}

impl PartialOrd for DrawInstructions {
    /// Ordered by when the instruction should be drawn relative to others. Lesser means drawn
    /// first, greater means drawn on the topmost layer.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (DIDiscriminants::from(self) as u8).partial_cmp(&(DIDiscriminants::from(other) as u8))
    }
}

impl Ord for DrawInstructions {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl DrawInstructions {
    fn draw(&self, cell_size: f32, offset: Vec2) {
        match self {
            DrawInstructions::ToMinor(_, _) => todo!(),
            DrawInstructions::ToMajor(_, _) => todo!(),
            DrawInstructions::MajorMinor(di) => {
                let (left, right) = if di.positive() {
                    (RED, GREEN)
                } else {
                    (GREEN, RED)
                };
                let center = offset + Vec2::splat(cell_size / 2.);
                let (p1, p2) = if di.horizontal() {
                    (
                        vec2(-cell_size / 2., 0.) + center,
                        vec2(cell_size / 2., 0.) + center,
                    )
                } else {
                    (
                        vec2(0., -cell_size / 2.) + center,
                        vec2(0., cell_size / 2.) + center,
                    )
                };
                //draw_line(p1.x, p1.y, center.x, center.y, 6.0, left);
                //draw_line(center.x, center.y, p2.x, p2.y, 6.0, right);
            }
            DrawInstructions::IntoLoop(_) => todo!(),
            DrawInstructions::Loop(_) => todo!(),
        }
    }
}

impl L3X {
    pub fn outputs(&self) -> ArrayVec<Output, 2> {
        match self.command {
            L3XCommand::Multiply(ref reg) if reg.is_one() => {
                arrayvec![Output::Major(self.direction)]
            }
            L3XCommand::Queue | L3XCommand::Annihilate => arrayvec![Output::Major(self.direction)],
            L3XCommand::Multiply(_) => arrayvec![
                Output::Major(self.direction),
                Output::Minor(self.direction.opposite()),
            ],
            L3XCommand::Duplicate => arrayvec![
                Output::Major(self.direction),
                Output::Major(self.direction.opposite()),
            ],
        }
    }
    pub fn inputs(
        &self,
        matrix: &HashMap<IVec2, L3X>,
        dims: UVec2,
        location: IVec2,
    ) -> ArrayVec<Direction, 4> {
        Direction::iter()
            .with_offsets(location)
            .filter_map(|(direction, location)| {
                location
                    .cmplt(dims.as_ivec2())
                    .all()
                    .then(|| {
                        matrix
                            .get(&location)
                            .map(|l3x| {
                                !l3x.outputs()
                                    .iter()
                                    .all(|o| o.direction() != direction.opposite())
                            })
                            .unwrap_or(false)
                            .then_some(direction)
                    })
                    .flatten()
            })
            .collect()
    }
    pub fn minor_is_active(
        &self,
        matrix: &HashMap<IVec2, L3X>,
        dims: UVec2,
        location: IVec2,
        recursion_depth: usize,
    ) -> bool {
        !self
            .active_inputs(matrix, dims, location, recursion_depth)
            .iter()
            .all(|d| d == &self.direction.opposite())
    }
    pub fn active_outputs(
        &self,
        matrix: &HashMap<IVec2, L3X>,
        dims: UVec2,
        location: IVec2,
        recursion_depth: usize,
    ) -> ArrayVec<Output, 2> {
        match self.command {
            L3XCommand::Multiply(ref reg) if !reg.is_one() => {
                if self.minor_is_active(matrix, dims, location, recursion_depth) {
                    arrayvec![
                        Output::Major(self.direction),
                        Output::Minor(self.direction.opposite()),
                    ]
                } else {
                    arrayvec![Output::Major(self.direction)]
                }
            }
            _ => self.outputs(),
        }
    }
    pub fn active_inputs(
        &self,
        matrix: &HashMap<IVec2, L3X>,
        dims: UVec2,
        location: IVec2,
        recursion_depth: usize,
    ) -> ArrayVec<Direction, 4> {
        if recursion_depth == 0 {
            return self.inputs(matrix, dims, location);
        }
        Direction::iter()
            .with_offsets(location)
            .filter_map(|(direction, location)| {
                location
                    .cmplt(dims.as_ivec2())
                    .all()
                    .then(|| {
                        matrix
                            .get(&location)
                            .map(|l3x| {
                                !l3x.active_outputs(matrix, dims, location, recursion_depth - 1)
                                    .iter()
                                    .all(|o| o.direction() != direction.opposite())
                            })
                            .unwrap_or(false)
                            .then_some(direction)
                    })
                    .flatten()
            })
            .collect()
    }
    pub fn is_one(&self) -> bool {
        matches!(self.command, L3XCommand::Multiply(ref reg) if reg.is_one())
    }

    pub fn draw_instructions(
        &self,
        matrix: &HashMap<IVec2, L3X>,
        dims: UVec2,
        location: IVec2,
    ) -> SmallVec<[DrawInstructions; 4]> {
        let inputs = Direction::iter()
            .with_offsets(location)
            .filter_map(|(direction, location)| {
                location
                    .cmplt(dims.as_ivec2())
                    .all()
                    .then(|| {
                        matrix
                            .get(&location)
                            .map(|l3x| l3x.direction == direction.opposite())
                            .unwrap_or(false)
                            .then_some(direction)
                    })
                    .flatten()
            })
            .collect::<SmallVec<[_; 4]>>();

        let outputs = self.outputs();

        let mut v = smallvec![];

        match self.command {
            L3XCommand::Multiply(ref x) if x.is_one() => (),
            L3XCommand::Multiply(_) => {
                if outputs.iter().any(|o| inputs.contains(&o.direction())) {
                    v.push(DrawInstructions::MajorMinor(self.direction))
                }
            }
            L3XCommand::Duplicate => (),
            L3XCommand::Queue => (),
            L3XCommand::Annihilate => (),
        }

        v.sort();
        v
    }

    pub fn draw(
        &self,
        matrix: &HashMap<IVec2, L3X>,
        dims: UVec2,
        location: IVec2,
        cell_size: f32,
        offset: Vec2,
        font_size: f32,
        primary_color: Color,
    ) {
        let text_offset = vec2(0.05, 0.67) * cell_size;
        let lower = (location.as_vec2() * cell_size) + offset;

        //collect input directions
        let inputs = self.active_inputs(matrix, dims, location, 2);
        let outputs = self.active_outputs(matrix, dims, location, 2);

        // TODO represent cell contents graphically
        draw_text(
            &self.to_string(),
            (lower + text_offset).x,
            (lower + text_offset).y,
            font_size,
            primary_color,
        );
        let minor_color = RED;

        let out_arrow_vertices = vec![
            vec2(-0., 0.75),
            vec2(-0.25, 1.0),
            vec2(-0.5, 0.75),
            vec2(-0.3, 0.75),
            vec2(-0.3, 0.25),
            vec2(0., 0.),
            vec2(-0.2, 0.25),
            vec2(-0.2, 0.75),
        ];
        let in_arrow_vertices = vec![
            vec2(0.3, 1.0),
            vec2(0.2, 1.0),
            vec2(0.2, 0.5),
            vec2(0., 0.5),
            vec2(0., 0.),
            vec2(0.1, 0.),
            vec2(0.1, 0.4),
            vec2(0.3, 0.4),
        ];
        let through_vertices = vec![
            vec2(0.3, 1.0),
            vec2(0.2, 1.0),
            vec2(0.2, -0.2),
            vec2(0.3, -0.2),
        ];
        let out_arrow_triangulation = triangulate_indices(&out_arrow_vertices);
        let in_arrow_triangulation = triangulate_indices(&in_arrow_vertices);
        let through_triangulation = triangulate_indices(&through_vertices);
        // let in_arrow_triangles: Vec<[Vec2; 3]> = triangulate(in_arrow_vertices);
        // let through_triangles = triangulate(through_vertices);
        for output in outputs {
            let out_color = if self.is_one() {
                GRAY
            } else if output.is_major() {
                GREEN
            } else {
                minor_color
            };
            let vertices = out_arrow_vertices
                .iter()
                .map(|&v| {
                    (Mat2::from(output.direction()) * v + Vec2::splat(1.)) * cell_size / 2. + lower
                })
                .map(|u| macroquad::models::Vertex {
                    position: u.extend(0.),
                    uv: u,
                    color: out_color,
                })
                .collect_vec();

            draw_mesh(&Mesh {
                vertices,
                indices: out_arrow_triangulation.clone(),
                texture: None,
            });
        }
        for input in inputs {
            let in_color = if self.is_one() {
                GRAY
            } else if input == self.direction.opposite() {
                BLUE
            } else {
                BROWN
            };
            let (vertices, indices) = (if input == self.direction.opposite() {
                // &through_triangles
                (through_vertices.clone(), through_triangulation.clone())
            } else {
                (in_arrow_vertices.clone(), in_arrow_triangulation.clone())
            })
            .pipe(|(vertices, triangulation)| {
                (
                    vertices
                        .into_iter()
                        .map(|v| (Mat2::from(input) * v + Vec2::splat(1.)) * cell_size / 2. + lower)
                        .map(|u| macroquad::models::Vertex {
                            position: u.extend(0.),
                            uv: u,
                            color: in_color,
                        })
                        .collect(),
                    triangulation,
                )
            });

            draw_mesh(&Mesh {
                vertices,
                indices,
                texture: None,
            })
        }

        for instr in self.draw_instructions(matrix, dims, location) {
            instr.draw(cell_size, lower)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l3_simple() {
        assert_eq!(
            L3X::try_from("3L"),
            Ok(L3X {
                direction: Direction::Left,
                command: L3XCommand::Multiply(Registers([(3, 1)].into_iter().collect()))
            })
        )
    }
}
