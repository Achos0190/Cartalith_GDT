#!/usr/bin/env node
/* Golden capture for the BLOCK-2 `_um*` adapter -- `URBAN_MORPHOLOGY_SCOPE.md`
 * milestone 17a, the row `OUTSTANDING_WORK.md` §2.1 records as blocked.
 *
 * ============================ why this exists ============================
 *
 * `urban_adapter.rs`'s own header recorded the blocker, and the blocker was
 * WRONG:
 *
 *     "The block-2 `_um*` functions run inside the host app's full civ scope
 *      (`field`, `flowField`, `civWays`, `state`, `_riverNet`,
 *      `currentWaterBodies`), and the capture harness slices *block 4*
 *      (reference lines 28167-31103) as one contiguous unit -- it has no
 *      block-2 fixture, and building one is a real harness effort."
 *
 * The premise that `tools/um_capture.js` could be extended is false --
 * that script is a block-4 slicer end to end, and block 2 needs block 1's
 * primitives (`chamferDist`, `slopeAt`, `gradAt`, `traceRiverPolylines`,
 * `riverFlowThresh`) and block 4's `UME.SITE_WM`/`SITE_HM` as well. But the
 * conclusion -- that no harness can run `_um*` in the host's full scope --
 * is also false, and this file is the counter-example. MEASURED, not argued:
 *
 *     BLOCK1 OK   (2084-14556, 12 473 lines)
 *     BLOCK2 OK   (14563-26720, 12 158 lines)
 *     BLOCK3 OK   (26723-28161,  1 439 lines)
 *     BLOCK4 OK   (28167-31103,  2 937 lines)
 *
 * All four blocks evaluate in a bare `vm` context, in the browser's own
 * order, given ONE thing: a self-similar Proxy standing in for the DOM. The
 * reference's top-level DOM statements then neither throw nor do anything.
 *
 * ======================= why NOT `um_capture.js`'s shape =================
 *
 * `um_capture.js` slices ONE contiguous block and asserts a comment balance
 * plus two boundary lines. Three of its conventions cannot carry over, and
 * each is replaced rather than dropped:
 *
 *  1. *One contiguous slice* -> FOUR contiguous slices, the file's own
 *     `<script>` blocks, in the browser's order. Boundaries are not hardcoded:
 *     `scriptBlocks()` finds every `<script>`/`</script>` line pair and this
 *     file asserts there are exactly four and that block 4's first/last lines
 *     are the two `um_capture.js` already pins. A cherry-picked per-function
 *     slice was rejected outright -- `_umSiteProfile` alone reaches sixteen
 *     other reference functions, and a slice that silently omits one is the
 *     documented failure mode.
 *  2. *A purity assertion* -> impossible, and stated rather than faked.
 *     Block 1 calls `Math.random()` at top level (`state.tect.seed`) and the
 *     blocks are full of `document.`; that is what the DOM stub is for. What
 *     IS asserted instead is that no function under test reads a stubbed DOM
 *     value: `state.tect.seed` is overwritten with a fixed integer before any
 *     capture runs, and the shape gate below refuses any non-finite output.
 *  3. *`vm.createContext({...})` carries the fixture* -> HALF the fixture.
 *     This is the trap `CLAUDE.md` records twice ("host-side assignment
 *     shadowing `let`-declared reference globals"), and it is live here:
 *
 *         function-declared (a global-object property; host-settable)
 *             currentWaterBodies, currentFloodField, currentCarryingCapacity,
 *             currentResourcePotentials, buildBiomeRaster
 *         let/const-declared (a global LEXICAL binding; host-INVISIBLE)
 *             GW, GH, field, flowField, tempField, rainField, civWays,
 *             _riverNet, _fieldGen, state, BIOME_KEYS, UME
 *
 *     `ctx.GW = 96` does nothing at all -- the probe that established this
 *     printed `typeof ctx.GW === 'undefined'` after block 1 declared it. So
 *     every lexical global is assigned by RUNNING an assignment inside the
 *     context, which does reach the shared global lexical environment, and
 *     `assertLexical()` re-reads each one from inside the vm afterwards.
 *
 * ===================== what is fixture and what is under test ============
 *
 * Fed in (these are `um_site_profile`'s explicit Rust parameters, so feeding
 * them is the fixture, exactly as `um_capture.js` feeds `waterCtx`/
 * `terrainCtx`): `field`, `flowField`, `tempField`, `rainField`, `civWays`,
 * `_riverNet`, `GW`, `GH`, `state.*`, and the five `current*()`/`build*()`
 * memos above.
 *
 * Run live, never stubbed -- these are the reference's own computation and
 * the Rust side has its own port of each: `chamferDist` (via
 * `_civCoastDistField`), `slopeAt`, `gradAt`, `traceRiverPolylines` (via
 * `_civRiverPolylines`), `_civPlaceConnectedRoads`, `_civPlaceResourceContext`,
 * `_civPlaceDefensibility`, `_umInferWalls`/`_umWallSpec`, `riverFlowThresh`,
 * and every `_um*` under test.
 *
 * ============================== the fixture ==============================
 *
 * Two synthetic worlds, both built from INTEGER arithmetic and one final
 * division by 1000 (IEEE-exact, so this side and the Rust side agree bit for
 * bit -- `um_capture.js`'s own lesson, learned there after two metric
 * fixtures crushed the town). The second world exists because EVERY
 * resolution-derived radius in this subsystem is a `Math.max(floor, GW/k)`
 * and a small grid reaches only the floor:
 *
 *                             SMALL 96x64/800km   LARGE 448x288/190km
 *   _umSiteProfile   defR      max(4,round(GW/70))=4  =6
 *   _umOreBearing    R         max(2,round(GW/64))=2  =7
 *   _civPlaceResCtx  r         max(3,round(GW/128))=3 =4
 *   _umWaterReachKm            cell arm (12.5 km)     near arm (2.125 km)
 *   _umSiteKind box            5x5 = 25 cells         11x11 = 121 cells
 *
 * so each of those constants is observable in exactly one of the two. The last
 * row is the one that cost a rewrite: see LARGE's own comment below.
 *
 * Usage:  node tools/um_block2_capture.js \
 *           > ../crates/cartalith-civ/tests/golden_parity_urban_adapter.rs
 * Prints the generated Rust on stdout; diagnostics go to stderr.
 * The frozen reference is READ ONLY here and is never written.
 */
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ROOT = path.resolve(__dirname, '..', '..');
const REF = path.join(ROOT, 'reference', 'Cartalith Gen1 v2.10.html');

