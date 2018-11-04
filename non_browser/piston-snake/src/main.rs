#![feature(generators, generator_trait)]

extern crate piston_window;
extern crate snake_wasm;
extern crate vec_map;

use std::ops::Generator;

use piston_window::*;

use snake_wasm::acceleration::{RenderSpeed, VariableFrame};
use snake_wasm::data::{Direction, Wrapping};
use snake_wasm::dead::Dead;
use snake_wasm::system::{Either, Stateful};
use snake_wasm::world::{World, WorldBuilder};
use snake_wasm::SmallRng;

use self::channel::channel;
use self::constants::*;
use self::key::KeyWrapper;
use self::render::{Clear, DrawUpdate, TilePatch, TileUpdate};
use self::render_state::Tiles;
use self::throttle::throttle;

mod channel;
mod constants;
mod key;
mod render;
mod render_state;
mod throttle;

fn create_window() -> PistonWindow {
    let width = WIDTH * TILE_SIZE;
    let height = HEIGHT * TILE_SIZE;

    let mut window: PistonWindow =
        WindowSettings::new("Piston Snake", [width, height])
            .exit_on_esc(true)
            .build()
            .unwrap();

    window.set_max_fps(60);
    window.set_ups(60);

    window
}

fn main() {
    let mut window = create_window();

    let facing = Direction::East;

    let world: World<SmallRng, Wrapping> = WorldBuilder::new()
        .width(WIDTH as u16)
        .height(HEIGHT as u16)
        .set_snake(1, 1)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .extend(facing)
        .build_with_seed([123; 16]);

    let (tx, rx) = channel();

    let game = world
        .zip_with(RenderSpeed::new(facing), VariableFrame::pack)
        .alternating::<KeyWrapper, _>(Dead::new())
        .make_game(TileUpdate::new(tx));

    let (cmd_buffer, mut generator) = game.new_game::<DrawUpdate, KeyWrapper>();

    let mut tiles = Tiles::new();

    let mut logic_tick = throttle(1.0 / 60.0, move |_event: &Event| unsafe {
        generator.resume();
    });

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            cmd_buffer.borrow_mut().write(KeyWrapper::from(key));
        }

        logic_tick.run(&event);

        // gymnastics to turn a system design for retained rendering back
        // to immediate rendering, but collecting render patches and
        // rebuild state
        if let Some(patch) = rx.recv() {
            match patch {
                Either::Left(tile_patch) => {
                    tiles.apply_patch(tile_patch);
                }
                Either::Right(Clear { .. }) => {
                    tiles.clear();
                }
            }
        }

        window.draw_2d(&event, |c, g| {
            clear(BACKGROUND_COLOR, g);

            tiles.iter().for_each(|TilePatch { rect, region, .. }| {
                rect.draw(region, &c.draw_state, c.transform, g);
            });
        });
    }
}
