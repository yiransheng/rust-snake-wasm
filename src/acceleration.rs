use data::Key;
use system::{GameSystem, Never, RenderQueue};
use world::WorldUpdate;

pub struct AccMiddleware {
    frames_accmulated: u8,
}

impl GameSystem for AccMiddleware {
    type InputCmd = Key;
    type Msg = WorldUpdate;
    type GameOver = Never;

    fn start_up(&mut self, _q: &mut RenderQueue<Self::Msg>) {}

    fn tick(
        &mut self,
        cmd: Self::InputCmd,
        q: &mut RenderQueue<Self::Msg>,
    ) -> Result<(), Self::GameOver> {
        self.set_key(cmd);

        self.speed_up(q);

        Ok(())
    }

    fn tear_down(&mut self) {
        self.frames_accmulated = 0;
    }
}

impl AccMiddleware {
    const MAX_ACCUMULATED: u8 = 64;

    pub fn new() -> Self {
        AccMiddleware {
            frames_accmulated: 0,
        }
    }
    #[inline]
    fn set_key(&mut self, k: Key) {
        if k.is_direction_key() && self.frames_accmulated < Self::MAX_ACCUMULATED {
            self.frames_accmulated += 1;
        } else if self.frames_accmulated > 2 {
            self.frames_accmulated -= 2;
        }
    }
    #[inline]
    fn speed_up<P>(&mut self, q: &mut RenderQueue<P>) {
        let frames_to_reduce = self.frames_accmulated / 16;

        if frames_to_reduce > 0 {
            for ru in q.iter_mut() {
                ru.modify_duration(|d| {
                    if d > frames_to_reduce {
                        d - frames_to_reduce
                    } else {
                        d
                    }
                });
            }
        }
    }
}
