//! **Ruling HN (owner, 2026-11-23): where the map draws lake and sea**, as a
//! point test, so a town's lake and sea edges are the map's and a census can
//! measure a town's footprint against them.
//!
//! The map decides water per pixel in `render::tile_water_class`: sea where
//! the height is under sea level, else lake by `render::is_lake_pixel`, whose
//! smooth band (the default look's `smooth_shores`) is water where the
//! bilinear of the four corner cells' shore depth is positive (all four
//! corners lake is water outright, none is land). The deep-zoom amplifier
//! never moves a pixel across sea level (`render::tile_water_differs`' doc),
//! so a pixel's sea side is its bilinear base's. [`DrawnWater::class_at`] is
//! that rule from the same inputs (`WorldGen::drawn_water_bodies`, forced
//! lakes included; `render::shore_field_forced_with`, the field
//! `map_shore.gdshader` contours), and
//! `tests::the_point_test_is_the_tiles_own_water_mask` holds it to
//! `lod_bridge::synthesize_tile_water_mask`.
//!
//! **The default look's rule, whatever look is on**: a town layout is cached,
//! hashed and frozen in a save, so it must read nothing a view can change
//! (the same reason `bank_site::town_origin` reads the river's ground width).
//! A look with `smooth_shores` off draws cell-shaped lakes the town does not
//! follow.
//!
//! Coordinates are the tiles' sample frame (cell `c`'s value at `c`), which is
//! the urban adapter's frame and the traced frame less a half
//! (`viewport_host.gd::_lod_tile_rect`).

use crate::{WorldGen, WorldSource, render};

/// [`DrawnWater::class_at`]'s answers, `build_water_bodies`' own classes.
pub(crate) const LAND: u8 = 0;
pub(crate) const SEA: u8 = 1;
pub(crate) const LAKE: u8 = 2;

/// The map's drawn lake and sea for one world state (module doc).
pub(crate) struct DrawnWater {
    /// `0` land, `1` ocean, `2` lake per cell: what the map draws.
    class: std::sync::Arc<[u8]>,
    /// `render::shore_field_forced_with` per cell.
    shore: Vec<f32>,
    gw: usize,
    gh: usize,
    sea_level: f64,
}

impl DrawnWater {
    /// From the drawn classification `class`, and the shore field built from it. `None` when either is not `gw * gh`.
    pub(crate) fn new(class: std::sync::Arc<[u8]>, shore: Vec<f32>, gw: usize, gh: usize, sea_level: f64) -> Option<DrawnWater> {
        (gw >= 2 && gh >= 2 && class.len() == gw * gh && shore.len() == gw * gh).then_some(DrawnWater { class, shore, gw, gh, sea_level })
    }

    /// What the map draws at sample point `(wx, wy)` of height field `field` (the world's height, never the shaded
    /// valley field): [`SEA`], [`LAKE`] or [`LAND`]. Land off the grid, where the map draws nothing.
    pub(crate) fn class_at(&self, field: &[f32], wx: f64, wy: f64) -> u8 {
        let (gw, gh) = (self.gw, self.gh);
        if !(wx >= 0.0 && wy >= 0.0 && wx <= (gw - 1) as f64 && wy <= (gh - 1) as f64) || field.len() < gw * gh {
            return LAND;
        }
        // `is_lake_pixel`'s own corner cells and weights.
        let fx = wx.clamp(0.0, gw as f64 - 1.001);
        let fy = wy.clamp(0.0, gh as f64 - 1.001);
        let (x0, y0) = (fx as usize, fy as usize);
        let (x1, y1) = ((x0 + 1).min(gw - 1), (y0 + 1).min(gh - 1));
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let k = [y0 * gw + x0, y0 * gw + x1, y1 * gw + x0, y1 * gw + x1];
        let wgt = [(1.0 - tx) * (1.0 - ty), tx * (1.0 - ty), (1.0 - tx) * ty, tx * ty];
        let bil = |v: &dyn Fn(usize) -> f64| (0..4).map(|i| wgt[i] * v(k[i])).sum::<f64>();
        if bil(&|i| f64::from(field[i])) < self.sea_level {
            return SEA;
        }
        match k.iter().filter(|&&i| self.class[i] == LAKE).count() {
            4 => LAKE,
            0 => LAND,
            _ if bil(&|i| f64::from(self.shore[i])) > 0.0 => LAKE,
            _ => LAND,
        }
    }
}

impl WorldGen {
    /// The map's drawn lake and sea for the world as it stands ([`DrawnWater`]), cached under `drawn_water_key` (one
    /// priority flood and one shore pass per world state). `None` before a world and for a loaded save, which lays
    /// out no town.
    pub(crate) fn town_water(&self) -> Option<std::sync::Arc<DrawnWater>> {
        let key = self.drawn_water_key();
        if let Some((k, w)) = self.town_water_cache.borrow().as_ref()
            && *k == key
        {
            return Some(w.clone());
        }
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else { return None };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let b = self.drawn_water_bodies()?;
        let shore = render::shore_field_forced_with(&ws.field, &b.classification, &b.fill_level, &ws.rainfall, self.drawn_lakes_rain_gated(), self.forced_lake_mask(), gw, gh, self.sea_level, self.world);
        let w = std::sync::Arc::new(DrawnWater::new(std::sync::Arc::from(b.classification), shore, gw, gh, self.sea_level)?);
        *self.town_water_cache.borrow_mut() = Some((key, w.clone()));
        Some(w)
    }

