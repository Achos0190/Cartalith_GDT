#!/usr/bin/env node
/* Golden capture for URBAN_MORPHOLOGY_SCOPE.md milestone 16 -- `generate()` + `hashModel`.
 *
 * RECONSTRUCTED 2026-09-02 from the prose headers of the fifteen committed
 * `src/<module>/tests/golden.rs` files, because the scripts that produced those
 * files were thrown away and survive only as those headers. This one is kept.
 * It follows the convention `URBAN_MORPHOLOGY_SCOPE.md` states under
 * "Verification convention for this subsystem", point for point:
 *
 *   - reference lines 28167-31103 sliced as ONE contiguous block, plus line
 *     2291 (`mulberry32`, which block 4 deliberately does not define) spliced
 *     in ahead of it;
 *   - a block-comment BALANCE assertion on the slice, plus the orphan-close
 *     counter milestone 2's negative control added;
 *   - the two STRUCTURAL assertions that actually pin the boundary -- the
 *     slice's FIRST line is block 4's header-comment opening (milestone 3
 *     tightened this from "contains"), and the slice's LAST line is the
 *     CommonJS export;
 *   - milestone 3's fourth assertion as a live negative control in the other
 *     direction: block 4 must NOT define `mulberry32`;
 *   - evaluation in a bare `vm.runInContext` with no DOM;
 *   - an emptiness / shape gate that refuses to write unless every arm this
 *     matrix claims to reach was actually reached.
 *
 * The frozen file is READ ONLY here and is never written.
 *
 * Usage:  node tools/um_capture.js  [> ../crates/cartalith-urban/src/generate/tests/golden.rs]
 * Prints the generated Rust on stdout; diagnostics go to stderr.
 */
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ROOT = path.resolve(__dirname, '..', '..');
const REF = path.join(ROOT, 'reference', 'Cartalith Gen1 v2.10.html');

const FIRST = 28167; // 1-indexed: block 4's header comment opening
const LAST = 31103;  // 1-indexed: `if(typeof module!=='undefined'...)module.exports=UME;`
const MULBERRY = 2291;

function die(msg) { console.error('CAPTURE ABORTED: ' + msg); process.exit(1); }

// ---------------------------------------------------------------- the slice --

const lines = fs.readFileSync(REF, 'utf8').split(/\r?\n/);

