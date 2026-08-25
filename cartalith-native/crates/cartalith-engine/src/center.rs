//! `centerLandmasses()` (reference HTML lines 3179-3199) — the WORLD
//! tool-options bar's *Center landmasses* action.
//!
//! Orchestration only: the three kernels it composes are
//! `cartalith_terrain::center`'s, golden-verified in
//! `golden_parity_center.rs`. This module's job is knowing *which* of
//! [`WorldState`]'s grids move and which are invalidated.

use crate::WorldState;
use cartalith_terrain::center::{best_empty_column, feather_seam_x, seam_column, shift_grid_x};

/// What one `center_landmasses` call did, so a caller can report it
/// honestly instead of assuming it worked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CenterResult {
    /// Columns the world was rotated by. `0` means the emptiest meridian
    /// was already at the edge and **nothing was touched** — the
    /// reference's own early return.
    pub offset: usize,
    /// Where the old `x=0 ↔ x=W-1` join now sits, and where the feather
    /// was applied. Meaningless when `offset == 0`.
    pub seam_column: usize,
    /// Whether the single-receiver channel tree was dropped (see below).
    pub channels_dropped: bool,
}

/// Rotate the wrapped world in X so the land sits away from the seam.
///
/// **Only meaningful in world (cylinder) mode.** In region mode the edges
/// are hard borders and there is nothing to re-center; the reference
/// `alert()`s and returns, and this returns `None` for the caller to say
/// so in whatever way suits it.
///
/// Returns `None` for a non-world state or a degenerate grid, and
/// `Some(CenterResult { offset: 0, .. })` when the world already needed no
/// centering.
///
/// # What moves, and what does not
///
/// Every **positional raster** on `WorldState` is shifted, which is a
/// larger set than the reference's own list only because this port retains
/// a larger set (`crust_field`, `boundary_type`, `shear_field` and
/// `stream_order` have no retained counterpart in the
/// reference, which recomputes or nulls them). The reference's own
/// `warpX`/`warpY`/`geoidField`/`tideField`/`koppenField`/`orogenyField`
/// and its four seasonal fields have no equivalent here at all.
///
/// `channels` is **dropped**, not shifted: `ChannelResult::recv` holds
/// flat grid *indices*, so rotating the array leaves every receiver
/// pointing at the cell it used to point at rather than the one it now
/// means. The reference does exactly this, via `_riverNet = null`.
///
/// # What this deliberately does not do
///
/// The civilisation layer is not touched. Settlement, way and route
/// coordinates live outside `WorldState`, and the reference does not shift
/// them either — `centerLandmasses` is a *pre-civilisation* action there
/// as well as here. A caller that already has a civ layer should discard
/// it rather than let it drift; `cartalith-godot`'s binding does.
pub fn center_landmasses(ws: &mut WorldState, gw: usize, gh: usize, world: bool) -> Option<CenterResult> {
    if !world || gw == 0 || gh == 0 || ws.field.len() != gw * gh {
        return None;
    }
    // `geoidField` is `None` throughout this port (no geoid exists --
    // `GUI_GAP_REGISTER.md` WW-07), which is what the reference's own
    // `geo?geo[i]:0` reduces to when the field is off.
    let off = best_empty_column(&ws.field, None, gw, gh, ws.sea_level);
    if off == 0 {
        return Some(CenterResult { offset: 0, seam_column: 0, channels_dropped: false });
    }
    let o = off as isize;

    for a in [
        &mut ws.field,
        &mut ws.stress_field,
        &mut ws.age_field,
        &mut ws.resistance_field,
        &mut ws.crust_field,
        &mut ws.shear_field,
        &mut ws.volcanic_field,
        &mut ws.impact_field,
        &mut ws.temperature,
        &mut ws.rainfall,
        &mut ws.flow_discharge,
    ] {
        shift_grid_x(a.as_mut_slice(), gw, gh, o);
    }
    shift_grid_x(ws.plate_id.as_mut_slice(), gw, gh, o);
    for a in [&mut ws.boundary_mask, &mut ws.boundary_type] {
        shift_grid_x(a.as_mut_slice(), gw, gh, o);
    }
    if let Some(a) = ws.stream_order.as_mut() {
        shift_grid_x(a.as_mut_slice(), gw, gh, o);
    }
    if let Some(a) = ws.river_mask.as_mut() {
        shift_grid_x(a.as_mut_slice(), gw, gh, o);
    }
    if let Some(a) = ws.river_floor.as_mut() {
        shift_grid_x(a.as_mut_slice(), gw, gh, o);
    }
    let channels_dropped = ws.channels.take().is_some();

    // The world is only *approximately* periodic in X (the reference's own
    // Invariant 9: seam wrap-delta < 0.12), so the shift moves that
    // original join into the interior where it reads as a straight
    // vertical line. Feather it away -- the same four fields the reference
    // feathers, minus the geoid this port does not have.
    let sc = seam_column(gw, off);
    for a in [&mut ws.field, &mut ws.temperature, &mut ws.rainfall] {
        feather_seam_x(a.as_mut_slice(), gw, gh, sc, 2);
    }

    Some(CenterResult { offset: off, seam_column: sc, channels_dropped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorldParams, generate_terrain};

    fn world(gw: usize, gh: usize, seed: i32) -> (WorldState, usize, usize) {
        let mut p = WorldParams::defaults(gw, gh, seed);
        p.world = true;
        p.map_width_km = 4000.0;
        (generate_terrain(&p), gw, gh)
    }

    #[test]
    fn region_mode_is_refused_rather_than_silently_rotated() {
        let (mut ws, gw, gh) = world(32, 24, 7);
        let before = ws.field.clone();
        assert_eq!(center_landmasses(&mut ws, gw, gh, false), None);
        assert_eq!(ws.field, before, "a refused call must not have moved anything");
    }

    #[test]
    fn every_retained_raster_moves_together() {
        let (mut ws, gw, gh) = world(48, 32, 24601);
        let before_field = ws.field.clone();
        let before_temp = ws.temperature.clone();
        let before_plates = ws.plate_id.clone();

        let r = center_landmasses(&mut ws, gw, gh, true).expect("a world-mode call should run");
        assert_ne!(r.offset, 0, "this seed must actually need centering for the test to say anything");

        // The plate partition is a pure permutation -- no feather touches
        // it -- so it must be exactly the rotation, cell for cell.
        for y in 0..gh {
            for x in 0..gw {
                assert_eq!(ws.plate_id[y * gw + x], before_plates[y * gw + (x + r.offset) % gw]);
            }
        }
        assert_ne!(ws.field, before_field);
        assert_ne!(ws.temperature, before_temp);
        // Away from the feathered band, height is the same permutation.
        let far = (r.seam_column + gw / 2) % gw;
        assert_eq!(ws.field[far], before_field[(far + r.offset) % gw]);
    }

    #[test]
    fn the_receiver_tree_is_dropped_because_its_indices_would_be_wrong() {
        let (mut ws, gw, gh) = world(48, 32, 24601);
        assert!(ws.channels.is_some(), "the fixture must have a channel tree to drop");
        let r = center_landmasses(&mut ws, gw, gh, true).unwrap();
        assert!(r.channels_dropped);
        assert!(ws.channels.is_none());
    }

    /// Centering twice must be a no-op the second time: the first call put
    /// the emptiest meridian at the edge, so `bestEmptyColumn` returns 0.
    #[test]
    fn a_second_call_finds_nothing_left_to_do() {
        let (mut ws, gw, gh) = world(48, 32, 24601);
        assert_ne!(center_landmasses(&mut ws, gw, gh, true).unwrap().offset, 0);
        let after = ws.field.clone();
        let second = center_landmasses(&mut ws, gw, gh, true).unwrap();
        assert_eq!(second.offset, 0);
        assert_eq!(ws.field, after, "an offset of 0 must touch nothing at all");
    }
}
