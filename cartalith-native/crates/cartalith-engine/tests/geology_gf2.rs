//! GF-2 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ), on whole
//! generated worlds: stream power reads rock, rebound lifts the column,
//! deposition becomes regolith.
//!
//! **What these tests protect.**
//! - The switches: `geology_processes` (off at both boundaries until §5.6's
//!   bars pass) acts only with `geology_model` on. With the model off, the
//!   processes switch changes nothing, bit for bit -- the parity path is
//!   untouched by control flow. The app-default world (model on, processes
//!   off) is GF-1's, asserted by `geology_gf1.rs` and hashed against HEAD in
//!   `GEOLOGY_FIRST_SCOPE.md` §5.6.
//! - With it on, the world *does* change (a GF-2 that moved nothing would
//!   pass every identity test and be inert).
//! - The evolved column stays well formed: regolith finite and never
//!   negative, contacts only where there is a second layer and only raised
//!   (rebound lifts; nothing in GF-2 lowers a contact), and the derivation
//!   itself untouched.
//!
//! - Since GF-3, the same switch runs the threshold hillslope stage (§4.3);
//!   the light-pass replay below replays it too, at an extent where it must
//!   move cells.
//!
//! The kernel's literal behaviour (κ lookup, contact switch, stripping) is
//! pinned in `cartalith-erosion/tests/gf2_rock_stream_power.rs`, and the
//! threshold stage's in `cartalith-erosion/tests/gf3_threshold_hillslope.rs`.

use cartalith_engine::{generate_terrain, WorldParams};

/// The treatment: GF-2's processes on (they are off at both boundaries,
/// `GEOLOGY_FIRST_SCOPE.md` §5.6, so every test here turns them on itself).
fn treated(p: &WorldParams) -> cartalith_engine::WorldState {
    generate_terrain(&WorldParams { geology_processes: true, ..p.clone() })
}

/// The control: the same world with the processes off -- GF-1's inert column.
fn inert(p: &WorldParams) -> cartalith_engine::WorldState {
    generate_terrain(&WorldParams { geology_processes: false, ..p.clone() })
}
use cartalith_terrain::geology::NO_LAYER;

/// This suite's fixed grid width -- the same size `geology_gf1.rs` uses.
const GW: usize = 256;
/// This suite's fixed grid height, paired with [`GW`].
const GH: usize = 164;

/// The app boundary's divergences (`cartalith-godot`'s `params::defaults`),
/// which this crate cannot import -- the same list `geology_gf1.rs` keeps.
fn app_params(seed: i32) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    // GF-2's processes: off at both boundaries (§5.6), on for every test
    // here. They act only once a test also turns `geology_model` on.
    p.geology_processes = true;
    p
}

/// Raw bits of a float slice, for comparisons that must not launder NaN or
/// signed-zero through `==`.
fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// With `geology_model` off there is no column, so turning the processes on
/// changes nothing, bit for bit -- on the parity baseline and on an app-like
/// world with the model off.
#[test]
fn with_the_switch_off_rock_reads_change_nothing() {
    // Protects: with geology_model off there is no column to read, so
    // turning geology_processes on changes nothing bit for bit -- both on
    // the parity baseline and an app-like world.
    for p in [WorldParams::defaults(GW, GH, 12345), app_params(314159)] {
        assert!(!p.geology_model);
        let a = treated(&p);
        let b = inert(&p);
        assert_eq!(bits(&a.field), bits(&b.field), "field");
        assert_eq!(bits(&a.rainfall), bits(&b.rainfall), "rainfall");
        assert_eq!(a.stream_order, b.stream_order, "stream_order");
        assert_eq!(a.river_mask, b.river_mask, "river_mask");
    }
}

/// On, at contrast 0 (`tect.resist = 0`, which §4.1 repurposes as `c`):
/// every κ^c is exactly 1 and the legacy factor `max(0.05, 1 − 0)` is exactly
/// 1, so the treatment's terrain must equal the control's bit for bit. That
/// proves the whole GF-2 wiring -- the kernel swap at every call site, the
/// rebound's column lift, the regolith bookkeeping -- writes nothing to the
/// field except through κ.
#[test]
fn at_contrast_zero_the_treatment_terrain_is_the_controls() {
    // Protects: the whole GF-2 wiring (kernel swap at every call site,
    // rebound's column lift, regolith bookkeeping) writes nothing to the
    // field except through kappa -- proven by forcing kappa^c to exactly 1.
    let mut p = app_params(24601);
    p.geology_model = true;
    p.tect.resist = 0.0;
    p.passes.evolve_cycles = 1;
    p.passes.sediment_fill = true;
    let t = generate_terrain(&p);
    let c = inert(&p);
    assert_eq!(bits(&t.field), bits(&c.field), "field");
    assert_eq!(bits(&t.rainfall), bits(&c.rainfall), "rainfall");
    assert_eq!(t.river_mask, c.river_mask, "river_mask");
    // ... while the column did evolve (so the arms really differ in path).
    assert_ne!(bits(&t.geology.column().unwrap().regolith), bits(&c.geology.column().unwrap().regolith), "regolith");
}

