//! The CIVIL tool group's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md`
//! milestone F, `DCC_SHELL_SPEC.md` §4.5.3's Settlement and Territory tools.
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`'s own module doc argues for: `lib.rs` owns the thin
//! `Variant`<->`f64`/`String` conversion and the `#[func]` surface; this
//! module owns the actual state (the territory-paint draft, the manual-
//! placement name/population RNG stream) and the pure helper functions that
//! stand between `cartalith-civ`'s tested primitives (`tools.rs`:
//! `civ_pick_place_at`, `civ_drop_place`, `merge_territory_paint`) and a
//! Godot method signature — with its own `#[cfg(test)]` suite below,
//! exercised by `cargo test -p cartalith-godot`'s ordinary unit-test pass,
//! no Godot runtime involved.
//!
//! ## §4.5.3 is a superset of what `cartalith-civ` actually models
//!
//! Two gaps found while binding this, both real and both reported rather
//! than papered over:
//!
//! - **POI is not a ported concept.** `cartalith-civ/src/tools.rs` ports
//!   Settlement (`civ_drop_place`) and Territory (`merge_territory_paint`)
//!   only; its own doc comment for `civ_place_pick_weight` says outright
//!   that "the reference's POI branch ... is likewise absent because this
//!   port has no POI concept." `_civDropPOI` has no Rust counterpart
//!   anywhere in the workspace. This module therefore binds Settlement and
//!   Territory only — no `civ_drop_poi`, no fabricated POI record type.
//! - **"Metropolis" is not a placeable kind either.** `civ_place_pick_weight`'s
//!   own doc: `SettlementKind` has the five tiers `place_settlements`
//!   actually produces (hamlet..capital); metropolis (reference rank 5) and
//!   the special kinds are "not approximated here." [`kind_from_str`] below
//!   accepts exactly those five and rejects `"metropolis"` the same as any
//!   other unknown string — §4.5.3's own class list ("metropolis / city /
//!   town / village / hamlet") is one tier wider than the engine.
//!
//! ## Territory paint needs a draft, and needs one more thing besides
//!
//! `merge_territory_paint(territory: &mut [i32], paint: &[u8])` merges
//! in place and is not itself an undo-friendly primitive: once a stroke is
//! merged into `civ.territory`, the pre-paint value at every touched cell is
//! gone. Milestone C's own precedent — territory paint is `PaintStamp`/
//! `PaintLayer`, unchanged — supplies the missing piece: `PaintStamp`
//! implements `cartalith_spatial::pass::Stamp`, so it gets
//! `PassBuffer<PaintStamp>`'s draft/preview/discard for one in-progress
//! stroke for free, the same way `sculpt_bridge::SculptEditor` gets it for
//! `SculptStamp`.
//!
//! That alone would still make **subtract** (⇧, §4.5.3) a one-way street:
//! a subtract dab has to mean "let the computed base show through again,"
//! not merely "write faction 0," because a cell already painted by an
//! *earlier, already-committed* stroke has no `0` in `civ.territory` to
//! fall back to — that value was overwritten at the earlier commit. So this
//! module keeps two more things past the draft: [`CivTools::territory_base`]
//! (a pristine, once-only snapshot of `assign_territory`'s own output,
//! captured in `WorldGen::absorb`) and [`CivTools::territory_paint`], a
//! **persistent** `PaintLayer` that accumulates every committed stroke
//! (not just the most recent one). Every commit rebuilds `civ.territory`
//! from scratch as `territory_base` merged with the full accumulated
//! `territory_paint` — so an "erase" dab (`value = 0`) genuinely restores
//! the algorithmic answer at that cell, however many strokes came before it.
//!
//! ## The manual-placement name/population RNG needs its own stream
//!
//! `cartalith_civ::civ_name_rng()` is a **pure function of a fixed
//! constant** (`CIV_NAME_RNG_SEED_INPUT`) — calling it fresh on every click
//! would hand every unnamed settlement of a session the identical name and
//! population. `tools.rs`'s own doc comment on `civ_drop_place` anticipates
//! exactly this call site: naming/populating a hand-placed settlement is
//! "the caller's explicit choice, not a hidden one," specifically so it
//! does not steal draws from the auto-populate pass's own stream. This
//! bridge is that caller: [`CivTools::name_rng`] is seeded once (also in
//! `WorldGen::absorb`, via `civ_name_rng()`) and advances across every
//! manual placement made against one generated world, so repeated blank-name
//! drops still get distinct names — its own independent stream, never
//! shared with `compute_civilisation`'s.
//!
//! ## "Contested cell" has no reference or engine meaning — this is new
//!
//! `DCC_SHELL_SPEC.md` §4.5.3 asks the Territory right dock for "a
//! contested-cell warning," but `assign_territory` produces a strict
//! single-owner-per-cell raster with no ambiguity or overlap representation
//! at all, and nothing in `cartalith-civ` computes one. [`contested_cell_count`]
//! is this bridge's own reading, flagged as an addition rather than parity
//! (`DECISIONS.md` §7d): a claimed cell counts as contested when it is
//! 4-adjacent to a *different* faction's claimed cell — a border-cell
//! heuristic computed from data that already exists, not a re-run of the
//! cost-distance Voronoi that would be needed to model genuine overlap.

