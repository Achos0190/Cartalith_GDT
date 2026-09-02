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
/// `channels` is **shifted and re-pointed**, like everything else.
/// `ChannelResult::recv` holds flat grid *indices*, so rotating the array
/// alone would leave every receiver aimed at the cell it used to mean --
/// which is why this used to drop the tree instead. Dropping it was wrong:
/// `ws.channels` is written only by `generate_terrain`, so nothing ever put
/// it back, and every river left the map, the exported rasters and the
/// GeoJSON export until the next Generate (measured: 137 river features
/// before centring, 0 after). The reference gets away with `_riverNet = null`
/// because `renderNow` rebuilds it on the next draw; this port has no such
/// rebuild, and one would not be faithful anyway -- see the body.
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
    // **The channel network is shifted, not dropped.** It used to be
    // `ws.channels.take()`, and since `ws.channels` is written in exactly one
    // place -- `generate_terrain` -- nothing ever put it back: every river
    // disappeared from the map, from the exported rasters and from the GeoJSON
    // export the moment the user pressed Center landmasses, until the next
    // Generate. Measured before the fix on seed 24601 in world mode
    // (offset 86): **137 river features before, 0 after**.
    //
    // The reference does not behave that way. `centerLandmasses` nulls
    // `_riverNet`, and `renderNow`'s own branch rebuilds it on the next draw
    // (`if(!_riverNet) _riverNet = buildRiverNetwork(...)`). This port has no
    // such rebuild, and adding one here would not be equivalent anyway:
    // `build_channels` is fed `flow_for_network` and the **pre-carve** field
    // at generation time, and neither survives into `WorldState`, so a rebuild
    // from `ws.field`/`ws.flow_discharge` would invent a *different* network
    // rather than restore the one that was there.
    //
    // Shifting preserves the exact topology, which is what every other grid
    // here does. `recv` needs one thing the value grids do not: its elements
    // are **cell indices**, so after the array moves, each stored index must
    // be re-pointed at its subject's new column. `shift_grid_x` writes
    // `new[x] = old[(x + off) % w]`, so a cell that was at column `cx` is now
    // at `(cx - off) mod w`. `-1` is the "no receiver" sentinel and is left
    // alone. `slope` is released by `generate_terrain` immediately after the
    // build and is empty here, so it is not shifted.
    if let Some(ch) = ws.channels.as_mut() {
        shift_grid_x(ch.chan.as_mut_slice(), gw, gh, o);
        shift_grid_x(ch.recv.as_mut_slice(), gw, gh, o);
        // The stamped width raster is an ordinary value grid. Empty on a
        // loaded save, and `shift_grid_x` no-ops on an empty slice.
        if ch.intensity.len() == gw * gh {
            shift_grid_x(ch.intensity.as_mut_slice(), gw, gh, o);
        }
        let back = gw - (off % gw);
        for v in ch.recv.iter_mut() {
            if *v < 0 {
                continue;
            }
            let idx = *v as usize;
            *v = ((idx / gw) * gw + (idx % gw + back) % gw) as i32;
        }
    }
    let channels_dropped = false;

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

    /// Replaces `the_receiver_tree_is_dropped_because_its_indices_would_be_
    /// wrong`, which asserted the old behaviour and named its own cause in its
    /// own title: the indices *would* have been wrong, so the tree was thrown
    /// away instead of being re-pointed. Dropping it meant every river left
    /// the map, the exported rasters and the GeoJSON export until the next
    /// Generate -- measured at 137 river features before centring and 0 after.
    /// Re-pointing them is what this pins.
    #[test]
    fn the_receiver_tree_moves_with_the_world_instead_of_being_dropped() {
        let (mut ws, gw, gh) = world(48, 32, 24601);
        // The two vectors, not the struct: `ChannelResult` is not `Clone`, and
        // deriving it on a shipped type to satisfy a test is the wrong way round.
        let (before_chan, before_recv) = {
            let c = ws.channels.as_ref().expect("the fixture must have a channel tree");
            (c.chan.clone(), c.recv.clone())
        };
        let channel_cells = before_chan.iter().filter(|&&c| c != 0).count();
        assert!(channel_cells > 0, "the fixture must actually contain channels");

        let r = center_landmasses(&mut ws, gw, gh, true).unwrap();
        assert_ne!(r.offset, 0, "this seed must actually shift or the test says nothing");
        assert!(!r.channels_dropped);
        let after = ws.channels.as_ref().expect("the channel tree must survive centring");

        // Where a cell ends up: `shift_grid_x` writes `new[x] = old[(x+off)%w]`,
        // so the cell at old column `cx` lands at `(cx - off) mod gw`.
        let moved = |idx: usize| -> usize {
            (idx / gw) * gw + (idx % gw + gw - r.offset % gw) % gw
        };

        assert_eq!(
            after.chan.iter().filter(|&&c| c != 0).count(),
            channel_cells,
            "centring must not gain or lose a single channel cell"
        );

        let mut checked = 0usize;
        for i in 0..gw * gh {
            assert_eq!(after.chan[moved(i)], before_chan[i], "chan at {i} did not move intact");
            let old_recv = before_recv[i];
            let new_recv = after.recv[moved(i)];
            if old_recv < 0 {
                assert_eq!(new_recv, -1, "the no-receiver sentinel at {i} must survive as -1");
            } else {
                assert_eq!(
                    new_recv as usize,
                    moved(old_recv as usize),
                    "the receiver of cell {i} must point at where its receiver actually went"
                );
                checked += 1;
            }
        }
        assert!(checked > 0, "no cell had a receiver -- this test proved nothing");
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
