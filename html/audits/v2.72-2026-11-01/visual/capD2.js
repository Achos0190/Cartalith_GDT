// Computed styles of the design frame's controls (1366 frame, shown at 100 %), for the design-vs-v2.72 table.
const L = require('./lib.js');
const { log, save } = L;
(async () => {
  const b = await L.chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1600, height: 1000 } });
  const p = await ctx.newPage();
  await p.goto('http://127.0.0.1:8770/Desktop.dc.html'); await p.waitForTimeout(3500);
  await p.locator('button.dk-chip', { hasText: '1366 ×' }).first().click(); await p.waitForTimeout(900);
  const out = await p.evaluate(() => {
    const pick = (pred) => [...document.querySelectorAll('*')].find(pred);
    const byText = (t, tag) => pick(e => (!tag || e.tagName === tag) && e.childElementCount <= 2 && (e.innerText || '').trim() === t);
    const byAria = a => document.querySelector(`[aria-label="${a}"]`);
    const st = (n, e) => { if (!e) return { n, missing: true }; const cs = getComputedStyle(e); const r = e.getBoundingClientRect(); return { n, tag: e.tagName, ff: cs.fontFamily.split(',')[0].replace(/['"]/g, ''), fs: cs.fontSize, fw: cs.fontWeight, ls: cs.letterSpacing, tt: cs.textTransform, color: cs.color, bg: cs.backgroundColor, bd: cs.borderTopWidth + ' ' + cs.borderTopStyle + ' ' + cs.borderTopColor, rad: cs.borderRadius, h: Math.round(r.height), w: Math.round(r.width), pad: cs.padding }; };
    return [
      st('menu File', byText('File', 'BUTTON')),
      st('wordmark', byText('CARTALITH')),
      st('search', byAria('search places and commands')),
      st('undo', byAria('undo nothing')),
      st('toolbar ctx GENERATE · WORLD', pick(e => e.childElementCount === 0 && /^GENERATE$/.test((e.innerText || '').trim()) && e.getBoundingClientRect().top < 120)),
      st('pin Save', byAria('Save (pinned)')),
      st('pin +', byAria('pin a command to the quick bar')),
      st('rail WORLD', byAria('WORLD domain')),
      st('dock path WORLD › GENERATE', pick(e => e.childElementCount === 0 && /WORLD\s*›\s*GENERATE/i.test((e.innerText || '').trim()))),
      st('category chip GENERATE (on)', pick(e => e.tagName === 'BUTTON' && (e.innerText || '').trim() === 'GENERATE')),
      st('category chip SCULPT (off)', byText('SCULPT', 'BUTTON')),
      st('section label TOOLS', byText('TOOLS')),
      st('section label RUN', byText('RUN')),
      st('field label Seed', byText('Seed')),
      st('seed input', pick(e => e.tagName === 'INPUT' && e.value === '483920') || pick(e => (e.innerText || '').trim() === '483920')),
      st('primary GENERATE WORLD', byText('GENERATE WORLD', 'BUTTON') || pick(e => e.tagName === 'BUTTON' && /GENERATE WORLD/.test(e.innerText || ''))),
      st('secondary NEW SEED', pick(e => e.tagName === 'BUTTON' && /NEW SEED/.test(e.innerText || ''))),
      st('tool tile Inspect', byAria('Inspect (V)')),
      st('hint last run', pick(e => e.childElementCount === 0 && /^last run/.test((e.innerText || '').trim()))),
      st('ctx tab INSPECT', byAria('inspect tab (Ctrl+1)')),
      st('ctx tab NOTE', byAria('note tab (Ctrl+2)')),
      st('ctx name Nothing selected', byText('Nothing selected')),
      st('list row Orentamwash', pick(e => e.childElementCount === 0 && (e.innerText || '').trim() === 'Orentamwash')),
      st('list row sub', pick(e => e.childElementCount === 0 && /^Monarchy/.test((e.innerText || '').trim()))),
      st('map 3D', byAria('switch to 3D')),
      st('map zoom in', byAria('zoom in')),
      st('base map chip', byAria('layers')),
      st('timeline toggle', pick(e => e.tagName === 'BUTTON' && /TIMELINE/.test(e.innerText || ''))),
      st('timeline play', byAria('play')),
      st('status left', pick(e => e.childElementCount === 0 && /^stages 01/.test((e.innerText || '').trim()))),
      st('status mid', pick(e => e.childElementCount === 0 && /^year 412/.test((e.innerText || '').trim()))),
      st('scale bar text', pick(e => e.childElementCount === 0 && /^\d+ km$/.test((e.innerText || '').trim())))
    ];
  });
  save('d_styles_1366', out);
  for (const o of out) console.log(JSON.stringify(o));
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
