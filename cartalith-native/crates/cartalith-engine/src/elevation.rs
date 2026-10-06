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
//! EF-1 (tile-bounded hydrology refinement), EF-6 (vector features) and EF-9
//! (importance-driven refinement) are separate, later milestones in the same
//! document and are not started here.

use cartalith_spatial::pyramid::{pyramid_dims, pyramid_tile_bounds, ChunkId};
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

/// **EF-9.3's morph band**, in texels of the tile being conformed: how far from
/// a coarser neighbour the finer tile's field is pulled toward that
/// neighbour's.
///
/// A labelled judgement bounded by two measured numbers, not a fitted value
/// (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.4, EF-9.3). *Lower bound:* a
/// smoothstep over `B` texels adds at most `1.5 * delta / B` to the per-texel
/// step, and the shared-edge mismatch `delta` measured on six worlds is up to
/// `3.2` natural texel steps per edge (mean over an edge, worst edge), so
/// keeping the added step under half a natural step (`lod_sweep::seam_ratio`'s
/// 1.5 bar, by the triangle inequality) needs `B >= 3 * 3.2 ~= 9.6`. *Upper
/// bound:* a wider band discards more of the finest octave next to every
/// coarse neighbour, and `256 / 16 = 16` texels is a sixteenth of a tile -- the
/// strip a viewer sees as "slightly smoother beside the coarser tile" rather
/// than as a missing tile edge. `16` is the smallest power of two above the
/// lower bound.
pub const CONFORM_BAND_TEXELS: f64 = 16.0;

/// **EF-9.3's blend weight.** `1.0` on the shared edge (`dist_texels == 0`),
/// `0.0` at and beyond `band_texels`, the cubic smoothstep between them (zero
/// slope at both ends, so the pull neither creases at the band's inner edge nor
/// leaves a ridge at the seam).
///
/// Total and NaN-safe in the direction that keeps today's output: a
/// non-positive or NaN band, a NaN distance, or a distance at or past the band
/// all return `0.0` (no pull). Exactly `1.0` only at distance `<= 0`, so the
/// caller's "weight 1 means the coarser field, bit for bit" branch is reachable
/// only on the edge itself. Never returns outside `[0, 1]`.
#[must_use]
pub fn conform_weight(dist_texels: f64, band_texels: f64) -> f64 {
    // `!(x > 0.0)` rather than `x <= 0.0` so a NaN band falls out as "no pull".
    if !(band_texels > 0.0) {
        return 0.0;
    }
    let d = dist_texels / band_texels;
    if !(d < 1.0) {
        return 0.0;
    }
    if d <= 0.0 {
        return 1.0;
    }
    let s = 1.0 - d;
    s * s * (3.0 - 2.0 * s)
}

/// Distance, in texels of the tile being conformed, from the point `(cx, cy)`
/// to the axis-aligned rectangle `r` (zero inside or on it). `sx`/`sy` are that
/// tile's texels per coarse-coordinate unit on each axis. `sqrt`, not `hypot`:
/// `sqrt` is correctly rounded everywhere, so two tiles evaluating the same
/// shared point agree to the bit on every platform.
fn rect_distance_texels(cx: f64, cy: f64, r: &cartalith_spatial::FloatRegion, sx: f64, sy: f64) -> f64 {
    let dx = f64::max(f64::max(r.x - cx, cx - (r.x + r.w)), 0.0) * sx;
    let dy = f64::max(f64::max(r.y - cy, cy - (r.y + r.h)), 0.0) * sy;
    (dx * dx + dy * dy).sqrt()
}

