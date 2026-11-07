/* ======================================================================================================
   Rows: every screen is a list of plain row objects drawn by Rows.dc.html. The same builders feed the
   phone sheet, the phone-landscape side panel, the tablet docks, menus and windows, so a control has one
   definition whatever the form factor. That is the point of the mockup for the refactor: layout is the
   frame's job, content is shared.
   ====================================================================================================== */
const hd = (label, right, pad) => ({ k: 'hd', label, right, pad });
const info = (label, warn) => ({ k: 'info', label, warn });
const read = (label, v, vCol) => ({ k: 'read', label, v: v == null ? '—' : String(v), vCol });
const sep = () => ({ k: 'sep' });
function getK(c, key) {
  const s = c.state; if (!key) return undefined;
  if (key.startsWith('prefs.')) return s.prefs[key.slice(6)];
  if (key === 'dim3') return s.dim3; if (key === 'dom') return s.dom; if (key === 'worldnode') return s.node.world;
  if (key === 'win.left') return c._mode() === 'tablet' ? s.ldockOpen : s.tab === 'dom';
  if (key === 'win.right') return c._mode() === 'tablet' ? s.rdockOpen : (s.tab === 'sel' || s.tab === 'ctx');
  if (key === 'win.timeline') return s.tlOpen; if (key === 'win.status' || key === 'win.rail') return true;
  if (key.startsWith('ctxmod.')) return ctxModsOf(c).includes(key.slice(7));
  return undefined;
}
function setK(c, key, v) {
  const s = c.state;
  if (key.startsWith('ctxmod.')) { c._ctxModSet(key.slice(7), !!v); return; }
  if (key.startsWith('prefs.')) {
    const k = key.slice(6); const prefs = Object.assign({}, s.prefs, { [k]: v }); const patch = { prefs };
    if (k === 'theme') patch.light = v === 'light'; if (k === 'units') patch.units = v;
    if (k === 'session' && v) { patch.dom = s.dom === 'world' ? 'civil' : s.dom; c._toast('Session mode · WORLD hidden, generation rows disabled'); }
    c.setState(patch); c._dirty = true; return;
  }
  if (key === 'dim3') { c.setState({ dim3: v }); c._toast(v ? '3D view · tier Basic (mock: the stand-in map stays 2D)' : '2D view'); return; }
  if (key === 'dom') { c._setDom(v); return; }
  if (key === 'worldnode') { c.setState({ dom: 'world', node: Object.assign({}, s.node, { world: v }) }); return; }
  if (key === 'win.left') { if (c._mode() === 'tablet') c.setState({ ldockOpen: v }); else c.setState({ tab: v ? 'dom' : null }); return; }
  if (key === 'win.right') { if (c._mode() === 'tablet') c.setState({ rdockOpen: v }); else c._openInspector(); return; }
  if (key === 'win.timeline') { c.setState({ tlOpen: v }); return; }
  c._toast('Region toggled (always on in this mockup)');
}
const hasSel = (c) => !!(c.state.sel && c.state.sel.kind === 'place');
function smGate(c, it) { return it.sm && c.state.prefs.session ? 'Session mode is on (Window ▸ Session mode)' : ''; }
function liveReason(c, it) {
  const s = c.state;
  if (smGate(c, it)) return smGate(c, it);
  if (it.needSel && !(c._ptr && c._ptr() ? !!subjectOf(c) : hasSel(c))) return c._ptr && c._ptr() ? 'Select something first: the map, Find (Ctrl+F) or any link' : 'Select a settlement on the map first';
  if (it.act === 'fromSelection' && !s.region) return 'Draw a region first: Region select (R) on the map';
  if (it.act === 'revert' && !s.savedAt) return 'Not saved yet';
  if (it.dyn === 'undo' && !s.undo.length) return 'Nothing to undo';
  if (it.dyn === 'redo' && !s.redo.length) return 'Nothing to redo';
  if (it.act === 'run' && s.running) return 'A generation is running';
  if (it.dyn === 'gpuRetry' && !s.gpuFailed) return 'Nothing to clear: no GPU readback has failed this session';
  return '';
}
function liveLabel(c, it) {
  const s = c.state;
  if (it.dyn === 'undo' && s.undo.length) return 'Undo ' + s.undo[s.undo.length - 1];
  if (it.dyn === 'redo' && s.redo.length) return 'Redo ' + s.redo[s.redo.length - 1];
  if (it.act === 'showOnDisk' && c._mode() !== 'tablet' && it.phone) return it.phone;
  return it.label;
}
function readoutV(c, it) { if (it.dyn === 'campaigns') return 'Campaigns in year ' + c.state.year + ': ' + conflictsOf(c).filter((k) => c.state.year >= k.from && c.state.year <= k.to).length; return it.v; }
/* Window ▸ Open windows, listed from what is open now (finding W5: the shipped list omits the pop-outs) */
function openWindowItems(c) {
  const s = c.state; const out = [];
  if (s.win) s.win.stack.slice().reverse().forEach((id, i) => out.push({ t: 'live', label: (i ? '↳ ' : '') + (winRows(c, id).title || id).toLowerCase().replace(/^./, (m) => m.toUpperCase()), act: 'toast', arg: 'Brought to the front' }));
  if (s.vt && s.vt.open) out.push({ t: 'live', label: 'Markdown vault', act: 'win:vault' });
  if (!out.length) out.push({ t: 'readout', label: 'No windows open', v: '' });
  out.push({ t: 'check', label: 'Context panel', key: 'win.right' });   // after the list, as drafts/pc-shell/05 builds it
  return out;
}
/* desktop menu rows → touch rows. A submenu of only radio rows is drawn inline as chips (as the phone does today) */
function menuRows(c, items, drill, tablet) {
  const rows = [];
  for (const it of items) {
    if (!it) continue;
    if (it.mobile && tablet === 'desktop') continue;
    if (it.desktopOnly) { if (!tablet) continue; rows.push({ k: 'todo', label: it.label, tag: 'DESKTOP ONLY', reason: 'Needs a second native window; phones and tablets have one window.' }); continue; }
    if (it.notPhone && !tablet) continue;   // a key-binding row: no accelerator fires on the phone (PC_SHELL_PROPOSAL.md §3.9)
    if (it.t === 'band') rows.push(hd(it.label));
    else if (it.t === 'sep') rows.push(sep());
    else if (it.t === 'sign') rows.push(info(it.label));
    else if (it.t === 'readout') rows.push(read(it.label, readoutV(c, it)));
    else if (it.t === 'todo') rows.push({ k: 'todo', label: it.label, reason: it.reason });
    else if (it.t === 'check') { const on = it.on != null && getK(c, it.key) == null ? it.on : !!getK(c, it.key); rows.push({ k: 'tog', label: it.label, sub: (it.note ? it.note + ' · ' : '') + (on ? 'on' : 'off') + (tablet && it.accel ? ' · ' + it.accel : ''), on, tap: () => setK(c, it.key, !on) }); }
    else if (it.t === 'radio') rows.push({ k: 'chk', radio: true, label: it.label, on: getK(c, it.key) === it.value, tap: () => setK(c, it.key, it.value) });
    else if (it.t === 'sub') {
      const kids = it.items.filter((x) => x && x.t !== 'sep');
      const radios = kids.filter((x) => x.t === 'radio');
      if (radios.length >= 2 && radios.length <= 9 && kids.every((x) => x.t === 'radio' || x.t === 'readout' || x.t === 'todo')) {
        rows.push({ k: 'seg', label: it.label, opts: radios.map((x) => ({ label: x.label, on: getK(c, x.key) === x.value, tap: () => setK(c, x.key, x.value) })), sub: kids.filter((x) => x.t !== 'radio').map((x) => x.t === 'todo' ? x.label + ': ' + x.reason : x.label + ' ' + readoutV(c, x)).join(' · ') });
      } else {
        const names = kids.filter((x) => x.label && x.t !== 'band' && x.t !== 'sign').slice(0, 3).map((x) => x.label.replace(/[⧉▦]/g, '').trim()).join(' · ');
        const reason = smGate(c, it);
        rows.push({ k: 'nav', d: IC.menu, label: it.label, sub: reason || names, badge: String(kids.length), badgeCol: 'var(--dim)', dis: !!reason, reason, tap: reason ? () => c._toast(reason) : () => drill(it) });
      }
    } else if (it.t === 'live') {
      const reason = liveReason(c, it);
      rows.push({ k: 'nav', d: it.danger ? IC.trash : (it.act && it.act.startsWith('win:') ? IC.window : (it.d || IC.inspect)), label: liveLabel(c, it), sub: reason ? '' : (it.danger ? 'destructive · asks first' : ''), accel: tablet ? it.accel : '', chev: !!(it.act && it.act.startsWith('win:')), dis: !!reason, reason, tap: reason ? () => c._toast(reason) : () => doAct(c, it) });
    }
  }
  return rows;
}
function doAct(c, it) {
  const s = c.state, a = it.act, arg = it.arg; const m = c._mode();
  if (!a) return;
  if (a.startsWith('win:')) { c._openWin(a.slice(4) + (arg ? ':' + arg : '')); return; }
  const closeMenus = () => c.setState({ dd: null, hamOpen: false });
  switch (a) {
    case 'newWorld': c._openWin('newworld'); break;
    case 'picker': closeMenus(); c.setState({ picker: true, tab: null }); break;
    case 'openWorld': closeMenus(); c.setState({ worldName: arg, seed: { 'VHAREN REACH': 129384, KESSA: 774201, THORNWOOD: 908133, 'HIGH SADDLE': 441907 }[arg] || 483920, picker: false, sel: null, sample: null, tab: null }, () => c._makeWorld(c.state.seed, true)); c._toast(arg + ' loaded'); break;
    case 'fromSelection': c._toast('New world from the region · resampled at 4× (mock)'); break;
    case 'save': c.setState({ savedAt: hm() }); c._toast('Project saved · ' + hm()); break;
    case 'saveAs': c._toast('Save as… · system file picker; the new path becomes the project path'); break;
    case 'revert': c._toast('Reverted to the save of ' + s.savedAt); break;
    case 'close': closeMenus(); c.setState({ picker: true, tab: null, sel: null, sample: null }); break;
    case 'showOnDisk': c._toast(m === 'tablet' ? 'Opens the project folder in the file manager' : 'Share project… · the Android share sheet'); break;
    case 'undo': c._undo(); break; case 'redo': c._redo(); break;
    case 'history': closeMenus(); c.setState({ hist: true }); break;
    case 'clip': c._toast(arg + ' · ' + (s.sel ? 'selection' : 'nothing selected')); break;
    case 'delete': if (s.sel && s.sel.kind === 'place') { const pl = s.userPlaces.find((p) => p.id === s.sel.id); if (pl) { c.setState({ userPlaces: s.userPlaces.filter((p) => p !== pl), sel: null, tab: null }); c._push('delete · ' + pl.name, () => c.setState((st) => ({ userPlaces: st.userPlaces.concat([pl]) })), () => c.setState((st) => ({ userPlaces: st.userPlaces.filter((p) => p !== pl) }))); break; } } c._toast(s.sel ? 'Generated places come back with Generate; delete hides it (mock)' : 'Nothing selected'); break;
    case 'selectAll': c._toast(s.dom === 'civil' ? 'Select all is off in CIVIL' : (s.dom === 'world' ? 'Selected ' + s.stamps.length + ' draft stamps' : 'Selected ' + (s.userIcons.length + s.userLabels.length) + ' icons and labels')); break;
    case 'deselect': closeMenus(); if (c._deselect) c._deselect(); else c.setState({ sel: null }); break;
    case 'selBack': closeMenus(); c._selBack(); break;
    case 'ctxCard': { closeMenus(); const { W: w0, H: h0 } = c._hostWH(); const sj = entOf(c, subjectOf(c)); const p = sj && sj.x != null ? c._w2s(sj.x, sj.y) : { x: w0 / 2, y: h0 / 2 }; c._longPress(p.x, p.y, true); break; }
    case 'ringHere': { closeMenus(); const p = c._lastPtr || (() => { const { W: w0, H: h0 } = c._hostWH(); return { x: w0 / 2, y: h0 / 2 }; })(); c._longPress(p.x, p.y, true, true); break; }
    case 'selFwd': closeMenus(); c._selFwd(); break;
    case 'ctx': closeMenus(); c._ctxTab(arg || 'inspect', arg === 'ask'); break;
    case 'ctxPin': closeMenus(); { const sj = subjectOf(c); if (sj) c.setState({ ctxPin: Object.assign({}, sj), rdockOpen: true }); else c._toast('Nothing selected to pin to'); } break;
    case 'ctxFollow': closeMenus(); c.setState({ ctxPin: null, rdockOpen: true }); break;
    case 'ctxReset': closeMenus(); c._ctxReset(); break;
    case 'factionsHere': { closeMenus(); const e = entOf(c, subjectOf(c)); let fi = -1; if (e) fi = e.kind === 'faction' ? e.i : (e.kind === 'place' ? e.o.fac : (e.kind === 'conflict' ? e.o.sides[0] : -1)); if (c._ptr()) { c._openWin('factions' + (arg === 'history' ? ':history' : '')); break; } c._openWin(fi >= 0 ? 'factions:f' + fi : 'factions' + (arg ? ':' + arg : '')); break; }
    case 'search': closeMenus(); c.setState({ srch: true, q: '' }); break;
    case 'resetParams': c.setState({ vals: {} }); c._markStale(1); c._toast('Every generation parameter is back to its default · applies on the next Generate'); break;
    case 'resetStage': { const vals = Object.assign({}, s.vals); PARAMS.filter((p) => p.stage === arg).forEach((p) => { delete vals[p.key]; }); c.setState({ vals }); c._markStale(arg); c._toast('Stage ' + String(arg).padStart(2, '0') + ' reset'); break; }
    case 'filePick': c._toast('System file picker · ' + arg); break;
    case 'confirm': c._openWin('confirm:' + arg); break;
    case 'toast': c._toast(arg); break;
    case 'node': { const [d, n] = arg.split('/'); closeMenus(); c.setState({ dom: d, node: Object.assign({}, s.node, { [d]: n }), win: null }); if (m === 'tablet') c.setState({ ldockOpen: true }); else c.setState({ tab: 'dom', det: 'half' }); break; }
    case 'story': closeMenus(); if (m === 'tablet') { if (arg === 'time') c.setState({ tlOpen: true }); else if (arg === 'journeys' && c._ptr()) { if (!(s.win && s.win.stack[s.win.stack.length - 1] === 'planner')) c._openWin('planner'); } else if (arg === 'journeys') c.setState({ dom: 'civil', node: Object.assign({}, s.node, { civil: 'journey' }), ldockOpen: true }); else c._openWin('story:' + arg); } else c.setState({ tab: 'story', storySeg: arg, det: 'half', planStage: -1, win: null }); break;
    case 'tool': closeMenus(); c.setState({ dom: 'civil', win: null }); setTimeout(() => c._arm(arg), 0); break;
    case 'placeHistory': c._openWin('place:history'); break;
    case 'notesHere': c._openWin('vault:' + (hasSel(c) ? 'place' : 'browse')); break;
    case 'theme': setK(c, 'prefs.theme', s.light ? 'dark' : 'light'); break;
    case 'clearUndo': c._undoFns = []; c._redoFns = []; c.setState({ undo: [], redo: [] }); c._toast('Undo history cleared'); break;
    case 'resetLayout': closeMenus(); c.setState({ ldockOpen: true, rdockOpen: false, tlOpen: false, ldockPin: false, tab: null }); c._toast('Layout reset'); break;
    case 'layout': { const [d, n] = arg.split('/'); closeMenus(); if (d === 'story') { doAct(c, { act: 'story', arg: n }); break; } c.setState({ dom: d, node: Object.assign({}, s.node, { [d]: n }) }); if (m !== 'tablet') c.setState({ tab: 'dom', det: 'half' }); c._toast('Layout · ' + it.label); break; }
    case 'lmType': c.setState({ lmKind: arg }); doAct(c, { act: 'node', arg: 'civil/landmarks' }); break;
    case 'run': c._run(1); break;
    default: c._toast(it.label + ' (mock)');
  }
}

