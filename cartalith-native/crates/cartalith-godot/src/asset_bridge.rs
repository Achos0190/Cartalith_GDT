//! The Asset Library's mutable, in-memory session: `GUI_GAP_REGISTER.md`
//! rows AS-01 through AS-08/AS-13 and DM-05 — every one of them a real
//! `cartalith-assets` function with no `#[func]` in front of it (verified by
//! reading `library.rs`/`raster.rs`/`archive.rs` directly, the same "read
//! before wiring" discipline `travel_bridge.rs` and `pack.rs` both document).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `travel_bridge.rs`/`journey_bridge.rs`/`timeline_bridge.rs` already
//! establish: this module owns everything expressible without one, and
//! `lib.rs`'s `#[godot_api(secondary)]` block owns the thin `Variant`<->Rust
//! conversion.
//!
//! # Why a parallel pixel store, not `AssetDB` alone
//!
//! `cartalith_assets::AssetDB` (`library.rs`) is deliberately pixel-free —
//! its own module doc says so: `LibraryItem` carries a `hash: String` but no
//! `img`/`w`/`h`. Every real operation this session needs (thumbnails, a
//! pack export's baked slot image, `Apply to map`) needs the actual decoded
//! pixels behind an item, so [`AssetLibrarySession`] keeps a second store,
//! [`AssetLibrarySession::images`], index-parallel to `AssetDB`'s own
//! `store[uid]` — same uid, same order, same length, always. Every method
//! below that mutates one mutates the other in the same call, so the two
//! never drift.
//!
//! # What this session does and does not restore
//!
//! This is the *authoring* session — `AssetImporter.intake`/`AssetBrowserUI`'s
//! batch operations/`PackManifestBuilder.build`/`applyToMap`/`exportPack` in
//! reference terms. It does not read a saved `assetlib/library.json` project
//! section back in (`AssetDB::apply_library_file_with_items`, a different,
//! save-format-adjacent feature `SAVEFILE_COMPAT.md` territory owns and this
//! dispatch was not asked to wire).

use std::collections::{HashMap, HashSet};

use cartalith_assets::{
    AssetDB, Anchor, ChromaKey, DecodedImage, Family, Finding, FindingKind, GridRect, ImportLimits, ItemTransform,
    LibraryItem, SeamScore, SeamlessOutcome, Severity, make_seamless, refine, seam_score,
    PackEntries, PackManifest, SliceCounts, SliceGrid, SliceOptions, SlotTarget, archive, check_png_header,
    compute_cells, count_cells, encode_png, fit_to_bottom, item_hash, library, plan_pack_import,
    read_pack_entries_limited, render_item, sheet_base_name, slice_sheet,
};

use cartalith_assets::lineart::{LineArtOptions, LineArtResult, convert_line_art, working_image};

use crate::pack::colour_import::{ColourReport, decode_png_managed};
/// The plain (unmanaged) decode is used only by this file's tests since
/// ET-E2 moved both import paths to `decode_png_managed`.
#[cfg(test)]
use cartalith_assets::decode_png;

/// The sprite-sheet slicer's loaded sheet — the reference's
/// `SpriteSheetImporter.sheet` (`{img,cv,ctx,w,h,name}`, line 27833), minus
/// the three canvas handles. Held on the session rather than passed per call
/// so the modal's live `N cells detected · M non-empty` readout can re-run the
/// real detection pass on every spinbox change without re-sending a 3072×2048
/// PNG across the boundary each time.
pub struct LoadedSheet {
    /// The sheet's file name; [`sheet_base_name`] turns it into the item-name prefix.
    pub name: String,
    /// The decoded sheet pixels, kept so every preview re-runs detection without re-sending the PNG.
    pub image: DecodedImage,
    /// What the colour-managed decode found (ET-E2), so every cell sliced
    /// from a converted sheet carries the same note into the pack report.
    pub colour: ColourReport,
}

/// The slicer modal's four numbers plus its three toggles, in engine terms.
#[derive(Debug, Clone, PartialEq)]
pub struct SliceParams {
    /// Column count as typed; [`SliceGrid::new`] applies the reference's input guards.
    pub cols: i64,
    /// Row count as typed; guarded like `cols`.
    pub rows: i64,
    /// §8's *Margin px* — a uniform inset of the reference's `gridRect`.
    pub margin: f64,
    /// Gutter between cells in sheet pixels; guarded by [`SliceGrid::new`].
    pub spacing: f64,
    /// `background → transparent` (`#alChEnable`/`#alChTol`), off when `None`.
    pub chroma: Option<ChromaKey>,
    /// §8's *Trim transparent edges* — a disclosed port-side addition; see
    /// `cartalith_assets::slicer`'s module docs.
    pub trim: bool,
    /// `#alSlSkip`, on by default in the reference.
    pub skip_blank: bool,
    /// AS-17's per-interior-line drag: `cartalith_assets::SliceGrid::with_lines`'s
    /// own override, carried through unvalidated (`compute_cells` already
    /// falls back to uniform for a `Some` of the wrong length).
    pub col_lines: Option<Vec<f64>>,
    /// The row equivalent of [`SliceParams::col_lines`].
    pub row_lines: Option<Vec<f64>>,
    /// AS-17's cell-scoped slicing: when `Some`, [`AssetLibrarySession::apply_slice`]
    /// cuts only the cell at this flat `row*cols+col` index instead of the
    /// whole grid. `None` is the reference's own always-whole-sheet slice.
    pub only_cell: Option<usize>,
}

/// Where a slice's cells land. The first three are the reference's own
/// `#alSlTarget` options; [`SliceTarget::Family`] is §8's instead — see
/// [`AssetLibrarySession::apply_slice`].
#[derive(Debug, Clone, PartialEq)]
pub enum SliceTarget {
    /// An existing slot, addressed by uid (the reference's `alSlTarget` slot value).
    Slot { uid: String },
    /// One new custom slot, every cell a variant of it (the reference's `__new__`).
    NewCustom { name: String, set: String },
    /// One new custom slot per cell (the reference's `__percell__`).
    PerCell { set: String },
    /// This port's *Assign to family* (`DCC_SHELL_SPEC.md` §8): one cell per slot of the family.
    Family { family: Family, overwrite: bool },
}

/// Everything the slicer modal redraws on a settings change: the real
/// detected/non-empty counts, plus the grid lines to draw them against.
#[derive(Debug, Clone, PartialEq)]
pub struct SlicePreview {
    pub counts: SliceCounts,
    /// `(left, right)` per column, in sheet pixels.
    pub col_spans: Vec<(f64, f64)>,
    /// `(top, bottom)` per row, in sheet pixels.
    pub row_spans: Vec<(f64, f64)>,
    /// AS-17: the raw column/row division lines in sheet pixels, `cols+1`/
    /// `rows+1` of them -- a drag target's hit-test needs the undisplaced
    /// line, not `col_spans`'/`row_spans`' gutter-displaced cell edges.
    pub col_lines_px: Vec<f64>,
    pub row_lines_px: Vec<f64>,
}

/// What a slice actually did — the numbers behind the reference's own
/// `'Added N cells (M blank skipped) → slot'` toast.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SliceOutcome {
    /// Cells that landed in a slot.
    pub added: usize,
    /// Cells dropped because they were blank, when skip-blank is on.
    pub skipped_blank: usize,
    /// Cells that had nowhere to go: a single-image family taking only the
    /// first, or more cells than the target family has slots.
    pub unplaced: usize,
    /// Every slot uid the slice wrote to, in write order.
    pub uids: Vec<String>,
}

/// How [`AssetLibrarySession::import_pack`] treats a library slot that
/// already holds art (`ASSET_PACK_TOOLS_PROPOSAL.md` AP-1: "never overwrite
/// existing slots without confirm").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackImportMode {
    /// Read and plan the pack, report the conflicts, change nothing. The
    /// shell calls this first and only asks the user when it reports any.
    DryRun,
    /// Import into empty slots only; a slot that already holds art keeps it
    /// and its pack paths are reported as skipped. Also keeps a library pack
    /// name, author or licence the pack would change.
    KeepExisting,
    /// The user confirmed: a slot that already holds art is emptied and then
    /// filled from the pack, and the pack's name, author and licence win.
    Replace,
}

/// What happened to one pack path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackRowStatus {
    /// Decoded and stored as a new item.
    Imported,
    /// Readable, but not stored, because its slot already held art and the
    /// mode was [`PackImportMode::KeepExisting`].
    Skipped,
    /// Not stored, for the reason on the row (missing, not a PNG, oversize,
    /// undecodable, unsafe path).
    Refused,
}

impl PackRowStatus {
    /// The lowercase word the shell shows and keys its dot colour on.
    pub fn key(self) -> &'static str {
        match self {
            PackRowStatus::Imported => "imported",
            PackRowStatus::Skipped => "skipped",
            PackRowStatus::Refused => "refused",
        }
    }
}

/// One row of a pack import's report: one manifest path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackImportRow {
    /// The manifest section (`"icons"`, `"structures.poi"`, `"custom"`, ...).
    pub section: String,
    /// The slot id as the manifest spells it.
    pub manifest_id: String,
    /// The library slot it landed in or would have, `""` when none exists
    /// (a custom slot is only created once one of its images imports).
    pub uid: String,
    /// The archive path.
    pub path: String,
    /// What happened.
    pub status: PackRowStatus,
    /// Why, for a skipped or refused row; `""` for an imported one.
    pub reason: String,
}

/// Everything [`AssetLibrarySession::import_pack`] reports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackImportOutcome {
    /// The pack's own name, when its manifest has a non-empty one.
    pub pack_name: Option<String>,
    /// One row per manifest path, in the reference's visit order. Empty for
    /// a dry run.
    pub rows: Vec<PackImportRow>,
    /// The reference's own reroute warnings (`unknown slot "x" →
    /// Custom/Imported`) and any kept pack field, in order.
    pub notes: Vec<String>,
    /// What the pack would overwrite, in words, one entry per slot or pack
    /// field. A dry run fills this; the other modes report what they did
    /// about it in `rows` and `notes`.
    pub conflicts: Vec<String>,
    /// The library slots behind `conflicts`, for a caller that wants to
    /// point at them.
    pub conflict_uids: Vec<String>,
}

impl PackImportOutcome {
    /// How many rows have `status`.
    pub fn count(&self, status: PackRowStatus) -> usize {
        self.rows.iter().filter(|r| r.status == status).count()
    }
}

/// The reference's `(E('alSlSet').value||'Default').trim()||'Default'` (line
/// 27790) in `add_custom_slot`'s own terms: a blank set name means "let
/// `AssetDB` apply its own `Default`", not "a set literally named nothing".
fn set_or_none(set: &str) -> Option<&str> {
    let t = set.trim();
    (!t.is_empty()).then_some(t)
}

/// AP-7's pixel REFINE verbs (`ASSET_PACK_TOOLS_PROPOSAL.md` §6 and §4,
/// Ruling CQ Q7: each writes a new variant). *Fit all* is the eighth verb and
/// is transform-only, so it is [`AssetLibrarySession::fit_all`], not a member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineVerb {
    /// Crop to the inked box ([`refine::trim`]).
    Trim,
    /// Add a transparent margin ([`refine::pad`]).
    Pad,
    /// Mirror left to right ([`refine::flip_horizontal`]).
    Flip,
    /// A small seeded hue/saturation/value change ([`refine::tint_jitter`]).
    Tint,
    /// An ink outline behind the art ([`refine::outline`]).
    Outline,
    /// A drop shadow behind the art ([`refine::drop_shadow`]).
    Shadow,
    /// Move the art's hue to the Look's nearest swatch ([`refine::harmonise`]).
    Harmonise,
}

impl RefineVerb {
    /// The verb for the shell's key (`"trim"`, `"pad"`, `"flip"`, `"tint"`,
    /// `"outline"`, `"shadow"`, `"harmonise"`), or `None`.
    pub fn from_key(key: &str) -> Option<RefineVerb> {
        Some(match key {
            "trim" => RefineVerb::Trim,
            "pad" => RefineVerb::Pad,
            "flip" => RefineVerb::Flip,
            "tint" => RefineVerb::Tint,
            "outline" => RefineVerb::Outline,
            "shadow" => RefineVerb::Shadow,
            "harmonise" => RefineVerb::Harmonise,
            _ => return None,
        })
    }

    /// What the new variant's name gets after `·`.
    pub fn suffix(self) -> &'static str {
        match self {
            RefineVerb::Trim => "trimmed",
            RefineVerb::Pad => "padded",
            RefineVerb::Flip => "flipped",
            RefineVerb::Tint => "tinted",
            RefineVerb::Outline => "outlined",
            RefineVerb::Shadow => "shadowed",
            RefineVerb::Harmonise => "harmonised",
        }
    }

    /// Why this verb does not apply to `family`, in words, or `None` when it
    /// does. Two rules:
    ///
    /// - trim, pad, outline and shadow are for sprites: a ground tile is
    ///   tiled edge to edge, so it has no margin to trim or pad and no edge
    ///   to outline or shade;
    /// - tint and harmonise are refused for the splat `textures` family:
    ///   the map takes a splat texture's colour from its own ramp and draws
    ///   only the texture's variation about its per-channel mean
    ///   (`finalize_pack_texture_inv_mean`, proposal §1.3 fact 3), so a
    ///   recolour there would not show.
    pub fn refusal(self, family: Family) -> Option<&'static str> {
        match self {
            RefineVerb::Trim | RefineVerb::Pad | RefineVerb::Outline | RefineVerb::Shadow if family.is_texture() => {
                Some("a ground tile is tiled edge to edge: it has no margin to trim or pad and no edge to outline or shade")
            }
            RefineVerb::Tint | RefineVerb::Harmonise if family == Family::Textures => Some(
                "splat textures take their colour from the map's own ramp; only their variation about their mean is drawn, so a recolour would not show",
            ),
            _ => None,
        }
    }
}

/// What [`AssetLibrarySession::refine_variant`] did.
#[derive(Debug, Clone, PartialEq)]
pub struct RefineOutcome {
    /// The new variant's index, or `None` when nothing was written.
    pub index: Option<usize>,
    /// One line for the status bar: what was done, or why nothing was.
    pub note: String,
    /// *Tint*: the jitter applied.
    pub jitter: Option<refine::Jitter>,
    /// *Harmonise*: the hues measured and the target swatch.
    pub harmonised: Option<refine::Harmonised>,
}

/// FNV-1a, 64-bit (the published offset basis and prime), for *Tint*'s seed.
/// Not a content hash of record (that is [`item_hash`]); only a stable way to
/// turn a string into a seed.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The live Asset Library: `AssetDB`'s metadata registry plus the decoded
/// pixels behind every stored item.
pub struct AssetLibrarySession {
    pub db: AssetDB,
    /// uid -> decoded variants, index-parallel to `db`'s own `items(uid)`.
    images: HashMap<String, Vec<DecodedImage>>,
    /// The slicer's currently-loaded sheet, if any. **Never** written to by a
    /// slice: the operation is non-destructive (`DCC_SHELL_SPEC.md` §8), so
    /// slicing the same sheet twice with different settings is a supported
    /// thing to do.
    sheet: Option<LoadedSheet>,
    /// AP-2: the colour report of every item whose import had something to
    /// say (a conversion, or a warning), keyed by the item's content hash
    /// ([`item_hash`]). Keyed by hash rather than by uid and index so a
    /// rename, move, duplicate or delete can never point a note at the wrong
    /// image; an entry whose image is gone is simply never matched. Session
    /// state only: not saved, like the pixels it describes.
    colour: HashMap<String, ColourReport>,
}

