use num_traits::{ToPrimitive, Unsigned};

use std::ops::Range;
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
        assert!(range.start != range.end);

        let start;
        let end;

        if range.start > range.end {
            start = range.end;
            end = range.start;
        } else {
            start = range.start;
            end = range.end;
        }

        if x < start {
            Self::min_value()
        } else if x >= end {
            Self::max_value()
        } else {
            let a = x - start + 1;
            let b = end - start;

            let v = (a as f64) / (b as f64);

            UnitInterval(v)
        }
    }

    #[inline(always)]
    pub fn scale(self, v: f64) -> f64 {
        self.0 * v
    }
    #[inline(always)]
    #[allow(dead_code)]
    pub fn supplement(self) -> Self {
        UnitInterval(1.0 - self.0)
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

impl Color {
    pub fn to_rgb(self) -> &'static str {
        match self {
            Color::Black => "rgb(34, 34, 34)",
            Color::Red => "rgb(240, 10, 10)",
        }
    }
}

pub trait DrawGrid {
    fn setup<T: ToPrimitive + Unsigned>(
        &mut self,
        tile_size: T,
        width: T,
        height: T,
    );

    fn clear(&mut self);

    // returns current fill color
    fn set_fill_color(&mut self, color: Color) -> Color;

    fn circle<T: ToPrimitive + Unsigned>(
        &mut self,
        x: T,
        y: T,
        radius: UnitInterval,
    );

    fn fill_tile<T: ToPrimitive + Unsigned>(
        &mut self,
        x: T,
        y: T,
        dir: Direction,
        size: UnitInterval,
    );

    fn clear_tile<T: ToPrimitive + Unsigned>(
        &mut self,
        x: T,
        y: T,
        dir: Direction,
        size: UnitInterval,
    );

    fn show_game_over(&mut self);

    fn with_fill_color<F>(&mut self, color: Color, mut f: F)
    where
        Self: Sized,
        F: FnMut(&mut Self),
    {
        let prev_color = self.set_fill_color(color);

        f(self);

        self.set_fill_color(prev_color);
    }
}

pub trait IncrRender {
    type Env: DrawGrid;
    type Patch;

    fn new_patch(u: Self::Patch) -> Self;

    fn render(&mut self, env: &mut Self::Env) -> Option<()>;

    fn to_generator(self, env: &mut Self::Env) -> IncrRenderGen<Self, Self::Env>
    where
        Self: Sized,
    {
        IncrRenderGen::Created(self, env)
    }
}

pub enum IncrRenderGen<'a, R, E> {
    Created(R, &'a mut E),
    InProgress(R, &'a mut E),
    Done,
}
impl<'a, R, E> Generator for IncrRenderGen<'a, R, E>
where
    R: IncrRender<Env = E>,
    E: DrawGrid,
{
    type Yield = ();
    type Return = ();

    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        let this = ::std::mem::replace(self, IncrRenderGen::Done);

        match this {
            // yield once
            IncrRenderGen::Created(mut renderer, env) => {
                match renderer.render(env) {
                    Some(_) => {
                        ::std::mem::replace(
                            self,
                            IncrRenderGen::InProgress(renderer, env),
                        );
                        GeneratorState::Yielded(())
                    }
                    _ => {
                        ::std::mem::replace(self, IncrRenderGen::Done);
                        GeneratorState::Yielded(())
                    }
                }
            }
            // yield zero or more times
            IncrRenderGen::InProgress(mut renderer, env) => {
                match renderer.render(env) {
                    Some(_) => {
                        ::std::mem::replace(
                            self,
                            IncrRenderGen::InProgress(renderer, env),
                        );
                        GeneratorState::Yielded(())
                    }
                    _ => {
                        ::std::mem::replace(self, IncrRenderGen::Done);
                        GeneratorState::Complete(())
                    }
                }
            }
            // return
            IncrRenderGen::Done => GeneratorState::Complete(()),
        }
    }
}
