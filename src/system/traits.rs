use super::{RenderQueue, RenderUnit};
pub use either::Either;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn _assert_is_object_safe(_: &dyn GameSystem<Msg = (), InputCmd = (), GameOver = ()>) {}

pub trait RenderSink<T> {
    fn is_ready(&self) -> bool;

    fn push(&mut self, unit: RenderUnit<T>);
}

pub trait DrawTile {
    const TILE_SIZE: f64;

    // normalized_progress: [0, 1]
    fn draw_tile(&self, gc: &CanvasRenderingContext2d, x: f64, y: f64, normalized_progress: f64);

    fn prepare_canvas(&self, canvas: &HtmlCanvasElement);

    fn instant(self) -> InstantDraw<Self>
    where
        Self: Sized,
    {
        InstantDraw { inner: self }
    }
}

pub struct InstantDraw<D> {
    inner: D,
}
impl<D: DrawTile> DrawTile for InstantDraw<D> {
    const TILE_SIZE: f64 = D::TILE_SIZE;

    fn prepare_canvas(&self, canvas: &HtmlCanvasElement) {
        self.inner.prepare_canvas(canvas);
    }

    #[inline]
    fn draw_tile(&self, gc: &CanvasRenderingContext2d, x: f64, y: f64, _normalized_progress: f64) {
        self.inner.draw_tile(gc, x, y, 1.0);
    }
}

pub trait GameSystem {
    type Msg;
    type InputCmd;
    type GameOver;

    fn start_up(&mut self, &mut RenderQueue<Self::Msg>);

    fn tick(
        &mut self,
        cmd: Self::InputCmd,
        &mut RenderQueue<Self::Msg>,
    ) -> Result<(), Self::GameOver>;

    fn tear_down(&mut self);

    fn with_renderer<R>(self, r: R) -> WithRenderer<Self, R>
    where
        Self: Sized,
        R: GameSystem<Msg = Self::Msg, InputCmd = (), GameOver = Never>,
    {
        WithRenderer {
            system: self,
            renderer: r,
        }
    }

    fn with_play_state(self) -> WithPlayState<Self>
    where
        Self: Sized,
    {
        WithPlayState {
            state: PlayState::NotRunning,
            system: self,
        }
    }
}

pub enum Never {}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum PlayState {
    NotRunning,
    Running,
    Over,
}

pub struct StartGame;

impl<T> Into<GameInput<T>> for StartGame {
    fn into(self) -> GameInput<T> {
        Either::Left(self)
    }
}

pub type GameInput<T> = Either<StartGame, T>;

pub struct WithPlayState<S> {
    state: PlayState,
    system: S,
}

impl<M, I, S> GameSystem for WithPlayState<S>
where
    S: GameSystem<Msg = M, InputCmd = I, GameOver = ()>,
{
    type Msg = M;
    type InputCmd = GameInput<I>;
    type GameOver = Never;

    fn start_up(&mut self, q: &mut RenderQueue<Self::Msg>) {
        match self.state {
            PlayState::Running => {}
            _ => {
                self.system.start_up(q);
                self.state = PlayState::Running;
            }
        }
    }

    fn tick(
        &mut self,
        cmd: Self::InputCmd,
        q: &mut RenderQueue<Self::Msg>,
    ) -> Result<(), Self::GameOver> {
        match self.state {
            PlayState::Running => match cmd {
                Either::Right(cmd) => {
                    if let Err(_) = self.system.tick(cmd, q) {
                        self.tear_down();
                    }
                    Ok(())
                }
                Either::Left(StartGame) => Ok(()),
            },
            _ => match cmd {
                Either::Left(StartGame) => {
                    self.start_up(q);
                    Ok(())
                }
                Either::Right(_) => Ok(()),
            },
        }
    }

    fn tear_down(&mut self) {
        match self.state {
            PlayState::Over => {}
            _ => {
                self.state = PlayState::Over;
                self.system.tear_down();
            }
        }
    }
}

pub struct WithRenderer<S, R> {
    system: S,
    renderer: R,
}

impl<M, I, E, S, R> GameSystem for WithRenderer<S, R>
where
    S: GameSystem<Msg = M, InputCmd = I, GameOver = E>,
    R: GameSystem<Msg = M, InputCmd = (), GameOver = Never>,
{
    type Msg = M;
    type InputCmd = I;
    type GameOver = ();

    fn start_up(&mut self, q: &mut RenderQueue<Self::Msg>) {
        self.system.start_up(q);
        self.renderer.start_up(q);
    }

    fn tick(
        &mut self,
        cmd: Self::InputCmd,
        q: &mut RenderQueue<Self::Msg>,
    ) -> Result<(), Self::GameOver> {
        self.system.tick(cmd, q).map_err(|_| ())?;

        match self.renderer.tick((), q) {
            Ok(_) => Ok(()),
            _ => unreachable!(),
        }
    }

    fn tear_down(&mut self) {
        self.renderer.tear_down();
        self.system.tear_down();
    }
}
