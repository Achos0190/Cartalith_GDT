//! Milestone 14 — amenities (`buildMarkets`, `buildCivic`, `orientedRect`,
//! `gamesShapeAt`, `buildGames`, reference lines 29160-29382).
//!
//! What a settlement *carries* rather than what it is shaped like. Three
//! stages, all of them run near the end of `generate()` (lines 31056-31058) on
//! a town that already has a graph, a plaza, blocks and parcels:
//!
//! - [`build_markets`] — the specialised markets that split off the main square
//!   as population crosses five documented thresholds (M-AMEN-1), each
//!   colonising a node of the street graph and clearing what stood there.
//! - [`build_civic`] — one civic hall on the plaza (M-ADMIN), in five styles,
//!   scaled by settlement rank.
//! - [`build_games`] — the population-gated spectacle building (M-GAMES),
//!   sited beside the plaza if anything fits there and out past the town's
//!   edge if not, and **honestly omitted** when nowhere fits at all.
//!
//! ## The line range, verified
//!
//! `URBAN_MORPHOLOGY_SCOPE.md` claims 29160-29382 and, unusually for this
//! subsystem, is right at **both** ends: 29160 is `function buildMarkets(` and
//! 29382 is `buildGames`' closing `return out;}`. (Milestone 7 had already
//! corrected the stated end down from 29389, which ran seven lines into
//! `logisticRamp`'s doc comment; that correction is the one in force.) The
//! section header at 29156-29159 sits above the range, as it does for every
//! other milestone whose start the scope document quotes as the `function`
//! keyword. `GAMES_SPEC` (lines 29258-29267) is inside the range even though
//! the scope document's five-function list does not name it.
//!
//! ## Three arguments this port narrows, and why each is exact
//!
//! The reference passes whole objects and mutates two of them. Nothing here
//! reads a field it is not given, so each narrowing is a projection rather
//! than an approximation — but they are deviations in *signature* and are
//! listed rather than buried:
//!
//! 1. **`buildMarkets`' `parcels` and `buildings`.** The reference reads
//!    exactly `polyCentroid(x.poly)` from each and writes `par.cleared = true`
//!    / `buildings.splice(i, 1)`. [`crate::blocks::Parcel`] has no `cleared`
//!    field (it is milestone 12's, and this milestone does not own it), and
//!    the building type is milestone 13's and does not exist yet. So
//!    [`build_markets`] takes the two **centroid lists** and returns the two
//!    **index lists** in [`Markets`], leaving the caller to apply them. The
//!    computation is untouched: a centroid does not change when a market is
//!    placed, and removing a building the reference had already spliced out is
//!    a no-op, so the set that comes back is the reference's set exactly.
//! 2. **`buildGames`' `parcels`.** Read for `p.poly` only, so a slice of
//!    polygons is passed rather than a parcel type this module would otherwise
//!    have to depend on.
//! 3. **`buildGames`' `wallState`.** Read as `wallState.ring` and nowhere else.
//!    A caller holding a [`crate::growth::WallState`] passes
//!    `wall_state.ring.as_deref()`.
//!
//! `buildCivic`'s `anchors` argument is **dead in the reference** — the body
//! never mentions it — so it is not on [`build_civic`].
//!
//! ## `Math.log10`
//!
//! [`build_civic`]'s rank scaling is the first place in this crate to need
//! `Math.log10`, which `cartalith-jsmath` does not carry. [`js_log10`] is the
//! fdlibm `__ieee754_log10` V8 runs, expressed over
//! [`js_log`](crate::geom::js_log) — the same treatment `js_log` itself got,
//! and for the same reason: the platform's `f64::log10` is a different
//! implementation and this one is pinned against V8's own output in the tests.

use crate::geom::{Vec2, js_cos, js_hypot, js_log, js_max, js_sin, point_in_poly, seg_int};
use crate::graph::Graph;
use crate::plaza::Plaza;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::rules::CultureProfile;
use crate::site::Site;

