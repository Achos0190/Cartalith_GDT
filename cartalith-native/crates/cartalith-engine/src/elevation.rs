//! **EF-0** — the elevation field as a queryable, deterministic,
//! multi-resolution primitive over a *generated world*
//! (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §5, EF-0).
//!
//! The design document's framing, which is what this module is for: not a
//! display trick bolted onto `WorldState.field`, but *"a queryable function:
//! coarse macro raster (tectonics, basins, mountains — what generation already
//! produces) plus deterministic procedural refinement at any requested
//! `(x, y, lod)`, seeded by world seed + tile coordinate + level so a re-query
//! is byte-identical."*
//!
//! # What was already here, and what this adds
//!
//! Most of EF-0 was already built and golden-tested, which is a finding worth
//! stating rather than working around:
//!
//! | EF-0 needs | already existed |
//! |---|---|
//! | `(z, col, row)` addressing, tile bounds, parent/child walk | [`cartalith_spatial::pyramid`] |
//! | detail synthesis that is not an upsample | [`cartalith_terrain::amplify::amplify_region`] / `refine_tile` |
//! | progressively finer octaves with depth | [`cartalith_terrain::amplify::add_zoom_detail`] |
//! | the three composed into one tile | [`crate::bake::pyramid_tile`] |
//!
//! Two things did not, and they are what this module and its
//! `cartalith-terrain` half add:
//!
//! 1. **A point query.** [`cartalith_terrain::amplify::sample_elevation`] —
//!    the refined height at one continuous coarse coordinate and one level,
//!    proven bit-identical to the tile path texel for texel. A tile is now one
//!    way of evaluating the field, not the only way.
//! 2. **A world-level entry point.** Everything above takes a loose
//!    `(&[f32], cw, ch, seed, sea, z_base)`, so every caller assembles the
//!    world's own numbers by hand and can assemble them wrong. [`world_amplify_opts`]
//!    derives them once, from the world, and [`world_elevation_tile`] /
//!    [`world_sample_elevation`] are the two queries over it.
//!
//! # The one phrase of EF-0's own text that must not be implemented literally
//!
//! §5 says the refinement is *"seeded by world seed + tile coordinate + level
//! so a re-query is byte-identical."* The goal is right and the recipe is not:
//! **seeding the noise by the tile coordinate breaks the seam.** Two tiles
//! that share an edge map that edge to the same coarse coordinate, and they
//! agree there because they draw from the same stream at it — give each tile
//! its own seed and the shared column is two different numbers.
//!
//! What actually holds, and what everything below is built on: **one stream,
//! seeded by the world seed alone, evaluated at the shared *coarse*
//! coordinate.** The level enters through the octave *count*
//! (`add_zoom_detail`'s `min(6, z − z_base)`), never through the seed, and
//! byte-identical re-query comes from the function being pure, not from
//! per-tile seeding.
//!
//! Measured rather than argued, because the failure mode is a quiet one: a
//! one-line mutant adding `id.col * 31 + id.row` to `opts.seed` in
//! [`world_elevation_tile`] leaves `the_same_chunk_id_always_produces_the_same_bytes`
//! **passing** and turns `adjacent_tiles_agree_bit_for_bit_on_their_shared_edge`
//! **red**. A determinism test cannot see this defect; only the seam test can.
//!
//! # Why `cartalith-engine`, and why the point query is not here
//!
//! `UNIFIED_TOOL_PLAN.md` milestone B's placement rule, the same one
//! [`cartalith_terrain::amplify`]'s own header applies to itself: subsystem-domain
//! math belongs to the crate that owns the field, and *"cartalith-engine
//! orchestrates; it does not compute."* The split falls out of that exactly:
//!
//! - the height formula — bilinear upsample, relief taper, sea fade, fBm
//!   octaves — is `cartalith-terrain`'s, and `sample_elevation` lives beside
//!   the two functions whose per-pixel bodies it reproduces;
//! - `WorldState`/`WorldParams` are this crate's, and nothing below computes a
//!   height: each function here reads the world, derives the options, and
//!   delegates.
//!
//! **Nothing here belongs in `cartalith-spatial`,** and the reason is already
//! written down: `pyramid`'s own header separates a *pyramid level* (the whole
//! field split `2^z × 2^z` ways, footprints shrinking with depth and generally
//! fractional) from fixed-size tiles over a field — *"Both are 'tiling'; only
//! one of them is this."* (`TiledField`/`QuadTree`, named below, were retired
//! from `cartalith-spatial` on 2026-09-22, never having had a caller.)
//! `LOD_TILING_INTEGRATION_SCOPE.md` says the same thing from the other end:
//! its tier table files `TiledField`/`QuadTree` under **Z3** (splitting the
//! base raster because holding it whole is the bottleneck), *"not triggered by
//! this port's real resolutions"*, while deep-zoom detail synthesis is tier
//! **Z2**. EF-0 is Z2. So this uses `pyramid`'s addressing and does not touch
//! `TiledField`/`QuadTree` — not an oversight, the documented boundary.
//!
//! **Said fully, because the same document cuts the other way once.** Its
//! milestone **M1** does name both for a Z2 job: *"using `TiledField` as the
//! synthesized-tile scratch buffer and `QuadTree::query_region` to resolve
//! which tiles the current viewport touches."* That is the **compositor** —
//! which tiles to ask for, and where to put them once they arrive — and the
//! compositor is the Godot-side follow-up this pass deliberately excludes (see
//! *Not in this pass*). It is not the per-tile synthesis primitive, which is
//! what EF-0 is and what everything below builds. So the boundary holds for
//! this module; it is not a claim that those two types have no Z2 role at all.
//!
//! # Not in this pass
//!
//! No Godot bridge and no `.gd`: `cartalith_godot::lod_bridge` already
//! synthesises interactive tiles through [`crate::bake::pyramid_tile`] and
//! would be the natural consumer of [`world_elevation_tile`], but wiring it is
//! a deliberate follow-up. Nothing in this module is called by shipping code
//! yet, exactly like `geojson` above it.
//!
//! EF-1 (tile-bounded hydrology refinement), EF-6 (vector features) and EF-7
//! (importance-driven refinement) are separate, later milestones in the same
//! document and are not started here.

