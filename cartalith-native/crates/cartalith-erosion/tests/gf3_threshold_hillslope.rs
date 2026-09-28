//! GF-3 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ): the
//! rock-aware threshold hillslope, `threshold_hillslope` and
//! `critical_talus` (§4.3).
//!
//! **What these tests protect.**
//! - The critical-angle scaling `talus = tan(θc)·cell_m·(1 − sea)/peak_m`, at
//!   two extents, against **literal** values computed by hand (GF-3's
//!   done-means: "tested at two extents with a literal expected talus ... never
//!   asserted against its own constant").
//! - That the threshold is each cell's **own exposed rock**: the same 51° step
//!   is held by granite (θc 60°) and shed by shale (θc 30°), and by a two-layer
//!   cell exactly when its surface lies below the contact (the substrate is
//!   exposed), and by any cell whose regolith reads unconsolidated (33°).
//! - The move rule is `erode_thermal`'s, unchanged: one pass sheds `0.125` of
//!   the excess, mass-conserving, to literal heights.
//! - The same physical angle behaves the same at 80 km and at 800 km (the
//!   extent-blindness §1.3 notes is gone for this stage).
//! - World maps wrap in x; the column is never written.
//!
//! The legacy scalar kernel stays pinned by `golden_parity_thermal.rs`, which
//! now runs through the same shared body with `per_cell = None`.

use cartalith_erosion::{critical_talus, threshold_hillslope, ThresholdHillslope};
use cartalith_terrain::geology::{GeologyColumn, Rock, NO_LAYER};

/// The fixtures' world: 800 km over 2048 cells (390.625 m cells, the app's
/// default), sea 0.42, peak 4000 m.
const CELL_800: f64 = 800_000.0 / 2048.0;
// The "80 km" extent the extent-blindness test compares against 800 km, same 2048-cell grid.
const CELL_80: f64 = 80_000.0 / 2048.0;
// Sea level, normalised -- the fixtures' shared world (see the doc comment above).
const SEA: f64 = 0.42;
// Peak altitude in metres -- the fixtures' shared world (see the doc comment above).
const PEAK: f64 = 4000.0;

/// A uniform `n`-cell column: every cell's exposed rock is `top`, no
/// substrate layer, no regolith. The per-test fixtures then override
/// `rock_sub`/`contact`/`regolith` on top of this baseline.
fn column(n: usize, top: Rock) -> GeologyColumn {
    GeologyColumn {
        rock_top: vec![top as u8; n],
        rock_sub: vec![NO_LAYER; n],
        contact: vec![f32::NAN; n],
        regolith: vec![0.0; n],
        volcanic_setting: vec![0; n],
    }
}

/// Builds the `ThresholdHillslope` context for a fixture column and cell
/// size, at this file's shared `SEA`/`PEAK`.
fn rock<'a>(col: &'a GeologyColumn, cell_m: f64) -> ThresholdHillslope<'a> {
    // R_EXPOSE: any positive threshold works here; the regolith test sets
    // regolith well above it.
    ThresholdHillslope { column: col, r_expose: 0.001, cell_m, sea: SEA, peak_m: PEAK }
}

/// §4.3's formula at two extents and three angles, against literals worked by
/// hand: `tan(45°) = 1`, so 390.625 × 0.58 / 4000 = 0.056640625 exactly, and a
/// tenth of that at 80 km; `tan(60°)` and `tan(30°)` to 16 digits.
#[test]
fn critical_talus_is_the_angle_scaled_by_cell_size_and_peak() {
    // Protects: critical_talus's §4.3 formula against hand-computed
    // literals at two map extents (800 km, 80 km) and three critical angles.
    let close = |got: f64, want: f64| assert!((got - want).abs() < 1e-12, "got {got}, want {want}");
    close(critical_talus(45.0, CELL_800, SEA, PEAK), 0.056640625);
    close(critical_talus(45.0, CELL_80, SEA, PEAK), 0.0056640625);
    close(critical_talus(60.0, CELL_800, SEA, PEAK), 0.09810444027245592);
    close(critical_talus(30.0, CELL_800, SEA, PEAK), 0.03270148009081865);
    close(critical_talus(30.0, CELL_80, SEA, PEAK), 0.0032701480090818647);
}

