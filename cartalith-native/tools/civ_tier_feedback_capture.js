#!/usr/bin/env node
/* Golden capture for `_civIterativeAutoWorld`'s centrality -> tier feedback
 * (`PHASE2_SCOPE.md` m8; `OUTSTANDING_WORK.md` §2.3's wantCounts row found the
 * loop was never ported).
 *
 * Runs the reference's OWN text, sliced out of the frozen v2.11 snapshot and
 * never retyped:
 *   - `function _civNetworkMetrics(places, ways){ ... }` (reference 22421),
 *     located by its declaration and closed by brace matching;
 *   - `const tierOrder=[...]` (25923), one line;
 *   - the promote/demote body inside `if(pass<passes-1){ ... }` (26109-26121),
 *     from `const metrics=` to the brace that closes `for(const m of metrics)`.
 * `_civNetworkMetrics` reads one global, `GW` (its `snapR`, used only for ways
 * without `aIdx`/`bIdx` -- none here), supplied as a parameter.
 *
 * Fixture: synthetic graphs, not worlds. The function under test takes a place
 * list and a way list and nothing else, so a graph is the whole input. Each way
 * is `{pts:[[0,0],[1,1]],aIdx,bIdx}` -- the shape `_civHierarchicalNetwork`
 * emits. The generated Rust feeds the same (aIdx,bIdx) sequence to
 * `civ_network_betweenness` + `civ_centrality_tier_feedback`.
 *
 * Usage:  node tools/civ_tier_feedback_capture.js \
 *           > crates/cartalith-civ/tests/golden_parity_centrality_feedback.rs
 * The frozen reference is READ ONLY here.
 */
'use strict';
const fs = require('fs');
const path = require('path');

const REF = path.join(__dirname, '..', '..', 'reference', 'Cartalith Gen1 v2.11.html');
const src = fs.readFileSync(REF, 'utf8');
function die(m) { console.error('CAPTURE ABORTED: ' + m); process.exit(1); }

function braceBlock(from) {           // text from `from` to its matching close brace
  const open = src.indexOf('{', from);
  let d = 0;
  for (let k = open; k < src.length; k++) {
    if (src[k] === '{') d++;
    else if (src[k] === '}' && --d === 0) return src.slice(from, k + 1);
  }
  die('unbalanced braces');
}
function once(needle) {
  const a = src.indexOf(needle);
  if (a < 0 || src.indexOf(needle, a + 1) >= 0) die(`not exactly one occurrence of: ${needle}`);
  return a;
}

const metricsSrc = braceBlock(once('function _civNetworkMetrics(places, ways){'));
const tierLine = src.slice(once("const tierOrder=['capital','city','town','village','hamlet'];"),
  src.indexOf('\n', once("const tierOrder=['capital','city','town','village','hamlet'];")));
const bodyStart = once('const metrics=_civNetworkMetrics(places,ways);');
const forStart = src.indexOf('for(const m of metrics){', bodyStart);
if (forStart - bodyStart > 200) die('promote/demote loop is not where it was');
const bodySrc = src.slice(bodyStart, forStart) + braceBlock(forStart);
for (const must of ['normB>0.65&&currTier>0', 'normB<0.08&&currTier<tierOrder.length-1',
                    'Math.max(...metrics.map(m=>m.betweenness),1e-9)']) {
  if (!bodySrc.includes(must)) die(`body does not contain ${must}`);
}

// eslint-disable-next-line no-new-func
const step = new Function('places', 'ways', 'GW',
  `${metricsSrc}\n${tierLine}\n${bodySrc}\nreturn _civNetworkMetrics(places,ways).map(m=>m.betweenness);`);
// ^ the trailing call is NOT under test: it only recovers the betweenness the
// body just used (same places/ways, and kind is not an input to the metrics).

// ------------------------------------------------------------- fixtures ----
const KINDS = ['capital', 'city', 'town', 'village', 'hamlet', 'metropolis'];
let seed = 20260923;
const rnd = () => { seed = (Math.imul(seed, 1103515245) + 12345) >>> 0; return seed / 4294967296; };

