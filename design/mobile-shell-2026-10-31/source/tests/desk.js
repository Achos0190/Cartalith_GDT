// Drives the desktop shell: one selection locked across Inspect/Note/Ask and windows, one year, menus, keys.
const { chromium } = require('playwright');
// Diagnostic captures go to SHOT_DIR, or the system temp folder: they are working images, not evidence.
const OUT = process.env.SHOT_DIR || (require('os').tmpdir() + '/');
const BASE = 'http://127.0.0.1:8770/';
(async () => {
  const b = await chromium.launch(); const errs = [];
  const p = await b.newPage({ viewport: { width: 1600, height: 1000 } });
  p.on('pageerror', (e) => errs.push('PAGEERROR ' + e.message + ' | ' + (e.stack || '').split('\n').slice(1, 3).join(' ')));
  p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push('CONSOLE ' + m.text().slice(0, 300)); });
  await p.goto(BASE + 'Desktop.dc.html'); await p.waitForTimeout(3500);
  const head = async () => (await p.locator('[aria-label="context panel"][role="complementary"]').first().innerText()).split('\n').slice(0, 4).join(' | ');
  const setYear = async (y) => { await p.locator('input[aria-label="timeline year"]').evaluate((el, v) => { const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; set.call(el, String(v)); el.dispatchEvent(new Event('input', { bubbles: true })); }, y); await p.waitForTimeout(250); };
  const find = async (q) => { await p.getByLabel('search places and commands').first().click(); await p.waitForTimeout(250); await p.keyboard.type(q); await p.waitForTimeout(300); };
  const pick = async (name) => { await p.getByRole('dialog', { name: 'search' }).getByText(name, { exact: true }).first().click(); await p.waitForTimeout(450); };
  // 1 select a settlement through Find
  await find('Vhal Serai'); await pick('Vhal Serai');
  console.log('1 header:', await head());
  console.log('  held-by row', await p.getByText('Held by', { exact: true }).count(), 'history rows', await p.getByText(/POLITICAL HISTORY/).count());
  await p.screenshot({ path: OUT + 'k_1_inspect.png' });
  // 2 Note tab follows; chronos row marked for the year
  await p.getByRole('tab', { name: /note tab/ }).click(); await p.waitForTimeout(500);
  console.log('2 note opens in Write (VQ4)', await p.locator('[aria-label="context panel"] textarea.md-ta').count());
  await p.locator('[aria-label="context panel"][role="complementary"]').getByRole('button', { name: 'PREVIEW' }).click(); await p.waitForTimeout(400);
  const md = p.locator('[aria-label="context panel"] .md').first();
  console.log('2 note: headings', await md.locator('h2,h3').count(), 'chronos rows', await md.locator('.md-chr-row').count(), 'now-marked', await md.locator('.md-chr-row.now').count());
  const nowY1 = await md.locator('.md-chr-row.now').first().getAttribute('data-year').catch(() => null);
  await setYear(0); const nowY2 = await md.locator('.md-chr-row.now').first().getAttribute('data-year').catch(() => null);
  console.log('  year 412 marks', nowY1, '· year 0 marks', nowY2);
  await p.screenshot({ path: OUT + 'k_2_note.png' });
  // 3 Inspect at year 0 vs founding
  await p.getByRole('tab', { name: /inspect tab/ }).click(); await p.waitForTimeout(300);
  console.log('3 inspect at year 0:', await p.getByText(/^AT YEAR 0$/).count(), (await p.locator('[aria-label="context panel"]').first().innerText()).split('\n').slice(4, 12).join(' | '));
  await setYear(412);
  // 4 Ask: chips follow, prompt, citations
  await p.getByRole('tab', { name: /ask tab/ }).click(); await p.waitForTimeout(400);
  const chips = await p.locator('[aria-label="context sent with the next message"] button').allInnerTexts(); console.log('4 chips', chips.map((x) => x.trim()));
  await p.getByRole('button', { name: /^Describe Vhal Serai$/ }).click(); await p.waitForTimeout(2600);
  const cites = await p.locator('.ask-cite').allInnerTexts(); console.log('  cites', cites.slice(0, 8));
  console.log('  action cards', await p.getByRole('button', { name: 'Apply' }).count(), 'tools line', await p.getByText(/^used · /).count());
  await p.screenshot({ path: OUT + 'k_3_ask.png' });
  // 5 Apply the note card; Note shows it
  await p.getByRole('button', { name: 'Apply' }).first().click(); await p.waitForTimeout(400);
  await p.getByRole('tab', { name: /note tab/ }).click(); await p.waitForTimeout(500);
  console.log('5 note has assistant section', await p.locator('[aria-label="context panel"] .md h2', { hasText: 'Description (assistant)' }).count());
  // 6 a citation selects the faction; Ask chips follow
  await p.getByRole('tab', { name: /ask tab/ }).click(); await p.waitForTimeout(300);
  const facCite = p.locator('.ask-cite').filter({ hasText: /Orentamwash|Sevjuni League|Korrath|Mirzafir Reach/ }).first(); const facName = (await facCite.innerText()).trim(); await facCite.click(); await p.waitForTimeout(500);
  console.log('6 after cite', facName, '→', await head());
  console.log('  chips now', (await p.locator('[aria-label="context sent with the next message"] button').allInnerTexts()).map((x) => x.trim()));
  // 7 Alt+Left goes back to the place
  await p.mouse.click(800, 980); await p.keyboard.press('Alt+ArrowLeft'); await p.waitForTimeout(400); console.log('7 after Alt+←', await head());
  // 8 pin, then select something else: banner
  await p.getByRole('button', { name: /^pin: keep this panel/ }).click(); await p.waitForTimeout(200);
  await find('Kess Ford'); await pick('Kess Ford');
  console.log('8 pinned header', await head(), '· banner', await p.getByText(/selection is now/).count());
  await p.screenshot({ path: OUT + 'k_4_pinned.png' });
  await p.getByRole('button', { name: 'FOLLOW', exact: true }).click(); await p.waitForTimeout(300); console.log('  after follow', await head());
  // 9 menus: cascade columns
  await p.getByRole('button', { name: 'Window', exact: true }).click(); await p.waitForTimeout(250);
  await p.getByRole('menu', { name: 'WINDOW' }).getByText('Context panel', { exact: true }).nth(1).click().catch(async () => { await p.getByRole('menu', { name: 'WINDOW' }).getByText('Context panel').last().click(); }); await p.waitForTimeout(250);
  console.log('9 menu columns', await p.locator('[role="menubar"] [role="menu"]').count());
  await p.screenshot({ path: OUT + 'k_5_menu.png' });
  await p.keyboard.press('Escape'); await p.waitForTimeout(200);
  await p.getByRole('button', { name: 'File', exact: true }).click(); await p.waitForTimeout(250); console.log('  File has Generate world', await p.getByRole('menu', { name: 'FILE' }).getByText('Generate world').count()); await p.keyboard.press('Escape');
  await p.getByRole('button', { name: 'Records', exact: true }).click(); await p.waitForTimeout(250); console.log('  Records has Generate world', await p.getByRole('menu', { name: 'RECORDS' }).getByText('Generate world').count(), '· a World menu on the desktop bar', await p.getByRole('button', { name: 'World', exact: true }).count()); await p.keyboard.press('Escape');
  // 10 factions window follows the selection, non-modal
  await p.keyboard.press('Control+Shift+F'); await p.waitForTimeout(500);
  const dlg = p.getByRole('dialog').first(); console.log('10 window', (await dlg.innerText()).split('\n').slice(0, 3).join(' | '), '· modal scrim', await p.locator('div[style*="rgba(0,0,0,.45)"]').count());
  await find('Sevjuni'); await pick('Sevjuni');
  console.log('  after selecting Sevjuni:', (await dlg.innerText()).split('\n').slice(0, 2).join(' | '));
  await p.screenshot({ path: OUT + 'k_6_window.png' });
  await p.keyboard.press('Escape'); await p.waitForTimeout(200);
  // 11 conflicts: pick → selection + year
  await p.getByRole('button', { name: 'Story', exact: true }).click(); await p.waitForTimeout(250); await p.getByRole('menu', { name: 'STORY' }).getByText('Conflicts…').click(); await p.waitForTimeout(400);
  await p.getByRole('dialog').getByText('Sevjuni campaign', { exact: true }).click(); await p.waitForTimeout(500);
  console.log('11 after conflict pick', await head(), '· year', await p.locator('input[aria-label="timeline year"]').inputValue());
  await p.screenshot({ path: OUT + 'k_7_conflict.png' });
  await p.keyboard.press('Control+r'); await p.waitForTimeout(700);
  const busy = p.getByRole('alertdialog', { name: 'generating the world' }); console.log('12 busy overlay', await busy.count(), (await busy.innerText().catch(() => '')).split('\n').slice(0, 3).join(' | '));
  await p.screenshot({ path: OUT + 'k_8_busy.png' });
  await p.keyboard.press('Control+1'); await p.waitForTimeout(200);
  for (let i = 0; i < 40 && await busy.count(); i++) await p.waitForTimeout(500);
  console.log('   overlay gone after the run', (await busy.count()) === 0);
  await b.close(); console.log(errs.slice(0, 12).join('\n') || 'no page errors');
})();
