// Phase B: the full surface tour at 1600x900 in one session (theme from argv), then the main views at the
// other three sizes. Real clicks wherever a user would click; evaluate() only READS state (positions,
// sizes) except where noted. Every step is guarded so one failure does not lose the rest.
const L = require('./lib.js');
const { log, shot, elShot, clipShot, save } = L;
const THEME = process.argv[2] || 'dark';
const SEED = +(process.argv[3] || 12345);
const RES = process.argv[4] || '';
const P = (process.argv[5] || 'b') + '_';
const SIZES = (process.argv[6] || '1366x768,1920x1080,2560x1440').split(',').filter(Boolean).map(s => s.split('x').map(Number));
const TOUR = process.argv[7] !== 'notour';

async function step(name, fn) { try { await fn(); } catch (e) { log('STEP FAILED', name, e.message.split('\n')[0]); } }
async function esc(p, n = 2) { for (let i = 0; i < n; i++) { await p.keyboard.press('Escape'); await p.waitForTimeout(120); } }
async function fit(p) { await p.click('#zoomReset'); await p.waitForTimeout(900); }
async function biggestVisible(p) {
  return p.evaluate(() => {
    const w = document.querySelector('.canvas-wrap').getBoundingClientRect();
    const qs = [...state.places].filter(x => x.pop).sort((a, b) => b.pop - a.pop);
    for (const q of qs) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) return { x: s[0], y: s[1], name: q.name, pop: q.pop }; }
    return null;
  });
}
async function dockSeries(p, sel, name, maxShots = 8) {
  const h = await p.evaluate(s => { const d = document.querySelector(s); if (!d) return 0; d.scrollTop = 0; return d.scrollHeight - d.clientHeight; }, sel);
  const ch = await p.evaluate(s => document.querySelector(s).clientHeight, sel);
  let i = 0;
  for (let y = 0; i < maxShots; y += Math.max(200, ch - 80), i++) {
    await p.evaluate(([s, y]) => { document.querySelector(s).scrollTop = y; }, [sel, Math.min(y, h)]);
    await p.waitForTimeout(150);
    await elShot(p, sel, `${name}_s${i}`);
    if (y >= h) break;
  }
  await p.evaluate(s => { document.querySelector(s).scrollTop = 0; }, sel);
}

