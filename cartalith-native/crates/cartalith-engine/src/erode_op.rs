//! `erode()` — the reference's droplet-erosion button, assembled.
//! `PARITY_AUDIT.md` §23 F11.
//!
//! Reference line 3898. Its body is two calls:
//!
//! ```text
//! function erode(){
//!   const p=state.erosion, ck=state.stream.climateK||0, pre=field.slice();
//!   dropletKernel(field, ck>0?rainField:null, GW, GH, dropletParams(p,ck));
//!   erodeFinish(pre,p);
//! }
//! function erodeFinish(pre,p){                                    // line 3892
//!   erodeThermal(p.thermalPasses);
//!   for(let i=0;i<field.length;i++){ if(field[i]<0)field[i]=0; else if(field[i]>1)field[i]=1; }
//!   isostaticRebound(pre);
//!   computeFlow(true); refreshClimate(); renderNow();
//! }
//! ```
//!
//! All three kernels are ported and golden-verified in the crate that owns
//! them (`cartalith_erosion::{droplet_kernel, erode_thermal,
//! isostatic_rebound}`, covered by `golden_parity_droplet`, `_thermal` and
//! `_rebound`). **Nothing ever assembled them** — this crate imported
//! `isostatic_rebound` alone, and `droplet_kernel`/`erode_thermal` appeared
//! nowhere outside their own crate and its tests. That is what §23 found.
//!
//! This module is only the assembly, which is why it sits in
//! `cartalith-engine` — *"cartalith-engine orchestrates; it does not
//! compute"*, milestone B's rule, and the same reason [`crate::region_export`]
//! and [`crate::sculpt_commit`] live here. It is also the only place it
//! **can** live: `cartalith-godot` does not depend on `cartalith-erosion` and
//! this crate re-exports none of it, so the bridge cannot name those three
//! functions at all.
//!
//! # This is an OP, not a generation stage
//!
//! In the reference `erode()` is a button over the finished `field`;
//! `generate()` never runs it, and `state.erosion`'s values are op parameters.
//! That is preserved exactly:
//!
//! - [`generate_terrain`](crate::generate_terrain) does not call anything in
//!   this module, and this module is not referenced from `lib.rs` beyond its
//!   `pub mod` line.
//! - [`ErodeOpts`] is a **separate struct**, not fields on
//!   [`WorldParams`](crate::WorldParams). No generation-derived hash, no
//!   `world_key`, no golden fixture and no `save_round_trip` input can move
//!   because this exists.
//!
//! Contrast [`crate::ErosionPassParams`], which takes `DECISIONS.md` §7d's
//! other route for the reference's *six other* manual erosion ops (velocity,
//! glacial, coastal, hillslope, sediment-fill, tidal-flats): same kernels, run
//! at the end of generation behind default-off toggles. Droplet is the one
//! that stays a button, because thermal relaxation and isostatic rebound are
//! its tail and neither belongs in a generation pass.
//!
//! # What the caller still owns
//!
//! `erodeFinish`'s last line — `computeFlow(true); refreshClimate();
//! renderNow()` — is deliberately **not** here. In this port those are
//! [`crate::staleness::recompute_stale`] over a `Height` mark, and the shell's
//! own texture refresh. [`erode_op`] mutates the surface and reports what it
//! did; running the graph is the bridge's job, the same division
//! `commit_sculpt_pass` already follows. An op that carves a valley and leaves
//! the flow field stale is a bug, so the bridge's `#[func]` does that
//! immediately after this returns.

use crate::{WorldParams, WorldState};
use cartalith_erosion::{droplet_kernel, erode_thermal, isostatic_rebound, DropletParams};

