use alloc::boxed::Box;
use alloc::rc::Rc;

use std::cell::RefCell;
use std::iter::{IntoIterator, Map, Zip};
use std::marker::PhantomData;

use std::ops::Generator;

use void::Void;

use super::input_buffer::InputDblBuffer;
use super::render::{DrawGrid, IncrRender};

pub struct GameOver;

impl Into<GameOver> for Void {
    fn into(self) -> GameOver {
        unreachable!()
    }
}

pub trait Stateful<'m> {
    type Cmd;
    type Init: IntoIterator<Item = Self::Update> + 'm;
    type Update;
    type Error: Into<GameOver>;

    fn initialize(&'m mut self) -> Self::Init;

    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error>;

    fn tear_down(&mut self);

    fn make_game<E>(self, env: E) -> Box<Game<Self, E>>
    where
        Self: Sized,
        E: 'static,
    {
        Box::new(Game { model: self, env })
    }

    fn zip_with<T, R, F>(self, other: R, f: F) -> ZipWith<Self, R, F>
    where
        R: Stateful<'m, Cmd = Self::Cmd, Error = Void>,
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

    fn alternating_after<C, B>(self, other: B) -> Alternating<Self, B, C>
    where
        Self: Sized,
        B: Stateful<'m, Update = Self::Update>,
        C: Into<Option<Self::Cmd>> + Into<Option<B::Cmd>>,
    {
        Alternating {
            swapped: true,
            left: self,
            right: other,
            _cmd: PhantomData,
        }
    }

    fn alternating<C, B>(self, other: B) -> Alternating<Self, B, C>
    where
        Self: Sized,
        B: Stateful<'m, Update = Self::Update>,
        C: Into<Option<Self::Cmd>> + Into<Option<B::Cmd>>,
    {
        Alternating {
            swapped: false,
            left: self,
            right: other,
            _cmd: PhantomData,
        }
    }
}

pub struct ZipWith<L, R, F> {
    left: L,
    right: R,
    f: F,
}

impl<'m, L, R, T, F> Stateful<'m> for ZipWith<L, R, F>
where
    L: Stateful<'m>,
    R: Stateful<'m, Cmd = L::Cmd, Error = Void>,
    F: Fn((L::Update, R::Update)) -> T + 'm,
    L::Cmd: Copy,
{
    type Cmd = L::Cmd;
    type Update = T;
    type Init = Map<
        Zip<
            <L::Init as IntoIterator>::IntoIter,
            <R::Init as IntoIterator>::IntoIter,
        >,
        &'m F,
    >;
    type Error = L::Error;

    fn initialize(&'m mut self) -> Self::Init {
        self.left
            .initialize()
            .into_iter()
            .zip(self.right.initialize().into_iter())
            .map(&self.f)
    }

    #[inline]
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

#[derive(Copy, Clone)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
impl<A, B> Iterator for Either<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(ref mut a) => a.next(),
            Either::Right(ref mut b) => b.next(),
        }
    }
}

pub struct Alternating<A, B, C> {
    swapped: bool,
    left: A,
    right: B,
    _cmd: PhantomData<C>,
}
impl<A, B, C> Alternating<A, B, C> {
    #[inline]
    fn current(&mut self) -> Either<&mut A, &mut B> {
        if self.swapped {
            Either::Right(&mut self.right)
        } else {
            Either::Left(&mut self.left)
        }
    }

    fn swap(&mut self) {
        self.swapped = !self.swapped;
    }
}

impl<'m, Cmd, A, B> Stateful<'m> for Alternating<A, B, Cmd>
where
    A: Stateful<'m>,
    B: Stateful<'m, Update = A::Update>,
    Cmd: Into<Option<A::Cmd>> + Into<Option<B::Cmd>>,
{
    type Cmd = Cmd;
    type Init = Either<
        <A::Init as IntoIterator>::IntoIter,
        <B::Init as IntoIterator>::IntoIter,
    >;
    type Update = A::Update;
    type Error = GameOver;

    fn initialize(&'m mut self) -> Self::Init {
        match self.current() {
            Either::Left(a) => Either::Left(a.initialize().into_iter()),
            Either::Right(b) => Either::Right(b.initialize().into_iter()),
        }
    }

    #[inline]
    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error> {
        match self.current() {
            Either::Left(a) => match a.step(cmd.and_then(|c| c.into())) {
                Ok(u) => return Ok(u),
                _ => {}
            },
            Either::Right(b) => match b.step(cmd.and_then(|c| c.into())) {
                Ok(u) => return Ok(u),
                _ => {}
            },
        }

        self.swap();

        Err(GameOver)
    }

    fn tear_down(&mut self) {
        match self.current() {
            Either::Left(a) => a.tear_down(),
            Either::Right(b) => b.tear_down(),
        }
    }
}

pub struct Game<M, E> {
    model: M,
    env: E,
}
impl<M, Cmd, U, E> Game<M, E>
where
    M: for<'m> Stateful<'m, Update = U, Cmd = Cmd>,
{
    pub fn new_game<R, Input>(
        self: Box<Self>,
    ) -> (
        Rc<RefCell<InputDblBuffer<Cmd>>>,
        impl Generator<Yield = (), Return = ()>,
    )
    where
        E: DrawGrid,
        R: IncrRender<Env = E, Patch = U>,
        Input: Into<Option<Cmd>> + Copy + 'static,
        Cmd: Eq,
    {
        let this = Box::leak(self);
        let buf = Rc::new(RefCell::new(InputDblBuffer::new()));

        (buf.clone(), move || loop {
            {
                let iter = this.model.initialize();
                for update in iter {
                    let renderer = R::new_patch(update);
                    yield_from!(renderer.to_generator(&mut this.env));
                }
            }

            loop {
                // render loop
                let cmd = buf.borrow_mut().read();
                let update = this.model.step(cmd);

                match update {
                    Ok(Some(u)) => {
                        let renderer = R::new_patch(u);
                        yield_from!(renderer.to_generator(&mut this.env));
                    }
                    Ok(None) => yield (),
                    Err(_) => break,
                }

                buf.borrow_mut()
                    .swap_when(|curr, next| curr.len() < next.len());
            }

            buf.borrow_mut().clear_both();
            this.model.tear_down();
        })
    }
}