use cartalith_spatial::pyramid::{pyramid_dims, ChunkId};
use cartalith_terrain::amplify::{sample_elevation, z_base_for_tile_size, AmplifyOpts};

use crate::bake::{pyramid_tile, PyramidTile};
use crate::{WorldParams, WorldState};

/// The smallest tile edge either query will accept.
///
/// Not a taste call: [`cartalith_terrain::amplify::amplify_region`] documents
/// a real `0/0` at `outW == 1` that turns the whole tile `NaN`, and
/// [`cartalith_spatial::tile_dims`] floors only the *derived* axis at 2 — the
/// axis given `tile_size` passes it straight through. One is therefore
/// reachable from a caller-supplied size, and is refused here rather than
/// returned as a plausible-looking tile of `NaN`.
const MIN_TILE_PX: usize = 2;

/// The deepest level addressable at all: [`cartalith_spatial::pyramid::pyramid_dims`]
/// clamps its shift at 31, so a `z` above that silently describes a different
/// (shallower) grid than the one its `col`/`row` were computed in.
const MAX_Z: u32 = 31;

/// The amplification options **this world** implies, for a tile `tile_size`
/// pixels across.
///
/// Public because it is the seam a caller with its own detail dials needs:
/// this fills in the three values that belong to the world and leaves the
/// rest at the reference's own defaults, so
/// `AmplifyOpts { detail_amp: my_amp, ..world_amplify_opts(..) }` keeps the
/// world-derived half and overrides only what the caller actually owns.
///
/// The three, and where each comes from:
///
/// - **`seed`** — `p.tect.seed`, the world seed every other stage is derived
///   from.
/// - **`sea`** — **`state.sea_level`, not `p.sea_level`.** `WorldState::sea_level`'s
///   own doc comment is explicit: it *"is equal to `p.sea_level` unless
///   `world_structure.enabled` re-anchored it... Callers that classify land vs.
///   ocean (a renderer, a land-fraction check) must use this, not `p.sea_level`
///   directly."* Both halves of the refinement classify land versus ocean —
///   `amplify_region`'s smooth `underwater` fade and `add_zoom_detail`'s hard
///   cut — so reading the parameter instead of the state would fade detail at
///   the wrong depth on every World-Structure world.
/// - **`z_base`** — [`z_base_for_tile_size`], so the octave schedule matches
///   the resolution actually being written. At `tile_size == 1024` this is the
///   reference's own `2`, i.e. the value every shipped bake already runs at.
///
/// `detail_freq`/`detail_amp`/`zoom_detail_k` are deliberately **not**
/// derived: they are user dials, `WorldParams` carries no equivalent of any of
/// them, and inventing a plausible world-shaped value for a number the world
/// does not have is the failure this project has catalogued most often.
///
/// **`ridged` is a different case and is left alone for a different reason**,
/// stated separately because the sentence above would be false of it:
/// `p.tect.ridged` exists, and the reference really does tie the two together
/// in one of its two callers. `regionNewWorldBtn` passes `{seed:
/// state.tect.seed, sea: state.seaLevel, ridged: state.tect.ridged}` into
/// `amplifyRegion` (reference line 13710, grepped this pass, not cited from
/// memory) — so on that path a world whose mountains were built from ridged
/// noise gets ridged detail. Its **other** caller does not: `lodTileOpts()`
/// (line 11059), the options every `pyramidTile` is synthesised with, sets
/// `seed`/`sea`/`detailAmp`/`detailFreq`/`zoomDetailK` and leaves `ridged`
/// unset, i.e. `false`.
///
/// [`world_elevation_tile`] composes `pyramidTile`, so `lodTileOpts` is the
/// caller it corresponds to, and this port's shipped tile path agrees:
/// `cartalith_godot::lod_bridge`'s tile synthesis builds
/// `AmplifyOpts { seed, sea, z_base, ..default() }` and no more. Deriving
/// `ridged` here would therefore make EF-0's tiles disagree with the tiles the
/// viewport already draws, on every ridged world — a change to generated
/// output, which needs an owner ruling rather than a tidy-up.
#[must_use]
pub fn world_amplify_opts(state: &WorldState, p: &WorldParams, tile_size: usize) -> AmplifyOpts {
    AmplifyOpts {
        seed: p.tect.seed,
        sea: state.sea_level,
        z_base: z_base_for_tile_size(tile_size),
        ..AmplifyOpts::default()
    }
}

