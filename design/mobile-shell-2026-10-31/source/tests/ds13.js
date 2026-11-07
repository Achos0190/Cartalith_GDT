// DS-13: the map's control row on touch (MOBILE_SHELL_PROPOSAL.md §2.5), checked on the phone (portrait and
// landscape) and the tablet, right- and left-handed. For each: the row sits on the dominant-hand side at the map's
// bottom edge, overlaps nothing (quick grip, scale bar, bottom bar, open sheet, tool strip), every target is at least
// 44 x 44, zoom and recentre act, the pan latch shows only while a drawing tool is armed and adds itself at the inner
// end without moving recentre, with the latch on a one-finger drag pans instead of drawing, and 3D is reached where
// §2.5 moved it (phone MAP ▸ View, tablet Window ▸ 3D view). Writes ds13-*.png. Exits 1 on any failed check or page
// error.
// Ruling EB (2026-11-01) adds, on fresh pages: the rocker and the desktop pointer column step by exactly 1.35 (item 2);
// a double-tap zooms ×1.7 at the finger; a double-tap held and slid zooms continuously, down in and up out, about the
// finger (item 3); a double-tap's held press opens no card; with a drawing tool armed neither changes the zoom and the
// map says "pending DS13-Q1"; and the long-press, the two-finger-tap undo and the pinch still act.
const { chromium } = require('playwright');
// SHOT_DIR redirects the captures; set it for a mutation run, or the mutant's frames overwrite the evidence.
const OUT = process.env.SHOT_DIR || '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/mobile-shell-2026-10-31/';
// BASE serves another build (an older canvas or a mutant), to show a check can fail.
const BASE = process.env.BASE || 'http://127.0.0.1:8770/';
let bad = 0;
const check = (ok, what) => { console.log((ok ? 'ok   ' : 'FAIL ') + what); if (!ok) bad += 1; };
const MAP = '[aria-label^="map: drag to pan"]';
// The component's view scale, its accessible name, and the world point (km) under a page point, read through React's
// fiber on the map host (the DC runtime keeps the logic object as the wrapper's stateNode.logic). It only reads.
// A missing component gives null, which every caller treats as a failure, never as a value.
const viewAt = (p, pt) => p.locator(MAP).first().evaluate((el, a) => {
  const key = Object.getOwnPropertyNames(el).find((k) => k.startsWith('__reactFiber'));
  let f = key ? el[key] : null; while (f && !(f.stateNode && f.stateNode.logic && f.stateNode.logic._s2w)) f = f.return;
  if (!f) return null; const L = f.stateNode.logic, r = el.getBoundingClientRect();
  const w = a ? L._s2w(a.x - r.left, a.y - r.top) : null;
  return { s: L.view.s, w, aria: el.getAttribute('aria-label') };
}, pt || null);
// Touch input as the mockup's own gesture code reads it: pointer events with pointerType 'touch' dispatched on the map
// host, with waits between them in the page. Steps: { t, id, x, y } or { wait: ms }.
const touchSeq = (p, steps) => p.locator(MAP).first().evaluate(async (el, a) => {
  const ev = (type, id, x, y) => el.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, pointerId: id, pointerType: 'touch', isPrimary: id === 1, clientX: x, clientY: y, button: 0, buttons: type === 'pointerup' ? 0 : 1 }));
  for (const s of a) { if (s.wait) await new Promise((r) => setTimeout(r, s.wait)); else ev(s.t, s.id || 1, s.x, s.y); }
}, steps);
const tapSteps = (pt) => [{ t: 'pointerdown', x: pt.x, y: pt.y }, { wait: 40 }, { t: 'pointerup', x: pt.x, y: pt.y }];
// A double-tap: the second press lands 90 ms after the first lifts, inside the 240 ms the first tap waits.
const dblSteps = (pt) => tapSteps(pt).concat([{ wait: 90 }], tapSteps(pt));
// A double-tap whose second press stays down and slides dy px (down positive) past the 10 px slop, in 2 px steps: the
// zoom is measured from the first move past the slop (12 px), so the slide that counts is exactly dy.
const holdSlideSteps = (pt, dy) => { const sg = Math.sign(dy), n = (12 + Math.abs(dy)) / 2; const out = tapSteps(pt).concat([{ wait: 90 }, { t: 'pointerdown', x: pt.x, y: pt.y }]); for (let i = 1; i <= n; i++) out.push({ t: 'pointermove', x: pt.x, y: pt.y + sg * i * 2 }); out.push({ t: 'pointerup', x: pt.x, y: pt.y + sg * n * 2 }); return out; };
const near = (a, b, tol) => Math.abs(a - b) <= tol;
const moved = (a, b) => (a && b && a.w && b.w ? Math.hypot(a.w.x - b.w.x, a.w.y - b.w.y) : Infinity);
const overlap = (a, b) => !!(a && b && a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height);
const box = async (loc) => ((await loc.count()) && (await loc.first().isVisible()) ? loc.first().boundingBox() : null);
(async () => {
  const b = await chromium.launch(); const errs = [];
  const configs = [
    ['phone-portrait', 412, 892, 'right'], ['phone-portrait', 412, 892, 'left'],
    ['phone-landscape', 892, 412, 'right'], ['phone-landscape', 892, 412, 'left'],
    ['tablet', 1194, 834, 'right'], ['tablet', 1194, 834, 'left'],
  ];
  // DS13_ONLY=<tag> runs one configuration, for re-running a mutation survivor in isolation.
  for (const [form, w, h, hand] of configs.filter(([f, , , hd]) => !process.env.DS13_ONLY || process.env.DS13_ONLY === f + '-' + hd)) {
    const tag = form + '-' + hand;
    const p = await b.newPage({ viewport: { width: w, height: h } });
    p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message));
    p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); });
    await p.goto(BASE + 'Live.dc.html'); await p.waitForTimeout(3000);
    const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
    const phone = form !== 'tablet';
    const cell = async (id) => { const r = await p.locator('[data-navcell="' + id + '"]').boundingBox(); await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await p.waitForTimeout(450); };
    const sheet = p.getByRole('dialog', { name: /^(MAP|WORLD|CIVIL|CARTO|STORY|MENU)\b/ }).filter({ visible: true });
    // A touch tablet's bar menu lays a full-window scrim under itself; a tap on the scrim, outside the menu, closes it.
    const closeMenu = async (title) => { const mb = await box(p.getByRole('menu', { name: title }).filter({ visible: true })); if (!mb) return; await p.mouse.click(mb.x > 100 ? 40 : w - 40, Math.round(h / 2)); await p.waitForTimeout(300); };
    // Handedness lives in Preferences on both forms; its two radios are drawn inline as a segmented row.
    if (hand === 'left') {
      if (phone) {
        await cell('menu'); await p.getByText('Preferences', { exact: true }).first().click(); await p.waitForTimeout(350);
        await p.getByRole('radio', { name: 'Left', exact: true }).first().click(); await p.waitForTimeout(300);
        await cell('menu');
        check((await sheet.count()) === 0, tag + ': MENU closes again after Handedness ▸ Left');
      } else {
        const prefs = p.getByRole('button', { name: 'Preferences', exact: true }).first();
        await prefs.click(); await p.waitForTimeout(300);
        await p.getByRole('radio', { name: 'Left', exact: true }).first().click(); await p.waitForTimeout(300);
        await closeMenu('PREFERENCES');
        check((await p.getByRole('menu', { name: 'PREFERENCES' }).filter({ visible: true }).count()) === 0, tag + ': the Preferences menu closes again');
      }
    }
    const mapBox = await p.locator('[aria-label^="map: drag to pan"]').first().boundingBox();
    const mc = p.locator('[data-mc="1"]').filter({ visible: true });
    const row = await box(mc);
    check(!!row, tag + ': the control row is drawn');
    if (!row) { await p.close(); continue; }
    const edgeGap = hand === 'left' ? row.x - mapBox.x : (mapBox.x + mapBox.width) - (row.x + row.width);
    check(edgeGap >= 10 && edgeGap <= 14, tag + ': on the ' + hand + ' side, ' + Math.round(edgeGap) + ' px from the map edge');
    const bottomGap = (mapBox.y + mapBox.height) - (row.y + row.height);
    check(bottomGap >= 8, tag + ': above the map bottom by ' + Math.round(bottomGap) + ' px');
    // Nothing it may collide with: the quick grip, the scale bar, the bottom bar's cells.
    const grip = await box(p.getByRole('button', { name: /quick drawer/i }));
    check(!overlap(row, grip), tag + ': clear of the quick-drawer grip');
    const scaleLabel = p.locator('span').filter({ hasText: /^\d[\d ,.]* (km|mi)$/ }).filter({ visible: true });
    const sc = await box(scaleLabel);
    check(!overlap(row, sc), tag + ': clear of the scale bar');
    if (sc) check(hand === 'left' ? sc.x > row.x + row.width : sc.x + sc.width < row.x, tag + ': the scale bar is on the other side');
    for (const id of ['map', 'dom', 'story', 'menu']) { const nb = await box(p.locator('[data-navcell="' + id + '"]')); check(!overlap(row, nb), tag + ': clear of the ' + id + ' cell'); }
    // 44 x 44 on every target in the row.
    const small = await mc.first().evaluate((el) => [...el.querySelectorAll('button')].map((x) => x.getBoundingClientRect()).filter((r) => r.width && (r.width < 44 || r.height < 44)).length);
    check(small === 0, tag + ': every target in the row is at least 44 x 44');
    // Zoom and recentre act. The scale bar keeps a round label and grows its bar, so read both.
    const scale = async () => ((await scaleLabel.count()) ? scaleLabel.first().evaluate((el) => { const host = el.classList.contains('sc-interp') ? el.parentElement : el; const bar = host.nextElementSibling; return host.textContent.trim() + ' / ' + (bar ? Math.round(bar.getBoundingClientRect().width) : 0) + ' px'; }) : '');
    const s0 = await scale(); const z0 = await viewAt(p);
    await mc.first().getByRole('button', { name: 'zoom in' }).click(); await p.waitForTimeout(700);
    const s1 = await scale(); const z1 = await viewAt(p);
    check(s0 !== s1, tag + ': zoom in changes the scale (' + s0 + ' → ' + s1 + ')');
    // Ruling EB item 2: the step is ZOOM_BUTTON_STEP, 1.35, the reference's, not the mockup's former 1.6.
    check(!!(z0 && z1) && near(z1.s / z0.s, 1.35, 0.005), tag + ': + zooms by exactly 1.35 (×' + (z0 && z1 ? (z1.s / z0.s).toFixed(4) : '?') + ')');
    await mc.first().getByRole('button', { name: 'zoom out' }).click(); await p.waitForTimeout(700);
    const s1b = await scale(); const z1b = await viewAt(p);
    check(s1b !== s1, tag + ': zoom out changes it back the other way (' + s1b + ')');
    check(!!(z1 && z1b) && near(z1b.s / z1.s, 1 / 1.35, 0.005), tag + ': − zooms by exactly 1 / 1.35 (×' + (z1 && z1b ? (z1b.s / z1.s).toFixed(4) : '?') + ')');
    await mc.first().getByRole('button', { name: 'zoom in' }).click(); await p.waitForTimeout(700);
    await mc.first().getByRole('button', { name: /recentre/ }).click(); await p.waitForTimeout(900);
    const s2 = await scale();
    check(s2 === s0, tag + ': recentre fits the world again (' + s2 + ')');
    // The latch: hidden with Inspect, shown once a drawing tool (Region select) is armed.
    check((await mc.first().locator('[data-mc-btn="pan"]').count()) === 0, tag + ': no pan latch while Inspect is armed');
    if (phone) await cell('dom');
    await p.getByRole('button', { name: 'Region (R)' }).first().click(); await p.waitForTimeout(500);
    const pan = mc.first().locator('[data-mc-btn="pan"]');
    check((await pan.count()) === 1, tag + ': the pan latch appears with Region select armed');
    if (phone) {
      // With the domain sheet still open the row must clear it: above it in portrait, beside it in landscape.
      const shBox = await box(sheet); const rowS = await box(mc);
      if (shBox) check(!overlap(rowS, shBox), tag + ': clear of the open ' + (form === 'phone-portrait' ? 'bottom' : 'side') + ' sheet');
      else console.log('note ' + tag + ': the sheet closed when the tool armed');
      if (shBox) { await cell('dom'); check((await sheet.count()) === 0, tag + ': re-tapping the tab closes the sheet'); }
    }
    const row2 = await box(mc);
    check(!!row2, tag + ': the row stays drawn with a tool armed');
    if (row2) {
      const strip = await box(p.locator('[aria-label*="tap for options, hold for"]'));
      check(!overlap(row2, strip), tag + ': clear of the tool strip');
      const fit2 = await box(mc.first().locator('[data-mc-btn="fit"]')); const fit1x = hand === 'left' ? row.x : row.x + row.width;
      check(fit2 && Math.abs((hand === 'left' ? fit2.x : fit2.x + fit2.width) - fit1x) <= 2, tag + ': the latch appears at the inner end; recentre does not move');
    }
    if (form === 'phone-portrait') await p.screenshot({ path: OUT + 'ds13-' + tag + '-latch.png' });
    // With the latch on, a one-finger drag pans: no region box is drawn. With it off, the same drag draws one.
    const dragAcross = async () => { const mx = mapBox.x + mapBox.width * 0.35, my = mapBox.y + mapBox.height * 0.35; await p.mouse.move(mx, my); await p.mouse.down(); for (let i = 1; i <= 8; i++) await p.mouse.move(mx + i * 12, my + i * 9); await p.mouse.up(); await p.waitForTimeout(400); };
    await pan.first().click(); await p.waitForTimeout(300);
    check((await pan.first().getAttribute('aria-pressed')) === 'true', tag + ': the latch reads pressed');
    // The latch announces itself with a toast; no toast may sit over the row it came from.
    const toasts = await p.locator('[aria-live="polite"]').evaluateAll((els) => els.flatMap((el) => [...el.querySelectorAll('div')].filter((d) => d.children.length === 0 || d.querySelector('.sc-interp')).filter((d) => d.textContent.trim()).map((d) => { const r = d.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; })));
    const rowT = await box(mc);
    check(toasts.length > 0 && !toasts.some((t) => overlap(t, rowT)), tag + ': its toast (' + toasts.length + ') does not cover the row');
    await dragAcross();
    check((await p.getByText(/box set/).count()) === 0, tag + ': with the latch on, a drag pans and draws no box');
    await pan.first().click(); await p.waitForTimeout(300);
    await dragAcross();
    check((await p.getByText(/box set/).count()) > 0, tag + ': with the latch off, the same drag draws a box');
    await p.screenshot({ path: OUT + 'ds13-' + tag + '.png' });
    // 3D left the row (§2.5): the phone reaches it in MAP ▸ View, the tablet in Window ▸ 3D view. Each must act.
    if (phone) {
      await cell('map'); await p.getByRole('radio', { name: '3D', exact: true }).first().click(); await p.waitForTimeout(400);
      check((await p.getByText('Height exaggeration (3D)').filter({ visible: true }).count()) > 0, tag + ': MAP ▸ View ▸ 3D turns 3D on (its height row appears)');
      await p.getByRole('radio', { name: '2D', exact: true }).first().click(); await p.waitForTimeout(300); await cell('map');
    } else {
      const win = p.getByRole('button', { name: 'Window', exact: true }).first();
      await win.click(); await p.waitForTimeout(300); await p.getByText('3D view', { exact: true }).first().click(); await p.waitForTimeout(400);
      await closeMenu('WINDOW');
      const v3 = await box(p.locator('[data-v3="1"]')); const rowNow = await box(mc);
      check(!!v3, tag + ': Window ▸ 3D view turns 3D on (its height card appears)');
      check(!overlap(v3, rowNow), tag + ': the 3D height card is clear of the control row');
    }
    await p.close();
  }
  // ---- Ruling EB item 2 on the desktop: the pointer column's + steps like the touch rocker (it stepped 1.6 before) ----
  if (!process.env.DS13_ONLY || process.env.DS13_ONLY === 'desktop') {
    const d = await b.newPage({ viewport: { width: 1600, height: 1000 } });
    d.on('pageerror', (e) => errs.push('desktop PAGEERROR ' + e.message));
    await d.goto(BASE + 'Desktop.dc.html'); await d.waitForTimeout(3000);
    const a0 = await viewAt(d); await d.getByRole('button', { name: 'zoom in', exact: true }).first().click(); await d.waitForTimeout(700); const a1 = await viewAt(d);
    check(!!(a0 && a1) && near(a1.s / a0.s, 1.35, 0.005), 'desktop: the pointer column\'s + zooms by exactly 1.35, as the touch rocker does (×' + (a0 && a1 ? (a1.s / a0.s).toFixed(4) : '?') + ')');
    await d.close();
  }
  // ---- Ruling EB item 3 on fresh pages: the double-tap and thumb zoom, and what they must not break ----
  for (const [form, w, h] of [['phone-portrait', 412, 892], ['phone-landscape', 892, 412], ['tablet', 1194, 834]].filter(([f]) => !process.env.DS13_ONLY || process.env.DS13_ONLY === 'gestures-' + f)) {
    const tag = 'thumb zoom, ' + form;
    const p = await b.newPage({ viewport: { width: w, height: h } });
    p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message));
    p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); });
    await p.goto(BASE + 'Live.dc.html'); await p.waitForTimeout(3000);
    const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
    const phone = form !== 'tablet';
    const cell = async (id) => { const r = await p.locator('[data-navcell="' + id + '"]').boundingBox(); await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await p.waitForTimeout(450); };
    const m = await p.locator(MAP).first().boundingBox();
    const P = { x: Math.round(m.x + m.width * 0.5), y: Math.round(m.y + m.height * 0.32) };
    const ratio = (a, b2) => (a && b2 ? b2.s / a.s : NaN);
    const say = (a, b2) => '×' + ratio(a, b2).toFixed(4) + ', the ground under the finger moved ' + moved(a, b2).toFixed(3) + ' km';
    const va = await viewAt(p);
    check(!!va && /double-tap to zoom in, double-tap and hold then slide down to zoom in or up to zoom out/.test(va.aria), tag + ': with Inspect the map names the double-tap and thumb zoom');
    // A double-tap zooms in ×1.7 at the finger (§4's approved gesture, built with thumb zoom).
    let v0 = await viewAt(p, P); await touchSeq(p, dblSteps(P)); await p.waitForTimeout(600); let v1 = await viewAt(p, P);
    check(near(ratio(v0, v1), 1.7, 0.01) && moved(v0, v1) < 0.05, tag + ': a double-tap zooms in ×1.7 at the finger (' + say(v0, v1) + ')');
    // Double-tap, hold, slide: down zooms in, up zooms out, about the finger, doubling every 120 px of slide.
    v0 = await viewAt(p, P); await touchSeq(p, holdSlideSteps(P, 120)); await p.waitForTimeout(300); v1 = await viewAt(p, P);
    check(near(ratio(v0, v1), 2, 0.02) && moved(v0, v1) < 0.05, tag + ': double-tap, hold, slide 120 px DOWN zooms IN, about the finger (' + say(v0, v1) + ')');
    await p.screenshot({ path: OUT + 'ds13-thumbzoom-' + form + '.png' });
    v0 = await viewAt(p, P); await touchSeq(p, holdSlideSteps(P, -120)); await p.waitForTimeout(300); v1 = await viewAt(p, P);
    check(near(ratio(v0, v1), 0.5, 0.01) && moved(v0, v1) < 0.05, tag + ': double-tap, hold, slide 120 px UP zooms OUT, about the finger (' + say(v0, v1) + ')');
    // The held second press belongs to thumb zoom: held still for 700 ms it opens no card, and lifting it is a double-tap.
    v0 = await viewAt(p, P); await touchSeq(p, tapSteps(P).concat([{ wait: 90 }, { t: 'pointerdown', x: P.x, y: P.y }, { wait: 700 }, { t: 'pointerup', x: P.x, y: P.y }])); await p.waitForTimeout(600); v1 = await viewAt(p, P);
    check(near(ratio(v0, v1), 1.7, 0.01) && (await p.getByText(/^SAMPLE ·/).count()) === 0, tag + ': a double-tap whose second press is held 700 ms opens no card and zooms ×1.7 on lifting (' + say(v0, v1) + ')');
    // A tap, then a two-finger tap whose first finger lands on it inside the double-tap window: still an undo, no zoom.
    const undos = () => p.getByText(/^(Nothing to undo|Undid · )/).count();
    const u0 = await undos(); v0 = await viewAt(p, P);
    await touchSeq(p, tapSteps(P).concat([{ wait: 90 }, { t: 'pointerdown', id: 1, x: P.x, y: P.y }, { t: 'pointerdown', id: 2, x: P.x + 80, y: P.y }, { wait: 60 }, { t: 'pointerup', id: 1, x: P.x, y: P.y }, { t: 'pointerup', id: 2, x: P.x + 80, y: P.y }]));
    await p.waitForTimeout(400); v1 = await viewAt(p, P);
    check((await undos()) > u0 && near(ratio(v0, v1), 1, 1e-9), tag + ': a tap then a two-finger tap still undoes (Ruling DU Q5) and zooms nothing (' + say(v0, v1) + ')');
    // A pinch still zooms: the two fingers' spread doubles, so does the scale.
    v0 = await viewAt(p); const pin = [{ t: 'pointerdown', id: 1, x: P.x - 50, y: P.y }, { t: 'pointerdown', id: 2, x: P.x + 50, y: P.y }]; for (let i = 1; i <= 10; i++) pin.push({ t: 'pointermove', id: 1, x: P.x - 50 - i * 5, y: P.y }, { t: 'pointermove', id: 2, x: P.x + 50 + i * 5, y: P.y }); pin.push({ t: 'pointerup', id: 1, x: P.x - 100, y: P.y }, { t: 'pointerup', id: 2, x: P.x + 100, y: P.y });
    await touchSeq(p, pin); await p.waitForTimeout(300); v1 = await viewAt(p);
    check(near(ratio(v0, v1), 2, 0.02), tag + ': a pinch still zooms (×' + ratio(v0, v1).toFixed(4) + ' for twice the spread)');
    // A plain long-press, with no tap before it, still opens the card.
    await touchSeq(p, [{ t: 'pointerdown', x: P.x, y: P.y }, { wait: 700 }, { t: 'pointerup', x: P.x, y: P.y }]); await p.waitForTimeout(400);
    check((await p.getByText(/^SAMPLE ·/).count()) > 0, tag + ': a long-press still opens the card');
    await touchSeq(p, tapSteps(P)); await p.waitForTimeout(500);
    // A drawing tool armed (Region select): thumb zoom is off, labelled pending DS13-Q1, and neither gesture zooms.
    if (phone) await cell('dom');
    await p.getByRole('button', { name: 'Region (R)' }).first().click(); await p.waitForTimeout(500);
    if (phone && (await p.getByRole('dialog', { name: /^(MAP|WORLD|CIVIL|CARTO|STORY|MENU)\b/ }).filter({ visible: true }).count())) await cell('dom');
    const vr = await viewAt(p);
    check(!!vr && /pending DS13-Q1/.test(vr.aria), tag + ': with Region select armed the map says thumb zoom is off, pending DS13-Q1');
    v0 = await viewAt(p, P); await touchSeq(p, dblSteps(P)); await p.waitForTimeout(600); v1 = await viewAt(p, P);
    check(near(ratio(v0, v1), 1, 1e-9), tag + ': with Region select armed a double-tap does not zoom (' + say(v0, v1) + ')');
    v0 = await viewAt(p, P); await touchSeq(p, holdSlideSteps(P, 120)); await p.waitForTimeout(400); v1 = await viewAt(p, P);
    check(near(ratio(v0, v1), 1, 1e-9) && (await p.getByText(/box set/).count()) > 0, tag + ': with Region select armed a double-tap-hold-slide does not zoom; the slide draws a box (' + say(v0, v1) + ')');
    if (form === 'phone-portrait') await p.screenshot({ path: OUT + 'ds13-thumbzoom-off-region-' + form + '.png' });
    // The gesture reference says so in words (phone: MENU ▸ Preferences ▸ Touch & gestures…).
    if (form === 'phone-portrait') {
      // Toasts live 2.4 s (c_class.js _toast); let the Region toasts go so none covers the window in the capture.
      await p.waitForTimeout(2600);
      await cell('menu'); await p.getByText('Preferences', { exact: true }).first().click(); await p.waitForTimeout(350);
      await p.getByText('Touch & gestures…', { exact: true }).first().click(); await p.waitForTimeout(500);
      const row = p.getByText('double-tap and thumb zoom off · pending DS13-Q1', { exact: true }).filter({ visible: true });
      if (await row.count()) await row.first().scrollIntoViewIfNeeded();
      check((await row.count()) > 0 && (await p.getByText('Double-tap, hold, slide', { exact: true }).filter({ visible: true }).count()) > 0, tag + ': Touch & gestures lists thumb zoom and marks it pending DS13-Q1 with a drawing tool');
      await p.waitForTimeout(250); await p.screenshot({ path: OUT + 'ds13-touch-gestures.png' });
    }
    await p.close();
  }
  console.log(errs.length ? errs.join('\n') : 'no page errors');
  await b.close();
  process.exit(bad || errs.length ? 1 : 0);
})();
