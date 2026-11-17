//! Milestone 16 — `generate()` and `hashModel` (reference lines 30931-31094).
//!
//! The orchestration: 29 stage calls in the reference's own order, over the
//! options every earlier milestone took a projection of. Nothing new is
//! computed here — every line is a call into a module that is already
//! golden-verified on its own — so what this module is *for* is the order, the
//! branch conditions and the small amount of arithmetic `generate()` does
//! between stages (the population clamp, the church count, the head count).
//!
//! **Three stages are not the reference's, and neither is one exemption.**
//! [`build_wall_lots`] (`crate::wallside`, Ruling H, owner 2026-09-22) plats
//! lots against the wall — intramural lots backing onto its inner face, and a
//! faubourg cluster against its outer face — straight after `buildParcels` on
//! both planning branches (the radial one since Ruling AA, 2026-09-23); and the
//! faubourg is exempt from `clearFortZone`'s rampart sweep. Both are deliberate
//! departures, and the whole-town golden that moved because of them is
//! disclosed case by case in `tests/golden.rs`'s header.
//! A second departure followed: [`build_citadel`] (`crate::citadel`, Ruling I,
//! sited by Ruling AC 2026-09-23) sets a citadel astride the curtain of the
//! largest organic towns, after the rampart sweep; its one moved golden case is
//! disclosed in the same header.
//! A third: [`build_courtyard_rings`] (`crate::courtyard`, Ruling H, sited by
//! Ruling AD 2026-09-23) re-plats the organic plan's outermost ring of dense
//! blocks as perimeter blocks round an open court, straight after the wall
//! lots; disclosed in the same header.
//!
//! # Two orderings that are not interchangeable
//!
//! 1. **`detectRiverCrossings` runs after every pass that can kill an edge** —
//!    `removeWaterCrossings`, `privatizeAlleys` *and* `clearFortZone` (reference
//!    line 31074's comment says so explicitly). A bridge recorded off an earlier
//!    graph can end up with no live road on it.
//! 2. **`buildPlaza` sits in two different places.** On the organic branch it is
//!    between the primaries and `grow` (line 31024), so the town accretes
//!    *around* the market square. On the radial branch it is the **last** thing,
//!    after `buildWall` (line 31018). One function, two positions.
//!
//! # `profile.noWalls` does not exist, and that is verified rather than assumed
//!
//! Line 30955 is `const walls = opts.walls !== false && !profile.noWalls`, and
//! `noWalls` appears in the whole 2.5 MB reference exactly twice: that line and
//! the comment above it. No profile object defines the key, so `!undefined` is
//! always `true` and the term is inert — which is what the reference's own
//! comment says ("no profile in the current two-culture roster sets it").
//! [`CultureProfile`] therefore has no such field and this port writes the
//! expression without it.
//!
//! # Where this port's shape differs from the reference's, and why
//!
//! The reference mutates plain objects in place and returns them. Four stages
//! were ported as *reporting* functions rather than mutating ones, because the
//! type they would have mutated belonged to a milestone that did not exist yet
//! (each of those modules' headers records the decision). `generate()` is the
//! caller those reports were left for, so it applies them:
//!
//! | stage | reports | applied here as |
//! |---|---|---|
//! | [`build_markets`](crate::amenities::build_markets) | `cleared_parcels`, `removed_buildings` | flags + `Vec::remove` in reverse |
//! | [`apply_decay`] | `ruined_parcels`, `ruined_buildings` | flags |
//! | [`clear_fort_zone`] | `parcels_cleared`, `buildings_removed`, `details_removed` | flags + `Vec::remove` (already descending) |
//! | [`detect_river_crossings`] | [`Crossings`] | `site.bridges` / `site.ford` on [`TownSite`] |
//!
//! [`assign_districts`] likewise returns [`Lot`]s — a parcel plus the seven
//! fields the reference adds to it — instead of writing onto the parcel. That
//! borrow cannot leave this function, so the model carries [`TownParcel`],
//! which is the same ten fields owned.
//!
//! # `oreBearing` is an input here, not a site field, and that is a known debt
//!
//! [`assign_districts`] takes the ore bearing as its own parameter because
//! [`crate::site::Economy::ore_bearing`] is declared `bool` and the reference's
//! `oreBearing` is a nullable angle in radians (`districts`' header states the
//! case). So [`GenOpts::ore_bearing`] exists and is threaded straight through.
//! When `Economy` is corrected both should be deleted and the field read off
//! `site.economy`.

use crate::amenities::{Civic, GamesBuilding, Market, build_civic, build_games, build_markets_within};
use crate::blocks::{Block, Parcel, build_blocks};
use crate::citadel::{CITADEL_MIN_POP, Citadel, build_citadel, citadel_sweep};
use crate::courtyard::build_courtyard_rings;
use crate::cleanup::{clear_fort_zone, lane_pass, privatize_alleys};
use crate::districts::{
    Building, FaithSite, Lot, assign_districts, build_buildings, build_faith_sites,
};
use crate::fortify::{FortOpts, FortificationBuilder, build_wall};
use crate::geom::{Vec2, js_cos, js_max, js_min, js_or, js_round, js_sin, poly_area, poly_centroid};
use crate::graph::{Edge, Graph, Node};
use crate::growth::{GrowOpts, WallState, grow};
use crate::hinterland::{
    Detail, DetailGeom, Metrics, apply_decay, build_details, build_farmland, compute_metrics,
};
use crate::plaza::{Plaza, build_plaza};
use crate::radial::{build_radial_streets, build_waterway};
use crate::riverbank::{BOTH_BANKS_MIN_POP, RiverbankPass, TownBridge, fine_river_path, lay_town_bridge};
use crate::rng::fnv1a;
use crate::routes::{Anchors, build_primaries, build_primaries_from_paths, build_primaries_from_paths_at, place_anchors};
use crate::rules::{CultureProfile, RulesPatch, resolve_profile, resolve_rules};
use crate::site::{Economy, Site, SiteOpts, TerrainCtx, WaterCtx, build_site};
use crate::wallhug::hug_wall;
use crate::wallside::build_wall_lots;
use crate::water::{
    Bridge, Crossings, Ford, HarbourOpts, HarbourOutcome, HarbourWorks, add_river_bridges,
    build_harbour, detect_river_crossings,
};
use std::f64::consts::PI;

/// Unit tests and the whole-subsystem golden (`generate/tests/golden.rs`).
#[cfg(test)]
mod tests;
/// Ruling FK item 1's gate: the harbour front (`GenOpts::harbour_front`).
#[cfg(test)]
mod harbour_front_tests;

/// `UME.SITE_WM` — the site box width in metres (`const Wm=1700` at line 30969).
pub const SITE_WM: f64 = 1700.0;
/// `UME.SITE_HM` — the site box height in metres (`const Hm=1250`).
pub const SITE_HM: f64 = 1250.0;

/// `fortMin` (line 30957) — a bastioned trace was a state investment, never a
/// hamlet's (M-FOR-4).
pub const FORT_MIN: f64 = 2500.0;

// ------------------------------------------------------------------ options --