/// **EF-0's tile query.** One pyramid tile's refined elevation, at that
/// level's own resolution, for this world.
///
/// Deterministic: a pure function of `(state.field, p.gw, p.gh, p.tect.seed,
/// state.sea_level, id, tile_size)` with no RNG state and no cache, so the
/// same world and the same [`ChunkId`] always produce the same bytes. Seam-
/// consistent with its four neighbours at the same level, because
/// [`cartalith_terrain::amplify::refine_tile`]'s sub-bounds overlap by exactly
/// one coarse column and every term of the synthesis is a pure function of the
/// shared coarse coordinate — asserted at `f32::to_bits` equality by this
/// module's own tests, over three seeds and both axes.
///
/// `None` rather than a panic for every reachable caller error, because the
/// only consumer worth having is a viewport and a panic there crosses the
/// gdext boundary and takes the process with it
/// (`cartalith-rust-conventions`). The refusals, each derived from a
/// precondition something downstream would otherwise assert on:
///
/// | refused | what would otherwise happen |
/// |---|---|
/// | `p.gw < 2` or `p.gh < 2` | a zero or negative pyramid step |
/// | `state.field` shorter than `p.gw * p.gh` | `amplify_region`'s own assert |
/// | `id.z > 31` | `pyramid_dims` clamps the shift, so `col`/`row` would index a different grid |
/// | `id.col`/`id.row` outside `2^z` | a tile that is not in the level |
/// | `tile_size < 2` | `amplify_region`'s documented `outW == 1` NaN tile |
///
/// Returns [`PyramidTile`] — the bake's own `{id, w, h, data}`, reused rather
/// than restated. A tile is a tile whether it is going to an atlas or to a
/// texture, and a second type of the same shape is how two of them drift.
#[must_use]
pub fn world_elevation_tile(
    state: &WorldState,
    p: &WorldParams,
    id: ChunkId,
    tile_size: usize,
) -> Option<PyramidTile> {
    let coarse = world_coarse(state, p, tile_size)?;
    if id.z > MAX_Z {
        return None;
    }
    let d = pyramid_dims(id.z as i32);
    if id.col >= d.cols || id.row >= d.rows {
        return None;
    }
    let opts = world_amplify_opts(state, p, tile_size);
    Some(pyramid_tile(coarse, p.gw, p.gh, id, tile_size, &opts))
}

/// **EF-0's point query** — `sample_elevation(x, y, lod)` in the design
/// document's own words, over a world rather than a loose field.
///
/// `cx`/`cy` are continuous **coarse sample** coordinates, the
/// `[0, gw−1] × [0, gh−1]` space [`cartalith_spatial::pyramid::pyramid_tile_bounds`]
/// returns; outside it the coarse sampler clamps at every edge. `z` is the
/// pyramid level, and `tile_size` is there for one reason only: it selects the
/// octave schedule (see [`z_base_for_tile_size`]), so this answers *"the
/// elevation at this point, as a tile of this size at this level resolves
/// it"*. Pass the same `tile_size` you would pass [`world_elevation_tile`] and
/// the two agree bit-for-bit, which this module's tests assert directly.
///
/// `None` on the same field/dimension refusals as [`world_elevation_tile`];
/// there is no tile index to be out of range, and no output grid to divide by,
/// so nothing else can fail.
#[must_use]
pub fn world_sample_elevation(
    state: &WorldState,
    p: &WorldParams,
    cx: f64,
    cy: f64,
    z: i32,
    tile_size: usize,
) -> Option<f32> {
    let coarse = world_coarse(state, p, tile_size)?;
    let opts = world_amplify_opts(state, p, tile_size);
    Some(sample_elevation(coarse, p.gw, p.gh, cx, cy, z, &opts))
}

