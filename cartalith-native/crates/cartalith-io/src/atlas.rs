//! The persistent tile atlas — reference lines 10688-10930, the store the
//! bake writes into and deep zoom reads back from.
//!
//! # What "the atlas" is, in the reference's own terms
//!
//! Not a texture atlas. It is a **per-world, content-addressed cache of baked
//! pyramid chunks**: for each `(z, col, row)` tile of the LOD pyramid, the
//! 16-bit-packed refined heightfield (`rg16`, [`crate::pack_height16`]) plus an
//! optional rendered PNG, stored under a key that begins with a hash of every
//! generation parameter that could change what that tile contains. Change the
//! seed, the sea level, the tectonics — anything `worldKey` hashes — and the
//! key namespace changes with it, so a stale chunk can never be served for a
//! world it was not baked from. That is the whole cache-invalidation story, and
//! it is why [`world_key`] is a hash rather than a counter.
//!
//! # Where this port necessarily diverges: IndexedDB
//!
//! The reference's store is IndexedDB (`atlasOpen`/`atlasPut`/`atlasGet`,
//! 10721-10733) — a browser API with no equivalent here, and one the reference
//! itself feature-detects and degrades to a no-op without
//! (*"browser-only; feature-detected → headless / no-IDB no-op"*). This port
//! stores the same records as files under a caller-supplied root
//! (`user://atlas` from Godot's side), which is the same durability contract:
//! survives a reload, survives a regenerate, is per-world, and is clearable.
//!
//! What is **not** a divergence, and is deliberately kept byte-identical:
//! the key string ([`atlas_key_str`]), the chunk encoding ([`encode_chunk`],
//! which is `packHeight16` unchanged), the ancestor-coverage rule
//! (`cartalith_spatial::pyramid::baked_cover`), and the portable-archive
//! layout ([`atlas_chunk_file`], [`build_atlas_manifest`]). Those four are what
//! a `World/` archive written by this port and one written by the reference
//! have to agree on, and all four are golden-pinned.
//!
//! # `cartalith-spatial` as a dependency of `cartalith-io`
//!
//! New here, and worth stating. A chunk's address (`z`/`col`/`row`) is used
//! identically by the pyramid geometry, the store key and the manifest; the
//! alternative was a second three-field address type in this crate that had to
//! agree with `cartalith_spatial::pyramid::ChunkId` by convention. `spatial` is
//! a leaf crate (`cartalith-jsmath` + `serde`, no gdext, no pipeline state), so
//! this adds no cycle and nothing Godot-shaped — the same test
//! `ARCHITECTURE.md` applies to `cartalith-engine`'s own dependency on it.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use cartalith_spatial::pyramid::ChunkId;
use serde::{Deserialize, Serialize};

use crate::{pack_height16, unpack_height16};

/// `ATLAS_MANIFEST` (reference line 10879) — the portable archive's index.
pub const ATLAS_MANIFEST: &str = "World/atlas.json";

/// `buildAtlasManifest`'s `kind` discriminator, checked on import.
pub const ATLAS_KIND: &str = "cartalith-atlas";

// ---------------------------------------------------------------------------
// world key
// ---------------------------------------------------------------------------

/// FNV-1a (32-bit), lower-case hex — the hash `worldKey` (reference line 10703)
/// runs over its parameter signature.
///
/// The reference hashes `sig.charCodeAt(i)`, i.e. **UTF-16 code units**, not
/// bytes. For the ASCII JSON a parameter signature actually is, that is the
/// same thing; it is spelled out here because a caller that passed a non-ASCII
/// string would silently diverge, and `chars()` (Unicode scalar values) would
/// be wrong for astral characters in a way `encode_utf16()` is not.
pub fn fnv1a32_hex(s: &str) -> String {
    let mut h: u32 = 2166136261;
    for u in s.encode_utf16() {
        h ^= u as u32;
        h = h.wrapping_mul(16777619);
    }
    format!("{h:x}")
}

/// `worldKey()` (reference line 10703) — *"FNV-1a hash (hex) over the
/// render-affecting state subset (the params that determine `field`/climate).
/// Excludes viz/debug/mode/view state. Same params → same key; a changed seed
/// (or any gen param) → a fresh atlas."*
///
/// The **signature string is the caller's**, not this function's, and that is
/// the one real structural difference from the reference. There, `worldKey`
/// reaches into a `state` global and `JSON.stringify`s thirteen named fields;
/// here nothing below `cartalith-godot` has a `state`, and the port's parameter
/// struct is not field-for-field the reference's anyway (`GENERATION_
/// PARAMETERS.md`). What must be preserved — and is, by construction — is the
/// *property*: the signature must include every parameter that changes the
/// height or climate field and none that only changes how it is drawn. The
/// caller that builds it is `cartalith_godot`'s `atlas_world_key`, whose own
/// doc comment lists the fields and why each is in or out.
pub fn world_key(signature: &str) -> String {
    fnv1a32_hex(signature)
}

