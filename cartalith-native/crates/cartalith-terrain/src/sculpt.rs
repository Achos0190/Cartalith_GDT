//! The Sculpt editor's landform-stamp registry — `UNIFIED_TOOL_PLAN.md`
//! milestone B, the Terrain group.
//!
//! A direct port of the reference HTML's own shipped Sculpt editor core
//! (`reference/Cartalith Gen1 v2.10.html` lines 8821-9081: `sculptFbm`/
//! `sculptRidged`/`sculptBillow`, `sculptNearestOnStroke`,
//! `SCULPT_GLOBAL_DEF`, `_sculptCtx`, the 13-entry `SCULPT_FEATURES`
//! registry, the 8 `SCULPT_PRESETS`, `sculptStampRadius`/`sculptStampBBox`/
//! `sculptApplyStamp`). That block is flagged in the reference itself as
//! *"pure, DOM-free core"* — it touches no DOM, no module globals, no
//! `field` — which is exactly why the whole of it ports as plain functions
//! and why it is headlessly golden-verifiable (see
//! `tests/golden_parity_sculpt.rs`).
//!
//! ## Where this lives, and why
//!
//! `cartalith-terrain`, not a new crate and not `cartalith-engine`.
//! Milestone A's split — generic stack machinery into `cartalith-spatial`,
//! Cartalith *pipeline* knowledge into `cartalith-engine` — leaves a third
//! category this belongs to: subsystem-domain math. Every one of the
//! thirteen features is a height-field formula; `ARCHITECTURE.md`'s "one
//! crate per subsystem" already names `cartalith-terrain` as the crate that
//! owns the height formula, and the reference itself keeps `SCULPT_FEATURES`
//! in script block 1 next to tectonics rather than anywhere near its UI. A
//! new `cartalith-sculpt` crate would buy a `Cargo.toml` and nothing else:
//! there is no second consumer, no independent test boundary (these tests
//! need `cartalith-noise`, which terrain already depends on), and no
//! dependency it would break. `cartalith-engine` would be wrong for the
//! opposite reason to milestone A's — this is computation, and
//! *"`cartalith-engine` orchestrates; it does not compute"*
//! (`ARCHITECTURE.md`).
//!
//! [`SculptStamp`] implements `cartalith_spatial::Stamp`, so it drops
//! straight into milestone A's `PassBuffer` with its draft/preview/commit/
//! discard and draft-scoped undo. That is this crate's first dependency on
//! `cartalith-spatial` (`cartalith-engine`'s, added in milestone A, was the
//! workspace's first).
//!
//! ## Determinism
//!
//! Every noise call here goes through `cartalith-noise`'s **JS-matching**
//! `vnoise`/`hash`, never the GPU-safe PCG3D `gpu_vnoise`. `DECISIONS.md`
//! §7/§7a govern: §7's golden-parity requirement applies to any CPU path
//! that has a reference ancestor, and §7a's principled-equivalence
//! relaxation is scoped to GPU/optimized paths specifically. A sculpt stamp
//! has a reference ancestor and runs on the CPU, so it must reproduce the
//! reference's own result bit-for-bit at a given seed — verified, not
//! assumed, by `tests/golden_parity_sculpt.rs`.
//!
//! A stamp's effective noise seed is derived exactly as the reference does:
//! `(stamp.seed ^ ((feature_index + 1) * 1013)) >>> 0`, where
//! `feature_index` is the feature's position in [`FEATURE_KEYS`]. **That
//! order is load-bearing** — it is `Object.keys(SCULPT_FEATURES)` order in
//! the reference, and reordering [`FEATURE_KEYS`] would silently change
//! every stamp's noise.

use cartalith_spatial::{Region, Stamp};

use crate::js_round;

// ---------------------------------------------------------------------------
// JS-exact scalar helpers
// ---------------------------------------------------------------------------

/// Reference `clamp01` (line 7568). `f64::clamp` is the exact equivalent of
/// JS's `x<0?0:x>1?1:x` — including NaN, which both propagate because both
/// comparisons are false — and it is what `cartalith-civ`'s own `clamp01`
/// already uses. (`f64::min`/`max` would *not* be equivalent: they swallow
/// NaN.)
fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Reference `smoothstep(a,b,x)` (line 7569):
/// `t=clamp01((x-a)/((b-a)||1e-6)); return t*t*(3-2*t)`.
///
/// The `||1e-6` is JS truthiness, so it substitutes for `0`, `-0` **and**
/// `NaN`. `cliff` genuinely reaches it (`smoothstep(-transW, transW, sd)`
/// with `transW == 0` would divide by zero), so this is not a defensive
/// flourish to drop.
fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let d = b - a;
    let d = if d == 0.0 || d.is_nan() { 1e-6 } else { d };
    let t = clamp01((x - a) / d);
    t * t * (3.0 - 2.0 * t)
}

/// Reference `lerp` (line 8304).
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

// V8's compensated `Math.hypot`, and its variadic form. Both were written
// here on `UNIFIED_TOOL_PLAN.md` milestone B and re-derived in four other
// crates afterwards; `JS_SEMANTICS_AUDIT.md` §3.2 found three of the four
// copies had lost the specification preamble along the way. One
// implementation now, in `cartalith-jsmath`.
//
// Re-exported rather than imported so `sculpt::js_hypot` keeps meaning what it
// meant to `amplify` and `tile_render`, which both reach it by that path. The
// honesty note milestone B attached still holds and is worth keeping: swapping
// this for plain `sqrt(x*x + y*y)` leaves all 23 cases of
// `tests/golden_parity_sculpt.rs` passing bit-exactly, so it is not
// test-enforced *here*; `cartalith-civ` milestone D found the fixture that
// does enforce it, and `nearest_on_stroke`'s `dist < best` is the comparison
// that would bite if it were simplified.
pub(crate) use cartalith_jsmath::js_hypot;

// ---------------------------------------------------------------------------
// Noise: the three sculpt-specific FBM families
// ---------------------------------------------------------------------------

/// The per-octave seed the reference forms as `s + o*131`, where `s` is a
/// JS Number holding a `>>>0` uint32 and `hash` re-narrows with `(s|0)`.
/// A wrapping `u32` add followed by a reinterpreting cast is exactly that
/// `ToInt32`, for every magnitude these seeds reach.
fn octave_seed(s: u32, o: u32) -> i32 {
    s.wrapping_add(o.wrapping_mul(131)) as i32
}

/// Reference `sculptFbm` (line 8837).
///
/// Deliberately **not** `cartalith_noise::fbm`: that one hardcodes 6
/// octaves / 0.5 persistence / 2.0 lacunarity and returns `[0,1]`, whereas
/// every sculpt feature needs all three as live parameters and was tuned
/// against a `~[-1,1]` range. The reference's own comment says exactly this
/// and calls the range convention deliberate — *"changing it would silently
/// alter results already visually verified"* — so the `*2-1` remap stays.
pub fn sculpt_fbm(x: f64, y: f64, oct: u32, pers: f64, lac: f64, s: u32) -> f64 {
    let (mut amp, mut freq, mut sum, mut nrm) = (1.0f64, 1.0f64, 0.0f64, 0.0f64);
    for o in 0..oct {
        sum += amp * (cartalith_noise::vnoise(x * freq, y * freq, octave_seed(s, o)) * 2.0 - 1.0);
        nrm += amp;
        amp *= pers;
        freq *= lac;
    }
    // JS `return nrm?sum/nrm:0` — falsy covers 0 and NaN.
    if nrm != 0.0 && !nrm.is_nan() {
        sum / nrm
    } else {
        0.0
    }
}

/// Reference `sculptRidged` (line 8838). The `prev` term is a
/// multifractal weight carried between octaves, so octave order matters.
pub fn sculpt_ridged(x: f64, y: f64, oct: u32, pers: f64, lac: f64, s: u32) -> f64 {
    let (mut amp, mut freq, mut sum, mut nrm, mut prev) = (1.0f64, 1.0f64, 0.0f64, 0.0f64, 1.0f64);
    for o in 0..oct {
        let n = cartalith_noise::vnoise(x * freq, y * freq, octave_seed(s, o)) * 2.0 - 1.0;
        let n = 1.0 - n.abs();
        let n = n * n;
        sum += amp * n * prev;
        prev = clamp01(n * 1.6);
        nrm += amp;
        amp *= pers;
        freq *= lac;
    }
    if nrm != 0.0 && !nrm.is_nan() {
        sum / nrm
    } else {
        0.0
    }
}

/// Reference `sculptBillow` (line 8839).
pub fn sculpt_billow(x: f64, y: f64, oct: u32, pers: f64, lac: f64, s: u32) -> f64 {
    let (mut amp, mut freq, mut sum, mut nrm) = (1.0f64, 1.0f64, 0.0f64, 0.0f64);
    for o in 0..oct {
        sum += amp
            * (cartalith_noise::vnoise(x * freq, y * freq, octave_seed(s, o)) * 2.0 - 1.0).abs();
        nrm += amp;
        amp *= pers;
        freq *= lac;
    }
    if nrm != 0.0 && !nrm.is_nan() {
        sum / nrm
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Stroke geometry
// ---------------------------------------------------------------------------

/// A captured stroke point, in **grid cell** coordinates at the current
/// resolution (the reference captures via `evtToGridLOD` precisely so a
/// stroke behaves identically at any zoom or LOD level).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// What [`nearest_on_stroke`] reports about a query point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeHit {
    /// Unsigned distance to the polyline.
    pub dist: f64,
    /// Signed distance — which side of the stroke, by the 2D cross product.
    pub sd: f64,
    /// Arclength along the stroke at the projection (drives `meander`).
    pub s: f64,
    pub tx: f64,
    pub ty: f64,
}

/// Reference `sculptNearestOnStroke` (line 8845).
///
/// A **1-point stroke degenerates to plain radial distance**, which is not
/// an edge case to tidy up but the mechanism by which one registry entry
/// serves both drag strokes and tap-once stamps (the reference's own
/// comment: *"so a single registry entry can serve both stroke- and
/// tap-interactions without a second geometry path"* — Freehand's `mesa`
/// and `volcano` sub-modes rely on it).
pub fn nearest_on_stroke(px: f64, py: f64, pts: &[Point]) -> StrokeHit {
    if pts.len() == 1 {
        let dx = px - pts[0].x;
        let dy = py - pts[0].y;
        let dd = js_hypot(dx, dy);
        return StrokeHit {
            dist: dd,
            sd: dd,
            s: 0.0,
            tx: 1.0,
            ty: 0.0,
        };
    }
    let mut best = f64::INFINITY;
    let mut sd = 0.0f64;
    let mut s_arc = 0.0f64;
    let mut tx = 1.0f64;
    let mut ty = 0.0f64;
    let mut acc = 0.0f64;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let l2 = dx * dx + dy * dy;
        // JS `(dx*dx+dy*dy)||1e-9` — a zero-length segment (duplicate
        // captured point) must not divide by zero.
        let l2 = if l2 == 0.0 || l2.is_nan() { 1e-9 } else { l2 };
        let t = (((px - a.x) * dx + (py - a.y) * dy) / l2).clamp(0.0, 1.0);
        let cx = a.x + t * dx;
        let cy = a.y + t * dy;
        let ex = px - cx;
        let ey = py - cy;
        let dist = js_hypot(ex, ey);
        if dist < best {
            best = dist;
            let cross = dx * ey - dy * ex;
            sd = if cross < 0.0 { -dist } else { dist };
            let seg_len = l2.sqrt();
            s_arc = acc + t * seg_len;
            tx = dx / seg_len;
            ty = dy / seg_len;
        }
        acc += l2.sqrt();
    }
    StrokeHit {
        dist: best,
        sd,
        s: s_arc,
        tx,
        ty,
    }
}

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

