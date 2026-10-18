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
//!
//! The sibling question -- whether a Sculpt commit should clear painted
//! overrides under the cells it touched -- was open here until the owner
//! answered it (Ruling AS, 2026-09-24, `LARGE_ITEM_RULINGS.md`): **it
//! does.** `WorldGen::sculpt_commit` takes the draft's
//! `SculptEditor::footprint` and hands it to
//! [`PaintEditor::clear_cells_under`], which zeroes all three committed
//! layers there. This water-mask cache is a separate question the ruling did
//! not touch, and it is still refreshed only by a regenerate or
//! [`PaintEditor::set_water_mask`].

use std::sync::Arc;

use cartalith_civ::{CART_BIOMES, CART_TERRAINS};
use cartalith_assets::SPLAT_PAINT_SLOTS;
use cartalith_spatial::stroke::{Dab, Dynamic, Scatter, Sample, Sensor, StrokeConfig, StrokeEngine, StrokeError, Symmetry};
use cartalith_spatial::{js_round, CommitSummary, DirtyTracker, EdgeShape, PaintLayer, PaintStamp, PassBuffer, Region};

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
    /// The biome override array (`paintBiome` in the reference).
    Biome,
    /// The terrain override array (`paintTerrain` in the reference).
    Terrain,
    /// The ground-texture splat override array (`paintSplat` in the reference).
    Splat,
    /// GF-8 (`GEOLOGY_FIRST_SCOPE.md` §4.13, Ruling BJ): the **surface rock**
    /// a commit bakes into the geology column's `rock_top`. Value = rock index
    /// + 1 (`1 ..= 11`). Not an override array: see [`PaintTarget::ROCK`].
    RockTop,
    /// GF-8: the **rock beneath** (`rock_sub`). Value = rock index + 1, or
    /// [`cartalith_engine::rock_paint::SUB_NONE`] (255) for "none: single layer".
    RockSub,
    /// GF-8: the **cap thickness**, in classes of
    /// [`cartalith_engine::rock_paint::CAP_CLASS_M`] (10 m): value `k`
    /// (`1 ..= 255`) is `10·k` m. Sets the column's `contact`.
    RockCap,
}

impl PaintTarget {
    /// The three **override-array** layers, in declaration order: what
    /// `get_paint_layers` lists, what `drafts/paint.json` saves
    /// (`project_bridge.rs::paint_document_json`) and what the renderer
    /// composites. The Rock layers are deliberately not in it (see
    /// [`PaintTarget::ROCK`]), so adding them moved neither the save document
    /// nor the target list an older shell reads.
    pub const ALL: [PaintTarget; 3] = [PaintTarget::Biome, PaintTarget::Terrain, PaintTarget::Splat];

    /// GF-8's three Rock layers (§4.13: "three new `PaintLayer`s in
    /// `PaintEditor`"). They are **drafts only**: a commit bakes them into the
    /// geology column (`cartalith_engine::rock_paint::commit`), as a sculpt
    /// commit bakes into the heightmap, so there is no committed Rock layer to
    /// render, save or composite. Their drafts are not saved yet either: the
    /// whole geology save set is GF-8's third slice (the scope's §7 "GF-8
    /// measured: the engine half").
    pub const ROCK: [PaintTarget; 3] = [PaintTarget::RockTop, PaintTarget::RockSub, PaintTarget::RockCap];

    /// The key a caller passes to `paint_set_layer` / receives from
    /// `get_paint_layers` (the three override layers) or
    /// `rock_paint_status` (the three Rock layers).
    pub fn key(self) -> &'static str {
        match self {
            PaintTarget::Biome => "biome",
            PaintTarget::Terrain => "terrain",
            PaintTarget::Splat => "splat",
            PaintTarget::RockTop => "rock_top",
            PaintTarget::RockSub => "rock_sub",
            PaintTarget::RockCap => "rock_cap",
        }
    }

    /// The inverse of [`PaintTarget::key`]: `None` for any string that is not one of
    /// the six keys, so a stale or mistyped key from the shell is rejected rather
    /// than defaulted (defaulting to Biome would silently paint the wrong layer).
    /// Must never panic: the key arrives from GDScript.
    pub fn from_key(key: &str) -> Option<Self> {
        PaintTarget::ALL.into_iter().chain(PaintTarget::ROCK).find(|t| t.key() == key)
    }

    /// Whether this is one of GF-8's Rock layers ([`PaintTarget::ROCK`]).
    pub fn is_rock(self) -> bool {
        PaintTarget::ROCK.contains(&self)
    }

    /// Every legal non-zero value this layer's dabs may write, in legend
    /// order. For the override layers and `RockTop` that is `1 ..=
    /// palette().len()`; `RockSub` adds `SUB_NONE` (255) after the eleven
    /// rocks; `RockCap` is every class `1 ..= 255` (it has no palette of
    /// names). [`PaintEditor::set_brush`] clamps into this set and
    /// [`PaintEditor::painted_counts`] counts by it, so the two cannot
    /// disagree about what a layer may hold.
    pub fn legal_values(self) -> Vec<u8> {
        match self {
            PaintTarget::RockCap => (1..=255u8).collect(),
            PaintTarget::RockSub => {
                let mut v: Vec<u8> = (1..=self.palette().len() as u8).collect();
                v.push(cartalith_engine::rock_paint::SUB_NONE);
                v
            }
            _ => (1..=self.palette().len() as u8).collect(),
        }
    }

    /// `value` mapped onto [`PaintTarget::legal_values`]: kept when legal,
    /// otherwise clamped into `1 ..= max` of the contiguous part (so a
    /// `RockSub` 200 becomes 11, never a silent `SUB_NONE`). Never `0`: that
    /// means erase, which is the separate `erase` flag.
    pub fn legal_value(self, value: i64) -> u8 {
        if self == PaintTarget::RockSub && value == cartalith_engine::rock_paint::SUB_NONE as i64 {
            return cartalith_engine::rock_paint::SUB_NONE;
        }
        let max = match self {
            PaintTarget::RockCap => 255,
            _ => self.palette().len().max(1) as i64,
        };
        value.clamp(1, max) as u8
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
    ///
    /// **Rock** (GF-8): `RockTop` and `RockSub` name the eleven rocks of
    /// `cartalith_terrain::geology::ROCK_PROPS`, read live in `Rock::ALL`
    /// order (index `i + 1` is rock `i`); `RockSub`'s "none" (255) is not a
    /// palette position (see [`PaintTarget::legal_values`]). `RockCap` has no
    /// names, only thickness classes, so it is empty.
    pub fn palette(self) -> Vec<&'static str> {
        match self {
            PaintTarget::Biome => CART_BIOMES[..13].to_vec(),
            PaintTarget::Terrain => CART_TERRAINS.to_vec(),
            PaintTarget::Splat => SPLAT_PAINT_SLOTS.to_vec(),
            PaintTarget::RockTop | PaintTarget::RockSub => cartalith_terrain::geology::ROCK_PROPS.iter().map(|r| r.name).collect(),
            PaintTarget::RockCap => Vec::new(),
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
            PaintTarget::RockTop => "rock_top_painted",
            PaintTarget::RockSub => "rock_sub_painted",
            PaintTarget::RockCap => "rock_cap_painted",
        }
    }
}

/// GF-8: the three Rock drafts, committed into whole-world byte grids for
/// [`cartalith_engine::rock_paint::commit`] to bake
/// ([`PaintEditor::take_rock_commit`]). `0` is "not painted" on every layer.
pub struct RockCommit {
    /// Surface rock, `rock index + 1`.
    pub top: Vec<u8>,
    /// Rock beneath, `rock index + 1` or `SUB_NONE`.
    pub sub: Vec<u8>,
    /// Cap thickness class `k` (`10·k` m).
    pub cap: Vec<u8>,
    /// One summary per Rock layer, `[top, sub, cap]`; `tiles_marked` are the
    /// painted tiles (the shared [`DirtyTracker`]'s tile ids).
    pub summaries: [CommitSummary; 3],
}

/// The brush parameters `DCC_SHELL_SPEC.md` §4.5.2's `PAINT · BIOME` tool
/// options row exposes, after clamping. `hardness`/`softness` feather the disc's
/// own edge since `DECISIONS.md` §7k (this module's own doc); they never blend
/// the categorical *value* a touched cell receives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    /// 1-based index into the *active* layer's own `PaintTarget::palette`.
    /// Meaningless while `erase` is set (a dab always writes `0` then), but
    /// kept rather than cleared, so turning `erase` back off restores the
    /// class the user had selected.
    pub value: u8,
    /// Dab radius in grid cells, clamped to [`PAINT_RADIUS_RANGE`].
    pub radius: f64,
    /// Edge hardness in `[0, 1]`; `1.0` is the hard disc (`DECISIONS.md` §7k).
    pub hardness: f64,
    /// Edge softness in `[0, 1]`; `0.0` is a zero-width feather band (`DECISIONS.md` §7k).
    pub softness: f64,
    /// When set, every dab writes `0` (unpainted) instead of [`Brush::value`].
    pub erase: bool,
    /// When set, a dab skips cells the water mask marks as water (this port's toggle; see the module doc).
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

/// The pressure-to-size floor a stroke uses when the caller does not name one:
/// size runs from 20 % of the brush radius at zero pressure to 100 % at full.
/// **Labelled judgement** (ET-D2): no reference value exists; 20 % keeps a
/// feather-light touch visible on a grid where a 1-cell radius is the minimum.
pub const DEFAULT_PRESSURE_MIN: f64 = 0.2;

