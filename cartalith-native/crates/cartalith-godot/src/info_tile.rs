//! Deep-zoom tiles for the **info views** (LOD-D7, Ruling AV, 2026-09-24).
//!
//! # What this is
//!
//! An info view (Temperature, Rainfall, River flow, ...) is drawn by
//! `sample_bridge::debug_raster_with` at the **map's own resolution** and then
//! stretched by the camera, so zooming in only ever magnifies one texel per
//! cell. This module draws the same views at a *tile's* resolution, so a view
//! stays as crisp as the terrain beneath it.
//!
//! # The honesty rule, which is the whole design
//!
//! A tile is derived from the underlying FIELDS at the tile's own pixels, never
//! from the screen raster. What each view can honestly say at a pixel finer than
//! the map's cell depends on what the field is, and [`Refinement`] discloses it
//! per view:
//!
//! * **Refined** -- the view is a function of *height*, and a tile has a real
//!   finer height (`pyramid_tile_padded`, the same amplifier terrain tiles use).
//!   Elevation, Slope and Aspect gain genuine detail.
//! * **Interpolated** -- the field exists only at map resolution (temperature,
//!   crust age, rock resistance). The tile samples it **bilinearly, THEN
//!   colour-ramps** -- never ramps first and upscales the colour. It is smooth,
//!   not detailed, and says so.
//! * **Mixed** -- Rainfall and River flow: the *value* is interpolated, but the
//!   coastline / water mask is the amplified height's, so the shore is as fine
//!   as the terrain's. (A drawn river's own geometry is `river_stroke`'s, not
//!   this module's -- Ruling BV, no river changes.)
//!
//! Every other view is **deferred** ([`DEFERRED_VIEWS`]): categorical (cannot
//! be interpolated), civ-dependent, vector, or a whole-grid computation.
//!
//! # What it must never do
//!
//! * Colour a value with its own ramp: every colour comes from
//!   `sample_bridge`'s `*_px` functions, the ones the raster uses, so a tile
//!   and the raster beneath it can never disagree about what a value looks like.
//! * Normalise per tile: the River-flow `log_max` is a whole-map number passed
//!   IN ([`InfoSource::flow_log_max`]); a per-tile maximum would seam every edge.
//! * Return a plausible tile for a missing input: no flow / age / resistance
//!   means `None` ("not available"), never a flat colour.
//! * Touch terrain tiles: it shares the worker and the snapshot, not a code
//!   path, so terrain tiles and their goldens are unchanged.

use cartalith_engine::bake::pyramid_tile_padded;
use cartalith_spatial::pyramid::ChunkId;
use cartalith_terrain::amplify::AmplifyOpts;

use crate::lod_bridge;
use crate::sample_bridge::{self as sb, Rgb};

/// Bumped whenever an info tile's output changes for the same inputs (a ramp, a
/// sampling rule, a pixel mapping). It is the first thing in [`cache_tag`], so
/// a stale tile from an older build can never be shown as current.
pub const INFO_TILE_VERSION: u32 = 1;

/// How a view's deep-zoom tile gets finer than the map raster. See the module
/// doc; this is the per-view disclosure the shell and the docs report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refinement {
    /// A function of height: the tile has genuinely finer detail.
    Refined,
    /// Bilinear of a map-resolution field, then ramped: smooth, no new detail.
    Interpolated,
    /// Interpolated value under a refined (amplified-height) water mask.
    Mixed,
}

/// The info views that can be tiled. Ids are `sample_bridge`'s view ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InfoView {
    Elevation,
    Temp,
    Rain,
    Flow,
    Slope,
    Aspect,
    Age,
    Resistance,
}

/// Every tileable view, in the order the shell lists them.
pub const TILED_VIEWS: [InfoView; 8] =
    [InfoView::Elevation, InfoView::Temp, InfoView::Rain, InfoView::Flow, InfoView::Slope, InfoView::Aspect, InfoView::Age, InfoView::Resistance];

