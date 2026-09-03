//! Hand-placed map icons — `UNIFIED_TOOL_PLAN.md` milestone E, the Icon stamp
//! tool's engine half.
//!
//! Sits beside [`crate::placement`]'s rule-driven auto-placement rather than
//! replacing it, which is exactly the plan's framing: the rule half shipped in
//! Asset Library milestones 3-4 (`place_map_icons_ruled`, the `ScatterRule`
//! system, `icon_slot_for_item`, `sprite_draw_rect`) and the manual half did
//! not. `cartalith-assets` is where it belongs for the same reason: a manual
//! icon addresses the identical slot/variant vocabulary
//! ([`crate::slots::Family`], [`crate::pick_icon_variant`]) that the scatterer
//! does, and [`icon_brush_rule`] reads the very same [`ScatterRule`] table.
//!
//! # The plan was wrong about what the manual half *is*
//!
//! `UNIFIED_TOOL_PLAN.md` describes `_carIconBrushStamp` (reference line
//! 15051) as *"stamp mode (place one icon by hand at a clicked point)"*.
//! Reading it, that is not what it does. There are **three** placement paths
//! in the reference, not two:
//!
//! 1. **Rule-driven autoplacement** — `placeMapIconsRuled`. Already ported.
//! 2. **Click-to-place a single icon** — the `_iconPlaceMode` branch of the
//!    click handler (reference lines 9776-9784). This *is* the "place one icon
//!    by hand" path, it is four lines, and it is [`place_manual_icon`] below.
//! 3. **A dart-throwing scatter *brush*** — `_carIconBrushStamp`, which paints
//!    a blue-noise *stand* of icons under a radius as the pointer drags. This
//!    is by far the larger of the two manual paths and the plan does not
//!    describe it at all.
//!
//! # The brush is deliberately non-deterministic, and that shapes its port
//!
//! The reference's own comment: *"Unlike the procedural scatterer this uses
//! `Math.random`, not `hash()`: a brush stroke is an authoring ACTION whose
//! result is persisted in `state.mapIcons` — re-painting the same spot should
//! add new icons, not deterministically reproduce the previous ones."*
//!
//! So [`icon_brush_stamp`] takes its randomness as a parameter
//! (`&mut dyn FnMut() -> f64` yielding `[0, 1)`) rather than owning an RNG.
//! That is not a testing convenience bolted on: it is the only way to golden-
//! verify the function at all — the harness overrode `Math.random` with a
//! seeded LCG inside the `vm` context and this port drives the identical
//! stream — *and* it keeps the reference's "a re-paint adds more" semantics
//! available to a shell that wants a real `rand` source.
//!
//! # No `PassBuffer`
//!
//! Milestone D's finding repeats: placing an icon is a discrete, already-atomic
//! append, and a brush stroke's own `state.mapIcons` is its staging. The
//! reference commits each dart immediately and relies on undo, not on a draft.

use crate::scatter::{scatter_rule_key, ScatterRule, ScatterRuleTable};

/// The `icon.fam` vocabulary — the reference's four manual-icon families.
///
/// Deliberately **not** [`crate::slots::Family`], which is the *pack
/// directory* taxonomy. The two overlap but do not agree: a manual icon's
/// `'feature'` addresses `CIV_FEATURE_ICON_TYPES`, which lives under the pack's
/// `icons` family, so a shared enum would have to answer to two different
/// spellings of the same thing. [`ManualIconFamily::pack_family`] maps between
/// them in the one place that needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualIconFamily {
    /// `CIV_SETTLEMENT_CLASSES`.
    Settlement,
    /// `CIV_FEATURE_ICON_TYPES` — the pack's `icons` family.
    Feature,
    /// `CIV_POI_TYPES`.
    Poi,
    /// The port's own fourth family — [`crate::slots::PACK_SEAMARK_SLOTS`],
    /// added by owner ruling 2026-09-02 so the design's *placement* vocabulary
    /// (PLACES · TREES · SEA MARKS · POI) has four real families under it
    /// instead of three plus a name with nothing behind it.
    ///
    /// The reference has no `icon.fam === 'seamarks'`, so **nothing loaded from
    /// a reference save can ever be one** — [`from_key`](Self::from_key) still
    /// answers `None` for every string the reference writes, and this variant
    /// is only ever reached by something this port placed.
    SeaMark,
    /// A user-defined custom-set asset; carries a set name as well as a slot.
    Custom,
}

impl ManualIconFamily {
    /// The reference's `icon.fam` string.
    pub fn key(self) -> &'static str {
        match self {
            ManualIconFamily::Settlement => "settlement",
            ManualIconFamily::Feature => "feature",
            ManualIconFamily::Poi => "poi",
            // The same string `slots::Family::SeaMark` uses. Every other pair
            // here is a rename this port inherited; a new family gets one name.
            ManualIconFamily::SeaMark => "seamarks",
            ManualIconFamily::Custom => "custom",
        }
    }

    pub fn from_key(key: &str) -> Option<ManualIconFamily> {
        match key {
            "settlement" => Some(ManualIconFamily::Settlement),
            "feature" => Some(ManualIconFamily::Feature),
            "poi" => Some(ManualIconFamily::Poi),
            "seamarks" => Some(ManualIconFamily::SeaMark),
            "custom" => Some(ManualIconFamily::Custom),
            _ => None,
        }
    }

    /// The pack family this manual family draws its art from. Note the one
    /// rename: `feature` art lives under the pack's `icons`.
    pub fn pack_family(self) -> crate::slots::Family {
        match self {
            ManualIconFamily::Settlement => crate::slots::Family::Settlement,
            ManualIconFamily::Feature => crate::slots::Family::Icons,
            ManualIconFamily::Poi => crate::slots::Family::Poi,
            ManualIconFamily::SeaMark => crate::slots::Family::SeaMark,
            ManualIconFamily::Custom => crate::slots::Family::Custom,
        }
    }
}

