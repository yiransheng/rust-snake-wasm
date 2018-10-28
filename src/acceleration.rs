use void::Void;

use constants::ANIMATION_FRAME_COUNT;
use data::Direction;
use renderers::{CanvasEnv, WorldUpdateDraw};
use system::{IncrRender, Model};
use world::WorldUpdate;

pub struct VariableFrame<T = WorldUpdate> {
    update: T,
    frame_count: u8,
}
impl Into<WorldUpdate> for VariableFrame<WorldUpdate> {
    fn into(self) -> WorldUpdate {
        self.update
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

impl IncrRender for WorldUpdateDraw<VariableFrame<WorldUpdate>> {
    type Env = CanvasEnv;
    type Patch = VariableFrame<WorldUpdate>;

    fn new_patch(u: Self::Patch) -> Self {
        let frame_count = u.frame_count;
        WorldUpdateDraw::new(u, frame_count)
    }

    #[inline]
    fn render(&mut self, env: &mut CanvasEnv) -> Option<()> {
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
    velocity: f64,
}

impl RenderSpeed {
    pub fn new(direction: Direction) -> Self {
        RenderSpeed {
            direction,
            velocity: ANIMATION_FRAME_COUNT as f64,
        }
    }
}

impl<'m> Model<'m> for RenderSpeed {
    type Cmd = Direction;
    type State = Forever<u8>;
    type Update = u8;
    type Error = Void;

    fn initialize(&'m mut self) -> Self::State {
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

        let frame_count = (MIN_VELOCITY / v * FRAMES).ceil();

        Ok(Some(frame_count as u8))
    }
    fn tear_down(&mut self) {}
}

pub struct Forever<T>(T);

impl<T: Copy> Iterator for Forever<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        Some(self.0)
    }
}
