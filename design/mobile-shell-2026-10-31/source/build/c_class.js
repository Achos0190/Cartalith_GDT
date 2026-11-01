/* ======================================================================================================
   The shell. One component draws every form factor from the frame size it is given (props w, h in dp):
     phone      width < 600 dp            bottom bar MAP · DOMAIN · STORY · MENU, bottom sheet with detents
     phoneLand  height < 500 dp, landscape  the same four destinations on a left rail, the sheet docks right
     tablet     everything else           the desktop shell (menu bar, rail, docks); docks turn into overlay
                                          drawers rather than squeezing the map below MAP_MIN dp
   Every number below is a dp value or a ratio, never a device pixel, so the same rules hold at any density.
   ====================================================================================================== */
const MAP_MIN = 520;          // tablet: narrowest map a docked panel may leave (dp)
const PHONE_MAX_W = 600;      // Material compact width class
const LAND_MAX_H = 500;       // a landscape window shorter than this is a phone in landscape
const NAV_H = 80;             // phone bottom bar: 64 dp of targets + 16 dp gesture inset
const RAIL_W = 80;            // phone landscape rail
const INSET_T = 28;           // status-bar inset: left unpainted, the OS draws there
const PEEK = 76;              // sheet peek: handle + header only
const LONG_MS = 480, LONG_MS_TABLET = 500, NAV_HOLD_MS = 380, DOUBLE_MS = 300, TAP_MS = 280;
// The double-tap and thumb zoom (Ruling EB item 3; MOBILE_SHELL_PROPOSAL.md §2.5). From
// design/dcc-environment-2026-08-31/spec/06-phone.md §7: the second tap within DOUBLE_MS (300 ms) and DOUBLE_SLOP
// (34 px) of the first, a ×DOUBLE_ZOOM (1.7) zoom at the tap, and a single tap resolved after TAP_RESOLVE_MS (240 ms)
// so a double-tap can win. QZ_DOUBLING_DP is how far the held second press slides for the scale to double: a labelled
// judgement for the mockup (none of the sources gives a figure); the build sets its own on glass.
const DOUBLE_SLOP = 34, DOUBLE_ZOOM = 1.7, TAP_RESOLVE_MS = 240, QZ_DOUBLING_DP = 120;
const SMAX = 14;
const DOMS = ['world', 'civil', 'carto'];

class Component extends DCLogic {
  constructor(props) {
    super(props);
    this.rootRef = React.createRef(); this.mapRef = React.createRef(); this.cvRef = React.createRef(); this.noteTaRef = React.createRef(); this.noteMirrorRef = React.createRef(); this.noteReadRef = React.createRef(); this.noteReadScrollRef = React.createRef(); this.noteWriteScrollRef = React.createRef(); this._neHist = {};
    this.sheetRef = React.createRef(); this.grabRef = React.createRef(); this.bodyRef = React.createRef(); this.searchRef = React.createRef();
    this.dockReadRef = React.createRef(); this.dockMirrorRef = React.createRef(); this.dockTaRef = React.createRef(); this.askScrollRef = React.createRef(); this.askInRef = React.createRef();
    this.view = { cx: 1024, cy: 1024, s: 0.2, r: 0 };
    this.ptrs = new Map(); this.g = null; this._dirty = true; this._undoFns = []; this._redoFns = [];
    this.world = null; this.places = []; this.factions = []; this.route = []; this.landmarks = []; this.rivers = []; this.ways = [];
    let coachDone = false; try { coachDone = localStorage.getItem('cartalith.mobile.coach') === 'done'; } catch (e) { coachDone = false; }
    let ctxMods = null; try { const v = JSON.parse(localStorage.getItem('cartalith.desktop.ctxMods') || 'null'); if (Array.isArray(v) && v.includes('inspect') && v.every((x) => modOf(x))) ctxMods = v; } catch (e) { ctxMods = null; }
    this.state = {
      light: !!props.light, picker: props.startIn === 'picker', loading: true,
      tab: null, det: 'half', wide: false,
      dom: 'world', node: { world: 'pipeline', civil: 'factions', carto: 'style' }, tool: 'inspect',
      menuPath: [], storySeg: 'journeys', planStage: -1, open: { 'st-01': true },
      fan: null, holdNav: null, press: null,
      srch: false, q: '', scope: 'all',
      sel: null, sample: null, hist: false, undo: [], redo: [],
      modal: null, win: null, toasts: [],
      base: 'relief', overlay: null, ovA: 0.7, style: 'atlas', ramp: 'earth', dim3: false,
      layers: { labels: true, settlements: true, ways: true, landmarks: true, political: false, water: true, rivers: true, vegetation: true, terrain: true, hillshade: true, colour: true, handdrawn: false, land: true, background: true, grid: false, scale: true },
      layerOp: {}, units: 'km',
      measure: [], wayDraft: [], labelAt: null, labelText: '', stamps: [], userPlaces: [], userLabels: [], userWays: [], userIcons: [], conflictDraft: [],
      // Colour paint marks and the map width they are measured against (d_cats.js, cpSample onwards). 800 km is the
      // width in the map-width notice's approved example sentence, not the stand-in world's 2 048 km: only the ratio of
      // two widths moves a mark.
      cpMarks: [], cpMapKm: 800,
      year: 412, playing: false, speed: 1,
      seed: 483920, running: null, stale: 0, lastRun: '14:02',
      vals: {}, brush: { size: 64, hard: 0.35, inten: 1.0, mode: 'raise' }, feature: 'Mountains', settleTier: 'town', wayKind: 'road', iconKind: 'diamond', lmKind: 'Peak',
      recYears: [-120, 0, 210, 398, 412], tlFilt: { exist: false, ghost: false, halo: false },
      selBack: [], selFwd: [], ctxTab: 'inspect', ctxPin: null, ctxEdit: false, ctxDraft: null, dn: null, ask: { msgs: [], draft: '', off: {} }, winPos: null,
      ctxMods: ctxMods || CTX_CORE.slice(), ctxAdd: false, ctxTabMenu: null, ctxDrag: null, bookmarks: [], caretLink: null, cfm: null, askSent: false,
      dd: null, hamOpen: false, ldockOpen: true, ldockPin: false, rdockOpen: !!props.pointer, layersPop: false, tlOpen: false, tctx: null, ring: null, qd: false, qdDrag: null, vt: null, ac: null, peek: null, neDlg: null, neSaved: 0, notes: null,
      coach: !!props.coach && !coachDone, coachStep: 0,
      savedAt: '14:02', worldName: 'ELDRA',
      quick: ['save', 'export.player', 'story.journeys', 'world.vault'],
      prefs: { theme: 'dark', units: 'km', gpu: true, threads: 12, quality: 'balanced', aa: 'MSAA 4×', lod: true, lodMode: 'auto', undoMB: 256, hand: 'right', assistant: !!props.assistantOn, askSave: null, session: false, diag: false, autosave: true, autoInt: 5, haptics: true, penOnly: false, twoFingerUndo: true, edgeSwipe: true, longPressMs: 0, density: 'auto', textScale: 1, uiScale: 1,
        // Preferences ▸ 3D viewport defaults ▸ relief exaggeration: the 3D height slider's value, remembered from the
        // last session (Ruling EB item 5; v3ExagStored in d_screens.js reads it, guarded, defaulting to 1×).
        v3exag: v3ExagStored() }
    };
  }

  /* ---------------- lifecycle ---------------- */
  componentDidMount() {
    if (this._mode() === 'tablet' && !this._tabletGeom().leftDocks) this.setState({ ldockOpen: false });
    this._raf = requestAnimationFrame(this._loop = () => { if (this._dirty) { this._dirty = false; this._draw(); } this._raf = requestAnimationFrame(this._loop); });
    this._bindMap(); this._bindSheet();
    setTimeout(() => this._makeWorld(this.state.seed, true), 30);
    this._ro = new ResizeObserver(() => { this._dirty = true; }); if (this.mapRef.current) this._ro.observe(this.mapRef.current);
    // pointer (desktop): one key handler for every accelerator the menus show and the context keys (d_desk.js deskKey)
    this._kd = (e) => { if (this._ptr()) deskKey(this, e); }; window.addEventListener('keydown', this._kd);
  }
  componentDidUpdate(pp, ps) {
    this._bindMap(); this._bindSheet(); this._dirty = true;
    const s = this.state;
    if (s.srch && !this._srchWas && this.searchRef.current) { const el = this.searchRef.current; setTimeout(() => { try { el.focus(); } catch (e) { } }, 30); }
    this._srchWas = s.srch;
    const key = [s.tab, s.menuPath.join('/'), s.storySeg, s.planStage, s.dom, s.node[s.dom]].join('|');
    if (this._screenKey !== key) { this._screenKey = key; if (this.bodyRef.current) this.bodyRef.current.scrollTop = 0; }
    if ((pp && (pp.w !== this.props.w || pp.h !== this.props.h)) || (ps && ps.prefs && ps.prefs.uiScale !== this.state.prefs.uiScale)) { this._fitSoon(); if (this._mode() === 'tablet') { const docks = this._tabletGeom().leftDocks; if (docks !== this._lastDocks) { this._lastDocks = docks; this.setState({ ldockOpen: docks }); } } }
    this._vaultDidUpdate(); this._dnDidUpdate();
    const A = s.ask; const last = A && A.msgs.length ? A.msgs[A.msgs.length - 1] : null; const ak = (A ? A.msgs.length : 0) + '|' + (last ? (last.shown || '') + '|' + !!last.pending + '|' + (last.acts || []).map((a) => a.state || '').join() : '');
    if (ak !== this._askKey && this.askScrollRef.current) { this._askKey = ak; const sc = this.askScrollRef.current; sc.scrollTop = sc.scrollHeight; }
    if (this._askFocus && this.askInRef.current) { this._askFocus = false; const el = this.askInRef.current; setTimeout(() => { try { el.focus(); } catch (e) { } }, 30); }
    if (pp && pp.backTick !== this.props.backTick && this.props.backTick) this._system();
    if (pp && pp.light !== this.props.light) this.setState({ light: !!this.props.light, prefs: Object.assign({}, this.state.prefs, { theme: this.props.light ? 'light' : 'dark' }) });
  }
  componentWillUnmount() { cancelAnimationFrame(this._raf); clearInterval(this._runT); clearInterval(this._playT); if (this._ro) this._ro.disconnect(); window.removeEventListener('keydown', this._kd); clearTimeout(this._askT); }

  /* ---------------- frame and size class ---------------- */
  _scale() { return (this.state && this.state.prefs && this.state.prefs.uiScale) || 1; }
  _W() { return Math.max(320, Math.round((+this.props.w || 412) / this._scale())); }
  _H() { return Math.max(320, Math.round((+this.props.h || 892) / this._scale())); }
  _mode() { const W = this._W(), H = this._H(); if (H < LAND_MAX_H && W > H) return 'phoneLand'; if (W < PHONE_MAX_W) return 'phone'; return 'tablet'; }
  _detH(det) {
    const H = this._H(); const full = H - NAV_H - (INSET_T + 4 + 52 + 8);
    if (det === 'peek') return PEEK;
    if (det === 'full') return full;
    return Math.min(full, Math.round((H - NAV_H) * 0.5));
  }
  _sheetW() { const W = this._W(); return this.state.wide ? Math.min(W - RAIL_W - 240, 600) : Math.max(300, Math.min(400, Math.round(W * 0.42))); }
  /* Pointer (desktop): the same shell at mouse density. Only the frame's numbers change; content is shared. */
  _ptr() { return !!this.props.pointer && this._mode() === 'tablet'; }
  _tabletGeom() {
    const W = this._W(), H = this._H(), s = this.state, ptr = this._ptr();
    const menuH = ptr ? 34 : 48, toolH = ptr ? 40 : 52, railW = ptr ? 48 : 64, statusH = ptr ? 24 : 30, tlH = s.tlOpen ? (ptr ? 128 : 156) : (ptr ? 34 : 44);
    const ldW = ptr ? (W >= 1920 ? 340 : (W >= 1600 ? 320 : 300)) : (W >= 1920 ? 400 : (W >= 1400 ? 360 : 340));
    const rdW = ptr ? (W >= 2400 ? 460 : (W >= 1920 ? 420 : (W >= 1600 ? 380 : 352))) : (W >= 1920 ? 380 : (W >= 1400 ? 340 : 312)), rtabW = ptr ? 26 : 44;
    const minMap = ptr ? 560 : MAP_MIN; const avail = W - railW;
    // docking: keep the map at least MAP_MIN wide; a dock that would squeeze it becomes an overlay drawer
    const leftDocks = avail - ldW - rtabW >= minMap || s.ldockPin;
    const rightDocks = leftDocks && (avail - ldW - rdW >= minMap);
    return { W, H, menuH, toolH, railW, statusH, tlH, tlBarH: ptr ? 34 : 44, ldW, rdW, rtabW, leftDocks, rightDocks, bodyT: menuH + toolH, bodyB: tlH + statusH, ptr };
  }
  _L() {
    const W = this._W(), H = this._H(), m = this._mode(), s = this.state;
    if (m === 'phone') {
      const sheetH = s.tab ? this._detH(s.det) : 0;
      return { mode: m, W, H, mapL: 0, mapT: 0, mapR: 0, mapB: NAV_H, appL: 10, appR: 10, appT: INSET_T + 4, scrimH: INSET_T + 84, fabR: 12, compT: INSET_T + 66, scaleL: 14, stripL: 10, stripR: 10, sheetH, sheetBottom: NAV_H, navH: NAV_H };
    }
    if (m === 'phoneLand') {
      const sw = s.tab ? this._sheetW() : 0;
      return { mode: m, W, H, mapL: RAIL_W, mapT: 0, mapR: 0, mapB: 0, appL: RAIL_W + 10, appR: sw + 10, appT: 8, scrimH: 72, fabR: sw + 12, compT: 66, scaleL: RAIL_W + 14, stripL: RAIL_W + 10, stripR: sw + 10, sheetH: 0, sheetW: sw, sheetBottom: 0, navH: 0 };
    }
    const G = this._tabletGeom();
    const mapL = G.railW + (s.ldockOpen && G.leftDocks ? G.ldW : 0);
    const mapR = s.rdockOpen && G.rightDocks ? G.rdW : (G.rightDocks || !s.rdockOpen ? G.rtabW : G.rtabW);
    return { mode: m, W, H, mapL, mapT: G.bodyT, mapR, mapB: G.bodyB, G };
  }
  _fitSoon() { clearTimeout(this._fitT); this._fitT = setTimeout(() => this._fit(false), 40); }

