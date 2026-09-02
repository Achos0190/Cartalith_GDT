//! Milestone 9 — water infrastructure (reference lines 28967-29154).
//!
//! Quays, back streets, herringbone stubs, the harbour road, timber piers, the
//! breakwater mole and the three harbour-defence repertoires; then the two
//! river-crossing stages — the synthetic path's automatic spans
//! ([`add_river_bridges`]) and the real-water path's *recorded* ones
//! ([`detect_river_crossings`]).
//!
//! ## `distToLine` is not here
//!
//! The scope document's range for this milestone opens on `distToLine`
//! (reference line 28971), and it is **already ported** — milestone 7 needed it
//! for `grow`'s harbour-distance term and brought it across as
//! [`crate::growth::dist_to_line`]. Nothing in this module calls it; it sits
//! inside the harbour block comment in the reference only because that is where
//! its one caller's neighbour happens to be. Porting it again would give the
//! crate two copies of a three-line function, which is exactly what milestone
//! 7's own forward note asked milestone 9 not to do.
//!
//! ## The runtime ordering constraint, which is not a porting order
//!
//! **[`detect_river_crossings`] must RUN after milestone 11's cleanup passes**
//! — specifically after `removeWaterCrossings`, which culls every unbridged
//! crossing away from `site.bridgePt`. The reference's own header (line 29126)
//! is explicit about why: the function records a bridge wherever a *live* road
//! meets the centreline, so running it before the cull would record spans on
//! roads that are about to be deleted. That constrains where milestone 16's
//! `generate()` calls it from; it does not constrain when it is ported, and
//! this module is written and tested without milestone 11 existing.
//!
//! [`add_river_bridges`] has the mirror-image constraint and it is a *site*
//! one, not an ordering one: it returns immediately on `site.usesRealWater`, so
//! the two functions are never both active on the same town. The synthetic
//! (headless-suite) path gets automatic spans; the real-map path gets recorded
//! ones or a ford.
//!
//! ## Three outputs the reference writes onto `site`, returned as values here
//!
//! `buildHarbour` stamps `site.harbourInvalid`, and `detectRiverCrossings`
//! writes `site.bridges` / `site.ford`. [`crate::site::Site`] is milestone 5's
//! module and models none of those fields, so this module returns them instead:
//! [`HarbourOutcome::Invalid`] carries the reason string verbatim, and
//! [`Crossings`] carries the bridges-or-ford choice as the three-way it is.
//! Nothing downstream of this milestone reads them inside block 4 — they are
//! for the host app's inspector and for the renderer — so returning them loses
//! nothing, and it keeps `Site` immutable through both calls.
//!
//! ## Where the RNG goes
//!
//! `stream(seed, 'harbour')` takes its draws in exactly this order, and the
//! branches matter because two of them are conditional:
//!
//! 1. `range(115, 150)` — the shore length `L`, once.
//! 2. `range(36, 46)` — the back-street offset `backD`, once.
//! 3. `int(2, 3)` — the base pier count, **only on a non-river-like site**
//!    (`rk` is `kind === 'river' || kind === 'riverthrough'`); a river-like one
//!    hard-codes 2 and draws nothing.
//! 4. `range(20, 32)` per pier — the pier length, again **only when `!rk`**.
//! 5. `range(30, 40)`, `range(38, 52)`, `range(28, 40)` — the three mole legs,
//!    **only on `kind === 'coast'`**.
//!
//! `stream(seed, 'bridges')` is created by `addRiverBridges` and then never
//! drawn from. Reproduced as a binding so the call site keeps matching the
//! reference line it is checked against; creating a substream has no side
//! effect, so it is unobservable either way.
//!
//! ## `site.kind` is compared as a string, twice
//!
//! Lines 29061 and 29081 both test `site.kind === 'coast'` — the first decides
//! whether a mole is built at all, the second picks `'molefort'` as the `auto`
//! defence. `rk` is a *different* string test (`'river'`/`'riverthrough'`) and
//! is **not** [`crate::site::Site::river_like`]: that flag is also true for a
//! real-water channel whose `kind` is something else. Both are computed here
//! from `kind` exactly as the reference computes them.

