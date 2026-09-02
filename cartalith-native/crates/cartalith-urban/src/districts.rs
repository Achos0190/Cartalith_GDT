//! Milestone 13 — districts and buildings (reference lines **30345-30682**,
//! seven functions).
//!
//! The stage that turns platted land into a *town*: [`assign_districts`] tags
//! every parcel with what happens on it, [`build_buildings`] runs the
//! parcel-conditioned building grammar that puts footprints and roof ridges on
//! it, and [`build_faith_sites`] plants the anchor institutions — a church,
//! temple, shrine, mosque or cross-in-square — clearing whatever houses were
//! already standing on the plots it claims. [`bmap`], [`rect_poly`],
//! [`rect_pts`] and [`peristyle`] are the four primitives those three share.
//!
//! # The line range in `URBAN_MORPHOLOGY_SCOPE.md` is wrong at the end
//!
//! The scope document says 30345-30710. **The real range is 30345-30682.**
//! 30683 is blank, 30684 is milestone 15's `details:` header comment, and
//! 30696-30701 are milestone 15's `FARM_SPEC` table — all three inside the
//! claimed range and none of them this milestone's. The *start* is right: 30345
//! is the `function assignDistricts` line and 30344 is the district header
//! comment. Milestone 15's own claimed start of 30711 is exactly
//! `function crossesStreet`, so nothing is orphaned between the two — the
//! overlap is the only error. Fifth consecutive urban milestone to find its
//! range wrong, which is by now the expectation rather than the exception.
//!
//! # Two fields this port's `Parcel` does not have, and where they went
//!
//! The reference mutates its parcel objects in place: `assignDistricts` writes
//! `district`, `provDistrict` and `suitability` onto them, `buildBuildings`
//! writes `empty`, `unsuitable` and `built`, and `buildFaithSites` writes
//! `churchyard` and overwrites `district`/`provDistrict`. [`Parcel`] is
//! milestone 12's and carries none of them — so this module carries them
//! instead, on [`Lot`], which borrows the parcel and holds the seven mutable
//! fields beside it. [`assign_districts`] is what produces the [`Lot`] list,
//! exactly as it is what first assigns a district in the reference; the other
//! two take `&mut [Lot]`.
//!
//! That is a shape change, not a behaviour change, and it is deliberate:
//! adding seven fields to [`Parcel`] would put milestone 13's state on
//! milestone 12's struct, where every earlier consumer would have to ignore it.
//!
//! # `opts.economy` is the honest gap, and it is stated rather than papered over
//!
//! `assignDistricts`' second half re-tags a bounded set of parcels from the
//! settlement's *function* — `site.economy.specialisation`, one of `mining`,
//! `fishing`, `timber`, `grain`, `trade_hub`, `pastoral` — producing the
//! `oreyard`/`fishery`/`sawyard`/`granary`/`warehouse` districts and, through
//! [`build_buildings`], the working-yard shed grammar.
//!
//! **Nothing in this port can reach that path yet.** `cartalith_civ`'s
//! settlements carry no `specialisation` (`urban_adapter.rs` says so in its own
//! header and passes `economy: None` unconditionally), so
//! [`Site::economy`](crate::site::Site::economy) is always [`None`] and the
//! whole block is skipped — which is precisely what the reference does on a
//! world where nobody set a specialisation. The path is ported in full and
//! golden-tested against the reference by injecting an economy directly into
//! the fixtures; it simply has no live caller. `URBAN_MORPHOLOGY_SCOPE.md`
//! milestone 17 predicted this gap and named `economy: null` as the honest
//! fallback. This is that fallback, with the code behind it already written.
//!
//! **`site::Economy::ore_bearing` is the wrong type for what the reference
//! stores there.** The reference's `oreBearing` is a *nullable angle in
//! radians* — `_umOreBearing` (reference line 22613) returns
//! `Math.atan2(by,bx) - orient` or `null` — and the ore-yard rule scores
//! parcels by their projection onto that bearing so the yard lands on the side
//! of town the ore actually comes from. Milestone 5's `Economy` declares it
//! `bool`, which cannot carry an angle. Rather than edit another milestone's
//! module or silently drop the arm, [`assign_districts`] takes the bearing as
//! its own parameter; when `Economy::ore_bearing` is corrected to
//! `Option<f64>` that parameter should be deleted and the field read directly.
//! Both arms — bearing and no-bearing — are golden-tested here today.
//!
//! # What is not modelled, because milestone 9 has not landed
//!
//! `assignDistricts` and `buildFaithSites` both take `buildHarbour`'s return
//! object and read exactly one field of it, `quay`. Milestone 9 owns that
//! function, so both take `Option<&[Vec2]>` — the quay polyline — the same call
//! [`crate::growth::HarbourFront`] made for `grow`. The reference tests the
//! *object* for truthiness and then indexes `.quay`, so a harbour whose quay is
//! empty is still a harbour: [`crate::growth::dist_to_line`] over fewer than
//! two points returns infinity and every distance test fails, which is what
//! `Some(&[])` reproduces.

use crate::blocks::{Parcel, Plaza};
use crate::geom::{
    Vec2, dist_pt_seg, js_cos, js_hypot, js_max, js_min, js_num_cmp, js_round, js_sin,
    point_in_poly, poly_area, poly_centroid,
};
use crate::growth::{WallState, dist_to_line};
use crate::rng::{Substream, fnv1a, stream};
use crate::routes::Anchors;
use crate::rules::{CultureProfile, MEDIEVAL};
use crate::site::{Site, terrain_suitability};
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

// ------------------------------------------------------------------ lots ----

/// A parcel plus the milestone-13 state the reference writes onto it.
///
/// The seven mutable fields are the reference's own, and their initial values
/// are what a freshly-platted parcel object has: JS leaves `district`,
/// `provDistrict` and `suitability` absent until `assignDistricts` runs, and
/// `empty`/`unsuitable`/`built`/`churchyard` absent until a later stage sets
/// them. Absent is falsy, so the three booleans start `false`; `district` and
/// `prov_district` start `""`, which no `=== '...'` test in the subsystem
/// matches, and `suitability` starts `f64::NAN`, which no `<` test matches
/// either — the same "reads as nothing" that `undefined` gives.
#[derive(Debug, Clone, PartialEq)]
pub struct Lot<'a> {
    pub par: &'a Parcel,
    /// One of `market`, `burgher`, `artisan`, `craftriver`, `suburb`,
    /// `agrarian`, `harbour`, `oreyard`, `fishery`, `sawyard`, `granary`,
    /// `warehouse`, `church` — or `""` before [`assign_districts`] has run.
    ///
    /// A string rather than an enum, for the same reason `Edge::cls`,
    /// `Parcel::edge_cls` and `Site::kind` are strings: the vocabulary is not
    /// closed. The economy half keys off `specialisation`, which is host data.
    pub district: &'static str,
    pub prov_district: &'static str,
    /// `terrainSuitability(site, centroid)` — informational unless
    /// `terrainAware` is on, where it gates the parcel (docs/08, M-TER-1/2).
    pub suitability: f64,
    /// No building was placed: a paddock, a plot too small to build on, ground
    /// too steep or too flood-prone, or a working yard's open ground.
    pub empty: bool,
    /// `empty` *because* of the terrain gate specifically.
    pub unsuitable: bool,
    /// [`build_buildings`] reached the end of this parcel — set even when every
    /// footprint it computed was rejected as too small.
    pub built: bool,
    /// Claimed by a place of worship.
    pub churchyard: bool,
}

