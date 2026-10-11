//! One table of way classes, read by **both** the Route/Way tools' least-cost
//! router and the Journey Planner (Ruling CD item 3, owner, 2026-10-11).
//!
//! What it is for. Before this module the two consumers each kept their own
//! idea of what a way is worth: the router (`tools::civ_dijkstra_path_guided`)
//! multiplied every way cell by one flat `x0.25` whatever the way's type, and
//! the planner (`jp_road_cells`) mapped the type onto a `JP_TERRAIN.land` /
//! `JP_ROUTE.land` row with its own `match`. `PLANNER_AUDIT_2026-10-11.md` Q2
//! finding 1 measured the result: the router thought every way 4x cheaper, the
//! planner thought a dirt road about 1.24x faster and a highway about 2.1x, so
//! the two could disagree about which route is best. Ruling CD item 3 asks for
//! one table so they *cannot* drift; this is that table.
//!
//! How the two columns relate. The planner row is the primary datum (it is the
//! reference's own `_jpRoadCells` mapping, `Cartalith Gen1 v2.11.html` line
//! 18821-18822, read at that function). The router discount is **derived** from
//! it in [`router_discount`]: the reference's flat `_CIV_EXISTING_WAY_DISCOUNT`
//! (`0.25`, v2.11 line 22241) is kept as the anchor for the plain road, and
//! every other land class is scaled by its planner speed relative to the plain
//! road (`JP_TERRAIN.land[terrain] x JP_ROUTE.land[cond]`, the same product
//! `jp_calc_land_ex` multiplies). So a class the planner believes is twice as
//! fast costs half as much to the router, and editing one row moves both
//! consumers together. The scaling rule is a **judgement** (labelled here and
//! in `doc_notes` for `DECISIONS.md`): the reference has no per-type router
//! cost at all; the anchor and every planner row but Track's are the
//! reference's (Track's own row is Ruling CD addendum item 5, see
//! [`planner_row`]).
//!
//! What it must never do. It must never give the plain road a discount other
//! than exactly `0.25` (every router golden and every route over a plain road
//! is pinned to it); it must never give a sea lane a planner row (the
//! reference's `_jpRoadCells` skips sea lanes, `w.sea||w.type==='sea-lane'`,
//! so there is no reference row to read and inventing one would encode "no
//! value" as a plausible terrain); and it must never be bypassed by a second
//! `match` on way type in either consumer.
//!
//! What it deliberately does not cover. `tools::civ_connect_village_addons`
//! (the *generation* pipeline's village-connector pass, golden-verified by
//! `golden_parity_village_connect.rs`) keeps the reference's flat `0.25`: it is
//! the network builder, not the Route tool, and changing it would move
//! generated output, which Ruling CD does not authorise.

use crate::tools::ManualWayType;
use crate::WayType;

/// Every kind of way either consumer can meet, generated or hand-drawn.
///
/// One enum over both of this crate's way-type vocabularies: [`WayType`] (the
/// generated network: highway/regional/road/track, plus `Ancient` for village
/// connectors) and [`ManualWayType`] (the user's declared intent:
/// road/track/sea lane/ancient). The reference keeps one `type` string on one
/// `civWays` array for both, and `_jpRoadCells` reads that one string, so a
/// generated `track` and a hand-drawn `track` are the same class here exactly
/// as they are there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WayClass {
    Highway,
    Regional,
    Road,
    Track,
    Ancient,
    SeaLane,
}

impl From<WayType> for WayClass {
    /// A generated way's class is its own type; there is no generated sea lane
    /// (`_civHierarchicalNetwork`'s v1.99 comment: "this network is always
    /// land-only").
    fn from(t: WayType) -> Self {
        match t {
            WayType::Highway => WayClass::Highway,
            WayType::Regional => WayClass::Regional,
            WayType::Road => WayClass::Road,
            WayType::Track => WayClass::Track,
            WayType::Ancient => WayClass::Ancient,
        }
    }
}

impl From<ManualWayType> for WayClass {
    /// A hand-drawn way's class is its declared type.
    fn from(t: ManualWayType) -> Self {
        match t {
            ManualWayType::Road => WayClass::Road,
            ManualWayType::Track => WayClass::Track,
            ManualWayType::Ancient => WayClass::Ancient,
            ManualWayType::SeaLane => WayClass::SeaLane,
        }
    }
}

