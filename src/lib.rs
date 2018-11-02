#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![feature(alloc, arbitrary_self_types, generators, generator_trait)]

#[cfg(not(any(feature = "std", test, debug)))]
extern crate core as std;

#[macro_use]
extern crate alloc;
extern crate console_error_panic_hook;

extern crate js_sys;
extern crate void;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate wee_alloc;

#[macro_use]
extern crate matches;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate indoc;
extern crate arraydeque;
extern crate morton;
extern crate rand;

#[macro_use]
#[cfg(test)]
extern crate quickcheck;

use alloc::boxed::Box;

use std::ops::Generator;

use rand::rngs::SmallRng;
use wasm_bindgen::prelude::*;

#[macro_use]
mod macros;

mod acceleration;
mod canvas;
mod constants;
mod data;
mod dead;
mod system;
mod world;

use acceleration::{RenderSpeed, VariableFrame};
use canvas::{CanvasEnv, WorldUpdateDraw};
use data::{Bounding, Direction, Key, Wrapping};
use dead::Dead;
use system::Stateful;
use world::{World, WorldBuilder, WorldUpdate};

#[global_allocator]
#[cfg(not(any(feature = "std", test, debug)))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
    #[cfg(feature = "std")]
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let facing = Direction::East;

    let world: World<SmallRng, Wrapping> = WorldBuilder::new()
        .width(64)
        .height(32)
        .set_snake(1, 1)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .build_with_seed([123; 16]);

    let game = world
        .zip_with(RenderSpeed::new(facing), VariableFrame::pack)
        .alternating::<Key, _>(Dead::new())
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
