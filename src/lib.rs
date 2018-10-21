#![feature(pin)]

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
#[macro_use]
mod patch;
mod system;
mod world;

use data::Direction;
use world::{World, WorldBuilder};

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
}
