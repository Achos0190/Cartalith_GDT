// Phase G: which findings are new in v2.72 and which v2.71 already had. Same boot world in both files
// (Skip for now, no generate), 1600x900 dark: the Assets toggle's colours in Assets mode, the status readout's
// overflow, the active sub-tab's border, and the File menu's Export GeoJSON button.
const L = require('./lib.js');
const { log, save, clipShot } = L;
(async () => {
  const b = await L.chromium.launch();
  const res = {};
  for (const [tag, file] of [['v271', L.V271], ['v272', L.V272]]) {
    const p = await L.boot(b, file, { width: 1600, height: 900 });
    const r = {};
    await p.click('#obSkip'); await p.waitForTimeout(600);
    r.readout = await p.evaluate(() => { const e = document.getElementById('readout'); return { client: e.clientWidth, scroll: e.scrollWidth }; });
    r.subtab = await p.evaluate(() => { const e = document.querySelector('.subtab.on'); if (!e) return null; const cs = getComputedStyle(e); return { top: cs.borderTopWidth + ' ' + cs.borderTopColor, left: cs.borderLeftWidth, bottom: cs.borderBottomWidth, fw: cs.fontWeight, text: e.textContent.trim() }; });
    await p.click('#assetsHeaderBtn'); await p.waitForTimeout(1000); await p.mouse.move(800, 600); await p.waitForTimeout(200);
    r.assetsPin = await p.evaluate(() => { const e = document.getElementById('assetsHeaderBtn'); const cs = getComputedStyle(e); const q = e.getBoundingClientRect(); return { bg: cs.backgroundColor, color: cs.color, text: e.textContent.trim(), rect: [q.left, q.top, q.width, q.height].map(Math.round) }; });
    const q = r.assetsPin.rect; await clipShot(p, { x: Math.max(0, q[0] - 20), y: Math.max(0, q[1] - 6), width: q[2] + 40, height: q[3] + 12 }, `g_${tag}_assets_toggle_rest`);
    await p.click('#assetsHeaderBtn'); await p.waitForTimeout(600);
    await p.click('#fileMenuBtn'); await p.waitForTimeout(300);
    if (tag === 'v272') { await p.hover('#mbExportRow'); await p.waitForTimeout(300); }
    r.geojson = await p.evaluate(() => { const e = document.getElementById('exportGeoBtn'); e.scrollIntoView({ block: 'center' }); const cs = getComputedStyle(e); return { bg: cs.backgroundColor, border: cs.borderTopWidth + ' ' + cs.borderTopStyle }; });
    res[tag] = r; log(tag, JSON.stringify(r));
    await p.context().close();
  }
  save('g_v271_vs_v272', res);
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