  /* ---------------- world (the stand-in map) ---------------- */
  _makeWorld(seed, fit) {
    const w = genWorld(seed); this.world = w;
    const made = placeWorld(w, seed, this.state.worldName);
    this.places = made.places; this.factions = made.factions; this.route = made.route; this.landmarks = made.landmarks; this.rivers = made.rivers; this.ways = made.ways; this.regions = made.regions; this.ownerGrid = made.owner; this._cmdIdx = null;
    this._renderBase(); this.setState({ loading: false }); if (fit) this._fit(false); this._dirty = true;
  }
  _renderBase() {
    const w = this.world; if (!w) return; const s = this.state;
    if (!this._baseCv) { this._baseCv = document.createElement('canvas'); this._baseCv.width = w.N; this._baseCv.height = w.N; }
    if (this._baseCv.width !== w.N) { this._baseCv.width = w.N; this._baseCv.height = w.N; }
    const ctx = this._baseCv.getContext('2d'); const id = ctx.createImageData(w.N, w.N);
    renderBase(w, { base: s.base, ramp: s.ramp, style: s.style, overlay: s.overlay, ovA: s.ovA, hillshade: s.layers.hillshade, colourRelief: s.layers.colour, owner: this.ownerGrid, facCol: this.factions.map((f) => f.rgb) }, id.data);
    ctx.putImageData(id, 0, 0); this._baseKey = this._lookKey(); this._dirty = true;
    let r = 0, g = 0, b = 0, n = 0; const d = id.data, N = w.N; for (let i = 0; i < N; i += 3) for (const k of [i, (N - 1) * N + i, i * N, i * N + N - 1]) { r += d[k * 4]; g += d[k * 4 + 1]; b += d[k * 4 + 2]; n++; }
    this._oceanFill = 'rgb(' + Math.round(r / n) + ',' + Math.round(g / n) + ',' + Math.round(b / n) + ')';
  }
  _lookKey() { const s = this.state; return [s.base, s.ramp, s.style, s.overlay, s.ovA, s.layers.hillshade, s.layers.colour].join('|'); }

  /* ---------------- view maths (world km <-> local dp) ---------------- */
  _hostWH() { const h = this.mapRef.current; return h ? { W: h.clientWidth, H: h.clientHeight } : { W: 400, H: 700 }; }
  _w2s(x, y, v) { v = v || this.view; const { W, H } = this._hostWH(); const c = Math.cos(v.r), sn = Math.sin(v.r); const dx = (x - v.cx) * v.s, dy = (y - v.cy) * v.s; return { x: W / 2 + dx * c - dy * sn, y: H / 2 + dx * sn + dy * c }; }
  _s2w(px, py, v) { v = v || this.view; const { W, H } = this._hostWH(); const c = Math.cos(v.r), sn = Math.sin(v.r); const ox = (px - W / 2) / v.s, oy = (py - H / 2) / v.s; return { x: v.cx + ox * c + oy * sn, y: v.cy - ox * sn + oy * c }; }
  _sMin() { const { W, H } = this._hostWH(); return Math.min(W, H) / 2048 * 0.6; }
  _anchor(wp, mx, my, s, r) { const { W, H } = this._hostWH(); const c = Math.cos(r), sn = Math.sin(r); const ox = (mx - W / 2) / s, oy = (my - H / 2) / s; this.view = { s, r, cx: wp.x - (ox * c + oy * sn), cy: wp.y - (-ox * sn + oy * c) }; this._dirty = true; }
  _zoomAt(f, px, py) { const v = this.view; const s = Math.max(this._sMin(), Math.min(SMAX, v.s * f)); const wp = this._s2w(px, py); this._anchor(wp, px, py, s, v.r); this._viewChanged(); }
  _zoomAnim(f, px, py) { const v0 = this.view.s, t0 = performance.now(); const step = () => { const k = Math.min(1, (performance.now() - t0) / 240); const e = 1 - Math.pow(1 - k, 3); const want = v0 * (1 + (f - 1) * e); this._zoomAt(want / this.view.s, px, py); if (k < 1) requestAnimationFrame(step); }; requestAnimationFrame(step); }
  _fit(anim) {
    const { W, H } = this._hostWH(); const L = this._L();
    const visH = L.mode === 'phone' ? H - (L.sheetH || 0) - (INSET_T + 60) : H; const visW = L.mode === 'phoneLand' ? W - (L.sheetW || 0) : W;
    const s = Math.max(0.05, Math.min(visW, visH) / 2048 * 0.98);
    const cy = L.mode === 'phone' ? 1024 + ((L.sheetH || 0) - (INSET_T + 60)) / 2 / s : 1024; const cx = L.mode === 'phoneLand' ? 1024 + (L.sheetW || 0) / 2 / s : 1024;
    this.view = { cx, cy, s, r: 0 }; this._dirty = true; this._viewChanged();
  }
  _viewChanged() { clearTimeout(this._vcT); this._vcT = setTimeout(() => this.forceUpdate(), 60); }

