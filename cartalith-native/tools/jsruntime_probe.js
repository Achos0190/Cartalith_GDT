#!/usr/bin/env node
/* Proof that a JS runtime can drive the FROZEN reference, checked against a
 * fixture already committed to this tree.
 *
 * Written 2026-09-02 because three live test headers cited "this environment
 * has no JS runtime" as the reason real golden fixtures could not be
 * extracted. `node --version` is v24.19.0, so the claim is stale -- but a
 * version string proves only that node exists, not that it can slice, load
 * and correctly evaluate THIS reference file. This script proves the whole
 * chain, end to end, and fails loudly at every step that could silently pass:
 *
 *   1. slice `deflectFlow` (reference lines 5315-5357) and its only
 *      dependency `blurCoarse` (5543-5548) out of the frozen HTML;
 *   2. pin both slices with structural first/last-line assertions, a
 *      block-comment balance + orphan-close check, and a purity check --
 *      the convention `tools/um_capture.js` established;
 *   3. read the INPUTS AND THE EXPECTED OUTPUTS back out of the committed
 *      Rust golden `cartalith-climate/tests/golden_parity_deflect_flow.rs`;
 *   4. run all three of its cases through the reference's own code in a bare
 *      `vm` context and require BIT-EXACT f32 agreement.
 *
 * Step 4 is the load-bearing one: it is a two-way check. A pass means the
 * runtime really does execute this reference, AND that the committed
 * fixtures genuinely came out of it rather than out of the Rust port.
 *
 * The frozen file is READ ONLY here and is never written.
 *
 * Usage:  node tools/jsruntime_probe.js       (exit 0 = the chain works)
 */
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const ROOT = path.resolve(__dirname, '..', '..');
const REF = path.join(ROOT, 'reference', 'Cartalith Gen1 v2.10.html');
const GOLDEN = path.join(__dirname, '..', 'crates', 'cartalith-climate', 'tests',
                         'golden_parity_deflect_flow.rs');

const DEFLECT_FIRST = 5315, DEFLECT_LAST = 5357;
const BLUR_FIRST = 5543, BLUR_LAST = 5548;

function die(msg) { console.error('PROBE FAILED: ' + msg); process.exit(1); }
function ok(msg) { console.error('  ok  ' + msg); }

// ------------------------------------------------------------- the slices --

const lines = fs.readFileSync(REF, 'utf8').split(/\r?\n/);
console.error(`reference: ${path.basename(REF)}, ${lines.length} lines`);

