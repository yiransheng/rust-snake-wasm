extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate either;
extern crate rand;

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
use system::{GameInput, GameSystem, RenderQueue};
use world::WorldBuilder;

#[wasm_bindgen(module = "./game-loop")]
extern "C" {
    type GameLoop;

    #[wasm_bindgen(constructor)]
    fn new(run: &Closure<FnMut(u8, u8)>) -> GameLoop;

    #[wasm_bindgen(method)]
    fn start(this: &GameLoop) -> bool;

    #[wasm_bindgen(method)]
    fn stop(this: &GameLoop) -> bool;
}

#[wasm_bindgen]
pub fn main() {
    let world = WorldBuilder::new()
        .width(50)
        .height(20)
        .set_snake(1, 1)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .build_with_seed::<SmallRng>([123; 16]);

    let mut game = world
        .map_input(|key: Key| key.into())
        .with_renderer(CanvasRenderer::new())
        .with_middlewares()
        .add_middleware(Box::new(AccMiddleware::new()))
        .with_play_state();
    let mut render_queue = RenderQueue::new();

    let each_tick = Closure::wrap(Box::new(move |key: u8, frame_pressed: u8| {
        let key = Key::new(key, frame_pressed);
        let cmd: GameInput<Key> = key.into();
        let _ = game.tick(cmd, &mut render_queue);
    }) as Box<FnMut(_, _)>);

    let game_loop = GameLoop::new(&each_tick);

    game_loop.start();

    each_tick.forget();
}