impl<'a> Lot<'a> {
    /// A parcel as it reaches `assignDistricts`: every milestone-13 field
    /// unset.
    pub fn new(par: &'a Parcel) -> Self {
        Lot {
            par,
            district: "",
            prov_district: "",
            suitability: f64::NAN,
            empty: false,
            unsuitable: false,
            built: false,
            churchyard: false,
        }
    }
}

// ------------------------------------------------------------ provenance ----

const PROV_HARBOUR: &str = "Harbour quarter: warehouses and merchant yards on the quay — goods land, are stored and are sold from here (harbour-city family, lit. review §1.1 #22).";
const PROV_MARKET: &str = "Market district: highest network access; commerce clusters on the most integrated frontages (M-NET-10).";
const PROV_BURGHER: &str = "Burgher district: prime intramural land near the market; dense, tall frontages (M-DEN-1/3).";
const PROV_ARTISAN: &str =
    "Artisan district: intramural but peripheral; workshops and yards (M-DEN-5).";
const PROV_CRAFTRIVER: &str = "Riverside craft district: water-dependent and noxious trades (tanning, dyeing) pushed to the bank (M-DEN-5).";
const PROV_SUBURB: &str =
    "Suburb: extramural ribbon along the approach roads (Strano exploration phase, M-GRW-1).";
const PROV_AGRARIAN: &str =
    "Agrarian fringe: smallholdings, orchards and paddocks at the walking limit (M-REG-4).";

const PROV_OREYARD: &str = "Ore yard: dressing floors and spoil ground of a mining settlement — the workings lie out in the hinterland, the processing at the town edge facing them (S6 economy rule).";
const PROV_FISHERY: &str = "Fishery yard: net lofts and drying racks of a fishing settlement strung along the waterfront (S6 economy rule).";
const PROV_SAWYARD: &str = "Saw yard: log landing and saw pits of a timber settlement, at the water/edge where the logs arrive (S6 economy rule).";
const PROV_GRANARY: &str = "Granary and mill plots of a grain settlement, hard by the market where the crop is weighed and sold (S6 economy rule).";
const PROV_WAREHOUSE: &str =
    "Warehouse row of a trading settlement along the through-road (S6 economy rule).";
const PROV_PASTORAL_AGRARIAN: &str = "Agrarian fringe: paddocks pushed close in — a pastoral settlement folds its stock into the town edge (S6 economy rule).";

const PROV_CHURCHYARD: &str = "Sacred precinct: the place of worship on its own plot; earlier houses cleared (M-DEN-8).";

/// `ecoProv[d]` — the reference's own lookup, which is `undefined` for any key
/// outside the five it defines. Only ever indexed with one of those five.
fn eco_prov(d: &str) -> &'static str {
    match d {
        "oreyard" => PROV_OREYARD,
        "fishery" => PROV_FISHERY,
        "sawyard" => PROV_SAWYARD,
        "granary" => PROV_GRANARY,
        "warehouse" => PROV_WAREHOUSE,
        _ => "",
    }
}

// ------------------------------------------------------- assignDistricts ----

