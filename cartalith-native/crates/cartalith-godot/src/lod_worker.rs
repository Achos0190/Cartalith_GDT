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
use std::sync::{Arc, Mutex, OnceLock};

use cartalith_spatial::pyramid::ChunkId;

use crate::lod_bridge;
use crate::render::{
    self, ColorSpace, GridPrecompute, GroundTile, GroundTiles, QualityTier, RenderCtx, RiverInk, SplatChannel, SplatTextures, TerrainAppearance, TileCryo, TileFields,
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
        QualityTier::Performance => LodBudget { tiles_per_update: 16, tiles_per_catchup: 3, max_level: lod_bridge::MAX_LEVEL - 1, cache_tiles: 32, max_in_flight: 2 },
        QualityTier::Balanced => LodBudget { tiles_per_update: 32, tiles_per_catchup: 4, max_level: lod_bridge::MAX_LEVEL, cache_tiles: 32, max_in_flight: 3 },
        QualityTier::Quality => LodBudget { tiles_per_update: 48, tiles_per_catchup: 6, max_level: lod_bridge::MAX_LEVEL, cache_tiles: 35, max_in_flight: 4 },
        QualityTier::Ultra => LodBudget { tiles_per_update: 48, tiles_per_catchup: 8, max_level: lod_bridge::MAX_LEVEL, cache_tiles: 35, max_in_flight: 6 },
    }
}

// ---------------------------------------------------------------------------
// The snapshot — everything a tile needs, owned, `Send + Sync`
// ---------------------------------------------------------------------------

/// The world's river ink, owned rather than borrowed.
///
/// `render::RiverInk` is two borrowed slices by design (its own doc records
/// what happened the week the bake path and the screen path held separate
/// copies). A background thread cannot borrow from `WorldGen`, so the
/// snapshot owns the bytes and hands out a `RiverInk` borrowing *itself*.
pub enum OwnedInk {
    Stamped(Vec<f32>),
    Flag(Vec<u8>),
}

impl OwnedInk {
    pub(crate) fn as_ink(&self) -> RiverInk<'_> {
        match self {
            OwnedInk::Stamped(v) => RiverInk::Stamped(v),
            OwnedInk::Flag(v) => RiverInk::Flag(v),
        }
    }
}

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
/// What is still copied: the ink (10.74 MB stamped, 2.68 MB as a flag), the
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
}

/// A word-at-a-time FNV-1a-style digest, for [`SnapshotInputs::fingerprint`].
///
/// Word-wise rather than the byte-wise FNV-1a `lod_source_key` uses because
/// it runs over every grid a tile reads — ~100 MB at 2048x1311 with a
/// lithology — and eight times fewer multiplies is the difference between
/// noise and a visible slice of the ~480 ms snapshot build. It guards against
/// a *changed* input, not a forged one: each step (`xor` a word, multiply by
/// an odd constant) is a bijection on the state, so any change confined to
/// one word always changes the result.
struct Digest(u64);

