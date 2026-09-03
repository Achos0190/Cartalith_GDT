//! The Icon tool's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md`
//! milestone F, the CARTO domain's Icon tool (`DCC_SHELL_SPEC.md` §4.5.5:
//! "Click stamps the armed icon (`place_manual_icon`)... The Asset library
//! arms an icon and closes; the Icon tool is what places it").
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`'s own doc comment argues for and follows the same
//! template: `lib.rs` owns the thin `Variant`<->`f64`/`String`/`Dictionary`
//! conversion and the `#[func]` surface; this module owns the actual state
//! — the placed-icon list, the armed selection, and which icon (if any) is
//! selected — with its own `#[cfg(test)]` suite below, exercised by `cargo
//! test -p cartalith-godot`'s ordinary unit-test pass with no Godot runtime
//! involved.
//!
//! ## Why this lives on `WorldGen`, not a second `GodotClass`
//!
//! Same reasoning as [`crate::sculpt_bridge::SculptEditor`]: every operation
//! here (arm, place, hit-test, resize) needs `WorldGen`'s own live grid
//! dimensions (`gw`/`gh`) and, for `icon_place`, the field/sea-level data a
//! future water-aware placement rule might read — nothing about the Icon
//! editor is independently constructible or reusable across worlds, so it is
//! a plain field (`icons: Option<IconEditor>`) exactly like `sculpt` and
//! `civ` already are, not a sibling `GodotClass` needing its own `Gd<>`
//! handle re-borrowed on every call for no benefit.
//!
//! ## What this module is built on
//!
//! Every placement/selection primitive here is a thin call into
//! `cartalith_assets::manual` — [`ArmedIcon`], [`ManualIcon`],
//! [`place_manual_icon`], [`icon_hit_test`], [`icon_resize_scale`] — already
//! written, golden-verified (`cartalith-assets/tests/
//! golden_parity_manual_icons.rs`, `UNIFIED_TOOL_PLAN.md`'s own "29 unit
//! tests... 7 golden tests" tally) and unit-tested in `manual.rs` itself.
//! Per this port's own discipline (`cartalith-porting-discipline`): expose
//! tested Rust, don't rewrite it. Nothing in this file duplicates or
//! second-guesses `manual.rs`'s own placement/hit-test/resize math; it only
//! adds the bookkeeping (a `Vec`, a selection index, an armed slot) that
//! `manual.rs` deliberately leaves to its caller.
//!
//! ## `family`/`variant` — how the numeric API addresses the asset library
//!
//! `DCC_SHELL_SPEC.md` §4.5.5's tool-options row is "family · variant ·
//! scale · rotation · jitter". `UNIFIED_TOOL_PLAN.md`'s own investigation
//! initially guessed `variant` would mean *art* variant (`pick_icon_variant`'s
//! seeded-variant logic) — but that guess was superseded by what `manual.rs`
//! actually shipped: [`ManualIcon`] carries no variant/art-index field at
//! all. Which of a slot's several art images is drawn is chosen at
//! *composite/render* time (`pack::composite_map_icons`'s
//! `pick_weighted_variant`, hashed from position + seed), identically for a
//! manually-placed icon and an auto-scattered one of the same slot — so
//! there is nothing for an *arm-time* variant index to select there.
//!
//! What a gallery tile picker genuinely needs to choose, instead, is *which
//! slot* — "mountain" vs. "hill" within the Feature family, "hamlet" vs.
//! "city" within Settlement, and so on. [`resolve_variant`] therefore reads
//! `variant` as a zero-based index into that family's own frozen vocabulary
//! (`cartalith_assets::slots::Family::slots()` — `PACK_ICON_SLOTS`,
//! `PACK_SETTLEMENT_SLOTS`, `PACK_POI_SLOTS`), the same lists
//! `ManualIconFamily::pack_family()` already resolves a family to. This
//! keeps the numeric API a caller (a gallery grid indexed 0..N) can drive
//! directly without a string round-trip, while staying inside the
//! vocabulary a real asset pack actually populates.
//!
//! **[`ManualIconFamily::Custom`] cannot be armed through this call.** Its
//! vocabulary is open (`Family::Custom.slots()` is `&[]` by design — see
//! `slots.rs`'s own doc comment) and addressed two levels deep, `set ->
//! slot`, which a single `i64` cannot express. [`resolve_variant`] returns
//! `None` for it rather than inventing an ordering over an open, pack-
//! defined set list that would silently break the moment two packs order
//! their custom sets differently. A richer arm call taking `set`/`slot`
//! strings is future work if the shell needs Custom icons before then.
//!
//! ## Arm-time `scale` is a disclosed addition; `rotation`/`jitter` are not applied
//!
//! The reference's own click path always places at `scale: 1` (reference
//! lines 9776-9784, `place_manual_icon`'s own doc comment) — there is no
//! arm-time scale control in the reference at all; only the resize handle
//! (`icon_resize_scale`) changes a placed icon's scale, after the fact.
//! `DCC_SHELL_SPEC.md`'s tool-options row adds one anyway, so [`IconEditor::place`]
//! honours it: it calls `place_manual_icon` unchanged (its own golden-backed
//! `scale == 1.0` behaviour is untouched) and then overwrites the *returned*
//! `ManualIcon`'s `scale` field with the armed value before storing it. This
//! is a disclosed, boundary-layer addition — not a change to `manual.rs` —
//! but it is a genuine behavioural difference from the reference's click
//! path, flagged here per `cartalith-porting-discipline`'s "anything that
//! changes output" rule rather than assumed correct.
//!
//! `rotation` and `jitter` have no equivalent in the reference at all
//! ([`ManualIcon`] carries no rotation field, and the brush's own
//! "jitter" is the dart-throwing randomness itself, not a scalar control —
//! see `manual.rs`'s module doc). [`IconEditor::arm`] accepts and stores
//! both purely so the tool-options row and the armed-icon "chip"
//! (`DCC_SHELL_SPEC.md` §4.5.5) have somewhere to keep them, but neither
//! ever reaches a placed [`ManualIcon`] — there is no field on that type to
//! write them into, and adding one would be exactly the kind of
//! `cartalith-assets` rewrite this module is chartered not to do.

use cartalith_assets::manual::{
    civ_zoom_k, icon_box, icon_brush_stamp, icon_hit_test, icon_resize_scale, place_manual_icon,
    ArmedIcon, IconBox, IconBrush, IconHandle, IconHit, IconHitKind, IconViewEnv, ManualIcon,
    ManualIconFamily,
};
use cartalith_assets::ScatterRule;
use cartalith_civ::labels::LabelRect;
use cartalith_rng::Mulberry32;

use crate::selection::{SelectMode, SelectionSet};

/// The generated placement pass's own `#[godot_api(secondary)]` surface —
/// `icon_bridge/generate.rs`.
///
/// A child module for exactly the reason `label_bridge.rs` gives for its own:
/// this file promises it is free of any `godot` dependency and is exercised
/// without a Godot runtime, a sibling top-level module would have to be
/// registered in `lib.rs` (which three lanes are editing on this date), and a
/// child module is neither.
mod generate;

/// The density brush's own `#[godot_api(secondary)]` surface —
/// `icon_bridge/brush.rs`. A child module for the same reason `generate`
/// above is one.
mod brush;

/// The reference's own `#carIconBrushR` slider bounds (reference line 1656:
/// `min="2" max="60" value="12"`), which [`IconBrush::default`] already
/// matches at 12.
pub const ICON_BRUSH_R_MIN: f64 = 2.0;
/// See [`ICON_BRUSH_R_MIN`].
pub const ICON_BRUSH_R_MAX: f64 = 60.0;
/// `#carIconBrushD` (reference line 1657: `min="5" max="200" value="60"`),
/// divided by 100 by the reference's own `input` listener at line 13515 —
/// so the authored range is `0.05 .. 2.00`, default `0.6`.
///
/// **This deliberately reaches past 1.0**, and that contradicts
/// [`IconBrush::density`]'s own doc comment, which says `0..1`. The slider is
/// the reference's and it is what `_carIconBrush.density` is actually fed, so
/// the range here follows the slider; the doc comment in `cartalith-assets`
/// is the stale party. Nothing breaks above 1: `icon_brush_stamp` only ever
/// uses `density` through `3.0 / density.sqrt()`, which stays finite and
/// simply floors at [`cartalith_assets::manual::ICON_BRUSH_MIN_SPACING`].
pub const ICON_BRUSH_DENSITY_MIN: f64 = 0.05;
/// See [`ICON_BRUSH_DENSITY_MIN`].
pub const ICON_BRUSH_DENSITY_MAX: f64 = 2.0;

/// The seed [`IconEditor::new`] starts its brush stream from.
///
/// The reference uses `Math.random` here and says why in its own comment:
/// *"a brush stroke is an authoring ACTION whose result is persisted in
/// `state.mapIcons` — re-painting the same spot should add new icons, not
/// deterministically reproduce the previous ones."* A **fixed** seed on a
/// **single long-lived stream** satisfies exactly that requirement — the
/// stream advances three draws per accepted dart and two per rejected one,
/// so the second stamp over one spot throws different darts from the first —
/// while leaving the editor testable without a clock. What it does *not* do
/// is vary between two sessions that place the same strokes in the same
/// order, and that is not a property the reference's comment asks for.
const BRUSH_SEED: u32 = 0x_CA12_1C04;

/// The armed selection plus the tool-options row's extra chip fields —
/// see the module doc's "Arm-time `scale`..." section for which of these
/// three actually reaches a placed icon.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmedSelection {
    pub icon: ArmedIcon,
    pub scale: f64,
    pub rotation: f64,
    pub jitter: f64,
}

