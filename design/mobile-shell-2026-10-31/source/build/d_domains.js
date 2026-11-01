/* ======================================================================================================
   Icons: 16×16 stroke paths, drawn at 1.2–1.4 px. Inline SVG only, never emoji or font glyphs, so they
   render the same on every Android build (⚙ and ⚄ turn into colour emoji on some).
   ====================================================================================================== */
const IC = {
  map: 'M1.5 3.5l4-1.5 5 1.5 4-1.5v10.5l-4 1.5-5-1.5-4 1.5ZM5.5 2v10.5M10.5 3.5V14',
  gen: 'M1.5 13.5l4-7 3 4.5 2-3 4 5.5ZM11.5 1.5v3.5M9.75 3.25h3.5',
  civil: 'M2 14V7.5L5 5l3 2.5V14M8 14V4.5L11 2l3 2.5V14M1 14h14M4.5 10h1M10.5 7h1M10.5 10h1',
  carto: 'M2.5 13.5l2.8-.7 7.4-7.4a1.4 1.4 0 0 0-2-2L3.3 10.8ZM9.5 4.5l2 2M2 2.5h5',
  story: 'M3 2h7.5L13 4.5V14H3ZM10 2v3h3M5.5 8h5M5.5 10.5h5',
  menu: 'M2.5 4h11M2.5 8h11M2.5 12h11',
  file: 'M3.5 1.5h6l3 3v10h-9ZM9.5 1.5v3h3',
  edit: 'M11 2.5l2.5 2.5L6 12.5l-3.5 1 1-3.5Z',
  assets: 'M2 2h5v5H2ZM9 2h5v5H9ZM2 9h5v5H2ZM9 9h5v5H9Z',
  world: 'M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM1.5 8h13M8 1.5c2 2 2.6 4.2 2.6 6.5S10 12.5 8 14.5M8 1.5C6 3.5 5.4 5.7 5.4 8S6 12.5 8 14.5',
  gear: 'M8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5ZM8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4',
  window: 'M1.5 2.5h13v11h-13ZM1.5 5.5h13M6 5.5v8',
  help: 'M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6 6.2a2 2 0 1 1 2.8 1.8c-.6.3-.8.7-.8 1.3v.5M8 11.5v.01',
  inspect: 'M3 2.5l4.3 10.5 1.6-4.4 4.4-1.6Z',
  measure: 'M2 11.5L11.5 2l2.5 2.5L4.5 14ZM5 9l1.5 1.5M7 7l1 1M9 5l1.5 1.5',
  region: 'M2.5 4.5v-2h2M11.5 2.5h2v2M13.5 11.5v2h-2M4.5 13.5h-2v-2M7 2.5h2M7 13.5h2M2.5 7v2M13.5 7v2',
  uplift: 'M1.5 13.5l4.5-8 2.5 4 2-3 4 7Z',
  carve: 'M1.5 3.5h4l1 9h3l1-9h4',
  freehand: 'M2 13c2-1 2-5 5-5s2 4 5 3 2-6 2-8',
  paint: 'M10 2.5l3.5 3.5-6 6-3.5-3.5ZM4 9l-1.5 4.5L7 12',
  landmark: 'M8 1.5v13M8 2.5h5l-1.5 2 1.5 2H8',
  settle: 'M2 14V7.5L8 3l6 4.5V14M6 14v-4h4v4',
  territory: 'M2.5 3.5l5-1.5 6 2.5-1 5.5-5 3-5.5-3Z',
  lasso: 'M8 3c3.5 0 6 1.6 6 3.5S11.5 10 8 10 2 8.4 2 6.5 4.5 3 8 3ZM5 9.5c-.5 1.5 0 3 2 4',
  conflict: 'M3 13L13 3M13 13L3 3M3 3h3M3 3v3M13 3h-3M13 3v3',
  way: 'M2 13c3-1 3-5 6-5s3-4 6-5',
  route: 'M3 12.5a1.5 1.5 0 1 0 0 .01ZM13 3.5a1.5 1.5 0 1 0 0 .01ZM4.5 12C9 12 7 4 11.5 4',
  label: 'M3 3.5h10M8 3.5v9M5.5 12.5h5',
  icon: 'M8 1.5l6.5 6.5L8 14.5 1.5 8Z',
  colour: 'M8 2c3 3.5 4.5 5.7 4.5 7.7a4.5 4.5 0 0 1-9 0C3.5 7.7 5 5.5 8 2Z',
  layers: 'M8 2l6 3.5-6 3.5-6-3.5ZM2 8.5L8 12l6-3.5M2 11.5L8 15l6-3.5',
  style: 'M8 1.5a6.5 6.5 0 1 0 0 13c1 0 1.5-.6 1.5-1.3 0-1-1-1.2-1-2.2 0-.8.6-1.5 1.5-1.5h1.5a3 3 0 0 0 3-3C14.5 3.6 11.6 1.5 8 1.5ZM4.5 7.5h.01M6.5 4.5h.01M10 4.5h.01',
  pipeline: 'M2.5 3h11M2.5 6.5h11M2.5 10h7M2.5 13.5h4',
  sculpt: 'M2 13.5c2.5-6 4-9 6-9s3.5 3 6 9ZM6 9.5h4',
  journey: 'M2.5 13.5c4 0 3-5 6.5-5s2.5-5 4.5-5M2.5 13.5h.01M13.5 3.5h.01',
  time: 'M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM8 4.5V8l2.5 2',
  notes: 'M3.5 1.5h9v13h-9ZM6 5h4M6 8h4M6 11h2.5',
  search: 'M7 2.5a4.5 4.5 0 1 0 0 9 4.5 4.5 0 0 0 0-9ZM10.4 10.4L14 14',
  pin: 'M8 14.5S3 9.6 3 6.5a5 5 0 0 1 10 0c0 3.1-5 8-5 8ZM8 8.2a1.7 1.7 0 1 0 0-3.4 1.7 1.7 0 0 0 0 3.4Z',
  star: 'M8 1.8l1.9 4 4.4.5-3.3 3 .9 4.4L8 11.5l-3.9 2.2.9-4.4-3.3-3 4.4-.5Z',
  dice: 'M3 2.5h10v11H3ZM6 6h.01M10 6h.01M8 8h.01M6 10h.01M10 10h.01',
  play: 'M4.5 2.5v11l9-5.5Z', pause: 'M4.5 2.5h2.5v11H4.5ZM9 2.5h2.5v11H9Z',
  save: 'M2.5 2.5h9l2 2v9h-11ZM5 2.5v3.5h5V2.5M5 13.5V9.5h6v4',
  export: 'M8 10V2M5 5l3-3 3 3M2.5 9.5v4h11v-4',
  vault: 'M2.5 3.5h11v10h-11ZM5.5 3.5V2h5v1.5M8 7v3M6.5 8.5h3',
  faction: 'M3 14V2M3 2.5h9l-2 2.5 2 2.5H3',
  ruler: 'M2 11.5L11.5 2l2.5 2.5L4.5 14ZM5 9l1.5 1.5M7 7l1 1M9 5l1.5 1.5',
  eye: 'M1.5 8s2.5-4.5 6.5-4.5S14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8ZM8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z',
  sample: 'M8 14.5S3 9.6 3 6.5a5 5 0 0 1 10 0c0 3.1-5 8-5 8ZM8 8.2a1.7 1.7 0 1 0 0-3.4 1.7 1.7 0 0 0 0 3.4Z',
  link: 'M6.5 9.5l3-3M7 4.5l1.2-1.2a2.8 2.8 0 0 1 4 4L11 8.5M9 11.5l-1.2 1.2a2.8 2.8 0 0 1-4-4L5 7.5',
  copy: 'M5.5 5.5h8v8h-8ZM3 10.5H2.5v-8h8V3',
  trash: 'M2.5 4h11M6 4V2.5h4V4M4 4l.7 9.5h6.6L12 4',
  check: 'M3.5 8.5l3 3 6-7', plus: 'M8 3v10M3 8h10', close: 'M3.5 3.5l9 9M12.5 3.5l-9 9',
  assistant: 'M8 1.5l1.6 4.9 4.9 1.6-4.9 1.6L8 14.5l-1.6-4.9L1.5 8l4.9-1.6Z',
  cube: 'M8 1.5l5.5 3v7L8 14.5l-5.5-3v-7ZM8 8l5.5-3.5M8 8L2.5 4.5M8 8v6.5',
  sim: 'M2 13.5h12M3.5 13.5V9M6.5 13.5V6M9.5 13.5V8M12.5 13.5V3.5',
  data: 'M8 2c3.6 0 6 1 6 2.2v7.6c0 1.2-2.4 2.2-6 2.2s-6-1-6-2.2V4.2C2 3 4.4 2 8 2ZM2 4.2c0 1.2 2.4 2.2 6 2.2s6-1 6-2.2M2 8c0 1.2 2.4 2.2 6 2.2s6-1 6-2.2',
  travel: 'M3 11.5h10M4 11.5l1-5h6l1 5M6 6.5V4h4v2.5M4.5 13.5h.01M11.5 13.5h.01',
  touch: 'M6 7.5V3a1.2 1.2 0 0 1 2.4 0v4l3.4.8c.8.2 1.2 1 1 1.8l-1 4.4H6.5L3.8 10.6a1.2 1.2 0 0 1 1.8-1.6L6 9.5'
};

