//! The bake/atlas/finalize system's shell-side state — `GUI_GAP_REGISTER.md`
//! WW-01 (*Finalize · LOD 0-3 · bake & freeze*), PR-10/S4 (*Tiled LOD · tile
//! size · atlas cache*), S5 and SH-07's `atlas` status slot.
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs` and `lod_bridge.rs` argue for: `lib.rs` owns the thin
//! `Variant`↔Rust conversion and the `#[func]` surface; this module owns where
//! the atlas lives, what the world's key is, and whether the world is locked.
//! Its `#[cfg(test)]` suite runs under plain `cargo test -p cartalith-godot`
//! with no Godot runtime.
//!
//! # The one judgement call this module makes: what goes into the world key
//!
//! [`cartalith_io::world_key`] is a hash of "every parameter that changes what
//! a baked tile would contain, and nothing that only changes how it is drawn".
//! The reference names its thirteen fields by hand (`worldKey`, reference line
//! 10703: `GW, GH, world, seaLevel, peakM, tect, world_structure, volc,
//! crater, planet, climate, erosion, stream`). This port's parameter set is not
//! field-for-field the same (`GENERATION_PARAMETERS.md`), so the list is
//! rebuilt rather than transcribed:
//!
//! **In**: the grid dimensions, the seed, the map width in km, sea level, the
//! east-west wrap flag, **how the height field was produced**
//! ([`ORIGIN_GENERATED`] and friends — see those constants), and every row of
//! `params::world_key_state` — the same
//! table `SAVEFILE_COMPAT.md`'s writer persists, minus the one group that is
//! not a terrain input. That sentence read "**every** row of
//! `params::save_state` … which is by construction every value
//! `generate_terrain` reads" until the civ `PARAMS` group landed
//! (`LARGE_ITEM_RULINGS.md`, 2026-08-31) and made the second half false: those
//! seven rows are read by `compute_civilisation` alone. Hashing them would
//! have invalidated a baked terrain atlas the moment somebody moved the
//! village toggle, on an atlas that is byte-identical either way. The
//! exclusion lives in `params::world_key_state`, derived from the group name
//! so a row added later is covered without editing this list.
//!
//! **Out**: `TerrainAppearance` and its override map, the elevation ramp, the
//! NPR block, the quality tier, layer visibility, labels, icons, the camera —
//! everything `render.rs` owns. That is exactly the reference's own
//! `#genV3dSec` exemption, and it has to be: a control the finalize lock lets
//! the user change must not invalidate the atlas they just paid for, and those
//! two rules are the same rule seen from either side.
//!
//! One row is in that arguably should not be: `river_density`, which the
//! reference keeps under `state.viz` (excluded there). Here it feeds
//! `PipelineStage::Climate` (`params::invalidates`), so it really can move a
//! baked tile's inputs. Included, because over-invalidating costs a re-bake and
//! under-invalidating serves the wrong terrain.
//!
//! # Where the atlas lives
//!
//! Nowhere, until the shell says. `AtlasStore` takes a real filesystem path and
//! Godot's `user://` is not one, so the shell resolves it
//! (`ProjectSettings.globalize_path`) and hands it over. Before that
//! [`BakeState::store`] is `None` and every operation reports "no atlas root"
//! rather than silently writing into the working directory — the same
//! fail-visibly stance `AtlasStore`'s own `io::Result` returns take.

use std::path::PathBuf;

use cartalith_engine::bake::{FinalizeLock, Mutation};
use cartalith_io::AtlasStore;

/// The reference's `_lodTile` default (reference line 10656).
pub const DEFAULT_TILE_SIZE: usize = 1024;

/// The reference's own `bakeAllDepth` ceiling, and a real one: depth 6 is
/// 5461 tiles. Values above this are clamped rather than refused, so a
/// mis-set control degrades to the deepest sane bake instead of doing nothing.
pub const MAX_BAKE_DEPTH: i32 = 6;

/// Everything the shell needs to remember about the atlas between calls.
#[derive(Debug, Clone, Default)]
pub struct BakeState {
    /// A real OS directory, resolved by the shell from `user://`. `None` until
    /// `atlas_set_root` is called.
    pub root: Option<PathBuf>,
    /// `state.finalized` (reference line 10872).
    pub finalized: bool,
    /// `_lodTile`. Part of the chunk key, not of the world hash — two tile
    /// sizes over one world are two valid bakes.
    pub tile_size: usize,
}

