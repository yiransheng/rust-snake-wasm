extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate rand;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::mem;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod data;
mod game;
#[macro_use]
mod patch;
mod render;
mod world;

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

    let cb = Closure::wrap(Box::new(move || {
        console::log_1(&1.into());
    }) as Box<Fn()>);

    let c = GameLoop::new(cb.as_ref().unchecked_ref());

    c.start();

    cb.forget();
}
