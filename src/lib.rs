extern crate js_sys;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod world;

#[wasm_bindgen]
pub fn main() {
    println!("Hello, world!");
}