/// `generate()`'s `opts`, restricted to the keys it or `buildSite` reads.
///
/// Every field is the reference's falsy-default, not a choice made here:
/// [`Default`] gives `culture: None` (→ `medieval`), `site: None` (→
/// `'river'`), `epochs: None` (→ 8), `pop: None` (→ 5000), `settlement_age:
/// None` (→ 300) and `walls: None` (→ enclosed), which is exactly what
/// `generate(seed)` with no options does in the reference.
#[derive(Debug, Clone, Default)]
pub struct GenOpts {
    /// `opts.culture` — an unknown or absent id resolves to `medieval`.
    pub culture: Option<String>,
    /// `opts.rules` — the partial merged over [`DEFAULT_RULES`](crate::rules::DEFAULT_RULES).
    pub rules: Option<RulesPatch>,
    /// `opts.site` — `'river'`, `'riverthrough'`, `'coast'`, `'estuary'`,
    /// `'inland'`. A falsy value (including `""`) becomes `'river'`.
    pub site: Option<String>,
    /// `!!opts.terrainAware` — the terrain-suitability building gate (docs/08).
    pub terrain_aware: bool,
    /// `!!opts.ruined` — a state any settlement can be in, not a culture.
    pub ruined: bool,
    /// `!!opts.wallGenerations` — successive circuits (M-GRW-2). Organic only;
    /// inert on the radial branch, which never calls `grow`.
    pub wall_generations: bool,
    /// `opts.settlementAge` in years. Falsy → 300, then clamped to `[30, 1000]`.
    pub settlement_age: Option<f64>,
    /// `opts.epochs`. Falsy → 8. (The reference would also accept a non-integer
    /// here; nothing in the engine does anything sensible with one, and an
    /// `i32` is what every consumer already takes.)
    pub epochs: Option<i32>,
    /// `opts.pop`. Falsy → 5000, then clamped to `[400, 20000]`.
    pub pop: Option<f64>,
    /// `opts.walls` — the reference tests `!== false`, so **only** an explicit
    /// `Some(false)` disables the enclosure. `None` is "not supplied", which is
    /// walled.
    pub walls: Option<bool>,
    /// `!!opts.fortified` — a *request*; see [`Town::fortified`] for the three
    /// further conditions.
    pub fortified: bool,
    /// `opts.wallStyle` — forwarded to `buildWall` untouched.
    pub wall_style: Option<String>,
    /// `opts.faith` — falsy takes `profile.defaultFaith`. `'none'` builds no
    /// place of worship at all.
    pub faith: Option<String>,
    /// `opts.civicStyle` — falsy takes `profile.defaultCivic`.
    pub civic_style: Option<String>,
    /// `opts.harbourDefence` / `opts.harbourScale`.
    pub harbour_defence: Option<String>,
    /// See [`Self::harbour_defence`].
    pub harbour_scale: Option<f64>,
    /// `opts.water` — the host's real water raster. Its presence also switches
    /// on the market pin (line 30985).
    pub water: Option<WaterCtx>,
    /// `opts.terrain` — the host's real relief raster.
    pub terrain: Option<TerrainCtx>,
    /// `opts.economy` — the settlement's specialisation.
    pub economy: Option<Economy>,
    /// `site.economy.oreBearing`, in radians. See this module's header for why
    /// it is here rather than on [`Economy`].
    pub ore_bearing: Option<f64>,
    /// `opts.routeEnds` — the host's real approach-road endpoints. Non-empty
    /// overrides `site.routeEnds` verbatim.
    pub route_ends: Vec<Vec2>,
    /// `opts.primaryPaths` — the host's real inter-settlement roads. Non-empty
    /// takes `buildPrimariesFromPaths` instead of `buildPrimaries`.
    pub primary_paths: Vec<Vec<Vec2>>,
    /// **Not the reference** -- Ruling EW item 1 (owner, 2026-11-13): the town wall
    /// hugs the built-up area ([`crate::wallhug`]); since 2026-11-17 its
    /// stretches over open water are set on the shore
    /// ([`crate::wallhug::snap_wall_to_shore`]). `false` is the reference's
    /// growth-reserve circuit, byte for byte, so every JS-captured golden and
    /// every other `GenOpts` literal is unmoved by control flow.
    pub wall_hug: bool,
    /// **Not the reference** -- 2026-11-15, under Ruling EX's follow-ups
    /// (owner: *"per town placement is critical. If it misses it just looks
    /// silly"*): on a real-water river site the town builds to the channel's
    /// bank ([`crate::riverbank`]). `false` is the reference, byte for byte, so
    /// every JS-captured golden and every other `GenOpts` literal is unmoved by
    /// control flow.
    pub riverbank: bool,
    /// **Not the reference** -- Ruling FE item 2 (owner, 2026-11-15:
    /// *"Villages: rows along the road"*): a settlement under
    /// [`crate::plaza::PLAZA_MARKET_POP`] on the organic plan is laid out as a
    /// street village ([`crate::village`]): no 1 600 m street floor, lanes only
    /// by the roads, the green at the market point and tofts lining the roads.
    /// On a riverbank river site (2026-11-16, the town leftovers) its market
    /// also goes down to its river ([`crate::village::landing_market`]) or
    /// across it to the bank its roads are on
    /// ([`crate::village::road_bank_market`]), and a road along the bank is
    /// lined on its river side too (`crate::village::plat_tofts`); since
    /// 2026-11-17 a market so moved gets a lane down to the water
    /// ([`crate::village::landing_lane`]) whose tofts, and a harbour street's,
    /// are platted first. `false` is the reference, byte for byte, so every JS-captured golden and
    /// every other `GenOpts` literal is unmoved by control flow.
    pub street_village: bool,
    /// **Not the reference** -- `TOWN_GENERATOR_JUDGEMENT.md` change 6
    /// (2026-11-15): on the organic plan the market square (or a village's
    /// green) is sized from the town's `pop_target`
    /// ([`crate::plaza::square_scale`]) instead of the reference's one size,
    /// and placed nearest the market point where it fits. `false` is the
    /// reference, byte for byte, so every JS-captured golden and every other
    /// `GenOpts` literal is unmoved by control flow.
    pub market_scale: bool,
    /// **Not the reference** -- Ruling FK item 1 (owner, 2026-11-16: *"the
    /// town layout grows toward its sea (quay, harbour district) ... so no
    /// harbour stands inland of its own water"*): on a real-water site the
    /// market, which the reference pins at the box centre, moves toward the
    /// nearest open water when it stands farther from it than the reference's
    /// own market-to-harbour distance ([`harbour_front_market`]), and the
    /// host's real roads stay where the map draws them
    /// ([`build_primaries_from_paths_at`](crate::routes::build_primaries_from_paths_at)).
    /// The host sets it only for a town whose box holds sea. `false` is the
    /// reference, byte for byte, so every JS-captured golden and every other
    /// `GenOpts` literal is unmoved by control flow.
    pub harbour_front: bool,
    /// **Not the reference** -- Ruling FK item 1: the host's water with the
    /// town's river carried on to its sea (`cartalith_civ`'s
    /// `UmWater::to_sea`), which [`generate`]'s second layout is laid out
    /// from in place of [`Self::water`]. Read only with
    /// [`Self::harbour_front`]; `None` is the river as traced.
    pub water_to_sea: Option<WaterCtx>,
    /// **Not the reference** -- Ruling FN (owner, 2026-11-16): the layout box,
    /// `(width, height)` in metres, when the host lays a stretched river town
    /// out in a box larger than the reference's [`SITE_WM`] by [`SITE_HM`]
    /// ([`crate::stretch::stretch_half_box`]). The market stays at its centre.
    /// `None` is the reference's box, so every JS-captured golden and every
    /// other `GenOpts` literal is unmoved by control flow.
    pub site_box: Option<(f64, f64)>,
    /// **Not the reference** -- Ruling FN: the town grows in a river-following
    /// frame stretched by this plan ([`crate::stretch::StretchFrame`]), the
    /// host's [`crate::stretch::plan_river_stretch`]. Read only on the organic
    /// plan, on a river site with a centreline. `None` is the reference's
    /// growth round the market, by control flow.
    pub river_stretch: Option<crate::stretch::StretchPlan>,
    /// **Not the reference** -- owner Ruling FE item 3 (2026-11-15: *"Deep
    /// burgage plots with backland are built behind their own switch, measured
    /// and shown in frames before the owner decides"*), a prototype
    /// ([`crate::burgage`]): streets further apart, deeper lots, fewer interior
    /// lanes. Read only on the organic plan of a town that
    /// is not a street village. `false` is the reference's layout, by control
    /// flow. The host sets it from `WorldParams::urban_burgage`, off in the
    /// app until the owner rules.
    pub burgage: bool,
}

/// `maxRf` (reference line 30967): the growth front's full radius, metres, for
/// a town of (clamped) population target `pop_target`. Split out of
/// [`generate`] (2026-11-16, Ruling FN) so the host can size a neighbour's
/// reach ([`crate::stretch::StretchNeighbour`]) by the same arithmetic; the
/// expression is the reference's, term for term.
pub fn town_reach_m(pop_target: f64) -> f64 {
    js_min(720.0, (pop_target * 21.0).sqrt() * 1.35 + 80.0)
}

/// The reference's market-to-harbour distance on a sea site, metres:
/// `placeAnchors` (`crate::routes::place_anchors`) scores a market candidate
/// by `|d - 100|` from the harbour point (`|d - 120|` on a `river` site, see
/// [`HARBOUR_FRONT_RIVER_M`]), so this is where the reference itself puts a
/// harbour town's market before the real-water branch pins it at the box
/// centre. [`harbour_front_market`] moves a market no closer to its water
/// than this.
pub const HARBOUR_FRONT_M: f64 = 100.0;
/// The same for a `river` site: `placeAnchors`' `|d - 120|`.
pub const HARBOUR_FRONT_RIVER_M: f64 = 120.0;

/// **Not the reference** -- Ruling FK item 1 (owner, 2026-11-16): where a
/// harbour town's market goes when the box centre `mc` stands inland of its
/// water, or `None` when it stays.
///
/// The nearest open-water cell of the host's raster (`WaterCtx::open_water`:
/// sea and lakes, before the river band is stamped, so a river is never
/// "the water" here) is found by the distance from `mc` to the cell's square
/// (ties: the first cell in row order). When that distance is more than the
/// reference's market-to-harbour distance ([`HARBOUR_FRONT_M`], or
/// [`HARBOUR_FRONT_RIVER_M`] on a `river` site), the market moves along the
/// straight line from `mc` toward that point until it stands exactly that far
/// from it. The line is dry: no open-water cell is nearer to `mc` than the
/// point it aims at. A market already that close, a site with no
/// open-water raster (or one of the wrong size) or with no open water at all
/// stays (`None`). The caller's ring search still moves a market that lands
/// in a channel onto the bank.
///
/// Why the reference's distance: `placeAnchors` sets a harbour town's market
/// about 100 m from its harbour point, and the reference's real-water branch
/// (v0.98, `generate`) overrode that with the box centre only so the market
/// would land on the map pin, which left a town 400-650 m inland of a sea at
/// the box's edge (`cfa1cf0d`'s frames). The map pin is kept by reporting it
/// separately ([`Town::pin`]), not by holding the market on it.
///
/// Must never return a point farther from the water than `mc`, nor move a
/// market that is already within the distance.
pub fn harbour_front_market(site: &Site, mc: Vec2) -> Option<Vec2> {
    harbour_front(site, mc).map(|(m, _)| m)
}

/// [`harbour_front_market`]'s market, with the water point it aims at (the
/// nearest point of the nearest open-water cell), for [`harbour_street`].
fn harbour_front(site: &Site, mc: Vec2) -> Option<(Vec2, Vec2)> {
    let w = site.water_ctx()?;
    let open = w.open_water.as_deref().filter(|o| o.len() == w.mw * w.mh)?;
    let c = w.cell_m;
    let mut best: Option<(f64, Vec2)> = None;
    for j in 0..w.mh {
        for i in 0..w.mw {
            if open[j * w.mw + i] == 0 {
                continue;
            }
            let (x0, y0) = (i as f64 * c, j as f64 * c);
            let q = Vec2::new(mc.x.clamp(x0, x0 + c), mc.y.clamp(y0, y0 + c));
            let d = q.dist(mc);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, q));
            }
        }
    }
    let (d, q) = best?;
    let front = if site.kind == "river" { HARBOUR_FRONT_RIVER_M } else { HARBOUR_FRONT_M };
    if d <= front {
        return None;
    }
    Some((q + (mc - q).norm() * front, q))
}

