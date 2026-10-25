//! Terrain-following lava flows, traced down the real height field from each
//! young vent (owner request 2026-10-25: *"Will the revised volcano system
//! also go with lava flows? Like in Iceland and Hawaii and such, not only at
//! waterline but also on land"*; `OUTSTANDING_WORK.md` row "Terrain-following
//! lava flows", step 1; recorded as owner Ruling DA).
//!
//! # What it does
//!
//! For every edifice of the volcano table ([`crate::PlacedEdifice`]) that is
//! active or dormant ([`volcano_attr::activity`]) and has a morphology, it
//! lays out a handful of lava flows -- each with an eruption age in years, a
//! vent, a length budget and a width -- and walks each one **downhill on the
//! world's own height field**: the flow turns toward the steepest descent
//! where the ground is steep (so it runs down valleys and hugs channels),
//! keeps its heading and spreads wider where the ground is flat (so it fans
//! out on plains), overtops rises lower than its own thickness, ponds and
//! stops in a closed hollow or once it stops finding lower ground, may split into a branch on a flat, and stops
//! where its length budget runs out or where it reaches the sea. The result
//! is a set of polylines (grid cells, with the distance along the flow and
//! the half width at every point) with a coarse spatial lattice over them,
//! so a renderer can ask, at any fractional grid position, which flows reach
//! it and how far inside each one it is.
//!
//! # Why this shape
//!
//! Before this the only lava on a map was `volcano_look`'s seven radial
//! colour ribbons per polygenetic cone, laid out on bearings inside the
//! edifice's own disc and blind to the ground: they could run uphill, never
//! left the cone and never reached a coast, which is exactly what the owner
//! asked about. Real basaltic flows are routed by topography -- they follow
//! the steepest path, are captured by valleys, and spread into wide lobes on
//! low slopes (e.g. Hawaiian channelised flows, USGS on Mauna Loa 1859 and
//! 1984; the 2014-15 Holuhraun field spread over about 84 km^2 on flat
//! ground). Steepest descent with momentum is the standard first-order
//! model behind lava-path forecasting (the "steepest descent" path tools
//! used for Hawaii and Etna, e.g. Favalli et al. 2005, DOWNFLOW); this is
//! that model with the length budget set from the eruption, not a
//! thermo-rheological one.
//!
//! # The length law (cited spans, labelled judgements)
//!
//! Flow length is controlled mainly by effusion rate (Walker 1973) or by
//! erupted volume (Malin 1980, *Lengths of Hawaiian lava flows*, Geology 8),
//! with a scatter of up to ten times in length for a given volume (Pinkerton
//! & Wilson 1994's re-analysis of Malin's data). The engine knows neither
//! effusion rate nor volume, so each flow draws its length log-uniformly from
//! a band per edifice type, the bands bracketing cited flows:
//!
//! | Type | Band, km | Cited anchors |
//! |---|---|---|
//! | Basaltic shield | 8-50 | Kilauea 2018 fissure 8: about 13 km to the ocean (USGS); Mauna Loa 1984: 25 km (USGS); Mauna Loa 1859: 51 km, the longest (USGS) |
//! | Basaltic stratocone (rift, hotspot) | 4-20 | judgement between the shield and the andesite bands |
//! | Andesitic stratocone | 1.5-8 | judgement: viscous lavas make short flows (Davidson & De Silva 2000, research §2.1) |
//! | Scoria cone (flows from the breached base) | 1-8 | judgement: Paricutin-scale cone-base flows |
//!
//! Widths are judgements too (whole `'a'a` flows "a few hundred metres
//! across", channels tens of metres wide, USGS/HVO and the lava-channel
//! literature): a half width of 2.5 % of a basalt flow's length (0.15-1.0
//! km) and 7 % of an andesite flow's (0.2-1.2 km), widening up to twice on
//! slopes under about 3 degrees.
//!
//! # Reproducibility
//!
//! Every choice -- how many flows, each one's age, length, width, vent and
//! wander -- comes from [`cartalith_noise::hash`] of the edifice's index
//! and recorded position, never from a random stream, and the walk reads
//! only the height field. So a world's flows are a pure function of its
//! saved volcano table and height field: no existing world's generation
//! moves (no draw is added to any stream), and a save reopens with the flows
//! it was drawn with.
//!
//! # Fissure fields (owner Ruling DB item 1, 2026-10-25)
//!
//! A [`crate::VolcanicSetting::Rift`] edifice that records its rift's trend
//! ([`crate::PlacedEdifice::rift_bearing`], read off the plate boundary at
//! generation) and is active or dormant also erupts from **fissures** along
//! that trend, as Iceland's rift volcanoes do: its newest eruption is a
//! fissure (Krafla 1975-84, Holuhraun 2014-15), with an older one beside it,
//! so the vent shows a fissure field. Each fissure is drawn as a few
//! en-echelon segments (the stepped pattern of rift-zone fissures) carrying a
//! spatter rampart ([`FlowSource::Fissure`]), and lava is fed from points
//! along it downhill as broad sheet flows ([`FlowSource::FissureFed`]). A
//! shield's rift-zone vents lie on the same recorded trend, so a shield, a
//! scoria cone and a fissure on one rift line up (the owner's co-occurrence
//! note: a shield coexists with cinder cones and fissures along its rift).
//! Placement follows the recorded tectonics only; nothing here adds an
//! edifice, and a table without the trend (every edifice that is not a
//! rift's, and every table saved before the ruling) draws no fissure.
//!
//! | Quantity | Value | Source |
//! |---|---|---|
//! | Fissure length | 1.5-12 km, log-uniform | judgement inside cited spans: Holuhraun 2014's eruptive fissure about 1.5 km (Sigmundsson et al. 2015, *Nature* 517), Krafla's fissures of 1975-84 up to about 10 km (Einarsson 1991); Laki 1783's 27 km (Thordarson & Self 1993) is left out as the outlier |
//! | Segments | 2-4, stepped 0.1-0.3 km sideways | judgement: en-echelon rift fissures (Gudmundsson 2000) |
//! | Rampart half width | 0.05-0.09 km | judgement: spatter ramparts metres to tens of metres high and tens to ~150 m wide |
//! | Fed flows | 4 on the newest fissure, 2 on the older | judgement |
//! | Fed flow length | 3-20 km, log-uniform | anchors: Holuhraun 2014-15's field about 17 km long over about 84 km^2 (Pedersen et al. 2017) |
//! | Fed flow half width | 6 % of its length, 0.4-1.5 km | judgement: sheet lava is broad (Holuhraun's field about 5 km across) |
//! | The older fissure | 0.8-3 km to one side, aged between the last eruption and the flank's building age | judgement: a fissure swarm is several km wide (Gudmundsson 2000) |
//!
//! # What it must never do
//!
//! - Never change the height field, the land mask or any water: it reads the
//!   field and returns geometry. A lava delta or bench that would move the
//!   coast is a separate, measured, generation-side step (the backlog row's
//!   step 3), not this module.
//! - Never trace from an edifice without a morphology (the reference model's
//!   `kind: None`), an extinct one, or with an unknown or non-positive scale:
//!   [`LavaFlows::trace`] returns no flows rather than guessing a scale.

