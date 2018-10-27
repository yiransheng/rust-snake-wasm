use rand::{Rng, SeedableRng};
use std::convert::{From, Into};
use std::iter::Map;

use data::{Block, Coordinate, Direction, Tile};
use system::{GameOver, Model};

pub use self::builder::WorldBuilder;

use self::grid::Grid;

mod builder;
mod grid;

type Result<T> = ::std::result::Result<T, UpdateError>;

// side effect of a world update
#[derive(Debug, Copy, Clone)]
pub enum WorldUpdate {
    SetBlock { block: Block, at: Coordinate },
    Clear { prev_block: Block, at: Coordinate },
    SetWorldSize(u32, u32),
}

#[derive(Debug, Copy, Clone)]
pub enum UpdateError {
    OutOfBound,
    CollideBody,
}

impl Into<GameOver> for UpdateError {
    fn into(self) -> GameOver {
        GameOver
    }
}

#[derive(Debug, Copy, Clone)]
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

impl<'a, R: Rng + 'a> Model<'a> for World<R> {
    type Cmd = Direction;
    type Update = WorldUpdate;
    type State = Initializer<'a, R>;

    type Error = UpdateError;

    fn initialize(&'a mut self) -> Self::State {
        let food_at = self.spawn_food();

        Initializer::WorldSize(&*self, food_at)
    }

    #[inline(always)]
    fn tear_down(&mut self) {
        self.reset();
    }

    fn step(&mut self, cmd: Option<Self::Cmd>) -> Result<Option<Self::Update>> {
        if let Some(dir) = cmd {
            self.set_direction(dir);
            // } else {
            // return Ok(None);
        }

        match self.state {
            SnakeState::Eaten => {
                let block = self.motion();
                let block = block?;
                self.state = SnakeState::Consuming(block.into());

                Ok(Some(WorldUpdate::SetBlock {
                    block: self.get_block(self.head),
                    at: self.head,
                }))
            }
            SnakeState::Consuming(tile) => {
                let r = self.digest(tile)?;
                self.state = SnakeState::Eaten;
                Ok(Some(r))
            }
        }
    }
}

impl<R: Rng> World<R> {
    fn set_direction(&mut self, dir: Direction) -> bool {
        let head = self.head;
        let head_dir: Direction =
            self.get_block(head).into_direction_unchecked();

        if dir != head_dir.opposite() && dir != head_dir {
            self.set_block(head, dir);
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.grid.clear();

        let initial_snake =
            ::std::mem::replace(&mut self.initial_snake, vec![]);
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

    fn motion(&mut self) -> Result<Block> {
        let head_block = self.get_block(self.head);
        let next_head = self
            .head
            .move_towards(head_block.into_direction_unchecked())
            .into_coordinate_wrapping(self.grid.width(), self.grid.height());

        let next_head_block = self.get_block(next_head);
        let tile: Tile = next_head_block.into();

        match tile {
            Tile::Empty | Tile::Food => {
                self.head = next_head;
                self.set_block(next_head, head_block);
                Ok(next_head_block)
            }
            Tile::Snake(_) => Err(UpdateError::CollideBody),
            _ => unreachable!(),
            // Tile::OutOfBound => Err(UpdateError::OutOfBound),
        }
    }

    fn digest(&mut self, tile: Tile) -> Result<WorldUpdate> {
        match tile {
            Tile::Empty => {
                let tail = self.tail;
                let tail_block = self.get_block(tail);
                let next_tail = tail
                    .move_towards(tail_block.into_direction_unchecked())
                    .into_coordinate_wrapping(
                        self.grid.width(),
                        self.grid.height(),
                    );

                self.tail = next_tail;

                self.set_block(tail, Block::empty());

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
            Tile::Snake(_) => Err(UpdateError::CollideBody),
            Tile::OutOfBound => Err(UpdateError::OutOfBound),
        }
    }

    fn spawn_food(&mut self) -> Coordinate {
        loop {
            let coord = self.grid.random_coordinate(&mut self.rng);
            let current_tile: Tile = Tile::from(self.grid.get_block(coord));

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

    #[inline(always)]
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        self.grid.set_block(coord, b.into());
    }

    #[inline]
    fn iter_snake(&self) -> SnakeIter {
        SnakeIter::new(&self.grid, self.tail)
    }
}

pub struct SnakeIter<'a> {
    grid: &'a Grid,
    at: Coordinate,
}

impl<'a> SnakeIter<'a> {
    pub(super) fn new(grid: &'a Grid, at: Coordinate) -> Self {
        SnakeIter { grid, at }
    }
}

impl<'a> Iterator for SnakeIter<'a> {
    type Item = (Coordinate, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.grid.get_block(self.at);

        match block.into() {
            Some(dir) => {
                let current = self.at;
                let next = current.move_towards(dir).unwrap_unchecked();

                self.at = next;

                Some((current, dir))
            }
            _ => None,
        }
    }
}

pub enum Initializer<'a, R> {
    WorldSize(&'a World<R>, Coordinate),
    FoodAt(&'a World<R>, Coordinate),
    SnakeIter(SnakeIter<'a>),
    Done,
}

impl<'a, R: Rng> Iterator for Initializer<'a, R> {
    type Item = WorldUpdate;

    fn next(&mut self) -> Option<Self::Item> {
        match ::std::mem::replace(self, Initializer::Done) {
            Initializer::Done => None,
            Initializer::WorldSize(world, at) => {
                *self = Initializer::FoodAt(world, at);

                Some(WorldUpdate::SetWorldSize(
                    world.grid.width(),
                    world.grid.height(),
                ))
            }
            Initializer::FoodAt(world, at) => {
                *self = Initializer::SnakeIter(world.iter_snake());

                Some(WorldUpdate::SetBlock {
                    block: Block::food(),
                    at,
                })
            }
            Initializer::SnakeIter(mut iter) => {
                let (at, dir) = iter.next()?;
                *self = Initializer::SnakeIter(iter);

                Some(WorldUpdate::SetBlock {
                    block: dir.into(),
                    at,
                })
            }
        }
    }
}
