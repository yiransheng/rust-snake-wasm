use super::{RenderQueue, RenderUnit};
use web_sys::CanvasRenderingContext2d;

pub trait RenderSink<T> {
    fn is_ready(&self) -> bool;

    fn push(&mut self, unit: RenderUnit<T>);
}

pub trait DrawTile {
    const TILE_SIZE: f64;

    // normalized_progress: [0, 1]
    fn draw_tile(&self, gc: &CanvasRenderingContext2d, x: f64, y: f64, normalized_progress: f64);
}

fn _assert_is_object_safe(_: &dyn GameSystem<Msg = (), InputCmd = (), GameOver = ()>) {}

pub trait GameSystem {
    type Msg;
    type InputCmd;
    type GameOver;

    fn start_up(&mut self, &mut RenderQueue<Self::Msg>);

    fn tick(
        &mut self,
        cmd: Self::InputCmd,
        &mut RenderQueue<Self::Msg>,
    ) -> Result<(), Self::GameOver>;

    fn tear_down(&mut self);
}
