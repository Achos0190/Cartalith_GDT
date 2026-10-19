//! Importing a **folder of art** into the Asset Library
//! (`ASSET_PACK_TOOLS_PROPOSAL.md` AP-8, Ruling CQ Q10): a loose folder of
//! PNGs first, then the two published layouts the proposal names, a
//! Wonderdraft asset folder and an *unpacked* Dungeondraft asset tree.
//!
//! The reference has no folder import (no `webkitdirectory` anywhere in
//! `Cartalith Gen1 v2.11.html`), so nothing here is a port; it is new scope
//! under Ruling CQ, and it writes only what the library already has fields
//! for (custom sets and slots, frozen ground slots, `SlotMeta::license`,
//! `SlotMeta::tags`, `SlotMeta::source`). The pack format (Ruling F) and the
//! save format are untouched.
//!
//! # The three halves
//!
//! - [`list_folder`] walks the chosen folder and records every file's
//!   relative path and size **without opening any file**. Links are never
//!   followed (on Windows every reparse point, junctions included), the walk
//!   is depth-capped, and the entry count is capped by AP-1's
//!   [`ImportLimits::max_entries`].
//! - [`detect_layout`] decides which of the three layouts the listing is,
//!   from names alone.
//! - [`plan_folder_import`] says which file lands in which library slot and
//!   reads **exactly** the files that plan uses (the PNGs it imports, a
//!   licence text, Dungeondraft's tag file), each through
//!   [`FolderListing::read`], under AP-1's per-entry and total caps.
//!
//! Decoding and the store writes are `cartalith-godot`'s
//! `AssetLibrarySession::import_folder`, for the same reason AP-1's are: the
//! colour-managed decode lives in that crate.
//!
//! # What must never happen here
//!
//! **Nothing outside the chosen folder may be read.** The listing only ever
//! descends through real directories (a link is refused, not followed), and
//! every read re-resolves its path and refuses one that no longer lies under
//! the folder's own canonical path, so a link swapped in after the listing
//! still cannot reach outside. A relative path that is absolute, names a
//! drive or climbs with `..` is refused before it is joined
//! ([`crate::unsafe_path_reason`], AP-1's own rule).
//!
//! **No third-party art is bundled, fetched or redistributed**
//! (`ASSET_PACK_TOOLS_PROPOSAL.md` §5): this reads files the user already has,
//! and every test builds its own synthetic tree.
//!
//! # The two named layouts, and how sure this is of them
//!
//! - **Dungeondraft (unpacked).** The proposal's §3 fetched Megasploot's
//!   *Custom Assets Guide*: art under `textures/objects/` (subfolders
//!   allowed), seamless ground under `textures/terrain/`, other `textures/`
//!   folders (walls, paths, portals, lights, materials, tilesets, patterns)
//!   with no Cartalith family, and `data/default.dungeondraft_tags` holding
//!   `"tags"` (tag name to asset paths) and `"sets"`. Objects become custom
//!   sets, terrain becomes ground candidates, tags become library tags.
//!   A packed `.dungeondraft_pack` is not read (§3: its container is
//!   unverified).
//! - **Wonderdraft.** The proposal's §3 had only a **search summary** of the
//!   layout: a package folder with `sprites/` (symbols, mountains, trees, in
//!   folders) and `textures/`, plus a hidden `.wonderdraft_symbols` JSON per
//!   sprite folder. Because that JSON's shape was never fetched, it is **not
//!   parsed**: display names come from file names, and the file is counted
//!   as not imported. Sprite folders become custom sets; `textures/` PNGs
//!   become ground candidates.
//! - **Loose.** Anything else: every PNG, its folder as the custom set.

use crate::archive::PackEntries;
use crate::pack_import::{ImportLimits, ImportRefusal, PlannedPath, SlotTarget, unsafe_path_reason};
use crate::slots::{Family, slug_id};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// How many folders deep the walk goes below the chosen folder. **A labelled
/// judgement**: Dungeondraft's own deepest documented path is
/// `textures/objects/<set>/<subset>/x.png` (four levels), so sixteen is four
/// times that, and it bounds a walk over a pathological tree.
pub const FOLDER_MAX_DEPTH: usize = 16;

/// The largest licence text read. **A labelled judgement**: a long licence
/// (the full GPL v3) is about 35 KB; 64 KiB keeps any real one and refuses a
/// file that is not a licence.
pub const LICENCE_MAX_BYTES: u64 = 64 * 1024;

