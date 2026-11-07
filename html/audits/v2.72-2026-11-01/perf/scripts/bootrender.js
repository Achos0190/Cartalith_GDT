// The boot render's own cost, unprofiled: PERF.render (written by the boot renderNow of the empty
// 2048 field behind the setup gate) and the load's longest task, after load + 1.5 s.
// Usage: node bootrender.js <file.html>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const file = process.argv[2];
  const b = await chromium.launch();
  const page = await b.newPage({ viewport: { width: 1400, height: 900 } });
  await page.addInitScript(`window.__lt=[]; new PerformanceObserver(l=>{ for(const e of l.getEntries()) window.__lt.push(e.duration); }).observe({type:'longtask',buffered:true});`);
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(1500);
  const r = await page.evaluate(() => ({ boot: { ...PERF.render }, grid: GW + 'x' + GH, pixelLoops: PERF.counters.renderPixelLoop, longTaskMax: Math.max(0, ...window.__lt), dcl: performance.getEntriesByType('navigation')[0].domContentLoadedEventEnd }));
  console.log(JSON.stringify({ file: path.basename(file), ...r }));
  await b.close();
})();
