/* v2.74 — the fix pass and the design alignment, each finding checked at its bar. Owner: "can you check and improve
 * the visuals and performance of the html? Maybe even upgrade some features based of the godot version", then
 * "Fixes and feature ports first then stage 3" (Cartalith_GDT Ruling EF). The findings are the 2026-11-01 audits of
 * v2.72 (Cartalith_GDT html/audits/v2.72-2026-11-01/PLAN.md §5 and §8), with Ruling EC item 6 (the planner's toll
 * fix, "Apply Godot's fix") and item 7 (rounded corners, "Rounded, as the design").
 *
 * One section per finding id. The fix pass: P7, V01, V10, V04, V05, V06, V23, V11, the Assets panel, V16,
 * V02/V08/V09, V12, V03, V19, V20, V21, V07, V13, V14, V15, V17, V18, V22, V25's arrow, V28, P4, V272-1/-2, F06.
 * The design alignment: V26 (the corners, desktop and touch), V24 (the dock's segments, action buttons, sub-tabs,
 * Bake and Un-finalize), V25 (one section label), V29 (the timeline strip's toggles), V30 (one window heading) and,
 * given a second file, the truncation census against it. Every assertion is meant to FAIL on v2.73 unless it says
 * otherwise in its own text ("…, as before"): run the probe on v2.73 too and read the count it prints (MISTAKES.md
 * "Add a probe in the same commit that changes the behaviour").
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
 * Usage: node tests/perf/probe_fixpass.js "Cartalith v2.74 DCC test.html" ["Cartalith v2.73 DCC test.html"]
 *   The first file is probed. The optional second file is the BEFORE build: given one, the truncation census
 *   (the visual audit's lib.js __aud.truncs) is taken on both at 1366×768 and 1600×900 on two seeds, and every
 *   truncation the first file has that the second does not is a failure; the rows of buttons that wrap in the
 *   first file are listed (not asserted). Without it that section is skipped and says so. Any other argument
 *   count, or a file that does not exist, is refused (exit 2). PROBE_ONLY=<regex> runs only the matching
 *   sections and exits 3 (not a verdict) even when nothing failed.
 * Env: PLAYWRIGHT_DIR, CHROME_BIN — as every probe in this folder.
 * Output: "ok   - …" / "FAIL - …" per assertion, then a count. Exit 1 if anything failed.
 */
const path = require('path');
const fs = require('fs');
const { chromium } = require(process.env.PLAYWRIGHT_DIR || 'playwright');

