use std::convert::{From, Into};

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
pub enum Block<T = Direction> {
    Empty,
    Snake(T),
    Food,
    OutOfBound,
}

impl<T> Block<T> {
    pub fn is_empty(self) -> bool {
        match self {
            Block::Empty => true,
            _ => false,
        }
    }
    pub fn is_snake(self) -> bool {
        match self {
            Block::Snake(_) => true,
            _ => false,
        }
    }

    pub fn snake(self) -> Option<T> {
        match self {
            Block::Snake(s) => Some(s),
            _ => None,
        }
    }

    pub fn snake_or_err<E>(self, err: E) -> Result<T, E> {
        match self {
            Block::Snake(s) => Ok(s),
            _ => Err(err),
        }
    }

    #[allow(dead_code)]
    pub fn map_snake<U, F>(self, f: F) -> Block<U>
    where
        F: Fn(T) -> U,
    {
        match self {
            Block::Snake(x) => Block::Snake(f(x)),
            Block::Empty => Block::Empty,
            Block::Food => Block::Food,
            Block::OutOfBound => Block::OutOfBound,
        }
    }
}

impl Block<Direction> {
    pub fn into_direction(self) -> Option<Direction> {
        match self {
            Block::Snake(dir) => Some(dir),
            _ => None,
        }
    }
    pub fn into_direction_unchecked(self) -> Direction {
        match self {
            Block::Snake(dir) => dir,
            _ => panic!(),
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

    pub fn unwrap(self) -> Coordinate {
        self.inner
    }

    pub fn bound_inside(
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

    pub fn wrap_inside(
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

impl From<Direction> for Block {
    fn from(dir: Direction) -> Block {
        Block::Snake(dir)
    }
}

/*
 * #[cfg(test)]
 * mod tests {
 *     use super::*;
 *
 *     #[test]
 *     fn test_converion_safety() {
 *         for x in 0..=u8::max_value() {
 *             // make sure every u8 bit pattern result in a valid Direction
 *             // ...that is, not getting SIGIL or something
 *             let b = unsafe { Block::from_raw(x) };
 *             let tile = Tile::from(b);
 *
 *             if b.is_snake() {
 *                 assert_eq!(tile, Tile::Snake(()));
 *             }
 *
 *             let dir = b.into_direction_unchecked();
 *
 *             assert!(
 *                 dir == Direction::North
 *                     || dir == Direction::South
 *                     || dir == Direction::East
 *                     || dir == Direction::West
 *             );
 *         }
 *     }
 * }
 */
