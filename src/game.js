import { config } from "./render";
import { GetDir, GetFrame, NextTick, Fill, Clear } from "./engine";

export function* gameLoop(wasmInstance, { foodX, foodY }) {
  while (true) {
    const frame = yield GetFrame; 
    // update direction pressed every tick
    // run game update every 4 ticks
    if (frame % 5 !== 0) {
      yield NextTick;
      continue;
    }
    const dir = yield GetDir;
    const ptr = wasmInstance.exports.tick(dir, foodX, foodY);
    // up to 20 bytes of update, read from WASM memory
    const update = new Int32Array(wasmInstance.exports.memory.buffer, ptr, 5);

    // update layout:
    // [i32] [i32] [i32] [i32] [i32]
    //  tag    x1    y1    x2    y2
    //
    //  tag=0: game over
    //  tag=2: update head position to x1, y1 (implies food eaten)
    //  tag=4: update head position to x1, y1, erase tail at x2, y2
    switch (update[0]) {
      case 2: {
        // generate a new food randomly (cannot get rand crate to work
        // on rust side)
        do {
          foodX = Math.floor(Math.random() * config.width);
          foodY = Math.floor(Math.random() * config.height);
        } while (wasmInstance.exports.is_snake(foodX, foodY) === 1);

        yield Fill(update[1], update[2]);
        yield Fill(foodX, foodY, "#f00000");
        break;
      }
      case 4: {
        yield Clear(update[3], update[4]);
        yield Fill(update[1], update[2]);
        break;
      }
      case 0:
      default:
        return;
        break;
    }

    yield NextTick;
  }
}