/// The planner's view of one land way class: the `JP_TERRAIN.land` row a
/// route point on it reads as, the `JP_ROUTE.land` condition, and the overlap
/// priority (`_jpRoadCells`' `pri`: a higher class wins a cell another class
/// also covers; equal priorities keep whichever was written first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WayPlannerRow {
    pub terrain: &'static str,
    pub cond: &'static str,
    pub pri: u8,
}

/// The planner row for `class`, or `None` for a sea lane, which the planner
/// does not credit (the reference's own `_jpRoadCells` guard; see the module
/// doc for why no row is invented).
///
/// Every row but `Track`'s is the reference's `_jpRoadCells` (v2.11 line
/// 18821-18826): `highway -> ["Paved Road","Maintained"]` pri 3, `regional ->
/// ["Paved Road","Standard"]` pri 2, `ancient -> ["Dirt Track","Deteriorated"]`,
/// every other type (road, and in the reference track too)
/// `["Dirt Track","Standard"]`, pri 1.
///
/// **`Track` has its own, slower row: `["Dirt Track","None / Wild"]`, pri 1**
/// (Ruling CD addendum item 5, owner, 2026-10-11; a deliberate departure from
/// `_jpRoadCells`, DECISIONS.md 7p). Why this row, of the reference's own
/// vocabulary (a **judgement**): a track is an unmaintained dirt way, so its
/// terrain stays `JP_TERRAIN.land["Dirt Track"]` (1.00) and only its condition
/// drops; `JP_ROUTE.land["None / Wild"]` (0.85) is the only condition strictly
/// between the road's `Standard` (1.00) and the ancient way's `Deteriorated`
/// (0.75), so the three way classes stay distinct and ordered road > track >
/// ancient. (`Deteriorated` would make a track identical to an ancient way;
/// `Broken`, 0.55, would make it slower.) Its priority stays the reference's 1,
/// so an overlap of a road and a track still keeps the first writer.
///
/// This moves the reference-verified milestone-5/6 planner goldens, whose
/// fixture rides a generated `Track` for stages 4 and 5 (41.318 to 43.203
/// days); they were re-baselined under the addendum, disclosed as made after
/// seeing the result. The router's discount for `Track` moves with it
/// ([`router_discount`]: 0.25 to `0.25 / 0.85` = 0.2941).
pub fn planner_row(class: WayClass) -> Option<WayPlannerRow> {
    let row = |terrain, cond, pri| Some(WayPlannerRow { terrain, cond, pri });
    match class {
        WayClass::Highway => row("Paved Road", "Maintained", 3),
        WayClass::Regional => row("Paved Road", "Standard", 2),
        WayClass::Road => row("Dirt Track", "Standard", 1),
        WayClass::Track => row("Dirt Track", "None / Wild", 1),
        WayClass::Ancient => row("Dirt Track", "Deteriorated", 1),
        WayClass::SeaLane => None,
    }
}

/// `_CIV_EXISTING_WAY_DISCOUNT` (`Cartalith Gen1 v2.11.html` line 22241):
/// the reference's one flat router multiplier for a cell on any way. Here it is
/// the plain road's discount (the anchor every land class scales from) and the
/// sea lane's (which has no planner speed to scale by, so it keeps the
/// reference value unchanged).
pub const REFERENCE_WAY_DISCOUNT: f32 = 0.25;

/// The planner's walking-speed multiplier for a land class: the product of its
/// `JP_TERRAIN.land` and `JP_ROUTE.land` modifiers, the same two factors
/// `jp_calc_land_ex` multiplies for a stage on that row. `None` for a class
/// with no planner row.
pub fn planner_speed(class: WayClass) -> Option<f64> {
    planner_row(class).map(|r| crate::jp_terrain_land_mod(r.terrain) * crate::jp_route_mod("land", r.cond))
}

