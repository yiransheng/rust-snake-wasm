#![feature(generators, generator_trait)]

extern crate snake_wasm;
extern crate termion;

use std::io::stdout;
use std::ops::{Generator, GeneratorState};
use std::thread::sleep;
use std::time::{Duration, Instant};

use termion::async_stdin;

use snake_wasm::data::{Direction, Wrapping};
use snake_wasm::dead::Dead;
use snake_wasm::system::Stateful;
use snake_wasm::world::{World, WorldBuilder};
use snake_wasm::{SmallRng, WorldUpdateDraw};

use self::key::Key;
use self::term_env::TermEnv;

mod key;
mod term_env;

fn main() {
    let stdout = stdout();
    let term_env = TermEnv::wrap(stdout.lock());

    let mut stdin = async_stdin();

    let facing = Direction::East;
    let world: World<SmallRng, Wrapping> = WorldBuilder::new()
        .width(48)
        .height(24)
        .set_snake(1, 1)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .build_with_seed([123; 16]);

    let game = world.alternating::<Key, _>(Dead::new()).make_game(term_env);

    let (tx, mut generator) = game.new_game::<WorldUpdateDraw, Key>();
    let interval = Duration::from_millis(16);

    let mut before = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(before);

        before = now;

        let key = Key::read_from(&mut stdin);

        if key.is_quit() {
            break;
        } else {
            tx.send(key);
        }

        if dt < interval {
            sleep(interval - dt);
            continue;
        }

        unsafe {
            match generator.resume() {
                GeneratorState::Yielded(_) => continue,
                GeneratorState::Complete(_) => break,
            }
        }
    }
}
