use std::f64::consts::PI;

use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use data::Direction;
use system::{DrawTile, GameSystem, Generation, Never, RenderQueue, RenderUnit};
use world::WorldUpdate;

pub struct CanvasRenderer<P> {
    generation: Generation,
    current_frame: u8,
    canvas: HtmlCanvasElement,
    gc: CanvasRenderingContext2d,

    pending_updates: SmallVec<[RenderUnit<P>; 8]>,
}

impl GameSystem for CanvasRenderer<WorldUpdate> {
    type Msg = WorldUpdate;
    type InputCmd = ();
    type GameOver = Never;

    fn start_up(&mut self, q: &mut RenderQueue<Self::Msg>) {
        self.setup(q);
    }
    fn tick(&mut self, _cmd: (), q: &mut RenderQueue<Self::Msg>) -> Result<(), Never> {
        self.render(q);

        Ok(())
    }

    fn tear_down(&mut self) {}
}

impl<P> CanvasRenderer<P> {
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

        CanvasRenderer {
            canvas,
            gc: context,
            generation: Generation::default(),
            current_frame: 0,

            pending_updates: SmallVec::new(),
        }
    }
    #[inline]
    fn setup(&mut self, q: &mut RenderQueue<P>)
    where
        P: DrawTile,
    {
        let gen_0 = Generation::default();
        let render_units = q.peek_generation(gen_0);
        for ru in render_units {
            ru.prepare_canvas(&self.canvas);
        }
        self.clear();

        self.generation = gen_0;
    }
    #[inline]
    fn clear(&self) {
        self.gc.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }
    #[inline]
    fn render(&mut self, q: &mut RenderQueue<P>)
    where
        P: DrawTile,
    {
        if self.pending_updates.is_empty() {
            self.fetch_updates(q);
        }

        let mut generation_completed = true;

        for ru in &self.pending_updates {
            if ru.draw_frame(&self.gc, self.current_frame) {
                generation_completed = false;
            }
        }

        if generation_completed {
            self.complete_generation(q);
        } else {
            self.current_frame += 1;
        }
    }

    fn fetch_updates(&mut self, q: &mut RenderQueue<P>) {
        for unit in q.drain_generation(self.generation) {
            self.pending_updates.push(unit);
        }
    }

    fn complete_generation(&mut self, q: &mut RenderQueue<P>) {
        self.generation += 1;
        self.current_frame = 0;

        self.pending_updates.clear();
        self.fetch_updates(q);
    }
}

impl DrawTile for WorldUpdate {
    const TILE_SIZE: f64 = 16.0;

    fn prepare_canvas(&self, canvas: &HtmlCanvasElement) {
        match self {
            WorldUpdate::SetWorldSize(w, h) => {
                let size = Self::TILE_SIZE as u32;
                canvas.set_width(*w * size);
                canvas.set_height(*h * size);
            }
            _ => {}
        }
    }

    fn draw_tile(&self, gc: &CanvasRenderingContext2d, x: f64, y: f64, normalized_progress: f64) {
        match self {
            WorldUpdate::SetWorldSize(_, _) => {}
            WorldUpdate::SetBlock { block } => {
                if block.is_snake() {
                    let ts = Self::TILE_SIZE;
                    let dir = Direction::from(*block);
                    let length = normalized_progress * ts;
                    match dir {
                        Direction::North => gc.fill_rect(x, y + ts - length, ts, length),
                        Direction::South => gc.fill_rect(x, y, ts, length),
                        Direction::East => gc.fill_rect(x, y, length, ts),
                        Direction::West => gc.fill_rect(x + ts - length, y, length, ts),
                    }
                } else if block.is_food() {
                    let r_full = Self::TILE_SIZE / 2.0;
                    let r = r_full * normalized_progress;
                    gc.save();

                    gc.set_fill_style(&"rgba(255, 0, 0, 1)".into());
                    gc.begin_path();
                    let _ = gc.arc(x + r_full, y + r_full, r, 0.0, 2.0 * PI);
                    gc.fill();

                    gc.restore();
                }
            }
            WorldUpdate::Clear { prev_block } => {
                let ts = Self::TILE_SIZE;
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
