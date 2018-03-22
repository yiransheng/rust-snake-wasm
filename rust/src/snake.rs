use super::geom::{Dimension, Direction, Point, Vec2d};

pub type Result<T> = ::std::result::Result<T, SnakeError>;

type Block = Option<Direction>;

#[derive(Debug)]
pub enum SnakeError {
    OutOfBound,
    NegLength,
    CollideBody,
}

pub struct Snake {
    head: Point,
    tail: Point,
    blocks: Vec2d<Block>,
}

impl Snake {
    pub fn new(dimension: Dimension) -> Self {
        Snake {
            head: Point::new(0, 0),
            tail: Point::new(0, 0),
            blocks: Vec2d::new(dimension),
        }
    }
    pub fn initialize(&mut self, tail: Point, dir: Direction, len: u32) -> Result<()> {
        debug_assert!(len > 1);
        let p_dir = Point::from(dir);
        for i in 0..len {
            let pt = tail + (p_dir * i);
            self.set_block(pt, Some(dir))?;
        }
        self.tail = tail;
        self.head = tail + (p_dir * (len - 1));

        Ok(())
    }
    pub fn initialized(&self) -> bool {
        self.head != self.tail
    }
    pub fn is_snake(&self, pt: Point) -> bool {
        match self.blocks.get(pt) {
            Some(Some(_)) => true,
            _ => false,
        }
    }
    pub fn set_dir(&mut self, dir: Direction) {
        let last_dir = self.get_dir();
        if dir != last_dir && dir != last_dir.opposite() {
            let head = self.head;
            self.set_block(head, Some(dir)).expect("Should not happen");
        }
    }
    // returns new head position
    pub fn grow_head(&mut self) -> Result<Point> {
        let dir = self.blocks[self.head].unwrap();
        let next_head = self.head + Point::from(dir);

        match self.blocks.get(next_head) {
            Some(Some(_)) => Err(SnakeError::CollideBody),
            Some(None) => self.set_block(next_head, Some(dir)).map(|_| {
                self.head = next_head;
                next_head
            }),
            _ => Err(SnakeError::OutOfBound),
        }
    }
    // returns previous tail position
    pub fn move_tail(&mut self) -> Result<Point> {
        let dir = self.blocks[self.tail].unwrap();
        let next_tail = self.tail + Point::from(dir);

        match self.blocks.get(next_tail) {
            Some(Some(_)) => {
                let tail = self.tail;
                self.tail = next_tail;
                self.set_block(tail, None)?;
                Ok(tail)
            }
            _ => Err(SnakeError::NegLength),
        }
    }
    fn get_dir(&self) -> Direction {
        self.blocks[self.head].unwrap()
    }
    fn set_block(&mut self, pt: Point, b: Block) -> Result<()> {
        match self.blocks.set(pt, b) {
            Some(_) => Ok(()),
            _ => Err(SnakeError::OutOfBound),
        }
    }
    /// debug/test use
    fn coords(&self) -> Vec<Point> {
        let mut cursor: Point = self.tail;
        let mut points = vec![cursor];
        while cursor != self.head {
            let dir = self.blocks.get(cursor).unwrap().unwrap();
            cursor = cursor + Point::from(dir);
            points.push(cursor);
        }
        points
    }
}

#[cfg(test)]
mod tests {
    #[macro_use(matches)]

    use super::Snake;
    use super::super::geom::{Dimension, Direction, Point, Vec2d};

    fn snake_invariants(snake: &Snake) {
        assert_eq!(snake.initialized(), true);
        // head points to a body block
        assert_matches!(snake.blocks.get(snake.head), Some(Some(_)));
        // tail points to a body block
        assert_matches!(snake.blocks.get(snake.tail), Some(Some(_)));

        let mut cursor: Point = snake.tail;
        while cursor != snake.head {
            let maybe_block = snake.blocks.get(cursor);
            assert_matches!(maybe_block, Some(Some(_)));

            let dir = maybe_block.unwrap().unwrap();
            cursor = cursor + Point::from(dir);
        }
    }

    #[test]
    fn pub_method_maintains_invariants() {
        let mut snake = Snake::new(Dimension(13, 7));
        let _ = snake.initialize(Point::new(0, 0), Direction::East, 4);
        snake_invariants(&snake);

        let _ = snake.set_dir(Direction::South);
        snake_invariants(&snake);
        // move south 3 times
        let _ = snake.grow_head();
        let _ = snake.move_tail();
        let _ = snake.grow_head();
        let _ = snake.move_tail();
        let _ = snake.grow_head();
        let _ = snake.move_tail();
        snake_invariants(&snake);
        // move east 2 times (grow one body length by skip move_tail)
        let _ = snake.set_dir(Direction::East);
        let _ = snake.grow_head();
        let _ = snake.move_tail();
        let _ = snake.grow_head();
        snake_invariants(&snake);

        assert_eq!(
            snake.coords(),
            vec![
                Point::new(3, 1),
                Point::new(3, 2),
                Point::new(3, 3),
                Point::new(4, 3),
                Point::new(5, 3),
            ]
        );
    }

    #[test]
    fn test_moves() {
        let moves: Vec<i32> = vec![
            3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let mut snake = Snake::new(Dimension(64, 32));
        let _ = snake.initialize(Point::new(1, 1), Direction::East, 4);
        for d in moves {
            let dir = Direction::from(d);
            snake.set_dir(dir);
            let _ = snake.grow_head();
            let _ = snake.move_tail();
        }
        assert_eq!(
            snake.coords(),
            vec![
                Point::new(9, 11),
                Point::new(9, 10),
                Point::new(9, 9),
                Point::new(9, 8),
            ]
        );
    }
}
