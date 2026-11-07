// Ruling DY, the rest: every desktop size; Here, Links and Recent & bookmarks; the phone assistant's first-use
// question; and a 44 dp audit of the new touch controls on the tablet.
const { chromium } = require('playwright');
const OUT = '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/desktop-shell-2026-10-31/';
const BASE = 'http://127.0.0.1:8770/';
const audit = async (p, tag, scope) => {
  const bad = await p.evaluate((sc) => {
    const out = []; const all = []; const walk = (n) => { if (!n) return; if (n.shadowRoot) walk(n.shadowRoot); for (const k of n.children || []) { all.push(k); walk(k); } }; walk(document.body);
    const root = sc ? all.find((e) => e.getAttribute && e.getAttribute('aria-label') === sc) : null; const set = root ? all.filter((e) => root.contains(e)) : all;
    for (const el of set) {
      if (!(el.tagName === 'BUTTON' || el.tagName === 'INPUT' || (el.getAttribute && el.getAttribute('role') === 'tab'))) continue;
      const r = el.getBoundingClientRect(); if (!r.width || !r.height) continue; if (r.right < 0 || r.bottom < 0 || r.left > innerWidth || r.top > innerHeight) continue;
      const cs = getComputedStyle(el); if (cs.visibility === 'hidden' || cs.display === 'none') continue;
      if (r.width < 44 || r.height < 44) out.push([(el.getAttribute('aria-label') || el.textContent || el.tagName).trim().slice(0, 36), Math.round(r.width), Math.round(r.height)]);
    }
    return out;
  }, scope || null);
  console.log(tag, bad.length ? JSON.stringify(bad.slice(0, 12)) : 'all targets >= 44 x 44');
};
(async () => {
  const b = await chromium.launch(); const errs = [];
  const watch = (p, tag) => { p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message)); p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); }); };
  // ---------- every desktop size ----------
  const d = await b.newPage({ viewport: { width: 1600, height: 1000 } }); watch(d, 'desk');
  await d.goto(BASE + 'Desktop.dc.html'); await d.waitForTimeout(3500);
  const panel = d.locator('[aria-label="context panel"][role="complementary"]').first();
  const tabs = async () => (await panel.getByRole('tab').allInnerTexts()).map((x) => x.trim()).join(' · ');
  const find = async (q) => { await d.getByLabel('search places and commands').first().click(); await d.waitForTimeout(250); await d.keyboard.type(q); await d.waitForTimeout(300); };
  const pick = async (name) => { await d.getByRole('dialog', { name: 'search' }).getByText(name, { exact: true }).first().click(); await d.waitForTimeout(450); };
  for (const size of ['1366 × 768', '1920 × 1080', '2560 × 1440', '1600 × 900']) {
    await d.getByRole('button', { name: size }).click(); await d.waitForTimeout(700);
    await find('Vhal Serai'); await pick('Vhal Serai');
    await panel.getByLabel('add a module to the panel').click(); await d.waitForTimeout(250);
    const add = d.getByRole('dialog', { name: 'add a module' }); const box = await add.boundingBox(); const pb = await panel.boundingBox();
    if (size === '1366 × 768') for (const name of ['Links', 'Here', 'Recent & bookmarks']) { await add.getByText(name, { exact: true }).click(); await d.waitForTimeout(150); }
    console.log(size, '· + list inside the panel', box && pb ? (box.x >= pb.x - 1 && box.x + box.width <= pb.x + pb.width + 1 && box.y + box.height <= pb.y + pb.height + 1) : 'n/a', '· tabs', await tabs());
    await panel.getByLabel('close the module list').click(); await d.waitForTimeout(150);
  }
  // Links: linked and unlinked mentions, one-click linking as one undo step
  await panel.getByRole('tab', { name: /^links tab/ }).click(); await d.waitForTimeout(300);
  const lines = async () => (await panel.innerText()).split('\n').filter((x) => x.trim());
  console.log('links module', (await lines()).filter((x) => /LINKED MENTIONS|UNLINKED MENTIONS|Link it in|No note links|already a link/.test(x)).join(' | '));
  for (const cand of ['Grey Vale', 'Kess Ford', 'Thornwood', 'Lakemouth', 'Sevjuni League', 'Korrath', 'Orentamwash']) { if (await panel.getByRole('button', { name: /^Link it in / }).count()) break; await find(cand); const hit = d.getByRole('dialog', { name: 'search' }).getByText(cand, { exact: true }); if (!(await hit.count())) { await d.keyboard.press('Escape'); continue; } await hit.first().click(); await d.waitForTimeout(400); await panel.getByRole('tab', { name: /^links tab/ }).click().catch(() => {}); await d.waitForTimeout(250); console.log('  links for', cand, (await lines()).map((x, i, a) => (/^(LINKED MENTIONS|UNLINKED MENTIONS)$/.test(x) ? x + ' ' + a[i + 1] : null)).filter(Boolean).join(' | ')); }
  const linkBtn = panel.getByRole('button', { name: /^Link it in / }).first();
  if (await linkBtn.count()) { const lab = await linkBtn.innerText(); await linkBtn.click(); await d.waitForTimeout(300); console.log('  after', lab.trim(), '→', (await lines()).map((x, i, a) => (/^(LINKED MENTIONS|UNLINKED MENTIONS)$/.test(x) ? x + ' ' + a[i + 1] : null)).filter(Boolean).join(' | ')); await d.mouse.click(800, 990); await d.keyboard.press('Control+z'); await d.waitForTimeout(300); console.log('  after Ctrl+Z', (await lines()).map((x, i, a) => (/^(LINKED MENTIONS|UNLINKED MENTIONS)$/.test(x) ? x + ' ' + a[i + 1] : null)).filter(Boolean).join(' | ')); }
  await d.screenshot({ path: OUT + 'dy-16-links-module.png' });
  // Here for a settlement, then with nothing selected
  await panel.getByRole('tab', { name: /^here tab/ }).click(); await d.waitForTimeout(250);
  console.log('here (settlement)', (await lines()).filter((x) => /THE GROUND UNDER|ELEVATION|BIOME/.test(x)).join(' | '));
  await panel.getByRole('tab', { name: /^recent & bookmarks tab/ }).click(); await d.waitForTimeout(250);
  await panel.getByRole('button', { name: /^☆ Bookmark / }).click(); await d.waitForTimeout(250);
  console.log('recent', (await lines()).filter((x) => /BOOKMARK|RECENT|Vhal Serai|Sevjuni/.test(x)).slice(0, 6).join(' | '));
  await d.screenshot({ path: OUT + 'dy-17-recent-bookmarks.png' });
  await d.mouse.click(800, 990); await d.keyboard.press('Control+d'); await d.waitForTimeout(300);
  console.log('nothing selected · tabs', await tabs());
  await panel.getByRole('tab', { name: /^here tab/ }).click(); await d.waitForTimeout(250);
  console.log('  here (none)', (await lines()).filter((x) => /sample pin|HERE/.test(x)).join(' | '));
  // ---------- the tablet at touch size: the + list and the first-use card ----------
  const t = await b.newPage({ viewport: { width: 1194, height: 834 } }); watch(t, 'tablet');
  await t.goto(BASE + 'Tablet.dc.html'); await t.waitForTimeout(3000); const tip = t.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
  await t.getByLabel('search places and commands').first().click(); await t.waitForTimeout(250); await t.keyboard.type('Vhal Serai'); await t.waitForTimeout(300); await t.getByRole('dialog', { name: 'search' }).getByText('Vhal Serai', { exact: true }).first().click(); await t.waitForTimeout(500);
  const tp = t.locator('[aria-label="context panel"][role="complementary"]').first();
  await tp.getByLabel('add a module to the panel').click(); await t.waitForTimeout(300); await audit(t, 'tablet + list', 'add a module');
  await t.screenshot({ path: OUT + 'dy-18-tablet-add-modules.png' });
  await tp.getByLabel('close the module list').click(); await t.waitForTimeout(200);
  await audit(t, 'tablet panel', 'context panel');
  // ---------- the phone assistant: the same first-use question ----------
  const p = await b.newPage({ viewport: { width: 412, height: 892 } }); watch(p, 'phone');
  await p.goto(BASE + 'Shell.dc.html'); await p.waitForTimeout(3000); const tip2 = p.getByLabel('dismiss tip'); if (await tip2.count()) await tip2.first().click();
  const cell = async (id) => { const r = await p.locator('[data-navcell="' + id + '"]').boundingBox(); await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await p.waitForTimeout(450); };
  await cell('menu'); await p.getByText('Preferences', { exact: true }).first().click(); await p.waitForTimeout(350);
  console.log('phone Preferences menu has no save row of its own (expect 0; the switch is in Assistant…)', await p.getByText('Save the assistant conversation with the world', { exact: true }).count());
  await p.getByText('Assistant…', { exact: true }).first().click(); await p.waitForTimeout(400);
  await p.getByText('Assistant enabled', { exact: true }).first().click(); await p.waitForTimeout(200);
  console.log('  assistant settings has the save row', await p.getByText('Save the conversation with the world', { exact: true }).count());
  await p.getByRole('dialog').getByText('Save', { exact: true }).last().click(); await p.waitForTimeout(300);
  await cell('story'); await p.getByText('NOTES', { exact: true }).first().click(); await p.waitForTimeout(300); await p.getByText('Assistant panel…', { exact: true }).first().click(); await p.waitForTimeout(450);
  const dlg = p.getByRole('dialog').last(); console.log('phone assistant window', (await dlg.innerText()).split('\n').slice(0, 2).join(' | '));
  await dlg.getByText('Send', { exact: true }).click(); await p.waitForTimeout(300);
  console.log('  first-use question', await dlg.getByText(/^Save this conversation with the world\?/).count(), '· buttons', await dlg.getByText('Save with the world', { exact: true }).count());
  await p.screenshot({ path: OUT + 'dy-19-phone-assistant-first-use.png' });
  await dlg.getByText('Not now', { exact: true }).click(); await p.waitForTimeout(300); console.log('  after Not now', await dlg.getByText(/^Save this conversation with the world\?/).count());
  await audit(p, 'phone assistant');
  await b.close(); console.log(errs.slice(0, 10).join('\n') || 'no page errors');
})();
