#!/usr/bin/env node
/* Golden capture for `_civConnectVillageAddons` (reference v2.11 25766) and the
 * v1.71 multi-source form of `roadDijkstra` (3301) it drives
 * (`OUTSTANDING_WORK.md`, the "149 of 200 addon villages have no road" row).
 *
 * Runs the reference's OWN text, sliced out of the frozen v2.11 snapshot and
 * never retyped. Every top-level function the connector reaches is sliced by
 * its declaration and closed by a comment/string-aware brace matcher, and each
 * slice's tail is checked against the text it must end with:
 *   _civConnectVillageAddons, roadDijkstra, buildTravelCost, _civRoutingGrid,
 *   _civLandCostGrid, _civTerrainValidTest, _civNearestValidPt, _civSmoothPath,
 *   rdpSimplify, catmullRomSample, _civMarkWayNeighborhood,
 *   _civMarkWaysOnGrid, _civWalkWayCells, and the `_CIV_EXISTING_WAY_DISCOUNT`
 *   declaration line.
 * The globals those read are supplied, not reimplemented: `GW`, `GH`, `field`
 * (a Float32Array), `state` ({seaLevel, world, mapWidthKm}), `civWays` (read
 * only on `_civTerrainValidTest`'s allowSeaLanes branch, which the connector
 * never takes) and `currentWaterBodies()` returning the fixture's classes.
 *
 * Fixture: small synthetic worlds whose height field is INTEGER value noise
 * quantised to q/4096 -- every value is an exact f32, and the Rust test
 * regenerates the same field from the same integer arithmetic (the JSON carries
 * a checksum so a generator mismatch fails as itself, not as a routing diff).
 * Quantisation also puts real ties into the cost grid, which is where a heap's
 * pop order shows. Places, villages and existing ways are emitted as data.
 *
 * The harness REFUSES TO EMIT unless the fixtures reach: a village attached to
 * a SIBLING village (the v1.79 growing forest), a village attached to a real
 * settlement, a village left unreachable, a village whose routing cell is
 * already a source (the `raw.length<2` join-without-drawing branch), a BATCH
 * above its floor of 4, a seam-crossing track with `brks` in world mode, a
 * routing grid with sc < 1, and a village track sharing cells with an
 * existing way (the discount doing something).
 *
 * Usage:  node tools/civ_village_connect_capture.js \
 *           > crates/cartalith-civ/tests/fixtures/village_connect_captured.json
 * The frozen reference is READ ONLY here.
 */
'use strict';
const fs = require('fs');
const path = require('path');

const REF = path.join(__dirname, '..', '..', 'reference', 'Cartalith Gen1 v2.11.html');
const src = fs.readFileSync(REF, 'utf8');
function die(m) { console.error('CAPTURE ABORTED: ' + m); process.exit(1); }

function once(needle) {
  const a = src.indexOf(needle);
  if (a < 0 || src.indexOf(needle, a + 1) >= 0) die(`not exactly one occurrence of: ${needle}`);
  return a;
}
// Brace matcher that skips /* */ and // comments and '…' "…" `…` strings, so a
// brace in prose or a literal cannot close a function early.
function braceBlock(from) {
  let k = src.indexOf('{', from), d = 0;
  for (; k < src.length; k++) {
    const c = src[k], n = src[k + 1];
    if (c === '/' && n === '*') { k = src.indexOf('*/', k + 2) + 1; continue; }
    if (c === '/' && n === '/') { k = src.indexOf('\n', k); continue; }
    if (c === "'" || c === '"' || c === '`') {
      for (k++; src[k] !== c; k++) if (src[k] === '\\') k++;
      continue;
    }
    if (c === '{') d++;
    else if (c === '}' && --d === 0) return src.slice(from, k + 1);
  }
  die('unbalanced braces');
}
function fn(name, tail) {
  const s = braceBlock(once(`function ${name}(`));
  if (!s.endsWith(tail)) die(`${name} slice does not end with ${JSON.stringify(tail)}`);
  return s;
}
const slices = [
  fn('_civConnectVillageAddons', 'return added;\n}'),
  fn('roadDijkstra', 'return {dist, prev};\n}'),
  fn('buildTravelCost', 'return cost;\n}'),
  fn('_civRoutingGrid', 'return {dfld,RW,RH,sc};\n}'),
  fn('_civLandCostGrid', 'return {cost,dfld,RW,RH,sc};\n}'),
  fn('_civTerrainValidTest', 'return false;\n  };\n}'),
  fn('_civNearestValidPt', 'return [x,y];\n}'),
  fn('_civSmoothPath', '}'),
  fn('rdpSimplify', 'return out;\n}'),
  fn('catmullRomSample', 'return out;\n}'),
  fn('_civMarkWayNeighborhood', '}'),
  fn('_civMarkWaysOnGrid', '}'),
  fn('_civWalkWayCells', '}'),
];
for (const must of ['if(Array.isArray(sx)){ for(const si of sx){ if(dist[si]!==0){ dist[si]=0; push(0,si); } } }',
                    'const BATCH=Math.max(4,Math.ceil(villages.length/25));',
                    'roadDijkstra(cost,RW,RH,[...sourceIdxs],null,wrapX)']) {
  if (!slices.some(s => s.includes(must))) die(`no slice contains ${must}`);
}
const discLine = src.slice(once('const _CIV_EXISTING_WAY_DISCOUNT='), src.indexOf('\n', once('const _CIV_EXISTING_WAY_DISCOUNT=')));

