// Split one off-LOD wheel step into its parts, measured separately (median of 15 each), at <res> with
// settlements: the civ canvas reallocation alone, drawCivLayer at zoom 1 and at the 4096 cap, the
// scale bar, _viewClampFill, a whole zoomAt; plus getImageData/putImageData of the full map (the
// bake cache's read-back and hit paths) and maxGrade/updateReadout (run on every full render).
// Usage: node split_event.js <file.html> <res>
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
    _civIterativeAutoWorld(3); renderNow(); state.lodAuto = false; _viewFill();
    const raf = () => new Promise(r => requestAnimationFrame(r));
    await raf(); await raf();
    const med = a => { const s = [...a].sort((x, y) => x - y); return +s[s.length >> 1].toFixed(2); };
    const T = (fn, n = 15) => { const o = []; for (let i = 0; i < n; i++) { const t = performance.now(); fn(i); o.push(performance.now() - t); } return med(o); };
    const out = { places: state.places.length, ways: civWays.length, territory: !!civTerritory, civW0: civCanvas.width, civH0: civCanvas.height, cssW: civCanvas.clientWidth, cssH: civCanvas.clientHeight };
    const w0 = civCanvas.width, h0 = civCanvas.height;
    // the reallocation alone: alternate two sizes 4 px apart (the threshold is >2 px)
    out.civResizeAtZ1 = T(i => { civCanvas.width = w0 + (i % 2 ? 4 : 0); civCanvas.height = h0; });
    out.civResizeAtCap = T(i => { civCanvas.width = 4096 - (i % 2 ? 4 : 0); civCanvas.height = Math.round(4096 * GH / GW); });
    civCanvas.width = w0; civCanvas.height = h0;
    out.drawCivAtZ1 = T(() => drawCivLayer());
    // at the cap: set viewT.scale so _civSyncCanvas picks 4096 (it reads viewT.scale), then draw
    const s0 = viewT.scale; viewT.scale = Math.max(2.2, 4096 / Math.max(1, civCanvas.clientWidth)); drawCivLayer();
    out.civAtCap = [civCanvas.width, civCanvas.height]; out.drawCivAtCap = T(() => drawCivLayer());
    viewT.scale = s0; _viewFill(); await raf();
    out.updateScaleBar = T(() => updateScaleBar());
    out.viewClampFill = T(() => _viewClampFill());
    // a whole wheel step at fixed size (no realloc): zoomAt in then out by the same factor
    out.zoomAtPair = T(i => { const wr = document.querySelector('.canvas-wrap').getBoundingClientRect(); zoomAt(wr.left + wr.width / 2, wr.top + wr.height / 2, i % 2 ? 1 / 1.16 : 1.16); });
    _viewFill(); await raf();
    out.getImageDataFull = T(() => vctx.getImageData(0, 0, GW, GH), 9);
    const img = vctx.getImageData(0, 0, GW, GH);
    out.putImageDataFull = T(() => vctx.putImageData(img, 0, 0), 9);
    out.maxGrade = T(() => maxGrade(), 9);
    out.updateReadout = T(() => updateReadout(), 9);
    out.renderNowHit = T(() => renderNow(), 9);   // bake-cache HIT: putImageData + drawCivLayer + overlays
    // one territory-brush step (CIVIL > Territory): what each pointermove of a paint drag runs
    if (typeof _civPaintTerritoryAt === 'function') {
      const f0 = _civActiveFaction; _civActiveFaction = 1;
      out.territoryPaintStep = T(i => _civPaintTerritoryAt(Math.round(GW * (0.3 + 0.02 * i)), Math.round(GH * 0.5)), 15);
      _civActiveFaction = f0;
    }
    return out;
  }, +res);
  console.log(JSON.stringify({ file: path.basename(file), res: +res, ...r }));
  await b.close();
})();
