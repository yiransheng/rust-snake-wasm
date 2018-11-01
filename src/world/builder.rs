use alloc::vec::Vec;
use std::convert::From;
use std::marker::PhantomData;

use rand::{Rng, SeedableRng};

use data::{
    Block, BoundingBehavior, Coordinate, Direction, Grid, Natnum, Wrapping,
};

use super::{SnakeIter, SnakeState, World};

#[derive(Copy, Clone)]
pub struct WorldBuilder<BB: BoundingBehavior = Wrapping> {
    width: Natnum,
    height: Natnum,

    _bounding_behavior: PhantomData<BB>,
}

impl<BB: BoundingBehavior> WorldBuilder<BB> {
    pub fn new() -> Self {
        WorldBuilder {
            width: 10,
            height: 10,

            _bounding_behavior: PhantomData,
        }
    }
    pub fn width<'a>(&'a mut self, width: Natnum) -> &'a mut Self {
        self.width = width;
        self
    }
    pub fn height(&mut self, height: Natnum) -> &mut Self {
        self.height = height;
        self
    }
    pub fn set_snake(self, x: Natnum, y: Natnum) -> SnakeBuilder<BB> {
        assert!(x < self.width && y < self.height);

        let grid = Grid::empty(self.width, self.height);
        let tail = Coordinate { x, y };

        SnakeBuilder {
            grid,
            head: tail,
            tail,
            next_head: tail,
            snake_len: 0,

            _bounding_behavior: PhantomData,
        }
    }
}

pub struct SnakeBuilder<BB: BoundingBehavior> {
    grid: Grid,
    head: Coordinate,
    next_head: Coordinate,
    tail: Coordinate,
    snake_len: u32,

    _bounding_behavior: PhantomData<BB>,
}

impl<BB: BoundingBehavior> SnakeBuilder<BB> {
    pub fn extend(mut self, dir: Direction) -> Self {
        let next_head = self.next_head;
        let next_head_block = self.grid[next_head];

        assert!(next_head_block.is_empty());

        self.grid[next_head] = Block::from(dir);

        self.snake_len += 1;

        if self.snake_len == 1 {
            // first block
            self.tail = next_head;
        }
        self.head = next_head;

        self.next_head = next_head
            .move_towards(dir)
            .bound_inside(self.grid.width(), self.grid.height())
            .unwrap();

        self
    }

    pub fn build_with_seed<R: Rng + SeedableRng>(
        self,
        seed: R::Seed,
    ) -> World<R, BB> {
        assert!(self.snake_len > 1);

        let rng = R::from_seed(seed);

        let initial_snake: Vec<(Coordinate, Direction)>;

        {
            let iter: SnakeIter<BB> = SnakeIter::new(&self.grid, self.tail);

            initial_snake = iter.collect();
        }

        World {
            grid: self.grid,

            state: SnakeState::Eaten,

            tail: self.tail,
            head: self.head,

            initial_snake,
            rng,

            _bounding_behavior: PhantomData,
        }
    }
}