// ---------------------------------------------------------------------------
// keys and paths
// ---------------------------------------------------------------------------

/// `atlasKeyStr(wk, ts, z, col, row)` (reference line 10709) — `wk:ts:z:col:row`.
///
/// `ts` is the *tile size* the chunk was baked at, and it is part of the key
/// rather than of the world hash on purpose: two tile sizes over the same world
/// are two valid, coexisting bakes, not a stale one and a fresh one.
pub fn atlas_key_str(wk: &str, ts: usize, c: ChunkId) -> String {
    format!("{wk}:{ts}:{}:{}:{}", c.z, c.col, c.row)
}

/// `atlasMetaKey(wk)` (reference line 10699).
pub fn atlas_meta_key(wk: &str) -> String {
    format!("meta:{wk}")
}

/// `atlasChunkFile(z, col, row, ext)` (reference line 10880) — a chunk's path
/// inside a portable `World/` archive, grouped by LOD level.
pub fn atlas_chunk_file(c: ChunkId, ext: &str) -> String {
    format!("World/LOD{}/{}_{}_{}.{}", c.z, c.z, c.col, c.row, ext)
}

// ---------------------------------------------------------------------------
// chunk records
// ---------------------------------------------------------------------------

/// One baked chunk: the packed height plus, optionally, its rendered visual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtlasChunk {
    pub id: ChunkId,
    pub w: usize,
    pub h: usize,
    /// `packHeight16` output — `w * h * 4` bytes.
    pub rg16: Vec<u8>,
    /// The reference's `png` blob. `None` where the reference stores `null`,
    /// which is what a headless bake produces there too.
    pub png: Option<Vec<u8>>,
}

/// `atlasEncodeChunk(tile)` (reference line 10712) — a refined tile's `f32`
/// heights to the stored 16-bit form. Round-trips to within one LSB
/// (`1/65535`), which [`decode_chunk`]'s own test pins.
pub fn encode_chunk(id: ChunkId, data: &[f32], w: usize, h: usize, png: Option<Vec<u8>>) -> AtlasChunk {
    AtlasChunk { id, w, h, rg16: pack_height16(data, w * h), png }
}

/// `atlasDecodeChunk(rec)` (reference line 10713) — the inverse.
pub fn decode_chunk(c: &AtlasChunk) -> Vec<f32> {
    unpack_height16(&c.rg16, c.w * c.h)
}

/// `atlasMetaRec(wk, opts)` (reference line 10700) — the per-world status
/// record. The reference's own note: *"Distinct key prefix; NO worldKey field
/// so the `world` index excludes it from chunk queries"* — an IndexedDB
/// indexing detail with no equivalent in a filesystem store, where the meta
/// record simply is not a chunk file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasMeta {
    /// The reference's `ts`: the tile size the world was baked at.
    pub ts: usize,
    pub ver: String,
    pub chunks: usize,
    /// `Date.now()` milliseconds.
    pub time: i64,
}

// ---------------------------------------------------------------------------
// the store
// ---------------------------------------------------------------------------

/// A chunk's full address in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtlasKey {
    pub ts: usize,
    pub id: ChunkId,
}

/// The filesystem stand-in for the reference's IndexedDB object store.
///
/// Layout, chosen so that the reference's key components are all recoverable
/// from the path (which is what makes [`keys_for_world`](Self::keys_for_world)
/// a directory walk rather than a separate index):
///
/// ```text
/// <root>/<worldKey>/meta.json
/// <root>/<worldKey>/ts<tileSize>/LOD<z>/<z>_<col>_<row>.bin   <- rg16
/// <root>/<worldKey>/ts<tileSize>/LOD<z>/<z>_<col>_<row>.png   <- visual
/// ```
///
/// Every operation returns `io::Result` rather than the reference's
/// swallow-and-return-false: a bake that silently wrote nothing is exactly the
/// failure `CLAUDE.md`'s *"watch for silently-empty golden output"* rule exists
/// to catch, and the Godot boundary turns these into a logged error anyway.
#[derive(Debug, Clone)]
pub struct AtlasStore {
    root: PathBuf,
}

impl AtlasStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        AtlasStore { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn world_dir(&self, wk: &str) -> PathBuf {
        self.root.join(sanitise(wk))
    }

    fn chunk_dir(&self, wk: &str, ts: usize, z: u32) -> PathBuf {
        self.world_dir(wk).join(format!("ts{ts}")).join(format!("LOD{z}"))
    }

