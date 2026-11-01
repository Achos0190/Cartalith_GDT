/* ======================================================================================================
   Screens that are not a dock category: MAP (view), STORY, the sample card, the selection inspector,
   the armed tool's options, the desktop windows, and search.
   ====================================================================================================== */
function quickRow(c) {
  return { k: 'acts', opts: quickItems(c).map((q) => ({ label: q.label, d: q.d, tap: q.run, flex: '0 0 auto' })).concat([{ label: 'Edit', d: IC.star, tap: () => c._openWin('quickbar'), flex: '0 0 auto' }]) };
}
/* ---------- the 3D view's live height-exaggeration slider (Ruling EA 14; Ruling EB items 5 and 6) ----------
   What it is: one preference, Preferences ▸ 3D viewport defaults ▸ relief exaggeration, which every slider for it
   writes: the phone's row in MAP ▸ View and the HEIGHT row of the 3D card on the tablet and the desktop.
   Why: Ruling EB item 5 makes the last value set the next session's default, as if set in that preference, so the
   slider writes the preference itself rather than a session value. Item 6 sets the range to 1× (true scale, Ruling CT,
   view3d.gd's VERTICAL_EXAGGERATION_DEFAULT) to 4×; 3D_VIEWPORT_SCOPE.md §8.3 saw spires at 3.4×, the owner accepted
   that cost, and the ceiling is judged again at 3D-3 on frames. Steps of 0.25 are the approved design's.
   What it must never do: touch the world, its save or WorldState (it writes state.prefs and this browser's storage
   only), or treat a missing or unreadable stored value as anything but the real default, 1×. Storage can be absent
   or throw (a private window, a preview), so every read and write is guarded and the page works without it. */