/// The custom set a ground image goes to when no ground slot carries its
/// name. The proposal (§3) calls these "candidates for ground slots, picked
/// by the user"; the library has no move-to-slot verb, so a candidate whose
/// name is a slot id goes straight to that slot and the rest wait here.
pub const GROUND_CANDIDATES_SET: &str = "Ground candidates";

/// The set name for images directly in the chosen folder when the folder's
/// own name is empty (a drive root). Never otherwise used.
pub const FALLBACK_SET: &str = "Imported folder";

/// Root-level file names (compared lowercased) read as the folder's licence
/// text, in the order they are preferred. **A labelled judgement**: the names
/// open-source and asset-store packs conventionally use.
pub const LICENCE_NAMES: [&str; 8] =
    ["license", "license.txt", "license.md", "licence", "licence.txt", "licence.md", "copying", "copying.txt"];

/// Dungeondraft's tag file, relative to the pack root (Megasploot's guide).
pub const DUNGEONDRAFT_TAGS: &str = "data/default.dungeondraft_tags";

/// One file the walk saw. Nothing about its content is known yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedFile {
    /// Path below the chosen folder, `/`-separated, as the file system spells
    /// it.
    pub rel: String,
    /// Size from the directory entry. A claim until the bounded read.
    pub len: u64,
}

/// The chosen folder, walked but not read.
#[derive(Debug, Clone)]
pub struct FolderListing {
    /// The folder's canonical path: every read must resolve under it.
    root: PathBuf,
    /// The folder's own name, used as the set name for images directly in
    /// it ([`FALLBACK_SET`] when it has none).
    pub root_name: String,
    /// Every regular file seen, sorted by `rel`.
    pub files: Vec<ListedFile>,
    /// Entries seen and deliberately not walked or listed, with the reason:
    /// links, folders past [`FOLDER_MAX_DEPTH`], names that are not Unicode
    /// or carry a control character.
    pub refused: Vec<(String, String)>,
}

/// Whether a directory entry is a link of any kind and must not be followed.
/// `FileType::is_symlink` covers symbolic links on every platform and, on
/// Windows, name-surrogate reparse points (symlinks and junctions); the
/// attribute check also refuses every other reparse point there, because a
/// reparse point can redirect a read elsewhere.
fn is_link(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        /// `FILE_ATTRIBUTE_REPARSE_POINT` (Win32 `winnt.h`).
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}

/// Walk `root` and list every regular file below it, opening none of them.
///
/// Refuses the whole folder ([`ImportRefusal::FolderUnreadable`]) when `root`
/// is not a readable directory, and ([`ImportRefusal::TooManyFiles`]) when it
/// holds more than `limits.max_entries` files and folders together, so a
/// walk over a huge tree stops at the cap rather than at the end. A link, a
/// folder past [`FOLDER_MAX_DEPTH`] or an unusable name refuses only that
/// entry, with a reason in [`FolderListing::refused`].
///
/// Deterministic: each folder's entries are visited in byte order of their
/// names, so two walks of the same tree give the same listing.
pub fn list_folder(root: &Path, limits: &ImportLimits) -> Result<FolderListing, ImportRefusal> {
    let unreadable = |why: String| ImportRefusal::FolderUnreadable(why);
    let canon = fs::canonicalize(root).map_err(|e| unreadable(format!("{}: {e}", root.display())))?;
    let meta = fs::metadata(&canon).map_err(|e| unreadable(format!("{}: {e}", root.display())))?;
    if !meta.is_dir() {
        return Err(unreadable(format!("{} is not a folder", root.display())));
    }
    let root_name = canon
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| FALLBACK_SET.to_string());
    let mut listing = FolderListing { root: canon.clone(), root_name, files: Vec::new(), refused: Vec::new() };
    let mut seen: usize = 0;
    // (absolute dir, relative prefix ending in '/' or empty, depth)
    let mut stack: Vec<(PathBuf, String, usize)> = vec![(canon, String::new(), 0)];
    while let Some((dir, prefix, depth)) = stack.pop() {
        let rd = match fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                // The root itself was readable above; a subfolder that is
                // not is reported and skipped, not fatal.
                listing.refused.push((prefix.trim_end_matches('/').to_string(), format!("could not be listed ({e})")));
                continue;
            }
        };
        let mut entries: Vec<(String, PathBuf)> = Vec::new();
        for ent in rd {
            let Ok(ent) = ent else { continue };
            let raw = ent.file_name();
            let shown = raw.to_string_lossy().into_owned();
            let Some(name) = raw.to_str() else {
                listing.refused.push((format!("{prefix}{shown}"), "has a name that is not valid Unicode".to_string()));
                continue;
            };
            entries.push((name.to_string(), ent.path()));
        }
        entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut subdirs = Vec::new();
        for (name, path) in entries {
            seen += 1;
            if seen > limits.max_entries {
                return Err(ImportRefusal::TooManyFiles { cap: limits.max_entries });
            }
            let rel = format!("{prefix}{name}");
            if let Some(why) = unsafe_path_reason(&rel) {
                listing.refused.push((rel, why.to_string()));
                continue;
            }
            let Ok(meta) = fs::symlink_metadata(&path) else {
                listing.refused.push((rel, "could not be read".to_string()));
                continue;
            };
            if is_link(&meta) {
                listing
                    .refused
                    .push((rel, "is a link; links are not followed, so nothing outside the folder is read".to_string()));
                continue;
            }
            if meta.is_dir() {
                if depth + 1 > FOLDER_MAX_DEPTH {
                    listing.refused.push((rel, format!("is more than {FOLDER_MAX_DEPTH} folders deep; not read")));
                } else {
                    subdirs.push((path, format!("{rel}/"), depth + 1));
                }
            } else if meta.is_file() {
                listing.files.push(ListedFile { rel, len: meta.len() });
            }
        }
        // Pushed in reverse so the stack pops them in name order.
        for d in subdirs.into_iter().rev() {
            stack.push(d);
        }
    }
    listing.files.sort_by(|a, b| a.rel.as_bytes().cmp(b.rel.as_bytes()));
    Ok(listing)
}

