// Dumps the shell mockup's menus, per platform, as the mockup draws them (design/mobile-shell-2026-10-31/Shell.dc.html).
// Run: node mockup_menus.js <path to Shell.dc.html> > mockup_menus.json
// It evaluates the mockup's own menuTree(), tabletOverflow() and tabletWorldMenu() in a sandbox, then applies the
// per-platform rules the mockup's menuRows() applies: rows marked `mobile` are left out on the desktop only; rows marked
// `desktopOnly` stay on the desktop, draw as "DESKTOP ONLY" (disabled) on the tablet and are left out on the phone (PW-5);
// rows marked `notPhone` (key-binding rows, which no phone accelerator can fire) are left out on the phone.
// Reads one file; writes stdout.
const fs = require('fs'); const vm = require('vm');
const html = fs.readFileSync(process.argv[2], 'utf8');
const js = html.slice(html.indexOf('<script type="text/x-dc"'), html.lastIndexOf('</script>'));
const body = js.slice(js.indexOf('>') + 1);
const pick = (name) => { const i = body.indexOf('function ' + name + '('); if (i < 0) throw new Error('no ' + name); let d = 0, j = body.indexOf('{', i); for (let k = j; k < body.length; k++) { if (body[k] === '{') d++; else if (body[k] === '}') { d--; if (d === 0) return body.slice(i, k + 1); } } };
const consts = ['STAGE_NAMES', 'LANDMARK_FAMILIES', 'ASSET_FAMILIES'].map((n) => { const i = body.indexOf('const ' + n + ' ='); const e = body.indexOf(';\n', i); return body.slice(i, e + 1); }).join('\n');
const icI = body.indexOf('const IC = {'); let d = 0, icE = icI; for (let k = body.indexOf('{', icI); k < body.length; k++) { if (body[k] === '{') d++; else if (body[k] === '}') { d--; if (d === 0) { icE = k + 1; break; } } }
const promoted = (() => { const i = body.indexOf('const PROMOTED ='); return body.slice(i, body.indexOf('\n', i)); })();   // one line, comment and all
const src = body.slice(icI, icE) + ';\n' + consts + '\n' + promoted + '\n' + pick('notPromoted') + '\n' + pick('menuTree') + '\n' + pick('tabletOverflow') + '\n' + pick('tabletWorldMenu') + '\nthis.out = { menus: menuTree() }; this.out.ham = tabletOverflow(this.out.menus); this.out.tworld = tabletWorldMenu(this.out.menus);';
const ctx = {}; vm.createContext(ctx); vm.runInContext(src, ctx);
const KIND = { live: 'command', todo: 'disabled (not built)', readout: 'readout', sign: 'signpost', check: 'check', radio: 'radio', sub: 'submenu', band: 'caption', sep: 'separator' };
function rows(items, platform) {
  const out = [];
  for (const it of items) {
    if (!it) continue;
    if (it.mobile && platform === 'desktop') continue;
    if ((it.desktopOnly || it.notPhone) && platform === 'phone') continue;
    const r = { label: it.label || '', kind: KIND[it.t] || it.t };
    if (it.accel) r.accel = it.accel;
    if (it.t === 'todo') r.reason = it.reason;
    if (it.desktopOnly && platform !== 'desktop') { r.kind = 'disabled (DESKTOP ONLY)'; }
    if (it.dyn === 'gpuRetry') { r.disabled_at_build = true; r.reason = 'Nothing to clear: no GPU readback has failed this session'; }   // liveReason() until a readback fails
    if (it.sm) r.session_mode_gated = true;
    if (it.items) r.items = rows(it.items, platform);
    out.push(r);
  }
  return out;
}
const M = ctx.out.menus; const by = (id) => M.find((m) => m.id === id);
const desktop = M.map((m) => ({ menu: m.label, rows: rows(m.items, 'desktop') }));
const tabletWide = M.map((m) => ({ menu: m.label, rows: rows(m.items, 'tablet') }));
const tabletNarrow = [{ menu: '☰', rows: rows(ctx.out.ham.items, 'tablet') }, { menu: 'File', rows: rows(by('file').items, 'tablet') }, { menu: 'World', rows: rows(ctx.out.tworld.items, 'tablet') }, { menu: 'Story', rows: rows(by('story').items, 'tablet') }];
const phone = { root: ['QUICK (pinned commands)'].concat(M.map((m) => m.label)).concat(['STATUS (world, state, last autosave)']), menus: M.map((m) => ({ menu: m.label, rows: rows(m.items, 'phone') })) };
process.stdout.write(JSON.stringify({ source: process.argv[2], platforms: { desktop, tablet_wide_1180dp_and_up: tabletWide, tablet_below_1180dp: tabletNarrow, phone_MENU: phone } }, null, 1));
