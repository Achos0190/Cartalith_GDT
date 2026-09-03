//! The Paint editor's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md`
//! milestone F, the WORLD-domain Biome/Terrain/Splat paint tool
//! (`DCC_SHELL_SPEC.md` §4.5.2's `PAINT · BIOME` tool options row;
//! `UNIFIED_TOOL_PLAN.md`'s "Biome paint (`P`)" investigation in the Water &
//! ecology group).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs` already established for this crate: `lib.rs` owns the
//! thin `Variant`<->`f64`/`String`/`Gd<ImageTexture>` conversion and the
//! `#[func]` surface; this module owns the real state — which layer is
//! active, the live brush parameters, and the three draft/committed pairs —
//! with its own `#[cfg(test)]` suite below, exercised by
//! `cargo test -p cartalith-godot` with no Godot runtime involved.
//!
//! ## Three layers, three drafts, one shared tracker — the one real
//! structural difference from `sculpt_bridge.rs`
//!
//! Sculpt has exactly one draft (`PassBuffer<SculptStamp>`) because every
//! feature it can paint writes the same destination, the height field.
//! Cartography paint does not have that property: `cartalith-spatial/src/
//! paint.rs`'s own module doc found the reference ships **three** peer
//! override arrays (`paintBiome`/`paintTerrain`/`paintSplat`) behind one
//! brush, switched by `_paintLayer`, "differing only in which palette the
//! value indexes." [`PassBuffer::commit`] bakes its *whole* stack into
//! *one* destination slice, so one shared draft cannot serve three
//! destinations at once — a caller who painted some Biome dabs, switched to
//! Terrain mid-session and painted more, then hit Commit, needs *both*
//! sets of dabs baked into *their own* arrays. [`PaintEditor`] therefore
//! keeps one [`cartalith_spatial::PassBuffer`]`<PaintStamp>` and one
//! [`cartalith_spatial::PaintLayer`] per layer, and [`PaintEditor::
//! commit_all`]/[`PaintEditor::discard_all`] act on all three every time —
//! matching the shell's single Commit/Discard pair in `DCC_SHELL_SPEC.md`
//! §4.5.2's tool options row, and meaning a layer switch never silently
//! drops the layer being left. All three share one [`DirtyTracker`]
//! (tile-version bookkeeping has no per-layer meaning of its own — see
//! [`PaintEditor::commit_all`]'s own doc).
//!
//! ## `hardness`/`softness` feather the disc's own edge (`DECISIONS.md` §7k)
//!
//! Until 2026-09-01 this section recorded that these two fields went
//! nowhere: `cartalith-spatial/src/paint.rs`'s own module doc was
//! unambiguous that painting is "a hard disc... unlike `sculpt()`/
//! `brushHeight` there's no soft falloff here" (the reference's own
//! comment, quoted there verbatim), and neither [`PaintStamp::apply`] nor
//! [`PaintEditor::stroke_at`] read either field — `DCC_SHELL_SPEC.md`
//! §4.5.2's tool options row lists them on the `PAINT · BIOME` row almost
//! certainly carried over from the Sculpt row's own shape, with nothing in
//! the reference or in `cartalith-spatial::paint` giving either a meaning
//! for a categorical brush.
//!
//! The owner ruled 2026-08-31 (`LARGE_ITEM_RULINGS.md`; `UNWIRED_
//! FUNCTIONS.md`'s highest-severity row): **bind it** — as a deliberate,
//! disclosed divergence from the reference, which has no falloff for this
//! brush at all, recorded in `DECISIONS.md` §7k. [`PaintEditor::
//! stroke_at`] now calls [`PaintStamp::with_falloff`] with both fields,
//! verbatim, on every dab. The categorical-blending objection
//! `cartalith-spatial/src/paint.rs` raises is real and untouched by this:
//! no palette index is ever blended with another, at any hardness or
//! softness. What softens is the disc's own *edge* — which cells a dab
//! touches at all — decided by a deterministic per-cell threshold, never
//! the *value* a touched cell receives. The mechanism lives entirely in
//! `cartalith-spatial`, not here — see [`PaintStamp`]'s own doc.
//!
//! [`Brush::default`]'s `hardness = 1.0, softness = 0.0` is the exact pair
//! [`PaintStamp::with_falloff`] treats as a literal zero-width band, so an
//! untouched brush paints the historical hard disc, bit-for-bit — this is a
//! strict superset of the old behaviour, not a replacement for it, and
//! every pre-existing golden/regression test for the hard-disc case (this
//! module's own, `cartalith-spatial`'s and `cartalith-civ`'s) keeps
//! passing unchanged.
//!
//! **The duplicate `Hardness` slider is resolved, not just this field.**
//! `UNWIRED_FUNCTIONS.md` also flagged two live copies of the same control
//! on screen at once — `world_workspace.gd`'s WORLD dock panel and
//! `tool_bar.gd`'s unified tool options bar both drew one. The dock panel
//! owns the actual brush state (`world_workspace.gd`'s own `_paint_brush`
//! dictionary; the tool bar only mirrors it through `_paint_state`/
//! `_write_paint_state`) and is the one place `Hardness` and `Softness`
//! already lived side by side, so the tool bar's copy was removed there —
//! nothing in this crate prescribes which surface keeps a control; that
//! choice is `world_workspace.gd`'s/`tool_bar.gd`'s own to make and is
//! recorded in each file's own history, not repeated here.
//!
//! ## The land-only gate is a toggle here, not the reference's hard-always
//!
//! `UNIFIED_TOOL_PLAN.md`'s own investigation calls the reference's gate
//! "hard, not a toggle." `PaintStamp::mask`'s own doc comment is explicit
//! that making it optional is **this port's addition**, built for
//! `UI_SHELL_DESIGN.md`'s "respect water mask" switch, which the reference
//! has no equivalent for — flagged there as a new affordance, not a parity
//! claim. [`Brush::land_only`] defaults to `true` (every dab gated, exactly
//! the reference's own behaviour) so an untouched editor behaves exactly
//! like the reference; a caller must deliberately turn it off to reach the
//! new affordance.
//!
//! ## Why the water mask is captured once, not recomputed per dab
//!
//! `_paintAt` is called once per pointer-move sample during a drag —
//! plausibly dozens of times a second. Recomputing `cartalith_civ::
//! build_water_bodies` (a flood fill from every ocean edge) on every one of
//! those calls would make the brush's cost scale with how long the user
//! drags rather than with how much they paint. The reference does not have
//! this problem (its own `_wb` is a module-global recomputed only on
//! terrain rebuild); this port's closest equivalent is `WorldGen::absorb`,
//! which already recomputes exactly this classification once per
//! `generate()`/`generate_sized()` call (`compute_civilisation`'s own
//! `wb.classification`, not persisted past that function's local scope) and
//! hands [`PaintEditor::new`] a cheap second copy of the same call's
//! result. `PaintEditor` caches it (`Arc<[u8]>`, shared by every dab in
//! every layer, no per-dab clone) for its whole lifetime, which matches
//! `SculptEditor`'s own precedent for `WaterState`: state that is real
//! input to a tool, computed once at construction, not touched again until
//! the next `generate()` replaces the whole editor.
//!
//! This does mean a Sculpt commit that changes elevation under a painted
//! cell leaves this cache stale until the next full `generate()`.
//! `cartalith-spatial/src/paint.rs`'s own module doc already flags the
//! sibling question (whether a Sculpt commit should clear painted overrides
//! under the cells it touched) as "a real open question this port has and
//! the reference did not... nothing here decides it, because the deciding
//! caller (the shell, milestone F) does not exist yet." This cache is the
//! same open question, not a new one: fixed here would mean guessing at an
//! answer nothing has settled yet.