impl Default for AssetLibrarySession {
    /// Same as [`AssetLibrarySession::new`], so `default()` and `new()` can never
    /// disagree about what an empty session is.
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLibrarySession {
    /// A freshly bootstrapped session: `AssetDB::new()`'s frozen slots all empty, no
    /// decoded pixels and no loaded sheet. `images` must start empty: every item in
    /// `db` has to have its pixels beside it (see the module doc).
    pub fn new() -> Self {
        AssetLibrarySession { db: AssetDB::new(), images: HashMap::new(), sheet: None, colour: HashMap::new() }
    }

    /// The decoded pixels behind one stored item, if both `uid` and `index`
    /// resolve on both sides of the parallel store.
    pub fn image(&self, uid: &str, index: usize) -> Option<&DecodedImage> {
        self.images.get(uid)?.get(index)
    }

    // -- Import ---------------------------------------------------------

    /// Decode `bytes` as a PNG and add it as a new item on `uid` — the
    /// reference's `AssetImporter.intake` (per-file half): computes a real
    /// [`item_hash`], and, for a bottom-anchored family (feature icons),
    /// applies [`fit_to_bottom`] on intake exactly as the reference does.
    /// `Err` for an unknown `uid` or bytes that don't decode as a PNG —
    /// never a panic across the boundary.
    ///
    /// Colour-managed since ET-E2: a thin wrapper over
    /// [`AssetLibrarySession::import_item_managed`] that drops the report.
    /// Since ET-E2 the `#[func]` layer calls the managed variant, so outside
    /// tests nothing calls this wrapper; it keeps the pre-ET-E2 signature
    /// for this session's existing unit tests.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn import_item(&mut self, uid: &str, name: String, bytes: &[u8]) -> Result<(), String> {
        self.import_item_managed(uid, name, bytes).map(|_| ())
    }

    /// [`AssetLibrarySession::import_item`], returning what the colour-managed
    /// decode found and did (`EDITING_TOOLS_SCOPE.md` ET-E2): a PNG tagged
    /// with a non-sRGB colour space is converted to sRGB before it is stored,
    /// hashed or fitted, so every later use (thumbnails, the pack bake, Apply
    /// to map) sees sRGB numbers. An untagged or sRGB-tagged PNG stores the
    /// same bytes as before. Colour problems are warnings in the report,
    /// never an `Err`; `Err` is only an unknown `uid` or an unreadable PNG.
    pub fn import_item_managed(&mut self, uid: &str, name: String, bytes: &[u8]) -> Result<ColourReport, String> {
        if self.db.get(uid).is_none() {
            return Err(format!("no such slot: {uid}"));
        }
        let (decoded, report) = decode_png_managed(bytes).map_err(|e| e.to_string())?;
        let hash = self.insert_decoded(uid, name, decoded)?;
        self.note_colour(hash, &report);
        Ok(report)
    }

    /// The reference's `mkItem` (line 27793) plus the store write: default
    /// transform, `fitToBottom` on a bottom-anchored family, a real
    /// [`item_hash`], and both stores advanced together. Shared by
    /// [`AssetLibrarySession::import_item`] and the slicer, which build the
    /// same item out of differently-sourced pixels.
    ///
    /// Returns the new item's content hash, which the colour notes key on.
    fn insert_decoded(&mut self, uid: &str, name: String, decoded: DecodedImage) -> Result<String, String> {
        let Some(slot) = self.db.get(uid) else {
            return Err(format!("no such slot: {uid}"));
        };
        let family = slot.family;
        let mut transform = ItemTransform::default();
        if family.anchor() == Anchor::Bottom {
            fit_to_bottom(&mut transform, decoded.w, decoded.h, family.size());
        }
        let hash = item_hash(&decoded);
        let item = LibraryItem::new(name, hash.clone()).with_transform(transform);
        if !self.db.add_item(uid, item) {
            return Err(format!("no such slot: {uid}"));
        }
        self.images.entry(uid.to_string()).or_default().push(decoded);
        Ok(hash)
    }

    /// Keep `report` for the pack report when it has something to say: a
    /// conversion, or a warning. An untagged or sRGB image (the common case)
    /// stores nothing, so the report never lists a non-event.
    fn note_colour(&mut self, hash: String, report: &ColourReport) {
        if report.converted || !report.warnings.is_empty() {
            self.colour.insert(hash, report.clone());
        }
    }

    // -- Pack import (`AssetImporter.importPackZip`, AP-1) ---------------

    /// Open a pack `.zip` into the library — the reference's
    /// `AssetImporter.importPackZip` (v2.11 line 27715), which
    /// `ASSET_PACK_TOOLS_PROPOSAL.md` §1.4 found unported. `source` is the
    /// file name the user picked; it goes into each filled slot's
    /// `SlotMeta::source` as provenance.
    ///
    /// Faithful to the reference: the raw manifest's sections are visited in
    /// its order, an id outside a frozen family's vocabulary goes to a custom
    /// slot in the `Imported` set with the reference's own warning, a custom
    /// set keeps its name, every item is named after its slot id (the
    /// reference's `label`), and a bottom-anchored family is fitted on intake
    /// (`insert_decoded`, the reference's `fitToBottom`).
    ///
    /// **Deliberate departures, each from an owner instruction:**
    ///
    /// - **A slot that already holds art is never overwritten unasked.** The
    ///   reference replaces a single-image slot and *appends* to a
    ///   multi-variant one, so importing the same pack twice doubled every
    ///   icon. Here a filled slot is a conflict ([`PackImportMode`]); with
    ///   [`PackImportMode::Replace`] it is emptied first, so a second import
    ///   of the same pack is a no-op on content.
    /// - **The same rule covers the pack name, author and licence.** The
    ///   reference overwrites the library's fields whenever the manifest has
    ///   a value.
    /// - **The archive is read under [`ImportLimits::DEFAULT`]**, and each
    ///   image's PNG header is checked before it is decoded
    ///   ([`check_png_header`]), because a pack opened for editing is
    ///   somebody else's file. A refusal is a row with a reason, never a
    ///   panic; a hostile archive (`ImportRefusal`) refuses the whole pack.
    /// - **Sea marks are imported.** The reference predates that family; this
    ///   port's exporter writes it (owner ruling 2026-09-02).
    /// - **Provenance.** Each slot filled here gets `SlotMeta::source`
    ///   (`"<file> (<section>/<id>)"`) and, when the pack has one, the pack
    ///   licence in `SlotMeta::license`, both only where the slot's own field
    ///   is blank or the slot was replaced. The reference writes neither.
    ///   Both are existing `library.json` fields, so no save key is added.
    ///
    /// Images are decoded with `decode_png_managed` (ET-E2), so a tagged
    /// non-sRGB image is converted on the way in, exactly as Import image…
    /// does.
    pub fn import_pack(&mut self, bytes: &[u8], source: &str, mode: PackImportMode) -> Result<PackImportOutcome, String> {
        let limits = ImportLimits::DEFAULT;
        let entries = read_pack_entries_limited(bytes, &limits).map_err(|e| e.to_string())?;
        let plan = {
            let db = &self.db;
            plan_pack_import(&entries, &limits, |family, id| {
                db.slots_in_family(family).iter().any(|s| s.id == id)
            })
            .map_err(|e| e.to_string())?
        };
        let mut out = PackImportOutcome { pack_name: plan.name.clone(), ..Default::default() };

        // Resolve every planned slot to its library uid without creating
        // anything: a custom slot's uid comes from `add_custom_slot` on a
        // metadata-only copy, so the real registry is untouched by a dry run
        // and by a slot whose every image is refused.
        let mut scratch = self.db.clone();
        let uids: Vec<String> = plan
            .slots
            .iter()
            .map(|s| match &s.target {
                SlotTarget::Frozen { family, id } => scratch
                    .slots_in_family(*family)
                    .into_iter()
                    .find(|slot| &slot.id == id)
                    .map(|slot| slot.uid.clone())
                    .unwrap_or_default(),
                SlotTarget::Custom { id, set } => scratch.add_custom_slot(id, Some(set)).uid.clone(),
            })
            .collect();

        // Conflicts: slots that hold art now and that this pack would write
        // to, and pack fields it would change.
        let mut conflicted: HashSet<String> = HashSet::new();
        for (slot, uid) in plan.slots.iter().zip(&uids) {
            let writes = slot.paths.iter().any(|p| p.refusal.is_none());
            let held = self.db.items(uid).len();
            if writes && held > 0 && conflicted.insert(uid.clone()) {
                let name = self.db.get(uid).map(|s| s.name.clone()).unwrap_or_else(|| uid.clone());
                out.conflicts.push(format!(
                    "{name} ({}) holds {held} item{}",
                    self.db.get(uid).map(|s| s.family.key()).unwrap_or(""),
                    if held == 1 { "" } else { "s" }
                ));
                out.conflict_uids.push(uid.clone());
            }
        }
        let fields: [(&str, &Option<String>, String); 3] = [
            ("name", &plan.name, self.db.pack.name.trim().to_string()),
            ("author", &plan.author, self.db.pack.author.trim().to_string()),
            ("licence", &plan.license, self.db.pack.license.trim().to_string()),
        ];
        for (label, theirs, ours) in &fields {
            if let Some(t) = theirs
                && !ours.is_empty()
                && ours != t.trim()
            {
                out.conflicts.push(format!("pack {label}: the library has \"{ours}\", the pack has \"{t}\""));
            }
        }
        if mode == PackImportMode::DryRun {
            return Ok(out);
        }

        // Pack fields: the reference's `if(m.name) E('alPackName').value=m.name`,
        // minus the unasked overwrite.
        for (label, theirs, ours) in fields {
            let Some(t) = theirs.clone() else { continue };
            if ours.is_empty() || ours == t.trim() || mode == PackImportMode::Replace {
                match label {
                    "name" => self.db.pack.name = t,
                    "author" => self.db.pack.author = t,
                    _ => self.db.pack.license = t,
                }
            } else {
                out.notes.push(format!("kept the library's pack {label} \"{ours}\"; the pack's is \"{t}\""));
            }
        }

        let mut cleared: HashSet<String> = HashSet::new();
        for (slot, uid) in plan.slots.iter().zip(&uids) {
            if let Some(note) = &slot.note {
                out.notes.push(note.clone());
            }
            let is_conflict = conflicted.contains(uid);
            for planned in &slot.paths {
                let mut row = PackImportRow {
                    section: slot.section.clone(),
                    manifest_id: slot.manifest_id.clone(),
                    uid: if self.db.get(uid).is_some() { uid.clone() } else { String::new() },
                    path: planned.path.clone(),
                    status: PackRowStatus::Refused,
                    reason: String::new(),
                };
                if let Some(why) = &planned.refusal {
                    row.reason = why.clone();
                    out.rows.push(row);
                    continue;
                }
                if is_conflict && mode == PackImportMode::KeepExisting {
                    row.status = PackRowStatus::Skipped;
                    row.reason = format!("the slot already held {} item(s); kept them", self.db.items(uid).len());
                    out.rows.push(row);
                    continue;
                }
                let png = entries.get(&planned.path).map(Vec::as_slice).unwrap_or_default();
                if let Err(why) = check_png_header(png, &limits) {
                    row.reason = why;
                    out.rows.push(row);
                    continue;
                }
                let (decoded, colour) = match decode_png_managed(png) {
                    Ok(pair) => pair,
                    Err(e) => {
                        row.reason = format!("not a readable PNG ({e})");
                        out.rows.push(row);
                        continue;
                    }
                };
                // Create a custom slot only now that something lands in it.
                let target_uid = match &slot.target {
                    SlotTarget::Custom { id, set } => self.add_custom_slot(id, Some(set)),
                    SlotTarget::Frozen { .. } => uid.clone(),
                };
                let first_write = cleared.insert(target_uid.clone());
                if first_write && is_conflict {
                    self.clear_slot_items(&target_uid);
                }
                let family = self.db.get(&target_uid).map(|s| s.family);
                // A single-image family holds one item: a later path replaces
                // it, the reference's `store[uid]=[item]`.
                if family.is_some_and(|f| !f.is_multi()) {
                    self.clear_slot_items(&target_uid);
                }
                match self.insert_decoded(&target_uid, slot.manifest_id.clone(), decoded) {
                    Ok(hash) => self.note_colour(hash, &colour),
                    Err(e) => {
                        row.reason = e;
                        out.rows.push(row);
                        continue;
                    }
                }
                if first_write && let Some(meta) = self.db.slot_meta_mut(&target_uid) {
                    if meta.source.trim().is_empty() || is_conflict {
                        meta.source = format!("{source} ({}/{})", slot.section, slot.manifest_id);
                    }
                    if let Some(lic) = &plan.license
                        && (meta.license.trim().is_empty() || is_conflict)
                    {
                        meta.license = lic.clone();
                    }
                }
                row.uid = target_uid;
                row.status = PackRowStatus::Imported;
                out.rows.push(row);
            }
        }
        Ok(out)
    }

    /// Add (or return the existing) custom slot — `AssetDB::add_custom_slot`,
    /// exposed so a caller can land a loose image somewhere real before
    /// [`import_item`] has a uid to target.
    pub fn add_custom_slot(&mut self, name: &str, set_name: Option<&str>) -> String {
        self.db.add_custom_slot(name, set_name).uid.clone()
    }

    // -- Sprite-sheet slicer (`SpriteSheetImporter`) -----------------------