impl BakeState {
    pub fn new() -> Self {
        BakeState { root: None, finalized: false, tile_size: DEFAULT_TILE_SIZE }
    }

    pub fn store(&self) -> Option<AtlasStore> {
        self.root.as_ref().map(AtlasStore::new)
    }

    pub fn lock(&self) -> FinalizeLock {
        FinalizeLock { finalized: self.finalized }
    }

    /// `Ok(())` or the message to show. See [`FinalizeLock::check`].
    pub fn check(&self, m: Mutation) -> Result<(), &'static str> {
        self.lock().check(m)
    }
}

/// A world whose height field `generate_terrain` produced from the parameter
/// tuple in this very signature — [`WorldGen::generate_sized`] and
/// [`WorldGen::generate_world_structure_sized`].
///
/// **Also what a loaded save reports when its archive did not record an
/// origin** — and only then. `cartalith_io::SaveParams::origin` carries
/// provenance through the format (`project.json`'s `world.origin` in the tree
/// layout, `params.json`'s top-level `origin` in the flat one) and both of
/// this port's writers fill it in, so a saved import or resample now reopens
/// as itself. What cannot be recovered is an archive written before that
/// member existed, including every genuine `Cartalith Gen1` export; those
/// restore every *other* element of this signature exactly, so
/// [`origin_for_key`] substitutes this value rather than giving them a
/// namespace of their own and orphaning an atlas already baked. That
/// substitution, and its cost, live there.
///
/// [`WorldGen::generate_sized`]: crate::WorldGen
/// [`WorldGen::generate_world_structure_sized`]: crate::WorldGen
pub const ORIGIN_GENERATED: &str = "gen";

/// A world whose height field came off disk as an image —
/// `WorldGen::import_heightmap`. Its substrate is *inverted* from that image
/// rather than generated, so the parameter tuple does not determine the field
/// and cannot be allowed to name the same atlas namespace a generated world
/// would.
pub const ORIGIN_IMPORTED: &str = "import";

/// A world resampled out of another world's Region-select marquee —
/// `WorldGen::region_new_world`. Its field is an amplified crop of the
/// *parent's*, and it inherits the parent's `seed`, so its parameter tuple is
/// especially likely to land on one a generated world already owns.
pub const ORIGIN_REGION: &str = "region";

/// The origin an atlas key uses for a world whose archive **did not record
/// one** — every project saved before `world.origin` existed
/// (`SAVEFILE_COMPAT.md` §7).
///
/// [`ORIGIN_GENERATED`], and that substitution is a cost paid on purpose
/// rather than a claim about the world. Such an archive restores every
/// *other* element of [`world_key_signature`] exactly, so giving the unknown
/// case a string of its own would change the key of every pre-provenance
/// project the first time it is reopened and orphan the atlas its owner
/// already paid to bake — including on the generate → bake → save → open
/// path, which is the common one.
///
/// It is a function, and the only one, so that the substitution has exactly
/// one home and a test can pin it: `WorldGen::world_key` calls it and nothing
/// else resolves a `None` origin. In particular `WorldGen::world_origin`
/// itself keeps the `None`, so re-saving a pre-provenance archive writes no
/// origin rather than starting to claim one.
pub fn origin_for_key(origin: Option<&str>) -> &str {
    origin.unwrap_or(ORIGIN_GENERATED)
}

/// The world-key signature — see this module's own header for what is in it
/// and why.
///
/// A JSON array rather than an object so the field *order* is part of the hash
/// and a reordering cannot silently produce the same key from different
/// parameters. The trailing element is `params::save_state`'s whole object,
/// whose own key order is `PARAMS`' static table order — deterministic across
/// runs and platforms, which is the property the hash needs.
///
/// `origin` is one of [`ORIGIN_GENERATED`]/[`ORIGIN_IMPORTED`]/
/// [`ORIGIN_REGION`], and it is in the signature because **nothing else in it
/// says how the field was produced**. Every other element is a generation
/// *input*, and an imported heightmap and a region resample both arrive at a
/// field the inputs did not determine: without this element a 1024×512 import
/// at seed 42 and a 1024×512 generate at seed 42 share one atlas namespace and
/// serve each other's baked tiles.
///
/// Adding it changed every key this port had ever computed, which is safe and
/// was checked rather than assumed: a key with no chunks under it makes
/// `AtlasStore::keys_for_world` return an empty set, so `atlas_status` reads
/// *"Atlas: empty (this world)"*, `bake_estimate` reports `already_baked: 0`
/// and `run_bake` bakes the pyramid again. The chunks under the old key are
/// orphaned on disk, not corrupted — the same outcome as moving any other
/// generation dial, which is the mechanism this hash exists to drive.
pub fn world_key_signature(
    gw: i32,
    gh: i32,
    seed: i32,
    map_width_km: f64,
    sea_level: f64,
    world: bool,
    origin: &str,
    params_state: serde_json::Value,
) -> String {
    serde_json::json!([gw, gh, seed, map_width_km, sea_level, world, origin, params_state])
        .to_string()
}

