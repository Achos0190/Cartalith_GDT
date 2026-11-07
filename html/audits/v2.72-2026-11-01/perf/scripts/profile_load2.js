// Load attribution, both versions: CPU profile from navigation to load+1.2 s, reported as inclusive
// ms for each top-level script block, the boot renderNow, _dskInit, and the profile's
// (program)/(garbage collector)/(idle) buckets, plus the boot render's own PERF.render split.
// Usage: node profile_load2.js <file.html>
const path = require('path');
const { chromium } = require('playwright');
const agg = require('./profagg.js');
(async () => {
  const file = process.argv[2];
  const html = require('fs').readFileSync(file, 'utf8').split('\n');
  const starts = []; html.forEach((l, i) => { if (/^<script>/.test(l)) starts.push(i + 1); });
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage();
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Profiler.enable'); await cdp.send('Profiler.setSamplingInterval', { interval: 100 }); await cdp.send('Profiler.start');
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(1200);
  const { profile } = await cdp.send('Profiler.stop');
  const perf = await page.evaluate(() => PERF.render);
  const a = agg(profile, 400);
  const T = new Map(a.total), S = new Map(a.self);
  const out = { file: path.basename(file), bootRender: perf, scripts: {} };
  starts.forEach((ln, k) => { out.scripts['block' + (k + 1)] = T.get('(anon):' + ln) || 0; });
  for (const k of ['(program):0', '(garbage collector):0', '(idle):0']) out[k] = S.get(k) || 0;
  for (const [k, v] of a.total) if (/^(renderNow|_dskInit|_dskPlace|_syncChromeTop|GENPOOL\.init|init|_civBuildTimelineUI|_carPopulateIconGallery|allocate|updateReadout|maxGrade|buildWaterBodies|buildRoutingSurface|seaColor|smoothSeaH|seaShadeFrom):/.test(k)) out[k] = Math.max(out[k] || 0, v);
  console.log(JSON.stringify(out));
  await b.close();
})();
