use std::fmt;
use std::marker::PhantomData;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use super::{SnakeIter, SnakeState, World};
use data::{Block, BoundingBehavior, Coordinate, Direction, Grid};

impl<BB: BoundingBehavior> World<SmallRng, BB> {
    pub fn from_ascii(string: &str) -> Self {
        let grid = chars_from_ascii_grid(string)
            .map(|(coord, c)| match c {
                '*' => (coord, Block::Food),
                '>' => (coord, Block::Snake(Direction::East)),
                '<' => (coord, Block::Snake(Direction::West)),
                'v' => (coord, Block::Snake(Direction::South)),
                '^' => (coord, Block::Snake(Direction::North)),
                _ => (coord, Block::Empty),
            })
            .collect();

        let initial_snake: Vec<_>;

        let tail = find_snake_tail::<BB>(&grid).unwrap();
        {
            let iter: SnakeIter<BB> = SnakeIter::new(&grid, tail);
            initial_snake = iter.collect();
        }

        let rng = SmallRng::from_seed([123; 16]);

        World {
            grid,
            state: SnakeState::Eaten,

            head: initial_snake.last().unwrap().0,
            tail,

            initial_snake,
            rng,

            _bounding_behavior: PhantomData,
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Block::Empty => '.'.fmt(f),
            Block::Snake(_) => 'o'.fmt(f),
            Block::Food => '*'.fmt(f),
            Block::OutOfBound => "".fmt(f),
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = self.width();
        let height = self.height();

        for y in 0..height {
            for x in 0..width {
                let coord = Coordinate { x, y };
                write!(f, "{}", self[coord])?;
            }
            if y < height - 1 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

pub fn chars_from_ascii_grid<'a>(
    string: &'a str,
) -> impl Iterator<Item = (Coordinate, char)> + 'a {
    string
        .lines()
        .filter(|line| !line.is_empty())
        .enumerate()
        .scan(0, |width, (line_no, line)| {
            let line_len = line.len();

            if *width == 0 || line_len == *width {
                *width = line_len;
                Some((line_no, line))
            } else {
                None
            }
        })
        .flat_map(|(line_no, line)| {
            line.chars().enumerate().map(move |(col, ch)| {
                (Coordinate::from_usizes(col, line_no), ch)
            })
        })
}

pub fn find_snake_tail<BB: BoundingBehavior>(
    grid: &Grid,
) -> Option<Coordinate> {
    for (x, y) in iproduct!(0..grid.width(), 0..grid.height()) {
        let coord = Coordinate { x, y };
        let block = grid[coord];

        let dir = match block {
            Block::Snake(dir) => dir,
            _ => continue,
        };

        let next_block = coord
            .move_towards(dir)
            .inside::<BB>(grid)
            .map(|c| grid[c])
            .unwrap_or(Block::OutOfBound);

        let prev_block = coord
            .move_towards(dir.opposite())
            .inside::<BB>(grid)
            .map(|c| grid[c])
            .unwrap_or(Block::OutOfBound);

        // is tail
        match (prev_block, next_block) {
            (Block::Empty, Block::Snake(_)) => {
                return Some(coord);
            }
            (Block::OutOfBound, Block::Snake(_)) => {
                return Some(coord);
            }
            (Block::Food, Block::Snake(_)) => {
                return Some(coord);
            }
            _ => {}
        }
    }

    None
}