// `civ_drop_place`/`DropPlace`/`merge_territory_paint`/the pick-radius
// helpers live in `cartalith_civ::tools` (a real submodule, `pub mod
// tools;` -- not re-exported at the crate root), unlike
// `civ_base_pop_for_kind`/`civ_settle_name`/`NamedSettlement`/
// `SettlementKind`, which are declared directly in `cartalith-civ/src/
// lib.rs` itself. Two different paths into the same crate, kept separate
// below rather than glossed over with a blanket `cartalith_civ::*`.
use cartalith_civ::tools::{civ_drop_place, DropPlace};
use cartalith_civ::{civ_base_pop_for_kind, civ_settle_name, NamedSettlement, SettlementKind};
use cartalith_spatial::{DirtyTracker, PaintLayer, PaintStamp, PassBuffer};

/// Tile granularity for the territory-paint draft's `PassBuffer`/
/// `DirtyTracker` pair. Same reasoning as `sculpt_bridge::SCULPT_TILE_SIZE`
/// (no reference tiling concept to match — the reference paints straight
/// into `civTerritory` with no draft at all): small enough that one dab at
/// the reference's own `_civTerRadius` maximum (20 cells, `tools.rs`'
/// `TERRITORY_BRUSH_RADIUS` doc) touches a handful of tiles, not one giant
/// tile or hundreds of tiny ones.
pub const TERRITORY_TILE_SIZE: usize = 64;

/// `§4.5.3`'s Settlement class dropdown, resolved against the five tiers
/// `SettlementKind` actually has. Case-insensitive (a shell's own combo box
/// choice, not reference JS text). `None` for `"metropolis"` and anything
/// else unrecognised — see this module's own doc comment on why metropolis
/// is a real, reported gap rather than mapped onto the nearest tier.
pub fn kind_from_str(s: &str) -> Option<SettlementKind> {
    match s.to_ascii_lowercase().as_str() {
        "capital" => Some(SettlementKind::Capital),
        "city" => Some(SettlementKind::City),
        "town" => Some(SettlementKind::Town),
        "village" => Some(SettlementKind::Village),
        "hamlet" => Some(SettlementKind::Hamlet),
        _ => None,
    }
}

