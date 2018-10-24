use std::borrow::Borrow;
use std::convert::AsRef;
use std::iter::IntoIterator;
use std::marker::PhantomData;
use std::slice::Iter;

use std::ops::{Generator, GeneratorState};

use void::Void;

pub struct GameOver;

pub trait Model<'m> {
    type Cmd;
    type State: IntoIterator<Item = Self::Update> + 'm;
    type Update;
    type Error: Into<GameOver>;

    fn initialize(&'m mut self) -> Self::State;

    fn step(&mut self, cmd: Option<Self::Cmd>) -> Result<Self::Update, Self::Error>;
}

pub trait Render {
    type Env;
    type Model: for<'m> Model<'m>;

    fn new(u: <Self::Model as Model>::Update) -> Self;

    fn render(&mut self, env: &mut Self::Env) -> Option<()>;

    #[inline]
    fn into_generator(self, env: &mut Self::Env) -> RenderGen<Self::Env, Self>
    where
        Self: Sized,
    {
        RenderGen {
            env,
            renderer: self,
        }
    }
}

pub struct RenderGen<'a, E, R> {
    env: &'a mut E,
    renderer: R,
}
impl<'a, E, R> Generator for RenderGen<'a, E, R>
where
    R: Render<Env = E>,
{
    type Yield = ();
    type Return = ();

    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        match self.renderer.render(self.env) {
            Some(_) => GeneratorState::Yielded(()),
            _ => GeneratorState::Complete(()),
        }
    }
}

/*
 * struct Game<M> {
 *     model: M,
 * }
 * impl<M: for<'a> Model<'a>> Game<M> {
 *     #[allow(dead_code)]
 *     fn run<'a, R, E>(&'a mut self, e: E) -> impl Generator<Yield = (), Return = ()> + 'a
 *     where
 *         R: Render<Env = E, Model = M>,
 *         E: Clone + 'a,
 *     {
 *         let e = e.clone();
 *         move || {
 *             {
 *                 let iter: <M::State as IntoIterator>::IntoIter =
 *                     self.model.initialize().into_iter();
 *                 for u in iter {
 *                     yield_from!(R::new(u).into_generator(e.clone()));
 *                 }
 *             }
 *
 *             loop {
 *                 let u = self.model.step(None);
 *
 *                 if let Ok(u) = u {
 *                     yield_from!(R::new(u).into_generator(e.clone()));
 *                 } else {
 *                     return ();
 *                 }
 *             }
 *         }
 *     }
 * }
 */