use crate::geom::{Vec2, js_cos, js_max, js_min, js_round, js_sin, seg_int, simplify};
use crate::graph::Graph;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::site::Site;
use cartalith_jsmath::js_truthy_num;

// ---------------------------------------------------------------------------
// Provenance strings, verbatim from the reference
// ---------------------------------------------------------------------------

/// Line 29012 — written on every quay segment and quoted inside the harbour's
/// own summary.
pub const QUAY_PROV: &str = "Quay: hard water edge of the harbour; the town turns its working front to the water (harbour-city family, lit. review §1.1 #22).";
/// Line 29031.
pub const BACK_PROV: &str = "Back street: the quay's landward twin; together they frame the warehouse blocks (harbour-city family, lit. review §1.1 #22).";
/// Line 29038.
pub const STUB_PROV: &str = "Quayside lane: perpendicular access inland from the quay (herringbone harbour fabric, lit. review §1.1 #22).";
/// Line 29044 — both halves of the harbour road carry it.
pub const HARBOUR_ROAD_PROV: &str = "Harbour road: the principal street from the quay to the market (break-of-bulk to point of sale, lit. review §5).";
/// Line 29057.
pub const PIER_PROV: &str = "Timber pier: mooring and unloading stage reaching into deeper water.";
/// Line 29084.
pub const CHAIN_PROV: &str = "Harbour chain raised between two mouth towers to bar hostile ships (Vitruvius; Kyrenia, the Golden Horn) — M-HARB-4.";
/// Line 29090.
pub const SEAWALL_PROV: &str = "Sea wall carried round the harbour with a water-gate — the 14th-c. fortified-port pattern that enclosed the basin — M-HARB-4.";
/// Line 29094.
pub const MOLEFORT_PROV: &str = "Mole-head fort/tower commanding the harbour mouth (the Hospitaller moles at Rhodes; lighthouse-forts) — M-HARB-4.";
/// Line 29117 — the synthetic river spans.
pub const RIVER_BRIDGE_PROV: &str = "Bridge: one of several crossings of a river that runs through the town (bridge-town family; M-REG-6).";
/// Line 29124.
pub const BRIDGE_APPROACH_PROV: &str = "Bridge approach linking the crossing into the town (M-REG-6).";
/// Line 29148 — the recorded (real-water) crossings.
pub const CROSSING_PROV: &str = "Bridge: a road genuinely crosses the river here — the crossing road IS the bridge (M-REG-6; S5 validity rule).";
/// Line 29154.
pub const FORD_PROV: &str = "Ford: no road crosses this river — the flattest-bank point stays an unbridged ford (M-REG-6; bridges must be justified by a crossing road).";

// ---------------------------------------------------------------------------
// buildHarbour
// ---------------------------------------------------------------------------

/// `buildHarbour`'s slice of the engine-wide `opts` object.
///
/// Both fields go through JS falsiness rather than presence: `harbourScale` is
/// read as `(opts && opts.harbourScale) || 1`, so `Some(0.0)` and `Some(NaN)`
/// both become 1, and `harbourDefence` as `(opts && opts.harbourDefence) ||
/// 'auto'`, so `Some("")` becomes `"auto"`. Milestone 16 will fold this into
/// the one options struct `generate()` passes everywhere; until then a
/// two-field struct is the whole surface this function reads.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HarbourOpts {
    /// `opts.harbourScale` — the port's population-derived size multiplier
    /// (`_umHarbourScale`). Clamped to `[0.5, 3]` after the falsiness default.
    pub harbour_scale: Option<f64>,
    /// `opts.harbourDefence` — `"auto"`, `"none"`, `"chain"`, `"seawall"` or
    /// `"molefort"`. Any other non-empty string reaches the `else` arm and
    /// builds a mole-head fort, which is the reference's own behaviour and is
    /// reproduced rather than validated.
    pub harbour_defence: Option<String>,
}