/// The nearest dry-land, non-water-body cell to `(gx, gy)` within `max_r`
/// cells, or `(gx, gy)` itself if it is already land. `None` if every cell
/// in range is water (or `(gx, gy)` is out of bounds).
///
/// **A new affordance, not a ported one.** Neither the reference nor
/// `cartalith-civ` has a "snap to water" concept for settlement placement —
/// `_civDropPlace` simply refuses a water click (`DropPlace::Water`). This
/// exists because `DCC_SHELL_SPEC.md` §4.5.3's Settlement tool options row
/// lists a "snap to water" toggle with nothing behind it yet; `lib.rs`'s
/// `civ_drop_settlement` calls this only when that toggle is on, and falls
/// through to the ordinary refusal when it finds nothing in range.
///
/// A full box scan, not a spiral search: `max_r` is always a handful of
/// cells (`civ_snap_radius`'s own floor is 5), so a `(2r+1)^2` scan is
/// trivial, and scanning the whole box up front (rather than stopping at
/// the first ring with a hit) is what guarantees the *nearest* cell wins
/// when a diagonal and an axis-aligned candidate tie on ring but not on
/// true distance.
pub fn nearest_land_cell(gx: usize, gy: usize, gw: usize, gh: usize, field: &[f32], water_bodies: &[u8], sea: f64, max_r: f64) -> Option<(usize, usize)> {
    if gw == 0 || gh == 0 || gx >= gw || gy >= gh {
        return None;
    }
    let is_land = |x: usize, y: usize| {
        let i = y * gw + x;
        (field[i] as f64) >= sea && water_bodies[i] == 0
    };
    if is_land(gx, gy) {
        return Some((gx, gy));
    }
    let r = max_r.floor().max(0.0) as isize;
    let (cx, cy) = (gx as isize, gy as isize);
    let mut best: Option<(usize, usize, f64)> = None;
    for dy in -r..=r {
        for dx in -r..=r {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x as usize >= gw || y as usize >= gh {
                continue;
            }
            let d2 = (dx * dx + dy * dy) as f64;
            if d2 > max_r * max_r {
                continue;
            }
            let (xu, yu) = (x as usize, y as usize);
            if !is_land(xu, yu) {
                continue;
            }
            if best.is_none_or(|(_, _, bd)| d2 < bd) {
                best = Some((xu, yu, d2));
            }
        }
    }
    best.map(|(x, y, _)| (x, y))
}

/// `civ_drop_place`'s `name: ""` default, resolved: a blank shell-supplied
/// name gets a real one from `civ_settle_name` (advancing `rng`); a
/// non-blank one is used verbatim. Trimmed before the blank check so a
/// shell that sends whitespace by accident still gets a generated name
/// rather than a settlement literally named " ".
pub fn manual_settlement_name(name: &str, faction: i32, rng: &mut cartalith_rng::Mulberry32) -> String {
    if name.trim().is_empty() {
        civ_settle_name(rng, faction)
    } else {
        name.to_string()
    }
}

/// `civ_drop_place`'s `pop: 1000` placeholder, resolved into the same
/// "properly named, tier-populated result" `tools.rs`'s own doc comment
/// invites a caller to build: `name_and_populate_settlements_with_rng`'s
/// exact formula (`base * (0.7 + suit*0.8) * (0.8 + rng*0.4)`), reused
/// rather than re-derived so a manual drop's population follows the same
/// curve an auto-populated one does. `suit` is `0.0` here (this bridge
/// samples no suitability raster at the click, same honesty `civ_drop_place`
/// itself already documents for its own `suit` field), which the formula's
/// own `0.7 + suit*0.8` folds down to a flat `x0.7`.
pub fn manual_settlement_pop(kind: SettlementKind, suit: f64, rng: &mut cartalith_rng::Mulberry32) -> u32 {
    let base = civ_base_pop_for_kind(kind);
    (base * (0.7 + suit * 0.8) * (0.8 + rng.next_f64() * 0.4)).round() as u32
}

/// `_civDropPlace` (via `cartalith_civ::civ_drop_place`) plus this bridge's
/// own naming/population resolution, in one call so `lib.rs`'s `#[func]`
/// stays thin. Returns the index the shell should select (an existing
/// settlement's, or the newly-appended one's) or `None` for every refusal
/// (`DropPlace::OutOfBounds`/`DropPlace::Water`) — `lib.rs` reports `None`
/// as `-1`, matching the rest of this crate's index-or-`-1` convention
/// (`explain_settlement`, etc).
#[allow(clippy::too_many_arguments)]
pub fn drop_settlement(
    settlements: &mut Vec<NamedSettlement>,
    next_tid: &mut u64,
    name_rng: &mut cartalith_rng::Mulberry32,
    gx: usize,
    gy: usize,
    pick_r: f64,
    field: &[f32],
    water_bodies: &[u8],
    gw: usize,
    gh: usize,
    sea: f64,
    faction: i32,
    kind: SettlementKind,
    name: &str,
) -> Option<usize> {
    match civ_drop_place(settlements, gx, gy, pick_r, field, water_bodies, gw, gh, sea, faction, kind, 0.0) {
        DropPlace::Selected(i) => Some(i),
        DropPlace::Placed(mut s) => {
            s.name = manual_settlement_name(name, faction, name_rng);
            s.pop = manual_settlement_pop(kind, 0.0, name_rng);
            // `TIMELINE_SCOPE.md` milestone 1: a hand-placed settlement is
            // exactly the "placement time" this port's `tid` design assigns
            // at -- see `cartalith_civ::timeline`'s module doc.
            s.tid = cartalith_civ::timeline::civ_assign_tid(s.tid, next_tid);
            settlements.push(*s);
            Some(settlements.len() - 1)
        }
        DropPlace::OutOfBounds | DropPlace::Water => None,
    }
}