/// The gallery selection that arms placement — `_carIconArmed`.
///
/// `set` is meaningful only for [`ManualIconFamily::Custom`]; the reference
/// carries it on every armed selection and reads it only there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmedIcon {
    pub family: ManualIconFamily,
    pub slot: String,
    pub set: Option<String>,
}

impl ArmedIcon {
    /// The [`ScatterRule`] key this selection addresses — the reference's
    /// `_carIconBrushRule` line 1. A custom asset keys by
    /// `custom::<set>::<slot>`; everything else keys by its bare slot.
    pub fn rule_key(&self) -> String {
        match self.family {
            ManualIconFamily::Custom => scatter_rule_key(&self.slot, self.set.as_deref()),
            _ => self.slot.clone(),
        }
    }
}

/// One hand-placed icon — the reference's `state.mapIcons[i]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIcon {
    pub x: f64,
    pub y: f64,
    pub family: ManualIconFamily,
    pub slot: String,
    /// `Some` only for [`ManualIconFamily::Custom`], matching the reference's
    /// two distinct object literals.
    pub set: Option<String>,
    /// Per-instance size multiplier. Click-placement always uses 1.0; the
    /// brush draws it from the rule's `min_size..max_size`.
    pub scale: f64,
}

/// `_carIconBrush` — the brush's own controls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconBrush {
    pub on: bool,
    /// Radius in grid cells.
    pub r: f64,
    /// `0..1`. Floored at [`ICON_BRUSH_MIN_DENSITY`] on use.
    pub density: f64,
}

impl Default for IconBrush {
    /// `{on: false, r: 12, density: 0.6}` (reference line 4746).
    fn default() -> Self {
        IconBrush { on: false, r: 12.0, density: 0.6 }
    }
}

/// The floor `_carIconBrushStamp` applies to `density`, so a zeroed slider
/// still paints rather than dividing by zero in the spacing formula.
pub const ICON_BRUSH_MIN_DENSITY: f64 = 0.02;
/// The minimum blue-noise separation in cells, whatever the density.
pub const ICON_BRUSH_MIN_SPACING: f64 = 1.2;
/// v1.27's per-stamp work cap. The reference's own note: attempts used to
/// scale with brush *area*, so *"radius 60 at max density asked for ~15k darts
/// per stamp — and a stamp runs on every pointermove... That drops frames on a
/// large brush. The cap bounds one stamp's work; a dense fill is still
/// reachable by dragging over the area again."*
pub const ICON_BRUSH_MAX_DARTS: usize = 1500;

// `Math.round` -- half **up**, not Rust's half-away-from-zero. Load-bearing
// here: a dart at `cx - R` can land on a negative coordinate, and
// `Math.round(-0.5)` is `0` where `(-0.5f64).round()` is `-1`, which would
// place an icon one cell further out than the reference does (and then have it
// rejected by the bounds test, changing the dart budget's outcome).
//
// V8's compensated `Math.hypot` alongside it, used by the hit test and the
// resize handle, both of which compare against a radius where one ULP decides
// a hit. Both from `cartalith-jsmath` now.
use cartalith_jsmath::{js_hypot, js_round};

/// `_carIconBrushRule()` (reference line 15046).
///
/// The armed selection's rule, or `defaultScatterRule()` when the table has
/// none for it. Returns `None` only when nothing is armed — the reference's
/// `if(!_carIconArmed) return null`, which its caller then treats as "place
/// nothing".
pub fn icon_brush_rule(armed: Option<&ArmedIcon>, rules: &ScatterRuleTable) -> Option<ScatterRule> {
    let a = armed?;
    Some(rules.get(&a.rule_key()).cloned().unwrap_or_default())
}

