//! **The two river gates, measured before anything changes** (owner, Ruling
//! CF item 6, 2026-10-12: *"order-versus-discharge river gate and River
//! Transport discharge gate: measure first, report numbers, change nothing
//! yet"*; Ruling BV stands meanwhile). The write-up is
//! `RIVER_GATE_MEASUREMENTS.md` at the repository root.
//!
//! **Gate 1 -- which river mouths get a delta fan** (`RIVERS_IN_MAP_SCOPE.md`
//! owner question 3; `STATUS.md`'s RIM-4 entry: *"the order >= 3 gate skips
//! 155 mouths that clear the discharge bar ... owner call: gate on order,
//! discharge, or both"*). Today `cartalith-godot/src/river_delta.rs`
//! `delta_fans` admits a mouth when `own_order >= MIN_ORDER` (3) **and** its
//! discharge `>= MIN_DISCHARGE_FRAC` (2e-4) of the grid's cells. This harness
//! counts, over the app's own drawn network, what each of the four possible
//! rules (order only, discharge only, both = today, either) admits, and what
//! the two disagreement sets look like.
//!
//! **Gate 2 -- which channel cells River Transport may sail**
//! (`OUTSTANDING_WORK.md` "River Transport: gate navigability on discharge as
//! well as Strahler order?"). Today `cartalith-civ/src/river_class.rs`
//! `NAVIGABLE_ORDER` (3, the reference's `_civNavigableRiverDiscount` gate) is
//! the only test; there is no discharge gate. This harness counts, over
//! `cartalith_civ::fresh_river_channels` (the network
//! `story_bridge::jp_world_parts` builds for the planner), the navigable cells,
//! length and connectivity under order gates 2/3/4 and under candidate
//! discharge gates `Q >= k x river_flow_thresh` -- alone, OR-ed with order 3
//! ("as well as"), and AND-ed with it -- at k = 8 (the marshland rule's
//! existing `DELTA_FLOW_MULT`, the anchor) and +-25 %/+-50 % of it, plus wider
//! context values.
//!
//! **What it is a proxy for, honestly.** Gate 1 reads `River` entities
//! (`river_entities` + `river_draw_plan`, the comb harness's construction), not
//! `DrawnRun`s: `River::order` is the run's highest order (`DrawnRun::own_order`
//! also excludes a junction cell and is raised to its continuation group), and
//! `River::discharge` is the run's maximum (`delta_fans` takes the maximum over
//! the LAST drawn piece). For a shore mouth those coincide except for runs cut
//! at a lake or the antimeridian. "On a shore" is the comb harness's `shore`
//! (the outlet cell or a neighbour is water or at/below sea level), not
//! `DrawnRun::reach`. The owner-world calibration row (`RG_OWNER=1`) puts its
//! counts beside `STATUS.md`'s engine figures so the size of the proxy error is
//! visible.
//!
//! **What it must never do.** Change, or call anything that changes, generated
//! output: it reads generation and draw-plan functions and computes counts. It
//! pins no value; its only assertions guard that the fixture reached the code
//! (rivers exist, mouths exist, navigable cells exist). Never un-ignore it into
//! the default suite: it generates 36 worlds.
//!
//! ```text
//! cargo test --release -p cartalith-godot --test river_gate_measurement -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `RG_SEEDS` (comma list) overrides the twelve seeds; `RG_OWNER=1` adds the
//! owner's world (seed 246371, 2048x1311, 40 075 km, density 1.55). An
//! unparsable value panics rather than falling back.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use std::time::Instant;

use cartalith_engine::{WorldState, generate_terrain};
use cartalith_hydrology::{River, river_draw_plan, river_entities, river_flow_thresh, river_width_scale_k};

/// The delta gate's two constants, restated here as LITERALS on purpose: the
/// harness measures today's rule and must not silently follow a later edit of
/// `river_delta.rs` (whose constants are private to the cdylib in any case).
/// Source: `river_delta::MIN_ORDER`, `river_delta::MIN_DISCHARGE_FRAC`.
const DELTA_MIN_ORDER: i16 = 3;
const DELTA_MIN_DISCHARGE_FRAC: f32 = 2.0e-4;

