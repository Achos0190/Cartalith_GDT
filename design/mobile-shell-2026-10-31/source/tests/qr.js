// Drives the quick bar and the cartwheel: tablet cluster, hold → ring + card, slide-select, sub-ring, dock;
// phone quick drawer by tap and by drag. Prints sizes and every page error.
const { chromium } = require('playwright');
// Diagnostic captures go to SHOT_DIR, or the system temp folder: they are working images, not evidence.
const OUT = process.env.SHOT_DIR || (require('os').tmpdir() + '/');
const BASE = 'http://127.0.0.1:8770/';
const bb = async (loc) => { const b = await loc.boundingBox(); return b ? { x: Math.round(b.x), y: Math.round(b.y), w: Math.round(b.width), h: Math.round(b.height) } : null; };
(async () => {
  const b = await chromium.launch();
  const errs = [];
  const watch = (p) => { p.on('pageerror', (e) => errs.push('PAGEERROR ' + e.message)); p.on('console', (m) => { if (m.type() === 'error' && !/attribute d: Expected/.test(m.text())) errs.push('CONSOLE ' + m.text()); }); };
  // ---------------- tablet ----------------
  const t = await b.newPage({ viewport: { width: 1194, height: 834 } }); watch(t);
  await t.goto(BASE + 'Tablet.dc.html'); await t.waitForTimeout(3000);
  const tip = t.getByLabel('dismiss tip'); if (await tip.count()) await tip.first().click();
  const pins = ['Save (pinned)', 'Player map (pinned)', 'Planner (pinned)', 'Vault (pinned)'];
  for (const a of pins) console.log('cluster', a, JSON.stringify(await bb(t.getByLabel(a, { exact: true }).first())));
  console.log('cluster +', JSON.stringify(await bb(t.getByLabel('pin a command to the quick bar').first())));
  await t.screenshot({ path: OUT + 'qr_t1_bar.png' });
  // hold on the map, slide to NW (Uplift), lift → its sub-ring
  const cx = 620, cy = 430;
  await t.mouse.move(cx, cy); await t.mouse.down(); await t.waitForTimeout(650);
  console.log('ring open', await t.getByLabel('close the ring').count(), 'card', await t.getByText(/^SAMPLE ·/).count());
  await t.screenshot({ path: OUT + 'qr_t2_ring.png' });
  await t.mouse.move(cx - 90, cy - 90, { steps: 8 }); await t.waitForTimeout(150);
  console.log('caption while hovering NW:', (await t.getByText(/UPLIFT ▸/).count()) > 0);
  await t.screenshot({ path: OUT + 'qr_t3_hover.png' });
  await t.mouse.up(); await t.waitForTimeout(300);
  console.log('sub-ring open', await t.getByLabel('back to the ring').count(), 'Hills slot', await t.getByLabel('Uplift ▸ Hills').count());
  await t.screenshot({ path: OUT + 'qr_t4_sub.png' });
  await t.getByLabel('Uplift ▸ Hills').click(); await t.waitForTimeout(300);
  console.log('after pick: ring', await t.getByLabel('close the ring').count(), 'tool bar shows', await t.getByText('SCULPT · HILLS').count());
  await t.screenshot({ path: OUT + 'qr_t5_armed.png' });
  // hold and lift in place: ring stays as tappable surfaces; the dock runs a pin
  await t.mouse.move(cx, cy); await t.mouse.down(); await t.waitForTimeout(650); await t.mouse.up(); await t.waitForTimeout(250);
  console.log('lift in place keeps ring', await t.getByLabel('close the ring').count());
  const slots = await t.locator('button[aria-label*="Inspect"]').count();
  const dockVault = t.getByLabel('Vault (pinned under the ring)');
  console.log('dock chip', JSON.stringify(await bb(dockVault.first())));
  const slotBox = await bb(t.getByLabel(/^Uplift, opens a sub-ring/).first()); console.log('slot box', JSON.stringify(slotBox), 'inspect slots', slots);
  const disc = await bb(t.getByLabel('close the ring').first());
  const card = await bb(t.getByText(/^SAMPLE ·/).first()); console.log('centre', JSON.stringify(disc), 'card title', JSON.stringify(card));
  await t.screenshot({ path: OUT + 'qr_t6_ring_dock.png' });
  await dockVault.first().click(); await t.waitForTimeout(400);
  console.log('vault window after dock tap', await t.getByText('VAULT', { exact: true }).count());
  await t.screenshot({ path: OUT + 'qr_t7_vault.png' });
  // ---------------- phone ----------------
  const p = await b.newPage({ viewport: { width: 412, height: 892 } }); watch(p);
  await p.goto(BASE + 'Shell.dc.html'); await p.waitForTimeout(3000);
  const tip2 = p.getByLabel('dismiss tip'); if (await tip2.count()) await tip2.first().click();
  const grip = p.getByLabel('open the quick drawer: pinned commands');
  const g = await bb(grip.first()); console.log('phone grip', JSON.stringify(g));
  await p.screenshot({ path: OUT + 'qr_p1_grip.png' });
  await grip.first().click(); await p.waitForTimeout(350);
  const dr = await bb(p.getByRole('dialog', { name: 'quick drawer: pinned commands' }).first()); console.log('drawer', JSON.stringify(dr));
  await p.screenshot({ path: OUT + 'qr_p2_open.png' });
  await p.getByText('Vault', { exact: true }).first().click(); await p.waitForTimeout(400);
  console.log('vault after drawer tap', await p.getByText('VAULT', { exact: true }).count(), 'drawer closed', await p.getByRole('dialog', { name: 'quick drawer: pinned commands' }).count() === 0);
  await p.screenshot({ path: OUT + 'qr_p3_vault.png' });
  // close the window by system back is the lab's job; reload for the drag test
  await p.goto(BASE + 'Shell.dc.html'); await p.waitForTimeout(2500); const tip3 = p.getByLabel('dismiss tip'); if (await tip3.count()) await tip3.first().click();
  const g2 = await bb(p.getByLabel('open the quick drawer: pinned commands').first());
  await p.mouse.move(g2.x + 10, g2.y + g2.h / 2); await p.mouse.down(); await p.mouse.move(g2.x + 70, g2.y + g2.h / 2, { steps: 5 }); await p.waitForTimeout(80);
  await p.screenshot({ path: OUT + 'qr_p4_dragging.png' });
  await p.mouse.move(g2.x + 170, g2.y + g2.h / 2, { steps: 5 }); await p.mouse.up(); await p.waitForTimeout(350);
  console.log('drag opens', await p.getByRole('dialog', { name: 'quick drawer: pinned commands' }).count());
  const g3 = await bb(p.getByLabel('close the quick drawer').first());
  await p.mouse.move(g3.x + 20, g3.y + g3.h / 2); await p.mouse.down(); await p.mouse.move(g3.x - 160, g3.y + g3.h / 2, { steps: 6 }); await p.mouse.up(); await p.waitForTimeout(350);
  console.log('drag back closes', await p.getByRole('dialog', { name: 'quick drawer: pinned commands' }).count() === 0);
  await b.close();
  console.log(errs.slice(0, 10).join('\n') || 'no page errors');
})();
