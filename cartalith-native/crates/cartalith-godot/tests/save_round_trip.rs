//! The end-to-end save round trip (`SAVEFILE_COMPAT.md`, `GUI_GAP_REGISTER.md`
//! FI-01): generate a **real** world through `cartalith-engine`, write it as
//! a `.zip` through `cartalith-io`, read it back through this port's own
//! `load_save`, and compare the meaningful state — not "it did not crash".
//!
//! `cartalith-io`'s own tests prove the container is right, and
//! `params_mapping.rs` proves the parameter table round-trips. Neither
//! exercises what `WorldGen::save_project` actually does: pull the six
//! fields off a live `WorldState`, saturate the `i16` stream order into the
//! save's `u8` raster, and pair them with the parameter block. That
//! extraction is the code between the two verified halves, so it is what
//! this file covers.
//!
//! `params.rs` comes in by `#[path]` (this crate is a `cdylib`, so there is
//! no rlib to link) — the same pattern `params_mapping.rs` and
//! `golden_parity_render.rs` already use. The field extraction is written
//! out again here rather than called: `lib.rs` cannot be pulled in this way
//! (it is `godot`-dependent throughout), so this asserts the *shape* the
//! binding must produce, and `save_project`'s own body is the one place it
//! has to match. Any divergence shows up the moment a real save is
//! reopened, which is the manual check this milestone also ran.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// What `WorldGen::save_project` builds out of a generated world.
fn fields_of(ws: &WorldState, n: usize) -> cartalith_io::SaveFields {
    cartalith_io::SaveFields {
        heightmap: ws.field.clone(),
        temperature: ws.temperature.clone(),
        rainfall: ws.rainfall.clone(),
        volcanic_field: ws.volcanic_field.clone(),
        impact_field: ws.impact_field.clone(),
        strahler_order: match ws.stream_order.as_ref() {
            Some(order) => order.iter().map(|&o| o.clamp(0, 255) as u8).collect(),
            None => vec![0u8; n],
        },
    }
}

/// A world with every stage really run, at a non-square grid — the shape a
/// real export has (`gridH = gw*0.64` in region mode), and the one a
/// row-major bug would survive on a square one.
fn a_real_world() -> (WorldParams, WorldState) {
    let mut p = WorldParams::defaults(24, 15, 24601);
    p.map_width_km = 640.0;
    // Deliberately off their defaults, so the parameter block in the save is
    // carrying real settings rather than something a broken writer could
    // reproduce by emitting defaults.
    p.climate.lat_n = 62.0;
    p.climate.equator_temp = 27.0;
    p.tect.plates = 11;
    p.tect.blur_r = 21.0;
    p.volc.count = 24;
    p.peak_m = 5200.0;
    let ws = generate_terrain(&p);
    (p, ws)
}

#[test]
fn a_generated_world_survives_save_and_reload() {
    let (p, ws) = a_real_world();
    let n = p.gw * p.gh;
    let fields = fields_of(&ws, n);

    let params = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
    };
    let mut buf = Vec::new();
    cartalith_io::write_save(std::io::Cursor::new(&mut buf), &cartalith_io::SaveWrite {
        params: &params,
        state: params::save_state(&p),
        fields: &fields,
    })
    .expect("a real world should save");

    let back = cartalith_io::load_save(std::io::Cursor::new(&buf)).expect("a saved world should reopen");

    // -- the world the renderer reads -------------------------------------
    assert_eq!(back.params.gw, 24);
    assert_eq!(back.params.gh, 15);
    assert_eq!(back.params.seed, 24601);
    assert_eq!(back.params.map_width_km, 640.0);
    assert_eq!(back.params.sea_level, ws.sea_level);
    assert!(!back.params.world);

    // Bit-exact: a `.f32` entry is a byte reinterpretation, not a second
    // computation, so anything short of equality is a bug in the format
    // handling rather than floating-point drift.
    assert_eq!(back.fields.heightmap, ws.field, "heightmap");
    assert_eq!(back.fields.temperature, ws.temperature, "temperature");
    assert_eq!(back.fields.rainfall, ws.rainfall, "rainfall");
    assert_eq!(back.fields.volcanic_field, ws.volcanic_field, "volcanic_field");
    assert_eq!(back.fields.impact_field, ws.impact_field, "impact_field");
    assert_eq!(back.fields.strahler_order, fields.strahler_order, "strahler_order");

    // "Watch for silently-empty golden output" (`CLAUDE.md`): a writer that
    // emitted `gw*gh` zeros would pass every equality above if the generator
    // had also produced zeros. It did not.
    assert!(back.fields.heightmap.iter().any(|&v| v > 0.0), "the heightmap should have real relief");
    assert!(back.fields.temperature.iter().any(|&v| v != 0.0), "temperature should be real");
    assert!(back.fields.rainfall.iter().any(|&v| v > 0.0), "rainfall should be real");
    assert!(back.fields.volcanic_field.iter().any(|&v| v > 0.0), "volcanic_field should be real");
    assert!(back.fields.strahler_order.iter().any(|&v| v > 0), "strahler_order should have real channels");
    assert!(back.fields.heightmap.iter().all(|v| v.is_finite()), "no NaN may reach the file");

    // -- the settings the world was generated from -------------------------
    let mut restored = params::defaults();
    let applied = params::apply_saved_state(&mut restored, &back.state);
    assert_eq!(applied, params::PARAMS.len());
    assert_eq!(restored.climate.lat_n, 62.0);
    assert_eq!(restored.climate.equator_temp, 27.0);
    assert_eq!(restored.tect.plates, 11);
    assert_eq!(restored.tect.blur_r, 21.0);
    assert_eq!(restored.volc.count, 24);
    assert_eq!(restored.peak_m, 5200.0);

    // Regenerating from the restored parameters must reproduce the world
    // bit-for-bit -- the strongest available statement that nothing
    // generation depends on was lost. (`gw`/`gh`/`seed`/`map_width_km` are
    // `generate()` arguments rather than table rows, so they come from
    // `back.params`, which is exactly how the GUI drives a reopen.)
    restored.gw = back.params.gw;
    restored.gh = back.params.gh;
    restored.tect.seed = back.params.seed;
    restored.map_width_km = back.params.map_width_km;
    assert_eq!(restored, p, "the restored parameters must equal the ones the world was generated from");
    assert_eq!(generate_terrain(&restored).field, ws.field, "the saved parameters must regenerate the same world");
}

/// `carve_rivers` off means no `stream_order` at all. The save format has no
/// "absent" for that raster, so it is written as `gw*gh` zeros -- "no
/// channels" -- rather than omitted, which the loader would report as a
/// missing entry.
#[test]
fn a_world_with_no_channels_still_writes_a_full_strahler_raster() {
    let mut p = WorldParams::defaults(12, 9, 7);
    p.carve_rivers = false;
    let ws = generate_terrain(&p);
    assert!(ws.stream_order.is_none(), "the fixture must actually have no stream order");

    let n = p.gw * p.gh;
    let params = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
    };
    let mut buf = Vec::new();
    cartalith_io::write_save(std::io::Cursor::new(&mut buf), &cartalith_io::SaveWrite {
        params: &params,
        state: params::save_state(&p),
        fields: &fields_of(&ws, n),
    })
    .expect("a channel-free world should still save");

    let back = cartalith_io::load_save(std::io::Cursor::new(&buf)).expect("it should reopen");
    assert_eq!(back.fields.strahler_order.len(), n);
    assert!(back.fields.strahler_order.iter().all(|&v| v == 0));
    assert_eq!(back.fields.heightmap, ws.field);
}
