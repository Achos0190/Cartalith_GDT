//! **`LOD_DETAIL_SCOPE.md` LOD-D6 — tile synthesis off the main thread, and
//! the per-quality-tier budgets that decide how much of it happens.**
//!
//! # What this module is for
//!
//! Until this milestone every deep-zoom tile was synthesised on the main
//! thread, inside `WorldGen::lod_synthesize_tile`, while Godot waited.
//! `viewport_host.gd` asks for up to a whole viewful on one zoom notch, and
//! the *context* those tiles share costs far more than the tiles themselves:
//! LOD-D2 measured `RenderCtx::with_appearance` + `with_map_scale` at
//! **201.5 ms** and `TileFields::new` at a further **278.2 ms** at 2048x1311,
//! against **5.98 ms** for one 256-px tile on all cores. LOD-D3's own report
//! recorded the consequence as a **132 ms -> 225 ms** worst camera step at LOD
//! entry, all of it the context build, and filed it as a D6 candidate.
//!
//! So there are two separate stalls to move, not one:
//!
//! 1. **The context build** (~480 ms, once per world-and-appearance).
//! 2. **The tiles** (~6 ms each, up to `tiles_per_update` per camera step).
//!
//! # Why a Rust-side pool and not `WorkerThreadPool`
//!
//! Godot's `WorkerThreadPool` takes a `Callable`. A `Callable` bound to a
//! `WorldGen` method would run `&self` Rust code on an engine worker thread —
//! and `WorldGen` holds a `RefCell` (`MISTAKES.md`'s own gdext rule aside,
//! `RefCell` is explicitly **not** `Sync`), plus `Gd<...>` handles which are
//! neither `Send` nor `Sync`. Calling into it from another thread is unsound
//! and nothing in the type system would stop it, because the call would come
//! back in through Godot's dynamic dispatch rather than through Rust.
//!
//! The scope's own wording is the design: *"Rust computes RGBA from a snapshot
//! of the height slice and an `Arc` of `TileFields`; the main thread uploads
//! the texture; a world version checked on landing drops stale tiles. **No
//! `Gd` crosses a thread.**"* That is what [`LodSnapshot`] and [`LodWorker`]
//! are. The only thing that crosses is an `Arc<LodWorker>`, which owns nothing
//! Godot knows about; the finished bytes come back as a plain `Vec<u8>` and
//! the `ImageTexture` is created on the main thread by the caller.
//!
//! `rayon` rather than raw `std::thread` because this crate already depends on
//! it (`TERRAIN_APPEARANCE_SCOPE.md` milestone 6) and because the tile
//! coloriser is itself `par_chunks_mut`-parallel inside — a job spawned into a
//! pool nests into that pool instead of fighting the global one.
//!
//! # Determinism
//!
//! The scope's parity class for this milestone is *determinism*: **a
//! worker-built tile is byte-identical to a main-thread one.** That is
//! structural here rather than tested-by-luck, and deliberately so
//! (`MISTAKES.md`: *"identity by control flow beats identity by arithmetic"*):
//! there is exactly one function that colours a tile, [`LodSnapshot::render_tile`],
//! and both `WorldGen::lod_tile_bytes` (main thread) and the worker jobs call
//! it on the same `Arc<LodSnapshot>`. `synthesize_tile_rgba`'s own internal
//! parallelism writes disjoint rows, so thread count does not enter the
//! arithmetic. `worker_and_main_thread_tiles_are_byte_identical` asserts it
//! anyway, because a structural argument that nothing exercises is prose.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cartalith_spatial::pyramid::ChunkId;

use crate::lod_bridge;
use crate::render::{
    self, ColorSpace, GridPrecompute, GroundTile, GroundTiles, QualityTier, RenderCtx, SplatChannel, SplatTextures, TerrainAppearance, TileCryo, TileFields,
};

// ---------------------------------------------------------------------------
// Per-tier budgets
// ---------------------------------------------------------------------------

/// **LOD-D6's *"the tier sets tile budget, maximum level and cache size"*.**
///
/// One struct rather than five `#[func]`s so the shell reads the whole policy
/// in one call and cannot pick up half of it. Every field is a literal in
/// [`budget_for_tier`] — `MISTAKES.md`'s rule for a test that pins a constant
/// is that it asserts a literal, and the same rule applies to the constant
/// itself: nothing here is derived from another tier's value, so mutating one
/// number moves exactly one test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LodBudget {
    /// How many *missing* tiles one `_update_lod()` call may request.
    /// `viewport_host.gd`'s `MAX_LOD_TILES_PER_UPDATE` was a flat 48.
    pub tiles_per_update: usize,
    /// How many backlog entries `_process()` drains per frame.
    /// `viewport_host.gd`'s `MAX_LOD_TILES_PER_CATCHUP` was a flat 6.
    pub tiles_per_catchup: usize,
    /// The deepest pyramid level this tier will enter. The scope's own
    /// example is *"Android Performance gets `MAX_LEVEL − 1`"*.
    pub max_level: i32,
    /// How many tiles the shell keeps alive after they leave the view, so
    /// panning back or re-entering a level costs no synthesis at all. Tiles
    /// are `TILE_PX`-square RGBA8, so what this authorises is
    /// `cache_tiles * 256 * 256 * 4` bytes — 8.4 MB at 32, 11.5 MB at 44.
    /// [`budget_for_tier`]'s own doc has the measurement it is sized against;
    /// it is not a taste.
    pub cache_tiles: usize,
    /// How many tile jobs may be outstanding at once. A cap rather than a
    /// pool size: the pool is process-wide and sized from the hardware, while
    /// *this* bounds how much tile RGBA is in flight (one job holds
    /// `256*256*4` = 262 kB of output) and how long a stale generation takes
    /// to drain after a world change.
    pub max_in_flight: usize,
    /// **EF-9.4's toggle**, beside the budget values it spends: whether the
    /// viewport draws importance-driven refinement
    /// (since 2026-10-18 the continuous effective level,
    /// `cartalith_engine::continuous_lod`: a per-pixel weight from importance
    /// plus zoom, with a `z0 + 1` layer over boosted ground) instead of one
    /// level for the whole view.
    ///
    /// **`true` on Balanced, Quality and Ultra; `false` on Performance.**
    /// Ruling CL (owner, 2026-10-18, `LARGE_ITEM_RULINGS.md`) ships EF-9 on
    /// by default, with the cheapest tier allowed to stay off if its frame
    /// budget needs it. The measured cost of "on" is about +0.9 ms per frame
    /// and +2.4 ms per camera step (medians, desktop, windowed;
    /// `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.5), and the `z0 + 1`
    /// layer more than doubles the live tiles at zoom 16 (105 against 41), each
    /// one synthesis and one texture. Desktop absorbs that. **Performance is
    /// the Android tier, and no phone timing exists**: §8.6 Q5's recorded
    /// default is that it stays off unless a phone shows no frame over one
    /// vsync per level change, and that measurement has not been taken, so
    /// Performance keeps the toggle off by that default (a labelled
    /// judgement, not a measurement). The shell may override it per session
    /// (`viewport_host.gd`'s `set_ef9_refine`), which is how the D0 harness
    /// measures both states.
    pub ef9_refine: bool,
}

/// The budget for one quality tier.
///
/// **`Quality` reproduces the pre-D6 constants exactly** (48 per update, 6 per
/// catch-up, no level cap), so the shipped desktop default schedules the same
/// work it scheduled before this milestone and any measured difference is the
/// threading, not a re-tuned budget. The other three tiers are new policy.
///
/// # Why `cache_tiles` is the size it is, which is not a taste
///
/// LOD-D6's memory bar is *"steady memory at most 60 MiB above the recorded
/// steady state"* = 62.91 MB, and the two things this milestone added to
/// steady memory were the snapshot's clone of the world and the shell's
/// parked tiles. **The first was measured, not estimated**: section 9 of
/// `_d6async_probe.gd` reported `retained_bytes` at 2048x1311 as **51.21 MB**
/// (this machine, 2026-09-21; the probe prints the figure so it can be
/// re-taken rather than remembered). That left **11.7 MB**, and a parked tile
/// is `TILE_PX`-square RGBA8 = 262 144 B, so **44 tiles** was the whole
/// remaining budget at that grid.
///
/// **43.0 MB of that 51.21 MB is no longer copied, and the table below was
/// NOT re-tuned for it.** `WorldState`'s four grids became `Arc<Vec<f32>>`,
/// so a snapshot takes a refcount on each instead of a copy. The in-app
/// figure has NOT been re-taken — that needs the probe and a rebuilt
/// `cartalith_godot.dll`. What *was* measured is the clone shape itself, at
/// the same 2 684 928 cells: a host-polled peak working set of 89 862 144 B
/// for four deep clones against 46 886 912 B for four `Arc` clones, 5 runs
/// each, spread under 0.01 MB — a **42 975 232 B** difference, which is the
/// four grids plus 16 KB. That is the mechanism, not this struct.
///
/// The budgets stay exactly where LOD-D6 set them, deliberately: `Quality`
/// must keep scheduling the pre-D6 shell's constants or a measured difference
/// cannot be attributed to the threading, and spending the new headroom is a
/// separate decision with its own measurement to take. **The bar assertion
/// below therefore still uses the PRE-`Arc` snapshot size** — an upper bound
/// now, so the guard is conservative and still refuses a `cache_tiles` rise.
///
/// The first draft of this table had `Quality` at 96 and `Ultra` at 128, which
/// is 25.2 MB and 33.6 MB — 76.4 MB and 84.8 MB of D6 delta, comfortably over
/// the bar this milestone is graded against, and nothing would have failed.
/// The second draft put `Quality` at 40 and `Ultra` at 44, and the test below
/// caught THAT too: 62 914 560 − 53 698 560 is 9 216 000 B, which is **35
/// tiles**, not 44. Both errors were arithmetic done in prose; the assertion
/// is what found them, which is why it is an assertion.
///
/// **So the ladder is nearly flat, and that is the honest shape of it.** The
/// snapshot takes 81% of the bar on its own, so how many tiles a tier may park
/// is not really a tier decision at this grid — the lever that matters is the
/// snapshot, and the way to move it is to stop cloning the world's four fields
/// (make `WorldState`'s `Vec<f32>`s `Arc<Vec<f32>>` at the source, a
/// cross-crate change this milestone did not take). Until then, a bigger tile
/// cache is not available to spend on.
///
/// `Performance` is **32 by the scope's own words** (*"Android Performance
/// gets `MAX_LEVEL − 1` and a 32-tile cache"*), not by this arithmetic — it
/// happens to fit, which is what made the number worth keeping rather than
/// re-deriving.
pub fn budget_for_tier(tier: QualityTier) -> LodBudget {
    match tier {
        QualityTier::Performance => LodBudget { tiles_per_update: 16, tiles_per_catchup: 3, max_level: lod_bridge::MAX_LEVEL - 1, cache_tiles: 32, max_in_flight: 2, ef9_refine: false },
        QualityTier::Balanced => LodBudget { tiles_per_update: 32, tiles_per_catchup: 4, max_level: lod_bridge::MAX_LEVEL, cache_tiles: 32, max_in_flight: 3, ef9_refine: true },
        QualityTier::Quality => LodBudget { tiles_per_update: 48, tiles_per_catchup: 6, max_level: lod_bridge::MAX_LEVEL, cache_tiles: 35, max_in_flight: 4, ef9_refine: true },
        QualityTier::Ultra => LodBudget { tiles_per_update: 48, tiles_per_catchup: 8, max_level: lod_bridge::MAX_LEVEL, cache_tiles: 35, max_in_flight: 6, ef9_refine: true },
    }
}

// ---------------------------------------------------------------------------
// The snapshot — everything a tile needs, owned, `Send + Sync`
// ---------------------------------------------------------------------------

/// A loaded save's river flag (`render::save_flag_at`), owned rather than
/// borrowed: a background thread cannot borrow from `WorldGen`, so the
/// snapshot owns the bytes. RV-5 (2026-09-28) retired the other arm this
/// type had, `Stamped` (the export-only disc raster): a generated world's
/// tiles and exports draw vector rivers, and only a save without a traced
/// network still carries a flag.
pub type OwnedInk = Vec<u8>;

/// A loaded pack's six splat channels, owned. Mirrors
/// [`render::SplatTextures`] field for field so the two cannot drift.
#[derive(Default)]
pub struct OwnedSplat {
    pub grass: Option<SplatChannel>,
    pub rock: Option<SplatChannel>,
    pub sand: Option<SplatChannel>,
    pub snow: Option<SplatChannel>,
    pub wetland: Option<SplatChannel>,
    pub canopy: Option<SplatChannel>,
}

impl OwnedSplat {
    /// Borrows the owned channels as the `render::SplatTextures` view the tile
    /// coloriser takes. Pure re-borrow; it must never copy a channel.
    pub(crate) fn as_textures(&self) -> SplatTextures<'_> {
        SplatTextures {
            grass: self.grass.as_ref(),
            rock: self.rock.as_ref(),
            sand: self.sand.as_ref(),
            snow: self.snow.as_ref(),
            wetland: self.wetland.as_ref(),
            canopy: self.canopy.as_ref(),
        }
    }
}

/// The four tectonic-substrate fields `cartalith_civ::build_lithology` needs.
///
/// **Transient, and that is the point.** They are shared into
/// [`SnapshotInputs`], consumed by [`LodSnapshot::build`] and dropped there —
/// the finished `Vec<u8>` lithology is a quarter the width of one of them and
/// is all that is retained. `None` for a loaded save, whose format stores none
/// of this (`SAVEFILE_COMPAT.md`), which is the same condition under which
/// `flow` is `None`.
///
/// **`Arc`, not `Vec` — four refcount bumps, not four memcpys.** These four
/// were plain `Vec<f32>` and were deep-cloned out of `WorldState` on every
/// snapshot: 10.74 MB each at 2 048 × 1 311, **43.0 MB** of transient peak,
/// which is what [`SnapshotInputs`]' own doc flagged as the follow-on to the
/// four world grids. Because the build drops them, this never appeared in
/// [`LodSnapshot::retained_bytes`] and never will — the win is in the peak,
/// not in steady residency, and the peak is what a small device runs out of.
/// `cartalith_civ::build_lithology` takes `&[f32]`, which `&Arc<Vec<f32>>`
/// still coerces to, so the consuming call is unchanged.
pub struct LithoSource {
    pub age: Arc<Vec<f32>>,
    pub volcanic: Arc<Vec<f32>>,
    pub crust: Arc<Vec<f32>>,
    pub resistance: Arc<Vec<f32>>,
}

/// Everything [`LodSnapshot::build`] consumes — assembled on the main thread
/// (it reads `WorldGen`), moved to a worker, and dropped there.
///
/// **The four world grids are no longer copied.** They were, and LOD-D6
/// stated it as this milestone's cost: at 2048x1311 (2 684 928 cells)
/// `field`/`temperature`/`rainfall`/`flow` are 10.74 MB each — **43.0 MB** —
/// and the fix was deferred because it reaches into `cartalith-engine`. It
/// has since been taken: those four are `Arc<Vec<f32>>` on
/// [`cartalith_engine::WorldState`], so a snapshot's copy of them is four
/// refcount bumps. **A *loaded* save's three are refcount bumps too now** —
/// `cartalith_io::SaveFields::heightmap`/`temperature`/`rainfall` are
/// `Arc<Vec<f32>>`, the same shape as `WorldState`'s. This used to be an
/// `Arc::new` of a fresh clone on every snapshot, since the save format owned
/// plain `Vec`s.
///
/// **[`LithoSource`]'s four are no longer copied either.** They were the
/// follow-on this doc named — `age_field`/`volcanic_field`/`crust_field`/
/// `resistance_field` on `WorldState`, another 43.0 MB, transient for the
/// duration of one build rather than retained. All four are `Arc<Vec<f32>>`
/// on `WorldState` now, so `litho` costs four refcount bumps as well.
///
/// What is still copied: a loaded save's river flag (2.68 MB at 2048x1311), the
/// paint grids, and a pack's splat and ground textures. A *loaded* save's
/// three fields are refcount bumps too now, the same as a generated world's.
pub struct SnapshotInputs {
    pub key: String,
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    pub world: bool,
    pub lat_n: f64,
    pub lat_s: f64,
    pub map_width_km: f64,
    pub seed: i32,
    /// `Arc`, so the four `WorldState` grids cross to the worker as refcount
    /// bumps rather than as 43.0 MB of memcpy. See this struct's own doc.
    pub field: Arc<Vec<f32>>,
    /// **RV-3: the height a tile is SHADED from** (`WorldGen::
    /// valley_shade_field`: the carve filled in, a valley cut along each drawn
    /// river line). `None` -- the reference's look, a loaded save, a world
    /// with no carve -- shades `field` itself. `field` above stays the world's
    /// height and alone decides water (`RenderCtx::water_height`), so the
    /// valley can never add or remove a lake in a tile.
    pub shade: Option<Arc<Vec<f32>>>,
    pub temperature: Arc<Vec<f32>>,
    pub rainfall: Arc<Vec<f32>>,
    pub flow: Option<Arc<Vec<f32>>>,
    pub litho: Option<LithoSource>,
    pub appearance: TerrainAppearance,
    pub color_space: ColorSpace,
    pub ink: Option<OwnedInk>,
    pub splat: Option<OwnedSplat>,
    pub ground_biomes: Vec<Option<GroundTile>>,
    pub ground_terrains: Vec<Option<GroundTile>>,
    /// `true` when a `PaintEditor` exists at all, mirroring the `if let
    /// Some(p) = self.paint` branch the pre-D6 `lod_tile_bytes` took — the
    /// three grids below are each `None` until their layer is committed, and
    /// `with_paint(None, None, None)` is already the no-paint picture, so this
    /// flag changes no pixel. It exists so the two paths have the *same*
    /// control flow and not merely the same output.
    pub paint_present: bool,
    pub paint_biome: Option<Vec<u8>>,
    pub paint_terrain: Option<Vec<u8>>,
    pub paint_splat: Option<Vec<u8>>,
    pub grid_rgb: Option<Vec<u8>>,
    pub glacial_snowline: f64,
    pub peak_m: f64,
    pub lapse_rate: f64,
    pub gravity: f64,
    /// The drawn river network (`WorldGen::river_geometry`), rasterized into
    /// every tile at the tile's own resolution
    /// (`lod_bridge::synthesize_tile_rgba_rivers`). `None` with the Rivers
    /// layer off, for a loaded save, and before any rivers were traced. An
    /// `Arc` because the geometry is built once per snapshot and read by
    /// every worker tile.
    pub rivers: Option<Arc<crate::river_stroke::RiverGeometry>>,
    /// **RIM-7**: RIM-4's delta fans for this look (`WorldGen::delta_fans_for`,
    /// gated by `river_delta::fans_drawn`), stroked before [`Self::rivers`] in
    /// every tile (`river_stroke::rasterize_with`), so the delta the painted
    /// screen path draws below the deep-zoom switch is still there above it.
    /// A second `Arc` beside the network rather than runs copied into it, so
    /// nothing copies the network. `None` -- never an empty geometry -- with
    /// the switch off, a look that draws no fan, the Rivers layer off, or a
    /// world with no fan; [`Self::fingerprint`] then reads exactly what it
    /// read before RIM-7.
    pub river_fans: Option<Arc<crate::river_stroke::RiverGeometry>>,
    /// **RIM-7, the painted river** (`WorldGen::river_paint_source`): for a
    /// look that paints its rivers into the map (`river_field::painted`), the
    /// network and its fans indexed with the screen's river colour field, so
    /// every tile paints the river by the screen's own law
    /// (`river_field::PaintSource::raster`) instead of stroking
    /// [`Self::rivers`] -- one width taper, colour, floodplain, bank line and
    /// headwater fade across the deep-zoom switch. Built from the same
    /// [`Self::rivers`] and [`Self::river_fans`] (which stay the hashed inputs).
    /// `None` -- the stroke, exactly as before -- for a look that strokes, the
    /// Rivers layer off, a grid over the field's budget, or no network.
    pub river_paint: Option<Arc<crate::river_field::PaintSource>>,
    /// Ruling BO: the forced-lake mask (`WorldGen::forced_lake_mask`), drawn
    /// into every tile as lake (`TileFields::with_forced_lakes`) so a tile
    /// shows the water the base map shows. `None` -- the absent state, never
    /// an all-zero mask -- for a world where nothing was forced, which keeps
    /// its tiles byte-identical to before the field existed.
    pub forced_lakes: Option<Vec<u8>>,
    /// **GF-6 §4.10**: the exposed rock per cell for rock-styled deep-zoom
    /// detail (`render::RockDetail`), attached to every tile's context
    /// (`RenderCtx::with_rock_detail`). `None` -- the absent state, never a
    /// guessed rock -- for a world whose landform processes do not read rock
    /// (the app's default), a loaded save and an import; the tiles are then
    /// byte-identical to before the field existed, and [`Self::fingerprint`]
    /// reads exactly what it read before.
    pub rock_detail: Option<Arc<render::RockDetail>>,
    /// Ruling CN's colour-paint layer (`COLOUR_PAINT_SCOPE.md` CP-2), attached
    /// to every tile's context (`RenderCtx::with_colour_paint`). `None` -- no
    /// layer, or a hidden or empty one (`WorldGen::colour_paint_for_snapshot`)
    /// -- attaches nothing and fingerprints exactly as before CP-2.
    pub colour_paint: Option<Arc<cartalith_spatial::colour_paint::ColourPaint>>,
}

