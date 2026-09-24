#!/usr/bin/env node
/* Fixture writer for `OUTSTANDING_WORK.md` §2.11's "Import legacy flat `.zip`
 * settlements, labels and icons" row (owner Ruling AU, 2026-09-24).
 *
 * WHY THIS EXISTS. The repository's one genuine HTML-app export,
 * `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`, carries
 * `state.places`, `state.labels` and `state.mapIcons` as EMPTY arrays and
 * `state.civ` as `null` -- it was exported straight after `generate()`, so it
 * cannot test an importer of those records at all. This script produces a
 * second export that does carry them, from the real reference engine rather
 * than from hand-typed JSON, so the record shapes are the ones the HTML app
 * actually writes.
 *
 * HOW. The loader is `tile_biome_capture.js`'s (all four `<script>` blocks of
 * `reference/Cartalith Gen1 v2.11.html` in one bare `vm` context behind a
 * self-similar DOM Proxy), with its three traps. Then, in order, every one of
 * them the reference's OWN code path unless marked "literal":
 *
 *   1. `generate()` on a 96 x 61 region world (61 = round(96 * 0.64), the
 *      reference's own `gridH` for `world = false`), seed 24601.
 *   2. `_civIterativeAutoWorld(3)` -- the Auto-populate button
 *      (`_civAutoWorld`, reference 22009). Its tier-count inputs read `''`, the
 *      "automatic" answer, so it places what it would for a user who typed
 *      nothing.
 *   3. `_civDropPlace(x, y)` -- the Settlement tool's click (reference 16534).
 *   4. `_civDropPOI(x, y)` -- the POI tool's click (reference 16558), kind
 *      `shrine` from the `civPoiKind` select.
 *   5. literal: `state.places.push({x:gx,y:gy})` -- the "designate places"
 *      click (reference 9801), at the fractional coordinates `evtToGrid`
 *      returns there (it does not round; the civ tools' handler does).
 *   6. literal + `_civSelectLabel(lb)`: the Label click (reference 9810), twice,
 *      each then passed through `_civPopulateLabelEditor` (17291), which is
 *      what fills a selected label's `font`/`color`/`sizeMode` in the app, and
 *      the second then edited the way the label editor writes
 *      (`sizeMode = 'fixed'`, reference 17308).
 *
 * The map redraw and the side-panel list renderers are replaced with no-ops
 * after step 2 (see the note at that point): they write no record, and one of
 * them never returns over the DOM Proxy.
 *   7. literal + `_carSelectIcon(icon)`: the Icon click (reference 9822), once
 *      per object-literal shape it has (a pack family, and `custom` with a
 *      `set`), plus one at a brush scale (reference 15560's shape).
 *   8. faction 2 renamed through the faction editor's own assignment
 *      (reference 16786), then `_civPaintTerritoryAt` twice -- the Territory tool (reference 16447),
 *      for factions 2 and 3.
 *   9. `_civSyncToState()` -- what the `exportZip` wrapper (reference 26709)
 *      runs first -- then the first seven entries of `exportZip` itself
 *      (reference 12466-12489: params.json, the five `.f32` fields,
 *      `heightmap_rg16.bin`, `strahler_order.bin`), zipped by the reference's
 *      own `zipStore` over Node's real `CompressionStream`.
 *
 * The frozen reference is READ ONLY here.
 *
 * Usage (from `cartalith-native/`):
 *   node tools/legacy_records_capture.js \
 *     crates/cartalith-io/tests/fixtures/legacy_records_seed24601.zip
 */
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ROOT = path.resolve(__dirname, '..', '..');
const REF = path.join(ROOT, 'reference', 'Cartalith Gen1 v2.11.html');
const OUT = process.argv[2];

function die(msg) { console.error('CAPTURE ABORTED: ' + msg); process.exit(1); }
if (!OUT) die('usage: node tools/legacy_records_capture.js <out.zip>');

const lines = fs.readFileSync(REF, 'utf8').split(/\r?\n/);
function scriptBlocks() {
  const out = [];
  let open = -1;
  for (let k = 0; k < lines.length; k++) {
    const t = lines[k].trim();
    if (t === '<script>') { if (open >= 0) die(`nested <script> at line ${k + 1}`); open = k + 2; }
    else if (t === '</script>') { if (open < 0) die(`</script> with no opener at line ${k + 1}`); out.push([open, k]); open = -1; }
  }
  if (open >= 0) die('unclosed <script>');
  return out;
}
const BLOCKS = scriptBlocks();
if (BLOCKS.length !== 4) die(`expected 4 <script> blocks, found ${BLOCKS.length}`);