/// What the status readout shows (`GUI_GAP_REGISTER.md` SH-07's `atlas` slot,
/// the reference's `updateAtlasStatus` at line 10748 plus the two numbers it
/// does not report).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AtlasStatus {
    pub chunks: usize,
    pub bytes: u64,
    pub deepest_level: i32,
    /// The reference's own wording, adapted: it distinguishes "no IndexedDB"
    /// from "empty (this world)", and so does this — an unconfigured root is
    /// the same *kind* of fact as a browser with no IDB, and reporting it as
    /// "empty" would tell the user their bake vanished.
    pub text: String,
}

/// `updateAtlasStatus()` (reference line 10748).
pub fn atlas_status(state: &BakeState, world_key: &str) -> AtlasStatus {
    let Some(store) = state.store() else {
        return AtlasStatus { text: "Atlas: — (no cache directory set)".into(), ..Default::default() };
    };
    let keys = store.keys_for_world(world_key).unwrap_or_default();
    let n = keys.len();
    let deepest = keys.iter().map(|k| k.id.z as i32).max().unwrap_or(-1);
    let bytes = store.world_bytes(world_key).unwrap_or(0);
    let text = if n == 0 {
        "Atlas: empty (this world)".to_string()
    } else {
        format!(
            "Atlas: {n} chunk{} baked to LOD {deepest} ({})",
            if n == 1 { "" } else { "s" },
            human_bytes(bytes)
        )
    };
    AtlasStatus { chunks: n, bytes, deepest_level: deepest, text }
}

/// Byte count for a status line — binary units, one decimal above KiB, which
/// is what `dcc_shell.gd`'s other size readouts already use.
pub fn human_bytes(b: u64) -> String {
    const K: f64 = 1024.0;
    let b = b as f64;
    if b < K {
        return format!("{} B", b as u64);
    }
    for (i, unit) in ["KiB", "MiB", "GiB", "TiB"].iter().enumerate() {
        let scale = K.powi(i as i32 + 1);
        if b < scale * K || i == 3 {
            return format!("{:.1} {unit}", b / scale);
        }
    }
    unreachable!("the TiB arm returns")
}

/// What a bake would cost, shown *before* the user commits.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BakeEstimate {
    pub tiles: u64,
    /// Bytes the packed heights alone will occupy. **Exact**, not a guess —
    /// see [`bake_estimate`].
    pub height_bytes: u64,
    /// `height_bytes` plus a per-chunk PNG allowance.
    pub total_bytes: u64,
    pub seconds: f64,
    /// The per-tile pixel size, which is the same at every level (see
    /// [`bake_estimate`]) and is what a user actually chose when they picked
    /// a tile size.
    pub tile_w: usize,
    pub tile_h: usize,
}