/// Bounds a caller-supplied arm-time scale is clamped to, reusing
/// `manual.rs`'s own resize bounds ([`cartalith_assets::manual::ICON_SCALE_MIN`]/
/// `_MAX`) rather than inventing a second range for what is, after
/// placement, the exact same field a resize drag also writes.
pub use cartalith_assets::manual::{ICON_SCALE_MAX, ICON_SCALE_MIN};

/// Resolves `(family, variant)` into the slot string [`ArmedIcon::slot`]
/// needs — see the module doc's "how the numeric API addresses the asset
/// library" section. `None` for [`ManualIconFamily::Custom`] (open
/// vocabulary, not expressible as one index) or an out-of-range/negative
/// `variant`.
pub fn resolve_variant(family: ManualIconFamily, variant: i64) -> Option<String> {
    if family == ManualIconFamily::Custom {
        return None;
    }
    let i = usize::try_from(variant).ok()?;
    family.pack_family().slots().get(i).map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// The generated placement pass
// ---------------------------------------------------------------------------

/// The design's four **placement** families — `cartalith-dcc-parts.js:364`'s
/// `FAM`, drawn as the ICONS panel's family chips.
///
/// **Four onto four, not four onto three, and that is the whole of the owner's
/// 2026-09-02 ruling.** `cartography_workspace.gd` used to carry these as a
/// transcribed table above a comment stating the problem: *"SEA MARKS has no
/// counterpart in the engine's three families at all. Mapping one onto the
/// other would be inventing a correspondence the design does not state."* The
/// answer was to build the missing family
/// ([`cartalith_assets::PACK_SEAMARK_SLOTS`]), not to invent the mapping — so
/// every arm of [`Self::icon_family`] below is now an identity between two real
/// things, and none of them is a guess.
///
/// The two lists still answer different questions, which is why this enum
/// exists at all rather than the panel driving [`ManualIconFamily`] directly: a
/// *placement* family is what a generated pass runs over, and TREES is a
/// deliberately narrower thing than the `icons` art family it draws from (see
/// [`Self::slots`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementFamily {
    Places,
    Trees,
    SeaMarks,
    Poi,
}

/// The `tree_*` half of [`cartalith_assets::PACK_ICON_SLOTS`].
///
/// The design's TREES family is trees; the engine's `icons` family is every
/// scattered feature glyph, mountains and boulders included. Placing a mountain
/// because the user pressed TREES would be the same kind of invented
/// correspondence the ruling exists to remove, so the placement family names
/// the five slots that are actually trees and leaves the other five to the
/// scatter-rule engine that already owns them
/// (`cartalith_assets::place_map_icons_ruled`).
pub const TREE_SLOTS: [&str; 5] = [
    "tree_conifer",
    "tree_broadleaf",
    "tree_rainforest",
    "tree_savanna",
    "tree_wetland",
];

impl PlacementFamily {
    /// Every family, in the design's own chip order.
    pub const ALL: [PlacementFamily; 4] = [
        PlacementFamily::Places,
        PlacementFamily::Trees,
        PlacementFamily::SeaMarks,
        PlacementFamily::Poi,
    ];

    /// The design's own id, as `parts.js` spells it and as the chip reads.
    pub fn key(self) -> &'static str {
        match self {
            PlacementFamily::Places => "PLACES",
            PlacementFamily::Trees => "TREES",
            PlacementFamily::SeaMarks => "SEA MARKS",
            PlacementFamily::Poi => "POI",
        }
    }

    pub fn from_key(key: &str) -> Option<PlacementFamily> {
        PlacementFamily::ALL.into_iter().find(|f| f.key() == key)
    }

    /// The asset family a glyph of this placement family is drawn from.
    pub fn icon_family(self) -> ManualIconFamily {
        match self {
            PlacementFamily::Places => ManualIconFamily::Settlement,
            PlacementFamily::Trees => ManualIconFamily::Feature,
            PlacementFamily::SeaMarks => ManualIconFamily::SeaMark,
            PlacementFamily::Poi => ManualIconFamily::Poi,
        }
    }

    /// The slot vocabulary a generated candidate of this family may name —
    /// its asset family's frozen list, narrowed to [`TREE_SLOTS`] for TREES.
    pub fn slots(self) -> &'static [&'static str] {
        match self {
            PlacementFamily::Trees => &TREE_SLOTS,
            other => other.icon_family().pack_family().slots(),
        }
    }
}

/// One thing the pass may place, before it decides whether it does.
///
/// `x`/`y` are grid-cell coordinates, the frame [`ManualIcon`] and every other
/// call in this module use. `slot` must be one of the plan family's
/// [`PlacementFamily::slots`]; one that is not is dropped and counted, never
/// silently placed under the wrong family.
#[derive(Debug, Clone, PartialEq)]
pub struct IconCandidate {
    pub x: f64,
    pub y: f64,
    pub slot: String,
}

/// What the *"snap sea marks to coast"* rule needs: the height field to find a
/// shore in, and how far it may look.
///
/// `Some` **is** the rule being on. It is ignored for every family except
/// [`PlacementFamily::SeaMarks`] — the design's rule names sea marks, and
/// pulling a settlement pin onto a beach would be a different rule nobody
/// asked for.
pub struct CoastSnap<'a> {
    pub field: &'a [f32],
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    /// Search radius in cells. A candidate with no coast cell inside it is
    /// **dropped**, not placed inland: a sea mark that did not land on a coast
    /// is the entire failure mode of this family.
    pub max_r: i64,
}

/// The default snap radius for a `gw`-wide grid: `max(8, gw / 32)`.
///
/// Wide enough that a coarse candidate sweep over a normal coastline always
/// finds the shore, small enough that a mark generated in the middle of a
/// continent is dropped rather than teleported to another sea. Both halves are
/// this port's choice — there is no reference figure — and the floor is what
/// keeps a tiny test grid working at all.
pub fn default_snap_radius(gw: usize) -> i64 {
    ((gw / 32) as i64).max(8)
}

/// One run's dials — the ICONS panel's own two sliders and three rule toggles.
pub struct IconGenPlan {
    pub family: PlacementFamily,
    /// Per-instance scale, clamped exactly as [`IconEditor::arm`] clamps the
    /// hand-placement one.
    pub scale: f64,
    /// Minimum centre-to-centre separation in **grid cells**.
    pub min_spacing: f64,
    /// *"avoid label boxes"* — measure generated icons against the label rects
    /// the caller reserved.
    pub avoid_labels: bool,
    /// *"enforce min spacing"*. With this off, icons are still kept from
    /// overlapping each other's own footprints; what it adds is the slider.
    pub enforce_spacing: bool,
}

/// What one run did. Every number is a real count, and the four rejection
/// counters are disjoint — a candidate is charged to exactly one of them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IconGenReport {
    pub placed: usize,
    /// Dropped for hitting another icon (generated this run, or already on the
    /// map) — the *"enforce min spacing"* rule and the footprint floor under it.
    pub culled_spacing: usize,
    /// Dropped for hitting a reserved label box — *"avoid label boxes"*.
    pub culled_label: usize,
    /// Sea marks with no coast cell within [`CoastSnap::max_r`].
    pub off_coast: usize,
    /// Candidates naming a slot outside the family's own vocabulary.
    pub unknown_slot: usize,
    /// How many sea marks the snap actually moved. `placed - snapped` were
    /// already on a coast cell.
    pub snapped: usize,
}

/// The live Icon-editor state for one generated world: every hand-placed
/// icon, the current armed selection (if any), and which placed icon (if
/// any) is selected — the reference's own `state.mapIcons`/`_carIconArmed`/
/// the click handler's own "select what was just placed or hit" convention,
/// kept together the way `SculptEditor` keeps its own draft/tool-state/
/// selection together.
pub struct IconEditor {
    pub icons: Vec<ManualIcon>,
    pub armed: Option<ArmedSelection>,
    /// Which placed icons are selected — [`crate::selection::SelectionSet`],
    /// whose `primary()` is what the old `selected: Option<usize>` field was
    /// and what `icon_get_selected` still reports. Step one of the owner's
    /// selection-sets ruling; see that module's own doc comment.
    pub selection: SelectionSet,
    /// The selected icon's own `scale` at the moment it became selected —
    /// the baseline [`IconEditor::resize`] scales from. Captured once per
    /// selection (by [`IconEditor::place`]/[`IconEditor::hit_test`], the
    /// two ways a selection can start) rather than read live off the icon
    /// on every `resize` call, because a drag calls `resize` repeatedly
    /// with the *same* `start_dist` it captured at grab-time
    /// (`icon_resize_scale`'s own contract, mirroring the reference's own
    /// fixed `_iconResize.startScale`/`startDist` for the whole gesture) —
    /// reading the icon's live scale instead would compound the ratio on
    /// every intermediate call rather than computing it fresh from the
    /// drag's own start each time.
    resize_base_scale: Option<f64>,
    /// The density brush's own three fields — the reference's
    /// `_carIconBrush={on, r, density}` minus its `painting` flag, which is
    /// pointer state and stays in the shell where the pointer is
    /// (`cartography_workspace.gd`'s `_icon_brush_painting`).
    pub brush: IconBrush,
    /// The brush's dart stream — see [`BRUSH_SEED`]. Not reset by
    /// [`IconEditor::clear_all`]: clearing the map is not a reason to
    /// re-throw the same darts the next stroke would have thrown anyway.
    brush_rng: Mulberry32,
}

impl IconEditor {
    /// A fresh, empty editor — nothing armed, nothing placed, nothing
    /// selected. Called once per `generate()`/`generate_sized()`
    /// (`WorldGen::absorb`), matching `SculptEditor::new`'s own "a fresh
    /// draft over this world's own dimensions" pattern: a hand-placed
    /// icon's `x`/`y` are grid-cell coordinates over one particular world,
    /// meaningless carried over to a differently-sized one.
    pub fn new() -> Self {
        IconEditor {
            icons: Vec::new(),
            armed: None,
            selection: SelectionSet::new(),
            resize_base_scale: None,
            brush: IconBrush::default(),
            brush_rng: Mulberry32::new(BRUSH_SEED),
        }
    }

