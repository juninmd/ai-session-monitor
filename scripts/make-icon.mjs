// Draws the application icon (a gauge ring on a dark rounded square) into a 1024px PNG.
// Regenerate the bundle icons with:
//   node scripts/make-icon.mjs && pnpm tauri icon src-tauri/icons/source.png -o src-tauri/icons
// then delete the android/ and ios/ folders `tauri icon` adds (Windows only app).
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { crc32, deflateSync } from 'node:zlib';

const SIZE = 1024;
const SS = 3; // supersampling per axis
const CENTER = SIZE / 2;

const BG = [0x22, 0x25, 0x2a];
const TRACK = [0x3e, 0x44, 0x4c];
const ACCENT = [0x5a, 0x9b, 0xec];

const HALF = 480;
const CORNER = 224;
const RING_R = 300;
const RING_W = 100;
const SWEEP = (Math.PI * 2 * 3) / 4;

function insideRoundedSquare(x, y) {
  const qx = Math.abs(x) - (HALF - CORNER);
  const qy = Math.abs(y) - (HALF - CORNER);
  const outside = Math.hypot(Math.max(qx, 0), Math.max(qy, 0));
  return outside + Math.min(Math.max(qx, qy), 0) - CORNER <= 0;
}

function inArc(x, y) {
  const r = Math.hypot(x, y);
  const half = RING_W / 2;
  if (Math.abs(r - RING_R) <= half) {
    const phi = (Math.atan2(x, -y) + Math.PI * 2) % (Math.PI * 2);
    if (phi <= SWEEP) return true;
  }
  const capA = Math.hypot(x, y + RING_R) <= half;
  const capB = Math.hypot(x - RING_R * Math.sin(SWEEP), y + RING_R * Math.cos(SWEEP)) <= half;
  return capA || capB;
}

function sample(x, y) {
  if (!insideRoundedSquare(x, y)) return null;
  const r = Math.hypot(x, y);
  if (inArc(x, y)) return ACCENT;
  if (Math.abs(r - RING_R) <= RING_W / 2) return TRACK;
  return BG;
}

const raw = Buffer.alloc((SIZE * 4 + 1) * SIZE);
for (let py = 0; py < SIZE; py++) {
  const row = py * (SIZE * 4 + 1);
  raw[row] = 0; // filter: none
  for (let px = 0; px < SIZE; px++) {
    let r = 0;
    let g = 0;
    let b = 0;
    let a = 0;
    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const color = sample(px + (sx + 0.5) / SS - CENTER, py + (sy + 0.5) / SS - CENTER);
        if (color) {
          r += color[0];
          g += color[1];
          b += color[2];
          a += 1;
        }
      }
    }
    const o = row + 1 + px * 4;
    if (a > 0) {
      raw[o] = Math.round(r / a);
      raw[o + 1] = Math.round(g / a);
      raw[o + 2] = Math.round(b / a);
      raw[o + 3] = Math.round((a / (SS * SS)) * 255);
    }
  }
}

function chunk(type, data) {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(data.length, 0);
  head.write(type, 4, 'ascii');
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([head.subarray(4), data])) >>> 0, 0);
  return Buffer.concat([head, data, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // RGBA

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);

const out = resolve(process.argv[2] ?? 'src-tauri/icons/source.png');
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, png);
process.stdout.write(`wrote ${out} (${png.length} bytes)\n`);
