/* ---------- stand-in world content: settlements, factions, a journey route, landmarks, rivers, ways ---------- */
const PLACE_NAMES = ['Vhal Serai', 'Kess Ford', 'Thornwood', 'High Saddle', 'Grey Vale', 'Lakemouth', 'Port Amre', 'Orent', 'Sevjuni', 'Farbahrir', 'Ashqirspan', 'Bomlavcastra', 'Mirzafir', 'Eldmoor', 'Qadrun', 'Tessaly', 'Wynora', 'Halvik', 'Corrabar', 'Daskin', 'Umbrel', 'Zafzafar'];
const FACTIONS = [
  { name: 'Orentamwash', col: '#c96a5a', rgb: [201, 106, 90], gov: 'Monarchy', rel: 'The Old Rite' },
  { name: 'Sevjuni League', col: '#6a9bc4', rgb: [106, 155, 196], gov: 'Merchant republic', rel: 'Lantern faith' },
  { name: 'Korrath', col: '#8fae7d', rgb: [143, 174, 125], gov: 'Tribal confederacy', rel: 'Ancestor cult' },
  { name: 'Mirzafir Reach', col: '#d3a84e', rgb: [211, 168, 78], gov: 'Theocracy', rel: 'The Old Rite' }
];
const TIERS = ['city', 'city', 'town', 'town', 'town', 'town', 'village', 'village', 'village', 'village', 'village', 'hamlet', 'hamlet', 'hamlet', 'hamlet', 'hamlet', 'hamlet', 'hamlet'];
function placeWorld(w, seed, worldName) {
  const N = w.N, cell = w.cell, R = mulberry32(seed + 77);
  const cand = [];
  for (let j = 4; j < N - 4; j += 2) for (let i = 4; i < N - 4; i += 2) {
    const k = j * N + i; const z = w.elevM[k];
    if (z <= 8 || z > 1500 || w.filled[k] - w.e[k] > 0.025) continue;
    let coast = 0; for (let q = -3; q <= 3; q++) { if (w.elevM[(j + q) * N + i] < 0 || w.elevM[j * N + i + q] < 0) coast = 1; }
    const river = Math.min(1, Math.log(w.acc[k]) / 7);
    const score = 1 + river * 1.4 + coast * 0.9 - w.slope[k] * 0.25 - z / 2500 + R() * 0.5;
    cand.push({ i, j, score });
  }
  cand.sort((a, b) => b.score - a.score);
  const places = [];
  for (const c of cand) {
    if (places.length >= 18) break;
    const x = (c.i + 0.5) * cell, y = (c.j + 0.5) * cell;
    if (places.some((p) => Math.hypot(p.x - x, p.y - y) < 150)) continue;
    places.push({ x, y, k: c.j * N + c.i });
  }
  // a west-to-east journey through seven of them, named as the planner canvases name it
  const route = [];
  if (places.length >= 7) {
    let cur = places.reduce((a, b) => (a.x < b.x ? a : b)); route.push(cur);
    while (route.length < 7) { let best = null, bd = 1e9; for (const p of places) { if (route.includes(p)) continue; const d = Math.hypot(p.x - cur.x, p.y - cur.y) - (p.x - cur.x) * 0.6; if (p.x > cur.x - 60 && d < bd) { bd = d; best = p; } } if (!best) break; route.push(best); cur = best; }
  }
  let ni = 7; const used = new Set();
  route.forEach((p, i) => { p.name = PLACE_NAMES[i]; used.add(p); });
  for (const p of places) if (!used.has(p)) p.name = PLACE_NAMES[ni++ % PLACE_NAMES.length];
  // tiers, factions (capitals = the four best-spread larger places), population
  places.forEach((p, i) => { p.tier = TIERS[i] || 'hamlet'; p.id = 'p' + i; });
  const caps = []; for (const p of places) { if (caps.length >= 4) break; if (caps.every((c) => Math.hypot(c.x - p.x, c.y - p.y) > 420)) caps.push(p); }
  for (const p of places) if (caps.length < 4 && !caps.includes(p)) caps.push(p);
  const owner = new Int8Array(N * N).fill(-1);
  for (let k = 0; k < N * N; k++) { if (w.elevM[k] < 0) continue; const x = ((k % N) + 0.5) * cell, y = (((k / N) | 0) + 0.5) * cell; let b = -1, bd = 1e12; caps.forEach((c, f) => { const d = (c.x - x) * (c.x - x) + (c.y - y) * (c.y - y); if (d < bd) { bd = d; b = f; } }); owner[k] = bd < 520 * 520 ? b : -1; }
  for (const p of places) { p.fac = owner[p.k]; const base = { city: 24000, town: 5200, village: 900, hamlet: 160 }[p.tier]; p.pop = Math.round(base * (0.7 + R() * 0.8)); p.founded = -300 + Math.round(R() * 900); p.elev = Math.round(w.elevM[p.k]); p.biome = biomeOf(w, p.k); }
  // the named route places carry the story and the seeded notes, so none is left unclaimed: an outpost of the nearest capital
  route.forEach((p) => { if (p.fac >= 0 || !caps.length) return; let bi = 0, bd = 1e9; caps.forEach((cp, i) => { const d = Math.hypot(cp.x - p.x, cp.y - p.y); if (d < bd) { bd = d; bi = i; } }); p.fac = bi; });
  caps.forEach((c, f) => { c.capital = true; c.tier = c.tier === 'hamlet' || c.tier === 'village' ? 'town' : c.tier; });
  const factions = FACTIONS.map((f, i) => Object.assign({ id: 'f' + i, capital: caps[i] ? caps[i].name : '—', places: places.filter((p) => p.fac === i).length, pop: places.filter((p) => p.fac === i).reduce((a, p) => a + p.pop, 0) }, f));
  const sums = caps.map(() => [0, 0, 0]); for (let k = 0; k < N * N; k += 3) { const f = owner[k]; if (f >= 0) { sums[f][0] += (k % N + 0.5) * cell; sums[f][1] += ((k / N | 0) + 0.5) * cell; sums[f][2]++; } }
  const regions = caps.map((c, i) => ({ name: FACTIONS[i].name.toUpperCase(), x: sums[i][2] ? sums[i][0] / sums[i][2] : c.x, y: sums[i][2] ? sums[i][1] / sums[i][2] : c.y }));
  // ways: a minimum spanning tree over the places, as roads; long legs as tracks
  const ways = []; const inT = [places[0]];
  while (inT.length < places.length) { let best = null; for (const a of inT) for (const b of places) { if (inT.includes(b)) continue; const d = Math.hypot(a.x - b.x, a.y - b.y); if (!best || d < best.d) best = { a, b, d }; } inT.push(best.b); ways.push({ a: best.a, b: best.b, kind: best.d > 240 ? 'track' : 'road', km: Math.round(best.d * 1.18) }); }
  // rivers as vector strokes (cells with a large accumulation, joined to their receiver)
  const rivers = [[], [], []];
  for (let k = 0; k < N * N; k++) { if (w.e[k] <= 0 || w.acc[k] < 70 || w.recv[k] < 0) continue; const r = w.recv[k]; const cls = w.acc[k] > 900 ? 2 : (w.acc[k] > 260 ? 1 : 0); rivers[cls].push((k % N + 0.5) * cell, ((k / N | 0) + 0.5) * cell, (r % N + 0.5) * cell, ((r / N | 0) + 0.5) * cell); }
  // landmarks: the highest local peaks, a few waterfalls and shrines, a ruin
  const landmarks = []; const peaks = [];
  for (let j = 6; j < N - 6; j += 3) for (let i = 6; i < N - 6; i += 3) { const k = j * N + i; const z = w.elevM[k]; if (z < 1800) continue; let top = true; for (let q = -5; q <= 5 && top; q++) for (let r = -5; r <= 5; r++) { if (w.elevM[(j + q) * N + i + r] > z) { top = false; break; } } if (top) peaks.push({ x: (i + 0.5) * cell, y: (j + 0.5) * cell, z }); }
  peaks.sort((a, b) => b.z - a.z).slice(0, 7).forEach((p, i) => landmarks.push({ id: 'lm' + landmarks.length, kind: 'Peak', fam: 'Physical', name: ['Mount Asker', 'The Grey Tooth', 'Kessan Crown', 'Hollow Peak', 'Varrow', 'Sentinel', 'Ember Spire'][i], x: p.x, y: p.y, z: Math.round(p.z) }));
  let wf = 0; for (let k = 0; k < N * N && wf < 4; k += 97) { if (w.e[k] > 0 && w.acc[k] > 300 && w.slope[k] > 1.2) { landmarks.push({ id: 'lm' + landmarks.length, kind: 'Waterfall', fam: 'Physical', name: 'Falls of ' + PLACE_NAMES[(wf + 9) % PLACE_NAMES.length], x: (k % N + 0.5) * cell, y: ((k / N | 0) + 0.5) * cell }); wf++; } }
  places.slice(2, 8).forEach((p, i) => { if (i % 2) landmarks.push({ id: 'lm' + landmarks.length, kind: i === 3 ? 'Ruin' : 'Shrine', fam: i === 3 ? 'Historical' : 'Cultural', name: (i === 3 ? 'Ruins near ' : 'Shrine of ') + p.name, x: p.x + 36, y: p.y - 28 }); });
  ways.filter((wy) => wy.kind === 'road').slice(0, 2).forEach((wy) => landmarks.push({ id: 'lm' + landmarks.length, kind: 'Ford', fam: 'Transport', name: 'Ford on the ' + wy.a.name + ' road', x: (wy.a.x + wy.b.x) / 2, y: (wy.a.y + wy.b.y) / 2 }));
  return { places, factions, route, landmarks, rivers, ways, regions, owner };
}
function hm() { const d = new Date(); return String(d.getHours()).padStart(2, '0') + ':' + String(d.getMinutes()).padStart(2, '0'); }
function fmtDist(km, units) { if (units === 'mi') return (km * 0.621371).toFixed(km < 10 ? 1 : 0) + ' mi'; if (units === 'nmi') return (km * 0.539957).toFixed(km < 10 ? 1 : 0) + ' nmi'; return (km < 10 ? km.toFixed(1) : Math.round(km).toLocaleString('en-US')) + ' km'; }
function niceScale(kmPerPx) { const want = kmPerPx * 90; const p = Math.pow(10, Math.floor(Math.log10(want))); const m = [1, 2, 5, 10].find((x) => x * p >= want * 0.7) * p; return { km: m, px: m / kmPerPx }; }

