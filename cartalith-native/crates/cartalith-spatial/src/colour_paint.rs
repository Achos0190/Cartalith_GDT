//! The non-destructive colour-paint layer (`COLOUR_PAINT_SCOPE.md`, milestone
//! CP-1; owner Ruling CN, 2026-10-18).
//!
//! # What it is
//!
//! A list of **marks** -- brush strokes (a polyline with a radius) and filled
//! areas (a closed ring) -- whose every coordinate is in **map kilometres**,
//! each carrying a colour, an opacity, a hardness and a blend mode. The
//! renderer asks this layer one question per output pixel: *"given this
//! colour at this kilometre position, with pixels this many kilometres
//! apart, what colour is it after the paint?"* ([`ColourPaint::apply`]).
//!
//! # Why vector and why kilometres
//!
//! The owner's hard requirement of 2026-10-17 is that zoom gains detail at
//! every level. A raster paint layer has a resolution, and past it a stroke
//! edge turns into blocks; a stroke stored as kilometre geometry is evaluated
//! analytically at whatever resolution asks (the 1-pixel-per-cell screen
//! texture, a deep-zoom tile at 30 pixels per cell, a 16K export), so its edge
//! is one output pixel wide at every one of them. The same kilometre means
//! the same ground whatever the grid size, so a 512 and a 2048 world of the
//! same map width draw a mark over the same place.
//!
//! # What it must never do
//!
//! - **Never touch world data or generation.** This module has no access to
//!   any field; it only recolours a colour it is handed. CARTO is
//!   presentation (`DCC_SHELL_SPEC.md` §4.5.2); WORLD paint stays WORLD.
//! - **Never encode "no paint" as a colour.** A pixel no mark reaches returns
//!   its input unchanged by control flow ([`ColourPaint::apply`] returns
//!   before any arithmetic), so an absent, hidden or empty layer is the
//!   unpainted picture bit for bit, not "nearly".
//! - **Never let a mark's cost be unbounded.** Every mark is checked against
//!   [`MAX_MARKS`], [`MAX_POINTS_PER_MARK`] and [`MAX_TOTAL_POINTS`] when the
//!   layer is built, and a query only visits the marks registered in the one
//!   bucket of [`BucketIndex`] its position falls in.
//!
//! # Provenance
//!
//! New capability, not a port: the reference has no colour paint. The blend
//! formulas are the W3C *Compositing and Blending Level 1* definitions (the
//! separable `multiply` mode; the non-separable `color` mode with its `Lum`,
//! `SetLum` and `ClipColor` helpers), evaluated on this renderer's own 0-255
//! channels. Every other constant is a
//! labelled judgement, named where it is defined.

use serde::{Deserialize, Serialize};

pub mod material;
pub use material::{EmissiveMaterial, GlowRamp, ProceduralLava};

/// Most marks one layer may hold. A labelled judgement sized from the scope's
/// cost model (`COLOUR_PAINT_SCOPE.md` §10, and §9 for the save size): 5 000 marks at the measured save
/// size and index build time are comfortably inside the bars; a layer that
/// wants more is a raster tool, which this deliberately is not.
pub const MAX_MARKS: usize = 5_000;

/// Most points one mark may carry. The shell simplifies a stroke before
/// committing it (`COLOUR_PAINT_SCOPE.md` CP-2), so this is a guard against
/// a runaway producer, not a figure a hand stroke reaches. Labelled judgement.
pub const MAX_POINTS_PER_MARK: usize = 4_096;

/// Most points across every mark of a layer -- the bound the index's memory
/// and build time are sized against. Labelled judgement (the scope's §10).
pub const MAX_TOTAL_POINTS: usize = 250_000;

/// Smallest radius (strokes) or edge width (areas) accepted, in km: one metre.
/// Below it a mark is narrower than any output pixel this product draws (a
/// 16K export of a 100 km map is 6 m a pixel), so it would only ever be
/// antialiasing. Labelled judgement.
pub const MIN_RADIUS_KM: f64 = 0.001;

/// The bucket grid's resolution along each axis. A labelled judgement: 128 x
/// 128 buckets over the marks' bounding box keeps a bucket a small fraction of
/// a typical mark at map scale while costing at most 16 384 bucket headers.
pub const BUCKETS_PER_AXIS: usize = 128;

/// How a mark's colour combines with the map colour under it.
///
/// Applied by the renderer to the **material** colour, before the light
/// (`render.rs::land_color`), so every mode keeps the relief shading: the
/// hillshade, ambient occlusion and the look's own light multiply the painted
/// colour exactly as they multiply the generated one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColourBlend {
    /// The W3C `color` mode: the paint's hue and saturation, the map's own
    /// luminance. Keeps the material's texture (grain, micro-tone) as well
    /// as the light, so it is the gentlest; on a very light or very dark
    /// material it cannot reach a saturated colour (a red over snow is pink),
    /// which is what [`ColourBlend::Replace`] is for.
    Colour,
    /// The paint's colour replaces the material colour. The light still
    /// shades it (it is applied before the light), so a red crater interior
    /// still reads as a crater; the material's own texture is replaced.
    Replace,
    /// `map * paint / 255` per channel: darkens toward the paint, never
    /// lightens. Useful for staining; a pure red over grey rock is a dark red.
    Multiply,
}

impl ColourBlend {
    /// Every mode, in the order a picker lists them.
    pub const ALL: [ColourBlend; 3] = [ColourBlend::Colour, ColourBlend::Replace, ColourBlend::Multiply];

    /// The stable key a save file and the shell name the mode by.
    pub fn key(self) -> &'static str {
        match self {
            ColourBlend::Colour => "colour",
            ColourBlend::Replace => "replace",
            ColourBlend::Multiply => "multiply",
        }
    }

    /// The inverse of [`ColourBlend::key`]; `None` for an unknown key, so a
    /// typo is refused instead of silently becoming a default mode.
    pub fn from_key(key: &str) -> Option<ColourBlend> {
        ColourBlend::ALL.into_iter().find(|m| m.key() == key)
    }

    /// `backdrop` combined with `src` at `alpha` (`0..=1`), per channel on
    /// `0..=255` floats: `b + (B(b, s) - b) * alpha`, the same alpha fold the
    /// renderer's own layer stack uses. Nothing is clamped beyond what the
    /// W3C `ClipColor` step does for [`ColourBlend::Colour`]; the renderer
    /// clamps once, at its end.
    pub fn apply(self, backdrop: [f64; 3], src: [f64; 3], alpha: f64) -> [f64; 3] {
        let blended = match self {
            ColourBlend::Colour => set_lum(src, lum(backdrop)),
            ColourBlend::Replace => src,
            ColourBlend::Multiply => [backdrop[0] * src[0] / 255.0, backdrop[1] * src[1] / 255.0, backdrop[2] * src[2] / 255.0],
        };
        [
            backdrop[0] + (blended[0] - backdrop[0]) * alpha,
            backdrop[1] + (blended[1] - backdrop[1]) * alpha,
            backdrop[2] + (blended[2] - backdrop[2]) * alpha,
        ]
    }
}

/// W3C Compositing Level 1's `Lum(C) = 0.3 R + 0.59 G + 0.11 B` (the
/// non-separable blend modes' luminosity helper).
pub fn lum(c: [f64; 3]) -> f64 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

/// W3C `SetLum(C, l)`: shift `c` so its [`lum`] is `l`, then `ClipColor` it
/// back into `0..=255` while keeping that luminance (on this file's 0-255
/// scale, so the spec's `1` is `255`).
fn set_lum(c: [f64; 3], l: f64) -> [f64; 3] {
    let d = l - lum(c);
    let c = [c[0] + d, c[1] + d, c[2] + d];
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    // `ClipColor`, literally as the spec writes it: `n` and `x` are the
    // pre-clip extremes and both branches read them. `l - n > 0` whenever
    // `n < 0 <= l`, and `x - l > 0` whenever `x > 255 >= l`, so neither
    // division can be by zero for an in-range luminance; the guards keep a
    // backdrop outside 0..=255 (the light can exceed 255 elsewhere, never
    // here, where it has not been applied) from dividing by zero.
    let mut out = c;
    if n < 0.0 && l - n > 0.0 {
        for v in &mut out {
            *v = l + (*v - l) * l / (l - n);
        }
    }
    if x > 255.0 && x - l > 0.0 {
        for v in &mut out {
            *v = l + (*v - l) * (255.0 - l) / (x - l);
        }
    }
    out
}

/// A mark's geometry, in map kilometres from the map's top-left corner,
/// `x` east and `y` south (the grid's own axes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MarkShape {
    /// A brush stroke: the union of discs of `radius_km` swept along the
    /// polyline (round caps and joins). One point is a single dab. The
    /// stroke is one shape, so a stroke crossing itself does not paint
    /// twice -- opacity is applied once, as a stroke in a paint program is.
    Line { points_km: Vec<[f64; 2]>, radius_km: f64 },
    /// A filled region (the fill helper's output, `COLOUR_PAINT_SCOPE.md`
    /// CP-5): the even-odd interior of the closed ring (the last point joins
    /// the first), its edge softened over `edge_km` centred on the ring.
    Area { ring_km: Vec<[f64; 2]>, edge_km: f64 },
}

impl MarkShape {
    /// The shape's vertices.
    fn points(&self) -> &[[f64; 2]] {
        match self {
            MarkShape::Line { points_km, .. } => points_km,
            MarkShape::Area { ring_km, .. } => ring_km,
        }
    }
}

/// One mark: geometry, colour and how it composites.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColourMark {
    pub shape: MarkShape,
    /// sRGB 0-255, the colour the picker chose.
    pub rgb: [u8; 3],
    /// `0..=1`, clamped on build.
    pub opacity: f64,
    /// `0..=1`, clamped on build: the solid fraction of a stroke's radius.
    /// `1` is a hard edge (one output pixel of antialiasing at any zoom),
    /// `0` fades from the centre line out. Read by strokes only; an area's
    /// edge width is its own `edge_km`.
    pub hardness: f64,
    pub blend: ColourBlend,
    /// `true` (the default): the mark paints land only, and water (sea and
    /// lakes) under it keeps its colour. `false` paints water too -- a crater
    /// lake inside a painted volcano, the owner's case. Per mark, so one
    /// stroke can include a lake and the next leave it.
    #[serde(default = "default_true")]
    pub land_only: bool,
    /// An emissive **material** (lava and its relatives, [`material`]) in
    /// place of a flat colour; `None` is the flat colour CP-1 shipped. With
    /// a material, `rgb` is the crust colour and `blend` how the crust
    /// combines with the ground before the light; the glow is added after the
    /// light. Omitted from the saved JSON when `None`, so a flat mark's
    /// document is the CP-1 bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissive: Option<EmissiveMaterial>,
}