/// `Math.log10`, as fdlibm computes it and therefore as V8 does.
///
/// The platform's `f64::log10` is a *different* implementation, and this
/// project has twice been bitten by assuming two libms agree
/// (`js_hypot`, `js_exp`). Written in terms of [`js_log`], which
/// `cartalith-jsmath` already carries as the fdlibm `__ieee754_log`, so the
/// only new arithmetic here is fdlibm's own three-constant scaling.
///
/// It belongs in `cartalith-jsmath` beside `js_log`; it is here because this
/// milestone does not own that crate. Moving it is a one-line change at each
/// end and costs nothing.
#[allow(clippy::excessive_precision, clippy::eq_op, clippy::approx_constant)]
pub fn js_log10(x: f64) -> f64 {
    const TWO54: f64 = 1.80143985094819840000e+16;
    const IVLN10: f64 = 4.34294481903251816668e-01;
    const LOG10_2HI: f64 = 3.01029995663611771306e-01;
    const LOG10_2LO: f64 = 3.69423907715893089906e-13;

    let mut x = x;
    let mut hx = (x.to_bits() >> 32) as i32;
    let lx = x.to_bits() as u32;
    let mut k = 0i32;

    if hx < 0x0010_0000 {
        if ((hx & 0x7fff_ffff) as u32 | lx) == 0 {
            return -TWO54 / 0.0; // log10(+-0) = -inf
        }
        if hx < 0 {
            return (x - x) / 0.0; // log10(negative) = NaN
        }
        k -= 54;
        x *= TWO54; // subnormal: scale up
        hx = (x.to_bits() >> 32) as i32;
    }
    if hx >= 0x7ff0_0000 {
        return x + x; // Inf or NaN
    }
    k += (hx >> 20) - 1023;
    let i = ((k as u32) & 0x8000_0000) >> 31;
    hx = (hx & 0x000f_ffff) | ((0x3ff - i as i32) << 20);
    let y = (k + i as i32) as f64;
    // SET_HIGH_WORD(x, hx) -- on the possibly-rescaled x, not the original
    x = f64::from_bits(((hx as u32 as u64) << 32) | (x.to_bits() & 0xffff_ffff));
    let z = y * LOG10_2LO + IVLN10 * js_log(x);
    z + y * LOG10_2HI
}

/* ---------------------------------------------------------------- markets */

/// One specialised market square (line 29184).
#[derive(Debug, Clone, PartialEq)]
pub struct Market {
    /// One of the five fixed trade names, in threshold order.
    pub name: &'static str,
    /// The graph node the square was colonised onto.
    pub center: Vec2,
    /// The axis-aligned quad, the reference's own winding
    /// (`-sw,-sh`, `+sw,-sh`, `+sw,+sh`, `-sw,+sh`).
    pub poly: Vec<Vec2>,
    pub prov: String,
}

/// [`build_markets`]' whole result: the squares, plus the two mutations the
/// reference performs in place (see this module's header).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Markets {
    pub markets: Vec<Market>,
    /// Indices into `parcel_centroids` the reference sets `cleared` on,
    /// ascending.
    pub cleared_parcels: Vec<usize>,
    /// Indices into `building_centroids` the reference splices out, ascending.
    pub removed_buildings: Vec<usize>,
}

/// The five thresholds, in the reference's own push order (line 29162-29166).
///
/// Meat splits off first; the cattle market is last because it is
/// space-hungry and wants a gate.
const MARKET_THRESHOLDS: [(f64, &str); 5] = [
    (1500.0, "Shambles"),
    (3500.0, "Fish Market"),
    (3500.0, "Corn Market"),
    (8000.0, "Cloth Market"),
    (14000.0, "Cattle Market"),
];