    /// **Ruling HS: the setback's hazard fields** for the world as it stands (`cartalith_civ::setback::SetbackFields`:
    /// the flood field, the wetland mask and the landform classes, about 65 ms at 2 048 x 1 311), cached under
    /// `drawn_water_key` (the terrain, climate and sea level they are built from), the peak and the two switches.
    /// `None` with both `urban_coastal_setback` and `urban_floodplain_setback` off (every town is laid out as
    /// before), before a world and for a loaded save.
    pub(crate) fn setback_fields(&self) -> Option<std::sync::Arc<cartalith_civ::setback::SetbackFields>> {
        let (coast, river) = (self.params.urban_coastal_setback, self.params.urban_floodplain_setback);
        if !coast && !river {
            return None;
        }
        let key = format!("{};p{};{coast}{river}", self.drawn_water_key(), self.params.peak_m.to_bits());
        if let Some((k, f)) = self.setback_cache.borrow().as_ref()
            && *k == key
        {
            return Some(f.clone());
        }
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else { return None };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let f = std::sync::Arc::new(cartalith_civ::setback::SetbackFields::build(
            &ws.field, &ws.flow_discharge, &ws.temperature, &ws.rainfall, &civ.water_bodies, gw, gh, self.sea_level, self.world, self.map_width_km, self.params.peak_m, coast, river,
        ));
        *self.setback_cache.borrow_mut() = Some((key, f.clone()));
        Some(f)
    }

    /// [`Self::town_water`] for the town layout: `None` with the `civ.bank_sites` switch off, so a world made
    /// before Ruling HN keeps the reference's lake and sea test until the switch is turned on.
    pub(crate) fn town_map_water(&self) -> Option<std::sync::Arc<DrawnWater>> {
        self.params.civ.bank_sites.then(|| self.town_water()).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lod_bridge, sea_live_bridge};

    /// Protects Ruling HN's premise: [`DrawnWater::class_at`] is the water the map's deep-zoom tiles draw
    /// (`lod_bridge::synthesize_tile_water_mask`, the renderer's own `tile_water_class`), pixel for pixel, on a world
    /// with a sea, a lake with a smooth shore band, and land. Every pixel of every level-3 tile is compared; the
    /// counts of each class and of lake-band pixels (a lake cell beside a land cell) are the positive controls.
    /// Mutating the sea test, the corner counts, the shore's sign or the half-cell frame turns it red.
    #[test]
    fn the_point_test_is_the_tiles_own_water_mask() {
        let (gw, gh) = (96usize, 72usize);
        let sea = 0.42f64;
        // A west-to-east ramp (sea in the west third) with a pit in the east that pools a lake, and a ripple so the
        // shores are not straight. Fixture judgement.
        let field: Vec<f32> = (0..gw * gh)
            .map(|k| {
                let (x, y) = ((k % gw) as f64, (k / gw) as f64);
                let pit = 0.30 * (-((x - 70.0).powi(2) + (y - 36.0).powi(2)) / 60.0).exp();
                (0.25 + 0.5 * x / gw as f64 + 0.03 * (x * 0.7).sin() * (y * 0.5).cos() - pit) as f32
            })
            .collect();
        let temp = vec![12.0f32; gw * gh];
        let rain = vec![0.6f32; gw * gh];
        let a = render::TerrainAppearance::default();
        assert!(a.smooth_shores, "the default look draws the smooth shore this test is about");
        let pre = render::GridPrecompute::build(&field, &temp, &rain, Some(&field), gw, gh, sea, false, &a, Some(800.0));
        let ctx = render::RenderCtx::from_precomputed(&field, &temp, &rain, Some(&field), gw, gh, sea, false, 55.0, 5.0, a, &pre).expect("context");
        let tf = render::TileFields::new(&ctx, None);
        // As `WorldGen::town_water` builds it.
        let b = sea_live_bridge::water_bodies_at(&field, &rain, gw, gh, sea, false, None, None, None);
        let shore = render::shore_field_forced_with(&field, &b.classification, &b.fill_level, &rain, true, None, gw, gh, sea, false);
        let dw = DrawnWater::new(std::sync::Arc::from(b.classification.clone()), shore, gw, gh, sea).expect("drawn water");
        let z = 3;
        let n = lod_bridge::tiles_per_axis(z) as i32;
        let (mut seen, mut diff, mut band) = ([0usize; 3], 0usize, 0usize);
        for row in 0..n {
            for col in 0..n {
                let mask = lod_bridge::synthesize_tile_water_mask(&ctx, &tf, z, col, row, 7).expect("tile mask");
                let tb = lod_bridge::tile_bounds(gw, gh, z, col, row).expect("bounds");
                let (w, h) = lod_bridge::tile_size_px(gw, gh, z);
                assert_eq!(mask.len(), w * h);
                for y in 0..h {
                    for x in 0..w {
                        let (wx, wy) = (tb.x + x as f64 * tb.w / (w - 1) as f64, tb.y + y as f64 * tb.h / (h - 1) as f64);
                        let got = dw.class_at(&field, wx, wy);
                        seen[got as usize] += 1;
                        diff += usize::from(got != mask[y * w + x]);
                        let (cx, cy) = ((wx as usize).min(gw - 2), (wy as usize).min(gh - 2));
                        let lakes = [(0, 0), (1, 0), (0, 1), (1, 1)].iter().filter(|&&(i, j)| b.classification[(cy + j) * gw + cx + i] == LAKE).count();
                        band += usize::from(lakes > 0 && lakes < 4);
                    }
                }
            }
        }
        assert!(seen[LAND as usize] > 1000 && seen[SEA as usize] > 1000 && seen[LAKE as usize] > 1000, "land, sea and lake must all be drawn: {seen:?}");
        assert!(band > 1000, "the smooth lake band must be crossed: {band} pixels");
        assert_eq!(diff, 0, "the point test disagrees with the tiles' water mask on {diff} pixels");
    }
}