// eslint-disable-next-line no-new-func
const connect = new Function('GW', 'GH', 'field', 'state', 'WB', 'places', 'villages', 'ways', `
  const civWays = [];
  function currentWaterBodies(){ return WB; }
  ${discLine}
  ${slices.join('\n')}
  return _civConnectVillageAddons(places, villages, ways);`);

// ------------------------------------------------------------- fixtures ----
// Integer value noise. Mirrored exactly by the Rust test's `fixture_field`.
function hash(ix, iy, seed) {
  let h = Math.imul(ix, 0x27d4eb2d) ^ Math.imul(iy, 0x165667b1) ^ Math.imul(seed, 0x9e3779b1);
  h = Math.imul(h ^ (h >>> 15), 0x85ebca6b);
  h ^= h >>> 13;
  return (h >>> 0) & 4095;
}
function octave(x, y, s, seed) {
  const ix = Math.floor(x / s), iy = Math.floor(y / s), tx = x - ix * s, ty = y - iy * s;
  return Math.floor((hash(ix, iy, seed) * (s - tx) * (s - ty) + hash(ix + 1, iy, seed) * tx * (s - ty)
    + hash(ix, iy + 1, seed) * (s - tx) * ty + hash(ix + 1, iy + 1, seed) * tx * ty) / (s * s));
}
function makeField(w) {
  const q = new Int32Array(w.gw * w.gh);
  for (let y = 0; y < w.gh; y++) for (let x = 0; x < w.gw; x++) {
    let v = Math.floor((2 * octave(x, y, 24, w.seed) + octave(x, y, 7, w.seed + 1)) / 3);
    // An island ring: water moat around a raised square -- unreachable by land.
    if (w.island) {
      const [x0, y0, x1, y1] = w.island;
      const inside = x >= x0 && x <= x1 && y >= y0 && y <= y1;
      const moat = x >= x0 - 4 && x <= x1 + 4 && y >= y0 - 4 && y <= y1 + 4;
      if (inside) v = 3000; else if (moat) v = 200;
    }
    q[y * w.gw + x] = v;
  }
  const field = new Float32Array(q.length);
  for (let i = 0; i < q.length; i++) field[i] = q[i] / 4096;
  let sum = 0; for (let i = 0; i < q.length; i++) sum = (sum * 31 + q[i]) >>> 0;
  return { field, checksum: sum };
}
function makeWb(w, field) {
  const wb = new Uint8Array(field.length);
  for (let i = 0; i < field.length; i++) wb[i] = field[i] < w.sea ? 1 : 0;
  if (w.lake) { const [x0, y0, x1, y1] = w.lake; for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) wb[y * w.gw + x] = 2; }
  return wb;
}

let seed = 20260923;
const rnd = () => { seed = (Math.imul(seed, 1103515245) + 12345) >>> 0; return seed / 4294967296; };
function landCell(w, wb, box) {
  for (let t = 0; t < 10000; t++) {
    const [x0, y0, x1, y1] = box || [1, 1, w.gw - 2, w.gh - 2];
    const x = x0 + Math.floor(rnd() * (x1 - x0 + 1)), y = y0 + Math.floor(rnd() * (y1 - y0 + 1));
    if (wb[y * w.gw + x] === 0) return [x, y];
  }
  die('no land cell');
}

const WORLDS = [
  { name: 'small', gw: 128, gh: 96, seed: 11, sea: 0.40, nBase: 10, nVil: 30, lake: [60, 40, 70, 48], island: [100, 70, 115, 85], islandVillages: 2, world: false },
  { name: 'downsampled', gw: 480, gh: 200, seed: 23, sea: 0.38, nBase: 16, nVil: 130, island: [20, 150, 40, 180], islandVillages: 1, world: false },
  { name: 'wrapped', gw: 160, gh: 80, seed: 37, sea: 0.30, nBase: 6, nVil: 24, world: true, seam: true },
  { name: 'dense', gw: 96, gh: 64, seed: 41, sea: 0.35, nBase: 3, nVil: 110, world: false },
  { name: 'no_base', gw: 64, gh: 48, seed: 5, sea: 0.30, nBase: 0, nVil: 5, world: false },
  { name: 'no_villages', gw: 64, gh: 48, seed: 6, sea: 0.30, nBase: 4, nVil: 0, world: false },
];