use std::sync::Arc;

use cartalith_civ::{CART_BIOMES, CART_TERRAINS};
use cartalith_assets::SPLAT_PAINT_SLOTS;
use cartalith_spatial::{js_round, CommitSummary, DirtyTracker, PaintLayer, PaintStamp, PassBuffer};

/// Tile granularity for every paint draft's `PassBuffer`/`DirtyTracker`
/// pair — the same value `sculpt_bridge::SCULPT_TILE_SIZE` uses and for the
/// same reason (see that constant's own doc comment): no reference value
/// to port against (the reference has no tiling concept at all here
/// either), picked so one dab at the largest legal radius
/// ([`PAINT_RADIUS_RANGE`]'s own 40) touches a handful of tiles rather than
/// one giant one or hundreds of tiny ones.
pub const PAINT_TILE_SIZE: usize = 64;

/// `_paintRadius`'s own reference range (`cartalith-spatial/src/paint.rs`'s
/// hypot-divergence test comment: *"every radius the reference's own
/// sliders can produce (`_paintRadius` 1..=40)"*). [`PaintEditor::
/// set_brush`] clamps to this.
pub const PAINT_RADIUS_RANGE: (f64, f64) = (1.0, 40.0);

/// Which of the three peer override arrays a dab writes —
/// `cartalith-spatial/src/paint.rs`'s own module doc: *"one `PaintStamp`
/// type serves all three, and the caller owns which array it is applied
/// to."* This is that caller-side selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintTarget {
    Biome,
    Terrain,
    Splat,
}

impl PaintTarget {
    pub const ALL: [PaintTarget; 3] = [PaintTarget::Biome, PaintTarget::Terrain, PaintTarget::Splat];

    /// The key a caller passes to `paint_set_layer` / receives from
    /// `get_paint_layers`.
    pub fn key(self) -> &'static str {
        match self {
            PaintTarget::Biome => "biome",
            PaintTarget::Terrain => "terrain",
            PaintTarget::Splat => "splat",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        PaintTarget::ALL.into_iter().find(|t| t.key() == key)
    }

    /// This layer's legal palette, 1-based (`palette()[0]` is legal index
    /// `1`) — `0` always means unpainted and is never a member of this
    /// list, matching every one of the three arrays' own convention
    /// (`cartalith-spatial/src/paint.rs`'s module doc).
    ///
    /// **Biome is 13 entries, not `CART_BIOMES`'s full 15.**
    /// `UNIFIED_TOOL_PLAN.md`'s own words: *"a value picker populated from
    /// `CART_BIOMES` (13 land biomes, water excluded — 'the brush never
    /// touches water')"* — `CART_BIOMES[13]`/`[14]` (`Lake`/`Ocean / Deep
    /// Water`) are real classifier outputs, not paintable values. Terrain
    /// and Splat expose their full arrays; `DCC_SHELL_SPEC.md` §4.5.2's own
    /// "answered" note names all three arrays without carving an exception
    /// out of either.
    ///
    /// Returns an owned `Vec` rather than a `&'static [_]` — the same
    /// choice `sculpt_bridge::global_controls()` makes and for the same
    /// reason: a slice of `CART_BIOMES` sliced to 13 is not itself a
    /// `'static` place, and duplicating the array as a second `const`
    /// would be exactly the hand-copy `sculpt_bridge`'s own doc warns
    /// against — this reads the live arrays instead, so it cannot drift
    /// from them.
    pub fn palette(self) -> Vec<&'static str> {
        match self {
            PaintTarget::Biome => CART_BIOMES[..13].to_vec(),
            PaintTarget::Terrain => CART_TERRAINS.to_vec(),
            PaintTarget::Splat => SPLAT_PAINT_SLOTS.to_vec(),
        }
    }

    /// The `DirtyTracker` reason string this layer's commit records —
    /// `"biome_painted"` matches `cartalith-spatial/src/paint.rs`'s and
    /// `cartalith-spatial/src/staleness.rs`'s own tests verbatim, so a
    /// caller reading `PassBuffer::commit`'s reason string sees the same
    /// vocabulary those crates already use.
    fn commit_reason(self) -> &'static str {
        match self {
            PaintTarget::Biome => "biome_painted",
            PaintTarget::Terrain => "terrain_painted",
            PaintTarget::Splat => "splat_painted",
        }
    }
}

/// The brush parameters `DCC_SHELL_SPEC.md` §4.5.2's `PAINT · BIOME` tool
/// options row exposes, after clamping. See this module's own doc for why
/// `hardness`/`softness` exist here but do nothing to a dab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    /// 1-based index into the *active* layer's own `PaintTarget::palette`.
    /// Meaningless while `erase` is set (a dab always writes `0` then), but
    /// kept rather than cleared, so turning `erase` back off restores the
    /// class the user had selected.
    pub value: u8,
    pub radius: f64,
    pub hardness: f64,
    pub softness: f64,
    pub erase: bool,
    pub land_only: bool,
}

