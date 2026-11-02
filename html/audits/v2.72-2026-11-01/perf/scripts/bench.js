// Performance bench for the Cartalith DCC-line HTML (read-only: it loads a COPY of the file in headless
// Chromium and only measures). One sample per process; the driver interleaves files (ABBA).
// Usage: node bench.js <mode> <file.html> [res]
//   mode load     : navigation -> DCL/load/last long task; CDP metrics; DOM size
//   mode gen      : generate() at 512 (cold), 512, 1024, 2048 — wall ms, PERF stages, longest long task
//   mode interact : one world at <res> (default 1024) + settlements, then render-after-view-change,
//                   per-event pan/zoom handler cost, real-input pan/zoom smoothness, shell costs,
//                   event-listener growth
const path = require('path');
const { chromium } = require('playwright');

const mode = process.argv[2], file = process.argv[3], RES = +(process.argv[4] || 1024);
const VIEW = { width: 1400, height: 900 };
const med = a => { const s = [...a].filter(x => x != null && !Number.isNaN(x)).sort((x, y) => x - y); return s.length ? s[s.length >> 1] : null; };

const INIT = `
  window.__lt = []; window.__paint = {};
  try { new PerformanceObserver(l => { for (const e of l.getEntries()) window.__lt.push([e.startTime, e.duration]); })
        .observe({ type: 'longtask', buffered: true }); } catch (e) { window.__ltErr = String(e); }
  try { new PerformanceObserver(l => { for (const e of l.getEntries()) window.__paint[e.name] = e.startTime; })
        .observe({ type: 'paint', buffered: true }); } catch (e) {}
`;

async function cdpMetrics(cdp) {
  const { metrics } = await cdp.send('Performance.getMetrics');
  const m = {}; for (const { name, value } of metrics) m[name] = value; return m;
}
const KEYS = ['LayoutCount', 'RecalcStyleCount', 'LayoutDuration', 'RecalcStyleDuration', 'ScriptDuration', 'TaskDuration', 'JSHeapUsedSize', 'Nodes', 'JSEventListeners'];
const pick = m => Object.fromEntries(KEYS.map(k => [k, m[k]]));
const delta = (a, b) => Object.fromEntries(['LayoutCount', 'RecalcStyleCount', 'LayoutDuration', 'RecalcStyleDuration', 'ScriptDuration', 'TaskDuration', 'JSEventListeners', 'Nodes'].map(k => [k, +(b[k] - a[k]).toFixed(4)]));

async function boot() {
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: VIEW });
  const page = await ctx.newPage();
  const errs = []; page.on('pageerror', e => errs.push(String(e).slice(0, 240)));
  page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.addInitScript(INIT);
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Performance.enable');
  return { browser, ctx, page, cdp, errs };
}

async function modeLoad() {
  const { browser, page, cdp, errs } = await boot();
  const t0 = Date.now();
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  const tGotoLoad = Date.now() - t0;
  await page.waitForTimeout(3000);   // let post-load work (pool warm-up, deferred timers) land
  const m = await cdpMetrics(cdp);
  const r = await page.evaluate(() => {
    const nav = performance.getEntriesByType('navigation')[0] || {};
    const lt = window.__lt || [];
    const lastEnd = lt.length ? Math.max(...lt.map(([s, d]) => s + d)) : 0;
    let all = 0; const tw = document.createTreeWalker(document, NodeFilter.SHOW_ALL); while (tw.nextNode()) all++;
    let rules = 0; for (const ss of document.styleSheets) { try { rules += ss.cssRules.length; } catch (e) {} }
    return {
      domInteractive: nav.domInteractive, dcl: nav.domContentLoadedEventEnd, loadEnd: nav.loadEventEnd,
      fp: window.__paint['first-paint'], fcp: window.__paint['first-contentful-paint'],
      longTasks: lt.length, longTaskTotal: lt.reduce((s, [, d]) => s + d, 0), longTaskMax: lt.length ? Math.max(...lt.map(([, d]) => d)) : 0,
      lastLongTaskEnd: lastEnd, tti: Math.max(nav.domContentLoadedEventEnd || 0, lastEnd),
      elements: document.getElementsByTagName('*').length, allNodes: all, cssRules: rules,
      gateShown: (() => { const o = document.getElementById('onboard'); return !!(o && o.style.display !== 'none' && o.getClientRects().length); })(),
      ltErr: window.__ltErr || null,
    };
  });
  await browser.close();
  return { tGotoLoad, ...r, cdp: pick(m), errs };
}

