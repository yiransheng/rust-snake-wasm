use rand::Rng;
use std::cell::Cell;
use std::rc::Rc;

use data::Direction;
use patch::Patch;
use world::{World, WorldUpdateEff};

pub trait GameState {
    type InputCmd;
    type UpdateEff;

    fn mount(&mut self) -> Self::UpdateEff;

    fn handle_input(&mut self, cmd: Self::InputCmd);

    fn update(&mut self) -> Result<Self::UpdateEff, ()>;
}

impl<R: Rng> GameState for World<R> {
    type InputCmd = Direction;
    type UpdateEff = Patch<WorldUpdateEff>;

    fn mount(&mut self) -> Self::UpdateEff {
        self.init_update()
    }
    fn handle_input(&mut self, cmd: Direction) {
        self.set_direction(cmd);
    }
    fn update(&mut self) -> Result<Self::UpdateEff, ()> {
        self.tick_update().map_err(|_| ())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameStatus {
    Idle,
    Ready,
    Running,
    Rendering,
    GameOver,
}

impl ::std::fmt::Display for GameStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            GameStatus::Idle => write!(f, "Idle"),
            GameStatus::Ready => write!(f, "Ready"),
            GameStatus::Running => write!(f, "Running"),
            GameStatus::Rendering => write!(f, "Rendering"),
            GameStatus::GameOver => write!(f, "GameOver"),
        }
    }
}

impl GameStatus {
    fn game_started(self) -> Self {
        match self {
            GameStatus::Idle => GameStatus::Ready,
            _ => self,
        }
    }
    fn render_started(self) -> Self {
        match self {
            GameStatus::Ready => GameStatus::Rendering,
            GameStatus::Running => GameStatus::Rendering,
            _ => self,
        }
    }
    fn render_done(self) -> Self {
        match self {
            GameStatus::Rendering => GameStatus::Running,
            _ => self,
        }
    }
    fn game_over(self) -> Self {
        GameStatus::GameOver
    }
}

pub trait Renderer {
    type UpdateEff;

    fn render<F>(&mut self, mut eff: Self::UpdateEff, callback: F)
    where
        F: Fn();
}

pub struct RunGame<G, R> {
    status: Rc<Cell<GameStatus>>,
    renderer: R,
    game: G,
}

impl<G: GameState, R: Renderer<UpdateEff = G::UpdateEff>> RunGame<G, R> {
    pub fn new(state: G, renderer: R) -> Self {
        RunGame {
            status: Rc::new(Cell::new(GameStatus::Idle)),
            game: state,
            renderer,
        }
    }

    pub fn start(&mut self) {
        self.status.set(GameStatus::Ready)
    }

    pub fn on_enter_frame(&mut self) {
        match self.status.get() {
            s @ GameStatus::Ready => {
                let eff = self.game.mount();
                self.queue_render(eff);
            }
            s @ GameStatus::Running => match self.game.update() {
                Ok(eff) => {
                    self.queue_render(eff);
                }
                Err(_) => {
                    web_sys::console::log_1(&"Is Over".into());
                    self.status.set(GameStatus::GameOver);
                }
            },
            _ => {}
        };
    }

    fn queue_render(&mut self, eff: G::UpdateEff) {
        self.status.set(GameStatus::Rendering);

        let status = self.status.clone();

        self.renderer.render(eff, || {
            let current = status.get();
            let next_status = current.render_done();
            status.set(next_status);
        });
    }
}
