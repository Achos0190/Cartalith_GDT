// A look slider dragged: 8 'input' events on each slider, one per frame (as a pointer drag delivers
// them), on a <res> world. Reports per-event render cost (PERF.render.total of the render the event
// schedules), the frame interval while dragging, and long tasks. Each slider is restored after.
// Usage: node slider.js <file.html> <res>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, res] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.addInitScript(`window.__lt=[]; new PerformanceObserver(l=>{ for(const e of l.getEntries()) window.__lt.push([e.startTime,e.duration]); }).observe({type:'longtask',buffered:true});`);
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async (R) => { state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide(); }, +res);
  await page.waitForTimeout(800);
  const out = {};
  for (const id of ['sun', 'exag', 'bioBlend', 'aoR', 'sea', 'peak']) {
    out[id] = await page.evaluate(async (id) => {
      const el = document.getElementById(id); if (!el) return null;
      const raf = () => new Promise(r => requestAnimationFrame(r));
      const v0 = el.value, lo = +el.min || 0, hi = +el.max || 100;
      const lt0 = window.__lt.length, renders = [], handler = [], frames = [];
      let tPrev = performance.now();
      for (let i = 0; i < 8; i++) {
        el.value = String(Math.round(lo + (hi - lo) * (0.3 + 0.05 * i)));
        const n0 = PERF.counters.renderPixelLoop, t0 = performance.now();
        el.dispatchEvent(new Event('input', { bubbles: true }));
        handler.push(performance.now() - t0);
        await raf(); await raf();
        frames.push(performance.now() - tPrev); tPrev = performance.now();
        renders.push(PERF.counters.renderPixelLoop > n0 ? PERF.render.total : 0);
      }
      el.value = v0; el.dispatchEvent(new Event('input', { bubbles: true })); el.dispatchEvent(new Event('change', { bubbles: true }));
      await raf(); await raf();
      const lt = window.__lt.slice(lt0);
      const med = a => { const s = [...a].sort((x, y) => x - y); return +s[s.length >> 1].toFixed(1); };
      return { handlerMs: med(handler), handlerMax: +Math.max(...handler).toFixed(1), renderMs: med(renders), framePerInputMs: med(frames), longTasks: lt.length, longTaskMax: Math.round(Math.max(0, ...lt.map(([, d]) => d))) };
    }, id);
  }
  console.log(JSON.stringify({ file: path.basename(file), res: +res, ...out }));
  await b.close();
})();
