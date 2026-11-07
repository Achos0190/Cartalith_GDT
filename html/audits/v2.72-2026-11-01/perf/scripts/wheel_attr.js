// Attribute the off-LOD wheel step's cost by switching parts OFF (measurement-only monkeypatches in
// the page; the file is untouched). 24 synthetic wheel ticks (12 in, 12 out) at the map centre, a frame
// between ticks, per variant, variants interleaved over 3 rounds:
//   full      : as shipped
//   noCiv     : drawCivLayerAuto() is a no-op (no civ redraw, no civ canvas resize)
//   civFixed  : the civ canvas keeps its zoom-1 size (_civSyncCanvas ignores the zoom), still redrawn
//   noScale   : updateScaleBar() is a no-op (no layout read, no innerHTML)
// Also: civ canvas resize+draw at the 4096 cap, alone.
// Usage: node wheel_attr.js <file.html> <res>
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, res] = process.argv.slice(2);
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage(); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  const r = await page.evaluate(async (R) => {
    state.tect.seed = 12345; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate(); await generate(); _setupHide();
    _civIterativeAutoWorld(3); renderNow(); state.lodAuto = false;
    const raf = () => new Promise(r => requestAnimationFrame(r));
    const wrap = document.querySelector('.canvas-wrap'); const wr = wrap.getBoundingClientRect(); const cx = wr.left + wr.width / 2, cy = wr.top + wr.height / 2;
    const orig = { dcl: window.drawCivLayerAuto, sync: window._civSyncCanvas, usb: window.updateScaleBar };
    const fixedSync = function () { const dpr = window.devicePixelRatio || 1; const tw = Math.max(GW, Math.round((civCanvas.clientWidth || GW) * dpr)), th = Math.max(GH, Math.round((civCanvas.clientHeight || GH) * dpr)); if (Math.abs(civCanvas.width - tw) > 2 || Math.abs(civCanvas.height - th) > 2) { civCanvas.width = tw; civCanvas.height = th; } };
    const set = v => { window.drawCivLayerAuto = v === 'noCiv' ? function () {} : orig.dcl; window._civSyncCanvas = v === 'civFixed' ? fixedSync : orig.sync; window.updateScaleBar = v === 'noScale' ? function () {} : orig.usb; };
    const run = async v => {
      set(v); _viewFill(); await raf(); await raf();
      const o = [];
      for (let i = 0; i < 24; i++) { const t0 = performance.now(); wrap.dispatchEvent(new WheelEvent('wheel', { clientX: cx, clientY: cy, deltaY: i < 12 ? -100 : 100, bubbles: true, cancelable: true })); o.push(performance.now() - t0); await raf(); }
      set('full'); _viewFill(); await raf();
      return o;
    };
    const V = ['full', 'noCiv', 'civFixed', 'noScale'], acc = Object.fromEntries(V.map(v => [v, []]));
    for (let k = 0; k < 3; k++) for (const v of (k % 2 ? [...V].reverse() : V)) acc[v].push(...(await run(v)));
    const med = a => { const s = [...a].sort((x, y) => x - y); return +s[s.length >> 1].toFixed(1); };
    const p90 = a => { const s = [...a].sort((x, y) => x - y); return +s[Math.floor(0.9 * (s.length - 1))].toFixed(1); };
    const out = {}; for (const v of V) out[v] = { median: med(acc[v]), p90: p90(acc[v]), max: +Math.max(...acc[v]).toFixed(1), sum: +acc[v].reduce((x, y) => x + y, 0).toFixed(0) };
    // resize + draw at the cap, alone (alternating two sizes 4 px apart, each followed by a draw)
    const s0 = viewT.scale; const capH = Math.round(4096 * GH / GW); const rd = [];
    for (let i = 0; i < 9; i++) { const t0 = performance.now(); civCanvas.width = 4096 - (i % 2 ? 4 : 0); civCanvas.height = capH; drawCivLayer(); rd.push(performance.now() - t0); await raf(); }
    // the same draw without the resize, at the cap
    const dr = []; for (let i = 0; i < 9; i++) { const t0 = performance.now(); civCtx.setTransform(1, 0, 0, 1, 0, 0); civCtx.clearRect(0, 0, civCanvas.width, civCanvas.height); drawCivLayer(); dr.push(performance.now() - t0); await raf(); }
    viewT.scale = s0; _viewFill();
    out.resizePlusDrawAtCap = med(rd); out.drawAtCapNoResize = med(dr);
    return out;
  }, +res);
  console.log(JSON.stringify({ file: path.basename(file), res: +res, ...r }));
  await b.close();
})();
