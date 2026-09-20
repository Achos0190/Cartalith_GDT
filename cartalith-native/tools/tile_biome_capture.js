#!/usr/bin/env node
/* Golden capture for `LOD_DETAIL_SCOPE.md` LOD-D1 -- `renderBiomeTileRGBA`,
 * reference HTML `Cartalith Gen1 v2.11.html` lines 11668-11779.
 *
 * The loader is `um_block2_capture.js`'s, lines 109-225, reused rather than
 * reinvented: all FOUR `<script>` blocks evaluate in one bare `vm` context, in
 * browser order, behind a self-similar DOM Proxy. Its three traps apply here
 * unchanged and are re-asserted live below:
 *
 *   1. `GW`/`GH`/`field`/`state`/... are `let`/`const` GLOBAL LEXICAL
 *      BINDINGS. Setting `ctx.GW` does nothing; every assignment must be
 *      RUN INSIDE the context. Proven, not trusted.
 *   2. Node's real `URL` ABORTS THE PROCESS (SIGABRT) on block 1's top-level
 *      `URL.createObjectURL(new Blob(...))`, so `URL` is shadowed by a stub.
 *   3. Block 1 calls `Math.random()` at top level for `state.tect.seed`, so
 *      the seed is overwritten inside the context before anything is captured.
 *      There is no purity assertion to make here and pretending otherwise
 *      would be dishonest.
 *
 * **v2.11, not v2.10.** Root `CLAUDE.md`: *"the version this repository ships
 * ... Read this one when you are reading the reference."* `LOD_DETAIL_SCOPE.md`
 * cites v2.11 line numbers for this function, and they were re-verified
 * against the file (`renderBiomeTileRGBA` opens at 11668 and its closing brace
 * is at 11779) before this script was written. The two existing harnesses in
 * this directory point at v2.10; that is a difference to know about, not to
 * copy.
 *
 * The frozen file is READ ONLY here and is never written.
 *
 * Usage:
 *   node tools/tile_biome_capture.js \
 *     > crates/cartalith-godot/tests/fixtures/tile_biome_fixture.rs
 *
 * (run from `cartalith-native/`). It goes in `tests/fixtures/`, not in
 * `tests/`, because cargo makes every top-level `tests/*.rs` its own test
 * target and this file has no tests in it -- `golden_parity_tile_biome.rs`
 * pulls it in with `#[path]`.
 *
 * Prints the generated Rust on stdout; diagnostics go to stderr. Refuses to
 * print anything at all unless every shape gate at the bottom passes.
 */
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ROOT = path.resolve(__dirname, '..', '..');
const REF = path.join(ROOT, 'reference', 'Cartalith Gen1 v2.11.html');

function die(msg) { console.error('CAPTURE ABORTED: ' + msg); process.exit(1); }

// ------------------------------------------------------- the script blocks --

const lines = fs.readFileSync(REF, 'utf8').split(/\r?\n/);

function scriptBlocks() {
  const out = [];
  let open = -1;
  for (let k = 0; k < lines.length; k++) {
    const t = lines[k].trim();
    if (t === '<script>') { if (open >= 0) die(`nested <script> at line ${k + 1}`); open = k + 2; }
    else if (t === '</script>') {
      if (open < 0) die(`</script> with no opener at line ${k + 1}`);
      out.push([open, k]); open = -1;
    }
  }
  if (open >= 0) die('unclosed <script>');
  return out;
}

const BLOCKS = scriptBlocks();
if (BLOCKS.length !== 4) die(`expected 4 <script> blocks, found ${BLOCKS.length}`);

