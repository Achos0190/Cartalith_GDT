// Ruling DY on the desktop: modules you arrange, the planner window with Pop out, the assistant's first-use question,
// Records and Generate world in the menus, notes opening in Write with Revert to opened…. Prints what each step found.
const { chromium } = require('playwright');
const OUT = '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/desktop-shell-2026-10-31/';
const BASE = 'http://127.0.0.1:8770/';
(async () => {
  const b = await chromium.launch(); const errs = [];
  const p = await b.newPage({ viewport: { width: 1600, height: 1000 } });
  p.on('pageerror', (e) => errs.push('PAGEERROR ' + e.message + ' | ' + (e.stack || '').split('\n').slice(1, 3).join(' ')));
  p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push('CONSOLE ' + m.text().slice(0, 300)); });
  await p.goto(BASE + 'Desktop.dc.html'); await p.waitForTimeout(3500);
  const panel = p.locator('[aria-label="context panel"][role="complementary"]').first();
  const tabs = async () => (await panel.getByRole('tab').allInnerTexts()).map((x) => x.trim());
  const find = async (q) => { await p.getByLabel('search places and commands').first().click(); await p.waitForTimeout(250); await p.keyboard.type(q); await p.waitForTimeout(300); };
  const pick = async (name) => { await p.getByRole('dialog', { name: 'search' }).getByText(name, { exact: true }).first().click(); await p.waitForTimeout(450); };
  const shot = (n) => p.screenshot({ path: OUT + n });
  // 1 a settlement; the + list
  await find('Vhal Serai'); await pick('Vhal Serai');
  console.log('1 tabs', (await tabs()).join(' · '));
  await panel.getByLabel('add a module to the panel').click(); await p.waitForTimeout(300);
  const add = p.getByRole('dialog', { name: 'add a module' });
  console.log('  + list:', (await add.innerText()).split('\n').filter((x) => x.trim()).length, 'lines ·', (await add.innerText()).split('\n').filter((x) => x.trim()).slice(0, 6).join(' | '));
  await shot('dy-1-add-modules.png');
  await add.getByText('History', { exact: true }).click(); await p.waitForTimeout(250);
  await add.getByText('Economy', { exact: true }).click(); await p.waitForTimeout(250);
  await panel.getByLabel('close the module list').click(); await p.waitForTimeout(200);
  console.log('2 tabs', (await tabs()).join(' · '));
  await panel.getByRole('tab', { name: /^history tab/ }).click(); await p.waitForTimeout(300);
  const body = async () => (await panel.innerText()).split('\n').filter((x) => x.trim());
  console.log('  history:', (await body()).filter((x) => /HOLDERS|EVENTS|FROM ITS NOTE|Founded|Shared with/.test(x)).join(' | '));
  await shot('dy-2-settlement-history.png');
  await panel.getByRole('tab', { name: /^economy tab/ }).click(); await p.waitForTimeout(300);
  console.log('  economy stand-in rows:', (await body()).filter((x) => /stand-in/.test(x)).length, '·', (await body()).filter((x) => /PRODUCES|MARKET|TAX YIELD/.test(x)).join(' | '));
  await shot('dy-3-settlement-economy.png');
  // 3 add Military and Territory from Window ▸ Context panel ▸ MODULES
  for (const name of ['Military', 'Territory']) {
    await p.getByRole('button', { name: 'Window', exact: true }).click(); await p.waitForTimeout(200);
    await p.getByRole('menu', { name: 'WINDOW' }).getByText('Context panel', { exact: true }).nth(1).click(); await p.waitForTimeout(200);
    const col = p.getByRole('menu', { name: 'CONTEXT PANEL' }); if (name === 'Military') console.log('3 menu rows', (await col.innerText()).split('\n').filter((x) => x.trim()).join(' | '));
    await col.getByText(name, { exact: true }).click(); await p.waitForTimeout(250); await p.keyboard.press('Escape');
  }
  console.log('  tabs for a settlement', (await tabs()).join(' · '));
  await find('Sevjuni'); await pick('Sevjuni League');
  console.log('4 tabs for a faction', (await tabs()).join(' · '));
  await panel.getByRole('tab', { name: /^military tab/ }).click(); await p.waitForTimeout(300);
  console.log('  military:', (await body()).filter((x) => /STANDING ARMY|FIELD ARMY|GARRISONS|CONFLICTS/.test(x)).join(' | '));
  await shot('dy-4-faction-military.png');
  await panel.getByRole('tab', { name: /^territory tab/ }).click(); await p.waitForTimeout(300);
  console.log('  territory:', (await body()).filter((x) => /SETTLEMENTS HELD|PROVINCES|CLAIMS|INFLUENCE/.test(x)).join(' | '));
  await shot('dy-5-faction-territory.png');
  // 5 the tab menu; move; open in a window
  await panel.getByRole('tab', { name: /^history tab/ }).click({ button: 'right' }); await p.waitForTimeout(250);
  const tm = p.getByRole('menu', { name: /^tab: / }); console.log('5 tab menu', (await tm.innerText()).split('\n').filter((x) => x.trim()).join(' | '));
  await shot('dy-6-tab-menu.png');
  await tm.getByText('Move right', { exact: true }).click(); await p.waitForTimeout(250); console.log('  after Move right', (await tabs()).join(' · '));
  await panel.getByRole('tab', { name: /^military tab/ }).click({ button: 'right' }); await p.waitForTimeout(250);
  await p.getByRole('menu', { name: /^tab: / }).getByText('Open in a window', { exact: true }).click(); await p.waitForTimeout(400);
  const mw = p.getByRole('dialog', { name: 'MILITARY' }); console.log('  window', await mw.count(), (await mw.innerText()).split('\n').slice(0, 3).join(' | '), '· modal scrims', await p.locator('div[style*="rgba(0,0,0,.45)"]').count());
  await mw.getByLabel('close window').click(); await p.waitForTimeout(200);
  // 6 drag ECONOMY onto INSPECT (pointer)
  const eb = await panel.getByRole('tab', { name: /^economy tab/ }).boundingBox(); const ib = await panel.getByRole('tab', { name: /^inspect tab/ }).boundingBox();
  await p.mouse.move(eb.x + eb.width / 2, eb.y + eb.height / 2); await p.mouse.down(); await p.mouse.move(eb.x - 20, eb.y + eb.height / 2, { steps: 4 }); await p.mouse.move(ib.x + 6, ib.y + ib.height / 2, { steps: 6 }); await p.mouse.up(); await p.waitForTimeout(300);
  console.log('6 after drag', (await tabs()).join(' · '));
  // 7 a culture hides what does not fit
  await panel.getByRole('tab', { name: /^inspect tab/ }).click(); await p.waitForTimeout(200);
  await panel.locator('xpath=.//span[normalize-space()="Faith"]/following-sibling::span//button').first().click(); await p.waitForTimeout(400);
  console.log('7 tabs for a culture', (await tabs()).join(' · '), '· + title:', await panel.getByLabel('add a module to the panel').getAttribute('title'));
  // 8 remove Note, then Ctrl+2 brings it back
  await panel.getByRole('tab', { name: /^note tab/ }).click({ button: 'right' }); await p.waitForTimeout(200);
  await p.getByRole('menu', { name: /^tab: / }).getByText('Remove', { exact: true }).click(); await p.waitForTimeout(200);
  console.log('8 after remove', (await tabs()).join(' · '));
  await p.mouse.click(800, 980); await p.keyboard.press('Control+2'); await p.waitForTimeout(300); console.log('  after Ctrl+2', (await tabs()).join(' · '));
  // 9 menus: File has Generate world after New world…; Records; no World on the desktop bar
  await p.getByRole('button', { name: 'File', exact: true }).click(); await p.waitForTimeout(200);
  console.log('9 File', (await p.getByRole('menu', { name: 'FILE' }).innerText()).split('\n').filter((x) => x.trim()).slice(0, 4).join(' | ')); await p.keyboard.press('Escape');
  await p.getByRole('button', { name: 'Records', exact: true }).click(); await p.waitForTimeout(200);
  const rec = p.getByRole('menu', { name: 'RECORDS' }); console.log('  Records', (await rec.innerText()).split('\n').filter((x) => x.trim()).slice(0, 6).join(' | '), '· has Generate', await rec.getByText('Generate world').count()); await p.keyboard.press('Escape');
  // 10 the planner opens in its own window; the map stays live; Pop out
  await p.getByRole('button', { name: 'Story', exact: true }).click(); await p.waitForTimeout(200); await p.getByRole('menu', { name: 'STORY' }).getByText('Journey planner', { exact: true }).click(); await p.waitForTimeout(500);
  const pw = p.getByRole('dialog', { name: 'JOURNEY PLANNER' }); console.log('10 planner window', await pw.count(), '· modal', await pw.getAttribute('aria-modal'), '· scrims', await p.locator('div[style*="rgba(0,0,0,.45)"]').count());
  await find('Kess Ford'); await pick('Kess Ford'); console.log('  selection in planner', (await pw.innerText()).split('\n').filter((x) => /SELECTION ·|START HERE|END HERE/i.test(x)).join(' | '));
  await pw.getByLabel(/^pop out/).click(); await p.waitForTimeout(300); console.log('  popped', (await pw.innerText()).split('\n').filter((x) => /Popped out|DOCK/.test(x)).join(' | '));
  await shot('dy-7-planner-window.png');
  await pw.getByLabel('close window').click(); await p.waitForTimeout(200);
  // 11 Ask: the first-use question, then the header says where the conversation lives
  await p.keyboard.press('Control+3'); await p.waitForTimeout(300);
  await panel.getByRole('button', { name: /^Describe / }).first().click(); await p.waitForTimeout(2600);
  const q = p.getByRole('group', { name: 'save this conversation with the world?' }); console.log('11 first-use card', await q.count(), '· header', await panel.getByText('session only').count());
  await shot('dy-8-ask-first-use.png');
  await q.getByRole('button', { name: 'Save with the world' }).click(); await p.waitForTimeout(300);
  console.log('  after yes: card', await q.count(), '· header', await panel.getByText('saved with the world').count());
  // 12 the panel's Note tab opens in Write
  await find('Vhal Serai'); await pick('Vhal Serai'); await p.keyboard.press('Control+2'); await p.waitForTimeout(400);
  console.log('12 note tab: textarea', await panel.locator('textarea.md-ta').count(), '· WRITE pressed', await panel.getByRole('button', { name: 'WRITE' }).getAttribute('aria-pressed'));
  // 13 the vault opens a note in Write; Revert to opened… asks and Ctrl+Z brings it back
  await p.keyboard.press('Control+Shift+V'); await p.waitForTimeout(500);
  const vault = p.getByRole('dialog', { name: 'vault' }); await vault.getByText('Vhal Serai', { exact: true }).first().click(); await p.waitForTimeout(500);
  const ta = vault.locator('textarea.md-ta'); console.log('13 vault: textarea', await ta.count(), '· Write pressed', await vault.getByLabel('Write view').getAttribute('style').then((st) => /var\(--acc\)/.test(st)));
  const t0 = await ta.inputValue(); await ta.click(); await p.keyboard.press('Control+End'); await p.keyboard.type('\nA line added this session.'); await p.waitForTimeout(1500);
  await vault.getByLabel('Revert to opened…').click(); await p.waitForTimeout(300);
  const cf = p.getByRole('alertdialog', { name: 'REVERT TO OPENED' }); console.log('  confirm', await cf.count(), (await cf.innerText()).split('\n').filter((x) => x.trim()).slice(1, 2).join(' '));
  await shot('dy-9-vault-revert.png');
  await cf.getByRole('button', { name: 'REVERT' }).click(); await p.waitForTimeout(400);
  console.log('  reverted to the opened text', (await ta.inputValue()) === t0);
  await ta.click(); await p.keyboard.press('Control+z'); await p.waitForTimeout(300); console.log('  Ctrl+Z brings the edit back', /A line added this session\./.test(await ta.inputValue()));
  // 14 Ctrl+click on a link in Write follows it
  const title = () => vault.locator('span[style*="600 16px"]').first().innerText();
  const firstLink = await ta.evaluate((el) => { const i = el.value.indexOf('[['); const j = el.value.indexOf(']]', i); return el.value.slice(i + 2, j).split('|')[0]; });
  console.log('14 before Ctrl+click: title', await title(), '· first link', firstLink);
  await ta.evaluate((el) => { const i = el.value.indexOf('[['); el.focus(); el.setSelectionRange(i + 3, i + 3); el.dispatchEvent(new MouseEvent('click', { bubbles: true, ctrlKey: true })); });
  await p.waitForTimeout(500); console.log('  after Ctrl+click: title', await title(), '· create card', await p.getByRole('dialog', { name: /^NEW NOTE/ }).count(), '· mode stays Write', await vault.locator('textarea.md-ta').count());
  await shot('dy-10-vault-write.png');
  await b.close(); console.log(errs.slice(0, 12).join('\n') || 'no page errors');
})();