use crate::volcano_attr::{self, Activity};
use crate::{EdificeKind, PlacedEdifice};
use cartalith_jsmath::smoothstep;
use cartalith_noise::{hash, vnoise};

/// What the walk needs from the world: the height field it runs on and the
/// two scales that turn its units into kilometres and metres.
#[derive(Clone, Copy, Debug)]
pub struct TraceInput<'a> {
    /// The world's own height (`WorldState::field`, the height water is
    /// decided from), `gw * gh`, normalised units.
    pub field: &'a [f32],
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    /// Kilometres per grid cell (map width / `gw`).
    pub km_per_cell: f64,
    /// Metres per normalised height unit, `peak_m / (1 - sea_level)`
    /// (`geology::m_to_norm`'s relation).
    pub m_per_unit: f64,
}

/// One point on a traced flow's centre line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowPoint {
    /// Grid position, cells (fractional).
    pub x: f32,
    pub y: f32,
    /// Distance along the flow from its vent, km.
    pub s_km: f32,
    /// Half width of the flow here, km.
    pub hw_km: f32,
}

/// Where a flow's lava came out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowSource {
    /// A central or flank vent of an edifice.
    Vent,
    /// An eruptive fissure itself: a straight segment along the rift's trend
    /// whose "flow" is the spatter rampart along it (Ruling DB item 1). The
    /// renderer draws it as a rampart with a crack, glowing while active.
    Fissure,
    /// A sheet flow fed from a point on an eruptive fissure, traced downhill
    /// like a vent's flow.
    FissureFed,
}

/// One traced lava flow.
#[derive(Clone, Debug, PartialEq)]
pub struct LavaFlow {
    /// Index of the edifice in the volcano table that erupted it.
    pub edifice: u32,
    /// Years since this flow was erupted. The newest flow of an edifice is
    /// its last eruption ([`volcano_attr::years`]); older flows of a
    /// polygenetic edifice lie between that and its building age
    /// ([`flank_log_years`]).
    pub years: f64,
    /// Whether the flow is still moving (the newest flow of an
    /// [`Activity::Active`] edifice): the renderer draws it hot.
    pub active: bool,
    /// Basaltic (dark, fluid) or andesitic lava ([`volcano_attr::is_basaltic`]).
    pub basaltic: bool,
    pub source: FlowSource,
    /// The centre line, from the vent; at least two points.
    pub pts: Vec<FlowPoint>,
    /// The length actually traced, km (at most the budget; less where the
    /// flow ponded, left the grid or reached the sea).
    pub len_km: f64,
    /// Whether the flow ended at the sea.
    pub reached_sea: bool,
    /// Why the walk ended (a census reads it; the renderer does not).
    pub end: FlowEnd,
}

/// `log10` of the years a polygenetic edifice's outer flank has stood since
/// it was last built over, or `None` for a monogenetic scoria cone.
/// Judgements inside the cited spans (research §2.1: composite
/// "cone-building episodes generally <100 ka"; shields "many eruptions over
/// tens to hundreds of ka", Davidson & De Silva 2000): a stratocone's flank
/// about 30 ka (`10^4.5`), a shield's about 100 ka (`10^5`). The renderer's
/// flank age (`volcano_look::building_log_years`) reads this, so the flank
/// and its older flows share one figure.
pub fn flank_log_years(kind: EdificeKind) -> Option<f64> {
    match kind {
        EdificeKind::Cinder => None,
        EdificeKind::Strato => Some(4.5),
        EdificeKind::Shield => Some(5.0),
    }
}

/// How many flows an edifice of `kind` shows. Judgement: a shield is built of
/// many flows and shows the most (Kinney et al. 2015's chronosequence of
/// flows on one shield); a stratocone fewer; a scoria cone one or two from
/// its breached base.
fn flow_count(kind: EdificeKind) -> usize {
    match kind {
        EdificeKind::Shield => 6,
        EdificeKind::Strato => 4,
        EdificeKind::Cinder => 2,
    }
}

/// The length band in km for one edifice (module doc's table).
fn length_band(kind: EdificeKind, basaltic: bool) -> (f64, f64) {
    match (kind, basaltic) {
        (EdificeKind::Shield, _) => (8.0, 50.0),
        (EdificeKind::Strato, true) => (4.0, 20.0),
        (EdificeKind::Strato, false) => (1.5, 8.0),
        (EdificeKind::Cinder, _) => (1.0, 8.0),
    }
}

/// A flow's base half width, km, for a flow of `len_km`: viscous andesite
/// flows are short and broad (width about a seventh of their length), fluid
/// basalt ones long and narrow (a twentieth), each clamped to the "few
/// hundred metres" to 2 km of real flows (module doc). Judgements: the
/// width-to-length ratios are read off the cited flows' shapes, not fitted.
fn half_width_km(len_km: f64, basaltic: bool) -> f64 {
    if basaltic {
        (0.025 * len_km).clamp(0.15, 1.0)
    } else {
        (0.07 * len_km).clamp(0.2, 1.2)
    }
}

/// A fissure's length band, km (module doc's fissure table).
const FISSURE_LEN_KM: (f64, f64) = (1.5, 12.0);
/// A fissure-fed flow's length band, km (module doc's fissure table).
const FED_LEN_KM: (f64, f64) = (3.0, 20.0);

/// A fissure-fed sheet flow's base half width, km (module doc's table).
fn fed_half_width_km(len_km: f64) -> f64 {
    (0.06 * len_km).clamp(0.4, 1.5)
}