impl FolderListing {
    /// Read one listed file, at most `cap` bytes.
    ///
    /// Refuses, in words, a path that is not relative and plain, a file that
    /// is now a link, one that resolves outside the folder (a link swapped in
    /// after the listing), and one larger than `cap`, whether its directory
    /// entry said so or the bounded read found it.
    pub fn read(&self, rel: &str, cap: u64) -> Result<Vec<u8>, String> {
        if let Some(why) = unsafe_path_reason(rel) {
            return Err(format!("refused: the path {why}"));
        }
        let mut path = self.root.clone();
        for seg in rel.split('/') {
            path.push(seg);
        }
        let meta = fs::symlink_metadata(&path).map_err(|e| format!("could not be read ({e})"))?;
        if is_link(&meta) {
            return Err("refused: it is a link".to_string());
        }
        let canon = fs::canonicalize(&path).map_err(|e| format!("could not be read ({e})"))?;
        if !canon.starts_with(&self.root) {
            return Err("refused: it resolves outside the chosen folder".to_string());
        }
        if meta.len() > cap {
            return Err(format!("refused: {} bytes, over the {cap}-byte limit", meta.len()));
        }
        let f = fs::File::open(&canon).map_err(|e| format!("could not be opened ({e})"))?;
        let mut buf = Vec::new();
        f.take(cap + 1).read_to_end(&mut buf).map_err(|e| format!("could not be read ({e})"))?;
        if buf.len() as u64 > cap {
            return Err(format!("refused: more than {cap} bytes"));
        }
        Ok(buf)
    }
}

/// Which layout a folder is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderLayout {
    /// Any folder of PNGs.
    Loose,
    /// A Wonderdraft asset folder (`sprites/`, `textures/`).
    Wonderdraft,
    /// An unpacked Dungeondraft asset tree (`textures/objects/`,
    /// `textures/terrain/`, `data/default.dungeondraft_tags`).
    Dungeondraft,
}

impl FolderLayout {
    /// The lowercase key the shell receives.
    pub fn key(self) -> &'static str {
        match self {
            FolderLayout::Loose => "loose",
            FolderLayout::Wonderdraft => "wonderdraft",
            FolderLayout::Dungeondraft => "dungeondraft",
        }
    }

    /// The words the import card shows.
    pub fn label(self) -> &'static str {
        match self {
            FolderLayout::Loose => "a folder of images",
            FolderLayout::Wonderdraft => "a Wonderdraft asset folder",
            FolderLayout::Dungeondraft => "an unpacked Dungeondraft asset folder",
        }
    }
}