/* ---------- drawing ---------- */
const LM_GLYPH = { Peak: 'M-5 4L0-5 5 4Z', Waterfall: 'M-4-5v5c0 3 2 4 4 4s4-1 4-4v-5', Shrine: 'M-5 5h10M-4 5v-5h8v5M-5 0l5-4 5 4', Ruin: 'M-5 5h10M-3 5V-2M1 5V-4M4 5V0', Ford: 'M-5-2c2-2 3 2 5 0s3 2 5 0M-5 2c2-2 3 2 5 0s3 2 5 0' };
function drawMap(c, ctx, W, H) {
  const s = c.state, v = c.view, KM = 2048, L = s.layers;
  ctx.save(); ctx.translate(W / 2, H / 2); ctx.rotate(v.r); ctx.scale(v.s, v.s); ctx.translate(-v.cx, -v.cy);
  ctx.imageSmoothingEnabled = true; ctx.drawImage(c._baseCv, 0, 0, KM, KM);
  const px = 1 / v.s; ctx.lineCap = 'round'; ctx.lineJoin = 'round';
  if (L.grid) { ctx.strokeStyle = 'rgba(255,255,255,.18)'; ctx.lineWidth = px; ctx.beginPath(); for (let g = 0; g <= KM; g += 256) { ctx.moveTo(g, 0); ctx.lineTo(g, KM); ctx.moveTo(0, g); ctx.lineTo(KM, g); } ctx.stroke(); }
  // Colour paint marks (COLOUR_PAINT_SCOPE.md; the mock's marks and their rules are in d_cats.js, cpSample onwards).
  // Drawn over the ground and under rivers and ways, as the build composites paint into the land colour. A land-only
  // mark wholly on water is hidden, not deleted (§5), and CARTO ▸ Colours ▸ Hide colour paint hides every mark.
  if (s.cpMarks.length && !V(c, 'cp.hide', false)) {
    for (const m of s.cpMarks) { if (cpMarkWet(c, m)) continue; ctx.globalAlpha = 0.62; ctx.strokeStyle = m.col; ctx.lineWidth = m.r * 2; ctx.beginPath(); m.pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y))); ctx.stroke(); }
    ctx.globalAlpha = 1;
  }
  if (L.water && L.rivers) {
    const rc = s.style === 'ink' ? '#5b6b78' : (s.style === 'parchment' ? '#5f7f8c' : '#6fa3c8');
    [[0, 0.7], [1, 1.3], [2, 2.2]].forEach(([cls, wpx]) => { const a = c.rivers[cls]; if (!a || !a.length) return; if (cls === 0 && v.s < 0.35) return; ctx.strokeStyle = rc; ctx.globalAlpha = cls === 0 ? 0.7 : 0.9; ctx.lineWidth = wpx * px * Math.max(1, Math.min(2.4, v.s * 2)); ctx.beginPath(); for (let i = 0; i < a.length; i += 4) { ctx.moveTo(a[i], a[i + 1]); ctx.lineTo(a[i + 2], a[i + 3]); } ctx.stroke(); });
    ctx.globalAlpha = 1;
  }
  if (L.ways) {
    for (const wy of c.ways.concat(s.userWays.map((u) => ({ pts: u.pts, kind: u.kind, user: true })))) {
      ctx.strokeStyle = wy.user ? '#e0a34a' : (s.style === 'ink' ? '#3a3a38' : 'rgba(60,40,24,.85)'); ctx.lineWidth = (wy.kind === 'road' ? 1.6 : 1.1) * px;
      ctx.setLineDash(wy.kind === 'track' ? [5 * px, 4 * px] : []); ctx.beginPath();
      if (wy.pts) { wy.pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y))); } else { const mx = (wy.a.x + wy.b.x) / 2 + (wy.b.y - wy.a.y) * 0.08, my = (wy.a.y + wy.b.y) / 2 - (wy.b.x - wy.a.x) * 0.08; ctx.moveTo(wy.a.x, wy.a.y); ctx.quadraticCurveTo(mx, my, wy.b.x, wy.b.y); }
      ctx.stroke();
    }
    ctx.setLineDash([]);
  }
  const showRoute = (s.tab === 'story' && s.storySeg === 'journeys') || (s.dom === 'civil' && s.node.civil === 'journey' && c._mode() === 'tablet') || (s.win && s.win.stack && ['journeys', 'planner'].includes(s.win.stack[s.win.stack.length - 1]));
  if (showRoute && c.route.length > 1) {
    ctx.strokeStyle = 'rgba(13,14,15,.8)'; ctx.lineWidth = 6 * px; ctx.beginPath(); c.route.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y))); ctx.stroke();
    ctx.strokeStyle = '#e0a34a'; ctx.lineWidth = 3 * px; ctx.stroke();
    if (s.planStage >= 0 && c.route[s.planStage + 1]) { const a = c.route[s.planStage], b = c.route[s.planStage + 1]; ctx.strokeStyle = '#ffd38a'; ctx.lineWidth = 6 * px; ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke(); }
  }
  // drafts: measure, way, conflict, sculpt stamps, live stroke
  const poly = (pts, col, dash, wpx) => { if (!pts.length) return; ctx.strokeStyle = col; ctx.lineWidth = wpx * px; ctx.setLineDash(dash ? [6 * px, 5 * px] : []); ctx.beginPath(); pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y))); ctx.stroke(); ctx.setLineDash([]); ctx.fillStyle = col; pts.forEach((p) => { ctx.beginPath(); ctx.arc(p.x, p.y, 4 * px, 0, 7); ctx.fill(); }); };
  poly(s.measure, '#ffffff', true, 2); poly(s.wayDraft, '#e0a34a', true, 2.4); poly(s.conflictDraft, '#e08070', true, 2.4);
  for (const st of s.stamps) { ctx.strokeStyle = st.lower ? '#7fb2d8' : '#e0a34a'; ctx.lineWidth = 2 * px; ctx.setLineDash([5 * px, 4 * px]); ctx.beginPath(); if (st.pts && st.pts.length > 1) { st.pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y))); ctx.globalAlpha = 0.35; ctx.lineWidth = st.r * 2; ctx.stroke(); ctx.globalAlpha = 1; ctx.lineWidth = 2 * px; ctx.stroke(); } else { ctx.arc(st.x, st.y, st.r, 0, 7); ctx.stroke(); } ctx.setLineDash([]); }
  if (c._stroke && c._stroke.length) { ctx.strokeStyle = 'rgba(224,163,74,.5)'; ctx.lineWidth = Math.max(4 * px, (s.brush.size || 40) * 0.8); ctx.beginPath(); c._stroke.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y))); ctx.stroke(); }
  ctx.restore();
  // ---- screen space: regions, places, landmarks, labels, pins ----
  const ink = s.style === 'ink' || s.style === 'parchment';
  const textHalo = (t, x, y, font, fill, halo, ls) => { ctx.font = font; ctx.letterSpacing = ls || '0px'; ctx.lineWidth = 3; ctx.strokeStyle = halo; ctx.fillStyle = fill; ctx.strokeText(t, x, y); ctx.fillText(t, x, y); ctx.letterSpacing = '0px'; };
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  const boxes = []; const free = (x, y, w, h) => { for (const b of boxes) if (x < b[0] + b[2] && x + w > b[0] && y < b[1] + b[3] && y + h > b[1]) return false; boxes.push([x, y, w, h]); return true; };
  if (L.labels && v.s < 0.9) for (const r of c.regions) { const p = c._w2s(r.x, r.y); const fs = Math.round(Math.max(10, Math.min(17, 64 * v.s))); ctx.font = '500 ' + fs + 'px "IBM Plex Mono", monospace'; ctx.letterSpacing = Math.round(Math.max(2, Math.min(8, 22 * v.s))) + 'px'; const tw = ctx.measureText(r.name).width + 8; ctx.letterSpacing = '0px'; if (!free(p.x - tw / 2, p.y - fs, tw, fs * 2)) continue; textHalo(r.name, p.x, p.y, '500 ' + fs + 'px "IBM Plex Mono", monospace', ink ? '#3b2f22' : 'rgba(255,248,232,.92)', ink ? 'rgba(255,250,240,.7)' : 'rgba(20,16,10,.55)', Math.round(Math.max(2, Math.min(8, 22 * v.s))) + 'px'); }
  if (L.landmarks) for (const lm of c.landmarks) {
    if (v.s < 0.24 && lm.kind !== 'Peak') continue; const p = c._w2s(lm.x, lm.y); if (p.x < -20 || p.y < -20 || p.x > W + 20 || p.y > H + 20) continue;
    const sel = s.sel && s.sel.kind === 'landmark' && s.sel.id === lm.id;
    ctx.save(); ctx.translate(p.x, p.y); ctx.strokeStyle = sel ? '#e0a34a' : (ink ? '#2d2a26' : '#f3efe6'); ctx.fillStyle = 'rgba(0,0,0,.0)'; ctx.lineWidth = sel ? 2.2 : 1.5; ctx.shadowColor = 'rgba(0,0,0,.6)'; ctx.shadowBlur = ink ? 0 : 3; ctx.stroke(new Path2D(LM_GLYPH[lm.kind] || LM_GLYPH.Peak)); ctx.restore();
    if (v.s > 0.55) textHalo(lm.name, p.x, p.y + 14, '9.5px "IBM Plex Mono", monospace', ink ? '#2d2a26' : '#e8ebec', ink ? 'rgba(255,250,240,.8)' : 'rgba(10,10,10,.7)');
  }
  const subj = subjectOf(c); const subjF = subj && subj.kind === 'faction' ? facIdxOf(subj.id) : -1;
  for (const k of conflictsOf(c)) {
    const on = s.year >= k.from && s.year <= k.to; const picked = subj && subj.kind === 'conflict' && subj.id === k.id; if (!on && !picked) continue;
    const ps = k.places.map((id) => c.places.find((q) => q.id === id)).filter(Boolean).map((q) => c._w2s(q.x, q.y)); if (ps.length < 2) continue;
    ctx.save(); ctx.strokeStyle = k.col; ctx.globalAlpha = picked ? 1 : 0.75; ctx.lineWidth = picked ? 3 : 2; ctx.setLineDash([7, 5]); ctx.beginPath(); ps.forEach((q, i) => (i ? ctx.lineTo(q.x, q.y) : ctx.moveTo(q.x, q.y))); ctx.stroke(); ctx.setLineDash([]);
    ps.forEach((q) => { ctx.beginPath(); ctx.moveTo(q.x - 6, q.y - 6); ctx.lineTo(q.x + 6, q.y + 6); ctx.moveTo(q.x + 6, q.y - 6); ctx.lineTo(q.x - 6, q.y + 6); ctx.stroke(); });
    const m = ps[Math.floor(ps.length / 2)]; ctx.globalAlpha = 1; ctx.textAlign = 'center'; textHalo(k.name.toUpperCase() + ' · ' + k.from + '–' + k.to, (ps[0].x + ps[ps.length - 1].x) / 2, (ps[0].y + ps[ps.length - 1].y) / 2 - 12, '500 10px "IBM Plex Mono", monospace', k.col, 'rgba(8,8,8,.8)', '1px'); ctx.restore();
  }
  if (L.settlements) {
    const rank = { city: 0, town: 1, village: 2, hamlet: 3 }; const all = c.places.concat(s.userPlaces).slice().sort((a, b) => (rank[a.tier] || 0) - (rank[b.tier] || 0));
    for (const pl of all) {
      const p = c._w2s(pl.x, pl.y); if (p.x < -40 || p.y < -20 || p.x > W + 40 || p.y > H + 20) continue;
      const r = { city: 5.5, town: 4.2, village: 3.2, hamlet: 2.4 }[pl.tier] || 3; const show = pl.tier === 'city' || (pl.tier === 'town' && v.s > 0.16) || (pl.tier === 'village' && v.s > 0.32) || v.s > 0.6 || pl.user;
      if (!show) continue;
      // the shared year: who held the place then, and whether it existed yet (PC_SHELL_PROPOSAL.md §3.2)
      const own = ownerAt(c, pl, s.year); const unborn = own === null;
      const fc = own != null && own >= 0 && FACTIONS[own] ? FACTIONS[own].col : '#e8ebec';
      if (unborn && s.tlFilt && s.tlFilt.exist) continue;   // Exist only: a place not founded yet is not handed to the map
      if (unborn) { ctx.save(); ctx.setLineDash([2.5, 2.5]); ctx.beginPath(); ctx.arc(p.x, p.y, r + 0.5, 0, 7); ctx.strokeStyle = ink ? 'rgba(45,42,38,.6)' : 'rgba(246,242,234,.6)'; ctx.lineWidth = 1.2; ctx.stroke(); ctx.restore(); }
      else { ctx.beginPath(); ctx.arc(p.x, p.y, r + 1.5, 0, 7); ctx.fillStyle = ink ? '#2d2a26' : 'rgba(10,10,10,.85)'; ctx.fill();
      ctx.beginPath(); ctx.arc(p.x, p.y, r, 0, 7); ctx.fillStyle = s.base === 'political' || L.political ? fc : (pl.user ? '#e0a34a' : '#f6f2ea'); ctx.fill(); }
      if (pl.capital && !unborn) { ctx.beginPath(); ctx.arc(p.x, p.y, r + 4, 0, 7); ctx.strokeStyle = ink ? '#2d2a26' : '#f6f2ea'; ctx.lineWidth = 1.2; ctx.stroke(); }
      if (subjF >= 0 && own === subjF) { ctx.beginPath(); ctx.arc(p.x, p.y, r + 7, 0, 7); ctx.strokeStyle = FACTIONS[subjF].col; ctx.lineWidth = 2; ctx.stroke(); }
      if (unborn) ctx.globalAlpha = 0.5;
      if (L.labels && (pl.user || !V(c, 'lb.cull', true) || free(p.x + r + 4, p.y - 8, pl.name.length * 6.6 + 4, 16))) { ctx.textAlign = 'left'; textHalo(pl.name, p.x + r + 6, p.y, (pl.tier === 'city' ? '500 12.5px' : pl.tier === 'town' ? '11.5px' : '10.5px') + ' "Helvetica Neue", Helvetica, Arial, sans-serif', ink ? '#221c15' : '#f4f1ea', ink ? 'rgba(255,250,240,.85)' : 'rgba(8,8,8,.75)'); ctx.textAlign = 'center'; }
      ctx.globalAlpha = 1;
      const sel = s.sel && s.sel.kind === 'place' && s.sel.id === pl.id;
      if (sel) { ctx.beginPath(); ctx.arc(p.x, p.y, r + 9, 0, 7); ctx.strokeStyle = '#e0a34a'; ctx.lineWidth = 2.5; ctx.stroke(); }
      const pin = s.ctxPin && s.ctxPin.kind === 'place' && s.ctxPin.id === pl.id && !sel;
      if (pin) { ctx.save(); ctx.setLineDash([4, 3]); ctx.beginPath(); ctx.arc(p.x, p.y, r + 9, 0, 7); ctx.strokeStyle = '#e0a34a'; ctx.lineWidth = 2; ctx.stroke(); ctx.restore(); }
    }
  }
  for (const ic of s.userIcons) { const p = c._w2s(ic.x, ic.y); ctx.save(); ctx.translate(p.x, p.y); ctx.strokeStyle = '#e0a34a'; ctx.lineWidth = 2; ctx.fillStyle = 'rgba(224,163,74,.25)'; const path = { diamond: 'M0-7L7 0 0 7-7 0Z', circle: 'M7 0A7 7 0 1 1-7 0 7 7 0 1 1 7 0', triangle: 'M0-7L7 6H-7Z', square: 'M-6-6h12v12h-12Z' }[ic.kind] || 'M0-7L7 0 0 7-7 0Z'; const pp = new Path2D(path); ctx.fill(pp); ctx.stroke(pp); ctx.restore(); }
  if (L.labels) for (const lb of s.userLabels) { const p = c._w2s(lb.x, lb.y); textHalo(lb.text, p.x, p.y, 'italic 500 13px "Helvetica Neue", Helvetica, Arial, sans-serif', ink ? '#221c15' : '#fff4dc', ink ? 'rgba(255,250,240,.85)' : 'rgba(8,8,8,.75)', '1px'); }
  if (s.measure.length > 1) { let km = 0; for (let i = 1; i < s.measure.length; i++) km += Math.hypot(s.measure[i].x - s.measure[i - 1].x, s.measure[i].y - s.measure[i - 1].y); const l = s.measure[s.measure.length - 1]; const p = c._w2s(l.x, l.y); ctx.textAlign = 'left'; textHalo(fmtDist(km, s.prefs.units), p.x + 10, p.y - 12, '500 11px "IBM Plex Mono", monospace', '#ffffff', 'rgba(0,0,0,.8)'); }
  if (s.labelAt) { const p = c._w2s(s.labelAt.x, s.labelAt.y); ctx.beginPath(); ctx.arc(p.x, p.y, 6, 0, 7); ctx.strokeStyle = '#e0a34a'; ctx.lineWidth = 2; ctx.stroke(); }
  if (s.sample) { const p = c._w2s(s.sample.x, s.sample.y); ctx.save(); ctx.translate(p.x, p.y); ctx.shadowColor = 'rgba(0,0,0,.5)'; ctx.shadowBlur = 4; ctx.fillStyle = '#e0a34a'; ctx.strokeStyle = '#16130c'; ctx.lineWidth = 1.5; const pin = new Path2D('M0 0C0 0-11-13-11-20.5a11 11 0 0 1 22 0C11-13 0 0 0 0Z'); ctx.fill(pin); ctx.stroke(pin); ctx.shadowBlur = 0; ctx.fillStyle = '#16130c'; ctx.beginPath(); ctx.arc(0, -20.5, 4, 0, 7); ctx.fill(); ctx.restore(); }
  if (s.prefs.diag) { ctx.textAlign = 'left'; textHalo('view ' + v.cx.toFixed(0) + ',' + v.cy.toFixed(0) + ' · ' + (v.s * 100).toFixed(0) + ' dp/100 km · rot ' + (v.r * 57.3).toFixed(0) + '°', 12, H - 14, '10px "IBM Plex Mono", monospace', '#9fe0a0', 'rgba(0,0,0,.8)'); }
}