// --- STRUCTURAL ASSERTION: the function under test is where we say it is -----
// Not "the file contains the name somewhere" -- the exact opening line at the
// exact number, and the exact closing brace. `MISTAKES.md`: *"Verify a scope
// document's line ranges against the real reference before slicing"*, and
// *"Re-resolve a citation late in a long pass"*.
const FN_FIRST = 11668, FN_LAST = 11779;
if (lines[FN_FIRST - 1] !== 'function renderBiomeTileRGBA(tile,W,H,bounds){') {
  die(`line ${FN_FIRST} is not renderBiomeTileRGBA's opening line; got:\n  ${lines[FN_FIRST - 1]}`);
}
if (lines[FN_LAST - 1] !== '}') {
  die(`line ${FN_LAST} is not the function's closing brace; got:\n  ${lines[FN_LAST - 1]}`);
}
// and it is inside block 1, where the renderer lives
if (!(FN_FIRST > BLOCKS[0][0] && FN_LAST < BLOCKS[0][1])) {
  die(`renderBiomeTileRGBA is not inside block 1 (${BLOCKS[0][0]}-${BLOCKS[0][1]})`);
}

// ---------------------------------------------------------- the DOM stub ----

function domStub() {
  const f = function () { return P; };
  const P = new Proxy(f, {
    get(_t, k) {
      if (k === Symbol.toPrimitive) return () => 0;
      if (k === Symbol.iterator) return function* () {};
      if (k === 'then') return undefined;
      if (k === 'length') return 0;
      if (k === 'constructor') return Object;
      return P;
    },
    set() { return true; },
    has() { return true; },
    apply() { return P; },
    construct() { return P; },
  });
  return P;
}

const D = domStub();
const ctx = vm.createContext({
  console,
  document: D, navigator: D, localStorage: D, sessionStorage: D, history: D, screen: D,
  requestAnimationFrame: () => 0, cancelAnimationFrame: () => 0,
  setTimeout: () => 0, clearTimeout: () => 0, setInterval: () => 0, clearInterval: () => 0,
  addEventListener: () => {}, removeEventListener: () => {},
  Image: function () { return D; }, Path2D: function () { return D; },
  OffscreenCanvas: function () { return D; }, ImageData: function () { return D; },
  Worker: function () { return D; }, Blob: function () { return D; },
  matchMedia: () => D, alert: () => {}, prompt: () => null, confirm: () => false,
  URL: { createObjectURL: () => 'blob:stub', revokeObjectURL: () => {} },
  TextDecoder, TextEncoder, performance: { now: () => 0 },
  devicePixelRatio: 1, location: { href: '', search: '' },
  module: { exports: {} },
});
ctx.window = ctx; ctx.self = ctx; ctx.globalThis = ctx;

for (let b = 0; b < 4; b++) {
  const [a, z] = BLOCKS[b];
  try { vm.runInContext(lines.slice(a - 1, z).join('\n'), ctx, { filename: `block${b + 1}.js` }); }
  catch (e) { die(`block ${b + 1} (${a}-${z}) threw ${e.constructor.name}: ${e.message}`); }
}

const run = (src) => vm.runInContext(src, ctx, { filename: 'harness.js' });

// Trap 1, proven live rather than trusted.
ctx.GW = 12345;
if (run('GW') === 12345) die('GW is a context property, not a lexical binding -- rewrite this harness');
delete ctx.GW;
if (typeof run('renderBiomeTileRGBA') !== 'function') die('renderBiomeTileRGBA is not defined after the blocks evaluated');
if (typeof run('sharedSeaFields') !== 'function') die('sharedSeaFields is not defined -- the v1.29 seam fix is missing');

// ---------------------------------------------------------- the fixture -----
//
// GW = GH = 10, seed 24601, `state.world = false` -- deliberately the SAME
// world `golden_parity_render.rs` was captured against, so a divergence
// between that file's `cell_color` numbers and this file's tile numbers is
// attributable to the tile path and to nothing else.

const SEED = 24601;
const GRID = 10;

run(`
  state.tect.seed = ${SEED};
  state.seed = ${SEED};
  state.world = false;
  state.resW = ${GRID};
  GW = ${GRID}; GH = ${GRID};
  allocate();
`);
if (run('GW') !== GRID || run('GH') !== GRID) die(`GW/GH did not take: ${run('GW')}x${run('GH')}`);

