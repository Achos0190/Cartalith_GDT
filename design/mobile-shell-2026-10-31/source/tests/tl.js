const { chromium } = require('playwright');
// Diagnostic captures go to SHOT_DIR, or the system temp folder: they are working images, not evidence.
const OUT = process.env.SHOT_DIR || (require('os').tmpdir() + '/');
(async () => {
  const b = await chromium.launch(); const errs = [];
  for (const [url, vw, vh, tag] of [['Desktop.dc.html', 1600, 1000, 'desk'], ['Tablet.dc.html', 1194, 834, 'tab']]) {
    const p = await b.newPage({ viewport: { width: vw, height: vh } }); p.on('pageerror', (e) => errs.push(tag + ' PAGEERROR ' + e.message)); p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push(tag + ' CONSOLE ' + m.text().slice(0, 200)); });
    await p.goto('http://127.0.0.1:8770/' + url); await p.waitForTimeout(3200); const tip = p.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
    await p.getByRole('button', { name: /TIMELINE/ }).first().click(); await p.waitForTimeout(400);
    const tb = p.getByRole('toolbar', { name: 'timeline controls' }); console.log(tag, 'controls', await tb.count(), (await tb.innerText()).replace(/\n/g, ' '));
    await p.locator('input[aria-label="timeline year"]').evaluate((el) => { const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; set.call(el, '300'); el.dispatchEvent(new Event('input', { bubbles: true })); }); await p.waitForTimeout(250);
    await tb.getByRole('button', { name: /RECORD 300/ }).click(); await p.waitForTimeout(250); console.log(tag, 'after record', (await tb.innerText()).replace(/\n/g, ' ').slice(0, 80));
    await p.screenshot({ path: OUT + 'tl_' + tag + '.png' }); await p.close();
  }
  await b.close(); console.log(errs.slice(0, 8).join('\n') || 'no page errors');
})();
