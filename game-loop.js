export class GameLoop {
  constructor(fn) {
    this._onEnterFrame = fn;
    this._rafId = null;
    this._run = this._run.bind(this);
  }

  get running() {
    return this._rafId !== null;
  }

  start() {
    if (!this.running) {
      this._rafId = window.requestAnimationFrame(this._run);
      return true;
    }

    return false;
  }
  stop() {
    if (this.running) {
      cancelAnimationFrame(this._rafId);
      this._rafId = null;
      return true;
    }

    return false;
  }

  _run() {
    if (this.running) {
      this._onEnterFrame();
      this._rafId = window.requestAnimationFrame(this._run);
    }
  }
}
