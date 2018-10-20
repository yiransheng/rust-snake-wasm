use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use data::{Coordinate, Direction, Tile};
use game::Renderer;
use patch::*;
use world::WorldUpdateEff;

pub struct CanvasRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        (document.body().unwrap().as_ref() as &web_sys::Node)
            .append_child(canvas.as_ref() as &web_sys::Node)
            .unwrap();
        canvas.set_width(640);
        canvas.set_height(480);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context.set_fill_style(&"rgba(0, 0, 0, 1)".into());

        CanvasRenderer { canvas, context }
    }
}

impl Renderer for CanvasRenderer {
    type UpdateEff = Patch<WorldUpdateEff>;

    fn render<F>(&mut self, mut eff: Self::UpdateEff, callback: F)
    where
        F: Fn(),
    {
        let w = 64;
        let h = 48;

        web_sys::console::log_1(&"Render".into());

        for update in eff.iter() {
            match update {
                WorldUpdateEff::SetBlock { at, block } => {
                    let x = (at.x * 10) as f64;
                    let y = (at.y * 10) as f64;
                    self.context.fill_rect(x, y, 10.0, 10.0);
                }
                WorldUpdateEff::Clear { at, prev_block } => {
                    let x = (at.x * 10) as f64;
                    let y = (at.y * 10) as f64;
                    self.context.clear_rect(x, y, 10.0, 10.0);
                }
            }
        }

        callback();
    }
}