/// The walk's step, cells. Half a cell: fine enough that a flow follows a
/// one-cell valley, coarse enough that a 50 km flow at 0.39 km a cell is a
/// few hundred steps.
const STEP_CELLS: f64 = 0.5;
/// A point is kept every this many steps (one per cell).
const KEEP_EVERY: usize = 2;
/// Slope (rise over run) below which a flow stops following the gradient and
/// keeps its heading, and above which it follows the gradient fully: about
/// 0.5 and 5 degrees. Judgement: on Hawaiian shields flows run straight down
/// 5-10 degree flanks and wander across the near-flat coastal plain.
const FOLLOW_LO: f64 = 0.0087;
const FOLLOW_HI: f64 = 0.0875;
/// The slope at and above which a flow keeps its base width, and the floor
/// the widening is computed from (about 3 and 0.25 degrees): below 3 degrees
/// the flow spreads as `sqrt(3 deg / slope)`, at most [`WIDEN_MAX`] times
/// (3 on the first frames drew a disc at every flow's end on a flat).
/// Judgement standing in for the spreading of a gravity current on a low
/// slope (the Holuhraun field's lobes on the Dyngjusandur plain).
const WIDEN_REF: f64 = 0.0524;
const WIDEN_FLOOR: f64 = 0.0044;
const WIDEN_MAX: f64 = 2.0;
/// A flow overtops a rise lower than this, metres: a flow is metres to tens
/// of metres thick (Columbia River flows 10 to >300 ft, research §2.2;
/// Deccan sheet lobes 8-17 m). Judgement: 10 m.
const OVERTOP_M: f64 = 10.0;
/// A flow that has climbed or stalled this far (km, along its path) without
/// finding lower ground has ponded and stops. Judgement.
const POND_KM: f64 = 1.5;
/// A flow whose position is within [`STALL_NET_KM`] of where it was
/// [`STALL_KM`] of path ago is circling in a closed hollow: it has ponded
/// there and stops. Without this a flow in a basin wandered in loops (the
/// first 314159 plan, 2026-10-25); a rule on "no new lowest ground" instead
/// cut every flow on a near-flat plain short (the second plan: shield flows'
/// median 2.3 km against a 20 km budget). Judgements.
const STALL_KM: f64 = 4.0;
const STALL_NET_KM: f64 = 2.0;

/// Why a walk ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowEnd {
    /// Its length budget ran out.
    Budget,
    /// It reached the sea.
    Sea,
    /// It met rising ground it could not overtop or go round.
    Ponded,
    /// It circled in a closed hollow ([`STALL_KM`]).
    Stalled,
    /// It left the grid.
    Edge,
}
/// The most branches one flow may split into, and the chance per kilometre
/// of flat ground that it does. Judgement: enough that wide plains show a
/// fan of lobes, few enough that a flow stays one flow.
const MAX_BRANCHES: usize = 2;
const BRANCH_PER_KM: f64 = 0.05;
/// Slope under which a flow may branch (about 1.5 degrees).
const BRANCH_SLOPE: f64 = 0.026;
/// The lattice bucket edge, cells, of [`LavaFlows`]' segment index.
const BUCKET_CELLS: f64 = 8.0;
/// How far beyond its half width a renderer may ask about a flow, as a
/// multiple of the half width plus a fixed margin in km: the renderer's
/// lobes (up to 1.33 times the width) and ragged margin (up to 1.45 times
/// that) reach 1.93 half widths, and an active flow's halo a little more.
/// At 1.7 the widest lobes were cut by straight lines along the segment
/// lattice's bucket edges (the 2026-10-25 x64 frames of 314159);
/// `volcano_look`'s `the_lookup_reach_covers_the_widest_margin` holds the
/// two together.
pub const REACH_HW: f64 = 2.1;
pub const REACH_KM: f64 = 0.15;
/// The most flows [`LavaFlows::hits_at`] reports at one point. 8 since owner
/// Ruling DB item 1: a fissure's segments and the sheets fed from it share
/// one age and lie side by side, so at 4 the reach of two or three rampart
/// segments (which cover nothing there) truncated away the sheet that did,
/// and cut it along a straight line (the 2026-10-25 x64 frame of 314159).
/// The renderer still draws at most two.
pub const MAX_HITS: usize = 8;

/// A world's traced flows with a lattice over their segments.
#[derive(Clone, Debug, Default)]
pub struct LavaFlows {
    flows: Vec<LavaFlow>,
    km_per_cell: f64,
    bw: usize,
    bh: usize,
    /// Per bucket, `(flow, segment)` pairs whose reach overlaps it.
    buckets: Vec<Vec<(u32, u32)>>,
}

/// One flow near a point: how far the point is from its centre line and
/// where along it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowHit {
    /// Index into [`LavaFlows::flows`].
    pub flow: usize,
    /// Distance from the centre line, km.
    pub d_km: f64,
    /// Distance along the flow at the nearest centre-line point, km.
    pub s_km: f64,
    /// Half width of the flow there, km.
    pub hw_km: f64,
}

/// Bilinear height at a fractional grid position, clamped to the grid.
fn height(inp: &TraceInput, x: f64, y: f64) -> f64 {
    let (gw, gh) = (inp.gw, inp.gh);
    let x = x.clamp(0.0, (gw - 1) as f64);
    let y = y.clamp(0.0, (gh - 1) as f64);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(gw - 1), (y0 + 1).min(gh - 1));
    let (tx, ty) = (x - x0 as f64, y - y0 as f64);
    let f = |xx: usize, yy: usize| inp.field[yy * gw + xx] as f64;
    (f(x0, y0) * (1.0 - tx) + f(x1, y0) * tx) * (1.0 - ty) + (f(x0, y1) * (1.0 - tx) + f(x1, y1) * tx) * ty
}

/// The height gradient, units per cell, by central differences three cells
/// apart: a little smoother than the grid's own one-cell slope, so a flow is
/// steered by the valley it is in rather than by one cell's noise.
fn gradient(inp: &TraceInput, x: f64, y: f64) -> (f64, f64) {
    ((height(inp, x + 1.5, y) - height(inp, x - 1.5, y)) / 3.0, (height(inp, x, y + 1.5) - height(inp, x, y - 1.5)) / 3.0)
}

/// The most a flow's heading turns in one step, radians (about 34 degrees a
/// cell). Without a cap the walk flipped between neighbouring headings at
/// every step in a narrow valley, so its path ran about twice its
/// displacement and a flow drew half its length (the 2026-10-25 314159
/// overlay). Judgement: a lava flow has inertia and turns over a few widths.
const MAX_TURN: f64 = 0.3;

/// `from` turned toward `to` by at most `max` radians.
fn turn_toward(from: (f64, f64), to: (f64, f64), max: f64) -> (f64, f64) {
    let a = (from.0 * to.1 - from.1 * to.0).atan2(from.0 * to.0 + from.1 * to.1);
    if a.abs() <= max {
        to
    } else {
        rotate(from, max * a.signum())
    }
}

fn rotate((x, y): (f64, f64), a: f64) -> (f64, f64) {
    let (s, c) = a.sin_cos();
    (x * c - y * s, x * s + y * c)
}

fn normalise((x, y): (f64, f64)) -> Option<(f64, f64)> {
    let l = x.hypot(y);
    (l > 1e-12 && l.is_finite()).then(|| (x / l, y / l))
}

/// A walk to make: where it starts, which way, how far and how wide.
struct Walk {
    x: f64,
    y: f64,
    dir: (f64, f64),
    budget_km: f64,
    hw0_km: f64,
    salt: i32,
    /// Distance already travelled by the parent flow at the branch point, so
    /// a branch's `s_km` continues the parent's (its cooling and its
    /// pahoehoe-to-'a'a change read the distance from the vent).
    s0_km: f64,
    may_branch: bool,
}

