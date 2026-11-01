/* ======================================================================================================
   The context every panel shares (PC_SHELL_PROPOSAL.md §3).
   One selection, one year and one note focus. The Context panel (Inspect · Note · Ask), the window host, the
   map, the timeline and the assistant read these three and nothing else, so they cannot drift apart the way
   the shipped desktop's two selection holders, its year writers that skip `timeline_changed` and its
   assistant's snapshot-at-open do (audit of 2026-10-31: findings S1, T1, A2).
   Selection kinds: place · landmark · faction · culture · conflict · note. A selection is {kind, id}.
   ====================================================================================================== */
const SEL_KINDS = { place: ['Settlement', IC.settle], landmark: ['Landmark', IC.landmark], faction: ['Faction', IC.faction], culture: ['Culture & faith', IC.faction], conflict: ['Conflict', IC.conflict], note: ['Note', IC.notes] };
const YEAR_MIN = -400, YEAR_MAX = 1200;   // the stand-in world's recorded range; the proposal takes it from the world (finding T3)
function selKey(x) { return x ? x.kind + ':' + x.id : ''; }
function subjectOf(c) { const s = c.state; return s.ctxPin || s.sel || null; }
function facIdxOf(id) { return typeof id === 'string' && id[0] === 'f' ? +id.slice(1) : -1; }
function cultureList() { const out = []; FACTIONS.forEach((f) => { if (!out.includes(f.rel)) out.push(f.rel); }); return out; }
function cultureSel(name) { return { kind: 'culture', id: 'c' + cultureList().indexOf(name) }; }

/* Conflicts as world records: years, sides, the places they touched. Stand-ins shaped like the engine's. */
function conflictsOf(c) {
  if (c._conflicts && c._conflictsFor === c.places) return c._conflicts;
  const P = c.places || []; const of = (f) => P.filter((p) => p.fac === f); const cap = (f) => P.find((p) => p.fac === f && p.capital) || of(f)[0];
  const ids = (a) => a.filter(Boolean).map((p) => p.id);
  c._conflicts = [
    { id: 'k0', name: 'War of the Split', from: 206, to: 214, sides: [2, 0], col: '#8fae7d', places: ids([cap(2), of(2).find((p) => !p.capital)]), sub: 'Korrath breaks away from Orentamwash' },
    { id: 'k1', name: 'Sevjuni campaign', from: 398, to: 404, sides: [1, 0], col: '#6a9bc4', places: ids([of(1).find((p) => !p.capital), cap(0)]), sub: 'front · 2 sieges · manpower 41 k vs 18 k' },
    { id: 'k2', name: 'Korrath raids', from: 401, to: 409, sides: [2, 3], col: '#c96a5a', places: ids([of(3)[0], of(3)[1]]), sub: '3 raids · the Kess Ford toll house burned in 401' }
  ];
  c._conflictsFor = c.places; return c._conflicts;
}
const RELATIONS = [[null, 'trade, wary', 'hostile since the split', 'shared faith'], ['trade, wary', null, 'neutral', 'rivals at sea'], ['hostile since the split', 'neutral', null, 'raiding'], ['shared faith', 'rivals at sea', 'raided', null]];

/* Who held a place in a given year. Deterministic stand-in periods shaped like the engine's ownership
   snapshots: Korrath's places were Orentamwash's until the split of 210; a third of the League's were taken
   in the Sevjuni campaign. `null` means not founded yet (never a faction index, so it cannot be misread). */
function ownerPeriods(c, p) {
  if (!p || p.fac == null || p.fac < 0) return [];
  const f0 = p.founded != null ? p.founded : -300;
  if (p.user) return [{ from: c.state.year, to: null, fac: p.fac }];
  if (p.fac === 2 && f0 < 210) return [{ from: f0, to: 210, fac: 0 }, { from: 210, to: null, fac: 2, why: 'k0' }];
  if (p.fac === 1 && !p.capital && p.k % 3 === 0 && f0 < 404) return [{ from: f0, to: 404, fac: 0 }, { from: 404, to: null, fac: 1, why: 'k1' }];
  return [{ from: f0, to: null, fac: p.fac }];
}
function ownerAt(c, p, y) {
  if (!p) return -1; if (p.founded != null && y < p.founded && !p.user) return null;
  const per = ownerPeriods(c, p).find((q) => y >= q.from && (q.to == null || y < q.to)); return per ? per.fac : p.fac;
}

/* A selection resolved to what every panel needs: name, kind, map position, the note it would own. */
function entOf(c, x) {
  if (!x) return null; const s = c.state;
  if (x.kind === 'place') { const o = c.places.concat(s.userPlaces).find((p) => p.id === x.id); if (!o) return null; const fac = o.fac >= 0 ? FACTIONS[o.fac] : null; return { kind: 'place', id: o.id, name: o.name, o, x: o.x, y: o.y, sub: (o.tier || 'settlement') + (o.capital ? ' · capital' : '') + (fac ? ' · ' + fac.name : ''), noteName: o.name, entityKind: 'settlement' }; }
  if (x.kind === 'landmark') { const o = c.landmarks.find((l) => l.id === x.id); if (!o) return null; return { kind: 'landmark', id: o.id, name: o.name, o, x: o.x, y: o.y, sub: o.kind + ' · ' + o.fam, noteName: o.name, entityKind: 'landmark' }; }
  if (x.kind === 'faction') { const i = facIdxOf(x.id); const f = FACTIONS[i]; if (!f) return null; const cap = c.places.find((p) => p.fac === i && p.capital); const r = (c.regions || [])[i]; return { kind: 'faction', id: x.id, name: f.name, o: f, i, x: r ? r.x : (cap ? cap.x : null), y: r ? r.y : (cap ? cap.y : null), sub: f.gov + ' · ' + f.rel, noteName: f.name, entityKind: 'faction' }; }
  if (x.kind === 'culture') { const name = cultureList()[+String(x.id).slice(1)]; if (!name) return null; return { kind: 'culture', id: x.id, name, sub: 'faith of ' + FACTIONS.filter((f) => f.rel === name).map((f) => f.name).join(', '), noteName: name, entityKind: 'culture' }; }
  if (x.kind === 'conflict') { const k = conflictsOf(c).find((q) => q.id === x.id); if (!k) return null; const ps = k.places.map((id) => c.places.find((p) => p.id === id)).filter(Boolean); return { kind: 'conflict', id: k.id, name: k.name, o: k, ps, x: ps.length ? ps.reduce((a, p) => a + p.x, 0) / ps.length : null, y: ps.length ? ps.reduce((a, p) => a + p.y, 0) / ps.length : null, sub: 'years ' + k.from + '–' + k.to + ' · ' + k.sides.map((i) => FACTIONS[i].name).join(' vs '), noteName: k.name, entityKind: 'note' }; }
  if (x.kind === 'note') { const n = noteById(c, x.id); if (!n) return null; return { kind: 'note', id: n.id, name: n.name, o: n, sub: (NOTE_KINDS[n.kind] ? NOTE_KINDS[n.kind][0] : 'Note') + ' · ' + wordCount(n.text) + ' words', noteName: n.name, entityKind: n.kind }; }
  return null;
}
function selFromEntity(e) { if (!e) return null; if (e.kind === 'settlement') return { kind: 'place', id: e.id }; if (['landmark', 'faction', 'culture'].includes(e.kind)) return { kind: e.kind, id: e.id }; return null; }
/* A link target to the thing it names: the map entity first (so the map, the panel and the windows follow),
   then a conflict of that name, then the note itself. `null` when nothing of that name exists yet. */
function selFromName(c, name) {
  const r = resolveLink(c, name); const k = String(name || '').trim().toLowerCase();
  const e = r.kind === 'note' ? (r.note.entity || entityByName(c, r.note.name)) : (r.kind === 'entity' ? r.entity : null);
  const s1 = selFromEntity(e); if (s1) return s1;
  const kk = conflictsOf(c).find((q) => q.name.toLowerCase() === k); if (kk) return { kind: 'conflict', id: kk.id };
  if (r.kind === 'note') return { kind: 'note', id: r.note.id };
  return null;
}
function noteOfSubject(c, e) { if (!e) return null; if (e.kind === 'note') return noteById(c, e.id); return noteByName(c, e.noteName); }

/* ---------------- link rows: a readout whose value is one or more links ---------------- */
const LK = (label, links, tail) => ({ k: 'lnk', label, links, tail });
function selLink(c, label, sel, o) { o = o || {}; return { label, dot: o.dot || '', title: 'Select ' + label + (o.frame === false ? '' : ' and show it on the map'), tap: () => c._select(sel, { frame: o.frame !== false }) }; }
function yearLink(c, y, label) { return { label: label || 'year ' + y, title: 'Move the timeline to year ' + y, tap: () => c._setYear(y) }; }
function ctxActs(c, e, first) {
  const n = noteOfSubject(c, e);
  const opts = (first || []).concat([{ label: n ? 'Note' : 'New note', d: IC.notes, tap: () => c._ctxTab('note') }, { label: 'Ask', d: IC.assistant, tap: () => c._ctxTab('ask') }]);
  if (e.x != null) opts.push({ label: 'Centre', d: IC.pin, tap: () => c._frameSel(e) });
  return { k: 'acts', opts: opts.map((o) => Object.assign({ flex: '1 1 22%' }, o)) };
}

