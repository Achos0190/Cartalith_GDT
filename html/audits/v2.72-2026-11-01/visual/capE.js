// Phase E: targeted measurements behind specific findings, each with a before/after where a CSS change is
// proposed. CSS is injected into this scratch page only (page.addStyleTag); the repo file is never touched.
const L = require('./lib.js');
const { log, shot, clipShot, save } = L;
const out = {};
async function step(name, fn) { try { out[name] = await fn(); log('ok', name, JSON.stringify(out[name]).slice(0, 300)); } catch (e) { out[name] = { error: e.message.split('\n')[0] }; log('STEP FAILED', name, e.message.split('\n')[0]); } }
const rect = (p, sel) => p.evaluate(s => { const e = document.querySelector(s); if (!e) return null; const r = e.getBoundingClientRect(); return [r.left, r.top, r.right, r.bottom].map(v => Math.round(v)); }, sel);
const css = (p, sel, props) => p.evaluate(([s, pr]) => { const e = document.querySelector(s); if (!e) return null; const cs = getComputedStyle(e); const o = {}; for (const k of pr) o[k] = cs.getPropertyValue(k); return o; }, [sel, props]);

(async () => {
  const b = await L.chromium.launch();
  const p = await L.boot(b, L.V272, { width: 1600, height: 900 });
  await L.installAudit(p);
  // A. the New World card
  await p.click('#obGenerate'); await p.waitForTimeout(300);
  await step('A1 world-shape buttons that overflow their cell', () => p.evaluate(() => [...document.querySelectorAll('#suArchSeg button')].map(b => ({ t: b.textContent.trim(), cw: b.clientWidth, sw: b.scrollWidth, h: b.getBoundingClientRect().height })).filter(x => x.sw > x.cw || x.h > 0)));
  await step('A2 sticky actions: what shows between the bar and the card bottom when scrolled half-way', async () => {
    const r = await p.evaluate(() => { const c = document.querySelector('#onboard .card'); c.scrollTop = Math.round((c.scrollHeight - c.clientHeight) / 2); const cr = c.getBoundingClientRect(); const bar = document.querySelector('#obStepGenerate .su-actions').getBoundingClientRect(); const y = Math.round((bar.bottom + cr.bottom) / 2); const x = Math.round(cr.left + cr.width / 2); const el = document.elementFromPoint(x, y); return { card: [cr.top, cr.bottom].map(Math.round), bar: [bar.top, bar.bottom].map(Math.round), gap: Math.round(cr.bottom - bar.bottom), y, hit: el ? (el.id || el.className || el.tagName) + ' :: ' + (el.textContent || '').trim().slice(0, 40) : null, scrollTop: c.scrollTop, scrollMax: c.scrollHeight - c.clientHeight }; });
    await clipShot(p, { x: 560, y: Math.max(0, r.bar[0] - 120), width: 480, height: Math.min(260, 900 - Math.max(0, r.bar[0] - 120)) }, 'e_A2_sticky_gap_before');
    await p.addStyleTag({ content: '#onboard .su-actions{bottom:-24px}' });
    await p.waitForTimeout(150);
    const r2 = await p.evaluate(() => { const c = document.querySelector('#onboard .card'); const cr = c.getBoundingClientRect(); const bar = document.querySelector('#obStepGenerate .su-actions').getBoundingClientRect(); return { gapAfter: Math.round(cr.bottom - bar.bottom) }; });
    await clipShot(p, { x: 560, y: Math.max(0, r.bar[0] - 120), width: 480, height: Math.min(260, 900 - Math.max(0, r.bar[0] - 120)) }, 'e_A2_sticky_gap_after');
    return { ...r, ...r2 };
  });
  await p.evaluate(() => { const c = document.querySelector('#onboard .card'); c.scrollTop = 0; });
  await step('A3 world-shape seg as a 3-column grid (after)', async () => {
    await clipShot(p, { x: 600, y: 280, width: 400, height: 70 }, 'e_A3_worldshape_before');
    await p.addStyleTag({ content: '#onboard #suArchSeg{display:grid;grid-template-columns:repeat(3,1fr)}' });
    await p.waitForTimeout(150);
    await clipShot(p, { x: 600, y: 280, width: 400, height: 100 }, 'e_A3_worldshape_after');
    return p.evaluate(() => [...document.querySelectorAll('#suArchSeg button')].filter(b => b.scrollWidth > b.clientWidth).length);
  });
  // reload for a clean page (the injected styles must not leak into later measurements)
  await p.reload(); await p.waitForFunction(() => typeof window.generate === 'function'); await p.waitForTimeout(800);
  await L.installAudit(p);
  await L.genByClicks(p, 12345, '', null);
  await L.installAudit(p);
  // C. the status readout, clipped?
  for (const [W, H] of [[1366, 768], [1600, 900], [1920, 1080], [2560, 1440]]) {
    await p.setViewportSize({ width: W, height: H }); await p.waitForTimeout(500);
    await step('C status readout ' + W, () => p.evaluate(() => { const r = document.getElementById('readout'); const kids = [...r.children].filter(k => getComputedStyle(k).display !== 'none'); const rr = r.getBoundingClientRect(); const cut = kids.filter(k => k.getBoundingClientRect().right > rr.right + 0.5).map(k => k.textContent.trim().slice(0, 40)); return { client: r.clientWidth, scroll: r.scrollWidth, cut }; }));
    await step('P ctx hint ' + W, () => p.evaluate(() => { const h = document.querySelector('.ctx-hint'); return h ? { client: h.clientWidth, scroll: h.scrollWidth, text: h.textContent } : null; }));
  }
  await p.setViewportSize({ width: 1600, height: 900 }); await p.waitForTimeout(500);
  // D/E. CIVIL: sub-tab active border, the settlements table, populate first
  await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(300);
  await p.click('[data-civsub="generation"]'); await p.waitForTimeout(300);
  await p.click('#civAutoPopulateBtn', { noWaitAfter: true, timeout: 120000 }); await p.waitForTimeout(500); await L.idle(p, 800);
  await step('D1 active sub-tab computed border (intended: underline only)', () => css(p, '.subtab.on', ['border-top', 'border-left', 'border-right', 'border-bottom', 'font-weight', 'background-color']));
  for (const [W, H] of [[1366, 768], [1600, 900]]) {
    await p.setViewportSize({ width: W, height: H }); await p.waitForTimeout(400);
    await step('Q civ sub-tab bar overflow ' + W, () => p.evaluate(() => { const bar = document.querySelector('[data-civsub="generation"]').parentElement; const d = document.querySelector('.dock-left'); return { bar: [bar.clientWidth, bar.scrollWidth], dock: [d.clientWidth, d.scrollWidth], tabs: [...bar.children].map(t => Math.round(t.getBoundingClientRect().right)), dockRight: Math.round(d.getBoundingClientRect().right) }; }));
  }
  await clipShot(p, { x: 40, y: 460, width: 400, height: 60 }, 'e_Q_civtabs_1366');
  await p.setViewportSize({ width: 1600, height: 900 }); await p.waitForTimeout(400);
  await p.click('[data-civsub="settlements"]'); await p.waitForTimeout(600);
  await step('E1 settlements table row display', () => p.evaluate(() => { const rows = [...document.querySelectorAll('#stSpacer > div')].filter(r => r.style.display !== 'none'); return { rows: rows.length, display: rows.slice(0, 3).map(r => getComputedStyle(r).display), inline: rows.slice(0, 3).map(r => r.getAttribute('style').match(/display:[^;]*/) ? 'has display' : 'no display in inline style') }; }));
  const tb = await rect(p, '#stViewport');
  if (tb) {
    await clipShot(p, { x: tb[0] - 4, y: tb[1] - 40, width: tb[2] - tb[0] + 8, height: 200 }, 'e_E1_table_before');
    await p.addStyleTag({ content: '#stSpacer > div{display:flex}' }); await p.waitForTimeout(200);
    await clipShot(p, { x: tb[0] - 4, y: tb[1] - 40, width: tb[2] - tb[0] + 8, height: 200 }, 'e_E1_table_after');
  }
  // F. color-scheme: the native checkboxes and range tracks before and after
  await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(400);
  await p.evaluate(() => { const d = document.querySelector('.dock-left'); const t = [...d.querySelectorAll('.sec h2,.sublabel-grp,.sublabel')].find(e => /planet/i.test(e.textContent)); if (t) t.scrollIntoView({ block: 'start' }); });
  await p.waitForTimeout(300);
  await clipShot(p, { x: 40, y: 80, width: 400, height: 480 }, 'e_F_colorscheme_before');
  await p.addStyleTag({ content: ':root{color-scheme:dark} :root[data-theme="light"]{color-scheme:light} input[type=checkbox],input[type=radio]{accent-color:var(--accent)}' });
  await p.waitForTimeout(300);
  await clipShot(p, { x: 40, y: 80, width: 400, height: 480 }, 'e_F_colorscheme_after');
  await p.reload(); await p.waitForFunction(() => typeof window.generate === 'function'); await p.waitForTimeout(800);
  await L.installAudit(p);
  await L.genByClicks(p, 12345, '1024', null);
  await L.installAudit(p);
  // G. the Assets pin in Assets mode, at rest, hovered, and with the slicer open
  await p.click('#assetsHeaderBtn'); await p.waitForTimeout(1000); await p.mouse.move(800, 500);
  await step('G1 Assets pin in Assets mode, at rest', () => css(p, '#assetsHeaderBtn', ['background-color', 'color', 'border-top-color']));
  await p.hover('#assetsHeaderBtn'); await p.waitForTimeout(150);
  await step('G2 Assets pin hovered', () => css(p, '#assetsHeaderBtn', ['background-color', 'color', 'border-top-color']));
  await clipShot(p, { x: 1260, y: 38, width: 340, height: 38 }, 'e_G2_assets_pin_hover');
  await p.mouse.move(800, 500); await p.waitForTimeout(150);
  await clipShot(p, { x: 1260, y: 38, width: 340, height: 38 }, 'e_G1_assets_pin_rest');
  await step('G3 empty asset tiles: background', () => p.evaluate(() => { const t = document.querySelector('.al-card .al-thumb'); return t ? { bg: getComputedStyle(t).backgroundColor, empty: !!t.querySelector('.al-empty'), var: getComputedStyle(document.documentElement).getPropertyValue('--preview-al') } : null; }));
  await clipShot(p, { x: 190, y: 78, width: 760, height: 480 }, 'e_G3_tiles_before');
  await p.addStyleTag({ content: '.al-card .al-thumb:has(> .al-empty){background:var(--ins)!important} .al-card .al-thumb .al-empty{color:var(--dim)}' });
  await p.waitForTimeout(200);
  await clipShot(p, { x: 190, y: 78, width: 760, height: 480 }, 'e_G3_tiles_after');
  await p.click('#assetsHeaderBtn'); await p.waitForTimeout(800);
  // H. File ▸ Export: the GeoJSON button
  await p.click('#fileMenuBtn'); await p.hover('#mbExportRow'); await p.waitForTimeout(300);
  await step('H Export GeoJSON button computed', () => css(p, '#exportGeoBtn', ['background-color', 'border-top-width', 'border-top-style', 'border-top-color', 'padding-top', 'color']));
  await step('H2 Export .zip button computed', () => css(p, '#exportBtn', ['background-color', 'border-top-width', 'color', 'font-family']));
  const gb = await rect(p, '#exportGeoBtn');
  if (gb) await clipShot(p, { x: gb[0] - 12, y: gb[1] - 40, width: 340, height: 90 }, 'e_H_geojson_button');
  await p.keyboard.press('Escape'); await p.keyboard.press('Escape');
  // I. the place popup Name row
  await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(300);
  await p.click('[data-civsub="generation"]'); await p.waitForTimeout(300);
  await p.click('#civAutoPopulateBtn', { noWaitAfter: true, timeout: 120000 }); await p.waitForTimeout(500); await L.idle(p, 800);
  const at = await p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); for (const q of [...state.places].filter(x => x.pop).sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) return { x: s[0], y: s[1] }; } return null; });
  if (at) { await p.mouse.click(at.x, at.y); await p.waitForTimeout(700); }
  await step('I1 popup Name row overflow', () => p.evaluate(() => { const pp = document.getElementById('placeEditPopup'); const roll = document.getElementById('_civPeNameRoll'); const pr = pp.getBoundingClientRect(), rr = roll ? roll.getBoundingClientRect() : null; return { popup: [pp.clientWidth, pp.scrollWidth], rollRight: rr && Math.round(rr.right), popupInnerRight: Math.round(pr.right - parseFloat(getComputedStyle(pp).paddingRight)), popupRight: Math.round(pr.right), bgAlpha: getComputedStyle(pp).backgroundColor }; }));
  await p.addStyleTag({ content: '.place-edit-popup .row input{min-width:0}' }); await p.waitForTimeout(150);
  await step('I2 popup Name row after min-width:0', () => p.evaluate(() => { const pp = document.getElementById('placeEditPopup'); const rr = document.getElementById('_civPeNameRoll').getBoundingClientRect(); return { popup: [pp.clientWidth, pp.scrollWidth], rollRight: Math.round(rr.right) }; }));
  const pr = await rect(p, '#placeEditPopup');
  if (pr) await clipShot(p, { x: pr[0] - 4, y: pr[1] + 240, width: pr[2] - pr[0] + 8, height: 60 }, 'e_I2_popup_name_after');
  // N. 1366 with the strip expanded: popup vs strip
  await p.setViewportSize({ width: 1366, height: 768 }); await p.waitForTimeout(700);
  if (await p.evaluate(() => document.getElementById('tlStripMore').hidden)) await p.click('#tlStripToggle');
  await p.waitForTimeout(400);
  await step('N popup vs strip at 1366', async () => ({ popup: await rect(p, '#placeEditPopup'), strip: await rect(p, '#tlStrip'), map: await rect(p, '.canvas-wrap') }));
  await p.setViewportSize({ width: 1600, height: 900 }); await p.waitForTimeout(600);
  await p.click('#placeEditPopup .si-close').catch(() => {});
  // M. scale bar and legend computed
  await step('M scale bar + legend computed', async () => ({ sb: await css(p, '#scaleBar', ['background-color', 'color']), legend: await css(p, '#legend', ['background-color', 'color']), sbRect: await rect(p, '#scaleBar'), legendRect: await rect(p, '#legend') }));
  // J. the sculpt bar's feature picker
  await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(300);
  await p.click('#worldCatSeg [data-wcat="sculpt"]'); await p.waitForTimeout(300);
  await p.click('#sculptFeatureSeg [data-sfeat="freehand"]'); await p.waitForTimeout(500);
  await step('J sculpt feature picker scroll state', () => p.evaluate(() => { const s = document.getElementById('sculptFeatureSeg'); const sr = s.getBoundingClientRect(); const first = [...s.children].find(c => c.getBoundingClientRect().right > sr.left + 1); return { scrollLeft: Math.round(s.scrollLeft), client: s.clientWidth, scroll: s.scrollWidth, hiddenLeft: [...s.children].filter(c => c.getBoundingClientRect().right <= sr.left + 1).map(c => c.textContent.trim()), partialFirst: first && first.getBoundingClientRect().left < sr.left - 0.5 ? first.textContent.trim() : null, hiddenRight: [...s.children].filter(c => c.getBoundingClientRect().left >= sr.right - 1).map(c => c.textContent.trim()) }; }));
  await step('J2 sculpt hint text', () => p.evaluate(() => document.getElementById('sculptFeatHint').textContent));
  await clipShot(p, { x: 0, y: 36, width: 1600, height: 80 }, 'e_J_sculpt_bar');
  // L. the two map chips in EXPLORE
  await p.click('#worldCatSeg [data-wcat="terrain"]'); await p.waitForTimeout(200);
  await p.click('.domain-rail [data-domain="explore"]'); await p.waitForTimeout(400);
  await step('L map chips heights', async () => ({ base: await rect(p, '#layersBtn'), filter: await rect(p, '#explFilterBtn') }));
  await clipShot(p, { x: 440, y: 78, width: 220, height: 60 }, 'e_L_map_chips');
  // D2. settings nav active item
  await p.click('#mbPrefsBtn'); await p.click('#mbPrefs [data-cmd="setPerf"]'); await p.waitForTimeout(500);
  await step('D2 settings nav active item computed border (intended: left bar only)', () => css(p, '#setNav button.on', ['border-top', 'border-right', 'border-bottom', 'border-left', 'font-weight']));
  const nr = await rect(p, '#setNav');
  if (nr) await clipShot(p, { x: nr[0], y: nr[1], width: 200, height: 120 }, 'e_D2_setnav_before');
  await p.addStyleTag({ content: '.set-nav button.on{border:0;border-left:2px solid var(--accent);font-weight:inherit} .subtab.on{border:0;border-bottom:2px solid var(--accent);font-weight:inherit}' });
  await p.waitForTimeout(150);
  if (nr) await clipShot(p, { x: nr[0], y: nr[1], width: 200, height: 120 }, 'e_D2_setnav_after');
  await p.click('#settingsClose');
  save('e_measures', out);
  log('errors:', p._errs.length ? p._errs.join(' | ') : 'none');
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
