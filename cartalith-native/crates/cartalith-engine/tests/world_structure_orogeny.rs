//! Ruling AS (2026-09-24): with World Structure on, the orogeny knobs are the
//! reference's own derivation, not the `0.16`/`1.0`/`0` this port hardcoded
//! until then. `deriveFromWorldStructure()` (v2.10 2536-2538) sets
//! `foldIntensity = +(0.6 + tectonicEnergy).toFixed(3)` and
//! `trenchDepth = +(0.7 + 0.8·oceanDepth).toFixed(3)`; the orogeny call (v2.10
//! 3442) passes `foldK: 0.16·foldIntensity`, `trenchK: trenchDepth`,
//! `faultBlockK: state.tect.faultBlock`, which is `0.6` (v2.10 2265).
//!
//! Every expected value is a literal computed outside this build (IEEE-754
//! doubles, the same arithmetic V8 does), never the function under test.

use cartalith_engine::{generate_terrain, world_structure_orogeny_ks, WorldParams, WorldStructureParams};

fn ws(tectonic_energy: f64, ocean_depth: f64) -> WorldStructureParams {
    WorldStructureParams {
        enabled: true,
        continentality: 0.30,
        fragmentation: 0.50,
        tectonic_energy,
        ocean_depth,
        hotspot_density: 0.20,
    }
}

#[test]
fn the_archetype_derivation_is_the_references() {
    // (tectonicEnergy, oceanDepth) -> (foldK, trenchK, faultBlockK).
    // earth (0.6/0.6), volcanic (0.9/0.8) and rift (0.75/0.55) are the
    // reference's own `ARCHETYPES` rows; (0.5, 0.7) is a case where
    // `toFixed(3)` is load-bearing: 0.7 + 0.7·0.8 is 1.2599999999999998 in
    // doubles and the reference keeps 1.26 (rift's 1.1400000000000001 -> 1.14
    // likewise).
    let cases = [
        ((0.6, 0.6), (0.192, 1.18, 0.6)),
        ((0.9, 0.8), (0.24, 1.34, 0.6)),
        ((0.75, 0.55), (0.21600000000000003, 1.14, 0.6)),
        ((0.5, 0.7), (0.17600000000000002, 1.26, 0.6)),
    ];
    for ((te, od), want) in cases {
        assert_eq!(world_structure_orogeny_ks(&ws(te, od)), want, "te={te} od={od}");
    }
}

/// The values this port used before Ruling AS, which the reference never
/// reaches with World Structure on — so none of them may come back.
#[test]
fn the_old_hardcoded_knobs_are_gone() {
    let (fold_k, trench_k, fault_block_k) = world_structure_orogeny_ks(&ws(0.6, 0.6));
    assert_ne!(fold_k, 0.16);
    assert_ne!(trench_k, 1.0);
    assert!(fault_block_k > 0.0, "the horst-and-graben branch must be able to run");
}

fn fnv1a_field(field: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in field {
        for b in v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

fn ws_world() -> WorldParams {
    let mut p = WorldParams::defaults(128, 64, 4242);
    p.use_gpu = false;
    p.world_structure = ws(0.75, 0.55);
    p.world_structure.continentality = 0.40;
    p.world_structure.fragmentation = 0.35;
    p.world_structure.hotspot_density = 0.30;
    p
}

/// `world_structure_orogeny_ks` is what `generate_terrain` actually feeds
/// `build_orogeny_field`: a rift-archetype World-Structure world, hashed.
/// **Re-baselined by Ruling AS (2026-09-24)**: with the pre-ruling hardcoded
/// `OrogenyParams { fold_k: 0.16, trench_k: 1.0, fault_block_k: 0.0 }` this
/// same world hashed `0xe8fa_007d_9d7d_50cb`; with the reference's derivation it
/// hashes the literal below. No hash of a World-Structure world existed before this test.
#[test]
fn a_world_structure_world_uses_the_derived_knobs() {
    let got = fnv1a_field(&generate_terrain(&ws_world()).field);
    println!("ws world hash {got:#018x}");
    assert_eq!(got, 0x916a_1193_0abe_f69e);
}

/// The same world with World Structure off takes no orogeny path at all, so
/// Ruling AS cannot have moved it: measured `0x34ab_5acc_582a_f633` both with
/// the pre-ruling hardcoded knobs and with the derivation.
#[test]
fn a_world_without_world_structure_is_untouched() {
    let mut p = ws_world();
    p.world_structure.enabled = false;
    let got = fnv1a_field(&generate_terrain(&p).field);
    println!("non-ws world hash {got:#018x}");
    assert_eq!(got, 0x34ab_5acc_582a_f633);
}
