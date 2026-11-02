// Phase J: every proposed CSS fix injected at once (fixes.css, scratch page only), then the same surfaces
// re-captured and re-measured in dark and light, so the "after" numbers sit beside the "before" ones.
const L = require('./lib.js');
const { log, shot, clipShot, save } = L;
const fs = require('fs');
const CSS = fs.readFileSync(L.path.join(L.A, 'fixes.css'), 'utf8');
async function step(n, fn) { try { await fn(); } catch (e) { log('STEP FAILED', n, e.message.split('\n')[0]); } }
(async () => {
  const b = await L.chromium.launch();
  const out = {};
  for (const theme of ['dark', 'light']) {
    const t = theme[0];
    const p = await L.boot(b, L.V272, { width: 1600, height: 900 }, { light: theme === 'light' });
    await p.addStyleTag({ content: CSS });
    await L.installAudit(p);
    await step('newworld', async () => { await p.click('#obGenerate'); await p.waitForTimeout(300); await shot(p, `j_${t}_newworld_fixed`); });
    await p.fill('#suSeedN', '12345'); await p.click('#suResSeg button[data-w="1024"]');
    await p.click('#suGenCommit', { noWaitAfter: true, timeout: 120000 }); await p.waitForTimeout(500); await L.idle(p, 800);
    await L.installAudit(p);
    await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(300);
    await p.click('[data-civsub="generation"]'); await p.waitForTimeout(300);
    await p.click('#civAutoPopulateBtn', { noWaitAfter: true, timeout: 120000 }); await p.waitForTimeout(500); await L.idle(p, 800);
    if (await p.evaluate(() => document.getElementById('tlStripMore').hidden)) await p.click('#tlStripToggle');
    for (const y of [380, 412]) { await p.fill('#civTlYear', String(y)); await p.click('#civTlAddYearBtn'); await p.waitForTimeout(400); await L.idle(p, 200); }
    await p.click('#tlStripToggle'); await p.waitForTimeout(200);
    const at = await p.evaluate(() => { const w = document.querySelector('.canvas-wrap').getBoundingClientRect(); for (const q of [...state.places].filter(x => x.pop).sort((a, b) => b.pop - a.pop)) { const s = _civPlaceScreenPos(q.x, q.y); if (s[0] > w.left + 60 && s[0] < w.right - 80 && s[1] > w.top + 60 && s[1] < w.bottom - 60) return { x: s[0], y: s[1] }; } return null; });
    if (at) { await p.mouse.click(at.x, at.y); await p.waitForTimeout(700); }
    await shot(p, `j_${t}_place_selected_fixed`);
    out[t + '_contrast_place'] = await p.evaluate(() => { const d = __aud.contrast(); return { texts: d.length, fail: d.filter(x => x.fail && !x.dis).length, failures: d.filter(x => x.fail && !x.dis).map(x => x.region + ' ' + x.fg + '/' + x.bg + ' ' + x.ratio + ' ' + x.text) }; });
    out[t + '_readout'] = await p.evaluate(() => { const r = document.getElementById('readout'); const rr = r.getBoundingClientRect(); const kids = [...r.children].filter(k => getComputedStyle(k).display !== 'none'); return { client: r.clientWidth, cutMidChip: kids.filter(k => { const q = k.getBoundingClientRect(); return q.top < rr.bottom - 1 && q.right > rr.right + 0.5; }).map(k => k.textContent.trim()), wrappedAway: kids.filter(k => k.getBoundingClientRect().top >= rr.bottom - 1).map(k => k.textContent.trim().slice(0, 30)) }; });
    out[t + '_popup'] = await p.evaluate(() => { const e = document.getElementById('placeEditPopup'); return e && e.style.display !== 'none' ? { client: e.clientWidth, scroll: e.scrollWidth } : null; });
    await step('settlements table', async () => { await p.click('#placeEditPopup .si-close').catch(() => {}); await p.click('[data-civsub="settlements"]'); await p.waitForTimeout(600); await L.elShot(p, '.dock-left', `j_${t}_dock_civ_settlements_fixed`); });
    await step('factions modal', async () => { await p.click('#mbRecordsBtn'); await p.click('#mbRecords [data-cmd="factions"]'); await p.waitForTimeout(800); await shot(p, `j_${t}_factions_fixed`); out[t + '_contrast_factions'] = await p.evaluate(() => { const d = __aud.contrast('#civFactionsModal'); return { texts: d.length, fail: d.filter(x => x.fail && !x.dis).length, failures: d.filter(x => x.fail && !x.dis).slice(0, 12).map(x => x.fg + '/' + x.bg + ' ' + x.ratio + ' ' + x.text) }; }); await p.click('#civPmBar button[data-pm="global"]'); await p.waitForTimeout(500); await shot(p, `j_${t}_settlements_global_fixed`); await p.click('#civFactionsModal .cfm-close'); await p.waitForTimeout(300); });
    await step('settings + credits', async () => { await p.click('#mbPrefsBtn'); await p.click('#mbPrefs [data-cmd="setPerf"]'); await p.waitForTimeout(400); await p.click('#setNav [data-setpage="about"]'); await p.waitForTimeout(200); await p.click('#creditsBtn'); await p.waitForTimeout(400); out[t + '_credits_top'] = await p.evaluate(() => { const el = document.elementFromPoint(innerWidth / 2, innerHeight / 2); return el && el.closest('#creditsModal') ? 'creditsModal' : el && el.closest('#settingsModal') ? 'settingsModal' : null; }); await shot(p, `j_${t}_credits_over_settings_fixed`); await p.click('#creditsModal .cr-close'); await p.waitForTimeout(200); await p.click('#settingsClose'); await p.waitForTimeout(200); });
    await step('assets', async () => { await p.click('#assetsHeaderBtn'); await p.waitForTimeout(900); await p.mouse.move(800, 600); await p.waitForTimeout(200); await shot(p, `j_${t}_assets_fixed`); await p.click('#assetsHeaderBtn'); await p.waitForTimeout(600); });
    await step('sculpt', async () => { await p.click('.domain-rail [data-domain="world"]'); await p.waitForTimeout(300); await p.click('#worldCatSeg [data-wcat="sculpt"]'); await p.waitForTimeout(300); await p.click('#sculptFeatureSeg [data-sfeat="freehand"]'); await p.waitForTimeout(500); out[t + '_sculpt'] = await p.evaluate(() => { const s = document.getElementById('sculptFeatureSeg'); const sr = s.getBoundingClientRect(); return { scrollLeft: Math.round(s.scrollLeft), client: s.clientWidth, scroll: s.scrollWidth, hiddenLeft: [...s.children].filter(c => c.getBoundingClientRect().right <= sr.left + 1).length, barH: Math.round(document.getElementById('toolOpts').getBoundingClientRect().height) }; }); await clipShot(p, { x: 0, y: 36, width: 1600, height: 90 }, `j_${t}_sculpt_bar_fixed`); await p.click('#worldCatSeg [data-wcat="terrain"]'); await p.waitForTimeout(300); });
    if (theme === 'dark') {
      await step('export menu', async () => { await p.click('#fileMenuBtn'); await p.hover('#mbExportRow'); await p.waitForTimeout(300); await shot(p, 'j_d_file_export_fixed'); await p.keyboard.press('Escape'); await p.keyboard.press('Escape'); });
      for (const [W, H] of [[1366, 768], [1920, 1080], [2560, 1440]]) {
        await step('size ' + W, async () => { await p.setViewportSize({ width: W, height: H }); await p.waitForTimeout(700); await p.click('.domain-rail [data-domain="civ"]'); await p.waitForTimeout(300); out['d_geom_' + W] = await p.evaluate(() => ({ truncs: __aud.truncs().map(x => x.el.slice(0, 50) + ' ' + x.client + '/' + x.scroll) })); await shot(p, `j_d_civil_${W}_fixed`); });
      }
    }
    log(theme, 'errors:', p._errs.length ? p._errs.join(' | ') : 'none');
    await p.context().close();
  }
  save('j_after', out);
  console.log(JSON.stringify(out, null, 0).slice(0, 4000));
  await b.close();
})().catch(e => { console.error('FAILED', e); process.exit(1); });
