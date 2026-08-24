//! Golden-parity tests for `cartalith-assets`' rule-driven icon placement
//! against the **real** reference implementation — `placeMapIconsRuled`
//! (reference `Cartalith Gen1 v2.10.html` line 7194), `TREE_SLOT`/
//! `SCATTER_SLOT`/`iconSlotForItem` (7289-7300) and `spriteDrawRect` (12173).
//!
//! Generated from a Node `vm` extraction run (harness transient, not checked
//! in — the same technique `tests/golden_parity_scatter_rules.rs` and
//! milestones 1-2 use) that loads those functions plus `hash` verbatim by
//! line range out of the frozen HTML and calls them on the fixtures below.
//! **The expected values here are that run's output verbatim**, not a
//! hand-derivation of what the port "should" produce.
//!
//! This is the first milestone in this crate with real golden-parity
//! *placement* surface: every draw is `hash(x,y,seed±k)` on a cell
//! coordinate, so the comparison is exact — integer cell coordinates and
//! string keys, not a tolerance-bearing float field. `s` (the size
//! multiplier) does carry a float and is compared to 1e-9, comfortably
//! inside `f64` round-trip precision for values in `[0,2]`.

use cartalith_assets::{
    IconCategory, IconKind, PlaceIconsRuledOpts, PlacedIcon, ScatterMode, ScatterRule,
    icon_slot_for_item, place_map_icons_ruled, sprite_draw_rect,
};

/// A convenience constructor mirroring the harness' own `rule(over)` helper:
/// `defaultScatterRule()`-shaped, with only the named fields overridden.
fn rule(mode: ScatterMode, biomes: &[f64], require_wetland: bool, elev: (Option<f64>, Option<f64>), size: (f64, f64)) -> ScatterRule {
    ScatterRule {
        enabled: true,
        mode,
        biomes: biomes.to_vec(),
        min_size: size.0,
        max_size: size.1,
        density: 1.0,
        spacing: None,
        elev_min: elev.0,
        elev_max: elev.1,
        require_wetland,
        variant_weights: None,
    }
}

fn scatter_rule(biomes: &[f64], require_wetland: bool) -> ScatterRule {
    rule(ScatterMode::Scatter, biomes, require_wetland, (None, None), (0.7, 1.2))
}

/// The 10x8 fixture grid every `placeMapIconsRuled` case below shares: a
/// single circular elevation peak at the grid centre, biome cycling through
/// `(x*3+y*5)%14`, and a wetland mask on `(x+y)%4==0`.
struct Grid {
    w: usize,
    h: usize,
    fld: Vec<f64>,
    biome: Vec<u8>,
    wetland: Vec<u8>,
}

fn fixture_grid() -> Grid {
    let (w, h) = (10usize, 8usize);
    let (cx, cy) = (4.5f64, 3.5f64);
    let maxd = (cx * cx + cy * cy).sqrt();
    let mut fld = vec![0.0; w * h];
    let mut biome = vec![0u8; w * h];
    let mut wetland = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let d = (((x as f64) - cx).powi(2) + ((y as f64) - cy).powi(2)).sqrt();
            fld[i] = (1.0 - d / maxd).max(0.0);
            biome[i] = ((x * 3 + y * 5) % 14) as u8;
            wetland[i] = if (x + y) % 4 == 0 { 1 } else { 0 };
        }
    }
    Grid { w, h, fld, biome, wetland }
}

/// The eight-rule table every broad-sweep case shares, in the harness' own
/// (deliberately non-alphabetical, non-priority-ordered) insertion order.
fn fixture_rules() -> Vec<(&'static str, ScatterRule)> {
    vec![
        ("generic_land", scatter_rule(&[], false)),
        ("narrow_biome", scatter_rule(&[7.0], false)),
        (
            "hill",
            rule(ScatterMode::Relief, &[], false, (Some(0.53), Some(0.58)), (0.5, 1.0)),
        ),
        ("wetland_grass", scatter_rule(&[7.0], true)),
        (
            "mountain",
            rule(ScatterMode::Relief, &[], false, (Some(0.58), None), (0.55, 1.0)),
        ),
        ("ghost_biome", scatter_rule(&[5.5], false)),
        ("tree_conifer", scatter_rule(&[3.0, 4.0], false)),
        (
            "any_relief",
            rule(ScatterMode::Relief, &[], false, (None, None), (0.4, 0.6)),
        ),
    ]
}

