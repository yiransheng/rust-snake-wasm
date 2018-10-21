use std::convert::{From, Into};
use std::mem::transmute;

use system::{Either, GameInput, StartGame};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    North = 0b1000_0000,
    South = 0b1000_0001,
    East = 0b1000_0010,
    West = 0b1000_0011,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Key {
    code: u8,
    pressed_frames: u8,
}

impl Key {
    pub fn new(code: u8, pressed_frames: u8) -> Self {
        Key {
            code,
            pressed_frames,
        }
    }

    pub fn modify_animation_frame(self, base_frame: u8) -> u8 {
        let frames_to_reduce = self.pressed_frames / 10;
        let min_frame = 1;

        if frames_to_reduce >= base_frame + min_frame {
            min_frame
        } else {
            base_frame - frames_to_reduce
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Snake,
    Food,
    OutOfBound,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Block {
    raw: u8,
}

impl Block {
    #[inline]
    pub fn empty() -> Self {
        Block { raw: 0 }
    }
    #[inline]
    pub fn food() -> Self {
        // second bit on: food
        Block { raw: 0b0100_0000 }
    }
    #[inline]
    pub fn out_of_bound() -> Self {
        // first two bits off: OOB
        Block { raw: 0b0011_1111 }
    }
    #[inline]
    pub fn from_raw(raw: u8) -> Self {
        Block { raw }
    }
    #[inline]
    pub fn into_raw(self) -> u8 {
        self.raw
    }
    #[inline]
    pub fn is_empty(self) -> bool {
        self.raw == 0
    }
    #[inline]
    pub fn is_food(self) -> bool {
        // second bit is 1
        self.raw & 0b0100_0000 != 0
    }
    #[inline]
    pub fn is_snake(self) -> bool {
        // first bit is 1
        self.raw & 0b1000_0000 != 0
    }
    #[inline(always)]
    unsafe fn into_direction(self) -> Direction {
        // keep only lower 2 bits, then turn on first bit
        let bits = self.raw & 0b0000_0011 | 0b1000_0000;
        transmute(bits)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn new(x: i32, y: i32) -> Self {
        Coordinate { x, y }
    }
    pub fn from_usize(x: usize, y: usize) -> Self {
        Coordinate {
            x: x as i32,
            y: y as i32,
        }
    }
    pub fn move_towards(self, dir: Direction) -> Self {
        let Coordinate { x, y } = self;
        match dir {
            Direction::North => Coordinate { x, y: y - 1 },
            Direction::South => Coordinate { x, y: y + 1 },
            Direction::East => Coordinate { x: x + 1, y },
            Direction::West => Coordinate { x: x - 1, y },
        }
    }
}

// conversions

impl Into<Block> for u8 {
    fn into(self) -> Block {
        Block::from_raw(self)
    }
}

impl From<Block> for Tile {
    fn from(b: Block) -> Tile {
        match b.raw {
            0 => Tile::Empty,
            b if (b & 0b1000_0000) != 0 => Tile::Snake,
            b if (b & 0b0100_0000) != 0 => Tile::Food,
            _ => Tile::OutOfBound,
        }
    }
}

impl From<Block> for Direction {
    fn from(b: Block) -> Direction {
        debug_assert!(b.is_snake());

        unsafe { b.into_direction() }
    }
}
impl From<Direction> for Block {
    fn from(dir: Direction) -> Self {
        Block { raw: (dir as u8) }
    }
}

impl From<u8> for Key {
    fn from(code: u8) -> Key {
        Key {
            code,
            pressed_frames: 1,
        }
    }
}

impl From<Key> for Option<Direction> {
    fn from(k: Key) -> Option<Direction> {
        match k.code {
            37 => Some(Direction::West),
            38 => Some(Direction::North),
            39 => Some(Direction::East),
            40 => Some(Direction::South),
            _ => None,
        }
    }
}

impl<T> Into<GameInput<T>> for Key
where
    T: From<Key>,
{
    fn into(self) -> GameInput<T> {
        match self.code {
            13 => Either::Left(StartGame),
            _ => Either::Right(T::from(self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converion() {
        for x in 0..u8::max_value() {
            let b = Block::from_raw(x);
            let tile = Tile::from(b);

            if b.is_snake() {
                assert_eq!(tile, Tile::Snake);
            }

            // make sure every u8 bit pattern result in a valid Direction
            // ...that is, not getting SIGIL or something
            unsafe {
                let dir = b.into_direction();
                assert!(
                    dir == Direction::North
                        || dir == Direction::South
                        || dir == Direction::East
                        || dir == Direction::West
                );
            }
        }
    }
}