async function genAt(page, cdp, res, seed) {
  const a = await cdpMetrics(cdp);
  const r = await page.evaluate(async ({ res, seed }) => {
    const lt0 = window.__lt.length;
    /* run inside an ordinary event-loop task (setTimeout), not the DevTools evaluation itself: work
       done directly inside Runtime.callFunctionOn is invisible to the longtask observer and to
       CDP ScriptDuration (measured: 0 long tasks and ~1 ms ScriptDuration for a 4.4 s generate) */
    const ms = await new Promise(resolve => setTimeout(async () => {
      state.tect.seed = seed; state.resW = res;
      if (typeof _setupSkipped !== 'undefined') _setupSkipped = true;
      GW = state.resW; GH = gridH(GW); allocate();
      const t = performance.now(); await generate(); resolve(performance.now() - t);
    }, 0));
    await new Promise(r => setTimeout(r, 50));
    const lt = window.__lt.slice(lt0);
    return { ms, genTotal: PERF.genTotal, stages: { ...PERF.gen }, render: { ...PERF.render },
             longTasks: lt.length, longTaskMax: lt.length ? Math.max(...lt.map(([, d]) => d)) : 0, longTaskTotal: lt.reduce((s, [, d]) => s + d, 0) };
  }, { res, seed });
  const b = await cdpMetrics(cdp);
  r.cdp = delta(a, b); r.heapAfterMB = +(b.JSHeapUsedSize / 1048576).toFixed(1);
  return r;
}

async function modeGen() {
  const { browser, page, cdp, errs } = await boot();
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(1000);
  const out = { env: await page.evaluate(() => ({ gpu: GPU.ok ? (GPU.enabled ? (GPU._r32f ? 'R32F' : 'RGBA32F') : 'off') : 'no-webgl2', pool: GENPOOL._tagStr || String(GENPOOL.usable) })) };
  const list = (process.env.GEN_RES || '512c,512,1024,2048').split(',');
  for (const k of list) out['g' + k] = await genAt(page, cdp, parseInt(k), 12345);
  await browser.close();
  out.errs = errs; return out;
}

/* ---------- interact ---------- */
async function frames(page) { await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))); }

