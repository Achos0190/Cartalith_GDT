//! Heightmap import — `loadImage()` + `inferTectonics()` (reference HTML
//! lines 4914-4928 and 6755-6797).
//!
//! The reference offers three ways into a world, and this is the third:
//! generate one from a seed, load a `.zip` save, or **bring in a heightmap
//! and infer a tectonic substrate under it**. Only the first two existed in
//! this port before.
//!
//! # The shape of the problem
//!
//! An imported DEM arrives as a bare elevation field. Everything downstream
//! of tectonics — lithology, soil, resources, settlement suitability, the
//! Tect/Lith debug views — reads plate and stress fields that simply are not
//! there, and reads them as zeros. `inferTectonics()` exists to fill that in
//! with a *plausible proxy* reconstructed from the heightmap's own
//! morphology. The pure per-cell maths lives in
//! [`cartalith_terrain::infer`]; this module is only the sequencing, per
//! `ARCHITECTURE.md`'s split.
//!
//! # Why this is not `generate_terrain` with a different first stage
//!
//! [`crate::generate_terrain`] runs *forward*: seed → plates → stress →
//! height. Here the height is given and everything else is derived
//! backwards from it, so the height stages (`compute_height`, `normalize`,
//! volcano and crater stamping, world-structure sea-level re-anchoring)
//! must all be **skipped** — the reference is explicit that `inferTectonics`
//! "leaves `field` untouched — only the tectonic/derived layers". Running
//! them would overwrite the very data the user imported.
//!
//! # Parity notes
//!
//! - The reference's tail is `refreshClimate(); enforceRiverChannels();
//!   computeFlow(true)`. `enforceRiverChannels` is a no-op in this port (see
//!   `crate`'s own module doc), and `refreshClimate` is
//!   `computeTemperature` → `simulateWeather` → moisture correctors →
//!   optional ocean currents.
//! - **The moisture correctors run against a ZERO flow field.** This is not
//!   an oversight being reproduced blindly: `allocate()` zeroes `flowField`,
//!   `loadImage` never fills it, and `inferTectonics` only calls
//!   `computeFlow(true)` *after* `refreshClimate()`. The reference's own
//!   v0.70 comment says so in as many words ("an imported heightmap arrives
//!   with flowField all-zero"). Passing a real flow field here would change
//!   rainfall for every imported world relative to the reference, so it is
//!   passed zero — and then computed properly afterwards, because this
//!   port splits JS's single `flowField` into `flow_area` and
//!   `flow_discharge` and leaving the former permanently zero would break
//!   every downstream consumer that reads it. See [`infer_tectonics`].

use cartalith_climate::{
    apply_climate_moisture_correctors, apply_ocean_currents, compute_temperature, simulate_weather, ClimateParams,
    WeatherParams,
};
use cartalith_hydrology::compute_flow;
use cartalith_terrain::infer::{
    build_relief_field, classify_plate_crust, heightmap_grid_h, heightmap_to_field, infer_plate_velocities,
    pick_plate_seeds, reconstruct_boundary_stress, stamp_volcanic_arcs,
};
use cartalith_terrain::{
    assign_plates, build_age_field, compute_flexure, compute_heterogeneity, compute_resistance, gauss_blur,
    normalize_field,
};

use crate::{WorldParams, WorldState};

/// Why a heightmap could not be imported. Every variant is a real,
/// reportable condition the boundary layer turns into a status line rather
/// than a panic (`cartalith-rust-conventions`: nothing may unwind across
/// the gdext boundary).
#[derive(Debug)]
#[non_exhaustive]
pub enum ImportError {
    /// The bytes are not a PNG, or are a corrupt one.
    Decode(String),
    /// The image decoded to zero pixels in one or both axes.
    Empty,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Decode(m) => write!(f, "could not decode heightmap: {m}"),
            ImportError::Empty => write!(f, "heightmap decoded to an empty image"),
        }
    }
}

impl std::error::Error for ImportError {}