const reached = { sibling: 0, toBase: 0, unreachable: 0, coincident: 0, batchAbove4: 0, seamBrks: 0, downsampled: 0, ridesWay: 0 };
const out = [];
for (const w of WORLDS) {
  const { field, checksum } = makeField(w);
  const wb = makeWb(w, field);
  const places = [];
  for (let i = 0; i < w.nBase; i++) {
    const box = w.seam ? (i % 2 ? [1, 1, 12, w.gh - 2] : [w.gw - 13, 1, w.gw - 2, w.gh - 2]) : null;
    const [x, y] = landCell(w, wb, box); places.push({ x, y });
  }
  // Existing ways: a straight land road between the first two places (sampled
  // sparsely, so `_civWalkWayCells`' rasterisation matters), a hidden way, and
  // a sea lane -- the last two must both be ignored by the discount.
  const ways = [];
  if (places.length >= 2) {
    const a = places[0], b = places[1], pts = [];
    for (let s = 0; s <= 6; s++) pts.push([Math.round(a.x + (b.x - a.x) * s / 6), Math.round(a.y + (b.y - a.y) * s / 6)]);
    ways.push({ pts, sea: false, hidden: false });
    if (places.length >= 3) {
      ways.push({ pts: [[places[2].x, places[2].y], [places[1].x, places[1].y]], sea: false, hidden: true });
      ways.push({ pts: [[places[2].x, places[2].y], [places[0].x, places[0].y]], sea: true, hidden: false });
    }
  }
  const villages = [];
  for (let i = 0; i < w.nVil; i++) {
    let x, y;
    if (i === 0 && places.length) { x = places[0].x; y = places[0].y; }                  // same cell as a base place
    else if (i === 1 && places.length && w.gw > 384) { x = places[1].x + 1; y = places[1].y; if (wb[y * w.gw + x]) x = places[1].x; }   // same ROUTING cell at sc<1
    else if (w.island && i >= 2 && i < 2 + w.islandVillages) { [x, y] = landCell(w, wb, w.island); }
    else if (w.seam) { [x, y] = landCell(w, wb, i % 2 ? [w.gw - 12, 1, w.gw - 2, w.gh - 2] : [1, 1, 11, w.gh - 2]); }
    else { [x, y] = landCell(w, wb); }
    villages.push({ x, y, villageAddon: true });
  }
  const all = places.concat(villages);
  const st = { seaLevel: w.sea, world: w.world, mapWidthKm: 800 };
  const res = connect(w.gw, w.gh, field, st, wb, all, villages, ways.map(v => ({ ...v })));

  // reach checks
  const nb = places.length;
  const joined = new Set(res.map(r => r.aIdx));
  for (const r of res) {
    if (r.bIdx >= nb) reached.sibling++; else reached.toBase++;
    if (r.brks && r.brks.length) reached.seamBrks++;
  }
  if (w.nVil > 100) reached.batchAbove4++;
  if (w.gw > 384) reached.downsampled++;
  if (w.island && w.islandVillages) for (let i = 2; i < 2 + w.islandVillages; i++) if (!joined.has(nb + i)) reached.unreachable++;
  if (nb && w.nVil && !joined.has(nb)) reached.coincident++;
  if (ways.length) {
    const cells = new Set(ways[0].pts.map(p => p[0] + ',' + p[1]));
    for (const r of res) if (r.pts.slice(1, -1).some(p => cells.has(p[0] + ',' + p[1]))) { reached.ridesWay++; break; }
  }
  const bits = x => { const b = Buffer.alloc(8); b.writeDoubleLE(x, 0); return b.readBigUInt64LE(0).toString(16).padStart(16, '0'); };
  out.push({
    name: w.name, gw: w.gw, gh: w.gh, seed: w.seed, sea: w.sea, world: w.world, map_width_km: 800,
    island: w.island || null, lake: w.lake || null, field_checksum: checksum,
    places: all.map(p => [p.x, p.y]), village_from: nb,
    ways: ways.map(v => ({ pts: v.pts, sea: v.sea, hidden: v.hidden })),
    expected: res.map(r => ({ a: r.aIdx, b: r.bIdx, pts: r.pts, brks: r.brks || [], km_bits: bits(r.km), type: r.type, village_addon: r.villageAddon })),
  });
  console.error(`${w.name}: ${w.nVil} villages, ${res.length} tracks, ${w.nVil - joined.size} with none`);
}
for (const [k, v] of Object.entries(reached)) if (!v) die(`fixture never reaches ${k}`);
console.error('reached', JSON.stringify(reached));
process.stdout.write(JSON.stringify({ generated_by: 'tools/civ_village_connect_capture.js', reference: 'Cartalith Gen1 v2.11.html', worlds: out }) + '\n');
