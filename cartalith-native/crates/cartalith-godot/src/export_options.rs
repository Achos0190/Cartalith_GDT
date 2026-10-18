//! The export options (`EXPORT_SCOPE.md` §7, milestone E3): one options value
//! across the boundary instead of a growing list of `#[func]` parameters.
//!
//! This module is the **Godot-free** half: the option types, the name
//! vocabularies they are parsed from, the style override's own edits, and the
//! band-plan arithmetic the estimate reports. The `Dictionary` walk and the
//! two `#[func]`s that consume it (`export_image`, `export_image_estimate`)
//! are in `export_raster.rs`, because they need `WorldGen`.
//!
//! Compiled standalone by `tests/export_options.rs` through the same `#[path]`
//! trick `tests/export_stream.rs` uses.
//!
//! # What E3 can draw, and what it only records
//!
//! The content set carries every item the owner named (*"settlements, routes,
//! what layers"*, `EXPORT_SCOPE.md` "What was asked for") and the explicit
//! settlement tier of §5. **Only the terrain and the baked river ink are
//! drawable by `export_image`.** Everything else is `map_overlay.gd`'s to
//! draw, through E4's overlay session (`export_session.rs`,
//! `WorldGen::export_session_begin`), whose Rust core exists but whose tile
//! renderer on the Godot side does not yet. So an overlay in the content set
//! is parsed and reported by name ([`ExportContent::overlays`]) and
//! `export_image` **refuses** it rather than writing a terrain-only file that
//! looks like it honoured the request.
#![allow(dead_code)]

use std::path::Path;

use crate::export_stream::StreamFormat;
use crate::render::{self, ColorSpace, ElevationRamp, ExportBandPlan, TerrainAppearance};

/// The file formats `export_banded` streams, by the names the options
/// dictionary carries. PNG first: ruling 26's format and the default. `"jpeg"`
/// (E5, Ruling AZ) names the JPEG at its default quality; [`format_with_quality`]
/// applies a `quality` option. One spelling per format: `"jpg"` is the file
/// extension the shell writes, not a second format name.
pub const FORMAT_NAMES: [(&str, StreamFormat); 3] = [("png", StreamFormat::Png), ("bigtiff", StreamFormat::BigTiff), ("jpeg", StreamFormat::JPEG)];

/// Parses a file-format name from the options dictionary, case-insensitively; `None` for an unknown name so the caller can refuse it rather than silently default to PNG.
pub fn format_from_name(name: &str) -> Option<StreamFormat> {
    FORMAT_NAMES.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|&(_, f)| f)
}

/// The canonical option name of a format, the inverse of [`format_from_name`]:
/// every JPEG quality is `"jpeg"` (matched by kind, so a non-default quality
/// does not fall through). The `"png"` fallback is unreachable while
/// `FORMAT_NAMES` lists every `StreamFormat` kind; it exists only so this
/// stays total.
pub fn format_name(f: StreamFormat) -> &'static str {
    FORMAT_NAMES.iter().find(|(_, g)| std::mem::discriminant(g) == std::mem::discriminant(&f)).map_or("png", |(n, _)| n)
}

/// The format after the options' `quality` (E5): `None` leaves `format` as
/// parsed (a JPEG keeps [`crate::export_stream::JPEG_QUALITY_DEFAULT`]); a
/// quality on a JPEG must lie in [`crate::export_stream::JPEG_QUALITY_RANGE`]
/// and is **refused, not clamped**, outside it; a quality on PNG or BigTIFF is
/// a contradiction (both are lossless), refused like a settlement tier with
/// settlements off, so the dialog cannot believe it set something it did not.
/// Pure so `tests/export_options.rs` reaches it; `export_raster.rs`' dictionary
/// reader is its one caller.
pub fn format_with_quality(format: StreamFormat, quality: Option<i64>) -> Result<StreamFormat, String> {
    let Some(q) = quality else {
        return Ok(format);
    };
    if !format.is_jpeg() {
        return Err(format!("quality applies to JPEG only -- {} is lossless", format_name(format)));
    }
    let range = crate::export_stream::JPEG_QUALITY_RANGE;
    match u8::try_from(q) {
        Ok(q) if range.contains(&q) => Ok(StreamFormat::Jpeg { quality: q }),
        _ => Err(format!("JPEG quality {q} is outside {}..={}", range.start(), range.end())),
    }
}

