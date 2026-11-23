//! Frozen town layouts (`TOWN_LAYOUT_SAVE_SCOPE.md`, Rulings GZ and HA, 2026-11-22): the pure half.
//!
//! A town layout is a pure function of a large, partly hidden input set (the world under the town, the place
//! editor, the `urban_*` switches, the rules table and the generator's own code), so a town a user has pinned is
//! **frozen**: its drawn geometry is kept as integer vector data and served in preference to generating it again.
//! This module holds everything about that which needs no Godot object:
//!
//! - [`URBAN_RULES_VERSION`] and its fixture-hash guard (TL-0, scope §4.1), the switch record
//!   ([`switch_record`]) and [`site_hash`], the digest of the world-derived context;
//! - the canonical codec (TL-1, scope §4.3): [`encode`] an [`UrbanLayout`] into the integer form stored in
//!   `entities/town_layouts.json`, [`decode`] it back, and [`layout_hash`], the fingerprint over the stored form;
//! - [`within_rounding`], the "same town within rounding" test that keeps a cross-architecture rounding flip from
//!   reading as a changed town (scope §4.2, R1);
//! - pin anchors (TL-4, scope §4.4): [`choose_anchor`], [`anchor_point`] and [`reattach`].
//!
//! The save slot, the freeze trigger and the serve path are `cartalith-godot`'s `town_layout_bridge.rs`.
//!
//! ## What it must never do
//!
//! Change a generated town: nothing here is on the generation path, and an unfrozen town is exactly the
//! computation it was. Never guess a string it does not know (an attribute is read from the row's own string
//! table, scope §4.3), never accept a row whose hash it did not recompute, and never delete a pin: a pin that
//! finds no feature is reported detached and stays where it is (Ruling HA, Q8).
//!
//! ## A decoded layout is a drawing, not a model
//!
//! The frozen form stores what `cartalith-godot`'s `layout_dict` hands the shell, and the feature attributes the
//! anchors match on (building kind, district, street class, gate side). It does **not** store what no reader
//! draws: building and parcel ids, a parcel's area, the harbour quay and defence, the citadel's approach, the
//! plaza kind's provenance strings, `FaithSite::yard`, the fort's bastions and moats beyond the trace and
//! ravelins, and PS-1's `parallel` plan (a probe diagnostic, `_viewerframe_probe.gd --parallel`). [`decode`]
//! leaves those empty. **Nothing but `layout_dict` and the anchor code may read a decoded layout**; a consumer that
//! needs another field must add it to the codec first (the exhaustive destructure in [`encode`] makes a new
//! `UrbanLayout` field a compile error here, scope R7).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use serde_json::{Map, Value, json};

use cartalith_urban::wallhug::KeptReason;
use cartalith_urban::{Citadel, FaithSite, HarbourWorks};

use crate::urban_adapter::{
    Bridge, Building, Detail, DetailGeom, Ford, Gate, HugOutcome, LayoutEdge, LayoutParcel, Market, Plaza, PlaceOverrides,
    UrbanContext, UrbanLayout, Vec2, WallState,
};

/// The urban rules version (scope §4.1): bumped **by hand** whenever a change moves a generated layout under any
/// switch setting. A frozen town records the version it was drawn under, so the City Viewer can say "drawn by
/// rules v1 (current v3)". Guarded by `the_rules_version_and_fixture_hashes_are_pinned` below: the fixture
/// towns' canonical hashes are committed literals beside this literal, so a behavioural change turns the test red
/// and the fix is to bump both. Starts at `1`, pinned 2026-11-22 after RR-T1 (`1f041a2e`) and fix E (`2aa402de`).
/// `2` (2026-11-23, Ruling HM): with `civ.bank_sites` on a river town is laid round the drawn river's edge, its
/// river is as wide as the map draws it and its roads keep to its bank; with it off every layout is v1's (the six
/// v1 fixture hashes did not move), and a seventh fixture pins the new path.
/// `3` (2026-11-23, Ruling HN): with `civ.bank_sites` on a town's lake and sea edges are the map's drawn water, a
/// town at a junction takes the wider drawn river, stands where its road reaches the bank, keeps its walled
/// streets, and lays no street, wall or house over the map's water or off its box; with the switch off every
/// layout is still v1's (the six v1 fixture hashes did not move), v2's seventh fixture moved, and an eighth pins
/// the map's shore.
/// `4` (2026-11-23, Ruling HQ): with `civ.bank_sites` on a both-banks town's far bank may grow to twice the market
/// bank's streets with more tries, a crossing wider than 400 m over the water is a ferry (a landing on each bank, no
/// bridge street), and a wall still over the water after its wet vertices go is routed round it along the shore;
/// with the switch off every layout is still v1's (the six v1 fixture hashes did not move), v2's fixture moved again,
/// v3's did not, and a ninth pins a ferry town.
/// `5` (2026-11-23, Ruling HU): with `civ.bank_sites` on the 400 m bridge limit counts the whole bridge, approaches
/// included; a ferried street's dry part stays up to its landing; a ferry whose road stops in the water lands on the
/// far bank (along the road's line, else square across); and the house sweep no longer tests the box's edge (it swept
/// no house). With the switch off every layout is still v1's (the six v1 fixture hashes did not move), and of the
/// three bank fixtures only the ferry town moved (its dry approaches now stand).
pub const URBAN_RULES_VERSION: u32 = 5;

/// The row encoding tag (scope §4.3): quantum 10 per metre, delta-coded vertex lists, version 1. A reader that does
/// not know a row's `enc` keeps the row verbatim and generates the town (scope §7).
pub const ENC: &str = "q10d1";

/// Integer steps per metre: a 0.1 m quantum (scope §4.3). The generator's own comparator rounds to a centimetre
/// (`hash_model`), a house is 4-15 m and the smallest feature drawn is a well or bollard; the worst-case error is
/// 0.05 m. Measured re-encode error is asserted in this module's tests.
pub const Q_PER_M: f64 = 10.0;

/// Integer steps per unit for dimensionless values (unit direction vectors, roof and lot tones, area ratios): 1e-4.
/// A labelled judgement: a bridge direction off by 1e-4 radians draws its span 0.002 m off at the 20 m span ends.
pub const Q_UNIT: f64 = 10_000.0;

/// Integer steps per unit for roof and lot tones: 1e-3, a labelled judgement (a tone scales a fill's lightness; 0.1 %
/// is below a visible step, and the fourth digit was incompressible noise in TL-1's census).
pub const Q_TONE: f64 = 1_000.0;

/// The version byte at the head of [`site_hash`]'s stream, so a later change to what the digest reads re-keys every
/// recorded hash deliberately rather than silently.
pub const SITE_HASH_VERSION: u8 = 1;

/// FNV-1a 64 over `bytes`: the hash `cartalith_urban::hash_model` and `SAVEFILE_COMPAT.md` §13.3.4 already use, so
/// the format has one hash function. It detects accident, not tampering.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    // FNV-1a's published 64-bit offset basis and prime (Fowler, Noll, Vo).
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// `h` as 16 lowercase hex digits, the form every hash in the slot is written in.
pub fn hex16(h: u64) -> String {
    format!("{h:016x}")
}

// ------------------------------------------------------------------------------------------- the recipe ---

