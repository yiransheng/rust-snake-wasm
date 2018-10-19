use data::{Coordinate, Direction, Tile};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RenderUpdate {
    Appear(Coordinate, Direction),
    Vanish(Coordinate, Direction),
    SpawnFood,
}

pub trait Render {}
