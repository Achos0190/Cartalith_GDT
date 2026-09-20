//! Tile-bounded erosion — EF-3 of `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`.
//!
//! Re-runs **this engine's own generation-time erosion kernel** over one
//! rectangle of the world at a finer resolution than the world pass ever ran
//! at, so the detail EF-0 synthesises into that rectangle is worked by the
//! tile's own routing rather than only by noise.
//!
//! **What that produces depends on `p.deposit`, and the honest statement of it
//! needs both halves.** Measured against a drainage network taken once from the
//! un-eroded tile and held fixed: with the deposition term **off**, the pass
//! deepens the ground the tile's own water already runs over. At the world's
//! own `deposit: 0.3` it moves that same measure the **other** way — and so
//! does the world's own pass on the world's own field, at the world's own
//! resolution, with no tile involved. The numbers and the mechanism are under
//! *"What the tile does to the drainage signal"* below. A one-sentence summary
//! would have to pick one of those two and be false for the other, so there
//! isn't one.
//!
//! # New capability, not a port — flagged, per `cartalith-porting-discipline`
//!
//! There is no reference function this ports, and **no golden value is claimed
//! or available**. The reference's own `amplifyRegion`/`refineTile` are
//! elevation-only, as is this port's [`cartalith_terrain::amplify::amplify_region`],
//! and neither ever re-runs erosion at zoom. Nothing below changes the output
//! of any existing function: [`crate::stream_power_kernel`] is now a one-line
//! delegation to [`crate::stream_power_kernel_bounded`], its golden fixtures
//! pass unmodified, and the two extra arguments are `None` on every world path.
//! What is new is a second, tile-scoped *entry* into the same kernel plus the
//! two boundary conditions that make a tile-scoped answer mean anything.
//!
//! # Which erosion, and why only one of them
//!
//! Checked at the call site (`cartalith-engine/src/lib.rs`), not assumed: the
//! **only** erosion a default `generate_terrain` runs is one
//! [`crate::stream_power_kernel`] pass — `light_iters =
//! max(4, round(p.stream.iters * 0.6))`, which is **9** at
//! `StreamParams`' own `iters: 15` default — inside the `p.carve_rivers`
//! block, followed by [`crate::isostatic_rebound`]. Everything else in this
//! crate is opt-in and **off by default**: [`crate::droplet_kernel`] and
//! [`crate::erode_thermal`] belong to `cartalith-engine`'s manual `erode_op`,
//! and `velocity`/`glacial`/`coastal`/`hillslope`/`sediment`/`tidal` are
//! `ErosionPassParams` toggles whose own doc comment states that *"every toggle
//! is off and every cycle count is zero by default"*.
//!
//! So a tile that re-runs stream power alone is re-running **what the world
//! actually did**, not a scoped-down subset of it. That is why this module
//! exposes stream power and nothing else, and it is a finding rather than a
//! simplification: a world that *has* turned a pass on would need that pass
//! mirrored here before a tile of it could claim consistency, and this module
//! does not do that yet.
//!
//! # The two boundary conditions
//!
//! **Elevation — a pinned outer ring.** The tile's outermost fine cells are
//! held at exactly the elevation they arrived with, which is EF-0's refinement
//! of the *coarse eroded field*. That is the Dirichlet base level the implicit
//! solver relaxes the interior toward, and it is the same open-boundary shape
//! EF-1 uses for accumulation (`cartalith_hydrology::tile`'s own "the tile's
//! edge is an open boundary"). It buys two things at once:
//!
//! - the interior erodes toward a base level the *world* chose, rather than
//!   toward whatever the tile's own lowest cell happens to be, and
//! - the tile's edge comes out **bit-identical to its input**, so two tiles
//!   meeting at a seam agree exactly as well after erosion as EF-0 made them
//!   agree before it. No new seam is introduced; the existing guarantee is
//!   inherited. `tile_erode_leaves_the_ring_bit_identical_and_moves_the_interior`
//!   and `the_ring_survives_the_deposition_pass_and_the_final_clamp` are that
//!   claim as tests — one per guard inside
//!   [`crate::stream_power_kernel_bounded`], because a mutation run found two
//!   of the three guards holding only by accident (see the second test's own
//!   comment).
//!
//! The price, measured rather than argued: an interior cell beside the ring
//! erodes while the ring does not, so a one-fine-cell lip forms at the tile
//! edge — 0.12% of the tile's relief on the world measured in
//! [`tile_erode`]'s own note, which carries the numbers and the bar.
//!
//! **Drainage area — the units are the world's, not the tile's.** The kernel
//! derives its incision coefficient from `A^m`, and it seeds `A` at `1.0` per
//! cell of whatever grid it is handed. On a `refine`-times-finer tile that
//! makes every drainage area `refine²` times too large, so `Cc` — and with it
//! the amount of erosion — comes out on the tile's scale instead of the
//! world's. [`uniform_area_seed`] is the correction, and it is EF-1's own unit
//! convention: `1/refine²` per fine cell, so one coarse cell's worth of fine
//! cells still contributes exactly `1.0`. A caller that also has EF-1's
//! boundary crossings (`cartalith_hydrology::tile`) can hand those in instead,
//! on the same scale and with no conversion.
//!
//! # The third correction, which is not a boundary condition and was not obvious
//!
//! A tile-bounded re-run at the world's own `k` and `iters` is very nearly a
//! **no-op that costs full price**. Measured on real generated worlds, at the
//! world's own `deposit`: it moves the surface by 0.03–0.11% of the tile's
//! relief, moves the fixed-network drainage measure by `-0.0042 .. +0.0031`
//! (which is nothing), and *lowers* the tile's curvature energy to 0.82–0.89x.
//! It is a weak smoother wearing the cost of an erosion pass. That is the
//! single finding that most shaped this module — the derivation and the numbers
//! are the next section.
//!
//! An earlier version of this paragraph said the unscaled re-run *subtracts*
//! drainage structure, `-0.026 .. -0.049`. That figure came from a measure that
//! recomputed drainage area **on the pass's own output**, which is circular;
//! the harness's own doc comment now carries the correction in full. Against a
//! network held fixed from the un-eroded tile the unscaled run is inert, not
//! harmful. The curvature half was never circular and stands unchanged.
//!
//! # Why [`tile_erode`] multiplies `p.k` by `refine`, and why it is not a taste call
//!
//! Written out in full because this is the piece of EF-3 a future reader is
//! most likely to take for a mistake and delete.
//!
//! ## The derivation
//!
//! [`crate::stream_power_kernel`] is implicit upwind advection along the
//! receiver chain: `h_i ← (h_i + C·h_r)/(1 + C)`, with
//! `C = K·dt·A^m / L`. `L` is the receiver distance **in cells** — `1` or
//! `√2`, never a physical length. So `C` is the distance, *in cells*, that an
//! incision signal travels per iteration, and the physical distance it travels
//! is `C · Δx`.
//!
//! A tile `refine` times finer has `Δx / refine`. Hold `A` on the world's scale
//! ([`uniform_area_seed`]) and `C` is unchanged, so each iteration moves the
//! signal `refine` times less **physical** ground than the world pass's own
//! iteration did. Matching the world therefore needs `C · refine`, and `C` is
//! linear in `k`.
//!
//! `iters · refine` is the same knob: measured over the same six cases below,
//! on the fixed network, it lands within `0.0009` of `k · refine`'s own
//! correlation at `deposit 0.0` and within `0.021` at the world's own
//! `deposit 0.3` — for `refine` times the work, since an iteration is `O(n)`
//! while `k` is free. Scaling `k` is the same correction
//! [`crate::hillslope_extent_scale`] already applies to `diffuse_d` for the
//! same reason (`DECISIONS.md` §7m: *"a coefficient over a one-cell Laplacian
//! must scale as 1/cell_km²"*), one process over.
//!
//! ## What the tile does to the drainage signal, measured
//!
//! Three tiles of a real generated world at `refine` 4 and 8. The measure is
//! the transverse-incision/log-drainage-area correlation — how much more deeply
//! a cell is cut when more water passes through it — **against a drainage
//! network computed once from the un-eroded EF-0 tile and then held fixed**.
//! Fixed is the whole point: recomputing drainage on the eroded output lets a
//! groove certify itself by existing, and that circular version of this table
//! reported the opposite sign in five of the six `deposit 0.3` cases.
//!
//! With the deposition term **off**:
//!
//! ```text
//!   tile          EF-0      as-world k         k x refine
//!   (48,120) r4   +0.2996   +0.3204 (+0.0208)  +0.3629 (+0.0632)
//!   (48,120) r8   +0.2761   +0.2935 (+0.0174)  +0.3615 (+0.0854)
//!   (40,40)  r4   +0.3753   +0.3966 (+0.0213)  +0.4337 (+0.0584)
//!   (40,40)  r8   +0.3591   +0.3792 (+0.0201)  +0.4446 (+0.0856)
//!   (180,120) r4  +0.3359   +0.3666 (+0.0307)  +0.4025 (+0.0666)
//!   (180,120) r8  +0.3210   +0.3473 (+0.0262)  +0.4078 (+0.0868)
//! ```
//!
//! At the world's own `deposit: 0.3`, the same six runs:
//!
//! ```text
//!   tile          EF-0      as-world k         k x refine
//!   (48,120) r4   +0.2996   +0.3024 (+0.0028)  +0.2495 (-0.0501)
//!   (48,120) r8   +0.2761   +0.2791 (+0.0031)  +0.1920 (-0.0840)
//!   (40,40)  r4   +0.3753   +0.3711 (-0.0042)  +0.2532 (-0.1221)
//!   (40,40)  r8   +0.3591   +0.3620 (+0.0029)  +0.2120 (-0.1471)
//!   (180,120) r4  +0.3359   +0.3340 (-0.0019)  +0.2191 (-0.1168)
//!   (180,120) r8  +0.3210   +0.3230 (+0.0020)  +0.1731 (-0.1480)
//! ```
//!
//! **The sign flip is the kernel's, not the tile's**, and the control that says
//! so runs no tile at all: `stream_power_kernel` over the whole 256x192 world
//! field, world path, no pin, no seed, `k` untouched, measured by this same
//! fixed-network metric, moves it `+0.0356` with deposition off and `-0.0551`
//! at `0.3`. A `refine = 1` tile reproduces both to three decimals (`+0.0360`
//! against `+0.0356`, `-0.0552` against `-0.0551`). So EF-3 at the world's
//! parameters is doing to a tile exactly what generation does to the world.
//!
//! **Why deposition inverts it**, read out of the kernel rather than supposed:
//! `sed[i]` is that iteration's own incision, it accumulates down the receiver
//! chain (`sed[r] += sed[i]`), and the transport capacity it is tested against
//! is `0.005·A^0.5·slope`, which a low-gradient valley floor cannot meet however
//! large `A` is. With `uplift: 0.0` — the world's default — the deposition
//! ceiling `old_h[i] + dt·u[i]` is **exactly the cell's height at the start of
//! the iteration**, so the trunk is refilled to where it began while the
//! low-drainage hillslopes keep their incision. On `(180,120)` at `refine 8`,
//! mean `Δh` by decile of the fixed drainage area: at `deposit 0.0` the top
//! decile falls `-4.52e-3` against the bottom decile's `-6.07e-4`; one step to
//! `deposit 0.05` and the top decile is `-1.20e-3` against `-5.43e-4`, the
//! bottom decile having barely moved. The cancellation is aimed at precisely the
//! cells the measure is asking about.
//!
//! **So a caller chooses.** Detail organised by the tile's own drainage wants
//! `deposit: 0.0`; a tile that looks like the world it is a zoom of wants the
//! world's own parameters and gets the world's own behaviour, sign included.
//! [`tile_erode`] takes `p.deposit` from the caller untouched — `k` is the one
//! field it corrects — and does not decide this for anyone.
//!
//! Curvature energy, both settings: `k x refine` holds or raises it (1.02–1.66x
//! at `deposit 0.0`, 0.91–2.08x at `0.3`) while the unscaled run lowers it
//! (0.96–0.99x and 0.82–0.89x). Surface movement, as rms over the tile's own
//! relief: `k x refine` 0.54–1.01% at `deposit 0.0` and 0.17–0.39% at `0.3`,
//! unscaled 0.07–0.26% and 0.03–0.11%.
//!
//! The harness is `cartalith-engine/tests/ef3_tile_erosion.rs`'s
//! `tile_erosion_incises_along_a_fixed_drainage_network_only_with_deposition_off`,
//! which asserts every direction above — including the world control and the
//! disagreement with the circular measure — and prints these columns on every
//! run, so a regression here goes red rather than going stale.
//!
//! ## The upper bound, also measured
//!
//! Sweeping the multiplier on the fixed network puts `refine` at the top of the
//! useful range rather than merely inside it. At `deposit 0.0`, against EF-0:
//!
//! ```text
//!   multiplier   correlation moved   curvature
//!   k x 1        +0.017 .. +0.031    0.96 .. 0.99x
//!   k x refine   +0.058 .. +0.087    1.02 .. 1.66x
//!   k x 2refine  +0.066 .. +0.102    1.30 .. 3.54x
//!   k x 4refine  +0.036 .. +0.088    2.13 .. 10.66x
//! ```
//!
//! `2 · refine` buys at most `+0.020` more correlation — and *loses* a little
//! in two of the six cases — for 1.3–2.1x the curvature energy. `4 · refine`
//! is past the peak outright: below `2 · refine` in all six cases and below
//! `refine` itself in three of them, with curvature reaching `10.66x`, which
//! is the implicit scheme's own numerical roughness at a large Courant number
//! rather than terrain. At `deposit 0.3` every multiplier above `1` moves the
//! correlation down and each step down is larger (`-0.05 .. -0.15` at `refine`,
//! `-0.15 .. -0.27` at `2 refine`, `-0.26 .. -0.38` at `4 refine`), so nothing
//! above `refine` is defensible from that side either. `refine` is the derived
//! value and the measurements bracket it from both.
//!
//! # What one tile costs, and whether that is affordable
//!
//! Measured through [`tile_erode`] itself on a real generated world, release
//! build, 16 logical cores, `iters: 9`; median of nine with min..max, from
//! `cartalith-engine/tests/ef3_tile_erosion.rs`'s `measure_tile_erosion_cost`
//! run **alone** (`MISTAKES.md`'s rule for quoting a timing), across **three
//! independent runs of the harness, in three separate processes**:
//!
//! ```text
//!   fine tile   EF-0 alone      EF-3             of which setup    9 iterations
//!   128x128     0.23..0.24 ms    4.21..4.29 ms    3.04..3.14 ms    1.15..1.21 ms
//!   256x256     0.91..1.09 ms   17.79..18.58 ms  12.94..13.65 ms   4.71..5.60 ms
//!   512x512     3.43..3.59 ms   72.83..73.25 ms  50.60..50.97 ms  21.86..22.63 ms
//! ```
//!
//! Each cell is the min..max of the three runs' medians; the 256x256 row is
//! measured twice per run (`refine 8` from a 32x32 coarse footprint and
//! `refine 16` from a 16x16 one), which is why it spreads widest.
//!
//! **The previous version of this table was two runs and its brackets were too
//! tight to survive a third.** It quoted 512x512 at `EF-0 3.35..3.37 /
//! EF-3 72.45..72.65 / setup 50.34..50.38 / 9 iters 22.1..22.3`, and **all
//! three** of the runs above land outside **all four** of those. (A verifier
//! reported the same disagreement from its own runs, which is what sent this
//! back to be re-measured; the numbers here are these three runs, not that
//! report.) That is `MISTAKES.md`'s own rule — a median of nine on a noisy
//! device is still one sample of the median — so this table is a spread across
//! processes rather than a bracket from one pair of them, and the right way to
//! widen it is another run, not a wider guess. One earlier cold run measured
//! the 128x128 median at 6.90 ms
//! with a 5.44..8.00 spread; every warm run since lands in the table, and the
//! first tile a process touches is the one that pays for the thread pool.
//!
//! Three things follow, and the third is the one that shaped the design:
//!
//! - **257-284 ns per fine cell**, near-linear from 16k to 262k cells. A
//!   256x256 tile is 17.8-18.6 ms; a viewport of a dozen of them is a fifth of
//!   a second of work. Against EF-0 alone for the same tile that is 16.8-21.4x
//!   across the twelve (size, run) pairs — given as a range rather than a
//!   single multiplier because the denominator is a sub-millisecond number at
//!   the small sizes, with a within-run spread (0.23..0.28 ms at 128x128) as
//!   wide as the spread between runs.
//!   That is off-main-thread, budgeted work — `LOD_DETAIL_SCOPE.md` D6's own
//!   shape — not a per-frame cost, and it is **not** the "prohibitively
//!   expensive" case the design document worried about. Stated plainly because
//!   the brief that produced this module asked for it plainly: EF-3 is usable
//!   at interactive zoom rates on this machine, with a worker thread and a
//!   cache (EF-5), and is not usable synchronously on the frame that needs it.
//! - **The setup dominates: 69-74% of every row.** The priority-flood
//!   depression fill, the receiver pass and the multiple-flow-direction
//!   drainage-area accumulation (whose per-cell body walks the 3x3
//!   neighbourhood twice, with a `powf` each time) all run once, before the
//!   first iteration; they are what a tile pays for even at `iters: 0`.
//! - **So an iteration is cheap — 0.13 ms at 128x128, 0.52-0.62 ms at
//!   256x256 — and yet `iters x refine` is still the expensive way to apply the
//!   resolution correction above.** At `refine 8` it means 72 iterations
//!   instead of 9: roughly 13 ms of setup plus 38-45 ms of iteration, so 51-58
//!   ms against 17.8-18.6 ms, for an answer that measured within 0.0009 of it
//!   at `deposit 0.0` and within 0.021 at `deposit 0.3`. `k x refine` is free.
//!
//! # What this does not do
//!
//! No uplift term is invented: `stress` reaches the kernel as the caller
//! supplies it, and at `StreamParams`' own `uplift: 0.0` default the whole
//! uplift term is zero whatever `stress` says. No refined `rain` or `resist`
//! field is invented either — both are required arguments, because a plausible
//! uniform stand-in for a field the world really has is exactly the failure
//! `MISTAKES.md` catalogues most often. A caller with only coarse fields
//! upsamples them and owns that approximation, the same way EF-1's own header
//! owns its uniform rainfall.

