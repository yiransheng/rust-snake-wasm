/* @flow */

import Hammer from 'hammerjs';

const keybox = '.keybox';

export class Keybox {
  constructor() {
    this._element = document.querySelector(keybox);
    this._hammer = new Hammer(this._element);
  }

}

export class GameLoop {
  _onEnterFrame: Function;
  _rafId: ?number;
  _key: number;
  _unlistens: Array<Function>;

  _run: Function;
  _onKeyUp: Function;
  _onKeyDown: Function;


  constructor(fn: Function) {
    this._onEnterFrame = fn;
    this._rafId = null;
    this._unlistens = [];
    this._key = 0;

    this._run = this._run.bind(this);
    this._onKeyDown = this._onKeyDown.bind(this);
    this._onKeyUp = this._onKeyUp.bind(this);
  }

  running(): boolean {
    return this._rafId !== null;
  }

  start() {
    if (!this.running()) {
      this._rafId = window.requestAnimationFrame(this._run);
      this._unlistens.push(addEventLisitener("keydown", this._onKeyDown));
      this._unlistens.push(addEventLisitener("keyup", this._onKeyUp));
      return true;
    }

    return false;
  }
  stop() {
    if (this.running()) {
      this._rafId && cancelAnimationFrame(this._rafId);
      this._rafId = null;
      this._unlistens.forEach(f => f());
      this._unlistens.length = 0;
      return true;
    }

    return false;
  }

  _run() {
    if (this.running()) {
      try {
        this._onEnterFrame(this._key);
        this._rafId = window.requestAnimationFrame(this._run);
      } catch (err) {
        this.stop();
      }
    }
  }

  _onKeyDown(e: KeyboardEvent) {
    switch (e.keyCode) {
      case 13:
      //case 27:
      case 37:
      case 38:
      case 39:
      case 40:
        this._key = e.keyCode;
        break;
      default:
    }
  }
  _onKeyUp() {
    this._key = 0;
  }
}

function addEventLisitener(event, listener) {
  document.addEventListener(event, listener);

  return function unlisten() {
    document.removeEventListener(event, listener);
  };
}
