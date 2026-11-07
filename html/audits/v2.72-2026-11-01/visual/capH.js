// Phase H: three stacking/clipping checks by real clicks. (1) Settings ▸ About ▸ ⓘ: does Credits open above the
// Settings window? (2) While the busy overlay is up, which element is under the zoom column and the Base map
// chip? (3) The Settlements screen's faction rows at phone width: is the name drawn?
const L = require('./lib.js');
const { log, shot, save, clipShot } = L;
(async () => {
  const b = await L.chromium.launch();
  const out = {};
  const p = await L.boot(b, L.V272, { width: 1600, height: 900 });
  await p.click('#obSkip'); await p.waitForTimeout(500);
  await p.click('#mbPrefsBtn'); await p.click('#mbPrefs [data-cmd="setPerf"]'); await p.waitForTimeout(400);
  await p.click('#setNav [data-setpage="about"]'); await p.waitForTimeout(300);
  await p.click('#creditsBtn'); await p.waitForTimeout(400);
  out.credits = await p.evaluate(() => { const m = document.getElementById('creditsModal'); const el = document.elementFromPoint(innerWidth / 2, innerHeight / 2); return { creditsOpen: m.classList.contains('open'), zCredits: getComputedStyle(m).zIndex, zSettings: getComputedStyle(document.getElementById('settingsModal')).zIndex, topAtCentre: el ? (el.closest('#creditsModal') ? 'creditsModal' : el.closest('#settingsModal') ? 'settingsModal' : el.tagName) : null }; });
  await shot(p, 'h_credits_from_settings_1600');
  await p.keyboard.press('Escape'); await p.waitForTimeout(200);
  await p.evaluate(() => { const m = document.getElementById('settingsModal'); if (m.classList.contains('open')) document.getElementById('settingsClose').click(); });
  await p.context().close();
  // (2) busy overlay vs the map controls
  const q = await L.boot(b, L.V272, { width: 1600, height: 900 });
  await L.genByClicks(q, 12345, '1024', null);
  await q.keyboard.press('Control+r');
  for (let i = 0; i < 30; i++) { await q.waitForTimeout(100); const on = await q.evaluate(() => getComputedStyle(document.getElementById('busy')).display !== 'none'); if (on) break; }
  out.busy = await q.evaluate(() => { const hit = sel => { const e = document.querySelector(sel); if (!e) return null; const r = e.getBoundingClientRect(); const t = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2); return t ? (t.closest('#busy') ? 'busy overlay' : (t.id || t.className || t.tagName)) : null; }; return { busyZ: getComputedStyle(document.getElementById('busy')).zIndex, mapBrZ: getComputedStyle(document.getElementById('mapBR')).zIndex, zoomIn: hit('#zoomIn'), viewDim3d: hit('#viewDimSeg button:not(.on)'), baseMap: hit('#layersBtn'), scaleBar: hit('#scaleBar'), legend: hit('#legend') }; });
  await clipShot(q, { x: 440, y: 76, width: 834, height: 764 }, 'h_busy_map_controls');
  await L.idle(q, 300);
  await q.context().close();
  // (3) phone: the faction rows of the Settlements screen
  const ph = await L.boot(b, L.V272, { width: 412, height: 915 }, { mobile: true });
  await ph.tap('#obSkip'); await ph.waitForTimeout(500);
  await ph.tap('#settlementsHeaderBtn'); await ph.waitForTimeout(800);
  out.phoneFactionRows = await ph.evaluate(() => { const rows = [...document.querySelectorAll('#civFactionsModal [class*="row"], #civFactionsModal li, #civFactionsModal button')].filter(e => /Monarchy/.test(e.textContent || '') && e.getBoundingClientRect().height < 80); const r = rows[0]; if (!r) return null; const kids = [...r.querySelectorAll('*')].map(k => ({ t: (k.childElementCount === 0 ? (k.textContent || '').trim().slice(0, 24) : ''), w: Math.round(k.getBoundingClientRect().width), sw: k.scrollWidth })).filter(k => k.t); return { rowW: Math.round(r.getBoundingClientRect().width), html: r.outerHTML.slice(0, 400), kids }; });
  await shot(ph, 'h_phone_faction_rows');
  save('h_measures', out);
  log(JSON.stringify(out).slice(0, 2500));
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