/// `state.erosion` (reference HTML line 2268) — the fourteen knobs
/// `dropletParams()` (line 3889) bundles, plus the two `erodeFinish` (line
/// 3892) reads, at the reference's own literal defaults.
///
/// Deliberately **excludes** `diffuseD`/`diffusePasses`, which sit in the same
/// JS literal but belong to `hillslopeDiffuse`, a different button — already
/// exposed as [`ErosionPassParams::diffuse_d`](crate::ErosionPassParams) and
/// `diffuse_passes`.
///
/// Also excludes `g`, `ck` and `seed`, which `dropletParams` reads from
/// `state.planet.g`, `state.stream.climateK` and `state.tect.seed` — from the
/// *world*, not from the erosion panel. [`erode_op`] takes them off the
/// `WorldParams` it is handed, so a caller cannot accidentally erode with a
/// gravity or a seed that disagrees with the world it is eroding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErodeOpts {
    /// `#drops` — slider 0-100, stored as `Math.round(v*1500)`.
    pub droplets: i32,
    pub inertia: f64,
    pub capacity: f64,
    pub min_slope: f64,
    /// `#edep` — slider 0-100, stored as `v/100`.
    pub deposit: f64,
    /// `#estr` — slider 0-100, stored as `v/100`.
    pub erode: f64,
    pub evaporate: f64,
    pub gravity: f64,
    pub max_lifetime: i32,
    pub init_speed: f64,
    pub init_water: f64,
    pub radius: i32,
    /// `#etal` — slider 1-40, stored as `v/1000`. `erodeThermalCPU`'s angle
    /// of repose.
    pub talus: f64,
    /// `#ethr` — slider 0-30. `erodeFinish`'s `erodeThermal(p.thermalPasses)`.
    pub thermal_passes: i32,
}

impl Default for ErodeOpts {
    /// The reference's own `state.erosion` literal, reference HTML line 2268.
    fn default() -> Self {
        Self {
            droplets: 60_000,
            inertia: 0.05,
            capacity: 4.0,
            min_slope: 0.01,
            deposit: 0.30,
            erode: 0.35,
            evaporate: 0.02,
            gravity: 4.0,
            max_lifetime: 30,
            init_speed: 1.0,
            init_water: 1.0,
            radius: 3,
            talus: 0.012,
            thermal_passes: 8,
        }
    }
}

impl ErodeOpts {
    /// Clamps the two things that would otherwise panic or spin rather than
    /// misbehave: `droplet_kernel` builds its brush from `-r..=r` and divides
    /// the kernel weights by `r`, so `radius < 1` is a division by zero, and
    /// negative counts would be nonsense. Called by [`erode_op`], so a caller
    /// coming across the gdext boundary — where a panic takes the Godot
    /// process with it — cannot get there with a bad value.
    pub fn sanitized(mut self) -> Self {
        self.droplets = self.droplets.max(0);
        self.max_lifetime = self.max_lifetime.max(0);
        self.thermal_passes = self.thermal_passes.max(0);
        self.radius = self.radius.max(1);
        self
    }
}

/// What [`erode_op`] did — the summary a caller reports and a test asserts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ErodeSummary {
    pub cells_changed: usize,
    pub cells_lowered: usize,
    pub cells_raised: usize,
    /// Whether the droplets actually spawned through the rain field.
    ///
    /// `false` either because `p.stream.climate_k` is `0` (the user turned the
    /// coupling off) **or** because this world carries no usable `rainfall`
    /// and the op fell back to uniform spawning — see [`erode_op`]'s own note
    /// on why that fallback exists and why it is reported rather than hidden.
    pub climate_coupled: bool,
}

