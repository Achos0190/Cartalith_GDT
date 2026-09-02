//! The site model — reference lines **28549-28741**, three functions.
//!
//! `buildSite` is the input contract for everything downstream: it fixes the
//! 1700 × 1250 m box the whole town is laid out in, decides where the water is,
//! and hands back the five field closures (`height`, `slope`, `riverDist`,
//! `isWater`, `bankSide`) that anchors, routes, growth, walls, parcels and
//! buildings all query. Nothing later in this subsystem re-derives any of it.
//!
//! # The two paths, and why the branch state is not one enum
//!
//! There are really two sites here wearing one name:
//!
//! - the **synthetic** one, which invents a river or a coastline out of the
//!   seed alone (`stream(seed,'site')`) and an analytic three-hill height
//!   proxy, and
//! - the **real-raster** one, which the host app actually uses: `opts.water`
//!   carries the map's own water mask, its distance transform and the river
//!   centreline, and `opts.terrain` carries the map's own heightfield. On that
//!   path `height` is a bilinear sample of real relief and `isWater` is a mask
//!   lookup — the hills and the analytic terrace are computed and then never
//!   read.
//!
//! Which of the two is live is decided per *field*, not per site: a real water
//! mask with no river centreline still runs the synthetic hills, and a real
//! heightfield with no water context still runs the synthetic channel. So the
//! branch state is carried as `Option<WaterCtx>` / `Option<TerrainCtx>` on
//! [`Site`] rather than as one source enum, which would have to lie about the
//! mixed cases the host actually produces.
//!
//! # `kind` stays a string
//!
//! The same call milestone 2 made about `Edge::cls`. The reference's vocabulary
//! looks closed (`river`, `riverthrough`, `bay`, `coast`, `landlocked`) but is
//! not: `kind = kind || 'river'` defaults only the *falsy* case, and every
//! unrecognised string falls through to the coastline branch while still being
//! returned verbatim — and milestone 9 compares `site.kind === 'coast'`
//! directly, so an unknown kind and a real coast are genuinely different sites
//! downstream. An enum would have to collapse them. Golden-pinned by the
//! `atoll` scenario.
//!
//! # Closures that outlive their construction
//!
//! `buildSite` calls its own `slope` — hence its own `height`, hence
//! `riverDist` and `isWater` — while it is still choosing the bridge point.
//! The port therefore builds a [`Site`] whose field-closure inputs (hills,
//! river, `rk`, the two rasters) are final *before* the bridge/quay/water-polygon
//! block runs, and fills the rest in afterwards. That is the same order the JS
//! closures see, not a rearrangement.
//!
//! # JS semantics that are load-bearing here
//!
//! Every distance goes through [`js_hypot`] and every clamp through [`js_min`]
//! / [`js_max`], for the reasons milestones 1-4 established. This milestone
//! adds a third: **JS reads out of bounds and gets `undefined`**, which then
//! poisons arithmetic to `NaN` and fails every `===`. The port reproduces that
//! rather than panicking — a `NaN` probe point, a `dt` shorter than its mask,
//! and a one-column terrain raster all reach it, and all three are goldens.

use crate::geom::{Vec2, chaikin, dist_pt_seg, js_exp, js_hypot, js_max, js_min, js_num_cmp, js_or};
use crate::rng::stream;

#[cfg(test)]
mod tests;

/// JS `x || d` for a field the reference may simply not have set at all.
///
/// `buildSite` leans on the plain [`js_or`] five times (`riverWidthM || 20`,
/// `riverOrder || 0`, `seaLakeCells || 0`, `hMax || 0`, `hMin || 0`) and
/// [`terrain_suitability`] a sixth (`site.riverW || 0`); this is the same thing
/// one level out, for an absent field rather than a falsy one.
fn js_or_opt(v: Option<f64>, d: f64) -> f64 {
    match v {
        Some(x) => js_or(x, d),
        None => d,
    }
}