/// Walk one flow; returns its points, length and whether it reached the sea,
/// and pushes any branches onto `out_branches`.
fn walk(inp: &TraceInput, w: &Walk, out_branches: &mut Vec<Walk>) -> Option<(Vec<FlowPoint>, f64, bool, FlowEnd)> {
    let kpc = inp.km_per_cell;
    let step_km = STEP_CELLS * kpc;
    let slope_k = inp.m_per_unit / (kpc * 1000.0);
    let overtop = OVERTOP_M / inp.m_per_unit;
    let max_steps = ((w.budget_km / step_km).ceil() as usize).min(20_000);
    let (mut x, mut y) = (w.x, w.y);
    let mut dir = w.dir;
    let mut hw = w.hw0_km;
    let mut s = 0.0;
    let mut pond = 0.0;
    // Positions every kilometre of path, for the circling test.
    let mut trail: Vec<(f64, f64)> = vec![(x, y)];
    let mut next_mark = 1.0;
    let mut end = FlowEnd::Budget;
    let mut branches = 0usize;
    let mut reached_sea = false;
    let mut pts = vec![FlowPoint { x: x as f32, y: y as f32, s_km: w.s0_km as f32, hw_km: hw as f32 }];
    for i in 0..max_steps {
        let (gx, gy) = gradient(inp, x, y);
        let slope = gx.hypot(gy) * slope_k;
        let down = normalise((-gx, -gy)).unwrap_or(dir);
        let follow = smoothstep(FOLLOW_LO, FOLLOW_HI, slope);
        // A slow lateral wander in kilometres along the path, strongest on
        // flat ground (where nothing else steers the flow): the meander a
        // flow shows across a plain.
        let wl = (6.0 * w.hw0_km).max(1.0);
        let wander = (vnoise((w.s0_km + s) / wl, 0.37, w.salt) - 0.5) * 2.0 * 0.35 * (1.0 - 0.7 * follow);
        let mut nd = normalise((dir.0 * (1.0 - 0.7 * follow) + down.0 * (0.2 + 0.7 * follow), dir.1 * (1.0 - 0.7 * follow) + down.1 * (0.2 + 0.7 * follow))).unwrap_or(dir);
        nd = rotate(nd, wander * 0.12);
        let h0 = height(inp, x, y);
        let mut best = (nd, height(inp, x + nd.0 * STEP_CELLS, y + nd.1 * STEP_CELLS));
        if best.1 > h0 + overtop {
            // Uphill ahead: look either side for the lowest way on, as a flow
            // is deflected along an obstacle's foot.
            for a in [0.3f64, -0.3, 0.6, -0.6, 0.9, -0.9] {
                let d = rotate(nd, a);
                let h = height(inp, x + d.0 * STEP_CELLS, y + d.1 * STEP_CELLS);
                if h < best.1 {
                    best = (d, h);
                }
            }
        }
        if best.1 > h0 + overtop {
            pond += step_km;
            if pond > POND_KM {
                end = FlowEnd::Ponded;
                break;
            }
        } else {
            pond = (pond - step_km).max(0.0);
        }
        dir = turn_toward(dir, best.0, MAX_TURN);
        x += dir.0 * STEP_CELLS;
        y += dir.1 * STEP_CELLS;
        s += step_km;
        if x < 0.0 || y < 0.0 || x > (inp.gw - 1) as f64 || y > (inp.gh - 1) as f64 {
            end = FlowEnd::Edge;
            break;
        }
        let target = w.hw0_km * (WIDEN_REF / slope.max(WIDEN_FLOOR)).sqrt().clamp(1.0, WIDEN_MAX);
        hw += (target - hw) * 0.06;
        if s >= next_mark {
            next_mark += 1.0;
            trail.push((x, y));
            let back = STALL_KM.round() as usize;
            if trail.len() > back {
                let (px, py) = trail[trail.len() - 1 - back];
                if (x - px).hypot(y - py) * kpc < STALL_NET_KM {
                    // Keep the flow only up to where it began to circle: the
                    // loop is the hollow the lava ponded in, drawn as the
                    // flow's toe there rather than as a coil (the coils read
                    // as a dark blob on the 2026-10-25 frames).
                    let cut = (w.s0_km + s - STALL_KM) as f32;
                    pts.retain(|p| p.s_km <= cut);
                    s -= STALL_KM;
                    end = FlowEnd::Stalled;
                    break;
                }
            }
        }
        let at_sea = height(inp, x, y) < inp.sea_level;
        if at_sea || i % KEEP_EVERY == KEEP_EVERY - 1 {
            pts.push(FlowPoint { x: x as f32, y: y as f32, s_km: (w.s0_km + s) as f32, hw_km: hw as f32 });
        }
        if at_sea {
            reached_sea = true;
            end = FlowEnd::Sea;
            break;
        }
        // A branch on flat ground: a share of what is left of the budget, on
        // a heading 25-45 degrees to one side.
        if w.may_branch && branches < MAX_BRANCHES && slope < BRANCH_SLOPE && s > 0.25 * w.budget_km {
            let km_bin = ((w.s0_km + s) / 1.0).floor() as i32;
            let prev_bin = ((w.s0_km + s - step_km) / 1.0).floor() as i32;
            if km_bin != prev_bin && hash(km_bin, w.salt, 7121) < BRANCH_PER_KM {
                let side = if hash(km_bin, w.salt, 7127) < 0.5 { -1.0 } else { 1.0 };
                let ang = side * (0.44 + 0.35 * hash(km_bin, w.salt, 7129));
                let rest = w.budget_km - s;
                out_branches.push(Walk { x, y, dir: rotate(dir, ang), budget_km: rest * (0.4 + 0.3 * hash(km_bin, w.salt, 7133)), hw0_km: w.hw0_km * 0.7, salt: w.salt.wrapping_add(7919 * (branches as i32 + 1)), s0_km: w.s0_km + s, may_branch: false });
                branches += 1;
            }
        }
    }
    if pts.len() < 2 {
        return None;
    }
    Some((pts, s, reached_sea, end))
}

/// A per-edifice salt from its index and recorded position, so two tables
/// that differ only in order or position never share flows. The position
/// is read at 1/16 cell, which the saved `f64` reproduces exactly.
fn edifice_salt(k: usize, e: &PlacedEdifice) -> i32 {
    (k as i32).wrapping_mul(7919) ^ ((e.cx * 16.0) as i32).wrapping_mul(31) ^ ((e.cy * 16.0) as i32).wrapping_mul(131)
}

