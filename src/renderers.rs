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
    tile_size: f64,
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
            tile_size: TILE_SIZE,
        }
    }
}

impl DrawGrid for CanvasEnv {
    fn setup(&mut self, tile_size: u32, width: u32, height: u32) {
        self.tile_size = tile_size as f64;
        self.canvas.set_width(width * tile_size);
        self.canvas.set_height(height * tile_size);
    }

    fn clear(&mut self) {
        self.gc.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }

    // returns current fill color
    fn set_fill_color(&mut self, color: Color) -> Color {
        color
    }

    #[inline(always)]
    fn fill_tile(&mut self, x: u32, y: u32, dir: Direction, size: UnitInterval) {
        let (x, y, w, h) = self.partial_tile(x, y, dir, size);

        self.gc.fill_rect(x, y, w, h);
    }

    #[inline(always)]
    fn clear_tile(&mut self, x: u32, y: u32, dir: Direction, size: UnitInterval) {
        let (x, y, w, h) = self.partial_tile(x, y, dir, size);

        self.gc.clear_rect(x, y, w, h);
    }

    fn circle(&mut self, x: u32, y: u32, radius: UnitInterval) {
        let x = f64::from(x) * self.tile_size;
        let y = f64::from(y) * self.tile_size;

        let r_full = self.tile_size / 2.0;
        let r = radius.scale(r_full);
        // gc.save();

        // gc.set_fill_style(&"rgba(255, 0, 0, 1)".into());
        self.gc.begin_path();
        let _ = self.gc.arc(x + r_full, y + r_full, r, 0.0, 2.0 * PI);
        self.gc.fill();

        // gc.restore();
    }
}

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
                    Tile::Snake => {
                        handle.clear_tile(at.x, at.y, prev_block.into_direction_unchecked(), t)
                    }
                    _ => handle.clear_tile(at.x, at.y, Direction::East, UnitInterval::max_value()),
                });
            }
            WorldUpdate::SetBlock { block, at } => match Tile::from(block) {
                Tile::Food => env.with_fill_color(Color::Red, |mut handle| {
                    handle.circle(at.x, at.y, t);
                }),
                Tile::Snake => env.with_defaults(|mut handle| {
                    handle.fill_tile(at.x, at.y, block.into_direction_unchecked(), t);
                }),
                _ => {}
            },
            _ => {}
        }
    }
}