function die(msg) { console.error('CAPTURE ABORTED: ' + msg); process.exit(1); }

// ------------------------------------------------------- the script blocks --

const lines = fs.readFileSync(REF, 'utf8').split(/\r?\n/);

/** Every `<script>` .. `</script>` pair, as 1-indexed INNER line ranges. */
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

// The two boundary lines `um_capture.js` already pins, re-asserted here so the
// two harnesses cannot drift apart about where block 4 is.
if (!/^\/\* v0\.95: ported from urban-morphology\//.test(lines[BLOCKS[3][0] - 1])) {
  die(`block 4 does not open with the UME header comment; got:\n  ${lines[BLOCKS[3][0] - 1]}`);
}
if (!/module\.exports\s*=\s*UME;\s*$/.test(lines[BLOCKS[3][1] - 1])) {
  die(`block 4 does not end at the CommonJS export; got:\n  ${lines[BLOCKS[3][1] - 1]}`);
}
// Block 2 is the civ block: it must contain the adapter under test and must
// NOT contain block 1's `mulberry32` or block 4's `UME` IIFE.
{
  const b2 = lines.slice(BLOCKS[1][0] - 1, BLOCKS[1][1]).join('\n');
  for (const f of ['_umSiteProfile', '_umOreBearing', '_umHarbourScale',
                   '_umSiteKindFromTerrain', '_umInferAge', '_umWaterReachKm',
                   '_umRayBoxExit', '_umPlaceContext']) {
    if (!new RegExp(`\\nfunction ${f}\\(`).test('\n' + b2)) die(`block 2 does not define ${f}`);
  }
  if (/\nfunction mulberry32\(/.test('\n' + b2)) die('block 2 defines mulberry32; it is block 1\'s');
  if (/\nconst UME = \(\(\) => \{/.test('\n' + b2)) die('block 2 defines UME; it is block 4\'s');
  const opens = (b2.match(/\/\*/g) || []).length, closes = (b2.match(/\*\//g) || []).length;
  if (opens !== closes) die(`block-comment imbalance in block 2: ${opens} '/*' vs ${closes} '*/'`);
}

// ---------------------------------------------------------- the DOM stub ----
// One self-similar Proxy. Every property of it is itself, it is callable and
// constructible, it iterates empty and numifies to 0 -- so the reference's
// top-level `document.getElementById(...).addEventListener(...)` chains run to
// completion and do nothing. Nothing under test reads it: see the header.
function domStub() {
  const f = function () { return P; };
  const P = new Proxy(f, {
    get(_t, k) {
      if (k === Symbol.toPrimitive) return () => 0;
      if (k === Symbol.iterator) return function* () {};
      if (k === 'then') return undefined;          // not a thenable
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
  // NOT node's real `URL`: block 1 calls `URL.createObjectURL(new Blob(...))`
  // at top level, and node's native binding ABORTS THE PROCESS (SIGABRT, not
  // an exception) when handed anything that is not a real Blob.
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

// The one thing every `_um*` needs that only block 4 can answer.
if (run('UME.SITE_WM') !== 1700 || run('UME.SITE_HM') !== 1250) {
  die('UME.SITE_WM/SITE_HM are not 1700/1250 -- block 4 did not evaluate');
}
// And the `let`-vs-context trap, proven live rather than trusted: setting the
// context property must NOT be visible to block-2 code.
ctx.GW = 12345;
if (run('GW') === 12345) die('GW is a context property, not a lexical binding -- rewrite this harness');
delete ctx.GW;

// ---------------------------------------------------------- the fixture -----
// Integer millis, one division by 1000. Reproduced verbatim on the Rust side;
// the generated file carries this same arithmetic as `build_world`.

/* `LARGE.map_width_km` is 190, not 400, and that is a MEASURED choice rather
 * than a round number. `_umSiteKindFromTerrain` samples a `(2r+1)^2` box with
 * `r = round(_umWaterReachKm()/cellKm)`, so `seaFrac` is quantised to `1/box`
 * and a threshold finer than that quantum is unobservable no matter how many
 * probes are swept. At 800 km/96 and at 400 km/448 the box is 5x5 = 25 either
 * way, so the smallest `seaFrac` above `0.02` is `0.04` -- and mutating the
 * `0.02` to `0.03` SURVIVED the whole matrix. At 190 km/448 the cell is
 * 0.4241 km, `r` is 5 and the box is 11x11 = 121:
 *
 *     smallest seaFrac > 0.02   3/121 = 0.02479   (so 0.02 -> 0.03 flips it)
 *     smallest seaFrac > 0.15  19/121 = 0.15702   (so 0.15 -> 0.16 flips it)
 *
 * Nothing else moves: `defR`, the ore radius and the resource radius are all
 * functions of `GW` alone, and the near/cell arm split is unchanged (190/448
 * is still the near arm, 800/96 still the cell arm -- both re-asserted below). */
const SMALL = { id: 'Small', gw: 96, gh: 64, map_width_km: 800, sea: 0.42 };
const LARGE = { id: 'Large', gw: 448, gh: 288, map_width_km: 190, sea: 0.42 };
const WORLDS = [SMALL, LARGE];

/** The nominal shore column; the river's outlet is here. */
const qOf = (gw) => gw >> 2;
/* The shoreline is RAGGED, on purpose. A vertical coast puts a whole 5-cell
 * column of sea in `_umSiteKindFromTerrain`'s 5x5 box or none at all, so
 * `seaFrac` can only be 0, 0.2, 0.4 ... and the `coast` arm (0.02 < seaFrac
 * <= 0.15) is unreachable by construction. `(j*5)%7 - 3` sweeps the shore
 * across seven columns, which is what makes a partial sea count possible. */
const shoreCol = (j, gw) => qOf(gw) + ((j * 5) % 7) - 3;
/** The trunk's centre row at column `i` -- a staircase, so it is not axis-aligned. */
const riverRow = (i, gh) => ((gh >> 1) + ((i / 8) | 0)) % gh;
/** The tributary's centre row -- descends to meet the trunk near `i == gw*5/8`. */
const tribRow = (i, gh) => ((gh >> 3) + ((i * 3) >> 3)) % gh;
/** A triangle wave, so land stays inside `[0,1]` at either grid width. */
const tri = (x) => { const t = x % 1040; return t < 520 ? t : 1040 - t; };

function heightMillis(i, j, gw, gh) {
  const s = shoreCol(j, gw);
  if (i < s) return 200 + ((i * 7) % 100);             // ocean floor, 0.200..0.299
  let v = 460 + tri((i - s) * 3) + ((j * 7) % 13);     // land, 0.460..0.992
  if (Math.abs(j - riverRow(i, gh)) <= 1) v -= 15;     // a shallow valley along the trunk
  if (Math.abs(j - tribRow(i, gh)) <= 1) v -= 9;       // and along the tributary
  return v;                                            // floor 0.436, above sea at 0.42
}

const FLOW_HIGH = 5000;   // asserted above `riverFlowThresh` below

function buildWorld(W) {
  const { gw, gh } = W, n = gw * gh;
  const field = new Float32Array(n), flow = new Float32Array(n);
  const temp = new Float32Array(n), rain = new Float32Array(n);
  const carry = new Float32Array(n), flood = new Float32Array(n);
  const wb = new Uint8Array(n), biome = new Uint8Array(n);
  const order = new Int16Array(n), recv = new Int32Array(n).fill(-1);
  for (let j = 0; j < gh; j++) for (let i = 0; i < gw; i++) {
    const k = j * gw + i;
    field[k] = heightMillis(i, j, gw, gh) / 1000;
    const onTrunk = j === riverRow(i, gh), onTrib = j === tribRow(i, gh);
    if ((onTrunk || onTrib) && i >= qOf(gw)) flow[k] = FLOW_HIGH;
    temp[k] = ((i * 5 + j * 3) % 41) - 6;              // -6..34 degC
    rain[k] = ((i * 3 + j * 11) % 101) / 100;          // 0..1
    carry[k] = ((i * 13 + j * 5) % 61) / 100;          // 0..0.60
    flood[k] = ((i + j) % 17) / 100;                   // 0..0.16
    wb[k] = field[k] < W.sea ? 1 : 0;                  // 1 = ocean; no lakes in this fixture
    // `_umSiteProfile` maps 0 -> 'ocean', 13 -> 'lake', else BIOME_KEYS[b-1].
    biome[k] = wb[k] === 1 ? 0 : (1 + ((i * 2 + j) % 12));
  }
  // The river network: a trunk running WEST to the shore, plus a tributary that
  // joins it. `recv` is the downstream receiver, `-1` at the outlet.
  const put = (i, j, ni, nj, ord) => {
    const k = j * gw + i;
    order[k] = ord;
    recv[k] = (ni < qOf(gw)) ? -1 : (nj * gw + ni);
  };
  for (let i = gw - 1; i >= qOf(gw); i--) put(i, riverRow(i, gh), i - 1, riverRow(i - 1, gh), 3);
  const join = (gw * 5) >> 3;
  for (let i = gw - 1; i > join; i--) {
    const j = tribRow(i, gh), nj = (i - 1 === join) ? riverRow(i - 1, gh) : tribRow(i - 1, gh);
    if (order[j * gw + i]) continue;                   // never overwrite the trunk
    put(i, j, i - 1, nj, 1);
  }
  // Ore: one strong deposit north-east of each probe band, one weak decoy.
  const pots = {};
  for (const k of run('CIV_RESOURCE_KEYS')) pots[k] = new Float32Array(n);
  const ore = (i, j, key, v) => { if (i >= 0 && j >= 0 && i < gw && j < gh) pots[key][j * gw + i] = v; };
  for (let j = 0; j < gh; j++) for (let i = 0; i < gw; i++) {
    pots.timber[j * gw + i] = ((i * 7 + j * 5) % 91) / 100;   // 0..0.90, so `nearby` is non-empty
    pots.clay[j * gw + i] = ((i + j * 3) % 51) / 100;
  }
  return { W, field, flow, temp, rain, carry, flood, wb, biome, order, recv, pots, ore };
}

/** Ways as the reference stores them: `pts` are `[x,y]` ARRAYS, not `{x,y}`. */
function buildWays(W) {
  const { gw, gh } = W, q = qOf(gw);
  const P = (i, j) => [i, j];
  return [
    { pts: [P(q + 6, gh >> 1), P(q + 20, gh >> 1), P(gw - 4, (gh >> 1) + 6)], type: 'highway', km: 120, hidden: false },
    { pts: [P(q + 6, gh >> 1), P(q + 6, 4)], type: 'regional', km: 60, hidden: false },
    { pts: [P(q + 6, gh >> 1), P(2, gh - 3)], type: 'track', km: 40, hidden: false },
    { pts: [P(q + 6, gh >> 1), P(gw - 2, 2)], type: 'road', km: 90, hidden: true },   // hidden: skipped
    { pts: [P(q + 30, 8)], type: 'road', km: 5, hidden: false },                      // <2 pts: skipped
    { pts: [P(gw - 6, gh - 6), P(q + 2, gh - 6)], type: 'regional', km: 70, hidden: false },
  ];
}

// ------------------------------------------------- install one world in the vm --

function install(w) {
  const { W } = w;
  ctx.__fx = {
    field: w.field, flow: w.flow, temp: w.temp, rain: w.rain,
    carry: w.carry, flood: w.flood, wb: w.wb, biome: w.biome,
    order: w.order, recv: w.recv, pots: w.pots, ways: buildWays(W),
  };
  // ASSIGN (not declare) the lexical globals -- see the header. A `let` at the
  // top of this script would create a SECOND binding and shadow nothing.
  run(`
    GW = ${W.gw}; GH = ${W.gh};
    field = __fx.field; flowField = __fx.flow;
    tempField = __fx.temp; rainField = __fx.rain;
    civWays = __fx.ways;
    _riverNet = { order: __fx.order, recv: __fx.recv };
    _fieldGen = (typeof _fieldGen === 'number' ? _fieldGen : 0) + 1;
    state.seaLevel = ${W.sea}; state.mapWidthKm = ${W.map_width_km};
    state.world = false; state.tect.seed = 12345;
    // The five memos. Function declarations ARE global-object properties, so
    // these assignments really do replace what block 2 calls.
    currentWaterBodies = () => __fx.wb;
    currentFloodField = () => __fx.flood;
    currentCarryingCapacity = () => __fx.carry;
    currentResourcePotentials = () => __fx.pots;
    buildBiomeRaster = () => __fx.biome;
  `);
  return assertLexical(W);
}

function assertLexical(W) {
  const got = run(`({gw:GW, gh:GH, fl:field.length, flow:flowField.length, ways:civWays.length,
                    rn:!!(_riverNet&&_riverNet.order), sea:state.seaLevel, mwk:state.mapWidthKm,
                    wb:currentWaterBodies().length, fld:currentFloodField().length,
                    ck:currentCarryingCapacity().length, pot:currentResourcePotentials().copper.length,
                    bio:buildBiomeRaster().length})`);
  const n = W.gw * W.gh;
  for (const [k, want] of [['gw', W.gw], ['gh', W.gh], ['fl', n], ['flow', n], ['wb', n],
                           ['fld', n], ['ck', n], ['pot', n], ['bio', n],
                           ['sea', W.sea], ['mwk', W.map_width_km]]) {
    if (got[k] !== want) die(`${W.id}: lexical global '${k}' is ${got[k]}, wanted ${want} -- the assignment did not reach block 2`);
  }
  if (!got.rn) die(`${W.id}: _riverNet did not take`);
  if (got.ways !== 6) die(`${W.id}: civWays did not take (${got.ways})`);
  const thr = run(`riverFlowThresh(GW,GH)`);
  if (!(thr > 0)) die(`${W.id}: riverFlowThresh is ${thr}`);
  if (!(FLOW_HIGH > thr)) die(`${W.id}: FLOW_HIGH ${FLOW_HIGH} does not exceed riverFlowThresh ${thr}`);
  const polys = run(`_civRiverPolylines()`);
  if (!polys || polys.length < 2) die(`${W.id}: expected >=2 traced stems, got ${polys ? polys.length : 'null'}`);
  const cdt = run(`_civCoastDistField()`);
  if (!cdt || cdt.length !== n) die(`${W.id}: coast DT is ${cdt ? cdt.length : 'null'}, wanted ${n}`);
  let mn = Infinity, mx = -Infinity;
  for (let i = 0; i < n; i++) { if (cdt[i] < mn) mn = cdt[i]; if (cdt[i] > mx) mx = cdt[i]; }
  if (!(mn === 0 && mx > 0)) die(`${W.id}: coast DT is degenerate (min ${mn}, max ${mx}) -- no sea, or all sea`);
  return { thr, polys };
}

// --------------------------------------------------------------- emitters --

function f64bits(x) { const b = Buffer.alloc(8); b.writeDoubleLE(x, 0); return b.readBigUInt64LE(0); }
function rf(x) {
  if (typeof x !== 'number') die(`rf() got a ${typeof x}: ${x}`);
  if (Number.isNaN(x)) return 'f64::NAN';
  if (x === Infinity) return 'f64::INFINITY';
  if (x === -Infinity) return 'f64::NEG_INFINITY';
  return `f64::from_bits(0x${f64bits(x).toString(16).padStart(16, '0')})`;
}
function rs(s) { return JSON.stringify(String(s)); }
function ropt(v, f) { return v === null || v === undefined ? 'None' : `Some(${f(v)})`; }

// ================================ the captures ==============================

const reached = {
  coast: 0, bay: 0, river: 0, riverthrough: 0, landlocked: 0,
  oreSome: 0, oreNone: 0, oreUnderfoot: 0,
  reachCellArm: 0, reachNearArm: 0,
  profileRiver: 0, profileNoRiver: 0, confluence: 0, noConfluence: 0,
  aspectFlat: 0, aspectSloped: 0, roads: 0, noRoads: 0,
  nearbyResources: 0, coastFinite: 0, coastInfinite: 0,
  harbourClampLo: 0, harbourClampHi: 0, harbourMid: 0,
};

// ---- 1. the pure scalar ladder: _umSiteBoxKm / Near / Reach, per world -----

const worldRows = [];
for (const W of WORLDS) {
  const w = buildWorld(W);
  const { thr } = install(w);
  const boxKm = run('_umSiteBoxKm()'), nearKm = run('_umWaterNearKm()'), reachKm = run('_umWaterReachKm()');
  const cellKm = W.map_width_km / W.gw;
  if (reachKm === nearKm) reached.reachNearArm++; else reached.reachCellArm++;
  worldRows.push({ W, w, thr, boxKm, nearKm, reachKm, cellKm });
}
if (reached.reachCellArm < 1 || reached.reachNearArm < 1) {
  die(`_umWaterReachKm's max() has an unreached arm: cell ${reached.reachCellArm}, near ${reached.reachNearArm}`);
}

// ---- 2. _umInferAge -- pure, no world needed -------------------------------

/* `_umInferAge`'s LOWER clamp is unreachable, and that is a finding, not a
 * gap in this matrix. `p = max(1, pop||0)` then `max(1, p/100)`, so the
 * logarithm's argument is >= 1 and the expression's floor is `60` -- the
 * `Math.max(30, ...)` can never fire. Both sides carry the dead constant
 * because the reference does; the gate below asks only for what is reachable,
 * and `deadFloor` records the smallest value the function can actually return.
 * The upper clamp IS reachable: 60+240*log10(p/100) >= 1000 at p >= ~825 404. */
const AGE_POPS = [
  0, 1, 50, 99, 100, 101, 250, 1000, 2500, 6500, 12000,
  46415.888336127786, 46416,            // 60+240*log10(p/100) ~ 900
  821360, 821361,                       // straddles the round-half-up boundary at 999.5
  825403, 825404, 825405,               // straddles the 1000 clamp
  1e6, 1e7, -5, NaN,
];
const ageRows = AGE_POPS.map((p) => ({ pop: p, age: run(`_umInferAge(${JSON.stringify(p)})`) }));
if (!ageRows.some((r) => r.age === 1000)) die('_umInferAge never hit its upper clamp');
const deadFloor = Math.min(...ageRows.map((r) => r.age));
if (ageRows.some((r) => r.age === 30)) die('_umInferAge hit 30 -- the dead-floor finding above is wrong, rewrite it');
if (deadFloor !== 60) die(`_umInferAge's real floor is ${deadFloor}, not 60 -- the header's claim is stale`);
if (new Set(ageRows.map((r) => r.age)).size < 6) die('_umInferAge produced too few distinct ages to be a test');

// ---- 3. _umHarbourScale -- pure -------------------------------------------

const HARB = [
  ['landlocked', 'landlocked', 3000], ['landlockedBigPop', 'landlocked', 200000],
  ['zeroPop', 'coast', 0], ['onePop', 'coast', 1], ['clampLo', 'coast', 500],
  ['refPort', 'coast', 3000], ['exactLoBoundary', 'coast', 3000 * Math.pow(0.6, 2.5)],
  ['bay', 'bay', 12000], ['river', 'river', 9000],
  ['clampHi', 'coast', 3000 * Math.pow(3, 2.5) + 1], ['exactHiBoundary', 'coast', 3000 * Math.pow(3, 2.5)],
  ['huge', 'coast', 1e7],
];
const harbRows = HARB.map(([name, site, pop]) => {
  const v = run(`_umHarbourScale(${JSON.stringify(pop)},${rs(site)})`);
  if (site !== 'landlocked') { if (v === 0.6) reached.harbourClampLo++; else if (v === 3) reached.harbourClampHi++; else reached.harbourMid++; }
  return { name, site, pop, v };
});
if (!reached.harbourClampLo || !reached.harbourClampHi || !reached.harbourMid) {
  die(`_umHarbourScale arms: lo ${reached.harbourClampLo}, hi ${reached.harbourClampHi}, mid ${reached.harbourMid}`);
}

// ---- 4. _umRayBoxExit -- pure, block-2, exercised by _umRouteEnds ----------

const RAYS = [
  ['east', 1, 0], ['west', -1, 0], ['north', 0, -1], ['south', 0, 1],
  ['ne', 1, -1], ['sw', -0.6, 0.8], ['degenerate', 0, 0],
  ['tinyDx', 1e-10, 1], ['tinyBoth', 1e-10, -1e-10],
  ['cornerExact', 1700, 1250],
];
const rayRows = RAYS.map(([name, dx, dy]) => {
  const p = run(`_umRayBoxExit(${dx},${dy},UME.SITE_WM,UME.SITE_HM)`);
  return { name, dx, dy, x: p.x, y: p.y };
});
if (!rayRows.some((r) => r.name === 'degenerate' && Number.isFinite(r.x))) die('_umRayBoxExit degenerate arm is not finite');

// ---- 5/6/7. the world-dependent captures ----------------------------------

const kindRows = [], oreRows = [], profRows = [];

/** Retention caps. The first guarantees every kind the reference answers
 *  appears in the golden; the second guarantees every *distinct `seaFrac`*
 *  the sweep produces appears, which is what pins the two `seaFrac`
 *  thresholds -- see the thinning comment below. A probe is kept if EITHER
 *  cap admits it. */
const KEEP_PER_KIND = 4, KEEP_PER_SEA_FRAC = 1;

for (const { W, w, thr } of worldRows) {
  install(w);
  const q = qOf(W.gw), gh = W.gh, gw = W.gw;
  // Probes chosen to sweep the shoreline and the river, then filtered by what
  // the reference actually answers -- the kinds are MEASURED, never assumed.
  /* The shore zone is swept rather than guessed, and then THINNED. "First four
   * per kind" is NOT enough, and this was measured rather than reasoned:
   * `_umSiteKindFromTerrain` compares `seaFrac` against `0.02` and `0.15`, and
   * with both caps set that way, mutating either constant SURVIVED the whole
   * matrix. Two separate reasons, both real:
   *
   *  1. `seaFrac` is `seaHits/box` and so QUANTISED. At the 5x5 box both
   *     original worlds had, the smallest value above `0.02` is `0.04`, and no
   *     probe anywhere can distinguish `0.02` from `0.03`. That is why LARGE
   *     is 190 km wide -- see its comment above; the box is 11x11 there.
   *  2. Even at 11x11, the probes that pin a threshold are the ones one
   *     quantum above it, and a per-KIND cap keeps whichever four came first.
   *
   * So the retained set is the union of two rules: up to `KEEP_PER_KIND` of
   * each kind (nothing the reference can answer is dropped), and up to
   * `KEEP_PER_SEA_FRAC` of each distinct `seaFrac` VALUE (every reachable
   * quantum is represented, the boundary-adjacent ones included).
   *
   * `seaFracAt` below is used only to CHOOSE probes, never as an expected
   * value: every `kind` in the golden is still the reference's own answer. */
  const cellKm = W.map_width_km / gw;
  const boxR = Math.max(1, Math.round(run('_umWaterReachKm()') / Math.max(1e-6, cellKm)));
  /* `seaHits/n` over the same `(2r+1)^2` box `_umSiteKindFromTerrain` scans --
   * the FRACTION, not the count, and that distinction is load-bearing. The box
   * is clipped at the grid edge, so `n` is 121 only in the interior and 110 or
   * 66 along it; bucketing by COUNT put `(118,0)` (3/66) and `(118,283)`
   * (3/110) in one bucket and dropped the second, which is the ONLY probe in
   * the whole sweep whose `seaFrac` lands in `(0.02, 0.03]`. The `0.02`
   * mutant survived until this keyed on the quotient. */
  const seaFracAt = (px, py) => {
    let sea = 0, n = 0;
    for (let dy = -boxR; dy <= boxR; dy++) {
      const yy = py + dy;
      if (yy < 0 || yy >= gh) continue;
      for (let dx = -boxR; dx <= boxR; dx++) {
        const xx = px + dx;
        if (xx < 0 || xx >= gw) continue;
        n++;
        if (w.field[yy * gw + xx] < W.sea) sea++;
      }
    }
    return n ? sea / n : -1;
  };
  const probes = [];
  for (let i = Math.max(0, q - 6); i <= q + 8; i++) {
    for (let j = 0; j < gh; j++) probes.push([i, j]);   // every row: the shore is ragged
  }
  for (const i of [q + 12, q + 20, gw >> 1, gw - 8, gw - 2]) {
    for (const dj of [-6, -1, 0, 1, 5]) {
      probes.push([i, Math.max(0, Math.min(gh - 1, riverRow(i, gh) + dj))]);
    }
  }
  probes.push([gw - 3, 1], [gw - 3, gh - 2], [q + 20, 2], [gw >> 1, gh - 2], [0, 0], [1, gh >> 1]);
  const seen = new Set(), perKind = {}, perSea = {};
  for (const [px, py] of probes) {
    const key = px + ',' + py;
    if (seen.has(key)) continue;
    seen.add(key);
    const kind = run(`_umSiteKindFromTerrain({x:${px},y:${py}})`);
    reached[kind] = (reached[kind] || 0) + 1;
    perKind[kind] = (perKind[kind] || 0) + 1;
    const sf = seaFracAt(px, py);
    perSea[sf] = (perSea[sf] || 0) + 1;
    if (perKind[kind] <= KEEP_PER_KIND || perSea[sf] <= KEEP_PER_SEA_FRAC) {
      kindRows.push({ world: W.id, px, py, kind });
    }
  }
  console.error(`${W.id} site kinds: ${JSON.stringify(perKind)}` +
                ` (box ${2 * boxR + 1}^2, ${Object.keys(perSea).length} distinct seaFracs)`);

  // --- _umOreBearing. One strong deposit is planted per case, so the answer
  // --- is a known direction rather than whatever the noise floor happens to be.
  const R = Math.max(2, Math.round(gw / 64));
  const base = [q + 12, riverRow(q + 12, gh)];
  const ORE = [
    ['noDeposit', base[0], base[1], 0, null, 0, 0],
    ['underfoot', base[0], base[1], 0, 'iron', 0, 0],
    ['east', base[0], base[1], 0, 'copper', R, 0],
    ['northWest', base[0], base[1], 0, 'gold', -R, -R],
    ['southOne', base[0], base[1], 0, 'salt', 0, 1],
    ['orientQuarterTurn', base[0], base[1], Math.PI / 2, 'tin', R, 0],
    ['orientUndefined', base[0], base[1], undefined, 'tin', 0, -R],
    ['justBelowFloor', base[0], base[1], 0, 'iron', 1, 1],     // value 0.25 exactly: NOT > best
    ['justAboveFloor', base[0], base[1], 0, 'iron', 1, 1],
    ['outsideRadius', base[0], base[1], 0, 'gold', R + 1, 0],
    ['edgeClamped', 1, 1, 0, 'copper', 3, 3],
  ];
  for (const [name, px, py, orient, key, dx, dy] of ORE) {
    for (const k of run('CIV_RESOURCE_KEYS')) w.pots[k].fill(0);
    let v = 0.9;
    if (name === 'justBelowFloor') v = 0.25;
    if (name === 'justAboveFloor') v = 0.25 + Math.pow(2, -24);
    if (key) w.ore(px + dx, py + dy, key, v);
    const got = run(`_umOreBearing({x:${px},y:${py}},${orient === undefined ? 'undefined' : orient})`);
    if (got === null) { reached.oreNone++; if (key && dx === 0 && dy === 0) reached.oreUnderfoot++; }
    else reached.oreSome++;
    oreRows.push({ world: W.id, name, px, py, orient, key, dx, dy, v, got, r: R });
  }
  for (const k of run('CIV_RESOURCE_KEYS')) w.pots[k].fill(0);
  // restore the background potentials the profile reads
  for (let j = 0; j < gh; j++) for (let i = 0; i < gw; i++) {
    w.pots.timber[j * gw + i] = ((i * 7 + j * 5) % 91) / 100;
    w.pots.clay[j * gw + i] = ((i + j * 3) % 51) / 100;
  }
  // one real deposit, so a profile case has ore in `resources` too
  w.ore(q + 14, riverRow(q + 14, gh) + 2, 'iron', 0.8);

  // --- _umSiteProfile -------------------------------------------------------
  const wayHead = [q + 6, gh >> 1];
  const PROF = [
    ['shorelineWalled', q + 1, riverRow(q + 1, gh), 'city', 12000],
    ['riverBankHamlet', q + 12, riverRow(q + 12, gh) + 1, 'hamlet', 200],
    ['onTheTrunk', gw >> 1, riverRow(gw >> 1, gh), 'town', 3000],
    ['roadHub', wayHead[0], wayHead[1], 'town', 4000],
    ['farInland', gw - 3, 1, 'village', 900],
    ['seaCell', 1, gh >> 1, 'village', 400],
    // `capital`, not `fortress`: `_umWallSpec`'s `fortress` rung is the one
    // branch this port cannot reach (`SettlementKind` has no such variant,
    // and `military.rs` says so in `um_wall_spec`'s own doc comment), so a
    // capture of it would be a golden no Rust call could ever answer. Rank 4
    // is what this row is here for, and `capital` is the rank-4 tier.
    ['cornerClamped', gw - 1, gh - 1, 'capital', 2000],
    ['negativeCoords', -4, -4, 'hamlet', 120],
    ['pastTheEdge', gw + 9, gh + 9, 'hamlet', 120],
  ];
  for (const [name, px, py, kind, pop] of PROF) {
    const p = { x: px, y: py, kind, pop };
    const sp = run(`_umSiteProfile(${JSON.stringify(p)})`);
    if (!sp) die(`${W.id}/${name}: _umSiteProfile returned null on a world that has a field`);
    if (!Number.isFinite(sp.elevation) || !Number.isFinite(sp.buildableFrac)) {
      die(`${W.id}/${name}: profile has a non-finite scalar`);
    }
    if (sp.riverOrder > 0) reached.profileRiver++; else reached.profileNoRiver++;
    if (sp.confluence) reached.confluence++; else reached.noConfluence++;
    if (sp.aspect === null) reached.aspectFlat++; else reached.aspectSloped++;
    if (sp.roadCount > 0) reached.roads++; else reached.noRoads++;
    if (sp.resourcesNearby.length) reached.nearbyResources++;
    if (Number.isFinite(sp.coastDistKm)) reached.coastFinite++; else reached.coastInfinite++;
    profRows.push({ world: W.id, name, px, py, kind, pop, sp,
                    walled: run(`_umInferWalls(${JSON.stringify(p)})`) });
  }
}

// ------------------------------------------------------ the matrix-wide gate --
for (const [k, min] of Object.entries({
  coast: 1, river: 1, landlocked: 1,
  oreSome: 2, oreNone: 2, oreUnderfoot: 1,
  reachCellArm: 1, reachNearArm: 1,
  profileRiver: 1, profileNoRiver: 1, aspectSloped: 1, roads: 1, noRoads: 1,
  nearbyResources: 1, coastFinite: 1,
  harbourClampLo: 1, harbourClampHi: 1, harbourMid: 1,
  // `bay`, `riverthrough` and `confluence` are NOT required. Stated rather
  // than faked: whether this fixture's coastline reaches them is measured
  // below and printed, and the generated file says which were reached.
})) {
  if ((reached[k] || 0) < min) die(`the matrix never reached '${k}' (${reached[k] || 0})`);
}
console.error('arms reached: ' + JSON.stringify(reached));

// =============================== emit the Rust ==============================

const rustWorld = (r) => `    World {
        id: ${rs(r.W.id)},
        gw: ${r.W.gw},
        gh: ${r.W.gh},
        map_width_km: ${rf(r.W.map_width_km)},
        sea_level: ${rf(r.W.sea)},
        flow_thresh: ${rf(r.thr)},
        cell_km: ${rf(r.cellKm)},
        site_box_km: ${rf(r.boxKm)},
        water_near_km: ${rf(r.nearKm)},
        water_reach_km: ${rf(r.reachKm)},
    },`;

const rustAge = (r) => `    (${rf(r.pop)}, ${rf(r.age)}),`;
const rustHarb = (r) => `    HarbourCase { name: ${rs(r.name)}, site: ${rs(r.site)}, pop: ${rf(r.pop)}, scale: ${rf(r.v)} },`;
const rustRay = (r) => `    RayCase { name: ${rs(r.name)}, dx: ${rf(r.dx)}, dy: ${rf(r.dy)}, x: ${rf(r.x)}, y: ${rf(r.y)} },`;
const rustKind = (r) => `    KindCase { world: ${rs(r.world)}, px: ${rf(r.px)}, py: ${rf(r.py)}, kind: ${rs(r.kind)} },`;
const rustOre = (r) => `    OreCase { world: ${rs(r.world)}, name: ${rs(r.name)}, px: ${rf(r.px)}, py: ${rf(r.py)}, orient: ${rf(r.orient === undefined ? 0 : r.orient)}, key: ${ropt(r.key, rs)}, dx: ${r.dx}, dy: ${r.dy}, v: ${rf(r.v)}, r: ${r.r}, bearing: ${ropt(r.got, rf)} },`;

const rustProf = (r) => {
  const s = r.sp;
  const mean = s.resources
    ? '&[' + Object.keys(s.resources).map((k) => `(${rs(k)}, ${rf(s.resources[k])})`).join(', ') + ']'
    : '&[]';
  return `    ProfileCase {
        world: ${rs(r.world)},
        name: ${rs(r.name)},
        px: ${rf(r.px)}, py: ${rf(r.py)},
        kind: ${rs(r.kind)}, pop: ${rf(r.pop)}, walled: ${r.walled},
        site_kind: ${rs(s.siteKind)},
        elevation: ${rf(s.elevation)},
        elev_n: ${rf(s.elevN)},
        slope_n: ${rf(s.slopeN)},
        aspect: ${ropt(s.aspect, rf)},
        local_relief: ${rf(s.localRelief)},
        visibility: ${rf(s.visibility)},
        coast_dist_km: ${rf(s.coastDistKm)},
        river_dist_km: ${rf(s.riverDistKm)},
        river_order: ${rf(s.riverOrder)},
        river_width_m: ${rf(s.riverWidthM)},
        confluence: ${s.confluence},
        floodplain: ${rf(s.floodplain)},
        road_count: ${s.roadCount},
        road_types: &[${s.roadTypes.map(rs).join(', ')}],
        resources: ${mean},
        resources_nearby: &[${s.resourcesNearby.map(rs).join(', ')}],
        biome: ${ropt(s.biome, rs)},
        temp_c: ${rf(s.tempC)},
        rain: ${rf(s.rain)},
        carry_k: ${rf(s.carryK)},
        defensibility: ${rf(s.defensibility)},
        buildable_frac: ${rf(s.buildableFrac)},
    },`;
};

/* The silently-empty-golden gate, on THIS side of the wire. Four subsystems in
 * this port shipped tests that passed over nothing, so nothing is written
 * unless every row array has content -- an emitter that produced `&[]` would
 * otherwise generate a file whose every `for c in ...` loop is a no-op and
 * whose test run reports a healthy `ok`. The generated file re-asserts the
 * same counts (`the_fixture_is_not_degenerate`), so a hand-edit that empties
 * one there fails too. */
for (const [name, rows, min] of [
  ['worldRows', worldRows, 2], ['ageRows', ageRows, 6], ['harbRows', harbRows, 6],
  ['rayRows', rayRows, 6], ['kindRows', kindRows, 10], ['oreRows', oreRows, 8],
  ['profRows', profRows, 8],
]) {
  if (!Array.isArray(rows) || rows.length < min) {
    die(`${name} has ${rows ? rows.length : 'no'} rows, wanted >= ${min} -- refusing to write an empty golden`);
  }
  for (const r of rows) {
    if (r === null || r === undefined || typeof r !== 'object') die(`${name} holds a ${typeof r}`);
  }
}
// Every captured site kind must be one the Rust side can answer, and every
// captured profile must carry the full resource map rather than an empty one.
for (const r of kindRows) {
  if (!['coast', 'bay', 'river', 'riverthrough', 'landlocked'].includes(r.kind)) {
    die(`kindRows holds an unknown site kind ${JSON.stringify(r.kind)}`);
  }
}
for (const r of profRows) {
  const n = r.sp.resources ? Object.keys(r.sp.resources).length : 0;
  if (n !== run('CIV_RESOURCE_KEYS').length) {
    die(`${r.world}/${r.name}: resources map has ${n} keys, wanted ${run('CIV_RESOURCE_KEYS').length}`);
  }
  if (typeof r.sp.siteKind !== 'string' || !r.sp.siteKind) die(`${r.world}/${r.name}: empty siteKind`);
}

const HEADER = require('./um_block2_golden_header.js')({
  arms: JSON.stringify(reached),
  worlds: worldRows.map((r) => r.W),
  nProf: profRows.length, nKind: kindRows.length, nOre: oreRows.length,
  nAge: ageRows.length, nHarb: harbRows.length, nRay: rayRows.length,
});

process.stdout.write(
  HEADER +
  `pub const WORLDS: &[World] = &[\n${worldRows.map(rustWorld).join('\n')}\n];\n\n` +
  `/// \`(pop, _umInferAge(pop))\`.\npub const AGES: &[(f64, f64)] = &[\n${ageRows.map(rustAge).join('\n')}\n];\n\n` +
  `pub const HARBOURS: &[HarbourCase] = &[\n${harbRows.map(rustHarb).join('\n')}\n];\n\n` +
  `pub const RAYS: &[RayCase] = &[\n${rayRows.map(rustRay).join('\n')}\n];\n\n` +
  `pub const KINDS: &[KindCase] = &[\n${kindRows.map(rustKind).join('\n')}\n];\n\n` +
  `pub const ORES: &[OreCase] = &[\n${oreRows.map(rustOre).join('\n')}\n];\n\n` +
  `pub const PROFILES: &[ProfileCase] = &[\n${profRows.map(rustProf).join('\n')}\n];\n`
);
console.error(`wrote ${worldRows.length} worlds, ${ageRows.length} ages, ${harbRows.length} harbours, ` +
              `${rayRows.length} rays, ${kindRows.length} kinds, ${oreRows.length} ores, ${profRows.length} profiles`);
