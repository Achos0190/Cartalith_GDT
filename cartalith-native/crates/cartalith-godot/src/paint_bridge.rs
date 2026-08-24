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
//! ## `hardness`/`softness` are accepted, not consumed
//!
//! `cartalith-spatial/src/paint.rs`'s own module doc is unambiguous:
//! painting is "a hard disc... unlike `sculpt()`/`brushHeight` there's no
//! soft falloff here" (the reference's own comment, quoted there verbatim).
//! `DCC_SHELL_SPEC.md` §4.5.2's tool options row nonetheless lists
//! `hardness`/`softness` on the `PAINT · BIOME` row — almost certainly
//! carried over from the Sculpt row's own shape rather than a deliberate
//! new paint behaviour, since nothing in the reference or in
//! `cartalith-spatial::paint` gives either one a meaning for a categorical
//! brush. [`Brush`] stores and round-trips both anyway, so a shell built
//! against that row does not have to special-case two of its own fields —
//! but [`PaintStamp::apply`] never reads them, and [`PaintEditor::
//! stroke_at`] never passes them to it. If the design intent turns out to
//! be real (a soft-edged alpha ramp on top of the hard disc, say), that is
//! new engine work in `cartalith-spatial`, not something to improvise here.
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
    /// `hardness = 1.0`/`softness = 0.0` (the pair that actually describes
    /// what a hard disc with no falloff does, even though neither is read
    /// — a default that lies about the brush's own behaviour would be
    /// worse than a merely-arbitrary one), `erase = false`, `land_only =
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

    pub fn active_draft(&self) -> &PassBuffer<PaintStamp> {
        match self.layer {
            PaintTarget::Biome => &self.draft_biome,
            PaintTarget::Terrain => &self.draft_terrain,
            PaintTarget::Splat => &self.draft_splat,
        }
    }

    fn active_draft_mut(&mut self) -> &mut PassBuffer<PaintStamp> {
        match self.layer {
            PaintTarget::Biome => &mut self.draft_biome,
            PaintTarget::Terrain => &mut self.draft_terrain,
            PaintTarget::Splat => &mut self.draft_splat,
        }
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
    /// clamp to `[0, 1]` (this module's own doc: stored, never read).
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
        let stamp = if land_only { PaintStamp::new(cx, cy, radius, value, mask) } else { PaintStamp::ungated(cx, cy, radius, value) };
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

/// A deterministic, distinct swatch colour for 1-based palette `index` out
/// of `palette_len` total classes (`0` or an out-of-range index both
/// return black, fully-transparent-in-practice since `lib.rs`'s own
/// `build_paint_preview_texture` never calls this for an unpainted cell).
///
/// **This port's own convention, not the reference's.** The reference's
/// real painted-cell colour (`landColorCore`, a 0.60-alpha blend of "the
/// painted index's palette colour" over the fully shaded procedural
/// colour, per `cartalith-spatial/src/paint.rs`'s own doc) has no *literal*
/// RGB table behind it that this workspace has ported —
/// `CART_BIOMES`/`CART_TERRAINS`/`SPLAT_PAINT_SLOTS` are label strings, and
/// `UNIFIED_TOOL_PLAN.md` itself records that "no producer of a
/// painted-cell array" has been wired into the renderer at all yet. Rather
/// than block a live preview on that unported table, or invent literal RGB
/// constants and present them as if they were a real port of it, this
/// spaces every index evenly around the hue wheel: stable across calls
/// (the same index always gets the same colour), visually distinct for any
/// palette this port currently ships (at most 15 classes), and honestly a
/// new convention rather than a guessed parity value.
pub fn swatch_color(index: u8, palette_len: usize) -> (u8, u8, u8) {
    if index == 0 || palette_len == 0 || index as usize > palette_len {
        return (0, 0, 0);
    }
    let hue = ((index - 1) as f64 / palette_len as f64) * 360.0;
    hsv_to_rgb(hue, 0.65, 0.95)
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
        let colors: Vec<_> = (1..=13u8).map(|i| swatch_color(i, 13)).collect();
        for i in 1..=13u8 {
            assert_eq!(swatch_color(i, 13), colors[i as usize - 1], "must be stable across calls");
        }
        let unique: std::collections::HashSet<_> = colors.iter().copied().collect();
        assert_eq!(unique.len(), colors.len(), "every class in a 13-entry palette should get its own colour");
    }

    #[test]
    fn swatch_color_is_black_for_unpainted_or_out_of_range() {
        assert_eq!(swatch_color(0, 13), (0, 0, 0));
        assert_eq!(swatch_color(99, 13), (0, 0, 0));
        assert_eq!(swatch_color(1, 0), (0, 0, 0));
    }
}