const V3_EXAG_MIN = 1, V3_EXAG_MAX = 4, V3_EXAG_STEP = 0.25, V3_EXAG_DEFAULT = 1;
const V3_EXAG_KEY = 'cartalith.prefs.v3exag';
const V3_EXAG_WHERE = 'Remembered in Preferences ▸ 3D viewport defaults for the next session · never saved with the world';
/* Reads the stored preference once, at boot: a finite value inside the range, or the default. */
function v3ExagStored() {
  let v = V3_EXAG_DEFAULT;
  try { const x = parseFloat(localStorage.getItem(V3_EXAG_KEY)); if (Number.isFinite(x) && x >= V3_EXAG_MIN && x <= V3_EXAG_MAX) v = x; } catch (e) { v = V3_EXAG_DEFAULT; }
  return v;
}
/* Sets the preference from a slider: clamped to the range, ignored if not a number, written through to storage. */
function v3ExagSet(c, v) {
  const x = Math.max(V3_EXAG_MIN, Math.min(V3_EXAG_MAX, +v));
  if (!Number.isFinite(x)) return;
  c.setState({ prefs: Object.assign({}, c.state.prefs, { v3exag: x }) });
  try { localStorage.setItem(V3_EXAG_KEY, String(x)); } catch (e) { }
}
/* The phone's MAP ▸ View row: R()'s shape, reading and writing the preference instead of a session value. */
function v3ExagRow(c) {
  const v = c.state.prefs.v3exag;
  return { k: 'rng', label: 'Height exaggeration (3D)', min: V3_EXAG_MIN, max: V3_EXAG_MAX, step: V3_EXAG_STEP, val: v, disp: fmtVal(v, { step: V3_EXAG_STEP, unit: '×' }), sub: V3_EXAG_WHERE, onIn: (e) => v3ExagSet(c, e.target.value), dec: () => v3ExagSet(c, v - V3_EXAG_STEP), inc: () => v3ExagSet(c, v + V3_EXAG_STEP) };
}
function rowsMap(c) {
  const s = c.state; const fid = s.field || 'off';
  const rows = [H('VIEW'), { k: 'seg', label: '', opts: [{ label: '2D', on: !s.dim3, tap: () => setK(c, 'dim3', false) }, { label: '3D', on: s.dim3, tap: () => setK(c, 'dim3', true) }, { label: 'North up', d: IC.locate || '', on: false, tap: () => { c.view = Object.assign({}, c.view, { r: 0 }); c._dirty = true; c.forceUpdate(); } }, { label: 'Fit world', on: false, tap: () => c._fit(true) }] }].concat(s.dim3 ? [v3ExagRow(c), info('1× is true scale (Ruling CT), up to 4× (Ruling EB). Live: the 3D view redraws as you drag. Mock: the stand-in map stays 2D.')] : []).concat([T(c, 'v.grid', 'Grid', false, 'graticule every 256 km', { after: (v) => { c.setState({ layers: Object.assign({}, s.layers, { grid: v }) }); c._dirty = true; } }), T(c, 'v.scale', 'Scale bar', true)]);
  rows.push(H('VIEW FIELD', 'what the map shows · keys 1–8'));
  ANALYSIS.forEach(([grp, list]) => rows.push({ k: 'seg', label: grp, opts: list.map(([id, l]) => ({ label: l, on: fid === id, tap: () => pickField(c, id, l) })) }));
  if (s.overlay) rows.push({ k: 'rng', label: 'Field opacity', min: 0.1, max: 1, step: 0.05, val: s.ovA, disp: Math.round(s.ovA * 100) + ' %', onIn: (e) => { c.setState({ ovA: +e.target.value }); c._dirty = true; }, dec: () => c.setState({ ovA: Math.max(0.1, +(s.ovA - 0.05).toFixed(2)) }), inc: () => c.setState({ ovA: Math.min(1, +(s.ovA + 0.05).toFixed(2)) }) });
  const L = s.layers; const lay = (k, label) => ({ k: 'layer', label, on: !!L[k], tap: () => { c.setState({ layers: Object.assign({}, L, { [k]: !L[k] }) }); c._dirty = true; } });
  rows.push(H('SHOW', 'quick visibility · styling is CARTO'), lay('settlements', 'Settlements'), lay('ways', 'Ways & routes'), lay('landmarks', 'Landmarks'), lay('rivers', 'Rivers'), lay('labels', 'Labels'), lay('political', 'Political — territory'), lay('hillshade', 'Hillshade'));
  rows.push(H('MEASURE', 'arms Measure (M)'), { k: 'tools', label: 'measure', cell: 92, tools: MEASURE_MODES.map((m) => ({ d: m === 'Area' ? 'M3 3h10v10H3Z' : (m === 'Radius' ? 'M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11ZM8 8h5.5' : (m === 'Bearing' ? 'M8 1.5v13M8 1.5l3 3M8 8l4-3' : (m === 'Cross-section' ? 'M1.5 12l3-5 3 3 3-6 4 8' : (m === 'Δ vertical' ? 'M8 2v12M5 5l3-3 3 3M5 11l3 3 3-3' : IC.measure)))), label: m, on: s.tool === 'measure' && V(c, 'ms.mode', 'Distance') === m, tap: () => { setV(c, 'ms.mode', m); if (s.tool !== 'measure') c._arm('measure'); } })) });
  rows.push(info('Long-press anywhere samples the ground. Swipe in from the right edge for the inspector. Nothing in MAP changes the world.'));
  return rows;
}
function timelineRows(c) {
  const s = c.state;
  return [H('TRANSPORT', 'year ' + s.year), BB({ label: s.playing ? 'Pause' : 'Play', variant: 'primary', d: s.playing ? IC.pause : IC.play, tap: () => c._play() }, { label: '◀ Step', tap: () => c.setState({ year: Math.max(-400, s.year - s.speed) }) }, { label: 'Step ▶', tap: () => c.setState({ year: Math.min(1200, s.year + s.speed) }) }), { k: 'seg', label: 'Speed', opts: [1, 10, 100].map((x) => ({ label: '×' + x, on: s.speed === x, tap: () => c.setState({ speed: x }) })) }, { k: 'rng', label: 'Year', min: -400, max: 1200, step: 1, val: s.year, disp: 'YEAR ' + s.year, onIn: (e) => c.setState({ year: +e.target.value }), dec: () => c.setState({ year: Math.max(-400, s.year - 1) }), inc: () => c.setState({ year: Math.min(1200, s.year + 1) }), sub: c.places.length + ' present · +2 / −1 since ' + (s.year - 10) }, R(c, 'tl.day', 'Date (day in year)', 1, 365, 1, 172), H('SIMULATION LAYERS', 'recorded, not yet rendered')].concat(['Climate', 'Population', 'Economy', 'Politics', 'Infrastructure', 'Warfare'].map((x, i) => T(c, 'tl.l.' + x, x, [true, true, false, true, false, false][i])), [T(c, 'tl.strip', 'Transport strip on the map', false, 'a compact player above the bar'), H('YEARS', '3 recorded'), { k: 'seg', label: '', opts: [0, 210, 412].map((y) => ({ label: 'YEAR ' + y, on: s.year === y, tap: () => c.setState({ year: y }) })) }, BB({ label: 'Add year ' + s.year, tap: () => c._toast('Year ' + s.year + ' recorded') }), H('FILTERS'), T(c, 'tl.exist', 'Exist only', false), T(c, 'tl.ghost', 'Ghost removed', false), T(c, 'tl.new', 'Highlight new', false)],
    fold(c, 'f-collapse', 'Simulate collapse / recovery', 'mode · severity · span', () => { const mode = V(c, 'sim.mode', 'Collapse'); return [S(c, 'sim.mode', 'Mode', ['Collapse', 'Recovery'], 'Collapse')].concat(mode === 'Collapse' ? [S(c, 'sim.char', 'Character', ['Mixed', 'Trade', 'Disease', 'Conflict'], 'Mixed', 'trade: hubs fall hardest · disease: dense places first · conflict: undefended first'), R(c, 'sim.sev', 'Severity', 0, 100, 1, 50, '%')] : [R(c, 'sim.regrow', 'Regrowth rate', 0.1, 3, 0.1, 1, '%/yr')], [R(c, 'sim.start', 'Start year', -400, 1200, 1, 0), R(c, 'sim.dur', 'Duration', 1, 1000, 1, 100, 'yr'), R(c, 'sim.step', 'Step', 1, 100, 1, 10, 'yr'), B('Simulate', () => c._toast('Simulated 100 years · 10 snapshots'), 'primary')]); }),
    [H('HISTORY'), { k: 'nav', d: IC.time, label: 'Political history of this place…', sub: hasSel(c) ? '' : '', dis: !hasSel(c), reason: 'Select a settlement on the map first', tap: hasSel(c) ? () => c._openWin('place:history') : () => c._toast('Select a settlement on the map first') }, { k: 'nav', d: IC.faction, label: 'Faction history…', chev: true, tap: () => c._openWin('factions:history') }]);
}
function routeStages(c) {
  const r = c.route, w = c.world; const out = [];
  for (let i = 0; i + 1 < r.length; i++) { const a = r[i], b = r[i + 1]; const km = Math.hypot(a.x - b.x, a.y - b.y) * 1.18; const asc = Math.max(0, Math.round((b.elev || 0) - (a.elev || 0))) + 40 * ((i * 7) % 5); const water = i === r.length - 2; const days = km / (water ? 70 : (asc > 600 ? 17 : 24)); out.push({ a, b, km, asc, water, days, terr: water ? 'water' : (asc > 600 ? 'mountain' : (asc > 200 ? 'hills' : 'plains')), biome: b.biome || '—' }); }
  return out;
}
function plannerRows(c, tablet) {
  const s = c.state; const st = routeStages(c); const total = st.reduce((a, x) => a + x.days, 0); const km = st.reduce((a, x) => a + x.km, 0);
  if (s.planStage >= 0 && st[s.planStage]) { const x = st[s.planStage]; return [H('STAGE ' + (s.planStage + 1) + ' OF ' + st.length, x.a.name + ' → ' + x.b.name), read('Terrain', x.terr), read('Biome', x.biome), read('Distance', fmtDist(x.km, s.prefs.units)), read('Ascent', x.asc + ' m'), read('Pace', x.water ? 'boat · 70 km/d' : (x.terr === 'mountain' ? 'pack · 17 km/d' : 'march · 24 km/d')), read('Days', x.days.toFixed(1)), read('Water crossing', x.water ? 'yes · ferry at ' + x.a.name : 'no', x.water ? 'var(--water)' : ''), S(c, 'pl.ov.' + s.planStage, 'Override pace', ['auto', 'forced march', 'slow'], 'auto'), BB({ label: '‹ Route', tap: () => c.setState({ planStage: -1 }) }, { label: 'Next stage ›', variant: 'primary', tap: () => c.setState({ planStage: Math.min(st.length - 1, s.planStage + 1) }) })]; }
  const feasible = total < 75; const stale = (s.staleLegs || []).filter((x) => st[x.i]);
  const staleRows = stale.length ? [info(stale.length + ' leg change' + (stale.length > 1 ? 's' : '') + ' no longer fit' + (stale.length > 1 ? '' : 's') + ' this world and ' + (stale.length > 1 ? 'were' : 'was') + ' dropped: ' + stale.map((x) => 'stage ' + (x.i + 1) + ', ' + st[x.i].a.name + ' → ' + st[x.i].b.name + ' (' + x.what + ')').join('; ') + '. The legs were rebuilt after the world changed; nothing was applied to a different leg.', true), BB({ label: 'Show stage ' + (stale[0].i + 1), tap: () => c.setState({ planStage: stale[0].i }) }, { label: 'Dismiss', tap: () => c.setState({ staleLegs: null }) })] : [];
  const rows = staleRows.concat(plannerSelRows(c)).concat([{ k: 'card', badge: feasible ? 'FEASIBLE · STRAINED' : 'INFEASIBLE', badgeCol: feasible ? 'var(--acc)' : 'var(--block)', right: 'confidence ± 4 d', big: String(Math.round(total + 7)), unit: 'calendar days', sub: total.toFixed(1) + ' travel · 4 rest · 3 layover · ' + fmtDist(km, s.prefs.units), note: 'Supply gap ' + (c.route[2] ? c.route[2].name : '') + ' → ' + (c.route[5] ? c.route[5].name : '') + ' runs 15.4 effective days but only 10 are carried. Add supplies, forage harder, or plan a resupply layover.', opts: [{ label: 'SUPPLIES +6 D', tap: () => setV(c, 'pl.sup', 16) }, { label: 'FORAGING ACTIVE', on: V(c, 'pl.forage', 'Active') === 'Active', tap: () => setV(c, 'pl.forage', 'Active') }] }]);
  rows.push(H('ROUTE', (c.route[0] ? c.route[0].name : '') + ' → ' + (c.route[c.route.length - 1] ? c.route[c.route.length - 1].name : '') + ' · ' + st.length + ' stages'));
  st.forEach((x, i) => rows.push({ k: 'item', label: x.a.name + ' → ' + x.b.name, sub: x.terr + ' · ' + fmtDist(x.km, s.prefs.units) + (x.asc ? ' · +' + x.asc + ' m' : ''), right: x.days.toFixed(1) + ' d', dot: x.water ? 'var(--water)' : 'var(--acc)', tap: () => c.setState({ planStage: i }) }));
  rows.push(BB({ label: 'Show on map', d: IC.map, tap: () => { c.setState({ det: 'peek' }); c._fit(true); } }, { label: 'Save journey', variant: 'primary', tap: () => c._toast('Journey saved · ' + (c.route[0] ? c.route[0].name : '')) }));
  rows.push(...fold(c, 'f-party', 'Party · Traveler', V(c, 'pl.size', 24) + ' people · ' + V(c, 'pl.pace', 'Normal'), () => [R(c, 'pl.size', 'Group size', 1, 100000, 1, 24, 'people'), S(c, 'pl.pace', 'Pace', ['Leisurely', 'Normal', 'Forced'], 'Normal'), R(c, 'pl.hours', 'Hours/day (land)', 1, 16, 0.5, 8), R(c, 'pl.cargo', 'Trade cargo', 0, 500000, 50, 400, 'kg'), R(c, 'pl.sup', 'Supplies carried', 1, 90, 1, 10, 'd'), T(c, 'pl.food', 'Carry food', true, 'off = live off the land'), S(c, 'pl.graze', 'Grazing', ['None', 'Light', 'Good'], 'Light'), S(c, 'pl.forage', 'Foraging', ['None', 'Active', 'Heavy'], 'Active')]));
  rows.push(...fold(c, 'f-season', 'Season & weather', V(c, 'pl.season', 'Summer') + ' · ' + V(c, 'pl.weather', 'Fair'), () => [S(c, 'pl.season', 'Season', ['Spring', 'Summer', 'Autumn', 'Winter'], 'Summer'), S(c, 'pl.weather', 'Weather', ['Fair', 'Mixed', 'Foul'], 'Fair'), S(c, 'pl.rest', 'Rest days', ['None', '1 in 7', '1 in 5'], '1 in 7'), T(c, 'pl.advance', 'Season advances during the journey', true)]));
  rows.push(...fold(c, 'f-carriage', 'Carriage', V(c, 'pl.trans', 'Walking'), () => [S(c, 'pl.trans', 'Transport', ['Walking', 'Riding', 'Baggage train', 'Wagon', 'River boat'], 'Walking'), { k: 'field', label: 'Mount or vessel', val: V(c, 'pl.mount', 'mule'), onIn: (e) => setV(c, 'pl.mount', e.target.value) }, T(c, 'pl.repack', 'Re-pack per stage where it pays', true), T(c, 'pl.promote', 'Auto-promote Walking → Baggage Train if overloaded', true), J(c, 'Animal definitions → Travel library', 'win:travel')]));
  rows.push(...fold(c, 'f-cond', 'Route conditions', V(c, 'pl.road', 'Mixed') + ' roads', () => [S(c, 'pl.road', 'Road quality', ['Poor', 'Mixed', 'Good'], 'Mixed'), S(c, 'pl.infra', 'Infrastructure', ['Sparse', 'Typical', 'Dense'], 'Typical'), S(c, 'pl.water', 'Desert water', ['Scarce', 'Wells', 'Plenty'], 'Wells'), T(c, 'pl.closures', 'Respect seasonal closures (winter passes)', true)]));
  rows.push(...fold(c, 'f-results', 'Results', 'totals · time · load · supply · cost', () => [read('Route totals', fmtDist(km, s.prefs.units) + ' · ' + st.length + ' stages'), read('Time', total.toFixed(1) + ' d travel'), read('Load', '1 840 kg · 92 % of capacity'), read('Supply reach', '10 d of 15.4 needed', 'var(--warn)'), read('Resupply stops', c.route[3] ? c.route[3].name : '—'), read('Cost', '412 silver'), read('Vessels · water legs', '1 ferry'), J(c, 'Calculation trace', 'win:trace')]));
  rows.push(H('LIBRARY'), J(c, 'Saved journeys…', 'win:journeys'), J(c, 'Travel library…', 'win:travel', 'animals · vehicles · vessels · parties'));
  if (tablet) rows.push(BB({ label: 'Export table', tap: () => c._toast('Journey table exported (CSV)') }, { label: 'Re-route…', tap: () => c._toast('Re-routed for ' + V(c, 'pl.trans', 'Walking')) }));
  return rows;
}
function rowsStory(c, segArg) {
  const s = c.state, seg = segArg || s.storySeg;
  if (seg === 'time') return timelineRows(c);
  if (seg === 'journeys') return plannerRows(c, false);
  if (seg === 'conflicts') return [read('Campaigns in year ' + s.year, '2'), H('CONFLICTS'), { k: 'item', label: 'Sevjuni campaign', sub: 'front · yr 398– · Sevjuni League vs Orentamwash', right: '2 sieges', dot: '#c96a5a', tap: () => c._openWin('conflict') }, { k: 'item', label: 'Korrath raids', sub: 'arrows · yr 405– · Korrath vs Mirzafir Reach', right: '3 raids', dot: '#8fae7d', tap: () => c._openWin('conflict') }, B('Draw a conflict', () => doAct(c, { act: 'tool', arg: 'conflict' }), 'primary', IC.conflict), info('Pick a kind in the tool strip: Front, Arrow, Siege or Battle. Commit, then fill it in.'), J(c, 'Conflicts…', 'win:conflicts'), J(c, 'Manpower → CIVIL ▸ Factions ▸ Military', 'civil/factions/Military')];
  return [H('THIS PLACE'), { k: 'nav', d: IC.notes, label: hasSel(c) ? 'Notes for ' + selName(c) : 'Notes for this place', sub: hasSel(c) ? 'open or attach a vault note' : 'select a settlement first', dis: !hasSel(c), reason: 'Select a settlement on the map first', tap: hasSel(c) ? () => c._openWin('vault:place') : () => c._toast('Select a settlement on the map first') }, H('VAULT'), J(c, 'Places without notes…', 'win:vault:without', (c.places.length - 3) + ' of ' + c.places.length + ' have none'), J(c, 'Markdown vault…', 'win:vault', 'link and attach notes · missing and orphan notes'), J(c, 'Browse & edit a note…', 'win:vault:browse'), H('ASSISTANT', 'optional · off by default'), J(c, 'Assistant panel…', 'win:assistant', 'prose for a place, a story, a vault note')];
}
function selName(c) { const p = selObj(c); return p ? p.name : ''; }
/* The selected settlement or landmark. A window pinned to something else reads that instead (c._selOverride,
   set only while its rows are built), so a pinned Place editor stays on its place while the map moves on. */