fn refs<'a>(rules: &'a [(&'static str, ScatterRule)]) -> Vec<(&'a str, &'a ScatterRule)> {
    rules.iter().map(|(k, r)| (*k, r)).collect()
}

fn run(grid: &Grid, sea: f64, seed: i32, t_gap: usize, with_biome: bool, with_wetland: bool, rules: &[(&str, &ScatterRule)]) -> Vec<PlacedIcon> {
    let opts = PlaceIconsRuledOpts {
        sea,
        seed,
        t_gap,
        wetland_mask: with_wetland.then_some(grid.wetland.as_slice()),
        rules,
    };
    place_map_icons_ruled(
        &grid.fld,
        with_biome.then_some(grid.biome.as_slice()),
        grid.w,
        grid.h,
        &opts,
    )
}

fn canon(items: &[PlacedIcon]) -> Vec<String> {
    items
        .iter()
        .map(|it| format!("{},{},key={},s={:.10}", it.x, it.y, it.key.as_deref().unwrap_or("null"), it.s))
        .collect()
}

#[test]
fn base_case_matches_the_reference() {
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 0.42, 7, 3, true, true, &refs(&rules));
    assert_eq!(
        canon(&items),
        vec![
            "4,3,key=mountain,s=0.7708717957",
            "3,3,key=generic_land,s=0.9610856497",
        ]
    );
}

#[test]
fn a_different_seed_reshuffles_the_scatter_grid() {
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 0.42, 11, 3, true, true, &refs(&rules));
    assert_eq!(
        canon(&items),
        vec![
            "3,2,key=generic_land,s=0.8266680018",
            "4,3,key=mountain,s=0.7708717957",
            "3,4,key=generic_land,s=1.1019682797",
            "7,4,key=generic_land,s=1.1258058076",
            "5,6,key=tree_conifer,s=0.8514280298",
        ]
    );
}

#[test]
fn absent_wetland_mask_matches_the_reference() {
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 0.42, 7, 3, true, false, &refs(&rules));
    assert_eq!(
        canon(&items),
        vec![
            "4,3,key=mountain,s=0.7708717957",
            "3,3,key=generic_land,s=0.9610856497",
        ]
    );
}

#[test]
fn absent_biome_array_matches_the_reference() {
    // None of this fixture's rules match without a biome array except the
    // unrestricted ones (generic_land, the two relief rules) -- identical to
    // the base case here because no biome-restricted rule won any sampled
    // cell in the base case either.
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 0.42, 7, 3, false, true, &refs(&rules));
    assert_eq!(
        canon(&items),
        vec![
            "4,3,key=mountain,s=0.7708717957",
            "3,3,key=generic_land,s=0.9610856497",
        ]
    );
}

#[test]
fn no_rules_places_nothing() {
    let grid = fixture_grid();
    let empty: Vec<(&str, &ScatterRule)> = Vec::new();
    let items = run(&grid, 0.42, 7, 3, true, true, &empty);
    assert!(items.is_empty());
}

#[test]
fn a_sea_level_above_every_elevation_places_nothing() {
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 1.0, 7, 3, true, true, &refs(&rules));
    assert!(items.is_empty());
}

#[test]
fn a_denser_grid_exercises_every_rule_family_and_matches_the_reference() {
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 0.2, 7, 2, true, true, &refs(&rules));
    assert_eq!(
        canon(&items),
        vec![
            "4,0,key=generic_land,s=1.1987537969",
            "1,1,key=generic_land,s=1.0016827543",
            "2,1,key=generic_land,s=1.1949361208",
            "7,1,key=generic_land,s=1.0897902540",
            "5,2,key=generic_land,s=0.7361504110",
            "6,2,key=generic_land,s=0.7813808639",
            "4,3,key=mountain,s=0.8338820519",
            "1,3,key=tree_conifer,s=0.9132321350",
            "2,3,key=narrow_biome,s=0.9158812442",
            "4,4,key=tree_conifer,s=0.9858021507",
            "7,4,key=generic_land,s=0.9555131561",
            "1,5,key=generic_land,s=0.8250400102",
            "3,5,key=generic_land,s=1.0224927408",
            "8,6,key=any_relief,s=0.4113815410",
            "5,6,key=tree_conifer,s=0.9839591321",
            "2,7,key=generic_land,s=1.1445397736",
            "6,7,key=generic_land,s=1.0591161928",
        ]
    );
}

