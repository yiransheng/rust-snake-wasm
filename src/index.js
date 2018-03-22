import snake from "../rust/target/wasm32-unknown-unknown/release/snake_wasm.small.wasm";
import { engine } from "./engine";
import { gameLoop } from "./game";

snake({
  global: {},
  env: {
    memory: new WebAssembly.Memory({ initial: 50, limit: 1000 }),
    table: new WebAssembly.Table({ initial: 0, element: "anyfunc" })
  }
}).then(({ instance }) => {
  const { run, running } = engine(instance);

  document.addEventListener("keydown", e => {
    if (e.keyCode === 13 && !running()) {
      console.log("starting a game...");
      run(gameLoop);
    }
  });
});
