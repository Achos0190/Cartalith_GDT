//! `wallside`'s own checks. The before/after measurement across real towns is
//! `generate`'s concern and lives beside its golden; these pin the helpers.

use super::*;

#[test]
fn face_offset_is_half_the_drawn_stroke_and_refuses_the_rest() {
    // Literals, not the function against itself: `urban_layout_draw.gd`'s
    // `WALL_W` is curtain 4.5 and palisade 2.2.
    assert_eq!(face_offset("curtain"), Some(2.25));
    assert_eq!(face_offset("palisade"), Some(1.1));
    assert_eq!(face_offset("ditch"), None);
    assert_eq!(face_offset("bastioned"), None);
}

#[test]
fn arc_walks_open_and_closed() {
    let sq = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)];
    let open = Arc::new(&sq, false);
    let closed = Arc::new(&sq, true);
    assert_eq!(open.len(), 30.0);
    assert_eq!(closed.len(), 40.0);
    assert_eq!(open.at(15.0), Vec2::new(10.0, 5.0));
    assert_eq!(open.at(99.0), Vec2::new(0.0, 10.0), "an open arc clamps");
    assert_eq!(closed.at(45.0), Vec2::new(5.0, 0.0), "a closed arc wraps");
    assert_eq!(closed.at(-5.0), Vec2::new(0.0, 5.0), "and wraps backwards");
    assert_eq!(closed.project(Vec2::new(12.0, 4.0)), 14.0);
}

#[test]
fn nothing_is_platted_against_an_unwalled_or_unbuildable_circuit() {
    let site = crate::site::build_site(1, 1700.0, 1250.0, "inland", Default::default());
    let g = Graph::new();
    assert!(build_wall_lots(1, &g, &WallState::default(), &site, &[], &[], 5000.0, 8).is_empty());
    let ring = vec![Vec2::new(500.0, 500.0), Vec2::new(900.0, 500.0), Vec2::new(900.0, 800.0), Vec2::new(500.0, 800.0)];
    for style in ["ditch", "bastioned"] {
        let w = WallState {
            ring: Some(ring.clone()),
            land_arc: Some(ring.clone()),
            style: style.into(),
            ..Default::default()
        };
        assert!(build_wall_lots(1, &g, &w, &site, &[], &[], 5000.0, 8).is_empty(), "{style}");
    }
}

#[test]
fn a_row_ends_raggedly_about_its_nominal_length() {
    // Literals, not the constants against themselves: onset 0.85, span 0.7.
    assert_eq!(taper_end_chance(0.0), 0.0);
    assert_eq!(taper_end_chance(0.85), 0.0, "never ends before 85% of nominal");
    assert!((taper_end_chance(1.2) - 0.5).abs() < 1e-12, "even odds at 120% of nominal");
    assert_eq!(taper_end_chance(1.55), 1.0, "always ended by 155%");
    assert_eq!(taper_end_chance(9.0), 1.0);
    // Monotone: a row that has survived further is never likelier to survive.
    let xs: Vec<f64> = (0..=30).map(|i| i as f64 * 0.05).collect();
    assert!(xs.windows(2).all(|w| taper_end_chance(w[0]) <= taper_end_chance(w[1])));
    // And outward-only back-line jitter, half the widest 3.5 m lane.
    assert_eq!(BACK_JITTER, 1.75);
}

#[test]
fn gate_quality_falls_off_the_gate_with_bounded_noise() {
    // Beside the gate, halfway, at the run's end and past it (a ragged row).
    assert_eq!(gate_quality(0.0, 80.0, 0.0), 1.0);
    assert_eq!(gate_quality(40.0, 80.0, 0.0), 0.5);
    assert_eq!(gate_quality(80.0, 80.0, 0.0), 0.0);
    assert_eq!(gate_quality(120.0, 80.0, 0.0), 0.0, "clamped, never negative");
    assert_eq!(gate_quality(0.0, 80.0, 0.15), 1.0, "clamped, never above 1");
    assert!((gate_quality(40.0, 80.0, -0.15) - 0.35).abs() < 1e-12);
    // "Not 100% a rule": the noise can invert two lots 0.3 of a run apart and
    // no further. Literal 0.15 = QUALITY_NOISE.
    assert_eq!(QUALITY_NOISE, 0.15);
    assert!(gate_quality(20.0, 80.0, -0.15) < gate_quality(40.0, 80.0, 0.15));
    assert!(gate_quality(10.0, 80.0, -0.15) > gate_quality(40.0, 80.0, 0.15));
}