/// The word [`SnapshotInputs::fingerprint`] folds in when the tiles paint the
/// river (`river_paint`). **Labelled judgement**: any fixed word does; this
/// one spells `RIM7PNT\0` in ASCII so it is recognisable in a stream dump.
const PAINTED_RIVERS_TAG: u64 = 0x0054_4e50_374d_4952;

/// The separator [`SnapshotInputs::fingerprint`] folds in before a
/// [`SnapshotInputs::rock_detail`]. **Labelled judgement**: any fixed word
/// does; this one spells `GF6ROCK ` in ASCII.
const ROCK_DETAIL_TAG: u64 = 0x004b_434f_5236_4647;

/// The separator [`SnapshotInputs::fingerprint`] folds in before a
/// [`SnapshotInputs::colour_paint`] document. **Labelled judgement**: any
/// fixed word does; this one spells `NPAINT ` and a space in ASCII.
const COLOUR_PAINT_TAG: u64 = 0x0020_544e_4941_504e;

/// A word-at-a-time FNV-1a-style digest, for [`SnapshotInputs::fingerprint`].
///
/// Word-wise rather than the byte-wise FNV-1a `lod_source_key` uses because
/// it runs over every grid a tile reads — ~100 MB at 2048x1311 with a
/// lithology — and eight times fewer multiplies is the difference between
/// noise and a visible slice of the ~480 ms snapshot build. It guards against
/// a *changed* input, not a forged one: each step (`xor` a word, multiply by
/// an odd constant) is a bijection on the state, so any change confined to
/// one word always changes the result.
///
/// Under `cfg(test)` it also records every word it was fed (`log`), so a test
/// can pin the STRUCTURE of the stream -- a separator no input can collide
/// with is invisible to any digest comparison (RIM-7's survivor), but not to
/// the stream. The digest value is the same with or without the record.
struct Digest {
    h: u64,
    #[cfg(test)]
    log: Vec<u64>,
}

impl Digest {
    /// A digest seeded with the 64-bit FNV offset basis `0xcbf29ce484222325`
    /// (FNV reference constant), with an empty test log under `cfg(test)`.
    fn new() -> Self {
        Digest {
            h: 0xcbf2_9ce4_8422_2325,
            #[cfg(test)]
            log: Vec::new(),
        }
    }
    /// Folds one 64-bit word into the state: xor, then multiply by the 64-bit
    /// FNV prime `0x100000001b3` (FNV reference constant). Every other `Digest`
    /// method reduces to this, so the structure of the stream is a sequence of
    /// `word` calls.
    fn word(&mut self, w: u64) {
        self.h = (self.h ^ w).wrapping_mul(0x0000_0100_0000_01b3);
        #[cfg(test)]
        self.log.push(w);
    }
    /// Folds a float by its exact bit pattern, so `-0.0` and `0.0`, or two NaN
    /// payloads, are distinct inputs; a changed value always changes the digest.
    fn f64(&mut self, v: f64) {
        self.word(v.to_bits());
    }
    /// Length-prefixed, so `[a, b] + []` and `[a] + [b]` differ.
    fn bytes(&mut self, b: &[u8]) {
        self.word(b.len() as u64);
        let mut it = b.chunks_exact(8);
        for c in &mut it {
            let mut w = [0u8; 8];
            w.copy_from_slice(c);
            self.word(u64::from_le_bytes(w));
        }
        let rem = it.remainder();
        let mut w = [0u8; 8];
        w[..rem.len()].copy_from_slice(rem);
        self.word(u64::from_le_bytes(w));
    }
    /// Two `f32` bit patterns per word, length-prefixed.
    fn f32s(&mut self, v: &[f32]) {
        self.word(v.len() as u64);
        let mut it = v.chunks_exact(2);
        for c in &mut it {
            self.word(c[0].to_bits() as u64 | ((c[1].to_bits() as u64) << 32));
        }
        if let [x] = it.remainder() {
            self.word(x.to_bits() as u64);
        }
    }
    /// Folds an is-present flag (1 or 0) so an absent optional input and a present
    /// but empty one cannot hash alike.
    fn tag(&mut self, present: bool) {
        self.word(if present { 1 } else { 0 });
    }
}

impl SnapshotInputs {
    /// A digest of **everything a tile is a function of** except its own
    /// address — the half of a stored pyramid's validity that the archive's
    /// `source_key` cannot give (owner rulings 28/29's read path).
    ///
    /// # Why the archive key is not enough
    ///
    /// `cartalith_io::project::lod_source_key` covers the heightmap, the grid,
    /// the seed and the sea level. A tile also reads the appearance, the
    /// **colour space** (`OUTSTANDING_WORK.md`'s *"stored LOD pyramids carry
    /// no colour space"*), the climate grids, the flow field, the lithology
    /// substrate, the river ink, the pack's splat and ground tiles, the paint
    /// layers, the finished grid raster local contrast is measured from, and
    /// four cryosphere parameters. A pyramid stored under one set of those
    /// and seeded under another would draw last week's look over this week's
    /// map, and a **generated** world and the same world **reopened** differ
    /// in two of them by construction (`lod_snapshot_inputs`' `Loaded` arm
    /// passes no flow and no lithology), so this is not a theoretical gap.
    ///
    /// # Derived from the definition
    ///
    /// The destructuring below names every field of [`SnapshotInputs`] and
    /// binds `key` to `_`: a field added to the struct is a compile error
    /// here until someone decides whether it is a tile input
    /// (`MISTAKES.md`: *"derive the list from the definition"*). `key` is
    /// deliberately excluded — it carries `world_epoch` and stage versions,
    /// which are per-session counters and would make every reopened
    /// project's key differ from its save's.
    ///
    /// Two inputs are folded in only where a tile can read them, mirroring
    /// the gates the consumers apply, so their *presence* cannot refuse a
    /// valid pyramid: `grid_rgb` only when `TileFields::new` would measure a
    /// detail band from it (`local_contrast > 0` and a full-length buffer —
    /// whether it is present at all depends on whether the base map has been
    /// drawn yet, which is timing, not world), and `glacial_snowline` only
    /// when `ice_strength > 0` (the same gate `WorldGen::lod_cache_key`
    /// applies, for the reason its comment gives).
    pub fn fingerprint(&self) -> u64 {
        self.digest().h
    }

    /// [`Self::fingerprint`]'s digest, its record included under `cfg(test)`.
    fn digest(&self) -> Digest {
        let SnapshotInputs {
            key: _,
            gw,
            gh,
            sea_level,
            world,
            lat_n,
            lat_s,
            map_width_km,
            seed,
            field,
            shade,
            temperature,
            rainfall,
            flow,
            litho,
            appearance,
            color_space,
            ink,
            splat,
            ground_biomes,
            ground_terrains,
            paint_present,
            paint_biome,
            paint_terrain,
            paint_splat,
            grid_rgb,
            glacial_snowline,
            peak_m,
            lapse_rate,
            gravity,
            rivers,
            river_fans,
            river_paint,
            forced_lakes,
            rock_detail,
            colour_paint,
        } = self;
        let mut d = Digest::new();
        d.word(*gw as u64);
        d.word(*gh as u64);
        d.f64(*sea_level);
        d.tag(*world);
        d.f64(*lat_n);
        d.f64(*lat_s);
        d.f64(*map_width_km);
        d.word(*seed as i64 as u64);
        d.f32s(field);
        // RV-3: a stored pyramid shaded from another valley field is not this
        // one's.
        d.tag(shade.is_some());
        if let Some(s) = shade {
            d.f32s(s);
        }
        d.f32s(temperature);
        d.f32s(rainfall);
        d.tag(flow.is_some());
        if let Some(f) = flow {
            d.f32s(f);
        }
        d.tag(litho.is_some());
        if let Some(l) = litho {
            for g in [&l.age, &l.volcanic, &l.crust, &l.resistance] {
                d.f32s(g);
            }
        }
        d.word(lod_bridge::appearance_fingerprint(appearance));
        // The rivers drawn into the tiles: whether any are, and every
        // point, width, colour and piece of them, so a pyramid stored with
        // the layer on is not seeded under it off, or over another network.
        // Ruling BO: whether any lake was forced, and which cells -- a
        // pyramid stored before a press must not seed the world after it.
        d.tag(forced_lakes.is_some());
        if let Some(m) = forced_lakes {
            d.bytes(m);
        }
        // GF-6 §4.10: the rock the tiles are styled by. Only when present (no
        // word when absent), so every world without it -- the app's default --
        // fingerprints exactly as before; with it, a separator word
        // (`ROCK_DETAIL_TAG`) then every cell byte and the metres per unit, so
        // a stored unstyled pyramid never seeds a styled world or the reverse.
        if let Some(rd) = rock_detail {
            d.word(ROCK_DETAIL_TAG);
            d.bytes(&rd.cells);
            d.f64(rd.m_per_unit);
        }
        // Ruling CN's colour paint: only when present (no word when absent,
        // so every snapshot without paint fingerprints as before CP-2); with
        // it, a separator then the whole saved document, so a pyramid stored
        // under one set of marks never seeds another.
        if let Some(cp) = colour_paint {
            d.word(COLOUR_PAINT_TAG);
            d.bytes(serde_json::to_string(cp.doc()).unwrap_or_default().as_bytes());
        }
        d.tag(rivers.is_some());
        if let Some(g) = rivers {
            d.word(g.runs.len() as u64);
            for r in &g.runs {
                d.f32s(&r.pts.iter().flat_map(|p| [p.0, p.1]).collect::<Vec<f32>>());
                d.f32s(&r.widths);
                d.f32s(&r.colors.iter().flat_map(|c| *c).collect::<Vec<f32>>());
                d.word(r.pieces.len() as u64);
                for &(a, b) in &r.pieces {
                    d.word(((a as u64) << 32) | b as u64);
                }
                d.word(r.orders.len() as u64);
                for o in &r.orders {
                    d.word(*o as u16 as u64);
                }
                d.f32s(&r.discharge);
                d.word(r.own_order as u16 as u64);
            }
        }
        // RIM-7: the delta fans, hashed run by run as the network is. Only
        // when present (no tag word when absent), so a snapshot with no fans
        // -- the switch off, or a world without one -- fingerprints exactly as
        // before RIM-7 and a pyramid stored then still seeds. With fans, the
        // extra words (every fan run, as the network's are hashed) make the
        // digest differ, so a stored fan-OFF pyramid never seeds a fan-ON
        // world or the reverse (`a_snapshot_with_delta_fans_draws_them_and_keys_them`).
        // The leading `u64::MAX` is a separator between the network's words
        // and the fans'; `the_fans_are_hashed_behind_a_separator` reads the
        // digest's test-only word log and fails if it is removed.
        if let Some(g) = river_fans {
            d.word(u64::MAX);
            d.word(g.runs.len() as u64);
            for r in &g.runs {
                d.f32s(&r.pts.iter().flat_map(|p| [p.0, p.1]).collect::<Vec<f32>>());
                d.f32s(&r.widths);
                d.f32s(&r.colors.iter().flat_map(|c| *c).collect::<Vec<f32>>());
                d.word(r.pieces.len() as u64);
                for &(a, b) in &r.pieces {
                    d.word(((a as u64) << 32) | b as u64);
                }
                d.word(r.orders.len() as u64);
                for o in &r.orders {
                    d.word(*o as u16 as u64);
                }
                d.f32s(&r.discharge);
                d.word(r.own_order as u16 as u64);
            }
        }
        // RIM-7, the painted river: whether the tiles PAINT the network
        // (`river_field::PaintSource`) rather than stroke it. Every input the
        // painting reads is already above -- the network, the fans, the
        // appearance (width, colour, outline, frame, the look's gate) and the
        // grid -- but the pixels drawn from them changed with the painting, so
        // a pyramid stored while these looks stroked must not seed a world that
        // paints. Only when painted (no word otherwise), so every stroked
        // snapshot fingerprints exactly as before.
        if river_paint.is_some() {
            d.word(PAINTED_RIVERS_TAG);
        }
        // Exhaustive, so a third colour space is a compile error here rather
        // than a silent alias of one of these two.
        d.word(match color_space {
            ColorSpace::Srgb => 0,
            ColorSpace::DisplayP3 => 1,
        });
        // `2` is the flag's tag from before RV-5 retired the stamp (`1`),
        // kept so a key's shape is unchanged for the one variant left.
        match ink {
            None => d.word(0),
            Some(v) => {
                d.word(2);
                d.bytes(v);
            }
        }
        d.tag(splat.is_some());
        if let Some(s) = splat {
            for c in [&s.grass, &s.rock, &s.sand, &s.snow, &s.wetland, &s.canopy] {
                d.tag(c.is_some());
                if let Some(c) = c {
                    d.word(c.w as u64);
                    d.word(c.h as u64);
                    d.bytes(&c.rgba);
                    for v in c.inv {
                        d.f64(v);
                    }
                }
            }
        }
        for family in [ground_biomes, ground_terrains] {
            d.word(family.len() as u64);
            for t in family {
                d.tag(t.is_some());
                if let Some(t) = t {
                    d.word(t.w as u64);
                    d.word(t.h as u64);
                    d.bytes(&t.rgba);
                }
            }
        }
        d.tag(*paint_present);
        for layer in [paint_biome, paint_terrain, paint_splat] {
            d.tag(layer.is_some());
            if let Some(v) = layer {
                d.bytes(v);
            }
        }
        let band = grid_rgb.as_ref().filter(|b| appearance.local_contrast > 0.0 && b.len() >= gw * gh * 3);
        d.tag(band.is_some());
        if let Some(b) = band {
            d.bytes(b);
        }
        d.f64(if appearance.ice_strength > 0.0 { *glacial_snowline } else { 0.0 });
        d.f64(*peak_m);
        d.f64(*lapse_rate);
        d.f64(*gravity);
        d
    }
}

/// LOD-D2's `LodCtxCache`, made **owned and shareable** so a worker thread can
/// colour a tile from it.
///
/// It holds the same three expensive things that cache held — the
/// `GridPrecompute`, the lithology and the `TileFields` — plus the world
/// slices a `RenderCtx` used to borrow from `WorldGen`. Held behind an `Arc`,
/// so the main thread and every worker share one copy.
pub struct LodSnapshot {
    key: String,
    /// The producer string a pyramid stored from this snapshot carries, and
    /// the one a seeded pyramid must match exactly — see
    /// [`LodSnapshot::producer_id`].
    producer: String,
    gw: usize,
    gh: usize,
    sea_level: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    seed: i32,
    /// The world's height: decides water ([`SnapshotInputs::field`]).
    field: Arc<Vec<f32>>,
    /// RV-3's shaded height ([`SnapshotInputs::shade`]); `None` shades `field`.
    shade: Option<Arc<Vec<f32>>>,
    temperature: Arc<Vec<f32>>,
    rainfall: Arc<Vec<f32>>,
    flow: Option<Arc<Vec<f32>>>,
    appearance: TerrainAppearance,
    color_space: ColorSpace,
    pre: GridPrecompute,
    lithology: Option<Vec<u8>>,
    /// GF-6 §4.10's rock styling input, carried from
    /// [`SnapshotInputs::rock_detail`] unchanged.
    rock_detail: Option<Arc<render::RockDetail>>,
    /// Ruling CN's colour paint, carried from [`SnapshotInputs::colour_paint`].
    colour_paint: Option<Arc<cartalith_spatial::colour_paint::ColourPaint>>,
    /// Ruling CO item 3: the world's edifices recovered from its
    /// `volcanic_field` (`SnapshotInputs::litho`), for the tiles' volcano
    /// furrows (`render::add_tile_furrows`). Built only when the look asks
    /// for furrows (`TerrainAppearance::edifice_furrows > 0`) and a
    /// generated world supplied the field; `None` otherwise (a loaded save,
    /// the parity look), which draws none. Derived from inputs already in
    /// [`SnapshotInputs::fingerprint`] (the volcanic grid and the
    /// appearance), so it adds no cache-key word.
    edifice_frame: Option<cartalith_terrain::edifice_pits::EdificeFrame>,
    fields: TileFields<'static>,
    ink: Option<OwnedInk>,
    splat: Option<OwnedSplat>,
    ground_biomes: Vec<Option<GroundTile>>,
    ground_terrains: Vec<Option<GroundTile>>,
    paint_present: bool,
    paint_biome: Option<Vec<u8>>,
    paint_terrain: Option<Vec<u8>>,
    paint_splat: Option<Vec<u8>>,
    rivers: Option<Arc<crate::river_stroke::RiverGeometry>>,
    /// RIM-7: [`SnapshotInputs::river_fans`], stroked before `rivers`.
    river_fans: Option<Arc<crate::river_stroke::RiverGeometry>>,
    /// RIM-7: [`SnapshotInputs::river_paint`]; when present the tiles paint
    /// the river with it instead of stroking `rivers` and `river_fans`.
    river_paint: Option<Arc<crate::river_field::PaintSource>>,
    /// LOD-D7: the generation parameters the info tiles' slope scale needs and
    /// the terrain tile does not keep ([`SnapshotInputs::peak_m`],
    /// [`SnapshotInputs::map_width_km`]).
    peak_m: f64,
    map_width_km: f64,
    /// LOD-D7: crust age and rock resistance, the two lithology rasters the
    /// Age and Resistance info views read. `Arc` clones of
    /// [`SnapshotInputs::litho`]'s, so they add no steady memory; `None` for a
    /// loaded save, which stores no tectonic substrate (the views are then
    /// "not available", never a flat colour).
    age: Option<Arc<Vec<f32>>>,
    resistance: Option<Arc<Vec<f32>>>,
    /// LOD-D7: the River-flow view's whole-map normaliser
    /// (`sample_bridge::flow_log_max`), scanned ONCE here so every tile shares
    /// one scale and no edge seams. The `1e-6` floor when there is no flow.
    flow_log_max: f64,
    /// EF-9.4's colour half (2026-10-18): the last few PLAIN tiles this
    /// snapshot coloured as a conformed tile's parent, `(z, col, row) ->
    /// (rgba, w, h)`, newest last. A
    /// conformed tile cross-fades toward its plain parent
    /// (`lod_bridge::synthesize_tile_rgba_rivers_conformed_with_parent`), and
    /// the shell asks for that same plain tile while the region is incomplete
    /// (`viewport_host.gd`'s hold rule), so one synthesis serves the region's
    /// four children and the shell. Per snapshot, so a world or look change
    /// starts empty -- a cached tile is always this snapshot's own output, never
    /// a stale one. Capacity [`PLAIN_TILE_CACHE`].
    plain_rgba: Mutex<std::collections::VecDeque<((i32, i32, i32), Arc<(Vec<u8>, usize, usize)>)>>,
}

/// How many plain tiles [`LodSnapshot`] keeps for the colour half. A labelled
/// judgement, not a measurement: one region needs one parent, and at most
/// `LodBudget::max_in_flight` (6 at Ultra) syntheses run at once, so 8 covers
/// every in-flight region with one to spare. 8 x 256 x 256 x 4 B = 2 MiB per
/// snapshot at most, outside `cache_tiles`' budget (that guard counts the
/// shell's textures). Only a conformed synthesis fills it, so with EF-9 off it
/// stays empty and costs nothing.
pub const PLAIN_TILE_CACHE: usize = 8;