/// `opts.water` — the host app's real water context for this site box.
///
/// `mask` is tested two different ways by the reference and this port keeps
/// both: [`shore_from_mask`] takes any non-zero cell as water (JS truthiness),
/// while [`Site::is_water`] tests `=== 1`. A cell holding `2` is therefore
/// water to the shoreline tracer and land to the water query. Reproduced, not
/// unified — see the `maskTwo` golden.
#[derive(Debug, Clone, Default)]
pub struct WaterCtx {
    /// Row-major `mh × mw`; non-zero is water for the shoreline tracer, `1` is
    /// water for [`Site::is_water`].
    pub mask: Vec<u8>,
    /// Distance transform, in cells; `riverDist` scales it by `cell_m`.
    pub dt: Vec<f64>,
    pub mw: usize,
    pub mh: usize,
    pub cell_m: f64,
    /// The river centreline in local box metres. `Some` — **including
    /// `Some(vec![])`** — is what makes the site river-like (`!!W.riverPath`);
    /// a path shorter than two points is river-like but still traces its water
    /// from the mask. Both halves are goldens.
    pub river_path: Option<Vec<Vec2>>,
    /// `W.riverWidthM || 20`.
    pub river_width_m: Option<f64>,
    /// Strahler order of the real river stem; `|| 0`.
    pub river_order: f64,
    /// Open-water (sea/lake, pre-river-stamp) cell count; `|| 0`.
    pub sea_lake_cells: f64,
}

/// `opts.terrain` — the host app's real heightfield for this site box.
#[derive(Debug, Clone, Default)]
pub struct TerrainCtx {
    /// Row-major `mh × mw` heights.
    pub grid: Vec<f64>,
    pub mw: usize,
    pub mh: usize,
    pub cell_m: f64,
    pub h_min: f64,
    pub h_max: f64,
}

/// `opts.economy` — carried through `buildSite` untouched and read by
/// milestones 13 and 15 (`site.economy.specialisation`,
/// `site.economy.oreBearing`). Nothing in this milestone looks inside it; the
/// synthetic path leaves it `None`, which is exactly what the reference's own
/// headless suite does.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Economy {
    pub specialisation: Option<String>,
    pub ore_bearing: bool,
}

/// `buildSite`'s `opts`. The reference's `opts` also carries `routeEnds` and
/// `primaryPaths`, which `buildSite` never reads — those are milestone 6's.
#[derive(Debug, Clone, Default)]
pub struct SiteOpts {
    pub water: Option<WaterCtx>,
    pub terrain: Option<TerrainCtx>,
    pub economy: Option<Economy>,
}

/// One of the three analytic hills of the synthetic height proxy. Drawn even
/// on the real-terrain path — the four draws per hill are part of the site
/// substream's call order whether or not anything ever reads them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hill {
    pub x: f64,
    pub y: f64,
    pub amp: f64,
    pub rad: f64,
}

/// `site.harbour`. `idx` is `-1` and `pt` is `None` on a landlocked site;
/// otherwise `pt` is `river[idx] || river[0] || null`, so an index past the end
/// of the centreline falls back to its first point rather than to nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Harbour {
    pub idx: i64,
    pub pt: Option<Vec2>,
}

/// `buildSite`'s return value — the plain facts, plus the five field queries as
/// methods.
#[derive(Debug, Clone)]
pub struct Site {
    /// Verbatim, after `kind || 'river'`. See the module docs on why this is
    /// not an enum.
    pub kind: String,
    pub through: bool,
    pub no_water: bool,
    pub wm: f64,
    pub hm: f64,
    /// The river centreline, or the shoreline polyline on a coastal site, or
    /// the far dummy pair on a landlocked one.
    pub river: Vec<Vec2>,
    pub river_w: f64,
    pub water_poly: Vec<Vec2>,
    pub bridge_pt: Option<Vec2>,
    pub bridge_dir: Option<Vec2>,
    /// `routeEnds` — the external route endpoints on the box edge. Four on a
    /// river-like or landlocked site, three on a sea one (it trades seaward).
    pub route_ends: Vec<Vec2>,
    pub uses_real_water: bool,
    pub real_river: bool,
    pub uses_real_terrain: bool,
    pub terrain_relief: f64,
    pub water_order: f64,
    pub sea_lake_cells: f64,
    pub economy: Option<Economy>,
    pub harbour: Harbour,