/// How far short of the open water's edge, metres, [`harbour_street`] stops:
/// the harbour's own quay inset on a coast (`crate::water::build_harbour`'s
/// 5 m), so the street meets the quay line rather than running into the sea.
pub const HARBOUR_STREET_INSET_M: f64 = 5.0;
/// The harbour street's width, metres: `crate::water::build_harbour`'s back
/// street and stubs (4 m), the harbour's own lesser streets.
pub const HARBOUR_STREET_W: f64 = 4.0;
/// Provenance of [`harbour_street`]'s street.
pub const HARBOUR_STREET_PROV: &str = "Harbour street: from the market down to the water's edge, so the town's streets reach its sea (Ruling FK item 1).";

/// **Not the reference** -- Ruling FK item 1 (owner, 2026-11-16: *"a quay or
/// harbour district on the shore, its streets reaching the water"*): a street
/// from a harbour-front market `m` straight down to its open water, stopping
/// [`HARBOUR_STREET_INSET_M`] short of the water's edge, laid before the town
/// grows so the town grows along it (and a street village lines it with
/// houses) to the water. Returns the street's water end, or `None`.
///
/// It aims at `q`, the water point the market was moved toward
/// ([`harbour_front`]), when the line there is dry ([`Site::is_water`] at
/// eleven samples, the test the water-crossing sweep reads); otherwise at the
/// next nearest point of an open-water cell (`WaterCtx::open_water`) that a
/// dry line reaches, no farther than twice `q`'s distance: beside a river
/// mouth the straight line to the sea can run along the channel. Ties go to
/// the first cell in row order.
pub fn harbour_street(site: &Site, g: &mut Graph, m: Vec2, q: Vec2) -> Option<Vec2> {
    let end_of = |p: Vec2| {
        let d = m.dist(p);
        (d > HARBOUR_STREET_INSET_M + 1.0).then(|| m + (p - m).norm() * (d - HARBOUR_STREET_INSET_M))
    };
    let dry = |e: Vec2| (0..=10).all(|i| !site.is_water(m.lerp(e, i as f64 / 10.0)));
    let mut end = end_of(q).filter(|&e| dry(e));
    if end.is_none()
        && let Some(w) = site.water_ctx()
        && let Some(open) = w.open_water.as_deref().filter(|o| o.len() == w.mw * w.mh)
    {
        let (c, reach) = (w.cell_m, 2.0 * m.dist(q));
        let mut cands: Vec<(f64, Vec2)> = Vec::new();
        for j in 0..w.mh {
            for i in 0..w.mw {
                if open[j * w.mw + i] == 0 {
                    continue;
                }
                let (x0, y0) = (i as f64 * c, j as f64 * c);
                let p = Vec2::new(m.x.clamp(x0, x0 + c), m.y.clamp(y0, y0 + c));
                let d = p.dist(m);
                if d <= reach {
                    cands.push((d, p));
                }
            }
        }
        // Stable: equal distances keep row order.
        cands.sort_by(|a, b| a.0.total_cmp(&b.0));
        end = cands.iter().filter_map(|&(_, p)| end_of(p)).find(|&e| dry(e));
    }
    let end = end?;
    g.add_street(m.x, m.y, end.x, end.y, "street", HARBOUR_STREET_W, 0, HARBOUR_STREET_PROV);
    Some(end)
}

// ------------------------------------------------------------------- output --

/// The `site` sub-object of the returned model (line 31081), which is a
/// **projection** of [`Site`](crate::site::Site) — thirteen named fields, not
/// the site itself — plus the two crossing fields `detectRiverCrossings` writes
/// and the harbour's refusal reason.
#[derive(Debug, Clone, PartialEq)]
pub struct TownSite {
    pub kind: String,
    pub through: bool,
    pub no_water: bool,
    pub river: Vec<Vec2>,
    pub river_w: f64,
    pub water_poly: Vec<Vec2>,
    pub bridge_pt: Option<Vec2>,
    pub bridge_dir: Option<Vec2>,
    pub route_ends: Vec<Vec2>,
    /// `site.bridges || null` — where a live road really crosses the real river.
    pub bridges: Option<Vec<Bridge>>,
    /// `site.ford || null` — the fallback when a through-town has no bridge.
    pub ford: Option<Ford>,
    /// `site.harbourInvalid || null` — `'unnavigable'` or `'cliff'`.
    pub harbour_invalid: Option<&'static str>,
}

/// `graph:{nodes:g.nodes,edges:g.edges.filter(e=>e.alive)}` (line 31085).
///
/// **The node list is not filtered and the edge list is** — so an index into
/// `nodes` is still a node id, while `edges` has lost its correspondence with
/// `Edge::id`. `hashModel` walks both in this order.
#[derive(Debug, Clone, PartialEq)]
pub struct TownGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// One parcel as the returned model carries it: [`Parcel`] plus the seven
/// fields [`assign_districts`] / [`build_buildings`] / [`build_faith_sites`]
/// add to it, plus the two `generate()` itself sets.
///
/// This is [`Lot`] with the parcel owned. `Lot` borrows, so it cannot leave
/// [`generate`].
#[derive(Debug, Clone, PartialEq)]
pub struct TownParcel {
    pub par: Parcel,
    /// `''` until [`assign_districts`] runs; `hashModel` serialises it as
    /// `p.district || ''`, which is the same string.
    pub district: &'static str,
    pub prov_district: &'static str,
    pub suitability: f64,
    pub empty: bool,
    pub unsuitable: bool,
    pub built: bool,
    pub churchyard: bool,
    /// `par.cleared` — set by [`build_markets`](crate::amenities::build_markets) and [`clear_fort_zone`]. Written
    /// twice in block 4 and read nowhere in it; carried because it is the
    /// model's, and a renderer wants to know the lot was swept.
    pub cleared: bool,
    /// `p.ruined` — [`apply_decay`]'s. Read by `generate()`'s own head count.
    pub ruined: bool,
}

/// `generate()`'s return value (lines 31078-31086).
#[derive(Debug, Clone)]
pub struct Town {
    pub seed: u32,
    pub epochs: i32,
    pub pop_target: f64,
    pub walls: bool,
    /// The *effective* flag: requested **and** big enough **and** walled **and**
    /// on an `'organic'` gate scheme (the anachronism guard — a trace italienne
    /// is a c.1500 answer to gunpowder artillery).
    pub fortified: bool,
    /// `!!opts.fortified`, before those three conditions.
    pub fort_requested: bool,
    pub fort_min: f64,
    pub terrain_aware: bool,
    pub ruined: bool,
    pub wall_generations: bool,
    pub settlement_age: f64,
    /// `targetLen` (line 30966) — the metres of street the epoch loop aims at.
    /// Returned because it is a *derived input* a caller cannot recover from
    /// [`Self::pop_target`] without restating the formula, and restating it is
    /// how two copies of a constant start disagreeing. The reference's own
    /// model does not carry it; nothing golden reads it.
    pub target_len: f64,
    /// `maxRf` (line 30967) — the urban radius the growth front stops at. Here
    /// for the same reason as [`Self::target_len`].
    pub max_rf: f64,
    pub wm: f64,
    pub hm: f64,
    pub culture: &'static str,
    pub culture_name: &'static str,
    pub site: TownSite,
    pub anchors: Anchors,
    pub plaza: Option<Plaza>,
    pub harbour: Option<HarbourWorks>,
    pub markets: Vec<Market>,
    pub civic: Option<Civic>,
    pub games: Vec<GamesBuilding>,
    pub graph: TownGraph,
    pub wall: WallState,
    pub blocks: Vec<Block>,
    pub parcels: Vec<TownParcel>,
    pub buildings: Vec<Building>,
    /// Parallel to [`Self::buildings`]: the reference's `b.ruined`, which
    /// [`apply_decay`] sets and nothing in block 4 reads. A parallel `Vec<bool>`
    /// rather than a wrapper struct, because it is one flag against
    /// [`TownParcel`]'s nine — and it survives the two building splices below,
    /// so an index into it is an index into `buildings`.
    pub building_ruined: Vec<bool>,
    pub churches: Vec<FaithSite>,
    pub details: Vec<Detail>,
    /// **Not the reference** — a citadel astride the curtain (Ruling I, sited
    /// by Ruling AC on the size tier). [`None`] below
    /// [`CITADEL_MIN_POP`], on the radial plan, with no circuit or a bastioned
    /// one, or where no stretch of curtain qualifies. **Not part of
    /// [`Self::wall`]'s ring** — see `crate::citadel` for that default.
    pub citadel: Option<Citadel>,
    /// **Not the reference** -- what Ruling EW's wall hug did ([`crate::wallhug`]):
    /// [`None`] when [`GenOpts::wall_hug`] was off or the town has no wall, else the
    /// outcome (a refit with its clearance, or why the circuit was kept).
    pub wall_hug: Option<crate::wallhug::HugOutcome>,
    /// **Not the reference** -- whether [`GenOpts::riverbank`] switched this
    /// town's site to the riverbank geometry ([`crate::site::Site::use_riverbank`]):
    /// `false` with the option off, and with it on for a site without a real
    /// river path or an open-water raster. A renderer reads it to draw the river
    /// from `water_poly` rather than the stamped mask.
    pub riverbank: bool,
    /// What [`crate::riverbank::build_embankment`] laid, [`None`] when
    /// [`Self::riverbank`] is off.
    pub embankment: Option<crate::riverbank::Embankment>,
    /// **Not the reference** -- what Ruling FB's town bridge did
    /// ([`crate::riverbank::lay_town_bridge`]): [`None`] for a town laid out
    /// once (the switch off, a village, the radial plan, a site with no
    /// river, or a town that housed its target on its own bank); laid only
    /// in a [`RiverbankPass::FarBank`] layout.
    pub town_bridge: Option<TownBridge>,
    /// **Not the reference** -- whether Ruling FB's both-banks rule applies to
    /// this town: `urban_riverbank` on, a `river` or `riverthrough` site of
    /// [`BOTH_BANKS_MIN_POP`] and up, the organic plan. `false` with the
    /// switch off.
    pub both_banks: bool,
    /// **Not the reference** -- which of `generate`'s layouts this town is
    /// ([`RiverbankPass`]); `None` with both `urban_riverbank` and
    /// `urban_street_village` off (the reference's one layout).
    pub riverbank_pass: Option<RiverbankPass>,
    /// **Not the reference** -- whether this town was laid out as a street
    /// village ([`crate::village`], Ruling FE item 2): `GenOpts::street_village`
    /// on, under `PLAZA_MARKET_POP`, the organic plan. `false` with it off.
    pub street_village: bool,
    /// **Not the reference** -- where the settlement's map position falls in
    /// this layout's box: the point a renderer anchors onto the settlement's
    /// cell. [`Self::anchors`]' market, as in the reference (whose
    /// `_umDrawLayout` anchors on the market), except where Ruling FK's harbour
    /// front ([`GenOpts::harbour_front`], [`harbour_front_market`]) moved the
    /// market toward the sea: then the box centre, which is where the host
    /// sampled the map, so the layout's water still lies on the map's.
    /// And where a street village's market went down to its river
    /// ([`crate::village::landing_market`]) or across it to its roads' bank
    /// ([`crate::village::road_bank_market`]; both `GenOpts::street_village`):
    /// then the market the reference placed, so the village is anchored on
    /// the map exactly as before.
    pub pin: Vec2,
    /// The head count: 5.2 per built, non-churchyard, non-ruined parcel,
    /// accumulated in parcel order and then rounded.
    pub pop: f64,
    pub metrics: Metrics,
    pub through: bool,
}

