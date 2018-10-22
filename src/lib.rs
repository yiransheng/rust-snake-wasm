extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate arraydeque;
extern crate either;
extern crate rand;
extern crate smallvec;

use wasm_bindgen::prelude::*;

use rand::rngs::SmallRng;

mod acceleration;
mod data;
mod renderers;
mod system;
mod world;

#[macro_use]
mod console_log;

use acceleration::AccMiddleware;
use data::{Direction, Key};
use renderers::CanvasRenderer;
use system::GameSystem;
use world::WorldBuilder;

#[wasm_bindgen(module = "./game-loop")]
extern "C" {
    type GameLoop;

    #[wasm_bindgen(constructor)]
    fn new(run: &Closure<FnMut(u8)>) -> GameLoop;

    #[wasm_bindgen(method)]
    fn start(this: &GameLoop) -> bool;

    #[wasm_bindgen(method)]
    fn stop(this: &GameLoop) -> bool;
}

#[wasm_bindgen]
pub fn main() {
    let world = WorldBuilder::new()
        .width(64)
        .height(32)
        .set_snake(1, 1)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .build_with_seed::<SmallRng>([123; 16]);

    let game = world
        .map_input(|key: Key| key.into())
        .with_renderer(CanvasRenderer::new())
        .with_middlewares()
        .add_middleware(Box::new(AccMiddleware::new()))
        .with_play_state();

    let each_tick = game.into_closure();

    GameLoop::new(&each_tick).start();

    each_tick.forget();
}
