// Phase A: v2.72, dark, seed 12345 at the default resolution (2K), every surface at 1600x900, then the
// main views at 1366x768, 1920x1080 and 2560x1440. Real clicks wherever a user would click.
const L = require('./lib.js');
const { log, shot, elShot, clipShot, save } = L;
const THEME = process.argv[2] || 'dark';
const SEED = +(process.argv[3] || 12345);
const RES = process.argv[4] || '';          // '' = the card's default (2K)
const P = (process.argv[5] || 'a') + '_';   // shot prefix
const FULL = THEME === 'dark' && !process.argv[6];

(async () => {
  const b = await L.chromium.launch();
  const p = await L.boot(b, L.V272, { width: 1600, height: 900 }, { light: THEME === 'light' });
  await L.installAudit(p);
  await L.genByClicks(p, SEED, RES, P + '00');
  await L.installAudit(p);
  await shot(p, P + '01_world_generate_1600');
  save(P + 'world_info', await p.evaluate(() => ({ seed: state.tect.seed, GW, GH, theme: document.documentElement.dataset.theme || 'dark', places: (state.places || []).length })));

  // Populate through the real controls: rail CIVIL, its Generation page, Auto-populate.
  await p.click('.domain-rail [data-domain="civ"]').catch(async e => { log('rail civ click failed', e.message); });
  await p.waitForTimeout(400);
  await shot(p, P + '02_civil_before_populate_1600');
  const gen = await p.$('[data-civsub="generation"]'); if (gen) { await gen.click(); await p.waitForTimeout(300); }
  await p.click('#civAutoPopulateBtn', { noWaitAfter: true, timeout: 120000 });
  await p.waitForTimeout(500); await L.idle(p, 800);
  await shot(p, P + '03_civil_populated_1600');
  // Two recorded years through the strip's own controls (expand the strip first).
  const more = await p.evaluate(() => { const m = document.getElementById('tlStripMore'); return m ? m.hidden : null; });
  if (more) await p.click('#tlStripToggle');
  await p.waitForTimeout(300);
  await shot(p, P + '04_strip_expanded_noyears_1600');
  for (const y of [380, 412]) { await p.fill('#civTlYear', String(y)); await p.click('#civTlAddYearBtn'); await p.waitForTimeout(500); await L.idle(p, 300); }
  await shot(p, P + '05_strip_expanded_twoyears_1600');
  save(P + 'pop_info', await p.evaluate(() => ({ places: (state.places || []).filter(x => x.pop).length, years: (typeof civTimeline !== 'undefined') ? civTimeline.map(t => t.year) : null })));

  // Select the biggest settlement with a real click.
  const at = await p.evaluate(() => { const q = [...state.places].filter(x => x.pop).sort((a, b) => b.pop - a.pop)[0]; const s = _civPlaceScreenPos(q.x, q.y); return { x: s[0], y: s[1], name: q.name }; });
  log('biggest place', JSON.stringify(at));
  await p.mouse.click(at.x, at.y); await p.waitForTimeout(600);
  await shot(p, P + '06_selected_place_1600');
  if (FULL) {
    save(P + 'contrast_selected_1600', await p.evaluate(() => __aud.contrast()));
    save(P + 'truncs_selected_1600', await p.evaluate(() => __aud.truncs()));
    save(P + 'controls_civil_1600', await p.evaluate(() => __aud.controls()));
    save(P + 'emoji_civil_1600', await p.evaluate(() => __aud.emoji()));
  }
  // The other sizes, same state.
  for (const [W, H] of [[1366, 768], [1920, 1080], [2560, 1440]]) {
    if (!FULL && W !== 1366) continue;
    await p.setViewportSize({ width: W, height: H }); await p.waitForTimeout(900);
    await shot(p, P + `07_selected_place_${W}`);
    save(P + `geom_${W}`, await p.evaluate(() => ({ truncs: __aud.truncs(), off: __aud.offscreen() })));
  }
  await p.setViewportSize({ width: 1600, height: 900 }); await p.waitForTimeout(800);
  save(P + 'geom_1600', await p.evaluate(() => ({ truncs: __aud.truncs(), off: __aud.offscreen() })));
  log('errors:', p._errs.length ? p._errs.join(' | ') : 'none');
  // keep the page state for the next phase by saving nothing else; close
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
