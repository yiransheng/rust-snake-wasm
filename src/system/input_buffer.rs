use arraydeque::{ArrayDeque, Wrapping};

use constants::ANIMATION_FRAME_COUNT;

type InputBuffer<T> = ArrayDeque<[T; ANIMATION_FRAME_COUNT as usize], Wrapping>;

pub struct InputDblBuffer<T> {
    swapped: bool,

    first: InputBuffer<T>,
    second: InputBuffer<T>,
}

macro_rules! current {
    (&$x:ident) => {
        if $x.swapped {
            &$x.second
        } else {
            &$x.first
        }
    };
    (&mut $x:ident) => {
        if $x.swapped {
            &mut $x.second
        } else {
            &mut $x.first
        }
    };
}
macro_rules! next {
    (&$x:ident) => {
        if $x.swapped {
            &$x.first
        } else {
            &$x.second
        }
    };
    (&mut $x:ident) => {
        if $x.swapped {
            &mut $x.first
        } else {
            &mut $x.second
        }
    };
}

impl<T> InputDblBuffer<T> {
    pub fn new() -> Self {
        InputDblBuffer {
            swapped: false,
            first: ArrayDeque::new(),
            second: ArrayDeque::new(),
        }
    }

    pub fn write<V>(&mut self, x: V) -> Option<()>
    where
        T: Eq,
        V: Into<Option<T>>,
    {
        let x = x.into()?;
        let next = next!(&mut self);

        match next.back() {
            Some(prev) if prev == &x => {
                return None;
            }
            _ => {}
        }

        next.push_back(x);
        Some(())
    }

    pub fn read(&mut self) -> Option<T> {
        current!(&mut self).pop_front()
    }

    pub fn swap_when<F>(&mut self, f: F)
    where
        F: Fn(&InputBuffer<T>, &InputBuffer<T>) -> bool,
    {
        let should_swap = f(current!(&self), next!(&self));

        if should_swap {
            self.swapped = !self.swapped;
            next!(&mut self).clear();
        }
    }

    pub fn clear_both(&mut self) {
        self.first.clear();
        self.second.clear();
    }
}
