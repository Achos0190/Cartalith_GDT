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
//! drawable today.** Everything else is `map_overlay.gd`'s to draw, through
//! E4's overlay session, which does not exist. So an overlay in the content
//! set is parsed and reported by name ([`ExportContent::overlays`]) and the
//! export **refuses** it rather than writing a terrain-only file that looks
//! like it honoured the request.
#![allow(dead_code)]

use std::path::Path;

use crate::export_stream::StreamFormat;
use crate::render::{self, ElevationRamp, ExportBandPlan, TerrainAppearance};

/// The file formats `export_banded` streams, by the names the options
/// dictionary carries. PNG first: ruling 26's format and the default.
pub const FORMAT_NAMES: [(&str, StreamFormat); 2] = [("png", StreamFormat::Png), ("bigtiff", StreamFormat::BigTiff)];

pub fn format_from_name(name: &str) -> Option<StreamFormat> {
    FORMAT_NAMES.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|&(_, f)| f)
}

pub fn format_name(f: StreamFormat) -> &'static str {
    FORMAT_NAMES.iter().find(|(_, g)| *g == f).map_or("png", |(n, _)| n)
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

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|&(_, t)| t)
    }

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

/// Everything one export is asked to be — the single value `export_image`
/// and `export_image_estimate` take, read from one `Dictionary`:
///
/// ```text
/// { width: 16384,                      # required, a BAKE_WIDTHS rung
///   format: "png" | "bigtiff",         # default "png"
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
    pub format: StreamFormat,
    pub style: ExportStyle,
    pub content: ExportContent,
}

impl ExportOptions {
    pub const KEYS: [&'static str; 4] = ["width", "format", "style", "content"];
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

pub fn look_from_name(name: &str) -> Option<&'static str> {
    render::LOOK_PRESETS.iter().copied().find(|n| n.eq_ignore_ascii_case(name))
}

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

pub fn parse_appearance_preset(text: &str) -> Result<TerrainAppearance, String> {
    let doc: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("parse failed: {e}"))?;
    if doc.get("format").and_then(|v| v.as_str()) != Some("cartalith-appearance") {
        return Err("not a Cartalith appearance preset".into());
    }
    let body = doc.get("appearance").ok_or("no appearance block")?;
    serde_json::from_value::<TerrainAppearance>(body.clone()).map_err(|e| format!("decode failed: {e}"))
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