/* ---------- small control factories bound to state ---------- */
function prefTog(c, key, label, sub) { const on = !!c.state.prefs[key]; return { k: 'tog', label, sub, on, tap: () => setK(c, 'prefs.' + key, !on) }; }
function prefSeg(c, key, label, opts, sub) { return { k: 'seg', label, sub, opts: opts.map((o) => { const v = Array.isArray(o) ? o[0] : o, l = Array.isArray(o) ? o[1] : o; return { label: l, on: c.state.prefs[key] === v, tap: () => setK(c, 'prefs.' + key, v) }; }) }; }
function stSeg(c, key, label, opts, sub, after) { return { k: 'seg', label, sub, opts: opts.map((o) => { const v = Array.isArray(o) ? o[0] : o, l = Array.isArray(o) ? o[1] : o; return { label: l, on: c.state[key] === v, tap: () => { c.setState({ [key]: v }); if (after) after(v); c._dirty = true; } }; }) }; }
function fmtVal(v, p) { if (typeof v === 'boolean') return v ? 'on' : 'off'; if (v == null) return '—'; const st = p.step || 1; const txt = st >= 1 ? Math.round(v).toLocaleString('en-US') : v.toFixed(Math.min(3, Math.max(2, -Math.floor(Math.log10(st))))); return txt + (p.unit ? (p.unit === '°' || p.unit === '×' ? p.unit : ' ' + p.unit) : ''); }
function paramRow(c, p, indent) {
  const v = c._val(p.key, p.def);
  if (p.kind === 'tog') return { k: 'tog', label: p.label, sub: p.def !== v ? 'edited · stage ' + String(p.stage).padStart(2, '0') + ' goes stale' : '', on: !!v, indent, tap: () => c._setVal(p.key, !v, p.stage) };
  const clamp = (x) => Math.max(p.min, Math.min(p.max, +x.toFixed(4)));
  return { k: 'rng', label: p.label, min: p.min, max: p.max, step: p.step, val: v, disp: fmtVal(v, p), indent, onIn: (e) => c._setVal(p.key, p.int ? Math.round(+e.target.value) : +e.target.value, p.stage), dec: () => c._setVal(p.key, clamp(v - p.step), p.stage), inc: () => c._setVal(p.key, clamp(v + p.step), p.stage) };
}
function accRow(c, id, label, num, state, stale) { const open = !!c.state.open[id]; return { k: 'acc', label, num, state, stale, open, tap: () => c.setState({ open: Object.assign({}, c.state.open, { [id]: !open }) }) }; }
function stageRows(c, n) {
  const ps = PARAMS.filter((p) => p.stage === n && !p.nodock);
  const s = c.state, stale = s.stale && n >= s.stale;
  const rows = [accRow(c, 'st-' + String(n).padStart(2, '0'), STAGE_NAMES[n - 1], String(n).padStart(2, '0'), s.running && s.running.i + 1 === n ? 'running' : (stale ? 'stale' : 'resolved'), stale)];
  if (!c.state.open['st-' + String(n).padStart(2, '0')]) return rows;
  if (!ps.length) { rows.push(info('No parameters: this stage runs with the engine defaults and is recomputed by Generate.')); return rows; }
  let lastSub = null; const showAdv = !!c.state.open['adv-' + n];
  for (const p of ps) {
    if (p.adv && !showAdv) continue;
    const sub = p.adv ? 'ADVANCED' : (p.sub && !/^\d\d /.test(p.sub) ? p.sub : '');
    if (sub && sub !== lastSub) { rows.push(hd(sub, '', '10px 4px 2px 22px')); lastSub = sub; }
    rows.push(paramRow(c, p, 1));
  }
  const nAdv = ps.filter((p) => p.adv).length;
  if (nAdv) rows.push({ k: 'nav', d: IC.gear, label: showAdv ? 'Hide advanced' : 'Show advanced (' + nAdv + ')', sub: 'expert controls for this stage', chev: false, indent: 1, tap: () => c.setState({ open: Object.assign({}, c.state.open, { ['adv-' + n]: !showAdv }) }) });
  rows.push({ k: 'acts', opts: [{ label: 'Reset stage', tap: () => doAct(c, { act: 'resetStage', arg: n }) }, { label: 'Run from ' + String(n).padStart(2, '0'), variant: 'primary', tap: () => c._run(n) }] });
  return rows;
}
