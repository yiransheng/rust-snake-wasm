# Rust Snake Game

Link: [http://yiransheng.github.io/rust-snake-wasm/](http://yiransheng.github.io/rust-snake-wasm/)



A snake game writing (mostly) in rust, using js for rendering (canvas).



* Implemented using a O(1) update algorithm I came up in high school, when trying to code the thing on a PDA with very limited rendering capacity
* Retained rendering mode, each tick, draws a new head, and erases snake tail (unless food is eaten)
* Unfortunately could not get rand crate to work in wasm, so did random food generation on js side
* State is split across js/rust, rust maintains game state with `'static mut` variable
* Mostly a learning exercise, not optimized much, but performs reasonably well (no problem running at 60fps, although slowed it down by a factor of 4 to make game playable)