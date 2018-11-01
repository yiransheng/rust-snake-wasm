use std::marker::PhantomData;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::{SnakeIter, SnakeState, World};
use data::{
    Block, Bounding, BoundingBehavior, Coordinate, Direction, Grid, Wrapping,
};

impl<BB: BoundingBehavior> World<SmallRng, BB> {
    pub fn from_ascii(string: &str) -> Self {
        let grid = chars_from_ascii_grid(string)
            .map(|(coord, c)| match c {
                'o' => (coord, Block::Empty),
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

pub fn chars_from_ascii_grid<'a>(
    string: &'a str,
) -> impl Iterator<Item = (Coordinate, char)> + 'a {
    string
        .lines()
        .filter(|line| !line.is_empty())
        .scan(0, |width, line| {
            let line_len = line.len();

            if *width == 0 || line_len == *width {
                *width = line_len;
                Some(line)
            } else {
                None
            }
        })
        .flat_map(|line| line.chars().enumerate())
        .enumerate()
        .map(|(line, (col, ch))| (Coordinate::from_usizes(col, line), ch))
}

pub fn find_snake_tail<BB: BoundingBehavior>(
    grid: &Grid,
) -> Option<Coordinate> {
    for (x, y) in iproduct!(0..grid.width(), 0..grid.height()) {
        let coord = Coordinate { x, y };
        let block = grid[coord];

        let dir = block.snake().unwrap_or({
            continue;
        });
        let next_block =
            coord.move_towards(dir).inside::<BB>(grid).map(|c| grid[c]);

        let prev_block = coord
            .move_towards(dir.opposite())
            .inside::<BB>(grid)
            .map(|c| grid[c]);

        // is tail
        match (prev_block, next_block) {
            (Some(Block::Empty), Some(Block::Snake(_))) => {
                return Some(coord);
            }
            _ => {}
        }
    }

    None
}
