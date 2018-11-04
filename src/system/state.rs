use alloc::boxed::Box;
use alloc::rc::Rc;

use std::cell::RefCell;
use std::iter::{IntoIterator, Map, Zip};
use std::marker::PhantomData;

use std::ops::Generator;

use void::Void;

use super::input_buffer::InputDblBuffer;
use super::render::{DrawGrid, IncrRender};

pub enum GameOver {
    Over,
    Quit,
}

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

    fn make_game<E>(self, env: E) -> Game<Self, E>
    where
        Self: Sized,
    {
        Game { model: self, env }
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
        let err: GameOver;

        match self.current() {
            Either::Left(a) => match a.step(cmd.and_then(|c| c.into())) {
                Ok(u) => return Ok(u),
                Err(e) => {
                    err = e.into();
                }
            },
            Either::Right(b) => match b.step(cmd.and_then(|c| c.into())) {
                Ok(u) => return Ok(u),
                Err(e) => {
                    err = e.into();
                }
            },
        }

        self.swap();

        Err(err)
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
        self,
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
        let mut model = Box::new(self.model);
        let env = Rc::new(RefCell::new(self.env));

        let buf = Rc::new(RefCell::new(InputDblBuffer::new()));

        (buf.clone(), move || 'app: loop {
            {
                let iter = model.initialize();
                for update in iter {
                    let renderer = R::new_patch(update);
                    yield_from!(renderer.to_generator(&env));
                }
            }

            'game: loop {
                let cmd = buf.borrow_mut().read();
                let update = model.step(cmd);

                match update {
                    Ok(Some(u)) => {
                        let renderer = R::new_patch(u);
                        yield_from!(renderer.to_generator(&env));
                    }
                    Ok(None) => yield (),
                    Err(err) => match err.into() {
                        GameOver::Over => break 'game,
                        GameOver::Quit => break 'app,
                    },
                }

                buf.borrow_mut()
                    .swap_when(|curr, next| curr.len() < next.len());
            }

            buf.borrow_mut().clear_both();
            model.tear_down();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::render::*;
    use super::*;
    use data::*;

    #[test]
    fn test_drop() {
        struct Empty<'a> {
            dropped: &'a mut bool,
        }
        impl<'a> Drop for Empty<'a> {
            fn drop(&mut self) {
                *self.dropped = true;
            }
        }
        impl<'a> Empty<'a> {
            fn new(dropped: &'a mut bool) -> Self {
                Empty { dropped }
            }
        }
        impl<'a, 'm> Stateful<'m> for Empty<'a> {
            type Cmd = ();
            type Update = ();
            type Init = Option<()>;
            type Error = GameOver;

            fn initialize(&'m mut self) -> Self::Init {
                None
            }

            fn step(
                &mut self,
                _cmd: Option<Self::Cmd>,
            ) -> Result<Option<Self::Update>, Self::Error> {
                Ok(Some(()))
            }

            fn tear_down(&mut self) {}
        }
        impl<'a> DrawGrid for Empty<'a> {
            fn setup(
                &mut self,
                _tile_size: SmallNat,
                _width: SmallNat,
                _height: SmallNat,
            ) {
            }

            fn clear(&mut self) {}

            // returns current fill color
            fn set_fill_color(&mut self, color: Color) -> Color {
                color
            }

            fn circle(
                &mut self,
                _x: SmallNat,
                _y: SmallNat,
                _radius: UnitInterval,
            ) {
            }

            fn fill_tile(
                &mut self,
                _x: SmallNat,
                _y: SmallNat,
                _dir: Direction,
                _size: UnitInterval,
            ) {
            }

            fn clear_tile(
                &mut self,
                _x: SmallNat,
                _y: SmallNat,
                _dir: Direction,
                _size: UnitInterval,
            ) {
            }

            fn show_game_over(&mut self) {}
        }
        struct RenderNothing<'a> {
            _lifetime: PhantomData<&'a ()>,
        }

        impl<'a> IncrRender for RenderNothing<'a> {
            type Patch = ();
            type Env = Empty<'a>;

            fn new_patch(_u: Self::Patch) -> Self {
                RenderNothing {
                    _lifetime: PhantomData,
                }
            }
            fn render(&mut self, _env: &mut Self::Env) -> Option<()> {
                Some(())
            }
        }
        // -------- test below ----------

        let mut model_dropped = false;
        let mut env_dropped = false;

        {
            let game = Empty::new(&mut model_dropped)
                .make_game(Empty::new(&mut env_dropped));
            let (_buf, mut gen) = game.new_game::<RenderNothing, ()>();

            unsafe {
                gen.resume();
            }
        }

        assert!(model_dropped);
        assert!(env_dropped);
    }

}
