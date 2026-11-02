// Event-listener and DOM-node growth across repeated ordinary operations (a leak shows as growth
// that scales with the repetition count). CDP JSEventListeners/Nodes after a forced GC, read after
// round 1 and after round 4 of: generate (512), open+close Settings, open+close the Settlements
// window, toggle the timeline strip (v2.72) / the legend, open+close every menu, select+deselect a
// settlement through the inspector, run Find.
// Usage: node listeners.js <file.html>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const file = process.argv[2];
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  const errs = []; page.on('pageerror', e => errs.push(String(e).slice(0, 200)));
  const cdp = await ctx.newCDPSession(page); await cdp.send('Performance.enable'); await cdp.send('HeapProfiler.enable');
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async () => { state.tect.seed = 12345; state.resW = 512; _setupSkipped = true; GW = 512; GH = gridH(GW); allocate(); await generate(); _setupHide(); _civIterativeAutoWorld(1); });
  const M = async () => { await cdp.send('HeapProfiler.collectGarbage'); await page.waitForTimeout(150); const { metrics } = await cdp.send('Performance.getMetrics'); const m = {}; for (const x of metrics) m[x.name] = x.value; return { listeners: m.JSEventListeners, nodes: m.Nodes, heapMB: +(m.JSHeapUsedSize / 1048576).toFixed(2) }; };
  const round = () => page.evaluate(async () => {
    const raf = () => new Promise(r => requestAnimationFrame(r));
    state.tect.seed = 777; await generate(); await raf();
    if (typeof window._openSettings === 'function') { window._openSettings('workspace'); await raf(); const m = document.getElementById('settingsModal'); const x = m && m.querySelector('[data-close],.close,.x,button[aria-label="Close"]'); if (x) x.click(); else if (m) m.classList.remove('open'); await raf(); }
    { const sb = document.getElementById('settlementsHeaderBtn'); if (sb) { sb.click(); await raf(); const m = document.getElementById('civFactionsModal'); const x = m && m.querySelector('.cfm-close,[data-close],button[aria-label="Close"]'); if (x) x.click(); else if (m) m.classList.remove('open'); await raf(); } }
    { const t = document.getElementById('tlStripToggle'); if (t) { t.click(); await raf(); t.click(); await raf(); } }
    for (const b of document.querySelectorAll('.mb-btn,#fileMenuBtn')) { if (!b.getClientRects().length) continue; b.click(); await raf(); document.body.click(); await raf(); }
    { const p = state.places.find(x => x && x.category === 'settlement'); if (p) { _civSelectedPlace = p; _civRenderInspector(); await raf(); _civSelectedPlace = null; _civRenderInspector(); await raf(); } }
    { const fi = document.getElementById('findInput'); if (fi) { fi.value = 'sea'; fi.dispatchEvent(new Event('input', { bubbles: true })); await raf(); fi.value = ''; fi.dispatchEvent(new Event('input', { bubbles: true })); } }
  });
  await round(); const r1 = await M();
  await round(); await round(); await round(); const r4 = await M();
  console.log(JSON.stringify({ file: path.basename(file), afterRound1: r1, afterRound4: r4, growthPerRound: { listeners: (r4.listeners - r1.listeners) / 3, nodes: (r4.nodes - r1.nodes) / 3, heapMB: +((r4.heapMB - r1.heapMB) / 3).toFixed(2) }, errs }));
  await b.close();
})();
