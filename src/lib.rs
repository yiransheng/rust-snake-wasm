#![feature(
    type_ascription,
    try_from,
    arbitrary_self_types,
    generators,
    generator_trait
)]

extern crate js_sys;
extern crate void;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate arraydeque;
extern crate either;
extern crate rand;
extern crate smallvec;

use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut, Generator, GeneratorState};
use std::rc::Rc;
use std::sync::Mutex;

use rand::rngs::SmallRng;
use wasm_bindgen::prelude::*;

#[macro_use]
mod macros;

mod constants;
// mod acceleration;
mod data;
mod renderers;
mod system;
mod world;

use data::{Direction, Key};
use renderers::{BlockRenderer, CanvasEnv};
use system::{Game, Model, Render};
use world::{World, WorldBuilder, WorldUpdate};

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

/*
 * lazy_static! {
 *     static ref GAME: Mutex<Game<World<SmallRng>, Cell<Key>, CanvasEnv>> = {
 *         let world = WorldBuilder::new()
 *             .width(64)
 *             .height(32)
 *             .set_snake(1, 1)
 *             .extend(Direction::East)
 *             .extend(Direction::East)
 *             .extend(Direction::East)
 *             .extend(Direction::East)
 *             .build_with_seed::<SmallRng>([123; 16]);
 *
 *         Mutex::new(world.make_game(Cell::new(Key::from(0)), CanvasEnv::new()))
 *     };
 * }
 *
 */

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

    let game = world.make_game(CanvasEnv::new());

    let (cmd_buffer, mut generator) = game.create::<BlockRenderer<_>, Key>();

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
