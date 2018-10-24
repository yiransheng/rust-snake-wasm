use rand::{Rng, SeedableRng};
use std::convert::{From, Into};
use std::iter::Map;

use data::{Block, Coordinate, Direction, Tile};
use system::Model;

mod grid;
use self::grid::Grid;

type Result<T> = ::std::result::Result<T, UpdateError>;

// side effect of a world update
#[derive(Debug, Copy, Clone)]
pub enum WorldUpdate {
    SetBlock { block: Block, at: Coordinate },
    Clear { prev_block: Block, at: Coordinate },
    SetWorldSize(u32, u32),
}

pub enum UpdateError {
    OutOfBound,
    CollideBody,
}

enum SnakeState {
    Eaten,
    Consuming(Tile),
}

pub struct World<R> {
    grid: Grid,
    state: SnakeState,

    head: Coordinate,
    tail: Coordinate,

    initial_snake: Vec<(Coordinate, Direction)>,
    rng: R,
}

// impl<'a, R: Rng> Model<'a> for World<R> {
// type Cmd = Direction;
// type Update = WorldUpdate;
// type State =
// }

impl<R: Rng> World<R> {
    fn set_direction(&mut self, dir: Direction) {
        let head = self.head;
        let head_dir: Direction = self.grid.get_block(head).into();

        if dir != head_dir.opposite() {
            self.set_block(head, dir);
        }
    }

    fn reset(&mut self) {
        self.grid.clear();

        let initial_snake = ::std::mem::replace(&mut self.initial_snake, vec![]);
        let n = initial_snake.len();

        for (i, (at, dir)) in initial_snake.iter().enumerate() {
            if i == 0 {
                self.tail = *at;
            }
            if i == n - 1 {
                self.head = *at;
            }
            self.set_block(*at, *dir);
        }

        self.initial_snake = initial_snake;
    }

    fn motion(&mut self) -> Result<Tile> {
        let head_block = self.get_block(self.head);
        let next_head = self.head.move_towards(head_block.into());
        let next_head_tile: Tile = self.get_block(next_head).into();

        match next_head_tile {
            Tile::Empty | Tile::Food => {
                self.head = next_head;
                self.set_block(next_head, head_block);
                Ok(next_head_tile)
            }
            Tile::Snake => Err(UpdateError::CollideBody),
            Tile::OutOfBound => Err(UpdateError::OutOfBound),
        }
    }

    fn digest(&mut self, tile: Tile) -> Result<WorldUpdate> {
        match tile {
            Tile::Empty => {
                let tail = self.tail;
                let tail_block = self.get_block(tail);
                let next_tail = tail.move_towards(tail_block.into());

                self.tail = next_tail;

                Ok(WorldUpdate::Clear {
                    prev_block: tail_block,
                    at: tail,
                })
            }
            Tile::Food => {
                let coord = self.spawn_food();

                Ok(WorldUpdate::SetBlock {
                    block: Block::food(),
                    at: coord,
                })
            }
            Tile::Snake => Err(UpdateError::CollideBody),
            Tile::OutOfBound => Err(UpdateError::OutOfBound),
        }
    }

    fn spawn_food(&mut self) -> Coordinate {
        loop {
            let coord = self.grid.random_coordinate(&mut self.rng);
            let current_tile = Tile::from(self.grid.get_block(coord));

            if current_tile == Tile::Empty {
                self.set_block(coord, Block::food());
                return coord;
            }
        }
    }

    #[inline(always)]
    fn get_block(&self, coord: Coordinate) -> Block {
        self.grid.get_block(coord)
    }

    #[inline]
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        self.grid.set_block(coord, b.into());
    }

    #[inline]
    fn iter_snake(&self) -> SnakeIter {
        SnakeIter::new(&self.grid, self.tail)
    }
}

struct SnakeIter<'a> {
    grid: &'a Grid,
    at: Coordinate,
}

impl<'a> SnakeIter<'a> {
    fn new(grid: &'a Grid, at: Coordinate) -> Self {
        SnakeIter { grid, at }
    }
}

impl<'a> Iterator for SnakeIter<'a> {
    type Item = (Coordinate, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.grid.get_block(self.at);

        if block.is_snake() {
            let dir = Direction::from(block);
            let current = self.at;
            let next = current.move_towards(dir);
            self.at = next;
            Some((current, dir))
        } else {
            None
        }
    }
}

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
    pub fn set_snake(self, x: i32, y: i32) -> SnakeBuilder {
        let grid = Grid::empty(self.width, self.height);
        let tail = Coordinate::new(x, y);

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
        self.next_head = next_head.move_towards(dir);

        self
    }
    pub fn build_with_seed<R: Rng + SeedableRng>(self, seed: R::Seed) -> World<R> {
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