/// A world brought in from a heightmap: the generated state, plus the
/// params it was actually built with.
///
/// `params` is returned rather than mutated in place because the import
/// **derives `gh` from the image's own aspect ratio**, not from the
/// caller's — `loadImage`'s `GH=Math.max(80,Math.round(GW/ar))`. A caller
/// that kept its original params would then index every field with the
/// wrong stride, which is exactly the bug class this port's own notes call
/// "the v0.071 warp-cache bug".
pub struct ImportedWorld {
    pub params: WorldParams,
    pub state: WorldState,
    /// The source image's pixel dimensions, for the status line.
    pub source_size: (u32, u32),
}

/// Decode PNG bytes into a normalised elevation field on the working grid.
///
/// Returns the field and the grid height derived from the image aspect.
/// Splitting this out from [`import_heightmap`] keeps the decode testable
/// without running the whole pipeline.
///
/// # Format support
///
/// PNG only. The reference's file input is `accept="image/*"` and it
/// decodes through the browser's own image pipeline, so in practice it
/// accepts PNG and JPEG and **not** TIFF (no browser decodes TIFF natively).
/// This port does PNG, which is the format every heightmap tool exports and
/// the only one the reference's own save/export path ever writes. A 16-bit
/// PNG decodes fine, at 8-bit precision — matching the reference, which
/// reads through a `<canvas>` and therefore cannot see more than 8 bits
/// either.
///
/// # Errors
///
/// [`ImportError::Decode`] if the bytes are not a readable PNG;
/// [`ImportError::Empty`] if it decodes to a zero-area image.
pub fn decode_heightmap(bytes: &[u8], gw: usize) -> Result<(Vec<f32>, usize), ImportError> {
    let img = cartalith_assets::raster::decode_png(bytes).map_err(|e| ImportError::Decode(e.to_string()))?;
    if img.w == 0 || img.h == 0 {
        return Err(ImportError::Empty);
    }
    let gh = heightmap_grid_h(gw, img.w, img.h);
    let raw = heightmap_to_field(&img.rgba, img.w as usize, img.h as usize, gw, gh);
    // `loadImage`'s own next call. Without it an 8-bit heightmap that never
    // reaches pure white sits below 1.0 and every elevation-relative
    // threshold downstream (sea level most of all) reads low.
    Ok((normalize_field(&raw), gh))
}

/// The full import: decode, infer a tectonic substrate, run the downstream
/// pipeline. This is `loadImage` followed by the calibrate gate's
/// `_suCalCommit` → `inferTectonics` (reference HTML line 13830), which is
/// the only route the reference offers into an imported world.
///
/// `base` supplies everything the image cannot: sea level, map width, peak
/// altitude, climate and planet parameters. Its `gh` is **ignored and
/// replaced** — see [`ImportedWorld`].
///
/// # Errors
///
/// Propagates [`decode_heightmap`]'s errors unchanged.
pub fn import_heightmap(bytes: &[u8], base: &WorldParams) -> Result<ImportedWorld, ImportError> {
    let img = cartalith_assets::raster::decode_png(bytes).map_err(|e| ImportError::Decode(e.to_string()))?;
    if img.w == 0 || img.h == 0 {
        return Err(ImportError::Empty);
    }
    let (field, gh) = decode_heightmap(bytes, base.gw)?;
    let mut params = base.clone();
    params.gh = gh;
    let state = infer_tectonics(field, &params);
    Ok(ImportedWorld { params, state, source_size: (img.w, img.h) })
}