// The citations above, checked against the file rather than trusted.
const CITE = {
  9801: 'state.places.push({x:gx,y:gy})',
  9810: 'const lb={x:gx,y:gy,name:name.trim(),angle:0,arc:0,size:16}; state.labels.push(lb); _civSelectLabel(lb);',
  9822: 'state.mapIcons.push(icon); _carSelectIcon(icon);',
  16534: 'function _civDropPlace(gx,gy){',
  22009: 'function _civAutoWorld(){',
  26633: 'function _civSyncToState(){',
};
for (const [ln, frag] of Object.entries(CITE)) {
  if (!lines[ln - 1].includes(frag)) die(`reference line ${ln} does not contain \`${frag}\`; got:\n  ${lines[ln - 1].trim().slice(0, 200)}`);
}

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
  matchMedia: () => D, alert: (m) => { console.error('reference alert(): ' + m); }, prompt: () => null, confirm: () => false,
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

ctx.GW = 12345;
if (run('GW') === 12345) die('GW is a context property, not a lexical binding -- rewrite this harness');
delete ctx.GW;

// The inputs the reference reads through `document.getElementById(...).value`.
// Blank tier counts are the Auto-populate panel's "automatic"; a DOM-stub
// Proxy would read as 0 for all five and the placer would refuse ("All
// settlement counts are 0").
const VALUES = { civNCap: '', civNCity: '', civNTown: '', civNVil: '', civNHam: '', civPoiKind: 'shrine' };
ctx.document = new Proxy(D, {
  get(t, k) {
    if (k === 'getElementById') return (id) => (id in VALUES ? { value: VALUES[id] } : D);
    return t[k];
  },
});

const SEED = 24601, W = 96, H = Math.round(W * 0.64);
run(`state.tect.seed=${SEED}; state.seed=${SEED}; state.world=false; state.resW=${W}; GW=${W}; GH=${H}; allocate();`);
if (run('GW') !== W || run('GH') !== H) die(`GW/GH did not take: ${run('GW')}x${run('GH')}`);