/// The cost of baking `0..=max_z` at `tile_size` over a `gw × gh` world.
///
/// **The byte figure is exact for the height half, and that is worth stating
/// because the number is much larger than it looks.** Every tile at every
/// level has the *same* pixel dimensions: `tile_dims` picks them from the
/// region's aspect ratio, and a level-`z` tile's coarse footprint is
/// `(gw-1)/2^z × (gh-1)/2^z`, whose aspect is `(gw-1)/(gh-1)` regardless of
/// `z`. So the pyramid's height storage is simply `tiles × tw × th × 4`.
///
/// Measured against a real bake to check the arithmetic rather than trusting
/// it: 2048×1311 at 1024 px, depth 3 — 85 chunks, predicted 217.6 MiB of
/// `rg16`, actual on-disk total 233.7 MiB including the PNGs. Depth 5 at the
/// same settings would be 1365 chunks and roughly **3.7 GiB**, which is
/// exactly the kind of thing a user must be told before they click and not
/// after.
///
/// The PNG allowance is [`PNG_SHARE`] of the height bytes, from that same
/// measurement (263 KiB of PNG against 2.56 MiB of `rg16` per chunk). It is a
/// ratio rather than a formula because a PNG's size depends on the terrain in
/// it; a flat ocean tile compresses to almost nothing and a mountain range
/// does not.
///
/// `seconds` is a straight per-tile rate — deliberately crude, because the
/// honest thing to show is an order of magnitude and a precise-looking wrong
/// number is worse than an obviously-rough right one.
pub fn bake_estimate(
    max_z: i32,
    gw: usize,
    gh: usize,
    tile_size: usize,
    per_tile_ms: f64,
) -> BakeEstimate {
    let tiles = cartalith_spatial::pyramid::pyramid_tile_count(max_z.clamp(0, MAX_BAKE_DEPTH));
    if gw < 2 || gh < 2 || tile_size == 0 {
        return BakeEstimate { tiles, seconds: tiles as f64 * per_tile_ms / 1000.0, ..Default::default() };
    }
    let td = cartalith_spatial::tile_dims(
        &cartalith_spatial::Region { x: 0, y: 0, w: gw - 1, h: gh - 1 },
        1,
        1,
        tile_size,
    );
    let height_bytes = tiles * (td.w as u64) * (td.h as u64) * 4;
    BakeEstimate {
        tiles,
        height_bytes,
        total_bytes: height_bytes + (height_bytes as f64 * PNG_SHARE) as u64,
        seconds: tiles as f64 * per_tile_ms / 1000.0,
        tile_w: td.w,
        tile_h: td.h,
    }
}

