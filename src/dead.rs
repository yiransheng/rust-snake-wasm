use std::marker::PhantomData;

use data::Key;
use system::{GameOver, Stateful};
use world::WorldUpdate;

pub enum CtrlEvent {
    StartGame,
    QuitGame,
}

impl Into<Option<CtrlEvent>> for Key {
    fn into(self) -> Option<CtrlEvent> {
        if self != Key::none() {
            Some(CtrlEvent::StartGame)
        } else {
            None
        }
    }
}

pub struct Dead<U> {
    _update_type: PhantomData<U>,
}
impl<U> Dead<U> {
    pub fn new() -> Self {
        Dead {
            _update_type: PhantomData,
        }
    }
}

impl<'m, U> Stateful<'m> for Dead<U>
where
    U: 'static + From<WorldUpdate>,
{
    type Cmd = CtrlEvent;
    type Init = Option<U>;
    type Update = U;
    type Error = GameOver;

    fn initialize(&'m mut self) -> Self::Init {
        Some(U::from(WorldUpdate::Dead))
    }

    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error> {
        match cmd {
            Some(CtrlEvent::StartGame) => Err(GameOver::Over),
            Some(CtrlEvent::QuitGame) => Err(GameOver::Quit),
            _ => Ok(None),
        }
    }

    fn tear_down(&mut self) {}
}