use crate::{stream_power_kernel_bounded, StreamPowerParams};

/// The tile's outer one-cell ring — the cells [`tile_erode`] holds fixed.
///
/// Public because it is what a caller needs to *check* the guarantee, and
/// because a caller with its own idea of which cells are authored
/// (`DirtyTracker`'s edited tiles, EF-4) can union it with this rather than
/// re-deriving the ring and getting the corner cells wrong.
///
/// A grid under three cells on an axis is all ring on that axis, which is
/// correct rather than degenerate: such a tile has no interior to erode and
/// comes back unchanged.
#[must_use]
pub fn ring_mask(w: usize, h: usize) -> Vec<bool> {
    let mut m = vec![false; w * h];
    for (i, slot) in m.iter_mut().enumerate() {
        let (x, y) = (i % w, i / w);
        *slot = x == 0 || y == 0 || x + 1 == w || y + 1 == h;
    }
    m
}

/// The interior drainage-area seed a tile starts from, in
/// **coarse-cell-equivalents**: `1/refine²` per fine cell.
///
/// The world pass seeds `1.0` per cell, meaning *"one cell drains itself"*. A
/// tile `refine` times finer has `refine²` fine cells per coarse cell, so the
/// same physical ground has to be seeded `1/refine²` per fine cell for a tile
/// `A` and a world `A` to mean the same drained area. Without it a `refine = 8`
/// tile reports `64x` the drainage area for the same ground, and since the
/// kernel's incision coefficient goes as `A^0.5`, erodes `8x` as hard as the
/// world pass did over the same ground.
///
/// `refine = 1` — a same-resolution re-run — gives the world pass's own `1.0`
/// back, which is the honest degenerate case rather than a special one.
///
/// This is deliberately not folded into [`tile_erode`]: a caller that has
/// EF-1's boundary crossings (`cartalith_hydrology::tile::tile_flow`'s own
/// seeding, on this same scale) passes those instead, and the choice belongs at
/// the call site where the caller knows whether a trunk river enters the tile
/// from outside.
///
/// # Panics
///
/// If `refine` is zero — a tile cannot be zero times finer than the world, and
/// the alternative is a division by zero producing an infinite seed.
#[must_use]
pub fn uniform_area_seed(w: usize, h: usize, refine: usize) -> Vec<f32> {
    assert!(refine > 0, "refine must be at least 1 (a same-resolution re-run), got 0");
    let k = refine as f64;
    vec![(1.0 / (k * k)) as f32; w * h]
}

