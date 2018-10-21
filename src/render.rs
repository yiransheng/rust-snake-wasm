use std::cell::Cell;
use std::cmp;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::marker::Pinned;
use std::ops::Range;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use data::{Coordinate, Direction, Tile};
use game::Renderer;
use patch::*;
use world::WorldUpdateEff;

pub struct CanvasRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl CanvasRenderer {
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
        canvas.set_width(640);
        canvas.set_height(480);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context.set_fill_style(&"rgba(0, 0, 0, 1)".into());

        CanvasRenderer { canvas, context }
    }
}

impl Renderer for CanvasRenderer {
    type UpdateEff = Patch<WorldUpdateEff>;

    fn render<F>(&mut self, mut eff: Self::UpdateEff, callback: F)
    where
        F: Fn(),
    {
        let w = 64;
        let h = 48;

        web_sys::console::log_1(&"Render".into());

        for update in eff.iter() {
            match update {
                WorldUpdateEff::SetBlock { at, block } => {
                    let x = (at.x * 10) as f64;
                    let y = (at.y * 10) as f64;
                    self.context.fill_rect(x, y, 10.0, 10.0);
                }
                WorldUpdateEff::Clear { at, prev_block } => {
                    let x = (at.x * 10) as f64;
                    let y = (at.y * 10) as f64;
                    self.context.clear_rect(x, y, 10.0, 10.0);
                }
            }
        }

        callback();
    }
}

type Generation = u32;

#[derive(Debug, Copy, Clone)]
pub struct RenderUnit<P> {
    generation: Generation,
    duration: u8,
    at: Coordinate,
    payload: P,
}

impl<P> Ord for RenderUnit<P> {
    fn cmp(&self, other: &RenderUnit<P>) -> ::std::cmp::Ordering {
        self.generation.cmp(&other.generation)
    }
}
impl<P> PartialOrd for RenderUnit<P> {
    fn partial_cmp(&self, other: &RenderUnit<P>) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}
impl<P> PartialEq for RenderUnit<P> {
    fn eq(&self, other: &RenderUnit<P>) -> bool {
        self.generation == other.generation
    }
}
impl<P> Eq for RenderUnit<P> {}

pub trait DrawTile {
    const TILE_SIZE: f64;

    // normalized_progress: [0, 1]
    fn draw_tile(&self, gc: &CanvasRenderingContext2d, x: f64, y: f64, normalized_progress: f64);
}

impl<P> RenderUnit<P>
where
    P: DrawTile,
{
    fn draw_frame(&self, gc: &CanvasRenderingContext2d, frame: u8) -> bool {
        let Coordinate { x, y } = self.at;
        let inner_x = (x as f64) * P::TILE_SIZE;
        let inner_y = (y as f64) * P::TILE_SIZE;

        if frame < self.duration {
            let normalized_progress = (frame as f64) / (self.duration as f64);

            self.payload
                .draw_tile(gc, inner_x, inner_y, normalized_progress);

            true
        } else {
            false
        }
    }
}

pub struct RenderQueue<P> {
    queue: Vec<RenderUnit<P>>,
}

impl<P> RenderQueue<P> {
    pub fn push(&mut self, unit: RenderUnit<P>) {
        self.queue.push(unit);
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clear(&mut self) {
        self.queue.clear();
    }

    fn first_generation(&self) -> Option<Generation> {
        self.queue.first().map(|u| u.generation)
    }
    fn last_generation(&self) -> Option<Generation> {
        self.queue.last().map(|u| u.generation)
    }

    fn slice_for_generation(&self, g: Generation) -> &[RenderUnit<P>] {
        // queue typically too small to warrant fancy things like binary search
        let rng = self
            .queue
            .iter()
            .enumerate()
            .skip_while(|(_, u)| u.generation < g)
            .take_while(|(_, u)| u.generation == g)
            .map(|(i, _)| i)
            .fold(0..0, |r, i| cmp::min(r.start, i)..cmp::max(r.end, i + 1));

        &self.queue[rng]
    }
}

pub struct MyRenderer {
    generation: Generation,
    current_frame: u8,
    canvas: HtmlCanvasElement,
    gc: CanvasRenderingContext2d,
}

impl MyRenderer {
    fn tick<P: DrawTile>(&mut self, q: &mut RenderQueue<P>) {
        let mut generation_completed = false;

        {
            let render_units = q.slice_for_generation(self.generation);

            if render_units.is_empty() {
                generation_completed = true;
            } else {
                for ru in render_units {
                    if ru.draw_frame(&self.gc, self.current_frame) {
                        generation_completed = false;
                    }
                }
            }
        }

        if generation_completed {
            self.complete_generation(q);
        } else {
            self.current_frame += 1;
        }
    }

    fn complete_generation<P: DrawTile>(&mut self, q: &mut RenderQueue<P>) {
        self.generation += 1;
        self.current_frame = 0;

        if let Some(last_gen) = q.last_generation() {
            if last_gen < self.generation {
                q.clear();
            }
        }
    }
}
