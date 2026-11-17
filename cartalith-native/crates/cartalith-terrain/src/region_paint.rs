//! Colour paint inside a painted region, for the assistant's `paint` edit operation (Ruling FW item 2,
//! owner 2026-11-17; owner question E11's default: Colour and Multiply, a closed list of colours).
//! Design: `LLM_ASSISTANT_SCOPE.md` s.5i.
//!
//! ## What it builds
//!
//! One `Area` mark (`cartalith_spatial::colour_paint::MarkShape::Area`) per 4-connected piece of the
//! region, its ring the piece's outline traced along cell EDGES ([`crate::region_edit::mask_rings`]) and
//! scaled to map kilometres (`km = grid edge coordinate x km_per_cell`, the colour-paint document's own
//! convention), `edge_km` 0 (a hard edge, antialiased by the renderer over one output pixel). Sampled at a
//! cell's centre, which is half a cell from every ring edge, the mark's coverage is exactly 1 inside the
//! region and exactly 0 outside it at a pixel footprint of one cell or finer, so **a cell outside the
//! painted region is returned bit for bit as it came in** (`tests` assert it over a world, with a notch).
//! Deeper zoom changes only the edge's width, never where it is.
//!
//! ## Why not the click-seeded fill helper
//!
//! The CP-5 fill (`colour_paint::fill`) thins a long ring "by taking every k-th point", which moves the
//! edge; that is fine for "fill the crater" and would spill past the painted cells here. This module
//! refuses a ring over `MAX_RING_POINTS` instead.
//!
//! The marks are ordinary Ruling CN marks: the shell adds them with the colour-paint tool's own
//! `commit_marks`, so they save in `appearance_paint.json`, composite before the light, and undo on the
//! tool's own stack (the assistant's undo group collects that step).
//!
//! MUST NEVER: paint a cell outside the region; thin or snap a ring; invent a scale (no km per cell, no
//! paint); be reachable from the seed-deterministic path.

use cartalith_spatial::colour_paint::{ColourBlend, ColourMark, ColourPaint, MarkShape, PaintLayers, PaintQuery};

use crate::region_edit::{EditError, RegionMask, mask_rings};

/// The marks that paint exactly `mask` in `rgb` with `blend` at `opacity`. Always `land_only` (the
/// colour-paint tool's own default; water keeps its colour). `Err` when `km_per_cell` is not a positive
/// finite number (no scale, no paint) or the outline cannot be one exact ring per piece
/// ([`mask_rings`]'s refusals, in words).
pub fn paint_marks(mask: &RegionMask, rgb: [u8; 3], blend: ColourBlend, opacity: f64, km_per_cell: f64) -> Result<Vec<ColourMark>, EditError> {
    if !(km_per_cell.is_finite() && km_per_cell > 0.0) {
        return Err(EditError::Ops("this world has no map width, so a colour in kilometres cannot be placed".into()));
    }
    if !(opacity.is_finite() && (0.0..=1.0).contains(&opacity)) {
        return Err(EditError::Ops("the opacity must be a number from 0 to 1".into()));
    }
    let rings = mask_rings(mask)?;
    Ok(rings
        .into_iter()
        .map(|ring| ColourMark {
            shape: MarkShape::Area { ring_km: ring.into_iter().map(|p| [p[0] * km_per_cell, p[1] * km_per_cell]).collect(), edge_km: 0.0 },
            rgb,
            opacity,
            hardness: 1.0,
            blend,
            land_only: true,
            emissive: None,
            strength: None,
        })
        .collect())
}

/// A colour-paint layer holding exactly `marks` (one default layer), built through the engine's own
/// constructor so the caps and value checks are the real ones. `margin_km` is what the renderer passes:
/// two grid cells in km.
pub fn paint_layer(marks: Vec<ColourMark>, km_per_cell: f64) -> Result<ColourPaint, EditError> {
    let mut layers = PaintLayers::empty();
    for m in marks {
        layers.add_mark(0, m).map_err(EditError::Ops)?;
    }
    ColourPaint::from_layers(layers, 2.0 * km_per_cell).map_err(|e| EditError::Ops(e.to_string()))
}