    /// `SpriteSheetImporter.loadSheet` (line 27825): decode a sprite sheet and
    /// hold it for slicing. Replaces any previously-loaded sheet. `Err` for
    /// bytes that don't decode as a PNG — never a panic across the boundary.
    ///
    /// PNG only, which is narrower than the reference's own
    /// `png|jpe?g|gif|webp|bmp|svg` accept list: `cartalith-assets` builds
    /// `image` with the `png` codec alone (its `Cargo.toml` says why), so this
    /// reports the limit honestly rather than pretending to a decoder that is
    /// not compiled in.
    ///
    /// Colour-managed since ET-E2: a thin wrapper over
    /// [`AssetLibrarySession::load_sheet_managed`] that drops the report.
    /// Outside tests nothing calls it since ET-E2 (see `import_item`).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn load_sheet(&mut self, name: String, bytes: &[u8]) -> Result<(u32, u32), String> {
        self.load_sheet_managed(name, bytes).map(|(w, h, _)| (w, h))
    }

    /// [`AssetLibrarySession::load_sheet`], returning the colour report
    /// (ET-E2). The sheet is converted to sRGB once, here, so every cell the
    /// slicer cuts from it is already sRGB.
    pub fn load_sheet_managed(&mut self, name: String, bytes: &[u8]) -> Result<(u32, u32, ColourReport), String> {
        let (image, report) = decode_png_managed(bytes).map_err(|e| e.to_string())?;
        let dims = (image.w, image.h);
        self.sheet = Some(LoadedSheet { name, image, colour: report.clone() });
        Ok((dims.0, dims.1, report))
    }

    /// Drop the loaded sheet (the modal closing, or a fresh pick failing).
    pub fn clear_sheet(&mut self) {
        self.sheet = None;
    }

    /// Turn the modal's four numbers into the reference's own grid: `margin`
    /// insets `gridRect` ([`GridRect::inset`]), `cols`/`rows`/`spacing` go
    /// through [`SliceGrid::new`]'s ported input guards.
    fn build_grid(&self, p: &SliceParams) -> Result<(&LoadedSheet, SliceGrid), String> {
        let Some(sheet) = self.sheet.as_ref() else {
            return Err("No sprite sheet loaded.".to_string());
        };
        let Some(rect) = GridRect::inset(sheet.image.w, sheet.image.h, p.margin) else {
            return Err("Margin leaves no room to slice.".to_string());
        };
        let grid = SliceGrid::new(rect, p.cols, p.rows, p.spacing)
            .with_lines(p.col_lines.clone(), p.row_lines.clone());
        Ok((sheet, grid))
    }

    /// §8's `N cells detected · M non-empty` readout, computed by the real
    /// detection pass ([`count_cells`]) rather than sampled — same crop, same
    /// chroma key, same `isBlank` threshold the slice itself would use —
    /// plus the column/row spans a grid *overlay* needs.
    ///
    /// The spans exist so the Godot side never reimplements
    /// `computeCells`'s half-gutter arithmetic: an overlay drawn from the
    /// obvious equal-pitch formula sits visibly off the cells the slice
    /// actually cuts, which is exactly the class of drift the "no numbers in
    /// GDScript" rule is there to prevent. `2*cols + 2*rows` floats at the
    /// 128×128 ceiling is 512 — cheap enough to re-send on every spinbox
    /// change.
    pub fn slice_preview(&self, p: &SliceParams) -> Result<SlicePreview, String> {
        let (sheet, grid) = self.build_grid(p)?;
        let computed = compute_cells(&grid);
        Ok(SlicePreview {
            counts: count_cells(&sheet.image, &grid, p.chroma.as_ref()),
            col_spans: computed.column_spans(),
            row_spans: computed.row_spans(),
            col_lines_px: computed.col_line_px,
            row_lines_px: computed.row_line_px,
        })
    }

    /// Slice the loaded sheet and land the cells in the library — the
    /// reference's `addSlices()` (line 27782). Non-destructive: the sheet
    /// stays loaded and untouched, so the same sheet can be re-sliced with
    /// different settings.
    ///
    /// The four targets are the reference's own three plus one this port
    /// adds:
    ///
    /// - [`SliceTarget::Slot`] — `alSlTarget` set to a real slot uid. A
    ///   multi-variant family takes every cell; a single-image family takes
    ///   the **first** cell and stops, replacing whatever was there
    ///   (`store[uid]=[item]`, line 27818).
    /// - [`SliceTarget::NewCustom`] — the reference's `__new__` + "New name":
    ///   one new custom slot, every cell as a variant of it.
    /// - [`SliceTarget::PerCell`] — the reference's `__percell__`: one new
    ///   custom slot *per cell*, named `cell N` (line 27796).
    /// - [`SliceTarget::Family`] — **not in the reference.**
    ///   `DCC_SHELL_SPEC.md` §8's *Assign to family* + *Fill from
    ///   first-empty/overwrite*, which the reference expresses as a flat slot
    ///   dropdown instead. One cell per slot, in the family's frozen
    ///   vocabulary order; `overwrite` starts at the first slot and replaces,
    ///   otherwise only slots that are currently empty are filled. Composed
    ///   entirely out of the reference's own primitives (`add_item` into a
    ///   real slot) — no new arithmetic, nothing golden-covered changes.
    pub fn apply_slice(&mut self, p: &SliceParams, target: &SliceTarget) -> Result<SliceOutcome, String> {
        let (sheet, grid) = self.build_grid(p)?;
        let base = sheet_base_name(&sheet.name);
        let opts = SliceOptions { chroma: p.chroma, trim: p.trim, skip_blank: p.skip_blank };
        let sheet_colour = sheet.colour.clone();
        let mut cells = slice_sheet(&sheet.image, &grid, &opts);
        let grid_total = (grid.cols as usize) * (grid.rows as usize);
        // AS-17: narrow to one cell -- everything below (naming, counts,
        // per-target placement) runs unchanged over whatever `cells` holds,
        // so a scoped slice is not a second code path, just a shorter list.
        if let Some(idx) = p.only_cell {
            cells.retain(|c| c.index == idx);
        }
        if cells.is_empty() {
            return Err(if p.only_cell.is_some() {
                "No cell to add — the selected cell is empty, or out of range for the current grid.".to_string()
            } else if p.skip_blank {
                "No cells to add — every cell in this grid is empty (or the grid is too dense).".to_string()
            } else {
                "No cells to add — the grid is too dense; reduce columns/rows or spacing.".to_string()
            });
        }
        let skipped_blank = if p.skip_blank {
            let total = if p.only_cell.is_some() { 1 } else { grid_total };
            total - cells.len()
        } else {
            0
        };

        let mut out = SliceOutcome { added: 0, skipped_blank, unplaced: 0, uids: Vec::new() };
        match target {
            SliceTarget::Slot { uid } => {
                let Some(slot) = self.db.get(uid) else {
                    return Err(format!("no such slot: {uid}"));
                };
                let multi = slot.family.is_multi();
                if !multi {
                    // `store[uid]=[item]`: a single-image family holds one.
                    self.clear_slot_items(uid);
                }
                for cell in &cells {
                    let name = cell.default_name(&base);
                    let hash = self.insert_decoded(uid, name, cell.image.clone())?;
                    self.note_colour(hash, &sheet_colour);
                    out.added += 1;
                    if !multi {
                        break;
                    }
                }
                out.uids.push(uid.clone());
                out.unplaced = cells.len() - out.added;
            }
            SliceTarget::NewCustom { name, set } => {
                let uid = self.add_custom_slot(name, set_or_none(set));
                for cell in &cells {
                    let item_name = cell.default_name(&base);
                    let hash = self.insert_decoded(&uid, item_name, cell.image.clone())?;
                    self.note_colour(hash, &sheet_colour);
                    out.added += 1;
                }
                out.uids.push(uid);
            }
            SliceTarget::PerCell { set } => {
                for cell in &cells {
                    let name = cell.default_cell_name();
                    let uid = self.add_custom_slot(&name, set_or_none(set));
                    let hash = self.insert_decoded(&uid, name, cell.image.clone())?;
                    self.note_colour(hash, &sheet_colour);
                    out.added += 1;
                    out.uids.push(uid);
                }
            }
            SliceTarget::Family { family, overwrite } => {
                let mut targets: Vec<String> = self
                    .db
                    .slots_in_family(*family)
                    .into_iter()
                    .map(|s| s.uid.clone())
                    .collect();
                if !overwrite {
                    targets.retain(|uid| self.db.items(uid).is_empty());
                }
                if targets.is_empty() {
                    return Err(if *overwrite {
                        format!("The {} family has no slots to fill.", family.key())
                    } else {
                        format!("Every slot in the {} family is already filled.", family.key())
                    });
                }
                for (cell, uid) in cells.iter().zip(&targets) {
                    if *overwrite {
                        self.clear_slot_items(uid);
                    }
                    let name = cell.default_name(&base);
                    let hash = self.insert_decoded(uid, name, cell.image.clone())?;
                    self.note_colour(hash, &sheet_colour);
                    out.added += 1;
                    out.uids.push(uid.clone());
                }
                out.unplaced = cells.len().saturating_sub(targets.len());
            }
        }
        Ok(out)
    }

    // -- Removal / clearing ----------------------------------------------

    /// Remove one item, keeping both stores in lockstep.
    pub fn remove_item(&mut self, uid: &str, index: usize) -> bool {
        let removed = self.db.remove_item(uid, index).is_some();
        if removed
            && let Some(v) = self.images.get_mut(uid)
            && index < v.len()
        {
            v.remove(index);
        }
        removed
    }

    /// Drop every item from a slot without removing the slot itself — the
    /// reference's `store[uid]=[]`, used by batch-delete on a *frozen* slot
    /// (a frozen slot cannot be removed, only emptied; see
    /// [`AssetLibrarySession::remove_custom_slot`] for the custom half).
    fn clear_slot_items(&mut self, uid: &str) {
        while self.db.remove_item(uid, 0).is_some() {}
        self.images.remove(uid);
    }

    /// Remove a custom slot entirely (and its images) — `false` for an
    /// unknown or frozen `uid`.
    pub fn remove_custom_slot(&mut self, uid: &str) -> bool {
        let ok = self.db.remove_custom_slot(uid);
        if ok {
            self.images.remove(uid);
        }
        ok
    }

    /// Reset to a freshly bootstrapped, empty library — `AssetDB::clear`
    /// plus dropping every decoded pixel.
    pub fn clear(&mut self) {
        self.db.clear();
        self.images.clear();
        self.colour.clear();
    }

    // -- Validation --------------------------------------------------------

    /// `AssetValidator.run()` — real warning strings, ordered exactly as the
    /// reference produces them.
    pub fn validate(&self) -> Vec<String> {
        library::run(&self.db)
    }

    // -- Pack report (AP-2) ----------------------------------------------------

    /// The pack report: `cartalith_assets::pack_report`'s findings over this
    /// session's pixels, then one [`FindingKind::Colour`] finding per stored
    /// item whose import converted its colour space or warned about it, in
    /// store order, carrying the kept [`ColourReport`] so the shell words it
    /// with its one existing formatter (`colour_note_for`). Read-only.
    pub fn pack_report(&self) -> Vec<(Finding, Option<ColourReport>)> {
        let mut out: Vec<(Finding, Option<ColourReport>)> =
            cartalith_assets::pack_report(&self.db, |u, i| self.image(u, i)).into_iter().map(|f| (f, None)).collect();
        for uid in self.db.uids_in_order() {
            let name = self.db.get(&uid).map(|s| s.name.clone()).unwrap_or_default();
            for (i, item) in self.db.items(&uid).iter().enumerate() {
                let Some(report) = self.colour.get(&item.hash) else { continue };
                let text = if report.converted {
                    format!("converted from {} to sRGB at import", report.source)
                } else {
                    report.warnings.join(" · ")
                };
                out.push((
                    Finding {
                        kind: FindingKind::Colour,
                        severity: Severity::Info,
                        uid: Some(uid.clone()),
                        index: Some(i),
                        label: format!("{name} #{}", i + 1),
                        text,
                    },
                    Some(report.clone()),
                ));
            }
        }
        out
    }

    // -- Seam checker and make-seamless (AP-4) ---------------------------------

    /// The item's own bake for its family (`render_item` at `Family::size`,
    /// opaque), which is what export writes and the map samples, so the seam
    /// tools measure and fix the pixels a pack actually ships -- not the
    /// source at its import size. `None` for an unknown uid or index, or a
    /// family that is not a tiled ground texture (`Family::is_texture`): a
    /// sprite has no wrap to check.
    fn texture_bake(&self, uid: &str, index: usize) -> Option<DecodedImage> {
        let slot = self.db.get(uid)?;
        if !slot.family.is_texture() {
            return None;
        }
        let item = self.db.items(uid).get(index)?;
        let img = self.image(uid, index)?;
        Some(render_item(img, &item.transform, slot.family.size(), true))
    }

    /// AP-4's checker: `seam_score` of one ground item's bake, and whether
    /// that is a seam. The same numbers AP-2's report words. `None` as
    /// [`AssetLibrarySession::texture_bake`]. Read-only.
    pub fn seam_check(&self, uid: &str, index: usize) -> Option<SeamScore> {
        self.texture_bake(uid, index).map(|b| seam_score(&b))
    }

    /// AP-4's fixer: make one ground item's bake seamless and store the
    /// result as a **new variant** of the same slot (Ruling CQ Q7: refine
    /// verbs never replace the original), named `<original> · seamless`.
    ///
    /// Returns the outcome and, when a variant was added, its index. A tile
    /// that already wraps adds nothing (`outcome.image == None`, index
    /// `None`): the bar is "left untouched (byte-identical)", and the
    /// original item is never written in either case. The new item is the
    /// baked pixels at the family's size with the identity transform, so its
    /// own bake is those pixels again (asserted by this file's tests) and
    /// export ships exactly what was measured. Undo is
    /// [`AssetLibrarySession::remove_item`] on the returned index.
    ///
    /// `Err` for an unknown uid/index or a non-texture family, in words.
    pub fn make_seamless_variant(&mut self, uid: &str, index: usize) -> Result<(SeamlessOutcome, Option<usize>), String> {
        let Some(slot) = self.db.get(uid) else { return Err(format!("no such slot: {uid}")) };
        if !slot.family.is_texture() {
            return Err(format!("{uid} is not a ground texture: only tiled textures wrap"));
        }
        let Some(item) = self.db.items(uid).get(index) else { return Err(format!("{uid} has no variant {}", index + 1)) };
        let name = format!("{} · seamless", item.name);
        let Some(bake) = self.texture_bake(uid, index) else { return Err(format!("{uid} variant {} has no pixels", index + 1)) };
        let mut outcome = make_seamless(&bake);
        let Some(img) = outcome.image.take() else { return Ok((outcome, None)) };
        self.insert_decoded(uid, name, img.clone())?;
        outcome.image = Some(img);
        Ok((outcome, Some(self.db.items(uid).len() - 1)))
    }

    // -- Line-art conversion (AP-10, Ruling CR) ------------------------------

    /// AP-10: convert one sprite variant's line art into a clean, solid,
    /// one-ink sprite (`cartalith_assets::lineart`) and store it as a **new
    /// variant** named `<original> · line art` (Ruling CQ Q7: refine verbs
    /// never replace the original; this never writes the original's record
    /// or pixels).
    ///
    /// The source is the original's own pixels at their full resolution,
    /// drawn with its transform (`working_image`, so the result lines up with
    /// what the user framed), never the 256² bake: Ruling CR's "at full
    /// source resolution". The stored variant is the family-size coverage
    /// bake with the identity transform, so its own bake is those pixels
    /// (tested below) and export ships what was converted. Its SDF is not
    /// kept: Ruling CR Q-L3 makes it a derived cache, so there is no save or
    /// pack key, and AP-5 rebuilds it from the original.
    ///
    /// Returns the result (with `image` emptied: it is in the store) and the
    /// new index. `Err` in words for an unknown uid or index, a ground-texture
    /// family (opaque tiles have no line art to key), or an image with no ink
    /// or no paper. Undo is [`AssetLibrarySession::remove_item`].
    pub fn line_art_variant(&mut self, uid: &str, index: usize, opts: &LineArtOptions) -> Result<(LineArtResult, usize), String> {
        let Some(slot) = self.db.get(uid) else { return Err(format!("no such slot: {uid}")) };
        let family = slot.family;
        if family.is_texture() {
            return Err(format!("{uid} is a ground texture: line-art conversion makes transparent sprites"));
        }
        let Some(item) = self.db.items(uid).get(index) else { return Err(format!("{uid} has no variant {}", index + 1)) };
        let name = format!("{} · line art", item.name);
        let transform = item.transform.clone();
        let Some(img) = self.image(uid, index) else { return Err(format!("{uid} variant {} has no pixels", index + 1)) };
        let work = working_image(img, &transform, family.size());
        let mut result = convert_line_art(&work, opts, family.size()).map_err(|e| e.to_string())?;
        let made = std::mem::replace(&mut result.image, DecodedImage { w: 0, h: 0, rgba: Vec::new() });
        self.insert_decoded(uid, name, made)?;
        Ok((result, self.db.items(uid).len() - 1))
    }

    // -- REFINE verbs (AP-7) -----------------------------------------------------

    /// The transform the inspector's *Fit* button gives one item -- the
    /// default (scale 1, no pan) plus, for a bottom-anchored family, the
    /// reference's `fitToBottom` -- the same arithmetic as
    /// `WorldGen::as_reset_item_transform(fit = true)`. `None` for an unknown
    /// uid or index, or an item without pixels.
    fn fitted_transform(&self, uid: &str, index: usize) -> Option<ItemTransform> {
        let slot = self.db.get(uid)?;
        let img = self.image(uid, index)?;
        let mut t = ItemTransform::default();
        if slot.family.anchor() == Anchor::Bottom {
            fit_to_bottom(&mut t, img.w, img.h, slot.family.size());
        }
        Some(t)
    }

    /// AP-7's *Fit all*: give every variant of every slot in `uids` its
    /// fitted transform ([`AssetLibrarySession::fitted_transform`]).
    ///
    /// **Transform-only**, the one REFINE verb that writes no new variant:
    /// a fit changes where the stored pixels are placed in the bake, never
    /// the pixels, and the per-item *Fit* it batches has always edited the
    /// item in place. The original's bytes are therefore unchanged (AP-7's
    /// bar), and the undo record is what this returns: `(uid, index, old
    /// transform)` for each item that actually moved, in `uids` then store
    /// order. An item already fitted is not listed. Unknown uids are skipped.
    pub fn fit_all(&mut self, uids: &[String]) -> Vec<(String, usize, ItemTransform)> {
        let mut changed = Vec::new();
        for uid in uids {
            for index in 0..self.db.items(uid).len() {
                let Some(t) = self.fitted_transform(uid, index) else { continue };
                let Some(item) = self.db.item_mut(uid, index) else { continue };
                if item.transform != t {
                    changed.push((uid.clone(), index, item.transform.clone()));
                    item.transform = t;
                }
            }
        }
        changed
    }

    /// The item's sprite bake (`render_item` at the family size over
    /// transparency): what export writes for it, and the pixels *Outline* and
    /// *Shadow* draw around, since their widths only mean something at the
    /// size the art ships at.
    fn sprite_bake(&self, uid: &str, index: usize) -> Option<DecodedImage> {
        let slot = self.db.get(uid)?;
        let item = self.db.items(uid).get(index)?;
        let img = self.image(uid, index)?;
        Some(render_item(img, &item.transform, slot.family.size(), slot.family.opaque()))
    }

    /// AP-7: apply one REFINE verb to variant `index` of `uid` and store the
    /// result as a **new variant** of the same slot (Ruling CQ Q7), named
    /// `<original> · <verb's suffix>`; the original item and its pixels are
    /// never written. `palette` is the target for [`RefineVerb::Harmonise`]
    /// (the caller's Look) and is ignored by the other verbs.
    ///
    /// Which pixels each verb takes, and the new variant's transform:
    ///
    /// - Trim, Pad: the **source**, losslessly cropped or extended; the new
    ///   variant gets the family's intake transform (default, fitted to the
    ///   bottom for a bottom-anchored family), because the old pan was
    ///   relative to the old geometry;
    /// - Flip: the source, mirrored; the transform is copied with `pan_x`
    ///   negated, so the new bake is the mirror of the old one;
    /// - Tint, Harmonise: the source, recoloured; the transform is copied;
    /// - Outline, Shadow: the **bake** (see [`Self::sprite_bake`]), with the
    ///   identity transform, so the new variant's own bake is its pixels (up
    ///   to the bake's known one-level RGB drift on soft edges, AP-1's
    ///   finding, which Ruling CR's switched `render_item` fix will remove).
    ///
    /// Returns what was done; `index: None` (nothing written) when the verb
    /// had nothing to do (nothing to trim, blank art for outline or shadow).
    /// `Err` in words for an unknown uid or index, or a verb that does not
    /// apply to the family ([`RefineVerb::refusal`]), or a harmonise refused
    /// for grey art or a grey palette. Undo is
    /// [`AssetLibrarySession::remove_item`] on the returned index.
    pub fn refine_variant(&mut self, uid: &str, index: usize, verb: RefineVerb, palette: &[[u8; 3]]) -> Result<RefineOutcome, String> {
        let Some(slot) = self.db.get(uid) else { return Err(format!("no such slot: {uid}")) };
        let family = slot.family;
        if let Some(why) = verb.refusal(family) {
            return Err(why.to_string());
        }
        let Some(item) = self.db.items(uid).get(index).cloned() else { return Err(format!("{uid} has no variant {}", index + 1)) };
        let Some(src) = self.image(uid, index).cloned() else { return Err(format!("{uid} variant {} has no pixels", index + 1)) };
        let mut out = RefineOutcome { index: None, note: String::new(), jitter: None, harmonised: None };
        // (pixels, transform or None for the family's intake transform)
        let made: Option<(DecodedImage, Option<ItemTransform>)> = match verb {
            RefineVerb::Trim => match refine::trim(&src) {
                Some(t) => {
                    out.note = format!("trimmed {}×{} to {}×{}", src.w, src.h, t.w, t.h);
                    Some((t, None))
                }
                None => {
                    out.note = "nothing to trim: the art already reaches every edge (or is blank)".to_string();
                    None
                }
            },
            RefineVerb::Pad => {
                let p = refine::pad(&src);
                out.note = format!("padded {}×{} to {}×{}", src.w, src.h, p.w, p.h);
                Some((p, None))
            }
            RefineVerb::Flip => {
                let mut t = item.transform.clone();
                t.pan_x = -t.pan_x;
                out.note = "mirrored left to right".to_string();
                Some((refine::flip_horizontal(&src), Some(t)))
            }
            RefineVerb::Tint => {
                // Seeded by the original's content hash and how many variants
                // the slot holds, so pressing it again makes a different
                // variant, and the same library state always makes the same one.
                let seed = fnv1a64(format!("{}:{}", item.hash, self.db.items(uid).len()).as_bytes());
                let j = refine::Jitter::from_seed(seed);
                out.note = format!("hue {:+.1}°, saturation ×{:.3}, value ×{:.3}", j.hue_deg, j.sat, j.val);
                out.jitter = Some(j);
                Some((refine::tint_jitter(&src, j), Some(item.transform.clone())))
            }
            RefineVerb::Harmonise => {
                let h = refine::harmonise(&src, palette).map_err(|r| r.text().to_string())?;
                out.note = format!("mean hue {:.0}° moved to {:.0}° (the Look's nearest swatch)", h.hue_before, h.hue_after);
                let img = h.image.clone();
                out.harmonised = Some(h);
                Some((img, Some(item.transform.clone())))
            }
            RefineVerb::Outline | RefineVerb::Shadow => {
                let Some(bake) = self.sprite_bake(uid, index) else { return Err(format!("{uid} variant {} has no pixels", index + 1)) };
                if !refine::has_ink(&bake) {
                    out.note = "nothing to draw around: the art is blank".to_string();
                    None
                } else if verb == RefineVerb::Outline {
                    out.note = format!("{} px outline drawn behind the art at the {}² bake", refine::OUTLINE_PX, family.size());
                    Some((refine::outline(&bake), Some(ItemTransform::default())))
                } else {
                    out.note = format!("drop shadow drawn behind the art at the {}² bake", family.size());
                    Some((refine::drop_shadow(&bake), Some(ItemTransform::default())))
                }
            }
        };
        let Some((img, transform)) = made else { return Ok(out) };
        let name = format!("{} · {}", item.name, verb.suffix());
        self.insert_decoded(uid, name, img)?;
        let new_index = self.db.items(uid).len() - 1;
        if let Some(t) = transform
            && let Some(it) = self.db.item_mut(uid, new_index)
        {
            it.transform = t;
        }
        out.index = Some(new_index);
        Ok(out)
    }

    // -- Thumbnails ----------------------------------------------------------

    /// A `render_item`-baked, PNG-encoded thumbnail for one stored item —
    /// the shared render core the reference itself uses for grid
    /// thumbnails, the inspector preview, and export bake alike. `None` when
    /// the uid/index/pixels don't all resolve, or if re-encoding fails.
    pub fn thumbnail_png(&self, uid: &str, index: usize, size: u32) -> Option<Vec<u8>> {
        let slot = self.db.get(uid)?;
        let item = self.db.items(uid).get(index)?;
        let img = self.image(uid, index)?;
        let baked = render_item(img, &item.transform, size, slot.family.opaque());
        encode_png(&baked).ok()
    }

    // -- Batch operations (`AssetBrowserUI.init`'s `alBatch*` handlers) -----

    /// `alBatchTag`: append every tag in `tags` (deduplicated per slot) to
    /// each of `uids`' `SlotMeta::tags`. Unknown uids are skipped, not an
    /// error — a batch op over a stale selection should apply to whatever
    /// still exists.
    pub fn batch_tag(&mut self, uids: &[String], tags: &[String]) -> usize {
        let mut tagged = 0usize;
        for uid in uids {
            let Some(meta) = self.db.slot_meta_mut(uid) else { continue };
            for t in tags {
                if !t.is_empty() && !meta.tags.contains(t) {
                    meta.tags.push(t.clone());
                }
            }
            tagged += 1;
        }
        tagged
    }

    /// `alBatchColl`: add every uid in `uids` to collection `name` —
    /// `AssetCollections::add`'s own blank-name-is-a-no-op and
    /// no-duplicate-membership rules apply unchanged.
    pub fn batch_collect(&mut self, name: &str, uids: &[String]) {
        self.db.collections.add(name, uids);
    }

    /// `alBatchRename`: `uids` become `{base}_01`, `{base}_02`, … in
    /// selection order. A custom slot is renamed via
    /// `AssetDB::rename_custom_slot` (one `_NN` per slot); a frozen slot
    /// instead renames each of its *item variants* in place via
    /// [`cartalith_assets::AssetDB::item_mut`] (`_NN` advances by
    /// `max(1, item_count)`), matching the reference's own
    /// `store[uid].forEach((it,i)=>{it.name=...})` — frozen slots are not
    /// nameable at all (`slot_title` is a constant), only their variants
    /// are. Returns `(renamed_slot_count, remap)`, where `remap` carries
    /// `old_uid -> new_uid` for every custom slot whose uid changed (a
    /// caller's selection set needs to follow it).
    pub fn batch_rename(&mut self, uids: &[String], base: &str) -> (usize, Vec<(String, String)>) {
        let mut n = 1usize;
        let mut renamed = 0usize;
        let mut remap = Vec::new();
        for uid in uids {
            let Some(slot) = self.db.get(uid) else { continue };
            if slot.family == Family::Custom {
                let new_name = format!("{base}_{n:02}");
                let nuid = self.db.rename_custom_slot(uid, &new_name);
                if &nuid != uid {
                    if let Some(v) = self.images.remove(uid) {
                        self.images.insert(nuid.clone(), v);
                    }
                    remap.push((uid.clone(), nuid));
                }
                n += 1;
            } else {
                let count = self.db.items(uid).len().max(1);
                for i in 0..self.db.items(uid).len() {
                    let name = format!("{base}_{:02}", n + i);
                    if let Some(item) = self.db.item_mut(uid, i) {
                        item.name = name;
                    }
                }
                n += count;
            }
            renamed += 1;
        }
        (renamed, remap)
    }

    /// `alBatchDup`: for each of `uids` carrying at least one item, a new
    /// custom slot named `"{slot name} copy"` under the `"Duplicates"` set,
    /// with every source item (and its decoded pixels) cloned in. Returns
    /// how many slots were duplicated.
    pub fn batch_duplicate(&mut self, uids: &[String]) -> usize {
        let mut made = 0usize;
        for uid in uids {
            let Some(slot) = self.db.get(uid) else { continue };
            if self.db.items(uid).is_empty() {
                continue;
            }
            let new_name = format!("{} copy", slot.name);
            let new_uid = self.db.add_custom_slot(&new_name, Some("Duplicates")).uid.clone();
            let items: Vec<LibraryItem> = self.db.items(uid).to_vec();
            let pixels: Vec<DecodedImage> = self.images.get(uid).cloned().unwrap_or_default();
            for item in items {
                self.db.add_item(&new_uid, item);
            }
            self.images.entry(new_uid).or_default().extend(pixels);
            made += 1;
        }
        made
    }

    /// `alBatchDel`: for each of `uids`, remove the whole slot if it's
    /// custom, or just empty its items if it's frozen (a frozen slot can
    /// never be removed). Returns how many slots were affected.
    pub fn batch_delete(&mut self, uids: &[String]) -> usize {
        let mut deleted = 0usize;
        for uid in uids {
            let Some(slot) = self.db.get(uid) else { continue };
            if slot.family == Family::Custom {
                if self.remove_custom_slot(uid) {
                    deleted += 1;
                }
            } else {
                self.clear_slot_items(uid);
                deleted += 1;
            }
        }
        deleted
    }

    // -- Export / apply ------------------------------------------------------

    /// `PackManifestBuilder.build()` + `zipStore`: bake every stored item at
    /// its family's own size/opacity (`render_item`), PNG-encode it, place
    /// it at the exporter's own path convention (`Family::asset_path`), and
    /// write the whole thing out as a pack `.zip` via
    /// `archive::write_pack`. `Err("Library is empty.")` when nothing is
    /// stored at all — the reference's own `AssetDB.totalItems()===0` guard
    /// on both `exportPack` and `applyToMap`. Pack name/author/license fall
    /// back to the reference's own defaults (`'Custom Asset Pack'`/
    /// `'Cartalith Gen1'`/`'CC0'`) when blank. Returns `(pack_name, bytes)`.
    pub fn export_pack_bytes(&mut self) -> Result<(String, Vec<u8>), String> {
        if self.db.total_items() == 0 {
            return Err("Library is empty.".to_string());
        }
        let name = if self.db.pack.name.trim().is_empty() {
            "Custom Asset Pack".to_string()
        } else {
            self.db.pack.name.trim().to_string()
        };
        let author = if self.db.pack.author.trim().is_empty() {
            "Cartalith Gen1".to_string()
        } else {
            self.db.pack.author.trim().to_string()
        };
        let license = if self.db.pack.license.trim().is_empty() {
            "CC0".to_string()
        } else {
            self.db.pack.license.trim().to_string()
        };

        let mut manifest = PackManifest { name: name.clone(), author, license, ..Default::default() };
        let mut images = PackEntries::new();

        for uid in self.db.uids_in_order() {
            let Some(slot) = self.db.get(&uid).cloned() else { continue };
            let items = self.db.items(&uid);
            if items.is_empty() {
                continue;
            }
            let family = slot.family;
            let set_id = slot.set_id.clone().unwrap_or_default();
            let mut paths = Vec::new();
            for (i, item) in items.iter().enumerate() {
                let Some(pixels) = self.image(&uid, i) else { continue };
                let baked = render_item(pixels, &item.transform, family.size(), family.opaque());
                let png = encode_png(&baked).map_err(|e| e.to_string())?;
                let path = family.asset_path(&slot.id, &set_id, i);
                images.insert(path.clone(), png);
                paths.push(path);
            }
            if paths.is_empty() {
                continue;
            }
            match family {
                Family::Textures => {
                    manifest.textures.insert(slot.id.clone(), paths.into_iter().next().expect("non-empty"));
                }
                Family::Biomes => {
                    manifest.biomes.insert(slot.id.clone(), paths.into_iter().next().expect("non-empty"));
                }
                Family::Terrains => {
                    manifest.terrains.insert(slot.id.clone(), paths.into_iter().next().expect("non-empty"));
                }
                Family::Icons => {
                    manifest.icons.insert(slot.id.clone(), paths);
                }
                // The sea-marks family (owner ruling 2026-09-02,
                // `slots::PACK_SEAMARK_SLOTS`). One arm, in a file this lane
                // does not otherwise own: without it a Library holding sea-mark
                // art would export its PNGs into the zip and then declare none
                // of them, which is silent data loss rather than a gap.
                Family::SeaMark => {
                    manifest.seamarks.insert(slot.id.clone(), paths);
                }
                Family::Settlement => {
                    manifest.structures.settlement.insert(slot.id.clone(), paths);
                }
                Family::Trait => {
                    manifest.structures.traits.insert(slot.id.clone(), paths);
                }
                Family::Poi => {
                    manifest.structures.poi.insert(slot.id.clone(), paths);
                }
                Family::Custom => {
                    let set_name = slot.set.clone().unwrap_or_default();
                    if manifest.custom.get(&set_name).is_none() {
                        manifest.custom.insert(set_name.clone(), Default::default());
                    }
                    manifest.custom.get_mut(&set_name).expect("just inserted").insert(slot.id.clone(), paths);
                }
            }
        }

        let mut buf = Vec::new();
        archive::write_pack(std::io::Cursor::new(&mut buf), &manifest, &images).map_err(|e| e.to_string())?;
        Ok((name, buf))
    }
}

