use std::collections::HashMap;

use macroquad::prelude::*;
use smallvec::{smallvec, SmallVec};
use strum::IntoEnumIterator;

use crate::registers::Registers;

#[derive(PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Debug)]
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

pub enum MaybeL3X {
    Some(L3X),
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

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
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
        let reflect = mat2(vec2(0., 1.), vec2(1., 0.));
        match value {
            Direction::Up => -Mat2::IDENTITY,
            Direction::Down => Mat2::IDENTITY,
            Direction::Left => -reflect,
            Direction::Right => reflect,
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

impl L3X {
    pub fn outputs(&self) -> SmallVec<[Output; 2]> {
        match self.command {
            L3XCommand::Multiply(ref reg) if reg.is_one() => {
                smallvec![Output::Major(self.direction)]
            }
            L3XCommand::Queue | L3XCommand::Annihilate => smallvec![Output::Major(self.direction)],
            L3XCommand::Multiply(_) => smallvec![
                Output::Major(self.direction),
                Output::Minor(self.direction.opposite()),
            ],
            L3XCommand::Duplicate => smallvec![
                Output::Major(self.direction),
                Output::Major(self.direction.opposite()),
            ],
        }
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

        // collect input directions
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

        // TODO represent cell contents graphically
        draw_text(
            &self.to_string(),
            (lower + text_offset).x,
            (lower + text_offset).y,
            font_size,
            primary_color,
        );

        let triangle_vertices = [vec2(0.0, 1.0), vec2(-0.25, 0.75), vec2(0.25, 0.75)];

        for output in outputs {
            let color = if output.is_major() { GREEN } else { RED };
            let triangle_vertices = triangle_vertices
                .map(|v| (Mat2::from(output.direction()) * v + Vec2::splat(1.)) * cell_size / 2. + lower );
            draw_triangle(
                triangle_vertices[0],
                triangle_vertices[1],
                triangle_vertices[2],
                color,
            );
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
