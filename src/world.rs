use std::convert::Into;
use std::default::Default;
use std::ops::Add;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    North = 1,
    South = 2,
    East = 3,
    West = 4,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Snake,
    Food,
    OutOfBound,
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct Block {
    raw: u8,
}

impl Block {
    pub fn empty() -> Self {
        Block { raw: 0 }
    }
    pub fn food() -> Self {
        Block { raw: 5 }
    }
    pub fn out_of_bound() -> Self {
        Block { raw: 255 }
    }
    pub fn from_raw(raw: u8) -> Self {
        Block { raw }
    }
    pub fn into_raw(self) -> u8 {
        self.raw
    }
}

impl Into<Tile> for Block {
    fn into(self) -> Tile {
        match self.raw {
            0 => Tile::Empty,
            1 | 2 | 3 | 4 => Tile::Snake,
            5 => Tile::Food,
            _ => Tile::OutOfBound,
        }
    }
}

impl Into<Block> for u8 {
    fn into(self) -> Block {
        Block::from_raw(self)
    }
}
impl Into<Direction> for u8 {
    fn into(self) -> Direction {
        match self {
            1 => Direction::North,
            2 => Direction::South,
            3 => Direction::East,
            4 => Direction::West,
            _ => panic!("not a direction"),
        }
    }
}

impl Into<Direction> for Block {
    fn into(self) -> Direction {
        self.raw.into()
    }
}
impl From<Direction> for Block {
    fn from(dir: Direction) -> Self {
        Block { raw: (dir as u8) }
    }
}

pub struct Board {
    blocks: Vec<Block>,
    width: u32,
    height: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    fn move_towards(self, dir: Direction) -> Self {
        let Coordinate { x, y } = self;
        match dir {
            Direction::North => Coordinate { x, y: y - 1 },
            Direction::South => Coordinate { x, y: y + 1 },
            Direction::East => Coordinate { x: x + 1, y },
            Direction::West => Coordinate { x: x - 1, y },
        }
    }
}

impl Board {
    pub fn get_block(&self, coord: Coordinate) -> Block {
        let Coordinate { x, y } = coord;
        let width = self.width as i32;
        let height = self.height as i32;

        if x < 0 || x >= width || y < 0 || y >= height {
            Block::out_of_bound()
        } else {
            self.blocks[(y * width + x) as usize]
        }
    }
    fn get_block_mut(&mut self, coord: Coordinate) -> Option<&mut Block> {
        let Coordinate { x, y } = coord;
        let width = self.width as i32;
        let height = self.height as i32;

        if x < 0 || x >= width || y < 0 || y >= height {
            None
        } else {
            self.blocks.get_mut((y * width + x) as usize)
        }
    }

    #[inline]
    pub fn get_tile(&self, coord: Coordinate) -> Tile {
        self.get_block(coord).into()
    }
    #[inline]
    pub fn get_snake_direction(&self, coord: Coordinate) -> Direction {
        self.get_block(coord).into()
    }
}

pub struct World {
    board: Board,
    head: Coordinate,
    tail: Coordinate,
    food: Coordinate,
}

enum UpdateError {
    OutOfBound,
    CollideBody,
}

type Result<T> = ::std::result::Result<T, UpdateError>;

impl World {
    fn set_snake_direction(&mut self, dir: Direction) {
        let block = self.board.get_block_mut(self.head).unwrap();
        *block = Block::from(dir)
    }
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        let block = self.board.get_block_mut(coord).unwrap();
        *block = b.into()
    }
    fn update(&mut self) -> Result<()> {
        let head_dir = self.board.get_snake_direction(self.head);
        let next_head = self.head.move_towards(head_dir);
        let next_head_tile = self.board.get_tile(next_head);

        match next_head_tile {
            Tile::OutOfBound => Err(UpdateError::OutOfBound),
            Tile::Snake => Err(UpdateError::CollideBody),
            Tile::Empty => {
                self.set_block(next_head, head_dir);

                let tail = self.tail;
                let tail_dir = self.board.get_snake_direction(self.tail);
                let next_tail = self.tail.move_towards(tail_dir);

                self.set_block(tail, Block::empty());
                self.tail = next_tail;

                Ok(())
            }
            Tile::Food => {
                self.set_block(next_head, head_dir);
                Ok(())
            }
        }
    }
}
