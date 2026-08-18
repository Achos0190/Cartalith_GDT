//! Milestone 5's tests. **All nineteen `shoreFromMask` scenarios and all
//! thirty-six `buildSite` scenarios are golden** — every expected value is the
//! reference engine's own output, captured by slicing block 4 out of the frozen
//! HTML and running it under a bare `vm` context with no DOM.
//!
//! Nothing in this milestone is on `UME`'s public export or its `_test` one, so
//! the capture adds the three functions to the returned object with a single
//! anchored replacement of the `return {` line, asserted to match exactly once.
//! The reference file itself is never touched.
//!
//! Everything is compared **bit for bit** through [`f64::to_bits`], so a `NaN`
//! must be a `NaN` and a `-0` cannot pass for a `+0`. No tolerances anywhere,
//! including on `height`, which runs through `exp`, and `slope`, which runs
//! through `exp` and `js_hypot`.
//!
//! # The fixtures that exist because a passing golden proves nothing
//!
//! Milestone 3's finding, applied. A continuous random raster cannot observe a
//! tie-break and a quantised output cannot observe a small change to its
//! inputs, so both shapes are here deliberately:
//!
//! - **`plusShape`** is the quantised/symmetric tie fixture. One water cell in
//!   a 5 × 5 land field leaves four shoreline points whose scatter matrix is
//!   perfectly isotropic: `sxy == 0`, `l1 - sxx == 0` *and* `l1 - syy == 0`, so
//!   the principal axis collapses to `(0, 0)`, the `|| 1` on its length fires,
//!   **every** projection is exactly zero and every comparison is a tie. It is
//!   the only scenario that can see the sort's stability or the fallback
//!   eigenvector at all.
//! - **`bay` and `coast` share a seed.** The coastline branch draws its harbour
//!   abscissa only when the site is *not* a bay, so the two differ by exactly
//!   one draw and their `routeEnds` diverge. That one-draw asymmetry is
//!   invisible to any scenario that does not pair them.
//! - **`atoll` and `coast` share a seed too**, which is what shows that an
//!   unrecognised kind takes the coastline branch while keeping its own name —
//!   the reason `kind` is not an enum.
//! - **`terrainAllNaN`** makes every slope `NaN`, so `bi` is never assigned and
//!   `Math.max(0, bi)` is the only thing that places the bridge. Nothing with a
//!   finite height field can exercise that line.
//! - **`shortDt`, `terrainShortGrid`, `terrainOneColumn`** and the `NaN` probe
//!   reach JS's out-of-bounds `undefined`, which this port reproduces as
//!   `f64::NAN` rather than as a panic.
//! - **`maskTwo`** separates the two different truthiness tests the reference
//!   applies to one mask.
//! - **`widthFallbackZero` / `widthFallbackAbsent`** sit on both sides of
//!   `riverWidthM || 20`, and `orderNaN` on the `NaN` side of `|| 0`.
//!
//! # The fixtures that exist because a *grid* of probes tests almost nothing
//!
//! The first mutation round left 46 survivors, and almost none of them were
//! equivalent mutants — they were two specific fixture gaps, both worth
//! stating because every later milestone in this subsystem will have them:
//!
//! - **Hand-built rasters are uniform along one axis.** `j >= 9 ? water : land`
//!   is the obvious mask, and it makes every mutation of `mask_idx`'s `i` clamp
//!   invisible, because column 0 and column 16 hold identical data. Every mask
//!   here now carries a per-column ripple, so no two adjacent columns agree.
//! - **A fixed fractional grid never lands near anything.** The interesting
//!   geometry is a 10-40 m band around a polyline; a `[0.1, 0.5, 0.9]²` grid
//!   essentially never enters it, so the whole `riverW/2 + 2` water band, the
//!   shoreline half-plane and both ends of `y_at_x` went unexercised. Most of
//!   the 106 probes are now derived from **the site's own river**: offsets
//!   straddling the band boundary at three points along the centreline, and a
//!   ladder sitting a quarter-metre either side of — and exactly on — the real
//!   waterline at nine abscissae.
//!
//! Three fixtures exist purely to make a constant observable and say so:
//! `riverCeiling`/`throughCeiling` are seeds found by scanning for a channel
//! whose drift actually saturates its upper clamp; `quayLadder` is a
//! 18.85 m-per-segment river whose quay walk accumulates 94.25 m in five steps,
//! just under its own 95 m stop; and `twoRowMicro` is `twoRowShore` at 4 mm
//! cells, which drives the eigenvalue discriminant below 1 so the `max(0, .)`
//! guarding it can be perturbed at all.
//!
//! Mutation results, including every reported survivor and the invariant it
//! rests on, are in `URBAN_MORPHOLOGY_SCOPE.md`.