/// The views that stay map-resolution, with the reason each one is not tiled.
/// Reported by the shell so the owner is told what is NOT refined rather than
/// left to infer it (Ruling AV: "disclose which views are truly refined").
pub const DEFERRED_VIEWS: [(&str, &str); 6] = [
    ("categorical", "lith, soil, bclass, cterrain, plates, bounds, btype, landform, koppen: classes cannot be bilinear-interpolated"),
    ("civ", "strahler, control, contested, settle, rsrc, carry, travel_cost, corridor, windthrow, wildlife: depend on the civ layer or vectors"),
    ("whole-grid", "npp, relief, tpi_multi, fjord, stress: whole-grid computations with no per-tile form yet"),
    ("vector", "wind, ocean, geoid, tides: coarse vector or analytic fields"),
    ("water", "water, flood: derived from the water-body solve, not a field"),
    ("rivers", "river geometry is river_stroke's (Ruling BV, no river changes)"),
];

impl InfoView {
    /// The view id `sample_bridge` and the shell use.
    pub fn id(self) -> &'static str {
        match self {
            InfoView::Elevation => "elevation",
            InfoView::Temp => "temp",
            InfoView::Rain => "rain",
            InfoView::Flow => "flow",
            InfoView::Slope => "slope",
            InfoView::Aspect => "aspect",
            InfoView::Age => "age",
            InfoView::Resistance => "resistance",
        }
    }

    /// The view for a shell id, `None` for a deferred or unknown id.
    pub fn from_id(id: &str) -> Option<InfoView> {
        TILED_VIEWS.iter().copied().find(|v| v.id() == id)
    }

    /// How this view's tile relates to the map raster (the module doc's table).
    pub fn refinement(self) -> Refinement {
        match self {
            InfoView::Elevation | InfoView::Slope | InfoView::Aspect => Refinement::Refined,
            InfoView::Temp | InfoView::Age | InfoView::Resistance => Refinement::Interpolated,
            InfoView::Rain | InfoView::Flow => Refinement::Mixed,
        }
    }

    /// Whether the view reads the amplified height (every view but the three
    /// pure-interpolation ones).
    fn needs_height(self) -> bool {
        !matches!(self, InfoView::Temp | InfoView::Age | InfoView::Resistance)
    }

    /// The halo the height tile needs: one texel for the central difference of
    /// Slope and Aspect, none otherwise.
    fn pad(self) -> usize {
        usize::from(matches!(self, InfoView::Slope | InfoView::Aspect))
    }
}

/// The cache identity of an info tile: the format version, the VIEW id, and the
/// snapshot's producer string (which digests every field a tile reads -- height,
/// temperature, rainfall, flow, crust age, resistance, `peak_m`, the map width,
/// the seed and the sea level). A view switch, a sculpt, a sea-level move or an
/// appearance change each change this string, so the shell drops, never reuses,
/// a tile whose tag differs.
pub fn cache_tag(view: InfoView, producer: &str) -> String {
    format!("info{INFO_TILE_VERSION};{};{producer}", view.id())
}

/// Borrowed world inputs for one tile. All slices are `gw * gh`.
pub struct InfoSource<'a> {
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    pub seed: i32,
    pub peak_m: f64,
    pub map_width_km: f64,
    pub field: &'a [f32],
    pub temperature: &'a [f32],
    pub rainfall: &'a [f32],
    pub flow: Option<&'a [f32]>,
    pub age: Option<&'a [f32]>,
    pub resistance: Option<&'a [f32]>,
    /// `sample_bridge::flow_log_max` of the whole flow field, computed once per
    /// snapshot (see the module doc: no per-tile normalisation).
    pub flow_log_max: f64,
}

impl InfoSource<'_> {
    /// Whether the inputs `view` reads exist. `false` means "not available";
    /// the shell keeps the map-resolution raster for that view.
    pub fn available(&self, view: InfoView) -> bool {
        let n = self.gw * self.gh;
        let has = |s: Option<&[f32]>| s.is_some_and(|s| s.len() >= n);
        self.gw >= 2
            && self.gh >= 2
            && self.field.len() >= n
            && match view {
                InfoView::Elevation | InfoView::Slope | InfoView::Aspect => true,
                InfoView::Temp => self.temperature.len() >= n,
                InfoView::Rain => self.rainfall.len() >= n,
                InfoView::Flow => has(self.flow),
                InfoView::Age => has(self.age),
                InfoView::Resistance => has(self.resistance),
            }
    }
}