    /// The density brush's three controls — the reference's own
    /// `carIconBrushChk`/`carIconBrushR`/`carIconBrushD` listeners (lines
    /// 13511/13513/13515), with the clamping the `<input type="range">`
    /// elements do for free and a Rust caller does not.
    ///
    /// `false` when either number was non-finite: the previous setting is
    /// then left untouched, matching [`IconEditor::arm`]'s own "a rejected
    /// call changes nothing" policy. `on` is applied either way — turning
    /// the brush off must not be refusable by a bad slider value beside it.
    pub fn set_brush(&mut self, on: bool, r: f64, density: f64) -> bool {
        self.brush.on = on;
        if !r.is_finite() || !density.is_finite() {
            return false;
        }
        self.brush.r = r.clamp(ICON_BRUSH_R_MIN, ICON_BRUSH_R_MAX);
        self.brush.density = density.clamp(ICON_BRUSH_DENSITY_MIN, ICON_BRUSH_DENSITY_MAX);
        true
    }

    /// One brush stamp at grid cell `(cx, cy)` — `_carIconBrushStamp`
    /// (reference line 15051), driven from this editor's own dart stream.
    /// Returns how many icons it added, which is legitimately `0` for a
    /// stamp that landed entirely in water or entirely inside the spacing
    /// of icons already there.
    ///
    /// **The arm-time `scale` override deliberately does not apply here.**
    /// [`IconEditor::place`] writes it over `place_manual_icon`'s `1.0`
    /// because the reference's click path has no size at all; the brush
    /// path *does* — every dart takes its own size from the rule's
    /// `min_size`/`max_size` — so overriding it would replace a real
    /// per-instance variation with one flat number.
    ///
    /// Appends only. Existing selection indices stay valid, which is why
    /// nothing here touches [`IconEditor::selection`]; the reference does
    /// not select brushed icons either.
    #[allow(clippy::too_many_arguments)]
    pub fn brush_stamp(
        &mut self,
        rule: &ScatterRule,
        field: &[f32],
        gw: usize,
        gh: usize,
        sea_level: f64,
        cx: f64,
        cy: f64,
    ) -> usize {
        if !self.brush.on {
            return 0;
        }
        // Destructured rather than called through `self`, so the dart
        // closure can borrow the RNG while `icon_brush_stamp` holds a
        // `&mut` on the icon list.
        let Self { icons, armed, brush, brush_rng, .. } = self;
        let armed = armed.as_ref().map(|a| &a.icon);
        let mut rng = || brush_rng.next_f64();
        icon_brush_stamp(icons, armed, brush, rule, field, gw, gh, sea_level, cx, cy, &mut rng)
    }

    /// The icon a single-selection operation acts on — the set's primary. What
    /// `icon_get_selected` reports, and what the old `selected` field held
    /// before it became a set.
    pub fn selected(&self) -> Option<usize> {
        self.selection.primary()
    }

    /// Arms `family`/`variant` (see the module doc) for the next
    /// `place()` call. `scale` is clamped to [`ICON_SCALE_MIN`]/
    /// [`ICON_SCALE_MAX`] (non-finite or non-positive -> `1.0`, the
    /// reference's own click-path default); `rotation`/`jitter` are stored
    /// as given (non-finite -> `0.0`) — see the module doc for why neither
    /// currently reaches a placed icon. `false` for an unrecognised
    /// `family` key, a `variant` outside that family's own vocabulary, or
    /// `family == "custom"` (not addressable this way — see
    /// [`resolve_variant`]); the previous armed selection (if any) is left
    /// untouched on a rejected call, matching `set_feature_param`'s own
    /// "typo is visibly rejected, not silently applied" policy elsewhere
    /// in this crate.
    pub fn arm(&mut self, family_key: &str, variant: i64, scale: f64, rotation: f64, jitter: f64) -> bool {
        let Some(family) = ManualIconFamily::from_key(family_key) else { return false };
        let Some(slot) = resolve_variant(family, variant) else { return false };
        let scale = if scale.is_finite() && scale > 0.0 { scale.clamp(ICON_SCALE_MIN, ICON_SCALE_MAX) } else { 1.0 };
        let rotation = if rotation.is_finite() { rotation } else { 0.0 };
        let jitter = if jitter.is_finite() { jitter } else { 0.0 };
        self.armed = Some(ArmedSelection { icon: ArmedIcon { family, slot, set: None }, scale, rotation, jitter });
        true
    }

    /// Disarms — the next `place()` call does nothing until `arm()` is
    /// called again. Matches the reference's own `_carIconArmed=null`
    /// (fired on Escape, switching family, or arming a different tool —
    /// `DCC_SHELL_SPEC.md` §4.5.6's "arming any tool clears... its armed
    /// icon"; `lib.rs` is responsible for calling this at those points,
    /// the same way it already owns cross-tool disarm sequencing).
    pub fn disarm(&mut self) {
        self.armed = None;
    }

    /// Stamps the armed icon at grid cell `(gx, gy)` — `place_manual_icon`
    /// plus the arm-time scale override (see the module doc). Selects the
    /// new icon (the reference's own "click... places... and selects it").
    /// Returns the new index, or `None` when nothing is armed or the click
    /// is off-grid (`place_manual_icon`'s own bounds gate).
    pub fn place(&mut self, gx: f64, gy: f64, gw: usize, gh: usize) -> Option<usize> {
        let armed = self.armed.as_ref()?;
        let mut icon = place_manual_icon(gx, gy, gw, gh, Some(&armed.icon))?;
        icon.scale = armed.scale;
        let index = self.icons.len();
        self.icons.push(icon);
        self.select(index, SelectMode::Replace);
        Some(index)
    }

    /// `_carIconHitTest`'s box-hit half only (`manual.rs`'s own
    /// `icon_hit_test`, `None` handle). Boxes are computed in **grid
    /// space** (`env`'s `zoom_scale`/`icon_scale` at their defaults unless
    /// the caller overrides them), matching `gx`/`gy` here and in `place`/
    /// `resize` all being grid coordinates, not screen pixels — a caller
    /// converts a real pointer event through its own view transform first,
    /// same convention `sculpt_add_point`'s own doc comment states for the
    /// Sculpt tool. Selects and returns the hit icon's index on a hit
    /// (matching the reference's own hit-then-select click sequencing);
    /// `None` (selection unchanged) on a miss.
    ///
    /// **Box hits only, still** — matching `label_bridge::LabelBridge::
    /// hit_test`'s own precedent: a *handle* hit is the shell's own job,
    /// by comparing the pointer against the circle [`IconEditor::handles`]
    /// returns for whichever icon is selected (`GUI_GAP_REGISTER.md` CA-05
    /// closed that gap — see this file's own module doc for why it lives
    /// here now).
    ///
    /// `mode` is what the hit does to the selection set —
    /// [`SelectMode::Replace`] is the plain click this always did, the other
    /// two are the modifier-click conventions the shell already uses in its
    /// own multi-select grid (`crate::selection`'s own doc comment). A **miss
    /// leaves the selection alone in every mode**, including Toggle: a
    /// Ctrl-click on empty ground is not a deselect-everything gesture in any
    /// of the three shells this port draws from.
    pub fn hit_test(&mut self, gx: f64, gy: f64, env: &IconViewEnv, mode: SelectMode) -> Option<usize> {
        let boxes: Vec<IconBox> = self.icons.iter().map(|ic| icon_box(ic, env)).collect();
        match icon_hit_test(&boxes, None, gx, gy) {
            Some(IconHit { kind: IconHitKind::Box, index: Some(i) }) => {
                self.select(i, mode);
                Some(i)
            }
            _ => None,
        }
    }

    /// Icon `index`'s on-canvas resize-handle circle — see [`icon_handle`].
    /// `None` for an out-of-range `index`. Unlike [`IconEditor::select`],
    /// this does not require `index` to be the current selection: exactly
    /// `label_bridge::LabelBridge::handles`' own contract (any valid index
    /// works; a caller decides which index to ask for, normally whichever
    /// one is currently selected).
    pub fn handles(&self, index: usize, env: &IconViewEnv) -> Option<IconHandle> {
        let icon = self.icons.get(index)?;
        let box_ = icon_box(icon, env);
        Some(icon_handle(&box_, env))
    }

    /// Applies one click at `index` in `mode` and re-snapshots whichever icon
    /// is primary afterwards as the next `resize()` gesture's baseline (see
    /// `resize_base_scale`'s own doc comment). Private: the two ways a
    /// selection legitimately starts are `place` and `hit_test`, both above.
    ///
    /// The snapshot follows the **primary**, not `index`: a Ctrl-click that
    /// toggles `index` *off* promotes an earlier member, and a baseline left
    /// pointing at the icon that just left the set is the same bug `deselect`
    /// clears it for.
    fn select(&mut self, index: usize, mode: SelectMode) {
        self.selection.apply(mode, index);
        self.resize_base_scale = self.selection.primary().and_then(|i| self.icons.get(i)).map(|ic| ic.scale);
    }

    /// Replaces the whole selection from a caller-supplied list, dropping any
    /// index past the end of `icons` (`SelectionSet::set_from`'s own contract).
    /// Returns whether every requested index was in range.
    pub fn select_set(&mut self, indices: impl IntoIterator<Item = usize>) -> bool {
        let all_valid = self.selection.set_from(indices, self.icons.len());
        self.resize_base_scale = self.selection.primary().and_then(|i| self.icons.get(i)).map(|ic| ic.scale);
        all_valid
    }

