use std::cmp;
use std::iter::FromIterator;
use std::ops::Range;

use arraydeque::ArrayDeque;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::super::data::Coordinate;
use super::traits::{DrawTile, RenderSink};

pub type Generation = u32;

#[derive(Debug)]
pub struct RenderUnit<P> {
    pub generation: Generation,
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
    pub fn modify_duration<F>(&mut self, f: F)
    where
        F: Fn(u8) -> u8,
    {
        self.duration = f(self.duration)
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

        if frame < self.duration {
            let normalized_progress = (frame as f64 + 1.0) / (self.duration as f64);

            self.payload
                .draw_tile(gc, inner_x, inner_y, normalized_progress);

            true
        } else {
            self.payload.draw_tile(gc, inner_x, inner_y, 1.0);
            false
        }
    }
    pub fn prepare_canvas(&self, canvas: &HtmlCanvasElement) {
        self.payload.prepare_canvas(canvas);
    }
}

pub struct RenderQueue<P> {
    queue: ArrayDeque<[RenderUnit<P>; 8]>,
}

impl<P> RenderSink<P> for RenderQueue<P> {
    fn is_ready(&self) -> bool {
        self.queue.is_empty()
    }

    fn push(&mut self, unit: RenderUnit<P>) {
        self.queue.push_back(unit).expect("It's full");
    }
}

impl<P> FromIterator<RenderUnit<P>> for RenderQueue<P> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = RenderUnit<P>>,
    {
        let queue = iter.into_iter().collect::<ArrayDeque<[_; 8]>>();
        RenderQueue { queue }
    }
}

impl<P> RenderQueue<P> {
    pub fn new() -> Self {
        RenderQueue {
            queue: ArrayDeque::new(),
        }
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RenderUnit<P>> {
        self.queue.iter_mut()
    }

    #[inline]
    pub fn peek_generation<'a>(&'a self, g: Generation) -> impl Iterator<Item = &'a RenderUnit<P>> {
        self.queue
            .iter()
            .skip_while(move |u| u.generation < g)
            .take_while(move |u| u.generation == g)
    }

    #[inline]
    pub fn drain_generation<'a>(
        &'a mut self,
        g: Generation,
    ) -> impl Iterator<Item = RenderUnit<P>> + 'a {
        let rng = self.range_for_generation(g);

        self.queue.drain(rng)
    }

    fn range_for_generation(&self, g: Generation) -> Range<usize> {
        // queue typically too small to warrant fancy things like binary search
        self.queue
            .iter()
            .enumerate()
            .skip_while(|(_, u)| u.generation < g)
            .take_while(|(_, u)| u.generation == g)
            .map(|(i, _)| i)
            .fold(0..0, |r, i| cmp::min(r.start, i)..cmp::max(r.end, i + 1))
    }
}
