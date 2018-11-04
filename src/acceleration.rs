use void::Void;

use canvas::WorldUpdateDraw;
use constants::ANIMATION_FRAME_COUNT;
use data::Direction;
use system::{DrawGrid, IncrRender, Stateful};
use world::WorldUpdate;

pub struct VariableFrame<T = WorldUpdate> {
    pub update: T,
    pub frame_count: u8,
}
impl Into<WorldUpdate> for VariableFrame<WorldUpdate> {
    fn into(self) -> WorldUpdate {
        self.update
    }
}
impl From<WorldUpdate> for VariableFrame<WorldUpdate> {
    fn from(u: WorldUpdate) -> Self {
        VariableFrame {
            update: u,
            frame_count: ANIMATION_FRAME_COUNT,
        }
    }
}
impl<T> VariableFrame<T> {
    pub fn pack(both: (T, u8)) -> Self {
        VariableFrame {
            update: both.0,
            frame_count: both.1,
        }
    }
}

impl<E: DrawGrid> IncrRender<E>
    for WorldUpdateDraw<VariableFrame<WorldUpdate>>
{
    type Patch = VariableFrame<WorldUpdate>;

    fn new_patch(u: Self::Patch) -> Self {
        let frame_count = u.frame_count;
        WorldUpdateDraw::new(u, frame_count)
    }

    #[inline]
    fn render(&mut self, env: &mut E) -> Option<()> {
        self.render(env)
    }
}

const FRAMES: f64 = ANIMATION_FRAME_COUNT as f64;
const TIME_DELTA: f64 = 0.01;
const MIN_VELOCITY: f64 = FRAMES;
const MAX_VELOCITY: f64 = MIN_VELOCITY * FRAMES * 0.6;
const FRICTION: f64 = 13.8629436112 / MIN_VELOCITY;
const POWER: f64 = MAX_VELOCITY * MAX_VELOCITY * FRICTION;

#[derive(Debug, Copy, Clone)]
pub struct RenderSpeed {
    direction: Direction,
    initial_direction: Direction,
    velocity: f64,
}

impl RenderSpeed {
    pub fn new(direction: Direction) -> Self {
        RenderSpeed {
            direction,
            initial_direction: direction,
            velocity: MIN_VELOCITY,
        }
    }
}

impl<'m> Stateful<'m> for RenderSpeed {
    type Cmd = Direction;
    type Init = Forever<u8>;
    type Update = u8;
    type Error = Void;

    fn initialize(&'m mut self) -> Self::Init {
        Forever(ANIMATION_FRAME_COUNT)
    }

    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error> {
        let mut v = self.velocity;
        let acceleration: f64;

        if let Some(dir) = cmd {
            if dir == self.direction {
                acceleration = POWER / v - FRICTION * v;
            } else if dir == self.direction.opposite() {
                acceleration = -POWER / v - FRICTION * v;
            } else {
                self.direction = dir;
                acceleration = 0.0;
            }
        } else {
            acceleration = -FRICTION * v * 0.5;
        }

        v = v + acceleration * TIME_DELTA;

        v = if v > MIN_VELOCITY { v } else { MIN_VELOCITY };

        self.velocity = v;

        Ok(Some(self.derive_frame_count()))
    }
    fn tear_down(&mut self) {
        self.direction = self.initial_direction;
        self.velocity = MIN_VELOCITY;
    }
}

impl RenderSpeed {
    #[inline(always)]
    fn derive_frame_count(&self) -> u8 {
        let v = self.velocity;
        let frame_count = (MIN_VELOCITY / v * FRAMES).ceil();

        frame_count as u8
    }
}

pub struct Forever<T>(T);

impl<T: Copy> Iterator for Forever<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        Some(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceleration_one_step() {
        let mut render_speed = RenderSpeed::new(Direction::East);
        let v0 = render_speed.velocity;

        render_speed
            .step(Some(Direction::East))
            .expect("It never errors");

        assert!(render_speed.velocity > v0);
    }

    #[test]
    fn test_deacceleration_one_step() {
        let mut render_speed = RenderSpeed::new(Direction::East);

        render_speed.step(Some(Direction::East)).unwrap();
        render_speed.step(Some(Direction::East)).unwrap();
        render_speed.step(Some(Direction::East)).unwrap();

        let v1 = render_speed.velocity;

        render_speed.step(None).unwrap();

        assert!(render_speed.velocity < v1);
    }

    #[test]
    fn test_deacceleration_forced() {
        let dir = Direction::North;

        let mut render_speed = RenderSpeed::new(dir);

        render_speed.step(Some(dir)).unwrap();
        render_speed.step(Some(dir)).unwrap();
        render_speed.step(Some(dir)).unwrap();

        let mut render_speed_a = render_speed;
        let mut render_speed_b = render_speed;

        render_speed_a.step(None).unwrap();
        render_speed_b.step(Some(dir.opposite())).unwrap();

        assert!(render_speed_a.velocity > render_speed_b.velocity);
    }

    #[test]
    fn test_acceleration_limiting_behavior() {
        fn final_frame_count_limit(n: usize) -> u8 {
            let render_speed = RenderSpeed::new(Direction::East);

            (0..n)
                .scan(render_speed, |rs, _| {
                    let frame_count =
                        rs.step(Some(Direction::East)).unwrap().unwrap();

                    Some(frame_count)
                })
                .last()
                .unwrap()
        }

        assert_eq!(final_frame_count_limit(100), final_frame_count_limit(500));

        assert!(final_frame_count_limit(20) < ANIMATION_FRAME_COUNT);
        assert!(final_frame_count_limit(20) > 0);
    }

    quickcheck! {
        fn render_speed_produces_bounded_frame_counts(maybe_dirs: Vec<Option<Direction>>) -> bool {
            let mut render_speed = RenderSpeed::new(Direction::East);

            for cmd in maybe_dirs {
                render_speed.step(cmd).unwrap();
            }

            let frame_count = render_speed.derive_frame_count();

            frame_count > 0 && frame_count <= ANIMATION_FRAME_COUNT
        }
    }
}