/* ======================================================================================================
   Tools. Each one is armed from its node's palette (phone: the domain sheet; tablet: the dock), drops
   the phone sheet to peek, and shows what it needs in the tool strip (phone) or tool-options bar (tablet).
   Keys are the desktop's tool shortcuts.
   ====================================================================================================== */
const SCULPT_FEATURES = { uplift: ['Mountains', 'Hills', 'Ridge', 'Plateau', 'Cliff / Escarpment', 'Volcano'], carve: ['Canyon', 'Valley', 'River', 'Lake', 'Basin', 'Coastline'], freehand: ['Raise', 'Lower', 'Smooth', 'Cliff', 'Ridge', 'Canyon', 'Mesa', 'Volcano'] };
const BIOME_CLASSES = ['Ice', 'Tundra', 'Taiga', 'Temperate forest', 'Grassland', 'Steppe', 'Savanna', 'Desert', 'Tropical forest', 'Wetland'];
const STYLE_PRESETS = ['Natural Vibrant', 'Default', 'Antique', 'Ink', 'Watercolor', 'Print', 'Village', 'Atlas', 'Imhof relief', 'Blueprint', 'Ink wash', 'Woodcut', 'Vintage atlas', 'Nautical', 'Night', 'Cel / Toon', 'Tanaka relief', 'Satellite relief'];
const STYLE_LOOK = { 'Natural Vibrant': ['physical', 'earth'], 'Default': ['atlas', 'earth'], 'Antique': ['parchment', 'imhof'], 'Ink': ['ink', 'earth'], 'Watercolor': ['parchment', 'earth'], 'Print': ['atlas', 'atlas'], 'Village': ['atlas', 'earth'], 'Atlas': ['atlas', 'atlas'], 'Imhof relief': ['physical', 'imhof'], 'Blueprint': ['atlas', 'darkice'], 'Ink wash': ['ink', 'mono'], 'Woodcut': ['ink', 'mono'], 'Vintage atlas': ['parchment', 'atlas'], 'Nautical': ['atlas', 'ice'], 'Night': ['atlas', 'darkatlas'], 'Cel / Toon': ['physical', 'earth'], 'Tanaka relief': ['physical', 'mono'], 'Satellite relief': ['physical', 'earth'] };
const MEASURE_MODES = ['Distance', 'Bearing', 'Area', 'Radius', 'Cross-section', 'Δ vertical'];
function stampTool(kind) {
  return {
    stroke: true,
    onTap: (c, wp) => { const s = c.state; const r = s.brush.size * 1.6; const st = { x: wp.x, y: wp.y, r, kind, feature: s.feature, lower: kind === 'carve' || s.feature === 'Lower' }; c._addTo('stamps', st, 'stamp · ' + s.feature); },
    onStroke: (c, pts) => { const s = c.state; const st = { x: pts[0].x, y: pts[0].y, pts, r: s.brush.size * 0.9, kind, feature: s.feature, lower: kind === 'carve' || s.feature === 'Lower' }; c._addTo('stamps', st, 'stroke · ' + s.feature); }
  };
}
const TOOLS = {
  inspect: { label: 'Inspect', short: 'INSPECT', key: 'V', d: IC.inspect, hint: 'tap selects · long-press samples' },
  measure: { label: 'Measure', short: 'MEASURE', key: 'M', d: IC.measure, hint: 'tap the map to add points', onTap: (c, wp) => c.setState((st) => ({ measure: st.measure.concat([wp]) })) },
  region: { label: 'Region select', short: 'REGION', key: 'R', d: IC.region, stroke: true, hint: 'drag a box on the map', onTap: (c) => c._toast('Drag a box to select a region'), onStroke: (c, pts) => { const a = pts[0], b = pts[pts.length - 1]; c._toast('Region ' + Math.round(Math.abs(b.x - a.x)) + ' × ' + Math.round(Math.abs(b.y - a.y)) + ' km selected · File ▸ New world from selection…'); c.setState({ region: { a, b } }); } },
  uplift: Object.assign({ label: 'Uplift', short: 'UPLIFT', key: '', d: IC.uplift, hint: 'drag to raise · tap stamps' }, stampTool('uplift')),
  carve: Object.assign({ label: 'Carve', short: 'CARVE', key: '', d: IC.carve, hint: 'drag to cut · tap stamps' }, stampTool('carve')),
  freehand: Object.assign({ label: 'Freehand', short: 'FREEHAND', key: 'F', d: IC.freehand, hint: 'drag to sculpt' }, stampTool('freehand')),
  paint: Object.assign({ label: 'Paint', short: 'PAINT', key: 'B', d: IC.paint, hint: 'drag paints the class' }, stampTool('paint')),
  landmark: { label: 'Landmark', short: 'LANDMARK', key: '', d: IC.landmark, hint: 'tap places a landmark by hand', onTap: (c, wp) => { const lm = { id: 'ul' + Math.random().toString(36).slice(2, 7), kind: c.state.lmKind, fam: 'Placed by hand', name: c.state.lmKind + ' (by hand)', x: wp.x, y: wp.y, user: true }; c.landmarks.push(lm); c._push('landmark · ' + lm.kind, () => { c.landmarks = c.landmarks.filter((x) => x !== lm); }, () => { c.landmarks.push(lm); }); c._toast(lm.kind + ' placed by hand'); } },
  settle: { label: 'Settlement', short: 'SETTLE', key: 'S', d: IC.settle, hint: 'tap drops a place', onTap: (c, wp) => { const n = PLACE_NAMES[(c.places.length + c.state.userPlaces.length + 3) % PLACE_NAMES.length] + ' ' + ['Minor', 'New', 'Upper', 'Lower'][c.state.userPlaces.length % 4]; const pl = { id: 'u' + Math.random().toString(36).slice(2, 7), name: n, x: wp.x, y: wp.y, tier: c.state.settleTier === 'metropolis' ? 'city' : c.state.settleTier, user: true, fac: -1, pop: 0, founded: c.state.year, biome: c.world ? biomeOf(c.world, Math.min(c.world.N * c.world.N - 1, Math.floor(wp.y / c.world.cell) * c.world.N + Math.floor(wp.x / c.world.cell))) : '—' }; c._addTo('userPlaces', pl, 'settlement · ' + n); c._toast(n + ' placed · ' + c.state.settleTier); } },
  territory: { label: 'Territory', short: 'TERRITORY', key: 'T', d: IC.territory, stroke: true, hint: 'drag to claim for the selected faction', onTap: (c) => c._toast('Drag across the map to claim'), onStroke: (c, pts) => c._toast('Claimed ' + Math.round(pts.length * 12) + ' km² for ' + (FACTIONS[c.state.facPick || 0].name) + ' · commit or discard') },
  lasso: { label: 'Territory lasso', short: 'LASSO', key: 'Shift+T', d: IC.lasso, hint: 'taps place a ring', onTap: (c, wp) => c.setState((st) => ({ wayDraft: st.wayDraft.concat([wp]) })) },
  conflict: { label: 'Conflict', short: 'CONFLICT', key: '', d: IC.conflict, hint: 'taps draw a front, arrow, siege or battle', onTap: (c, wp) => c.setState((st) => ({ conflictDraft: st.conflictDraft.concat([wp]) })) },
  way: { label: 'Way', short: 'WAY', key: 'W', d: IC.way, stroke: true, hint: 'tap appends waypoints · drag draws', onTap: (c, wp) => c.setState((st) => ({ wayDraft: st.wayDraft.concat([wp]) })), onStroke: (c, pts) => c.setState((st) => ({ wayDraft: st.wayDraft.concat(pts.filter((p, i) => i % 2 === 0)) })) },
  route: { label: 'Route', short: 'ROUTE', key: 'Shift+R', d: IC.route, hint: 'tap two ports: sea and river legs', onTap: (c, wp) => c.setState((st) => ({ wayDraft: st.wayDraft.length >= 2 ? [wp] : st.wayDraft.concat([wp]) })) },
  label: { label: 'Label', short: 'LABEL', key: 'L', d: IC.label, hint: 'tap where the label goes', onTap: (c, wp) => c.setState({ labelAt: wp }) },
  icon: { label: 'Icon', short: 'ICON', key: 'I', d: IC.icon, hint: 'tap stamps the armed icon', onTap: (c, wp) => { const ic = { x: wp.x, y: wp.y, kind: c.state.iconKind }; c._addTo('userIcons', ic, 'icon · ' + ic.kind); } },
  colourpaint: { label: 'Colour paint', short: 'COLOUR', key: '', d: IC.colour, stroke: true, hint: 'drag paints a colour mark', onTap: (c) => c._toast('Drag to paint a colour mark'), onStroke: (c) => c._toast('Colour mark added · Replace · land only') }
};
function toolDef(id) { return TOOLS[id]; }