(async () => {
  const b = await L.chromium.launch();
  const p = await L.boot(b, L.V272, { width: 1600, height: 900 }, { light: THEME === 'light' });
  await L.installAudit(p);
  if (THEME === 'dark' && TOUR) {
    await step('startcard scrolled', async () => {
      await p.click('#obGenerate'); await p.waitForTimeout(300);
      // scroll the New World card half-way, as a user with a short window would, to see the sticky actions
      await p.mouse.move(800, 450); await p.mouse.wheel(0, 260); await p.waitForTimeout(300);
      await shot(p, P + '00_newworld_scrolled');
      await p.mouse.wheel(0, -2000); await p.waitForTimeout(200);
      await p.click('#suGenBack'); await p.waitForTimeout(300);
    });
  }
  await L.genByClicks(p, SEED, RES, null);
  await L.installAudit(p);
  await step('populate', async () => {
    await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(300);
    await p.click('[data-civsub="generation"]'); await p.waitForTimeout(300);
    await p.click('#civAutoPopulateBtn', { noWaitAfter: true, timeout: 120000 }); await p.waitForTimeout(500); await L.idle(p, 800);
    if (await p.evaluate(() => document.getElementById('tlStripMore').hidden)) await p.click('#tlStripToggle');
    for (const y of [380, 412]) { await p.fill('#civTlYear', String(y)); await p.click('#civTlAddYearBtn'); await p.waitForTimeout(400); await L.idle(p, 200); }
    await p.click('#tlStripToggle'); await p.waitForTimeout(300);   // collapse again: the default look
  });
  await step('fit', async () => { await fit(p); });
  await shot(p, P + '10_civil_fit_1600');
  let at = null;
  await step('select', async () => {
    at = await biggestVisible(p); log('select', JSON.stringify(at));
    await p.mouse.click(at.x, at.y); await p.waitForTimeout(700);
    await shot(p, P + '11_place_selected_1600');
  });
  if (TOUR) {
    await step('popup series', async () => { await dockSeries(p, '#placeEditPopup', P + '12_placepopup', 6); });
    await step('context panel', async () => { await elShot(p, '.dock-right', P + '13_context_place'); });
    save(P + 'contrast_place_1600', await p.evaluate(() => __aud.contrast()));
    save(P + 'truncs_place_1600', await p.evaluate(() => __aud.truncs()));
    await step('city viewer', async () => {
      const btn = await p.$('#placeEditPopup button:has-text("city view")');
      if (btn) { await btn.click(); await p.waitForTimeout(1500); await shot(p, P + '14_cityviewer_1600'); save(P + 'contrast_cityviewer', await p.evaluate(() => __aud.contrast('#cityViewerModal'))); await p.click('#cityViewerModal .cv-close'); await p.waitForTimeout(300); }
      else log('no city view button');
    });
    // close the popup with its own ×
    await step('close popup', async () => { await p.click('#placeEditPopup .si-close'); await p.waitForTimeout(300); });

    // menus, each opened by a real click
    for (const [id, nm] of [['#fileMenuBtn', 'file'], ['#mbEditBtn', 'edit'], ['#mbAssetsBtn', 'assets'], ['#mbRecordsBtn', 'records'], ['#mbStoryBtn', 'story'], ['#mbPrefsBtn', 'prefs'], ['#mbWindowBtn', 'window'], ['#mbHelpBtn', 'help']]) {
      await step('menu ' + nm, async () => { await p.click(id); await p.waitForTimeout(250); await shot(p, P + '20_menu_' + nm); await esc(p, 1); });
    }
    await step('file export sub', async () => { await p.click('#fileMenuBtn'); await p.hover('#mbExportRow'); await p.waitForTimeout(350); await shot(p, P + '21_menu_file_export'); await esc(p, 2); });
    await step('prefs theme sub', async () => { await p.click('#mbPrefsBtn'); const r = await p.$('#mbPrefs .mb-sub > [data-sub]'); await r.hover(); await p.waitForTimeout(300); await shot(p, P + '21_menu_prefs_sub'); await esc(p, 2); });
    await step('window workspace sub', async () => { await p.click('#mbWindowBtn'); const rs = await p.$$('#mbWindow .mb-sub > [data-sub]'); if (rs.length) { await rs[rs.length - 1].hover(); await p.waitForTimeout(300); } await shot(p, P + '21_menu_window_sub'); await esc(p, 2); });
    await step('menu hover row', async () => { await p.click('#mbRecordsBtn'); await p.hover('#mbRecords [data-cmd="places"]'); await p.waitForTimeout(200); await clipShot(p, { x: 200, y: 0, width: 420, height: 200 }, P + '22_menu_row_hover'); await esc(p, 1); });

    // modals
    await step('factions', async () => { await p.click('#mbRecordsBtn'); await p.click('#mbRecords [data-cmd="factions"]'); await p.waitForTimeout(900); await shot(p, P + '30_factions_modal'); save(P + 'contrast_factions', await p.evaluate(() => __aud.contrast('#civFactionsModal'))); });
    for (const tab of ['global', 'places', 'types']) {
      await step('settlements tab ' + tab, async () => { await p.click(`#civPmBar button[data-pm="${tab}"]`); await p.waitForTimeout(700); await shot(p, P + '31_settlements_' + tab); });
    }
    await step('factions drawer', async () => { await p.click('#civPmBar button[data-pm="factions"]'); await p.waitForTimeout(400); const row = await p.$('#civFactionsModal .civ-fac-row, #civFactionsModal [data-fac], #civFactionsModal .fac-row'); if (row) { await row.click(); await p.waitForTimeout(700); await shot(p, P + '32_faction_drawer'); } else log('no faction row selector'); });
    await step('close factions', async () => { await p.click('#civFactionsModal .cfm-close'); await p.waitForTimeout(300); });
    await step('settings pages', async () => {
      await p.click('#mbPrefsBtn'); await p.click('#mbPrefs [data-cmd="setPerf"]'); await p.waitForTimeout(600);
      for (const pg of ['perf', 'storage', 'appearance', 'workspace', 'about']) { await p.click(`#setNav [data-setpage="${pg}"]`); await p.waitForTimeout(350); await shot(p, P + '40_settings_' + pg); }
      save(P + 'contrast_settings', await p.evaluate(() => __aud.contrast('#settingsModal')));
      await p.click('#settingsClose'); await p.waitForTimeout(300);
    });
    await step('credits', async () => { await p.click('#mbHelpBtn'); await p.click('#mbHelp [data-cmd="credits"]'); await p.waitForTimeout(500); await shot(p, P + '41_credits'); save(P + 'contrast_credits', await p.evaluate(() => __aud.contrast('#creditsModal'))); await p.click('#creditsModal .cr-close'); await p.waitForTimeout(300); });
    await step('geninfo', async () => { await p.click('#mbHelpBtn'); await p.click('#mbHelp [data-cmd="genInfo"]'); await p.waitForTimeout(600); await shot(p, P + '42_geninfo'); await esc(p, 1); const open = await p.evaluate(() => document.getElementById('settingsModal').classList.contains('open')); if (open) await p.click('#settingsClose'); });
    await step('find', async () => { await p.click('#findInput'); await p.keyboard.type('sea'); await p.waitForTimeout(500); await shot(p, P + '43_find'); await esc(p, 2); });
    await step('map menu', async () => { const w = await p.evaluate(() => { const r = document.querySelector('.canvas-wrap').getBoundingClientRect(); return { x: r.left + r.width * 0.5, y: r.top + r.height * 0.4 }; }); await p.mouse.click(w.x, w.y, { button: 'right' }); await p.waitForTimeout(400); await shot(p, P + '44_map_context_menu'); await esc(p, 2); });
    await step('base map chip', async () => { await p.click('#layersBtn'); await p.waitForTimeout(400); await shot(p, P + '45_basemap_popover'); await p.click('#layersBtn'); await p.waitForTimeout(200); });
    await step('assets', async () => {
      await p.click('#assetsHeaderBtn'); await p.waitForTimeout(1200); await shot(p, P + '50_asset_library'); save(P + 'contrast_assetlib', await p.evaluate(() => __aud.contrast('#assetLibrary')));
      await p.click('#mbAssetsBtn'); await p.click('#mbAssets [data-cmd="slicer"]'); await p.waitForTimeout(700); await shot(p, P + '51_slicer');
      const c = await p.$('.al-modal .al-dlg-h button'); if (c) await c.click(); await p.waitForTimeout(300);
      await p.click('#assetsHeaderBtn'); await p.waitForTimeout(800);
    });
    // domains and their docks
    await step('world generate dock', async () => { await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(400); await shot(p, P + '60_world_generate'); await dockSeries(p, '.dock-left', P + '61_dock_world_generate', 14); save(P + 'controls_world', await p.evaluate(() => __aud.controls('.dock-left'))); });
    await step('world sculpt', async () => { await p.click('#worldCatSeg [data-wcat="sculpt"]'); await p.waitForTimeout(300); await p.click('#sculptFeatureSeg [data-sfeat="freehand"]'); await p.waitForTimeout(500); await shot(p, P + '62_world_sculpt'); await dockSeries(p, '.dock-left', P + '63_dock_world_sculpt', 8); await p.click('#worldCatSeg [data-wcat="terrain"]'); await p.waitForTimeout(300); });
    await step('civil docks', async () => { await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(300); for (const sub of ['factions', 'generation', 'settlements', 'economy', 'statistics']) { await p.click(`[data-civsub="${sub}"]`); await p.waitForTimeout(500); await dockSeries(p, '.dock-left', P + '64_dock_civ_' + sub, 6); } save(P + 'controls_civ', await p.evaluate(() => __aud.controls('.dock-left'))); });
    await step('carto', async () => { await p.click('.domain-rail [data-domain="carto"]'); await p.waitForTimeout(500); await shot(p, P + '65_carto'); await dockSeries(p, '.dock-left', P + '66_dock_carto', 12); save(P + 'controls_carto', await p.evaluate(() => __aud.controls('.dock-left'))); });
    await step('explore', async () => { await p.click('.domain-rail [data-domain="explore"]'); await p.waitForTimeout(500); await shot(p, P + '67_explore'); await dockSeries(p, '.dock-left', P + '68_dock_explore', 8); save(P + 'controls_explore', await p.evaluate(() => __aud.controls('.dock-left'))); });
    await step('route editor', async () => {
      // Explore ▸ Route tool, two clicks on two settlements, Commit route, then Edit route…
      await p.click('[data-civtool="route"]'); await p.waitForTimeout(300);
      const two = await p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); const out = []; for (const q of [...state.places].filter(x => x.pop).sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) out.push({ x: s[0], y: s[1] }); if (out.length === 2) break; } return out; });
      for (const q of two) { await p.mouse.click(q.x, q.y); await p.waitForTimeout(400); }
      await p.click('#civCommitRouteBtn'); await p.waitForTimeout(1200); await L.idle(p, 400);
      await shot(p, P + '69_explore_route_committed');
      await p.click('#jpOpenEditorBtn'); await p.waitForTimeout(1500); await shot(p, P + '70_route_editor');
      save(P + 'contrast_routeeditor', await p.evaluate(() => __aud.contrast('#routeEditorModal')));
      await dockSeries(p, '#routeEditorModal .re-body', P + '71_route_editor_body', 5);
      await p.click('#routeEditorModal .re-close'); await p.waitForTimeout(300);
      await p.click('[data-civtool="info"]').catch(() => {}); await p.click('[data-civtool="inspect"]').catch(() => {});
    });
    // hover and focus states
    await step('hover states', async () => {
      await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(300);
      await p.hover('#reseedBtn'); await p.waitForTimeout(150); await clipShot(p, { x: 40, y: 240, width: 400, height: 170 }, P + '80_hover_newseed');
      await p.hover('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(150); await clipShot(p, { x: 0, y: 70, width: 200, height: 420 }, P + '81_hover_rail');
      await p.hover('#pinPlanner'); await p.waitForTimeout(150); await clipShot(p, { x: 1100, y: 36, width: 500, height: 42 }, P + '82_hover_pin');
      await p.hover('#zoomIn'); await p.waitForTimeout(150); await clipShot(p, { x: 1180, y: 600, width: 100, height: 240 }, P + '83_hover_zoom');
    });
    await step('focus states', async () => {
      await p.click('#seedN'); await p.waitForTimeout(100);
      const seq = [];
      for (let i = 0; i < 6; i++) { await p.keyboard.press('Tab'); await p.waitForTimeout(120); const d = await p.evaluate(() => { const e = document.activeElement; const r = e.getBoundingClientRect(); const cs = getComputedStyle(e); return { el: __aud.desc(e), r: [r.left, r.top, r.width, r.height].map(Math.round), outline: cs.outlineStyle + ' ' + cs.outlineWidth + ' ' + cs.outlineColor, fv: e.matches(':focus-visible') }; }); seq.push(d); await clipShot(p, { x: Math.max(0, d.r[0] - 20), y: Math.max(0, d.r[1] - 20), width: Math.min(600, d.r[2] + 40), height: Math.min(300, d.r[3] + 40) }, P + '84_focus_' + i); }
      save(P + 'focus_seq', seq);
    });
  }
  // the other sizes: WORLD generate, then CIVIL with a selection
  for (const [W, H] of SIZES) {
    await step('size ' + W, async () => {
      await p.setViewportSize({ width: W, height: H }); await p.waitForTimeout(900);
      await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(400);
      await shot(p, P + `90_world_${W}`);
      await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(400);
      await fit(p);
      const q = await biggestVisible(p); if (q) { await p.mouse.click(q.x, q.y); await p.waitForTimeout(700); }
      if (await p.evaluate(() => document.getElementById('tlStripMore').hidden)) await p.click('#tlStripToggle');
      await p.waitForTimeout(400);
      await shot(p, P + `91_civil_selected_${W}`);
      save(P + `geom_${W}`, await p.evaluate(() => ({ truncs: __aud.truncs(), off: __aud.offscreen() })));
      await p.click('#tlStripToggle'); await p.waitForTimeout(200);
      const c = await p.$('#placeEditPopup .si-close'); if (c && await c.isVisible()) await c.click();
    });
  }
  log('errors:', p._errs.length ? p._errs.join(' | ') : 'none');
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