fn default_true() -> bool {
    true
}

/// The saved form: what `appearance_paint.json` holds (`COLOUR_PAINT_SCOPE.md`
/// §9). `version` is checked on load; an unknown version is refused rather
/// than guessed at.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColourPaintDoc {
    pub version: u32,
    pub visible: bool,
    pub marks: Vec<ColourMark>,
}

/// The single-layer document version (CP-1 … CP-3's `appearance_paint.json`).
/// [`ColourPaint::new`] takes only this version; [`PaintLayers`] reads it as
/// one layer named [`DEFAULT_LAYER_NAME`] and writes it back unchanged.
pub const DOC_VERSION: u32 = 1;

/// The layered document version (Ruling CS, 2026-10-19: *"use paint layers"*).
/// Written only when the layers cannot be said in version 1 (see
/// [`PaintLayers::is_v1_shaped`]), so a project painted on one default layer
/// still saves the bytes an older build reads.
pub const DOC_VERSION_LAYERS: u32 = 2;

/// Most paint layers one document may hold. A labelled judgement: a layers
/// list on a phone is one 44 px row a layer, and 32 is far past what an
/// annotation layer needs while keeping the list a short scroll. The mark
/// caps ([`MAX_MARKS`], [`MAX_TOTAL_POINTS`]) are counted across **every**
/// layer, hidden ones included, so layers never raise the cost bound.
pub const MAX_LAYERS: usize = 32;

/// Longest layer name, in characters. A labelled judgement: a name is a row
/// label in a dock, not prose; longer is refused, not truncated.
pub const MAX_LAYER_NAME_CHARS: usize = 64;

/// The name a version-1 document's one layer is read under, and the first
/// name [`PaintLayers::add_layer`] picks.
pub const DEFAULT_LAYER_NAME: &str = "Paint 1";

/// One named paint layer: its own marks, composited in mark order, with its
/// own visibility and lock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaintLayer {
    /// Shown in the layers list; 1 ..= [`MAX_LAYER_NAME_CHARS`] characters,
    /// not blank. Names need not be unique (a label, never a key).
    pub name: String,
    /// `false`: the layer's marks are not composited at all -- they are left
    /// out of the render list by control flow ([`ColourPaint::from_layers`]),
    /// so a hidden layer draws the unpainted bytes, not "nearly".
    pub visible: bool,
    /// `true`: the layer's marks cannot be added to, cleared or deleted (its
    /// name, order and visibility still can). Missing reads as `false`.
    #[serde(default)]
    pub locked: bool,
    pub marks: Vec<ColourMark>,
}

/// The edited and saved colour-paint document: a master visibility and the
/// layers, **bottom first** (index 0 is composited first; a later layer's
/// marks go over an earlier one's, exactly as a later mark goes over an
/// earlier one inside a layer).
///
/// # Wire format
///
/// Serialises as version 1 (`{"version":1,"visible":…,"marks":[…]}`, the
/// CP-3 bytes) when [`PaintLayers::is_v1_shaped`] holds, else as version 2
/// (`{"version":2,"visible":…,"layers":[{"name","visible","locked","marks"}]}`).
/// Reads either; a version-1 document becomes one visible, unlocked layer
/// named [`DEFAULT_LAYER_NAME`], so opening an old project and saving it
/// unchanged writes the same bytes. Any other version, or a version without
/// its own member (`marks` for 1, `layers` for 2), is refused, never guessed.
///
/// # What it must never do
///
/// Never validate marks itself: the caps and value checks are
/// [`ColourPaint::from_layers`]'s, so there is one rule set, not two.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PaintLayersIn", into = "PaintLayersOut")]
pub struct PaintLayers {
    /// The master switch (the tool's "Hide colour paint"): `false` hides every
    /// layer without touching their own flags.
    pub visible: bool,
    pub layers: Vec<PaintLayer>,
}

/// What either document version deserialises through (see [`PaintLayers`]).
#[derive(Deserialize)]
struct PaintLayersIn {
    version: u32,
    visible: bool,
    #[serde(default)]
    marks: Option<Vec<ColourMark>>,
    #[serde(default)]
    layers: Option<Vec<PaintLayer>>,
}

impl TryFrom<PaintLayersIn> for PaintLayers {
    type Error = String;
    fn try_from(w: PaintLayersIn) -> Result<PaintLayers, String> {
        match w.version {
            DOC_VERSION => {
                let marks = w.marks.ok_or("a version-1 colour-paint document has no `marks`")?;
                Ok(PaintLayers::from_v1(ColourPaintDoc { version: DOC_VERSION, visible: w.visible, marks }))
            }
            DOC_VERSION_LAYERS => {
                let layers = w.layers.ok_or("a version-2 colour-paint document has no `layers`")?;
                Ok(PaintLayers { visible: w.visible, layers })
            }
            v => Err(ColourPaintError::UnknownVersion(v).to_string()),
        }
    }
}

/// The version-2 wire form (field order is the written order).
#[derive(Serialize)]
struct LayersV2 {
    version: u32,
    visible: bool,
    layers: Vec<PaintLayer>,
}

/// What [`PaintLayers`] serialises as: version 1 when it can be, else 2.
#[derive(Serialize)]
#[serde(untagged)]
enum PaintLayersOut {
    V1(ColourPaintDoc),
    V2(LayersV2),
}

impl From<PaintLayers> for PaintLayersOut {
    fn from(l: PaintLayers) -> PaintLayersOut {
        if l.is_v1_shaped() {
            let visible = l.visible;
            let marks = l.layers.into_iter().next().map(|x| x.marks).unwrap_or_default();
            PaintLayersOut::V1(ColourPaintDoc { version: DOC_VERSION, visible, marks })
        } else {
            PaintLayersOut::V2(LayersV2 { version: DOC_VERSION_LAYERS, visible: l.visible, layers: l.layers })
        }
    }
}

impl PaintLayers {
    /// No layers, visible: the state before the first stroke.
    pub fn empty() -> PaintLayers {
        PaintLayers { visible: true, layers: Vec::new() }
    }

    /// A version-1 document as one visible, unlocked layer named
    /// [`DEFAULT_LAYER_NAME`]; its `visible` becomes the master switch, which
    /// is what it switched in version 1.
    pub fn from_v1(doc: ColourPaintDoc) -> PaintLayers {
        PaintLayers { visible: doc.visible, layers: vec![PaintLayer { name: DEFAULT_LAYER_NAME.to_string(), visible: true, locked: false, marks: doc.marks }] }
    }

    /// Whether version 1 says this document exactly: one layer, named
    /// [`DEFAULT_LAYER_NAME`], visible and unlocked. Anything a version-1
    /// reader would lose (a second layer, a name, a per-layer hide, a lock)
    /// makes it version 2.
    pub fn is_v1_shaped(&self) -> bool {
        matches!(self.layers.as_slice(), [l] if l.name == DEFAULT_LAYER_NAME && l.visible && !l.locked)
    }

    /// Marks across every layer, hidden ones included (what a save holds and
    /// what [`MAX_MARKS`] bounds).
    pub fn total_marks(&self) -> usize {
        self.layers.iter().map(|l| l.marks.len()).sum()
    }

    /// The marks that composite, in order: every visible layer's, bottom
    /// layer first, as the version-1 document the renderer builds from. With
    /// the master switch off the list still holds them and `visible` is
    /// `false` (the layer is then inert, exactly as a hidden version-1 one).
    pub fn flattened(&self) -> ColourPaintDoc {
        let marks = self.layers.iter().filter(|l| l.visible).flat_map(|l| l.marks.iter().cloned()).collect();
        ColourPaintDoc { version: DOC_VERSION, visible: self.visible, marks }
    }

    /// The first `Paint N` (N from 1) no layer is named.
    fn next_default_name(&self) -> String {
        (1..).map(|n| format!("Paint {n}")).find(|s| self.layers.iter().all(|l| &l.name != s)).unwrap_or_default()
    }

    /// `Err` with why when `i` is not a layer.
    fn check_index(&self, i: usize) -> Result<(), String> {
        if i < self.layers.len() {
            Ok(())
        } else {
            Err(format!("there is no layer {} (the document has {})", i + 1, self.layers.len()))
        }
    }

    /// `Err` with why when layer `i` is locked (its marks cannot change).
    fn check_unlocked(&self, i: usize) -> Result<(), String> {
        self.check_index(i)?;
        if self.layers[i].locked {
            Err(format!("layer \u{201c}{}\u{201d} is locked", self.layers[i].name))
        } else {
            Ok(())
        }
    }

    /// Adds an empty, visible, unlocked layer **on top** and returns its
    /// index. `name` blank or `None` picks the next free `Paint N`. Refused
    /// past [`MAX_LAYERS`] or with a name over [`MAX_LAYER_NAME_CHARS`].
    pub fn add_layer(&mut self, name: Option<&str>) -> Result<usize, String> {
        if self.layers.len() >= MAX_LAYERS {
            return Err(ColourPaintError::TooManyLayers(self.layers.len() + 1).to_string());
        }
        let name = match name.map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => checked_name(s)?,
            None => self.next_default_name(),
        };
        self.layers.push(PaintLayer { name, visible: true, locked: false, marks: Vec::new() });
        Ok(self.layers.len() - 1)
    }

    /// Appends `mark` to layer `i`. With no layers at all and `i == 0`, first
    /// adds the default layer, so the first stroke needs no "new layer" step.
    /// Refused on a locked layer.
    pub fn add_mark(&mut self, i: usize, mark: ColourMark) -> Result<(), String> {
        if self.layers.is_empty() && i == 0 {
            self.add_layer(None)?;
        }
        self.check_unlocked(i)?;
        self.layers[i].marks.push(mark);
        Ok(())
    }

    /// Renames layer `i` (trimmed; blank refused). Allowed on a locked layer:
    /// the lock protects marks, not the label.
    pub fn rename(&mut self, i: usize, name: &str) -> Result<(), String> {
        self.check_index(i)?;
        let name = name.trim();
        if name.is_empty() {
            return Err("a layer name cannot be blank".to_string());
        }
        self.layers[i].name = checked_name(name)?;
        Ok(())
    }

    /// Moves layer `from` to index `to` (0 = bottom), shifting the layers in
    /// between. Changes the compositing order, so it changes the picture.
    pub fn move_layer(&mut self, from: usize, to: usize) -> Result<(), String> {
        self.check_index(from)?;
        self.check_index(to)?;
        let l = self.layers.remove(from);
        self.layers.insert(to, l);
        Ok(())
    }

    /// Shows or hides layer `i`.
    pub fn set_layer_visible(&mut self, i: usize, visible: bool) -> Result<(), String> {
        self.check_index(i)?;
        self.layers[i].visible = visible;
        Ok(())
    }

    /// Locks or unlocks layer `i`.
    pub fn set_layer_locked(&mut self, i: usize, locked: bool) -> Result<(), String> {
        self.check_index(i)?;
        self.layers[i].locked = locked;
        Ok(())
    }

    /// Removes layer `i` and its marks. Refused on a locked layer.
    pub fn delete_layer(&mut self, i: usize) -> Result<(), String> {
        self.check_unlocked(i)?;
        self.layers.remove(i);
        Ok(())
    }

    /// Empties layer `i`'s marks, keeping the layer. Refused on a locked layer.
    pub fn clear_layer(&mut self, i: usize) -> Result<(), String> {
        self.check_unlocked(i)?;
        self.layers[i].marks.clear();
        Ok(())
    }
}

