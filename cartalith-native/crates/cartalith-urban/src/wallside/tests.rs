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
