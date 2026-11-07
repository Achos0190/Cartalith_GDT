/* v2.74's contrast gate (html/audits/v2.72-2026-11-01/PLAN.md, "The v2.74 gate"): the visual audit's in-page WCAG
 * census (its lib.js __aud.contrast, the arithmetic unchanged) over the seven views it audited — a place selected
 * on the map (the whole page), the Settlements screen, the Asset Library, the route editor, the City viewer,
 * Settings (every page) and Credits — in both themes, at 1600×900 and 1366×768.
 * The bar: 0 ENABLED text runs below 4.5:1 (3:1 for large text), or each remaining failure listed by element
 * and ratio. Disabled controls' text is counted and listed apart (WCAG exempts it).
 *
 * The views are reached by real clicks, as the audit's capB.js reached them; page.evaluate is used for SET-UP
 * (a 256 world at seed 12345 with settlements and polities, the theme in storage before load) and for READS (the
 * census itself). Over the map the census composites a translucent box over the page's own background, not
 * over terrain (the audit's limit too: the legend and scale bar over real terrain are V21's computed pairs).
 *
 * Usage: node tests/perf/census_v274.js "Cartalith v2.74 DCC test.html"   (env PLAYWRIGHT_DIR, CHROME_BIN)
 * Output: one line per view × theme × size (runs, failures), then each failure; exit 1 if any enabled run fails.
 */
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_DIR || 'playwright');
const FILE = process.argv[2];
if (!FILE || process.argv.length > 3 || !fs.existsSync(FILE)) { console.error('usage: node census_v274.js <file.html>'); process.exit(2); }
const sleep = ms => new Promise(r => setTimeout(r, ms));
/* the census, as lib.js wrote it (READ only) */
const AUD = `(${function () {
  const parse = s => { const m = s.match(/rgba?\(([^)]+)\)/); if (!m) return null; const v = m[1].split(/[ ,\/]+/).filter(Boolean).map(Number); return { r: v[0], g: v[1], b: v[2], a: v.length > 3 ? v[3] : 1 }; };
  const over = (f, b) => ({ r: f.r * f.a + b.r * (1 - f.a), g: f.g * f.a + b.g * (1 - f.a), b: f.b * f.a + b.b * (1 - f.a), a: 1 });
  const lum = c => { const t = x => { x /= 255; return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4); }; return 0.2126 * t(c.r) + 0.7152 * t(c.g) + 0.0722 * t(c.b); };
  const ratio = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const vis = el => { if (!el.isConnected) return false; const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; for (let e = el; e && e.nodeType === 1; e = e.parentElement) { const cs = getComputedStyle(e); if (cs.display === 'none' || cs.visibility === 'hidden' || +cs.opacity === 0) return false; } return r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth; };
  const bgOf = el => { const stack = []; for (let e = el; e && e.nodeType === 1; e = e.parentElement) { const c = parse(getComputedStyle(e).backgroundColor); if (c && c.a > 0) { stack.push(c); if (c.a >= 0.999) break; } }
    let base = parse(getComputedStyle(document.body).backgroundColor) || { r: 0, g: 0, b: 0, a: 1 }; for (let i = stack.length - 1; i >= 0; i--) base = over(stack[i], base); return base; };
  const opac = el => { let o = 1; for (let e = el; e && e.nodeType === 1; e = e.parentElement) o *= +getComputedStyle(e).opacity; return o; };
  const desc = el => { let s = el.tagName.toLowerCase(); if (el.id) s += '#' + el.id; else if (el.className && typeof el.className === 'string') s += '.' + el.className.trim().split(/\s+/).slice(0, 2).join('.'); const pid = el.closest('[id]'); if (pid && pid !== el) s = pid.tagName.toLowerCase() + '#' + pid.id + ' ' + s; return s; };
  window.__census = rootSel => { const root = rootSel ? document.querySelector(rootSel) : document.body; const out = []; if (!root) return out;
    const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT); const seen = new Set();
    while (w.nextNode()) { const t = w.currentNode; if (!t.nodeValue.trim()) continue; const el = t.parentElement; if (!el || seen.has(el)) continue; seen.add(el);
      if (!vis(el) || el.closest('option')) continue; const cs = getComputedStyle(el); const fg = parse(cs.color); if (!fg) continue;
      const b0 = bgOf(el), o = opac(el), body = parse(getComputedStyle(document.body).backgroundColor); let f = over(fg, b0), bb = b0;
      if (o < 1) { f = over({ ...f, a: o }, body); bb = over({ ...b0, a: o }, body); }
      const cr = ratio(f, bb), px = parseFloat(cs.fontSize), large = px >= 24 || (+cs.fontWeight >= 600 && px >= 18.66);
      out.push({ el: desc(el), text: t.nodeValue.trim().slice(0, 40), ratio: +cr.toFixed(2), fail: cr < (large ? 3 : 4.5), dis: !!el.closest(':disabled,[disabled],[aria-disabled="true"]') }); }
    return out; };
}})();`;
(async () => {
  const b = await chromium.launch({ executablePath: process.env.CHROME_BIN, args: ['--no-sandbox'] });
  const rows = []; let enabledFails = 0;
  const click = async (p, sel) => { const h = await p.$(sel); if (!h) return false; await h.scrollIntoViewIfNeeded().catch(() => {}); const r = await h.boundingBox(); if (!r) return false; await p.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await sleep(250); return true; };
  const idle = async p => { for (let i = 0; i < 300; i++) { const d = await p.evaluate(async () => { try { await _busyChain; } catch (_) {} return _busyDepth || 0; }); if (!d) break; await sleep(200); } await sleep(250); };
  for (const light of [false, true]) for (const vp of [{ width: 1600, height: 900 }, { width: 1366, height: 768 }]) {
    const ctx = await b.newContext({ viewport: vp }); await ctx.addInitScript(AUD);
    await ctx.addInitScript(l => { try { localStorage.clear(); localStorage.setItem('cartalith_theme', l ? 'light' : 'dark'); } catch (_) {} }, light);
    const p = await ctx.newPage(); p.on('dialog', d => d.accept().catch(() => {}));
    await p.goto('file://' + path.resolve(FILE)); await p.waitForFunction(() => typeof generate === 'function', { timeout: 90000 }); await sleep(400);
    /* SET-UP: the world */
    await p.evaluate(async () => { state.tect.seed = 12345; state.resW = 256; GW = 256; GH = gridH(GW); allocate(); _setupHide(); await generate(); syncUI(); _viewFill(); renderNow(); _civIterativeAutoWorld(3); _civAutoPolity(); drawCivLayerAuto(); });
    const tag = (light ? 'light' : 'dark') + ' ' + vp.width + 'x' + vp.height;
    const take = async (view, sel) => { const r = await p.evaluate(s => __census(s), sel); const f = r.filter(x => x.fail && !x.dis), fd = r.filter(x => x.fail && x.dis);
      enabledFails += f.length; rows.push({ tag, view, runs: r.length, fails: f, disFails: fd.length }); };
    await click(p, '#domainRail [data-domain="civ"]');
    const at = await p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); for (const q of [...state.places].filter(x => x.pop && x.category === 'settlement').sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) return { x: s[0], y: s[1] }; } return null; });
    if (at) { await p.mouse.click(at.x, at.y); await sleep(700); }
    await take('place view', null);
    if (await click(p, '#placeEditPopup #peCityOpen')) { await sleep(1500); await take('city viewer', '#cityViewerModal'); await click(p, '#cityViewerModal .cv-close'); }
    await click(p, '#placeEditPopup .si-close');
    await click(p, '#mbRecordsBtn'); await click(p, '#mbRecords [data-cmd="factions"]'); await sleep(800);
    await take('settlements', '#civFactionsModal'); await click(p, '#cfmCloseBtn');
    await click(p, '#mbPrefsBtn'); await click(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(500);
    const all = [];
    for (const pg of ['perf', 'storage', 'appearance', 'workspace', 'about']) { if (await click(p, '#setNav [data-setpage="' + pg + '"]')) { await sleep(300); all.push(...await p.evaluate(() => __census('#settingsModal'))); } }
    { const f = all.filter(x => x.fail && !x.dis); enabledFails += f.length; rows.push({ tag, view: 'settings (5 pages)', runs: all.length, fails: f, disFails: all.filter(x => x.fail && x.dis).length }); }
    await click(p, '#settingsClose');
    await click(p, '#mbHelpBtn'); await click(p, '#mbHelp [data-cmd="credits"]'); await sleep(500);
    await take('credits', '#creditsModal'); await click(p, '#creditsClose');
    await click(p, '#assetsHeaderBtn'); await sleep(1000); await p.mouse.move(3, 300);
    await take('asset library', '#assetLibrary'); await click(p, '#assetsHeaderBtn'); await sleep(500);
    await click(p, '#domainRail [data-domain="explore"]'); await click(p, '[data-civtool="route"]');
    await p.evaluate(() => { const S = state.places.filter(q => q.category === 'settlement').sort((a, b) => b.pop - a.pop);   /* SET-UP: a priced route */
      for (let i = 0; i < S.length; i++) for (let k = i + 1; k < Math.min(S.length, i + 6); k++) { _civWaypoints = [[S[i].x, S[i].y], [S[k].x, S[k].y]]; _civCommitRoute(); const jn = civJourneys[civJourneys.length - 1]; const pl = _jpPlan(jn); if (pl && jpJourneyCost(pl)) return; } });
    await idle(p);
    if (await click(p, '#jpOpenEditorBtn')) { await p.waitForFunction(() => /Priced in/.test(document.getElementById('routeEditorModal').textContent), null, { timeout: 15000 }).catch(() => {}); await take('route editor', '#routeEditorModal'); }
    await ctx.close();
  }
  for (const r of rows) console.log((r.fails.length ? 'FAIL ' : 'ok   ') + r.tag.padEnd(16) + ' ' + r.view.padEnd(20) + ' runs ' + String(r.runs).padStart(4) + '  below AA ' + r.fails.length + (r.disFails ? '  (disabled, exempt: ' + r.disFails + ')' : ''));
  for (const r of rows) for (const f of r.fails) console.log('   ' + r.tag + ' · ' + r.view + ' · ' + f.el + ' "' + f.text + '" ' + f.ratio);
  const views = new Set(rows.map(r => r.view)).size;
  console.log('\n' + rows.length + ' censuses (' + views + ' views × 2 themes × 2 sizes), ' + rows.reduce((s, r) => s + r.runs, 0) + ' text runs; enabled runs below AA: ' + enabledFails);
  await b.close();
  process.exit(enabledFails ? 1 : 0);
})();