/// The JPEG quality of `format`, or `None` for a lossless format -- for the
/// result dictionaries, which report `quality` only for a JPEG (a `0` or
/// `100` there for a PNG would read as a setting it never had).
pub fn format_quality(format: StreamFormat) -> Option<u8> {
    match format {
        StreamFormat::Jpeg { quality } => Some(quality),
        _ => None,
    }
}

/// Parses the `color_space` export option (ET-E1, "Export colour space: sRGB /
/// Display P3") against `render::COLOR_SPACES`' own names (`"sRGB"`,
/// `"Display P3"`), case-insensitively like [`format_from_name`]; `None` for
/// anything else, which the parser turns into a refusal of the whole options
/// dictionary, never a silent sRGB. This is the **export's** colour space: the
/// encoding and label of a document. It is deliberately not
/// `WorldGen::color_space`, which describes the monitor in front of the session
/// and never reaches an export.
pub fn color_space_from_name(name: &str) -> Option<ColorSpace> {
    render::COLOR_SPACES.iter().find(|n| n.eq_ignore_ascii_case(name)).and_then(|n| ColorSpace::from_name(n))
}

/// How far down the settlement hierarchy an export draws (`EXPORT_SCOPE.md`
/// §5). A flat image has no zoom, so `map_overlay.gd`'s `SETTLEMENT_LOD`
/// cannot be inherited: the export states the tier instead.
///
/// Ordered largest first, so a tier includes every variant **before** it.
/// `All` is `Hamlet` plus the additive villages `map_overlay.gd` hides
/// outright below `VILLAGE_ADDON_LOD` — the one class that is not a rung of
/// the ladder but a separate layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettlementTier {
    Metropolis,
    Capital,
    City,
    Town,
    Village,
    Hamlet,
    All,
}

impl SettlementTier {
    /// The tier `settlements: true` draws when no `settlement_tier` is given:
    /// **towns** (owner, Ruling AZ, 2026-09-28, `LARGE_ITEM_RULINGS.md`: "the
    /// user picks a minimum settlement tier (default towns)"). Until
    /// 2026-10-13 the parser defaulted to `All` (§5's "or simply all of
    /// them"), which disagreed with the ruling; this constant is the fix at
    /// its cause, and `shell/export_overlay.gd::overlay_content` mirrors it.
    /// Must never become `All` again without a new ruling: `All` also draws
    /// the additive villages, the densest layer the overlay has.
    pub const DEFAULT: SettlementTier = SettlementTier::Town;

    /// `map_overlay.gd`'s `SETTLEMENT_LOD` keys, in its order, plus `"all"`.
    pub const ALL: [(&'static str, SettlementTier); 7] = [
        ("metropolis", SettlementTier::Metropolis),
        ("capital", SettlementTier::Capital),
        ("city", SettlementTier::City),
        ("town", SettlementTier::Town),
        ("village", SettlementTier::Village),
        ("hamlet", SettlementTier::Hamlet),
        ("all", SettlementTier::All),
    ];

    /// Parses a tier name (`"metropolis"` .. `"all"`) case-insensitively; `None` for anything else, which the options parser turns into a refusal.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|&(_, t)| t)
    }

    /// The canonical lowercase name of this tier, the inverse of [`Self::from_name`] (the `"all"` fallback is unreachable while `ALL` lists every variant).
    pub fn name(self) -> &'static str {
        Self::ALL.iter().find(|(_, t)| *t == self).map_or("all", |(n, _)| n)
    }

    /// Whether a settlement of `kind` (a `SETTLEMENT_LOD` key) is drawn at
    /// this tier. An additive village is asked for as `"village_addon"`.
    pub fn includes(self, kind: &str) -> bool {
        if kind == "village_addon" {
            return self == SettlementTier::All;
        }
        Self::from_name(kind).is_some_and(|k| k != SettlementTier::All && k <= self)
    }
}

