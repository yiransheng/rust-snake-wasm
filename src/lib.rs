#![feature(
    type_ascription,
    arbitrary_self_types,
    generators,
    generator_trait
)]

extern crate console_error_panic_hook;
extern crate js_sys;
extern crate void;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate arraydeque;
extern crate rand;

use std::ops::Generator;
use std::panic;

use rand::rngs::SmallRng;
use wasm_bindgen::prelude::*;

#[macro_use]
mod macros;

mod acceleration;
mod constants;
mod data;
mod renderers;
mod system;
mod world;

use acceleration::{RenderSpeed, VariableFrame};
use data::{Direction, Key};
use renderers::{CanvasEnv, WorldUpdateDraw};
use system::Model;
use world::{WorldBuilder, WorldUpdate};

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
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let facing = Direction::East;

    let world = WorldBuilder::new()
        .width(96)
        .height(64)
        .set_snake(1, 1)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .build_with_seed::<SmallRng>([123; 16]);

    let game = world
        .zip_with(RenderSpeed::new(facing), VariableFrame::pack)
        .make_game(CanvasEnv::new());

    let (cmd_buffer, mut generator) =
        game.new_game::<WorldUpdateDraw<VariableFrame<WorldUpdate>>, Key>();

    let each_tick = Closure::wrap(Box::new(move |key: u8| {
        let key = Key::from(key);

        cmd_buffer.borrow_mut().write(key);

        unsafe {
            generator.resume();
        }
    }) as Box<FnMut(_)>);

    let game_loop = GameLoop::new(&each_tick);

    game_loop.start();

    each_tick.forget();
}