const cases = [];
const add = (name, kinds, pairs) => cases.push({ name, kinds, pairs });
add('star', ['hamlet', 'capital', 'city', 'town', 'village', 'hamlet'], [[0, 1], [0, 2], [0, 3], [0, 4], [0, 5]]);
add('path', ['town', 'town', 'town', 'town', 'town', 'town'], [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]]);
add('single', ['capital'], []);
add('pair', ['city', 'village'], [[0, 1]]);
add('complete5', ['capital', 'capital', 'capital', 'capital', 'capital'],
  [[0, 1], [0, 2], [0, 3], [0, 4], [1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]);
add('components', ['city', 'town', 'village', 'hamlet', 'capital', 'town', 'metropolis'],
  [[0, 1], [1, 2], [2, 0], [2, 3], [4, 5], [5, 4], [3, 3], [1, 9]]);   // duplicate, self, out-of-range; 6 isolated
for (let g = 0; g < 14; g++) {
  const n = 8 + Math.floor(rnd() * 23);
  const kinds = Array.from({ length: n }, () => KINDS[Math.floor(rnd() * 5.2)]);
  const pairs = [];
  for (let i = 1; i < n; i++) pairs.push([i, Math.floor(rnd() * i)]);          // a spanning tree
  const extra = Math.floor(rnd() * n);
  for (let e = 0; e < extra; e++) pairs.push([Math.floor(rnd() * n), Math.floor(rnd() * n)]);
  for (let e = pairs.length - 1; e > 0; e--) { const j = Math.floor(rnd() * (e + 1)); [pairs[e], pairs[j]] = [pairs[j], pairs[e]]; }
  add(`random${g}`, kinds, pairs);
}

const reached = { up: 0, down: 0, capDown: 0, metroKept: 0, near065: 0, near008: 0 };
const out = [];
for (const c of cases) {
  const places = c.kinds.map(k => ({ kind: k, klass: k, traits: [] }));
  const ways = c.pairs.map(([a, b]) => ({ pts: [[0, 0], [1, 1]], aIdx: a, bIdx: b }));
  const btw = step(places, ways, 256);
  const max = Math.max(...btw, 1e-9);
  c.kinds.forEach((k, i) => {
    const r = btw[i] / max, after = places[i].kind;
    if (KINDS.indexOf(after) < KINDS.indexOf(k)) reached.up++;
    if (KINDS.indexOf(after) > KINDS.indexOf(k) && k !== 'metropolis') reached.down++;
    if (k === 'capital' && after === 'city') reached.capDown++;
    if (k === 'metropolis' && after === 'metropolis') reached.metroKept++;
    if (r > 0.6 && r < 0.7) reached.near065++;
    if (r > 0.04 && r < 0.12) reached.near008++;
  });
  out.push({ ...c, btw, after: places.map(p => p.kind) });
}
for (const [k, v] of Object.entries(reached)) if (!v) die(`fixture never reaches ${k}`);
console.error('reached', JSON.stringify(reached));

// -------------------------------------------------------------- emit Rust --
function bits(x) { const b = Buffer.alloc(8); b.writeDoubleLE(x, 0); return '0x' + b.readBigUInt64LE(0).toString(16).padStart(16, '0'); }
const K = k => 'K::' + k[0].toUpperCase() + k.slice(1);
let rs = `//! GENERATED by \`tools/civ_tier_feedback_capture.js\` -- do not hand-edit.
//!
//! Golden parity for \`_civIterativeAutoWorld\`'s centrality -> tier feedback
//! (reference v2.11 lines 26109-26121) and the \`_civNetworkMetrics\`
//! betweenness it reads (22421-22486): the reference's own source text, sliced
//! from the frozen snapshot and run under Node over ${out.length} synthetic graphs
//! (a star, a path, a single place, a pair, K5, a multi-component graph with a
//! duplicate, a self pair, an out-of-range pair, an isolated metropolis, and
//! 14 seeded random graphs of 8-30 places). Betweenness is compared bit for
//! bit, tiers exactly. The harness refuses to emit unless the fixtures reach a
//! promotion, a demotion, a capital demoted, a metropolis left alone, and a
//! ratio within 0.05 of each threshold.

use cartalith_civ::SettlementKind as K;

fn place(kind: K) -> cartalith_civ::SettlementPlacement {
    cartalith_civ::SettlementPlacement { x: 0, y: 0, suit: 0.0, faction: 1, capital: false, kind, coastal: false }
}

fn check(name: &str, before: &[K], pairs: &[(usize, usize)], btw_bits: &[u64], after: &[K]) {
    let btw = cartalith_civ::civ_network_betweenness(before.len(), pairs);
    let got: Vec<u64> = btw.iter().map(|b| b.to_bits()).collect();
    assert_eq!(got, btw_bits, "{name}: betweenness");
    let mut places: Vec<_> = before.iter().map(|&k| place(k)).collect();
    cartalith_civ::civ_centrality_tier_feedback(&mut places, &btw);
    let kinds: Vec<K> = places.iter().map(|p| p.kind).collect();
    assert_eq!(kinds, after, "{name}: tiers");
}
`;
for (const c of out) {
  rs += `
#[test]
fn ${c.name}() {
    check(
        ${JSON.stringify(c.name)},
        &[${c.kinds.map(K).join(', ')}],
        &[${c.pairs.map(([a, b]) => `(${a}, ${b})`).join(', ')}],
        &[${c.btw.map(bits).join(', ')}],
        &[${c.after.map(K).join(', ')}],
    );
}
`;
}
process.stdout.write(rs);
