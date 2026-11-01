/* ======================================================================================================
   renderVals: state + frame → everything the template reads. All geometry is derived from the frame's
   dp size here, in one place, which is the part a refactor of the Godot shells should copy.
   ====================================================================================================== */
function themeVars(light) {
  return light
    ? '--sur:#f4f2ee;--pan:#fbfaf7;--pan2:#eceae4;--ink:#111210;--body:#23241f;--sec:#3d3f39;--dim:#5d615c;--acc:#9a5d0c;--accInk:#fbf7ef;--hair:rgba(0,0,0,.14);--hair2:rgba(0,0,0,.08);--bord:rgba(0,0,0,.2);--wash:rgba(164,101,15,.12);--chip:rgba(0,0,0,.05);--chipOn:rgba(0,0,0,.14);--warn:#8a5a00;--warnText:#7a4f00;--warnBord:rgba(138,90,0,.45);--block:#a8402f;--good:#3d7a4c;--water:#3f6f96;--pill:rgba(251,250,247,.94);--navpill:rgba(251,250,247,.92);--navarm:rgba(164,101,15,.16);--navacc:#a4650f;--scrim:rgba(244,242,238,.85);--sheetShadow:0 -10px 30px rgba(0,0,0,.18);'
    : '--sur:#0d0e0f;--pan:#15171a;--pan2:#121314;--ink:#e8ebec;--body:#c8cbcd;--sec:#a9adb0;--dim:#8d9296;--acc:#e0a34a;--accInk:#16130c;--hair:rgba(255,255,255,.10);--hair2:rgba(255,255,255,.07);--bord:rgba(255,255,255,.16);--wash:rgba(224,163,74,.14);--chip:rgba(255,255,255,.05);--chipOn:rgba(255,255,255,.12);--warn:#e0a840;--warnText:#ebc080;--warnBord:rgba(224,168,64,.45);--block:#e08070;--good:#6fae7d;--water:#6a9bc4;--pill:rgba(18,20,21,.92);--navpill:rgba(18,19,20,.92);--navarm:rgba(224,163,74,.16);--navacc:#e0a34a;--scrim:rgba(10,11,12,.72);--sheetShadow:0 -10px 30px rgba(0,0,0,.35);';
}
const chipStyle = (on) => ({ bg: on ? 'var(--wash)' : 'var(--chip)', bord: on ? 'var(--acc)' : 'transparent', col: on ? 'var(--acc)' : 'var(--sec)' });
function paletteFor(c) { const s = c.state; return s.dom === 'world' ? (s.node.world === 'sculpt' ? ['inspect', 'measure', 'region', 'paint'] : ['inspect', 'measure', 'region']) : (s.dom === 'civil' ? ['inspect', 'measure', 'region', 'settle', 'territory', 'lasso', 'way', 'route', 'conflict'] : ['inspect', 'measure', 'region', 'icon', 'label']); }
function domainRows(c, tablet) {
  const s = c.state, node = nodeDef(s.dom, s.node[s.dom]); const rows = [];
  if (s.tool !== 'inspect' && !tablet) rows.push(...toolOptRows(c));
  const pal = paletteFor(c);
  rows.push(H('TOOLS', s.dom === 'civil' ? 'no POI tool · the engine has none' : ''), { k: 'tools', label: 'tools', cell: 64, tools: pal.map((t) => ({ d: TOOLS[t].d, label: TOOLS[t].label.replace('Territory lasso', 'Lasso').replace('Region select', 'Region').replace('Settlement', 'Settle'), key: TOOLS[t].key, on: s.tool === t, tap: () => c._arm(t) })) });
  node.cats.forEach((cat, i) => {
    const id = 'cat-' + s.dom + '-' + cat; const open = s.open[id] == null ? i === 0 : !!s.open[id];
    rows.push({ k: 'acc', label: cat, num: '', state: catState(c, cat), stale: catStale(c, cat), open, tap: () => c.setState({ open: Object.assign({}, s.open, { [id]: !open }) }) });
    if (open) rows.push(...catRows(c, cat, tablet));
  });
  return rows;
}
function catState(c, cat) { const s = c.state; if (cat === 'Generate') return s.running ? 'running' : (s.stale ? 'stale from ' + String(s.stale).padStart(2, '0') : 'resolved'); const st = { Planet: [1, 3], Geology: [4, 5], Hydrology: [6, 7], Climate: [8, 8], Ecology: [9, 10] }[cat]; if (st) return s.stale && st[1] >= s.stale ? 'stale' : PARAMS.filter((p) => p.stage >= st[0] && p.stage <= st[1] && !p.nodock).length + ' params'; if (cat === 'Terrain') return s.stamps.filter((x) => x.kind !== 'paint').length + ' stamps'; return ''; }
function catStale(c, cat) { const s = c.state; const st = { Planet: 3, Geology: 5, Hydrology: 7, Climate: 8, Ecology: 10, Generate: 10 }[cat]; return !!(s.stale && st && st >= s.stale); }
function nodeChips(c) { const s = c.state; return DOMAIN[s.dom].nodes.map((n) => Object.assign({ label: n.short, d: n.d, iconDisp: 'inline', on: s.node[s.dom] === n.id, tap: () => (s.dom === 'civil' && n.id === 'journey' && (c._mode() !== 'tablet' || c._ptr()) ? doAct(c, { act: 'story', arg: 'journeys' }) : c._setNode(n.id)) }, chipStyle(s.node[s.dom] === n.id))); }
function findSub(items, path) { let cur = items; for (const p of path) { const it = cur.find((x) => x && x.t === 'sub' && x.label === p); if (!it) return cur; cur = it.items; } return cur; }
function menusFor(c) { const all = menuTree(); return all; }
function rowsMenuTab(c) {
  const s = c.state, menus = menusFor(c);
  if (!s.menuPath.length) {
    const rows = [H('QUICK', s.quick.length + ' pinned'), quickRow(c), H('MENUS', 'the desktop menu bar, same names, same order')];
    menus.forEach((m) => rows.push({ k: 'nav', d: m.d, label: m.label, sub: m.sub, badge: String(m.items.filter((x) => x && x.t !== 'sep' && x.t !== 'band' && x.t !== 'sign').length), badgeCol: 'var(--dim)', tap: () => c.setState({ menuPath: [m.id] }) }));
    rows.push(H('STATUS'), read('World', s.worldName + ' · ' + s.seed), read('State', s.running ? 'generating…' : (s.stale ? 'stale from ' + String(s.stale).padStart(2, '0') : 'stages 01–10 resolved'), s.stale ? 'var(--warn)' : ''), read('Last autosave', s.savedAt + ' · every ' + s.prefs.autoInt + ' min'), read('Undo depth', s.undo.length + ' steps'), read('Window', c._W() + ' × ' + c._H() + ' dp · ' + sizeClass(c)));
    if (s.stale) rows.push(B('Recompute stale stages', () => c._run(s.stale), 'primary'));
    return rows;
  }
  const m = menus.find((x) => x.id === s.menuPath[0]); const items = findSub(m.items, s.menuPath.slice(1));
  return menuRows(c, items, (it) => c.setState({ menuPath: s.menuPath.concat([it.label]) }), false);
}
function sizeClass(c) { const W = c._W(), H = c._H(); const m = c._mode(); if (m === 'phone') return 'compact · phone'; if (m === 'phoneLand') return 'compact height · phone landscape'; if (W < 840) return 'medium · tablet portrait / fold'; if (W < 1200) return 'expanded · tablet'; if (W < 1600) return 'large · tablet'; return 'extra-large · 4K-class'; }

