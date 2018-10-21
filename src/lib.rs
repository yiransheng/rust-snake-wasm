#![feature(pin)]

extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate either;
extern crate rand;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use either::Either;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod data;
mod render;
mod system;
mod world;

use data::Direction;
use render::CanvasRenderer;
use system::{GameInput, GameSystem, RenderQueue, StartGame};
use world::{World, WorldBuilder, WorldUpdate};

#[wasm_bindgen(module = "./game-loop")]
extern "C" {
    type GameLoop;

    #[wasm_bindgen(constructor)]
    fn new(run: &Closure<FnMut(f64)>) -> GameLoop;

    #[wasm_bindgen(method)]
    fn start(this: &GameLoop) -> bool;

    #[wasm_bindgen(method)]
    fn stop(this: &GameLoop) -> bool;
}

struct Useme {
    c: f64,
}

#[wasm_bindgen]
pub fn main() {
    use web_sys::console;

    let window = web_sys::window().expect("should have a window in this context");

    let world = WorldBuilder::new()
        .width(64)
        .height(48)
        .set_snake(1, 1)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .build_with_seed::<SmallRng>([123; 16]);

    let mut game = world.with_renderer(CanvasRenderer::new()).with_play_state();
    let mut render_queue = RenderQueue::new();

    let each_tick = Closure::wrap(Box::new(move |n: f64| {
        let cmd: GameInput<Option<Direction>>;
        if n % 60.0 == 1.0 {
            cmd = Either::Left(StartGame);
            web_sys::console::log_1(&n.into());
        } else {
            cmd = Either::Right(None);
        }

        game.tick(cmd, &mut render_queue);
    }) as Box<FnMut(_)>);

    let game_loop = GameLoop::new(&each_tick);

    game_loop.start();

    each_tick.forget();
}
