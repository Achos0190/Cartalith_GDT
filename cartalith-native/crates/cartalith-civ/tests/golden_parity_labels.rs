//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's Label tool:
//! `drawArcLabel` (reference line 15244), `_civLabelBox` (15280),
//! `_civLabelHitTest` (15296), `_civSelectLabel`/`_civConfirmLabel`/
//! `_civCancelLabel` (15356-15367) and the three handle formulas inline in the
//! pointer-move handler (9686-9717).
//!
//! # The harness, and the one place it stubs the reference's environment
//!
//! Node `vm.runInContext` over **whole `<script>` blocks** (#1 2084-14556,
//! #2 14563-26720), delimiters asserted against the real `<script>`/
//! `</script>` tags — milestone D's technique, with the balance check that
//! fired twice (wrongly) and was fixed rather than deleted; see
//! `cartalith-terrain/tests/golden_parity_amplify.rs` for that write-up.
//!
//! `drawArcLabel` and `_civLabelBox` both take a **Canvas 2D context**, so the
//! harness supplies one: a stub that records `translate`/`rotate`/`strokeText`
//! and answers `measureText` from a fixed formula. Disclosed plainly, because
//! it is a modification of the reference's environment — but note what it is
//! not. No function body is transcribed or edited; the layout arithmetic that
//! is under test runs entirely inside the real `drawArcLabel`, and the stub
//! only supplies the two things a font provides (glyph advances) and receives
//! the two things a transform is (translate, rotate).
//!
//! The stub's `measureText` deliberately makes `measureText(text).width` **not
//! equal** the sum of the per-`char` widths (a 3%-per-gap "kerning" term).
//! Real fonts kern; `drawArcLabel` reads both numbers; and a port that summed
//! the char widths instead of taking the measured total would have passed a
//! stub where the two agreed. That is urban M3's lesson applied to a fixture's
//! *shape* rather than its values.
//!
//! # Shape assertions before any golden was written down
//!
//! The extraction asserted: at least one case takes the straight branch and at
//! least one takes the arc branch (119 arc glyphs across 11 cases, 2 straight);
//! every label box is non-degenerate; the hit test produces hits, misses **and**
//! a topmost-wins overlap; all five armed handle kinds are reachable; and
//! cancel reverts the name while leaving the position alone. Every one is
//! re-asserted here.
//!
//! # Transcription, disclosed
//!
//! The three handle formulas are **transcribed**, not sliced: they are inline
//! in a `pointermove` listener, not callable functions, so there is nothing to
//! call. Lines 9686-9689, 9698-9702 and 9711-9716 were copied verbatim into
//! the harness with the DOM reads replaced by parameters. Weaker evidence than
//! a slice, stated rather than implied — the same disclosure milestone C made
//! about `sculptCommit`'s body.

use cartalith_civ::labels::*;

/// The harness's stub metrics, reproduced exactly. `size_px` here is the
/// **truncated** size, because the reference builds its font string as
/// `${sizePx|0}px` and `measureText` reads the font.
fn char_width(ch: char, font_px: f64) -> f64 {
    font_px * (0.4 + 0.35 * ((ch as u32 % 7) as f64 / 6.0))
}

fn measured(text: &str, size_px: f64) -> (Vec<f64>, f64) {
    let font_px = (size_px as i64) as f64;
    let per: Vec<f64> = text.chars().map(|c| char_width(c, font_px)).collect();
    let mut total: f64 = per.iter().sum();
    let n = per.len();
    if n > 1 {
        total += font_px * 0.03 * (n as f64 - 1.0);
    }
    (per, total)
}