/// `name` if it is within [`MAX_LAYER_NAME_CHARS`], else why not.
fn checked_name(name: &str) -> Result<String, String> {
    let n = name.chars().count();
    if n > MAX_LAYER_NAME_CHARS {
        Err(format!("a layer name of {n} characters is longer than {MAX_LAYER_NAME_CHARS}"))
    } else {
        Ok(name.to_string())
    }
}

/// Why a layer was refused. A refused layer changes nothing.
#[derive(Clone, Debug, PartialEq)]
pub enum ColourPaintError {
    TooManyMarks(usize),
    TooManyPoints { mark: usize, points: usize },
    TooManyTotalPoints(usize),
    /// A mark with no points, or an area with fewer than three.
    TooFewPoints(usize),
    /// A coordinate, radius, edge, opacity or hardness that is not finite.
    NotFinite(usize),
    /// A radius or edge width below [`MIN_RADIUS_KM`] (strokes need a
    /// radius; an area's edge may be any width at or above it).
    BadRadius(usize),
    UnknownVersion(u32),
    /// More than [`MAX_LAYERS`] layers.
    TooManyLayers(usize),
    /// Layer `i`'s name is blank or longer than [`MAX_LAYER_NAME_CHARS`].
    BadLayerName(usize),
}

impl std::fmt::Display for ColourPaintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColourPaintError::TooManyMarks(n) => write!(f, "{n} marks (at most {MAX_MARKS})"),
            ColourPaintError::TooManyPoints { mark, points } => write!(f, "mark {mark} has {points} points (at most {MAX_POINTS_PER_MARK})"),
            ColourPaintError::TooManyTotalPoints(n) => write!(f, "{n} points in all (at most {MAX_TOTAL_POINTS})"),
            ColourPaintError::TooFewPoints(i) => write!(f, "mark {i} has too few points"),
            ColourPaintError::NotFinite(i) => write!(f, "mark {i} has a value that is not finite"),
            ColourPaintError::BadRadius(i) => write!(f, "mark {i}'s radius or edge is below {MIN_RADIUS_KM} km"),
            ColourPaintError::UnknownVersion(v) => write!(f, "colour paint version {v} is not one this build reads ({DOC_VERSION} or {DOC_VERSION_LAYERS})"),
            ColourPaintError::TooManyLayers(n) => write!(f, "{n} paint layers (at most {MAX_LAYERS})"),
            ColourPaintError::BadLayerName(i) => write!(f, "layer {}'s name is blank or longer than {MAX_LAYER_NAME_CHARS} characters", i + 1),
        }
    }
}

/// One mark as the renderer evaluates it: validated, clamped, with its
/// colour in float channels and its edge profile resolved.
#[derive(Clone, Debug)]
struct Prepared {
    shape: MarkShape,
    rgb: [f64; 3],
    opacity: f64,
    blend: ColourBlend,
    land_only: bool,
    /// The soft band's width in km (strokes: `radius * (1 - hardness)`;
    /// areas: `edge_km`).
    band: f64,
    /// The signed distance, in km, at which coverage is one half. Strokes
    /// measure `radius - distance to the centre line` (zero at the radius),
    /// so theirs is `band / 2` (half-way into the soft band; `0`, the
    /// radius itself, for a hard stroke); areas `0` (the ring).
    half: f64,
    /// Bounding box `[x0, y0, x1, y1]` of the mark's ink in km.
    bbox: [f64; 4],
    /// The resolved emissive material, or `None` for a flat colour.
    emissive: Option<material::Resolved>,
    /// A stroke's cumulative length at each point, km (for `heat_along`);
    /// empty when nothing reads it.
    cum: Vec<f64>,
}

/// One mark's entry in one bucket.
#[derive(Clone, Debug)]
struct Entry {
    mark: u32,
    /// For an area: whether the bucket's centre is inside the ring. The
    /// inside test of a pixel in the bucket starts from here and counts only
    /// the edges listed in `segs`, which are every edge that touches the
    /// bucket -- so it is exact without walking the whole ring.
    centre_inside: bool,
    /// Range into [`BucketIndex::segs`]: the segment indices of this mark
    /// (segment `k` joins point `k` and `k + 1`, wrapping for an area) whose
    /// expanded bounding box overlaps the bucket.
    segs: (u32, u32),
}

/// The spatial index: a uniform grid of buckets over the marks' bounding box,
/// each listing, in mark order, the marks that can reach it and which of
/// their segments are near.
///
/// "Can reach" is decided with every segment's bounding box expanded by the
/// mark's ink extent plus `margin_km`, the widest antialiasing footprint the
/// index answers exactly. A query with a wider footprint (a view zoomed out
/// past the grid) falls back to visiting every mark ([`ColourPaint::apply`]),
/// so the answer is never wrong, only slower where it is rare.
#[derive(Clone, Debug)]
struct BucketIndex {
    x0: f64,
    y0: f64,
    bw: f64,
    bh: f64,
    nx: usize,
    ny: usize,
    margin_km: f64,
    /// `buckets[k]` = range into `entries`.
    buckets: Vec<(u32, u32)>,
    entries: Vec<Entry>,
    segs: Vec<u32>,
}

/// The layer the renderer reads. Immutable once built: every edit builds a
/// new one (the shell's undo holds whole documents, `COLOUR_PAINT_SCOPE.md`
/// §8), so a render in flight never sees a half-edited layer.
///
/// Built from [`PaintLayers`] ([`ColourPaint::from_layers`]): every mark of
/// every layer is validated, but only the visible layers' marks, bottom layer
/// first, are prepared into the render list and the index. That flattened
/// list is the whole of what the renderer sees, so "composited in layer
/// order" is mark order by construction and the screen, tiles and both export
/// paths (which all call [`ColourPaint::apply_lit`]) cannot disagree about it.
#[derive(Clone, Debug)]
pub struct ColourPaint {
    /// The edited document, every layer (what a save writes).
    layers: PaintLayers,
    /// The flattened visible marks (what the renderer composites, and what
    /// the tile fingerprint hashes).
    doc: ColourPaintDoc,
    marks: Vec<Prepared>,
    index: Option<BucketIndex>,
    any_water: bool,
    any_emissive: bool,
}

/// The renderer's per-pixel query: where, how wide a pixel is, and whether
/// the pixel is water (which only `land_only == false` marks paint).
#[derive(Clone, Copy, Debug)]
pub struct PaintQuery {
    pub x_km: f64,
    pub y_km: f64,
    /// The distance in km between neighbouring output pixels: the
    /// antialiasing width. Strokes are blurred by at most this much.
    pub px_km: f64,
    pub water: bool,
}

impl ColourPaint {
    /// Builds a layer from its document, refusing it whole if any mark breaks
    /// a cap or carries a non-finite value. `margin_km` is the widest pixel
    /// footprint the index serves exactly (the renderer passes two grid cells
    /// in km; see [`BucketIndex`]).
    pub fn new(doc: ColourPaintDoc, margin_km: f64) -> Result<ColourPaint, ColourPaintError> {
        if doc.version != DOC_VERSION {
            return Err(ColourPaintError::UnknownVersion(doc.version));
        }
        ColourPaint::from_layers(PaintLayers::from_v1(doc), margin_km)
    }

    /// Builds a layer from a layered document (Ruling CS), refusing it whole
    /// if it has more than [`MAX_LAYERS`] layers, a blank or over-long name,
    /// or if any mark of **any** layer -- hidden and locked ones included --
    /// breaks a cap or carries a non-finite value: [`MAX_MARKS`] and
    /// [`MAX_TOTAL_POINTS`] are totals over every layer, so hiding a layer
    /// never admits more than the cost model was sized for. A mark's index in
    /// an error counts across the layers, bottom first.
    pub fn from_layers(layers: PaintLayers, margin_km: f64) -> Result<ColourPaint, ColourPaintError> {
        if layers.layers.len() > MAX_LAYERS {
            return Err(ColourPaintError::TooManyLayers(layers.layers.len()));
        }
        for (i, l) in layers.layers.iter().enumerate() {
            if l.name.trim().is_empty() || l.name.chars().count() > MAX_LAYER_NAME_CHARS {
                return Err(ColourPaintError::BadLayerName(i));
            }
        }
        let n_marks = layers.total_marks();
        if n_marks > MAX_MARKS {
            return Err(ColourPaintError::TooManyMarks(n_marks));
        }
        let total: usize = layers.layers.iter().flat_map(|l| &l.marks).map(|m| m.shape.points().len()).sum();
        if total > MAX_TOTAL_POINTS {
            return Err(ColourPaintError::TooManyTotalPoints(total));
        }
        let mut marks = Vec::with_capacity(n_marks);
        let mut i = 0;
        for l in &layers.layers {
            for m in &l.marks {
                // Validated whatever the layer's visibility; kept only when
                // it composites (a hidden layer is not attached at all).
                let p = prepare(i, m)?;
                if l.visible {
                    marks.push(p);
                }
                i += 1;
            }
        }
        let doc = layers.flattened();
        let margin_km = if margin_km.is_finite() && margin_km > 0.0 { margin_km } else { 0.0 };
        let index = if marks.is_empty() { None } else { Some(build_index(&marks, margin_km)) };
        let any_water = marks.iter().any(|m| !m.land_only);
        let any_emissive = marks.iter().any(|m| m.emissive.is_some());
        Ok(ColourPaint { layers, doc, marks, index, any_water, any_emissive })
    }

    /// The marks the renderer composites, as a version-1 document: every
    /// visible layer's marks, bottom layer first ([`PaintLayers::flattened`]).
    /// For a layer built by [`ColourPaint::new`] this is the document it was
    /// built from. Not what a save writes: that is [`ColourPaint::layers`].
    pub fn doc(&self) -> &ColourPaintDoc {
        &self.doc
    }

    /// The layered document, every layer (what a save writes and the shell's
    /// undo holds).
    pub fn layers(&self) -> &PaintLayers {
        &self.layers
    }