/// One eruptive fissure of edifice `edifice` (`e`) along trend `bearing`,
/// `off_km` to the side of the vent, `years` old: its en-echelon rampart
/// segments ([`FlowSource::Fissure`]) and `fed` sheet flows traced downhill
/// from points along it ([`FlowSource::FissureFed`]), pushed onto `flows`.
/// Every choice is a hash of `salt` (module doc, Reproducibility). A segment
/// is cut where it leaves the land (a fissure under the sea erupts no
/// subaerial lava), and dropped when fewer than two of its points are dry.
#[allow(clippy::too_many_arguments)]
fn fissure(inp: &TraceInput, flows: &mut Vec<LavaFlow>, edifice: u32, e: &PlacedEdifice, bearing: f64, years: f64, active: bool, off_km: f64, fed: usize, salt: i32) {
    let kpc = inp.km_per_cell;
    let h = |j: i32, s: i32| hash(salt, j, s);
    let (ux, uy) = (bearing.cos(), bearing.sin());
    let (nx, ny) = (-uy, ux);
    let len_km = FISSURE_LEN_KM.0 * (FISSURE_LEN_KM.1 / FISSURE_LEN_KM.0).powf(h(0, 11));
    // The fissure's middle: beside the vent by `off_km`, slid along the trend
    // by up to a third of its length (a fissure rarely centres on the cone).
    let slide = (h(0, 13) - 0.5) * 0.66 * len_km;
    let (mx, my) = (e.cx + (nx * off_km + ux * slide) / kpc, e.cy + (ny * off_km + uy * slide) / kpc);
    let nseg = 2 + (h(0, 17) * 3.0) as usize;
    let seg_km = len_km / nseg as f64;
    let step_km = (STEP_CELLS * kpc).max(1e-6);
    let mut line: Vec<(f64, f64, f64)> = Vec::new();
    for sg in 0..nseg {
        // En echelon: each segment stepped sideways, alternating, and a
        // tenth longer so neighbours overlap a little.
        let step = (0.1 + 0.2 * h(sg as i32, 19)) * if sg % 2 == 0 { 1.0 } else { -1.0 };
        let hw = 0.05 + 0.04 * h(sg as i32, 23);
        let a = -0.5 * len_km + sg as f64 * seg_km - 0.05 * seg_km;
        let n = ((1.1 * seg_km / step_km).ceil() as usize).max(2);
        let mut pts: Vec<FlowPoint> = Vec::new();
        let push = |pts: &mut Vec<FlowPoint>, flows: &mut Vec<LavaFlow>| {
            if pts.len() >= 2 {
                let l = pts.last().unwrap().s_km - pts[0].s_km;
                flows.push(LavaFlow { edifice, years, active, basaltic: true, source: FlowSource::Fissure, pts: std::mem::take(pts), len_km: l as f64, reached_sea: false, end: FlowEnd::Budget });
            } else {
                pts.clear();
            }
        };
        for i in 0..=n {
            let t = a + 1.1 * seg_km * i as f64 / n as f64;
            let (x, y) = (mx + (ux * t + nx * step) / kpc, my + (uy * t + ny * step) / kpc);
            let inside = x >= 0.0 && y >= 0.0 && x <= (inp.gw - 1) as f64 && y <= (inp.gh - 1) as f64;
            if inside && height(inp, x, y) >= inp.sea_level {
                pts.push(FlowPoint { x: x as f32, y: y as f32, s_km: (t - a) as f32, hw_km: hw as f32 });
                line.push((x, y, t));
            } else {
                push(&mut pts, flows);
            }
        }
        push(&mut pts, flows);
    }
    if line.is_empty() {
        return;
    }
    // Sheet flows fed from points spread along the dry part of the fissure,
    // each leaving downhill (or, on dead-flat ground, square to the fissure).
    for j in 0..fed {
        let pick = ((j as f64 + 0.15 + 0.7 * h(j as i32, 29)) / fed as f64 * line.len() as f64) as usize;
        let (x, y, _) = line[pick.min(line.len() - 1)];
        let (gx, gy) = gradient(inp, x, y);
        let side = if h(j as i32, 31) < 0.5 { -1.0 } else { 1.0 };
        let dir = normalise((-gx, -gy)).unwrap_or((nx * side, ny * side));
        let budget = FED_LEN_KM.0 * (FED_LEN_KM.1 / FED_LEN_KM.0).powf(h(j as i32, 37));
        let hw0 = fed_half_width_km(budget) * (0.8 + 0.4 * h(j as i32, 41));
        let mut queue = vec![Walk { x, y, dir, budget_km: budget, hw0_km: hw0, salt: salt.wrapping_add(j as i32 * 104_729), s0_km: 0.0, may_branch: true }];
        while let Some(w) = queue.pop() {
            let mut more = Vec::new();
            if let Some((pts, len, sea, end)) = walk(inp, &w, &mut more) {
                flows.push(LavaFlow { edifice, years, active, basaltic: true, source: FlowSource::FissureFed, pts, len_km: w.s0_km + len, reached_sea: sea, end });
            }
            queue.extend(more);
        }
    }
}

