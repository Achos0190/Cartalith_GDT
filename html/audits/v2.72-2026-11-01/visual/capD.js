// Phase D: the design board (Desktop.dc.html) at its four window sizes in dark, 1600 in light, and a 2x
// capture of the 1366 frame (shown at 100 %) for detail crops. Also dumps the frame's controls.
const L = require('./lib.js');
const { log, shot, save, clipShot } = L;
const URL = 'http://127.0.0.1:8770/Desktop.dc.html';
async function open(b, dpr) {
  const ctx = await b.newContext({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: dpr || 1 });
  const p = await ctx.newPage(); p._errs = [];
  p.on('pageerror', e => p._errs.push('pageerror ' + e.message));
  p.on('console', m => { if (m.type() === 'error') p._errs.push('console ' + m.text().slice(0, 160)); });
  await p.goto(URL); await p.waitForTimeout(3500);
  const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click().catch(() => {});
  return p;
}
async function pick(p, label) { const c = p.locator('button.dk-chip', { hasText: label }); await c.first().click(); await p.waitForTimeout(900); }
(async () => {
  const b = await L.chromium.launch();
  const p = await open(b, 1);
  for (const [lab, w] of [['1366', 1366], ['1600', 1600], ['1920', 1920], ['2560', 2560]]) { await pick(p, lab + ' ×'); await shot(p, `d_frame_${w}_dark`); }
  await pick(p, '1600 ×'); await pick(p, 'THEME'); await shot(p, 'd_frame_1600_light'); await pick(p, 'THEME');
  // inventory of the frame's buttons (text, aria, box) at 1600, so later steps can press them by name
  const inv = await p.evaluate(() => [...document.querySelectorAll('button,[role=button],input,select')].map(e => { const r = e.getBoundingClientRect(); return { t: (e.innerText || e.value || '').trim().slice(0, 40), a: e.getAttribute('aria-label'), x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) }; }).filter(o => o.w > 0));
  save('d_inventory_1600', inv);
  log('design errors:', p._errs.length ? p._errs.join(' | ') : 'none');
  await p.close();
  const q = await open(b, 2);
  await pick(q, '1366 ×'); await shot(q, 'd_frame_1366_dark_2x');
  await q.close();
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
