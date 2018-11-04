use std::io::Read;

use snake_wasm::data::Direction;
use snake_wasm::CtrlEvent;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Key {
    byte: [u8; 1],
}

impl Key {
    pub fn read_from<R: Read>(reader: &mut R) -> Self {
        let mut bytes = [0];
        reader.read(&mut bytes).unwrap();

        Key { byte: bytes }
    }
    pub fn is_quit(self) -> bool {
        match self.byte[0] {
            b'q' => true,
            _ => false,
        }
    }
}

impl Into<Option<CtrlEvent>> for Key {
    fn into(self) -> Option<CtrlEvent> {
        match self.byte[0] {
            b' ' => Some(CtrlEvent::StartGame),
            b'q' => Some(CtrlEvent::QuitGame),
            _ => None,
        }
    }
}

impl Into<Option<Direction>> for Key {
    fn into(self) -> Option<Direction> {
        match self.byte[0] {
            b'k' | b'w' => Some(Direction::North),
            b'j' | b's' => Some(Direction::South),
            b'h' | b'a' => Some(Direction::West),
            b'l' | b'd' => Some(Direction::East),
            _ => None,
        }
    }
}