impl LavaFlows {
    /// Traces every active or dormant edifice's flows (module doc). An empty
    /// set when the scales are unusable or the field is short -- never a
    /// guessed scale.
    pub fn trace(edifices: &[PlacedEdifice], inp: TraceInput) -> LavaFlows {
        let ok = inp.gw >= 2 && inp.gh >= 2 && inp.field.len() >= inp.gw * inp.gh && inp.km_per_cell.is_finite() && inp.km_per_cell > 0.0 && inp.m_per_unit.is_finite() && inp.m_per_unit > 0.0;
        if !ok {
            return LavaFlows::default();
        }
        let mut flows = Vec::new();
        for (k, e) in edifices.iter().enumerate() {
            let Some(kind) = e.kind else { continue };
            if !(e.cx.is_finite() && e.cy.is_finite() && e.r.is_finite() && e.r > 0.0) {
                continue;
            }
            let act = volcano_attr::activity(e);
            if act == Activity::Extinct {
                continue;
            }
            let Some(basaltic) = volcano_attr::is_basaltic(e) else { continue };
            let Some((_, rim)) = volcano_attr::zones(e) else { continue };
            let lg = volcano_attr::years(e).log10();
            let lb = flank_log_years(kind).filter(|&b| b > lg).unwrap_or(lg);
            let (lmin, lmax) = length_band(kind, basaltic);
            let salt0 = edifice_salt(k, e);
            let h = |j: usize, s: i32| hash(salt0, j as i32, s);
            // A shield's two rift zones (opposite bearings, a little bent):
            // its flank vents lie along them (Hawaiian shields' rift zones).
            // On a rift edifice that recorded its trend (Ruling DB item 1)
            // they lie along the plate boundary's own trend; otherwise on a
            // hashed bearing, as before the ruling.
            let rift = e.rift_bearing.unwrap_or_else(|| std::f64::consts::TAU * h(0, 9001));
            // With a recorded trend the edifice's newest eruption is its
            // fissure (below), so none of its vent flows is the newest.
            let fissured = e.rift_bearing.is_some() && e.setting == crate::VolcanicSetting::Rift && basaltic;
            for j in 0..flow_count(kind) {
                let u_len = h(j, 9011);
                let budget = lmin * (lmax / lmin).powf(u_len);
                let hw0 = half_width_km(budget, basaltic) * (0.8 + 0.4 * h(j, 9013));
                // Age: the newest flow is the last eruption; the rest spread
                // between it and the flank's building age, weighted young.
                let newest = j == 0 && !fissured;
                let lgj = if newest { lg } else { lg + (lb - lg) * h(j, 9017).powf(0.6) };
                let (t_vent, bearing) = match kind {
                    EdificeKind::Shield if j % 3 != 2 => {
                        let side = if j % 2 == 0 { 0.0 } else { std::f64::consts::PI };
                        (0.05 + 0.5 * h(j, 9019), rift + side + 0.35 * (h(j, 9023) - 0.5))
                    }
                    EdificeKind::Cinder => (0.9, std::f64::consts::TAU * h(j, 9029)),
                    _ => ((rim * 1.05).min(0.95), std::f64::consts::TAU * h(j, 9029)),
                };
                let dir = (bearing.cos(), bearing.sin());
                let (x, y) = (e.cx + dir.0 * e.r * t_vent, e.cy + dir.1 * e.r * t_vent);
                // A vent under the sea erupts no subaerial flow (its lava is
                // pillow basalt this map does not draw).
                if height(&inp, x, y) < inp.sea_level {
                    continue;
                }
                let salt = salt0.wrapping_add(j as i32 * 104_729);
                let mut queue = vec![Walk { x, y, dir, budget_km: budget, hw0_km: hw0, salt, s0_km: 0.0, may_branch: true }];
                while let Some(w) = queue.pop() {
                    let mut more = Vec::new();
                    if let Some((pts, len, sea, end)) = walk(&inp, &w, &mut more) {
                        flows.push(LavaFlow { edifice: k as u32, years: if newest { volcano_attr::years(e) } else { 10f64.powf(lgj) }, active: newest && act == Activity::Active, basaltic, source: FlowSource::Vent, pts, len_km: w.s0_km + len, reached_sea: sea, end });
                    }
                    queue.extend(more);
                }
            }
            if fissured && let Some(b) = e.rift_bearing {
                // The newest fissure (the last eruption) and an older one
                // beside it (module doc's fissure table).
                let lg_old = lg + (lb - lg) * h(0, 9101).powf(0.6);
                let side = if h(0, 9103) < 0.5 { -1.0 } else { 1.0 };
                let off_km = side * (0.8 + 2.2 * h(0, 9107));
                for (f, (years, active, off, fed)) in [(volcano_attr::years(e), act == Activity::Active, 0.0, 4usize), (10f64.powf(lg_old), false, off_km, 2)].into_iter().enumerate() {
                    fissure(&inp, &mut flows, k as u32, e, b, years, active, off, fed, salt0.wrapping_add(31_337 * (f as i32 + 1)));
                }
            }
        }
        LavaFlows::index(flows, inp.gw, inp.gh, inp.km_per_cell)
    }

    /// Builds the segment lattice over `flows`.
    fn index(flows: Vec<LavaFlow>, gw: usize, gh: usize, km_per_cell: f64) -> LavaFlows {
        let bw = ((gw as f64 / BUCKET_CELLS).ceil() as usize).max(1);
        let bh = ((gh as f64 / BUCKET_CELLS).ceil() as usize).max(1);
        let mut buckets = vec![Vec::new(); bw * bh];
        for (f, fl) in flows.iter().enumerate() {
            for (s, seg) in fl.pts.windows(2).enumerate() {
                let reach = (seg[0].hw_km.max(seg[1].hw_km) as f64 * REACH_HW + REACH_KM) / km_per_cell;
                let b = |v: f64, n: usize| ((v / BUCKET_CELLS).floor().max(0.0) as usize).min(n - 1);
                let (x0, x1) = (b((seg[0].x.min(seg[1].x)) as f64 - reach, bw), b((seg[0].x.max(seg[1].x)) as f64 + reach, bw));
                let (y0, y1) = (b((seg[0].y.min(seg[1].y)) as f64 - reach, bh), b((seg[0].y.max(seg[1].y)) as f64 + reach, bh));
                for by in y0..=y1 {
                    for bx in x0..=x1 {
                        buckets[by * bw + bx].push((f as u32, s as u32));
                    }
                }
            }
        }
        LavaFlows { flows, km_per_cell, bw, bh, buckets }
    }

    /// The traced flows.
    pub fn flows(&self) -> &[LavaFlow] {
        &self.flows
    }

    /// Whether there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }

