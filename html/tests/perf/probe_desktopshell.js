/* v2.72 — the desktop shell, stage 1 of 3: menu bar, tool-options bar, rail, left dock, Context panel,
 * timeline strip, status bar and map controls, for windows wider than 860px. A phone (<=860px) keeps
 * v2.71's layout until stage 2, and this probe checks that too.
 *
 * Owner: "Can we upgrade the layout in the html with the new design?" — then "A working copy in this
 * repo" and "Everything, staged". Design: Cartalith_GDT PC_SHELL_PROPOSAL.md §2–§3.12, Rulings DY, DZ.
 *
 * Every claim here is a rendered geometry or a REAL click (page.click / page.mouse / page.keyboard), never
 * a function call standing in for one: a control can exist, be wired and be unreachable (v2.46), and
 * nothing about a shell is visible to tests/run.sh. Where a probe step does call into the page, it is to
 * SET UP state (a world, a settlement, a recorded year), or to install a spy on a heavy action so the
 * click can be counted without running an 8K export — never to navigate.
 *
 * Usage: node tests/perf/probe_desktopshell.js "Cartalith v2.72 DCC test.html" ["Cartalith v2.71 DCC test.html"]
 *   The optional second file is the BEFORE build (v2.71). With it, the phone section compares the drawer,
 *   the rail and the map at 390x760 and 412x915 against that build's own geometry, rather than only
 *   checking invariants. Any other argument is refused (exit 2), never ignored.
 * Env: PLAYWRIGHT_DIR, CHROME_BIN — as every probe in this folder.
 * Output: one "ok   - …" or "FAIL - …" line per assertion, then a count. Exit status 1 if anything failed.
 */
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_DIR || '/opt/node22/lib/node_modules/playwright');

const FILE = process.argv[2], BEFORE = process.argv[3];
if (!FILE || process.argv.length > 4 || !fs.existsSync(FILE) || (BEFORE && !fs.existsSync(BEFORE))) {
  console.error('usage: node probe_desktopshell.js <v2.72.html> [<v2.71.html>]  — got: ' + JSON.stringify(process.argv.slice(2)));
  process.exit(2);
}
let pass = 0, fail = 0;
const A = (n, c, extra) => { if (c) { pass++; console.log('ok   - ' + n); }
  else { fail++; console.log('FAIL - ' + n + (extra !== undefined ? '  [' + extra + ']' : '')); } };
const errors = [];
const sleep = ms => new Promise(r => setTimeout(r, ms));
/* a section that throws (an element this build does not have, say) fails ONCE, by name, and the next
   section still runs — so a run against v2.71 counts its failures section by section */
const section = async (name, fn) => { console.log('\n--- ' + name + ' ---');
  try { await fn(); } catch (e) { A(name + ': the section ran to the end', false, String((e && e.message) || e).split('\n')[0]); } };

const boot = async (b, file, vp, mobile) => {
  const p = await b.newPage({ viewport: vp, isMobile: !!mobile, hasTouch: !!mobile });
  p.on('pageerror', e => errors.push(path.basename(file) + ' @' + vp.width + ': ' + e.message));
  p.on('dialog', d => { p._dialogs = (p._dialogs || []).concat(d.message()); d.dismiss().catch(() => {}); });
  await p.goto('file://' + path.resolve(file));
  await p.waitForFunction(() => typeof window.generate === 'function', { timeout: 60000 });
  return p;
};
/* SET UP only: a small world, the gate dismissed the way a generated world dismisses it */
const gen = (p, seed) => p.evaluate(async (seed) => {
  state.tect.seed = seed; state.resW = 256;
  GW = state.resW; GH = gridH(GW); allocate();
  if (typeof _setupHide === 'function') _setupHide();
  await generate(); syncUI(); if (typeof _viewFill === 'function') _viewFill(); renderNow();
}, seed || 12345);
/* a counter on generate() itself: every Generate path ends in withBusy(…, generate), and the global
   binding is what that call reads, so wrapping it counts real runs without changing what they do */
const spyGenerate = p => p.evaluate(() => { if (window.__genSpy) return; window.__genSpy = true; window.__gens = 0;
  const orig = window.generate; window.generate = async function () { window.__gens++; return orig.apply(this, arguments); }; });
const settle = async p => { await p.evaluate(() => (typeof _busyChain !== 'undefined' ? _busyChain : Promise.resolve())); await sleep(120); };
const vis = (p, sel) => p.evaluate(sel => { const e = document.querySelector(sel); if (!e) return false;
  const r = e.getBoundingClientRect(); const cs = getComputedStyle(e);
  return r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && r.right > 0 && r.bottom > 0 && r.left < innerWidth && r.top < innerHeight; }, sel);
const tryClick = async (p, sel) => { try { await p.click(sel, { timeout: 3000 }); return true; } catch (_) { return false; } };
const openMenu = async (p, btnSel) => { await p.evaluate(() => { if (typeof _dskMenuClose === 'function') _dskMenuClose(); }); return tryClick(p, btnSel); };
/* open a menu by its button and press one of its rows, the way a person does; `sub` hovers a submenu row first */
const row = async (p, btnSel, cmd, sub) => {
  if (!(await openMenu(p, btnSel))) return false;
  if (sub) { try { await p.hover(sub, { timeout: 2000 }); } catch (_) { return false; } await sleep(80); }
  return tryClick(p, '[data-cmd="' + cmd + '"]:not(#pinSave):not(#pinPlanner)');
};