/// A claimed cell counts as contested when it borders (4-connected) a
/// *different* nonzero faction's claimed cell. See this module's own doc
/// comment: not a reference or engine concept, this bridge's own heuristic.
///
/// `faction == 0` ("Unclaimed", `assign_territory`'s own sentinel, never a
/// real assignable id -- `FactionAggregatesInput`'s own doc comment: index
/// 0 "never accumulates territory") always answers `0` rather than counting
/// unclaimed cells that happen to border a claimed one -- "how contested is
/// the unclaimed pool" is a category error this function refuses to
/// dignify with a number, the same way `assign_territory` itself never
/// treats `0` as an owner.
pub fn contested_cell_count(territory: &[i32], faction: i32, gw: usize, gh: usize) -> usize {
    if faction == 0 || gw == 0 || gh == 0 {
        return 0;
    }
    let mut n = 0;
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if territory[i] != faction {
                continue;
            }
            let mut bordered = false;
            if x > 0 {
                bordered |= is_other_faction(territory[i - 1], faction);
            }
            if x + 1 < gw {
                bordered |= is_other_faction(territory[i + 1], faction);
            }
            if y > 0 {
                bordered |= is_other_faction(territory[i - gw], faction);
            }
            if y + 1 < gh {
                bordered |= is_other_faction(territory[i + gw], faction);
            }
            if bordered {
                n += 1;
            }
        }
    }
    n
}

fn is_other_faction(cell: i32, faction: i32) -> bool {
    cell != 0 && cell != faction
}

/// The live CIVIL-tool-group state for one generated world: the territory
/// paint draft/accumulator pair, and the manual-placement name/population
/// RNG stream. See this module's own doc comment for why each exists.
pub struct CivTools {
    /// One in-progress territory stroke, not yet baked in.
    pub territory_draft: PassBuffer<PaintStamp>,
    pub territory_tracker: DirtyTracker,
    /// Every committed stroke this session, accumulated (not just the most
    /// recent commit) — see the module doc's "needs one more thing besides"
    /// section for why a single-stroke draft is not enough on its own to
    /// make subtract restore the computed base.
    pub territory_paint: PaintLayer,
    /// `assign_territory`'s own output, captured once in `WorldGen::absorb`
    /// before any paint touches it. Never mutated after that.
    pub territory_base: Vec<i32>,
    /// The manual-placement name/population stream, seeded once per world.
    /// See the module doc's RNG section.
    pub name_rng: cartalith_rng::Mulberry32,
}

impl CivTools {
    /// `gw x gh` must match the world's own grid — `territory_base.len()`
    /// is asserted against it defensively (`compute_civilisation` always
    /// produces a full-length `territory`, but this guards the same
    /// resolution-mismatch class of bug `PaintLayer::cells_mut`'s own
    /// reallocate-on-mismatch guard exists for).
    pub fn new(gw: usize, gh: usize, territory_base: Vec<i32>, seed: u32) -> Self {
        debug_assert_eq!(territory_base.len(), gw * gh, "territory_base must be gw*gh -- caller's own assign_territory output");
        let draft = PassBuffer::new(gw, gh, TERRITORY_TILE_SIZE);
        let tracker = DirtyTracker::new(draft.tile_count());
        // `civ_name_rng()` is a pure function of a fixed constant -- calling
        // it fresh per placement would repeat one name forever (see the
        // module doc). `seed` folds this world's own generation seed in so
        // two different worlds' manual-placement streams diverge too,
        // rather than every world's first hand-placed settlement sharing
        // one fixed name.
        let mut name_rng = cartalith_civ::civ_name_rng();
        for _ in 0..(seed % 17) {
            name_rng.next_f64();
        }
        Self { territory_draft: draft, territory_tracker: tracker, territory_paint: PaintLayer::new(), territory_base, name_rng }
    }

