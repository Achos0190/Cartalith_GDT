// Screenshot the boot state (setup gate over the boot render) and report the gate's backdrop.
// Usage: node shot_boot.js <file.html> <out.png> [width] [height]
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, out, w, h] = process.argv.slice(2);
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: +(w || 1400), height: +(h || 900) } });
  await p.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await p.waitForTimeout(1500);
  const info = await p.evaluate(() => {
    const ob = document.getElementById('onboard'); const cs = ob ? getComputedStyle(ob) : null;
    const card = ob ? ob.querySelector('.card') : null; const r = card ? card.getBoundingClientRect() : null;
    const view = document.getElementById('view'); const vr = view.getBoundingClientRect();
    return { obDisplay: ob && ob.style.display, obBg: cs && cs.backgroundColor, obBackdrop: cs && (cs.backdropFilter || cs.webkitBackdropFilter),
             obRect: ob && ob.getBoundingClientRect().toJSON(), card: r && r.toJSON(), viewRect: vr.toJSON(), viewWH: [view.width, view.height] };
  });
  await p.screenshot({ path: out });
  console.log(JSON.stringify(info));
  await b.close();
})();
