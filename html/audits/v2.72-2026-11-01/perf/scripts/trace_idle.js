// What does the renderer main thread do while the page sits idle? A 3 s CDP trace after a 512 world
// is generated and the page has settled, summarised by trace-event name (main-thread complete
// events, total ms) — plus the same window's TaskDuration/ScriptDuration from Performance.getMetrics,
// measured twice (right after settling, and again 6 s later).
// Usage: node trace_idle.js <file.html> [res]
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, res] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  const cdp = await ctx.newCDPSession(page); await cdp.send('Performance.enable');
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  await page.evaluate(async (R) => { state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide(); }, +(res || 512));
  await page.waitForTimeout(1500);
  const M = async () => { const { metrics } = await cdp.send('Performance.getMetrics'); const m = {}; for (const x of metrics) m[x.name] = x.value; return m; };
  const win = async () => { const a = await M(); await page.waitForTimeout(3000); const z = await M(); return { task: +((z.TaskDuration - a.TaskDuration) * 1000).toFixed(0), script: +((z.ScriptDuration - a.ScriptDuration) * 1000).toFixed(0) }; };
  const w1 = await win(); await page.waitForTimeout(6000); const w2 = await win();
  // trace 3 s
  const events = [];
  cdp.on('Tracing.dataCollected', p => { for (const e of p.value) events.push(e); });
  const done = new Promise(r => cdp.once('Tracing.tracingComplete', r));
  await cdp.send('Tracing.start', { categories: 'devtools.timeline,v8,blink,cc,gpu,disabled-by-default-devtools.timeline', transferMode: 'ReportEvents' });
  await page.waitForTimeout(3000);
  await cdp.send('Tracing.end'); await done;
  // find the renderer main thread: the thread that has the most 'RunTask' time in the page's process
  const meta = events.filter(e => e.ph === 'M' && e.name === 'thread_name');
  const tname = new Map(meta.map(e => [e.pid + ':' + e.tid, e.args.name]));
  const by = new Map();
  for (const e of events) {
    if (e.ph !== 'X' || !e.dur) continue;
    const t = tname.get(e.pid + ':' + e.tid) || '?';
    if (t !== 'CrRendererMain') continue;
    by.set(e.name, (by.get(e.name) || 0) + e.dur);
  }
  const top = [...by.entries()].sort((a, c) => c[1] - a[1]).slice(0, 15).map(([n, us]) => [n, +(us / 1000).toFixed(1)]);
  const threads = {}; for (const e of events) { if (e.ph !== 'X' || !e.dur) continue; const t = tname.get(e.pid + ':' + e.tid) || '?'; if (e.name === 'ThreadControllerImpl::RunTask' || e.name === 'RunTask') threads[t] = +(((threads[t] || 0) + e.dur / 1000)).toFixed(1); }
  console.log(JSON.stringify({ file: path.basename(file), idleWindow1: w1, idleWindow2: w2, mainThreadTop: top, runTaskMsByThread: threads }));
  await b.close();
})();
