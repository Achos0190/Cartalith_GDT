// Shared helpers for the v2.72 visual audit (read-only; writes only into this folder).
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const A = '/tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-visual';
const SHOTS = path.join(A, 'shots');
const DATA = path.join(A, 'data');
fs.mkdirSync(SHOTS, { recursive: true });
fs.mkdirSync(DATA, { recursive: true });
const V272 = path.join(A, 'v272.html');
const V271 = path.join(A, 'v271.html');

function log(...a) { const s = `[${new Date().toISOString().slice(11, 19)}] ` + a.join(' '); console.log(s); fs.appendFileSync(path.join(A, 'run.log'), s + '\n'); }

async function boot(browser, file, vp, opts = {}) {
  const ctx = await browser.newContext({ viewport: vp, deviceScaleFactor: opts.dpr || 1, isMobile: !!opts.mobile, hasTouch: !!opts.mobile });
  const p = await ctx.newPage();
  p._errs = [];
  p.on('pageerror', e => p._errs.push('pageerror: ' + e.message));
  p.on('console', m => { if (m.type() === 'error') p._errs.push('console: ' + m.text().slice(0, 200)); });
  p.on('dialog', d => { p._errs.push('dialog: ' + d.message().slice(0, 120)); d.accept().catch(() => {}); });
  await p.addInitScript(l => { try { localStorage.clear(); localStorage.setItem('cartalith_theme', l ? 'light' : 'dark'); } catch (_) {} }, !!opts.light);
  await p.goto('file://' + file);
  await p.waitForFunction(() => typeof window.generate === 'function' && document.getElementById('onboard'), { timeout: 90000 });
  await p.waitForTimeout(800);
  return p;
}

async function idle(p, extra = 300) {
  for (let i = 0; i < 600; i++) {
    const d = await p.evaluate(async () => { try { await _busyChain; } catch (_) {} return (typeof _busyDepth !== 'undefined') ? _busyDepth : 0; });
    if (d === 0) break;
    await p.waitForTimeout(250);
  }
  await p.waitForTimeout(extra);
}

// The start card -> New World -> Generate world, by real clicks only.
async function genByClicks(p, seed, res, shotPrefix) {
  await p.waitForSelector('#obGenerate', { state: 'visible', timeout: 30000 });
  if (shotPrefix) await shot(p, shotPrefix + '_startcard');
  await p.click('#obGenerate');
  await p.waitForSelector('#obStepGenerate.on', { timeout: 10000 });
  await p.waitForTimeout(200);
  if (shotPrefix) await shot(p, shotPrefix + '_newworld');
  if (res) await p.click(`#suResSeg button[data-w="${res}"]`);
  await p.fill('#suSeedN', String(seed));
  const t0 = Date.now();
  await p.click('#suGenCommit', { noWaitAfter: true, timeout: 120000 });
  await p.waitForTimeout(400);
  await idle(p, 600);
  log('generated seed', seed, 'res', res || 'default', 'in', ((Date.now() - t0) / 1000).toFixed(1), 's');
}

async function shot(p, name, opts = {}) {
  const f = path.join(SHOTS, name + '.png');
  await p.screenshot({ path: f, ...opts });
  log('shot', path.basename(f));
  return f;
}
async function elShot(p, sel, name) {
  const h = await p.$(sel); if (!h) { log('elShot: no', sel); return null; }
  const f = path.join(SHOTS, name + '.png');
  await h.screenshot({ path: f }); log('shot', path.basename(f)); return f;
}
async function clipShot(p, clip, name) {
  const f = path.join(SHOTS, name + '.png');
  await p.screenshot({ path: f, clip }); log('shot', path.basename(f)); return f;
}
function save(name, obj) { fs.writeFileSync(path.join(DATA, name + '.json'), JSON.stringify(obj, null, 1)); log('data', name + '.json'); }