/// `_carIconBrushStamp(cx, cy)` (reference line 15051) — one brush stamp.
///
/// Dart-throwing with a blue-noise rejection radius, tested against **both**
/// the icons already on the map and the ones this stamp is placing, so
/// dragging slowly over one spot thickens the stand up to the spacing limit
/// rather than piling sprites on top of each other.
///
/// `rng` must yield `[0, 1)`, standing in for `Math.random` — see the module
/// docs on why this is a parameter. It is called **three** times per accepted
/// dart and twice per rejected one (angle, radius, then size only on
/// acceptance); the order matters for stream parity.
///
/// Appends to `icons` in place and returns how many were added.
// The reference reads all of these off module globals; making them parameters
// is what keeps this function pure and testable, so the count is the point
// rather than an accident.
#[allow(clippy::too_many_arguments)]
pub fn icon_brush_stamp(
    icons: &mut Vec<ManualIcon>,
    armed: Option<&ArmedIcon>,
    brush: &IconBrush,
    rule: &ScatterRule,
    field: &[f32],
    gw: usize,
    gh: usize,
    sea_level: f64,
    cx: f64,
    cy: f64,
    rng: &mut dyn FnMut() -> f64,
) -> usize {
    let Some(armed) = armed else { return 0 };
    let r = brush.r;
    let dens = f64::max(ICON_BRUSH_MIN_DENSITY, brush.density);
    // spacing shrinks as density rises; the floor keeps sprites from ever
    // fully overlapping
    let spacing = f64::max(ICON_BRUSH_MIN_SPACING, 3.0 / dens.sqrt());
    let sp2 = spacing * spacing;
    // 2x oversample for dart-throwing, capped
    let attempts = {
        let raw = ((std::f64::consts::PI * r * r) / (spacing * spacing)).ceil() * 2.0;
        let capped = f64::min(ICON_BRUSH_MAX_DARTS as f64, raw);
        f64::max(1.0, capped) as usize
    };
    // only nearby existing icons can conflict -- a cheap prefilter keeps this
    // O(nearby) rather than O(all icons) per stamp
    let rr = (r + spacing) * (r + spacing);
    let mut near: Vec<(f64, f64)> = icons
        .iter()
        .filter(|ic| {
            let (dx, dy) = (ic.x - cx, ic.y - cy);
            dx * dx + dy * dy <= rr
        })
        .map(|ic| (ic.x, ic.y))
        .collect();

    let mut placed = 0usize;
    for _ in 0..attempts {
        let ang = rng() * std::f64::consts::PI * 2.0;
        // sqrt => uniform over the disc, not centre-biased
        let rad = rng().sqrt() * r;
        let x = js_round(cx + ang.cos() * rad);
        let y = js_round(cy + ang.sin() * rad);
        if x < 0.0 || y < 0.0 || x >= gw as f64 || y >= gh as f64 {
            continue;
        }
        let idx = y as usize * gw + x as usize;
        if field[idx] as f64 <= sea_level {
            continue; // never paint into water
        }
        if near.iter().any(|&(nx, ny)| {
            let (dx, dy) = (nx - x, ny - y);
            dx * dx + dy * dy < sp2
        }) {
            continue;
        }
        let s = rule.min_size + (rule.max_size - rule.min_size) * rng();
        icons.push(ManualIcon {
            x,
            y,
            family: armed.family,
            slot: armed.slot.clone(),
            set: match armed.family {
                ManualIconFamily::Custom => armed.set.clone(),
                _ => None,
            },
            scale: s,
        });
        near.push((x, y));
        placed += 1;
    }
    placed
}

/// The click handler's single-icon placement (reference lines 9776-9784).
///
/// *"click empty land places the gallery-armed family/slot at full scale and
/// selects it. No prompt/confirm step, unlike labels."* Returns `None` when
/// the click is off-grid or nothing is armed.
///
/// Note what is **not** here: the reference's click path has no sea-level
/// gate, unlike the brush's. Ported as written — a hand-placed lighthouse or
/// buoy is a legitimate thing to want, and adding a gate the reference does
/// not have would refuse it.
pub fn place_manual_icon(gx: f64, gy: f64, gw: usize, gh: usize, armed: Option<&ArmedIcon>) -> Option<ManualIcon> {
    if gx < 0.0 || gx >= gw as f64 || gy < 0.0 || gy >= gh as f64 {
        return None;
    }
    let armed = armed?;
    Some(ManualIcon {
        x: gx,
        y: gy,
        family: armed.family,
        slot: armed.slot.clone(),
        set: match armed.family {
            ManualIconFamily::Custom => armed.set.clone(),
            _ => None,
        },
        scale: 1.0,
    })
}

/// The view state an icon's box depends on — the icon equivalent of
/// `cartalith_civ::labels::LabelViewEnv`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconViewEnv {
    pub grid_w: usize,
    /// `viewT.scale`, the raw zoom before [`civ_zoom_k`]'s clamp -- the same
    /// contract `cartalith_civ::labels::LabelViewEnv::zoom_scale` states.
    ///
    /// **This port's shell does not pass `viewT.scale`.**
    /// `cartography_workspace.gd` reaches `icon_handles` with
    /// `app.viewport.zoom()`, the deep camera zoom, whose ceiling is
    /// `max(64, width_km / ZOOM_TARGET_SPAN_KM)` and so is far past the
    /// clamp. See [`civ_zoom_k`].
    pub zoom_scale: f64,
    pub icon_scale: f64,
}

impl Default for IconViewEnv {
    fn default() -> Self {
        IconViewEnv { grid_w: 512, zoom_scale: 1.0, icon_scale: 1.0 }
    }
}

