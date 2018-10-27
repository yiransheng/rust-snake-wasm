use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::convert::AsRef;
use std::iter::{Fuse, IntoIterator, Map, Zip};
use std::marker::PhantomData;
use std::rc::Rc;
use std::slice::Iter;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use std::ops::{Generator, GeneratorState};

use arraydeque::{ArrayDeque, Wrapping};
use void::Void;

use super::render::{DrawGrid, IncrRender};
use constants::ANIMATION_FRAME_COUNT;

pub struct GameOver;

impl Into<GameOver> for Void {
    fn into(self) -> GameOver {
        unreachable!()
    }
}

pub trait Model<'m> {
    type Cmd;
    type State: IntoIterator<Item = Self::Update> + 'm;
    type Update;
    type Error: Into<GameOver>;

    fn initialize(&'m mut self) -> Self::State;

    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error>;

    fn tear_down(&mut self);

    #[inline]
    fn make_game<E>(self, env: E) -> Box<Game<Self, E>>
    where
        Self: Sized,
        E: 'static,
    {
        Box::new(Game { model: self, env })
    }

    #[inline]
    fn zip_with<T, R, F>(self, other: R, f: F) -> ZipWith<Self, R, F>
    where
        R: Model<'m, Cmd = Self::Cmd, Error = Void>,
        F: Fn((Self::Update, R::Update)) -> T + 'm,
        Self::Cmd: Copy,
        Self: Sized,
    {
        ZipWith {
            left: self,
            right: other,
            f,
        }
    }
}

pub struct ZipWith<L, R, F> {
    left: L,
    right: R,
    f: F,
}

impl<'m, L, R, T, F> Model<'m> for ZipWith<L, R, F>
where
    L: Model<'m>,
    R: Model<'m, Cmd = L::Cmd, Error = Void>,
    F: Fn((L::Update, R::Update)) -> T + 'm,
    L::Cmd: Copy,
{
    type Cmd = L::Cmd;
    type Update = T;
    type State = Map<
        Zip<
            <L::State as IntoIterator>::IntoIter,
            <R::State as IntoIterator>::IntoIter,
        >,
        &'m F,
    >;
    type Error = L::Error;

    fn initialize(&'m mut self) -> Self::State {
        self.left
            .initialize()
            .into_iter()
            .zip(self.right.initialize().into_iter())
            .map(&self.f)
    }

    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error> {
        let ul = self.left.step(cmd)?;
        let ur = self.right.step(cmd).unwrap();

        match (ul, ur) {
            (Some(x), Some(y)) => {
                let update = (self.f)((x, y));
                Ok(Some(update))
            }
            _ => Ok(None),
        }
    }

    fn tear_down(&mut self) {
        self.right.tear_down();
        self.left.tear_down();
    }
}

/*
 * pub struct Empty<C, U> {
 *     _cmd: PhantomData<C>,
 *     _update: PhantomData<U>,
 * }
 *
 * impl<'m, C, U> Model<'m> for Empty<C, U>
 * where
 *     U: 'm,
 * {
 *     type Cmd = C;
 *     type Update = U;
 *     type Error = GameOver;
 *     type State = ::std::iter::Empty<U>;
 *
 *     fn initialize(&'m mut self) -> Self::State {
 *         ::std::iter::empty()
 *     }
 *
 *     fn step(&mut self, cmd: Option<Self::Cmd>) -> Result<Self::Update, Self::Error> {
 *         Err(GameOver)
 *     }
 *
 *     fn tear_down(&mut self) {}
 * }
 *
 * pub struct Replay<C, U> {
 *     _cmd: PhantomData<C>,
 *     updates: Vec<U>,
 *     index: usize,
 * }
 * impl<'m, C, U> Model<'m> for Replay<C, U>
 * where
 *     U: Clone + 'm,
 * {
 *     type Cmd = C;
 *     type Update = U;
 *     type Error = GameOver;
 *     type State = ::std::iter::Empty<U>;
 *
 *     fn initialize(&'m mut self) -> Self::State {
 *         self.index = 0;
 *         ::std::iter::empty()
 *     }
 *
 *     fn step(&mut self, _cmd: Option<Self::Cmd>) -> Result<Self::Update, Self::Error> {
 *         self.updates.get(self.index).cloned().ok_or(GameOver)
 *     }
 *
 *     fn tear_down(&mut self) {}
 * }
 *
 */
