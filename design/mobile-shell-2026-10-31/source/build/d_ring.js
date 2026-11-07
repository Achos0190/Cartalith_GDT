/* ======================================================================================================
   The cartwheel (tablet) and the adaptable quick bar (tablet cluster, ring dock, phone side drawer).

   The ring is the shipped CM-3/CM-4 compass at its touch scale (`radial_ring.gd::_apply_touch_scale()`:
   ring radius max(96, 92 × 1.53) ≈ 141, slot 60 × 1.53 ≈ 92, dead zone ≈ 34, disc ≈ 205 dp). Its four
   cardinals are global and its four diagonals belong to the domain (`MAP_CONTEXT_SCOPE.md` §5.1).
   The quick pins are `CUSTOM_QUICK_BAR_SCOPE.md`'s one list drawn in several places: the tool bar's
   cluster (CQ-2), the ring dock (CQ-4), MENU's Quick section, and, new here, the phone's side drawer.
   ====================================================================================================== */
const RING = { r: 141, slot: 92, dead: 34, disc: 205, subR: 129, subSlot: 86, centre: 70, dockW: 86, dockH: 61, dockGap: 9 };
const RING_POS = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];
const IC_UNDO = 'M5.5 3.5L2.5 6.5l3 3M3 6.5h6.5a3.5 3.5 0 0 1 0 7H7';
const QD_W = 248;                 // the phone quick drawer's width, capped at 78 % of the window

function quickDefs(c) {
  return {
    save: ['Save', IC.save, () => doAct(c, { act: 'save' })],
    'export.player': ['Player map', IC.export, () => c._openWin('export:player')],
    'story.journeys': ['Planner', IC.journey, () => doAct(c, { act: 'story', arg: 'journeys' })],
    'world.vault': ['Vault', IC.vault, () => c._openWin('vault')],
    assistant: ['Assistant', IC.assistant, () => c._openWin('assistant')]
  };
}
/* The pins in drawing order. The Assistant chip comes first, and only while the assistant is on (CQ-7). */
function quickItems(c) {
  const s = c.state, Q = quickDefs(c);
  const ids = (s.prefs.assistant ? ['assistant'] : []).concat(s.quick.filter((q) => q !== 'assistant'));
  return ids.filter((q) => Q[q]).map((q) => ({ id: q, label: Q[q][0], d: Q[q][1], run: Q[q][2] }));
}
/* Arms a tool without the toggle `_arm` applies when the same tool is already armed. */
function armOnly(c, t) { if (c.state.tool !== t) c._arm(t); }
function ringViews() { return ANALYSIS.flatMap((g) => g[1]).filter((f) => f[0] !== 'off').slice(0, 8); }

/* The eight slots for the current domain: N Inspect, E Measure ▸, S Undo, W Region everywhere; the
   diagonals are the domain's four tool families. A slot with `sub` opens its sub-ring. */
