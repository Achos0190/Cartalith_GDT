//! Ruling FR (`LARGE_ITEM_RULINGS.md`, owner 2026-11-16): **the whole asset library persists per device.**
//!
//! One store, in the app's private user directory, holding everything the Asset Library holds: every family and set, custom
//! slots, packs' pictures, slices, refined assets, tags, scatter rules, collections and pack info. It generalises Ruling FP's
//! `user://custom_pins.zip` (pins only) and **replaces** it: there is no second store, and an existing `custom_pins.zip` is
//! migrated into this one on the first start ([`LibraryStore::open`]).
//!
//! # Layout
//!
//! ```text
//! <dir>/index.json              the library record (cartalith_assets::LibraryFile) + "store_version" + a per-item "h"
//! <dir>/img/<key>.png           one file per distinct picture; key = <16 hex content hash>-<w>x<h>
//! <dir>/index.json.corrupt-<n>  an index that would not parse, moved aside (never deleted, never "repaired")
//! <dir>/img/<key>.png.corrupt   a picture that failed its check at load, moved aside
//! ```
//!
//! # Rules this module keeps
//!
//! * **Incremental.** [`LibraryStore::sync`] writes only the pictures the disk does not hold yet, then the (small) index when it
//!   changed. It never rewrites a pack. A budget in milliseconds lets the app spread an import of hundreds of pictures over
//!   several ticks instead of freezing the UI; the index is written only once every picture it names is on disk.
//! * **Atomic.** Every file is written to `<name>.tmp`, flushed (`sync_all`), then renamed over its name
//!   ([`write_atomic`]). A crash leaves the old file or the new one, never half of either; a stale `.tmp` is swept at open.
//!   Order matters: pictures, then the index, then the garbage collection of pictures the index no longer names. A crash
//!   between any two steps leaves a store that loads (at worst with orphan pictures the next sync removes).
//! * **Never repaired into something plausible.** A picture whose file is missing, will not decode, or whose content hash is
//!   not the one its name carries is *skipped* and reported in [`LoadReport::skipped`]; its slot stays. A corrupt file is moved
//!   aside. An index that does not parse is moved aside and the library starts empty with garbage collection **off** for the
//!   session, so the pictures that index named are not deleted behind the user's back.
//! * **Change detection** is a fingerprint, not a flag every mutator must remember to raise: the serialised record plus a
//!   strided sample of every picture's bytes. A pixel edit that changes none of the sampled bytes, the size, the name or the
//!   record would go unseen until the next full check (the app flushes with an unlimited budget at exit); no library operation
//!   edits a few pixels in place, they all replace or re-encode a picture.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use cartalith_assets::{DecodedImage, LibraryFile, decode_png, encode_png, parse_library_json};

use super::{AssetLibrarySession, PackImportMode};

/// The index's own format number. Bumped only if the layout above changes incompatibly; a store with a higher number than this
/// build knows is treated like a corrupt index (moved aside, nothing deleted) rather than read half-understood.
pub const STORE_VERSION: u32 = 1;

/// What opening the store found, for the shell to word (a warning per skipped entry, a line for a migration).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LoadReport {
    /// Slots the library holds after the load (frozen slots with items and every custom slot).
    pub slots: usize,
    /// Pictures restored.
    pub items: usize,
    /// One sentence per entry that was skipped and why. Empty for a clean load.
    pub skipped: Vec<String>,
    /// `true` when `index.json` existed but could not be used and was moved aside.
    pub index_corrupt: bool,
    /// Pins the legacy `custom_pins.zip` contributed on this first start (`0` when there was none or it was not read).
    pub migrated_pins: usize,
    /// `true` when no store existed yet (a first start, with or without a legacy pin file).
    pub fresh: bool,
}

/// What one [`LibraryStore::sync`] did.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SyncReport {
    /// Pictures written this call.
    pub written: usize,
    /// Orphan files removed (a picture the index no longer names, a stale `.tmp`).
    pub removed: usize,
    /// `true` when the index was rewritten this call.
    pub index_written: bool,
    /// `true` when work remains (the budget ran out before every picture was on disk); call again.
    pub pending: bool,
    /// `true` when nothing had changed since the last sync (the cheap path).
    pub unchanged: bool,
    /// A write that failed (disk full, permissions). The store keeps trying on the next call; nothing is lost in memory.
    pub error: Option<String>,
}