// ------------------------------------------------------------------- helpers --

/// JS `s || d` for a string: `undefined`, `null` and `""` are all falsy.
fn or_str<'a>(s: Option<&'a str>, d: &'a str) -> &'a str {
    match s {
        Some(v) if !v.is_empty() => v,
        _ => d,
    }
}

// ----------------------------------------------------------------- generate --

/// `generate(seed, opts)` — reference line 30931.
///
/// Every stage in the reference's order; see this module's header for the two
/// orderings that are load-bearing and for the four report-and-apply stages.
///
/// **Not the reference, with `urban_riverbank` on only** (2026-11-15, Ruling
/// FB and the village cap): a river town on a riverbank site may be laid out
/// twice. The first layout is [`RiverbankPass::First`]; when it houses less
/// than its `pop_target` and may grow on both banks, or is a village housing
/// more than `crate::riverbank::RIVER_HOUSING_CAP` of it, the town is laid out
/// again with the second pass `crate::riverbank::second_pass` names, from the
/// same seed, so the second differs from the first only where that pass acts
/// (the far bank after the epochs, or a village's street budget, which
/// `crate::riverbank::fit_village` searches over a few layouts). With the
/// switch off the first layout is returned as it stands: the reference.
///
///
/// **Not the reference, with `harbour_front` on only** (Ruling FK item 1,
/// owner 2026-11-16): the town is laid out a second time when its body does
/// not come within [`HARBOUR_REACH_M`] of its open water
/// ([`town_reaches_water`]), with its market moved to the water
/// ([`harbour_front_market`]), or when the host carried its river on to the
/// sea ([`GenOpts::water_to_sea`]), from that river (and, where the market
/// move with it costs the town its houses, from that river alone); and a
/// second layout is returned when it changed something (the market moved, or
/// the river) and houses at least [`HARBOUR_HOUSING_KEEP`] of the first
/// layout's people or of its target, whichever is less. A town whose body already reaches its
/// water keeps its market where the reference puts it (a large town that
/// fills its box to the shore is not shrunk by a market pulled toward a strip
/// of sea at its edge), and a second layout that would cost the town its
/// houses (a hamlet pressed against a river mouth running out of frontage, a
/// river carried across the middle of a town) is not kept. With the switch
/// off, the first layout: the reference.
pub fn generate(seed: u32, opts: &GenOpts) -> Town {
    let town = generate_laid(seed, opts, false);
    let Some(w) = opts.water.as_ref().filter(|_| opts.harbour_front) else {
        return town;
    };
    let front = !town_reaches_water(&town, w, HARBOUR_REACH_M);
    let to_sea = opts.water_to_sea.as_ref();
    if !front && to_sea.is_none() {
        return town;
    }
    let keep = HARBOUR_HOUSING_KEEP * town.pop.min(town.pop_target);
    // The market at the water with the river carried on, then (where the
    // first costs the town its houses) the river carried on alone.
    let tries: &[bool] = if front && to_sea.is_some() { &[true, false] } else { &[front] };
    for &f in tries {
        let second = match to_sea {
            Some(ws) => generate_laid(seed, &GenOpts { water: Some(ws.clone()), water_to_sea: None, ..opts.clone() }, f),
            None => generate_laid(seed, opts, f),
        };
        let changed = to_sea.is_some() || second.pin != second.anchors.market;
        if changed && second.pop >= keep {
            return second;
        }
    }
    town
}

/// [`generate`]'s layouts for one market placement: the first layout, then
/// the riverbank second pass where `crate::riverbank::second_pass` asks for
/// one. `front` is whether the harbour front may move the market
/// ([`GenOpts::harbour_front`] must be on too).
fn generate_laid(seed: u32, opts: &GenOpts, front: bool) -> Town {
    let first = generate_pass(seed, opts, RiverbankPass::First, front);
    match crate::riverbank::second_pass(&first) {
        Some(RiverbankPass::Village { .. }) => crate::riverbank::fit_village(first, |street_scale, trim| {
            generate_pass(seed, opts, RiverbankPass::Village { street_scale, trim }, front)
        }),
        Some(pass) => generate_pass(seed, opts, pass, front),
        None => first,
    }
}

/// **Not the reference** -- Ruling FK item 1: whether a market at `q` can be
/// joined by a dry line to one of the host's roads (`paths`, offsets from the
/// map pin `origin`, laid as [`build_primaries_from_paths_at`] lays them:
/// their points inside the box), by [`crate::routes::dry_join`]. `true` with
/// no roads at all (nothing to join). For the market ring search in
/// `generate_pass` only.
fn harbour_market_reaches_a_road(site: &Site, q: Vec2, origin: Vec2, paths: &[Vec<Vec2>]) -> bool {
    let in_box = |p: Vec2| p.x >= -6.0 && p.x <= site.wm + 6.0 && p.y >= -6.0 && p.y <= site.hm + 6.0;
    let roads: Vec<Vec<Vec2>> = paths
        .iter()
        .map(|path| path.iter().map(|o| Vec2::new(origin.x + o.x, origin.y + o.y)).take_while(|&p| in_box(p)).collect::<Vec<_>>())
        .filter(|pts| pts.len() >= 2)
        .collect();
    roads.is_empty() || roads.iter().any(|pts| crate::routes::dry_join(site, q, pts).is_some())
}

/// The share of the first layout's housed people (`Town::pop`), or of its
/// target (`Town::pop_target`) where that is less, a harbour-front layout must
/// house to replace it in [`generate`]. **A labelled judgement**: moving a
/// town to its water must not cost it a tenth of the houses it needs. On the
/// census (`town_river_census.rs::harbour_front_audit`, 384 x 288) one hamlet
/// pressed between a river mouth and the sea housed 166 of its 400 people from
/// the water against 369 where it stood, and a town of 7 904 housed 11 430
/// from the water against 13 369 (both over its target).
pub const HARBOUR_HOUSING_KEEP: f64 = 0.9;

/// How near its open water, metres, a harbour town's nearest house must stand
/// for [`generate`] to call the town built to its water (Ruling FK item 1):
/// **one street**, the quay, between the houses and the drawn water. A
/// labelled judgement from the harbour's own geometry
/// ([`crate::water::build_harbour`]): its shoreline is the land cells beside
/// the 22 m water raster, at their centres, so half a cell (11 m) from the
/// drawn water's edge; the quay is laid 5 m inland of that line and is 7 m
/// wide, so the lots behind it begin 11 + 5 + 7 = 23 m from the water. The
/// census (`town_river_census.rs::harbour_front_audit`) counts harbours over
/// this line.
pub const HARBOUR_REACH_M: f64 = 23.0;

/// Whether the town's **body** reaches its water: [`houses_reach_water`] over
/// the buildings whose centroid stands within [`Town::max_rf`] (the radius the
/// growth front stops at) of the market. Not every building: the reference's
/// harbour lays its quay and lots at the shore however far the town is, so a
/// handful of quay houses 650 m from the walls answered "yes" for a town whose
/// wall stood that far inland of its sea (483920 359:258 in `cfa1cf0d`'s and
/// this change's frames).
pub fn town_reaches_water(t: &Town, w: &WaterCtx, reach: f64) -> bool {
    let body: Vec<crate::districts::Building> =
        t.buildings.iter().filter(|b| poly_centroid(&b.poly).dist(t.anchors.market) <= t.max_rf).cloned().collect();
    houses_reach_water(&body, w, reach)
}

/// Whether any building of `buildings` has a vertex or an edge midpoint
/// within `reach` metres of an open-water cell of `w` (`WaterCtx::open_water`,
/// sea and lakes; distance to the cell's square, so 0 inside it). `false`
/// without the raster, with one of the wrong size, or with no building. For
/// [`generate`]'s harbour-front test; reads nothing else.
pub fn houses_reach_water(buildings: &[crate::districts::Building], w: &WaterCtx, reach: f64) -> bool {
    let Some(open) = w.open_water.as_deref().filter(|o| o.len() == w.mw * w.mh) else {
        return false;
    };
    let c = w.cell_m;
    let r = (reach / c).ceil() as i64 + 1;
    let near = |p: Vec2| {
        let (ci, cj) = ((p.x / c).floor() as i64, (p.y / c).floor() as i64);
        for j in (cj - r).max(0)..=(cj + r).min(w.mh as i64 - 1) {
            for i in (ci - r).max(0)..=(ci + r).min(w.mw as i64 - 1) {
                if open[j as usize * w.mw + i as usize] == 0 {
                    continue;
                }
                let (x0, y0) = (i as f64 * c, j as f64 * c);
                let q = Vec2::new(p.x.clamp(x0, x0 + c), p.y.clamp(y0, y0 + c));
                if q.dist(p) <= reach {
                    return true;
                }
            }
        }
        false
    };
    buildings.iter().any(|b| {
        let n = b.poly.len();
        (0..n).any(|k| near(b.poly[k]) || near(b.poly[k].lerp(b.poly[(k + 1) % n], 0.5)))
    })
}

