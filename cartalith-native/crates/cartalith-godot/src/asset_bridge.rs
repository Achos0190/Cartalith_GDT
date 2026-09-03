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

use std::collections::HashMap;

use cartalith_assets::{
    AssetDB, Anchor, ChromaKey, DecodedImage, Family, GridRect, ItemTransform, LibraryItem,
    PackEntries, PackManifest, SliceCounts, SliceGrid, SliceOptions, archive, compute_cells, count_cells,
    decode_png, encode_png, fit_to_bottom, item_hash, library, render_item, sheet_base_name,
    slice_sheet,
};

/// The sprite-sheet slicer's loaded sheet — the reference's
/// `SpriteSheetImporter.sheet` (`{img,cv,ctx,w,h,name}`, line 27833), minus
/// the three canvas handles. Held on the session rather than passed per call
/// so the modal's live `N cells detected · M non-empty` readout can re-run the
/// real detection pass on every spinbox change without re-sending a 3072×2048
/// PNG across the boundary each time.
pub struct LoadedSheet {
    pub name: String,
    pub image: DecodedImage,
}

/// The slicer modal's four numbers plus its three toggles, in engine terms.
#[derive(Debug, Clone, PartialEq)]
pub struct SliceParams {
    pub cols: i64,
    pub rows: i64,
    /// §8's *Margin px* — a uniform inset of the reference's `gridRect`.
    pub margin: f64,
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
    Slot { uid: String },
    NewCustom { name: String, set: String },
    PerCell { set: String },
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
    pub added: usize,
    pub skipped_blank: usize,
    /// Cells that had nowhere to go: a single-image family taking only the
    /// first, or more cells than the target family has slots.
    pub unplaced: usize,
    /// Every slot uid the slice wrote to, in write order.
    pub uids: Vec<String>,
}

/// The reference's `(E('alSlSet').value||'Default').trim()||'Default'` (line
/// 27790) in `add_custom_slot`'s own terms: a blank set name means "let
/// `AssetDB` apply its own `Default`", not "a set literally named nothing".
fn set_or_none(set: &str) -> Option<&str> {
    let t = set.trim();
    (!t.is_empty()).then_some(t)
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
}

impl Default for AssetLibrarySession {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLibrarySession {
    pub fn new() -> Self {
        AssetLibrarySession { db: AssetDB::new(), images: HashMap::new(), sheet: None }
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
    pub fn import_item(&mut self, uid: &str, name: String, bytes: &[u8]) -> Result<(), String> {
        if self.db.get(uid).is_none() {
            return Err(format!("no such slot: {uid}"));
        }
        let decoded = decode_png(bytes).map_err(|e| e.to_string())?;
        self.insert_decoded(uid, name, decoded)
    }

    /// The reference's `mkItem` (line 27793) plus the store write: default
    /// transform, `fitToBottom` on a bottom-anchored family, a real
    /// [`item_hash`], and both stores advanced together. Shared by
    /// [`AssetLibrarySession::import_item`] and the slicer, which build the
    /// same item out of differently-sourced pixels.
    fn insert_decoded(&mut self, uid: &str, name: String, decoded: DecodedImage) -> Result<(), String> {
        let Some(slot) = self.db.get(uid) else {
            return Err(format!("no such slot: {uid}"));
        };
        let family = slot.family;
        let mut transform = ItemTransform::default();
        if family.anchor() == Anchor::Bottom {
            fit_to_bottom(&mut transform, decoded.w, decoded.h, family.size());
        }
        let hash = item_hash(&decoded);
        let item = LibraryItem::new(name, hash).with_transform(transform);
        if !self.db.add_item(uid, item) {
            return Err(format!("no such slot: {uid}"));
        }
        self.images.entry(uid.to_string()).or_default().push(decoded);
        Ok(())
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
    pub fn load_sheet(&mut self, name: String, bytes: &[u8]) -> Result<(u32, u32), String> {
        let image = decode_png(bytes).map_err(|e| e.to_string())?;
        let dims = (image.w, image.h);
        self.sheet = Some(LoadedSheet { name, image });
        Ok(dims)
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
                    self.insert_decoded(uid, name, cell.image.clone())?;
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
                    self.insert_decoded(&uid, item_name, cell.image.clone())?;
                    out.added += 1;
                }
                out.uids.push(uid);
            }
            SliceTarget::PerCell { set } => {
                for cell in &cells {
                    let name = cell.default_cell_name();
                    let uid = self.add_custom_slot(&name, set_or_none(set));
                    self.insert_decoded(&uid, name, cell.image.clone())?;
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
                    self.insert_decoded(uid, name, cell.image.clone())?;
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
    }

    // -- Validation --------------------------------------------------------

    /// `AssetValidator.run()` — real warning strings, ordered exactly as the
    /// reference produces them.
    pub fn validate(&self) -> Vec<String> {
        library::run(&self.db)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        let img = DecodedImage::new(w, h, data).unwrap();
        encode_png(&img).unwrap()
    }

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

    #[test]
    fn import_item_into_settlement_family_does_not_fit_to_bottom() {
        let mut s = AssetLibrarySession::new();
        s.import_item("settlement:hamlet", "h.png".to_string(), &png_bytes(16, 8, [1, 2, 3, 255])).unwrap();
        assert_eq!(s.db.items("settlement:hamlet")[0].transform.pan_y, 0.0);
    }

    #[test]
    fn import_item_rejects_unknown_uid_and_bad_bytes_without_panicking() {
        let mut s = AssetLibrarySession::new();
        assert!(s.import_item("nope:nope", "x.png".to_string(), &png_bytes(4, 4, [0, 0, 0, 255])).is_err());
        assert!(s.import_item("icons:mountain", "x.png".to_string(), b"not a png").is_err());
    }

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

    #[test]
    fn export_pack_bytes_fails_honestly_on_an_empty_library() {
        let mut s = AssetLibrarySession::new();
        assert!(s.export_pack_bytes().is_err());
    }

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

    fn params(cols: i64, rows: i64) -> SliceParams {
        SliceParams { cols, rows, margin: 0.0, spacing: 0.0, chroma: None, trim: false, skip_blank: true, col_lines: None, row_lines: None, only_cell: None }
    }

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

    #[test]
    fn slice_preview_without_a_sheet_is_an_error_not_a_panic() {
        let s = AssetLibrarySession::new();
        assert!(s.slice_preview(&params(2, 1)).is_err());
    }

    #[test]
    fn load_sheet_rejects_non_png_bytes_without_panicking() {
        let mut s = AssetLibrarySession::new();
        assert!(s.load_sheet("x.gif".to_string(), b"not a png").is_err());
    }

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

    #[test]
    fn an_impossible_margin_is_an_error_not_a_negative_grid() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        let p = SliceParams { margin: 40.0, ..params(2, 1) };
        assert!(s.slice_preview(&p).is_err());
        assert!(s.apply_slice(&p, &SliceTarget::PerCell { set: String::new() }).is_err());
    }

    #[test]
    fn slicing_into_an_unknown_slot_is_an_error_not_a_panic() {
        let mut s = AssetLibrarySession::new();
        s.load_sheet("sheet.png".to_string(), &half_blank_sheet()).unwrap();
        assert!(s.apply_slice(&params(2, 1), &SliceTarget::Slot { uid: "nope:nope".to_string() }).is_err());
    }

    // -- AS-17: cell-scoped slicing / draggable interior lines -------------

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