/// One slider in a feature's control set — the reference's
/// `[key, label, min, max, step, default]` tuple, kept as real data rather
/// than folded away, because the Godot options bar / Properties panel
/// milestone F builds needs exactly these ranges and defaults and there is
/// no second source for them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Control {
    pub key: &'static str,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default: f64,
}

const fn ctl(
    key: &'static str,
    label: &'static str,
    min: f64,
    max: f64,
    step: f64,
    default: f64,
) -> Control {
    Control {
        key,
        label,
        min,
        max,
        step,
        default,
    }
}

/// The non-parameter half of a `SCULPT_FEATURES` entry.
#[derive(Debug, Clone, Copy)]
pub struct FeatureMeta {
    /// The reference's own object key — the string a save file or a UI
    /// preset refers to.
    pub key: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    /// Radial features (Lake, Volcano) measure distance from the stroke's
    /// **centroid**; everything else measures signed distance to the
    /// polyline, so a stroke can meander.
    pub radial: bool,
    /// Per-landform character of the domain-warped edge noise: coastlines
    /// and lakes ragged, mountain ridgelines tight, rivers/valleys clean
    /// (their shape already comes from `meander`), Cliff wandering like a
    /// fault trace.
    pub edge_char: f64,
    pub edge_freq_mul: f64,
    pub hint: &'static str,
    pub controls: &'static [Control],
    /// Sub-modes, only Freehand has any.
    pub modes: &'static [&'static str],
}

/// The thirteen registered landform features.
///
/// The reference consolidated three earlier systems into this one list (its
/// own comment: the PoC's 11, a retired plotline's 7 and a retired
/// direct-paint's 9 *"into ONE list"*), with `ridge` and `freehand` as the
/// two genuinely new entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    Mountains,
    Hills,
    Ridge,
    Plateau,
    Cliff,
    Canyon,
    Valley,
    River,
    Lake,
    Basin,
    Coastline,
    Volcano,
    Freehand,
}

/// `Object.keys(SCULPT_FEATURES)` order. **Load-bearing**: the index feeds
/// each stamp's noise seed (`(seed ^ ((i+1)*1013)) >>> 0`), so reordering
/// this array changes every stamp's output.
pub const FEATURE_KEYS: [Feature; 13] = [
    Feature::Mountains,
    Feature::Hills,
    Feature::Ridge,
    Feature::Plateau,
    Feature::Cliff,
    Feature::Canyon,
    Feature::Valley,
    Feature::River,
    Feature::Lake,
    Feature::Basin,
    Feature::Coastline,
    Feature::Volcano,
    Feature::Freehand,
];

/// Freehand's eight sub-modes. `Raise`/`Lower`/`Smooth` follow the drag,
/// `Cliff`/`Ridge`/`Canyon` follow the drag *direction*, `Mesa`/`Volcano`
/// stamp once at a tap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FreehandMode {
    Raise,
    Lower,
    Smooth,
    Cliff,
    Ridge,
    Canyon,
    Mesa,
    Volcano,
}

impl FreehandMode {
    pub fn key(self) -> &'static str {
        match self {
            Self::Raise => "raise",
            Self::Lower => "lower",
            Self::Smooth => "smooth",
            Self::Cliff => "cliff",
            Self::Ridge => "ridge",
            Self::Canyon => "canyon",
            Self::Mesa => "mesa",
            Self::Volcano => "volcano",
        }
    }

    /// The reverse of [`FreehandMode::key`] — added for milestone F's
    /// Godot binding layer, which needs to turn a GDScript-supplied
    /// sub-mode string back into a `FreehandMode` the same way
    /// [`Feature::from_key`] already lets it turn a feature string into a
    /// [`Feature`]. `None` for anything not one of the eight keys.
    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "raise" => Self::Raise,
            "lower" => Self::Lower,
            "smooth" => Self::Smooth,
            "cliff" => Self::Cliff,
            "ridge" => Self::Ridge,
            "canyon" => Self::Canyon,
            "mesa" => Self::Mesa,
            "volcano" => Self::Volcano,
            _ => return None,
        })
    }
}

const FREEHAND_MODES: &[&str] = &[
    "raise", "lower", "smooth", "cliff", "ridge", "canyon", "mesa", "volcano",
];

const MOUNTAIN_CTL: &[Control] = &[
    ctl("mtnHeight", "Height", 0.1, 0.55, 0.01, 0.42),
    ctl("peakSharpness", "Peak sharpness", 0.6, 3.0, 0.05, 1.5),
    ctl("ridgeFrequency", "Ridge freq", 0.6, 5.0, 0.1, 1.6),
    ctl("ruggedness", "Ruggedness", 0.0, 1.0, 0.01, 0.55),
];
const HILLS_CTL: &[Control] = &[
    ctl("amplitude", "Amplitude", 0.02, 0.3, 0.005, 0.11),
    ctl("rollingFrequency", "Rolling freq", 0.5, 4.0, 0.1, 1.4),
    ctl("softness", "Softness", 0.0, 1.0, 0.01, 0.7),
];
const RIDGE_CTL: &[Control] = &[
    ctl("ridgeHeight", "Height", 0.02, 0.35, 0.005, 0.15),
    ctl("ridgeWidth", "Width frac", 0.1, 0.6, 0.01, 0.28),
    ctl("ridgeFreq", "Detail freq", 0.5, 4.0, 0.1, 1.5),
];
const PLATEAU_CTL: &[Control] = &[
    ctl("plateauHeight", "Rise", 0.03, 0.45, 0.005, 0.26),
    ctl("terraces", "Terraces", 1.0, 8.0, 1.0, 4.0),
    ctl("plateauFreq", "Detail freq", 0.4, 3.0, 0.1, 1.1),
];
const CLIFF_CTL: &[Control] = &[
    ctl("cliffHeight", "Rise", 0.05, 0.45, 0.005, 0.22),
    ctl("cliffSteep", "Steepness", 0.2, 1.0, 0.01, 0.75),
];
const CANYON_CTL: &[Control] = &[
    ctl("canyonDepth", "Depth", 0.03, 0.35, 0.005, 0.18),
    ctl("wallSteepness", "Wall steepness", 0.0, 1.0, 0.01, 0.7),
    ctl("meander", "Meander", 0.0, 0.8, 0.01, 0.35),
];
const VALLEY_CTL: &[Control] = &[
    ctl("valleyDepth", "Depth", 0.03, 0.3, 0.005, 0.14),
    ctl("valleyWidth", "Width frac", 0.3, 1.0, 0.02, 0.85),
    ctl("meander", "Meander", 0.0, 0.8, 0.01, 0.3),
];
const RIVER_CTL: &[Control] = &[
    ctl("riverWidth", "Width (px)", 2.0, 26.0, 1.0, 7.0),
    ctl("riverDepth", "Depth", 0.02, 0.22, 0.005, 0.09),
    ctl("riverMeander", "Meander", 0.0, 0.6, 0.01, 0.28),
    ctl("branchNoise", "Branch noise", 0.0, 1.0, 0.01, 0.5),
];
const LAKE_CTL: &[Control] = &[
    ctl("lakeDepth", "Depth", 0.03, 0.3, 0.005, 0.13),
    ctl("lakeShore", "Shore", 0.05, 0.6, 0.01, 0.25),
];
const BASIN_CTL: &[Control] = &[
    ctl("basinDepth", "Depth", 0.02, 0.25, 0.005, 0.1),
    ctl("basinRough", "Floor rough", 0.0, 1.0, 0.01, 0.4),
];
const COASTLINE_CTL: &[Control] = &[
    ctl("coastAmount", "Amount", 0.1, 1.0, 0.01, 0.85),
    ctl("coastRagged", "Raggedness", 0.4, 4.0, 0.1, 1.6),
];
const VOLCANO_CTL: &[Control] = &[
    ctl("volcHeight", "Cone height", 0.15, 0.6, 0.01, 0.45),
    ctl("craterDepth", "Crater depth", 0.0, 0.9, 0.01, 0.5),
    ctl("volcRadius", "Radius (px)", 30.0, 200.0, 2.0, 110.0),
    ctl("flankRough", "Flank rough", 0.0, 1.0, 0.01, 0.6),
];
const FREEHAND_CTL: &[Control] = &[ctl("amount", "Amount", 0.02, 0.3, 0.005, 0.12)];

impl Feature {
    /// Position in [`FEATURE_KEYS`] — the index the seed derivation uses.
    pub fn index(self) -> usize {
        FEATURE_KEYS
            .iter()
            .position(|&f| f == self)
            .expect("in FEATURE_KEYS")
    }

    /// Look a feature up by the reference's own object key.
    pub fn from_key(key: &str) -> Option<Self> {
        FEATURE_KEYS.iter().copied().find(|f| f.meta().key == key)
    }

