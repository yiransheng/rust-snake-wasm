use std::convert::{From, Into};
use std::mem::transmute;

use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    North = 0b1000_0000,
    South = 0b1000_0001,
    East = 0b1000_0010,
    West = 0b1000_0011,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Block {
    raw: u8,
}

impl Block {
    pub fn empty() -> Self {
        Block { raw: 0 }
    }
    pub fn food() -> Self {
        // second bit on: food
        Block { raw: 0b0100_0000 }
    }
    pub fn out_of_bound() -> Self {
        // first two bits off: OOB
        Block { raw: 0b0011_1111 }
    }
    pub fn from_raw(raw: u8) -> Self {
        Block { raw }
    }
    pub fn into_raw(self) -> u8 {
        self.raw
    }
    pub fn is_snake(self) -> bool {
        // first bit is 1
        self.raw & 0b1000_0000 != 0
    }
    #[inline(always)]
    unsafe fn into_direction(self) -> Direction {
        // keep only lower 2 bits, then turn on first bit
        let bits = self.raw & 0b0000_0011 | 0b1000_0000;
        transmute(bits)
    }
}

// conversions

impl Into<Block> for u8 {
    fn into(self) -> Block {
        Block::from_raw(self)
    }
}

impl From<Block> for Tile {
    fn from(b: Block) -> Tile {
        match b.raw {
            0 => Tile::Empty,
            b if (b & 0b1000_0000) != 0 => Tile::Snake,
            b if (b & 0b0100_0000) != 0 => Tile::Food,
            _ => Tile::OutOfBound,
        }
    }
}

impl From<Block> for Direction {
    fn from(b: Block) -> Direction {
        debug_assert!(b.is_snake());

        unsafe { b.into_direction() }
    }
}
impl From<Direction> for Block {
    fn from(dir: Direction) -> Self {
        Block { raw: (dir as u8) }
    }
}

pub struct Board {
    blocks: Vec<Block>,
    width: i32,
    height: i32,
}

// game states

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    pub fn new(x: i32, y: i32) -> Self {
        Coordinate { x, y }
    }
    pub fn from_usize(x: usize, y: usize) -> Self {
        Coordinate {
            x: x as i32,
            y: y as i32,
        }
    }
    fn random_within<R: Rng>(rng: &mut R, width: i32, height: i32) -> Self {
        debug_assert!(width > 0 && height > 0);

        let x = rng.gen_range(0, width);
        let y = rng.gen_range(0, height);

        Coordinate { x, y }
    }

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
    fn empty(width: u32, height: u32) -> Self {
        Board {
            width: width as i32,
            height: height as i32,
            blocks: vec![Block::empty(); (width * height) as usize],
        }
    }
    fn get_block(&self, coord: Coordinate) -> Block {
        let Coordinate { x, y } = coord;
        let width = self.width;
        let height = self.height;

        if x < 0 || x >= width || y < 0 || y >= height {
            Block::out_of_bound()
        } else {
            self.blocks[(y * width + x) as usize]
        }
    }
    fn set_block(&mut self, coord: Coordinate, b: Block) -> bool {
        if let Some(block) = self.get_block_mut(coord) {
            *block = b;
            true
        } else {
            false
        }
    }
    fn get_block_mut(&mut self, coord: Coordinate) -> Option<&mut Block> {
        let Coordinate { x, y } = coord;
        let width = self.width;
        let height = self.height;;

        if x < 0 || x >= width || y < 0 || y >= height {
            None
        } else {
            self.blocks.get_mut((y * width + x) as usize)
        }
    }

    fn get_prev_snake_block(&self, coord: Coordinate) -> Option<Block> {
        let b = self.get_block(coord);

        if b.is_snake() {
            let dir = Direction::from(b);
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
            let dir = Direction::from(b);

            let next_coord = coord.move_towards(dir);
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

pub struct World<R> {
    board: Board,
    head: Coordinate,
    tail: Coordinate,
    rng: R,
}

enum UpdateError {
    OutOfBound,
    CollideBody,
}

type Result<T> = ::std::result::Result<T, UpdateError>;

impl<R: Rng> World<R> {
    pub fn set_direction(&mut self, dir: Direction) {
        let head = self.head;
        self.set_block(head, dir);
    }
    pub fn update(&mut self) -> Result<()> {
        let head_dir: Direction = self.board.get_block(self.head).into();
        let next_head = self.head.move_towards(head_dir);
        let next_head_tile: Tile = self.board.get_block(next_head).into();

        match next_head_tile {
            Tile::OutOfBound => Err(UpdateError::OutOfBound),
            Tile::Snake => Err(UpdateError::CollideBody),
            Tile::Empty => {
                self.set_block(next_head, head_dir);
                self.head = next_head;

                let tail = self.tail;
                let tail_dir = self.board.get_block(self.tail).into();
                let next_tail = self.tail.move_towards(tail_dir);

                self.set_block(tail, Block::empty());
                self.tail = next_tail;

                Ok(())
            }
            Tile::Food => {
                self.set_block(next_head, head_dir);
                self.spawn_food();
                Ok(())
            }
        }
    }

    fn spawn_food(&mut self) {
        loop {
            let coord =
                Coordinate::random_within(&mut self.rng, self.board.width, self.board.height);
            let current_tile = Tile::from(self.board.get_block(coord));

            if current_tile == Tile::Empty {
                self.set_block(coord, Block::food());
                return;
            }
        }
    }

    #[inline]
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        self.board.set_block(coord, b.into());
    }
}

pub struct WorldBuilder {
    width: u32,
    height: u32,
    tail: Coordinate,
}

pub struct SnakeBuilder {
    board: Board,
    head: Coordinate,
}

impl SnakeBuilder {
    pub fn extend(&mut self, dir: Direction) -> &mut Self {
        let next_head = self.head.move_towards(dir);
        if self.board.set_block(next_head, Block::from(dir)) {
            self.head = next_head;
        }

        self
    }
}

mod test_utils {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
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

    impl World<SmallRng> {
        #[allow(dead_code)]
        pub fn from_ascii(s: &str) -> Self {
            let height = s.lines().count() as u32;
            let width = s.lines().next().unwrap().chars().count() as u32;

            let mut board = Board::empty(width, height);

            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    board.set_block(Coordinate::from_usize(x, y), c.into());
                }
            }

            let mut head = Coordinate { x: 0, y: 0 };
            let mut tail = Coordinate { x: 0, y: 0 };

            for x in 0..board.width {
                for y in 0..board.height {
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

            World {
                board,
                head,
                tail,
                rng: SmallRng::from_seed([0; 16]),
            }
        }
    }

    impl<R> fmt::Display for World<R> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for y in 0..self.board.height {
                for x in 0..self.board.width {
                    let coord = Coordinate { x, y };
                    write!(f, "{}", self.board.get_block(coord))?;
                }
                write!(f, "\n")?;
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
    fn test_converion() {
        for x in 0..u8::max_value() {
            let b = Block::from_raw(x);
            let tile = Tile::from(b);

            if b.is_snake() {
                assert_eq!(tile, Tile::Snake);
            }

            // make sure every u8 bit pattern result in a valid Direction
            // ...that is, not getting SIGIL or other UB
            unsafe {
                let dir = b.into_direction();
                assert!(
                    dir == Direction::North
                        || dir == Direction::South
                        || dir == Direction::East
                        || dir == Direction::West
                );
            }
        }
    }

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

        // ate food -> new food deterministically
        // generated from known seed
        let final_state = "....*.....
..........
..........
.......o..
....oooo..
..........
";

        assert_eq!(final_state, &world.to_string());
    }

}
