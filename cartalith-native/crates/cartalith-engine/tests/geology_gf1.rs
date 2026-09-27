//! GF-1 of `GEOLOGY_FIRST_SCOPE.md`: the lithology model and column, read by
//! no process.
//!
//! - **Identity by control flow.** With `geology_model` on, every array
//!   `generate_terrain` produced before GF-1 is bit-identical to the switch
//!   off (done-means, §7 GF-1).
//! - **Not circular.** The column is built from pre-erosion causes only, so
//!   changing erosion or climate must not move a single byte of it -- the
//!   property the legacy `build_lithology` label lacks (§1.2).
//! - **Alive and consistent** on real worlds (B10's shape, at test size).

use cartalith_engine::{generate_terrain, Geology, GeologyAbsent, WorldParams, WorldState};
use cartalith_terrain::geology::{Formation, GeologyColumn, Rock, NO_LAYER};

const GW: usize = 256;
const GH: usize = 164;

fn params(seed: i32, app_like: bool) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    if app_like {
        // The app boundary's divergences (`cartalith-godot`'s
        // `params::defaults`), which this crate cannot import.
        p.crater.physical_model = true;
        p.volc.exclude_transform = true;
        p.volc.edifice_model = true;
        p.integrate_drainage = true;
        p.tect.narrow_plate_base_blur = true;
        p.passes.glacial = true;
    }
    p
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Every pre-GF-1 array, compared bit for bit.
fn assert_same_world(a: &WorldState, b: &WorldState, label: &str) {
    assert_eq!(a.sea_level.to_bits(), b.sea_level.to_bits(), "{label}: sea_level");
    assert_eq!(bits(&a.field), bits(&b.field), "{label}: field");
    assert_eq!(a.plate_id, b.plate_id, "{label}: plate_id");
    assert_eq!(a.boundary_mask, b.boundary_mask, "{label}: boundary_mask");
    assert_eq!(bits(&a.stress_field), bits(&b.stress_field), "{label}: stress");
    assert_eq!(bits(&a.age_field), bits(&b.age_field), "{label}: age");
    assert_eq!(bits(&a.resistance_field), bits(&b.resistance_field), "{label}: resistance");
    assert_eq!(bits(&a.crust_field), bits(&b.crust_field), "{label}: crust");
    assert_eq!(a.boundary_type, b.boundary_type, "{label}: boundary_type");
    assert_eq!(bits(&a.shear_field), bits(&b.shear_field), "{label}: shear");
    assert_eq!(bits(&a.volcanic_field), bits(&b.volcanic_field), "{label}: volcanic");
    assert_eq!(bits(&a.impact_field), bits(&b.impact_field), "{label}: impact");
    assert_eq!(bits(&a.temperature), bits(&b.temperature), "{label}: temperature");
    assert_eq!(bits(&a.rainfall), bits(&b.rainfall), "{label}: rainfall");
    assert_eq!(bits(&a.flow_discharge), bits(&b.flow_discharge), "{label}: flow");
    assert_eq!(a.integrated_drainage, b.integrated_drainage, "{label}: integrated_drainage");
    assert_eq!(a.channels.as_ref().map(|c| (&c.recv, &c.chan)), b.channels.as_ref().map(|c| (&c.recv, &c.chan)), "{label}: channels");
    assert_eq!(
        a.channels.as_ref().map(|c| bits(&c.intensity)),
        b.channels.as_ref().map(|c| bits(&c.intensity)),
        "{label}: intensity"
    );
    assert_eq!(a.stream_order, b.stream_order, "{label}: stream_order");
    assert_eq!(a.river_mask, b.river_mask, "{label}: river_mask");
    assert_eq!(a.river_floor.as_deref().map(bits), b.river_floor.as_deref().map(bits), "{label}: river_floor");
    assert_eq!(a.gpu_stages_used, b.gpu_stages_used, "{label}: gpu_stages_used");
}

fn column(ws: &WorldState) -> &GeologyColumn {
    ws.geology.column().expect("geology_model on must store a column")
}

fn assert_same_column(a: &GeologyColumn, b: &GeologyColumn, label: &str) {
    assert_eq!(a.rock_top, b.rock_top, "{label}: rock_top");
    assert_eq!(a.rock_sub, b.rock_sub, "{label}: rock_sub");
    assert_eq!(bits(&a.contact), bits(&b.contact), "{label}: contact");
    assert_eq!(bits(&a.regolith), bits(&b.regolith), "{label}: regolith");
    assert_eq!(a.volcanic_setting, b.volcanic_setting, "{label}: volcanic_setting");
}

#[test]
fn geology_stage_leaves_every_other_array_bit_identical() {
    for &(seed, app) in &[(12345, false), (12345, true), (314159, true), (24601, true)] {
        let off = params(seed, app);
        let mut on = off.clone();
        on.geology_model = true;
        let a = generate_terrain(&off);
        let b = generate_terrain(&on);
        assert!(matches!(a.geology, Geology::Absent(GeologyAbsent::ModelOff)), "off must store no column");
        assert_eq!(column(&b).len(), GW * GH, "the column must cover the grid");
        assert_same_world(&a, &b, &format!("seed {seed} app {app}"));
    }
}