/// The whole safety argument for this module in one line the compiler checks.
/// If a future field makes `LodSnapshot` un-`Send`, this fails to compile
/// rather than being discovered on a device.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LodSnapshot>();
    assert_send_sync::<SnapshotInputs>();
};

impl LodSnapshot {
    /// The pre-D6 `WorldGen::build_lod_cache`, moved here verbatim in the
    /// arithmetic it performs and in the order it performs it.
    ///
    /// `None` on the same conditions that function returned `None`: a
    /// degenerate grid, a short height field, or a `RenderCtx` the
    /// precompute does not match.
    pub fn build(i: SnapshotInputs) -> Option<LodSnapshot> {
        let producer = format!("{};in={:016x}", lod_bridge::tile_producer_id(&i.appearance), i.fingerprint());
        let SnapshotInputs {
            key,
            gw,
            gh,
            sea_level,
            world,
            lat_n,
            lat_s,
            map_width_km,
            seed,
            field,
            shade,
            temperature,
            rainfall,
            flow,
            litho,
            appearance,
            color_space,
            ink,
            splat,
            ground_biomes,
            ground_terrains,
            paint_present,
            paint_biome,
            paint_terrain,
            paint_splat,
            grid_rgb,
            glacial_snowline,
            peak_m,
            lapse_rate,
            gravity,
            rivers,
            river_fans,
            river_paint,
            forced_lakes,
            rock_detail,
            colour_paint,
        } = i;
        if gw < 2 || gh < 2 || field.len() < gw.checked_mul(gh)? {
            return None;
        }
        // RV-3: a shaded field of the wrong length is dropped, and the tile
        // shades the world's field -- never indexes past a short grid.
        let shade = shade.filter(|s| s.len() == field.len());
        // The height the tile is shaded from (relief rasters, rock, ice, the
        // amplified tile surface); `field` still decides water.
        let shaded: &[f32] = shade.as_deref().map_or(&field, |s| s.as_slice());
        // Ruling BO: the forced lakes reach the grid's `sdf_biomes` band, as
        // on screen (`RenderCtx::with_map_scale_forced`); already in
        // `fingerprint()`, so no cache key changes.
        let pre = GridPrecompute::build_split(shaded, &field, &temperature, &rainfall, flow.as_ref().map(|v| v.as_slice()), gw, gh, sea_level, world, &appearance, Some(map_width_km), forced_lakes.as_deref());
        // The same call `build_color_texture` makes, and `None` under the same
        // condition: a loaded save's format stores none of the tectonic
        // substrate this needs (`SAVEFILE_COMPAT.md`), which is why its
        // caller hands us no `LithoSource` at all.
        let lithology = litho.as_ref().map(|l| cartalith_civ::build_lithology(shaded, &l.age, &l.volcanic, &l.crust, &l.resistance, &rainfall, sea_level));
        // LOD-D7: the two rasters the info views read, kept as `Arc` clones.
        let edifice_frame = litho
            .as_ref()
            .filter(|l| appearance.edifice_furrows > 0.0 && l.volcanic.len() >= gw * gh)
            .map(|l| cartalith_terrain::edifice_pits::EdificeFrame::from_volcanic_field(&l.volcanic, gw, gh));
        let info_age = litho.as_ref().map(|l| Arc::clone(&l.age));
        let info_resistance = litho.as_ref().map(|l| Arc::clone(&l.resistance));
        let info_flow_log_max = crate::sample_bridge::flow_log_max(flow.as_ref().map_or(&[][..], |v| v.as_slice()));
        // A throwaway context, only so `TileFields::new` has the `ctx` its
        // signature takes. It borrows everything above, which is why the
        // struct is assembled after this block and not before it.
        let fields = {
            let mut ctx = RenderCtx::from_precomputed(shaded, &temperature, &rainfall, flow.as_ref().map(|v| v.as_slice()), gw, gh, sea_level, world, lat_n, lat_s, appearance.clone(), &pre)?.with_precomputed_water_height(&field);
            if let Some(l) = lithology.as_ref() {
                ctx = ctx.with_lithology(l);
            }
            // `LOD_DETAIL_SCOPE.md` LOD-D4. Built HERE and not inside
            // `TileFields::new`: the field needs `glacial_snowline` and
            // `peak_m`, which are *generation* parameters, and the map width
            // in km -- three numbers a `RenderCtx` does not carry.
            //
            // A **loaded save takes the snow-only fallback by data, not by a
            // branch here**: `flow` is already `None` for a loaded save and
            // `build_glacier_potential` returns an empty field for a `None`
            // flow, with that reason in its own doc comment.
            let glacier = render::build_glacier_potential(shaded, &temperature, flow.as_ref().map(|v| v.as_slice()), gw, gh, sea_level, glacial_snowline, map_width_km / gw.max(1) as f64, world);
            let cryo = TileCryo {
                lapse_rate,
                g: gravity,
                meters_per_unit: if (1.0 - sea_level).abs() > 1e-9 { peak_m / (1.0 - sea_level) } else { peak_m / 1e-6 },
            };
            let tf = TileFields::new(&ctx, grid_rgb.as_deref()).with_cryo(glacier, cryo);
            // Ruling BO: the forced lakes the base map draws, drawn here too.
            match forced_lakes.as_deref() {
                Some(m) => tf.with_forced_lakes(m),
                None => tf,
            }
        };
        Some(LodSnapshot {
            key,
            producer,
            gw,
            gh,
            sea_level,
            world,
            lat_n,
            lat_s,
            seed,
            field,
            shade,
            temperature,
            rainfall,
            flow,
            appearance,
            color_space,
            pre,
            lithology,
            rock_detail,
            colour_paint,
            edifice_frame,
            fields,
            ink,
            splat,
            ground_biomes,
            ground_terrains,
            paint_present,
            paint_biome,
            paint_terrain,
            paint_splat,
            rivers,
            river_fans,
            river_paint,
            peak_m,
            map_width_km,
            age: info_age,
            resistance: info_resistance,
            flow_log_max: info_flow_log_max,
            plain_rgba: Mutex::new(std::collections::VecDeque::new()),
        })
    }

    /// The key this snapshot was built for — `WorldGen::lod_cache_key`'s
    /// string, carried so a landing worker and the main thread can agree on
    /// whether it is still the current world.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// What a pyramid stored from this snapshot calls its producer, and
    /// therefore the exact string a stored pyramid must carry to be seeded
    /// in its place: `lod_bridge::tile_producer_id` (the build's constants
    /// and the appearance) plus `;in=` and [`SnapshotInputs::fingerprint`]
    /// (every other tile input, the colour space among them).
    ///
    /// `SAVEFILE_COMPAT.md` §16.1: *"A reader MUST drop the tiles unless it
    /// produced this exact string itself."* A project saved before
    /// 2026-09-24 carries the bare `tile_producer_id`, with no `;in=`, and so
    /// is never seeded — the backward-compatible reading of a key that cannot
    /// say which look and colour space its tiles were drawn in.
    pub fn producer_id(&self) -> &str {
        &self.producer
    }

    /// **The one function that colours a tile**, called identically from the
    /// main thread and from a worker.
    ///
    /// Its body is the pre-D6 `WorldGen::lod_tile_bytes` from the cache
    /// borrow onward, attachment for attachment and in the same order — the
    /// four builders `build_color_texture` attaches, then the ink and the
    /// colour space on the `TileFields`. `with_map_scale` is deliberately
    /// **not** called: its three outputs are already in `pre`
    /// ([`LodSnapshot::build`] passes `Some(map_width_km)`), and calling it
    /// again would recompute two full-grid distance transforms per tile and
    /// overwrite them with identical values.
    pub fn render_tile(&self, z: i32, col: i32, row: i32) -> Option<(Vec<u8>, usize, usize)> {
        self.render_tile_conformed(z, col, row, 0)
    }

    /// **EF-9.4** -- [`Self::render_tile`] for a tile drawn beside ground one
    /// level coarser, `conform` being its 8-neighbour mask
    /// (`lod_bridge::synthesize_tile_rgba_rivers_conformed`). `0` is
    /// `render_tile`; the attachments, their order and the coloriser are the
    /// same call for both, so a conformed tile differs from a plain one only
    /// inside the conform band: its heights there, and its colour cross-faded
    /// toward the plain parent tile (EF-9.4's colour half, 2026-10-18).
    pub fn render_tile_conformed(&self, z: i32, col: i32, row: i32, conform: u8) -> Option<(Vec<u8>, usize, usize)> {
        self.render_tile_conformed_cached(z, col, row, conform, true)
    }

    /// [`Self::render_tile_conformed`] with the plain-tile cache switchable:
    /// `use_cache == false` never reads or fills it, so the colour half's
    /// parent is synthesised by `lod_bridge` itself -- the independent
    /// reference the cache tests compare against.
    fn render_tile_conformed_cached(&self, z: i32, col: i32, row: i32, conform: u8, use_cache: bool) -> Option<(Vec<u8>, usize, usize)> {
        // RV-3: shaded from `shade` when there is one, water from `field`,
        // exactly as `build` built the precompute.
        let shaded: &[f32] = self.shade.as_deref().map_or(&self.field, |s| s.as_slice());
        let mut ctx = RenderCtx::from_precomputed(shaded, &self.temperature, &self.rainfall, self.flow.as_ref().map(|v| v.as_slice()), self.gw, self.gh, self.sea_level, self.world, self.lat_n, self.lat_s, self.appearance.clone(), &self.pre)?
            .with_precomputed_water_height(&self.field);
        if let Some(l) = self.lithology.as_ref() {
            ctx = ctx.with_lithology(l);
        }
        // GF-6 §4.10: rock-styled deep-zoom detail, only for a world that has
        // it (`SnapshotInputs::rock_detail`); `None` attaches nothing.
        if let Some(rd) = self.rock_detail.as_deref() {
            ctx = ctx.with_rock_detail(rd);
        }
        // Ruling CN's colour paint, the layer the base map attaches
        // (`build_color_texture`), so a tile and the screen agree.
        ctx = ctx.with_colour_paint(self.colour_paint.as_deref());
        // Ruling CO item 3: the volcano furrows, only for a world that has an
        // edifice frame (`LodSnapshot::edifice_frame`); `None` attaches
        // nothing.
        if let Some(fr) = self.edifice_frame.as_ref() {
            ctx = ctx.with_edifice_frame(fr, self.peak_m);
        }
        if let Some(s) = self.splat.as_ref() {
            ctx = ctx.with_splat(s.as_textures());
            ctx = ctx.with_ground_tiles(GroundTiles { biomes: &self.ground_biomes, terrains: &self.ground_terrains });
        }
        if self.paint_present {
            ctx = ctx.with_paint(self.paint_biome.as_deref(), self.paint_terrain.as_deref(), self.paint_splat.as_deref());
        }
        let mut tf = self.fields.borrowed();
        if let Some(ink) = self.ink.as_ref() {
            tf = tf.with_ink(ink);
        }
        tf = tf.with_color_space(self.color_space);
        let (rivers, fans, paint) = (self.rivers.as_deref(), self.river_fans.as_deref(), self.river_paint.as_deref());
        if !use_cache {
            return lod_bridge::synthesize_tile_rgba_rivers_conformed(&ctx, &tf, z, col, row, self.seed, rivers, fans, paint, conform);
        }
        if conform == 0 {
            // A plain tile a conformed sibling already coloured is that very
            // output (same snapshot, same function, `conform == 0`), so a hit
            // is returned as is. Never inserts: see `PLAIN_TILE_CACHE`.
            if let Some(hit) = self.plain_cached(z, col, row) {
                return Some((*hit).clone());
            }
            return lod_bridge::synthesize_tile_rgba_rivers_conformed(&ctx, &tf, z, col, row, self.seed, rivers, fans, paint, 0);
        }
        let key = (z - 1, col >> 1, row >> 1);
        let parent = match self.plain_cached(key.0, key.1, key.2) {
            Some(p) => p,
            None => {
                let p = Arc::new(lod_bridge::synthesize_tile_rgba_rivers_conformed(&ctx, &tf, key.0, key.1, key.2, self.seed, rivers, fans, paint, 0)?);
                if let Ok(mut c) = self.plain_rgba.lock() {
                    if c.len() >= PLAIN_TILE_CACHE {
                        c.pop_front();
                    }
                    c.push_back((key, Arc::clone(&p)));
                }
                p
            }
        };
        let pr = (parent.0.as_slice(), parent.1, parent.2);
        lod_bridge::synthesize_tile_rgba_rivers_conformed_with_parent(&ctx, &tf, z, col, row, self.seed, rivers, fans, paint, conform, Some(pr))
    }

    /// The cached plain tile `(z, col, row)`, if [`Self::render_tile_conformed`]
    /// coloured it as a parent recently. A poisoned lock reads as a miss: the
    /// caller then synthesises, which is always correct, only slower.
    fn plain_cached(&self, z: i32, col: i32, row: i32) -> Option<Arc<(Vec<u8>, usize, usize)>> {
        let c = self.plain_rgba.lock().ok()?;
        c.iter().rev().find(|(k, _)| *k == (z, col, row)).map(|(_, v)| Arc::clone(v))
    }

    /// The borrowed inputs an info tile reads (LOD-D7). One constructor, so the
    /// availability test and the synthesis can never look at different slices.
    fn info_source(&self) -> crate::info_tile::InfoSource<'_> {
        crate::info_tile::InfoSource {
            gw: self.gw,
            gh: self.gh,
            sea: self.sea_level,
            seed: self.seed,
            peak_m: self.peak_m,
            map_width_km: self.map_width_km,
            field: &self.field,
            temperature: &self.temperature,
            rainfall: &self.rainfall,
            flow: self.flow.as_ref().map(|v| v.as_slice()),
            age: self.age.as_ref().map(|v| v.as_slice()),
            resistance: self.resistance.as_ref().map(|v| v.as_slice()),
            flow_log_max: self.flow_log_max,
            zoom_detail_km: self.appearance.zoom_detail_km,
        }
    }

    /// **The one function that draws a deep-zoom info tile** (LOD-D7, Ruling
    /// AV), called identically from the main thread and from a worker. Derives
    /// from this snapshot's fields -- see `info_tile`'s module doc -- and
    /// never touches the terrain tile path.
    pub fn render_info_tile(&self, view: crate::info_tile::InfoView, z: i32, col: i32, row: i32) -> Option<(Vec<u8>, usize, usize)> {
        crate::info_tile::synthesize_info_tile(&self.info_source(), view, z, col, row)
    }

    /// Whether the inputs `view` reads exist in this snapshot.
    pub fn info_available(&self, view: crate::info_tile::InfoView) -> bool {
        self.info_source().available(view)
    }

    /// The cache tag every tile of `view` from this snapshot carries
    /// ([`crate::info_tile::cache_tag`] over [`Self::producer_id`]).
    pub fn info_tag(&self, view: crate::info_tile::InfoView) -> String {
        crate::info_tile::cache_tag(view, &self.producer)
    }

    /// Every tile of levels `0..=z_max`, as storable masks — owner rulings
    /// 28/29's producer, `SAVEFILE_COMPAT.md` §16.1.
    ///
    /// Loops over [`render_tile`](Self::render_tile) rather than calling
    /// [`lod_bridge::synthesize_pyramid_masks`] with a hand-built
    /// `RenderCtx`/`TileFields` pair: `render_tile` is the one function that
    /// attaches lithology, the pack splat, ground tiles, paint and the river
    /// ink exactly the way the interactive path does (this module's own
    /// determinism argument, in its header), so a stored pyramid can never
    /// silently disagree with what the same world draws live. The per-tile
    /// cost is the same either way — `render_tile` rebuilds only the cheap
    /// `RenderCtx::from_precomputed` borrow each call, not the ~480 ms
    /// `GridPrecompute`/`TileFields` build this snapshot already paid once.
    ///
    /// Returns `(tile_w, tile_h, tiles)` in exactly the shape
    /// `cartalith_io::project::LodTiles` takes. `None` for a `z_max` outside
    /// `0..=`[`lod_bridge::MAX_LEVEL`], or if any tile [`render_tile`](Self::render_tile)
    /// itself would refuse — which nothing in a snapshot that already built
    /// successfully should reach, but the caller must not receive a partial
    /// pyramid silently either way.
    pub fn render_pyramid_masks(
        &self,
        z_max: i32,
    ) -> Option<(usize, usize, std::collections::BTreeMap<ChunkId, Vec<u8>>)> {
        self.pyramid_masks_with(z_max, |z, col, row| {
            self.render_tile(z, col, row).map(|(rgba, _, _)| lod_bridge::tile_mask(&rgba))
        })
    }

    /// The pyramid walk itself, with where each tile's mask comes from left
    /// to `mask`: [`Self::render_pyramid_masks`] synthesises every one, and
    /// [`LodWorker::pyramid_masks`] reuses a held seeded mask where its
    /// producer is still this snapshot's. One walk, so the two cannot
    /// disagree about which tiles a pyramid has or in what shape.
    fn pyramid_masks_with(
        &self,
        z_max: i32,
        mut mask: impl FnMut(i32, i32, i32) -> Option<Vec<u8>>,
    ) -> Option<(usize, usize, std::collections::BTreeMap<ChunkId, Vec<u8>>)> {
        if !(0..=lod_bridge::MAX_LEVEL).contains(&z_max) {
            return None;
        }
        let (tile_w, tile_h) = lod_bridge::tile_size_px(self.gw, self.gh, 0);
        let mut tiles = std::collections::BTreeMap::new();
        for z in 0..=z_max {
            let n = lod_bridge::tiles_per_axis(z) as i32;
            for col in 0..n {
                for row in 0..n {
                    tiles.insert(ChunkId::new(z as u32, col as u32, row as u32), mask(z, col, row)?);
                }
            }
        }
        Some((tile_w, tile_h, tiles))
    }

    /// Bytes retained by this snapshot, for the memory bar LOD-D6 is graded
    /// against (*"steady memory at most 60 MiB above the recorded steady
    /// state"*).
    ///
    /// **What this snapshot COPIED, which is no longer the four world
    /// grids.** `field`/`temperature`/`rainfall`/`flow` used to be counted
    /// here and were 43.0 MB of the 51.21 MB measured at 2048x1311. They are
    /// `Arc`s shared with `WorldState` now and add nothing to steady memory,
    /// so counting them would report bytes that are not there. `pre`,
    /// `lithology` and `fields` were already retained by LOD-D2's cache and
    /// were never counted, for the same reason; nor is Ruling CO's
    /// `edifice_frame` (4 bytes of owner index per cell, about 10.7 MB at
    /// 2048 x 1311, when the look draws furrows), which is derived per
    /// snapshot like `lithology`. A pack's textures are counted
    /// because that clone is still real. Reported rather than estimated from
    /// a formula, so the shell can show the real number.
    ///
    /// **A shared `Arc` is not free in every case** — see
    /// [`cartalith_engine::WorldState::field`]: a sculpt or erode landing
    /// while a snapshot is alive copies the one grid it touches, once. That
    /// copy belongs to the world, not to this snapshot, and is not counted
    /// here either.
    pub fn retained_bytes(&self) -> usize {
        let mut n = 0usize;
        n += self.ink.as_ref().map_or(0, |v| v.len());
        for g in [&self.paint_biome, &self.paint_terrain, &self.paint_splat] {
            n += g.as_ref().map_or(0, |v| v.len());
        }
        if let Some(s) = self.splat.as_ref() {
            for c in [&s.grass, &s.rock, &s.sand, &s.snow, &s.wetland, &s.canopy] {
                n += c.as_ref().map_or(0, |c| c.rgba.len());
            }
        }
        for t in self.ground_biomes.iter().chain(self.ground_terrains.iter()).flatten() {
            n += t.rgba.len();
        }
        n
    }
}

// ---------------------------------------------------------------------------
// The pool
// ---------------------------------------------------------------------------

