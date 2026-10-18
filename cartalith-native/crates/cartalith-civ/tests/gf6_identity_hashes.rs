//! GF-6's switch-off identity check for the landmark pass: app worlds
//! (`cartalith-godot`'s `params::defaults()`, `geology_karst` off, so no
//! `KarstView`) through the full default landmark pass.
//!
//! Recorded in `GEOLOGY_FIRST_SCOPE.md` §7. **The cave half** (`bfadd3fb`)
//! moved nothing but cave's own funnel row. **The rock half** moves rock
//! formation on every app world by design (it reads the column's `s`, which
//! every app world has, in place of `resistance_field`; §4.11), and through
//! cross-kind spacing the kinds near it. So the hash is printed with and
//! without rock formation's row, plus every kind's placed count, and the
//! comparison against the build before the rock half is per kind. Waterfall
//! reads no knickpoint on an app world (processes off), so its rule is the
//! rock-blind one there.
//!
//! `cargo test -j 2 --release -p cartalith-civ --test gf6_identity_hashes -- --ignored --nocapture`

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_civ::landmark::{self, LandmarkInputs, LandmarkSettings};
use cartalith_engine::generate_terrain;
use std::hash::{Hash, Hasher};

/// Prints, per seed, a hash of every placement and every funnel row except
/// cave's, every kind's placed count, then cave's row on its own. Protects
/// the record of what GF-6 moves on an app world (module doc); it asserts
/// nothing about the other build itself, because the comparison is between
/// two builds.
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
        let rock = landmark::RockView::for_world(&ws, &p).ok();
        assert!(rock.as_ref().is_some_and(|v| v.knickpoints.is_none()), "an app world: a column, no knickpoints");
        inp.rock = rock.as_ref();
        let r = landmark::generate(&inp, &LandmarkSettings::default(), seed as u64);
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for l in &r.landmarks {
            (&l.kind, l.x, l.y, l.score.to_bits(), l.elevation.to_bits(), l.importance.to_bits(), &l.causal, l.seed).hash(&mut h);
        }
        for f in r.funnels.iter().filter(|f| f.kind != "cave") {
            (&f.kind, f.candidates, f.rejected_constraint, f.rejected_score, f.rejected_spacing, f.rejected_cap, f.cap, f.placed, f.limit.as_str()).hash(&mut h);
        }
        let mut per: Vec<String> = r.funnels.iter().filter(|f| f.placed > 0).map(|f| format!("{}={}", f.kind, f.placed)).collect();
        per.sort();
        println!("GF6KINDS seed {seed}: {}", per.join(" "));
        println!("GF6RF seed {seed}: {:?}", r.funnel("rock_formation"));
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
