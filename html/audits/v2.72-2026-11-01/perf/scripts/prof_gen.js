// CPU profile of one generate() at <res> (after a warm 512 generate), aggregated by function; also
// times each computeFlow() call and the routing-surface build inside it by wrapping the globals
// (timing only: the wrapped functions are called unchanged with the same arguments).
// Usage: node prof_gen.js <file.html> <res>
const path = require('path');
const { chromium } = require('playwright');
const agg = require('./profagg.js');
(async () => {
  const [file, res] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(1500);
  await page.evaluate(async () => { state.tect.seed = 12345; state.resW = 512; _setupSkipped = true; GW = 512; GH = gridH(GW); allocate(); await generate(); });
  await page.evaluate(() => {
    window.__calls = [];
    const wrap = name => { const o = window[name]; if (typeof o !== 'function') return; window[name] = function () { const t = performance.now(); const r = o.apply(this, arguments); window.__calls.push([name, +(performance.now() - t).toFixed(1)]); return r; }; };
    ['computeFlow', 'buildRoutingSurface', '_flowRadixSortDesc', 'refreshClimate', 'streamPowerKernel', 'buildRiverNetwork', 'traceRiverPolylines', 'enforceChannelDescent', 'applyRiverDeltas', 'isostaticRebound', 'enforceRiverChannels', 'refreshTides', 'buildWaterBodies', 'smoothSeaH', 'seaShadeFrom', 'updateReadout', 'maxGrade', 'drawCivLayer'].forEach(wrap);
  });
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Profiler.enable'); await cdp.send('Profiler.setSamplingInterval', { interval: 1000 }); await cdp.send('Profiler.start');
  const r = await page.evaluate(async (R) => {
    return await new Promise(resolve => setTimeout(async () => {
      state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate();
      const t = performance.now(); await generate(); resolve({ ms: performance.now() - t, stages: PERF.gen, render: PERF.render, flowSortMs: PERF.flowSortMs, integrate: !!(state.hydro && state.hydro.integrate) });
    }, 0));
  }, +res);
  const { profile } = await cdp.send('Profiler.stop');
  const calls = await page.evaluate(() => window.__calls);
  const byName = {}; for (const [n, ms] of calls) { (byName[n] = byName[n] || []).push(ms); }
  console.log(JSON.stringify({ file: path.basename(file), res: +res, ...r, calls: byName, prof: agg(profile, 30) }));
  await b.close();
})();