/// One layout of [`generate`], under one [`RiverbankPass`] (`First` is the
/// reference's whole `generate()` when `urban_riverbank` is off).
///
/// `front` lets Ruling FK's harbour front move the market (see [`generate`]);
/// `false` is the reference's market.
#[allow(clippy::too_many_lines)]
pub(crate) fn generate_pass(seed: u32, opts: &GenOpts, pass: RiverbankPass, front: bool) -> Town {
    let profile: CultureProfile = resolve_profile(or_str(opts.culture.as_deref(), ""));
    let rules = resolve_rules(opts.rules.as_ref());
    let terrain_aware = opts.terrain_aware;
    let ruined = opts.ruined;
    let wall_generations = opts.wall_generations;
    let settlement_age = js_max(30.0, js_min(1000.0, js_or(opts.settlement_age.unwrap_or(0.0), 300.0)));
    let epochs = match opts.epochs {
        Some(e) if e != 0 => e,
        _ => 8,
    };
    let pop_target = js_max(400.0, js_min(20000.0, js_or(opts.pop.unwrap_or(0.0), 5000.0)));
    // `opts.walls !== false && !profile.noWalls`; the second term is inert —
    // see this module's header.
    let walls = opts.walls != Some(false);
    let fort_requested = opts.fortified;
    let fortified = fort_requested
        && pop_target >= FORT_MIN
        && walls
        && profile.wall_gates_scheme == "organic";
    // M-DEN-1/2, M-PAR-1: ~150 p/ha and ~8 m frontages give ~2.1 m of street per
    // inhabitant; the floor keeps a hamlet a crossroads cluster (M-AMEN-2).
    let target_len = js_max(1600.0, js_min(42000.0, pop_target * 2.1));
    let max_rf = town_reach_m(pop_target);
    // **Not the reference** (Ruling FE item 2, `crate::village`): a street
    // village. Organic plan only (the radial plan lays its own rings).
    let street_village = opts.street_village && pop_target < crate::plaza::PLAZA_MARKET_POP && profile.planning != "radial";
    // Its lanes: a quarter of a town's street a head, with no floor.
    let target_len = if street_village { pop_target * crate::village::STREET_PER_HEAD_M } else { target_len };
    // **Not the reference** (owner Ruling FE item 3's prototype,
    // `crate::burgage`): deep plots, on the organic plan of a town that is not
    // a street village (the radial plan never calls `grow`; a street village
    // has its tofts). Its street rules and, below, the deeper platting and
    // fewer lanes; `false` touches nothing.
    let burgage = opts.burgage && !street_village && profile.planning != "radial";
    let rules = if burgage { crate::burgage::rules(rules) } else { rules };
    // **Not the reference** (Ruling FN, `GenOpts::site_box`): a stretched
    // river town's larger box; `None` is the reference's.
    let (wm, hm) = opts.site_box.unwrap_or((SITE_WM, SITE_HM));
    let site_kind = or_str(opts.site.as_deref(), "river").to_string();

    // **Not the reference** (2026-11-15, `crate::riverbank`): with the
    // riverbank geometry a real river's centreline is clipped near the box and
    // resampled every 8 m before the site is built, so the stages that walk it
    // vertex by vertex (the bridge search, the river port, the harbour's shore)
    // walk metres, not map cells. Only where `use_riverbank` below will take
    // the site (an open-water raster of the mask's size): a site it refuses
    // keeps the reference's path.
    let water = match &opts.water {
        Some(w) if opts.riverbank && w.open_water.as_ref().is_some_and(|o| o.len() == w.mw * w.mh) => {
            let fine = w.river_path.as_deref().filter(|p| p.len() >= 2).and_then(|p| fine_river_path(p, wm, hm));
            Some(WaterCtx { river_path: fine.or_else(|| w.river_path.clone()), ..w.clone() })
        }
        other => other.clone(),
    };
    let mut site = build_site(
        seed,
        wm,
        hm,
        &site_kind,
        SiteOpts {
            water,
            terrain: opts.terrain.clone(),
            economy: opts.economy.clone(),
        },
    );
    // **Not the reference** (2026-11-15, `crate::riverbank`): a real river is
    // measured from its own channel. After `build_site`, whose bridge point,
    // river port, `water_poly` and route ends are the reference's choices on
    // the resampled centreline. Off: the reference's stamped-mask water.
    let riverbank = opts.riverbank && site.use_riverbank();
    // **Not the reference** (Ruling FB, owner 2026-11-15: *"Big river towns
    // can grow around both banks when the terrain allows it"*): which towns
    // may. A riverbank river site of the chartered-town size class and up;
    // the organic plan only (the radial plan never calls `grow`).
    let both_banks = riverbank
        && pop_target >= BOTH_BANKS_MIN_POP
        && (site.kind == "river" || site.kind == "riverthrough")
        && profile.planning != "radial";
    let mut town_bridge: Option<TownBridge> = None;
    // v0.95: the host's real connected neighbours, not `buildSite`'s synthetic
    // map-edge endpoints. `buildPrimaries` reads only `site.routeEnds`, so this
    // is the whole of the route-locking hook.
    if !opts.route_ends.is_empty() {
        site.route_ends = opts.route_ends.clone();
    }

    let mut anchors = place_anchors(seed, &site);
    // v0.98/v1.00: with real water supplied, the town is drawn AT the box
    // centre, so pin the market there for a pixel-for-pixel overlay — nudging
    // outward ring by ring if the centre lands in the channel, since a
    // settlement sits on the bank rather than in it. Rings ascend, so the first
    // land found is the nearest.
    // **Not the reference** (Ruling FK item 1, `GenOpts::harbour_front`): a
    // harbour town's market moves from the box centre toward its water when it
    // stands inland of it ([`harbour_front_market`]); `pin` then keeps the box
    // centre, the map position, for the renderer and the host's roads. `None`
    // with the switch off: the reference, by control flow.
    let mut pin: Option<Vec2> = None;
    // The water point the market was moved toward, for `harbour_street`.
    let mut front_water: Option<Vec2> = None;
    if opts.water.is_some() {
        let mut mc = Vec2::new(wm / 2.0, hm / 2.0);
        if opts.harbour_front
            && front
            && let Some((q, water)) = harbour_front(&site, mc)
        {
            pin = Some(mc);
            mc = q;
            front_water = Some(water);
        } else if street_village
            && site.riverbank()
            && (site.kind == "river" || site.kind == "riverthrough")
            && let Some(q) = crate::village::landing_market(&site, mc)
        {
            // **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 5's
            // "no river town with every house over two widths away"; rides
            // `urban_street_village`, `crate::village::landing_market`): a
            // street village whose box centre stands inland of its river
            // takes its market down to the landing. `pin` keeps the box
            // centre, and the ring search below sets the market on dry ground
            // joined to a road, as for the harbour front.
            pin = Some(mc);
            mc = q;
        }
        // **Not the reference** (Ruling FK item 1; and a street village's
        // landing, `crate::village::landing_market`): a market the harbour
        // front or the landing moved goes, when it landed in a channel or
        // across a river from the host's roads, to the nearest dry ring point a dry line joins to one
        // of those roads (`harbour_market_reaches_a_road`), so it lands on the
        // bank the roads are on; with none such, a dry market stays and a wet
        // one takes the reference's first dry point. Off (`pin` `None`): the
        // reference's test and search, by control flow.
        let joined = |q: Vec2| pin.is_none_or(|origin| harbour_market_reaches_a_road(&site, q, origin, &opts.primary_paths));
        if site.is_water(mc) || !joined(mc) {
            let ring = |accept: &dyn Fn(Vec2) -> bool| {
                let mut best: Option<Vec2> = None;
                let max_r = js_max(wm, hm) * 0.5;
                let mut rr = 30.0;
                while rr <= max_r && best.is_none() {
                    for a in 0..24 {
                        let ang = a as f64 / 24.0 * PI * 2.0;
                        let q = Vec2::new(mc.x + js_cos(ang) * rr, mc.y + js_sin(ang) * rr);
                        if q.x < 25.0 || q.y < 25.0 || q.x > wm - 25.0 || q.y > hm - 25.0 {
                            continue;
                        }
                        if !site.is_water(q) && accept(q) {
                            best = Some(q);
                            break;
                        }
                    }
                    rr += 30.0;
                }
                best
            };
            let wet = site.is_water(mc);
            let best = if pin.is_some() { ring(&joined).or_else(|| if wet { ring(&|_| true) } else { None }) } else { ring(&|_| true) };
            if let Some(b) = best {
                mc = b;
            }
        }
        // **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 2's
        // green bar; rides `urban_street_village`,
        // `crate::village::road_bank_market`): a street village whose market
        // point stands across the river from its roads takes its market on
        // the roads' bank, at their landing. `pin` keeps the reference's
        // market (the map position, and the origin of the host's roads), so
        // the town sits on the map exactly where it did; after the landing
        // move above it is already set. Not where the harbour front moved
        // the market.
        if street_village
            && front_water.is_none()
            && site.riverbank()
            && (site.kind == "river" || site.kind == "riverthrough")
            && let Some(q) = crate::village::road_bank_market(&site, mc, pin.unwrap_or(mc), &opts.primary_paths, max_rf)
        {
            pin.get_or_insert(mc);
            mc = q;
        }
        anchors.market = mc;
    }

    // **Not the reference** (Ruling FN, `GenOpts::river_stretch`): a river
    // town the host stretched is laid out in the river-following frame
    // (`Site::market_dist`), set once its market is placed and before any
    // street. Only the organic plan on a river site with a real centreline,
    // never a street village; `None`: the reference's distances.
    if let Some(plan) = opts.river_stretch
        && !street_village
        && profile.planning != "radial"
        && site.real_river
        && (site.kind == "river" || site.kind == "riverthrough")
    {
        let f = crate::stretch::StretchFrame::new(&site.river, anchors.market, plan, max_rf);
        site.set_stretch(f);
    }

    let mut g = Graph::new();
    let mut wall_state = WallState::default();

    let harbour_opts = HarbourOpts {
        harbour_scale: opts.harbour_scale,
        harbour_defence: opts.harbour_defence.clone(),
    };

    let plaza: Option<Plaza>;
    let harbour_outcome: HarbourOutcome;

    if profile.planning == "radial" {
        // The Venus branch: concentric rings and spokes around a hub. Its
        // return value is discarded, exactly as at line 31012.
        let _rings = build_radial_streets(seed, &site, &anchors, &mut g, max_rf);
        harbour_outcome = build_harbour(seed, &site, &anchors, &mut g, Some(&harbour_opts));
        if site.through {
            add_river_bridges(seed, &site, &anchors, &mut g, 2);
        }
        if walls {
            let front = harbour_works(&harbour_outcome).map(HarbourWorks::front);
            // `wetMoat: profile.waterway` — the circular irrigation canal
            // supplies the star fort a WET ditch even on a landlocked site
            // (M-VEN-3), and line 31063 reads `fort.canalFed` back.
            let fo = FortOpts {
                wall_style: opts.wall_style.clone(),
                fortified,
                wet_moat: profile.waterway,
            };
            build_wall(
                seed,
                &site,
                &anchors,
                &g,
                &mut wall_state,
                1,
                front.as_ref(),
                &fo,
            );
        }
        // LAST on this branch, after the wall — not between the streets and the
        // growth, because there is no growth here.
        plaza = build_plaza(seed, &site, &anchors, &mut g, pop_target);
    } else {
        // v0.97: grow around the host's real roads when supplied, else
        // synthesise primaries from `routeEnds`. Both returns are discarded.
        if !opts.primary_paths.is_empty() {
            // The host's roads are offsets from the map pin: from the market in
            // the reference, from the box centre where Ruling FK's harbour
            // front moved the market off it (`pin`), so they stay on the map's.
            match pin {
                Some(origin) => {
                    build_primaries_from_paths_at(seed, &site, &anchors, &mut g, &opts.primary_paths, origin);
                }
                None => {
                    build_primaries_from_paths(seed, &site, &anchors, &mut g, &opts.primary_paths);
                }
            }
        } else {
            build_primaries(seed, &site, &anchors, &mut g);
        }
        // BEFORE `grow`: the market square's three streets are in the graph
        // before the epoch loop, so the town accretes around it.
        // A street village's green at the market point (`build_plaza_with`).
        // `TOWN_GENERATOR_JUDGEMENT.md` change 6 (`GenOpts::market_scale`):
        // the square sized from the town (organic plan only; this branch).
        plaza = crate::plaza::build_plaza_with(seed, &site, &anchors, &mut g, pop_target, street_village, opts.market_scale);
        harbour_outcome = build_harbour(seed, &site, &anchors, &mut g, Some(&harbour_opts));
        // **Not the reference** (Ruling FK item 1): a harbour-front market's
        // street down to the water, before the town grows, so it grows to the
        // water. `None` with the switch off or the market unmoved.
        if let Some(water) = front_water {
            harbour_street(&site, &mut g, anchors.market, water);
        }
        // **Not the reference** (Ruling FN, `crate::stretch::lay_river_spine`):
        // a stretched river town's road along its bank, before it grows, so
        // it grows along its river. Nothing without a stretch frame.
        crate::stretch::lay_river_spine(&site, &mut g, anchors.market);
        if site.through {
            add_river_bridges(seed, &site, &anchors, &mut g, 2);
        }
        // **Not the reference** (2026-11-15, the village cap,
        // `crate::riverbank::second_pass`): a riverbank village laid out
        // over-built the first time is grown again from a street budget
        // scaled to its target, so the lots it does not need are never
        // platted (rather than platted and emptied).
        let street_scale = match pass {
            RiverbankPass::Village { street_scale, .. } => street_scale,
            _ => 1.0,
        };
        let grow_opts = GrowOpts {
            target_len: target_len * street_scale,
            max_rf,
            walls,
            wall_generations,
            settlement_age: Some(settlement_age),
            harbour: harbour_works(&harbour_outcome).map(HarbourWorks::front),
            rules: Some(rules),
            wall_style: opts.wall_style.clone(),
            fortified,
            // Not on the object at line 31027 — `undefined`, i.e. falsy. Only
            // the radial branch sets a wet moat.
            wet_moat: false,
            pop: pop_target,
            // A street village's lanes start by the roads only.
            road_reach: street_village.then_some(crate::village::ROAD_REACH_M),
            wide_spacing: burgage,
        };
        let mut builder = FortificationBuilder;
        grow(
            seed,
            &site,
            &anchors,
            &mut g,
            epochs,
            &mut wall_state,
            &grow_opts,
            &mut builder,
        );
        // **Not the reference** (Ruling FB, `crate::riverbank`): the second
        // layout of a both-banks town that housed under its target lays a
        // bridge joined to its streets and grows the far bank from it, AFTER
        // the market's bank has grown exactly as the reference grows it (the
        // epochs above drew from the same graph the first layout's did), so
        // that bank is not shrunk to pay for the other.
        if let RiverbankPass::FarBank { share } = pass
            && both_banks
        {
            let tb = lay_town_bridge(&site, &anchors, &mut g, max_rf);
            if tb != TownBridge::None
                && let Some(far_centre) = crate::riverbank::far_bridge_end(&site, anchors.market)
            {
                crate::growth::grow_far_bank(seed, &site, &anchors, &mut g, &wall_state, &grow_opts, epochs, share, far_centre);
            }
            town_bridge = Some(tb);
        }
    }

    let harbour: Option<HarbourWorks> = match &harbour_outcome {
        HarbourOutcome::Built(h) => Some((**h).clone()),
        _ => None,
    };
    let harbour_invalid = match &harbour_outcome {
        HarbourOutcome::Invalid(why) => Some(*why),
        _ => None,
    };

    // Up to three interior lane passes, stopping the first time one adds
    // nothing (`if(!lanePass(...))break`).
    // The deep-plot prototype (`crate::burgage`) splits only larger faces;
    // `None` is the reference's 12 000 m2.
    let lane_min = burgage.then_some(crate::burgage::LANE_MIN_AREA_M2);
    for _ in 0..3 {
        if lane_pass(seed, &site, &anchors, &mut g, epochs, lane_min) == 0 {
            break;
        }
    }
    // No street may run through the channel; only bridges and the quay do.
    // A street village keeps its market's road (Ruling FE item 2,
    // `crate::cleanup::prune_keeping`); every other town prunes as the
    // reference does.
    crate::cleanup::remove_water_crossings_keeping(&site, &mut g, street_village.then_some(anchors.market));
    // **Not the reference** (2026-11-15, `crate::riverbank`): the river town's
    // embankment, after the last street pass and before the faces become
    // blocks, so the riverside blocks it closes are platted like any other.
    // **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 5, Ruling EZ
    // item 1; `crate::riverbank::lay_back_street`): a chartered river town's
    // waterfront back street and water lanes, laid first so the embankment
    // below runs along all of it and the strip between closes into waterfront
    // blocks. Organic plan only (the radial plan lays its own rings).
    let waterfront = if riverbank && pop_target >= BOTH_BANKS_MIN_POP && profile.planning != "radial" {
        Some(crate::riverbank::lay_back_street(&site, &mut g, wall_state.ring.as_deref(), plaza.as_ref().map(|p| p.poly.as_slice()), seed))
    } else {
        None
    };
    let embankment = if riverbank {
        let wf = waterfront.unwrap_or_default();
        let e = crate::riverbank::build_embankment_with(&site, &mut g, wall_state.ring.as_deref(), Some(seed), &wf.water_pts);
        Some(crate::riverbank::Embankment { back_streets: wf.streets, water_lanes: wf.lanes, ..e })
    } else {
        None
    };

    // **Not the reference** (rides `urban_street_village`,
    // `crate::village::landing_lane`, 2026-11-17): a street village whose
    // market the landing or its roads' bank moved (`pin` set, not by the
    // harbour front) gets a lane from the market down to its river, so
    // its tofts reach the water. Nothing for a town, a harbour front or
    // a market already within two street widths of the bank. Laid after the
    // growth and the last street pass, so the village's street budget and
    // every draw of `grow` stay as they were, and before the faces become
    // blocks, so the blocks and tofts see it.
    if street_village
        && pin.is_some()
        && front_water.is_none()
        && site.riverbank()
        && (site.kind == "river" || site.kind == "riverthrough")
    {
        crate::village::landing_lane(&site, &mut g, anchors.market);
    }
    let blocks = build_blocks(&g, plaza.as_ref(), &site);
    let mut parcels = crate::blocks::build_parcels_with(seed, &g, &blocks, anchors.market, epochs, &site, Some(&rules), burgage);
    // **Not the reference** (Ruling FE item 2, `crate::village::plat_tofts`): a
    // street village's roads are lined with tofts up to its target, after the
    // block lots (which they never overlap) and before the wall hug, so a
    // walled village's wall is fitted round them too.
    if street_village {
        let tofts = crate::village::plat_tofts(seed, &g, &site, &blocks, &parcels, anchors.market, max_rf, pop_target, epochs, terrain_aware);
        parcels.extend(tofts);
    }
    // **Not the reference** (Ruling EW item 1, `crate::wallhug`): the wall is
    // refitted to the platted town, so no wide empty band stands between it and
    // the first buildings. After the street lots exist (the mass it fits) and
    // before the wall lots, courtyards, districts, buildings, farmland, details
    // and the citadel, so all of them are made from the circuit that stays.
    // Off (`false`) is the reference's growth-reserve circuit untouched.
    let mut hug_outcome = None;
    if opts.wall_hug && walls && wall_state.ring.is_some() {
        let front = harbour_works(&harbour_outcome).map(HarbourWorks::front);
        let fo = FortOpts {
            wall_style: opts.wall_style.clone(),
            fortified,
            wet_moat: profile.planning == "radial" && profile.waterway,
        };
        hug_outcome = Some(hug_wall(seed, &site, &anchors, &mut g, &parcels, terrain_aware, &mut wall_state, front.as_ref(), &fo, both_banks, plaza.as_ref().filter(|p| p.kind == crate::plaza::PlazaKind::Market).map(|p| p.poly.as_slice())));
        // **Not the reference** (2026-11-17, Ruling FK's left item (iii);
        // rides `urban_wall_hug`, `crate::wallhug::shore_snap`): the stretches
        // of the fitted wall that stand over open water are set on the shore.
        // Not a bastioned trace (its ring is the gorge, not the drawn wall).
        crate::wallhug::snap_wall_to_shore(&mut wall_state, &site);
    }
    // **Not the reference** — a Ruling H departure (owner, 2026-09-22): lots
    // backing onto the wall from inside, and a faubourg against it outside.
    // Appended after every street-platted lot and drawn from their own
    // substreams, so no earlier lot moves; see `crate::wallside` for why this
    // is a plat of the unplatted band rather than a bias on existing blocks.
    // Both planning branches (Ruling AA, owner 2026-09-23, extended this to the
    // radial plan: its rings and spokes are the wedge blocks the owner's town
    // plan wants, and the wall lots and faubourg are what it lacked). Nothing
    // here reads the street graph's shape — only live edges, the ring and the
    // gates — so the stage is unchanged; see `crate::wallside`.
    let wall_lots =
        build_wall_lots(seed, &g, &wall_state, &site, &blocks, &parcels, pop_target, epochs);
    parcels.extend(wall_lots);
    // **Not the reference** (Ruling H, sited by Ruling AD, `crate::courtyard`):
    // the outermost ring's dense blocks are re-platted as perimeter blocks
    // round an open court. After the wall lots, so they plat against exactly
    // the street lots they always did; organic plan only, so the Venus
    // warehouse belt is untouched. A block that is not converted keeps its
    // strip lots, ids and draws.
    // Not for a street village (Ruling FE item 2: *"not the towns' courtyard
    // blocks of Rulings H and AD"*): its few blocks keep their strip lots.
    if profile.planning != "radial" && !street_village {
        build_courtyard_rings(seed, &g, &blocks, &mut parcels, anchors.market, epochs, &site);
    }

    let quay = |h: &Option<HarbourWorks>| h.as_ref().map(|w| w.quay.clone());
    let quay_pts = quay(&harbour);

    let mut lots: Vec<Lot<'_>> = assign_districts(
        &site,
        &anchors,
        plaza.as_ref(),
        &wall_state,
        &parcels,
        max_rf,
        quay_pts.as_deref(),
        opts.ore_bearing,
    );
    let mut buildings = build_buildings(
        seed,
        &mut lots,
        plaza.as_ref(),
        &anchors,
        Some(&profile),
        terrain_aware,
    );
    // **Not the reference** (2026-11-15, Ruling FB's "villages must not be
    // over-built"; `crate::riverbank::trim_overbuilt`): on a riverbank site a
    // village (under `BOTH_BANKS_MIN_POP`) housing more than
    // `RIVER_HOUSING_CAP` times its target leaves its outermost inland lots
    // vacant down to it, those outside its wall first. Only in the layout
    // `crate::riverbank::fit_village` chose when no street scale it tried
    // brought the village inside the cap (`trim: true`), so this empties only
    // the remainder; every other layout's count is untrimmed, which is what
    // `second_pass` and `fit_village` judge. Before the places of
    // worship, which claim lots of their own.
    if (riverbank || street_village) && pop_target < BOTH_BANKS_MIN_POP && matches!(pass, RiverbankPass::Village { trim: true, .. }) {
        crate::riverbank::trim_overbuilt(&mut lots, &mut buildings, anchors.market, pop_target, wall_state.ring.as_deref());
    }

    // `p.ruined` / `b.ruined`. Both are absent (falsy) with the toggle off,
    // which is what these all-false vectors are.
    let mut parcel_ruined = vec![false; lots.len()];
    let mut building_ruined = vec![false; buildings.len()];
    if ruined {
        let decay = apply_decay(seed, &lots, &buildings);
        for &i in &decay.ruined_parcels {
            parcel_ruined[i] = true;
        }
        for &i in &decay.ruined_buildings {
            building_ruined[i] = true;
        }
    }

    let faith = or_str(opts.faith.as_deref(), profile.default_faith).to_string();
    // A hamlet has no church or chapel, and a rite can be turned off outright.
    let hamlet = pop_target < 600.0;
    let n_church = if faith == "none" || hamlet {
        0.0
    } else {
        js_max(1.0, js_min(4.0, js_round((pop_target / 5.2) / 500.0))) // M-DEN-8
    };
    let churches = if n_church > 0.0 {
        build_faith_sites(
            seed,
            &mut lots,
            &mut buildings,
            &anchors,
            n_church as usize,
            &faith,
            &site,
            quay_pts.as_deref(),
        )
    } else {
        Vec::new()
    };
    // `buildFaithSites` can push buildings; keep the ruin flags index-aligned.
    building_ruined.resize(buildings.len(), false);

    // M-DEN-2: one household of 5.2 per built parcel. **Accumulated, not
    // multiplied** — `reduce((s,p)=>s+5.2,0)` sums in parcel order and the
    // rounding of that sum is not `5.2 * n`.
    let mut pop = 0.0f64;
    for (i, l) in lots.iter().enumerate() {
        if l.built && !l.churchyard && !parcel_ruined[i] {
            pop += 5.2;
        }
    }
    let pop = js_round(pop);

    // `par.cleared`, written by the two sweeps below and read by neither.
    let mut parcel_cleared = vec![false; lots.len()];

    // Specialised markets multiply with rank (M-AMEN-1). `profile.markets` is a
    // data-driven hook for a commerce that did not run through a chartered
    // square; both live profiles set it, so this always runs today.
    let markets: Vec<Market> = if profile.markets {
        let parcel_centroids: Vec<Vec2> =
            lots.iter().map(|l| poly_centroid(&l.par.poly)).collect();
        let building_centroids: Vec<Vec2> =
            buildings.iter().map(|b| poly_centroid(&b.poly)).collect();
        // **Not the reference** (2026-11-15, riverbank sites only,
        // `crate::amenities::build_markets_within`): a walled riverbank
        // town's specialised markets stand inside its wall.
        let ring = wall_state.ring.clone().filter(|r| riverbank && r.len() >= 3);
        let inside = |p: Vec2| {
            ring.as_deref().is_none_or(|r| {
                crate::geom::point_in_poly(p, r)
                    && (0..r.len()).all(|k| crate::geom::dist_pt_seg(p, r[k], r[(k + 1) % r.len()]) >= crate::amenities::MARKET_WALL_CLEAR_M)
            })
        };
        let m = build_markets_within(
            seed,
            &site,
            &anchors,
            &g,
            plaza.as_ref(),
            pop_target,
            &parcel_centroids,
            &building_centroids,
            ring.is_some().then_some(&inside as &dyn Fn(Vec2) -> bool),
        );
        for &i in &m.cleared_parcels {
            parcel_cleared[i] = true;
        }
        // Reported ascending; the reference splices descending, so remove in
        // reverse to keep every later index valid.
        for &i in m.removed_buildings.iter().rev() {
            buildings.remove(i);
            building_ruined.remove(i);
        }
        m.markets
    } else {
        Vec::new()
    };

    let civic = build_civic(
        seed,
        plaza.as_ref(),
        pop_target,
        or_str(opts.civic_style.as_deref(), profile.default_civic),
        &faith,
    );
    let games = {
        let parcel_polys: Vec<&[Vec2]> = lots.iter().map(|l| l.par.poly.as_slice()).collect();
        build_games(
            seed,
            &site,
            &anchors,
            &g,
            &parcel_polys,
            wall_state.ring.as_deref(),
            pop_target,
            &profile,
            plaza.as_ref(),
            civic.as_ref(),
        )
    };

    let mut details = build_details(
        seed,
        &site,
        &anchors,
        &g,
        &blocks,
        &lots,
        plaza.as_ref(),
        pop,
        harbour.as_ref(),
        &profile,
    );
    details.extend(build_farmland(
        seed,
        &site,
        &anchors,
        &g,
        &wall_state,
        max_rf,
        &profile,
    ));
    // When the star fort is canal-fed (M-VEN-3) its own wet-moat rendering
    // already carries the canal, so the decorative ring would duplicate it.
    if profile.waterway && !wall_state.fort.as_ref().is_some_and(|f| f.canal_fed) {
        details.extend(build_waterway(seed, &site, &anchors, max_rf * 0.95).into_iter().map(
            |w| Detail {
                // The reference's canal record is `{kind, poly, prov}` — no id.
                id: String::new(),
                kind: w.kind,
                geom: DetailGeom::Poly(w.poly),
                rr: None,
                orchard: false,
                prov: w.prov,
            },
        ));
    }

    // M-ISL-2. Runs AFTER blocks/parcels/buildings, so privatising a lane closes
    // through access only — it does not demolish the dwellings on it.
    privatize_alleys(seed, &profile, &mut g, Some(&rules));

    // Sweep the fortification's field of fire (esplanade/glacis).
    if wall_state.ring.is_some() {
        let building_polys: Vec<Vec<Vec2>> = buildings.iter().map(|b| b.poly.clone()).collect();
        let parcel_polys: Vec<Vec<Vec2>> = lots.iter().map(|l| l.par.poly.clone()).collect();
        let detail_pts: Vec<Option<Vec2>> = details.iter().map(Detail::anchor).collect();
        let sweep = clear_fort_zone(&wall_state, &mut g, &building_polys, &parcel_polys, &detail_pts);
        // **Not the reference** (Ruling H, `crate::wallside`): a faubourg is a
        // cluster built against the wall's outer face, so its first rows lie
        // inside the curtain's 15 m rampart strip on purpose — so every one of
        // its lots, in any row, and their buildings are
        // exempt from the sweep. Keyed on the lot, not on the district string,
        // because a faubourg lot can since have been retagged (a churchyard,
        // an ore yard). A bastioned trace never gets a faubourg at all, so its
        // glacis is swept exactly as before.
        let faubourg: std::collections::HashSet<&str> = lots
            .iter()
            .filter(|l| l.par.wall_backing.is_faubourg())
            .map(|l| l.par.id.as_str())
            .collect();
        // Both removal lists are already descending — the reference splices
        // while walking backwards — so they apply in the order they arrive.
        for &i in &sweep.buildings_removed {
            if faubourg.contains(buildings[i].parcel.as_str()) {
                continue;
            }
            buildings.remove(i);
            building_ruined.remove(i);
        }
        for &i in &sweep.parcels_cleared {
            if lots[i].par.wall_backing.is_faubourg() {
                continue;
            }
            parcel_cleared[i] = true;
        }
        for &i in &sweep.details_removed {
            details.remove(i);
        }
    }

    // **Not the reference** (Ruling I / Ruling AC, `crate::citadel`): the
    // largest organic towns get a citadel astride the curtain. After every lot,
    // building and detail exists and after the rampart sweep, and drawn from its
    // own substream, so a town under the tier is untouched byte for byte; before
    // the crossing and metrics passes, because it kills and adds streets.
    let citadel = if profile.planning != "radial" && pop_target >= CITADEL_MIN_POP {
        build_citadel(seed, &site, &wall_state, &mut g)
    } else {
        None
    };
    if let Some(c) = &citadel {
        let building_polys: Vec<Vec<Vec2>> = buildings.iter().map(|b| b.poly.clone()).collect();
        let parcel_polys: Vec<Vec<Vec2>> = lots.iter().map(|l| l.par.poly.clone()).collect();
        let detail_pts: Vec<Option<Vec2>> = details.iter().map(Detail::anchor).collect();
        let sweep = citadel_sweep(c, &building_polys, &parcel_polys, &detail_pts);
        // Both removal lists are descending.
        for &i in &sweep.buildings_removed {
            buildings.remove(i);
            building_ruined.remove(i);
        }
        for &i in &sweep.parcels_cleared {
            parcel_cleared[i] = true;
        }
        for &i in &sweep.details_removed {
            details.remove(i);
        }
    }

    // **Not the reference** (2026-11-15, `crate::riverbank`): on a riverbank
    // site, a house that reaches into the water anyway (the lot test samples
    // nine points; a corner of a 22 m sea or lake cell can fall between them)
    // is swept and its lot marked cleared, as the rampart and citadel sweeps
    // do. Last, so every house any stage made is checked.
    let mut wet_swept = 0usize;
    if riverbank {
        let wet: Vec<usize> = (0..buildings.len()).rev().filter(|&i| site.poly_in_water(&buildings[i].poly)).collect();
        for i in wet {
            if let Some(k) = lots.iter().position(|l| l.par.id == buildings[i].parcel) {
                parcel_cleared[k] = true;
            }
            buildings.remove(i);
            building_ruined.remove(i);
            wet_swept += 1;
        }
    }
    // The farmland strips likewise: `build_farmland` tests three points along a
    // 70-140 m strip for water, which the band stamped into the mask used to
    // catch and a 12-46 m channel does not (seed 483920 at 384 x 288,
    // settlement #36: strips drawn across the river). Fields and pasture only;
    // a pier, crane or log boom belongs on the water.
    let mut fields_swept = 0usize;
    if riverbank {
        let before = details.len();
        details.retain(|d| {
            !(matches!(d.kind, "field" | "pasture") && matches!(&d.geom, DetailGeom::Poly(p) if site.poly_in_water(p)))
        });
        fields_swept = before - details.len();
    }
    let embankment = embankment.map(|e| crate::riverbank::Embankment { wet_swept, fields_swept, ..e });

    // v1.17 (S5). On the FINAL graph — after `removeWaterCrossings`,
    // `privatizeAlleys` AND `clearFortZone`, the last three passes that can kill
    // an edge — so a recorded bridge always has a live road on it.
    let crossings = detect_river_crossings(&site, &g);
    let metrics = compute_metrics(&g, &blocks, &parcels);

    let out_parcels: Vec<TownParcel> = lots
        .iter()
        .enumerate()
        .map(|(i, l)| TownParcel {
            par: l.par.clone(),
            district: l.district,
            prov_district: l.prov_district,
            suitability: l.suitability,
            empty: l.empty,
            unsuitable: l.unsuitable,
            built: l.built,
            churchyard: l.churchyard,
            cleared: parcel_cleared[i],
            ruined: parcel_ruined[i],
        })
        .collect();
    drop(lots);

    let (bridges, ford) = match crossings {
        Crossings::Bridges(b) => (Some(b), None),
        Crossings::Ford(f) => (None, Some(f)),
        Crossings::None => (None, None),
    };
    // `Town::pin`: the market, unless the harbour front or a street
    // village's landing or road-bank move (`crate::village`) moved it.
    let pin = pin.unwrap_or(anchors.market);

    Town {
        seed,
        epochs,
        pop_target,
        walls,
        fortified,
        fort_requested,
        fort_min: FORT_MIN,
        terrain_aware,
        ruined,
        wall_generations,
        settlement_age,
        target_len,
        max_rf,
        wm,
        hm,
        culture: profile.id,
        culture_name: profile.name,
        site: TownSite {
            kind: site.kind.clone(),
            through: site.through,
            no_water: site.no_water,
            river: site.river.clone(),
            river_w: site.river_w,
            water_poly: site.water_poly.clone(),
            bridge_pt: site.bridge_pt,
            bridge_dir: site.bridge_dir,
            route_ends: site.route_ends.clone(),
            bridges,
            ford,
            harbour_invalid,
        },
        anchors,
        plaza,
        harbour,
        markets,
        civic,
        games,
        graph: TownGraph {
            nodes: g.nodes.clone(),
            edges: g.edges.iter().filter(|e| e.alive).cloned().collect(),
        },
        wall: wall_state,
        blocks,
        parcels: out_parcels,
        buildings,
        building_ruined,
        churches,
        details,
        citadel,
        wall_hug: hug_outcome,
        riverbank,
        town_bridge,
        both_banks,
        riverbank_pass: (riverbank || street_village).then_some(pass),
        street_village,
        embankment,
        pin,
        pop,
        metrics,
        through: site.through,
    }
}

