// Ruling DZ: the tool-options bar holds the armed tool's settings only. Checks, on the desktop (1600 x 1000) and the
// tablet (1194 x 834, Live.dc.html), that WORLD's idle bar has no Generate, New seed, Center landmasses or Bake button,
// shows its context and hint, keeps the quick-bar pins, and that the WORLD dock still offers Generate world and New
// seed; then that a sculpt tool still puts its brush settings on the bar. Writes dz-*.png captures. Exits 1 on a
// failed check or a page error.
const { chromium } = require('playwright');
const OUT = '/home/user/Cartalith_GDT/cartalith-native/docs/evidence/desktop-shell-2026-10-31/';
const BASE = 'http://127.0.0.1:8770/';
let bad = 0;
const check = (ok, what) => { console.log((ok ? 'ok   ' : 'FAIL ') + what); if (!ok) bad += 1; };
// Every element under the canvas, shadow roots included, so a check sees what the page draws.
const texts = (p, top, bottom) => p.evaluate(([y0, y1]) => {
  const all = []; const walk = (n) => { if (!n) return; if (n.shadowRoot) walk(n.shadowRoot); for (const k of n.children || []) { all.push(k); walk(k); } }; walk(document.body);
  return all.filter((e) => { if (!(e.tagName === 'BUTTON' || e.tagName === 'SPAN' || e.tagName === 'DIV')) return false; const r = e.getBoundingClientRect(); return r.width && r.height && r.top >= y0 && r.bottom <= y1 && e.children.length === 0; })
    .map((e) => (e.tagName === 'BUTTON' ? 'BUTTON:' : '') + (e.textContent || '').trim()).filter((x) => x && x !== 'BUTTON:');
}, [top, bottom]);
(async () => {
  const b = await chromium.launch(); const errs = [];
  const watch = (p, tag) => { p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message)); p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); }); };
  for (const [file, w, h, tag] of [['Desktop.dc.html', 1600, 1000, 'desktop'], ['Live.dc.html', 1194, 834, 'tablet']]) {
    const p = await b.newPage({ viewport: { width: w, height: h } }); watch(p, tag);
    await p.goto(BASE + file); await p.waitForTimeout(3200); const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
    // The tool-options bar is the second band from the top on both frames; read the band below the menu bar.
    const band = await texts(p, tag === 'desktop' ? 80 : 50, tag === 'desktop' ? 122 : 125);
    console.log(tag + ' bar:', band.join(' | '));
    for (const gone of ['New seed', 'Center landmasses', 'Bake ALL & finalize', 'GENERATE']) check(!band.includes('BUTTON:' + gone) && !band.includes(gone), tag + ': no "' + gone + '" on the bar');
    check(band.some((x) => /GENERATE · WORLD/.test(x)), tag + ': the context name stays');
    check(band.some((x) => /selects/.test(x)), tag + ': the one-line hint stays');
    if (tag === 'desktop') check(band.some((x) => /Vault|Planner|Assistant/.test(x)), tag + ': the quick-bar pins stay');
    await p.screenshot({ path: OUT + 'dz-1-' + tag + '-bar.png', clip: { x: 0, y: 0, width: w, height: tag === 'desktop' ? 130 : 130 } });
    // The WORLD dock keeps every action the bar gave up.
    const dock = await texts(p, 120, h);
    check(dock.some((x) => /GENERATE WORLD/i.test(x)), tag + ': the WORLD dock still has Generate world');
    check(dock.some((x) => /NEW SEED/i.test(x)), tag + ': the WORLD dock still has New seed');
    await p.close();
  }
  // A sculpt tool still puts its brush settings on the bar (desktop).
  const p = await b.newPage({ viewport: { width: 1600, height: 1000 } }); watch(p, 'sculpt');
  await p.goto(BASE + 'Desktop.dc.html'); await p.waitForTimeout(3200);
  // WORLD ▸ Sculpt opens the node; its brush tools are buttons named with their key ("Freehand (F)", "Paint (B)").
  await p.getByRole('button', { name: 'WORLD ▸ Sculpt' }).click(); await p.waitForTimeout(500);
  for (const [name, head, shot] of [['Freehand (F)', /SCULPT ·/, 'dz-2-desktop-sculpt-bar.png'], ['Paint (B)', /PAINT ·/, 'dz-3-desktop-paint-bar.png']]) {
    await p.getByRole('button', { name }).first().click(); await p.waitForTimeout(450);
    const band = await texts(p, 80, 122);
    console.log(name + ' bar:', band.join(' | '));
    check(band.some((x) => head.test(x)), name + ': the bar names the armed brush and shows its settings');
    check(!band.some((x) => /New seed|Center landmasses|Bake ALL/.test(x)), name + ': still no generation buttons');
    await p.screenshot({ path: OUT + shot, clip: { x: 0, y: 0, width: 1600, height: 130 } });
  }
  console.log(errs.length ? errs.join('\n') : 'no page errors');
  await b.close();
  process.exit(bad || errs.length ? 1 : 0);
})();
