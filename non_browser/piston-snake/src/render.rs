use piston_window::types::Color;
use piston_window::Rectangle;

use snake_wasm::data::{Coordinate, Direction, SmallNat};
use snake_wasm::{
    partial_tile, Color as GameColor, DrawGrid, Either, UnitInterval,
};

use crate::channel::Sender;
use crate::constants::*;

#[derive(Copy, Clone)]
pub struct TilePatch {
    pub rect: Rectangle,
    pub hash: usize,
    pub region: [f64; 4],
}
#[derive(Copy, Clone)]
pub struct Clear {
    pub background_color: Color,
}

pub struct TileUpdate {
    tx: Sender<Either<TilePatch, Clear>>,
    tile_size: f64,
    game_color: GameColor,
    background_color: Color,
}
impl TileUpdate {
    pub fn new(tx: Sender<Either<TilePatch, Clear>>) -> Self {
        TileUpdate {
            tx,
            tile_size: TILE_SIZE as f64,
            game_color: GameColor::Black,
            background_color: BACKGROUND_COLOR,
        }
    }
}

impl DrawGrid for TileUpdate {
    fn setup(
        &mut self,
        _tile_size: SmallNat,
        _width: SmallNat,
        _height: SmallNat,
    ) {
        self.clear();
    }

    fn clear(&mut self) {
        let patch = Either::Right(Clear {
            background_color: self.background_color,
        });

        let _ = self.tx.send(patch);
    }

    fn set_fill_color(&mut self, color: GameColor) -> GameColor {
        let prev_color = self.game_color;
        self.game_color = color;

        prev_color
    }

    fn circle(&mut self, x: SmallNat, y: SmallNat, radius: UnitInterval) {
        let tile_size = self.tile_size;
        let hash = Coordinate { x, y }.encode_usize();

        let x = (x as f64) * tile_size + tile_size / 2.0;
        let y = (y as f64) * tile_size + tile_size / 2.0;

        let r = radius.scale(tile_size) * 0.5;
        let d = r * 2.0;
        let color = self.color();

        let rect = Rectangle::new_round(color, r);
        let region = [x - r, y - r, d, d];

        let patch = Either::Left(TilePatch { rect, region, hash });

        let _ = self.tx.send(patch);
    }

    fn fill_tile(
        &mut self,
        x: SmallNat,
        y: SmallNat,
        dir: Direction,
        size: UnitInterval,
    ) {
        let hash = Coordinate { x, y }.encode_usize();

        let (x, y, w, h) = partial_tile(self.tile_size, x, y, dir, size);
        let color = self.color();

        let patch = Either::Left(TilePatch {
            rect: Rectangle::new(color),
            hash,
            region: [x, y, w, h],
        });

        let _ = self.tx.send(patch);
    }

    fn clear_tile(
        &mut self,
        x: SmallNat,
        y: SmallNat,
        dir: Direction,
        size: UnitInterval,
    ) {
        let hash = Coordinate { x, y }.encode_usize();
        let color = self.color();

        let (x, y, w, h) = partial_tile(
            self.tile_size,
            x,
            y,
            dir.opposite(),
            size.complement(),
        );

        let patch = Either::Left(TilePatch {
            rect: Rectangle::new(color),
            hash,
            region: [x, y, w, h],
        });

        let _ = self.tx.send(patch);
    }

    fn show_game_over(&mut self) {}
}

impl TileUpdate {
    #[inline]
    fn color(&self) -> Color {
        match self.game_color {
            GameColor::Black => [0.13, 0.13, 0.13, 1.0],
            GameColor::Red => [0.95, 0.04, 0.04, 1.0],
        }
    }
}