/// The router's cost multiplier for a cell on a way of `class`.
///
/// `0.25 x speed(Road) / speed(class)` for a land class (a judgement: the
/// router's preference between two way classes is the planner's speed ratio
/// between them; see the module doc), and the reference's `0.25` for a sea
/// lane. Values today, from the rows in [`planner_row`]: highway
/// `0.25 / 1.80 = 0.1389`, regional `0.25 / 1.50 = 0.1667`, road `0.25`,
/// track `0.25 / 0.85 = 0.2941`, ancient `0.25 / 0.75 = 0.3333`, sea lane
/// `0.25`. So a faster class is always cheaper: highway < regional < road <
/// track < ancient.
///
/// Computed in `f64` and narrowed once, so the plain road is `0.25` to the bit
/// (`0.25 * 1.0 / 1.0`), exactly the constant the router used before.
pub fn router_discount(class: WayClass) -> f32 {
    match (planner_speed(class), planner_speed(WayClass::Road)) {
        (Some(s), Some(road)) if s > 0.0 => (REFERENCE_WAY_DISCOUNT as f64 * road / s) as f32,
        _ => REFERENCE_WAY_DISCOUNT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the plain road's router discount is exactly 0.25 (bit for bit),
    /// so every route over a plain road -- and every router golden, which passes
    /// plain-road ways -- is unchanged by Ruling CD item 3.
    #[test]
    fn the_plain_road_keeps_the_references_flat_discount_to_the_bit() {
        assert_eq!(router_discount(WayClass::Road).to_bits(), 0.25f32.to_bits());
        assert_eq!(router_discount(WayClass::SeaLane).to_bits(), 0.25f32.to_bits());
    }

    /// Protects: the derived router discounts, as literals computed by hand from
    /// the reference's JP_TERRAIN.land/JP_ROUTE.land rows (Paved Road 1.50,
    /// Dirt Track 1.00; Maintained 1.20, Standard 1.00, None / Wild 0.85,
    /// Deteriorated 0.75), and their ordering: a faster way is always cheaper
    /// to the router. Track's 0.25 / 0.85 re-baselined 2026-10-11 under Ruling
    /// CD (addendum item 5; was 0.25, the road's).
    #[test]
    fn faster_way_classes_are_cheaper_to_the_router() {
        let d = |c| router_discount(c) as f64;
        assert!((d(WayClass::Highway) - 0.25 / 1.8).abs() < 1e-7, "highway {}", d(WayClass::Highway));
        assert!((d(WayClass::Regional) - 0.25 / 1.5).abs() < 1e-7, "regional {}", d(WayClass::Regional));
        assert!((d(WayClass::Ancient) - 0.25 / 0.75).abs() < 1e-7, "ancient {}", d(WayClass::Ancient));
        assert!((d(WayClass::Track) - 0.294_117_647).abs() < 1e-7, "track {}", d(WayClass::Track));
        assert!(d(WayClass::Highway) < d(WayClass::Regional));
        assert!(d(WayClass::Regional) < d(WayClass::Road));
        assert!(d(WayClass::Road) < d(WayClass::Track));
        assert!(d(WayClass::Track) < d(WayClass::Ancient));
    }

    /// Protects: the planner rows are the reference's `_jpRoadCells` tuples
    /// (v2.11 line 18821-18826), written out as literals, except Track's own
    /// row (Ruling CD addendum item 5; re-baselined 2026-10-11 under Ruling CD,
    /// was the road's `Dirt Track / Standard`), and a sea lane has no row
    /// rather than a made-up one.
    #[test]
    fn planner_rows_are_the_references_own_tuples() {
        let r = |c| planner_row(c).map(|r| (r.terrain, r.cond, r.pri));
        assert_eq!(r(WayClass::Highway), Some(("Paved Road", "Maintained", 3)));
        assert_eq!(r(WayClass::Regional), Some(("Paved Road", "Standard", 2)));
        assert_eq!(r(WayClass::Road), Some(("Dirt Track", "Standard", 1)));
        assert_eq!(r(WayClass::Track), Some(("Dirt Track", "None / Wild", 1)));
        assert_eq!(r(WayClass::Ancient), Some(("Dirt Track", "Deteriorated", 1)));
        assert_eq!(r(WayClass::SeaLane), None);
    }

    /// Protects: both way-type vocabularies map onto the shared classes the way
    /// the reference's one `type` string does (a generated and a hand-drawn
    /// track are the same class).
    #[test]
    fn both_way_type_vocabularies_map_onto_one_class_set() {
        assert_eq!(WayClass::from(WayType::Track), WayClass::from(ManualWayType::Track));
        assert_eq!(WayClass::from(WayType::Road), WayClass::from(ManualWayType::Road));
        assert_eq!(WayClass::from(WayType::Ancient), WayClass::from(ManualWayType::Ancient));
        assert_eq!(WayClass::from(ManualWayType::SeaLane), WayClass::SeaLane);
        assert_eq!(WayClass::from(WayType::Highway), WayClass::Highway);
        assert_eq!(WayClass::from(WayType::Regional), WayClass::Regional);
    }
}
