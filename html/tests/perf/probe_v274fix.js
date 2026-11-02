/* v2.74 — the fix pass (html/audits/v2.72-2026-11-01/PLAN.md §5) and the planner's toll fix (Ruling EC item 6),
 * each finding checked at its bar. Owner: "can you check and improve the visuals and performance of the html?",
 * then "Fixes and feature ports first then stage 3" (Ruling EF).
 *
 * One section per finding id (P7, V01, V10, V04, V05, V06, V23, V11, the Assets panel, V16, V02/V08/V09, V12,
 * V03, V19, V20, V21, V07, V13, V14, V15, V17, V18, V22, V25, V28, P4, V272-1/-2, F06). Every assertion is meant
 * to FAIL on v2.73 unless it says otherwise in its own text ("…, as before"): run the probe on v2.73 too and
 * read the count it prints (MISTAKES.md "Add a probe in the same commit that changes the behaviour").
 *
 * The shell is navigated by REAL input — page.mouse clicks, page.keyboard, page.touchscreen taps — and
 * page.evaluate is used for four things only, each marked where it happens:
 *   - SET-UP a person would take minutes over: a small world (generate at 256); settlements and polities
 *     (_civIterativeAutoWorld / _civAutoPolity, where a section is not about the Auto-populate button); a
 *     settlement placed as Unclaimed for the overview's second dash; the busy overlay raised with showBusy()
 *     (V11 is about what the overlay covers, not about a generate); the theme put in storage before a page
 *     loads (cartalith_theme, as a returning user's browser has it); a route's two clicks aimed at two places;
 *   - DISPATCHED input events for the bake-key sweep (P7), which changes every render-calling control once —
 *     the 'input'/'change' events those controls' handlers bind to (bind()/addEventListener), as RC's
 *     sun_probe.js did; the Sun slider is ALSO moved by a real mouse click on its track;
 *   - READS: computed styles, drawn rects, counters (PERF.counters.renderPixelLoop), hashes of #view, the
 *     in-page WCAG census (the visual audit's lib.js __aud.contrast, injected below);
 *   - RESETS between steps, after their assertions (a window closed, the busy overlay lowered).
 * Colour assertions pin LITERAL values and force the theme they were written for (MISTAKES.md "Assert a drawn
 * colour", "Assert on pixels"); every literal below is the token's value as v2.74 defines it.
 *
 * Usage: node tests/perf/probe_v274fix.js "Cartalith v2.74 DCC test.html"
 *   One file; any other argument is refused (exit 2). PROBE_ONLY=<regex> runs only the matching sections and
 *   exits 3 (not a verdict) even when nothing failed.
 * Env: PLAYWRIGHT_DIR, CHROME_BIN — as every probe in this folder.
 * Output: "ok   - …" / "FAIL - …" per assertion, then a count. Exit 1 if anything failed.
 */
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_DIR || 'playwright');

const FILE = process.argv[2];
if (!FILE || process.argv.length > 3 || !fs.existsSync(FILE)) {
  console.error('usage: node probe_v274fix.js <v2.74.html>  — got: ' + JSON.stringify(process.argv.slice(2)));
  process.exit(2);
}
let pass = 0, fail = 0, skipped = 0;
const A = (n, c, extra) => { if (c) { pass++; console.log('ok   - ' + n); }
  else { fail++; console.log('FAIL - ' + n + (extra !== undefined ? '  [' + extra + ']' : '')); } };
const errors = [];
const sleep = ms => new Promise(r => setTimeout(r, ms));
const ONLY = process.env.PROBE_ONLY ? new RegExp(process.env.PROBE_ONLY) : null;
const section = async (name, fn) => {
  if (ONLY && !ONLY.test(name)) { skipped++; return; }
  console.log('\n--- ' + name + ' ---');
  try { await fn(); } catch (e) { A(name + ': the section ran to the end', false, String((e && e.message) || e).split('\n')[0]); } };
const AND = 'Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Mobile Safari/537.36';
/* READ helpers injected before the file loads: the visual audit's WCAG census (lib.js, unchanged in its
   arithmetic) and the drawn-rect measure (probe_mobileshell.js's). They read; they never write. */