/// The switch set a town was laid out with (scope §4.1): each `urban_*` name to the value **in effect** for that
/// layout. Built from an exhaustive destructure of [`PlaceOverrides`] (no `..`), so a switch a later build adds is a
/// compile error here rather than a silently unrecorded input. `urban_river_stretch` and `urban_river_smooth` are
/// recorded as in effect (`Some`), which is what the layout read: the app passes `None` for the smooth curve on a
/// world with no drawn network even when the parameter is on. The place-editor fields are recipe members of their
/// own, not switches. `river_kind_from_map` is [`crate::urban_adapter::UrbanWorld::river_kind_from_map`].
/// `civ_bank_origin` (Ruling HM) is per town: whether this town was laid round the drawn river's edge
/// (`PlaceOverrides::origin`), which is `civ.bank_sites` on and its cell inside the drawn river.
pub fn switch_record(o: &PlaceOverrides, river_kind_from_map: bool) -> BTreeMap<String, bool> {
    let PlaceOverrides {
        specialisation: _,
        fortified_trait: _,
        walls_override: _,
        age_override: _,
        resources: _,
        culture: _,
        variant: _,
        faith: _,
        wall_hug,
        riverbank,
        street_village,
        market_scale,
        harbour_front,
        river_stretch,
        burgage,
        market_road_bank,
        water_widen,
        town_bridge_search,
        parallel_streets,
        river_smooth,
        origin,
    } = *o;
    [
        ("urban_wall_hug", wall_hug),
        ("urban_riverbank", riverbank),
        ("urban_street_village", street_village),
        ("urban_market_scale", market_scale),
        ("urban_harbour_front", harbour_front),
        ("urban_river_stretch", river_stretch.is_some()),
        ("urban_burgage", burgage),
        ("urban_market_road_bank", market_road_bank),
        ("urban_growth_water_widen", water_widen),
        ("urban_town_bridge_search", town_bridge_search),
        ("urban_parallel_streets", parallel_streets),
        ("urban_river_smooth", river_smooth.is_some()),
        ("urban_river_kind_map", river_kind_from_map),
        // Ruling HM: this town was laid round the drawn river's edge (`civ.bank_sites` on and its cell in the stroke).
        ("civ_bank_origin", origin.is_some()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// The names of the switches that differ between a recorded set and today's. **A switch absent from `recorded`
/// reads as off** (scope §4.1, §7 MUST): a build that adds a switch does not make every older frozen town differ;
/// it differs only where the new switch is on. Sorted, so a report reads the same every time.
pub fn switch_diff(recorded: &BTreeMap<String, bool>, now: &BTreeMap<String, bool>) -> Vec<String> {
    let keys: std::collections::BTreeSet<&String> = recorded.keys().chain(now.keys()).collect();
    keys.into_iter()
        .filter(|k| recorded.get(*k).copied().unwrap_or(false) != now.get(*k).copied().unwrap_or(false))
        .cloned()
        .collect()
}

/// A canonical byte stream for [`site_hash`]: fixed-width little-endian integers, so the digest reads the same
/// on every architecture and never through `Debug` text or `std`'s unspecified hasher (scope §1.4).
struct Stream(Vec<u8>);

impl Stream {
    fn i(&mut self, v: i64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    /// A metre value at the 0.1 m quantum (scope §4.1: `round(v * 10)`).
    fn m(&mut self, v: f64) {
        self.i((v * Q_PER_M).round() as i64);
    }
    /// A height in field units at 1e-6, since `round(v * 10)` of a 0..1 field would read every terrain as flat.
    fn h(&mut self, v: f64) {
        self.i((v * 1e6).round() as i64);
    }
    fn tag(&mut self, t: u8) {
        self.0.push(t);
    }
    fn pts(&mut self, v: &[Vec2]) {
        self.i(v.len() as i64);
        for p in v {
            self.m(p.x);
            self.m(p.y);
        }
    }
}

/// `site_hash` (scope §4.1): FNV-1a 64 over the **world-derived** part of a layout context, so "has the ground or
/// the roads under this town moved since it was drawn" is answered without laying the town out, and separately from
/// "have the rules moved". Reads the site kind and orientation, the site box and stretch plan, the water context
/// (mask, river path, width and order, sea and lake share, the sea context), the terrain grid, the route ends and
/// the primary paths: every [`UrbanContext`] member that is derived from the world rather than from a setting.
/// Settings (population, age, walls, culture, faith, economy, switches) are recipe members of their own.
pub fn site_hash(ctx: &UrbanContext) -> u64 {
    let mut s = Stream(vec![SITE_HASH_VERSION]);
    s.0.extend_from_slice(ctx.site_kind.as_bytes());
    s.tag(0);
    s.i(ctx.orient.to_bits() as i64);
    match ctx.site_box {
        Some((w, h)) => {
            s.tag(1);
            s.m(w);
            s.m(h);
        }
        None => s.tag(0),
    }
    match &ctx.river_stretch {
        Some(p) => {
            s.tag(1);
            s.m(p.ahead_m);
            s.m(p.behind_m);
            s.m(p.across_m);
        }
        None => s.tag(0),
    }
    let water = |s: &mut Stream, w: &cartalith_urban::WaterCtx| {
        s.i(w.mw as i64);
        s.i(w.mh as i64);
        s.m(w.cell_m);
        s.i(w.mask.len() as i64);
        s.0.extend_from_slice(&w.mask);
        match &w.river_path {
            Some(p) => {
                s.tag(1);
                s.pts(p);
            }
            None => s.tag(0),
        }
        s.m(w.river_width_m.unwrap_or(-1.0));
        s.h(w.river_order);
        s.h(w.sea_lake_cells);
    };
    match &ctx.water {
        Some(w) => {
            s.tag(1);
            water(&mut s, &w.ctx);
            s.tag(u8::from(w.mostly_water));
            s.h(w.sea_frac);
            match &w.to_sea {
                Some(t) => {
                    s.tag(1);
                    water(&mut s, t);
                }
                None => s.tag(0),
            }
        }
        None => s.tag(0),
    }
    match &ctx.terrain {
        Some(t) => {
            s.tag(1);
            s.i(t.mw as i64);
            s.i(t.mh as i64);
            s.m(t.cell_m);
            for v in &t.grid {
                s.h(*v);
            }
        }
        None => s.tag(0),
    }
    match &ctx.route_ends {
        Some(r) => {
            s.tag(1);
            s.pts(r);
        }
        None => s.tag(0),
    }
    match &ctx.primary_paths {
        Some(pp) => {
            s.tag(1);
            s.i(pp.len() as i64);
            for p in pp {
                s.pts(p);
            }
        }
        None => s.tag(0),
    }
    fnv1a64(&s.0)
}

// -------------------------------------------------------------------------------------- the codec ---

/// A metre value as an integer at [`Q_PER_M`].
fn q(v: f64) -> i64 {
    (v * Q_PER_M).round() as i64
}

/// A dimensionless value as an integer at [`Q_UNIT`].
fn qu(v: f64) -> i64 {
    (v * Q_UNIT).round() as i64
}

/// An `f64` as its exact bit pattern in hex: `orient` (it rotates the whole town about the pin, so a rounded angle
/// would walk every vertex) and the scalar measures the City Viewer prints.
fn bits(v: f64) -> Value {
    Value::String(hex16(v.to_bits()))
}

/// The row's string table: every attribute string once, in first-use order, referred to by index (scope §4.3), so
/// a later build that renames a district cannot silently relabel a frozen town.
#[derive(Default)]
struct Strs {
    list: Vec<String>,
    at: HashMap<String, usize>,
}

impl Strs {
    fn of(&mut self, s: &str) -> Value {
        let n = self.list.len();
        let i = *self.at.entry(s.to_string()).or_insert_with(|| {
            self.list.push(s.to_string());
            n
        });
        Value::from(i)
    }
}

/// A vertex list delta-coded: `[x0, y0, dx1, dy1, ...]`, the first absolute (scope §4.3).
fn pts(v: &[Vec2]) -> Value {
    let mut out = Vec::with_capacity(v.len() * 2);
    let (mut px, mut py) = (0i64, 0i64);
    for p in v {
        let (x, y) = (q(p.x), q(p.y));
        out.push(Value::from(x - px));
        out.push(Value::from(y - py));
        (px, py) = (x, y);
    }
    Value::Array(out)
}

/// One point as `[x, y]`.
fn pt(p: Vec2) -> Value {
    json!([q(p.x), q(p.y)])
}

/// A list of vertex lists as `{"n": [count per list], "d": [one delta stream]}`: the lists are chained, each one's first
/// vertex a delta from the previous list's last, since neighbouring houses, lots and trees are stored next to each other
/// and a delta of tens of metres is a shorter integer than an absolute coordinate (measured in TL-1's census: the
/// per-list absolute start made a hamlet's row 6.3 KB against the 5 KB bar).
///
/// Within a list, from the fourth vertex on, each vertex is stored as its residual from the **parallelogram prediction**
/// `p[k-1] + p[k-3] - p[k-2]` (the fourth corner of the parallelogram through the three before it): a house or a lot is
/// near-rectangular, so the residual is a step or two where a delta is tens of metres. The first three vertices are
/// plain deltas. Lossless on the integers: [`dpolys`] runs the same prediction forward.
fn polys(v: &[Vec<Vec2>]) -> Value {
    let mut d = Vec::new();
    let mut last = (0i64, 0i64);
    for p in v {
        let ip: Vec<(i64, i64)> = p.iter().map(|q2| (q(q2.x), q(q2.y))).collect();
        for (k, &(x, y)) in ip.iter().enumerate() {
            let pred = if k >= 3 {
                (ip[k - 1].0 + ip[k - 3].0 - ip[k - 2].0, ip[k - 1].1 + ip[k - 3].1 - ip[k - 2].1)
            } else {
                last
            };
            d.push(Value::from(x - pred.0));
            d.push(Value::from(y - pred.1));
            last = (x, y);
        }
    }
    json!({ "n": v.iter().map(Vec::len).collect::<Vec<_>>(), "d": d })
}

fn flags(v: impl Iterator<Item = bool>) -> Value {
    Value::Array(v.map(|b| Value::from(u8::from(b))).collect())
}

/// [`HugOutcome`] as `{"refit": [clearance bits, area ratio bits, streets removed]}` or `{"kept": "<reason>"}`.
fn hug(h: &HugOutcome) -> Value {
    match h {
        HugOutcome::Refit { clearance_m, area_ratio, streets_removed } => {
            json!({ "refit": [bits(*clearance_m), bits(*area_ratio), *streets_removed] })
        }
        HugOutcome::Kept(r) => json!({ "kept": format!("{r:?}") }),
    }
}

fn kept_reason(s: &str) -> Option<KeptReason> {
    Some(match s {
        "NoWall" => KeptReason::NoWall,
        "Bastioned" => KeptReason::Bastioned,
        "TooFewLots" => KeptReason::TooFewLots,
        "Refused" => KeptReason::Refused,
        "MarginNotMet" => KeptReason::MarginNotMet,
        "NotSmaller" => KeptReason::NotSmaller,
        "Jagged" => KeptReason::Jagged,
        _ => return None,
    })
}

/// Details (farmland and hinterland props) as columns: kind index, geometry type (0 point, 1 segment, 2 polygon),
/// the vertices, `rr` as sparse `[index, value]` pairs and the orchard indices.
fn details(v: &[Detail], st: &mut Strs) -> Value {
    let mut kind = Vec::new();
    let mut g = Vec::new();
    let mut geo = Vec::new();
    let mut rr = Vec::new();
    let mut orchard = Vec::new();
    for (i, d) in v.iter().enumerate() {
        let Detail { id: _, kind: k, geom, rr: r, orchard: o, prov: _ } = d;
        kind.push(st.of(k));
        let (t, p) = match geom {
            DetailGeom::Point(a) => (0, vec![*a]),
            DetailGeom::Seg(a, b) => (1, vec![*a, *b]),
            DetailGeom::Poly(p) => (2, p.clone()),
        };
        g.push(Value::from(t));
        geo.push(p);
        if let Some(r) = r {
            rr.push(json!([i, q(*r)]));
        }
        if *o {
            orchard.push(Value::from(i));
        }
    }
    json!({ "kind": kind, "g": g, "pts": polys(&geo), "rr": rr, "orchard": orchard })
}

/// The two ridges a quad footprint can carry (`cartalith_urban::districts::ridge_of`: the spine along the longer axis,
/// or front to back for a gable, inset 16 % from each end), from its quantised corners. `None` for a footprint that is
/// not a quad. Computed in `f64` from integers with exact-IEEE operations only, so every reader gets the same answer.
fn ridge_preds(ip: &[(i64, i64)]) -> Option<[[(i64, i64); 2]; 2]> {
    if ip.len() != 4 {
        return None;
    }
    let f = |k: usize| (ip[k].0 as f64, ip[k].1 as f64);
    let mid = |a: (f64, f64), b: (f64, f64)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let ins = |r0: (f64, f64), r1: (f64, f64)| {
        let l = |t: f64| ((r0.0 + (r1.0 - r0.0) * t).round() as i64, (r0.1 + (r1.1 - r0.1) * t).round() as i64);
        [l(0.16), l(0.84)]
    };
    Some([ins(mid(f(0), f(3)), mid(f(1), f(2))), ins(mid(f(0), f(1)), mid(f(3), f(2)))])
}

/// A building's ridge as `(selector, four residuals)`: selector 0 or 1 names the [`ridge_preds`] candidate it is stored
/// against (the residuals are then a step or so, where the points themselves are 3-4 digit deltas), 2 stores it as
/// offsets from the footprint's first vertex (a footprint that is not a quad, or a ridge no candidate predicts).
/// Lossless on the integers.
fn ridge_code(ip: &[(i64, i64)], r: [(i64, i64); 2]) -> (i64, [i64; 4]) {
    let res = |p: [(i64, i64); 2]| [r[0].0 - p[0].0, r[0].1 - p[0].1, r[1].0 - p[1].0, r[1].1 - p[1].1];
    if let Some(c) = ridge_preds(ip) {
        let (a, b) = (res(c[0]), res(c[1]));
        let worst = |v: &[i64; 4]| v.iter().map(|x| x.abs()).max().unwrap_or(0);
        let (sel, best) = if worst(&a) <= worst(&b) { (0, a) } else { (1, b) };
        if worst(&best) <= 50 {
            return (sel, best);
        }
    }
    let o = ip.first().copied().unwrap_or((0, 0));
    (2, res([o, o]))
}

/// [`ridge_code`] inverted; `None` for an unknown selector or a candidate selector on a footprint that is not a quad.
fn ridge_decode(ip: &[(i64, i64)], sel: i64, r: [i64; 4]) -> Option<[(i64, i64); 2]> {
    let base = match sel {
        0 | 1 => ridge_preds(ip)?[sel as usize],
        2 => {
            let o = *ip.first()?;
            [o, o]
        }
        _ => return None,
    };
    Some([(base[0].0 + r[0], base[0].1 + r[1]), (base[1].0 + r[2], base[1].1 + r[3])])
}

/// The frozen form of `l` (scope §4.3): a JSON object of integer arrays (and a few exact-bit hex strings and
/// attribute strings), in local box metres at a 0.1 m quantum. Deterministic: the same layout gives the same value,
/// and [`canonical`] text of it is what [`layout_hash`] reads. Destructures [`UrbanLayout`] exhaustively, so a field
/// a later build adds must be decided here (scope R7).
#[allow(clippy::too_many_lines)]
pub fn encode(l: &UrbanLayout) -> Value {
    let UrbanLayout {
        wm,
        hm,
        market,
        pin,
        market_prov,
        site_kind,
        orient,
        water_poly,
        river,
        river_w,
        bridge_pt,
        bridges,
        ford,
        route_ends,
        harbour_pt,
        harbour,
        edges,
        blocks,
        block_plaza,
        plaza,
        parcels,
        buildings,
        building_tone,
        building_intramural,
        wall,
        citadel,
        wall_hug,
        wall_spec,
        markets,
        farmland,
        details: props,
        street_len,
        pop,
        target_len,
        max_rf,
        // PS-1's plan is a probe diagnostic (`_viewerframe_probe.gd --parallel`), not drawn: not frozen (module doc).
        parallel: _,
        pop_target,
        settlement_age,
        uses_real_water,
        uses_real_terrain,
        water_mask_runs,
        faith_sites,
    } = l;
    let mut st = Strs::default();
    let mut g = Map::new();
    g.insert("box".into(), json!([q(*wm), q(*hm)]));
    g.insert("market".into(), pt(*market));
    g.insert("pin".into(), pt(*pin));
    g.insert("market_prov".into(), Value::from(*market_prov));
    g.insert("site_kind".into(), Value::from(site_kind.as_str()));
    g.insert("orient".into(), bits(*orient));
    g.insert("wall_spec".into(), Value::from(*wall_spec));
    g.insert(
        "scalars".into(),
        json!([bits(*street_len), bits(*pop), bits(*target_len), bits(*max_rf), bits(*pop_target), bits(*settlement_age), bits(*river_w)]),
    );
    g.insert("real".into(), json!([u8::from(*uses_real_water), u8::from(*uses_real_terrain)]));
    g.insert("water".into(), pts(water_poly));
    g.insert("river".into(), pts(river));
    g.insert("route_ends".into(), pts(route_ends));
    g.insert("mask_runs".into(), pts(&water_mask_runs.iter().flat_map(|(a, b)| [*a, *b]).collect::<Vec<_>>()));
    if let Some(b) = bridge_pt {
        g.insert("bridge_pt".into(), pt(*b));
    }
    if let Some(bs) = bridges {
        let v: Vec<Value> = bs
            .iter()
            .map(|Bridge { pt: p, dir: d, cls }| json!([q(p.x), q(p.y), qu(d.x), qu(d.y), st.of(cls)]))
            .collect();
        g.insert("bridges".into(), Value::Array(v));
    }
    if let Some(Ford { pt: p, dir: d }) = ford {
        let mut v = vec![Value::from(q(p.x)), Value::from(q(p.y))];
        if let Some(d) = d {
            v.push(Value::from(qu(d.x)));
            v.push(Value::from(qu(d.y)));
        }
        g.insert("ford".into(), Value::Array(v));
    }
    if let Some(h) = harbour_pt {
        g.insert("harbour_pt".into(), pt(*h));
    }
    if let Some(h) = harbour {
        let mut hv = Map::new();
        hv.insert("piers".into(), pts(&h.piers.iter().flat_map(|p| [p.a, p.b]).collect::<Vec<_>>()));
        if let Some(m) = &h.mole {
            hv.insert("mole".into(), pts(m));
        }
        hv.insert("pt".into(), pt(h.pt));
        g.insert("harbour".into(), Value::Object(hv));
    }

    // Streets: a node table plus index pairs, so a street graph's shared nodes are stored once and a segment's id is
    // its index (scope §4.3).
    let mut node_at: HashMap<(i64, i64), usize> = HashMap::new();
    let mut nodes: Vec<Vec2> = Vec::new();
    let mut node = |p: Vec2| -> usize {
        let k = (q(p.x), q(p.y));
        *node_at.entry(k).or_insert_with(|| {
            nodes.push(p);
            nodes.len() - 1
        })
    };
    let mut ev = Vec::with_capacity(edges.len() * 4);
    for LayoutEdge { a, b, cls, w } in edges {
        ev.push(Value::from(node(*a)));
        ev.push(Value::from(node(*b)));
        ev.push(st.of(cls));
        ev.push(Value::from(q(*w)));
    }
    g.insert("nodes".into(), pts(&nodes));
    g.insert("edges".into(), Value::Array(ev));
    g.insert("blocks".into(), polys(blocks));
    g.insert("block_plaza".into(), flags(block_plaza.iter().copied()));
    if let Some(Plaza { center, poly, kind }) = plaza {
        let k = u8::from(matches!(kind, cartalith_urban::plaza::PlazaKind::Green));
        g.insert("plaza".into(), json!({ "c": pt(*center), "poly": pts(poly), "kind": k }));
    }
    let mut pp: Vec<Vec<Vec2>> = Vec::new();
    let mut pc = Vec::new();
    let mut ptone = Vec::new();
    let mut pd = Vec::new();
    for LayoutParcel { poly, area: _, edge_cls, tone, district } in parcels {
        pp.push(poly.clone());
        pc.push(st.of(edge_cls));
        ptone.push(Value::from((*tone * Q_TONE).round() as i64));
        pd.push(st.of(district));
    }
    g.insert("parcels".into(), json!({ "poly": polys(&pp), "cls": pc, "tone": ptone, "district": pd }));
    let mut bp: Vec<Vec<Vec2>> = Vec::new();
    let mut rsel: Vec<Value> = Vec::new();
    let mut rres: Vec<Value> = Vec::new();
    let mut bk = Vec::new();
    let mut bd = Vec::new();
    let mut bc = Vec::new();
    for Building { id: _, poly, ridge, parcel: _, kind, district, age: _, courtyard, prov: _ } in buildings {
        bp.push(poly.clone());
        let ip: Vec<(i64, i64)> = poly.iter().map(|p| (q(p.x), q(p.y))).collect();
        let ir = [(q(ridge[0].x), q(ridge[0].y)), (q(ridge[1].x), q(ridge[1].y))];
        let (sel, res) = ridge_code(&ip, ir);
        rsel.push(Value::from(sel));
        rres.extend(res.into_iter().map(Value::from));
        bk.push(st.of(kind));
        bd.push(st.of(district));
        bc.push(Value::from(u8::from(*courtyard)));
    }
    let mut bv = Map::new();
    bv.insert("poly".into(), polys(&bp));
    bv.insert("ridge".into(), json!({ "s": rsel, "r": rres }));
    bv.insert("kind".into(), Value::Array(bk));
    bv.insert("district".into(), Value::Array(bd));
    bv.insert("court".into(), Value::Array(bc));
    bv.insert("tone".into(), Value::Array(building_tone.iter().map(|t| Value::from((*t * Q_TONE).round() as i64)).collect()));
    if let Some(i) = building_intramural {
        bv.insert("intra".into(), flags(i.iter().copied()));
    }
    g.insert("buildings".into(), Value::Object(bv));

    if let Some(ring) = &wall.ring {
        let mut w = Map::new();
        w.insert("ring".into(), pts(ring));
        w.insert("style".into(), Value::from(wall.style.as_str()));
        w.insert(
            "gates".into(),
            Value::Array(wall.gates.iter().map(|Gate { pt: p, water, prov: _ }| json!([q(p.x), q(p.y), u8::from(*water)])).collect()),
        );
        w.insert("spurs".into(), Value::Array(wall.spurs.iter().map(|s| json!([q(s.a.x), q(s.a.y), q(s.b.x), q(s.b.y)])).collect()));
        if let Some(c) = wall.centroid {
            w.insert("centroid".into(), pt(c));
        }
        if let Some(f) = &wall.fort {
            w.insert("fort".into(), json!({ "trace": pts(&f.trace), "ravelins": polys(&f.ravelins) }));
        }
        g.insert("wall".into(), Value::Object(w));
    }
    if let Some(Citadel { wall: cw, towers, tower_r, keep, court, gate, approach: _, on_curtain: _, prov: _ }) = citadel {
        g.insert(
            "citadel".into(),
            json!({ "wall": pts(cw), "towers": pts(towers), "r": bits(*tower_r), "keep": pts(keep), "court": pts(court), "gate": pt(*gate) }),
        );
    }
    if let Some(h) = wall_hug {
        g.insert("hug".into(), hug(h));
    }
    let mut mp: Vec<Vec<Vec2>> = Vec::new();
    let mut mc = Vec::new();
    let mut mn = Vec::new();
    for Market { name, center, poly, prov: _ } in markets {
        mp.push(poly.clone());
        mc.push(*center);
        mn.push(st.of(name));
    }
    g.insert("markets".into(), json!({ "poly": polys(&mp), "c": pts(&mc), "name": mn }));
    g.insert("farm".into(), details(farmland, &mut st));
    g.insert("details".into(), details(props, &mut st));
    let faith: Vec<Value> = faith_sites
        .iter()
        .map(|f| {
            let FaithSite { id, faith, name, center, yard: _, polys: fp, open, columns, steps, tower, ridge, prov } = f;
            let mut m = Map::new();
            m.insert("id".into(), Value::from(id.as_str()));
            m.insert("faith".into(), Value::from(faith.as_str()));
            if let Some(n) = name {
                m.insert("name".into(), Value::from(*n));
            }
            m.insert("c".into(), pt(*center));
            m.insert("polys".into(), polys(fp));
            m.insert("open".into(), polys(open));
            m.insert("cols".into(), pts(columns));
            m.insert("steps".into(), pts(&steps.iter().flat_map(|s| [s[0], s[1]]).collect::<Vec<_>>()));
            m.insert("ridge".into(), pts(ridge));
            m.insert("prov".into(), Value::from(prov.as_str()));
            if let Some(t) = tower {
                m.insert("tower".into(), json!([q(t.x), q(t.y), bits(t.r), st.of(t.kind)]));
            }
            Value::Object(m)
        })
        .collect();
    g.insert("faith".into(), Value::Array(faith));
    g.insert("strs".into(), Value::Array(st.list.into_iter().map(Value::from).collect()));
    Value::Object(g)
}

/// The canonical text of a stored value: compact JSON with sorted keys (`serde_json`'s map is ordered by key in this
/// workspace: no crate enables `preserve_order`, checked 2026-11-22 against `Cargo.lock`). What the slot writes and
/// what [`layout_hash`] reads.
pub fn canonical(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// The fingerprint (scope §4.2): FNV-1a 64 over the canonical text of the stored geometry, so a reader that never runs
/// the generator can still check a row.
pub fn layout_hash(geom: &Value) -> String {
    hex16(fnv1a64(canonical(geom).as_bytes()))
}

/// A bounded interner for the `&'static str` fields of a decoded layout: a known string returns the one already
/// leaked, an unknown one (a newer build's district, read from the row's own table) is leaked once.
/// `ponytail:` the set grows by every distinct string ever read, which is bounded by the save's own size; a
/// per-world arena would free them on close if that ever matters.
fn intern(s: &str) -> &'static str {
    static SET: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut g = SET.get_or_init(|| Mutex::new(HashSet::new())).lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(x) = g.get(s) {
        return x;
    }
    let l: &'static str = Box::leak(s.to_string().into_boxed_str());
    g.insert(l);
    l
}

/// A decode failure: what was wrong, for the row's warning.
pub type DecodeError = String;

/// Reads the frozen form back into a value [`decode`]'s caller can index, with every lookup failing as an error
/// naming the member rather than panicking.
struct Rd<'a> {
    g: &'a Map<String, Value>,
    strs: Vec<&'static str>,
}

fn int(v: &Value, what: &str) -> Result<i64, DecodeError> {
    v.as_i64().ok_or_else(|| format!("{what}: not an integer"))
}

fn arr<'a>(v: &'a Value, what: &str) -> Result<&'a Vec<Value>, DecodeError> {
    v.as_array().ok_or_else(|| format!("{what}: not an array"))
}

fn m(v: i64) -> f64 {
    v as f64 / Q_PER_M
}

fn u(v: i64) -> f64 {
    v as f64 / Q_UNIT
}

fn dpts(v: &Value, what: &str) -> Result<Vec<Vec2>, DecodeError> {
    let a = arr(v, what)?;
    if a.len() % 2 != 0 {
        return Err(format!("{what}: odd coordinate count"));
    }
    let (mut x, mut y) = (0i64, 0i64);
    let mut out = Vec::with_capacity(a.len() / 2);
    for c in a.chunks(2) {
        x += int(&c[0], what)?;
        y += int(&c[1], what)?;
        out.push(Vec2 { x: m(x), y: m(y) });
    }
    Ok(out)
}

fn dpt(v: &Value, what: &str) -> Result<Vec2, DecodeError> {
    let a = arr(v, what)?;
    if a.len() < 2 {
        return Err(format!("{what}: not a point"));
    }
    Ok(Vec2 { x: m(int(&a[0], what)?), y: m(int(&a[1], what)?) })
}

fn dbits(v: &Value, what: &str) -> Result<f64, DecodeError> {
    let s = v.as_str().ok_or_else(|| format!("{what}: not a hex string"))?;
    u64::from_str_radix(s, 16).map(f64::from_bits).map_err(|_| format!("{what}: bad hex"))
}

fn dpolys(v: &Value, what: &str) -> Result<Vec<Vec<Vec2>>, DecodeError> {
    let n = arr(v.get("n").ok_or_else(|| format!("{what}: no counts"))?, what)?;
    let d = arr(v.get("d").ok_or_else(|| format!("{what}: no vertices"))?, what)?;
    let mut out = Vec::with_capacity(n.len());
    let mut at = 0usize;
    let mut last = (0i64, 0i64);
    for c in n {
        let c = usize::try_from(int(c, what)?).map_err(|_| format!("{what}: negative count"))?;
        if at.checked_add(c.checked_mul(2).ok_or_else(|| format!("{what}: count overflow"))?).is_none_or(|e| e > d.len()) {
            return Err(format!("{what}: counts exceed the vertices"));
        }
        let mut ip: Vec<(i64, i64)> = Vec::with_capacity(c);
        for k in 0..c {
            let pred = if k >= 3 { (ip[k - 1].0 + ip[k - 3].0 - ip[k - 2].0, ip[k - 1].1 + ip[k - 3].1 - ip[k - 2].1) } else { last };
            let p = (pred.0 + int(&d[at], what)?, pred.1 + int(&d[at + 1], what)?);
            at += 2;
            ip.push(p);
            last = p;
        }
        out.push(ip.into_iter().map(|(x, y)| Vec2 { x: m(x), y: m(y) }).collect());
    }
    if at != d.len() {
        return Err(format!("{what}: vertices left over"));
    }
    Ok(out)
}

fn dflags(v: &Value, what: &str) -> Result<Vec<bool>, DecodeError> {
    arr(v, what)?.iter().map(|b| int(b, what).map(|i| i != 0)).collect()
}

impl Rd<'_> {
    fn get(&self, k: &str) -> Result<&Value, DecodeError> {
        self.g.get(k).ok_or_else(|| format!("missing member {k}"))
    }
    fn s(&self, v: &Value, what: &str) -> Result<&'static str, DecodeError> {
        let i = usize::try_from(int(v, what)?).map_err(|_| format!("{what}: negative string index"))?;
        self.strs.get(i).copied().ok_or_else(|| format!("{what}: string index {i} out of range"))
    }
    fn str_member(&self, k: &str) -> Result<&'static str, DecodeError> {
        self.get(k)?.as_str().map(intern).ok_or_else(|| format!("{k}: not a string"))
    }
    fn details(&self, k: &str) -> Result<Vec<Detail>, DecodeError> {
        let d = self.get(k)?;
        let kind = arr(&d["kind"], k)?;
        let g = arr(&d["g"], k)?;
        let geo = dpolys(&d["pts"], k)?;
        if kind.len() != g.len() || kind.len() != geo.len() {
            return Err(format!("{k}: column lengths differ"));
        }
        let mut out = Vec::with_capacity(kind.len());
        for i in 0..kind.len() {
            let p = geo[i].clone();
            let geom = match (int(&g[i], k)?, p.len()) {
                (0, 1) => DetailGeom::Point(p[0]),
                (1, 2) => DetailGeom::Seg(p[0], p[1]),
                (2, _) => DetailGeom::Poly(p),
                _ => return Err(format!("{k}: bad geometry at {i}")),
            };
            out.push(Detail { id: String::new(), kind: self.s(&kind[i], k)?, geom, rr: None, orchard: false, prov: "frozen" });
        }
        for e in arr(&d["rr"], k)? {
            let e = arr(e, k)?;
            let i = usize::try_from(int(&e[0], k)?).map_err(|_| format!("{k}: bad rr index"))?;
            let r = m(int(e.get(1).ok_or_else(|| format!("{k}: short rr"))?, k)?);
            out.get_mut(i).ok_or_else(|| format!("{k}: rr index out of range"))?.rr = Some(r);
        }
        for e in arr(&d["orchard"], k)? {
            let i = usize::try_from(int(e, k)?).map_err(|_| format!("{k}: bad orchard index"))?;
            out.get_mut(i).ok_or_else(|| format!("{k}: orchard index out of range"))?.orchard = true;
        }
        Ok(out)
    }
}