impl Default for Brush {
    /// The reference's own defaults where it has one — `_paintRadius`'s
    /// default of 6 cells (`UNIFIED_TOOL_PLAN.md`) — and this port's own
    /// otherwise: `value = 1` (an arbitrary first selection, same
    /// "nothing to port, pick something reasonable" precedent
    /// `SculptEditor::new`'s own doc comment sets for `feature`),
    /// `hardness = 1.0`/`softness = 0.0` (this module's own doc: now
    /// genuinely read, and the exact pair `PaintStamp::with_falloff`
    /// treats as a zero-width band — an untouched brush paints the
    /// historical hard disc, which was already the honest description of
    /// "no falloff" before there was a falloff to make honest, and needed
    /// no change now that there is one), `erase = false`, `land_only =
    /// true` (the reference's own always-on gate — see this module's own
    /// doc on why it is a toggle here at all). Every layer shares this same
    /// default; there is no per-layer reason to differ.
    fn default() -> Self {
        Self {
            value: 1,
            radius: 6.0,
            hardness: 1.0,
            softness: 0.0,
            erase: false,
            land_only: true,
        }
    }
}

/// The live Paint-editor state for one generated world: which layer is
/// active, the live brush parameters, this world's own cached land-only
/// gate mask, and the three (draft, committed-layer) pairs — one each for
/// Biome/Terrain/Splat, sharing one [`DirtyTracker`] (see this module's own
/// doc on why three drafts rather than one).
pub struct PaintEditor {
    pub layer: PaintTarget,
    pub brush: Brush,
    /// This world's water-body classification (`cartalith_civ::
    /// build_water_bodies`'s own `wb.classification`, `0` = land) —
    /// [`PaintStamp::mask`]'s gate when [`Brush::land_only`] is set. See
    /// this module's own doc for why it is captured once at construction
    /// rather than recomputed per dab.
    water_mask: Arc<[u8]>,
    tracker: DirtyTracker,
    biome: PaintLayer,
    terrain: PaintLayer,
    splat: PaintLayer,
    draft_biome: PassBuffer<PaintStamp>,
    draft_terrain: PassBuffer<PaintStamp>,
    draft_splat: PassBuffer<PaintStamp>,
}

impl PaintEditor {
    /// A fresh editor over a `gw x gh` world — called once per
    /// `generate()`/`generate_sized()` (`WorldGen::absorb`), never reused
    /// across worlds, the same lifetime `SculptEditor::new`'s own doc
    /// comment describes. `water_mask` must be `gw * gh` long — a caller
    /// mismatch degrades to "the gate never excludes anything" rather than
    /// panicking, the same defensive posture `SculptEditor::new` takes for
    /// a mismatched `river_mask`/`river_floor` pair (see
    /// [`PaintStamp::apply`]'s own `mask.get(i)` — an out-of-range index
    /// reads as "ungate this cell", not a panic).
    pub fn new(gw: usize, gh: usize, water_mask: Arc<[u8]>) -> Self {
        let draft_biome = PassBuffer::new(gw, gh, PAINT_TILE_SIZE);
        let draft_terrain = PassBuffer::new(gw, gh, PAINT_TILE_SIZE);
        let draft_splat = PassBuffer::new(gw, gh, PAINT_TILE_SIZE);
        let tracker = DirtyTracker::new(draft_biome.tile_count());
        Self {
            layer: PaintTarget::Biome,
            brush: Brush::default(),
            water_mask,
            tracker,
            biome: PaintLayer::new(),
            terrain: PaintLayer::new(),
            splat: PaintLayer::new(),
            draft_biome,
            draft_terrain,
            draft_splat,
        }
    }

    pub fn active_layer(&self) -> &PaintLayer {
        match self.layer {
            PaintTarget::Biome => &self.biome,
            PaintTarget::Terrain => &self.terrain,
            PaintTarget::Splat => &self.splat,
        }
    }

    /// One named layer's *committed* cells, or `None` while that layer is
    /// still unallocated — `render::RenderCtx::with_paint`'s own input, and
    /// the reason the map can show a painted cell at all.
    ///
    /// Deliberately by [`PaintTarget`] rather than "the active one":
    /// `landColorCore` blends Biome and Terrain in the same pixel and lets
    /// Splat force a ground texture under both, so the renderer needs all
    /// three at once regardless of which one the brush is pointed at.
    pub fn layer_cells(&self, target: PaintTarget) -> Option<&[u8]> {
        match target {
            PaintTarget::Biome => self.biome.cells(),
            PaintTarget::Terrain => self.terrain.cells(),
            PaintTarget::Splat => self.splat.cells(),
        }
    }

    pub fn active_draft(&self) -> &PassBuffer<PaintStamp> {
        match self.layer {
            PaintTarget::Biome => &self.draft_biome,
            PaintTarget::Terrain => &self.draft_terrain,
            PaintTarget::Splat => &self.draft_splat,
        }
    }

    /// Pending, uncommitted dabs across **all three** drafts — exactly what
    /// [`PaintEditor::commit_all`] would bake and what
    /// [`PaintEditor::discard_all`] would throw away
    /// (`GUI_GAP_REGISTER.md` WW-13).
    ///
    /// This is deliberately not [`PaintEditor::painted_counts`]: that one
    /// reports the *composite* of the committed layer and the live draft,
    /// which is the right number for a legend and the wrong one for the
    /// Commit / Discard pair. Gated on the composite, both buttons stayed
    /// live after a commit with nothing left to commit or discard, and
    /// "Discard draft" then read as "remove the paint I can see" and did
    /// nothing at all.
    ///
    /// All three layers, not just the active one, for the same reason
    /// `commit_all` covers all three: a layer switch does not discard the
    /// layer left behind, so a pending dab on Terrain must keep Commit live
    /// while the panel is showing Biome.
    pub fn pending_stamps(&self) -> usize {
        self.draft_biome.len() + self.draft_terrain.len() + self.draft_splat.len()
    }

    fn active_draft_mut(&mut self) -> &mut PassBuffer<PaintStamp> {
        match self.layer {
            PaintTarget::Biome => &mut self.draft_biome,
            PaintTarget::Terrain => &mut self.draft_terrain,
            PaintTarget::Splat => &mut self.draft_splat,
        }
    }