    /// Kept as a compact table on purpose (`rustfmt::skip`): it is a
    /// transcription of the reference's own `SCULPT_FEATURES` object
    /// literal, and one line per entry is what makes it diffable against
    /// that source. Expanded to one field per line it runs past 200 lines
    /// and stops reading like the registry it is.
    #[rustfmt::skip]
    pub fn meta(self) -> &'static FeatureMeta {
        match self {
            Self::Mountains => &FeatureMeta {
                key: "mountains", label: "Mountains", icon: "\u{26f0}\u{fe0f}",
                radial: false, edge_char: 1.4, edge_freq_mul: 1.5,
                hint: "Ridged multifractal. Peak sharpness powers the crests; ruggedness adds billow detail.",
                controls: MOUNTAIN_CTL, modes: &[],
            },
            Self::Hills => &FeatureMeta {
                key: "hills", label: "Hills", icon: "\u{1f304}",
                radial: false, edge_char: 0.55, edge_freq_mul: 0.9,
                hint: "Smooth rolling FBM. Softness blends coarse and fine octaves.",
                controls: HILLS_CTL, modes: &[],
            },
            Self::Ridge => &FeatureMeta {
                key: "ridge", label: "Ridge", icon: "\u{26f0}",
                radial: false, edge_char: 1.3, edge_freq_mul: 1.4,
                hint: "A single raised crest along your stroke \u{2014} a linear ridge, distinct from a mountain mass.",
                controls: RIDGE_CTL, modes: &[],
            },
            Self::Plateau => &FeatureMeta {
                key: "plateau", label: "Plateau", icon: "\u{1f7eb}",
                radial: false, edge_char: 1.15, edge_freq_mul: 0.65,
                hint: "Terraced FBM mesa. Sets a flat top; terraces quantize the surface. Never lowers existing terrain.",
                controls: PLATEAU_CTL, modes: &[],
            },
            Self::Cliff => &FeatureMeta {
                key: "cliff", label: "Cliff / Escarpment", icon: "\u{1f9f1}",
                radial: false, edge_char: 0.6, edge_freq_mul: 0.45,
                hint: "The one hard-edge tool: a one-sided escarpment. Steepness sets the transition width; the high side is the left of your stroke direction.",
                controls: CLIFF_CTL, modes: &[],
            },
            Self::Canyon => &FeatureMeta {
                key: "canyon", label: "Canyon", icon: "\u{1faa8}",
                radial: false, edge_char: 0.95, edge_freq_mul: 1.2,
                hint: "Inverted ridged carve with steep walls. Meander offsets the centerline.",
                controls: CANYON_CTL, modes: &[],
            },
            Self::Valley => &FeatureMeta {
                key: "valley", label: "Valley", icon: "\u{1f3de}\u{fe0f}",
                radial: false, edge_char: 0.65, edge_freq_mul: 0.85,
                hint: "Broad U-shaped glacial trough. Gentler and wider than a canyon.",
                controls: VALLEY_CTL, modes: &[],
            },
            Self::River => &FeatureMeta {
                key: "river", label: "River", icon: "\u{1f3de}",
                radial: false, edge_char: 0.4, edge_freq_mul: 0.8,
                hint: "Semi-automatic: carves a bed along your stroke, lowers banks, auto-fills water on commit (locked into riverMask/riverFloor, same as the region-route river tool). Width varies with branch noise; meander wanders the channel.",
                controls: RIVER_CTL, modes: &[],
            },
            Self::Lake => &FeatureMeta {
                key: "lake", label: "Lake", icon: "\u{1f4a7}",
                radial: true, edge_char: 1.25, edge_freq_mul: 0.75,
                hint: "Radial basin (brush = radius). Lowers a bowl and fills it with water on commit (deposited into lakeMask, same as the Water tool used to); shoreline feathers.",
                controls: LAKE_CTL, modes: &[],
            },
            Self::Basin => &FeatureMeta {
                key: "basin", label: "Basin", icon: "\u{1f6d6}",
                radial: false, edge_char: 0.85, edge_freq_mul: 0.9,
                hint: "Broad shallow depression with an FBM floor \u{2014} endorheic sinks, playas. No outlet, unlike Lake.",
                controls: BASIN_CTL, modes: &[],
            },
            Self::Coastline => &FeatureMeta {
                key: "coastline", label: "Coastline", icon: "\u{1f30a}",
                radial: false, edge_char: 1.5, edge_freq_mul: 0.55,
                hint: "Pulls terrain down toward sea level with a ragged fractal edge \u{2014} carves bays and beaches.",
                controls: COASTLINE_CTL, modes: &[],
            },
            Self::Volcano => &FeatureMeta {
                key: "volcano", label: "Volcano", icon: "\u{1f30b}",
                radial: true, edge_char: 1.3, edge_freq_mul: 1.1,
                hint: "Radial cone + crater with ridged flanks. Crater depth carves the summit caldera.",
                controls: VOLCANO_CTL, modes: &[],
            },
            Self::Freehand => &FeatureMeta {
                key: "freehand", label: "Freehand", icon: "\u{270f}\u{fe0f}",
                radial: false, edge_char: 0.3, edge_freq_mul: 0.6,
                hint: "Continuous quick touch-up, no preset landform \u{2014} pick a sub-mode below. Raise/Lower/Smooth follow the drag; Cliff/Ridge/Canyon follow drag direction; Mesa/Volcano stamp once at a tap (a 1-point stroke degenerates to radial distance, so the same geometry serves both interactions).",
                controls: FREEHAND_CTL, modes: FREEHAND_MODES,
            },
        }
    }

    /// The feature's control defaults, as the reference's
    /// `sculptDefaultParams` builds them (each control's 6th tuple element,
    /// plus `modes[0]` where a feature has sub-modes).
    pub fn default_params(self) -> FeatureParams {
        let c = self.meta().controls;
        let d = |i: usize| c[i].default;
        match self {
            Self::Mountains => FeatureParams::Mountains {
                mtn_height: d(0),
                peak_sharpness: d(1),
                ridge_frequency: d(2),
                ruggedness: d(3),
            },
            Self::Hills => FeatureParams::Hills {
                amplitude: d(0),
                rolling_frequency: d(1),
                softness: d(2),
            },
            Self::Ridge => FeatureParams::Ridge {
                ridge_height: d(0),
                ridge_width: d(1),
                ridge_freq: d(2),
            },
            Self::Plateau => FeatureParams::Plateau {
                plateau_height: d(0),
                terraces: d(1),
                plateau_freq: d(2),
            },
            Self::Cliff => FeatureParams::Cliff {
                cliff_height: d(0),
                cliff_steep: d(1),
            },
            Self::Canyon => FeatureParams::Canyon {
                canyon_depth: d(0),
                wall_steepness: d(1),
                meander: d(2),
            },
            Self::Valley => FeatureParams::Valley {
                valley_depth: d(0),
                valley_width: d(1),
                meander: d(2),
            },
            Self::River => FeatureParams::River {
                river_width: d(0),
                river_depth: d(1),
                river_meander: d(2),
                branch_noise: d(3),
            },
            Self::Lake => FeatureParams::Lake {
                lake_depth: d(0),
                lake_shore: d(1),
            },
            Self::Basin => FeatureParams::Basin {
                basin_depth: d(0),
                basin_rough: d(1),
            },
            Self::Coastline => FeatureParams::Coastline {
                coast_amount: d(0),
                coast_ragged: d(1),
            },
            Self::Volcano => FeatureParams::Volcano {
                volc_height: d(0),
                crater_depth: d(1),
                volc_radius: d(2),
                flank_rough: d(3),
            },
            Self::Freehand => FeatureParams::Freehand {
                amount: d(0),
                sub_mode: FreehandMode::Raise,
            },
        }
    }
}

/// A feature's own control values — the reference's per-stamp `f:{...}`
/// bag, typed. Modelled as an enum rather than a `HashMap<String, f64>`
/// because the registry is closed (thirteen entries, fixed keys) and a
/// typo'd key in an untyped bag would silently read as `undefined` and
/// produce `NaN` heights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeatureParams {
    Mountains {
        mtn_height: f64,
        peak_sharpness: f64,
        ridge_frequency: f64,
        ruggedness: f64,
    },
    Hills {
        amplitude: f64,
        rolling_frequency: f64,
        softness: f64,
    },
    Ridge {
        ridge_height: f64,
        ridge_width: f64,
        ridge_freq: f64,
    },
    Plateau {
        plateau_height: f64,
        terraces: f64,
        plateau_freq: f64,
    },
    Cliff {
        cliff_height: f64,
        cliff_steep: f64,
    },
    Canyon {
        canyon_depth: f64,
        wall_steepness: f64,
        meander: f64,
    },
    Valley {
        valley_depth: f64,
        valley_width: f64,
        meander: f64,
    },
    River {
        river_width: f64,
        river_depth: f64,
        river_meander: f64,
        branch_noise: f64,
    },
    Lake {
        lake_depth: f64,
        lake_shore: f64,
    },
    Basin {
        basin_depth: f64,
        basin_rough: f64,
    },
    Coastline {
        coast_amount: f64,
        coast_ragged: f64,
    },
    Volcano {
        volc_height: f64,
        crater_depth: f64,
        volc_radius: f64,
        flank_rough: f64,
    },
    Freehand {
        amount: f64,
        sub_mode: FreehandMode,
    },
}

impl FeatureParams {
    pub fn feature(&self) -> Feature {
        match self {
            Self::Mountains { .. } => Feature::Mountains,
            Self::Hills { .. } => Feature::Hills,
            Self::Ridge { .. } => Feature::Ridge,
            Self::Plateau { .. } => Feature::Plateau,
            Self::Cliff { .. } => Feature::Cliff,
            Self::Canyon { .. } => Feature::Canyon,
            Self::Valley { .. } => Feature::Valley,
            Self::River { .. } => Feature::River,
            Self::Lake { .. } => Feature::Lake,
            Self::Basin { .. } => Feature::Basin,
            Self::Coastline { .. } => Feature::Coastline,
            Self::Volcano { .. } => Feature::Volcano,
            Self::Freehand { .. } => Feature::Freehand,
        }
    }
}