/// The colour of cell `(x, y)`'s centre after `paint`, given the colour `c` beneath it (0-255 per
/// channel), at the one-cell-per-pixel footprint of the map texture. `water` is the cell's own water
/// flag: land-only marks leave water as it is.
pub fn paint_cell(paint: &ColourPaint, x: usize, y: usize, km_per_cell: f64, water: bool, c: [f64; 3]) -> [f64; 3] {
    paint.apply(c, PaintQuery { x_km: (x as f64 + 0.5) * km_per_cell, y_km: (y as f64 + 0.5) * km_per_cell, px_km: km_per_cell, water })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn art(x0: usize, y0: usize, rows: &[&str]) -> RegionMask {
        let (w, h) = (rows[0].len(), rows.len());
        let cells: Vec<bool> = rows.iter().flat_map(|r| r.bytes().map(|b| b == b'#')).collect();
        let count = cells.iter().filter(|c| **c).count();
        RegionMask { x0, y0, w, h, cells, count, clipped: false, path: vec![], path_r: 0.0, boxed: false }
    }

    const GREY: [f64; 3] = [120.0, 130.0, 140.0];

    fn inside(m: &RegionMask, x: usize, y: usize) -> bool {
        x >= m.x0 && y >= m.y0 && x < m.x0 + m.w && y < m.y0 + m.h && m.cells[(y - m.y0) * m.w + (x - m.x0)]
    }

    /// Protects Ruling FW item 2's bound: over a 40 x 30 world, a literal mask with a notch and two pieces,
    /// every cell centre OUTSIDE the mask returns the backdrop's exact bits and every cell INSIDE changes,
    /// for both offered blends, at a footprint of one cell and of a quarter of a cell (deeper zoom). The
    /// kilometre scale is 2.5 km per cell so a unit slip (cells for km) fails. MUTATION: trace the ring one
    /// cell wider, drop the `* km_per_cell`, or make `edge_km` positive.
    #[test]
    fn colour_paint_never_spills_past_a_literal_mask() {
        let kpc = 2.5;
        // A notch open at the top (columns 3-4), joined along the bottom, plus a separate 4 x 2 block.
        let rows = ["###..###....####", "###..###....####", "###..###........", "########........", "########........"];
        let m = art(6, 5, &rows);
        assert_eq!(m.count, 42, "34 cells in the notched piece and 8 in the block");
        for blend in [ColourBlend::Colour, ColourBlend::Multiply] {
            let marks = paint_marks(&m, [200, 40, 40], blend, 1.0, kpc).unwrap();
            assert_eq!(marks.len(), 2, "two 4-connected pieces, so two marks");
            let layer = paint_layer(marks, kpc).unwrap();
            for footprint in [1.0, 0.25] {
                let (mut inside_changed, mut inside_n) = (0, 0);
                for y in 0..30 {
                    for x in 0..40 {
                        let q = PaintQuery { x_km: (x as f64 + 0.5) * kpc, y_km: (y as f64 + 0.5) * kpc, px_km: kpc * footprint, water: false };
                        let out = layer.apply(GREY, q);
                        if inside(&m, x, y) {
                            inside_n += 1;
                            inside_changed += usize::from(out != GREY);
                        } else {
                            assert_eq!(out.map(f64::to_bits), GREY.map(f64::to_bits), "{blend:?} @{footprint}: the cell ({x},{y}) OUTSIDE the mask changed");
                        }
                    }
                }
                assert_eq!((inside_changed, inside_n), (m.count, m.count), "{blend:?} @{footprint}: every inside cell is painted");
            }
        }
    }

    /// Protects the land-only default: a water cell inside the region keeps its colour. MUTATION: set
    /// `land_only` to false.
    #[test]
    fn water_inside_the_region_keeps_its_colour() {
        let m = art(0, 0, &["####", "####", "####", "####"]);
        let layer = paint_layer(paint_marks(&m, [200, 40, 40], ColourBlend::Multiply, 1.0, 1.0).unwrap(), 1.0).unwrap();
        assert_eq!(paint_cell(&layer, 1, 1, 1.0, true, GREY), GREY);
        assert_ne!(paint_cell(&layer, 1, 1, 1.0, false, GREY), GREY);
    }

    /// Protects the literal ring and its kilometres: a 3 x 2 block at (5, 7) with 2 km cells is the rectangle
    /// (10, 14) to (16, 18). MUTATION: change the scaling or the offset.
    #[test]
    fn the_ring_is_in_kilometres() {
        let m = art(5, 7, &["###", "###"]);
        let marks = paint_marks(&m, [1, 2, 3], ColourBlend::Colour, 0.5, 2.0).unwrap();
        let MarkShape::Area { ring_km, edge_km } = &marks[0].shape else { panic!("area") };
        assert_eq!((ring_km.as_slice(), *edge_km), ([[10.0, 14.0], [16.0, 14.0], [16.0, 18.0], [10.0, 18.0]].as_slice(), 0.0));
        assert_eq!((marks[0].rgb, marks[0].opacity, marks[0].land_only), ([1, 2, 3], 0.5, true));
    }

    /// Protects the refusals: no map width, a hole, a bad opacity. Each is an error in words, never a default.
    #[test]
    fn no_scale_a_hole_and_a_bad_opacity_are_refused() {
        let m = art(0, 0, &["###", "###", "###"]);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(paint_marks(&m, [1, 2, 3], ColourBlend::Colour, 1.0, bad).unwrap_err().to_string().contains("map width"));
        }
        for bad in [-0.1, 1.1, f64::NAN] {
            assert!(paint_marks(&m, [1, 2, 3], ColourBlend::Colour, bad, 1.0).is_err());
        }
        let holed = art(0, 0, &["###", "#.#", "###"]);
        assert!(paint_marks(&holed, [1, 2, 3], ColourBlend::Colour, 1.0, 1.0).unwrap_err().to_string().contains("hole"));
    }
}
