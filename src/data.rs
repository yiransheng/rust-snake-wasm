use alloc::vec::Vec;
use std::cmp::{max, Ordering};
use std::convert::{From, Into};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use morton::{deinterleave_morton, interleave_morton};
use rand::Rng;

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

pub type Natnum = u16;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Coordinate {
    pub x: Natnum,
    pub y: Natnum,
}

impl Coordinate {
    pub fn new_unchecked(x: Natnum, y: Natnum) -> Self {
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

    #[inline(always)]
    pub fn as_usize(self) -> usize {
        interleave_morton(self.x, self.y) as usize
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.x.cmp(&other.x), self.y.cmp(&other.y)) {
            (Ordering::Equal, o) => Some(o),
            (o, Ordering::Equal) => Some(o),
            (ox, oy) if ox == oy => Some(ox),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UncheckedCoordinate {
    inner: Coordinate,
}
impl UncheckedCoordinate {
    #[inline(always)]
    fn new(x: Natnum, y: Natnum) -> Self {
        UncheckedCoordinate {
            inner: Coordinate { x, y },
        }
    }

    pub fn unwrap(self) -> Coordinate {
        self.inner
    }

    pub fn bound_inside(
        self,
        bound_width: Natnum,
        bound_height: Natnum,
    ) -> Option<Coordinate> {
        if self.inner.x < bound_width && self.inner.y < bound_height {
            Some(self.inner)
        } else {
            None
        }
    }

    pub fn wrap_inside(
        self,
        bound_width: Natnum,
        bound_height: Natnum,
    ) -> Coordinate {
        debug_assert!(bound_width > 0 && bound_height > 0);

        let Coordinate { x, y } = self.inner;

        Coordinate {
            x: x.wrapping_add(bound_width) % bound_width,
            y: y.wrapping_add(bound_height) % bound_height,
        }
    }
}

pub struct Grid {
    blocks: Vec<Block>,
    width: Natnum,
    height: Natnum,
}

impl Grid {
    pub fn empty(width: Natnum, height: Natnum) -> Self {
        let width = max(1, width);
        let height = max(1, height);

        let max_coord = Coordinate {
            x: width - 1,
            y: height - 1,
        };
        let size_requirement = max_coord.as_usize() + 1;

        let mut blocks = vec![Block::OutOfBound; size_requirement];

        for x in 0..width {
            for y in 0..height {
                let index = Coordinate { x, y }.as_usize();
                blocks[index] = Block::Empty;
            }
        }

        Grid {
            width: width,
            height: height,
            blocks,
        }
    }

    #[inline(always)]
    pub fn width(&self) -> Natnum {
        self.width
    }
    #[inline(always)]
    pub fn height(&self) -> Natnum {
        self.height
    }

    pub fn random_coordinate<R: Rng>(&self, rng: &mut R) -> Coordinate {
        let x = rng.gen_range(0, self.width);
        let y = rng.gen_range(0, self.height);

        Coordinate { x, y }
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (Coordinate, Block)> + 'a {
        self.iter_coordinates()
            .map(move |coord| (coord, self[coord]))
    }

    pub fn clear(&mut self) {
        self.iter_coordinates().for_each(|coord| {
            self[coord] = Block::Empty;
        });
    }
}

impl Grid {
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        iproduct!(0..self.width, 0..self.height)
            .map(|(x, y)| Coordinate { x, y })
    }
}

// traits implements

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

impl Index<Coordinate> for Grid {
    type Output = Block;

    fn index<'a>(&'a self, index: Coordinate) -> &'a Block {
        if index.x < self.width && index.y < self.height {
            &self.blocks[index.as_usize()]
        } else {
            &Block::OutOfBound
        }
    }
}
impl IndexMut<Coordinate> for Grid {
    fn index_mut<'a>(&'a mut self, index: Coordinate) -> &'a mut Block {
        if index.x < self.width && index.y < self.height {
            &mut self.blocks[index.as_usize()]
        } else {
            panic!("Accessing out of bound block")
        }
    }
}

impl FromIterator<(Coordinate, Block)> for Grid {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Coordinate, Block)>,
    {
        let mut blocks = vec![Block::Empty];
        let mut x_max: Natnum = 0;
        let mut y_max: Natnum = 0;

        for (coord, block) in iter {
            let index = coord.as_usize();

            for _ in blocks.len()..index {
                blocks.push(Block::Empty);
            }
            x_max = max(x_max, coord.x);
            y_max = max(y_max, coord.y);

            blocks[index] = block;
        }

        let width = x_max + 1;
        let height = y_max + 1;

        blocks.iter_mut().enumerate().for_each(|(index, block)| {
            let (x, y) = deinterleave_morton(index as u32);
            if x > x_max || y > y_max {
                *block = Block::OutOfBound;
            }
        });

        Grid {
            width,
            height,
            blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct Bound {
        width: Natnum,
        height: Natnum,
    }

    impl Arbitrary for Bound {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let w = Natnum::arbitrary(g);
            let h = Natnum::arbitrary(g);

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
                x: Natnum::arbitrary(g),
                y: Natnum::arbitrary(g),
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

        fn coordinate_usize_preserves_partial_order(inputs: (Coordinate, Coordinate)) -> bool {
            let (a, b) = inputs;

            if let Some(ordering) = a.partial_cmp(&b) {
                ordering == a.as_usize().cmp(&b.as_usize())
            } else {
                true
            }
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
