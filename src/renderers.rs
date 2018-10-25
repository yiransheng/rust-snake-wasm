use std::f64::consts::PI;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use constants::{ANIMATION_FRAME_COUNT, TILE_SIZE};
use data::Direction;
use system::{CanvasTile, Render};
use world::WorldUpdate;

pub struct CanvasEnv {
    canvas: HtmlCanvasElement,
    gc: CanvasRenderingContext2d,
}
impl CanvasEnv {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
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
        }
    }
}

pub struct BlockRenderer<J> {
    current_frame: u8,
    job: J,
}
impl<J> BlockRenderer<J> {}

impl<J: CanvasTile> Render for BlockRenderer<J> {
    type Update = J;
    type Env = CanvasEnv;

    fn create(u: Self::Update, env: &mut Self::Env) -> Self {
        u.setup_canvas(&env.canvas);

        BlockRenderer {
            current_frame: 0,
            job: u,
        }
    }

    fn render(&mut self, env: &mut Self::Env) -> Option<()> {
        if self.current_frame >= ANIMATION_FRAME_COUNT {
            self.job.draw_tile(&env.gc, 1.0);
            None
        } else {
            let progress = (self.current_frame as f64) / (ANIMATION_FRAME_COUNT as f64);
            self.job.draw_tile(&env.gc, progress);

            self.current_frame += 1;
            Some(())
        }
    }
}

impl CanvasTile for WorldUpdate {
    fn setup_canvas(&self, canvas: &HtmlCanvasElement) {
        match self {
            WorldUpdate::SetWorldSize(w, h) => {
                let size = TILE_SIZE as u32;
                canvas.set_width(*w * size);
                canvas.set_height(*h * size);
            }
            _ => {}
        }
    }

    fn draw_tile(&self, gc: &CanvasRenderingContext2d, normalized_progress: f64) {
        match self {
            WorldUpdate::SetWorldSize(_, _) => {}
            WorldUpdate::SetBlock { block, at } => {
                let x = f64::from(at.x) * TILE_SIZE;
                let y = f64::from(at.y) * TILE_SIZE;

                if block.is_snake() {
                    let ts = TILE_SIZE;
                    let dir = Direction::from(*block);
                    let length = normalized_progress * ts;
                    match dir {
                        Direction::North => gc.fill_rect(x, y + ts - length, ts, length),
                        Direction::South => gc.fill_rect(x, y, ts, length),
                        Direction::East => gc.fill_rect(x, y, length, ts),
                        Direction::West => gc.fill_rect(x + ts - length, y, length, ts),
                    }
                } else if block.is_food() {
                    let r_full = TILE_SIZE / 2.0;
                    let r = r_full * normalized_progress;
                    gc.save();

                    gc.set_fill_style(&"rgba(255, 0, 0, 1)".into());
                    gc.begin_path();
                    let _ = gc.arc(x + r_full, y + r_full, r, 0.0, 2.0 * PI);
                    gc.fill();

                    gc.restore();
                }
            }
            WorldUpdate::Clear { prev_block, at } => {
                let x = f64::from(at.x) * TILE_SIZE;
                let y = f64::from(at.y) * TILE_SIZE;

                let ts = TILE_SIZE;

                if prev_block.is_snake() {
                    let dir = Direction::from(*prev_block);
                    let length = normalized_progress * ts;
                    match dir {
                        Direction::North => gc.clear_rect(x, y + ts - length, ts, length),
                        Direction::South => gc.clear_rect(x, y, ts, length),
                        Direction::East => gc.clear_rect(x, y, length, ts),
                        Direction::West => gc.clear_rect(x + ts - length, y, length, ts),
                    }
                } else if normalized_progress == 1.0 {
                    gc.clear_rect(x, y, ts, ts);
                }
            }
        }
    }
}
