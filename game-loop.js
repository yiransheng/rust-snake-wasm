/* @flow */

import Hammer from "hammerjs";

type Listener<E> = (event: E) => void;
type Unlisten<E> = () => void;

type DOMEvent = KeyboardEvent | MouseEvent;
type KeyCode = number;

type KeyboxEvent =
  | { type: "tap", keyCode: KeyCode }
  | { type: "press", keyCode: KeyCode }
  | { type: "pressup", keyCode: KeyCode };

function addEventLisitener<E: DOMEvent>(
  event: *,
  listener: Listener<E>
): Unlisten<E> {
  document.addEventListener(event, listener);

  return function unlisten() {
    document.removeEventListener(event, listener);
  };
}

const keyboxSelector = ".keybox";

export class Keybox {
  _listeners: Array<Listener<KeyboxEvent>>;
  _element: HTMLElement;
  _scale: number;

  constructor() {
    this._listeners = [];

    const element = document.querySelector(keyboxSelector);
    if (element) {
      this._element = element;
      this._scale = 1;

      const hammer = new Hammer(this._element);
      hammer.get("pinch").set({ enable: true });

      hammer.on("tap", e => {
        if (e.tapCount === 2) {
          const isZoomedIn = this._scale > 1.0;
          isZoomedIn ? this._zoomOut() : this._zoomIn();
        }

        const keyCode = Number(e.target.dataset.keyCode);
        if (keyCode) {
          this._emit("tap", keyCode);
        }
      });
      hammer.on("press", e => {
        const keyCode = Number(e.target.dataset.keyCode);
        if (keyCode) {
          this._emit("press", keyCode);
        }
      });
      hammer.on("pressup", e => {
        const keyCode = Number(e.target.dataset.keyCode);
        if (keyCode) {
          this._emit("pressup", keyCode);
        }
      });
    } else {
      throw Error("Keybox element not on page");
    }
  }

  _zoomIn() {
    if (this._scale < 3) {
      this._scale += 0.5;
    } else {
      this._scale = 3;
    }
    this._resize();
  }
  _zoomOut() {
    if (this._scale > 1) {
      this._scale -= 0.5;
    } else {
      this._scale = 1.0;
    }
    this._resize();
  }
  _resize() {
    const fontSize = 24 * this._scale;
    this._element.style.fontSize = `${fontSize}px`;
  }

  _emit(type: "tap" | "press" | "pressup", keyCode: KeyCode) {
    const currListeners = this._listeners;
    const nextListeners = [...this._listeners];

    this._listeners = nextListeners;

    currListeners.forEach(f =>
      f(
        (({
          type,
          keyCode
        }: any): KeyboxEvent)
      )
    );
  }

  addKeyLisitener(listener: Listener<KeyboxEvent>): Unlisten<KeyboxEvent> {
    this._listeners.push(listener);

    return () => {
      this._listeners = this._listeners.filter(f => f !== listener);
    };
  }
}

export class GameLoop {
  _keybox: Keybox;

  _onEnterFrame: Function;
  _rafId: ?AnimationFrameID;
  _key: number;
  _isTap: boolean;
  _unlistens: Array<Unlisten<KeyboardEvent> | Unlisten<KeyCode>>;

  _run: Function;
  _onKeyUp: Listener<KeyboardEvent>;
  _onKeyDown: Listener<KeyboardEvent>;

  constructor(fn: Function) {
    this._onEnterFrame = fn;
    this._rafId = null;
    this._unlistens = [];
    this._key = 0;
    this._isTap = false;

    this._run = this._run.bind(this);
    this._onKeyDown = this._onKeyDown.bind(this);
    this._onKeyUp = this._onKeyUp.bind(this);

    this._keybox = new Keybox();

    const unlisten = this._keybox.addKeyLisitener(this._onKeybox.bind(this));

    this._unlistens.push(unlisten);
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

        if (this._isTap) {
          this._key = 0;
          this._isTap = false;
        }
      } catch (err) {
        this.stop();
      }
    }
  }

  _onKeybox(e: KeyboxEvent) {
    switch (e.type) {
      case "tap":
        this._key = e.keyCode;
        this._isTap = true;
        break;
      case "press":
        this._key = e.keyCode;
        this._isTap = false;
        break;
      default:
      case "pressup":
        this._key = 0;
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
      case 87: // w
      case 75: // k
        this._key = 38;
        break;
      case 74: // j
      case 83: // s
        this._key = 40;
        break;
      case 72: // h
      case 65: // a
        this._key = 37;
        break;
      case 76: // l
      case 68: // d
        this._key = 39;
        break;
      default:
    }
  }
  _onKeyUp() {
    this._key = 0;
  }
}