    /// Replaces the cached land-only gate with a fresh classification —
    /// `PARITY_AUDIT.md` §23's third wiring item.
    ///
    /// **The cache stays a cache.** This module's own doc explains at length
    /// why the mask is captured once per `generate()` rather than recomputed
    /// per dab (`build_water_bodies` is a flood fill from every ocean edge,
    /// and `_paintAt` runs dozens of times a second during one drag; 417 ms
    /// of it per stroke on the measured world). Nothing about that changes:
    /// this is the *refresh* path the cache never had, called by the one op
    /// that edits the classification itself
    /// ([`crate::WorldGen::apply_force_lake`]), not by the brush.
    ///
    /// Same contract as [`PaintEditor::new`]'s own `water_mask`: `gw * gh`
    /// long, `0` = land, and a length mismatch degrades to "the gate never
    /// excludes anything" rather than panicking.
    ///
    /// Pending drafts are untouched by design, and that is a real decision
    /// rather than an omission: a [`PaintStamp`] captures its own `Arc` of
    /// the mask at [`PaintEditor::stroke_at`] time, so dabs already laid down
    /// keep the gate they were painted under and only the next dab sees the
    /// new water. Rewriting stamps already in the draft would change what the
    /// user has painted underneath them.
    pub fn set_water_mask(&mut self, water_mask: Arc<[u8]>) {
        self.water_mask = water_mask;
    }

    /// Replaces all three **committed** layers from `drafts/paint.json`'s
    /// sparse `[index, value, ...]` pair lists, and returns how many painted
    /// cells each one came back with.
    ///
    /// The reader half of `project_bridge.rs::paint_document_json`, and the
    /// reason a painted world reopens painted. Until this existed the
    /// document was written and never applied, so every biome, terrain and
    /// splat override a person had painted was in the archive and invisible.
    ///
    /// `n` is `gw * gh` for the world being restored **into**, and the caller
    /// is the one that must have checked it against the document's own grid:
    /// an index is a cell number, so a layer decoded against a different grid
    /// is not a smaller picture but a scrambled one
    /// ([`PaintLayer::decode_sparse`] silently drops an out-of-range index,
    /// which would turn that scrambling into a plausible-looking result).
    ///
    /// **Replaces, and clears the drafts with it.** A restore is the moment
    /// the layers become the file's; a pending dab from before it would bake
    /// into content it was never painted over. That is the same choice
    /// `sculpt_restore_document` makes for its own draft, for the same
    /// reason.
    pub fn restore_layers(
        &mut self,
        n: usize,
        biome: &[u32],
        terrain: &[u32],
        splat: &[u32],
    ) -> (usize, usize, usize) {
        self.biome = PaintLayer::decode_sparse(biome, n);
        self.terrain = PaintLayer::decode_sparse(terrain, n);
        self.splat = PaintLayer::decode_sparse(splat, n);
        self.draft_biome.discard();
        self.draft_terrain.discard();
        self.draft_splat.discard();
        let painted = |l: &PaintLayer| l.cells().map_or(0, |c| c.iter().filter(|&&v| v != 0).count());
        (painted(&self.biome), painted(&self.terrain), painted(&self.splat))
    }

    /// Switches which layer the next [`PaintEditor::stroke_at`] writes to.
    /// Each layer keeps its own draft (this module's own doc), so switching
    /// never discards or commits anything pending in the layer being left.
    ///
    /// Clamps `brush.value` into the new layer's own palette range —
    /// without this, a value valid for Splat's 6 slots could silently
    /// persist as an out-of-range Biome index (13 slots) after a switch,
    /// and the next dab would write a value `PaintTarget::palette` cannot
    /// explain.
    pub fn set_layer(&mut self, target: PaintTarget) {
        self.layer = target;
        let max = target.palette().len() as u8;
        if self.brush.value == 0 || self.brush.value > max {
            self.brush.value = 1;
        }
    }

    /// Applies a full brush parameter set, clamped, and returns what was
    /// actually stored. `value` clamps to the *current* layer's own
    /// palette size, minimum `1` — there is no legal "paint index 0", that
    /// value means erase and is controlled by the separate `erase` flag.
    /// `radius` clamps to [`PAINT_RADIUS_RANGE`]. `hardness`/`softness`
    /// clamp to `[0, 1]` (this module's own doc: now genuinely read, by
    /// [`PaintEditor::stroke_at`] via [`PaintStamp::with_falloff`]).
    /// A non-finite `radius`/`hardness`/`softness` is rejected outright
    /// (leaves the previous value in place) rather than clamped — the same
    /// "a NaN must never reach a stamp's own math" policy
    /// `sculpt_bridge::set_global`'s own doc comment defends.
    pub fn set_brush(&mut self, value: i64, radius: f64, hardness: f64, softness: f64, erase: bool, land_only: bool) -> Brush {
        let max = self.layer.palette().len().max(1) as i64;
        let value = value.clamp(1, max) as u8;
        let radius = if radius.is_finite() { radius.clamp(PAINT_RADIUS_RANGE.0, PAINT_RADIUS_RANGE.1) } else { self.brush.radius };
        let hardness = if hardness.is_finite() { hardness.clamp(0.0, 1.0) } else { self.brush.hardness };
        let softness = if softness.is_finite() { softness.clamp(0.0, 1.0) } else { self.brush.softness };
        self.brush = Brush { value, radius, hardness, softness, erase, land_only };
        self.brush
    }

    /// One brush dab at grid coordinates `(gx, gy)`, pushed straight onto
    /// the active layer's own draft as one complete [`PaintStamp`] —
    /// `_paintAt`'s own shape (this module's own doc): paint is a
    /// continuous drag with no captured polyline the way Sculpt's
    /// `sculpt_add_point`/`sculpt_end_stroke` pair needs, so every call
    /// here is already independently undo-able (`PassBuffer::push`'s own
    /// draft-scoped history), with no begin/end pair required.
    ///
    /// `gx`/`gy` round to the nearest grid cell via [`js_round`] — not
    /// because this specific rounding is golden-pinned (`_paintAt` is fed
    /// integer cell coordinates by the reference's own `_carPointerMove`,
    /// not a raw pointer position, so there is no reference value to match
    /// here at all), but because every other float-to-grid-cell
    /// conversion in this workspace goes through the JS-semantics helper
    /// rather than Rust's own `.round()` (`cartalith-rust-conventions`),
    /// and there is no reason for this, this port's one genuinely new
    /// interactive surface, to be the exception.
    pub fn stroke_at(&mut self, gx: f64, gy: f64) {
        let cx = js_round(gx) as i64;
        let cy = js_round(gy) as i64;
        let value = if self.brush.erase { 0 } else { self.brush.value };
        let radius = self.brush.radius;
        let land_only = self.brush.land_only;
        let mask = Arc::clone(&self.water_mask);
        let stamp = if land_only { PaintStamp::new(cx, cy, radius, value, mask) } else { PaintStamp::ungated(cx, cy, radius, value) }
            // `DECISIONS.md` §7k. Applied unconditionally -- an eraser dab
            // (`value == 0`) is geometrically the same disc as a paint dab,
            // just writing a different value, so it gets the same edge.
            .with_falloff(self.brush.hardness, self.brush.softness);
        self.active_draft_mut().push(stamp);
    }