    fn chunk_stem(&self, wk: &str, ts: usize, c: ChunkId) -> PathBuf {
        self.chunk_dir(wk, ts, c.z).join(format!("{}_{}_{}", c.z, c.col, c.row))
    }

    /// `atlasPut(rec)` (reference line 10731).
    pub fn put(&self, wk: &str, ts: usize, chunk: &AtlasChunk) -> io::Result<()> {
        let stem = self.chunk_stem(wk, ts, chunk.id);
        fs::create_dir_all(stem.parent().expect("chunk stem always has a parent"))?;
        fs::write(stem.with_extension("bin"), &chunk.rg16)?;
        let png_path = stem.with_extension("png");
        match &chunk.png {
            Some(bytes) => fs::write(png_path, bytes)?,
            // A re-bake that produces no visual must not leave the previous
            // one behind claiming to describe the new heights.
            None => match fs::remove_file(&png_path) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            },
        }
        Ok(())
    }

    /// `atlasGet(key)` (reference line 10732). `Ok(None)` for a chunk that was
    /// never baked, which is not an error.
    pub fn get(&self, wk: &str, ts: usize, id: ChunkId) -> io::Result<Option<AtlasChunk>> {
        let stem = self.chunk_stem(wk, ts, id);
        let rg16 = match fs::read(stem.with_extension("bin")) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let png = match fs::read(stem.with_extension("png")) {
            Ok(b) => Some(b),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        // The read *is* the access, and this is where the access order
        // `evict_to` sorts on gets written. See `touch_access` for why the
        // filesystem has to be told rather than asked.
        touch_access(&stem.with_extension("bin"));
        // `w`/`h` are not stored beside the bytes: `rg16` is exactly four bytes
        // per cell, and the tile's dimensions are a pure function of the
        // pyramid level and tile size the key already carries. Recovering them
        // from the manifest instead would make a chunk unreadable without one.
        Ok(Some(AtlasChunk { id, w: 0, h: 0, rg16, png }))
    }

    /// Byte length of a stored chunk's `rg16`, without reading it.
    /// `Ok(None)` if not baked.
    ///
    /// **Nothing calls this.** Cache sizing goes through [`Self::key_bytes`],
    /// which stats *both* files of the pair and is what [`Self::world_bytes`]
    /// and eviction actually use; this one covers the `.bin` alone.
    /// `UNWIRED_FUNCTIONS.md` registers it as the superseded half of that
    /// pair. Corrected 2026-09-01, where this said it was "used to size the
    /// status readout".
    pub fn chunk_len(&self, wk: &str, ts: usize, id: ChunkId) -> io::Result<Option<u64>> {
        match fs::metadata(self.chunk_stem(wk, ts, id).with_extension("bin")) {
            Ok(m) => Ok(Some(m.len())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// `atlasDelete(key)` (reference line 10733).
    pub fn delete(&self, wk: &str, ts: usize, id: ChunkId) -> io::Result<()> {
        let stem = self.chunk_stem(wk, ts, id);
        for ext in ["bin", "png"] {
            match fs::remove_file(stem.with_extension(ext)) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// `atlasKeysForWorld(wk)` (reference line 10735) — every baked chunk this
    /// world holds, recovered by walking the directory layout. Sorted, so a
    /// caller's iteration order (and therefore an exported archive's entry
    /// order) is deterministic across platforms.
    ///
    /// An unreadable individual name is skipped rather than failing the whole
    /// listing: a stray file in the cache directory must not make an otherwise
    /// good atlas unusable.
    pub fn keys_for_world(&self, wk: &str) -> io::Result<BTreeSet<AtlasKey>> {
        let mut out = BTreeSet::new();
        let dir = self.world_dir(wk);
        let ts_dirs = match fs::read_dir(&dir) {
            Ok(d) => d,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e),
        };
        for ts_ent in ts_dirs.flatten() {
            let Some(ts) = ts_ent.file_name().to_str().and_then(|s| s.strip_prefix("ts")?.parse::<usize>().ok())
            else {
                continue;
            };
            let Ok(lod_dirs) = fs::read_dir(ts_ent.path()) else { continue };
            for lod_ent in lod_dirs.flatten() {
                if lod_ent.file_name().to_str().and_then(|s| s.strip_prefix("LOD")).is_none() {
                    continue;
                }
                let Ok(files) = fs::read_dir(lod_ent.path()) else { continue };
                for f in files.flatten() {
                    let name = f.file_name();
                    let Some(name) = name.to_str() else { continue };
                    let Some(stem) = name.strip_suffix(".bin") else { continue };
                    let mut parts = stem.split('_');
                    let (Some(z), Some(col), Some(row), None) =
                        (parts.next(), parts.next(), parts.next(), parts.next())
                    else {
                        continue;
                    };
                    let (Ok(z), Ok(col), Ok(row)) = (z.parse(), col.parse(), row.parse()) else {
                        continue;
                    };
                    out.insert(AtlasKey { ts, id: ChunkId::new(z, col, row) });
                }
            }
        }
        Ok(out)
    }

    /// `atlasGetMeta(wk)` (reference line 10736).
    pub fn get_meta(&self, wk: &str) -> io::Result<Option<AtlasMeta>> {
        match fs::read(self.world_dir(wk).join("meta.json")) {
            Ok(b) => Ok(serde_json::from_slice(&b).ok()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// `atlasPutMeta(wk)` (reference line 10737).
    pub fn put_meta(&self, wk: &str, meta: &AtlasMeta) -> io::Result<()> {
        let dir = self.world_dir(wk);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("meta.json"), serde_json::to_vec_pretty(meta).map_err(io::Error::other)?)
    }

    /// `atlasClearWorld(wk)` (reference line 10738) — returns how many chunks
    /// were removed, which the reference's boolean does not report and the
    /// Preferences "Clear caches…" row (`GUI_GAP_REGISTER.md` PR-12) wants.
    pub fn clear_world(&self, wk: &str) -> io::Result<usize> {
        let n = self.keys_for_world(wk)?.len();
        let dir = self.world_dir(wk);
        match fs::remove_dir_all(&dir) {
            Ok(()) => Ok(n),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(0),
            Err(e) => Err(e),
        }
    }

    /// Total bytes this world's chunks occupy — what the status readout and
    /// the cache-size preference both need, and what the reference's
    /// `updateAtlasStatus` (a chunk *count* only) does not offer.
    pub fn world_bytes(&self, wk: &str) -> io::Result<u64> {
        let mut total = 0u64;
        for k in self.keys_for_world(wk)? {
            total += self.key_bytes(wk, k);
        }
        Ok(total)
    }

    /// One chunk's on-disk cost, both files. Metadata only — the point of
    /// [`Self::chunk_len`] applied to the pair, so sizing the cache never
    /// reads a byte of it.
    fn key_bytes(&self, wk: &str, k: AtlasKey) -> u64 {
        let stem = self.chunk_stem(wk, k.ts, k.id);
        ["bin", "png"]
            .iter()
            .filter_map(|ext| fs::metadata(stem.with_extension(ext)).ok())
            .map(|m| m.len())
            .sum()
    }

    /// When a chunk was last *read*, as [`Self::get`] records it.
    ///
    /// Falls back to the modification time (the bake) where the platform has
    /// no access time, and to the epoch where it has neither — both of which
    /// make the chunk look maximally cold, which is the safe direction: the
    /// worst case is evicting something that was in fact warm, never keeping
    /// something stale forever.
    fn last_access(&self, wk: &str, k: AtlasKey) -> SystemTime {
        fs::metadata(self.chunk_stem(wk, k.ts, k.id).with_extension("bin"))
            .and_then(|m| m.accessed().or_else(|_| m.modified()))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }

    /// Every world key that has a directory in this store.
    ///
    /// The directory name *is* the key as [`sanitise`] wrote it, and
    /// `sanitise` is a character filter and therefore idempotent, so a name
    /// read back out here addresses the same directory when handed to
    /// [`Self::keys_for_world`].
    pub fn worlds(&self) -> io::Result<Vec<String>> {
        let entries = match fs::read_dir(&self.root) {
            Ok(d) => d,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut out: Vec<String> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().to_str().map(str::to_string))
            .collect();
        out.sort();
        Ok(out)
    }

    /// Bytes held by **every** world in this store, not only the current one.
    pub fn total_bytes(&self) -> io::Result<u64> {
        let mut total = 0u64;
        for wk in self.worlds()? {
            total += self.world_bytes(&wk)?;
        }
        Ok(total)
    }

    /// Bring the whole cache under `max_bytes` by deleting chunks, coldest
    /// first. Returns the bytes actually freed.
    ///
    /// The engine half of *Preferences ▸ Memory ▸ Atlas cache · Size cap*,
    /// which until now had a status readout and a clear button and nothing in
    /// between.
    ///
    /// **The budget is over the store, not over one world.** A cap that only
    /// saw the current world would leave every previously-generated world's
    /// chunks on disk untouched — which is most of what a long session
    /// accumulates, and exactly what a user sets a cap to bound.
    ///
    /// Order:
    ///
    /// 1. **Coldest first**, by last read ([`Self::last_access`]).
    /// 2. **At equal age, the deeper LOD level first.** A coarse tile covers
    ///    the whole world at a glance, is what a re-bake has to rebuild
    ///    before anything else, and costs a quarter of what the level below
    ///    it does — cheaper to keep and more expensive to lose.
    /// 3. Then by world key and chunk address, only so the order is total and
    ///    therefore reproducible across platforms.
    ///
    /// **A world's coarsest level is never evicted.** Reducing a world to no
    /// tiles at all would draw as an empty atlas rather than as a coarse one,
    /// and a size cap is a budget, not a clear button — [`Self::clear_world`]
    /// is the clear button. A cache that cannot reach the budget without
    /// crossing that line stops above it and reports what it did free.
    pub fn evict_to(&self, max_bytes: u64) -> io::Result<u64> {
        let mut total = 0u64;
        // (last access, level, world key, chunk, bytes)
        let mut cold: Vec<(SystemTime, u32, String, AtlasKey, u64)> = Vec::new();
        for wk in self.worlds()? {
            let keys = self.keys_for_world(&wk)?;
            let Some(coarsest) = keys.iter().map(|k| k.id.z).min() else { continue };
            for k in keys {
                let bytes = self.key_bytes(&wk, k);
                total += bytes;
                if k.id.z == coarsest {
                    continue;
                }
                cold.push((self.last_access(&wk, k), k.id.z, wk.clone(), k, bytes));
            }
        }
        cold.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then((&a.2, a.3).cmp(&(&b.2, b.3))));
        let mut freed = 0u64;
        for (_, _, wk, k, bytes) in cold {
            if total <= max_bytes {
                break;
            }
            self.delete(&wk, k.ts, k.id)?;
            total -= bytes;
            freed += bytes;
        }
        Ok(freed)
    }
}

/// Record that a chunk was just read, by writing its access time.
///
/// The filesystem **is** the access record here, because there is nowhere
/// else to keep one: [`AtlasStore`] is rebuilt from a root path on every call
/// (`bake_bridge::BakeState::store`) and holds no state between them, so an
/// in-memory LRU would be born empty on every use.
///
/// It is written explicitly rather than left to the OS because Windows ships
/// with NTFS last-access updates disabled (Win10 onwards), so an atime read
/// back there would silently be the bake time and [`AtlasStore::evict_to`]
/// would degrade from "coldest first" to "oldest bake first" without saying
/// so.
///
/// Best-effort and deliberately silent: a read-only atlas (an imported
/// archive, a cache on a read-only mount) must still serve chunks, and a
/// failed touch costs eviction accuracy and nothing else.
fn touch_access(path: &Path) {
    if let Ok(f) = fs::File::options().write(true).open(path) {
        let _ = f.set_times(fs::FileTimes::new().set_accessed(SystemTime::now()));
    }
}

/// Keep a world key (or anything else) from escaping the atlas root.
///
/// [`world_key`] only ever produces `[0-9a-f]{1,8}`, so on the shipped path
/// this is the identity. It exists because the key crosses the gdext boundary
/// as a caller-supplied string and a `..` in it would otherwise be a directory
/// traversal — the same defensive stance `cartalith-rust-conventions` takes
/// about panics there, applied to paths.
fn sanitise(s: &str) -> String {
    let cleaned: String =
        s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    if cleaned.is_empty() { "unkeyed".to_string() } else { cleaned }
}

// ---------------------------------------------------------------------------
// the portable manifest
// ---------------------------------------------------------------------------

/// One chunk's record in a portable `World/` archive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasChunkRecord {
    pub z: u32,
    pub col: u32,
    pub row: u32,
    pub w: usize,
    pub h: usize,
    pub bin: String,
    pub png: Option<String>,
    pub gzip: bool,
}

/// `buildAtlasManifest`'s output (reference line 10882), field for field and in
/// the reference's own key order — `serde_json`'s pretty printer emits the
/// declaration order with the same two-space indent `JSON.stringify(m, null, 2)`
/// uses, and every value here is an integer, string, bool or null, so the two
/// renderings are byte-identical. (`crate::tiles`' hand-rolled writer exists
/// because *that* manifest carries fractional coarse bounds, which serde and
/// `JSON.stringify` format differently. This one carries none.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasManifest {
    pub schema: u32,
    pub kind: String,
    #[serde(rename = "worldKey")]
    pub world_key: String,
    pub version: String,
    #[serde(rename = "tileSize")]
    pub tile_size: usize,
    pub time: i64,
    pub count: usize,
    pub params: Option<serde_json::Value>,
    pub chunks: Vec<AtlasChunkRecord>,
}

/// A chunk descriptor as [`build_atlas_manifest`] takes it — the reference's
/// `{z, col, row, w, h, gzip, png}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasChunkDesc {
    pub id: ChunkId,
    pub w: usize,
    pub h: usize,
    pub gzip: bool,
    pub png: bool,
}

/// `buildAtlasManifest(wk, chunks, opts)` (reference line 10882) — pure.
pub fn build_atlas_manifest(
    wk: &str,
    chunks: &[AtlasChunkDesc],
    tile_size: usize,
    version: &str,
    time: i64,
    params: Option<serde_json::Value>,
) -> AtlasManifest {
    AtlasManifest {
        schema: 1,
        kind: ATLAS_KIND.to_string(),
        world_key: wk.to_string(),
        version: version.to_string(),
        tile_size,
        time,
        count: chunks.len(),
        params,
        chunks: chunks
            .iter()
            .map(|c| AtlasChunkRecord {
                z: c.id.z,
                col: c.id.col,
                row: c.id.row,
                w: c.w,
                h: c.h,
                bin: atlas_chunk_file(c.id, if c.gzip { "bin.gz" } else { "bin" }),
                png: c.png.then(|| atlas_chunk_file(c.id, "png")),
                gzip: c.gzip,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cartalith-atlas-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn chunk(z: u32, col: u32, row: u32, n: usize) -> AtlasChunk {
        let data: Vec<f32> = (0..n).map(|i| (i as f32 * 0.017) % 1.0).collect();
        encode_chunk(ChunkId::new(z, col, row), &data, n, 1, Some(vec![0x89, b'P', b'N', b'G']))
    }

    #[test]
    fn fnv1a_matches_the_reference_on_known_strings() {
        // Extracted from the reference's own `worldKey` hash loop, run over
        // three fixed strings so the hash is pinned without reproducing the
        // reference's `state` object.
        assert_eq!(fnv1a32_hex(""), "811c9dc5");
        assert_eq!(fnv1a32_hex("a"), "e40c292c");
        assert_eq!(fnv1a32_hex("cartalith"), "8bd9c985");
    }

    #[test]
    fn a_changed_signature_changes_the_key() {
        // The invalidation property the whole atlas rests on.
        assert_ne!(world_key("[512,0.42]"), world_key("[512,0.5]"));
        assert_eq!(world_key("[512,0.42]"), world_key("[512,0.42]"));
    }

    #[test]
    fn key_and_path_spellings_match_the_reference() {
        assert_eq!(atlas_key_str("abc", 1024, ChunkId::new(3, 5, 7)), "abc:1024:3:5:7");
        assert_eq!(atlas_meta_key("abc"), "meta:abc");
        assert_eq!(atlas_chunk_file(ChunkId::new(0, 0, 0), "bin"), "World/LOD0/0_0_0.bin");
        assert_eq!(atlas_chunk_file(ChunkId::new(3, 5, 7), "bin.gz"), "World/LOD3/3_5_7.bin.gz");
        assert_eq!(atlas_chunk_file(ChunkId::new(2, 1, 0), "png"), "World/LOD2/2_1_0.png");
    }

    #[test]
    fn a_chunk_round_trips_through_the_store() {
        let root = tmp("roundtrip");
        let s = AtlasStore::new(&root);
        let c = chunk(2, 1, 3, 64);
        s.put("deadbeef", 1024, &c).unwrap();
        let back = s.get("deadbeef", 1024, ChunkId::new(2, 1, 3)).unwrap().unwrap();
        assert_eq!(back.rg16, c.rg16);
        assert_eq!(back.png, c.png);
        assert!(s.get("deadbeef", 1024, ChunkId::new(2, 1, 4)).unwrap().is_none());
        // ...and a *different* world key sees nothing at all.
        assert!(s.get("cafe", 1024, ChunkId::new(2, 1, 3)).unwrap().is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn keys_for_world_recovers_every_address_from_the_layout() {
        let root = tmp("keys");
        let s = AtlasStore::new(&root);
        let want: BTreeSet<AtlasKey> = [(0, 0, 0), (1, 0, 1), (1, 1, 1), (3, 7, 2)]
            .into_iter()
            .map(|(z, c, r)| AtlasKey { ts: 512, id: ChunkId::new(z, c, r) })
            .collect();
        for k in &want {
            s.put("w1", k.ts, &chunk(k.id.z, k.id.col, k.id.row, 16)).unwrap();
        }
        // A second tile size coexists rather than replacing the first.
        s.put("w1", 1024, &chunk(0, 0, 0, 16)).unwrap();
        let got = s.keys_for_world("w1").unwrap();
        assert_eq!(got.len(), 5);
        assert!(want.is_subset(&got));
        assert!(got.contains(&AtlasKey { ts: 1024, id: ChunkId::new(0, 0, 0) }));
        assert!(s.keys_for_world("never-baked").unwrap().is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn clear_world_removes_only_that_world() {
        let root = tmp("clear");
        let s = AtlasStore::new(&root);
        s.put("a", 512, &chunk(0, 0, 0, 16)).unwrap();
        s.put("b", 512, &chunk(0, 0, 0, 16)).unwrap();
        assert_eq!(s.clear_world("a").unwrap(), 1);
        assert!(s.keys_for_world("a").unwrap().is_empty());
        assert_eq!(s.keys_for_world("b").unwrap().len(), 1);
        // Clearing an absent world is not an error.
        assert_eq!(s.clear_world("never").unwrap(), 0);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_key_cannot_escape_the_atlas_root() {
        let root = tmp("escape");
        let s = AtlasStore::new(&root);
        s.put("../../evil", 512, &chunk(0, 0, 0, 16)).unwrap();
        assert!(root.join("evil").is_dir(), "traversal was not neutralised");
        assert!(!root.parent().unwrap().join("evil").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rg16_round_trips_to_within_one_lsb() {
        let data: Vec<f32> = (0..256).map(|i| i as f32 / 255.0).collect();
        let c = encode_chunk(ChunkId::new(1, 0, 0), &data, 16, 16, None);
        assert_eq!(c.rg16.len(), 16 * 16 * 4);
        let back = decode_chunk(&c);
        let maxd = data.iter().zip(&back).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        assert!(maxd <= 1.0 / 65535.0, "round trip lost {maxd}");
    }

    #[test]
    fn meta_round_trips() {
        let root = tmp("meta");
        let s = AtlasStore::new(&root);
        assert!(s.get_meta("w").unwrap().is_none());
        let m = AtlasMeta { ts: 1024, ver: "0.1".into(), chunks: 85, time: 1_700_000_000_000 };
        s.put_meta("w", &m).unwrap();
        assert_eq!(s.get_meta("w").unwrap().unwrap(), m);
        let _ = fs::remove_dir_all(&root);
    }

    // -- eviction (`Preferences ▸ Memory ▸ Atlas cache · Size cap`) --------

    /// Real clocks have a resolution a test cannot rely on — two `put`s in
    /// the same millisecond are the same age — so age is *stated* here
    /// rather than produced, through the same access time `touch_access`
    /// writes and `last_access` reads.
    fn age(s: &AtlasStore, wk: &str, k: AtlasKey, secs: u64) {
        let p = s.chunk_stem(wk, k.ts, k.id).with_extension("bin");
        let f = fs::File::options().write(true).open(p).unwrap();
        f.set_times(
            fs::FileTimes::new()
                .set_accessed(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs)),
        )
        .unwrap();
    }

    fn key(z: u32, col: u32, row: u32) -> AtlasKey {
        AtlasKey { ts: 512, id: ChunkId::new(z, col, row) }
    }

    /// Each chunk here is 16 cells of `rg16` (64 bytes) plus a 4-byte PNG
    /// stub, so every count in these tests is an exact byte figure rather
    /// than an approximate one.
    const CHUNK_BYTES: u64 = 16 * 4 + 4;

    #[test]
    fn evicting_to_a_budget_below_the_cache_reaches_the_budget() {
        let root = tmp("evict-budget");
        let s = AtlasStore::new(&root);
        // One coarse tile (never evictable) and four deep ones.
        s.put("w1", 512, &chunk(0, 0, 0, 16)).unwrap();
        for (col, row) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            s.put("w1", 512, &chunk(2, col, row, 16)).unwrap();
        }
        assert_eq!(s.total_bytes().unwrap(), 5 * CHUNK_BYTES);
        let budget = 2 * CHUNK_BYTES;
        let freed = s.evict_to(budget).unwrap();
        assert_eq!(freed, 3 * CHUNK_BYTES);
        assert!(s.total_bytes().unwrap() <= budget, "the budget was not reached");
        // ...and no further eviction is owed once inside it.
        assert_eq!(s.evict_to(budget).unwrap(), 0);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn eviction_takes_the_coldest_chunk_first() {
        let root = tmp("evict-cold");
        let s = AtlasStore::new(&root);
        s.put("w1", 512, &chunk(0, 0, 0, 16)).unwrap();
        for row in 0..3 {
            s.put("w1", 512, &chunk(2, 0, row, 16)).unwrap();
        }
        // Row 1 is the coldest, row 2 the warmest.
        age(&s, "w1", key(2, 0, 0), 2_000);
        age(&s, "w1", key(2, 0, 1), 1_000);
        age(&s, "w1", key(2, 0, 2), 3_000);
        s.evict_to(3 * CHUNK_BYTES).unwrap();
        let left = s.keys_for_world("w1").unwrap();
        assert!(!left.contains(&key(2, 0, 1)), "the coldest chunk survived");
        assert!(left.contains(&key(2, 0, 0)) && left.contains(&key(2, 0, 2)));
        let _ = fs::remove_dir_all(&root);
    }

    /// At the same age the deeper level goes first: a coarse tile is cheaper
    /// to keep and more expensive to lose.
    #[test]
    fn at_equal_age_the_deeper_level_goes_first() {
        let root = tmp("evict-level");
        let s = AtlasStore::new(&root);
        s.put("w1", 512, &chunk(0, 0, 0, 16)).unwrap(); // coarsest: protected
        s.put("w1", 512, &chunk(1, 0, 0, 16)).unwrap();
        s.put("w1", 512, &chunk(3, 0, 0, 16)).unwrap();
        for k in [key(1, 0, 0), key(3, 0, 0)] {
            age(&s, "w1", k, 1_000);
        }
        s.evict_to(2 * CHUNK_BYTES).unwrap();
        let left = s.keys_for_world("w1").unwrap();
        assert!(!left.contains(&key(3, 0, 0)), "the deep level survived a tie");
        assert!(left.contains(&key(1, 0, 0)) && left.contains(&key(0, 0, 0)));
        let _ = fs::remove_dir_all(&root);
    }

    /// The floor: a budget of zero still leaves every world its coarsest
    /// level, so the atlas reads as coarse rather than as absent.
    #[test]
    fn eviction_never_takes_a_world_below_one_level() {
        let root = tmp("evict-floor");
        let s = AtlasStore::new(&root);
        for wk in ["w1", "w2"] {
            s.put(wk, 512, &chunk(1, 0, 0, 16)).unwrap();
            s.put(wk, 512, &chunk(1, 1, 0, 16)).unwrap();
            s.put(wk, 512, &chunk(4, 0, 0, 16)).unwrap();
        }
        s.evict_to(0).unwrap();
        for wk in ["w1", "w2"] {
            let left = s.keys_for_world(wk).unwrap();
            assert_eq!(left.len(), 2, "{wk} kept the wrong number of chunks");
            assert!(left.iter().all(|k| k.id.z == 1), "{wk} kept a level it should have evicted");
        }
        assert!(s.total_bytes().unwrap() > 0, "a size cap is not a clear button");
        let _ = fs::remove_dir_all(&root);
    }

    /// The budget is over the *store*: an old world nobody is looking at is
    /// exactly what a long session accumulates, and is evictable.
    #[test]
    fn the_budget_spans_every_world_in_the_store() {
        let root = tmp("evict-worlds");
        let s = AtlasStore::new(&root);
        for wk in ["old", "new"] {
            s.put(wk, 512, &chunk(0, 0, 0, 16)).unwrap();
            s.put(wk, 512, &chunk(2, 0, 0, 16)).unwrap();
        }
        age(&s, "old", key(2, 0, 0), 1_000);
        age(&s, "new", key(2, 0, 0), 9_000);
        assert_eq!(s.worlds().unwrap(), vec!["new".to_string(), "old".to_string()]);
        s.evict_to(3 * CHUNK_BYTES).unwrap();
        assert!(s.keys_for_world("old").unwrap().len() == 1, "the cold world was not touched");
        assert!(s.keys_for_world("new").unwrap().len() == 2, "the warm world was evicted first");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn evicting_an_empty_store_is_not_an_error() {
        let root = tmp("evict-empty");
        let s = AtlasStore::new(&root);
        assert_eq!(s.evict_to(0).unwrap(), 0);
        assert_eq!(s.total_bytes().unwrap(), 0);
        assert!(s.worlds().unwrap().is_empty());
    }

    /// `get` is what makes a chunk warm, and it must not need a writable
    /// clock to succeed — the returned bytes are the contract, the touch is
    /// bookkeeping.
    #[test]
    fn reading_a_chunk_refreshes_its_access_time() {
        let root = tmp("evict-touch");
        let s = AtlasStore::new(&root);
        s.put("w1", 512, &chunk(2, 0, 0, 16)).unwrap();
        age(&s, "w1", key(2, 0, 0), 1_000);
        assert_eq!(s.last_access("w1", key(2, 0, 0)), SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000));
        assert!(s.get("w1", 512, ChunkId::new(2, 0, 0)).unwrap().is_some());
        assert!(
            s.last_access("w1", key(2, 0, 0)) > SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000),
            "a read left the chunk as cold as it found it"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