    // --- closure state: captured by the JS closures, not on its return object ---
    hills: [Hill; 3],
    /// `rk` — river-*like*: a channel (real or synthetic) rather than a
    /// half-plane sea. Selects the synthetic `isWater` band, suppresses
    /// `height`'s sea step, and picks the four-endpoint `routeEnds` set.
    rk: bool,
    water: Option<WaterCtx>,
    terrain: Option<TerrainCtx>,
}

impl Site {
    /// The three analytic hills, exposed for tests and for anyone auditing the
    /// site substream's draw order.
    pub fn hills(&self) -> &[Hill; 3] {
        &self.hills
    }
    /// `rk` — whether this site's water is a channel rather than a half-plane.
    pub fn river_like(&self) -> bool {
        self.rk
    }
    pub fn water_ctx(&self) -> Option<&WaterCtx> {
        self.water.as_ref()
    }
    pub fn terrain_ctx(&self) -> Option<&TerrainCtx> {
        self.terrain.as_ref()
    }
}

/// `maskIdx` — `Math.max(0, Math.min(mw-1, Math.floor(p.x/cellM)))` on both
/// axes, then `j*mw+i`.
///
/// `None` stands for JS's `undefined`, which this reaches two ways: a `NaN`
/// coordinate (the clamp propagates it, and `arr[NaN]` is `undefined`), and a
/// raster shorter than `mw*mh`. The clamp itself cannot go out of range except
/// when `mw` or `mh` is zero, where JS computes `mw-1 === -1` and then indexes
/// an empty array with `0` — also `undefined`.
fn mask_idx(w: &WaterCtx, p: Vec2) -> Option<usize> {
    let i = js_max(0.0, js_min(w.mw as f64 - 1.0, (p.x / w.cell_m).floor()));
    let j = js_max(0.0, js_min(w.mh as f64 - 1.0, (p.y / w.cell_m).floor()));
    if i.is_nan() || j.is_nan() || i < 0.0 || j < 0.0 {
        return None;
    }
    Some(j as usize * w.mw + i as usize)
}

impl Site {
    /// `riverDistSynth` — nearest distance to the centreline polyline.
    ///
    /// Seeded with `Infinity` and folded with `Math.min`, so a `NaN` probe
    /// point poisons it to `NaN` rather than being absorbed — hence [`js_min`],
    /// not `f64::min`. A polyline of fewer than two points runs no iterations
    /// and returns `Infinity`, exactly as `i < river.length-1` does in JS.
    fn river_dist_synth(&self, p: Vec2) -> f64 {
        let mut d = f64::INFINITY;
        for i in 0..self.river.len().saturating_sub(1) {
            d = js_min(d, dist_pt_seg(p, self.river[i], self.river[i + 1]));
        }
        d
    }

    /// `site.riverDist(p)` — the real distance transform where there is one,
    /// the synthetic polyline distance otherwise.
    pub fn river_dist(&self, p: Vec2) -> f64 {
        match &self.water {
            Some(w) => match mask_idx(w, p).and_then(|i| w.dt.get(i)) {
                Some(v) => v * w.cell_m,
                // JS `undefined * cellM`
                None => f64::NAN,
            },
            None => self.river_dist_synth(p),
        }
    }

    /// `yAtX` — the shoreline's y at a given x, by linear interpolation between
    /// the two straddling vertices and flat outside the ends. Only ever reached
    /// on the synthetic-coast path, which is the only branch that defines it.
    fn y_at_x(&self, x: f64) -> f64 {
        let c = &self.river;
        if x <= c[0].x {
            return c[0].y;
        }
        for i in 0..c.len().saturating_sub(1) {
            if x <= c[i + 1].x {
                // `(c[i+1].x - c[i].x) || 1` — a duplicated vertex would
                // otherwise divide by zero.
                let denom = js_or(c[i + 1].x - c[i].x, 1.0);
                let t = (x - c[i].x) / denom;
                return c[i].y + t * (c[i + 1].y - c[i].y);
            }
        }
        c[c.len() - 1].y
    }