/// The frozen form back into an [`UrbanLayout`] (scope §4.3). Every vertex comes back within half a quantum
/// (0.05 m) of the layout it was encoded from; fields the form does not store are empty (module doc). An error names
/// the first damaged member, and the caller keeps the row verbatim and generates the town (scope §7).
#[allow(clippy::too_many_lines)]
pub fn decode(geom: &Value) -> Result<UrbanLayout, DecodeError> {
    let g = geom.as_object().ok_or("the geometry is not an object")?;
    let strs = arr(g.get("strs").ok_or("missing member strs")?, "strs")?
        .iter()
        .map(|s| s.as_str().map(intern).ok_or_else(|| "strs: not a string".to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let r = Rd { g, strs };
    let bx = arr(r.get("box")?, "box")?;
    if bx.len() != 2 {
        return Err("box: not two values".into());
    }
    let sc = arr(r.get("scalars")?, "scalars")?;
    if sc.len() != 7 {
        return Err("scalars: not seven values".into());
    }
    let scal = |i: usize| dbits(&sc[i], "scalars");
    let real = dflags(r.get("real")?, "real")?;
    if real.len() != 2 {
        return Err("real: not two flags".into());
    }
    let runs = dpts(r.get("mask_runs")?, "mask_runs")?;
    if runs.len() % 2 != 0 {
        return Err("mask_runs: odd corner count".into());
    }
    let bridges = match g.get("bridges") {
        Some(v) => Some(
            arr(v, "bridges")?
                .iter()
                .map(|b| {
                    let b = arr(b, "bridges")?;
                    if b.len() != 5 {
                        return Err("bridges: not five values".to_string());
                    }
                    Ok(Bridge {
                        pt: Vec2 { x: m(int(&b[0], "bridges")?), y: m(int(&b[1], "bridges")?) },
                        dir: Vec2 { x: u(int(&b[2], "bridges")?), y: u(int(&b[3], "bridges")?) },
                        cls: r.s(&b[4], "bridges")?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };
    let ford = match g.get("ford") {
        Some(v) => {
            let f = arr(v, "ford")?;
            let p = Vec2 { x: m(int(f.first().ok_or("ford: empty")?, "ford")?), y: m(int(f.get(1).ok_or("ford: short")?, "ford")?) };
            let d = match (f.get(2), f.get(3)) {
                (Some(a), Some(b)) => Some(Vec2 { x: u(int(a, "ford")?), y: u(int(b, "ford")?) }),
                _ => None,
            };
            Some(Ford { pt: p, dir: d })
        }
        None => None,
    };
    let harbour = match g.get("harbour") {
        Some(h) => {
            let piers = dpts(&h["piers"], "harbour.piers")?;
            if piers.len() % 2 != 0 {
                return Err("harbour.piers: odd point count".into());
            }
            let mole = match h.get("mole") {
                Some(mv) => {
                    let mp = dpts(mv, "harbour.mole")?;
                    Some(<[Vec2; 4]>::try_from(mp.as_slice()).map_err(|_| "harbour.mole: not four points")?)
                }
                None => None,
            };
            Some(HarbourWorks {
                quay: Vec::new(),
                piers: piers.chunks(2).map(|c| cartalith_urban::water::Pier { a: c[0], b: c[1] }).collect(),
                mole,
                pt: dpt(&h["pt"], "harbour.pt")?,
                defence: None,
                prov: String::new(),
            })
        }
        None => None,
    };
    let nodes = dpts(r.get("nodes")?, "nodes")?;
    let ev = arr(r.get("edges")?, "edges")?;
    if ev.len() % 4 != 0 {
        return Err("edges: not four values per edge".into());
    }
    let node = |v: &Value| -> Result<Vec2, DecodeError> {
        let i = usize::try_from(int(v, "edges")?).map_err(|_| "edges: negative node".to_string())?;
        nodes.get(i).copied().ok_or_else(|| format!("edges: node {i} out of range"))
    };
    let edges = ev
        .chunks(4)
        .map(|c| Ok(LayoutEdge { a: node(&c[0])?, b: node(&c[1])?, cls: r.s(&c[2], "edges")?, w: m(int(&c[3], "edges")?) }))
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let blocks = dpolys(r.get("blocks")?, "blocks")?;
    let block_plaza = dflags(r.get("block_plaza")?, "block_plaza")?;
    let plaza = match g.get("plaza") {
        Some(p) => Some(Plaza {
            center: dpt(&p["c"], "plaza.c")?,
            poly: dpts(&p["poly"], "plaza.poly")?,
            kind: if int(&p["kind"], "plaza.kind")? == 1 { cartalith_urban::plaza::PlazaKind::Green } else { cartalith_urban::plaza::PlazaKind::Market },
        }),
        None => None,
    };
    let pv = r.get("parcels")?;
    let pp = dpolys(&pv["poly"], "parcels.poly")?;
    let pc = arr(&pv["cls"], "parcels.cls")?;
    let ptone = arr(&pv["tone"], "parcels.tone")?;
    let pd = arr(&pv["district"], "parcels.district")?;
    if pc.len() != pp.len() || ptone.len() != pp.len() || pd.len() != pp.len() {
        return Err("parcels: column lengths differ".into());
    }
    let parcels = pp
        .into_iter()
        .enumerate()
        .map(|(i, poly)| {
            Ok(LayoutParcel {
                area: cartalith_urban::geom::poly_area(&poly).abs(),
                poly,
                edge_cls: r.s(&pc[i], "parcels.cls")?,
                tone: int(&ptone[i], "parcels.tone")? as f64 / Q_TONE,
                district: r.s(&pd[i], "parcels.district")?,
            })
        })
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let bv = r.get("buildings")?;
    let bp = dpolys(&bv["poly"], "buildings.poly")?;
    let rsel = arr(&bv["ridge"]["s"], "buildings.ridge.s")?;
    let rres = arr(&bv["ridge"]["r"], "buildings.ridge.r")?;
    let bk = arr(&bv["kind"], "buildings.kind")?;
    let bd = arr(&bv["district"], "buildings.district")?;
    let bc = dflags(&bv["court"], "buildings.court")?;
    let btone = arr(&bv["tone"], "buildings.tone")?;
    let n = bp.len();
    if rsel.len() != n || rres.len() != 4 * n || bk.len() != n || bd.len() != n || bc.len() != n || btone.len() != n {
        return Err("buildings: column lengths differ".into());
    }
    let building_intramural = match bv.get("intra") {
        Some(v) => {
            let f = dflags(v, "buildings.intra")?;
            if f.len() != n {
                return Err("buildings.intra: length differs".into());
            }
            Some(f)
        }
        None => None,
    };
    let buildings = bp
        .into_iter()
        .enumerate()
        .map(|(i, poly)| {
            let ip: Vec<(i64, i64)> = poly.iter().map(|p| (q(p.x), q(p.y))).collect();
            let rk = |k: usize| int(&rres[4 * i + k], "buildings.ridge.r");
            let [a, b] = ridge_decode(&ip, int(&rsel[i], "buildings.ridge.s")?, [rk(0)?, rk(1)?, rk(2)?, rk(3)?]).ok_or("buildings.ridge: bad selector")?;
            let ridge = [Vec2 { x: m(a.0), y: m(a.1) }, Vec2 { x: m(b.0), y: m(b.1) }];
            Ok(Building {
                id: String::new(),
                poly,
                ridge,
                parcel: String::new(),
                kind: r.s(&bk[i], "buildings.kind")?,
                district: r.s(&bd[i], "buildings.district")?,
                age: 0.0,
                courtyard: bc[i],
                prov: "frozen",
            })
        })
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let building_tone = btone.iter().map(|t| int(t, "buildings.tone").map(|v| v as f64 / Q_TONE)).collect::<Result<Vec<_>, _>>()?;
    let mut wall = WallState::default();
    if let Some(w) = g.get("wall") {
        wall.ring = Some(dpts(&w["ring"], "wall.ring")?);
        wall.style = w["style"].as_str().ok_or("wall.style: not a string")?.to_string();
        for gt in arr(&w["gates"], "wall.gates")? {
            let gt = arr(gt, "wall.gates")?;
            if gt.len() != 3 {
                return Err("wall.gates: not three values".into());
            }
            wall.gates.push(Gate {
                pt: Vec2 { x: m(int(&gt[0], "wall.gates")?), y: m(int(&gt[1], "wall.gates")?) },
                water: int(&gt[2], "wall.gates")? != 0,
                prov: String::new(),
            });
        }
        for s in arr(&w["spurs"], "wall.spurs")? {
            let s = arr(s, "wall.spurs")?;
            if s.len() != 4 {
                return Err("wall.spurs: not four values".into());
            }
            let c = |i: usize| int(&s[i], "wall.spurs").map(m);
            wall.spurs.push(cartalith_urban::fortify::Spur { a: Vec2 { x: c(0)?, y: c(1)? }, b: Vec2 { x: c(2)?, y: c(3)? }, prov: "frozen" });
        }
        if let Some(c) = w.get("centroid") {
            wall.centroid = Some(dpt(c, "wall.centroid")?);
        }
        if let Some(f) = w.get("fort") {
            wall.fort = Some(cartalith_urban::fortify::Fort {
                trace: dpts(&f["trace"], "wall.fort.trace")?,
                ravelins: dpolys(&f["ravelins"], "wall.fort.ravelins")?,
                ..Default::default()
            });
        }
    }
    let citadel = match g.get("citadel") {
        Some(c) => {
            let gate = dpt(&c["gate"], "citadel.gate")?;
            Some(Citadel {
                wall: dpts(&c["wall"], "citadel.wall")?,
                towers: dpts(&c["towers"], "citadel.towers")?,
                tower_r: dbits(&c["r"], "citadel.r")?,
                keep: dpts(&c["keep"], "citadel.keep")?,
                court: dpts(&c["court"], "citadel.court")?,
                gate,
                approach: Vec::new(),
                on_curtain: gate,
                prov: "frozen",
            })
        }
        None => None,
    };
    let wall_hug = match g.get("hug") {
        Some(h) => Some(if let Some(rf) = h.get("refit") {
            let rf = arr(rf, "hug.refit")?;
            if rf.len() != 3 {
                return Err("hug.refit: not three values".into());
            }
            HugOutcome::Refit {
                clearance_m: dbits(&rf[0], "hug")?,
                area_ratio: dbits(&rf[1], "hug")?,
                streets_removed: usize::try_from(int(&rf[2], "hug")?).map_err(|_| "hug: negative count")?,
            }
        } else {
            HugOutcome::Kept(kept_reason(h["kept"].as_str().unwrap_or("")).ok_or("hug.kept: unknown reason")?)
        }),
        None => None,
    };
    let mv = r.get("markets")?;
    let mp = dpolys(&mv["poly"], "markets.poly")?;
    let mc = dpts(&mv["c"], "markets.c")?;
    let mn = arr(&mv["name"], "markets.name")?;
    if mc.len() != mp.len() || mn.len() != mp.len() {
        return Err("markets: column lengths differ".into());
    }
    let markets = mp
        .into_iter()
        .enumerate()
        .map(|(i, poly)| Ok(Market { name: r.s(&mn[i], "markets.name")?, center: mc[i], poly, prov: String::new() }))
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let faith_sites = arr(r.get("faith")?, "faith")?
        .iter()
        .map(|f| {
            let steps = dpts(&f["steps"], "faith.steps")?;
            if steps.len() % 2 != 0 {
                return Err("faith.steps: odd point count".to_string());
            }
            let tower = match f.get("tower") {
                Some(t) => {
                    let t = arr(t, "faith.tower")?;
                    if t.len() != 4 {
                        return Err("faith.tower: not four values".into());
                    }
                    Some(cartalith_urban::districts::Tower {
                        x: m(int(&t[0], "faith.tower")?),
                        y: m(int(&t[1], "faith.tower")?),
                        r: dbits(&t[2], "faith.tower")?,
                        kind: r.s(&t[3], "faith.tower")?,
                    })
                }
                None => None,
            };
            Ok(FaithSite {
                id: f["id"].as_str().ok_or("faith.id: not a string")?.to_string(),
                faith: f["faith"].as_str().ok_or("faith.faith: not a string")?.to_string(),
                name: f.get("name").and_then(Value::as_str).map(intern),
                center: dpt(&f["c"], "faith.c")?,
                yard: Vec::new(),
                polys: dpolys(&f["polys"], "faith.polys")?,
                open: dpolys(&f["open"], "faith.open")?,
                columns: dpts(&f["cols"], "faith.cols")?,
                steps: steps.chunks(2).map(|c| [c[0], c[1]]).collect(),
                tower,
                ridge: dpts(&f["ridge"], "faith.ridge")?,
                prov: f["prov"].as_str().ok_or("faith.prov: not a string")?.to_string(),
            })
        })
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let orient = dbits(r.get("orient")?, "orient")?;
    Ok(UrbanLayout {
        wm: m(int(&bx[0], "box")?),
        hm: m(int(&bx[1], "box")?),
        market: dpt(r.get("market")?, "market")?,
        pin: dpt(r.get("pin")?, "pin")?,
        market_prov: r.str_member("market_prov")?,
        site_kind: r.str_member("site_kind")?.to_string(),
        orient,
        water_poly: dpts(r.get("water")?, "water")?,
        river: dpts(r.get("river")?, "river")?,
        river_w: scal(6)?,
        bridge_pt: g.get("bridge_pt").map(|v| dpt(v, "bridge_pt")).transpose()?,
        bridges,
        ford,
        route_ends: dpts(r.get("route_ends")?, "route_ends")?,
        harbour_pt: g.get("harbour_pt").map(|v| dpt(v, "harbour_pt")).transpose()?,
        harbour,
        edges,
        blocks,
        block_plaza,
        plaza,
        parcels,
        buildings,
        building_tone,
        building_intramural,
        wall,
        citadel,
        wall_hug,
        wall_spec: r.str_member("wall_spec")?,
        markets,
        farmland: r.details("farm")?,
        details: r.details("details")?,
        street_len: scal(0)?,
        pop: scal(1)?,
        target_len: scal(2)?,
        max_rf: scal(3)?,
        parallel: None,
        pop_target: scal(4)?,
        settlement_age: scal(5)?,
        uses_real_water: real[0],
        uses_real_terrain: real[1],
        water_mask_runs: runs.chunks(2).map(|c| (c[0], c[1])).collect(),
        faith_sites,
    })
}

/// "Same town within rounding" (scope §4.2, R1): two stored forms with the same structure and strings, whose
/// integers differ by at most 2 steps each **and** whose running sums of every integer array (x and y separately)
/// differ by at most 2 steps (0.2 m), and whose exact-bit scalars agree to a relative 1e-9. A rounding flip on another
/// architecture moves one vertex one step: in a delta stream that is +1 then -1, and under the parallelogram
/// residuals of [`polys`] an alternating +1, -1, +1, -1, so every running sum stays within one step and it passes; a
/// building moved by a metre shifts a running sum by ten steps and fails. Equal values pass trivially.
pub fn within_rounding(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| within_rounding(v, w))),
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                return false;
            }
            if x.iter().all(Value::is_i64) && y.iter().all(Value::is_i64) {
                let (mut sa, mut sb) = ([0i64; 2], [0i64; 2]);
                for (i, (p, q)) in x.iter().zip(y).enumerate() {
                    let (p, q) = (p.as_i64().unwrap_or(0), q.as_i64().unwrap_or(0));
                    sa[i % 2] += p;
                    sb[i % 2] += q;
                    if (p - q).abs() > 2 || (sa[i % 2] - sb[i % 2]).abs() > 2 {
                        return false;
                    }
                }
                return true;
            }
            x.iter().zip(y).all(|(p, q)| within_rounding(p, q))
        }
        (Value::String(p), Value::String(q)) => {
            if p == q {
                return true;
            }
            // Exact-bit scalars (16 hex digits): equal to a relative 1e-9.
            match (dbits(a, ""), dbits(b, "")) {
                (Ok(x), Ok(y)) if p.len() == 16 && q.len() == 16 => (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1.0),
                _ => false,
            }
        }
        _ => a == b,
    }
}

// ------------------------------------------------------------------------------------------ anchors ---

/// What a pin binds to inside a frozen town (scope §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatKind {
    Building,
    Plot,
    Street,
    Gate,
    Bridge,
    Plaza,
    Market,
    /// Nothing nearer: the pin keeps its absolute place and never re-attaches.
    Ground,
}

impl FeatKind {
    /// The stored key.
    pub fn key(self) -> &'static str {
        match self {
            FeatKind::Building => "building",
            FeatKind::Plot => "plot",
            FeatKind::Street => "street",
            FeatKind::Gate => "gate",
            FeatKind::Bridge => "bridge",
            FeatKind::Plaza => "plaza",
            FeatKind::Market => "market",
            FeatKind::Ground => "ground",
        }
    }
    /// The kind of stored key `k`; `None` for a key this build does not know.
    pub fn from_key(k: &str) -> Option<Self> {
        [FeatKind::Building, FeatKind::Plot, FeatKind::Street, FeatKind::Gate, FeatKind::Bridge, FeatKind::Plaza, FeatKind::Market, FeatKind::Ground]
            .into_iter()
            .find(|f| f.key() == k)
    }
}

