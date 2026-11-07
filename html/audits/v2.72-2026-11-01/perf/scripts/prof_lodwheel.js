// The DEFAULT wheel-zoom path (state.lodAuto true: past 2.2x the map enters the tiled-LOD viewer):
// real CDP wheel input, 20 ticks in then 20 out at the map centre, with a CPU profile, the longtask
// list, rAF intervals, and per-call timings of the LOD entry points (wrapped globals, timing only).
// Usage: node prof_lodwheel.js <file.html> <res> [civ=1]
const path = require('path');
const { chromium } = require('playwright');
const agg = require('./profagg.js');
(async () => {
  const [file, res, civ] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  const errs = []; page.on('pageerror', e => errs.push(String(e).slice(0, 200)));
  await page.addInitScript(`window.__lt=[]; new PerformanceObserver(l=>{ for(const e of l.getEntries()) window.__lt.push([e.startTime,e.duration]); }).observe({type:'longtask',buffered:true});`);
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  const w = await page.evaluate(async ({ R, civ }) => {
    state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide();
    if (civ) _civIterativeAutoWorld(3);
    renderNow(); _viewFill();
    return { places: state.places.length, lodAuto: state.lodAuto, gpu: GPU.enabled, pool: GENPOOL._tagStr };
  }, { R: +res, civ: civ !== '0' });
  await page.waitForTimeout(1000);
  await page.evaluate(() => {
    window.__calls = [];
    const wrap = name => { const o = window[name]; if (typeof o !== 'function') return; window[name] = function () { const t = performance.now(); const r = o.apply(this, arguments); if (r && typeof r.then === 'function') { r.then(() => window.__calls.push([name + '(async)', +(performance.now() - t).toFixed(1)])); } else window.__calls.push([name, +(performance.now() - t).toFixed(1)]); return r; }; };
    ['enterLodFromView', 'drawLODView', 'refineVisibleTiles', 'pyramidTile', 'drawCivLayer', 'applyView', 'updateScaleBar'].forEach(wrap);
    window.__fr = []; window.__run = true; window.__lt0 = window.__lt.length;
    const loop = t => { window.__fr.push(t); if (window.__run) requestAnimationFrame(loop); }; requestAnimationFrame(loop);
  });
  const c = await page.evaluate(() => { const r = document.querySelector('.canvas-wrap').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; });
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Profiler.enable'); await cdp.send('Profiler.setSamplingInterval', { interval: 500 }); await cdp.send('Profiler.start');
  const T0 = Date.now();
  await page.mouse.move(c.x, c.y);
  for (let i = 0; i < 40; i++) { await page.mouse.wheel(0, i < 20 ? -100 : 100); await new Promise(r => setTimeout(r, 16)); }
  const wall = Date.now() - T0;
  await page.waitForTimeout(1500);   // the 240 ms settle-refine and its budgeted fade-in
  const { profile } = await cdp.send('Profiler.stop');
  const r = await page.evaluate(() => {
    window.__run = false;
    const f = window.__fr, iv = []; for (let i = 1; i < f.length; i++) iv.push(f[i] - f[i - 1]);
    const s = [...iv].sort((x, y) => x - y), q = p => s[Math.min(s.length - 1, Math.floor(p * s.length))];
    const lt = window.__lt.slice(window.__lt0);
    const byName = {}; for (const [n, ms] of window.__calls) { (byName[n] = byName[n] || []).push(ms); }
    const sum = a => +a.reduce((x, y) => x + y, 0).toFixed(0);
    return { frames: f.length, ivMedian: q(0.5), ivP95: q(0.95), ivMax: s[s.length - 1], longTasks: lt.length, longTaskTotal: Math.round(lt.reduce((x, [, d]) => x + d, 0)), longTaskMax: Math.round(Math.max(0, ...lt.map(([, d]) => d))),
             lt: lt.map(([s0, d]) => Math.round(d)), lodOn: _lodOn, lodZoom: _lodZoom,
             calls: Object.fromEntries(Object.entries(byName).map(([k, v]) => [k, { n: v.length, sum: sum(v), max: Math.max(...v) }])) };
  });
  console.log(JSON.stringify({ file: path.basename(file), res: +res, world: w, wallMs: wall, ...r, prof: agg(profile, 25), errs }));
  await b.close();
})();