/// A zero or negative anchor would make every threshold 0 or negative: a
/// plausible-looking number that collapses every slope. It must refuse.
#[test]
#[should_panic(expected = "positive peak_m")]
fn critical_talus_refuses_a_non_positive_peak() {
    // Protects: critical_talus panics on a non-positive peak_m rather than
    // silently returning a plausible-looking threshold that would collapse
    // every slope (MISTAKES.md: never encode "no value" as a plausible one).
    critical_talus(45.0, CELL_800, SEA, 0.0);
}

/// One pass over a 0.07 step (51° at 390.625 m cells) on a two-cell row.
fn one_pass(col: &GeologyColumn, cell_m: f64, step: f32) -> Vec<f32> {
    let mut f = vec![0.5 + step, 0.5];
    threshold_hillslope(&mut f, 2, 1, 1, false, &rock(col, cell_m));
    f
}

/// Strong rock holds the 51° step exactly; shale (30°) sheds 0.125 of the
/// excess to literal heights, and the pair's mass is conserved.
#[test]
fn granite_holds_a_step_that_shale_sheds() {
    // Protects: threshold_hillslope's per-cell rock threshold -- granite's
    // 60 degree critical angle holds a 51 degree step exactly, shale's 30
    // degree angle sheds 0.125 of the excess to a hand-computed height, and
    // the pair's combined mass is conserved by the move.
    let granite = column(2, Rock::Granite);
    let f = one_pass(&granite, CELL_800, 0.07);
    assert_eq!(f, vec![0.57f32, 0.5], "granite (θc 60°) must not move a 51° step");

    let shale = column(2, Rock::Shale);
    let f = one_pass(&shale, CELL_800, 0.07);
    // a − 0.125·(0.07 − tan30°·390.625·0.58/4000), computed by hand.
    assert!((f[0] as f64 - 0.5653376787528647).abs() < 1e-6, "upper {}", f[0]);
    assert!((f[1] as f64 - 0.504662314094578).abs() < 1e-6, "lower {}", f[1]);
    assert!(((f[0] + f[1]) as f64 - 1.07).abs() < 1e-6, "mass must be conserved");
}

/// The threshold is the **shedding** cell's: a granite upper cell over a shale
/// lower cell holds; the reverse would never shed uphill. Swapping which cell
/// is shale moves the result.
#[test]
fn the_threshold_is_the_upper_cells_rock() {
    // Protects: the shedding threshold is read from the UPPER (shedding)
    // cell's own rock, not the lower cell's or some grid-wide value --
    // swapping which of the two cells is shale changes the result, and it
    // changes correctly regardless of which grid index is the upper cell.
    let mut col = column(2, Rock::Granite);
    col.rock_top[1] = Rock::Shale as u8;
    assert_eq!(one_pass(&col, CELL_800, 0.07), vec![0.57f32, 0.5], "granite on top holds");
    col.rock_top = vec![Rock::Shale as u8, Rock::Granite as u8];
    assert_ne!(one_pass(&col, CELL_800, 0.07), vec![0.57f32, 0.5], "shale on top sheds");
    // Mirrored, so the shedding cell is not cell 0: shale upper at index 1
    // sheds onto granite at index 0; granite upper at index 1 holds.
    let mirrored = |top: [Rock; 2]| {
        let mut c = column(2, Rock::Granite);
        c.rock_top = vec![top[0] as u8, top[1] as u8];
        let mut f = vec![0.5f32, 0.57];
        threshold_hillslope(&mut f, 2, 1, 1, false, &rock(&c, CELL_800));
        f
    };
    assert_eq!(mirrored([Rock::Shale, Rock::Granite]), vec![0.5f32, 0.57], "granite upper at index 1 holds");
    let f = mirrored([Rock::Granite, Rock::Shale]);
    assert!((f[1] as f64 - 0.5653376787528647).abs() < 1e-6, "shale upper at index 1 sheds, got {}", f[1]);
}