/// Where a pin sits on a frozen town (scope §4.4): the feature, its index in the frozen array (`None` for the plaza
/// and the ground), the offset from the feature's anchor point in the feature's own axes (0.1 m integers), the
/// feature's class for same-kind matching, and for a street the position along the segment (per mille).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub kind: FeatKind,
    pub fid: Option<usize>,
    pub off: (i64, i64),
    pub tag: String,
    pub u: Option<i64>,
}

/// Within this many metres of a street segment a pin binds to it (scope §4.4).
pub const STREET_BIND_M: f64 = 6.0;
/// Within this many metres of a gate a pin binds to it (scope §4.4).
pub const GATE_BIND_M: f64 = 12.0;
/// Within this many metres of a bridge a pin binds to it; a labelled judgement (a 20 m span's half plus margin).
pub const BRIDGE_BIND_M: f64 = 12.0;

/// The re-attach bar per kind (scope §4.4 step 3): the nearest same-kind, same-tag feature of the new layout within
/// this many metres of where the old one stood, in the town frame. Building and plot 25 m, street 15 m, plaza 60 m,
/// bridge 20 m, market (a named square) 60 m like the plaza. Gates match by bearing ([`GATE_BEARING_DEG`]) first.
pub fn reattach_bar_m(k: FeatKind) -> f64 {
    match k {
        FeatKind::Building | FeatKind::Plot => 25.0,
        FeatKind::Street => 15.0,
        FeatKind::Plaza | FeatKind::Market => 60.0,
        FeatKind::Bridge => 20.0,
        // A gate's bearing gate is the angle below; the distance is not bounded (scope §3.5: when the road moves the
        // gate moves a long way).
        FeatKind::Gate => f64::INFINITY,
        FeatKind::Ground => 0.0,
    }
}