    /// One dab, `subtract` selecting erase (`value = 0`, "fall through to
    /// the computed base") over paint (`value = faction`).
    pub fn paint_at(&mut self, gx: f64, gy: f64, faction: i32, radius: f64, subtract: bool) {
        let value = if subtract { 0u8 } else { faction.clamp(0, u8::MAX as i32) as u8 };
        self.territory_draft.push(PaintStamp::ungated(gx.round() as i64, gy.round() as i64, radius, value));
    }

    /// Bakes the in-progress draft into `territory_paint`, then rebuilds
    /// `territory` in place from `territory_base` merged with the *whole*
    /// accumulated `territory_paint` -- not just this stroke. Returns
    /// `false` (a no-op) when there is nothing to commit, so a caller can
    /// skip the O(gw*gh) rebuild on a spurious commit with no dabs pending.
    pub fn commit(&mut self, territory: &mut Vec<i32>) -> bool {
        if self.territory_draft.is_empty() {
            return false;
        }
        let n = self.territory_base.len();
        self.territory_draft.commit(self.territory_paint.cells_mut(n), &mut self.territory_tracker, "territory_painted");
        let mut rebuilt = self.territory_base.clone();
        if let Some(cells) = self.territory_paint.cells() {
            cartalith_civ::tools::merge_territory_paint(&mut rebuilt, cells);
        }
        *territory = rebuilt;
        true
    }

