use std::f64::consts::PI;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use constants::{ANIMATION_FRAME_COUNT, TILE_SIZE};
use data::{Direction, Tile};
use system::{Color, DrawGrid, UnitInterval};
use world::WorldUpdate;

pub struct CanvasEnv {
    canvas: HtmlCanvasElement,
    gc: CanvasRenderingContext2d,
    tile_size: f64,
}
impl CanvasEnv {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        (document.body().unwrap().as_ref() as &web_sys::Node)
            .append_child(canvas.as_ref() as &web_sys::Node)
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context.set_fill_style(&"rgba(0, 0, 0, 1)".into());

        CanvasEnv {
            canvas,
            gc: context,
            tile_size: TILE_SIZE,
        }
    }
}

// impl DrawGrid for CanvasEnv {
// fn setup(&mut self, tile_size: u32, width: u32, height: u32) {
// self.tile_size = tile_size as f64;
// self.canvas.set_width(width * tile_size);
// self.canvas.set_height(height * tile_size);
// }

// fn clear(&mut self) {
// self.gc.clear_rect(
// 0.0,
// 0.0,
// self.canvas.width() as f64,
// self.canvas.height() as f64,
// );
// }

// // returns current fill color
// fn set_fill_color(&mut self, color: Color) -> Color {
// color
// }

// #[inline(always)]
// fn fill_tile(
// &mut self,
// x: u32,
// y: u32,
// dir: Direction,
// size: UnitInterval,
// ) {
// let (x, y, w, h) = self.partial_tile(x, y, dir, size);

// self.gc.fill_rect(x, y, w, h);
// }

// #[inline(always)]
// fn clear_tile(
// &mut self,
// x: u32,
// y: u32,
// dir: Direction,
// size: UnitInterval,
// ) {
// let (x, y, w, h) = self.partial_tile(x, y, dir, size);

// self.gc.clear_rect(x, y, w, h);
// }

// fn circle(&mut self, x: u32, y: u32, radius: UnitInterval) {
// let x = f64::from(x) * self.tile_size;
// let y = f64::from(y) * self.tile_size;

// let r_full = self.tile_size / 2.0;
// let r = radius.scale(r_full);
// // gc.save();

// // gc.set_fill_style(&"rgba(255, 0, 0, 1)".into());
// self.gc.begin_path();
// let _ = self.gc.arc(x + r_full, y + r_full, r, 0.0, 2.0 * PI);
// self.gc.fill();

// // gc.restore();
// }
// }

impl CanvasEnv {
    fn partial_tile(
        &mut self,
        x: u32,
        y: u32,
        dir: Direction,
        size: UnitInterval,
    ) -> (f64, f64, f64, f64) {
        let x0 = f64::from(x) * self.tile_size;
        let y0 = f64::from(y) * self.tile_size;

        let long = self.tile_size;
        let short = size.scale(self.tile_size);

        let x;
        let y;
        let w;
        let h;

        match dir {
            Direction::East => {
                x = x0;
                y = y0;
                w = short;
                h = long;
            }
            Direction::West => {
                x = x0 + long - short;
                y = y0;
                w = short;
                h = long;
            }
            Direction::South => {
                x = x0;
                y = y0;
                w = long;
                h = short;
            }
            Direction::North => {
                x = x0;
                y = y0 + long - short;
                w = long;
                h = short;
            }
        }
        (x, y, w, h)
    }
}

// pub struct WorldUpdateDraw {
// update: WorldUpdate,
// current_frame: u8,
// total_frame: u8,
// }

// impl WorldUpdateDraw {
// fn draw_into<E: DrawGrid>(&mut self, mut env: DrawHandle<E>) {
// let t = UnitInterval::from_u8_and_range(
// self.current_frame,
// 0..self.total_frame,
// );

// match self.update {
// WorldUpdate::Clear { prev_block, at } => {
// match Tile::from(prev_block) {
// Tile::Snake(dir) => env.clear_tile(at.x, at.y, dir, t),
// _ => env.clear_tile(
// at.x,
// at.y,
// Direction::East,
// UnitInterval::max_value(),
// ),
// }
// }
// WorldUpdate::SetBlock { block, at } => match Tile::from(block) {
// Tile::Food => env.with_fill_color(Color::Red, |mut env| {
// env.circle(at.x, at.y, t);
// }),
// Tile::Snake(dir) => env.fill_tile(at.x, at.y, dir, t),
// _ => {}
// },
// _ => {}
// }
// }
// }
