/* v2.73 — the phone and tablet shells, stage 2 of 3: one layout class decided by one function (phone under
 * 600px wide or under 500px tall in landscape; tablet for any other coarse primary pointer; desktop for a fine
 * one), the phone's bottom bar, sheet, MENU, STORY, tool strip, quick drawer and long-press card, the DS-13 map
 * row with Handedness, the touch gestures, the tablet's touch-sized desktop shell with its narrow bar, and the
 * docking rule on the desktop and the tablet.
 *
 * Owner: "Can we upgrade the layout in the html with the new design?", "Everything, staged", and on v2.72's
 * phone capture: "The screenshots on phone don't seem to have the pull-out tab from the side." Design:
 * Cartalith_GDT MOBILE_SHELL_PROPOSAL.md §2.1-§2.5, §3, §4; Rulings DT, DU, DX, DY (MQ-1, Q10-Q12, PQ6), EB.
 *
 * Every claim is a rendered geometry or a REAL input: page.touchscreen taps, CDP Input.dispatchTouchEvent for
 * drags, holds and two-finger touches (Playwright has no touch drag), page.mouse with a mouse, page.keyboard.
 * A page.evaluate is used only to SET UP state (a world, a settlement), to READ state the input changed (the
 * camera, the domain, the territory raster), or to clear a popup between steps — never to navigate the shell.
 * Sizes: phone 360x780, 412x915 and 844x390 (touch, Android user agent so the file's own touch flag is set);
 * tablet 768x1024 and 1194x834 (touch); the narrow tablet bar at 600x960, 768x1024 and 1024x768; desktop
 * 700x900 and 1366x768 (mouse).
 *
 * Usage: node tests/perf/probe_mobileshell.js "Cartalith v2.73 DCC test.html"
 *   One file. Any other argument is refused (exit 2), never ignored. Run it on the previous build too: every
 *   section that build cannot run fails once, by name, and the next section still runs.
 * Env: PLAYWRIGHT_DIR, CHROME_BIN — as every probe in this folder.
 * Output: one "ok   - …" or "FAIL - …" line per assertion, then a count. Exit status 1 if anything failed.
 */
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_DIR || '/opt/node22/lib/node_modules/playwright');

const FILE = process.argv[2];
if (!FILE || process.argv.length > 3 || !fs.existsSync(FILE)) {
  console.error('usage: node probe_mobileshell.js <v2.73.html>  — got: ' + JSON.stringify(process.argv.slice(2)));
  process.exit(2);
}
let pass = 0, fail = 0;
const A = (n, c, extra) => { if (c) { pass++; console.log('ok   - ' + n); }
  else { fail++; console.log('FAIL - ' + n + (extra !== undefined ? '  [' + extra + ']' : '')); } };
const errors = [];
const sleep = ms => new Promise(r => setTimeout(r, ms));
const section = async (name, fn) => { console.log('\n--- ' + name + ' ---');
  try { await fn(); } catch (e) { A(name + ': the section ran to the end', false, String((e && e.message) || e).split('\n')[0]); } };
/* an Android phone/tablet user agent: the file's own touch flag (isMobile, v1.19) reads the UA or
   maxTouchPoints>1, and Playwright's touch emulation reports maxTouchPoints 1 */
const AND = 'Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Mobile Safari/537.36';
const boot = async (b, vp, touch) => {
  const ctx = await b.newContext({ viewport: vp, hasTouch: !!touch, isMobile: !!touch && vp.width < 1000, userAgent: touch ? AND : undefined });
  const p = await ctx.newPage();
  p.on('pageerror', e => errors.push(vp.width + 'x' + vp.height + ': ' + e.message));
  p.on('console', m => { if (m.type() === 'error') errors.push(vp.width + 'x' + vp.height + ' console: ' + m.text()); });
  p.on('dialog', d => d.dismiss().catch(() => {}));
  await p.goto('file://' + path.resolve(FILE));
  await p.waitForFunction(() => typeof window.generate === 'function', { timeout: 60000 });
  p._cdp = await ctx.newCDPSession(p);
  p._ctx = ctx;
  return p;
};
/* SET UP only: a small world, the gate dismissed the way a generated world dismisses it */
const gen = (p, seed) => p.evaluate(async (seed) => {
  state.tect.seed = seed; state.resW = 256; GW = state.resW; GH = gridH(GW); allocate();
  if (typeof _setupHide === 'function') _setupHide();
  await generate(); syncUI(); if (typeof _viewFill === 'function') _viewFill(); renderNow();
}, seed || 12345);
const rectOf = (p, sel) => p.evaluate(s => { const e = document.querySelector(s); if (!e || !e.getClientRects().length) return null;
  const r = e.getBoundingClientRect(); const cs = getComputedStyle(e);
  if (cs.visibility === 'hidden' || r.width === 0 || r.height === 0) return null;
  return { l: r.left, t: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height }; }, sel);
const vis = async (p, sel) => !!(await rectOf(p, sel));
/* a real tap at an element's centre (touch) */
/* scroll the control into view first, as a person scrolls the sheet to reach a row below the fold: an
   unclipped rect below a scroller's edge is not a place a finger can press */
const reach = (p, sel) => p.evaluate(s => { const e = document.querySelector(s); if (e && e.scrollIntoView) e.scrollIntoView({ block: 'nearest' }); }, sel);
const tap = async (p, sel) => { await reach(p, sel); const r = await rectOf(p, sel); if (!r) return false; await p.touchscreen.tap(r.l + r.w / 2, r.t + r.h / 2); await sleep(220); return true; };
/* a tap near an element's EDGE, 4px in from its left and top: a centre press cannot see a target narrower
   than it looks (MISTAKES.md, "Write a probe that PRESSES a control") */
const tapEdge = async (p, sel) => { await reach(p, sel); const r = await rectOf(p, sel); if (!r) return false; await p.touchscreen.tap(r.l + 4, r.t + 4); await sleep(220); return true; };
const T = (p, type, pts) => p._cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map((q, i) => ({ x: q[0], y: q[1], id: q[2] !== undefined ? q[2] : i })) });
/* a drag that settles before it lifts: the finger rests 150ms at the end, so the lift carries no velocity.
   A drag lifted at speed is a FLING to the browser, and the next tap then only stops the fling — measured
   here, the first tap after a fast drag produced no click at all (Chrome's gesture detector) */
const drag = async (p, x0, y0, x1, y1, steps) => { steps = steps || 8; await T(p, 'touchStart', [[x0, y0]]);
  for (let i = 1; i <= steps; i++) { await T(p, 'touchMove', [[x0 + (x1 - x0) * i / steps, y0 + (y1 - y0) * i / steps]]); await sleep(16); }
  await T(p, 'touchMove', [[x1, y1]]); await sleep(150); await T(p, 'touchEnd', []); await sleep(260); };
const hold = async (p, x, y, ms) => { await T(p, 'touchStart', [[x, y]]); await sleep(ms); await T(p, 'touchEnd', []); await sleep(260); };
const cam = p => p.evaluate(() => ({ s: viewT.scale, x: viewT.panX, y: viewT.panY }));
const shell = p => p.evaluate(() => document.documentElement.getAttribute('data-shell'));
const clearSel = p => p.evaluate(() => { if (typeof _civCtxHide === 'function') _civCtxHide(); _civSelectedPlace = null; if (typeof _civClosePlacePopup === 'function') _civClosePlacePopup(); });
/* the map's centre band, clear of the sheet, the bars and the row */
const mapPt = async (p, fx, fy) => { const r = await rectOf(p, '.canvas-wrap'); return [r.l + r.w * (fx || 0.5), r.t + r.h * (fy || 0.35)]; };
/* Every tappable control under the given roots that is drawn (not display:none, not visibility:hidden,
   non-zero), with its laid size. The floor is on the TARGET, both axes (MISTAKES.md, "Ask a lane to walk a
   touch floor"); a checkbox's target is its label. */