/// Decide the layout from the listing's names alone (case-insensitive).
///
/// Dungeondraft is checked first because both layouts can have a `textures/`
/// folder and only Dungeondraft nests `objects/` or `terrain/` in it, or
/// carries its tag file. Wonderdraft is a `sprites/` folder at the root.
/// Anything else is loose.
pub fn detect_layout(listing: &FolderListing) -> FolderLayout {
    let lower: Vec<String> = listing.files.iter().map(|f| f.rel.to_lowercase()).collect();
    if lower
        .iter()
        .any(|r| r.starts_with("textures/objects/") || r.starts_with("textures/terrain/") || r == DUNGEONDRAFT_TAGS)
    {
        return FolderLayout::Dungeondraft;
    }
    if lower.iter().any(|r| r.starts_with("sprites/")) {
        return FolderLayout::Wonderdraft;
    }
    FolderLayout::Loose
}

/// The licence text found at the folder's root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenceText {
    /// The file it was read from, relative.
    pub path: String,
    /// Its text, trimmed, decoded as UTF-8 with replacement characters for
    /// anything that is not.
    pub text: String,
}

/// One library slot a folder import fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderSlotPlan {
    /// Where in the layout the art came from: `"loose"`, `"sprites"`
    /// (Wonderdraft), `"objects"` or `"terrain"` (Dungeondraft), `"textures"`
    /// (Wonderdraft ground). The report's section column.
    pub area: String,
    /// The library slot. A custom target's `id` is the slot's display name
    /// (`AssetDB::add_custom_slot` slugs it), exactly as AP-1's plan does.
    pub target: SlotTarget,
    /// The display name the row shows.
    pub label: String,
    /// The files, in name order. A refusal here is per file.
    pub paths: Vec<PlannedPath>,
    /// One item name per path: the file's stem.
    pub item_names: Vec<String>,
    /// Library tags (Dungeondraft's tag file), in the file's order.
    pub tags: Vec<String>,
    /// Why this slot landed where it did, when that is not obvious (a ground
    /// image with no slot of its name).
    pub note: Option<String>,
}

/// A whole folder, resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderPlan {
    /// The layout it was read as.
    pub layout: FolderLayout,
    /// The folder's own name.
    pub root_name: String,
    /// Every slot, in the order they are filled.
    pub slots: Vec<FolderSlotPlan>,
    /// The licence text, when the folder has one at its root.
    pub licence: Option<LicenceText>,
    /// What was not imported and why, in words, one line per kind.
    pub notes: Vec<String>,
}

/// The variant group of a file stem: the stem with one trailing run of
/// digits removed, together with a single `_`, `-`, `.` or space before it,
/// so `tree_01`, `tree-2` and `tree 3` are three variants of `tree`.
/// **A labelled judgement** about how packs number their variants; a stem
/// that is only digits keeps its digits.
pub fn variant_base(stem: &str) -> &str {
    let trimmed = stem.trim_end_matches(|c: char| c.is_ascii_digit());
    if trimmed.len() == stem.len() {
        return stem;
    }
    let base = trimmed.strip_suffix(['_', '-', '.', ' ']).unwrap_or(trimmed);
    if base.trim().is_empty() { stem } else { base }
}

/// `(directory, file stem)` of a relative path; the directory is `""` for a
/// file at the root of `rel`.
fn split_rel(rel: &str) -> (&str, &str) {
    let (dir, file) = match rel.rfind('/') {
        Some(i) => (&rel[..i], &rel[i + 1..]),
        None => ("", rel),
    };
    let stem = match file.rfind('.') {
        Some(i) if i > 0 => &file[..i],
        _ => file,
    };
    (dir, stem)
}

/// Whether `rel` names a PNG by its extension (case-insensitive).
fn is_png(rel: &str) -> bool {
    rel.to_lowercase().ends_with(".png")
}

/// The custom set name for images in `dir` (relative to the area root): the
/// folder names joined with ` / `, or `root_name` for the area root itself.
fn set_name(dir: &str, root_name: &str) -> String {
    if dir.is_empty() { root_name.to_string() } else { dir.split('/').collect::<Vec<_>>().join(" / ") }
}

/// Dungeondraft's tag file: tag name to the asset paths it holds, in the
/// file's order. Tolerant: anything that is not the documented shape is
/// skipped. Paths are normalised to `/` separators with any leading `./`
/// or `res://` removed, compared lowercased.
fn parse_dungeondraft_tags(bytes: &[u8]) -> Result<(Vec<(String, HashSet<String>)>, bool), String> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("{DUNGEONDRAFT_TAGS} is not valid JSON ({e})"))?;
    let mut out = Vec::new();
    if let Some(tags) = v.get("tags").and_then(|t| t.as_object()) {
        for (tag, paths) in tags {
            let set: HashSet<String> = paths
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|p| p.as_str())
                        .map(|p| {
                            let p = p.replace('\\', "/");
                            let p = p.strip_prefix("res://").unwrap_or(&p);
                            p.strip_prefix("./").unwrap_or(p).to_lowercase()
                        })
                        .collect()
                })
                .unwrap_or_default();
            out.push((tag.clone(), set));
        }
    }
    let has_sets = v.get("sets").and_then(|s| s.as_object()).is_some_and(|s| !s.is_empty());
    Ok((out, has_sets))
}