/// `buildMarkets` (line 29160) — the specialised markets that multiply as the
/// town crosses its population thresholds.
///
/// Each name in turn takes the junction (degree ≥ 3 over **live** edges) whose
/// distance from the market anchor is closest to 170 m — jittered by a draw
/// from the `'markets'` substream taken **per surviving candidate**, so the
/// substream's position depends on how many nodes pass the filters, not only
/// on how many squares are placed. Candidates must be 85-300 m out, dry, and
/// 95 m clear of every square already used (the plaza itself counts as the
/// first).
///
/// See the module header for why `parcel_centroids`/`building_centroids` are
/// centroids and why the clearing comes back as index lists.
#[allow(clippy::too_many_arguments)]
pub fn build_markets(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    plaza: Option<&Plaza>,
    pop: f64,
    parcel_centroids: &[Vec2],
    building_centroids: &[Vec2],
) -> Markets {
    let mut r = stream(seed, "markets");
    let names: Vec<&'static str> =
        MARKET_THRESHOLDS.iter().filter(|(t, _)| pop >= *t).map(|(_, n)| *n).collect();

    let mut out = Markets::default();
    let mut cleared = vec![false; parcel_centroids.len()];
    let mut removed = vec![false; building_centroids.len()];
    let mut used: Vec<Vec2> = vec![plaza.map_or(anchors.market, |p| p.center)];

    for nm in names {
        let mut best: Option<Vec2> = None;
        let mut bs = f64::INFINITY;
        for nd in &g.nodes {
            if nd.adj.iter().filter(|&&id| g.edges[id].alive).count() < 3 {
                continue;
            }
            let p = nd.pt();
            let d_m = p.dist(anchors.market);
            // Written as the reference writes it: `!(85.0..=300.0).contains()`
            // is not the same function when `d_m` is NaN, and JS's chain of
            // `<`/`>` comparisons is all-false there.
            #[allow(clippy::manual_range_contains, reason = "NaN semantics differ; see above")]
            if d_m < 85.0 || d_m > 300.0 || site.is_water(p) {
                continue;
            }
            if used.iter().any(|u| u.dist(p) < 95.0) {
                continue;
            }
            // The jitter draw happens for every candidate that got this far,
            // whether or not it wins.
            let s = (d_m - 170.0).abs() + r.range(0.0, 50.0);
            if s < bs {
                bs = s;
                best = Some(p);
            }
        }
        let Some(best) = best else { continue };
        used.push(best);
        let sw = r.range(15.0, 22.0);
        let sh = r.range(11.0, 16.0);
        let poly = vec![
            Vec2::new(best.x - sw, best.y - sh),
            Vec2::new(best.x + sw, best.y - sh),
            Vec2::new(best.x + sw, best.y + sh),
            Vec2::new(best.x - sw, best.y + sh),
        ];

        // Market colonisation: whatever stood on the square goes. The
        // reference walks `buildings` backwards because it splices; a mask
        // needs no such care, and re-testing an already-removed building is
        // the no-op the reference's shorter array made unnecessary.
        for (i, c) in building_centroids.iter().enumerate() {
            if point_in_poly(*c, &poly) {
                removed[i] = true;
            }
        }
        for (i, c) in parcel_centroids.iter().enumerate() {
            if point_in_poly(*c, &poly) {
                cleared[i] = true;
            }
        }

        out.markets.push(Market {
            name: nm,
            center: best,
            poly,
            prov: format!(
                "Specialised market ({nm}): as the town grew past its threshold, this trade \
                 split off onto its own square (M-AMEN-1; multiple-market towns like King's Lynn)."
            ),
        });
    }
    out.cleared_parcels = cleared.iter().enumerate().filter(|&(_, &c)| c).map(|(i, _)| i).collect();
    out.removed_buildings = removed.iter().enumerate().filter(|&(_, &c)| c).map(|(i, _)| i).collect();
    out
}

/* ------------------------------------------------------------------ civic */