    /// Up to [`MAX_HITS`] flows whose reach (`REACH_HW` half widths plus
    /// `REACH_KM`) covers the fractional grid position `(x, y)`: per flow
    /// its nearest centre-line point (by plain distance), **youngest flow first** (a younger
    /// flow lies on top of an older one), ties by table order. Empty off
    /// every flow.
    pub fn hits_at(&self, x: f64, y: f64) -> Vec<FlowHit> {
        let mut out: Vec<FlowHit> = Vec::new();
        if self.flows.is_empty() || !(x.is_finite() && y.is_finite()) || x < -BUCKET_CELLS || y < -BUCKET_CELLS {
            return out;
        }
        let bx = ((x / BUCKET_CELLS).floor().max(0.0) as usize).min(self.bw - 1);
        let by = ((y / BUCKET_CELLS).floor().max(0.0) as usize).min(self.bh - 1);
        let kpc = self.km_per_cell;
        for &(f, s) in &self.buckets[by * self.bw + bx] {
            let fl = &self.flows[f as usize];
            let (a, b) = (fl.pts[s as usize], fl.pts[s as usize + 1]);
            let (ax, ay, bxx, byy) = (a.x as f64, a.y as f64, b.x as f64, b.y as f64);
            let (dx, dy) = (bxx - ax, byy - ay);
            let l2 = dx * dx + dy * dy;
            let t = if l2 > 0.0 { (((x - ax) * dx + (y - ay) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let d_km = (x - (ax + dx * t)).hypot(y - (ay + dy * t)) * kpc;
            let hw = a.hw_km as f64 + (b.hw_km as f64 - a.hw_km as f64) * t;
            if d_km > hw * REACH_HW + REACH_KM {
                continue;
            }
            let hit = FlowHit { flow: f as usize, d_km, s_km: a.s_km as f64 + (b.s_km as f64 - a.s_km as f64) * t, hw_km: hw };
            match out.iter_mut().find(|h| h.flow == hit.flow) {
                // The nearest centre-line point (plain distance): the
                // distance to a polyline is continuous, so the margin drawn
                // from it is too. Choosing by distance over half width
                // instead switched segments wherever two widths met and cut
                // the margin into spikes and blocks (the 2026-10-25 x64
                // coverage image of 314159).
                Some(h) => {
                    if hit.d_km < h.d_km {
                        *h = hit;
                    }
                }
                None => out.push(hit),
            }
        }
        out.sort_by(|p, q| self.flows[p.flow].years.total_cmp(&self.flows[q.flow].years).then(p.flow.cmp(&q.flow)));
        out.truncate(MAX_HITS);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VolcanicSetting;

    /// A plane falling to the east (x), below the sea at its east edge, with
    /// a valley running east along y = 40, so a flow released north of the
    /// valley must turn into it and follow it to the sea.
    fn plane_with_valley(gw: usize, gh: usize) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let base = 0.9 - 0.5 * x as f64 / gw as f64;
                // A valley along y = 40, 6 cells deep in the middle.
                let v = (-((y as f64 - 40.0) / 4.0).powi(2)).exp() * 0.05;
                f[y * gw + x] = (base - v) as f32;
            }
        }
        f
    }

    fn shield(cx: f64, cy: f64, years: f64) -> PlacedEdifice {
        PlacedEdifice { rift_bearing: None, setting: VolcanicSetting::Hotspot, cx, cy, r: 6.0, h: 0.1, kind: Some(EdificeKind::Shield), age: 0.0, years: Some(years), crater: None }
    }

    /// Protects Ruling DB item 1's fissure field. A rift shield that records
    /// a trend of 1.1 rad erupts rampart segments that each run along that
    /// trend (to 1e-4 rad, as an axis), are 1.5-12 km long in total (each
    /// segment at most a tenth longer than its share), and carry the
    /// edifice's own age and activity; sheet flows are fed from them, and
    /// with a fissure no vent flow is the newest or active. The same edifice
    /// without a trend (every table saved before the ruling) draws no fissure
    /// and is the pre-ruling trace bit for bit except for the setting; an
    /// extinct one draws nothing.
    #[test]
    fn a_recorded_rift_trend_draws_a_fissure_field_along_it() {
        let (gw, gh) = (160usize, 80usize);
        let f = plane_with_valley(gw, gh);
        let inp = TraceInput { field: &f, gw, gh, sea_level: 0.43, km_per_cell: 0.5, m_per_unit: 4000.0 };
        let b = 1.1f64;
        let mut e = shield(60.0, 30.0, 20.0);
        e.setting = VolcanicSetting::Rift;
        e.rift_bearing = Some(b);
        let fl = LavaFlows::trace(&[e], inp);
        let segs: Vec<&LavaFlow> = fl.flows().iter().filter(|l| l.source == FlowSource::Fissure).collect();
        assert!(!segs.is_empty(), "no fissure segment");
        for l in &segs {
            let (p, q) = (l.pts[0], *l.pts.last().unwrap());
            let a = ((q.y - p.y) as f64).atan2((q.x - p.x) as f64).rem_euclid(std::f64::consts::PI);
            let d = (a - b).abs().min(std::f64::consts::PI - (a - b).abs());
            assert!(d < 1e-4, "a segment at {a} against the trend {b}");
            assert!(l.pts.iter().all(|p| p.hw_km >= 0.05 && p.hw_km <= 0.09 + 1e-6));
        }
        let newest: Vec<&&LavaFlow> = segs.iter().filter(|l| l.years == 20.0).collect();
        assert!(!newest.is_empty() && newest.iter().all(|l| l.active), "the newest fissure is the last eruption, active");
        let total: f64 = newest.iter().map(|l| l.len_km).sum();
        assert!(total <= 12.0 * 1.1 + 1e-6 && total >= 1.5 * 0.5, "newest fissure {total} km");
        assert!(fl.flows().iter().any(|l| l.source == FlowSource::FissureFed), "no sheet fed from the fissure");
        assert!(fl.flows().iter().filter(|l| l.source == FlowSource::Vent).all(|l| !l.active && l.years > 20.0), "a vent flow kept the newest eruption");
        // Without the trend: no fissure, and the setting alone changes nothing.
        let mut old = e;
        old.rift_bearing = None;
        let fo = LavaFlows::trace(&[old], inp);
        assert!(fo.flows().iter().all(|l| l.source == FlowSource::Vent));
        let mut hot = old;
        hot.setting = VolcanicSetting::Hotspot;
        assert_eq!(fo.flows(), LavaFlows::trace(&[hot], inp).flows());
        let mut dead = e;
        dead.years = Some(2.0e6);
        assert!(LavaFlows::trace(&[dead], inp).is_empty());
        // The hit cap a fissure field needs (`MAX_HITS`' doc), a literal.
        assert_eq!(MAX_HITS, 8);
    }

    /// Protects the walk's physics on a fixture built for it: every flow
    /// runs downhill overall (its end lower than its vent), a flow released
    /// beside a valley ends up in it, flows that reach the east edge's sea
    /// say so, and widths widen only within the stated cap.
    #[test]
    fn flows_run_downhill_into_the_valley_and_to_the_sea() {
        let (gw, gh) = (160usize, 80usize);
        let f = plane_with_valley(gw, gh);
        // Sea at the east: the plane falls to 0.4 at x = gw.
        let inp = TraceInput { field: &f, gw, gh, sea_level: 0.43, km_per_cell: 0.5, m_per_unit: 4000.0 };
        let fl = LavaFlows::trace(&[shield(100.0, 34.0, 20.0)], inp);
        assert!(fl.flows().len() >= 6, "{} flows", fl.flows().len());
        let h = |p: &FlowPoint| height(&inp, p.x as f64, p.y as f64);
        for l in fl.flows() {
            let (a, b) = (l.pts[0], *l.pts.last().unwrap());
            assert!(h(&b) < h(&a), "a flow ended higher than it started: {a:?} -> {b:?}");
            for p in &l.pts {
                assert!(p.hw_km > 0.0 && (p.hw_km as f64) <= 2.4 + 1e-6, "width {} over 1 km x 1.2 jitter x 2 widening", p.hw_km);
            }
        }
        // The longest flow: in the valley (|y - 40| < 3) for most of its
        // second half, and at the sea if it got there.
        let long = fl.flows().iter().max_by(|a, b| a.len_km.total_cmp(&b.len_km)).unwrap();
        let half = &long.pts[long.pts.len() / 2..];
        let in_valley = half.iter().filter(|p| (p.y as f64 - 40.0).abs() < 3.0).count() as f64 / half.len() as f64;
        assert!(in_valley > 0.6, "the longest flow spent {in_valley:.2} of its second half in the valley");
        assert!(fl.flows().iter().any(|l| l.reached_sea), "no flow reached the sea on a plane falling into it");
        for l in fl.flows().iter().filter(|l| l.reached_sea) {
            assert!(h(l.pts.last().unwrap()) < 0.43);
        }
    }

    /// Protects reproducibility: the same table and field give the same
    /// flows bit for bit; an extinct edifice, one without a morphology and an
    /// unknown scale give none; and the newest flow of an active edifice is
    /// the only active one, at the edifice's own age.
    #[test]
    fn flows_are_a_pure_function_of_the_table_and_the_field() {
        let (gw, gh) = (160usize, 80usize);
        let f = plane_with_valley(gw, gh);
        let inp = TraceInput { field: &f, gw, gh, sea_level: 0.43, km_per_cell: 0.5, m_per_unit: 4000.0 };
        let t = [shield(100.0, 34.0, 20.0)];
        let (a, b) = (LavaFlows::trace(&t, inp), LavaFlows::trace(&t, inp));
        assert_eq!(a.flows(), b.flows());
        assert_eq!(a.flows().iter().filter(|l| l.active).count(), 1);
        let act = a.flows().iter().find(|l| l.active).unwrap();
        assert_eq!(act.years, 20.0);
        assert!(a.flows().iter().all(|l| l.years >= 20.0 && l.years <= 1e5 + 1.0));
        assert!(LavaFlows::trace(&[shield(100.0, 34.0, 2.0e6)], inp).is_empty(), "an extinct edifice erupts no flows");
        let mut none = shield(100.0, 34.0, 20.0);
        none.kind = None;
        assert!(LavaFlows::trace(&[none], inp).is_empty());
        assert!(LavaFlows::trace(&t, TraceInput { km_per_cell: 0.0, ..inp }).is_empty());
        assert!(LavaFlows::trace(&t, TraceInput { m_per_unit: f64::NAN, ..inp }).is_empty());
    }

    /// Protects the length law's bands (literals, the module doc's table):
    /// every flow's budget-limited length stays inside its band, and a long
    /// plane lets a shield's flows reach well past the andesite band.
    #[test]
    fn lengths_stay_in_their_bands() {
        let (gw, gh) = (400usize, 100usize);
        let mut f = vec![0.0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                f[y * gw + x] = (0.95 - 0.5 * x as f64 / gw as f64) as f32;
            }
        }
        let inp = TraceInput { field: &f, gw, gh, sea_level: 0.2, km_per_cell: 0.25, m_per_unit: 4000.0 };
        let fl = LavaFlows::trace(&[shield(20.0, 50.0, 20.0)], inp);
        let mains: Vec<&LavaFlow> = fl.flows().iter().filter(|l| l.pts[0].s_km == 0.0).collect();
        assert!(!mains.is_empty());
        for l in &mains {
            assert!(l.len_km <= 50.0 + 0.2, "a shield flow of {} km", l.len_km);
        }
        assert!(mains.iter().any(|l| l.len_km > 8.0), "no shield flow passed 8 km on an open plane");
        let mut strato = shield(20.0, 50.0, 20.0);
        strato.kind = Some(EdificeKind::Strato);
        strato.setting = VolcanicSetting::Arc;
        for l in LavaFlows::trace(&[strato], inp).flows() {
            assert!(l.len_km <= 8.0 + 0.2, "an andesite flow of {} km", l.len_km);
        }
    }