const TAPPABLE = 'button, select, textarea, input:not([type=hidden]):not([type=checkbox]):not([type=radio]):not([type=file]), summary, label.chk, [role="button"], [role="tab"], .pill';
const smallTargets = (p, roots) => p.evaluate(([roots, sel]) => {
  const out = [];
  for (const rs of roots) for (const root of document.querySelectorAll(rs)) for (const e of root.querySelectorAll(sel)) {
    if (!e.getClientRects().length) continue;
    const cs = getComputedStyle(e); if (cs.visibility === 'hidden') continue;
    const r = e.getBoundingClientRect(); if (r.width === 0 || r.height === 0) continue;
    if (r.width < 43.5 || r.height < 43.5) out.push((e.id ? '#' + e.id : e.tagName.toLowerCase() + (e.className ? '.' + String(e.className).split(' ')[0] : '')) + ' "' + (e.textContent || e.value || '').trim().slice(0, 18) + '" ' + Math.round(r.width) + 'x' + Math.round(r.height));
  }
  return [...new Set(out)];
}, [roots, TAPPABLE]);
/* how many of these targets were measured — a floor walk that found nothing is not a pass */
const countTargets = (p, roots) => p.evaluate(([roots, sel]) => { let n = 0;
  for (const rs of roots) for (const root of document.querySelectorAll(rs)) for (const e of root.querySelectorAll(sel)) if (e.getClientRects().length && getComputedStyle(e).visibility !== 'hidden') n++;
  return n; }, [roots, TAPPABLE]);
/* nothing wider than the window: the document does not scroll sideways, and no drawn element of the shell
   runs past either edge (a horizontal scroller's own children are allowed to) */
const overflow = p => p.evaluate(() => {
  const W = document.documentElement.clientWidth, bad = [];
  if (document.documentElement.scrollWidth > W + 1) bad.push('document ' + document.documentElement.scrollWidth + '>' + W);
  for (const e of document.querySelectorAll('header *, .ph-bar *, .ph-sheet, .ph-sheet-head *, .ph-strip, .ph-drawer, .ph-fan, .toolopts *, .status-bar, #zoomOverlay, .ph-grip')) {
    if (!e.getClientRects().length) continue; const r = e.getBoundingClientRect(); if (!r.width) continue;
    let sc = false; for (let a = e.parentElement; a; a = a.parentElement) { const ox = getComputedStyle(a).overflowX; if (ox === 'auto' || ox === 'scroll' || ox === 'hidden') { sc = true; break; } }
    if (sc) continue;
    if (r.left < -1 || r.right > W + 1) bad.push((e.id || e.className || e.tagName) + ' ' + Math.round(r.left) + '..' + Math.round(r.right));
  }
  return bad.slice(0, 6);
});
const dupIds = p => p.evaluate(() => { const seen = {}, d = []; for (const e of document.querySelectorAll('[id]')) { if (seen[e.id]) d.push(e.id); seen[e.id] = 1; } return d; });

