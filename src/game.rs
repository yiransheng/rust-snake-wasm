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

#[derive(Debug, Copy, Clone)]
enum GameStatus {
    Idle,
    Ready,
    Running,
    Rendering,
    GameOver,
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

struct RunGameState<G, R> {
    status: Rc<Cell<GameStatus>>,
    renderer: R,
    game: G,
}

pub trait Renderer<G: GameState> {
    fn render<F>(&mut self, eff: G::UpdateEff, callback: F)
    where
        F: FnMut();
}

impl<G: GameState, R: Renderer<G>> RunGameState<G, R> {
    fn on_enter_frame(&mut self) {
        let next_status = match self.status.get() {
            s @ GameStatus::Ready => {
                let eff = self.game.mount();
                self.queue_render(eff);

                s.render_started()
            }
            s @ GameStatus::Running => match self.game.update() {
                Ok(eff) => {
                    self.queue_render(eff);

                    s.render_started()
                }
                Err(_) => GameStatus::GameOver,
            },
            s @ _ => s,
        };

        self.status.set(next_status);
    }

    fn queue_render(&mut self, eff: G::UpdateEff) {
        let status = self.status.clone();

        self.renderer.render(eff, || {
            let next_status = status.get().render_done();
            status.set(next_status);
        });
    }
}
