// Ruling DY on touch: the narrow tablet bar (File · World · Records · Story) measured at four widths; the tablet's
// panel modules at touch size; notes in Write with the Open › chip; the phone's MENU, vault and assistant question.
const { chromium } = require('playwright');
const OUT = '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/desktop-shell-2026-10-31/';
const BASE = 'http://127.0.0.1:8770/';
const bb = async (loc) => { const b = await loc.boundingBox(); return b ? { x: Math.round(b.x), w: Math.round(b.width), h: Math.round(b.height) } : null; };
(async () => {
  const b = await chromium.launch(); const errs = [];
  const watch = (p, tag) => { p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message)); p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); }); };
  // ---------- the narrow tablet bar ----------
  for (const [w, h] of [[600, 960], [768, 1024], [1024, 768], [1179, 820]]) {
    const p = await b.newPage({ viewport: { width: w, height: h } }); watch(p, 'bar' + w);
    await p.goto(BASE + 'Live.dc.html'); await p.waitForTimeout(2800); const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
    const bar = await p.evaluate(() => {
      const host = document.querySelector('x-dc') || document.body; const all = []; const walk = (n) => { if (!n) return; if (n.shadowRoot) walk(n.shadowRoot); for (const k of n.children || []) { all.push(k); walk(k); } }; walk(document.body);
      const btn = (lab) => all.find((e) => e.tagName === 'BUTTON' && (e.getAttribute('aria-label') === lab || (e.getAttribute('aria-haspopup') === 'menu' && e.textContent.trim() === lab)));
      const r = (e) => { if (!e) return null; const q = e.getBoundingClientRect(); return [Math.round(q.left), Math.round(q.right)]; };
      const brand = all.find((e) => e.tagName === 'SPAN' && e.textContent.trim() === 'CARTALITH');
      return { brand: r(brand), ham: r(btn('more menus')), File: r(btn('File')), World: r(btn('World')), Records: r(btn('Records')), Story: r(btn('Story')), undo: r(all.find((e) => e.tagName === 'BUTTON' && /^undo /.test(e.getAttribute('aria-label') || ''))), redo: r(all.find((e) => e.tagName === 'BUTTON' && /^redo /.test(e.getAttribute('aria-label') || ''))), search: r(btn('search places and commands')), vw: innerWidth };
    });
    const menusEnd = bar.Story ? bar.Story[1] : null; const rightStart = bar.undo ? bar.undo[0] : null;
    console.log('bar ' + w + '×' + h, JSON.stringify(bar), '· gap between Story and undo', rightStart - menusEnd, 'px', rightStart - menusEnd >= 8 ? 'fits' : 'OVERLAPS');
    if (w === 600 || w === 1024) await p.screenshot({ path: OUT + 'dy-11-tablet-bar-' + w + '.png', clip: { x: 0, y: 0, width: w, height: 120 } });
    if (w === 600) {
      await p.getByRole('button', { name: 'World', exact: true }).click(); await p.waitForTimeout(300);
      console.log('  World menu', (await p.getByRole('menu').first().innerText()).split('\n').filter((x) => x.trim()).join(' | '));
      await p.screenshot({ path: OUT + 'dy-12-tablet-world-menu-600.png' });
      await p.mouse.click(w / 2, h - 200); await p.waitForTimeout(200); await p.getByRole('button', { name: 'Records', exact: true }).click(); await p.waitForTimeout(300);
      console.log('  Records menu', (await p.getByRole('menu').first().innerText()).split('\n').filter((x) => x.trim()).slice(0, 8).join(' | '));
      await p.mouse.click(w / 2, h - 200); await p.waitForTimeout(200); await p.getByRole('button', { name: 'File', exact: true }).click(); await p.waitForTimeout(300);
      console.log('  File menu has Generate world', await p.getByRole('menu').first().getByText('Generate world').count());
    }
    await p.close();
  }
  // ---------- the tablet: modules at touch size, notes in Write, the Open › chip ----------
  const t = await b.newPage({ viewport: { width: 1194, height: 834 } }); watch(t, 'tablet');
  await t.goto(BASE + 'Tablet.dc.html'); await t.waitForTimeout(3000); const tip = t.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
  await t.getByLabel('search places and commands').first().click(); await t.waitForTimeout(250); await t.keyboard.type('Vhal Serai'); await t.waitForTimeout(300); await t.getByRole('dialog', { name: 'search' }).getByText('Vhal Serai', { exact: true }).first().click(); await t.waitForTimeout(500);
  const panel = t.locator('[aria-label="context panel"][role="complementary"]').first();
  console.log('tablet + button', JSON.stringify(await bb(panel.getByLabel('add a module to the panel'))), '· tab', JSON.stringify(await bb(panel.getByRole('tab').first())));
  await panel.getByRole('tab', { name: /^note tab/ }).click({ button: 'right' }); await t.waitForTimeout(250);
  const tm = t.getByRole('menu', { name: /^tab: / }); console.log('tablet tab menu', (await tm.innerText()).split('\n').filter((x) => x.trim()).join(' | '));
  const rowsH = await tm.getByRole('button').evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().height))); console.log('  tab menu row heights', rowsH.join(','));
  await t.keyboard.press('Escape'); await t.waitForTimeout(150);
  await panel.getByRole('tab', { name: /^note tab/ }).click(); await t.waitForTimeout(400);
  const dta = panel.locator('textarea.md-ta'); console.log('tablet note tab textarea', await dta.count());
  await dta.evaluate((el) => { const i = el.value.indexOf('[['); el.focus(); el.setSelectionRange(i + 3, i + 3); el.dispatchEvent(new MouseEvent('click', { bubbles: true })); }); await t.waitForTimeout(300);
  const chip = panel.getByRole('button', { name: /^Open .* ›$/ }); console.log('  Open › chip', await chip.count(), JSON.stringify(await bb(chip.first())), (await chip.first().innerText().catch(() => '')));
  await t.screenshot({ path: OUT + 'dy-13-tablet-note-open-chip.png' });
  await chip.first().click(); await t.waitForTimeout(500); console.log('  after chip: panel header', (await panel.innerText()).split('\n').slice(0, 3).join(' | '));
  await b.close().catch(() => {});
  // ---------- the phone: MENU, the vault in Write, the assistant's first-use question ----------
  const b2 = await chromium.launch(); const p = await b2.newPage({ viewport: { width: 412, height: 892 } }); watch(p, 'phone');
  await p.goto(BASE + 'Shell.dc.html'); await p.waitForTimeout(3000); const tip2 = p.getByLabel('dismiss tip'); if (await tip2.count()) await tip2.first().click();
  const cell = async (id) => { const r = await p.locator('[data-navcell="' + id + '"]').boundingBox(); await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await p.waitForTimeout(450); };
  await cell('menu'); console.log('phone MENU', (await p.locator('body').innerText()).split('\n').filter((x) => /^(File|Edit|Assets|Records|World|Story|Preferences|Window|Help)$/.test(x.trim())).join(' · '));
  await p.getByText('File', { exact: true }).first().click(); await p.waitForTimeout(350);
  const fileRows = (await p.locator('body').innerText()).split('\n').filter((x) => x.trim()); const iNew = fileRows.indexOf('New world…'); console.log('  File rows after New world…:', fileRows.slice(iNew, iNew + 3).join(' | '));
  await p.screenshot({ path: OUT + 'dy-14-phone-file.png' });
  // the vault: a note opens in Write; the caret in a link shows Open ›
  await p.getByLabel('open the quick drawer: pinned commands').first().click(); await p.waitForTimeout(300); await p.getByText('Vault', { exact: true }).first().click(); await p.waitForTimeout(500);
  await p.getByText('Vhal Serai', { exact: true }).first().click(); await p.waitForTimeout(500);
  const pta = p.locator('textarea.md-ta'); console.log('phone vault: textarea', await pta.count(), '· toolbar', await p.getByRole('toolbar', { name: 'formatting' }).count());
  await pta.evaluate((el) => { const i = el.value.indexOf('[['); el.focus(); el.setSelectionRange(i + 3, i + 3); el.dispatchEvent(new Event('select', { bubbles: true })); el.dispatchEvent(new MouseEvent('click', { bubbles: true })); }); await p.waitForTimeout(300);
  const pchip = p.getByRole('button', { name: /^open the link at the caret/ }); console.log('  Open › chip', await pchip.count(), JSON.stringify(await bb(pchip.first())));
  await p.screenshot({ path: OUT + 'dy-15-phone-vault-write.png' });
  await pchip.first().click(); await p.waitForTimeout(400); console.log('  after chip: create card', await p.getByRole('dialog', { name: /^NEW NOTE/ }).count());
  await b2.close();
  console.log(errs.slice(0, 10).join('\n') || 'no page errors');
})();