    /// `true` when nothing can be painted: the master switch off, or no mark
    /// in any visible layer. The renderer tests this once per render and
    /// attaches nothing when it holds.
    pub fn is_inert(&self) -> bool {
        !self.doc.visible || self.marks.is_empty()
    }

    /// Whether any mark paints water. When `false` a water pixel never asks.
    pub fn paints_water(&self) -> bool {
        self.any_water && !self.is_inert()
    }

    /// Number of marks in every layer, hidden ones included (what a save
    /// holds). [`ColourPaint::drawn_len`] counts the ones that composite.
    pub fn len(&self) -> usize {
        self.layers.total_marks()
    }

    /// Number of marks that composite: those of visible layers.
    pub fn drawn_len(&self) -> usize {
        self.marks.len()
    }

    /// `true` with no marks in any layer (a save then writes nothing).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `c` after every mark that reaches `q`, in mark order (later marks
    /// over earlier ones). Returns `c` itself -- the same bits -- when no
    /// mark reaches, when the layer is inert, and for a water pixel no
    /// water-painting mark covers. An emissive mark contributes its crust
    /// here and its glow is dropped: the renderer calls
    /// [`ColourPaint::apply_lit`], which returns the glow as well.
    pub fn apply(&self, c: [f64; 3], q: PaintQuery) -> [f64; 3] {
        self.apply_lit(c, q).0
    }

    /// Whether any mark carries an emissive material.
    pub fn has_emissive(&self) -> bool {
        self.any_emissive && !self.is_inert()
    }

    /// The material colour after the marks (as [`ColourPaint::apply`]) and
    /// the **emissive** colour (0-255 per channel) the renderer adds after
    /// its light: the glow of every emissive mark reaching `q`, halos
    /// included, each later mark covering the pixel at alpha `a` scaling the
    /// glow beneath it by `1 - a` (a flat mark painted over lava hides the
    /// glow it covers). The emissive part is exactly `[0.0; 3]` when no
    /// emissive mark reaches.
    pub fn apply_lit(&self, c: [f64; 3], q: PaintQuery) -> ([f64; 3], [f64; 3]) {
        let mut emis = [0.0; 3];
        if self.is_inert() || (q.water && !self.any_water) {
            return (c, emis);
        }
        let Some(ix) = self.index.as_ref() else { return (c, emis) };
        let p = [q.x_km, q.y_km];
        let px = if q.px_km.is_finite() && q.px_km > 0.0 { q.px_km } else { 0.0 };
        let mut out = c;
        let mut group = LavaGroup::default();
        if px > ix.margin_km {
            // Wider than the index answers exactly: visit every mark.
            for m in &self.marks {
                if q.water && m.land_only {
                    continue;
                }
                let ext = ink_extent(m) + px;
                if p[0] < m.bbox[0] - ext || p[0] > m.bbox[2] + ext || p[1] < m.bbox[1] - ext || p[1] > m.bbox[3] + ext {
                    continue;
                }
                let s = signed_distance_full(m, p);
                composite(m, p, s, || along(m, p, 0..segment_count(m)), px, &mut out, &mut emis, &mut group);
            }
            group.flush(p, px, &mut out, &mut emis);
            return (out, emis);
        }
        let bx = ((p[0] - ix.x0) / ix.bw).floor();
        let by = ((p[1] - ix.y0) / ix.bh).floor();
        if bx < 0.0 || by < 0.0 || bx >= ix.nx as f64 || by >= ix.ny as f64 {
            return (c, emis);
        }
        let (bx, by) = (bx as usize, by as usize);
        let (e0, e1) = ix.buckets[by * ix.nx + bx];
        if e0 == e1 {
            return (c, emis);
        }
        let centre = [ix.x0 + (bx as f64 + 0.5) * ix.bw, ix.y0 + (by as f64 + 0.5) * ix.bh];
        for e in &ix.entries[e0 as usize..e1 as usize] {
            let m = &self.marks[e.mark as usize];
            if q.water && m.land_only {
                continue;
            }
            let segs = &ix.segs[e.segs.0 as usize..e.segs.1 as usize];
            let s = signed_distance_local(m, p, centre, e.centre_inside, segs);
            composite(m, p, s, || along(m, p, segs.iter().map(|&k| k as usize)), px, &mut out, &mut emis, &mut group);
        }
        group.flush(p, px, &mut out, &mut emis);
        (out, emis)
    }

    /// Diagnostics for the cost model: (buckets, entries, segment refs).
    pub fn index_stats(&self) -> (usize, usize, usize) {
        self.index.as_ref().map_or((0, 0, 0), |ix| (ix.buckets.len(), ix.entries.len(), ix.segs.len()))
    }
}

/// Validates one mark and resolves its profile. `i` is its index, for the error.
fn prepare(i: usize, m: &ColourMark) -> Result<Prepared, ColourPaintError> {
    let pts = m.shape.points();
    if pts.len() > MAX_POINTS_PER_MARK {
        return Err(ColourPaintError::TooManyPoints { mark: i, points: pts.len() });
    }
    let need = match m.shape {
        MarkShape::Line { .. } => 1,
        MarkShape::Area { .. } => 3,
    };
    if pts.len() < need {
        return Err(ColourPaintError::TooFewPoints(i));
    }
    if pts.iter().any(|p| !p[0].is_finite() || !p[1].is_finite()) || !m.opacity.is_finite() || !m.hardness.is_finite() {
        return Err(ColourPaintError::NotFinite(i));
    }
    let opacity = m.opacity.clamp(0.0, 1.0);
    let hardness = m.hardness.clamp(0.0, 1.0);
    let (band, half) = match &m.shape {
        MarkShape::Line { radius_km, .. } => {
            if !radius_km.is_finite() {
                return Err(ColourPaintError::NotFinite(i));
            }
            if *radius_km < MIN_RADIUS_KM {
                return Err(ColourPaintError::BadRadius(i));
            }
            let band = radius_km * (1.0 - hardness);
            (band, band / 2.0)
        }
        MarkShape::Area { edge_km, .. } => {
            if !edge_km.is_finite() {
                return Err(ColourPaintError::NotFinite(i));
            }
            if *edge_km < 0.0 {
                return Err(ColourPaintError::BadRadius(i));
            }
            (*edge_km, 0.0)
        }
    };
    let mut bbox = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for p in pts {
        bbox[0] = bbox[0].min(p[0]);
        bbox[1] = bbox[1].min(p[1]);
        bbox[2] = bbox[2].max(p[0]);
        bbox[3] = bbox[3].max(p[1]);
    }
    let rgb = [m.rgb[0] as f64, m.rgb[1] as f64, m.rgb[2] as f64];
    let (emissive, cum) = match &m.emissive {
        None => (None, Vec::new()),
        Some(e) => {
            // The interior depth the temperature rises over: a stroke's
            // radius; an area's equivalent radius (of the disc of its area)
            // halved, so a crater's hot core is its middle.
            let depth = match &m.shape {
                MarkShape::Line { radius_km, .. } => *radius_km,
                MarkShape::Area { ring_km, .. } => {
                    let n = ring_km.len();
                    let a2: f64 = (0..n).map(|k| ring_km[k][0] * ring_km[(k + 1) % n][1] - ring_km[(k + 1) % n][0] * ring_km[k][1]).sum();
                    0.5 * (0.5 * a2.abs() / std::f64::consts::PI).sqrt()
                }
            };
            let r = material::Resolved::new(e, depth).ok_or(ColourPaintError::NotFinite(i))?;
            let cum = match &m.shape {
                MarkShape::Line { points_km, .. } if r.heat_along > 0.0 => {
                    let mut c = Vec::with_capacity(points_km.len());
                    let mut acc = 0.0;
                    c.push(0.0);
                    for w in points_km.windows(2) {
                        acc += (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
                        c.push(acc);
                    }
                    c
                }
                _ => Vec::new(),
            };
            (Some(r), cum)
        }
    };
    Ok(Prepared { shape: m.shape.clone(), rgb, opacity, blend: m.blend, land_only: m.land_only, band, half, bbox, emissive, cum })
}

/// The emissive marks reaching one pixel since the last flat mark that
/// covered it: **one body of lava**. Overlapping lava strokes and fills merge
/// -- their coverage is the union, their temperature the hottest of their
/// [`material::Resolved::base_heat`]s, their crust and cracks sampled once
/// from the hottest mark's material -- so a stroke laid over a fill (or a
/// "grow" ring around it) adds lava instead of drawing its own cool rim
/// across the middle of a lake, which is what compositing them one by one
/// drew (the first lava frames, 2026-10-18).
///
/// A flat mark that covers the pixel ends the group: the group is flushed
/// under it, so later flat paint still covers lava (and its glow) in order.
/// A flat mark *between* two lava marks therefore sits under the second, a
/// deliberate simplification: lava over paint over lava is one lava body.
#[derive(Default)]
struct LavaGroup<'a> {
    /// `prod(1 - a_i)` over the group's covering marks.
    keep: f64,
    /// The hottest covering mark and its base heat.
    hottest: Option<(&'a Prepared, &'a material::Resolved, f64)>,
    /// Halo glow from marks reaching the pixel without covering it.
    halo: [f64; 3],
    any: bool,
}

impl<'a> LavaGroup<'a> {
    /// Composites the group into `out` and `emis` and empties it: the crust
    /// at the union's coverage, the glow over the glow beneath it, the halo
    /// where the group does not cover.
    fn flush(&mut self, p: [f64; 2], px: f64, out: &mut [f64; 3], emis: &mut [f64; 3]) {
        if !self.any {
            return;
        }
        let cover = 1.0 - self.keep;
        let mut e = [self.halo[0] * self.keep, self.halo[1] * self.keep, self.halo[2] * self.keep];
        if let Some((m, r, base)) = self.hottest {
            let (k, glow) = r.sample(p, base, px);
            let crust = [(m.rgb[0] * k).clamp(0.0, 255.0), (m.rgb[1] * k).clamp(0.0, 255.0), (m.rgb[2] * k).clamp(0.0, 255.0)];
            *out = m.blend.apply(*out, crust, cover);
            e = [e[0] + glow[0] * cover, e[1] + glow[1] * cover, e[2] + glow[2] * cover];
        }
        for (v, add) in emis.iter_mut().zip(e) {
            *v = *v * self.keep + add;
        }
        *self = LavaGroup::default();
    }
}

