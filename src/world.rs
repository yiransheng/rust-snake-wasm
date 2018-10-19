use rand::{Rng, SeedableRng};
use std::convert::{From, Into};

pub use data::{Block, Coordinate, Direction, Tile};
use patch::Patch;

struct Board {
    blocks: Vec<Block>,
    width: i32,
    height: i32,
}

// game states

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

pub enum UpdateError {
    OutOfBound,
    CollideBody,
}

// side effect of a world update
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WorldUpdateEff {
    SetBlock { at: Coordinate, block: Block },
    Clear { at: Coordinate, prev_block: Block },
}

type Result<T> = ::std::result::Result<T, UpdateError>;

impl<R: Rng> World<R> {
    pub fn set_direction(&mut self, dir: Direction) {
        let head = self.head;
        let head_dir: Direction = self.board.get_block(head).into();

        if dir != head_dir.opposite() {
            self.set_block(head, dir);
        }
    }
    pub fn tick_update(&mut self) -> Result<Patch<WorldUpdateEff>> {
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

                Ok(patch!(
                    WorldUpdateEff::SetBlock {
                        at: next_head,
                        block: head_dir.into(),
                    },
                    WorldUpdateEff::Clear {
                        at: tail,
                        prev_block: tail_dir.into()
                    }
                ))
            }
            Tile::Food => {
                self.set_block(next_head, head_dir);
                let food_coord = self.spawn_food();

                Ok(patch!(
                    WorldUpdateEff::SetBlock {
                        at: next_head,
                        block: head_dir.into(),
                    },
                    WorldUpdateEff::SetBlock {
                        at: food_coord,
                        block: Block::food(),
                    }
                ))
            }
        }
    }

    fn spawn_food(&mut self) -> Coordinate {
        loop {
            let coord =
                Coordinate::random_within(&mut self.rng, self.board.width, self.board.height);
            let current_tile = Tile::from(self.board.get_block(coord));

            if current_tile == Tile::Empty {
                self.set_block(coord, Block::food());
                return coord;
            }
        }
    }

    #[inline]
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        self.board.set_block(coord, b.into());
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
    pub fn set_snake(&self, x: i32, y: i32) -> SnakeBuilder {
        let board = Board::empty(self.width, self.height);
        let tail = Coordinate::new(x, y);

        SnakeBuilder {
            board,
            head: tail,
            tail,
            next_head: tail,
            snake_len: 0,
        }
    }
}

pub struct SnakeBuilder {
    board: Board,
    head: Coordinate,
    next_head: Coordinate,
    tail: Coordinate,
    snake_len: u32,
}

impl SnakeBuilder {
    pub fn extend(mut self, dir: Direction) -> Self {
        let next_head = self.next_head;
        let next_head_block = self.board.get_block(next_head);

        assert!(Tile::from(next_head_block) == Tile::Empty);

        if !self.board.set_block(next_head, Block::from(dir)) {
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

        let mut world = World {
            board: self.board,
            tail: self.tail,
            head: self.head,
            rng,
        };

        world.spawn_food();

        world
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
    fn test_game() {
        let world = "oooooooooo
oooooooooo
o>>>>ooooo
oooooooooo
oooo*ooooo
oooooooooo";
        let mut world = World::from_ascii(world);

        // east 3
        world.tick_update();
        world.tick_update();
        world.tick_update();
        // turn south
        world.set_direction(Direction::South);
        // south 2
        world.tick_update();
        world.tick_update();
        // turn west
        world.set_direction(Direction::West);
        // west 3
        world.tick_update();
        world.tick_update();
        world.tick_update();

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
