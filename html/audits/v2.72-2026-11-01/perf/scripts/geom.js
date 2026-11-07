// Map viewport geometry per version at a window size: the .canvas-wrap box (what every map-sized cost
// scales with: the civ canvas, the LOD composite, visible LOD tiles), plus the LOD view's visible tile
// count after entering LOD at 2.2x about the centre. 512 world, seed 12345, gate closed.
// Usage: node geom.js <file.html> [w] [h]
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, w, h] = process.argv.slice(2);
  const b = await chromium.launch();
  const page = await b.newPage({ viewport: { width: +(w || 1400), height: +(h || 900) } }); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  const r = await page.evaluate(async () => {
    state.tect.seed = 12345; state.resW = 512; _setupSkipped = true; GW = 512; GH = gridH(GW); allocate(); await generate(); _setupHide();
    const raf = () => new Promise(r => requestAnimationFrame(r)); await raf(); await raf();
    const cw = document.querySelector('.canvas-wrap').getBoundingClientRect();
    _viewFill(); await raf();
    const coverScale = viewT.scale;
    enterLodFromView(cw.left + cw.width / 2, cw.top + cw.height / 2, 2.2); await raf(); await raf();
    const v = lodViewRect(); let tiles = 0; for (const k of visibleTileKeys(v.z, v.x0, v.y0, v.x1, v.y1)) tiles++;
    const lodCanvas = [view.width, view.height], lodCss = [view.style.width, view.style.height];
    return { wrap: { w: Math.round(cw.width), h: Math.round(cw.height), area: Math.round(cw.width * cw.height) }, coverScale: +coverScale.toFixed(3), lodTilesVisible: tiles, lodZ: v.z, lodCanvas, lodCss };
  });
  console.log(JSON.stringify({ file: path.basename(file), window: [+(w || 1400), +(h || 900)], ...r }));
  await b.close();
})();
