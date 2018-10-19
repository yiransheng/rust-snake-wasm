extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate rand;

use wasm_bindgen::prelude::*;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod data;
mod render;
mod world;

use world::{Direction, World, WorldBuilder};

#[wasm_bindgen]
pub fn main() {
    use web_sys::console;

    let world = WorldBuilder::new()
        .width(32)
        .height(16)
        .set_snake(1, 1)
        .extend(Direction::West)
        .extend(Direction::West)
        .extend(Direction::West)
        .build_with_seed::<SmallRng>([123; 16]);
}