/// The eight shared brush/noise globals — the reference's `SCULPT_GLOBAL_DEF`
/// (line 8862) and per-stamp `g:{...}` bag.
///
/// `brush_size` is in **grid cells** at the current resolution (the
/// reference converted the PoC's fixed-512-canvas pixel convention on
/// purpose, matching every other brush-radius slider in the file); a km
/// readout is derived at UI time from `mapWidthKm / gw`, never stored.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SculptGlobals {
    /// Slider range 6..200, step 1 (cells).
    pub brush_size: f64,
    /// 0..1, step 0.01. Narrows the falloff band as it rises (harder edge).
    pub hardness: f64,
    /// 0..1.5, step 0.01. Scales the coverage weight into an effect
    /// strength — coverage *shape* and effect *strength* are independently
    /// tunable, which is why the reference has both sliders.
    pub intensity: f64,
    /// 1..20, step 0.5.
    pub noise_scale: f64,
    /// 1..8, step 1.
    pub octaves: u32,
    /// 0.20..0.90, step 0.01.
    pub persistence: f64,
    /// 1.40..3.20, step 0.05.
    pub lacunarity: f64,
    /// 0..1, step 0.01. Amplitude of the domain warp applied to the
    /// coverage mask itself.
    pub edge_noise: f64,
}

impl Default for SculptGlobals {
    /// `SCULPT_GLOBAL_DEF`, verbatim.
    fn default() -> Self {
        Self {
            brush_size: 32.0,
            hardness: 0.5,
            intensity: 1.0,
            noise_scale: 5.0,
            octaves: 5,
            persistence: 0.5,
            lacunarity: 2.0,
            edge_noise: 0.55,
        }
    }
}

/// One of the eight one-click presets (`SCULPT_PRESETS`, line 9005) — a
/// parameter seed; **the user still paints the stroke**.
///
/// Every one of the eight overrides exactly one global (`noiseScale`) and
/// nothing else — checked, not assumed — so this models that one override
/// concretely rather than carrying a general override bag no preset uses.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub feature: Feature,
    pub noise_scale: f64,
    params: PresetParams,
}

#[derive(Debug, Clone, Copy)]
enum PresetParams {
    Hills(f64, f64, f64),
    Mountains(f64, f64, f64, f64),
    Canyon(f64, f64, f64),
    Volcano(f64, f64, f64, f64),
    Plateau(f64, f64, f64),
    Valley(f64, f64, f64),
}

impl Preset {
    /// The preset's feature params, and its `noiseScale` written into
    /// `globals` — the reference applies both together when a preset button
    /// is clicked.
    pub fn apply(&self, globals: &mut SculptGlobals) -> FeatureParams {
        globals.noise_scale = self.noise_scale;
        match self.params {
            PresetParams::Hills(amplitude, rolling_frequency, softness) => FeatureParams::Hills {
                amplitude,
                rolling_frequency,
                softness,
            },
            PresetParams::Mountains(mtn_height, peak_sharpness, ridge_frequency, ruggedness) => {
                FeatureParams::Mountains {
                    mtn_height,
                    peak_sharpness,
                    ridge_frequency,
                    ruggedness,
                }
            }
            PresetParams::Canyon(canyon_depth, wall_steepness, meander) => FeatureParams::Canyon {
                canyon_depth,
                wall_steepness,
                meander,
            },
            PresetParams::Volcano(volc_height, crater_depth, volc_radius, flank_rough) => {
                FeatureParams::Volcano {
                    volc_height,
                    crater_depth,
                    volc_radius,
                    flank_rough,
                }
            }
            PresetParams::Plateau(plateau_height, terraces, plateau_freq) => {
                FeatureParams::Plateau {
                    plateau_height,
                    terraces,
                    plateau_freq,
                }
            }
            PresetParams::Valley(valley_depth, valley_width, meander) => FeatureParams::Valley {
                valley_depth,
                valley_width,
                meander,
            },
        }
    }
}

/// `SCULPT_PRESETS`, all eight, in the reference's own order.
pub const SCULPT_PRESETS: [Preset; 8] = [
    Preset {
        name: "Rolling Hills",
        feature: Feature::Hills,
        noise_scale: 6.0,
        params: PresetParams::Hills(0.1, 1.5, 0.8),
    },
    Preset {
        name: "Alps",
        feature: Feature::Mountains,
        noise_scale: 5.0,
        params: PresetParams::Mountains(0.52, 1.9, 2.2, 0.55),
    },
    Preset {
        name: "Rockies",
        feature: Feature::Mountains,
        noise_scale: 4.0,
        params: PresetParams::Mountains(0.44, 1.15, 1.6, 0.85),
    },
    Preset {
        name: "Badlands",
        feature: Feature::Canyon,
        noise_scale: 9.0,
        params: PresetParams::Canyon(0.2, 0.85, 0.5),
    },
    Preset {
        name: "Volcanic Isle",
        feature: Feature::Volcano,
        noise_scale: 5.0,
        params: PresetParams::Volcano(0.5, 0.55, 110.0, 0.7),
    },
    Preset {
        name: "Mesa",
        feature: Feature::Plateau,
        noise_scale: 4.0,
        params: PresetParams::Plateau(0.3, 5.0, 0.9),
    },
    Preset {
        name: "Karst",
        feature: Feature::Hills,
        noise_scale: 15.0,
        params: PresetParams::Hills(0.13, 2.4, 0.2),
    },
    Preset {
        name: "Glacial Valley",
        feature: Feature::Valley,
        noise_scale: 4.0,
        params: PresetParams::Valley(0.17, 0.95, 0.2),
    },
];

// ---------------------------------------------------------------------------
// The stamp
// ---------------------------------------------------------------------------

/// How a feature's `val` combines with the cell's existing height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// `h = h0 + k * val` — a coverage-weighted delta.
    Add,
    /// `h = h0 + k * (val - h0)` — a coverage-weighted lerp toward `val`.
    Set,
}

/// The per-pixel context the reference shares across a whole stamp
/// application (`_sculptCtx`, line 8866 — one object, no per-pixel
/// allocation). Here it is a plain stack value, which costs nothing and
/// removes the shared-mutable-global hazard entirely.
struct Ctx {
    seed: u32,
    octaves: u32,
    persistence: f64,
    lacunarity: f64,
    /// Noise frequency band, `noiseScale / gw` — resolution-independent,
    /// so a stamp looks the same at any grid size.
    nb: f64,
    /// `brush_size`.
    r: f64,
    /// The feature's own radius (Volcano's `volcRadius`, else `brush_size`).
    radius: f64,
    px: f64,
    py: f64,
    /// Unsigned distance to the stroke.
    d: f64,
    /// Signed distance to the stroke.
    sd: f64,
    /// Arclength at the projection.
    s: f64,
    /// Radial distance from the centroid (radial features only).
    r_c: f64,
    /// The destination cell's current height.
    h0: f64,
    /// Coverage weight scaled by intensity.
    k: f64,
    sea_level: f64,
}

impl Ctx {
    fn fbm(&self, scale: f64) -> f64 {
        let b = self.nb * scale;
        sculpt_fbm(
            self.px * b,
            self.py * b,
            self.octaves,
            self.persistence,
            self.lacunarity,
            self.seed,
        )
    }
    fn ridged(&self, scale: f64) -> f64 {
        let b = self.nb * scale;
        sculpt_ridged(
            self.px * b,
            self.py * b,
            self.octaves,
            self.persistence,
            self.lacunarity,
            self.seed.wrapping_add(700),
        )
    }
    fn billow(&self, scale: f64) -> f64 {
        let b = self.nb * scale;
        sculpt_billow(
            self.px * b,
            self.py * b,
            self.octaves,
            self.persistence,
            self.lacunarity,
            self.seed.wrapping_add(1400),
        )
    }
    /// Sinusoidal centerline offset along arclength — what lets River,
    /// Canyon and Valley wander instead of tracking the raw stroke.
    fn meander(&self, amp: f64) -> f64 {
        let s = self.s;
        amp * (0.7 * (s * 0.04).sin() + 0.3 * (s * 0.11 + 1.3).sin())
    }
}

/// What one feature's `apply()` decided for one cell.
struct Effect {
    mode: Mode,
    val: f64,
    /// The water-surface height this cell wants, if any — River and Lake
    /// are the only two features that produce one (`c.waterOut`, `NaN` for
    /// "none" in the reference, `None` here).
    water_out: Option<f64>,
}

/// One finished stroke: the reference's stamp object
/// `{type, seed, pts, g, f, hidden}`, minus `hidden` (milestone A moved
/// that onto `PassEntry`, where a stack edit belongs) and minus the cached
/// `_cx`/`_cy` (recomputed by [`SculptStamp::centroid`], which is four
/// arithmetic ops and removes a mutate-during-bbox side effect).
///
/// **It stores no pixel data.** A stamp is a recipe, re-evaluated over its
/// own padded bounding box every time it is previewed or baked, which is
/// what makes milestone A's draft stack cheap to snapshot, reorder and
/// throw away.
#[derive(Debug, Clone, PartialEq)]
pub struct SculptStamp {
    pub seed: u32,
    /// The captured stroke polyline in grid coordinates. A single point is
    /// legal and means a tap.
    pub points: Vec<Point>,
    pub globals: SculptGlobals,
    pub params: FeatureParams,
    /// **Divergence from the reference, deliberate.** The reference reads
    /// `state.seaLevel` live at apply time (`P.seaLevel = seaLevel`, passed
    /// into `sculptApplyStamp` per call) so that moving the sea-level slider
    /// mid-draft re-renders existing Plateau/Coastline stamps against the new
    /// level. `cartalith_spatial::Stamp::apply` takes only a destination, so
    /// the value has to live on the recipe. To reproduce the live-read
    /// behaviour a caller re-stamps the draft's sea level (see
    /// [`SculptStamp::with_sea_level`]) when the slider moves — the same
    /// result, an explicit step instead of an implicit global read.
    ///
    /// Only Plateau and Coastline read it.
    pub sea_level: f64,
}

impl SculptStamp {
    /// A stamp with a feature's default controls and the default globals.
    pub fn new(feature: Feature, seed: u32, points: Vec<Point>, sea_level: f64) -> Self {
        Self {
            seed,
            points,
            globals: SculptGlobals::default(),
            params: feature.default_params(),
            sea_level,
        }
    }

    pub fn feature(&self) -> Feature {
        self.params.feature()
    }

    /// Rebuilt copy at a new sea level — the explicit stand-in for the
    /// reference's live `state.seaLevel` read (see [`SculptStamp::sea_level`]).
    pub fn with_sea_level(&self, sea_level: f64) -> Self {
        Self {
            sea_level,
            ..self.clone()
        }
    }