/// The tile pool, or `None` when one could not be created.
///
/// **A dedicated pool, not rayon's global one.** The global pool is what every
/// synchronous `par_iter` in this crate runs on — `build_color_texture`'s
/// per-pixel pass among them — and queueing a viewful of tile jobs into it
/// would put them ahead of the main thread's own render work. A separate pool
/// sized `cores - 1` also leaves a core for the main thread, which is the
/// whole point of the milestone.
///
/// **`None` is a real state and the caller must handle it**: a pool that
/// cannot be created means synthesis stays synchronous, which is slow but
/// correct. Returning an `Option` rather than `expect`ing is
/// `cartalith-rust-conventions`' rule — a panic here would be on the main
/// thread inside a `#[func]` and would take the Godot process down.
///
/// Returns a clone of the pool's `Arc`, not a `&'static`: the pool can be
/// torn down by [`shutdown_pool`] and a borrow of it must not outlive that.
/// Callers hold the clone only for the length of one `spawn`.
fn pool() -> Option<Arc<rayon::ThreadPool>> {
    let Ok(mut slot) = POOL.lock() else { return None };
    if let PoolSlot::Unbuilt = *slot {
        *slot = build_pool().map_or(PoolSlot::Failed, PoolSlot::Live);
    }
    match &*slot {
        PoolSlot::Live(p) => Some(Arc::clone(&p.pool)),
        // A failed build stays failed, as it did when this was a `OnceLock`:
        // retrying would spawn threads on every tile request of a machine
        // that already said it cannot.
        PoolSlot::Unbuilt | PoolSlot::Failed => None,
    }
}

/// The tile pool plus the join handles of every thread it runs on.
///
/// **Why the handles are kept.** Without them nothing can wait for the pool's
/// threads to *finish*, and that is the exit crash `OUTSTANDING_WORK.md`
/// filed as "several probes crash with an access violation at exit": a probe
/// that quits right after queueing tiles (`_riverstroke_probe` flips the
/// rivers layer, which re-queues the view, and then quits) leaves these
/// threads inside `LodSnapshot::render_tile` while Godot unloads this
/// library. Their code is unmapped under them and the process dies with
/// 0xC0000005 after the probe has printed its verdict. Bisected 2026-09-28
/// in a scratch copy of that probe: quitting before the flip, or after
/// waiting for the pyramid to settle, exits 0; quitting straight after the
/// flip exits 139 every time.
struct LodPool {
    pool: Arc<rayon::ThreadPool>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

/// The pool's lifecycle. `Unbuilt` at load and again after [`shutdown_pool`],
/// so a library that Godot re-initialises in the same process (the editor's
/// extension hot-reload) builds a fresh pool instead of finding a dead one.
enum PoolSlot {
    Unbuilt,
    Failed,
    Live(LodPool),
}

static POOL: Mutex<PoolSlot> = Mutex::new(PoolSlot::Unbuilt);

/// Build the pool, spawning its threads ourselves (`spawn_handler`) so their
/// `JoinHandle`s can be kept. `None` if the pool or any thread could not be
/// created; the caller records that as [`PoolSlot::Failed`].
fn build_pool() -> Option<LodPool> {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    let threads: Arc<Mutex<Vec<std::thread::JoinHandle<()>>>> = Arc::default();
    let sink = Arc::clone(&threads);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cores.saturating_sub(1).max(1))
        .thread_name(|i| format!("cartalith-lod-{i}"))
        // A panic inside a tile job must not abort the process, which is
        // rayon's default for an unhandled one. There is no `Gd` and no
        // Godot API in a job, so there is nothing to report *to* from
        // here; the job simply produces no tile and the shell keeps its
        // fallback. `eprintln!` rather than `godot_error!` because the
        // latter is not documented as callable off the main thread.
        .panic_handler(|_| eprintln!("cartalith: a LOD tile job panicked; the tile was dropped"))
        // rayon's own default spawn, except that the handle is kept.
        .spawn_handler(move |t| {
            let mut b = std::thread::Builder::new();
            if let Some(name) = t.name() {
                b = b.name(name.to_owned());
            }
            if let Some(size) = t.stack_size() {
                b = b.stack_size(size);
            }
            let h = b.spawn(|| t.run())?;
            if let Ok(mut v) = sink.lock() {
                v.push(h);
            }
            Ok(())
        })
        .build()
        .ok()?;
    // The builder (and its clone of `threads`) is gone once `build` returns,
    // so this is normally the only owner; a poisoned lock yields no handles,
    // which only makes a later shutdown unable to wait, never unsound.
    let threads = threads.lock().map(|mut v| std::mem::take(&mut *v)).unwrap_or_default();
    Some(LodPool { pool: Arc::new(pool), threads })
}

/// **Stop the tile pool and wait for every one of its threads to exit.**
/// Called from the extension's deinit (`lib.rs`, `on_stage_deinit`), before
/// Godot unloads this library.
///
/// What it does: takes the pool out of [`POOL`], drops it (rayon's signal to
/// its threads to finish the jobs already spawned and then exit), and joins
/// each thread. A job's output lands in an `Arc<LodWorker>` the job itself
/// owns, so finishing it after every `WorldGen` is gone is harmless.
///
/// Why: this library's code must not be running on any thread when Godot
/// unmaps it -- see [`LodPool`] for the crash that happens otherwise.
///
/// Must never: call a Godot API (none is needed, and at deinit much of
/// Godot is already down); panic (a panic here is caught by gdext but the
/// join it skipped would reinstate the crash); or be called while a caller
/// still holds a [`pool`] clone -- then the drop below is not the last one,
/// the pool is not terminated, and joining would wait forever. It checks
/// that instead of assuming it and skips the join, reporting why.
///
/// Rayon's *global* pool is not covered: it cannot be dropped, and every use
/// of it is a synchronous `par_iter` the caller waits for, so it has no work
/// in flight once the main loop has ended.
pub fn shutdown_pool() {
    let taken = match POOL.lock() {
        Ok(mut slot) => std::mem::replace(&mut *slot, PoolSlot::Unbuilt),
        Err(_) => return,
    };
    let PoolSlot::Live(LodPool { pool, threads }) = taken else { return };
    match Arc::try_unwrap(pool) {
        // Dropping the last handle terminates the pool; the threads then run
        // out their queue and return, which is what the joins wait for.
        Ok(pool) => drop(pool),
        Err(_) => {
            eprintln!("cartalith: LOD pool still borrowed at shutdown; not waiting for its threads");
            return;
        }
    }
    for h in threads {
        // `Err` means the thread panicked outside a job; it has still exited,
        // which is all this function needs.
        let _ = h.join();
    }
}

// ---------------------------------------------------------------------------
// The worker
// ---------------------------------------------------------------------------

/// A finished tile on its way back to the main thread. Plain bytes — the
/// `ImageTexture` is created by the caller, on the main thread.
pub struct ReadyTile {
    pub z: i32,
    pub col: i32,
    pub row: i32,
    /// EF-9.4's conform mask the tile was drawn with (`0` = the plain tile).
    /// Part of the tile's identity: the shell keys a conformed tile apart from
    /// the plain one at the same `(z, col, row)`.
    pub conform: u8,
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

/// What [`LodWorker::prepare`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareState {
    /// No world, or the inputs could not be assembled. Nothing is in flight.
    Unavailable,
    /// A build for this key is running. Tiles cannot be requested yet; the
    /// shell draws its base-map fallback (LOD-D3) in the meantime, which is
    /// exactly the state that fallback was built for.
    Building,
    /// A snapshot for this key is live and tiles can be requested.
    Ready,
}

/// A finished info tile (LOD-D7). Carries its view and tag so the shell can
/// refuse one whose view or inputs have moved since it was requested.
pub struct ReadyInfoTile {
    pub view: crate::info_tile::InfoView,
    pub tag: String,
    pub z: i32,
    pub col: i32,
    pub row: i32,
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

/// Everything [`LodWorker`] guards behind its one `Mutex`: the live snapshot and
/// its key, the in-flight sets, the finished-tile queues and the diagnostics
/// counters. Never held across `make()` or a tile render, so a landing job
/// is not blocked behind the main thread's clone.
#[derive(Default)]
struct WorkerState {
    /// The key of `snapshot`, or of the build in flight.
    key: String,
    snapshot: Option<Arc<LodSnapshot>>,
    building: bool,
    /// `(z, col, row, conform)`: EF-9.4's mask is part of a tile's identity, so a
    /// conformed and a plain request for one chunk are two jobs, not a duplicate.
    in_flight: HashSet<(i32, i32, i32, u8)>,
    ready: Vec<ReadyTile>,
    /// LOD-D7: info tiles in flight / finished, keyed by view as well as
    /// `(z, col, row)` so two views of one tile never collide. Cleared with
    /// `in_flight`/`ready` everywhere those are, so the same generation rule
    /// drops them.
    info_in_flight: HashSet<(crate::info_tile::InfoView, i32, i32, i32)>,
    info_ready: Vec<ReadyInfoTile>,
    /// Counters for the shell's diagnostics, never for a decision.
    built: u64,
    dropped: u64,
    info_built: u64,
    info_dropped: u64,
}

/// A pyramid read back from a project archive, held until a snapshot can say
/// whether it is still this world's (owner rulings 28/29's read half).
///
/// Held rather than decided at open because the decision needs the
/// snapshot's [`LodSnapshot::producer_id`], and the one input that is timing-
/// rather than world-dependent — the finished grid raster — only exists once
/// the base map has been drawn.
pub struct StoredPyramid {
    pub producer: String,
    pub tile_w: usize,
    pub tile_h: usize,
    pub tiles: std::collections::BTreeMap<ChunkId, Vec<u8>>,
}

/// What `WorldGen::project_open` holds out of `ProjectData::lod_tiles` — one
/// conversion, so the tests below exercise the open path's own code.
impl From<cartalith_io::project::LodTiles> for StoredPyramid {
    /// Field-for-field move; no validation here, because validity is decided later
    /// by the producer string (see [`LodWorker::seed`]).
    fn from(l: cartalith_io::project::LodTiles) -> Self {
        StoredPyramid { producer: l.producer, tile_w: l.tile_w, tile_h: l.tile_h, tiles: l.tiles }
    }
}

/// The main thread's handle on the background synthesis.
///
/// Held by `WorldGen` as an `Arc<LodWorker>`; the `Arc` is what a job captures.
/// Nothing inside is Godot-aware, which is what makes crossing a thread with it
/// legal at all. It must never hold a `Gd` or a `WorldGen` borrow.
#[derive(Default)]
pub struct LodWorker {
    /// Bumped whenever the snapshot key changes or [`Self::invalidate`] runs.
    /// **This is the scope's *"a world version checked on landing drops stale
    /// tiles"***: every job captures the generation it was spawned under and
    /// its result is discarded if the counter has moved since. A tile for the
    /// previous world can therefore never be installed, however late it lands.
    generation: AtomicU64,
    state: Mutex<WorkerState>,
    /// The reopened project's stored pyramid, if it carried one. **Not
    /// cleared by [`Self::invalidate`]**: that runs whenever the snapshot is
    /// dropped, and the seed's validity is its producer string, not the
    /// snapshot's lifetime. [`Self::clear_seed`] is the world-release half.
    seed: Mutex<Option<StoredPyramid>>,
    /// Tiles answered from `seed` instead of synthesised. A probe's
    /// "reopened and did not re-synthesise" signal.
    seeded_served: AtomicU64,
    /// Tiles actually coloured by [`LodSnapshot::render_tile`] through this
    /// worker, on either path.
    synthesized: AtomicU64,
}

impl LodWorker {
    /// Ensure a snapshot for `key` exists, building one in the background if
    /// not.
    ///
    /// `make` is called **only** when a build is actually needed, and is
    /// called on the caller's thread (it reads `WorldGen`). It is the
    /// expensive clone; the ~480 ms of precompute that follows is what moves
    /// off the thread.
    ///
    /// **Only the main thread calls this.** The lock is released across
    /// `make()` so a landing job is not blocked behind a 43 MB memcpy, which
    /// is safe precisely because there is one caller; two concurrent
    /// `prepare` calls for different keys could otherwise both decide to
    /// build.
    pub fn prepare(self: &Arc<Self>, key: &str, make: impl FnOnce() -> Option<SnapshotInputs>) -> PrepareState {
        {
            let st = self.state.lock().ok();
            match st {
                Some(st) if st.key == key => {
                    if st.snapshot.is_some() {
                        return PrepareState::Ready;
                    }
                    if st.building {
                        return PrepareState::Building;
                    }
                }
                Some(_) => {}
                // A poisoned lock means a job panicked while holding it, which
                // the panic handler above makes unreachable. Report
                // unavailable rather than unwrapping into the gdext boundary.
                None => return PrepareState::Unavailable,
            }
        }
        let Some(inputs) = make() else {
            return PrepareState::Unavailable;
        };
        let Some(pool) = pool() else {
            // No pool: build it here and now, synchronously. Slow, correct,
            // and the same bytes -- `render_tile` is the only coloriser.
            let Some(snap) = LodSnapshot::build(inputs) else {
                return PrepareState::Unavailable;
            };
            self.install(Arc::new(snap));
            return PrepareState::Ready;
        };
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        {
            let Ok(mut st) = self.state.lock() else {
                return PrepareState::Unavailable;
            };
            st.key = key.to_string();
            st.snapshot = None;
            st.building = true;
            st.in_flight.clear();
            st.dropped += st.ready.len() as u64;
            st.ready.clear();
            st.info_in_flight.clear();
            st.info_dropped += st.info_ready.len() as u64;
            st.info_ready.clear();
        }
        let me = Arc::clone(self);
        pool.spawn(move || {
            let built = LodSnapshot::build(inputs);
            let Ok(mut st) = me.state.lock() else { return };
            // The generation check is the whole staleness rule: if the world,
            // the appearance or anything else in the key moved while this was
            // building, another `prepare` has already bumped it and this
            // result belongs to a world nobody is looking at.
            if me.generation.load(Ordering::SeqCst) != generation {
                return;
            }
            st.building = false;
            st.snapshot = built.map(Arc::new);
        });
        PrepareState::Building
    }

    /// Install a snapshot built on the caller's thread — the synchronous
    /// path's way of sharing its work with the asynchronous one, so the two
    /// never hold two copies of the same 50 MB.
    pub fn install(&self, snap: Arc<LodSnapshot>) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut st) = self.state.lock() {
            st.key = snap.key().to_string();
            st.snapshot = Some(snap);
            st.building = false;
            st.in_flight.clear();
            st.dropped += st.ready.len() as u64;
            st.ready.clear();
            st.info_in_flight.clear();
            st.info_dropped += st.info_ready.len() as u64;
            st.info_ready.clear();
        }
    }

    /// The live snapshot, if it is for `key`. `None` means the caller must
    /// build one (synchronously) or wait for [`Self::prepare`].
    pub fn snapshot_for(&self, key: &str) -> Option<Arc<LodSnapshot>> {
        let st = self.state.lock().ok()?;
        if st.key == key {
            st.snapshot.clone()
        } else {
            None
        }
    }

    /// Queue one tile. `false` when it was not queued — no snapshot, already
    /// in flight, already waiting in `ready`, or the tier's `max_in_flight`
    /// cap is full — and the caller simply asks again next frame.
    pub fn request(self: &Arc<Self>, z: i32, col: i32, row: i32, max_in_flight: usize) -> bool {
        self.request_conformed(z, col, row, 0, max_in_flight)
    }

    /// [`Self::request`] for EF-9.4's conformed tile (`conform` != 0 is never
    /// served from a stored pyramid, which holds plain tiles only). Same
    /// refusals, same shared in-flight cap, same generation rule.
    pub fn request_conformed(self: &Arc<Self>, z: i32, col: i32, row: i32, conform: u8, max_in_flight: usize) -> bool {
        let Some(pool) = pool() else { return false };
        let (snap, generation) = {
            let Ok(mut st) = self.state.lock() else { return false };
            let Some(snap) = st.snapshot.clone() else { return false };
            let id = (z, col, row, conform);
            if st.in_flight.contains(&id) || st.ready.iter().any(|t| (t.z, t.col, t.row, t.conform) == id) {
                return false;
            }
            // A seeded tile needs no job: it is already coloured. Handed
            // straight to `ready`, ahead of the in-flight cap, which exists
            // to bound work and there is none. Plain tiles only: a stored
            // pyramid holds no conformed variant.
            if conform == 0 {
                if let Some((rgba, w, h)) = self.seeded_tile(&snap, z, col, row) {
                    self.seeded_served.fetch_add(1, Ordering::SeqCst);
                    st.ready.push(ReadyTile { z, col, row, conform, w, h, rgba });
                    return true;
                }
            }
            if st.in_flight.len() >= max_in_flight.max(1) {
                return false;
            }
            st.in_flight.insert(id);
            (snap, self.generation.load(Ordering::SeqCst))
        };
        let me = Arc::clone(self);
        pool.spawn(move || {
            me.synthesized.fetch_add(1, Ordering::SeqCst);
            let out = snap.render_tile_conformed(z, col, row, conform);
            let Ok(mut st) = me.state.lock() else { return };
            st.in_flight.remove(&(z, col, row, conform));
            if me.generation.load(Ordering::SeqCst) != generation {
                st.dropped += 1;
                return;
            }
            match out {
                Some((rgba, w, h)) => {
                    st.ready.push(ReadyTile { z, col, row, conform, w, h, rgba });
                    st.built += 1;
                }
                None => st.dropped += 1,
            }
        });
        true
    }

    /// Queue one info tile of `view` (LOD-D7). Same discipline as
    /// [`Self::request`]: a dedicated-pool job, the generation check on landing,
    /// and refusal (never an error) when there is no snapshot, the inputs for
    /// `view` are missing, the tile is already in flight or waiting, or the
    /// tier's `max_in_flight` cap is full. **The cap is shared**: terrain and
    /// info jobs together are counted against it by this call, so an info view
    /// cannot double the background work a tier was sized for.
    pub fn request_info(self: &Arc<Self>, view: crate::info_tile::InfoView, z: i32, col: i32, row: i32, max_in_flight: usize) -> bool {
        let Some(pool) = pool() else { return false };
        let (snap, generation) = {
            let Ok(mut st) = self.state.lock() else { return false };
            let Some(snap) = st.snapshot.clone() else { return false };
            if !snap.info_available(view) {
                return false;
            }
            let id = (view, z, col, row);
            if st.info_in_flight.contains(&id) || st.info_ready.iter().any(|t| (t.view, t.z, t.col, t.row) == id) {
                return false;
            }
            if st.in_flight.len() + st.info_in_flight.len() >= max_in_flight.max(1) {
                return false;
            }
            st.info_in_flight.insert(id);
            (snap, self.generation.load(Ordering::SeqCst))
        };
        let me = Arc::clone(self);
        pool.spawn(move || {
            let out = snap.render_info_tile(view, z, col, row);
            let tag = snap.info_tag(view);
            let Ok(mut st) = me.state.lock() else { return };
            st.info_in_flight.remove(&(view, z, col, row));
            if me.generation.load(Ordering::SeqCst) != generation {
                st.info_dropped += 1;
                return;
            }
            match out {
                Some((rgba, w, h)) => {
                    st.info_ready.push(ReadyInfoTile { view, tag, z, col, row, w, h, rgba });
                    st.info_built += 1;
                }
                None => st.info_dropped += 1,
            }
        });
        true
    }

    /// Take up to `max` finished info tiles. Main thread only, which then
    /// creates the textures.
    pub fn take_ready_info(&self, max: usize) -> Vec<ReadyInfoTile> {
        let Ok(mut st) = self.state.lock() else { return Vec::new() };
        let n = max.min(st.info_ready.len());
        st.info_ready.drain(..n).collect()
    }

    /// `(in_flight, waiting, built, dropped)` for info tiles -- diagnostics and
    /// probes, never a decision.
    pub fn info_stats(&self) -> (usize, usize, u64, u64) {
        let Ok(st) = self.state.lock() else { return (0, 0, 0, 0) };
        (st.info_in_flight.len(), st.info_ready.len(), st.info_built, st.info_dropped)
    }

    /// Hold a stored pyramid for seeding. Replaces any held before.
    pub fn seed(&self, p: StoredPyramid) {
        if let Ok(mut s) = self.seed.lock() {
            *s = Some(p);
        }
    }

    /// Drop the held pyramid — a world release, whose tiles it described.
    pub fn clear_seed(&self) {
        if let Ok(mut s) = self.seed.lock() {
            *s = None;
        }
    }

    /// `(tiles held, tiles served from them, tiles synthesised)`.
    pub fn seed_stats(&self) -> (usize, u64, u64) {
        let held = self.seed.lock().ok().and_then(|s| s.as_ref().map(|p| p.tiles.len())).unwrap_or(0);
        (held, self.seeded_served.load(Ordering::SeqCst), self.synthesized.load(Ordering::SeqCst))
    }

    /// The held tile at `(z, col, row)` as RGBA, **only if** the held
    /// pyramid's producer is exactly `snap`'s — the world, the look, the
    /// colour space and every attachment the same — and it has that tile at
    /// the size `snap` would draw it. Otherwise `None`, and the caller
    /// synthesises as it always did.
    fn seeded_tile(&self, snap: &LodSnapshot, z: i32, col: i32, row: i32) -> Option<(Vec<u8>, usize, usize)> {
        if z < 0 || col < 0 || row < 0 {
            return None;
        }
        let held = self.seed.lock().ok()?;
        let p = held.as_ref()?;
        let (w, h) = lod_bridge::tile_size_px(snap.gw, snap.gh, z);
        if p.producer != snap.producer || (p.tile_w, p.tile_h) != (w, h) {
            return None;
        }
        let mask = p.tiles.get(&ChunkId::new(z as u32, col as u32, row as u32))?;
        if mask.len() != w * h * 3 {
            return None;
        }
        Some((lod_bridge::mask_to_rgba(mask), w, h))
    }

    /// The held mask at `(z, col, row)`, verbatim, under exactly
    /// [`Self::seeded_tile`]'s conditions -- the producer is `snap`'s and the
    /// mask is the size `snap` would draw. The save path's counterpart of
    /// that function: a mask goes back into an archive as a mask, so it is
    /// cloned rather than round-tripped through RGBA.
    fn seeded_mask(&self, snap: &LodSnapshot, z: i32, col: i32, row: i32) -> Option<Vec<u8>> {
        if z < 0 || col < 0 || row < 0 {
            return None;
        }
        let held = self.seed.lock().ok()?;
        let p = held.as_ref()?;
        let (w, h) = lod_bridge::tile_size_px(snap.gw, snap.gh, z);
        if p.producer != snap.producer || (p.tile_w, p.tile_h) != (w, h) {
            return None;
        }
        let mask = p.tiles.get(&ChunkId::new(z as u32, col as u32, row as u32))?;
        (mask.len() == w * h * 3).then(|| mask.clone())
    }

    /// Every tile of levels `0..=z_max` for `snap` as storable masks --
    /// [`LodSnapshot::render_pyramid_masks`]' shape and walk -- **reusing
    /// each held seeded mask whose producer is still `snap`'s** and
    /// synthesising only the rest. The save path's entry point
    /// (`project_bridge.rs`, `include_lod_tiles`), so re-saving a project
    /// that reopened with a still-valid pyramid writes those tiles back
    /// without drawing them again (`OUTSTANDING_WORK.md` §2.11, 2026-09-24).
    ///
    /// Safe for the same reason serving them on screen is: the producer
    /// string is the snapshot's own digest of every tile input, and
    /// [`LodSnapshot::render_tile`] is deterministic in those inputs, so a
    /// matching held mask is the mask synthesis would produce. Counts into
    /// [`Self::seed_stats`] exactly as [`Self::tile`] does.
    pub fn pyramid_masks(
        &self,
        snap: &LodSnapshot,
        z_max: i32,
    ) -> Option<(usize, usize, std::collections::BTreeMap<ChunkId, Vec<u8>>)> {
        snap.pyramid_masks_with(z_max, |z, col, row| {
            if let Some(m) = self.seeded_mask(snap, z, col, row) {
                self.seeded_served.fetch_add(1, Ordering::SeqCst);
                return Some(m);
            }
            self.synthesized.fetch_add(1, Ordering::SeqCst);
            snap.render_tile(z, col, row).map(|(rgba, _, _)| lod_bridge::tile_mask(&rgba))
        })
    }

    /// One tile for `snap`: the seeded one if it is valid, else synthesised.
    /// The synchronous path's entry point; [`Self::request`] applies the
    /// same rule before it spawns a job.
    pub fn tile(&self, snap: &LodSnapshot, z: i32, col: i32, row: i32) -> Option<(Vec<u8>, usize, usize)> {
        self.tile_conformed(snap, z, col, row, 0)
    }

    /// [`Self::tile`] for EF-9.4's conformed tile -- the synchronous path's
    /// counterpart of [`Self::request_conformed`]; a stored pyramid serves only
    /// `conform == 0`.
    pub fn tile_conformed(&self, snap: &LodSnapshot, z: i32, col: i32, row: i32, conform: u8) -> Option<(Vec<u8>, usize, usize)> {
        if conform == 0 {
            if let Some(t) = self.seeded_tile(snap, z, col, row) {
                self.seeded_served.fetch_add(1, Ordering::SeqCst);
                return Some(t);
            }
        }
        self.synthesized.fetch_add(1, Ordering::SeqCst);
        snap.render_tile_conformed(z, col, row, conform)
    }

    /// Take up to `max` finished tiles. Called on the main thread, which then
    /// creates the `ImageTexture`s.
    pub fn take_ready(&self, max: usize) -> Vec<ReadyTile> {
        let Ok(mut st) = self.state.lock() else { return Vec::new() };
        let n = max.min(st.ready.len());
        st.ready.drain(..n).collect()
    }

    /// `(ready_snapshot, building, in_flight, waiting, built, dropped,
    /// retained_bytes)` — for the shell's diagnostics and for probes.
    pub fn stats(&self) -> (bool, bool, usize, usize, u64, u64, usize) {
        let Ok(st) = self.state.lock() else { return (false, false, 0, 0, 0, 0, 0) };
        (st.snapshot.is_some(), st.building, st.in_flight.len(), st.ready.len(), st.built, st.dropped, st.snapshot.as_ref().map_or(0, |s| s.retained_bytes()))
    }

    /// Bumped by [`Self::install`] (the synchronous path's snapshot swap)
    /// and by [`Self::prepare`] each time it decides a background build is
    /// needed — never by a cache hit. A probe reading this before and after
    /// a call is the "rebuilt, not just happened to match" signal that
    /// `_glaciallodkey_probe.gd` needs and no `#[func]` otherwise exposes;
    /// exposed through [`crate::WorldGen::lod_worker_stats`] the same way
    /// `built`/`dropped` already are, rather than inventing a second
    /// counter.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    /// Drop the snapshot and everything queued against it, and bump the
    /// generation so anything still running lands stale.
    ///
    /// Called when a world is released and when the shell leaves the LOD
    /// layer for good, so the retained clone is not held for a session.
    /// Jobs already running are **not** cancelled — nothing in `rayon` can —
    /// they simply find the generation moved and discard their output.
    pub fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut st) = self.state.lock() {
            st.key.clear();
            st.snapshot = None;
            st.building = false;
            st.in_flight.clear();
            st.dropped += st.ready.len() as u64;
            st.ready.clear();
            st.info_in_flight.clear();
            st.info_dropped += st.info_ready.len() as u64;
            st.info_ready.clear();
        }
    }
}

