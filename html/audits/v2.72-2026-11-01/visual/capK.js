// Phase K: the sculpt bar with the full V15 fix (mode strip to row two, spacer not growing), at 1600 and
// 1366, plus the idle bar (no tool armed) to check the pins still sit at the right end.
const L = require('./lib.js');
const fs = require('fs');
const CSS = fs.readFileSync(L.path.join(L.A, 'fixes.css'), 'utf8');
(async () => {
  const b = await L.chromium.launch(); const out = {};
  for (const fixed of [false, true]) {
    const p = await L.boot(b, L.V272, { width: 1600, height: 900 });
    if (fixed) await p.addStyleTag({ content: CSS });
    await L.genByClicks(p, 12345, '1024', null);
    for (const [W, H] of [[1600, 900], [1366, 768]]) {
      await p.setViewportSize({ width: W, height: H }); await p.waitForTimeout(500);
      const tag = (fixed ? 'after' : 'before') + '_' + W;
      out[tag + '_idle'] = await p.evaluate(() => { const pins = document.getElementById('toolOptsPins').getBoundingClientRect(); return { pinsRight: Math.round(pins.right), barH: Math.round(document.getElementById('toolOpts').getBoundingClientRect().height) }; });
      await p.click('#worldCatSeg [data-wcat="sculpt"]'); await p.waitForTimeout(300);
      await p.click('#sculptFeatureSeg [data-sfeat="freehand"]'); await p.waitForTimeout(600);
      out[tag + '_sculpt'] = await p.evaluate(() => { const s = document.getElementById('sculptFeatureSeg'); const sr = s.getBoundingClientRect(); return { scrollLeft: Math.round(s.scrollLeft), client: s.clientWidth, scroll: s.scrollWidth, hiddenLeft: [...s.children].filter(c => c.getBoundingClientRect().right <= sr.left + 1).length, hiddenRight: [...s.children].filter(c => c.getBoundingClientRect().left >= sr.right - 1).length, barH: Math.round(document.getElementById('toolOpts').getBoundingClientRect().height) }; });
      await L.clipShot(p, { x: 0, y: 36, width: W, height: 90 }, `k_sculpt_bar_${tag}`);
      await p.click('#worldCatSeg [data-wcat="terrain"]'); await p.waitForTimeout(300);
    }
    await p.context().close();
  }
  L.save('k_sculpt', out); console.log(JSON.stringify(out));
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
