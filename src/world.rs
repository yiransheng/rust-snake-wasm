use rand::{Rng, SeedableRng};
use std::convert::{From, Into};

use data::{Block, Coordinate, Direction, Tile};
use system::{GameSystem, Generation, RenderQueue, RenderSink, RenderUnit};

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
    fn clear(&mut self) {
        self.blocks.iter_mut().map(|x| *x = Block::empty());
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
    generation: Generation,

    initial_snake: Vec<(Coordinate, Direction)>,
}

pub enum UpdateError {
    OutOfBound,
    CollideBody,
}

// side effect of a world update
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WorldUpdate {
    SetBlock { block: Block },
    Clear { prev_block: Block },
}

type Result<T> = ::std::result::Result<T, UpdateError>;

impl<R: Rng> GameSystem for World<R> {
    type Msg = WorldUpdate;
    type GameOver = UpdateError;
    type InputCmd = Option<Direction>;

    fn start_up(&mut self, q: &mut RenderQueue<Self::Msg>) {
        self.setup(q);
    }
    fn tick(&mut self, cmd: Option<Direction>, q: &mut RenderQueue<Self::Msg>) -> Result<()> {
        if let Some(dir) = cmd {
            self.set_direction(dir);
        }
        if q.is_ready() {
            self.step_update(q)
        } else {
            Ok(())
        }
    }

    fn tear_down(&mut self) {
        self.reset();
    }
}

impl<R: Rng> World<R> {
    const RENDER_TICKS: u8 = 5;

    pub fn set_direction(&mut self, dir: Direction) {
        let head = self.head;
        let head_dir: Direction = self.board.get_block(head).into();

        if dir != head_dir.opposite() {
            self.set_block(head, dir);
        }
    }

    fn reset(&mut self) {
        self.generation = Generation::default();
        self.board.clear();

        let initial_snake = ::std::mem::replace(&mut self.initial_snake, vec![]);
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

    #[inline]
    fn setup<Q: RenderSink<WorldUpdate>>(&mut self, render_queue: &mut Q) {
        use self::WorldUpdate::*;

        if self.generation > Generation::default() {
            self.reset();
        }

        let mut gen = Generation::default();

        for (at, dir) in self.iter_snake() {
            let update =
                RenderUnit::new(gen, Self::RENDER_TICKS, at, SetBlock { block: dir.into() });
            gen += 1;

            render_queue.push(update);
        }

        self.generation = gen;

        self.spawn_food_and_push_update(render_queue);
    }
    #[inline]
    fn step_update<Q: RenderSink<WorldUpdate>>(&mut self, render_queue: &mut Q) -> Result<()> {
        debug_assert!(self.generation > Generation::default());

        self.generation += 1;

        let head_dir: Direction = self.board.get_block(self.head).into();
        let next_head = self.head.move_towards(head_dir);
        let next_head_tile: Tile = self.board.get_block(next_head).into();

        match next_head_tile {
            Tile::OutOfBound => Err(UpdateError::OutOfBound),
            Tile::Snake => Err(UpdateError::CollideBody),
            Tile::Empty => {
                self.set_block_and_push_update(next_head, head_dir, render_queue);
                self.head = next_head;

                let tail = self.tail;
                let tail_dir = self.board.get_block(self.tail).into();
                let next_tail = self.tail.move_towards(tail_dir);

                self.set_block_and_push_update(tail, Block::empty(), render_queue);
                self.tail = next_tail;

                Ok(())
            }
            Tile::Food => {
                self.set_block_and_push_update(next_head, head_dir, render_queue);
                self.spawn_food_and_push_update(render_queue);

                Ok(())
            }
        }
    }

    #[inline(always)]
    fn mk_render_unit(&self, at: Coordinate, u: WorldUpdate) -> RenderUnit<WorldUpdate> {
        RenderUnit::new(self.generation, Self::RENDER_TICKS, at, u)
    }

    fn spawn_food_and_push_update<Q: RenderSink<WorldUpdate>>(&mut self, q: &mut Q) {
        loop {
            let coord =
                Coordinate::random_within(&mut self.rng, self.board.width, self.board.height);
            let current_tile = Tile::from(self.board.get_block(coord));

            if current_tile == Tile::Empty {
                self.set_block_and_push_update(coord, Block::food(), q);
                return;
            }
        }
    }

    fn set_block_and_push_update<B: Into<Block>, Q: RenderSink<WorldUpdate>>(
        &mut self,
        coord: Coordinate,
        b: B,
        q: &mut Q,
    ) {
        let b: Block = b.into();
        self.set_block(coord, b);

        if b.is_empty() {
            let prev_block = self.board.get_block(coord);
            q.push(self.mk_render_unit(coord, WorldUpdate::Clear { prev_block }));
        } else {
            q.push(self.mk_render_unit(coord, WorldUpdate::SetBlock { block: b }));
        }
    }

    #[inline]
    fn set_block<B: Into<Block>>(&mut self, coord: Coordinate, b: B) {
        self.board.set_block(coord, b.into());
    }

    #[inline]
    fn iter_snake(&self) -> SnakeIter {
        SnakeIter {
            board: &self.board,
            at: self.tail,
        }
    }
}

pub struct SnakeIter<'a> {
    board: &'a Board,
    at: Coordinate,
}

impl<'a> Iterator for SnakeIter<'a> {
    type Item = (Coordinate, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.board.get_block(self.at);

        if block.is_snake() {
            let dir = Direction::from(block);
            let current = self.at;
            let next = current.move_towards(dir);
            self.at = next;
            Some((current, dir))
        } else {
            None
        }
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
    pub fn set_snake(self, x: i32, y: i32) -> SnakeBuilder {
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

        let initial_snake: Vec<(Coordinate, Direction)>;

        {
            let iter = SnakeIter {
                board: &self.board,
                at: self.tail,
            };

            initial_snake = iter.collect();
        }

        World {
            board: self.board,
            tail: self.tail,
            head: self.head,
            initial_snake,
            generation: Generation::default(),
            rng,
        }
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

            let initial_snake: Vec<(Coordinate, Direction)>;

            {
                let iter = SnakeIter {
                    board: &board,
                    at: tail,
                };

                initial_snake = iter.collect();
            }

            World {
                board,
                head,
                tail,
                // reading from ascii can skip initial setup
                generation: Generation::default() + 1,
                rng: SmallRng::from_seed([0; 16]),

                initial_snake,
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
    use std::marker::PhantomData;

    struct MockSink<T> {
        _marker: PhantomData<T>,
    }

    impl<T> RenderSink<T> for MockSink<T> {
        fn is_ready(&self) -> bool {
            true
        }
        fn push(&mut self, x: RenderUnit<T>) {}
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
        let mut q = MockSink {
            _marker: PhantomData,
        };

        {
            let mut update = |n: usize, dir: Direction| {
                for _ in 0..n {
                    world.step_update(&mut q);
                }
                world.set_direction(dir);
            };

            // east 3
            // turn south
            update(3, Direction::South);
            // south 2
            // turn west
            update(2, Direction::West);
            // west 3
            update(3, Direction::West);
        }

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
