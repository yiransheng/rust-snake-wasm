extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;

extern crate either;
extern crate rand;

use wasm_bindgen::prelude::*;

use either::Either;

use rand::rngs::SmallRng;

mod data;
mod render;
mod system;
mod world;

use data::Direction;
use render::CanvasRenderer;
use system::{GameInput, GameSystem, RenderQueue, StartGame};
use world::WorldBuilder;

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

#[wasm_bindgen]
pub fn main() {
    let world = WorldBuilder::new()
        .width(64)
        .height(48)
        .set_snake(1, 1)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .build_with_seed::<SmallRng>([123; 16]);

    let mut game = world.with_renderer(CanvasRenderer::new()).with_play_state();
    let mut render_queue = RenderQueue::new();

    let each_tick = Closure::wrap(Box::new(move |n: f64| {
        let cmd: GameInput<Option<Direction>>;
        if n % 60.0 == 1.0 {
            cmd = Either::Left(StartGame);
        } else {
            cmd = Either::Right(None);
        }

        let _ = game.tick(cmd, &mut render_queue);
    }) as Box<FnMut(_)>);

    let game_loop = GameLoop::new(&each_tick);

    game_loop.start();

    each_tick.forget();
}