/// `inferTectonics()` (reference HTML line 6755): reconstruct a plate set
/// and every tectonic proxy field from an existing elevation field, then
/// run the downstream stages the reference runs.
///
/// `field` is taken by value and returned untouched inside the
/// [`WorldState`] — the reference is explicit that this pass "leaves
/// `field` untouched", and taking ownership makes that structurally true
/// rather than a promise.
///
/// Deterministic: no RNG anywhere in the inversion itself. (The
/// heterogeneity field below *is* seeded, from `p.tect.seed`, because it is
/// the forward `computeHeterogeneity` reused verbatim — the reference does
/// the same.)
#[must_use]
pub fn infer_tectonics(field: Vec<f32>, p: &WorldParams) -> WorldState {
    let gw = p.gw;
    let gh = p.gh;
    let world = p.world;
    let sea_level = p.sea_level;
    let n = gw * gh;

    // ---- the inversion (reference HTML lines 6757-6771) ----
    let relief = build_relief_field(&field, gw, gh, world, None);
    let mut plates = pick_plate_seeds(&relief, gw, gh, None);
    // `assignPlates()` unchanged -- the reference reuses the same proven
    // JFA Voronoi here rather than a second partitioner. `warpX`/`warpY` are
    // null after an import (`loadImage` sets them so explicitly), which is
    // what the `None`s are.
    let plate_id = assign_plates(gw, gh, world, &plates, None, None);
    let base = classify_plate_crust(&field, &plate_id, plates.len(), gw, gh, sea_level);
    for (pl, &b) in plates.iter_mut().zip(base.iter()) {
        pl.base = b as f64;
    }
    let stress = reconstruct_boundary_stress(&field, &plate_id, &base, &relief, gw, gh, world, None, None, None);

    // `baseField` = per-cell plate base, then the same 0.35x blur the
    // forward substrate applies (reference line 6767).
    let crust_field: Vec<f32> = plate_id.iter().map(|&pi| plates[pi].base as f32).collect();
    let _base_field = gauss_blur(&crust_field, (p.tect.blur_r * 0.35).max(2.0), gw, gh, world);

    // ---- forward stages, reused verbatim (reference HTML lines 6769-6776) ----
    let age_field = build_age_field(gw, gh, &stress.boundary_mask);
    // `warpX`/`warpY` are null after an import; the reference's own comment
    // on this line says "needs ageField + warpX/Y (null after import =>
    // handled)".
    let heterogeneity_field =
        compute_heterogeneity(gw, gh, p.tect.seed, p.map_width_km, world, &age_field, None, None);
    let resistance_field = compute_resistance(gw, gh, &plate_id, &plates, &age_field);
    let flexure_field = compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, p.tect.blur_r, world);
    let volcanic_field = stamp_volcanic_arcs(&stress.boundary_type, gw, gh, None);
    infer_plate_velocities(&mut plates, &plate_id, &stress.boundary_mask, &stress.stress_field, gw);

    // ---- refreshClimate() (reference HTML line 5154) ----
    let climate_params = ClimateParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        g: p.planet.g,
        sea_level,
        peak_m: p.peak_m,
        albedo_k: p.climate.albedo_k,
    };
    let mut temperature = compute_temperature(gw, gh, &field, None, &climate_params);

    let weather_params = WeatherParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        sea_level,
        peak_m: p.peak_m,
        wind_manual: p.climate.wind_manual,
        wind_dir_deg: p.climate.wind_dir_deg,
        press_k: p.climate.press_k,
        ocean_hum: p.climate.ocean_hum,
        evap: p.climate.evap,
        ocean: p.climate.ocean,
        rain_k: p.climate.rain_k,
        rain_dep: p.climate.rain_dep,
        bulk_evap: p.climate.bulk_evap,
        terrain_wind_deflection: p.climate.terrain_wind_deflection,
        currents: p.climate.currents,
        current_k: p.climate.current_k,
    };
    let mut rainfall = simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params);

    // The zero flow field the correctors genuinely see on this path. Named
    // rather than inlined so it cannot be mistaken for an oversight -- see
    // this module's doc comment for why it is zero and must stay zero.
    let flow_field_at_corrector_time = vec![0f32; n];
    apply_climate_moisture_correctors(
        gw,
        gh,
        &field,
        &flow_field_at_corrector_time,
        &mut rainfall,
        sea_level,
        world,
        p.climate.lat_n,
        p.climate.lat_s,
        p.climate.zonal_k,
    );
    if p.climate.currents {
        apply_ocean_currents(
            gw,
            gh,
            &field,
            &mut temperature,
            &mut rainfall,
            sea_level,
            world,
            p.climate.lat_n,
            p.climate.lat_s,
            p.climate.equator_temp,
            p.climate.pole_temp,
            p.planet.axial_tilt_deg,
            p.planet.rotation_hours,
            p.climate.wind_manual,
            p.climate.wind_dir_deg,
            p.climate.press_k,
            p.climate.current_k,
        );
    }

    // ---- computeFlow(true) (reference HTML line 6797) ----
    let flow_discharge = compute_flow(gw, gh, &field, Some(&rainfall), true, world);
    // JS has ONE `flowField`, so the reference simply never has a separate
    // rain-independent accumulation for an imported world. This port splits
    // the two, and every consumer of `flow_area` (the drainage-area debug
    // view, `build_water_access`) would read zeros if it were left empty --
    // a port-level bug, not fidelity. Computed here, AFTER the correctors,
    // so the parity-relevant read above still sees the zero field.
    let flow_area = compute_flow(gw, gh, &field, None, false, world);

    WorldState {
        sea_level,
        field,
        plate_id,
        boundary_mask: stress.boundary_mask,
        stress_field: stress.stress_field,
        flexure_field,
        age_field,
        heterogeneity_field,
        resistance_field,
        crust_field,
        boundary_type: stress.boundary_type,
        shear_field: stress.shear_field,
        volcanic_field,
        // No craters on an imported world: `stampCraters` is a *height*
        // stage, and this pass must not touch the imported height.
        impact_field: vec![0f32; n],
        temperature,
        rainfall,
        flow_area,
        flow_discharge,
        // River carving is likewise a height stage. The reference's
        // inferTectonics does not carve either -- it stops at computeFlow.
        channels: None,
        stream_order: None,
        river_mask: None,
        river_floor: None,
        // Nothing here takes a GPU path.
        gpu_stages_used: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small PNG with a real elevation gradient, encoded through the same
    /// codec the import decodes with.
    fn ramp_png(w: u32, h: u32) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                // A diagonal ramp with a ridge, so relief is not uniform and
                // the inversion has something to find.
                let u = x as f64 / (w - 1) as f64;
                let v = y as f64 / (h - 1) as f64;
                let e = (0.2 + 0.6 * u + 0.2 * (1.0 - (v - 0.5).abs() * 2.0)).clamp(0.0, 1.0);
                let g = (e * 255.0) as u8;
                rgba.extend_from_slice(&[g, g, g, 255]);
            }
        }
        let img = cartalith_assets::raster::DecodedImage::new(w, h, rgba).expect("w*h*4 by construction");
        cartalith_assets::raster::encode_png(&img).expect("encoding a valid RGBA8 image cannot fail")
    }

    #[test]
    fn import_produces_a_complete_world_state() {
        let png = ramp_png(64, 32);
        let base = WorldParams::defaults(48, 999, 12345);
        let out = import_heightmap(&png, &base).expect("valid PNG must import");
        let n = out.params.gw * out.params.gh;

        // The grid height came from the IMAGE aspect (64x32 => 2:1 => 24),
        // floored at 80 -- not from the caller's 999.
        assert_eq!(out.params.gw, 48);
        assert_eq!(out.params.gh, 80, "aspect gives 24, the floor lifts it to 80");
        assert_eq!(out.source_size, (64, 32));

        for (name, len) in [
            ("field", out.state.field.len()),
            ("plate_id", out.state.plate_id.len()),
            ("boundary_mask", out.state.boundary_mask.len()),
            ("stress_field", out.state.stress_field.len()),
            ("flexure_field", out.state.flexure_field.len()),
            ("age_field", out.state.age_field.len()),
            ("heterogeneity_field", out.state.heterogeneity_field.len()),
            ("resistance_field", out.state.resistance_field.len()),
            ("crust_field", out.state.crust_field.len()),
            ("boundary_type", out.state.boundary_type.len()),
            ("shear_field", out.state.shear_field.len()),
            ("volcanic_field", out.state.volcanic_field.len()),
            ("temperature", out.state.temperature.len()),
            ("rainfall", out.state.rainfall.len()),
            ("flow_area", out.state.flow_area.len()),
            ("flow_discharge", out.state.flow_discharge.len()),
        ] {
            assert_eq!(len, n, "{name} has the wrong length");
        }
    }

    /// The whole point of the pass: the fields that were zero before it must
    /// not be zero after it. This is the assertion that would have caught
    /// the bug the reference wrote `inferTectonics` to fix.
    #[test]
    fn import_leaves_no_tectonic_field_dead() {
        let png = ramp_png(64, 32);
        let base = WorldParams::defaults(48, 999, 12345);
        let out = import_heightmap(&png, &base).expect("valid PNG must import");
        let s = &out.state;
        assert!(s.boundary_mask.iter().any(|&v| v != 0), "no plate boundaries");
        assert!(s.stress_field.iter().any(|&v| v != 0.0), "stress is dead");
        assert!(s.shear_field.iter().any(|&v| v != 0.0), "shear is dead");
        assert!(s.age_field.iter().any(|&v| v != 0.0), "age is dead");
        assert!(s.flexure_field.iter().any(|&v| v != 0.0), "flexure is dead");
        assert!(s.resistance_field.iter().any(|&v| v != 0.0), "resistance is dead");
        assert!(s.heterogeneity_field.iter().any(|&v| v != 0.0), "heterogeneity is dead");
        assert!(s.crust_field.iter().any(|&v| v != 0.0), "crust is dead");
        assert!(s.plate_id.iter().any(|&v| v != 0), "every cell landed on one plate");
        assert!(s.rainfall.iter().any(|&v| v > 0.0), "no rain anywhere");
        assert!(s.flow_discharge.iter().any(|&v| v > 0.0), "discharge is dead");
        assert!(s.flow_area.iter().any(|&v| v > 0.0), "flow_area is dead -- see this module's doc");
        assert!(s.temperature.iter().any(|&v| v != 0.0), "temperature is dead");
        // Everything must be finite: a NaN here propagates silently through
        // every downstream layer.
        assert!(s.field.iter().all(|v| v.is_finite()));
        assert!(s.stress_field.iter().all(|v| v.is_finite()));
        assert!(s.rainfall.iter().all(|v| v.is_finite()));
    }

    /// `inferTectonics` "leaves `field` untouched — only the tectonic/derived
    /// layers". Structurally guaranteed by taking the field by value, and
    /// pinned here anyway because it is the invariant a user notices first.
    #[test]
    fn infer_tectonics_does_not_modify_the_imported_field() {
        let png = ramp_png(64, 32);
        let mut p = WorldParams::defaults(48, 0, 7);
        let (field, gh) = decode_heightmap(&png, p.gw).expect("valid PNG");
        p.gh = gh;
        let before = field.clone();
        let state = infer_tectonics(field, &p);
        assert_eq!(state.field, before, "the imported elevation was overwritten");
    }

    #[test]
    fn decode_normalises_to_the_full_zero_one_range() {
        let png = ramp_png(64, 32);
        let (field, _) = decode_heightmap(&png, 48).expect("valid PNG");
        let mn = field.iter().copied().fold(f32::INFINITY, f32::min);
        let mx = field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(mn, 0.0);
        assert_eq!(mx, 1.0);
    }

    #[test]
    fn decode_rejects_non_png_bytes_rather_than_panicking() {
        assert!(matches!(decode_heightmap(b"definitely not a png", 64), Err(ImportError::Decode(_))));
    }

    /// A flat heightmap has no relief at all -- every cell ties. It must
    /// still import without panicking or producing NaN, because a user
    /// really can drop a blank image in.
    #[test]
    fn a_featureless_heightmap_imports_without_panicking() {
        let rgba = vec![128u8; 32 * 32 * 4];
        let img = cartalith_assets::raster::DecodedImage::new(32, 32, rgba).expect("sized correctly");
        let png = cartalith_assets::raster::encode_png(&img).expect("valid image");
        let base = WorldParams::defaults(32, 0, 1);
        let out = import_heightmap(&png, &base).expect("a flat image is still a valid heightmap");
        assert!(out.state.field.iter().all(|v| v.is_finite()));
        assert!(out.state.stress_field.iter().all(|v| v.is_finite()));
        assert!(out.state.rainfall.iter().all(|v| v.is_finite()));
    }
}
