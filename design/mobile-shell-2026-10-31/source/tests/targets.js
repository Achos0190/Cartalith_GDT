const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch();
  const audit = async (p, tag) => {
    const bad = await p.evaluate(() => {
      const out = []; const vw = innerWidth, vh = innerHeight;
      for (const el of document.querySelectorAll('button, input, [role="tab"], a[href]')) {
        const r = el.getBoundingClientRect(); if (r.width === 0 || r.height === 0) continue;
        if (r.right < 0 || r.bottom < 0 || r.left > vw || r.top > vh) continue;
        const cs = getComputedStyle(el); if (cs.visibility === 'hidden' || cs.display === 'none') continue;
        // the drawn rect clipped by scrolling ancestors
        let x0 = r.left, y0 = r.top, x1 = r.right, y1 = r.bottom; let a = el.parentElement;
        while (a) { const s = getComputedStyle(a); if (/(auto|scroll|hidden)/.test(s.overflow + s.overflowX + s.overflowY)) { const q = a.getBoundingClientRect(); x0 = Math.max(x0, q.left); y0 = Math.max(y0, q.top); x1 = Math.min(x1, q.right); y1 = Math.min(y1, q.bottom); } a = a.parentElement; }
        if (x1 - x0 <= 1 || y1 - y0 <= 1) continue; // fully scrolled out of view
        if (el.type === 'range') { if (r.height < 44) out.push(['range', Math.round(r.width), Math.round(r.height)]); continue; }
        if (r.width < 44 || r.height < 44) out.push([(el.getAttribute('aria-label') || el.innerText || el.placeholder || el.tagName).slice(0, 40).replace(/\n/g, ' '), Math.round(r.width), Math.round(r.height)]);
      }
      return out;
    });
    console.log(tag, bad.length ? JSON.stringify(bad.slice(0, 14)) : 'all targets >= 44 x 44');
  };
  for (const [w, h] of [[360, 780], [412, 892], [780, 360]]) {
    const p = await b.newPage({ viewport: { width: w, height: h } });
    await p.goto('http://127.0.0.1:8770/Live.dc.html'); await p.waitForTimeout(3000);
    const c0 = p.getByLabel('dismiss tip'); if (await c0.count()) await c0.click();
    await audit(p, w + ' map');
    const cell = async (id) => { const bb = await p.locator('[data-navcell="' + id + '"]').boundingBox(); await p.mouse.click(bb.x + bb.width / 2, bb.y + bb.height / 2); await p.waitForTimeout(500); };
    await cell('dom'); await audit(p, w + ' domain sheet');
    await cell('menu'); await audit(p, w + ' menu');
    await p.getByText('Preferences', { exact: true }).first().click(); await p.waitForTimeout(400); await audit(p, w + ' prefs');
    await p.screenshot({ path: 'render/tg_' + w + '.png' });
    await cell('story'); await audit(p, w + ' story');
    await p.close();
  }
  const p = await b.newPage({ viewport: { width: 1194, height: 834 } });
  await p.goto('http://127.0.0.1:8770/Tablet.dc.html'); await p.waitForTimeout(3000); await audit(p, 'tablet 1194');
  await p.getByRole('button', { name: 'File', exact: true }).click(); await p.waitForTimeout(300); await audit(p, 'tablet File menu');
  await b.close();
})();