(async () => {
  const b = await chromium.launch({ executablePath: process.env.CHROME_BIN, args: ['--no-sandbox'] });
  try {
  /* ════════════════════════════ BEFORE ANY WORLD EXISTS ════════════════════════════ */
  await section('the start card, and a readout that shows only what exists', async () => {
    const q = await boot(b, FILE, { width: 1600, height: 900 });
    await sleep(500);
    const G = await q.evaluate(() => ({ gated: document.body.classList.contains('setup-gated'), ro: (document.getElementById('docReadout') || {}).textContent }));
    A('a fresh load opens the start card', G.gated);
    A('...and the menu bar\'s document readout says "no world" — not the boot seed', G.ro === 'no world', JSON.stringify(G.ro));
    A('the start card\'s Generate a world still opens the New World card', (await tryClick(q, '#obGenerate')) && (await vis(q, '#suGenCommit')));
    await tryClick(q, '#suGenBack'); await tryClick(q, '#obSkip'); await sleep(150);
    A('Skip for now leaves an empty app that still says "no world"', await q.evaluate(() => !document.body.classList.contains('setup-gated') && document.getElementById('docReadout').textContent === 'no world'));
    await q.close();
  });

  /* ════════════════════════════ DESKTOP, 1600 x 900 ════════════════════════════ */
  const p = await boot(b, FILE, { width: 1600, height: 900 });
  await gen(p); await spyGenerate(p);

  await section('the menu bar', async () => {
    const R = await p.evaluate(() => ({
      menus: [...document.querySelectorAll('.mb-btn')].filter(b => b.getClientRects().length).map(b => b.textContent.replace(/▾/g, '').trim()),
      inHeader: [...document.querySelectorAll('.mb-btn')].every(b => !!b.closest('header')),
      ver: (document.getElementById('verTag') || {}).textContent, title: document.title,
      word: (document.querySelector('header h1') || {}).innerText }));
    A('eight menus, in the design\'s order', JSON.stringify(R.menus) === JSON.stringify(['File', 'Edit', 'Assets', 'Records', 'Story', 'Preferences', 'Window', 'Help']), JSON.stringify(R.menus));
    A('...all in the header bar', R.inHeader && R.menus.length === 8);
    A('the wordmark reads CARTALITH', (R.word || '').trim().toUpperCase() === 'CARTALITH', JSON.stringify(R.word));
    A('the version is v2.72 (the tag and the title)', R.ver === 'v2.72' && /v2\.72/.test(R.title), R.ver + ' / ' + R.title);
    const ro = await p.evaluate(() => (document.getElementById('docReadout') || {}).textContent);
    A('with a world open, the document readout shows its seed, grid and extent', /^seed 12345 · 256×\d+ · (region|whole world)$/.test(ro || ''), JSON.stringify(ro));
    const menuBtns = ['#fileMenuBtn', '#mbEditBtn', '#mbAssetsBtn', '#mbRecordsBtn', '#mbStoryBtn', '#mbPrefsBtn', '#mbWindowBtn', '#mbHelpBtn'];
    let allOpen = true, allInside = true, oneLine = true, keysRight = true, why = '';
    for (const s of menuBtns) {
      const opened = await openMenu(p, s); await sleep(60);
      const M = await p.evaluate(s => { const b = document.querySelector(s), pn = b && document.getElementById(b.getAttribute('aria-controls'));
        if (!pn) return null; const r = pn.getBoundingClientRect();
        const rows = [...pn.querySelectorAll('.mb-row,.mb-panel > button')].filter(e => e.getClientRects().length && !e.closest('.mb-subpanel'));
        return { open: pn.classList.contains('open') && r.height > 0, inside: r.left >= 0 && r.right <= innerWidth && r.bottom <= innerHeight,
          tall: rows.filter(e => e.getBoundingClientRect().height > 34).map(e => e.textContent.trim()),
          off: [...pn.querySelectorAll('.mb-k')].filter(k => k.getClientRects().length).filter(k => { const rr = k.closest('button').getBoundingClientRect(), kr = k.getBoundingClientRect(); return rr.right - kr.right > 20; }).map(k => k.textContent) }; }, s);
      if (!opened || !M || !M.open) { allOpen = false; why += s + ' '; continue; }
      if (!M.inside) allInside = false; if (M.tall.length) { oneLine = false; why += M.tall.join(',') + ' '; } if (M.off.length) keysRight = false;
    }
    A('each menu opens on a real click', allOpen, why);
    A('...and draws inside the window', allOpen && allInside);
    A('one line per row', allOpen && oneLine, why);
    A('accelerators sit at the right edge of their row', allOpen && keysRight);
    await openMenu(p, '#fileMenuBtn'); await p.hover('#mbEditBtn', { timeout: 3000 }); await sleep(80);
    const H = await p.evaluate(() => ({ edit: document.getElementById('mbEdit').classList.contains('open'), file: document.getElementById('fileMenu').classList.contains('open') }));
    A('while a menu is open, moving across the bar switches menus', H.edit && !H.file, JSON.stringify(H));
    await p.keyboard.press('Escape'); await sleep(60);
    A('Esc closes it', await p.evaluate(() => !document.querySelector('.mb-panel.open')));
    await openMenu(p, '#mbHelpBtn'); await p.mouse.click(800, 500); await sleep(60);
    A('an outside click closes it', await p.evaluate(() => !document.querySelector('.mb-panel.open')));
    await p.focus('#fileMenuBtn'); await p.keyboard.press('ArrowDown'); await sleep(60);
    const K1 = await p.evaluate(() => { const a = document.activeElement; return { open: document.getElementById('fileMenu').classList.contains('open'), inMenu: !!(a && a.closest('#fileMenu')), outline: a ? getComputedStyle(a).outlineStyle + ' ' + getComputedStyle(a).outlineWidth : '' }; });
    A('the keyboard opens a menu and lands on its first row', K1.open && K1.inMenu, JSON.stringify(K1));
    A('...with the focus visibly outlined', /solid/.test(K1.outline) && !/ 0px/.test(K1.outline), K1.outline);
    await p.keyboard.press('ArrowRight'); await sleep(60);
    A('ArrowRight moves to the next menu', await p.evaluate(() => document.getElementById('mbEdit').classList.contains('open') && !!document.activeElement.closest('#mbEdit')));
    await p.keyboard.press('Escape'); await sleep(40);
    A('Esc returns the focus to the menu\'s own button', await p.evaluate(() => document.activeElement && document.activeElement.id === 'mbEditBtn'));
    await openMenu(p, '#fileMenuBtn'); await p.hover('#mbExportRow', { timeout: 3000 }); await sleep(120);
    const S = await p.evaluate(() => { const m = document.getElementById('fileMenu').getBoundingClientRect(), r = document.getElementById('mbExportRow').getBoundingClientRect(), sp = document.getElementById('mbExportPanel'), q = sp && sp.getBoundingClientRect(), br = document.getElementById('bakeRes');
      return { open: !!(q && q.height > 0), beside: !!q && q.left >= m.right - 3, inside: !!q && q.top >= 0 && q.bottom <= innerHeight && q.right <= innerWidth, form: !!(br && sp.contains(br) && br.getBoundingClientRect().height > 0) }; });
    A('a submenu opens beside its row (File ▸ Export)', S.open && S.beside, JSON.stringify(S));
    A('...inside the window', S.inside);
    A('...holding v0.87\'s export form, ids kept (#bakeRes)', S.form);
    await p.selectOption('#bakeRes', '4096'); await sleep(60);
    A('changing the form does not close the menu (v0.87\'s contract)', await p.evaluate(() => document.getElementById('fileMenu').classList.contains('open')));
    await p.selectOption('#bakeRes', '8192');
    await p.keyboard.press('Escape'); await p.keyboard.press('Escape');
  });

  await section('every wired row does what it says', async () => {
    const gens = () => p.evaluate(() => window.__gens);
    for (const dom of ['world', 'civ', 'carto', 'explore']) {
      await tryClick(p, '#domainRail [data-domain="' + dom + '"]');
      let g0 = await gens(); await row(p, '#fileMenuBtn', 'generate'); await settle(p);
      A('File ▸ Generate world generates — in ' + dom.toUpperCase(), (await gens()) === g0 + 1, g0 + '→' + (await gens()));
      g0 = await gens(); await p.evaluate(() => { window.__noReload = 1; });
      await p.keyboard.press('Control+r'); await settle(p);
      const KR = await p.evaluate(() => ({ g: window.__gens, still: window.__noReload === 1 }));
      A('Ctrl+R generates — in ' + dom.toUpperCase() + ', and the page did not reload', KR.g === g0 + 1 && KR.still, JSON.stringify(KR));
    }
    await tryClick(p, '#domainRail [data-domain="world"]');
    const inDock = id => p.evaluate(id => !!document.getElementById(id).closest('.dock-left #runSec'), id);
    let g0 = await gens(); const clicked = await tryClick(p, '#genBtn'); await settle(p);
    A('the dock\'s own GENERATE WORLD button generates (a real click, in Run)', clicked && (await inDock('genBtn')) && (await gens()) === g0 + 1);
    g0 = await gens(); const ng = await tryClick(p, '#reseedBtn'); await settle(p);
    A('NEW SEED is clickable in the dock and generates', ng && (await inDock('reseedBtn')) && (await gens()) === g0 + 1);

    let chooser = p.waitForEvent('filechooser', { timeout: 3000 }).then(() => true, () => false);
    await openMenu(p, '#fileMenuBtn'); await tryClick(p, '#loadZipBtn');
    A('File ▸ Load project .zip… opens the file picker', await chooser);
    chooser = p.waitForEvent('filechooser', { timeout: 3000 }).then(() => true, () => false);
    await p.keyboard.press('Control+o');
    A('Ctrl+O opens the same picker', await chooser);
    /* a spy on the export button, so pressing it does not bake an 8K map */
    await p.evaluate(() => { window.__exports = 0; document.getElementById('exportBtn').onclick = () => { window.__exports++; }; });
    await row(p, '#fileMenuBtn', 'save'); await sleep(60);
    A('File ▸ Save project presses the project export (#exportBtn)', await p.evaluate(() => window.__exports === 1));
    await p.keyboard.press('Control+s'); await sleep(60);
    A('Ctrl+S does the same', await p.evaluate(() => window.__exports === 2));
    await tryClick(p, '#pinSave'); await sleep(60);
    A('the Save pin does the same', await p.evaluate(() => window.__exports === 3));
    const as0 = await p.evaluate(() => document.getElementById('autosaveChk').checked);
    await row(p, '#fileMenuBtn', 'autosave'); await sleep(60);
    A('File ▸ Autosave toggles the real autosave box', await p.evaluate(a => document.getElementById('autosaveChk').checked === !a, as0));
    await row(p, '#fileMenuBtn', 'autosave');
    await row(p, '#fileMenuBtn', 'regionWorld'); await sleep(300);
    A('File ▸ New world from selection… opens Region export, where Select region and Extract as new world live', (await p.evaluate(() => document.getElementById('settingsModal').classList.contains('open') && document.querySelector('.set-page.on').dataset.setpage === 'storage' && document.getElementById('genRegionSec').open)) && (await vis(p, '#regionNewWorldBtn')) && (await vis(p, '#regionBtn')));
    await tryClick(p, '#settingsClose'); await sleep(80);

    await p.evaluate(() => { pushUndo(); updateUndoUI(); });
    const u0 = await p.evaluate(() => undoStack.length);
    await row(p, '#mbEditBtn', 'undo'); await sleep(200);
    A('Edit ▸ Undo undoes (the stack shrinks)', await p.evaluate(u => undoStack.length === u - 1 && redoStack.length > 0, u0));
    const st1 = await p.evaluate(() => [undoStack.length, redoStack.length]);
    await row(p, '#mbEditBtn', 'redo'); await sleep(200);
    A('Edit ▸ Redo redoes (a step moves back from redo to undo)', await p.evaluate(st => undoStack.length === st[0] + 1 && redoStack.length === st[1] - 1, st1), JSON.stringify(st1));
    await openMenu(p, '#mbEditBtn');
    A('Edit ▸ Delete selected place is disabled with nothing selected', await p.evaluate(() => document.querySelector('[data-cmd="deletePlace"]').disabled));
    await p.keyboard.press('Escape');
    await p.evaluate(() => { const q = { x: 5, y: 5, name: 'Probe Delete', kind: 'hamlet', faction: 0, pop: 10, traits: [] }; state.places.push(q); _civSelectedPlace = q; _civRenderPlaceEditor(); });
    await row(p, '#mbEditBtn', 'deletePlace'); await sleep(80);
    A('...and deletes the selected place when there is one (the Delete key\'s own code)', await p.evaluate(() => !state.places.some(q => q.name === 'Probe Delete') && !_civSelectedPlace));
    await row(p, '#mbEditBtn', 'find'); await sleep(60);
    A('Edit ▸ Find… puts the caret in the search box', await p.evaluate(() => document.activeElement && document.activeElement.id === 'findInput'));
    await p.keyboard.press('Escape');

    await row(p, '#mbAssetsBtn', 'assetLibrary'); await sleep(250);
    A('Assets ▸ Asset library enters the library', await p.evaluate(() => document.body.classList.contains('assets-mode')));
    A('...and the Assets pin, now "← Map", stays visible and clickable in the bar', (await vis(p, '#assetsHeaderBtn')) && /Map/.test(await p.evaluate(() => document.getElementById('assetsHeaderBtn').textContent)));
    await openMenu(p, '#mbAssetsBtn');
    A('...the menu row reads checked while the library is open', await p.evaluate(() => document.querySelector('[data-cmd="assetLibrary"]').getAttribute('aria-pressed') === 'true'));
    await p.keyboard.press('Escape');
    await tryClick(p, '#assetsHeaderBtn'); await sleep(250);
    A('the pin takes you back to the map', await p.evaluate(() => !document.body.classList.contains('assets-mode')));
    await p.keyboard.press('Shift+A'); await sleep(250);
    A('Shift+A is the same toggle', await p.evaluate(() => document.body.classList.contains('assets-mode')));
    await p.keyboard.press('Shift+A'); await sleep(250);
    await row(p, '#mbAssetsBtn', 'slicer'); await sleep(250);
    A('Assets ▸ Sprite sheet slicer… opens the slicer (in the library)', await p.evaluate(() => { const m = document.getElementById('alSliceModal'); return document.body.classList.contains('assets-mode') && m && m.style.display !== 'none'; }));
    await p.evaluate(() => { document.getElementById('alSliceModal').style.display = 'none'; });
    chooser = p.waitForEvent('filechooser', { timeout: 3000 }).then(() => true, () => false);
    await row(p, '#mbAssetsBtn', 'alImport');
    A('Assets ▸ Open pack in library… opens the pack picker', await chooser);
    p._dialogs = [];
    await row(p, '#mbAssetsBtn', 'alClear'); await sleep(150);
    A('Assets ▸ Clear library… asks first (the library\'s own confirm)', (p._dialogs || []).some(m => /Clear the entire asset library/.test(m)), JSON.stringify(p._dialogs));
    await row(p, '#mbAssetsBtn', 'alApply'); await sleep(250);
    A('Assets ▸ Apply library to map answers in the library (its own toast)', await p.evaluate(() => /empty|Pack applied|Compiling/i.test(document.getElementById('alToast').textContent)));
    await tryClick(p, '#assetsHeaderBtn'); await sleep(200);

    await row(p, '#mbRecordsBtn', 'factions'); await sleep(250);
    A('Records ▸ Factions… opens the Settlements screen on Factions', await p.evaluate(() => document.getElementById('civFactionsModal').classList.contains('open') && getComputedStyle(document.getElementById('civPmFactions')).display !== 'none'));
    await tryClick(p, '#cfmCloseBtn'); await sleep(120);
    await row(p, '#mbRecordsBtn', 'places'); await sleep(250);
    A('Records ▸ Settlements… opens it on Settlements', await p.evaluate(() => document.getElementById('civFactionsModal').classList.contains('open') && getComputedStyle(document.getElementById('civPmPlaces')).display !== 'none'));
    await tryClick(p, '#cfmCloseBtn'); await sleep(120);
    await p.keyboard.press('Control+Shift+F'); await sleep(250);
    A('Ctrl+Shift+F opens Factions', await p.evaluate(() => document.getElementById('civFactionsModal').classList.contains('open')));
    await tryClick(p, '#cfmCloseBtn'); await sleep(120);

    await row(p, '#mbStoryBtn', 'timeline'); await sleep(120);
    A('Story ▸ Timeline opens the strip\'s second row', await p.evaluate(() => !document.getElementById('tlStripMore').hidden));
    await row(p, '#mbStoryBtn', 'timeline'); await sleep(120);
    A('...and pressing it again closes that row', await p.evaluate(() => document.getElementById('tlStripMore').hidden));
    await row(p, '#mbStoryBtn', 'simulate'); await sleep(300);
    A('Story ▸ Simulate collapse / recovery… reveals the simulator (Explore, opened, on screen)', (await p.evaluate(() => _domain === 'explore')) && (await vis(p, '#civSimulateBtn')));
    await tryClick(p, '#domainRail [data-domain="world"]');
    await row(p, '#mbStoryBtn', 'planner'); await sleep(300);
    A('Story ▸ Journey planner… with no journey selected shows Explore ▸ Journeys (never a silent no-op)', (await p.evaluate(() => _domain === 'explore')) && (await vis(p, '#civJourneyList')));
    await tryClick(p, '#domainRail [data-domain="world"]');
    await row(p, '#mbStoryBtn', 'journeys'); await sleep(300);
    A('Story ▸ Saved journeys shows the Journeys list', (await p.evaluate(() => _domain === 'explore')) && (await vis(p, '#civJourneyList')));
    await p.evaluate(() => { window.__plannerCalls = 0; const o = window._civOpenRouteEditor; window._civOpenRouteEditor = function () { window.__plannerCalls++; return true; }; window.__restorePlanner = () => { window._civOpenRouteEditor = o; }; });
    await tryClick(p, '#pinPlanner'); await sleep(60);
    A('the Planner pin runs the planner editor\'s own opener (the action behind #jpOpenEditorBtn)', await p.evaluate(() => window.__plannerCalls === 1));
    await p.evaluate(() => window.__restorePlanner());

    const gpu = await p.evaluate(() => ({ dis: document.getElementById('gpuToggle').disabled, on: document.getElementById('gpuToggle').checked }));
    await openMenu(p, '#mbPrefsBtn');
    A('Preferences ▸ GPU acceleration follows the real box (enabled ⇔ enabled, checked ⇔ checked)', await p.evaluate(g => { const r = document.querySelector('[data-cmd="gpu"]'); return r.disabled === g.dis && (r.getAttribute('aria-pressed') === 'true') === g.on; }, gpu));
    await p.keyboard.press('Escape');
    for (const [cmd, page] of [['setPerf', 'perf'], ['setStorage', 'storage'], ['setAppearance', 'appearance']]) {
      await row(p, '#mbPrefsBtn', cmd); await sleep(150);
      A('Preferences ▸ ' + page + ' opens Settings on that page', await p.evaluate(pg => document.getElementById('settingsModal').classList.contains('open') && document.querySelector('.set-page.on').dataset.setpage === pg, page));
      await tryClick(p, '#settingsClose'); await sleep(80);
    }
    await row(p, '#mbPrefsBtn', 'themeLight', '#mbPrefs .mb-sub:has([data-cmd="themeLight"]) > [data-sub]'); await sleep(80);
    A('Preferences ▸ Theme ▸ Light switches the theme (#themeToggleBtn\'s own handler)', await p.evaluate(() => document.documentElement.getAttribute('data-theme') === 'light' && /Light/.test(document.getElementById('themeToggleBtn').textContent)));
    await row(p, '#mbPrefsBtn', 'themeDark', '#mbPrefs .mb-sub:has([data-cmd="themeLight"]) > [data-sub]'); await sleep(80);
    A('...and Dark switches it back', await p.evaluate(() => document.documentElement.getAttribute('data-theme') !== 'light'));
    await row(p, '#mbPrefsBtn', 'unitsMi', '#mbPrefs .mb-sub:has([data-cmd="unitsMi"]) > [data-sub]'); await sleep(80);
    A('Preferences ▸ Units ▸ Miles switches the display unit (#calUnitSeg)', await p.evaluate(() => _units === 'mi' && document.querySelector('#calUnitSeg [data-unit="mi"]').classList.contains('on')));
    await row(p, '#mbPrefsBtn', 'unitsKm', '#mbPrefs .mb-sub:has([data-cmd="unitsMi"]) > [data-sub]'); await sleep(80);
    A('...and Kilometres back', await p.evaluate(() => _units === 'km'));

    const W0 = await p.evaluate(() => ({ dl: document.querySelector('.dock-left').getBoundingClientRect().width, dr: document.querySelector('.dock-right').getBoundingClientRect().width }));
    await row(p, '#mbWindowBtn', 'winLeftDock'); await sleep(150);
    A('Window ▸ Left dock collapses the dock to its strip', await p.evaluate(() => document.querySelector('.dock-left').getBoundingClientRect().width <= 32));
    await tryClick(p, '#dockLeftCollapse'); await sleep(150);
    A('...and the chevron left on that strip brings it back (one toggle)', await p.evaluate(w => Math.abs(document.querySelector('.dock-left').getBoundingClientRect().width - w) < 2, W0.dl));
    await row(p, '#mbWindowBtn', 'winContext'); await sleep(150);
    A('Window ▸ Context panel collapses the Context panel', await p.evaluate(() => document.querySelector('.dock-right').getBoundingClientRect().width <= 32));
    await row(p, '#mbWindowBtn', 'winContext'); await sleep(150);
    A('...and shows it again', await p.evaluate(w => Math.abs(document.querySelector('.dock-right').getBoundingClientRect().width - w) < 2, W0.dr));
    const mapH0 = await p.evaluate(() => document.querySelector('.canvas-wrap').getBoundingClientRect().height);
    await row(p, '#mbWindowBtn', 'winTimeline'); await sleep(250);
    A('Window ▸ Timeline hides the strip, and the map takes the room (refit)', await p.evaluate(h => !document.getElementById('tlStrip').getClientRects().length && document.querySelector('.canvas-wrap').getBoundingClientRect().height > h, mapH0));
    await row(p, '#mbWindowBtn', 'winStatus'); await sleep(120);
    A('Window ▸ Status bar hides the status bar', await p.evaluate(() => !document.getElementById('statusBar').getClientRects().length));
    await row(p, '#mbWindowBtn', 'winRail'); await sleep(120);
    A('Window ▸ Domain rail hides the rail', await p.evaluate(() => !document.getElementById('domainRail').getClientRects().length));
    await row(p, '#mbWindowBtn', 'resetLayout'); await sleep(250);
    A('Window ▸ Reset layout brings all of them back', await p.evaluate(() => ['tlStrip', 'statusBar', 'domainRail'].every(id => document.getElementById(id).getClientRects().length)));
    await row(p, '#mbWindowBtn', 'diagnostics'); await sleep(120);
    A('Window ▸ Diagnostics overlay shows the Shift+D overlay', await p.evaluate(() => document.getElementById('resOverlay').classList.contains('visible')));
    await p.keyboard.press('Shift+D'); await sleep(120);
    A('...and Shift+D still toggles it (the same function)', await p.evaluate(() => !document.getElementById('resOverlay').classList.contains('visible')));
    for (const [cmd, d] of [['domCiv', 'civ'], ['domCarto', 'carto'], ['domExplore', 'explore'], ['domWorld', 'world']]) {
      await row(p, '#mbWindowBtn', cmd, '#mbWindow .mb-sub [data-sub]'); await sleep(120);
      A('Window ▸ Workspace ▸ ' + d + ' switches the domain (the rail\'s own button)', await p.evaluate(d => _domain === d && document.querySelector('#domainRail [data-domain="' + d + '"]').classList.contains('on'), d));
    }
    await row(p, '#mbWindowBtn', 'view3d'); await sleep(700);
    A('Window ▸ 3D view enters the 3D view (the 2D/3D switch\'s own button)', await p.evaluate(() => _view3dOn === true));
    await tryClick(p, '#viewDimSeg [data-dim="2d"]'); await sleep(300);
    A('...and the map\'s own 2D/3D square, drawn as "2D" there, brings the flat map back', await p.evaluate(() => _view3dOn === false));
    await row(p, '#mbHelpBtn', 'credits'); await sleep(120);
    A('Help ▸ Credits opens the credits', await p.evaluate(() => document.getElementById('creditsModal').classList.contains('open')));
    await tryClick(p, '#creditsClose'); await sleep(80);
    await row(p, '#mbHelpBtn', 'genInfo'); await sleep(150);
    A('Help ▸ Generation info… opens the parameter dump (Settings ▸ Workspace)', await p.evaluate(() => document.getElementById('settingsModal').classList.contains('open') && document.querySelector('.set-page.on').dataset.setpage === 'workspace'));
    await tryClick(p, '#settingsClose'); await sleep(80);
    await openMenu(p, '#mbHelpBtn');
    A('Help carries the version', await p.evaluate(() => /v2\.72/.test(document.getElementById('mbHelp').textContent)));
    await p.keyboard.press('Escape');
  });

  await section('one id, one element; every moved control still found', async () => {
    const D = await p.evaluate(() => { const seen = {}, dup = []; for (const e of document.querySelectorAll('[id]')) { if (seen[e.id]) dup.push(e.id); seen[e.id] = 1; } return dup; });
    A('no id appears twice in the document', D.length === 0, D.slice(0, 8).join(','));
    for (const [label, id] of [['Seed', 'seedN'], ['Brush size', 'sBrush'], ['Hardness', 'sHard'], ['Intensity', 'sInten'], ['Image size', 'bakeRes']]) {
      await p.evaluate(() => { if (typeof _dskMenuClose === 'function') _dskMenuClose(); });
      /* start somewhere else, so the search has to navigate */
      await tryClick(p, '#domainRail [data-domain="civ"]');
      await p.fill('#findInput', label); await sleep(120);
      const listed = await p.evaluate(id => [...document.querySelectorAll('#findResults [data-find-i]')].some((r, i) => document.getElementById('findResults')._hits[i].el && document.getElementById('findResults')._hits[i].el.id === id), id);
      await p.keyboard.press('Enter'); await sleep(350);
      A('Find "' + label + '" lists #' + id + ' and Enter reveals it on screen', listed && (await vis(p, '#' + id)), 'listed=' + listed);
    }
    await p.evaluate(() => { if (typeof _dskMenuClose === 'function') _dskMenuClose(); });
    await tryClick(p, '#worldCatSeg [data-wcat="terrain"]');
  });

  await section('the tool-options bar holds the armed tool\'s settings only (Ruling DZ)', async () => {
    await tryClick(p, '#domainRail [data-domain="world"]'); await tryClick(p, '#worldCatSeg [data-wcat="terrain"]'); await sleep(120);
    const T = await p.evaluate(() => { const to = document.getElementById('toolOpts'), run = document.getElementById('runSec');
      const ids = ['seedN', 'genBtn', 'reseedBtn', 'centerBtn'];
      return { offBar: ids.filter(i => to.contains(document.getElementById(i))), inRun: ids.filter(i => run && run.contains(document.getElementById(i)) && !!document.getElementById(i).closest('.dock-left')),
        ctx: (document.getElementById('toolOptsCtx') || {}).textContent, view: !!document.querySelector('#layersPopover #modeSeg') }; });
    A('no Seed, ⟳, Center or Generate on the bar', T.offBar.length === 0, T.offBar.join(','));
    A('all four are in the WORLD dock\'s Run section', T.inRun.length === 4, T.inRun.join(','));
    A('with no tool armed the bar names the context (GENERATE · WORLD)', /Generate\s*·\s*World/i.test(T.ctx || ''), JSON.stringify(T.ctx));
    A('View moved into the Base map chip', T.view);
    for (const s of ['#seedN', '#genBtn', '#reseedBtn', '#centerBtn']) A('...' + s + ' is on screen, in the dock', (await vis(p, s)) && (await p.evaluate(s => !!document.querySelector(s).closest('.dock-left'), s)));
    A('the pins are in the bar and reachable: Save, Settlements, Assets, Planner', await p.evaluate(() => ['pinSave', 'settlementsHeaderBtn', 'assetsHeaderBtn', 'pinPlanner'].every(i => { const e = document.getElementById(i); return e && e.closest('#toolOptsPins') && e.getBoundingClientRect().width > 0; })));
    A('...Settlements opens its screen on a real click', (await tryClick(p, '#settlementsHeaderBtn')) && await p.evaluate(() => document.getElementById('civFactionsModal').classList.contains('open')));
    await tryClick(p, '#cfmCloseBtn'); await sleep(80);
    await tryClick(p, '#worldCatSeg [data-wcat="sculpt"]'); await sleep(200);
    const Sc = await p.evaluate(() => { const t = document.getElementById('toolOptsTool'); return { shown: !!t && !t.hidden, feats: document.querySelectorAll('#toolOpts #sculptFeatureSeg button').length,
      brush: ['sBrush', 'sHard', 'sInten'].every(i => t && t.contains(document.getElementById(i))), rail: getComputedStyle(document.getElementById('toolRail')).display, ctx: document.getElementById('toolOptsCtx').textContent,
      label: SCULPT_FEATURES[_sculptType].label }; });
    A('Sculpt armed: the bar holds the feature picker (all 13)', Sc.shown && Sc.feats === 13, JSON.stringify(Sc));
    A('...and the brush: size, hardness, intensity', Sc.brush);
    A('...and names it SCULPT · <feature>', Sc.ctx === 'Sculpt · ' + Sc.label, Sc.ctx);
    A('...and the old tool-rail band does not draw on the desktop', Sc.rail === 'none', Sc.rail);
    await tryClick(p, '#sculptFeatureSeg [data-sfeat="freehand"]'); await sleep(120);
    A('Freehand\'s brush MODE joins them in the bar', await p.evaluate(() => { const m = document.getElementById('sculptModeSeg'); return !!m.closest('#toolOpts') && m.getBoundingClientRect().width > 0 && m.querySelectorAll('button').length >= 3; }));
    A('...and the bar says so', await p.evaluate(() => document.getElementById('toolOptsCtx').textContent === 'Sculpt · Freehand'));
    A('...with the armed feature scrolled into view in the picker', await p.evaluate(() => { const fs = document.getElementById('sculptFeatureSeg'), on = fs.querySelector('button.on'); if (!on) return false; const a = fs.getBoundingClientRect(), r = on.getBoundingClientRect(); return r.left >= a.left - 1 && r.right <= a.right + 1; }));
    const br0 = await p.evaluate(() => _sculptGlobal.brushSize);
    await p.focus('#sBrush'); await p.keyboard.press('ArrowRight'); await p.keyboard.press('ArrowRight'); await sleep(60);
    A('the brush size slider in the bar still drives the brush', await p.evaluate(b => _sculptGlobal.brushSize === b + 2, br0));
    await tryClick(p, '#worldCatSeg [data-wcat="terrain"]'); await sleep(120);
    A('leaving Sculpt hides the tool\'s settings again', await p.evaluate(() => document.getElementById('toolOptsTool').hidden));
    await p.evaluate(() => setFinalized(true)); await sleep(100);
    const L = await p.evaluate(() => ({ dis: ['seedN', 'genBtn', 'reseedBtn', 'centerBtn'].filter(i => !document.getElementById(i).disabled) }));
    A('finalize still locks all four where they now live (data-genlock by id)', L.dis.length === 0, L.dis.join(','));
    await openMenu(p, '#fileMenuBtn');
    A('...and File ▸ Generate world reads disabled', await p.evaluate(() => document.querySelector('#fileMenu [data-cmd="generate"]').disabled));
    await p.keyboard.press('Escape');
    const g0 = await p.evaluate(() => window.__gens); p._dialogs = [];
    await p.keyboard.press('Control+r'); await settle(p);
    A('...and Ctrl+R says why (the finalize alert) instead of generating', (await p.evaluate(() => window.__gens)) === g0 && (p._dialogs || []).some(m => /finalized/i.test(m)), JSON.stringify(p._dialogs));
    await p.evaluate(() => { setFinalized(false); syncUI(); }); await sleep(80);
    A('un-finalize releases them', await p.evaluate(() => ['seedN', 'genBtn', 'reseedBtn', 'centerBtn'].every(i => !document.getElementById(i).disabled)));
  });

  await section('the timeline strip, in every domain', async () => {
    await tryClick(p, '#domainRail [data-domain="world"]');
    if (await p.evaluate(() => document.getElementById('tlStripMore') && document.getElementById('tlStripMore').hidden)) await tryClick(p, '#tlStripToggle');
    /* SET UP: three recorded years with different painted territory, entered through the strip's own
       Add year (the territory paint itself is the setup) */
    for (const [yr, fid] of [[100, 1], [250, 2], [400, 3]]) {
      await p.evaluate(fid => { const t = getCivTerritory(); for (let i = 0; i < t.length; i += 7) t[i] = fid; _civTerrGen++; }, fid);
      await p.fill('#civTlYear', String(yr), { timeout: 3000 }); await tryClick(p, '#civTlAddYearBtn'); await sleep(100);
    }
    A('Add year in the strip records years', await p.evaluate(() => civTimeline.length === 3), await p.evaluate(() => civTimeline.length));
    const hashT = () => p.evaluate(() => { const t = getCivTerritory(); let h = 2166136261; for (let i = 0; i < t.length; i += 3) { h ^= t[i]; h = Math.imul(h, 16777619) >>> 0; } return h; });
    const hashC = () => p.evaluate(() => { const c = document.getElementById('civCanvas'), x = c.getContext('2d').getImageData(0, 0, c.width, c.height).data; let h = 2166136261; for (let i = 0; i < x.length; i += 97) { h ^= x[i]; h = Math.imul(h, 16777619) >>> 0; } return h; });
    for (const dom of ['world', 'civ', 'carto', 'explore']) {
      await tryClick(p, '#domainRail [data-domain="' + dom + '"]'); await sleep(150);
      const shown = await vis(p, '#explTimelineSlider');
      const y0 = await p.evaluate(() => civYear), t0 = await hashT(), c0 = await hashC();
      await p.focus('#explTimelineSlider'); await p.keyboard.press(y0 === 100 ? 'End' : 'Home'); await sleep(250);
      const y1 = await p.evaluate(() => civYear), t1 = await hashT(), c1 = await hashC();
      const label = await p.evaluate(() => document.getElementById('explTlActiveYear').textContent);
      A('in ' + dom.toUpperCase() + ' the strip shows, and its slider moves the year', shown && y1 !== y0 && label === _fy(y1), y0 + '→' + y1 + ' label=' + label);
      A('...and moves the map with it (territory and the civ layer redraw)', t1 !== t0 && c1 !== c0, [t0, t1, c0, c1].join(','));
    }
    A('the strip is a band of its own above the status bar', await p.evaluate(() => { const s = document.getElementById('tlStrip').getBoundingClientRect(), b = document.getElementById('statusBar').getBoundingClientRect(); return s.height > 0 && Math.abs(s.bottom - b.top) <= 1; }));
    const before = await p.evaluate(() => document.querySelector('.canvas-wrap').getBoundingClientRect().height);
    await tryClick(p, '#tlStripToggle'); await sleep(250);
    A('collapsing it gives the map its row back (the viewport refits)', await p.evaluate(h => document.querySelector('.canvas-wrap').getBoundingClientRect().height > h, before));
    A('the simulator stayed in the Explore dock', await p.evaluate(() => !!document.getElementById('civSimulateBtn').closest('#explorePanel')));
  });

  await section('the Context panel names what Inspect shows', async () => {
    await tryClick(p, '#domainRail [data-domain="civ"]'); await sleep(120);
    /* SET UP: one named town on land, drawn */
    const at = await p.evaluate(() => {
      let x = -1, y = -1; for (let j = 20; j < GH - 20 && x < 0; j += 3) for (let i = 20; i < GW - 20; i += 3) { const h = field[j * GW + i]; if (h > state.seaLevel + 0.04) { x = i; y = j; break; } }
      state.places.push({ x, y, name: 'Probe Greyhollow', kind: 'town', faction: 1, pop: 4200, traits: [] }); drawCivLayerAuto(); renderNow();
      const s = _civPlaceScreenPos(x, y); return { x: s[0], y: s[1] }; });
    await sleep(200);
    const empty = await p.evaluate(() => document.getElementById('ctxName').textContent);
    A('with nothing selected the header says so', empty === 'Nothing selected', empty);
    await p.mouse.click(at.x, at.y); await sleep(250);
    const C = await p.evaluate(() => ({ name: document.getElementById('ctxName').textContent, kind: document.getElementById('ctxKind').textContent, hint: document.getElementById('ctxHint').textContent, body: document.getElementById('inspectorBody').textContent }));
    A('clicking a settlement on the map names it in the header', C.name === 'Probe Greyhollow', JSON.stringify(C));
    A('...with its kind and facts (from the place itself)', /Settlement/.test(C.kind) && /4,200/.test(C.hint), C.kind + ' / ' + C.hint);
    A('...and Inspect fills (the place editor opens, as v0.90 has it, and Inspect says where)', /Editing in the popup/.test(C.body), C.body.slice(0, 80));
    A('the tab strip holds Inspect only (no note editor, no assistant in this file)', await p.evaluate(() => [...document.querySelectorAll('.ctx-tab')].map(t => t.textContent.trim()).join(',') === 'Inspect'));
    await tryClick(p, '#placeEditPopup .si-close'); await sleep(150);
    const E = await p.evaluate(() => ({ name: document.getElementById('ctxName').textContent, body: document.getElementById('inspectorBody').textContent }));
    A('closing the popup with its × empties the header AND Inspect (no stale "Editing in the popup")', E.name === 'Nothing selected' && /Nothing selected/.test(E.body), JSON.stringify(E).slice(0, 160));
  });
  await p.close();

  /* ════════════════════════════ LAYOUT, four desktop sizes x two seeds ════════════════════════════ */
  console.log('\n--- nothing overflows (1366x768, 1600x900, 1920x1080, 2560x1440; seeds 12345 and 777) ---');
  for (const [W, H] of [[1366, 768], [1600, 900], [1920, 1080], [2560, 1440]]) for (const seed of [12345, 777]) await section('layout ' + W + 'x' + H + ' seed ' + seed, async () => {
    const q = await boot(b, FILE, { width: W, height: H });
    await gen(q, seed);
    const tag = W + 'x' + H + ' seed ' + seed;
    const O = await q.evaluate(() => {
      const bad = [], vw = innerWidth, vh = innerHeight, de = document.documentElement;
      if (de.scrollWidth > vw + 1 || de.scrollHeight > vh + 1) bad.push('page scrolls ' + de.scrollWidth + 'x' + de.scrollHeight);
      for (const sel of ['header', '#toolOpts', '#tlStrip', '#statusBar', '#dockLeftHead', '#ctxHead', '#mapTL', '#mapBR']) {
        const B = document.querySelector(sel); if (!B || !B.getClientRects().length) { bad.push(sel + ' not drawn'); continue; }
        const br = B.getBoundingClientRect();
        if (br.left < -1 || br.top < -1 || br.right > vw + 1 || br.bottom > vh + 1) bad.push(sel + ' leaves the window');
        for (const e of B.querySelectorAll('*')) { if (e.closest('.mb-panel,.mb-subpanel,#findResults,.dropdown')) continue; const r = e.getBoundingClientRect(); if (!r.width || !r.height) continue;
          if (r.right > br.right + 1 || r.left < br.left - 1) bad.push((e.id || e.className || e.tagName) + ' overflows ' + sel); }
      }
      const cw = document.querySelector('.canvas-wrap').getBoundingClientRect();
      const boxes = ['mapTL', 'mapBR', 'scaleBar', 'legend'].map(id => [id, document.getElementById(id)]).filter(([, e]) => e && e.getClientRects().length).map(([id, e]) => [id, e.getBoundingClientRect()]);
      for (const [id, r] of boxes) if (r.left < cw.left - 1 || r.right > cw.right + 1 || r.top < cw.top - 1 || r.bottom > cw.bottom + 1) bad.push(id + ' outside the map');
      for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length; j++) { const a = boxes[i][1], c = boxes[j][1];
        if (a.left < c.right && c.left < a.right && a.top < c.bottom && c.top < a.bottom) bad.push(boxes[i][0] + ' overlaps ' + boxes[j][0]); }
      return bad;
    });
    A(tag + ': every band and map control inside the window, nothing overlapping', O.length === 0, O.slice(0, 6).join(' | '));
    let menusIn = true, why = '';
    for (const s of ['#fileMenuBtn', '#mbEditBtn', '#mbAssetsBtn', '#mbRecordsBtn', '#mbStoryBtn', '#mbPrefsBtn', '#mbWindowBtn', '#mbHelpBtn']) {
      await openMenu(q, s); await sleep(40);
      const r = await q.evaluate(() => { const pn = document.querySelector('.mb-panel.open'); if (!pn) return null; const x = pn.getBoundingClientRect(); return [x.left, x.top, x.right, x.bottom, innerWidth, innerHeight]; });
      if (!r || r[0] < 0 || r[2] > r[4] || r[3] > r[5]) { menusIn = false; why += s + ' '; }
    }
    await q.keyboard.press('Escape');
    await openMenu(q, '#fileMenuBtn'); await q.hover('#mbExportRow', { timeout: 3000 }); await sleep(120);
    const ex = await q.evaluate(() => { const x = document.getElementById('mbExportPanel').getBoundingClientRect(); return x.height > 0 && x.top >= 0 && x.bottom <= innerHeight && x.right <= innerWidth; });
    A(tag + ': every menu, and File ▸ Export, opens inside the window', menusIn && ex, why + (ex ? '' : 'export'));
    await q.close();
  });

  /* ════════════════════════════ PHONE ════════════════════════════ */
  console.log('\n--- a phone keeps v2.71\'s layout (stage 2 changes it) ---');
  const phone = async (file, W, H) => {
    const q = await boot(b, file, { width: W, height: H }, true);
    await gen(q);
    const R = {};
    Object.assign(R, await q.evaluate(() => { const g = s => document.querySelector(s), rr = e => { const r = e.getBoundingClientRect(); return [r.left, r.top, r.width, r.height].map(Math.round); };
      const hidden = s => { const e = g(s); return !e || !e.getClientRects().length; };
      return { map: rr(g('.canvas-wrap')), header: [...g('header').children].filter(e => e.getClientRects().length).map(e => [e, e.getBoundingClientRect()]).sort((x, y) => (Math.round((x[1].top + x[1].height / 2) / 24) - Math.round((y[1].top + y[1].height / 2) / 24)) || (x[1].left - y[1].left)).map(([e]) => e.id || e.tagName.toLowerCase()).join(','),
        toolOpts: [...g('#toolOpts').querySelectorAll('input,button,.seg')].filter(e => e.getClientRects().length).map(e => e.id).filter(Boolean).join(','),
        deskHidden: ['#menuBar', '#toolOptsCtx', '#toolOptsPins', '#runSec', '#tlStrip', '#ctxHead', '#dockLeftHead', '#mapTL', '#mapBR', '#docReadout'].filter(s => !hidden(s)),
        featInRail: !!g('#toolRail #sculptFeatureSeg'), sliderInExplore: !!g('#explTimelineSection #explTimelineSlider') }; }));
    await q.tap('#panelToggle'); await sleep(450);
    Object.assign(R, await q.evaluate(() => { const g = s => document.querySelector(s), rr = e => { const r = e.getBoundingClientRect(); return [r.left, r.top, r.width, r.height].map(Math.round); };
      const rail = g('.domain-rail'), wrap = g('#dockWrap');
      return { open: document.body.classList.contains('panel-open'), drawer: rr(wrap), rail: rr(rail), railInDrawer: wrap.contains(rail), sticky: getComputedStyle(rail).position }; }));
    await q.tap('#domainRail [data-domain="carto"]'); await sleep(250);
    R.railSwitch = await q.evaluate(() => _domain === 'carto');
    await q.tap('#scrim', { position: { x: 8, y: Math.round(H / 2) } }); await sleep(400);
    R.closed = await q.evaluate(() => !document.body.classList.contains('panel-open'));
    await q.close();
    return R;
  };
  for (const [W, H] of [[390, 760], [412, 915]]) await section('phone ' + W + 'x' + H, async () => {
    const N = await phone(FILE, W, H), tag = W + 'x' + H;
    A(tag + ': none of the desktop shell draws on a phone', N.deskHidden.length === 0, N.deskHidden.join(','));
    A(tag + ': the header is v2.71\'s (☰ · Settlements · Assets · title · version · Find · Undo · Redo · File · cog)', /panelToggle,settlementsHeaderBtn,assetsHeaderBtn,h1,verTag,findWrap,undoBtn,redoBtn,fileWrap,cogBtn/.test(N.header), N.header);
    A(tag + ': the document bar is v2.71\'s (seed · ⟳ · View · Center · Generate)', N.toolOpts === 'seedN,reseedBtn,modeSeg,centerBtn,genBtn', N.toolOpts);
    A(tag + ': the sculpt picker is back in the tool rail, the timeline back in Explore', N.featInRail && N.sliderInExplore);
    A(tag + ': ☰ opens the drawer, with the rail as its sticky head', N.open && N.railInDrawer && N.sticky === 'sticky' && N.rail[1] === N.drawer[1], JSON.stringify([N.drawer, N.rail, N.sticky]));
    A(tag + ': the rail still switches domains, and the scrim closes the drawer', N.railSwitch && N.closed);
    if (BEFORE) {
      const O = await phone(BEFORE, W, H);
      A(tag + ': the map is where v2.71 puts it (to the pixel)', JSON.stringify(N.map) === JSON.stringify(O.map), JSON.stringify([N.map, O.map]));
      A(tag + ': the drawer and its rail are where v2.71 puts them', JSON.stringify(N.drawer) === JSON.stringify(O.drawer) && JSON.stringify(N.rail) === JSON.stringify(O.rail), JSON.stringify([N.drawer, O.drawer, N.rail, O.rail]));
      A(tag + ': the header holds what v2.71\'s holds, in its order', N.header === O.header, N.header + ' vs ' + O.header);
    }
  });
  } catch (e) { A('the probe ran to the end', false, e && e.message ? e.message.split('\n')[0] : String(e)); }

  console.log('\n--- page errors ---');
  A('no page errors on any page this probe opened', errors.length === 0, errors.slice(0, 3).join(' | '));
  await b.close();
  console.log('\n' + pass + ' passed, ' + fail + ' failed');
  process.exit(fail ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });

/* (the header list above is in VISUAL order: rows by their vertical centre, then left to right —
   the DOM order changed in v2.72 on purpose, and only what is drawn has to match v2.71) */
/* the page's own year format, mirrored for one comparison (block 2's _civFormatYear) */
function _fy(y) { return y < 0 ? (-y) + ' BC' : y + ' AD'; }