/// The three refusals both queries share, in one place so they cannot drift
/// apart: a field big enough to index, a grid the pyramid step is positive
/// over, and a tile size that cannot produce the `outW == 1` NaN.
fn world_coarse<'a>(
    state: &'a WorldState,
    p: &WorldParams,
    tile_size: usize,
) -> Option<&'a [f32]> {
    if p.gw < 2 || p.gh < 2 || tile_size < MIN_TILE_PX {
        return None;
    }
    let need = p.gw.checked_mul(p.gh)?;
    if state.field.len() < need {
        return None;
    }
    Some(&state.field)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::pyramid::pyramid_tile_bounds;
    use cartalith_spatial::Region;
    use cartalith_terrain::amplify::refine_tile;

    /// The same synthetic field `bake.rs` and the amplify goldens build, in
    /// the same order: pure arithmetic (no `sin`/`cos`/`exp`) so nothing here
    /// depends on a libm, with a deliberately quantised `% 11` term.
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
                v += 0.10
                    * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    /// A `WorldState` carrying only what EF-0 reads — `field` and
    /// `sea_level`. Every other grid is empty on purpose: if a query here ever
    /// grows a dependency on one, this stops compiling rather than silently
    /// reading a zero. (The same construction, for the same reason, as
    /// `erode_op`'s own test world.)
    fn world(field: Vec<f32>, sea_level: f64) -> WorldState {
        WorldState {
            sea_level,
            field: std::sync::Arc::new(field),
            plate_id: Vec::new(),
            boundary_mask: Vec::new(),
            stress_field: Vec::new(),
            age_field: std::sync::Arc::new(Vec::new()),
            resistance_field: std::sync::Arc::new(Vec::new()),
            crust_field: std::sync::Arc::new(Vec::new()),
            boundary_type: Vec::new(),
            shear_field: Vec::new(),
            volcanic_field: std::sync::Arc::new(Vec::new()),
            impact_field: Vec::new(),
            temperature: std::sync::Arc::new(Vec::new()),
            rainfall: std::sync::Arc::new(Vec::new()),
            flow_discharge: std::sync::Arc::new(Vec::new()),
            channels: None,
            stream_order: None,
            river_mask: None,
            river_floor: None,
            gpu_stages_used: Vec::new(),
        }
    }

    const GW: usize = 48;
    const GH: usize = 32;
    const TS: usize = 32;

    /// One world per seed: both the coarse field and `p.tect.seed` move with
    /// it, so a seam that only held for one noise stream cannot pass.
    fn seeded(seed: i32) -> (WorldState, WorldParams) {
        let p = WorldParams::defaults(GW, GH, seed);
        (world(synthetic_field(GW, GH, seed as i64), 0.42), p)
    }

    // -- the wrapper adds nothing -------------------------------------------

    #[test]
    fn the_world_query_is_the_bake_composition_with_the_worlds_own_numbers() {
        let (ws, p) = seeded(4242);
        let id = ChunkId::new(3, 5, 2);
        let got = world_elevation_tile(&ws, &p, id, TS).expect("tile");
        let opts = world_amplify_opts(&ws, &p, TS);
        let want = pyramid_tile(&ws.field, GW, GH, id, TS, &opts);
        assert_eq!(got.id, want.id);
        assert_eq!((got.w, got.h), (want.w, want.h));
        assert!(got.w >= 2 && got.h >= 2, "degenerate tile {}x{}", got.w, got.h);
        assert_eq!(
            got.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            want.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_options_come_from_the_world_state_not_the_parameters() {
        // `apply_world_structure_sea_level` can re-anchor sea level away from
        // `p.sea_level`; a query that read the parameter would fade detail at
        // the wrong depth on every such world.
        let mut p = WorldParams::defaults(GW, GH, 7);
        p.sea_level = 0.42;
        let ws = world(synthetic_field(GW, GH, 7), 0.61);
        let o = world_amplify_opts(&ws, &p, 1024);
        // Literals throughout, never `o.sea == ws.sea_level` or `o.seed ==
        // p.tect.seed`: an assertion written against the very field it is
        // checking holds for every value of that field.
        assert_eq!(o.sea, 0.61, "sea must be the state's, not p.sea_level's 0.42");
        assert_eq!(o.seed, 7, "seed must be the world seed, not AmplifyOpts' own 1234");
        // 1024 px is the reference's own `_lodTile`, whose schedule is 2.
        assert_eq!(o.z_base, 2);
        assert_eq!(world_amplify_opts(&ws, &p, 256).z_base, 4);
    }

    // -- seam ---------------------------------------------------------------

    /// **The seam property**, at three seeds and two adjacent pairs each
    /// (one horizontal, one vertical), at exact `f32::to_bits` equality — the
    /// same shape `amplify::tests::adjacent_tiles_agree_exactly_on_their_shared_edge`
    /// uses for the region-based version, raised to the world-level query.
    ///
    /// A non-zero delta here reads as a hairline down every tile boundary in
    /// the viewport, which is the artifact the whole pyramid convention (the
    /// one-coarse-column overlap, the fractional step, the shared-coordinate
    /// sampling) exists to prevent.
    #[test]
    fn adjacent_tiles_agree_bit_for_bit_on_their_shared_edge() {
        let mut pairs_checked = 0usize;
        let mut cells_checked = 0usize;
        for seed in [1, 4242, -77_777] {
            let (ws, p) = seeded(seed);
            for z in [2u32, 4] {
                let n = pyramid_dims(z as i32).cols;
                let (col, row) = (n / 2, n / 3);
                // Horizontal: the left tile's last column is the right tile's
                // first column.
                let l = world_elevation_tile(&ws, &p, ChunkId::new(z, col, row), TS).expect("l");
                let r = world_elevation_tile(&ws, &p, ChunkId::new(z, col + 1, row), TS).expect("r");
                assert_eq!((l.w, l.h), (r.w, r.h));
                for y in 0..l.h {
                    assert_eq!(
                        l.data[y * l.w + (l.w - 1)].to_bits(),
                        r.data[y * r.w].to_bits(),
                        "seed {seed} z{z} horizontal seam at row {y}"
                    );
                    cells_checked += 1;
                }
                // Vertical: the top tile's last row is the bottom tile's first.
                let t = world_elevation_tile(&ws, &p, ChunkId::new(z, col, row), TS).expect("t");
                let b = world_elevation_tile(&ws, &p, ChunkId::new(z, col, row + 1), TS).expect("b");
                for x in 0..t.w {
                    assert_eq!(
                        t.data[(t.h - 1) * t.w + x].to_bits(),
                        b.data[x].to_bits(),
                        "seed {seed} z{z} vertical seam at col {x}"
                    );
                    cells_checked += 1;
                }
                pairs_checked += 2;
            }
        }
        // A silently-empty loop passes every assertion inside it.
        assert_eq!(pairs_checked, 3 * 2 * 2, "3 seeds x 2 levels x 2 pairs");
        assert!(cells_checked > 100, "only {cells_checked} seam cells compared");
    }

    /// The seam holds because the two tiles read the *same* coarse coordinate,
    /// not because both happen to be smooth there. Stated as its own check so
    /// a regression that made every tile constant could not pass the seam test
    /// silently.
    #[test]
    fn the_shared_edge_is_not_merely_a_flat_run() {
        let (ws, p) = seeded(4242);
        let l = world_elevation_tile(&ws, &p, ChunkId::new(4, 7, 5), TS).expect("l");
        let edge: Vec<u32> = (0..l.h).map(|y| l.data[y * l.w + (l.w - 1)].to_bits()).collect();
        let distinct = edge.iter().collect::<std::collections::BTreeSet<_>>().len();
        assert!(distinct > l.h / 2, "shared edge has only {distinct} distinct values of {}", l.h);
    }

    // -- determinism --------------------------------------------------------

    #[test]
    fn the_same_chunk_id_always_produces_the_same_bytes() {
        let (ws, p) = seeded(31337);
        let id = ChunkId::new(5, 11, 9);
        let a = world_elevation_tile(&ws, &p, id, TS).expect("a");
        let b = world_elevation_tile(&ws, &p, id, TS).expect("b");
        assert_eq!(
            a.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            b.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        // And across a freshly-built, equal world -- so the determinism is of
        // the inputs, not of one `Vec` staying at one address.
        let (ws2, p2) = seeded(31337);
        let c = world_elevation_tile(&ws2, &p2, id, TS).expect("c");
        assert_eq!(
            a.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            c.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        // Different world, same id: the bytes must move, or "deterministic"
        // would be satisfied by returning a constant.
        let (ws3, p3) = seeded(31338);
        let d = world_elevation_tile(&ws3, &p3, id, TS).expect("d");
        assert_ne!(a.data, d.data);
    }

    #[test]
    fn the_point_query_and_the_tile_query_are_the_same_field() {
        let (ws, p) = seeded(4242);
        let z = 6u32;
        let id = ChunkId::new(z, 40, 33);
        let t = world_elevation_tile(&ws, &p, id, TS).expect("tile");
        let b = pyramid_tile_bounds(GW, GH, z as i32, id.col, id.row);
        for (ox, oy) in [(0usize, 0usize), (1, 3), (t.w / 2, t.h / 2), (t.w - 1, t.h - 1)] {
            let cx = b.x + ox as f64 / (t.w as f64 - 1.0) * b.w;
            let cy = b.y + oy as f64 / (t.h as f64 - 1.0) * b.h;
            let q = world_sample_elevation(&ws, &p, cx, cy, z as i32, TS).expect("point");
            assert_eq!(q.to_bits(), t.data[oy * t.w + ox].to_bits(), "texel ({ox},{oy})");
        }
    }

    // -- not an upsample ----------------------------------------------------

    /// Sum of squared discrete Laplacians over the interior: how much
    /// curvature — high-frequency content — a patch carries. Bilinear
    /// interpolation is linear along both axes inside one coarse cell, so an
    /// upsample of coarse data scores near zero on it by construction, which
    /// is exactly why it is the right measure here.
    fn curvature_energy(v: &[f32], w: usize, h: usize) -> f64 {
        let mut e = 0.0;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let c = v[y * w + x] as f64;
                let lx = v[y * w + x - 1] as f64 - 2.0 * c + v[y * w + x + 1] as f64;
                let ly = v[(y - 1) * w + x] as f64 - 2.0 * c + v[(y + 1) * w + x] as f64;
                e += lx * lx + ly * ly;
            }
        }
        e
    }

    /// **The "real new information, not smoothing" proof, at tile level.**
    ///
    /// The control is *the same tile*, over the same footprint, at the same
    /// output resolution, with the detail term switched off — which is a plain
    /// bilinear upsample of the coarse field and nothing else, reusing
    /// `refine_tile` rather than reimplementing bilinear so the two can differ
    /// only by the detail. That is the same construction
    /// `cartalith_godot::lod_bridge::synthesize_tile_rgba` already uses for
    /// its "plain half", for the same reason.
    ///
    /// Three seeds, because one measurement is one sample.
    #[test]
    fn a_deep_tile_is_not_a_bilinear_upsample_of_the_coarse_tile() {
        let z = 8u32;
        // Over the dome's flank: land, and sloped, so neither
        // `add_zoom_detail`'s hard sea cut nor its `relief <= 0` skip is what
        // is being measured.
        let id = ChunkId::new(z, 147, 145);
        // A real screen tile, not the 32 px one the structural tests use for
        // speed: `z_base_for_tile_size` gives a 32 px tile only
        // `min(6, 8 − 7) = 1` extra octave at this level, which is correct for
        // that size and not what a viewport asks for. At 256 px the schedule
        // is `min(6, 8 − 4) = 4` octaves.
        const DEEP_TS: usize = 256;
        for seed in [1, 4242, -77_777] {
            let (ws, p) = seeded(seed);
            let t = world_elevation_tile(&ws, &p, id, DEEP_TS).expect("tile");
            let n = pyramid_dims(z as i32).cols as usize;
            let region = Region { x: 0, y: 0, w: GW - 1, h: GH - 1 }.to_float();
            let plain_opts =
                AmplifyOpts { detail_amp: 0.0, ..world_amplify_opts(&ws, &p, DEEP_TS) };
            let plain = refine_tile(
                &ws.field, GW, GH, &region, n, n, id.col as usize, id.row as usize, t.w, t.h,
                &plain_opts,
            );
            assert_eq!(plain.len(), t.data.len());
            assert!(plain[0] > ws.sea_level as f32, "seed {seed}: tile is underwater");
            let e_ref = curvature_energy(&t.data, t.w, t.h);
            let e_up = curvature_energy(&plain, t.w, t.h);
            let max_dev = t
                .data
                .iter()
                .zip(&plain)
                .map(|(a, b)| (*a as f64 - *b as f64).abs())
                .fold(0.0f64, f64::max);
            println!(
                "seed {seed}: curvature refined {e_ref:.4e} upsampled {e_up:.4e} \
                 ratio {:.0}x, max |refined-upsampled| {max_dev:.5}",
                e_ref / e_up
            );
            // Measured, this run, the three seeds in order:
            //
            //   seed      1  refined 9.37e-5  upsampled 4.29e-9  ratio 21840x
            //   seed   4242  refined 4.91e-5  upsampled 1.80e-9  ratio 27314x
            //   seed -77777  refined 4.64e-5  upsampled 2.13e-7  ratio   218x
            //
            // with max deviations of 0.0219, 0.0104 and 0.0140. The spread in
            // the ratio is the honest shape of it, not noise: a footprint that
            // crosses a coarse cell boundary gets a real crease from the
            // bilinear upsample (that crease **is** the blockiness), one that
            // does not is planar to f64 rounding. The bars clear the weaker of
            // each by roughly 2x.
            assert!(
                e_ref > 100.0 * e_up,
                "seed {seed}: refinement carried only {:.1}x the curvature of the upsample",
                e_ref / e_up
            );
            assert!(max_dev > 0.005, "seed {seed}: refinement moved at most {max_dev:.6}");
        }
    }

    /// **The same proof, swept instead of sampled once** — the test above
    /// measures one depth; this follows one piece of ground down eleven of
    /// them, which is the shape `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §4
    /// states its own acceptance bar in: *"one continuous zoom, three seeds,
    /// sampled at a dense sequence of depths, zero flat/blocky runs at any
    /// depth."*
    ///
    /// Three measures are asserted; a fourth is only printed, because
    /// asserting it would pin the wrong thing:
    ///
    /// - **the ratio against the upsample** never falls below `5x`, and the
    ///   weakest depth measured is `11.9x` — weakest at the *shallow* end, not
    ///   the deep one, because a shallow tile spans many coarse cells and the
    ///   bilinear upsample still has real creases of its own to carry;
    /// - **`max |refined − upsampled|`** stays above `0.003` at every depth:
    ///   at z12 the three seeds still move the surface by `0.008`–`0.017` of
    ///   full height range, so refinement is not converging onto the upsample;
    /// - **the tile's own peak-to-peak** stays above `0.001` at every depth,
    ///   which is the *"zero flat runs"* half of §4 said as a number;
    /// - **curvature per texel**, which is printed and asserted only through
    ///   the ratio, because it falls with depth and *should*. Each level
    ///   halves the ground a tile covers while the tile keeps its pixel count,
    ///   so every octave already present doubles its wavelength in texels, and
    ///   a deeper tile covering less ground per texel reads smoother per
    ///   texel. The claim worth making is the one against what the viewer
    ///   would otherwise be shown at that same depth — the ratio — not an
    ///   absolute roughness.
    ///
    /// # The measured floor of "more detail on zoom", which is not infinite
    ///
    /// Numbers from this test's own printout, `--nocapture`, this run. At a
    /// 256 px tile ([`z_base_for_tile_size`] gives `z_base = 4`), seed 1:
    ///
    /// ```text
    ///   z    ratio     refined     upsampled    p2p      max dev
    ///   8    2.16e4    9.26e-5     4.29e-9      0.0149   0.0173
    ///   9    3.37e4    3.50e-5     1.04e-9      0.0130   0.0175
    ///  10    2.84e4    1.22e-5     4.28e-10     0.0072   0.0177
    ///  11    3.31e3    9.34e-7     2.82e-10     0.0039   0.0177
    ///  12    4.27e2    6.11e-8     1.43e-10     0.0032   0.0174
    /// ```
    ///
    /// Level to level, that curvature column falls by **2.4–2.9x** from z4 to
    /// z10 and then by **13x** at z10→z11 and **15x** at z11→z12. Two regimes,
    /// and the boundary between them is `add_zoom_detail`'s octave count
    /// `min(6, z − z_base)`:
    ///
    /// - **while an octave is still being added** (`z ≤ z_base + 6`), the new
    ///   octave's wavelength *in texels* is constant — frequency doubles as the
    ///   footprint halves — and only its amplitude falls, by `0.6` per octave.
    ///   `0.6² ≈ 0.36`, i.e. `2.8x` less curvature energy per level, which is
    ///   the 2.4–2.9x measured. Structure at texel scale is still arriving;
    ///   each layer is simply lower-contrast than the last, which is fBm's own
    ///   spectrum rather than a defect.
    /// - **once the count saturates** (`z > z_base + 6`) the field is fixed in
    ///   coarse space and only the sampling keeps refining, so a discrete
    ///   Laplacian falls as `h²` and its energy as `h⁴` — `16x` per level,
    ///   against 13x and 15x measured. No new detail; the same six octaves
    ///   stretched.
    ///
    /// **The cause is measured, not inferred: the knee follows `z_base`.** Run
    /// this same sweep with `SWEEP_TS = 128` (so `z_base_for_tile_size` gives
    /// 5 instead of 4) and the 2.8x band extends one level further — z10→z11
    /// measures 2.85x instead of 13x — while the 13.7x knee appears at
    /// z11→z12. One level of `z_base`, one level of knee.
    ///
    /// So the honest reading of §4's *"fbm/ridged have no inherent resolution
    /// floor"* is: the **noise** has none, and the **ported schedule** imposes
    /// one anyway, at `z_base + 6`. What is drawn past it is still real
    /// synthesized relief rather than an upsample — the ratio is still `10²`
    /// to `10⁴` and the surface still moves by more than `0.003` — but it is
    /// the same six octaves stretched, not new information. Raising the cap is
    /// a change to a ported reference constant and therefore a golden
    /// re-baseline, not a tuning knob; EF-2/EF-3 are where the document puts
    /// that question.
    #[test]
    fn a_continuous_zoom_keeps_adding_structure_at_every_depth() {
        const SWEEP_TS: usize = 256;
        // One piece of ground, land and sloped on the dome's flank -- the same
        // ground the single-depth test above measures, followed down.
        let (gx, gy) = (27.0f64, 17.55f64);
        let mut depths_checked = 0usize;
        for seed in [1, 4242, -77_777] {
            let (ws, p) = seeded(seed);
            let region = Region { x: 0, y: 0, w: GW - 1, h: GH - 1 }.to_float();
            let plain_opts =
                AmplifyOpts { detail_amp: 0.0, ..world_amplify_opts(&ws, &p, SWEEP_TS) };
            for z in 2u32..=12 {
                let n = pyramid_dims(z as i32).cols;
                let col = ((gx * n as f64 / (GW as f64 - 1.0)) as u32).min(n - 1);
                let row = ((gy * n as f64 / (GH as f64 - 1.0)) as u32).min(n - 1);
                let t = world_elevation_tile(&ws, &p, ChunkId::new(z, col, row), SWEEP_TS)
                    .expect("tile");
                let plain = refine_tile(
                    &ws.field, GW, GH, &region, n as usize, n as usize, col as usize,
                    row as usize, t.w, t.h, &plain_opts,
                );
                let e_ref = curvature_energy(&t.data, t.w, t.h);
                let e_up = curvature_energy(&plain, t.w, t.h);
                let (lo, hi) = t
                    .data
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
                let max_dev = t
                    .data
                    .iter()
                    .zip(&plain)
                    .map(|(a, b)| (*a as f64 - *b as f64).abs())
                    .fold(0.0f64, f64::max);
                println!(
                    "seed {seed:>7} z{z:<3} ratio {:>10.3e}  refined {e_ref:.3e}  up {e_up:.3e}  \
                     p2p {:.5}  dev {max_dev:.5}",
                    e_ref / e_up,
                    hi - lo
                );
                // Weakest measured, over the whole sweep: ratio 11.86 (seed
                // 4242, z2), dev 0.00763 (seed 4242, z12), p2p 0.00318 (seed
                // 1, z12). Each bar clears its own weakest depth by 2.4x or
                // better, so a regression trips it before one run's noise
                // does.
                assert!(
                    e_ref > 5.0 * e_up,
                    "seed {seed} z{z}: only {:.2}x the curvature of the upsample",
                    e_ref / e_up
                );
                assert!(max_dev > 0.003, "seed {seed} z{z}: refinement moved at most {max_dev:.6}");
                assert!(hi - lo > 0.001, "seed {seed} z{z}: tile is a flat run, p2p {:.6}", hi - lo);
                depths_checked += 1;
            }
        }
        // A loop that never ran passes every assertion inside it.
        assert_eq!(depths_checked, 3 * 11, "3 seeds x z2..=z12");
    }

    // -- refusals -----------------------------------------------------------

    #[test]
    fn every_reachable_caller_error_is_a_none_rather_than_a_panic() {
        let (ws, p) = seeded(9);
        let ok = ChunkId::new(3, 0, 0);
        assert!(world_elevation_tile(&ws, &p, ok, TS).is_some(), "the control must pass");

        // A grid the pyramid step is not positive over.
        let mut tiny = WorldParams::defaults(GW, GH, 9);
        tiny.gw = 1;
        assert!(world_elevation_tile(&ws, &tiny, ok, TS).is_none(), "gw < 2");
        let mut tiny_h = WorldParams::defaults(GW, GH, 9);
        tiny_h.gh = 1;
        assert!(world_elevation_tile(&ws, &tiny_h, ok, TS).is_none(), "gh < 2");

        // A field shorter than the grid it claims -- `amplify_region` asserts.
        let short = world(vec![0.5f32; GW * GH - 1], 0.42);
        assert!(world_elevation_tile(&short, &p, ok, TS).is_none(), "short field");

        // A tile index outside the level's own 2^z grid.
        assert!(world_elevation_tile(&ws, &p, ChunkId::new(3, 8, 0), TS).is_none(), "col == 2^z");
        assert!(world_elevation_tile(&ws, &p, ChunkId::new(3, 0, 8), TS).is_none(), "row == 2^z");

        // A level past the addressable depth.
        assert!(world_elevation_tile(&ws, &p, ChunkId::new(32, 0, 0), TS).is_none(), "z > 31");

        // A tile size that would reach the documented `outW == 1` NaN.
        assert!(world_elevation_tile(&ws, &p, ok, 1).is_none(), "tile_size 1");
        assert!(world_elevation_tile(&ws, &p, ok, 0).is_none(), "tile_size 0");

        // The point query shares the field/grid refusals and has no others.
        assert!(world_sample_elevation(&ws, &tiny, 1.0, 1.0, 4, TS).is_none());
        assert!(world_sample_elevation(&short, &p, 1.0, 1.0, 4, TS).is_none());
        assert!(world_sample_elevation(&ws, &p, 1.0, 1.0, 4, TS).is_some());
        // Far outside the field: the coarse sampler clamps, it does not fail.
        assert!(world_sample_elevation(&ws, &p, -9e3, 9e3, 4, TS).is_some_and(|v| v.is_finite()));
    }
}
