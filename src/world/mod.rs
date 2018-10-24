use rand::{Rng, SeedableRng};
use std::convert::{From, Into};
use std::iter::Map;

use data::{Block, Coordinate, Direction, Tile};
use system::Model;

mod grid;
mod world;

pub use self::world::World;