/// A granite cap over a shale substrate (§2.5): while the surface is above
/// the contact the cap holds; once it is below, the substrate is exposed and
/// the same step sheds. This is the contact switch applied per pass.
#[test]
fn a_breached_cap_sheds_at_the_substrates_angle() {
    // Protects: §2.5's contact switch inside threshold_hillslope -- while
    // the surface is above the contact the cap's own angle holds the step,
    // and once the surface drops below the contact the substrate's (lower)
    // angle takes over and the same step sheds.
    let mut col = column(2, Rock::Granite);
    col.rock_sub = vec![Rock::Shale as u8; 2];
    col.contact = vec![0.40; 2];
    assert_eq!(one_pass(&col, CELL_800, 0.07), vec![0.57f32, 0.5], "cap intact (surface above contact): holds");
    col.contact = vec![0.60; 2];
    let f = one_pass(&col, CELL_800, 0.07);
    assert!((f[0] as f64 - 0.5653376787528647).abs() < 1e-6, "substrate exposed: sheds at shale's 30°, got {}", f[0]);
}

/// The exposed rock is re-read **every pass**: a granite cap standing a
/// hair above its contact over a 0.12 step (steeper than granite's 60°,
/// 0.0981) sheds as granite in the first pass, drops below the contact, and
/// from then on sheds at shale's 30° (0.0327). After 16 passes the step is
/// therefore far below granite's threshold; read once, it would stop at it.
#[test]
fn a_cap_breached_mid_stage_sheds_at_the_substrates_angle_from_the_next_pass() {
    // Protects: the exposed rock is re-read every pass, not once at the
    // start of the multi-pass call -- a cap that breaches partway through
    // must shed at the substrate's (shallower) angle from the very next
    // pass onward, ending far below the cap's own threshold after 16
    // passes, which a read-once implementation would not reach.
    let mut col = column(2, Rock::Granite);
    col.rock_sub = vec![Rock::Shale as u8; 2];
    col.contact = vec![0.619, f32::NAN];
    col.rock_sub[1] = NO_LAYER;
    let mut f = vec![0.62f32, 0.5];
    threshold_hillslope(&mut f, 2, 1, 16, false, &rock(&col, CELL_800));
    let step = (f[0] - f[1]) as f64;
    assert!(f[0] < 0.619, "the first pass must lower the cap through its contact: {}", f[0]);
    assert!(step < 0.06, "after the breach the step relaxes toward shale's 0.0327, not granite's 0.0981; got {step}");
}

/// Regolith thicker than `R_EXPOSE` reads unconsolidated (33°), whatever the
/// bedrock, so a talus apron on granite sheds.
#[test]
fn thick_regolith_sheds_at_the_angle_of_repose() {
    // Protects: regolith thicker than r_expose reads as unconsolidated
    // material (33 degree angle of repose) regardless of the bedrock
    // underneath -- a granite bedrock cell with a talus apron sheds at
    // shale's steeper-than-granite, shallower-than-bare-granite angle.
    let mut col = column(2, Rock::Granite);
    col.regolith = vec![0.01; 2];
    let f = one_pass(&col, CELL_800, 0.07);
    assert!((f[0] as f64 - 0.5658478502363189).abs() < 1e-6, "unconsolidated (33°), got {}", f[0]);
    assert!((f[1] as f64 - 0.5041521426111237).abs() < 1e-6, "lower {}", f[1]);
}

/// The same physical angle at two extents: a step of 51° (0.07 at 800 km,
/// 0.007 at 80 km) is held by granite and shed by shale at both. The legacy
/// scalar `talus = 0.012` would shed the first and hold the second.
#[test]
fn the_same_angle_behaves_the_same_at_80_and_800_km() {
    // Protects: the same physical 51 degree step is held by granite and shed
    // by shale at BOTH 800 km and 80 km map extents -- the extent-blindness
    // the legacy scalar talus (a single fixed constant) had is gone, since
    // that constant would shed the 800 km step and hold the 80 km one.
    let granite = column(2, Rock::Granite);
    let shale = column(2, Rock::Shale);
    for (cell, step) in [(CELL_800, 0.07f32), (CELL_80, 0.007)] {
        let held = one_pass(&granite, cell, step);
        assert_eq!(held, vec![0.5 + step, 0.5], "granite holds at cell {cell} m");
        let shed = one_pass(&shale, cell, step);
        assert!(shed[0] < 0.5 + step, "shale sheds at cell {cell} m");
    }
}

