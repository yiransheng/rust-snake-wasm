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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Snake,
    Food,
    OutOfBound,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
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
    pub fn is_snake(self) -> bool {
        self.raw > 0 && self.raw <= 4
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
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
    fn new(width: u32, height: u32) -> Self {
        Board {
            width,
            height,
            blocks: vec![Block::empty(); (width * height) as usize],
        }
    }
    fn get_block(&self, coord: Coordinate) -> Block {
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
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        if let Some(block) = self.get_block_mut(coord) {
            *block = b.into();
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

    fn get_prev_snake_block(&self, coord: Coordinate) -> Option<Block> {
        let b = self.get_block(coord);
        if b.is_snake() {
            let dir: Direction = b.into();
            let dir = dir.opposite();
            let prev_coord = coord.move_towards(dir);
            let prev_block = self.get_block(prev_coord);
            if prev_block.is_snake() {
                Some(prev_block)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn get_next_snake_block(&self, coord: Coordinate) -> Option<Block> {
        let b = self.get_block(coord);
        if b.is_snake() {
            let next_coord = coord.move_towards(b.into());
            let next_block = self.get_block(next_coord);
            if next_block.is_snake() {
                Some(next_block)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct World {
    board: Board,
    head: Coordinate,
    tail: Coordinate,
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
    fn set_direction(&mut self, dir: Direction) {
        let head = self.head;
        self.set_block(head, dir);
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
                self.head = next_head;

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

mod test_utils {
    use super::*;
    use std::fmt;

    impl From<char> for Block {
        fn from(c: char) -> Block {
            match c {
                'o' => Block::empty(),
                '*' => Block::food(),
                '>' => Block::from(Direction::East),
                '<' => Block::from(Direction::West),
                'v' => Block::from(Direction::South),
                '^' => Block::from(Direction::North),
                _ => Block::out_of_bound(),
            }
        }
    }

    impl fmt::Display for Block {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let tile = (*self).into();
            match tile {
                Tile::Empty => '.'.fmt(f),
                Tile::Snake => 'o'.fmt(f),
                Tile::Food => '*'.fmt(f),
                Tile::OutOfBound => "".fmt(f),
            }
        }
    }

    impl World {
        pub fn from_ascii(s: &str) -> Self {
            let height = s.lines().count() as u32;
            let width = s.lines().next().unwrap().chars().count() as u32;

            let mut board = Board::new(width, height);

            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    let x = x as i32;
                    let y = y as i32;
                    board.set_block(Coordinate { x, y }, c);
                }
            }

            let mut head = Coordinate { x: 0, y: 0 };
            let mut tail = Coordinate { x: 0, y: 0 };

            for x in 0..width {
                for y in 0..height {
                    let x = x as i32;
                    let y = y as i32;
                    let coord = Coordinate { x, y };
                    let b = board.get_block(coord);
                    if b.is_snake() {
                        if board.get_prev_snake_block(coord).is_none() {
                            tail = coord;
                        }
                        if board.get_next_snake_block(coord).is_none() {
                            head = coord;
                        }
                    }
                }
            }

            World { board, head, tail }
        }
    }

    impl fmt::Display for World {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for y in 0..self.board.height {
                for x in 0..self.board.width {
                    let x = x as i32;
                    let y = y as i32;
                    let coord = Coordinate { x, y };
                    write!(f, "{}", self.board.get_block(coord))?;
                }
                write!(f, "\n");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;

    #[test]
    fn test_game() {
        let world = "oooooooooo
oooooooooo
o>>>>ooooo
oooooooooo
oooo*ooooo
oooooooooo";
        let mut world = World::from_ascii(world);

        // east 3
        world.update();
        world.update();
        world.update();
        // turn south
        world.set_direction(Direction::South);
        // south 2
        world.update();
        world.update();
        // turn west
        world.set_direction(Direction::West);
        // west 3
        world.update();
        world.update();
        world.update();

        // ate food
        let final_state = "..........
..........
..........
.......o..
....oooo..
..........
";

        assert_eq!(final_state, &world.to_string());
    }

}