use super::*;

mod golden;

/// Bit-exact comparison, so `NaN == NaN` and `+0.0 != -0.0`.
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

fn eq(name: &str, what: &str, got: f64, want: f64) {
    assert!(same(got, want), "{name}: {what}: got {got:?} ({:x}), want {want:?} ({:x})", got.to_bits(), want.to_bits());
}

fn pts_of(flat: &[f64]) -> Vec<Vec2> {
    flat.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect()
}

fn eq_poly(name: &str, what: &str, got: &[Vec2], want: &[f64]) {
    assert_eq!(got.len() * 2, want.len(), "{name}: {what}: length");
    for (i, q) in got.iter().enumerate() {
        eq(name, &format!("{what}[{i}].x"), q.x, want[2 * i]);
        eq(name, &format!("{what}[{i}].y"), q.y, want[2 * i + 1]);
    }
}

fn water_of(s: &golden::WaterSpec) -> WaterCtx {
    WaterCtx {
        mask: s.mask.to_vec(),
        dt: s.dt.to_vec(),
        mw: s.mw,
        mh: s.mh,
        cell_m: s.cell_m,
        river_path: s.river_path.map(pts_of),
        river_width_m: s.river_width_m,
        river_order: s.river_order,
        sea_lake_cells: s.sea_lake_cells,
    }
}

fn terrain_of(s: &golden::TerrainSpec) -> TerrainCtx {
    TerrainCtx {
        grid: s.grid.to_vec(),
        mw: s.mw,
        mh: s.mh,
        cell_m: s.cell_m,
        h_min: s.h_min,
        h_max: s.h_max,
    }
}

fn opts_for(c: &golden::SiteCase) -> SiteOpts {
    SiteOpts {
        water: c.water.map(|i| water_of(&golden::WATERS[i])),
        terrain: c.terrain.map(|i| terrain_of(&golden::TERRAINS[i])),
        economy: None,
    }
}