  /* ---------------- map gestures ---------------- */
  _bindMap() {
    const host = this.mapRef.current; if (!host || host === this._boundHost) return; this._boundHost = host;
    host.addEventListener('pointerdown', (e) => this._pd(e)); host.addEventListener('pointermove', (e) => this._pm(e));
    host.addEventListener('pointerup', (e) => this._pu(e)); host.addEventListener('pointercancel', (e) => this._pu(e, true));
    host.addEventListener('wheel', (e) => { e.preventDefault(); const p = this._loc(e); this._zoomAt(Math.pow(1.0016, -e.deltaY), p.x, p.y); }, { passive: false });
    host.addEventListener('contextmenu', (e) => { e.preventDefault(); const p = this._loc(e); this._longPress(p.x, p.y, true); });
    host.addEventListener('pointermove', (e) => { if (e.pointerType === 'mouse') this._lastPtr = this._loc(e); });
    host.addEventListener('pointerleave', () => { this._lastPtr = null; });
    if (this._ro) this._ro.observe(host);
  }
  _loc(e) { const host = this.mapRef.current; const r = host.getBoundingClientRect(); const k = host.clientWidth / (r.width || 1); return { x: (e.clientX - r.left) * k, y: (e.clientY - r.top) * k }; }
  _pd(e) {
    const host = this.mapRef.current; try { host.setPointerCapture(e.pointerId); } catch (_) { }
    const p = this._loc(e); this.ptrs.set(e.pointerId, { x: p.x, y: p.y });
    const n = this.ptrs.size;
    if (n === 1) {
      // A held left button pans; right-click and Q open the card and ring.
      const mouseDesk = e.pointerType === 'mouse' && this._ptr();
      // Thumb zoom (Ruling EB item 3): a press that lands while the last tap is still waiting to resolve, within
      // DOUBLE_SLOP of it, is that double-tap's second press. Its hold belongs to thumb zoom, so it starts no
      // long-press and draws no press ring; lifting it without sliding is the plain double-tap (_pu). The window is the
      // TAP_RESOLVE_MS the first tap waits, inside the spec's DOUBLE_MS, so one pair of taps never both acts and zooms.
      // _lastTap is only set while _thumbZoomOn(), so with a drawing tool armed no press is ever a second press.
      const lt = this._lastTap;
      const dbl = !!(lt && this._tapT && e.button === 0 && performance.now() - lt.t < DOUBLE_MS && Math.hypot(p.x - lt.x, p.y - lt.y) < DOUBLE_SLOP);
      clearTimeout(this._tapT); this._tapT = null; this._lastTap = null;
      // A press that is not a second press lets a waiting tap act now, rather than swallowing it.
      if (lt && !dbl) this._tap(lt.x, lt.y);
      this.g = { mode: 'one', t0: performance.now(), x0: p.x, y0: p.y, moved: false, v0: Object.assign({}, this.view), edge: p.x > host.clientWidth - 28, pen: e.pointerType === 'pen', dbl, dblTap: dbl ? lt : null, mouseDesk };
      clearTimeout(this._lpT); if (!mouseDesk && e.button === 0 && !dbl) this._lpT = setTimeout(() => this._longPress(p.x, p.y), this.state.prefs.longPressMs || (this._mode() === 'tablet' ? LONG_MS_TABLET : LONG_MS));
      if (!mouseDesk && e.button === 0 && !dbl && (this._mode() !== 'phone' || this.state.tool === 'inspect')) this.setState({ press: { x: p.x, y: p.y } });
      this._closeTransient();
    } else if (n === 2) {
      clearTimeout(this._lpT); if (this.state.press) this.setState({ press: null });
      // A second finger ends a double-tap's press and hands the view to pinch. If that press had not slid yet, it was the
      // start of a two-finger gesture, not a double-tap: the first tap it held back acts now, so a tap followed by a
      // two-finger tap is still a tap and an undo (Ruling DU Q5).
      const g1 = this.g; if (g1 && g1.mode === 'one' && g1.dbl && !g1.zooming && g1.dblTap) this._tap(g1.dblTap.x, g1.dblTap.y);
      const [a, b] = [...this.ptrs.values()];
      this.g = { mode: 'two', tTwo: performance.now(), d0: Math.hypot(b.x - a.x, b.y - a.y), a0: Math.atan2(b.y - a.y, b.x - a.x), mx: (a.x + b.x) / 2, my: (a.y + b.y) / 2, v0: Object.assign({}, this.view), moved: false, maxN: 2 };
    } else if (this.g) this.g.maxN = Math.max(this.g.maxN || 0, n);
  }
  _pm(e) {
    const q = this.ptrs.get(e.pointerId); if (!q) return; const p = this._loc(e); q.x = p.x; q.y = p.y; const g = this.g; if (!g) return;
    if (g.mode === 'one') {
      if (g.long && this.state.ring && this.state.ring.live) { this._ringHover(p); return; }
      const dx = p.x - g.x0, dy = p.y - g.y0;
      if (!g.moved && Math.hypot(dx, dy) > 10) {
        g.moved = true; clearTimeout(this._lpT); if (this.state.press) this.setState({ press: null });
        // A double-tap's second press that slides is thumb zoom, measured from where the slide begins. With a mouse on
        // the desktop it stays a pan: the hold-slide is a touch gesture.
        if (g.dbl && !g.mouseDesk) { g.zooming = true; g.zy = p.y; }
        else if (g.edge && dx < 0 && Math.abs(dy) < Math.abs(dx) && this.state.prefs.edgeSwipe) g.edgeSwipe = true;
        else if (this._strokeTool() && !this.state.panLatch && !(this.state.prefs.penOnly && !g.pen)) { g.stroke = true; this._strokeStart(this._s2w(g.x0, g.y0)); }
      }
      // Thumb zoom: sliding down zooms in and up zooms out ("Google Maps style", the owner's words, Ruling EB item 3;
      // MAP_INTERACTION_RESEARCH.md §2, where Apple Maps is the reverse), doubling every QZ_DOUBLING_DP, about the
      // point the press landed on, inside the same scale limits as pinch.
      if (g.zooming) { const sc = Math.max(this._sMin(), Math.min(SMAX, g.v0.s * Math.pow(2, (p.y - g.zy) / QZ_DOUBLING_DP))); this._anchor(this._s2w(g.x0, g.y0, g.v0), g.x0, g.y0, sc, g.v0.r); return; }
      if (g.edgeSwipe) { if (dx < -60 && !g.done) { g.done = true; this._openInspector(); } return; }
      if (g.stroke) { this._strokeMove(this._s2w(p.x, p.y)); return; }
      if (g.moved) {
        const v = g.v0, c = Math.cos(v.r), sn = Math.sin(v.r);
        this.view = { s: v.s, r: v.r, cx: v.cx - (dx * c + dy * sn) / v.s, cy: v.cy - (-dx * sn + dy * c) / v.s }; this._dirty = true;
        if (this.state.coach && this.state.coachStep === 0) { this._coachMoves = (this._coachMoves || 0) + 1; }
      }
    } else if (g.mode === 'two' && this.ptrs.size >= 2) {
      const [a, b] = [...this.ptrs.values()]; const d = Math.hypot(b.x - a.x, b.y - a.y), ang = Math.atan2(b.y - a.y, b.x - a.x), mx = (a.x + b.x) / 2, my = (a.y + b.y) / 2;
      if (Math.abs(d - g.d0) > 8 || Math.abs(ang - g.a0) > 0.08 || Math.hypot(mx - g.mx, my - g.my) > 8) g.moved = true;
      if (!g.moved) return;
      const s = Math.max(this._sMin(), Math.min(SMAX, g.v0.s * d / Math.max(20, g.d0)));
      const wp = this._s2w(g.mx, g.my, g.v0); this._anchor(wp, mx, my, s, g.v0.r + (ang - g.a0));
    }
  }
  _pu(e, cancel) {
    const q = this.ptrs.get(e.pointerId); this.ptrs.delete(e.pointerId); const g = this.g; if (!g) return;
    if (g.mode === 'one') {
      clearTimeout(this._lpT); if (this.state.press) this.setState({ press: null });
      if (g.long && this.state.ring && this.state.ring.live) { const R = this.state.ring; if (R.hover) this._ringPick(R.hover); else this.setState({ ring: Object.assign({}, R, { live: false }) }); this.g = null; return; }
      // A double-tap's second press lifted without sliding is the plain double-tap, ×DOUBLE_ZOOM at the finger; one that
      // slid has already zoomed (_pm). Either way it is not a tap.
      if (g.dbl) { if (!cancel && !g.moved && q) this._zoomAnim(DOUBLE_ZOOM, q.x, q.y); }
      else if (g.stroke) this._strokeEnd();
      else if (!cancel && !g.moved && !g.long && !g.done && performance.now() - g.t0 < TAP_MS && q) this._tapCandidate(q);
      if (g.moved) this._viewChanged();
      this.g = null;
    } else if (g.mode === 'two' && this.ptrs.size === 0) {
      if (!cancel && !g.moved && performance.now() - g.tTwo < 300 && this.state.prefs.twoFingerUndo) { if ((g.maxN || 2) >= 3) this._redo(); else this._undo(); }
      this._viewChanged(); this.g = null;
    }
  }
  /* A one-finger tap that did not move. With thumb zoom on it waits TAP_RESOLVE_MS for a second press, which would make
     it a double-tap (_pd decides that when the second press lands). With thumb zoom off nothing can follow it, so it
     goes to the armed tool at once. It must never act twice, or act after a double-tap has claimed it. */
  _tapCandidate(p) {
    clearTimeout(this._tapT); this._tapT = null; this._lastTap = null;
    if (!this._thumbZoomOn()) { this._tap(p.x, p.y); return; }
    this._lastTap = { t: performance.now(), x: p.x, y: p.y };
    this._tapT = setTimeout(() => { this._tapT = null; this._lastTap = null; this._tap(p.x, p.y); }, TAP_RESOLVE_MS);
  }
  /* Whether thumb zoom (the double-tap and its hold-slide, Ruling EB item 3) is on: yes unless a drawing tool is armed.
     With a drawing tool, how it behaves waits on the owner's answer to DS13-Q1 (Ruling EB items 1 and 3). Until then the
     mock keeps it off and labels it "pending DS13-Q1" (the map's accessible name; Touch & gestures), so a double-tap is
     two dabs and nothing collides with a stroke. MAP_INTERACTION_RESEARCH.md §6 sets out the options for the owner;
     this placeholder decides none of them. */
  _thumbZoomOn() { return !this._strokeTool(); }
  _longPress(x, y, mouse, ringOnly) {
    if (this.g && this.g.mode === 'one') { if (this.g.moved) return; this.g.long = true; }
    const wp = this._s2w(x, y); if (!this._inWorld(wp)) return;
    this._haptic();
    const m = this._mode(); const patch = { sample: { x: wp.x, y: wp.y }, press: null };
    if (m === 'tablet') { const L = this._L(); const ptr = this._ptr() && mouse; if (!ringOnly) patch.tctx = { x: x + L.mapL, y: y + L.mapT }; else patch.tctx = null; patch.ring = ptr && !ringOnly ? null : { x: x + L.mapL, y: y + L.mapT, live: !mouse, hover: null, sub: null }; if (ringOnly) delete patch.sample; }
    else { patch.tab = 'ctx'; patch.det = 'half'; }
    if (this.state.coach && this.state.coachStep === 1) this._coachDone();
    this.setState(patch); this._dirty = true;
  }
  _inWorld(p) { return p.x >= 0 && p.y >= 0 && p.x <= 2048 && p.y <= 2048; }
  _haptic() { try { if (this.state.prefs.haptics && navigator.vibrate) navigator.vibrate(12); } catch (e) { } }
  _closeTransient() { const s = this.state; const p = {}; if (this._mode() === 'tablet') { const G = this._tabletGeom(); if (s.ldockOpen && !G.leftDocks) p.ldockOpen = false; if (s.rdockOpen && !G.rightDocks && !s.sel) p.rdockOpen = false; } if (s.layersPop) p.layersPop = false; if (s.dd) p.dd = null; if (s.hamOpen) p.hamOpen = false; if (s.tctx) p.tctx = null; if (s.ring) p.ring = null; if (s.qd) { p.qd = false; p.qdDrag = null; } if (s.ctxAdd) p.ctxAdd = false; if (s.ctxTabMenu) p.ctxTabMenu = null; if (Object.keys(p).length) this.setState(p); }

