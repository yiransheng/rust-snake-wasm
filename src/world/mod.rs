use alloc::vec::Vec;
use std::marker::PhantomData;

use rand::Rng;

use data::{
    Block, BoundingBehavior, Coordinate, Direction, Grid, Natnum, Wrapping,
};
use system::{GameOver, Stateful};

pub use self::builder::WorldBuilder;

mod builder;

type Result<T> = ::std::result::Result<T, UpdateError>;

// side effect of a world update
#[derive(Debug, Copy, Clone)]
pub enum WorldUpdate {
    SetBlock { block: Block, at: Coordinate },
    Clear { prev_block: Block, at: Coordinate },
    SetWorldSize(Natnum, Natnum),
    Dead,
}

#[derive(Debug, Copy, Clone)]
pub enum UpdateError {
    HeadDetached,
    TailDetached,
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
    Consuming(Block),
}

pub struct World<R, BB: BoundingBehavior = Wrapping> {
    grid: Grid,
    state: SnakeState,

    head: Coordinate,
    tail: Coordinate,

    initial_snake: Vec<(Coordinate, Direction)>,
    rng: R,

    _bounding_behavior: PhantomData<BB>,
}

impl<'a, R: Rng + 'a, BB: BoundingBehavior + 'static> Stateful<'a>
    for World<R, BB>
{
    type Cmd = Direction;
    type Update = WorldUpdate;
    type Init = Initializer<'a, R, BB>;

    type Error = UpdateError;

    fn initialize(&'a mut self) -> Self::Init {
        let food_at = self.spawn_food();

        Initializer::WorldSize(&*self, food_at)
    }

    fn tear_down(&mut self) {
        self.reset();
    }

    #[inline(always)]
    fn step(&mut self, cmd: Option<Self::Cmd>) -> Result<Option<Self::Update>> {
        match self.step(cmd) {
            Ok(r) => Ok(r),
            Err(err) => match err {
                UpdateError::HeadDetached | UpdateError::TailDetached => {
                    panic!("Game breaking bug, snake invairant violation")
                }
                _ => Err(err),
            },
        }
    }
}

impl<R: Rng, BB: BoundingBehavior> World<R, BB> {
    fn step(&mut self, cmd: Option<Direction>) -> Result<Option<WorldUpdate>> {
        if let Some(dir) = cmd {
            self.set_direction(dir)?;
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
    fn set_direction(&mut self, dir: Direction) -> Result<()> {
        let head = self.head;
        let head_dir = self
            .get_block(head)
            .snake_or_err(UpdateError::HeadDetached)?;

        if dir != head_dir.opposite() {
            self.set_block(head, dir);
        }

        Ok(())
    }

    fn motion(&mut self) -> Result<Block> {
        let head_block = self.get_block(self.head);
        let head_dir = head_block.snake_or_err(UpdateError::HeadDetached)?;

        let next_head = self
            .head
            .move_towards(head_dir)
            .inside::<BB>(&self.grid)
            .ok_or(UpdateError::OutOfBound)?;

        let next_head_block = self.get_block(next_head);

        match next_head_block {
            Block::Empty | Block::Food => {
                self.head = next_head;
                self.set_block(next_head, head_block);
                Ok(next_head_block)
            }
            Block::Snake(_) => Err(UpdateError::CollideBody),
            Block::OutOfBound => Err(UpdateError::OutOfBound),
        }
    }

    fn digest(&mut self, tile: Block) -> Result<WorldUpdate> {
        match tile {
            Block::Empty => {
                let tail = self.tail;
                let tail_block = self.get_block(tail);

                let tail_dir =
                    tail_block.snake_or_err(UpdateError::TailDetached)?;

                let next_tail = tail
                    .move_towards(tail_dir)
                    .inside::<BB>(&self.grid)
                    .ok_or(UpdateError::OutOfBound)?;

                self.tail = next_tail;

                self.set_block(tail, Block::Empty);

                Ok(WorldUpdate::Clear {
                    prev_block: tail_block,
                    at: tail,
                })
            }
            Block::Food => {
                let coord = self.spawn_food();

                Ok(WorldUpdate::SetBlock {
                    block: Block::Food,
                    at: coord,
                })
            }
            Block::Snake(_) => Err(UpdateError::CollideBody),
            Block::OutOfBound => Err(UpdateError::OutOfBound),
        }
    }

    fn spawn_food(&mut self) -> Coordinate {
        loop {
            let coord = self.grid.random_coordinate(&mut self.rng);
            let current_block = self.get_block(coord);

            if current_block == Block::Empty {
                self.set_block(coord, Block::Food);
                return coord;
            }
        }
    }

    fn reset(&mut self) {
        self.grid.clear();

        let initial_snake =
            ::std::mem::replace(&mut self.initial_snake, Vec::new());
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

    #[inline(always)]
    fn get_block(&self, coord: Coordinate) -> Block {
        self.grid[coord]
    }

    #[inline(always)]
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        self.grid[coord] = b.into()
    }

    #[inline]
    fn iter_snake(&self) -> SnakeIter<BB> {
        SnakeIter::new(&self.grid, self.tail)
    }
}

pub struct SnakeIter<'a, BB: BoundingBehavior> {
    grid: &'a Grid,
    at: Coordinate,

    _bounding_behavior: PhantomData<BB>,
}

impl<'a, BB: BoundingBehavior> SnakeIter<'a, BB> {
    pub(super) fn new(grid: &'a Grid, at: Coordinate) -> Self {
        SnakeIter {
            grid,
            at,
            _bounding_behavior: PhantomData,
        }
    }
}

impl<'a, BB: BoundingBehavior> Iterator for SnakeIter<'a, BB> {
    type Item = (Coordinate, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.grid[self.at];

        match block.snake() {
            Some(dir) => {
                let current = self.at;
                let next =
                    current.move_towards(dir).inside::<BB>(&self.grid)?;

                self.at = next;

                Some((current, dir))
            }
            _ => None,
        }
    }
}

pub enum Initializer<'a, R, BB: BoundingBehavior> {
    WorldSize(&'a World<R, BB>, Coordinate),
    FoodAt(&'a World<R, BB>, Coordinate),
    SnakeIter(SnakeIter<'a, BB>),
    Done,
}

impl<'a, R: Rng, BB: BoundingBehavior> Iterator for Initializer<'a, R, BB> {
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
                    block: Block::Food,
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