    /// Protects the walk's guards, each on a fixture built to reach it:
    /// - the turn cap: no flow on the valley plane turns more than two capped
    ///   steps' worth (0.6 rad, a literal) between kept points;
    /// - the circling stop: on a dead-flat plateau (no gradient, only the
    ///   wander steering) a flow never runs on as a coil -- each one's last
    ///   4 km cover at least 2 km of ground (literals);
    /// - the submarine vent: on a shield half in the sea, every flow starts
    ///   on land.
    #[test]
    fn the_walk_turns_smoothly_stops_circling_and_starts_on_land() {
        let (gw, gh) = (160usize, 80usize);
        let f = plane_with_valley(gw, gh);
        let inp = TraceInput { field: &f, gw, gh, sea_level: 0.43, km_per_cell: 0.5, m_per_unit: 4000.0 };
        let fl = LavaFlows::trace(&[shield(100.0, 34.0, 20.0)], inp);
        for l in fl.flows() {
            for w in l.pts.windows(3) {
                let (a, b) = (((w[1].x - w[0].x) as f64, (w[1].y - w[0].y) as f64), ((w[2].x - w[1].x) as f64, (w[2].y - w[1].y) as f64));
                let ang = (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1).abs();
                assert!(ang <= 0.6 + 1e-3, "a flow turned {ang:.2} rad between kept points");
            }
        }
        // The plateau: flat at 0.9, its edge far beyond any flow.
        let (pw, ph) = (400usize, 400usize);
        let flat = vec![0.9f32; pw * ph];
        let pin = TraceInput { field: &flat, gw: pw, gh: ph, sea_level: 0.2, km_per_cell: 0.5, m_per_unit: 4000.0 };
        let mut big = shield(200.0, 200.0, 20.0);
        big.r = 10.0;
        let pf = LavaFlows::trace(&[big], pin);
        assert!(!pf.is_empty());
        for l in pf.flows() {
            let end = l.pts.last().unwrap();
            if let Some(p4) = l.pts.iter().rev().find(|p| (end.s_km - p.s_km) as f64 >= 4.0) {
                let net = (end.x - p4.x).hypot(end.y - p4.y) as f64 * 0.5;
                assert!(net >= 2.0, "a flow coiled: its last 4 km cover {net:.2} km ({:?})", l.end);
            }
        }
        // Half in the sea: the plane's sea starts at x = 147.
        let island = LavaFlows::trace(&[shield(147.0, 34.0, 20.0)], inp);
        assert!(!island.is_empty(), "premise: the land half erupts");
        for l in island.flows() {
            let p = l.pts[0];
            assert!(height(&inp, p.x as f64, p.y as f64) >= 0.43, "a flow starts under the sea at {p:?}");
        }
    }

    /// Protects `hits_at`: a point on a flow's centre line reports it at
    /// distance about 0, a point far off reports nothing, and hits come
    /// youngest first.
    #[test]
    fn hits_report_the_nearest_point_youngest_first() {
        let (gw, gh) = (160usize, 80usize);
        let f = plane_with_valley(gw, gh);
        let inp = TraceInput { field: &f, gw, gh, sea_level: 0.43, km_per_cell: 0.5, m_per_unit: 4000.0 };
        let fl = LavaFlows::trace(&[shield(100.0, 34.0, 20.0)], inp);
        let l0 = &fl.flows()[0];
        let p = l0.pts[l0.pts.len() / 2];
        let hits = fl.hits_at(p.x as f64, p.y as f64);
        assert!(hits.iter().any(|h| h.flow == 0 && h.d_km < 1e-6));
        for w in hits.windows(2) {
            assert!(fl.flows()[w[0].flow].years <= fl.flows()[w[1].flow].years);
        }
        assert!(fl.hits_at(150.0, 5.0).is_empty() || fl.flows().iter().any(|l| l.pts.iter().any(|q| (q.x as f64 - 150.0).abs() < 6.0 && (q.y as f64 - 5.0).abs() < 6.0)));
    }
}
