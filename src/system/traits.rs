use super::RenderQueue;
use web_sys::CanvasRenderingContext2d;

pub trait DrawTile {
    const TILE_SIZE: f64;

    // normalized_progress: [0, 1]
    fn draw_tile(&self, gc: &CanvasRenderingContext2d, x: f64, y: f64, normalized_progress: f64);
}

pub trait GameSystem {
    type Msg;
    type GameOver;

    fn start_up(&mut self, &mut RenderQueue<Self::Msg>);

    fn tick(&mut self, &mut RenderQueue<Self::Msg>) -> Result<(), Self::GameOver>;

    fn tear_down(&mut self);
}