// In-page audit helpers, installed once per page.
async function installAudit(p) {
  await p.evaluate(() => {
    if (window.__aud) return;
    const parse = s => { const m = s.match(/rgba?\(([^)]+)\)/); if (!m) return null; const v = m[1].split(/[ ,\/]+/).filter(Boolean).map(Number); return { r: v[0], g: v[1], b: v[2], a: v.length > 3 ? v[3] : 1 }; };
    const over = (f, b) => ({ r: f.r * f.a + b.r * (1 - f.a), g: f.g * f.a + b.g * (1 - f.a), b: f.b * f.a + b.b * (1 - f.a), a: 1 });
    const lum = c => { const t = x => { x /= 255; return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4); }; return 0.2126 * t(c.r) + 0.7152 * t(c.g) + 0.0722 * t(c.b); };
    const ratio = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
    const hex = c => '#' + [c.r, c.g, c.b].map(x => Math.round(x).toString(16).padStart(2, '0')).join('');
    const vis = el => { if (!el.isConnected) return false; const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; for (let e = el; e && e.nodeType === 1; e = e.parentElement) { const cs = getComputedStyle(e); if (cs.display === 'none' || cs.visibility === 'hidden' || +cs.opacity === 0) return false; } return r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth; };
    // effective background behind an element: composite ancestors' background-colors; note gradients/images/canvases
    const bgOf = el => {
      const stack = []; let flag = '';
      for (let e = el; e && e.nodeType === 1; e = e.parentElement) {
        const cs = getComputedStyle(e);
        if (cs.backgroundImage && cs.backgroundImage !== 'none') flag = flag || ('image@' + (e.id || e.className || e.tagName));
        const c = parse(cs.backgroundColor); if (c && c.a > 0) { stack.push(c); if (c.a >= 0.999) break; }
        if (e.classList && (e.classList.contains('canvas-wrap'))) { flag = flag || 'over-map'; }
      }
      let base = parse(getComputedStyle(document.body).backgroundColor) || { r: 0, g: 0, b: 0, a: 1 };
      for (let i = stack.length - 1; i >= 0; i--) base = over(stack[i], base);
      return { c: base, flag };
    };
    const opac = el => { let o = 1; for (let e = el; e && e.nodeType === 1; e = e.parentElement) o *= +getComputedStyle(e).opacity; return o; };
    const desc = el => { let s = el.tagName.toLowerCase(); if (el.id) s += '#' + el.id; else if (el.className && typeof el.className === 'string') s += '.' + el.className.trim().split(/\s+/).slice(0, 2).join('.'); const pid = el.closest('[id]'); if (pid && pid !== el) s = pid.tagName.toLowerCase() + '#' + pid.id + ' ' + s; return s; };
    const region = el => { const m = [['#menuBar,header', 'menubar'], ['#toolOpts', 'toolopts'], ['.dock-left', 'dock-left'], ['.dock-right', 'context'], ['#tlStrip', 'timeline'], ['#statusBar', 'status'], ['.domain-rail', 'rail'], ['.canvas-wrap', 'map'], ['#settingsModal', 'settings'], ['#creditsModal', 'credits'], ['#onboard', 'startcard'], ['#cityViewerModal', 'cityviewer'], ['#routeEditorModal', 'routeeditor'], ['#civFactionsModal', 'factions'], ['#assetLibrary', 'assetlib'], ['#placeEditPopup', 'placepopup']]; for (const [s, n] of m) if (el.closest(s)) return n; return 'other'; };
    window.__aud = { parse, over, lum, ratio, hex, vis, bgOf, opac, desc, region };
    window.__aud.contrast = (rootSel) => {
      const root = rootSel ? document.querySelector(rootSel) : document.body; const out = [];
      if (!root) return out;
      const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
      const seen = new Set();
      while (walker.nextNode()) {
        const t = walker.currentNode; if (!t.nodeValue.trim()) continue; const el = t.parentElement; if (!el || seen.has(el)) continue; seen.add(el);
        if (!vis(el)) continue;
        if (el.closest('option')) continue;
        const cs = getComputedStyle(el); const fg = parse(cs.color); if (!fg) continue;
        const b = bgOf(el); const o = opac(el);
        let f = over(fg, b.c); if (o < 1) { f = over({ ...f, a: o }, parse(getComputedStyle(document.body).backgroundColor)); }
        let bb = b.c; if (o < 1) bb = over({ ...b.c, a: o }, parse(getComputedStyle(document.body).backgroundColor));
        const cr = ratio(f, bb); const px = parseFloat(cs.fontSize); const bold = +cs.fontWeight >= 600; const large = px >= 24 || (bold && px >= 18.66);
        const dis = !!el.closest(':disabled,[disabled],[aria-disabled="true"]');
        out.push({ el: desc(el), region: region(el), text: t.nodeValue.trim().slice(0, 40), fg: hex(f), bg: hex(bb), ratio: +cr.toFixed(2), px, w: cs.fontWeight, large, need: large ? 3 : 4.5, fail: cr < (large ? 3 : 4.5), dis, flag: b.flag, opacity: +o.toFixed(2) });
      }
      return out;
    };
    // text that is cut: ellipsis or hidden overflow on a box whose content is wider than it
    window.__aud.truncs = (rootSel) => {
      const root = rootSel ? document.querySelector(rootSel) : document.body; const out = [];
      if (!root) return out;
      for (const el of root.querySelectorAll('*')) {
        if (!vis(el)) continue; const cs = getComputedStyle(el);
        const txt = (el.innerText || '').trim(); if (!txt) continue;
        const hidX = cs.overflowX !== 'visible'; const ell = cs.textOverflow === 'ellipsis';
        if ((hidX || ell) && el.scrollWidth > el.clientWidth + 1 && el.children.length <= 6 && !/auto|scroll/.test(cs.overflowX)) out.push({ el: desc(el), region: region(el), text: txt.slice(0, 60), client: el.clientWidth, scroll: el.scrollWidth, ell });
        else if (/auto|scroll/.test(cs.overflowX) && el.scrollWidth > el.clientWidth + 1 && el.clientWidth > 0) out.push({ el: desc(el), region: region(el), text: txt.slice(0, 40), client: el.clientWidth, scroll: el.scrollWidth, scroller: true });
      }
      return out;
    };
    // elements that poke outside the window
    window.__aud.offscreen = (sel) => { const out = []; for (const el of document.querySelectorAll(sel || 'button,select,input,.mb-panel,.mb-subpanel,.dropdown,.place-edit-popup,.settle-info')) { if (!vis(el)) continue; const r = el.getBoundingClientRect(); if (r.left < -0.5 || r.top < -0.5 || r.right > innerWidth + 0.5 || r.bottom > innerHeight + 0.5) out.push({ el: desc(el), r: [r.left, r.top, r.right, r.bottom].map(Math.round) }); } return out; };
    // control style census
    window.__aud.controls = (rootSel) => {
      const root = rootSel ? document.querySelector(rootSel) : document.body; const out = [];
      if (!root) return out;
      for (const el of root.querySelectorAll('button,select,input,textarea,.pill,summary')) {
        if (!vis(el)) continue; const cs = getComputedStyle(el); const r = el.getBoundingClientRect();
        const inSeg = !!el.closest('.seg'); const type = el.tagName.toLowerCase() + (el.type && el.tagName === 'INPUT' ? '[' + el.type + ']' : '') + (inSeg ? '(seg)' : '') + (el.classList.contains('pill') ? '(pill)' : '');
        out.push({ el: desc(el), region: region(el), type, text: (el.innerText || el.value || '').trim().slice(0, 30), ff: cs.fontFamily.split(',')[0].replace(/"/g, ''), fs: cs.fontSize, ls: cs.letterSpacing, tt: cs.textTransform, fw: cs.fontWeight, h: Math.round(r.height), w: Math.round(r.width), pad: cs.padding, bg: cs.backgroundColor, fg: cs.color, bd: cs.borderTopColor + ' ' + cs.borderTopWidth, rad: cs.borderRadius, accent: cs.accentColor });
      }
      return out;
    };
    window.__aud.emoji = (rootSel) => {
      const root = rootSel ? document.querySelector(rootSel) : document.body; const out = []; if (!root) return out;
      const re = /[\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}\u{2B00}-\u{2BFF}\u{2300}-\u{23FF}\u{FE0F}]/u;
      const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
      while (walker.nextNode()) { const t = walker.currentNode; if (!re.test(t.nodeValue)) continue; const el = t.parentElement; if (!el || !vis(el)) continue; const m = [...t.nodeValue].filter(ch => re.test(ch) && ch !== '️'); out.push({ el: desc(el), region: region(el), glyphs: m.join(''), text: t.nodeValue.trim().slice(0, 50) }); }
      return out;
    };
  });
}

module.exports = { chromium, fs, path, A, SHOTS, DATA, V272, V271, log, boot, idle, genByClicks, shot, elShot, clipShot, save, installAudit };