function ringSlots(c) {
  const s = c.state;
  const sub = (list, fn) => list.map((x) => ({ label: Array.isArray(x) ? x[1] : x, run: () => fn(Array.isArray(x) ? x[0] : x) }));
  const card = {
    N: { label: 'Inspect', key: 'V', d: IC.inspect, run: () => armOnly(c, 'inspect') },
    E: { label: 'Measure', key: 'M', d: IC.measure, sub: sub(MEASURE_MODES, (m) => { setV(c, 'ms.mode', m); armOnly(c, 'measure'); }) },
    S: { label: 'Undo', key: 'Ctrl+Z', d: IC_UNDO, run: () => c._undo() },
    W: { label: 'Region', key: 'R', d: IC.region, run: () => armOnly(c, 'region') }
  };
  const diag = {
    world: {
      NW: { label: 'Uplift', d: IC.uplift, sub: sub(SCULPT_FEATURES.uplift, (f) => { c.setState({ feature: f }); armOnly(c, 'uplift'); }) },
      NE: { label: 'Carve & water', d: IC.carve, sub: sub(SCULPT_FEATURES.carve, (f) => { c.setState({ feature: f }); armOnly(c, 'carve'); }) },
      SE: { label: 'Freehand', d: IC.freehand, sub: sub(SCULPT_FEATURES.freehand, (f) => { setV(c, 'fh.mode', f); armOnly(c, 'freehand'); }) },
      SW: { label: 'Biome paint', d: IC.paint, sub: sub(['Biome', 'Terrain', 'Splat'], (x) => { setV(c, 'pt.target', x); armOnly(c, 'paint'); }) }
    },
    civil: {
      NW: { label: 'Settlement', d: IC.settle, sub: sub(['metropolis', 'city', 'town', 'village', 'hamlet'], (x) => { c.setState({ settleTier: x }); armOnly(c, 'settle'); }) },
      NE: { label: 'Territory', d: IC.territory, sub: sub([['add', 'Add'], ['subtract', 'Subtract']], (x) => { setV(c, 'tr.op', x); armOnly(c, 'territory'); }) },
      SE: { label: 'Way', d: IC.way, sub: sub(['road', 'track', 'sea lane', 'ancient'], (x) => { c.setState({ wayKind: x }); armOnly(c, 'way'); }) },
      SW: { label: 'Route', d: IC.route, run: () => armOnly(c, 'route') }
    },
    carto: {
      NW: { label: 'Label', d: IC.label, run: () => armOnly(c, 'label') },
      NE: { label: 'Icon', d: IC.icon, sub: sub(['Settlement', 'Feature', 'POI', 'Custom'], (x) => { setV(c, 'icon.fam', x); armOnly(c, 'icon'); }) },
      SE: { label: 'View field', d: IC.layers, sub: sub(ringViews(), (f) => { c.setState({ field: f }); c._dirty = true; }) },
      SW: { label: 'Style preset', d: IC.style, sub: sub(['Natural Vibrant', 'Default', 'Antique', 'Ink', 'Watercolor', 'Print'], (p) => applyPreset(c, p)) }
    }
  }[s.dom];
  return RING_POS.map((pos) => Object.assign({ pos }, card[pos] || diag[pos]));
}
/* The slot under a point, from its angle round the centre: nothing inside the dead zone. */
function ringSlotAt(dx, dy) {
  if (Math.hypot(dx, dy) < RING.dead) return null;
  const a = (Math.atan2(dx, -dy) * 180 / Math.PI + 360) % 360;
  return RING_POS[Math.round(a / 45) % 8];
}
/* Where the ring draws: centred on the finger, slid inward near an edge rather than clipped. */
function ringCentre(c, R) {
  const WW = c._W(), HH = c._H(), G = c._tabletGeom(), L = c._L(), half = RING.disc;
  const x0 = L.mapL + half, x1 = WW - L.mapR - half, y0 = G.bodyT + half, y1 = HH - G.bodyB - half - 96;
  return { x: x1 < x0 ? (L.mapL + WW - L.mapR) / 2 : Math.max(x0, Math.min(x1, R.x)), y: y1 < y0 ? (G.bodyT + HH - G.bodyB) / 2 : Math.max(y0, Math.min(y1, R.y)) };
}
function ringVals(c) {
  const s = c.state, R = s.ring; if (!R) return { show: false, items: [], dock: [] };
  const half = RING.disc, ctr = ringCentre(c, R), slots = ringSlots(c);
  const parent = R.sub ? slots.find((x) => x.pos === R.sub) : null;
  const place = (x, y, size) => ({ left: Math.round(half + x - size / 2), top: Math.round(half + y - size / 2), size });
  let items;
  if (parent) {
    const n = parent.sub.length;
    items = parent.sub.map((it, i) => { const a = i * 2 * Math.PI / n; return Object.assign(place(RING.subR * Math.sin(a), -RING.subR * Math.cos(a), RING.subSlot), { label: it.label, d: '', key: '', on: false, aria: parent.label + ' ▸ ' + it.label, tap: () => c._ringRun(it.run) }); });
  } else {
    items = slots.map((sl, i) => { const a = i * Math.PI / 4; return Object.assign(place(RING.r * Math.sin(a), -RING.r * Math.cos(a), RING.slot), { label: sl.label + (sl.sub ? ' ▸' : ''), d: sl.d, key: sl.key || '', on: R.hover === sl.pos, aria: sl.label + (sl.sub ? ', opens a sub-ring' : '') + (sl.key ? ' (' + sl.key + ')' : ''), tap: () => c._ringPick(sl.pos) }); });
  }
  items = items.map((it) => Object.assign(it, { bg: it.on ? 'var(--wash)' : 'var(--chip)', bord: it.on ? 'var(--acc)' : 'var(--hair)', col: it.on ? 'var(--acc)' : 'var(--body)', iconDisp: it.d ? 'block' : 'none' }));
  const hov = R.hover ? slots.find((x) => x.pos === R.hover) : null;
  const caption = parent ? parent.label.toUpperCase() + ' ▸ tap one' : (hov ? (hov.label + (hov.sub ? ' ▸' : '') + (hov.key ? ' · ' + hov.key : '')).toUpperCase() : (R.live ? 'SLIDE TO A SLOT · LIFT TO CHOOSE' : 'TAP A SLOT'));
  const dock = quickItems(c).slice(0, 4).map((q) => ({ label: q.label, d: q.d, aria: q.label + ' (pinned under the ring)', tap: () => { c._ringClose(); q.run(); } }));
  return {
    show: true, left: Math.round(ctr.x - half), top: Math.round(ctr.y - half), size: 2 * half, items, caption,
    centreSize: RING.centre, centreOff: Math.round(half - RING.centre / 2), centreLabel: parent ? '‹' : '✕', centreAria: parent ? 'back to the ring' : 'close the ring',
    hCentre: () => (parent ? c.setState({ ring: Object.assign({}, R, { sub: null }) }) : c._ringClose()),
    captionTop: 2 * half + 6, dock, dockShow: dock.length > 0, dockTop: 2 * half + 44, dockW: RING.dockW, dockH: RING.dockH, dockGap: RING.dockGap
  };
}

