use std::convert::TryFrom;
use std::ops::Range;

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

pub struct OutOfRange(f64);

impl TryFrom<f64> for UnitInterval {
    type Error = OutOfRange;

    fn try_from(v: f64) -> Result<Self, Self::Error> {
        if v.is_nan() || v < 0.0 || v > 1.0 {
            return Err(OutOfRange(v));
        }

        Ok(UnitInterval(v))
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

    fn with_defaults<F>(&mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(DrawHandle<Self>),
    {
        let handle = DrawHandle {
            grid: self,
            prev_color: None,
        };

        f(handle);
    }

    fn with_fill_color<C, F>(&mut self, color: C, mut f: F)
    where
        Self: Sized,
        C: Into<Color>,
        F: FnMut(DrawHandle<Self>),
    {
        let prev_color = self.set_fill_color(color.into());

        let handle = DrawHandle {
            grid: self,
            prev_color: Some(prev_color),
        };

        f(handle);
    }
}

pub struct DrawHandle<'a, G: DrawGrid> {
    grid: &'a mut G,
    prev_color: Option<Color>,
}

impl<'a, G> DrawHandle<'a, G>
where
    G: DrawGrid,
{
    #[inline]
    pub fn circle<U: Into<u32>>(&mut self, x: U, y: U, radius: UnitInterval) {
        self.grid.circle(x.into(), y.into(), radius);
    }

    #[inline]
    pub fn fill_tile<U: Into<u32>>(
        &mut self,
        x: U,
        y: U,
        dir: Direction,
        size: UnitInterval,
    ) {
        self.grid.fill_tile(x.into(), y.into(), dir, size);
    }

    #[inline]
    pub fn clear_tile<U: Into<u32>>(
        &mut self,
        x: U,
        y: U,
        dir: Direction,
        size: UnitInterval,
    ) {
        self.grid.clear_tile(x.into(), y.into(), dir, size);
    }

    pub fn set_fill_color(self, color: Color) -> Self {
        let prev_color =
            self.prev_color.unwrap_or(self.grid.set_fill_color(color));

        DrawHandle {
            grid: self.grid,
            prev_color: Some(prev_color),
        }
    }
}

impl<'a, G: DrawGrid> Drop for DrawHandle<'a, G> {
    fn drop(&mut self) {
        match self.prev_color.take() {
            Some(color) => {
                self.grid.set_fill_color(color);
            }
            _ => {}
        }
    }
}
