use alloc::vec::Vec;
use std::ops::{Index, IndexMut};

use data::{Block, Coordinate, Natnum};
use rand::Rng;

pub struct Grid {
    blocks: Vec<Block>,
    width: Natnum,
    height: Natnum,
}

// Storage of game data

impl Grid {
    pub(super) fn empty(width: Natnum, height: Natnum) -> Self {
        debug_assert!(width > 0 && height > 0);

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
    pub(super) fn width(&self) -> Natnum {
        self.width
    }
    #[inline(always)]
    pub(super) fn height(&self) -> Natnum {
        self.height
    }

    pub fn contains(&self, coord: Coordinate) -> bool {
        coord.x < self.width && coord.y < self.height
    }

    pub(super) fn random_coordinate<R: Rng>(&self, rng: &mut R) -> Coordinate {
        let x = rng.gen_range(0, self.width);
        let y = rng.gen_range(0, self.height);

        Coordinate { x, y }
    }

    pub(super) fn clear(&mut self) {
        self.blocks.iter_mut().for_each(|x| *x = Block::Empty);
    }

    /*
     *     #[cfg(test)]
     *     fn get_prev_block(&self, coord: Coordinate) -> Option<Block> {
     *         let b = self.get_block(coord);
     *
     *         let dir = b.snake()?;
     *         let dir = dir.opposite();
     *
     *         let prev_coord =
     *             coord.move_towards(dir).wrap_inside(self.width, self.height);
     *
     *         let prev_block = self.get_block(prev_coord);
     *
     *         Some(prev_block)
     *     }
     *     #[cfg(test)]
     *     fn get_next_block(&self, coord: Coordinate) -> Option<Block> {
     *         let b = self.get_block(coord);
     *
     *         let dir = b.snake()?;
     *         let next_coord =
     *             coord.move_towards(dir).wrap_inside(self.width, self.height);
     *
     *         let next_block = self.get_block(next_coord);
     *
     *         Some(next_block)
     *     }
     */
}

impl Index<Coordinate> for Grid {
    type Output = Block;

    fn index<'a>(&'a self, index: Coordinate) -> &'a Block {
        &self.blocks[index.as_usize()]
    }
}
impl IndexMut<Coordinate> for Grid {
    fn index_mut<'a>(&'a mut self, index: Coordinate) -> &'a mut Block {
        &mut self.blocks[index.as_usize()]
    }
}