const AUD_SRC = `(${function () {
  const parse = s => { const m = s.match(/rgba?\(([^)]+)\)/); if (!m) return null; const v = m[1].split(/[ ,\/]+/).filter(Boolean).map(Number); return { r: v[0], g: v[1], b: v[2], a: v.length > 3 ? v[3] : 1 }; };
  const over = (f, b) => ({ r: f.r * f.a + b.r * (1 - f.a), g: f.g * f.a + b.g * (1 - f.a), b: f.b * f.a + b.b * (1 - f.a), a: 1 });
  const lum = c => { const t = x => { x /= 255; return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4); }; return 0.2126 * t(c.r) + 0.7152 * t(c.g) + 0.0722 * t(c.b); };
  const ratio = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const vis = el => { if (!el.isConnected) return false; const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; for (let e = el; e && e.nodeType === 1; e = e.parentElement) { const cs = getComputedStyle(e); if (cs.display === 'none' || cs.visibility === 'hidden' || +cs.opacity === 0) return false; } return r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth; };
  const bgOf = el => { const stack = []; for (let e = el; e && e.nodeType === 1; e = e.parentElement) { const c = parse(getComputedStyle(e).backgroundColor); if (c && c.a > 0) { stack.push(c); if (c.a >= 0.999) break; } }
    let base = parse(getComputedStyle(document.body).backgroundColor) || { r: 0, g: 0, b: 0, a: 1 }; for (let i = stack.length - 1; i >= 0; i--) base = over(stack[i], base); return base; };
  const opac = el => { let o = 1; for (let e = el; e && e.nodeType === 1; e = e.parentElement) o *= +getComputedStyle(e).opacity; return o; };
  const desc = el => { let s = el.tagName.toLowerCase(); if (el.id) s += '#' + el.id; else if (el.className && typeof el.className === 'string') s += '.' + el.className.trim().split(/\s+/).slice(0, 2).join('.'); const pid = el.closest('[id]'); if (pid && pid !== el) s = pid.tagName.toLowerCase() + '#' + pid.id + ' ' + s; return s; };
  /* one element's own text against what is behind it (the census's arithmetic, for one control) */
  const pair = el => { const cs = getComputedStyle(el); const fg = parse(cs.color); const b = bgOf(el); return +ratio(over(fg, b), b).toFixed(2); };
  const contrast = rootSel => { const root = rootSel ? document.querySelector(rootSel) : document.body; const out = []; if (!root) return out;
    const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT); const seen = new Set();
    while (w.nextNode()) { const t = w.currentNode; if (!t.nodeValue.trim()) continue; const el = t.parentElement; if (!el || seen.has(el)) continue; seen.add(el);
      if (!vis(el) || el.closest('option')) continue; const cs = getComputedStyle(el); const fg = parse(cs.color); if (!fg) continue;
      const bb0 = bgOf(el), o = opac(el); let f = over(fg, bb0); let bb = bb0; const body = parse(getComputedStyle(document.body).backgroundColor);
      if (o < 1) { f = over({ ...f, a: o }, body); bb = over({ ...bb0, a: o }, body); }
      const cr = ratio(f, bb), px = parseFloat(cs.fontSize), large = px >= 24 || (+cs.fontWeight >= 600 && px >= 18.66);
      const dis = !!el.closest(':disabled,[disabled],[aria-disabled="true"]');
      out.push({ el: desc(el), text: t.nodeValue.trim().slice(0, 40), ratio: +cr.toFixed(2), px, fail: cr < (large ? 3 : 4.5), dis }); }
    return out; };
  window.__aud = { parse, ratio, pair, contrast, vis, desc };
  window.__drawn = function (e) { var r = e.getBoundingClientRect(), L = r.left, T = r.top, R = r.right, B = r.bottom, pos = getComputedStyle(e).position;
    for (var a = e.parentElement; a && a !== document.body && a !== document.documentElement; a = a.parentElement) { if (pos === 'fixed') break; var s = getComputedStyle(a);
      if (!(pos === 'absolute' && s.position === 'static')) { var q = a.getBoundingClientRect(); if (s.overflowX !== 'visible') { L = Math.max(L, q.left); R = Math.min(R, q.right); } if (s.overflowY !== 'visible') { T = Math.max(T, q.top); B = Math.min(B, q.bottom); } }
      if (s.position !== 'static') pos = s.position; }
    return { l: L, t: T, r: R, b: B, w: Math.max(0, R - L), h: Math.max(0, B - T) }; };
}})();`;
const VIEWPORTS = new Set();
const boot = async (b, vp, o = {}) => {
  VIEWPORTS.add(vp.width + 'x' + vp.height + (o.touch ? ' touch' : ' mouse') + (o.light ? ' light' : ' dark'));
  const ctx = await b.newContext({ viewport: vp, hasTouch: !!o.touch, isMobile: !!o.touch && vp.width < 1000, userAgent: o.touch ? AND : undefined });
  await ctx.addInitScript(AUD_SRC);
  /* SET-UP: the theme in storage before the file loads, as a returning user's browser has it */
  await ctx.addInitScript(l => { try { localStorage.clear(); localStorage.setItem('cartalith_theme', l ? 'light' : 'dark'); } catch (_) {} }, !!o.light);
  const p = await ctx.newPage(); p._vp = vp;
  p.on('pageerror', e => errors.push(vp.width + 'x' + vp.height + ': ' + e.message));
  p.on('dialog', d => d.accept().catch(() => {}));
  await p.goto('file://' + path.resolve(FILE));
  await p.waitForFunction(() => typeof window.generate === 'function' && document.getElementById('onboard'), { timeout: 90000 });
  await sleep(500);
  p._ctx = ctx;
  return p;
};
const close = async p => { try { await p._ctx.close(); } catch (_) {} };
/* SET-UP: a small world, the gate dismissed the way a generated world dismisses it */
const gen = (p, seed, res) => p.evaluate(async ([seed, res]) => {
  state.tect.seed = seed; state.resW = res; GW = res; GH = gridH(GW); allocate();
  if (typeof _setupHide === 'function') _setupHide();
  await generate(); syncUI(); if (typeof _viewFill === 'function') _viewFill(); renderNow();
}, [seed || 12345, res || 256]);
const idle = async p => { for (let i = 0; i < 600; i++) { const d = await p.evaluate(async () => { try { await _busyChain; } catch (_) {} return typeof _busyDepth !== 'undefined' ? _busyDepth : 0; }); if (!d) break; await sleep(200); } await sleep(250); };
/* SET-UP: settlements and polities without the button (sections not about the button) */
const settle = p => p.evaluate(() => { _civIterativeAutoWorld(3); _civAutoPolity(); if (typeof drawCivLayerAuto === 'function') drawCivLayerAuto(); });
const rect = (p, sel) => p.evaluate(s => { const e = document.querySelector(s); if (!e || !e.getClientRects().length) return null; const r = e.getBoundingClientRect(); return { l: r.left, t: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height }; }, sel);
const click = async (p, sel) => { await p.evaluate(s => { const e = document.querySelector(s); if (e && e.scrollIntoView) e.scrollIntoView({ block: 'nearest' }); }, sel); const r = await rect(p, sel); if (!r || r.w < 1) return false; await p.mouse.click(r.l + r.w / 2, r.t + r.h / 2); await sleep(220); return true; };
const tap = async (p, sel) => { await p.evaluate(s => { const e = document.querySelector(s); if (e && e.scrollIntoView) e.scrollIntoView({ block: 'nearest' }); }, sel); const r = await rect(p, sel); if (!r || r.w < 1) return false; await p.touchscreen.tap(r.l + r.w / 2, r.t + r.h / 2); await sleep(260); return true; };
const css = (p, sel, prop, pseudo) => p.evaluate(([s, pr, ps]) => { const e = document.querySelector(s); return e ? getComputedStyle(e, ps || null)[pr] : null; }, [sel, prop, pseudo || null]);
const hashView = p => p.evaluate(() => { const d = vctx.getImageData(0, 0, view.width, view.height).data; let h = 2166136261 >>> 0; for (let i = 0; i < d.length; i++) { h ^= d[i]; h = Math.imul(h, 16777619) >>> 0; } return h >>> 0; });
const raf2 = p => p.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
/* a settlement visible on the map, biggest first (READ), for a real click on it */
const placeOnMap = p => p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect();
  for (const q of [...state.places].filter(x => x.pop && x.category === 'settlement').sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) return { x: s[0], y: s[1] }; } return null; });

