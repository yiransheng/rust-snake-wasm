use std::convert::{From, Into};
use std::mem::transmute;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Key {
    code: u8,
}

impl Key {
    pub fn new(code: u8) -> Self {
        Key { code }
    }

    pub fn is_direction_key(self) -> bool {
        let dir: Option<Direction> = self.into();
        dir.is_some()
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
    pub unsafe fn from_raw(raw: u8) -> Self {
        Block { raw }
    }
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
    // this is memory safe but calling it in wrong place
    // may result in logic errors
    #[inline]
    pub fn into_direction_unchecked(self) -> Direction {
        // keep only lower 2 bits, then turn on first bit
        let bits = self.raw & 0b0000_0011 | 0b1000_0000;
        unsafe { transmute(bits) }
    }

    #[inline]
    pub fn into_direction(self) -> Option<Direction> {
        if self.is_snake() {
            Some(self.into_direction_unchecked())
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

impl Coordinate {
    pub fn new_unchecked(x: u32, y: u32) -> Self {
        Coordinate { x, y }
    }
    pub fn move_towards(self, dir: Direction) -> UncheckedCoordinate {
        let Coordinate { x, y } = self;
        match dir {
            Direction::North => UncheckedCoordinate::new(x, y.wrapping_sub(1)),
            Direction::South => UncheckedCoordinate::new(x, y.wrapping_add(1)),
            Direction::East => UncheckedCoordinate::new(x.wrapping_add(1), y),
            Direction::West => UncheckedCoordinate::new(x.wrapping_sub(1), y),
        }
    }
}

pub struct UncheckedCoordinate {
    inner: Coordinate,
}
impl UncheckedCoordinate {
    #[inline(always)]
    fn new(x: u32, y: u32) -> Self {
        UncheckedCoordinate {
            inner: Coordinate { x, y },
        }
    }

    #[inline(always)]
    pub fn unwrap_unchecked(self) -> Coordinate {
        self.inner
    }

    pub fn into_coordinate(
        self,
        bound_width: u32,
        bound_height: u32,
    ) -> Option<Coordinate> {
        if self.inner.x < bound_width && self.inner.y < bound_height {
            Some(self.inner)
        } else {
            None
        }
    }

    pub fn into_coordinate_wrapping(
        self,
        bound_width: u32,
        bound_height: u32,
    ) -> Coordinate {
        let Coordinate { x, y } = self.inner;

        Coordinate {
            x: x.wrapping_add(bound_width) % bound_width,
            y: y.wrapping_add(bound_height) % bound_height,
        }
    }
}

// conversions

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

impl From<Direction> for Block {
    fn from(dir: Direction) -> Self {
        Block { raw: (dir as u8) }
    }
}

impl Into<Option<Direction>> for Block {
    #[inline]
    fn into(self) -> Option<Direction> {
        self.into_direction()
    }
}

impl From<u8> for Key {
    fn from(code: u8) -> Key {
        Key { code }
    }
}

impl Default for Key {
    fn default() -> Self {
        Key { code: 0 }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converion_safety() {
        for x in 0..=u8::max_value() {
            // make sure every u8 bit pattern result in a valid Direction
            // ...that is, not getting SIGIL or something
            unsafe {
                let b = Block::from_raw(x);
                let tile = Tile::from(b);

                if b.is_snake() {
                    assert_eq!(tile, Tile::Snake);
                }

                let dir = b.into_direction_unchecked();

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