(async () => {
  console.error('generate()...');
  await run('generate()');
  console.error('generate() done');
  if (run('field.length') !== W * H) die('field has the wrong length after generate()');

  run('_civAutoWorld()');
  console.error('auto-populate done');
  const auto = run('state.places.length');

  // The tools below end by redrawing the map and rebuilding the side-panel
  // lists. Those are presentation only -- they write no record -- and over
  // the DOM Proxy one of them never returns (a `while(el.firstChild)` style
  // loop sees a truthy Proxy forever; measured: `_civDropPlace` hung for
  // 15 minutes). Replaced with no-ops AFTER Auto-populate, so everything
  // that does write a record still runs as the reference wrote it.
  // Top-level function declarations are properties of the context's global,
  // so this assignment is what the reference's own call sites resolve to.
  for (const f of ['renderNow', 'drawCivLayerAuto', '_civRenderPlaceEditor', '_civRenderPoiList',
    '_civRenderLabelList', '_carRenderIconList', '_civRenderLabelEditor', '_carRenderIconEditor', '_civRenderInspector']) {
    if (typeof run(`typeof ${f}`) !== 'string' || run(`typeof ${f}`) !== 'function') die(`${f} is not a function in the reference`);
    ctx[f] = () => {};
    if (run(`${f}.toString()`) !== '() => {}') die(`${f} could not be replaced from outside the context`);
  }
  if (auto < 3) die(`Auto-populate placed ${auto} settlements; the fixture needs several`);

  // A land cell, clear of every existing place by `r` cells -- the Settlement
  // tool's own click would otherwise *select* the nearest place (16541).
  const clearLand = (r) => run(`(()=>{ const wb=currentWaterBodies();
    for(let y=3;y<GH-3;y++) for(let x=3;x<GW-3;x++){ const i=y*GW+x;
      if(field[i]<state.seaLevel||(wb&&wb[i]!==0)) continue;
      if(state.places.every(p=>(p.x-x)*(p.x-x)+(p.y-y)*(p.y-y)>${r * r})) return [x,y]; }
    return null; })()`);
  const a = clearLand(9); if (!a) die('no clear land cell for the Settlement tool');
  console.error(`Settlement tool at ${a}`);
  run(`_civDropPlace(${a[0]},${a[1]})`);
  console.error('Settlement tool done');
  const b = clearLand(9); if (!b) die('no clear land cell for the POI tool');
  console.error(`POI tool at ${b}`);
  run(`_civDropPOI(${b[0]},${b[1]})`);
  console.error('POI tool done');
  run(`state.places.push({x:${a[0] + 0.37},y:${a[1] + 2.61}})`);

  run(`{ const lb={x:12.4,y:30.8,name:'Vharen Reach',angle:0,arc:0,size:16}; state.labels.push(lb); _civSelectLabel(lb); _civPopulateLabelEditor(document.createElement('div'),lb,null); }`);
  run(`{ const lb={x:70.25,y:9.5,name:'The Sundering Sea',angle:0,arc:0,size:16}; state.labels.push(lb); _civSelectLabel(lb); _civPopulateLabelEditor(document.createElement('div'),lb,null);
         lb.angle=-12; lb.arc=0.3; lb.size=22; lb.sizeMode='fixed'; }`);
  run(`{ const icon={x:40.5,y:20.25,fam:'poi',slot:'ruin',scale:1}; state.mapIcons.push(icon); _carSelectIcon(icon); }`);
  run(`{ const icon={x:55.75,y:44.5,fam:'custom',set:'my_set',slot:'tower',scale:1}; state.mapIcons.push(icon); _carSelectIcon(icon); }`);
  run(`{ const icon={x:60,y:47,fam:'feature',slot:'mountain',scale:0.8125}; state.mapIcons.push(icon); }`);

  console.error('labels and icons placed');

  // The Territory tool (reference 16447): the reference has no algorithmic
  // territory, so a claim exists only where a user painted one. Two strokes,
  // two factions, at the tool's own radius.
  // The faction editor's name field (reference 16786's `oninput`), so the
  // archive's roster differs from the default table a reader might fall back
  // to -- otherwise a reader that ignored the names would pass.
  run(`civFactionNames[2]='The Kessan March';`);
  run(`_civActiveFaction=2; _civPaintTerritoryAt(20,20); _civActiveFaction=3; _civPaintTerritoryAt(60,30); _civActiveFaction=1;`);
  run('_civSyncToState()');
  console.error('_civSyncToState done');
  run('buildGridFields()');
  console.error('buildGridFields done');

  // --- SHAPE GATES --------------------------------------------------------
  const pk = run('serializeState()');
  const st = pk.state;
  const settle = st.places.filter((p) => p.category === 'settlement' || ['town'].includes(p.kind));
  if (st.places.length !== auto + 3) die(`expected ${auto + 3} places, got ${st.places.length}`);
  if (!st.places.some((p) => p.kind === 'shrine')) die('the POI tool placed nothing');
  if (st.labels.length !== 2 || st.mapIcons.length !== 3) die(`labels ${st.labels.length}, icons ${st.mapIcons.length}`);
  if (!st.civ || !Array.isArray(st.civ.factionNames) || st.civ.factionNames[2] !== 'The Kessan March') die('state.civ carries no renamed faction roster');
  if (!Array.isArray(st.civ.territory) || st.civ.territory.length < 4) die('state.civ.territory carries no painted claim');
  console.error(`places ${st.places.length} (auto ${auto}, settlement-shaped ${settle.length}), labels ${st.labels.length}, icons ${st.mapIcons.length}`);
  console.error(`factions ${st.civ.factionNames.length}, territory pairs ${st.civ.territory.length / 2}, ways ${st.civ.ways.length}, journeys ${st.civ.journeys.length}, timeline ${st.civ.timeline.length}`);
  console.error(`label[0] ${JSON.stringify(st.labels[0])}`);
  console.error(`place members: ${[...new Set(st.places.flatMap((p) => Object.keys(p)))].sort().join(', ')}`);

  // --- the archive, through the reference's own writer ---------------------
  ctx.Blob = Blob; ctx.Response = Response; ctx.CompressionStream = CompressionStream;
  const blob = await run(`(async()=>{ const E=[];
    E.push({name:'params.json',data:new TextEncoder().encode(JSON.stringify(serializeState(),null,2))});
    E.push({name:'heightmap.f32',data:f32bytes(field)}); E.push({name:'temperature.f32',data:f32bytes(tempField)}); E.push({name:'rainfall.f32',data:f32bytes(rainField)});
    E.push({name:'volcanic_field.f32',data:f32bytes(volcanicField)}); E.push({name:'impact_field.f32',data:f32bytes(impactField)});
    E.push({name:'heightmap_rg16.bin',data:packHeight16(field,GW*GH)});
    { const net=buildRiverNetwork(field,flowField,GW,GH,state.seaLevel,{world:state.world,riverDensity:(state.viz&&state.viz.riverDensity)||1}), so=new Uint8Array(GW*GH); for(let i=0;i<so.length;i++){ const o=net.order[i]; so[i]=o>255?255:o; } E.push({name:'strahler_order.bin',data:so}); }
    return await zipStore(E); })()`);
  const bytes = Buffer.from(await blob.arrayBuffer());
  if (bytes.length < 1000) die(`zip is ${bytes.length} bytes`);
  fs.writeFileSync(OUT, bytes);
  console.error(`wrote ${OUT} (${bytes.length} bytes)`);
})().catch((e) => die(`${e && e.constructor && e.constructor.name}: ${e && e.message}\n${e && e.stack}`));
