use data::{Direction, Key};
use system::{GameSystem, Never, RenderQueue, RenderUnit};
use world::WorldUpdate;

pub struct AccMiddleware {
    key: Option<Key>,
}

impl GameSystem for AccMiddleware {
    type InputCmd = Key;
    type Msg = WorldUpdate;
    type GameOver = Never;

    fn start_up(&mut self, q: &mut RenderQueue<Self::Msg>) {}

    fn tick(
        &mut self,
        cmd: Self::InputCmd,
        q: &mut RenderQueue<Self::Msg>,
    ) -> Result<(), Self::GameOver> {
        self.key = Some(cmd);

        self.speed_up(q);

        Ok(())
    }

    fn tear_down(&mut self) {
        self.key = None;
    }
}

impl AccMiddleware {
    pub fn new() -> Self {
        AccMiddleware { key: None }
    }
    #[inline]
    fn set_key(&mut self, k: Option<Key>) {
        self.key = k;
    }
    #[inline]
    fn speed_up<P>(&self, q: &mut RenderQueue<P>) {
        if let Some(key) = self.key {
            let dir: Option<Direction> = key.into();

            if dir.is_some() {
                for ru in q.iter_mut() {
                    ru.modify_duration(|d| key.modify_animation_frame(d))
                }
            }
        }
    }
}
