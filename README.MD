# Rust WASM Snake Game

Play Link: [http://yiransheng.github.io/rust-snake-wasm/](http://yiransheng.github.io/rust-snake-wasm/)



![](./screenshots/games.png)



A weirdly designed snake game in rust & WebAssembley, primarily aimed for retained mode rendering targeting canvas. Two non-browser targets are also included in [./non_browser](./non_browser).

## Features

* Snake movements are animated and smoothed over
* Variable render speed, hold down direction keys to accelerate (opposite direction key to deaccelerate)

![](./screenshots/acceleration.gif)

* VIM key bindings supported (h,j,k,l)
* `no_std`
* Retained rendering; in each tick, only one tile is drawn or cleared leaving vast majority of pixels on canvas untouched
* Perhaps too over-engineered, trying to explore various fun and unique abstractions afforded by rust
* Minimal world model completely decoupled from drawing code, easily ported over to other environments and rendering targets (although in trying to run it with piston, retained rendering doesn't quite work..)
  * ..and enabled writing unit tests nicely like so:

```rust
#[test]
fn test_death() {
    let snake_string = indoc!(
        "
        ..........
        >>>>>>v...
        ......v...
        ..^<<<<...
        .........*"
    );
    let afterwards = indoc!(
        "
        ..........
        .oooooo...
        ..o...o...
        ..ooooo...
        .........*"
    );

    let mut world: World<SmallRng, Bounding> = World::from_ascii(snake_string);

    while let Ok(_) = world.step(None) {}

    assert_matches!(world.step(None), Err(UpdateError::CollideBody));

    assert_eq!(&afterwards, &world.grid.to_string());
}
```



## Usage

### Stuff to Install

1. `wasm32-unknown-unknown` target and `wasm-bindgen-cli`

```
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo +nightly install wasm-bindgen-cli
```

2. `node` and `yarn`
3. (optional) `wasm-opt` install from: [https://github.com/WebAssembly/binaryen](https://github.com/WebAssembly/binaryen)

### Build

Development

```
make dev
```

Build (cargo release build, `no_std`, `wee_alloc`, `webpack --mode=production` and `wasm-opt`)

```
make
# serve ./docs
```

Test

```
cargo test
```

## Brief Overview

The core structure of this game is `World` (mod: `world`), its side-effects/outputs are:

```rust
enum WorldUpdate {
    SetBlock { block: Block, at: Coordinate },
    Clear { prev_block: Block, at: Coordinate },
    SetWorldSize(u16, u16),
}

enum UpdateError {
    HeadDetached,  // panics, game bug, should not happen
    TailDetached,  // panics, game bug, should not happen
    OutOfBound,
    CollideBody,
}

type Result<T> = ::std::result<T, UpdateError>;
```

`World` itself is just:

```rust
struct World<R: Rng, BB: BoundingBehavior = Wrapping> {
    fn initialize(&'a mut self) -> impl Iterator<Item=WorldUpdate> {
        // ...
    }

    fn tear_down(&mut self) {
        // ...
    }

    fn step(&mut self, cmd: Option<Direction>) -> Result<Option<WorldUpdate>> {
        // ...
    }
}
```

It's complete devoid of drawing code and does not concern itself with game start/stop and other game loop level controls. This allowed itself to be reused for very different runtimes (browser, terminal etc., Although when trying to adapt it to a `piston_window`, not exposing its internal state really drove things into a corner).

`initialize` returns an `Iterator` of `WorldUpdate`, this simplifies things for renderers, as they only need to deal with `WorldUpdate` data type alone, and do not need to worry about behaving differently during initialization vs. normal game play.

This formulation of game world is abstracted out as a `trait` (generic lifetime ``m` allows `initialize` to return a iterator that borrows the struct itself).

```rust
trait Stateful<'m> {
    type Cmd;
    type Init: IntoIterator<Item = Self::Update> + 'm;
    type Update;
    type Error: Into<GameOver>;

    fn initialize(&'m mut self) -> Self::Init;

    fn step(
        &mut self,
        cmd: Option<Self::Cmd>,
    ) -> Result<Option<Self::Update>, Self::Error>;

    fn tear_down(&mut self);
}
```

Two other addons that implement this `trait` are `RenderSpeed` and `Dead`, responsible for controlling acceleration and gameover/restart respectively. In the end all things are glued together with provided combinator methods on `Stateful` `trait` :

```rust
    let world: World<SmallRng, Wrapping> = WorldBuilder::new()
        .width(64)
        .height(32)
        .set_snake(1, 1)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .extend(Direction::East)
        .build_with_seed([123; 16]);

    let game = world
        .zip_with(RenderSpeed::new(Direction::East), VariableFrame::pack)
        .alternating::<Key, _>(Dead::new())
        .make_game(CanvasEnv::new());
```

This same code is pretty much reused for `piston_snake` and `terminal_snake` as well, just using different `Env` (than `CanvasEnv`) implementations there. 

Game loop is provided by `js` side:

```rust
#[wasm_bindgen(module = "./game-loop")]
extern "C" {
    type GameLoop;

    #[wasm_bindgen(constructor)]
    fn new(run: &Closure<FnMut(u8)>) -> GameLoop;

    #[wasm_bindgen(method)]
    fn start(this: &GameLoop) -> bool;

    #[wasm_bindgen(method)]
    fn stop(this: &GameLoop) -> bool;
}
```

`js` class `GameLoop` takes a closure from `wasm`, and run it on a `requestAnimationFrame` loop, supplying a `u8` for pressed key code in each tick.

Finally game update code is packaged as a `Generator`. Since rust does not allow sending data into generator (unlike `javaScript` and `python`), a channel-like "`Sender`" is also returned to feed in data/commands (it's just a `Rc<RefCell<InputBuffer<_>>>` under the hood), but I imagine in a multi-threaded context by using `std::sync::mpsc::channel` , this pattern would still work.

```rust
let (tx, mut generator) = game.new_game();

let each_tick = Closure::wrap(Box::new(move |key: u8| {
    let key = Key::from(key);

    tx.send(key);

    unsafe {
        generator.resume();
    }
}) as Box<FnMut(_)>);

let game_loop = GameLoop::new(&each_tick);

game_loop.start();
each_tick.forget();
```

The returned `generator` uses `Generator` `yield` syntax to encode a simple state machine that alternates between rendering ticks and game logic tick. Getting [this piece](./src/system/state.rs#L252) to compile (and not leak memory) took me a long time to figure out, but it was a pretty good exercise to understand `rust` ownership model on a deeper level.

While this architecture is largely unnecessary for such a simple game (and probably does not scale to real world games at all) - going about in in a very generic and modular way and having everything tied together in the end was still very satisfying.