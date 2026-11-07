/* v2.74's evidence captures (review only; Cartalith_RC needs none of them): the sixteen views the version's review
 * looks at — the fix pass and the design alignment together — written to html/evidence/v2.74/ (or the folder given
 * second), replacing whatever v274_*.png are there. Reached by real clicks and taps as probe_fixpass.js reaches them;
 * page.evaluate only for SET-UP (a 256 world at seed 12345 with settlements and polities, two recorded years, a
 * priced route, the theme in storage before load) and to open every section of a dock for the capture.
 * Usage: node tests/perf/shots_v274.js "Cartalith v2.74 DCC test.html" [outDir]   (env PLAYWRIGHT_DIR, CHROME_BIN)
 * Output: one line per file written; exit 2 on a missing file.
 */
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_DIR || 'playwright');
const FILE = process.argv[2], OUT = process.argv[3] || path.join(__dirname, '..', '..', 'evidence', 'v2.74');
if (!FILE || !fs.existsSync(FILE) || process.argv.length > 4) { console.error('usage: node shots_v274.js <file.html> [outDir]'); process.exit(2); }
fs.mkdirSync(OUT, { recursive: true });
for (const f of fs.readdirSync(OUT)) if (/^v274_.*\.png$/.test(f)) fs.unlinkSync(path.join(OUT, f));
const sleep = ms => new Promise(r => setTimeout(r, ms));
const AND = 'Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Mobile Safari/537.36';
(async () => {
  const b = await chromium.launch({ executablePath: process.env.CHROME_BIN, args: ['--no-sandbox'] });
  const boot = async (vp, light, touch) => { const ctx = await b.newContext({ viewport: vp, hasTouch: !!touch, isMobile: !!touch && vp.width < 1000, userAgent: touch ? AND : undefined });
    await ctx.addInitScript(l => { try { localStorage.clear(); localStorage.setItem('cartalith_theme', l ? 'light' : 'dark'); } catch (_) {} }, !!light);
    const p = await ctx.newPage(); p.on('dialog', d => d.accept().catch(() => {})); await p.goto('file://' + path.resolve(FILE)); await p.waitForFunction(() => typeof generate === 'function'); await sleep(600); p._ctx = ctx; return p; };
  /* SET-UP: the world, its settlements and polities, and two recorded years */
  const world = p => p.evaluate(async () => { state.tect.seed = 12345; state.resW = 256; GW = 256; GH = gridH(GW); allocate(); _setupHide(); await generate(); syncUI(); _viewFill(); renderNow(); _civIterativeAutoWorld(3); _civAutoPolity(); drawCivLayerAuto();
    const y = document.getElementById('civTlYear'); y.value = 100; document.getElementById('civTlAddYearBtn').click(); y.value = 250; document.getElementById('civTlAddYearBtn').click(); });
  const click = async (p, sel) => { const h = await p.$(sel); if (!h) return false; await h.scrollIntoViewIfNeeded().catch(() => {}); const r = await h.boundingBox(); if (!r || r.width < 1) return false; await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await sleep(300); return true; };
  const tap = async (p, sel) => { const h = await p.$(sel); if (!h) return false; await h.scrollIntoViewIfNeeded().catch(() => {}); const r = await h.boundingBox(); if (!r || r.width < 1) return false; await p.touchscreen.tap(r.x + r.width / 2, r.y + r.height / 2); await sleep(400); return true; };
  const shot = async (p, name, clip) => { const f = path.join(OUT, 'v274_' + name + '.png'); await p.screenshot({ path: f, clip }); console.log(f); };
  const place = p => p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); for (const q of [...state.places].filter(x => x.pop && x.category === 'settlement').sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) return { x: s[0], y: s[1] }; } return null; });
  const route = p => p.evaluate(() => { const S = state.places.filter(q => q.category === 'settlement').sort((a, b) => b.pop - a.pop);   /* SET-UP: a priced route */
    for (let i = 0; i < S.length; i++) for (let k = i + 1; k < Math.min(S.length, i + 6); k++) { _civWaypoints = [[S[i].x, S[i].y], [S[k].x, S[k].y]]; _civCommitRoute(); const jn = civJourneys[civJourneys.length - 1]; const pl = _jpPlan(jn); if (pl && jpJourneyCost(pl)) return; } });
  /* 1-4: the desktop at 1366 and 1600, dark and light — WORLD › Generate at 1366, CIVIL › Generation at 1600, the strip open */
  for (const [w, h] of [[1366, 768], [1600, 900]]) for (const light of [false, true]) {
    const p = await boot({ width: w, height: h }, light); await world(p);
    if (w === 1600) { await click(p, '#domainRail [data-domain="civ"]'); await click(p, '[data-civsub="generation"]'); }
    await click(p, '#tlStripToggle'); await sleep(300);
    await shot(p, 'desktop' + w + '_' + (light ? 'light' : 'dark'));
    if (w === 1366 && !light) {   /* 11: the sculpt bar with Freehand armed */
      await click(p, '#tlStripToggle'); await click(p, '#domainRail [data-domain="world"]'); await click(p, '#worldCatSeg [data-wcat="sculpt"]'); await click(p, '#sculptFeatureSeg [data-sfeat="freehand"]'); await sleep(500);
      await shot(p, 'sculpt_freehand_1366', { x: 0, y: 0, width: 1366, height: 330 });
    }
    if (w === 1600 && light) {    /* 9: the place popup, light */
      const at = await place(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(700); } await shot(p, 'place_popup_1600_light');
    }
    if (w === 1600 && !light) {   /* 7, 8, 10, 13: the Settlements screen, the route editor, File ▸ Export, the asset library */
      await click(p, '#mbRecordsBtn'); await click(p, '#mbRecords [data-cmd="factions"]'); await sleep(700); await shot(p, 'settlements_1600_dark'); await click(p, '#cfmCloseBtn');
      await click(p, '#fileMenuBtn'); const er = await p.$('#mbExportRow'); const r = er && await er.boundingBox(); if (r) { await p.mouse.move(r.x + r.width / 2, r.y + r.height / 2); await sleep(400); }
      await shot(p, 'file_export_1600_dark'); await p.keyboard.press('Escape');
      await click(p, '#assetsHeaderBtn'); await sleep(900); await p.mouse.move(3, 400); await shot(p, 'asset_library_1600_dark'); await click(p, '#assetsHeaderBtn'); await sleep(500);
      await click(p, '#domainRail [data-domain="explore"]'); await click(p, '[data-civtool="route"]'); await route(p); await sleep(800);
      if (await click(p, '#jpOpenEditorBtn')) { await p.waitForFunction(() => /Priced in/.test(document.getElementById('routeEditorModal').textContent), null, { timeout: 15000 }).catch(() => {}); await shot(p, 'route_editor_1600_dark'); }
    }
    await p._ctx.close();
  }
  /* 5, 6: 1920 (CARTO, its sections open) and 2560 (EXPLORE, a settlement selected), dark */
  { const p = await boot({ width: 1920, height: 1080 }); await world(p); await click(p, '#domainRail [data-domain="carto"]');
    await p.evaluate(() => { for (const d of document.querySelectorAll('#genCarto details')) d.open = true; }); await click(p, '#tlStripToggle'); await sleep(300); await shot(p, 'desktop1920_dark'); await p._ctx.close(); }
  { const p = await boot({ width: 2560, height: 1440 }); await world(p); await click(p, '#domainRail [data-domain="explore"]'); await click(p, '#tlStripToggle'); await sleep(300);
    const at = await place(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(700); } await shot(p, 'desktop2560_dark'); await p._ctx.close(); }
  /* 12: New World */
  { const p = await boot({ width: 1366, height: 768 }); await click(p, '#obGenerate'); await sleep(400); await shot(p, 'newworld_1366_dark'); await p._ctx.close(); }
  /* 14, 15: the phone at 412×915 — the map, and the Settlements screen's faction rows (MENU › Records › Factions…) */
  { const p = await boot({ width: 412, height: 915 }, false, true); await world(p); await shot(p, 'phone412_map');
    await tap(p, '#phTabMenu'); await tap(p, '#phMenuRoot > .mb-sub[data-menu="records"] > [data-sub]'); await tap(p, '#mbRecords [data-cmd="factions"]'); await sleep(700);
    await tap(p, '#civPmBar button[data-pm="factions"]'); await sleep(500); await shot(p, 'phone412_settlements'); await p._ctx.close(); }
  /* 16: the tablet at 1194×834, WORLD › Generate */
  { const p = await boot({ width: 1194, height: 834 }, false, true); await world(p); await shot(p, 'tablet1194'); await p._ctx.close(); }
  await b.close();
})();