    /// Per-class painted-cell counts for the *active* layer, live: the
    /// already-committed layer composited with its own pending draft
    /// (`PassBuffer::preview_into`, never mutating either) — `DCC_SHELL_
    /// SPEC.md` §4.5.2's right dock wants a running total while Commit/
    /// Discard are still available, not only after committing.
    ///
    /// `n` must be `gw * gh` for this editor's own world — the caller's
    /// responsibility, the same contract `PassBuffer::preview_into` itself
    /// documents (it asserts this internally). Returns `(total, by_class)`
    /// where `by_class[i]` is the count for 1-based palette index `i + 1`,
    /// one entry per legal index of the active layer's own palette —
    /// zero-count entries included, so a legend can render every class
    /// every time rather than only the ones currently painted.
    pub fn painted_counts(&self, n: usize) -> (i64, Vec<i64>) {
        let base: Vec<u8> = self.active_layer().cells().map(<[u8]>::to_vec).unwrap_or_else(|| vec![0u8; n]);
        let mut scratch = vec![0u8; n];
        self.active_draft().preview_into(&base, &mut scratch);

        let palette_len = self.layer.palette().len();
        let mut by_class = vec![0i64; palette_len];
        for &v in &scratch {
            if v != 0 {
                let idx = v as usize;
                if idx <= palette_len {
                    by_class[idx - 1] += 1;
                }
            }
        }
        let total = by_class.iter().sum();
        (total, by_class)
    }

    /// Bakes every layer's pending draft into its own [`PaintLayer`] and
    /// clears all three drafts — every layer that actually has a pending
    /// dab, not only the active one (this module's own doc: a layer switch
    /// must not lose the layer left behind).
    ///
    /// A layer with an empty draft is skipped entirely, **not** committed
    /// as a documented no-op the way `PassBuffer::commit`'s own "an empty
    /// commit writes nothing and marks nothing" would suggest is harmless
    /// either way: committing still requires `PaintLayer::cells_mut`, which
    /// *allocates* the layer's backing array on first use even when
    /// nothing is about to be written into it. A layer nobody ever painted
    /// must stay [`PaintLayer::is_unallocated`], not merely
    /// [`PaintLayer::is_empty`] — the distinction `cartalith-spatial/src/
    /// paint.rs` itself keeps (`is_unallocated`'s own doc: "an unpainted
    /// layer costs nothing").
    ///
    /// `n` must be `gw * gh` for this editor's own world (`PaintLayer::
    /// cells_mut`'s own reallocate-on-mismatch guard makes a wrong `n`
    /// degrade to "start this layer over" rather than corrupt data, but the
    /// caller should still always pass the real grid size).
    ///
    /// **Deliberately does not touch `field`/`temperature`/`rainfall` at
    /// all** — nothing in this method borrows a `WorldState` in the first
    /// place, so `UNIFIED_TOOL_PLAN.md`'s own Biome-paint staleness rule
    /// (*"painting biome does not mark height/hydrology/climate dirty"*)
    /// holds by construction, not by a check. See `lib.rs`'s own
    /// `paint_commit` for how the DCC shell's stage-09/10 staleness note
    /// is surfaced from this summary.
    pub fn commit_all(&mut self, n: usize) -> [CommitSummary; 3] {
        fn commit_one(draft: &mut PassBuffer<PaintStamp>, layer: &mut PaintLayer, tracker: &mut DirtyTracker, n: usize, reason: &str) -> CommitSummary {
            if draft.is_empty() {
                return CommitSummary { stamps_applied: 0, stamps_skipped: 0, tiles_marked: Vec::new() };
            }
            draft.commit(layer.cells_mut(n), tracker, reason)
        }
        let biome = commit_one(&mut self.draft_biome, &mut self.biome, &mut self.tracker, n, PaintTarget::Biome.commit_reason());
        let terrain = commit_one(&mut self.draft_terrain, &mut self.terrain, &mut self.tracker, n, PaintTarget::Terrain.commit_reason());
        let splat = commit_one(&mut self.draft_splat, &mut self.splat, &mut self.tracker, n, PaintTarget::Splat.commit_reason());
        [biome, terrain, splat]
    }

    /// Drops every layer's pending draft, touching nothing committed.
    /// Returns how many dabs were dropped in total, across all three.
    pub fn discard_all(&mut self) -> usize {
        self.draft_biome.discard() + self.draft_terrain.discard() + self.draft_splat.discard()
    }
}

