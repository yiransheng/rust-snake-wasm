use super::geom::{Dimension, Direction, Point};
use super::snake::{Result, Snake};

pub enum Game {
    Over,
    Running(Snake),
}

pub enum GameDelta {
    GameOver,
    GrowHead(Point),
    MoveHead(Point, Point),
}

impl GameDelta {
    pub fn write_to_slice(&self, buf: &mut [i32]) {
        debug_assert!(buf.len() >= 5);
        match self {
            &GameDelta::GameOver => {
                buf[0] = 0;
            }
            &GameDelta::GrowHead(head) => {
                buf[0] = 2;
                buf[1] = head.x;
                buf[2] = head.y;
            }
            &GameDelta::MoveHead(head, tail) => {
                buf[0] = 4;
                buf[1] = head.x;
                buf[2] = head.y;
                buf[3] = tail.x;
                buf[4] = tail.y;
            }
        }
    }
}

pub struct GameInitParams {
    pub world_width: u32,
    pub world_height: u32,
    pub tail_x: i32,
    pub tail_y: i32,
    pub dir: i32,
    pub len: u32,
}

impl Game {
    pub fn start(&mut self, params: GameInitParams) {
        let mut snake = Snake::new(Dimension(params.world_width, params.world_height));
        let tail = Point::new(params.tail_x, params.tail_y);
        if let Ok(_) = snake.initialize(tail, Direction::from(params.dir), params.len) {
            *self = Game::Running(snake);
        }
    }
    pub fn is_snake(&self, x: i32, y: i32) -> bool {
        match self {
            &Game::Over => false,
            &Game::Running(ref snake) => snake.is_snake(Point::new(x, y)),
        }
    }
    pub fn tick(&mut self, food_position: Point, dir: Direction) -> GameDelta {
        let delta = self._tick(food_position, dir)
            .unwrap_or(GameDelta::GameOver);

        match delta {
            GameDelta::GameOver => {
                *self = Game::Over;
            }
            _ => (),
        }

        delta
    }
    fn _tick(&mut self, food_position: Point, dir: Direction) -> Result<GameDelta> {
        match *self {
            Game::Over => Ok(GameDelta::GameOver),
            Game::Running(ref mut snake) => {
                snake.set_dir(dir);
                let next_head = snake.grow_head()?;
                if next_head != food_position {
                    snake
                        .move_tail()
                        .map(|last_tail| GameDelta::MoveHead(next_head, last_tail))
                } else {
                    Ok(GameDelta::GrowHead(next_head))
                }
            }
        }
    }
}