/// The fixed seed matrix: the seeds this project's harnesses and audits
/// already use (246371 owner, 483920 and 912345 RIM-4 Part A, 24601 planner
/// audit, 404/505/808/707 planner probe, 71077345 river_stroke) plus three
/// arbitrary ones, chosen before any result was seen.
const SEEDS: [i32; 12] = [246_371, 483_920, 912_345, 24_601, 404, 505, 808, 707, 71_077_345, 1_001, 31_337, 123_456];

/// The three world shapes: `(gw, gh, km)`. 512x328 at 40 075 km is the world
/// Ruling CA's comb addendum and item 8 name; 384x246 at 800 km is the
/// default-km regional map the comb harness's second test uses; 1024x656 at
/// 40 075 km is the RIM-4 Part A second world's size.
const SHAPES: [(usize, usize, f64); 3] = [(512, 328, 40_075.0), (384, 246, 800.0), (1024, 656, 40_075.0)];

/// The anchor of the candidate discharge gate for River Transport, in
/// multiples of `river_flow_thresh`: `river_class::DELTA_FLOW_MULT` (8), the
/// "high flow" multiple the marshland rule in the same module already uses.
/// A labelled choice of anchor, not a recommendation; the sweep around it is
/// what the brief asked for (+-25 %, +-50 %).
const Q_ANCHOR: f64 = 8.0;

/// Env override, or `default`; an unparsable value panics.
fn env_list<T: std::str::FromStr + Copy>(key: &str, default: &[T]) -> Vec<T> {
    match std::env::var(key) {
        Ok(v) => v.split(',').map(|s| s.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse {s:?}"))).collect(),
        Err(_) => default.to_vec(),
    }
}

/// A world at the app's parameters (`params::defaults()`, integrated drainage
/// on) with the given size, km and river density. Returns the state, its wrap
/// flag and the generation time in seconds.
fn world(seed: i32, gw: usize, gh: usize, km: f64, density: f64) -> (WorldState, bool, f64, f64) {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = km;
    p.use_gpu = false;
    p.river_density = density;
    assert!(p.integrate_drainage, "the app's defaults must have integrated drainage on");
    let t = Instant::now();
    let ws = generate_terrain(&p);
    (ws, p.world, t.elapsed().as_secs_f64(), p.river_density)
}

/// Whether `c` or a neighbour is water or at/below sea level -- the comb
/// harness's `shore`, kept identical so the two harnesses count the same
/// mouths.
fn shore(c: usize, gw: usize, gh: usize, fld: &[f32], sea: f64, water: &[u8]) -> bool {
    let (x, y) = ((c % gw) as i64, (c / gw) as i64);
    (-1..=1).any(|dy| {
        (-1..=1).any(|dx| {
            let (nx, ny) = (x + dx, y + dy);
            nx >= 0 && ny >= 0 && (nx as usize) < gw && (ny as usize) < gh && {
                let q = ny as usize * gw + nx as usize;
                fld[q] as f64 <= sea || water[q] != 0
            }
        })
    })
}

/// Median of a list (NaN when empty -- printed as "-", never as a number).
fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] } else { 0.5 * (v[m - 1] + v[m]) }
}

/// A number, or "-" for NaN (an empty set has no median).
fn f(x: f64, prec: usize) -> String {
    if x.is_nan() { "-".into() } else { format!("{x:.prec$}") }
}

// ---------------------------------------------------------------------------
// Gate 1: delta-fan mouths.
// ---------------------------------------------------------------------------

/// One mouth's facts, as read for the gate.
#[derive(Clone, Copy, Debug)]
struct Mouth {
    order: i16,
    /// The run's largest discharge, in upstream (rain-weighted) cells.
    q: f64,
    /// `q` as a multiple of the world's river threshold (scale-free).
    q_thr: f64,
    len_cells: f64,
    tributaries: u32,
}