/// `erode()` + `erodeFinish()` (reference HTML lines 3892-3902), minus the
/// three tail calls that belong to the host — see the module doc.
///
/// Mutates `ws.field` in place. `p` supplies the four values the reference
/// reads from `state` rather than from the erosion panel: `p.planet.g`
/// (`dropletParams`' `g`), `p.stream.climate_k` (`erode()`'s `ck`),
/// `p.tect.seed` (`dropletParams`' `seed`) and `p.tect.blur_r` +`p.world`
/// (`isostaticRebound`'s blur radius and X-wrap).
///
/// Returns a zeroed [`ErodeSummary`] and touches nothing when the grid is
/// empty or `ws.field` does not match `p.gw * p.gh` — a mismatched field is a
/// caller bug, and refusing beats indexing off the end inside a kernel.
pub fn erode_op(ws: &mut WorldState, p: &WorldParams, opts: &ErodeOpts) -> ErodeSummary {
    let (gw, gh) = (p.gw, p.gh);
    let n = gw * gh;
    if n == 0 || ws.field.len() != n {
        return ErodeSummary::default();
    }
    let opts = opts.sanitized();

    // `erode()`'s own `ck>0?rainField:null`. `droplet_kernel` does not
    // re-validate that invariant -- it does `rain.expect("rain field required
    // when ck > 0")` inside the spawn loop -- so honouring it is this call
    // site's job, exactly as it is the JS caller's.
    //
    // **`ck` and the rain field must be decided together, and this is why.**
    // The reference's `rainField` is a module global that always exists, so
    // `ck>0` there can never find it missing. In this port a `WorldState` can
    // legitimately carry an empty `rainfall` -- an imported heightmap, a
    // partially-run pipeline -- and `state.stream.climateK` defaults to `0.5`,
    // so the DEFAULT op on such a world takes the `ck>0` branch and hits that
    // `expect`. Under a `#[func]` that panic takes the Godot process with it
    // (`cartalith-rust-conventions`). The first version of this function
    // passed `None` while leaving `ck` at `0.5` and did exactly that; two of
    // the tests below caught it.
    //
    // So a missing or wrong-sized rain field zeroes `ck` rather than being
    // dropped underneath it. That is not an invented behaviour: `ck == 0` is
    // precisely "no climate coupling", which is the honest description of a
    // world with no rain to couple to, and it is what `state.stream.climateK
    // || 0` yields whenever the coupling is unavailable. It is reported on
    // `ErodeSummary::climate_coupled` rather than hidden.
    let rain: Option<&[f32]> = (ws.rainfall.len() == n).then_some(&ws.rainfall[..]);
    let ck = if rain.is_some() { p.stream.climate_k } else { 0.0 };

    // `pre=field.slice()` — isostatic rebound is measured against the surface
    // as it stood *before* the droplets, not before the thermal pass.
    let pre: Vec<f32> = ws.field.clone();

    droplet_kernel(
        &mut ws.field,
        rain,
        gw,
        gh,
        &DropletParams {
            droplets: opts.droplets,
            inertia: opts.inertia,
            capacity: opts.capacity,
            min_slope: opts.min_slope,
            deposit: opts.deposit,
            erode: opts.erode,
            evaporate: opts.evaporate,
            gravity: opts.gravity,
            g: p.planet.g,
            max_lifetime: opts.max_lifetime,
            init_speed: opts.init_speed,
            init_water: opts.init_water,
            radius: opts.radius,
            ck,
            // JS `^` is an int32 bitwise op and `droplet_kernel` seeds with
            // `p.seed ^ 0x9e3779b9`, so `as u32` reproduces the same bit
            // pattern `mulberry32(state.tect.seed^0x9e3779b9)` gets for any
            // seed, negative included.
            seed: p.tect.seed as u32,
        },
    );
    erode_thermal(&mut ws.field, gw, gh, opts.thermal_passes, opts.talus);

    // `erodeFinish`'s clamp, written as the reference's own two-branch
    // `if/else if` rather than `f32::clamp`. Deliberate: JS lets a NaN fall
    // through both comparisons and survive, where `clamp` would absorb it --
    // the `cartalith-rust-conventions` "JS propagates NaN where Rust absorbs
    // it" rule. Reproducing the divergence is the point; a NaN that vanishes
    // here would hide the bug that produced it.
    for v in ws.field.iter_mut() {
        if *v < 0.0 {
            *v = 0.0;
        } else if *v > 1.0 {
            *v = 1.0;
        }
    }
    isostatic_rebound(&mut ws.field, &pre, gw, gh, p.tect.blur_r, p.world);

    let mut s = ErodeSummary { climate_coupled: ck > 0.0, ..Default::default() };
    for (a, b) in ws.field.iter().zip(pre.iter()) {
        if a != b {
            s.cells_changed += 1;
            if a < b {
                s.cells_lowered += 1;
            } else {
                s.cells_raised += 1;
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ridge running down the middle, plus a deterministic bit of roughness
    /// so the droplets have real gradients to follow. Not a golden fixture —
    /// the kernels' bit-exactness is already pinned by
    /// `cartalith-erosion/tests/golden_parity_*`; what this file has to prove
    /// is that the *assembly* still erodes.
    fn synthetic(gw: usize, gh: usize) -> Vec<f32> {
        let mut f = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let nx = x as f64 / (gw - 1) as f64;
                let ny = y as f64 / (gh - 1) as f64;
                let ridge = 1.0 - (nx - 0.5).abs() * 1.6;
                let bump = 0.06 * ((nx * 17.0).sin() + (ny * 11.0).cos());
                f[y * gw + x] = (0.15 + 0.7 * ridge + bump).clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    /// A `WorldState` carrying only what [`erode_op`] reads. Every other
    /// field is empty on purpose: if the op ever grows a dependency on one,
    /// this stops compiling rather than silently reading a zero.
    fn world(field: Vec<f32>, rainfall: Vec<f32>) -> WorldState {
        WorldState {
            sea_level: 0.42,
            field,
            plate_id: Vec::new(),
            boundary_mask: Vec::new(),
            stress_field: Vec::new(),
            age_field: Vec::new(),
            resistance_field: Vec::new(),
            crust_field: Vec::new(),
            boundary_type: Vec::new(),
            shear_field: Vec::new(),
            volcanic_field: Vec::new(),
            impact_field: Vec::new(),
            temperature: Vec::new(),
            rainfall,
            flow_discharge: Vec::new(),
            channels: None,
            stream_order: None,
            river_mask: None,
            river_floor: None,
            gpu_stages_used: Vec::new(),
        }
    }

    fn params(gw: usize, gh: usize, seed: i32) -> WorldParams {
        WorldParams { world: false, ..WorldParams::defaults(gw, gh, seed) }
    }

    /// The runnable check §23 F11 asks for: the op erodes, it stays inside
    /// `[0,1]`, and it is deterministic across two identical runs.
    #[test]
    fn the_op_erodes_stays_in_range_and_repeats() {
        let (gw, gh) = (48usize, 32usize);
        let opts = ErodeOpts { droplets: 400, ..Default::default() };
        let p = params(gw, gh, 12345);
        let before = synthetic(gw, gh);

        let mut a = world(before.clone(), Vec::new());
        let sa = erode_op(&mut a, &p, &opts);

        assert!(sa.cells_changed > 0, "the op must actually erode: {sa:?}");
        assert!(sa.cells_lowered > 0, "droplets must cut somewhere: {sa:?}");
        assert!(a.field != before, "the height field must change");
        assert!(
            a.field.iter().all(|&v| (0.0..=1.0).contains(&v)),
            "erodeFinish's clamp must hold the field inside [0,1]"
        );

        let mut b = world(before.clone(), Vec::new());
        let sb = erode_op(&mut b, &p, &opts);
        assert_eq!(a.field, b.field, "same seed + same field + same opts => bit-identical");
        assert_eq!(sa, sb);
    }

    /// `ck > 0` is what switches the droplet spawn to rain-weighted rejection
    /// sampling, so the rain field must actually reach the kernel — and must
    /// be ignored when `ck` is zero, which is `erode()`'s own
    /// `ck>0?rainField:null`. `WorldParams::defaults` sets `stream.climate_k`
    /// to the reference's default, so this drives it explicitly both ways.
    #[test]
    fn rain_only_bites_when_the_climate_coupling_is_on() {
        let (gw, gh) = (48usize, 32usize);
        let opts = ErodeOpts { droplets: 300, ..Default::default() };
        let base = synthetic(gw, gh);
        // All the rain weight on the left half, so a rain-driven spawn cannot
        // coincide with a uniform one.
        let rain: Vec<f32> =
            (0..gw * gh).map(|i| if i % gw < gw / 2 { 0.9 } else { 0.05 }).collect();

        let mut off = params(gw, gh, 999);
        off.stream.climate_k = 0.0;
        let mut w_off = world(base.clone(), rain.clone());
        erode_op(&mut w_off, &off, &opts);

        let mut w_none = world(base.clone(), Vec::new());
        erode_op(&mut w_none, &off, &opts);
        assert_eq!(w_off.field, w_none.field, "ck == 0 must ignore the rain field entirely");

        let mut on = params(gw, gh, 999);
        on.stream.climate_k = 0.5;
        let mut w_on = world(base.clone(), rain);
        erode_op(&mut w_on, &on, &opts);
        assert_ne!(w_on.field, w_off.field, "ck > 0 must route the droplets through the rain");
    }

    /// **The regression guard for a real panic.** `state.stream.climateK`
    /// defaults to `0.5`, so the DEFAULT op takes `droplet_kernel`'s `ck>0`
    /// branch — and that branch does `rain.expect("rain field required when
    /// ck > 0")`. A `WorldState` with no `rainfall` (an imported heightmap, a
    /// partially-run pipeline) therefore crashed the first version of this
    /// function, and would have crashed the Godot process through the
    /// `#[func]`. The op must fall back to uniform spawning and say so.
    #[test]
    fn a_world_with_no_rain_falls_back_instead_of_panicking() {
        let (gw, gh) = (32usize, 24usize);
        let p = params(gw, gh, 4242);
        assert!(p.stream.climate_k > 0.0, "this test is only meaningful while the default couples");

        // No rainfall at all.
        let mut none = world(synthetic(gw, gh), Vec::new());
        let s = erode_op(&mut none, &p, &ErodeOpts { droplets: 200, ..Default::default() });
        assert!(s.cells_changed > 0, "it must still erode, not refuse");
        assert!(!s.climate_coupled, "and must report that it ran uncoupled");

        // Rainfall of the wrong length is the same story: not sliced, not
        // trusted, and not a panic.
        let mut wrong = world(synthetic(gw, gh), vec![0.5f32; gw * gh - 1]);
        let s2 = erode_op(&mut wrong, &p, &ErodeOpts { droplets: 200, ..Default::default() });
        assert!(!s2.climate_coupled);
        assert_eq!(none.field, wrong.field, "both fell back to the same uncoupled run");
    }

    /// The other half: a full-length rain field at the default `climate_k`
    /// really does couple, so the fallback above is a fallback and not the
    /// only path.
    #[test]
    fn a_world_with_rain_couples_at_the_default_climate_k() {
        let (gw, gh) = (32usize, 24usize);
        let p = params(gw, gh, 4242);
        let rain: Vec<f32> =
            (0..gw * gh).map(|i| if i % gw < gw / 2 { 0.9 } else { 0.05 }).collect();
        let mut w = world(synthetic(gw, gh), rain);
        let s = erode_op(&mut w, &p, &ErodeOpts { droplets: 200, ..Default::default() });
        assert!(s.climate_coupled);
        assert!(s.cells_changed > 0);
    }

    /// Zero droplets and zero thermal passes: `isostatic_rebound` early-returns
    /// when nothing was removed, so the field must come back untouched. The
    /// guard that says the op is not quietly doing something of its own.
    #[test]
    fn a_zero_op_changes_nothing() {
        let (gw, gh) = (48usize, 32usize);
        let opts = ErodeOpts { droplets: 0, thermal_passes: 0, ..Default::default() };
        let before = synthetic(gw, gh);
        let mut w = world(before.clone(), Vec::new());
        let s = erode_op(&mut w, &params(gw, gh, 1), &opts);
        assert_eq!(w.field, before);
        assert_eq!(s, ErodeSummary::default());
    }

    /// A field that does not match the grid is refused rather than sliced.
    #[test]
    fn a_mismatched_field_is_refused_untouched() {
        let before = synthetic(16, 12);
        let mut w = world(before.clone(), Vec::new());
        // Claim a bigger grid than the field actually holds.
        let s = erode_op(&mut w, &params(48, 32, 7), &ErodeOpts::default());
        assert_eq!(s, ErodeSummary::default());
        assert_eq!(w.field, before, "a refusal must not have half-eroded the field");
    }

    /// `radius: 0` would divide by zero inside `droplet_kernel`'s brush
    /// builder, and this runs under a `#[func]` where a panic takes the Godot
    /// process with it. [`ErodeOpts::sanitized`] is what stops it; this is the
    /// test that says so.
    #[test]
    fn a_zero_radius_is_clamped_rather_than_dividing_by_zero() {
        assert_eq!(ErodeOpts { radius: 0, ..Default::default() }.sanitized().radius, 1);
        assert_eq!(ErodeOpts { droplets: -5, ..Default::default() }.sanitized().droplets, 0);
        assert_eq!(
            ErodeOpts { thermal_passes: -3, ..Default::default() }.sanitized().thermal_passes,
            0
        );

        let (gw, gh) = (24usize, 16usize);
        let mut w = world(synthetic(gw, gh), Vec::new());
        let opts = ErodeOpts { droplets: 40, radius: 0, ..Default::default() };
        let s = erode_op(&mut w, &params(gw, gh, 3), &opts);
        assert!(w.field.iter().all(|v| v.is_finite()), "no NaN/inf from a zero radius");
        assert!(s.cells_changed > 0);
    }

    /// The defaults are checked against the JS literal at reference line 2268,
    /// not against a comment restating it.
    #[test]
    fn defaults_match_the_reference_state_literal() {
        let d = ErodeOpts::default();
        assert_eq!(d.droplets, 60_000);
        assert_eq!(d.inertia, 0.05);
        assert_eq!(d.capacity, 4.0);
        assert_eq!(d.min_slope, 0.01);
        assert_eq!(d.deposit, 0.30);
        assert_eq!(d.erode, 0.35);
        assert_eq!(d.evaporate, 0.02);
        assert_eq!(d.gravity, 4.0);
        assert_eq!(d.max_lifetime, 30);
        assert_eq!(d.init_speed, 1.0);
        assert_eq!(d.init_water, 1.0);
        assert_eq!(d.radius, 3);
        assert_eq!(d.talus, 0.012);
        assert_eq!(d.thermal_passes, 8);
    }
}