/// **EF-3.** One tile's refined elevation, eroded in place by the world's own
/// stream-power kernel, bounded to the tile.
///
/// `refined` arrives as EF-0's output for this rectangle and leaves eroded.
/// `stress`, `resist` and `rain` are the world's own fields **at the tile's
/// resolution** — the caller upsamples them and owns that approximation (see
/// the module header). `p` is the world's own [`StreamPowerParams`], passed
/// unmodified: the one correction a tile needs, `k · refine`, is applied here
/// rather than asked of the caller, because forgetting it produces a pass that
/// runs, costs its full time, and changes almost nothing but the tile's
/// curvature, which it lowers — see the module header's own measurements. `refine` is how many fine cells cover one
/// coarse cell along an axis; `1` is a legitimate same-resolution re-run and
/// leaves `k` alone. `area_seed` is [`uniform_area_seed`] or EF-1's
/// boundary-seeded equivalent, on that function's documented scale.
///
/// Deterministic: a pure function of its arguments, with no RNG anywhere in the
/// kernel's path — `stream_power_kernel` draws no random numbers at all
/// (unlike [`crate::droplet_kernel`], which is seeded). The same tile and the
/// same inputs give bit-identical bytes, which is what makes a cached tile
/// safe to drop and re-derive (EF-5).
///
/// # The ring, and the lip beside it
///
/// Every cell [`ring_mask`] marks comes back bit-identical, which is the seam
/// guarantee the module header describes. The cost is a one-cell lip: the first
/// interior cell erodes and its neighbour on the ring does not, so the step
/// between them opens. **Measured** on a real world at `refine 8`, over a
/// 16x16-coarse tile of relief `0.5654`: the worst ring-to-interior step goes
/// from `0.002086` before erosion to `0.002751` after, i.e. it opens by 0.12%
/// of the tile's relief. `two_tiles_agree_with_the_one_tile_that_covers_them`
/// (`cartalith-engine/tests/ef3_tile_erosion.rs`) is where that is measured,
/// and it holds the lip under 2% of relief so a regression trips it.
///
/// A consumer that cannot spend that cell asks for a tile one coarse cell
/// larger on each side and crops — the apron a nested-grid model uses, and one
/// this module deliberately does not do for the caller, since only the caller
/// knows whether the extra ground is free to synthesise.
///
/// # Panics
///
/// If any dimension is zero, if `refine` is zero, if any of the four slices is
/// not exactly `w * h` cells, or if `p.world` is set. A deep-zoom tile is a
/// sub-rectangle of the world by definition, so wrapping it would join the
/// tile's own left and right edges to each other — the ring would stop being a
/// ring and the pinned boundary would stop bounding anything. EF-1's
/// `tile_flow` refuses the same case rather than silently treating a tile edge
/// as the map seam.
///
/// Every one of those is a programmer error in assembling the tile, never a
/// property of the data, which is why this asserts where
/// `cartalith_engine::elevation::world_elevation_tile` returns `None` — that
/// one is reachable from a viewport, and `cartalith-rust-conventions` is
/// explicit that a panic crossing the gdext boundary takes the Godot process
/// with it. **A future bridge validates before calling this**, the same way
/// `world_elevation_tile` does; it does not hand a `#[func]` straight through.
/// Nothing in this module is called by shipping code yet.
#[allow(clippy::too_many_arguments)]
pub fn tile_erode(
    refined: &mut [f32],
    stress: &[f32],
    resist: &[f32],
    rain: &[f32],
    w: usize,
    h: usize,
    p: &StreamPowerParams,
    refine: usize,
    area_seed: &[f32],
) {
    assert!(w > 0 && h > 0, "tile_erode needs a non-empty tile, got {w}x{h}");
    assert!(refine > 0, "refine must be at least 1 (a same-resolution re-run), got 0");
    let n = w * h;
    assert!(
        !p.world,
        "tile_erode is a sub-rectangle of the world, so it cannot wrap: p.world must be false"
    );
    for (name, len) in
        [("refined", refined.len()), ("stress", stress.len()), ("resist", resist.len()), ("rain", rain.len()), ("area_seed", area_seed.len())]
    {
        assert!(len == n, "{name} is {len} cells, needs exactly {n} ({w}x{h})");
    }
    let pinned = ring_mask(w, h);
    let tile_p = StreamPowerParams { k: p.k * refine as f64, ..*p };
    stream_power_kernel_bounded(refined, stress, resist, rain, w, h, &tile_p, Some(&pinned), Some(area_seed));
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = 96;
    const H: usize = 64;
    const REFINE: usize = 8;

    /// Deterministic value hash in `[-1, 1)` — reproducible pseudo-detail with
    /// no dependency on `cartalith-noise`'s own seeding, so a change there
    /// cannot quietly move these fixtures.
    fn hash_noise(x: i64, y: i64, salt: u64) -> f64 {
        let mut v = (x as u64)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ salt.wrapping_mul(0x1656_67B1_9E37_79F9);
        v ^= v >> 33;
        v = v.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
        v ^= v >> 33;
        v = v.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
        v ^= v >> 33;
        ((v >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }

    /// Smooth (value-noise) fBm, so the fixture reads as terrain rather than as
    /// white noise — a per-cell random field has no coherent flow paths and
    /// would make every routing property below vacuous.
    fn fbm(x: f64, y: f64, salt: u64) -> f64 {
        let (mut v, mut a, mut f, mut norm) = (0.0, 1.0, 1.0, 0.0);
        for o in 0..4u64 {
            let (px, py) = (x * f, y * f);
            let (x0, y0) = (px.floor(), py.floor());
            let (tx, ty) = (px - x0, py - y0);
            let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
            let c00 = hash_noise(x0 as i64, y0 as i64, salt + o);
            let c10 = hash_noise(x0 as i64 + 1, y0 as i64, salt + o);
            let c01 = hash_noise(x0 as i64, y0 as i64 + 1, salt + o);
            let c11 = hash_noise(x0 as i64 + 1, y0 as i64 + 1, salt + o);
            v += a * ((c00 * (1.0 - sx) + c10 * sx) * (1.0 - sy) + (c01 * (1.0 - sx) + c11 * sx) * sy);
            norm += a;
            a *= 0.6;
            f *= 2.0;
        }
        v / norm
    }

    /// A refined tile: a regional slope with fBm relief on it, at roughly the
    /// amplitude `amplify_region`'s own `detail_amp: 0.14` reaches on sloped
    /// ground. The real-EF-0 composition is checked in
    /// `cartalith-engine/tests/ef3_tile_erosion.rs`, which is the only crate
    /// that can see both halves; these tests deliberately stay independent of
    /// it, the same split EF-1's own tests make.
    fn tile(salt: u64) -> Vec<f32> {
        let n = W * H;
        let mut f = vec![0f32; n];
        for y in 0..H {
            for x in 0..W {
                let (u, t) = (x as f64 / W as f64, y as f64 / H as f64);
                let v = 0.62 - 0.24 * t + 0.05 * fbm(u * 7.0, t * 5.0, salt) + 0.02 * fbm(u * 19.0, t * 13.0, salt + 99);
                f[y * W + x] = v as f32;
            }
        }
        f
    }

    /// The world's own defaults, as `cartalith-engine`'s carve block builds
    /// them: `StreamParams { k: 0.012, uplift: 0.0, deposit: 0.3,
    /// climate_k: 0.5, iters: 15 }` with `light_iters = max(4, round(15*0.6))`.
    fn world_params() -> StreamPowerParams {
        StreamPowerParams { k: 0.012, uplift: 0.0, deposit: 0.3, climate_k: 0.5, iters: 9, resist: 0.50, g: 1.0, world: false, sea: 0.30 }
    }

    struct Fields {
        stress: Vec<f32>,
        resist: Vec<f32>,
        rain: Vec<f32>,
    }

    fn fields() -> Fields {
        Fields { stress: vec![0f32; W * H], resist: vec![0.5f32; W * H], rain: vec![1.0f32; W * H] }
    }

    fn run(refined: &mut [f32], p: &StreamPowerParams) {
        let f = fields();
        tile_erode(refined, &f.stress, &f.resist, &f.rain, W, H, p, REFINE, &uniform_area_seed(W, H, REFINE));
    }

    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|x| x.to_bits()).collect()
    }

    // -- the ring ------------------------------------------------------------

    /// **The seam guarantee.** Every ring cell comes out bit-identical, and the
    /// interior does not — a test that only checked the first half would pass
    /// on a `tile_erode` that did nothing at all.
    #[test]
    fn tile_erode_leaves_the_ring_bit_identical_and_moves_the_interior() {
        let before = tile(7);
        let mut after = before.clone();
        run(&mut after, &world_params());

        let mask = ring_mask(W, H);
        let (mut ring, mut moved_interior, mut interior) = (0usize, 0usize, 0usize);
        for i in 0..W * H {
            if mask[i] {
                assert_eq!(
                    before[i].to_bits(),
                    after[i].to_bits(),
                    "ring cell ({}, {}) moved from {} to {}",
                    i % W,
                    i / W,
                    before[i],
                    after[i]
                );
                ring += 1;
            } else {
                interior += 1;
                if before[i].to_bits() != after[i].to_bits() {
                    moved_interior += 1;
                }
            }
        }
        assert_eq!(ring, 2 * (W + H) - 4, "the ring is the perimeter");
        assert!(
            moved_interior > interior / 2,
            "only {moved_interior} of {interior} interior cells moved -- the erosion barely ran, so the ring check proves nothing"
        );
    }

    /// **The two halves of the ring guarantee a mutation test found were only
    /// holding by accident**, added after it did rather than claimed before.
    ///
    /// The bit-identity above is really three guards inside
    /// [`crate::stream_power_kernel_bounded`] — the incision loop, the
    /// deposition loop and the final `clamp` — and the test above exercises
    /// only the first. Deleting either of the other two left every test in this
    /// module green:
    ///
    /// - **the `clamp` guard** is invisible while every ring value is already
    ///   inside `[0, 1]`, which `amplify_region` guarantees for EF-0's own
    ///   output. It is not invisible for a caller whose field is not — and the
    ///   contract [`tile_erode`] documents is *bit-identical to what went in*,
    ///   without a range qualifier. Clamping a boundary condition would break
    ///   the seam exactly when two tiles share an out-of-range edge, which is
    ///   why the guard is right and why it is asserted here rather than
    ///   removed.
    /// - **the deposition guard** is inert at `uplift == 0` — and the world's
    ///   own default *is* `0.0`, which is why it looked dead. The mechanism,
    ///   read back out of the kernel rather than guessed: a pinned cell's
    ///   `sed[i]` ceiling is `old_h[i] + dt*u[i]`, and with `u == 0` that is
    ///   exactly the height it still has, so both deposition blocks compute
    ///   `d = max(0, ceil - fld[i]) = 0` on their own. Away from `uplift == 0`
    ///   the ceiling rises above the pinned height and deposition really would
    ///   lift the ring. So this case has to run at a non-zero uplift or it
    ///   asserts nothing.
    #[test]
    fn the_ring_survives_the_deposition_pass_and_the_final_clamp() {
        let f = fields();
        let seed = uniform_area_seed(W, H, REFINE);

        // (a) Out of `[0, 1]` on the ring, so the final clamp is reachable.
        let mut before = tile(7);
        for (k, i) in (0..W * H).filter(|&i| ring_mask(W, H)[i]).enumerate() {
            before[i] = if k % 2 == 0 { 1.5 } else { -0.5 };
        }
        let mut after = before.clone();
        tile_erode(&mut after, &f.stress, &f.resist, &f.rain, W, H, &world_params(), REFINE, &seed);
        let mask = ring_mask(W, H);
        let mut out_of_range = 0usize;
        for i in 0..W * H {
            if mask[i] {
                assert_eq!(before[i].to_bits(), after[i].to_bits(), "the clamp moved ring cell {i} from {} to {}", before[i], after[i]);
                out_of_range += 1;
            }
        }
        assert_eq!(out_of_range, 2 * (W + H) - 4, "every ring cell must have been out of range, or the clamp was never reached");
        assert!(after.iter().enumerate().any(|(i, v)| !mask[i] && v.to_bits() != before[i].to_bits()), "the interior must still have eroded");

        // (b) Non-zero uplift, so the deposition pass can actually lift a
        //     pinned cell. `deposit` must be live too, or there is no
        //     deposition pass at all to guard.
        let p = StreamPowerParams { uplift: 0.02, ..world_params() };
        assert!(p.deposit > 0.0, "the world's own deposit must be non-zero, or this case runs no deposition pass");
        let before = tile(7);
        let mut after = before.clone();
        tile_erode(&mut after, &f.stress, &f.resist, &f.rain, W, H, &p, REFINE, &seed);
        let mut ring = 0usize;
        for i in 0..W * H {
            if mask[i] {
                assert_eq!(before[i].to_bits(), after[i].to_bits(), "deposition moved ring cell {i} from {} to {}", before[i], after[i]);
                ring += 1;
            }
        }
        assert_eq!(ring, 2 * (W + H) - 4);
        // And the control that says the deposition pass had something to do:
        // at this uplift an unpinned run really does raise ring cells.
        let mut unpinned = before.clone();
        crate::stream_power_kernel_bounded(&mut unpinned, &f.stress, &f.resist, &f.rain, W, H, &p, None, Some(&seed));
        let raised = (0..W * H).filter(|&i| mask[i] && unpinned[i] > before[i]).count();
        assert!(raised > 0, "no ring cell rises without the pin at uplift {}, so this case still proves nothing", p.uplift);
    }

    /// The pin is a live parameter, not a documented intention: running the
    /// bare kernel over the same tile with no pin **does** move the ring, so
    /// the bit-identity above is [`tile_erode`]'s doing.
    #[test]
    fn without_the_pin_the_ring_would_move() {
        let before = tile(7);
        let mut unpinned = before.clone();
        let f = fields();
        crate::stream_power_kernel_bounded(
            &mut unpinned,
            &f.stress,
            &f.resist,
            &f.rain,
            W,
            H,
            &world_params(),
            None,
            Some(&uniform_area_seed(W, H, REFINE)),
        );
        let mask = ring_mask(W, H);
        let moved = (0..W * H).filter(|&i| mask[i] && before[i].to_bits() != unpinned[i].to_bits()).count();
        assert!(moved > 0, "the unpinned control must move at least one ring cell, or the pin is untested");
    }

    // -- determinism ---------------------------------------------------------

    /// Same inputs, bit-identical output — including from separately allocated
    /// copies of every input, so nothing has leaked in from an address or an
    /// allocation order.
    #[test]
    fn tile_erode_is_deterministic() {
        let base = tile(7);
        let (mut a, mut b) = (base.clone(), base.clone());
        run(&mut a, &world_params());
        run(&mut b, &world_params());
        assert_eq!(bits(&a), bits(&b), "two runs of the same tile must be bit-identical");

        // A separately built, equal tile -- determinism of the inputs, not of
        // one `Vec` staying at one address.
        let mut c = tile(7);
        assert_eq!(bits(&c), bits(&base), "the fixture itself must be reproducible");
        run(&mut c, &world_params());
        assert_eq!(bits(&a), bits(&c), "a freshly built equal tile must give the same bytes");

        // A different tile must give different bytes, or "deterministic" would
        // be satisfied by returning a constant.
        let mut d = tile(8);
        run(&mut d, &world_params());
        assert_ne!(bits(&a), bits(&d), "a different tile must erode to a different answer");
    }

    // -- every input reaches the output --------------------------------------

    /// `MISTAKES.md`'s "add a capability" rule applied literally: *derive the
    /// list from the definition — every argument of the function you are
    /// guarding — exercise each in turn, change each input and confirm the
    /// dependent thing reacts.*
    ///
    /// `tile_erode(refined, stress, resist, rain, w, h, p, refine,
    /// area_seed)`, all nine. `w`/`h` are exercised through the refusals below
    /// (a tile of a different shape is a different length, which is asserted),
    /// and `p` through each of its own live fields.
    #[test]
    fn every_input_of_tile_erode_reaches_its_output() {
        let base = tile(7);
        let f = fields();
        let seed = uniform_area_seed(W, H, REFINE);
        let go_r = |field: &Fields, p: &StreamPowerParams, refine: usize, s: &[f32]| {
            let mut t = base.clone();
            tile_erode(&mut t, &field.stress, &field.resist, &field.rain, W, H, p, refine, s);
            t
        };
        let go = |field: &Fields, p: &StreamPowerParams, s: &[f32]| go_r(field, p, REFINE, s);
        let control = go(&f, &world_params(), &seed);
        assert_ne!(bits(&control), bits(&base), "`refined` must reach the output");

        // `area_seed` -- the drainage-area boundary condition itself. A bigger
        // seed is a bigger `A`, a bigger `Cc` and therefore more erosion.
        let bigger = go(&f, &world_params(), &uniform_area_seed(W, H, 1));
        assert_ne!(bits(&control), bits(&bigger), "`area_seed` must reach the output");
        let move_of = |v: &[f32]| base.iter().zip(v).map(|(a, b)| (*a as f64 - *b as f64).abs()).sum::<f64>();
        assert!(
            move_of(&bigger) > move_of(&control),
            "seeding 64x the drainage area must erode harder: {} vs {}",
            move_of(&bigger),
            move_of(&control)
        );

        // `refine` -- the resolution correction, which reaches the output as
        // `k * refine` and nothing else. Asserted against an explicit `k`
        // rather than against `refine` itself: the two must be the SAME knob,
        // which is the claim the module header makes and the one a future
        // reader would break by deleting the multiply.
        let doubled = StreamPowerParams { k: world_params().k * 2.0, ..world_params() };
        assert_eq!(
            bits(&go_r(&f, &world_params(), REFINE * 2, &seed)),
            bits(&go_r(&f, &doubled, REFINE, &seed)),
            "doubling `refine` must be exactly doubling `k`"
        );
        assert_ne!(bits(&control), bits(&go_r(&f, &world_params(), REFINE * 2, &seed)), "`refine` must reach the output");
        assert_eq!(
            bits(&go_r(&f, &StreamPowerParams { k: world_params().k * REFINE as f64, ..world_params() }, 1, &seed)),
            bits(&control),
            "refine 1 must leave `k` alone, so the control is reproducible by hand"
        );

        // `rain` -- live through `climate_k`, which the world's default is
        // 0.5 rather than 0.
        let wet = Fields { rain: vec![3.0f32; W * H], ..fields() };
        assert_ne!(bits(&control), bits(&go(&wet, &world_params(), &seed)), "`rain` must reach the output");

        // `resist` -- live through `p.resist * 0.7 * resist[i]`.
        let hard = Fields { resist: vec![1.0f32; W * H], ..fields() };
        assert_ne!(bits(&control), bits(&go(&hard, &world_params(), &seed)), "`resist` must reach the output");

        // `stress` -- the uplift term's source, and **inert at the world's own
        // `uplift: 0.0`**, which is a property worth asserting in both
        // directions rather than leaving as a surprise.
        let stressed = Fields { stress: vec![0.4f32; W * H], ..fields() };
        assert_eq!(
            bits(&control),
            bits(&go(&stressed, &world_params(), &seed)),
            "at uplift 0 the whole uplift term is zero, so `stress` cannot matter"
        );
        let uplifting = StreamPowerParams { uplift: 0.02, ..world_params() };
        assert_ne!(
            bits(&go(&f, &uplifting, &seed)),
            bits(&go(&stressed, &uplifting, &seed)),
            "away from uplift 0, `stress` must reach the output"
        );

        // `p`'s own live fields, one at a time.
        for (name, p) in [
            ("iters", StreamPowerParams { iters: 30, ..world_params() }),
            ("k", StreamPowerParams { k: 0.05, ..world_params() }),
            ("g", StreamPowerParams { g: 2.0, ..world_params() }),
            ("deposit", StreamPowerParams { deposit: 0.0, ..world_params() }),
            ("climate_k", StreamPowerParams { climate_k: 0.0, ..world_params() }),
            ("resist", StreamPowerParams { resist: 0.9, ..world_params() }),
            ("uplift", uplifting),
            ("sea", StreamPowerParams { sea: 0.55, ..world_params() }),
        ] {
            assert_ne!(bits(&control), bits(&go(&f, &p, &seed)), "`p.{name}` must reach the output");
        }
    }

    // -- refusals ------------------------------------------------------------

    /// Each refusal is reachable from a caller that has assembled the tile
    /// wrongly, and each would otherwise be a panic deep inside the kernel
    /// after the expensive fill had already run — or, worse, a plausible
    /// answer over the wrong grid.
    #[test]
    fn a_mis_assembled_tile_is_refused_rather_than_computed() {
        let f = fields();
        let seed = uniform_area_seed(W, H, REFINE);
        let ok = std::panic::catch_unwind(|| {
            let mut t = tile(7);
            tile_erode(&mut t, &f.stress, &f.resist, &f.rain, W, H, &world_params(), REFINE, &seed);
        });
        assert!(ok.is_ok(), "the control must pass");

        // A wrapping world: the ring would stop being a ring.
        let f2 = fields();
        let wrapped = StreamPowerParams { world: true, ..world_params() };
        assert!(
            std::panic::catch_unwind(|| {
                let mut t = tile(7);
                tile_erode(&mut t, &f2.stress, &f2.resist, &f2.rain, W, H, &wrapped, REFINE, &uniform_area_seed(W, H, REFINE));
            })
            .is_err(),
            "p.world must be refused"
        );

        // A slice that does not match the grid, one at a time.
        for which in 0..5 {
            assert!(
                std::panic::catch_unwind(move || {
                    let n = W * H;
                    let short = |k: usize| vec![0.5f32; if which == k { n - 1 } else { n }];
                    let mut t = if which == 0 { vec![0.5f32; n - 1] } else { tile(7) };
                    tile_erode(&mut t, &short(1), &short(2), &short(3), W, H, &world_params(), REFINE, &short(4));
                })
                .is_err(),
                "a slice of the wrong length ({which}) must be refused"
            );
        }

        // A zero dimension.
        assert!(
            std::panic::catch_unwind(|| {
                tile_erode(&mut [], &[], &[], &[], 0, 4, &world_params(), REFINE, &[]);
            })
            .is_err(),
            "a zero dimension must be refused"
        );

        // `uniform_area_seed`'s own division.
        assert!(std::panic::catch_unwind(|| uniform_area_seed(4, 4, 0)).is_err(), "refine 0 must be refused");
    }

    /// [`uniform_area_seed`] is the unit convention, so it is asserted against
    /// literals rather than against the expression that produced it
    /// (`MISTAKES.md`: an assertion written against the very field it is
    /// checking holds for every value of that field).
    #[test]
    fn the_area_seed_is_one_coarse_cell_per_refine_squared_fine_cells() {
        assert_eq!(uniform_area_seed(3, 2, 1), vec![1.0f32; 6], "refine 1 is the world pass's own seed");
        assert_eq!(uniform_area_seed(2, 2, 2), vec![0.25f32; 4]);
        assert_eq!(uniform_area_seed(2, 2, 8), vec![0.015625f32; 4]);
        // The whole point of the convention, stated as the sum: one coarse
        // cell's worth of fine cells contributes exactly 1.0.
        for refine in [1usize, 2, 4, 8, 16] {
            let s = uniform_area_seed(refine, refine, refine);
            assert_eq!(s.iter().map(|v| *v as f64).sum::<f64>(), 1.0, "refine {refine}");
        }
    }

    /// The ring is the perimeter and nothing else — the corner cells included
    /// once each, and a three-cell axis leaving exactly one interior line.
    #[test]
    fn the_ring_mask_is_the_perimeter() {
        let m = ring_mask(4, 3);
        assert_eq!(
            m,
            vec![true, true, true, true, true, false, false, true, true, true, true, true],
            "a 4x3 ring leaves two interior cells"
        );
        assert_eq!(ring_mask(2, 9).iter().filter(|v| **v).count(), 18, "a two-wide tile is all ring");
        assert_eq!(ring_mask(40, 30).iter().filter(|v| **v).count(), 2 * (40 + 30) - 4);
    }
}