/// The swatch colour for 1-based palette `index` of `target` (`0` or an
/// out-of-range index both return black, fully-transparent-in-practice
/// since `lib.rs`'s own `build_paint_preview_texture` never calls this for
/// an unpainted cell).
///
/// **Biome and Terrain are the reference's own literal tables**,
/// `CART_BIOME_COLS` (reference 6813) and `CART_TERRAIN_COLS` (6858) — the
/// exact colours `landColorCore` blends into the map at weight `0.60`
/// (`render::land_color`'s own paint blend), so the overlay preview and the
/// committed map now name a class with the same colour instead of two
/// unrelated ones.
///
/// This function previously spaced every index around the hue wheel, on the
/// stated grounds that "no literal RGB table behind it ... has been ported".
/// That was true when it was written and had stopped being true: both
/// tables were already in this crate, for the `bclass`/`cterrain` debug
/// views. Corrected 2026-08-24; the generated-hue path survives only for
/// Splat, which genuinely has no reference colour.
///
/// **Splat keeps the generated hue, and that is the honest answer, not a
/// leftover.** `SPLAT_PAINT_SLOTS` names *ground textures*, not colours:
/// the reference renders a painted splat cell by forcing that pack
/// texture's own pixels at full coverage (7765-7773), so a splat class has
/// no swatch colour to port — only a texture, which is a different thing
/// and is not something a flat overlay can show.
pub fn swatch_color(target: PaintTarget, index: u8, palette_len: usize) -> (u8, u8, u8) {
    if index == 0 || palette_len == 0 || index as usize > palette_len {
        return (0, 0, 0);
    }
    let i = index as usize - 1;
    match target {
        PaintTarget::Biome => crate::render::CART_BIOME_COLS[i],
        PaintTarget::Terrain => crate::render::CART_TERRAIN_COLS[i],
        PaintTarget::Splat => hsv_to_rgb(((index - 1) as f64 / palette_len as f64) * 360.0, 0.65, 0.95),
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (((r1 + m) * 255.0).round() as u8, ((g1 + m) * 255.0).round() as u8, ((b1 + m) * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Only the falloff tests below call `PaintStamp::apply` directly, to
    // build a same-shape comparison stamp -- everything else in this module
    // reaches it indirectly through `PassBuffer`.
    use cartalith_spatial::Stamp;

    fn land_mask(n: usize) -> Arc<[u8]> {
        vec![0u8; n].into()
    }

    // ---- registry ----

    #[test]
    fn every_target_round_trips_through_its_own_key() {
        for t in PaintTarget::ALL {
            assert_eq!(PaintTarget::from_key(t.key()), Some(t));
        }
        assert_eq!(PaintTarget::from_key("nope"), None);
    }

    #[test]
    fn biome_palette_is_13_not_all_15_cart_biomes() {
        let p = PaintTarget::Biome.palette();
        assert_eq!(p.len(), 13);
        assert!(!p.contains(&"Lake"), "water classes must not be paintable");
        assert!(!p.contains(&"Ocean / Deep Water"));
        assert_eq!(p[0], CART_BIOMES[0]);
        assert_eq!(p[12], CART_BIOMES[12]);
    }

    #[test]
    fn terrain_and_splat_palettes_match_their_full_arrays() {
        assert_eq!(PaintTarget::Terrain.palette(), CART_TERRAINS.to_vec());
        assert_eq!(PaintTarget::Splat.palette(), SPLAT_PAINT_SLOTS.to_vec());
    }

    // ---- editor construction ----

    #[test]
    fn new_editor_starts_on_biome_with_empty_layers_and_the_reference_default_brush() {
        let e = PaintEditor::new(16, 12, land_mask(16 * 12));
        assert_eq!(e.layer, PaintTarget::Biome);
        assert_eq!(e.brush.radius, 6.0);
        assert!(e.brush.land_only, "the reference's own gate is always on");
        assert!(!e.brush.erase);
        assert!(e.active_layer().is_unallocated());
        assert!(e.active_draft().is_empty());
    }

    // ---- set_layer ----

    #[test]
    fn set_layer_switches_the_active_draft_and_layer() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_layer(PaintTarget::Splat);
        assert_eq!(e.layer, PaintTarget::Splat);
    }

    #[test]
    fn set_layer_clamps_a_value_that_would_be_illegal_in_the_new_layer() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(13, 6.0, 1.0, 0.0, false, true); // legal for Biome's 13 slots
        e.set_layer(PaintTarget::Splat); // only 6 slots
        assert_eq!(e.brush.value, 1, "an out-of-range value must reset, not persist silently");
    }

    #[test]
    fn set_layer_leaves_an_in_range_value_untouched() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(4, 6.0, 1.0, 0.0, false, true);
        e.set_layer(PaintTarget::Terrain); // 13 slots, 4 still legal
        assert_eq!(e.brush.value, 4);
    }

    // ---- set_brush ----

    #[test]
    fn set_brush_clamps_every_field_and_reports_what_was_stored() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        let b = e.set_brush(999, 9999.0, 5.0, -5.0, true, false);
        assert_eq!(b.value, 13, "clamped to Biome's own 13-slot palette");
        assert_eq!(b.radius, PAINT_RADIUS_RANGE.1);
        assert_eq!(b.hardness, 1.0);
        assert_eq!(b.softness, 0.0);
        assert!(b.erase);
        assert!(!b.land_only);
        assert_eq!(e.brush, b);
    }

    #[test]
    fn set_brush_rejects_non_finite_values_and_keeps_the_previous_ones() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(3, 10.0, 0.5, 0.5, false, true);
        let before = e.brush;
        let after = e.set_brush(3, f64::NAN, f64::NAN, f64::NAN, false, true);
        assert_eq!(after.radius, before.radius);
        assert_eq!(after.hardness, before.hardness);
        assert_eq!(after.softness, before.softness);
    }

    // ---- stroke_at / drafting ----

    #[test]
    fn stroke_at_pushes_into_the_active_layers_own_draft_only() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(5, 3.0, 1.0, 0.0, false, false);
        e.stroke_at(8.0, 6.0);
        assert_eq!(e.draft_biome.len(), 1);
        assert_eq!(e.draft_terrain.len(), 0);
        assert_eq!(e.draft_splat.len(), 0);
    }

    #[test]
    fn stroke_at_rounds_to_the_nearest_cell() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(5, 0.4, 1.0, 0.0, false, false); // radius clamps up to PAINT_RADIUS_RANGE's 1.0 minimum
        e.stroke_at(8.3, 6.6);
        let n = 16 * 12;
        let base = vec![0u8; n];
        let mut scratch = vec![0u8; n];
        e.draft_biome.preview_into(&base, &mut scratch);
        assert_eq!(scratch[7 * 16 + 8], 5, "(8.3, 6.6) rounds to grid cell (8, 7)");
    }

    #[test]
    fn erase_writes_zero_regardless_of_the_stored_value() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(7, 3.0, 1.0, 0.0, true, false);
        e.stroke_at(8.0, 8.0);
        let n = 16 * 12;
        let mut layer = PaintLayer::new();
        layer.cells_mut(n)[8 * 16 + 8] = 7; // pretend something was painted here before
        let mut scratch = vec![0u8; n];
        e.draft_biome.preview_into(layer.cells().unwrap(), &mut scratch);
        assert_eq!(scratch[8 * 16 + 8], 0);
    }

    #[test]
    fn land_only_gates_against_the_captured_water_mask() {
        let n = 16 * 12;
        let mut mask = vec![0u8; n];
        mask[8 * 16 + 8] = 2; // lake at the dab's own centre
        let mut e = PaintEditor::new(16, 12, mask.into());
        e.set_brush(5, 0.0, 1.0, 0.0, false, true); // clamps up to radius 1.0; only the centre is masked
        e.stroke_at(8.0, 8.0);
        let base = vec![0u8; n];
        let mut scratch = vec![0u8; n];
        e.draft_biome.preview_into(&base, &mut scratch);
        assert_eq!(scratch[8 * 16 + 8], 0, "gated cell must stay unpainted");
    }

    #[test]
    fn land_only_off_paints_straight_through_the_water_mask() {
        let n = 16 * 12;
        let mut mask = vec![0u8; n];
        mask[8 * 16 + 8] = 2;
        let mut e = PaintEditor::new(16, 12, mask.into());
        e.set_brush(5, 0.0, 1.0, 0.0, false, false);
        e.stroke_at(8.0, 8.0);
        let base = vec![0u8; n];
        let mut scratch = vec![0u8; n];
        e.draft_biome.preview_into(&base, &mut scratch);
        assert_eq!(scratch[8 * 16 + 8], 5, "the ungated affordance really does bypass the mask");
    }

    // ---- brush falloff (`DECISIONS.md` §7k) ----
    //
    // `cartalith-spatial/src/paint.rs`'s own test module proves the
    // mechanism directly against `PaintStamp`; these exercise the same two
    // claims through the real public surface `stroke_at` actually is, using
    // `Brush`'s own `hardness`/`softness` names end to end.

    #[test]
    fn stroke_at_with_the_default_brush_paints_the_historical_hard_disc() {
        // hardness=1.0, softness=0.0 is `Brush::default()`'s own pair --
        // the exact claim `DECISIONS.md` §7k rests its bit-identity promise
        // on. A big enough grid/radius that the falloff test below can
        // reuse the same shape.
        let n = 40 * 40;
        let mut e = PaintEditor::new(40, 40, land_mask(n));
        e.set_brush(5, 20.0, 1.0, 0.0, false, false);
        e.stroke_at(20.0, 20.0);
        let base = vec![0u8; n];
        let mut got = vec![0u8; n];
        e.draft_biome.preview_into(&base, &mut got);

        let mut want = vec![0u8; n];
        PaintStamp::ungated(20, 20, 20.0, 5).apply(&mut want, 40, 40);
        assert_eq!(got, want, "an untouched brush must reproduce PaintStamp's own hard disc exactly");
    }

    #[test]
    fn stroke_at_with_hardness_below_one_measurably_feathers_the_edge() {
        let n = 40 * 40;
        let mut e = PaintEditor::new(40, 40, land_mask(n));
        e.set_brush(5, 20.0, 0.4, 0.0, false, false);
        e.stroke_at(20.0, 20.0);
        let base = vec![0u8; n];
        let mut soft = vec![0u8; n];
        e.draft_biome.preview_into(&base, &mut soft);

        let mut hard = vec![0u8; n];
        PaintStamp::ungated(20, 20, 20.0, 5).apply(&mut hard, 40, 40);

        assert_ne!(soft, hard, "hardness=0.4 must paint a different set than the hard disc");
        let soft_count = soft.iter().filter(|&&v| v != 0).count();
        let hard_count = hard.iter().filter(|&&v| v != 0).count();
        assert!(soft_count < hard_count, "a feathered edge only ever drops cells, never adds them");
        assert!(soft_count > 0, "the disc's interior must still paint something");
    }

    #[test]
    fn set_brush_round_trips_hardness_and_softness_into_the_next_stamp() {
        // `set_brush` and `stroke_at` are two calls, not one -- this pins
        // that the value `set_brush` stores is really what the following
        // `stroke_at` hands to `PaintStamp::with_falloff`, not a separate
        // copy that could drift.
        let n = 40 * 40;
        let mut e = PaintEditor::new(40, 40, land_mask(n));
        e.set_brush(5, 20.0, 0.4, 0.0, false, false);
        e.stroke_at(20.0, 20.0);
        let base = vec![0u8; n];
        let mut via_editor = vec![0u8; n];
        e.draft_biome.preview_into(&base, &mut via_editor);

        let mut via_stamp = vec![0u8; n];
        PaintStamp::ungated(20, 20, 20.0, 5).with_falloff(0.4, 0.0).apply(&mut via_stamp, 40, 40);
        assert_eq!(via_editor, via_stamp);
    }

    // ---- painted_counts ----

    #[test]
    fn painted_counts_reflects_the_live_draft_not_only_the_committed_layer() {
        // Radius 1.0 (`PAINT_RADIUS_RANGE`'s own minimum) paints a 5-cell
        // "plus", not a single cell (`radius_one_paints_a_plus_not_a_square`
        // in `cartalith-spatial/src/paint.rs`) -- the two centres below are
        // placed far enough apart that their two pluses cannot overlap, so
        // this test's counts aren't sensitive to that shape.
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        e.set_brush(2, 1.0, 1.0, 0.0, false, false);
        e.stroke_at(1.0, 1.0);
        e.set_brush(9, 1.0, 1.0, 0.0, false, false);
        e.stroke_at(10.0, 8.0);
        let (total, by_class) = e.painted_counts(n);
        assert_eq!(total, 10);
        assert_eq!(by_class[1], 5); // class 2 -> index 1
        assert_eq!(by_class[8], 5); // class 9 -> index 8
        assert_eq!(by_class.len(), 13);
    }

    // ---- commit_all / discard_all ----

    #[test]
    fn commit_all_bakes_every_layer_that_has_pending_work_and_clears_all_drafts() {
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        e.set_brush(2, 0.0, 1.0, 0.0, false, false);
        e.stroke_at(1.0, 1.0);
        e.set_layer(PaintTarget::Terrain);
        e.set_brush(4, 0.0, 1.0, 0.0, false, false);
        e.stroke_at(3.0, 3.0);

        let [biome, terrain, splat] = e.commit_all(n);
        assert_eq!(biome.stamps_applied, 1);
        assert_eq!(terrain.stamps_applied, 1);
        assert_eq!(splat.stamps_applied, 0);
        assert!(e.draft_biome.is_empty());
        assert!(e.draft_terrain.is_empty());
        assert_eq!(e.biome.cells().unwrap()[1 * 16 + 1], 2);
        assert_eq!(e.terrain.cells().unwrap()[3 * 16 + 3], 4);
    }

    /// `GUI_GAP_REGISTER.md` WW-13 — the exact divergence the Commit /
    /// Discard pair was gated on the wrong side of: after a commit,
    /// `painted_counts` still reports the painted cells (correctly — they
    /// exist), while `pending_stamps` goes to zero (correctly — there is
    /// nothing left to commit or discard).
    #[test]
    fn pending_stamps_counts_every_layers_draft_and_goes_to_zero_on_commit() {
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        assert_eq!(e.pending_stamps(), 0, "a fresh editor has nothing pending");

        e.set_brush(2, 1.0, 1.0, 0.0, false, false);
        e.stroke_at(1.0, 1.0);
        assert_eq!(e.pending_stamps(), 1);

        // A layer switch does not discard the layer left behind, so the
        // count must still see the Biome dab from inside Terrain.
        e.set_layer(PaintTarget::Terrain);
        e.set_brush(4, 1.0, 1.0, 0.0, false, false);
        e.stroke_at(10.0, 8.0);
        assert_eq!(e.pending_stamps(), 2, "all three drafts, not just the active one");

        let (total_before, _) = e.painted_counts(n);
        e.commit_all(n);
        assert_eq!(e.pending_stamps(), 0, "nothing left to commit or discard");
        let (total_after, _) = e.painted_counts(n);
        assert_eq!(
            total_after, total_before,
            "the composite total is unchanged by a commit -- which is exactly why it \
             is the wrong number to gate Commit / Discard on"
        );
        assert!(total_after > 0);
    }

    #[test]
    fn pending_stamps_goes_to_zero_on_discard_too() {
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        e.set_brush(2, 1.0, 1.0, 0.0, false, false);
        e.stroke_at(1.0, 1.0);
        e.stroke_at(4.0, 4.0);
        assert_eq!(e.pending_stamps(), 2);
        e.discard_all();
        assert_eq!(e.pending_stamps(), 0);
        let (total, _) = e.painted_counts(n);
        assert_eq!(total, 0, "a discard really did remove the cells too");
    }

    #[test]
    fn commit_all_never_touches_a_layer_with_nothing_pending() {
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        e.set_brush(2, 0.0, 1.0, 0.0, false, false);
        e.stroke_at(1.0, 1.0);
        e.commit_all(n);
        assert!(e.terrain.is_unallocated(), "an untouched layer stays unallocated, not merely empty");
        assert!(e.splat.is_unallocated());
    }

    #[test]
    fn discard_all_drops_every_pending_dab_and_touches_no_committed_layer() {
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        e.set_brush(2, 0.0, 1.0, 0.0, false, false);
        e.stroke_at(1.0, 1.0);
        e.set_layer(PaintTarget::Splat);
        e.set_brush(3, 0.0, 1.0, 0.0, false, false);
        e.stroke_at(2.0, 2.0);

        let dropped = e.discard_all();
        assert_eq!(dropped, 2);
        assert!(e.draft_biome.is_empty());
        assert!(e.draft_splat.is_empty());
        assert!(e.biome.is_unallocated());
        assert!(e.splat.is_unallocated());
    }

    // ---- swatch_color ----

    #[test]
    fn swatch_color_is_stable_and_distinct_across_a_palette() {
        for t in PaintTarget::ALL {
            let n = t.palette().len();
            let colors: Vec<_> = (1..=n as u8).map(|i| swatch_color(t, i, n)).collect();
            for i in 1..=n as u8 {
                assert_eq!(swatch_color(t, i, n), colors[i as usize - 1], "must be stable across calls");
            }
            let unique: std::collections::HashSet<_> = colors.iter().copied().collect();
            assert_eq!(unique.len(), colors.len(), "{t:?}: every class should get its own colour");
        }
    }

    /// The correction this function needed: the overlay preview and the
    /// committed map must name a class with the *same* colour, and that
    /// colour is the reference's own table, not a generated hue.
    #[test]
    fn biome_and_terrain_swatches_are_the_reference_tables_the_renderer_blends() {
        for i in 1..=13u8 {
            assert_eq!(swatch_color(PaintTarget::Biome, i, 13), crate::render::CART_BIOME_COLS[i as usize - 1]);
            assert_eq!(swatch_color(PaintTarget::Terrain, i, 13), crate::render::CART_TERRAIN_COLS[i as usize - 1]);
        }
        // Spot-check two literal reference values so a table edit cannot
        // pass this by moving both sides together.
        assert_eq!(swatch_color(PaintTarget::Biome, 2, 13), (58, 122, 74), "CART_BIOME_COLS[1], Temperate Forest");
        assert_eq!(swatch_color(PaintTarget::Terrain, 2, 13), (154, 122, 74), "CART_TERRAIN_COLS[1]");
    }

    /// Splat has no reference colour at all — it names pack textures. Kept
    /// on the generated hue, and pinned so the distinction stays deliberate.
    #[test]
    fn splat_swatches_stay_generated_because_the_reference_has_no_colour_for_them() {
        assert_ne!(swatch_color(PaintTarget::Splat, 1, 6), crate::render::CART_BIOME_COLS[0]);
        assert_eq!(swatch_color(PaintTarget::Splat, 1, 6), swatch_color(PaintTarget::Splat, 1, 6));
    }

    #[test]
    fn swatch_color_is_black_for_unpainted_or_out_of_range() {
        for t in PaintTarget::ALL {
            assert_eq!(swatch_color(t, 0, 13), (0, 0, 0));
            assert_eq!(swatch_color(t, 99, 13), (0, 0, 0));
            assert_eq!(swatch_color(t, 1, 0), (0, 0, 0));
        }
    }

    #[test]
    fn layer_cells_reports_only_committed_state() {
        let n = 16 * 12;
        let mut e = PaintEditor::new(16, 12, land_mask(n));
        assert!(e.layer_cells(PaintTarget::Biome).is_none(), "unallocated before anything is painted");
        e.set_brush(2, 1.0, 1.0, 0.0, false, false);
        e.stroke_at(4.0, 4.0);
        assert!(e.layer_cells(PaintTarget::Biome).is_none(), "a pending draft is not committed state");
        e.commit_all(n);
        assert_eq!(e.layer_cells(PaintTarget::Biome).unwrap()[4 * 16 + 4], 2);
        assert!(e.layer_cells(PaintTarget::Splat).is_none(), "an untouched layer stays unallocated");
    }
}
