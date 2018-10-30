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
    North,
    South,
    East,
    West,
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

#[derive(Debug, Copy, Clone)]
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
        debug_assert!(bound_width > 0 && bound_height > 0);

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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct Bound {
        width: u32,
        height: u32,
    }

    impl Arbitrary for Bound {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let w = u32::arbitrary(g);
            let h = u32::arbitrary(g);

            Bound {
                width: if w > 0 { w } else { 1 },
                height: if h > 0 { h } else { 1 },
            }
        }
    }

    impl Arbitrary for Direction {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match g.size() % 4 {
                0 => Direction::East,
                1 => Direction::West,
                2 => Direction::South,
                3 => Direction::North,
                _ => unreachable!(),
            }
        }
    }
    impl Arbitrary for Coordinate {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Coordinate {
                x: u32::arbitrary(g),
                y: u32::arbitrary(g),
            }
        }
    }
    impl Arbitrary for UncheckedCoordinate {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            UncheckedCoordinate {
                inner: Coordinate::arbitrary(g),
            }
        }
    }

    #[test]
    fn test_block_size() {
        // nested enum optimization kicking in
        assert_eq!(::std::mem::size_of::<Block>(), 1,)
    }

    quickcheck! {
        fn double_opposite_is_identity(dir: Direction) -> bool {
            dir.opposite().opposite() == dir
        }

        fn opposite_direction_moves_cancel(inputs: (UncheckedCoordinate,Direction,Bound)) -> bool {
            let (coord, dir, Bound { width, height }) = inputs;

            // make sure starting coord is inside bound
            let coord = coord.wrap_inside(width, height);
            let orig_coord = coord;

            let coord = coord.move_towards(dir).wrap_inside(width, height);
            let coord = coord.move_towards(dir.opposite()).wrap_inside(width, height);

            coord == orig_coord
        }

        fn coord_returns_after_ENWS_moves(inputs: (UncheckedCoordinate, Bound)) -> bool {
            let (coord, Bound { width, height }) = inputs;

            // make sure starting coord is inside bound
            let mut coord = coord.wrap_inside(width, height);
            let orig_coord = coord;

            coord = coord.move_towards(Direction::East).wrap_inside(width, height);
            coord = coord.move_towards(Direction::North).wrap_inside(width, height);
            coord = coord.move_towards(Direction::West).wrap_inside(width, height);
            coord = coord.move_towards(Direction::South).wrap_inside(width, height);

            coord == orig_coord
        }

        fn coord_wrap_is_identity_if_already_inside(inputs: (UncheckedCoordinate, Bound)) -> bool {
            let (coord, bound) = inputs;
            let Bound { width, height } = bound;

            if let Some(bounded) = coord.bound_inside(width, height) {
                coord.wrap_inside(width, height) == bounded
            } else {
                true
            }
        }
    }
}
