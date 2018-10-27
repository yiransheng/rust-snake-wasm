use std::convert::TryFrom;
use std::ops::{Deref, DerefMut, Range};
use std::ops::{Generator, GeneratorState};

use data::Direction;

// https://english.stackexchange.com/questions/275734/a-word-for-a-value-between-0-and-1-inclusive
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct UnitInterval(f64);

impl UnitInterval {
    pub fn min_value() -> Self {
        UnitInterval(0.0)
    }
    pub fn max_value() -> Self {
        UnitInterval(1.0)
    }

    pub fn from_u8_and_range(x: u8, range: Range<u8>) -> Self {
        assert!(range.end > range.start);

        if x < range.start {
            Self::min_value()
        } else if x >= range.end {
            Self::max_value()
        } else {
            let a = x - range.start + 1;
            let b = range.end - range.start;

            let v = (a as f64) / (b as f64);

            UnitInterval(v)
        }
    }

    #[inline(always)]
    pub fn scale(self, v: f64) -> f64 {
        self.0 * v
    }
}

impl Eq for UnitInterval {}

impl Default for UnitInterval {
    fn default() -> Self {
        Self::min_value()
    }
}

#[derive(Copy, Clone)]
pub enum Color {
    Red,
    Black,
}

pub trait DrawGrid {
    fn setup(&mut self, tile_size: u32, width: u32, height: u32);

    fn clear(&mut self);

    // returns current fill color
    fn set_fill_color(&mut self, color: Color) -> Color;

    fn circle(&mut self, x: u32, y: u32, radius: UnitInterval);

    fn fill_tile(&mut self, x: u32, y: u32, dir: Direction, size: UnitInterval);

    fn clear_tile(
        &mut self,
        x: u32,
        y: u32,
        dir: Direction,
        size: UnitInterval,
    );
}

pub trait IncrRender: Iterator<Item = ()> {
    type Env: DrawGrid;
    type Patch;

    fn new_patch(u: Self::Patch, env: &mut Self::Env) -> Self;

    fn to_generator(self) -> IncrRenderGen<Self>
    where
        Self: Sized,
    {
        IncrRenderGen::Created(self)
    }
}

pub enum IncrRenderGen<R> {
    Created(R),
    InProgress(R),
    Done,
}
impl<R> Generator for IncrRenderGen<R>
where
    R: IncrRender,
{
    type Yield = ();
    type Return = ();

    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        let this = ::std::mem::replace(self, IncrRenderGen::Done);

        match this {
            // yield once
            IncrRenderGen::Created(mut renderer) => match renderer.next() {
                Some(_) => {
                    ::std::mem::replace(
                        self,
                        IncrRenderGen::InProgress(renderer),
                    );
                    GeneratorState::Yielded(())
                }
                _ => {
                    ::std::mem::replace(self, IncrRenderGen::Done);
                    GeneratorState::Yielded(())
                }
            },
            // yield zero or more times
            IncrRenderGen::InProgress(mut renderer) => match renderer.next() {
                Some(_) => GeneratorState::Yielded(()),
                _ => {
                    ::std::mem::replace(self, IncrRenderGen::Done);
                    GeneratorState::Complete(())
                }
            },
            // return
            IncrRenderGen::Done => GeneratorState::Complete(()),
        }
    }
}