(async () => {
  const b = await chromium.launch({ executablePath: process.env.CHROME_BIN, args: ['--no-sandbox'] });
  try {
  /* ════════════════════════ THE CLASS, AT EVERY SIZE ════════════════════════ */
  const SIZES = [['phone', 360, 780, true], ['phone', 412, 915, true], ['phone', 844, 390, true], ['tablet', 768, 1024, true],
    ['tablet', 1194, 834, true], ['desktop', 700, 900, false], ['desktop', 1366, 768, false]];
  for (const [want, w, h, touch] of SIZES) {
    await section('the class at ' + w + 'x' + h + (touch ? ' (touch)' : ' (mouse)'), async () => {
      const p = await boot(b, { width: w, height: h }, touch); await gen(p); await sleep(300);
      A(w + 'x' + h + ': the shell is the ' + want + "'s (html[data-shell])", (await shell(p)) === want, await shell(p));
      const ov = await overflow(p);
      A(w + 'x' + h + ': nothing runs past the window\'s edges, and the page does not scroll sideways', ov.length === 0, ov.join('; '));
      const d = await dupIds(p);
      A(w + 'x' + h + ': no id is used twice (every moved control is one element)', d.length === 0, d.slice(0, 5).join(','));
      if (want === 'phone') {
        A(w + 'x' + h + ': the bottom bar (or the landscape rail) is drawn, and the menu bar is not', (await vis(p, '#phBar')) && !(await vis(p, '#menuBar')));
        A(w + 'x' + h + ': the quick drawer\'s grip — the pull-out tab — is drawn on the map\'s edge', await vis(p, '#phGrip'));
      } else {
        A(w + 'x' + h + ': the menu bar is drawn, and the phone\'s bar, sheet and grip are not', (await vis(p, '#menuBar')) && !(await vis(p, '#phBar')) && !(await vis(p, '#phGrip')));
      }
      await p._ctx.close();
    });
  }

  /* ════════════════════════ ONE DECIDER, RE-EVALUATED, WITH HYSTERESIS ════════════════════════ */
  await section('re-deciding on resize, with hysteresis', async () => {
    const p = await boot(b, { width: 700, height: 900 }, false); await gen(p);
    const at = async (w, h) => { await p.setViewportSize({ width: w, height: h || 900 }); await sleep(250); return shell(p); };
    A('700px with a mouse: the desktop shell', (await at(700)) === 'desktop');
    A('at 590px still the desktop: the line moves 16px away from the class in force (600 - 16)', (await at(590)) === 'desktop');
    A('at 570px a phone, without a reload', (await at(570)) === 'phone');
    A('back at 610px still a phone (600 + 16 to leave it) — a window on the line does not flap', (await at(610)) === 'phone');
    A('at 620px the desktop again', (await at(620)) === 'desktop');
    A('down to 570px: a phone once more', (await at(570)) === 'phone');
    const ph = await p.evaluate(() => ({ left: !!document.getElementById('dockLeft').closest('#phPageDomain'), file: !!document.getElementById('fileMenu').closest('#phPageMenu') }));
    A('as a phone the left dock is the domain page and File is MENU\'s (moved, not copied)', ph.left && ph.file, JSON.stringify(ph));
    await at(1366, 768);
    const dk = await p.evaluate(() => ({ left: !!document.getElementById('dockLeft').closest('#dockWrap'), file: !!document.getElementById('fileMenu').closest('#menuBar') }));
    A('back on the desktop both are home again', dk.left && dk.file, JSON.stringify(dk));
    A('a landscape window under 500px tall is a phone (844x390)', (await at(844, 390)) === 'phone');
    const d = await dupIds(p);
    A('after five class changes no id is used twice', d.length === 0, d.join(','));
    await p._ctx.close();
  });

  /* ════════════════════════ THE PHONE, 412 x 915 ════════════════════════ */
  const p = await boot(b, { width: 412, height: 915 }, true);
  await gen(p);
  await sleep(300);

  await section('phone: the bottom bar and the app bar', async () => {
    const tabs = await p.evaluate(() => [...document.querySelectorAll('#phBar .ph-tab')].filter(t => t.getClientRects().length).map(t => { const r = t.getBoundingClientRect(); return { tx: t.textContent.replace(/\s+/g, ' ').trim(), w: r.width, h: r.height, b: r.bottom }; }));
    A('four destinations, MAP · the domain tab · STORY · MENU', tabs.length === 4 && /map/i.test(tabs[0].tx) && /generate/i.test(tabs[1].tx) && /story/i.test(tabs[2].tx) && /menu/i.test(tabs[3].tx), JSON.stringify(tabs.map(t => t.tx)));
    A('each is a 64px target at least 44px wide', tabs.length === 4 && tabs.every(t => t.h >= 63.5 && t.w >= 44), JSON.stringify(tabs.map(t => [Math.round(t.w), Math.round(t.h)])));
    A('the bar sits on the bottom edge', tabs.length === 4 && tabs.every(t => Math.abs(t.b - 915) <= 1));
    const pill = await p.evaluate(() => { const e = document.getElementById('phWorldPill'); return e && e.getClientRects().length ? e.textContent.replace(/\s+/g, ' ').trim() : null; });
    A('the app bar\'s world pill reads the open world (seed · grid · extent)', /seed 12345 · 256×\d+ · (region|whole world)/.test(pill || ''), pill);
    A('...beside Undo, Redo and Search', (await vis(p, 'header #undoBtn')) && (await vis(p, 'header #redoBtn')) && (await vis(p, '#phSearchBtn')));
  });

  await section('phone: the domain tab opens the domain sheet; the detents', async () => {
    await tapEdge(p, '#phTabDomain');
    const s = await p.evaluate(() => ({ det: _ph.detent, page: _ph.page, dock: !!document.getElementById('dockLeft').closest('#phPageDomain'), title: document.getElementById('phSheetTitleTx').textContent }));
    A('a tap (near its edge) opens the sheet at half on the domain page', s.det === 'half' && s.page === 'domain', JSON.stringify(s));
    A('...which is the left dock itself, with its Run section', s.dock && (await vis(p, '#phPageDomain #genBtn')));
    A('...titled World · Generate', s.title === 'World · Generate', s.title);
    const H = await p.evaluate(() => { const bar = document.getElementById('phBar').getBoundingClientRect().top, app = document.querySelector('header').getBoundingClientRect().bottom; return { avail: bar - app, h: document.getElementById('phSheet').getBoundingClientRect().height }; });
    A('half is 50% of the space between the app bar and the bottom bar', Math.abs(H.h - H.avail / 2) <= 2, JSON.stringify(H));
    await tap(p, '#phSheetHandle'); await sleep(250);
    const full = await p.evaluate(() => { const sh = document.getElementById('phSheet').getBoundingClientRect(), app = document.querySelector('header').getBoundingClientRect().bottom; return { det: _ph.detent, gap: sh.top - app }; });
    A('the handle\'s tap takes it to full, just under the app bar', full.det === 'full' && full.gap >= 0 && full.gap <= 4, JSON.stringify(full));
    /* drag the head down to near the peek height: it snaps to peek (76px) */
    const hd = await rectOf(p, '#phSheetHead');
    await drag(p, hd.l + 60, hd.t + 10, hd.l + 60, 915 - 64 - 90, 10); await sleep(250);
    const pk = await p.evaluate(() => ({ det: _ph.detent, h: document.getElementById('phSheet').getBoundingClientRect().height }));
    A('dragging the head down snaps to peek, 76px (Ruling DU Q9)', pk.det === 'peek' && Math.abs(pk.h - 76) <= 1, JSON.stringify(pk));
    await tapEdge(p, '#phTabDomain'); await sleep(200);
    A('the domain tab, tapped again, opens it at half', (await p.evaluate(() => _ph.detent)) === 'half');
    await tapEdge(p, '#phTabDomain'); await sleep(200);
    A('...and the open tab\'s re-tap closes the sheet', (await p.evaluate(() => _ph.detent)) === 'closed' && !(await vis(p, '#phSheet')));
  });

  await section('phone: switching domain — the hold-and-slide fan, the swipe, the title\'s ▾', async () => {
    const t = await rectOf(p, '#phTabDomain');
    const cx = t.l + t.w / 2, cy = t.t + t.h / 2;
    await T(p, 'touchStart', [[cx, cy]]); await sleep(470);
    const fanOpen = await vis(p, '#phFan');
    const row = await rectOf(p, '#phFan [data-cmd="domCiv"]');
    A('a 380ms hold on the domain tab opens the fan of domains', fanOpen && !!row);
    if (row) { for (let i = 1; i <= 6; i++) { await T(p, 'touchMove', [[cx + (row.l + 40 - cx) * i / 6, cy + (row.t + row.h / 2 - cy) * i / 6]]); await sleep(20); } }
    await T(p, 'touchEnd', []); await sleep(300);
    const d1 = await p.evaluate(() => ({ dom: _domain, tab: document.getElementById('phTabDomainTx').textContent, fan: !document.getElementById('phFan').hidden }));
    A('sliding onto CIVIL and lifting switches the domain, and the tab reads CIVIL', d1.dom === 'civ' && /civil/i.test(d1.tab) && !d1.fan, JSON.stringify(d1));
    await drag(p, cx, cy, cx - 70, cy, 6);
    A('a sideways swipe on the tab cycles to the next domain (CARTO)', (await p.evaluate(() => _domain)) === 'carto');
    await tapEdge(p, '#phTabDomain');
    await tap(p, '#phSheetTitle');
    A('the sheet title\'s ▾ opens the same fan (the hold is never the only way)', await vis(p, '#phFan'));
    await tap(p, '#phFan [data-cmd="domWorld"]');
    A('...and a tap on World · Generate switches back', (await p.evaluate(() => _domain)) === 'world' && !(await vis(p, '#phFan')));
    await p.evaluate(() => _phSheetSet(null, 'closed'));
  });

  await section('phone: MAP and STORY', async () => {
    await tapEdge(p, '#phTabMap');
    A('MAP holds the 2D/3D switch, View and the layers list, and the legend', (await vis(p, '#phPageMap #viewDimSeg')) && (await vis(p, '#phPageMap #modeSeg')) && (await vis(p, '#phPageMap #layersList')) && (await vis(p, '#phPageMap #legend')));
    await tap(p, '#phPageMap #modeSeg [data-mode="hypso"]');
    A('...and its View row is the real control (Relief takes)', (await p.evaluate(() => state.mode)) === 'hypso');
    await tap(p, '#phPageMap #modeSeg [data-mode="biome"]');
    await tapEdge(p, '#phTabStory');
    A('STORY › Time is Explore\'s Timeline section (Add year, the simulator)', (await vis(p, '#phPageStory #civTlAddYearBtn')) && (await p.evaluate(() => !!document.getElementById('civSimulateBtn').closest('#phPageStory'))));
    await tap(p, '#phStorySeg [data-story="journeys"]');
    A('STORY › Journeys is the journey list and the planner', (await vis(p, '#phPageStory #civJourneyList')) && (await p.evaluate(() => !!document.getElementById('civPlannerSec').closest('#phPageStory'))));
    A('STORY draws only the segments this file has (Time, Journeys)', (await p.evaluate(() => document.querySelectorAll('#phStorySeg [data-story]').length)) === 2);
    await tap(p, '#phStorySeg [data-story="time"]');
    await tapEdge(p, '#phTabStory');
  });

  await section('phone: MENU — the eight menus row for row, drilling in, pressing the real commands', async () => {
    await tapEdge(p, '#phTabMenu');
    const rows = await p.evaluate(() => [...document.querySelectorAll('#phMenuRoot > .mb-sub > [data-sub]')].filter(e => e.getClientRects().length).map(e => e.textContent.replace(/[›▸]/g, '').trim()));
    A('MENU lists File · Edit · Assets · Records · Story · Preferences · Window · Help', JSON.stringify(rows) === JSON.stringify(['File', 'Edit', 'Assets', 'Records', 'Story', 'Preferences', 'Window', 'Help']), JSON.stringify(rows));
    const hs = await p.evaluate(() => [...document.querySelectorAll('#phMenuRoot .mb-row')].filter(e => e.getClientRects().length).map(e => e.getBoundingClientRect().height));
    A('every row is at least 48px tall', hs.length > 8 && hs.every(h => h >= 47.5), JSON.stringify(hs.slice(0, 6)));
    A('Quick comes first and Status last (the status bar itself)', (await p.evaluate(() => { const caps = [...document.querySelectorAll('#phMenuRoot > .mb-cap')].map(c => c.textContent.trim()); return caps[0] === 'Quick' && caps[caps.length - 1] === 'Status'; })) && (await p.evaluate(() => !!document.getElementById('statusBar').closest('#phPageMenu'))));
    /* Preferences › Theme › Light, by taps */
    await tap(p, '#phMenuRoot > .mb-sub[data-menu="prefs"] > [data-sub]');
    const drilled = await p.evaluate(() => ({ prefs: !!(document.getElementById('mbPrefs') && document.getElementById('mbPrefs').getClientRects().length), file: !!document.querySelector('#phMenuRoot > .mb-sub[data-menu="file"] > [data-sub]').getClientRects().length }));
    A('a tap drills into Preferences, and the other menus step aside (drill-in, not a cascade)', drilled.prefs && !drilled.file, JSON.stringify(drilled));
    const themeRow = await p.evaluate(() => { const s = [...document.querySelectorAll('#mbPrefs .mb-sub > [data-sub]')].find(e => /Theme/.test(e.textContent)); if (s) s.id = '__probeTheme'; return !!s; });
    if (themeRow) await tap(p, '#__probeTheme');
    const was = await p.evaluate(() => document.documentElement.getAttribute('data-theme'));
    await tap(p, '#mbPrefs [data-cmd="themeLight"]');
    A('Preferences › Theme › Light presses the theme switch (the page goes light) and the sheet closes', (await p.evaluate(() => document.documentElement.getAttribute('data-theme'))) === 'light' && (await p.evaluate(() => _ph.detent)) === 'closed', was);
    await p.evaluate(() => { const b = document.getElementById('themeToggleBtn'); if (document.documentElement.getAttribute('data-theme') === 'light' && b) b.click(); });
    /* Help › Credits opens the credits window as a full-height sheet */
    await tapEdge(p, '#phTabMenu');
    await tap(p, '#phMenuRoot > .mb-sub[data-menu="help"] > [data-sub]');
    await tap(p, '#mbHelp [data-cmd="credits"]');
    const cr = await p.evaluate(() => { const m = document.getElementById('creditsModal'), c = m && m.querySelector('.cr-card'); return m && m.classList.contains('open') && c ? Math.round(c.getBoundingClientRect().height) : 0; });
    A('Help › Credits opens Credits as a full-height sheet (a window on a phone, §2.1)', cr >= 914, cr);
    await p.evaluate(() => document.getElementById('creditsModal').classList.remove('open'));
    /* Window › Workspace › Civil: a domain change from MENU reaches the domain tab too */
    await tapEdge(p, '#phTabMenu');
    await tap(p, '#phMenuRoot > .mb-sub[data-menu="window"] > [data-sub]');
    await p.evaluate(() => { const s = [...document.querySelectorAll('#mbWindow .mb-sub > [data-sub]')].find(e => /Workspace/.test(e.textContent)); if (s) s.id = '__probeWs'; });
    await tap(p, '#__probeWs');
    await tap(p, '#mbWindow [data-cmd="domCiv"]');
    A('Window › Workspace › Civil switches the domain, and the domain tab follows (CIVIL)', (await p.evaluate(() => _domain === 'civ' && /civil/i.test(document.getElementById('phTabDomainTx').textContent))));
    await p.evaluate(() => _setDomain('world'));
    /* the world pill opens MENU › File */
    await tap(p, '#phWorldPill');
    A('the world pill opens MENU drilled into File', (await p.evaluate(() => _ph.page === 'menu' && _ph.detent !== 'closed')) && (await vis(p, '#fileMenu [data-cmd="generate"]')) && (await vis(p, '#loadZipBtn')));
    await p.evaluate(() => _phSheetSet(null, 'closed'));
  });

  await section('phone: Search', async () => {
    await tap(p, '#phSearchBtn');
    A('Search opens a search row over the app bar with the Find box focused', (await vis(p, '#phSearch #findInput')) && (await p.evaluate(() => document.activeElement && document.activeElement.id === 'findInput')));
    await p.keyboard.type('Seed'); await sleep(300);
    const hit = await p.evaluate(() => { const rs = document.getElementById('findResults'); const r = rs && [...rs.querySelectorAll('[data-find-i]')].find(e => /^Seed/.test(e.textContent)); if (r) r.id = '__probeSeedHit'; return !!r; });
    A('...it finds Seed', hit);
    if (hit) await tap(p, '#__probeSeedHit');
    await sleep(300);
    const sd = await rectOf(p, '#seedN');
    A('...and a tap on it closes the search and shows Seed on the domain page, on screen', !(await vis(p, '#phSearch')) && !!sd && sd.t >= 0 && sd.b <= 915 && (await p.evaluate(() => _ph.page === 'domain' && _ph.detent !== 'closed')), JSON.stringify(sd));
    await p.evaluate(() => _phSheetSet(null, 'closed'));
  });

  await section('phone: the quick drawer — the pull-out tab', async () => {
    const g = await rectOf(p, '#phGrip'), m = await rectOf(p, '.canvas-wrap');
    A('the grip is at least 44 x 88 and sits at the middle of the map\'s LEFT edge (away from the right hand)', g && g.w >= 44 && g.h >= 88 && Math.abs(g.l - m.l) <= 1 && Math.abs((g.t + g.b) / 2 - (m.t + m.b) / 2) <= 2, JSON.stringify(g));
    await tapEdge(p, '#phGrip');
    const pins = await p.evaluate(() => [...document.querySelectorAll('#phDrawer .shell-slot > button')].filter(e => e.getClientRects().length).map(e => e.id));
    A('a tap opens the drawer of pins: Save, Settlements, Assets, Planner (the bar\'s own four elements)', JSON.stringify(pins) === JSON.stringify(['pinSave', 'settlementsHeaderBtn', 'assetsHeaderBtn', 'pinPlanner']), JSON.stringify(pins));
    await tap(p, '#phDrawer #settlementsHeaderBtn'); await sleep(300);
    A('the Settlements pin opens the Settlements window, and the drawer closes', (await p.evaluate(() => document.getElementById('civFactionsModal').classList.contains('open'))) && !(await vis(p, '#phDrawer')));
    await p.evaluate(() => { const c = document.getElementById('cfmCloseBtn'); if (c) c.click(); });
    await drag(p, g.l + 10, g.t + g.h / 2, g.l + 90, g.t + g.h / 2, 6);
    A('dragging the grip inward opens it too', await vis(p, '#phDrawer'));
    const sc = await rectOf(p, '#scrim');
    if (sc) await p.touchscreen.tap(sc.r - 20, sc.t + 200); await sleep(250);
    A('a tap on the scrim closes it', !(await vis(p, '#phDrawer')));
    await tapEdge(p, '#phTabDomain');
    const gh = await rectOf(p, '#phGrip'), sh = await rectOf(p, '#phSheet'), sb = await rectOf(p, '#scaleBar');
    A('with the sheet at half the grip moves up into the map left above it, clear of the sheet and the scale bar', !!gh && !!sh && gh.b <= sh.t - 12 && (!sb || gh.b <= sb.t || gh.r <= sb.l), JSON.stringify([gh && gh.b, sh && sh.t, sb && sb.t]));
    await tapEdge(p, '#phTabDomain');
  });

  await section('phone: DS-13, the map\'s control row', async () => {
    const R = await p.evaluate(() => { const g = id => { const e = document.getElementById(id); if (!e || !e.getClientRects().length || getComputedStyle(e).display === 'none') return null; const r = e.getBoundingClientRect(); return { l: r.left, r: r.right, t: r.top, b: r.bottom, w: r.width, h: r.height }; };
      return { map: document.querySelector('.canvas-wrap').getBoundingClientRect().toJSON(), reset: g('zoomReset'), inn: g('zoomIn'), out: g('zoomOut'), pan: g('panBtn') }; });
    A('recentre, − and + are drawn; ✋ is not while Inspect is armed (DS13-Q1 default, pending)', R.reset && R.inn && R.out && !R.pan, JSON.stringify(R));
    A('every target is 44 x 44', [R.reset, R.inn, R.out].every(r => r && r.w >= 43.5 && r.h >= 43.5));
    A('from the outer (right) edge: ⌖, then the − | + rocker', R.reset && R.inn && R.out && R.reset.l > R.inn.l && R.inn.l > R.out.l && Math.abs(R.inn.l - R.out.r) <= 1);
    A('12px in from the map\'s right edge, 12px above the bottom bar', R.reset && Math.abs(R.map.right - R.reset.r - 12) <= 1 && Math.abs(R.map.bottom - R.reset.b - 12) <= 1, JSON.stringify([R.map.right - (R.reset || {}).r, R.map.bottom - (R.reset || {}).b]));
    const c0 = await cam(p); await tapEdge(p, '#zoomIn'); const c1 = await cam(p);
    A('+ steps 1.35 (Ruling EB item 2), pressed at its edge', Math.abs(c1.s / c0.s - 1.35) < 0.01, (c1.s / c0.s).toFixed(4));
    await tapEdge(p, '#zoomOut'); const c2 = await cam(p);
    A('− steps back by 1.35', Math.abs(c1.s / c2.s - 1.35) < 0.01, (c1.s / c2.s).toFixed(4));
    await tap(p, '#zoomIn'); await tap(p, '#zoomIn'); await tap(p, '#zoomReset'); const c3 = await cam(p);
    A('⌖ recentres (fits the map again)', Math.abs(c3.s - c0.s) < 0.001, c3.s + ' vs ' + c0.s);
    /* the row rides above the sheet */
    await tapEdge(p, '#phTabMap');
    const hr = await p.evaluate(() => ({ row: document.getElementById('zoomReset').getBoundingClientRect().bottom, sheet: document.getElementById('phSheet').getBoundingClientRect().top }));
    A('with the sheet at half the row sits 12px above it', Math.abs(hr.sheet - hr.row - 12) <= 1, JSON.stringify(hr));
    await tap(p, '#phSheetHandle'); await sleep(250);
    A('at full there is no room, and the row hides (§2.5, When it hides)', !(await vis(p, '#zoomReset')));
    await p.evaluate(() => _phSheetSet(null, 'closed'));
  });

  await section('phone: the tool strip, and ✋ with a drawing tool', async () => {
    await tapEdge(p, '#phTabDomain');
    await tap(p, '#worldCatSeg [data-wcat="sculpt"]'); await sleep(300);
    const S = await p.evaluate(() => ({ strip: !document.getElementById('phStrip').hidden, ctx: document.getElementById('toolOptsCtx').textContent, inStrip: !!document.getElementById('sBrushRow').closest('#phStrip'), commit: !!(document.getElementById('sculptCommitBtn').closest('#phStrip') && document.getElementById('sculptCommitBtn').getClientRects().length), det: _ph.detent }));
    A('arming Sculpt shows the strip: the tool\'s name, its brush (the same rows the desktop bar hosts) and Commit', S.strip && /^Sculpt · /.test(S.ctx) && S.inStrip && S.commit, JSON.stringify(S));
    A('...and drops the sheet to peek (§2.1, "Arming drops the sheet to peek")', S.det === 'peek', S.det);
    const st = await rectOf(p, '#phStrip'), sh = await rectOf(p, '#phSheet'), zr = await rectOf(p, '#zoomReset');
    A('the strip sits above the sheet, and the row above the strip', st && sh && zr && st.b <= sh.t - 7 && zr.b <= st.t - 11, JSON.stringify([st && st.b, sh && sh.t, zr && zr.b, st && st.t]));
    A('✋ appears at the row\'s INNER end, and ⌖ does not move', (await vis(p, '#panBtn')) && (await p.evaluate(() => document.getElementById('panBtn').getBoundingClientRect().right <= document.getElementById('zoomOut').getBoundingClientRect().left)));
    /* a drag with the brush armed draws: it does not pan */
    const [mx, my] = await mapPt(p, 0.5, 0.2);
    const c0 = await cam(p); await drag(p, mx, my, mx - 60, my + 20, 6); const c1 = await cam(p);
    A('with a drawing tool one finger draws — the map does not pan', c0.x === c1.x && c0.y === c1.y, JSON.stringify([c0, c1]));
    await p.evaluate(() => { if (typeof sculptDiscard === 'function') sculptDiscard(); });
    await tap(p, '#panBtn');
    const c2 = await cam(p); await drag(p, mx, my, mx - 60, my + 20, 6); const c3 = await cam(p);
    A('with ✋ on, the same drag pans', c3.x !== c2.x, JSON.stringify([c2.x, c3.x]));
    await tapEdge(p, '#phTabDomain');
    A('at peek, the domain tab\'s tap brings the page back up to half', (await p.evaluate(() => _ph.detent)) === 'half');
    await tap(p, '#worldCatSeg [data-wcat="terrain"]'); await sleep(250);
    const S2 = await p.evaluate(() => ({ strip: !document.getElementById('phStrip').hidden, latch: panMode }));
    A('disarming hides the strip and ✋, and clears the latch (§2.5)', !S2.strip && !S2.latch && !(await vis(p, '#panBtn')), JSON.stringify(S2));
    await p.evaluate(() => _phSheetSet(null, 'closed'));
  });

  await section('phone: gestures with a tap-only tool (Inspect)', async () => {
    await p.evaluate(() => { if (typeof _civSetTool === 'function' && _civTool !== 'inspect') _civSetTool('inspect'); });
    const [mx, my] = await mapPt(p, 0.5, 0.3);
    await tap(p, '#zoomIn');
    const c0 = await cam(p); await drag(p, mx, my, mx - 70, my - 40, 8); const c1 = await cam(p);
    A('one finger drags the map (DS13-Q2 default, pending)', Math.abs((c1.x - c0.x) + 70) <= 2 && Math.abs((c1.y - c0.y) + 40) <= 2, JSON.stringify([c1.x - c0.x, c1.y - c0.y]));
    /* a settlement to tap */
    const at = await p.evaluate(([x, y]) => { const g = evtToGridLOD({ clientX: x, clientY: y }); const gx = Math.round(g[0]), gy = Math.round(g[1]);
      for (let dy = 0; dy < 40; dy += 3) for (let dx = 0; dx < 40; dx += 3) { const n0 = state.places.length; _civDropPlace(gx + dx, gy + dy); if (state.places.length > n0) { _civSelectedPlace = null; _civClosePlacePopup(); return _civPlaceScreenPos(gx + dx, gy + dy); } } return null; }, [mx, my]);
    A('(set-up: a settlement on land under the finger)', !!at);
    if (at) {
      const b0 = await cam(p);
      await drag(p, at[0] + 30, at[1] + 30, at[0] + 30 - 50, at[1] + 30, 6);
      const b1 = await cam(p);
      A('a drag that starts beside it pans and selects nothing', b1.x !== b0.x && (await p.evaluate(() => !_civSelectedPlace)), JSON.stringify([b0.x, b1.x]));
      const pos = await p.evaluate(() => { const q = state.places[state.places.length - 1]; return _civPlaceScreenPos(q.x, q.y); });
      await p.touchscreen.tap(pos[0], pos[1]);
      A('a tap waits for a second tap (240ms) before it acts…', (await p.evaluate(() => !_civSelectedPlace)));
      await sleep(400);
      A('…then selects the settlement under it', (await p.evaluate(() => !!_civSelectedPlace)));
      await clearSel(p);
    }
    /* the long-press card */
    const [hx, hy] = await mapPt(p, 0.55, 0.3);
    await hold(p, hx, hy, 650);
    const card = await p.evaluate(([x, y]) => { const m = document.querySelector('.civ-ctx'); if (!m) return null; const r = m.getBoundingClientRect();
      return { rows: [...m.querySelectorAll('button')].map(b => b.textContent), under: x >= r.left && x <= r.right && y >= r.top && y <= r.bottom, minH: Math.min(...[...m.querySelectorAll('button')].map(b => b.getBoundingClientRect().height)) }; }, [hx, hy]);
    A('a 480ms hold opens the map menu\'s own rows as a card (one implementation)', !!card && card.rows.some(r => /Drop settlement here/.test(r)) && card.rows.some(r => /Info here/.test(r)), JSON.stringify(card && card.rows));
    A('...not under the finger, with 48px rows, and the lift pressed nothing', !!card && !card.under && card.minH >= 47.5 && (await p.evaluate(() => !_civSelectedPlace)), JSON.stringify(card));
    if (card) { const id = await p.evaluate(() => { const b = [...document.querySelectorAll('.civ-ctx button')].find(x => /Info here/.test(x.textContent)); b.id = '__probeInfo'; return 1; }); await tap(p, '#__probeInfo'); }
    A('the card\'s Info here opens the context page with the reading', (await p.evaluate(() => _ph.page === 'context' && _ph.detent !== 'closed')) && (await p.evaluate(() => !!document.getElementById('civInfoPanel').closest('#phPageContext'))));
    await p.evaluate(() => _phSheetSet(null, 'closed'));
    /* the double-tap and thumb zoom (Ruling EB item 3) */
    /* start two − steps below the fitted view, so ×1.7 and ×2 stay under the camera's 8x cap (zoomAt) */
    await tap(p, '#zoomReset'); await tap(p, '#zoomOut'); await tap(p, '#zoomOut');
    const [dx, dy] = await mapPt(p, 0.5, 0.3);
    const s0 = (await cam(p)).s;
    await T(p, 'touchStart', [[dx, dy]]); await sleep(40); await T(p, 'touchEnd', []); await sleep(90);
    await T(p, 'touchStart', [[dx + 2, dy + 2]]); await sleep(40); await T(p, 'touchEnd', []); await sleep(450);
    const s1 = (await cam(p)).s;
    A('a double-tap zooms ×1.7 at the finger', Math.abs(s1 / s0 - 1.7) < 0.02, (s1 / s0).toFixed(3));
    await tap(p, '#zoomReset'); await tap(p, '#zoomOut'); await tap(p, '#zoomOut');
    const s2 = (await cam(p)).s;
    await T(p, 'touchStart', [[dx, dy]]); await sleep(40); await T(p, 'touchEnd', []); await sleep(90);
    await T(p, 'touchStart', [[dx, dy]]); for (let i = 1; i <= 12; i++) { await T(p, 'touchMove', [[dx, dy + i * 10]]); await sleep(16); }
    await sleep(700);
    const mid = await p.evaluate(() => !!document.querySelector('.civ-ctx'));
    await T(p, 'touchEnd', []); await sleep(250);
    const s3 = (await cam(p)).s;
    /* the zoom is measured from where the slide begins, past the 10px slop — the first 10px step that
       exceeds it lands at +20 — and doubles every 120px: 2^((120-20)/120) */
    const want = Math.pow(2, (120 - 20) / 120);
    A('double-tap, hold and slide DOWN zooms in, doubling every 120px past the slop ("Google Maps style")', Math.abs(s3 / s2 - want) < 0.03, (s3 / s2).toFixed(3) + ' vs ' + want.toFixed(3));
    A('...and that held press opens no card', !mid);
    await T(p, 'touchStart', [[dx, dy]]); await sleep(40); await T(p, 'touchEnd', []); await sleep(90);
    await T(p, 'touchStart', [[dx, dy]]); for (let i = 1; i <= 6; i++) { await T(p, 'touchMove', [[dx, dy - i * 10]]); await sleep(16); } await T(p, 'touchEnd', []); await sleep(250);
    const s4 = (await cam(p)).s;
    A('sliding UP zooms out', s4 < s3 * 0.8, (s4 / s3).toFixed(3));
  });

  await section('phone: gestures with a drawing tool (Territory) — and Krita\'s guard', async () => {
    await tap(p, '#zoomReset');
    await p.evaluate(() => { _setDomain('civ'); _civSetTool('territory'); _civTerRadius = 3; if (!civTerritory || civTerritory.length !== GW * GH) civTerritory = new Uint8Array(GW * GH); _civActiveFaction = 1; });
    await sleep(200);
    const sum = () => p.evaluate(() => { let s = 0; for (let i = 0; i < civTerritory.length; i++) s += civTerritory[i]; return s; });
    const [mx, my] = await mapPt(p, 0.5, 0.25);
    const s0 = (await cam(p)).s, t0 = await sum();
    await T(p, 'touchStart', [[mx, my]]); await sleep(40); await T(p, 'touchEnd', []); await sleep(90);
    await T(p, 'touchStart', [[mx, my]]); await sleep(40); await T(p, 'touchEnd', []); await sleep(450);
    A('a double-tap with a drawing tool armed does not zoom (pending the owner, §6)', (await cam(p)).s === s0);
    const t1 = await sum();
    /* two fingers down, one lifts, the other moves: nothing of that touch may paint */
    await T(p, 'touchStart', [[mx, my, 0]]); await sleep(30);
    await T(p, 'touchStart', [[mx, my, 0], [mx + 80, my + 10, 1]]); await sleep(30);
    await T(p, 'touchMove', [[mx - 20, my, 0], [mx + 100, my + 10, 1]]); await sleep(30);
    await T(p, 'touchEnd', [[mx - 20, my, 0]]); await sleep(30);
    for (let i = 1; i <= 6; i++) { await T(p, 'touchMove', [[mx - 20 - i * 12, my + i * 6, 0]]); await sleep(20); }
    const t2 = await sum();
    await T(p, 'touchEnd', []); await sleep(250);
    A('after two fingers have been down, the remaining finger paints nothing (Krita\'s guard)', t2 === t1, t1 + ' → ' + t2);
    const t3 = await sum(); await drag(p, mx, my + 40, mx - 60, my + 60, 6); const t4 = await sum();
    A('...while a plain one-finger drag still paints territory (the guard is not a blanket block)', t4 !== t3, t3 + ' → ' + t4);
    await p.evaluate(() => { _civSetTool('inspect'); _setDomain('world'); });
  });

  await section('phone: Handedness mirrors the row, the grip and the card', async () => {
    await tapEdge(p, '#phTabMenu');
    await tap(p, '#phMenuRoot > .mb-sub[data-menu="prefs"] > [data-sub]');
    await p.evaluate(() => { const s = [...document.querySelectorAll('#mbPrefs .mb-sub > [data-sub]')].find(e => /Handedness/.test(e.textContent)); if (s) s.id = '__probeHand'; });
    await tap(p, '#__probeHand');
    await tap(p, '#mbPrefs [data-cmd="handLeft"]'); await sleep(250);
    const L = await p.evaluate(() => { const m = document.querySelector('.canvas-wrap').getBoundingClientRect(), z = document.getElementById('zoomReset').getBoundingClientRect(), o = document.getElementById('zoomOut').getBoundingClientRect(), g = document.getElementById('phGrip').getBoundingClientRect();
      return { left: document.documentElement.classList.contains('hand-left'), rowIn: Math.round(z.left - m.left), order: z.left < o.left, grip: Math.round(m.right - g.right), saved: (() => { try { return localStorage.getItem('cartalith_hand'); } catch (_) { return null; } })() }; });
    A('Preferences › Handedness › Left hand moves the row to the LEFT edge, 12px in, ⌖ outermost', L.left && L.rowIn === 12 && L.order, JSON.stringify(L));
    A('...and the grip to the RIGHT edge (away from the dominant hand, Ruling DY Q11)', L.grip === 0, JSON.stringify(L));
    /* near the map's left edge, where a card fits to the finger's right (it falls back above or below the
       finger where it fits on neither side, as a 412px phone is narrower than finger + card + finger) */
    const [hx, hy] = await mapPt(p, 0.06, 0.3);
    await hold(p, hx, hy, 650);
    const cs = await p.evaluate(x => { const m = document.querySelector('.civ-ctx'); return m ? m.getBoundingClientRect().left > x : null; }, hx);
    A('...and the card opens to the finger\'s RIGHT', cs === true, cs);
    await clearSel(p);
    A('the choice is kept in this browser (localStorage)', L.saved === 'left');
    /* a fresh page in the same browser profile: navigating this one away is refused by the file's own
       beforeunload guard while a world is open */
    const p2 = await p._ctx.newPage(); await p2.goto('file://' + path.resolve(FILE)); await p2.waitForFunction(() => typeof window.generate === 'function', { timeout: 120000 });
    A('...and is there when the file is opened again', await p2.evaluate(() => document.documentElement.classList.contains('hand-left')));
    await p2.close();
    await p.evaluate(() => _handSet('right'));
  });

  await section('phone: the 44px floor on every surface, and windows as sheets', async () => {
    const roots = ['header', '#phBar', '#zoomOverlay', '#phGrip'];
    const bad = [], seen = { n: 0 };
    const check = async (label, rs) => { const s = await smallTargets(p, rs); seen.n += await countTargets(p, rs); for (const x of s) bad.push(label + ': ' + x); };
    await check('chrome', roots);
    for (const d of ['world', 'civ', 'carto', 'explore']) {
      await p.evaluate(d => { _setDomain(d); _phSheetSet('domain', 'full'); document.querySelectorAll('#phPageDomain details').forEach(x => x.open = true); }, d); await sleep(200);
      await check('domain ' + d, ['#phPageDomain']);
    }
    await p.evaluate(() => { _setDomain('world'); _phSheetSet('map', 'full'); }); await sleep(150); await check('map', ['#phPageMap']);
    await p.evaluate(() => { _phSheetSet('story', 'full'); document.querySelectorAll('#phPageStory details').forEach(x => x.open = true); }); await sleep(150); await check('story', ['#phPageStory']);
    await p.evaluate(() => { _phSheetSet('menu', 'full'); }); await sleep(150); await check('menu', ['#phPageMenu']);
    await p.evaluate(() => { _phSheetSet('context', 'full'); }); await sleep(150); await check('context', ['#phPageContext']);
    await p.evaluate(() => { _phSheetSet(null, 'closed'); _phDrawerSet(true); }); await sleep(150); await check('drawer', ['#phDrawer']);
    await p.evaluate(() => { _phDrawerSet(false); _setSculptCategory(true); }); await sleep(250); await check('strip', ['#phStrip']);
    await p.evaluate(() => { _setSculptCategory(false); window._openSettings('perf'); }); await sleep(200);
    await check('settings', ['#settingsModal']);
    const set = await p.evaluate(() => Math.round(document.querySelector('#settingsModal .set-shell').getBoundingClientRect().height));
    await p.evaluate(() => { const c = document.getElementById('settingsClose'); if (c) c.click(); });
    A('Settings opens as a full-height sheet', set >= 914, set);
    A('the floor walk measured a real population of targets (' + seen.n + ')', seen.n > 300, seen.n);
    A('every one of them is at least 44 x 44 (both axes)', bad.length === 0, bad.slice(0, 8).join(' | ') + (bad.length > 8 ? ' … +' + (bad.length - 8) : ''));
  });

  await p._ctx.close();

  /* ════════════════════════ THE PHONE, 360 x 780, AND LANDSCAPE 844 x 390 ════════════════════════ */
  await section('phone 360 x 780: the bar, the sheet and the grip fit', async () => {
    const q = await boot(b, { width: 360, height: 780 }, true); await gen(q); await sleep(250);
    await tapEdge(q, '#phTabDomain');
    const ov = await overflow(q);
    A('360: with the domain sheet open nothing runs past the edges', ov.length === 0, ov.join('; '));
    const s = await smallTargets(q, ['header', '#phBar', '#zoomOverlay', '#phGrip', '#phSheet']);
    A('360: no target under 44 x 44 in the app bar, the bar, the row, the grip or the sheet', s.length === 0, s.slice(0, 6).join(' | '));
    const g = await rectOf(q, '#phGrip'), z = await rectOf(q, '#zoomReset');
    A('360: the grip and the row never meet', !!g && !!z && (g.r <= z.l || g.b <= z.t), JSON.stringify([g, z]));
    await q._ctx.close();
  });
  await section('phone landscape 844 x 390: the rail, the side sheet, the row', async () => {
    const q = await boot(b, { width: 844, height: 390 }, true); await gen(q); await sleep(250);
    const bar = await rectOf(q, '#phBar');
    A('the bar is an 80px rail on the left, full height', !!bar && Math.round(bar.w) === 80 && bar.l === 0 && Math.round(bar.h) === 390, JSON.stringify(bar));
    await tapEdge(q, '#phTabDomain');
    const sh = await rectOf(q, '#phSheet'), z = await rectOf(q, '#zoomReset');
    A('the domain sheet docks on the right, 300-400px wide', !!sh && Math.abs(sh.r - 844) <= 1 && sh.w >= 300 && sh.w <= 400, JSON.stringify(sh));
    A('...and the row clears it by 12px', !!sh && !!z && Math.abs(sh.l - z.r - 12) <= 1, JSON.stringify([sh && sh.l, z && z.r]));
    const g = await rectOf(q, '#phGrip');
    A('the grip sits beside the rail, on the map\'s left edge', !!g && Math.abs(g.l - 80) <= 1, JSON.stringify(g));
    const ov = await overflow(q);
    A('landscape: nothing runs past the edges', ov.length === 0, ov.join('; '));
    await q._ctx.close();
  });

  /* ════════════════════════ THE TABLET ════════════════════════ */
  await section('tablet 1194 x 834: the desktop shell at touch size, and the docking rule', async () => {
    const q = await boot(b, { width: 1194, height: 834 }, true); await gen(q); await sleep(250);
    const menus = await q.evaluate(() => [...document.querySelectorAll('.mb-btn')].filter(e => e.getClientRects().length).map(e => e.textContent.trim()));
    A('at 1194 the bar is whole: the eight menus', JSON.stringify(menus) === JSON.stringify(['File', 'Edit', 'Assets', 'Records', 'Story', 'Preferences', 'Window', 'Help']), JSON.stringify(menus));
    const D = await q.evaluate(() => ({ lf: document.body.classList.contains('ldock-float'), rf: document.body.classList.contains('ctx-float'),
      map: Math.round(document.querySelector('.canvas-wrap').getBoundingClientRect().width), ctx: Math.round(document.getElementById('dockRight').getBoundingClientRect().width) }));
    A('the left dock stays docked (the map keeps ≥520px), the Context panel floats as a 44px edge tab', !D.lf && D.rf && D.ctx === 44 && D.map >= 520, JSON.stringify(D));
    await tap(q, '#ctxHideBtn');
    const O = await q.evaluate(() => ({ open: document.body.classList.contains('ctx-open'), w: Math.round(document.getElementById('dockRight').getBoundingClientRect().width), map: Math.round(document.querySelector('.canvas-wrap').getBoundingClientRect().width), shadow: getComputedStyle(document.getElementById('dockRight')).boxShadow !== 'none' }));
    A('its chevron opens it OVER the map, with a shadow — the map keeps its width', O.open && O.w >= 300 && O.map === D.map && O.shadow, JSON.stringify(O));
    const [mx, my] = await mapPt(q, 0.3, 0.4);
    await q.touchscreen.tap(mx, my); await sleep(250);
    A('a press on the map closes it (Ruling DU Q8)', O.open && !(await q.evaluate(() => document.body.classList.contains('ctx-open'))));
    await tap(q, '#ctxHideBtn'); await tap(q, '#ctxPinBtn');
    await q.touchscreen.tap(mx, my); await sleep(250);
    A('...unless it is pinned', await q.evaluate(() => document.body.classList.contains('ctx-open')));
    await tap(q, '#ctxPinBtn'); await tap(q, '#ctxHideBtn');
    /* menus: 48px rows, and a submenu drills in */
    await tap(q, '#fileMenuBtn');
    const fr = await q.evaluate(() => [...document.querySelectorAll('#fileMenu .mb-row, #fileMenu > button')].filter(e => e.getClientRects().length).map(e => e.getBoundingClientRect().height));
    A('a menu\'s rows are 48px', fr.length > 5 && fr.every(h => h >= 47.5), JSON.stringify(fr));
    await tap(q, '#mbExportRow');
    const dr = await q.evaluate(() => ({ form: !!document.getElementById('bakeRes').getClientRects().length, load: !!document.getElementById('loadZipBtn').getClientRects().length, sub: getComputedStyle(document.getElementById('mbExportPanel')).position }));
    A('File › Export drills in: the form replaces File\'s rows (not a cascade beside them)', dr.form && !dr.load && dr.sub === 'static', JSON.stringify(dr));
    await q.evaluate(() => _dskMenuClose());
    /* DS-13 on the tablet */
    const R = await q.evaluate(() => { const m = document.querySelector('.canvas-wrap').getBoundingClientRect(), z = document.getElementById('zoomReset').getBoundingClientRect(), v = document.getElementById('viewDimSeg').getBoundingClientRect();
      return { right: Math.round(m.right - z.right), bottom: Math.round(m.bottom - z.bottom), w: z.width, h: z.height, dimAbove: v.bottom <= z.top, pan: getComputedStyle(document.getElementById('panBtn')).display }; });
    A('DS-13 sits at the map\'s bottom right, 12px in and 14px up, 44px targets, 2D/3D above it, no ✋ with Inspect', R.right === 12 && R.bottom === 14 && R.w >= 43.5 && R.dimAbove && R.pan === 'none', JSON.stringify(R));
    const [hx, hy] = await mapPt(q, 0.4, 0.4);
    await hold(q, hx, hy, 650);
    A('a 500ms hold opens the map menu as a card on the tablet too', await q.evaluate(() => !!document.querySelector('.civ-ctx')));
    await clearSel(q);
    const s = await smallTargets(q, ['header', '.toolopts', '#domainRail', '#dockLeft', '.status-bar', '#zoomOverlay', '.map-tl', '#tlStrip']);
    A('the 44px floor holds on the tablet\'s bars, rail, dock and map controls', s.length === 0, s.slice(0, 8).join(' | '));
    const ov = await overflow(q);
    A('nothing runs past the edges', ov.length === 0, ov.join('; '));
    await q._ctx.close();
  });

  await section('the narrow bar at 600, 768 and 1024 (touch), and the World menu', async () => {
    for (const [w, h] of [[600, 960], [768, 1024], [1024, 768]]) {
      const q = await boot(b, { width: w, height: h }, true); await gen(q); await sleep(250);
      const M = await q.evaluate(() => { const items = [...document.querySelectorAll('header > *, .menubar > *, .menubar .mb-btn')].filter(e => e.getClientRects().length && getComputedStyle(e).display !== 'contents' && !e.classList.contains('menubar') && !e.classList.contains('mb-wrap')).map(e => ({ id: e.id || e.tagName, r: e.getBoundingClientRect().toJSON() }));
        const hdr = document.querySelector('header').getBoundingClientRect(); const over = [];
        for (let i = 0; i < items.length; i++) for (let j = i + 1; j < items.length; j++) { const a = items[i].r, c = items[j].r; if (a.left < c.right - 1 && c.left < a.right - 1 && a.top < c.bottom - 1 && c.top < a.bottom - 1) over.push(items[i].id + '/' + items[j].id); }
        return { menus: [...document.querySelectorAll('.mb-btn')].filter(e => e.getClientRects().length).map(e => e.textContent.trim()), over, outside: items.filter(x => x.r.right > hdr.right + 1).map(x => x.id) }; });
      A(w + ': the bar is ☰ · File · World · Records · Story', JSON.stringify(M.menus) === JSON.stringify(['☰', 'File', 'World', 'Records', 'Story']), JSON.stringify(M.menus));
      A(w + ': nothing on the bar overlaps, and nothing runs past it', M.over.length === 0 && M.outside.length === 0, M.over.concat(M.outside).join(','));
      if (w === 768) {
        await tap(q, '#mbMoreBtn');
        const more = await q.evaluate(() => [...document.querySelectorAll('#mbMore > .mb-sub > [data-sub]')].filter(e => e.getClientRects().length).map(e => e.textContent.replace(/[▸›]/g, '').trim()));
        A('768: ☰ holds Edit, Assets, Preferences, Window, Help', JSON.stringify(more) === JSON.stringify(['Edit', 'Assets', 'Preferences', 'Window', 'Help']), JSON.stringify(more));
        await tap(q, '#mbMore > .mb-sub[data-menu="edit"] > [data-sub]');
        await tap(q, '#mbEdit [data-cmd="find"]');
        A('768: ☰ › Edit › Find… presses Find (the Find box takes focus)', await q.evaluate(() => document.activeElement && document.activeElement.id === 'findInput'));
        await q.keyboard.press('Escape');
        await tap(q, '#fileMenuBtn');
        A('768: File leaves out Generate world on this bar (World holds it)', !(await vis(q, '#fileMenu [data-cmd="generate"]')) && (await vis(q, '#loadZipBtn')) && (await vis(q, '#mbWorldBtn')));
        await q.evaluate(() => _dskMenuClose());
        await tap(q, '#mbWorldBtn');
        const wr = await q.evaluate(() => [...document.querySelectorAll('#mbWorld [data-cmd]')].filter(e => e.getClientRects().length).map(e => e.dataset.cmd));
        A('768: World holds Generate world, the Generate | Sculpt category and Seed & size…', JSON.stringify(wr) === JSON.stringify(['generate', 'catGenerate', 'catSculpt', 'seedSize']), JSON.stringify(wr));
        await tap(q, '#mbWorld [data-cmd="catSculpt"]');
        A('768: World › Sculpt opens WORLD\'s Sculpt category (the real control)', await q.evaluate(() => _domain === 'world' && _sculptCategoryOpen));
        await q.evaluate(() => _setSculptCategory(false));
        const D = await q.evaluate(() => ({ lf: document.body.classList.contains('ldock-float'), rf: document.body.classList.contains('ctx-float'), lw: Math.round(document.getElementById('dockLeft').getBoundingClientRect().width) }));
        A('768: both docks float (the map would keep under 520px), the left one a 44px strip', D.lf && D.rf && D.lw === 44, JSON.stringify(D));
      }
      await q._ctx.close();
    }
  });

  /* ════════════════════════ THE DESKTOP, WITH A MOUSE ════════════════════════ */
  await section('desktop 1366 x 768 (mouse): stage 1 kept, the touch layer silent', async () => {
    const q = await boot(b, { width: 1366, height: 768 }, false); await gen(q); await sleep(250);
    const D = await q.evaluate(() => ({ lf: document.body.classList.contains('ldock-float'), rf: document.body.classList.contains('ctx-float'), pan: getComputedStyle(document.getElementById('panBtn')).display, col: getComputedStyle(document.getElementById('zoomOverlay')).flexDirection }));
    A('both docks docked, the zoom controls stage 1\'s column, no ✋', !D.lf && !D.rf && D.pan === 'none' && D.col === 'column', JSON.stringify(D));
    const [mx, my] = await mapPt(q, 0.5, 0.4);
    const c0 = await cam(q); await q.mouse.move(mx, my); await q.mouse.down(); await q.mouse.move(mx - 60, my - 30, { steps: 6 }); await q.mouse.up(); await sleep(200); const c1 = await cam(q);
    A('a mouse drag with Inspect does not pan (the touch gestures never touch a mouse)', c0.x === c1.x && c0.y === c1.y);
    await q.mouse.click(mx, my, { button: 'right' }); await sleep(200);
    A('a right-click still opens the map menu at the pointer', await q.evaluate(() => !!document.querySelector('.civ-ctx')));
    await q._ctx.close();
  });
  await section('desktop 700 x 900 (mouse): the narrow bar and floating docks', async () => {
    const q = await boot(b, { width: 700, height: 900 }, false); await gen(q); await sleep(250);
    const M = await q.evaluate(() => ({ menus: [...document.querySelectorAll('.mb-btn')].filter(e => e.getClientRects().length).map(e => e.textContent.trim()), lf: document.body.classList.contains('ldock-float'), rf: document.body.classList.contains('ctx-float') }));
    A('700: the narrow bar (☰ · File · World · Records · Story) and both docks floating', JSON.stringify(M.menus) === JSON.stringify(['☰', 'File', 'World', 'Records', 'Story']) && M.lf && M.rf, JSON.stringify(M));
    await q.click('#dockLeftCollapse'); await sleep(200);
    const o = await q.evaluate(() => ({ open: document.body.classList.contains('ldock-open'), w: Math.round(document.getElementById('dockLeft').getBoundingClientRect().width) }));
    A('700: its chevron opens the left dock over the map at its full width', o.open && o.w >= 360, JSON.stringify(o));
    const [mx, my] = await mapPt(q, 0.8, 0.5);
    await q.mouse.click(mx, my); await sleep(200);
    A('700: a click on the map closes it', !(await q.evaluate(() => document.body.classList.contains('ldock-open'))));
    await q._ctx.close();
  });

  A('no page errors and no console errors anywhere', errors.length === 0, errors.slice(0, 4).join(' | '));
  } finally { await b.close(); }
  console.log('\n' + pass + ' passed, ' + fail + ' failed');
  process.exit(fail ? 1 : 0);
})();