/// A gate re-attaches to a new gate whose bearing from the market differs by at most this many degrees (scope §4.4).
pub const GATE_BEARING_DEG: f64 = 15.0;

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 { x: a.x - b.x, y: a.y - b.y }
}

fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 { x: a.x + b.x, y: a.y + b.y }
}

fn len(a: Vec2) -> f64 {
    a.x.hypot(a.y)
}

fn rot(p: Vec2, a: f64) -> Vec2 {
    let (s, c) = a.sin_cos();
    Vec2 { x: p.x * c - p.y * s, y: p.x * s + p.y * c }
}

/// A local point in the **town frame**: metres from the pin, rotated by `orient`, which is the map's own transform
/// without the cell (`map_overlay.gd::_draw_urban_layouts`: map point = cell + 0.5 + R(orient) (local - pin) / m per
/// cell). Two layouts of one settlement compare here, so a market that moved does not shift the search.
pub fn to_town(l: &UrbanLayout, p: Vec2) -> Vec2 {
    rot(sub(p, l.pin), l.orient)
}

/// [`to_town`] inverted.
pub fn from_town(l: &UrbanLayout, t: Vec2) -> Vec2 {
    add(l.pin, rot(t, -l.orient))
}

fn centroid(p: &[Vec2]) -> Vec2 {
    let n = p.len().max(1) as f64;
    let (x, y) = p.iter().fold((0.0, 0.0), |(x, y), q| (x + q.x, y + q.y));
    Vec2 { x: x / n, y: y / n }
}