/// **EF-9.3 -- a tile that agrees with its coarser neighbours.** The same tile
/// as [`world_elevation_tile`], except that near every tile in `coarser` its
/// field is pulled toward the coarser level's own field
/// (CDLOD-style geomorphing, but by *position*, not by time), so the shared
/// edge carries the coarser neighbour's texel values bit for bit.
///
/// # Why
///
/// EF-0's seam holds only between same-level neighbours. A level-`z` tile and a
/// level-`z - 1` tile run different octave counts, so their shared edge is two
/// different fields; EF-9.2's selection makes that the normal case (a promoted
/// tile's four children sit beside plain `z0` tiles). Measured on six worlds
/// (EF-9.3), the disagreement averages about 1.2 natural texel steps along an
/// edge, past the half-step bound `lod_sweep::seam_ratio <= 1.5` implies.
/// LOD-D3's morph cannot repair it: that fade is one scalar per *level*
/// (`lod_bridge::morph_for_zoom`), a temporal blend of a level over its parent,
/// and never reconciles two live tiles of different levels side by side.
///
/// # The rule
///
/// `out(p) = F_z(p) * (1 - m) + F_{z-1}(p) * m` with
/// `m(p) = max over T in coarser of conform_weight(dist(p, T), band)`. `F_k`
/// is the level-`k` field ([`sample_elevation`] at level `k`, which EF-0 pins
/// bit-identical to a tile texel). Consequences, each asserted by a test:
///
/// * `m == 0` returns `F_z(p)` untouched, so a texel farther than the band from
///   every coarser tile is **bit-identical** to [`world_elevation_tile`]'s, and
///   an empty `coarser` returns that tile outright (identity by control flow).
/// * `m == 1` returns `F_{z-1}(p)` exactly; on the shared edge this is the
///   value the coarser tile holds at the coincident texel.
/// * `m` is a function of the world position alone, never of which edge
///   brought a coarser tile into the list, so two *same-level* neighbours that
///   both border the same coarser tile (or its corner) still agree bit for bit
///   on their own shared edge. Callers must therefore pass **every** coarser
///   tile within a band of this tile, diagonal neighbours included; passing
///   more is harmless (a far tile contributes `0`).
///
/// # What it deliberately does not do
///
/// It does not make the in-between fine texels equal the coarse tile's
/// *interpolated* edge: the coarse tile has half as many texels, and a
/// bilinear sampler draws a straight line between them, while this returns
/// `F_{z-1}` at the half position. The remaining gap is the coarse field's own
/// curvature over one coarse texel -- measured by EF-9.3's test.
///
/// `None` on every refusal [`world_elevation_tile`] has, and also when
/// `coarser` is non-empty and `id.z == 0`, or any entry is not at level
/// `id.z - 1` or lies outside that level. An entry that is not actually near
/// this tile is not an error.
#[must_use]
pub fn world_elevation_tile_conformed(
    state: &WorldState,
    p: &WorldParams,
    id: ChunkId,
    tile_size: usize,
    coarser: &[ChunkId],
) -> Option<PyramidTile> {
    conformed_with_band(state, p, id, tile_size, coarser, CONFORM_BAND_TEXELS)
}

/// [`world_elevation_tile_conformed`] with an explicit band, so tests can
/// exercise bands other than the shipped [`CONFORM_BAND_TEXELS`] (a mutant of
/// the constant is otherwise invisible to a test that only uses the default).
fn conformed_with_band(
    state: &WorldState,
    p: &WorldParams,
    id: ChunkId,
    tile_size: usize,
    coarser: &[ChunkId],
    band_texels: f64,
) -> Option<PyramidTile> {
    let mut tile = world_elevation_tile(state, p, id, tile_size)?;
    if coarser.is_empty() {
        return Some(tile);
    }
    let zc = id.z.checked_sub(1)?;
    let dc = pyramid_dims(zc as i32);
    if coarser.iter().any(|c| c.z != zc || c.col >= dc.cols || c.row >= dc.rows) {
        return None;
    }
    let field = world_coarse(state, p, tile_size)?;
    let opts = world_amplify_opts(state, p, tile_size);
    let rects: Vec<cartalith_spatial::FloatRegion> = coarser
        .iter()
        .map(|c| pyramid_tile_bounds(p.gw, p.gh, zc as i32, c.col, c.row))
        .collect();
    let b = pyramid_tile_bounds(p.gw, p.gh, id.z as i32, id.col, id.row);
    let (w, h) = (tile.w, tile.h);
    // Texels per coordinate unit on each axis; `w, h >= 2` (MIN_TILE_PX, and
    // `tile_dims` floors the derived axis at 2), so no divide by zero.
    let sx = (w as f64 - 1.0) / b.w;
    let sy = (h as f64 - 1.0) / b.h;
    for j in 0..h {
        // The same coordinate expression `amplify_region` uses, so a texel's
        // position here is bit-identical to the one its tile was built at.
        let cy = b.y + (j as f64 / (h as f64 - 1.0)) * b.h;
        for i in 0..w {
            let cx = b.x + (i as f64 / (w as f64 - 1.0)) * b.w;
            let mut dist = f64::INFINITY;
            for r in &rects {
                dist = f64::min(dist, rect_distance_texels(cx, cy, r, sx, sy));
            }
            let m = conform_weight(dist, band_texels);
            if m <= 0.0 {
                continue;
            }
            let c = sample_elevation(field, p.gw, p.gh, cx, cy, zc as i32, &opts);
            let at = j * w + i;
            tile.data[at] = if m >= 1.0 {
                c
            } else {
                (f64::from(tile.data[at]) * (1.0 - m) + f64::from(c) * m) as f32
            };
        }
    }
    Some(tile)
}

