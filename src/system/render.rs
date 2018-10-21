use std::cmp;
use std::iter::FromIterator;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::super::data::Coordinate;
use super::traits::{DrawTile, RenderSink};

pub type Generation = u32;

#[derive(Debug)]
pub struct RenderUnit<P> {
    generation: Generation,
    duration: u8,
    at: Coordinate,
    payload: P,
}

impl<P> RenderUnit<P> {
    pub fn new(generation: Generation, duration: u8, at: Coordinate, payload: P) -> Self {
        RenderUnit {
            generation,
            duration,
            at,
            payload,
        }
    }
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

impl<P> RenderUnit<P>
where
    P: DrawTile,
{
    pub fn draw_frame(&self, gc: &CanvasRenderingContext2d, frame: u8) -> bool {
        let Coordinate { x, y } = self.at;
        let inner_x = (x as f64) * P::TILE_SIZE;
        let inner_y = (y as f64) * P::TILE_SIZE;

        web_sys::console::log_2(&"Frame".into(), &frame.into());

        if frame < self.duration {
            let normalized_progress = (frame as f64 + 1.0) / (self.duration as f64);

            self.payload
                .draw_tile(gc, inner_x, inner_y, normalized_progress);

            true
        } else {
            false
        }
    }
    pub fn prepare_canvas(&self, canvas: &HtmlCanvasElement) {
        self.payload.prepare_canvas(canvas);
    }
}

pub struct RenderQueue<P> {
    queue: Vec<RenderUnit<P>>,
}

impl<P> FromIterator<RenderUnit<P>> for RenderQueue<P> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = RenderUnit<P>>,
    {
        let queue = iter.into_iter().collect::<Vec<_>>();
        RenderQueue { queue }
    }
}

impl<P> RenderQueue<P> {
    pub fn new() -> Self {
        RenderQueue { queue: Vec::new() }
    }
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn first_generation(&self) -> Option<Generation> {
        self.queue.first().map(|u| u.generation)
    }
    pub fn last_generation(&self) -> Option<Generation> {
        self.queue.last().map(|u| u.generation)
    }

    pub fn slice_for_generation(&self, g: Generation) -> &[RenderUnit<P>] {
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

impl<P> RenderSink<P> for RenderQueue<P> {
    fn is_ready(&self) -> bool {
        self.queue.is_empty()
    }

    fn push(&mut self, unit: RenderUnit<P>) {
        self.queue.push(unit);
    }
}
