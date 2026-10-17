//! GF-6's switch-off identity check: app worlds (`cartalith-godot`'s
//! `params::defaults()`, `geology_karst` off, so no `KarstView`) give the same
//! landmark result before and after GF-6, apart from cave's own funnel row,
//! which changes from `not_buildable` to `no_terrain` because the kind is now
//! built and its input is absent.
//!
//! Deliberately uses only API that predates GF-6, so the same file builds
//! against the commit before it: run it on both builds and diff the printed
//! lines (`GEOLOGY_FIRST_SCOPE.md` §7 "GF-6 measured" records the result).
//!
//! `cargo test -j 2 --release -p cartalith-civ --test gf6_identity_hashes -- --ignored --nocapture`

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_civ::landmark::{self, LandmarkInputs, LandmarkSettings};
use cartalith_engine::generate_terrain;
use std::hash::{Hash, Hasher};

/// Prints, per seed, a hash of every placement and every funnel row except
/// cave's, then cave's row on its own. Protects the claim that GF-6 moves
/// nothing an app world shows; it asserts nothing itself, because the
/// comparison is between two builds.
#[test]
#[ignore]
fn gf6_identity_hashes() {
    for seed in [483_920, 314_159] {
        let mut p = params::defaults();
        p.gw = 512;
        p.gh = 328;
        p.tect.seed = seed;
        p.map_width_km = 800.0;
        let ws = generate_terrain(&p);
        let wb = cartalith_civ::build_water_bodies(&ws.field, p.gw, p.gh, ws.sea_level, p.world, Some(&ws.rainfall));
        let mut inp = LandmarkInputs::new(&ws.field, p.gw, p.gh, ws.sea_level, p.world, p.map_width_km);
        inp.peak_m = p.peak_m;
        inp.flow = Some(&ws.flow_discharge);
        inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
        inp.recv = ws.channels.as_ref().map(|c| c.recv.as_slice());
        inp.order = ws.stream_order.as_deref();
        inp.water = Some(&wb.classification);
        inp.volcanism = Some(&ws.volcanic_field);
        inp.resistance = Some(&ws.resistance_field);
        let r = landmark::generate(&inp, &LandmarkSettings::default(), seed as u64);
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for l in &r.landmarks {
            (&l.kind, l.x, l.y, l.score.to_bits(), l.elevation.to_bits(), l.importance.to_bits(), &l.causal, l.seed).hash(&mut h);
        }
        for f in r.funnels.iter().filter(|f| f.kind != "cave") {
            (&f.kind, f.candidates, f.rejected_constraint, f.rejected_score, f.rejected_spacing, f.rejected_cap, f.cap, f.placed, f.limit.as_str()).hash(&mut h);
        }
        let c = r.funnel("cave").unwrap();
        println!(
            "GF6ID seed {seed}: {} landmarks, hash {:016x}; cave {}/{}/{}",
            r.landmarks.len(),
            h.finish(),
            c.candidates,
            c.placed,
            c.limit.as_str()
        );
    }
}