fn fnv_f64(vals: &[f64]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in vals {
        for &b in &v.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

fn layout_hash(text: &str, arc: f64, size_px: f64) -> (String, usize, f64) {
    let (per, total) = measured(text, size_px);
    match arc_label_layout(&per, total, arc, size_px) {
        ArcLayout::Straight => (fnv_f64(&[]), 0, total),
        ArcLayout::Arc(g) => {
            let mut flat = Vec::with_capacity(g.len() * 3);
            for p in &g {
                flat.push(p.dx);
                flat.push(p.dy);
                flat.push(p.rot);
            }
            (fnv_f64(&flat), g.len(), total)
        }
    }
}

const TEXT: &str = "Kingdom of Aldar";

#[test]
fn the_stub_metrics_are_reproduced_exactly() {
    // Every layout golden below is only evidence about the layout if the
    // widths going in match the harness's. Spot-checked against the recorded
    // per-char array for the 24px case.
    let (per, total) = measured(TEXT, 24.0);
    assert_eq!(total, 248.400_000_000_000_03);
    let want: [f64; 16] = [
        16.6,
        9.600_000_000_000_001,
        16.6,
        16.6,
        12.400_000_000_000_002,
        18.0,
        15.2,
        15.2,
        18.0,
        15.2,
        15.2,
        12.400_000_000_000_002,
        13.799_999_999_999_999,
        12.400_000_000_000_002,
        18.0,
        12.400_000_000_000_002,
    ];
    assert_eq!(per.len(), want.len());
    for (i, (a, b)) in per.iter().zip(want.iter()).enumerate() {
        assert_eq!(a.to_bits(), b.to_bits(), "char {i}");
    }
}

#[test]
fn case0_a_flat_label_takes_the_straight_branch() {
    assert_eq!(arc_label_layout(&measured(TEXT, 24.0).0, 248.400_000_000_000_03, 0.0, 24.0),
               ArcLayout::Straight);
}

#[test]
fn case1_an_arc_just_below_the_threshold_is_still_straight() {
    let (per, total) = measured(TEXT, 24.0);
    assert_eq!(arc_label_layout(&per, total, 0.009, 24.0), ArcLayout::Straight);
}

#[test]
fn case2_the_threshold_itself_takes_the_arc_branch() {
    let (h, n, _) = layout_hash(TEXT, 0.01, 24.0);
    assert_eq!(n, 16);
    assert_eq!(h, "94cbe6de7105da60");
}

#[test]
fn case3_a_half_dome_matches_the_reference_glyph_for_glyph() {
    let (per, total) = measured(TEXT, 24.0);
    let ArcLayout::Arc(g) = arc_label_layout(&per, total, 0.5, 24.0) else { panic!("expected arc") };
    assert_eq!(g.len(), 16);
    // The first two glyphs spelled out, so a regression names itself rather
    // than only moving the hash.
    assert_eq!(g[0].dx.to_bits(), (-110.878_200_549_594_14f64).to_bits());
    assert_eq!(g[0].dy.to_bits(), 29.095_341_551_594_057f64.to_bits());
    assert_eq!(g[0].rot.to_bits(), (-0.513_244_766_505_636_1f64).to_bits());
    assert_eq!(g[1].dx.to_bits(), (-99.285_940_829_807_02f64).to_bits());
    assert_eq!(g[1].dy.to_bits(), 22.997_686_815_311_148f64.to_bits());
    assert_eq!(g[1].rot.to_bits(), (-0.455_233_494_363_929_24f64).to_bits());
    let (h, _, _) = layout_hash(TEXT, 0.5, 24.0);
    assert_eq!(h, "0126b945ee51b93e");
}

#[test]
fn case4_a_negative_arc_at_a_smaller_size() {
    let (h, n, total) = layout_hash(TEXT, -0.8, 18.0);
    assert_eq!(n, 16);
    assert_eq!(total, 186.300_000_000_000_04);
    assert_eq!(h, "8dac858619f45187");
}

#[test]
fn case5_and_6_arc_is_clamped_to_the_unit_range() {
    let (h5, _, _) = layout_hash(TEXT, 1.5, 24.0);
    assert_eq!(h5, "0d17bdea1675a959");
    let (h6, _, _) = layout_hash(TEXT, -3.0, 24.0);
    assert_eq!(h6, "e948f82766648559");
    // and they really are the clamped endpoints, not some other value
    let (h1, _, _) = layout_hash(TEXT, 1.0, 24.0);
    let (hm1, _, _) = layout_hash(TEXT, -1.0, 24.0);
    assert_eq!(h5, h1);
    assert_eq!(h6, hm1);
    assert_ne!(h5, h6);
}

#[test]
fn case7_a_single_glyph_hits_the_radius_floor() {
    let (h, n, _) = layout_hash("X", 0.7, 40.0);
    assert_eq!(n, 1);
    assert_eq!(h, "81d23fd7003c2305");
}

#[test]
fn case8_empty_text_on_a_real_arc_produces_no_glyphs_and_does_not_throw() {
    let (per, total) = measured("", 16.0);
    assert_eq!(total, 0.0);
    let ArcLayout::Arc(g) = arc_label_layout(&per, total, 0.7, 16.0) else { panic!("expected arc") };
    assert!(g.is_empty());
}

/// The reference's own 36-glyph layout for case 9, dumped value by value.
///
/// Spelled out rather than hashed because this is the **one** case in the
/// whole milestone that is not bit-exact, and the test below pins exactly
/// which two values diverge and by exactly how much. See
/// [`case9_a_long_name_at_a_shallow_arc`].
const CASE9: [(f64, f64, f64); 36] = [
    (-137.31262636533583, 18.397117841262432, -0.2663733008016731),
    (-129.62735788683614, 16.36292879632904, -0.25113279888462886),
    (-121.9119810689574, 14.446098805096618, -0.23589229696758454),
    (-113.82674802622843, 12.570584682855417, -0.21998082955733708),
    (-105.71269752788113, 10.823950465787554, -0.20406936214708962),
    (-97.57188380364771, 9.206638348323718, -0.18815789473684214),
    (-90.0959287085503, 7.839497649510417, -0.1735883583130011),
    (-81.9096068087364, 6.471038541392996, -0.15767689090275364),
    (-73.70254789851097, 5.233003557213615, -0.14176542349250618),
    (-65.47682975449513, 4.125706129612899, -0.1258539560822587),
    (-57.93025682194173, 3.2266982053336224, -0.11128441965841764),
    (-51.06807033977412, 2.505794261432237, -0.09805681422098299),
    (-43.84819725246733, 1.8461834044961125, -0.08415824329034512),
    (-36.619854133234526, 1.2869789427433798, -0.07025967235970729),
    (-28.336045699143188, 0.7701961980237916, -0.0543482049494598),
    (-19.695317468152528, 0.3719486437446822, -0.03776577204600912),
    (-12.448818023840925, 0.1485662742605793, -0.023867201115371262),
    (-5.54989528967122, 0.02952459948687366, -0.010639595677936618),
    (2.749987261728207, 0.0072488068432762735, 0.005271871732310855),
    (11.749006390776898, 0.13233036663131179, 0.02252527012896474),
    (20.394800395348895, 0.3988477334861638, 0.03910770303241542),
    (28.68552251948022, 0.7893259510630103, 0.0550191704426629),
    (36.61985413323446, 1.2869789427433798, 0.07025967235970716),
    (44.19694863082784, 1.8757210329701457, 0.08482920878354822),
    (52.112922272127, 2.6096406248910773, 0.10006971070059249),
    (58.625874447064334, 3.3049033591722585, 0.11262635064482394),
    (64.4350024293868, 3.9949565506345537, 0.12384105960264898),
    (71.62302325481012, 4.940477831561099, 0.13773963053328683),
    (79.14317483044857, 6.038794067299786, 0.15230916695712787),
    (86.30136958173512, 7.188533943123156, 0.16620773788776572),
    (92.06520433923245, 8.188709032952321, 0.17742244684559078),
    (98.5049544209069, 9.385185187447194, 0.1899790867898222),
    (106.30017747295774, 10.94588189915104, 0.20521958870686643),
    (113.38754999108328, 12.472579206649698, 0.2191181596375043),
    (119.77182955436282, 13.936447629900954, 0.23167479958173573),
    (126.13722503641465, 15.480363644383432, 0.2442314395259672),
];

/// Distance between two doubles in units in the last place.
fn ulps_apart(a: f64, b: f64) -> i64 {
    if a == b {
        return 0;
    }
    let (ia, ib) = (a.to_bits() as i64, b.to_bits() as i64);
    let key = |i: i64| if i < 0 { i64::MIN - i } else { i };
    (key(ia) - key(ib)).abs()
}

#[test]
fn case9_a_long_name_at_a_shallow_arc() {
    // # The one non-bit-exact result in this milestone, measured rather than
    // # assumed
    //
    // 106 of this case's 108 values are bit-identical to the reference. Two
    // are one ULP away, both `dx` (glyphs 28 and 34) and both from
    // `r * sin(theta)`: `dy` and `rot` agree exactly at those same glyphs, so
    // `theta` itself is bit-identical and the divergence is purely V8's
    // `Math.sin` against Rust's. Every other arc case in this file is exact.
    //
    // This is the project's second such allowance after `CHANGELOG.md`'s
    // `1e-4` for `Math.pow`/`Math.exp`, and it is very much tighter: one ULP
    // is ~1.4e-16 relative, i.e. a sub-picometre glyph offset. It is safe here
    // in a way it would not have been in milestone D, where an ULP could flip
    // a `dist < best` segment pick and with it the *sign* of a signed
    // distance. Nothing branches on a glyph position; it goes straight into a
    // canvas transform.
    //
    // The test pins the exact divergence so it cannot quietly grow: every
    // value within 1 ULP, and *exactly two* values not bit-identical, at
    // exactly those two indices.
    let (per, total) = measured("Sea of Storms and Long Quiet Winters", 12.0);
    assert_eq!(total, 286.900_000_000_000_03);
    let ArcLayout::Arc(g) = arc_label_layout(&per, total, 0.25, 12.0) else { panic!("expected arc") };
    assert_eq!(g.len(), 36);

    let mut inexact: Vec<(usize, &str)> = Vec::new();
    for (i, (dx, dy, rot)) in CASE9.into_iter().enumerate() {
        for (name, got, want) in
            [("dx", g[i].dx, dx), ("dy", g[i].dy, dy), ("rot", g[i].rot, rot)]
        {
            let u = ulps_apart(got, want);
            assert!(u <= 1, "glyph {i} {name}: {got} vs {want} ({u} ulps)");
            if u != 0 {
                inexact.push((i, name));
            }
        }
    }
    assert_eq!(inexact, vec![(28, "dx"), (34, "dx")], "the known Math.sin divergence moved");
}

#[test]
fn case10_two_glyphs_at_the_minimum_size_and_a_full_bow() {
    let (h, n, total) = layout_hash("ab", 1.0, 9.0);
    assert_eq!(n, 2);
    assert_eq!(total, 10.62);
    assert_eq!(h, "58a64011adce33d5");
}

#[test]
fn the_halo_stroke_width_matches_the_reference_at_every_tested_size() {
    assert_eq!(arc_label_line_width(24.0), 3.84);
    assert_eq!(arc_label_line_width(18.0), 2.88);
    assert_eq!(arc_label_line_width(40.0), 6.4);
    assert_eq!(arc_label_line_width(16.0), 2.56);
    assert_eq!(arc_label_line_width(12.0), 1.92);
    assert_eq!(arc_label_line_width(9.0), 1.44);
}

// ---------------------------------------------------------------------------
// _civLabelBox
// ---------------------------------------------------------------------------

fn fixture_labels() -> Vec<MapLabel> {
    let mut a = MapLabel::new(10.0, 8.0, "Aldar");
    a.size = 16.0;
    let mut b = MapLabel::new(30.0, 20.0, "The Long Quiet Sea");
    b.angle = 12.0;
    b.arc = 0.4;
    b.size = 28.0;
    b.font = Some("Times, serif".into());
    b.color = Some("#abc".into());
    b.size_mode = LabelSizeMode::Fixed;
    let mut c = MapLabel::new(3.0, 3.0, "i");
    c.size = 8.0;
    let mut d = MapLabel::new(40.0, 26.0, "");
    d.size = 48.0;
    vec![a, b, c, d]
}

fn boxed(lb: &MapLabel, env: &LabelViewEnv) -> LabelBox {
    let fsz = label_font_size(lb, env);
    let (_, w) = measured(&lb.name, fsz);
    label_box(lb, env, w)
}

#[test]
fn label_box_matches_the_reference_across_every_zoom_and_scale() {
    let labels = fixture_labels();
    // (grid_w, zoom_scale, icon_scale, label index, side, fsz)
    // (grid_w, zoom_scale, icon_scale, label index, px, py, side, fsz)
    type BoxCase = (usize, f64, f64, usize, f64, f64, f64, f64);
    let want: &[BoxCase] = &[
        (48, 1.0, 1.0, 0, 10.5, 8.5, 59.900_000_000_000_006, 16.0),
        (48, 1.0, 1.0, 1, 30.5, 20.5, 420.933_333_333_333_34, 28.0),
        (48, 1.0, 1.0, 2, 3.5, 3.5, 14.625_000_000_000_002, 9.0),
        (48, 1.0, 1.0, 3, 40.5, 26.5, 78.0, 48.0),
        (48, 4.0, 1.0, 0, 10.5, 8.5, 33.693_749_999_999_994, 9.0),
        (48, 4.0, 1.0, 1, 30.5, 20.5, 420.933_333_333_333_34, 28.0),
        (48, 4.0, 1.0, 2, 3.5, 3.5, 14.625_000_000_000_002, 9.0),
        (48, 4.0, 1.0, 3, 40.5, 26.5, 19.5, 12.0),
        (48, 0.2, 1.0, 0, 10.5, 8.5, 168.468_75, 45.714_285_714_285_715),
        (48, 0.2, 1.0, 1, 30.5, 20.5, 420.933_333_333_333_34, 28.0),
        (48, 0.2, 1.0, 2, 3.5, 3.5, 37.142_857_142_857_146, 22.857_142_857_142_858),
        (48, 0.2, 1.0, 3, 40.5, 26.5, 222.857_142_857_142_83, 137.142_857_142_857_14),
        (48, 9.0, 1.0, 0, 10.5, 8.5, 33.693_749_999_999_994, 9.0),
        (48, 9.0, 1.0, 1, 30.5, 20.5, 420.933_333_333_333_34, 28.0),
        (48, 9.0, 1.0, 2, 3.5, 3.5, 14.625_000_000_000_002, 9.0),
        (48, 9.0, 1.0, 3, 40.5, 26.5, 15.600_000_000_000_003, 9.600_000_000_000_001),
        (2048, 1.0, 1.0, 0, 10.5, 8.5, 239.600_000_000_000_02, 64.0),
        (2048, 1.0, 1.0, 1, 30.5, 20.5, 1_683.733_333_333_333_3, 112.0),
        (2048, 1.0, 1.0, 2, 3.5, 3.5, 52.0, 32.0),
        (2048, 1.0, 1.0, 3, 40.5, 26.5, 312.0, 192.0),
        (2048, 2.0, 1.75, 0, 10.5, 8.5, 209.65, 56.0),
        (2048, 2.0, 1.75, 1, 30.5, 20.5, 2_946.533_333_333_333, 196.0),
        (2048, 2.0, 1.75, 2, 3.5, 3.5, 45.5, 28.0),
        (2048, 2.0, 1.75, 3, 40.5, 26.5, 273.0, 168.0),
    ];
    assert_eq!(want.len(), 24, "all 24 recorded runs are covered");
    for &(gw, zoom, icon, li, px, py, side, fsz) in want {
        let env = LabelViewEnv { grid_w: gw, zoom_scale: zoom, icon_scale: icon };
        let b = boxed(&labels[li], &env);
        let at = format!("gw={gw} zoom={zoom} icon={icon} li={li}");
        assert_eq!((b.px, b.py), (px, py), "position for {at}");
        assert_eq!(b.fsz.to_bits(), fsz.to_bits(), "fsz for {at}");
        assert_eq!(b.side.to_bits(), side.to_bits(), "side for {at}");
        // shape, re-asserted from the harness: no run is degenerate
        assert!(b.side > 0.0 && b.fsz >= 9.0, "degenerate box for {at}");
    }
}

#[test]
fn a_fixed_mode_label_is_the_same_size_at_every_zoom_and_a_zoom_mode_one_is_not() {
    let labels = fixture_labels();
    let e1 = LabelViewEnv { grid_w: 48, zoom_scale: 1.0, icon_scale: 1.0 };
    let e4 = LabelViewEnv { grid_w: 48, zoom_scale: 4.0, icon_scale: 1.0 };
    assert_eq!(boxed(&labels[1], &e1).fsz, boxed(&labels[1], &e4).fsz);
    assert_ne!(boxed(&labels[0], &e1).fsz, boxed(&labels[0], &e4).fsz);
}

// ---------------------------------------------------------------------------
// _civLabelHitTest
// ---------------------------------------------------------------------------

fn hit_fixture() -> (Vec<MapLabel>, Vec<LabelBox>) {
    let env = LabelViewEnv { grid_w: 200, zoom_scale: 1.0, icon_scale: 1.0 };
    let mut a = MapLabel::new(10.0, 8.0, "Aldar");
    a.size = 16.0;
    let mut b = MapLabel::new(60.0, 40.0, "The Long Quiet Sea");
    b.angle = 12.0;
    b.arc = 0.4;
    b.size = 20.0;
    let mut c = MapLabel::new(12.0, 10.0, "ii");
    c.size = 10.0;
    let labels = vec![a, b, c];
    let boxes = labels.iter().map(|l| boxed(l, &env)).collect();
    (labels, boxes)
}

#[test]
fn hit_testing_matches_the_reference_including_its_misses_and_overlaps() {
    let (_, boxes) = hit_fixture();
    let h = LabelHandles::default();
    // (px, py, expected kind, expected index)
    // Label 1's own box is 420 cells wide (an 18-character name at size 20),
    // which is most of this 200x200 grid -- so it legitimately swallows every
    // probe that labels 0 and 2 do not, and the one true miss is the far
    // corner. That is the reference's answer, not a fixture accident: it is
    // exactly why the reference replaced its old fixed-radius-circle hit test.
    let want: &[(f64, f64, Option<LabelHitKind>, Option<usize>)] = &[
        (10.5, 8.5, Some(LabelHitKind::Box), Some(2)),   // label 2 overlaps 0 and 1, and wins
        (60.5, 40.5, Some(LabelHitKind::Box), Some(1)),
        (12.5, 10.5, Some(LabelHitKind::Box), Some(2)),
        (0.0, 0.0, Some(LabelHitKind::Box), Some(1)),
        (199.0, 199.0, None, None),                       // the negative control
        (150.0, 20.0, Some(LabelHitKind::Box), Some(1)),
        (30.0, 8.5, Some(LabelHitKind::Box), Some(1)),
        (60.5, 8.5, Some(LabelHitKind::Box), Some(1)),
        // Straddling label 1's own lower box edge by two cells. Added after
        // mutation testing found the `side / 2.0` half-side comparison
        // survived: with only a far-corner miss in the table, widening the
        // box changed nothing anyone looked at.
        (60.5, 190.0, Some(LabelHitKind::Box), Some(1)),
        (60.5, 192.0, None, None),
    ];
    for &(px, py, kind, idx) in want {
        let got = label_hit_test(&boxes, &h, px, py);
        assert_eq!(got.map(|g| g.kind), kind, "kind at ({px},{py})");
        assert_eq!(got.and_then(|g| g.index), idx, "index at ({px},{py})");
    }
    // Shape, re-asserted from the harness: hits, misses and an overlap are all
    // genuinely present, so this is not a table of eight nulls.
    assert!(want.iter().any(|w| w.2.is_some()));
    assert!(want.iter().any(|w| w.2.is_none()));
    assert!(want.iter().any(|w| w.3 == Some(2)));
}

#[test]
fn armed_handles_beat_every_box_in_the_references_own_order() {
    let (_, boxes) = hit_fixture();
    let h = LabelHandles {
        resize: Some(HandleCircle { x: 120.0, y: 140.0, r: 2.0 }),
        rotate: Some(HandleCircle { x: 130.0, y: 140.0, r: 2.0 }),
        arc: Some(HandleCircle { x: 140.0, y: 140.0, r: 2.0 }),
        check: Some(HandleCircle { x: 150.0, y: 140.0, r: 1.0 }),
        cross: Some(HandleCircle { x: 160.0, y: 140.0, r: 1.0 }),
    };
    let want: &[(f64, f64, Option<LabelHitKind>)] = &[
        (120.0, 140.0, Some(LabelHitKind::Resize)),
        (130.0, 140.0, Some(LabelHitKind::Rotate)),
        (140.0, 140.0, Some(LabelHitKind::Arc)),
        (150.0, 140.0, Some(LabelHitKind::Check)),
        (160.0, 140.0, Some(LabelHitKind::Cross)),
        (151.29, 140.0, Some(LabelHitKind::Check)),      // inside the 1.3x slack
        (151.31, 140.0, Some(LabelHitKind::Box)),        // just outside it -> falls through
        (122.01, 140.0, Some(LabelHitKind::Box)),        // just past the resize radius
        (10.5, 8.5, Some(LabelHitKind::Box)),
        (180.0, 180.0, Some(LabelHitKind::Box)),
    ];
    for &(px, py, kind) in want {
        assert_eq!(label_hit_test(&boxes, &h, px, py).map(|g| g.kind), kind, "at ({px},{py})");
    }
    // All five armed kinds really are reachable in this table.
    for k in [LabelHitKind::Resize, LabelHitKind::Rotate, LabelHitKind::Arc,
              LabelHitKind::Check, LabelHitKind::Cross] {
        assert!(want.iter().any(|w| w.2 == Some(k)), "{k:?} unreachable");
    }
}

// ---------------------------------------------------------------------------
// select / confirm / cancel
// ---------------------------------------------------------------------------

#[test]
fn the_edit_session_matches_the_references_snapshot_semantics() {
    let mut labels = vec![{
        let mut lb = MapLabel::new(5.0, 5.0, "Aldar");
        lb.angle = 3.0;
        lb.arc = 0.1;
        lb.size = 20.0;
        lb
    }];
    let mut s = LabelEditSession::new();
    s.select(&labels, Some(0));
    let snap = s.snapshot().expect("snapshot").clone();
    // The reference's recorded snapshot, field for field.
    assert_eq!(snap.name, "Aldar");
    assert_eq!(snap.angle, 3.0);
    assert_eq!(snap.arc, 0.1);
    assert_eq!(snap.size, 20.0);
    assert_eq!(snap.font, "Georgia, serif");
    assert_eq!(snap.color, "#f0e4c8");
    assert_eq!(snap.size_mode, LabelSizeMode::Zoom);

    labels[0].name = "Aldar Reach".into();
    labels[0].angle = 45.0;
    labels[0].arc = -0.6;
    labels[0].size = 33.0;
    s.select(&labels, Some(0)); // re-select must NOT retake the snapshot
    assert_eq!(s.snapshot().expect("snapshot"), &snap);

    labels[0].x = 11.0;
    labels[0].y = 12.0;
    s.cancel(&mut labels);
    // The reference's own post-cancel object: style reverted, position kept.
    assert_eq!(labels[0].name, "Aldar");
    assert_eq!(labels[0].angle, 3.0);
    assert_eq!(labels[0].arc, 0.1);
    assert_eq!(labels[0].size, 20.0);
    assert_eq!((labels[0].x, labels[0].y), (11.0, 12.0));
}

#[test]
fn confirming_keeps_the_edits_and_clears_the_selection() {
    let mut labels = vec![MapLabel::new(1.0, 1.0, "Bree")];
    let mut s = LabelEditSession::new();
    s.select(&labels, Some(0));
    labels[0].name = "Breeland".into();
    labels[0].size = 22.0;
    s.confirm();
    assert_eq!(labels[0].name, "Breeland");
    assert_eq!(labels[0].size, 22.0);
    assert_eq!(s.selected(), None);
}

// ---------------------------------------------------------------------------
// Handle drag math (transcribed -- see the module docs)
// ---------------------------------------------------------------------------

#[test]
fn the_resize_handle_matches_the_reference() {
    let want: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
        (16.0, 10.0, 10.0, 14.0, 14.0, 5.0, 20.364_675_298_172_57),
        (16.0, 10.0, 10.0, 10.0, 10.0, 5.0, 8.0),   // clamped low
        (16.0, 10.0, 10.0, 60.0, 60.0, 3.0, 48.0),  // clamped high
        (40.0, 10.0, 10.0, 11.0, 11.0, 20.0, 8.0),
        (8.0, 10.0, 10.0, 12.0, 12.0, 0.5, 48.0),
    ];
    for &(s0, cx, cy, gx, gy, d0, out) in want {
        assert_eq!(label_resize_size(s0, cx, cy, gx, gy, d0).to_bits(), out.to_bits(),
                   "resize {s0} {cx} {cy} {gx} {gy} {d0}");
    }
}

#[test]
fn the_rotate_handle_matches_the_reference() {
    let want: &[(f64, f64, f64, f64, f64)] = &[
        (10.0, 10.0, 10.0, 5.0, 6.340_191_745_909_919_5),
        (10.0, 10.0, 10.0, 15.0, 174.805_571_092_265_15),
        (10.0, 10.0, 5.0, 10.0, -96.340_191_745_909_92),
        (10.0, 10.0, 15.0, 10.0, 95.194_428_907_734_85),
        (10.0, 10.0, 9.5, 9.5, 90.0),
        (10.0, 10.0, 14.0, 6.0, 52.125_016_348_901_795),
        (10.0, 10.0, 9.5, 14.0, -180.0),
    ];
    for &(cx, cy, gx, gy, out) in want {
        assert_eq!(label_rotate_deg(cx, cy, gx, gy).to_bits(), out.to_bits(),
                   "rotate {cx} {cy} {gx} {gy}");
    }
}

#[test]
fn the_arc_handle_matches_the_reference() {
    let want: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
        (10.0, 10.0, 0.0, 20.0, 10.0, 0.0, -0.025),
        (10.0, 10.0, 0.0, 20.0, 10.0, 20.0, -1.0),   // clamped
        (10.0, 10.0, 90.0, 20.0, 10.0, 0.0, -0.475), // the inverse rotation matters
        (10.0, 10.0, 45.0, 60.0, 30.0, 30.0, -0.555_555_555_555_555_6),
        (10.0, 10.0, -30.0, 8.0, 10.0, 4.0, 0.025_656_986_040_720_663),
        (10.0, 10.0, 0.0, 20.0, 10.0, -40.0, 1.0),   // clamped the other way
    ];
    for &(cx, cy, ang, side, gx, gy, out) in want {
        assert_eq!(label_arc_value(cx, cy, ang, side, gx, gy).to_bits(), out.to_bits(),
                   "arc {cx} {cy} {ang} {side} {gx} {gy}");
    }
    // The grab-angle inverse rotation is genuinely load-bearing: the same
    // pointer at two different label angles must give two different arcs.
    assert_ne!(label_arc_value(10.0, 10.0, 0.0, 20.0, 10.0, 0.0),
               label_arc_value(10.0, 10.0, 90.0, 20.0, 10.0, 0.0));
}
