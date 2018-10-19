use std::mem;

#[derive(Debug)]
pub enum Patch<T> {
    Empty,
    One(T),
    Two(T, T),
    Many(Vec<T>),
}

impl<T: Copy> Patch<T> {
    fn iter(&mut self) -> PatchIter<T> {
        match *self {
            Patch::Empty => PatchIter::Empty,
            Patch::One(x) => PatchIter::One(x),
            Patch::Two(x, y) => PatchIter::Two(x, y),
            Patch::Many(ref mut xs) => PatchIter::Many(xs.drain(..)),
        }
    }
}

macro_rules! patch {
    () => (
        Patch::Empty
    );
    ($x:expr) => (
        Patch::One($x)
    );
    ($x:expr, $y:expr) => (
        Patch::Two($x, $y)
    );
    ($($more:expr),+) => (
        Patch::Many(vec!($($more),*))
    );
}

pub enum PatchIter<'a, T> {
    Empty,
    One(T),
    Two(T, T),
    Many(::std::vec::Drain<'a, T>),
}

impl<'a, T: Copy> Iterator for PatchIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            PatchIter::Empty => None,
            PatchIter::One(x) => {
                mem::replace(self, PatchIter::Empty);
                Some(x)
            }
            PatchIter::Two(a, b) => {
                mem::replace(self, PatchIter::One(b));
                Some(a)
            }
            PatchIter::Many(ref mut d) => d.next(),
        }
    }
}