/// The unit tests of the asset session. They run without a Godot runtime (this module
/// is `godot`-free) over real PNG bytes encoded by `png_bytes`.
#[cfg(test)]
mod tests {
    use super::*;

    /// A `w` by `h` PNG filled with one `rgba` colour, so a test can import real,
    /// decodable bytes with a recognisable pixel value.
    fn png_bytes(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        let img = DecodedImage::new(w, h, data).unwrap();
        encode_png(&img).unwrap()
    }

    /// Protects: importing into a bottom-anchored frozen slot stores both the item and its
    /// pixels and runs `fit_to_bottom` (a non-square image gets a non-zero `pan_y`).
    #[test]
    fn import_item_into_a_frozen_slot_fits_to_bottom_and_stores_pixels() {
        let mut s = AssetLibrarySession::new();
        // Wider than tall, so `w` (not `h`) is the longer side: `fit_to_bottom`
        // then leaves genuine vertical slack to centre away from zero.
        s.import_item("icons:mountain", "m1.png".to_string(), &png_bytes(16, 8, [1, 2, 3, 255])).unwrap();
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        assert!(s.image("icons:mountain", 0).is_some());
        // fit_to_bottom mutates pan_y away from the identity default for a
        // non-square item on a bottom-anchored family.
        assert_ne!(s.db.items("icons:mountain")[0].transform.pan_y, 0.0);
    }