/// The pooled character of a set of mouths.
#[derive(Default, Clone, Debug)]
struct SetStats {
    n: usize,
    q_thr: Vec<f64>,
    len: Vec<f64>,
    order: Vec<f64>,
    no_trib: usize,
    order1: usize,
}

impl SetStats {
    fn add(&mut self, m: &Mouth) {
        self.n += 1;
        self.q_thr.push(m.q_thr);
        self.len.push(m.len_cells);
        self.order.push(m.order as f64);
        self.no_trib += (m.tributaries == 0) as usize;
        self.order1 += (m.order <= 1) as usize;
    }
    fn merge(&mut self, o: &SetStats) {
        self.n += o.n;
        self.q_thr.extend_from_slice(&o.q_thr);
        self.len.extend_from_slice(&o.len);
        self.order.extend_from_slice(&o.order);
        self.no_trib += o.no_trib;
        self.order1 += o.order1;
    }
    fn line(&self, name: &str) -> String {
        let (mut q, mut l, mut o) = (self.q_thr.clone(), self.len.clone(), self.order.clone());
        let pct = |k: usize| if self.n == 0 { "-".to_string() } else { format!("{:.0}%", 100.0 * k as f64 / self.n as f64) };
        format!(
            "{name:<28} n={:<6} medQ/thr={:<8} medLen={:<6} medOrder={:<4} noTrib={:<5} order1={}",
            self.n,
            f(median(&mut q), 1),
            f(median(&mut l), 1),
            f(median(&mut o), 1),
            pct(self.no_trib),
            pct(self.order1)
        )
    }
}

/// Gate-1 result for one world.
#[derive(Default, Clone, Debug)]
struct Gate1 {
    runs: usize,
    drawn: usize,
    mouths: usize,
    order_only: SetStats,
    disch_only_gate: SetStats,
    both: SetStats,
    either: SetStats,
    /// Admitted by order but not by discharge (order-3 stubs).
    o_not_d: SetStats,
    /// Admitted by discharge but not by order (large low-order mouths).
    d_not_o: SetStats,
    /// Of the world's 10 largest-discharge mouths, how many each rule admits.
    top10_both: usize,
    top10_order: usize,
    top10_disch: usize,
    top10_n: usize,
    /// The world's own 90th percentile of mouth discharge, as a share of its
    /// cells (the statistic `MIN_DISCHARGE_FRAC` was taken from on one world).
    p90_frac: f64,
}