/// Plan a folder import and read exactly the files the plan uses.
///
/// `is_frozen(family, id)` answers whether the library has a frozen slot
/// `id` in `family` (the session passes `AssetDB`'s own registry). It is
/// asked only about the three ground families, for ground candidates.
///
/// Returns the plan and the bytes of every file it will import (keyed by
/// relative path), or a whole-folder refusal when the bytes read together
/// pass [`ImportLimits::max_total_bytes`]. A file the read refuses becomes a
/// refused path in its slot, so the report names it.
///
/// Order: slots in name order of their folder, then of their group; ground
/// candidates after sprites. Within a slot, files in name order.
pub fn plan_folder_import(
    listing: &FolderListing,
    limits: &ImportLimits,
    is_frozen: impl Fn(Family, &str) -> bool,
) -> Result<(FolderPlan, PackEntries), ImportRefusal> {
    let layout = detect_layout(listing);
    let mut plan = FolderPlan {
        layout,
        root_name: listing.root_name.clone(),
        slots: Vec::new(),
        licence: None,
        notes: Vec::new(),
    };
    let mut entries = PackEntries::new();
    let mut total: u64 = 0;
    let mut read_counted = |rel: &str, cap: u64, entries: &mut PackEntries| -> Result<Result<(), String>, ImportRefusal> {
        match listing.read(rel, cap) {
            Ok(bytes) => {
                total += bytes.len() as u64;
                if total > limits.max_total_bytes {
                    return Err(ImportRefusal::TotalTooLarge { cap: limits.max_total_bytes });
                }
                entries.insert(rel.to_string(), bytes);
                Ok(Ok(()))
            }
            Err(why) => Ok(Err(why)),
        }
    };

    for (rel, why) in &listing.refused {
        plan.notes.push(format!("skipped {rel}: {why}"));
    }

    // Licence: the first root-level file whose lowercased name is in
    // LICENCE_NAMES, in that list's order of preference.
    for want in LICENCE_NAMES {
        if let Some(f) = listing.files.iter().find(|f| !f.rel.contains('/') && f.rel.to_lowercase() == want) {
            match listing.read(&f.rel, LICENCE_MAX_BYTES) {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).trim().to_string();
                    if !text.is_empty() {
                        plan.licence = Some(LicenceText { path: f.rel.clone(), text });
                    }
                }
                Err(why) => plan.notes.push(format!("licence {} not read: {why}", f.rel)),
            }
            break;
        }
    }

    // Dungeondraft tags, read before the slots so each slot can carry them.
    let mut dd_tags: Vec<(String, HashSet<String>)> = Vec::new();
    if layout == FolderLayout::Dungeondraft
        && let Some(f) = listing.files.iter().find(|f| f.rel.to_lowercase() == DUNGEONDRAFT_TAGS)
    {
        match listing.read(&f.rel, limits.max_manifest_bytes).map_err(|e| format!("{}: {e}", f.rel)) {
            Ok(bytes) => match parse_dungeondraft_tags(&bytes) {
                Ok((tags, has_sets)) => {
                    dd_tags = tags;
                    if has_sets {
                        plan.notes.push(
                            "the tag file's \"sets\" (groups of tags) were not imported: the library has tags, not tag groups"
                                .to_string(),
                        );
                    }
                }
                Err(why) => plan.notes.push(why),
            },
            Err(why) => plan.notes.push(why),
        }
    }

    // Which listed PNGs go where. `area_root` is the lowercase prefix the
    // area's set names are taken relative to.
    let lower = |s: &str| s.to_lowercase();
    let (sprite_areas, ground_areas): (Vec<(&str, &str)>, Vec<(&str, &str)>) = match layout {
        FolderLayout::Loose => (vec![("loose", "")], vec![]),
        FolderLayout::Wonderdraft => (vec![("sprites", "sprites/")], vec![("textures", "textures/")]),
        FolderLayout::Dungeondraft => (vec![("objects", "textures/objects/")], vec![("terrain", "textures/terrain/")]),
    };
    let in_area = |rel: &str, root: &str| root.is_empty() || lower(rel).starts_with(root);

    let mut used: HashSet<String> = HashSet::new();
    // Sprites: grouped by (folder, variant base).
    for (area, root) in &sprite_areas {
        let mut groups: BTreeMap<(String, String), (String, Vec<&ListedFile>)> = BTreeMap::new();
        for f in listing.files.iter().filter(|f| is_png(&f.rel) && in_area(&f.rel, root)) {
            let below = &f.rel[root.len()..];
            let (dir, stem) = split_rel(below);
            let base = variant_base(stem);
            let key = (dir.to_string(), slug_id(base));
            groups.entry(key).or_insert_with(|| (base.to_string(), Vec::new())).1.push(f);
        }
        for ((dir, _), (label, files)) in groups {
            let set = set_name(&dir, &listing.root_name);
            let mut slot = FolderSlotPlan {
                area: area.to_string(),
                target: SlotTarget::Custom { id: label.clone(), set },
                label,
                paths: Vec::new(),
                item_names: Vec::new(),
                tags: Vec::new(),
                note: None,
            };
            for f in files {
                used.insert(f.rel.clone());
                let refusal = read_counted(&f.rel, limits.max_entry_bytes, &mut entries)?.err();
                slot.paths.push(PlannedPath { path: f.rel.clone(), refusal });
                slot.item_names.push(split_rel(&f.rel).1.to_string());
                for (tag, paths) in &dd_tags {
                    if paths.contains(&lower(&f.rel)) && !slot.tags.contains(tag) {
                        slot.tags.push(tag.clone());
                    }
                }
            }
            plan.slots.push(slot);
        }
    }

    // Ground: one slot per image; a frozen ground slot of the same name
    // takes it (first come, first served), otherwise the candidates set.
    let mut claimed: HashSet<(Family, String)> = HashSet::new();
    for (area, root) in &ground_areas {
        for f in listing.files.iter().filter(|f| is_png(&f.rel) && in_area(&f.rel, root)) {
            used.insert(f.rel.clone());
            let stem = split_rel(&f.rel).1.to_string();
            let id = slug_id(&stem);
            let frozen = [Family::Textures, Family::Biomes, Family::Terrains]
                .into_iter()
                .find(|fam| is_frozen(*fam, &id) && !claimed.contains(&(*fam, id.clone())));
            let (target, note) = match frozen {
                Some(family) => {
                    claimed.insert((family, id.clone()));
                    (SlotTarget::Frozen { family, id: id.clone() }, None)
                }
                None => (
                    SlotTarget::Custom { id: stem.clone(), set: GROUND_CANDIDATES_SET.to_string() },
                    Some(format!(
                        "no ground slot is named \"{id}\" (or it was already taken), so {} is in the custom set {GROUND_CANDIDATES_SET}",
                        f.rel
                    )),
                ),
            };
            let refusal = read_counted(&f.rel, limits.max_entry_bytes, &mut entries)?.err();
            let mut tags = Vec::new();
            for (tag, paths) in &dd_tags {
                if paths.contains(&lower(&f.rel)) {
                    tags.push(tag.clone());
                }
            }
            plan.slots.push(FolderSlotPlan {
                area: area.to_string(),
                target,
                label: stem.clone(),
                paths: vec![PlannedPath { path: f.rel.clone(), refusal }],
                item_names: vec![stem],
                tags,
                note,
            });
        }
    }

    // What was left out, counted by kind, never read.
    let licence_path = plan.licence.as_ref().map(|l| l.path.clone());
    let mut other_png = 0usize;
    let mut not_png = 0usize;
    for f in &listing.files {
        if used.contains(&f.rel) || Some(&f.rel) == licence_path.as_ref() || lower(&f.rel) == DUNGEONDRAFT_TAGS {
            continue;
        }
        if is_png(&f.rel) { other_png += 1 } else { not_png += 1 }
    }
    if other_png > 0 {
        // A loose folder imports every PNG, so only the two named layouts
        // can leave one out.
        let why = match layout {
            FolderLayout::Loose => "",
            FolderLayout::Wonderdraft => {
                " outside sprites/ and textures/ (brushes, themes and the like have no library family)"
            }
            FolderLayout::Dungeondraft => {
                " outside textures/objects/ and textures/terrain/ (walls, paths, portals, lights, materials, tilesets, patterns and the preview have no library family)"
            }
        };
        plan.notes.push(format!("{other_png} image{}{why} not imported", if other_png == 1 { " was" } else { "s were" }));
    }
    if not_png > 0 {
        plan.notes.push(format!(
            "{not_png} {} not read{}",
            if not_png == 1 { "file that is not a PNG image was" } else { "files that are not PNG images were" },
            if layout == FolderLayout::Wonderdraft {
                " (Wonderdraft's .wonderdraft_symbols files among them: their format is unverified, so names come from file names)"
            } else {
                ""
            }
        ));
    }
    Ok((plan, entries))
}