/// Everything a stroke session needs beyond the brush: the ET-D2 options the
/// shell passes to `paint_stroke_begin` as one Dictionary. `godot`-free so the
/// session is testable without a runtime; `lib.rs` only converts the Dictionary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeOptions {
    /// The stroke RNG seed (scatter, jitter); the same seed and samples give the same dabs.
    pub seed: u64,
    /// Spacing as a fraction of the dab diameter, or `None` for **spacing off**: one
    /// dab per sample, which is exactly the [`PaintEditor::stroke_at`] picture.
    /// `Some` is the Paint Toolbar artboard's `0.18` by default ([`StrokeOptions::default`]).
    pub spacing: Option<f64>,
    /// Pulled-string length in cells; `0.0` is off (the stroke engine's own default).
    pub smoothing: f64,
    /// Symmetry mode (`Symmetry::None` is off).
    pub symmetry: Symmetry,
    /// Symmetry centre in cells, or `None` for the grid centre.
    pub symmetry_centre: Option<(f64, f64)>,
    /// Mirror-axis rotation in radians (mirror modes only).
    pub symmetry_angle: f64,
    /// Scatter settings; `Scatter::OFF` is one copy, no offset.
    pub scatter: Scatter,
    /// The edge-band shape for this stroke's dabs (only visible with a soft brush).
    pub edge_shape: EdgeShape,
    /// Whether pressure drives dab size. Pressure never drives flow here: paint is
    /// categorical (`DECISIONS.md` section 7k), so flow has no meaning.
    pub pressure_size: bool,
    /// The size multiplier at zero pressure, in `[0, 1]` ([`DEFAULT_PRESSURE_MIN`]).
    pub pressure_min: f64,
}

impl Default for StrokeOptions {
    /// Spacing on at the artboard's `0.18`, everything else off, pressure-size on.
    fn default() -> Self {
        Self {
            seed: 0,
            spacing: Some(cartalith_spatial::stroke::DEFAULT_SPACING),
            smoothing: 0.0,
            symmetry: Symmetry::None,
            symmetry_centre: None,
            symmetry_angle: 0.0,
            scatter: Scatter::OFF,
            edge_shape: EdgeShape::Linear,
            pressure_size: true,
            pressure_min: DEFAULT_PRESSURE_MIN,
        }
    }
}

/// One open stroke: the engine plus a **snapshot** of the brush taken at
/// [`PaintEditor::stroke_begin`], so a mid-stroke slider move cannot change the
/// dabs of a stroke already under the pointer, and so every dab of the stroke
/// lands in one layer's draft whatever `set_layer` does meanwhile.
struct StrokeSession {
    engine: StrokeEngine,
    layer: PaintTarget,
    value: u8,
    radius: f64,
    land_only: bool,
    hardness: f64,
    softness: f64,
    shape: EdgeShape,
}