/// Tests for the budget table, the snapshot, the worker's generation rule and
/// the stored-pyramid seed, driven through the real archive writer and reader.
#[cfg(test)]
mod tests {
    use super::*;

    /// A small deterministic world: a diagonal ramp with a dome, enough
    /// relief for the tile coloriser to have something to say.
    fn inputs(gw: usize, gh: usize) -> SnapshotInputs {
        let n = gw * gh;
        let mut field = vec![0f32; n];
        let mut temperature = vec![0f32; n];
        let mut rainfall = vec![0f32; n];
        let mut flow = vec![0f32; n];
        for y in 0..gh {
            for x in 0..gw {
                let fx = x as f32 / gw as f32;
                let fy = y as f32 / gh as f32;
                let d = ((fx - 0.5) * (fx - 0.5) + (fy - 0.5) * (fy - 0.5)).sqrt();
                field[y * gw + x] = (0.30 + 0.55 * (1.0 - d * 1.6).max(0.0) + 0.05 * ((x * 7 + y * 13) % 11) as f32 / 11.0).clamp(0.0, 1.0);
                temperature[y * gw + x] = 24.0 - 30.0 * fy;
                rainfall[y * gw + x] = 0.2 + 0.6 * fx;
                flow[y * gw + x] = if x == gw / 2 { 500.0 } else { 1.0 };
            }
        }
        SnapshotInputs {
            key: "test".to_string(),
            gw,
            gh,
            sea_level: 0.42,
            world: false,
            lat_n: 55.0,
            lat_s: 5.0,
            map_width_km: 800.0,
            seed: 1234,
            field: Arc::new(field),
            shade: None,
            temperature: Arc::new(temperature),
            rainfall: Arc::new(rainfall),
            flow: Some(Arc::new(flow)),
            litho: None,
            appearance: TerrainAppearance::default(),
            color_space: ColorSpace::Srgb,
            ink: None,
            splat: None,
            ground_biomes: Vec::new(),
            ground_terrains: Vec::new(),
            paint_present: false,
            paint_biome: None,
            paint_terrain: None,
            paint_splat: None,
            grid_rgb: None,
            glacial_snowline: 0.55,
            peak_m: 4000.0,
            lapse_rate: 6.5,
            gravity: 1.0,
            rivers: None,
            river_fans: None,
            river_paint: None,
            forced_lakes: None,
            rock_detail: None,
            colour_paint: None,
            }
    }

    /// **The milestone's parity class, asserted.** The structural argument is
    /// that both paths call `render_tile`; this is the argument being
    /// exercised rather than written down, over enough tiles that a
    /// thread-count-dependent seam would have to show.
    /// Protects: the milestone's determinism claim: tiles produced by the worker pool are byte-for-byte what `render_tile` draws on the calling thread, and none is empty so the comparison cannot pass vacuously.
    #[test]
    fn worker_and_main_thread_tiles_are_byte_identical() {
        let snap = Arc::new(LodSnapshot::build(inputs(96, 72)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));

        let asked: Vec<(i32, i32, i32)> = vec![(0, 0, 0), (1, 0, 0), (1, 1, 0), (1, 0, 1), (1, 1, 1), (2, 2, 1)];
        let direct: Vec<(Vec<u8>, usize, usize)> = asked.iter().map(|&(z, c, r)| snap.render_tile(z, c, r).expect("direct tile")).collect();
        assert!(direct.iter().all(|(b, w, h)| !b.is_empty() && *w > 1 && *h > 1), "a tile that is empty would make every comparison below vacuous");

        let mut got: Vec<ReadyTile> = Vec::new();
        let mut queued = 0usize;
        // `request` refuses while the in-flight cap is full, so drive it as
        // the shell does: ask, drain, ask again. A deadline rather than an
        // iteration count -- an iteration count is a claim about how fast the
        // machine is, and 2000 spins of `yield_now` is about a millisecond.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline {
            while queued < asked.len() {
                let (z, c, r) = asked[queued];
                if !worker.request(z, c, r, 8) {
                    break;
                }
                queued += 1;
            }
            got.extend(worker.take_ready(16));
            if got.len() == asked.len() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(got.len(), asked.len(), "every requested tile must come back; got {} of {}", got.len(), asked.len());
        for t in &got {
            let i = asked.iter().position(|&a| a == (t.z, t.col, t.row)).expect("a tile nobody asked for");
            assert_eq!((t.w, t.h), (direct[i].1, direct[i].2), "tile {:?} size", asked[i]);
            assert_eq!(t.rgba, direct[i].0, "tile {:?} bytes differ between the worker and the main thread", asked[i]);
        }
    }