/// One timber jetty (line 29056). `a` is on the water edge, `b` is `len` metres
/// out into the water — `inland[qi] * -len`, so the pier runs *away* from land.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pier {
    pub a: Vec2,
    pub b: Vec2,
}

/// The harbour-defence work (lines 29075-29096), one variant per `mode`.
///
/// `towers` is the same `[eA, eB]` pair in two of the three, and is carried
/// per-variant rather than hoisted because `molefort` does not have it — the
/// reference's three object literals do not share a shape.
#[derive(Debug, Clone, PartialEq)]
pub enum Defence {
    /// `type: 'chain'` — two mouth towers and the chain slung between them
    /// through `mouthOut`.
    Chain { towers: [Vec2; 2], chain: [Vec2; 3] },
    /// `type: 'seawall'` — the water edge pushed 3 m seaward, split either side
    /// of a water-gate.
    Seawall {
        wall_a: Vec<Vec2>,
        wall_b: Vec<Vec2>,
        gate: Vec2,
        towers: [Vec2; 2],
    },
    /// `type: 'molefort'` — a fort on the mole head, or on the harbour mouth
    /// when there is no mole.
    Molefort { fort: Vec2, base: Vec2 },
}

impl Defence {
    /// The `defence.type` string, which the harbour's own provenance sentence
    /// concatenates (line 29098).
    pub fn type_str(&self) -> &'static str {
        match self {
            Defence::Chain { .. } => "chain",
            Defence::Seawall { .. } => "seawall",
            Defence::Molefort { .. } => "molefort",
        }
    }
    /// The provenance string this variant carries.
    pub fn prov(&self) -> &'static str {
        match self {
            Defence::Chain { .. } => CHAIN_PROV,
            Defence::Seawall { .. } => SEAWALL_PROV,
            Defence::Molefort { .. } => MOLEFORT_PROV,
        }
    }
}

/// `buildHarbour`'s return object (line 29097).
///
/// `quay` is the **simplified** quay polyline (`simplify(quay, 0.8)`), which is
/// not the same list the back street and the stubs are built off — those use the
/// unsimplified `quay`. That asymmetry is the reference's; see
/// [`build_harbour`].
#[derive(Debug, Clone, PartialEq)]
pub struct HarbourWorks {
    pub quay: Vec<Vec2>,
    pub piers: Vec<Pier>,
    /// The breakwater, `[p0, p1, p2, p3]`. Open coasts only.
    pub mole: Option<[Vec2; 4]>,
    /// `H.pt`, carried through unchanged.
    pub pt: Vec2,
    pub defence: Option<Defence>,
    /// The assembled sentence at line 29098, with both of its conditional
    /// clauses resolved.
    pub prov: String,
}

impl HarbourWorks {
    /// The view of this harbour that `grow` takes — [`crate::growth::GrowOpts`]
    /// wants a [`HarbourFront`](crate::growth::HarbourFront), and the quay is
    /// all of it that milestone 7 reads.
    pub fn front(&self) -> crate::growth::HarbourFront {
        crate::growth::HarbourFront { quay: self.quay.clone() }
    }
}

