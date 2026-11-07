// Drives the vault and its editor on the phone and the tablet; prints what each step found and every page error.
const { chromium } = require('playwright');
// Diagnostic captures go to SHOT_DIR, or the system temp folder: they are working images, not evidence.
const OUT = process.env.SHOT_DIR || (require('os').tmpdir() + '/');
const BASE = 'http://127.0.0.1:8770/';
(async () => {
  const b = await chromium.launch(); const errs = [];
  const watch = (p) => { p.on('pageerror', (e) => errs.push('PAGEERROR ' + e.message)); p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push('CONSOLE ' + m.text()); }); };
  const p = await b.newPage({ viewport: { width: 412, height: 892 } }); watch(p);
  await p.goto(BASE + 'Shell.dc.html'); await p.waitForTimeout(3000);
  const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
  // open the vault through the quick drawer's Vault pin
  await p.getByLabel('open the quick drawer: pinned commands').first().click(); await p.waitForTimeout(300);
  await p.getByText('Vault', { exact: true }).first().click(); await p.waitForTimeout(500);
  console.log('vault list groups', await p.getByText(/^SETTLEMENTS$|^FACTIONS$|^NOTES$|^LANDMARKS$|^CULTURES & FAITHS$/).count());
  await p.screenshot({ path: OUT + 'v_p1_list.png' });
  await p.getByText('Vhal Serai', { exact: true }).first().click(); await p.waitForTimeout(500);
  console.log('opens in Write (VQ4)', await p.locator('textarea.md-ta').count());
  await p.getByLabel('Preview view').first().click(); await p.waitForTimeout(400);
  const md = p.locator('.md').first();
  console.log('preview: headings', await md.locator('h2,h3').count(), 'links', await md.locator('[data-link]').count(), 'tasks', await md.locator('[data-task]').count(), 'chronos rows', await md.locator('.md-chr-row').count());
  await p.screenshot({ path: OUT + 'v_p2_note.png' });
  // follow a link, then back
  await md.locator('[data-link="The Old Rite"]').first().click(); await p.waitForTimeout(400);
  console.log('followed to', await p.locator('text=The Old Rite').count() > 0, 'title now', await p.locator('span', { hasText: /^The Old Rite$/ }).count());
  await p.getByLabel('back').first().click(); await p.waitForTimeout(400);
  // long-press a link: peek
  const lk = p.locator('.md [data-link]').first(); const bb = await lk.boundingBox();
  await p.mouse.move(bb.x + 5, bb.y + bb.height / 2); await p.mouse.down(); await p.waitForTimeout(700); await p.mouse.up(); await p.waitForTimeout(250);
  console.log('peek open', await p.getByRole('dialog', { name: 'link preview' }).count());
  await p.screenshot({ path: OUT + 'v_p3_peek.png' });
  await p.mouse.click(5, 5); await p.waitForTimeout(250);
  // task toggle in preview
  const t0 = await p.locator('.md [data-task]').first().getAttribute('aria-checked');
  await p.locator('.md [data-task]').first().click(); await p.waitForTimeout(300);
  console.log('task toggled', t0, '->', await p.locator('.md [data-task]').first().getAttribute('aria-checked'));
  // chronos row moves the timeline
  await p.locator('.md-chr-row').nth(2).click(); await p.waitForTimeout(300);
  console.log('chronos toast', await p.getByText(/Timeline moved to year/).count());
  // write mode, [[ autocomplete
  await p.getByLabel('Write view').first().click(); await p.waitForTimeout(400);
  const ta = p.locator('textarea.md-ta'); console.log('textarea', await ta.count());
  await ta.click(); await p.keyboard.press('Control+End'); await p.keyboard.type('\nSee also [[Gre'); await p.waitForTimeout(300);
  console.log('suggestions', await p.getByRole('listbox', { name: 'link suggestions' }).count(), await p.locator('[role=listbox] button').allInnerTexts().then((x) => x.slice(0, 3).map((s) => s.split('\n')[0])));
  await p.screenshot({ path: OUT + 'v_p4_ac.png' });
  await p.keyboard.press('Enter'); await p.waitForTimeout(300);
  const val = await ta.inputValue(); console.log('inserted link', /See also \[\[Grey Vale\]\]/.test(val), JSON.stringify(val.slice(-30)));
  // toolbar bold on a selection
  await p.keyboard.type(' salt'); await p.keyboard.press('Shift+ArrowLeft'); await p.keyboard.press('Shift+ArrowLeft'); await p.keyboard.press('Shift+ArrowLeft'); await p.keyboard.press('Shift+ArrowLeft');
  await p.getByLabel('Bold (Ctrl+B)').first().click(); await p.waitForTimeout(250);
  console.log('bold', /\*\*salt\*\*/.test(await ta.inputValue()));
  // list continuation
  await p.keyboard.press('Control+End'); await p.keyboard.type('\n- [ ] first'); await p.keyboard.press('Enter'); await p.keyboard.type('second'); await p.waitForTimeout(200);
  console.log('list continues', /- \[ \] first\n- \[ \] second/.test(await ta.inputValue()));
  await p.waitForTimeout(1500); console.log('autosaved', await p.getByText(/^saved/).count());
  await p.screenshot({ path: OUT + 'v_p5_write.png' });
  await p.getByLabel('done: preview the note').first().click(); await p.waitForTimeout(300);
  console.log('new link resolves', await p.locator('.md [data-link="Grey Vale"]').count());
  // tablet
  const t = await b.newPage({ viewport: { width: 1194, height: 834 } }); watch(t);
  await t.goto(BASE + 'Tablet.dc.html'); await t.waitForTimeout(3000);
  const tip2 = t.getByLabel('dismiss tip'); if (await tip2.count()) await tip2.first().click();
  await t.getByLabel('Vault (pinned)').first().click(); await t.waitForTimeout(500);
  await t.getByText('The Old Rite', { exact: true }).first().click(); await t.waitForTimeout(500);
  console.log('tablet: opens in Write (VQ4)', await t.locator('textarea.md-ta').count());
  await t.getByLabel('Preview view').first().click(); await t.waitForTimeout(400);
  console.log('tablet: panel tabs', await t.getByText(/^Links \d+$/).count(), 'linked mentions', await t.getByText(/^LINKED MENTIONS · \d+$/).first().innerText());
  await t.screenshot({ path: OUT + 'v_t1_read.png' });
  await t.getByLabel('Split view').first().click(); await t.waitForTimeout(500);
  console.log('split: textarea', await t.locator('textarea.md-ta').count(), 'preview', await t.locator('.md').count());
  await t.screenshot({ path: OUT + 'v_t2_split.png' });
  await b.close();
  console.log(errs.slice(0, 12).join('\n') || 'no page errors');
})();