/* ======================================================================================================
   Domains and nodes, from `DccShell.RAIL_NODES`: each node owns the dock categories listed here, and
   on the phone those categories are the node's sheet sections (today most of them are reachable only
   through MORE ▸ Window ▸ Left dock).
   ====================================================================================================== */
const DOMAIN = {
  world: { id: 'world', label: 'WORLD', tab: 'GENERATE', d: IC.gen, sub: 'generate · sculpt · paint', nodes: [
    { id: 'pipeline', label: 'Generate', short: 'GENERATE', d: IC.pipeline, cats: ['Generate', 'Planet', 'Geology', 'Hydrology', 'Climate', 'Ecology', 'World data'], tools: ['inspect', 'measure', 'region'] },
    { id: 'sculpt', label: 'Sculpt', short: 'SCULPT', d: IC.sculpt, cats: ['Terrain', 'Biomes'], tools: ['inspect', 'uplift', 'carve', 'freehand', 'paint'] }
  ] },
  civil: { id: 'civil', label: 'CIVIL', tab: 'CIVIL', d: IC.civil, sub: 'landmarks · factions · ways · journeys', nodes: [
    { id: 'landmarks', label: 'Landmarks', short: 'LANDMARKS', d: IC.landmark, cats: ['Landmarks'], tools: ['inspect', 'landmark'] },
    { id: 'factions', label: 'Factions', short: 'FACTIONS', d: IC.faction, cats: ['Populate', 'Settlements', 'Factions', 'Territories', 'Relationships', 'Military', 'Culture', 'Religion', 'Economy', 'Timeline'], tools: ['inspect', 'settle', 'territory', 'lasso', 'conflict'] },
    { id: 'ways', label: 'Routes & ways', short: 'ROUTES & WAYS', d: IC.way, cats: ['Routes & ways'], tools: ['inspect', 'way', 'route'] },
    { id: 'journey', label: 'Journey planner', short: 'JOURNEYS', d: IC.journey, cats: ['Travel'], tools: ['inspect', 'measure'] }
  ] },
  carto: { id: 'carto', label: 'CARTO', tab: 'CARTO', d: IC.carto, sub: 'style · layers · labels · icons', nodes: [
    { id: 'style', label: 'Style', short: 'STYLE', d: IC.style, cats: ['Style', 'Relief & light', 'Colours', 'Feature style'], tools: ['inspect', 'colourpaint'] },
    { id: 'layers', label: 'Layers', short: 'LAYERS', d: IC.layers, cats: ['Layers'], tools: ['inspect'] },
    { id: 'labels', label: 'Labels', short: 'LABELS', d: IC.label, cats: ['Labels'], tools: ['inspect', 'label'] },
    { id: 'icons', label: 'Icons', short: 'ICONS', d: IC.icon, cats: ['Icons'], tools: ['inspect', 'icon'] }
  ] }
};
function domainDef(d) { return DOMAIN[d]; }
function nodeDef(d, n) { const D = DOMAIN[d]; return D.nodes.find((x) => x.id === n) || D.nodes[0]; }