    /// Protects: EF-9.4's conform mask as part of a tile's identity on the
    /// worker. A plain and a conformed request for ONE chunk are two jobs (the
    /// second is not refused as "already in flight"), each lands carrying its
    /// own mask, each matches the main thread's `render_tile_conformed` byte for
    /// byte, and the two differ (the mask reached the heights) -- while
    /// `render_tile_conformed(.., 0)` is `render_tile` and the synchronous
    /// `tile_conformed` agrees with both paths.
    #[test]
    fn a_conformed_and_a_plain_request_for_one_chunk_are_two_tiles() {
        let snap = Arc::new(LodSnapshot::build(inputs(96, 72)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        // Tile (z5, col 14, row 10) sits on the fixture dome's flank, so its W,
        // NW and SW neighbours (bits 3, 0, 5) are land with relief to pull
        // toward. Level 5, not shallower: at or below `lod_bridge::z_base()` (4)
        // `add_zoom_detail` adds nothing, so a level and its parent are the same
        // field and conforming is a no-op (measured: a z3 tile came back
        // byte-identical).
        let (z, c, r, mask) = (5, 14, 10, 0b0010_1001u8);
        let plain = snap.render_tile(z, c, r).expect("plain");
        assert_eq!(snap.render_tile_conformed(z, c, r, 0), Some(plain.clone()), "mask 0 is the plain tile");
        let conf = snap.render_tile_conformed(z, c, r, mask).expect("conformed");
        assert!(conf.0 != plain.0, "the mask must move the tile, or this test proves nothing");
        assert_eq!(worker.tile_conformed(&snap, z, c, r, mask), Some(conf.clone()), "synchronous path");
        assert!(worker.request_conformed(z, c, r, 0, 8));
        assert!(worker.request_conformed(z, c, r, mask, 8), "a conformed request is not a duplicate of the plain one");
        assert!(!worker.request_conformed(z, c, r, mask, 8), "but a second conformed request is");
        let mut got: Vec<ReadyTile> = Vec::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while got.len() < 2 && std::time::Instant::now() < deadline {
            got.extend(worker.take_ready(4));
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(got.len(), 2, "both tiles must land");
        for t in &got {
            let want = if t.conform == 0 { &plain.0 } else { &conf.0 };
            assert!(t.conform == 0 || t.conform == mask, "unexpected mask {}", t.conform);
            assert!(&t.rgba == want, "worker tile with mask {} differs from the main thread", t.conform);
        }
        assert_ne!(got[0].conform, got[1].conform);
    }

    /// Protects: the plain-tile cache EF-9.4's colour half reads
    /// (`LodSnapshot::plain_rgba`). Every answer through the cache must be the
    /// answer without it, byte for byte: a conformed child (cache miss, fills
    /// its parent), its sibling (cache hit), and the parent itself asked as a
    /// plain tile (cache hit) -- each against a snapshot whose cache is never
    /// used, where `lod_bridge` synthesises the parent itself. A cache keyed on
    /// the wrong tile, or one that returned another tile's pixels, fails; the
    /// two siblings' parents are one tile, so the hit is really exercised
    /// (asserted: the cache holds exactly that one tile afterwards).
    #[test]
    fn the_plain_tile_cache_never_changes_a_pixel() {
        let snap = LodSnapshot::build(inputs(96, 72)).expect("snapshot");
        let (z, mask) = (5, 0b0010_1001u8);
        // (14, 10) and (15, 10) are siblings: both children of level-4 (7, 5).
        let a = snap.render_tile_conformed(z, 14, 10, mask).expect("child a");
        let b = snap.render_tile_conformed(z, 15, 10, mask).expect("child b");
        let p = snap.render_tile(z - 1, 7, 5).expect("parent");
        {
            let c = snap.plain_rgba.lock().expect("cache lock");
            assert_eq!(c.len(), 1, "two siblings share one cached parent");
            assert_eq!(c[0].0, (4, 7, 5), "the cached tile is the quadtree parent");
        }
        assert_eq!(Some(a), snap.render_tile_conformed_cached(z, 14, 10, mask, false), "child a, cache miss");
        assert_eq!(Some(b), snap.render_tile_conformed_cached(z, 15, 10, mask, false), "child b, cache hit");
        assert_eq!(Some(p), snap.render_tile_conformed_cached(z - 1, 7, 5, 0, false), "the parent as a plain tile, cache hit");
        // Its neighbour is NOT in the cache, and must not be served from it.
        assert_eq!(snap.render_tile(z - 1, 6, 5), snap.render_tile_conformed_cached(z - 1, 6, 5, 0, false), "a neighbour of the cached tile");
    }

    // -----------------------------------------------------------------
    // LOD-D7 -- the info-tile half of the worker
    // -----------------------------------------------------------------

    /// [`inputs`] with the lithology rasters the Age and Resistance views read.
    fn inputs_with_litho(gw: usize, gh: usize) -> SnapshotInputs {
        let n = gw * gh;
        let mut i = inputs(gw, gh);
        let ramp = |m: usize| -> Arc<Vec<f32>> { Arc::new((0..n).map(|k| (k % m) as f32 / (m - 1) as f32).collect()) };
        i.litho = Some(LithoSource { age: ramp(17), volcanic: ramp(11), crust: ramp(5), resistance: ramp(7) });
        i
    }

    /// Drive `request_info` the way the shell does (ask, drain, ask again) until
    /// `asked` has all come back or a deadline passes. A deadline, not an
    /// iteration count: an iteration count is a claim about machine speed.
    fn drain_info(worker: &Arc<LodWorker>, asked: &[(crate::info_tile::InfoView, i32, i32, i32)], cap: usize) -> Vec<ReadyInfoTile> {
        let mut got = Vec::new();
        let mut queued = 0usize;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline {
            while queued < asked.len() {
                let (v, z, c, r) = asked[queued];
                if !worker.request_info(v, z, c, r, cap) {
                    break;
                }
                queued += 1;
            }
            got.extend(worker.take_ready_info(16));
            if got.len() == asked.len() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        got
    }

    /// Protects: an info tile drawn on a worker is byte-identical to the same
    /// tile drawn on the main thread, for EVERY tiled view -- the determinism
    /// argument (both call `render_info_tile`) exercised, and each tile carries
    /// the tag of its own view. Non-empty is asserted so the comparison cannot
    /// pass vacuously.
    #[test]
    fn info_tiles_from_the_worker_equal_the_main_thread_for_every_view() {
        use crate::info_tile::TILED_VIEWS;
        let snap = Arc::new(LodSnapshot::build(inputs_with_litho(96, 72)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        let mut asked = Vec::new();
        for v in TILED_VIEWS {
            asked.push((v, 2, 1, 1));
            asked.push((v, 0, 0, 0));
        }
        let got = drain_info(&worker, &asked, 8);
        assert_eq!(got.len(), asked.len(), "every info tile must come back");
        for t in &got {
            let direct = snap.render_info_tile(t.view, t.z, t.col, t.row).expect("direct");
            assert!(!direct.0.is_empty() && direct.1 > 1, "empty tile would make this vacuous");
            assert_eq!((t.w, t.h), (direct.1, direct.2));
            assert_eq!(t.rgba, direct.0, "{} differs between worker and main thread", t.view.id());
            assert_eq!(t.tag, snap.info_tag(t.view));
        }
    }

    /// Protects: two views of the SAME tile index never collide in the worker
    /// and never share bytes or a tag -- the negative control for "a view switch
    /// never shows another view's tile". Requests Temp and Rain for one tile;
    /// both must come back, with different tags and different pixels.
    #[test]
    fn two_views_of_one_tile_are_distinct_tiles() {
        use crate::info_tile::InfoView;
        let snap = Arc::new(LodSnapshot::build(inputs_with_litho(96, 72)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        // Both asked back to back, with no drain between them: a duplicate check
        // keyed on (z,col,row) alone would refuse the second while the first is
        // in flight or waiting, and `drain_info`'s retry loop would hide that by
        // asking again later -- so the acceptance itself is asserted.
        assert!(worker.request_info(InfoView::Temp, 1, 0, 0, 8), "temp accepted");
        assert!(worker.request_info(InfoView::Rain, 1, 0, 0, 8), "rain refused: the duplicate check ignores the view");
        let mut got = Vec::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while got.len() < 2 && std::time::Instant::now() < deadline {
            got.extend(worker.take_ready_info(16));
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(got.len(), 2, "both views must be built, not deduplicated by (z,col,row)");
        let temp = got.iter().find(|t| t.view == InfoView::Temp).expect("temp");
        let rain = got.iter().find(|t| t.view == InfoView::Rain).expect("rain");
        assert_ne!(temp.tag, rain.tag);
        assert!(temp.tag.contains(";temp;") && rain.tag.contains(";rain;"), "{} / {}", temp.tag, rain.tag);
        assert_ne!(temp.rgba, rain.rgba, "different views must not paint the same pixels");
    }

    /// Protects: the info tag moves with every input an info tile reads, and
    /// only with those -- changed one at a time, each must change the tag, so a
    /// sculpt, a climate change, a lithology change or a scale change can never
    /// leave a stale tile looking current. (The view id is covered by
    /// `two_views_of_one_tile_are_distinct_tiles`.)
    #[test]
    fn the_info_tag_moves_with_every_input_an_info_tile_reads() {
        use crate::info_tile::InfoView;
        let base = LodSnapshot::build(inputs_with_litho(48, 36)).expect("base").info_tag(InfoView::Flow);
        let tag = |f: &dyn Fn(&mut SnapshotInputs)| {
            let mut i = inputs_with_litho(48, 36);
            f(&mut i);
            LodSnapshot::build(i).expect("variant").info_tag(InfoView::Flow)
        };
        assert_eq!(tag(&|_| {}), base, "an unchanged world must keep its tag");
        assert_ne!(tag(&|i| i.sea_level = 0.43), base, "sea level");
        assert_ne!(tag(&|i| i.peak_m = 4100.0), base, "peak_m");
        assert_ne!(tag(&|i| i.map_width_km = 900.0), base, "map width");
        assert_ne!(tag(&|i| i.seed = 99), base, "seed");
        assert_ne!(tag(&|i| i.temperature = Arc::new(vec![1.0; 48 * 36])), base, "temperature");
        assert_ne!(tag(&|i| i.rainfall = Arc::new(vec![0.9; 48 * 36])), base, "rainfall");
        assert_ne!(tag(&|i| i.flow = Some(Arc::new(vec![7.0; 48 * 36]))), base, "flow");
        assert_ne!(tag(&|i| i.field = Arc::new(vec![0.6; 48 * 36])), base, "field");
        assert_ne!(
            tag(&|i| {
                if let Some(l) = i.litho.as_mut() {
                    l.age = Arc::new(vec![0.9; 48 * 36]);
                }
            }),
            base,
            "crust age"
        );
        assert_ne!(
            tag(&|i| {
                if let Some(l) = i.litho.as_mut() {
                    l.resistance = Arc::new(vec![0.1; 48 * 36]);
                }
            }),
            base,
            "rock resistance"
        );
    }

    /// Protects: the info path obeys the world-version rule -- a tile landing
    /// after `invalidate` is dropped, never handed to the shell, and the info
    /// containers are emptied with the terrain ones.
    #[test]
    fn an_info_tile_landing_after_an_invalidate_is_dropped() {
        use crate::info_tile::InfoView;
        let snap = Arc::new(LodSnapshot::build(inputs_with_litho(64, 48)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        assert!(worker.request_info(InfoView::Slope, 3, 1, 1, 4));
        worker.invalidate();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline && worker.info_stats().0 != 0 {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(worker.info_stats().0, 0, "the job never finished, so this would pass vacuously");
        assert!(worker.take_ready_info(16).is_empty(), "a superseded tile reached the shell");
        let (_, waiting, built, _) = worker.info_stats();
        assert_eq!((waiting, built), (0, 0));
    }

    /// Protects: the in-flight cap is SHARED -- terrain jobs count against an
    /// info request and vice versa, so an info view cannot double a tier's
    /// background work; and a view whose input is absent is refused, not queued.
    #[test]
    fn info_requests_share_the_cap_and_refuse_missing_inputs() {
        use crate::info_tile::InfoView;
        let snap = Arc::new(LodSnapshot::build(inputs(160, 128)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        assert!(!worker.request_info(InfoView::Age, 0, 0, 0, 8), "no litho in this snapshot: Age is not available");
        assert!(worker.request(5, 3, 3, 1), "one terrain job fills a cap of 1");
        assert!(!worker.request_info(InfoView::Temp, 5, 3, 3, 1), "the cap is shared, so the info request is refused");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline && worker.stats().2 != 0 {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(worker.request_info(InfoView::Temp, 5, 3, 3, 1), "accepted once the terrain job has landed");
    }

    /// A result that lands after the world has moved on must be discarded,
    /// not installed. Driven through `invalidate`, which is what a world
    /// release does.
    /// Protects: a job that finishes after `invalidate` bumps the generation is discarded, never handed to `take_ready`, and counts as neither built nor live.
    #[test]
    fn a_tile_landing_after_an_invalidate_is_dropped() {
        let snap = Arc::new(LodSnapshot::build(inputs(64, 48)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        assert!(worker.request(1, 0, 0, 4), "the first request must be accepted");
        worker.invalidate();
        // Drain long enough that the job has certainly finished.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline {
            let (_, _, in_flight, _, _, _, _) = worker.stats();
            if in_flight == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(worker.stats().2, 0, "the job never finished, so this test would pass vacuously");
        assert!(worker.take_ready(16).is_empty(), "a tile built for a superseded generation must never be handed to the shell");
        let (ready_snap, building, in_flight, waiting, built, _dropped, bytes) = worker.stats();
        assert!(!ready_snap && !building && in_flight == 0 && waiting == 0, "invalidate leaves nothing live");
        assert_eq!(built, 0, "nothing counts as built across an invalidate");
        assert_eq!(bytes, 0, "no snapshot, no retained bytes");
    }

    /// `snapshot_for` is the key check the synchronous path relies on: a
    /// wrong key must miss, or a tile would be coloured with the previous
    /// world's precomputes — the whole-map colour error LOD-D2's cache key
    /// exists to prevent.
    /// Protects: `snapshot_for` hits only on the exact key: a prefix, the empty key and another world's key all miss.
    #[test]
    fn snapshot_for_matches_on_the_key_and_only_on_the_key() {
        let snap = Arc::new(LodSnapshot::build(inputs(48, 36)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(snap);
        assert!(worker.snapshot_for("test").is_some(), "its own key must hit");
        assert!(worker.snapshot_for("tes").is_none(), "a prefix must not hit");
        assert!(worker.snapshot_for("").is_none(), "the empty key must not hit");
        assert!(worker.snapshot_for("other").is_none(), "another world's key must not hit");
    }

    /// The in-flight cap is a cap. Asserted by asking for more than it allows
    /// and counting the refusals, because a cap that is never reached is a
    /// number nobody checks.
    /// Protects: `request` refuses once `max_in_flight` jobs are outstanding, so a viewful of requests cannot queue unbounded work.
    #[test]
    fn the_in_flight_cap_refuses_rather_than_queueing_without_bound() {
        let snap = Arc::new(LodSnapshot::build(inputs(160, 128)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(snap);
        let mut accepted = 0;
        for col in 0..32 {
            if worker.request(3, col, 0, 2) {
                accepted += 1;
            }
        }
        assert!(accepted <= 32, "sanity");
        // Two in flight at most, so with 32 asked in a tight loop the cap has
        // to have refused at least once on any machine.
        assert!(accepted < 32, "the cap never engaged: {accepted} of 32 accepted");
    }

    /// The budget table, asserted as literals rather than against itself
    /// (`MISTAKES.md`: *"assert a literal, or the independent thing the value
    /// must equal"*), plus the two relationships the shell depends on.
    /// Protects: the per-tier `LodBudget` literals, `Quality` keeping the pre-D6 shell constants, a monotone ladder, and the 60 MiB memory bar against the pre-`Arc` snapshot size.
    #[test]
    fn the_tier_budgets_are_the_documented_literals() {
        let p = budget_for_tier(QualityTier::Performance);
        assert_eq!((p.tiles_per_update, p.tiles_per_catchup, p.cache_tiles, p.max_in_flight), (16, 3, 32, 2));
        assert_eq!(p.max_level, 9, "Performance caps one level shallower than MAX_LEVEL = 10");
        let b = budget_for_tier(QualityTier::Balanced);
        assert_eq!((b.tiles_per_update, b.tiles_per_catchup, b.cache_tiles, b.max_in_flight, b.max_level), (32, 4, 32, 3, 10));
        let q = budget_for_tier(QualityTier::Quality);
        assert_eq!((q.tiles_per_update, q.tiles_per_catchup, q.cache_tiles, q.max_in_flight, q.max_level), (48, 6, 35, 4, 10));
        let u = budget_for_tier(QualityTier::Ultra);
        assert_eq!((u.tiles_per_update, u.tiles_per_catchup, u.cache_tiles, u.max_in_flight, u.max_level), (48, 8, 35, 6, 10));
        // EF-9.4's toggle (Ruling CL, 2026-10-18): on for the three desktop
        // tiers, off for Performance (the Android tier, no phone timing yet;
        // see `LodBudget::ef9_refine`) -- a literal per tier, so changing one
        // is a deliberate, test-visible edit.
        assert_eq!([p.ef9_refine, b.ef9_refine, q.ef9_refine, u.ef9_refine], [false, true, true, true]);
        // **The memory bar, as a test rather than as a comment.** 60 MiB is
        // 62 914 560 B; the snapshot at 2048x1311 measured 53 698 560 B
        // (`_d6async_probe.gd` section 9, 2026-09-21) and a parked tile is
        // 256*256*4. A tier whose cache pushes the pair over the bar is a
        // regression this catches, and raising any `cache_tiles` above 35 with
        // that snapshot size fails it.
        //
        // **That 53 698 560 is the PRE-`Arc` figure and is deliberately kept.**
        // Four of the five grids it counted are shared with `WorldState` now
        // and cost nothing, so the real snapshot is smaller and this is an
        // UPPER BOUND: the guard still refuses a `cache_tiles` rise, which is
        // its whole job, and it does so without a number nobody re-measured.
        // Re-taking it needs `_d6async_probe.gd` section 9 and a rebuilt
        // `cartalith_godot.dll`; substituting arithmetic for that run is the
        // exact mistake the paragraph above this test records twice.
        const BAR: usize = 60 * 1024 * 1024;
        // 53 698 560 B: the snapshot's retained size at 2048x1311, measured by
        // `_d6async_probe.gd` section 9 on 2026-09-21 and kept as the PRE-`Arc` upper
        // bound (see the paragraph above).
        const SNAPSHOT_2048X1311: usize = 53_698_560;
        // One parked tile: 256 x 256 px RGBA8 (`TILE_PX` = 256, 4 bytes per pixel).
        const TILE_BYTES: usize = 256 * 256 * 4;
        for t in QualityTier::ALL {
            let worst = SNAPSHOT_2048X1311 + budget_for_tier(t).cache_tiles * TILE_BYTES;
            assert!(worst <= BAR, "{:?}'s worst-case LOD-D6 memory is {worst} B, over the milestone's {BAR} B bar", t);
        }
        // The pre-D6 shell constants, as literals: `Quality` must schedule
        // exactly what `viewport_host.gd` scheduled before this milestone, or
        // a measured difference cannot be attributed to the threading.
        assert_eq!(q.tiles_per_update, 48, "MAX_LOD_TILES_PER_UPDATE before LOD-D6");
        assert_eq!(q.tiles_per_catchup, 6, "MAX_LOD_TILES_PER_CATCHUP before LOD-D6");
        // Monotonic in the direction the ladder means, over the tier order
        // `QualityTier::ALL` already fixes.
        let all: Vec<LodBudget> = QualityTier::ALL.iter().map(|&t| budget_for_tier(t)).collect();
        for w in all.windows(2) {
            assert!(w[1].tiles_per_update >= w[0].tiles_per_update, "tile budget must not fall as the tier rises");
            assert!(w[1].cache_tiles >= w[0].cache_tiles, "cache must not shrink as the tier rises");
            assert!(w[1].max_level >= w[0].max_level, "a richer tier must not be shallower");
        }
    }

    /// `retained_bytes` is what the memory bar is read off, so it has to
    /// count the right things — and since the four world grids became `Arc`s
    /// shared with `WorldState`, the right count for them is **zero**. This
    /// used to assert `n * 4 * 4`; it asserts the opposite now, and the rest
    /// of it proves the counter is not merely broken: the buffer is asserted
    /// to be the same allocation, and something that IS copied is still
    /// counted.
    /// Protects: `retained_bytes` is zero for the four shared world grids (same allocation, `Arc::ptr_eq`) and counts a copied one-byte-per-cell ink exactly.
    #[test]
    fn retained_bytes_counts_only_what_the_snapshot_copied() {
        let (gw, gh) = (64usize, 48usize);
        let n = gw * gh;
        let i = inputs(gw, gh);
        let held = i.field.clone();
        let snap = LodSnapshot::build(i).expect("snapshot");
        assert!(Arc::ptr_eq(&held, &snap.field), "the snapshot must hold the SAME buffer as its input, not a copy of it");
        assert_eq!(snap.retained_bytes(), 0, "the four world grids are shared, not copied: a pack-less, paint-less, ink-less world costs nothing");
        let mut no_flow = inputs(gw, gh);
        no_flow.flow = None;
        no_flow.ink = Some(vec![0u8; n]);
        let snap2 = LodSnapshot::build(no_flow).expect("snapshot");
        assert_eq!(snap2.retained_bytes(), n, "a one-byte-per-cell flag ink is copied, and is the only thing counted");
    }

    /// [`LithoSource`]'s four grids are **shared with `WorldState`, not
    /// copied** — the 43.0 MB transient peak a snapshot used to pay for them.
    ///
    /// No test passed `litho: Some(..)` at all before this one, so
    /// [`LodSnapshot::build`]'s `build_lithology` branch was unexercised as
    /// well; this asserts the sharing, that the branch still produces a real
    /// per-cell lithology, and — via `strong_count` — that the build drops
    /// the source, which is what makes the cost a peak rather than a
    /// residency.
    /// Protects: a `LithoSource` is shared not copied, still yields a per-cell lithology, keeps only the age and resistance grids for the info views, and drops volcanic and crust after the build.
    #[test]
    fn a_litho_source_shares_its_grids_and_still_builds_a_lithology() {
        let (gw, gh) = (64usize, 48usize);
        let n = gw * gh;
        let mut i = inputs(gw, gh);
        let age = Arc::new(vec![0.7f32; n]);
        let volcanic = Arc::new(vec![0.9f32; n]);
        let crust = Arc::new(vec![-0.2f32; n]);
        let resistance = Arc::new(vec![0.5f32; n]);
        i.litho = Some(LithoSource {
            age: Arc::clone(&age),
            volcanic: Arc::clone(&volcanic),
            crust: Arc::clone(&crust),
            resistance: Arc::clone(&resistance),
        });
        {
            let l = i.litho.as_ref().expect("just set");
            for (held, got, name) in [(&age, &l.age, "age"), (&volcanic, &l.volcanic, "volcanic"), (&crust, &l.crust, "crust"), (&resistance, &l.resistance, "resistance")] {
                assert!(Arc::ptr_eq(held, got), "{name} must be the SAME allocation the world holds, not a copy of it");
            }
        }
        let snap = LodSnapshot::build(i).expect("snapshot");
        let lith = snap.lithology.as_ref().expect("a LithoSource must produce a lithology");
        assert_eq!(lith.len(), n, "one rock class per cell");
        // LOD-D7 (Ruling AV): the Age and Resistance info views read those two
        // rasters, so the snapshot keeps one SHARED clone of each (the same
        // allocation, not a copy: no steady memory, `retained_bytes` below stays
        // 0). `volcanic` and `crust` are only inputs to the lithology and are
        // still dropped by the build.
        for (held, kept, name) in [(&age, snap.age.as_ref(), "age"), (&resistance, snap.resistance.as_ref(), "resistance")] {
            assert!(Arc::ptr_eq(held, kept.expect("the info views' input is kept")), "{name}: the snapshot's copy must be the world's allocation");
            assert_eq!(Arc::strong_count(held), 2, "{name}: this test plus the snapshot's shared clone, and nothing else");
        }
        for (held, name) in [(&volcanic, "volcanic"), (&crust, "crust")] {
            assert_eq!(Arc::strong_count(held), 1, "{name}: the build must drop the LithoSource, leaving only this test holding it");
        }
        assert_eq!(snap.retained_bytes(), 0, "a lithology is built, not copied, so the substrate costs no retained bytes");
    }

    /// A degenerate grid must return `None` rather than panic — this runs on
    /// a worker thread, where a panic has no `#[func]` to return through.
    /// Protects: `LodSnapshot::build` returns `None` (no panic on the worker thread) for a one-column grid or a height field shorter than its grid.
    #[test]
    fn a_degenerate_grid_refuses_to_build() {
        let mut i = inputs(8, 8);
        i.gw = 1;
        assert!(LodSnapshot::build(i).is_none(), "a one-column grid is not a grid");
        let mut i = inputs(8, 8);
        Arc::make_mut(&mut i.field).truncate(3);
        assert!(LodSnapshot::build(i).is_none(), "a height field shorter than the grid it claims");
    }

    /// An out-of-range tile index comes back as `None` from the worker path
    /// too, and is counted as dropped rather than silently forgotten.
    /// Protects: a request for a tile outside the level is accepted, finishes with nothing built, and is counted as dropped rather than left in flight.
    #[test]
    fn an_out_of_range_tile_is_dropped_not_hung() {
        let snap = Arc::new(LodSnapshot::build(inputs(48, 36)).expect("snapshot"));
        let worker = Arc::new(LodWorker::default());
        worker.install(Arc::clone(&snap));
        assert!(snap.render_tile(1, 9, 9).is_none(), "the direct path refuses an index outside the level");
        assert!(worker.request(1, 9, 9, 4), "the request itself is accepted -- the refusal happens in the job");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline {
            let (_, _, in_flight, _, _, dropped, _) = worker.stats();
            if in_flight == 0 && dropped > 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let (_, _, in_flight, waiting, built, dropped, _) = worker.stats();
        assert_eq!((in_flight, waiting, built), (0, 0, 0), "nothing left in flight, nothing to hand over, nothing built");
        assert_eq!(dropped, 1, "the refusal is counted");
    }

    // -- owner rulings 28/29: the stored pyramid, real end to end ---------

    /// [`LodSnapshot::render_pyramid_masks`] produces **real, non-trivial**
    /// tile data — not a placeholder — whose total byte count matches
    /// [`lod_bridge::pyramid_mask_bytes`]'s save-time estimate exactly, and
    /// which survives a genuine `cartalith_io::project` write/read cycle
    /// intact: every tile entry present, `ProjectData::lod_tiles` populated
    /// on read, and nothing dropped as stale (the empty `source_key` this
    /// test leaves, like the real save path does, is exactly the "a fresh
    /// producer" contract `LodTiles::source_key`'s own doc names).
    /// Protects: the pyramid masks are real data, match `pyramid_mask_bytes`'s estimate, and survive a real project write and read byte-exactly with no warnings.
    #[test]
    fn render_pyramid_masks_round_trips_through_the_project_archive() {
        let (gw, gh) = (48usize, 36usize);
        let inp = inputs(gw, gh);
        let heightmap: Vec<f32> = (*inp.field).clone();
        let snap = LodSnapshot::build(inp).expect("snapshot");

        let z_max = 2;
        let (tile_w, tile_h, tiles) = snap.render_pyramid_masks(z_max).expect("pyramid synthesizes");
        let expected_tile_count: usize =
            (0..=z_max).map(|z| (lod_bridge::tiles_per_axis(z) as usize).pow(2)).sum();
        assert_eq!(tiles.len(), expected_tile_count, "one tile per (z, col, row) across the whole pyramid");
        assert!(tile_w > 1 && tile_h > 1, "a degenerate tile size would make the byte checks vacuous");
        assert!(tiles.values().all(|t| t.len() == tile_w * tile_h * 3), "every mask must be tile_w*tile_h*3 RGB bytes");
        // Not every byte zero -- a placeholder/fake payload would still pass
        // the length checks above.
        assert!(tiles.values().any(|t| t.iter().any(|&b| b != 0)), "a real tile must carry real colour, not an all-zero placeholder");

        let raw_total: u64 = tiles.values().map(|t| t.len() as u64).sum();
        let estimate = lod_bridge::pyramid_mask_bytes(gw, gh, z_max).expect("estimate for a real grid");
        assert_eq!(raw_total, estimate, "the save-time size estimate must equal the real byte count");

        let params = cartalith_io::SaveParams {
            gw,
            gh,
            seed: 1234,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: Arc::new(heightmap),
            temperature: Arc::new(vec![15.0f32; gw * gh]),
            rainfall: Arc::new(vec![0.5f32; gw * gh]),
            volcanic_field: vec![0.0f32; gw * gh],
            impact_field: vec![0.0f32; gw * gh],
            strahler_order: vec![0u8; gw * gh],
        };
        let mut write = cartalith_io::project::ProjectWrite::new(&params, &fields);
        write.lod_tiles = Some(cartalith_io::project::LodTiles {
            source_key: String::new(),
            producer: lod_bridge::tile_producer_id(&TerrainAppearance::default()),
            tile_w,
            tile_h,
            tiles: tiles.clone(),
        });

        let mut buf: Vec<u8> = Vec::new();
        let warnings = cartalith_io::project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write succeeds");
        assert!(warnings.is_empty(), "a fresh producer's empty source_key must be trusted, not warned about: {warnings:?}");

        {
            let mut zr = zip::ZipArchive::new(std::io::Cursor::new(&buf)).expect("a valid zip");
            // `.ends_with(".u8")` excludes `LOD_TILE_INDEX`
            // (`cartography/tiles/index.json`), which also starts with the
            // prefix but is the one index entry, not a tile.
            let tile_entries = (0..zr.len())
                .filter(|&i| {
                    let name = zr.by_index_raw(i).unwrap().name().to_string();
                    name.starts_with(cartalith_io::project::LOD_TILE_PREFIX) && name.ends_with(".u8")
                })
                .count();
            assert_eq!(tile_entries, tiles.len(), "one archive entry per tile");
        }

        let data = cartalith_io::project::read_project(std::io::Cursor::new(&buf)).expect("read succeeds");
        assert!(data.warnings.is_empty(), "a just-written archive must read back with no warnings: {:?}", data.warnings);
        let got = data.lod_tiles.expect("the pyramid must round-trip, not be dropped as stale");
        assert_eq!(got.tile_w, tile_w);
        assert_eq!(got.tile_h, tile_h);
        assert_eq!(got.tiles.len(), tiles.len());
        assert_eq!(got.tiles, tiles, "every tile's bytes must survive the round trip exactly");
    }

    /// The **default** path — `include_lod_tiles` never set, so
    /// `ProjectWrite::lod_tiles` stays `None` — must write no
    /// `cartography/tiles/**` entries and no index at all, matching every
    /// save before this feature existed (ruling 28: *"off by default"*).
    /// This is the "structurally identical to today's un-ticked save" half
    /// of the round-trip check.
    /// Protects: a save with `lod_tiles = None` writes no LOD entries and reads back clean, matching every pre-feature save (ruling 28, off by default).
    #[test]
    fn no_lod_tiles_means_no_cartography_entries_at_all() {
        let (gw, gh) = (8usize, 8usize);
        let params = cartalith_io::SaveParams {
            gw,
            gh,
            seed: 1,
            map_width_km: 100.0,
            sea_level: 0.4,
            world: false,
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: Arc::new(vec![0.5f32; gw * gh]),
            temperature: Arc::new(vec![15.0f32; gw * gh]),
            rainfall: Arc::new(vec![0.5f32; gw * gh]),
            volcanic_field: vec![0.0f32; gw * gh],
            impact_field: vec![0.0f32; gw * gh],
            strahler_order: vec![0u8; gw * gh],
        };
        // `lod_tiles` is `None` by construction (`ProjectWrite::new`'s own
        // contract) -- never set here, the same as every save before this
        // feature existed.
        let write = cartalith_io::project::ProjectWrite::new(&params, &fields);
        let mut buf: Vec<u8> = Vec::new();
        cartalith_io::project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write succeeds");

        let mut zr = zip::ZipArchive::new(std::io::Cursor::new(&buf)).expect("a valid zip");
        let names: Vec<String> = (0..zr.len()).map(|i| zr.by_index_raw(i).unwrap().name().to_string()).collect();
        assert!(
            names.iter().all(|n| !n.starts_with(cartalith_io::project::LOD_TILE_PREFIX) && n != cartalith_io::project::LOD_TILE_INDEX),
            "no LOD entries at all when lod_tiles is None: {names:?}"
        );

        // And a pre-existing archive with no LOD entries at all -- the exact
        // shape of every project saved before this feature existed -- still
        // opens clean, with no pyramid and no warning about one.
        let data = cartalith_io::project::read_project(std::io::Cursor::new(&buf)).expect("read succeeds");
        assert!(data.lod_tiles.is_none(), "no tiles were written, so none must be read back");
        assert!(data.warnings.is_empty(), "an archive with no LOD entries is not a damaged one: {:?}", data.warnings);
    }

    // -- owner rulings 28/29: the stored pyramid, read back and seeded ------

    /// The `SaveParams` of the 1234-seed, 800 km, sea 0.42 test world at `gw` x
    /// `gh`; the values mirror [`inputs`] so an archive built from it matches.
    fn save_params(gw: usize, gh: usize) -> cartalith_io::SaveParams {
        cartalith_io::SaveParams { gw, gh, seed: 1234, map_width_km: 800.0, sea_level: 0.42, world: false, origin: None, name: None }
    }

    /// A `SaveFields` around `heightmap` with a flat 15.0 temperature and 0.5
    /// rainfall (arbitrary fillers: the pyramid tests read only the heightmap).
    fn save_fields(heightmap: Vec<f32>) -> cartalith_io::SaveFields {
        let n = heightmap.len();
        cartalith_io::SaveFields {
            heightmap: Arc::new(heightmap),
            temperature: Arc::new(vec![15.0f32; n]),
            rainfall: Arc::new(vec![0.5f32; n]),
            volcanic_field: vec![0.0f32; n],
            impact_field: vec![0.0f32; n],
            strahler_order: vec![0u8; n],
        }
    }

    /// A pyramid for `snap`'s world, written through the real archive writer
    /// under `producer` and read back through the real reader — the bytes
    /// `project_open` receives, not a hand-built struct.
    fn archived(snap: &LodSnapshot, heightmap: &[f32], producer: String, z_max: i32) -> Option<cartalith_io::project::LodTiles> {
        let (tile_w, tile_h, tiles) = snap.render_pyramid_masks(z_max).expect("pyramid synthesizes");
        let params = save_params(snap.gw, snap.gh);
        let fields = save_fields(heightmap.to_vec());
        let mut write = cartalith_io::project::ProjectWrite::new(&params, &fields);
        write.lod_tiles = Some(cartalith_io::project::LodTiles { source_key: String::new(), producer, tile_w, tile_h, tiles });
        let mut buf: Vec<u8> = Vec::new();
        cartalith_io::project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write succeeds");
        cartalith_io::project::read_project(std::io::Cursor::new(&buf)).expect("read succeeds").lod_tiles
    }

    /// Every `(z, col, row)` of levels `0..=z_max`, the walk a full pyramid save
    /// makes (`1 + 4 + 16 ...` tiles).
    fn every_tile(z_max: i32) -> Vec<(i32, i32, i32)> {
        let mut v = Vec::new();
        for z in 0..=z_max {
            let n = lod_bridge::tiles_per_axis(z) as i32;
            for col in 0..n {
                for row in 0..n {
                    v.push((z, col, row));
                }
            }
        }
        v
    }

    /// **The seeded path's positive case, end to end.** A pyramid saved from
    /// one snapshot and reopened against a snapshot built from the same
    /// inputs is served back **byte-identical to what synthesis draws**, on
    /// both the synchronous path and the worker path, and nothing is
    /// synthesised while it is.
    /// Protects: a stored pyramid whose producer matches is served byte-identical to synthesis on both the sync and worker paths with nothing synthesised, and a deeper tile is still synthesised.
    #[test]
    fn a_reopened_pyramid_is_served_instead_of_synthesised() {
        let (gw, gh) = (48usize, 36usize);
        let saved = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let heightmap: Vec<f32> = (*saved.field).clone();
        let z_max = 2;
        let lod = archived(&saved, &heightmap, saved.producer_id().to_string(), z_max).expect("the pyramid survives the archive");

        // The reopened session builds its own snapshot from the same world.
        let live = Arc::new(LodSnapshot::build(inputs(gw, gh)).expect("snapshot"));
        assert_eq!(live.producer_id(), saved.producer_id(), "the same inputs must give the same producer");
        assert!(live.producer_id().contains(";in="), "the producer must carry the input digest: {}", live.producer_id());

        let worker = Arc::new(LodWorker::default());
        worker.seed(StoredPyramid::from(lod));
        let asked = every_tile(z_max);
        assert_eq!(worker.seed_stats().0, asked.len(), "every tile of the pyramid is held");
        for &(z, c, r) in &asked {
            let seeded = worker.tile(&live, z, c, r).expect("a seeded tile");
            let drawn = live.render_tile(z, c, r).expect("a synthesised tile");
            assert_eq!(seeded, drawn, "tile {:?}: the seeded bytes are not what synthesis draws", (z, c, r));
        }
        let (_, served, synthesized) = worker.seed_stats();
        assert_eq!((served, synthesized), (asked.len() as u64, 0), "every tile served from the seed, none synthesised");

        // The worker path: a seeded tile is handed straight to `ready`,
        // with no job and nothing synthesised.
        worker.install(Arc::clone(&live));
        assert!(worker.request(1, 1, 0, 1), "a seeded tile is accepted");
        let got = worker.take_ready(4);
        assert_eq!(got.len(), 1, "the seeded tile is ready at once, not queued");
        assert_eq!(got[0].rgba, live.render_tile(1, 1, 0).expect("tile").0);
        assert_eq!(worker.seed_stats().2, 0, "the worker path synthesised nothing");

        // A tile deeper than the stored pyramid is synthesised as before.
        let deeper = worker.tile(&live, z_max + 1, 0, 0).expect("a deeper tile");
        assert_eq!(deeper, live.render_tile(z_max + 1, 0, 0).expect("tile"));
        assert_eq!(worker.seed_stats().2, 1, "a tile the pyramid does not carry is synthesised");
    }

    /// **A re-save reuses the seed.** A project reopened with a still-valid
    /// pyramid and saved again with tiles writes the held masks back: nothing
    /// is synthesised, and the archive's pyramid is byte-identical to the one
    /// it was opened with. The negative control is a seed under another
    /// producer, where every tile is synthesised and still comes out the same
    /// -- so the zero above is the reuse, not an empty walk.
    /// Protects: re-saving after a seeded open reuses all 21 held masks byte-for-byte, and a seed under another producer is held but never reused.
    #[test]
    fn a_re_save_after_a_seeded_open_synthesises_nothing() {
        let (gw, gh) = (48usize, 36usize);
        let saved = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let heightmap: Vec<f32> = (*saved.field).clone();
        let z_max = 2;
        let lod = archived(&saved, &heightmap, saved.producer_id().to_string(), z_max).expect("pyramid");
        let opened_tiles = lod.tiles.clone();
        let live = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");

        let worker = LodWorker::default();
        worker.seed(StoredPyramid::from(lod.clone()));
        let (tile_w, tile_h, tiles) = worker.pyramid_masks(&live, z_max).expect("pyramid");
        let n = every_tile(z_max).len();
        assert_eq!(n, 21, "z 0..=2 is 1 + 4 + 16 tiles");
        assert_eq!(worker.seed_stats(), (21, 21, 0), "every tile reused, none synthesised");
        assert_eq!((tile_w, tile_h), (lod.tile_w, lod.tile_h));
        assert_eq!(tiles, opened_tiles, "the re-saved pyramid is the opened one, byte for byte");

        // Through the real writer: the re-saved archive's pyramid reads back
        // identical to the one the project was opened with.
        let params = save_params(gw, gh);
        let fields = save_fields(heightmap.clone());
        let mut write = cartalith_io::project::ProjectWrite::new(&params, &fields);
        write.lod_tiles = Some(cartalith_io::project::LodTiles { source_key: String::new(), producer: live.producer_id().to_string(), tile_w, tile_h, tiles });
        let mut buf: Vec<u8> = Vec::new();
        cartalith_io::project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write");
        let back = cartalith_io::project::read_project(std::io::Cursor::new(&buf)).expect("read").lod_tiles.expect("pyramid kept");
        assert_eq!(back.tiles, opened_tiles);
        assert_eq!(back.producer, lod.producer);

        // Negative control: a seed under another producer is not reused.
        let other = LodWorker::default();
        let mut stale = StoredPyramid::from(lod);
        stale.producer.push_str(";stale");
        other.seed(stale);
        let (_, _, again) = other.pyramid_masks(&live, z_max).expect("pyramid");
        assert_eq!(other.seed_stats(), (21, 0, 21), "a stale seed is held but never reused");
        assert_eq!(again, opened_tiles, "and synthesis draws the same pyramid");
    }

    /// **The backward-compatible case.** A project saved by the code before
    /// 2026-09-24 stored `tile_producer_id(appearance)` alone — no colour
    /// space, no attachments, no `;in=`. It is built here exactly as that
    /// code built it (`project_bridge.rs`'s save path at `5e09ae8`), and it
    /// must never be seeded, because nothing in it says which colour space or
    /// attachments its tiles were drawn under.
    /// Protects: a pyramid stored under the old digest-less producer id is held yet never served, since it cannot say which colour space or attachments drew it.
    #[test]
    fn a_pyramid_saved_before_the_input_digest_is_never_seeded() {
        let (gw, gh) = (48usize, 36usize);
        let saved = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let heightmap: Vec<f32> = (*saved.field).clone();
        let head_producer = lod_bridge::tile_producer_id(&TerrainAppearance::default());
        let lod = archived(&saved, &heightmap, head_producer, 1).expect("the old archive still reads, as a cache");
        let live = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let worker = LodWorker::default();
        worker.seed(StoredPyramid::from(lod));
        for (z, c, r) in every_tile(1) {
            worker.tile(&live, z, c, r).expect("synthesised");
        }
        let (held, served, synthesized) = worker.seed_stats();
        assert!(held > 0, "the old pyramid is held -- the refusal below is the producer check, not an empty seed");
        assert_eq!(served, 0, "a pre-digest pyramid must not be served");
        assert_eq!(synthesized, every_tile(1).len() as u64);
    }

    /// Every input the archive's `source_key` does not cover refuses the seed
    /// on its own: the colour space (the audit's "no colour space" gap), the
    /// look, and the reopened world's missing flow and lithology
    /// (`lod_snapshot_inputs`' `Loaded` arm). Each is one change from the
    /// saving snapshot; the negative control at the end shows the same seed
    /// IS served when nothing changed, so the refusals are not vacuous.
    /// Protects: changing colour space, appearance, flow, ink or paint each moves the producer and refuses the seed, while unchanged inputs are served (the control).
    #[test]
    fn a_seed_drawn_under_other_inputs_is_not_served() {
        let (gw, gh) = (48usize, 36usize);
        let saved = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let heightmap: Vec<f32> = (*saved.field).clone();
        let lod = archived(&saved, &heightmap, saved.producer_id().to_string(), 1).expect("pyramid");

        let variants: Vec<(&str, SnapshotInputs)> = vec![
            ("colour space", { let mut i = inputs(gw, gh); i.color_space = ColorSpace::DisplayP3; i }),
            ("appearance", { let mut i = inputs(gw, gh); i.appearance.exag += 0.5; i }),
            ("no flow (a reopened world)", { let mut i = inputs(gw, gh); i.flow = None; i }),
            ("river ink", { let mut i = inputs(gw, gh); i.ink = Some(vec![1u8; gw * gh]); i }),
            ("paint", { let mut i = inputs(gw, gh); i.paint_present = true; i.paint_biome = Some(vec![2u8; gw * gh]); i }),
        ];
        for (what, i) in variants {
            let live = LodSnapshot::build(i).expect("snapshot");
            assert_ne!(live.producer_id(), saved.producer_id(), "{what}: the producer did not move");
            let worker = LodWorker::default();
            worker.seed(StoredPyramid::from(lod.clone()));
            let got = worker.tile(&live, 1, 0, 0).expect("tile");
            assert_eq!(got, live.render_tile(1, 0, 0).expect("tile"), "{what}: not the live world's tile");
            assert_eq!(worker.seed_stats().1, 0, "{what}: a tile drawn under other inputs was served");
        }
        let live = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let worker = LodWorker::default();
        worker.seed(StoredPyramid::from(lod));
        worker.tile(&live, 1, 0, 0).expect("tile");
        assert_eq!(worker.seed_stats().1, 1, "negative control: unchanged inputs must be served");
    }

    /// **The key-mismatch case**: an archive whose stored `source_key` names
    /// a different world than the heightmap beside it — a project re-sculpted
    /// by another tool that kept the old tiles — reaches `project_open` with
    /// no pyramid at all, so there is nothing to seed. Built by rewriting a
    /// real archive's index, not by constructing a `ProjectData`.
    /// Protects: an archive whose index `source_key` names another world is dropped at read with a warning, so nothing is seeded and the tile is synthesised.
    #[test]
    fn a_source_key_mismatch_leaves_nothing_to_seed() {
        use std::io::{Read, Write};
        let (gw, gh) = (48usize, 36usize);
        let saved = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let heightmap: Vec<f32> = (*saved.field).clone();
        let (tile_w, tile_h, tiles) = saved.render_pyramid_masks(1).expect("pyramid");
        let params = save_params(gw, gh);
        let fields = save_fields(heightmap.clone());
        let mut write = cartalith_io::project::ProjectWrite::new(&params, &fields);
        write.lod_tiles = Some(cartalith_io::project::LodTiles { source_key: String::new(), producer: saved.producer_id().to_string(), tile_w, tile_h, tiles });
        let mut buf: Vec<u8> = Vec::new();
        cartalith_io::project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write");
        assert!(cartalith_io::project::read_project(std::io::Cursor::new(&buf)).expect("read").lod_tiles.is_some(), "precondition: the untampered archive keeps its pyramid");

        // The same archive with the index's key swapped for another world's.
        let mut other = heightmap.clone();
        other[0] += 0.25;
        let other_key = cartalith_io::project::lod_source_key(&params, &other);
        let mut zr = zip::ZipArchive::new(std::io::Cursor::new(&buf)).expect("zip");
        let mut out: Vec<u8> = Vec::new();
        {
            let mut zw = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
            let opts = zip::write::SimpleFileOptions::default();
            let mut swapped = false;
            for k in 0..zr.len() {
                let mut f = zr.by_index(k).expect("entry");
                let name = f.name().to_string();
                let mut bytes = Vec::new();
                f.read_to_end(&mut bytes).expect("read entry");
                if name == cartalith_io::project::LOD_TILE_INDEX {
                    let mut v: serde_json::Value = serde_json::from_slice(&bytes).expect("index json");
                    v["source_key"] = serde_json::Value::String(other_key.clone());
                    bytes = serde_json::to_vec(&v).expect("json");
                    swapped = true;
                }
                zw.start_file(name, opts).expect("start");
                zw.write_all(&bytes).expect("write entry");
            }
            zw.finish().expect("finish");
            assert!(swapped, "the archive had no LOD index to tamper with");
        }
        let data = cartalith_io::project::read_project(std::io::Cursor::new(&out)).expect("the archive still opens");
        assert!(data.lod_tiles.is_none(), "a pyramid keyed to another world must be dropped at read");
        assert!(data.warnings.iter().any(|w| w.contains("cartography/tiles")), "and the drop is reported: {:?}", data.warnings);

        // What `project_open` does with it: nothing to hold, nothing served.
        let worker = LodWorker::default();
        if let Some(l) = data.lod_tiles {
            worker.seed(StoredPyramid::from(l));
        }
        let live = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        worker.tile(&live, 0, 0, 0).expect("tile");
        assert_eq!(worker.seed_stats(), (0, 0, 1), "nothing held, nothing served, the tile synthesised");
    }

    /// The digest is derived from the definition, and every field it reads
    /// moves it. One mutation per field class; `key` must NOT move it.
    /// Protects: `fingerprint` moves with each tile input (including forced lakes and which cells) and not with the session `key`, and ignores `grid_rgb` while local contrast is off.
    #[test]
    fn the_input_fingerprint_moves_with_every_tile_input_and_not_with_the_key() {
        let (gw, gh) = (16usize, 12usize);
        let base = inputs(gw, gh).fingerprint();
        let mut i = inputs(gw, gh);
        i.key = "another session's key".to_string();
        assert_eq!(i.fingerprint(), base, "the session key must not move the digest");
        let mut moved: Vec<(&str, SnapshotInputs)> = Vec::new();
        moved.push(("field", { let mut i = inputs(gw, gh); Arc::make_mut(&mut i.field)[5] += 0.01; i }));
        moved.push(("temperature", { let mut i = inputs(gw, gh); Arc::make_mut(&mut i.temperature)[5] += 1.0; i }));
        moved.push(("rainfall", { let mut i = inputs(gw, gh); Arc::make_mut(&mut i.rainfall)[5] += 0.1; i }));
        moved.push(("sea_level", { let mut i = inputs(gw, gh); i.sea_level = 0.43; i }));
        moved.push(("seed", { let mut i = inputs(gw, gh); i.seed = 1235; i }));
        moved.push(("map_width_km", { let mut i = inputs(gw, gh); i.map_width_km = 801.0; i }));
        moved.push(("colour space", { let mut i = inputs(gw, gh); i.color_space = ColorSpace::DisplayP3; i }));
        moved.push(("peak_m", { let mut i = inputs(gw, gh); i.peak_m = 4001.0; i }));
        moved.push(("litho", { let mut i = inputs(gw, gh); let n = gw * gh; i.litho = Some(LithoSource { age: Arc::new(vec![0.1; n]), volcanic: Arc::new(vec![0.1; n]), crust: Arc::new(vec![0.1; n]), resistance: Arc::new(vec![0.1; n]) }); i }));
        moved.push(("grid_rgb under local contrast", { let mut i = inputs(gw, gh); i.appearance.local_contrast = 0.5; i.grid_rgb = Some(vec![9u8; gw * gh * 3]); i }));
        // Ruling BO: a forced lake, and a different forced lake.
        moved.push(("forced lakes", { let mut i = inputs(gw, gh); let mut m = vec![0u8; gw * gh]; m[5] = 1; i.forced_lakes = Some(m); i }));
        for (what, i) in moved {
            assert_ne!(i.fingerprint(), base, "{what} did not move the digest");
        }
        // Gated inputs: a finished grid raster with local contrast OFF is
        // never read by a tile, so it must not refuse a valid seed.
        let mut i = inputs(gw, gh);
        i.appearance.local_contrast = 0.0;
        let off = i.fingerprint();
        i.grid_rgb = Some(vec![9u8; gw * gh * 3]);
        assert_eq!(i.fingerprint(), off, "grid_rgb must not count while local contrast is off");
        // Which cells are forced moves it too, not just whether any are.
        let at = |c: usize| {
            let mut i = inputs(gw, gh);
            let mut m = vec![0u8; gw * gh];
            m[c] = 1;
            i.forced_lakes = Some(m);
            i.fingerprint()
        };
        assert_ne!(at(5), at(6), "the forced cells must move the digest");
    }

    /// The inputs of a coastal world with one order-4 river due east into the
    /// sea and its RIM-4 delta fans: land west of x = 64, sea east of it, the
    /// river along y = 36.5 from x = 10.5 past the shore. Returns the inputs
    /// with the river and no fans, and the fans.
    fn delta_inputs(gw: usize, gh: usize) -> (SnapshotInputs, crate::river_stroke::RiverGeometry) {
        let mut i = inputs(gw, gh);
        {
            let f = Arc::make_mut(&mut i.field);
            for y in 0..gh {
                for x in 0..gw {
                    f[y * gw + x] = if x < 64 { 0.5 + 0.002 * (64 - x) as f32 } else { 0.3 };
                }
            }
        }
        let n = 58usize;
        let net = crate::river_stroke::RiverGeometry {
            runs: vec![crate::river_stroke::DrawnRun {
                pts: (0..n).map(|k| (10.5 + k as f32, 36.5)).collect(),
                widths: vec![2.0; n],
                colors: vec![[0.1, 0.2, 0.6, 1.0]; n],
                orders: vec![4; n],
                discharge: vec![5000.0; n],
                pieces: vec![(0, n)],
                reach: vec![(None, Some(crate::river_stroke::ShoreReach::default()))],
                own_order: 4,
            }],
        };
        let field = i.field.clone();
        let wet = move |x: i64, y: i64| x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh && field[y as usize * gw + x as usize] < 0.42;
        let (fans, st) = crate::river_delta::delta_fans(&net, gw, gh, 1.0, &wet);
        assert!(st.branches >= 2, "premise: the mouth fans: {st:?}");
        i.rivers = Some(Arc::new(net));
        (i, fans)
    }

    /// Protects: **RIM-7 in the worker path and the stored-pyramid key.** A
    /// snapshot handed RIM-4's delta fans (`SnapshotInputs::river_fans`, what
    /// `WorldGen::lod_snapshot_inputs` passes) draws them in its tiles -- the
    /// tile over the mouth differs from the fan-less one there, and only
    /// round the mouth -- and fingerprints differently (a pyramid stored with
    /// the fans off never seeds a fan-on world, or the reverse; a different
    /// fan set differs too). With `river_fans` `None` the digest is the one
    /// the fields alone give, so a pyramid stored before RIM-7 still seeds a
    /// world that draws no fan. A worker that dropped the fans, or a digest
    /// blind to them, fails here.
    #[test]
    fn a_snapshot_with_delta_fans_draws_them_and_keys_them() {
        let (gw, gh) = (96usize, 72usize);
        let (plain_in, fans) = delta_inputs(gw, gh);
        let base = plain_in.fingerprint();
        let (mut with_in, _) = delta_inputs(gw, gh);
        with_in.river_fans = Some(Arc::new(fans));
        assert_ne!(with_in.fingerprint(), base, "the fans must move the digest");
        let (mut fewer_in, mut fewer) = delta_inputs(gw, gh);
        fewer.runs.pop();
        fewer_in.river_fans = Some(Arc::new(fewer));
        assert_ne!(fewer_in.fingerprint(), with_in.fingerprint(), "which fans must move the digest");
        let plain = LodSnapshot::build(plain_in).expect("snapshot");
        let with = LodSnapshot::build(with_in).expect("snapshot");
        let (a, w, h) = plain.render_tile(0, 0, 0).expect("tile");
        let (b, _, _) = with.render_tile(0, 0, 0).expect("tile");
        // Tile (0, 0, 0) spans the grid; pixel (px, py) sits on cell (px * (gw-1)/(w-1), ...).
        let (cx, cy) = ((gw - 1) as f64 / (w - 1) as f64, (gh - 1) as f64 / (h - 1) as f64);
        let mut changed = 0usize;
        for (k, (p, q)) in a.chunks(4).zip(b.chunks(4)).enumerate() {
            if p == q {
                continue;
            }
            changed += 1;
            let (x, y) = ((k % w) as f64 * cx, (k / w) as f64 * cy);
            // The fan reaches from its apex (x 39.5, a setback of 24 upstream of
            // the last dry point 63.5) to the shore, at most 1.6 setbacks out.
            assert!(x > 36.0 && x < 72.0 && (y - 36.0).abs() < 40.0, "a change far from the mouth at cell ({x:.1}, {y:.1})");
        }
        assert!(changed > w / 2, "the fans changed only {changed} tile pixels");
    }

    /// Protects: **the fans' separator word** in the stored-pyramid digest
    /// (RIM-7's disclosed survivor). No digest comparison can see it -- every
    /// field of the stream is length-prefixed, so no two inputs collide with
    /// or without it -- so the stream itself is read back (`Digest::log`):
    /// the fans section opens with `u64::MAX` and then the fans' run count,
    /// and a snapshot without fans has no `u64::MAX` word anywhere.
    #[test]
    fn the_fans_are_hashed_behind_a_separator() {
        let (mut i, fans) = delta_inputs(96, 72);
        assert!(!i.digest().log.contains(&u64::MAX), "premise: nothing else in the stream is u64::MAX");
        let n = fans.runs.len() as u64;
        i.river_fans = Some(Arc::new(fans));
        let log = i.digest().log;
        assert!(log.windows(2).any(|w| w == [u64::MAX, n]), "the fans section must open with u64::MAX and then its {n} runs");
    }

    /// Protects: **RIM-7's painted river in the worker path and the stored-
    /// pyramid key.** A snapshot handed a paint source
    /// (`SnapshotInputs::river_paint`, what `WorldGen::lod_snapshot_inputs`
    /// passes for a look that paints its rivers) paints the river into its
    /// tiles instead of stroking it: the tile differs from the stroked one,
    /// only near the river, it carries the floodplain tint beside the water
    /// (pixels changed while their alpha stays 255 -- not river) and still
    /// marks the water in the alpha. It fingerprints differently (a pyramid
    /// stored while the look stroked never seeds a painted world), by exactly
    /// one word -- [`PAINTED_RIVERS_TAG`] -- and a snapshot without paint has
    /// the digest the stroke always had.
    #[test]
    fn a_snapshot_that_paints_its_rivers_draws_and_keys_them() {
        let (gw, gh) = (96usize, 72usize);
        let (stroke_in, _) = delta_inputs(gw, gh);
        let base = stroke_in.fingerprint();
        assert!(!stroke_in.digest().log.contains(&PAINTED_RIVERS_TAG));
        let (mut paint_in, _) = delta_inputs(gw, gh);
        assert_eq!(paint_in.fingerprint(), base, "premise: the same inputs");
        let a = paint_in.appearance.clone();
        assert!(crate::river_field::painted(&a, gw, gh), "premise: the default look paints its rivers");
        let g = paint_in.rivers.clone().expect("a network");
        let colour = Arc::new(crate::river_stroke::rasterize_colour_field_with(&g, None, &a, gw, gh));
        paint_in.river_paint = Some(Arc::new(crate::river_field::PaintSource::new(&g, None, gw, gh, &a, colour).expect("a source")));
        assert_ne!(paint_in.fingerprint(), base, "painting must move the digest");
        let (sl, pl) = (stroke_in.digest().log, paint_in.digest().log);
        assert_eq!(pl.len(), sl.len() + 1, "painting adds exactly one word");
        assert_eq!(pl.iter().filter(|&&w| w == PAINTED_RIVERS_TAG).count(), 1);
        let s = LodSnapshot::build(stroke_in).expect("snapshot");
        let p = LodSnapshot::build(paint_in).expect("snapshot");
        let (a_px, w, h) = s.render_tile(0, 0, 0).expect("tile");
        let (b_px, w2, h2) = p.render_tile(0, 0, 0).expect("tile");
        assert_eq!((w, h), (w2, h2));
        // Tile (0, 0, 0) spans the grid; pixel (px, py) sits on cell (px * (gw-1)/(w-1), ...).
        let (cx, cy) = ((gw - 1) as f64 / (w - 1) as f64, (gh - 1) as f64 / (h - 1) as f64);
        let (mut changed, mut tint, mut water) = (0usize, 0usize, 0usize);
        for (k, (q, r)) in a_px.chunks(4).zip(b_px.chunks(4)).enumerate() {
            if r[3] < 255 {
                water += 1;
            }
            if q == r {
                continue;
            }
            changed += 1;
            let y = (k / w) as f64 * cy;
            let x = (k % w) as f64 * cx;
            // The river is at y = 36 (cell index): water, the tint band and the
            // bank are all within a few cells of it, and only west of the shore.
            assert!((y - 36.0).abs() < 6.0, "a change far from the river at cell ({x:.1}, {y:.1})");
            if r[3] == 255 && q[3] == 255 {
                tint += 1;
            }
        }
        assert!(changed > 0 && water > 0, "the painted tile must draw the river ({changed} changed, {water} water)");
        assert!(tint > 0, "the floodplain tint must reach the painted tile beside the water");
        // The bank line is drawn from this tile alone: with an outline the
        // tile marks more pixels as river in its alpha (the line's own),
        // and its colour changes beside the water.
        let (mut bank_in, _) = delta_inputs(gw, gh);
        bank_in.appearance.river_bank = 0.8;
        let ab = bank_in.appearance.clone();
        let colour = Arc::new(crate::river_stroke::rasterize_colour_field_with(&g, None, &ab, gw, gh));
        bank_in.river_paint = Some(Arc::new(crate::river_field::PaintSource::new(&g, None, gw, gh, &ab, colour).expect("a source")));
        let (c_px, _, _) = LodSnapshot::build(bank_in).expect("snapshot").render_tile(0, 0, 0).expect("tile");
        let marked = |v: &[u8]| v.chunks(4).filter(|p| p[3] < 255).count();
        assert!(marked(&c_px) > marked(&b_px), "the bank line must be marked as river: {} -> {}", marked(&b_px), marked(&c_px));
    }

    /// Protects Ruling BO in the worker path: a snapshot built with a forced
    /// lake draws it into its tiles. A land block on the test world's
    /// plateau is forced, and the tile over it must differ from the unforced
    /// tile there while every pixel away from the block is unchanged -- the
    /// mask reaches `TileFields` through `LodSnapshot::build` and nowhere else.
    /// Protects: Ruling BO in the worker path: a forced-lake mask changes the tile pixels over the forced block and leaves pixels far from it untouched.
    #[test]
    fn a_snapshot_with_forced_lakes_draws_them_in_its_tiles() {
        let (gw, gh) = (96usize, 72usize);
        let plain = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let mut i = inputs(gw, gh);
        // The test field peaks at the centre (0.30 + 0.55 * ...), so the
        // 5 x 5 around (48, 36) is well above sea level 0.42: land.
        let mut m = vec![0u8; gw * gh];
        for y in 34..=38 {
            for x in 46..=50 {
                assert!(i.field[y * gw + x] > 0.5, "premise: ({x}, {y}) is land");
                m[y * gw + x] = 1;
            }
        }
        i.forced_lakes = Some(m);
        let forced = LodSnapshot::build(i).expect("snapshot");
        let (a, w, h) = plain.render_tile(0, 0, 0).expect("tile");
        let (b, _, _) = forced.render_tile(0, 0, 0).expect("tile");
        // Tile (0, 0, 0) covers the whole grid: grid cell (x, y) is pixel
        // (x * w / gw, y * h / gh).
        let px = |x: f64, y: f64| ((y * h as f64 / gh as f64) as usize * w + (x * w as f64 / gw as f64) as usize) * 4;
        let c = px(48.5, 36.5);
        assert_ne!(a[c..c + 3], b[c..c + 3], "the forced block's centre must be drawn differently");
        let far = px(10.5, 10.5);
        assert_eq!(a[far..far + 4], b[far..far + 4], "far from the block nothing moves");
    }

    /// Protects: **GF-6 §4.10 in the worker path and the stored-pyramid
    /// key.** A snapshot without a rock detail fingerprints with no
    /// [`ROCK_DETAIL_TAG`] in its stream (so every app world keys exactly as
    /// before the field existed); one with it carries the tag, fingerprints
    /// differently (an unstyled stored pyramid never seeds a styled world),
    /// and a deep tile over land is drawn differently -- the detail reaches
    /// the tile through `render_tile`'s attachment. Under the reference
    /// schedule (`zoom_detail_km` 0) the same detail draws nothing: the
    /// styling is off by control flow there.
    #[test]
    fn a_snapshot_with_rock_detail_styles_and_keys_its_tiles() {
        let (gw, gh) = (96usize, 72usize);
        let plain_in = inputs(gw, gh);
        let base = plain_in.fingerprint();
        assert!(!plain_in.digest().log.contains(&ROCK_DETAIL_TAG), "no rock detail, no tag");
        let mut i = inputs(gw, gh);
        i.rock_detail = Some(Arc::new(render::RockDetail { cells: vec![cartalith_terrain::geology::Rock::Granite as u8; gw * gh], m_per_unit: 4000.0 / 0.58 }));
        assert!(i.digest().log.contains(&ROCK_DETAIL_TAG), "the rock detail section opens with its tag");
        assert_ne!(i.fingerprint(), base, "a styled snapshot must not seed from an unstyled pyramid");
        let plain = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let styled = LodSnapshot::build(i).expect("snapshot");
        // Level 5 over the land peak at the grid's centre: 32 x 32 tiles, so
        // tile (16, 16) is land at several pixels per cell.
        let (a, _, _) = plain.render_tile(5, 16, 16).expect("tile");
        let (b, _, _) = styled.render_tile(5, 16, 16).expect("tile");
        assert!(a.iter().any(|&v| v != 0), "an empty tile proves nothing");
        assert_ne!(a, b, "the rock detail must reach a deep land tile");
        let reference = |rd: bool| {
            let mut j = inputs(gw, gh);
            j.appearance = TerrainAppearance { zoom_detail_km: 0.0, ..TerrainAppearance::default() };
            if rd {
                j.rock_detail = Some(Arc::new(render::RockDetail { cells: vec![cartalith_terrain::geology::Rock::Granite as u8; gw * gh], m_per_unit: 4000.0 / 0.58 }));
            }
            LodSnapshot::build(j).expect("snapshot").render_tile(5, 16, 16).expect("tile").0
        };
        assert_eq!(reference(false), reference(true), "the reference schedule must ignore the rock detail");
    }

    /// A lava mark over the centre of the [`inputs`] world, as CP-2's binding
    /// would hand the snapshot (`WorldGen::colour_paint_for_snapshot`).
    fn centre_lava(i: &SnapshotInputs) -> Arc<cartalith_spatial::colour_paint::ColourPaint> {
        use cartalith_spatial::colour_paint::*;
        let kpc = i.map_width_km / i.gw as f64;
        let (cx, cy) = (i.gw as f64 * 0.5 * kpc, i.gh as f64 * 0.5 * kpc);
        let m = ColourMark {
            shape: MarkShape::Line { points_km: vec![[cx - 8.0 * kpc, cy], [cx + 8.0 * kpc, cy]], radius_km: 4.0 * kpc },
            rgb: [34, 33, 38],
            opacity: 1.0,
            hardness: 1.0,
            blend: ColourBlend::Replace,
            land_only: false,
            emissive: Some(EmissiveMaterial { ramp: GlowRamp::Fire, heat: 0.9, crack_density: 0.5, glow_radius_km: kpc, heat_along: 0.0 }),
        };
        Arc::new(ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks: vec![m] }, 2.0 * kpc).expect("layer"))
    }

    /// Protects CP-2's tile producer (`MISTAKES.md`: attaching a capability is
    /// not the consumer reading it): a snapshot with no colour paint carries
    /// no [`COLOUR_PAINT_TAG`] (every app world keys as before CP-2); one with
    /// a layer carries the tag, fingerprints differently (a stored unpainted
    /// pyramid never seeds a painted world), and its centre tile is drawn
    /// differently -- the layer reaches the pixels through `render_tile`.
    /// Two different documents also fingerprint differently.
    #[test]
    fn a_snapshot_with_colour_paint_paints_and_keys_its_tiles() {
        let (gw, gh) = (96usize, 72usize);
        let plain_in = inputs(gw, gh);
        let base = plain_in.fingerprint();
        assert!(!plain_in.digest().log.contains(&COLOUR_PAINT_TAG), "no paint, no tag");
        let mut i = inputs(gw, gh);
        i.colour_paint = Some(centre_lava(&i));
        assert!(i.digest().log.contains(&COLOUR_PAINT_TAG), "the paint section opens with its tag");
        let painted_fp = i.fingerprint();
        assert_ne!(painted_fp, base, "a painted snapshot must not seed from an unpainted pyramid");
        let mut other = inputs(gw, gh);
        let mut doc = centre_lava(&other).doc().clone();
        doc.marks[0].rgb = [200, 30, 18];
        other.colour_paint = Some(Arc::new(cartalith_spatial::colour_paint::ColourPaint::new(doc, 1.0).unwrap()));
        assert_ne!(other.fingerprint(), painted_fp, "another document, another key");
        // Ruling CS's paint layers: the fingerprint hashes the marks that
        // composite, so a second, hidden layer keys like no second layer, and
        // showing it moves the key.
        let mut layered = centre_lava(&other).layers().clone();
        layered.add_layer(Some("Second")).unwrap();
        let extra = layered.layers[0].marks[0].clone();
        layered.add_mark(1, extra).unwrap();
        layered.set_layer_visible(1, false).unwrap();
        let mut hid = inputs(gw, gh);
        hid.colour_paint = Some(Arc::new(cartalith_spatial::colour_paint::ColourPaint::from_layers(layered.clone(), 1.0).unwrap()));
        assert_eq!(hid.fingerprint(), painted_fp, "a hidden layer does not reach the tiles, so not the key");
        layered.set_layer_visible(1, true).unwrap();
        hid.colour_paint = Some(Arc::new(cartalith_spatial::colour_paint::ColourPaint::from_layers(layered, 1.0).unwrap()));
        assert_ne!(hid.fingerprint(), painted_fp, "shown, it is another key");
        let plain = LodSnapshot::build(inputs(gw, gh)).expect("snapshot");
        let painted = LodSnapshot::build(i).expect("snapshot");
        let (a, _, _) = plain.render_tile(2, 2, 2).expect("tile");
        let (b, _, _) = painted.render_tile(2, 2, 2).expect("tile");
        assert!(a.iter().any(|&v| v != 0), "an empty tile proves nothing");
        let moved = a.iter().zip(&b).filter(|(x, y)| x != y).count();
        assert!(moved > 100, "the colour paint must reach the centre tile ({moved} bytes moved)");
    }
}