/* ---------- search: places, factions, labels, routes, landmarks, and every command ---------- */
function commandIndex(c) {
  if (c._cmdIdx) return c._cmdIdx; const out = [];
  const walk = (items, path, menuId) => items.forEach((it) => { if (!it || !it.label || it.t === 'band' || it.t === 'sep' || it.t === 'sign') return; if (it.t === 'sub') { walk(it.items, path.concat([it.label]), menuId); out.push({ kind: 'menu', label: it.label, sub: path.concat([it.label]).join(' ▸ '), it, path: path.concat([it.label]), menuId }); return; } out.push({ kind: it.t === 'readout' ? 'readout' : 'command', label: it.label, sub: path.join(' ▸ ') + (it.accel ? ' · ' + it.accel : ''), it, path, menuId }); });
  menuTree().forEach((m) => walk(m.items, [m.label], m.id));
  Object.values(DOMAIN).forEach((D) => D.nodes.forEach((n) => { out.push({ kind: 'node', label: n.label, sub: D.label + ' ▸ ' + n.label, target: D.id + '/' + n.id }); n.cats.forEach((cat) => out.push({ kind: 'section', label: cat, sub: D.label + ' ▸ ' + n.label + ' ▸ ' + cat, target: D.id + '/' + n.id + '/' + cat })); }));
  PARAMS.filter((p) => p.stage).forEach((p) => out.push({ kind: 'param', label: p.label, sub: 'WORLD ▸ Generate ▸ ' + String(p.stage).padStart(2, '0') + ' ' + STAGE_NAMES[p.stage - 1], param: p }));
  Object.keys(TOOLS).forEach((t) => out.push({ kind: 'tool', label: TOOLS[t].label, sub: 'tool' + (TOOLS[t].key ? ' · ' + TOOLS[t].key : ''), tool: t }));
  c._cmdIdx = out; return out;
}
function runCommand(c, r) {
  const m = c._mode(); c.setState({ srch: false });
  if (r.kind === 'node' || r.kind === 'section') { jump(c, r.target); return; }
  if (r.kind === 'param') { const p = r.param; const grp = p.stage <= 3 ? 'Planet' : p.stage <= 5 ? 'Geology' : p.stage <= 7 ? 'Hydrology' : p.stage === 8 ? 'Climate' : 'Ecology'; const open = Object.assign({}, c.state.open, { ['cat-world-' + grp]: true, ['st-' + String(p.stage).padStart(2, '0')]: true }); c.setState({ open }); jump(c, 'world/pipeline/' + grp); c._toast(p.label + ' · in ' + String(p.stage).padStart(2, '0') + ' ' + STAGE_NAMES[p.stage - 1]); return; }
  if (r.kind === 'tool') { const dom = { settle: 'civil', territory: 'civil', lasso: 'civil', way: 'civil', route: 'civil', conflict: 'civil', icon: 'carto', label: 'carto', colourpaint: 'carto', uplift: 'world', carve: 'world', freehand: 'world', paint: 'world' }[r.tool]; if (dom) c.setState({ dom }); setTimeout(() => c._arm(r.tool), 0); return; }
  const it = r.it;
  if (r.kind === 'menu') { if (m === 'tablet') c.setState({ dd: { menu: r.menuId, path: r.path.slice(1) } }); else c.setState({ tab: 'menu', det: 'half', menuPath: [r.menuId].concat(r.path.slice(1)) }); return; }
  if (it.t === 'check') { setK(c, it.key, !getK(c, it.key)); c._toast(it.label + ' · ' + (getK(c, it.key) ? 'off' : 'on')); return; }
  if (it.t === 'radio') { setK(c, it.key, it.value); c._toast(r.sub + ' ▸ ' + it.label); return; }
  if (it.t === 'todo') { c._toast(it.label + ' · ' + it.reason); return; }
  if (it.t === 'readout') { c._toast(it.label + ' · ' + readoutV(c, it)); return; }
  const why = liveReason(c, it); if (why) { c._toast(why); return; }
  doAct(c, it);
}
function searchResults(c) {
  const s = c.state; let q = (s.q || '').trim().toLowerCase(); let scope = s.scope;
  const m = q.match(/^(s|f|lb|r|\.|l|n)\s+(.*)$/); if (m) { scope = { s: 'places', f: 'factions', lb: 'labels', r: 'routes', '.': 'commands', l: 'landmarks', n: 'notes' }[m[1]]; q = m[2]; }
  const hit = (t) => !q || t.toLowerCase().includes(q); const out = [];
  if (scope === 'all' || scope === 'places') c.places.concat(s.userPlaces).filter((p) => hit(p.name)).forEach((p) => out.push({ kind: p.tier, label: p.name, sub: (p.fac >= 0 ? FACTIONS[p.fac].name : 'unclaimed') + ' · pop ' + (p.pop || 0).toLocaleString('en-US'), d: IC.settle, tap: () => { c.setState({ srch: false }); selectPlace(c, p); } }));
  if (scope === 'all' || scope === 'factions') c.factions.filter((f) => hit(f.name)).forEach((f) => out.push({ kind: 'faction', label: f.name, sub: f.gov + ' · ' + f.places + ' places', d: IC.faction, tap: () => { if (c._mode() === 'tablet') { c.setState({ srch: false }); c._select({ kind: 'faction', id: f.id }, { frame: true }); } else c._openWin('factions:' + f.id); } }));
  if (scope === 'all' || scope === 'labels') (c.regions || []).map((r) => ({ text: r.name, x: r.x, y: r.y })).concat(s.userLabels).filter((l) => hit(l.text)).forEach((l) => out.push({ kind: 'label', label: l.text, sub: 'label', d: IC.label, tap: () => { c.setState({ srch: false }); c.view = Object.assign({}, c.view, { cx: l.x, cy: l.y }); c._dirty = true; } }));
  if (scope === 'all' || scope === 'routes') c.ways.filter((w) => hit(w.a.name + ' ' + w.b.name)).slice(0, 12).forEach((w) => out.push({ kind: w.kind, label: w.a.name + ' → ' + w.b.name, sub: fmtDist(w.km, s.prefs.units), d: IC.way, tap: () => { c.setState({ srch: false }); c.view = Object.assign({}, c.view, { cx: (w.a.x + w.b.x) / 2, cy: (w.a.y + w.b.y) / 2 }); c._dirty = true; } }));
  if ((scope === 'all' && q) || scope === 'notes') notesOf(c).filter((n) => hit(n.name) || (q.length > 2 && hit(n.text))).slice(0, 12).forEach((n) => { const ln = hit(n.name) ? '' : (n.text.split('\n').find((x) => x.toLowerCase().includes(q)) || '').replace(/[#>*=\[\]-]/g, '').trim().slice(0, 60); out.push({ kind: 'note', label: n.name, sub: ln ? '… ' + ln : (NOTE_KINDS[n.kind] || NOTE_KINDS.note)[0] + ' note' + (n.entity ? ' · on the map' : ''), d: IC.notes, tap: () => { c.setState({ srch: false }); if (c._ptr()) { const sl = selFromName(c, n.name) || { kind: 'note', id: n.id }; c._select(sl, { frame: true, tab: 'note' }); return; } c._openVault(''); setTimeout(() => c._noteOpen(n.id, 'read'), 0); } }); });
  if (scope === 'landmarks') c.landmarks.filter((l) => hit(l.name + ' ' + l.kind)).forEach((l) => out.push({ kind: l.kind, label: l.name, sub: l.fam, d: IC.landmark, tap: () => { c.setState({ srch: false }); c._select({ kind: 'landmark', id: l.id }); c.view = Object.assign({}, c.view, { cx: l.x, cy: l.y, s: Math.max(c.view.s, 0.6) }); c._dirty = true; } }));
  if ((scope === 'all' && q) || scope === 'commands') commandIndex(c).filter((r) => hit(r.label) || (q.length > 2 && hit(r.sub || ''))).slice(0, scope === 'commands' ? 60 : 24).forEach((r) => out.push({ kind: r.kind, label: r.label, sub: r.sub, d: r.kind === 'param' ? IC.gen : (r.kind === 'tool' ? IC.inspect : (r.kind === 'node' || r.kind === 'section' ? IC.layers : IC.menu)), tap: () => runCommand(c, r) }));
  return out.slice(0, 80);
}

function buildVals(c) {
  const s = c.state, W = c._W(), H = c._H(), m = c._mode(), L = c._L(), isPhone = m !== 'tablet', land = m === 'phoneLand';
  const v = {
    W, H, L, isPhone, isTablet: !isPhone, themeVars: themeVars(s.light), loading: s.loading, uiScale: c._scale(),
    rootRef: c.rootRef, mapRef: c.mapRef, cvRef: c.cvRef, sheetRef: c.sheetRef, grabRef: c.grabRef, bodyRef: c.bodyRef, searchRef: c.searchRef, noteTaRef: c.noteTaRef, noteMirrorRef: c.noteMirrorRef, noteReadRef: c.noteReadRef, noteReadScrollRef: c.noteReadScrollRef, noteWriteScrollRef: c.noteWriteScrollRef, dockReadRef: c.dockReadRef, dockMirrorRef: c.dockMirrorRef, dockTaRef: c.dockTaRef, askScrollRef: c.askScrollRef, askInRef: c.askInRef,
    densVars: c._ptr() ? '--c44:30px;--c48:32px;--c52:34px;--c40:28px;--c56:34px;--i32:24px;--rowpad:3px;--r22:15px;--r24:16px;--r20:14px;' : '', ptr: c._ptr(),
    mapCursor: s.tool === 'inspect' ? 'grab' : 'crosshair', noCtx: (e) => e.preventDefault(),
    // The map's accessible name lists its gestures, and says when thumb zoom (Ruling EB item 3) is off: while a drawing
    // tool is armed, pending DS13-Q1 (c._thumbZoomOn). Every form keeps the "map: drag to pan" opening the tests find
    // it by. The desktop with a mouse keeps its own wording: it has no touch gestures.
    mapAria: c._ptr() ? 'map: drag to pan, pinch to zoom, long-press to sample' : (c._thumbZoomOn() ? 'map: drag to pan, pinch to zoom, double-tap to zoom in, double-tap and hold then slide down to zoom in or up to zoom out, long-press to sample' : 'map: drag to pan with two fingers or with ✋ on (one finger draws with this tool), pinch to zoom, long-press to sample; double-tap and thumb zoom are off while a drawing tool is armed, pending DS13-Q1'),
    press: s.press ? { show: true, x: s.press.x, y: s.press.y } : { show: false, x: 0, y: 0 },
    world: { name: s.worldName, seed: s.seed, saved: 'autosaved ' + s.savedAt, sub: s.seed + ' · ' + (s.running ? 'generating…' : (s.stale ? 'stale from ' + String(s.stale).padStart(2, '0') : 'resolved')) + ' · ' + domainDef(s.dom).label.toLowerCase() },
    undoLabel: s.undo.length ? s.undo[s.undo.length - 1] : 'nothing', undoCol: s.undo.length ? 'var(--acc)' : 'var(--dim)', undoN: String(s.undo.length), undoDot: s.undo.length ? 'block' : 'none',
    redoLabel: s.redo.length ? s.redo[s.redo.length - 1] : 'nothing', redoCol: s.redo.length ? 'var(--acc)' : 'var(--dim)', redoN: s.redo.length ? '· ' + s.redo.length : '',
    labelText: s.labelText, q: s.q, year: s.year, playAria: s.playing ? 'pause' : 'play', playD: s.playing ? IC.pause : IC.play,
    compass: { show: Math.abs(c.view.r) > 0.02, deg: Math.round(c.view.r * 180 / Math.PI) }
  };
  const sc = niceScale(1 / Math.max(1e-6, c.view.s)); v.scale = { disp: s.prefs && V(c, 'v.scale', true) ? 'flex' : 'none', px: Math.round(sc.px), label: fmtDist(sc.km, s.prefs.units) };
  /* ---- handlers ---- */
  Object.assign(v, {
    hWorldPill: () => c.setState({ tab: 'menu', menuPath: ['file'], det: 'half' }),
    hUndoDown: () => { clearTimeout(c._uT); c._uHeld = false; c._uT = setTimeout(() => { c._uHeld = true; c._haptic(); c.setState({ hist: true }); }, 520); },
    hUndoUp: () => { clearTimeout(c._uT); if (!c._uHeld) c._undo(); c._uHeld = false; }, hUndoLeave: () => clearTimeout(c._uT),
    hUndo: () => c._undo(), hRedo: () => c._redo(), hSearch: () => c.setState({ srch: true, q: '', scope: 'all' }),
    hNorth: () => { c.view = Object.assign({}, c.view, { r: 0 }); c._dirty = true; c.forceUpdate(); },
    h3D: () => setK(c, 'dim3', !s.dim3), hFit: () => c._fit(true),
    // The desktop pointer column's + and −: the same step as the touch rocker, ZOOM_BUTTON_STEP 1.35, which Ruling EB
    // item 2 keeps (it stepped by 1.6 here before; MC_ZOOM_STEP in d_screens.js says why there is one constant).
    hZoomIn: () => { const { W: w, H: h } = c._hostWH(); c._zoomAnim(MC_ZOOM_STEP, w / 2, h / 2); }, hZoomOut: () => { const { W: w, H: h } = c._hostWH(); c._zoomAnim(1 / MC_ZOOM_STEP, w / 2, h / 2); },
    hToolDown: (e) => c._toolDown(e), hToolUp: (e) => c._toolUp(e), hToolLeave: () => {},
    hLabelText: (e) => c.setState({ labelText: e.target.value }),
    hBack: () => c._back(), hClose: () => { c.setState({ tab: null }); }, hSheetWide: () => c.setState({ wide: !s.wide }),
    hCoach: () => c._coachDone(), hHam: () => c.setState({ dd: s.dd && s.dd.menu === 'ham' ? null : { menu: 'ham', path: [] } }),
    hLeftDock: () => c.setState({ ldockOpen: !s.ldockOpen }), hPinLeft: () => c.setState({ ldockPin: !s.ldockPin }), hRightDock: () => c.setState({ rdockOpen: !s.rdockOpen }),
    hLayersPop: () => c.setState({ layersPop: !s.layersPop }), hTimeline: () => c.setState({ tlOpen: !s.tlOpen }), hPlay: () => c._play(), hYear: (e) => c._setYear(+e.target.value),
    hSelBack: () => c._selBack(), hSelFwd: () => c._selFwd(), hCtxPin: () => { const sj = subjectOf(c); c.setState({ ctxPin: s.ctxPin ? null : (sj ? Object.assign({}, sj) : null) }); },
    hDnInput: (e) => c._dnInput(e), hDnKey: (e) => c._dnKey(e), hDnClick: (e) => c._writeClick(e, 'dock'), hDnSel: (e) => c._caretLink(e.target, 'dock'),
    hWinPop: () => { const W0 = s.win; const id = W0.stack[W0.stack.length - 1]; const out = W0.popped !== id; c.setState({ win: Object.assign({}, W0, { popped: out ? id : null }) }); c._toast(out ? 'Popped out (mockup) · on a desktop it is its own window: drag it to a second screen; the map stays live' : 'Docked back in the main window'); },
    hAskInput: (e) => c.setState({ ask: Object.assign({}, s.ask, { draft: e.target.value }) }), hAskKey: (e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); c._askSend(); } }, hAskSend: () => c._askSend(),
    hAskOn: () => setK(c, 'prefs.assistant', true), hAskSettings: () => c._openWin('assistantSettings'), hAskClear: () => c.setState({ ask: Object.assign({}, s.ask, { msgs: [], busy: false }) }),
    hWinDragDown: (e) => c._winDragDown(e), hWinDragMove: (e) => c._winDragMove(e), hWinDragUp: () => c._winDragUp(),
    hWinFollow: () => { const W0 = s.win; const fol = W0.follow !== false; c.setState({ win: Object.assign({}, W0, { follow: !fol, pin: fol ? (subjectOf(c) ? Object.assign({}, subjectOf(c)) : null) : null }) }); },
    hDdScrim: () => c.setState({ dd: null }), hDdBack: () => c.setState({ dd: Object.assign({}, s.dd, { path: s.dd.path.slice(0, -1) }) }), hCtxClose: () => { c.setState({ tctx: null, sample: null }); c._dirty = true; },
    hFanScrim: () => c.setState({ fan: null }), hHistClose: () => c.setState({ hist: false }), hSearchClose: () => c.setState({ srch: false }), hQ: (e) => c.setState({ q: e.target.value }),
    hWinClose: () => c.setState({ win: null }), hWinBack: () => c.setState({ win: { stack: s.win.stack.slice(0, -1) } }), hNewWorld: () => c._openWin('newworld'), hOpenZip: () => c._toast('System file picker · project .zip')
  });
  const tb = toolBar(c);
  /* ---- phone ---- */
  if (isPhone) {
    const sheetOpen = !!s.tab; const sh = land ? 0 : (sheetOpen ? c._detH(s.det) : 0); const sw = land ? (sheetOpen ? c._sheetW() : 0) : 0;
    const showStrip = s.tool !== 'inspect' && !(s.tab && s.det === 'full' && !land);
    const stripH = showStrip ? 66 : 0;
    const free = land ? H : H - NAV_H - sh - (INSET_T + 60) - stripH;
    v.fab = { bottom: land ? 16 + stripH : NAV_H + sh + 12 + stripH, scaleBottom: land ? 18 + stripH : NAV_H + sh + 14 + stripH, disp: free >= 120 ? 'flex' : 'none', more: free >= 250, dimLabel: s.dim3 ? '2D' : '3D', dimAria: s.dim3 ? 'switch to 2D' : 'switch to 3D' };
    v.qd = quickDrawerVals(c, land, sh, stripH);
    // DS-13: the control row on the dominant-hand side; the scale bar takes the other side.
    const handL = s.prefs.hand === 'left';
    v.mc = mapCtlVals(c, { show: free >= 120, bottom: v.fab.bottom, edge: land ? (handL ? RAIL_W + 12 : sw + 12) : 12 });
    v.fab.scaleLeft = handL ? 'auto' : L.scaleL + 'px'; v.fab.scaleRight = handL ? (land ? sw + 14 : 14) + 'px' : 'auto';
    v.strip = Object.assign({ show: showStrip, bottom: land ? 12 : NAV_H + sh + 10, toolD: tb.d, toolShort: tb.short, toolLabel: tb.label, main: tb.main, hint: tb.hint, field: tb.field, acts: tb.acts }, {});
    let title = '', sub = '', chips = [], rows = [], back = false, caret = false, titleTap = () => {}, titleAria = '';
    if (s.tab === 'map') { title = 'MAP'; sub = 'view · fields · layers · measure'; rows = rowsMap(c); }
    else if (s.tab === 'dom') { const D = DOMAIN[s.dom]; title = D.label + ' · ' + nodeDef(s.dom, s.node[s.dom]).short; sub = D.sub; chips = nodeChips(c); rows = domainRows(c, false); caret = true; titleTap = () => c._openFan('dom', null); titleAria = 'switch domain'; }
    else if (s.tab === 'story') { title = s.storySeg === 'journeys' && s.planStage >= 0 ? 'STORY · STAGE ' + (s.planStage + 1) : 'STORY'; sub = { time: 'time · year ' + s.year, journeys: 'journey · ' + (c.route[0] ? c.route[0].name + ' → ' + c.route[c.route.length - 1].name : ''), conflicts: 'conflicts & campaigns', notes: 'notes · vault · assistant' }[s.storySeg]; chips = [['time', 'TIME', IC.time], ['journeys', 'JOURNEYS', IC.journey], ['conflicts', 'CONFLICTS', IC.conflict], ['notes', 'NOTES', IC.notes]].map(([id, l, d]) => Object.assign({ label: l, d, iconDisp: 'inline', on: s.storySeg === id, tap: () => c.setState({ storySeg: id, planStage: -1 }) }, chipStyle(s.storySeg === id))); rows = rowsStory(c); back = s.storySeg === 'journeys' && s.planStage >= 0; }
    else if (s.tab === 'menu') { const menus = menuTree(); const mm = menus.find((x) => x.id === s.menuPath[0]); title = mm ? [mm.label].concat(s.menuPath.slice(1)).join(' ▸ ').toUpperCase() : 'MENU'; sub = mm ? mm.sub : 'the desktop menus · same names · same order'; rows = rowsMenuTab(c); back = s.menuPath.length > 0; }
    else if (s.tab === 'ctx') { const a = sampleAt(c, s.sample); title = 'SAMPLE · ' + (a ? a.elev + ' m' : ''); sub = s.sample ? Math.round(s.sample.x) + ' km E · ' + Math.round(s.sample.y) + ' km S · ' + (a ? a.biome : '') : ''; rows = rowsSample(c); }
    else if (s.tab === 'sel') { const o = selObj(c); title = o ? o.name.toUpperCase() : 'SELECTION'; sub = o ? (o.tier || o.kind) + (o.fac >= 0 ? ' · ' + FACTIONS[o.fac].name : '') : ''; rows = rowsSel(c); }
    const style = land ? 'top:0;bottom:0;right:0;width:' + sw + 'px;border-left:1px solid var(--hair);transition:width .28s cubic-bezier(.3,.9,.3,1);overflow:hidden' : 'left:0;right:0;bottom:' + NAV_H + 'px;height:' + sh + 'px;border-radius:22px 22px 0 0;transition:height .28s cubic-bezier(.3,.9,.3,1);overflow:hidden';
    v.sheet = { disp: 'flex', style, handleH: land ? 8 : 20, handleDisp: land ? 'none' : 'block', back, title, sub, titleTap, titleAria: titleAria || title, caretDisp: caret ? 'block' : 'none', pinBtn: land, wideAria: s.wide ? 'narrow panel' : 'widen panel', wideD: s.wide ? 'M10 3l-5 5 5 5M14 3v10' : 'M6 3l5 5-5 5M2 3v10', chips, hasChips: chips.length > 0, chipsAria: s.tab === 'dom' ? 'nodes' : 'sections', rows };
    const D = DOMAIN[s.dom]; const domOn = s.tab === 'dom';
    const cells = [{ id: 'map', label: 'MAP', d: IC.map }, { id: 'dom', label: D.tab, d: D.d }, { id: 'story', label: 'STORY', d: IC.story }, { id: 'menu', label: 'MENU', d: IC.menu }].map((n) => {
      const on = n.id === 'dom' ? domOn : s.tab === n.id;
      return { id: n.id, label: n.label, d: n.d, on, pill: on ? 'var(--wash)' : 'transparent', col: on ? 'var(--acc)' : 'var(--sec)', holding: s.holdNav === n.id, dotsDisp: 'flex', dots: n.id === 'dom' ? DOMS.map((d) => ({ c: d === s.dom ? 'var(--acc)' : 'var(--bord)' })) : [], aria: n.id === 'dom' ? D.label + ' domain: tap opens it, hold or swipe switches to ' + DOMS.filter((d) => d !== s.dom).map((d) => DOMAIN[d].label).join(' or ') : n.label, down: (e) => c._navDown(n.id, e), move: (e) => c._navMove(e), up: (e) => c._navUp(n.id, e), cancel: () => c._navCancel() };
    });
    v.nav = land ? { style: 'left:0;top:0;bottom:0;width:' + RAIL_W + 'px;flex-direction:column;justify-content:center;gap:6px;border-right:1px solid var(--hair2);padding:8px 0;box-sizing:border-box', cellStyle: 'width:100%;height:72px', gesture: false, cells } : { style: 'left:0;right:0;bottom:0;height:' + NAV_H + 'px;flex-direction:row;padding:0 4px 16px;border-top:1px solid var(--hair2);box-sizing:border-box', cellStyle: 'flex:1;min-width:0;height:64px', gesture: false, cells };
    const cw = land ? RAIL_W : (W - 8) / 4; const cx = land ? RAIL_W + 12 : Math.max(10, Math.min(W - 290, 4 + cw * 1.5 - 140));
    v.coach = { show: s.coach && !s.loading && !s.fan && !s.srch && !s.win && !s.picker && !s.tab, x: cx, y: land ? Math.max(8, H / 2 - 120) : H - NAV_H - 112, w: 280, arrow: land ? -40 : Math.max(16, Math.min(250, 4 + cw * 1.5 - cx - 7)), text: 'Hold ' + D.tab + ' to switch to ' + DOMS.filter((d) => d !== s.dom).map((d) => DOMAIN[d].label).join(' or ') + ' — or swipe it sideways. Long-press the map to sample.' };
    v.toastB = null; v.toastT = land ? 70 : INSET_T + 64;
  } else {
    v.fab = { dimLabel: s.dim3 ? '2D' : '3D', dimAria: s.dim3 ? 'switch to 2D' : 'switch to 3D' };
    v.strip = { show: false, acts: [] }; v.sheet = { disp: 'none', chips: [], hasChips: false, rows: [] }; v.nav = { cells: [], style: 'display:none', cellStyle: '', gesture: false }; v.coach = { show: false };
  }
  /* ---- tablet ---- */
  if (!isPhone) {
    const G = L.G; const menus = menuTree(); const ptr = c._ptr(); const wide = W >= 1180 || ptr;
    const visible = wide ? menus : [tabletFileMenu(menus), tabletWorldMenu(menus), menus.find((x) => x.id === 'records'), menus.find((x) => x.id === 'story')];
    let mx = 14 + 98 + (wide ? 0 : 50); const pos = {};
    const menuBtns = visible.map((mm) => { const w = Math.round(mm.label.length * (ptr ? 7.0 : 7.4) + (ptr ? 18 : 26)); pos[mm.id] = mx; mx += w + 2; const on = !!(s.dd && s.dd.menu === mm.id); return { label: mm.label, on, col: on ? 'var(--acc)' : 'var(--body)', bg: on ? 'var(--wash)' : 'transparent', tap: () => c.setState({ dd: on ? null : { menu: mm.id, path: [] }, layersPop: false }), enter: () => { if (ptr && c.state.dd && c.state.dd.menu !== mm.id && c.state.dd.menu !== 'ham') c.setState({ dd: { menu: mm.id, path: [] } }); } }; });
    pos.ham = 14 + 98;
    let dd = { show: false, rows: [] };
    let ddCols = { show: false, cols: [] };
    if (s.dd && ptr) { const src = visible.find((x) => x.id === s.dd.menu) || menus.find((x) => x.id === s.dd.menu); if (src) { const cols = deskMenuCols(c, src, s.dd.path); const cw = 286; const x0 = Math.max(8, Math.min(W - cw * cols.length - 8, pos[s.dd.menu] != null ? pos[s.dd.menu] : 120)); ddCols = { show: true, x: x0, cw, maxH: H - G.menuH - 24, cols }; } }
    else if (s.dd) { const src = s.dd.menu === 'ham' ? tabletOverflow(menus) : (visible.find((x) => x.id === s.dd.menu) || menus.find((x) => x.id === s.dd.menu)); if (src) { const items = findSub(src.items, s.dd.path); const ddw = Math.min(360, W - 24); dd = { show: true, x: Math.max(8, Math.min(W - ddw - 8, pos[s.dd.menu] != null ? pos[s.dd.menu] : 120)), w: ddw, maxH: H - G.menuH - 24, title: (s.dd.path.length ? s.dd.path[s.dd.path.length - 1] : src.label).toUpperCase(), back: s.dd.path.length > 0, rows: menuRows(c, items, (it) => c.setState({ dd: Object.assign({}, s.dd, { path: s.dd.path.concat([it.label]) }) }), true) }; } }
    const ldOverlay = !G.leftDocks; const ldW = ldOverlay ? Math.min(380, W - G.railW - 48) : G.ldW;
    const D = DOMAIN[s.dom], nd = nodeDef(s.dom, s.node[s.dom]);
    const rail = DOMS.map((d) => ({ label: DOMAIN[d].label, on: s.dom === d, h: ptr ? 84 : 92, bg: s.dom === d ? 'var(--wash)' : 'transparent', edge: s.dom === d ? 'var(--acc)' : 'transparent', col: s.dom === d ? 'var(--acc)' : 'var(--sec)', tap: () => c._setDom(d), nodes: DOMAIN[d].nodes.map((n) => ({ d: n.d, label: DOMAIN[d].label + ' ▸ ' + n.label, on: s.node[d] === n.id, bg: s.node[d] === n.id ? 'var(--wash)' : 'transparent', col: s.node[d] === n.id ? 'var(--acc)' : 'var(--sec)', tap: () => { if (ptr && d === 'civil' && n.id === 'journey') { doAct(c, { act: 'story', arg: 'journeys' }); return; } c.setState({ dom: d, node: Object.assign({}, s.node, { [d]: n.id }), ldockOpen: true, tool: 'inspect' }); } })) }));
    const rdOverlay = !G.rightDocks; const mapR = s.rdockOpen && !rdOverlay ? G.rdW : G.rtabW; const mapL = L.mapL;
    const toolRows = [];   // draft state of the armed tool: shown at the top of Inspect, whatever is selected
    if (s.stamps.length && s.dom === 'world') toolRows.push(H('STAMP STACK', s.stamps.length + ''), ...s.stamps.slice(-5).map((st, i) => read((i + 1) + ' · ' + st.feature, st.lower ? 'lower' : 'raise')), BB({ label: 'Discard draft', tap: () => discardStamps(c) }, { label: 'Commit to map', variant: 'primary', tap: () => commitStamps(c) }));
    if (s.tool === 'way' || s.tool === 'route') toolRows.push(H('WAY DRAFT', s.wayDraft.length + ' points'));
    let tctx = { show: false, rows: [] };
    if (s.tctx) { const cw = 320; const maxH = Math.min(560, H - G.bodyT - G.bodyB - 16); const rc = s.ring ? ringCentre(c, s.ring) : null; const gap = rc ? RING.disc + 16 : 20; const ax = rc ? rc.x : s.tctx.x; let x = s.prefs.hand === 'left' ? ax + gap : ax - cw - gap; if (s.prefs.hand !== 'left' && x < mapL + 8) x = ax + gap; if (s.prefs.hand === 'left' && x + cw > W - 8) x = ax - cw - gap; x = Math.max(8, Math.min(W - cw - 8, x)); const y = Math.max(G.bodyT + 8, Math.min(H - G.bodyB - maxH - 8, s.tctx.y - 60)); const a = sampleAt(c, s.sample); tctx = { show: true, x, y, maxH, title: 'SAMPLE · ' + (a ? a.elev + ' m' : ''), sub: s.sample ? Math.round(s.sample.x) + ' km E · ' + Math.round(s.sample.y) + ' km S' : '', rows: rowsSample(c) }; }
    v.T = {
      menuH: G.menuH, toolH: G.toolH, railW: G.railW, statusH: G.statusH, tlH: G.tlH, tlBarH: G.tlBarH, ddCols, ptr, menuPad: ptr ? 9 : 12, menuFont: ptr ? 13 : 13.5, bodyT: G.bodyT, bodyB: G.bodyB, mapL,
      ham: !wide, hamOpen: !!(s.dd && s.dd.menu === 'ham'), hamBg: s.dd && s.dd.menu === 'ham' ? 'var(--wash)' : 'transparent', menus: menuBtns,
      searchW: W >= 1920 ? 360 : (W >= 1180 ? 260 : 48), searchTextDisp: W >= 1180 ? 'inline' : 'none', readoutDisp: W >= 1440 ? 'inline' : 'none', sysDisp: W >= 1920 ? 'inline' : 'none', sys: '2048² · CPU 8 workers · GPU Vulkan · 1.4 GB of 8 GB',
      toolName: tb.main, toolChips: tb.chips.map((x) => Object.assign({ label: x.label, tap: x.tap, on: !!x.on }, chipStyle(!!x.on))), toolHint: tb.hint, toolActs: tb.acts.map((a) => ({ label: a.label, tap: a.tap, bg: a.bg, col: a.col })),
      rail, ldockAria: s.ldockOpen ? 'hide the left panel' : 'show the left panel',
      ldock: { show: s.ldockOpen, w: ldW, shadow: ldOverlay ? '16px 0 40px rgba(0,0,0,.45)' : 'none', z: ldOverlay ? 9 : 5, title: D.label + ' › ' + nd.label.toUpperCase(), sub: ldOverlay ? 'overlay · the map keeps ' + (W - G.railW - G.rtabW) + (ptr ? ' px' : ' dp') : 'docked · ' + ldW + (ptr ? ' px' : ' dp'), canPin: ldOverlay || s.ldockPin, pinned: s.ldockPin, pinCol: s.ldockPin ? 'var(--acc)' : 'var(--sec)', pinAria: s.ldockPin ? 'unpin: float over the map' : 'pin: dock beside the map', chips: nodeChips(c).map((x) => Object.assign({}, x)), rows: domainRows(c, true) },
      rdock: ctxDockVals(c, G, rdOverlay, toolRows),
      rtab: !s.rdockOpen || rdOverlay, rtabW: G.rtabW, rtabCol: s.sel || s.sample ? 'var(--acc)' : 'var(--dim)', rtabLabel: 'CONTEXT',
      furnT: G.bodyT + 12, furnR: mapR + 12, furnB: G.bodyB + 14, scaleL: mapL + 14, scaleB: G.bodyB + 16,
      touchNav: !ptr, ptrNav: !!ptr,
      // The 3D card's HEIGHT row reads and writes the one preference (v3ExagSet, d_screens.js; Ruling EB items 5 and 6).
      v3: { disp: s.dim3 ? 'flex' : 'none', furnDisp: s.dim3 ? 'none' : 'flex', left: mapL + 12, top: G.bodyT + 12, w: Math.min(460, W - mapL - mapR - 24), val: s.prefs.v3exag, min: V3_EXAG_MIN, max: V3_EXAG_MAX, step: V3_EXAG_STEP, aria: 'height exaggeration, ' + V3_EXAG_MIN + ' to ' + V3_EXAG_MAX + ' times', where: V3_EXAG_WHERE, label: '×' + Number(s.prefs.v3exag).toFixed(2), onIn: (e) => v3ExagSet(c, e.target.value), back: () => setK(c, 'dim3', false), retest: () => c._toast('Testing 3D on this device… about 3 seconds (mock)'), force: () => c._toast('Force tier… · Automatic (as measured) · Off · Basic · Standard · Full (mock)') }, mc: mapCtlVals(c, { show: !ptr, bottom: G.bodyB + 14, edge: s.prefs.hand === 'left' ? mapL + 12 : mapR + 12 }),
      scaleLeft: !ptr && s.prefs.hand === 'left' ? 'auto' : (mapL + 14) + 'px', scaleRight: !ptr && s.prefs.hand === 'left' ? (mapR + 14) + 'px' : 'auto', layersT: G.bodyT + 62, popMaxH: H - G.bodyT - G.bodyB - 80,
      layersOpen: s.layersPop, layerRows: rowsMap(c), baseLabel: ((ANALYSIS.flatMap((g) => g[1]).find((f) => f[0] === (s.field || 'off')) || ['', 'Relief'])[1]).toUpperCase().replace('NO OVERLAY (BASE MAP)', 'BASE MAP'),
      dd: ptr ? { show: false, rows: [] } : dd, ctx: tctx, tlOpen: s.tlOpen, tlRot: s.tlOpen ? 180 : 0,
      quick: quickCluster(c, W), ring: ringVals(c)
    };
    v.qd = { show: false, rows: [] };
    // DS-13: on touch the map's control row holds the bottom band (bodyB + 14, 44 tall), so toasts stack 20 above it.
    v.toastB = G.bodyB + 20 + (ptr ? 0 : 58); v.toastT = null;
    v.status = { left: s.running ? 'generating · stage ' + String(s.running.i + 1).padStart(2, '0') : (s.stale ? 'stale from stage ' + String(s.stale).padStart(2, '0') : 'stages 01–10 resolved'), mid: (subjectOf(c) && entOf(c, subjectOf(c)) ? (s.ctxPin ? 'pinned ' : 'selected ') + entOf(c, subjectOf(c)).name + ' · ' : '') + 'year ' + s.year + ' · ' + s.worldName.toLowerCase() + ' · seed ' + s.seed + ' · ' + (s.tool !== 'inspect' ? 'tool ' + TOOLS[s.tool].label.toLowerCase() : 'inspect') + ' · autosave ' + s.savedAt, right: ptr ? W + ' × ' + H + ' · desktop · docks ' + (G.rightDocks ? 'both' : (G.leftDocks ? 'left' : 'overlay')) : W + '×' + H + ' dp · ' + sizeClass(c) + ' · docks ' + (G.rightDocks ? 'both' : (G.leftDocks ? 'left' : 'overlay')) };
  } else { v.T = { menus: [], rail: [], ldock: { show: false, chips: [], rows: [] }, rdock: Object.assign(ctxDockVals(c, { rdW: 0 }, false, []), { show: false }), ddCols: { show: false, cols: [] }, dd: { show: false, rows: [] }, ctx: { show: false, rows: [] }, toolChips: [], toolActs: [], layerRows: [], rtab: false }; v.status = { left: '', mid: '', right: '' }; }
  /* ---- shared overlays ---- */
  const evs = tlEvents(c).map((e) => [e.y, e.t + (e.src === 'note' ? ' · ' + e.note + '.md' : ''), e]);
  v.events = evs.map(([y, t, e]) => { const on = s.year === y || (e.y1 != null && s.year >= y && s.year <= e.y1); return { y: 'YEAR ' + y + (e.y1 != null ? '–' + e.y1 : ''), t, bg: on ? 'var(--wash)' : 'var(--chip)', bord: on ? 'var(--acc)' : (e.src === 'note' ? 'rgba(224,163,74,.35)' : 'transparent'), tap: () => { if (e.sel && c._mode() === 'tablet') c._select(e.sel, { frame: true, year: y }); else c._setYear(y); } }; });
  v.ticks = tlTicks(c); v.yrMin = YEAR_MIN; v.yrMax = YEAR_MAX; v.tlc = tlControls(c);
  const ev = evs.filter((e) => e[0] <= s.year).pop(); v.yearEvent = ev ? ev[1] : '';
  if (s.fan) {
    const isDom = s.fan.kind === 'dom'; const fw = Math.min(330, W - 24); const n = s.fan.items.length; const fh = n * 72 + 64;
    const a = c._fanAnchor || { ax: W / 2, ay: H - NAV_H };
    const fx = land ? RAIL_W + 10 : Math.max(12, Math.min(W - fw - 12, (a.ax || W / 2) - fw / 2)); const fy = land ? Math.max(8, Math.min(H - fh - 8, (a.ay || H / 2) - fh / 2)) : Math.max(INSET_T + 8, (a.ay || H - NAV_H) - fh - 8);
    v.fan = { show: true, x: fx, y: fy, w: fw, title: isDom ? 'SWITCH DOMAIN' : 'TOOLS · ' + nodeDef(s.dom, s.node[s.dom]).label.toUpperCase(), hint: isDom ? 'slide and release, or tap · swipe the tab to cycle' : 'slide and release, or tap', items: s.fan.items.map((it, i) => { const hov = s.fan.hover === i; const on = isDom ? it.d === s.dom : it.on; const meta = isDom ? { d: DOMAIN[it.d].d, label: it.d === 'world' ? 'WORLD · GENERATE' : DOMAIN[it.d].label, sub: DOMAIN[it.d].nodes.map((x) => x.label).join(' · ') } : { d: TOOLS[it.tool].d, label: TOOLS[it.tool].label.toUpperCase(), sub: TOOLS[it.tool].hint || '' }; return Object.assign(meta, { i, on, tap: () => it.pick(), bord: hov ? 'var(--acc)' : 'var(--bord)', bg: hov ? 'var(--pan)' : 'var(--pan)', scale: hov ? 1.03 : 1, iconBg: on || hov ? 'var(--wash)' : 'var(--chip)', iconCol: on || hov ? 'var(--acc)' : 'var(--sec)', col: on ? 'var(--acc)' : 'var(--ink)', checkOp: on ? 1 : 0 }); }) };
  } else v.fan = { show: false, items: [] };
  v.hist = s.hist ? { show: true, r: isPhone ? 10 : 120, t: isPhone ? (land ? 64 : INSET_T + 60) : 50, rows: s.undo.slice().reverse().slice(0, 8).map((u, i) => ({ n: String(s.undo.length - i), label: u, kind: i === 0 ? 'latest' : '', col: i === 0 ? 'var(--ink)' : 'var(--body)', tap: () => c._rollTo(s.undo.length - i - 1) })).concat(s.undo.length ? [] : [{ n: '', label: 'nothing to undo yet', kind: '', col: 'var(--dim)', tap: () => c.setState({ hist: false }) }]) } : { show: false, rows: [] };
  if (s.srch) {
    const res = searchResults(c); const sw2 = isPhone ? W - (land ? RAIL_W + 16 : 16) : Math.min(640, W - 80);
    const scopes = [['all', 'all'], ['places', 's settlements'], ['factions', 'f factions'], ['labels', 'lb labels'], ['routes', 'r routes'], ['commands', '. commands'], ['landmarks', 'l landmarks'], ['notes', 'n notes']];
    v.srch = { show: true, x: isPhone ? (land ? RAIL_W + 8 : 8) : (W - sw2) / 2, y: isPhone ? (land ? 8 : INSET_T + 4) : 60, w: sw2, maxH: isPhone ? H - (land ? 16 : INSET_T + 16) : Math.min(720, H - 120), scopes: scopes.map(([id, l]) => Object.assign({ label: l, n: '', on: s.scope === id, tap: () => c.setState({ scope: id }) }, chipStyle(s.scope === id))), results: res, empty: !res.length, emptyText: s.q ? 'No match for “' + s.q + '”. Try a prefix: s, f, lb, r, ., l or n.' : 'Type a place, faction, route, setting or command. Every menu row, dock section and generation parameter is findable here, and runs from here.' };
  } else v.srch = { show: false, scopes: [], results: [] };
  if (s.win) {
    const id = s.win.stack[s.win.stack.length - 1]; const ptr = c._ptr(); const fol = s.win.follow !== false; const canFollow = ptr && /^(factions|place|city|conflicts?|mod)(:|$)/.test(id) && !/^factions:(all|types|history)/.test(id);
    c._selOverride = canFollow && !fol ? (s.win.pin || null) : null; const sj = c._selOverride || subjectOf(c); const ej = entOf(c, sj);
    c._winFac = canFollow && !/^factions:f/.test(id) ? (ej ? (ej.kind === 'faction' ? ej.i : (ej.kind === 'place' ? (ownerAt(c, ej.o, s.year) != null && ownerAt(c, ej.o, s.year) >= 0 ? ownerAt(c, ej.o, s.year) : ej.o.fac) : (ej.kind === 'conflict' ? ej.o.sides[0] : -1))) : -1) : -1;
    const wr = winRows(c, id); c._selOverride = null; c._winFac = -1;
    const ww = isPhone ? (land ? W - RAIL_W : W) : Math.min(wr.wide ? 1000 : (W >= 1920 ? 760 : 680), W - 80); const wh = isPhone ? (land ? H : H - INSET_T - 4) : Math.min(H - 96, wr.wide ? 780 : (W >= 1920 ? 960 : 820));
    const twoCol = !!wr.left && (isPhone ? land && ww >= 700 : ww >= 900); const leftW = isPhone ? 316 : Math.round((ww - 54) * 0.56);
    let wx = isPhone ? (land ? RAIL_W : 0) : (W - ww) / 2, wy = isPhone ? (land ? 0 : INSET_T + 4) : (H - wh) / 2; let ww2 = ww, wh2 = wh;
    if (ptr) { ww2 = Math.min(wr.wide ? 900 : 560, W - 120); wh2 = Math.min(H - L.G.bodyT - L.G.bodyB - 40, wr.wide ? 720 : 640); const dx = L.G.railW + (s.ldockOpen && L.G.leftDocks ? L.G.ldW : 0) + 24; wx = s.winPos ? s.winPos.x : dx; wy = s.winPos ? s.winPos.y : L.G.bodyT + 16; c._winBox = { x: wx, y: wy }; }
    v.win = { show: true, modal: !ptr, title: wr.title, sub: (canFollow ? (fol ? 'follows the selection · ' : 'pinned · ') : '') + wr.sub, rows: twoCol || !wr.left ? wr.rows : wr.left.concat(wr.rows), left: twoCol ? wr.left : [], twoCol: twoCol && (!ptr || ww2 >= 900), leftW, back: s.win.stack.length > 1, x: wx, y: wy, w: ww2, h: wh2, radius: isPhone ? (land ? '0' : '22px 22px 0 0') : (ptr ? '12px' : '18px'), canFollow, follow: fol, followCol: fol ? 'var(--acc)' : 'var(--sec)', followBg: fol ? 'var(--wash)' : 'transparent', followAria: fol ? 'pin this window to what it shows now' : 'follow the selection again', followLabel: fol ? 'FOLLOW' : 'PINNED', popout: ptr && !!wr.popout, popLabel: s.win.popped === id ? 'DOCK' : '⧉ POP OUT', popAria: s.win.popped === id ? 'dock: back into the main window' : 'pop out: its own window, which can go on a second screen', grab: ptr ? 'move' : 'default', shadow: ptr ? '0 14px 40px rgba(0,0,0,.5)' : '0 20px 50px rgba(0,0,0,.55)' };
  } else v.win = { show: false, rows: [] };
  const grads = ['linear-gradient(135deg,#2c4a5e,#4a7a52 55%,#c9b57a)', 'linear-gradient(135deg,#33415c,#5c677d 60%,#d9dcd6)', 'linear-gradient(135deg,#5a3a32,#7d5a4a 50%,#c9a27a)', 'linear-gradient(135deg,#1b3a2c,#3c6e51 60%,#a9c48a)', 'linear-gradient(135deg,#3b3f52,#6a6f8a 60%,#c8c3b8)'];
  v.picker = s.picker ? { show: true, pad: isPhone ? (land ? '16px 16px 16px ' + (RAIL_W + 0) + 'px' : (INSET_T + 16) + 'px 16px 24px') : '40px', maxW: 1100, cols: isPhone ? (land ? 2 : 1) : (W >= 1100 ? 3 : 2), worlds: [[s.worldName, s.seed, 'OPEN NOW', 'var(--acc)'], ['VHAREN REACH', 129384, 'IN PROGRESS', 'var(--sec)'], ['KESSA', 774201, 'DRAFT', 'var(--sec)'], ['THORNWOOD', 908133, 'ARCHIVE', 'var(--dim)'], ['HIGH SADDLE', 441907, 'STALE FROM 07', 'var(--warn)']].map((x, i) => ({ name: x[0], sub: x[1] + ' · 2 048² · edited ' + ['now', '5 d ago', '3 w ago', '2 mo ago', '5 d ago'][i], badge: x[2], badgeCol: x[3], grad: grads[i], tap: () => (i === 0 ? c.setState({ picker: false }) : doAct(c, { act: 'openWorld', arg: x[0] })) })) } : { show: false, worlds: [] };
  v.toasts = s.toasts; v.toastPos = v.toastT != null ? 'top:' + v.toastT + 'px' : 'bottom:' + v.toastB + 'px';
  v.vt = vaultVals(c);
  v.busy = busyVals(c); v.swallow = (e) => { e.stopPropagation(); e.preventDefault(); };
  const CF = s.cfm; v.cfm = CF ? { show: true, title: CF.title, text: CF.text, ok: CF.ok || '', hasOk: !!CF.run, run: CF.run || (() => {}), okBg: CF.danger ? 'var(--block)' : 'var(--acc)', okCol: CF.danger ? '#16130c' : 'var(--accInk)', opts: CF.opts || [], hasOpts: !!(CF.opts && CF.opts.length), cancel: () => c.setState({ cfm: null }) } : { show: false, opts: [] };
  Object.assign(v, {
    hVaultClose: () => c._vaultClose(), hVaultQ: (e) => c._vtSet({ q: e.target.value }), hNoteNew: () => c._noteCreate('Untitled', 'note', null),
    hNotePanel: () => { const V = c.state.vt; if (c._mode() !== 'tablet') c._vtSet({ sheet: V.sheet ? null : 'links' }); else c._vtSet({ panel: V.panel === 'none' ? 'links' : 'none' }); },
    hNeInput: (e) => c._neInput(e), hNeSel: (e) => { c._neAc(e.target); c._caretLink(e.target, 'vault'); }, hNeClick: (e) => { c._neAc(e.target); c._writeClick(e, 'vault'); }, hNeKey: (e) => c._neKey(e), hNeDone: () => c._noteMode('read'),
    hNoteSheetClose: () => c._vtSet({ sheet: null }), hNoteDlgClose: () => c.setState({ neDlg: null }), hPeekClose: () => c.setState({ peek: null }),
    hNoteRename: () => { const n = vtNote(c); if (!n) return; const k = n.name.toLowerCase(); const per = notesOf(c).filter((x) => x.id !== n.id).map((x) => (x.text.match(/\[\[([^\]|#]+)/g) || []).filter((m) => m.slice(2).trim().toLowerCase() === k).length).filter((q) => q > 0); const cnt = per.reduce((a, q) => a + q, 0); c.setState({ neDlg: { kind: 'rename', value: n.name, count: cnt, notes: per.length } }); },
    hNoteRenameInput: (e) => c.setState({ neDlg: Object.assign({}, c.state.neDlg, { value: e.target.value }) }),
    hNoteRenameDo: () => { const n = vtNote(c), D = c.state.neDlg; if (!n || !D || !D.value.trim()) return; const to = D.value.trim(), from = n.name; const re = new RegExp('\\[\\[' + from.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + '(?=[\\]|#])', 'gi'); c._neFlush(); const notes = notesOf(c).map((x) => (x.id === n.id ? Object.assign({}, x, { name: to, text: x.text }) : Object.assign({}, x, { text: x.text.replace(re, '[[' + to) }))); c.setState({ notes, neDlg: null }); c._toast('Renamed to ' + to + (D.count ? ' · ' + D.count + ' link' + (D.count === 1 ? '' : 's') + ' in ' + D.notes + ' note' + (D.notes === 1 ? '' : 's') + ' updated' : '')); },
    hNoteConflict: () => { const V = c.state.vt; const n = vtNote(c); if (!n) return; const notes = notesOf(c).map((x) => (x.id === n.id ? Object.assign({}, x, { text: x.text + '\n\nAdded in Obsidian on another device.\n' }) : x)); c.setState({ notes, vt: Object.assign({}, V, { conflict: true, buf: V.buf != null ? V.buf : n.text, sheet: null }) }); },
    hConflictReload: () => c.setState({ vt: Object.assign({}, c.state.vt, { conflict: false, buf: null, dirty: false }) }),
    hConflictKeep: () => { const V = c.state.vt; c.state.vt = Object.assign({}, V, { conflict: false, dirty: true }); c._neFlush(); c._toast('Kept your version; the outside edit was overwritten'); }
  });
  return v;
}
