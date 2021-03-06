use piston_window::Key;
use snake_wasm::data::Direction;
use snake_wasm::CtrlEvent;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct KeyWrapper {
    key: Key,
}
impl Into<Option<Direction>> for KeyWrapper {
    fn into(self) -> Option<Direction> {
        match self.key {
            Key::Up => Some(Direction::North),
            Key::Down => Some(Direction::South),
            Key::Left => Some(Direction::West),
            Key::Right => Some(Direction::East),
            _ => None,
        }
    }
}
impl Into<Option<CtrlEvent>> for KeyWrapper {
    fn into(self) -> Option<CtrlEvent> {
        Some(CtrlEvent::StartGame)
    }
}

impl From<Key> for KeyWrapper {
    fn from(key: Key) -> Self {
        KeyWrapper { key }
    }
}
