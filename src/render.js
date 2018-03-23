export const config = {
  blockSize: 10,
  width: 64,
  height: 32,
  color: "#222"
};

export function createDrawer(canvas) {
  const widthPixel = config.width * config.blockSize;
  const heightPixel = config.height * config.blockSize;

  canvas.width = widthPixel;
  canvas.height = heightPixel;

  const ctx = canvas.getContext("2d");
  ctx.fillStyle = config.color;

  const { width, height, blockSize } = config;

  return {
    fill(x, y, color) {
      if (color) {
        ctx.save();
        ctx.fillStyle = color;
      }
      if (x >= 0 && x <= width && y >= 0 && y < height) {
        ctx.fillRect(x * blockSize, y * blockSize, blockSize, blockSize);
      }
      if (color) {
        ctx.restore();
      }
    },
    clear(x, y) {
      if (x >= 0 && x <= width && y >= 0 && y < height) {
        ctx.clearRect(x * blockSize, y * blockSize, blockSize, blockSize);
      }
    },
    clearAll() {
      ctx.clearRect(0, 0, widthPixel, heightPixel);
    }
  };
}

export function initialDraw(drawer, x, y, dir, len) {
  drawer.clearAll();
  switch (dir) {
    case 1: {
      // N
      for (let i = 0; i < len; i++) {
        drawer.fill(x, y - i);
      }
      break;
    }
    case 2: {
      // S
      for (let i = 0; i < len; i++) {
        drawer.fill(x, y + i);
      }
      break;
    }
    case 3: {
      // E
      for (let i = 0; i < len; i++) {
        drawer.fill(x + i, y);
      }
      break;
    }
    case 4: {
      // W
      for (let i = 0; i < len; i++) {
        drawer.fill(x - i, y);
      }
      break;
    }
    default:
  }
}