/// The volcano setting survives when volcanism runs the simple (unprovinced)
/// stamper too, and that path is still identity.
#[test]
fn simple_volcanism_path_is_identity_and_marks_unclassified() {
    let mut off = params(2026, true);
    off.volc.provinces = false;
    let mut on = off.clone();
    on.geology_model = true;
    let a = generate_terrain(&off);
    let b = generate_terrain(&on);
    assert_same_world(&a, &b, "simple volcanism");
    let c = column(&b);
    let codes: std::collections::BTreeSet<u8> = c.volcanic_setting.iter().copied().collect();
    assert!(codes.contains(&4), "simple stamper placements are unclassified: {codes:?}");
    assert!(codes.iter().all(|&k| k == 0 || k == 4), "the simple stamper rolls no setting: {codes:?}");
}

/// The fix for `build_lithology`'s circularity: the column is a function of
/// pre-erosion causes only. Erosion off, every pass on, the stream-power
/// constants and the whole climate moved -- the column must not change.
#[test]
fn the_column_does_not_depend_on_erosion_or_climate() {
    let mut base = params(483920, true);
    base.geology_model = true;
    let ref_ws = generate_terrain(&base);
    let reference = column(&ref_ws);

    let mut no_erosion = base.clone();
    no_erosion.carve_rivers = false;
    no_erosion.passes = cartalith_engine::ErosionPassParams::off();
    let mut more_erosion = base.clone();
    more_erosion.passes.coastal = true;
    more_erosion.passes.hillslope = true;
    more_erosion.stream.k *= 3.0;
    more_erosion.tect.dynamic_lithology = true;
    let mut other_climate = base.clone();
    other_climate.climate.equator_temp += 12.0;
    other_climate.climate.wind_manual = true;
    other_climate.climate.wind_dir_deg = 200.0;

    for (label, p) in [("no erosion", no_erosion), ("more erosion", more_erosion), ("other climate", other_climate)] {
        let ws = generate_terrain(&p);
        // The variant really did change the finished terrain or climate --
        // otherwise this test could not fail.
        assert!(
            bits(&ws.field) != bits(&ref_ws.field) || bits(&ws.rainfall) != bits(&ref_ws.rainfall),
            "{label}: the variant changed nothing downstream, so it proves nothing"
        );
        assert_same_column(reference, column(&ws), label);
    }
}

#[test]
fn the_column_is_deterministic() {
    let mut p = params(71077345, true);
    p.geology_model = true;
    let a = generate_terrain(&p);
    let b = generate_terrain(&p);
    assert_same_column(column(&a), column(&b), "same seed twice");
}

/// B10's shape at test size, and the column's own invariants, over real
/// worlds.
#[test]
fn real_worlds_have_a_live_consistent_column() {
    for seed in [483920, 24601, 71077345, 12345, 314159] {
        let mut p = params(seed, true);
        p.geology_model = true;
        let ws = generate_terrain(&p);
        let c = column(&ws);
        let n = GW * GH;
        let sea = ws.sea_level as f32;
        let land: Vec<usize> = (0..n).filter(|&i| ws.field[i] > sea).collect();
        assert!(land.len() > 1000, "seed {seed}: too little land to judge");

        let mut present = [0usize; 11];
        let mut two_layer = 0usize;
        for i in 0..n {
            let top = Rock::from_u8(c.rock_top[i]).unwrap_or_else(|| panic!("seed {seed}: rock_top {} at {i}", c.rock_top[i]));
            assert_ne!(top, Rock::Unconsolidated, "GF-1 writes no regolith rock as bedrock");
            assert_eq!(c.regolith[i], 0.0, "GF-1 writes regolith 0 (bare rock) everywhere");
            match c.substrate(i) {
                None => {
                    assert_eq!(c.rock_sub[i], NO_LAYER, "seed {seed}: sub code without a contact at {i}");
                    assert!(c.contact[i].is_nan(), "seed {seed}: a single-layer cell stores a plausible contact at {i}");
                }
                Some((sub, _)) => {
                    // Formation order: the upper unit is never older.
                    assert!(top.props().formation >= sub.props().formation, "seed {seed}: {top:?} over {sub:?}");
                    assert!(sub.props().formation < Formation::Regolith);
                }
            }
            // Oceanic crust is oceanic basalt, or volcanics built on it.
            if ws.crust_field[i] < 0.0 {
                assert!(
                    matches!(top, Rock::OceanicBasalt | Rock::Andesite | Rock::Tuff),
                    "seed {seed}: oceanic crust topped by {top:?}"
                );
            }
            // A volcanic setting only where volcanism reached the cell.
            if c.volcanic_setting[i] != 0 {
                assert!(ws.volcanic_field[i] > 0.0, "seed {seed}: a setting with no volcano at {i}");
            }
        }
        for &i in &land {
            present[c.rock_top[i] as usize] += 1;
            if c.substrate(i).is_some() {
                two_layer += 1;
            }
        }
        let kinds = present.iter().filter(|&&k| k > 0).count();
        let share = two_layer as f64 / land.len() as f64;
        println!("seed {seed}: {kinds} rock types on land, two-layer share {share:.4}, counts {present:?}");
        assert!(kinds >= 4, "seed {seed}: only {kinds} rock types on land");
        assert!(two_layer > 0, "seed {seed}: no two-layer cell on land");
    }
}
