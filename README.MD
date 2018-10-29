# Rust Snake Game

Link: [http://yiransheng.github.io/rust-snake-wasm/](http://yiransheng.github.io/rust-snake-wasm/)



A snake game writing (mostly) in rust, using js for rendering (canvas).

* Implemented using a O(1) update algorithm I came up in high school, when trying to code the thing on a PDA with very limited rendering capacity
* Retained rendering mode, each tick, draws a new head, and erases snake tail (unless food is eaten)
* Probably over-abstracted, but it's ok as this was mostly an exercise to explore rust in-depth





## Build

Requires:

- rust nightly
- wasm-bindgen
- npm / yarn

TODO: more details

