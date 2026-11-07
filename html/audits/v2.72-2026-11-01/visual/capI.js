// Phase I: the Settlements screen's faction rows at phone width and at 1366: the name's drawn width.
const L = require('./lib.js');
(async () => {
  const b = await L.chromium.launch(); const out = {};
  for (const [tag, vp, mobile] of [['phone412', { width: 412, height: 915 }, true], ['desk1366', { width: 1366, height: 768 }, false]]) {
    const p = await L.boot(b, L.V272, vp, { mobile });
    if (mobile) { await p.tap('#obSkip'); await p.waitForTimeout(400); await p.tap('#settlementsHeaderBtn'); }
    else { await p.click('#obSkip'); await p.waitForTimeout(400); await p.click('#settlementsHeaderBtn'); }
    await p.waitForTimeout(800);
    out[tag] = await p.evaluate(() => [...document.querySelectorAll('#civFactionList > div')].map(r => { const nm = r.querySelector('b'); const q = nm.getBoundingClientRect(); return { name: nm.textContent, drawnW: Math.round(q.width), needW: nm.scrollWidth, rowW: Math.round(r.getBoundingClientRect().width) }; }));
    out[tag + '_overview'] = await p.evaluate(() => { const t = document.getElementById('civFactionsModal').innerText; const m = t.match(/Largest by population:[^\n]*/); return m ? m[0] : null; });
    await L.shot(p, 'i_faction_rows_' + tag);
    await p.context().close();
  }
  L.save('i_faction_rows', out); console.log(JSON.stringify(out));
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