(async () => {
  await run('generate()');

  if (run('field.length') !== GRID * GRID) die(`field is ${run('field.length')} long, expected ${GRID * GRID}`);
  if (run('!flowField')) die('flowField is empty after generate()');

  const grab = (name) => Array.from(run(name));
  const FIELD = grab('field');
  const TEMP = grab('tempField');
  const RAIN = grab('rainField');
  const FLOW = grab('flowField');
  const SEA = run('state.seaLevel');
  const MAP_KM = run('state.mapWidthKm');
  const EXAG = run('state.exag');
  const SUN_AZ = run('state.sunAz');
  const BIO_BLEND = run('state.bioBlend');
  const RIVER_THRESH = run('riverFlowThresh(GW,GH)');
  // `latAt` (reference 4991) reads `state.climate.latN/latS`, not `state.latN`.
  const LAT_N = run('state.climate.latN'), LAT_S = run('state.climate.latS');
  if (typeof LAT_N !== 'number' || typeof LAT_S !== 'number') die(`state.climate.latN/latS are not numbers: ${LAT_N}/${LAT_S}`);

  const water = FIELD.filter((v) => v < SEA).length;
  if (water === 0 || water === FIELD.length) die(`fixture is all-${water ? 'water' : 'land'} -- one colour branch is unreachable`);
  console.error(`fixture: ${GRID}x${GRID}, ${water}/${FIELD.length} cells below sea level ${SEA}`);
  console.error(`state: exag=${EXAG} sunAz=${SUN_AZ} bioBlend=${BIO_BLEND} mapWidthKm=${MAP_KM} latN=${LAT_N} latS=${LAT_S}`);
  console.error(`riverFlowThresh(GW,GH) = ${RIVER_THRESH}`);

  // --- the tile ------------------------------------------------------------
  // An AMPLIFIED tile is an INPUT to the function under test, so it is dumped
  // into the fixture rather than reproduced on the Rust side: this captures
  // `renderBiomeTileRGBA`, not `amplifyRegion`. It is built as a bilinear read
  // of the coarse field plus a deterministic sub-cell ripple, which is the
  // shape a real refined tile has (a smooth base plus seeded detail) without
  // dragging the amplifier's own golden into this one. Stored through a
  // `Float32Array` so the values are f32 on both sides.
  const TW = 32, TH = 32;
  function makeTile(bx, by, bw, bh) {
    const t = new Float32Array(TW * TH);
    const cx = bw / Math.max(1, TW - 1), cy = bh / Math.max(1, TH - 1);
    for (let y = 0; y < TH; y++) {
      const wy = by + y * cy;
      for (let x = 0; x < TW; x++) {
        const wx = bx + x * cx;
        t[y * TW + x] = run('sampleArr')(run('field'), wx, wy)
          + 0.02 * Math.sin(wx * 7.1 + 1.3) * Math.cos(wy * 5.7 + 0.4)
          + 0.008 * Math.sin(wx * 23.0) * Math.sin(wy * 19.0);
      }
    }
    return t;
  }

  // Two cases. The first is the reference's own default `state.viz` -- every
  // enhancement slider at 0, which is what `TerrainAppearance::js_reference()`
  // reproduces. The second turns on the four per-tile stages the prologue
  // builds (crest, coast SDF, river SDF, biome-boundary distance) so the
  // golden actually reaches them. `viz.ao` is deliberately left at 0 in BOTH:
  // this port's screen AO is a different algorithm from the reference's
  // (`render.rs`'s tile section, departure 1), the tile samples the grid's,
  // and a golden that asserted the reference's tile-local `aoMul` would be
  // asserting a term this port's own map does not have.
  const CASES = [
    { name: 'reference_defaults', bounds: { x: 1, y: 1, w: 6, h: 6 }, viz: {} },
    { name: 'sdf_and_crest_on', bounds: { x: 0.5, y: 2.25, w: 7.5, h: 5.5 }, viz: { crest: 0.35, sdfCoast: 0.6, sdfRivers: 0.5, sdfBiomes: 0.4 } },
  ];

  const VIZ0 = JSON.parse(JSON.stringify(run('state.viz')));
  const out = [];
  for (const c of CASES) {
    // Restore, then apply. `state.viz` is a property of a lexical `state`, so
    // this has to run INSIDE the context like everything else.
    run(`Object.assign(state.viz, ${JSON.stringify(VIZ0)});`);
    run(`Object.assign(state.viz, ${JSON.stringify(c.viz)});`);
    const readback = {};
    for (const k of Object.keys(c.viz)) readback[k] = run(`state.viz[${JSON.stringify(k)}]`);
    for (const k of Object.keys(c.viz)) {
      if (readback[k] !== c.viz[k]) die(`case ${c.name}: state.viz.${k} read back as ${readback[k]}, not ${c.viz[k]}`);
    }
    const tile = makeTile(c.bounds.x, c.bounds.y, c.bounds.w, c.bounds.h);
    const rgba = run('renderBiomeTileRGBA')(tile, TW, TH, c.bounds);
    if (!rgba || rgba.length !== TW * TH * 4) die(`case ${c.name}: renderBiomeTileRGBA returned ${rgba && rgba.length} bytes, expected ${TW * TH * 4}`);

    // --- SHAPE GATES: refuse to emit a golden that proves nothing -----------
    const distinct = new Set();
    let opaque = 0, belowSea = 0;
    for (let i = 0; i < TW * TH; i++) {
      distinct.add(`${rgba[i * 4]},${rgba[i * 4 + 1]},${rgba[i * 4 + 2]}`);
      if (rgba[i * 4 + 3] === 255) opaque++;
      if (tile[i] < SEA) belowSea++;
    }
    if (distinct.size < 16) die(`case ${c.name}: only ${distinct.size} distinct colours -- the tile is flat, the golden would assert nothing`);
    if (opaque !== TW * TH) die(`case ${c.name}: ${TW * TH - opaque} pixels are not opaque`);
    if (belowSea === 0) die(`case ${c.name}: no tile pixel is below sea level -- the ocean branch is unreachable`);
    if (belowSea === TW * TH) die(`case ${c.name}: every tile pixel is below sea level -- the land branch is unreachable`);
    console.error(`case ${c.name}: ${distinct.size} distinct colours, ${belowSea}/${TW * TH} px below sea level`);

    out.push({ name: c.name, bounds: c.bounds, viz: c.viz, tile: Array.from(tile), rgba: Array.from(rgba), distinct: distinct.size, belowSea });
  }

  // Negative control in the other direction: the two cases must NOT be the
  // same bytes, or one of them is not exercising what it claims to.
  if (out[0].rgba.join(',') === out[1].rgba.join(',')) die('both cases produced identical RGBA -- the viz sliders changed nothing');

  // ------------------------------------------------------------ emit --------
  const f32 = (v) => `${Number(Math.fround(v)).toPrecision(9)}f32`;
  // An integer-valued f64 must still print with its suffix -- `= 800;` is a
  // type error, not an f64, and `SUN_AZ_DEG`/`LAT_N`/`MAP_WIDTH_KM` are all
  // whole numbers in this world. Caught by the first `cargo test`, recorded
  // here so a later re-capture on a different world cannot reintroduce it.
  const f64 = (v) => `${Number.isInteger(v) ? v + '.0' : String(v)}f64`;
  const arr = (name, v, fmt) => `const ${name}: [${fmt === f32 ? 'f32' : 'u8'}; ${v.length}] = [${v.map(fmt).join(', ')}];`;
  const u8 = (v) => `${v}`;

  const P = [];
  // `dead_code` too: the fixture carries the whole provenance (`SEED`,
  // `GW`/`GH`, the state scalars) so a reader can see what world this is,
  // and not every constant has an assertion pointing at it.
  P.push(`#![allow(clippy::excessive_precision, dead_code)]`);
  P.push(`//! GENERATED by \`cartalith-native/tools/tile_biome_capture.js\` -- do not hand-edit.`);
  P.push(`//!`);
  P.push(`//! Golden-parity fixture for \`render::render_biome_tile_rgba\``);
  P.push(`//! (\`LOD_DETAIL_SCOPE.md\` LOD-D1, Ruling K step 1a), captured from`);
  P.push(`//! \`reference/Cartalith Gen1 v2.11.html\` lines ${FN_FIRST}-${FN_LAST} by running the`);
  P.push(`//! real engine headlessly under Node: all four \`<script>\` blocks in one \`vm\``);
  P.push(`//! context behind a DOM stub, \`GW = GH = ${GRID}\`, seed ${SEED}, \`state.world = false\`.`);
  P.push(`//!`);
  P.push(`//! **The same world \`golden_parity_render.rs\` uses**, deliberately, so a`);
  P.push(`//! divergence here is attributable to the tile path and to nothing else.`);
  P.push(`//! ${water}/${GRID * GRID} grid cells are below sea level ${SEA}.`);
  P.push(`//!`);
  P.push(`//! Two cases. \`reference_defaults\` is \`state.viz\` untouched -- every`);
  P.push(`//! enhancement slider at 0, which is what \`TerrainAppearance::js_reference()\``);
  P.push(`//! reproduces. \`sdf_and_crest_on\` turns on the four stages the per-tile`);
  P.push(`//! prologue builds, so the golden reaches them. \`viz.ao\` is 0 in both: see`);
  P.push(`//! departure 1 in \`render.rs\`'s tile section for why asserting the`);
  P.push(`//! reference's tile-local \`aoMul\` would be asserting a term this port's own`);
  P.push(`//! map does not have.`);
  P.push('');
  P.push(`pub const SEED: u64 = ${SEED};`);
  P.push(`pub const GW: usize = ${GRID};`);
  P.push(`pub const GH: usize = ${GRID};`);
  P.push(`pub const SEA_LEVEL: f64 = ${f64(SEA)};`);
  P.push(`pub const MAP_WIDTH_KM: f64 = ${f64(MAP_KM)};`);
  P.push(`pub const LAT_N: f64 = ${f64(LAT_N)};`);
  P.push(`pub const LAT_S: f64 = ${f64(LAT_S)};`);
  P.push(`pub const EXAG: f64 = ${f64(EXAG)};`);
  P.push(`pub const SUN_AZ_DEG: f64 = ${f64(SUN_AZ)};`);
  P.push(`pub const BIO_BLEND: f64 = ${f64(BIO_BLEND)};`);
  P.push(`/// \`riverFlowThresh(GW, GH)\` as the reference itself computed it for this`);
  P.push(`/// world. Asserted against \`cartalith_hydrology::river_flow_thresh\` rather`);
  P.push(`/// than merely used, so the tile's river mask cannot silently diverge.`);
  P.push(`pub const RIVER_FLOW_THRESH: f64 = ${f64(RIVER_THRESH)};`);
  P.push(`pub const TILE_W: usize = ${TW};`);
  P.push(`pub const TILE_H: usize = ${TH};`);
  P.push('');
  P.push(`pub ${arr('FIELD', FIELD, f32)}`);
  P.push(`pub ${arr('TEMPERATURE', TEMP, f32)}`);
  P.push(`pub ${arr('RAINFALL', RAIN, f32)}`);
  P.push(`pub ${arr('FLOW', FLOW, f32)}`);
  for (const c of out) {
    const N = c.name.toUpperCase();
    P.push('');
    P.push(`// --- case \`${c.name}\`: viz ${JSON.stringify(c.viz)}, bounds ${JSON.stringify(c.bounds)}`);
    P.push(`//     ${c.distinct} distinct colours, ${c.belowSea}/${TW * TH} px below sea level`);
    P.push(`pub const ${N}_BOUNDS: [f64; 4] = [${f64(c.bounds.x)}, ${f64(c.bounds.y)}, ${f64(c.bounds.w)}, ${f64(c.bounds.h)}];`);
    P.push(`pub ${arr(`${N}_TILE`, c.tile, f32)}`);
    P.push(`pub ${arr(`${N}_EXPECTED_RGBA`, c.rgba, u8)}`);
  }
  P.push('');
  process.stdout.write(P.join('\n') + '\n');
  console.error('capture OK');
})().catch((e) => die(`${e && e.constructor && e.constructor.name}: ${e && e.message}\n${e && e.stack}`));
