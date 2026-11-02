// Controlled LOD wheel A/B (removes the timer races of the real-input run): synthetic wheel ticks at
// the map centre with the default lodAuto (so the map enters the tiled-LOD viewer past 2.2x), 20 in
// then 20 out; after each tick the page gets a frame, a setTimeout(0) turn (the overview rebuild's
// slot) and another frame. The 240 ms settle refine is switched off for the run (measurement-only
// monkeypatch of scheduleLodRefine), so both versions do the same per-tick work. Reports main-thread
// time (CDP TaskDuration) for the gesture, per-tick handler ms, overview rebuild count, and the
// canvas-wrap size. Usage: node lod_ctrl.js <file.html> <res>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, res] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: +(process.env.VIEW_H || 900) } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  const cdp = await ctx.newCDPSession(page); await cdp.send('Performance.enable');
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async (R) => { state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide(); _civIterativeAutoWorld(3); renderNow(); _viewFill(); }, +res);
  await page.waitForTimeout(1000);
  const M = async () => { const { metrics } = await cdp.send('Performance.getMetrics'); const m = {}; for (const x of metrics) m[x.name] = x.value; return m; };
  const a = await M();
  const r = await page.evaluate(async () => {
    const raf = () => new Promise(r => requestAnimationFrame(r)), turn = () => new Promise(r => setTimeout(r, 0));
    window.scheduleLodRefine = function () {};
    let rebuilds = 0; const oR = window._lodScheduleOverviewRebuild; window._lodScheduleOverviewRebuild = function () { rebuilds++; return oR.apply(this, arguments); };
    const wrap = document.querySelector('.canvas-wrap'), wr = wrap.getBoundingClientRect(), cx = wr.left + wr.width / 2, cy = wr.top + wr.height / 2;
    const tick = [];
    const t0 = performance.now();
    for (let i = 0; i < 40; i++) {
      const t = performance.now(); wrap.dispatchEvent(new WheelEvent('wheel', { clientX: cx, clientY: cy, deltaY: i < 20 ? -100 : 100, bubbles: true, cancelable: true })); tick.push(performance.now() - t);
      await raf(); await turn(); await raf();
    }
    const total = performance.now() - t0;
    const med = x => { const s = [...x].sort((p, q) => p - q); return +s[s.length >> 1].toFixed(1); };
    return { wrap: [Math.round(wr.width), Math.round(wr.height)], gestureMs: Math.round(total), tickMedian: med(tick), tickMax: +Math.max(...tick).toFixed(1), rebuilds, lodOn: _lodOn, lodZoom: _lodZoom };
  });
  const z = await M();
  console.log(JSON.stringify({ file: path.basename(file), res: +res, ...r, taskMs: Math.round((z.TaskDuration - a.TaskDuration) * 1000), scriptMs: Math.round((z.ScriptDuration - a.ScriptDuration) * 1000) }));
  await b.close();
})();
