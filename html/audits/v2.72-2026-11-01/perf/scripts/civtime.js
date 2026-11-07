// Time _civIterativeAutoWorld(passes) at a resolution, with a CPU profile of the call aggregated by
// function (self and inclusive time). Usage: node civtime.js <file.html> <res> <passes>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, res, passes] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async (R) => { state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide(); }, +res);
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Profiler.enable'); await cdp.send('Profiler.setSamplingInterval', { interval: 500 }); await cdp.send('Profiler.start');
  const r = await page.evaluate((P) => { const t = performance.now(); _civIterativeAutoWorld(P); return { ms: performance.now() - t, places: state.places.length, ways: civWays.length }; }, +passes);
  const { profile } = await cdp.send('Profiler.stop');
  const dt = new Map(); profile.samples.forEach((s, i) => dt.set(s, (dt.get(s) || 0) + (profile.timeDeltas[i] || 0)));
  const byId = new Map(profile.nodes.map(n => [n.id, n])); const parent = new Map(); for (const n of profile.nodes) for (const c of (n.children || [])) parent.set(c, n.id);
  const self = new Map(), total = new Map(); const key = n => (n.callFrame.functionName || '(anon)') + ':' + (n.callFrame.lineNumber + 1);
  for (const n of profile.nodes) { const us = dt.get(n.id) || 0; if (!us) continue; self.set(key(n), (self.get(key(n)) || 0) + us);
    const seen = new Set(); let id = n.id; while (id != null) { const k = key(byId.get(id)); if (!seen.has(k)) { seen.add(k); total.set(k, (total.get(k) || 0) + us); } id = parent.get(id); } }
  const top = (m, n) => [...m.entries()].sort((a, b) => b[1] - a[1]).slice(0, n).map(([k, v]) => k + ' ' + (v / 1000).toFixed(0));
  console.log(JSON.stringify({ file: path.basename(file), res: +res, passes: +passes, ...r }));
  console.log('SELF:\n  ' + top(self, 15).join('\n  ')); console.log('TOTAL:\n  ' + top(total, 30).join('\n  '));
  await b.close();
})();