impl Digest {
    fn new() -> Self {
        Digest(0xcbf2_9ce4_8422_2325)
    }
    fn word(&mut self, w: u64) {
        self.0 = (self.0 ^ w).wrapping_mul(0x0000_0100_0000_01b3);
    }
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
        // Exhaustive, so a third colour space is a compile error here rather
        // than a silent alias of one of these two.
        d.word(match color_space {
            ColorSpace::Srgb => 0,
            ColorSpace::DisplayP3 => 1,
        });
        match ink {
            None => d.word(0),
            Some(OwnedInk::Stamped(v)) => {
                d.word(1);
                d.f32s(v);
            }
            Some(OwnedInk::Flag(v)) => {
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
        d.0
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
    field: Arc<Vec<f32>>,
    temperature: Arc<Vec<f32>>,
    rainfall: Arc<Vec<f32>>,
    flow: Option<Arc<Vec<f32>>>,
    appearance: TerrainAppearance,
    color_space: ColorSpace,
    pre: GridPrecompute,
    lithology: Option<Vec<u8>>,
    fields: TileFields<'static>,
    ink: Option<OwnedInk>,
    splat: Option<OwnedSplat>,
    ground_biomes: Vec<Option<GroundTile>>,
    ground_terrains: Vec<Option<GroundTile>>,
    paint_present: bool,
    paint_biome: Option<Vec<u8>>,
    paint_terrain: Option<Vec<u8>>,
    paint_splat: Option<Vec<u8>>,
}

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
        } = i;
        if gw < 2 || gh < 2 || field.len() < gw.checked_mul(gh)? {
            return None;
        }
        let pre = GridPrecompute::build(&field, &temperature, &rainfall, flow.as_ref().map(|v| v.as_slice()), gw, gh, sea_level, world, &appearance, Some(map_width_km));
        // The same call `build_color_texture` makes, and `None` under the same
        // condition: a loaded save's format stores none of the tectonic
        // substrate this needs (`SAVEFILE_COMPAT.md`), which is why its
        // caller hands us no `LithoSource` at all.
        let lithology = litho.as_ref().map(|l| cartalith_civ::build_lithology(&field, &l.age, &l.volcanic, &l.crust, &l.resistance, &rainfall, sea_level));
        // A throwaway context, only so `TileFields::new` has the `ctx` its
        // signature takes. It borrows everything above, which is why the
        // struct is assembled after this block and not before it.
        let fields = {
            let mut ctx = RenderCtx::from_precomputed(&field, &temperature, &rainfall, flow.as_ref().map(|v| v.as_slice()), gw, gh, sea_level, world, lat_n, lat_s, appearance.clone(), &pre)?;
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
            let glacier = render::build_glacier_potential(&field, &temperature, flow.as_ref().map(|v| v.as_slice()), gw, gh, sea_level, glacial_snowline, map_width_km / gw.max(1) as f64, world);
            let cryo = TileCryo {
                lapse_rate,
                g: gravity,
                meters_per_unit: if (1.0 - sea_level).abs() > 1e-9 { peak_m / (1.0 - sea_level) } else { peak_m / 1e-6 },
            };
            TileFields::new(&ctx, grid_rgb.as_deref()).with_cryo(glacier, cryo)
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
            temperature,
            rainfall,
            flow,
            appearance,
            color_space,
            pre,
            lithology,
            fields,
            ink,
            splat,
            ground_biomes,
            ground_terrains,
            paint_present,
            paint_biome,
            paint_terrain,
            paint_splat,
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
        let mut ctx = RenderCtx::from_precomputed(&self.field, &self.temperature, &self.rainfall, self.flow.as_ref().map(|v| v.as_slice()), self.gw, self.gh, self.sea_level, self.world, self.lat_n, self.lat_s, self.appearance.clone(), &self.pre)?;
        if let Some(l) = self.lithology.as_ref() {
            ctx = ctx.with_lithology(l);
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
            tf = tf.with_ink(ink.as_ink());
        }
        tf = tf.with_color_space(self.color_space);
        lod_bridge::synthesize_tile_rgba(&ctx, &tf, z, col, row, self.seed)
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
    /// were never counted, for the same reason. A pack's textures are counted
    /// because that clone is still real. Reported rather than estimated from
    /// a formula, so the shell can show the real number.
    ///
    /// **A shared `Arc` is not free in every case** — see
    /// [`cartalith_engine::WorldState::field`]: a sculpt or erode landing
    /// while a snapshot is alive copies the one grid it touches, once. That
    /// copy belongs to the world, not to this snapshot, and is not counted
    /// here either.
    pub fn retained_bytes(&self) -> usize {
        let f = std::mem::size_of::<f32>();
        let mut n = 0usize;
        n += match self.ink.as_ref() {
            Some(OwnedInk::Stamped(v)) => v.len() * f,
            Some(OwnedInk::Flag(v)) => v.len(),
            None => 0,
        };
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
fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
        rayon::ThreadPoolBuilder::new()
            .num_threads(cores.saturating_sub(1).max(1))
            .thread_name(|i| format!("cartalith-lod-{i}"))
            // A panic inside a tile job must not abort the process, which is
            // rayon's default for an unhandled one. There is no `Gd` and no
            // Godot API in a job, so there is nothing to report *to* from
            // here; the job simply produces no tile and the shell keeps its
            // fallback. `eprintln!` rather than `godot_error!` because the
            // latter is not documented as callable off the main thread.
            .panic_handler(|_| eprintln!("cartalith: a LOD tile job panicked; the tile was dropped"))
            .build()
            .ok()
    })
    .as_ref()
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

#[derive(Default)]
struct WorkerState {
    /// The key of `snapshot`, or of the build in flight.
    key: String,
    snapshot: Option<Arc<LodSnapshot>>,
    building: bool,
    in_flight: HashSet<(i32, i32, i32)>,
    ready: Vec<ReadyTile>,
    /// Counters for the shell's diagnostics, never for a decision.
    built: u64,
    dropped: u64,
}

/// The main thread's handle on the background synthesis.
///
/// Held by `WorldGen` as an `Arc<LodWorker>`; the `Arc` is what a job captures.
/// Nothing inside is Godot-aware, which is what makes crossing a thread with it
/// legal at all.
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
    fn from(l: cartalith_io::project::LodTiles) -> Self {
        StoredPyramid { producer: l.producer, tile_w: l.tile_w, tile_h: l.tile_h, tiles: l.tiles }
    }
}

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
        let Some(pool) = pool() else { return false };
        let (snap, generation) = {
            let Ok(mut st) = self.state.lock() else { return false };
            let Some(snap) = st.snapshot.clone() else { return false };
            let id = (z, col, row);
            if st.in_flight.contains(&id) || st.ready.iter().any(|t| (t.z, t.col, t.row) == id) {
                return false;
            }
            // A seeded tile needs no job: it is already coloured. Handed
            // straight to `ready`, ahead of the in-flight cap, which exists
            // to bound work and there is none.
            if let Some((rgba, w, h)) = self.seeded_tile(&snap, z, col, row) {
                self.seeded_served.fetch_add(1, Ordering::SeqCst);
                st.ready.push(ReadyTile { z, col, row, w, h, rgba });
                return true;
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
            let out = snap.render_tile(z, col, row);
            let Ok(mut st) = me.state.lock() else { return };
            st.in_flight.remove(&(z, col, row));
            if me.generation.load(Ordering::SeqCst) != generation {
                st.dropped += 1;
                return;
            }
            match out {
                Some((rgba, w, h)) => {
                    st.ready.push(ReadyTile { z, col, row, w, h, rgba });
                    st.built += 1;
                }
                None => st.dropped += 1,
            }
        });
        true
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
        if let Some(t) = self.seeded_tile(snap, z, col, row) {
            self.seeded_served.fetch_add(1, Ordering::SeqCst);
            return Some(t);
        }
        self.synthesized.fetch_add(1, Ordering::SeqCst);
        snap.render_tile(z, col, row)
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
        }
    }
}

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
            }
    }

    /// **The milestone's parity class, asserted.** The structural argument is
    /// that both paths call `render_tile`; this is the argument being
    /// exercised rather than written down, over enough tiles that a
    /// thread-count-dependent seam would have to show.
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

    /// A result that lands after the world has moved on must be discarded,
    /// not installed. Driven through `invalidate`, which is what a world
    /// release does.
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
        const SNAPSHOT_2048X1311: usize = 53_698_560;
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
        no_flow.ink = Some(OwnedInk::Flag(vec![0u8; n]));
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
        for (held, name) in [(&age, "age"), (&volcanic, "volcanic"), (&crust, "crust"), (&resistance, "resistance")] {
            assert_eq!(Arc::strong_count(held), 1, "{name}: the build must drop the LithoSource, leaving only this test holding it");
        }
        assert_eq!(snap.retained_bytes(), 0, "a lithology is built, not copied, so the substrate costs no retained bytes");
    }

    /// A degenerate grid must return `None` rather than panic — this runs on
    /// a worker thread, where a panic has no `#[func]` to return through.
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

    fn save_params(gw: usize, gh: usize) -> cartalith_io::SaveParams {
        cartalith_io::SaveParams { gw, gh, seed: 1234, map_width_km: 800.0, sea_level: 0.42, world: false, origin: None, name: None }
    }

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
            ("river ink", { let mut i = inputs(gw, gh); i.ink = Some(OwnedInk::Flag(vec![1u8; gw * gh])); i }),
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
    }
}