/// The content set's settlement tier from the options dictionary's
/// `settlements` flag and `settlement_tier` name (already parsed): `Ok(None)`
/// draws no settlements, `Ok(Some(t))` draws down to `t`.
///
/// - `settlements: true` with no tier is [`SettlementTier::DEFAULT`] (towns,
///   Ruling AZ) -- not `All`, which this parser returned before 2026-10-13.
/// - A tier alone turns settlements on at that tier.
/// - A tier with `settlements: false` is a contradiction, refused rather than
///   resolved either way, so the dialog cannot silently lose a choice.
///
/// Pure so `tests/export_options.rs` can reach it; `export_raster.rs`'s
/// dictionary reader is its one caller.
pub fn settlements_from(flag: Option<bool>, tier: Option<SettlementTier>) -> Result<Option<SettlementTier>, String> {
    match (flag, tier) {
        (Some(false), Some(_)) => Err("settlement_tier given with settlements off".into()),
        (Some(true), t) => Ok(Some(t.unwrap_or(SettlementTier::DEFAULT))),
        (None, t) => Ok(t),
        (Some(false), None) => Ok(None),
    }
}

/// Everything one export is asked to be — the single value `export_image`
/// and `export_image_estimate` take, read from one `Dictionary`:
///
/// ```text
/// { width: 16384,                      # required, a BAKE_WIDTHS rung
///   format: "png" | "bigtiff" | "jpeg", # default "png"
///   quality: 1..100,                   # JPEG only, default 90 (E5)
///   color_space: "sRGB" | "Display P3", # default "sRGB" (ET-E1); P3 is PNG or JPEG
///   style: { look, preset, ramp, tunables: {key: value} },   # all optional
///   content: { rivers, settlements, settlement_tier,
///              routes, ways, labels, territory, icons } }    # all optional
/// ```
///
/// Unknown keys at any level are refused, not ignored: `"fromat"` quietly
/// exporting a PNG is the failure this shape exists to prevent.
#[derive(Clone)]
pub struct ExportOptions {
    pub width: i64,
    /// The file format, a JPEG's quality included ([`format_with_quality`]).
    pub format: StreamFormat,
    /// The export's colour space (ET-E1): the pixels' encoding and the PNG's
    /// label. Default [`ColorSpace::Srgb`]; never read from the session's
    /// monitor setting.
    pub color_space: ColorSpace,
    pub style: ExportStyle,
    pub content: ExportContent,
}