async function modeInteract() {
  const { browser, page, cdp, errs } = await boot();
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  const out = { res: RES };
  out.world = await page.evaluate(async (R) => {
    state.tect.seed = 12345; state.resW = R; if (typeof _setupSkipped !== 'undefined') _setupSkipped = true;
    GW = R; GH = gridH(GW); allocate();
    let t = performance.now(); await generate(); const gen = performance.now() - t;
    if (typeof _setupHide === 'function') _setupHide();
    t = performance.now(); try { _civIterativeAutoWorld(3); } catch (e) { return { err: String(e) }; } const civ = performance.now() - t;
    t = performance.now(); renderNow(); const rn = performance.now() - t;
    return { gen, civ, renderNowAfterCiv: rn, places: state.places.length, ways: (typeof civWays !== 'undefined') ? civWays.length : -1 };
  }, RES);
  await page.waitForTimeout(1000);
  out.dom = await page.evaluate(() => ({ elements: document.getElementsByTagName('*').length }));

  /* 1. render after a view change: cold (bake cache cleared) and warm (cache hit) */
  out.viewChange = await page.evaluate(async () => {
    const raf2 = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const res = [];
    const btn = m => document.querySelector('#modeSeg [data-mode="' + m + '"]');
    for (const [m, cold] of [['hypso', 1], ['gray', 1], ['shade', 1], ['biome', 1], ['hypso', 0], ['biome', 0]]) {
      if (cold) _civBakeCache.clear();
      await raf2();
      const t0 = performance.now(); btn(m).click(); const sync = performance.now() - t0;
      await raf2(); const frame = performance.now() - t0;
      res.push({ m, cold: !!cold, sync, frame, render: cold ? { ...PERF.render } : null });
    }
    return res;
  });

  /* 2. per-event handler cost, synthetic dispatch, off-LOD (lodAuto off so zoom-in stays on the CSS path) */
  const perEvent = async (withFrames) => page.evaluate(async (withFrames) => {
    const raf = () => new Promise(r => requestAnimationFrame(r));
    const wrap = document.querySelector('.canvas-wrap'), view = document.getElementById('view');
    const wr = wrap.getBoundingClientRect(); const cx = wr.left + wr.width / 2, cy = wr.top + wr.height / 2;
    const saveAuto = state.lodAuto; state.lodAuto = false;
    if (typeof _lodOn !== 'undefined' && _lodOn) return { err: 'in LOD' };
    _viewFill(); await raf(); await raf();
    // count layout-reading calls per event
    let gbcr = 0; const og = Element.prototype.getBoundingClientRect; Element.prototype.getBoundingClientRect = function () { gbcr++; return og.apply(this, arguments); };
    const cw0 = civCanvas.width;
    const wheel = [], wheelG = [], resizes = [];
    for (let i = 0; i < 24; i++) {
      const dy = i < 12 ? -100 : 100;
      gbcr = 0; const w0 = civCanvas.width;
      const t0 = performance.now();
      wrap.dispatchEvent(new WheelEvent('wheel', { clientX: cx, clientY: cy, deltaY: dy, bubbles: true, cancelable: true }));
      wheel.push(performance.now() - t0); wheelG.push(gbcr); resizes.push(civCanvas.width !== w0 ? civCanvas.width : 0);
      if (withFrames) await raf();
    }
    const scaleAfterIn = viewT.scale;
    // pan: zoom to ~1.8x about the centre first, then drive the pan branch with panDrag set
    _viewFill(); zoomAt(cx, cy, 1.8); await raf(); await raf();
    panDrag = { x: cx, y: cy, px: viewT.panX, py: viewT.panY };
    const pan = [], panG = [];
    for (let i = 0; i < 30; i++) {
      gbcr = 0; const t0 = performance.now();
      view.dispatchEvent(new PointerEvent('pointermove', { clientX: cx + Math.sin(i / 4) * 120, clientY: cy + Math.cos(i / 5) * 60, bubbles: true, pointerId: 1 }));
      pan.push(performance.now() - t0); panG.push(gbcr);
      if (withFrames) await raf();
    }
    panDrag = null;
    Element.prototype.getBoundingClientRect = og;
    state.lodAuto = saveAuto; _viewFill(); await raf();
    const med = a => { const s = [...a].sort((x, y) => x - y); return s[s.length >> 1]; };
    return { wheelMs: wheel, wheelMedian: med(wheel), wheelMax: Math.max(...wheel), gbcrPerWheel: med(wheelG), civCanvasResizes: resizes.filter(Boolean).length, civCanvasMaxW: Math.max(cw0, ...resizes),
             panMs: pan, panMedian: med(pan), panMax: Math.max(...pan), gbcrPerPan: med(panG), scaleAfterIn, civW: civCanvas.width, civH: civCanvas.height };
  }, withFrames);
  { const a = await cdpMetrics(cdp); out.perEventFramed = await perEvent(true); const b = await cdpMetrics(cdp); out.perEventFramed.cdp = delta(a, b); }
  { const a = await cdpMetrics(cdp); out.perEventB2B = await perEvent(false); const b = await cdpMetrics(cdp); out.perEventB2B.cdp = delta(a, b); }

  /* 3. real input through CDP: rAF intervals + long tasks while wheeling and middle-drag panning */
  const smooth = async (kind, lodAuto) => {
    await page.evaluate((lodAuto) => { state.lodAuto = lodAuto; if (typeof _lodOn !== 'undefined' && _lodOn) { const lc = document.getElementById('lodChk'); if (lc && lc.checked) lc.click(); } _viewFill(); }, lodAuto);
    await frames(page);
    const wr = await page.evaluate(() => { const r = document.querySelector('.canvas-wrap').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; });
    if (kind === 'pan') { await page.evaluate(({ x, y }) => { zoomAt(x, y, 1.8); }, wr); await frames(page); }
    await page.evaluate(() => {
      window.__fr = []; window.__lt0 = window.__lt.length; window.__run = true;
      const loop = t => { window.__fr.push(t); if (window.__run) requestAnimationFrame(loop); }; requestAnimationFrame(loop);
    });
    const a = await cdpMetrics(cdp);
    const T0 = Date.now();
    await page.mouse.move(wr.x, wr.y);
    if (kind === 'wheel') {
      for (let i = 0; i < 40; i++) { await page.mouse.wheel(0, i < 20 ? -100 : 100); await new Promise(r => setTimeout(r, 16)); }
    } else {
      await page.mouse.down({ button: 'middle' });
      for (let i = 0; i < 60; i++) { await page.mouse.move(wr.x + Math.sin(i / 6) * 200, wr.y + Math.cos(i / 7) * 100); await new Promise(r => setTimeout(r, 16)); }
      await page.mouse.up({ button: 'middle' });
    }
    const wall = Date.now() - T0;
    await page.waitForTimeout(300);
    const b = await cdpMetrics(cdp);
    const r = await page.evaluate(() => {
      window.__run = false;
      const f = window.__fr, iv = []; for (let i = 1; i < f.length; i++) iv.push(f[i] - f[i - 1]);
      const s = [...iv].sort((x, y) => x - y), q = p => s.length ? s[Math.min(s.length - 1, Math.floor(p * s.length))] : null;
      const lt = window.__lt.slice(window.__lt0);
      return { frames: f.length, ivMedian: q(0.5), ivP95: q(0.95), ivMax: s.length ? s[s.length - 1] : null, over33: iv.filter(x => x > 33.4).length, over50: iv.filter(x => x > 50).length,
               longTasks: lt.length, longTaskTotal: lt.reduce((x, [, d]) => x + d, 0), longTaskMax: lt.length ? Math.max(...lt.map(([, d]) => d)) : 0,
               lodOn: (typeof _lodOn !== 'undefined') ? _lodOn : null, scale: viewT.scale };
    });
    r.wallMs = wall; r.cdp = delta(a, b);
    await page.evaluate(() => { if (typeof _lodOn !== 'undefined' && _lodOn) { const lc = document.getElementById('lodChk'); if (lc && lc.checked) lc.click(); } state.lodAuto = true; _viewFill(); });
    await frames(page);
    return r;
  };
  out.smoothWheelCss = await smooth('wheel', false);
  out.smoothPan = await smooth('pan', false);
  if (process.env.LOD_WHEEL) out.smoothWheelDefault = await smooth('wheel', true);

  /* 4. shell costs */
  out.shell = await page.evaluate(async () => {
    const raf2 = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const med = a => { const s = [...a].sort((x, y) => x - y); return s[s.length >> 1]; };
    const R = {};
    // File menu: exists in both (v2.71 .menubtn with its own handler; v2.72 the shared .mb-btn controller)
    const fm = document.getElementById('fileMenuBtn');
    /* "sync" = the click handler alone; "+layout" = handler plus the style recalc and layout it
       leaves pending, forced at once by reading offsetHeight (paint and composite not included) */
    const fileOpen = [], fileFrame = [], fileLay = [];
    for (let i = 0; i < 7; i++) {
      await raf2();
      const t0 = performance.now(); fm.click(); fileOpen.push(performance.now() - t0); void document.body.offsetHeight; fileLay.push(performance.now() - t0); await raf2(); fileFrame.push(performance.now() - t0);
      document.body.click(); await raf2();
    }
    R.fileOpenMs = med(fileOpen); R.fileOpenLayoutMs = med(fileLay); R.fileOpenFrameMs = med(fileFrame);
    // v2.72's eight menus
    const mbs = [...document.querySelectorAll('.mb-btn')].filter(b => b.getClientRects().length && b.id !== 'fileMenuBtn');
    if (mbs.length) {
      const per = {}, perL = {};
      for (const b of mbs) {
        const o = [], l = [];
        for (let i = 0; i < 5; i++) { await raf2(); const t0 = performance.now(); b.click(); o.push(performance.now() - t0); void document.body.offsetHeight; l.push(performance.now() - t0); await raf2(); document.body.click(); await raf2(); }
        per[b.id] = +med(o).toFixed(3); perL[b.id] = +med(l).toFixed(3);
      }
      R.menuOpenMs = per; R.menuOpenLayoutMs = perL;
    }
    // refit: _refitViewportNow exists since v2.24 in both
    if (typeof _refitViewportNow === 'function') { const o = []; for (let i = 0; i < 9; i++) { const t0 = performance.now(); _refitViewportNow(); o.push(performance.now() - t0); await raf2(); } R.refitMs = med(o); }
    // a band change that resizes .canvas-wrap (status bar hidden then shown) -> ResizeObserver -> rAF refit
    { const sb = document.querySelector('.status-bar'); const l = [], f = [], rf = [];
      let refits = 0; const orig = window._refitViewportNow; window._refitViewportNow = function () { refits++; const t = performance.now(); const r = orig.apply(this, arguments); rf.push(performance.now() - t); return r; };
      for (let i = 0; i < 6; i++) { await raf2(); const t0 = performance.now(); sb.style.display = (i % 2 ? '' : 'none'); void document.body.offsetHeight; l.push(performance.now() - t0); await raf2(); await raf2(); f.push(performance.now() - t0); }
      window._refitViewportNow = orig; sb.style.display = ''; await raf2(); await raf2();
      R.bandChangeLayoutMs = med(l); R.bandChangeFrameMs = med(f); R.bandChangeRefits = refits; R.bandChangeRefitMs = med(rf); }
    // _dskToolOptsSync (v2.72): no tool armed, and with Sculpt armed (whose rAF callback reads layout)
    if (typeof _dskToolOptsSync === 'function') {
      const o = []; for (let i = 0; i < 101; i++) { const t0 = performance.now(); _dskToolOptsSync(); o.push(performance.now() - t0); }
      R.toolOptsSyncMs = med(o);
      _setSculptCategory(true); await raf2(); await raf2();
      const s = [], cb = [];
      const oRaf = window.requestAnimationFrame;
      for (let i = 0; i < 21; i++) {
        await raf2();
        window.requestAnimationFrame = fn => oRaf.call(window, ts => { const t = performance.now(); fn(ts); cb.push(performance.now() - t); });
        const t0 = performance.now(); _dskToolOptsSync(); s.push(performance.now() - t0);
        window.requestAnimationFrame = oRaf;
        await raf2();
      }
      R.toolOptsSyncSculptMs = med(s); R.toolOptsSyncSculptRafCbMs = med(cb); R.toolOptsSyncSculptRafCbN = cb.length;
      _setSculptCategory(false); await raf2();
    }
    // the same domain switch in both: World -> Civil -> Carto -> Explore -> World via the rail
    { const o = [], s = [], l = []; for (const d of ['civ', 'carto', 'explore', 'world', 'civ', 'carto', 'explore', 'world']) { const b = document.querySelector('#domainRail [data-domain="' + d + '"]'); if (!b) continue; await raf2(); const t0 = performance.now(); b.click(); s.push(performance.now() - t0); void document.body.offsetHeight; l.push(performance.now() - t0); await raf2(); o.push(performance.now() - t0); }
      R.domainSwitchMs = med(s); R.domainSwitchLayoutMs = med(l); R.domainSwitchFrameMs = med(o); }
    return R;
  });

  /* 5. resize across the 860px line (v2.72 runs _dskPlace from its matchMedia listener) */
  {
    await page.evaluate(() => { window.__placeMs = []; if (typeof _dskPlace === 'function') { const o = window._dskPlace; window._dskPlace = function () { const t = performance.now(); const r = o.apply(this, arguments); window.__placeMs.push(performance.now() - t); return r; }; } });
    const a = await cdpMetrics(cdp);
    const times = [];
    for (const w of [800, 1400, 800, 1400, 800, 1400]) {
      await page.evaluate(() => { window.__rz = null; window.addEventListener('resize', () => { if (window.__rz == null) window.__rz = performance.now(); }, { once: true }); });
      await page.setViewportSize({ width: w, height: 900 });
      const t = await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(() => r(performance.now() - (window.__rz || performance.now()))))));
      times.push(t);
      await page.waitForTimeout(150);
    }
    const b = await cdpMetrics(cdp);
    out.resize = { toFrameMs: times, medianToFrame: med(times), placeMs: await page.evaluate(() => window.__placeMs), cdp: delta(a, b) };
  }

  /* 6. listener growth: repeat ordinary operations, compare JSEventListeners and Nodes */
  {
    await frames(page);
    const a = await cdpMetrics(cdp);
    await page.evaluate(async () => {
      const raf = () => new Promise(r => requestAnimationFrame(r));
      for (let k = 0; k < 10; k++) {
        for (const b of document.querySelectorAll('.mb-btn,#fileMenuBtn')) { if (!b.getClientRects().length) continue; b.click(); await raf(); document.body.click(); }
        for (const d of ['civ', 'carto', 'explore', 'world']) { const b = document.querySelector('#domainRail [data-domain="' + d + '"]'); if (b) b.click(); }
        const p = state.places.find(x => x && x.category === 'settlement');
        if (p && typeof _civSelectPlaceAt === 'function') { _civSelectedPlace = p; if (typeof _civRenderInspector === 'function') _civRenderInspector(); _civSelectedPlace = null; if (typeof _civRenderInspector === 'function') _civRenderInspector(); }
        await raf();
      }
    });
    if (typeof global.gc === 'function') global.gc();
    await cdp.send('HeapProfiler.enable'); await cdp.send('HeapProfiler.collectGarbage'); await page.waitForTimeout(200);
    const b = await cdpMetrics(cdp);
    out.listenerGrowth = { before: a.JSEventListeners, after: b.JSEventListeners, nodesBefore: a.Nodes, nodesAfter: b.Nodes };
  }
  out.errs = errs;
  await browser.close();
  return out;
}

(async () => {
  const load = await require('fs').promises.readFile('/proc/loadavg', 'utf8');
  const t = Date.now();
  let r;
  try {
    r = mode === 'load' ? await modeLoad() : mode === 'gen' ? await modeGen() : await modeInteract();
  } catch (e) { r = { fatal: String(e && e.stack || e).slice(0, 800) }; }
  console.log(JSON.stringify({ mode, file: path.basename(file), res: RES, loadavgStart: load.trim(), wallS: (Date.now() - t) / 1000, ...r }));
  process.exit(0);
})();