fn inside(p: Vec2, poly: &[Vec2]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + n - 1) % n]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            c = !c;
        }
    }
    c
}

fn area(p: &[Vec2]) -> f64 {
    cartalith_urban::geom::poly_area(p).abs()
}

/// The unit axis of a direction, or the town frame's x axis (in local coordinates) when it has no length.
fn axis_of(l: &UrbanLayout, d: Vec2) -> Vec2 {
    let n = len(d);
    if n > 1e-9 { Vec2 { x: d.x / n, y: d.y / n } } else { rot(Vec2 { x: 1.0, y: 0.0 }, -l.orient) }
}

/// A feature's anchor point (local metres) and its own x axis (local unit vector); `None` for an index out of range.
fn feature_frame(l: &UrbanLayout, k: FeatKind, fid: Option<usize>, u: Option<i64>) -> Option<(Vec2, Vec2)> {
    let town_x = rot(Vec2 { x: 1.0, y: 0.0 }, -l.orient);
    Some(match k {
        FeatKind::Building => {
            let b = l.buildings.get(fid?)?;
            (centroid(&b.poly), axis_of(l, sub(b.ridge[1], b.ridge[0])))
        }
        FeatKind::Plot => (centroid(&l.parcels.get(fid?)?.poly), town_x),
        FeatKind::Street => {
            let e = l.edges.get(fid?)?;
            let t = u.unwrap_or(500) as f64 / 1000.0;
            (Vec2 { x: e.a.x + (e.b.x - e.a.x) * t, y: e.a.y + (e.b.y - e.a.y) * t }, axis_of(l, sub(e.b, e.a)))
        }
        FeatKind::Gate => (l.wall.gates.get(fid?)?.pt, town_x),
        FeatKind::Bridge => (l.bridges.as_ref()?.get(fid?)?.pt, town_x),
        FeatKind::Plaza => (l.plaza.as_ref()?.center, town_x),
        FeatKind::Market => (l.markets.get(fid?)?.center, town_x),
        FeatKind::Ground => (l.market, town_x),
    })
}