/// `assignDistricts` (line 30345) — the district of every parcel, from access
/// and anchors first (M-NET-10, M-DEN-5) and then from the settlement's
/// economic function.
///
/// Returns the [`Lot`] list the rest of this milestone works on; the reference
/// mutates its parcel objects instead and returns nothing.
///
/// `ore_bearing` is `site.economy.oreBearing`, which this port's
/// [`crate::site::Economy`] cannot carry — see the module header. Pass [`None`]
/// unless you have a real bearing in radians.
///
/// # NaN
///
/// The economy candidate lists are sorted with the reference's own comparator,
/// `a.s - b.s`, encoded rather than replaced: a difference that is neither
/// `< 0` nor `> 0` — which includes every comparison against a NaN score —
/// compares [`Ordering::Equal`], the same "no swap" V8's sort takes from a NaN
/// comparator result. Both sorts are stable, so equal scores keep parcel order.
/// With NaN scores actually present the *permutation* V8's TimSort produces is
/// implementation-defined and this port cannot promise to match it; the
/// comparator is what is ported. No scenario in this subsystem reaches it: a
/// NaN score needs a NaN centroid, and `build_parcels` rejects those long
/// before here.
#[allow(clippy::too_many_arguments)]
pub fn assign_districts<'a>(
    site: &Site,
    anchors: &Anchors,
    plaza: Option<&Plaza>,
    wall_state: &WallState,
    parcels: &'a [Parcel],
    max_rf: f64,
    harbour: Option<&[Vec2]>,
    ore_bearing: Option<f64>,
) -> Vec<Lot<'a>> {
    // without a wall the intramural/extramural split falls back to the
    // urban-core radius
    let in_wall = |p: Vec2| match &wall_state.ring {
        Some(ring) => point_in_poly(p, ring),
        None => p.dist(anchors.market) < max_rf * 0.72,
    };

    let mut lots: Vec<Lot<'a>> = parcels.iter().map(Lot::new).collect();

    for lot in &mut lots {
        let c = poly_centroid(&lot.par.poly);
        let d_m = c.dist(anchors.market);
        let rd = site.river_dist(c);
        // docs/08: informational by default (M-TER-1)
        lot.suitability = terrain_suitability(site, c);
        // `plaza && distPtSeg(c, plaza.poly[0], plaza.poly[1]) < depth+22`: a
        // plaza with fewer than two points would index `undefined` in JS and
        // make the comparison NaN-false, which is what the length guard says.
        let on_plaza_front = match plaza {
            Some(p) if p.poly.len() >= 2 => {
                dist_pt_seg(c, p.poly[0], p.poly[1]) < lot.par.depth + 22.0 && d_m < 120.0
            }
            _ => false,
        };
        // The reference writes the first two as separate arms that both yield
        // `'market'` (the plaza frontage rule and the core radius); `||` is the
        // same test in the same order, and neither side has a side effect.
        let mut d = if on_plaza_front || d_m < 140.0 {
            "market"
        } else if rd < 60.0 {
            "craftriver"
        } else if !in_wall(c) {
            if d_m > 430.0 { "agrarian" } else { "suburb" }
        } else if d_m < 260.0 {
            "burgher"
        } else {
            "artisan"
        };
        // harbour quarter overrides: parcels fronting the quay, or hard by it,
        // carry the warehouses (harbour-city family, lit. review §1.1 #22)
        if let Some(quay) = harbour
            && (lot.par.edge_cls == "quay" || (dist_to_line(c, quay) < 52.0 && rd < 95.0))
        {
            d = "harbour";
        }
        lot.district = d;
        lot.prov_district = match d {
            "harbour" => PROV_HARBOUR,
            "market" => PROV_MARKET,
            "burgher" => PROV_BURGHER,
            "artisan" => PROV_ARTISAN,
            "craftriver" => PROV_CRAFTRIVER,
            "suburb" => PROV_SUBURB,
            "agrarian" => PROV_AGRARIAN,
            _ => "",
        };
    }

    /* v1.17 (S6 — audit "districts are pure radial zoning with no economic input"): the settlement's
       FUNCTION (site.economy.specialisation — auto-populate's Site-Profile classifier or the editor
       dropdown) re-tags a BOUNDED set of parcels on top of the radial base. Rules, not templates:
       each override keys on the same physical predicates the base pass uses (water distance, market
       distance, wall, frontage class), so the yard lands where the trade physically works. Guarded
       on site.economy ⇒ the synthetic path (headless UME suite) stays byte-identical. */
    let eco = site
        .economy
        .as_ref()
        .and_then(|e| e.specialisation.as_deref())
        // `site.economy && site.economy.specialisation` is a *truthiness* test:
        // an empty specialisation string is falsy and skips the block.
        .filter(|s| !s.is_empty());
    let Some(eco) = eco else {
        return lots;
    };

    // `cand(pred, score)`: every parcel the predicate accepts, sorted by score
    // ascending. Indices, not references — the caller has to retag through the
    // same `&mut` borrow the list came from.
    fn cand(
        lots: &[Lot<'_>],
        pred: &mut dyn FnMut(&Lot<'_>, Vec2) -> bool,
        score: &mut dyn FnMut(&Lot<'_>, Vec2) -> f64,
    ) -> Vec<(usize, f64)> {
        let mut list: Vec<(usize, f64)> = Vec::new();
        for (i, lot) in lots.iter().enumerate() {
            let c = poly_centroid(&lot.par.poly);
            if pred(lot, c) {
                list.push((i, score(lot, c)));
            }
        }
        // The reference's `(a,b)=>a.s-b.s`, encoded. See this function's docs.
        list.sort_by(|a, b| js_num_cmp(a.1, b.1));
        list
    }
    // `for(let i=0;i<Math.min(n,list.length);i++)` — `take` is that cap.
    fn retag(lots: &mut [Lot<'_>], list: &[(usize, f64)], n: usize, d: &'static str) {
        for &(i, _) in list.iter().take(n) {
            lots[i].district = d;
            lots[i].prov_district = eco_prov(d);
        }
    }

    match eco {
        "mining" => {
            // periphery, dry ground; when the adapter supplied a real ore
            // bearing, prefer the side of town facing the deposit (the yard
            // meets the ore road), else simply the outermost parcels. A small
            // town whose every parcel is intramural still works its ore — fall
            // back to the intramural edge (dressing floors inside a mining
            // village's own bounds).
            let bv = ore_bearing.map(|ob| Vec2::new(js_cos(ob), js_sin(ob)));
            let market = anchors.market;
            let mut ore_score = |_l: &Lot<'_>, c: Vec2| match bv {
                Some(b) => -((c.x - market.x) * b.x + (c.y - market.y) * b.y),
                None => -c.dist(market),
            };
            let mut list = cand(
                &lots,
                &mut |_l, c| !in_wall(c) && site.river_dist(c) > 80.0,
                &mut ore_score,
            );
            if list.is_empty() {
                // hamlet scale: relative edge, no absolute floor
                list = cand(
                    &lots,
                    &mut |_l, c| c.dist(anchors.market) > max_rf * 0.4,
                    &mut ore_score,
                );
            }
            retag(&mut lots, &list, 4, "oreyard");
        }
        "fishing" => {
            let list = cand(
                &lots,
                &mut |l, c| site.river_dist(c) < 75.0 && l.district != "harbour",
                &mut |_l, c| site.river_dist(c),
            );
            retag(&mut lots, &list, 5, "fishery");
        }
        "timber" => {
            // logs arrive by water when there is water, by road when there is
            // not: prefer the bank, fall back to the town edge (a dry-country
            // saw yard at the periphery, not no yard at all)
            let mut list = cand(
                &lots,
                &mut |_l, c| site.river_dist(c) < 70.0 && c.dist(anchors.market) > 180.0,
                &mut |_l, c| site.river_dist(c),
            );
            if list.is_empty() {
                // hamlet scale: relative edge
                list = cand(
                    &lots,
                    &mut |_l, c| c.dist(anchors.market) > max_rf * 0.4,
                    &mut |_l, c| -c.dist(anchors.market),
                );
            }
            retag(&mut lots, &list, 4, "sawyard");
        }
        "grain" => {
            let list = cand(
                &lots,
                &mut |l, _c| l.district == "market" || l.district == "burgher",
                &mut |_l, c| c.dist(anchors.market),
            );
            retag(&mut lots, &list, 2, "granary");
        }
        "trade_hub" => {
            let list = cand(
                &lots,
                &mut |l, c| l.par.edge_cls == "primary" && in_wall(c),
                &mut |_l, c| c.dist(anchors.market),
            );
            retag(&mut lots, &list, 6, "warehouse");
        }
        "pastoral" => {
            // a pastoral settlement keeps its stock at hand: outer suburbs
            // become enclosed paddocks
            for lot in &mut lots {
                if lot.district != "suburb" {
                    continue;
                }
                let c = poly_centroid(&lot.par.poly);
                if c.dist(anchors.market) > max_rf * 0.62 {
                    lot.district = "agrarian";
                    lot.prov_district = PROV_PASTORAL_AGRARIAN;
                }
            }
        }
        // garrison/monastic need no district override: the fortress wall spec
        // (S4) and the church/monastery machinery (buildFaithSites) already
        // carry those functions.
        _ => {}
    }

    lots
}

// -------------------------------------------------------------- buildings ---

/// One building footprint.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    /// `'bld' + n`, counted across the whole run.
    pub id: String,
    pub poly: Vec<Vec2>,
    /// The roof ridge, always two points, inset 16% from each end (M-BLD-7).
    pub ridge: [Vec2; 2],
    /// [`Parcel::id`].
    pub parcel: String,
    pub kind: &'static str,
    pub district: &'static str,
    /// `par.age` on every grammar except Venus's, which writes a literal `0`.
    pub age: f64,
    pub courtyard: bool,
    pub prov: &'static str,
}

/// One footprint in a parcel's own `(u, v)` space, as the grammar tables write
/// them.
struct Rect {
    u0: f64,
    u1: f64,
    v0: f64,
    v1: f64,
    kind: &'static str,
    /// Gable to the street: the ridge runs front-to-back regardless of which
    /// footprint axis is longer. Absent in the reference's tables, so `false`.
    gable: bool,
    prov: &'static str,
}

/// `bmap(par, u, v)` (line 30426) — the bilinear patch over a parcel quad.
///
/// `par.F0/F1/B0/B1` are milestone 12's `poly[0]/poly[1]/poly[3]/poly[2]`: the
/// reference stores the quad as `[P0, P1, Q1, Q0]` and the four corners as
/// `F0:P0, F1:P1, B1:Q1, B0:Q0` (line 30324), so the back edge runs
/// `poly[3] → poly[2]`, not `poly[2] → poly[3]`. Getting that pair the wrong
/// way round mirrors every building in the town and still produces a plausible
/// picture.
pub fn bmap(par: &Parcel, u: f64, v: f64) -> Vec2 {
    let e0 = par.poly[0].lerp(par.poly[1], u);
    let e1 = par.poly[3].lerp(par.poly[2], u);
    e0.lerp(e1, v)
}

/// `rectPoly(par, u0, u1, v0, v1)` (line 30429) — a sub-rectangle of a parcel
/// in `(u, v)` space, wound front-left, front-right, back-right, back-left.
pub fn rect_poly(par: &Parcel, u0: f64, u1: f64, v0: f64, v1: f64) -> Vec<Vec2> {
    vec![
        bmap(par, u0, v0),
        bmap(par, u1, v0),
        bmap(par, u1, v1),
        bmap(par, u0, v1),
    ]
}

/// The ridge line of a footprint: along the longer axis, then inset 16% from
/// each end (M-BLD-7, the straight-skeleton spine of a quad).
///
/// `gable` forces the front-to-back orientation — the warehouse and machiya
/// signature, gable to the street.
fn ridge_of(poly: &[Vec2], gable: bool) -> [Vec2; 2] {
    let w_u = poly[0].dist(poly[1]);
    let w_v = poly[1].dist(poly[2]);
    let r = if gable {
        [poly[0].lerp(poly[1], 0.5), poly[3].lerp(poly[2], 0.5)]
    } else if w_u >= w_v {
        [poly[0].lerp(poly[3], 0.5), poly[1].lerp(poly[2], 0.5)]
    } else {
        [poly[0].lerp(poly[1], 0.5), poly[3].lerp(poly[2], 0.5)]
    };
    let ins = 0.16;
    [r[0].lerp(r[1], ins), r[0].lerp(r[1], 1.0 - ins)]
}

/// The Venus grammar's `emit(rectset, courtyard)` — reference line 30462.
///
/// Its `age` is a literal `0`, not `par.age`, and its `kind` comes off the
/// rect rather than being rewritten for warehouses the way the burgage
/// grammar's is.
fn emit(
    out: &mut Vec<Building>,
    bid: &mut usize,
    par: &Parcel,
    d: &'static str,
    rectset: &[Rect],
    courtyard: bool,
) {
    for rc in rectset {
        let poly = rect_poly(par, rc.u0, rc.u1, rc.v0, rc.v1);
        if poly_area(&poly).abs() < 9.0 {
            continue;
        }
        let ridge = ridge_of(&poly, rc.gable);
        out.push(Building {
            id: format!("bld{}", *bid),
            poly,
            ridge,
            parcel: par.id.clone(),
            kind: rc.kind,
            district: d,
            age: 0.0,
            courtyard,
            prov: rc.prov,
        });
        *bid += 1;
    }
}

const PROV_PAVILION: &str = "Circular pavilion: a round civic/amenity building clustered at the hub and inner spokes, echoing the circular buildings of Fresco's plans (M-VEN-5).";
const PROV_VENUS_WAREHOUSE: &str = "Logistics warehouse: a deep gable-fronted store on the outer ring — the distribution belt of the resource-based city (M-VEN-5).";
const PROV_COURTYARD_STREET: &str = "Courtyard-house street range: an Asian-influenced inward-facing dwelling woven into the residential ring (M-VEN-5).";
const PROV_COURTYARD_WING: &str =
    "Courtyard wing opening onto the private central court (M-VEN-5).";
const PROV_COURTYARD_REAR: &str = "Rear range closing the courtyard house (M-VEN-5).";
const PROV_MACHIYA: &str = "Machiya rowhouse: a narrow-fronted, deep Japanese townhouse, mixed into the residential ring for variety (M-VEN-5).";
const PROV_MODULAR: &str = "Modular apartment: the standardized prefabricated residential block of the resource-based city — the base fabric the courtyard houses and machiya are mixed into (M-VEN-5).";

const PROV_ORE_SHED: &str = "Ore shed: dressing floor cover of the mining yard; the spoil ground lies open behind (S6 economy rule).";
const PROV_NET_LOFT: &str = "Net loft: the fishery yard's one covered range; racks and tarring ground lie open (S6 economy rule).";
const PROV_SAW_SHED: &str =
    "Saw shed: covered saw pit of the timber yard; the log landing lies open (S6 economy rule).";

const PROV_TRADE_WAREHOUSE: &str = "Warehouse: deep gable-fronted store along the through-road (trade-hub row, S6 economy rule).";
const PROV_QUAY_WAREHOUSE: &str = "Warehouse: deep gable-fronted store on the quay; goods land at the break-of-bulk point (harbour-city family, lit. review §1.1 #22, §5).";
const PROV_MAIN: &str = "Main range on the build-to line (zero setback, M-BLD-1; depth M-BLD-2).";
const PROV_WING: &str =
    "Rear wing: burgage-cycle infill along the plot side (age-driven, M-BLD-6).";
const PROV_REAR_RANGE: &str = "Rear range closing a courtyard plan (M-BLD-3).";
const PROV_OUTBUILDING: &str = "Rear outbuilding (barn/workshop) at the plot tail (M-BLD-6).";

/// `buildBuildings` (line 30431) — the parcel-conditioned building grammar.
///
/// Three grammars share one loop: Venus's banded blend (pavilions inside
/// `rNorm < 0.42`, logistics warehouses beyond `0.8`, a seeded
/// modular/courtyard/machiya mix between), the S6 working-yard shed, and the
/// medieval burgage cycle — main range on the build-to line, an age-driven rear
/// wing, a rear outbuilding, and a courtyard plan on grand market and burgher
/// plots.
///
/// `terrain_aware` is the opt-in terrain gate (docs/08, M-TER-2): a parcel
/// whose ground scores below 0.5 is left as a vacant lot rather than having its
/// terrain silently ignored.
///
/// **`_seed` and `_plaza` are dead parameters, and they are the reference's.**
/// A grep of the whole function body finds `seed` only on the signature line
/// and inside two prose comments, and `plaza` only on the signature line. Every
/// draw in here comes from `stream(fnv1a(par.id), 'bld')`, which is why two
/// towns with different seeds but the same parcel ids get the same buildings.
/// Both are kept so the call site reads like `generate()` line 30434.
pub fn build_buildings(
    _seed: u32,
    lots: &mut [Lot<'_>],
    _plaza: Option<&Plaza>,
    anchors: &Anchors,
    profile: Option<&CultureProfile>,
    terrain_aware: bool,
) -> Vec<Building> {
    let profile = profile.unwrap_or(&MEDIEVAL);
    let mut out: Vec<Building> = Vec::new();
    let mut bid = 0usize;
    // radial banding extent for the Venus profile: the max parcel distance from
    // the hub, so the grammar can place circular pavilions in the inner band and
    // logistics warehouses in the outer.
    let mut venus_max_r = 0.0f64;
    if profile.building_grammar == "venus-mixed" {
        for lot in lots.iter() {
            let p = lot.par;
            let c = Vec2::new(
                (p.poly[0].x + p.poly[1].x + p.poly[3].x + p.poly[2].x) / 4.0,
                (p.poly[0].y + p.poly[1].y + p.poly[3].y + p.poly[2].y) / 4.0,
            );
            venus_max_r = js_max(
                venus_max_r,
                js_hypot(c.x - anchors.market.x, c.y - anchors.market.y),
            );
        }
    }

    for lot in lots.iter_mut() {
        let par = lot.par;
        let mut r: Substream = stream(fnv1a(&par.id), "bld");
        let d = lot.district;
        if d == "agrarian" && r.chance(0.45) {
            lot.empty = true; // paddocks
            continue;
        }
        if par.frontage < 3.2 || par.depth < 6.0 {
            lot.empty = true;
            continue;
        }
        // terrain-aware placement (docs/08, M-TER-2): opt-in only (default
        // false) — a parcel whose ground is too steep and/or too flood-prone is
        // left unbuilt (a vacant/cleared lot, same rendering as any other
        // `empty` parcel) rather than silently ignoring the terrain, matching
        // the same additive/opt-in discipline as GenerationRules (docs/07 §3.4).
        // Threshold 0.5: a parcel is flagged once EITHER factor alone reaches
        // its own "moderate concern" reference point (slope alone at the
        // ~15%-grade-equivalent reference scores ~0.6; flood alone at half the
        // setback margin scores 0.5) — either constraint alone can already drag
        // a real site below this bar, matching the "neither factor can cancel
        // the other out" design (docs/08 §2).
        if terrain_aware && lot.suitability < 0.5 {
            lot.empty = true;
            lot.unsuitable = true;
            continue;
        }
        // Venus Project blended grammar (M-VEN-5): a deliberate fusion, not a
        // reconstruction. Banded by distance from the hub — circular pavilions
        // (regular polygons standing in for a circle) near the centre/inner
        // rings, a seeded blend of the standardized modular apartment / Asian
        // courtyard house / Japanese machiya through the residential rings, and
        // logistics warehouses on the outermost built ring.
        if profile.building_grammar == "venus-mixed" && d != "harbour" {
            let cx = (par.poly[0].x + par.poly[1].x + par.poly[3].x + par.poly[2].x) / 4.0;
            let cy = (par.poly[0].y + par.poly[1].y + par.poly[3].y + par.poly[2].y) / 4.0;
            let r_norm = if venus_max_r > 0.0 {
                js_hypot(cx - anchors.market.x, cy - anchors.market.y) / venus_max_r
            } else {
                0.5
            };
            // inner band → circular pavilion (civic/amenity), the "circular
            // buildings at the centre/spokes"
            if r_norm < 0.42 && r.chance(0.55) {
                let rad = js_max(3.0, js_min(9.0, js_min(par.frontage, par.depth) / 2.0 - 0.6));
                let n_sides = 14;
                let mut poly = Vec::with_capacity(n_sides);
                for k in 0..n_sides {
                    let a = 2.0 * PI * k as f64 / n_sides as f64;
                    poly.push(Vec2::new(cx + js_cos(a) * rad, cy + js_sin(a) * rad));
                }
                if poly_area(&poly).abs() >= 9.0 {
                    out.push(Building {
                        id: format!("bld{}", bid),
                        poly,
                        ridge: [
                            Vec2::new(cx - rad * 0.35, cy),
                            Vec2::new(cx + rad * 0.35, cy),
                        ],
                        parcel: par.id.clone(),
                        kind: "pavilion",
                        district: d,
                        age: 0.0,
                        courtyard: false,
                        prov: PROV_PAVILION,
                    });
                    bid += 1;
                }
                lot.built = true;
                continue;
            }
            // outer band → logistics warehouse (the distribution belt)
            if r_norm > 0.8 {
                emit(
                    &mut out,
                    &mut bid,
                    par,
                    d,
                    &[Rect {
                        u0: 0.06,
                        u1: 0.94,
                        v0: 0.04,
                        v1: js_min(0.9, js_max(0.4, 14.0 / js_max(par.depth, 1.0) + 0.2)),
                        kind: "warehouse",
                        gable: true,
                        prov: PROV_VENUS_WAREHOUSE,
                    }],
                    false,
                );
                lot.built = true;
                continue;
            }
            // residential rings → seeded blend: modular apartment / Asian
            // courtyard house / Japanese machiya
            let pick = if r.chance(0.45) {
                "modular"
            } else if r.chance(0.55) {
                "courtyard"
            } else {
                "machiya"
            };
            if pick == "courtyard" && par.frontage >= 8.0 && par.depth >= 12.0 {
                emit(
                    &mut out,
                    &mut bid,
                    par,
                    d,
                    &[
                        Rect { u0: 0.0, u1: 1.0, v0: 0.0, v1: 0.24, kind: "street range", gable: false, prov: PROV_COURTYARD_STREET },
                        Rect { u0: 0.0, u1: 0.24, v0: 0.22, v1: 0.8, kind: "wing", gable: false, prov: PROV_COURTYARD_WING },
                        Rect { u0: 0.76, u1: 1.0, v0: 0.22, v1: 0.8, kind: "wing", gable: false, prov: PROV_COURTYARD_WING },
                        Rect { u0: 0.0, u1: 1.0, v0: 0.78, v1: 0.98, kind: "rear range", gable: false, prov: PROV_COURTYARD_REAR },
                    ],
                    true,
                );
            } else if pick == "machiya" {
                emit(
                    &mut out,
                    &mut bid,
                    par,
                    d,
                    &[Rect {
                        u0: 0.06,
                        u1: 0.94,
                        v0: 0.0,
                        v1: js_min(0.82, js_max(0.4, 12.0 / js_max(par.depth, 1.0) + 0.15)),
                        kind: "machiya",
                        gable: true,
                        prov: PROV_MACHIYA,
                    }],
                    false,
                );
            } else {
                emit(
                    &mut out,
                    &mut bid,
                    par,
                    d,
                    &[Rect { u0: 0.04, u1: 0.96, v0: 0.04, v1: 0.96, kind: "modular apartment", gable: false, prov: PROV_MODULAR }],
                    false,
                );
            }
            lot.built = true;
            continue;
        }
        /* v1.17 (S6): working-yard grammar for the economy districts — one long open-sided shed on
           the frontage, the rest open working ground (a yard is defined by its emptiness). Only the
           adapter's economy path ever assigns these districts, so the synthetic suite never enters. */
        if d == "oreyard" || d == "fishery" || d == "sawyard" {
            if r.chance(0.3) {
                lot.empty = true; // some plots are pure open yard
                continue;
            }
            let shed_depth = js_min(0.5, js_max(0.28, 6.0 / js_max(par.depth, 1.0)));
            let poly = rect_poly(par, 0.06, 0.94, 0.0, shed_depth);
            if poly_area(&poly).abs() >= 9.0 {
                // Written out rather than routed through `ridge_of`: this one
                // is unconditionally along the u axis and its 0.16/0.84 insets
                // are a single lerp pair, not the `1 - ins` form the others use.
                let mid_a = poly[0].lerp(poly[3], 0.5);
                let mid_b = poly[1].lerp(poly[2], 0.5);
                let ridge = [mid_a.lerp(mid_b, 0.16), mid_a.lerp(mid_b, 0.84)];
                out.push(Building {
                    id: format!("bld{}", bid),
                    poly,
                    ridge,
                    parcel: par.id.clone(),
                    kind: "shed",
                    district: d,
                    age: par.age,
                    courtyard: false,
                    prov: if d == "oreyard" {
                        PROV_ORE_SHED
                    } else if d == "fishery" {
                        PROV_NET_LOFT
                    } else {
                        PROV_SAW_SHED
                    },
                });
                bid += 1;
            }
            lot.built = true;
            continue;
        }
        let mut rects: Vec<Rect> = Vec::new(); // in (u,v) space
        /* v1.17 (S6): trade-hub warehouse rows share the deep gable-fronted store grammar */
        let is_ware = d == "harbour" || d == "warehouse";
        // warehouses run deep to the plot tail (goods storage); houses stop at
        // roof-span depth
        let main_depth_m = if is_ware {
            js_min(par.depth * 0.85, js_max(9.0, r.logn(15.0, 0.18)))
        } else {
            // M-BLD-2
            js_min(
                par.depth * 0.72,
                js_max(
                    7.0,
                    r.logn(if d == "market" || d == "burgher" { 11.5 } else { 9.5 }, 0.2),
                ),
            )
        };
        let dv = main_depth_m / par.depth;
        let mut g_l = 0.0f64;
        let mut g_r = 0.0f64;
        let detached = (d == "suburb" && r.chance(0.35)) || (d == "agrarian");
        if detached {
            g_l = js_min(0.25, 1.2 / par.frontage);
            g_r = js_min(0.25, (if r.chance(0.5) { 1.4 } else { 0.4 }) / par.frontage);
        } else if !is_ware && r.chance(0.12) {
            // occasional eaves gap (M-BLD-5)
            g_r = js_min(0.2, 0.9 / par.frontage);
        }
        rects.push(Rect { u0: g_l, u1: 1.0 - g_r, v0: 0.0, v1: dv, kind: "main", gable: false, prov: "" });
        // rear wing along one side (burgage-cycle infill, M-BLD-6): probability
        // grows with age
        let mut wing_p = js_min(0.85, 0.18 + par.age * 0.09 + if d == "market" { 0.25 } else { 0.0 });
        if is_ware {
            wing_p = 0.12;
        }
        if par.depth * (1.0 - dv) > 7.0 && r.chance(wing_p) {
            let side = r.chance(0.5);
            let wu = js_min(0.55, js_max(0.3, 4.5 / par.frontage));
            let wl = js_min(0.92, dv + (par.depth * r.range(0.2, 0.42)) / par.depth);
            rects.push(Rect {
                u0: if side { g_l } else { 1.0 - g_r - wu },
                u1: if side { g_l + wu } else { 1.0 - g_r },
                v0: dv - 0.02,
                v1: wl,
                kind: "wing",
                gable: false,
                prov: "",
            });
        }
        // detached rear outbuilding (barn/workshop); a granary plot always
        // builds its store (S6)
        let out_p = if d == "agrarian" {
            0.7
        } else if d == "granary" {
            0.9
        } else {
            0.28
        };
        if par.depth * (1.0 - dv) > 10.0 && r.chance(out_p) {
            let ou = js_min(0.8, js_max(0.35, 5.0 / par.frontage));
            let c = r.range(0.1, 0.9 - ou);
            rects.push(Rect { u0: c, u1: c + ou, v0: 0.78, v1: 0.95, kind: "outbuilding", gable: false, prov: "" });
        }
        // courtyard ring for grand market/burgher parcels (M-BLD-3)
        let mut courtyard = false;
        if (d == "market" || d == "burgher")
            && par.frontage > 10.0
            && par.depth > 18.0
            && r.chance(0.4)
        {
            rects.clear();
            courtyard = true;
            rects.push(Rect { u0: 0.0, u1: 1.0, v0: 0.0, v1: 0.3, kind: "main", gable: false, prov: "" });
            rects.push(Rect { u0: 0.0, u1: 0.32, v0: 0.28, v1: 0.75, kind: "wing", gable: false, prov: "" });
            rects.push(Rect { u0: 0.68, u1: 1.0, v0: 0.28, v1: 0.75, kind: "wing", gable: false, prov: "" });
            rects.push(Rect { u0: 0.0, u1: 1.0, v0: 0.73, v1: 0.9, kind: "rear range", gable: false, prov: "" });
        }
        for rc in &rects {
            let poly = rect_poly(par, rc.u0, rc.u1, rc.v0, rc.v1);
            if poly_area(&poly).abs() < 9.0 {
                continue;
            }
            // warehouse signature: gable to the street, ridge perpendicular to
            // the frontage
            let is_ware_main = is_ware && rc.kind == "main";
            let kind_name = if is_ware_main { "warehouse" } else { rc.kind };
            let ridge = ridge_of(&poly, is_ware_main);
            out.push(Building {
                id: format!("bld{}", bid),
                poly,
                ridge,
                parcel: par.id.clone(),
                kind: kind_name,
                district: d,
                age: par.age,
                courtyard,
                prov: if kind_name == "warehouse" {
                    if d == "warehouse" {
                        PROV_TRADE_WAREHOUSE
                    } else {
                        PROV_QUAY_WAREHOUSE
                    }
                } else if rc.kind == "main" {
                    PROV_MAIN
                } else if rc.kind == "wing" {
                    PROV_WING
                } else if rc.kind == "rear range" {
                    PROV_REAR_RANGE
                } else {
                    PROV_OUTBUILDING
                },
            });
            bid += 1;
        }
        lot.built = true;
    }
    out
}

// ----------------------------------------------------------- faith sites ----

/// A tower on a place of worship — a spire, a minaret or a dome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tower {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub kind: &'static str,
}

/// One place of worship and the precinct it claims.
#[derive(Debug, Clone, PartialEq)]
pub struct FaithSite {
    /// `'worship' + ci`.
    pub id: String,
    /// The rite, verbatim as passed.
    pub faith: String,
    /// `NAME[faith]` — [`None`] where the reference produces `undefined`, which
    /// is any rite outside the five it names. Reachable: `generate()` takes
    /// `opts.faith` from the host and only special-cases `'none'`, so an
    /// unknown rite builds the church form under an undefined name.
    pub name: Option<&'static str>,
    pub center: Vec2,
    /// The [`Parcel::id`]s of the churchyard.
    pub yard: Vec<String>,
    /// Built form — nave and tower, cella, prayer hall, naos/narthex/apse.
    pub polys: Vec<Vec<Vec2>>,
    /// Open precinct — podium, courtyard.
    pub open: Vec<Vec<Vec2>>,
    /// Colonnade points (temple and shrine only).
    pub columns: Vec<Vec2>,
    /// Frontal steps, as `[from, to]` pairs (temple and shrine only).
    pub steps: Vec<[Vec2; 2]>,
    pub tower: Option<Tower>,
    /// Two points, or empty on a rite that sets none. Every branch sets it.
    pub ridge: Vec<Vec2>,
    /// Built by concatenation on the temple and church branches, hence a
    /// [`String`] rather than a `&'static str`.
    pub prov: String,
}

/// `_rectPts(rx, ry, w, h)` (line 30579) — an axis-aligned rectangle about a
/// centre, wound top-left, top-right, bottom-right, bottom-left.
pub fn rect_pts(rx: f64, ry: f64, w: f64, h: f64) -> [Vec2; 4] {
    [
        Vec2::new(rx - w / 2.0, ry - h / 2.0),
        Vec2::new(rx + w / 2.0, ry - h / 2.0),
        Vec2::new(rx + w / 2.0, ry + h / 2.0),
        Vec2::new(rx - w / 2.0, ry + h / 2.0),
    ]
}

/// `_peristyle(rx, ry, w, h, sp)` (line 30580) — colonnade points around a
/// rectangle at roughly `sp` spacing.
///
/// The count per side is `max(2, round(len/sp))` and the columns are laid at
/// `k/n` for `k` in `0..n`, so the far corner of each side is left to the next
/// side's first column and no corner is doubled.
///
/// The loop bound is kept as a float: `js_max` propagates NaN exactly as
/// `Math.max` does, and `k < NaN` is false in both languages, so a NaN spacing
/// produces no columns rather than a panic.
pub fn peristyle(rx: f64, ry: f64, w: f64, h: f64, sp: f64) -> Vec<Vec2> {
    let mut cols = Vec::new();
    let cn = rect_pts(rx, ry, w, h);
    for i in 0..4 {
        let a = cn[i];
        let b = cn[(i + 1) % 4];
        let len = js_hypot(b.x - a.x, b.y - a.y);
        let n = js_max(2.0, js_round(len / sp));
        let mut k = 0.0f64;
        while k < n {
            cols.push(Vec2::new(
                a.x + (b.x - a.x) * k / n,
                a.y + (b.y - a.y) * k / n,
            ));
            k += 1.0;
        }
    }
    cols
}

/// `NAME[faith]`.
fn faith_name(faith: &str) -> Option<&'static str> {
    match faith {
        "church" => Some("Church"),
        "temple" => Some("Temple"),
        "shrine" => Some("Shrine"),
        "mosque" => Some("Mosque"),
        "orthodox" => Some("Orthodox church"),
        _ => None,
    }
}

