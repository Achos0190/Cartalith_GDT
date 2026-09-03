//! Milestone 16 — `generate()` and `hashModel` (reference lines 30931-31094).
//!
//! The orchestration: 29 stage calls in the reference's own order, over the
//! options every earlier milestone took a projection of. Nothing new is
//! computed here — every line is a call into a module that is already
//! golden-verified on its own — so what this module is *for* is the order, the
//! branch conditions and the small amount of arithmetic `generate()` does
//! between stages (the population clamp, the church count, the head count).
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
//! | [`build_markets`] | `cleared_parcels`, `removed_buildings` | flags + `Vec::remove` in reverse |
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

use crate::amenities::{Civic, GamesBuilding, Market, build_civic, build_games, build_markets};
use crate::blocks::{Block, Parcel, build_blocks, build_parcels};
use crate::cleanup::{clear_fort_zone, lane_pass, privatize_alleys, remove_water_crossings};
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
use crate::rng::fnv1a;
use crate::routes::{Anchors, build_primaries, build_primaries_from_paths, place_anchors};
use crate::rules::{CultureProfile, RulesPatch, resolve_profile, resolve_rules};
use crate::site::{Economy, SiteOpts, TerrainCtx, WaterCtx, build_site};
use crate::water::{
    Bridge, Crossings, Ford, HarbourOpts, HarbourOutcome, HarbourWorks, add_river_bridges,
    build_harbour, detect_river_crossings,
};
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

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
    /// `par.cleared` — set by [`build_markets`] and [`clear_fort_zone`]. Written
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
#[allow(clippy::too_many_lines)]
pub fn generate(seed: u32, opts: &GenOpts) -> Town {
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
    let max_rf = js_min(720.0, (pop_target * 21.0).sqrt() * 1.35 + 80.0);
    let (wm, hm) = (SITE_WM, SITE_HM);
    let site_kind = or_str(opts.site.as_deref(), "river").to_string();

    let mut site = build_site(
        seed,
        wm,
        hm,
        &site_kind,
        SiteOpts {
            water: opts.water.clone(),
            terrain: opts.terrain.clone(),
            economy: opts.economy.clone(),
        },
    );
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
    if opts.water.is_some() {
        let mut mc = Vec2::new(wm / 2.0, hm / 2.0);
        if site.is_water(mc) {
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
                    if !site.is_water(q) {
                        best = Some(q);
                        break;
                    }
                }
                rr += 30.0;
            }
            if let Some(b) = best {
                mc = b;
            }
        }
        anchors.market = mc;
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
        plaza = build_plaza(seed, &site, &anchors, &mut g);
    } else {
        // v0.97: grow around the host's real roads when supplied, else
        // synthesise primaries from `routeEnds`. Both returns are discarded.
        if !opts.primary_paths.is_empty() {
            build_primaries_from_paths(seed, &site, &anchors, &mut g, &opts.primary_paths);
        } else {
            build_primaries(seed, &site, &anchors, &mut g);
        }
        // BEFORE `grow`: the market square's three streets are in the graph
        // before the epoch loop, so the town accretes around it.
        plaza = build_plaza(seed, &site, &anchors, &mut g);
        harbour_outcome = build_harbour(seed, &site, &anchors, &mut g, Some(&harbour_opts));
        if site.through {
            add_river_bridges(seed, &site, &anchors, &mut g, 2);
        }
        let grow_opts = GrowOpts {
            target_len,
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
    for _ in 0..3 {
        if lane_pass(seed, &site, &anchors, &mut g, epochs, None) == 0 {
            break;
        }
    }
    // No street may run through the channel; only bridges and the quay do.
    remove_water_crossings(&site, &mut g);

    let blocks = build_blocks(&g, plaza.as_ref(), &site);
    let parcels = build_parcels(seed, &g, &blocks, anchors.market, epochs, &site, Some(&rules));

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
        let m = build_markets(
            seed,
            &site,
            &anchors,
            &g,
            plaza.as_ref(),
            pop_target,
            &parcel_centroids,
            &building_centroids,
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
        // Both removal lists are already descending — the reference splices
        // while walking backwards — so they apply in the order they arrive.
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