fn tag_of(l: &UrbanLayout, k: FeatKind, i: usize) -> String {
    match k {
        FeatKind::Building => l.buildings.get(i).map_or_else(String::new, |b| format!("{}|{}", b.kind, b.district)),
        FeatKind::Plot => l.parcels.get(i).map_or_else(String::new, |p| p.district.to_string()),
        FeatKind::Street => l.edges.get(i).map_or_else(String::new, |e| e.cls.to_string()),
        FeatKind::Gate => l.wall.gates.get(i).map_or_else(String::new, |g| if g.water { "water" } else { "land" }.to_string()),
        FeatKind::Market => l.markets.get(i).map_or_else(String::new, |m| m.name.to_string()),
        FeatKind::Bridge | FeatKind::Plaza | FeatKind::Ground => String::new(),
    }
}

/// The offset of `p` from a feature frame, in the frame's axes, as 0.1 m integers.
fn offset(p: Vec2, (o, ax): (Vec2, Vec2)) -> (i64, i64) {
    let d = sub(p, o);
    let ay = Vec2 { x: -ax.y, y: ax.x };
    (q(d.x * ax.x + d.y * ax.y), q(d.x * ay.x + d.y * ay.y))
}

/// Distance from `p` to segment `a..b`, and the parameter of the nearest point (0..1).
fn seg(p: Vec2, a: Vec2, b: Vec2) -> (f64, f64) {
    let d = sub(b, a);
    let l2 = d.x * d.x + d.y * d.y;
    let t = if l2 > 0.0 { (((p.x - a.x) * d.x + (p.y - a.y) * d.y) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (len(sub(p, Vec2 { x: a.x + d.x * t, y: a.y + d.y * t })), t)
}

/// Builds the anchor of kind `k`, index `fid`, for local point `p`.
fn make(l: &UrbanLayout, k: FeatKind, fid: Option<usize>, u: Option<i64>, p: Vec2) -> Anchor {
    let fr = feature_frame(l, k, fid, u).unwrap_or((l.market, Vec2 { x: 1.0, y: 0.0 }));
    Anchor { kind: k, fid, off: offset(p, fr), tag: fid.map_or_else(String::new, |i| tag_of(l, k, i)), u }
}

/// Chooses what a pin at local point `p` sits on (scope §4.4): the smallest containing building; else the containing
/// plot; else a gate within [`GATE_BIND_M`]; else a bridge within [`BRIDGE_BIND_M`]; else the nearest street segment
/// within [`STREET_BIND_M`] (with its position along it); else the plaza or a named market square if inside; else the
/// ground. Ties go to the lower index. **Gates and bridges are tried before streets, not after as scope §4.4 lists
/// them:** a gate is where a road meets the wall and a bridge carries a road, so every gate and bridge lies on a
/// street and the scope's order would never bind one (found by `gates_reattach_by_bearing`, 2026-11-22).
pub fn choose_anchor(l: &UrbanLayout, p: Vec2) -> Anchor {
    let best_bld = l
        .buildings
        .iter()
        .enumerate()
        .filter(|(_, b)| inside(p, &b.poly))
        .min_by(|(i, a), (j, b)| area(&a.poly).total_cmp(&area(&b.poly)).then(i.cmp(j)));
    if let Some((i, _)) = best_bld {
        return make(l, FeatKind::Building, Some(i), None, p);
    }
    if let Some(i) = l.parcels.iter().position(|q| inside(p, &q.poly)) {
        return make(l, FeatKind::Plot, Some(i), None, p);
    }
    let near = |pts: Vec<Vec2>, r: f64| {
        pts.into_iter().enumerate().map(|(i, q)| (i, len(sub(p, q)))).filter(|(_, d)| *d <= r).min_by(|(i, a), (j, b)| a.total_cmp(b).then(i.cmp(j)))
    };
    if let Some((i, _)) = near(l.wall.gates.iter().map(|g| g.pt).collect(), GATE_BIND_M) {
        return make(l, FeatKind::Gate, Some(i), None, p);
    }
    if let Some((i, _)) = near(l.bridges.iter().flatten().map(|b| b.pt).collect(), BRIDGE_BIND_M) {
        return make(l, FeatKind::Bridge, Some(i), None, p);
    }
    let street = l
        .edges
        .iter()
        .enumerate()
        .map(|(i, e)| (i, seg(p, e.a, e.b)))
        .filter(|(_, (d, _))| *d <= STREET_BIND_M)
        .min_by(|(i, (a, _)), (j, (b, _))| a.total_cmp(b).then(i.cmp(j)));
    if let Some((i, (_, t))) = street {
        let u = (t * 1000.0).round() as i64;
        return make(l, FeatKind::Street, Some(i), Some(u), p);
    }
    if l.plaza.as_ref().is_some_and(|z| inside(p, &z.poly)) {
        return make(l, FeatKind::Plaza, None, None, p);
    }
    if let Some(i) = l.markets.iter().position(|m| inside(p, &m.poly)) {
        return make(l, FeatKind::Market, Some(i), None, p);
    }
    make(l, FeatKind::Ground, None, None, p)
}

/// The anchor of a pin standing exactly on feature `fid` of kind `k` (its anchor point, zero offset; a street at its
/// midpoint): what a measurement places on each feature to rate re-attach (TL-4's census).
pub fn anchor_on(l: &UrbanLayout, k: FeatKind, fid: Option<usize>) -> Anchor {
    let u = (k == FeatKind::Street).then_some(500);
    Anchor { kind: k, fid, off: (0, 0), tag: fid.map_or_else(String::new, |i| tag_of(l, k, i)), u }
}

/// Where anchor `a` puts its pin on layout `l` (local metres): the feature's anchor point plus the offset in the
/// feature's axes. `None` when the index is out of range (the caller reports the pin detached).
pub fn anchor_point(l: &UrbanLayout, a: &Anchor) -> Option<Vec2> {
    let (o, ax) = feature_frame(l, a.kind, a.fid, a.u)?;
    let ay = Vec2 { x: -ax.y, y: ax.x };
    let (dx, dy) = (m(a.off.0), m(a.off.1));
    Some(Vec2 { x: o.x + ax.x * dx + ay.x * dy, y: o.y + ax.y * dx + ay.y * dy })
}

fn bearing(t: Vec2) -> f64 {
    t.y.atan2(t.x)
}

/// Re-attaches anchor `a`, made on `old`, to the new layout `new` (scope §4.4 steps 1-4): the nearest feature of the
/// same kind and the same tag within [`reattach_bar_m`] of where the old feature stood, both in the town frame; a
/// gate by bearing from the market within [`GATE_BEARING_DEG`] first, then nearest. Ties go to the lower index. The
/// returned anchor keeps the offset (so the pin keeps its place on its building) and a street keeps its `u`.
/// `None` means the pin did not find a match: the caller leaves it where it is and reports it detached. A ground
/// anchor never re-attaches (`None` too; the caller keeps it at its absolute place).
pub fn reattach(old: &UrbanLayout, a: &Anchor, new: &UrbanLayout) -> Option<Anchor> {
    if a.kind == FeatKind::Ground {
        return None;
    }
    let (o, _) = feature_frame(old, a.kind, a.fid, a.u)?;
    let want = to_town(old, o);
    if a.kind == FeatKind::Plaza {
        let z = new.plaza.as_ref()?;
        return (len(sub(to_town(new, z.center), want)) <= reattach_bar_m(FeatKind::Plaza))
            .then(|| Anchor { kind: FeatKind::Plaza, fid: None, off: a.off, tag: String::new(), u: None });
    }
    let n = match a.kind {
        FeatKind::Building => new.buildings.len(),
        FeatKind::Plot => new.parcels.len(),
        FeatKind::Street => new.edges.len(),
        FeatKind::Gate => new.wall.gates.len(),
        FeatKind::Bridge => new.bridges.as_ref().map_or(0, Vec::len),
        FeatKind::Market => new.markets.len(),
        FeatKind::Plaza | FeatKind::Ground => 0,
    };
    let at = |i: usize| -> Option<Vec2> {
        let (p, _) = feature_frame(new, a.kind, Some(i), if a.kind == FeatKind::Street { Some(500) } else { None })?;
        Some(to_town(new, p))
    };
    let cand = (0..n).filter(|i| tag_of(new, a.kind, *i) == a.tag).filter_map(|i| at(i).map(|p| (i, p)));
    let pick = if a.kind == FeatKind::Gate {
        let want_b = bearing(sub(want, to_town(old, old.market)));
        let mkt = to_town(new, new.market);
        cand.filter(|(_, p)| {
            let d = (bearing(sub(*p, mkt)) - want_b).rem_euclid(std::f64::consts::TAU);
            d.min(std::f64::consts::TAU - d).to_degrees() <= GATE_BEARING_DEG
        })
        .map(|(i, p)| (i, len(sub(p, want))))
        .min_by(|(i, a), (j, b)| a.total_cmp(b).then(i.cmp(j)))
    } else {
        // A street compares its nearest point to the old street's anchor point, not its midpoint.
        cand.map(|(i, p)| {
            let d = if a.kind == FeatKind::Street {
                let e = &new.edges[i];
                seg(want, to_town(new, e.a), to_town(new, e.b)).0
            } else {
                len(sub(p, want))
            };
            (i, d)
        })
        .filter(|(_, d)| *d <= reattach_bar_m(a.kind))
        .min_by(|(i, a), (j, b)| a.total_cmp(b).then(i.cmp(j)))
    };
    let (i, _) = pick?;
    Some(Anchor { kind: a.kind, fid: Some(i), off: a.off, tag: a.tag.clone(), u: a.u })
}

#[cfg(test)]
mod tests;
