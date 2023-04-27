use macroquad::prelude::*;

use crate::traveler::Registers;

#[derive(PartialEq, Eq, Debug)]
pub struct L3X {
    // TODO support watch points
    pub direction: Direction,
    pub command: L3XCommand,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum Direction {
    Up, Down, Left, Right,
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
        let value = value.trim();

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

        Ok(Self { direction, command })
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
