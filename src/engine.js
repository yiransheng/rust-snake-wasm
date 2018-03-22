import { config, initialDraw, createDrawer } from "./render";

export function engine(wasm) {
  const ge = new GameEngine(wasm);
  return {
    run(loop) {
      if (!ge.running()) {
        ge.run(loop);
      }
    },
    running() {
      return ge.running();
    }
  };
}

// commands
export const GetDir = { type: "get_dir" };

export const NextTick = { type: "next_tick" };

export function Fill(...args) {
  return {
    type: "fill",
    args
  };
}
export function Clear(...args) {
  return {
    type: "clear",
    args
  };
}

const initParams = [
  config.width, // world_width
  config.height, // world_height
  1, // tail_x
  1, // tail_y
  3, // dir = East
  4 // length
];

class GameEngine {
  constructor(wasm) {
    this._wasm = wasm;
    this._dir = initParams[4];
    this._gen = null;
    this._frame = 0;
    this._running = false;

    this._onKeyDown = this._onKeyDown.bind(this);
    this._tick = this._tick.bind(this);

    this._initDOM();
  }
  running() {
    return this._running;
  }
  run(loop) {
    this._wasm.exports.start_game(...initParams);
    initialDraw(
      this._drawer,
      initParams[2],
      initParams[3],
      initParams[4],
      initParams[5]
    );
    this._drawer.fill(16, 16, "#f00000");

    this._gen = loop(this._wasm, { foodX: 16, foodY: 16 });
    this._running = true;
    this._dir = initParams[4];
    this._frame = 0;
    requestAnimationFrame(this._tick);
  }
  _initDOM() {
    const h1 = document.createElement("h1");
    h1.innerHTML = "Press Enter to Start";
    document.body.appendChild(h1);
    const canvas = document.createElement("canvas");
    canvas.style.border = "1px solid #000";
    document.body.appendChild(canvas);
    document.addEventListener("keydown", this._onKeyDown);

    this._drawer = createDrawer(canvas);
  }
  _onKeyDown(e) {
    if (this._running) {
      switch (e.keyCode) {
        case 37:
          this._dir = 4;
          break;
        case 38:
          this._dir = 1;
          break;
        case 39:
          this._dir = 3;
          break;
        case 40:
          this._dir = 2;
          break;
        default:
      }
    }
  }
  _tick() {
    if (!this._running) {
      return;
    }
    if (this._frame++ % 4 !== 1) {
      requestAnimationFrame(this._tick);
      return;
    }
    const cmds = [];
    while (true) {
      const { done, value } = this._gen.next();
      let cmd = value;
      if (done) {
        this._stop();
        break;
      }
      cmds.push(cmd);
      if (cmd.type === "get_dir") {
        const { done, value: nextCmd } = this._gen.next(this._dir);
        !done && cmds.push(nextCmd);
      } else if (cmd.type === "next_tick") {
        break;
      } else {
        cmds.push(cmd);
      }
    }
    for (const cmd of cmds) {
      this._exec(cmd);
    }
    requestAnimationFrame(this._tick);
  }
  _exec(cmd) {
    switch (cmd.type) {
      case "fill":
        this._drawer.fill(...cmd.args);
        break;
      case "clear":
        this._drawer.clear(...cmd.args);
        break;
      default:
        return;
    }
  }
  _stop() {
    this._gen = null;
    this._running = false;
    this._frame = 0;
  }
}