/// The Rust-side mirror of the capture's own emptiness / shape gate. Three
/// subsystems in this project have shipped a harness whose output was silently
/// empty and passed every structural check; a truncated or half-written
/// `golden.rs` would otherwise make every test below vacuously pass.
#[test]
fn golden_data_is_not_vacuous() {
    assert!(golden::SHORES.len() >= 19, "too few shore scenarios");
    assert!(golden::SITES.len() >= 34, "too few site scenarios");
    assert!(golden::WATERS.len() >= 12 && golden::TERRAINS.len() >= 5);
    assert!(
        golden::SHORES.iter().filter(|c| c.pts.is_none()).count() >= 3,
        "no null shorelines"
    );
    assert!(
        golden::SHORES.iter().filter(|c| c.pts.is_some_and(|p| p.len() >= 8)).count() >= 6,
        "too few non-trivial shorelines"
    );
    for w in golden::WATERS {
        assert_eq!(w.mask.len().min(w.mw * w.mh), w.mask.len().min(w.mw * w.mh));
        assert!(w.mw > 0 && w.mh > 0 && w.cell_m > 0.0, "{}: degenerate raster", w.name);
    }
    let mut any_water_probe = false;
    let mut any_dry_site = false;
    let mut bank_signs = (false, false);
    let mut ts_zero = false;
    let mut ts_high = false;
    for c in golden::SITES {
        assert!(c.river.len() >= 4, "{}: river shorter than 2 points", c.name);
        assert!(c.river.iter().all(|v| v.is_finite()), "{}: non-finite river vertex", c.name);
        assert!(
            c.route_ends.len() == 6 || c.route_ends.len() == 8,
            "{}: routeEnds is neither 3 nor 4 long",
            c.name
        );
        assert_eq!(c.probes.len(), 106, "{}: probe count", c.name);
        // a real heightfield of nothing but NaN is a deliberate fixture, and
        // every probe on it is legitimately NaN; everywhere else a constant
        // height column would mean the probes are not probing anything
        assert!(
            c.probes.iter().all(|p| p.h.is_nan())
                || c.probes.iter().any(|p| !same(p.h, c.probes[0].h)),
            "{}: height is constant across every probe",
            c.name
        );
        any_water_probe |= c.probes.iter().any(|p| p.is_water == 1);
        any_dry_site |= c.probes.iter().all(|p| p.is_water == 0);
        for p in c.probes {
            if p.bank_side == 1.0 {
                bank_signs.0 = true;
            }
            if p.bank_side == -1.0 {
                bank_signs.1 = true;
            }
            if p.suitability == 0.0 {
                ts_zero = true;
            }
            if p.suitability > 0.5 {
                ts_high = true;
            }
        }
    }
    assert!(any_water_probe, "no probe anywhere is in water");
    assert!(any_dry_site, "no site is dry at every probe");
    assert!(bank_signs.0 && bank_signs.1, "bankSide never took both signs");
    assert!(ts_zero && ts_high, "terrainSuitability never spanned its range");
    // the NaN probe must really have produced a NaN somewhere
    assert!(
        golden::SITES.iter().any(|c| c.probes[14].h.is_nan()),
        "the NaN probe never produced a NaN height"
    );
}

#[test]
fn golden_shore_from_mask() {
    for c in golden::SHORES {
        let w = WaterCtx {
            mask: c.mask.to_vec(),
            dt: Vec::new(),
            mw: c.mw,
            mh: c.mh,
            cell_m: c.cell_m,
            river_path: None,
            river_width_m: None,
            river_order: 0.0,
            sea_lake_cells: 0.0,
        };
        let got = shore_from_mask(&w);
        match (&got, c.pts) {
            (None, None) => {}
            (Some(g), Some(want)) => eq_poly(c.name, "shore", g, want),
            _ => panic!("{}: null-ness disagrees (got {:?})", c.name, got.as_ref().map(Vec::len)),
        }
    }
}

/// The isotropic case, called out on its own because it is the only scenario
/// that observes the fallback eigenvector, the `|| 1` on the axis length, and
/// the sort's stability — and because a reader deleting one of those three
/// would otherwise see only "one golden failed".
#[test]
fn the_degenerate_axis_returns_raster_order() {
    let c = golden::SHORES.iter().find(|c| c.name == "plusShape").expect("plusShape");
    let want = c.pts.expect("plusShape is not null");
    // row-major over a 5 x 5 grid with one water cell at (2,2), cell 40 m
    let expected = [(2.0, 1.0), (1.0, 2.0), (3.0, 2.0), (2.0, 3.0)];
    assert_eq!(want.len(), 8);
    for (k, (i, j)) in expected.iter().enumerate() {
        assert_eq!(want[2 * k], (i + 0.5) * 40.0, "plusShape[{k}].x");
        assert_eq!(want[2 * k + 1], (j + 0.5) * 40.0, "plusShape[{k}].y");
    }
}

