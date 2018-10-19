extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate rand;

use wasm_bindgen::prelude::*;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod world;

#[wasm_bindgen]
pub fn main() {
    use web_sys::console;

    let mut rng = SmallRng::from_seed([123; 16]);
    let n: u32 = rng.gen_range(0, 34);

    console::log_2(&"Hello wasm".into(), &n.into());
}