/* The tablet tool bar's quick cluster (CQ-2): the pins that fit, `⋯ n` for the rest, and `＋`. A wider
   window keeps more pins on the bar before the overflow takes them. */
function quickCluster(c, W) {
  const items = quickItems(c), cap = W >= 1600 ? 6 : (W >= 1180 ? 4 : 2);
  const shown = items.slice(0, cap), more = items.length - shown.length;
  const chips = shown.map((q) => ({ label: q.label, d: q.d, iconDisp: 'block', tap: q.run, aria: q.label + ' (pinned)' }));
  if (more > 0) chips.push({ label: '⋯ ' + more, d: '', iconDisp: 'none', tap: () => c._openWin('quickbar'), aria: more + ' more pinned commands' });
  chips.push({ label: '＋', d: '', iconDisp: 'none', tap: () => c._openWin('quickbar'), aria: 'pin a command to the quick bar' });
  return chips;
}

/* The phone's quick drawer: a grip on the side away from the dominant hand (the holding hand's thumb),
   tap or drag it to open; tap the scrim, drag the grip back, or press system back to close. It draws
   nothing while nothing is pinned and the assistant is off (CQ-6: an empty bar draws nothing). */
function quickDrawerVals(c, land, sh, stripH) {
  const s = c.state, WW = c._W(), HH = c._H(), items = quickItems(c);
  const side = s.prefs.hand === 'left' ? 'right' : 'left';
  const top = land ? 72 : INSET_T + 60, bottom = land ? HH - 8 : HH - NAV_H - sh - stripH - 8;
  const edge = land ? (side === 'left' ? RAIL_W : (s.tab ? c._sheetW() : 0)) : 0;
  const room = bottom - top;
  const show = items.length > 0 && room >= 180 && !s.win;
  const w = Math.min(QD_W, Math.round(WW * 0.78)), gripH = 88;
  const travel = s.qd ? w : (s.qdDrag != null ? Math.max(0, Math.min(w, s.qdDrag)) : 0);
  const gripTop = Math.round((top + bottom) / 2 - gripH / 2);
  const px = (n) => n + 'px';
  const rows = [H('QUICK', items.length + ' pinned')]
    .concat(items.map((q) => ({ k: 'nav', d: q.d, label: q.label, sub: q.id === 'assistant' ? 'shown while the assistant is on' : 'pinned command', tap: () => { c.setState({ qd: false, qdDrag: null }); q.run(); } })))
    .concat([{ k: 'nav', d: IC.star, label: 'Pin a command…', sub: 'Quick bar…', tap: () => { c.setState({ qd: false, qdDrag: null }); c._openWin('quickbar'); } }]);
  return {
    show, open: travel > 0, side, top, h: room, w, rows, gripTop, gripH,
    gripL: side === 'left' ? px(edge + travel) : 'auto', gripR: side === 'right' ? px(edge + travel) : 'auto',
    drawL: side === 'left' ? px(edge - w + travel) : 'auto', drawR: side === 'right' ? px(edge - w + travel) : 'auto',
    gripRadius: side === 'left' ? '0 14px 14px 0' : '14px 0 0 14px', gripJustify: side === 'left' ? 'flex-start' : 'flex-end', drawRadius: side === 'left' ? '0 18px 18px 0' : '18px 0 0 18px', gripAria: s.qd ? 'close the quick drawer' : 'open the quick drawer: pinned commands',
    scrimL: px(land ? RAIL_W : 0), scrimR: px(land ? (s.tab ? c._sheetW() : 0) : 0), scrimA: (travel / w * 0.35).toFixed(3),
    hGripDown: (e) => c._qdDown(e), hGripMove: (e) => c._qdMove(e), hGripUp: (e) => c._qdUp(e), hScrim: () => c.setState({ qd: false, qdDrag: null })
  };
}
