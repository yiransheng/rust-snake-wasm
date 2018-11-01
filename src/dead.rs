use std::marker::PhantomData;

use data::Key;
use system::{GameOver, Model};
use world::WorldUpdate;

pub struct StartGame;

impl Into<Option<StartGame>> for Key {
    fn into(self) -> Option<StartGame> {
        if self != Key::none() {
            Some(StartGame)
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

impl<'m, U> Model<'m> for Dead<U>
where
    U: 'static + From<WorldUpdate>,
{
    type Cmd = StartGame;
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
            Some(StartGame) => Err(GameOver),
            _ => Ok(None),
        }
    }

    fn tear_down(&mut self) {}
}