    /// A `w`x`h` PNG whose red channel ramps left to right -- a ground tile
    /// with a real left-right seam (255 levels across the wrap, ~1 across
    /// the centre).
    fn ramp_png(w: u32, h: u32) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..h {
            for x in 0..w {
                data.extend_from_slice(&[(x * 255 / (w - 1)) as u8, 80, 40, 255]);
            }
        }
        encode_png(&DecodedImage::new(w, h, data).unwrap()).unwrap()
    }

    /// Protects: AP-4 through the session -- make-seamless on a seamed ground
    /// tile adds exactly one new variant (Ruling CQ Q7), leaves the original's
    /// pixels and record untouched, stores pixels whose own bake is
    /// themselves (so export ships what was measured) and which meet the
    /// <= 1.25x bar; undo is `remove_item` on the returned index.
    #[test]
    fn make_seamless_adds_a_new_variant_and_keeps_the_original() {
        let mut s = AssetLibrarySession::new();
        s.import_item("textures:grass", "g.png".to_string(), &ramp_png(512, 512)).unwrap();
        let before_px = s.image("textures:grass", 0).unwrap().clone();
        let before_rec = s.db.items("textures:grass")[0].clone();
        assert!(s.seam_check("textures:grass", 0).unwrap().is_seam());
        let (out, idx) = s.make_seamless_variant("textures:grass", 0).unwrap();
        assert_eq!(idx, Some(1));
        assert!(out.fixed_x && !out.fixed_y);
        assert_eq!(s.db.items("textures:grass").len(), 2);
        assert_eq!(s.image("textures:grass", 0).unwrap(), &before_px, "original pixels untouched");
        assert_eq!(s.db.items("textures:grass")[0], before_rec, "original record untouched");
        assert_eq!(s.db.items("textures:grass")[1].name, "g.png · seamless");
        let after = s.seam_check("textures:grass", 1).unwrap();
        assert!(after.edge <= cartalith_assets::SEAM_RATIO * after.interior, "{after:?}");
        assert!(!after.is_seam());
        let stored = s.image("textures:grass", 1).unwrap().clone();
        assert_eq!(s.texture_bake("textures:grass", 1).unwrap(), stored, "the new variant's bake is its own pixels");
        assert_eq!(Some(&stored), out.image.as_ref());
        assert!(s.remove_item("textures:grass", 1));
        assert_eq!(s.db.items("textures:grass").len(), 1);
    }

    /// Protects: a tile that already wraps adds nothing and changes nothing
    /// (byte-identical); a sprite family and an unknown index are refused in
    /// words, never a panic.
    #[test]
    fn make_seamless_leaves_a_seamless_tile_and_refuses_sprites() {
        let mut s = AssetLibrarySession::new();
        s.import_item("biomes:jungle", "j.png".to_string(), &png_bytes(512, 512, [30, 120, 40, 255])).unwrap();
        let px = s.image("biomes:jungle", 0).unwrap().clone();
        let (out, idx) = s.make_seamless_variant("biomes:jungle", 0).unwrap();
        assert_eq!((idx, out.image.is_none()), (None, true));
        assert_eq!(s.db.items("biomes:jungle").len(), 1);
        assert_eq!(s.image("biomes:jungle", 0).unwrap(), &px);
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(64, 64, [1, 2, 3, 255])).unwrap();
        assert!(s.make_seamless_variant("icons:mountain", 0).unwrap_err().contains("not a ground texture"));
        assert!(s.seam_check("icons:mountain", 0).is_none());
        assert!(s.make_seamless_variant("biomes:jungle", 5).is_err());
        assert!(s.make_seamless_variant("textures:nope", 0).is_err());
    }

    /// A 1024² white page with black strokes, as PNG bytes: a line drawing
    /// an author would import (AP-10's tests).
    fn line_drawing_png() -> Vec<u8> {
        let n = 1024u32;
        let mut data = vec![255u8; (n * n * 4) as usize];
        for y in 0..n {
            for x in 0..n {
                let on = ((200..824).contains(&x) && (300..306).contains(&y)) || ((500..506).contains(&x) && (300..900).contains(&y));
                if on {
                    let i = ((y * n + x) * 4) as usize;
                    data[i..i + 3].copy_from_slice(&[10, 10, 10]);
                }
            }
        }
        encode_png(&DecodedImage::new(n, n, data).unwrap()).unwrap()
    }

    /// Protects AP-10 through the session: the conversion adds exactly one
    /// variant named `· line art` (Ruling CQ Q7), the original's pixels and
    /// record are untouched, the new variant is one ink colour at the family
    /// size, and its own bake (what export writes) equals its pixels except
    /// for `render_item`'s known one-level RGB drift on soft alpha (AP-1's
    /// finding, `ASSET_PACK_TOOLS_PROPOSAL.md` §10): alpha is exact and RGB
    /// moves at most one level. Undo is `remove_item`.
    #[test]
    fn line_art_adds_a_new_variant_and_keeps_the_original() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "peak.png".to_string(), &line_drawing_png()).unwrap();
        let before_px = s.image("icons:mountain", 0).unwrap().clone();
        let before_rec = s.db.items("icons:mountain")[0].clone();
        let opts = LineArtOptions { ink: [40, 30, 20], ..LineArtOptions::default() };
        let (r, idx) = s.line_art_variant("icons:mountain", 0, &opts).unwrap();
        assert_eq!(idx, 1);
        assert!(r.image.rgba.is_empty(), "the image is moved into the store, not duplicated");
        assert_eq!(r.clean.components, 1);
        assert!(!r.clean.closed, "a clean drawing is not a scan");
        assert_eq!(s.db.items("icons:mountain").len(), 2);
        assert_eq!(s.image("icons:mountain", 0).unwrap(), &before_px, "original pixels untouched");
        assert_eq!(s.db.items("icons:mountain")[0], before_rec, "original record untouched");
        let rec = &s.db.items("icons:mountain")[1];
        assert_eq!(rec.name, "peak.png · line art");
        assert_eq!(rec.transform, ItemTransform::default());
        let made = s.image("icons:mountain", 1).unwrap().clone();
        assert_eq!((made.w, made.h), (256, 256));
        assert!(made.rgba.chunks_exact(4).all(|p| p[3] == 0 || p[..3] == [40, 30, 20]));
        let bake = render_item(&made, &rec.transform, 256, false);
        for (a, b) in made.rgba.chunks_exact(4).zip(bake.rgba.chunks_exact(4)) {
            assert_eq!(a[3], b[3], "alpha moved in the bake");
            if a[3] > 0 {
                assert!((0..3).all(|c| a[c].abs_diff(b[c]) <= 1), "{a:?} vs {b:?}");
            }
        }
        assert!(s.remove_item("icons:mountain", 1));
        assert_eq!(s.db.items("icons:mountain").len(), 1);
    }

    /// Protects AP-10's refusals: ground textures, blank art, unknown slots
    /// and indexes answer in words and add nothing.
    #[test]
    fn line_art_refuses_textures_blank_art_and_bad_indexes() {
        let mut s = AssetLibrarySession::new();
        let o = LineArtOptions::default();
        s.import_item("biomes:jungle", "j.png".to_string(), &png_bytes(512, 512, [30, 120, 40, 255])).unwrap();
        assert!(s.line_art_variant("biomes:jungle", 0, &o).unwrap_err().contains("ground texture"));
        s.import_item("icons:mountain", "blank.png".to_string(), &png_bytes(64, 64, [255, 255, 255, 255])).unwrap();
        assert!(s.line_art_variant("icons:mountain", 0, &o).unwrap_err().contains("no ink"));
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        assert!(s.line_art_variant("icons:mountain", 3, &o).is_err());
        assert!(s.line_art_variant("icons:nope", 0, &o).is_err());
    }

    /// Protects: a settlement-family import is not fitted to the bottom: its `pan_y` stays 0.
    #[test]
    fn import_item_into_settlement_family_does_not_fit_to_bottom() {
        let mut s = AssetLibrarySession::new();
        s.import_item("settlement:hamlet", "h.png".to_string(), &png_bytes(16, 8, [1, 2, 3, 255])).unwrap();
        assert_eq!(s.db.items("settlement:hamlet")[0].transform.pan_y, 0.0);
    }

    /// Protects: an unknown uid and bytes that are not a PNG both return `Err`, never a panic.
    #[test]
    fn import_item_rejects_unknown_uid_and_bad_bytes_without_panicking() {
        let mut s = AssetLibrarySession::new();
        assert!(s.import_item("nope:nope", "x.png".to_string(), &png_bytes(4, 4, [0, 0, 0, 255])).is_err());
        assert!(s.import_item("icons:mountain", "x.png".to_string(), b"not a png").is_err());
    }

    /// Protects: removing an item removes its pixels too (the survivor keeps its own colour and
    /// name), and an out-of-range index returns `false`.
    #[test]
    fn remove_item_keeps_both_stores_in_lockstep() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        s.import_item("icons:mountain", "b.png".to_string(), &png_bytes(4, 4, [0, 1, 0, 255])).unwrap();
        assert!(s.remove_item("icons:mountain", 0));
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        assert_eq!(s.db.items("icons:mountain")[0].name, "b.png");
        assert_eq!(s.image("icons:mountain", 0).unwrap().rgba[1], 1);
        assert!(!s.remove_item("icons:mountain", 5));
    }

    /// Protects: batch delete removes a custom slot entirely but only empties a frozen slot,
    /// items and pixels both, and counts both as affected.
    #[test]
    fn batch_delete_empties_a_frozen_slot_but_removes_a_custom_one() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        let cuid = s.add_custom_slot("Lighthouse", Some("Naval"));
        s.import_item(&cuid, "l.png".to_string(), &png_bytes(4, 4, [0, 1, 0, 255])).unwrap();

        let n = s.batch_delete(&[cuid.clone(), "icons:mountain".to_string()]);
        assert_eq!(n, 2);
        assert!(s.db.get(&cuid).is_none(), "custom slot removed entirely");
        assert!(s.db.get("icons:mountain").is_some(), "frozen slot survives");
        assert!(s.db.items("icons:mountain").is_empty(), "but its items are gone");
        assert!(s.image("icons:mountain", 0).is_none());
    }

    /// Protects: batch rename numbers a frozen slot's item variants in place (`Base_01`,
    /// `Base_02`), renames a custom slot (`Base_03`) and reports its uid change, and
    /// the pixel store follows the new uid.
    #[test]
    fn batch_rename_renames_custom_slots_and_frozen_item_variants_differently() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        s.import_item("icons:mountain", "b.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        let cuid = s.add_custom_slot("Lighthouse", Some("Naval"));
        s.import_item(&cuid, "l.png".to_string(), &png_bytes(4, 4, [0, 1, 0, 255])).unwrap();

        let (n, remap) = s.batch_rename(&["icons:mountain".to_string(), cuid.clone()], "Base");
        assert_eq!(n, 2);
        assert_eq!(s.db.items("icons:mountain")[0].name, "Base_01");
        assert_eq!(s.db.items("icons:mountain")[1].name, "Base_02");
        assert_eq!(remap.len(), 1);
        let new_uid = &remap[0].1;
        assert_eq!(s.db.get(new_uid).unwrap().name, "Base_03");
        assert!(s.image(new_uid, 0).is_some(), "pixel store followed the uid change");
    }

    /// Protects: batch duplicate makes a `... copy` custom slot holding a clone of the item and
    /// of its pixels.
    #[test]
    fn batch_duplicate_clones_items_and_pixels_into_a_new_custom_slot() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [9, 9, 9, 255])).unwrap();
        let made = s.batch_duplicate(&["icons:mountain".to_string()]);
        assert_eq!(made, 1);
        let dup_uid = s
            .db
            .slots_in_family(Family::Custom)
            .into_iter()
            .find(|s| s.name == "Mountain copy")
            .unwrap()
            .uid
            .clone();
        assert_eq!(s.db.items(&dup_uid).len(), 1);
        assert_eq!(s.image(&dup_uid, 0).unwrap().rgba[0], 9);
    }

    /// Protects: batch tag adds each tag once per slot even when it is passed twice, in order.
    #[test]
    fn batch_tag_deduplicates_per_slot() {
        let mut s = AssetLibrarySession::new();
        let n = s.batch_tag(
            &["icons:mountain".to_string()],
            &["tall".to_string(), "tall".to_string(), "rocky".to_string()],
        );
        assert_eq!(n, 1);
        assert_eq!(s.db.get("icons:mountain").unwrap().meta.tags, vec!["tall", "rocky"]);
    }

    /// Protects: exporting an empty library is an `Err` (the reference's empty-library guard).
    #[test]
    fn export_pack_bytes_fails_honestly_on_an_empty_library() {
        let mut s = AssetLibrarySession::new();
        assert!(s.export_pack_bytes().is_err());
    }

    /// Protects: an exported pack reads back with its name, icon and texture entries at the
    /// exporter's own paths and no warnings.
    #[test]
    fn export_pack_bytes_round_trips_through_read_pack() {
        let mut s = AssetLibrarySession::new();
        s.db.pack.name = "My Pack".to_string();
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(8, 8, [10, 20, 30, 255])).unwrap();
        s.import_item("textures:grass", "g.png".to_string(), &png_bytes(8, 8, [40, 50, 60, 255])).unwrap();
        let (name, bytes) = s.export_pack_bytes().unwrap();
        assert_eq!(name, "My Pack");

        let (manifest, _entries) = cartalith_assets::read_pack(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(manifest.name, "My Pack");
        assert_eq!(manifest.icons.get("mountain").unwrap().len(), 1);
        assert_eq!(manifest.textures.get("grass").unwrap(), "textures/grass.png");
        assert!(manifest.warnings.is_empty());
    }

    /// Protects: a thumbnail is a real PNG of exactly the requested size, whatever the source size.
    #[test]
    fn thumbnail_png_bakes_a_real_size_by_size_image() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(8, 8, [1, 2, 3, 255])).unwrap();
        let png = s.thumbnail_png("icons:mountain", 0, 32).unwrap();
        let decoded = decode_png(&png).unwrap();
        assert_eq!((decoded.w, decoded.h), (32, 32));
    }

    // -- Sprite-sheet slicer ---------------------------------------------

    /// A 4×2 sheet cut 2×1: the left cell opaque, the right cell fully
    /// transparent — so "skip empty cells" has something real to skip, and
    /// the non-empty count has a number other than the cell count.
    fn half_blank_sheet() -> Vec<u8> {
        let mut rgba = Vec::new();
        for _y in 0..2 {
            for x in 0..4u32 {
                rgba.extend_from_slice(if x < 2 { &[7, 8, 9, 255] } else { &[0, 0, 0, 0] });
            }
        }
        encode_png(&DecodedImage::new(4, 2, rgba).unwrap()).unwrap()
    }

    /// The default slicer settings for a `cols` by `rows` grid: no margin, spacing,
    /// chroma key, trim, custom lines or scoped cell, and skip-blank on (the
    /// reference's own default for `#alSlSkip`).
    fn params(cols: i64, rows: i64) -> SliceParams {
        SliceParams { cols, rows, margin: 0.0, spacing: 0.0, chroma: None, trim: false, skip_blank: true, col_lines: None, row_lines: None, only_cell: None }
    }

    /// Protects: the preview's counts come from the real detection pass (2 cells, 1 non-empty)
    /// and its column/row spans are the engine's own.
    #[test]
    fn slice_preview_reports_the_real_non_empty_count_not_a_sample() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("towns-sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let p = s.slice_preview(&params(2, 1)).unwrap();
        assert_eq!((p.counts.total, p.counts.non_empty, p.counts.usable), (2, 1, true));
        // The overlay spans come back engine-computed, one per column/row.
        assert_eq!(p.col_spans, vec![(0.0, 2.0), (2.0, 4.0)]);
        assert_eq!(p.row_spans, vec![(0.0, 2.0)]);
    }

    /// Protects: previewing with no sheet loaded is an `Err`, not a panic.
    #[test]
    fn slice_preview_without_a_sheet_is_an_error_not_a_panic() {
        let s = AssetLibrarySession::new();
        assert!(s.slice_preview(&params(2, 1)).is_err());
    }

    /// Protects: bytes that are not a PNG make `load_sheet` return `Err`.
    #[test]
    fn load_sheet_rejects_non_png_bytes_without_panicking() {
        let mut s = AssetLibrarySession::new();
        assert!(s.load_sheet("x.gif".to_string(), b"not a png").is_err());
    }

    /// Protects: slicing into a multi-variant slot adds each non-blank cell, counts the blank one
    /// as skipped, and names the item `<sheet>_r1c1`.
    #[test]
    fn slicing_into_a_multi_family_slot_adds_every_non_blank_cell() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("towns-sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let out = s
            .apply_slice(&params(2, 1), &SliceTarget::Slot { uid: "icons:mountain".to_string() })
            .unwrap();
        assert_eq!((out.added, out.skipped_blank, out.unplaced), (1, 1, 0));
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        // The reference's own `base_r1c1` naming, base = filename sans ext.
        assert_eq!(s.db.items("icons:mountain")[0].name, "towns-sheet_r1c1");
        assert_eq!(s.image("icons:mountain", 0).unwrap().rgba[0], 7);
    }

    /// Protects: a single-image family takes only the first cell and replaces its old item
    /// (`store[uid]=[item]`), reporting the other cell as unplaced.
    #[test]
    fn slicing_into_a_single_image_family_takes_the_first_cell_and_replaces() {
        // `textures` is not multi: the reference does `store[uid]=[item]` and
        // breaks after the first cell.
        let mut s = AssetLibrarySession::new();
        s.import_item("textures:grass", "old.png".to_string(), &png_bytes(4, 4, [1, 1, 1, 255])).unwrap();
        s.load_sheet("t.png".to_string(), &half_blank_sheet()).unwrap();
        let out = s
            .apply_slice(
                &SliceParams { skip_blank: false, ..params(2, 1) },
                &SliceTarget::Slot { uid: "textures:grass".to_string() },
            )
            .unwrap();
        assert_eq!((out.added, out.unplaced), (1, 1));
        assert_eq!(s.db.items("textures:grass").len(), 1, "the old item was replaced, not appended to");
        assert_eq!(s.db.items("textures:grass")[0].name, "t_r1c1");
    }

    /// Protects: per-cell slicing makes one custom slot per cell, named `cell 1`, `cell 2`.
    #[test]
    fn slicing_per_cell_makes_one_custom_slot_per_cell() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let out = s
            .apply_slice(&SliceParams { skip_blank: false, ..params(2, 1) }, &SliceTarget::PerCell { set: "Towns".to_string() })
            .unwrap();
        assert_eq!(out.added, 2);
        assert_eq!(out.uids.len(), 2);
        let names: Vec<&str> = out.uids.iter().map(|u| s.db.get(u).unwrap().name.as_str()).collect();
        assert_eq!(names, vec!["cell 1", "cell 2"]);
    }

    /// Protects: slicing to a new custom slot stacks every cell as a variant of that one slot.
    #[test]
    fn slicing_to_a_new_custom_slot_stacks_every_cell_as_a_variant() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let out = s
            .apply_slice(
                &SliceParams { skip_blank: false, ..params(2, 1) },
                &SliceTarget::NewCustom { name: "Windmill".to_string(), set: String::new() },
            )
            .unwrap();
        assert_eq!((out.added, out.uids.len()), (2, 1));
        assert_eq!(s.db.items(&out.uids[0]).len(), 2);
    }

    /// Protects: a first-empty family fill leaves a slot that already has art alone and fills
    /// the following empty slots.
    #[test]
    fn family_fill_from_first_empty_skips_slots_that_already_have_art() {
        let mut s = AssetLibrarySession::new();
        // `settlement`'s first slot is already filled, so a first-empty fill
        // must start at the second one and leave the first alone.
        s.import_item("settlement:hamlet", "keep.png".to_string(), &png_bytes(4, 4, [3, 3, 3, 255])).unwrap();
        s.load_sheet("pins.png".to_string(), &half_blank_sheet()).unwrap();
        let out = s
            .apply_slice(
                &SliceParams { skip_blank: false, ..params(2, 1) },
                &SliceTarget::Family { family: Family::Settlement, overwrite: false },
            )
            .unwrap();
        assert_eq!(out.added, 2);
        assert_eq!(s.db.items("settlement:hamlet")[0].name, "keep.png", "the filled slot was untouched");
        assert_eq!(s.db.items("settlement:village").len(), 1);
        assert_eq!(s.db.items("settlement:town").len(), 1);
    }

    /// Protects: an overwrite family fill starts at the first slot and replaces what is there.
    #[test]
    fn family_fill_with_overwrite_starts_at_the_first_slot_and_replaces() {
        let mut s = AssetLibrarySession::new();
        s.import_item("settlement:hamlet", "old.png".to_string(), &png_bytes(4, 4, [3, 3, 3, 255])).unwrap();
        s.load_sheet("pins.png".to_string(), &half_blank_sheet()).unwrap();
        s.apply_slice(
            &SliceParams { skip_blank: false, ..params(2, 1) },
            &SliceTarget::Family { family: Family::Settlement, overwrite: true },
        )
        .unwrap();
        assert_eq!(s.db.items("settlement:hamlet").len(), 1);
        assert_eq!(s.db.items("settlement:hamlet")[0].name, "pins_r1c1");
    }

    /// Protects: slicing is non-destructive (`DCC_SHELL_SPEC.md` §8): the same sheet can be
    /// sliced again with different settings and previewed afterwards.
    #[test]
    fn a_slice_never_consumes_the_sheet() {
        // Non-destructive per DCC_SHELL_SPEC.md §8: the same sheet slices
        // twice, with different settings, without being reloaded.
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        s.apply_slice(&params(2, 1), &SliceTarget::Slot { uid: "icons:mountain".to_string() }).unwrap();
        let again = s
            .apply_slice(&params(1, 1), &SliceTarget::Slot { uid: "icons:hill".to_string() })
            .unwrap();
        assert_eq!(again.added, 1);
        assert!(s.slice_preview(&params(2, 1)).is_ok());
    }

    /// Protects: an all-blank sheet and a too-dense grid each give a distinct `Err` (naming
    /// emptiness and density) instead of silently adding nothing.
    #[test]
    fn a_slice_that_would_add_nothing_is_an_error_rather_than_a_silent_no_op() {
        let mut s = AssetLibrarySession::new();
        // A fully transparent sheet with skip-empty on: nothing to add.
        let blank = encode_png(&DecodedImage::new(4, 2, vec![0u8; 4 * 2 * 4]).unwrap()).unwrap();
        s.load_sheet("blank.png".to_string(), &blank).unwrap();
        let err = s
            .apply_slice(&params(2, 1), &SliceTarget::Slot { uid: "icons:mountain".to_string() })
            .unwrap_err();
        assert!(err.contains("empty"), "{err}");
        // And a too-dense grid says so instead.
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let dense = SliceParams { spacing: 500.0, skip_blank: false, ..params(2, 1) };
        assert!(s.apply_slice(&dense, &SliceTarget::PerCell { set: String::new() }).unwrap_err().contains("dense"));
    }

    /// Protects: a margin larger than the sheet is an `Err` from both preview and slice.
    #[test]
    fn an_impossible_margin_is_an_error_not_a_negative_grid() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let p = SliceParams { margin: 40.0, ..params(2, 1) };
        assert!(s.slice_preview(&p).is_err());
        assert!(s.apply_slice(&p, &SliceTarget::PerCell { set: String::new() }).is_err());
    }

    /// Protects: slicing into an unknown slot uid is an `Err`, not a panic.
    #[test]
    fn slicing_into_an_unknown_slot_is_an_error_not_a_panic() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        assert!(s.apply_slice(&params(2, 1), &SliceTarget::Slot { uid: "nope:nope".to_string() }).is_err());
    }

    // -- AS-17: cell-scoped slicing / draggable interior lines -------------

    /// Protects: a scoped slice (AS-17, `only_cell`) lands only that one cell, not the whole grid.
    #[test]
    fn only_cell_narrows_the_slice_to_one_cell_out_of_the_grid() {
        let mut s = AssetLibrarySession::new();
        // 2x1, cell 0 opaque, cell 1 blank (`half_blank_sheet`).
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet())
            .unwrap();
        let p = SliceParams {
            only_cell: Some(0),
            skip_blank: false,
            ..params(2, 1)
        };
        let out = s
            .apply_slice(&p, &SliceTarget::PerCell { set: String::new() })
            .unwrap();
        assert_eq!(
            out.added, 1,
            "only the scoped cell landed, not the whole 2-cell grid"
        );
    }

    /// Protects: scoping to a blank cell with skip-blank on is an `Err` that names the selected cell.
    #[test]
    fn only_cell_pointing_at_a_blank_cell_is_an_honest_error() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet())
            .unwrap();
        // Cell 1 is blank and skip_blank defaults true in `params`, so nothing
        // in the scoped selection survives.
        let p = SliceParams {
            only_cell: Some(1),
            ..params(2, 1)
        };
        let err = s
            .apply_slice(&p, &SliceTarget::PerCell { set: String::new() })
            .unwrap_err();
        assert!(err.contains("selected cell"), "{err}");
    }

    /// Protects: a scoped cell index beyond the grid is an `Err`, not a panic.
    #[test]
    fn only_cell_out_of_range_for_the_current_grid_is_an_error_not_a_panic() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet())
            .unwrap();
        let p = SliceParams {
            only_cell: Some(99),
            skip_blank: false,
            ..params(2, 1)
        };
        assert!(
            s.apply_slice(&p, &SliceTarget::PerCell { set: String::new() })
                .is_err()
        );
    }

    /// Protects: custom column lines (AS-17) reach the engine grid: moving the interior line to
    /// 0.75 moves the cut to 3 px.
    #[test]
    fn col_lines_reach_the_engine_grid_and_change_where_cells_are_cut() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet())
            .unwrap(); // 4x2
        // Default 2x1 split is at x=2; dragging the (only) interior line to
        // .75 moves the cut to x=3, so cell 0 is now 3px wide, not 2.
        let p = SliceParams {
            col_lines: Some(vec![0.0, 0.75, 1.0]),
            skip_blank: false,
            ..params(2, 1)
        };
        let preview = s.slice_preview(&p).unwrap();
        assert_eq!(preview.col_spans[0], (0.0, 3.0));
        assert_eq!(preview.col_spans[1], (3.0, 4.0));
    }

    /// Protects: a stale column-line array of the wrong length falls back to a uniform split
    /// instead of erroring.
    #[test]
    fn col_lines_of_the_wrong_length_fall_back_to_uniform_rather_than_erroring() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet())
            .unwrap();
        // Stale 3-line array (from a previous cols=2) against a cols=3 grid.
        let p = SliceParams {
            cols: 3,
            col_lines: Some(vec![0.0, 0.75, 1.0]),
            skip_blank: false,
            ..params(2, 1)
        };
        let preview = s.slice_preview(&p).unwrap();
        assert_eq!(preview.col_spans.len(), 3);
        assert_eq!(preview.col_spans[0], (0.0, 4.0 / 3.0));
    }

    // -- AP-1: opening a pack into the library ----------------------------

    /// The reference pack fixture `cartalith-assets` ships (17 PNGs and a
    /// schema-2 `pack.json`; `ASSET_PACK_TOOLS_PROPOSAL.md` §1.5).
    fn reference_pack() -> Vec<u8> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../cartalith-assets/tests/fixtures/reference_pack.zip");
        std::fs::read(path).expect("reference_pack.zip fixture")
    }

    /// Everything a pack round trip must preserve, per filled slot in store
    /// order: uid, family, custom set, metadata, and each item's name,
    /// transform and decoded RGBA. Pack info rides along. **Asserts it is not
    /// empty**, so a helper mutated to return nothing cannot pass an
    /// equality check vacuously.
    #[allow(clippy::type_complexity)]
    fn snapshot(s: &AssetLibrarySession) -> (library::PackInfo, Vec<(String, String, Option<String>, library::SlotMeta, Vec<(String, ItemTransform, Vec<u8>)>)>) {
        let mut out = Vec::new();
        for uid in s.db.uids_in_order() {
            let items = s.db.items(&uid);
            if items.is_empty() {
                continue;
            }
            let slot = s.db.get(&uid).unwrap();
            let rows = items
                .iter()
                .enumerate()
                .map(|(i, it)| (it.name.clone(), it.transform.clone(), s.image(&uid, i).unwrap().rgba.clone()))
                .collect();
            out.push((uid.clone(), slot.family.key().to_string(), slot.set.clone(), slot.meta.clone(), rows));
        }
        assert!(!out.is_empty(), "snapshot of an empty library proves nothing");
        (s.db.pack.clone(), out)
    }

    /// A pack's images keyed by path, decoded to RGBA, plus its manifest.
    fn decoded_pack(bytes: &[u8]) -> (cartalith_assets::PackManifest, std::collections::BTreeMap<String, (u32, u32, Vec<u8>)>) {
        let (manifest, entries) = cartalith_assets::read_pack(std::io::Cursor::new(bytes)).unwrap();
        let images = entries
            .iter()
            .filter(|(k, _)| k.ends_with(".png"))
            .map(|(k, v)| {
                let d = decode_png(v).unwrap();
                (k.clone(), (d.w, d.h, d.rgba))
            })
            .collect();
        (manifest, images)
    }

    /// Protects: AP-1 bar (a) -- `reference_pack.zip` opens to exactly 17
    /// items in 12 slots, with the pack's name, author and licence, every row
    /// imported and no warning.
    #[test]
    fn the_reference_pack_opens_into_the_library() {
        let mut s = AssetLibrarySession::new();
        let out = s.import_pack(&reference_pack(), "reference_pack.zip", PackImportMode::KeepExisting).unwrap();
        assert_eq!(s.db.total_items(), 17);
        let filled = s.db.uids_in_order().into_iter().filter(|u| !s.db.items(u).is_empty()).count();
        assert_eq!(filled, 12);
        assert_eq!(s.db.pack.name, "Reference Round-Trip Pack");
        assert_eq!(s.db.pack.author, "Cartalith Gen1");
        assert_eq!(s.db.pack.license, "CC0-1.0");
        assert_eq!(out.count(PackRowStatus::Imported), 17);
        assert_eq!(out.rows.len(), 17);
        assert!(out.notes.is_empty(), "{:?}", out.notes);
        assert_eq!(s.db.items("icons:mountain").len(), 3);
        assert_eq!(s.db.items("icons:mountain")[0].name, "mountain", "items take the slot id, the reference's label");
        let naval = s.db.get("custom:naval/lighthouse").expect("custom set kept");
        assert_eq!(naval.set.as_deref(), Some("Naval"));
        assert_eq!(naval.meta.source, "reference_pack.zip (custom/lighthouse)");
        assert_eq!(naval.meta.license, "CC0-1.0");
    }

    /// Protects: AP-1 bar (b) -- re-exporting the opened reference pack
    /// unchanged gives a `pack.json` with the same entries and every PNG
    /// decodes to identical RGBA bytes.
    #[test]
    fn reexporting_the_reference_pack_keeps_entries_and_pixels() {
        let original = reference_pack();
        let mut s = AssetLibrarySession::new();
        s.import_pack(&original, "reference_pack.zip", PackImportMode::KeepExisting).unwrap();
        let (_, exported) = s.export_pack_bytes().unwrap();
        let (m0, img0) = decoded_pack(&original);
        let (m1, img1) = decoded_pack(&exported);
        assert_eq!(m1.textures, m0.textures);
        assert_eq!(m1.biomes, m0.biomes);
        assert_eq!(m1.terrains, m0.terrains);
        assert_eq!(m1.icons, m0.icons);
        assert_eq!(m1.structures, m0.structures);
        assert_eq!(m1.custom, m0.custom);
        assert_eq!((m1.name.as_str(), m1.author.as_str(), m1.license.as_str()), (m0.name.as_str(), m0.author.as_str(), m0.license.as_str()));
        assert_eq!(img0.len(), 17);
        assert_eq!(img1, img0, "every PNG decodes to the same RGBA");
    }

    /// A synthetic pack, made for the test: sprites with an RGB gradient and
    /// a checker of fully transparent holes (so a bake that resampled or
    /// shifted them would move bytes) in icons, sea marks, POI and a custom
    /// set, plus an opaque texture, each at its family's bake size.
    fn synthetic_pack() -> Vec<u8> {
        let mut s = AssetLibrarySession::new();
        s.db.pack.name = "Synthetic".into();
        s.db.pack.author = "test".into();
        s.db.pack.license = "MIT".into();
        let sprite = |seed: u8| {
            let mut v = Vec::with_capacity(256 * 256 * 4);
            for y in 0..256u32 {
                for x in 0..256u32 {
                    // Binary alpha: a checker of transparent holes. See
                    // `the_bake_drifts_semi_transparent_rgb_by_one_level`
                    // for why a fractional alpha is not round-tripped.
                    let a = if (x / 16 + y / 16) % 2 == 0 { 255 } else { 0 };
                    v.extend_from_slice(&[(x as u8).wrapping_add(seed), y as u8, seed, a]);
                }
            }
            encode_png(&DecodedImage::new(256, 256, v).unwrap()).unwrap()
        };
        s.import_item("icons:hill", "a".into(), &sprite(3)).unwrap();
        s.import_item("icons:hill", "b".into(), &sprite(9)).unwrap();
        s.import_item("seamarks:buoy", "c".into(), &sprite(17)).unwrap();
        s.import_item("poi:ruin", "d".into(), &sprite(30)).unwrap();
        let cuid = s.add_custom_slot("Windmill", Some("Farm Things"));
        s.import_item(&cuid, "e".into(), &sprite(44)).unwrap();
        s.import_item("textures:grass", "f".into(), &png_bytes(512, 512, [10, 120, 30, 255])).unwrap();
        s.export_pack_bytes().unwrap().1
    }

    /// Protects: the library round trip -- open a pack, export it, open the
    /// export into a fresh library: the two libraries are equal (slots, sets,
    /// metadata, item names, transforms and byte-equal pixels), for the
    /// reference fixture and for a synthetic pack with alpha gradients and a
    /// sea-mark slot. Both opens use the same `source` label, since it is
    /// recorded as provenance.
    #[test]
    fn export_then_import_gives_an_equal_library() {
        for (label, pack) in [("reference", reference_pack()), ("synthetic", synthetic_pack())] {
            let mut a = AssetLibrarySession::new();
            a.import_pack(&pack, "p.zip", PackImportMode::KeepExisting).unwrap();
            let (_, exported) = a.export_pack_bytes().unwrap();
            let mut b = AssetLibrarySession::new();
            let out = b.import_pack(&exported, "p.zip", PackImportMode::KeepExisting).unwrap();
            assert_eq!(out.count(PackRowStatus::Refused), 0, "{label}: {:?}", out.rows);
            assert_eq!(snapshot(&b), snapshot(&a), "{label}");
        }
        // The synthetic pack really did carry a sea mark and a custom set.
        let mut s = AssetLibrarySession::new();
        s.import_pack(&synthetic_pack(), "p.zip", PackImportMode::KeepExisting).unwrap();
        assert_eq!(s.db.items("seamarks:buoy").len(), 1);
        assert_eq!(s.db.get("custom:farm_things/windmill").and_then(|x| x.set.clone()).as_deref(), Some("Farm Things"));
    }

    /// Measures, and pins, a finding of this pass (2026-10-18) rather than a
    /// wanted behaviour: `render_item` composites with `imageops::overlay`
    /// (source-over) onto a transparent canvas, and for a pixel whose alpha
    /// is strictly between 0 and 255 that moves RGB by one level per bake,
    /// on every bake (measured: worst 1, 1, 1 over three cycles), and it
    /// zeroes the invisible RGB under alpha 0. So a sprite with soft edges is
    /// NOT byte-identical after export, re-import and export, however
    /// unchanged it is. Alpha is untouched. Fixing it changes Apply-to-map pixels, so it is left to
    /// Ruling CR's separate, switched bake fix (`render_item`), not AP-1.
    /// If that fix lands, this test should fail and be replaced.
    #[test]
    fn the_bake_drifts_semi_transparent_rgb_by_one_level() {
        let mut v = Vec::with_capacity(256 * 256 * 4);
        for y in 0..256u32 {
            for x in 0..256u32 {
                v.extend_from_slice(&[x as u8, y as u8, 200, ((x + y) / 2) as u8]);
            }
        }
        let original = DecodedImage::new(256, 256, v).unwrap();
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:hill", "h".into(), &encode_png(&original).unwrap()).unwrap();
        let mut worst_per_cycle = Vec::new();
        let mut prev = original.rgba.clone();
        for _ in 0..3 {
            let (_, bytes) = s.export_pack_bytes().unwrap();
            let mut next = AssetLibrarySession::new();
            next.import_pack(&bytes, "p.zip", PackImportMode::KeepExisting).unwrap();
            let now = next.image("icons:hill", 0).unwrap().rgba.clone();
            // RGB under alpha 0 is invisible and the bake zeroes it; only
            // visible pixels count.
            let worst = prev
                .chunks_exact(4)
                .zip(now.chunks_exact(4))
                .filter(|(a, _)| a[3] > 0)
                .flat_map(|(a, b)| (0..3).map(move |c| a[c].abs_diff(b[c])))
                .max()
                .unwrap();
            let alpha_same = prev.chunks_exact(4).zip(now.chunks_exact(4)).all(|(a, b)| a[3] == b[3]);
            assert!(alpha_same, "alpha must survive the bake byte for byte");
            worst_per_cycle.push(worst);
            prev = now;
            s = next;
        }
        assert_eq!(worst_per_cycle, vec![1, 1, 1], "one level per bake, every bake");
    }

    /// A pack zip of `manifest` plus `files`, written with this crate's own
    /// pack writer so names and methods are what an exporter produces.
    fn pack_zip(manifest: &str, files: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut entries: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (*n, d.as_slice())).collect();
        entries.push(("pack.json", manifest.as_bytes()));
        archive::zip_store_bytes(&entries).unwrap()
    }

    /// Protects: AP-1 bar (c) -- an unknown slot id goes to Custom/Imported
    /// with one warning per unknown slot, in the reference's own words, and
    /// the import still succeeds for the known slot beside it.
    #[test]
    fn an_unknown_slot_lands_in_custom_imported_with_a_warning() {
        let png = png_bytes(4, 4, [1, 2, 3, 255]);
        let z = pack_zip(
            r#"{"icons":{"mountain":["m.png"],"volcano":["v.png"]},"structures":{"poi":{"obelisk":["o.png"]}}}"#,
            &[("m.png", png.clone()), ("v.png", png.clone()), ("o.png", png)],
        );
        let mut s = AssetLibrarySession::new();
        let out = s.import_pack(&z, "x.zip", PackImportMode::KeepExisting).unwrap();
        assert_eq!(
            out.notes,
            vec!["unknown slot \"volcano\" → Custom/Imported".to_string(), "unknown slot \"obelisk\" → Custom/Imported".to_string()]
        );
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        assert_eq!(s.db.items("custom:imported/volcano").len(), 1);
        assert_eq!(s.db.items("custom:imported/obelisk").len(), 1);
    }

    /// Protects: "never overwrite existing slots without confirm" -- a dry
    /// run names the filled slot and the differing licence and changes
    /// nothing; KeepExisting skips the filled slot (with the reason) and the
    /// licence; Replace empties the slot first, so importing the same pack
    /// twice leaves the same three variants, not six.
    #[test]
    fn a_filled_slot_is_a_conflict_until_the_user_replaces_it() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "mine.png".into(), &png_bytes(4, 4, [9, 9, 9, 255])).unwrap();
        s.db.pack.license = "All rights reserved".into();
        let pack = reference_pack();

        let dry = s.import_pack(&pack, "r.zip", PackImportMode::DryRun).unwrap();
        assert_eq!(dry.conflict_uids, vec!["icons:mountain".to_string()]);
        assert_eq!(dry.conflicts.len(), 2, "{:?}", dry.conflicts);
        assert!(dry.conflicts[1].contains("All rights reserved"), "{:?}", dry.conflicts);
        assert!(dry.rows.is_empty());
        assert_eq!(s.db.total_items(), 1, "a dry run changes nothing");
        assert!(s.db.get("custom:naval/lighthouse").is_none(), "a dry run creates no custom slot");

        let keep = s.import_pack(&pack, "r.zip", PackImportMode::KeepExisting).unwrap();
        assert_eq!(keep.count(PackRowStatus::Skipped), 3);
        assert!(keep.rows.iter().filter(|r| r.status == PackRowStatus::Skipped).all(|r| r.reason.contains("kept")));
        assert_eq!(s.db.items("icons:mountain")[0].name, "mine.png");
        assert_eq!(s.db.pack.license, "All rights reserved");
        assert_eq!(s.db.total_items(), 1 + 14);

        s.import_pack(&pack, "r.zip", PackImportMode::Replace).unwrap();
        assert_eq!(s.db.items("icons:mountain").len(), 3);
        assert_eq!(s.db.items("icons:mountain")[0].name, "mountain");
        assert_eq!(s.db.pack.license, "CC0-1.0");
        s.import_pack(&pack, "r.zip", PackImportMode::Replace).unwrap();
        assert_eq!(s.db.total_items(), 17, "a second Replace is idempotent");
    }

    /// Protects: hostile and invalid archives. A path-traversal entry and a
    /// deflate bomb refuse the whole pack with a reason and change nothing;
    /// a non-PNG, an oversize IHDR and a corrupt PNG each refuse just their
    /// own row, with a reason, while a good image beside them still imports.
    #[test]
    fn hostile_and_invalid_packs_are_refused_with_reasons() {
        let png = png_bytes(4, 4, [1, 2, 3, 255]);
        let mut s = AssetLibrarySession::new();

        let traversal = pack_zip(r#"{"icons":{"hill":["h.png"]}}"#, &[("h.png", png.clone()), ("../evil.png", png.clone())]);
        let err = s.import_pack(&traversal, "t.zip", PackImportMode::KeepExisting).unwrap_err();
        assert!(err.contains("../evil.png") && err.contains(".."), "{err}");

        // 80 MiB of spaces deflates to well under 1 MiB: the entry cap
        // (64 MiB decompressed), not the archive size, must catch it.
        let spaces = vec![b' '; 80 * 1024 * 1024];
        let bomb = archive::zip_store_bytes(&[("pack.json", spaces.as_slice())]).unwrap();
        assert!(bomb.len() < 1024 * 1024, "the fixture is a bomb: {} bytes", bomb.len());
        let err = s.import_pack(&bomb, "b.zip", PackImportMode::KeepExisting).unwrap_err();
        assert!(err.contains("pack.json") && err.contains("64 MiB"), "{err}");
        assert_eq!(s.db.total_items(), 0);

        // An IHDR claiming 9000 x 9000: 24 real header bytes, no pixels.
        let mut huge = png[..24].to_vec();
        huge[16..20].copy_from_slice(&9000u32.to_be_bytes());
        huge[20..24].copy_from_slice(&9000u32.to_be_bytes());
        let mut corrupt = png[..33].to_vec();
        corrupt.extend_from_slice(b"garbage, not an IDAT");
        let z = pack_zip(
            r#"{"icons":{"hill":["gif.png","huge.png","bad.png","ok.png"]}}"#,
            &[("gif.png", b"GIF89a-and-then-some-bytes".to_vec()), ("huge.png", huge), ("bad.png", corrupt), ("ok.png", png)],
        );
        let out = s.import_pack(&z, "m.zip", PackImportMode::KeepExisting).unwrap();
        let reasons: Vec<&str> = out.rows.iter().map(|r| r.reason.as_str()).collect();
        assert!(reasons[0].contains("not a PNG"), "{reasons:?}");
        assert!(reasons[1].contains("8192"), "{reasons:?}");
        assert!(reasons[2].contains("not a readable PNG"), "{reasons:?}");
        assert_eq!(out.rows[3].status, PackRowStatus::Imported);
        assert_eq!(out.count(PackRowStatus::Refused), 3);
        assert_eq!(s.db.items("icons:hill").len(), 1);
    }

    /// Protects: a custom slot whose every image is refused is never created
    /// (it would otherwise trip the validator's "Empty custom slot").
    #[test]
    fn a_custom_slot_with_nothing_importable_is_not_created() {
        let z = pack_zip(r#"{"custom":{"Naval":{"wreck":["missing.png"]}}}"#, &[]);
        let mut s = AssetLibrarySession::new();
        let out = s.import_pack(&z, "w.zip", PackImportMode::KeepExisting).unwrap();
        assert_eq!(out.rows[0].reason, "missing file: missing.png");
        assert!(s.db.get("custom:naval/wreck").is_none());
    }

    // -- AP-2: the pack report ------------------------------------------------

    /// A 4x4 PNG tagged Display P3 through a `cICP` chunk (primaries 12,
    /// transfer 13), so the colour-managed decode converts it. The chunk is
    /// spliced in after IHDR with its CRC computed here.
    fn p3_png() -> Vec<u8> {
        let plain = png_bytes(4, 4, [200, 30, 30, 255]);
        let data = [12u8, 13, 0, 1];
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&4u32.to_be_bytes());
        chunk.extend_from_slice(b"cICP");
        chunk.extend_from_slice(&data);
        let mut crc_in = b"cICP".to_vec();
        crc_in.extend_from_slice(&data);
        chunk.extend_from_slice(&crc32(&crc_in).to_be_bytes());
        let ihdr_end = 8 + 4 + 4 + 13 + 4;
        let mut out = plain[..ihdr_end].to_vec();
        out.extend_from_slice(&chunk);
        out.extend_from_slice(&plain[ihdr_end..]);
        out
    }

    /// CRC-32 (ISO 3309 / PNG §5.5), bitwise; only for building a fixture.
    fn crc32(bytes: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for b in bytes {
            c ^= u32::from(*b);
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }

    /// Protects: a converted import becomes one Colour finding at its slot
    /// and index, carrying the kept report; an untagged import beside it
    /// raises none; `clear` drops the notes with the pixels.
    #[test]
    fn a_colour_conversion_reaches_the_report_and_a_plain_import_does_not() {
        let mut s = AssetLibrarySession::new();
        s.db.pack.license = "MIT".into();
        let report = s.import_item_managed("icons:hill", "p3.png".into(), &p3_png()).unwrap();
        assert!(report.converted, "the fixture must really convert: {report:?}");
        s.import_item("icons:boulder", "plain.png".into(), &png_bytes(4, 4, [1, 2, 3, 0])).unwrap();
        let colour: Vec<(Finding, Option<ColourReport>)> =
            s.pack_report().into_iter().filter(|(f, _)| f.kind == FindingKind::Colour).collect();
        assert_eq!(colour.len(), 1);
        assert_eq!(colour[0].0.uid.as_deref(), Some("icons:hill"));
        assert_eq!(colour[0].0.index, Some(0));
        assert!(colour[0].1.as_ref().unwrap().converted);
        s.clear();
        assert!(s.colour.is_empty());
    }

    /// Protects: the report on the opened reference pack. Its validator line
    /// (identical flat-colour images) comes first; the licence is set so no
    /// licence finding; nothing is named as never drawn (until AP-3 its
    /// settlement, POI and custom slots were; AP-3 draws them); Sea marks is
    /// the one empty family; the pack's flat textures
    /// raise no seam. Every row that names a slot names one that exists.
    #[test]
    fn the_reference_pack_report() {
        let mut s = AssetLibrarySession::new();
        s.import_pack(&reference_pack(), "r.zip", PackImportMode::KeepExisting).unwrap();
        let r: Vec<Finding> = s.pack_report().into_iter().map(|(f, _)| f).collect();
        assert_eq!(r[0].kind, FindingKind::Validator);
        assert!(r[0].text.starts_with("Identical images"), "{}", r[0].text);
        assert!(!r.iter().any(|f| f.kind == FindingKind::BlankLicence));
        assert!(!r.iter().any(|f| f.kind == FindingKind::Seam));
        let not_drawn: Vec<&str> = r.iter().filter(|f| f.kind == FindingKind::NotDrawn).map(|f| f.uid.as_deref().unwrap()).collect();
        // AP-3 (2026-10-18). Old -> new: ["settlement:village",
        // "settlement:city", "poi:ruin", "custom:naval/lighthouse"] -> [].
        assert_eq!(not_drawn, Vec::<&str>::new());
        let empty: Vec<&str> = r.iter().filter(|f| f.kind == FindingKind::EmptyFamily).map(|f| f.label.as_str()).collect();
        assert_eq!(empty, vec!["Sea marks"]);
        for f in &r {
            if let Some(u) = &f.uid {
                assert!(s.db.get(u).is_some(), "{u}");
            }
        }
    }

    /// A 200x160 PNG sprite: an orange disc shaded left to right, with a soft
    /// edge, off centre on transparency -- art with a margin to trim, a hue
    /// to harmonise and an edge to outline.
    fn disc_png() -> Vec<u8> {
        let (w, h) = (200u32, 160u32);
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f64 - 90.0).powi(2) + (y as f64 - 70.0).powi(2)).sqrt();
                let a = (50.0 - d).clamp(0.0, 1.0);
                data.extend_from_slice(&[(150 + x / 3) as u8, (60 + x / 6) as u8, 30, (a * 255.0).round() as u8]);
            }
        }
        encode_png(&DecodedImage::new(w, h, data).unwrap()).unwrap()
    }

    /// Protects: AP-7's bar through the session, verb by verb. Each verb that
    /// has something to do adds exactly one new variant, named after the
    /// verb; the original's pixels and record are byte-identical afterwards;
    /// and undo (`remove_item` on the returned index) takes the slot back to
    /// one item with the original intact.
    #[test]
    fn every_refine_verb_adds_one_variant_and_keeps_the_original() {
        let palette = [[60, 140, 50], [40, 80, 200], [222, 172, 78]];
        for verb in [RefineVerb::Trim, RefineVerb::Pad, RefineVerb::Flip, RefineVerb::Tint, RefineVerb::Outline, RefineVerb::Shadow, RefineVerb::Harmonise] {
            let mut s = AssetLibrarySession::new();
            s.import_item("icons:mountain", "m.png".to_string(), &disc_png()).unwrap();
            let (px, rec) = (s.image("icons:mountain", 0).unwrap().clone(), s.db.items("icons:mountain")[0].clone());
            let out = s.refine_variant("icons:mountain", 0, verb, &palette).unwrap();
            assert_eq!(out.index, Some(1), "{verb:?}: {}", out.note);
            assert_eq!(s.db.items("icons:mountain").len(), 2);
            assert_eq!(s.db.items("icons:mountain")[1].name, format!("m.png · {}", verb.suffix()));
            assert_ne!(s.image("icons:mountain", 1).unwrap(), &px, "{verb:?} changed something");
            assert_eq!(s.image("icons:mountain", 0).unwrap(), &px, "{verb:?}: original pixels untouched");
            assert_eq!(s.db.items("icons:mountain")[0], rec, "{verb:?}: original record untouched");
            assert!(!out.note.is_empty());
            assert!(s.remove_item("icons:mountain", 1));
            assert_eq!(s.db.items("icons:mountain").len(), 1);
            assert_eq!(s.image("icons:mountain", 0).unwrap(), &px);
        }
    }

    /// Protects: the per-verb pixel source and transform rules
    /// (`refine_variant`'s doc). Trim and pad keep the source's pixels
    /// (no resampling) and take the intake fit; flip negates `pan_x`; outline
    /// and shadow are drawn on the 256² bake with the identity transform, so
    /// their own bake is their pixels; tint and harmonise keep alpha
    /// byte-equal and the transform; harmonise meets AP-7's 10° bar.
    #[test]
    fn refine_verbs_take_the_documented_pixels_and_transforms() {
        let palette = [[60, 140, 50], [40, 80, 200], [222, 172, 78]];
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "m.png".to_string(), &disc_png()).unwrap();
        let src = s.image("icons:mountain", 0).unwrap().clone();
        let alpha = |im: &DecodedImage| im.rgba.chunks_exact(4).map(|p| p[3]).collect::<Vec<u8>>();
        s.db.item_mut("icons:mountain", 0).unwrap().transform.pan_x = 7.0;
        let t0 = s.db.items("icons:mountain")[0].transform.clone();

        let i = s.refine_variant("icons:mountain", 0, RefineVerb::Trim, &palette).unwrap().index.unwrap();
        let t = s.image("icons:mountain", i).unwrap().clone();
        assert!(t.w <= 102 && t.h <= 102 && t.w >= 98, "{}x{}", t.w, t.h);
        assert_eq!(s.db.items("icons:mountain")[i].transform.pan_x, 0.0, "intake transform, not the old pan");
        assert!(s.db.items("icons:mountain")[i].transform.pan_y > 0.0 || t.h == t.w, "bottom family: fitted on intake");

        let i = s.refine_variant("icons:mountain", 0, RefineVerb::Flip, &palette).unwrap().index.unwrap();
        assert_eq!(s.db.items("icons:mountain")[i].transform.pan_x, -7.0);
        assert_eq!(s.db.items("icons:mountain")[i].transform.pan_y, t0.pan_y);

        let i = s.refine_variant("icons:mountain", 0, RefineVerb::Outline, &palette).unwrap().index.unwrap();
        let o = s.image("icons:mountain", i).unwrap().clone();
        assert_eq!((o.w, o.h), (256, 256), "drawn at the bake size");
        assert_eq!(s.db.items("icons:mountain")[i].transform, ItemTransform::default());
        // Its own bake is its pixels, up to the bake's known drift on soft
        // edges (AP-1's finding, `the_bake_drifts_semi_transparent_rgb_by_one_level`):
        // alpha exact, RGB within one level wherever alpha is not 0.
        let rebake = s.sprite_bake("icons:mountain", i).unwrap();
        assert_eq!(alpha(&rebake), alpha(&o));
        for (a, b) in rebake.rgba.chunks_exact(4).zip(o.rgba.chunks_exact(4)).filter(|(_, b)| b[3] > 0) {
            assert!((0..3).all(|c| a[c].abs_diff(b[c]) <= 1), "{a:?} vs {b:?}");
        }

        let i = s.refine_variant("icons:mountain", 0, RefineVerb::Harmonise, &palette).unwrap();
        let h = i.harmonised.as_ref().unwrap();
        assert_eq!(h.target_rgb, [222, 172, 78], "the gold is the nearest hue to the orange disc");
        let d = (h.hue_after - h.target_hue + 540.0).rem_euclid(360.0) - 180.0;
        assert!(d.abs() <= 10.0, "AP-7 bar: {h:?}");
        let hi = s.image("icons:mountain", i.index.unwrap()).unwrap();
        assert_eq!(alpha(hi), alpha(&src), "alpha byte-equal");
        assert_eq!(s.db.items("icons:mountain")[i.index.unwrap()].transform, t0);

        let i = s.refine_variant("icons:mountain", 0, RefineVerb::Tint, &palette).unwrap().index.unwrap();
        assert_eq!(alpha(s.image("icons:mountain", i).unwrap()), alpha(&src));
        let j = s.refine_variant("icons:mountain", 0, RefineVerb::Tint, &palette).unwrap().index.unwrap();
        assert_ne!(s.image("icons:mountain", i).unwrap(), s.image("icons:mountain", j).unwrap(), "a second tint is a different variant");
    }

    /// Protects: the family rules and the nothing-to-do answers. Trim, pad,
    /// outline and shadow refuse a ground tile; tint and harmonise refuse a
    /// splat texture but run on a biome tile; trim on art that already
    /// reaches every edge, and harmonise on grey art, write nothing; an
    /// unknown uid or index is an error in words, never a panic.
    #[test]
    fn refine_refusals_are_in_words_and_write_nothing() {
        let palette = [[222, 172, 78]];
        let mut s = AssetLibrarySession::new();
        s.import_item("textures:grass", "g.png".to_string(), &png_bytes(64, 64, [40, 120, 40, 255])).unwrap();
        s.import_item("biomes:jungle", "j.png".to_string(), &png_bytes(64, 64, [40, 120, 40, 255])).unwrap();
        s.import_item("icons:hill", "grey.png".to_string(), &png_bytes(32, 32, [90, 90, 90, 255])).unwrap();
        for v in [RefineVerb::Trim, RefineVerb::Pad, RefineVerb::Outline, RefineVerb::Shadow] {
            assert!(s.refine_variant("biomes:jungle", 0, v, &palette).unwrap_err().contains("tiled edge to edge"), "{v:?}");
        }
        for v in [RefineVerb::Tint, RefineVerb::Harmonise] {
            assert!(s.refine_variant("textures:grass", 0, v, &palette).unwrap_err().contains("splat"), "{v:?}");
        }
        assert_eq!(s.refine_variant("biomes:jungle", 0, RefineVerb::Harmonise, &palette).unwrap().index, Some(1));
        assert_eq!(s.refine_variant("textures:grass", 0, RefineVerb::Flip, &palette).unwrap().index, Some(1));
        let r = s.refine_variant("icons:hill", 0, RefineVerb::Trim, &palette).unwrap();
        assert_eq!(r.index, None, "opaque to every edge: nothing to trim");
        assert!(r.note.contains("nothing to trim"));
        assert!(s.refine_variant("icons:hill", 0, RefineVerb::Harmonise, &palette).unwrap_err().contains("grey"));
        assert_eq!(s.db.items("icons:hill").len(), 1);
        assert!(s.refine_variant("icons:hill", 3, RefineVerb::Flip, &palette).is_err());
        assert!(s.refine_variant("icons:nope", 0, RefineVerb::Flip, &palette).is_err());
    }

    /// Protects: *Fit all* is transform-only and undoable. It moves every
    /// variant of every listed slot to its Fit transform, reports each
    /// item's OLD transform (and only the items that moved), writes no pixel
    /// and adds no variant; a second call moves nothing; restoring the
    /// reported transforms puts the library back exactly.
    #[test]
    fn fit_all_moves_transforms_only_and_reports_the_old_ones() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &disc_png()).unwrap();
        s.import_item("icons:mountain", "b.png".to_string(), &disc_png()).unwrap();
        s.import_item("settlement:city", "c.png".to_string(), &disc_png()).unwrap();
        s.db.item_mut("icons:mountain", 1).unwrap().transform = ItemTransform { scale: 1.4, pan_x: 5.0, pan_y: -3.0 };
        s.db.item_mut("settlement:city", 0).unwrap().transform = ItemTransform { scale: 0.8, pan_x: 0.0, pan_y: 2.0 };
        let before: Vec<LibraryItem> = ["icons:mountain", "settlement:city"].iter().flat_map(|u| s.db.items(u).to_vec()).collect();
        let px: Vec<DecodedImage> = vec![s.image("icons:mountain", 0).unwrap().clone(), s.image("icons:mountain", 1).unwrap().clone()];
        let moved = s.fit_all(&["icons:mountain".to_string(), "settlement:city".to_string(), "icons:nope".to_string()]);
        assert_eq!(moved.len(), 2, "variant 0 was already fitted on intake: {moved:?}");
        assert_eq!((moved[0].0.as_str(), moved[0].1), ("icons:mountain", 1));
        assert_eq!(moved[0].2, ItemTransform { scale: 1.4, pan_x: 5.0, pan_y: -3.0 });
        assert_eq!(s.db.items("settlement:city")[0].transform, ItemTransform::default(), "a centre-anchored family fits to the default");
        assert_eq!(s.db.items("icons:mountain")[1].transform, s.db.items("icons:mountain")[0].transform);
        assert_eq!(s.db.items("icons:mountain").len(), 2, "no variant added");
        assert_eq!(s.image("icons:mountain", 1).unwrap(), &px[1], "no pixel written");
        assert!(s.fit_all(&["icons:mountain".to_string(), "settlement:city".to_string()]).is_empty());
        for (uid, i, t) in moved {
            s.db.item_mut(&uid, i).unwrap().transform = t;
        }
        let after: Vec<LibraryItem> = ["icons:mountain", "settlement:city"].iter().flat_map(|u| s.db.items(u).to_vec()).collect();
        assert_eq!(after, before, "undo restores every record");
    }

    /// Protects: AP-6 on the reference pack, a format fixture of flat
    /// swatches. No near-duplicate row: the identical mountains, conifers and
    /// lighthouses are exact groups (reported as such), and the swatches of
    /// similar colour (rock and paved road; village, city and ruin, all
    /// within 32 levels) are flat images, which the finder leaves out
    /// (`NearHash::is_flat`). Measured while building: without that rule it
    /// gave two groups, and without the sprite/ground split as well, one
    /// six-member group.
    #[test]
    fn the_reference_pack_has_no_near_duplicate_row() {
        let mut s = AssetLibrarySession::new();
        s.import_pack(&reference_pack(), "r.zip", PackImportMode::KeepExisting).unwrap();
        let r: Vec<Finding> = s.pack_report().into_iter().map(|(f, _)| f).collect();
        let near: Vec<&Finding> = r.iter().filter(|f| f.kind == FindingKind::NearDuplicate).collect();
        assert!(near.is_empty(), "{near:?}");
    }

    /// Protects: `clear` empties the library: items, custom slots and decoded pixels.
    #[test]
    fn clear_drops_every_item_and_every_decoded_pixel() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(4, 4, [1, 2, 3, 255])).unwrap();
        let cuid = s.add_custom_slot("Lighthouse", Some("Naval"));
        s.import_item(&cuid, "l.png".to_string(), &png_bytes(4, 4, [1, 2, 3, 255])).unwrap();
        s.clear();
        assert_eq!(s.db.total_items(), 0);
        assert!(s.db.get(&cuid).is_none());
        assert!(s.image("icons:mountain", 0).is_none());
    }
}