#[test]
fn golden_build_site() {
    for c in golden::SITES {
        let s = build_site(c.seed, c.wm, c.hm, c.kind, opts_for(c));
        let n = c.name;
        assert_eq!(s.kind, c.kind_out, "{n}: kind");
        assert_eq!(s.through, c.through, "{n}: through");
        assert_eq!(s.no_water, c.no_water, "{n}: noWater");
        eq(n, "riverW", s.river_w, c.river_w);
        eq_poly(n, "river", &s.river, c.river);
        eq_poly(n, "waterPoly", &s.water_poly, c.water_poly);
        eq_poly(n, "routeEnds", &s.route_ends, c.route_ends);
        match (s.bridge_pt, c.bridge_pt) {
            (None, None) => {}
            (Some(g), Some(w)) => {
                eq(n, "bridgePt.x", g.x, w[0]);
                eq(n, "bridgePt.y", g.y, w[1]);
            }
            _ => panic!("{n}: bridgePt null-ness disagrees"),
        }
        match (s.bridge_dir, c.bridge_dir) {
            (None, None) => {}
            (Some(g), Some(w)) => {
                eq(n, "bridgeDir.x", g.x, w[0]);
                eq(n, "bridgeDir.y", g.y, w[1]);
            }
            _ => panic!("{n}: bridgeDir null-ness disagrees"),
        }
        assert_eq!(s.uses_real_water, c.uses_real_water, "{n}: usesRealWater");
        assert_eq!(s.real_river, c.real_river, "{n}: realRiver");
        assert_eq!(s.uses_real_terrain, c.uses_real_terrain, "{n}: usesRealTerrain");
        eq(n, "terrainRelief", s.terrain_relief, c.terrain_relief);
        eq(n, "waterOrder", s.water_order, c.water_order);
        eq(n, "seaLakeCells", s.sea_lake_cells, c.sea_lake_cells);
        assert_eq!(s.harbour.idx, c.harbour_idx, "{n}: harbour.idx");
        match (s.harbour.pt, c.harbour_pt) {
            (None, None) => {}
            (Some(g), Some(w)) => {
                eq(n, "harbour.pt.x", g.x, w[0]);
                eq(n, "harbour.pt.y", g.y, w[1]);
            }
            _ => panic!("{n}: harbour.pt null-ness disagrees"),
        }
        assert!(s.economy.is_none(), "{n}: economy");
        for (k, p) in c.probes.iter().enumerate() {
            let q = Vec2::new(p.x, p.y);
            eq(n, &format!("height[{k}]"), s.height(q), p.h);
            eq(n, &format!("slope[{k}]"), s.slope(q), p.slope);
            eq(n, &format!("riverDist[{k}]"), s.river_dist(q), p.river_dist);
            assert_eq!(
                u8::from(s.is_water(q)),
                p.is_water,
                "{n}: isWater[{k}] at ({}, {})",
                p.x,
                p.y
            );
            eq(n, &format!("bankSide[{k}]"), s.bank_side(q), p.bank_side);
            eq(n, &format!("suitability[{k}]"), terrain_suitability(&s, q), p.suitability);
        }
    }
}

