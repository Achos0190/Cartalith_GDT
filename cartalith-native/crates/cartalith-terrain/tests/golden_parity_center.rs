//! Golden-parity test for landmass centering (`PARITY_TESTING.md`) —
//! reference HTML lines 3156-3199 (`bestEmptyColumn`, `shiftGridX`,
//! `featherSeamX`).
//!
//! `center_landmasses_captured.json` was captured under Node's
//! `vm.runInContext` with the DOM/timer-stub harness this port's
//! `CHANGELOG.md` "extraction harness upgrade" entry established, running
//! script tag #1 of the frozen reference with zero-indent `let`/`const`
//! rewritten to `var` so top-level bindings land on the sandbox context.
//! The `real_*` cases come from an actual `generate()` at
//! `gw=48 gh=32 seed=24601 world=true mapWidthKm=4000`; the `tie` and
//! `feather` cases are hand-shaped fixtures aimed at branches real data
//! does not reliably reach.
//!
//! Assertions are exact (`assert_eq!` on `f32`/`usize`), not
//! tolerance-based. `shiftGridX` is a permutation and does no arithmetic
//! at all; `bestEmptyColumn` is a comparison count; `featherSeamX` is a
//! fixed-order `f64` box sum stored back through `f32`, exactly as the
//! reference does it. A tolerance here would only hide a real mismatch.
//!
//! # Fixture shapes
//!
//! - `tie`: an 8×5 grid whose columns 1, 2 and 4 all hold **zero** land
//!   cells. Only a strict `<` (first wins) returns column 1; a `<=` would
//!   return 4. Continuous generated data essentially never ties, so this
//!   branch is unreachable without a fixture built for it. The same grid
//!   is re-tested at a sea level where subtracting the geoid genuinely
//!   flips the answer, so the `geo?geo[i]:0` term is measured rather than
//!   assumed.
//! - `shift`: `off` of 3, 0, -3, 11 and 8 over a width of 8 — in range,
//!   the early-return no-op, negative, wider than the grid, and exactly
//!   the grid width (which reduces to the no-op). Also run over a
//!   `Uint8Array`, because the reference allocates its row buffer as `new
//!   arr.constructor(W)` and shifts `plateId`/`boundaryMask`/`riverMask`
//!   through the same function.
//! - `feather`: `col` 0 (every neighbour read wraps in both directions),
//!   an interior column, `halfW` 1 and `halfW` 3 — the band width is a
//!   parameter and one value would not prove it is read.
//! - `real_shift_feather`: the exact composition `centerLandmasses` runs
//!   — `bestEmptyColumn`, then `shiftGridX`, then `featherSeamX` at
//!   `(GW-off)%GW` — over the real 48×32 height field.

use cartalith_terrain::center::{best_empty_column, feather_seam_x, seam_column, shift_grid_x};

fn fixture() -> serde_json::Value {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/center_landmasses_captured.json"
    ))
    .expect("center_landmasses_captured.json fixture should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn u8s(v: &serde_json::Value) -> Vec<u8> {
    v.as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect()
}

#[test]
fn best_empty_column_matches_the_reference_on_a_real_world() {
    let g = fixture();
    let field = f32s(&g["field"]);
    let (gw, gh) = (g["gw"].as_u64().unwrap() as usize, g["gh"].as_u64().unwrap() as usize);
    let sea = g["sea"].as_f64().unwrap();
    assert_eq!(field.len(), gw * gh, "the fixture must be a whole grid");

    let expected = g["real_best"].as_u64().unwrap() as usize;
    assert_eq!(best_empty_column(&field, None, gw, gh, sea), expected);

    // Not a tautology: a world whose emptiest meridian is already at the
    // edge returns 0 and `centerLandmasses` early-outs, so a fixture that
    // happened to land there would test nothing.
    assert_ne!(expected, 0, "the fixture must be a world that actually needs centering");
}

#[test]
fn best_empty_column_ties_go_to_the_lowest_column_and_the_geoid_is_read() {
    let g = fixture();
    let t = &g["tie"];
    let field = f32s(&t["field"]);
    let geo = f32s(&t["geo"]);
    let (w, h) = (t["w"].as_u64().unwrap() as usize, t["h"].as_u64().unwrap() as usize);

    assert_eq!(best_empty_column(&field, None, w, h, 0.5), t["best_no_geo"].as_u64().unwrap() as usize);
    assert_eq!(best_empty_column(&field, Some(&geo), w, h, 0.5), t["best_with_geo"].as_u64().unwrap() as usize);
    assert_eq!(best_empty_column(&field, None, w, h, 0.87), t["best_no_geo_high"].as_u64().unwrap() as usize);
    assert_eq!(best_empty_column(&field, Some(&geo), w, h, 0.87), t["best_geo_flips"].as_u64().unwrap() as usize);

    // The fixture is only worth anything if the geoid actually changes an
    // answer somewhere; otherwise `geo?geo[i]:0` could be dropped and this
    // suite would still pass.
    assert_ne!(t["best_geo_flips"], t["best_no_geo_high"], "the geoid must change an answer in this fixture");
}

#[test]
fn shift_grid_x_matches_the_reference_for_every_offset_class() {
    let g = fixture();
    let src = f32s(&g["tie"]["field"]);
    let (w, h) = (8usize, 5usize);
    for (key, off) in [("off3", 3isize), ("off0", 0), ("off-3", -3), ("off11", 11), ("off8", 8)] {
        let mut a = src.clone();
        shift_grid_x(&mut a, w, h, off);
        assert_eq!(a, f32s(&g["shift"][key]), "offset {off}");
    }
}

#[test]
fn shift_grid_x_shifts_an_integer_grid_the_same_way() {
    let g = fixture();
    let mut a = u8s(&g["shift"]["u8_in"]);
    shift_grid_x(&mut a, 8, 5, 3);
    assert_eq!(a, u8s(&g["shift"]["u8_off3"]));
}

#[test]
fn feather_seam_x_matches_the_reference_including_the_wrap() {
    let g = fixture();
    let src = f32s(&g["tie"]["field"]);
    for (key, col, hw) in [("c0_h2", 0usize, 2usize), ("c4_h2", 4, 2), ("c6_h1", 6, 1), ("c2_h3", 2, 3)] {
        let mut a = src.clone();
        feather_seam_x(&mut a, 8, 5, col, hw);
        assert_eq!(a, f32s(&g["feather"][key]), "col {col} half-width {hw}");
    }
}

#[test]
fn the_whole_centering_composition_matches_the_reference_on_a_real_world() {
    let g = fixture();
    let field = f32s(&g["field"]);
    let (gw, gh) = (g["gw"].as_u64().unwrap() as usize, g["gh"].as_u64().unwrap() as usize);
    let sea = g["sea"].as_f64().unwrap();

    let off = best_empty_column(&field, None, gw, gh, sea);
    let mut shifted = field.clone();
    shift_grid_x(&mut shifted, gw, gh, off as isize);
    let sc = seam_column(gw, off);
    assert_eq!(sc, g["real_seam_col"].as_u64().unwrap() as usize);
    feather_seam_x(&mut shifted, gw, gh, sc, 2);

    assert_eq!(shifted, f32s(&g["real_shift_feather"]));
    // The pass must have done something, and the feather must have done
    // something beyond the shift -- a silently-empty golden is exactly the
    // failure mode this project's own working rules call out.
    assert_ne!(shifted, field, "centering must change the field");
    let mut shift_only = field.clone();
    shift_grid_x(&mut shift_only, gw, gh, off as isize);
    assert_ne!(shifted, shift_only, "the feather must change something the shift did not");
}
