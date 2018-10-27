use std::f64::consts::PI;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use constants::{ANIMATION_FRAME_COUNT, TILE_SIZE};
use data::{Direction, Tile};
use system::{CanvasTile, Color, DrawGrid, Render, UnitInterval};
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

#[derive(Debug, Copy, Clone)]
pub enum RenderStep {
    Instant,
    Frames(u8),
}
impl Default for RenderStep {
    fn default() -> Self {
        RenderStep::Frames(1)
    }
}
impl RenderStep {
    pub fn from_frame_count(f: u8) -> Self {
        match f {
            0 => RenderStep::Frames(1),
            x if x < ANIMATION_FRAME_COUNT => RenderStep::Frames(x),
            _ => RenderStep::Instant,
        }
    }
    pub fn from_factor(f: f64) -> Self {
        if f <= 0.0 || f > 1.0 {
            RenderStep::Frames(ANIMATION_FRAME_COUNT)
        } else {
            let frames = (ANIMATION_FRAME_COUNT as f64) * f;
            let frames = frames.ceil() as u8;

            RenderStep::Frames(frames)
        }
    }
}

pub struct BlockRenderer<J> {
    current_frame: u8,
    job: J,
    render_step: RenderStep,
}

impl<J> Render for BlockRenderer<J>
where
    J: CanvasTile + Copy + Into<RenderStep>,
{
    type Update = J;
    type Env = CanvasEnv;

    fn create(u: Self::Update, env: &mut Self::Env) -> Self {
        u.setup_canvas(&env.canvas);

        let render_step = u.into();

        BlockRenderer {
            current_frame: 0,
            job: u,
            render_step,
        }
    }

    fn render(&mut self, env: &mut Self::Env) -> Option<()> {
        match self.render_step {
            RenderStep::Instant => {
                self.job.draw_tile(&env.gc, 1.0);
                None
            }
            RenderStep::Frames(step) => {
                if self.current_frame >= ANIMATION_FRAME_COUNT {
                    self.job.draw_tile(&env.gc, 1.0);
                    None
                } else {
                    let progress = (self.current_frame as f64) / (ANIMATION_FRAME_COUNT as f64);
                    self.job.draw_tile(&env.gc, progress);

                    self.current_frame += if step > 0 { step } else { 1 };
                    Some(())
                }
            }
        }
    }
}

impl Into<RenderStep> for WorldUpdate {
    fn into(self) -> RenderStep {
        match self {
            WorldUpdate::SetWorldSize(_, _) => RenderStep::Instant,
            _ => RenderStep::default(),
        }
    }
}

impl<T> From<(RenderStep, T)> for RenderStep {
    fn from(tp: (RenderStep, T)) -> RenderStep {
        tp.0
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

                match Tile::from(*block) {
                    Tile::Snake => {
                        let ts = TILE_SIZE - 4.0;
                        let dir = block.into_direction_unchecked();
                        let length = normalized_progress * ts;

                        let x = x + 2.0;
                        let y = y + 2.0;

                        match dir {
                            Direction::North => gc.fill_rect(x, y + ts - length, ts, length),
                            Direction::South => gc.fill_rect(x, y, ts, length),
                            Direction::East => gc.fill_rect(x, y, length, ts),
                            Direction::West => gc.fill_rect(x + ts - length, y, length, ts),
                        }
                    }
                    Tile::Food => {
                        let r_full = TILE_SIZE / 2.0;
                        let r = r_full * normalized_progress;
                        gc.save();

                        gc.set_fill_style(&"rgba(255, 0, 0, 1)".into());
                        gc.begin_path();
                        let _ = gc.arc(x + r_full, y + r_full, r, 0.0, 2.0 * PI);
                        gc.fill();

                        gc.restore();
                    }
                    _ => {}
                }
            }
            WorldUpdate::Clear { prev_block, at } => {
                let x = f64::from(at.x) * TILE_SIZE;
                let y = f64::from(at.y) * TILE_SIZE;

                let ts = TILE_SIZE;
                let dir: Option<Direction> = (*prev_block).into();

                match dir {
                    Some(dir) => {
                        let length = normalized_progress * ts;
                        match dir {
                            Direction::North => gc.clear_rect(x, y + ts - length, ts, length),
                            Direction::South => gc.clear_rect(x, y, ts, length),
                            Direction::East => gc.clear_rect(x, y, length, ts),
                            Direction::West => gc.clear_rect(x + ts - length, y, length, ts),
                        }
                    }
                    _ => {
                        gc.clear_rect(x, y, ts, ts);
                    }
                }
            }
        }
    }
}

pub struct WorldUpdateDraw {
    update: WorldUpdate,
    current_frame: u8,
    total_frame: u8,
}

impl WorldUpdateDraw {
    fn draw_into<E: DrawGrid>(&mut self, env: &mut E) {
        let t = UnitInterval::from_u8_and_range(self.current_frame, 0..self.total_frame);

        match self.update {
            WorldUpdate::Clear { prev_block, at } => {
                env.with_defaults(|mut handle| match Tile::from(prev_block) {
                    Tile::Snake => handle.clear_tile(
                        at.x as u32,
                        at.y as u32,
                        prev_block.into_direction_unchecked(),
                        t,
                    ),
                    _ => handle.clear_tile(
                        at.x as u32,
                        at.y as u32,
                        Direction::East,
                        UnitInterval::max_value(),
                    ),
                });
            }
            WorldUpdate::SetBlock { block, at } => match Tile::from(block) {
                Tile::Food => env.with_fill_color(Color::Red, |mut handle| {
                    handle.circle(at.x as u32, at.y as u32, t);
                }),
                Tile::Snake => env.with_defaults(|mut handle| {
                    handle.fill_tile(
                        at.x as u32,
                        at.y as u32,
                        block.into_direction_unchecked(),
                        t,
                    );
                }),
                _ => {}
            },
            _ => {}
        }
    }
}