/// Sum of squared discrete Laplacians over the interior of a `w x h` patch:
/// how much curvature -- high-frequency content -- the patch carries.
///
/// Bilinear interpolation is linear along both axes inside one coarse cell,
/// so an upsample of coarse data scores near zero on it by construction,
/// which is exactly why it is the right measure for EF-0's "real new
/// information, not smoothing" bar (the `a_deep_tile_is_not_a_bilinear_upsample_of_the_coarse_tile`
/// and `a_continuous_zoom_keeps_adding_structure_at_every_depth` tests).
///
/// Public since EF-9.0 (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.3):
/// that harness scores refinement gain in A1's own quantity, so it shares
/// this one definition rather than restating it. The body is the tests'
/// private helper moved verbatim; no behaviour changed.
///
/// Requires `w >= 2` and `h >= 2` and `v.len() >= w * h`; a patch with no
/// interior (`w < 3` or `h < 3`) sums over nothing and returns `0.0`.
/// Must never be read as a roughness in absolute terms: it depends on the
/// tile's pixel size, so only ratios between patches of one size are
/// meaningful.
#[must_use]
pub fn curvature_energy(v: &[f32], w: usize, h: usize) -> f64 {
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

/// Coverage for EF-0: that the world-level wrapper adds nothing of its own
/// beyond deriving options, the seam property across tile boundaries,
/// determinism, tile/point-query agreement, the "real detail, not an
/// upsample" measurement, and every reachable caller-error refusal.
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
            integrated_drainage: false,
            channels: None,
            stream_order: None,
            river_mask: None,
            river_floor: None,
            gpu_stages_used: Vec::new(),
            geology: crate::Geology::Absent(crate::GeologyAbsent::ModelOff),
        }
    }

    // Fixture grid and tile size shared by every structural test below.
    // Small enough that a whole level's tiles are cheap; large enough
    // (>= MIN_TILE_PX by a wide margin) that the refinement has real coarse
    // cells to interpolate between.
    const GW: usize = 48; // fixture grid width, see the comment above
    const GH: usize = 32; // fixture grid height, see the comment above
    const TS: usize = 32; // fixture tile size, see the comment above

    /// One world per seed: both the coarse field and `p.tect.seed` move with
    /// it, so a seam that only held for one noise stream cannot pass.
    fn seeded(seed: i32) -> (WorldState, WorldParams) {
        let p = WorldParams::defaults(GW, GH, seed);
        (world(synthetic_field(GW, GH, seed as i64), 0.42), p)
    }

    // -- the wrapper adds nothing -------------------------------------------

    /// `world_elevation_tile` adds no numeric drift over calling
    /// `pyramid_tile` directly with `world_amplify_opts`'s own output.
    #[test]
    fn the_world_query_is_the_bake_composition_with_the_worlds_own_numbers() {
        // Protects: world_elevation_tile against calling pyramid_tile
        // directly with world_amplify_opts's own output -- the wrapper must
        // add no numeric drift of its own, bit for bit.
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

    /// `world_amplify_opts` reads `state.sea_level`/`p.tect.seed`, never
    /// `p.sea_level` or `AmplifyOpts`'s own default seed.
    #[test]
    fn the_options_come_from_the_world_state_not_the_parameters() {
        // Protects: `world_amplify_opts` reading `state.sea_level` and
        // `p.tect.seed`, not `p.sea_level` or `AmplifyOpts`'s own default
        // seed, and z_base tracking the reference's own tile-size schedule.
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
        // Protects: the seam property at three seeds and both axes, at exact
        // f32::to_bits equality -- a mutant that seeds the noise by tile
        // coordinate instead of the world seed alone passes determinism but
        // fails exactly this.
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
        // Protects: the seam test above against a regression that made
        // every tile constant, which would pass a bit-for-bit comparison
        // vacuously.
        let (ws, p) = seeded(4242);
        let l = world_elevation_tile(&ws, &p, ChunkId::new(4, 7, 5), TS).expect("l");
        let edge: Vec<u32> = (0..l.h).map(|y| l.data[y * l.w + (l.w - 1)].to_bits()).collect();
        let distinct = edge.iter().collect::<std::collections::BTreeSet<_>>().len();
        assert!(distinct > l.h / 2, "shared edge has only {distinct} distinct values of {}", l.h);
    }

    // -- EF-9.3: mixed-level shared edges -----------------------------------

    /// Tile size at which the octave schedule is live from level 4
    /// (`z_base_for_tile_size(256) == 4`): the shared `TS = 32` fixture sits at
    /// `z_base` 7, where levels 2..=6 add no detail at all and a mixed-edge test
    /// would compare two identical fields.
    const MIX_TS: usize = 256;

    /// `start + (k / (n - 1)) * span`, the coordinate expression the tile path
    /// uses, so a position here is bit-identical to the tile's own.
    fn tcoord(start: f64, k: usize, n: usize, span: f64) -> f64 {
        start + (k as f64 / (n as f64 - 1.0)) * span
    }

    /// One edge of `t` as `(world coordinate along the edge, value)` pairs.
    /// `column == true`: the tile's first (`last == false`) or last column,
    /// listed by row (the shared line is vertical); otherwise a row, by column.
    fn edge_values(t: &PyramidTile, column: bool, last: bool) -> Vec<(f64, f32)> {
        let b = pyramid_tile_bounds(GW, GH, t.id.z as i32, t.id.col, t.id.row);
        if column {
            let i = if last { t.w - 1 } else { 0 };
            (0..t.h).map(|j| (tcoord(b.y, j, t.h, b.h), t.data[j * t.w + i])).collect()
        } else {
            let j = if last { t.h - 1 } else { 0 };
            (0..t.w).map(|i| (tcoord(b.x, i, t.w, b.w), t.data[j * t.w + i])).collect()
        }
    }

    /// `(positions present in both edges, of which unequal at f32::to_bits)`.
    /// Positions are matched by exact `f64` bit pattern: a coarse tile has about
    /// half the texels of its finer neighbour, so only coincident positions are
    /// comparable, and EF-0's dyadic bounds make them bit-identical.
    fn compare_edges(a: &[(f64, f32)], b: &[(f64, f32)]) -> (usize, usize) {
        let (mut compared, mut unequal) = (0, 0);
        for (pa, va) in a {
            if let Some((_, vb)) = b.iter().find(|(pb, _)| pb.to_bits() == pa.to_bits()) {
                compared += 1;
                if va.to_bits() != vb.to_bits() {
                    unequal += 1;
                }
            }
        }
        (compared, unequal)
    }

    /// The weight is a cubic smoothstep from 1 on the edge to 0 at the band.
    #[test]
    fn the_conform_weight_is_a_smoothstep_from_one_at_the_edge_to_zero_at_the_band() {
        // Protects: the blend profile's three anchors (1 at distance 0, 1/2 at
        // mid-band, 0 at the band), its exact interior values (a linear ramp or
        // a different cubic fails), its monotonicity, and the total/NaN-safe
        // "no pull" answers for a degenerate band or distance. Expected values
        // are the closed form x^2 (3 - 2x) at x = 1 - d/B, written out as
        // literals, not recomputed by the code under test.
        assert_eq!(conform_weight(0.0, 16.0), 1.0);
        assert_eq!(conform_weight(-3.0, 16.0), 1.0, "a point on the far side is on the edge");
        assert_eq!(conform_weight(16.0, 16.0), 0.0);
        assert_eq!(conform_weight(40.0, 16.0), 0.0);
        assert_eq!(conform_weight(8.0, 16.0), 0.5);
        assert_eq!(conform_weight(4.0, 16.0), 0.84375);
        assert_eq!(conform_weight(12.0, 16.0), 0.15625);
        let mut prev = 1.0;
        for k in 1..=160 {
            let w = conform_weight(f64::from(k) * 0.1, 16.0);
            assert!((0.0..=1.0).contains(&w) && w <= prev, "not monotone at {k}: {w} after {prev}");
            prev = w;
        }
        for (d, band) in [(1.0, 0.0), (1.0, -4.0), (1.0, f64::NAN), (f64::NAN, 16.0)] {
            assert_eq!(conform_weight(d, band), 0.0, "d {d}, band {band}");
        }
    }

    /// **The EF-9.3 property**: a finer tile conformed to its coarser neighbour
    /// carries that neighbour's texel values on the shared edge, bit for bit,
    /// where the plain tiles do not.
    #[test]
    fn a_conformed_tile_agrees_bit_for_bit_with_its_coarser_neighbour() {
        // Protects: the mixed-level seam at three seeds, three coarse levels and
        // all four sides (both axes, the fine tile after and before the coarse
        // one), at exact f32::to_bits equality over every coincident texel;
        // and that the plain tiles really do disagree there, so a regression
        // that made the two fields equal everywhere (detail off, flat world)
        // cannot pass this vacuously.
        let mut cases = 0usize;
        for seed in [1, 4242, -77_777] {
            let (ws, p) = seeded(seed);
            for zc in [4u32, 5, 6] {
                let s = 1u32 << (zc - 4);
                let (cc, rc) = (6 * s, 8 * s);
                let coarse_id = ChunkId::new(zc, cc, rc);
                let coarse = world_elevation_tile(&ws, &p, coarse_id, MIX_TS).expect("coarse");
                // (fine tile, shared line is vertical, fine tile is east/south of the coarse one)
                let sides = [
                    (ChunkId::new(zc + 1, 2 * cc + 2, 2 * rc), true, true),
                    (ChunkId::new(zc + 1, 2 * cc + 2, 2 * rc + 1), true, true),
                    (ChunkId::new(zc + 1, 2 * cc - 1, 2 * rc), true, false),
                    (ChunkId::new(zc + 1, 2 * cc, 2 * rc + 2), false, true),
                    (ChunkId::new(zc + 1, 2 * cc + 1, 2 * rc - 1), false, false),
                ];
                let (mut plain_unequal, mut conf_unequal, mut compared) = (0, 0, 0);
                for (fid, column, fine_after) in sides {
                    let plain = world_elevation_tile(&ws, &p, fid, MIX_TS).expect("plain");
                    let conf = world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[coarse_id])
                        .expect("conformed");
                    let ce = edge_values(&coarse, column, fine_after);
                    let (n, u0) = compare_edges(&edge_values(&plain, column, !fine_after), &ce);
                    let (m, u1) = compare_edges(&edge_values(&conf, column, !fine_after), &ce);
                    assert_eq!(n, m);
                    assert!(n >= 80, "seed {seed} z{zc}: only {n} coincident texels compared");
                    assert_eq!(u1, 0, "seed {seed} z{zc} {fid:?}: {u1} of {m} conformed texels disagree");
                    plain_unequal += u0;
                    conf_unequal += u1;
                    compared += n;
                    cases += 1;
                }
                assert!(plain_unequal > 0, "seed {seed} z{zc}: plain tiles already agree ({compared} compared), nothing is tested");
                assert_eq!(conf_unequal, 0);
            }
        }
        assert_eq!(cases, 3 * 3 * 5, "3 seeds x 3 levels x 5 sides");
    }

    /// Same-level neighbours stay bit-identical even when only one of them
    /// borders the coarse tile along an edge and the other only at its corner.
    #[test]
    fn same_level_neighbours_still_agree_at_a_coarse_corner() {
        // Protects: the weight being a function of *position*, not of which
        // edge brought a coarser tile in. A = (z, 2cc+2, 2rc+1) borders the
        // coarse tile C along its east edge; B = (z, 2cc+2, 2rc+2), directly
        // below A, touches C only at C's south-east corner. Both are handed [C]
        // (the contract: every coarser tile within a band, diagonals included)
        // and must agree on their shared horizontal edge at every texel; B
        // handed [] must NOT (the contract is real, not decorative); and A's
        // south row must actually be pulled near the corner, so the corner is
        // exercised.
        let mut compared_total = 0usize;
        for seed in [1, 4242, -77_777] {
            let (ws, p) = seeded(seed);
            for zc in [4u32, 5, 6] {
                let s = 1u32 << (zc - 4);
                let (cc, rc) = (6 * s, 8 * s);
                let c = ChunkId::new(zc, cc, rc);
                let (a_id, b_id) = (
                    ChunkId::new(zc + 1, 2 * cc + 2, 2 * rc + 1),
                    ChunkId::new(zc + 1, 2 * cc + 2, 2 * rc + 2),
                );
                let a = world_elevation_tile_conformed(&ws, &p, a_id, MIX_TS, &[c]).expect("a");
                let b = world_elevation_tile_conformed(&ws, &p, b_id, MIX_TS, &[c]).expect("b");
                let (n, u) = compare_edges(&edge_values(&a, false, true), &edge_values(&b, false, false));
                assert_eq!(n, a.w, "every texel of a same-level edge is comparable");
                assert_eq!(u, 0, "seed {seed} z{zc}: {u} of {n} corner-edge texels disagree");
                compared_total += n;
                let plain_a = world_elevation_tile(&ws, &p, a_id, MIX_TS).expect("plain a");
                let moved = edge_values(&a, false, true)
                    .iter()
                    .zip(edge_values(&plain_a, false, true))
                    .filter(|((_, x), (_, y))| x.to_bits() != y.to_bits())
                    .count();
                assert!(moved > 0, "seed {seed} z{zc}: the corner texels were not pulled at all");
                let b_blind = world_elevation_tile_conformed(&ws, &p, b_id, MIX_TS, &[]).expect("b blind");
                let (_, u_blind) =
                    compare_edges(&edge_values(&a, false, true), &edge_values(&b_blind, false, false));
                assert!(u_blind > 0, "seed {seed} z{zc}: leaving the diagonal tile out changed nothing");
            }
        }
        assert!(compared_total >= 9 * 100, "only {compared_total} texels compared");
    }

    /// With no coarser tile near, the conformed tile *is* the plain tile.
    #[test]
    fn a_conformed_tile_is_the_plain_tile_wherever_no_coarser_tile_is_near() {
        // Protects: identity -- an empty list returns the plain tile outright;
        // a coarser tile that is far away, or a zero-width band, changes no
        // bit; and beside a real coarse tile, texels at or past the band are
        // bit-identical while the band itself is not (so the identity is not
        // just "the function does nothing").
        let (ws, p) = seeded(4242);
        let zc = 5u32;
        let c = ChunkId::new(zc, 12, 16);
        let fid = ChunkId::new(zc + 1, 2 * 12 + 2, 2 * 16);
        let plain = world_elevation_tile(&ws, &p, fid, MIX_TS).expect("plain");
        let bits = |t: &PyramidTile| t.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>();

        let none = world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[]).expect("none");
        assert_eq!(bits(&none), bits(&plain));

        let far = ChunkId::new(zc, 2, 3);
        let far_only = world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[far]).expect("far");
        assert_eq!(bits(&far_only), bits(&plain), "a coarser tile nowhere near must change nothing");

        let zero_band = conformed_with_band(&ws, &p, fid, MIX_TS, &[c], 0.0).expect("zero band");
        assert_eq!(bits(&zero_band), bits(&plain), "a zero band pulls nothing");

        let near = world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[c, far]).expect("near");
        let (mut inside, mut beyond) = (0usize, 0usize);
        for j in 0..plain.h {
            for i in 0..plain.w {
                let same = near.data[j * plain.w + i].to_bits() == plain.data[j * plain.w + i].to_bits();
                if i >= 17 {
                    assert!(same, "texel ({i}, {j}) lies beyond the band yet changed");
                    beyond += 1;
                } else if !same {
                    inside += 1;
                }
            }
        }
        assert!(inside > 0, "no texel inside the band moved: the test does not exercise the pull");
        assert!(beyond > 100);
    }

    /// Half way across the band the tile is half way to the coarser field.
    #[test]
    fn the_pull_is_half_way_at_half_the_band_and_total_on_the_edge() {
        // Protects: the band's width and the lerp direction against an
        // independent expectation: at 8 texels from the edge the output is the
        // mean of the plain value and the coarser level's own value, at 0 texels
        // it is the coarser value exactly, and at 1 texel it lies strictly
        // between and nearer the coarser one. A band mutated to 8 or 32, a
        // weight applied to the wrong operand, or a pull that is all-or-nothing
        // each fails one of these.
        let (ws, p) = seeded(1);
        let zc = 5u32;
        let (cc, rc) = (12u32, 16u32);
        let fid = ChunkId::new(zc + 1, 2 * cc + 2, 2 * rc);
        let plain = world_elevation_tile(&ws, &p, fid, MIX_TS).expect("plain");
        let conf = world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ChunkId::new(zc, cc, rc)])
            .expect("conf");
        let opts = world_amplify_opts(&ws, &p, MIX_TS);
        let b = pyramid_tile_bounds(GW, GH, fid.z as i32, fid.col, fid.row);
        let mut seen = 0usize;
        for j in (0..plain.h).step_by(7) {
            let cy = tcoord(b.y, j, plain.h, b.h);
            let coarse_at = |i: usize| {
                sample_elevation(&ws.field, GW, GH, tcoord(b.x, i, plain.w, b.w), cy, zc as i32, &opts)
            };
            let f8 = plain.data[j * plain.w + 8];
            let c8 = coarse_at(8);
            let got = conf.data[j * plain.w + 8];
            assert!((got - (f8 + c8) / 2.0).abs() < 1e-6, "row {j}: {got} is not the mean of {f8} and {c8}");
            assert_eq!(
                conf.data[j * plain.w].to_bits(),
                coarse_at(0).to_bits(),
                "row {j}: the edge is the coarser value"
            );
            let (f1, c1, g1) = (plain.data[j * plain.w + 1], coarse_at(1), conf.data[j * plain.w + 1]);
            if (f1 - c1).abs() > 1e-5 {
                assert!((g1 - c1).abs() < (g1 - f1).abs(), "row {j}: one texel in should still be mostly coarse");
                seen += 1;
            }
        }
        assert!(seen > 3, "only {seen} rows had a measurable difference");
    }

    /// The band is a distance in texels on *each* axis, even when the tile's
    /// texel grid is not square in world units.
    #[test]
    fn the_band_is_eight_texels_half_way_on_both_axes_of_a_rounded_tile() {
        // Protects: `sx`/`sy` (texels per coordinate unit) and their use on the
        // matching axis. At the fixture's 48x32 grid a tile is 3:2, and at a
        // tile size of 20 the short axis rounds 13.33 to 13 texels, so
        // `sx != sy` by about 5 % -- enough that a distance scaled by the
        // other axis's factor lands 0.4 texel off and the half-way expectation
        // below (8 texels in is the mean of the plain and coarser values)
        // fails. At the default tile size the two factors agree to under 1 %
        // and no test of the band could tell the axes apart. One tile lies
        // east of the coarse tile (distance along x) and one south of it
        // (distance along y).
        const TS20: usize = 20;
        let (ws, p) = seeded(1);
        // deep enough that the two levels differ at this tile size (measured:
        // at z5/z6 a 20-texel tile's plain and coarser values were identical, so
        // the half-way expectation would hold vacuously)
        let zc = 9u32;
        let (cc, rc) = (192u32, 256u32);
        let opts = world_amplify_opts(&ws, &p, TS20);
        let coarse = ChunkId::new(zc, cc, rc);
        let mut checked = 0usize;
        for (fid, along_x) in [
            (ChunkId::new(zc + 1, 2 * cc + 2, 2 * rc), true),
            (ChunkId::new(zc + 1, 2 * cc, 2 * rc + 2), false),
        ] {
            let plain = world_elevation_tile(&ws, &p, fid, TS20).expect("plain");
            let conf = world_elevation_tile_conformed(&ws, &p, fid, TS20, &[coarse]).expect("conf");
            assert_ne!((plain.w - 1) as f64 / (plain.h - 1) as f64, 1.5, "the fixture must be rounded for this test");
            let b = pyramid_tile_bounds(GW, GH, fid.z as i32, fid.col, fid.row);
            // texels 8 in from the shared edge, along the whole edge
            let (n_along, at) = if along_x { (plain.h, 8usize) } else { (plain.w, 8usize) };
            for k in (0..n_along).step_by(2) {
                let (i, j) = if along_x { (at, k) } else { (k, at) };
                let (cx, cy) = (tcoord(b.x, i, plain.w, b.w), tcoord(b.y, j, plain.h, b.h));
                let c = sample_elevation(&ws.field, GW, GH, cx, cy, zc as i32, &opts);
                let f = plain.data[j * plain.w + i];
                let got = conf.data[j * plain.w + i];
                assert!((got - (f + c) / 2.0).abs() < 1e-6, "{fid:?} ({i},{j}): {got} is not the mean of {f} and {c}");
                if (f - c).abs() > 1e-5 {
                    checked += 1;
                }
            }
        }
        assert!(checked > 6, "only {checked} texels had a measurable difference");
    }

    /// Every caller error is a `None`, not a panic.
    #[test]
    fn a_conformed_tile_refuses_a_coarser_list_that_cannot_be_right() {
        // Protects: the validation of `coarser` -- an entry not exactly one
        // level above, a level that does not exist, an index outside its level,
        // and a non-empty list at level 0 each return None (the module's
        // convention, because the consumer is a viewport that must not panic);
        // and the plain refusals still apply.
        let (ws, p) = seeded(4242);
        let fid = ChunkId::new(5, 12, 8);
        let ok = ChunkId::new(4, 5, 4);
        assert!(world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ok]).is_some());
        assert!(world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ChunkId::new(5, 5, 4)]).is_none(), "same level");
        assert!(world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ChunkId::new(3, 2, 2)]).is_none(), "two levels up");
        assert!(world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ChunkId::new(6, 5, 4)]).is_none(), "finer level");
        assert!(world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ChunkId::new(4, 16, 4)]).is_none(), "column out of range");
        assert!(world_elevation_tile_conformed(&ws, &p, fid, MIX_TS, &[ChunkId::new(4, 5, 16)]).is_none(), "row out of range");
        assert!(world_elevation_tile_conformed(&ws, &p, ChunkId::new(0, 0, 0), MIX_TS, &[ok]).is_none(), "level 0 has no coarser level");
        assert!(world_elevation_tile_conformed(&ws, &p, ChunkId::new(0, 0, 0), MIX_TS, &[]).is_some());
        assert!(world_elevation_tile_conformed(&ws, &p, ChunkId::new(5, 32, 0), MIX_TS, &[ok]).is_none(), "tile outside its level");
        assert!(world_elevation_tile_conformed(&ws, &p, fid, 1, &[ok]).is_none(), "tile size below the NaN floor");
    }

    // -- determinism --------------------------------------------------------

    /// Determinism: same world and chunk id give the same bytes across a
    /// fresh equal world, and a different world moves them.
    #[test]
    fn the_same_chunk_id_always_produces_the_same_bytes() {
        // Protects: determinism -- the same world and chunk id always
        // produce the same bytes, across a freshly-built equal world too
        // (not one `Vec` staying at one address), and a different world
        // must move the bytes so "deterministic" is not satisfied by a
        // constant.
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

    /// `world_sample_elevation` and `world_elevation_tile` agree texel for
    /// texel -- a tile is one way of evaluating the field, not a second
    /// implementation.
    #[test]
    fn the_point_query_and_the_tile_query_are_the_same_field() {
        // Protects: world_sample_elevation and world_elevation_tile agreeing
        // texel for texel at the tile's own sample coordinates -- a tile is
        // one way of evaluating the field, not a second implementation of
        // it.
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
        // Protects: the "real new information, not smoothing" claim at one
        // depth -- the refined tile's curvature must clear a plain bilinear
        // upsample of the same footprint by a wide, measured margin.
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
        // Protects: EF-4's acceptance bar swept across depth rather than
        // sampled once -- the curvature ratio, the max deviation from a
        // plain upsample, and the tile's own peak-to-peak all clear their
        // measured floors at every depth from z2 to z12, three seeds each.
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

    /// Every named refusal in `world_coarse`/`world_elevation_tile` degrades
    /// to `None`, never a panic crossing the gdext boundary.
    #[test]
    fn every_reachable_caller_error_is_a_none_rather_than_a_panic() {
        // Protects: every refusal in world_coarse/world_elevation_tile's own
        // table returns None rather than panicking or indexing out of
        // bounds -- a panic here crosses the gdext boundary and takes the
        // Godot process with it.
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
