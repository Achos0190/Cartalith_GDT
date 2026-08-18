//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's Icon stamp
//! tool: `_carIconBrushRule` (reference line 15046), `_carIconBrushStamp`
//! (15051), `_carIconBox` (15319), `_carIconHitTest` (15325), the
//! click-to-place branch (9776-9784) and the resize handle (9721-9724).
//!
//! # The harness
//!
//! Node `vm.runInContext` over **whole `<script>` blocks** (#1 2084-14556,
//! #2 14563-26720), delimiters asserted against the real `<script>`/
//! `</script>` tags — see `cartalith-terrain/tests/golden_parity_amplify.rs`
//! for the full write-up, including the balance check that fired twice
//! (wrongly) and was fixed rather than deleted.
//!
//! # Seeding a function the reference deliberately left unseeded
//!
//! `_carIconBrushStamp` calls `Math.random`, on purpose — its own comment:
//! *"a brush stroke is an authoring ACTION whose result is persisted in
//! `state.mapIcons` — re-painting the same spot should add new icons, not
//! deterministically reproduce the previous ones."* There is therefore no
//! deterministic output to diff against unless the stream is fixed on both
//! sides, so the harness replaced `Math.random` **inside the vm context** with
//! a 32-bit LCG (`s = s*1664525 + 1013904223`, `s / 2^32`) and this file
//! drives the identical sequence. Disclosed as an environment modification —
//! but note that no function body is transcribed or edited, and the LCG only
//! feeds values in; every decision about them is the reference's own.
//!
//! Because the RNG is consumed **three times per accepted dart and twice per
//! rejected one**, this is a much sharper test than a per-icon comparison
//! looks: one extra or missing draw anywhere desynchronises every later dart,
//! so matching all 36 placed icons across eight runs pins the exact sequence
//! of accept/reject decisions, not merely the outcome.
//!
//! # The world under the brush is bit-identical, checked first
//!
//! The fixture is the same synthetic pure-arithmetic field the region-export
//! goldens use (no `sin`/`cos`/`exp`, and a deliberately **quantised** `% 11`
//! term). Both sides FNV-1a-64 its raw `f32` bytes; that hash is asserted
//! before any brush golden is trusted. It carries 370 land cells and 1166
//! water cells, so the brush's sea-level gate is genuinely exercised rather
//! than trivially satisfied.
//!
//! # Shape assertions before any golden was written down
//!
//! 36 icons across 8 runs; **2 runs legitimately empty** (a real negative
//! control, not an absent assertion); every placed icon in bounds and on land;
//! both accepted and rejected clicks present. All re-asserted here.
//!
//! # Transcription, disclosed
//!
//! Click placement and the resize handle are **transcribed**, not sliced: both
//! are inline in DOM event listeners rather than callable functions. Lines
//! 9776-9784 and 9721-9724 were copied verbatim into the harness with the DOM
//! reads replaced by parameters. Weaker evidence than a slice, stated rather
//! than implied.

use cartalith_assets::manual::*;
use cartalith_assets::scatter::{ScatterRule, ScatterRuleTable};

const GW: usize = 48;
const GH: usize = 32;
const SEA: f64 = 0.42;

fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
    let mut f = vec![0.0f32; gw * gh];
    let cx = gw as f64 * 0.42;
    let cy = gh as f64 * 0.55;
    let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
    for y in 0..gh {
        for x in 0..gw {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
            let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
            v += 0.05 * ((q as f64 / 10.0) - 0.5);
            v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
            f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
        }
    }
    f
}