  /* ---------------- the vault and its note editor ---------------- */
  _vtSet(p) { this.setState({ vt: Object.assign({}, this.state.vt, p) }); }
  _openVault(arg) {
    const s = this.state; notesOf(this);
    const base = { open: true, view: 'list', q: '', filter: 'all', noteId: null, mode: 'write', panel: 'links', back: [], buf: null, dirty: false, sheet: null, conflict: false, side: true };
    const V = Object.assign(base, s.vt || {}, { open: true, sheet: null });
    if (arg === 'without') { V.filter = 'without'; V.view = 'list'; V.q = ''; }
    if (arg === 'browse' || arg === '') { if (!V.noteId) V.view = 'list'; }
    if (arg === 'place') {
      const o = selObj(this);
      if (o) { const n = noteByName(this, o.name); if (n) { V.noteId = n.id; V.view = 'note'; V.mode = V.mode === 'split' ? 'split' : 'write'; this._hist(n.id); } else { this.setState({ vt: V, win: null, dd: null, hamOpen: false, srch: false, qd: false }); this._noteCreate(o.name, o.tier ? 'settlement' : 'landmark', { kind: o.tier ? 'settlement' : 'landmark', id: o.id, name: o.name, sub: o.tier || o.kind || '' }); return; } }
    }
    this.setState({ vt: V, win: null, dd: null, hamOpen: false, srch: false, qd: false, ac: null, peek: null });
  }
  _vaultClose() { this._neFlush(); this.setState({ vt: Object.assign({}, this.state.vt, { open: false, sheet: null }), ac: null, peek: null, neDlg: null }); }
  _noteOpen(id, mode) {
    this._neFlush(); const V = this.state.vt || {}; const n = noteById(this, id) || (this._justMade && this._justMade.id === id ? this._justMade : null); if (!n) return;
    const back = V.noteId && V.noteId !== id ? (V.back || []).concat([V.noteId]).slice(-20) : (V.back || []);
    this._hist(id);
    this.setState({ vt: Object.assign({}, V, { open: true, noteId: id, view: 'note', mode: mode || (V.mode === 'split' ? 'split' : 'write'), back, buf: null, dirty: false, sheet: null, conflict: false }), ac: null, peek: null });
    if (mode === 'write' || mode === 'split') this._neFocusSoon = true;
  }
  _noteBack() {
    const V = this.state.vt; this._neFlush();
    if (V.back && V.back.length) { const id = V.back[V.back.length - 1]; this.setState({ vt: Object.assign({}, V, { noteId: id, back: V.back.slice(0, -1), buf: null, dirty: false, view: 'note', sheet: null, conflict: false }), ac: null }); return; }
    if (this._mode() !== 'tablet') this._vtSet({ view: 'list', noteId: null, buf: null, dirty: false, sheet: null }); else this._vtSet({ noteId: null, buf: null, dirty: false });
  }
  _noteMode(m) { this._neFlush(); this.setState({ vt: Object.assign({}, this.state.vt, { mode: m }), ac: null }); if (m !== 'read') this._neFocusSoon = true; }
  _noteFollow(name, head) {
    const r = resolveLink(this, name);
    if (r.kind === 'note') { this._noteOpen(r.note.id, (this.state.vt && this.state.vt.mode) || 'write'); if (head) this._neJumpHead = head; return; }
    if (r.kind === 'entity') { this.setState({ neDlg: { kind: 'create', name: r.entity.name, entityKind: r.entity.kind, entity: r.entity } }); return; }
    this.setState({ neDlg: { kind: 'create', name: r.name || String(name), entityKind: 'note', entity: null } });
  }
  _noteCreate(name, kind, entity) { this.setState({ neDlg: { kind: 'create', name, entityKind: kind || 'note', entity: entity || null } }); }
  _noteCreateDo(name, kind, entity) {
    const n = createNote(this, name, kind, entity && entity.id ? { kind: entity.kind, id: entity.id, name: entity.name } : null); this._justMade = n;
    this.setState({ neDlg: null }); this._toast('Created ' + name + '.md' + (kind !== 'note' ? ' from the ' + kind + ' template' : '') + (entity ? ', linked to the map' : ''));
    setTimeout(() => this._noteOpen(n.id, 'write'), 0);
  }
  _noteShowOnMap(note) {
    const e = note && (note.entity || entityByName(this, note.name)); if (!e) { this._toast('This note is not linked to anything on the map'); return; }
    this._neFlush();
    const p = (this.places || []).find((q) => q.id === e.id || q.name === e.name);
    if (this._mode() !== 'tablet') this.setState({ vt: Object.assign({}, this.state.vt, { open: false }) }); else this.setState({ vt: Object.assign({}, this.state.vt, { open: false }) });
    if (p) { selectPlace(this, p); return; }
    const lm = (this.landmarks || []).find((q) => q.id === e.id || q.name === e.name);
    if (lm) { this.view = Object.assign({}, this.view, { cx: lm.x, cy: lm.y, s: Math.max(this.view.s, 0.6) }); this._dirty = true; this._toast('Centred on ' + lm.name); return; }
    const fi = FACTIONS.findIndex((f) => f.name === e.name); if (fi >= 0) { this._openWin('factions:' + fi); return; }
    this._toast(e.name + ' is a culture: it has no single place on the map');
  }
  _notePrivate(id) { const notes = notesOf(this).map((n) => (n.id === id ? Object.assign({}, n, { private: !n.private }) : n)); this.setState({ notes }); }
  _noteJump(h) {
    const V = this.state.vt; if (this._mode() !== 'tablet') this._vtSet({ sheet: null });
    if (V.mode === 'read' || V.mode === 'split') { const el = this.noteReadRef.current && this.noteReadRef.current.querySelector('#' + h.id); const sc = this.noteReadScrollRef.current; if (el && sc) sc.scrollTop = el.offsetTop - 8; }
    if (V.mode !== 'read') { const t = this._neText(); const L = t.split('\n'); let off = 0; for (let i = 0; i < h.line && i < L.length; i++) off += L[i].length + 1; this._neSelPending = { a: off, b: off }; const sc = this.noteWriteScrollRef.current; if (sc) sc.scrollTop = Math.max(0, h.line * 21.9 - 40); this.forceUpdate(); }
  }
  _neText() { const V = this.state.vt; if (!V || !V.noteId) return ''; if (V.buf != null) return V.buf; const n = noteById(this, V.noteId); return n ? n.text : ''; }
  _neSet(t, sel, snapNow) { this.setState({ vt: Object.assign({}, this.state.vt, { buf: t, dirty: true }) }); if (sel) this._neSelPending = sel; this._neSnap(t, snapNow); clearTimeout(this._neSaveT); this._neSaveT = setTimeout(() => this._neFlush(), 1100); }
  _neFlush() { clearTimeout(this._neSaveT); const V = this.state.vt; if (!V || !V.dirty || V.buf == null || !V.noteId || V.conflict) return; saveNote(this, V.noteId, V.buf); this.state.vt = Object.assign({}, this.state.vt, { dirty: false }); this.setState({ vt: Object.assign({}, this.state.vt, { dirty: false }) }); }
  _neSnap(t, force) { this._histSnap(this.state.vt.noteId, t, force); }
  _neUndo(redo) { const id = this.state.vt.noteId; const t = this._histStep(id, redo); if (t == null) { this._toast(redo ? 'Nothing to redo' : 'Nothing to undo: this is the note as you first opened it this session'); return; } this.setState({ vt: Object.assign({}, this.state.vt, { buf: t, dirty: true }) }); clearTimeout(this._neSaveT); this._neSaveT = setTimeout(() => this._neFlush(), 1100); }
  _neInput(e) { const ta = e.target; this._neSet(ta.value, null, false); this._neAc(ta); this._caretLink(ta, 'vault'); }
  _neAc(ta) {
    if (!ta) return; const t = ta.value; if (ta.selectionStart !== ta.selectionEnd) { if (this.state.ac) this.setState({ ac: null }); return; }
    const lq = linkQueryAt(t, ta.selectionStart); if (!lq) { if (this.state.ac) this.setState({ ac: null }); return; }
    this.setState({ ac: { from: lq.from, to: lq.to, items: linkSuggestions(this, lq), sel: 0 } });
  }
  _neAcPick(i) {
    const ac = this.state.ac; if (!ac) return; const it = ac.items[i]; if (!it) return;
    const t = this._neText(); const rest = t.slice(ac.to); const close = /^[^\]\n]*\]\]/.exec(rest);
    let nt, caret;
    if (close) { const tail = rest.slice(close[0].indexOf('|') >= 0 ? close[0].indexOf('|') : close[0].length - 2); nt = t.slice(0, ac.from) + '[[' + it.target + tail; caret = ac.from + 2 + it.target.length + (tail.indexOf(']]') + 2); }
    else { nt = t.slice(0, ac.from) + '[[' + it.target + ']]' + rest; caret = ac.from + it.target.length + 4; }
    this.setState({ ac: null }); this._neSet(nt, { a: caret, b: caret }, true);
  }
  _neKey(e) {
    const ac = this.state.ac;
    if (ac && ac.items.length) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); const n = ac.items.length; this.setState({ ac: Object.assign({}, ac, { sel: (ac.sel + (e.key === 'ArrowDown' ? 1 : n - 1)) % n }) }); return; }
      if (e.key === 'Enter' || e.key === 'Tab') { e.preventDefault(); this._neAcPick(ac.sel); return; }
      if (e.key === 'Escape') { e.preventDefault(); this.setState({ ac: null }); return; }
    }
    const ta = e.target, t = ta.value, a = ta.selectionStart, b = ta.selectionEnd;
    if (e.ctrlKey || e.metaKey) {
      const k = e.key.toLowerCase(); const map = { b: 'bold', i: 'italic', u: 'underline', k: 'link', l: 'task', z: e.shiftKey ? 'redo' : 'undo', y: 'redo' };
      if (k === 'e') { e.preventDefault(); this._noteMode(this.state.vt.mode === 'read' ? 'write' : 'read'); return; }
      if (k === 's') { e.preventDefault(); this._neFlush(); this._toast('Saved'); return; }
      if (map[k]) { e.preventDefault(); this._neTx(map[k]); return; }
    }
    if (e.key === '[' && a === b && t.charAt(a - 1) === '[') { e.preventDefault(); this._neSet(t.slice(0, a) + '[]]' + t.slice(b), { a: a + 1, b: a + 1 }, true); this._neAcAfter = true; return; }
    if (e.key === 'Enter' && !e.shiftKey && a === b) {
      const ls = t.lastIndexOf('\n', a - 1) + 1; const line = t.slice(ls, a);
      const m = /^(\s*)([-*+] \[[ xX]\] |[-*+] |(\d+)\. |> )(.*)$/.exec(line);
      if (m) {
        e.preventDefault();
        if (!m[4].trim()) { this._neSet(t.slice(0, ls) + t.slice(a), { a: ls, b: ls }, true); return; }
        const mk = m[3] ? (+m[3] + 1) + '. ' : (/\[[ xX]\]/.test(m[2]) ? m[2].replace(/\[[xX]\]/, '[ ]') : m[2]);
        const ins = '\n' + m[1] + mk; this._neSet(t.slice(0, a) + ins + t.slice(b), { a: a + ins.length, b: a + ins.length }, true); return;
      }
    }
  }
  _neTx(id) {
    const V = this.state.vt; if (!V || !V.noteId) return;
    if (id === 'undo' || id === 'redo') { this._neUndo(id === 'redo'); return; }
    if (id === 'revert') { this._revertAsk(V.noteId, 'vault'); return; }
    if (V.mode === 'read') { this._noteMode('write'); return; }
    if (id === 'template') { this.setState({ neDlg: { kind: 'template' } }); return; }
    const ta = this.noteTaRef.current; const t = this._neText(); const a = ta ? ta.selectionStart : t.length, b = ta ? ta.selectionEnd : t.length;
    const line0 = t.lastIndexOf('\n', a - 1) + 1; const nl = a === line0 ? '' : '\n';
    let r = null;
    switch (id) {
      case 'bold': r = MD_TX.wrap(t, a, b, '**'); break; case 'italic': r = MD_TX.wrap(t, a, b, '*'); break; case 'underline': r = MD_TX.wrap(t, a, b, '<u>', '</u>'); break;
      case 'strike': r = MD_TX.wrap(t, a, b, '~~'); break; case 'highlight': r = MD_TX.wrap(t, a, b, '=='); break; case 'code': r = MD_TX.wrap(t, a, b, '`'); break;
      case 'heading': r = MD_TX.linePrefix(t, a, b, '', true); break; case 'bullet': r = MD_TX.linePrefix(t, a, b, '- '); break; case 'number': r = MD_TX.linePrefix(t, a, b, '1. '); break;
      case 'task': r = MD_TX.linePrefix(t, a, b, '- [ ] '); break; case 'quote': r = MD_TX.linePrefix(t, a, b, '> '); break;
      case 'link': { const sel = t.slice(a, b); r = sel ? MD_TX.insert(t, a, b, '[[|' + sel + ']]', 2) : MD_TX.insert(t, a, b, '[[]]', 2); this._neAcAfter = true; break; }
      case 'tag': { const sel = t.slice(a, b).trim().replace(/\s+/g, '-'); r = MD_TX.insert(t, a, b, '#' + sel); break; }
      case 'callout': r = MD_TX.insert(t, a, b, nl + '> [!note] Title\n> ', (nl + '> [!note] Title\n> ').length); break;
      case 'table': r = MD_TX.insert(t, a, b, nl + '| Column | Column |\n|---|---|\n|  |  |\n'); break;
      case 'rule': r = MD_TX.insert(t, a, b, nl + '---\n'); break;
    }
    if (r) this._neSet(r.text, { a: r.a, b: r.b }, true);
  }
  _neTemplate(kind) {
    const T = { settlement: '### General Info\n\n- **Size / Population:**\n- **Founding date / myth:**\n- **Ruling authority:**\n', landmark: '## Cultural Significance\n- Associated myths or legends:\n- Ritual use or forbidden status:\n', chronos: '```chronos\n- [400] Event | what happened\n= [412] Today\n```\n', faction: '## Relations\n\n| With | Stance | Since |\n|---|---|---|\n|  |  |  |\n' }[kind] || '';
    const ta = this.noteTaRef.current; const t = this._neText(); const a = ta ? ta.selectionStart : t.length;
    this.setState({ neDlg: null }); this._neSet(t.slice(0, a) + T + t.slice(a), { a: a + T.length, b: a + T.length }, true); this._toast('Inserted the ' + kind + ' template');
  }
  _vaultDidUpdate() {
    const s = this.state, V = s.vt; if (!V || !V.open || !V.noteId) return;
    const text = this._neText();
    const rr = this.noteReadRef.current;
    if (rr) { const key = text + '§' + notesOf(this).length + '§' + notesOf(this).map((n) => n.name).join('|'); if (rr !== this._neReadEl || key !== this._neReadKey) { rr.innerHTML = mdRender(text, this).html; this._neReadKey = key; this._neReadEl = rr; this._bindNoteRead(rr); } if (this._neJumpHead) { const h = this._neJumpHead; this._neJumpHead = null; const el = [...rr.querySelectorAll('h1,h2,h3,h4,h5,h6')].find((x) => x.textContent.trim() === h); const sc = this.noteReadScrollRef.current; if (el && sc) sc.scrollTop = el.offsetTop - 8; } }
    const mr = this.noteMirrorRef.current; if (mr && (mr !== this._neMirEl || this._neMirText !== text)) { mr.innerHTML = mdHighlight(text); this._neMirEl = mr; this._neMirText = text; }
    const ta = this.noteTaRef.current;
    if (ta && this._neSelPending) { const p = this._neSelPending; this._neSelPending = null; try { ta.focus({ preventScroll: true }); ta.setSelectionRange(p.a, p.b); } catch (e) { } if (this._neAcAfter) { this._neAcAfter = false; this._neAc(ta); } }
    if (ta && this._neFocusSoon) { this._neFocusSoon = false; setTimeout(() => { try { ta.focus({ preventScroll: true }); } catch (e) { } }, 40); }
  }
  _bindNoteRead(el) {
    if (el._nb) return; el._nb = true; let lp = null, fired = false;
    const toShell = (e) => { const root = this.rootRef.current; const r = root ? root.getBoundingClientRect() : { left: 0, top: 0, width: this._W() }; const k = this._W() / (r.width || 1); return { x: (e.clientX - r.left) * k, y: (e.clientY - r.top) * k }; };
    el.addEventListener('pointerdown', (e) => { fired = false; const a = e.target.closest('[data-link]'); clearTimeout(lp); if (!a) return; const q = toShell(e); lp = setTimeout(() => { fired = true; this._haptic(); this.setState({ peek: { name: a.dataset.link, head: a.dataset.head || null, x: q.x, y: q.y } }); }, 480); });
    ['pointerup', 'pointercancel', 'pointerleave'].forEach((ev) => el.addEventListener(ev, () => clearTimeout(lp)));
    let hov = null; el.addEventListener('pointerover', (e) => { if (e.pointerType !== 'mouse') return; const a = e.target.closest('[data-link]'); clearTimeout(hov); if (!a) return; const q = toShell(e); hov = setTimeout(() => this.setState({ peek: { name: a.dataset.link, head: a.dataset.head || null, x: q.x, y: q.y } }), 650); });
    el.addEventListener('pointerout', () => clearTimeout(hov));
    el.addEventListener('click', (e) => {
      if (fired) { fired = false; e.preventDefault(); return; }
      const a = e.target.closest('[data-link]'); if (a) { clearTimeout(hov); this._noteFollow(a.dataset.link, a.dataset.head || null); return; }
      const tk = e.target.closest('[data-task]'); if (tk) { const L = this._neText().split('\n'); const i = +tk.dataset.task; const m = /^(\s*[-*+]\s+)\[([ xX])\]/.exec(L[i] || ''); if (m) { L[i] = L[i].replace(/^(\s*[-*+]\s+)\[([ xX])\]/, m[1] + (m[2] === ' ' ? '[x]' : '[ ]')); this._neSet(L.join('\n'), null, true); this._haptic(); } return; }
      const g = e.target.closest('[data-tag]'); if (g) { this._vtSet({ view: 'list', q: '#' + g.dataset.tag, filter: 'all', side: true }); return; }
      const y = e.target.closest('[data-year]'); if (y) { const yr = +y.dataset.year; this.setState({ year: yr, tlOpen: true }); this._toast('Timeline moved to year ' + yr); return; }
      const x = e.target.closest('[data-ext]'); if (x) this._toast('Opens ' + x.dataset.ext + ' in your browser');
    });
  }

  /* ---------------- the cartwheel (tablet) ---------------- */
  _ringHover(p) {
    const R = this.state.ring; if (!R || R.sub) return; const L = this._L(); const ctr = ringCentre(this, R);
    const h = ringSlotAt(p.x + L.mapL - ctr.x, p.y + L.mapT - ctr.y);
    if (h !== R.hover) { if (h) this._haptic(); this.setState({ ring: Object.assign({}, R, { hover: h }) }); }
  }
  _ringPick(pos) {
    const R = this.state.ring; const sl = ringSlots(this).find((x) => x.pos === pos); if (!R || !sl) return;
    if (sl.sub) { this.setState({ ring: Object.assign({}, R, { sub: pos, hover: null, live: false }) }); return; }
    this._ringRun(sl.run);
  }
  _ringRun(fn) { this._ringClose(); fn(); this._dirty = true; }
  _ringClose() { this.setState({ ring: null, tctx: null }); }
  /* ---------------- the phone's quick drawer ---------------- */
  _qdDown(e) { const el = e.currentTarget; try { el.setPointerCapture(e.pointerId); } catch (_) { } const r = el.getBoundingClientRect(); this._qd = { x0: e.clientX, k: (el.offsetWidth || 1) / (r.width || 1), open: this.state.qd, moved: false }; }
  _qdMove(e) {
    const q = this._qd; if (!q) return; let dx = (e.clientX - q.x0) * q.k;
    if (this.state.prefs.hand === 'left') dx = -dx;
    if (Math.abs(dx) > 6) q.moved = true; if (!q.moved) return;
    const w = Math.min(QD_W, Math.round(this._W() * 0.78));
    this.setState({ qd: false, qdDrag: Math.max(0, Math.min(w, (q.open ? w : 0) + dx)) });
  }
  _qdUp(e) {
    const q = this._qd; this._qd = null; if (!q) return; const s = this.state;
    if (!q.moved) { this.setState({ qd: !q.open, qdDrag: null }); if (!q.open) this._haptic(); return; }
    const w = Math.min(QD_W, Math.round(this._W() * 0.78));
    this.setState({ qd: (s.qdDrag || 0) > w * 0.4, qdDrag: null });
  }

  /* ---------------- taps and tools on the map ---------------- */
  _hit(px, py) {
    let best = null, bd = 22;
    const all = this.places.concat(this.state.userPlaces);
    for (const pl of all) { const q = this._w2s(pl.x, pl.y); const d = Math.hypot(q.x - px, q.y - py); if (d < bd) { bd = d; best = { kind: 'place', id: pl.id }; } }
    if (!best) for (const lm of this.landmarks) { const q = this._w2s(lm.x, lm.y); const d = Math.hypot(q.x - px, q.y - py); if (d < 18 && d < bd) { bd = d; best = { kind: 'landmark', id: lm.id }; } }
    return best;
  }
  _tap(px, py) {
    const s = this.state, wp = this._s2w(px, py), t = s.tool;
    if (!this._inWorld(wp)) return;
    if (t === 'inspect') {
      const h = this._hit(px, py);
      if (h) this._select(h);
      else if (s.sel || s.sample || s.tab === 'ctx' || s.tab === 'sel') this._deselect();
      this._dirty = true; return;
    }
    const T = toolDef(t);
    if (T && T.onTap) T.onTap(this, wp, px, py);
    this._dirty = true;
  }
  _strokeTool() { const T = toolDef(this.state.tool); return !!(T && T.stroke); }
  _strokeStart(wp) { this._stroke = [wp]; this._dirty = true; }
  _strokeMove(wp) { const a = this._stroke; if (!a) return; const l = a[a.length - 1]; if (Math.hypot(wp.x - l.x, wp.y - l.y) * this.view.s > 10) { a.push(wp); this._dirty = true; } }
  _strokeEnd() { const a = this._stroke; this._stroke = null; const T = toolDef(this.state.tool); if (a && a.length > 1 && T && T.onStroke) T.onStroke(this, a); this._dirty = true; }

  /* ---------------- edit history ---------------- */
  _push(label, undo, redo) { this._undoFns.push({ label, undo, redo }); this._redoFns = []; this.setState({ undo: this._undoFns.map((u) => u.label), redo: [] }); }
  _undo() { const u = this._undoFns.pop(); if (!u) { this._toast('Nothing to undo'); return; } u.undo(); this._redoFns.push(u); this.setState({ undo: this._undoFns.map((x) => x.label), redo: this._redoFns.map((x) => x.label) }); this._toast('Undid · ' + u.label); this._dirty = true; }
  _redo() { const u = this._redoFns.pop(); if (!u) { this._toast('Nothing to redo'); return; } u.redo(); this._undoFns.push(u); this.setState({ undo: this._undoFns.map((x) => x.label), redo: this._redoFns.map((x) => x.label) }); this._toast('Redid · ' + u.label); this._dirty = true; }
  _rollTo(i) { while (this._undoFns.length > i) { const u = this._undoFns.pop(); u.undo(); this._redoFns.push(u); } this.setState({ undo: this._undoFns.map((x) => x.label), redo: this._redoFns.map((x) => x.label), hist: false }); this._dirty = true; }
  _addTo(key, item, label) {
    const add = () => this.setState((st) => ({ [key]: st[key].concat([item]) }), () => { this._dirty = true; });
    const rem = () => this.setState((st) => ({ [key]: st[key].filter((x) => x !== item) }), () => { this._dirty = true; });
    add(); this._push(label, rem, add);
  }

  /* ---------------- toasts ---------------- */
  _toast(text) { const id = Math.random(); this.setState((st) => ({ toasts: st.toasts.concat([{ id, text }]).slice(-2) })); setTimeout(() => this.setState((st) => ({ toasts: st.toasts.filter((t) => t.id !== id) })), 2400); }

  /* ---------------- navigation ---------------- */
  _openTab(t, det) {
    const s = this.state;
    if (s.tab === t && !det) { this.setState({ tab: null }); return; }
    this.setState({ tab: t, det: det || (s.tab && s.det !== 'peek' ? s.det : 'half'), fan: null, hist: false });
  }
  _setDom(d, open) {
    const s = this.state; if (!DOMS.includes(d)) return;
    const patch = { dom: d, fan: null, tool: 'inspect' };
    if (this._mode() !== 'tablet' && open !== false) { patch.tab = 'dom'; patch.det = s.tab && s.det !== 'peek' ? s.det : 'half'; }
    if (this._mode() === 'tablet') patch.ldockOpen = true;
    this.setState(patch); this._haptic();
    if (s.dom !== d) this._toast(domainDef(d).label + (d === 'world' ? ' · generate & sculpt' : d === 'civil' ? ' · landmarks, settlements, ways, journeys' : ' · layers, style, labels, icons'));
    this._dirty = true;
  }
  _cycleDom(dir) { const i = DOMS.indexOf(this.state.dom); this._setDom(DOMS[(i + dir + 3) % 3]); }
  _setNode(n) { const s = this.state; this.setState({ node: Object.assign({}, s.node, { [s.dom]: n }), tool: 'inspect' }); this._dirty = true; }
  _arm(tool) {
    const s = this.state; const T = toolDef(tool);
    if (s.tool === tool || tool === 'inspect') { this.setState({ tool: 'inspect', measure: [], wayDraft: [], labelAt: null, conflictDraft: [], panLatch: false }); this._dirty = true; return; }
    const patch = { tool, measure: [], wayDraft: [], labelAt: null, conflictDraft: [], panLatch: false };
    if (this._mode() !== 'tablet' && s.tab) patch.det = 'peek';
    this.setState(patch); this._haptic(); this._dirty = true;
  }
  _back() {
    const s = this.state;
    if (s.tab === 'menu' && s.menuPath.length) { this.setState({ menuPath: s.menuPath.slice(0, -1) }); return; }
    if (s.tab === 'story' && s.planStage >= 0) { this.setState({ planStage: -1 }); return; }
    if (s.tab === 'sel' || s.tab === 'ctx') { this.setState({ tab: null, sel: null, sample: null }); this._dirty = true; return; }
    this.setState({ tab: null });
  }
  _system() { // Android system back: closes the topmost layer
    const s = this.state;
    if (s.modal) return this.setState({ modal: null });
    if (s.cfm) return this.setState({ cfm: null });
    if (s.ctxAdd || s.ctxTabMenu) return this.setState({ ctxAdd: false, ctxTabMenu: null });
    if (s.fan) return this.setState({ fan: null });
    if (s.qd) return this.setState({ qd: false, qdDrag: null });
    if (s.peek) return this.setState({ peek: null });
    if (s.neDlg) return this.setState({ neDlg: null });
    if (s.ac) return this.setState({ ac: null });
    if (s.vt && s.vt.open) { if (s.vt.sheet) return this._vtSet({ sheet: null }); if (this._mode() !== 'tablet' && s.vt.view === 'note') return this._noteBack(); return this._vaultClose(); }
    if (s.ring) return this.setState({ ring: null, tctx: null });
    if (s.hist) return this.setState({ hist: false });
    if (s.srch) return this.setState({ srch: false });
    if (s.win) return s.win.stack.length > 1 ? this.setState({ win: { stack: s.win.stack.slice(0, -1) } }) : this.setState({ win: null });
    if (s.dd) return this.setState({ dd: null });
    if (s.tctx) return this.setState({ tctx: null });
    if (s.layersPop) return this.setState({ layersPop: false });
    if (s.tool !== 'inspect') return this._arm('inspect');
    if (s.tab) return this._back();
    this._toast('Back again leaves Cartalith (Android system back)');
  }
  _openWin(id) { if (/^vault/.test(id)) return this._openVault(id.split(':')[1] || ''); const s = this.state; const stack = s.win ? s.win.stack.concat([id]) : [id]; this.setState({ win: { stack }, dd: null, hamOpen: false, srch: false }); }
  _openInspector() { if (this._mode() === 'tablet') this.setState({ rdockOpen: true }); else { const s = this.state; this.setState({ tab: s.sel ? 'sel' : 'ctx', det: 'half', sample: s.sample || this._s2w(this._hostWH().W / 2, this._hostWH().H / 2) }); } this._dirty = true; }
  /* ---------------- the one selection, the one year (PC_SHELL_PROPOSAL.md §3) ----------------
     Every producer calls _select: the map, Find, the card, window rows, note links and assistant citations.
     It replaces the shipped pair of holders (RightDock and CivilizationWorkspace._selected_index, finding S1). */
  _select(sel, o) {
    o = o || {}; const s = this.state; const same = selKey(sel) === selKey(s.sel); this._dnFlush();
    const patch = { sample: null, tctx: null };
    if (!same) { patch.sel = sel; patch.selBack = s.sel ? (s.selBack || []).concat([s.sel]).slice(-30) : (s.selBack || []); if (!o.keepFwd) patch.selFwd = []; patch.ctxEdit = false; patch.ctxDraft = null; }
    if (o.year != null) patch.year = Math.max(YEAR_MIN, Math.min(YEAR_MAX, Math.round(o.year)));
    if (o.tab) patch.ctxTab = o.tab;
    if (this._mode() === 'tablet') patch.rdockOpen = true; else if (sel && !o.quiet) { patch.tab = 'sel'; patch.det = s.det === 'full' ? 'full' : 'half'; }
    this.setState(patch); if (o.frame) this._frameSel(sel); this._dirty = true;
  }
  _deselect() {
    const s = this.state; this._dnFlush(); const patch = { sel: null, sample: null, ctxEdit: false, ctxDraft: null, tctx: null };
    if (s.sel) { patch.selBack = (s.selBack || []).concat([s.sel]).slice(-30); patch.selFwd = []; }
    if (this._mode() !== 'tablet' && (s.tab === 'ctx' || s.tab === 'sel')) patch.tab = null;
    this.setState(patch); this._dirty = true;
  }
  _selBack() { const s = this.state, b = s.selBack || []; if (!b.length) { this._toast('No earlier selection'); return; } const prev = b[b.length - 1]; this._dnFlush(); this.setState({ sel: prev, selBack: b.slice(0, -1), selFwd: s.sel ? (s.selFwd || []).concat([s.sel]) : (s.selFwd || []), ctxEdit: false, sample: null, rdockOpen: this._mode() === 'tablet' ? true : s.rdockOpen }); this._frameSel(prev); }
  _selFwd() { const s = this.state, f = s.selFwd || []; if (!f.length) { this._toast('Nothing to go forward to'); return; } const nx = f[f.length - 1]; this._dnFlush(); this.setState({ sel: nx, selFwd: f.slice(0, -1), selBack: s.sel ? (s.selBack || []).concat([s.sel]) : (s.selBack || []), ctxEdit: false, sample: null }); this._frameSel(nx); }
  _frameSel(x) {
    const e = x && x.sub != null ? x : entOf(this, x); if (!e || e.x == null) return;
    const sc = e.kind === 'faction' ? Math.min(Math.max(this.view.s, 0.22), 0.42) : (e.kind === 'conflict' ? Math.max(this.view.s, 0.32) : Math.max(this.view.s, 0.6));
    this.view = Object.assign({}, this.view, { cx: e.x, cy: e.y, s: sc }); this._dirty = true; this._viewChanged();
  }
  /* The one year writer (the port's DccShell.tl_set_year): the strip, the CIVIL timeline, note chronos rows,
     the assistant's citations and every "go to" link call this, so nothing moves the year behind the others (T1). */
  _setYear(y) { const yy = Math.max(YEAR_MIN, Math.min(YEAR_MAX, Math.round(+y))); if (isNaN(yy)) return; if (yy !== this.state.year) this.setState({ year: yy }); this._dirty = true; }
  _ctxTab(tab, focus) {
    const m = this._mode();
    if (m !== 'tablet') { if (tab === 'note') this._openVault(hasSel(this) ? 'place' : 'browse'); else if (tab === 'ask') this._openWin('assistant'); else this._openInspector(); return; }
    const mods = ctxModsOf(this); const next = this._ctxEnsure(tab); if (next !== mods) this._ctxModsSave(next);
    this.setState({ ctxTab: tab, rdockOpen: true, ctxMods: next, ctxAdd: false, ctxTabMenu: null }); if (tab === 'ask' && (focus || this._ptr())) this._askFocus = true;
  }
  _planFrom(o) { this.route = [o].concat(this.route.slice(1).filter((p) => p !== o)); doAct(this, { act: 'story', arg: 'journeys' }); this._toast((this._ptr() ? 'Planner window open · the map stays live' : 'Planner open beside the map') + ' · start ' + o.name + ' · select a settlement to add it'); this._dirty = true; }
  _applyPlaceEdit(o, d) {
    const before = { name: o.name, tier: o.tier, fac: o.fac, fort: !!o.fort }; const after = { name: String(d.name || '').trim() || o.name, tier: d.tier || o.tier, fac: d.fac != null ? d.fac : o.fac, fort: !!d.fort };
    const put = (v) => { Object.assign(o, v); this._cmdIdx = null; this._conflicts = null; this._dirty = true; this.forceUpdate(); };
    put(after); this._push('edit · ' + after.name, () => put(before), () => put(after));
    this.setState({ ctxEdit: false, ctxDraft: null }); this._toast('Saved ' + after.name + ' · Ctrl+Z undoes it');
  }
  _openVaultAt(id) { this._dnFlush(); this._openVault('browse'); setTimeout(() => this._noteOpen(id), 0); }
  /* ---------------- the Context panel's Note tab: the subject's note, read or written in place ---------------- */
  _dnNote() { const e = entOf(this, subjectOf(this)); return e ? noteOfSubject(this, e) : null; }
  _dnText(n) { const d = this.state.dn; return d && n && d.id === n.id && d.buf != null ? d.buf : (n ? n.text : ''); }
  _dnMode(m) { const n = this._dnNote(); if (!n) return; this._dnFlush(); this.setState({ dn: { id: n.id, mode: m, buf: null, dirty: false } }); if (m === 'write') this._dnFocusSoon = true; }
  _dnInput(ev) { const n = this._dnNote(); if (!n) return; this._histSnap(n.id, ev.target.value, false); this.setState({ dn: { id: n.id, mode: 'write', buf: ev.target.value, dirty: true } }); clearTimeout(this._dnT); this._dnT = setTimeout(() => this._dnFlush(), 900); this._caretLink(ev.target, 'dock'); }
  _dnUndo(redo) { const n = this._dnNote(); if (!n) return; const t = this._histStep(n.id, redo); if (t == null) { this._toast(redo ? 'Nothing to redo' : 'Nothing to undo: this is the note as you first opened it this session'); return; } this.setState({ dn: { id: n.id, mode: 'write', buf: t, dirty: true } }); clearTimeout(this._dnT); this._dnT = setTimeout(() => this._dnFlush(), 900); }
  _dnKey(e) {
    const ctl = e.ctrlKey || e.metaKey; if (!ctl) return; const k = e.key.toLowerCase(); const ta = e.target;
    if (k === 'e') { e.preventDefault(); this._dnMode('read'); return; }
    if (k === 's') { e.preventDefault(); this._dnFlush(); this._toast('Saved'); return; }
    if (k === 'z' || k === 'y') { e.preventDefault(); this._dnUndo(k === 'y' || e.shiftKey); return; }
    const wrap = { b: '**', i: '*' }[k]; if (wrap || k === 'k') { e.preventDefault(); const t = ta.value, a = ta.selectionStart, b = ta.selectionEnd; const sel = t.slice(a, b); const ins = k === 'k' ? '[[' + sel + ']]' : wrap + sel + wrap; const nt = t.slice(0, a) + ins + t.slice(b); this._dnInput({ target: { value: nt } }); setTimeout(() => { try { ta.setSelectionRange(a + ins.length, a + ins.length); } catch (_) { } }, 0); }
  }
  _dnFlush() { clearTimeout(this._dnT); const d = this.state.dn; if (!d || !d.dirty || d.buf == null) return; saveNote(this, d.id, d.buf); this.state.dn = Object.assign({}, d, { dirty: false }); this.setState({ dn: Object.assign({}, d, { dirty: false }) }); }
  _dnCreate(e) {
    if (!e) return; const ent = e.kind === 'place' ? { kind: 'settlement', id: e.id, name: e.name } : (['landmark', 'faction', 'culture'].includes(e.kind) ? { kind: e.kind, id: e.id, name: e.name } : null);
    const n = createNote(this, e.noteName, e.entityKind === 'note' ? 'note' : e.entityKind, ent);
    this._toast('Created ' + n.name + '.md' + (ent ? ', linked to this ' + SEL_KINDS[e.kind][0].toLowerCase() : ''));
    setTimeout(() => this.setState({ dn: { id: n.id, mode: 'write', buf: null, dirty: false }, ctxTab: 'note' }), 0); this._dnFocusSoon = true;
  }
  _dnDidUpdate() {
    const n = this.state.rdockOpen ? this._dnNote() : null;
    const rr = this.dockReadRef.current;
    if (rr && n) {
      const text = this._dnText(n); const key = n.id + '§' + text + '§' + notesOf(this).map((x) => x.name).join('|');
      if (rr !== this._dnReadEl || key !== this._dnReadKey) { rr.innerHTML = mdRender(text, this).html; this._dnReadKey = key; this._dnReadEl = rr; this._bindDockRead(rr); }
      // the shared year: mark the chronos rows the timeline is inside
      const y = this.state.year; const rows = [...rr.querySelectorAll('.md-chr-row[data-year]')]; let best = -Infinity; rows.forEach((b) => { const y0 = +b.dataset.year; if (y0 <= y && y0 > best) best = y0; }); rows.forEach((b) => b.classList.toggle('now', +b.dataset.year === best));
    }
    const mr = this.dockMirrorRef.current; if (mr && n) { const text = this._dnText(n); if (mr !== this._dnMirEl || this._dnMirText !== text) { mr.innerHTML = mdHighlight(text); this._dnMirEl = mr; this._dnMirText = text; } }
    const ta = this.dockTaRef.current; if (ta && this._dnFocusSoon) { this._dnFocusSoon = false; setTimeout(() => { try { ta.focus({ preventScroll: true }); } catch (e) { } }, 40); }
  }
  _bindDockRead(el) {
    if (el._nb) return; el._nb = true;
    el.addEventListener('click', (e) => {
      const a = e.target.closest('[data-link]');
      if (a) { this._dockFollow(a.dataset.link); return; }
      const tk = e.target.closest('[data-task]'); if (tk) { const n = this._dnNote(); if (!n) return; const L = this._dnText(n).split('\n'); const i = +tk.dataset.task; const m = /^(\s*[-*+]\s+)\[([ xX])\]/.exec(L[i] || ''); if (m) { L[i] = L[i].replace(/^(\s*[-*+]\s+)\[([ xX])\]/, m[1] + (m[2] === ' ' ? '[x]' : '[ ]')); saveNote(this, n.id, L.join('\n')); } return; }
      const y = e.target.closest('[data-year]'); if (y) { this._setYear(+y.dataset.year); return; }
      const g = e.target.closest('[data-tag]'); if (g) { this._openVault('browse'); setTimeout(() => this._vtSet({ view: 'list', q: '#' + g.dataset.tag, filter: 'all', side: true }), 0); return; }
      const x = e.target.closest('[data-ext]'); if (x) this._toast('Opens ' + x.dataset.ext + ' in your browser');
    });
  }
  /* ---------------- Ask: the assistant docked in the Context panel ---------------- */
  _askSend(text) {
    const s = this.state; const A = s.ask || { msgs: [], off: {} }; const q = String(text != null ? text : A.draft || '').trim(); if (!q || A.busy) return;
    if (!s.prefs.assistant) { this._toast('The assistant is off · Preferences ▸ Assistant'); return; }
    const k = askCtx(this); const chips = askChips(this).filter((x) => !x.off && !x.fixed).map((x) => x.label);
    const user = { role: 'user', parts: [{ t: 'text', s: q }], ctx: chips.length ? 'sent with · ' + chips.join(' · ') : 'sent with no context' };
    this.setState({ ask: Object.assign({}, A, { msgs: (A.msgs || []).concat([user, { role: 'assistant', parts: [], pending: true }]), draft: '', busy: true }) });
    const r = askReply(this, q, k); clearTimeout(this._askT);
    this._askT = setTimeout(() => {
      const A2 = this.state.ask; const msgs = A2.msgs.slice(); const i = msgs.length - 1;
      msgs[i] = { role: 'assistant', parts: r.parts, acts: r.acts, tools: r.tools, srcs: r.srcs, shown: 0, pending: false }; this.setState({ ask: Object.assign({}, A2, { msgs }) });
      const step = () => { const A3 = this.state.ask; const m3 = A3.msgs.slice(); const mm = Object.assign({}, m3[i]); mm.shown = Math.min(r.parts.length, (mm.shown || 0) + 2); const done = mm.shown >= r.parts.length; if (done) delete mm.shown; m3[i] = mm; this.setState({ ask: Object.assign({}, A3, { msgs: m3, busy: !done }) }); if (!done) this._askT = setTimeout(step, 60); };
      step();
    }, 620);
  }
  _askAct(mi, ai, apply) {
    const A = this.state.ask; const msgs = A.msgs.slice(); const m = Object.assign({}, msgs[mi]); const acts = (m.acts || []).slice(); const a = Object.assign({}, acts[ai]); if (a.state) return;
    if (apply) { try { a.apply(this); } catch (err) { this._toast('Could not apply: ' + err.message); } a.state = { setting: 'Applied · Generate to see it · Ctrl+Z undoes it', view: 'Applied', note: 'Applied · Ctrl+Z undoes it · the Note tab shows it' }[a.kind] || 'Applied'; }
    else a.state = 'Discarded · nothing changed';
    acts[ai] = a; m.acts = acts; msgs[mi] = m; this.setState({ ask: Object.assign({}, A, { msgs }) });
  }
  _askNoteFor(e) {
    let n = noteOfSubject(this, e); if (n) return { n, made: false };
    const ent = e.kind === 'place' ? { kind: 'settlement', id: e.id, name: e.name } : (['landmark', 'faction', 'culture'].includes(e.kind) ? { kind: e.kind, id: e.id, name: e.name } : null);
    n = createNote(this, e.noteName, e.entityKind === 'note' ? 'note' : e.entityKind, ent); return { n, made: true };
  }
  _askAppend(e, md) {
    const r = this._askNoteFor(e); const n = r.n; const add = md + '\n\n*Drafted by the assistant. Review it before keeping it.*\n';
    if (r.made) { n.text = n.text.replace(/\s*$/, '') + '\n\n' + add; this._toast('Created ' + n.name + '.md with the text'); return; }
    const before = n.text; const after = before.replace(/\s*$/, '') + '\n\n' + add; saveNote(this, n.id, after); this._push('assistant · ' + n.name + '.md', () => saveNote(this, n.id, before), () => saveNote(this, n.id, after));
  }
  _askAppendChronos(e, line) {
    const r = this._askNoteFor(e); const n = r.n; const before = n.text; let after;
    const m = /```chronos\n([\s\S]*?)```/.exec(before);
    if (m) after = before.slice(0, m.index) + '```chronos\n' + m[1] + line + '\n```' + before.slice(m.index + m[0].length);
    else after = before.replace(/\s*$/, '') + '\n\n```chronos\n' + line + '\n```\n';
    if (r.made) { n.text = after; return; }
    saveNote(this, n.id, after); this._push('assistant · chronos in ' + n.name, () => saveNote(this, n.id, before), () => saveNote(this, n.id, after));
  }
  /* ---------------- the desktop window host: non-modal, movable, following the selection ---------------- */
  _winDragDown(e) { if (e.target && e.target.closest && e.target.closest('button')) return; const el = e.currentTarget; try { el.setPointerCapture(e.pointerId); } catch (_) { } const root = this.rootRef.current; const r = root.getBoundingClientRect(); const k = this._W() / (r.width || 1); const w = this._winBox || { x: 0, y: 0 }; this._wd = { k, x0: e.clientX, y0: e.clientY, bx: w.x, by: w.y }; }
  _winDragMove(e) { const d = this._wd; if (!d) return; const nx = Math.round(d.bx + (e.clientX - d.x0) * d.k), ny = Math.round(d.by + (e.clientY - d.y0) * d.k); this.setState({ winPos: { x: Math.max(-200, Math.min(this._W() - 120, nx)), y: Math.max(0, Math.min(this._H() - 60, ny)) } }); }
  _winDragUp() { this._wd = null; }
  _coachDone() { try { localStorage.setItem('cartalith.mobile.coach', 'done'); } catch (e) { } this.setState({ coach: false }); }

  /* ---------------- the Context panel's modules (Ruling DY, PQ1: "Tabs you arrange") ----------------
     The person's tabs are `ctxMods`, in their order; saved with their settings (this browser here, the build's
     per-user settings file there), never in the world file. Ctrl+1/2/3 stay Inspect, Note and Ask whatever the
     order (PQ8), and bring the tab back if it was removed. Inspect cannot be removed: it anchors the panel. */
  _ctxModsSave(mods) { try { localStorage.setItem('cartalith.desktop.ctxMods', JSON.stringify(mods)); } catch (e) { } }
  _ctxSetMods(mods, patch) { this.setState(Object.assign({ ctxMods: mods }, patch || {})); this._ctxModsSave(mods); }
  /* A core tab that comes back goes after the core tab before it in Inspect · Note · Ask, wherever the person has
     moved that one; any other module goes at the end. */
  _ctxEnsure(id) {
    const mods = ctxModsOf(this); if (mods.includes(id)) return mods; const out = mods.slice(); const at = CTX_CORE.indexOf(id);
    if (at < 0) return out.concat([id]);
    const prev = CTX_CORE.slice(0, at).reverse().find((x) => out.includes(x)); out.splice(prev ? out.indexOf(prev) + 1 : 0, 0, id); return out;
  }
  _ctxModSet(id, on) {
    const m = modOf(id); if (!m) return; const mods = ctxModsOf(this);
    if (on) {
      if (mods.includes(id)) { this.setState({ ctxTab: id, rdockOpen: true }); return; }
      const e = entOf(this, subjectOf(this)); const fits = modFits(m, subjKind(e));
      this._ctxSetMods(mods.concat([id]), { ctxTab: fits ? id : this.state.ctxTab, rdockOpen: true });
      this._toast(m.name + ' added' + (fits ? '' : ' · it shows for ' + modKindsText(m)));
      return;
    }
    if (id === 'inspect') { this._toast('Inspect anchors the panel and stays'); return; }
    if (!mods.includes(id)) return;
    this._ctxSetMods(mods.filter((x) => x !== id), { ctxTab: this.state.ctxTab === id ? 'inspect' : this.state.ctxTab, ctxTabMenu: null });
    this._toast(m.name + ' removed · + or Window ▸ Context panel brings it back');
  }
  /* Move one place left or right among the tabs shown now, so the move is always visible; hidden tabs keep their slots. */
  _ctxMove(id, dir) {
    const kind = subjKind(entOf(this, subjectOf(this))); const mods = ctxModsOf(this).slice(); const vis = mods.filter((x) => modFits(modOf(x), kind));
    const vi = vis.indexOf(id); const nb = vis[vi + dir]; if (vi < 0 || !nb) return;
    mods.splice(mods.indexOf(id), 1); mods.splice(mods.indexOf(nb) + (dir > 0 ? 1 : 0), 0, id); this._ctxSetMods(mods, { ctxTabMenu: null });
  }
  _ctxDrop(id, over) {
    const mods = ctxModsOf(this).slice(); const a = mods.indexOf(id), b = mods.indexOf(over); if (a < 0 || b < 0 || a === b) return;
    mods.splice(a, 1); mods.splice(mods.indexOf(over) + (a < b ? 1 : 0), 0, id); this._ctxSetMods(mods);
  }
  _ctxReset() { this._ctxSetMods(CTX_CORE.slice(), { ctxTab: 'inspect', ctxTabMenu: null, ctxAdd: false }); this._toast('Context panel reset · Inspect · Note · Ask'); }
  _tabClick(id) { if (this._tabNoClick) { this._tabNoClick = false; return; } this._ctxTab(id); }
  /* Pointer: drag a tab sideways to move it. Touch: hold a tab for its menu (the strip scrolls sideways under a finger). */
  _tabDown(id, e) {
    if (e.button != null && e.button !== 0) return; clearTimeout(this._tabLp);
    if (e.pointerType === 'touch') { this._tabT = { id, x0: e.clientX, y0: e.clientY, el: e.currentTarget }; this._tabLp = setTimeout(() => { const t = this._tabT; if (!t) return; this._tabNoClick = true; this._haptic(); this._tabMenuAt(t.id, { currentTarget: t.el }); this._tabT = null; }, LONG_MS_TABLET); return; }
    this._tabD = { id, x0: e.clientX, active: false }; try { e.currentTarget.setPointerCapture(e.pointerId); } catch (_) { }
  }
  _tabMove(e) {
    const t = this._tabT; if (t && Math.hypot(e.clientX - t.x0, e.clientY - t.y0) > 10) { clearTimeout(this._tabLp); this._tabT = null; }
    const d = this._tabD; if (!d) return; if (!d.active && Math.abs(e.clientX - d.x0) < 7) return; d.active = true;
    const rn = e.currentTarget && e.currentTarget.getRootNode ? e.currentTarget.getRootNode() : document; const hit = (rn.elementFromPoint ? rn : document).elementFromPoint(e.clientX, e.clientY);
    const el = hit && hit.closest ? hit.closest('[data-ctxtab]') : null; const over = el ? el.getAttribute('data-ctxtab') : null;
    const cur = this.state.ctxDrag; if (!cur || !cur.active || cur.over !== over) this.setState({ ctxDrag: { id: d.id, over, active: true } });
  }
  _tabUp() {
    clearTimeout(this._tabLp); this._tabT = null; const d = this._tabD; this._tabD = null; if (!d || !d.active) return;
    const over = this.state.ctxDrag && this.state.ctxDrag.over; this._tabNoClick = true; this.setState({ ctxDrag: null });
    if (over && over !== d.id) { this._ctxDrop(d.id, over); this._toast((modOf(d.id) || {}).name + ' moved'); }
    setTimeout(() => { this._tabNoClick = false; }, 0);
  }
  _tabCancel() { clearTimeout(this._tabLp); this._tabT = null; this._tabD = null; if (this.state.ctxDrag) this.setState({ ctxDrag: null }); }
  _tabMenuAt(id, e) {
    if (e && e.preventDefault) e.preventDefault(); const el = e && e.currentTarget; let x = 6;
    if (el && el.parentElement) x = el.offsetLeft - el.parentElement.scrollLeft;
    this.setState({ ctxTabMenu: { id, x }, ctxAdd: false });
  }
  /* ---------------- the session's undo history per note (VQ1, Ruling DY) ----------------
     One history per note, shared by the vault's editor and the panel's Note tab (one editor). It starts at the note as
     it was the first time it was opened this session; closing and reopening the note keeps it; nothing of it is saved,
     so quitting (here: reloading) clears it. Undo can always walk back to that first-open state, because the trim that
     keeps the history short never drops the first entry. */
  _hist(id) {
    let H = this._neHist[id]; if (H) return H; const n = noteById(this, id); const t = n ? n.text : '';
    H = this._neHist[id] = { base: t, at0: hm(), stack: [t], idx: 0, at: 0 }; return H;
  }
  _histChanged(id) { const H = this._neHist[id]; if (!H) return false; const n = noteById(this, id); const d = this.state.dn; const cur = d && d.id === id && d.buf != null ? d.buf : (this.state.vt && this.state.vt.noteId === id && this.state.vt.buf != null ? this.state.vt.buf : (n ? n.text : '')); return cur !== H.base; }
  _histSnap(id, t, force) {
    const H = this._hist(id); const now = performance.now(); if (H.stack[H.idx] === t) return;
    if (!force && now - H.at < 700 && H.idx > 0) H.stack[H.idx] = t; else { H.stack = H.stack.slice(0, H.idx + 1).concat([t]); if (H.stack.length > 60) H.stack = [H.stack[0]].concat(H.stack.slice(-59)); H.idx = H.stack.length - 1; }
    H.at = now;
  }
  _histStep(id, redo) { const H = this._hist(id); const j = H.idx + (redo ? 1 : -1); if (j < 0 || j >= H.stack.length) return null; H.idx = j; H.at = 0; return H.stack[j]; }
  /* VQ6 (Ruling DY): the old Cancel is "Revert to opened…". It asks first and says what goes; the revert is itself one
     step of the same history, so Ctrl/Cmd+Z brings the edits back while the session lasts. */
  _revertAsk(id, where) {
    const n = noteById(this, id); if (!n) return; const H = this._hist(id); if (!this._histChanged(id)) { this._toast('Nothing to revert: ' + n.name + '.md is as you first opened it'); return; }
    const cur = where === 'dock' ? this._dnText(n) : this._neText(); const dw = wordCount(cur) - wordCount(H.base); const dl = cur.split('\n').length - H.base.split('\n').length;
    const what = (dw ? Math.abs(dw) + ' word' + (Math.abs(dw) === 1 ? '' : 's') + (dw > 0 ? ' added' : ' removed') : 'edits that keep the word count') + (dl ? ', ' + Math.abs(dl) + ' line' + (Math.abs(dl) === 1 ? '' : 's') + (dl > 0 ? ' added' : ' removed') : '');
    this.setState({ cfm: { title: 'REVERT TO OPENED', text: 'Put ' + n.name + '.md back as it was when you first opened it this session, at ' + H.at0 + '? This drops your changes since then (' + what + '). Ctrl+Z brings them back while this session lasts.', ok: 'REVERT', danger: true, run: () => this._revertDo(id, where) } });
  }
  _revertDo(id, where) {
    const H = this._hist(id); const n = noteById(this, id); if (!n) return; this.setState({ cfm: null });
    if (where === 'dock') { this._histSnap(id, this._dnText(n), true); this._histSnap(id, H.base, true); this.setState({ dn: { id, mode: (this.state.dn && this.state.dn.mode) || 'write', buf: H.base, dirty: true } }, () => this._dnFlush()); }
    else { this._histSnap(id, this._neText(), true); this._histSnap(id, H.base, true); this.setState({ vt: Object.assign({}, this.state.vt, { buf: H.base, dirty: true }) }, () => this._neFlush()); }
    this._toast('Reverted ' + n.name + '.md to ' + H.at0 + ' · Ctrl+Z brings your changes back');
  }
  /* VQ4 (Ruling DY): notes open in Write, so a link is followed from the source: Ctrl/Cmd+click on a pointer, and an
     "Open ›" chip on touch while the caret sits inside [[…]]. */
  _linkAt(text, pos) { const a = text.lastIndexOf('[[', pos); if (a < 0) return null; const b = text.indexOf(']]', a); if (b < 0 || pos > b + 2 || pos < a) return null; const inner = text.slice(a + 2, b); if (/\n/.test(inner)) return null; const name = inner.split('|')[0].split('#')[0].trim(); return name || null; }
  _caretLink(ta, where) { const name = ta && ta.selectionStart === ta.selectionEnd ? this._linkAt(ta.value, ta.selectionStart) : null; const cur = this.state.caretLink; if ((cur && cur.name) !== name || (cur && cur.where) !== where) this.setState({ caretLink: name ? { name, where } : null }); }
  _writeClick(e, where) {
    const ta = e.target; if ((e.ctrlKey || e.metaKey) && ta) { const name = this._linkAt(ta.value, ta.selectionStart); if (name) { e.preventDefault(); this._followFromWrite(name, where); return; } }
    this._caretLink(ta, where);
  }
  _followFromWrite(name, where) {
    const w = where || (this.state.caretLink && this.state.caretLink.where) || 'vault'; this.setState({ caretLink: null });
    if (w === 'vault') { this._neFlush(); this._noteFollow(name); return; }
    this._dnFlush(); this._dockFollow(name);
  }
  /* The panel's links select what they name. A name with no note and nothing on the map is never created silently
     (VQ2): the create card offers a template first. */
  _dockFollow(name) {
    const sl = selFromName(this, name); if (sl) { this._select(sl, { frame: true }); return; }
    this._dockCreateAsk(name);
  }
  _dockCreateAsk(name) {
    const make = (kind) => () => { const n = createNote(this, name, kind, null); this.setState({ cfm: null }); this._toast('Created ' + n.name + '.md' + (kind !== 'note' ? ' from the ' + kind + ' template' : '')); setTimeout(() => this._select({ kind: 'note', id: n.id }, { tab: 'note' }), 0); };
    this.setState({ cfm: { title: 'NEW NOTE · ' + String(name).toUpperCase(), text: 'No note or place is called “' + name + '” yet. Create the note? Pick a template; nothing is made until you do.', opts: [['Settlement', 'settlement'], ['Landmark', 'landmark'], ['Faction', 'faction'], ['Region overview', 'region'], ['Blank', 'note']].map(([label, k]) => ({ label, tap: make(k) })) } });
  }

  /* ---------------- generation (mock run: the real one is the engine's) ---------------- */
  _run(from) {
    if (this.state.running) return;
    const names = STAGE_NAMES; let i = Math.max(0, (from || 1) - 1), pct = 0; const log = [];
    this.setState({ running: { i, pct: 0, log: [] } });
    this._runT = setInterval(() => {
      pct += 16 + Math.random() * 26;
      if (pct >= 100) { log.push(String(i + 1).padStart(2, '0') + ' ' + names[i] + ' · resolved · ' + (0.3 + Math.random() * 2).toFixed(1) + ' s'); i++; pct = 0; }
      if (i >= names.length) {
        clearInterval(this._runT); const hhmm = hm();
        this.setState({ running: null, stale: 0, lastRun: hhmm });
        this._makeWorld(this.state.seed, false);
        this._toast('World regenerated · stages ' + String((from || 1)).padStart(2, '0') + ' → ' + names.length + ' resolved');
        return;
      }
      this.setState({ running: { i, pct, log: log.slice(-3) } });
    }, 220);
  }
  _cancelRun() { clearInterval(this._runT); this.setState({ running: null }); this._toast('Generation cancelled · nothing was written'); }
  _markStale(stage) { const s = this.state; this.setState({ stale: s.stale ? Math.min(s.stale, stage) : stage }); }
  _setVal(key, v, stage) { const s = this.state; this.setState({ vals: Object.assign({}, s.vals, { [key]: v }) }); if (stage) this._markStale(stage); }
  _val(key, d) { const v = this.state.vals[key]; return v == null ? d : v; }

  /* ---------------- timeline ---------------- */
  _play() {
    if (this.state.playing) { clearInterval(this._playT); this.setState({ playing: false }); return; }
    this.setState({ playing: true });
    this._playT = setInterval(() => { const s = this.state; const y = Math.min(YEAR_MAX, s.year + s.speed); this._setYear(y); if (y >= YEAR_MAX) { clearInterval(this._playT); this.setState({ playing: false }); } }, 500);
  }

  /* ---------------- sheet drag (phone) ---------------- */
  _bindSheet() {
    const grab = this.grabRef.current; if (!grab || grab === this._boundGrab) return; this._boundGrab = grab;
    grab.addEventListener('pointerdown', (e) => this._sd(e)); grab.addEventListener('pointermove', (e) => this._sm(e));
    grab.addEventListener('pointerup', (e) => this._su(e)); grab.addEventListener('pointercancel', (e) => this._su(e));
  }
  _sd(e) {
    if (e.target.closest && e.target.closest('button')) return;
    const m = this._mode(); const el = this.sheetRef.current; if (!el) return;
    try { e.currentTarget.setPointerCapture(e.pointerId); } catch (_) { }
    const r = el.getBoundingClientRect(); const k = el.offsetWidth / (r.width || 1);
    this._sdrag = { m, k, y0: e.clientY, x0: e.clientX, h0: m === 'phone' ? this._detH(this.state.det) : this._sheetW(), moved: false };
    el.style.transition = 'none';
  }
  _sm(e) {
    const d = this._sdrag; if (!d) return; const el = this.sheetRef.current; if (!el) return;
    if (d.m === 'phone') { const dy = (e.clientY - d.y0) * d.k; if (Math.abs(dy) > 6) d.moved = true; const h = Math.max(40, Math.min(this._detH('full') + 20, d.h0 - dy)); el.style.height = h + 'px'; d.h = h; }
    else { const dx = (e.clientX - d.x0) * d.k; if (Math.abs(dx) > 6) d.moved = true; const w = Math.max(120, Math.min(this._W() - RAIL_W - 200, d.h0 - dx)); el.style.width = w + 'px'; d.h = w; }
  }
  _su() {
    const d = this._sdrag; this._sdrag = null; const el = this.sheetRef.current; if (!d || !el) return;
    el.style.transition = '';
    if (!d.moved) { // a tap on the handle row toggles between half and full
      if (d.m === 'phone') this.setState({ det: this.state.det === 'full' ? 'half' : 'full' });
      el.style.height = ''; el.style.width = ''; return;
    }
    if (d.m === 'phone') {
      el.style.height = '';
      if (d.h < 52) { this.setState({ tab: null }); return; }
      const cands = ['peek', 'half', 'full']; let best = 'half', bd = 1e9; for (const c of cands) { const dd = Math.abs(this._detH(c) - d.h); if (dd < bd) { bd = dd; best = c; } }
      this.setState({ det: best });
    } else {
      el.style.width = '';
      const narrow = Math.max(300, Math.min(400, Math.round(this._W() * 0.42))); const wide = Math.min(this._W() - RAIL_W - 240, 600);
      if (d.h < narrow * 0.6) { this.setState({ tab: null }); return; }
      this.setState({ wide: Math.abs(d.h - wide) < Math.abs(d.h - narrow) });
    }
  }

  /* ---------------- bottom-bar / rail cells: tap, hold for the domain fan, swipe to cycle ---------------- */
  _navDown(id, e) {
    try { e.currentTarget.setPointerCapture(e.pointerId); } catch (_) { }
    this._nav = { id, x0: e.clientX, y0: e.clientY, t0: performance.now(), moved: false, fan: false };
    if (id === 'dom') {
      clearTimeout(this._navT); this.setState({ holdNav: 'dom' });
      this._navT = setTimeout(() => { if (!this._nav || this._nav.moved) return; this._nav.fan = true; this._haptic(); this._openFan('dom', e.currentTarget); }, NAV_HOLD_MS);
    }
  }
  _navMove(e) {
    const n = this._nav; if (!n) return; const dx = e.clientX - n.x0, dy = e.clientY - n.y0;
    if (n.fan || this.state.fan) { const el = document.elementFromPoint(e.clientX, e.clientY); const f = el && el.closest ? el.closest('[data-fan]') : null; const hv = f ? +f.getAttribute('data-fan') : -1; if (this.state.fan && this.state.fan.hover !== hv) this.setState({ fan: Object.assign({}, this.state.fan, { hover: hv, slid: true }) }); return; }
    if (!n.moved && Math.hypot(dx, dy) > 12) { n.moved = true; clearTimeout(this._navT); if (this.state.holdNav) this.setState({ holdNav: null }); n.swipe = Math.abs(dx) > Math.abs(dy) ? Math.sign(dx) : 0; n.swipeUp = dy < -24 && Math.abs(dy) > Math.abs(dx); }
  }
  _navUp(id, e) {
    const n = this._nav; this._nav = null; clearTimeout(this._navT); if (this.state.holdNav) this.setState({ holdNav: null });
    if (!n) return;
    const f = this.state.fan;
    if (n.fan && f) { if (f.slid && f.hover >= 0) { f.items[f.hover] && f.items[f.hover].pick(); } return; }
    if (n.moved) {
      if (id === 'dom' && n.swipe) { this._cycleDom(-n.swipe); return; }
      if (n.swipeUp) { this.setState({ tab: id, det: 'full' }); return; }
      return;
    }
    this._openTab(id);
  }
  _navCancel() { this._nav = null; clearTimeout(this._navT); if (this.state.holdNav) this.setState({ holdNav: null }); }
  _openFan(kind, anchorEl) {
    const root = this.rootRef.current; let ax = this._W() / 2, ay = this._H() - NAV_H;
    if (!anchorEl) this._fanAnchor = { ax: this._W() / 2, ay: this._mode() === 'phone' ? this._H() - NAV_H - this._detH(this.state.det) + 76 : this._H() / 2 };
    if (anchorEl && root) { const rr = root.getBoundingClientRect(), r = anchorEl.getBoundingClientRect(); const k = root.offsetWidth / (rr.width || 1); ax = (r.left + r.width / 2 - rr.left) * k; ay = (r.top - rr.top) * k; this._fanAnchor = { ax, ay, ar: (r.right - rr.left) * k, ab: (r.bottom - rr.top) * k }; }
    const items = kind === 'dom' ? DOMS.map((d, i) => ({ i, d, pick: () => this._setDom(d) })) : this._toolFanItems();
    const cur = kind === 'dom' ? DOMS.indexOf(this.state.dom) : items.findIndex((x) => x.on);
    this.setState({ fan: { kind, hover: cur, slid: false, items } });
  }
  _toolFanItems() {
    const s = this.state;
    return paletteFor(this).map((t, i) => ({ i, tool: t, on: s.tool === t, pick: () => { this.setState({ fan: null }); this._arm(t); } }));
  }
  _toolDown(e) { try { e.currentTarget.setPointerCapture(e.pointerId); } catch (_) { } const el = e.currentTarget; this._tl = { t0: performance.now() }; clearTimeout(this._tlT); this._tlT = setTimeout(() => { if (this._tl) { this._tl.fan = true; this._haptic(); this._openFan('tool', el); } }, NAV_HOLD_MS); }
  _toolUp(e) {
    clearTimeout(this._tlT); const t = this._tl; this._tl = null; if (!t) return;
    const f = this.state.fan;
    if (t.fan && f) { const el = document.elementFromPoint(e.clientX, e.clientY); const fe = el && el.closest ? el.closest('[data-fan]') : null; if (fe) { const it = f.items[+fe.getAttribute('data-fan')]; if (it) it.pick(); } return; }
    // tap: open the tool's options (the domain sheet) at half
    this.setState({ tab: 'dom', det: 'half' });
  }

  /* ---------------- map drawing ---------------- */
  _draw() {
    const cv = this.cvRef.current, host = this.mapRef.current; if (!cv || !host) return;
    const W = host.clientWidth, H = host.clientHeight; if (!W || !H) return;
    let dpr = Math.min(2, window.devicePixelRatio || 1) * this._scale(); if (W * H * dpr * dpr > 8e6) dpr = Math.sqrt(8e6 / (W * H)); // a 4K window keeps an ~8 MP backing store
    if (cv.width !== Math.round(W * dpr) || cv.height !== Math.round(H * dpr)) { cv.width = Math.round(W * dpr); cv.height = Math.round(H * dpr); }
    const ctx = cv.getContext('2d'); ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const s = this.state;
    ctx.fillStyle = this._oceanFill || '#1a364a'; ctx.fillRect(0, 0, W, H);
    if (!this.world || !this._baseCv) return;
    if (this._baseKey !== this._lookKey()) this._renderBase();
    drawMap(this, ctx, W, H);
  }

  /* ---------------- render values ---------------- */
  renderVals() { return buildVals(this); }
}