impl ExportOptions {
    pub const KEYS: [&'static str; 6] = ["width", "format", "quality", "color_space", "style", "content"];
    pub const STYLE_KEYS: [&'static str; 4] = ["look", "preset", "ramp", "tunables"];
    pub const CONTENT_KEYS: [&'static str; 8] = ["rivers", "settlements", "settlement_tier", "routes", "ways", "labels", "territory", "icons"];
}

/// What the exported image contains. `rivers` is the baked river ink every
/// shipped export carries; the rest are overlays (see the module doc).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportContent {
    pub rivers: bool,
    /// `None` draws no settlements; `Some(tier)` draws down to `tier`.
    pub settlements: Option<SettlementTier>,
    pub routes: bool,
    pub ways: bool,
    pub labels: bool,
    pub territory: bool,
    pub icons: bool,
}

impl Default for ExportContent {
    /// What `export_raster_png` has always drawn: terrain and rivers.
    fn default() -> Self {
        ExportContent { rivers: true, settlements: None, routes: false, ways: false, labels: false, territory: false, icons: false }
    }
}

impl ExportContent {
    /// The boolean overlay keys, in the order `overlays` reports them.
    pub const OVERLAY_KEYS: [&'static str; 5] = ["routes", "ways", "labels", "territory", "icons"];

    /// Every requested item only E4's overlay session could draw.
    pub fn overlays(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.settlements.is_some() {
            out.push("settlements");
        }
        for (key, on) in Self::OVERLAY_KEYS.iter().zip([self.routes, self.ways, self.labels, self.territory, self.icons]) {
            if on {
                out.push(key);
            }
        }
        out
    }

    /// Sets one boolean overlay flag by its option key. Returns `false`, changing nothing, for a key that is not one of [`Self::OVERLAY_KEYS`], so the parser can refuse unknown content keys instead of dropping them.
    pub fn set_overlay(&mut self, key: &str, on: bool) -> bool {
        let slot = match key {
            "routes" => &mut self.routes,
            "ways" => &mut self.ways,
            "labels" => &mut self.labels,
            "territory" => &mut self.territory,
            "icons" => &mut self.icons,
            _ => return false,
        };
        *slot = on;
        true
    }
}

/// The export's own style, layered over the session's appearance **without
/// writing to it** (`EXPORT_SCOPE.md` §2.4). Every field is optional; an
/// empty override exports exactly what is on screen.
///
/// Two layers, applied in `WorldGen::appearance()`'s own order:
///
/// 1. **A new base** — `look` *or* `preset`, never both (a preset replaces
///    the look, so naming both is a contradiction and is refused). Composed by
///    `WorldGen::appearance_rebased`, which mirrors what `set_look` /
///    `load_appearance_preset` would have produced, on a copy.
/// 2. **Edits on top** — [`Self::apply_edits`]: a ramp preset, then tunables.
#[derive(Clone, Default)]
pub struct ExportStyle {
    /// A `render::LOOK_PRESETS` name, canonicalised.
    pub look: Option<&'static str>,
    /// A decoded `cartalith-appearance` preset file.
    pub preset: Option<TerrainAppearance>,
    /// A `render::RAMP_PRESETS` name, canonicalised.
    pub ramp: Option<&'static str>,
    /// `(key, value)` already clamped to `list_appearance_tunables`' range.
    pub tunables: Vec<(&'static str, f64)>,
}

impl ExportStyle {
    /// True when no override is set, i.e. the export reproduces exactly what is on screen.
    pub fn is_empty(&self) -> bool {
        self.look.is_none() && self.preset.is_none() && self.ramp.is_none() && self.tunables.is_empty()
    }

    /// The edits that sit on top of whatever base was chosen, on
    /// `load_ramp_preset` / `set_appearance`'s exact terms: a ramp preset
    /// keeps the interpolation mode already in force, and a tunable is
    /// clamped to the published range (already done at parse time).
    ///
    /// Takes the appearance **by value** and returns a new one: there is no
    /// reference to the session's state here to write through.
    pub fn apply_edits(&self, mut a: TerrainAppearance) -> TerrainAppearance {
        if let Some(name) = self.ramp
            && let Some(mut ramp) = ElevationRamp::preset(name)
        {
            ramp.set_mode(a.ramp.mode());
            a.ramp = ramp;
        }
        for &(key, v) in &self.tunables {
            if key == render::TUNABLE_LIGHTS.0 {
                a.relief_lights = v.round().max(1.0) as usize;
            } else {
                a.set_tunable(key, v);
            }
        }
        a
    }
}

/// Canonicalises a look-preset name (case-insensitive) to the `render::LOOK_PRESETS` entry; `None` if unknown. Returning the static name keeps [`ExportStyle::look`] allocation-free.
pub fn look_from_name(name: &str) -> Option<&'static str> {
    render::LOOK_PRESETS.iter().copied().find(|n| n.eq_ignore_ascii_case(name))
}

/// Canonicalises a ramp-preset name to the `render::RAMP_PRESETS` entry. Unlike looks this match is case-sensitive, mirroring `load_ramp_preset`; `None` if unknown.
pub fn ramp_from_name(name: &str) -> Option<&'static str> {
    render::RAMP_PRESETS.iter().map(|(n, _)| *n).find(|n| *n == name)
}

/// One tunable, validated and clamped as `set_appearance` clamps it. An
/// unknown key is an `Err`: a typo in an export style must not quietly render
/// a look nobody asked for.
pub fn tunable_from(key: &str, v: f64) -> Result<(&'static str, f64), String> {
    if !v.is_finite() {
        return Err(format!("style tunable {key} is not a finite number"));
    }
    if let Some((k, lo, hi, _)) = TerrainAppearance::TUNABLE.iter().find(|(k, ..)| *k == key) {
        return Ok((k, v.clamp(*lo, *hi)));
    }
    let (lk, llo, lhi, _) = render::TUNABLE_LIGHTS;
    if key == lk {
        return Ok((lk, v.round().clamp(llo, lhi)));
    }
    Err(format!("unknown style tunable {key:?} -- see list_appearance_tunables()"))
}

/// Read a preset `save_appearance_preset` wrote. The one parser for that
/// format: `load_appearance_preset` calls it too, so the export and the
/// session can never disagree about what a file means.
pub fn read_appearance_preset(path: &Path) -> Result<TerrainAppearance, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("open failed: {e}"))?;
    parse_appearance_preset(&text)
}