/// `_civZoomK()` — duplicated here rather than depended on, because
/// `cartalith-assets` does not (and should not) depend on `cartalith-civ`.
///
/// Byte-identical to `cartalith_civ::labels::civ_zoom_k`, **including the
/// `[0.35, 5]` clamp the shell's own third copy deliberately drops**
/// (`map_overlay.gd::_civ_zoom_k`, 2026-08-24, measured). That divergence and
/// why this copy keeps the clamp are written out once, in full, on the
/// `cartalith-civ` copy's doc comment; the short version is that above
/// `zoom_scale == 5` this term saturates at `0.2` while the shell's keeps
/// shrinking, so [`icon_box_at`]'s `r`/`side` -- and the resize handle
/// `icon_handles` derives from them -- grow relative to the mark the shell
/// actually draws. Noted here 2026-09-01 so a reader of this copy is not the
/// one reader who never sees it.
pub fn civ_zoom_k(zoom_scale: f64) -> f64 {
    1.0 / zoom_scale.clamp(0.35, 5.0)
}

/// An icon's screen box — `_carIconBox`'s return value.
///
/// The reference's own note on why this is a separate function from
/// `_civLabelBox` rather than a shared abstraction: *"an icon's box comes from
/// its sprite's native size × per-instance scale, not text metrics, so a
/// shared abstraction with labels would just be indirection with no code
/// actually in common."* Kept separate here for the same reason.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconBox {
    pub px: f64,
    pub py: f64,
    /// Sprite radius.
    pub r: f64,
    /// The **full** side of the square hit box (`r * 2.6`, not `r * 2`).
    pub side: f64,
}

/// `_carIconBox(icon)` with the default identity mapping `(x+0.5, y+0.5)`.
///
/// Unlike a label's, an icon's size is **always** zoom-relative — there is no
/// `sizeMode` equivalent.
pub fn icon_box(icon: &ManualIcon, env: &IconViewEnv) -> IconBox {
    icon_box_at(icon.x + 0.5, icon.y + 0.5, icon, env)
}

/// `_carIconBox(icon, toScreenFn)` — the render-time call site with its own
/// screen mapping.
///
/// **`r` is in grid cells and the shell draws the mark in screen pixels.**
/// `map_overlay.gd` uses its own `ICON_BASE_RADIUS := 5.5` times the icon's
/// scale, with no `sc` term at all -- unlike the settlement pins in the same
/// file, which do apply `sc`. So `r` here (`5.0 * sc * scale`, where
/// `sc = max(1, grid_w/512) * civ_zoom_k(zoom_scale) * icon_scale`) and the
/// drawn glyph are two different sizes in two different units, and the hit
/// box and resize handle follow this one. Disclosed 2026-09-01; the fix is to
/// make one side authoritative -- draw at the engine's radius, or publish `r`
/// on `icon_handles` and draw against that -- which is a change in
/// `cartalith-godot` and the shell, not here.
pub fn icon_box_at(px: f64, py: f64, icon: &ManualIcon, env: &IconViewEnv) -> IconBox {
    let sc = f64::max(1.0, env.grid_w as f64 / 512.0) * civ_zoom_k(env.zoom_scale) * env.icon_scale;
    let r = 5.0 * sc * if icon.scale == 0.0 { 1.0 } else { icon.scale };
    IconBox { px, py, r, side: r * 2.6 }
}

/// What `_carIconHitTest` found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconHitKind {
    /// `'handle'` — the resize handle.
    Handle,
    /// `'box'`.
    Box,
}

/// A hit, with the icon index for [`IconHitKind::Box`] only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconHit {
    pub kind: IconHitKind,
    pub index: Option<usize>,
}

/// A circular on-canvas handle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconHandle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

/// `_carIconHitTest(px, py)` (reference line 15325).
///
/// The armed resize handle wins over every box; icons are otherwise scanned
/// back to front, so the most recently placed icon wins an overlap.
pub fn icon_hit_test(
    boxes: &[IconBox],
    handle: Option<&IconHandle>,
    px: f64,
    py: f64,
) -> Option<IconHit> {
    if let Some(h) = handle
        && js_hypot(px - h.x, py - h.y) <= h.r
    {
        return Some(IconHit { kind: IconHitKind::Handle, index: None });
    }
    for (k, b) in boxes.iter().enumerate().rev() {
        if (px - b.px).abs() <= b.side / 2.0 && (py - b.py).abs() <= b.side / 2.0 {
            return Some(IconHit { kind: IconHitKind::Box, index: Some(k) });
        }
    }
    None
}

/// The lower and upper bounds the icon resize handle clamps `scale` to.
pub const ICON_SCALE_MIN: f64 = 0.2;
pub const ICON_SCALE_MAX: f64 = 4.0;

