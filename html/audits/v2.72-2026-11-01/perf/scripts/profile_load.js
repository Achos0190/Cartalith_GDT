// Attribute the load-time long task: CPU profile from navigation to load+1.5s, aggregated by function
// (self time) and by top-level script, plus PERF.render from the boot render.
// Usage: node profile_load.js <file.html>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const file = process.argv[2];
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage();
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Profiler.enable');
  await cdp.send('Profiler.setSamplingInterval', { interval: 200 });
  await cdp.send('Profiler.start');
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(1500);
  const { profile } = await cdp.send('Profiler.stop');
  const perf = await page.evaluate(() => ({ render: (typeof PERF !== 'undefined') ? PERF.render : null, GW, GH }));
  // self time per node
  const dt = new Map(); const { samples, timeDeltas, nodes } = profile;
  for (let i = 0; i < samples.length; i++) dt.set(samples[i], (dt.get(samples[i]) || 0) + (timeDeltas[i] || 0));
  const byId = new Map(nodes.map(n => [n.id, n]));
  const parent = new Map(); for (const n of nodes) for (const c of (n.children || [])) parent.set(c, n.id);
  const self = new Map(), total = new Map();
  for (const n of nodes) {
    const us = dt.get(n.id) || 0; if (!us) continue;
    const key = (n.callFrame.functionName || '(anon)') + ':' + (n.callFrame.lineNumber + 1);
    self.set(key, (self.get(key) || 0) + us);
    // inclusive: walk up, count each distinct key once per sample-node
    const seen = new Set(); let id = n.id;
    while (id != null) { const m = byId.get(id); const k = (m.callFrame.functionName || '(anon)') + ':' + (m.callFrame.lineNumber + 1); if (!seen.has(k)) { seen.add(k); total.set(k, (total.get(k) || 0) + us); } id = parent.get(id); }
  }
  const top = (m, n) => [...m.entries()].sort((a, b) => b[1] - a[1]).slice(0, n).map(([k, v]) => [k, +(v / 1000).toFixed(1)]);
  console.log(JSON.stringify({ file: path.basename(file), perf, selfMs: top(self, 25), totalMs: top(total, 40) }, null, 1));
  await b.close();
})();