/// The walk's refusals, the layout detection, the planning of each layout
/// and the caps, on trees made for the test (no third-party art).
#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty directory under the system temp dir, removed on drop.
    struct TempTree(PathBuf);
    impl TempTree {
        fn new(tag: &str) -> TempTree {
            let p = std::env::temp_dir().join(format!("cartalith_ap8_{tag}_{}", std::process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            TempTree(p)
        }
        fn file(&self, rel: &str, bytes: &[u8]) {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, bytes).unwrap();
        }
    }
    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Ground-family frozen vocabulary, as the session passes it.
    fn frozen(f: Family, id: &str) -> bool {
        f.has_slot(id)
    }

    /// Protects: the variant grouping rule's literal cases.
    #[test]
    fn variant_base_strips_one_numbered_suffix() {
        assert_eq!(variant_base("tree_01"), "tree");
        assert_eq!(variant_base("tree-2"), "tree");
        assert_eq!(variant_base("Pine Tree 3"), "Pine Tree");
        assert_eq!(variant_base("castle2"), "castle");
        assert_eq!(variant_base("tower"), "tower");
        assert_eq!(variant_base("2"), "2", "a digits-only stem keeps its digits");
        assert_eq!(variant_base("_7"), "_7", "nothing left but a separator keeps the stem");
    }

    /// Protects: the caps' literal figures.
    #[test]
    fn folder_caps_are_the_documented_figures() {
        assert_eq!(FOLDER_MAX_DEPTH, 16);
        assert_eq!(LICENCE_MAX_BYTES, 65_536);
        assert_eq!(GROUND_CANDIDATES_SET, "Ground candidates");
    }

    /// Protects: layout detection from names, Dungeondraft before
    /// Wonderdraft, loose otherwise.
    #[test]
    fn layouts_are_detected_from_names() {
        let t = TempTree::new("detect");
        t.file("a.png", b"x");
        assert_eq!(detect_layout(&list_folder(&t.0, &ImportLimits::DEFAULT).unwrap()), FolderLayout::Loose);
        t.file("sprites/trees/oak.png", b"x");
        assert_eq!(detect_layout(&list_folder(&t.0, &ImportLimits::DEFAULT).unwrap()), FolderLayout::Wonderdraft);
        t.file("textures/terrain/grass.png", b"x");
        assert_eq!(detect_layout(&list_folder(&t.0, &ImportLimits::DEFAULT).unwrap()), FolderLayout::Dungeondraft);
    }

    /// Protects: a link inside the folder is refused, not followed, so a file
    /// outside the folder is never listed or read. Skips (with a printed
    /// reason) only when this machine cannot create any kind of link.
    #[test]
    fn links_are_refused_and_nothing_outside_is_listed() {
        let outside = TempTree::new("outside");
        outside.file("secret.png", b"outside");
        let t = TempTree::new("links");
        t.file("inside.png", b"inside");
        let made = make_dir_link(&outside.0, &t.0.join("escape"));
        let listing = list_folder(&t.0, &ImportLimits::DEFAULT).unwrap();
        let rels: Vec<&str> = listing.files.iter().map(|f| f.rel.as_str()).collect();
        assert_eq!(rels, vec!["inside.png"]);
        if !made {
            eprintln!("no directory link could be created on this machine; the refusal branch was not exercised");
            return;
        }
        assert_eq!(listing.refused.len(), 1, "{:?}", listing.refused);
        assert_eq!(listing.refused[0].0, "escape");
        assert!(listing.refused[0].1.contains("link"));
        let (plan, entries) = plan_folder_import(&listing, &ImportLimits::DEFAULT, frozen).unwrap();
        assert_eq!(entries.keys().collect::<Vec<_>>(), vec!["inside.png"]);
        assert!(plan.notes.iter().any(|n| n.starts_with("skipped escape:")), "{:?}", plan.notes);
        // A read through the link's name is refused too.
        assert!(listing.read("escape/secret.png", 1024).is_err());
    }

    /// A directory link at `link` pointing at `target`: a symbolic link
    /// where the OS allows one, else (Windows without the privilege) a
    /// junction made with `mklink /J`. False when neither could be made.
    fn make_dir_link(target: &Path, link: &Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link).is_ok()
        }
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_dir(target, link).is_ok() {
                return true;
            }
            std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Protects: every read that would leave the folder is refused in words.
    #[test]
    fn reads_that_climb_out_are_refused() {
        let t = TempTree::new("climb");
        t.file("a.png", b"x");
        let listing = list_folder(&t.0, &ImportLimits::DEFAULT).unwrap();
        for bad in ["../a.png", "/etc/passwd", "C:/Windows/win.ini", "x/../../a.png"] {
            let e = listing.read(bad, 1024).unwrap_err();
            assert!(e.starts_with("refused"), "{bad}: {e}");
        }
        assert_eq!(listing.read("a.png", 1024).unwrap(), b"x");
        assert!(listing.read("a.png", 0).unwrap_err().contains("limit"));
    }

    /// Protects: the entry-count cap stops the walk, the depth cap refuses a
    /// folder past it, and the total-bytes cap refuses the whole import.
    #[test]
    fn the_caps_fire() {
        let t = TempTree::new("caps");
        for i in 0..6 {
            t.file(&format!("f{i}.png"), &[0u8; 100]);
        }
        let few = ImportLimits { max_entries: 5, ..ImportLimits::DEFAULT };
        assert_eq!(list_folder(&t.0, &few).unwrap_err(), ImportRefusal::TooManyFiles { cap: 5 });
        let listing = list_folder(&t.0, &ImportLimits::DEFAULT).unwrap();
        let tight = ImportLimits { max_total_bytes: 450, ..ImportLimits::DEFAULT };
        assert_eq!(
            plan_folder_import(&listing, &tight, frozen).unwrap_err(),
            ImportRefusal::TotalTooLarge { cap: 450 }
        );
        let small = ImportLimits { max_entry_bytes: 50, ..ImportLimits::DEFAULT };
        let (plan, entries) = plan_folder_import(&listing, &small, frozen).unwrap();
        assert!(entries.is_empty());
        assert!(plan.slots.iter().all(|s| s.paths.iter().all(|p| p.refusal.as_deref().is_some_and(|r| r.contains("limit")))));

        let deep = TempTree::new("deep");
        let mut rel = String::new();
        for i in 0..=FOLDER_MAX_DEPTH {
            rel.push_str(&format!("d{i}/"));
        }
        deep.file(&format!("{rel}x.png"), b"x");
        deep.file("top.png", b"x");
        let listing = list_folder(&deep.0, &ImportLimits::DEFAULT).unwrap();
        assert_eq!(listing.files.len(), 1, "only the top image is listed");
        assert_eq!(listing.refused.len(), 1);
        assert!(listing.refused[0].1.contains("16 folders deep"));
        // The folder refused is the 17th level, not the 16th: the cap is
        // "more than 16 deep", so a 16-deep image would still be listed.
        assert!(listing.refused[0].0.ends_with("/d16"), "{}", listing.refused[0].0);
    }

    /// Protects: a ground slot is claimed once. Two images whose names slug
    /// to the same ground slot id: the first (by name) fills it, the second
    /// goes to the candidates set with a note, never to the same slot.
    #[test]
    fn a_ground_slot_is_claimed_once() {
        let t = TempTree::new("claim");
        t.file("sprites/a/x.png", b"x");
        t.file("textures/grass.png", b"x");
        t.file("textures/grass_.png", b"x");
        let listing = list_folder(&t.0, &ImportLimits::DEFAULT).unwrap();
        let (plan, _) = plan_folder_import(&listing, &ImportLimits::DEFAULT, frozen).unwrap();
        let ground: Vec<&SlotTarget> = plan.slots.iter().filter(|s| s.area == "textures").map(|s| &s.target).collect();
        assert_eq!(
            ground,
            vec![
                &SlotTarget::Frozen { family: Family::Textures, id: "grass".into() },
                &SlotTarget::Custom { id: "grass_".into(), set: GROUND_CANDIDATES_SET.into() },
            ]
        );
    }

    /// Protects: a path that is not a folder is refused for the whole import.
    #[test]
    fn a_file_or_missing_path_is_not_a_folder() {
        let t = TempTree::new("notdir");
        t.file("a.png", b"x");
        assert!(matches!(list_folder(&t.0.join("a.png"), &ImportLimits::DEFAULT), Err(ImportRefusal::FolderUnreadable(_))));
        assert!(matches!(list_folder(&t.0.join("nope"), &ImportLimits::DEFAULT), Err(ImportRefusal::FolderUnreadable(_))));
    }
}