/// The site substream's draw budget, pinned directly rather than only through
/// the values it produces.
///
/// Twelve draws for the hills, then the branch's own, then three or four for
/// the route endpoints. Advancing a fresh stream by exactly that many draws and
/// rebuilding the endpoints by hand must reproduce the golden — so any mutation
/// that adds or drops a draw anywhere in `build_site` shows up here as well as
/// in the values, and shows up localised.
#[test]
fn site_substream_draw_budget() {
    // name, draws consumed between the hills and the route endpoints
    for (name, branch_draws) in [
        ("riverSeed1", 18usize),   // baseY, jitter, 15 x drift, riverW
        ("riverThrough", 18),
        ("coast", 32),             // baseY, bayAmp, bx, bw, 27 x sample, harbour abscissa
        ("atoll", 32),
        ("bay", 31),               // ... minus the harbour abscissa, which a bay reuses
        ("landlocked", 0),
        ("realRiver", 0),
        ("realShore", 0),
    ] {
        let c = golden::SITES.iter().find(|c| c.name == name).expect(name);
        let mut r = crate::rng::stream(c.seed, "site");
        for _ in 0..(12 + branch_draws) {
            r.u();
        }
        let ends: Vec<Vec2> = if c.route_ends.len() == 8 {
            vec![
                Vec2::new(r.range(0.08, 0.3) * c.wm, 0.0),
                Vec2::new(c.wm, r.range(0.1, 0.4) * c.hm),
                Vec2::new(0.0, r.range(0.15, 0.45) * c.hm),
                Vec2::new(r.range(0.4, 0.75) * c.wm, c.hm),
            ]
        } else {
            vec![
                Vec2::new(r.range(0.15, 0.4) * c.wm, 0.0),
                Vec2::new(c.wm, r.range(0.08, 0.3) * c.hm),
                Vec2::new(0.0, r.range(0.08, 0.32) * c.hm),
            ]
        };
        eq_poly(name, "routeEnds from the hand-advanced stream", &ends, c.route_ends);
    }
}

/// `bay` and `coast` differ by exactly one draw, and nothing else. A test
/// rather than a comment, because the asymmetry is easy to "tidy away".
#[test]
fn a_bay_draws_one_fewer_than_a_coast() {
    let bay = golden::SITES.iter().find(|c| c.name == "bay").unwrap();
    let coast = golden::SITES.iter().find(|c| c.name == "coast").unwrap();
    assert_eq!(bay.seed, coast.seed);
    assert_ne!(bay.route_ends, coast.route_ends, "the one-draw asymmetry is not observable");
    // and the bay really did indent its shoreline much harder than the coast
    let bay_y: Vec<f64> = bay.river.chunks(2).map(|c| c[1]).collect();
    let coast_y: Vec<f64> = coast.river.chunks(2).map(|c| c[1]).collect();
    let span = |v: &[f64]| v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min);
    assert!(span(&bay_y) > 3.0 * span(&coast_y), "the bay indent is not visible");
}

/// An unrecognised kind takes the coastline branch and keeps its own name. This
/// is the whole argument for `kind` being a `String` rather than an enum, and
/// milestone 9 reads `site.kind === 'coast'` directly.
#[test]
fn an_unknown_kind_is_a_coast_that_is_not_called_coast() {
    let atoll = golden::SITES.iter().find(|c| c.name == "atoll").unwrap();
    let coast = golden::SITES.iter().find(|c| c.name == "coast").unwrap();
    assert_eq!(atoll.kind_out, "atoll");
    assert_eq!(coast.kind_out, "coast");
    assert_eq!(atoll.river, coast.river, "the unknown kind did not take the coast branch");
    let s = build_site(atoll.seed, atoll.wm, atoll.hm, "atoll", SiteOpts::default());
    assert_eq!(s.kind, "atoll");
    assert!(!s.river_like());
    // and the falsy kind defaults to river
    let d = build_site(1, 1700.0, 1250.0, "", SiteOpts::default());
    assert_eq!(d.kind, "river");
    assert!(d.river_like());
}

/// One mask, two different truthiness tests, reproduced rather than unified.
#[test]
fn a_mask_cell_of_two_is_water_to_the_tracer_and_land_to_the_query() {
    let spec = golden::WATERS.iter().find(|w| w.name == "maskTwo").expect("maskTwo raster");
    let w = water_of(spec);
    let shore = shore_from_mask(&w).expect("a mask of 2s must still trace a shoreline");
    assert!(shore.len() >= 4);
    let s = build_site(4, 1700.0, 1250.0, "coast", SiteOpts { water: Some(w), ..Default::default() });
    // every cell is 0 or 2, so `mask[idx] === 1` is false everywhere
    for j in 0..spec.mh {
        for i in 0..spec.mw {
            let p = Vec2::new((i as f64 + 0.5) * spec.cell_m, (j as f64 + 0.5) * spec.cell_m);
            assert!(!s.is_water(p), "a mask cell of 2 read as water at ({i}, {j})");
        }
    }
}