#[test]
fn a_denser_grid_at_another_seed_matches_the_reference() {
    let grid = fixture_grid();
    let rules = fixture_rules();
    let items = run(&grid, 0.2, 99, 2, true, true, &refs(&rules));
    assert_eq!(
        canon(&items),
        vec![
            "2,0,key=generic_land,s=1.1365463410",
            "6,0,key=tree_conifer,s=0.8432828117",
            "1,1,key=generic_land,s=1.0900912920",
            "5,1,key=generic_land,s=1.0607153799",
            "3,2,key=generic_land,s=1.0721609576",
            "7,2,key=tree_conifer,s=0.7232346921",
            "8,2,key=generic_land,s=0.9079328883",
            "4,3,key=mountain,s=0.8338820519",
            "1,3,key=tree_conifer,s=0.7683549344",
            "4,3,key=generic_land,s=0.7741111917",
            "3,4,key=generic_land,s=0.7935737165",
            "4,4,key=tree_conifer,s=1.0293166790",
            "7,4,key=generic_land,s=0.9568697488",
            "9,4,key=generic_land,s=0.7533302815",
            "8,6,key=any_relief,s=0.4113815410",
            "4,6,key=generic_land,s=0.8736272244",
            "6,6,key=generic_land,s=1.0931874428",
            "3,7,key=generic_land,s=0.7011403138",
        ]
    );
}

// ============================================================================
// v1.27 fix proof: priority sort + requireWetland ANDed with biome
// ============================================================================
//
// A tiny hand-traceable fixture, deliberately not part of the broad sweeps
// above: a 3x1 grid, `sea=-1` (every cell is land regardless of `fld`), and
// `tGap=1`. That last choice is the trick that makes this fixture provable by
// hand rather than merely "matches whatever the reference happened to do":
// `hash(*)` is always in `[0,1)`, so `(hash(gx,gy,seed)*1)|0` is always `0`,
// meaning the scatter grid's jitter degenerates to zero and `jx=gx, jy=gy`
// exactly for every cell. Confirmed against the reference's own `hash` in the
// harness run rather than assumed.
//
// Cells: (0,0) biome=grass(7) + wetland, (1,0) biome=grass(7) + dry,
// (2,0) biome=shrub(8) + wetland.
#[test]
fn v1_27_fix_proof_priority_and_wetland_and() {
    let grid = Grid {
        w: 3,
        h: 1,
        fld: vec![1.0, 1.0, 1.0],
        biome: vec![7, 7, 8],
        wetland: vec![1, 0, 1],
    };
    let wetland_grass = scatter_rule(&[7.0], true);
    let narrow_biome = scatter_rule(&[7.0], false);
    let generic_land = scatter_rule(&[], false);

    // Insertion order deliberately puts the LEAST specific rule first, to
    // prove the winner is a function of specificity, not array position.
    let rules = [
        ("generic_land", &generic_land),
        ("narrow_biome", &narrow_biome),
        ("wetland_grass", &wetland_grass),
    ];

    for seed in [7, 42, 123] {
        let items = run(&grid, -1.0, seed, 1, true, true, &rules);
        let got: Vec<(i32, i32, &str)> = items
            .iter()
            .map(|it| (it.x, it.y, it.key.as_deref().unwrap()))
            .collect();
        assert_eq!(
            got,
            vec![
                (0, 0, "wetland_grass"), // wetland AND grass: most specific, wins despite being last in the array
                (1, 0, "narrow_biome"),  // grass but dry: wetland_grass's wetland half fails -- the v1.27 AND, not OR
                (2, 0, "generic_land"),  // wetland but shrub: wetland_grass's biome half fails too
            ],
            "seed {seed}"
        );
    }

    // Reversing the insertion order must not change the outcome -- that is
    // the whole point of the priority-sort fix.
    let rules_reversed = [
        ("wetland_grass", &wetland_grass),
        ("narrow_biome", &narrow_biome),
        ("generic_land", &generic_land),
    ];
    let items = run(&grid, -1.0, 7, 1, true, true, &rules_reversed);
    let got: Vec<(i32, i32, &str)> = items
        .iter()
        .map(|it| (it.x, it.y, it.key.as_deref().unwrap()))
        .collect();
    assert_eq!(
        got,
        vec![(0, 0, "wetland_grass"), (1, 0, "narrow_biome"), (2, 0, "generic_land")]
    );
}