/// What `buildHarbour` produced, including the two ways it declines.
///
/// The reference returns `null` in four places and stamps `site.harbourInvalid`
/// on two of them. Splitting the stamped cases out is the whole reason this is
/// a three-way instead of an [`Option`]: `'unnavigable'` and `'cliff'` are
/// diagnostics the host app displays, and collapsing them into `None` would
/// throw the only record of *why* a waterfront town has no quay.
#[derive(Debug, Clone, PartialEq)]
pub enum HarbourOutcome {
    Built(Box<HarbourWorks>),
    /// `site.harbourInvalid = <reason>`, then `return null`. The reason is one
    /// of the reference's two literals, [`INVALID_UNNAVIGABLE`] or
    /// [`INVALID_CLIFF`].
    Invalid(&'static str),
    /// `return null` with nothing stamped: no harbour point at all (a landlocked
    /// site), or a shore stretch shorter than two points.
    None,
}

/// `site.harbourInvalid = 'unnavigable'` (line 29008) — no open water at the box
/// and no Strahler-3 stem.
pub const INVALID_UNNAVIGABLE: &str = "unnavigable";
/// `site.harbourInvalid = 'cliff'` (line 29009) — a steep real shore at the
/// harbour point.
pub const INVALID_CLIFF: &str = "cliff";

/// `dry(a, b)` (line 29017) — six probes along the segment, rejecting it if any
/// of them is inside the channel clearance or in the water.
///
/// **`t` is accumulated, not computed, and the accumulation is kept.**
/// `for(let t=0;t<=1;t+=0.2)` yields
/// `0, 0.2, 0.4, 0.6000000000000001, 0.8, 1` — the rounding error at the third
/// step cancels at the fourth, so it does reach `1` exactly and both endpoints
/// are probed. The interesting value is the fourth: `0.6000000000000001` is not
/// `0.6`, and `a.lerp(b, t)` at the two differs in the last bits. Rewriting
/// this as `for i in 0..=5 { let t = i as f64 / 5.0; … }` looks equivalent and
/// is not — it would probe `0.6`, and this engine's thresholds are compared
/// bit for bit. Accumulated here for that reason, and the sequence is pinned by
/// its own test.
fn dry(site: &Site, a: Vec2, b: Vec2, clear: f64) -> bool {
    let mut t = 0.0f64;
    while t <= 1.0 {
        let p = a.lerp(b, t);
        if site.river_dist(p) < clear || site.is_water(p) {
            return false;
        }
        t += 0.2;
    }
    true
}

/// `buildHarbour` (reference line 28974) — quay, back street, herringbone
/// stubs, harbour road, piers, mole and defence.
///
/// **`g` is mutated before the return value exists**, exactly as in
/// [`crate::plaza::build_plaza`]: the quay, back-street, stub and harbour-road
/// segments all go in through [`Graph::add_street`] and are therefore
/// planar-corrected, while the polylines this returns are the *pre-snap* ones.
/// The two disagree wherever `add_street` moved an endpoint, and the reference
/// returns the un-moved geometry.
///
/// Three list asymmetries are the reference's and are load-bearing:
///
/// - the **quay streets** are laid along `simplify(quay, 0.8)`, but the **back
///   street** and the **stubs** are built from the *unsimplified* `quay` (with
///   `inland[i]`, which is indexed in step with it). Simplifying first would
///   drop the very vertices `inland` is aligned to.
/// - the **back street** is filtered before it is simplified, so a run of
///   in-water points collapses to a straight segment across the gap rather than
///   to two separate runs.
/// - `stubIdx` is `[0, len/3, 2·len/3, len-1]` with **no de-duplication**. On a
///   two-point shore that is `[0, 0, 1, 1]` and the same two stubs are offered
///   twice; the second offer of each is a no-op only because
///   [`Graph::add_street`] finds the segment already there.
pub fn build_harbour(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    opts: Option<&HarbourOpts>,
) -> HarbourOutcome {
    let mut r = stream(seed, "harbour");
    // `rk` here is a STRING test on `site.kind`, not `site.river_like()`.
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let Some(h_pt) = site.harbour.pt else {
        return HarbourOutcome::None;
    };

    // v1.17 navigability guards. Guarded on `uses_real_water`, so the synthetic
    // headless path is byte-identical with them present.
    if site.uses_real_water {
        if !(site.sea_lake_cells >= 40.0 || site.water_order >= 3.0) {
            return HarbourOutcome::Invalid(INVALID_UNNAVIGABLE);
        }
        if site.slope(h_pt) > 0.5 {
            return HarbourOutcome::Invalid(INVALID_CLIFF);
        }
    }

    let line = &site.river;
    // `(opts && opts.harbourScale) || 1`, then clamped. `js_truthy_num` is what
    // turns a supplied `0` or `NaN` into the default 1.
    let hs_raw = opts
        .and_then(|o| o.harbour_scale)
        .filter(|v| js_truthy_num(*v))
        .unwrap_or(1.0);
    let hs = js_max(0.5, js_min(3.0, hs_raw));

    // gather waterline points along ~115-150 m of shore (x hs) centred on H.
    let l = r.range(115.0, 150.0) * hs;
    // `H.idx` indexes `site.river`. `build_site` never produces one outside it
    // (it is either `-1` with a `null` point, which returned above, or a real
    // shoreline index), and the reference would throw a TypeError on
    // `V.dist(undefined, …)` if it did. Declining is this port's one deviation
    // from that, and it is unreachable from `build_site`.
    let Ok(start) = usize::try_from(site.harbour.idx) else {
        return HarbourOutcome::None;
    };
    if start >= line.len() {
        return HarbourOutcome::None;
    }
    let (mut i0, mut i1) = (start, start);
    let (mut acc0, mut acc1) = (0.0f64, 0.0f64);
    while i0 > 0 && acc0 < l / 2.0 {
        acc0 += line[i0].dist(line[i0 - 1]);
        i0 -= 1;
    }
    while i1 < line.len() - 1 && acc1 < l / 2.0 {
        acc1 += line[i1].dist(line[i1 + 1]);
        i1 += 1;
    }
    let shore = &line[i0..=i1];
    if shore.len() < 2 {
        return HarbourOutcome::None;
    }

    // quay: the shoreline offset onto land. For rivers the polyline is the
    // CENTRELINE, so the bank is riverW/2 out; for coasts it IS the waterline.
    let inset = if rk { site.river_w / 2.0 + 4.0 } else { 5.0 };
    let mut quay: Vec<Vec2> = Vec::with_capacity(shore.len());
    let mut water_edge: Vec<Vec2> = Vec::with_capacity(shore.len());
    let mut inland: Vec<Vec2> = Vec::with_capacity(shore.len());
    for i in 0..shore.len() {
        let a = shore[i.saturating_sub(1)];
        let b = shore[usize::min(shore.len() - 1, i + 1)];
        let mut nl = (b - a).norm().rot90();
        if site.is_water(shore[i] + nl * (inset + 6.0)) {
            nl = nl * -1.0; // point inland
        }
        quay.push(shore[i] + nl * inset);
        water_edge.push(shore[i] + nl * (if rk { site.river_w / 2.0 } else { 0.0 }));
        inland.push(nl);
    }
    let quay_s = simplify(&quay, 0.8);

    // add the quay segment-wise, dropping any piece that corner-cuts across a
    // water bend.
    let clear = if rk { site.river_w / 2.0 + 1.5 } else { 2.0 };
    for w in quay_s.windows(2) {
        if dry(site, w[0], w[1], clear) {
            g.add_street(w[0].x, w[0].y, w[1].x, w[1].y, "quay", 7.0, 0, QUAY_PROV);
        }
    }

    // back street: the quay's landward twin ~40 m inland.
    let back_d = r.range(36.0, 46.0);
    let back: Vec<Vec2> = quay
        .iter()
        .zip(inland.iter())
        .map(|(q, n)| *q + *n * back_d)
        .filter(|p| !site.is_water(*p))
        .collect();
    let back_s = simplify(&back, 1.2);
    for w in back_s.windows(2) {
        if dry(site, w[0], w[1], clear) {
            g.add_street(w[0].x, w[0].y, w[1].x, w[1].y, "street", 4.0, 0, BACK_PROV);
        }
    }

    // herringbone stubs: perpendicular connectors closing the harbour blocks.
    let n_shore = shore.len();
    let stub_idx = [0usize, n_shore / 3, 2 * n_shore / 3, n_shore - 1];
    for qi in stub_idx {
        let e = quay[qi] + inland[qi] * (back_d + 6.0);
        if !site.is_water(e) && dry(site, quay[qi], e, clear) {
            g.add_street(quay[qi].x, quay[qi].y, e.x, e.y, "street", 4.0, 0, STUB_PROV);
        }
    }

    // harbour road: quay -> market, stepping inland first so it never runs
    // along the waterline. The first leg is laid unconditionally; only the long
    // run to the market is gated on `dry`.
    let mid_i = n_shore / 2;
    let way_pt = quay[mid_i] + inland[mid_i] * (back_d + 6.0);
    g.add_street(
        quay[mid_i].x,
        quay[mid_i].y,
        way_pt.x,
        way_pt.y,
        "primary",
        6.0,
        0,
        HARBOUR_ROAD_PROV,
    );
    if dry(site, way_pt, anchors.market, clear) {
        g.add_street(
            way_pt.x,
            way_pt.y,
            anchors.market.x,
            anchors.market.y,
            "primary",
            6.0,
            0,
            HARBOUR_ROAD_PROV,
        );
    }

    // piers: count scales with sqrt(harbourScale), length mildly too.
    let mut piers: Vec<Pier> = Vec::new();
    let n_p_base = if rk { 2.0 } else { r.int(2, 3) as f64 };
    let n_p = js_max(2.0, js_round(n_p_base * hs.sqrt()));
    let mut k = 0.0f64;
    while k < n_p {
        let qi = js_max(
            0.0,
            js_min(
                n_shore as f64 - 1.0,
                ((k + 0.7) * n_shore as f64 / (n_p + 0.6)).floor(),
            ),
        ) as usize;
        let len = if rk {
            js_min(site.river_w * 0.55, 13.0)
        } else {
            r.range(20.0, 32.0)
        } * js_min(1.7, hs.sqrt());
        piers.push(Pier { a: water_edge[qi], b: water_edge[qi] + inland[qi] * -len });
        k += 1.0;
    }

    // breakwater mole: only the open coast needs artificial shelter.
    let mut mole: Option<[Vec2; 4]> = None;
    if site.kind == "coast" {
        let ms = js_min(2.0, hs);
        let sea = inland[n_shore - 1] * -1.0;
        let rot = |v: Vec2, a: f64| {
            Vec2::new(
                v.x * js_cos(a) - v.y * js_sin(a),
                v.x * js_sin(a) + v.y * js_cos(a),
            )
        };
        let p0 = water_edge[n_shore - 1];
        let p1 = p0 + sea * (r.range(30.0, 40.0) * ms);
        let p2 = p1 + rot(sea, -0.9) * (r.range(38.0, 52.0) * ms);
        let p3 = p2 + rot(sea, -1.7) * (r.range(28.0, 40.0) * ms);
        mole = Some([p0, p1, p2, p3]);
    }

    // harbour defence. `waterEdge.length >= 2` is always true here — it has one
    // entry per shore point and `shore.len() < 2` returned above — but it is the
    // reference's guard and is kept as written.
    let def_mode: &str = opts
        .and_then(|o| o.harbour_defence.as_deref())
        .filter(|s| !s.is_empty())
        .unwrap_or("auto");
    let mut defence: Option<Defence> = None;
    if def_mode != "none" && water_edge.len() >= 2 {
        let e_a = water_edge[0];
        let e_b = water_edge[water_edge.len() - 1];
        let sd = (inland[0] + inland[n_shore - 1]).norm() * -1.0; // toward the water
        let mid = e_a.lerp(e_b, 0.5);
        let mouth_out = mid + sd * js_max(28.0, e_a.dist(e_b) * 0.35);
        let mode = if def_mode == "auto" {
            if site.kind == "coast" {
                "molefort"
            } else if site.through {
                "seawall"
            } else {
                "chain"
            }
        } else {
            def_mode
        };
        defence = Some(if mode == "chain" {
            Defence::Chain { towers: [e_a, e_b], chain: [e_a, mouth_out, e_b] }
        } else if mode == "seawall" {
            let wp: Vec<Vec2> = water_edge.iter().map(|p| *p + sd * 3.0).collect();
            let gi = wp.len() / 2;
            // `wp[gi] || mid`: `gi < wp.len()` always holds here (`wp.len() >= 2`
            // makes `gi >= 1` and `gi < wp.len()`), so the `|| mid` arm is
            // defensive in the reference and unreachable. Kept as the fallback
            // it is written as rather than as an index that could panic.
            let gate = wp.get(gi).copied().unwrap_or(mid);
            Defence::Seawall {
                wall_a: wp[..usize::max(1, gi)].to_vec(),
                wall_b: wp[usize::min(gi + 1, wp.len())..].to_vec(),
                gate,
                towers: [e_a, e_b],
            }
        } else {
            // molefort, and every unrecognised mode string with it.
            let head = mole.map_or(mouth_out, |m| m[3]);
            Defence::Molefort { fort: head, base: mole.map_or(e_b, |m| m[0]) }
        });
    }

    let prov = format!(
        "Harbour: sheltered landing with quay, piers{}{}.",
        if mole.is_some() {
            " and breakwater mole (open coasts need artificial shelter)"
        } else {
            " (the bay provides natural shelter)"
        },
        match &defence {
            Some(d) => format!("; protected by {}", d.type_str()),
            None => "; unprotected".to_string(),
        }
    );

    HarbourOutcome::Built(Box::new(HarbourWorks { quay: quay_s, piers, mole, pt: h_pt, defence, prov }))
}

// ---------------------------------------------------------------------------
// addRiverBridges
// ---------------------------------------------------------------------------

/// `addRiverBridges` (reference line 29101) — the synthetic path's automatic
/// spans across a river that runs through the town, each with a short approach
/// toward the market on either bank.
///
/// Returns without touching `g` in two cases, and both are v1.17 fixes rather
/// than defensive padding:
///
/// - **`site.uses_real_water`** — with real map water no span is free; a bridge
///   has to be justified by a road that genuinely crosses, which is what
///   [`detect_river_crossings`] records instead.
/// - **`site.river.len() < 3`** — a real river path clipped to the site box can
///   be exactly two points, and there is then no interior vertex to bridge.
///   Before the guard the reference read `site.river[i+1]` as `undefined` and
///   the resulting throw was swallowed by `_umModelFor`'s `try`/`catch` as a
///   model that silently never landed. Synthetic rivers are ~54 points, so the
///   headless suite never saw it.
pub fn add_river_bridges(seed: u32, site: &Site, anchors: &Anchors, g: &mut Graph, count: i64) {
    if site.uses_real_water {
        return;
    }
    // Created and never drawn from, exactly as at line 29109.
    let _r = stream(seed, "bridges");
    let n = site.river.len();
    if n < 3 {
        return;
    }
    let nf = n as f64;
    for k in 1..=count {
        let i = js_max(
            1.0,
            js_min(nf - 2.0, (nf * (0.28 + 0.44 * k as f64 / (count as f64 + 1.0))).floor()),
        ) as usize;
        let p = site.river[i];
        let a = site.river[i - 1];
        let b = site.river[i + 1];
        let nl = (b - a).norm().rot90();
        let half = site.river_w / 2.0 + 16.0;
        let p1 = p + nl * half;
        let p2 = p + nl * -half;
        g.add_street(p1.x, p1.y, p2.x, p2.y, "primary", 6.0, 0, RIVER_BRIDGE_PROV);
        // connect each bridgehead into the fabric so it is not a stranded
        // segment, and so later growth radiates from it.
        for end in [p1, p2] {
            let dm = (anchors.market - end).norm();
            let stub = end + dm * js_min(end.dist(anchors.market), 120.0);
            if !site.is_water(stub) {
                g.add_street(end.x, end.y, stub.x, stub.y, "street", 4.0, 0, BRIDGE_APPROACH_PROV);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// detectRiverCrossings
// ---------------------------------------------------------------------------

/// One recorded crossing (line 29147). `cls` is the *crossing road's* class —
/// the road is the bridge, so the span inherits whatever the road was, as the
/// same `&'static str` [`crate::graph::Edge`] carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bridge {
    pub pt: Vec2,
    /// The river's local direction at the crossing, from the centreline
    /// vertices either side of the crossed segment.
    pub dir: Vec2,
    pub cls: &'static str,
}

/// The unbridged crossing a through-town falls back to (line 29153).
///
/// `dir` is `site.bridgeDir`, which is [`None`] whenever `build_site` never set
/// one; the reference writes `undefined` into the field in that case rather
/// than skipping it, and this reproduces that.
#[derive(Debug, Clone, PartialEq)]
pub struct Ford {
    pub pt: Vec2,
    pub dir: Option<Vec2>,
}

/// What `detectRiverCrossings` writes onto the site — `site.bridges`, or
/// `site.ford`, or (the common case) neither.
#[derive(Debug, Clone, PartialEq)]
pub enum Crossings {
    /// `site.bridges = bridges` — at least one live non-quay road meets the
    /// centreline.
    Bridges(Vec<Bridge>),
    /// `site.ford = {…}` — no crossing road, and the town is a through one with
    /// a `bridgePt`.
    Ford(Ford),
    /// Nothing written: either a guard returned, or there were no crossings and
    /// the site is not a through town with a bridge point.
    None,
}

/// `detectRiverCrossings` (reference line 29134) — record where roads
/// **actually** cross the real river.
///
/// **Pure annotation: `g` is not mutated, and this takes it by shared
/// reference to make that a compile-time fact rather than a comment.**
///
/// Must be run on the FINAL street graph — see this module's header for the
/// ordering constraint, which milestone 16's `generate()` has to honour and
/// which nothing in this signature can enforce.
///
/// Three details worth naming:
///
/// - `e.cls === 'quay'` is skipped, because a quay runs *along* the water and
///   would otherwise register a crossing at every wobble of the centreline.
/// - the dedup is `< 80` metres against **already-accepted** bridges, in edge
///   order then segment order, so the first crossing of a stretch wins and the
///   result depends on `g.edges`' order. That order is the graph's construction
///   order and is itself golden-pinned by milestone 2.
/// - `n < 2` returns, so a one-point centreline is a no-op — but a two-point one
///   runs, with a single segment.
pub fn detect_river_crossings(site: &Site, g: &Graph) -> Crossings {
    if !site.uses_real_water || !site.real_river {
        return Crossings::None;
    }
    let line = &site.river;
    let n = line.len();
    if n < 2 {
        return Crossings::None;
    }
    let mut bridges: Vec<Bridge> = Vec::new();
    for e in &g.edges {
        if !e.alive || e.cls == "quay" {
            continue;
        }
        let a = g.nodes[e.a].pt();
        let b = g.nodes[e.b].pt();
        for i in 0..n - 1 {
            let Some(h) = seg_int(a, b, line[i], line[i + 1]) else {
                continue;
            };
            if bridges.iter().any(|br| br.pt.dist(h.pt) < 80.0) {
                continue;
            }
            let rd = (line[usize::min(i + 1, n - 1)] - line[i.saturating_sub(1)]).norm();
            bridges.push(Bridge { pt: h.pt, dir: rd, cls: e.cls });
        }
    }
    if !bridges.is_empty() {
        Crossings::Bridges(bridges)
    } else if site.through && let Some(pt) = site.bridge_pt {
        Crossings::Ford(Ford { pt, dir: site.bridge_dir })
    } else {
        Crossings::None
    }
}

#[cfg(test)]
mod tests;