(async () => {
  const b = await chromium.launch({ executablePath: process.env.CHROME_BIN, args: ['--no-sandbox'] });

  await section('version', async () => {
    const p = await boot(b, { width: 1400, height: 900 });
    const v = await p.evaluate(() => ({ t: document.title, tag: document.getElementById('verTag').textContent }));
    A('the version is v2.74 (title and tag)', /v2\.74/.test(v.t) && v.tag === 'v2.74', JSON.stringify(v));
    await close(p);
  });

  await section('P4 — no render behind the start card', async () => {
    const p = await boot(b, { width: 1400, height: 900 });
    const s = await p.evaluate(() => ({ loops: PERF.counters.renderPixelLoop, w: view.width, h: view.height, GW, GH, gate: getComputedStyle(document.getElementById('onboard')).display }));
    A('the gate is up and #view is already GW×GH (the layout and cover zoom a render would give), as before', s.gate === 'flex' && s.w === s.GW && s.h === s.GH, JSON.stringify(s));
    A('0 full renders before the user leaves the gate (renderPixelLoop)', s.loops === 0, s.loops);
    await click(p, '#obSkip'); await raf2(p);
    const a = await p.evaluate(() => { const d = vctx.getImageData(0, 0, view.width, view.height).data; const set = new Set(); for (let i = 0; i < d.length; i += 4 * 997) set.add(d[i] + ',' + d[i + 1] + ',' + d[i + 2]); return { loops: PERF.counters.renderPixelLoop, colours: set.size }; });
    A('"Skip for now" renders the empty world once (a rendered ocean, not the flat backdrop)', a.loops === 1 && a.colours > 3, JSON.stringify(a));
    await close(p);
  });

  await section('P7 — the bake-cache key', async () => {
    const p = await boot(b, { width: 1400, height: 900 });
    await gen(p, 12345, 256);
    const r = await p.evaluate(async () => {
      const raf2 = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
      const H = () => { const d = vctx.getImageData(0, 0, GW, GH).data; let h = 2166136261 >>> 0; for (let i = 0; i < d.length; i++) { h ^= d[i]; h = Math.imul(h, 16777619) >>> 0; } return h >>> 0; };
      renderNow(); await raf2();
      const h0 = H(); const el = document.getElementById('sun'); el.value = String((+el.value + 90) % 360); el.dispatchEvent(new Event('input', { bubbles: true })); await raf2();
      const h1 = H(); _civBakeCache.clear(); renderNow(); await raf2(); const h2 = H();
      const ex = document.getElementById('exag'); const e0 = H(); ex.value = String(Math.min(100, +ex.value + 20)); ex.dispatchEvent(new Event('input', { bubbles: true })); await raf2(); const e1 = H();
      const parts = [...new Set((_civBakeKey.toString().match(/state\.([A-Za-z_$][\w$]*)/g) || []).map(s => s.slice(6)))];
      return { sunMoves: h1 !== h0, dependsOnSun: h2 !== h0, exagMoves: e1 !== e0, missing: parts.filter(k => !(k in state)), parts };
    });
    A('positive control: the Relief (exag) slider changes the map, as before', r.exagMoves);
    A('the render depends on the sun azimuth (cache cleared), as before', r.dependsOnSun);
    A('dragging the Sun slider changes the map (the key reads state.sunAz)', r.sunMoves);
    A('every state.X part of _civBakeKey names a field the state literal declares', r.missing.length === 0, 'missing: ' + r.missing.join(','));
    /* the Sun slider by a REAL click on its track: CARTO › Map view, opened by clicks */
    await click(p, '#domainRail [data-domain="carto"]');
    const opened = await p.evaluate(() => { const s = [...document.querySelectorAll('details.cat-acc > summary')].find(x => /Map view/.test(x.textContent)); if (!s) return false; s.id = '__pbMapView'; return true; });
    if (opened && !(await p.evaluate(() => document.getElementById('__pbMapView').parentElement.open))) await click(p, '#__pbMapView');
    const sr = await rect(p, '#sun');
    const h0 = await hashView(p);
    if (sr) { await p.mouse.click(sr.l + sr.w * 0.25, sr.t + sr.h / 2); await sleep(400); await raf2(p); }
    const h1 = await hashView(p); const az = await p.evaluate(() => state.sunAz);
    A('a real click on the Sun slider\'s track moves the sun and redraws the map', !!sr && az !== 315 && h1 !== h0, 'sunAz ' + az + ', moved ' + (h1 !== h0));
    /* every render-calling control, once each, in Biome and in Relief view (dispatched events; see the header) */
    const src = fs.readFileSync(FILE, 'utf8'); const ids = new Set();
    for (const m of src.matchAll(/^bind\('([^']+)',[^\n]*render\(\)/gm)) ids.add('R:' + m[1]);
    for (const m of src.matchAll(/getElementById\('([^']+)'\)[^\n]{0,120}addEventListener\('change'[^\n]*render\(\)/g)) ids.add('C:' + m[1]);
    for (const m of src.matchAll(/const el=document\.getElementById\('([^']+)'\); if\(el\)\{[^\n]*addEventListener\('change'[^\n]*render\(\)/g)) ids.add('C:' + m[1]);
    const stale = await p.evaluate(async ids => {
      const raf2 = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
      const H = () => { const d = vctx.getImageData(0, 0, GW, GH).data; let h = 2166136261 >>> 0; for (let i = 0; i < d.length; i++) { h ^= d[i]; h = Math.imul(h, 16777619) >>> 0; } return h >>> 0; };
      const out = []; let n = 0;
      for (const mode of ['biome', 'hypso']) { state.mode = mode; _civBakeCache.clear(); renderNow(); await raf2();
        for (const t of ids) { const id = t.slice(2), el = document.getElementById(id); if (!el) continue; n++; let undo;
          if (t[0] === 'R') { const v0 = el.value, mn = +el.min || 0, mx = +el.max || 100; el.value = String(+v0 === mn ? (mn + mx) / 2 : mn); el.dispatchEvent(new Event('input', { bubbles: true })); undo = () => { el.value = v0; el.dispatchEvent(new Event('input', { bubbles: true })); }; }
          else { const c0 = el.checked; el.checked = !c0; el.dispatchEvent(new Event('change', { bubbles: true })); undo = () => { el.checked = c0; el.dispatchEvent(new Event('change', { bubbles: true })); }; }
          try { await _busyChain; } catch (_) {} await raf2(); await raf2(); const h1 = H(); _civBakeCache.clear(); renderNow(); await raf2(); const h2 = H();
          if (h1 !== h2) out.push(mode + ':' + id);
          undo(); try { await _busyChain; } catch (_) {} await raf2(); _civBakeCache.clear(); renderNow(); await raf2(); } }
      state.mode = 'biome'; return { out, n };
    }, [...ids]);
    A('no render-calling control serves a stale cached image, in Biome or Relief view (' + stale.n + ' control changes)', stale.out.length === 0 && stale.n > 40, stale.out.join(', '));
    await close(p);
  });

  await section('V01 — the Settlements table rows', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 12345, 256);
    await click(p, '#domainRail [data-domain="civ"]'); await click(p, '[data-civsub="generation"]');
    await click(p, '#civAutoPopulateBtn'); await idle(p);
    await click(p, '[data-civsub="settlements"]'); await sleep(500);
    const r = await p.evaluate(() => { const rows = [...document.querySelectorAll('#stSpacer > div')].filter(e => e.style.display !== 'none' && e.getClientRects().length);
      const head = document.getElementById('stTableHead'); const hk = head ? [...head.children].map(c => Math.round(c.getBoundingClientRect().left)) : [];
      const rk = rows[0] ? [...rows[0].children].map(c => Math.round(c.getBoundingClientRect().left)) : [];
      return { n: rows.length, flex: rows.filter(e => getComputedStyle(e).display === 'flex').length, hk, rk }; });
    A('every visible row of CIVIL ▸ Settlements computes display:flex', r.n > 0 && r.flex === r.n, r.flex + '/' + r.n);
    const aligned = r.hk.length > 1 && r.hk.length === r.rk.length && r.hk.every((x, i) => Math.abs(x - r.rk[i]) <= 1);
    A('the table head\'s columns line up with the first row\'s within 1px', aligned, 'head ' + r.hk.join(',') + ' row ' + r.rk.join(','));
    await close(p);
  });

  for (const light of [false, true]) {
    await section('V10 — Credits above Settings, desktop ' + (light ? 'light' : 'dark'), async () => {
      const p = await boot(b, { width: 1600, height: 900 }, { light });
      await gen(p, 12345, 256);
      await click(p, '#mbPrefsBtn'); await click(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(400);
      await click(p, '#setNav [data-setpage="about"]'); await click(p, '#creditsBtn'); await sleep(300);
      const top = await p.evaluate(() => { const e = document.elementFromPoint(innerWidth / 2, innerHeight / 2); return !!(e && e.closest('#creditsModal')); });
      A('after Settings ▸ About ▸ ⓘ the element at the window\'s centre is Credits\'', top);
      await close(p);
    });
  }
  await section('V10 — Credits above Settings, phone 412×915', async () => {
    const p = await boot(b, { width: 412, height: 915 }, { touch: true });
    await gen(p, 12345, 256);
    await tap(p, '#phTabMenu'); await tap(p, '#phMenuRoot > .mb-sub[data-menu="prefs"] > [data-sub]');
    await tap(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(400);
    await tap(p, '#setNav [data-setpage="about"]'); await tap(p, '#creditsBtn'); await sleep(300);
    const top = await p.evaluate(() => { const e = document.elementFromPoint(innerWidth / 2, innerHeight / 2); return !!(e && e.closest('#creditsModal')); });
    A('on a phone, after a tap on ⓘ, the screen\'s centre is Credits\'', top);
    const xr = await rect(p, '#creditsClose');
    if (xr) { await p.touchscreen.tap(xr.l + xr.w / 2, xr.t + xr.h / 2); await sleep(300); }
    A('a real tap on Credits\' × closes it', !!xr && !(await p.evaluate(() => document.getElementById('creditsModal').classList.contains('open'))));
    await close(p);
  });

  for (const light of [false, true]) {
    await section('V04/V05/V06 — pins, ×, panels (' + (light ? 'light' : 'dark') + ')', async () => {
      const p = await boot(b, { width: 1600, height: 900 }, { light });
      await gen(p, 12345, 256); await settle(p);
      await click(p, '#settlementsHeaderBtn'); await sleep(600);
      const s = await p.evaluate(() => ({ pin: __aud.pair(document.getElementById('settlementsHeaderBtn')), x: __aud.pair(document.querySelector('.cfm-close')),
        body: getComputedStyle(document.querySelector('.cfm-body')).backgroundColor, fails: __aud.contrast('#civFactionsModal').filter(r => r.fail && !r.dis).map(r => r.el + ' "' + r.text + '" ' + r.ratio), runs: __aud.contrast('#civFactionsModal').length }));
      A('the armed Settlements pin\'s label measures ≥ 4.5:1 at rest', s.pin >= 4.5, s.pin);
      A('the Settlements screen\'s × measures ≥ 4.5:1', s.x >= 4.5, s.x);
      A('the Settlements screen draws on --panel (' + (light ? 'rgb(242, 240, 236)' : 'rgb(18, 19, 20)') + ')', s.body === (light ? 'rgb(242, 240, 236)' : 'rgb(18, 19, 20)'), s.body);
      A('0 of the Settlements screen\'s ' + s.runs + ' text runs below AA', s.runs > 10 && s.fails.length === 0, s.fails.slice(0, 6).join(' | '));
      await click(p, '#cfmCloseBtn'); await sleep(300);
      await click(p, '#assetsHeaderBtn'); await sleep(700);
      const a = await p.evaluate(() => __aud.pair(document.getElementById('assetsHeaderBtn')));
      A('in the Asset Library the "← Map" pin\'s label measures ≥ 4.5:1 at rest', a >= 4.5, a);
      const re = await p.evaluate(() => ({ re: getComputedStyle(document.querySelector('.route-editor-modal')).backgroundColor, rex: (() => { const m = document.querySelector('.route-editor-modal'); m.classList.add('open'); const v = __aud.pair(m.querySelector('.re-close')); m.classList.remove('open'); return v; })(),
        cvx: (() => { const m = document.querySelector('.city-viewer-modal'); m.classList.add('open'); const v = __aud.pair(m.querySelector('.cv-close')); m.classList.remove('open'); return v; })() }));
      A('the route editor draws on --bg (' + (light ? 'rgb(244, 242, 238)' : 'rgb(13, 14, 15)') + ')', re.re === (light ? 'rgb(244, 242, 238)' : 'rgb(13, 14, 15)'), re.re);
      A('the route editor\'s × measures ≥ 4.5:1', re.rex >= 4.5, re.rex);
      A('the City viewer\'s × measures ≥ 4.5:1', re.cvx >= 4.5, re.cvx);
      await close(p);
    });
  }

  await section('V23 — the Settlements screen on a phone', async () => {
    const p = await boot(b, { width: 412, height: 915 }, { touch: true });
    await gen(p, 12345, 256);
    /* first with factions but no settlement: the overview's dash */
    await p.evaluate(() => { _civOpenFactionsModal(); });   /* SET-UP: the window opened; the rows are what is measured */
    await sleep(400);
    const o1 = await p.evaluate(() => document.getElementById('civWorldOverviewOut').textContent);
    A('with no settlement the overview dashes the largest faction with its reason', /Largest by population: — \(no settlements yet\)/.test(o1) && !/\(0\)/.test(o1), o1.slice(0, 160));
    /* SET-UP: one settlement, Unclaimed — the dash's second reason */
    const o2 = await p.evaluate(() => { const i = field.findIndex(v => v > state.seaLevel + 0.05); state.places.push({ name: 'Probe', kind: 'town', category: 'settlement', faction: 0, pop: 1500, x: i % GW, y: (i / GW) | 0, traits: [], history: '' }); _civRenderFactionsWorldOverview(); return document.getElementById('civWorldOverviewOut').textContent; });
    A('with only an Unclaimed settlement it says no faction holds a populated one', /no faction holds a populated settlement/.test(o2), o2.slice(0, 160));
    await p.evaluate(() => { state.places.length = 0; _civCloseFactionsModal(); });
    await settle(p);
    await tap(p, '#phTabMenu'); await tap(p, '#phMenuRoot > .mb-sub[data-menu="records"] > [data-sub]');
    await tap(p, '#mbRecords [data-cmd="factions"]'); await sleep(600);
    await tap(p, '#civPmBar button[data-pm="factions"]'); await sleep(400);
    const r = await p.evaluate(() => [...document.querySelectorAll('#civFactionList > div > b')].map(e => ({ t: e.textContent, w: Math.round(e.getBoundingClientRect().width), need: e.scrollWidth })));
    A('at 412×915 every faction name is drawn at least as wide as its text', r.length > 0 && r.every(x => x.w > 0 && x.w + 1 >= x.need), JSON.stringify(r.slice(0, 6)));
    await close(p);
  });

  await section('V11 — the busy overlay takes the map', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 12345, 256); await settle(p);
    const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(500); }
    await p.evaluate(() => showBusy('probe'));   /* SET-UP: the overlay up, as during a generate */
    await sleep(200);
    const r = await p.evaluate(() => { const hit = sel => { const e = document.querySelector(sel); if (!e || !e.getClientRects().length) return 'absent'; const q = e.getBoundingClientRect(); const t = document.elementFromPoint(q.left + q.width / 2, q.top + q.height / 2); return t && t.closest('#busy') ? 'overlay' : (t ? (t.id || t.className || t.tagName) : 'none'); };
      const pop = document.getElementById('placeEditPopup'); const pr = pop && pop.getClientRects().length ? pop.getBoundingClientRect() : null;
      const pt = pr ? document.elementFromPoint(pr.left + pr.width / 2, pr.top + 40) : null;
      return { zoom: hit('#zoomIn'), dim: hit('#viewDimSeg [data-dim="3d"]'), base: hit('#layersBtn'), popup: !!(pt && pt.closest('#placeEditPopup')), havePopup: !!pr }; });
    A('mid-operation the overlay is what lies over zoom, the 3D switch and the Base map chip (3 of 3)', r.zoom === 'overlay' && r.dim === 'overlay' && r.base === 'overlay', JSON.stringify(r));
    A('the place popup stays above the overlay', r.havePopup && r.popup, JSON.stringify(r));
    const fr = await rect(p, '#fileMenuBtn'); await p.mouse.click(fr.l + fr.w / 2, fr.t + fr.h / 2); await sleep(300);
    const m = await p.evaluate(() => { const pn = document.getElementById('fileMenu'); if (!pn || !pn.getClientRects().length) return 'closed'; const q = pn.getBoundingClientRect(), cw = document.querySelector('.canvas-wrap').getBoundingClientRect();
      const y = Math.max(q.top, cw.top) + 10, x = Math.max(q.left, cw.left) + 10; const t = document.elementFromPoint(x, y); return t && t.closest('#fileMenu') ? 'menu' : (t ? t.id || t.className : 'none'); });
    A('an open menu panel over the map stays above the overlay', m === 'menu', m);
    await p.keyboard.press('Escape'); await p.evaluate(() => hideBusy());
    await close(p);
  });

  for (const vp of [{ width: 1366, height: 768 }, { width: 1194, height: 834, touch: true }]) {
    await section('Assets panel in Assets mode, ' + vp.width + '×' + vp.height, async () => {
      const p = await boot(b, { width: vp.width, height: vp.height }, { touch: !!vp.touch });
      await gen(p, 12345, 256);
      const press = vp.touch ? tap : click;
      await press(p, '#assetsHeaderBtn'); await sleep(800);
      const floating = await p.evaluate(() => document.body.classList.contains('ldock-float'));
      if (floating) await press(p, '#dockLeftCollapse');
      await sleep(300);
      const r = await p.evaluate(() => { const ap = document.getElementById('assetsPanel'); const d = __drawn(ap); const vis = s => { const e = document.querySelector(s); return !!(e && e.getClientRects().length && getComputedStyle(e).display !== 'none'); };
        return { drawn: d.w > 50 && d.h > 50 && d.l >= 0 && d.r <= innerWidth && d.t >= 0 && d.t < innerHeight, ctx: vis('.dock-right'), rail: vis('#domainRail'), strip: vis('#tlStrip'), tools: vis('#toolsBlock'), gen: vis('#generatePanel'), lib: vis('#assetLibrary'),
          cols: getComputedStyle(document.getElementById('alGrid')).gridTemplateColumns.split(' ').length, path: (document.getElementById('dockLeftPath') || {}).textContent }; });
      A('#assetsPanel is drawn inside the window beside the library' + (floating ? ' (its floating dock opened by the chevron)' : ''), r.drawn && r.lib, JSON.stringify(r));
      A('what v2.24 hid stays hidden: the Context panel, the rail, the timeline strip, the dock\'s tools and domain bodies', !r.ctx && !r.rail && !r.strip && !r.tools && !r.gen, JSON.stringify(r));
      A('the library keeps at least two tile columns', r.cols >= 2, r.cols);
      A('the dock\'s head names the pack, not the domain the library hid', r.path === 'Assets › Pack', r.path);
      const v0 = await p.evaluate(() => document.getElementById('alValidation').textContent);
      await press(p, '#alValidateBtn'); await sleep(500);
      const v1 = await p.evaluate(() => document.getElementById('alValidation').textContent);
      A('Validate runs on a real ' + (vp.touch ? 'tap' : 'click') + ' (its report appears)', v1 !== v0 && v1.trim().length > 0, JSON.stringify(v1.slice(0, 80)));
      if (floating) await press(p, '#dockLeftCollapse');
      await press(p, '#assetsHeaderBtn'); await sleep(600);
      const back = await p.evaluate(() => ({ tools: getComputedStyle(document.getElementById('toolsBlock')).display !== 'none', panel: getComputedStyle(document.getElementById('assetsPanel')).display, path: document.getElementById('dockLeftPath').textContent }));
      A('leaving the library restores the dock (its tools back, the pack panel hidden, the domain in its head)', back.tools && back.panel === 'none' && back.path !== 'Assets › Pack', JSON.stringify(back));
      if (!vp.touch) {
        await click(p, '#findInput'); await p.keyboard.type('Pack name'); await sleep(500);
        const hit = await p.evaluate(() => { const rs = [...document.querySelectorAll('#findResults [data-find-i]')]; const h = rs.find(e => /Pack name/.test(e.textContent)); if (h) h.id = '__pbPack'; return !!h; });
        if (hit) await click(p, '#__pbPack');
        await sleep(700);
        const f = await p.evaluate(() => ({ on: document.body.classList.contains('assets-mode'), d: __drawn(document.getElementById('alPackName')) }));
        A('Find reveals "Pack name": the library opens and the field is drawn', hit && f.on && f.d.w > 20 && f.d.h > 5, JSON.stringify(f));
      }
      await close(p);
    });
  }

  await section('V16 — the place popup above the strip, 1366×768', async () => {
    const p = await boot(b, { width: 1366, height: 768 });
    await gen(p, 12345, 256); await settle(p);
    await click(p, '#tlStripToggle'); await sleep(400);
    let worst = -1e9, n = 0;
    for (let k = 0; k < 4; k++) {
      const at = await p.evaluate(k => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); const qs = [...state.places].filter(x => x.pop && x.category === 'settlement').map(q => [q, _civPlaceScreenPos(q.x, q.y)]).filter(([, s]) => s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 40 && s[1] < w.bottom - 40); const e = qs[k * 3]; return e ? { x: e[1][0], y: e[1][1] } : null; }, k);
      if (!at) continue; await p.mouse.click(at.x, at.y); await sleep(500);
      const r = await p.evaluate(() => { const pe = document.getElementById('placeEditPopup'), st = document.getElementById('tlStrip'); if (!pe || !pe.getClientRects().length) return null; return { pb: pe.getBoundingClientRect().bottom, st: st.getBoundingClientRect().top }; });
      if (r) { n++; worst = Math.max(worst, r.pb - r.st); }
      await p.keyboard.press('Escape'); await sleep(200);
    }
    A('with the strip expanded the popup\'s bottom stays at or above the strip\'s top (' + n + ' places, worst overlap px)', n > 0 && worst <= 0, worst.toFixed(1));
    await close(p);
    const q = await boot(b, { width: 412, height: 915 }, { touch: true });
    await gen(q, 12345, 256); await settle(q);
    await q.evaluate(() => { const s = state.places.find(x => x.category === 'settlement'); _civSelectedPlace = s; _civOpenPlacePopup(); });   /* SET-UP: a selection */
    const mh = await q.evaluate(() => document.getElementById('placeEditPopup').style.maxHeight);
    A('on a phone the popup keeps v2.73\'s cap (no inline max-height), as before', mh === '', mh);
    await close(q);
  });

  for (const light of [false, true]) {
    await section('V02/V08/V09/V12/V21 — tokens and the popup (' + (light ? 'light' : 'dark') + ')', async () => {
      const p = await boot(b, { width: 1600, height: 900 }, { light });
      await gen(p, 12345, 256); await settle(p);
      const t = await p.evaluate(() => { const r = getComputedStyle(document.documentElement); const cb = [...document.querySelectorAll('input[type=checkbox]')].find(e => e.checked);
        const pill = document.createElement('span'); pill.className = 'pill on'; pill.textContent = 'x'; document.body.appendChild(pill); const pc = getComputedStyle(pill).color; pill.remove();
        const lp = document.querySelector('.layers-fab .lp-item.on');
        return { scheme: r.colorScheme, faint: r.getPropertyValue('--faint').trim(), accent: r.getPropertyValue('--accent').trim(), check: cb ? getComputedStyle(cb).accentColor : null, pill: pc, lp: lp ? getComputedStyle(lp).color : null,
          sb: getComputedStyle(document.getElementById('scaleBar')).backgroundColor, lg: getComputedStyle(document.getElementById('legend')).backgroundColor, lgc: getComputedStyle(document.getElementById('legend')).color }; });
      A('the browser\'s own controls follow the theme: color-scheme ' + (light ? 'light' : 'dark'), t.scheme === (light ? 'light' : 'dark'), t.scheme);
      A('a checked box paints the accent (' + (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)') + ')', t.check === (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)'), t.check);
      A('--faint is ' + (light ? '#676762' : '#7d8285'), t.faint === (light ? '#676762' : '#7d8285'), t.faint);
      if (light) A('the light --accent is #8b550d (owner default Q3)', t.accent === '#8b550d', t.accent);
      A('an active pill\'s ink is --accInk (' + (light ? 'rgb(251, 250, 247)' : 'rgb(20, 16, 5)') + ')', t.pill === (light ? 'rgb(251, 250, 247)' : 'rgb(20, 16, 5)'), t.pill);
      A('the Base map popover\'s active row ink is --accInk', t.lp === (light ? 'rgb(251, 250, 247)' : 'rgb(20, 16, 5)'), t.lp);
      A('the scale bar\'s box is ' + (light ? 'the light theme\'s .85, as before' : '.85'), t.sb === (light ? 'rgba(251, 250, 247, 0.85)' : 'rgba(10, 12, 16, 0.85)'), t.sb);
      A('the legend\'s box is ' + (light ? 'the light theme\'s, as before' : '.86 with --muted text'), light ? (t.lg === 'rgba(251, 250, 247, 0.85)' && t.lgc === 'rgb(91, 92, 86)') : (t.lg === 'rgba(10, 12, 17, 0.86)' && t.lgc === 'rgb(169, 173, 176)'), t.lg + ' ' + t.lgc);
      const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(600); }
      const pp = await p.evaluate(() => { const pe = document.getElementById('placeEditPopup'); if (!pe || !pe.getClientRects().length) return null; const roll = document.getElementById('_civPeNameRoll');
        const inner = pe.getBoundingClientRect().right - parseFloat(getComputedStyle(pe).paddingRight) - parseFloat(getComputedStyle(pe).borderRightWidth);
        return { bg: getComputedStyle(pe).backgroundColor, cw: pe.clientWidth, sw: pe.scrollWidth, roll: roll ? roll.getBoundingClientRect().right : null, inner, fails: __aud.contrast('#placeEditPopup').filter(r => r.fail && !r.dis).map(r => r.el + ' "' + r.text + '" ' + r.ratio) }; });
      A('a settlement\'s popup opened by a real click', !!pp);
      if (pp) {
        A(light ? 'in light the popup draws on --panel (rgb(242, 240, 236))' : 'in dark the popup keeps its v0.90 palette (rgba(14, 17, 23, 0.97)), as before (owner default Q6)', pp.bg === (light ? 'rgb(242, 240, 236)' : 'rgba(14, 17, 23, 0.97)'), pp.bg);
        A('the Name row fits: clientWidth = scrollWidth and 🎲 inside the inner edge, as v2.73\'s fix pass made it', pp.cw === pp.sw && pp.roll != null && pp.roll <= pp.inner + 0.5, pp.cw + '/' + pp.sw + ' roll ' + pp.roll + ' inner ' + pp.inner);
        if (light) A('in light 0 popup text runs below AA', pp.fails.length === 0, pp.fails.slice(0, 6).join(' | '));
      }
      await close(p);
    });
  }

  await section('V03/V19/V20/V25 — active states, focus ring, fields, arrows', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 12345, 256);
    await click(p, '#domainRail [data-domain="civ"]'); await click(p, '[data-civsub="economy"]');
    const sides = sel => p.evaluate(s => { const e = document.querySelector(s); if (!e) return null; const c = getComputedStyle(e); return [c.borderTopWidth, c.borderRightWidth, c.borderBottomWidth, c.borderLeftWidth].join(' '); }, sel);
    const st = await sides('.subtab.on');
    A('an active sub-tab draws one 2px side (the bottom), three at 0', st === '0px 0px 2px 0px', st);
    await click(p, '#mbPrefsBtn'); await click(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(400);
    const sn = await sides('#setNav button.on');
    A('the active Settings page draws one 2px side (the left), three at 0', sn === '0px 0px 0px 2px', sn);
    await click(p, '#settingsClose');
    await click(p, '#domainRail [data-domain="world"]');
    await click(p, '#seedN'); const ring = [];
    for (let i = 0; i < 4; i++) { await p.keyboard.press('Tab'); await sleep(120); ring.push(await p.evaluate(() => { const e = document.activeElement, c = getComputedStyle(e); return (e.id || e.tagName) + ':' + c.outlineStyle + ' ' + c.outlineWidth + ' ' + c.outlineColor + ' ' + c.outlineOffset; })); }
    A('four controls reached by Tab draw the design\'s ring (2px solid accent, 2px out)', ring.every(x => /solid 2px rgb\(224, 163, 74\) 2px$/.test(x)), ring.join(' | '));
    const f = await p.evaluate(() => { const t = [...document.querySelectorAll('input[type=text]')].find(e => !e.getAttribute('style')); const n = document.querySelector('input[type=number]'), s = document.querySelector('select');
      return t ? { id: t.id, ff: getComputedStyle(t).fontFamily, pad: getComputedStyle(t).padding, nf: getComputedStyle(n).fontFamily, sf: getComputedStyle(s).fontFamily } : null; });
    A('a text field takes the UI font, not Arial, with 5px 7px padding', !!f && !/Arial/.test(f.ff) && f.pad === '5px 7px', JSON.stringify(f));
    A('numbers and selects stay mono', !!f && /monospace/.test(f.nf) && /monospace/.test(f.sf), f && (f.nf + ' / ' + f.sf));
    const mr = await css(p, 'details.cat-acc > summary', 'marginRight', '::before');
    A('a category\'s arrow keeps an 8px gap before its label', mr === '8px', mr);
    await close(p);
    const q = await boot(b, { width: 412, height: 915 }, { touch: true });
    await gen(q, 12345, 256);
    await tap(q, '#phTabMenu'); await tap(q, '#phMenuRoot > .mb-sub[data-menu="prefs"] > [data-sub]'); await tap(q, '#mbPrefs [data-cmd="setPerf"]'); await sleep(400);
    const ps = await q.evaluate(() => { const e = document.querySelector('#setNav button.on'); if (!e) return null; const c = getComputedStyle(e); return [c.borderTopWidth, c.borderRightWidth, c.borderBottomWidth, c.borderLeftWidth].join(' '); });
    A('on a phone the active Settings tab draws one 2px side (the bottom)', ps === '0px 0px 2px 0px', ps);
    await close(q);
  });

  await section('V07/V13 — asset wells, Export GeoJSON', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 12345, 256);
    await click(p, '#assetsHeaderBtn'); await sleep(900);
    const w = await p.evaluate(() => { const e = [...document.querySelectorAll('.al-card .al-thumb')].filter(t => t.querySelector(':scope > .al-empty') && t.getClientRects().length); return { n: e.length, white: e.filter(t => getComputedStyle(t).backgroundColor === 'rgb(255, 255, 255)').length }; });
    A('0 empty asset slots draw white on the dark library (' + w.n + ' empty slots)', w.n > 0 && w.white === 0, JSON.stringify(w));
    await click(p, '#assetsHeaderBtn'); await sleep(500);
    await click(p, '#fileMenuBtn'); const er = await rect(p, '#mbExportRow'); if (er) { await p.mouse.move(er.l + er.w / 2, er.t + er.h / 2); await sleep(400); }
    const g = await p.evaluate(() => { const e = document.getElementById('exportGeoBtn'), c = getComputedStyle(e); return c.borderTopStyle + ' ' + c.borderTopWidth + ' ' + c.backgroundColor; });
    A('Export GeoJSON is drawn as a button (1px solid border on --panel2)', g === 'solid 1px rgb(23, 25, 26)', g);
    await close(p);
  });

  for (const w of [1366, 1600, 1920, 2560]) {
    await section('V14/V18/V22 — status chips, CIVIL sub-tabs, chips and hint at ' + w, async () => {
      const p = await boot(b, { width: w, height: 900 });
      await gen(p, 12345, 256);
      const c = await p.evaluate(() => { const ro = document.getElementById('readout'), q = ro.getBoundingClientRect(); let cut = 0, shown = 0;
        for (const e of ro.children) { const r = e.getBoundingClientRect(); if (!r.width || getComputedStyle(e).display === 'none') continue; if (r.top >= q.bottom - 1) continue; shown++; if (r.right > q.right + 0.5 || r.left < q.left - 0.5) cut++; } return { cut, shown }; });
      A('0 status chips drawn cut', c.shown > 3 && c.cut === 0, JSON.stringify(c));
      const h = await p.evaluate(() => { const e = document.getElementById('ctxHint'); return { cw: e.clientWidth, sw: e.scrollWidth, t: e.textContent, body: document.getElementById('inspectorBody').textContent }; });
      A('the Context panel\'s empty hint is drawn whole and does not repeat the body', h.cw > 0 && h.sw <= h.cw + 1 && !h.body.includes(h.t), JSON.stringify({ cw: h.cw, sw: h.sw, t: h.t }));
      if (w === 1366 || w === 1600) {
        await click(p, '#domainRail [data-domain="civ"]');
        const s = await p.evaluate(() => { const d = document.querySelector('.dock-left'); const tabs = [...document.querySelectorAll('[data-civsub]')].filter(e => e.getClientRects().length); return { dsw: d.scrollWidth, dcw: d.clientWidth, cut: tabs.filter(e => e.scrollWidth > e.clientWidth + 1).map(e => e.textContent.trim()) }; });
        A('CIVIL\'s five sub-tabs fit the dock (no sideways scroll, no label cut)', s.dsw <= s.dcw && s.cut.length === 0, JSON.stringify(s));
      }
      await click(p, '#domainRail [data-domain="explore"]');
      const ch = await p.evaluate(() => { const a = document.getElementById('layersBtn').getBoundingClientRect(), f = document.querySelector('.map-tl .funnel-btn'); if (!f || !f.getClientRects().length) return null; return { a: a.bottom, f: f.getBoundingClientRect().bottom }; });
      A('the Base map chip and the Explore filter share a bottom edge', !!ch && Math.abs(ch.a - ch.f) < 0.5, JSON.stringify(ch));
      await close(p);
    });
  }

  for (const w of [1600, 1366]) {
    await section('V15/V28 — the sculpt bar with Freehand armed at ' + w + ', and the stale copy', async () => {
      const p = await boot(b, { width: w, height: 900 });
      await gen(p, 12345, 256);
      const idlePins = (await rect(p, '#toolOptsPins')) || {};
      await click(p, '#domainRail [data-domain="world"]'); await click(p, '#worldCatSeg [data-wcat="sculpt"]');
      await click(p, '#sculptFeatureSeg [data-sfeat="freehand"]'); await sleep(500);
      const r = await p.evaluate(() => { const fs = document.getElementById('sculptFeatureSeg'), q = fs.getBoundingClientRect(); const bs = [...fs.querySelectorAll('button')];
        const hidden = bs.filter(e => { const r = e.getBoundingClientRect(); return r.right > q.right + 0.5 || r.left < q.left - 0.5; }).length;
        return { n: bs.length, hidden, bar: Math.round(document.getElementById('toolOpts').getBoundingClientRect().height), hint: (document.getElementById('sculptFeatHint') || {}).textContent || '', world: getComputedStyle(document.querySelector('#toolsBlock [data-tooldomain="world"]')).display }; });
      A('with Freehand armed, ' + (w === 1600 ? '0 features are' : 'at most 1 feature is') + ' out of the picker\'s sight (of ' + r.n + ')', r.n >= 13 && r.hidden <= (w === 1600 ? 0 : 1), r.hidden);
      A('the bar stays two rows (≤ 80px)', r.bar <= 80, r.bar);
      A('the idle bar\'s pins end ' + (w - 10) + 'px in, as before', Math.round(idlePins.r || 0) === w - 10, idlePins.r);
      A('Freehand\'s hint no longer says "below"', r.hint.length > 20 && !/below/.test(r.hint), r.hint.slice(0, 80));
      A('WORLD\'s "no map tools" sentence is hidden while Sculpt is open', r.world === 'none', r.world);
      if (w === 1600) {
        await click(p, '#worldCatSeg [data-wcat="terrain"]');
        A('…and shown in Generate, as before', (await p.evaluate(() => getComputedStyle(document.querySelector('#toolsBlock [data-tooldomain="world"]')).display)) !== 'none');
        await click(p, '#tlStripToggle'); await sleep(300);
        const e1 = await p.evaluate(() => document.getElementById('tlStripEmpty').textContent);
        A('with the strip open and no year, the note says to press Add year below (not to open the strip)', /press Add year below/.test(e1) && !/open the strip/.test(e1), e1);
        await click(p, '#tlStripToggle'); await sleep(300);
        const e2 = await p.evaluate(() => document.getElementById('tlStripEmpty').textContent);
        A('…and with it closed, to open the strip, as before', /open the strip/.test(e2), e2);
      }
      await close(p);
    });
  }

  for (const vp of [{ width: 1366, height: 768 }, { width: 412, height: 915, touch: true }]) {
    await section('V17 — the New World card at ' + vp.width + '×' + vp.height, async () => {
      const p = await boot(b, { width: vp.width, height: vp.height }, { touch: !!vp.touch });
      await (vp.touch ? tap : click)(p, '#obGenerate'); await sleep(400);
      const c = await p.evaluate(() => [...document.querySelectorAll('#suArchSeg button')].filter(e => e.scrollWidth > e.clientWidth + 1).map(e => e.textContent));
      A('0 world-shape cells clip their label', c.length === 0, c.join(','));
      const card = await p.evaluate(() => { const c = document.querySelector('#onboard .card'); c.scrollTop = 160; return c.scrollHeight > c.clientHeight; });
      await sleep(200);
      const g = await p.evaluate(() => { const c = document.querySelector('#onboard .card').getBoundingClientRect(), a = document.querySelector('#obStepGenerate .su-actions').getBoundingClientRect(); return +(c.bottom - 1 - a.bottom).toFixed(1); });
      A('the sticky Back/Generate bar meets the card\'s edge (gap ≤ 1px)' + (card ? '' : ' [card did not scroll]'), g <= 1, g);
      await close(p);
    });
  }

  await section('V272-1/-2 — no forced layout before the controls move', async () => {
    const p = await boot(b, { width: 1400, height: 900 });
    await click(p, '#obSkip'); await sleep(500);
    const tf = path.join(require('os').tmpdir(), 'probe_v274fix_trace_' + process.pid + '.json');
    await b.startTracing(p, { path: tf, categories: ['devtools.timeline', 'disabled-by-default-devtools.timeline', 'disabled-by-default-devtools.timeline.stack'] });
    for (const w of [800, 1400]) { await p.setViewportSize({ width: w, height: 900 }); await raf2(p); await sleep(150); }
    await b.stopTracing();
    const ev = (JSON.parse(fs.readFileSync(tf, 'utf8')).traceEvents || []).filter(e => e.name === 'Layout');
    try { fs.unlinkSync(tf); } catch (_) {}
    const stacks = ev.map(e => ((e.args && e.args.beginData && e.args.beginData.stackTrace) || []).map(f => f.functionName || '(anon)').join('<'));
    const fromPlace = stacks.filter(s => /^_syncChromeTop<_shellPlace/.test(s)).length;
    const early = stacks.filter(s => /^(_syncChromeTop|_viewCoverScale<_viewFill)<\(anon\)$|^_syncChromeTop$/.test(s)).length;
    A('a class crossing forces no layout from _shellPlace (it measured right after moving the rows)', fromPlace === 0, fromPlace);
    A('a resize forces no layout from a window listener that ran before the controls moved', early === 0, early + ' of ' + ev.length + ' layouts');
    await close(p);
  });

  await section('F06 — the planner\'s claimed-land test and its toll', async () => {
    const p = await boot(b, { width: 1400, height: 900 });
    for (const seed of [12345, 777]) {
      await gen(p, seed, 256); await settle(p);
      const r = await p.evaluate(() => { const terr = getCivTerritory(); let sea = 0, seaT = 0, un = 0, unT = 0, cl = 0, clT = 0;
        for (let y = 0; y < GH; y++) for (let x = 0; x < GW; x++) { const i = y * GW + x; if (field[i] < state.seaLevel) { sea++; if (_jpClaimedAt(x, y)) seaT++; } else if (terr[i] === 0) { un++; if (_jpClaimedAt(x, y)) unT++; } else { cl++; if (_jpClaimedAt(x, y)) clT++; } }
        return { sea, seaT, un, unT, cl, clT }; });
      A('seed ' + seed + ': no sea cell reads as claimed (' + r.sea + ' cells)', r.sea > 0 && r.seaT === 0, r.seaT);
      A('seed ' + seed + ': no unclaimed land cell reads as claimed (' + r.un + ' cells)', r.unT === 0, r.unT);
      A('seed ' + seed + ': every faction\'s cell does, as before (' + r.cl + ' cells)', r.cl > 0 && r.clT === r.cl, r.clT + '/' + r.cl);
      if (seed === 777) {
        /* SET-UP: the audit's six journeys — settlements of different factions more than 15% of the map apart */
        const j = await p.evaluate(() => { const S = state.places.filter(q => q && q.category === 'settlement' && q.faction > 0); const pairs = [];
          for (let i = 0; i < S.length && pairs.length < 6; i++) for (let k = i + 1; k < S.length && pairs.length < 6; k++) if (S[i].faction !== S[k].faction && Math.hypot(S[i].x - S[k].x, S[i].y - S[k].y) > GW * 0.15) pairs.push([S[i], S[k]]);
          const out = []; for (const [a, c] of pairs) { _civWaypoints = [[a.x, a.y], [c.x, c.y]]; _civCommitRoute(); const jn = civJourneys[civJourneys.length - 1]; const pl = _jpPlan(jn); const co = pl && jpJourneyCost(pl); out.push(co ? co.tolls : null); }
          return out; });
        const priced = j.filter(x => x != null);
        A('seed 777: every priced journey pays a frontier toll (' + priced.length + ' of ' + j.length + ' priced)', priced.length >= 4 && priced.every(x => x > 0), JSON.stringify(j));
      }
    }
    await close(p);
  });

  await section('F06 — the planner says what the floor and the toll are', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 777, 256); await settle(p);
    await click(p, '#domainRail [data-domain="explore"]');
    await click(p, '[data-civtool="route"]');
    const two = await p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); const out = []; for (const q of [...state.places].filter(x => x.pop && x.category === 'settlement').sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) out.push({ x: s[0], y: s[1] }); if (out.length === 2) break; } return out; });
    for (const q of two) { await p.mouse.click(q.x, q.y); await sleep(400); }
    await click(p, '#civCommitRouteBtn'); await idle(p);
    await click(p, '#jpOpenEditorBtn'); await sleep(1200);
    const t = await p.evaluate(() => document.getElementById('routeEditorModal').textContent);
    A('the route editor says a faction\'s land never reads below Sparse Settlements', /never reads below\s*Sparse Settlements/.test(t));
    A('…and that a toll is one levy between a faction\'s land and unclaimed land or water', /between a faction's land and unclaimed land or water/.test(t));
    await close(p);
  });

  await b.close();
  console.log('\n' + (errors.length ? 'page errors:\n  ' + errors.slice(0, 10).join('\n  ') : 'no page errors'));
  A('no page errors', errors.length === 0, errors.length);
  console.log('\nviewports (' + VIEWPORTS.size + '): ' + [...VIEWPORTS].join(', '));
  console.log('\n' + pass + ' passed, ' + fail + ' failed' + (skipped ? ', ' + skipped + ' sections skipped (PROBE_ONLY — not a verdict)' : ''));
  process.exit(fail ? 1 : (skipped ? 3 : 0));
})();