/* ---------------- Inspect: one builder per kind, every name a link ---------------- */
function ctxInspectRows(c, e) {
  if (!e) return inspectNone(c);
  if (e.kind === 'place') return c.state.ctxEdit ? editPlaceRows(c, e) : inspectPlace(c, e);
  if (e.kind === 'faction') return inspectFaction(c, e);
  if (e.kind === 'conflict') return inspectConflict(c, e);
  if (e.kind === 'culture') return inspectCulture(c, e);
  if (e.kind === 'note') return inspectNote(c, e);
  const o = e.o; const near = nearestPlace(c, o);
  return [ctxActs(c, e), H('LANDMARK'), read('Kind', o.kind), read('Family', o.fam), read('Elevation', o.z ? o.z + ' m' : '—'), near ? LK('Nearest place', [selLink(c, near.name, { kind: 'place', id: near.id })], fmtDist(Math.hypot(near.x - o.x, near.y - o.y), c.state.prefs.units)) : read('Nearest place', '—'), read('Position', Math.round(o.x) + ' km E · ' + Math.round(o.y) + ' km S'), H('WHY HERE?'), info(o.kind === 'Peak' ? 'Highest ground within 32 km, prominence over 600 m and visible from three settlements.' : 'Scored on the site rules for ' + o.kind.toLowerCase() + ', then spaced by class.')];
}
function nearestPlace(c, o) { let best = null, bd = 1e9; for (const p of c.places) { if (p === o) continue; const d = Math.hypot(p.x - o.x, p.y - o.y); if (d < bd) { bd = d; best = p; } } return best; }
function inspectNone(c) {
  const s = c.state; const rows = [];
  if (s.sample) rows.push(...rowsSample(c));
  else rows.push(info('Click a settlement or landmark on the map, a link in a note, a row in any window, a citation in an answer, or a result in Find (Ctrl+F). Inspect, Note and Ask follow it, and so do the windows set to follow.'));
  const recent = (s.selBack || []).slice(-5).reverse().map((x) => entOf(c, x)).filter(Boolean);
  if (recent.length) rows.push(H('RECENT', 'Alt+← goes back'), ...recent.map((e) => ({ k: 'item', label: e.name, sub: SEL_KINDS[e.kind][0] + ' · ' + e.sub, dot: 'var(--sec)', tap: () => c._select({ kind: e.kind, id: e.id }, { frame: true }) })));
  rows.push(H('FACTIONS'), ...FACTIONS.map((f, i) => ({ k: 'item', label: f.name, sub: f.gov + ' · ' + f.rel, dot: f.col, square: true, tap: () => c._select({ kind: 'faction', id: 'f' + i }, { frame: true }) })));
  rows.push(H('CONFLICTS'), ...conflictsOf(c).map((k) => ({ k: 'item', label: k.name, sub: 'years ' + k.from + '–' + k.to + ' · ' + k.sub, dot: k.col, tap: () => c._select({ kind: 'conflict', id: k.id }, { frame: true, year: k.from }) })));
  return rows;
}
function inspectPlace(c, e) {
  const s = c.state, o = e.o, y = s.year; const fac = o.fac >= 0 ? FACTIONS[o.fac] : null;
  const own = ownerAt(c, o, y); const per = ownerPeriods(c, o);
  const near = c.places.filter((p) => p !== o).map((p) => [p, Math.hypot(p.x - o.x, p.y - o.y)]).sort((a, b) => a[1] - b[1]).slice(0, 3);
  const rows = [ctxActs(c, e, [{ label: 'Edit', variant: 'primary', d: IC.edit, tap: () => c.setState({ ctxEdit: true, ctxDraft: null, ctxTab: 'inspect' }) }])];
  rows.push(H('AT YEAR ' + y, own === null ? 'not founded yet' : (own !== o.fac ? 'held by another faction then' : 'as today')));
  if (own === null) rows.push(info(o.name + ' is founded in year ' + o.founded + '. The map draws it as an outline until then.'), LK('Founded', [yearLink(c, o.founded, 'go to ' + o.founded)]));
  else if (own >= 0) rows.push(LK('Held by', [selLink(c, FACTIONS[own].name, { kind: 'faction', id: 'f' + own }, { dot: FACTIONS[own].col })]));
  rows.push(H('SETTLEMENT'), read('Class', o.tier + (o.capital ? ' · capital' : '')), read('Population', o.pop ? o.pop.toLocaleString('en-US') : 'new · grows on recompute'), LK('Founded', [yearLink(c, o.founded)]));
  if (fac) rows.push(LK('Faction today', [selLink(c, fac.name, { kind: 'faction', id: 'f' + o.fac }, { dot: fac.col })]), read('Government', fac.gov), LK('Faith', [selLink(c, fac.rel, cultureSel(fac.rel), { frame: false })]));
  rows.push(read('Elevation', (o.elev || 0) + ' m'), read('Biome', o.biome || '—'));
  rows.push(H('POLITICAL HISTORY', per.length + (per.length === 1 ? ' period' : ' periods')));
  per.forEach((q) => { const on = own !== null && y >= q.from && (q.to == null || y < q.to); rows.push(LK(q.from + ' – ' + (q.to == null ? 'now' : q.to), [selLink(c, FACTIONS[q.fac].name, { kind: 'faction', id: 'f' + q.fac }, { dot: FACTIONS[q.fac].col }), yearLink(c, q.from, '⟲ ' + q.from)], on ? '◂ year ' + y : '')); });
  const wars = conflictsOf(c).filter((k) => k.places.includes(o.id));
  if (wars.length) rows.push(LK('Conflicts here', wars.map((k) => selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col }))));
  rows.push(H('NEIGHBOURS · ROADS'), LK('Nearest', near.map(([p, d]) => selLink(c, p.name, { kind: 'place', id: p.id })), near[0] ? fmtDist(near[0][1], s.prefs.units) : ''), read('Roads', c.ways.filter((w) => w.a === o || w.b === o).length + ' ways'), read('Garrison', o.capital ? '1 200' : '80'));
  rows.push(H('MORE'), J(c, 'City viewer…', 'win:city', 'its own window, follows the selection'), { k: 'nav', d: IC.journey, label: 'Plan a journey from here', sub: (c._ptr() ? 'the planner opens in its own window, beside the live map, with ' : 'the planner opens beside the map with ') + o.name + ' as the start', tap: () => c._planFrom(o) });
  if (o.user) rows.push(B('Delete ' + o.name, () => doAct(c, { act: 'delete' }), 'danger'));
  return rows;
}
function editPlaceRows(c, e) {
  const s = c.state, o = e.o; const d = s.ctxDraft || { name: o.name, tier: o.tier, fac: o.fac, fort: !!o.fort };
  const set = (k, v) => c.setState({ ctxDraft: Object.assign({}, d, { [k]: v }) });
  return [info('Editing ' + o.name + ' here: the Place editor is this panel’s edit mode, so the map, the note and the assistant stay on the same settlement. The map stays live.'),
    { k: 'field', label: 'Name', val: d.name, onIn: (ev) => set('name', ev.target.value) },
    { k: 'seg', label: 'Class', opts: ['city', 'town', 'village', 'hamlet'].map((t) => ({ label: t, on: d.tier === t, tap: () => set('tier', t) })) },
    { k: 'seg', label: 'Faction', opts: FACTIONS.map((f, i) => ({ label: f.name.split(' ')[0], on: d.fac === i, tap: () => set('fac', i) })) },
    { k: 'tog', label: 'Fortified', sub: 'walls on the city viewer, garrison × 2', on: !!d.fort, tap: () => set('fort', !d.fort) },
    BB({ label: 'Cancel', tap: () => c.setState({ ctxEdit: false, ctxDraft: null }) }, { label: 'Apply · one undo step', variant: 'primary', tap: () => c._applyPlaceEdit(o, d) }),
    H('MORE OF THE PLACE EDITOR'), J(c, 'Economy, buildings and layout…', 'win:place', 'the full editor window, following this settlement')];
}
function inspectFaction(c, e) {
  const s = c.state, f = e.o, i = e.i, y = s.year; const held = c.places.filter((p) => ownerAt(c, p, y) === i);
  const cap = c.places.find((p) => p.fac === i && p.capital); const wars = conflictsOf(c).filter((k) => k.sides.includes(i));
  const active = wars.filter((k) => y >= k.from && y <= k.to);
  const rows = [ctxActs(c, e, [{ label: 'Factions window', variant: 'primary', d: IC.faction, tap: () => c._openWin('factions') }])];
  rows.push(H('AT YEAR ' + y, active.length ? 'at war' : 'at peace'), read('Settlements held', held.length + ' of ' + c.places.length), read('Population', held.reduce((a, p) => a + (p.pop || 0), 0).toLocaleString('en-US')));
  if (active.length) rows.push(LK('At war', active.map((k) => selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col }))));
  rows.push(H('IDENTITY'), read('Government', f.gov), LK('Faith', [selLink(c, f.rel, cultureSel(f.rel), { frame: false })]), read('Colour', f.col, f.col));
  if (cap) rows.push(LK('Capital', [selLink(c, cap.name, { kind: 'place', id: cap.id })]));
  rows.push(H('RELATIONS'), ...FACTIONS.map((g, j) => (j === i ? null : LK(g.name.split(' ')[0], [selLink(c, g.name, { kind: 'faction', id: 'f' + j }, { dot: g.col })], RELATIONS[i][j]))).filter(Boolean));
  if (wars.length) rows.push(H('CONFLICTS'), ...wars.map((k) => LK(k.from + '–' + k.to, [selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col })], y >= k.from && y <= k.to ? '◂ now' : '')));
  rows.push(H('SETTLEMENTS', held.length + ' in year ' + y), ...held.slice(0, 10).map((p) => ({ k: 'item', label: p.name, sub: p.tier + (p.capital ? ' · capital' : '') + (p.fac !== i ? ' · lost later' : ''), right: p.pop ? p.pop.toLocaleString('en-US') : '', dot: f.col, on: false, tap: () => c._select({ kind: 'place', id: p.id }, { frame: true }) })));
  if (held.length > 10) rows.push(J(c, 'All ' + held.length + ' in the Factions window…', 'win:factions'));
  return rows;
}
function inspectCulture(c, e) {
  const fs = FACTIONS.map((f, i) => [f, i]).filter(([f]) => f.rel === e.name);
  const ps = c.places.filter((p) => p.fac >= 0 && FACTIONS[p.fac].rel === e.name);
  return [ctxActs(c, e), H('CULTURE & FAITH'), LK('Followed by', fs.map(([f, i]) => selLink(c, f.name, { kind: 'faction', id: 'f' + i }, { dot: f.col }))), read('Settlements', ps.length + ''), read('People', ps.reduce((a, p) => a + (p.pop || 0), 0).toLocaleString('en-US')), info('A culture has no single place on the map, so Centre is not offered. Its note and the assistant still follow it.')];
}
function inspectConflict(c, e) {
  const s = c.state, k = e.o, y = s.year; const st = y < k.from ? 'not started' : (y > k.to ? 'over' : 'active');
  return [ctxActs(c, e, [{ label: 'Go to ' + k.from, variant: 'primary', d: IC.time, tap: () => c._setYear(k.from) }]), H('AT YEAR ' + y, st), LK('Years', [yearLink(c, k.from), yearLink(c, k.to)]),
    LK('Sides', k.sides.map((i) => selLink(c, FACTIONS[i].name, { kind: 'faction', id: 'f' + i }, { dot: FACTIONS[i].col }))),
    LK('Places', (e.ps || []).map((p) => selLink(c, p.name, { kind: 'place', id: p.id }))), read('Summary', k.sub),
    info('The map draws the campaign while the timeline is inside its years, in every domain.'), J(c, 'All conflicts…', 'win:conflicts')];
}
function inspectNote(c, e) {
  const n = e.o; const outs = []; (n.text.match(/\[\[([^\]|#]+)/g) || []).forEach((m) => { const t = m.slice(2).trim(); if (!outs.includes(t)) outs.push(t); });
  const bl = backlinksOf(c, n); const backs = (bl.linked || []).map((h) => h.note || h).filter((x, i, a) => x && x.name && a.findIndex((q) => q.name === x.name) === i);
  const lnk = (name) => { const sl = selFromName(c, name); return sl ? selLink(c, name, sl) : { label: name, title: 'No note or place of that name yet', tap: () => c._toast('“' + name + '” has no note yet · Note ▸ create it from the link') }; };
  return [ctxActs(c, e, [{ label: 'Open in vault', variant: 'primary', d: IC.vault, tap: () => c._openVaultAt(n.id) }]), H('NOTE'), read('Kind', NOTE_KINDS[n.kind] ? NOTE_KINDS[n.kind][0] : 'Note'), read('Path', vtPath(n)), read('Words', wordCount(n.text) + ''), read('Edited', vtAgo(n.edited)),
    LK('Links to', outs.slice(0, 8).map(lnk)), LK('Linked from', backs.slice(0, 8).map((x) => lnk(x.name)), backs.length ? '' : 'none yet'), info(n.private ? 'Private note: never sent to the assistant.' : 'Shared with the assistant when Ask includes the note.')];
}

/* ---------------- the journey planner reads the selection (flow F2): select a place, then place it ---------------- */
function plannerSelRows(c) {
  const s = c.state; const e = entOf(c, s.sel); if (!e || e.kind !== 'place') return [info('Select a settlement on the map (or anywhere a name is a link) to make it the start, a stop or the end. ' + (c._ptr() ? 'The map stays live while the planner window is open.' : 'The map stays visible beside the planner.'))];
  const o = e.o; const inRoute = c.route.includes(o);
  const set = (fn, label) => () => { const before = c.route.slice(); fn(); const after = c.route.slice(); c._push('route · ' + label + ' ' + o.name, () => { c.route = before; c._dirty = true; c.forceUpdate(); }, () => { c.route = after; c._dirty = true; c.forceUpdate(); }); c._dirty = true; c.forceUpdate(); };
  return [H('SELECTION · ' + o.name.toUpperCase(), inRoute ? 'already on the route' : 'not on the route'), { k: 'acts', opts: [
    { label: 'Start here', flex: '1 1 30%', tap: set(() => { c.route = [o].concat(c.route.slice(1).filter((p) => p !== o)); }, 'start') },
    { label: 'Add as stop', flex: '1 1 30%', tap: set(() => { const r = c.route.filter((p) => p !== o); r.splice(Math.max(1, r.length - 1), 0, o); c.route = r; }, 'stop') },
    { label: 'End here', variant: 'primary', flex: '1 1 30%', tap: set(() => { c.route = c.route.slice(0, -1).filter((p) => p !== o).concat([o]); }, 'end') }] }];
}

/* ---------------- the timeline: one range, events from the world and from notes ---------------- */
function worldEvents(c) {
  // the seeded story's first place, by name: the first in the list is not it
  const vs = (c.places || []).find((p) => p.name === PLACE_NAMES[0]) || (c.places || [])[0]; const ev = [{ y: vs ? vs.founded : -120, t: 'Founding of ' + (vs ? vs.name : 'Vhal Serai'), src: 'world', sel: vs ? { kind: 'place', id: vs.id } : null }, { y: 0, t: 'Year zero', src: 'world' }];
  conflictsOf(c).forEach((k) => ev.push({ y: k.from, y1: k.to, t: k.name, src: 'conflict', col: k.col, sel: { kind: 'conflict', id: k.id } }));
  ev.push({ y: 412, t: 'Now', src: 'world' });
  return ev;
}
function noteEvents(c, n) {
  if (!n) return []; const out = []; const L = n.text.split('\n'); let inC = false; const block = [];
  L.forEach((ln) => { if (/^```chronos/.test(ln.trim())) { inC = true; return; } if (inC && /^```/.test(ln.trim())) { inC = false; return; } if (inC) block.push(ln); });
  chronosParse(block).forEach((e) => out.push({ y: e.y0, y1: e.y1, t: e.name.replace(/\[\[|\]\]/g, ''), src: 'note', note: n.name }));
  return out;
}
function tlEvents(c) {
  const e = entOf(c, subjectOf(c)); const n = e ? noteOfSubject(c, e) : null;
  return worldEvents(c).concat(noteEvents(c, n)).sort((a, b) => a.y - b.y);
}
/* The strip's expanded row: what CIVIL ▸ Timeline holds today (recorded years, Add year, the three filters) at
   program scope, so the year has one home in every domain (GUI_GAP_REGISTER.md CV-24's design question). */
function tlControls(c) {
  const s = c.state; const F = s.tlFilt || {}; const ch = (on) => (on ? { bg: 'var(--wash)', bord: 'var(--acc)', col: 'var(--acc)' } : { bg: 'var(--chip)', bord: 'var(--hair)', col: 'var(--body)' });
  const rec = (s.recYears || []).map((y) => Object.assign({ label: String(y), tap: () => c._setYear(y), aria: 'go to recorded year ' + y }, ch(s.year === y)));
  const filt = [['exist', 'Exist only', 'hide places not founded yet, instead of drawing their outlines'], ['ghost', 'Ghost removed', 'draw places lost by this year at half strength'], ['halo', 'Highlight new', 'ring places founded since the previous recorded year']].map(([k, l, why]) => Object.assign({ label: l, title: why, on: !!F[k], tap: () => c.setState({ tlFilt: Object.assign({}, F, { [k]: !F[k] }) }) }, ch(!!F[k])));
  const sp = [1, 10, 100].map((v) => Object.assign({ label: '×' + v, on: s.speed === v, tap: () => c.setState({ speed: v }) }, ch(s.speed === v)));
  const has = (s.recYears || []).includes(s.year);
  return { rec, filt, sp, addLabel: has ? 'RECORDED' : '+ RECORD ' + s.year, addTap: () => { if (has) return; const y = s.year; const before = (s.recYears || []).slice(); const after = before.concat([y]).sort((a, b) => a - b); c.setState({ recYears: after }); c._push('record year ' + y, () => c.setState({ recYears: before }), () => c.setState({ recYears: after })); c._toast('Recorded year ' + y + ': who holds what is kept as a snapshot'); }, addBg: has ? 'var(--chip)' : 'var(--acc)', addCol: has ? 'var(--dim)' : 'var(--accInk)' };
}
function tlTicks(c) {
  const span = YEAR_MAX - YEAR_MIN; const pos = (y) => Math.max(0, Math.min(100, (y - YEAR_MIN) / span * 100));
  return tlEvents(c).map((ev) => ({ left: pos(ev.y).toFixed(2), w: ev.y1 != null ? Math.max(0.6, pos(ev.y1) - pos(ev.y)).toFixed(2) : '0.35', col: ev.src === 'note' ? 'var(--acc)' : (ev.col || 'var(--sec)'), h: ev.src === 'note' ? 10 : 6, op: ev.src === 'note' ? 1 : 0.85 }));
}

/* ---------------- the assistant (Ask): scripted stand-in for the shipped LLM-0…LLM-10 panel ----------------
   No model is called. Replies are built from the stand-in world and the notes so the flow can be tried:
   context chips that follow the panel, citations that select, years that move the timeline, and action
   cards that change nothing until Apply (one undo step each, Rulings DC/DD/DS). */
function askPrompts(c, e) {
  if (!e) return ['What is on this map?', 'Which factions are at war this year?', 'Suggest a site for a new port'];
  if (e.kind === 'place') return ['Describe ' + e.name, 'Its history to year ' + c.state.year, 'Suggest links for its note', 'Draft a chronos entry'];
  if (e.kind === 'faction') return ['Describe ' + e.name, 'Who are its enemies in year ' + c.state.year + '?', 'Show its territory'];
  if (e.kind === 'conflict') return ['Summarise ' + e.name, 'Who held what when it ended?', 'Add it to a note as chronos'];
  if (e.kind === 'note') return ['Summarise this note', 'Suggest links', 'Clean up the formatting'];
  return ['Describe ' + e.name, 'Where is it mentioned?'];
}
function askCtx(c) {
  const s = c.state; const e = entOf(c, subjectOf(c)); const n = e ? noteOfSubject(c, e) : null; const off = (s.ask && s.ask.off) || {};
  return { e, n: n && !n.private ? n : null, priv: !!(n && n.private), year: s.year, off };
}
function askChips(c) {
  const k = askCtx(c); const out = [];
  if (k.e) out.push({ id: 'sel', label: k.e.name, d: SEL_KINDS[k.e.kind][1], title: SEL_KINDS[k.e.kind][0] + ' · follows the panel' });
  out.push({ id: 'year', label: 'year ' + k.year, d: IC.time, title: 'The timeline year · moves with the strip' });
  if (k.n) out.push({ id: 'note', label: k.n.name + '.md', d: IC.notes, title: 'The note of the selection, as saved' });
  else if (k.priv) out.push({ id: 'note', label: 'private note · not sent', d: IC.notes, title: 'Private notes never leave the vault', fixed: true });
  return out.map((x) => Object.assign(x, { off: !!k.off[x.id] }));
}
const T_ = (s) => ({ t: 'text', s });
const Y_ = (c, y) => ({ t: 'cite', s: 'year ' + y, title: 'Move the timeline to year ' + y, run: (cc) => cc._setYear(y) });
const E_ = (c, label, sel) => ({ t: 'cite', s: label, title: 'Select ' + label, run: (cc) => cc._select(sel, { frame: true }) });
const N_ = (c, n, head) => ({ t: 'cite', s: n.name + '.md' + (head ? ' › ' + head : ''), title: 'Open the note in the panel', run: (cc) => { cc._select({ kind: 'note', id: n.id }, { tab: 'note' }); } });
function askReply(c, q, k) {
  const s = c.state, ql = q.toLowerCase(); const e = k.off.sel ? null : k.e; const y = k.off.year ? 412 : k.year; const n = k.off.note ? null : k.n;
  const tools = []; const parts = []; const acts = []; const srcs = [];
  const lf = (i) => E_(c, FACTIONS[i].name, { kind: 'faction', id: 'f' + i });
  // commands first: the model proposes, the person applies
  if (/sea level|sealevel/.test(ql)) { tools.push('describe_action(world.sea_level)', 'propose_setting'); parts.push(T_('Raising sea level floods the low coast; on this world that drowns two coastal villages. The change regenerates from stage 06, Hydrology.')); acts.push({ kind: 'setting', title: 'Sea level 0.40 → 0.44', sub: 'regenerates stages 06–10 · one undo step · Session mode blocks it', apply: (cc) => { cc._markStale(6); cc._push('assistant · sea level 0.44', () => cc._toast('Sea level back to 0.40'), () => cc._toast('Sea level 0.44')); cc._toast('Sea level 0.44 · stages 06–10 stale · Generate to apply'); } }); return { parts, acts, tools, srcs: ['generation parameters'] }; }
  if (/layer|political|territor/.test(ql)) { tools.push('search_actions("political layer")', 'run_action(view.layer.political)'); parts.push(T_('The Political layer colours every settlement by who holds it in the timeline year, so it changes as you scrub.')); acts.push({ kind: 'view', title: 'Show the Political layer', sub: 'a view change · no undo needed', apply: (cc) => { cc.setState({ layers: Object.assign({}, cc.state.layers, { political: true }) }); cc._dirty = true; } }); return { parts, acts, tools, srcs: ['view state'] }; }
  const go = /(?:go to|year)\s*(-?\d{1,4})/.exec(ql); if (go && /go to|move|jump/.test(ql)) { const yy = +go[1]; tools.push('run_action(timeline.set_year, ' + yy + ')'); parts.push(T_('Moving the timeline moves every panel with it: the map, Inspect, the note’s chronos and this context.')); acts.push({ kind: 'view', title: 'Move the timeline to year ' + yy, sub: 'one shared year · no undo needed', apply: (cc) => cc._setYear(yy) }); return { parts, acts, tools, srcs: [] }; }
  if (!e) {
    tools.push('get_world_summary', 'list_conflicts(year=' + y + ')'); srcs.push('world data');
    const active = conflictsOf(c).filter((x) => y >= x.from && y <= x.to);
    parts.push(T_(c.places.length + ' settlements under four factions: '), lf(0), T_(', '), lf(1), T_(', '), lf(2), T_(' and '), lf(3), T_('. '));
    if (active.length) { parts.push(T_('In '), Y_(c, y), T_(' ' + (active.length === 1 ? 'one conflict is' : active.length + ' conflicts are') + ' active: ')); active.forEach((x, i) => { parts.push(E_(c, x.name, { kind: 'conflict', id: x.id })); if (i < active.length - 1) parts.push(T_(', ')); }); parts.push(T_('.')); }
    else parts.push(T_('Nothing is at war in '), Y_(c, y), T_('.'));
    parts.push(T_('\n\nSelect something and I will answer about it; the chips above show what I can see.'));
    return { parts, acts, tools, srcs };
  }
  tools.push('get_entity(' + e.kind + ', “' + e.name + '”)'); srcs.push('world data');
  if (n) { tools.push('read_source(' + n.name + '.md)'); srcs.push(n.name + '.md'); }
  if (e.kind === 'place') {
    const o = e.o; const own = ownerAt(c, o, y); const per = ownerPeriods(c, o); const near = nearestPlace(c, o);
    if (/link/.test(ql) && n) {
      tools.push('find_unlinked_mentions(' + n.name + '.md)'); const cands = linkEntities(c).filter((x) => x.name !== n.name && n.text.includes(x.name) && !n.text.includes('[[' + x.name)).slice(0, 3);
      if (!cands.length) { parts.push(T_('Every place and faction named in '), N_(c, n), T_(' is already a link.')); return { parts, acts, tools, srcs }; }
      parts.push(T_('These are named in the note but not linked: ')); cands.forEach((x, i) => { const sl = selFromEntity(x); parts.push(sl ? E_(c, x.name, sl) : T_(x.name)); if (i < cands.length - 1) parts.push(T_(', ')); }); parts.push(T_('.'));
      acts.push({ kind: 'note', title: 'Link ' + cands.length + ' mention' + (cands.length > 1 ? 's' : '') + ' in ' + n.name + '.md', sub: 'rewrites only the first mention of each · one undo step', preview: cands.map((x) => x.name + ' → [[' + x.name + ']]').join('\n'), apply: (cc) => { const nn = noteById(cc, n.id); if (!nn) return; let t = nn.text; cands.forEach((x) => { t = t.replace(x.name, '[[' + x.name + ']]'); }); const before = nn.text; saveNote(cc, n.id, t); cc._push('assistant · links in ' + n.name, () => saveNote(cc, n.id, before), () => saveNote(cc, n.id, t)); } });
      return { parts, acts, tools, srcs };
    }
    if (/chronos|entry/.test(ql)) {
      const line = '- [' + y + '] ' + (own === null ? 'Site of the future ' + o.name : o.name + ' under ' + FACTIONS[own].name) + ' | drafted by the assistant';
      parts.push(T_('A chronos line for '), Y_(c, y), T_(', ready to add to '), n ? N_(c, n) : T_(o.name + '.md (it would be created)'), T_('. It shows on the timeline strip as soon as it is saved.'));
      acts.push({ kind: 'note', title: n ? 'Add to ' + n.name + '.md' : 'Create ' + o.name + '.md with it', sub: 'appended inside the chronos block · marked as drafted · one undo step', preview: line, apply: (cc) => cc._askAppendChronos(e, line) });
      return { parts, acts, tools, srcs };
    }
    if (/history/.test(ql)) {
      tools.push('get_ownership(' + o.name + ')', 'list_conflicts(place)');
      parts.push(T_(o.name + ' was founded in '), Y_(c, o.founded), T_('. '));
      per.forEach((q, i) => { parts.push(T_(i ? 'From ' : 'It was held by '), lf(q.fac), T_(i ? ' held it from ' : ' from '), Y_(c, q.from), T_(q.to == null ? ' to this day' : ' to '), q.to == null ? T_('') : Y_(c, q.to), T_('. ')); });
      const wars = conflictsOf(c).filter((x) => x.places.includes(o.id)); wars.forEach((x) => { parts.push(T_('It was fought over in '), E_(c, x.name, { kind: 'conflict', id: x.id }), T_(' ('), Y_(c, x.from), T_('). ')); });
      if (n) { const ev = noteEvents(c, n); if (ev.length) { parts.push(T_('\n\nYour note adds ' + ev.length + ' dated entries, the latest '), Y_(c, ev[ev.length - 1].y), T_(': “' + ev[ev.length - 1].t + '”.')); } }
      return { parts, acts, tools, srcs };
    }
    // describe
    parts.push(T_(o.name + ' is a ' + o.tier + (o.capital ? ', the capital of ' : ' of ')), o.fac >= 0 ? lf(o.fac) : T_('no faction'), T_(', with ' + (o.pop || 0).toLocaleString('en-US') + ' people on ' + (o.biome || 'open').toLowerCase() + ' ground at ' + (o.elev || 0) + ' m. '));
    if (own === null) parts.push(T_('In '), Y_(c, y), T_(' it does not exist yet: it is founded in '), Y_(c, o.founded), T_('. '));
    else if (own !== o.fac) parts.push(T_('In '), Y_(c, y), T_(' it is still held by '), lf(own), T_('. '));
    if (near) parts.push(T_('Its nearest neighbour is '), E_(c, near.name, { kind: 'place', id: near.id }), T_(', ' + fmtDist(Math.hypot(near.x - o.x, near.y - o.y), s.prefs.units) + ' away. '));
    if (n) { const hs = (n.text.match(/^#{2,3} (.+)$/gm) || []).map((h) => h.replace(/^#+ /, '')); parts.push(T_('\n\nYour note covers '), N_(c, n, hs[0] || ''), T_(hs.length > 1 ? ' and ' + (hs.length - 1) + ' more sections.' : '.')); }
    else parts.push(T_('\n\nIt has no note yet.'));
    const prose = o.name + ' is a ' + o.tier + ' of ' + (o.pop || 0).toLocaleString('en-US') + ' on ' + (o.biome || 'open').toLowerCase() + ' ground, ' + (o.fac >= 0 ? 'held by ' + FACTIONS[o.fac].name : 'unclaimed') + '.';
    acts.push({ kind: 'note', title: n ? 'Append to ' + n.name + '.md' : 'Create ' + o.name + '.md with this', sub: 'under “## Description (assistant)” · your own text is never changed · one undo step', preview: prose, apply: (cc) => cc._askAppend(e, '## Description (assistant)\n\n' + prose) });
    return { parts, acts, tools, srcs };
  }
  if (e.kind === 'faction') {
    const i = e.i; const held = c.places.filter((p) => ownerAt(c, p, y) === i); const cap = c.places.find((p) => p.fac === i && p.capital);
    if (/territor|show/.test(ql)) { tools.push('run_action(view.layer.political)'); parts.push(lf(i), T_(' holds ' + held.length + ' settlements in '), Y_(c, y), T_('. The Political layer shows its ground.')); acts.push({ kind: 'view', title: 'Show the Political layer', sub: 'a view change · no undo needed', apply: (cc) => { cc.setState({ layers: Object.assign({}, cc.state.layers, { political: true }) }); cc._dirty = true; } }); return { parts, acts, tools, srcs }; }
    const wars = conflictsOf(c).filter((x) => x.sides.includes(i) && y >= x.from && y <= x.to);
    parts.push(lf(i), T_(' is a ' + e.o.gov.toLowerCase() + ' of '), E_(c, e.o.rel, cultureSel(e.o.rel)), T_(', ruled from '), cap ? E_(c, cap.name, { kind: 'place', id: cap.id }) : T_('no capital'), T_('. In '), Y_(c, y), T_(' it holds ' + held.length + ' settlements'));
    if (wars.length) { parts.push(T_(' and is at war: ')); wars.forEach((x, j) => { parts.push(E_(c, x.name, { kind: 'conflict', id: x.id }), T_(' against '), lf(x.sides.find((q) => q !== i))); if (j < wars.length - 1) parts.push(T_('; ')); }); parts.push(T_('.')); } else parts.push(T_(' and is at peace.'));
    return { parts, acts, tools, srcs };
  }
  if (e.kind === 'conflict') {
    const x = e.o; tools.push('get_conflict(' + x.name + ')');
    parts.push(E_(c, x.name, { kind: 'conflict', id: x.id }), T_(' ran from '), Y_(c, x.from), T_(' to '), Y_(c, x.to), T_(', '), lf(x.sides[0]), T_(' against '), lf(x.sides[1]), T_('. It touched '));
    (e.ps || []).forEach((p, j) => { parts.push(E_(c, p.name, { kind: 'place', id: p.id })); if (j < e.ps.length - 1) parts.push(T_(' and ')); }); parts.push(T_('. ' + x.sub + '.'));
    if (/chronos|note/.test(ql)) { const line = '- [' + x.from + '~' + x.to + '] ' + x.name + ' | ' + FACTIONS[x.sides[0]].name + ' vs ' + FACTIONS[x.sides[1]].name; acts.push({ kind: 'note', title: 'Add to Campaign log.md', sub: 'into its chronos block · one undo step', preview: line.replace('- [', '@ ['), apply: (cc) => { const cl = noteByName(cc, 'Campaign log'); if (cl) cc._askAppendChronos({ kind: 'note', id: cl.id, name: cl.name, noteName: cl.name, entityKind: 'note' }, line.replace('- [', '@ [')); } }); }
    return { parts, acts, tools, srcs };
  }
  if (e.kind === 'note' && n) {
    const hs = (n.text.match(/^#{1,3} (.+)$/gm) || []).map((h) => h.replace(/^#+ /, '')); const links = (n.text.match(/\[\[([^\]|#]+)/g) || []).map((m) => m.slice(2));
    parts.push(N_(c, n), T_(' has ' + wordCount(n.text) + ' words in ' + hs.length + ' sections and links to ' + links.length + ' notes or places'));
    const first = links[0] && selFromName(c, links[0]); if (first) parts.push(T_(', starting with '), E_(c, links[0], first)); parts.push(T_('.'));
    return { parts, acts, tools, srcs };
  }
  parts.push(T_('I can see '), E_(c, e.name, { kind: e.kind, id: e.id }), T_(' and the year '), Y_(c, y), T_('. Ask me to describe it, or to link it from a note.'));
  return { parts, acts, tools, srcs };
}
function askVals(c) {
  const s = c.state; const A = s.ask || { msgs: [], draft: '' }; const on = !!s.prefs.assistant;
  const e = entOf(c, subjectOf(c));
  const msgs = (A.msgs || []).map((m, mi) => {
    const user = m.role === 'user';
    const parts = (m.parts || []).slice(0, m.shown == null ? 1e9 : m.shown).map((p) => ({ isText: p.t === 'text', isCite: p.t === 'cite', s: p.s, title: p.title || '', tap: () => p.run && p.run(c) }));
    return { user, align: user ? 'flex-end' : 'flex-start', bg: user ? 'var(--wash)' : 'var(--chip)', bord: user ? 'rgba(224,163,74,.35)' : 'var(--hair)', parts, thinking: !!m.pending, ctxLine: m.ctx || '', ctxDisp: user && m.ctx ? 'block' : 'none', tools: m.tools && m.tools.length ? 'used · ' + m.tools.join(' · ') : '', toolsDisp: !user && m.tools && m.tools.length && !m.pending ? 'block' : 'none', srcs: m.srcs && m.srcs.length ? 'sources · ' + m.srcs.join(' · ') : '', srcsDisp: !user && m.srcs && m.srcs.length && !m.pending ? 'block' : 'none',
      acts: (m.acts || []).map((a, ai) => ({ title: a.title, sub: a.sub, preview: a.preview || '', prevDisp: a.preview ? 'block' : 'none', state: a.state || '', open: !a.state && !m.pending, stateDisp: a.state ? 'block' : 'none', kindLabel: { setting: 'SETTING · PROPOSED', view: 'VIEW', note: 'NOTE · PROPOSED' }[a.kind] || 'ACTION', apply: () => c._askAct(mi, ai, true), discard: () => c._askAct(mi, ai, false) })) };
  });
  const chips = askChips(c).map((x) => ({ label: x.label, d: x.d, title: x.title, off: x.off, deco: x.off ? 'line-through' : 'none', op: x.off ? 0.5 : 1, tap: () => { if (x.fixed) return; const off = Object.assign({}, A.off || {}); off[x.id] = !off[x.id]; c.setState({ ask: Object.assign({}, A, { off }) }); }, aria: (x.off ? 'include ' : 'leave out ') + x.label }));
  const keep = s.prefs.askSave;
  return { savePrompt: on && keep == null && (A.msgs || []).length > 0, saveLabel: keep ? 'saved with the world' : 'session only', saveCol: keep ? 'var(--acc)' : 'var(--dim)',
    saveTitle: keep ? 'Kept inside the world file; it travels with the world when you share it · Preferences changes this' : 'One conversation per session: New clears it, quitting ends it · Preferences ▸ Save the assistant conversation with the world',
    saveYes: () => { setK(c, 'prefs.askSave', true); c._toast('This conversation is saved with the world · Preferences changes it'); }, saveNo: () => { setK(c, 'prefs.askSave', false); c._toast('Kept for this session only · Preferences changes it'); },
    on, off: !on, prov: V(c, 'as.prov', 'LM Studio (local network)') + ' · ' + V(c, 'as.model', 'qwen3.8-27b'), chips, msgs, empty: !msgs.length, prompts: askPrompts(c, e).map((p) => ({ label: p, tap: () => c._askSend(p) })), draft: A.draft || '', busy: !!A.busy, sendBg: (A.draft || '').trim() && !A.busy ? 'var(--acc)' : 'var(--chip)', sendCol: (A.draft || '').trim() && !A.busy ? 'var(--accInk)' : 'var(--dim)', followNote: s.ctxPin ? 'pinned to ' + (e ? e.name : '—') : 'follows the selection', hint: 'Enter sends · Shift+Enter new line · nothing is sent until you press it' };
}

/* ---------------- the Context panel's modules (Ruling DY, PQ1: "Tabs you arrange") ----------------
   The panel is a host and a module is one view of the one subject in the one year. Inspect · Note · Ask are the default
   tabs; the ten after them are the modules the owner chose to offer (Compare was offered and not taken: pinning the
   panel and its "selection is now …" banner stay the way to compare two subjects, flow F8).
   - Each module names the shipped builder it reuses (`from`), so in the build the panel and that window share one
     implementation. The rows below are the mockup's stand-ins; a figure the stand-in world does not hold is labelled
     "stand-in", and a module with nothing to show says so instead of showing zeros.
   - A module that does not fit the subject's kind is hidden from the strip, never removed, and comes back with a
     fitting subject. `kinds: 'any'` fits everything; 'none' is "nothing selected".
   - The arrangement is the person's: kept with their settings (here: this browser), never in the world file. */
const CTX_CORE = ['inspect', 'note', 'ask'];
const CTX_MODS = [
  { id: 'inspect', tab: 'INSPECT', name: 'Inspect', kinds: 'any', key: 'Ctrl+1', what: 'facts, every name a link', from: 'right_dock.gd (RightDock) builders' },
  { id: 'note', tab: 'NOTE', name: 'Note', kinds: 'any', key: 'Ctrl+2', what: 'the subject’s own note, written here', from: 'markdown_editor.gd' },
  { id: 'ask', tab: 'ASK', name: 'Ask', kinds: 'any', key: 'Ctrl+3', what: 'the assistant, about the subject', from: 'llm_assistant_window.gd' },
  { id: 'history', tab: 'HISTORY', name: 'History', kinds: ['place', 'faction', 'landmark'], what: 'holders over time, dated events, the note’s chronos', from: 'Place editor ▸ Timeline and Political history; faction window ▸ History' },
  { id: 'economy', tab: 'ECONOMY', name: 'Economy', kinds: ['place', 'faction'], what: 'production, trade and notables; currency and tariffs', from: 'Place editor ▸ Economy & notables; faction window ▸ Economy' },
  { id: 'military', tab: 'MILITARY', name: 'Military', kinds: ['faction', 'place'], what: 'armies, garrisons, conflicts', from: 'faction window ▸ Military' },
  { id: 'faith', tab: 'FAITH', name: 'Faith, culture & relations', kinds: ['faction', 'culture'], what: 'faith group, culture card, relations', from: 'faction window ▸ Identity (faith group, culture card) and Relations' },
  { id: 'territory', tab: 'TERRITORY', name: 'Territory', kinds: ['faction'], what: 'claims, provinces, influence, places held', from: 'faction window ▸ Territory' },
  { id: 'layout', tab: 'LAYOUT', name: 'City layout', kinds: ['place'], what: 'the settlement’s layout', from: 'Place editor ▸ Layout; the City viewer' },
  { id: 'here', tab: 'HERE', name: 'Here', kinds: ['place', 'landmark', 'none'], what: 'the ground under the subject, or under the sample pin', from: 'right_dock.gd’s cursor sample (SAMPLE_FIELDS)' },
  { id: 'routes', tab: 'ROUTES', name: 'Routes & journeys', kinds: ['place', 'landmark'], what: 'roads, journeys through it, plan from here', from: 'right_dock.gd’s route context; journey_planner_view.gd' },
  { id: 'links', tab: 'LINKS', name: 'Links', kinds: ['place', 'landmark', 'faction', 'culture', 'conflict', 'note'], what: 'notes that link here, unlinked mentions', from: 'cartalith-vault’s backlink index' },
  { id: 'recent', tab: 'RECENT', name: 'Recent & bookmarks', kinds: 'any', what: 'the selection history, and bookmarks', from: 'the one selection’s history (session only); bookmarks are new and saved in the world file' }
];
const KIND_WORDS = { place: 'settlements', landmark: 'landmarks', faction: 'factions', culture: 'cultures', conflict: 'conflicts', note: 'notes', none: 'nothing selected' };
function modOf(id) { return CTX_MODS.find((m) => m.id === id) || null; }
function modFits(m, kind) { return !!m && (m.kinds === 'any' || m.kinds.includes(kind || 'none')); }
function modKindsText(m) { return m.kinds === 'any' ? 'every subject' : m.kinds.map((k) => KIND_WORDS[k]).join(', '); }
function ctxModsOf(c) { const a = c.state.ctxMods; return Array.isArray(a) && a.length ? a : CTX_CORE.slice(); }
function subjKind(e) { return e ? e.kind : 'none'; }
/* Window ▸ Context panel ▸ MODULES: one check row per module, checked when it is in the panel. Not on the phone, which
   has no panel (Ruling DX PQ13 keeps the phone's Context panel rows to the sheet, the vault and the assistant). */
function ctxModuleMenuItems() {
  return [{ t: 'band', label: 'MODULES', notPhone: true }]
    .concat(CTX_MODS.filter((m) => !CTX_CORE.includes(m.id)).map((m) => ({ t: 'check', label: m.name, key: 'ctxmod.' + m.id, notPhone: true })))
    .concat([{ t: 'sep', notPhone: true }, { t: 'live', label: 'Reset panel', act: 'ctxReset', notPhone: true }]);
}
/* A small deterministic number from a string, for the stand-in figures (never Math.random: a figure must not change
   between two renders of the same subject). */
function standIn(str, lo, hi) { let h = 2166136261; for (const ch of String(str)) { h ^= ch.charCodeAt(0); h = Math.imul(h, 16777619); } return lo + ((h >>> 0) % (hi - lo + 1)); }
const SI = 'stand-in';
const GOODS = { 'temperate forest': 'timber · charcoal · game', taiga: 'timber · furs', 'tropical forest': 'hardwood · dyes', grassland: 'grain · wool · horses', steppe: 'horses · hides', savanna: 'cattle · hides', desert: 'salt · dates · glass', tundra: 'furs · ivory', ice: 'none worth the walk', lake: 'fish · reed', ocean: 'fish · salt' };
const CURRENCY = ['silver mark', 'sevjuni crown', 'iron bit', 'ancestor shell'];
function modFooter(m) { return info('Shared with ' + m.from + ' in the build: one builder, two hosts.'); }
function modRows(c, id, e) {
  const m = modOf(id); if (!m) return [];
  if (!modFits(m, subjKind(e))) return [info(m.name + ' shows for ' + modKindsText(m) + '. Select one of those, or pin this to one.')];
  const fn = { history: modHistory, economy: modEconomy, military: modMilitary, faith: modFaith, territory: modTerritory, layout: modLayout, here: modHere, routes: modRoutes, links: modLinks, recent: modRecent, inspect: (cc, ee) => ctxInspectRows(cc, ee) }[id];
  if (!fn) return [info(m.name + ' lives in the panel only.')];
  return fn(c, e).concat(id === 'inspect' ? [] : [modFooter(m)]);
}
/* History: who held it and when, what happened there, and the dated lines of its note. Every year moves the timeline. */
function modHistory(c, e) {
  const s = c.state, y = s.year; const n = noteOfSubject(c, e); const chron = noteEvents(c, n);
  const chronRows = chron.length ? chron.map((ev) => LK(String(ev.y) + (ev.y1 != null ? '–' + ev.y1 : ''), [yearLink(c, ev.y, 'year ' + ev.y)], ev.t.slice(0, 34))) : [info(n ? 'Its note has no chronos lines yet.' : 'No note yet, so no chronos lines.')];
  if (e.kind === 'place') {
    const o = e.o; const per = ownerPeriods(c, o); const own = ownerAt(c, o, y);
    const wars = conflictsOf(c).filter((k) => k.places.includes(o.id));
    const rows = [H('HOLDERS', per.length + (per.length === 1 ? ' period' : ' periods'))];
    per.forEach((q) => { const on = own !== null && y >= q.from && (q.to == null || y < q.to); rows.push(LK(q.from + ' – ' + (q.to == null ? 'now' : q.to), [selLink(c, FACTIONS[q.fac].name, { kind: 'faction', id: 'f' + q.fac }, { dot: FACTIONS[q.fac].col }), yearLink(c, q.from, '⟲ ' + q.from)], on ? '◂ year ' + y : '')); });
    rows.push(H('EVENTS'), o.founded != null ? LK('Founded', [yearLink(c, o.founded)]) : read('Founded', 'not recorded'));
    wars.forEach((k) => rows.push(LK(k.from + '–' + k.to, [selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col }), yearLink(c, k.from, '⟲ ' + k.from)])));
    if (!wars.length) rows.push(read('Conflicts here', 'none recorded'));
    rows.push(H('FROM ITS NOTE', chron.length + ' dated')); return rows.concat(chronRows);
  }
  if (e.kind === 'faction') {
    const i = e.i; const ev = [];
    c.places.forEach((p) => ownerPeriods(c, p).forEach((q) => { if (q.fac === i && q.why) ev.push({ y: q.from, label: 'gained ' + p.name, sel: { kind: 'place', id: p.id } }); if (q.fac === i && q.to != null) ev.push({ y: q.to, label: 'lost ' + p.name, sel: { kind: 'place', id: p.id } }); }));
    conflictsOf(c).filter((k) => k.sides.includes(i)).forEach((k) => ev.push({ y: k.from, label: k.name, sel: { kind: 'conflict', id: k.id }, col: k.col }));
    ev.sort((a, b) => a.y - b.y);
    const rows = [H('DATED EVENTS', ev.length + '')].concat(ev.length ? ev.map((x) => LK(String(x.y), [selLink(c, x.label, x.sel, { dot: x.col || '' }), yearLink(c, x.y, '⟲ ' + x.y)])) : [info('No dated events for this faction in the stand-in world.')]);
    rows.push(H('FROM ITS NOTE', chron.length + ' dated')); return rows.concat(chronRows);
  }
  return [H('HISTORY'), info('Landmarks have no ownership record in the stand-in world. Dated lines in its note are its history.'), H('FROM ITS NOTE', chron.length + ' dated')].concat(chronRows);
}
/* Economy. The stand-in world holds population and biome; every other figure here is a labelled stand-in. */
function modEconomy(c, e) {
  const s = c.state, y = s.year;
  if (e.kind === 'place') {
    const o = e.o; const roads = c.ways.filter((w) => w.a === o || w.b === o);
    const market = { city: 'regional market', town: 'weekly market', village: 'a fair each season', hamlet: 'none' }[o.tier] || 'none';
    const near = roads.map((w) => (w.a === o ? w.b : w.a)).sort((a, b) => (b.pop || 0) - (a.pop || 0))[0];
    return [H('ECONOMY', 'figures marked stand-in are not in the stand-in world'),
      read('Population', o.pop ? o.pop.toLocaleString('en-US') : 'new · grows on recompute'), read('Ground', o.biome || '—'),
      read('Produces', (GOODS[o.biome] || 'grain · livestock') + ' · ' + SI), read('Market', market + ' · ' + SI),
      read('Tax yield', (o.pop ? Math.round(o.pop * 0.08).toLocaleString('en-US') + ' silver a year' : 'none yet') + ' · ' + SI),
      read('Roads', roads.length + (roads.length === 1 ? ' way' : ' ways')), near ? LK('Busiest neighbour', [selLink(c, near.name, { kind: 'place', id: near.id })]) : read('Busiest neighbour', 'no road yet'),
      H('NOTABLES', SI), read('Households of note', String(Math.max(1, Math.round((o.pop || 300) / 2500))) + ' · ' + SI),
      J(c, 'Economy & notables in the Place editor…', 'win:place', 'the full editor window, following this settlement')];
  }
  const i = e.i, f = e.o; const held = c.places.filter((p) => ownerAt(c, p, y) === i); const cap = c.places.find((p) => p.fac === i && p.capital);
  const people = held.reduce((a, p) => a + (p.pop || 0), 0);
  return [H('ECONOMY · YEAR ' + y, 'figures marked stand-in are not in the stand-in world'),
    read('Settlements held', held.length + ' of ' + c.places.length), read('People', people.toLocaleString('en-US')),
    read('Currency', CURRENCY[i % CURRENCY.length] + ' · ' + SI), read('Tariff on roads', standIn(f.name, 2, 9) + ' % · ' + SI),
    read('Treasury', Math.round(people * 0.02).toLocaleString('en-US') + ' k silver · ' + SI),
    read('Main exports', (cap ? (GOODS[cap.biome] || 'grain · livestock') : '—') + ' · ' + SI),
    J(c, 'Currency and tariffs in the Factions window…', 'win:factions', 'its Economy tab')];
}
/* Military. Army sizes repeat the Factions window's own stand-in figures, so the two places agree. */
function modMilitary(c, e) {
  const s = c.state, y = s.year;
  if (e.kind === 'faction') {
    const i = e.i; const fx = c.factions[i] || { places: 0 }; const held = c.places.filter((p) => ownerAt(c, p, y) === i);
    const wars = conflictsOf(c).filter((k) => k.sides.includes(i)); const active = wars.filter((k) => y >= k.from && y <= k.to);
    const gar = held.reduce((a, p) => a + (p.capital ? 1200 : 80), 0);
    return [H('ARMIES · YEAR ' + y, active.length ? 'at war' : 'at peace'),
      read('Standing army', (fx.places * 1.8).toFixed(1) + ' k · ' + SI), read('Field army', (fx.places * 4.1).toFixed(1) + ' k · ' + SI),
      read('Garrisons', gar.toLocaleString('en-US') + ' in ' + held.length + ' places · ' + SI),
      active.length ? LK('At war', active.map((k) => selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col }))) : read('At war', 'no'),
      H('CONFLICTS', wars.length + '')].concat(wars.length ? wars.map((k) => LK(k.from + '–' + k.to, [selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col })], y >= k.from && y <= k.to ? '◂ now' : '')) : [info('No conflicts recorded for this faction.')]).concat([J(c, 'Military in the Factions window…', 'win:factions', 'standing, field and emergency armies')]);
  }
  const o = e.o; const own = ownerAt(c, o, y); const wars = conflictsOf(c).filter((k) => k.places.includes(o.id));
  return [H('DEFENCE · YEAR ' + y),
    own != null && own >= 0 ? LK('Held by', [selLink(c, FACTIONS[own].name, { kind: 'faction', id: 'f' + own }, { dot: FACTIONS[own].col })]) : read('Held by', own === null ? 'not founded yet' : 'unclaimed'),
    read('Fortified', o.fort ? 'yes' : 'no'), read('Garrison', (o.capital ? '1 200' : '80') + ' · ' + SI),
    wars.length ? LK('Fought over in', wars.map((k) => selLink(c, k.name, { kind: 'conflict', id: k.id }, { dot: k.col }))) : read('Fought over in', 'no recorded conflict')];
}
/* Faith, culture & relations: all from the stand-in world's factions and the relations table. */
function modFaith(c, e) {
  if (e.kind === 'faction') {
    const i = e.i, f = e.o; const same = FACTIONS.map((g, j) => [g, j]).filter(([g, j]) => j !== i && g.rel === f.rel);
    return [H('FAITH & CULTURE'), LK('Faith', [selLink(c, f.rel, cultureSel(f.rel), { frame: false })]), read('Government', f.gov),
      same.length ? LK('Shares the faith with', same.map(([g, j]) => selLink(c, g.name, { kind: 'faction', id: 'f' + j }, { dot: g.col }))) : read('Shares the faith with', 'no other faction'),
      H('RELATIONS')].concat(FACTIONS.map((g, j) => (j === i ? null : LK(g.name.split(' ')[0], [selLink(c, g.name, { kind: 'faction', id: 'f' + j }, { dot: g.col })], RELATIONS[i][j]))).filter(Boolean));
  }
  const fs = FACTIONS.map((f, i) => [f, i]).filter(([f]) => f.rel === e.name); const ps = c.places.filter((p) => p.fac >= 0 && FACTIONS[p.fac].rel === e.name);
  const pairs = []; fs.forEach(([a, i]) => fs.forEach(([b, j]) => { if (i < j) pairs.push(LK(a.name.split(' ')[0] + ' · ' + b.name.split(' ')[0], [selLink(c, b.name, { kind: 'faction', id: 'f' + j }, { dot: b.col })], RELATIONS[i][j])); }));
  return [H('CULTURE & FAITH'), LK('Followed by', fs.map(([f, i]) => selLink(c, f.name, { kind: 'faction', id: 'f' + i }, { dot: f.col }))), read('Settlements', String(ps.length)), read('People', ps.reduce((a, p) => a + (p.pop || 0), 0).toLocaleString('en-US')),
    H('BETWEEN ITS FOLLOWERS', pairs.length ? '' : 'one faction')].concat(pairs.length ? pairs : [info('Only one faction follows it, so there is no relation inside the faith to show.')]);
}
/* Territory: places held, gained and lost come from the ownership periods; claims, provinces and influence are stand-ins. */
function modTerritory(c, e) {
  const s = c.state, y = s.year, i = e.i, f = e.o; const held = c.places.filter((p) => ownerAt(c, p, y) === i); const today = c.places.filter((p) => p.fac === i);
  const cap = c.places.find((p) => p.fac === i && p.capital);
  return [H('TERRITORY · YEAR ' + y), read('Settlements held', held.length + ' (' + today.length + ' today)'), cap ? LK('Capital', [selLink(c, cap.name, { kind: 'place', id: cap.id })]) : read('Capital', '—'),
    read('Provinces', standIn(f.name + 'p', 2, 6) + ' · ' + SI), read('Claims beyond its border', standIn(f.name + 'c', 0, 3) + ' · ' + SI), read('Influence', ['low', 'regional', 'wide'][standIn(f.name + 'i', 0, 2)] + ' · ' + SI),
    { k: 'acts', opts: [{ label: 'Show the Political layer', variant: 'primary', flex: '1 1 60%', tap: () => { c.setState({ layers: Object.assign({}, s.layers, { political: true }) }); c._dirty = true; c._toast('Political layer on · coloured by who holds what in year ' + y); } }] },
    H('PLACES', held.length + ' in year ' + y)].concat(held.slice(0, 8).map((p) => ({ k: 'item', label: p.name, sub: p.tier + (p.capital ? ' · capital' : '') + (p.fac !== i ? ' · lost later' : ''), right: p.pop ? p.pop.toLocaleString('en-US') : '', dot: f.col, tap: () => c._select({ kind: 'place', id: p.id }, { frame: true }) })));
}
/* City layout: walls come from the settlement's fortified flag; the rest is a labelled stand-in until the layout is read. */
function modLayout(c, e) {
  const o = e.o; const plan = { city: 'walled core, organic outskirts', town: 'organic · market square', village: 'street village', hamlet: 'cluster of farmsteads' }[o.tier] || 'cluster';
  const districts = { city: 6, town: 3, village: 1, hamlet: 1 }[o.tier] || 1;
  return [H('LAYOUT', o.name), read('Plan', plan + ' · ' + SI), read('Districts', districts + ' · ' + SI), read('Walls', o.fort ? 'full circuit' : 'none'), read('Buildings', Math.max(4, Math.round((o.pop || 120) / 6)).toLocaleString('en-US') + ' · ' + SI),
    J(c, 'City viewer…', 'win:city', 'its own window, following the selection'), J(c, 'Layout in the Place editor…', 'win:place', 'the full editor window')];
}
/* Here: the ground under a settlement or landmark, or under the sample pin when nothing is selected. */
function modHere(c, e) {
  const s = c.state;
  if (!e) return s.sample ? rowsSample(c) : [H('HERE'), info('Click the map to drop a sample pin: this shows the ground under it. Select a settlement or landmark to read the ground under that instead.')];
  const a = sampleAt(c, { x: e.o.x, y: e.o.y }); if (!a) return [info('No world to sample yet.')];
  return [H('THE GROUND UNDER ' + e.name.toUpperCase()), read('Elevation', a.elev + ' m', 'var(--acc)'), read('Slope', a.slope + '°'), read('Temperature', a.temp + ' °C'), read('Precipitation', a.rain + ' mm/yr'), read('Drainage', a.flow > 200 ? 'river · ' + a.flow + ' cells upstream' : 'hillslope'), read('Biome', a.biome), read('Lithology', a.lith), read('Plate', a.plate), read('Position', Math.round(e.o.x) + ' km E · ' + Math.round(e.o.y) + ' km S')];
}
/* Routes & journeys: the ways that start here, the current journey, and the planner from here. */
function modRoutes(c, e) {
  const s = c.state; const o = e.kind === 'place' ? e.o : nearestPlace(c, e.o); if (!o) return [info('No settlement near enough to route from.')];
  const roads = c.ways.filter((w) => w.a === o || w.b === o).sort((a, b) => a.km - b.km); const at = c.route.indexOf(o);
  const rows = [];
  if (e.kind !== 'place') rows.push(LK('Nearest settlement', [selLink(c, o.name, { kind: 'place', id: o.id })], fmtDist(Math.hypot(o.x - e.o.x, o.y - e.o.y), s.prefs.units)));
  rows.push(H('ROADS FROM ' + o.name.toUpperCase(), roads.length + ''));
  if (roads.length) roads.slice(0, 8).forEach((w) => { const b = w.a === o ? w.b : w.a; rows.push(LK(w.kind, [selLink(c, b.name, { kind: 'place', id: b.id })], fmtDist(w.km, s.prefs.units))); });
  else rows.push(info('No way starts here yet. Draw one with the Way tool (W), or Generate roads in CIVIL.'));
  rows.push(H('JOURNEYS'), read('The current journey', at >= 0 ? 'stop ' + (at + 1) + ' of ' + c.route.length + ' · ' + c.route[0].name + ' → ' + c.route[c.route.length - 1].name : 'does not pass here'));
  rows.push({ k: 'acts', opts: [{ label: 'Plan a journey from here', variant: 'primary', flex: '1 1 55%', tap: () => c._planFrom(o) }, { label: 'Saved journeys…', flex: '1 1 35%', tap: () => c._openWin('journeys') }] });
  return rows;
}
/* Links: notes that link to the subject, and unlinked mentions with one-click linking (the vault's backlink index). */
function modLinks(c, e) {
  const n = noteOfSubject(c, e); const target = n || { id: null, name: e.noteName };
  const bl = backlinksOf(c, target); const rows = [H('LINKED MENTIONS', String(bl.linked.length))];
  if (bl.linked.length) bl.linked.slice(0, 10).forEach((h) => rows.push({ k: 'item', label: h.from.name, sub: h.snippet, dot: 'var(--acc)', tap: () => c._select({ kind: 'note', id: h.from.id }, { tab: 'note' }) }));
  else rows.push(info('No note links to ' + e.name + ' yet.'));
  rows.push(H('UNLINKED MENTIONS', String(bl.unlinked.length)));
  if (bl.unlinked.length) bl.unlinked.slice(0, 8).forEach((h) => { rows.push({ k: 'item', label: h.from.name, sub: h.snippet, dot: 'var(--bord)', tap: () => c._select({ kind: 'note', id: h.from.id }, { tab: 'note' }) }); rows.push({ k: 'acts', opts: [{ label: 'Link it in ' + h.from.name, flex: '1 1 100%', tap: () => linkMentionUndoable(c, target, h) }] }); });
  else rows.push(info('Every mention of ' + e.name + ' in the vault is already a link.'));
  if (!n) rows.push(info('It has no note of its own yet: Note ▸ Create makes one, already linked to it.'));
  return rows;
}
/* Recent & bookmarks: the selection history as a list, and subjects the person bookmarks (new). The history is
   session-only. Bookmarks belong to the world: the build saves them inside the world file (Ruling DY, the owner's answer
   "In the world file"), a new optional slot; the mockup keeps them for the session. */
function modRecent(c, e) {
  const s = c.state; const bm = s.bookmarks || []; const rows = [];
  if (e) { const on = bm.some((x) => selKey(x) === selKey({ kind: e.kind, id: e.id })); rows.push({ k: 'acts', opts: [{ label: on ? '★ Bookmarked · remove' : '☆ Bookmark ' + e.name, variant: on ? '' : 'primary', flex: '1 1 100%', tap: () => c.setState({ bookmarks: on ? bm.filter((x) => selKey(x) !== selKey({ kind: e.kind, id: e.id })) : bm.concat([{ kind: e.kind, id: e.id }]) }) }] }); }
  const ent = (x) => entOf(c, x); const item = (x, sub) => { const ee = ent(x); return ee ? { k: 'item', label: ee.name, sub: SEL_KINDS[ee.kind][0] + (sub ? ' · ' + sub : ''), dot: 'var(--acc)', tap: () => c._select({ kind: ee.kind, id: ee.id }, { frame: true }) } : null; };
  rows.push(H('BOOKMARKS', String(bm.length)));
  if (bm.length) rows.push(...bm.map((x) => item(x)).filter(Boolean)); else rows.push(info('Nothing bookmarked yet.'));
  const seen = new Set(); const back = (s.selBack || []).slice().reverse().filter((x) => { const k = selKey(x); if (seen.has(k) || (e && k === selKey({ kind: e.kind, id: e.id }))) return false; seen.add(k); return true; });
  rows.push(H('RECENT', 'Alt+← goes back'));
  if (back.length) rows.push(...back.slice(0, 10).map((x) => item(x)).filter(Boolean)); else rows.push(info('No earlier selection this session.'));
  return rows;
}
/* One-click linking from the Links module, as one undo step (the vault's own Link button rewrites the same way:
   the first bare mention on that line becomes [[Name]], or [[Name|as written]] when the case differs). */
function linkMentionUndoable(c, target, h) {
  const before = h.from.text; const L = before.split('\n');
  const re = new RegExp('(^|[^\\w\\[])(' + target.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + ')(?![\\w\\]])', 'i');
  L[h.line] = L[h.line].replace(re, (m0, a, b) => a + '[[' + target.name + (b !== target.name ? '|' + b : '') + ']]');
  const after = L.join('\n'); if (after === before) { c._toast('Nothing to link on that line'); return; }
  saveNote(c, h.from.id, after); c._push('link ' + target.name + ' in ' + h.from.name, () => saveNote(c, h.from.id, before), () => saveNote(c, h.from.id, after));
  c._toast('Linked ' + target.name + ' in ' + h.from.name + '.md · Ctrl+Z undoes it');
}

/* ---------------- the Context panel's header, tabs and Note tab ---------------- */
/* Inspect's rows end with the armed tool's draft section: the selection wins and the tool appends below it
   (LARGE_ITEM_RULINGS.md, the rdExtraMode row). */
function ctxDockVals(c, G, rdOverlay, toolRows) {
  const s = c.state; const subj = subjectOf(c); const e = entOf(c, subj);
  const pinned = !!s.ctxPin; const selE = s.sel ? entOf(c, s.sel) : null; const diverged = pinned && !!s.sel && selKey(s.sel) !== selKey(s.ctxPin);
  const n = e ? noteOfSubject(c, e) : null; const ptr = c._ptr(); const kind = subjKind(e);
  /* the modules (Ruling DY, PQ1): the person's tabs in their order, less those that do not fit this subject's kind */
  const mine = ctxModsOf(c); const mods = mine.map(modOf).filter(Boolean); const shown = mods.filter((m) => modFits(m, kind)); const hidden = mods.filter((m) => !modFits(m, kind));
  const tab = shown.some((m) => m.id === s.ctxTab) ? s.ctxTab : 'inspect'; const isMod = !CTX_CORE.includes(tab);
  const drag = s.ctxDrag && s.ctxDrag.active ? s.ctxDrag : null; const di = drag ? mine.indexOf(drag.id) : -1;
  const tabs = shown.map((m) => { const on = tab === m.id; const over = !!drag && drag.over === m.id && drag.id !== m.id; return { id: m.id, label: m.tab + (m.id === 'note' && e && !n ? ' +' : ''), on, col: on ? 'var(--acc)' : 'var(--sec)', edge: on ? 'var(--acc)' : 'transparent', drop: over ? (di < mine.indexOf(m.id) ? 'inset -3px 0 0 var(--acc)' : 'inset 3px 0 0 var(--acc)') : 'none', op: drag && drag.id === m.id ? 0.45 : 1, tap: () => c._tabClick(m.id), down: (ev) => c._tabDown(m.id, ev), move: (ev) => c._tabMove(ev), up: (ev) => c._tabUp(ev), cancel: () => c._tabCancel(), menu: (ev) => c._tabMenuAt(m.id, ev), aria: m.name.toLowerCase() + ' tab' + (m.key ? ' (' + m.key + ')' : ''), title: m.name + (m.key ? ' · ' + m.key : '') + (ptr ? ' · drag to move · right-click for more' : ' · hold for more') }; });
  const addRows = !s.ctxAdd ? [] : [H('TICK TO SHOW AS A TAB', mine.length + ' in the panel')].concat(CTX_MODS.map((m) => { const inP = mine.includes(m.id); const fits = modFits(m, kind); return { k: 'tog', label: m.name, sub: m.what + ' · ' + (m.id === 'inspect' ? 'always in the panel' : (m.kinds === 'any' ? 'every subject' : 'for ' + modKindsText(m))) + (inP && !fits ? ' · hidden for this subject' : ''), on: inP, tap: m.id === 'inspect' ? () => c._toast('Inspect anchors the panel and stays') : () => c._ctxModSet(m.id, !inP) }; }), [info('A ticked module is a tab. One that does not fit the selection hides until a subject it fits is selected. The arrangement is yours: kept with your settings, never in the world file.'), B('Reset panel · Inspect · Note · Ask', () => c._ctxReset())]);
  const TM = s.ctxTabMenu; let tabMenu = { show: false, rows: [] };
  if (TM && modOf(TM.id)) {
    const tm = modOf(TM.id); const vis = shown.map((x) => x.id); const vi = vis.indexOf(TM.id);
    const rows = [{ k: 'nav', d: 'M10 3L5 8l5 5', label: 'Move left', dis: vi <= 0, reason: 'Already the first tab', chev: false, tap: () => c._ctxMove(TM.id, -1) }, { k: 'nav', d: 'M6 3l5 5-5 5', label: 'Move right', dis: vi < 0 || vi >= vis.length - 1, reason: 'Already the last tab', chev: false, tap: () => c._ctxMove(TM.id, 1) }];
    if (ptr && TM.id === 'note') rows.push({ k: 'nav', d: IC.window, label: 'Open in the vault window', sub: 'split view, outline, backlinks', chev: false, tap: () => { c.setState({ ctxTabMenu: null }); if (n) c._openVaultAt(n.id); else c._toast('No note yet · create it in the Note tab first'); } });
    else if (ptr && TM.id !== 'ask') rows.push({ k: 'nav', d: IC.window, label: 'Open in a window', sub: 'follows the selection until pinned', chev: false, tap: () => { c.setState({ ctxTabMenu: null }); c._openWin('mod:' + TM.id); } });
    // Ruling DY PQ1: any tab can open in its own window. Ask's window is the assistant window (the one PW-3 already pops out), holding the session's one conversation (PQ5).
    else if (ptr && TM.id === 'ask') rows.push({ k: 'nav', d: IC.window, label: 'Open in a window', sub: 'the assistant window · the same conversation', chev: false, tap: () => { c.setState({ ctxTabMenu: null }); c._openWin('assistant'); } });
    rows.push({ k: 'nav', d: IC.trash, label: 'Remove', sub: TM.id === 'inspect' ? '' : '+ brings it back', dis: TM.id === 'inspect', reason: 'Inspect anchors the panel and stays', chev: false, tap: () => c._ctxModSet(TM.id, false) });
    tabMenu = { show: true, x: Math.max(6, Math.min((G.rdW || 320) - 266, TM.x || 6)), title: tm.name.toUpperCase(), rows };
  }
  const dn = s.dn && n && s.dn.id === n.id ? s.dn : null; const mode = dn ? dn.mode : 'write';
  const note = { has: !!n, none: !!e && !n, nothing: !e, name: n ? n.name : '', text: n ? c._dnText(n) : '', path: n ? vtPath(n) : '', words: n ? wordCount(c._dnText(n)) + ' words' : '', isRead: mode === 'read', isWrite: mode === 'write', saved: dn && dn.dirty ? 'editing…' : (n ? 'saved · ' + vtAgo(n.edited) : ''), priv: !!(n && n.private),
    canRevert: !!n && c._histChanged(n.id), revert: () => n && c._revertAsk(n.id, 'dock'), revertTitle: n ? 'Revert to the note as you first opened it this session (' + c._hist(n.id).at0 + ')' : '',
    chip: !!n && !ptr && mode === 'write' && !!(s.caretLink && s.caretLink.where === 'dock'), chipLabel: s.caretLink ? 'Open ' + s.caretLink.name + ' ›' : '', chipTap: () => s.caretLink && c._followFromWrite(s.caretLink.name),
    taTitle: ptr ? 'Markdown · Ctrl+click a link to open it · Ctrl+E preview' : 'Markdown · put the caret in a link to open it',
    createLabel: e ? 'Create ' + e.noteName + '.md' : '', createSub: e ? 'from the ' + (NOTE_KINDS[e.entityKind] ? NOTE_KINDS[e.entityKind][0].toLowerCase() : 'note') + ' template, linked to this ' + SEL_KINDS[e.kind][0].toLowerCase() : '',
    modeRead: () => c._dnMode('read'), modeWrite: () => c._dnMode('write'), create: () => c._dnCreate(e), openVault: () => n && c._openVaultAt(n.id), readBg: mode === 'read' ? 'var(--wash)' : 'transparent', readCol: mode === 'read' ? 'var(--acc)' : 'var(--sec)', writeBg: mode === 'write' ? 'var(--wash)' : 'transparent', writeCol: mode === 'write' ? 'var(--acc)' : 'var(--sec)' };
  return {
    show: s.rdockOpen, w: G.rdW, shadow: rdOverlay ? '-16px 0 40px rgba(0,0,0,.45)' : 'none', z: rdOverlay ? 9 : 5,
    kind: e ? SEL_KINDS[e.kind][0].toUpperCase() : (s.sample ? 'SAMPLE' : 'CONTEXT'), name: e ? e.name : (s.sample ? 'Pinned sample' : 'Nothing selected'), sub: e ? e.sub : (s.sample ? Math.round(s.sample.x) + ' km E · ' + Math.round(s.sample.y) + ' km S' : 'click anything: map, note link, window row, answer'), d: e ? SEL_KINDS[e.kind][1] : IC.inspect,
    canBack: (s.selBack || []).length > 0, canFwd: (s.selFwd || []).length > 0, backCol: (s.selBack || []).length ? 'var(--body)' : 'var(--hair)', fwdCol: (s.selFwd || []).length ? 'var(--body)' : 'var(--hair)',
    pinned, pinCol: pinned ? 'var(--acc)' : 'var(--sec)', pinBg: pinned ? 'var(--wash)' : 'transparent', pinAria: pinned ? 'unpin: follow the selection again' : 'pin: keep this panel on ' + (e ? e.name : 'this'), pinTitle: pinned ? 'Pinned · click to follow the selection again' : 'Follows the selection · click to pin',
    follow: pinned ? 'PINNED' : 'FOLLOWING', followCol: pinned ? 'var(--acc)' : 'var(--dim)', canPin: !!e,
    diverged, divName: selE ? selE.name : '', divShow: () => c.setState({ ctxPin: s.sel }), divFollow: () => c.setState({ ctxPin: null }),
    tabs, isInspect: tab === 'inspect', isNote: tab === 'note', isAsk: tab === 'ask', isMod, isRows: tab === 'inspect' || isMod, rows: tab === 'inspect' ? ctxInspectRows(c, e).concat(toolRows || []) : (isMod ? modRows(c, tab, e) : []), note, ask: tab === 'ask' ? askVals(c) : { msgs: [], chips: [], prompts: [] }, ptr,
    addTap: () => c.setState({ ctxAdd: !s.ctxAdd, ctxTabMenu: null }), addOpen: !!s.ctxAdd, addCol: s.ctxAdd ? 'var(--acc)' : 'var(--sec)', addBg: s.ctxAdd ? 'var(--wash)' : 'transparent',
    addTitle: 'Add a module' + (hidden.length ? ' · hidden for this subject: ' + hidden.map((m) => m.name).join(', ') : ''), hiddenN: hidden.length ? '+' + hidden.length : '', hiddenDisp: hidden.length ? 'inline' : 'none',
    add: { show: !!s.ctxAdd, rows: addRows, w: Math.min(320, (G.rdW || 320) - 12), maxH: Math.max(220, c._H() - (G.bodyT || 0) - (G.bodyB || 0) - 150) }, tabMenu, pop: !!s.ctxAdd || !!tabMenu.show, closePops: () => c.setState({ ctxAdd: false, ctxTabMenu: null })
  };
}

/* ---------------- the busy overlay: the shipped loading screen and UI guard, on every form factor ----------------
   busy_overlay.gd (owner, 2026-10-11): a scrim that takes every input while the engine works, and a card with the
   action, the relief trace (a ridge line that only advances to the stages the engine has reported), the stage line and
   one line of wit per stage. The lines are the shipped ones, two per stage, verbatim from loading_messages.gd NEW_MSGS.
   No cancel button, as shipped: the engine has no cancel entry point. */
const BUSY_MSGS = [
  ['Choosing a planet size. Medium. Always medium.', 'Measuring the radius twice, cutting the crust once.'],
  ['Measuring the map in kilometres, then double-checking the kilometres.', 'Deciding how big a cell is. Bigger than a breadbox.'],
  ['Sketching continents on a napkin first.', 'Deciding how many continents is too many.'],
  ['Pushing plates together until someone says ouch.', 'Subducting the evidence.'],
  ['Locating hot spots. Do not touch the hot spots.', 'Placing volcanoes where the crust looked tired.'],
  ['Raining on every parade, systematically.', 'Holding a grudge against mountains for a million years.'],
  ['Teaching every river the way to the sea.', 'Rivers are asking for directions. Downhill, always downhill.'],
  ['Blowing the prevailing winds in a prevailing direction.', 'Wringing rain out of clouds on the windward slopes.'],
  ['Planting forests where the rain said it was fine.', 'Rolling out grassland by the hectare.'],
  ['Burying ore where prospectors will complain about the walk.', 'Mixing soils: two parts clay, one part ambition.']
];
function busyVals(c) {
  const s = c.state; const R = s.running; if (!R) return { show: false, ticks: [], line: 'M0 0', fill: 'M0 0' };
  const n = STAGE_NAMES.length, SW = 520, SH = 78, per = 24; const frac = Math.min(1, R.i / n); const pts = [];
  for (let k = 0; k <= n * per; k++) { const x = k / (n * per) * SW; if (x > frac * SW + 0.01) break; const st = Math.floor(k / per), t = (k % per) / per; const h = 0.22 + 0.6 * Math.abs(Math.sin((st + 1) * 1.7 + t * Math.PI * (2 + st % 3))) * (0.6 + 0.4 * Math.sin(k * 0.9 + st)); pts.push([x, SH - 8 - h * (SH - 22)]); }
  if (!pts.length) pts.push([0, SH - 8]);
  const line = pts.map((q) => 'L' + q[0].toFixed(1) + ' ' + q[1].toFixed(1)).join(' ').replace(/^L/, 'M');   // the template prefixes "M0 0", so this starts a new subpath
  const last = pts[pts.length - 1]; const fill = line + ' L' + last[0].toFixed(1) + ' ' + (SH - 6) + ' L0 ' + (SH - 6) + ' Z';
  const m = BUSY_MSGS[R.i] || ['Tidying up.', 'Tidying up.'];
  return { show: true, w: Math.min(560, c._W() - 32), title: 'GENERATING · ' + s.worldName, pct: R.i + ' of ' + n + ' stages done', stage: 'stage ' + String(R.i + 1).padStart(2, '0') + ' of ' + n + ' · ' + (STAGE_NAMES[R.i] || ''), msg: m[R.pct < 50 ? 0 : 1], line, fill, hxp: (last[0] / SW * 100).toFixed(2), hyp: (last[1] / SH * 100).toFixed(2), ticks: Array.from({ length: n - 1 }, (_, k) => ({ pct: ((k + 1) / n * 100).toFixed(2) })), foot: 'Input is held until the engine finishes, as shipped. Generation has no cancel.' };
}

/* ---------------- desktop menus: cascading columns (pointer) ---------------- */
/* Desktop menu rows: one line each, accelerator on the right, checks as ticks, submenus as a ▸ that opens a column. */
function deskMenuRows(c, items, drill, openLabel) {
  const rows = [];
  for (const it of items) {
    if (!it || it.mobile) continue;
    if (it.t === 'check') { const on = it.on != null && getK(c, it.key) == null ? it.on : !!getK(c, it.key); rows.push({ k: 'chk', label: it.label, on, accel: it.accel || '', tap: () => setK(c, it.key, !on) }); continue; }
    if (it.t === 'sub') { const reason = smGate(c, it); rows.push({ k: 'nav', d: IC.menu, label: it.label, sub: reason, badge: '▸', badgeCol: it.label === openLabel ? 'var(--acc)' : 'var(--dim)', on: it.label === openLabel, dis: !!reason, reason, chev: false, tap: reason ? () => c._toast(reason) : () => drill(it) }); continue; }
    rows.push(...menuRows(c, [it], drill, 'desktop'));
  }
  return rows;
}
function deskMenuCols(c, src, path) {
  const cols = []; let items = src.items; const p = [];
  cols.push({ title: src.label.toUpperCase(), rows: deskMenuRows(c, items, (it) => c.setState({ dd: Object.assign({}, c.state.dd, { path: [it.label] }) }), path[0]) });
  for (let i = 0; i < path.length; i++) {
    const it = (items || []).find((x) => x.label === path[i] && x.items); if (!it) break; items = it.dyn === 'openWindows' ? openWindowItems(c) : it.items; p.push(path[i]); const at = p.slice();
    cols.push({ title: path[i].toUpperCase(), rows: deskMenuRows(c, items, (sub) => c.setState({ dd: Object.assign({}, c.state.dd, { path: at.concat([sub.label]) }) }), path[i + 1]) });
  }
  return cols;
}

/* ---------------- keyboard (pointer): every accelerator a menu row shows, plus the context keys ---------------- */
function deskKey(c, e) {
  const s = c.state; const t = e.target; const typing = t && (t.tagName === 'TEXTAREA' || t.tagName === 'INPUT');
  if (s.running) { e.preventDefault(); return; }   // the busy overlay holds every key while the engine works
  const k = e.key; const ctl = e.ctrlKey || e.metaKey;
  if (k === 'Escape') { if (s.cfm) { c.setState({ cfm: null }); e.preventDefault(); return; } if (s.ctxAdd || s.ctxTabMenu) { c.setState({ ctxAdd: false, ctxTabMenu: null }); e.preventDefault(); return; } if (s.dd || s.tctx || s.ring || s.layersPop || s.srch) { c.setState({ dd: null, tctx: null, ring: null, layersPop: false, srch: false }); e.preventDefault(); return; } if (s.win) { c.setState({ win: null }); e.preventDefault(); return; } if (typing) { t.blur(); return; } if (s.ctxEdit) { c.setState({ ctxEdit: false, ctxDraft: null }); return; } if (s.sel || s.sample) { c._deselect(); e.preventDefault(); } return; }
  if (ctl && !e.shiftKey && ['1', '2', '3'].includes(k)) { e.preventDefault(); c._ctxTab(['inspect', 'note', 'ask'][+k - 1]); return; }
  if (ctl && !e.shiftKey && (k === 'j' || k === 'J')) { e.preventDefault(); c._ctxTab('ask', true); return; }
  if (ctl && !e.shiftKey && (k === 'f' || k === 'F' || k === 'k' || k === 'K')) { e.preventDefault(); c.setState({ srch: true, q: k.toLowerCase() === 'k' ? '. ' : '', scope: 'all', dd: null }); return; }
  if (typing) return;   // below here, nothing acts while a text field has focus: Alt+arrows included (Ruling DX, PQ12)
  if (e.altKey && (k === 'ArrowLeft' || k === 'ArrowRight')) { e.preventDefault(); if (k === 'ArrowLeft') c._selBack(); else c._selFwd(); return; }
  if (ctl && !e.shiftKey && (k === 'd' || k === 'D')) { e.preventDefault(); c._deselect(); return; }
  if (ctl && !e.shiftKey && (k === 'r' || k === 'R')) { e.preventDefault(); c._run(1); return; }
  if (ctl && (k === 'z' || k === 'Z')) { e.preventDefault(); if (e.shiftKey) c._redo(); else c._undo(); return; }
  if (ctl && (k === 'y' || k === 'Y')) { e.preventDefault(); c._redo(); return; }
  if (ctl && e.shiftKey && (k === 'F' || k === 'f')) { e.preventDefault(); doAct(c, { act: 'factionsHere' }); return; }
  if (ctl && e.shiftKey && (k === 'V' || k === 'v')) { e.preventDefault(); c._openWin('vault'); return; }
  if (ctl && e.shiftKey && (k === 'N' || k === 'n')) { e.preventDefault(); c._ctxTab('note'); return; }
  if (ctl && e.shiftKey && (k === 'T' || k === 't')) { e.preventDefault(); c.setState({ tlOpen: !s.tlOpen }); return; }
  if (!ctl && !e.altKey && (k === 'q' || k === 'Q') && c._lastPtr) { e.preventDefault(); const p = c._lastPtr; c._longPress(p.x, p.y, true, true); return; }
  if (!ctl && k === 'Delete' && s.sel) { e.preventDefault(); doAct(c, { act: 'delete' }); }
}