/// `buildCivic`'s return value (line 29202) — the civic hall on the market.
///
/// `apse` carries two unrelated things, exactly as the reference does: the
/// basilica's semicircular apse, and the keep's inset donjon roofline, which
/// re-uses the apse render path rather than adding tiered-roof geometry
/// (M-JPN-3). `columns` likewise doubles as the keep's corner turrets.
#[derive(Debug, Clone, PartialEq)]
pub struct Civic {
    /// The **resolved** style — never `'auto'`, and never empty.
    pub style: String,
    pub center: Vec2,
    pub columns: Vec<Vec2>,
    pub belfry: Option<Vec2>,
    pub apse: Option<Vec<Vec2>>,
    pub hall: Vec<Vec2>,
    pub name: &'static str,
    /// The reference sets this only on the dome branch; absent is falsy.
    pub dome: bool,
    pub prov: &'static str,
}

/// `buildCivic` (line 29189) — one civic hall on the plaza, rank-scaled.
///
/// [`None`] on either of the reference's two refusals: no plaza or fewer than
/// 1500 people (a civic hall appears once a place is a chartered town), and a
/// resolved style of `'none'` (Islamic governance was not a monumental civic
/// building). Both are real states, not errors.
///
/// `style` is resolved when it is `'auto'` or falsy — which in JS includes the
/// empty string, so `""` resolves rather than falling through to the default
/// branch. The reference's `anchors` parameter is dead and is not taken here;
/// see the module header.
pub fn build_civic(
    seed: u32,
    plaza: Option<&Plaza>,
    pop: f64,
    style: &str,
    faith: &str,
) -> Option<Civic> {
    let plaza = plaza?;
    if pop < 1500.0 {
        return None;
    }
    let style: &str = if style == "auto" || style.is_empty() {
        match faith {
            "temple" | "shrine" | "orthodox" => "basilica",
            "mosque" => "none",
            _ => "townhall",
        }
    } else {
        style
    };
    if style == "none" {
        return None;
    }

    let c = plaza.center;
    let p0 = plaza.poly[0];
    let p1 = plaza.poly[1];
    let mid = p0.lerp(p1, 0.5);
    let mut inl = (c - mid).norm();
    if !inl.x.is_finite() {
        inl = Vec2::new(0.0, 1.0);
    }
    let perp = inl.rot90();
    let base = mid + inl * 8.0;
    let mut r = stream(seed, "civic");

    let rect = |cc: Vec2, ww: f64, dd: f64| -> Vec<Vec2> {
        vec![
            (cc + perp * (-ww / 2.0)) + inl * (-dd / 2.0),
            (cc + perp * (ww / 2.0)) + inl * (-dd / 2.0),
            (cc + perp * (ww / 2.0)) + inl * (dd / 2.0),
            (cc + perp * (-ww / 2.0)) + inl * (dd / 2.0),
        ]
    };

    let mut columns: Vec<Vec2> = Vec::new();
    let mut belfry: Option<Vec2> = None;
    let mut apse: Option<Vec<Vec2>> = None;
    let mut dome = false;

    // Civic-hall scale by settlement rank (M-AMEN-3): 1.0x at the pop-1500
    // gate, ~1.9x at the 20 000 cap, log-scaled so the earliest growth reads
    // as clearly as the latest. Declared a PoC convention by the reference
    // itself (L confidence), not a measured curve. `js_max` because JS
    // propagates a NaN population where Rust's `f64::max` would absorb it.
    let size_mult = 1.0 + 0.9 * js_log10(js_max(pop, 1500.0) / 1500.0) / js_log10(20000.0 / 1500.0);

    let (hall, name, prov);
    match style {
        "basilica" => {
            let w = r.range(30.0, 42.0) * size_mult;
            let d = r.range(14.0, 18.0) * size_mult;
            hall = rect(base, w, d);
            name = "Basilica";
            // Semicircular apse at one short end: nine points, endpoints
            // included.
            let ex = base + perp * (w / 2.0);
            let mut ap = Vec::with_capacity(9);
            for k in 0..=8 {
                let a = (-std::f64::consts::FRAC_PI_2) + std::f64::consts::PI * f64::from(k) / 8.0;
                ap.push((ex + perp * (js_cos(a) * d * 0.5)) + inl * (js_sin(a) * d * 0.5));
            }
            apse = Some(ap);
            for k in -3..=3 {
                columns
                    .push((base + perp * (f64::from(k) * w * 0.14)) + inl * (-d / 2.0 - 1.4));
            }
            prov = "Basilica: the Roman civic hall (law-court + assembly) — a long colonnaded hall with an apse — standing in for the town hall in a classical rite (M-ADMIN; docs/05 §2.1).";
        }
        "loggia" => {
            let w = r.range(22.0, 30.0) * size_mult;
            let d = r.range(11.0, 15.0) * size_mult;
            hall = rect(base, w, d);
            name = "Guild loggia";
            for k in -3..=3 {
                columns.push((base + perp * (f64::from(k) * w * 0.15)) + inl * (-d / 2.0));
            }
            prov = "Guild loggia: an open ground-floor arcade on the market where the guilds and merchants met and traded (M-ADMIN; docs/05 §2.2).";
        }
        "keep" => {
            let w = r.range(26.0, 36.0) * size_mult;
            let d = r.range(20.0, 26.0) * size_mult;
            hall = rect(base, w, d);
            name = "Castle keep";
            // Tiered donjon (tenshu): a smaller inset roofline on the same
            // footprint, drawn through the apse path; the corner turrets are
            // the hall's own vertices through the column path.
            apse = Some(rect(base, w * 0.52, d * 0.52));
            columns = hall.clone();
            prov = "Castle keep (tenshu): a tiered stone-and-timber donjon at the jokamachi's core, the daimyo's seat of military and administrative authority (M-JPN-3).";
        }
        "dome" => {
            // The one style that is not a rectangle re-skin: a 16-gon drum.
            let rad = r.range(13.0, 18.0) * size_mult;
            let n_sides = 16;
            let mut h = Vec::with_capacity(n_sides);
            for k in 0..n_sides {
                let a = 2.0 * std::f64::consts::PI * (k as f64) / (n_sides as f64);
                h.push(Vec2::new(base.x + js_cos(a) * rad, base.y + js_sin(a) * rad));
            }
            hall = h;
            name = "Center for Resource Management";
            dome = true;
            for k in 0..12 {
                let a = 2.0 * std::f64::consts::PI * f64::from(k) / 12.0;
                columns.push(Vec2::new(
                    base.x + js_cos(a) * rad * 0.82,
                    base.y + js_sin(a) * rad * 0.82,
                ));
            }
            prov = "Center for Resource Management: a domed circular hub housing the cybernated system coordinating resource management, education, health and communications at the centre of the circular city (M-VEN-2).";
        }
        _ => {
            let w = r.range(22.0, 30.0) * size_mult;
            let d = r.range(13.0, 17.0) * size_mult;
            hall = rect(base, w, d);
            name = if pop >= 10000.0 { "Town hall" } else { "Guildhall" };
            belfry = Some(Vec2::new(
                base.x + perp.x * (w * 0.32),
                base.y + perp.y * (w * 0.32),
            ));
            prov = "Town hall/guildhall on the market square, with a belfry — the communal movement (late 12th–13th c.) made civic self-government a building; a rank marker (M-ADMIN; docs/05 §2.3).";
        }
    }

    Some(Civic { style: style.to_string(), center: base, columns, belfry, apse, hall, name, dome, prov })
}

