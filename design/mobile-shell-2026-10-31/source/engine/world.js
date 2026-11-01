/* ---------- Cartalith mock world: a small deterministic terrain so the shell has a real map to drive ----------
   Not the engine. A labelled stand-in: fractal noise continents, a priority-flood fill, D8 flow accumulation for
   rivers, latitude/elevation climate and a biome table. Coordinates are kilometres on a 2 048 km square. */
function mulberry32(a) { return function () { a |= 0; a = a + 0x6D2B79F5 | 0; let t = Math.imul(a ^ a >>> 15, 1 | a); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; }; }
function makeNoise(seed) {
  const R = mulberry32(seed), P = new Uint16Array(512), G = new Float32Array(256);
  for (let i = 0; i < 256; i++) { P[i] = i; G[i] = R() * 2 - 1; }
  for (let i = 255; i > 0; i--) { const j = (R() * (i + 1)) | 0; const t = P[i]; P[i] = P[j]; P[j] = t; }
  for (let i = 0; i < 256; i++) P[i + 256] = P[i];
  return function (x, y) {
    const xi = Math.floor(x), yi = Math.floor(y), xf = x - xi, yf = y - yi, X = xi & 255, Y = yi & 255;
    const a = G[P[P[X] + Y]], b = G[P[P[X + 1] + Y]], c = G[P[P[X] + Y + 1]], d = G[P[P[X + 1] + Y + 1]];
    const u = xf * xf * xf * (xf * (xf * 6 - 15) + 10), v = yf * yf * yf * (yf * (yf * 6 - 15) + 10);
    return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
  };
}
function fbm(n, x, y, oct) { let s = 0, a = 0.5, f = 1, norm = 0; for (let i = 0; i < oct; i++) { s += a * n(x * f, y * f); norm += a; a *= 0.5; f *= 2.03; } return s / norm; }
function genWorld(seed) {
  const N = 384, KM = 2048, cell = KM / N;
  const n1 = makeNoise(seed), n2 = makeNoise(seed + 17), n3 = makeNoise(seed + 101), n4 = makeNoise(seed + 911), n5 = makeNoise(seed + 4242);
  const e = new Float32Array(N * N);
  for (let j = 0; j < N; j++) for (let i = 0; i < N; i++) {
    const x = i / N, y = j / N, dx = x - 0.5, dy = y - 0.5;
    const d = Math.sqrt(dx * dx * 1.05 + dy * dy * 1.25);
    const warp = fbm(n5, x * 2 + 9, y * 2 + 4, 3) * 0.18;
    let c = fbm(n1, x * 2.1 + 3.1 + warp, y * 2.1 + 1.7 - warp, 7) * 1.45 + 0.43 - d * 1.05;
    const rr = 1 - Math.abs(fbm(n2, x * 3.4 + 7 + warp, y * 3.4 + 2, 5)) * 2.1;
    const ridge = Math.max(0, rr);
    const land = Math.max(0, Math.min(1, c * 4 + 0.15));
    const belt = Math.max(0, fbm(n4, x * 1.6 + 2, y * 1.6 + 6, 3) + 0.15) * 2.2;
    e[j * N + i] = c + ridge * ridge * 0.95 * land * Math.min(1, belt) + Math.abs(fbm(n3, x * 12, y * 12, 4)) * 0.16 * land + fbm(n3, x * 30, y * 30, 2) * 0.02;
    const bx = Math.min(x, 1 - x, y, 1 - y); if (bx < 0.1) e[j * N + i] -= (0.1 - bx) * 9; // the world ends in open sea, so the map has no visible edge
  }
  // priority-flood (Barnes et al. 2014) so every land cell drains to the sea or a lake
  const filled = Float32Array.from(e), closed = new Uint8Array(N * N);
  const heap = new Int32Array(N * N); let hn = 0;
  const less = (a, b) => filled[a] < filled[b];
  const push = (v) => { let i = hn++; heap[i] = v; while (i > 0) { const p = (i - 1) >> 1; if (less(heap[i], heap[p])) { const t = heap[i]; heap[i] = heap[p]; heap[p] = t; i = p; } else break; } };
  const pop = () => { const top = heap[0]; heap[0] = heap[--hn]; let i = 0; for (;;) { const l = 2 * i + 1, r = l + 1; let m = i; if (l < hn && less(heap[l], heap[m])) m = l; if (r < hn && less(heap[r], heap[m])) m = r; if (m === i) break; const t = heap[i]; heap[i] = heap[m]; heap[m] = t; i = m; } return top; };
  for (let k = 0; k < N * N; k++) { const i = k % N, j = (k / N) | 0; if (e[k] < 0 || i === 0 || j === 0 || i === N - 1 || j === N - 1) { closed[k] = 1; push(k); } }
  const DX = [-1, 0, 1, -1, 1, -1, 0, 1], DY = [-1, -1, -1, 0, 0, 1, 1, 1];
  while (hn) { const c = pop(), ci = c % N, cj = (c / N) | 0; for (let q = 0; q < 8; q++) { const ni = ci + DX[q], nj = cj + DY[q]; if (ni < 0 || nj < 0 || ni >= N || nj >= N) continue; const nk = nj * N + ni; if (closed[nk]) continue; closed[nk] = 1; if (filled[nk] <= filled[c]) filled[nk] = filled[c] + 1e-6; push(nk); } }
  // D8 receivers and accumulation (cells in decreasing filled order)
  const recv = new Int32Array(N * N).fill(-1);
  for (let k = 0; k < N * N; k++) { if (e[k] < 0) continue; const ci = k % N, cj = (k / N) | 0; let best = -1, bz = filled[k]; for (let q = 0; q < 8; q++) { const ni = ci + DX[q], nj = cj + DY[q]; if (ni < 0 || nj < 0 || ni >= N || nj >= N) continue; const nk = nj * N + ni; const z = filled[nk]; if (z < bz) { bz = z; best = nk; } } recv[k] = best; }
  const order = new Int32Array(N * N); for (let k = 0; k < N * N; k++) order[k] = k;
  order.sort((a, b) => filled[b] - filled[a]);
  const acc = new Float32Array(N * N).fill(1);
  for (let t = 0; t < N * N; t++) { const k = order[t]; const r = recv[k]; if (r >= 0) acc[r] += acc[k]; }
  // climate
  const elevM = new Float32Array(N * N), temp = new Float32Array(N * N), rain = new Float32Array(N * N), slope = new Float32Array(N * N);
  let maxE = 0; for (let k = 0; k < N * N; k++) if (e[k] > maxE) maxE = e[k];
  for (let k = 0; k < N * N; k++) { const z = e[k]; elevM[k] = z > 0 ? Math.pow(z / maxE, 1.35) * 4300 : z * 5200; }
  for (let j = 0; j < N; j++) for (let i = 0; i < N; i++) {
    const k = j * N + i, x = i / N, y = j / N;
    const gx = (elevM[j * N + Math.min(N - 1, i + 1)] - elevM[j * N + Math.max(0, i - 1)]) / (2 * cell * 1000);
    const gy = (elevM[Math.min(N - 1, j + 1) * N + i] - elevM[Math.max(0, j - 1) * N + i]) / (2 * cell * 1000);
    slope[k] = Math.atan(Math.sqrt(gx * gx + gy * gy)) * 180 / Math.PI;
    temp[k] = -9 + 40 * y - Math.max(0, elevM[k]) * 0.0065 + fbm(n4, x * 4, y * 4, 3) * 4;
    const windward = Math.max(0, -gx * 900);
    rain[k] = Math.max(80, 520 + fbm(n4, x * 3 + 5, y * 3 + 8, 4) * 900 + windward * 260 + (1 - Math.abs(y - 0.55) * 1.6) * 420);
  }
  return { N, KM, cell, e, filled, recv, acc, elevM, temp, rain, slope, maxE };
}
function biomeOf(w, k) {
  const t = w.temp[k], r = w.rain[k], z = w.elevM[k];
  if (z < 0) return 'ocean';
  if (w.filled[k] - w.e[k] > 0.025) return 'lake';
  if (t < -10 || z > 3950) return 'ice';
  if (t < -3) return 'tundra';
  if (t < 3) return r > 520 ? 'taiga' : 'tundra';
  if (t > 21 && r > 1250) return 'tropical forest';
  if (t > 19 && r < 420) return 'desert';
  if (t > 18 && r < 820) return 'savanna';
  if (r < 460) return 'steppe';
  if (r > 980) return 'temperate forest';
  return 'grassland';
}
const BIOME_COL = { ocean: [36, 64, 84], lake: [64, 108, 132], ice: [232, 236, 238], tundra: [168, 170, 150], taiga: [76, 104, 82], 'tropical forest': [52, 108, 64], desert: [214, 188, 136], savanna: [190, 176, 104], steppe: [178, 168, 118], 'temperate forest': [72, 118, 74], grassland: [132, 156, 92] };
if (typeof module !== 'undefined') module.exports = { genWorld, biomeOf, makeNoise, fbm, mulberry32, BIOME_COL };