fn fnv_f32(a: &[f32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in a {
        for &b in &v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

/// The harness's seeded stand-in for `Math.random`.
fn lcg(seed: u32) -> impl FnMut() -> f64 {
    let mut s = seed;
    move || {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        s as f64 / 4294967296.0
    }
}

fn feature() -> ArmedIcon {
    ArmedIcon { family: ManualIconFamily::Feature, slot: "mountain".into(), set: None }
}
fn custom() -> ArmedIcon {
    ArmedIcon { family: ManualIconFamily::Custom, slot: "thing".into(), set: Some("myset".into()) }
}

/// The rule the harness put in `assetRules` for `mountain`.
fn mountain_rule() -> ScatterRule {
    ScatterRule { min_size: 0.55, max_size: 1.9, density: 1.4, ..Default::default() }
}

#[test]
fn the_fixture_field_is_bit_identical_and_carries_both_land_and_water() {
    let f = synthetic_field(GW, GH, 5);
    assert_eq!(fnv_f32(&f), "e6a8f7dd46187082");
    let land = f.iter().filter(|v| **v as f64 > SEA).count();
    assert_eq!(land, 370);
    assert_eq!(f.len() - land, 1166);
}

#[test]
fn brush_rule_lookup_matches_the_reference() {
    let mut table = ScatterRuleTable::new();
    table.insert("mountain", mountain_rule());

    let known = icon_brush_rule(Some(&feature()), &table).expect("armed");
    assert_eq!((known.min_size, known.max_size, known.density), (0.55, 1.9, 1.4));

    let missing =
        ArmedIcon { family: ManualIconFamily::Feature, slot: "notinrules".into(), set: None };
    let unknown = icon_brush_rule(Some(&missing), &table).expect("armed");
    assert_eq!((unknown.min_size, unknown.max_size, unknown.density), (0.7, 1.2, 1.0));

    // A custom asset keys by `custom::<set>::<slot>` and, absent from the
    // table, also falls back to the default rule.
    assert_eq!(custom().rule_key(), "custom::myset::thing");
    let c = icon_brush_rule(Some(&custom()), &table).expect("armed");
    assert_eq!((c.min_size, c.max_size), (unknown.min_size, unknown.max_size));

    assert!(icon_brush_rule(None, &table).is_none());
}

/// One recorded brush run: `(x, y, scale)` per placed icon.
struct BrushRun {
    label: &'static str,
    seed: u32,
    armed: ArmedIcon,
    brush: IconBrush,
    taps: Vec<(f64, f64)>,
    per_tap: Vec<usize>,
    icons: Vec<(f64, f64, f64)>,
}

impl BrushRun {
    /// The rule `_carIconBrushStamp` resolves for itself via
    /// `_carIconBrushRule()`. The harness's `assetRules` held an entry for
    /// `mountain` only, so the custom-family run falls back to
    /// `defaultScatterRule()` and its scales come from `0.7..1.2` rather than
    /// `0.55..1.9`. Getting this wrong is visible: the *positions* still match
    /// (the RNG stream is unchanged) and only the sizes move.
    fn rule(&self) -> ScatterRule {
        let mut table = ScatterRuleTable::new();
        table.insert("mountain", mountain_rule());
        icon_brush_rule(Some(&self.armed), &table).expect("armed")
    }
}

fn runs() -> Vec<BrushRun> {
    let f = || feature();
    vec![
        BrushRun {
            label: "single tap, r=6 d=0.6",
            seed: 12345,
            armed: f(),
            brush: IconBrush { on: true, r: 6.0, density: 0.6 },
            taps: vec![(20.0, 10.0)],
            per_tap: vec![4],
            icons: vec![
                (21.0, 10.0, 1.283260322571732),
                (16.0, 9.0, 1.607658061676193),
                (25.0, 11.0, 1.853842646256089),
                (19.0, 14.0, 1.8490736709791236),
            ],
        },
        BrushRun {
            label: "drag, three taps on the same spot (thickening)",
            seed: 777,
            armed: f(),
            brush: IconBrush { on: true, r: 6.0, density: 0.6 },
            taps: vec![(20.0, 10.0), (20.0, 10.0), (20.0, 10.0)],
            per_tap: vec![4, 1, 0],
            icons: vec![
                (15.0, 9.0, 1.6811128467554226),
                (21.0, 13.0, 1.6794390222872606),
                (19.0, 8.0, 1.0683349846163765),
                (23.0, 9.0, 1.2960772431571967),
                (15.0, 13.0, 1.7413923610001802),
            ],
        },
        BrushRun {
            label: "tap over deep water only",
            seed: 42,
            armed: f(),
            brush: IconBrush { on: true, r: 3.0, density: 0.6 },
            taps: vec![(2.0, 30.0)],
            per_tap: vec![0],
            icons: vec![],
        },
        BrushRun {
            label: "tap at the grid corner (bounds rejection)",
            seed: 5150,
            armed: f(),
            brush: IconBrush { on: true, r: 8.0, density: 0.6 },
            taps: vec![(0.0, 0.0)],
            per_tap: vec![0],
            icons: vec![],
        },
        BrushRun {
            label: "max density, small brush",
            seed: 31337,
            armed: f(),
            brush: IconBrush { on: true, r: 4.0, density: 1.0 },
            taps: vec![(20.0, 10.0)],
            per_tap: vec![4],
            icons: vec![
                (18.0, 12.0, 1.2555964965606108),
                (18.0, 8.0, 1.4102866556146183),
                (23.0, 12.0, 0.8164463865803555),
                (22.0, 8.0, 1.0969836156233215),
            ],
        },
        BrushRun {
            label: "density below the 0.02 floor",
            seed: 8,
            armed: f(),
            brush: IconBrush { on: true, r: 5.0, density: 0.0 },
            taps: vec![(20.0, 10.0)],
            per_tap: vec![1],
            icons: vec![(20.0, 10.0, 0.9512199172168039)],
        },
        BrushRun {
            label: "custom family",
            seed: 2024,
            armed: custom(),
            brush: IconBrush { on: true, r: 5.0, density: 0.5 },
            taps: vec![(22.0, 9.0)],
            per_tap: vec![3],
            icons: vec![
                (27.0, 10.0, 1.0345384878339245),
                (20.0, 8.0, 1.194301235070452),
                (21.0, 13.0, 1.1756405433872714),
            ],
        },
        BrushRun {
            label: "big brush: the BRUSH_MAX_DARTS cap",
            seed: 60606,
            armed: f(),
            brush: IconBrush { on: true, r: 60.0, density: 1.0 },
            taps: vec![(24.0, 16.0)],
            per_tap: vec![19],
            icons: vec![
                (25.0, 11.0, 1.4911577436956578),
                (23.0, 24.0, 1.4520221115089953),
                (16.0, 15.0, 0.971925177949015),
                (12.0, 15.0, 1.418989379168488),
                (27.0, 24.0, 1.8991371586103922),
                (20.0, 20.0, 1.8110238581895828),
                (25.0, 15.0, 1.8126278197043575),
                (13.0, 23.0, 0.943854354484938),
                (20.0, 9.0, 1.5884786740294656),
                (16.0, 23.0, 1.7290201872354372),
                (21.0, 17.0, 1.662496913166251),
                (19.0, 28.0, 1.6932467284612356),
                (17.0, 12.0, 1.7106176753412),
                (33.0, 8.0, 1.5650334717240184),
                (24.0, 27.0, 1.153892252582591),
                (24.0, 8.0, 1.8700701457448303),
                (29.0, 13.0, 1.0277281687478534),
                (11.0, 18.0, 1.0640597657766193),
                (23.0, 21.0, 0.9016341911512427),
            ],
        },
        // The two runs below were added after mutation testing: with only the
        // eight above, the density floor, the spacing constant and the dart
        // oversample all survived, because a small saturated disc reaches the
        // same answer at either setting.
        BrushRun {
            label: "large brush at the density floor",
            seed: 424242,
            armed: f(),
            brush: IconBrush { on: true, r: 15.0, density: 0.0 },
            taps: vec![(20.0, 12.0)],
            per_tap: vec![1],
            icons: vec![(16.0, 17.0, 1.675101577246096)],
        },
        BrushRun {
            label: "an unsaturated drag across the landmass",
            seed: 191919,
            armed: f(),
            brush: IconBrush { on: true, r: 9.0, density: 0.85 },
            taps: vec![(14.0, 9.0), (18.0, 11.0), (22.0, 13.0), (26.0, 11.0), (30.0, 9.0)],
            per_tap: vec![5, 5, 1, 1, 0],
            icons: vec![
                (14.0, 11.0, 1.7798500236822292),
                (21.0, 13.0, 1.4231531217345035),
                (16.0, 17.0, 1.4100225172238425),
                (19.0, 7.0, 1.3734782006940804),
                (11.0, 13.0, 0.6945733941858635),
                (23.0, 17.0, 1.1559983006794936),
                (26.0, 13.0, 1.4869050859007984),
                (25.0, 8.0, 1.0166365213808604),
                (19.0, 20.0, 1.7858864763053133),
                (12.0, 17.0, 1.381606677651871),
                (29.0, 17.0, 0.5691963572287932),
                (26.0, 20.0, 1.7013888516812585),
            ],
        },
    ]
}

#[test]
fn the_icon_brush_matches_the_reference_dart_for_dart() {
    let field = synthetic_field(GW, GH, 5);
    let all = runs();
    let mut total = 0usize;
    let mut empty = 0usize;

    for r in &all {
        let rule = r.rule();
        let mut icons: Vec<ManualIcon> = Vec::new();
        let mut rng = lcg(r.seed);
        let mut per_tap = Vec::new();
        for &(cx, cy) in &r.taps {
            per_tap.push(icon_brush_stamp(&mut icons, Some(&r.armed), &r.brush, &rule, &field,
                                          GW, GH, SEA, cx, cy, &mut rng));
        }
        assert_eq!(per_tap, r.per_tap, "{}: per-tap counts", r.label);
        assert_eq!(icons.len(), r.icons.len(), "{}: total", r.label);
        for (i, (x, y, s)) in r.icons.iter().copied().enumerate() {
            assert_eq!(icons[i].x, x, "{} icon {i} x", r.label);
            assert_eq!(icons[i].y, y, "{} icon {i} y", r.label);
            assert_eq!(icons[i].scale.to_bits(), s.to_bits(), "{} icon {i} scale", r.label);
            assert_eq!(icons[i].family, r.armed.family, "{} icon {i} family", r.label);
            assert_eq!(icons[i].slot, r.armed.slot, "{} icon {i} slot", r.label);
            let want_set = match r.armed.family {
                ManualIconFamily::Custom => r.armed.set.clone(),
                _ => None,
            };
            assert_eq!(icons[i].set, want_set, "{} icon {i} set", r.label);
            // shape, re-asserted from the harness
            assert!((0.0..GW as f64).contains(&x) && (0.0..GH as f64).contains(&y));
            assert!(field[y as usize * GW + x as usize] as f64 > SEA, "{} placed into water", r.label);
        }
        total += icons.len();
        if icons.is_empty() {
            empty += 1;
        }
    }
    assert_eq!(total, 49, "49 icons across 10 runs");
    assert_eq!(empty, 2, "two runs legitimately empty -- the negative controls");
}

#[test]
fn icon_box_matches_the_reference_across_every_zoom_and_scale() {
    let icons = [
        ManualIcon { x: 10.0, y: 8.0, family: ManualIconFamily::Settlement, slot: "city".into(),
                     set: None, scale: 1.0 },
        ManualIcon { x: 30.0, y: 20.0, family: ManualIconFamily::Feature, slot: "mountain".into(),
                     set: None, scale: 2.5 },
        ManualIcon { x: 10.4, y: 8.4, family: ManualIconFamily::Custom, slot: "thing".into(),
                     set: Some("myset".into()), scale: 0.4 },
    ];
    // (grid_w, zoom, icon_scale, index, px, py, r, side)
    type BoxCase = (usize, f64, f64, usize, f64, f64, f64, f64);
    let want: &[BoxCase] = &[
        (48, 1.0, 1.0, 0, 10.5, 8.5, 5.0, 13.0),
        (48, 1.0, 1.0, 1, 30.5, 20.5, 12.5, 32.5),
        (48, 1.0, 1.0, 2, 10.9, 8.9, 2.0, 5.2),
        (2048, 3.0, 1.5, 0, 10.5, 8.5, 10.0, 26.0),
        (2048, 3.0, 1.5, 1, 30.5, 20.5, 25.0, 65.0),
        (2048, 3.0, 1.5, 2, 10.9, 8.9, 4.0, 10.4),
        (48, 0.1, 1.0, 0, 10.5, 8.5, 14.285_714_285_714_286, 37.142_857_142_857_146),
        (48, 0.1, 1.0, 1, 30.5, 20.5, 35.714_285_714_285_715, 92.857_142_857_142_86),
        (48, 0.1, 1.0, 2, 10.9, 8.9, 5.714_285_714_285_715, 14.857_142_857_142_86),
    ];
    for &(gw, zoom, s, k, px, py, r, side) in want {
        let env = IconViewEnv { grid_w: gw, zoom_scale: zoom, icon_scale: s };
        let b = icon_box(&icons[k], &env);
        let at = format!("gw={gw} zoom={zoom} s={s} k={k}");
        assert_eq!(b.px.to_bits(), px.to_bits(), "px for {at}");
        assert_eq!(b.py.to_bits(), py.to_bits(), "py for {at}");
        assert_eq!(b.r.to_bits(), r.to_bits(), "r for {at}");
        assert_eq!(b.side.to_bits(), side.to_bits(), "side for {at}");
    }
}

#[test]
fn icon_hit_testing_matches_the_reference_including_its_one_miss() {
    let icons = [
        ManualIcon { x: 10.0, y: 8.0, family: ManualIconFamily::Settlement, slot: "city".into(),
                     set: None, scale: 1.0 },
        ManualIcon { x: 30.0, y: 20.0, family: ManualIconFamily::Feature, slot: "mountain".into(),
                     set: None, scale: 2.5 },
        ManualIcon { x: 10.4, y: 8.4, family: ManualIconFamily::Custom, slot: "thing".into(),
                     set: Some("myset".into()), scale: 0.4 },
    ];
    let env = IconViewEnv { grid_w: 48, zoom_scale: 1.0, icon_scale: 1.0 };
    let boxes: Vec<IconBox> = icons.iter().map(|i| icon_box(i, &env)).collect();
    let want: &[(f64, f64, Option<IconHitKind>, Option<usize>)] = &[
        (10.5, 8.5, Some(IconHitKind::Box), Some(2)),  // the small custom icon is on top
        (30.5, 20.5, Some(IconHitKind::Box), Some(1)),
        (0.0, 0.0, None, None),                        // the negative control
        (10.9, 8.9, Some(IconHitKind::Box), Some(2)),
        (31.0, 19.0, Some(IconHitKind::Box), Some(1)),
    ];
    for &(px, py, kind, idx) in want {
        let got = icon_hit_test(&boxes, None, px, py);
        assert_eq!(got.map(|g| g.kind), kind, "kind at ({px},{py})");
        assert_eq!(got.and_then(|g| g.index), idx, "index at ({px},{py})");
    }
}

#[test]
fn click_placement_matches_the_reference() {
    let f = feature();
    let c = custom();
    // (gx, gy, armed, expected)
    type ClickCase<'a> = (f64, f64, Option<&'a ArmedIcon>, Option<(f64, f64, Option<&'a str>)>);
    let want: &[ClickCase] = &[
        (5.0, 5.0, Some(&f), Some((5.0, 5.0, None))),
        (5.0, 5.0, Some(&c), Some((5.0, 5.0, Some("myset")))),
        (-1.0, 5.0, Some(&f), None),
        (48.0, 5.0, Some(&f), None),
        (5.0, 32.0, Some(&f), None),
        (47.0, 31.0, Some(&f), Some((47.0, 31.0, None))),
        (5.0, 5.0, None, None),
    ];
    let mut placed = 0;
    let mut refused = 0;
    for &(gx, gy, armed, expected) in want {
        let got = place_manual_icon(gx, gy, GW, GH, armed);
        match expected {
            Some((x, y, set)) => {
                let ic = got.expect("expected a placement");
                assert_eq!((ic.x, ic.y), (x, y));
                assert_eq!(ic.scale, 1.0);
                assert_eq!(ic.set.as_deref(), set);
                placed += 1;
            }
            None => {
                assert!(got.is_none(), "expected a refusal at ({gx},{gy})");
                refused += 1;
            }
        }
    }
    // shape: both outcomes are genuinely present
    assert_eq!((placed, refused), (3, 4));
}

#[test]
fn the_icon_resize_handle_matches_the_reference() {
    let want: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
        (1.0, 10.0, 10.0, 14.0, 14.0, 5.0, 1.272_792_206_135_785_7),
        (1.0, 10.0, 10.0, 10.0, 10.0, 5.0, 0.2),  // clamped low
        (1.0, 10.0, 10.0, 60.0, 60.0, 3.0, 4.0),  // clamped high
        (0.3, 10.0, 10.0, 10.2, 10.2, 8.0, 0.2),
    ];
    for &(s0, cx, cy, gx, gy, d0, out) in want {
        assert_eq!(icon_resize_scale(s0, cx, cy, gx, gy, d0).to_bits(), out.to_bits(),
                   "resize {s0} {cx} {cy} {gx} {gy} {d0}");
    }
}
