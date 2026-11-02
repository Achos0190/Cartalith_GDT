// Does dragging the Sun slider (#sun -> state.sunAz) change the map? Hashes the main canvas before and
// after an 'input' on #sun, then again after clearing the civ bake cache and rendering. If the slider
// works, hash1 != hash0. If hash1 == hash0 but hash2 != hash0, the render DOES depend on the sun and
// the bake cache served a stale image (its key, _civBakeKey, reads state.sun; the slider writes
// state.sunAz). Also reports whether the cache key itself changed.
// Usage: node sun_probe.js <file.html>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const file = process.argv[2];
  const b = await chromium.launch();
  const page = await b.newPage({ viewport: { width: 1400, height: 900 } }); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  const r = await page.evaluate(async () => {
    state.tect.seed = 12345; state.resW = 512; _setupSkipped = true; GW = 512; GH = gridH(GW); allocate(); await generate(); _setupHide();
    const raf2 = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const fnv = d => { let h = 2166136261 >>> 0; for (let i = 0; i < d.length; i++) { h ^= d[i]; h = Math.imul(h, 16777619) >>> 0; } return h >>> 0; };
    const H = () => fnv(vctx.getImageData(0, 0, GW, GH).data);
    renderNow(); await raf2();
    const h0 = H(), k0 = _civBakeKey(), az0 = state.sunAz, n0 = PERF.counters.renderPixelLoop;
    const el = document.getElementById('sun'); el.value = String((+el.value + 90) % 360); el.dispatchEvent(new Event('input', { bubbles: true }));
    await raf2();
    const h1 = H(), k1 = _civBakeKey(), az1 = state.sunAz, n1 = PERF.counters.renderPixelLoop;
    _civBakeCache.clear(); renderNow(); await raf2();
    const h2 = H();
    // the same check for a slider that is in the key (exag), as a positive control
    const ex = document.getElementById('exag'); const e0 = H(); ex.value = String(Math.min(100, +ex.value + 20)); ex.dispatchEvent(new Event('input', { bubbles: true })); await raf2(); const e1 = H();
    return { sunAzBefore: az0, sunAzAfter: az1, keyChanged: k0 !== k1, pixelLoopRan: n1 > n0, mapChangedOnInput: h1 !== h0, mapChangesWhenCacheCleared: h2 !== h0, control_exagChangesMap: e1 !== e0, stateHasSunProp: ('sun' in state) };
  });
  const ok = (n, c) => console.log((c ? 'ok   ' : 'FAIL ') + n);
  console.log(path.basename(file), JSON.stringify(r));
  ok('positive control: the exag slider changes the map', r.control_exagChangesMap);
  ok('the render depends on the sun azimuth (cache cleared)', r.mapChangesWhenCacheCleared);
  ok('dragging the Sun slider changes the map', r.mapChangedOnInput);
  await b.close();
})();