/* ------------------------------------------------------------------ games */

/// One entry of `GAMES_SPEC` (line 29258).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamesSpec {
    pub kind: &'static str,
    pub name: &'static str,
    /// **Dead in the reference.** Every surviving spec is `'rect'` and
    /// [`games_shape_at`] never reads it; the ellipse/stadium/ballcourt cases
    /// it dispatched went with the post-launch simplification pass (docs/07
    /// §3.10). Kept because the table has the field.
    pub shape: &'static str,
    /// `'plaza'` selects the intramural search before the peripheral one.
    pub siting: &'static str,
    pub w: (f64, f64),
    pub d: (f64, f64),
    pub min_pop: f64,
    pub prov: &'static str,
}

/// `GAMES_SPEC.medieval` — one entry, the tiltyard.
pub static GAMES_SPEC_MEDIEVAL: &[GamesSpec] = &[GamesSpec {
    kind: "tiltyard",
    name: "Tiltyard",
    shape: "rect",
    siting: "plaza",
    w: (30.0, 42.0),
    d: (10.0, 14.0),
    min_pop: 3000.0,
    prov: "Tiltyard: for an ordinary town (not a royal palace), tournaments were staged directly in the town's own marketplace (Brussels' Grote Markt; also Lille, Cambrai) — dedicated palace tiltyards (Hampton Court, Whitehall, Kenilworth) were the exception, not the norm (Damen 2016, M-GAMES-1).",
}];