/// On a world map the row wraps: a step between the last and the first cell
/// sheds only when `wrap` is on.
#[test]
fn world_maps_wrap_in_x() {
    // Protects: threshold_hillslope's wrap flag -- with wrap off the row's
    // two ends do not interact; with wrap on, the seam neighbour (cell 0 to
    // cell 2 and back) is added to the shedding, checked at both ends of the
    // row so neither edge is a special case the other is not.
    let col = column(3, Rock::Shale);
    let base = vec![0.57f32, 0.535, 0.5];
    let mut open = base.clone();
    threshold_hillslope(&mut open, 3, 1, 1, false, &rock(&col, CELL_800));
    let mut wrapped = base.clone();
    threshold_hillslope(&mut wrapped, 3, 1, 1, true, &rock(&col, CELL_800));
    // Each 0.035 step is just over shale's 0.0327, so the open row sheds a
    // little along x; with wrap, cell 0's left neighbour is cell 2, a 0.07
    // drop, which sheds far more: cell 0 loses more and cell 2 gains more.
    assert!(wrapped[2] > open[2], "wrap must add the 0 -> 2 neighbour: open {open:?} wrapped {wrapped:?}");
    assert!(wrapped[0] < open[0]);
    // The other edge: the high cell is the last one, whose right neighbour
    // wraps to cell 0.
    let base = vec![0.5f32, 0.535, 0.57];
    let mut open = base.clone();
    threshold_hillslope(&mut open, 3, 1, 1, false, &rock(&col, CELL_800));
    let mut wrapped = base.clone();
    threshold_hillslope(&mut wrapped, 3, 1, 1, true, &rock(&col, CELL_800));
    assert!(wrapped[0] > open[0], "wrap must add the 2 -> 0 neighbour: open {open:?} wrapped {wrapped:?}");
    assert!(wrapped[2] < open[2]);
}

/// The stage reads the column and never writes it (§4.3; the caller accounts
/// regolith).
#[test]
fn the_column_is_never_written() {
    // Protects: threshold_hillslope only reads the GeologyColumn (§4.3 --
    // the caller accounts regolith) -- rock_top/rock_sub/contact/regolith
    // are all byte-identical after an 8-pass run over a cap-and-substrate
    // fixture that exercises every field the stage reads.
    let mut col = column(2, Rock::Granite);
    col.rock_sub = vec![Rock::Shale as u8; 2];
    col.contact = vec![0.60; 2];
    col.regolith = vec![0.0005; 2];
    let before = (col.rock_top.clone(), col.rock_sub.clone(), col.contact.clone(), col.regolith.clone());
    let mut f = vec![0.57f32, 0.5];
    threshold_hillslope(&mut f, 2, 1, 8, false, &rock(&col, CELL_800));
    assert_eq!((col.rock_top.clone(), col.rock_sub.clone(), col.contact.clone(), col.regolith.clone()), before);
}

/// Passes relax a lone over-steep pair geometrically: each pass removes a
/// quarter of the excess (an eighth leaves the upper cell and lands on the
/// lower), so after `n` passes the excess is `0.75^n` of what it was.
#[test]
fn each_pass_removes_a_quarter_of_a_lone_steps_excess() {
    // Protects: the per-pass relaxation rate itself -- a lone over-steep
    // pair's excess-over-threshold shrinks geometrically by exactly 0.75
    // per pass (an eighth of the excess moves each direction), checked at
    // 1, 4 and 16 passes against the closed-form 0.75^n prediction.
    let shale = column(2, Rock::Shale);
    let t = critical_talus(30.0, CELL_800, SEA, PEAK);
    for n in [1, 4, 16] {
        let mut f = vec![0.6f32, 0.5];
        threshold_hillslope(&mut f, 2, 1, n, false, &rock(&shale, CELL_800));
        let excess = (f[0] - f[1]) as f64 - t;
        let want = (0.1 - t) * 0.75f64.powi(n);
        assert!((excess - want).abs() < 1e-6, "{n} passes: excess {excess}, want {want}");
    }
}