/// See [`bake_estimate`]: PNG bytes as a fraction of `rg16` bytes, measured
/// on a real 2048×1311 bake rather than assumed.
pub const PNG_SHARE: f64 = 0.10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_root_is_reported_as_such_and_never_as_empty() {
        // The distinction that matters: "your bake is gone" and "there is
        // nowhere to bake to" must not read the same.
        let s = BakeState::new();
        let st = atlas_status(&s, "abc");
        assert_eq!(st.chunks, 0);
        assert!(st.text.contains("no cache directory"), "{}", st.text);
        assert!(!st.text.contains("empty"));
    }

    #[test]
    fn an_empty_configured_atlas_reports_empty() {
        let root = std::env::temp_dir()
            .join(format!("cartalith-bakebridge-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let s = BakeState { root: Some(root.clone()), ..BakeState::new() };
        assert_eq!(atlas_status(&s, "abc").text, "Atlas: empty (this world)");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_signature_changes_with_every_generation_input() {
        let base = |sea: f64| {
            world_key_signature(
                512, 512, 7, 800.0, sea, false, ORIGIN_GENERATED, serde_json::json!({"a": 1}),
            )
        };
        assert_eq!(base(0.42), base(0.42));
        assert_ne!(base(0.42), base(0.43));
        let g = ORIGIN_GENERATED;
        let a =
            world_key_signature(512, 512, 7, 800.0, 0.42, false, g, serde_json::json!({"a": 1}));
        let b =
            world_key_signature(512, 512, 8, 800.0, 0.42, false, g, serde_json::json!({"a": 1}));
        let c = world_key_signature(512, 512, 7, 800.0, 0.42, true, g, serde_json::json!({"a": 1}));
        let d =
            world_key_signature(512, 512, 7, 800.0, 0.42, false, g, serde_json::json!({"a": 2}));
        assert_ne!(a, b, "seed");
        assert_ne!(a, c, "wrap flag");
        assert_ne!(a, d, "a parameter row");
    }

    #[test]
    fn the_signature_order_is_part_of_the_hash() {
        // Two worlds whose gw and gh are swapped must not collide -- an
        // object-keyed signature with a non-deterministic order could.
        let a = world_key_signature(
            512, 256, 7, 800.0, 0.42, false, ORIGIN_GENERATED, serde_json::Value::Null,
        );
        let b = world_key_signature(
            256, 512, 7, 800.0, 0.42, false, ORIGIN_GENERATED, serde_json::Value::Null,
        );
        assert_ne!(cartalith_io::world_key(&a), cartalith_io::world_key(&b));
    }

    /// The defect this element exists for: three ways of arriving at a world
    /// could share one atlas namespace at the same parameter tuple, and one
    /// would then read another's baked tiles.
    ///
    /// Asserted on the **hashed** key rather than the signature string, since
    /// that is what `AtlasStore` files chunks under -- a signature that differs
    /// only in a field the hash never sees would pass the cheaper assertion.
    #[test]
    fn the_same_parameters_from_three_different_origins_are_three_atlases() {
        let key = |origin: &str| {
            cartalith_io::world_key(&world_key_signature(
                1024,
                512,
                42,
                800.0,
                0.42,
                false,
                origin,
                serde_json::json!({"cartalith": {"tect.plates": 9}}),
            ))
        };
        let generated = key(ORIGIN_GENERATED);
        let import = key(ORIGIN_IMPORTED);
        let region = key(ORIGIN_REGION);
        assert_ne!(
            generated, import,
            "an imported heightmap must not read a generated world's tiles"
        );
        assert_ne!(generated, region, "a region resample must not read a generated world's tiles");
        assert_ne!(import, region, "an import and a resample are two different worlds");
        // The three constants must also stay distinct *strings*: two of them
        // spelled the same would make the three assertions above pass on a
        // one-line typo, since the hash of equal inputs is equal.
        assert_ne!(ORIGIN_GENERATED, ORIGIN_IMPORTED);
        assert_ne!(ORIGIN_GENERATED, ORIGIN_REGION);
        assert_ne!(ORIGIN_IMPORTED, ORIGIN_REGION);
    }

    /// The other half of the same rule: the origin must be the **only** thing
    /// that changed, so this pins that adding it did not disturb the six
    /// elements already in the array or the object at the tail.
    #[test]
    fn origin_sits_between_the_wrap_flag_and_the_parameter_object() {
        let s = world_key_signature(
            1024, 512, 42, 800.0, 0.42, true, ORIGIN_IMPORTED, serde_json::json!({"a": 1}),
        );
        assert_eq!(s, r#"[1024,512,42,800.0,0.42,true,"import",{"a":1}]"#);
    }

    /// A `.zip` written before `world.origin` existed **must keep the atlas
    /// namespace it already had**, and this pins that end to end: a
    /// pre-provenance project archive is built, read back through
    /// `cartalith_io`, and its key computed the way `WorldGen::world_key`
    /// computes one.
    ///
    /// `117cb87a` is a literal measured on this signature at `686cd2a`,
    /// before `SaveParams::origin` existed — the state of a user's disk. It
    /// is not `world_key(...)` restated, so changing `ORIGIN_GENERATED`,
    /// changing what `origin_for_key` substitutes, or writing an `origin`
    /// member into an archive that had none all turn it red.
    #[test]
    fn a_save_written_before_origin_existed_reopens_into_the_same_atlas() {
        let (gw, gh) = (64usize, 32usize);
        let n = gw * gh;
        let params = cartalith_io::SaveParams {
            gw,
            gh,
            seed: 4242,
            map_width_km: 1234.5,
            sea_level: 0.37,
            world: true,
            // The whole point: what an archive on a user's disk says.
            origin: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: vec![0.5; n],
            temperature: vec![10.0; n],
            rainfall: vec![0.25; n],
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0u8; n],
        };
        let mut buf = Vec::new();
        cartalith_io::project::write_project(
            std::io::Cursor::new(&mut buf),
            &cartalith_io::ProjectWrite::new(&params, &fields),
        )
        .expect("write_project");

        // No `origin` member reached the archive, which is what makes the
        // bytes a genuine stand-in for a pre-provenance save rather than a
        // new one that happens to read back the same.
        let manifest: serde_json::Value = {
            let mut a = zip::ZipArchive::new(std::io::Cursor::new(&buf)).expect("open");
            let mut e = a.by_name(cartalith_io::PROJECT_MANIFEST).expect("project.json");
            serde_json::from_reader(&mut e).expect("json")
        };
        assert!(
            manifest["world"].get("origin").is_none(),
            "a None origin must be an absent key, not a written one: {manifest}"
        );

        let read = cartalith_io::load_save(std::io::Cursor::new(&buf)).expect("load_save");
        assert_eq!(read.params.origin, None, "absent must read back as absent");

        let key = cartalith_io::world_key(&world_key_signature(
            read.params.gw as i32,
            read.params.gh as i32,
            read.params.seed,
            read.params.map_width_km,
            read.params.sea_level,
            read.params.world,
            origin_for_key(read.params.origin.as_deref()),
            serde_json::json!({"legacy": true}),
        ));
        assert_eq!(key, "117cb87a", "a pre-provenance save changed atlas namespace");
    }

    /// The other direction, and the defect the format change exists to close:
    /// a saved import must not come back as a generated world.
    #[test]
    fn a_saved_import_and_a_saved_generate_reopen_into_two_atlases() {
        let key = |origin: Option<&str>| {
            cartalith_io::world_key(&world_key_signature(
                64,
                32,
                4242,
                1234.5,
                0.37,
                true,
                origin_for_key(origin),
                serde_json::json!({"legacy": true}),
            ))
        };
        assert_ne!(key(Some(ORIGIN_IMPORTED)), key(Some(ORIGIN_GENERATED)));
        assert_ne!(key(Some(ORIGIN_REGION)), key(Some(ORIGIN_GENERATED)));
        // ...while the unknown case is the one that deliberately shares.
        assert_eq!(key(None), key(Some(ORIGIN_GENERATED)));
        // An origin from a newer writer keeps its own namespace rather than
        // being folded into one of the three this build knows.
        assert_ne!(key(Some("sculpt")), key(Some(ORIGIN_GENERATED)));
        assert_eq!(origin_for_key(Some("sculpt")), "sculpt");
    }

    #[test]
    fn human_bytes_reads_like_a_status_line() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(999), "999 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.0 GiB");
    }

    #[test]
    fn the_estimate_uses_the_references_own_tile_counts() {
        let e = |z| bake_estimate(z, 2048, 1311, 1024, 0.0);
        assert_eq!(e(3).tiles, 85);
        assert_eq!(e(4).tiles, 341);
        assert_eq!(e(5).tiles, 1365);
        // Clamped rather than allowed to run away.
        assert_eq!(e(99).tiles, e(MAX_BAKE_DEPTH).tiles);
        assert!((bake_estimate(3, 2048, 1311, 1024, 100.0).seconds - 8.5).abs() < 1e-9);
    }

    #[test]
    fn the_byte_estimate_matches_a_real_measured_bake() {
        // `bake_real_world.rs` at 2048x1311, tile 1024, depth 3 reported 85
        // chunks and 233.73 MiB on disk. The prediction must land on that,
        // not merely be "a big number" -- an estimate nobody checked is how a
        // user ends up 3 GiB into a bake they were told would be small.
        let e = bake_estimate(3, 2048, 1311, 1024, 19.0);
        assert_eq!((e.tile_w, e.tile_h), (1024, 655));
        assert_eq!(e.height_bytes, 85 * 1024 * 655 * 4);
        let mib = e.total_bytes as f64 / 1048576.0;
        assert!((mib - 233.7).abs() < 12.0, "predicted {mib:.1} MiB against a measured 233.7");
        // ...and depth 5 really is the multi-gigabyte commitment the doc says.
        let deep = bake_estimate(5, 2048, 1311, 1024, 19.0);
        assert!(deep.total_bytes > 3 * 1024 * 1024 * 1024, "{}", deep.total_bytes);
    }

    #[test]
    fn every_level_shares_one_tile_size() {
        // The property the byte arithmetic rests on: `tile_dims` reads the
        // region's aspect, and a level-z tile's footprint has the same aspect
        // at every z. Checked against `tile_dims` directly rather than
        // assumed, because if it were ever false the estimate would be wrong
        // by a factor that grows with depth.
        let (gw, gh) = (2048usize, 1311usize);
        let want = cartalith_spatial::tile_dims(
            &cartalith_spatial::Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }, 1, 1, 1024);
        for z in 0..6 {
            let n = 1usize << z;
            let got = cartalith_spatial::tile_dims(
                &cartalith_spatial::Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }, n, n, 1024);
            assert_eq!(got, want, "level {z} tile size differs");
        }
    }

    #[test]
    fn an_estimate_before_any_world_does_not_divide_by_zero() {
        let e = bake_estimate(3, 0, 0, 1024, 19.0);
        assert_eq!(e.tiles, 85);
        assert_eq!(e.total_bytes, 0);
    }

    #[test]
    fn finalize_gates_generation_and_editing_but_not_presentation() {
        let mut s = BakeState::new();
        assert!(s.check(Mutation::Generation).is_ok());
        s.finalized = true;
        assert!(s.check(Mutation::Generation).is_err());
        assert!(s.check(Mutation::HeightEdit).is_err());
        assert!(s.check(Mutation::Presentation).is_ok());
    }
}