/// Synthesise tile `(z, col, row)` of `view`: `(rgba, w, h)`, `rgba.len() ==
/// w * h * 4`. `None` when [`InfoSource::available`] says the inputs are
/// missing or the tile index is outside the pyramid.
///
/// Pixel `(px, py)` sits at coarse sample coordinate `(tb.x + px * cx, tb.y +
/// py * cy)` with `cx = tb.w / (w - 1)` -- the terrain renderer's own mapping --
/// so adjacent tiles share an edge sample and agree on it bit for bit.
/// Deterministic in its inputs, so a worker thread and the main thread produce
/// identical bytes.
pub fn synthesize_info_tile(src: &InfoSource, view: InfoView, z: i32, col: i32, row: i32) -> Option<(Vec<u8>, usize, usize)> {
    if !src.available(view) {
        return None;
    }
    let (gw, gh) = (src.gw, src.gh);
    let tb = lod_bridge::tile_bounds(gw, gh, z, col, row)?;
    let (w, h) = lod_bridge::tile_size_px(gw, gh, z);
    if w < 2 || h < 2 {
        return None;
    }
    let cx = tb.w / (w - 1) as f64;
    let cy = tb.h / (h - 1) as f64;

    // The amplified height, only for the views that read it. Water is decided
    // from the world's own `field` (never RV-3's shaded valley field), as the
    // terrain tile's water mask is.
    let pad = view.pad();
    let stride = w + 2 * pad;
    let height: Option<Vec<f32>> = if view.needs_height() {
        let opts = AmplifyOpts { seed: src.seed, sea: src.sea, z_base: lod_bridge::z_base(), ..AmplifyOpts::default() };
        let id = ChunkId::new(z as u32, col as u32, row as u32);
        let (tw, th, t) = pyramid_tile_padded(src.field, gw, gh, id, lod_bridge::TILE_PX, pad, &opts);
        if (tw, th) != (w, h) || t.len() != stride * (h + 2 * pad) {
            return None;
        }
        Some(t)
    } else {
        None
    };
    let hgt = |px: usize, py: usize| -> f64 {
        // Core pixel (px, py) of the padded tile.
        height.as_ref().map_or(0.0, |t| t[(py + pad) * stride + px + pad] as f64)
    };
    // Height units per CELL: a per-pixel difference over its pixel pitch.
    let grad = |px: usize, py: usize| -> (f64, f64) {
        let l = height.as_ref().map_or(0.0, |t| t[(py + pad) * stride + px + pad - 1] as f64);
        let r = height.as_ref().map_or(0.0, |t| t[(py + pad) * stride + px + pad + 1] as f64);
        let u = height.as_ref().map_or(0.0, |t| t[(py + pad - 1) * stride + px + pad] as f64);
        let d = height.as_ref().map_or(0.0, |t| t[(py + pad + 1) * stride + px + pad] as f64);
        ((r - l) * 0.5 / cx, (d - u) * 0.5 / cy)
    };
    let k = sb::slope_k(src.peak_m, src.sea, if src.map_width_km > 0.0 { src.map_width_km * 1000.0 / gw as f64 } else { 0.0 });
    let bil = |a: &[f32], px: usize, py: usize| sb::bil_c(a, tb.x + px as f64 * cx, tb.y + py as f64 * cy, gw, gh, false);

    let mut out: Vec<u8> = Vec::with_capacity(w * h * 4);
    for py in 0..h {
        for px in 0..w {
            let c: Rgb = match view {
                InfoView::Elevation => sb::hypso(hgt(px, py), src.sea),
                InfoView::Temp => sb::temp_color(bil(src.temperature, px, py)),
                InfoView::Rain => sb::rain_px(hgt(px, py) < src.sea, bil(src.rainfall, px, py)),
                InfoView::Flow => sb::flow_px(hgt(px, py), bil(src.flow?, px, py), src.flow_log_max, src.sea),
                InfoView::Slope => {
                    let (dx, dy) = grad(px, py);
                    sb::slope_px(dx.hypot(dy), k)
                }
                InfoView::Aspect => {
                    let (dx, dy) = grad(px, py);
                    sb::aspect_px(dx, dy, k)
                }
                InfoView::Age => sb::age_px(bil(src.age?, px, py)),
                InfoView::Resistance => sb::resistance_px(bil(src.resistance?, px, py)),
            };
            sb::push(&mut out, c);
        }
    }
    Some((out, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small world with real structure: a ridge-and-valley height (so slope
    /// and aspect vary), a temperature gradient, rain, flow, age, resistance.
    struct World {
        gw: usize,
        gh: usize,
        field: Vec<f32>,
        temp: Vec<f32>,
        rain: Vec<f32>,
        flow: Vec<f32>,
        age: Vec<f32>,
        res: Vec<f32>,
    }

    fn world(gw: usize, gh: usize) -> World {
        let n = gw * gh;
        let at = |f: &dyn Fn(usize, usize) -> f32| -> Vec<f32> { (0..n).map(|i| f(i % gw, i / gw)).collect() };
        World {
            gw,
            gh,
            field: at(&|x, y| 0.30 + 0.45 * (((x as f32) * 0.35).sin() * 0.5 + 0.5) * (y as f32 / gh as f32 * 0.8 + 0.2)),
            temp: at(&|x, y| 30.0 - y as f32 * 0.9 + (x % 5) as f32 * 0.4),
            rain: at(&|x, y| (((x * 3 + y) % 17) as f32 / 16.0).clamp(0.0, 1.0)),
            flow: at(&|x, y| ((x + 2 * y) % 13) as f32 * 40.0),
            age: at(&|x, y| (((x + y) % 11) as f32 / 10.0).clamp(0.0, 1.0)),
            res: at(&|x, y| (((x * 2 + y) % 7) as f32 / 6.0).clamp(0.0, 1.0)),
        }
    }

    fn src(w: &World) -> InfoSource<'_> {
        InfoSource {
            gw: w.gw,
            gh: w.gh,
            sea: 0.42,
            seed: 24601,
            peak_m: 4000.0,
            map_width_km: 800.0,
            field: &w.field,
            temperature: &w.temp,
            rainfall: &w.rain,
            flow: Some(&w.flow),
            age: Some(&w.age),
            resistance: Some(&w.res),
            flow_log_max: sb::flow_log_max(&w.flow),
        }
    }

    fn px(t: &(Vec<u8>, usize, usize), x: usize, y: usize) -> [u8; 4] {
        let i = (y * t.1 + x) * 4;
        [t.0[i], t.0[i + 1], t.0[i + 2], t.0[i + 3]]
    }

    fn rgb(c: Rgb) -> [u8; 4] {
        let mut v = Vec::new();
        sb::push(&mut v, c);
        [v[0], v[1], v[2], v[3]]
    }

    /// Protects: an interpolated view's tile is the bilinear of the FIELD run
    /// through the SAME ramp the raster uses. At level 0 of a 256 square grid
    /// the pixel pitch is exactly one cell, so every tile pixel must equal the
    /// ramp of the field's own value at that cell -- not an upscale of colours.
    /// Literal expectation: temp_color of the raw sample.
    #[test]
    fn interpolated_views_at_integer_sampling_equal_the_ramp_of_the_field() {
        let w = world(256, 256);
        let s = src(&w);
        for (view, want) in [
            (InfoView::Temp, (|w: &World, i: usize| sb::temp_color(w.temp[i] as f64)) as fn(&World, usize) -> Rgb),
            (InfoView::Age, |w, i| sb::age_px(w.age[i] as f64)),
            (InfoView::Resistance, |w, i| sb::resistance_px(w.res[i] as f64)),
        ] {
            let t = synthesize_info_tile(&s, view, 0, 0, 0).expect("tile");
            assert_eq!((t.1, t.2), (256, 256), "level 0 of a 256 grid is 256 px");
            for (x, y) in [(0, 0), (17, 3), (100, 200), (255, 255), (128, 128)] {
                assert_eq!(px(&t, x, y), rgb(want(&w, y * 256 + x)), "{} at ({x},{y})", view.id());
            }
        }
    }

    /// Protects: an interpolated value between two cells is the BILINEAR value
    /// ramped, which for a linear ramp differs from the average of the two
    /// ramped colours only through clamping -- here, a half-cell tile pixel of
    /// the temperature view equals temp_color of the mean temperature.
    #[test]
    fn a_pixel_between_two_cells_ramps_the_interpolated_value() {
        let w = world(8, 8);
        let s = src(&w);
        // z=1 of an 8-wide grid: tile pitch is (gw-1)/2^1 over 256-ish px, far
        // finer than a cell, so an arbitrary pixel is a genuine in-between.
        let t = synthesize_info_tile(&s, InfoView::Temp, 1, 0, 0).expect("tile");
        let tb = lod_bridge::tile_bounds(8, 8, 1, 0, 0).unwrap();
        let (x, y) = (37usize, 91usize);
        let cx = tb.w / (t.1 - 1) as f64;
        let cy = tb.h / (t.2 - 1) as f64;
        let (fx, fy) = (tb.x + x as f64 * cx, tb.y + y as f64 * cy);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let v = |xx: usize, yy: usize| w.temp[yy * 8 + xx] as f64;
        let want = (v(x0, y0) * (1.0 - tx) + v(x0 + 1, y0) * tx) * (1.0 - ty) + (v(x0, y0 + 1) * (1.0 - tx) + v(x0 + 1, y0 + 1) * tx) * ty;
        assert!(tx > 0.01 && tx < 0.99, "the pixel must sit between cells");
        assert_eq!(px(&t, x, y), rgb(sb::temp_color(want)));
    }

    /// Protects: a refined view is finer than the map raster, and an
    /// interpolated one is not. At a deep level, the Elevation tile must differ
    /// from the colour of the plain bilinear of the coarse field in a large share
    /// of its pixels (the amplifier's added relief), while the Temperature tile
    /// equals the ramp of ITS bilinear exactly (the positive/negative control:
    /// the comparison can tell the two apart). The Slope tile must also carry
    /// many distinct colours (>40) across a span the raster covers with one cell.
    #[test]
    fn a_refined_view_gains_detail_the_raster_does_not_have() {
        let w = world(64, 64);
        let s = src(&w);
        let (z, col, row) = (6, 20, 20);
        let tb = lod_bridge::tile_bounds(64, 64, z, col, row).unwrap();
        let elev = synthesize_info_tile(&s, InfoView::Elevation, z, col, row).expect("elev");
        let temp = synthesize_info_tile(&s, InfoView::Temp, z, col, row).expect("temp");
        let (mut elev_diff, mut temp_diff) = (0usize, 0usize);
        for y in 0..elev.2 {
            for x in 0..elev.1 {
                let cx = tb.w / (elev.1 - 1) as f64;
                let cy = tb.h / (elev.2 - 1) as f64;
                let (fx, fy) = (tb.x + x as f64 * cx, tb.y + y as f64 * cy);
                let smooth_h = sb::bil_c(&w.field, fx, fy, 64, 64, false);
                let smooth_t = sb::bil_c(&w.temp, fx, fy, 64, 64, false);
                elev_diff += usize::from(px(&elev, x, y) != rgb(sb::hypso(smooth_h, 0.42)));
                temp_diff += usize::from(px(&temp, x, y) != rgb(sb::temp_color(smooth_t)));
            }
        }
        let n = elev.1 * elev.2;
        assert_eq!(temp_diff, 0, "an interpolated tile is exactly the ramp of the bilinear field");
        assert!(elev_diff * 4 > n, "refined elevation differs from the smooth field in only {elev_diff} of {n} px");
        let slope = synthesize_info_tile(&s, InfoView::Slope, z, col, row).expect("slope");
        let mut set = std::collections::HashSet::new();
        for i in 0..slope.1 * slope.2 {
            set.insert([slope.0[i * 4], slope.0[i * 4 + 1], slope.0[i * 4 + 2]]);
        }
        assert!(set.len() > 40, "slope distinct colours {}", set.len());
    }

    /// Protects: no per-tile normalisation. The flow view's whole-map scale is
    /// shared, so two tiles meeting at an edge agree on the shared sample's
    /// colour bit for bit; a per-tile `log_max` would break this. Compares the
    /// last column of tile (1,0,0) with the first of (1,1,0).
    #[test]
    fn adjacent_flow_tiles_agree_along_their_shared_edge() {
        let mut w = world(64, 64);
        // Flow that GROWS exponentially to the right, so the two tiles' own
        // maxima differ and the colour has not saturated at the shared edge
        // (the default world's flow repeats every 13 cells, giving every tile
        // the same maximum; a quadratic one saturated `flow_px`'s ramp at the
        // edge. Each let a per-tile `log_max` mutant survive).
        // The field is all land (0.7 > sea 0.42): the default world's edge at
        // x = 31.5 is entirely sea, where flow_px ignores flow altogether.
        w.field = vec![0.7; 64 * 64];
        w.flow = (0..64 * 64).map(|i| (((i % 64) as f32 + (i / 64) as f32 / 4.0) / 8.0).exp() - 1.0).collect();
        let s = src(&w);
        let mut distinct = std::collections::HashSet::new();
        let a = synthesize_info_tile(&s, InfoView::Flow, 1, 0, 0).expect("a");
        let b = synthesize_info_tile(&s, InfoView::Flow, 1, 1, 0).expect("b");
        assert_eq!(a.2, b.2);
        for y in 0..a.2 {
            assert_eq!(px(&a, a.1 - 1, y), px(&b, 0, y), "edge row {y}");
            distinct.insert(px(&a, a.1 - 1, y));
        }
        assert!(distinct.len() > 1, "a single-colour edge would make the comparison vacuous");
    }

    /// Protects: a flat field makes Elevation, Slope and Aspect flat, never
    /// noisy: the amplifier adds no relief where there is none to refine, and a
    /// zero gradient reads as the flat aspect colour, not "faces north".
    #[test]
    fn a_flat_world_yields_flat_height_views() {
        let mut w = world(32, 32);
        w.field = vec![0.7; 32 * 32];
        let s = src(&w);
        let e = synthesize_info_tile(&s, InfoView::Elevation, 2, 1, 1).expect("e");
        let first = px(&e, 0, 0);
        assert!((0..e.1 * e.2).all(|i| e.0[i * 4..i * 4 + 3] == first[..3]), "flat elevation not flat");
        let a = synthesize_info_tile(&s, InfoView::Aspect, 2, 1, 1).expect("a");
        assert_eq!(px(&a, 5, 5), rgb((34.0, 36.0, 42.0)), "flat aspect is the no-bearing colour");
    }

    /// Protects: a view whose input is missing is "not available", never a flat
    /// plausible colour -- a loaded save has no flow or crust age.
    #[test]
    fn a_view_without_its_input_is_unavailable() {
        let w = world(16, 16);
        let mut s = src(&w);
        s.flow = None;
        s.age = None;
        s.resistance = None;
        for v in [InfoView::Flow, InfoView::Age, InfoView::Resistance] {
            assert!(synthesize_info_tile(&s, v, 0, 0, 0).is_none(), "{}", v.id());
        }
        assert!(synthesize_info_tile(&s, InfoView::Temp, 0, 0, 0).is_some());
        assert!(synthesize_info_tile(&s, InfoView::Temp, 0, 5, 0).is_none(), "index outside level 0");
    }

    /// Protects: the cache tag moves with the VIEW (and the producer), so a view
    /// switch can never match another view's tile. Literal expectations.
    #[test]
    fn the_cache_tag_names_the_view_and_the_producer() {
        assert_eq!(cache_tag(InfoView::Temp, "P"), "info1;temp;P");
        assert_ne!(cache_tag(InfoView::Temp, "P"), cache_tag(InfoView::Rain, "P"));
        assert_ne!(cache_tag(InfoView::Temp, "P"), cache_tag(InfoView::Temp, "Q"));
    }

    /// Protects: the disclosure table (which views are refined, interpolated or
    /// mixed) -- literal, since it is what the docs and the shell report -- and
    /// that every tiled view round-trips its id.
    #[test]
    fn the_disclosure_table_is_pinned() {
        let got: Vec<(&str, Refinement)> = TILED_VIEWS.iter().map(|v| (v.id(), v.refinement())).collect();
        assert_eq!(
            got,
            vec![
                ("elevation", Refinement::Refined),
                ("temp", Refinement::Interpolated),
                ("rain", Refinement::Mixed),
                ("flow", Refinement::Mixed),
                ("slope", Refinement::Refined),
                ("aspect", Refinement::Refined),
                ("age", Refinement::Interpolated),
                ("resistance", Refinement::Interpolated),
            ]
        );
        for v in TILED_VIEWS {
            assert_eq!(InfoView::from_id(v.id()), Some(v));
        }
        assert_eq!(InfoView::from_id("lith"), None);
    }
}