/// The icon resize handle: `scale = clamp(startScale * dist / startDist,
/// 0.2, 4)`, with `dist` floored at 1.
///
/// Transcribed from the pointer-move handler (reference lines 9721-9724), not
/// sliced — it is inline in a DOM event listener. Disclosed rather than
/// implied.
pub fn icon_resize_scale(start_scale: f64, cx: f64, cy: f64, gx: f64, gy: f64, start_dist: f64) -> f64 {
    let dist = f64::max(1.0, js_hypot(gx + 0.5 - cx, gy + 0.5 - cy));
    (start_scale * dist / start_dist).clamp(ICON_SCALE_MIN, ICON_SCALE_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature() -> ArmedIcon {
        ArmedIcon { family: ManualIconFamily::Feature, slot: "mountain".into(), set: None }
    }
    fn custom() -> ArmedIcon {
        ArmedIcon {
            family: ManualIconFamily::Custom,
            slot: "thing".into(),
            set: Some("myset".into()),
        }
    }

    /// The harness's seeded LCG, standing in for `Math.random`.
    fn lcg(seed: u32) -> impl FnMut() -> f64 {
        let mut s = seed;
        move || {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            s as f64 / 4294967296.0
        }
    }

    fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        let cx = gw as f64 * 0.42;
        let cy = gh as f64 * 0.55;
        let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
        for y in 0..gh {
            for x in 0..gw {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
                let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
                v += 0.05 * ((q as f64 / 10.0) - 0.5);
                v += 0.10
                    * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    #[test]
    fn the_rule_key_matches_the_scatterer_for_both_family_shapes() {
        assert_eq!(feature().rule_key(), "mountain");
        assert_eq!(custom().rule_key(), "custom::myset::thing");
    }

    #[test]
    fn an_unknown_slot_falls_back_to_the_default_rule() {
        let table = ScatterRuleTable::new();
        let r = icon_brush_rule(Some(&feature()), &table).expect("armed");
        assert_eq!(r, ScatterRule::default());
    }

    #[test]
    fn a_known_slot_uses_its_own_rule() {
        let mut table = ScatterRuleTable::new();
        table.insert("mountain", ScatterRule { min_size: 0.55, max_size: 1.9, ..Default::default() });
        let r = icon_brush_rule(Some(&feature()), &table).expect("armed");
        assert_eq!((r.min_size, r.max_size), (0.55, 1.9));
    }

    #[test]
    fn nothing_armed_means_no_rule_and_no_placement() {
        let table = ScatterRuleTable::new();
        assert!(icon_brush_rule(None, &table).is_none());
        assert!(place_manual_icon(5.0, 5.0, 48, 32, None).is_none());
        let mut icons = Vec::new();
        let mut rng = lcg(1);
        assert_eq!(
            icon_brush_stamp(&mut icons, None, &IconBrush::default(), &ScatterRule::default(),
                             &synthetic_field(48, 32, 5), 48, 32, 0.42, 20.0, 10.0, &mut rng),
            0
        );
        assert!(icons.is_empty());
    }

    #[test]
    fn click_placement_is_bounds_gated_on_every_side() {
        let a = feature();
        assert!(place_manual_icon(5.0, 5.0, 48, 32, Some(&a)).is_some());
        assert!(place_manual_icon(47.0, 31.0, 48, 32, Some(&a)).is_some());
        assert!(place_manual_icon(-1.0, 5.0, 48, 32, Some(&a)).is_none());
        assert!(place_manual_icon(48.0, 5.0, 48, 32, Some(&a)).is_none());
        assert!(place_manual_icon(5.0, 32.0, 48, 32, Some(&a)).is_none());
        assert!(place_manual_icon(5.0, -1.0, 48, 32, Some(&a)).is_none());
    }

    #[test]
    fn a_clicked_icon_is_full_scale_and_carries_a_set_only_when_custom() {
        let f = place_manual_icon(5.0, 5.0, 48, 32, Some(&feature())).expect("placed");
        assert_eq!(f.scale, 1.0);
        assert_eq!(f.set, None);
        let c = place_manual_icon(5.0, 5.0, 48, 32, Some(&custom())).expect("placed");
        assert_eq!(c.set.as_deref(), Some("myset"));
    }

    #[test]
    fn click_placement_has_no_water_gate_unlike_the_brush() {
        // The reference's click path really does not check sea level -- a
        // hand-placed buoy is a legitimate thing to want.
        let field = synthetic_field(48, 32, 5);
        let (wx, wy) = (2usize, 30usize);
        assert!(field[wy * 48 + wx] as f64 <= 0.42, "fixture cell is not water");
        assert!(place_manual_icon(wx as f64, wy as f64, 48, 32, Some(&feature())).is_some());
    }

    #[test]
    fn the_brush_never_paints_into_water_or_out_of_bounds() {
        let field = synthetic_field(48, 32, 5);
        let mut icons = Vec::new();
        let mut rng = lcg(12345);
        icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r: 12.0, density: 1.0 },
                         &ScatterRule::default(), &field, 48, 32, 0.42, 20.0, 10.0, &mut rng);
        assert!(!icons.is_empty());
        for ic in &icons {
            assert!((0.0..48.0).contains(&ic.x) && (0.0..32.0).contains(&ic.y));
            assert!(field[ic.y as usize * 48 + ic.x as usize] as f64 > 0.42);
        }
    }

    #[test]
    fn the_brush_respects_the_blue_noise_spacing() {
        let field = synthetic_field(48, 32, 5);
        let mut icons = Vec::new();
        let mut rng = lcg(999);
        let brush = IconBrush { on: true, r: 10.0, density: 0.6 };
        icon_brush_stamp(&mut icons, Some(&feature()), &brush, &ScatterRule::default(),
                         &field, 48, 32, 0.42, 20.0, 10.0, &mut rng);
        let spacing = f64::max(ICON_BRUSH_MIN_SPACING, 3.0 / 0.6f64.sqrt());
        for i in 0..icons.len() {
            for j in (i + 1)..icons.len() {
                let (dx, dy) = (icons[i].x - icons[j].x, icons[i].y - icons[j].y);
                assert!(dx * dx + dy * dy >= spacing * spacing, "icons {i} and {j} are too close");
            }
        }
    }

    #[test]
    fn re_stamping_the_same_spot_thickens_the_stand_and_then_saturates() {
        let field = synthetic_field(48, 32, 5);
        let mut icons = Vec::new();
        let mut rng = lcg(777);
        let brush = IconBrush { on: true, r: 6.0, density: 0.6 };
        let mut counts = Vec::new();
        for _ in 0..4 {
            counts.push(icon_brush_stamp(&mut icons, Some(&feature()), &brush,
                                         &ScatterRule::default(), &field, 48, 32, 0.42,
                                         20.0, 10.0, &mut rng));
        }
        assert!(counts[0] > 0);
        assert!(counts.iter().sum::<usize>() == icons.len());
        // Later stamps add less, never more -- the spacing limit filling up.
        assert!(counts[3] <= counts[0]);
    }

    #[test]
    fn a_brush_entirely_over_water_places_nothing() {
        let field = synthetic_field(48, 32, 5);
        let mut icons = Vec::new();
        let mut rng = lcg(42);
        assert_eq!(
            icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r: 3.0, density: 0.6 },
                             &ScatterRule::default(), &field, 48, 32, 0.42, 2.0, 30.0, &mut rng),
            0
        );
    }

    #[test]
    fn the_dart_budget_is_capped_however_big_the_brush_gets() {
        // A radius-200 brush at max density would ask for ~170k darts
        // uncapped; the cap means the RNG is called at most 3 * 1500 times.
        let field = synthetic_field(48, 32, 5);
        let mut icons = Vec::new();
        let mut calls = 0usize;
        let mut inner = lcg(5);
        let mut rng = || {
            calls += 1;
            inner()
        };
        icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r: 200.0, density: 1.0 },
                         &ScatterRule::default(), &field, 48, 32, 0.42, 24.0, 16.0, &mut rng);
        assert!(calls <= ICON_BRUSH_MAX_DARTS * 3, "{calls} rng calls");
        assert!(calls >= ICON_BRUSH_MAX_DARTS * 2);
    }

    #[test]
    fn the_density_floor_keeps_a_zeroed_slider_painting() {
        let field = synthetic_field(48, 32, 5);
        let mut icons = Vec::new();
        let mut rng = lcg(8);
        let n = icon_brush_stamp(&mut icons, Some(&feature()),
                                 &IconBrush { on: true, r: 5.0, density: 0.0 },
                                 &ScatterRule::default(), &field, 48, 32, 0.42, 20.0, 10.0, &mut rng);
        assert!(n > 0);
    }

    #[test]
    fn brushed_scale_stays_inside_the_rules_size_band() {
        let field = synthetic_field(48, 32, 5);
        let rule = ScatterRule { min_size: 0.55, max_size: 1.9, ..Default::default() };
        let mut icons = Vec::new();
        let mut rng = lcg(31337);
        icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r: 10.0, density: 0.8 },
                         &rule, &field, 48, 32, 0.42, 20.0, 10.0, &mut rng);
        assert!(!icons.is_empty());
        assert!(icons.iter().all(|i| (0.55..=1.9).contains(&i.scale)));
    }

    #[test]
    fn existing_icons_block_new_darts_even_at_fractional_positions() {
        let field = synthetic_field(48, 32, 5);
        // A dense pre-existing mat over the brush disc leaves nowhere to land.
        let mut icons: Vec<ManualIcon> = (0..48)
            .flat_map(|x| (0..32).map(move |y| (x, y)))
            .map(|(x, y)| ManualIcon {
                x: x as f64 + 0.25,
                y: y as f64 + 0.25,
                family: ManualIconFamily::Feature,
                slot: "mountain".into(),
                set: None,
                scale: 1.0,
            })
            .collect();
        let before = icons.len();
        let mut rng = lcg(3);
        let n = icon_brush_stamp(&mut icons, Some(&feature()),
                                 &IconBrush { on: true, r: 8.0, density: 0.6 },
                                 &ScatterRule::default(), &field, 48, 32, 0.42, 20.0, 10.0, &mut rng);
        assert_eq!(n, 0);
        assert_eq!(icons.len(), before);
    }

    #[test]
    fn js_round_is_half_up_which_matters_at_the_left_edge() {
        assert_eq!(js_round(-0.5), 0.0);
        assert_eq!(js_round(-0.51), -1.0);
        assert_eq!(js_round(0.5), 1.0);
    }

    #[test]
    fn the_icon_box_is_wider_than_the_sprite_it_wraps() {
        let ic = ManualIcon {
            x: 10.0,
            y: 8.0,
            family: ManualIconFamily::Settlement,
            slot: "city".into(),
            set: None,
            scale: 1.0,
        };
        let b = icon_box(&ic, &IconViewEnv { grid_w: 48, zoom_scale: 1.0, icon_scale: 1.0 });
        assert_eq!((b.px, b.py), (10.5, 8.5));
        assert_eq!(b.r, 5.0);
        assert_eq!(b.side, 13.0);
    }

    #[test]
    fn per_instance_scale_scales_the_box() {
        let mut ic = ManualIcon {
            x: 0.0,
            y: 0.0,
            family: ManualIconFamily::Feature,
            slot: "mountain".into(),
            set: None,
            scale: 1.0,
        };
        let env = IconViewEnv { grid_w: 48, zoom_scale: 1.0, icon_scale: 1.0 };
        let a = icon_box(&ic, &env).r;
        ic.scale = 2.5;
        assert_eq!(icon_box(&ic, &env).r, a * 2.5);
    }

    #[test]
    fn the_handle_beats_every_box_and_a_miss_returns_nothing() {
        let boxes = vec![IconBox { px: 0.0, py: 0.0, side: 100.0, r: 38.0 }];
        let h = IconHandle { x: 5.0, y: 5.0, r: 2.0 };
        assert_eq!(icon_hit_test(&boxes, Some(&h), 5.0, 5.0).unwrap().kind, IconHitKind::Handle);
        assert_eq!(icon_hit_test(&boxes, None, 5.0, 5.0).unwrap().kind, IconHitKind::Box);
        assert!(icon_hit_test(&boxes, Some(&h), 500.0, 500.0).is_none());
    }

    #[test]
    fn the_topmost_icon_wins_an_overlap() {
        let boxes = vec![
            IconBox { px: 0.0, py: 0.0, side: 20.0, r: 7.7 },
            IconBox { px: 1.0, py: 1.0, side: 20.0, r: 7.7 },
        ];
        assert_eq!(icon_hit_test(&boxes, None, 0.5, 0.5).unwrap().index, Some(1));
    }

    #[test]
    fn resizing_an_icon_clamps_between_a_fifth_and_four_times() {
        assert_eq!(icon_resize_scale(1.0, 10.0, 10.0, 10.0, 10.0, 5.0), ICON_SCALE_MIN);
        assert_eq!(icon_resize_scale(1.0, 10.0, 10.0, 60.0, 60.0, 3.0), ICON_SCALE_MAX);
    }

    #[test]
    fn the_family_keys_round_trip() {
        for f in [
            ManualIconFamily::Settlement,
            ManualIconFamily::Feature,
            ManualIconFamily::Poi,
            ManualIconFamily::Custom,
        ] {
            assert_eq!(ManualIconFamily::from_key(f.key()), Some(f));
        }
        assert_eq!(ManualIconFamily::from_key("nope"), None);
    }

    #[test]
    fn the_feature_family_draws_from_the_packs_icons_directory() {
        // The one rename between the two taxonomies, pinned so it is not
        // "tidied" into a same-name mapping.
        assert_eq!(ManualIconFamily::Feature.pack_family(), crate::slots::Family::Icons);
        assert_eq!(ManualIconFamily::Settlement.pack_family(), crate::slots::Family::Settlement);
    }

    // -----------------------------------------------------------------
    // The brush's derived constants, pinned directly.
    //
    // Mutation testing found that `ICON_BRUSH_MIN_DENSITY`,
    // `ICON_BRUSH_MIN_SPACING`, `ICON_BRUSH_MAX_DARTS`, the `3.0` spacing
    // constant and the `* 2` dart oversample all survived the golden fixtures:
    // a small saturated disc reaches the same answer at either setting, so the
    // goldens could not see them. These tests drive the RNG with a **scripted**
    // sequence instead of a stream, so each constant's effect is observed on
    // its own rather than through a statistical outcome.
    // -----------------------------------------------------------------

    /// An RNG that replays a fixed script, then returns 0.0 forever, and
    /// counts how many times it was asked.
    fn scripted(values: Vec<f64>) -> (impl FnMut() -> f64, std::rc::Rc<std::cell::Cell<usize>>) {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0usize));
        let c = calls.clone();
        let mut i = 0usize;
        (
            move || {
                c.set(c.get() + 1);
                let v = values.get(i).copied().unwrap_or(0.0);
                i += 1;
                v
            },
            calls,
        )
    }

    /// Land everywhere, so only the spacing rule can reject a dart.
    fn all_land(gw: usize, gh: usize) -> Vec<f32> {
        vec![0.9f32; gw * gh]
    }

    /// Place a dart at exactly `(cx + dx, cy)` by scripting `ang = 0` (so
    /// `cos = 1`) and `rad = dx` via `rng()^2 * r^2`.
    fn dart_at(dx: f64, r: f64) -> [f64; 2] {
        [0.0, (dx / r) * (dx / r)]
    }

    /// Fire exactly one dart at `(cx + d, cy)` against a pre-existing icon at
    /// `existing`, and report whether it was accepted.
    ///
    /// A dart always lands on an integer cell, so two *darts* can only ever be
    /// an integer distance apart — and no integer separation lies between 2.9
    /// and 3.0, which is why a dart-versus-dart fixture structurally cannot see
    /// the spacing constant. An icon already on the map can sit anywhere, so
    /// the fixture that *can* see it seeds one at a fractional position.
    fn one_dart_against(existing: (f64, f64), d: f64, r: f64, density: f64) -> bool {
        let field = all_land(48, 32);
        let mut icons = vec![ManualIcon {
            x: existing.0,
            y: existing.1,
            family: ManualIconFamily::Feature,
            slot: "mountain".into(),
            set: None,
            scale: 1.0,
        }];
        let (mut rng, _) = scripted([dart_at(d, r).to_vec(), vec![1.0]].concat());
        icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r, density },
                         &ScatterRule::default(), &field, 48, 32, 0.42, 20.0, 10.0, &mut rng) == 1
    }

    #[test]
    fn the_spacing_constant_decides_a_rejection_at_a_known_separation() {
        // At density 1.0 the spacing is exactly 3.0 / sqrt(1) = 3.0. A dart
        // 2.95 from an existing icon collides; 3.05 does not. A spacing
        // constant of 2.9 would accept the 2.95 case.
        assert!(!one_dart_against((17.05, 10.0), 0.0, 10.0, 1.0), "2.95 apart must collide at 3.0");
        assert!(one_dart_against((16.95, 10.0), 0.0, 10.0, 1.0), "3.05 apart must not");
    }

    #[test]
    fn the_spacing_floor_binds_only_above_a_density_of_625_percent() {
        // `max(1.2, 3/sqrt(d))` reaches its floor only at d > 6.25, which the
        // reference's own 0..1 density slider cannot reach -- so inside the
        // shipped parameter range the floor is unobservable and no golden can
        // see it. Driven out of range here: at d = 100 the spacing is 1.2, so
        // 1.25 away is accepted and 1.15 away is not. A floor of 1.3 would
        // reject the 1.25 case.
        assert!(one_dart_against((18.75, 10.0), 0.0, 10.0, 100.0), "1.25 apart must be accepted at 1.2");
        assert!(!one_dart_against((18.85, 10.0), 0.0, 10.0, 100.0), "1.15 apart must not be");
    }

    #[test]
    fn the_dart_budget_is_exactly_the_references_formula() {
        // Counted rather than inferred: with every dart rejected (all water)
        // the RNG is consulted exactly twice per attempt, so the call count
        // *is* the attempt count. This is the only thing that can see the `* 2`
        // oversample and the 1500 cap, both of which a saturated disc hides.
        let water = vec![0.1f32; 48 * 32];
        let cases: &[(f64, f64, usize)] = &[
            // r, density, expected attempts = max(1, min(1500, ceil(PI r^2 / spacing^2) * 2))
            (6.0, 0.6, 16),     // spacing 3.873, PI*36/15.0 = 7.54 -> ceil 8 -> 16
            (4.0, 1.0, 12),     // spacing 3.0,   PI*16/9    = 5.59 -> ceil 6 -> 12
            (60.0, 1.0, 1500),  // uncapped 2514 -> the cap
            (0.5, 0.6, 2),      // PI*0.25/15 = 0.05 -> ceil 1 -> 2
        ];
        for &(r, density, want) in cases {
            let mut icons = Vec::new();
            let (mut rng, calls) = scripted(Vec::new());
            icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r, density },
                             &ScatterRule::default(), &water, 48, 32, 0.42, 24.0, 16.0, &mut rng);
            assert!(icons.is_empty());
            assert_eq!(calls.get(), want * 2, "r={r} density={density}");
        }
    }

    #[test]
    fn the_density_floor_pins_the_attempt_count_below_two_percent() {
        // Zero and 0.02 must derive the same spacing (and so the same attempt
        // count); 0.03 must not. A raised floor would make all three agree.
        let water = vec![0.1f32; 48 * 32];
        let count = |density: f64| {
            let mut icons = Vec::new();
            let (mut rng, calls) = scripted(Vec::new());
            icon_brush_stamp(&mut icons, Some(&feature()), &IconBrush { on: true, r: 15.0, density },
                             &ScatterRule::default(), &water, 48, 32, 0.42, 20.0, 12.0, &mut rng);
            calls.get()
        };
        assert_eq!(count(0.0), count(0.02), "zero density must clamp to the floor");
        assert_ne!(count(0.0), count(0.03), "0.03 is above the floor and must differ");
    }

    #[test]
    fn a_dart_landing_on_exactly_minus_a_half_rounds_into_the_grid() {
        // The one input where JS's `Math.round` and Rust's `f64::round`
        // disagree. A continuous RNG effectively never produces it, which is
        // why every golden survived the mutation; scripting `rad = 0` at a
        // centre of -0.5 hits it exactly.
        let field = all_land(48, 32);
        let (mut rng, _) = scripted(vec![0.0, 0.0, 1.0]);
        let mut icons = Vec::new();
        let n = icon_brush_stamp(&mut icons, Some(&feature()),
                                 &IconBrush { on: true, r: 0.5, density: 0.6 },
                                 &ScatterRule::default(), &field, 48, 32, 0.42, -0.5, 10.0, &mut rng);
        assert_eq!(n, 1, "Math.round(-0.5) is 0 and lands in bounds");
        assert_eq!(icons[0].x, 0.0);
    }

    #[test]
    fn the_brush_defaults_are_the_references_own() {
        let b = IconBrush::default();
        assert!(!b.on);
        assert_eq!(b.r, 12.0);
        assert_eq!(b.density, 0.6);
    }
}
