const { chromium } = require('playwright');
(async () => {
  const S = process.argv[2];
  const b = await chromium.launch(); const ctx = await b.newContext();
  const p1 = await ctx.newPage(); await p1.goto('file://' + S + '/ls_a/a.html');
  const w = await p1.evaluate(() => { try { localStorage.setItem('probe_key', 'secret-from-a'); return location.origin; } catch (e) { return 'throw:' + e.message; } });
  const p2 = await ctx.newPage(); await p2.goto('file://' + S + '/ls_b/b.html');
  const r = await p2.evaluate(() => { try { return [location.origin, localStorage.getItem('probe_key')]; } catch (e) { return ['throw', e.message]; } });
  console.log(JSON.stringify({ writerOrigin: w, readerOriginAndValue: r, chromium: b.version() }));
  await b.close();
})();