/// One mark's contribution to a pixel `s` km inside its edge: the flat
/// colour (CP-1's arithmetic, unchanged) into `out`, or an emissive mark into
/// the pixel's [`LavaGroup`]. `along` yields the pixel's fraction along a
/// stroke, evaluated only when the material reads it.
#[allow(clippy::too_many_arguments)]
fn composite<'a>(m: &'a Prepared, p: [f64; 2], s: f64, along: impl FnOnce() -> f64, px: f64, out: &mut [f64; 3], emis: &mut [f64; 3], group: &mut LavaGroup<'a>) {
    let a = coverage(s, m.half, m.band, px) * m.opacity;
    match &m.emissive {
        None => {
            if a > 0.0 {
                group.flush(p, px, out, emis);
                *out = m.blend.apply(*out, m.rgb, a);
                if *emis != [0.0; 3] {
                    // A flat mark over glow hides the glow it covers.
                    for v in emis.iter_mut() {
                        *v *= 1.0 - a;
                    }
                }
            }
        }
        Some(r) => {
            // The material's ragged edge: the signed distance moved by its
            // km-keyed lobes before anything reads it.
            let s = s + r.edge_offset(p, px);
            let a = coverage(s, m.half, m.band, px) * m.opacity;
            // km outside the half-coverage contour (negative inside).
            let outside = m.half - s;
            if a <= 0.0 && (outside <= 0.0 || outside >= r.glow_km) {
                return;
            }
            if !group.any {
                *group = LavaGroup { keep: 1.0, hottest: None, halo: [0.0; 3], any: true };
            }
            let u = if r.heat_along > 0.0 { along() } else { 0.0 };
            if a > 0.0 {
                group.keep *= 1.0 - a;
                let base = r.base_heat(-outside, u);
                if group.hottest.is_none_or(|h| base > h.2) {
                    group.hottest = Some((m, r, base));
                }
            }
            if a < m.opacity {
                // The halo where the mark does not fully cover: outside its
                // edge, falling to nothing at the glow radius.
                let h = r.halo(outside.max(1e-9), u);
                let w = m.opacity - a;
                for (v, x) in group.halo.iter_mut().zip(h) {
                    *v += x * w;
                }
            }
        }
    }
}

/// The fraction `0..=1` along a stroke of the point on it nearest `p`, over
/// the segments `segs` (the bucket's own list, or every segment); `0` when
/// the mark kept no lengths (an area, or a material that does not read it).
fn along(m: &Prepared, p: [f64; 2], segs: impl Iterator<Item = usize>) -> f64 {
    let total = m.cum.last().copied().unwrap_or(0.0);
    if total <= 0.0 {
        return 0.0;
    }
    let mut best = (f64::INFINITY, 0.0);
    for k in segs {
        let (a, b) = segment(m, k);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let d2 = seg_dist2(p, a, b);
        if d2 < best.0 {
            best = (d2, m.cum[k] + t * (m.cum[k + 1] - m.cum[k]));
        }
    }
    (best.1 / total).clamp(0.0, 1.0)
}

/// How far past its vertices' bounding box a mark's ink can reach, in km,
/// before antialiasing: a stroke's radius; half an area's edge band.
fn ink_extent(m: &Prepared) -> f64 {
    let base = match &m.shape {
        MarkShape::Line { radius_km, .. } => *radius_km,
        MarkShape::Area { .. } => m.band / 2.0,
    };
    // An emissive mark's halo reaches past its ink by its glow radius; a
    // flat mark's extent is CP-1's by control flow.
    match &m.emissive {
        Some(r) => base + r.glow_km + r.edge_reach(),
        None => base,
    }
}

