// Behavioural check for the v2.11 ponytail fixes. Parsing proves nothing about
// whether the code still answers the same question, and this repository has
// been bitten four times by output that passed every structural check and was
// silently empty.
//
// The whole file cannot simply be evaluated here: `let` in a vm script is a
// lexical binding, not a context property, so any block that dies on a DOM stub
// leaves its `let`s in the temporal dead zone. That is the exact pitfall
// golden_parity_civ_tools.rs documents. So the behavioural cases lift the
// SHIPPED function text out of the file and close it over bindings we control --
// testing the real source, not a reimplementation of it.
//
// Run: node ponycheck.js
const fs = require('fs'), vm = require('vm'), path = require('path');

const REPO = 'C:\\Users\\Vincent\\Cartalith_GDT';
const NEW = path.join(REPO, 'Cartalith Gen1 v2.11.html');
const OLD = path.join(REPO, 'reference', 'Cartalith Gen1 v2.10.html');
const newSrc = fs.readFileSync(NEW, 'utf8');
const oldSrc = fs.readFileSync(OLD, 'utf8');

let fails = 0;
const ok = (name, got, want) => {
  const good = JSON.stringify(got) === JSON.stringify(want);
  if (!good) fails++;
  console.log(`  ${good ? 'ok  ' : 'FAIL'} ${name}   got=${JSON.stringify(got)} want=${JSON.stringify(want)}`);
};
const fnText = (name) => {
  const m = newSrc.match(new RegExp('function ' + name + '\\([\\s\\S]*?\\n\\}'));
  if (!m) { fails++; console.log(`  FAIL could not extract ${name}`); return 'function ' + name + '(){}'; }
  return m[0];
};

console.log('=== fix 1: _umWaterCtx uses the shared cache; the cache keys on its source ===');
{
  const water = fnText('_umWaterCtx'), cache = fnText('_civRiverPolylines');
  ok('_umWaterCtx no longer traces directly', /traceRiverPolylines\s*\(/.test(water), false);
  ok('_umWaterCtx calls the shared cache', /_civRiverPolylines\s*\(/.test(water), true);
  ok('_umWaterCtx no longer duplicates the lazy _riverNet build', /buildRiverNetwork/.test(water), false);
  ok('the cache no longer keys on _fieldGen', /_fieldGen/.test(cache), false);
  ok('the cache keys on the source object', /_civRiverPolysSrc\s*===\s*_riverNet/.test(cache), true);

  const box = vm.createContext({ Int32Array, result: null });
  new vm.Script(`
    let _civRiverPolys=null, _civRiverPolysSrc=null;
    let _riverNet=null, traceCalls=0;
    const GW=8, GH=8, field=null, flowField=null;
    const state={seaLevel:0.42, world:false, viz:{riverDensity:1}};
    const buildRiverNetwork=undefined;
    function traceRiverPolylines(){ traceCalls++; return [[{x:1,y:1},{x:2,y:2}]]; }
    ${cache}
    result = (()=>{
      const out={};
      _riverNet={order:new Int32Array(64), recv:new Int32Array(64)};
      _civRiverPolylines();                       out.first=traceCalls;
      _civRiverPolylines(); _civRiverPolylines();  out.cached=traceCalls;
      // riverDensR's own path: the source object is replaced and nothing else
      // moves. The old _fieldGen key served a stale trace here.
      _riverNet={order:new Int32Array(64), recv:new Int32Array(64)};
      _civRiverPolylines();                       out.afterReplace=traceCalls;
      _riverNet=null;                             out.nullSource=_civRiverPolylines();
      return out;
    })();
  `).runInContext(box);
  ok('first call traces', box.result.first, 1);
  ok('repeats are served from cache', box.result.cached, 1);
  ok('a replaced river network re-traces (the path _fieldGen missed)', box.result.afterReplace, 2);
  ok('a missing river network returns null, not a throw', box.result.nullSource, null);
}

console.log('\n=== fix 2: state.seaLevel read without the falsy-zero fallback ===');
{
  const count = s => (s.match(/state\.seaLevel\s*\|\|\s*0\.42/g) || []).length;
  ok('v2.10 carried the fallback', count(oldSrc) > 0, true);
  ok('v2.11 carries none', count(newSrc), 0);
  ok('_civDropPlace no longer hardcodes a floor', /state\.seaLevel\s*\|\|/.test(fnText('_civDropPlace')), false);
  ok('the slider still has min="0", so 0 is reachable', /id="sea"[^>]*min="0"/.test(newSrc), true);
  ok('state.seaLevel is still initialised, so it is never undefined', /seaLevel:\s*0\.42/.test(newSrc), true);
}

console.log('\n=== fix 3: one invalidation list, eight callers ===');
{
  const LIST = '_resourcePots=null; _carryCapField=null; _settleSuitField=null; _wildlife=null; _nppField=null; _triField=null; _popDensityField=null; _wetlandMask=null;';
  ok('v2.10 carried eight copies', (oldSrc.split(LIST).length - 1), 8);
  ok('v2.11 carries none', (newSrc.split(LIST).length - 1), 0);
  ok('eight call sites', (newSrc.match(/invalidateDerived\(\);/g) || []).length, 8);

  const box = vm.createContext({ result: null });
  new vm.Script(`
    let _resourcePots=1,_carryCapField=1,_settleSuitField=1,_wildlife=1;
    let _nppField=1,_triField=1,_popDensityField=1,_wetlandMask=1;
    ${fnText('invalidateDerived')}
    invalidateDerived();
    result=[_resourcePots,_carryCapField,_settleSuitField,_wildlife,_nppField,_triField,_popDensityField,_wetlandMask];
  `).runInContext(box);
  ok('it nulls every one of the eight', box.result, [null, null, null, null, null, null, null, null]);
  // The _biomeK toggle nulls only the four caches that actually depend on it.
  // That narrower list is correct, not drift, and must not have been swept in.
  // Asserted on the toggle's own LINE -- an earlier version of this check used
  // a proximity regex over the whole file and matched invalidateDerived's own
  // doc comment, which mentions _biomeK by name. A test that greps near prose
  // tests the prose.
  const bkLine = newSrc.split('\n').find(l => /_biomeK\s*=\s*bk\.checked/.test(l)) || '';
  ok('the _biomeK toggle still nulls its own four', [
    /_carryCapField=null/.test(bkLine), /_settleSuitField=null/.test(bkLine),
    /_popDensityField=null/.test(bkLine), /_wetlandMask=null/.test(bkLine)],
    [true, true, true, true]);
  ok('...and does not call the shared list', /invalidateDerived/.test(bkLine), false);
  ok('...and still leaves the four it does not own alone', [
    /_resourcePots/.test(bkLine), /_wildlife/.test(bkLine),
    /_nppField/.test(bkLine), /_triField/.test(bkLine)], [false, false, false, false]);
}

console.log('\n=== the frozen reference is untouched ===');
{
  const crypto = require('crypto');
  ok('reference md5', crypto.createHash('md5').update(fs.readFileSync(OLD)).digest('hex'),
    '9cba09ace11670c412ee35ca3e266d6c');
}

console.log(`\nponycheck: ${fails ? fails + ' FAILURE(S)' : 'PASS'}`);
process.exit(fails ? 1 : 0);