    /// Drops the selection without touching any icon — `Edit ▸ Deselect`
    /// (`DCC_SHELL_SPEC.md` §2.2, "Select all / Deselect ⌘A ⌘D"), the third
    /// legitimate way a selection changes and the only one that ends it.
    ///
    /// `resize_base_scale` goes with it. It is the baseline the next
    /// `resize()` drag measures against, snapshotted by `select`, so leaving
    /// it behind would let a drag on a freshly-selected icon measure from the
    /// *previous* one's scale.
    pub fn deselect(&mut self) {
        self.selection.clear();
        self.resize_base_scale = None;
    }

    /// Applies one resize-drag sample to the selected icon's `scale` —
    /// `icon_resize_scale(base, cx, cy, gx, gy, start_dist)`, `base` being
    /// the snapshot `select` took, not the icon's live (already-updated-
    /// this-gesture) scale (see `resize_base_scale`'s own doc comment for
    /// why). Requires `index` to already be the selected icon (a drag on a
    /// box the caller hasn't hit-tested/selected first is a caller bug, not
    /// a silently-accepted resize of the wrong icon); `false` otherwise, or
    /// for an out-of-range `index`.
    pub fn resize(&mut self, index: usize, cx: f64, cy: f64, gx: f64, gy: f64, start_dist: f64) -> bool {
        if self.selected() != Some(index) {
            return false;
        }
        let Some(base) = self.resize_base_scale else { return false };
        let Some(icon) = self.icons.get_mut(index) else { return false };
        icon.scale = icon_resize_scale(base, cx, cy, gx, gy, start_dist);
        true
    }

    /// Removes icon `index`. Clears the selection if it pointed at the
    /// removed icon; shifts a selection pointing past it down by one so it
    /// keeps addressing the same logical icon in the now-shorter `Vec`
    /// (`sculpt_bridge`'s stamp-stack equivalents clear rather than shift,
    /// but the stamp *stack* has no "everything after this one renumbers"
    /// property to preserve — a flat `Vec::remove` here does, and losing
    /// track of an unrelated selection on every delete would be a worse
    /// default for a list a caller is actively editing). `false` for an
    /// out-of-range `index`.
    pub fn delete(&mut self, index: usize) -> bool {
        if index >= self.icons.len() {
            return false;
        }
        let was_primary = self.selected() == Some(index);
        self.icons.remove(index);
        self.selection.retain_after_remove(index);
        if was_primary {
            // The baseline belonged to the icon that just left. At one member
            // the set is now empty and this is the old `None`; with more, it
            // re-snapshots whichever member the promotion made primary.
            self.resize_base_scale = self.selection.primary().and_then(|i| self.icons.get(i)).map(|ic| ic.scale);
        }
        true
    }

    /// Run the generated placement pass over `candidates` and append what
    /// survives to [`Self::icons`].
    ///
    /// # It appends, and re-running it is a no-op
    ///
    /// There is no second list and no `generated` flag, unlike
    /// `label_bridge::LabelBridge`'s two-list split — and that difference is
    /// deliberate rather than an omission. A label needed the split because
    /// generated labels are *replaced* wholesale on every re-run, which would
    /// shift every hand-placed index underneath the edit session. This pass
    /// does not need to replace anything: existing icons are measured as
    /// obstacles like any other, candidate positions are a deterministic
    /// function of the world, so a second run over the same world and family
    /// finds every one of its own candidates already occupied and places
    /// nothing. Idempotence falls out of the culling instead of out of
    /// bookkeeping. `icon_list`, `icon_delete`, `icon_hit_test` and the
    /// selection therefore keep working on one flat list, unchanged.
    ///
    /// # The culler is the one that shipped in `0f0fe55`, not a second one
    ///
    /// Every overlap test here is [`LabelRect::overlaps`] over
    /// [`cartalith_civ::labels::LabelRect`] — the same type, the same
    /// half-sum comparison and the same NaN rule (`<` is false on NaN, so an
    /// unmeasurable box never suppresses anything) that
    /// `cartalith_civ::labels::generate_labels` uses for label collision. Two
    /// boxes of side `s` centred `d` apart overlap when `d < s`, so a spacing
    /// rect of side `min_spacing` expresses *"centres at least `min_spacing`
    /// cells apart"* exactly, and `reserved` — measured by the caller with
    /// `label_cull_rect` off the live typography table — needs no conversion
    /// at all. Nothing in `cartalith-assets` was grown to match: this pass
    /// lives here, next to the icon state, precisely because `cartalith-assets`
    /// does not depend on `cartalith-civ` and giving it that dependency would
    /// be a `Cargo.toml` edit. The geometry that *doesn't* need the culler —
    /// the coastline — is in `cartalith_assets::coast` where it belongs.
    ///
    /// Two rects per candidate, not one:
    /// - its **footprint** ([`icon_box`]'s own `side`, the box `hit_test`
    ///   already uses) is what `avoid_labels` measures against a label, since
    ///   a spacing slider should not decide how far a glyph sits from a name;
    /// - its **spacing rect**, the footprint widened to `min_spacing` when
    ///   `enforce_spacing` is on, is what icon-versus-icon uses. With the rule
    ///   off it is the footprint alone, so icons still never overlap.
    ///
    /// `reserved` is never culled and never returned — `generate_labels`' own
    /// contract for the same argument.
    pub fn generate(
        &mut self,
        plan: &IconGenPlan,
        candidates: &[IconCandidate],
        env: &IconViewEnv,
        reserved: &[LabelRect],
        snap: Option<&CoastSnap>,
    ) -> IconGenReport {
        let mut report = IconGenReport::default();
        let family = plan.family.icon_family();
        let scale = if plan.scale.is_finite() && plan.scale > 0.0 {
            plan.scale.clamp(ICON_SCALE_MIN, ICON_SCALE_MAX)
        } else {
            1.0
        };
        let spacing = if plan.enforce_spacing && plan.min_spacing.is_finite() {
            plan.min_spacing.max(0.0)
        } else {
            0.0
        };
        // The snap is the sea-marks rule, and only that family's.
        let snap = snap.filter(|_| plan.family == PlacementFamily::SeaMarks);

        // Everything already on the map is an obstacle. Measured once, with
        // each icon's own scale, not the plan's.
        let mut occupied: Vec<LabelRect> =
            self.icons.iter().map(|ic| spacing_rect(ic, env, spacing)).collect();

        for cand in candidates {
            if !plan.family.slots().contains(&cand.slot.as_str()) {
                report.unknown_slot += 1;
                continue;
            }
            let (mut x, mut y) = (cand.x, cand.y);
            // Counted only if this candidate survives to be placed —
            // `IconGenReport::snapped` describes icons on the map, not
            // candidates that moved and were then culled.
            let mut moved = false;
            if let Some(s) = snap {
                let Some((sx, sy)) = cartalith_assets::coast::snap_to_coast(
                    s.field,
                    s.gw,
                    s.gh,
                    s.sea,
                    // `js_round`, not Rust's: a candidate at exactly -0.5 must
                    // round to 0 the way every other coordinate in this port
                    // does. `cartalith-rust-conventions`' own rule.
                    cartalith_jsmath::js_round(x) as i64,
                    cartalith_jsmath::js_round(y) as i64,
                    s.max_r,
                ) else {
                    report.off_coast += 1;
                    continue;
                };
                moved = (sx as f64, sy as f64) != (x, y);
                x = sx as f64;
                y = sy as f64;
            }
            let icon = ManualIcon {
                x,
                y,
                family,
                slot: cand.slot.clone(),
                // `set` is meaningful only for `Custom`, which no placement
                // family resolves to -- `ManualIcon`'s own contract.
                set: None,
                scale,
            };
            if plan.avoid_labels {
                let foot = footprint_rect(&icon, env);
                if reserved.iter().any(|r| foot.overlaps(r)) {
                    report.culled_label += 1;
                    continue;
                }
            }
            let rect = spacing_rect(&icon, env, spacing);
            if occupied.iter().any(|r| rect.overlaps(r)) {
                report.culled_spacing += 1;
                continue;
            }
            occupied.push(rect);
            self.icons.push(icon);
            report.placed += 1;
            report.snapped += usize::from(moved);
        }
        report
    }

    /// Drops every placed icon and the current selection (armed selection
    /// untouched — `DCC_SHELL_SPEC.md` §4.5.5's list panel's own
    /// "Clear-all" clears placements, not the gallery's own arming state).
    pub fn clear_all(&mut self) {
        self.icons.clear();
        self.selection.clear();
        self.resize_base_scale = None;
    }
}

impl Default for IconEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// The reference's `drawCivLayer` selected-icon resize-handle geometry
/// (lines 15883-15893 of `reference/Cartalith Gen1 v2.10.html`),
/// transcribed rather than sliced — exactly `label_bridge::handle_circles`'
/// own reasoning: this is inline canvas drawing code, not a callable
/// function, so `manual.rs` never had a home for it (that module's own
/// `IconEditor::hit_test`/`icon_bridge.rs` doc comments called this out as
/// the one acknowledged gap — `GUI_GAP_REGISTER.md` CA-05).
///
/// One handle only — a manually-placed icon has no rotate/arc field at all
/// (`manual.rs`'s own module doc), so unlike a label's five circles there
/// is nothing else to compute here.
///
/// `lsc` is `label_bridge::handle_circles`'s own render-pass constant
/// (`Math.max(1,GW/512)*_civZoomK()*_civIconScale()`) — computed inline
/// here rather than shared with that function, matching `IconBox`'s own
/// doc comment on why icon/label geometry stay separate rather than behind
/// a shared abstraction (`cartalith-assets` has no dependency on
/// `cartalith-civ` to share one through, either).
/// One icon's own drawn footprint as a [`LabelRect`] — [`icon_box`]'s centre
/// and its full `side`, in grid cells.
///
/// The same box `IconEditor::hit_test` compares a click against, so what the
/// pass calls "overlapping" is what a user would call it. No new geometry:
/// this is a two-field rename of a value `manual.rs` already computes.
fn footprint_rect(icon: &ManualIcon, env: &IconViewEnv) -> LabelRect {
    let b = icon_box(icon, env);
    LabelRect { cx: b.px, cy: b.py, w: b.side, h: b.side }
}