const PROV_MOSQUE: &str = "Mosque: a covered prayer hall on the qibla side with an open courtyard (sahn) and a minaret — the Islamic rite (M-BLD-8).";
const PROV_ORTHODOX: &str = "Orthodox church: cross-in-square plan — a domed naos on four columns, a narthex (entrance hall) toward the town, an apsed bema behind — the mature Middle Byzantine rite, 9th-12th c. (M-BYZ-1).";

/// `buildFaithSites` (line 30588) — places of worship by rite, each claiming a
/// run of parcels as its churchyard and clearing the houses already on them.
///
/// One parish per ~500 households (M-DEN-8 midpoint). Worship sits in the
/// civic/residential core, never on the working waterfront, so the harbour and
/// craftriver quarters, the quay and the water's edge are all excluded before
/// anything is sited.
///
/// **`lots` and `buildings` are both mutated**: the claimed parcels become the
/// `church` district with `churchyard` set, and every building on them — plus
/// every building whose centroid falls inside the finished precinct — is
/// removed from `buildings`. A caller that keeps its own copy of either will
/// see a town with houses standing inside its church.
#[allow(clippy::too_many_arguments)]
pub fn build_faith_sites(
    seed: u32,
    lots: &mut [Lot<'_>],
    buildings: &mut Vec<Building>,
    anchors: &Anchors,
    count: usize,
    faith: &str,
    site: &Site,
    harbour: Option<&[Vec2]>,
) -> Vec<FaithSite> {
    let mut sites: Vec<FaithSite> = Vec::new();
    for ci in 0..count {
        let mut r: Substream = stream(seed, &format!("faith/{}", ci));
        let cif = ci as f64;
        let d_min = if ci == 0 { 70.0 } else { 170.0 + cif * 60.0 };
        let d_max = if ci == 0 { 230.0 } else { 320.0 + cif * 130.0 };
        let mut best: Option<usize> = None;
        let mut bs = f64::INFINITY;
        for (i, lot) in lots.iter().enumerate() {
            if lot.churchyard {
                continue;
            }
            // worship sits in the civic/residential core, NOT on the working
            // waterfront — skip the harbour quarter, the noxious riverside
            // crafts, and the immediate quay/water edge
            if lot.district == "harbour" || lot.district == "craftriver" {
                continue;
            }
            let c = poly_centroid(&lot.par.poly);
            let d_m = c.dist(anchors.market);
            if d_m < d_min || d_m > d_max {
                continue;
            }
            if lot.par.frontage < 7.0 || lot.par.depth < 10.0 {
                continue;
            }
            if let Some(quay) = harbour
                && dist_to_line(c, quay) < 70.0
            {
                continue;
            }
            if !site.no_water && site.river_dist(c) < 45.0 {
                continue;
            }
            if sites.iter().any(|s| c.dist(s.center) < 180.0) {
                continue;
            }
            let sc = d_m + r.range(0.0, 40.0);
            if sc < bs {
                bs = sc;
                best = Some(i);
            }
        }
        let Some(best) = best else { continue };

        let best_block = lots[best].par.block.clone();
        let best_centroid = poly_centroid(&lots[best].par.poly);
        let best_frontage = lots[best].par.frontage;
        let mut group: Vec<usize> = vec![best];
        for (i, lot) in lots.iter().enumerate() {
            if i == best || lot.par.block != best_block || lot.churchyard {
                continue;
            }
            if poly_centroid(&lot.par.poly).dist(best_centroid)
                < best_frontage + lot.par.frontage + 6.0
            {
                group.push(i);
            }
            // Checked every iteration, after the push — so the loop stops on
            // the iteration that takes the group to three, not on the next one.
            if group.len() >= 3 {
                break;
            }
        }
        for &i in &group {
            lots[i].district = "church";
            lots[i].churchyard = true;
            lots[i].prov_district = PROV_CHURCHYARD;
        }
        let ids: Vec<String> = group.iter().map(|&i| lots[i].par.id.clone()).collect();
        buildings.retain(|b| !ids.contains(&b.parcel));

        let mut minx = f64::INFINITY;
        let mut maxx = f64::NEG_INFINITY;
        let mut miny = f64::INFINITY;
        let mut maxy = f64::NEG_INFINITY;
        for &i in &group {
            for pt in &lots[i].par.poly {
                if pt.x < minx {
                    minx = pt.x;
                }
                if pt.x > maxx {
                    maxx = pt.x;
                }
                if pt.y < miny {
                    miny = pt.y;
                }
                if pt.y > maxy {
                    maxy = pt.y;
                }
            }
        }
        let cx = (minx + maxx) / 2.0;
        let cy = (miny + maxy) / 2.0;
        let ex_x = maxx - minx;
        let ex_y = maxy - miny;
        let mg = 2.5;
        let horiz = ex_x >= ex_y;

        let mut s = FaithSite {
            id: format!("worship{}", ci),
            faith: faith.to_string(),
            name: faith_name(faith),
            center: Vec2::new(cx, cy),
            yard: ids,
            polys: Vec::new(),
            open: Vec::new(),
            columns: Vec::new(),
            steps: Vec::new(),
            tower: None,
            ridge: Vec::new(),
            prov: String::new(),
        };

        if faith == "temple" || faith == "shrine" {
            let big = faith == "temple";
            let l = js_max(
                if big { 11.0 } else { 6.0 },
                js_min(
                    if big {
                        if ci == 0 { 26.0 } else { 20.0 }
                    } else {
                        11.0
                    },
                    (if horiz { ex_x } else { ex_y }) - 2.0 * mg,
                ),
            );
            let w = js_max(
                if big { 7.0 } else { 4.0 },
                js_min(
                    if big { 13.0 } else { 7.0 },
                    (if horiz { ex_y } else { ex_x }) - 2.0 * mg,
                ),
            );
            let lw = if horiz { l } else { w };
            let lh = if horiz { w } else { l };
            s.open.push(rect_pts(cx, cy, lw + 2.5, lh + 2.5).to_vec()); // podium
            s.polys.push(rect_pts(cx, cy, lw, lh).to_vec()); // cella
            // colonnade (peristyle)
            s.columns = peristyle(cx, cy, lw + 1.6, lh + 1.6, if big { 3.4 } else { 3.0 });
            for st in 0..3 {
                let o = lw / 2.0 + 2.0 + st as f64 * 1.3;
                let o_y = lh / 2.0 + 2.0 + st as f64 * 1.3;
                if horiz {
                    let dx = if anchors.market.x > cx { o } else { -o };
                    s.steps.push([
                        Vec2::new(cx + dx, cy - lh / 2.0),
                        Vec2::new(cx + dx, cy + lh / 2.0),
                    ]);
                } else {
                    let dy = if anchors.market.y > cy { o_y } else { -o_y };
                    s.steps.push([
                        Vec2::new(cx - lw / 2.0, cy + dy),
                        Vec2::new(cx + lw / 2.0, cy + dy),
                    ]);
                }
            }
            s.ridge = if horiz {
                vec![
                    Vec2::new(cx - lw / 2.0 + 2.0, cy),
                    Vec2::new(cx + lw / 2.0 - 2.0, cy),
                ]
            } else {
                vec![
                    Vec2::new(cx, cy - lh / 2.0 + 2.0),
                    Vec2::new(cx, cy + lh / 2.0 - 2.0),
                ]
            };
            s.prov = format!(
                "{}: a {}cella on a raised podium with a frontal flight of steps toward the \
                 forum/market — the Roman/Greek rite, oriented to its surroundings rather than to \
                 the east (M-BLD-8; Roman temple form).",
                if big { "Classical temple" } else { "Shrine" },
                if big { "colonnaded " } else { "" }
            );
        } else if faith == "mosque" {
            let l = js_max(
                12.0,
                js_min(if ci == 0 { 26.0 } else { 20.0 }, ex_x - 2.0 * mg),
            );
            let h = js_max(9.0, js_min(ex_y - 2.0 * mg, l * 0.85));
            // prayer hall (qibla side)
            s.polys.push(rect_pts(cx, cy + h * 0.25, l, h * 0.5).to_vec());
            // sahn (courtyard)
            s.open.push(rect_pts(cx, cy - h * 0.25, l * 0.92, h * 0.44).to_vec());
            // minaret at a courtyard corner
            s.tower = Some(Tower { x: cx - l / 2.0 + 2.0, y: cy - h * 0.25, r: 2.4, kind: "minaret" });
            s.ridge = vec![
                Vec2::new(cx - l / 2.0 + 2.0, cy + h * 0.25),
                Vec2::new(cx + l / 2.0 - 2.0, cy + h * 0.25),
            ];
            s.prov = PROV_MOSQUE.to_string();
        } else if faith == "orthodox" {
            // cross-in-square: a domed naos on four columns (nine-bay), a
            // narthex (entrance hall) toward the town, an apsed bema behind —
            // the mature Middle Byzantine plan (M-BYZ-1)
            let side = js_max(
                8.0,
                js_min(
                    if ci == 0 { 18.0 } else { 14.0 },
                    js_min(ex_x, ex_y) - 2.0 * mg,
                ),
            );
            s.polys.push(rect_pts(cx, cy, side, side).to_vec());
            let mut inl = (anchors.market - Vec2::new(cx, cy)).norm();
            if !inl.x.is_finite() {
                inl = Vec2::new(0.0, 1.0);
            }
            let perp = inl.rot90();
            let nd = side * 0.34;
            let nw = side * 0.9;
            let narthex_c = Vec2::new(cx, cy) + inl * (side / 2.0 + nd / 2.0);
            s.polys.push(vec![
                narthex_c + perp * (-nw / 2.0) + inl * (-nd / 2.0),
                narthex_c + perp * (nw / 2.0) + inl * (-nd / 2.0),
                narthex_c + perp * (nw / 2.0) + inl * (nd / 2.0),
                narthex_c + perp * (-nw / 2.0) + inl * (nd / 2.0),
            ]);
            let apse_c = Vec2::new(cx, cy) - inl * (side / 2.0);
            let apse_r = side * 0.26;
            let mut ap = Vec::with_capacity(9);
            for k in 0..=8 {
                let a = (-PI / 2.0) + PI * k as f64 / 8.0;
                ap.push(apse_c + perp * (js_cos(a) * apse_r) + inl * (-js_sin(a) * apse_r));
            }
            s.polys.push(ap);
            // central dome on the crossing
            s.tower = Some(Tower { x: cx, y: cy, r: 2.6, kind: "dome" });
            s.ridge = vec![
                Vec2::new(cx - side / 2.0 + 1.5, cy),
                Vec2::new(cx + side / 2.0 - 1.5, cy),
            ];
            s.prov = PROV_ORTHODOX.to_string();
        } else {
            let l = js_max(9.0, js_min(if ci == 0 { 30.0 } else { 22.0 }, ex_x - 2.0 * mg));
            let wn = js_max(4.5, js_min(l * 0.42, ex_y * 0.5 - mg));
            let nave = rect_pts(cx, cy, l, wn);
            let t = js_max(wn, js_min(wn * 1.7, ex_y - 2.0 * mg));
            let tw = js_min(wn * 0.85, l * 0.28);
            let tcx = cx + js_min(l * 0.14, (ex_x - l) / 2.0 + l * 0.14);
            s.polys.push(nave.to_vec());
            s.polys.push(rect_pts(tcx, cy, tw, t).to_vec());
            // west tower
            s.tower = Some(Tower { x: cx - l / 2.0 + 1.6, y: cy, r: 1.8, kind: "spire" });
            s.ridge = vec![
                Vec2::new(cx - l / 2.0 + 2.0, cy),
                Vec2::new(cx + l / 2.0 - 2.0, cy),
            ];
            s.prov = format!(
                "{}: cross-plan on the liturgical east-west axis (chancel eastward), sized to its \
                 churchyard (M-DEN-8, M-BLD-4, M-BLD-8).",
                if ci == 0 {
                    "Principal parish church"
                } else {
                    "Parish church"
                }
            );
        }

        buildings.retain(|b| {
            let bc = poly_centroid(&b.poly);
            !(s.polys.iter().any(|pl| point_in_poly(bc, pl))
                || s.open.iter().any(|pl| point_in_poly(bc, pl)))
        });
        sites.push(s);
    }
    sites
}