/// The per-device store: where it lives and what it knows is on disk.
pub struct LibraryStore {
    dir: PathBuf,
    /// Picture keys known to be on disk and valid (loaded and checked, or written by this process).
    on_disk: HashSet<String>,
    /// The fingerprint of the library as last fully synced; `None` before the first sync.
    last_fp: Option<u64>,
    /// The index text last written (or read), to skip an identical rewrite.
    last_index: Option<String>,
    /// `false` after an unusable index: no picture may be deleted until the next start (see the module doc).
    gc_enabled: bool,
}

/// A 64-bit content hash over the pixel bytes, eight at a time (multiply-xorshift, wyhash-like). **Not cryptographic**: it
/// names a file and detects accidental damage (a truncated or flipped file), which is all this store needs. Width and height are
/// mixed in so two images of different shape with equal bytes cannot share a key.
fn content_hash(img: &DecodedImage) -> u64 {
    const K: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ (u64::from(img.w) << 32 | u64::from(img.h));
    let mut chunks = img.rgba.chunks_exact(8);
    for c in &mut chunks {
        let w = u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
        h = (h ^ w).wrapping_mul(K);
        h ^= h >> 29;
    }
    for &b in chunks.remainder() {
        h = (h ^ u64::from(b)).wrapping_mul(K);
    }
    h ^ (h >> 32)
}

/// The file stem a picture is stored under: `<16 hex>-<w>x<h>`.
pub fn content_key(img: &DecodedImage) -> String {
    format!("{:016x}-{}x{}", content_hash(img), img.w, img.h)
}

/// `true` for exactly the shape [`content_key`] writes. Applied to every key read from an index **before** it becomes part of a
/// path, so a hostile or damaged index cannot name `../../anything`.
fn is_valid_key(k: &str) -> bool {
    let Some((hash, dims)) = k.split_once('-') else { return false };
    let Some((w, h)) = dims.split_once('x') else { return false };
    hash.len() == 16
        && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && !w.is_empty()
        && !h.is_empty()
        && w.len() <= 6
        && h.len() <= 6
        && w.bytes().all(|b| b.is_ascii_digit())
        && h.bytes().all(|b| b.is_ascii_digit())
}

/// A fast fingerprint of one picture for change detection: size, length and 64 words sampled at an even stride (see the module
/// doc for what this can and cannot see).
fn weak_sig(img: &DecodedImage) -> u64 {
    const K: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut h = (u64::from(img.w) << 32 | u64::from(img.h)) ^ (img.rgba.len() as u64).wrapping_mul(K);
    let words = img.rgba.len() / 8;
    let step = (words / 64).max(1);
    let mut i = 0;
    while i < words {
        let c = &img.rgba[i * 8..i * 8 + 8];
        let w = u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
        h = (h ^ w).wrapping_mul(K);
        h ^= h >> 31;
        i += step;
    }
    h
}

/// FNV-1a, 64-bit, of a byte string (the change-detection hash of the index text).
fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Writes `bytes` to `path` so that a crash leaves either the old file or the whole new one: the bytes go to `<path>.tmp`, are
/// flushed to the disk, and the temporary is renamed over `path` (`rename` replaces an existing file on Windows and Unix).
/// The temporary is removed if any step fails, so a failed write leaves no litter.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