function selObj(c) { const s = c.state; const sel = c._selOverride || s.sel; if (!sel) return null; if (sel.kind === 'place') return c.places.concat(s.userPlaces).find((p) => p.id === sel.id); if (sel.kind === 'landmark') return c.landmarks.find((l) => l.id === sel.id); return null; }
function sampleAt(c, p) {
  const w = c.world; if (!w || !p) return null; const N = w.N; const i = Math.max(0, Math.min(N - 1, Math.floor(p.x / w.cell))), j = Math.max(0, Math.min(N - 1, Math.floor(p.y / w.cell))); const k = j * N + i;
  let near = null, nd = 1e9; for (const pl of c.places) { const d = Math.hypot(pl.x - p.x, pl.y - p.y); if (d < nd) { nd = d; near = pl; } }
  const own = c.ownerGrid ? c.ownerGrid[k] : -1;
  return { k, elev: Math.round(w.elevM[k]), slope: w.slope[k].toFixed(1), temp: w.temp[k].toFixed(1), rain: Math.round(w.rain[k]), biome: biomeOf(w, k), flow: Math.round(w.acc[k]), near, nd, own, lith: ['Granite', 'Sandstone', 'Limestone', 'Schist', 'Andesite'][k % 5], plate: 'P' + (3 + (Math.floor(i / 70) + Math.floor(j / 90)) % 9) };
}
function domainVerbs(c, p) {
  const s = c.state, d = s.dom; const go = (fn, label) => ({ label, tap: fn });
  const common = [go(() => { c.view = Object.assign({}, c.view, { cx: p.x, cy: p.y }); c._dirty = true; c.forceUpdate(); }, 'Centre here'), go(() => { c._arm('measure'); c.setState({ measure: [p] }); }, 'Measure from here')];
  if (d === 'world') return [go(() => { c.setState({ feature: 'Mountains' }); c._arm('uplift'); }, 'Sculpt here'), go(() => { setV(c, 'ms.mode', 'Cross-section'); c._arm('measure'); c.setState({ measure: [p] }); }, 'Cross-section from here'), go(() => { c._arm('paint'); }, 'Paint with this biome')].concat(common);
  if (d === 'civil') return [go(() => { c._arm('settle'); TOOLS.settle.onTap(c, p); }, 'Drop settlement here'), go(() => { c._arm('way'); c.setState({ wayDraft: [p] }); }, 'Start way here'), go(() => { c._arm('route'); c.setState({ wayDraft: [p] }); }, 'Start route here'), go(() => c._toast('Info here · settlement & ecology (see the readouts)'), 'Info here')].concat(common);
  return [go(() => { c._arm('label'); c.setState({ labelAt: p }); }, 'Add label here…'), go(() => { c._arm('icon'); TOOLS.icon.onTap(c, p); }, 'Stamp icon here'), go(() => { if (c._mode() === 'tablet') c.setState({ layersPop: true, tctx: null }); else c.setState({ tab: 'map', det: 'half' }); }, 'View field ▸'), go(() => jump(c, 'carto/style/Style'), 'Style preset ▸'), go(() => c._openWin('export:image'), 'Export region from here')].concat(common);
}
function rowsSample(c) {
  const s = c.state, p = s.sample; const a = sampleAt(c, p); if (!a) return [info('Long-press the map to sample.')];
  const verbs = domainVerbs(c, p);
  const rows = [H(domainDef(s.dom).label + ' · ACTIONS', 'follow the domain'), { k: 'acts', opts: verbs.map((v, i) => ({ label: v.label, tap: v.tap, flex: '1 1 40%', variant: i === 0 ? 'primary' : '' })) }];
  rows.push(H('HERE'), read('Elevation', a.elev + ' m', 'var(--acc)'), read('Slope', a.slope + '°'), read('Plate', a.plate + ' · continental'), read('Lithology', a.lith), read('Temperature', a.temp + ' °C'), read('Precipitation', a.rain + ' mm/yr'), read('Drainage', a.flow > 200 ? 'river · ' + a.flow + ' cells upstream' : 'hillslope'), read('Biome', a.biome), read('Control', a.own >= 0 ? FACTIONS[a.own].name : 'unclaimed'), read('Nearest', a.near ? a.near.name + ' · ' + fmtDist(a.nd, s.prefs.units) : '—'), read('Position', Math.round(p.x) + ' km E · ' + Math.round(p.y) + ' km S'));
  rows.push(BB({ label: 'Copy coordinate', d: IC.copy, tap: () => c._toast('Copied ' + Math.round(p.x) + ' km E · ' + Math.round(p.y) + ' km S') }, { label: 'Clear pin', tap: () => { c.setState({ sample: null, tab: c._mode() === 'tablet' ? s.tab : null, tctx: null }); c._dirty = true; } }));
  return rows;
}
function rowsSel(c) {
  const s = c.state; const o = selObj(c); if (!o) { const e = entOf(c, s.sel); return e ? ctxInspectRows(c, e) : rowsSample(c); }
  if (s.sel.kind === 'landmark') return [read('Kind', o.kind), read('Family', o.fam), read('Elevation', o.z ? o.z + ' m' : '—'), read('Position', Math.round(o.x) + ' km E · ' + Math.round(o.y) + ' km S'), H('WHY HERE?'), info(o.kind === 'Peak' ? 'Highest ground within 32 km, prominence over 600 m and visible from three settlements.' : 'Scored on the site rules for ' + o.kind.toLowerCase() + ', then spaced by class.'), BB({ label: 'Inspect ' + o.kind, tap: () => c._toast(o.kind + ' · full card on desktop') }, { label: 'Hide type', tap: () => c._toast(o.kind + ' hidden · CARTO ▸ Layers ▸ Landmarks by type') })];
  const fac = o.fac >= 0 ? FACTIONS[o.fac] : null; const verbs = [{ label: 'Edit ' + o.name, variant: 'primary', tap: () => c._openWin('place') }, { label: 'City viewer…', tap: () => c._openWin('city') }, { label: hasNote(o, c) ? 'Open note' : 'Attach note…', tap: () => c._openWin('vault:place') }, { label: 'Journey from here', tap: () => doAct(c, { act: 'story', arg: 'journeys' }) }];
  if (o.user) verbs.push({ label: 'Delete', variant: 'danger', tap: () => doAct(c, { act: 'delete' }) });
  return [{ k: 'acts', opts: verbs.map((v) => Object.assign({ flex: '1 1 40%' }, v)) }, H('SETTLEMENT'), read('Class', o.tier + (o.capital ? ' · capital' : '')), read('Population', o.pop ? o.pop.toLocaleString('en-US') : 'new · grows on recompute'), read('Faction', fac ? fac.name : 'unclaimed', fac ? fac.col : ''), read('Government', fac ? fac.gov : '—'), read('Faith', fac ? fac.rel : '—'), read('Founded', 'year ' + o.founded), read('Elevation', (o.elev || 0) + ' m'), read('Biome', o.biome || '—'), H('ECONOMY · POLITICS · LOGISTICS'), read('Trade', 'grain · salt'), read('Garrison', o.capital ? '1 200' : '80'), read('Roads', c.ways.filter((w) => w.a === o || w.b === o).length + ' ways'), J(c, 'Political history…', 'win:place:history'), J(c, 'Faction: ' + (fac ? fac.name : '—'), 'win:factions')];
}
function hasNote(o, c) { return !!(o && c && noteByName(c, o.name)); }