    /// Drops the in-progress stroke. `territory_paint`/`territory` are
    /// untouched -- discard only ever affects what has not been committed.
    pub fn discard(&mut self) {
        self.territory_draft.discard();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- kind_from_str ----------

    #[test]
    fn kind_from_str_accepts_the_five_real_tiers_case_insensitively() {
        assert_eq!(kind_from_str("Capital"), Some(SettlementKind::Capital));
        assert_eq!(kind_from_str("CITY"), Some(SettlementKind::City));
        assert_eq!(kind_from_str("town"), Some(SettlementKind::Town));
        assert_eq!(kind_from_str("Village"), Some(SettlementKind::Village));
        assert_eq!(kind_from_str("hamlet"), Some(SettlementKind::Hamlet));
    }

    #[test]
    fn kind_from_str_rejects_metropolis_and_garbage() {
        assert_eq!(kind_from_str("metropolis"), None, "a real DCC_SHELL_SPEC option, not a real engine tier -- see the module doc");
        assert_eq!(kind_from_str("monastery"), None);
        assert_eq!(kind_from_str(""), None);
    }

    // ---------- nearest_land_cell ----------

    fn strip_world(w: usize, h: usize, land_cols: std::ops::Range<usize>) -> (Vec<f32>, Vec<u8>) {
        let mut field = vec![0.2f32; w * h];
        let mut wb = vec![1u8; w * h];
        for y in 0..h {
            for x in land_cols.clone() {
                field[y * w + x] = 0.6;
                wb[y * w + x] = 0;
            }
        }
        (field, wb)
    }

    #[test]
    fn nearest_land_cell_returns_the_same_cell_when_already_land() {
        let (field, wb) = strip_world(16, 16, 6..10);
        assert_eq!(nearest_land_cell(8, 8, 16, 16, &field, &wb, 0.42, 5.0), Some((8, 8)));
    }

    #[test]
    fn nearest_land_cell_finds_the_closest_shore_cell() {
        let (field, wb) = strip_world(16, 16, 6..10);
        // (2, 8) is water; the nearest land is the strip's left edge at x=6.
        assert_eq!(nearest_land_cell(2, 8, 16, 16, &field, &wb, 0.42, 10.0), Some((6, 8)));
    }

    #[test]
    fn nearest_land_cell_returns_none_when_nothing_is_in_range() {
        let (field, wb) = strip_world(16, 16, 6..10);
        assert_eq!(nearest_land_cell(0, 0, 16, 16, &field, &wb, 0.42, 2.0), None);
    }

    // ---------- manual naming / population ----------

    #[test]
    fn manual_settlement_name_uses_the_given_name_verbatim_when_not_blank() {
        let mut rng = cartalith_civ::civ_name_rng();
        assert_eq!(manual_settlement_name("Port Callis", 1, &mut rng), "Port Callis");
    }

    #[test]
    fn manual_settlement_name_generates_one_when_blank_or_whitespace() {
        let mut rng = cartalith_civ::civ_name_rng();
        let a = manual_settlement_name("", 1, &mut rng);
        let b = manual_settlement_name("   ", 1, &mut rng);
        assert!(!a.is_empty());
        assert!(!b.is_empty());
        assert_ne!(a, b, "the stream must have advanced between the two calls, not repeated");
    }

    #[test]
    fn manual_settlement_pop_matches_the_auto_populate_formula() {
        let mut rng_a = cartalith_civ::civ_name_rng();
        let mut rng_b = cartalith_civ::civ_name_rng();
        let expected = (civ_base_pop_for_kind(SettlementKind::Town) * 0.7 * (0.8 + rng_a.next_f64() * 0.4)).round() as u32;
        let got = manual_settlement_pop(SettlementKind::Town, 0.0, &mut rng_b);
        assert_eq!(got, expected);
    }

    // ---------- drop_settlement ----------

    fn drop_fixture() -> (Vec<NamedSettlement>, Vec<f32>, Vec<u8>, usize, usize, f64) {
        let (gw, gh) = (8usize, 8usize);
        let mut field = vec![0.6f32; gw * gh];
        let mut wb = vec![0u8; gw * gh];
        for y in 0..gh {
            for x in 0..3 {
                field[y * gw + x] = 0.2;
                wb[y * gw + x] = 1;
            }
        }
        (Vec::new(), field, wb, gw, gh, 0.42)
    }

    #[test]
    fn drop_settlement_appends_and_names_a_blank_placement() {
        let (mut places, field, wb, gw, gh, sea) = drop_fixture();
        let mut rng = cartalith_civ::civ_name_rng();
        let mut next_tid = 1u64;
        let idx = super::drop_settlement(&mut places, &mut next_tid, &mut rng, 5, 2, 5.0, &field, &wb, gw, gh, sea, 2, SettlementKind::Town, "");
        assert_eq!(idx, Some(0));
        assert_eq!(places.len(), 1);
        assert!(!places[0].name.is_empty());
        assert_eq!(places[0].placement.faction, 2);
    }

    #[test]
    fn drop_settlement_refuses_water_and_out_of_bounds() {
        let (mut places, field, wb, gw, gh, sea) = drop_fixture();
        let mut rng = cartalith_civ::civ_name_rng();
        let mut next_tid = 1u64;
        assert_eq!(super::drop_settlement(&mut places, &mut next_tid, &mut rng, 1, 1, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, ""), None);
        assert_eq!(super::drop_settlement(&mut places, &mut next_tid, &mut rng, 99, 1, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, ""), None);
        assert!(places.is_empty());
    }

    #[test]
    fn drop_settlement_selects_an_existing_place_instead_of_stacking() {
        let (mut places, field, wb, gw, gh, sea) = drop_fixture();
        let mut rng = cartalith_civ::civ_name_rng();
        let mut next_tid = 1u64;
        let first = super::drop_settlement(&mut places, &mut next_tid, &mut rng, 5, 2, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, "First").unwrap();
        let second = super::drop_settlement(&mut places, &mut next_tid, &mut rng, 5, 2, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, "");
        assert_eq!(second, Some(first));
        assert_eq!(places.len(), 1, "clicking an existing place must not stack a second settlement");
    }

    /// `TIMELINE_SCOPE.md` milestone 1: a hand-placed settlement gets a
    /// real, nonzero, monotonically-increasing `tid` from the shared
    /// counter -- not the crate's `0` "unassigned" sentinel -- and re-
    /// clicking an existing settlement (the `Selected` branch) never
    /// consumes the counter.
    #[test]
    fn drop_settlement_assigns_a_real_tid_and_reuses_the_counter_across_drops() {
        let (mut places, field, wb, gw, gh, sea) = drop_fixture();
        let mut rng = cartalith_civ::civ_name_rng();
        let mut next_tid = 1u64;
        let first = super::drop_settlement(&mut places, &mut next_tid, &mut rng, 5, 2, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, "First").unwrap();
        assert_eq!(places[first].tid, 1);
        assert_eq!(next_tid, 2);
        let second = super::drop_settlement(&mut places, &mut next_tid, &mut rng, 7, 2, 1.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Village, "Second").unwrap();
        assert_eq!(places[second].tid, 2);
        assert_eq!(next_tid, 3);
        // Re-clicking the first settlement selects it (no new tid drawn).
        let reselect = super::drop_settlement(&mut places, &mut next_tid, &mut rng, 5, 2, 5.0, &field, &wb, gw, gh, sea, 1, SettlementKind::Town, "");
        assert_eq!(reselect, Some(first));
        assert_eq!(next_tid, 3, "reselecting an existing settlement must not advance the counter");
    }

    // ---------- contested_cell_count ----------

    #[test]
    fn contested_counts_only_cells_bordering_a_different_faction() {
        // 3x1: faction 1, faction 2, unclaimed.
        let territory = vec![1i32, 2, 0];
        assert_eq!(contested_cell_count(&territory, 1, 3, 1), 1);
        assert_eq!(contested_cell_count(&territory, 2, 3, 1), 1);
    }

    #[test]
    fn contested_ignores_unclaimed_neighbours() {
        // 3x1: faction 1, unclaimed, faction 1 -- neither claimed cell
        // touches a *different* faction, only open ground.
        let territory = vec![1i32, 0, 1];
        assert_eq!(contested_cell_count(&territory, 1, 3, 1), 0);
    }

    #[test]
    fn contested_cell_count_of_an_unclaimed_faction_id_is_zero() {
        let territory = vec![1i32, 2, 3, 0];
        assert_eq!(contested_cell_count(&territory, 0, 4, 1), 0);
    }

    // ---------- CivTools: territory draft/commit/discard ----------

    #[test]
    fn new_civ_tools_has_no_pending_draft_and_the_given_base() {
        let base = vec![0i32; 16];
        let tools = CivTools::new(4, 4, base.clone(), 7);
        assert!(tools.territory_draft.is_empty());
        assert!(tools.territory_paint.is_unallocated());
        assert_eq!(tools.territory_base, base);
    }

    #[test]
    fn discard_leaves_the_committed_territory_untouched() {
        let base = vec![0i32; 16];
        let mut tools = CivTools::new(4, 4, base.clone(), 1);
        tools.paint_at(1.0, 1.0, 3, 1.0, false);
        tools.discard();
        assert!(tools.territory_draft.is_empty());
        let mut territory = base.clone();
        assert!(!tools.commit(&mut territory), "nothing left to commit after a discard");
        assert_eq!(territory, base);
    }

    #[test]
    fn commit_paints_over_the_base_and_leaves_unpainted_cells_alone() {
        let base = vec![0i32; 16]; // 4x4, all unclaimed
        let mut tools = CivTools::new(4, 4, base.clone(), 1);
        tools.paint_at(1.0, 1.0, 5, 0.0, false); // faction 5, radius 0 -> exactly cell (1,1)
        let mut territory = base.clone();
        assert!(tools.commit(&mut territory));
        assert_eq!(territory[1 * 4 + 1], 5);
        assert_eq!(territory[0], 0, "cells outside the dab fall through to the base");
    }

    #[test]
    fn a_later_subtract_restores_the_base_even_across_two_commits() {
        let base = vec![9i32; 16]; // every cell pre-claimed by faction 9
        let mut tools = CivTools::new(4, 4, base.clone(), 1);
        tools.paint_at(1.0, 1.0, 3, 0.0, false); // faction 3 paints (1,1)
        let mut territory = base.clone();
        tools.commit(&mut territory);
        assert_eq!(territory[1 * 4 + 1], 3);

        // Second stroke, subtract at the same cell: must restore the
        // BASE's faction 9, not a bare 0 -- the whole reason this module
        // keeps `territory_base` instead of merging directly into the live
        // `territory` array every commit.
        tools.paint_at(1.0, 1.0, 3, 0.0, true);
        tools.commit(&mut territory);
        assert_eq!(territory[1 * 4 + 1], 9, "subtract must fall through to the computed base, not to unclaimed");
    }
}
