use rand::{Rng, SeedableRng};
use std::convert::{From, Into};
use std::iter::Map;

use data::{Block, Coordinate, Direction, Tile};
use system::Model;

use super::grid::Grid;
use super::{SnakeIter, SnakeState, World};

#[derive(Copy, Clone)]
pub struct WorldBuilder {
    width: u32,
    height: u32,
}

impl WorldBuilder {
    pub fn new() -> Self {
        WorldBuilder {
            width: 10,
            height: 10,
        }
    }
    pub fn width<'a>(&'a mut self, width: u32) -> &'a mut Self {
        self.width = width;
        self
    }
    pub fn height(&mut self, height: u32) -> &mut Self {
        self.height = height;
        self
    }
    pub fn set_snake(self, x: u32, y: u32) -> SnakeBuilder {
        assert!(x < self.width && y < self.height);

        let grid = Grid::empty(self.width, self.height);
        let tail = Coordinate::new_unchecked(x, y);

        SnakeBuilder {
            grid,
            head: tail,
            tail,
            next_head: tail,
            snake_len: 0,
        }
    }
}

pub struct SnakeBuilder {
    grid: Grid,
    head: Coordinate,
    next_head: Coordinate,
    tail: Coordinate,
    snake_len: u32,
}

impl SnakeBuilder {
    pub fn extend(mut self, dir: Direction) -> Self {
        let next_head = self.next_head;
        let next_head_block = self.grid.get_block(next_head);

        assert!(Tile::from(next_head_block) == Tile::Empty);

        if !self.grid.set_block(next_head, Block::from(dir)) {
            return self;
        }

        self.snake_len += 1;

        if self.snake_len == 1 {
            // first block
            self.tail = next_head;
        }
        self.head = next_head;

        self.next_head = next_head
            .move_towards(dir)
            .into_coordinate(self.grid.width(), self.grid.height())
            .unwrap();

        self
    }
    pub fn build_with_seed<R: Rng + SeedableRng>(
        self,
        seed: R::Seed,
    ) -> World<R> {
        assert!(self.snake_len > 1);

        let rng = R::from_seed(seed);

        let initial_snake: Vec<(Coordinate, Direction)>;

        {
            let iter = SnakeIter::new(&self.grid, self.tail);

            initial_snake = iter.collect();
        }

        World {
            grid: self.grid,

            state: SnakeState::Eaten,

            tail: self.tail,
            head: self.head,

            initial_snake,
            rng,
        }
    }
}