    /// `site.isWater(p)`.
    ///
    /// Four branches, in the reference's own ternary order: the real mask wins
    /// whenever there is one, then a landlocked site is dry everywhere, then a
    /// channel is a band of `riverW/2 + 2` around the centreline, and a sea is
    /// the half-plane one metre below the shoreline.
    pub fn is_water(&self, p: Vec2) -> bool {
        if let Some(w) = &self.water {
            return mask_idx(w, p).and_then(|i| w.mask.get(i)).copied() == Some(1);
        }
        if self.no_water {
            return false;
        }
        if self.rk {
            return self.river_dist_synth(p) < self.river_w / 2.0 + 2.0;
        }
        p.y > self.y_at_x(p.x) - 1.0
    }

    /// `site.height(p)`.
    ///
    /// With a real heightfield this is a bilinear sample of it and nothing
    /// else — no invented hills, no synthetic terrace, because the real field
    /// already carries the real valley. Without one it is the analytic proxy:
    /// an inland rise, three seeded Gaussian hills, a terrace cut down toward
    /// the water, and a further step down over open sea.
    ///
    /// The four bilinear terms are summed in the reference's own order; that
    /// sum is not associative in `f64`, and every downstream slope, route cost
    /// and building gate reads the result.
    pub fn height(&self, p: Vec2) -> f64 {
        if let Some(t) = &self.terrain {
            let gx = js_max(0.0, js_min(t.mw as f64 - 1.001, p.x / t.cell_m - 0.5));
            let gy = js_max(0.0, js_min(t.mh as f64 - 1.001, p.y / t.cell_m - 0.5));
            if gx.is_nan() || gy.is_nan() {
                // JS: `grid[NaN]` is `undefined`, and the sum of four
                // `undefined`-derived terms is `NaN`.
                return f64::NAN;
            }
            let (i0, j0) = (gx.floor(), gy.floor());
            let (fx, fy) = (gx - i0, gy - j0);
            let i1 = js_min(t.mw as f64 - 1.0, i0 + 1.0);
            let j1 = js_min(t.mh as f64 - 1.0, j0 + 1.0);
            let at = |j: f64, i: f64| -> f64 {
                if j < 0.0 || i < 0.0 {
                    return f64::NAN;
                }
                t.grid.get(j as usize * t.mw + i as usize).copied().unwrap_or(f64::NAN)
            };
            return at(j0, i0) * (1.0 - fx) * (1.0 - fy)
                + at(j0, i1) * fx * (1.0 - fy)
                + at(j1, i0) * (1.0 - fx) * fy
                + at(j1, i1) * fx * fy;
        }
        let mut h = 0.4 + 0.00008 * (self.hm - p.y);
        for hl in &self.hills {
            let d = js_hypot(p.x - hl.x, p.y - hl.y);
            h += hl.amp * js_exp(-(d * d) / (2.0 * hl.rad * hl.rad));
        }
        // gentle terrace down to the water: the valley floor / shore flat is
        // buildable prime land
        let rd = self.river_dist(p);
        h -= 0.04 * js_exp(-(rd * rd) / (2.0 * 180.0 * 180.0));
        if !self.rk && self.is_water(p) {
            h -= 0.05;
        }
        h
    }

    /// `site.slope(p)` — a central difference at ±8 m, scaled by 900 to a
    /// grade-like magnitude. The reference's own comment calls the scaling
    /// schematic; [`terrain_suitability`]'s falloff and `buildPrimaries`'
    /// Tobler penalty are both tuned against these units, so it is not a free
    /// parameter.
    pub fn slope(&self, p: Vec2) -> f64 {
        let e = 8.0;
        let hx = (self.height(Vec2::new(p.x + e, p.y)) - self.height(Vec2::new(p.x - e, p.y)))
            / (2.0 * e);
        let hy = (self.height(Vec2::new(p.x, p.y + e)) - self.height(Vec2::new(p.x, p.y - e)))
            / (2.0 * e);
        js_hypot(hx, hy) * 900.0
    }

