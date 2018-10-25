use std::f64::consts::PI;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use constants::{ANIMATION_FRAME_COUNT, TILE_SIZE};
use data::Direction;
use system::DrawTile;
use world::WorldUpdate;

impl DrawTile for WorldUpdate {
    fn setup_canvas(&self, canvas: &HtmlCanvasElement) {
        match self {
            WorldUpdate::SetWorldSize(w, h) => {
                let size = TILE_SIZE as u32;
                canvas.set_width(*w * size);
                canvas.set_height(*h * size);
            }
            _ => {}
        }
    }

    fn draw_tile(&self, gc: &CanvasRenderingContext2d, normalized_progress: f64) {
        match self {
            WorldUpdate::SetWorldSize(_, _) => {}
            WorldUpdate::SetBlock { block, at } => {
                let x = f64::from(at.x) * TILE_SIZE;
                let y = f64::from(at.y) * TILE_SIZE;

                if block.is_snake() {
                    let ts = TILE_SIZE;
                    let dir = Direction::from(*block);
                    let length = normalized_progress * ts;
                    match dir {
                        Direction::North => gc.fill_rect(x, y + ts - length, ts, length),
                        Direction::South => gc.fill_rect(x, y, ts, length),
                        Direction::East => gc.fill_rect(x, y, length, ts),
                        Direction::West => gc.fill_rect(x + ts - length, y, length, ts),
                    }
                } else if block.is_food() {
                    let r_full = TILE_SIZE / 2.0;
                    let r = r_full * normalized_progress;
                    gc.save();

                    gc.set_fill_style(&"rgba(255, 0, 0, 1)".into());
                    gc.begin_path();
                    let _ = gc.arc(x + r_full, y + r_full, r, 0.0, 2.0 * PI);
                    gc.fill();

                    gc.restore();
                }
            }
            WorldUpdate::Clear { prev_block, at } => {
                let x = f64::from(at.x) * TILE_SIZE;
                let y = f64::from(at.y) * TILE_SIZE;

                let ts = TILE_SIZE;

                if prev_block.is_snake() {
                    let dir = Direction::from(*prev_block);
                    let length = normalized_progress * ts;
                    match dir {
                        Direction::North => gc.clear_rect(x, y + ts - length, ts, length),
                        Direction::South => gc.clear_rect(x, y, ts, length),
                        Direction::East => gc.clear_rect(x, y, length, ts),
                        Direction::West => gc.clear_rect(x + ts - length, y, length, ts),
                    }
                } else if normalized_progress == 1.0 {
                    gc.clear_rect(x, y, ts, ts);
                }
            }
        }
    }
}