/// The footprint widened to at least `spacing` cells on each side.
///
/// `max`, not a sum: with `spacing` at 0 (the rule off) this is the footprint
/// exactly, and with a spacing larger than the glyph it is the slider's own
/// figure, which is what makes `LabelRect::overlaps` read as *"centres at
/// least `spacing` apart"* between two icons of the same size.
fn spacing_rect(icon: &ManualIcon, env: &IconViewEnv, spacing: f64) -> LabelRect {
    let mut r = footprint_rect(icon, env);
    r.w = r.w.max(spacing);
    r.h = r.h.max(spacing);
    r
}

pub fn icon_handle(box_: &IconBox, env: &IconViewEnv) -> IconHandle {
    let lsc = f64::max(1.0, env.grid_w as f64 / 512.0) * civ_zoom_k(env.zoom_scale) * env.icon_scale;
    let hr = f64::max(4.0, 3.2 * lsc);
    let hx = box_.px + box_.side / 2.0 * 0.7;
    let hy = box_.py + box_.side / 2.0 * 0.7;
    // The reference's own hit-test radius bakes in the *displayed* circle's
    // own further slack (`_iconHandle={..,r:hr*1.6,..}`, reference line
    // 15893) — not the drawn `hr` alone, matching `label_bridge::
    // handle_circles`' own resize/rotate/arc handles doing the same thing.
    IconHandle { x: hx, y: hy, r: hr * 1.6 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> IconViewEnv {
        IconViewEnv { grid_w: 48, zoom_scale: 1.0, icon_scale: 1.0 }
    }

    // ---- resolve_variant ----

    #[test]
    fn resolve_variant_indexes_the_familys_own_frozen_slots() {
        assert_eq!(resolve_variant(ManualIconFamily::Feature, 0).as_deref(), Some("mountain"));
        assert_eq!(resolve_variant(ManualIconFamily::Feature, 1).as_deref(), Some("hill"));
        assert_eq!(resolve_variant(ManualIconFamily::Settlement, 0).as_deref(), Some("hamlet"));
        assert_eq!(resolve_variant(ManualIconFamily::Poi, 0).as_deref(), Some("ruin"));
    }

    #[test]
    fn resolve_variant_rejects_out_of_range_and_negative() {
        assert_eq!(resolve_variant(ManualIconFamily::Feature, 999), None);
        assert_eq!(resolve_variant(ManualIconFamily::Feature, -1), None);
    }

    #[test]
    fn resolve_variant_rejects_custom_entirely() {
        assert_eq!(resolve_variant(ManualIconFamily::Custom, 0), None);
    }

    // ---- arm / disarm ----

    #[test]
    fn arm_with_an_unknown_family_key_fails_and_changes_nothing() {
        let mut e = IconEditor::new();
        assert!(!e.arm("nope", 0, 1.0, 0.0, 0.0));
        assert!(e.armed.is_none());
    }

    #[test]
    fn arm_with_custom_fails() {
        let mut e = IconEditor::new();
        assert!(!e.arm("custom", 0, 1.0, 0.0, 0.0));
    }

    #[test]
    fn arm_rejects_leave_a_previous_armed_selection_untouched() {
        let mut e = IconEditor::new();
        assert!(e.arm("feature", 0, 1.0, 0.0, 0.0));
        let before = e.armed.clone();
        assert!(!e.arm("feature", 999, 1.0, 0.0, 0.0));
        assert_eq!(e.armed, before);
    }

    #[test]
    fn arm_clamps_scale_and_defaults_a_bad_one_to_one() {
        let mut e = IconEditor::new();
        assert!(e.arm("feature", 0, 999.0, 0.0, 0.0));
        assert_eq!(e.armed.as_ref().unwrap().scale, ICON_SCALE_MAX);
        assert!(e.arm("feature", 0, f64::NAN, 0.0, 0.0));
        assert_eq!(e.armed.as_ref().unwrap().scale, 1.0);
        assert!(e.arm("feature", 0, -5.0, 0.0, 0.0));
        assert_eq!(e.armed.as_ref().unwrap().scale, 1.0);
    }

    #[test]
    fn arm_stores_rotation_and_jitter_verbatim_when_finite() {
        let mut e = IconEditor::new();
        assert!(e.arm("feature", 0, 1.0, 45.0, 0.7));
        let a = e.armed.as_ref().unwrap();
        assert_eq!(a.rotation, 45.0);
        assert_eq!(a.jitter, 0.7);
    }

    #[test]
    fn disarm_clears_the_armed_selection() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.disarm();
        assert!(e.armed.is_none());
    }

    // ---- place ----

    #[test]
    fn place_with_nothing_armed_does_nothing() {
        let mut e = IconEditor::new();
        assert_eq!(e.place(5.0, 5.0, 48, 32), None);
        assert!(e.icons.is_empty());
    }

    #[test]
    fn place_stamps_selects_and_honours_the_armed_scale() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 2.5, 0.0, 0.0);
        let idx = e.place(5.0, 5.0, 48, 32).expect("placed");
        assert_eq!(idx, 0);
        assert_eq!(e.selected(), Some(0));
        let ic = &e.icons[0];
        assert_eq!((ic.x, ic.y), (5.0, 5.0));
        assert_eq!(ic.family, ManualIconFamily::Feature);
        assert_eq!(ic.slot, "mountain");
        assert_eq!(ic.scale, 2.5, "arm-time scale must override place_manual_icon's own 1.0");
    }

    #[test]
    fn place_off_grid_fails_without_disturbing_the_armed_selection() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        assert_eq!(e.place(-1.0, 5.0, 48, 32), None);
        assert!(e.icons.is_empty());
        assert!(e.armed.is_some());
    }

    // ---- hit_test ----

    #[test]
    fn hit_test_finds_and_selects_a_placed_icon() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(5.0, 5.0, 48, 32);
        let hit = e.hit_test(5.5, 5.5, &env(), SelectMode::Replace);
        assert_eq!(hit, Some(0));
        assert_eq!(e.selected(), Some(0));
    }

    #[test]
    fn deselect_clears_the_selection_and_the_resize_baseline() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(5.0, 5.0, 48, 32);
        assert_eq!(e.hit_test(5.5, 5.5, &env(), SelectMode::Replace), Some(0));
        assert_eq!(e.selected(), Some(0));
        assert!(e.resize_base_scale.is_some());
        e.deselect();
        assert_eq!(e.selected(), None);
        // The baseline goes with it: a drag after a fresh select must measure
        // from THAT icon's scale, never the previous selection's.
        assert!(e.resize_base_scale.is_none());
        // Idempotent -- Deselect with nothing selected is a legal no-op.
        e.deselect();
        assert_eq!(e.selected(), None);
        // And nothing was deleted.
        assert_eq!(e.icons.len(), 1);
    }

    #[test]
    fn hit_test_miss_leaves_selection_unchanged() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(5.0, 5.0, 48, 32);
        e.deselect();
        assert_eq!(e.hit_test(500.0, 500.0, &env(), SelectMode::Replace), None);
        assert_eq!(e.selected(), None);
    }

    // ---- resize ----

    #[test]
    fn resize_requires_the_target_to_already_be_selected() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(20.0, 10.0, 48, 32);
        e.deselect();
        assert!(!e.resize(0, 20.0, 10.0, 60.0, 60.0, 3.0));
        assert_eq!(e.icons[0].scale, 1.0);
    }

    #[test]
    fn resize_scales_from_the_selection_time_snapshot_not_the_live_value() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(10.0, 10.0, 48, 32); // scale 1.0, selected, base snapshot = 1.0
        // Two calls with the SAME start_dist, as a real drag would send --
        // each must compute fresh off the base of 1.0, not compound.
        assert!(e.resize(0, 10.0, 10.0, 10.0, 10.0, 5.0)); // dist floor -> min clamp
        let after_first = e.icons[0].scale;
        assert!(e.resize(0, 10.0, 10.0, 10.0, 10.0, 5.0));
        assert_eq!(e.icons[0].scale, after_first, "same inputs must give the same result, not compound");
    }

    #[test]
    fn resize_rejects_an_out_of_range_index() {
        let mut e = IconEditor::new();
        assert!(!e.resize(0, 0.0, 0.0, 0.0, 0.0, 1.0));
    }

    // ---- handles / icon_handle ----

    #[test]
    fn icon_handle_matches_the_reference_formula() {
        let ic = ManualIcon { x: 10.0, y: 8.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 1.0 };
        // grid_w=2048 -> sc/lsc base = 4; zoom_scale=1 -> civ_zoom_k=1.
        let env = IconViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let box_ = icon_box(&ic, &env); // px=10.5, py=8.5, r=20, side=52
        let h = icon_handle(&box_, &env);
        assert!((h.x - 28.7).abs() < 1e-9, "hx = px + side/2*0.7 = 10.5 + 18.2");
        assert!((h.y - 26.7).abs() < 1e-9, "hy = py + side/2*0.7 = 8.5 + 18.2");
        assert!((h.r - 20.48).abs() < 1e-9, "hr=max(4,3.2*4)=12.8, stored r = hr*1.6");
    }

    #[test]
    fn icon_handle_follows_the_boxs_own_per_instance_scale() {
        let small = ManualIcon { x: 0.0, y: 0.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 1.0 };
        let big = ManualIcon { x: 0.0, y: 0.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 2.5 };
        let env = IconViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let h_small = icon_handle(&icon_box(&small, &env), &env);
        let h_big = icon_handle(&icon_box(&big, &env), &env);
        // A bigger box pushes the handle further from the icon's own centre,
        // but the handle's own radius (a fixed on-screen affordance size,
        // not sprite-relative) is unchanged -- exactly `hr`'s own formula,
        // which depends only on `lsc`, never on the box.
        assert!(h_big.x > h_small.x);
        assert!(h_big.y > h_small.y);
        assert!((h_big.r - h_small.r).abs() < 1e-9);
    }

    #[test]
    fn icon_handle_never_shrinks_below_its_own_floor_at_low_zoom() {
        // zoom_scale pushed far past civ_zoom_k's own [0.35,5] clamp so lsc
        // collapses toward its minimum and the max(4,...) floor takes over
        // -- same fixture shape `label_bridge::handle_circles`' own
        // low-zoom-floor test uses.
        let ic = ManualIcon { x: 0.0, y: 0.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 1.0 };
        let env = IconViewEnv { grid_w: 512, zoom_scale: 1000.0, icon_scale: 1.0 };
        let h = icon_handle(&icon_box(&ic, &env), &env);
        assert!((h.r - 6.4).abs() < 1e-9, "hr floors at 4, stored r = 4*1.6");
    }

    #[test]
    fn editor_handles_matches_the_selected_icons_box() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(10.0, 8.0, 2048, 2048);
        let env = IconViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let h = e.handles(0, &env).expect("placed icon has a handle");
        assert!((h.x - 28.7).abs() < 1e-9);
        assert!((h.y - 26.7).abs() < 1e-9);
    }

    #[test]
    fn editor_handles_does_not_require_the_index_to_be_selected() {
        // Mirrors `label_bridge::LabelBridge::handles`' own contract: any
        // valid index works, not only the current selection.
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32); // index 0
        e.place(2.0, 2.0, 48, 32); // index 1, now selected
        assert_eq!(e.selected(), Some(1));
        assert!(e.handles(0, &env()).is_some(), "index 0 is not selected but is still valid");
    }

    #[test]
    fn editor_handles_out_of_range_is_none() {
        let e = IconEditor::new();
        assert!(e.handles(0, &env()).is_none());
    }

    // ---- delete ----

    #[test]
    fn delete_removes_and_clears_selection_when_it_pointed_at_the_removed_icon() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32);
        assert!(e.delete(0));
        assert!(e.icons.is_empty());
        assert_eq!(e.selected(), None);
    }

    #[test]
    fn delete_shifts_a_selection_pointing_past_the_removed_index() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32); // index 0
        e.place(2.0, 2.0, 48, 32); // index 1, now selected
        assert_eq!(e.selected(), Some(1));
        assert!(e.delete(0));
        assert_eq!(e.selected(), Some(0), "the icon formerly at 1 is now at 0");
        assert_eq!(e.icons.len(), 1);
    }

    #[test]
    fn delete_out_of_range_fails() {
        let mut e = IconEditor::new();
        assert!(!e.delete(0));
    }

    // ---- the selection set (step one of the selection-sets ruling) ----

    /// Three placed icons, far enough apart that each has its own box.
    fn three_icons() -> IconEditor {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        for k in 0..3 {
            e.place(2.0 + 8.0 * k as f64, 2.0, 48, 32);
        }
        e
    }

    #[test]
    fn a_plain_hit_replaces_the_selection_the_way_it_always_did() {
        let mut e = three_icons();
        assert_eq!(e.selected(), Some(2), "place selects what it placed");
        assert_eq!(e.hit_test(2.5, 2.5, &env(), SelectMode::Replace), Some(0));
        assert_eq!(e.selection.sorted(), vec![0]);
        assert_eq!(e.selected(), Some(0));
    }

    #[test]
    fn a_toggle_hit_adds_a_second_icon_and_makes_it_primary() {
        let mut e = three_icons();
        e.hit_test(2.5, 2.5, &env(), SelectMode::Replace);
        assert_eq!(e.hit_test(10.5, 2.5, &env(), SelectMode::Toggle), Some(1));
        assert_eq!(e.selection.sorted(), vec![0, 1]);
        assert_eq!(e.selected(), Some(1), "the just-toggled icon owns the handle");
    }

    #[test]
    fn a_toggle_hit_on_a_selected_icon_removes_it_and_promotes_the_one_before() {
        let mut e = three_icons();
        e.hit_test(2.5, 2.5, &env(), SelectMode::Replace);
        e.hit_test(10.5, 2.5, &env(), SelectMode::Toggle);
        assert_eq!(e.hit_test(10.5, 2.5, &env(), SelectMode::Toggle), Some(1));
        assert_eq!(e.selection.sorted(), vec![0]);
        assert_eq!(e.selected(), Some(0));
    }

    #[test]
    fn an_extend_hit_covers_the_range_from_the_anchor() {
        let mut e = three_icons();
        e.hit_test(2.5, 2.5, &env(), SelectMode::Replace);
        assert_eq!(e.hit_test(18.5, 2.5, &env(), SelectMode::Extend), Some(2));
        assert_eq!(e.selection.sorted(), vec![0, 1, 2]);
        assert_eq!(e.selected(), Some(2));
    }

    #[test]
    fn a_miss_leaves_a_multi_selection_alone_in_every_mode() {
        for mode in [SelectMode::Replace, SelectMode::Toggle, SelectMode::Extend] {
            let mut e = three_icons();
            e.select_set([0, 1]);
            assert_eq!(e.hit_test(500.0, 500.0, &env(), mode), None);
            assert_eq!(e.selection.sorted(), vec![0, 1], "mode {mode:?} lost the selection on a miss");
        }
    }

    #[test]
    fn the_resize_baseline_follows_the_primary_not_the_clicked_index() {
        // Toggling the primary back off must not leave the baseline pointing
        // at the icon that just left the set: the next drag on the promoted
        // one would measure from the wrong scale.
        let mut e = three_icons();
        e.icons[0].scale = 3.0;
        e.icons[1].scale = 0.5;
        e.hit_test(2.5, 2.5, &env(), SelectMode::Replace); // primary 0, base 3.0
        e.hit_test(10.5, 2.5, &env(), SelectMode::Toggle); // primary 1, base 0.5
        assert_eq!(e.resize_base_scale, Some(0.5));
        e.hit_test(10.5, 2.5, &env(), SelectMode::Toggle); // 1 off again, primary 0
        assert_eq!(e.selected(), Some(0));
        assert_eq!(e.resize_base_scale, Some(3.0), "the baseline was re-snapshotted from icon 0");
    }

    #[test]
    fn resize_still_refuses_an_index_that_is_not_the_primary() {
        let mut e = three_icons();
        e.select_set([0, 1]); // primary is 1 (set_from is ascending)
        assert!(!e.resize(0, 2.0, 2.0, 9.0, 9.0, 3.0), "0 is selected but is not the primary");
        assert_eq!(e.icons[0].scale, 1.0);
    }

    #[test]
    fn select_set_drops_an_out_of_range_index_and_reports_it() {
        let mut e = three_icons();
        assert!(!e.select_set([1, 99]));
        assert_eq!(e.selection.sorted(), vec![1]);
        assert!(e.select_set([0, 2]));
        assert_eq!(e.selection.sorted(), vec![0, 2]);
    }

    #[test]
    fn select_set_snapshots_the_new_primarys_scale() {
        let mut e = three_icons();
        e.icons[2].scale = 4.0;
        e.select_set([0, 2]);
        assert_eq!(e.selected(), Some(2));
        assert_eq!(e.resize_base_scale, Some(4.0));
    }

    #[test]
    fn deleting_a_member_of_a_multi_selection_shifts_the_rest() {
        let mut e = three_icons();
        e.select_set([0, 1, 2]);
        assert!(e.delete(0));
        assert_eq!(e.selection.sorted(), vec![0, 1], "1 and 2 became 0 and 1");
        assert_eq!(e.selected(), Some(1));
    }

    #[test]
    fn deselect_and_clear_all_empty_the_whole_set() {
        let mut e = three_icons();
        e.select_set([0, 1, 2]);
        e.deselect();
        assert!(e.selection.sorted().is_empty());
        assert_eq!(e.selected(), None);
        assert!(e.resize_base_scale.is_none());

        let mut f = three_icons();
        f.select_set([0, 1]);
        f.clear_all();
        assert!(f.selection.sorted().is_empty());
        assert!(f.icons.is_empty());
    }

    // ---- clear_all ----

    // ---- the generated placement pass ----

    /// A tiny grid whose left `land_cols` columns are land: the shore runs
    /// down column `land_cols`, and every cell of it is a coast cell.
    fn shore(w: usize, h: usize, land_cols: usize) -> Vec<f32> {
        (0..w * h).map(|i| if i % w < land_cols { 1.0 } else { 0.0 }).collect()
    }

    fn cand(x: f64, y: f64, slot: &str) -> IconCandidate {
        IconCandidate { x, y, slot: slot.to_string() }
    }

    fn plan(family: PlacementFamily, min_spacing: f64) -> IconGenPlan {
        IconGenPlan {
            family,
            scale: 1.0,
            min_spacing,
            avoid_labels: false,
            enforce_spacing: min_spacing > 0.0,
        }
    }

    #[test]
    fn placement_families_are_four_real_families_not_three_and_a_name() {
        // The ruling, as an assertion: every design chip resolves to an asset
        // family with its own non-empty vocabulary, and no two chips share one.
        let mut seen: Vec<ManualIconFamily> = Vec::new();
        for f in PlacementFamily::ALL {
            assert!(!f.slots().is_empty(), "{} has no slots", f.key());
            assert!(!seen.contains(&f.icon_family()), "{} duplicates another family", f.key());
            seen.push(f.icon_family());
            assert_eq!(PlacementFamily::from_key(f.key()), Some(f));
        }
        assert_eq!(PlacementFamily::SeaMarks.icon_family(), ManualIconFamily::SeaMark);
        assert_eq!(PlacementFamily::SeaMarks.slots().len(), 8);
        // TREES is the trees, not the whole `icons` art family.
        assert_eq!(PlacementFamily::Trees.slots(), &TREE_SLOTS);
        assert!(!PlacementFamily::Trees.slots().contains(&"mountain"));
        assert_eq!(PlacementFamily::from_key("SEA MARKS"), Some(PlacementFamily::SeaMarks));
        assert_eq!(PlacementFamily::from_key("seamarks"), None);
    }

    /// **The rule's own test** — owner ruling 2026-09-02: *"a sea mark that
    /// does not land on a coast is the entire failure mode of this feature."*
    #[test]
    fn every_snapped_sea_mark_lands_on_a_coast_cell() {
        let (w, h) = (20usize, 60usize);
        let f = shore(w, h, 8);
        let mut e = IconEditor::new();
        let env = IconViewEnv { grid_w: w, zoom_scale: 1.0, icon_scale: 1.0 };
        // Candidates deliberately scattered anywhere BUT the shore: deep
        // inland, far offshore, and on both map edges.
        let cands: Vec<IconCandidate> = (0..h)
            .step_by(20)
            .flat_map(|y| {
                [0.0, 3.0, 15.0, 19.0]
                    .into_iter()
                    .map(move |x| cand(x, y as f64, "lighthouse"))
            })
            .collect();
        assert_eq!(cands.len(), 12);
        let snap = CoastSnap { field: &f, gw: w, gh: h, sea: 0.5, max_r: 20 };
        let r = e.generate(&plan(PlacementFamily::SeaMarks, 0.0), &cands, &env, &[], Some(&snap));
        assert_eq!(r.placed, 3, "one per candidate row; the other three collapse onto it");
        assert_eq!(r.off_coast, 0, "every candidate had a shore within reach");
        for ic in &e.icons {
            assert!(
                cartalith_assets::coast::is_coast(&f, w, h, 0.5, ic.x as i64, ic.y as i64),
                "({}, {}) is not a coast cell",
                ic.x,
                ic.y
            );
            assert_eq!(ic.family, ManualIconFamily::SeaMark);
        }
        // Every one of them moved -- none of the candidates started on a coast.
        assert_eq!(r.snapped, r.placed);
    }

    #[test]
    fn a_sea_mark_with_no_coast_in_reach_is_dropped_not_placed_inland() {
        // All land: there is no coast anywhere, so no sea mark may be placed.
        let (w, h) = (12usize, 6usize);
        let f = vec![1.0f32; w * h];
        let mut e = IconEditor::new();
        let env = IconViewEnv { grid_w: w, zoom_scale: 1.0, icon_scale: 1.0 };
        let snap = CoastSnap { field: &f, gw: w, gh: h, sea: 0.5, max_r: 4 };
        let r = e.generate(
            &plan(PlacementFamily::SeaMarks, 0.0),
            &[cand(3.0, 3.0, "buoy"), cand(8.0, 2.0, "reef")],
            &env,
            &[],
            Some(&snap),
        );
        assert_eq!((r.placed, r.off_coast), (0, 2));
        assert!(e.icons.is_empty(), "a sea mark with nowhere to go is dropped, not stranded inland");
    }

    #[test]
    fn the_snap_is_only_the_sea_marks_rule() {
        // Same coast data, a different family: settlements are not dragged to
        // the beach. The design's rule names sea marks and nothing else.
        let (w, h) = (20usize, 6usize);
        let f = shore(w, h, 8);
        let mut e = IconEditor::new();
        let env = IconViewEnv { grid_w: w, zoom_scale: 1.0, icon_scale: 1.0 };
        let snap = CoastSnap { field: &f, gw: w, gh: h, sea: 0.5, max_r: 12 };
        let r = e.generate(
            &plan(PlacementFamily::Places, 0.0),
            &[cand(2.0, 3.0, "hamlet")],
            &env,
            &[],
            Some(&snap),
        );
        assert_eq!((r.placed, r.snapped), (1, 0));
        assert_eq!((e.icons[0].x, e.icons[0].y), (2.0, 3.0));
    }

    #[test]
    fn a_candidate_naming_another_familys_slot_is_counted_not_placed() {
        let mut e = IconEditor::new();
        let r = e.generate(
            &plan(PlacementFamily::SeaMarks, 0.0),
            &[cand(1.0, 1.0, "mountain"), cand(3.0, 1.0, "lighthouse")],
            &env(),
            &[],
            None,
        );
        assert_eq!((r.placed, r.unknown_slot), (1, 1));
        assert_eq!(e.icons[0].slot, "lighthouse");
    }

    #[test]
    fn min_spacing_thins_the_run_to_the_sliders_own_figure() {
        // grid_w 512 -> sc 1, so a scale-1 footprint is side 13 cells. Spacing
        // of 40 is the design slider's own top end (`parts.js:392`, `p*40`).
        let env = IconViewEnv { grid_w: 512, zoom_scale: 1.0, icon_scale: 1.0 };
        let mut e = IconEditor::new();
        // Five candidates 20 cells apart: at spacing 40 only every other one
        // survives; the geometry, not a magic number.
        let cands: Vec<IconCandidate> =
            (0..5).map(|i| cand(i as f64 * 20.0, 50.0, "hamlet")).collect();
        let r = e.generate(&plan(PlacementFamily::Places, 40.0), &cands, &env, &[], None);
        assert_eq!(r.placed, 3, "0, 40 and 80 survive; 20 and 60 are inside 40 cells of a placed one");
        assert_eq!(r.culled_spacing, 2);
        let xs: Vec<f64> = e.icons.iter().map(|ic| ic.x).collect();
        assert_eq!(xs, vec![0.0, 40.0, 80.0]);
    }

    #[test]
    fn with_the_spacing_rule_off_icons_still_do_not_stack_on_one_cell() {
        // The footprint floor under the slider: `enforce_spacing == false` is
        // not a licence to place four icons on the same cell.
        let mut e = IconEditor::new();
        let cands = vec![cand(4.0, 4.0, "ruin"), cand(4.0, 4.0, "cave"), cand(4.0, 4.0, "shrine")];
        let r = e.generate(&plan(PlacementFamily::Poi, 0.0), &cands, &env(), &[], None);
        assert_eq!((r.placed, r.culled_spacing), (1, 2));
    }

    #[test]
    fn a_second_run_over_the_same_world_places_nothing() {
        // Why there is no `generated` list: the pass is idempotent because its
        // own output is an obstacle to its own candidates.
        let mut e = IconEditor::new();
        let cands: Vec<IconCandidate> =
            (0..4).map(|i| cand(i as f64 * 30.0, 10.0, "hamlet")).collect();
        let p = plan(PlacementFamily::Places, 8.0);
        let first = e.generate(&p, &cands, &env(), &[], None);
        assert_eq!(first.placed, 4);
        let second = e.generate(&p, &cands, &env(), &[], None);
        assert_eq!((second.placed, second.culled_spacing), (0, 4));
        assert_eq!(e.icons.len(), 4, "a re-run adds nothing rather than doubling the map");
    }

    #[test]
    fn hand_placed_icons_are_obstacles_the_pass_respects() {
        let mut e = IconEditor::new();
        e.arm("settlement", 0, 1.0, 0.0, 0.0);
        e.place(10.0, 10.0, 512, 512);
        let r = e.generate(
            &plan(PlacementFamily::Places, 20.0),
            &[cand(11.0, 10.0, "village"), cand(60.0, 10.0, "town")],
            &env(),
            &[],
            None,
        );
        assert_eq!((r.placed, r.culled_spacing), (1, 1));
        assert_eq!(e.icons.len(), 2);
        // And the hand-placed one is untouched, still at index 0.
        assert_eq!((e.icons[0].x, e.icons[0].y), (10.0, 10.0));
    }

    #[test]
    fn avoid_label_boxes_suppresses_against_the_shipped_culler_and_nothing_else() {
        // `reserved` is measured by the caller with `label_cull_rect`; the pass
        // only compares. A rect over (5,5) big enough to swallow an icon there.
        let over = LabelRect { cx: 5.0, cy: 5.0, w: 30.0, h: 30.0 };
        let mut e = IconEditor::new();
        let mut p = plan(PlacementFamily::Poi, 0.0);
        p.avoid_labels = true;
        let r = e.generate(&p, &[cand(5.0, 5.0, "ruin"), cand(40.0, 40.0, "cave")], &env(), &[over], None);
        assert_eq!((r.placed, r.culled_label), (1, 1));
        assert_eq!(e.icons[0].slot, "cave");
        // With the rule off, the same reservation suppresses nothing.
        let mut e2 = IconEditor::new();
        let r2 = e2.generate(
            &plan(PlacementFamily::Poi, 0.0),
            &[cand(5.0, 5.0, "ruin")],
            &env(),
            &[over],
            None,
        );
        assert_eq!((r2.placed, r2.culled_label), (1, 0));
    }

    #[test]
    fn a_reserved_rect_is_never_placed_and_never_returned() {
        // `generate_labels`' own contract for the same argument, asserted here
        // too: reservations are obstacles, not content.
        let over = LabelRect { cx: 5.0, cy: 5.0, w: 30.0, h: 30.0 };
        let mut e = IconEditor::new();
        let mut p = plan(PlacementFamily::Poi, 0.0);
        p.avoid_labels = true;
        let r = e.generate(&p, &[cand(5.0, 5.0, "ruin")], &env(), &[over], None);
        assert_eq!(r.placed, 0);
        assert!(e.icons.is_empty());
    }

    #[test]
    fn a_non_finite_reservation_suppresses_nothing() {
        // The culler's own NaN rule, inherited rather than re-implemented:
        // `<` is false on NaN, so an unmeasurable box fails towards drawing.
        let bad = LabelRect { cx: f64::NAN, cy: 5.0, w: 30.0, h: 30.0 };
        let mut e = IconEditor::new();
        let mut p = plan(PlacementFamily::Poi, 0.0);
        p.avoid_labels = true;
        let r = e.generate(&p, &[cand(5.0, 5.0, "ruin")], &env(), &[bad], None);
        assert_eq!((r.placed, r.culled_label), (1, 0));
    }

    #[test]
    fn the_generated_scale_is_clamped_like_the_armed_one() {
        let mut e = IconEditor::new();
        let mut p = plan(PlacementFamily::Places, 0.0);
        p.scale = 999.0;
        e.generate(&p, &[cand(1.0, 1.0, "hamlet")], &env(), &[], None);
        assert_eq!(e.icons[0].scale, ICON_SCALE_MAX);
        let mut e2 = IconEditor::new();
        let mut p2 = plan(PlacementFamily::Places, 0.0);
        p2.scale = f64::NAN;
        e2.generate(&p2, &[cand(1.0, 1.0, "hamlet")], &env(), &[], None);
        assert_eq!(e2.icons[0].scale, 1.0);
    }

    #[test]
    fn default_snap_radius_floors_at_eight_and_scales_with_the_grid() {
        assert_eq!(default_snap_radius(64), 8);
        assert_eq!(default_snap_radius(512), 16);
        assert_eq!(default_snap_radius(2048), 64);
    }

    #[test]
    fn clear_all_drops_placements_and_selection_but_not_the_armed_chip() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32);
        e.place(2.0, 2.0, 48, 32);
        e.clear_all();
        assert!(e.icons.is_empty());
        assert_eq!(e.selected(), None);
        assert!(e.armed.is_some(), "Clear-all clears placements, not the armed gallery selection");
    }

    // ---- the density brush ----

    /// An armed editor with the brush on, over `shore()`'s own land/sea split.
    fn brushed() -> IconEditor {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.set_brush(true, 8.0, 0.6);
        e
    }

    #[test]
    fn a_fresh_editors_brush_is_the_references_own_defaults_and_off() {
        let e = IconEditor::new();
        assert!(!e.brush.on);
        assert_eq!(e.brush.r, 12.0);
        assert_eq!(e.brush.density, 0.6);
    }

    #[test]
    fn set_brush_clamps_both_numbers_to_the_reference_sliders_own_range() {
        let mut e = IconEditor::new();
        assert!(e.set_brush(true, 1000.0, 1000.0));
        assert_eq!(e.brush.r, ICON_BRUSH_R_MAX);
        assert_eq!(e.brush.density, ICON_BRUSH_DENSITY_MAX);
        assert!(e.set_brush(true, -5.0, 0.0));
        assert_eq!(e.brush.r, ICON_BRUSH_R_MIN);
        assert_eq!(e.brush.density, ICON_BRUSH_DENSITY_MIN);
    }

    /// Pins the four bounds to the **reference's own slider attributes**.
    ///
    /// The clamp test above cannot: `assert_eq!(e.brush.r, ICON_BRUSH_R_MAX)`
    /// compares the constant against itself and holds for every value of it.
    /// Measured — all four survived mutation (`R_MAX 60 -> 47`,
    /// `DENSITY_MIN 0.05 -> 0.01`, `R_MIN 2 -> 5`, and the seed) with the whole
    /// suite green, which is the same self-referential shape that let
    /// `MIN_REGION_WORLD_AXIS` survive `4 -> 3`.
    ///
    /// Literals, and the reference lines they come from:
    /// `#carIconBrushR` at 1656 is `min="2" max="60"`; `#carIconBrushD` at 1657
    /// is `min="5" max="200"`, divided by 100 by the `input` listener at 13515,
    /// so `0.05 ..= 2.00`. A change to any of these is a deliberate divergence
    /// from the reference's authored range and should fail here first.
    #[test]
    fn the_brush_bounds_are_the_reference_sliders_literal_attributes() {
        assert_eq!(ICON_BRUSH_R_MIN, 2.0, "#carIconBrushR min, reference 1656");
        assert_eq!(ICON_BRUSH_R_MAX, 60.0, "#carIconBrushR max, reference 1656");
        assert_eq!(ICON_BRUSH_DENSITY_MIN, 0.05, "#carIconBrushD min 5/100, reference 1657+13515");
        assert_eq!(ICON_BRUSH_DENSITY_MAX, 2.0, "#carIconBrushD max 200/100, reference 1657+13515");
        // The default the reference ships sits inside the range it authored.
        let d = IconBrush::default();
        assert!(
            (ICON_BRUSH_R_MIN..=ICON_BRUSH_R_MAX).contains(&d.r),
            "the default radius {} is outside the slider's own range",
            d.r
        );
    }

    /// The brush stream is seeded, not clock-driven, and the seed is fixed.
    ///
    /// Two independent editors must throw the **same** first stroke — that is
    /// what makes the brush testable at all — while a second stroke from one
    /// editor differs from its first, which is the reference's stated
    /// requirement (re-painting a spot adds new icons rather than reproducing
    /// the previous ones). Mutating `BRUSH_SEED` survived the suite before
    /// this test existed.
    #[test]
    fn the_brush_seed_is_fixed_and_the_stream_advances() {
        // The literal, first. Two fresh editors agreeing proves only that they
        // share *a* seed -- which is true of every value, and is why mutating
        // BRUSH_SEED survived this test's first version.
        assert_eq!(BRUSH_SEED, 0x_CA12_1C04, "the brush stream's chosen seed");
        let (mut a, mut b) = (IconEditor::new(), IconEditor::new());
        let first_a: Vec<f64> = (0..4).map(|_| a.brush_rng.next_f64()).collect();
        let first_b: Vec<f64> = (0..4).map(|_| b.brush_rng.next_f64()).collect();
        assert_eq!(first_a, first_b, "two fresh editors must start from one seed");
        let second_a: Vec<f64> = (0..4).map(|_| a.brush_rng.next_f64()).collect();
        assert_ne!(
            first_a, second_a,
            "the stream must advance, or a second stroke would repeat the first"
        );
    }

    #[test]
    fn set_brush_rejects_a_non_finite_number_but_still_applies_on() {
        let mut e = IconEditor::new();
        e.set_brush(true, 20.0, 0.9);
        assert!(!e.set_brush(false, f64::NAN, 0.9), "a NaN radius is a rejected call");
        assert!(!e.brush.on, "...and turning the brush off is not refusable by it");
        assert_eq!(e.brush.r, 20.0, "the previous radius survives the rejection");
        assert_eq!(e.brush.density, 0.9);
    }

    #[test]
    fn a_brush_stamp_paints_a_stand_of_icons_on_land() {
        let mut e = brushed();
        let field = shore(48, 32, 40);
        let n = e.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 10.0, 16.0);
        assert!(n >= 2, "a radius-8 stamp well inside land should place a stand, placed {n}");
        assert_eq!(e.icons.len(), n);
        for ic in &e.icons {
            assert!(field[ic.y as usize * 48 + ic.x as usize] > 0.5, "every dart is on land");
            assert_eq!(ic.slot, "mountain", "every dart takes the armed slot");
            assert!(
                (0.7..=1.2).contains(&ic.scale),
                "size comes from the rule's own min/max, not the armed scale: {}",
                ic.scale
            );
        }
    }

    #[test]
    fn re_stamping_the_same_spot_throws_new_darts_rather_than_repeating_the_first_run() {
        // The reference's own reason for using `Math.random` here, asserted
        // rather than assumed -- `BRUSH_SEED`'s doc comment.
        //
        // **The map is cleared between the two stamps, and that is what gives
        // this test teeth.** Stamping twice onto the *same* map would pass
        // even against a stream re-seeded per stamp, because the first run's
        // own icons are then in `near` and the blue-noise culler rejects the
        // repeats for a completely different reason -- which is exactly what
        // a re-seed mutant proved before this was rewritten. `clear_all`
        // deliberately does not reset `brush_rng`, so after it the second
        // stamp runs over an identical map and can differ only if the stream
        // advanced.
        let field = shore(48, 32, 40);
        let mut e = brushed();
        e.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 10.0, 16.0);
        let first: Vec<(f64, f64)> = e.icons.iter().map(|i| (i.x, i.y)).collect();
        assert!(!first.is_empty(), "the first stamp placed nothing, so this would pin nothing");
        e.clear_all();
        e.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 10.0, 16.0);
        let second: Vec<(f64, f64)> = e.icons.iter().map(|i| (i.x, i.y)).collect();
        assert_ne!(first, second, "the second stamp threw the same darts as the first");
    }

    #[test]
    fn a_stamp_in_open_water_places_nothing() {
        // The gate the click path deliberately does not have.
        let mut e = brushed();
        let field = shore(48, 32, 4);
        assert_eq!(e.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 40.0, 16.0), 0);
        assert!(e.icons.is_empty());
    }

    #[test]
    fn the_brush_places_nothing_while_it_is_switched_off_or_nothing_is_armed() {
        let field = shore(48, 32, 40);
        let mut off = brushed();
        off.set_brush(false, 8.0, 0.6);
        assert_eq!(off.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 10.0, 16.0), 0);

        let mut unarmed = IconEditor::new();
        unarmed.set_brush(true, 8.0, 0.6);
        assert_eq!(unarmed.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 10.0, 16.0), 0);
    }

    #[test]
    fn a_brush_stamp_appends_and_leaves_an_existing_selection_pointing_at_the_same_icon() {
        let mut e = brushed();
        let field = shore(48, 32, 40);
        e.place(1.0, 1.0, 48, 32);
        assert_eq!(e.selected(), Some(0));
        let before = e.icons[0].clone();
        e.brush_stamp(&ScatterRule::default(), &field, 48, 32, 0.5, 10.0, 16.0);
        assert_eq!(e.selected(), Some(0), "brushing does not move the selection");
        assert_eq!(e.icons[0], before, "and does not disturb the icon it points at");
    }
}
