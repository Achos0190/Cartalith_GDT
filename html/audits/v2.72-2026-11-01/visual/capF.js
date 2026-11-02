// Phase F: (1) the light start card and New World card; (2) the loading screen (busy overlay) mid-generate;
// (3) the finalized state (phase chip, locked controls) after a real Bake & finalize at the smallest depth;
// (4) the modals at phone width (412x915, touch), which stage 2 does not rebuild.
const L = require('./lib.js');
const { log, shot, clipShot, save } = L;
async function step(n, fn) { try { await fn(); } catch (e) { log('STEP FAILED', n, e.message.split('\n')[0]); } }
(async () => {
  const b = await L.chromium.launch();
  // (1) light start card
  const l = await L.boot(b, L.V272, { width: 1600, height: 900 }, { light: true });
  await step('light start', async () => { await shot(l, 'f_light_startcard'); await l.click('#obGenerate'); await l.waitForTimeout(300); await shot(l, 'f_light_newworld'); });
  await l.context().close();
  // (2)+(3) dark, 1K world
  const p = await L.boot(b, L.V272, { width: 1600, height: 900 });
  await L.installAudit(p);
  await L.genByClicks(p, 12345, '1024', null);
  await step('busy overlay', async () => {
    await p.keyboard.press('Control+r');
    for (let i = 0; i < 20; i++) { await p.waitForTimeout(150); const on = await p.evaluate(() => getComputedStyle(document.getElementById('busy')).display !== 'none'); if (on) { await shot(p, 'f_busy_overlay_1600'); break; } }
    await L.idle(p, 500);
  });
  await step('disabled undo/redo crop', async () => { await clipShot(p, { x: 1080, y: 0, width: 520, height: 36 }, 'f_menubar_right'); });
  await step('finalize', async () => {
    await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(300);
    const opts = await p.evaluate(() => [...document.querySelectorAll('#bakeAllDepth option')].map(o => o.value));
    log('bake depth options', JSON.stringify(opts));
    await p.selectOption('#bakeAllDepth', opts[0]);
    await p.click('#bakeAllBtn', { noWaitAfter: true, timeout: 120000 });
    await p.waitForTimeout(800); await L.idle(p, 1000);
    for (let i = 0; i < 120; i++) { const f = await p.evaluate(() => document.body.classList.contains('phase-explore')); if (f) break; await p.waitForTimeout(500); }
    await shot(p, 'f_finalized_world_1600');
    save('f_finalized_state', await p.evaluate(() => ({ phase: document.body.classList.contains('phase-explore'), genBtnDisabled: document.getElementById('genBtn').disabled, seedDisabled: document.getElementById('seedN').disabled })));
    await p.click('#fileMenuBtn'); await p.waitForTimeout(300); await shot(p, 'f_finalized_file_menu'); await p.keyboard.press('Escape');
  });
  await p.context().close();
  // (4) phone modals
  const ph = await L.boot(b, L.V272, { width: 412, height: 915 }, { mobile: true });
  await L.installAudit(ph);
  await step('phone start', async () => { await shot(ph, 'f_phone_startcard'); await ph.tap('#obGenerate'); await ph.waitForTimeout(300); await shot(ph, 'f_phone_newworld'); await ph.tap('#suResSeg button[data-w="1024"]'); await ph.fill('#suSeedN', '12345'); await ph.tap('#suGenCommit', { noWaitAfter: true, timeout: 120000 }); await ph.waitForTimeout(500); await L.idle(ph, 800); await shot(ph, 'f_phone_world'); });
  await L.installAudit(ph);
  await step('phone settlements screen', async () => { await ph.tap('#settlementsHeaderBtn'); await ph.waitForTimeout(800); await shot(ph, 'f_phone_settlements'); save('f_phone_settlements_truncs', await ph.evaluate(() => ({ t: __aud.truncs('#civFactionsModal'), off: __aud.offscreen('#civFactionsModal button, #civFactionsModal select, #civFactionsModal input') }))); await ph.tap('#civFactionsModal .cfm-close'); await ph.waitForTimeout(300); });
  await step('phone settings', async () => { await ph.tap('#cogBtn'); await ph.waitForTimeout(600); await shot(ph, 'f_phone_settings'); save('f_phone_settings_truncs', await ph.evaluate(() => __aud.truncs('#settingsModal'))); await ph.tap('#settingsClose'); await ph.waitForTimeout(300); });
  await step('phone credits', async () => { await ph.evaluate(() => { const b = document.getElementById('creditsBtn'); if (b) b.scrollIntoView(); }); await ph.tap('#cogBtn'); await ph.waitForTimeout(500); await ph.tap('#setNav [data-setpage="about"]'); await ph.waitForTimeout(300); const cb = await ph.$('#creditsBtn'); if (cb && await cb.isVisible()) { await cb.tap(); await ph.waitForTimeout(500); await shot(ph, 'f_phone_credits'); await ph.tap('#creditsModal .cr-close'); } else log('no visible credits button on phone'); const open = await ph.evaluate(() => document.getElementById('settingsModal').classList.contains('open')); if (open) await ph.tap('#settingsClose'); });
  await step('phone file menu', async () => { await ph.tap('#fileMenuBtn'); await ph.waitForTimeout(400); await shot(ph, 'f_phone_file_menu'); await ph.tap('#fileMenuBtn'); await ph.waitForTimeout(200); });
  log('errors:', [p, ph].map(x => x._errs.join(' | ')).join(' || '));
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
