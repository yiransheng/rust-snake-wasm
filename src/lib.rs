extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate rand;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod data;
mod game;
#[macro_use]
mod patch;
mod render;
mod world;

use game::{GameState, Renderer, RunGame};
use render::CanvasRenderer;
use world::{Direction, World, WorldBuilder};

#[wasm_bindgen(module = "./game-loop")]
extern "C" {
    type GameLoop;

    #[wasm_bindgen(constructor)]
    fn new(run: &JsValue) -> GameLoop;

    #[wasm_bindgen(method)]
    fn start(this: &GameLoop) -> bool;

    #[wasm_bindgen(method)]
    fn stop(this: &GameLoop) -> bool;
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

    let renderer = CanvasRenderer::new();
    let game = Rc::new(RefCell::new(RunGame::new(world, renderer)));

    game.borrow_mut().start();

    let run = Closure::wrap(Box::new(move || {
        game.borrow_mut().on_enter_frame();
    }) as Box<Fn()>);

    let gameLoop = GameLoop::new(run.as_ref().unchecked_ref());

    gameLoop.start();

    run.forget();
}