/// `GAMES_SPEC.venus` — deliberately empty, which is what makes
/// [`build_games`] return nothing on that profile.
pub static GAMES_SPEC_VENUS: &[GamesSpec] = &[];

/// `GAMES_SPEC[profile.id]`. A profile with no entry at all is `undefined` in
/// the reference and takes the same `!specs` exit as an empty list, so both
/// map onto an empty slice here.
pub fn games_spec(profile_id: &str) -> &'static [GamesSpec] {
    match profile_id {
        "medieval" => GAMES_SPEC_MEDIEVAL,
        _ => GAMES_SPEC_VENUS,
    }
}

/// One placed spectacle building (line 29380).
///
/// The reference also writes `inner: null` with a comment saying the ushnu
/// platform inset it once carried has had no live caller since docs/07 §3.10.
/// A field that is unconditionally null is not modelled.
#[derive(Debug, Clone, PartialEq)]
pub struct GamesBuilding {
    pub id: String,
    pub kind: &'static str,
    pub name: &'static str,
    pub poly: Vec<Vec2>,
    pub center: Vec2,
    pub prov: &'static str,
}

/// `orientedRect` (line 29268) — a rectangle from a centre and an axis.
///
/// `along` is the unit vector the `w` axis runs on; its perpendicular carries
/// `d`. Four points, wound as `(-w,-d)`, `(+w,-d)`, `(+w,+d)`, `(-w,+d)` in
/// that frame.
pub fn oriented_rect(center: Vec2, along: Vec2, w: f64, d: f64) -> [Vec2; 4] {
    let perp = along.rot90();
    [
        (center + along * (-w / 2.0)) + perp * (-d / 2.0),
        (center + along * (w / 2.0)) + perp * (-d / 2.0),
        (center + along * (w / 2.0)) + perp * (d / 2.0),
        (center + along * (-w / 2.0)) + perp * (d / 2.0),
    ]
}

/// `gamesShapeAt` (line 29274) — the footprint for a spec.
///
/// Every surviving spec uses a plain oriented rectangle, so this is
/// [`oriented_rect`] and the `spec` argument goes unread. Kept as its own
/// function because the reference has one, and because it is the seam the
/// removed ellipse/stadium/ballcourt shapes came through.
pub fn games_shape_at(_spec: &GamesSpec, center: Vec2, along: Vec2, w: f64, d: f64) -> [Vec2; 4] {
    oriented_rect(center, along, w, d)
}

/// The map-edge keep-out every candidate centre and vertex is tested against.
const MARGIN: f64 = 25.0;

/// A parcel reduced to what the collision test reads: its polygon, the plain
/// mean of its vertices (**not** `polyCentroid`), and its bounding radius.
struct ParcelInfo<'a> {
    poly: &'a [Vec2],
    cx: f64,
    cy: f64,
    r: f64,
}