    /// `let bi=0,bd=Infinity; for(…) if(d<bd){bd=d;bi=i;}` — the index of the
    /// centreline segment nearest `p`.
    ///
    /// Strict `<` from `Infinity`, so the **first** of several equidistant
    /// segments wins and a `NaN` distance never wins at all; an empty or
    /// single-vertex centreline gives `0`, which is the reference's own answer.
    ///
    /// The reference writes this loop twice — once inside `bankSide` (line
    /// 28696) and once inside `buildDetails`' log-boom branch (line 30872),
    /// which then *also* calls `bankSide` and so walks it a second time. Both
    /// call sites read it here now; what they do **not** share is how they
    /// resolve the far end, and that difference is the reference's, not a
    /// port artefact: `bankSide` writes `river[bi+1] || river[bi]` and
    /// `buildDetails` writes `river[Math.min(bi+1, river.length-1)]`. The two
    /// agree on every input, and each caller keeps its own.
    pub fn nearest_river_seg(&self, p: Vec2) -> usize {
        let mut bi = 0usize;
        let mut bd = f64::INFINITY;
        for i in 0..self.river.len().saturating_sub(1) {
            let d = dist_pt_seg(p, self.river[i], self.river[i + 1]);
            if d < bd {
                bd = d;
                bi = i;
            }
        }
        bi
    }

    /// `site.bankSide(p)` — which side of the nearest centreline segment a
    /// point lies on, as `+1` or `-1`.
    ///
    /// `Math.sign(x) || 1` never returns `0`: a point exactly on the line, a
    /// `-0` cross product and a `NaN` one all come back `+1`. `grow`'s
    /// bridgehead rule and `buildWall`'s far-bank test both read it, so the
    /// on-the-line case having a definite answer is load-bearing rather than
    /// incidental.
    pub fn bank_side(&self, p: Vec2) -> f64 {
        let bi = self.nearest_river_seg(p);
        let a = self.river[bi];
        let b = self.river.get(bi + 1).copied().unwrap_or(a);
        let s = (b - a).cross(p - a);
        if s > 0.0 {
            1.0
        } else if s < 0.0 {
            -1.0
        } else {
            1.0
        }
    }
}

/// `shoreFromMask(W)` — reference line 28557.
///
/// A rough ordered shoreline for a coastal town that has real water but no
/// river centreline: every **land** cell orthogonally adjacent to a water cell,
/// ordered along the principal axis of that point cloud.
///
/// The axis comes from the 2 × 2 scatter matrix's dominant eigenvector, with
/// two guards that both matter and are both goldens: when `(sxy, l1-sxx)` is
/// degenerate the fallback `(l1-syy, sxy)` is used instead, and when *that* is
/// degenerate too the `|| 1` on the length leaves the axis at `(0, 0)` — every
/// projection is then zero, every comparison a tie, and the stable sort returns
/// the raster's own row-major order. A four-point plus-shape reaches exactly
/// that, and is this milestone's quantised tie fixture.
///
/// Returns `None` (JS `null`) below two points; `buildSite` substitutes a
/// straight bottom-edge polyline.
pub fn shore_from_mask(w: &WaterCtx) -> Option<Vec<Vec2>> {
    let (mw, mh, cell_m) = (w.mw, w.mh, w.cell_m);
    // JS truthiness on a possibly-short array: a missing cell is `undefined`,
    // which is falsy, i.e. land.
    let wet = |j: usize, i: usize| -> bool { w.mask.get(j * mw + i).copied().unwrap_or(0) != 0 };
    let mut pts: Vec<Vec2> = Vec::new();
    for j in 0..mh {
        for i in 0..mw {
            if wet(j, i) {
                continue;
            }
            let adj = (i > 0 && wet(j, i - 1))
                || (i + 1 < mw && wet(j, i + 1))
                || (j > 0 && wet(j - 1, i))
                || (j + 1 < mh && wet(j + 1, i));
            if adj {
                pts.push(Vec2::new((i as f64 + 0.5) * cell_m, (j as f64 + 0.5) * cell_m));
            }
        }
    }
    if pts.len() < 2 {
        return None;
    }
    let n = pts.len() as f64;
    let (mut mx, mut my) = (0.0, 0.0);
    for q in &pts {
        mx += q.x;
        my += q.y;
    }
    mx /= n;
    my /= n;
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for q in &pts {
        let (ax, ay) = (q.x - mx, q.y - my);
        sxx += ax * ax;
        sxy += ax * ay;
        syy += ay * ay;
    }
    let tr = sxx + syy;
    let det = sxx * syy - sxy * sxy;
    let l1 = tr / 2.0 + js_max(0.0, tr * tr / 4.0 - det).sqrt();
    let (mut vx, mut vy) = (sxy, l1 - sxx);
    if js_hypot(vx, vy) < 1e-9 {
        vx = l1 - syy;
        vy = sxy;
    }
    let vl = js_or(js_hypot(vx, vy), 1.0);
    vx /= vl;
    vy /= vl;
    // `js_num_cmp` compares the *difference* of the two projections, which is
    // the expression the reference writes and is what maps a NaN to "equal"
    // under a stable sort.
    pts.sort_by(|a, b| {
        js_num_cmp((a.x - mx) * vx + (a.y - my) * vy, (b.x - mx) * vx + (b.y - my) * vy)
    });
    Some(pts)
}