/// JS reads past the end of an array and gets `undefined`; this port returns
/// `NaN` there rather than panicking. Three separate routes into it.
#[test]
fn out_of_bounds_reads_are_nan_not_panics() {
    let short = golden::WATERS.iter().find(|w| w.name == "shortDt").unwrap();
    let s = build_site(4, 1700.0, 1250.0, "coast", SiteOpts {
        water: Some(water_of(short)),
        ..Default::default()
    });
    // the dt raster stops after 40 of 221 cells, so anything past row 2 is NaN
    assert!(s.river_dist(Vec2::new(850.0, 900.0)).is_nan(), "short dt did not read NaN");
    assert!(s.river_dist(Vec2::new(850.0, 100.0)).is_finite(), "an in-range dt cell read NaN");
    // a NaN probe clamps to NaN and indexes `undefined`
    assert!(s.river_dist(Vec2::new(f64::NAN, 0.0)).is_nan());
    assert!(!s.is_water(Vec2::new(f64::NAN, f64::NAN)));
    // a short terrain grid
    let t = golden::TERRAINS.iter().find(|t| t.name == "shortGrid").unwrap();
    let s2 = build_site(6, 1700.0, 1250.0, "coast", SiteOpts {
        terrain: Some(terrain_of(t)),
        ..Default::default()
    });
    assert!(s2.height(Vec2::new(850.0, 1150.0)).is_nan(), "short grid did not read NaN");
    assert!(s2.height(Vec2::new(850.0, 100.0)).is_finite());
}

/// `Math.sign(x) || 1` never yields `0`, and this port must not either — a
/// point exactly on the centreline, a degenerate segment and a `NaN` probe all
/// have to come back `+1`. Swept over every golden site rather than asserted at
/// one point.
#[test]
fn bank_side_is_never_zero() {
    for c in golden::SITES {
        let s = build_site(c.seed, c.wm, c.hm, c.kind, opts_for(c));
        for i in 0..s.river.len() {
            let v = s.bank_side(s.river[i]);
            assert!(v == 1.0 || v == -1.0, "{}: bankSide on vertex {i} was {v}", c.name);
        }
        for v in [
            s.bank_side(Vec2::new(f64::NAN, f64::NAN)),
            s.bank_side(Vec2::new(f64::INFINITY, f64::INFINITY)),
            s.bank_side(Vec2::new(0.0, 0.0)),
        ] {
            assert!(v == 1.0 || v == -1.0, "{}: bankSide was {v}", c.name);
        }
    }
}

/// The NaN discriminator for [`js_min`] / [`js_max`], milestone 4's central
/// hazard, arriving at three new call sites.
///
/// `terrainSuitability` is `Math.max(0, Math.min(1, rd/margin))` over a
/// `Math.min(1, slope)`. With Rust's absorbing `f64::min` / `f64::max` a `NaN`
/// slope would come back as a **plausible finite score** instead of a `NaN`,
/// which is precisely how a bad site would pass the building gate. Written out
/// so the simplification fails loudly and with the reason attached, exactly as
/// `geom::js_hypot` and `rules::clamp` are.
#[test]
fn nan_must_propagate_through_the_suitability_clamps() {
    let t = golden::TERRAINS.iter().find(|t| t.name == "allNaN").unwrap();
    let s = build_site(6, 1700.0, 1250.0, "river", SiteOpts {
        terrain: Some(terrain_of(t)),
        ..Default::default()
    });
    let p = Vec2::new(700.0, 400.0);
    assert!(s.slope(p).is_nan(), "an all-NaN heightfield must give a NaN slope");
    assert!(
        terrain_suitability(&s, p).is_nan(),
        "terrainSuitability absorbed a NaN slope -- js_min/js_max were replaced by f64::min/f64::max, \
         and every site with a hole in its heightfield now scores as buildable"
    );
    // and the direct statement of the asymmetry, so the reason survives a refactor
    assert!(js_min(1.0, f64::NAN).is_nan() && js_max(0.0, f64::NAN).is_nan());
    assert_eq!(f64::min(1.0, f64::NAN), 1.0);
    assert_eq!(f64::max(0.0, f64::NAN), 0.0);
}

