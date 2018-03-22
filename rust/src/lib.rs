#[macro_use]
extern crate matches;

mod geom;
mod snake;
mod game;

use game::{Game, GameInitParams};
use geom::{Direction, Point};

static mut GAME: Game = Game::Over;

static mut OUTPUT: [i32; 5] = [0; 5];

#[no_mangle]
pub fn start_game(
    world_width: u32,
    world_height: u32,
    tail_x: i32,
    tail_y: i32,
    dir: i32,
    len: u32,
) {
    unsafe {
        GAME.start(GameInitParams {
            world_width,
            world_height,
            tail_x,
            tail_y,
            dir,
            len,
        });
    }
}

#[no_mangle]
pub fn tick(dir: i32, food_x: i32, food_y: i32) -> *const i32 {
    let dir = Direction::from(dir);
    let food = Point::new(food_x, food_y);
    unsafe {
        let deltas = GAME.tick(food, dir);
        deltas.write_to_slice(&mut OUTPUT);
        &OUTPUT as *const i32
    }
}

#[no_mangle]
pub fn is_snake(x: i32, y: i32) -> i32 {
    unsafe {
        if GAME.is_snake(x, y) {
            1
        } else {
            0
        }
    }
}
