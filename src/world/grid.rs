use alloc::vec::Vec;

use data::{Block, Coordinate};
use rand::Rng;

pub struct Grid {
    blocks: Vec<Block>,
    width: u32,
    height: u32,
}

// Storage of game data

impl Grid {
    pub(super) fn empty(width: u32, height: u32) -> Self {
        Grid {
            width: width as u32,
            height: height as u32,
            blocks: vec![Block::Empty; (width * height) as usize],
        }
    }

    #[inline(always)]
    pub(super) fn width(&self) -> u32 {
        self.width
    }
    #[inline(always)]
    pub(super) fn height(&self) -> u32 {
        self.height
    }

    pub(super) fn random_coordinate<R: Rng>(&self, rng: &mut R) -> Coordinate {
        // don't put the damn thing on edge
        let x = rng.gen_range(1, self.width - 1);
        let y = rng.gen_range(1, self.height - 1);

        Coordinate { x, y }
    }

    pub(super) fn clear(&mut self) {
        self.blocks.iter_mut().for_each(|x| *x = Block::Empty);
    }

    #[inline]
    pub(super) fn get_block(&self, coord: Coordinate) -> Block {
        if let Some(index) = self.find_index(coord) {
            self.blocks[index]
        } else {
            Block::OutOfBound
        }
    }

    #[inline]
    pub(super) fn set_block(&mut self, coord: Coordinate, b: Block) -> bool {
        if let Some(block) = self.get_block_mut(coord) {
            *block = b;
            true
        } else {
            false
        }
    }

    #[inline]
    fn get_block_mut(&mut self, coord: Coordinate) -> Option<&mut Block> {
        if let Some(index) = self.find_index(coord) {
            self.blocks.get_mut(index)
        } else {
            None
        }
    }

    fn find_index(&self, coord: Coordinate) -> Option<usize> {
        let width = self.width;
        let height = self.height;;

        let Coordinate { x, y } = coord;

        if x >= width || y >= height {
            None
        } else {
            Some((y * width + x) as usize)
        }
    }

    #[cfg(test)]
    fn get_prev_snake_block(&self, coord: Coordinate) -> Option<Block> {
        let b = self.get_block(coord);
        let dir = b.into_direction()?;
        let dir = dir.opposite();

        let prev_coord = coord
            .move_towards(dir)
            .bound_inside(self.width, self.height)?;

        let prev_block = self.get_block(prev_coord);

        if prev_block.is_snake() {
            Some(prev_block)
        } else {
            None
        }
    }
    #[cfg(test)]
    fn get_next_snake_block(&self, coord: Coordinate) -> Option<Block> {
        let b = self.get_block(coord);

        let dir = b.into_direction()?;

        let next_coord = coord
            .move_towards(dir)
            .bound_inside(self.width, self.height)?;

        let next_block = self.get_block(next_coord);

        if next_block.is_snake() {
            Some(next_block)
        } else {
            None
        }
    }
}