/// The works inside a [`HarbourOutcome`], or [`None`] for either refusal — the
/// reference's `harbour` variable, which is `null` in both cases.
fn harbour_works(o: &HarbourOutcome) -> Option<&HarbourWorks> {
    match o {
        HarbourOutcome::Built(h) => Some(h),
        _ => None,
    }
}

// ----------------------------------------------------------------- hashModel --

/// `String(n)` for the **integral** doubles `Math.round` produces.
///
/// The domain is the reason this is three lines rather than a JS number
/// formatter: every value hashed is `Math.round` of a coordinate (≤ 1 700 m),
/// a width (metres), or an area inside a 1 700 × 1 250 m box, times at most
/// 100 — so the largest possible magnitude is ~2·10⁸ and `1e21`'s exponential
/// form is unreachable. What *is* reachable is a `NaN` coordinate, which
/// `js_round` passes through and JS stringifies as `"NaN"`, and `-0`, which JS
/// prints as `"0"` where Rust's own `Display` prints `"-0"`.
fn js_int_str(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    debug_assert!(v.abs() < 9.007_199_254_740_992e15, "outside String(n)'s integer form");
    if v == 0.0 {
        // Covers -0.0, which JS prints as "0".
        return "0".to_string();
    }
    format!("{}", v as i64)
}

