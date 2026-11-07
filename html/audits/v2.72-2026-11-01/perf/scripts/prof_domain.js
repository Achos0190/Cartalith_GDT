// Domain switches by real clicks on the rail (WORLD -> CIVIL -> CARTO -> EXPLORE, twice), with a CPU
// profile aggregated by function, and whether a switch keeps the user's zoom: the map is zoomed to
// 1.8x first and viewT.scale is read after each switch.
// Usage: node prof_domain.js <file.html> <res>
const path = require('path');
const { chromium } = require('playwright');
const agg = require('./profagg.js');
(async () => {
  const [file, res] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async (R) => { state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide(); _civIterativeAutoWorld(3); renderNow(); state.lodAuto = false; }, +res);
  await page.waitForTimeout(800);
  const settle = () => page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
  const cs = {};
  for (const d of ['world', 'civ', 'carto', 'explore']) cs[d] = await page.evaluate(d => { const e = document.querySelector('#domainRail [data-domain="' + d + '"]'); const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; }, d);
  // zoom-keeping check
  const keep = [];
  for (const d of ['civ', 'carto', 'explore', 'world']) {
    await page.evaluate(() => { _viewFill(); const r = document.querySelector('.canvas-wrap').getBoundingClientRect(); zoomAt(r.left + r.width / 2, r.top + r.height / 2, 1.8); });
    await settle();
    const before = await page.evaluate(() => viewT.scale);
    await page.mouse.click(cs[d].x, cs[d].y); await settle(); await settle();
    const after = await page.evaluate(() => viewT.scale);
    keep.push({ to: d, scaleBefore: +before.toFixed(3), scaleAfter: +after.toFixed(3) });
  }
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Profiler.enable'); await cdp.send('Profiler.setSamplingInterval', { interval: 100 }); await cdp.send('Profiler.start');
  for (let k = 0; k < 2; k++) for (const d of ['civ', 'carto', 'explore', 'world']) { await page.mouse.click(cs[d].x, cs[d].y); await settle(); await settle(); }
  const { profile } = await cdp.send('Profiler.stop');
  const a = agg(profile, 25);
  console.log(JSON.stringify({ file: path.basename(file), res: +res, keep, prof: { self: a.self.filter(([k]) => !/^\(idle\)/.test(k)), total: a.total.filter(([k]) => !/^\((idle|root)\)/.test(k)).slice(0, 40) } }));
  await b.close();
})();