/// Decodes preset-file text: checks the `"format": "cartalith-appearance"` marker, then deserialises the `appearance` block. Every failure is an `Err` string; it never panics on a malformed or foreign JSON file.
pub fn parse_appearance_preset(text: &str) -> Result<TerrainAppearance, String> {
    let doc: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("parse failed: {e}"))?;
    if doc.get("format").and_then(|v| v.as_str()) != Some("cartalith-appearance") {
        return Err("not a Cartalith appearance preset".into());
    }
    let body = doc.get("appearance").ok_or("no appearance block")?;
    let a = serde_json::from_value::<TerrainAppearance>(body.clone()).map_err(|e| format!("decode failed: {e}"))?;
    // ET-C2: a preset's adjustment stacks go straight onto the look, so they
    // are validated here, the one reader of the format: a file carrying an
    // out-of-range parameter is refused whole, never loaded clamped.
    a.terrain_adjust.validate().map_err(|e| format!("terrain_adjust refused: {e}"))?;
    a.water_adjust.validate().map_err(|e| format!("water_adjust refused: {e}"))?;
    Ok(a)
}

/// The saved-look document `WorldGen::save_appearance_preset` (and its ET-F3
/// sibling `save_appearance_preset_styled`) writes: the format tag, `v: 1`,
/// the display name and the merged appearance, plus -- ET-F3,
/// `EDITING_TOOLS_SCOPE.md` section 7 -- an optional top-level
/// `"style": {"preset", "variant"}` naming the style-preset variant the look
/// was saved under.
///
/// `style` is `None` for a look saved with no variant (a base preset, Custom),
/// and then the document is exactly the pre-ET-F3 shape: no `style` key at
/// all, never an empty one, so every look saved without a variant keeps its
/// bytes. The member is outside `appearance` on purpose:
/// [`parse_appearance_preset`] reads only that block, so an older build loads a
/// styled file unchanged and simply ignores the name. The preset and variant
/// strings are written as given; their vocabulary is the shell's
/// (`render_workspace.gd`'s `STYLE_PRESETS` / `STYLE_VARIANTS`), and the shell
/// is what falls back on a name it does not know.
pub fn appearance_preset_document(
    name: &str,
    appearance: &TerrainAppearance,
    style: Option<(&str, &str)>,
) -> serde_json::Value {
    let mut doc = serde_json::json!({
        "format": "cartalith-appearance",
        "v": 1,
        "name": name,
        "appearance": appearance,
    });
    if let Some((preset, variant)) = style {
        doc["style"] = serde_json::json!({ "preset": preset, "variant": variant });
    }
    doc
}

/// The `style` member of a saved-look file (see [`appearance_preset_document`])
/// as `(preset, variant)`, or `None` when the text is not a Cartalith look, has
/// no `style`, or either member is missing, not a string or empty. Never an
/// error: a look without a variant is the common case, and a malformed `style`
/// costs only the variant name -- the look itself is still loadable by
/// [`parse_appearance_preset`], which never reads this member. Validating the
/// names is the shell's job (an unknown variant falls back to the base preset
/// there), so any non-empty string is returned as written.
pub fn parse_preset_style(text: &str) -> Option<(String, String)> {
    let doc: serde_json::Value = serde_json::from_str(text).ok()?;
    if doc.get("format").and_then(|v| v.as_str()) != Some("cartalith-appearance") {
        return None;
    }
    let style = doc.get("style")?;
    let preset = style.get("preset")?.as_str()?;
    let variant = style.get("variant")?.as_str()?;
    if preset.is_empty() || variant.is_empty() {
        return None;
    }
    Some((preset.to_string(), variant.to_string()))
}

/// The band budget, in output pixels, for a banded export.
///
/// **Derived, not chosen**: a band holds at most `ungated_px` pixels — the
/// whole raster at `export_raster.rs`'s `UNGATED_MAX_WIDTH`, the largest
/// export that has ever shipped with no memory gate. So every width up to and
/// including it is **one band with no apron** (E1's monolithic path, byte for
/// byte) and a 16K/32K export costs at most what an 8K one always has. Where
/// the platform reports a budget smaller than that, the bands shrink to fit it
/// rather than the export being refused.
pub fn band_budget_px(ungated_px: u64, available_bytes: Option<u64>, peak_bytes_per_px: u64) -> u64 {
    match available_bytes {
        Some(avail) => ungated_px.min(avail / peak_bytes_per_px.max(1)),
        None => ungated_px,
    }
}

/// The tallest band the plan renders, apron included, in output pixels —
/// the banded export's peak is this times the per-pixel peak.
pub fn band_peak_px(plan: &ExportBandPlan) -> u64 {
    plan.bands().map(|b| (b.rows + b.top + b.bottom) as u64).max().unwrap_or(0) * plan.w as u64
}
