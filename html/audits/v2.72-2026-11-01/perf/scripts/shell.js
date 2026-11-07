// Shell-only costs through REAL input (CDP mouse/keyboard/viewport), per version. Main-thread cost is
// read as CDP TaskDuration / ScriptDuration / RecalcStyleDuration / LayoutDuration deltas over N
// repetitions (so handler + style + layout + paint are all counted, and nothing depends on a
// synthetic click's detail). World: 512, seed 12345, no settlements (shell costs, not civ costs).
// Usage: node shell.js <file.html>
const path = require('path');
const { chromium } = require('playwright');
const file = process.argv[2];
const D = (a, b, n) => Object.fromEntries(['TaskDuration', 'ScriptDuration', 'RecalcStyleDuration', 'LayoutDuration', 'LayoutCount', 'RecalcStyleCount'].map(k => [k, +(((b[k] - a[k]) / n) * (k.endsWith('Count') ? 1 : 1000)).toFixed(3)]));
async function M(cdp) { const { metrics } = await cdp.send('Performance.getMetrics'); const m = {}; for (const x of metrics) m[x.name] = x.value; return m; }
(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  const errs = []; page.on('pageerror', e => errs.push(String(e).slice(0, 200)));
  const cdp = await ctx.newCDPSession(page); await cdp.send('Performance.enable');
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async () => { state.tect.seed = 12345; state.resW = 512; _setupSkipped = true; GW = 512; GH = gridH(GW); allocate(); await generate(); _setupHide(); });
  await page.waitForTimeout(800);
  const out = { file: path.basename(file), loadavg: require('fs').readFileSync('/proc/loadavg', 'utf8').trim() };
  const settle = () => page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
  const center = async sel => page.evaluate(s => { const e = document.querySelector(s); if (!e || !e.getClientRects().length) return null; const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; }, sel);

  // 1. File menu: open + close by real clicks, 10 times (20 clicks)
  { const c = await center('#fileMenuBtn'); await settle(); const a = await M(cdp);
    for (let i = 0; i < 10; i++) { await page.mouse.click(c.x, c.y); await settle(); await page.mouse.click(c.x, c.y); await settle(); }
    const z = await M(cdp); out.fileMenuPerClick = D(a, z, 20);
    out.fileMenuOpenAfter = await page.evaluate(() => { const m = document.getElementById('fileMenu'); return !!(m && m.classList.contains('open')); }); }
  // 2. v2.72's other menus, each open + close by real clicks, 5 times
  { const ids = await page.evaluate(() => [...document.querySelectorAll('.mb-btn')].filter(b => b.getClientRects().length && b.id !== 'fileMenuBtn').map(b => '#' + b.id));
    if (ids.length) { await settle(); const a = await M(cdp); let n = 0;
      for (const id of ids) { const c = await center(id); for (let i = 0; i < 5; i++) { await page.mouse.click(c.x, c.y); await settle(); await page.mouse.click(c.x, c.y); await settle(); n += 2; } }
      const z = await M(cdp); out.otherMenusPerClick = D(a, z, n); out.otherMenus = ids.length; } }
  // 3. hover-switching across the bar with a menu open (v2.72 only): 7 mouse moves per pass, 5 passes
  { const ids = await page.evaluate(() => [...document.querySelectorAll('.mb-btn')].filter(b => b.getClientRects().length).map(b => '#' + b.id));
    if (ids.length > 1) { const cs = []; for (const id of ids) cs.push(await center(id));
      await page.mouse.click(cs[0].x, cs[0].y); await settle(); const a = await M(cdp); let n = 0;
      for (let k = 0; k < 5; k++) for (const c of (k % 2 ? [...cs].reverse() : cs)) { await page.mouse.move(c.x, c.y); await settle(); n++; }
      const z = await M(cdp); out.menuHoverSwitchPerMove = D(a, z, n);
      await page.keyboard.press('Escape'); await settle(); } }
  // 4. domain switch by real clicks on the rail, 3 cycles of 4
  { const doms = ['civ', 'carto', 'explore', 'world']; const cs = []; for (const d of doms) cs.push(await center('#domainRail [data-domain="' + d + '"]'));
    await settle(); const a = await M(cdp); let n = 0;
    for (let k = 0; k < 3; k++) for (const c of cs) { if (!c) continue; await page.mouse.click(c.x, c.y); await settle(); n++; }
    const z = await M(cdp); out.domainSwitchPerClick = D(a, z, n); }
  // 5. Find: type 8 characters, then clear
  { const c = await center('#findInput'); await page.mouse.click(c.x, c.y); await settle(); const a = await M(cdp);
    for (const ch of 'elevation') { await page.keyboard.type(ch); await settle(); }
    const z = await M(cdp); out.findPerKey = D(a, z, 9);
    await page.keyboard.press('Escape'); await settle(); }
  // 6. resize across the 860px line and back, 3 times (6 resizes)
  { await page.evaluate(() => { window.__placeMs = []; if (typeof _dskPlace === 'function') { const o = window._dskPlace; window._dskPlace = function () { const t = performance.now(); const r = o.apply(this, arguments); window.__placeMs.push(+(performance.now() - t).toFixed(2)); return r; }; } });
    await settle(); const a = await M(cdp);
    for (let k = 0; k < 3; k++) { await page.setViewportSize({ width: 800, height: 900 }); await settle(); await page.waitForTimeout(100); await page.setViewportSize({ width: 1400, height: 900 }); await settle(); await page.waitForTimeout(100); }
    const z = await M(cdp); out.resizeAcrossLinePerResize = D(a, z, 6); out.placeMs = await page.evaluate(() => window.__placeMs); }
  // 7. a resize that stays on the desktop side (1400 -> 1200 -> 1400), 3 times
  { await settle(); const a = await M(cdp);
    for (let k = 0; k < 3; k++) { await page.setViewportSize({ width: 1200, height: 900 }); await settle(); await page.waitForTimeout(100); await page.setViewportSize({ width: 1400, height: 900 }); await settle(); await page.waitForTimeout(100); }
    const z = await M(cdp); out.resizeDesktopPerResize = D(a, z, 6); }
  // 8. idle: 3 s with nothing happening (any timer or rAF loop shows up here)
  { await settle(); const a = await M(cdp); await page.waitForTimeout(3000); const z = await M(cdp); out.idle3s = D(a, z, 1); }
  out.errs = errs;
  console.log(JSON.stringify(out));
  await b.close();
})();