/// On: the treatment moves the terrain, and on a later pass too (evolve
/// cycles and sediment fill take the same rock path).
#[test]
fn with_the_switch_on_the_world_changes() {
    // Protects: GF-2 actually moves the terrain (a no-op GF-2 would pass
    // every identity test above and be inert), on both the light pass and a
    // later pass (evolve cycles + sediment fill).
    let mut p = app_params(483920);
    p.geology_model = true;
    let treated = generate_terrain(&p);
    let control = inert(&p);
    let moved = treated.field.iter().zip(control.field.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    assert!(moved > GW * GH / 10, "GF-2 moved only {moved} of {} cells", GW * GH);

    let mut q = p.clone();
    q.passes.evolve_cycles = 1;
    q.passes.sediment_fill = true;
    let t2 = generate_terrain(&q);
    let c2 = inert(&q);
    assert_ne!(bits(&t2.field), bits(&c2.field), "the later stream-power passes must read rock too");
    // Sediment routing deposits regolith somewhere the light pass alone did not.
    let reg_more = t2.geology.column().unwrap().regolith.iter().zip(treated.geology.column().unwrap().regolith.iter()).any(|(a, b)| a != b);
    assert!(reg_more, "evolve + sediment fill left the regolith unchanged");
}

/// Protects the glacial call site's regolith-first stripping (§4.9) by exact
/// replay. The glacial pass is the only pass on, and the passes run after the
/// hydrology, so the same world with glacial off is exactly the state the pass
/// received (field, temperature, column). Replaying `glacial_kernel` on it,
/// the column must equal `account_regolith(strip only)` of that lowering, bit
/// for bit, and rebound + the passes' clamp must reproduce the field -- or
/// this is not the pass the world ran.
#[test]
fn glacial_lowering_strips_regolith_first_exactly() {
    // Protects: §4.9's regolith-first stripping at the glacial call site
    // specifically, by exact replay of glacial_kernel + rebound + clamp
    // against the world the pass actually ran on.
    let mut p = app_params(24601);
    p.geology_model = true;
    let mut off = p.clone();
    off.passes.glacial = false;
    assert!(!off.passes.any());
    let w0 = generate_terrain(&off);
    let w1 = generate_terrain(&p);
    let q = &p.passes;
    let mut after_kernel = w0.field.to_vec();
    cartalith_erosion::glacial_kernel(
        &mut after_kernel,
        &w0.temperature,
        GW,
        GH,
        &cartalith_erosion::GlacialParams {
            kg: q.glacial_kg,
            mg: q.glacial_mg,
            snowline: q.glacial_snowline,
            u_factor: q.glacial_u_factor,
            passes: q.glacial_passes,
            g: p.planet.g,
            sea: w0.sea_level,
            world: p.world,
        },
    );
    let mut replay = after_kernel.clone();
    cartalith_erosion::isostatic_rebound(&mut replay, &w0.field, GW, GH, p.tect.blur_r, p.world);
    for v in replay.iter_mut() {
        *v = v.clamp(0.0, 1.0);
    }
    assert_eq!(bits(&replay), bits(&w1.field), "the replay is not the pass the world ran");
    let mut col = w0.geology.column().unwrap().clone();
    let had = col.regolith.iter().zip(w0.field.iter().zip(after_kernel.iter())).filter(|(r, (a, b))| **r > 0.0 && b < a).count();
    assert!(had > 0, "no glacially lowered cell held regolith, so this test cannot fail");
    cartalith_erosion::account_regolith(&mut col, &w0.field, &after_kernel, false);
    assert_eq!(bits(&col.regolith), bits(&w1.geology.column().unwrap().regolith), "glacial stripping");
}

/// Protects the light pass end to end and the RV-1 carve's regolith-first
/// stripping (§4.9), by exact replay from the pre-erosion world (carve off,
/// passes off: nothing after the geology stage writes the field or the column
/// there). Replayed: the rock kernel, §4.9's net-change accounting, rebound
/// with the column lifted (§4.2), and since GF-3 the threshold hillslope
/// stage (§4.3: `THRESHOLD_HILLSLOPE_PASSES` passes at `cell_m =
/// map_width_km·1000/gw`, its net change accounted as regolith). The world
/// with passes off then differs from that replay only by the carve, so the
/// carve's lowering must have stripped regolith exactly as `account_regolith`
/// would.
///
/// Run at the default 800 km and at 20 km. At 800 km over 256 cells (3.1 km
/// cells) the threshold stage has nothing over-steep to move; at 20 km (78 m
/// cells) it must move cells, or its call site is not what is being replayed.
#[test]
fn the_light_pass_and_the_carve_replay_exactly() {
    // Protects: the light pass end to end and RV-1 carve's regolith-first
    // stripping, at both a scale where the GF-3 threshold stage has nothing
    // to move (800 km) and one where it must move cells (20 km) -- see
    // light_pass_and_carve_replay's own doc for the full replay chain.
    light_pass_and_carve_replay(800.0, false);
    light_pass_and_carve_replay(20.0, true);
}

/// The shared body [`the_light_pass_and_the_carve_replay_exactly`] runs
/// twice: replays the rock kernel, §4.9's net-change accounting, rebound
/// with the lifted column, and the GF-3 threshold hillslope stage, then
/// checks the carve's own lowering stripped regolith exactly as
/// `account_regolith` would.
fn light_pass_and_carve_replay(km: f64, threshold_must_move: bool) {
    let mut p = app_params(483920);
    p.geology_model = true;
    p.map_width_km = km;
    p.passes = cartalith_engine::ErosionPassParams::off();
    let mut pre_p = p.clone();
    pre_p.carve_rivers = false;
    let pre = generate_terrain(&pre_p);
    let w = generate_terrain(&p);
    let sea = pre.sea_level;
    let mut field = pre.field.to_vec();
    let mut col = pre.geology.column().unwrap().clone();
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    // `generate_terrain_inner`'s light pass: max(4, round(iters·0.6)).
    let iters = ((p.stream.iters as f64 * 0.6).round() as i32).max(4);
    let sp = cartalith_erosion::StreamPowerParams {
        k: p.stream.k,
        uplift: p.stream.uplift,
        deposit: p.stream.deposit,
        climate_k: p.stream.climate_k,
        iters,
        resist: p.tect.resist,
        g: p.planet.g,
        world: p.world,
        sea,
    };
    let start = field.clone();
    cartalith_erosion::stream_power_kernel_rock(
        &mut field,
        &pre.stress_field,
        &pre.rainfall,
        GW,
        GH,
        &sp,
        &mut cartalith_erosion::StreamPowerRock { column: &mut col, contrast: p.tect.resist, r_expose },
    );
    cartalith_erosion::account_regolith(&mut col, &start, &field, true);
    let before_rebound = field.clone();
    cartalith_erosion::isostatic_rebound(&mut field, &start, GW, GH, p.tect.blur_r, p.world);
    cartalith_erosion::lift_column(&mut col, &before_rebound, &field);
    // GF-3's threshold hillslope, as `RockContext::threshold_hillslope` runs it.
    let before_threshold = field.clone();
    cartalith_erosion::threshold_hillslope(
        &mut field,
        GW,
        GH,
        cartalith_erosion::THRESHOLD_HILLSLOPE_PASSES,
        p.world,
        &cartalith_erosion::ThresholdHillslope { column: &col, r_expose, cell_m: km * 1000.0 / GW as f64, sea, peak_m: p.peak_m },
    );
    let moved = before_threshold.iter().zip(field.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    if threshold_must_move {
        assert!(moved > 0, "{km} km: the threshold stage moved nothing, so its replay cannot fail");
    }
    cartalith_erosion::account_regolith(&mut col, &before_threshold, &field, true);
    // Everything off the carve's footprint must already match, or the replay
    // is not the pass the world ran.
    let mask = w.river_mask.as_ref().unwrap();
    let carved: Vec<usize> = (0..GW * GH).filter(|&i| w.field[i].to_bits() != field[i].to_bits()).collect();
    assert!(!carved.is_empty(), "the carve moved nothing, so this test cannot fail");
    assert!(carved.iter().all(|&i| mask[i] != 0), "the replay differs off the carve's footprint");
    assert_eq!(bits(&col.contact), bits(&w.geology.column().unwrap().contact), "rebound's column lift");
    let stripped = carved.iter().filter(|&&i| col.regolith[i] > 0.0 && w.field[i] < field[i]).count();
    assert!(stripped > 0, "no carved cell held regolith, so this test cannot fail");
    cartalith_erosion::account_regolith(&mut col, &field, &w.field, false);
    assert_eq!(bits(&col.regolith), bits(&w.geology.column().unwrap().regolith), "carve stripping");
}

/// Protects the sediment-routing call site (§4.9: routed deposits become
/// regolith) by exact replay of the `sediment_fill` pass on the state it
/// received -- the same world with that pass off.
#[test]
fn routed_sediment_becomes_regolith_exactly() {
    // Protects: §4.9's sediment-routing call site -- routed deposits become
    // regolith, by exact replay of the sediment_fill pass against the same
    // world with that pass off.
    let mut p = app_params(314159);
    p.geology_model = true;
    p.passes.glacial = false;
    let off = p.clone();
    p.passes.sediment_fill = true;
    let w0 = generate_terrain(&off);
    let w1 = generate_terrain(&p);
    let sea = w0.sea_level;
    let mut field = w0.field.to_vec();
    let mut col = w0.geology.column().unwrap().clone();
    let r_expose = cartalith_terrain::geology::m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32;
    let sp = cartalith_erosion::StreamPowerParams {
        k: p.stream.k,
        uplift: p.stream.uplift,
        deposit: p.stream.deposit,
        climate_k: p.stream.climate_k,
        iters: p.stream.iters,
        resist: p.tect.resist,
        g: p.planet.g,
        world: p.world,
        sea,
    };
    let pre = field.clone();
    cartalith_erosion::stream_power_kernel_rock(
        &mut field,
        &w0.stress_field,
        &w0.rainfall,
        GW,
        GH,
        &sp,
        &mut cartalith_erosion::StreamPowerRock { column: &mut col, contrast: p.tect.resist, r_expose },
    );
    cartalith_erosion::account_regolith(&mut col, &pre, &field, true);
    let supply: Vec<f32> = (0..GW * GH).map(|i| ((pre[i] as f64 - field[i] as f64) as f32).max(0.0)).collect();
    let flow = cartalith_hydrology::compute_flow_routed(GW, GH, &field, Some(&w0.rainfall), true, p.world, sea, p.integrate_drainage);
    let before_route = field.clone();
    cartalith_erosion::route_sediment(&mut field, &flow, &supply, GW, GH, sea, p.passes.sediment_capacity, p.world);
    let routed = before_route.iter().zip(field.iter()).filter(|(a, b)| b > a).count();
    assert!(routed > 0, "routing deposited nothing, so this test cannot fail");
    cartalith_erosion::account_regolith(&mut col, &before_route, &field, true);
    for v in field.iter_mut() {
        *v = v.clamp(0.0, 1.0);
    }
    assert_eq!(bits(&field), bits(&w1.field), "the replay is not the pass the world ran");
    assert_eq!(bits(&col.regolith), bits(&w1.geology.column().unwrap().regolith), "routed deposits");
}

/// The evolved column is well formed on five worlds.
#[test]
fn the_evolved_column_stays_well_formed() {
    // Protects: the evolved column's own invariants -- regolith finite and
    // never negative, contacts only where there is a second layer and only
    // ever raised (nothing in GF-2 lowers one), and the derivation itself
    // untouched -- over several real, generated worlds.
    for seed in [483920, 24601, 71077345, 12345, 314159] {
        let mut p = app_params(seed);
        p.geology_model = true;
        let t = generate_terrain(&p);
        let c = inert(&p);
        let (tc, cc) = (t.geology.column().unwrap(), c.geology.column().unwrap());
        assert_eq!(tc.rock_top, cc.rock_top, "seed {seed}: processes must not re-derive the rock");
        assert_eq!(tc.rock_sub, cc.rock_sub, "seed {seed}: rock_sub");
        assert_eq!(tc.volcanic_setting, cc.volcanic_setting, "seed {seed}: setting");
        let mut deposited = 0usize;
        for i in 0..GW * GH {
            let r = tc.regolith[i];
            assert!(r.is_finite() && r >= 0.0, "seed {seed}: regolith {r} at {i}");
            if r > 0.0 {
                deposited += 1;
            }
            if tc.rock_sub[i] == NO_LAYER {
                assert!(tc.contact[i].is_nan(), "seed {seed}: a single-layer cell grew a contact at {i}");
            } else {
                assert!(tc.contact[i].is_finite(), "seed {seed}: a two-layer contact is not finite at {i}");
                assert!(tc.contact[i] >= cc.contact[i], "seed {seed}: a contact went down at {i}: {} < {}", tc.contact[i], cc.contact[i]);
            }
        }
        // The light pass deposits (`stream.deposit = 0.3`), so some regolith
        // must exist, or deposition is not reaching the column.
        assert!(deposited > 0, "seed {seed}: no regolith anywhere after generation");
        // And the rebound lifted some contact.
        assert!(
            (0..GW * GH).any(|i| tc.rock_sub[i] != NO_LAYER && tc.contact[i] > cc.contact[i]),
            "seed {seed}: rebound lifted no contact"
        );
    }
}