impl LibraryStore {
    fn index_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }

    fn img_dir(&self) -> PathBuf {
        self.dir.join("img")
    }

    fn img_path(&self, key: &str) -> PathBuf {
        self.img_dir().join(format!("{key}.png"))
    }

    /// The store's directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Opens (creating if absent) the store at `dir` and reads it **into** `session`, which must hold no item (a store is read
    /// once, at start; the session's registry is replaced by the record). `legacy_pins` is the path of Ruling FP's
    /// `custom_pins.zip`: when no index exists yet and that file does, its pins are imported (keeping what is there) and the
    /// store is written at once; the old file is renamed `custom_pins.zip.migrated` only after that write succeeded, so a
    /// failure at any point leaves it where it was and the next start tries again.
    ///
    /// `Err` only when the directory cannot be created or the session is not empty; every damaged entry is a
    /// [`LoadReport::skipped`] line, never an `Err`.
    pub fn open(session: &mut AssetLibrarySession, dir: &Path, legacy_pins: Option<&Path>) -> Result<(LibraryStore, LoadReport), String> {
        if session.db.total_items() != 0 {
            return Err("the library already holds pictures; a store is read once, at start".to_string());
        }
        fs::create_dir_all(dir.join("img")).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        let mut store = LibraryStore { dir: dir.to_path_buf(), on_disk: HashSet::new(), last_fp: None, last_index: None, gc_enabled: true };
        let mut report = LoadReport::default();
        // A `.tmp` left by a crash mid-write is garbage by construction (the rename never happened); the file it was meant to
        // replace is intact.
        store.sweep_tmp();
        let idx = store.index_path();
        if idx.exists() {
            match fs::read(&idx) {
                Ok(bytes) => store.load_index(session, &bytes, &mut report),
                Err(e) => {
                    report.skipped.push(format!("index.json could not be read ({e}); the library starts empty and nothing on disk is deleted"));
                    report.index_corrupt = true;
                    store.gc_enabled = false;
                }
            }
        } else {
            report.fresh = true;
            if let Some(zip) = legacy_pins.filter(|p| p.exists()) {
                match fs::read(zip) {
                    Ok(bytes) => match session.import_pack(&bytes, "custom pins", PackImportMode::KeepExisting) {
                        Ok(_) => report.migrated_pins = session.pin_catalog().len(),
                        Err(e) => report.skipped.push(format!("{} is not a readable pack ({e}); it was left where it is", zip.display())),
                    },
                    Err(e) => report.skipped.push(format!("{} could not be read ({e}); it was left where it is", zip.display())),
                }
            }
        }
        report.slots = session.db.uids_in_order().iter().filter(|u| session.db.get(u).is_some_and(|s| !session.db.items(u).is_empty() || s.family == cartalith_assets::Family::Custom)).count();
        report.items = session.db.total_items();
        if report.migrated_pins > 0 {
            // The first start with a legacy file: write the store now, with no budget, then retire the old file.
            let sr = store.sync(session, 0);
            if sr.error.is_none()
                && !sr.pending
                && let Some(zip) = legacy_pins
            {
                let mut moved = zip.as_os_str().to_owned();
                moved.push(".migrated");
                if let Err(e) = fs::rename(zip, PathBuf::from(moved)) {
                    report.skipped.push(format!("{} was migrated but could not be renamed ({e})", zip.display()));
                }
            } else if let Some(e) = sr.error {
                report.skipped.push(format!("the migrated pins could not be written to the store ({e}); the old file was left where it is"));
            }
        }
        Ok((store, report))
    }

    /// Removes stale `.tmp` files from the store's two directories.
    fn sweep_tmp(&self) -> usize {
        let mut n = 0;
        for d in [self.dir.clone(), self.img_dir()] {
            let Ok(rd) = fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                if e.path().extension().is_some_and(|x| x == "tmp") && fs::remove_file(e.path()).is_ok() {
                    n += 1;
                }
            }
        }
        n
    }

    /// Moves a damaged file aside as `<name>.corrupt` (replacing an older such file), so it is kept for a person to look at and
    /// is never read, deleted by garbage collection, or silently "fixed".
    fn quarantine(path: &Path, suffix: &str) {
        let mut to = path.as_os_str().to_owned();
        to.push(suffix);
        let _ = fs::rename(path, PathBuf::from(to));
    }

    fn load_index(&mut self, session: &mut AssetLibrarySession, bytes: &[u8], report: &mut LoadReport) {
        let corrupt = |store: &mut LibraryStore, report: &mut LoadReport, why: String| {
            let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            Self::quarantine(&store.index_path(), &format!(".corrupt-{stamp}"));
            report.skipped.push(format!("index.json {why}; it was moved aside and the library starts empty (no picture is deleted)"));
            report.index_corrupt = true;
            store.gc_enabled = false;
        };
        let raw: serde_json::Value = match serde_json::from_slice(bytes) {
            Ok(v) => v,
            Err(e) => return corrupt(self, report, format!("is not valid JSON ({e})")),
        };
        let version = raw.get("store_version").and_then(serde_json::Value::as_u64);
        if version != Some(u64::from(STORE_VERSION)) {
            return corrupt(self, report, format!("has store_version {version:?}, this build reads {STORE_VERSION}"));
        }
        let file = match parse_library_json(bytes) {
            Ok(f) => f,
            Err(e) => return corrupt(self, report, format!("is not a library record ({e})")),
        };
        // img index -> the key its "h" names, read from the raw document because the typed record has no field for it.
        let mut key_of: HashMap<usize, String> = HashMap::new();
        for slot in raw.get("slots").and_then(serde_json::Value::as_array).into_iter().flatten() {
            for it in slot.get("items").and_then(serde_json::Value::as_array).into_iter().flatten() {
                if let (Some(img), Some(h)) = (it.get("img").and_then(serde_json::Value::as_u64), it.get("h").and_then(serde_json::Value::as_str)) {
                    key_of.insert(img as usize, h.to_string());
                }
            }
        }
        let mut bytes_of: HashMap<usize, Vec<u8>> = HashMap::new();
        let mut checked: HashMap<String, Option<String>> = HashMap::new();
        for slot in &file.slots {
            for it in &slot.items {
                let label = format!("\"{}\" in {}", it.name, slot.name);
                let Some(key) = key_of.get(&it.img) else {
                    report.skipped.push(format!("{label}: the index names no picture file for it"));
                    continue;
                };
                if !is_valid_key(key) {
                    report.skipped.push(format!("{label}: \"{key}\" is not a picture key"));
                    continue;
                }
                let path = self.img_path(key);
                let verdict = checked.entry(key.clone()).or_insert_with(|| Self::check_picture(&path, key));
                if let Some(why) = verdict {
                    report.skipped.push(format!("{label}: {why}"));
                    continue;
                }
                if let Ok(b) = fs::read(&path) {
                    bytes_of.insert(it.img, b);
                    self.on_disk.insert(key.clone());
                }
            }
        }
        let restored = session.db.apply_library_file_collecting(&file, &bytes_of);
        session.images.clear();
        for (uid, img) in restored {
            session.images.entry(uid).or_default().push(img);
        }
        // `parse_library_json` follows the reference and turns a blank licence into "CC0" on load. A store must hand back what the
        // user left (Ruling CQ Q8 asks about a blank licence at EXPORT, not at every start), so the three pack fields are
        // taken verbatim from the document.
        if let Some(p) = raw.get("pack").and_then(serde_json::Value::as_object) {
            let text = |k: &str| p.get(k).and_then(serde_json::Value::as_str).unwrap_or("").to_string();
            session.db.pack.name = text("name");
            session.db.pack.author = text("author");
            session.db.pack.license = text("license");
        }
        self.last_index = String::from_utf8(bytes.to_vec()).ok();
    }

    /// `None` when `path` holds a PNG that decodes and whose content hash is the one `key` names; otherwise the reason it cannot
    /// be used. A file that exists and fails is moved aside (`.corrupt`); a missing one is just reported.
    fn check_picture(path: &Path, key: &str) -> Option<String> {
        let Ok(bytes) = fs::read(path) else {
            let mut aside = path.as_os_str().to_owned();
            aside.push(".corrupt");
            return Some(if PathBuf::from(aside).exists() {
                format!("{key}.png was damaged earlier and is kept aside as {key}.png.corrupt")
            } else {
                format!("{key}.png is missing")
            });
        };
        match decode_png(&bytes) {
            Ok(img) if content_key(&img) == key => None,
            Ok(_) => {
                Self::quarantine(path, ".corrupt");
                Some(format!("{key}.png does not hold the picture its name says (moved aside as {key}.png.corrupt)"))
            }
            Err(e) => {
                Self::quarantine(path, ".corrupt");
                Some(format!("{key}.png is not a readable PNG ({e}); moved aside as {key}.png.corrupt"))
            }
        }
    }

    /// The fingerprint [`Self::sync`] compares: the serialised record plus every picture's weak signature, in store order.
    fn fingerprint(index_text: &str, session: &AssetLibrarySession) -> u64 {
        let mut h = fnv(index_text.as_bytes());
        for uid in session.db.uids_in_order() {
            for i in 0..session.db.items(&uid).len() {
                if let Some(img) = session.image(&uid, i) {
                    h = (h ^ weak_sig(img)).wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        h
    }

    /// The index document for the session as it is now, and the `(key, uid, item index)` of every picture it names.
    fn build_index(session: &mut AssetLibrarySession, with_keys: bool) -> Result<(String, Vec<(String, String, usize)>), String> {
        let file: LibraryFile = session.db.to_library_file();
        let mut v = serde_json::to_value(&file).map_err(|e| e.to_string())?;
        let mut keys = Vec::new();
        if with_keys {
            // The `img` numbering of `to_library_file` is a running counter over `uids_in_order()` and each slot's items; the
            // same walk gives every item its key, and a slot the record leaves out (an empty frozen one) has no items to number.
            let mut key_by_img: HashMap<usize, String> = HashMap::new();
            let mut n = 0usize;
            for uid in session.db.uids_in_order() {
                for i in 0..session.db.items(&uid).len() {
                    if let Some(img) = session.image(&uid, i) {
                        let k = content_key(img);
                        key_by_img.insert(n, k.clone());
                        keys.push((k, uid.clone(), i));
                    }
                    n += 1;
                }
            }
            for slot in v.get_mut("slots").and_then(serde_json::Value::as_array_mut).into_iter().flatten() {
                for it in slot.get_mut("items").and_then(serde_json::Value::as_array_mut).into_iter().flatten() {
                    let Some(img) = it.get("img").and_then(serde_json::Value::as_u64) else { continue };
                    if let (Some(k), Some(obj)) = (key_by_img.get(&(img as usize)), it.as_object_mut()) {
                        obj.insert("h".to_string(), serde_json::Value::String(k.clone()));
                    }
                }
            }
        }
        if let Some(obj) = v.as_object_mut() {
            obj.insert("store_version".to_string(), serde_json::Value::from(STORE_VERSION));
        }
        let text = serde_json::to_string(&v).map_err(|e| e.to_string())?;
        Ok((text, keys))
    }

    /// Brings the disk in step with `session`. `budget_ms` bounds the time spent encoding and writing pictures (`0` = no bound,
    /// for the flush at exit). Cheap when nothing changed: one record serialisation and a strided scan of the pictures.
    pub fn sync(&mut self, session: &mut AssetLibrarySession, budget_ms: u64) -> SyncReport {
        let mut rep = SyncReport::default();
        let (probe_text, _) = match Self::build_index(session, false) {
            Ok(x) => x,
            Err(e) => {
                rep.error = Some(e);
                return rep;
            }
        };
        let fp = Self::fingerprint(&probe_text, session);
        if self.last_fp == Some(fp) {
            rep.unchanged = true;
            return rep;
        }
        let (text, keys) = match Self::build_index(session, true) {
            Ok(x) => x,
            Err(e) => {
                rep.error = Some(e);
                return rep;
            }
        };
        let started = Instant::now();
        let mut done: HashSet<&str> = HashSet::new();
        for (key, uid, i) in &keys {
            if self.on_disk.contains(key) || done.contains(key.as_str()) {
                continue;
            }
            if budget_ms > 0 && started.elapsed().as_millis() as u64 >= budget_ms {
                rep.pending = true;
                return rep;
            }
            let Some(img) = session.image(uid, *i) else { continue };
            let png = match encode_png(img) {
                Ok(p) => p,
                Err(e) => {
                    rep.error = Some(format!("{key}: {e}"));
                    return rep;
                }
            };
            if let Err(e) = write_atomic(&self.img_path(key), &png) {
                rep.error = Some(format!("{key}.png: {e}"));
                return rep;
            }
            self.on_disk.insert(key.clone());
            done.insert(key.as_str());
            rep.written += 1;
        }
        if self.last_index.as_deref() != Some(text.as_str()) {
            if let Err(e) = write_atomic(&self.index_path(), text.as_bytes()) {
                rep.error = Some(format!("index.json: {e}"));
                return rep;
            }
            rep.index_written = true;
            self.last_index = Some(text);
        }
        if self.gc_enabled {
            let live: HashSet<&str> = keys.iter().map(|(k, _, _)| k.as_str()).collect();
            if let Ok(rd) = fs::read_dir(self.img_dir()) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.extension().is_some_and(|x| x == "png")
                        && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
                        && !live.contains(stem)
                        && fs::remove_file(&p).is_ok()
                    {
                        rep.removed += 1;
                        self.on_disk.remove(stem);
                    }
                }
            }
        }
        self.last_fp = Some(fp);
        rep
    }

    /// Bytes the store occupies on disk (the index and every picture file, corrupt ones aside included), for the library
    /// window's readout.
    pub fn size_bytes(&self) -> u64 {
        let mut total = 0u64;
        for d in [self.dir.clone(), self.img_dir()] {
            let Ok(rd) = fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                if let Ok(m) = e.metadata()
                    && m.is_file()
                {
                    total += m.len();
                }
            }
        }
        total
    }
}

#[cfg(test)]
#[path = "asset_store_tests.rs"]
mod tests;