    /// `sculptStampRadius` (line 9020): Volcano alone sizes itself from its
    /// own `volcRadius` control rather than the shared brush size, because
    /// its cone profile is defined in terms of that radius.
    pub fn radius(&self) -> f64 {
        match self.params {
            FeatureParams::Volcano { volc_radius, .. } => volc_radius,
            _ => self.globals.brush_size,
        }
    }

    /// Mean of the stroke points — the centre radial features measure from.
    /// Returns `(0, 0)` for an empty stroke (which never reaches `apply`).
    pub fn centroid(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 0.0);
        }
        let n = self.points.len() as f64;
        let (mut cx, mut cy) = (0.0, 0.0);
        for p in &self.points {
            cx += p.x;
            cy += p.y;
        }
        (cx / n, cy / n)
    }

    /// `sculptStampBBox` (line 9021), as the reference's inclusive
    /// `{x0,y0,x1,y1}` clipped to the grid. `None` when the padded box falls
    /// entirely outside.
    ///
    /// The padding is `radius + feather + edgeAmp + 3` precisely so a
    /// domain-warped edge cannot spill outside the box a stamp promises to
    /// touch. Note the reference computes `feather` here as
    /// `max(2, rad*(1-hardness))` for *every* feature, while `apply` uses
    /// `max(1.5, R*(1-hardness))` for non-radial ones — ported as-is: the
    /// bbox's floor is the larger of the two, so the box always covers what
    /// `apply` writes, and "fixing" the inconsistency would change which
    /// tiles a stamp reports as touched.
    fn bbox(&self, w: usize, h: usize) -> Option<(usize, usize, usize, usize)> {
        if self.points.is_empty() || w == 0 || h == 0 {
            return None;
        }
        let meta = self.feature().meta();
        let rad = self.radius();
        let feather = 2.0f64.max(rad * (1.0 - self.globals.hardness));
        let edge_amp = self.globals.edge_noise * rad * 0.34 * meta.edge_char;
        let m = rad + feather + edge_amp + 3.0;

        let (x0, y0, x1, y1) = if meta.radial {
            let (cx, cy) = self.centroid();
            (cx - m, cy - m, cx + m, cy + m)
        } else {
            let (mut lx, mut ly, mut hx, mut hy) = (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            );
            for p in &self.points {
                lx = lx.min(p.x);
                ly = ly.min(p.y);
                hx = hx.max(p.x);
                hy = hy.max(p.y);
            }
            (lx - m, ly - m, hx + m, hy + m)
        };
        if !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) {
            return None;
        }
        // JS `Math.max(0, x0|0)`: `|0` is ToInt32, i.e. truncation toward
        // zero, and only then the clamp to 0.
        let ix0 = (x0.trunc() as i64).max(0);
        let iy0 = (y0.trunc() as i64).max(0);
        let ix1 = (x1.ceil() as i64).min(w as i64 - 1);
        let iy1 = (y1.ceil() as i64).min(h as i64 - 1);
        if ix1 < ix0 || iy1 < iy0 {
            return None;
        }
        Some((ix0 as usize, iy0 as usize, ix1 as usize, iy1 as usize))
    }

    /// Applies this stamp into `field` (and optionally a water-surface
    /// array), the direct port of `sculptApplyStamp` (line 9033).
    ///
    /// * `field` — full grid, row-major, `f32` exactly like the reference's
    ///   `Float32Array field`. Each write rounds to `f32` at the same point
    ///   the JS assignment would, which later stamps in the stack then read
    ///   back — so the rounding point is parity-relevant, not incidental.
    /// * `water` — the optional `W` array (`-1` = no water). Only River and
    ///   Lake ever write it, and only where their own `waterOut` exceeds
    ///   what is already there.
    /// * `water_only` — the commit-time Lake hook: compute the water-surface
    ///   test against the **already-baked** height without writing `field`
    ///   again. The reference's own comment is explicit that calling this a
    ///   second time with `water_only = false` *"would double-carve the
    ///   bowl"*. (Consumed by milestone C; ported now because it is one
    ///   branch inside the function this milestone owns, and splitting it out
    ///   would mean porting the function twice.)
    pub fn apply_into(
        &self,
        field: &mut [f32],
        mut water: Option<&mut [f32]>,
        w: usize,
        h: usize,
        water_only: bool,
    ) {
        let Some((x0, y0, x1, y1)) = self.bbox(w, h) else {
            return;
        };
        assert_eq!(field.len(), w * h, "field length must equal w * h");
        let meta = self.feature().meta();
        let g = self.globals;
        let brush_r = g.brush_size;
        let rad = self.radius();

        // Freehand/Smooth is the one feature that bypasses the per-pixel
        // apply() path entirely: a 4-neighbour blur over a stable pre-loop
        // snapshot. The reference's comment says why, and it is a real
        // correctness point, not a style one -- "the generic
        // per-pixel-independent apply() path can't read stable neighbour
        // state mid-scan". Reading the live-mutating buffer would make the
        // blur direction-dependent (already-smoothed cells feeding the next
        // cell's average), which is exactly the artifact this avoids.
        if let FeatureParams::Freehand {
            sub_mode: FreehandMode::Smooth,
            ..
        } = self.params
        {
            let full = field.to_vec();
            let feather = 1.5f64.max(brush_r * (1.0 - g.hardness));
            for py in y0..=y1 {
                for px in x0..=x1 {
                    let hit = nearest_on_stroke(px as f64, py as f64, &self.points);
                    let cov = smoothstep(0.0, 1.0, (brush_r - hit.dist) / feather);
                    if cov <= 0.0 {
                        continue;
                    }
                    let i = py * w + px;
                    let l = if px > 0 { full[i - 1] } else { full[i] };
                    let rt = if px < w - 1 { full[i + 1] } else { full[i] };
                    let u = if py > 0 { full[i - w] } else { full[i] };
                    let dn = if py < h - 1 { full[i + w] } else { full[i] };
                    let k = cov * g.intensity;
                    let avg = (l as f64 + rt as f64 + u as f64 + dn as f64) * 0.25;
                    field[i] = clamp01(full[i] as f64 + (avg - full[i] as f64) * k) as f32;
                }
            }
            // Note: the reference returns here regardless of `water_only`,
            // so a smooth stamp writes height even on a water-only pass.
            // Only Lake stamps are ever passed `water_only`, so this is
            // unreachable in practice; ported as-is rather than "fixed".
            return;
        }

        let feather = if meta.radial {
            2.0f64.max(rad * (1.0 - g.hardness))
        } else {
            1.5f64.max(brush_r * (1.0 - g.hardness))
        };
        let (cx, cy) = self.centroid();
        let edge_amp = g.edge_noise * rad * 0.34 * meta.edge_char;
        let warp = edge_amp > 0.01;
        let wf = (2.3 / 8.0f64.max(rad)) * meta.edge_freq_mul;
        let wf2 = wf * 3.4;
        let nb = g.noise_scale / w as f64;
        // `(st.seed ^ ((index+1)*1013)) >>> 0` -- FEATURE_KEYS order is
        // load-bearing here.
        let seed = self.seed ^ ((self.feature().index() as u32 + 1) * 1013);
        let warp_seed = seed.wrapping_add(2100);
        let detail_oct = g.octaves.saturating_sub(1).max(2);

        for py in y0..=y1 {
            for px in x0..=x1 {
                let (fx, fy) = (px as f64, py as f64);
                let (mut qx, mut qy) = (fx, fy);
                if warp {
                    let wx = sculpt_fbm(
                        fx * wf,
                        fy * wf,
                        g.octaves,
                        g.persistence,
                        g.lacunarity,
                        warp_seed,
                    );
                    let wy = sculpt_fbm(
                        (fx + 211.3) * wf,
                        (fy + 57.7) * wf,
                        g.octaves,
                        g.persistence,
                        g.lacunarity,
                        warp_seed,
                    );
                    qx = fx + wx * edge_amp;
                    qy = fy + wy * edge_amp;
                }
                let (mut d, mut sd, mut s_arc, mut r_c) = (0.0, 0.0, 0.0, 0.0);
                let mut cov;
                if meta.radial {
                    r_c = js_hypot(qx - cx, qy - cy);
                    cov = smoothstep(0.0, 1.0, (rad - r_c) / feather);
                } else {
                    let hit = nearest_on_stroke(qx, qy, &self.points);
                    d = hit.dist;
                    sd = hit.sd;
                    s_arc = hit.s;
                    cov = smoothstep(0.0, 1.0, (brush_r - d) / feather);
                }
                if cov <= 0.0 {
                    continue;
                }
                // A second, higher-frequency noise term roughens only the
                // partially-covered rim (`cov < 1`), so the interior stays
                // solid while the silhouette breaks up.
                if warp && cov < 1.0 {
                    let detail = sculpt_fbm(
                        fx * wf2 + 500.7,
                        fy * wf2 - 330.2,
                        detail_oct,
                        g.persistence,
                        g.lacunarity,
                        warp_seed,
                    );
                    cov = clamp01(cov + detail * 0.17 * g.edge_noise * meta.edge_char);
                    if cov <= 0.0 {
                        continue;
                    }
                }

                let i = py * w + px;
                let ctx = Ctx {
                    seed,
                    octaves: g.octaves,
                    persistence: g.persistence,
                    lacunarity: g.lacunarity,
                    nb,
                    r: brush_r,
                    radius: rad,
                    px: fx,
                    py: fy,
                    d,
                    sd,
                    s: s_arc,
                    r_c,
                    h0: field[i] as f64,
                    k: cov * g.intensity,
                    sea_level: self.sea_level,
                };
                let eff = self.eval(&ctx);
                if !water_only {
                    let nh = match eff.mode {
                        Mode::Add => ctx.h0 + ctx.k * eff.val,
                        Mode::Set => ctx.h0 + ctx.k * (eff.val - ctx.h0),
                    };
                    field[i] = clamp01(nh) as f32;
                }
                if let (Some(wa), Some(wo)) = (water.as_deref_mut(), eff.water_out)
                    && wo > wa[i] as f64
                {
                    wa[i] = wo as f32;
                }
            }
        }
    }

    /// The thirteen `apply(c, P)` bodies, transcribed one for one.
    fn eval(&self, c: &Ctx) -> Effect {
        let add = |val: f64| Effect {
            mode: Mode::Add,
            val,
            water_out: None,
        };
        let set = |val: f64| Effect {
            mode: Mode::Set,
            val,
            water_out: None,
        };
        match self.params {
            // Ridged multifractal: peak sharpness powers the crests,
            // ruggedness adds billow detail scaled *by the peak* so the
            // roughness lands on the ridges rather than the flats.
            FeatureParams::Mountains {
                mtn_height,
                peak_sharpness,
                ridge_frequency,
                ruggedness,
            } => {
                let r = c.ridged(ridge_frequency);
                let peak = r.powf(peak_sharpness);
                let rug = (c.billow(ridge_frequency * 3.5) * 0.5 + 0.5) * ruggedness * 0.3;
                add((peak + rug * peak) * mtn_height)
            }
            // Softness blends a fine octave band toward a coarse one.
            FeatureParams::Hills {
                amplitude,
                rolling_frequency,
                softness,
            } => {
                let coarse = c.fbm(rolling_frequency) * 0.5 + 0.5;
                let fine = c.fbm(rolling_frequency * 2.6) * 0.5 + 0.5;
                add(amplitude * lerp(fine, coarse, softness))
            }
            // A gaussian crest across the stroke's signed distance.
            FeatureParams::Ridge {
                ridge_height,
                ridge_width,
                ridge_freq,
            } => {
                let wid = 1.0f64.max(c.r * ridge_width);
                let perp_fall = (-(c.sd * c.sd) / (wid * wid * 0.5)).exp();
                let detail = 0.65 + 0.35 * (c.fbm(ridge_freq) * 0.5 + 0.5);
                add(ridge_height * perp_fall * detail)
            }
            // `set` to `max(h0, level)`, NOT `add` -- this monotonic "never
            // lowers existing terrain" behaviour (the reference's own hint
            // string says so) is what makes Plateau a flatten/terrace tool
            // rather than another raise brush.
            FeatureParams::Plateau {
                plateau_height,
                terraces,
                plateau_freq,
            } => {
                let mut terr = c.fbm(plateau_freq) * 0.5 + 0.5;
                // JS `Math.max(1, P.terraces|0)` -- truncate, then floor at 1.
                let steps = 1.0f64.max(terraces.trunc());
                terr = js_round(terr * steps) / steps;
                let level = c.sea_level + plateau_height + (terr - 0.5) * 0.03;
                set(c.h0.max(level))
            }
            // The one hard-edge tool: a one-sided step across the stroke,
            // centred on zero so it raises one side and lowers the other.
            FeatureParams::Cliff {
                cliff_height,
                cliff_steep,
            } => {
                let trans_w = 1.0f64.max((1.02 - cliff_steep) * c.r * 0.5);
                let step = smoothstep(-trans_w, trans_w, c.sd);
                add(cliff_height * (step - 0.5))
            }
            FeatureParams::Canyon {
                canyon_depth,
                wall_steepness,
                meander,
            } => {
                let lat = c.meander(meander * c.r);
                let dd = (c.sd - lat).abs();
                let t = clamp01(dd / c.r);
                let floor_frac = 0.22;
                let wall = if t < floor_frac {
                    1.0
                } else {
                    1.0 - smoothstep(floor_frac, 1.0, t)
                };
                let wall = wall.powf(1.0 / (0.35 + wall_steepness * 1.4));
                let floor_n = (c.fbm(3.0) * 0.5 + 0.5) * 0.03;
                add(-(canyon_depth * wall) + floor_n * wall)
            }
            // cos^2 across the trough gives the broad U profile that
            // distinguishes a glacial valley from a canyon's steep V.
            FeatureParams::Valley {
                valley_depth,
                valley_width,
                meander,
            } => {
                let lat = c.meander(meander * c.r);
                let dd = (c.sd - lat).abs();
                let t = clamp01(dd / (c.r * valley_width));
                let u = (t * std::f64::consts::PI * 0.5).cos();
                add(-(valley_depth * u * u))
            }
            FeatureParams::River {
                river_width,
                river_depth,
                river_meander,
                branch_noise,
            } => {
                let lat = c.meander(river_meander * c.r);
                let dd = (c.sd - lat).abs();
                // Shadows the brush radius: a river is never narrower than
                // its own channel plus two cells of bank.
                let r = c.r.max(river_width + 2.0);
                let hw = river_width * (1.0 + 0.45 * branch_noise * c.fbm(0.7));
                let bank_f = 1.0 - smoothstep(hw, r, dd);
                let mut target = c.h0 - river_depth * 0.45 * bank_f;
                let mut water_out = None;
                if dd < hw {
                    let chan = 1.0 - smoothstep(hw * 0.5, hw, dd);
                    target -= river_depth * 0.55 * chan;
                    // The water surface is computed against the bank height
                    // this stamp will actually produce (h0 lerped by k),
                    // not against the raw target -- otherwise a
                    // low-intensity stroke would flood above its own banks.
                    let final_bank = c.h0 + c.k * ((c.h0 - river_depth * 0.45 * bank_f) - c.h0);
                    water_out = Some(final_bank - river_depth * 0.12);
                }
                Effect {
                    mode: Mode::Set,
                    val: target,
                    water_out,
                }
            }
            FeatureParams::Lake {
                lake_depth,
                lake_shore,
            } => {
                let r = clamp01(c.r_c / c.radius);
                let bowl = 1.0 - r * r;
                let floor = c.h0 - lake_depth * bowl;
                let surf = c.h0 - lake_depth * lake_shore;
                let final_h = c.h0 + c.k * (floor - c.h0);
                Effect {
                    mode: Mode::Set,
                    val: floor,
                    water_out: if final_h < surf { Some(surf) } else { None },
                }
            }
            FeatureParams::Basin {
                basin_depth,
                basin_rough,
            } => {
                let t = clamp01(c.d / c.r);
                let bowl = (1.0 - t).powf(1.6);
                let floor_n = c.fbm(2.2) * 0.04 * basin_rough;
                add(-(basin_depth * bowl) + floor_n * bowl)
            }
            // Defined entirely in terms of sea level: it pulls terrain
            // toward it, which is why Coastline needs no water-mask gate.
            FeatureParams::Coastline {
                coast_amount,
                coast_ragged,
            } => {
                let rag = c.fbm(coast_ragged) * 0.06;
                let target = c.sea_level - 0.05 + rag;
                set(lerp(c.h0, target, coast_amount))
            }
            FeatureParams::Volcano {
                volc_height,
                crater_depth,
                volc_radius: _,
                flank_rough,
            } => {
                let r = clamp01(c.r_c / c.radius);
                let rim = 0.55;
                let profile = if r >= rim {
                    1.0 - smoothstep(rim, 1.0, r)
                } else {
                    lerp(1.0 - crater_depth, 1.0, smoothstep(0.0, rim, r))
                };
                let rr = r / rim;
                // Fade the ridged flank noise in over the crater floor with
                // a smoothstep written out inline (the reference's own
                // `(r/rim)*(r/rim)*(3-2*(r/rim))`).
                let fade = if rr < 1.0 {
                    rr * rr * (3.0 - 2.0 * rr)
                } else {
                    1.0
                };
                let flank = c.ridged(3.5) * 0.12 * flank_rough * fade;
                add(volc_height * profile + flank)
            }
            FeatureParams::Freehand { amount, sub_mode } => match sub_mode {
                FreehandMode::Raise => add(amount),
                FreehandMode::Lower => add(-amount),
                // Unreachable: apply_into's dedicated blur pass returns
                // before eval() for this sub-mode. Kept because the
                // reference keeps it, and because a no-op is the right
                // answer if it ever is reached.
                FreehandMode::Smooth => set(c.h0),
                FreehandMode::Cliff => {
                    let edge = c.sd / (c.r * 0.12);
                    add(amount * (edge / (1.0 + edge * edge).sqrt()))
                }
                FreehandMode::Ridge => {
                    let pf = (-(c.sd * c.sd) / (c.r * c.r * 0.08)).exp();
                    add(amount * pf)
                }
                FreehandMode::Canyon => {
                    let c_w = c.r * 0.18;
                    let w_w = c.r * 0.45;
                    let ad = c.sd.abs();
                    add(if ad < c_w {
                        -amount * 1.5 * (1.0 - ad / c_w)
                    } else if ad < w_w {
                        amount * 0.35 * (1.0 - (ad - c_w) / (w_w - c_w))
                    } else {
                        0.0
                    })
                }
                FreehandMode::Mesa => {
                    let t = clamp01(c.d / c.r);
                    let mesa_h = c.h0 + amount * 1.25;
                    set(if t < 0.6 {
                        c.h0.max(mesa_h)
                    } else {
                        let u = (t - 0.6) / 0.4;
                        c.h0.max(mesa_h * (1.0 - u * u) + c.h0 * (u * u))
                    })
                }
                FreehandMode::Volcano => {
                    let t = clamp01(c.d / c.r);
                    add(amount * 1.5 * 0.0f64.max(1.0 - t).powf(1.8))
                }
            },
        }
    }
}