/// Coverage `0..=1` for signed distance `s` (km, positive inside):
/// `smoothstep` of `0.5 + (s - half) / max(band, px)`.
///
/// The half-coverage contour sits at `s == half` whatever the zoom, so the
/// same kilometre footprint is drawn at every resolution; only the edge's
/// width changes, and it is never narrower than one output pixel (`px`),
/// which is the antialiasing. `max(band, px)` with both zero (a hard mark
/// queried with no footprint) is a hard step at the contour.
fn coverage(s: f64, half: f64, band: f64, px: f64) -> f64 {
    let w = band.max(px);
    if w <= 0.0 {
        return if s >= half { 1.0 } else { 0.0 };
    }
    let t = (0.5 + (s - half) / w).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Squared distance from `p` to segment `a`-`b`.
fn seg_dist2(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (a[0] + t * dx - p[0], a[1] + t * dy - p[1]);
    qx * qx + qy * qy
}

/// Segment `k` of a mark: points `k`, `k + 1` (a line's last point is a
/// zero-length segment when it has one point; an area wraps).
fn segment(m: &Prepared, k: usize) -> ([f64; 2], [f64; 2]) {
    let pts = m.shape.points();
    match m.shape {
        MarkShape::Line { .. } => {
            if pts.len() == 1 {
                (pts[0], pts[0])
            } else {
                (pts[k], pts[k + 1])
            }
        }
        MarkShape::Area { .. } => (pts[k], pts[(k + 1) % pts.len()]),
    }
}

/// Number of segments of a mark.
fn segment_count(m: &Prepared) -> usize {
    let n = m.shape.points().len();
    match m.shape {
        MarkShape::Line { .. } => n.saturating_sub(1).max(1),
        MarkShape::Area { .. } => n,
    }
}

/// Whether segment `p`-`q` properly crosses segment `a`-`b` (strict
/// orientation tests; a touching or collinear pair does not count). Used for
/// the local even-odd test, where the probability of a pixel lying exactly on
/// an edge's extension is negligible and a miss costs one pixel's parity.
fn crosses(p: [f64; 2], q: [f64; 2], a: [f64; 2], b: [f64; 2]) -> bool {
    let o = |u: [f64; 2], v: [f64; 2], w: [f64; 2]| (v[0] - u[0]) * (w[1] - u[1]) - (v[1] - u[1]) * (w[0] - u[0]);
    let (d1, d2) = (o(a, b, p), o(a, b, q));
    let (d3, d4) = (o(p, q, a), o(p, q, b));
    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

/// Even-odd inside test of `p` against an area's whole ring (a +x ray).
fn inside_ring(ring: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut inside = false;
    let n = ring.len();
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (ring[i], ring[j]);
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let x = a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]);
            if p[0] < x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Signed distance (km, positive inside) of `p` to a mark, over every
/// segment. The fallback path and the index builder's reference.
fn signed_distance_full(m: &Prepared, p: [f64; 2]) -> f64 {
    let mut d2 = f64::INFINITY;
    for k in 0..segment_count(m) {
        let (a, b) = segment(m, k);
        d2 = d2.min(seg_dist2(p, a, b));
    }
    let d = d2.sqrt();
    match &m.shape {
        MarkShape::Line { radius_km, .. } => radius_km - d,
        MarkShape::Area { ring_km, .. } => {
            if inside_ring(ring_km, p) {
                d
            } else {
                -d
            }
        }
    }
}

/// Signed distance from the bucket's own segment list. Exact wherever the
/// coverage is not already saturated: every segment within the mark's reach
/// of `p` is in `segs` (see [`BucketIndex`]); a segment that is not is
/// farther than the reach, where coverage is 0 (strokes) or decided by the
/// inside test alone (areas), and the distance is then reported as that reach.
fn signed_distance_local(m: &Prepared, p: [f64; 2], centre: [f64; 2], centre_inside: bool, segs: &[u32]) -> f64 {
    let mut d2 = f64::INFINITY;
    for &k in segs {
        let (a, b) = segment(m, k as usize);
        d2 = d2.min(seg_dist2(p, a, b));
    }
    match &m.shape {
        MarkShape::Line { radius_km, .. } => radius_km - d2.sqrt(),
        MarkShape::Area { .. } => {
            let mut inside = centre_inside;
            for &k in segs {
                let (a, b) = segment(m, k as usize);
                if crosses(centre, p, a, b) {
                    inside = !inside;
                }
            }
            // No listed segment: far from every edge, `d` is "at least the
            // reach"; infinity saturates the coverage on the right side.
            let d = d2.sqrt();
            if inside {
                d
            } else {
                -d
            }
        }
    }
}

/// Builds the bucket grid (see [`BucketIndex`]).
fn build_index(marks: &[Prepared], margin_km: f64) -> BucketIndex {
    let reach = |m: &Prepared| ink_extent(m) + m.band / 2.0 + margin_km;
    let mut bb = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for m in marks {
        let r = reach(m);
        bb[0] = bb[0].min(m.bbox[0] - r);
        bb[1] = bb[1].min(m.bbox[1] - r);
        bb[2] = bb[2].max(m.bbox[2] + r);
        bb[3] = bb[3].max(m.bbox[3] + r);
    }
    let (nx, ny) = (BUCKETS_PER_AXIS, BUCKETS_PER_AXIS);
    let bw = ((bb[2] - bb[0]) / nx as f64).max(1e-9);
    let bh = ((bb[3] - bb[1]) / ny as f64).max(1e-9);
    // Per bucket, per mark (in mark order): the near segments. Built as a
    // flat list of (bucket, mark, seg) then grouped, so memory is one pass.
    let mut triples: Vec<(u32, u32, u32)> = Vec::new();
    let bucket_range = |lo: f64, hi: f64, o: f64, s: f64, n: usize| -> (usize, usize) {
        let a = ((lo - o) / s).floor().max(0.0) as usize;
        let b = (((hi - o) / s).floor().max(0.0) as usize).min(n - 1);
        (a.min(n - 1), b)
    };
    // Areas also need every bucket their (reach-expanded) box covers, even
    // with no near edge, so the interior paints.
    let mut area_cover: Vec<(u32, u32)> = Vec::new();
    for (mi, m) in marks.iter().enumerate() {
        let r = reach(m);
        for k in 0..segment_count(m) {
            let (a, b) = segment(m, k);
            let (x0, x1) = bucket_range(a[0].min(b[0]) - r, a[0].max(b[0]) + r, bb[0], bw, nx);
            let (y0, y1) = bucket_range(a[1].min(b[1]) - r, a[1].max(b[1]) + r, bb[1], bh, ny);
            for by in y0..=y1 {
                for bx in x0..=x1 {
                    triples.push(((by * nx + bx) as u32, mi as u32, k as u32));
                }
            }
        }
        if matches!(m.shape, MarkShape::Area { .. }) {
            let (x0, x1) = bucket_range(m.bbox[0] - r, m.bbox[2] + r, bb[0], bw, nx);
            let (y0, y1) = bucket_range(m.bbox[1] - r, m.bbox[3] + r, bb[1], bh, ny);
            for by in y0..=y1 {
                for bx in x0..=x1 {
                    area_cover.push(((by * nx + bx) as u32, mi as u32));
                }
            }
        }
    }
    triples.sort_unstable();
    triples.dedup();
    area_cover.sort_unstable();
    area_cover.dedup();
    let mut buckets = vec![(0u32, 0u32); nx * ny];
    let mut entries: Vec<Entry> = Vec::new();
    let mut segs: Vec<u32> = Vec::new();
    let (mut ti, mut ai) = (0usize, 0usize);
    for (bk, slot) in buckets.iter_mut().enumerate() {
        let start = entries.len() as u32;
        let centre = [bb[0] + ((bk % nx) as f64 + 0.5) * bw, bb[1] + ((bk / nx) as f64 + 0.5) * bh];
        // Merge, in mark order, this bucket's segment triples and area covers.
        loop {
            let tm = triples.get(ti).filter(|t| t.0 as usize == bk).map(|t| t.1);
            let am = area_cover.get(ai).filter(|t| t.0 as usize == bk).map(|t| t.1);
            let mark = match (tm, am) {
                (None, None) => break,
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (Some(a), Some(b)) => a.min(b),
            };
            let s0 = segs.len() as u32;
            while let Some(t) = triples.get(ti).filter(|t| t.0 as usize == bk && t.1 == mark) {
                segs.push(t.2);
                ti += 1;
            }
            if am == Some(mark) {
                ai += 1;
            }
            let m = &marks[mark as usize];
            let centre_inside = match &m.shape {
                MarkShape::Area { ring_km, .. } => inside_ring(ring_km, centre),
                MarkShape::Line { .. } => false,
            };
            // A stroke with no near segment cannot reach this bucket.
            if matches!(m.shape, MarkShape::Line { .. }) && segs.len() as u32 == s0 {
                continue;
            }
            entries.push(Entry { mark, centre_inside, segs: (s0, segs.len() as u32) });
        }
        *slot = (start, entries.len() as u32);
    }
    BucketIndex { x0: bb[0], y0: bb[1], bw, bh, nx, ny, margin_km, buckets, entries, segs }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(pts: &[[f64; 2]], r: f64, rgb: [u8; 3], blend: ColourBlend) -> ColourMark {
        ColourMark { shape: MarkShape::Line { points_km: pts.to_vec(), radius_km: r }, rgb, opacity: 1.0, hardness: 1.0, blend, land_only: true, emissive: None }
    }

    fn layer(marks: Vec<ColourMark>) -> ColourPaint {
        ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks }, 1.0).unwrap()
    }

    fn q(x: f64, y: f64, px: f64) -> PaintQuery {
        PaintQuery { x_km: x, y_km: y, px_km: px, water: false }
    }

    const GREY: [f64; 3] = [128.0, 128.0, 128.0];

    /// Protects: the W3C formulas against literal values (not against
    /// themselves). Colour of red over mid grey keeps Lum 128 and is red;
    /// multiply of (255,0,0) over grey is (128,0,0); replace is the source.
    #[test]
    fn blend_formulas_match_literal_values() {
        let red = [255.0, 0.0, 0.0];
        let c = ColourBlend::Colour.apply(GREY, red, 1.0);
        assert!((lum(c) - 128.0).abs() < 1e-9, "colour keeps the backdrop's luminance: {c:?}");
        assert!(c[0] > 200.0 && c[1] < 100.0 && (c[1] - c[2]).abs() < 1e-9, "and is red: {c:?}");
        assert_eq!(ColourBlend::Multiply.apply(GREY, red, 1.0), [128.0, 0.0, 0.0]);
        assert_eq!(ColourBlend::Replace.apply(GREY, red, 1.0), red);
        assert_eq!(ColourBlend::Replace.apply(GREY, red, 0.5), [191.5, 64.0, 64.0]);
        // SetLum then ClipColor at a bright backdrop stays in range.
        let s = ColourBlend::Colour.apply([240.0, 240.0, 240.0], red, 1.0);
        assert!(s.iter().all(|v| (0.0..=255.0 + 1e-9).contains(v)), "{s:?}");
        assert!((lum(s) - 240.0).abs() < 1e-6, "{s:?}");
    }

    /// Protects: "no mark reaches" returns the input's exact bits.
    #[test]
    fn unreached_pixels_are_returned_bit_for_bit() {
        let l = layer(vec![line(&[[10.0, 10.0], [20.0, 10.0]], 1.0, [255, 0, 0], ColourBlend::Replace)]);
        let c = [12.345678, 98.7654321, 3.0e-7];
        assert_eq!(l.apply(c, q(50.0, 50.0, 0.1)), c);
        assert_eq!(l.apply(c, q(15.0, 13.0, 0.1)), c);
        // Hidden: same bits even on the stroke.
        let mut d = l.doc().clone();
        d.visible = false;
        let h = ColourPaint::new(d, 1.0).unwrap();
        assert!(h.is_inert());
        assert_eq!(h.apply(c, q(15.0, 10.0, 0.1)), c);
    }

    /// Protects: the half-coverage contour is at the radius whatever the
    /// pixel footprint (same km footprint at every zoom), and the edge is
    /// at most one pixel wide for a hard stroke.
    #[test]
    fn same_km_footprint_at_every_footprint() {
        let l = layer(vec![line(&[[0.0, 0.0], [100.0, 0.0]], 5.0, [0, 0, 0], ColourBlend::Replace)]);
        for px in [0.01, 0.1, 0.5, 0.9] {
            let at = |y: f64| 128.0 - l.apply(GREY, q(50.0, y, px))[0];
            // Exactly half covered at the radius.
            assert!((at(5.0) - 64.0).abs() < 1e-9, "px {px}: {}", at(5.0));
            assert_eq!(at(5.0 + px / 2.0 + 1e-9), 0.0, "px {px}: outside the AA band");
            assert_eq!(at(5.0 - px / 2.0 - 1e-9), 128.0, "px {px}: inside the AA band");
        }
    }

    /// Protects: a soft stroke's profile -- solid inside `radius - band`,
    /// half covered half-way into the band, empty at the radius -- with
    /// `band = radius * (1 - hardness)`.
    #[test]
    fn soft_stroke_profile() {
        let mut m = line(&[[0.0, 0.0], [100.0, 0.0]], 10.0, [0, 0, 0], ColourBlend::Replace);
        m.hardness = 0.5;
        let l = layer(vec![m]);
        let cov = |y: f64| (128.0 - l.apply(GREY, q(50.0, y, 0.01))[0]) / 128.0;
        assert_eq!(cov(4.9), 1.0);
        assert!((cov(7.5) - 0.5).abs() < 1e-9, "{}", cov(7.5));
        assert_eq!(cov(10.01), 0.0);
        assert!(cov(6.0) > 0.5 && cov(9.0) < 0.5);
    }

    /// Protects: `ClipColor`'s dark branch -- a saturated colour set to a
    /// low luminance keeps that luminance with no negative channel.
    #[test]
    fn colour_blend_clips_dark_backdrops() {
        let c = ColourBlend::Colour.apply([20.0, 20.0, 20.0], [255.0, 0.0, 0.0], 1.0);
        assert!(c.iter().all(|v| *v >= -1e-9), "{c:?}");
        assert!((lum(c) - 20.0).abs() < 1e-9, "{c:?}");
        assert!(c[0] > c[1] && (c[1] - c[2]).abs() < 1e-9, "{c:?}");
    }

    /// Protects: the index answers what the brute-force fallback answers,
    /// for strokes and areas, over a dense set of points (the index is an
    /// optimisation and must never change a colour).
    #[test]
    fn index_matches_brute_force() {
        let mut marks = vec![];
        for k in 0..40 {
            let y = k as f64 * 2.3;
            marks.push(line(&[[0.0, y], [30.0, y + 7.0], [55.0, y - 3.0]], 0.5 + (k % 5) as f64, [(k * 6) as u8, 100, 200], ColourBlend::ALL[k % 3]));
        }
        let ring: Vec<[f64; 2]> = (0..64).map(|i| {
            let a = i as f64 / 64.0 * std::f64::consts::TAU;
            let r = 20.0 + 6.0 * (3.0 * a).sin();
            [40.0 + r * a.cos(), 50.0 + r * a.sin()]
        }).collect();
        marks.push(ColourMark { shape: MarkShape::Area { ring_km: ring, edge_km: 0.7 }, rgb: [250, 20, 10], opacity: 0.8, hardness: 1.0, blend: ColourBlend::Colour, land_only: true, emissive: None });
        let l = layer(marks);
        // The same document with a zero margin forces every query (px > 0)
        // through the brute-force path.
        let brute = ColourPaint::new(l.doc().clone(), 0.0).unwrap();
        let mut differing = 0;
        let mut painted = 0;
        for yi in 0..300 {
            for xi in 0..300 {
                let p = q(-5.0 + xi as f64 * 0.31, -5.0 + yi as f64 * 0.37, 0.4);
                let a = l.apply(GREY, p);
                let b = brute.apply(GREY, p);
                if a != GREY {
                    painted += 1;
                }
                if a.iter().zip(&b).any(|(x, y)| (x - y).abs() > 1e-9) {
                    differing += 1;
                }
            }
        }
        assert!(painted > 10_000, "premise: the fixture paints much of the window ({painted})");
        assert_eq!(differing, 0);
    }

    fn lava(pts: &[[f64; 2]], r: f64, heat: f64, along: f64, glow: f64) -> ColourMark {
        let mut m = line(pts, r, [34, 33, 38], ColourBlend::Replace);
        m.emissive = Some(EmissiveMaterial { ramp: GlowRamp::Fire, heat, crack_density: 0.5, glow_radius_km: glow, heat_along: along });
        m
    }

    /// Protects: with emissive marks (their halo and ragged edge widening
    /// each mark's reach) the index still answers exactly what the
    /// brute-force path answers, crust and glow both -- a reach that forgot
    /// the halo or the edge would cut them off at bucket borders.
    #[test]
    fn index_matches_brute_force_with_lava() {
        let mut marks = vec![];
        for k in 0..12 {
            let y = k as f64 * 4.1;
            marks.push(lava(&[[0.0, y], [20.0, y + 5.0], [40.0, y - 2.0]], 1.0 + (k % 3) as f64, 0.4 + 0.05 * k as f64, 0.5, 3.0));
            if k % 4 == 0 {
                marks.push(line(&[[10.0, y], [30.0, y]], 0.8, [200, 200, 0], ColourBlend::Colour));
            }
        }
        let l = layer(marks);
        let brute = ColourPaint::new(l.doc().clone(), 0.0).unwrap();
        let (mut glowing, mut differing) = (0, 0);
        for yi in 0..260 {
            for xi in 0..200 {
                let p = q(-6.0 + xi as f64 * 0.27, -6.0 + yi as f64 * 0.23, 0.05);
                let (a, ea) = l.apply_lit(GREY, p);
                let (b, eb) = brute.apply_lit(GREY, p);
                if ea != [0.0; 3] {
                    glowing += 1;
                }
                if a.iter().chain(&ea).zip(b.iter().chain(&eb)).any(|(x, y)| (x - y).abs() > 1e-9) {
                    differing += 1;
                }
            }
        }
        assert!(glowing > 10_000, "premise: the fixture glows ({glowing})");
        assert_eq!(differing, 0);
    }

    /// Protects the glow's order rules: a flat mark painted over lava hides
    /// the glow it covers; overlapping lava marks merge (the hotter one sets
    /// the temperature, so a cool stroke over a hot pool does not cool it);
    /// and the halo reaches past the edge but not past its radius.
    #[test]
    fn glow_order_merge_and_halo() {
        let pool = lava(&[[0.0, 0.0]], 5.0, 0.9, 0.0, 2.0);
        let cool = lava(&[[-10.0, 0.0], [10.0, 0.0]], 1.0, 0.1, 0.0, 0.0);
        let cover = line(&[[0.0, 0.0]], 6.0, [0, 0, 0], ColourBlend::Replace);
        let mean_glow = |l: &ColourPaint| (0..200).map(|k| l.apply_lit(GREY, q(-2.0 + k as f64 * 0.02, 0.0, 0.01)).1[0]).sum::<f64>() / 200.0;
        let hot = mean_glow(&layer(vec![pool.clone()]));
        assert!(hot > 50.0, "premise: the pool glows ({hot})");
        assert_eq!(mean_glow(&layer(vec![pool.clone(), cover])), 0.0, "an opaque flat mark hides the glow");
        let merged = mean_glow(&layer(vec![pool.clone(), cool.clone()]));
        assert!(merged > 0.8 * hot, "a cool stroke over a hot pool merges, not cools it: {merged} vs {hot}");
        let l = layer(vec![pool]);
        // The ragged edge moves the contour by at most `edge_reach`, here
        // 2 * 0.16 * 1.6 * 5 = 2.56 km. Around the dab: some pixels the
        // mark does not cover (the colour is untouched) still glow -- the
        // halo -- and nothing glows past radius + reach + glow radius.
        let (mut halo_only, mut beyond) = (0, 0);
        for k in 0..720 {
            let a = k as f64 / 720.0 * std::f64::consts::TAU;
            for r in [5.5, 6.5, 7.5, 8.5] {
                let (c, e) = l.apply_lit(GREY, q(r * a.cos(), r * a.sin(), 0.01));
                if c == GREY && e[0] > 0.0 {
                    halo_only += 1;
                }
            }
            let (c, e) = l.apply_lit(GREY, q(9.6 * a.cos(), 9.6 * a.sin(), 0.01));
            if c != GREY || e != [0.0; 3] {
                beyond += 1;
            }
        }
        assert!(halo_only > 200, "the halo reaches past the edge ({halo_only})");
        assert_eq!(beyond, 0, "and stops at its radius");
        // A halo lights the ground, not other lava: inside a cold crust
        // beside a hot pool with a wide halo, the glow is the crust's own.
        let mut wide = lava(&[[0.0, 0.0]], 5.0, 0.9, 0.0, 12.0);
        wide.land_only = true;
        let cold = lava(&[[14.0, 0.0]], 4.0, 0.0, 0.0, 0.0);
        let alone = layer(vec![cold.clone()]);
        let both = layer(vec![wide, cold]);
        let at = q(15.0, 0.0, 0.01);
        assert!(both.apply_lit(GREY, at).1[0] <= alone.apply_lit(GREY, at).1[0] + 1e-9, "a halo shows through a covering lava mark");
        assert!(both.apply_lit(GREY, q(9.5, 3.0, 0.01)).1[0] > 0.0 || both.apply_lit(GREY, q(9.0, -3.0, 0.01)).1[0] > 0.0, "premise: the wide halo reaches the cold crust's side");
    }

    /// Protects the ragged edge: a lava dab's half-coverage radius varies
    /// with direction by at least 15 % of its radius (lobes, not a pen's
    /// circle), while a flat dab's is constant -- and the lava's edge stays
    /// within `edge_reach` of the stroke's.
    #[test]
    fn a_lava_edge_is_ragged() {
        let radius_at = |l: &ColourPaint, a: f64| {
            let mut r = 0.0;
            while r < 20.0 && l.apply_lit(GREY, q(r * a.cos(), r * a.sin(), 0.005)).0 != GREY {
                r += 0.01;
            }
            r
        };
        let lv = layer(vec![lava(&[[0.0, 0.0]], 5.0, 0.6, 0.0, 0.0)]);
        let fl = layer(vec![line(&[[0.0, 0.0]], 5.0, [0, 0, 0], ColourBlend::Replace)]);
        let rs: Vec<f64> = (0..90).map(|k| radius_at(&lv, k as f64 * 0.0698)).collect();
        let (lo, hi) = (rs.iter().cloned().fold(f64::MAX, f64::min), rs.iter().cloned().fold(0.0, f64::max));
        assert!(hi - lo >= 0.75, "lava edge spread {lo:.2}..{hi:.2} km");
        assert!(lo >= 5.0 - 2.57 && hi <= 5.0 + 2.57, "{lo:.2}..{hi:.2}");
        let fr: Vec<f64> = (0..90).map(|k| radius_at(&fl, k as f64 * 0.0698)).collect();
        assert!(fr.iter().all(|r| (r - fr[0]).abs() < 0.02), "a flat dab stays round");
    }

    /// Protects: an area paints its interior (far from edges, via the
    /// bucket-centre state) and not its outside; a hole-free ring.
    #[test]
    fn area_fills_its_interior_only() {
        let sq = vec![[10.0, 10.0], [30.0, 10.0], [30.0, 30.0], [10.0, 30.0]];
        let l = layer(vec![ColourMark { shape: MarkShape::Area { ring_km: sq, edge_km: 0.0 }, rgb: [255, 0, 0], opacity: 1.0, hardness: 1.0, blend: ColourBlend::Replace, land_only: true, emissive: None }]);
        assert_eq!(l.apply(GREY, q(20.0, 20.0, 0.1)), [255.0, 0.0, 0.0]);
        assert_eq!(l.apply(GREY, q(10.5, 29.5, 0.1)), [255.0, 0.0, 0.0]);
        assert_eq!(l.apply(GREY, q(5.0, 20.0, 0.1)), GREY);
        assert_eq!(l.apply(GREY, q(35.0, 35.0, 0.1)), GREY);
    }

    /// Protects: `land_only` per mark -- a water pixel is painted only by a
    /// mark with `land_only == false`.
    #[test]
    fn land_only_is_per_mark() {
        let mut wet = line(&[[0.0, 0.0]], 3.0, [255, 0, 0], ColourBlend::Replace);
        wet.land_only = false;
        let dry = line(&[[10.0, 0.0]], 3.0, [0, 0, 255], ColourBlend::Replace);
        let l = layer(vec![wet, dry]);
        let w = |x: f64| l.apply(GREY, PaintQuery { x_km: x, y_km: 0.0, px_km: 0.1, water: true });
        assert_eq!(w(0.0), [255.0, 0.0, 0.0]);
        assert_eq!(w(10.0), GREY);
        assert_eq!(l.apply(GREY, q(10.0, 0.0, 0.1)), [0.0, 0.0, 255.0]);
        let only_dry = layer(vec![line(&[[10.0, 0.0]], 3.0, [0, 0, 255], ColourBlend::Replace)]);
        assert!(!only_dry.paints_water());
    }

    /// Protects: later marks composite over earlier ones.
    #[test]
    fn marks_composite_in_order() {
        let l = layer(vec![line(&[[0.0, 0.0]], 3.0, [255, 0, 0], ColourBlend::Replace), line(&[[0.0, 0.0]], 3.0, [0, 0, 255], ColourBlend::Replace)]);
        assert_eq!(l.apply(GREY, q(0.0, 0.0, 0.1)), [0.0, 0.0, 255.0]);
    }

    /// Protects: every cap and every non-finite input is refused, naming the
    /// mark; a refused document builds nothing.
    #[test]
    fn caps_and_bad_values_are_refused() {
        let ok = line(&[[0.0, 0.0]], 1.0, [0, 0, 0], ColourBlend::Colour);
        let doc = |marks: Vec<ColourMark>| ColourPaintDoc { version: DOC_VERSION, visible: true, marks };
        assert_eq!(ColourPaint::new(doc(vec![ok.clone(); MAX_MARKS + 1]), 1.0).unwrap_err(), ColourPaintError::TooManyMarks(MAX_MARKS + 1));
        assert!(ColourPaint::new(doc(vec![ok.clone(); MAX_MARKS]), 1.0).is_ok());
        let long = line(&vec![[0.0, 0.0]; MAX_POINTS_PER_MARK + 1], 1.0, [0, 0, 0], ColourBlend::Colour);
        assert_eq!(ColourPaint::new(doc(vec![long]), 1.0).unwrap_err(), ColourPaintError::TooManyPoints { mark: 0, points: MAX_POINTS_PER_MARK + 1 });
        let many = line(&vec![[0.0, 0.0]; MAX_POINTS_PER_MARK], 1.0, [0, 0, 0], ColourBlend::Colour);
        let n = MAX_TOTAL_POINTS / MAX_POINTS_PER_MARK + 1;
        assert_eq!(ColourPaint::new(doc(vec![many; n]), 1.0).unwrap_err(), ColourPaintError::TooManyTotalPoints(n * MAX_POINTS_PER_MARK));
        let nan = line(&[[f64::NAN, 0.0]], 1.0, [0, 0, 0], ColourBlend::Colour);
        assert_eq!(ColourPaint::new(doc(vec![ok.clone(), nan]), 1.0).unwrap_err(), ColourPaintError::NotFinite(1));
        let thin = line(&[[0.0, 0.0]], 0.0005, [0, 0, 0], ColourBlend::Colour);
        assert_eq!(ColourPaint::new(doc(vec![thin]), 1.0).unwrap_err(), ColourPaintError::BadRadius(0));
        let empty = ColourMark { shape: MarkShape::Area { ring_km: vec![[0.0, 0.0], [1.0, 0.0]], edge_km: 0.1 }, ..ok.clone() };
        assert_eq!(ColourPaint::new(doc(vec![empty]), 1.0).unwrap_err(), ColourPaintError::TooFewPoints(0));
        let v2 = ColourPaintDoc { version: 2, visible: true, marks: vec![] };
        assert_eq!(ColourPaint::new(v2, 1.0).unwrap_err(), ColourPaintError::UnknownVersion(2));
    }

    /// Protects: the document round-trips through JSON unchanged, a missing
    /// `land_only` reads as `true`, and an unknown blend key is refused.
    #[test]
    fn document_round_trips_and_defaults() {
        let l = layer(vec![line(&[[1.5, 2.25]], 0.75, [10, 20, 30], ColourBlend::Multiply)]);
        let s = serde_json::to_string(l.doc()).unwrap();
        let back: ColourPaintDoc = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, l.doc());
        let old = r#"{"version":1,"visible":true,"marks":[{"shape":{"kind":"line","points_km":[[0,0]],"radius_km":1},"rgb":[1,2,3],"opacity":1,"hardness":1,"blend":"colour"}]}"#;
        let d: ColourPaintDoc = serde_json::from_str(old).unwrap();
        assert!(d.marks[0].land_only);
        assert!(serde_json::from_str::<ColourPaintDoc>(&old.replace("colour", "glow")).is_err());
        assert_eq!(ColourBlend::from_key("replace"), Some(ColourBlend::Replace));
        assert_eq!(ColourBlend::from_key("Replace"), None);
    }

    fn layers_of(ls: Vec<(&str, bool, bool, Vec<ColourMark>)>) -> PaintLayers {
        PaintLayers { visible: true, layers: ls.into_iter().map(|(n, v, k, m)| PaintLayer { name: n.to_string(), visible: v, locked: k, marks: m }).collect() }
    }

    /// Protects Ruling CS's compositing order: layers composite bottom
    /// first, so the top layer's Replace wins where both reach, and moving it
    /// to the bottom flips the winner; a hidden layer is left out of the
    /// render list entirely (its marks reach no pixel and `drawn_len` drops),
    /// yet it is still counted and saved.
    #[test]
    fn layers_composite_bottom_first_and_hidden_layers_are_not_drawn() {
        let red = line(&[[0.0, 0.0], [10.0, 0.0]], 2.0, [255, 0, 0], ColourBlend::Replace);
        let blue = line(&[[0.0, 0.0], [10.0, 0.0]], 2.0, [0, 0, 255], ColourBlend::Replace);
        let mut ls = layers_of(vec![("Under", true, false, vec![red]), ("Over", true, false, vec![blue])]);
        let at = |l: &PaintLayers| ColourPaint::from_layers(l.clone(), 1.0).unwrap().apply(GREY, q(5.0, 0.0, 0.01));
        assert_eq!(at(&ls), [0.0, 0.0, 255.0], "the top layer is drawn last");
        ls.move_layer(1, 0).unwrap();
        assert_eq!(ls.layers[0].name, "Over");
        assert_eq!(at(&ls), [255.0, 0.0, 0.0], "reordered: red is now on top");
        ls.set_layer_visible(1, false).unwrap();
        let cp = ColourPaint::from_layers(ls.clone(), 1.0).unwrap();
        assert_eq!(cp.apply(GREY, q(5.0, 0.0, 0.01)), [0.0, 0.0, 255.0], "the hidden top layer draws nothing");
        assert_eq!((cp.len(), cp.drawn_len()), (2, 1));
        assert_eq!(cp.doc().marks.len(), 1, "the flattened document (what the tile key hashes) leaves the hidden layer out");
        ls.set_layer_visible(0, false).unwrap();
        let cp = ColourPaint::from_layers(ls, 1.0).unwrap();
        assert!(cp.is_inert() && !cp.is_empty(), "every layer hidden: inert, but still holding marks to save");
        assert_eq!(cp.apply(GREY, q(5.0, 0.0, 0.01)), GREY);
    }

    /// Protects backward compatibility: a version-1 document reads as one
    /// visible, unlocked "Paint 1" layer and writes back the **same bytes**
    /// (compact and pretty); a renamed, locked, hidden or second layer writes
    /// version 2, which reads back equal; a one-layer layered document built
    /// from version 1 renders exactly what [`ColourPaint::new`] rendered; an
    /// unknown version, or a version without its own member, is refused.
    #[test]
    fn version_one_reads_as_one_layer_and_writes_back_byte_identically() {
        let v1 = r#"{"version":1,"visible":false,"marks":[{"shape":{"kind":"line","points_km":[[1.5,2.25]],"radius_km":0.75},"rgb":[10,20,30],"opacity":1.0,"hardness":1.0,"blend":"multiply","land_only":true}]}"#;
        let ls: PaintLayers = serde_json::from_str(v1).unwrap();
        assert_eq!(ls.layers.len(), 1);
        assert_eq!(ls.layers[0].name, "Paint 1");
        assert!(!ls.visible && ls.layers[0].visible && !ls.layers[0].locked, "v1's flag is the master switch");
        assert_eq!(serde_json::to_string(&ls).unwrap(), v1, "unchanged data re-saves byte-identically");
        let pretty = serde_json::to_string_pretty(&serde_json::from_str::<ColourPaintDoc>(v1).unwrap()).unwrap();
        assert_eq!(serde_json::to_string_pretty(&ls).unwrap(), pretty, "and in the pretty form the archive writes");
        let edits: [fn(&mut PaintLayers); 4] = [
            |l| l.rename(0, "Lava").unwrap(),
            |l| l.set_layer_locked(0, true).unwrap(),
            |l| l.set_layer_visible(0, false).unwrap(),
            |l| {
                l.add_layer(None).unwrap();
            },
        ];
        for edit in edits {
            let mut e = ls.clone();
            edit(&mut e);
            let s = serde_json::to_string(&e).unwrap();
            assert!(s.starts_with(r#"{"version":2,"visible":false,"layers":["#), "{s}");
            assert_eq!(serde_json::from_str::<PaintLayers>(&s).unwrap(), e);
        }
        let mut shown: ColourPaintDoc = serde_json::from_str(v1).unwrap();
        shown.visible = true;
        let a = ColourPaint::new(shown.clone(), 1.0).unwrap();
        let b = ColourPaint::from_layers(PaintLayers::from_v1(shown), 1.0).unwrap();
        assert_eq!(a.doc(), b.doc());
        let c = [12.5, 99.0, 201.0];
        assert_eq!(a.apply(c, q(1.6, 2.3, 0.01)), b.apply(c, q(1.6, 2.3, 0.01)));
        assert_ne!(a.apply(c, q(1.6, 2.3, 0.01)), c, "premise: the mark reaches");
        for bad in [r#"{"version":3,"visible":true,"layers":[]}"#, r#"{"version":2,"visible":true,"marks":[]}"#, r#"{"version":1,"visible":true,"layers":[]}"#] {
            assert!(serde_json::from_str::<PaintLayers>(bad).is_err(), "{bad}");
        }
    }

    /// Protects the layer caps: the mark cap is a total over every layer
    /// (hidden included; exactly 5 000 accepted, 5 002 refused), the layer
    /// count and names are capped, and a bad mark in a hidden layer is still
    /// refused (one rule set).
    #[test]
    fn caps_count_every_layer() {
        let ok = line(&[[0.0, 0.0]], 1.0, [0, 0, 0], ColourBlend::Colour);
        let ls = layers_of(vec![("A", true, false, vec![ok.clone(); 2_501]), ("B", false, false, vec![ok.clone(); 2_501])]);
        assert_eq!(ColourPaint::from_layers(ls, 1.0).unwrap_err(), ColourPaintError::TooManyMarks(5_002));
        let ls = layers_of(vec![("A", true, false, vec![ok.clone(); 2_500]), ("B", false, false, vec![ok.clone(); 2_500])]);
        assert_eq!(ColourPaint::from_layers(ls, 1.0).unwrap().len(), 5_000, "exactly the cap is accepted");
        let nan = line(&[[f64::NAN, 0.0]], 1.0, [0, 0, 0], ColourBlend::Colour);
        let ls = layers_of(vec![("A", true, false, vec![ok.clone()]), ("B", false, false, vec![nan])]);
        assert_eq!(ColourPaint::from_layers(ls, 1.0).unwrap_err(), ColourPaintError::NotFinite(1));
        let many: Vec<_> = (0..33).map(|i| PaintLayer { name: format!("L{i}"), visible: true, locked: false, marks: vec![] }).collect();
        assert_eq!(ColourPaint::from_layers(PaintLayers { visible: true, layers: many }, 1.0).unwrap_err(), ColourPaintError::TooManyLayers(33));
        let ls = layers_of(vec![("  ", true, false, vec![])]);
        assert_eq!(ColourPaint::from_layers(ls, 1.0).unwrap_err(), ColourPaintError::BadLayerName(0));
        let long = "x".repeat(65);
        let ls = layers_of(vec![(long.as_str(), true, false, vec![])]);
        assert_eq!(ColourPaint::from_layers(ls, 1.0).unwrap_err(), ColourPaintError::BadLayerName(0));
        let mut e = PaintLayers::empty();
        for _ in 0..32 {
            e.add_layer(None).unwrap();
        }
        assert!(e.add_layer(None).is_err(), "add_layer refuses the 33rd");
        assert!(e.rename(0, &"y".repeat(65)).is_err());
        assert!(e.rename(0, &"y".repeat(64)).is_ok());
    }

    /// Protects the edit verbs: the first stroke creates "Paint 1"; new
    /// layers take the next free "Paint N" on top; a lock refuses add, clear
    /// and delete but not rename, hide or move; out-of-range indices are
    /// refused, never clamped onto another layer.
    #[test]
    fn layer_edits_respect_locks_and_names() {
        let m = line(&[[0.0, 0.0]], 1.0, [0, 0, 0], ColourBlend::Colour);
        let mut l = PaintLayers::empty();
        l.add_mark(0, m.clone()).unwrap();
        assert_eq!(l.layers.len(), 1);
        assert_eq!(l.layers[0].name, "Paint 1");
        assert!(l.is_v1_shaped());
        assert_eq!(l.add_layer(Some("  ")).unwrap(), 1);
        assert_eq!(l.layers[1].name, "Paint 2");
        l.rename(0, "Base").unwrap();
        assert_eq!(l.add_layer(None).unwrap(), 2);
        assert_eq!(l.layers[2].name, "Paint 1", "the first free number");
        assert!(l.add_mark(5, m.clone()).is_err());
        assert!(l.set_layer_visible(3, false).is_err());
        assert!(l.move_layer(0, 3).is_err());
        l.set_layer_locked(0, true).unwrap();
        assert!(l.add_mark(0, m.clone()).is_err());
        assert!(l.clear_layer(0).is_err());
        assert!(l.delete_layer(0).is_err());
        l.rename(0, "Base, kept").unwrap();
        l.set_layer_visible(0, false).unwrap();
        l.move_layer(0, 2).unwrap();
        assert_eq!(l.layers[2].name, "Base, kept");
        assert_eq!(l.layers[2].marks.len(), 1, "the locked layer kept its mark");
        l.add_mark(1, m.clone()).unwrap();
        l.clear_layer(1).unwrap();
        assert!(l.layers[1].marks.is_empty());
        l.delete_layer(0).unwrap();
        assert_eq!(l.layers.len(), 2);
        assert!(serde_json::to_string(&l).unwrap().contains(r#""locked":true"#));
    }
}
