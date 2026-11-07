/* Read-only audit probe (scratch, not for the repo).
 * Measures two claims in the feature-port audit against a real page:
 *  (1) Find (the unified search, v2.15) indexes controls and map entities but no menu command;
 *  (2) _jpClaimedAt reads every cell as claimed, because it tests >=0 on a Uint8Array whose
 *      unclaimed value is 0 (faction 0 is 'Unclaimed').
 * Usage: node probe_audit.js "<html file>" [seed ...]
 */
const path = require('path');
const { chromium } = require('playwright');

(async () => {
  const file = process.argv[2];
  const seeds = process.argv.slice(3).map(Number);
  if (!seeds.length) seeds.push(12345, 777);
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  page.on('dialog', d => d.dismiss());
  const errs = []; page.on('pageerror', e => errs.push(String(e).slice(0, 200)));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 120000 });
  await page.waitForTimeout(600);

  const find = await page.evaluate(() => {
    const R = {};
    const q = ['generate', 'save', 'credits', 'diagnostics', 'factions', 'asset library', 'timeline'];
    for (const s of q) { const h = _findRun(s); R[s] = h.map(x => x.kind + ':' + x.label).slice(0, 6); }
    const rows = [...document.querySelectorAll('.mb-row[data-cmd]')];
    R.menuCommandRows = rows.length;
    R.distinctCmds = new Set(rows.map(r => r.getAttribute('data-cmd'))).size;
    R.cmdTableSize = Object.keys(_DSK_CMDS).length;
    R.findKinds = [...new Set(_findControlIndex().concat(_findMapIndex()).map(r => r.kind))];
    return R;
  });

  const worlds = [];
  for (const seed of seeds) {
    const r = await page.evaluate(async (seed) => {
      const R = { seed };
      state.tect.seed = seed; state.resW = 256; GW = 256; GH = gridH(GW); allocate();
      await generate();
      const ob = document.getElementById('onboard'); if (ob) ob.style.display = 'none';
      _civIterativeAutoWorld(3);
      R.territoryBeforePolity = (typeof civTerritory !== 'undefined' && civTerritory) ? 'present' : 'null';
      R.claimedAtBeforePolity = _jpClaimedAt(GW >> 1, GH >> 1);
      _civAutoPolity();
      const terr = getCivTerritory();
      R.territoryType = terr ? terr.constructor.name : null;
      R.factions = CIV_FACTIONS.length - 1;
      R.unclaimedName = CIV_FACTIONS[0][0];
      let land = 0, landUnclaimed = 0, sea = 0, trueUnclaimed = 0, trueSea = 0, fixedUnclaimed = 0, fixedSea = 0;
      const fixed = (gx, gy) => { const xi = Math.max(0, Math.min(GW - 1, Math.round(gx))), yi = Math.max(0, Math.min(GH - 1, Math.round(gy))); return terr[yi * GW + xi] > 0; };
      for (let y = 0; y < GH; y++) for (let x = 0; x < GW; x++) {
        const i = y * GW + x, water = field[i] < state.seaLevel;
        if (water) { sea++; if (_jpClaimedAt(x, y)) trueSea++; if (fixed(x, y)) fixedSea++; }
        else { land++; if (terr[i] === 0) { landUnclaimed++; if (_jpClaimedAt(x, y)) trueUnclaimed++; if (fixed(x, y)) fixedUnclaimed++; } }
      }
      Object.assign(R, { land, landUnclaimed, sea, claimedAt_true_on_unclaimed_land: trueUnclaimed,
        claimedAt_true_on_sea: trueSea, fixed_true_on_unclaimed_land: fixedUnclaimed, fixed_true_on_sea: fixedSea });
      R.places = state.places.length;
      /* Journeys between settlements of different factions: tolls and borders as shipped, then with
         the predicate fixed (>0), on the SAME journey. Reassigning the global function is in-page
         only; the file is not touched. */
      const S = state.places.filter(p => p && p.category === 'settlement' && p.faction > 0);
      const pairs = [];
      for (let i = 0; i < S.length && pairs.length < 6; i++) for (let j = i + 1; j < S.length && pairs.length < 6; j++)
        if (S[i].faction !== S[j].faction && Math.hypot(S[i].x - S[j].x, S[i].y - S[j].y) > GW * 0.15) pairs.push([S[i], S[j]]);
      const shipped = window._jpClaimedAt;
      R.journeys = [];
      for (const [a, b] of pairs) {
        _civWaypoints = [[a.x, a.y], [b.x, b.y]]; _civCommitRoute();
        const jn = civJourneys[civJourneys.length - 1];
        window._jpClaimedAt = shipped;
        const p0 = _jpPlan(jn); const c0 = p0 && jpJourneyCost(p0);
        window._jpClaimedAt = fixed;
        if (jn.plan) jn.plan.stages = undefined;
        const p1 = _jpPlan(jn); const c1 = p1 && jpJourneyCost(p1);
        window._jpClaimedAt = shipped;
        R.journeys.push({ from: a.name + '(f' + a.faction + ')', to: b.name + '(f' + b.faction + ')', km: Math.round(jn.km),
          stages: p0 ? p0.stages.length : null,
          claimedFracs_shipped: p0 ? p0.stages.map(s => +(s.claimedFrac || 0).toFixed(2)) : null,
          claimedFracs_fixed: p1 ? p1.stages.map(s => +(s.claimedFrac || 0).toFixed(2)) : null,
          borders_shipped: c0 ? c0.borders : null, tolls_shipped: c0 ? c0.tolls : null,
          borders_fixed: c1 ? c1.borders : null, tolls_fixed: c1 ? c1.tolls : null,
          infra_shipped: p0 ? p0.stages.map(s => s.infra) : null, infra_fixed: p1 ? p1.stages.map(s => s.infra) : null });
      }
      return R;
    }, seed);
    worlds.push(r);
  }
  console.log(JSON.stringify({ file: path.basename(file), find, worlds, pageErrors: errs }, null, 1));
  await browser.close();
})().catch(e => { console.error(e); process.exit(1); });
