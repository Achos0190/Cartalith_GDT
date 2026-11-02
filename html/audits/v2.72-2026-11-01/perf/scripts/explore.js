// Exploratory: load one file, report env, time generate() once at each resolution, report DOM size.
// Usage: node explore.js <file.html> [res,res,...]
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const file = process.argv[2];
  const resList = (process.argv[3] || '512,1024,2048').split(',').map(Number);
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1400, height: 900 } });
  const errs = []; p.on('pageerror', e => errs.push(String(e).slice(0, 200)));
  p.on('dialog', d => d.dismiss());
  const t0 = Date.now();
  await p.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 120000 });
  const tLoad = Date.now() - t0;
  await p.waitForFunction(() => typeof window.generate === 'function', { timeout: 60000 });
  const env = await p.evaluate(() => ({
    gpu: (typeof GPU !== 'undefined') ? (GPU.ok ? (GPU.enabled ? (GPU._r32f ? 'R32F' : 'RGBA32F') : 'off') : 'no-webgl2') : 'none',
    pool: (typeof GENPOOL !== 'undefined') ? (GENPOOL._tagStr || String(GENPOOL.usable)) : 'none',
    cores: navigator.hardwareConcurrency, dpr: devicePixelRatio,
    elements: document.getElementsByTagName('*').length,
    gate: (document.getElementById('onboard') || {}).style ? document.getElementById('onboard').style.display : 'n/a',
  }));
  const gens = {};
  for (const res of resList) {
    gens[res] = await p.evaluate(async (res) => {
      state.tect.seed = 12345; state.resW = res;
      if (typeof _setupSkipped !== 'undefined') _setupSkipped = true;
      GW = state.resW; GH = gridH(GW); allocate();
      const t = performance.now(); await generate(); const ms = performance.now() - t;
      return { ms: Math.round(ms), genTotal: Math.round(PERF.genTotal), stages: Object.fromEntries(Object.entries(PERF.gen).map(([k, v]) => [k, Math.round(v)])) };
    }, res);
  }
  console.log(JSON.stringify({ file, tLoad, env, gens, errs }, null, 1));
  await b.close();
})();