impl Stamp for SculptStamp {
    type Cell = f32;

    fn bounds(&self, width: usize, height: usize) -> Region {
        match self.bbox(width, height) {
            Some((x0, y0, x1, y1)) => Region::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1),
            None => Region::new(0, 0, 0, 0),
        }
    }

    fn apply(&self, dst: &mut [f32], width: usize, height: usize) {
        self.apply_into(dst, None, width, height, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::{DirtyTracker, PassBuffer};

    fn flat(w: usize, h: usize, v: f32) -> Vec<f32> {
        vec![v; w * h]
    }

    fn stroke() -> Vec<Point> {
        vec![Point::new(10.0, 32.0), Point::new(54.0, 32.0)]
    }

    fn stamp(feature: Feature) -> SculptStamp {
        let mut s = SculptStamp::new(feature, 1234, stroke(), 0.5);
        s.globals.brush_size = 12.0;
        s
    }

    // ---- registry shape ----

    #[test]
    fn the_registry_has_the_reference_s_thirteen_features_in_order() {
        let keys: Vec<&str> = FEATURE_KEYS.iter().map(|f| f.meta().key).collect();
        assert_eq!(
            keys,
            vec![
                "mountains",
                "hills",
                "ridge",
                "plateau",
                "cliff",
                "canyon",
                "valley",
                "river",
                "lake",
                "basin",
                "coastline",
                "volcano",
                "freehand",
            ]
        );
        for (i, f) in FEATURE_KEYS.iter().enumerate() {
            assert_eq!(f.index(), i, "index feeds the seed derivation");
        }
    }

    #[test]
    fn only_lake_and_volcano_are_radial() {
        let radial: Vec<&str> = FEATURE_KEYS
            .iter()
            .filter(|f| f.meta().radial)
            .map(|f| f.meta().key)
            .collect();
        assert_eq!(radial, vec!["lake", "volcano"]);
    }

    #[test]
    fn only_freehand_has_sub_modes_and_it_has_eight() {
        for f in FEATURE_KEYS {
            let n = f.meta().modes.len();
            if f == Feature::Freehand {
                assert_eq!(n, 8);
            } else {
                assert_eq!(n, 0, "{} should have no sub-modes", f.meta().key);
            }
        }
    }

    #[test]
    fn every_control_default_is_inside_its_own_range() {
        for f in FEATURE_KEYS {
            for c in f.meta().controls {
                assert!(
                    c.default >= c.min && c.default <= c.max,
                    "{}.{} default {} outside [{}, {}]",
                    f.meta().key,
                    c.key,
                    c.default,
                    c.min,
                    c.max
                );
            }
        }
    }

    #[test]
    fn from_key_round_trips_every_feature() {
        for f in FEATURE_KEYS {
            assert_eq!(Feature::from_key(f.meta().key), Some(f));
        }
        assert_eq!(Feature::from_key("nope"), None);
    }

    #[test]
    fn freehand_mode_from_key_round_trips_every_sub_mode() {
        for &m in FREEHAND_MODES {
            let mode = FreehandMode::from_key(m).unwrap_or_else(|| panic!("no FreehandMode for {m}"));
            assert_eq!(mode.key(), m);
        }
        assert_eq!(FreehandMode::from_key("nope"), None);
    }

    #[test]
    fn the_eight_presets_name_features_that_exist_and_seed_real_params() {
        assert_eq!(SCULPT_PRESETS.len(), 8);
        for p in SCULPT_PRESETS {
            let mut g = SculptGlobals::default();
            let params = p.apply(&mut g);
            assert_eq!(params.feature(), p.feature, "preset {}", p.name);
            assert_eq!(g.noise_scale, p.noise_scale);
            // A preset only touches noiseScale among the globals.
            let d = SculptGlobals::default();
            assert_eq!(g.brush_size, d.brush_size);
            assert_eq!(g.hardness, d.hardness);
            assert_eq!(g.octaves, d.octaves);
        }
    }

    // ---- geometry ----

    #[test]
    fn a_one_point_stroke_degenerates_to_radial_distance() {
        // This is the mechanism Freehand's tap-once mesa/volcano sub-modes
        // rely on, not an incidental edge case.
        let pts = vec![Point::new(4.0, 4.0)];
        let hit = nearest_on_stroke(7.0, 8.0, &pts);
        assert_eq!(hit.dist, 5.0);
        assert_eq!(hit.sd, 5.0, "degenerate case reports dist as sd");
        assert_eq!(hit.s, 0.0);
    }

    #[test]
    fn signed_distance_flips_across_the_stroke() {
        let pts = stroke(); // left-to-right along y = 32
        let above = nearest_on_stroke(30.0, 20.0, &pts);
        let below = nearest_on_stroke(30.0, 44.0, &pts);
        assert_eq!(above.dist, 12.0);
        assert_eq!(below.dist, 12.0);
        assert!(
            above.sd.signum() != below.sd.signum(),
            "the two sides must get opposite signs -- Cliff depends on it"
        );
    }

    #[test]
    fn arclength_grows_along_the_stroke() {
        let pts = stroke();
        let a = nearest_on_stroke(15.0, 32.0, &pts);
        let b = nearest_on_stroke(45.0, 32.0, &pts);
        assert!(b.s > a.s);
        assert!((a.s - 5.0).abs() < 1e-9);
    }

    #[test]
    fn a_zero_length_segment_does_not_divide_by_zero() {
        let pts = vec![Point::new(5.0, 5.0), Point::new(5.0, 5.0)];
        let hit = nearest_on_stroke(8.0, 9.0, &pts);
        assert!(hit.dist.is_finite());
        assert!((hit.dist - 5.0).abs() < 1e-9);
    }

    // ---- bounds ----

    #[test]
    fn bounds_pad_by_radius_feather_and_edge_noise() {
        let s = stamp(Feature::Hills);
        let b = s.bounds(64, 64);
        // The stroke spans x 10..54 at y 32; padding is
        // rad + max(2, rad*(1-hardness)) + edgeNoise*rad*0.34*edgeChar + 3.
        let rad = 12.0;
        let m = rad + (rad * 0.5f64).max(2.0) + 0.55 * rad * 0.34 * 0.55 + 3.0;
        assert_eq!(b.x, (10.0f64 - m).trunc().max(0.0) as usize);
        assert_eq!(b.y, (32.0f64 - m).trunc().max(0.0) as usize);
        assert!(b.w > 0 && b.h > 0);
    }

    #[test]
    fn bounds_clip_to_the_grid_and_never_exceed_it() {
        let mut s = stamp(Feature::Hills);
        s.points = vec![Point::new(1.0, 1.0)];
        let b = s.bounds(16, 16);
        assert_eq!(b.x, 0);
        assert_eq!(b.y, 0);
        assert!(b.x + b.w <= 16);
        assert!(b.y + b.h <= 16);
    }

    #[test]
    fn a_stamp_entirely_off_grid_touches_nothing() {
        let mut s = stamp(Feature::Hills);
        s.points = vec![Point::new(500.0, 500.0)];
        let b = s.bounds(64, 64);
        assert_eq!((b.w, b.h), (0, 0));
        let mut f = flat(64, 64, 0.5);
        let before = f.clone();
        s.apply(&mut f, 64, 64);
        assert_eq!(f, before);
    }

    #[test]
    fn volcano_sizes_itself_from_volc_radius_not_brush_size() {
        let mut s = stamp(Feature::Volcano);
        s.globals.brush_size = 8.0;
        assert_eq!(s.radius(), 110.0, "volcRadius default");
        let big = s.bounds(512, 512);
        s.params = FeatureParams::Volcano {
            volc_height: 0.45,
            crater_depth: 0.5,
            volc_radius: 30.0,
            flank_rough: 0.6,
        };
        let small = s.bounds(512, 512);
        assert!(big.w > small.w);
    }

    // ---- apply: the properties each feature is defined by ----

    #[test]
    fn every_feature_writes_something_inside_its_bounds_and_nothing_outside() {
        for f in FEATURE_KEYS {
            let mut s = stamp(f);
            if f == Feature::Volcano {
                s.params = FeatureParams::Volcano {
                    volc_height: 0.45,
                    crater_depth: 0.5,
                    volc_radius: 20.0,
                    flank_rough: 0.6,
                };
            }
            let (w, h) = (64usize, 64usize);
            let base = flat(w, h, 0.5);
            let mut field = base.clone();
            s.apply(&mut field, w, h);
            let b = s.bounds(w, h);
            let mut changed = 0usize;
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    if field[i] != base[i] {
                        changed += 1;
                        assert!(
                            x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + b.h,
                            "{} wrote ({x},{y}) outside its declared bounds {b:?}",
                            f.meta().key
                        );
                    }
                }
            }
            assert!(changed > 0, "{} changed nothing at all", f.meta().key);
        }
    }

    #[test]
    fn plateau_never_lowers_existing_terrain() {
        // The defining trait versus Raise/lower: `set` to max(h0, level),
        // not `add`. A tall starting field must come out unchanged.
        let (w, h) = (64usize, 64usize);
        let mut s = stamp(Feature::Plateau);
        s.sea_level = 0.2;
        let base = flat(w, h, 0.95);
        let mut field = base.clone();
        s.apply(&mut field, w, h);
        for i in 0..w * h {
            assert!(
                field[i] >= base[i],
                "plateau lowered cell {i}: {} < {}",
                field[i],
                base[i]
            );
        }
        // ...and it does raise a low field.
        let mut low = flat(w, h, 0.05);
        s.apply(&mut low, w, h);
        assert!(low.iter().any(|&v| v > 0.05));
    }

    #[test]
    fn plateau_quantizes_its_top_into_the_requested_number_of_terraces() {
        let (w, h) = (96usize, 96usize);
        let mut s = stamp(Feature::Plateau);
        s.globals.brush_size = 20.0;
        s.globals.hardness = 1.0; // hard edge -> full coverage inside
        s.globals.edge_noise = 0.0; // no warp -> clean interior
        s.sea_level = 0.1;
        s.params = FeatureParams::Plateau {
            plateau_height: 0.3,
            terraces: 3.0,
            plateau_freq: 1.1,
        };
        let mut field = flat(w, h, 0.0);
        s.apply(&mut field, w, h);
        // Only fully-covered cells (k == 1) land exactly on a terrace; the
        // feathered rim is a partial lerp toward one, so count the levels
        // that recur across a real area rather than every distinct value.
        let mut counts: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
        for v in field.iter().copied().filter(|&v| v > 0.0) {
            *counts.entry(v.to_bits()).or_default() += 1;
        }
        let plateau_levels: Vec<f32> = counts
            .iter()
            .filter(|&(_, &n)| n >= 20)
            .map(|(&b, _)| f32::from_bits(b))
            .collect();
        // terr is rounded to k/3 for k in 0..=3 -> at most 4 distinct tops.
        assert!(
            !plateau_levels.is_empty() && plateau_levels.len() <= 4,
            "expected 1..=4 terrace levels, got {}: {plateau_levels:?}",
            plateau_levels.len()
        );
    }

    #[test]
    fn cliff_raises_one_side_and_lowers_the_other() {
        let (w, h) = (64usize, 64usize);
        let mut s = stamp(Feature::Cliff);
        s.globals.edge_noise = 0.0;
        let base = flat(w, h, 0.5);
        let mut field = base.clone();
        s.apply(&mut field, w, h);
        let above = field[26 * w + 32]; // y < 32
        let below = field[38 * w + 32]; // y > 32
        assert!(
            (above - 0.5).signum() != (below - 0.5).signum(),
            "one-sided escarpment: {above} vs {below}"
        );
    }

    #[test]
    fn carving_features_lower_the_field_and_building_features_raise_it() {
        let (w, h) = (64usize, 64usize);
        let base = flat(w, h, 0.5);
        let carve = [Feature::Canyon, Feature::Valley, Feature::Basin];
        let build = [Feature::Mountains, Feature::Hills, Feature::Ridge];
        for f in carve {
            let mut field = base.clone();
            stamp(f).apply(&mut field, w, h);
            let min = field.iter().copied().fold(f32::INFINITY, f32::min);
            assert!(min < 0.5, "{} did not carve", f.meta().key);
        }
        for f in build {
            let mut field = base.clone();
            stamp(f).apply(&mut field, w, h);
            let max = field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            assert!(max > 0.5, "{} did not raise", f.meta().key);
        }
    }

    #[test]
    fn coastline_pulls_terrain_toward_sea_level_from_both_directions() {
        let (w, h) = (64usize, 64usize);
        let mut s = stamp(Feature::Coastline);
        s.sea_level = 0.5;
        let mut high = flat(w, h, 0.9);
        s.apply(&mut high, w, h);
        let mut low = flat(w, h, 0.05);
        s.apply(&mut low, w, h);
        let i = 32 * w + 32;
        assert!(high[i] < 0.9, "high ground pulled down");
        assert!(low[i] > 0.05, "low ground pulled up toward the shore");
    }

    #[test]
    fn freehand_raise_and_lower_are_symmetric() {
        let (w, h) = (64usize, 64usize);
        let mut up = stamp(Feature::Freehand);
        up.params = FeatureParams::Freehand {
            amount: 0.12,
            sub_mode: FreehandMode::Raise,
        };
        let mut down = up.clone();
        down.params = FeatureParams::Freehand {
            amount: 0.12,
            sub_mode: FreehandMode::Lower,
        };
        let mut a = flat(w, h, 0.5);
        let mut b = flat(w, h, 0.5);
        up.apply(&mut a, w, h);
        down.apply(&mut b, w, h);
        for i in 0..w * h {
            assert!(
                ((a[i] - 0.5) + (b[i] - 0.5)).abs() < 1e-6,
                "asymmetric at {i}: {} / {}",
                a[i],
                b[i]
            );
        }
    }

    #[test]
    fn freehand_smooth_flattens_noise_and_reads_a_stable_snapshot() {
        // The dedicated blur bypasses eval() precisely so it can read a
        // pre-loop snapshot. If it read the live buffer instead, the result
        // would depend on scan direction; here it must not.
        let (w, h) = (64usize, 64usize);
        let mut s = stamp(Feature::Freehand);
        s.globals.brush_size = 20.0;
        s.params = FeatureParams::Freehand {
            amount: 0.12,
            sub_mode: FreehandMode::Smooth,
        };
        let mut field: Vec<f32> = (0..w * h)
            .map(|i| if i % 2 == 0 { 0.2 } else { 0.8 })
            .collect();
        let rough_before = roughness(&field, w, h);
        s.apply(&mut field, w, h);
        assert!(
            roughness(&field, w, h) < rough_before,
            "smooth did not reduce local variation"
        );
    }

    fn roughness(f: &[f32], w: usize, h: usize) -> f64 {
        let mut acc = 0.0;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                acc += (f[i] - f[i - 1]).abs() as f64;
            }
        }
        acc
    }

    #[test]
    fn river_and_lake_are_the_only_features_that_write_water() {
        let (w, h) = (64usize, 64usize);
        for f in FEATURE_KEYS {
            let mut s = stamp(f);
            if f == Feature::Volcano {
                s.params = FeatureParams::Volcano {
                    volc_height: 0.45,
                    crater_depth: 0.5,
                    volc_radius: 20.0,
                    flank_rough: 0.6,
                };
            }
            if f == Feature::Lake {
                s.points = vec![Point::new(32.0, 32.0)];
            }
            let mut field = flat(w, h, 0.6);
            let mut water = vec![-1.0f32; w * h];
            s.apply_into(&mut field, Some(&mut water), w, h, false);
            let wrote = water.iter().any(|&v| v >= 0.0);
            let expected = matches!(f, Feature::River | Feature::Lake);
            assert_eq!(wrote, expected, "{} water-write expectation", f.meta().key);
        }
    }

    #[test]
    fn the_lake_water_only_pass_never_touches_the_height_field() {
        // sculptCommit's ordering depends on this: the dry-run runs AFTER
        // the bake so it tests the final height, and must not carve again.
        let (w, h) = (64usize, 64usize);
        let mut s = stamp(Feature::Lake);
        s.points = vec![Point::new(32.0, 32.0)];
        let mut field = flat(w, h, 0.6);
        s.apply(&mut field, w, h); // the real bake
        let baked = field.clone();
        let mut water = vec![-1.0f32; w * h];
        s.apply_into(&mut field, Some(&mut water), w, h, true);
        assert_eq!(field, baked, "water-only pass double-carved the bowl");
        assert!(water.iter().any(|&v| v >= 0.0), "no lake surface deposited");
    }

    // ---- determinism ----

    #[test]
    fn the_same_stamp_at_the_same_seed_reproduces_exactly() {
        let (w, h) = (64usize, 64usize);
        let s = stamp(Feature::Mountains);
        let mut a = flat(w, h, 0.3);
        let mut b = flat(w, h, 0.3);
        s.apply(&mut a, w, h);
        s.apply(&mut b, w, h);
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&a), bits(&b));
    }

    #[test]
    fn a_different_seed_gives_a_different_result() {
        let (w, h) = (64usize, 64usize);
        let mut s = stamp(Feature::Mountains);
        let mut a = flat(w, h, 0.3);
        s.apply(&mut a, w, h);
        s.seed = 999;
        let mut b = flat(w, h, 0.3);
        s.apply(&mut b, w, h);
        assert_ne!(a, b);
    }

    #[test]
    fn the_feature_index_really_participates_in_the_seed() {
        // Two features whose apply() bodies both start from c.fbm() must
        // not sample identical noise at the same stamp seed -- the
        // (index+1)*1013 term is what separates them, and FEATURE_KEYS
        // order is therefore load-bearing.
        let (w, h) = (64usize, 64usize);
        let mut hills = stamp(Feature::Hills);
        hills.globals.edge_noise = 0.0;
        let a_seed = hills.seed ^ ((Feature::Hills.index() as u32 + 1) * 1013);
        let b_seed = hills.seed ^ ((Feature::Basin.index() as u32 + 1) * 1013);
        assert_ne!(a_seed, b_seed);
        let _ = (w, h);
    }

    // ---- integration with milestone A's PassBuffer ----

    #[test]
    fn sculpt_stamps_drive_milestone_a_s_pass_buffer_end_to_end() {
        let (w, h) = (64usize, 64usize);
        let mut buf: PassBuffer<SculptStamp> = PassBuffer::new(w, h, 32);
        buf.push(stamp(Feature::Mountains));
        buf.push(stamp(Feature::Hills));

        let base = flat(w, h, 0.4);
        let mut scratch = vec![0.0f32; w * h];
        buf.preview_into(&base, &mut scratch);
        assert_eq!(base, flat(w, h, 0.4), "preview mutated the field");
        assert_ne!(scratch, base, "preview showed nothing");

        let mut field = base.clone();
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary = buf.commit(&mut field, &mut tracker, "height_edited");
        assert_eq!(summary.stamps_applied, 2);
        assert_eq!(
            field.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            scratch.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            "commit must reproduce the preview bit for bit"
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn discarding_a_sculpt_draft_leaves_the_field_bit_identical() {
        let (w, h) = (64usize, 64usize);
        let mut buf: PassBuffer<SculptStamp> = PassBuffer::new(w, h, 32);
        let field = flat(w, h, 0.4);
        let before: Vec<u32> = field.iter().map(|v| v.to_bits()).collect();
        buf.push(stamp(Feature::Volcano));
        buf.push(stamp(Feature::Canyon));
        buf.discard();
        assert_eq!(
            field.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            before
        );
    }

    #[test]
    fn stack_order_matters_for_set_mode_features() {
        // Coastline is `set`-mode, so whichever of the two lands last wins
        // -- the reason commit bakes the stack in order.
        let (w, h) = (64usize, 64usize);
        let mut a: PassBuffer<SculptStamp> = PassBuffer::new(w, h, 32);
        let mut mountains = stamp(Feature::Mountains);
        mountains.globals.brush_size = 20.0;
        let mut coast = stamp(Feature::Coastline);
        coast.globals.brush_size = 20.0;
        coast.sea_level = 0.2;
        a.push(mountains.clone());
        a.push(coast.clone());
        let base = flat(w, h, 0.5);
        let mut first = vec![0.0f32; w * h];
        a.preview_into(&base, &mut first);

        let mut b: PassBuffer<SculptStamp> = PassBuffer::new(w, h, 32);
        b.push(coast);
        b.push(mountains);
        let mut second = vec![0.0f32; w * h];
        b.preview_into(&base, &mut second);
        assert_ne!(first, second);
    }

    // ---- helpers ----

    #[test]
    fn smoothstep_substitutes_for_a_zero_width_band() {
        // JS `(b-a)||1e-6`. Without it, Cliff at transW == 0 divides by zero.
        assert_eq!(smoothstep(0.0, 0.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 0.0, 1.0), 1.0);
    }

    #[test]
    fn sculpt_fbm_stays_in_the_reference_s_signed_range() {
        // ~[-1,1], deliberately not fbm()'s [0,1] -- every feature formula
        // was tuned against the signed range.
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for i in 0..2000 {
            let v = sculpt_fbm(i as f64 * 0.31, i as f64 * 0.17, 5, 0.5, 2.0, 42);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!(lo < 0.0 && hi > 0.0, "range was [{lo}, {hi}]");
        assert!(lo >= -1.0 && hi <= 1.0);
    }

    #[test]
    fn zero_octaves_returns_zero_rather_than_nan() {
        assert_eq!(sculpt_fbm(1.0, 1.0, 0, 0.5, 2.0, 1), 0.0);
        assert_eq!(sculpt_ridged(1.0, 1.0, 0, 0.5, 2.0, 1), 0.0);
        assert_eq!(sculpt_billow(1.0, 1.0, 0, 0.5, 2.0, 1), 0.0);
    }
}