/// `buildGames` (line 29277) — the population-gated spectacle building.
///
/// Two searches, in order. **Plaza-adjacent** first (14 bearings from a random
/// start, at four expanding tiers off the plaza's own radius) because most of
/// the M-GAMES register was genuinely intramural, immediately beside the main
/// square; it is the only one that has to check parcels, since that zone is
/// not empty by construction. **Peripheral** second (16 bearings, four tiers
/// past the town's realised extent) when the first found nowhere safe.
///
/// Every candidate is generated, then range- and collision-checked against the
/// map edge, water, the live street graph and the civic hall; a doomed
/// candidate is retried at another bearing or radius, never forced in. **An
/// empty result is a valid outcome** — the same discipline [`build_civic`]
/// uses when its own gate is not met.
///
/// The peripheral search's own `a0` is drawn **only if the plaza search
/// failed**, so the `'games'` substream's position depends on which branch
/// ran. `parcel_polys` and `wall_ring` are narrowed from the reference's
/// arguments; see the module header.
#[allow(clippy::too_many_arguments)]
pub fn build_games(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &Graph,
    parcel_polys: &[&[Vec2]],
    wall_ring: Option<&[Vec2]>,
    pop_target: f64,
    profile: &CultureProfile,
    plaza: Option<&Plaza>,
    civic: Option<&Civic>,
) -> Vec<GamesBuilding> {
    let specs = games_spec(profile.id);
    if specs.is_empty() {
        return Vec::new();
    }
    let mut r = stream(seed, "games");

    // A bounding circle per parcel, computed once rather than per candidate.
    let mut parcel_info: Vec<ParcelInfo> = Vec::with_capacity(parcel_polys.len());
    let mut built_r = 0.0f64;
    for poly in parcel_polys {
        let (mut cx, mut cy) = (0.0f64, 0.0f64);
        for v in *poly {
            cx += v.x;
            cy += v.y;
        }
        cx /= poly.len() as f64;
        cy /= poly.len() as f64;
        let mut pr = 0.0f64;
        for v in *poly {
            pr = js_max(pr, js_hypot(v.x - cx, v.y - cy));
        }
        parcel_info.push(ParcelInfo { poly, cx, cy, r: pr });
        for v in *poly {
            built_r = js_max(built_r, v.dist(anchors.market));
        }
    }
    if let Some(ring) = wall_ring {
        for v in ring {
            built_r = js_max(built_r, v.dist(anchors.market));
        }
    }
    let plaza_center = plaza.map_or(anchors.market, |p| p.center);
    let mut plaza_r = 0.0f64;
    if let Some(p) = plaza {
        for v in &p.poly {
            plaza_r = js_max(plaza_r, v.dist(plaza_center));
        }
    }

    let overlaps_parcels = |poly: &[Vec2], cx: f64, cy: f64, rad: f64| -> bool {
        for pi in &parcel_info {
            if js_hypot(cx - pi.cx, cy - pi.cy) > rad + pi.r + 2.0 {
                continue;
            }
            for k in 0..poly.len() {
                let a = poly[k];
                let b = poly[(k + 1) % poly.len()];
                for j in 0..pi.poly.len() {
                    if seg_int(a, b, pi.poly[j], pi.poly[(j + 1) % pi.poly.len()]).is_some() {
                        return true;
                    }
                }
            }
            if point_in_poly(poly[0], pi.poly) || point_in_poly(pi.poly[0], poly) {
                return true;
            }
        }
        false
    };

    // Map bounds, water, the live street graph (the same `edgesNear`+`segInt`
    // pair `grow` itself uses) and the civic hall.
    let blocked = |poly: &[Vec2]| -> bool {
        if poly
            .iter()
            .any(|v| v.x < MARGIN || v.y < MARGIN || v.x > site.wm - MARGIN || v.y > site.hm - MARGIN)
        {
            return true;
        }
        if poly.iter().any(|v| site.is_water(*v)) {
            return true;
        }
        for k in 0..poly.len() {
            let a = poly[k];
            let b = poly[(k + 1) % poly.len()];
            for eid in g.edges_near(a, b) {
                let Some(e) = g.edges.get(eid) else { continue };
                if !e.alive {
                    continue;
                }
                if seg_int(a, b, g.nodes[e.a].pt(), g.nodes[e.b].pt()).is_some() {
                    return true;
                }
            }
            if let Some(cv) = civic {
                let h = &cv.hall;
                for j in 0..h.len() {
                    if seg_int(a, b, h[j], h[(j + 1) % h.len()]).is_some() {
                        return true;
                    }
                }
            }
        }
        false
    };

    let mut out: Vec<GamesBuilding> = Vec::new();
    let mut placed_at: Vec<(Vec2, f64)> = Vec::new();
    let mut gid = 0usize;

    for spec in specs {
        if pop_target < spec.min_pop {
            continue;
        }
        let w = r.range(spec.w.0, spec.w.1);
        let d = r.range(spec.d.0, spec.d.1);
        let half = js_max(w, d) / 2.0;
        let mut found: Option<(Vec2, [Vec2; 4])> = None;

        if spec.siting == "plaza" {
            let n_ang = 14;
            let a0 = r.u() * std::f64::consts::PI * 2.0;
            'tiers: for tier in [20.0, 70.0, 140.0, 230.0] {
                let cand_r = plaza_r + half + tier;
                for i in 0..n_ang {
                    let ang = a0 + f64::from(i) * (2.0 * std::f64::consts::PI / f64::from(n_ang));
                    let center = Vec2::new(
                        plaza_center.x + js_cos(ang) * cand_r,
                        plaza_center.y + js_sin(ang) * cand_r,
                    );
                    if center.x < MARGIN
                        || center.y < MARGIN
                        || center.x > site.wm - MARGIN
                        || center.y > site.hm - MARGIN
                    {
                        continue;
                    }
                    if placed_at.iter().any(|(c, oh)| center.dist(*c) < half + oh + 20.0) {
                        continue;
                    }
                    let radial = (center - plaza_center).norm();
                    let poly = games_shape_at(spec, center, radial.rot90(), w, d);
                    if blocked(&poly) {
                        continue;
                    }
                    if overlaps_parcels(&poly, center.x, center.y, half) {
                        continue;
                    }
                    found = Some((center, poly));
                    break 'tiers;
                }
            }
        }

        // Peripheral siting, or a plaza search that found nowhere safe: beyond
        // the town's realised extent, which is clear of every parcel by
        // construction, so no parcel check is needed here.
        if found.is_none() {
            let n_ang = 16;
            let a0 = r.u() * std::f64::consts::PI * 2.0;
            'tiers: for extra in [0.0, 90.0, 180.0, 280.0] {
                let cand_r = built_r + half + 40.0 + extra;
                for i in 0..n_ang {
                    let ang = a0 + f64::from(i) * (2.0 * std::f64::consts::PI / f64::from(n_ang));
                    let center = Vec2::new(
                        anchors.market.x + js_cos(ang) * cand_r,
                        anchors.market.y + js_sin(ang) * cand_r,
                    );
                    if center.x < MARGIN
                        || center.y < MARGIN
                        || center.x > site.wm - MARGIN
                        || center.y > site.hm - MARGIN
                    {
                        continue;
                    }
                    if placed_at.iter().any(|(c, oh)| center.dist(*c) < half + oh + 40.0) {
                        continue;
                    }
                    let radial = Vec2::new(js_cos(ang), js_sin(ang)).norm();
                    let poly = games_shape_at(spec, center, radial.rot90(), w, d);
                    if blocked(&poly) {
                        continue;
                    }
                    found = Some((center, poly));
                    break 'tiers;
                }
            }
        }

        // No safe site: an honest omission, never forced in.
        let Some((center, poly)) = found else { continue };
        placed_at.push((center, half));
        out.push(GamesBuilding {
            id: format!("games{gid}"),
            kind: spec.kind,
            name: spec.name,
            poly: poly.to_vec(),
            center,
            prov: spec.prov,
        });
        gid += 1;
    }
    out
}

#[cfg(test)]
mod tests;