// ============================================================================
// iconSlotForItem
// ============================================================================

fn item(cat: IconCategory, kind: Option<IconKind>, key: Option<&str>) -> PlacedIcon {
    PlacedIcon { x: 0, y: 0, s: 1.0, key: key.map(str::to_string), cat, kind }
}

#[test]
fn icon_slot_for_item_matches_the_reference_on_every_shape() {
    let cases: &[(&str, PlacedIcon)] = &[
        ("mountain-cat", item(IconCategory::Mountain, None, None)),
        ("hill-cat", item(IconCategory::Hill, None, None)),
        ("tree-conifer", item(IconCategory::Tree, Some(IconKind::Conifer), None)),
        ("tree-broadleaf", item(IconCategory::Tree, Some(IconKind::Broadleaf), None)),
        ("tree-rainforest", item(IconCategory::Tree, Some(IconKind::Rainforest), None)),
        ("tree-savanna", item(IconCategory::Tree, Some(IconKind::Savanna), None)),
        ("tree-wetland", item(IconCategory::Tree, Some(IconKind::Wetland), None)),
        ("tree-unknown-kind", item(IconCategory::Tree, None, None)),
        ("scatter-shrub", item(IconCategory::Scatter, Some(IconKind::Shrub), None)),
        ("scatter-cactus", item(IconCategory::Scatter, Some(IconKind::Cactus), None)),
        ("scatter-boulder", item(IconCategory::Scatter, Some(IconKind::Boulder), None)),
        ("scatter-unknown-kind", item(IconCategory::Scatter, None, None)),
        ("ruled-with-key", item(IconCategory::Ruled, None, Some("custom::Trees::oak"))),
        ("ruled-empty-key", item(IconCategory::Ruled, None, Some(""))),
        ("key-wins-over-mountain-cat", item(IconCategory::Mountain, None, Some("mountain_pack"))),
        ("ruled-no-kind", item(IconCategory::Ruled, None, Some("shrub"))),
    ];
    let expected = [
        "mountain",
        "hill",
        "tree_conifer",
        "tree_broadleaf",
        "tree_rainforest",
        "tree_savanna",
        "tree_wetland",
        "tree_broadleaf",
        "shrub",
        "cactus",
        "boulder",
        "shrub",
        "custom::Trees::oak",
        "shrub",
        "mountain_pack",
        "shrub",
    ];
    for ((name, it), want) in cases.iter().zip(expected.iter()) {
        assert_eq!(icon_slot_for_item(it), *want, "case {name}");
    }
}

// ============================================================================
// spriteDrawRect
// ============================================================================

/// `(name, x, y, s, base, sw, sh, (dx, dy, dw, dh))`.
type SpriteRectCase = (&'static str, f64, f64, f64, f64, f64, f64, (f64, f64, f64, f64));

#[test]
fn sprite_draw_rect_matches_the_reference() {
    let cases: &[SpriteRectCase] = &[
        ("normal", 50.0, 80.0, 1.3, 4.5, 32.0, 48.0, (45.71, 67.13, 8.58, 12.870000000000001)),
        ("sh-zero", 100.0, 200.0, 1.0, 5.0, 64.0, 0.0, (-252.0, 189.0, 704.0, 11.0)),
        ("square", 0.0, 0.0, 1.0, 10.0, 10.0, 10.0, (-11.0, -22.0, 22.0, 22.0)),
        ("tall-sprite", 300.0, 150.0, 0.75, 6.0, 20.0, 60.0, (298.35, 140.1, 3.3000000000000003, 9.9)),
    ];
    for (name, x, y, s, base, sw, sh, (dx, dy, dw, dh)) in cases {
        let r = sprite_draw_rect(*x, *y, *s, *base, *sw, *sh);
        assert!((r.dx - dx).abs() < 1e-9, "{name} dx");
        assert!((r.dy - dy).abs() < 1e-9, "{name} dy");
        assert!((r.dw - dw).abs() < 1e-9, "{name} dw");
        assert!((r.dh - dh).abs() < 1e-9, "{name} dh");
    }
}