/// `hashModel(m)` — reference line 31087.
///
/// Stable serialisation plus FNV-1a, the reference's own determinism golden.
/// Five loops, in this order, joined with `'|'`:
///
/// | loop | pushes |
/// |---|---|
/// | live edges | `a`, `b`, `cls`, `round(w*10)` |
/// | **all** nodes | `round(x*100)`, `round(y*100)` |
/// | blocks | `id`, `round(area)` |
/// | parcels | `id`, `round(area*10)`, `district \|\| ''` |
/// | buildings | `id`, `kind`, `round(polyArea(poly)*10)` |
///
/// Coordinates are rounded to the **centimetre** and areas to a **tenth of a
/// square metre**, so this is a coarser instrument than a field-by-field
/// comparison — it will not see a divergence below half a step. It is
/// nonetheless the only whole-model comparator the reference ships, and it is
/// the one `generate()` was written to be checked with.
pub fn hash_model(m: &Town) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for e in &m.graph.edges {
        parts.push(e.a.to_string());
        parts.push(e.b.to_string());
        parts.push(e.cls.to_string());
        parts.push(js_int_str(js_round(e.w * 10.0)));
    }
    for n in &m.graph.nodes {
        parts.push(js_int_str(js_round(n.x * 100.0)));
        parts.push(js_int_str(js_round(n.y * 100.0)));
    }
    for b in &m.blocks {
        parts.push(b.id.clone());
        parts.push(js_int_str(js_round(b.area)));
    }
    for p in &m.parcels {
        parts.push(p.par.id.clone());
        parts.push(js_int_str(js_round(p.par.area * 10.0)));
        parts.push(p.district.to_string());
    }
    for b in &m.buildings {
        parts.push(b.id.clone());
        parts.push(b.kind.to_string());
        parts.push(js_int_str(js_round(poly_area(&b.poly) * 10.0)));
    }
    fnv1a(&parts.join("|"))
}
