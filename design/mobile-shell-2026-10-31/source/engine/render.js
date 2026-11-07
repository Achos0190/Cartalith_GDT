/* base raster for the mock map: colour by base mode and ramp, Lambertian hillshade, optional thematic overlay */
const RAMPS = {
  earth: ['#2c4a5e', '#4a7a52', '#8aa05a', '#c9b57a', '#e8e0c8'],
  elevation: ['#1d3557', '#457b9d', '#a8dadc', '#f1faee', '#e63946'],
  atlas: ['#33415c', '#5c677d', '#979dac', '#d9dcd6', '#f5f3ef'],
  mono: ['#111111', '#555555', '#999999', '#dddddd'],
  imhof: ['#5a7d8c', '#8fa98a', '#c9c489', '#e6d3a3', '#f2ead9'],
  ice: ['#274060', '#5a7fa8', '#a8c6de', '#e8f1f8'],
  darkice: ['#0d1b2a', '#1b3a5c', '#3c6e91', '#89b6d5'],
  desert: ['#7a4a2b', '#b07d46', '#d9b380', '#f0e0bd'],
  darkatlas: ['#0e1116', '#232a36', '#48526a', '#8892aa']
};
function hex2rgb(h) { const n = parseInt(h.slice(1), 16); return [(n >> 16) & 255, (n >> 8) & 255, n & 255]; }
function rampAt(stops, t) {
  t = Math.max(0, Math.min(1, t)); const s = stops.map(hex2rgb); const f = t * (s.length - 1); const i = Math.min(s.length - 2, Math.floor(f)); const u = f - i;
  return [s[i][0] + (s[i + 1][0] - s[i][0]) * u, s[i][1] + (s[i + 1][1] - s[i][1]) * u, s[i][2] + (s[i + 1][2] - s[i][2]) * u];
}
function renderBase(w, o, out) {
  const N = w.N, cell = w.cell * 1000;
  const ramp = (RAMPS[o.ramp] || RAMPS.earth).slice(1); // land part of the ramp
  const rampLUT = []; for (let i = 0; i < 256; i++) rampLUT.push(rampAt(ramp, i / 255));
  const ovStops = { elevation: ['#1d3557', '#2a9d8f', '#e9c46a', '#f4a261', '#e76f51', '#ffffff'], slope: ['#f6f1e7', '#f4c27a', '#e07a3f', '#b0302c'], flow: ['#0b1d2c', '#1f5f8b', '#5fb4e6', '#e6f6ff'], temperature: ['#3b4cc0', '#8db0fe', '#f2f0ec', '#f49a7b', '#b40426'], rainfall: ['#a6611a', '#dfc27d', '#f5f5f5', '#80cdc1', '#018571'] }[o.overlay];
  const la = 315 * Math.PI / 180, lalt = 45 * Math.PI / 180, lx = Math.sin(la) * Math.cos(lalt), ly = -Math.cos(la) * Math.cos(lalt), lz = Math.sin(lalt);
  const exag = o.style === 'physical' ? 26 : 18;
  const showShade = o.hillshade !== false, showColour = o.colourRelief !== false;
  for (let j = 0; j < N; j++) for (let i = 0; i < N; i++) {
    const k = j * N + i, z = w.elevM[k];
    const zl = w.elevM[j * N + Math.max(0, i - 1)], zr = w.elevM[j * N + Math.min(N - 1, i + 1)], zu = w.elevM[Math.max(0, j - 1) * N + i], zd = w.elevM[Math.min(N - 1, j + 1) * N + i];
    const gx = Math.max(0, zr) - Math.max(0, zl), gy = Math.max(0, zd) - Math.max(0, zu);
    const nx = -gx * exag / (2 * cell), ny = -gy * exag / (2 * cell), nz = 1; const nl = Math.sqrt(nx * nx + ny * ny + nz * nz);
    let sh = (nx * lx + ny * ly + nz * lz) / nl; sh = Math.max(0, sh);
    let r, g, b;
    const lake = z >= 0 && (w.filled[k] - w.e[k]) > 0.025;
    if (z < 0 || lake) {
      const edge = Math.min(i, j, N - 1 - i, N - 1 - j), ef = Math.min(1, edge / 28);
      const dpt = lake ? 0.2 : Math.min(1, Math.pow(Math.max(0, -z) / 3200, 0.55)) * ef + (1 - ef);
      r = 92 - dpt * 66; g = 138 - dpt * 84; b = 158 - dpt * 84;
      if (o.style === 'parchment') { r = 160 - dpt * 40; g = 170 - dpt * 40; b = 160 - dpt * 30; }
      if (o.style === 'ink') { r = g = b = 236 - dpt * 18; }
      sh = 0.78;
    } else {
      if (o.base === 'biome') { const c = BIOME_COL[biomeOf(w, k)]; r = c[0]; g = c[1]; b = c[2]; }
      else { const c = rampLUT[Math.min(255, Math.max(0, Math.round(Math.pow(z / 4300, 0.62) * 255)))]; r = c[0]; g = c[1]; b = c[2]; }
      if (!showColour) { r = g = b = 196; }
      if (o.base === 'political' && o.owner) { const f = o.owner[k]; if (f >= 0) { const fc = o.facCol[f]; const m = (r + g + b) / 3; r = m * 0.55 + fc[0] * 0.45; g = m * 0.55 + fc[1] * 0.45; b = m * 0.55 + fc[2] * 0.45; } }
      if (o.style === 'parchment') { const m = (r * 0.3 + g * 0.59 + b * 0.11); r = m * 0.55 + 214 * 0.45; g = m * 0.55 + 196 * 0.45; b = m * 0.55 + 156 * 0.45; }
      if (o.style === 'ink') { r = g = b = 238; }
      if (o.style === 'physical') { r *= 1.06; g *= 1.04; }
    }
    let shade = showShade ? (o.style === 'ink' ? 0.55 + sh * 0.55 : 0.42 + sh * 0.78) : 1;
    r *= shade; g *= shade; b *= shade;
    if (ovStops && z >= 0 || (ovStops && o.overlay === 'elevation')) {
      let t = 0;
      if (o.overlay === 'elevation') t = (z + 5200) / 9500;
      else if (o.overlay === 'slope') t = Math.min(1, w.slope[k] / 6);
      else if (o.overlay === 'flow') t = Math.min(1, Math.log(w.acc[k]) / 9);
      else if (o.overlay === 'temperature') t = (w.temp[k] + 25) / 60;
      else if (o.overlay === 'rainfall') t = Math.min(1, w.rain[k] / 2200);
      const c = rampAt(ovStops, t), a = o.ovA == null ? 0.7 : o.ovA;
      r = r * (1 - a) + c[0] * a * (0.75 + sh * 0.3); g = g * (1 - a) + c[1] * a * (0.75 + sh * 0.3); b = b * (1 - a) + c[2] * a * (0.75 + sh * 0.3);
    }
    if (o.base === 'political' && o.owner && z >= 0) {
      const f = o.owner[k]; const fr = i < N - 1 ? o.owner[k + 1] : f, fd = j < N - 1 ? o.owner[k + N] : f;
      if (f !== fr || f !== fd) { r *= 0.35; g *= 0.35; b *= 0.35; }
    }
    const p = k * 4; out[p] = r; out[p + 1] = g; out[p + 2] = b; out[p + 3] = 255;
  }
}