/// An all-NaN slope field never assigns `bi`, so `Math.max(0, bi)` is the only
/// thing placing the bridge. Nothing with a finite heightfield reaches it.
#[test]
fn a_bridge_with_no_finite_slope_lands_on_the_rivers_first_point() {
    let c = golden::SITES.iter().find(|c| c.name == "terrainAllNaN").unwrap();
    let s = build_site(c.seed, c.wm, c.hm, c.kind, opts_for(c));
    let b = s.bridge_pt.expect("a river site has a bridge point");
    assert_eq!((b.x, b.y), (s.river[0].x, s.river[0].y));
}

/// `riverWidthM || 20` and `riverOrder || 0` — the falsy arms, which no
/// well-formed adapter output reaches and every hand-built fixture can.
#[test]
fn falsy_water_fields_take_their_defaults() {
    for name in ["riverWidthZero", "riverWidthAbsent"] {
        let w = golden::WATERS.iter().find(|w| w.name == name).unwrap();
        let s = build_site(4, 1700.0, 1250.0, "river", SiteOpts {
            water: Some(water_of(w)),
            ..Default::default()
        });
        assert_eq!(s.river_w, 20.0, "{name}: riverWidthM did not fall back to 20");
    }
    let w = golden::WATERS.iter().find(|w| w.name == "orderNaN").unwrap();
    let s = build_site(4, 1700.0, 1250.0, "coast", SiteOpts {
        water: Some(water_of(w)),
        ..Default::default()
    });
    assert_eq!((s.water_order, s.sea_lake_cells), (0.0, 0.0));
    assert_eq!(js_or(f64::NAN, 7.0), 7.0);
    assert_eq!(js_or(-0.0, 7.0), 7.0);
    assert_eq!(js_or(0.5, 7.0), 0.5);
}

/// A river path of one point is truthy, so the site is river-like — but it is
/// too short to *be* the river, so the shoreline is traced from the mask
/// instead. Both halves matter: `rk` picks the four-endpoint route set and
/// suppresses `height`'s sea step, while the geometry comes from the mask.
#[test]
fn a_one_point_river_path_is_river_like_but_not_the_river() {
    for name in ["pathOfOne", "pathEmpty"] {
        let c = golden::SITES.iter().find(|c| c.name == name).unwrap();
        let s = build_site(c.seed, c.wm, c.hm, c.kind, opts_for(c));
        assert!(s.real_river, "{name}: not flagged realRiver");
        assert!(s.river_like(), "{name}: not river-like");
        assert!(s.river.len() > 2, "{name}: the short path became the river");
        assert_eq!(s.route_ends.len(), 4, "{name}: took the seaward route-end set");
    }
}

/// `economy` is carried through untouched; nothing in this milestone reads it.
#[test]
fn economy_passes_through() {
    let eco = Economy { specialisation: Some("oreyard".into()), ore_bearing: true };
    let s = build_site(4, 1700.0, 1250.0, "river", SiteOpts {
        economy: Some(eco.clone()),
        ..Default::default()
    });
    assert_eq!(s.economy.as_ref(), Some(&eco));
    // and the synthetic path leaves it None, which is what the reference's own
    // headless suite does
    let plain = build_site(4, 1700.0, 1250.0, "river", SiteOpts::default());
    assert!(plain.economy.is_none());
}