/* ---------- DS-13: the map's control cluster on touch (phone and tablet), MOBILE_SHELL_PROPOSAL.md §2.5 ----------
   One row at the bottom of the map on the dominant-hand side (Preferences ▸ Handedness), outer end first: recentre (⌖),
   the zoom rocker (− | +), and the pan latch (✋) at the inner end, shown only while a drawing tool is armed, so its
   coming and going moves nothing else. A tap-only tool (Inspect, Measure, Settlement, Label, Icon …) already gives a
   one-finger drag to the camera, which is what the approved gesture table says; the latch is for the tools that draw.
   The rocker steps by the shipped ZOOM_BUTTON_STEP, 1.35 (viewport_host.gd, reference 13464), about the map's centre.
   Ruling EB item 2 (DS13-Q5) keeps that step. It is the mockup's only button zoom step: the desktop pointer column
   (d_vals.js hZoomIn / hZoomOut) uses this constant too, where it used to step by 1.6.
   Ruling EB item 4 (DS13-Q4): from 3D-2 the same −|+ and ⌖ drive the 3D camera (zoom moves it in and out, ⌖ frames
   the world). The mockup's 3D is a label over the 2D stand-in, so here they still act on that map.
   o: { bottom (px from the map host's bottom), edge (px from the dominant side), show } */