/// `buildSite(seed, Wm, Hm, kind, opts)` — reference line 28571.
///
/// **The site substream's draw order is the whole contract.** `stream(seed,
/// 'site')` is consumed in one sequence: twelve draws for the three hills
/// (always, even when a real heightfield makes them dead), then the channel or
/// coastline draws (none on the real-water and landlocked paths, eighteen for a
/// synthetic river, thirty-two for a coastline — except a **bay**, which reuses
/// its own indent centre as the harbour and so draws one *fewer*), then three or
/// four for the external route endpoints. One extra or missing draw moves every
/// later one, and `placeAnchors` and `grow` draw from their own labelled
/// substreams, so a desynchronisation here would surface as a different town
/// rather than as an error.
pub fn build_site(seed: u32, wm: f64, hm: f64, kind: &str, opts: SiteOpts) -> Site {
    // `kind = kind || 'river'` — the empty string is the only falsy string.
    let kind = if kind.is_empty() { "river" } else { kind };
    let SiteOpts { water, terrain, economy } = opts;
    let through = kind == "riverthrough";
    let no_water = kind == "landlocked" && water.is_none();
    // `!!W.riverPath` is truthy for *any* path object, including an empty or
    // one-point one — so a site can be river-like and still trace its water
    // from the mask. Reproduced; `pathOfOne` is the golden.
    let rk = match &water {
        Some(w) => w.river_path.is_some(),
        None => kind == "river" || through,
    };
    let mut r = stream(seed, "site");
    // analytic height proxy: inland rise + gentle undulation; water carves its terrace
    let mut hills = [Hill { x: 0.0, y: 0.0, amp: 0.0, rad: 0.0 }; 3];
    for h in hills.iter_mut() {
        *h = Hill {
            x: r.range(0.15, 0.9) * wm,
            y: r.range(0.05, 0.45) * hm,
            amp: r.range(0.06, 0.16),
            rad: r.range(300.0, 650.0),
        };
    }

    let river: Vec<Vec2>;
    let river_w: f64;
    let mut water_poly: Vec<Vec2> = Vec::new();
    let mut harbour_idx: i64 = 0;

    if let Some(w) = &water {
        // the town's water IS the map's water: a real river centreline, else a
        // rough shoreline for a purely coastal town.
        match &w.river_path {
            Some(rp) if rp.len() >= 2 => {
                river = rp.clone();
                river_w = js_or_opt(w.river_width_m, 20.0);
            }
            _ => {
                river = shore_from_mask(w)
                    .unwrap_or_else(|| vec![Vec2::new(0.0, hm), Vec2::new(wm, hm)]);
                river_w = 12.0;
            }
        }
    } else if no_water {
        // a far dummy so every water query comes back "dry"
        river = vec![Vec2::new(-1e4, -1e4), Vec2::new(-1e4 + 1.0, -1e4 + 1.0)];
        river_w = 0.0;
        water_poly = Vec::new();
        harbour_idx = 0;
    } else if rk {
        // river: west -> east; through the middle (bisecting) or around the lower third
        let n = 14usize;
        let base_y = (if through { r.range(0.44, 0.54) } else { r.range(0.58, 0.72) }) * hm;
        let mut y = base_y + r.range(-40.0, 40.0);
        let lo = (if through { 0.34 } else { 0.45 }) * hm;
        let hi = (if through { 0.66 } else { 0.88 }) * hm;
        let mut pts = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let x = (i as f64 / n as f64) * wm;
            y += r.range(-1.0, 1.0) * (if through { 44.0 } else { 55.0 });
            y = js_max(lo, js_min(hi, y));
            pts.push(Vec2::new(x, y));
        }
        river = chaikin(&chaikin(&pts, false), false);
        river_w = if through { r.range(22.0, 30.0) } else { r.range(17.0, 24.0) };
    } else {
        // coastline: sea fills the map south of a west->east shoreline; a bay indents it
        let base_y = r.range(0.64, 0.76) * hm;
        let bay_amp = if kind == "bay" { r.range(150.0, 230.0) } else { r.range(6.0, 22.0) };
        let bx = r.range(0.35, 0.62) * wm;
        let bw = r.range(230.0, 330.0);
        let n = 26usize;
        let mut pts = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let x = (i as f64 / n as f64) * wm;
            let y = base_y - bay_amp * js_exp(-((x - bx) * (x - bx)) / (2.0 * bw * bw))
                + r.range(-24.0, 24.0);
            pts.push(Vec2::new(x, y));
        }
        river = chaikin(&chaikin(&pts, false), false); // the waterline polyline (shoreline)
        river_w = 12.0; // nominal band for wall/flood offsets; the sea itself is a half-plane
        // harbour point: bay head (most sheltered) for bays, seed-chosen stretch
        // for coasts. The bay branch draws nothing here — that is the one-draw
        // asymmetry the module docs call out.
        let hx = if kind == "bay" { bx } else { r.range(0.35, 0.6) * wm };
        let (mut best, mut hd) = (0usize, f64::INFINITY);
        for (i, q) in river.iter().enumerate() {
            let d = (q.x - hx).abs();
            if d < hd {
                hd = d;
                best = i;
            }
        }
        harbour_idx = best as i64;
    }

    let uses_real_water = water.is_some();
    let real_river = water.as_ref().is_some_and(|w| w.river_path.is_some());
    let uses_real_terrain = terrain.is_some();
    let terrain_relief =
        terrain.as_ref().map_or(0.0, |t| js_or(t.h_max, 0.0) - js_or(t.h_min, 0.0));
    let water_order = water.as_ref().map_or(0.0, |w| js_or(w.river_order, 0.0));
    let sea_lake_cells = water.as_ref().map_or(0.0, |w| js_or(w.sea_lake_cells, 0.0));

    let mut site = Site {
        kind: kind.to_string(),
        through,
        no_water,
        wm,
        hm,
        river,
        river_w,
        water_poly,
        bridge_pt: None,
        bridge_dir: None,
        route_ends: Vec::new(),
        uses_real_water,
        real_river,
        uses_real_terrain,
        terrain_relief,
        water_order,
        sea_lake_cells,
        economy,
        harbour: Harbour { idx: 0, pt: None },
        hills,
        rk,
        water,
        terrain,
    };

    if rk {
        let len = site.river.len();
        // bridge point: flattest approaches, in the middle half of the river.
        // `bi` starts at -1 and is only assigned by a strictly-better `s`, so a
        // NaN slope field (a hole in a real heightfield) leaves it at -1 and
        // `Math.max(0, bi)` puts the bridge on the river's first point.
        let mut bi: i64 = -1;
        let mut bs = f64::INFINITY;
        let start = (len as f64 * 0.3).floor() as usize;
        let end = (len as f64 * 0.72).floor() as usize;
        for i in start..end {
            let p = site.river[i];
            let s = site.slope(Vec2::new(p.x, p.y - 40.0)) + site.slope(Vec2::new(p.x, p.y + 40.0));
            if s < bs {
                bs = s;
                bi = i as i64;
            }
        }
        let bi = bi.max(0) as usize;
        site.bridge_pt = Some(site.river[bi]);
        let a = site.river[bi.saturating_sub(1)];
        let b = site.river[(bi + 1).min(len - 1)];
        site.bridge_dir = Some((b - a).norm());
        // river port quay sits a little downstream of the bridge (break-of-bulk)
        let mut acc = 0.0;
        let mut hi = bi;
        while hi + 2 < len && acc < 95.0 {
            acc += site.river[hi].dist(site.river[hi + 1]);
            hi += 1;
        }
        harbour_idx = hi as i64;
        // water polygon band
        let (mut left, mut right) = (Vec::with_capacity(len), Vec::with_capacity(len));
        for i in 0..len {
            let a = site.river[i.saturating_sub(1)];
            let b = site.river[(i + 1).min(len - 1)];
            let d = (b - a).norm();
            let nl = d.rot90();
            left.push(site.river[i] + nl * (site.river_w / 2.0));
            right.push(site.river[i] + nl * (-site.river_w / 2.0));
        }
        right.reverse();
        left.extend(right);
        site.water_poly = left;
    } else if site.uses_real_water {
        // coastal town with real water: the real sea/lake is already on the map, so no
        // synthetic fill; the harbour goes on the real shoreline nearest the box centre.
        site.water_poly = Vec::new();
        let (mut best, mut hd) = (0usize, f64::INFINITY);
        for (i, q) in site.river.iter().enumerate() {
            let d = js_hypot(q.x - wm / 2.0, q.y - hm / 2.0);
            if d < hd {
                hd = d;
                best = i;
            }
        }
        harbour_idx = best as i64;
    } else if !no_water {
        let mut wp = site.river.clone();
        wp.push(Vec2::new(wm, hm));
        wp.push(Vec2::new(0.0, hm));
        site.water_poly = wp;
    }

    // external route endpoints; sea sites trade seaward, so they get one land
    // route fewer, and landlocked sites are all-land.
    site.route_ends = if rk || no_water {
        vec![
            Vec2::new(r.range(0.08, 0.3) * wm, 0.0),
            Vec2::new(wm, r.range(0.1, 0.4) * hm),
            Vec2::new(0.0, r.range(0.15, 0.45) * hm),
            Vec2::new(r.range(0.4, 0.75) * wm, hm),
        ]
    } else {
        vec![
            Vec2::new(r.range(0.15, 0.4) * wm, 0.0),
            Vec2::new(wm, r.range(0.08, 0.3) * hm),
            Vec2::new(0.0, r.range(0.08, 0.32) * hm),
        ]
    };

    site.harbour = if no_water {
        Harbour { idx: -1, pt: None }
    } else {
        let pt = usize::try_from(harbour_idx)
            .ok()
            .and_then(|i| site.river.get(i))
            .or_else(|| site.river.first())
            .copied();
        Harbour { idx: harbour_idx, pt }
    };
    site
}

/// `terrainSuitability(site, p)` — reference line 28723.
///
/// A `[0, 1]` buildability score, **multiplicative** by design: a flood-prone
/// flat is still bad because it floods and a dry steep slope is still bad
/// because it is steep, so either factor alone can drag the score to zero and
/// neither can cancel the other out. Attached to every parcel by
/// `assignDistricts` (milestone 13) and, only when `opts.terrainAware` is on,
/// used to exclude the worst parcels from building at all.
///
/// `hashModel` deliberately does not hash it, which is what keeps it
/// cross-version neutral.
pub fn terrain_suitability(site: &Site, p: Vec2) -> f64 {
    // clip the rare coastline isWater() step-discontinuity
    let s = js_min(1.0, site.slope(p));
    let slope_score = js_exp(-(s * s) / (2.0 * 0.3 * 0.3));
    // the SAME flood-band margin placeAnchors and buildWall use, not a new constant
    let margin = js_or(site.river_w, 0.0) / 2.0 + 30.0;
    let rd = if site.is_water(p) { 0.0 } else { site.river_dist(p) };
    let flood_score = js_max(0.0, js_min(1.0, rd / margin));
    slope_score * flood_score
}
