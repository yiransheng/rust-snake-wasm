use std::marker::PhantomData;

use constants::{ANIMATION_FRAME_COUNT, TILE_SIZE};
use data::{Block, Direction, SmallNat};
use system::{Color, DrawGrid, IncrRender, UnitInterval};
use world::WorldUpdate;

pub struct WorldUpdateDraw<U: Into<WorldUpdate> = WorldUpdate> {
    update: WorldUpdate,
    current_frame: u8,
    total_frame: u8,
    _update_type: PhantomData<U>,
}

impl<E: DrawGrid> IncrRender<E> for WorldUpdateDraw<WorldUpdate> {
    type Patch = WorldUpdate;

    fn new_patch(u: WorldUpdate) -> Self {
        WorldUpdateDraw::new(u, ANIMATION_FRAME_COUNT)
    }
    #[inline]
    fn render(&mut self, env: &mut E) -> Option<()> {
        self.render(env)
    }
}

impl<U> WorldUpdateDraw<U>
where
    U: Into<WorldUpdate>,
{
    pub fn new(u: U, total_frame: u8) -> Self {
        WorldUpdateDraw {
            update: u.into(),
            current_frame: 0,
            total_frame,
            _update_type: PhantomData,
        }
    }

    #[inline]
    pub fn render<E: DrawGrid>(&mut self, env: &mut E) -> Option<()> {
        let next_frame = self.draw_frame(env);

        if next_frame >= self.total_frame {
            self.current_frame = self.total_frame;
            None
        } else {
            self.current_frame = next_frame;
            Some(())
        }
    }
    fn draw_frame<E: DrawGrid>(&self, env: &mut E) -> u8 {
        let t = UnitInterval::from_u8_and_range(
            self.current_frame,
            0..self.total_frame,
        );

        match self.update {
            WorldUpdate::SetWorldSize(w, h) => {
                env.clear();
                env.setup(TILE_SIZE as SmallNat, w, h);
                self.total_frame
            }
            WorldUpdate::Clear { prev_block, at } => {
                match prev_block {
                    Block::Snake(dir) => env.clear_tile(at.x, at.y, dir, t),
                    _ => env.clear_tile(
                        at.x,
                        at.y,
                        Direction::East,
                        UnitInterval::max_value(),
                    ),
                }
                self.current_frame + 1
            }
            WorldUpdate::SetBlock { block, at } => {
                match block {
                    Block::Food => env.with_fill_color(Color::Red, |env| {
                        env.circle(at.x, at.y, t);
                    }),
                    Block::Snake(dir) => env.fill_tile(at.x, at.y, dir, t),
                    _ => {}
                }
                self.current_frame + 1
            }
            WorldUpdate::Dead => {
                env.show_game_over();
                self.total_frame
            }
        }
    }
}