const MC_ZOOM_STEP = 1.35;
function mapCtlVals(c, o) {
  const s = c.state; const left = s.prefs.hand === 'left';
  const draws = c._strokeTool() && !s.prefs.penOnly;
  const on = draws && !!s.panLatch;
  const px = (n) => n + 'px';
  return {
    disp: o.show ? 'flex' : 'none', left: left ? px(o.edge) : 'auto', right: left ? 'auto' : px(o.edge), bottom: o.bottom,
    dir: left ? 'row-reverse' : 'row', side: left ? 'left' : 'right',
    panShow: draws, panOn: on ? 'true' : 'false',
    panAria: on ? 'pan latch on: one finger pans; tap to give it back to the tool' : 'pan latch: one finger pans the map while the tool stays armed',
    panBg: on ? 'linear-gradient(var(--navarm),var(--navarm)),var(--navpill)' : 'var(--navpill)', panBorder: on ? 'var(--navacc)' : 'var(--hair)', panCol: on ? 'var(--navacc)' : 'var(--body)',
    panTap: () => { const v = !s.panLatch; c.setState({ panLatch: v }); if (v) c._toast('One finger pans — tap ✋ again to give it back to the tool'); },
    zoomIn: () => { const { W: w, H: h } = c._hostWH(); c._zoomAnim(MC_ZOOM_STEP, w / 2, h / 2); },
    zoomOut: () => { const { W: w, H: h } = c._hostWH(); c._zoomAnim(1 / MC_ZOOM_STEP, w / 2, h / 2); },
    fit: () => { c.setState({ panLatch: false }); c._fit(true); }
  };
}
/* ---------- the armed tool: one description for the phone strip and the tablet tool-options bar ---------- */
function toolBar(c) {
  const s = c.state, t = s.tool, T0 = TOOLS[t] || TOOLS.inspect; const acts = []; let main = T0.label, hint = T0.hint || '', chips = [], field = false;
  const commitDiscard = (n, commit, discard) => { acts.push({ label: 'DISCARD', tap: discard, bg: 'var(--chip)', col: 'var(--sec)', aria: 'discard' }, { label: 'COMMIT', tap: commit, bg: 'var(--acc)', col: 'var(--accInk)', aria: 'commit' }); };
  if (t === 'inspect') {
    main = { world: 'GENERATE · WORLD', civil: 'CIVIL · INSPECT', carto: 'CARTOGRAPHY · ' + nodeDef('carto', s.node.carto).short }[s.dom];
    hint = c._ptr() ? { world: 'Click selects · right-click opens the card · Q opens the cartwheel', civil: 'click a settlement, route or landmark · Esc deselects', carto: 'presentation only — nothing here marks a stage stale' }[s.dom] : { world: 'Tap selects · hold opens the cartwheel and the card', civil: 'tap a settlement, route or landmark', carto: 'presentation only — nothing here marks a stage stale' }[s.dom];
    // Ruling DZ (owner, 2026-11-01): the tool-options bar holds the armed tool's settings only. WORLD's idle row is its
    // context name and one hint; Generate world, New seed, Center landmasses and Bake ALL & finalize live in the WORLD
    // dock (and Generate world also in File, Ctrl+R), so the bar carries no actions here.
  } else if (t === 'measure') {
    const mode = V(c, 'ms.mode', 'Distance'); let km = 0; for (let i = 1; i < s.measure.length; i++) km += Math.hypot(s.measure[i].x - s.measure[i - 1].x, s.measure[i].y - s.measure[i - 1].y);
    main = mode.toUpperCase() + ' · ' + (s.measure.length > 1 ? fmtDist(km, s.prefs.units) : '—'); hint = s.measure.length + ' points · tap the map to add';
    chips = MEASURE_MODES.map((m) => ({ label: m, on: m === mode, tap: () => setV(c, 'ms.mode', m) }));
    acts.push({ label: 'CLEAR', tap: () => c.setState({ measure: [] }), bg: 'var(--chip)', col: 'var(--sec)', aria: 'clear points' }, { label: 'DONE', tap: () => { if (s.measure.length > 1) c._toast('Measured ' + fmtDist(km, s.prefs.units) + ' along ' + (s.measure.length - 1) + ' segments'); c._arm('inspect'); }, bg: 'var(--acc)', col: 'var(--accInk)', aria: 'done' });
  } else if (t === 'region') { main = 'REGION SELECT'; hint = s.region ? 'box set · File ▸ New world from selection' : 'drag a box on the map'; acts.push({ label: 'CLEAR', tap: () => c.setState({ region: null }), bg: 'var(--chip)', col: 'var(--sec)', aria: 'clear region' });
  } else if (t === 'uplift' || t === 'carve' || t === 'freehand') {
    main = 'SCULPT · ' + s.feature.toUpperCase(); const n = s.stamps.filter((x) => x.kind !== 'paint').length; hint = n + ' stamps on the draft · size ' + V(c, 'sc.size', 32) + ' px';
    chips = (SCULPT_FEATURES[t] || []).map((f) => ({ label: f, on: (t === 'freehand' ? V(c, 'fh.mode', 'Raise') : s.feature) === f, tap: () => (t === 'freehand' ? setV(c, 'fh.mode', f) : c.setState({ feature: f })) }));
    commitDiscard(n, () => commitStamps(c), () => discardStamps(c));
  } else if (t === 'paint') { const tgt = V(c, 'pt.target', 'Biome'); main = 'PAINT · ' + tgt.toUpperCase(); hint = V(c, 'pt.val', (PAINT_PAL[tgt] || ['—'])[0]) + ' · radius ' + V(c, 'pt.rad', 6) + ' c'; chips = ['Biome', 'Terrain', 'Splat'].map((x) => ({ label: x, on: tgt === x, tap: () => setV(c, 'pt.target', x) })); commitDiscard(0, () => commitStamps(c, 'paint'), () => discardStamps(c, 'paint'));
  } else if (t === 'settle') { main = 'SETTLEMENT · ' + s.settleTier.toUpperCase(); hint = 'tap drops a place · faction ' + FACTIONS[V(c, 'st.fac', 0)].name; chips = ['metropolis', 'capital', 'city', 'town', 'village', 'hamlet'].map((x) => ({ label: x, on: s.settleTier === x, tap: () => c.setState({ settleTier: x }) }));
  } else if (t === 'territory' || t === 'lasso') { main = (t === 'lasso' ? 'TERRITORY LASSO · ' : 'TERRITORY · ') + FACTIONS[V(c, 'st.fac', 0)].name.toUpperCase(); hint = t === 'lasso' ? s.wayDraft.length + ' vertices · commit assigns the inside' : 'drag to claim · radius ' + V(c, 'tr.rad', 5) + ' c'; chips = FACTIONS.map((f, i) => ({ label: f.name, on: V(c, 'st.fac', 0) === i, tap: () => setV(c, 'st.fac', i) })).concat([{ label: V(c, 'tr.mode', 'Add') === 'Add' ? 'Mode: Add' : 'Mode: Subtract', on: false, tap: () => setV(c, 'tr.mode', V(c, 'tr.mode', 'Add') === 'Add' ? 'Subtract' : 'Add') }]); commitDiscard(0, () => { c.setState({ wayDraft: [] }); c._toast('Territory committed'); }, () => c.setState({ wayDraft: [] }));
  } else if (t === 'way' || t === 'route') {
    let km = 0; for (let i = 1; i < s.wayDraft.length; i++) km += Math.hypot(s.wayDraft[i].x - s.wayDraft[i - 1].x, s.wayDraft[i].y - s.wayDraft[i - 1].y);
    main = (t === 'way' ? 'WAY · ' + s.wayKind.toUpperCase() : 'ROUTE') + ' · ' + s.wayDraft.length + ' pts · ' + fmtDist(km, s.prefs.units); hint = t === 'way' ? 'tap appends waypoints · drag draws' : 'sea and river legs between ports';
    if (t === 'way') chips = ['road', 'track', 'sea lane', 'ancient'].map((x) => ({ label: x, on: s.wayKind === x, tap: () => c.setState({ wayKind: x }) })).concat([{ label: 'Mode: ' + V(c, 'wy.mode', 'Routed'), tap: () => setV(c, 'wy.mode', V(c, 'wy.mode', 'Routed') === 'Routed' ? 'Freehand' : 'Routed') }, { label: 'Smooth', on: !!V(c, 'wy.smooth', true), tap: () => setV(c, 'wy.smooth', !V(c, 'wy.smooth', true)) }]);
    commitDiscard(s.wayDraft.length, () => { if (s.wayDraft.length < 2) { c._toast('Add at least two points'); return; } const wy = { pts: s.wayDraft, kind: t === 'route' ? 'sea lane' : s.wayKind, km: Math.round(km) }; c.setState({ wayDraft: [] }); c._addTo('userWays', wy, (t === 'way' ? 'way · ' : 'route · ') + fmtDist(km, s.prefs.units)); c._toast('Committed · ' + fmtDist(km, s.prefs.units) + ' — routes can use it now'); }, () => c.setState({ wayDraft: [] }));
  } else if (t === 'conflict') { main = 'CONFLICT · ' + V(c, 'cf.kind', 'Front').toUpperCase() + ' · ' + s.conflictDraft.length + ' pts'; hint = 'taps draw · commit, then fill it in'; chips = ['Front', 'Arrow', 'Siege', 'Battle'].map((x) => ({ label: x, on: V(c, 'cf.kind', 'Front') === x, tap: () => setV(c, 'cf.kind', x) })); commitDiscard(0, () => { c.setState({ conflictDraft: [] }); c._toast('Conflict committed · fill it in from the inspector'); }, () => c.setState({ conflictDraft: [] }));
  } else if (t === 'label') { main = s.labelAt ? 'LABEL' : 'LABEL · tap the map'; hint = s.labelAt ? 'type, then ADD' : 'tap empty ground, or a label to edit it'; field = !!s.labelAt; if (s.labelAt) { acts.push({ label: 'CANCEL', tap: () => c.setState({ labelAt: null, labelText: '' }), bg: 'var(--chip)', col: 'var(--sec)', aria: 'cancel label' }, { label: 'ADD', tap: () => { const txt = (s.labelText || '').trim(); if (!txt) { c.setState({ labelAt: null }); return; } c._addTo('userLabels', { x: s.labelAt.x, y: s.labelAt.y, text: txt }, 'label · ' + txt); c.setState({ labelAt: null, labelText: '' }); }, bg: 'var(--acc)', col: 'var(--accInk)', aria: 'add label' }); }
  } else if (t === 'icon') { main = 'ICON · ' + V(c, 'ic.tfam', 'Feature').toUpperCase() + ' · ' + s.iconKind; hint = 'tap stamps · scale ' + V(c, 'ic.tscale', 1) + '×'; chips = ['diamond', 'circle', 'triangle', 'square'].map((x) => ({ label: x, on: s.iconKind === x, tap: () => c.setState({ iconKind: x }) }));
  } else if (t === 'colourpaint') { main = 'COLOUR PAINT · ' + V(c, 'cp.mat', 'Colour').toUpperCase(); hint = V(c, 'cp.blend', 'Colour (tint)') + ' · radius ' + V(c, 'cp.rad', 1) + ' km'; chips = ['Colour', 'Lava', 'Embers', 'Ice glow', 'Toxic glow'].map((x) => ({ label: x, on: V(c, 'cp.mat', 'Colour') === x, tap: () => setV(c, 'cp.mat', x) })); acts.push({ label: 'DONE', tap: () => c._arm('inspect'), bg: 'var(--acc)', col: 'var(--accInk)', aria: 'done' }); }
  return { tool: t, d: T0.d, short: T0.short, label: T0.label, main, hint, chips, acts, field };
}
/* The armed tool's full options, at the top of the domain sheet when it is opened from the strip */
function toolOptRows(c) {
  const s = c.state, t = s.tool, tb = toolBar(c); const rows = [H('TOOL OPTIONS · ' + tb.label.toUpperCase(), tb.hint)];
  if (tb.chips.length) rows.push({ k: 'seg', label: '', opts: tb.chips.map((x) => ({ label: x.label, on: !!x.on, tap: x.tap })) });
  if (t === 'settle') rows.push(S(c, 'st.fac', 'Faction', FACTIONS.map((f, i) => [i, f.name]), 0), { k: 'field', label: 'Name', val: V(c, 'st.name', ''), ph: '(blank = generated)', onIn: (e) => setV(c, 'st.name', e.target.value) }, T(c, 'st.snap', 'Snap to water', false));
  if (t === 'territory') rows.push(R(c, 'tr.rad', 'Radius', 1, 20, 1, 5, 'c'));
  if (t === 'way') rows.push(...fold(c, 'f-wayadv', 'Road drag · advanced', 'waypoints · corridor · pull', () => [R(c, 'wy.wp', 'Waypoints', 1, 8, 1, 3), R(c, 'wy.cor', 'Corridor', 1, 16, 1, 6, 'c'), R(c, 'wy.pull', 'Pull', 0, 20, 1, 8)]));
  if (t === 'icon') rows.push(S(c, 'ic.tfam', 'Family', ['Settlement', 'Feature', 'POI'], 'Feature'), S(c, 'ic.var', 'Variant', ICON_FAMILIES[V(c, 'ic.tfam', 'Feature')], ICON_FAMILIES[V(c, 'ic.tfam', 'Feature')][0]), R(c, 'ic.tscale', 'Scale', 0.2, 4, 0.1, 1, '×'), R(c, 'ic.rot', 'Rotation', -180, 180, 1, 0, '°'), R(c, 'ic.jit', 'Jitter', 0, 1, 0.05, 0), T(c, 'ic.brush', 'Brush (drag paints a stand)', false), R(c, 'ic.brad', 'Radius', 2, 60, 1, 12, 'cells'), R(c, 'ic.dens', 'Density', 0.05, 2, 0.05, 0.6));
  if (t === 'uplift' || t === 'carve' || t === 'freehand') rows.push(R(c, 'sc.size', 'Brush size', 6, 200, 1, 32, 'px'), R(c, 'sc.int', 'Intensity', 0, 1.5, 0.01, 1, '×'), R(c, 'sc.hard', 'Hardness', 0, 1, 0.01, 0.5), T(c, 'sc.press', 'Pressure → strength', true));
  if (t === 'measure' && V(c, 'ms.mode', 'Distance') === 'Cross-section') rows.push(S(c, 'ms.chan', 'Channel', ['Elevation', 'Terrain', 'Climate', 'Hydrology', 'Geology'], 'Elevation'), R(c, 'ms.samples', 'Samples', 32, 1024, 32, 256), R(c, 'ms.vex', 'V. exag', 1, 6, 1, 2, '×'));
  rows.push(BB({ label: 'Back to Inspect', tap: () => c._arm('inspect') }));
  return rows;
}