type InputBuffer<T> = ArrayDeque<[T; ANIMATION_FRAME_COUNT as usize], Wrapping>;

pub struct InputDblBuffer<T> {
    swapped: bool,

    first: InputBuffer<T>,
    second: InputBuffer<T>,
}

impl<T> InputDblBuffer<T> {
    fn new() -> Self {
        InputDblBuffer {
            swapped: false,
            first: ArrayDeque::new(),
            second: ArrayDeque::new(),
        }
    }

    pub fn write(&mut self, x: T) -> Option<()>
    where
        T: Eq,
    {
        let next = self.next();

        match next.back() {
            Some(prev) if prev == &x => {
                return None;
            }
            _ => {}
        }

        next.push_back(x);
        Some(())
    }

    fn current(&mut self) -> &mut InputBuffer<T> {
        if self.swapped {
            &mut self.first
        } else {
            &mut self.second
        }
    }
    fn next(&mut self) -> &mut InputBuffer<T> {
        if self.swapped {
            &mut self.second
        } else {
            &mut self.first
        }
    }
    fn swap_if<F>(&mut self, f: F)
    where
        F: Fn(&InputBuffer<T>, &InputBuffer<T>) -> bool,
    {
        let should_swap;
        {
            let curr;
            let next;

            if self.swapped {
                curr = &self.first;
                next = &self.second;
            } else {
                curr = &self.second;
                next = &self.first;
            }

            should_swap = f(curr, next);
        }
        if should_swap {
            self.swapped = !self.swapped;
            self.next().clear();
        }
    }
    fn read(&mut self) -> Option<T> {
        self.current().pop_front()
    }
    fn clear_both(&mut self) {
        self.first.clear();
        self.second.clear();
    }
}

pub struct Game<M, E> {
    model: M,
    env: E,
}
impl<M, Cmd, U, E> Game<M, E>
where
    M: for<'m> Model<'m, Update = U, Cmd = Cmd>,
    E: 'static,
{
    #[allow(dead_code)]
    pub fn create<R, Input>(
        self: Box<Self>,
    ) -> (
        Rc<RefCell<InputDblBuffer<Input>>>,
        impl Generator<Yield = (), Return = ()>,
    )
    where
        E: DrawGrid,
        R: IncrRender<Env = E, Patch = U>,
        Input: Into<Option<Cmd>> + Copy + 'static + Eq + ::std::fmt::Debug,
    {
        let this = Box::leak(self);
        let buf = Rc::new(RefCell::new(InputDblBuffer::new()));

        (buf.clone(), move || loop {
            {
                this.env.clear();
                let iter = this.model.initialize();
                for update in iter {
                    let renderer = R::new_patch(update);
                    yield_from!(renderer.to_generator(&mut this.env));
                }
            }

            loop {
                /*
                 * console_log!(
                 *     "Tick Start Current: {:?}",
                 *     buf.borrow_mut().current()
                 * );
                 * console_log!(
                 *     "Tick Start Next   : {:?}",
                 *     buf.borrow_mut().next()
                 * );
                 */

                let cmd = buf.borrow_mut().read().and_then(|c| c.into());
                let update = this.model.step(cmd);

                match update {
                    Ok(Some(u)) => {
                        let renderer = R::new_patch(u);
                        yield_from!(renderer.to_generator(&mut this.env));
                    }
                    Ok(None) => yield (),
                    Err(_) => break,
                }

                // console_log!("Current: {:?}", buf.borrow_mut().current());
                // console_log!("Next   : {:?}", buf.borrow_mut().next());
                // buf.borrow_mut()
                // .swap_if(|_c, n| n.iter().any(|&t| t.into().is_some()));
                buf.borrow_mut().swap_if(|curr, next| {
                    let a =
                        curr.iter().filter(|t| (**t).into().is_some()).count();
                    let b =
                        next.iter().filter(|t| (**t).into().is_some()).count();

                    a < b
                });
            }

            buf.borrow_mut().clear_both();
            this.model.tear_down();
        })
    }
}