// --- structural assertion 1: the slice's FIRST line is block 4's header ------
const first = lines[FIRST - 1];
if (!/^\/\* v0\.95: ported from urban-morphology\//.test(first)) {
  die(`line ${FIRST} is not block 4's header-comment opening; got:\n  ${first}`);
}
// --- structural assertion 2: the slice's LAST line is the CommonJS export ----
const last = lines[LAST - 1];
if (!/module\.exports\s*=\s*UME;\s*$/.test(last)) {
  die(`line ${LAST} is not the CommonJS export; got:\n  ${last}`);
}
// --- and the IIFE really opens two lines in, and closes two lines up ---------
// The header comment runs seven lines, so the IIFE opens at 28174.
if (!/^const UME = \(\(\) => \{/.test(lines[FIRST + 6])) {
  die(`the UME IIFE does not open at line ${FIRST + 7}; got:\n  ${lines[FIRST + 6]}`);
}
if (lines[LAST - 2].trim() !== '})();') {
  die(`the UME IIFE does not close at line ${LAST - 1}; got:\n  ${lines[LAST - 2]}`);
}

const slice = lines.slice(FIRST - 1, LAST).join('\n');

// --- block-comment balance, plus the orphan-close counter -------------------
// Necessary, not sufficient (milestone 2's negative control found the hole an
// apostrophe in comment prose opens); the two structural assertions above are
// what actually pin the boundary. Both are kept, as the convention says.
{
  const opens = (slice.match(/\/\*/g) || []).length;
  const closes = (slice.match(/\*\//g) || []).length;
  if (opens !== closes) die(`block-comment imbalance in the slice: ${opens} '/*' vs ${closes} '*/'`);
  let depth = 0, orphans = 0;
  const re = /\/\*|\*\//g;
  let m;
  while ((m = re.exec(slice))) {
    if (m[0] === '/*') { depth++; } else { if (depth === 0) orphans++; else depth--; }
  }
  if (orphans !== 0) die(`${orphans} orphan '*/' before any '/*' -- the slice starts inside a comment`);
  if (depth !== 0) die(`the slice ends inside a block comment (depth ${depth})`);
}

// --- negative control: block 4 must NOT define mulberry32 -------------------
if (/function\s+mulberry32\s*\(/.test(slice)) {
  die('block 4 defines mulberry32 -- the whole reason line 2291 is spliced in is that it does not');
}
// --- and no DOM / no wall-clock, so the slice really is pure ----------------
// `\b...\.` rather than a bare substring: block 4's prose is full of "documented
// history" and "docs/07", which a substring test for `document` flags. This is
// the same purity claim `lib.rs`'s header records, re-run rather than trusted.
for (const bad of [/\bdocument\./, /\bwindow\./, /\blocalStorage\b/, /\brequestAnimationFrame\b/,
                   /\bMath\.random\b/, /\bDate\.now\b/, /\bperformance\./, /\bgetElementById\b/]) {
  if (bad.test(slice)) die(`the slice references ${bad}; this engine is supposed to be pure`);
}

const mulberry = lines[MULBERRY - 1];
if (!/^function mulberry32\(a\)\{/.test(mulberry)) {
  die(`line ${MULBERRY} is not mulberry32; got:\n  ${mulberry}`);
}

// ------------------------------------------------------------- evaluate it --

const ctx = vm.createContext({ module: { exports: {} }, console });
vm.runInContext(mulberry + '\n' + slice, ctx, { filename: 'UME-block4.js' });
const UME = ctx.module.exports;
if (!UME || typeof UME.cityGen !== 'function' || typeof UME.hashModel !== 'function') {
  die('the slice did not export cityGen/hashModel');
}
if (UME.SITE_WM !== 1700 || UME.SITE_HM !== 1250) die('SITE_WM/SITE_HM are not 1700/1250');

// ----------------------------------------------------- the synthetic rasters --

// Integer arithmetic only, so both this side and the Rust side compute bit-identical
// values with no libm anywhere: a horizontal river band that steps down twice.
const CELL_M = 22, MW = 78, MH = 57;
const riverRow = (i) => 27 + (i < 26 ? 0 : (i < 52 ? 1 : 2));
function waterCtx(order) {
  const mask = new Array(MW * MH), dt = new Array(MW * MH);
  for (let j = 0; j < MH; j++) for (let i = 0; i < MW; i++) {
    const d = Math.abs(j - riverRow(i));
    mask[j * MW + i] = d <= 1 ? 1 : 0;
    dt[j * MW + i] = d <= 1 ? 0 : d - 1;
  }
  const riverPath = [];
  for (let i = 0; i < MW; i += 6) riverPath.push({ x: (i + 0.5) * CELL_M, y: (riverRow(i) + 0.5) * CELL_M });
  return { mask, dt, mw: MW, mh: MH, cellM: CELL_M, riverPath, riverWidthM: 26,
           riverOrder: order === undefined ? 3 : order, seaLakeCells: 0 };
}
// A gentle integer pyramid, then /1000 -- because `_umTerrainCtx` (reference
// line 22403) samples the host's `field`, which is a NORMALISED 0..1 elevation,
// and `slope()` multiplies its finite difference by 900. Two earlier fixtures
// here were metric and both crushed the town: `(i*i)%37 + ((j*j)%23)*2` (jagged
// at cell scale) and the same pyramid in metres (2 m per 22 m cell -> slope 82,
// so `terrainSuitability`'s `exp(-s^2/0.18)` is ~0.004 EVERYWHERE). Both were
// the reference behaving correctly on a silly fixture, and both produced the
// silently-degenerate output the shape gate below exists to refuse. At /1000
// the gradient is ~9e-5/m, i.e. slope ~0.08 -- the same regime the synthetic
// path's own `0.4 + 0.00008*(Hm-y)` sits in. Division is IEEE-exact, so this
// side and the Rust side agree bit for bit.
function terrainCtx() {
  const grid = new Array(MW * MH);
  for (let j = 0; j < MH; j++) for (let i = 0; i < MW; i++) {
    grid[j * MW + i] = ((i < MW / 2 ? i : MW - i) * 2 + (j < MH / 2 ? j : MH - j)) / 1000;
  }
  return { grid, mw: MW, mh: MH, cellM: CELL_M, hMin: 0, hMax: 106 / 1000 };
}
// Integer-metre polylines, so no rounding enters through the fixture.
const ROUTE_ENDS = [{ x: 0, y: 300 }, { x: 1700, y: 900 }, { x: 800, y: 0 }];
const PRIMARY_PATHS = [
  [{ x: 0, y: 400 }, { x: 600, y: 560 }, { x: 1100, y: 640 }, { x: 1700, y: 720 }],
  [{ x: 850, y: 0 }, { x: 830, y: 500 }, { x: 900, y: 1250 }],
];

// ------------------------------------------------------------- the scenarios --

const SCENARIOS = [
  ['riverDefault',      12345, {}],
  ['riverThroughFort',    777, { site: 'riverthrough', pop: 9000, fortified: true }],
  ['coastHarbourChain',  2026, { site: 'coast', pop: 12000, harbourDefence: 'chain', harbourScale: 1.6 }],
  ['bayFortGenerations',   99, { site: 'bay', pop: 18000, fortified: true, wallGenerations: true, settlementAge: 800 }],
  ['landlockedHamlet',   4242, { site: 'landlocked', pop: 450, walls: false }],
  ['venusRadial',       31337, { culture: 'venus', pop: 8000 }],
  ['venusFortCanal',     8080, { culture: 'venus', site: 'coast', pop: 16000, fortified: true }],
  ['venusThroughBridges', 606, { culture: 'venus', site: 'riverthrough', pop: 6000, walls: false }],
  ['ruinedTerrainAware',  515, { pop: 6000, ruined: true, terrainAware: true, terrain: terrainCtx() }],
  ['realWaterRiver',    60606, { pop: 7000, water: waterCtx() }],
  ['realWaterThroughFort', 60607, { site: 'riverthrough', pop: 11000, fortified: true, water: waterCtx(), terrain: terrainCtx() }],
  ['faithNoneBasilica',  1111, { faith: 'none', civicStyle: 'basilica' }],
  ['miningOreYard',      2222, { pop: 9500, economy: { specialisation: 'mining', oreBearing: 0.75 } }],
  ['hostRoads',          3333, { routeEnds: ROUTE_ENDS, primaryPaths: PRIMARY_PATHS }],
  // The one rules patch. `deadEndBias` is the important field: both live
  // profiles set 0 and so does `DEFAULT_RULES`, so without this `privatizeAlleys`
  // returns at its first line in every other scenario and is never exercised.
  // `carryingCapacityWeight: 0` pins `logisticRamp`'s capacity factor to 1.
  ['rulesPatched',       4444, { rules: {
      street: { segmentLengthMedian: 46, deadEndBias: 0.18, parallelStreetSpacing: 31 },
      parcels: { plotDepthVariance: 0.4, subdivisionCap: 4 },
      settlement: { carryingCapacityWeight: 0 } } }],
  // No open water and a stem below Strahler 3: `buildHarbour` stamps
  // `site.harbourInvalid = 'unnavigable'` and returns null.
  ['unnavigableStem',   70707, { pop: 5000, water: waterCtx(1) }],
  // The convention's quantised/symmetric fixture: pop lands EXACTLY on fortMin,
  // so `popTarget >= fortMin` is tested at its boundary rather than past it.
  ['fortMinBoundary',    1000, { pop: 2500, fortified: true }],
  ['epochsOne',          5555, { epochs: 1 }],
  ['wallStyleBastioned', 6666, { pop: 14000, fortified: true, wallStyle: 'bastioned', harbourDefence: 'seawall', site: 'coast' }],

  // ---- the five boundary fixtures the first mutation sweep asked for -------
  // Each exists because a specific mutant SURVIVED the matrix above, and each
  // is quantised or sits exactly on a threshold, as the convention asks.
  //
  // `Math.max(400, ...)`: nothing else in the matrix is below 450.
  ['popFloorClamp',      8123, { pop: 100 }],
  // `Math.round((pop/5.2)/500)` lands on EXACTLY 2.5 at pop 6500, so the divisor
  // is observable here and nowhere else: 500 rounds up to 3 churches, 501 down
  // to 2. This is the quantised fixture the convention asks each milestone for.
  ['churchRoundBoundary', 6500, { pop: 6500 }],
  // `popTarget < 600` at exactly 600: NOT a hamlet, so churches are built.
  ['hamletBoundary',     6001, { pop: 600 }],
  // `Math.max(30, ...)`: nothing else is below 300.
  ['ageFloorClamp',      9090, { pop: 9000, wallGenerations: true, settlementAge: 10 }],
  // Real water AND a dead-end bias, so `privatizeAlleys` can kill an edge a
  // river crossing sits on -- which is what makes `detectRiverCrossings`'
  // position AFTER it observable at all.
  ['realWaterPrivatized', 5252, { pop: 9000, water: waterCtx(),
      rules: { street: { deadEndBias: 0.38 } } }],
  // `applyStarFort`: `wet = minWaterD<175 || opts.wetMoat` and
  // `canalFed = !(minWaterD<175) && opts.wetMoat`. EVERY other fortified
  // scenario sits on water, so its ditch is already wet and `wetMoat` is
  // invisible -- three mutants survived the second sweep on exactly that. A
  // LANDLOCKED fort is the only place the flag can be seen.
  ['landlockedFortDry',  3131, { site: 'landlocked', pop: 12000, fortified: true }],
  ['venusLandlockedCanal', 4747, { culture: 'venus', site: 'landlocked', pop: 14000, fortified: true }],
  // `opts.routeEnds` with NO `primaryPaths`, so `buildPrimaries` -- the only
  // reader of `site.routeEnds` -- actually runs against the override.
  ['hostRouteEndsOnly',  2727, { routeEnds: ROUTE_ENDS }],
  // A small radial town, to try to make the radial `buildPlaza` position
  // observable through `builtMassHull`.
  ['venusSmall',          818, { culture: 'venus', pop: 1800 }],
  // `buildWaterway` clamps its radius to the box edge (`edgeR = min(cx, cy,
  // Wm-cx, Hm-cy) - 12`), and in EVERY larger Venus town `maxRF*0.95` is past
  // that clamp -- so the 0.95 is invisible and a mutation to it survived twice.
  // At pop 1000, `maxRF*0.95` is 262 against an `edgeR` of 309: unclamped, and
  // the multiplier is finally the thing that decides the radius.
  ['venusTinyCanal',     1414, { culture: 'venus', pop: 1000 }],
];

// --------------------------------------------------------------- capture it --

function f64bits(x) { const b = Buffer.alloc(8); b.writeDoubleLE(x, 0); return b.readBigUInt64LE(0); }
function rf(x) {
  if (Number.isNaN(x)) return 'f64::NAN';
  if (x === Infinity) return 'f64::INFINITY';
  if (x === -Infinity) return 'f64::NEG_INFINITY';
  return `f64::from_bits(0x${f64bits(x).toString(16).padStart(16, '0')})`;
}
function rs(s) { return JSON.stringify(String(s)); }
function ropt(v, f) { return v === null || v === undefined ? 'None' : `Some(${f(v)})`; }
function rpt(p) { return `(${rf(p.x)}, ${rf(p.y)})`; }

/* `hashModel` does not hash `details` at all, so without this the whole
 * hinterland/canal/clutter half of the model is unobserved by the golden --
 * a mutation to `buildWaterway`'s radius survived the first sweep for exactly
 * that reason. Min/max over every point of every detail: exact, cheap, and
 * general enough to catch a geometry change in any detail kind. */
function waterwayPts(details) { const w = details.find(d => d.kind === 'waterway'); return w ? w.poly.length : 0; }
function waterwayFirst(details) { const w = details.find(d => d.kind === 'waterway'); return w ? w.poly[0] : null; }
function detailBBox(details) {
  let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  const put = (p) => { if (p.x < x0) x0 = p.x; if (p.y < y0) y0 = p.y; if (p.x > x1) x1 = p.x; if (p.y > y1) y1 = p.y; };
  for (const d of details) {
    if (d.x !== undefined) put(d);
    if (d.a) { put(d.a); put(d.b); }
    if (d.poly) for (const q of d.poly) put(q);
  }
  return [x0, y0, x1, y1];
}

const rows = [];
const reached = {
  organic: 0, radial: 0, walled: 0, unwalled: 0, starFort: 0, harbour: 0, noHarbour: 0,
  bridges: 0, ford: 0, markets: 0, noMarkets: 0, civic: 0, noCivic: 0, games: 0,
  churches: 0, noChurches: 0, ruined: 0, waterway: 0, cleared: 0, harbourInvalid: 0,
};

for (const [name, seed, opts] of SCENARIOS) {
  const m = UME.cityGen(seed, opts);
  const hash = UME.hashModel(m);
  const live = m.graph.edges; // already filtered to alive
  const districts = {};
  for (const p of m.parcels) districts[p.district || ''] = (districts[p.district || ''] || 0) + 1;
  const detailKinds = {};
  for (const d of m.details) detailKinds[d.kind] = (detailKinds[d.kind] || 0) + 1;

  if (m.culture === 'venus') reached.radial++; else reached.organic++;
  if (m.wall.ring) reached.walled++; else reached.unwalled++;
  if (m.wall.fort) reached.starFort++;
  if (m.harbour) reached.harbour++; else reached.noHarbour++;
  if (m.site.bridges) reached.bridges++;
  if (m.site.ford) reached.ford++;
  if (m.markets.length) reached.markets++; else reached.noMarkets++;
  if (m.civic) reached.civic++; else reached.noCivic++;
  if (m.games.length) reached.games++;
  if (m.churches.length) reached.churches++; else reached.noChurches++;
  if (m.parcels.some(p => p.ruined)) reached.ruined++;
  if (detailKinds.waterway) reached.waterway++;
  if (m.parcels.some(p => p.cleared)) reached.cleared++;
  if (m.site.harbourInvalid) reached.harbourInvalid++;

  // A per-scenario shape gate: nothing below may be silently empty.
  if (!m.graph.nodes.length || !live.length) die(`${name}: empty graph`);
  if (!m.blocks.length) die(`${name}: no blocks`);
  if (!m.parcels.length) die(`${name}: no parcels`);
  if (!m.details.length) die(`${name}: no details`);
  if (!Number.isFinite(m.pop)) die(`${name}: pop is ${m.pop}`);

  const firstE = live[0], lastE = live[live.length - 1];
  const firstN = m.graph.nodes[0], lastN = m.graph.nodes[m.graph.nodes.length - 1];
  const firstP = m.parcels[0], lastP = m.parcels[m.parcels.length - 1];

  rows.push(`    Case {
        name: ${rs(name)},
        // --- fixture inputs: opts verbatim, so the Rust side cannot drift ---
        seed: ${seed},
        o_culture: ${rs(opts.culture || '')},
        o_site: ${rs(opts.site || '')},
        o_pop: ${ropt(opts.pop, rf)},
        o_epochs: ${ropt(opts.epochs, String)},
        o_settlement_age: ${ropt(opts.settlementAge, rf)},
        o_walls: ${ropt(opts.walls, String)},
        o_fortified: ${!!opts.fortified},
        o_terrain_aware: ${!!opts.terrainAware},
        o_ruined: ${!!opts.ruined},
        o_wall_generations: ${!!opts.wallGenerations},
        o_wall_style: ${rs(opts.wallStyle || '')},
        o_faith: ${rs(opts.faith || '')},
        o_civic_style: ${rs(opts.civicStyle || '')},
        o_harbour_defence: ${rs(opts.harbourDefence || '')},
        o_harbour_scale: ${ropt(opts.harbourScale, rf)},
        o_water_order: ${opts.water ? ropt(opts.water.riverOrder, rf) : 'None'},
        o_terrain: ${!!opts.terrain},
        o_economy: ${rs((opts.economy && opts.economy.specialisation) || '')},
        o_ore_bearing: ${opts.economy ? ropt(opts.economy.oreBearing, rf) : 'None'},
        o_route_ends: ${!!opts.routeEnds},
        o_primary_paths: ${!!opts.primaryPaths},
        o_rules: ${opts.rules ? (opts.rules.parcels ? 1 : 2) : 0},
        // --- the reference's own whole-model hash ---
        hash: ${hash >>> 0},
        // --- the scalars generate() derives itself ---
        pop_target: ${rf(m.popTarget)},
        settlement_age: ${rf(m.settlementAge)},
        epochs: ${m.epochs},
        walls: ${m.walls},
        fortified: ${m.fortified},
        fort_requested: ${m.fortRequested},
        culture: ${rs(m.culture)},
        site_kind: ${rs(m.site.kind)},
        through: ${m.through},
        pop: ${rf(m.pop)},
        // --- counts ---
        nodes: ${m.graph.nodes.length},
        live_edges: ${live.length},
        blocks: ${m.blocks.length},
        parcels: ${m.parcels.length},
        buildings: ${m.buildings.length},
        churches: ${m.churches.length},
        markets: ${m.markets.length},
        games: ${m.games.length},
        details: ${m.details.length},
        ruined_parcels: ${m.parcels.filter(p => p.ruined).length},
        cleared_parcels: ${m.parcels.filter(p => p.cleared).length},
        district_counts: &[${Object.keys(districts).sort().map(k => `(${rs(k)}, ${districts[k]})`).join(', ')}],
        detail_kinds: &[${Object.keys(detailKinds).sort().map(k => `(${rs(k)}, ${detailKinds[k]})`).join(', ')}],
        // --- the stages whose presence is a branch ---
        has_plaza: ${!!m.plaza},
        plaza_center: ${ropt(m.plaza, p => rpt(p.center))},
        has_harbour: ${!!m.harbour},
        harbour_invalid: ${ropt(m.site.harbourInvalid, rs)},
        has_civic: ${!!m.civic},
        civic_style: ${ropt(m.civic, c => rs(c.style))},
        wall_ring: ${ropt(m.wall.ring, r => String(r.length))},
        wall_gates: ${m.wall.gates.length},
        wall_style: ${rs(m.wall.style === undefined ? '' : m.wall.style)},
        wall_epoch: ${m.wall.epoch},
        has_fort: ${!!m.wall.fort},
        fort_canal_fed: ${!!(m.wall.fort && m.wall.fort.canalFed)},
        fort_wet_ditch: ${!!(m.wall.fort && m.wall.fort.wetDitch)},
        fort_double_moat: ${!!(m.wall.fort && m.wall.fort.doubleMoat)},
        fort_outer_moat: ${!!(m.wall.fort && m.wall.fort.outerMoat)},
        fort_bastions: ${m.wall.fort ? m.wall.fort.bastions.length : 0},
        fort_trace: ${m.wall.fort ? m.wall.fort.trace.length : 0},
        fort_ravelins: ${m.wall.fort ? m.wall.fort.ravelins.length : 0},
        fort_glacis_off: ${rf(m.wall.fort ? m.wall.fort.glacisOff : 0)},
        wall_generation: ${m.wall.generation || 1},
        wall_history: ${(m.wall.history || []).length},
        detail_bbox: (${detailBBox(m.details).map(rf).join(', ')}),
        route_ends: &[${m.site.routeEnds.map(rpt).join(', ')}],
        waterway_pts: ${waterwayPts(m.details)},
        waterway_first: ${ropt(waterwayFirst(m.details), rpt)},
        bridges: ${ropt(m.site.bridges, b => String(b.length))},
        has_ford: ${!!m.site.ford},
        // --- computeMetrics, a full-precision probe over the FINAL graph ---
        m_nodes: ${m.metrics.nodes},
        m_edges: ${m.metrics.edges},
        m_total_len: ${rf(m.metrics.totalLen)},
        m_dead_end_share: ${rf(m.metrics.deadEndShare)},
        m_deg3_share: ${rf(m.metrics.deg3Share)},
        m_deg4_share: ${rf(m.metrics.deg4Share)},
        m_mean_deg: ${rf(m.metrics.meanDeg)},
        m_median_seg: ${rf(m.metrics.medianSeg)},
        m_meshedness: ${rf(m.metrics.meshedness)},
        m_median_block_area: ${rf(m.metrics.medianBlockArea)},
        m_median_frontage: ${rf(m.metrics.medianFrontage)},
        // --- fully-written anchors, so a hash miss localises ---
        market: ${rpt(m.anchors.market)},
        market_prov: ${rs(m.anchors.prov)},
        first_edge: (${firstE.a}, ${firstE.b}, ${rs(firstE.cls)}, ${rf(firstE.w)}),
        last_edge: (${lastE.a}, ${lastE.b}, ${rs(lastE.cls)}, ${rf(lastE.w)}),
        first_node: ${rpt(firstN)},
        last_node: ${rpt(lastN)},
        first_parcel: (${rs(firstP.id)}, ${rf(firstP.area)}, ${rs(firstP.district || '')}),
        last_parcel: (${rs(lastP.id)}, ${rf(lastP.area)}, ${rs(lastP.district || '')}),
    },`);
}

// ---------------------------------------------------- the matrix-wide gate --
// The convention's "a golden that passes is not a golden that tests anything":
// refuse to write unless every arm this matrix exists to reach was reached.
for (const [k, min] of Object.entries({
  organic: 1, radial: 1, walled: 1, unwalled: 1, starFort: 1, harbour: 1, noHarbour: 1,
  bridges: 1, markets: 1, noMarkets: 1, civic: 1, noCivic: 1, games: 1,
  churches: 1, noChurches: 1, ruined: 1, waterway: 1, cleared: 1, harbourInvalid: 1,
  // `ford` is deliberately NOT required. It needs a through-town whose bridgePt
  // is set and where NO live road crosses the real centreline -- reachable only
  // on a town degenerate enough to fail the per-scenario shape gate above. Stated
  // rather than faked: `Crossings::Ford` is unexercised by this matrix.
})) {
  if (reached[k] < min) die(`the matrix never reached '${k}' (${reached[k]} scenarios)`);
}
console.error('arms reached: ' + JSON.stringify(reached));

// --------------------------------------------------------------- emit Rust --

const header = `//! GENERATED by \`cartalith-native/tools/um_capture.js\` -- do not hand-edit.
//!
//! Milestone 16's whole-subsystem golden: the frozen reference engine's own
//! \`generate()\` output for ${SCENARIOS.length} scenarios, compared against this port's.
//!
//! Every value below came out of \`vm.runInContext\` over reference lines
//! 28167-31103 sliced as one contiguous block, with line 2291 (\`mulberry32\`)
//! spliced in ahead of it -- the convention \`URBAN_MORPHOLOGY_SCOPE.md\` states,
//! with all four of its assertions live: the comment-balance check plus the
//! orphan-close counter, the two structural boundary assertions, and the
//! negative control that block 4 does **not** define \`mulberry32\`. The frozen
//! file is read and never written.
//!
//! \`hash\` is the reference's OWN \`hashModel\` (line 31087). It is coarse by
//! construction -- coordinates to the centimetre, areas to a tenth of a square
//! metre -- so it is not the whole test: the counts, the \`computeMetrics\`
//! readout (full-precision doubles over the FINAL graph, after every pass that
//! can kill an edge) and the written-out anchors are what localise a failure,
//! and they are asserted BEFORE the hash for that reason.
//!
//! The capture refuses to write unless the matrix reaches both planning
//! branches, a walled and an unwalled town, a star fort, a harbour and a
//! harbourless site, real river bridges, a town with markets and one without,
//! a civic hall and none, a games building, churches and none, ruined parcels,
//! the Venus canal, and a swept (\`cleared\`) parcel.

#![allow(clippy::approx_constant, clippy::unreadable_literal, clippy::excessive_precision)]

/// One captured scenario. Field order matches the assertion order in
/// \`super::whole_subsystem_matches_reference\`.
pub struct Case {
    pub name: &'static str,
    // --- fixture inputs. \`""\` / \`None\` is "key absent from \`opts\`". ---
    pub seed: u32,
    pub o_culture: &'static str,
    pub o_site: &'static str,
    pub o_pop: Option<f64>,
    pub o_epochs: Option<i32>,
    pub o_settlement_age: Option<f64>,
    pub o_walls: Option<bool>,
    pub o_fortified: bool,
    pub o_terrain_aware: bool,
    pub o_ruined: bool,
    pub o_wall_generations: bool,
    pub o_wall_style: &'static str,
    pub o_faith: &'static str,
    pub o_civic_style: &'static str,
    pub o_harbour_defence: &'static str,
    pub o_harbour_scale: Option<f64>,
    /// \`Some(order)\` means \`opts.water\` was the synthetic raster below at that
    /// Strahler order; \`None\` means no \`opts.water\` at all.
    pub o_water_order: Option<f64>,
    pub o_terrain: bool,
    pub o_economy: &'static str,
    pub o_ore_bearing: Option<f64>,
    /// \`opts.routeEnds\` — the fixed triple in \`super::route_ends\`.
    pub o_route_ends: bool,
    /// \`opts.primaryPaths\` — the fixed pair in \`super::primary_paths\`.
    pub o_primary_paths: bool,
    /// Which \`opts.rules\` patch: 0 none, 1 \`super::rules_patch\`,
    /// 2 \`super::dead_end_patch\`.
    pub o_rules: u8,
    /// The reference's own \`hashModel\`.
    pub hash: u32,
    pub pop_target: f64,
    pub settlement_age: f64,
    pub epochs: i32,
    pub walls: bool,
    pub fortified: bool,
    pub fort_requested: bool,
    pub culture: &'static str,
    pub site_kind: &'static str,
    pub through: bool,
    pub pop: f64,
    pub nodes: usize,
    pub live_edges: usize,
    pub blocks: usize,
    pub parcels: usize,
    pub buildings: usize,
    pub churches: usize,
    pub markets: usize,
    pub games: usize,
    pub details: usize,
    pub ruined_parcels: usize,
    pub cleared_parcels: usize,
    /// \`(district, count)\`, sorted by district. \`""\` is a parcel
    /// \`assignDistricts\` left untagged.
    pub district_counts: &'static [(&'static str, usize)],
    /// \`(kind, count)\`, sorted by kind.
    pub detail_kinds: &'static [(&'static str, usize)],
    pub has_plaza: bool,
    pub plaza_center: Option<(f64, f64)>,
    pub has_harbour: bool,
    pub harbour_invalid: Option<&'static str>,
    pub has_civic: bool,
    pub civic_style: Option<&'static str>,
    /// Vertex count of \`wallState.ring\`, or \`None\` for an unwalled town.
    pub wall_ring: Option<usize>,
    pub wall_gates: usize,
    pub wall_style: &'static str,
    pub wall_epoch: i32,
    pub has_fort: bool,
    pub fort_canal_fed: bool,
    pub fort_wet_ditch: bool,
    pub fort_double_moat: bool,
    pub fort_outer_moat: bool,
    pub fort_bastions: usize,
    pub fort_trace: usize,
    pub fort_ravelins: usize,
    pub fort_glacis_off: f64,
    pub wall_generation: u32,
    pub wall_history: usize,
    /// \`(minX, minY, maxX, maxY)\` over every point of every detail. \`hashModel\`
    /// hashes no details at all, so this is the only thing watching them.
    pub detail_bbox: (f64, f64, f64, f64),
    /// \`site.routeEnds\` — the only thing that sees the \`opts.routeEnds\`
    /// override on a scenario that also supplies \`primaryPaths\`.
    pub route_ends: &'static [(f64, f64)],
    /// The Venus canal's vertex count, and its first vertex. \`detail_bbox\` does
    /// NOT see the canal: the farmland always reaches further out than it does,
    /// so the bbox is farmland's and a change to the canal radius hides inside it.
    pub waterway_pts: usize,
    pub waterway_first: Option<(f64, f64)>,
    /// \`site.bridges.length\`, or \`None\`.
    pub bridges: Option<usize>,
    pub has_ford: bool,
    pub m_nodes: usize,
    pub m_edges: usize,
    pub m_total_len: f64,
    pub m_dead_end_share: f64,
    pub m_deg3_share: f64,
    pub m_deg4_share: f64,
    pub m_mean_deg: f64,
    pub m_median_seg: f64,
    pub m_meshedness: f64,
    pub m_median_block_area: f64,
    pub m_median_frontage: f64,
    pub market: (f64, f64),
    /// The whole derivation sentence, not its length: JS \`.length\` counts
    /// UTF-16 code units and Rust's counts UTF-8 bytes, and these strings carry
    /// \`§\` and em dashes.
    pub market_prov: &'static str,
    pub first_edge: (usize, usize, &'static str, f64),
    pub last_edge: (usize, usize, &'static str, f64),
    pub first_node: (f64, f64),
    pub last_node: (f64, f64),
    pub first_parcel: (&'static str, f64, &'static str),
    pub last_parcel: (&'static str, f64, &'static str),
}

pub const CASES: &[Case] = &[
`;

process.stdout.write(header + rows.join('\n') + '\n];\n');
console.error(`wrote ${rows.length} cases`);