const FILE = process.argv[2], BEFORE = process.argv[3];
if (!FILE || process.argv.length > 4 || !fs.existsSync(FILE) || (BEFORE !== undefined && !fs.existsSync(BEFORE))) {
  console.error('usage: node probe_fixpass.js <v2.74.html> [<before.html>]  — got: ' + JSON.stringify(process.argv.slice(2)));
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
  /* the visual audit's truncation census (lib.js __aud.truncs, its test unchanged): text cut by a box that hides
     its overflow or ellipsises, and boxes that scroll sideways */
  const truncs = () => { const out = []; for (const el of document.body.querySelectorAll('*')) { if (!vis(el)) continue; const cs = getComputedStyle(el);
      const txt = (el.innerText || '').trim(); if (!txt) continue; const hidX = cs.overflowX !== 'visible'; const ell = cs.textOverflow === 'ellipsis';
      if ((hidX || ell) && el.scrollWidth > el.clientWidth + 1 && el.children.length <= 6 && !/auto|scroll/.test(cs.overflowX)) out.push(desc(el) + ' "' + txt.slice(0, 40) + '"');
      else if (/auto|scroll/.test(cs.overflowX) && el.scrollWidth > el.clientWidth + 1 && el.clientWidth > 0) out.push('scroller ' + desc(el) + ' "' + txt.slice(0, 30) + '"'); }
    return out; };
  /* rows of buttons that wrap: a flex row whose laid-out children sit on more than one line (below the fold too) */
  const laid = el => { const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; for (let e = el; e && e.nodeType === 1; e = e.parentElement) { const c = getComputedStyle(e); if (c.display === 'none' || c.visibility === 'hidden') return false; } return true; };
  const wraps = () => { const out = []; for (const c of document.querySelectorAll('.btnrow,.seg:not(.grid),.run-row,.subtab-bar,.tl-more,.tl-filters,.to-pins')) { if (!laid(c) || !/flex/.test(getComputedStyle(c).display)) continue;
      const kids = [...c.children].filter(laid); if (kids.length < 2) continue; const lines = new Set(kids.map(k => Math.round(k.getBoundingClientRect().top / 4))).size;
      if (lines > 1) out.push(desc(c) + ' [' + lines + ' lines: ' + kids.map(k => (k.innerText || '').trim().slice(0, 14)).join(' | ') + ']'); }
    return out; };
  window.__aud = { parse, ratio, pair, contrast, vis, desc, truncs, wraps, over };
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
  /* o.file: the BEFORE build, for the truncation comparison only */
  await p.goto('file://' + path.resolve(o.file || FILE));
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
    A('"Skip for now" leaves the empty world rendered (a rendered ocean, not the flat backdrop), as v2.73\'s boot left it', a.loops === 1 && a.colours > 3, JSON.stringify(a));
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
      const code = _civBakeKey.toString().replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/[^\n]*/g, '');   /* comments stripped: they name the old fields in prose */
      const parts = [...new Set((code.match(/state\.([A-Za-z_$][\w$]*)/g) || []).map(s => s.slice(6)))];
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
      await p.mouse.move(3, 300); await sleep(150);   /* at REST: the pointer off the pin (the audit's 1:1 was at rest; a hover washes it) */
      const s = await p.evaluate(() => ({ pin: __aud.pair(document.getElementById('settlementsHeaderBtn')), pinBg: getComputedStyle(document.getElementById('settlementsHeaderBtn')).backgroundColor, x: __aud.pair(document.querySelector('.cfm-close')),
        body: getComputedStyle(document.querySelector('.cfm-body')).backgroundColor, fails: __aud.contrast('#civFactionsModal').filter(r => r.fail && !r.dis).map(r => r.el + ' "' + r.text + '" ' + r.ratio), runs: __aud.contrast('#civFactionsModal').length }));
      const WASH2 = light ? 'rgba(164, 101, 15, 0.15)' : 'rgba(224, 163, 74, 0.16)';
      A('the armed Settlements pin is a wash (' + WASH2 + '), not the accent fill', s.pinBg === WASH2, s.pinBg);
      A('the armed Settlements pin\'s label measures ≥ 4.5:1 at rest (as before: the fill carried --accInk)', s.pin >= 4.5, s.pin);
      A('the Settlements screen\'s × measures ≥ 4.5:1' + (light ? '' : ' (as before in dark)'), s.x >= 4.5, s.x);
      A('the Settlements screen draws on --panel (' + (light ? 'rgb(242, 240, 236)' : 'rgb(18, 19, 20)') + ')', s.body === (light ? 'rgb(242, 240, 236)' : 'rgb(18, 19, 20)'), s.body);
      A('0 of the Settlements screen\'s ' + s.runs + ' text runs below AA', s.runs > 10 && s.fails.length === 0, s.fails.slice(0, 6).join(' | '));
      await click(p, '#cfmCloseBtn'); await sleep(300);
      await click(p, '#assetsHeaderBtn'); await sleep(700);
      await p.mouse.move(3, 300); await sleep(150);
      const a = await p.evaluate(() => ({ r: __aud.pair(document.getElementById('assetsHeaderBtn')), bg: getComputedStyle(document.getElementById('assetsHeaderBtn')).backgroundColor }));
      A('in the Asset Library the "← Map" pin\'s label measures ≥ 4.5:1 at rest (it was accent on accent, 1:1)', a.r >= 4.5, a.r);
      A('…and it is a wash, not the accent fill', a.bg === WASH2, a.bg);
      const re = await p.evaluate(() => ({ re: getComputedStyle(document.querySelector('.route-editor-modal')).backgroundColor, rex: (() => { const m = document.querySelector('.route-editor-modal'); m.classList.add('open'); const v = __aud.pair(m.querySelector('.re-close')); m.classList.remove('open'); return v; })(),
        cvx: (() => { const m = document.querySelector('.city-viewer-modal'); m.classList.add('open'); const v = __aud.pair(m.querySelector('.cv-close')); m.classList.remove('open'); return v; })() }));
      A('the route editor draws on --bg (' + (light ? 'rgb(244, 242, 238)' : 'rgb(13, 14, 15)') + ')', re.re === (light ? 'rgb(244, 242, 238)' : 'rgb(13, 14, 15)'), re.re);
      A('the route editor\'s × measures ≥ 4.5:1' + (light ? '' : ' (as before in dark)'), re.rex >= 4.5, re.rex);
      A('the City viewer\'s × measures ≥ 4.5:1' + (light ? '' : ' (as before in dark)'), re.cvx >= 4.5, re.cvx);
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
    A('the place popup stays above the overlay, as before', r.havePopup && r.popup, JSON.stringify(r));
    const fr = await rect(p, '#mbHelpBtn'); await p.mouse.click(fr.l + fr.w / 2, fr.t + fr.h / 2); await sleep(300);
    const m = await p.evaluate(() => { const pn = document.getElementById('mbHelp'); if (!pn || !pn.getClientRects().length) return 'closed'; const q = pn.getBoundingClientRect(), cw = document.querySelector('.canvas-wrap').getBoundingClientRect();
      /* a point inside BOTH the open panel and the map */
      const l = Math.max(q.left, cw.left), r = Math.min(q.right, cw.right), t0 = Math.max(q.top, cw.top), b0 = Math.min(q.bottom, cw.bottom);
      if (r - l < 4 || b0 - t0 < 4) return 'no overlap with the map';
      const t = document.elementFromPoint((l + r) / 2, (t0 + b0) / 2); return t && t.closest('#mbHelp') ? 'menu' : (t ? t.id || t.className : 'none'); });
    A('an open menu panel over the map stays above the overlay, as before', m === 'menu', m);
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
      A('as before, what v2.24 hid stays hidden: the Context panel, the rail, the timeline strip, the dock\'s tools and domain bodies', !r.ctx && !r.rail && !r.strip && !r.tools && !r.gen, JSON.stringify(r));
      A('the library keeps at least two tile columns (v2.73: it had the whole stage)', r.cols >= 2, r.cols);
      A('the dock\'s head names the pack, not the domain the library hid', r.path === 'Assets › Pack', r.path);
      const v0 = await p.evaluate(() => document.getElementById('alValidation').textContent);
      await press(p, '#alValidateBtn'); await sleep(500);
      const v1 = await p.evaluate(() => document.getElementById('alValidation').textContent);
      A('Validate runs on a real ' + (vp.touch ? 'tap' : 'click') + ' (its report appears)', v1 !== v0 && v1.trim().length > 0, JSON.stringify(v1.slice(0, 80)));
      if (floating) await press(p, '#dockLeftCollapse');
      await press(p, '#assetsHeaderBtn'); await sleep(600);
      const back = await p.evaluate(() => ({ tools: getComputedStyle(document.getElementById('toolsBlock')).display !== 'none', panel: getComputedStyle(document.getElementById('assetsPanel')).display, path: document.getElementById('dockLeftPath').textContent }));
      A('leaving the library restores the dock (its tools back, the pack panel hidden, the domain in its head), as before', back.tools && back.panel === 'none' && back.path !== 'Assets › Pack', JSON.stringify(back));
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
        const pill = document.createElement('span'); pill.className = 'pill on'; pill.textContent = 'x'; document.body.appendChild(pill); const pc = getComputedStyle(pill).color, pb = getComputedStyle(pill).backgroundColor; pill.remove();
        const lp = document.querySelector('.layers-fab .lp-item.on');
        return { scheme: r.colorScheme, faint: r.getPropertyValue('--faint').trim(), accent: r.getPropertyValue('--accent').trim(), check: cb ? getComputedStyle(cb).accentColor : null, pill: pc, pillBg: pb, lp: lp ? getComputedStyle(lp).color : null, lpBg: lp ? getComputedStyle(lp).backgroundColor : null,
          sb: getComputedStyle(document.getElementById('scaleBar')).backgroundColor, lg: getComputedStyle(document.getElementById('legend')).backgroundColor, lgc: getComputedStyle(document.getElementById('legend')).color }; });
      A('the browser\'s own controls follow the theme: color-scheme ' + (light ? 'light' : 'dark'), t.scheme === (light ? 'light' : 'dark'), t.scheme);
      A('a checked box paints the accent (' + (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)') + ')', t.check === (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)'), t.check);
      A('--faint is ' + (light ? '#676762' : '#7d8285'), t.faint === (light ? '#676762' : '#7d8285'), t.faint);
      if (light) A('the light --accent is #8b550d (owner default Q3)', t.accent === '#8b550d', t.accent);
      /* V09's two inks on the accent fill were first moved to --accInk; the design alignment (V29, V24) then
         washed both on-states, so no ink sits on the accent fill there any more: the bar is the washed state's
         literal accent ink (the 2026-09-06 ruling, item 7). Its contrast is V29's and the census's. */
      A('an active pill is washed: accent ink (' + (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)') + ') on wash2, no accent fill', t.pill === (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)') && t.pillBg === (light ? 'rgba(164, 101, 15, 0.15)' : 'rgba(224, 163, 74, 0.16)'), t.pill + ' on ' + t.pillBg);
      A('the Base map popover\'s active row is washed: accent ink on wash2, no accent fill', t.lp === (light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)') && t.lpBg === (light ? 'rgba(164, 101, 15, 0.15)' : 'rgba(224, 163, 74, 0.16)'), t.lp + ' on ' + t.lpBg);
      A('the scale bar\'s box is ' + (light ? 'the light theme\'s .85, as before' : '.85'), t.sb === (light ? 'rgba(251, 250, 247, 0.85)' : 'rgba(10, 12, 16, 0.85)'), t.sb);
      A('the legend\'s box is ' + (light ? 'the light theme\'s, as before' : '.86 with --muted text'), light ? (t.lg === 'rgba(251, 250, 247, 0.85)' && t.lgc === 'rgb(91, 92, 86)') : (t.lg === 'rgba(10, 12, 17, 0.86)' && t.lgc === 'rgb(169, 173, 176)'), t.lg + ' ' + t.lgc);
      const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(600); }
      const pp = await p.evaluate(() => { const pe = document.getElementById('placeEditPopup'); if (!pe || !pe.getClientRects().length) return null; const roll = document.getElementById('_civPeNameRoll');
        const inner = pe.getBoundingClientRect().right - parseFloat(getComputedStyle(pe).paddingRight) - parseFloat(getComputedStyle(pe).borderRightWidth);
        return { bg: getComputedStyle(pe).backgroundColor, cw: pe.clientWidth, sw: pe.scrollWidth, roll: roll ? roll.getBoundingClientRect().right : null, inner, fails: __aud.contrast('#placeEditPopup').filter(r => r.fail && !r.dis).map(r => r.el + ' "' + r.text + '" ' + r.ratio) }; });
      A('(set-up) a settlement\'s popup opened by a real click', !!pp);
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
    /* SET-UP: three bare fields in the dock — what any field without a rule of its own gets (the landmass names'
       case, which are built by script) — read, then removed */
    const f = await p.evaluate(() => { const host = document.getElementById('toolsBlock'); const t = document.createElement('input'); t.type = 'text'; const n = document.createElement('input'); n.type = 'number'; const s = document.createElement('select');
      [t, n, s].forEach(e => host.appendChild(e)); const r = { ff: getComputedStyle(t).fontFamily, pad: getComputedStyle(t).padding, nf: getComputedStyle(n).fontFamily, sf: getComputedStyle(s).fontFamily }; [t, n, s].forEach(e => e.remove()); return r; });
    A('a text field takes the UI font, not Arial, with 5px 7px padding', !!f && !/Arial/.test(f.ff) && f.pad === '5px 7px', JSON.stringify(f));
    A('numbers and selects stay mono, as before', !!f && /monospace/.test(f.nf) && /monospace/.test(f.sf), f && (f.nf + ' / ' + f.sf));
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
      A('0 status chips drawn cut' + (w === 1600 ? '' : ' (as before at this width: v2.73 cut one at 1600)'), c.shown > 3 && c.cut === 0, JSON.stringify(c));
      const h = await p.evaluate(() => { const e = document.getElementById('ctxHint'); return { cw: e.clientWidth, sw: e.scrollWidth, t: e.textContent, body: document.getElementById('inspectorBody').textContent }; });
      A('the Context panel\'s empty hint is drawn whole and does not repeat the body', h.cw > 0 && h.sw <= h.cw + 1 && !h.body.includes(h.t), JSON.stringify({ cw: h.cw, sw: h.sw, t: h.t }));
      if (w === 1366 || w === 1600) {
        await click(p, '#domainRail [data-domain="civ"]');
        const s = await p.evaluate(() => { const d = document.querySelector('.dock-left'); const tabs = [...document.querySelectorAll('[data-civsub]')].filter(e => e.getClientRects().length); return { dsw: d.scrollWidth, dcw: d.clientWidth, cut: tabs.filter(e => e.scrollWidth > e.clientWidth + 1).map(e => e.textContent.trim()) }; });
        A('CIVIL\'s five sub-tabs fit the dock (no sideways scroll, no label cut)' + (w === 1600 ? ' (as before at 1600, with 1px to spare)' : ''), s.dsw <= s.dcw && s.cut.length === 0, JSON.stringify(s));
      }
      await click(p, '#domainRail [data-domain="explore"]');
      const ch = await p.evaluate(() => { const a = document.getElementById('layersBtn').getBoundingClientRect(), f = document.querySelector('#explFilterFab .funnel-btn'); if (!f || !f.getClientRects().length) return null; return { a: a.bottom, f: f.getBoundingClientRect().bottom }; });
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
      A('the bar stays two rows (≤ 80px), as before', r.bar <= 80, r.bar);
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
    const tf = path.join(require('os').tmpdir(), 'probe_fixpass_trace_' + process.pid + '.json');
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
    /* SET-UP: a route the planner can price (a blocked plan draws no cost block), committed through the same
       _civCommitRoute the ✓ Commit route button runs; the editor is then opened by a real click */
    const priced = await p.evaluate(() => { const S = state.places.filter(q => q.category === 'settlement').sort((a, b) => b.pop - a.pop);
      for (let i = 0; i < S.length; i++) for (let k = i + 1; k < Math.min(S.length, i + 6); k++) { _civWaypoints = [[S[i].x, S[i].y], [S[k].x, S[k].y]]; _civCommitRoute();
        const jn = civJourneys[civJourneys.length - 1]; const pl = _jpPlan(jn); if (pl && jpJourneyCost(pl)) return true; }
      return false; });
    await idle(p);
    await click(p, '#jpOpenEditorBtn');
    await p.waitForFunction(() => /Priced in/.test(document.getElementById('routeEditorModal').textContent), null, { timeout: 15000 }).catch(() => {});
    const t = await p.evaluate(() => document.getElementById('routeEditorModal').textContent);
    A('the route editor says a faction\'s land never reads below Sparse Settlements', /never reads below\s*Sparse Settlements/.test(t));
    const ti = t.indexOf('Tolls are');
    A('…and that a toll is one levy between a faction\'s land and unclaimed land or water', /between a faction's land and unclaimed land or water/.test(t), ti >= 0 ? JSON.stringify(t.slice(ti, ti + 140)) : 'no cost block; "Priced in" ' + /Priced in/.test(t));
    await close(p);
  });

  /* ═══════════ the design alignment: V26 (the corners), V24, V25, V29, V30, and the truncation census ═══════════
     Every literal below is the value v2.74 means — a token's value, or a length from the design
     (design/mobile-shell-2026-10-31: the pointer radii 16/14/12/10px and pills, the touch radii 24/20px) —
     never a value read back from the page and compared with itself (MISTAKES.md "Assert a drawn colour"). */
  const T = light => ({ accent: light ? 'rgb(139, 85, 13)' : 'rgb(224, 163, 74)', wash2: light ? 'rgba(164, 101, 15, 0.15)' : 'rgba(224, 163, 74, 0.16)',
    panel2: light ? 'rgb(251, 250, 247)' : 'rgb(23, 25, 26)', accInk: light ? 'rgb(251, 250, 247)' : 'rgb(20, 16, 5)', dim: light ? 'rgb(91, 92, 86)' : 'rgb(141, 146, 150)',
    muted: light ? 'rgb(78, 79, 74)' : 'rgb(169, 173, 176)', line: light ? 'rgba(0, 0, 0, 0.12)' : 'rgba(255, 255, 255, 0.1)' });
  /* READ: the four corners of every element a selector matches that is drawn (getClientRects), as one string */
  const corners = (p, sel) => p.evaluate(s => [...document.querySelectorAll(s)].filter(e => e.getClientRects().length).map(e => { const c = getComputedStyle(e); return [c.borderTopLeftRadius, c.borderTopRightRadius, c.borderBottomRightRadius, c.borderBottomLeftRadius].join(' '); }), sel);
  const radiusIs = async (p, sel, want, label) => { const r = await corners(p, sel); const w4 = want.split(' ').length === 1 ? [want, want, want, want].join(' ') : want;
    A(label + ' (' + sel + ') has ' + want + ' corners', r.length > 0 && r.every(x => x === w4), r.slice(0, 3).join(' | ') || 'not drawn'); };

  await section('V26 — the corners, desktop 1600×900', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 12345, 256); await settle(p);
    await radiusIs(p, '#reseedBtn', '16px', 'a dock button (the design\'s --r24 at a pointer: a pill when it is one line)');
    await radiusIs(p, '#genBtn', '16px', 'the primary button');
    await radiusIs(p, '#seedN', '12px', 'a number field (the design\'s field, .clr-in)');
    await radiusIs(p, '#bakeAllDepth', '12px', 'a select');
    await radiusIs(p, '#pinSave', '999px', 'a pin');
    await radiusIs(p, '#layersBtn', '999px', 'the Base map chip');
    await radiusIs(p, '#findInput', '999px', 'the search field');
    await radiusIs(p, '#zoomIn', '999px', 'a map button');
    await radiusIs(p, '#fileMenuBtn', '10px', 'a menu-bar button (the design\'s menu buttons)');
    await radiusIs(p, '#undoBtn', '10px', 'Undo');
    await radiusIs(p, '#tlStripToggle', '10px', 'the strip\'s TIMELINE toggle');
    await radiusIs(p, '#extentSeg', '16px', 'a segmented control');
    const sk = await p.evaluate(() => { const s = document.getElementById('extentSeg'); return { ov: getComputedStyle(s).overflow, kids: [...s.children].map(e => { const c = getComputedStyle(e); return c.borderTopRightRadius + ' ' + c.borderBottomRightRadius; }) }; });
    A('…which clips its children (overflow hidden), the dividing edges square, so its inner edges stay straight, as before', sk.ov === 'hidden' && sk.kids.length >= 2 && sk.kids.every(k => k === '0px 0px'), JSON.stringify(sk));
    /* the regression guards: what the universal reset kept square must stay square without it */
    for (const s of ['#dockLeft', '#dockRight', 'header', '#toolOpts', '#statusBar', '#tlStrip', '.canvas-wrap', '#legend', '#scaleBar']) await radiusIs(p, s, '0px', 'a surface stays square, as before:');
    for (const s of ['.domain-rail .dr-btn', '#ctxTabInspect']) await radiusIs(p, s, '0px', 'a rail item or a tab stays square, as before:');
    await click(p, '#mbHelpBtn'); await sleep(200);
    await radiusIs(p, '#mbHelp', '0px', 'an open menu stays square, as before:');
    await radiusIs(p, '#mbHelp .mb-row', '0px', 'its rows stay square, as before:');
    await p.keyboard.press('Escape'); await sleep(150);
    await click(p, '#domainRail [data-domain="civ"]');
    await radiusIs(p, '#civSubBar .subtab', '0px', 'a sub-tab stays square (an underline on a band), as before:');
    const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(600); }
    await radiusIs(p, '#placeEditPopup', '0px', 'the place popup stays square, as before:');
    await radiusIs(p, '#placeEditPopup .pill', '999px', 'a trait pill');
    await radiusIs(p, '#_civPeHist', '12px', 'a textarea (the place\'s History)');
    await click(p, '#placeEditPopup .si-close');
    await click(p, '#mbRecordsBtn'); await click(p, '#mbRecords [data-cmd="factions"]'); await sleep(600);
    await radiusIs(p, '#civFactionsModal .cfm-body', '0px', 'the Settlements window stays square, as before:');
    await radiusIs(p, '.cfm-close', '16px', 'its × (a 32px button, so a circle)');
    await click(p, '#cfmCloseBtn');
    const uni = await p.evaluate(() => { let n = 0; for (const sh of document.styleSheets) { let rs; try { rs = sh.cssRules; } catch (_) { continue; } for (const r of rs) if (r.selectorText && r.selectorText.trim() === '*' && r.style && r.style.getPropertyValue('border-radius')) n++; } return n; });
    A('no stylesheet rule sets a radius on every element (v2.23\'s *{border-radius:0 !important} is gone)', uni === 0, uni);
    const src = fs.readFileSync(FILE, 'utf8');
    A('the shell block\'s comment no longer calls the mockup flat (Ruling BK)', /mockup is NOT flat/.test(src) && !/the accent used sparingly;\s*\n\s*flat; radius 0/.test(src));
    await close(p);
  });

  await section('V26 — the corners on the touch shells (the design\'s touch values)', async () => {
    const p = await boot(b, { width: 1194, height: 834 }, { touch: true });
    await gen(p, 12345, 256);
    await radiusIs(p, '#reseedBtn', '24px', 'tablet: a dock button (the design\'s --r24 at touch)');
    await radiusIs(p, '#genBtn', '24px', 'tablet: the primary button');
    await radiusIs(p, '#seedN', '12px', 'tablet: a field keeps 12px (the design\'s .clr-in at both densities)');
    await radiusIs(p, '#worldCatSeg', '24px', 'tablet: a segmented control');
    const rk = await p.evaluate(() => ['zoomOut', 'zoomIn'].map(id => { const c = getComputedStyle(document.getElementById(id)); return [c.borderTopLeftRadius, c.borderTopRightRadius, c.borderBottomRightRadius, c.borderBottomLeftRadius].join(' '); }));
    A('tablet: DS-13\'s − | + rocker is one pill — − rounds its outer corners, + its own, and they meet square', rk[0] === '999px 0px 0px 999px' && rk[1] === '0px 999px 999px 0px', rk.join(' | '));
    await radiusIs(p, '#zoomReset', '999px', 'tablet: ⌖ is round');
    await radiusIs(p, '#undoBtn', '10px', 'tablet: Undo keeps the menu bar\'s 10px');
    await close(p);
    const q = await boot(b, { width: 412, height: 915 }, { touch: true });
    await gen(q, 12345, 256);
    await radiusIs(q, '#undoBtn', '24px', 'phone: the app bar\'s Undo is round (44px, the design\'s app bar)');
    await radiusIs(q, '.ph-tab', '0px', 'phone: the bottom bar\'s cells stay square (a band), as before');
    await radiusIs(q, '.ph-grip-bar', '0px 14px 14px 0px', 'phone: the pull-out tab rounds the side facing the map (the design\'s gripRadius)');
    await tap(q, '#phTabMenu'); await sleep(300);
    await radiusIs(q, '#phMenuRoot [data-sub]', '0px', 'phone: MENU\'s rows stay square, as before');
    await radiusIs(q, '.ph-sheet-btn', '24px', 'phone: the sheet\'s head buttons are round');
    await close(q);
  });

  await section('V26 — a grid segment keeps its grid borders (#debugSeg)', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    /* READ: computed values of the hidden 6-column grid the Layers popover proxies (laid out nowhere visible) */
    const g = await p.evaluate(() => { const s = document.getElementById('debugSeg'), bs = [...s.querySelectorAll('button')], c = getComputedStyle(s), w = (e, k) => getComputedStyle(e)[k];
      return { n: bs.length, ov: c.overflow, rad: c.borderTopLeftRadius, top0: bs.slice(0, 6).every(e => w(e, 'borderTopWidth') === '0px'), topRest: bs.slice(6).every(e => w(e, 'borderTopWidth') === '1px'),
        r6: bs.filter((e, i) => (i + 1) % 6 === 0).every(e => w(e, 'borderRightWidth') === '0px'), rRest: bs.filter((e, i) => (i + 1) % 6 !== 0).every(e => w(e, 'borderRightWidth') === '1px'),
        onRight: (() => { const o = s.querySelector('button.on'); return o ? w(o, 'borderRightColor') : null; })(), kids: bs.every(e => w(e, 'borderTopLeftRadius') === '0px') }; });
    A('its dividers still encode six columns: none on top of row one, none right of every sixth, 1px elsewhere, as before', g.n >= 12 && g.top0 && g.topRest && g.r6 && g.rRest, JSON.stringify(g));
    A('its armed cell keeps the --line divider (rgba(255, 255, 255, 0.1)), not an accent one', g.onRight === 'rgba(255, 255, 255, 0.1)', g.onRight);
    A('the grid clips square cells inside a 16px corner (overflow hidden)', g.ov === 'hidden' && g.rad === '16px' && g.kids, JSON.stringify({ ov: g.ov, rad: g.rad, kids: g.kids }));
    await close(p);
  });

  for (const light of [false, true]) {
    await section('V24 — the dock\'s segments, action buttons and sub-tabs (' + (light ? 'light' : 'dark') + ')', async () => {
      const p = await boot(b, { width: 1600, height: 900 }, { light });
      await gen(p, 12345, 256); await settle(p);
      const C = T(light), segs = [], acts = [], fills = [];
      /* READ: every drawn control of a kind, with its type, its on-state and its chip (::before). The fill check is
         structural — "is any armed control's ground the accent colour" — so it compares with the PAGE's own
         --accent, whatever its value (a literal would let the old light accent of a before-build slip past); the
         accent's value itself is pinned as a literal in V02/V08/V09 above. */
      const take = () => p.evaluate(() => { const vis = e => e.getClientRects().length && getComputedStyle(e).visibility !== 'hidden';
        const hx = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim(), acc = 'rgb(' + [1, 3, 5].map(i => parseInt(hx.substr(i, 2), 16)).join(', ') + ')';
        const cs = e => { const c = getComputedStyle(e), b4 = getComputedStyle(e, '::before'); return { id: ((e.closest('[id]') || {}).id || '') + ' ' + e.textContent.trim().slice(0, 16), ff: c.fontFamily, fs: c.fontSize, fw: c.fontWeight, ls: c.letterSpacing, tt: c.textTransform,
          on: e.classList.contains('on'), tool: e.hasAttribute('data-civtool') || !!e.closest('#sculptFeatureSeg'), cat: !!e.closest('#worldCatSeg'), bg: c.backgroundColor, fg: c.color, bc: b4.content, bbg: b4.backgroundColor, bbd: b4.borderTopWidth + ' ' + b4.borderTopStyle + ' ' + b4.borderTopColor }; };
        const fill = [...document.querySelectorAll('.on,[aria-pressed="true"],[aria-selected="true"]')].filter(e => vis(e) && getComputedStyle(e).backgroundColor === acc && !(e.hasAttribute('data-civtool') || e.closest('#sculptFeatureSeg'))).map(e => (e.id || e.className) + ' "' + e.textContent.trim().slice(0, 14) + '"');
        return { segs: [...document.querySelectorAll('.dock .seg button')].filter(vis).map(cs), acts: [...document.querySelectorAll('.dock .btnrow > button')].filter(vis).map(cs), fill }; });
      for (const d of ['world', 'civ', 'carto', 'explore']) {
        await click(p, '#domainRail [data-domain="' + d + '"]');
        await p.evaluate(() => { for (const x of document.querySelectorAll('.dock-left details')) x.open = true; });   /* SET-UP: every section open, for the READ */
        await sleep(150); const r = await take(); segs.push(...r.segs); acts.push(...r.acts); fills.push(...r.fill);
      }
      await click(p, '#domainRail [data-domain="world"]'); await click(p, '#worldCatSeg [data-wcat="sculpt"]'); await click(p, '#sculptFeatureSeg [data-sfeat="freehand"]'); await sleep(300);
      { const r = await take(); fills.push(...r.fill); }
      await click(p, '#worldCatSeg [data-wcat="terrain"]');
      await click(p, '#mbRecordsBtn'); await click(p, '#mbRecords [data-cmd="factions"]'); await sleep(500); { const r = await take(); fills.push(...r.fill); } await click(p, '#cfmCloseBtn');
      await click(p, '#mbPrefsBtn'); await click(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(300); { const r = await take(); fills.push(...r.fill); } await click(p, '#settingsClose');
      const mono = x => /^ui-monospace/.test(x.ff);
      const segBad = segs.filter(x => !mono(x) || x.fs !== '10px' || x.fw !== '500' || (!x.cat && x.ls !== '0.8px'));
      A('every segment in the docks takes the design\'s chip type: mono 10px, weight 500, 0.8px tracking (' + segs.length + ' segments)', segs.length >= 26 && segBad.length === 0, segBad.slice(0, 4).map(x => x.id + ': ' + x.ff.split(',')[0] + ' ' + x.fs + ' ' + x.fw + ' ' + x.ls).join(' | '));
      const onW = segs.filter(x => x.on && !x.tool), onT = segs.filter(x => x.on && x.tool);
      const washBad = onW.filter(x => x.bg !== C.panel2 || x.fg !== C.accent || x.bbg !== C.wash2 || x.bbd !== '1px solid ' + C.accent);
      A('an armed segment is washed: a chip of wash2 (' + C.wash2 + ') with a 1px accent edge, accent ink, no fill (' + onW.length + ' armed)', onW.length >= 4 && washBad.length === 0, washBad.slice(0, 3).map(x => x.id + ': ' + x.bg + ' ' + x.fg + ' ' + x.bbg + ' ' + x.bbd).join(' | '));
      const ink = await p.evaluate(() => { const e = document.querySelector('#extentSeg button.on'); if (!e) return null; const c = getComputedStyle(e), b4 = getComputedStyle(e, '::before');
        const g = __aud.over(__aud.parse(b4.backgroundColor), __aud.parse(c.backgroundColor)); return +__aud.ratio(__aud.parse(c.color), g).toFixed(2); });
      A('…an armed segment\'s ink measures ≥ 4.5:1 on what it sits on — the chip, wash2 over --panel2 (v2.73\'s fill passed too, as before)', ink >= 4.5, ink);
      const toolBad = onT.filter(x => x.bg !== C.accent || x.fg !== C.accInk || x.bc !== 'none');
      A('an armed TOOL-MODE segment keeps the fill, as before (owner default Q5): accent with --accInk (' + onT.length + ' tool palettes)', onT.length >= 3 && toolBad.length === 0, toolBad.slice(0, 3).map(x => x.id + ': ' + x.bg + ' ' + x.fg + ' ' + x.bc).join(' | '));
      A('no other armed control draws the solid accent fill — docks, the sculpt bar, the Settlements window, Settings', fills.length === 0, [...new Set(fills)].slice(0, 6).join(' | '));
      const actBad = acts.filter(x => !mono(x) || x.tt !== 'uppercase' || x.fw !== '500' || !/^(10px|10\.5px)$/.test(x.fs));
      A('every action button in a dock is mono, uppercase, weight 500 (' + acts.length + ' buttons)', acts.length >= 8 && actBad.length === 0, actBad.slice(0, 4).map(x => x.id + ': ' + x.ff.split(',')[0] + ' ' + x.tt + ' ' + x.fs).join(' | '));
      await click(p, '#domainRail [data-domain="civ"]');
      const st = await p.evaluate(() => [...document.querySelectorAll('#civSubBar .subtab')].map(e => { const c = getComputedStyle(e); return /^ui-monospace/.test(c.fontFamily) && c.fontSize === '10px' && c.fontWeight === '500' && c.textTransform === 'uppercase'; }));
      A('the five CIVIL sub-tabs are mono 10px, weight 500, uppercase — the armed one too', st.length === 5 && st.every(Boolean), JSON.stringify(st));
      await click(p, '#domainRail [data-domain="world"]');
      const full = id => p.evaluate(i => { const bb = document.getElementById(i); if (!bb || !bb.getClientRects().length) return null; const row = bb.parentElement, cs = getComputedStyle(row); return { w: Math.round(bb.getBoundingClientRect().width), row: Math.round(row.getBoundingClientRect().width - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)) }; }, id);
      const bk = await full('bakeAllBtn');
      A('Bake ALL levels runs the dock\'s full width (its row\'s content box)', !!bk && Math.abs(bk.w - bk.row) <= 1, JSON.stringify(bk));
      if (!light) {
        /* the smallest bake (LOD 0–2), by the select's own API and a real click on Bake; then Un-finalize, by a click */
        await p.selectOption('#bakeAllDepth', { index: 0 }).catch(() => {});
        await click(p, '#bakeAllBtn'); await idle(p); await sleep(400);
        const uf = await full('unfinalizeBtn');
        A('after a real bake, Un-finalize runs the dock\'s full width too', !!uf && Math.abs(uf.w - uf.row) <= 1, JSON.stringify(uf));
        if (uf) { await click(p, '#unfinalizeBtn'); await idle(p); }
      }
      await close(p);
    });
  }

  for (const light of [false, true]) {
    await section('V25 — one section label (' + (light ? 'light' : 'dark') + ')', async () => {
      const p = await boot(b, { width: 1600, height: 900 }, { light });
      const C = T(light);
      const SEL = ['.sec h2', '.dock .sublabel', '.dock .sublabel-grp', '.tl-lab', '.layers-fab .lp-grp', '.al-rail .al-rail-h', '#onboard .su-label', '.cv-section h3', '.re-col h3'];
      const seen = {};
      /* READ: the first drawn instance of each selector */
      const look = async () => Object.assign(seen, await p.evaluate(sels => { const o = {}; for (const s of sels) { const e = [...document.querySelectorAll(s)].find(x => x.getClientRects().length && getComputedStyle(x).visibility !== 'hidden');
        if (e) { const c = getComputedStyle(e); o[s] = [c.fontFamily.split(',')[0], c.fontSize, c.fontWeight, c.letterSpacing, c.textTransform, c.color].join(' | '); } } return o; }, SEL));
      await click(p, '#obGenerate'); await sleep(300); await look();
      await gen(p, 12345, 256); await settle(p);
      await p.evaluate(() => { const y = document.getElementById('civTlYear'); y.value = 100; document.getElementById('civTlAddYearBtn').click(); y.value = 250; document.getElementById('civTlAddYearBtn').click(); });   /* SET-UP: two recorded years */
      await click(p, '#tlStripToggle'); await sleep(250);
      for (const d of ['world', 'civ']) { await click(p, '#domainRail [data-domain="' + d + '"]'); await p.evaluate(() => { for (const x of document.querySelectorAll('.dock-left details')) x.open = true; }); await look(); }
      await click(p, '#layersBtn'); await sleep(250); await look(); await click(p, '#layersBtn'); await sleep(150);
      await click(p, '#assetsHeaderBtn'); await sleep(800); await look(); await click(p, '#assetsHeaderBtn'); await sleep(400);
      await click(p, '#domainRail [data-domain="civ"]');
      const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(600); }
      if (await click(p, '#placeEditPopup #peCityOpen')) { await sleep(1500); await look(); await click(p, '#cityViewerModal .cv-close'); }
      await click(p, '#placeEditPopup .si-close');
      await click(p, '#domainRail [data-domain="explore"]'); await click(p, '[data-civtool="route"]');
      await p.evaluate(() => { const S = state.places.filter(q => q.category === 'settlement').sort((a, b) => b.pop - a.pop);   /* SET-UP: a priced route */
        for (let i = 0; i < S.length; i++) for (let k = i + 1; k < Math.min(S.length, i + 6); k++) { _civWaypoints = [[S[i].x, S[i].y], [S[k].x, S[k].y]]; _civCommitRoute(); const jn = civJourneys[civJourneys.length - 1]; const pl = _jpPlan(jn); if (pl && jpJourneyCost(pl)) return; } });
      await idle(p);
      if (await click(p, '#jpOpenEditorBtn')) { await p.waitForFunction(() => /Priced in/.test(document.getElementById('routeEditorModal').textContent), null, { timeout: 15000 }).catch(() => {}); await look(); await click(p, '#reCloseBtn'); }
      const want = 'ui-monospace | 9.5px | 500 | 1.9px | uppercase | ' + C.dim;
      const missing = SEL.filter(s => !seen[s]), off = SEL.filter(s => seen[s] && seen[s] !== want);
      A('the nine section-label selectors compute one style: ' + want, missing.length === 0 && off.length === 0, (missing.length ? 'not reached: ' + missing.join(', ') + '; ' : '') + off.map(s => s + ' = ' + seen[s]).join(' || '));
      const loc = await p.evaluate(() => ['dockLeftPath', 'toolOptsCtx'].map(id => getComputedStyle(document.getElementById(id)).color));
      A('a location label keeps the accent (the dock\'s path, the tool bar\'s context), as before', loc.every(c => c === C.accent), loc.join(' '));
      await close(p);
    });
  }

  await section('V29 — the timeline strip\'s toggles are washed chips', async () => {
    const p = await boot(b, { width: 1600, height: 900 });
    await gen(p, 12345, 256); await settle(p);
    const C = T(false);
    await click(p, '#tlStripToggle'); await sleep(300);
    /* two recorded years by real input: the year field, typed, and Add year, clicked */
    for (const y of ['100', '250']) { await click(p, '#civTlYear'); await p.keyboard.press('Control+A'); await p.keyboard.type(y); await click(p, '#civTlAddYearBtn'); await sleep(300); }
    const py = await p.evaluate(() => [...document.querySelectorAll('#civTimelinePanel .pill')].filter(e => e.getClientRects().length).map(e => { const c = getComputedStyle(e), x = e.querySelector('.pill-x') || e.querySelector('span'); return { on: e.classList.contains('on'), bg: c.backgroundColor, bd: c.borderTopColor, fg: c.color, rad: c.borderTopLeftRadius, x: x ? (() => { const cc = getComputedStyle(x); const o = +cc.opacity; const f = __aud.parse(cc.color), g = __aud.parse(getComputedStyle(document.getElementById('tlStrip')).backgroundColor); const bgp = __aud.over(__aud.parse(c.backgroundColor), g); return +__aud.ratio(__aud.over({ ...f, a: o }, bgp), bgp).toFixed(2); })() : null }; }));
    const on = py.find(x => x.on);
    A('the active recorded year is a washed pill: wash2, an accent edge and accent ink (' + py.length + ' years)', py.length >= 2 && !!on && on.bg === C.wash2 && on.bd === C.accent && on.fg === C.accent && on.rad === '999px', JSON.stringify(on));
    A('every year pill\'s ✕ measures ≥ 4.5:1 on its pill (it was the label at .6 opacity)', py.length >= 2 && py.every(x => x.x >= 4.5), py.map(x => x.x).join(', '));
    const ch = await p.evaluate(() => [...document.querySelectorAll('#tlStrip .tl-filters label.chk')].map(l => { const c = getComputedStyle(l), i = l.querySelector('input'); return { rad: c.borderTopLeftRadius, bd: c.borderTopWidth + ' ' + c.borderTopStyle, ff: c.fontFamily.split(',')[0], fs: c.fontSize, fg: c.color, ck: i.checked, inDoc: !!i && getComputedStyle(i).display !== 'none' }; }));
    A('the three filters are chips: a pill, a 1px edge, mono 10px, --muted ink (' + C.muted + ') at rest', ch.length === 3 && ch.every(x => x.rad === '999px' && x.bd === '1px solid' && x.ff === 'ui-monospace' && x.fs === '10px' && x.fg === C.muted && !x.ck && x.inDoc), JSON.stringify(ch[0]));
    await click(p, '#civTlYear');
    let reached = false; for (let i = 0; i < 5 && !reached; i++) { await p.keyboard.press('Tab'); await sleep(100); reached = await p.evaluate(() => !!document.activeElement && document.activeElement.id === 'explTlExistOnly'); }
    A('Tab still reaches the checkbox inside the first chip, as before (Exist only)', reached);
    const ring = await p.evaluate(() => { const c = getComputedStyle(document.getElementById('explTlExistOnly').closest('label')); return c.outlineStyle + ' ' + c.outlineWidth + ' ' + c.outlineColor; });
    A('…and the chip draws the focus ring (2px solid accent)', ring === 'solid 2px ' + C.accent, ring);
    await p.keyboard.press('Space'); await sleep(200);
    const sp = await p.evaluate(() => { const i = document.getElementById('explTlExistOnly'), l = i.closest('label'), c = getComputedStyle(l); return { ck: i.checked, bg: c.backgroundColor, bd: c.borderTopColor, fg: c.color, txt: l.textContent.trim().replace(/\s+/g, ' ') }; });
    A('Space toggles it, and the checked chip is washed (wash2, accent edge and ink)', sp.ck && sp.bg === C.wash2 && sp.bd === C.accent && sp.fg === C.accent, JSON.stringify(sp));
    A('…and the chip keeps its label, as before', /Exist only/.test(sp.txt), sp.txt);
    await p.keyboard.press('Space'); await sleep(150);
    await click(p, '#tlStrip .tl-filters label.chk:nth-child(2)');
    A('a real click on the Ghost removed chip checks its box, as before', await p.evaluate(() => document.getElementById('explTlGhost').checked));
    await click(p, '#tlStrip .tl-filters label.chk:nth-child(2)');
    const pb = await p.evaluate(() => { const c = getComputedStyle(document.getElementById('explTlPlayBtn')); return { bg: c.backgroundColor, fg: c.color, ff: c.fontFamily.split(',')[0], tt: c.textTransform, t: document.getElementById('explTlPlayBtn').textContent }; });
    A('the play button is the washed accent control, mono uppercase, its label kept', pb.bg === C.wash2 && pb.fg === C.accent && pb.ff === 'ui-monospace' && pb.tt === 'uppercase' && /Animate/.test(pb.t), JSON.stringify(pb));
    await close(p);
  });

  for (const light of [false, true]) {
    await section('V30 — one window heading (' + (light ? 'light' : 'dark') + ')', async () => {
      const p = await boot(b, { width: 1600, height: 900 }, { light });
      const C = T(light);
      const hd = sel => p.evaluate(s => { const e = [...document.querySelectorAll(s)].find(x => x.getClientRects().length); if (!e) return null; const c = getComputedStyle(e); return [c.fontFamily.split(',')[0], c.fontSize, c.fontWeight, c.letterSpacing, c.textTransform, c.color].join(' | '); }, sel);
      const got = {};
      got.start = await hd('#onboard h3');
      await click(p, '#obGenerate'); await sleep(300); got.newWorld = await hd('#obStepGenerate h3');
      await gen(p, 12345, 256); await settle(p);
      await click(p, '#mbRecordsBtn'); await click(p, '#mbRecords [data-cmd="factions"]'); await sleep(500); got.settlements = await hd('.cfm-body h2'); await click(p, '#cfmCloseBtn');
      await click(p, '#mbPrefsBtn'); await click(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(300); got.settings = await hd('.set-head h3'); await click(p, '#settingsClose');
      await click(p, '#mbHelpBtn'); await click(p, '#mbHelp [data-cmd="credits"]'); await sleep(300); got.credits = await hd('#creditsModal h2'); await click(p, '#creditsClose');
      await click(p, '#domainRail [data-domain="explore"]'); await click(p, '[data-civtool="route"]');
      await p.evaluate(() => { const S = state.places.filter(q => q.category === 'settlement').sort((a, b) => b.pop - a.pop);   /* SET-UP: a priced route */
        for (let i = 0; i < S.length; i++) for (let k = i + 1; k < Math.min(S.length, i + 6); k++) { _civWaypoints = [[S[i].x, S[i].y], [S[k].x, S[k].y]]; _civCommitRoute(); const jn = civJourneys[civJourneys.length - 1]; const pl = _jpPlan(jn); if (pl && jpJourneyCost(pl)) return; } });
      await idle(p);
      if (await click(p, '#jpOpenEditorBtn')) { await sleep(800); got.routeEditor = await hd('.re-summary h2'); await click(p, '#reCloseBtn'); }
      const want = 'ui-monospace | 11px | 500 | 2.2px | uppercase | ' + C.accent;
      const off = Object.entries(got).filter(([, v]) => v !== want);
      A('the windows\' headings compute one style — ' + want + ' (' + Object.keys(got).length + ' of 6 reached)', Object.keys(got).length === 6 && off.length === 0, off.map(([k, v]) => k + ' = ' + v).join(' || '));
      await click(p, '#domainRail [data-domain="civ"]');
      const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(600); }
      let cv = null; if (await click(p, '#placeEditPopup #peCityOpen')) { await sleep(1500); cv = await p.evaluate(() => { const e = document.querySelector('#cvInfoPanel h2'); return e ? getComputedStyle(e).fontSize : null; }); }
      A('the City viewer\'s place name keeps its 16px (a proper noun, exempt), as before', cv === '16px', cv);
      await close(p);
    });
  }

  await section('the truncation census against the before file', async () => {
    if (!BEFORE) { console.log('  (skipped: no before file — give one as the second argument; not a verdict)'); return; }
    /* the views a person reaches: every dock with its sections open (CIVIL's five sub-tabs), a place's popup, the
       Settlements window's four tabs, Settings' five pages, Credits, File ▸ Export, the asset library, the strip
       expanded with two years, and the New World card. The Settlements TABLE (#stTableHead, #stSpacer) is left out:
       the fix pass made every one of its cells ellipsise by design, so its head lines up with its rows (V01; at 1366
       "Faction" and "Econ. role" are its two cut heads), and which rows are on screen moves with any band above it.
       A row is keyed in lower case: text-transform changes innerText, and the same row in sentence case and in
       capitals is the same row. */
    const census = async (file, vp, seed) => {
      const p = await boot(b, vp, { file }); const t = new Set(), w = new Set();
      const take = async () => { const r = await p.evaluate(() => ({ t: __aud.truncs(), w: __aud.wraps() })); r.t.filter(k => !/#stSpacer|#stTableHead/.test(k)).forEach(k => t.add(k.toLowerCase())); r.w.forEach(k => w.add(k.toLowerCase())); };
      await click(p, '#obGenerate'); await sleep(300); await take();
      await gen(p, seed, 256); await settle(p);
      await p.evaluate(() => { const y = document.getElementById('civTlYear'); y.value = 100; document.getElementById('civTlAddYearBtn').click(); y.value = 250; document.getElementById('civTlAddYearBtn').click(); });   /* SET-UP: two recorded years */
      for (const d of ['world', 'civ', 'carto', 'explore']) { await click(p, '#domainRail [data-domain="' + d + '"]'); await p.evaluate(() => { for (const x of document.querySelectorAll('.dock-left details')) x.open = true; }); await take(); }
      await click(p, '#domainRail [data-domain="world"]'); await click(p, '#worldCatSeg [data-wcat="sculpt"]'); await click(p, '#sculptFeatureSeg [data-sfeat="freehand"]'); await sleep(300); await take(); await click(p, '#worldCatSeg [data-wcat="terrain"]');
      await click(p, '#domainRail [data-domain="civ"]');
      for (const s of ['generation', 'settlements', 'economy', 'statistics', 'factions']) { await click(p, '[data-civsub="' + s + '"]'); await sleep(200); await take(); }
      await click(p, '#tlStripToggle'); await sleep(250); await take(); await click(p, '#tlStripToggle');
      const at = await placeOnMap(p); if (at) { await p.mouse.click(at.x, at.y); await sleep(600); await take(); await click(p, '#placeEditPopup .si-close'); }
      await click(p, '#mbRecordsBtn'); await click(p, '#mbRecords [data-cmd="factions"]'); await sleep(500);
      for (const pm of ['global', 'factions', 'places', 'types']) { await click(p, '#civPmBar button[data-pm="' + pm + '"]'); await sleep(300); await take(); }
      await click(p, '#cfmCloseBtn');
      await click(p, '#mbPrefsBtn'); await click(p, '#mbPrefs [data-cmd="setPerf"]'); await sleep(300);
      for (const pg of ['perf', 'storage', 'appearance', 'workspace', 'about']) { await click(p, '#setNav [data-setpage="' + pg + '"]'); await sleep(200); await take(); }
      await click(p, '#settingsClose');
      await click(p, '#mbHelpBtn'); await click(p, '#mbHelp [data-cmd="credits"]'); await sleep(300); await take(); await click(p, '#creditsClose');
      await click(p, '#fileMenuBtn'); { const er = await rect(p, '#mbExportRow'); if (er) { await p.mouse.move(er.l + er.w / 2, er.t + er.h / 2); await sleep(350); } } await take(); await p.keyboard.press('Escape');
      await click(p, '#assetsHeaderBtn'); await sleep(800); await p.mouse.move(3, 300); await take(); await click(p, '#assetsHeaderBtn'); await sleep(400);
      await close(p); return { t, w };
    };
    for (const vp of [{ width: 1366, height: 768 }, { width: 1600, height: 900 }]) for (const seed of [12345, 777]) {
      const a = await census(BEFORE, vp, seed), z = await census(FILE, vp, seed);
      const fresh = [...z.t].filter(k => !a.t.has(k));
      A('at ' + vp.width + '×' + vp.height + ', seed ' + seed + ': no truncation the before build did not have (' + z.t.size + ' here, ' + a.t.size + ' before)', fresh.length === 0, fresh.slice(0, 6).join(' | '));
      console.log('  rows of buttons that wrap at ' + vp.width + ', seed ' + seed + ' (' + z.w.size + '; before ' + a.w.size + '):');
      for (const k of z.w) console.log('    ' + (a.w.has(k) ? 'as before  ' : 'NEW        ') + k);
    }
  });

  await b.close();
  console.log('\n' + (errors.length ? 'page errors:\n  ' + errors.slice(0, 10).join('\n  ') : 'no page errors'));
  A('no page errors, as before', errors.length === 0, errors.length);
  console.log('\nviewports (' + VIEWPORTS.size + '): ' + [...VIEWPORTS].join(', '));
  console.log('\n' + pass + ' passed, ' + fail + ' failed' + (skipped ? ', ' + skipped + ' sections skipped (PROBE_ONLY — not a verdict)' : ''));
  process.exit(fail ? 1 : (skipped ? 3 : 0));
})();
