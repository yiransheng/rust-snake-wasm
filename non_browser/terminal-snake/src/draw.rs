use std::io::StdoutLock;
use std::marker::PhantomData;

use termion::raw::RawTerminal;

use snake_wasm::{IncrRender, WorldUpdate, WorldUpdateDraw};

use crate::term_env::TermEnv;

pub struct DrawUpdate<'a> {
    draw: WorldUpdateDraw<WorldUpdate>,
    _lifetime: PhantomData<&'a ()>,
}
impl<'a> IncrRender for DrawUpdate<'a> {
    type Env = TermEnv<RawTerminal<StdoutLock<'a>>>;
    type Patch = WorldUpdate;

    fn new_patch(u: Self::Patch) -> Self {
        DrawUpdate {
            draw: WorldUpdateDraw::new(u, 4),
            _lifetime: PhantomData,
        }
    }

    #[inline(always)]
    fn render(
        &mut self,
        env: &mut TermEnv<RawTerminal<StdoutLock<'a>>>,
    ) -> Option<()> {
        self.draw.render(env)
    }
}
