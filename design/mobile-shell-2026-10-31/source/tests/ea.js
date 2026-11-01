// Ruling EA's three small designs, checked in the mockup on the forms each one reaches:
//   EA 10  the planner's stale-leg notice (desktop planner window, phone STORY ▸ Journeys): a saved journey whose
//          per-leg override no longer fits is opened; the notice names the dropped change, offers Show stage and
//          Dismiss, and Dismiss removes it.
//   EA 14  the 3D view's live height-exaggeration slider: 1× to 4× in 0.25 steps (Ruling EB item 6), 1× by default
//          (Ruling CT); on the phone a row in MAP ▸ View, on the tablet and desktop a HEIGHT row in the 3D view's own
//          card (top-left, as `view3d.gd` ships it), whose Back to 2D ends 3D and brings the 2D furniture back. The
//          value is the preference Preferences ▸ 3D viewport defaults (Ruling EB item 5): each slider says so, the
//          value survives a reload, and with storage blocked the page still works and starts at 1×.
//   EA 15  colour paint's one-line notice when a regenerate changes the map width (CARTO ▸ Style ▸ Colours), with
//          Ruling EB item 7's Scale with the map: every mark ×1.5 about the top-left edge, the notice gone, one undo
//          step that restores the marks and the notice, and a redo that scales again.
// Writes ea-*.png (mobile captures beside the mobile shell's, desktop ones beside the desktop shell's). Exits 1 on a
// failed check or a page error.
const { chromium } = require('playwright');
// SHOT_DIR redirects the captures; set it for a mutation run, or the mutant's frames overwrite the evidence.
const MOB = process.env.SHOT_DIR || '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/mobile-shell-2026-10-31/';
const DESK = process.env.SHOT_DIR || '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/desktop-shell-2026-10-31/';
// BASE serves another build (an older canvas or a mutant), to show a check can fail.
const BASE = process.env.BASE || 'http://127.0.0.1:8770/';
let bad = 0;
const check = (ok, what) => { console.log((ok ? 'ok   ' : 'FAIL ') + what); if (!ok) bad += 1; };
const MAP = '[aria-label^="map: drag to pan"]';
// The mockup's colour-paint marks ([[x, y]...] points and radius), read through React's fiber on the map host (the DC
// runtime keeps the logic object as the wrapper's stateNode.logic). It only reads; a missing component gives null.
const paintMarks = (p) => p.locator(MAP).first().evaluate((el) => {
  const key = Object.getOwnPropertyNames(el).find((k) => k.startsWith('__reactFiber'));
  let f = key ? el[key] : null; while (f && !(f.stateNode && f.stateNode.logic && f.stateNode.logic._s2w)) f = f.return;
  if (!f || !Array.isArray(f.stateNode.logic.state.cpMarks)) return null;
  return f.stateNode.logic.state.cpMarks.map((m) => ({ pts: m.pts.map((q) => [q.x, q.y]), r: m.r }));
});
// Whether every mark of b is the same mark of a scaled by k about the top-left edge. Never true of an empty set.
const scaledBy = (a, b, k) => !!(a && b) && a.length > 0 && a.length === b.length && a.every((m, i) => Math.abs(b[i].r - m.r * k) < 1e-9 && m.pts.length === b[i].pts.length && m.pts.every((q, j) => Math.abs(b[i].pts[j][0] - q[0] * k) < 1e-9 && Math.abs(b[i].pts[j][1] - q[1] * k) < 1e-9));
// A three-finger tap on the map (redo on touch, Ruling DU Q5), as pointer events the mockup's gesture code reads.
const threeTap = (p) => p.locator(MAP).first().evaluate((el) => {
  const r = el.getBoundingClientRect(), y = r.top + r.height * 0.3;
  const ev = (type, id, x) => el.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, pointerId: id, pointerType: 'touch', isPrimary: id === 1, clientX: x, clientY: y, button: 0, buttons: type === 'pointerup' ? 0 : 1 }));
  [1, 2, 3].forEach((id) => ev('pointerdown', id, r.left + r.width * 0.3 + id * 40)); [1, 2, 3].forEach((id) => ev('pointerup', id, r.left + r.width * 0.3 + id * 40));
});
const overlap = (a, b) => !!(a && b && a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height);
const box = async (loc) => ((await loc.count()) && (await loc.first().isVisible()) ? loc.first().boundingBox() : null);
const STALE = /1 leg change no longer fits this world and was dropped: stage 3, .+ → .+ \(pace override “forced march”\)\. The legs were rebuilt after the world changed; nothing was applied to a different leg\./;
const WIDTH = /Map width 800 km → 1,200 km: 17 paint marks kept their real size and position; 3 now on water are hidden, not deleted\./;
(async () => {
  const b = await chromium.launch(); const errs = [];
  // init: an optional script run in the page before the shell boots (the storage-blocked case below).
  const open = async (file, w, h, tag, init) => {
    const p = await b.newPage({ viewport: { width: w, height: h } });
    if (init) await p.addInitScript(init);
    p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message));
    p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); });
    await p.goto(BASE + file); await p.waitForTimeout(3000);
    const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
    return p;
  };
  const cellOf = (p) => async (id) => { const r = await p.locator('[data-navcell="' + id + '"]').boundingBox(); await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await p.waitForTimeout(450); };
  // A two-finger turn on the map, as pointer events with two ids (the mockup's own gesture code reads them).
  const turn = async (p, deg) => {
    const m = await p.locator('[aria-label^="map: drag to pan"]').first().boundingBox();
    const cx = m.x + m.width / 2, cy = m.y + m.height / 2, r = 80;
    await p.locator('[aria-label^="map: drag to pan"]').first().evaluate((el, a) => {
      const ev = (type, id, x, y) => el.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, pointerId: id, pointerType: 'touch', isPrimary: id === 1, clientX: x, clientY: y, buttons: type === 'pointerup' ? 0 : 1 }));
      const at = (k, t) => [a.cx + Math.cos(t + k * Math.PI) * a.r, a.cy + Math.sin(t + k * Math.PI) * a.r];
      ev('pointerdown', 1, ...at(0, 0)); ev('pointerdown', 2, ...at(1, 0));
      for (let i = 1; i <= 10; i++) { const t = (a.deg * Math.PI / 180) * i / 10; ev('pointermove', 1, ...at(0, t)); ev('pointermove', 2, ...at(1, t)); }
      const t = a.deg * Math.PI / 180; ev('pointerup', 1, ...at(0, t)); ev('pointerup', 2, ...at(1, t));
    }, { cx, cy, r, deg });
    await p.waitForTimeout(400);
  };

  // ---- EA 10, desktop: Story ▸ Saved journeys… ▸ the journey saved before a regenerate opens the planner window.
  {
    const p = await open('Desktop.dc.html', 1600, 1000, 'ea10-desktop');
    await p.getByRole('button', { name: 'Story', exact: true }).click(); await p.waitForTimeout(200);
    await p.getByRole('menu', { name: 'STORY' }).getByText('Saved journeys…', { exact: true }).click(); await p.waitForTimeout(400);
    await p.getByText(/· before the regenerate$/).first().click(); await p.waitForTimeout(600);
    const pw = p.getByRole('dialog', { name: 'JOURNEY PLANNER' });
    check((await pw.count()) === 1, 'EA 10 desktop: the planner window opens');
    const txt = (await pw.innerText()).replace(/\s+/g, ' ');
    check(STALE.test(txt), 'EA 10 desktop: the notice names the dropped change and says nothing moved to another leg');
    check((await pw.getByRole('button', { name: 'Show stage 3' }).count()) === 1 && (await pw.getByRole('button', { name: 'Dismiss' }).count()) === 1, 'EA 10 desktop: Show stage 3 and Dismiss are offered');
    const nb = await box(pw.getByText(/leg change no longer fits/)); const pb = await box(pw);
    check(nb && pb && nb.y - pb.y < 200, 'EA 10 desktop: the notice is at the top of the planner, ' + (nb && pb ? Math.round(nb.y - pb.y) : '?') + ' px under its top edge');
    await p.screenshot({ path: DESK + 'ea-10-desktop-planner-stale.png' });
    await pw.getByRole('button', { name: 'Dismiss' }).click(); await p.waitForTimeout(300);
    check(!/leg change no longer fits/.test(await pw.innerText()), 'EA 10 desktop: Dismiss removes the notice');
    await p.close();
  }
  // ---- EA 10, phone: MENU ▸ Story ▸ Saved journeys… ▸ the same journey opens STORY ▸ Journeys.
  {
    const p = await open('Live.dc.html', 412, 892, 'ea10-phone'); const cell = cellOf(p);
    await cell('menu'); await p.getByText('Story', { exact: true }).first().click(); await p.waitForTimeout(350);
    await p.getByText('Saved journeys…', { exact: true }).first().click(); await p.waitForTimeout(450);
    await p.getByText(/· before the regenerate$/).first().click(); await p.waitForTimeout(600);
    const sh = p.getByRole('dialog', { name: /^STORY/ }).filter({ visible: true });
    check((await sh.count()) === 1, 'EA 10 phone: STORY ▸ Journeys opens');
    const txt = (await sh.innerText()).replace(/\s+/g, ' ');
    check(STALE.test(txt), 'EA 10 phone: the same notice, word for word');
    const show = sh.getByRole('button', { name: 'Show stage 3' }); const dis = sh.getByRole('button', { name: 'Dismiss' });
    const sb = await box(show), db = await box(dis);
    check(sb && db && sb.height >= 44 && db.height >= 44, 'EA 10 phone: both buttons are at least 44 tall (' + (sb ? Math.round(sb.height) : '?') + ', ' + (db ? Math.round(db.height) : '?') + ')');
    await p.screenshot({ path: MOB + 'ea-10-phone-planner-stale.png' });
    await show.click(); await p.waitForTimeout(400);
    check((await p.getByRole('dialog', { name: 'STORY · STAGE 3' }).filter({ visible: true }).count()) === 1, 'EA 10 phone: Show stage 3 opens stage 3');
    await p.close();
  }

  // ---- EA 14, phone: MAP ▸ View ▸ 3D, then the height row.
  {
    const p = await open('Live.dc.html', 412, 892, 'ea14-phone'); const cell = cellOf(p);
    await cell('map'); check((await p.getByText('Height exaggeration (3D)').filter({ visible: true }).count()) === 0, 'EA 14 phone: no height row while the view is 2D');
    await p.getByRole('radio', { name: '3D', exact: true }).first().click(); await p.waitForTimeout(400);
    const sl = p.getByRole('slider', { name: /Height exaggeration \(3D\)/ }).filter({ visible: true });
    check((await sl.count()) === 1, 'EA 14 phone: the 3D height row appears in MAP ▸ View');
    if (await sl.count()) {
      const a = await sl.first().evaluate((el) => [el.min, el.max, el.step, el.value].join(' '));
      check(a === '1 4 0.25 1', 'EA 14 phone: range 1 to 4 (Ruling EB item 6), step 0.25, starts at 1 (' + a + ')');
      await sl.first().fill('2'); await p.waitForTimeout(250);
      const v = await sl.first().evaluate((el) => el.value); check(v === '2', 'EA 14 phone: the row takes a new value (' + v + ')');
      const sb = await box(sl); check(sb && sb.height >= 24, 'EA 14 phone: the slider row is a touch target (' + (sb ? Math.round(sb.height) : '?') + ' px tall)');
      check((await p.getByText(/^Remembered in Preferences ▸ 3D viewport defaults for the next session · never saved with the world$/).filter({ visible: true }).count()) === 1, 'EA 14 phone: the row says where the value is kept (Ruling EB item 5)');
    }
    await p.screenshot({ path: MOB + 'ea-14-phone-3d-height.png' });
    // Ruling EB item 5: the last value set is the next session's default. Set 2.5, reload, and the row opens at 2.5.
    if (await sl.count()) {
      await sl.first().fill('2.5'); await p.waitForTimeout(250);
      await p.reload(); await p.waitForTimeout(3000);
      const tip2 = p.getByLabel('dismiss tip'); if (await tip2.count()) await tip2.first().click();
      await cell('map'); await p.getByRole('radio', { name: '3D', exact: true }).first().click(); await p.waitForTimeout(400);
      const v2 = await p.getByRole('slider', { name: /Height exaggeration \(3D\)/ }).filter({ visible: true }).first().evaluate((el) => el.value);
      check(v2 === '2.5', 'EA 14 phone: after a reload the row opens at the last value set (' + v2 + ')');
    }
    await p.close();
  }
  // ---- EA 14, phone, with this browser's storage blocked: the page works, and the row starts at the default, 1×.
  {
    const block = () => { const no = () => { throw new Error('storage blocked by the test'); }; Storage.prototype.getItem = no; Storage.prototype.setItem = no; };
    const p = await open('Live.dc.html', 412, 892, 'ea14-phone-nostorage', block); const cell = cellOf(p);
    // A throw in the shell's constructor is caught by the DC runtime and fires no pageerror: the page is simply empty.
    // So the first check is that the shell drew its bottom bar at all.
    const drew = (await p.locator('[data-navcell="map"]').count()) === 1;
    check(drew, 'EA 14 phone, storage blocked: the shell still draws');
    if (drew) {
      await cell('map'); await p.getByRole('radio', { name: '3D', exact: true }).first().click(); await p.waitForTimeout(400);
      const sl = p.getByRole('slider', { name: /Height exaggeration \(3D\)/ }).filter({ visible: true });
      const v0 = (await sl.count()) ? await sl.first().evaluate((el) => el.value) : '?';
      if (await sl.count()) { await sl.first().fill('3'); await p.waitForTimeout(250); }
      const v1 = (await sl.count()) ? await sl.first().evaluate((el) => el.value) : '?';
      check(v0 === '1' && v1 === '3', 'EA 14 phone, storage blocked: the row starts at 1 and still takes a value (' + v0 + ' → ' + v1 + ')');
    }
    await p.close();
  }
  // ---- EA 14, tablet and desktop: Window ▸ 3D view, then the HEIGHT row in the 3D view's own card.
  for (const [file, w, h, tag, dir] of [['Live.dc.html', 1194, 834, 'tablet', MOB], ['Desktop.dc.html', 1600, 1000, 'desktop', DESK]]) {
    const p = await open(file, w, h, 'ea14-' + tag);
    const card = p.locator('[data-v3="1"]');
    check(!(await box(card)), 'EA 14 ' + tag + ': no 3D card while the view is 2D');
    const win = p.getByRole('button', { name: 'Window', exact: true }).first();
    await win.click(); await p.waitForTimeout(300);
    const menu = p.getByRole('menu', { name: 'WINDOW' }).filter({ visible: true });
    await menu.getByText('3D view', { exact: true }).first().click(); await p.waitForTimeout(400);
    const mb = await box(menu); if (mb) { await p.mouse.click(mb.x > 100 ? 40 : w - 40, Math.round(h / 2)); await p.waitForTimeout(300); }
    if (await box(p.getByRole('menu', { name: 'WINDOW' }))) { await p.keyboard.press('Escape'); await p.waitForTimeout(250); }
    const cb = await box(card);
    check(!!cb, 'EA 14 ' + tag + ': Window ▸ 3D view shows the 3D card');
    const mapB = await p.locator('[aria-label^="map: drag to pan"]').first().boundingBox();
    check(cb && mapB && Math.abs(cb.x - mapB.x - 12) <= 2 && Math.abs(cb.y - mapB.y - 12) <= 2 && cb.width <= 460, 'EA 14 ' + tag + ': the card sits at the map\'s top-left, 12 in, at most 460 wide (' + (cb ? Math.round(cb.x - mapB.x) + ', ' + Math.round(cb.y - mapB.y) + ', ' + Math.round(cb.width) : '?') + ')');
    for (const name of ['BACK TO 2D', 'RE-TEST 3D', 'FORCE TIER…']) check((await card.getByRole('button', { name, exact: true }).count()) === 1, 'EA 14 ' + tag + ': the card keeps 3D-1\'s ' + name);
    check(!(await box(p.getByRole('button', { name: 'layers', exact: true }))), 'EA 14 ' + tag + ': the 2D layers button is covered while 3D is on');
    const sl = card.getByRole('slider');
    const a = await sl.evaluate((el) => [el.min, el.max, el.step, el.value].join(' '));
    check(a === '1 4 0.25 1', 'EA 14 ' + tag + ': range 1 to 4 (Ruling EB item 6), step 0.25, starts at 1 (' + a + ')');
    check((await sl.getAttribute('aria-label')) === 'height exaggeration, 1 to 4 times', 'EA 14 ' + tag + ': the slider is named 1 to 4 times');
    const where = card.locator('[data-v3-where]');
    check((await where.count()) === 1 && ((await where.innerText()) || '').trim() === 'Remembered in Preferences ▸ 3D viewport defaults for the next session · never saved with the world', 'EA 14 ' + tag + ': the card says where the value is kept (Ruling EB item 5)');
    const order = await card.evaluate((el) => { const t = el.innerText; return [t.indexOf('Terrain is not drawn'), t.indexOf('HEIGHT'), t.indexOf('BACK TO 2D')].join(' '); });
    const [oi, oh, ob] = order.split(' ').map(Number);
    check(oi >= 0 && oi < oh && oh < ob, 'EA 14 ' + tag + ': the HEIGHT row sits between the info line and the actions');
    await sl.fill('2.5'); await p.waitForTimeout(250);
    check((await card.innerText()).includes('×2.50'), 'EA 14 ' + tag + ': the card reads ×2.50 after the drag');
    const sb = await box(sl); check(sb && sb.height >= (tag === 'tablet' ? 44 : 28), 'EA 14 ' + tag + ': the slider is ' + (sb ? Math.round(sb.height) : '?') + ' px tall');
    await p.screenshot({ path: dir + 'ea-14-' + tag + '-3d-card.png' });
    await turn(p, 35);
    const comp = await box(p.getByRole('button', { name: 'rotate north up' })); const cb2 = await box(card);
    check(!overlap(comp, cb2), 'EA 14 ' + tag + ': turned, the compass and the card do not meet');
    await card.getByRole('button', { name: 'BACK TO 2D', exact: true }).click(); await p.waitForTimeout(350);
    check(!(await box(card)) && !!(await box(p.getByRole('button', { name: 'layers', exact: true }))), 'EA 14 ' + tag + ': Back to 2D ends 3D and brings the layers button back');
    // Ruling EB item 5: the 2.5 set above is the next session's default. Reload, open 3D again, and the card reads it.
    await p.reload(); await p.waitForTimeout(3000);
    const tip2 = p.getByLabel('dismiss tip'); if (await tip2.count()) await tip2.first().click();
    await p.getByRole('button', { name: 'Window', exact: true }).first().click(); await p.waitForTimeout(300);
    const menu2 = p.getByRole('menu', { name: 'WINDOW' }).filter({ visible: true });
    await menu2.getByText('3D view', { exact: true }).first().click(); await p.waitForTimeout(400);
    const mb2 = await box(menu2); if (mb2) { await p.mouse.click(mb2.x > 100 ? 40 : w - 40, Math.round(h / 2)); await p.waitForTimeout(300); }
    if (await box(p.getByRole('menu', { name: 'WINDOW' }))) { await p.keyboard.press('Escape'); await p.waitForTimeout(250); }
    const after = (await box(card)) ? await card.getByRole('slider').evaluate((el) => el.value) : '?';
    check(after === '2.5' && (await card.innerText()).includes('×2.50'), 'EA 14 ' + tag + ': after a reload the card opens at the last value set (' + after + ')');
    await p.close();
  }

  // ---- EA 15: CARTO ▸ Style ▸ Colours, after a regenerate changed the map width.
  {
    const p = await open('Live.dc.html', 412, 892, 'ea15-phone'); const cell = cellOf(p);
    await cell('dom'); await p.getByLabel('switch domain').click(); await p.waitForTimeout(300); await p.locator('[data-fan="2"]').click(); await p.waitForTimeout(400);
    const sh = p.getByRole('dialog', { name: /^CARTO/ }).filter({ visible: true });
    check((await sh.count()) === 1, 'EA 15 phone: the CARTO sheet is open');
    await sh.getByText('Colours', { exact: true }).first().click(); await p.waitForTimeout(350);
    check(!WIDTH.test((await sh.innerText()).replace(/\s+/g, ' ')), 'EA 15 phone: no notice before the width changes');
    await sh.getByText('Simulate a regenerate at another width', { exact: true }).first().click(); await p.waitForTimeout(400);
    const txt = (await sh.innerText()).replace(/\s+/g, ' ');
    check(WIDTH.test(txt), 'EA 15 phone: the notice gives the old and new width, the mark count and the hidden ones');
    const nb = await box(sh.getByText(/paint marks kept their real size/)); const rel = await box(sh.getByText('COLOUR RELIEF', { exact: true }));
    check(nb && rel && nb.y < rel.y, 'EA 15 phone: the notice heads the Colours section');
    // The mock's Simulate row sits below the notice it inserts, so bring the notice back into view for the capture.
    await sh.getByText(/paint marks kept their real size/).first().scrollIntoViewIfNeeded(); await p.waitForTimeout(250);
    await p.screenshot({ path: MOB + 'ea-15-phone-paint-width.png' });
    // Ruling EB item 7: Scale with the map, beside Show marks and Dismiss.
    const names = ['Show marks', 'Scale with the map', 'Dismiss'];
    const have = []; for (const n of names) have.push(await sh.getByRole('button', { name: n, exact: true }).count());
    check(have.every((x) => x === 1), 'EA 15 phone: the notice offers Show marks, Scale with the map and Dismiss (' + have.join(' ') + ')');
    const m0 = await paintMarks(p);
    check(!!m0 && m0.length === 17, 'EA 15 phone: the map holds the 17 marks the notice counts (' + (m0 ? m0.length : '?') + ')');
    if (have[1]) { await sh.getByRole('button', { name: 'Scale with the map', exact: true }).click(); await p.waitForTimeout(400); }
    const m1 = await paintMarks(p);
    check(scaledBy(m0, m1, 1.5), 'EA 15 phone: Scale with the map moves every mark ×1.5 (1 200 ÷ 800) about the top-left edge, points and radius');
    check(!WIDTH.test((await sh.innerText()).replace(/\s+/g, ' ')), 'EA 15 phone: the notice goes once the marks are scaled');
    await p.screenshot({ path: MOB + 'ea-15-phone-paint-scaled.png' });
    const undo = p.getByRole('button', { name: /^undo Scale paint with the map; hold for history$/ });
    check((await undo.count()) === 1, 'EA 15 phone: the undo step is named "Scale paint with the map"');
    if (await undo.count()) { await undo.first().click(); await p.waitForTimeout(400); }
    const m2 = await paintMarks(p);
    check(scaledBy(m0, m2, 1) && WIDTH.test((await sh.innerText()).replace(/\s+/g, ' ')), 'EA 15 phone: one undo puts every mark back at its kilometres and the notice with it');
    await threeTap(p); await p.waitForTimeout(400);
    const m3 = await paintMarks(p);
    check(scaledBy(m0, m3, 1.5) && !WIDTH.test((await sh.innerText()).replace(/\s+/g, ' ')), 'EA 15 phone: a three-finger tap redoes it: scaled again, notice gone');
    const undo2 = p.getByRole('button', { name: /^undo Scale paint with the map; hold for history$/ });
    if (await undo2.count()) { await undo2.first().click(); await p.waitForTimeout(400); }
    await sh.getByRole('button', { name: 'Dismiss' }).click(); await p.waitForTimeout(300);
    check(!WIDTH.test((await sh.innerText()).replace(/\s+/g, ' ')), 'EA 15 phone: Dismiss removes it');
    check(scaledBy(m0, await paintMarks(p), 1), 'EA 15 phone: Dismiss moves no mark');
    await p.close();
  }
  {
    const p = await open('Desktop.dc.html', 1600, 1000, 'ea15-desktop');
    await p.getByRole('tab', { name: 'CARTO domain' }).click(); await p.waitForTimeout(400);
    await p.getByText('Colours', { exact: true }).first().click(); await p.waitForTimeout(350);
    await p.getByText('Simulate a regenerate at another width', { exact: true }).first().click(); await p.waitForTimeout(400);
    const body = (await p.locator('body').innerText()).replace(/\s+/g, ' ');
    check(WIDTH.test(body), 'EA 15 desktop: the same notice in the CARTO dock');
    await p.screenshot({ path: DESK + 'ea-15-desktop-paint-width.png' });
    // The widths are distances, so they follow Preferences ▸ Units like every other distance in the shell.
    await p.getByRole('button', { name: 'Preferences', exact: true }).click(); await p.waitForTimeout(250);
    await p.getByRole('menu', { name: 'PREFERENCES' }).getByText('Units', { exact: true }).first().click(); await p.waitForTimeout(250);
    await p.getByText('Miles', { exact: true }).first().click(); await p.waitForTimeout(250); await p.keyboard.press('Escape'); await p.waitForTimeout(250);
    const body2 = (await p.locator('body').innerText()).replace(/\s+/g, ' ');
    check(/Map width 497 mi → 746 mi: 17 paint marks kept their real size and position;/.test(body2), 'EA 15 desktop: the widths follow Preferences ▸ Units (miles)');
    // Ruling EB item 7 in the CARTO dock: Scale with the map, then the menu bar's Undo and Redo, one step each way.
    const m0 = await paintMarks(p);
    const scale = p.getByRole('button', { name: 'Scale with the map', exact: true });
    check((await scale.count()) === 1, 'EA 15 desktop: the dock\'s notice offers Scale with the map');
    if (await scale.count()) { await scale.first().click(); await p.waitForTimeout(400); }
    const m1 = await paintMarks(p);
    check(scaledBy(m0, m1, 1.5) && !/paint marks kept their real size/.test((await p.locator('body').innerText()).replace(/\s+/g, ' ')), 'EA 15 desktop: it scales every mark ×1.5 and the notice goes');
    const undoB = p.getByRole('button', { name: 'undo Scale paint with the map', exact: true });
    if (await undoB.count()) { await undoB.first().click(); await p.waitForTimeout(400); }
    const m2 = await paintMarks(p);
    check(scaledBy(m0, m2, 1) && /paint marks kept their real size/.test((await p.locator('body').innerText()).replace(/\s+/g, ' ')), 'EA 15 desktop: Undo (one step, "Scale paint with the map") restores every mark and the notice');
    const redoB = p.getByRole('button', { name: 'redo Scale paint with the map', exact: true });
    if (await redoB.count()) { await redoB.first().click(); await p.waitForTimeout(400); }
    check(scaledBy(m0, await paintMarks(p), 1.5), 'EA 15 desktop: Redo scales them again');
    await p.screenshot({ path: DESK + 'ea-15-desktop-paint-scaled.png' });
    await p.close();
  }
  console.log(errs.length ? errs.join('\n') : 'no page errors');
  await b.close();
  process.exit(bad || errs.length ? 1 : 0);
})();