/// Gate 1 on one world: the app's drawn network, its shore mouths, and the
/// four rules.
fn gate1(ws: &WorldState, wrap: bool, gw: usize, gh: usize, km: f64) -> Gate1 {
    let n = gw * gh;
    let order = ws.stream_order.as_ref().expect("river extraction ran");
    let ch = ws.channels.as_ref().expect("a generated world retains its channel tree");
    let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
    // `WorldGen::river_draws`' `down`: channel receiver, else steepest descent
    // over the routing surface the app's discharge was accumulated on.
    let route = cartalith_hydrology::routing_view(&ws.field, gw, gh, ws.sea_level, wrap, ws.integrated_drainage);
    let d8 = cartalith_hydrology::flow_receivers(&route, gw, gh, wrap);
    let down: Vec<i32> = (0..n).map(|i| if ch.recv[i] >= 0 { ch.recv[i] } else { d8[i] }).collect();
    let thresh = river_flow_thresh(gw, gh, gw, km);
    let rivers: Vec<River> =
        river_entities(order, &ch.recv, &ws.flow_discharge, &ws.field, gw, gh, 1, thresh, river_width_scale_k(km), wrap);
    let plan = river_draw_plan(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
    let cap = DELTA_MIN_DISCHARGE_FRAC as f64 * n as f64;
    let mut g = Gate1 { runs: rivers.len(), ..Default::default() };
    let mut mouths: Vec<Mouth> = Vec::new();
    for (i, r) in rivers.iter().enumerate() {
        if plan.parallel_of[i].is_some() {
            continue;
        }
        g.drawn += 1;
        if !shore(r.mouth as usize, gw, gh, &ws.field, ws.sea_level, &water) || !r.discharge.is_finite() {
            continue;
        }
        let q = r.discharge as f64;
        mouths.push(Mouth { order: r.order, q, q_thr: q / thresh, len_cells: r.length_cells, tributaries: r.tributaries });
    }
    g.mouths = mouths.len();
    for m in &mouths {
        let (o, d) = (m.order >= DELTA_MIN_ORDER, m.q >= cap);
        if o {
            g.order_only.add(m);
        }
        if d {
            g.disch_only_gate.add(m);
        }
        if o && d {
            g.both.add(m);
        }
        if o || d {
            g.either.add(m);
        }
        if o && !d {
            g.o_not_d.add(m);
        }
        if d && !o {
            g.d_not_o.add(m);
        }
    }
    let mut by_q = mouths.clone();
    by_q.sort_by(|a, b| b.q.total_cmp(&a.q));
    for m in by_q.iter().take(10) {
        g.top10_n += 1;
        let (o, d) = (m.order >= DELTA_MIN_ORDER, m.q >= cap);
        g.top10_both += (o && d) as usize;
        g.top10_order += o as usize;
        g.top10_disch += d as usize;
    }
    let mut fr: Vec<f64> = mouths.iter().map(|m| m.q / n as f64).collect();
    fr.sort_by(|a, b| a.total_cmp(b));
    g.p90_frac = if fr.is_empty() { f64::NAN } else { fr[((fr.len() - 1) as f64 * 0.9) as usize] };
    g
}

// ---------------------------------------------------------------------------
// Gate 2: River Transport navigability.
// ---------------------------------------------------------------------------

/// What one navigability rule gives on one world.
#[derive(Default, Clone, Copy, Debug)]
struct Nav {
    cells: usize,
    /// Navigable length in km: each navigable cell's step to its receiver
    /// (wrap-aware), or one cell width where it has none.
    km: f64,
    /// Connected pieces of the navigable network (cells joined along
    /// `recv` where both ends are navigable).
    comps: usize,
    /// The largest piece's share of the navigable length.
    largest_share: f64,
    /// Share of navigable length whose navigable chain reaches water (sea or
    /// lake) without a portage: down `recv` while navigable, until a water
    /// cell, or a tree end that touches water.
    to_water_share: f64,
    /// Pieces shorter than 5 cells: fragments a planner stage would flicker
    /// between river and portage on.
    short_frags: usize,
    /// Cells this rule admits that order >= 3 does not, and the reverse.
    added: usize,
    removed: usize,
}

/// Union-find root with path halving.
fn root(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}

/// Measures one navigability mask over one world's planner network.
fn nav_stats(nav: &[bool], base: &[bool], recv: &[i32], wb: &[u8], gw: usize, wrap: bool, km_cell: f64) -> Nav {
    let n = nav.len();
    let step_km = |i: usize| -> f64 {
        let r = recv[i];
        if r < 0 {
            return km_cell;
        }
        let r = r as usize;
        let mut dx = ((i % gw) as i64 - (r % gw) as i64).abs();
        if wrap && dx > gw as i64 / 2 {
            dx = gw as i64 - dx;
        }
        let dy = ((i / gw) as i64 - (r / gw) as i64).abs();
        ((dx * dx + dy * dy) as f64).sqrt() * km_cell
    };
    let mut parent: Vec<usize> = (0..n).collect();
    let mut s = Nav::default();
    for i in 0..n {
        if !nav[i] {
            if base[i] {
                s.removed += 1;
            }
            continue;
        }
        s.cells += 1;
        s.km += step_km(i);
        if !base[i] {
            s.added += 1;
        }
        let r = recv[i];
        if r >= 0 && nav[r as usize] {
            let (a, b) = (root(&mut parent, i), root(&mut parent, r as usize));
            if a != b {
                parent[a] = b;
            }
        }
    }
    // Component lengths.
    let mut comp_km: std::collections::HashMap<usize, (f64, usize)> = std::collections::HashMap::new();
    for i in 0..n {
        if nav[i] {
            let r = root(&mut parent, i);
            let e = comp_km.entry(r).or_insert((0.0, 0));
            e.0 += step_km(i);
            e.1 += 1;
        }
    }
    s.comps = comp_km.len();
    s.short_frags = comp_km.values().filter(|c| c.1 < 5).count();
    s.largest_share = if s.km > 0.0 { comp_km.values().map(|c| c.0).fold(0.0, f64::max) / s.km } else { f64::NAN };
    // Reaches water without a portage: memoised walk down `recv`.
    // 0 = unknown, 1 = yes, 2 = no.
    let mut memo = vec![0u8; n];
    let mut to_water_km = 0.0;
    for i in 0..n {
        if !nav[i] {
            continue;
        }
        let mut path = Vec::new();
        let mut c = i;
        let verdict = loop {
            if memo[c] != 0 {
                break memo[c];
            }
            path.push(c);
            let r = recv[c];
            if r < 0 {
                // The channel tree ends here: reached water when the cell
                // touches a water cell (the coast or a lake shore), else an
                // inland end (a portage would follow).
                let (x, y) = ((c % gw) as i64, (c / gw) as i64);
                let gh = (n / gw) as i64;
                let touches = (-1..=1).any(|dy| {
                    (-1..=1).any(|dx| {
                        let (mut nx, ny) = (x + dx, y + dy);
                        if wrap {
                            nx = nx.rem_euclid(gw as i64);
                        }
                        nx >= 0 && ny >= 0 && nx < gw as i64 && ny < gh && wb[ny as usize * gw + nx as usize] != 0
                    })
                });
                break if touches { 1 } else { 2 };
            }
            let r = r as usize;
            if wb[r] != 0 {
                break 1;
            }
            if !nav[r] || path.len() > n {
                break 2;
            }
            c = r;
        };
        for &p in &path {
            memo[p] = verdict;
        }
        if verdict == 1 {
            to_water_km += step_km(i);
        }
    }
    s.to_water_share = if s.km > 0.0 { to_water_km / s.km } else { f64::NAN };
    s
}

/// The named rules of gate 2, in print order.
fn rule_names() -> Vec<String> {
    let mut v: Vec<String> = vec!["order>=2".into(), "order>=3 (today)".into(), "order>=4".into()];
    for m in [0.5, 0.75, 1.0, 1.25, 1.5] {
        v.push(format!("Q>={}xT", Q_ANCHOR * m));
    }
    for m in [0.5, 0.75, 1.0, 1.25, 1.5] {
        v.push(format!("o3 OR Q>={}xT", Q_ANCHOR * m));
    }
    for m in [0.5, 1.0, 1.5] {
        v.push(format!("o3 AND Q>={}xT", Q_ANCHOR * m));
    }
    for k in [16.0, 32.0, 75.0] {
        v.push(format!("Q>={k}xT (context)"));
    }
    v
}

/// Gate-2 result for one world: per rule, its [`Nav`]; plus context.
#[derive(Default, Clone, Debug)]
struct Gate2 {
    rules: Vec<Nav>,
    /// Median `Q / thresh` over land cells of order exactly 3 (where the
    /// order gate opens), and over order 1-2 channel cells.
    med_q_o3: f64,
    med_q_o12: f64,
    /// Land channel cells of order 1-2 carrying >= 75x the threshold (the
    /// planner audit's "a 75x-threshold river reads order 1 to 2").
    big_low_order: usize,
    /// `fresh_river_channels` wall time, seconds: (median, min, max) of 5.
    fresh_s: (f64, f64, f64),
}

/// Gate 2 on one world.
fn gate2(ws: &WorldState, wrap: bool, gw: usize, gh: usize, km: f64, density: f64, time_fresh: bool) -> Gate2 {
    let n = gw * gh;
    let run = || {
        cartalith_civ::fresh_river_channels(
            &ws.field,
            &ws.flow_discharge,
            gw,
            gh,
            ws.sea_level,
            wrap,
            density,
            km,
            ws.integrated_drainage,
        )
    };
    let mut g = Gate2::default();
    let (order, recv) = if time_fresh {
        let mut ts = Vec::new();
        let mut last = None;
        for _ in 0..5 {
            let t = Instant::now();
            last = Some(run());
            ts.push(t.elapsed().as_secs_f64());
        }
        ts.sort_by(|a, b| a.total_cmp(b));
        g.fresh_s = (ts[2], ts[0], ts[4]);
        last.unwrap()
    } else {
        run()
    };
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
    let thresh = river_flow_thresh(gw, gh, gw, km);
    let q = &ws.flow_discharge;
    let land = |i: usize| wb[i] == 0;
    let qt = |i: usize| q[i] as f64 / thresh;
    let ord_ge = |k: i16| -> Vec<bool> { (0..n).map(|i| land(i) && order[i] >= k).collect() };
    // A discharge gate applies to channel cells only (order >= 1): a cell the
    // network does not carry is not a river whatever its D8 accumulation.
    let q_ge = |k: f64| -> Vec<bool> { (0..n).map(|i| land(i) && order[i] >= 1 && q[i].is_finite() && qt(i) >= k).collect() };
    let base = ord_ge(3);
    if std::env::var("RG_DIAG").is_ok() {
        // `RG_DIAG=1`: where does an order>=3 land cell's receiver go? Added
        // because the to-water share came out low (25-31 %); it shows the
        // navigable chain breaking where a receiver carries a LOWER order
        // (0-2) on this network, which no gate choice repairs. A finding, not
        // a cause: why the order drops is not investigated here.
        let mut h: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for i in 0..n {
            if !base[i] {
                continue;
            }
            let r = recv[i];
            let k = if r < 0 {
                "recv<0".to_string()
            } else if wb[r as usize] != 0 {
                format!("water{}", wb[r as usize])
            } else {
                format!("land order {}", order[r as usize].min(9))
            };
            *h.entry(k).or_insert(0) += 1;
        }
        eprintln!("DIAG {gw}x{gh}: {h:?}");
    }
    let mut masks: Vec<Vec<bool>> = vec![ord_ge(2), base.clone(), ord_ge(4)];
    for m in [0.5, 0.75, 1.0, 1.25, 1.5] {
        masks.push(q_ge(Q_ANCHOR * m));
    }
    for m in [0.5, 0.75, 1.0, 1.25, 1.5] {
        let qm = q_ge(Q_ANCHOR * m);
        masks.push((0..n).map(|i| base[i] || qm[i]).collect());
    }
    for m in [0.5, 1.0, 1.5] {
        let qm = q_ge(Q_ANCHOR * m);
        masks.push((0..n).map(|i| base[i] && qm[i]).collect());
    }
    for k in [16.0, 32.0, 75.0] {
        masks.push(q_ge(k));
    }
    let km_cell = km / gw as f64;
    g.rules = masks.iter().map(|m| nav_stats(m, &base, &recv, &wb, gw, wrap, km_cell)).collect();
    let mut o3: Vec<f64> = (0..n).filter(|&i| land(i) && order[i] == 3 && q[i].is_finite()).map(qt).collect();
    let mut o12: Vec<f64> = (0..n).filter(|&i| land(i) && (1..=2).contains(&order[i]) && q[i].is_finite()).map(qt).collect();
    g.med_q_o3 = median(&mut o3);
    g.med_q_o12 = median(&mut o12);
    g.big_low_order = (0..n).filter(|&i| land(i) && (1..=2).contains(&order[i]) && q[i].is_finite() && qt(i) >= 75.0).count();
    g
}

// ---------------------------------------------------------------------------
// The measurement.
// ---------------------------------------------------------------------------

/// Protects nothing by assertion beyond "the fixture reached the code" (each
/// world has drawn rivers, shore mouths and order >= 3 navigable cells; every
/// pooled size has at least one). Prints, per world and pooled per shape, the
/// gate-1 and gate-2 tables `RIVER_GATE_MEASUREMENTS.md` quotes, with timing.
#[test]
#[ignore = "Ruling CF item 6 measurement: 36 generated worlds; run alone in release (see the module doc)"]
fn river_gate_measurement() {
    let seeds = env_list::<i32>("RG_SEEDS", &SEEDS);
    let owner = std::env::var("RG_OWNER").is_ok_and(|v| v == "1");
    let names = rule_names();
    let mut shapes: Vec<(usize, usize, f64, f64, Vec<i32>)> =
        SHAPES.iter().map(|&(w, h, k)| (w, h, k, params::defaults().river_density, seeds.clone())).collect();
    if owner {
        shapes.push((2048, 1311, 40_075.0, 1.55, vec![246_371]));
    }
    let t_all = Instant::now();
    for (gw, gh, km, density, seeds) in shapes {
        println!("\n=== {gw}x{gh}, {km} km, river density {density}, {} seeds ===", seeds.len());
        let mut pool1 = Gate1::default();
        let mut pool2: Vec<Nav> = vec![Nav::default(); names.len()];
        let (mut mq3, mut mq12, mut big) = (Vec::new(), Vec::new(), 0usize);
        let mut gen_t = Vec::new();
        let mut fresh = Vec::new();
        for (si, &seed) in seeds.iter().enumerate() {
            let (ws, wrap, gen_s, d) = world(seed, gw, gh, km, density);
            assert_eq!(d, density);
            gen_t.push(gen_s);
            let t1 = Instant::now();
            let g1 = gate1(&ws, wrap, gw, gh, km);
            let g1_s = t1.elapsed().as_secs_f64();
            // `fresh_river_channels` is timed on the first seed of each shape
            // only (5 runs), so the matrix stays short.
            let g2 = gate2(&ws, wrap, gw, gh, km, density, si == 0);
            if si == 0 {
                fresh.push(g2.fresh_s);
            }
            assert!(g1.drawn > 0 && g1.mouths > 0, "seed {seed}: no drawn rivers or mouths -- the fixture measured nothing");
            assert!(g2.rules[1].cells > 0, "seed {seed}: no order>=3 navigable cell -- the fixture measured nothing");
            let today = g2.rules[1];
            println!(
                "seed {seed:>9} gen {gen_s:5.1}s gate1 {g1_s:4.2}s | mouths {:>4}: order {:>3} disch {:>3} both(today) {:>3} either {:>3} | O\\D {:>3} D\\O {:>3} | top10 both/order/disch {}/{}/{} | p90 {:.1e} | nav o3 {:>5} cells {:>7.0} km {:>4} pieces, to-water {:.0}%",
                g1.mouths,
                g1.order_only.n,
                g1.disch_only_gate.n,
                g1.both.n,
                g1.either.n,
                g1.o_not_d.n,
                g1.d_not_o.n,
                g1.top10_both,
                g1.top10_order,
                g1.top10_disch,
                g1.p90_frac,
                today.cells,
                today.km,
                today.comps,
                100.0 * today.to_water_share
            );
            pool1.runs += g1.runs;
            pool1.drawn += g1.drawn;
            pool1.mouths += g1.mouths;
            pool1.order_only.merge(&g1.order_only);
            pool1.disch_only_gate.merge(&g1.disch_only_gate);
            pool1.both.merge(&g1.both);
            pool1.either.merge(&g1.either);
            pool1.o_not_d.merge(&g1.o_not_d);
            pool1.d_not_o.merge(&g1.d_not_o);
            pool1.top10_both += g1.top10_both;
            pool1.top10_order += g1.top10_order;
            pool1.top10_disch += g1.top10_disch;
            pool1.top10_n += g1.top10_n;
            if g1.p90_frac.is_finite() {
                mq3.push(g1.p90_frac); // reused below as the p90 list
            }
            for (a, b) in pool2.iter_mut().zip(&g2.rules) {
                // Pooled: sums of counts and km; shares and pieces averaged
                // per world below (accumulated here as sums).
                a.cells += b.cells;
                a.km += b.km;
                a.comps += b.comps;
                a.short_frags += b.short_frags;
                a.added += b.added;
                a.removed += b.removed;
                a.largest_share += if b.largest_share.is_finite() { b.largest_share } else { 0.0 };
                a.to_water_share += if b.to_water_share.is_finite() { b.to_water_share } else { 0.0 };
            }
            if g2.med_q_o3.is_finite() {
                mq12.push((g2.med_q_o3, g2.med_q_o12));
            }
            big += g2.big_low_order;
        }
        let ns = seeds.len() as f64;
        let mut p90 = mq3.clone();
        println!("--- gate 1 (delta-fan mouths), pooled over {} worlds: {} runs, {} drawn, {} shore mouths", seeds.len(), pool1.runs, pool1.drawn, pool1.mouths);
        println!("    per-world p90 of mouth discharge as a share of cells: median {:.2e} (min {:.2e}, max {:.2e}); today's floor 2.0e-4",
            median(&mut p90), mq3.iter().cloned().fold(f64::MAX, f64::min), mq3.iter().cloned().fold(0.0, f64::max));
        for (name, s) in [
            ("order>=3 only", &pool1.order_only),
            ("discharge>=2e-4 only", &pool1.disch_only_gate),
            ("BOTH (today)", &pool1.both),
            ("EITHER", &pool1.either),
            ("order-not-discharge (O\\D)", &pool1.o_not_d),
            ("discharge-not-order (D\\O)", &pool1.d_not_o),
        ] {
            println!("    {}", s.line(name));
        }
        println!(
            "    top-10 mouths by discharge, summed over worlds ({}): today admits {}, order alone {}, discharge alone {}",
            pool1.top10_n, pool1.top10_both, pool1.top10_order, pool1.top10_disch
        );
        let mut a3: Vec<f64> = mq12.iter().map(|p| p.0).collect();
        let mut a12: Vec<f64> = mq12.iter().map(|p| p.1).collect();
        println!(
            "--- gate 2 (River Transport), summed over {} worlds; shares are per-world means. median over worlds of median Q/T: order-3 cells {}, order-1/2 channel cells {}; order-1/2 cells >= 75xT: {}",
            seeds.len(),
            f(median(&mut a3), 1),
            f(median(&mut a12), 1),
            big
        );
        println!("    {:<22} {:>8} {:>9} {:>7} {:>8} {:>7} {:>9} {:>8} {:>8}", "rule", "cells", "km", "pieces", "<5cell", "largest", "to-water", "+vs o3", "-vs o3");
        for (name, a) in names.iter().zip(&pool2) {
            println!(
                "    {:<22} {:>8} {:>9.0} {:>7} {:>8} {:>6.0}% {:>8.0}% {:>8} {:>8}",
                name,
                a.cells,
                a.km,
                a.comps,
                a.short_frags,
                100.0 * a.largest_share / ns,
                100.0 * a.to_water_share / ns,
                a.added,
                a.removed
            );
        }
        let mut gt = gen_t.clone();
        println!(
            "--- timing: generate_terrain median {:.2}s (min {:.2}, max {:.2}) over {} worlds; fresh_river_channels on seed {} (5 runs): median {:.3}s (min {:.3}, max {:.3})",
            median(&mut gt),
            gen_t.iter().cloned().fold(f64::MAX, f64::min),
            gen_t.iter().cloned().fold(0.0, f64::max),
            seeds.len(),
            seeds[0],
            fresh[0].0,
            fresh[0].1,
            fresh[0].2
        );
        assert!(pool1.both.n > 0 || pool1.order_only.n > 0, "{gw}x{gh}: no order>=3 mouth on any world");
    }
    println!("\ntotal wall {:.0}s", t_all.elapsed().as_secs_f64());
}
