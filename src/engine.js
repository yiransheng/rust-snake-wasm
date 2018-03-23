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
export const GetFrame = { type: "get_frame" };
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
    this._dirs = [initParams[4]];
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
    if (this._running && this._dirs.length < 4) {
      switch (e.keyCode) {
        case 37:
          this._dirs.push(4);
          break;
        case 38:
          this._dirs.push(1);
          break;
        case 39:
          this._dirs.push(3);
          break;
        case 40:
          this._dirs.push(2);
          break;
        default:
      }
    }
  }
  _get_dir() {
    if (this._dirs.length === 1) {
      return this._dirs[0];
    } else {
      return this._dirs.shift();
    }
  }
  _tick() {
    if (!this._running) {
      return;
    }
    this._frame++;

    let input;
    const cmds = [];
    while (true) {
      const { done, value: cmd } = this._gen.next(input);
      if (done) {
        this._stop();
        return;
      }
      
      if (cmd.type === 'next_tick') {
        break;
      } else if (cmd.type === 'get_frame') {
        input = this._frame;
      } else if (cmd.type === 'get_dir') {
        input = this._get_dir();
      } else {
        input = undefined;
      }
      cmds.push(cmd);
    }
    cmds.forEach(cmd => this._exec(cmd));
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
  }
}