function slice(first, last, headRe, tailRe, label) {
  const head = lines[first - 1], tail = lines[last - 1];
  if (!headRe.test(head)) die(`line ${first} is not ${label}'s opening; got:\n  ${head}`);
  if (!tailRe.test(tail)) die(`line ${last} is not ${label}'s closing brace; got:\n  ${tail}`);
  const s = lines.slice(first - 1, last).join('\n');
  // Block-comment balance plus the orphan-close counter: a slice that starts
  // inside a comment parses fine and silently omits its own first statements.
  const opens = (s.match(/\/\*/g) || []).length, closes = (s.match(/\*\//g) || []).length;
  if (opens !== closes) die(`${label}: comment imbalance, ${opens} '/*' vs ${closes} '*/'`);
  let depth = 0, orphans = 0, m, re = /\/\*|\*\//g;
  while ((m = re.exec(s))) { if (m[0] === '/*') depth++; else if (depth === 0) orphans++; else depth--; }
  if (orphans) die(`${label}: ${orphans} orphan '*/' -- the slice starts inside a comment`);
  if (depth) die(`${label}: the slice ends inside a block comment`);
  // Purity: neither function may reach for the DOM, the clock, or Math.random.
  for (const bad of [/\bdocument\./, /\bwindow\./, /\bMath\.random\b/, /\bDate\.now\b/,
                     /\bperformance\./, /\bstate\./, /\bgetElementById\b/]) {
    if (bad.test(s)) die(`${label} references ${bad}; this slice is supposed to be pure`);
  }
  ok(`${label}: lines ${first}-${last} sliced, boundaries + comment balance + purity verified`);
  return s;
}

const deflectSrc = slice(DEFLECT_FIRST, DEFLECT_LAST,
  /^function deflectFlow\(u0, v0, block0, WW, WH, wrapX, opts\)\{/, /^\}\s*$/, 'deflectFlow');
const blurSrc = slice(BLUR_FIRST, BLUR_LAST,
  /^function blurCoarse\(a,WW,WH,wrapX,passes\)\{/, /^\}\s*$/, 'blurCoarse');

// Negative control: deflectFlow must NOT define blurCoarse, which is the whole
// reason the second slice exists. If it ever did, the splice would shadow it.
if (/function\s+blurCoarse\s*\(/.test(deflectSrc)) die('deflectFlow defines blurCoarse');
ok('negative control: deflectFlow does not define blurCoarse');

// ------------------------------------------------- the committed fixtures --

const rs = fs.readFileSync(GOLDEN, 'utf8');
function rustArray(name) {
  // `const U0: [f32; 192] = [...];` or `let expected_u: Vec<f32> = vec![...];`
  const re = new RegExp('(?:const|let)\\s+' + name + '\\s*:\\s*(?:\\[f32;\\s*\\d+\\]|Vec<f32>)\\s*=\\s*(?:vec!)?\\[([^\\]]*)\\]', 'g');
  const out = [];
  let m;
  while ((m = re.exec(rs))) {
    const v = m[1].split(',').map(t => t.trim()).filter(t => t.length)
                  .map(t => Math.fround(Number(t.replace(/f32$/, ''))));
    if (v.some(Number.isNaN)) die(`${name}: a value in the Rust golden did not parse as a number`);
    out.push(v);
  }
  if (!out.length) die(`${name}: not found in ${path.basename(GOLDEN)}`);
  return out;
}
const [U0] = rustArray('U0'), [V0] = rustArray('V0'), [BLOCK0] = rustArray('BLOCK0');
const EXP_U = rustArray('expected_u'), EXP_V = rustArray('expected_v');
if (U0.length !== 192 || V0.length !== 192 || BLOCK0.length !== 192) {
  die(`fixture inputs are ${U0.length}/${V0.length}/${BLOCK0.length}, expected 192 each`);
}
if (EXP_U.length !== 3 || EXP_V.length !== 3) {
  die(`expected 3 golden cases, parsed ${EXP_U.length} u / ${EXP_V.length} v`);
}
// A golden of all-zeros would compare equal to a broken run of all-zeros.
for (const [n, a] of [['U0', U0], ['V0', V0], ['BLOCK0', BLOCK0]]) {
  if (!a.some(x => x !== 0)) die(`${n} is entirely zero -- the fixture parsed empty`);
}
ok(`fixtures read from ${path.basename(GOLDEN)}: 3 inputs of 192, 3 expected pairs`);

// --------------------------------------------------------------- evaluate --

const ctx = vm.createContext({ Float32Array, Math, module: { exports: {} } });
vm.runInContext(blurSrc + '\n' + deflectSrc + '\nmodule.exports = deflectFlow;', ctx,
                { filename: 'reference-deflectFlow.js' });
const deflectFlow = ctx.module.exports;
if (typeof deflectFlow !== 'function') die('the slice did not evaluate to a function');
ok('slices evaluated in a bare vm context (no DOM, no globals beyond Float32Array/Math)');

// The three cases, transcribed from the golden's own `DeflectFlowParams`.
const CASES = [
  ['deflect_flow_default_no_wrap', false, { strength: 1.0, k1: 0.55, k2: 0.65, gapK: 0.4, iterations: 16, blockBlur: 2 }],
  ['deflect_flow_wrap_x',           true, { strength: 1.0, k1: 0.55, k2: 0.65, gapK: 0.4, iterations: 16, blockBlur: 2 }],
  ['deflect_flow_custom_knobs',    false, { strength: 0.7, k1: 0.4,  k2: 0.5,  gapK: 0.6, iterations: 6,  blockBlur: 1 }],
];

let bad = 0;
CASES.forEach(([name, wrapX, opts], c) => {
  // `deflectFlow` mutates nothing it is given (it copies via .slice()), but
  // pass fresh arrays anyway so a case cannot contaminate the next.
  const r = deflectFlow(Float32Array.from(U0), Float32Array.from(V0),
                        Float32Array.from(BLOCK0), 16, 12, wrapX, opts);
  const u = Array.from(r.u), v = Array.from(r.v);
  if (u.length !== 192 || v.length !== 192) die(`${name}: reference returned ${u.length}/${v.length}`);
  if (!u.some(x => x !== 0) || !v.some(x => x !== 0)) die(`${name}: reference output is entirely zero`);
  let mism = 0, worst = null;
  for (let i = 0; i < 192; i++) {
    if (!Object.is(u[i], EXP_U[c][i])) { mism++; if (!worst) worst = `u[${i}] js=${u[i]} rust=${EXP_U[c][i]}`; }
    if (!Object.is(v[i], EXP_V[c][i])) { mism++; if (!worst) worst = `v[${i}] js=${v[i]} rust=${EXP_V[c][i]}`; }
  }
  if (mism) { bad++; console.error(`  FAIL  ${name}: ${mism}/384 differ; first ${worst}`); }
  else ok(`${name}: 384/384 f32 values bit-identical to the committed golden`);
});

if (bad) die(`${bad} of ${CASES.length} cases disagree with the committed golden`);
console.error('PROBE PASSED: node v' + process.versions.node +
              ' runs the frozen reference, and the committed fixtures are its real output.');