/// The live Paint-editor state for one generated world: which layer is
/// active, the live brush parameters, this world's own cached land-only
/// gate mask, and the three (draft, committed-layer) pairs — one each for
/// Biome/Terrain/Splat, sharing one [`DirtyTracker`] (see this module's own
/// doc on why three drafts rather than one).
pub struct PaintEditor {
    /// The layer the next [`PaintEditor::stroke_at`] writes to; change it only through [`PaintEditor::set_layer`].
    pub layer: PaintTarget,
    /// The live brush; change it only through [`PaintEditor::set_brush`], which clamps.
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
    /// GF-8's three Rock drafts ([`PaintTarget::ROCK`]). Drafts only: a
    /// commit takes them out through [`Self::take_rock_commit`] and the
    /// engine bakes them into the geology column, so they have no committed
    /// [`PaintLayer`] of their own.
    draft_rock_top: PassBuffer<PaintStamp>,
    draft_rock_sub: PassBuffer<PaintStamp>,
    draft_rock_cap: PassBuffer<PaintStamp>,
    /// The always-unallocated layer [`Self::active_layer`] answers for a Rock
    /// target: committed rock lives in the column, never here. Never written.
    no_layer: PaintLayer,
    /// How many times a **committed** layer has changed — see [`Self::epoch`].
    epoch: u64,
    /// The open ET-D2 stroke, if any ([`PaintEditor::stroke_begin`] to [`PaintEditor::stroke_end`]).
    stroke: Option<StrokeSession>,
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
            draft_rock_top: PassBuffer::new(gw, gh, PAINT_TILE_SIZE),
            draft_rock_sub: PassBuffer::new(gw, gh, PAINT_TILE_SIZE),
            draft_rock_cap: PassBuffer::new(gw, gh, PAINT_TILE_SIZE),
            no_layer: PaintLayer::new(),
            epoch: 0,
            stroke: None,
        }
    }

    /// How many times a **committed** layer has changed, for a consumer that
    /// caches something derived from [`Self::layer_cells`].
    ///
    /// Added for `WorldGen::lod_cache_key` (`LOD_DETAIL_SCOPE.md` LOD-D2),
    /// which needs "have the paint overrides moved" per LOD tile and cannot
    /// afford the O(gw*gh) scan `painted_counts` is.
    ///
    /// **Derived from the definition, not from a guess about usage:** the
    /// three committed layers are private to this struct and exactly three
    /// methods write them — `commit_all` and `clear_cells_under` (both
    /// through `PaintLayer::cells_mut`, the only mutable accessor) and
    /// `restore_layers` (which replaces all three outright). All three bump
    /// this (`clear_cells_under` only when a cell actually changed). `discard_all`, `set_layer`,
    /// `set_brush` and `stroke_at` touch only drafts, which no consumer of
    /// `layer_cells` can see, so they deliberately do not.
    /// `grep -n 'cells_mut\|self\.biome = \|self\.terrain = \|self\.splat = ' paint_bridge.rs`
    /// is how that list was produced and is how to re-check it.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// The *committed* [`PaintLayer`] of the currently selected layer. Committed
    /// state only: the layer's pending dabs are in [`PaintEditor::active_draft`],
    /// and neither accessor ever shows the other's content.
    ///
    /// A Rock target answers the always-unallocated `no_layer`: committed rock
    /// is in the geology column, which this editor never holds.
    pub fn active_layer(&self) -> &PaintLayer {
        match self.layer {
            PaintTarget::Biome => &self.biome,
            PaintTarget::Terrain => &self.terrain,
            PaintTarget::Splat => &self.splat,
            PaintTarget::RockTop | PaintTarget::RockSub | PaintTarget::RockCap => &self.no_layer,
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
    ///
    /// `None` for every Rock target, always: there is no committed Rock layer
    /// (see [`PaintTarget::ROCK`]).
    pub fn layer_cells(&self, target: PaintTarget) -> Option<&[u8]> {
        match target {
            PaintTarget::Biome => self.biome.cells(),
            PaintTarget::Terrain => self.terrain.cells(),
            PaintTarget::Splat => self.splat.cells(),
            PaintTarget::RockTop | PaintTarget::RockSub | PaintTarget::RockCap => None,
        }
    }

    /// The pending (uncommitted) draft of the currently selected layer, read-only.
    /// Pairs with [`PaintEditor::active_layer`]; the live picture is the composite
    /// of the two (`PassBuffer::preview_into`), never either alone.
    pub fn active_draft(&self) -> &PassBuffer<PaintStamp> {
        match self.layer {
            PaintTarget::Biome => &self.draft_biome,
            PaintTarget::Terrain => &self.draft_terrain,
            PaintTarget::Splat => &self.draft_splat,
            PaintTarget::RockTop => &self.draft_rock_top,
            PaintTarget::RockSub => &self.draft_rock_sub,
            PaintTarget::RockCap => &self.draft_rock_cap,
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
    ///
    /// GF-8: the Rock drafts count too, because the one Commit / Discard pair
    /// acts on them as well (`WorldGen::paint_commit`'s Rock branch,
    /// [`Self::discard_all`]).
    pub fn pending_stamps(&self) -> usize {
        self.draft_biome.len() + self.draft_terrain.len() + self.draft_splat.len() + self.rock_pending()
    }

    /// GF-8: pending dabs across the three Rock drafts only — what the next
    /// commit hands the engine's rock bake.
    pub fn rock_pending(&self) -> usize {
        self.draft_rock_top.len() + self.draft_rock_sub.len() + self.draft_rock_cap.len()
    }

    /// Mutable twin of [`PaintEditor::active_draft`] and the one place
    /// [`PaintEditor::stroke_at`] chooses which draft a dab lands in. Its `match`
    /// must stay in step with `active_draft`'s, or previews would read a different
    /// layer's draft than the one the brush is writing.
    fn active_draft_mut(&mut self) -> &mut PassBuffer<PaintStamp> {
        let layer = self.layer;
        self.draft_of_mut(layer)
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
        self.abort_stroke();
        self.biome = PaintLayer::decode_sparse(biome, n);
        self.terrain = PaintLayer::decode_sparse(terrain, n);
        self.splat = PaintLayer::decode_sparse(splat, n);
        self.draft_biome.discard();
        self.draft_terrain.discard();
        self.draft_splat.discard();
        // GF-8: a restore replaces the world's state, so a pending Rock dab
        // would bake into a column it was never painted over. (The Rock
        // drafts themselves are not saved yet: the geology save set is
        // GF-8's third slice.)
        self.draft_rock_top.discard();
        self.draft_rock_sub.discard();
        self.draft_rock_cap.discard();
        self.epoch = self.epoch.wrapping_add(1);
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
    ///
    /// The value check reads [`PaintTarget::legal_values`], so `RockSub`'s
    /// 255 ("none") and `RockCap`'s `1 ..= 255` survive a switch and anything
    /// else illegal for the new layer resets to `1`. Switching to a Rock layer
    /// also drops `erase`: Erase is disabled for Rock (§4.13; see
    /// [`Self::set_brush`]).
    ///
    /// **Whether Rock may be selected at all is the caller's gate**
    /// (`WorldGen::paint_set_layer` refuses it when the world has no column
    /// or erosion does not read rock); this editor has no world to ask.
    pub fn set_layer(&mut self, target: PaintTarget) {
        self.layer = target;
        if !target.legal_values().contains(&self.brush.value) {
            self.brush.value = 1;
        }
        if target.is_rock() {
            self.brush.erase = false;
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
    ///
    /// GF-8: on a Rock layer `value` is mapped by [`PaintTarget::legal_value`]
    /// (so `RockSub` keeps 255 = "none" and `RockCap` takes any class
    /// `1 ..= 255`), and **`erase` is always stored `false`**: §4.13 disables
    /// Erase (⇧) for Rock because "a committed rock is reverted with Undo; the
    /// column has no unpainted state to fall back to". The shell's momentary
    /// Shift-erase reaches here too, so it is refused at the one place both
    /// the latch and the modifier pass through.
    pub fn set_brush(&mut self, value: i64, radius: f64, hardness: f64, softness: f64, erase: bool, land_only: bool) -> Brush {
        let value = self.layer.legal_value(value);
        let erase = erase && !self.layer.is_rock();
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

    // ---- ET-D2: a stroke session on the ET-D1 engine ----

    /// The draft of a **named** layer (the stroke session's, which may differ
    /// from the active one if `set_layer` is called mid-stroke).
    fn draft_of_mut(&mut self, target: PaintTarget) -> &mut PassBuffer<PaintStamp> {
        match target {
            PaintTarget::Biome => &mut self.draft_biome,
            PaintTarget::Terrain => &mut self.draft_terrain,
            PaintTarget::Splat => &mut self.draft_splat,
            PaintTarget::RockTop => &mut self.draft_rock_top,
            PaintTarget::RockSub => &mut self.draft_rock_sub,
            PaintTarget::RockCap => &mut self.draft_rock_cap,
        }
    }

    /// Whether a stroke session is open.
    pub fn stroke_active(&self) -> bool {
        self.stroke.is_some()
    }

    /// Starts a stroke on the active layer with the current brush (ET-D2,
    /// `EDITING_TOOLS_SCOPE.md`). The brush is **snapshotted** here.
    ///
    /// Everything until [`PaintEditor::stroke_end`] is **one undo step** (scope
    /// question OQ-4, default yes): the layer's draft opens a
    /// [`PassBuffer::begin_group`]. A stroke already open is ended first, so a
    /// lost pointer-up cannot leave two strokes merged into one step.
    ///
    /// The engine's base diameter is `2 x brush.radius`, so `spacing` is a
    /// fraction of the brush diameter. Pressure drives **size only**: flow has no
    /// meaning for a categorical layer (`DECISIONS.md` section 7k), so the engine's
    /// flow dynamic is left off and `Dab::flow_k` is never read.
    ///
    /// Errors are the engine's own [`StrokeError`] naming the offending option
    /// (never a clamp: a clamped nonsense value is a brush nobody asked for);
    /// on error nothing changes, including any stroke already open.
    pub fn stroke_begin(&mut self, o: StrokeOptions) -> Result<(), StrokeError> {
        let mut cfg = StrokeConfig::new(o.seed, 2.0 * self.brush.radius);
        match o.spacing {
            Some(s) => cfg.spacing = s,
            None => cfg.per_sample = true,
        }
        cfg.smoothing = o.smoothing;
        cfg.symmetry = o.symmetry;
        let (gw, gh) = (self.draft_biome.width() as f64, self.draft_biome.height() as f64);
        let (cx, cy) = o.symmetry_centre.unwrap_or((gw / 2.0, gh / 2.0));
        cfg.symmetry_cx = cx;
        cfg.symmetry_cy = cy;
        cfg.symmetry_angle = o.symmetry_angle;
        cfg.scatter = o.scatter;
        if o.pressure_size {
            cfg.size = Dynamic { sensor: Sensor::Pressure, min: o.pressure_min, jitter: 0.0 };
        }
        let engine = StrokeEngine::new(cfg)?;
        self.abort_stroke();
        let layer = self.layer;
        self.draft_of_mut(layer).begin_group();
        self.stroke = Some(StrokeSession {
            engine,
            layer,
            value: if self.brush.erase { 0 } else { self.brush.value },
            radius: self.brush.radius,
            land_only: self.brush.land_only,
            hardness: self.brush.hardness,
            softness: self.brush.softness,
            shape: o.edge_shape,
        });
        Ok(())
    }

    /// Feeds one pointer sample to the open stroke and pushes one
    /// [`PaintStamp`] per dab it emits, returning how many it pushed. `0` with
    /// no stroke open (never a panic: the caller is GDScript).
    ///
    /// Dab radius is `brush.radius x size_k`, clamped to
    /// [`PAINT_RADIUS_RANGE`]. With pressure absent `size_k` is exactly `1.0`
    /// (the engine's control flow, not a default), so the radius is the brush's
    /// own, bit for bit. The sample's `x`/`y` round to a cell exactly as
    /// [`PaintEditor::stroke_at`] does ([`js_round`]).
    pub fn stroke_sample(&mut self, s: Sample) -> usize {
        let Some(mut sess) = self.stroke.take() else { return 0 };
        let mut dabs = Vec::new();
        sess.engine.feed_one(s, &mut dabs);
        let n = self.push_dabs(&sess, &dabs);
        self.stroke = Some(sess);
        n
    }

    /// Ends the open stroke: flushes the smoothing tail
    /// ([`StrokeEngine::finish`]) so the paint reaches the last pointer
    /// position, pushes those dabs, and closes the undo group. Returns the
    /// number of dabs the flush pushed; `0` with no stroke open.
    pub fn stroke_end(&mut self) -> usize {
        let Some(mut sess) = self.stroke.take() else { return 0 };
        let mut dabs = Vec::new();
        sess.engine.finish(&mut dabs);
        let n = self.push_dabs(&sess, &dabs);
        self.draft_of_mut(sess.layer).end_group();
        n
    }

    /// Drops an open stroke without flushing it, closing its undo group; what
    /// was already pushed stays in the draft. Used by every operation that
    /// replaces or empties the drafts under a stroke (commit, discard, restore),
    /// so a group can never outlive the stroke that opened it.
    fn abort_stroke(&mut self) {
        if let Some(sess) = self.stroke.take() {
            self.draft_of_mut(sess.layer).end_group();
        }
    }

    /// Turns dabs into stamps on the session's layer: the single place a dab
    /// becomes paint, shared by `stroke_sample` and `stroke_end`.
    fn push_dabs(&mut self, sess: &StrokeSession, dabs: &[Dab]) -> usize {
        for d in dabs {
            let radius = (sess.radius * d.size_k).clamp(PAINT_RADIUS_RANGE.0, PAINT_RADIUS_RANGE.1);
            let (cx, cy) = (js_round(d.x) as i64, js_round(d.y) as i64);
            let stamp = if sess.land_only {
                PaintStamp::new(cx, cy, radius, sess.value, Arc::clone(&self.water_mask))
            } else {
                PaintStamp::ungated(cx, cy, radius, sess.value)
            }
            .with_falloff(sess.hardness, sess.softness)
            .with_edge_shape(sess.shape);
            self.draft_of_mut(sess.layer).push(stamp);
        }
        dabs.len()
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
    ///
    /// GF-8: `by_class` is aligned with [`PaintTarget::legal_values`], not
    /// with the palette, so for the override layers and `RockTop` it is the
    /// same `index - 1` list as before, `RockSub` gains a twelfth entry for
    /// "none" (255) and `RockCap` has 255 (one per thickness class). A Rock
    /// layer counts its **draft only**: committed rock is in the column.
    pub fn painted_counts(&self, n: usize) -> (i64, Vec<i64>) {
        let base: Vec<u8> = self.active_layer().cells().map(<[u8]>::to_vec).unwrap_or_else(|| vec![0u8; n]);
        let mut scratch = vec![0u8; n];
        self.active_draft().preview_into(&base, &mut scratch);

        let legal = self.layer.legal_values();
        // value -> slot in `legal`; 0 and illegal values map to none.
        let mut slot = [usize::MAX; 256];
        for (i, &v) in legal.iter().enumerate() {
            slot[v as usize] = i;
        }
        let mut by_class = vec![0i64; legal.len()];
        for &v in &scratch {
            if v != 0 && slot[v as usize] != usize::MAX {
                by_class[slot[v as usize]] += 1;
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
        self.abort_stroke();
        /// Bakes one layer's draft into its committed layer and returns the summary.
        /// An empty draft returns a zeroed summary *without* calling
        /// `PaintLayer::cells_mut`, which would allocate the layer's backing array for
        /// nothing; see [`PaintEditor::commit_all`] for why an untouched layer must stay
        /// unallocated.
        fn commit_one(draft: &mut PassBuffer<PaintStamp>, layer: &mut PaintLayer, tracker: &mut DirtyTracker, n: usize, reason: &str) -> CommitSummary {
            if draft.is_empty() {
                return CommitSummary { stamps_applied: 0, stamps_skipped: 0, tiles_marked: Vec::new() };
            }
            draft.commit(layer.cells_mut(n), tracker, reason)
        }
        let biome = commit_one(&mut self.draft_biome, &mut self.biome, &mut self.tracker, n, PaintTarget::Biome.commit_reason());
        let terrain = commit_one(&mut self.draft_terrain, &mut self.terrain, &mut self.tracker, n, PaintTarget::Terrain.commit_reason());
        let splat = commit_one(&mut self.draft_splat, &mut self.splat, &mut self.tracker, n, PaintTarget::Splat.commit_reason());
        // Bumped unconditionally rather than only when a stamp landed: a
        // commit whose three drafts were all empty writes nothing, and a
        // consumer rebuilding a cache for it costs one rebuild, where a
        // missed bump costs a wrong picture. See `epoch`.
        self.epoch = self.epoch.wrapping_add(1);
        [biome, terrain, splat]
    }

    /// GF-8: commits the three Rock drafts into fresh whole-world byte grids
    /// and clears them, for `WorldGen::paint_commit`'s Rock branch to hand to
    /// `cartalith_engine::rock_paint::commit` in the same press as
    /// [`Self::commit_all`]. `None` (and nothing touched) when no Rock dab is
    /// pending.
    ///
    /// Separate from `commit_all` because the Rock half is **gated** and the
    /// override half is not: the caller takes the Rock drafts only once it has
    /// checked the world can take them (a column, erosion that reads rock, a
    /// world not finalized), so a refused Rock commit leaves the Rock drafts
    /// pending instead of dropping the user's paint. `n` must be `gw * gh`.
    ///
    /// Never moves [`Self::epoch`]: no committed override layer changes. The
    /// field and column the bake moves are keyed by the pipeline's height
    /// stage version, which the caller marks.
    pub fn take_rock_commit(&mut self, n: usize) -> Option<RockCommit> {
        if self.rock_pending() == 0 {
            return None;
        }
        self.abort_stroke();
        let mut top = vec![0u8; n];
        let mut sub = vec![0u8; n];
        let mut cap = vec![0u8; n];
        let mut one = |draft: &mut PassBuffer<PaintStamp>, buf: &mut [u8], t: PaintTarget| {
            if draft.is_empty() {
                CommitSummary { stamps_applied: 0, stamps_skipped: 0, tiles_marked: Vec::new() }
            } else {
                draft.commit(buf, &mut self.tracker, t.commit_reason())
            }
        };
        let s_top = one(&mut self.draft_rock_top, &mut top, PaintTarget::RockTop);
        let s_sub = one(&mut self.draft_rock_sub, &mut sub, PaintTarget::RockSub);
        let s_cap = one(&mut self.draft_rock_cap, &mut cap, PaintTarget::RockCap);
        Some(RockCommit { top, sub, cap, summaries: [s_top, s_sub, s_cap] })
    }

    /// Whether any **committed** layer holds an allocated array at all --
    /// the cheap gate `WorldGen::sculpt_commit` asks before it spends a
    /// footprint pass on [`Self::clear_cells_under`]. An allocated layer
    /// that was since erased back to all-zero still answers `true`; the
    /// clear then finds nothing, which costs a pass and changes nothing.
    pub fn has_committed_paint(&self) -> bool {
        !(self.biome.is_unallocated() && self.terrain.is_unallocated() && self.splat.is_unallocated())
    }

    /// Ruling AS (2026-09-24): a sculpt commit clears the painted override
    /// cells it covers. Zeroes every **committed** cell of all three layers
    /// where `footprint[i]` is set, and returns how many painted cells each
    /// layer lost, `[biome, terrain, splat]`.
    ///
    /// All three layers, Splat included: the ruling's reason is *"a sculpt
    /// changes the ground"*, and Splat is the ground-texture override -- the
    /// most literal of the three.
    ///
    /// **Pending drafts are untouched.** An uncommitted dab is the user's
    /// next edit, not ground the sculpt changed under it; it bakes over the
    /// new ground when the paint is committed, as it would have over the old.
    ///
    /// An unallocated layer stays unallocated (`commit_all`'s own rule: "an
    /// unpainted layer costs nothing"), and a layer whose length is not `n`
    /// is left alone rather than reallocated -- that is the
    /// `PaintLayer::cells_mut` mismatch path, which would silently wipe it.
    /// [`Self::epoch`] moves only when a cell actually changed.
    pub fn clear_cells_under(&mut self, n: usize, footprint: &[bool]) -> [usize; 3] {
        /// Zeroes one committed layer's painted cells wherever `footprint` is set and
        /// returns how many it cleared. Skips a layer that is unallocated or whose length
        /// is not `n` (`cells_mut` would reallocate and silently wipe a mismatched
        /// layer), and counts only cells that were actually non-zero so the caller's
        /// epoch moves only on a real change. See [`PaintEditor::clear_cells_under`].
        fn clear_one(layer: &mut PaintLayer, n: usize, footprint: &[bool]) -> usize {
            if layer.cells().is_none_or(|c| c.len() != n) {
                return 0;
            }
            let mut cleared = 0;
            for (c, &f) in layer.cells_mut(n).iter_mut().zip(footprint) {
                if f && *c != 0 {
                    *c = 0;
                    cleared += 1;
                }
            }
            cleared
        }
        let out = [
            clear_one(&mut self.biome, n, footprint),
            clear_one(&mut self.terrain, n, footprint),
            clear_one(&mut self.splat, n, footprint),
        ];
        if out.iter().any(|&k| k > 0) {
            self.epoch = self.epoch.wrapping_add(1);
        }
        out
    }

    /// Drops every layer's pending draft, touching nothing committed.
    /// Returns how many dabs were dropped in total, across all six (the three
    /// override layers and GF-8's three Rock layers: one Discard for the one
    /// Commit, §4.13).
    pub fn discard_all(&mut self) -> usize {
        self.abort_stroke();
        self.draft_biome.discard()
            + self.draft_terrain.discard()
            + self.draft_splat.discard()
            + self.draft_rock_top.discard()
            + self.draft_rock_sub.discard()
            + self.draft_rock_cap.discard()
    }

    // ---- preview rasters ----

    /// The whole-grid RGBA preview: the active layer's committed cells with
    /// its uncommitted draft composited over them, one texel per grid cell,
    /// `0` (unpainted) rendering fully transparent.
    ///
    /// `None` — and this is the same condition `WorldGen::
    /// build_paint_preview_texture` returns `None` for — means there is
    /// **nothing to draw at all**: nothing committed and nothing pending.
    /// It is not "nothing changed".
    ///
    /// The `c.len() == n` guard on the committed cells is deliberate and is
    /// this module's existing posture, not a new one: a caller whose `n`
    /// disagrees with the layer it allocated gets an unpainted base rather
    /// than the panic `PassBuffer::preview_into`'s own length assert would
    /// raise **across the gdext boundary**, the same trade
    /// [`PaintEditor::new`] documents for a mismatched `water_mask`. It
    /// cannot fire through the shipped path — `PaintLayer::cells_mut(n)`
    /// takes its length from the same `gw * gh` every caller here passes,
    /// and the editor is rebuilt per `generate()`.
    ///
    /// Biome cells take the frozen [`crate::render::CART_BIOME_COLS`]; a
    /// caller with a live `WorldGen` uses [`PaintEditor::preview_full_with`].
    pub fn preview_full(&self, gw: usize, gh: usize) -> Option<Vec<u8>> {
        self.preview_full_with(gw, gh, &crate::render::CART_BIOME_COLS)
    }

    /// [`PaintEditor::preview_full`], with Biome cells coloured from
    /// `biome_cols` — CA-19 (`LARGE_ITEM_RULINGS.md` Ruling P): the preview
    /// has to show a painted class in the colour the committed map will blend
    /// it toward, and after `WorldGen::set_biome_color` that is the override,
    /// not the constant. `WorldGen::build_paint_preview_texture` passes
    /// `self.appearance().biome_cols`. Terrain and Splat ignore it.
    pub fn preview_full_with(&self, gw: usize, gh: usize, biome_cols: &[(u8, u8, u8); 15]) -> Option<Vec<u8>> {
        let n = gw * gh;
        if self.active_layer().is_empty() && self.active_draft().is_empty() {
            return None;
        }
        let zeros;
        let base: &[u8] = match self.active_layer().cells() {
            Some(c) if c.len() == n => c,
            _ => {
                zeros = vec![0u8; n];
                &zeros
            }
        };
        let mut scratch = vec![0u8; n];
        self.active_draft().preview_into(base, &mut scratch);
        Some(pack_window(self.layer, &scratch, gw, Region::new(0, 0, gw, gh), self.layer.palette().len(), biome_cols))
    }

    /// The same raster, restricted to the rectangle the uncommitted draft
    /// touches — `OUTSTANDING_WORK.md` §2.6's bounded paint-preview upload.
    ///
    /// **The returned [`Region`] is always explicit and always non-empty.**
    /// That is the whole point of returning a [`PreviewPatch`] rather than
    /// the bare `Option<Region>` [`PassBuffer::touched_bounds`] hands back:
    /// a consumer of this function never has to decide what an absent
    /// rectangle meant, because there is no absent rectangle. The three
    /// states a caller can be in are distinguished by *shape*, not by a
    /// sentinel:
    ///
    /// | returned | means | what the caller must draw |
    /// |---|---|---|
    /// | `None` | nothing committed, nothing pending | nothing — clear the preview |
    /// | `Some`, `region == (0, 0, gw, gh)` | the draft touched nothing, so the committed layer is the whole answer | the whole grid |
    /// | `Some`, `region` a sub-rectangle | only these cells can have changed | this rectangle, at its own offset |
    ///
    /// Row two is the dangerous one and it is why `touched_bounds()`'s
    /// `None` is *not* forwarded: an empty draft over a painted layer needs
    /// the **whole grid**, which is the exact opposite of "nothing is
    /// dirty". Reading that `None` as an empty window would blank a painted
    /// map. Here it becomes a full-grid region, so the worst a caller can
    /// do by ignoring the distinction is upload more than it needed.
    ///
    /// Inside `region` the bytes are identical to [`PaintEditor::
    /// preview_full`]'s at the same offsets — same composite (`PassBuffer::
    /// preview_touched_into`, whose own doc carries the argument), same
    /// packer, asserted byte-for-byte by
    /// `tests/paint_preview_cost.rs::the_patch_is_byte_identical_to_a_full_reupload`.
    ///
    /// Biome cells take the frozen table; see [`PaintEditor::preview_patch_with`].
    pub fn preview_patch(&self, gw: usize, gh: usize) -> Option<PreviewPatch> {
        self.preview_patch_with(gw, gh, &crate::render::CART_BIOME_COLS)
    }

    /// [`PaintEditor::preview_patch`], with Biome cells coloured from
    /// `biome_cols`, for [`PaintEditor::preview_full_with`]'s reason.
    /// `WorldGen::build_paint_preview_patch` passes
    /// `self.appearance().biome_cols`.
    pub fn preview_patch_with(&self, gw: usize, gh: usize, biome_cols: &[(u8, u8, u8); 15]) -> Option<PreviewPatch> {
        let n = gw * gh;
        if self.active_layer().is_empty() && self.active_draft().is_empty() {
            return None;
        }
        let zeros;
        let base: &[u8] = match self.active_layer().cells() {
            Some(c) if c.len() == n => c,
            _ => {
                zeros = vec![0u8; n];
                &zeros
            }
        };
        // Allocated unconditionally rather than only on the bounded branch:
        // `vec![0u8; n]` is a calloc whose pages are never faulted in
        // outside the window (0.012-0.013 ms at every size the bench
        // covers, 512² through 4096²), so branching around it would buy
        // nothing and cost a second code path.
        let mut scratch = vec![0u8; n];
        let (region, src) = match self.active_draft().preview_touched_into(base, &mut scratch) {
            Some(win) => (win, &scratch[..]),
            None => (Region::new(0, 0, gw, gh), base),
        };
        let rgba = pack_window(self.layer, src, gw, region, self.layer.palette().len(), biome_cols);
        Some(PreviewPatch { region, rgba })
    }
}

/// One paint-preview raster and the rectangle of the grid it covers.
///
/// `rgba.len()` is always exactly `region.w * region.h * 4` — there is no
/// state of this struct in which the caller has to guess a stride, and no
/// state in which `region` is absent. See [`PaintEditor::preview_patch`]
/// for what a full-grid `region` means as against a sub-rectangle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewPatch {
    /// The grid rectangle `rgba` covers; always non-empty.
    pub region: Region,
    /// RGBA8 texels for `region`, row-major; length is exactly `region.w * region.h * 4`.
    pub rgba: Vec<u8>,
}

/// Packs `win` of a `stride`-wide composited override grid into RGBA8,
/// row-major from `win`'s own top-left. Unpainted (`0`) cells render
/// `(0, 0, 0, 0)`; every other index takes [`swatch_color_with`] over
/// `biome_cols`.
///
/// The single packer both preview paths run, so "the patch matches a full
/// re-upload" is a property of one loop over two windows rather than of two
/// loops that have to be kept in step.
fn pack_window(target: PaintTarget, cells: &[u8], stride: usize, win: Region, palette_len: usize, biome_cols: &[(u8, u8, u8); 15]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(win.w * win.h * 4);
    for y in win.y..win.y + win.h {
        let row = y * stride + win.x;
        for &v in &cells[row..row + win.w] {
            if v == 0 {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let (r, g, b) = swatch_color_with(target, v, palette_len, biome_cols);
                bytes.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }
    bytes
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
///
/// **No shipped caller since CA-19's preview routing (2026-09-24)**: the
/// preview packer reads [`swatch_color_with`] over the live table. This
/// frozen-table form stays for the tests that pin the reference's own
/// colours (`tests/paint_preview_cost.rs`, this file's `tests`).
#[cfg_attr(not(test), allow(dead_code))]
pub fn swatch_color(target: PaintTarget, index: u8, palette_len: usize) -> (u8, u8, u8) {
    swatch_color_with(target, index, palette_len, &crate::render::CART_BIOME_COLS)
}

/// [`swatch_color`], but the Biome swatch is read from `biome_cols` rather
/// than the frozen [`crate::render::CART_BIOME_COLS`] constant — CA-19
/// (`LARGE_ITEM_RULINGS.md`, Ruling P, 2026-09-21): the biome colour table
/// is user-editable, and this is the swatch picker's own path onto a live
/// `WorldGen`'s overrides, exactly as `render::land_color`'s paint blend
/// reads `TerrainAppearance::biome_cols` (`render.rs`) rather than the
/// constant directly. `swatch_color(t, i, n) ==
/// swatch_color_with(t, i, n, &CART_BIOME_COLS)` always — every existing
/// caller and golden that pins `swatch_color` against the frozen table
/// keeps doing so unchanged; this function exists only for a caller that
/// actually has an override table to pass.
///
/// Terrain and Splat are unaffected — CA-19 is scoped to the biome table
/// alone, matching `render.rs`'s own `TerrainAppearance::biome_cols`.
///
/// **Rock (GF-8) has no reference colour either**, so it takes the same
/// generated-hue rule as Splat, over the eleven rocks; `RockSub`'s "none"
/// (255) is a neutral grey `(150, 150, 150)`; `RockCap` is a ramp from light
/// sand `(235, 215, 170)` at 10 m to dark brown `(90, 60, 30)` at 2 550 m,
/// linear in the class. All three are **labelled judgements** (legible over
/// the map, ordered for the cap), not measurements: this is the in-flight
/// draft's overlay only, and a committed rock is drawn by the renderer's own
/// lithology colours.
pub fn swatch_color_with(target: PaintTarget, index: u8, palette_len: usize, biome_cols: &[(u8, u8, u8); 15]) -> (u8, u8, u8) {
    if index != 0 {
        match target {
            PaintTarget::RockSub if index == cartalith_engine::rock_paint::SUB_NONE => return (150, 150, 150),
            PaintTarget::RockCap => {
                let t = (index - 1) as f64 / 254.0;
                let lerp = |a: f64, b: f64| (a + (b - a) * t).round() as u8;
                return (lerp(235.0, 90.0), lerp(215.0, 60.0), lerp(170.0, 30.0));
            }
            _ => {}
        }
    }
    if index == 0 || palette_len == 0 || index as usize > palette_len {
        return (0, 0, 0);
    }
    let i = index as usize - 1;
    match target {
        PaintTarget::Biome => biome_cols[i],
        PaintTarget::Terrain => crate::render::CART_TERRAIN_COLS[i],
        // Equal hue steps around the wheel so neighbouring classes differ. Saturation 0.65 and
        // value 0.95 are a labelled judgement (legible over the map), not a measurement: Splat
        // has no reference colour (see this function's doc), so there is nothing to match.
        PaintTarget::Splat | PaintTarget::RockTop | PaintTarget::RockSub => {
            hsv_to_rgb(((index - 1) as f64 / palette_len as f64) * 360.0, 0.65, 0.95)
        }
        // Reached only for index 0, which returned above; kept total.
        PaintTarget::RockCap => (0, 0, 0),
    }
}

/// Standard HSV to RGB conversion: `h` in degrees, `s` and `v` in `[0, 1]`, the
/// result rounded to bytes. Used only for Splat's generated swatch hue (see
/// [`swatch_color_with`]); the sextant `match` falls through to the last arm for
/// sextant 5, and callers pass `h < 360`. Not a parity function: no reference
/// value exists for it.
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

/// The unit tests of the paint bridge. They run without a Godot runtime (this module
/// is `godot`-free), so they exercise the state machine, not the `#[func]` surface.
#[cfg(test)]
mod tests {
    use super::*;
    // Only the falloff tests below call `PaintStamp::apply` directly, to
    // build a same-shape comparison stamp -- everything else in this module
    // reaches it indirectly through `PassBuffer`.
    use cartalith_spatial::Stamp;

    /// An all-land water mask of `n` cells (`0` = land everywhere), so the land-only
    /// gate excludes nothing unless a test marks a cell itself.
    fn land_mask(n: usize) -> Arc<[u8]> {
        vec![0u8; n].into()
    }

    // ---- registry ----

    /// Protects: `PaintTarget::key` and `from_key` are inverses for all three layers,
    /// and an unknown key resolves to `None` rather than a default layer.
    #[test]
    fn every_target_round_trips_through_its_own_key() {
        for t in PaintTarget::ALL {
            assert_eq!(PaintTarget::from_key(t.key()), Some(t));
        }
        assert_eq!(PaintTarget::from_key("nope"), None);
    }

    /// Protects: the Biome palette stays the 13 land biomes: `Lake` and `Ocean / Deep Water`
    /// are never paintable, and the 13 are the first 13 `CART_BIOMES` entries.
    #[test]
    fn biome_palette_is_13_not_all_15_cart_biomes() {
        let p = PaintTarget::Biome.palette();
        assert_eq!(p.len(), 13);
        assert!(!p.contains(&"Lake"), "water classes must not be paintable");
        assert!(!p.contains(&"Ocean / Deep Water"));
        assert_eq!(p[0], CART_BIOMES[0]);
        assert_eq!(p[12], CART_BIOMES[12]);
    }

    /// Protects: Terrain and Splat expose their complete source arrays, with no carve-out
    /// like Biome's.
    #[test]
    fn terrain_and_splat_palettes_match_their_full_arrays() {
        assert_eq!(PaintTarget::Terrain.palette(), CART_TERRAINS.to_vec());
        assert_eq!(PaintTarget::Splat.palette(), SPLAT_PAINT_SLOTS.to_vec());
    }

    // ---- editor construction ----

    /// Protects: a fresh editor's starting state: Biome active, default radius 6, the land-only
    /// gate on (the reference's always-on behaviour), erase off, and an unallocated
    /// layer with an empty draft.
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

    /// Protects: `set_layer` changes the editor's active layer (only `layer` is asserted).
    #[test]
    fn set_layer_switches_the_active_draft_and_layer() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_layer(PaintTarget::Splat);
        assert_eq!(e.layer, PaintTarget::Splat);
    }

    /// Protects: a brush value legal for Biome (13) is reset to 1 on switching to Splat
    /// (6 slots), never left as an index Splat's palette cannot explain.
    #[test]
    fn set_layer_clamps_a_value_that_would_be_illegal_in_the_new_layer() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(13, 6.0, 1.0, 0.0, false, true); // legal for Biome's 13 slots
        e.set_layer(PaintTarget::Splat); // only 6 slots
        assert_eq!(e.brush.value, 1, "an out-of-range value must reset, not persist silently");
    }

    /// Protects: a value still legal in the new layer (4, switching Biome to Terrain) survives
    /// the switch unchanged: the clamp resets only out-of-range values.
    #[test]
    fn set_layer_leaves_an_in_range_value_untouched() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(4, 6.0, 1.0, 0.0, false, true);
        e.set_layer(PaintTarget::Terrain); // 13 slots, 4 still legal
        assert_eq!(e.brush.value, 4);
    }

    // ---- set_brush ----

    /// Protects: `set_brush` clamps value to the layer's palette size, radius to
    /// `PAINT_RADIUS_RANGE`'s maximum and hardness/softness to `[0, 1]`, and the
    /// returned `Brush` equals what the editor stored.
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

    /// Protects: a NaN radius, hardness or softness leaves the previous value in place rather
    /// than clamping, so a NaN never reaches a stamp's arithmetic.
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

    /// Protects: a dab lands in the active layer's draft only; the other two drafts stay empty.
    #[test]
    fn stroke_at_pushes_into_the_active_layers_own_draft_only() {
        let mut e = PaintEditor::new(16, 12, land_mask(16 * 12));
        e.set_brush(5, 3.0, 1.0, 0.0, false, false);
        e.stroke_at(8.0, 6.0);
        assert_eq!(e.draft_biome.len(), 1);
        assert_eq!(e.draft_terrain.len(), 0);
        assert_eq!(e.draft_splat.len(), 0);
    }

    /// Protects: fractional pointer coordinates round to the nearest grid cell ((8.3, 6.6) to
    /// cell (8, 7)) via the JS-semantics rounding.
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

    /// Protects: an erase dab writes 0 whatever `brush.value` holds, clearing a previously
    /// painted cell in the composited preview.
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

    /// Protects: with `land_only` on, a cell the captured water mask marks as water stays
    /// unpainted even when the dab's centre is on it.
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

    /// Protects: with `land_only` off, the same masked cell is painted: the toggle really
    /// bypasses the mask.
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

    /// Protects: `DECISIONS.md` §7k's bit-identity promise: the default brush (hardness 1.0,
    /// softness 0.0) paints exactly `PaintStamp`'s own hard disc.
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

    /// Protects: hardness 0.4 paints a strictly different, strictly smaller set of cells than
    /// the hard disc (a feather only drops cells, never adds them), while the interior
    /// still paints.
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

    /// Protects: the hardness/softness `set_brush` stores are what the following `stroke_at`
    /// hands to `PaintStamp::with_falloff`: the editor path equals building that stamp
    /// directly.
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

    /// Protects: `painted_counts` counts pending dabs too: two non-overlapping radius-1 pluses
    /// give a total of 10, 5 each in their own class, with one entry per Biome class (13).
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

    /// Protects: `commit_all` bakes the Biome and Terrain drafts into their own layers (not just
    /// the active one), reports Splat as untouched, and clears the drafts.
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

    /// Protects: `pending_stamps` counts all three drafts, and reaches 0 on commit while
    /// `painted_counts` keeps its total, which is why that total cannot gate the
    /// Commit / Discard pair.
    ///
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

    /// Protects: `discard_all` empties every draft and really removes the painted cells from
    /// the composite.
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

    /// Protects: a layer with no pending dabs stays unallocated (not merely empty) across a
    /// commit of another layer.
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

    /// Protects: `discard_all` returns the total dabs dropped across layers, clears those
    /// drafts, and allocates no committed layer.
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

    /// Protects: within each palette, `swatch_color` is deterministic and gives every class a
    /// distinct colour.
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

    /// Protects: Biome and Terrain swatches equal the reference colour tables, with two literal
    /// spot values so a table edit cannot pass by moving both sides together.
    ///
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

    /// Protects: `swatch_color_with` changes only the overridden Biome index, and the 3-argument
    /// `swatch_color` stays equal to the frozen table whatever override exists.
    ///
    /// CA-19 (`LARGE_ITEM_RULINGS.md`, Ruling P, 2026-09-21): the golden
    /// re-baseline this ruling actually needs. `swatch_color` (the existing,
    /// pinned 3-arg entry point) must stay exactly the frozen table for
    /// every caller who does not pass an override — proven here by mutating
    /// only ONE entry of a *copy* of `CART_BIOME_COLS` and asserting the
    /// other 14 are untouched, both in the override table itself and in
    /// `swatch_color_with`'s answer for every index but the mutated one.
    /// `swatch_color` never sees the override at all, so the old test above
    /// this one is undisturbed — the "change recorded at the symbol" the
    /// ruling asks for is this test, not an edit to the pinned one.
    #[test]
    fn swatch_color_with_overrides_exactly_one_index_and_swatch_color_stays_pinned() {
        let mut overridden = crate::render::CART_BIOME_COLS;
        let edited_index = 5u8; // "Boreal Forest" — arbitrary, mid-table.
        let custom = (1, 2, 3);
        overridden[edited_index as usize - 1] = custom;

        for i in 1..=15u8 {
            let with_ov = swatch_color_with(PaintTarget::Biome, i, 15, &overridden);
            if i == edited_index {
                assert_eq!(with_ov, custom, "the overridden index must read back the override");
            } else {
                assert_eq!(
                    with_ov,
                    crate::render::CART_BIOME_COLS[i as usize - 1],
                    "index {i} has no override and must still read the default table"
                );
            }
            // The pinned entry point is untouched by any override table that
            // exists elsewhere — it always resolves against the constant.
            assert_eq!(swatch_color(PaintTarget::Biome, i, 15), crate::render::CART_BIOME_COLS[i as usize - 1]);
        }
        // `swatch_color`'s own definition, byte-for-byte: calling it is
        // exactly calling `swatch_color_with` against the un-overridden table.
        for i in 1..=15u8 {
            assert_eq!(swatch_color(PaintTarget::Biome, i, 15), swatch_color_with(PaintTarget::Biome, i, 15, &crate::render::CART_BIOME_COLS));
        }
    }

    /// Protects: both preview rasters colour a painted Biome cell from the table passed in, while
    /// the frozen entry point keeps `CART_BIOME_COLS`. The dab's centre cell is the
    /// probe point, chosen by where the stroke was placed, not by its colour.
    ///
    /// CA-19 (Ruling P): both preview rasters colour a painted Biome cell from
    /// the table they are handed, so the overlay names a class in the colour
    /// the committed map will blend toward. The dab's centre cell is the
    /// probe point, chosen by where the stroke was placed, not by its colour.
    #[test]
    fn biome_previews_take_the_table_they_are_handed() {
        let (gw, gh) = (16usize, 12usize);
        let mut e = PaintEditor::new(gw, gh, land_mask(gw * gh));
        e.set_brush(6, 2.0, 1.0, 0.0, false, false);
        e.stroke_at(8.0, 6.0);
        let mut edited = crate::render::CART_BIOME_COLS;
        edited[5] = (1, 2, 3);
        let at = (6 * gw + 8) * 4;
        let full = e.preview_full_with(gw, gh, &edited).expect("a pending dab draws");
        assert_eq!(&full[at..at + 4], &[1, 2, 3, 255], "full preview takes the override");
        let frozen = e.preview_full(gw, gh).unwrap();
        assert_eq!(&frozen[at..at + 4], &[42, 106, 58, 255], "the frozen entry point keeps CART_BIOME_COLS[5]");
        let patch = e.preview_patch_with(gw, gh, &edited).expect("a pending dab draws");
        let (rx, ry) = (8 - patch.region.x, 6 - patch.region.y);
        let pat = (ry * patch.region.w + rx) * 4;
        assert_eq!(&patch.rgba[pat..pat + 4], &[1, 2, 3, 255], "patch preview takes the override");
    }

    /// Protects: Splat has no reference colour (it names pack textures), so it keeps the generated
    /// hue, deterministic and different from the Biome table's first colour.
    #[test]
    fn splat_swatches_stay_generated_because_the_reference_has_no_colour_for_them() {
        assert_ne!(swatch_color(PaintTarget::Splat, 1, 6), crate::render::CART_BIOME_COLS[0]);
        assert_eq!(swatch_color(PaintTarget::Splat, 1, 6), swatch_color(PaintTarget::Splat, 1, 6));
    }

    /// Protects: index 0, an out-of-range index and an empty palette all give black for every layer.
    #[test]
    fn swatch_color_is_black_for_unpainted_or_out_of_range() {
        for t in PaintTarget::ALL {
            assert_eq!(swatch_color(t, 0, 13), (0, 0, 0));
            assert_eq!(swatch_color(t, 99, 13), (0, 0, 0));
            assert_eq!(swatch_color(t, 1, 0), (0, 0, 0));
        }
    }

    /// Protects: `layer_cells` is `None` while a layer is unallocated or has only a pending
    /// draft, and shows painted cells only after a commit; other layers stay `None`.
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

    // ---- ET-D2: the stroke session ----

    /// A pressure-free, tilt-free sample at `(x, y)`, time = `i` ms.
    fn smp(x: f64, y: f64, i: u64) -> Sample {
        Sample { x, y, pressure: None, tilt: None, t_ms: i }
    }

    /// An editor over a `w x h` all-land world with an ungated brush of `radius`.
    fn editor(w: usize, h: usize, radius: f64) -> PaintEditor {
        let mut e = PaintEditor::new(w, h, land_mask(w * h));
        e.set_brush(5, radius, 1.0, 0.0, false, false);
        e
    }

    /// 4-connected component count of the non-zero cells of a `w`-wide grid: the
    /// independent oracle for "one connected stroke".
    fn components(cells: &[u8], w: usize) -> usize {
        let mut seen = vec![false; cells.len()];
        let mut n = 0;
        for start in 0..cells.len() {
            if cells[start] == 0 || seen[start] {
                continue;
            }
            n += 1;
            let mut stack = vec![start];
            seen[start] = true;
            while let Some(i) = stack.pop() {
                let (x, y) = (i % w, i / w);
                let mut nb = Vec::new();
                if x > 0 { nb.push(i - 1); }
                if x + 1 < w { nb.push(i + 1); }
                if y > 0 { nb.push(i - w); }
                if i + w < cells.len() { nb.push(i + w); }
                for j in nb {
                    if cells[j] != 0 && !seen[j] {
                        seen[j] = true;
                        stack.push(j);
                    }
                }
            }
        }
        n
    }

    /// The committed biome cells of `e` after baking its draft.
    fn committed(e: &mut PaintEditor, n: usize) -> Vec<u8> {
        e.commit_all(n);
        e.layer_cells(PaintTarget::Biome).expect("something was painted").to_vec()
    }

    /// Spacing-off, pressure-absent options.
    fn off() -> StrokeOptions {
        StrokeOptions { spacing: None, ..StrokeOptions::default() }
    }

    /// Protects: ET-D2's byte-equality acceptance. With spacing off and pressure
    /// absent a scripted drag through the stroke session commits exactly the
    /// layer the one-dab-per-sample `stroke_at` path commits, with a soft brush
    /// so the edge band (and its hash) is exercised too, and over a non-trivial
    /// drag that revisits and repeats positions. The painted count is asserted
    /// non-zero so a silently-empty pair cannot pass.
    #[test]
    fn spacing_off_stroke_is_byte_equal_to_the_one_dab_path() {
        let (w, h) = (96, 64);
        let drag = [(10.3, 12.0), (14.9, 15.5), (14.9, 15.5), (30.0, 20.2), (28.0, 40.0), (60.7, 33.3), (61.0, 10.0)];
        let mut a = editor(w, h, 5.0);
        a.set_brush(5, 5.0, 0.6, 0.1, false, false);
        for &(x, y) in &drag {
            a.stroke_at(x, y);
        }
        let mut b = editor(w, h, 5.0);
        b.set_brush(5, 5.0, 0.6, 0.1, false, false);
        b.stroke_begin(off()).unwrap();
        for (i, &(x, y)) in drag.iter().enumerate() {
            assert_eq!(b.stroke_sample(smp(x, y, i as u64)), 1, "one dab per sample, repeats included");
        }
        assert_eq!(b.stroke_end(), 0, "no smoothing, so nothing to flush");
        assert_eq!(a.pending_stamps(), b.pending_stamps());
        let (ca, cb) = (committed(&mut a, w * h), committed(&mut b, w * h));
        assert!(ca.iter().filter(|&&c| c != 0).count() > 200, "premise: the drag painted something");
        assert_eq!(ca, cb);
    }

    /// Protects: ET-D2's connectivity acceptance. A fast drag (one sample per 3
    /// radii) leaves a dotted line through `stroke_at` (11 separate discs, the
    /// pre-change contrast, asserted) and one connected stroke at the artboard's
    /// 18 % spacing. The literal 11 is the geometry: discs of radius 3, 9 cells apart.
    #[test]
    fn eighteen_percent_spacing_joins_a_fast_drag_the_old_path_leaves_dotted() {
        let (w, h) = (128, 32);
        let xs: Vec<f64> = (0..11).map(|k| 10.0 + 9.0 * k as f64).collect();
        let mut old = editor(w, h, 3.0);
        for &x in &xs {
            old.stroke_at(x, 16.0);
        }
        let old_cells = committed(&mut old, w * h);
        assert_eq!(components(&old_cells, w), 11, "today: one disc per sample, nothing between");

        let mut new = editor(w, h, 3.0);
        new.stroke_begin(StrokeOptions::default()).unwrap();
        for (i, &x) in xs.iter().enumerate() {
            new.stroke_sample(smp(x, 16.0, i as u64));
        }
        new.stroke_end();
        let cells = committed(&mut new, w * h);
        assert_eq!(components(&cells, w), 1, "18 % spacing: one connected stroke");
    }

    /// Protects: ET-D2's one-undo-step acceptance (scope OQ-4's default). A
    /// stroke of well over 10 dabs empties the draft with ONE undo; the same dabs
    /// through `stroke_at` need one undo each (the contrast); two strokes are two
    /// steps; and a stroke that emits nothing leaves no step.
    #[test]
    fn a_whole_stroke_is_one_undo_step() {
        let mut e = editor(128, 32, 3.0);
        e.stroke_begin(StrokeOptions::default()).unwrap();
        for (i, x) in (0..6).map(|k| 10.0 + 9.0 * k as f64).enumerate() {
            e.stroke_sample(smp(x, 16.0, i as u64));
        }
        e.stroke_end();
        assert!(e.pending_stamps() > 10, "premise: more than 10 dabs, got {}", e.pending_stamps());
        let stroke_dabs = e.pending_stamps();
        e.stroke_at(100.0, 5.0);
        e.stroke_at(110.0, 5.0);
        assert!(e.draft_biome.undo() && e.draft_biome.undo());
        assert_eq!(e.pending_stamps(), stroke_dabs, "after stroke_end, each stroke_at is its own step again");
        assert!(e.draft_biome.undo(), "one undo available");
        assert!(e.draft_biome.is_empty(), "and it empties the whole stroke");
        assert!(!e.draft_biome.can_undo(), "no second step");

        let mut old = editor(128, 32, 3.0);
        for k in 0..12 {
            old.stroke_at(10.0 + k as f64, 16.0);
        }
        let mut steps = 0;
        while old.draft_biome.undo() {
            steps += 1;
        }
        assert_eq!(steps, 12, "contrast: the one-dab path is one step per dab");

        let mut two = editor(128, 32, 3.0);
        for row in [8.0, 24.0] {
            two.stroke_begin(StrokeOptions::default()).unwrap();
            for (i, x) in (0..4).map(|k| 10.0 + 9.0 * k as f64).enumerate() {
                two.stroke_sample(smp(x, row, i as u64));
            }
            two.stroke_end();
        }
        assert!(two.draft_biome.undo() && !two.draft_biome.is_empty(), "first undo leaves stroke one");
        assert!(two.draft_biome.undo() && two.draft_biome.is_empty(), "second undo removes stroke one");
        assert!(!two.draft_biome.can_undo());

        let mut none = editor(128, 32, 3.0);
        none.stroke_begin(StrokeOptions::default()).unwrap();
        none.stroke_end();
        assert!(!none.draft_biome.can_undo(), "an empty stroke records no undo step");
    }

    /// Protects: an absent pressure giving exactly the brush radius (never a
    /// default pressure), a present pressure driving size through the documented
    /// law `r x (min + (1 - min) x p)`, zero pressure being a real reading
    /// distinct from absent, and the radius clamp's lower bound. Literals are
    /// computed by hand: 10 x (0.2 + 0.8 x 0.5) = 6.0; 10 x 0.2 = 2.0.
    #[test]
    fn pressure_drives_size_and_absent_pressure_is_exactly_the_base_radius() {
        let radius_after = |p: Option<f32>, min: f64| {
            let mut e = editor(64, 64, 10.0);
            e.stroke_begin(StrokeOptions { spacing: None, pressure_min: min, ..StrokeOptions::default() }).unwrap();
            e.stroke_sample(Sample { x: 20.0, y: 20.0, pressure: p, tilt: None, t_ms: 0 });
            e.draft_biome.entries()[0].stamp.radius
        };
        assert_eq!(radius_after(None, 0.2), 10.0, "absent: the base radius, exactly");
        // Within 1e-9, not exact: the law is float arithmetic (6.000000000000001 measured).
        assert!((radius_after(Some(0.5), 0.2) - 6.0).abs() < 1e-9);
        assert!((radius_after(Some(0.0), 0.2) - 2.0).abs() < 1e-9, "zero pressure is a reading, not absence");
        assert!((radius_after(Some(1.0), 0.2) - 10.0).abs() < 1e-9);
        assert_eq!(radius_after(Some(0.0), 0.0), 1.0, "clamped up to PAINT_RADIUS_RANGE's floor");
        let mut e = editor(64, 64, 10.0);
        e.stroke_begin(StrokeOptions { pressure_size: false, spacing: None, ..StrokeOptions::default() }).unwrap();
        e.stroke_sample(Sample { x: 20.0, y: 20.0, pressure: Some(0.1), tilt: None, t_ms: 0 });
        assert_eq!(e.draft_biome.entries()[0].stamp.radius, 10.0, "pressure_size off ignores pressure");
    }

    /// Protects: `stroke_end` flushing the smoothing tail. A 6-cell string behind
    /// a pointer that ran to x = 60 leaves the last dab at or before x = 54 until
    /// `stroke_end`; after it the paint reaches within one spacing step (0.18 x
    /// diameter 8 = 1.44) of the pointer.
    #[test]
    fn stroke_end_flushes_the_smoothing_tail_to_the_pointer() {
        let mut e = editor(128, 32, 4.0);
        e.stroke_begin(StrokeOptions { smoothing: 6.0, ..StrokeOptions::default() }).unwrap();
        for (i, x) in [10.0, 25.0, 40.0, 60.0].into_iter().enumerate() {
            e.stroke_sample(smp(x, 16.0, i as u64));
        }
        let last_x = |e: &PaintEditor| e.draft_biome.entries().last().unwrap().stamp.cx;
        assert!(last_x(&e) <= 54, "before the flush the dab lags: {}", last_x(&e));
        let flushed = e.stroke_end();
        assert!(flushed > 0);
        assert!(last_x(&e) >= 60 - 2 && last_x(&e) <= 60, "after: {}", last_x(&e));
    }

    /// Protects: every refusal path returning quietly. No session: `stroke_sample`
    /// and `stroke_end` push nothing; a bad option errors and changes nothing (an
    /// open stroke survives it); `commit_all` and `discard_all` close an open
    /// stroke and its undo group, so a later `stroke_at` is its own step.
    #[test]
    fn refusals_are_quiet_and_commit_or_discard_close_the_stroke() {
        let mut e = editor(64, 64, 3.0);
        assert_eq!(e.stroke_sample(smp(5.0, 5.0, 0)), 0);
        assert_eq!(e.stroke_end(), 0);
        assert_eq!(e.pending_stamps(), 0);

        e.stroke_begin(off()).unwrap();
        e.stroke_sample(smp(5.0, 5.0, 0));
        assert_eq!(e.stroke_begin(StrokeOptions { spacing: Some(0.0), ..StrokeOptions::default() }), Err(StrokeError::Invalid("spacing")));
        assert!(e.stroke_active(), "a refused begin leaves the open stroke alone");
        assert_eq!(e.stroke_sample(smp(9.0, 5.0, 1)), 1);

        e.discard_all();
        assert!(!e.stroke_active(), "discard closes the stroke");
        e.stroke_begin(off()).unwrap();
        e.stroke_sample(smp(5.0, 5.0, 0));
        e.commit_all(64 * 64);
        assert!(!e.stroke_active(), "commit closes the stroke");
        assert_eq!(e.stroke_sample(smp(5.0, 5.0, 1)), 0);
        e.stroke_at(1.0, 1.0);
        e.stroke_at(2.0, 2.0);
        assert!(e.draft_biome.undo());
        assert_eq!(e.draft_biome.len(), 1, "a stroke_at after the closed group is its own step");
    }

    /// Protects: a second `stroke_begin` ending the first (two undo steps, not one
    /// merged group) and the options reaching the dabs: vertical symmetry about
    /// the grid centre mirrors each dab (x = 10 and 2 x 32 - 10 = 54), the edge
    /// shape lands on every stamp, and the layer is the one active at begin.
    #[test]
    fn options_reach_the_dabs_and_a_second_begin_closes_the_first_group() {
        let mut e = editor(64, 32, 3.0);
        e.set_layer(PaintTarget::Terrain);
        let sym = StrokeOptions { spacing: None, symmetry: Symmetry::Vertical, edge_shape: EdgeShape::Sharp, ..StrokeOptions::default() };
        e.stroke_begin(sym).unwrap();
        assert_eq!(e.stroke_sample(smp(10.0, 7.0, 0)), 2, "a dab and its mirror");
        e.set_layer(PaintTarget::Splat);
        e.stroke_sample(smp(12.0, 7.0, 1));
        let stamps: Vec<_> = e.draft_terrain.entries().iter().map(|p| (p.stamp.cx, p.stamp.cy, p.stamp.shape)).collect();
        assert_eq!(stamps[0], (10, 7, EdgeShape::Sharp));
        assert_eq!(stamps[1], (54, 7, EdgeShape::Sharp));
        assert_eq!(e.draft_splat.len(), 0, "the layer was fixed at begin");
        e.set_layer(PaintTarget::Terrain);
        e.stroke_begin(off()).unwrap();
        e.stroke_sample(smp(30.0, 20.0, 2));
        e.stroke_end();
        assert!(e.draft_terrain.undo());
        assert_eq!(e.draft_terrain.len(), 4, "the second stroke was its own step");
        assert!(e.draft_terrain.undo());
        assert!(e.draft_terrain.is_empty());
    }

    /// Protects: the brush's `erase` flag reaching a stroke's dabs (value `0`, the
    /// eraser) and being snapshotted at begin. A painted block is erased by a
    /// stroke begun with `erase` on, and a stroke begun with it off then painted
    /// over a layer does not erase.
    #[test]
    fn an_erase_brush_stroke_writes_zero() {
        let n = 64 * 32;
        let mut e = editor(64, 32, 4.0);
        e.stroke_at(20.0, 16.0);
        e.commit_all(n);
        assert_eq!(e.layer_cells(PaintTarget::Biome).unwrap()[16 * 64 + 20], 5, "premise: painted");
        e.set_brush(5, 4.0, 1.0, 0.0, true, false);
        e.stroke_begin(off()).unwrap();
        e.set_brush(5, 4.0, 1.0, 0.0, false, false); // a mid-stroke slider move must not matter
        e.stroke_sample(smp(20.0, 16.0, 0));
        e.stroke_end();
        e.commit_all(n);
        assert_eq!(e.layer_cells(PaintTarget::Biome).unwrap()[16 * 64 + 20], 0, "erased by the snapshotted erase flag");
    }

    /// Protects: determinism and seed sensitivity at the bridge. The same seed and
    /// samples give identical stamp lists; a different seed moves scattered dabs.
    #[test]
    fn a_stroke_is_deterministic_in_its_seed() {
        let run = |seed: u64| {
            let mut e = editor(96, 48, 3.0);
            let sc = Scatter { count: 3, count_jitter: 0.0, amount: 1.5, both_axes: true };
            e.stroke_begin(StrokeOptions { seed, scatter: sc, ..StrokeOptions::default() }).unwrap();
            for (i, x) in [10.0, 30.0, 50.0].into_iter().enumerate() {
                e.stroke_sample(smp(x, 20.0, i as u64));
            }
            e.stroke_end();
            e.draft_biome.entries().iter().map(|p| (p.stamp.cx, p.stamp.cy)).collect::<Vec<_>>()
        };
        assert_eq!(run(7), run(7));
        assert_ne!(run(7), run(8));
        assert!(run(7).len() > 30, "scatter 3 over a spaced stroke");
    }

    // ---- GF-8: the Rock layers ----

    /// Protects: the six keys resolve, and the Rock layers are NOT in `ALL`,
    /// which `get_paint_layers`, `drafts/paint.json` and the renderer read --
    /// so adding them moved neither the save document nor the old target list.
    #[test]
    fn rock_keys_resolve_and_stay_out_of_the_override_list() {
        for (t, k) in [(PaintTarget::RockTop, "rock_top"), (PaintTarget::RockSub, "rock_sub"), (PaintTarget::RockCap, "rock_cap")] {
            assert_eq!(t.key(), k);
            assert_eq!(PaintTarget::from_key(k), Some(t));
            assert!(t.is_rock());
            assert!(!PaintTarget::ALL.contains(&t));
        }
        assert_eq!(PaintTarget::ALL.len(), 3);
        assert!(!PaintTarget::Biome.is_rock());
        // The rock palette is the eleven names of ROCK_PROPS, in order.
        let pal = PaintTarget::RockTop.palette();
        assert_eq!(pal.len(), 11);
        assert_eq!(pal[0], "Granite / granitoid");
        assert_eq!(PaintTarget::RockSub.palette(), pal);
        assert!(PaintTarget::RockCap.palette().is_empty());
    }

    /// Protects: each Rock layer's legal value set, with literals: `rock_top`
    /// 1..=11, `rock_sub` 1..=11 plus 255 ("none"), `rock_cap` 1..=255; an
    /// illegal value clamps into the contiguous part, never onto 255.
    #[test]
    fn rock_values_clamp_to_their_layer() {
        assert_eq!(PaintTarget::RockTop.legal_value(12), 11);
        assert_eq!(PaintTarget::RockTop.legal_value(255), 11);
        assert_eq!(PaintTarget::RockSub.legal_value(255), 255);
        assert_eq!(PaintTarget::RockSub.legal_value(200), 11);
        assert_eq!(PaintTarget::RockCap.legal_value(255), 255);
        assert_eq!(PaintTarget::RockCap.legal_value(300), 255);
        assert_eq!(PaintTarget::RockCap.legal_value(0), 1);
        assert_eq!(PaintTarget::RockSub.legal_values().len(), 12);
        assert_eq!(*PaintTarget::RockSub.legal_values().last().unwrap(), 255);
        assert_eq!(PaintTarget::RockCap.legal_values().len(), 255);
        // The override layers are unchanged: Biome 1..=13.
        assert_eq!(PaintTarget::Biome.legal_value(255), 13);
        let mut e = PaintEditor::new(8, 8, land_mask(64));
        e.set_layer(PaintTarget::RockSub);
        assert_eq!(e.set_brush(255, 2.0, 1.0, 0.0, false, true).value, 255, "none survives set_brush");
        e.set_layer(PaintTarget::RockCap);
        assert_eq!(e.brush.value, 255, "255 is a legal cap class (2 550 m): kept across the switch");
        e.set_layer(PaintTarget::RockTop);
        assert_eq!(e.brush.value, 1, "255 is not a rock index: reset to 1");
    }

    /// Protects: §4.13's "Erase (⇧) is disabled for the Rock target" at the
    /// one place both the latch and the Shift modifier pass: `set_brush`
    /// stores `erase = false` on a Rock layer and a dab writes the value.
    #[test]
    fn erase_is_refused_on_a_rock_layer() {
        let n = 32 * 32;
        let mut e = PaintEditor::new(32, 32, land_mask(n));
        e.set_layer(PaintTarget::RockTop);
        let b = e.set_brush(10, 3.0, 1.0, 0.0, true, true);
        assert!(!b.erase, "erase stored on a Rock layer");
        e.stroke_at(16.0, 16.0);
        let rc = e.take_rock_commit(n).unwrap();
        assert_eq!(rc.top[16 * 32 + 16], 10, "the dab wrote the rock, not 0");
        // Back on Biome, erase is honoured again.
        e.set_layer(PaintTarget::Biome);
        assert!(e.set_brush(1, 3.0, 1.0, 0.0, true, true).erase);
    }

    /// Protects: the Rock drafts ride the one Commit / Discard pair --
    /// counted in `pending_stamps`, untouched by `commit_all` (the gated Rock
    /// half is the caller's), dropped by `discard_all` -- and
    /// `take_rock_commit` routes each layer's draft into its own grid.
    #[test]
    fn rock_drafts_share_the_one_commit_and_discard() {
        let n = 32 * 32;
        let mut e = PaintEditor::new(32, 32, land_mask(n));
        assert!(e.take_rock_commit(n).is_none(), "nothing pending: no take");
        for (t, v, x) in [(PaintTarget::RockTop, 3, 6.0), (PaintTarget::RockSub, 255, 16.0), (PaintTarget::RockCap, 7, 26.0)] {
            e.set_layer(t);
            e.set_brush(v, 2.0, 1.0, 0.0, false, true);
            e.stroke_at(x, 16.0);
        }
        assert_eq!((e.rock_pending(), e.pending_stamps()), (3, 3));
        e.commit_all(n);
        assert_eq!(e.rock_pending(), 3, "commit_all leaves the Rock drafts to the gated caller");
        let rc = e.take_rock_commit(n).unwrap();
        assert_eq!((rc.top[16 * 32 + 6], rc.sub[16 * 32 + 16], rc.cap[16 * 32 + 26]), (3, 255, 7));
        assert_eq!((rc.top[16 * 32 + 16], rc.sub[16 * 32 + 26], rc.cap[16 * 32 + 6]), (0, 0, 0), "each layer only its own dab");
        assert!(rc.summaries.iter().all(|s| s.stamps_applied == 1));
        assert!(rc.summaries[0].tiles_marked.contains(&0), "the painted tile is reported");
        assert!(e.layer_cells(PaintTarget::RockTop).is_none(), "no committed Rock layer exists");
        // One dab on each Rock layer: Discard drops all three.
        for t in PaintTarget::ROCK {
            e.set_layer(t);
            e.stroke_at(6.0, 6.0);
        }
        assert_eq!(e.discard_all(), 3);
        assert_eq!(e.rock_pending(), 0);
    }

    /// Protects: `painted_counts` on `rock_sub` counts "none" (255) in the
    /// twelfth slot, which the `index - 1` layout could not hold.
    #[test]
    fn rock_sub_counts_none_in_its_own_slot() {
        let n = 32 * 32;
        let mut e = PaintEditor::new(32, 32, land_mask(n));
        e.set_layer(PaintTarget::RockSub);
        e.set_brush(255, 1.0, 1.0, 0.0, false, true);
        e.stroke_at(16.0, 16.0);
        let (total, by) = e.painted_counts(n);
        assert_eq!(by.len(), 12);
        assert!(total > 0);
        assert_eq!(by[11], total, "every painted cell is 'none'");
        assert_eq!(by[..11].iter().sum::<i64>(), 0);
    }

    /// Protects: the Rock draft overlay's labelled colours, as literals: the
    /// "none" grey and the cap ramp's two ends; a rock takes the hue wheel.
    #[test]
    fn rock_swatches_are_the_labelled_literals() {
        assert_eq!(swatch_color(PaintTarget::RockSub, 255, 11), (150, 150, 150));
        assert_eq!(swatch_color(PaintTarget::RockCap, 1, 0), (235, 215, 170));
        assert_eq!(swatch_color(PaintTarget::RockCap, 255, 0), (90, 60, 30));
        assert_eq!(swatch_color(PaintTarget::RockTop, 1, 11), hsv_to_rgb(0.0, 0.65, 0.95));
        assert_eq!(swatch_color(PaintTarget::RockTop, 0, 11), (0, 0, 0));
    }
}
