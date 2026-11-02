// Phase D3: the design frame's surfaces that v2.72 has counterparts for, captured by clicking inside the
// 1600 frame: a menu open, a submenu, the timeline expanded, CIVIL's dock, a selected place, Sculpt armed.
const L = require('./lib.js');
const { log, shot, save } = L;
(async () => {
  const b = await L.chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1600, height: 1000 } });
  const p = await ctx.newPage(); const errs = [];
  p.on('pageerror', e => errs.push(e.message)); p.on('console', m => { if (m.type() === 'error') errs.push(m.text().slice(0, 150)); });
  await p.goto('http://127.0.0.1:8770/Desktop.dc.html'); await p.waitForTimeout(3500);
  const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click().catch(() => {});
  const byText = (t) => p.locator('button', { hasText: t }).first();
  const step = async (n, fn) => { try { await fn(); } catch (e) { log('D3 failed', n, e.message.split('\n')[0]); } };
  await step('file menu', async () => { await byText('File').click(); await p.waitForTimeout(400); await shot(p, 'd3_menu_file'); });
  await step('file export sub', async () => { const r = p.locator('text=Export').first(); await r.hover(); await p.waitForTimeout(400); await shot(p, 'd3_menu_file_export'); await p.keyboard.press('Escape'); await p.keyboard.press('Escape'); await p.waitForTimeout(200); });
  await step('prefs menu', async () => { await byText('Preferences').click(); await p.waitForTimeout(400); await shot(p, 'd3_menu_prefs'); await p.keyboard.press('Escape'); await p.waitForTimeout(200); });
  await step('timeline expanded', async () => { await byText('TIMELINE').click(); await p.waitForTimeout(500); await shot(p, 'd3_timeline_expanded'); await byText('TIMELINE').click(); await p.waitForTimeout(300); });
  await step('civil', async () => { await p.locator('[aria-label="CIVIL domain"]').click(); await p.waitForTimeout(600); await shot(p, 'd3_civil'); });
  await step('select place', async () => { const r = await p.evaluate(() => { const t = [...document.querySelectorAll('*')].find(e => e.childElementCount === 0 && (e.innerText || '').trim() === 'Grey Vale'); if (!t) return null; const q = t.getBoundingClientRect(); return { x: q.left - 8, y: q.top + q.height / 2 }; }); if (r) { await p.mouse.click(r.x, r.y); await p.waitForTimeout(700); } await shot(p, 'd3_civil_selected'); });
  await step('world sculpt', async () => { await p.locator('[aria-label="WORLD domain"]').click(); await p.waitForTimeout(400); await p.locator('[aria-label="WORLD ▸ Sculpt"]').click(); await p.waitForTimeout(600); await shot(p, 'd3_world_sculpt'); });
  log('D3 errors:', errs.length ? errs.join(' | ') : 'none');
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
