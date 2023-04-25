pub struct L3X {
    direction: Direction,
    command: L3XCommand,
}

#[rustfmt::skip]
pub enum Direction {
    Up, Down, Left, Right
}

pub enum L3XCommand {
    Number(u32),
    Duplicate,
    Queue,
    Annihilate,
}

impl From<&str> for L3X {
    fn from(value: &str) -> Self {
        todo!()
    }
}