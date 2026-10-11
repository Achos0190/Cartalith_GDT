//! Per-layer, non-destructive **adjustment stacks** (`EDITING_TOOLS_SCOPE.md`
//! milestone C, ET-C1; Ruling BX item 4): the model and the colour maths.
//!
//! # What this module is
//!
//! An [`AdjustStack`] is an ordered list of [`Adjustment`]s. Each has a
//! [`AdjustKind`] (Levels, Curves, Hue/Saturation/Lightness in OkLCh, Gradient
//! map), an `enabled` flag and an `opacity`. [`AdjustStack::apply`] runs the
//! enabled entries in order over one colour held as **continuous `f64`
//! channels on the 0..255 scale, with no clamp to bytes and no rounding
//! inside**: the stack is one stage of a float pipeline that quantises exactly
//! once at the end (Ruling AN; `render::finish_rgb`'s single quantiser).
//!
//! # What it must never do
//!
//! * **Wire into rendering.** This is ET-C1: nothing calls it yet. ET-C2 puts
//!   it inside `finish_rgb`, after the grade and before the display matrix.
//! * **Quantise or clamp to bytes.** Levels clips its *input* range (that is
//!   the operator) and Curves is flat beyond its end points, but no stage rounds
//!   and the HSL stage leaves out-of-gamut results unclipped for the final
//!   quantiser to clamp.
//! * **Encode "no adjustment" as a multiply by an identity value.** Identity is
//!   by **control flow** (`MISTAKES.md`, "Change generated output"): an empty
//!   stack, a disabled entry, an opacity of exactly 0 and a parameter set at its
//!   neutral values are all *skipped* ([`Adjustment::is_skipped`]), so the pixels
//!   are never touched and a caller that tests [`AdjustStack::is_identity`] first
//!   is byte-identical to today by construction, not by arithmetic that happens
//!   to round-trip.
//! * **Panic or clamp silently.** [`AdjustStack::validate`] returns an
//!   [`AdjustError`] for any out-of-range or non-finite parameter. `apply` itself
//!   is infallible and, on an *unvalidated* stack, returns unspecified values but
//!   cannot panic; callers (ET-C2's setter) validate on the way in.
//!
//! # Sources
//!
//! * **OkLab / OkLCh:** Björn Ottosson, "A perceptual color space for image
//!   processing", 2020, <https://bottosson.github.io/posts/oklab/>. The repo had
//!   no OkLab code when this was written (`grep -ri oklab` over `crates/` found
//!   none), so the two forward matrices are transcribed from that post; see
//!   [`LINEAR_SRGB_TO_LMS`] and [`LMS_CUBE_ROOT_TO_OKLAB`].
//! * **sRGB transfer function:** IEC 61966-2-1, the same constants as
//!   `render.rs::srgb_eotf` / `srgb_oetf` (those are private there; the two are
//!   re-stated here, mirrored through zero so an out-of-gamut channel stays
//!   continuous, which the render copies do not need).
//! * **Curves:** monotone cubic Hermite interpolation, Fritsch & Carlson,
//!   "Monotone piecewise cubic interpolation", SIAM J. Numer. Anal. 17(2),
//!   1980.
//! * **Gradient map:** `render::ElevationRamp::sample`, reused (scope: "reusing
//!   `ElevationRamp::sample`").

use crate::render::ElevationRamp;
use serde::{Deserialize, Serialize};

/// A colour as three continuous channels on the 0..255 scale. Same shape as
/// `render.rs`'s private `Rgb` alias; the two are interchangeable tuples.
pub type Rgb = (f64, f64, f64);

/// Most entries one stack may hold. **A labelled judgement:** the scope gives no
/// number; 32 is far above any hand-built stack (Photoshop-style panels stop
/// being usable well before) and bounds the per-pixel cost ET-C2 will measure.
pub const MAX_ENTRIES: usize = 32;

/// Fewest and most control points a Curves adjustment may have. **A labelled
/// judgement** ("the count is a judgement", scope section 5): two is the least
/// that defines a curve, 16 is more than a hand-edited tone curve needs and is
/// the size of the fixed tangent array [`curve_eval`] keeps on the stack.
pub const MIN_CURVE_POINTS: usize = 2;
/// See [`MIN_CURVE_POINTS`].
pub const MAX_CURVE_POINTS: usize = 16;

/// Allowed Levels gamma range. **A labelled judgement**, matching the 0.1..10
/// span common raster editors expose; gamma must be strictly positive for the
/// `1/gamma` exponent to exist.
pub const GAMMA_RANGE: (f64, f64) = (0.1, 10.0);

/// Allowed hue rotation in degrees. +-360 is accepted (and is *not* neutral, so
/// the scope's "hue +360 returns its input within 1e-9" is a real test of the
/// maths rather than of a skip).
pub const HUE_RANGE_DEG: (f64, f64) = (-360.0, 360.0);

/// Relative Rec.709 luma weights for the gradient map's luminance. The same
/// weighting `render.rs::luma` documents as "the one weighting this file
/// already used" (ITU-R BT.709), applied to the gamma-encoded channels exactly
/// as the render does, so a gradient map keys on the same number the grade does.
const LUMA_WEIGHTS: (f64, f64, f64) = (0.2126, 0.7152, 0.0722);

/// Why a parameter was refused. Carries where (`entry` is the stack index when
/// the error came from [`AdjustStack::validate`]), which parameter, and a short
/// reason; never a clamped value, because a silent clamp would hide the mistake
/// from the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdjustError {
    /// Index into [`AdjustStack::entries`], or `None` for a stack-level or
    /// single-adjustment check.
    pub entry: Option<usize>,
    /// The parameter name as it appears in the saved form.
    pub param: &'static str,
    /// What was wrong with it.
    pub reason: &'static str,
}

impl std::fmt::Display for AdjustError {
    /// `entry 3: gamma: out of range`, or without the entry prefix.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.entry {
            Some(i) => write!(f, "adjustment {i}: {}: {}", self.param, self.reason),
            None => write!(f, "{}: {}", self.param, self.reason),
        }
    }
}

impl std::error::Error for AdjustError {}

/// Shorthand for building an [`AdjustError`] without an entry index.
fn err(param: &'static str, reason: &'static str) -> AdjustError {
    AdjustError { entry: None, param, reason }
}

/// `v` is finite and within `lo..=hi`. A NaN fails (every comparison is false),
/// which is the point: NaN is never an acceptable parameter.
fn in_range(v: f64, lo: f64, hi: f64) -> bool {
    v.is_finite() && v >= lo && v <= hi
}

/// One adjustment's maths and parameters. Serialised externally tagged with a
/// snake_case name (`{"levels": {...}}`), so the saved form names the kind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustKind {
    /// Photoshop-style Levels, per channel on the 0..255 scale:
    /// `t = clip((x - in_black) / (in_white - in_black), 0, 1)`,
    /// `t' = t^(1/gamma)`, `out = out_black + t' * (out_white - out_black)`.
    /// `gamma > 1` brightens midtones. `out_black > out_white` is allowed and
    /// inverts. **Neutral:** `(0, 255, 1, 0, 255)`.
    Levels {
        /// Input value mapped to 0; `0..=255`, strictly below `in_white`.
        in_black: f64,
        /// Input value mapped to 1; `0..=255`.
        in_white: f64,
        /// Midtone exponent, [`GAMMA_RANGE`].
        gamma: f64,
        /// Output for `t = 0`; `0..=255`.
        out_black: f64,
        /// Output for `t = 1`; `0..=255`.
        out_white: f64,
    },
    /// A tone curve through `points` (x = input, y = output, both `0..=255`),
    /// x strictly increasing, interpolated by monotone cubic Hermite
    /// (Fritsch-Carlson), flat beyond the first and last x. Applied to each RGB
    /// channel. **Neutral:** every point has `x == y`.
    Curves {
        /// Control points, [`MIN_CURVE_POINTS`]..=[`MAX_CURVE_POINTS`] of them.
        points: Vec<(f64, f64)>,
    },
    /// Hue / saturation / lightness in OkLCh. `hue_deg` rotates the (a, b)
    /// plane (lightness untouched, which is the reason for OkLab); `saturation`
    /// scales chroma by `1 + saturation` (`-1` removes all chroma); `lightness`
    /// moves OkLab L toward 1 (`> 0`: `L + l(1-L)`) or toward 0 (`< 0`:
    /// `L(1+l)`). **Neutral:** all three zero.
    Hsl {
        /// Hue rotation, degrees, [`HUE_RANGE_DEG`].
        hue_deg: f64,
        /// Chroma scale offset, `-1..=1`.
        saturation: f64,
        /// Lightness shift, `-1..=1`.
        lightness: f64,
    },
    /// Replace the colour by `ramp` sampled at the colour's Rec.709 luma
    /// (`luma/255`, clamped to 0..1); the ramp stop's alpha mixes the mapped
    /// colour over the original. Never neutral.
    GradientMap {
        /// The stop table; must be non-empty and well-formed ([`validate`]).
        ///
        /// [`validate`]: AdjustKind::validate
        ramp: ElevationRamp,
    },
}

impl AdjustKind {
    /// True when the parameters are exactly the neutral set for this kind, so
    /// [`Adjustment::is_skipped`] can leave the pixels alone. Exact equality is
    /// deliberate: a *nearly* neutral parameter set is a real adjustment, and
    /// only the exact one is promised to be byte-identical. A gradient map is
    /// never neutral.
    pub fn is_neutral(&self) -> bool {
        match self {
            AdjustKind::Levels { in_black, in_white, gamma, out_black, out_white } => {
                *in_black == 0.0 && *in_white == 255.0 && *gamma == 1.0 && *out_black == 0.0 && *out_white == 255.0
            }
            AdjustKind::Curves { points } => points.iter().all(|&(x, y)| x == y),
            AdjustKind::Hsl { hue_deg, saturation, lightness } => *hue_deg == 0.0 && *saturation == 0.0 && *lightness == 0.0,
            AdjustKind::GradientMap { .. } => false,
        }
    }

    /// Refuse out-of-range, non-finite or malformed parameters. Never clamps and
    /// never panics; the first problem found is returned.
    pub fn validate(&self) -> Result<(), AdjustError> {
        match self {
            AdjustKind::Levels { in_black, in_white, gamma, out_black, out_white } => {
                if !in_range(*in_black, 0.0, 255.0) {
                    return Err(err("in_black", "must be a finite value in 0..=255"));
                }
                if !in_range(*in_white, 0.0, 255.0) {
                    return Err(err("in_white", "must be a finite value in 0..=255"));
                }
                if in_black >= in_white {
                    return Err(err("in_black", "must be below in_white"));
                }
                if !in_range(*gamma, GAMMA_RANGE.0, GAMMA_RANGE.1) {
                    return Err(err("gamma", "must be a finite value in 0.1..=10"));
                }
                if !in_range(*out_black, 0.0, 255.0) {
                    return Err(err("out_black", "must be a finite value in 0..=255"));
                }
                if !in_range(*out_white, 0.0, 255.0) {
                    return Err(err("out_white", "must be a finite value in 0..=255"));
                }
                Ok(())
            }
            AdjustKind::Curves { points } => {
                if points.len() < MIN_CURVE_POINTS || points.len() > MAX_CURVE_POINTS {
                    return Err(err("points", "must hold 2..=16 control points"));
                }
                for (i, &(x, y)) in points.iter().enumerate() {
                    if !in_range(x, 0.0, 255.0) {
                        return Err(err("points", "every x must be a finite value in 0..=255"));
                    }
                    if !in_range(y, 0.0, 255.0) {
                        return Err(err("points", "every y must be a finite value in 0..=255"));
                    }
                    if i > 0 && x <= points[i - 1].0 {
                        return Err(err("points", "x must be strictly increasing"));
                    }
                }
                Ok(())
            }
            AdjustKind::Hsl { hue_deg, saturation, lightness } => {
                if !in_range(*hue_deg, HUE_RANGE_DEG.0, HUE_RANGE_DEG.1) {
                    return Err(err("hue_deg", "must be a finite value in -360..=360"));
                }
                if !in_range(*saturation, -1.0, 1.0) {
                    return Err(err("saturation", "must be a finite value in -1..=1"));
                }
                if !in_range(*lightness, -1.0, 1.0) {
                    return Err(err("lightness", "must be a finite value in -1..=1"));
                }
                Ok(())
            }
            AdjustKind::GradientMap { ramp } => {
                let stops = ramp.stops();
                if stops.is_empty() {
                    return Err(err("ramp", "must hold at least one stop"));
                }
                let mut prev = f64::NEG_INFINITY;
                for s in stops {
                    if !in_range(s.at, 0.0, 1.0) {
                        return Err(err("ramp", "every stop position must be a finite value in 0..=1"));
                    }
                    if s.at < prev {
                        return Err(err("ramp", "stops must be sorted by position"));
                    }
                    prev = s.at;
                    if !in_range(s.col.0, 0.0, 255.0) || !in_range(s.col.1, 0.0, 255.0) || !in_range(s.col.2, 0.0, 255.0) {
                        return Err(err("ramp", "every stop channel must be a finite value in 0..=255"));
                    }
                    if !in_range(s.a, 0.0, 1.0) {
                        return Err(err("ramp", "every stop alpha must be a finite value in 0..=1"));
                    }
                }
                Ok(())
            }
        }
    }

    /// The adjusted colour, **at full strength, ignoring `enabled`/`opacity`
    /// and neutrality** (those are [`Adjustment`]'s concern). Unclamped.
    fn apply(&self, c: Rgb) -> Rgb {
        match self {
            AdjustKind::Levels { in_black, in_white, gamma, out_black, out_white } => {
                let f = |x: f64| levels_channel(x, *in_black, *in_white, *gamma, *out_black, *out_white);
                (f(c.0), f(c.1), f(c.2))
            }
            AdjustKind::Curves { points } => {
                let mut m = [0.0f64; MAX_CURVE_POINTS];
                if !pchip_tangents(points, &mut m) {
                    // Not 2..=16 points: invalid by `validate`, which callers
                    // run first. Leave the colour as it is rather than index
                    // out of range.
                    return c;
                }
                (curve_eval(points, &m, c.0), curve_eval(points, &m, c.1), curve_eval(points, &m, c.2))
            }
            AdjustKind::Hsl { hue_deg, saturation, lightness } => hsl_oklch(c, *hue_deg, *saturation, *lightness),
            AdjustKind::GradientMap { ramp } => {
                let y = (LUMA_WEIGHTS.0 * c.0 + LUMA_WEIGHTS.1 * c.1 + LUMA_WEIGHTS.2 * c.2) / 255.0;
                match ramp.sample(y.clamp(0.0, 1.0)) {
                    Some((r, g, b, a)) => mix3(c, (r, g, b), a),
                    // An empty ramp is invalid (`validate`); leave the colour.
                    None => c,
                }
            }
        }
    }
}

/// One entry of a stack: a kind with its parameters, an `enabled` flag and an
/// `opacity`. `enabled` defaults to `true` and `opacity` to `1.0` when absent
/// from a saved document (the saved form of a fresh adjustment may omit both).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Adjustment {
    /// The maths and its parameters.
    pub kind: AdjustKind,
    /// A disabled entry contributes nothing (it is skipped, not applied at
    /// zero strength).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Strength `0..=1`: `out = mix(input, adjusted, opacity)`. Exactly `0` is
    /// skipped; exactly `1` returns the adjusted colour without a mix.
    #[serde(default = "default_opacity")]
    pub opacity: f64,
}

/// serde default for [`Adjustment::enabled`].
fn default_true() -> bool {
    true
}

/// serde default for [`Adjustment::opacity`].
fn default_opacity() -> f64 {
    1.0
}

impl Adjustment {
    /// An enabled, full-opacity entry of `kind`.
    pub fn new(kind: AdjustKind) -> Self {
        Adjustment { kind, enabled: true, opacity: 1.0 }
    }

    /// True when this entry leaves the pixels untouched **by control flow**:
    /// disabled, opacity `<= 0` (exactly 0 for a valid entry), or parameters at
    /// their neutral set. A NaN opacity is not skipped; it propagates NaN so a
    /// bad value is visible, and [`Self::validate`] refuses it first.
    pub fn is_skipped(&self) -> bool {
        !self.enabled || self.opacity <= 0.0 || self.kind.is_neutral()
    }

    /// Validate opacity (`0..=1`, finite) and the kind's parameters.
    pub fn validate(&self) -> Result<(), AdjustError> {
        if !in_range(self.opacity, 0.0, 1.0) {
            return Err(err("opacity", "must be a finite value in 0..=1"));
        }
        self.kind.validate()
    }

    /// Apply this one entry, or return `c` untouched if it
    /// [is skipped](Self::is_skipped).
    pub fn apply(&self, c: Rgb) -> Rgb {
        if self.is_skipped() {
            return c;
        }
        let adjusted = self.kind.apply(c);
        if self.opacity >= 1.0 {
            adjusted
        } else {
            mix3(c, adjusted, self.opacity)
        }
    }
}

/// An ordered, toggleable stack of adjustments for one layer. The saved form is
/// `{"entries": [...]}`; a struct that owns a stack marks the field
/// `#[serde(default, skip_serializing_if = "AdjustStack::is_empty")]` so a
/// document that never used adjustments serialises exactly as it did before
/// this type existed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdjustStack {
    /// The entries, applied first to last.
    #[serde(default)]
    pub entries: Vec<Adjustment>,
}

impl AdjustStack {
    /// True when there are no entries at all. This is the `skip_serializing_if`
    /// predicate (a stack of disabled entries is *user data* and must be saved).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// True when applying the stack would leave every pixel untouched: no entry
    /// is live. A caller tests this first and skips [`Self::apply`] entirely,
    /// which is what makes the default byte-identical by control flow.
    pub fn is_identity(&self) -> bool {
        self.entries.iter().all(Adjustment::is_skipped)
    }

    /// Validate the entry count and every entry; the error carries the entry
    /// index.
    pub fn validate(&self) -> Result<(), AdjustError> {
        if self.entries.len() > MAX_ENTRIES {
            return Err(err("entries", "more than 32 adjustments"));
        }
        for (i, e) in self.entries.iter().enumerate() {
            e.validate().map_err(|mut er| {
                er.entry = Some(i);
                er
            })?;
        }
        Ok(())
    }

    /// Run every live entry in order over `c`, in continuous 0..255 `f64`
    /// with no clamp or rounding between or after entries (the caller's single
    /// quantiser comes later). Skipped entries are not touched.
    pub fn apply(&self, c: Rgb) -> Rgb {
        let mut c = c;
        for e in &self.entries {
            c = e.apply(c);
        }
        c
    }

    /// Parse a saved stack and [validate](Self::validate) it: a syntactically
    /// valid document with an out-of-range parameter is refused rather than
    /// loaded.
    pub fn from_json(s: &str) -> Result<Self, AdjustError> {
        let stack: AdjustStack = serde_json::from_str(s).map_err(|_| err("document", "not a valid adjustment stack"))?;
        stack.validate()?;
        Ok(stack)
    }
}

// ---------------------------------------------------------------------------
// The maths
// ---------------------------------------------------------------------------

/// Per-channel `a + (b - a) * t` on a colour; the same expression as
/// `render.rs::mix`, kept here so this module does not need that private fn.
fn mix3(a: Rgb, b: Rgb, t: f64) -> Rgb {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// One channel of Levels; see [`AdjustKind::Levels`]. The `t` clip is the
/// operator's input range, not a byte clamp.
fn levels_channel(x: f64, in_black: f64, in_white: f64, gamma: f64, out_black: f64, out_white: f64) -> f64 {
    let t = ((x - in_black) / (in_white - in_black)).clamp(0.0, 1.0);
    let t = t.powf(1.0 / gamma);
    out_black + t * (out_white - out_black)
}

/// Fritsch-Carlson monotone tangents for `pts` into `m`; `false` (and `m`
/// untouched) unless `2..=MAX_CURVE_POINTS` points. Interior tangent is the mean
/// of the two secants, or 0 where they differ in sign or either is 0; end
/// tangents are the end secants; then each interval's `(alpha, beta)` is scaled
/// onto the radius-3 circle when `alpha^2 + beta^2 > 9`. The `9` / `3` are the
/// paper's monotonicity region (Fritsch & Carlson 1980, Theorem 2), not tunables.
fn pchip_tangents(pts: &[(f64, f64)], m: &mut [f64; MAX_CURVE_POINTS]) -> bool {
    let n = pts.len();
    if !(MIN_CURVE_POINTS..=MAX_CURVE_POINTS).contains(&n) {
        return false;
    }
    let mut d = [0.0f64; MAX_CURVE_POINTS];
    for i in 0..n - 1 {
        d[i] = (pts[i + 1].1 - pts[i].1) / (pts[i + 1].0 - pts[i].0);
    }
    m[0] = d[0];
    m[n - 1] = d[n - 2];
    for i in 1..n - 1 {
        m[i] = if d[i - 1] * d[i] <= 0.0 { 0.0 } else { (d[i - 1] + d[i]) / 2.0 };
    }
    for i in 0..n - 1 {
        if d[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
            continue;
        }
        let a = m[i] / d[i];
        let b = m[i + 1] / d[i];
        let r2 = a * a + b * b;
        if r2 > 9.0 {
            let tau = 3.0 / r2.sqrt();
            m[i] = tau * a * d[i];
            m[i + 1] = tau * b * d[i];
        }
    }
    true
}

/// Evaluate the curve at `x` given tangents from [`pchip_tangents`]; flat
/// outside the first/last control point.
fn curve_eval(pts: &[(f64, f64)], m: &[f64; MAX_CURVE_POINTS], x: f64) -> f64 {
    let n = pts.len();
    if x <= pts[0].0 {
        return pts[0].1;
    }
    if x >= pts[n - 1].0 {
        return pts[n - 1].1;
    }
    // x is strictly inside the span here, so a bracketing interval exists.
    let mut i = 0;
    while i + 2 < n && pts[i + 1].0 <= x {
        i += 1;
    }
    let (x0, y0) = pts[i];
    let (x1, y1) = pts[i + 1];
    let h = x1 - x0;
    let t = (x - x0) / h;
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * y0 + (t3 - 2.0 * t2 + t) * h * m[i] + (-2.0 * t3 + 3.0 * t2) * y1 + (t3 - t2) * h * m[i + 1]
}

/// Ottosson 2020, linear sRGB to LMS (the post's `M1`, rows l, m, s).
/// Transcribed from the post's published 10-decimal values; the white-point
/// residual of those digits (OkLab b of sRGB white is about 4e-8) is inherent to
/// the published matrices and is why a neutral grey comes back from OkLab a few
/// 1e-8 of the 0..255 scale away from neutral.
pub const LINEAR_SRGB_TO_LMS: [[f64; 3]; 3] = [
    [0.4122214708, 0.5363325363, 0.0514459929],
    [0.2119034982, 0.6806995451, 0.1073969566],
    [0.0883024619, 0.2817188376, 0.6299787005],
];

/// Ottosson 2020, cube-rooted LMS to OkLab (the post's `M2`, rows L, a, b).
pub const LMS_CUBE_ROOT_TO_OKLAB: [[f64; 3]; 3] = [
    [0.2104542553, 0.7936177850, -0.0040720468],
    [1.9779984951, -2.4285922050, 0.4505937099],
    [0.0259040371, 0.7827717662, -0.8086757660],
];

/// The exact inverse of [`LMS_CUBE_ROOT_TO_OKLAB`], computed in exact rational
/// arithmetic from the matrix above (Python `fractions`, rounded to f64 once),
/// **not** the post's 10-digit published inverse: the published inverse is only
/// an approximate inverse of the published forward matrix (round trip off by
/// about 1e-8), which would break the 1e-9 hue-360 guarantee.
const OKLAB_TO_LMS_CUBE_ROOT: [[f64; 3]; 3] = [
    [0.9999999984505198, 0.39633779217376786, 0.2158037580607588],
    [1.0000000088817609, -0.10556134232365635, -0.06385417477170591],
    [1.0000000546724108, -0.08948418209496575, -1.2914855378640917],
];

/// The exact inverse of [`LINEAR_SRGB_TO_LMS`], same derivation and reason.
const LMS_TO_LINEAR_SRGB: [[f64; 3]; 3] = [
    [4.076741661347994, -3.3077115904081933, 0.2309699287294279],
    [-1.268438004092176, 2.6097574006633715, -0.3413193963102196],
    [-0.004196086541837109, -0.7034186144594496, 1.7076147009309448],
];

/// 3x3 matrix times a column vector.
fn mat_vec(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// sRGB EOTF (IEC 61966-2-1) on an encoded `0..1` value, mirrored through zero
/// so an out-of-gamut (negative or above 1) channel stays continuous.
fn eotf(v: f64) -> f64 {
    let a = v.abs();
    let l = if a <= 0.04045 { a / 12.92 } else { ((a + 0.055) / 1.055).powf(2.4) };
    l.copysign(v)
}

/// sRGB OETF, the inverse of [`eotf`], mirrored through zero.
fn oetf(v: f64) -> f64 {
    let a = v.abs();
    let e = if a <= 0.0031308 { a * 12.92 } else { 1.055 * a.powf(1.0 / 2.4) - 0.055 };
    e.copysign(v)
}

/// A 0..255 sRGB colour to OkLab `[L, a, b]` (Ottosson 2020).
pub fn srgb_to_oklab(c: Rgb) -> [f64; 3] {
    let lin = [eotf(c.0 / 255.0), eotf(c.1 / 255.0), eotf(c.2 / 255.0)];
    let lms = mat_vec(&LINEAR_SRGB_TO_LMS, lin);
    mat_vec(&LMS_CUBE_ROOT_TO_OKLAB, [lms[0].cbrt(), lms[1].cbrt(), lms[2].cbrt()])
}

/// OkLab `[L, a, b]` back to a 0..255 sRGB colour, **unclamped**.
pub fn oklab_to_srgb(lab: [f64; 3]) -> Rgb {
    let p = mat_vec(&OKLAB_TO_LMS_CUBE_ROOT, lab);
    let lin = mat_vec(&LMS_TO_LINEAR_SRGB, [p[0] * p[0] * p[0], p[1] * p[1] * p[1], p[2] * p[2] * p[2]]);
    (255.0 * oetf(lin[0]), 255.0 * oetf(lin[1]), 255.0 * oetf(lin[2]))
}

/// Hue rotation, chroma scale and lightness shift in OkLCh; see
/// [`AdjustKind::Hsl`]. Each of the three is applied only when non-zero (control
/// flow, so a zero axis costs and changes nothing). Rotating the (a, b) plane by
/// the angle *is* the hue rotation of OkLCh, with chroma and L untouched.
fn hsl_oklch(c: Rgb, hue_deg: f64, saturation: f64, lightness: f64) -> Rgb {
    let [mut l, mut a, mut b] = srgb_to_oklab(c);
    if hue_deg != 0.0 {
        let th = hue_deg.to_radians();
        let (s, co) = th.sin_cos();
        let (a2, b2) = (a * co - b * s, a * s + b * co);
        a = a2;
        b = b2;
    }
    if saturation != 0.0 {
        let k = 1.0 + saturation;
        a *= k;
        b *= k;
    }
    if lightness > 0.0 {
        l += lightness * (1.0 - l);
    } else if lightness < 0.0 {
        l *= 1.0 + lightness;
    }
    oklab_to_srgb([l, a, b])
}

#[cfg(test)]
mod tests {
    //! Expected values are literals produced by an independent Python reference
    //! (`scratchpad/etc1/ref.py`: its own OkLab with an exact-rational matrix
    //! inverse, its own Fritsch-Carlson, Levels and ramp code), never by the code
    //! under test.
    use super::*;
    use crate::render::RampStop;

    /// Assert two colours agree within `tol` per channel.
    fn close(got: Rgb, want: Rgb, tol: f64) {
        for (g, w, n) in [(got.0, want.0, "r"), (got.1, want.1, "g"), (got.2, want.2, "b")] {
            assert!((g - w).abs() <= tol, "{n}: got {g:.15}, want {w:.15} (tol {tol})");
        }
    }

    /// Bit-exact colour equality.
    fn same_bits(a: Rgb, b: Rgb) -> bool {
        a.0.to_bits() == b.0.to_bits() && a.1.to_bits() == b.1.to_bits() && a.2.to_bits() == b.2.to_bits()
    }

    fn levels(ib: f64, iw: f64, g: f64, ob: f64, ow: f64) -> Adjustment {
        Adjustment::new(AdjustKind::Levels { in_black: ib, in_white: iw, gamma: g, out_black: ob, out_white: ow })
    }
    fn hsl(h: f64, s: f64, l: f64) -> Adjustment {
        Adjustment::new(AdjustKind::Hsl { hue_deg: h, saturation: s, lightness: l })
    }
    fn curve(pts: &[(f64, f64)]) -> Adjustment {
        Adjustment::new(AdjustKind::Curves { points: pts.to_vec() })
    }
    fn gmap() -> Adjustment {
        let stops = [(0.0, (0.0, 0.0, 128.0), 1.0), (0.5, (255.0, 255.0, 0.0), 1.0), (1.0, (255.0, 0.0, 0.0), 0.5)];
        Adjustment::new(AdjustKind::GradientMap {
            ramp: ElevationRamp::normalized(stops.iter().map(|&(at, col, a)| RampStop { at, col, a })),
        })
    }
    fn stack(e: Vec<Adjustment>) -> AdjustStack {
        AdjustStack { entries: e }
    }
    /// A pixel inside the gamut for every HSL pin.
    const PX: Rgb = (130.0, 100.0, 70.0);

    #[test]
    fn identity_empty_disabled_zero_opacity_and_neutral() {
        // Protects: every way a stack can be "off" is skipped by control flow
        // (is_identity true, apply returns the same bits), including fractional
        // and out-of-range inputs a multiply-by-identity could perturb.
        let neutral = [
            levels(0.0, 255.0, 1.0, 0.0, 255.0),
            curve(&[(0.0, 0.0), (255.0, 255.0)]),
            curve(&[(0.0, 0.0), (80.0, 80.0), (255.0, 255.0)]),
            hsl(0.0, 0.0, 0.0),
        ];
        let mut live_off = levels(10.0, 200.0, 2.0, 0.0, 255.0);
        live_off.enabled = false;
        let mut zero_op = hsl(90.0, 0.5, 0.2);
        zero_op.opacity = 0.0;
        let mut entries: Vec<Adjustment> = neutral.to_vec();
        entries.push(live_off);
        entries.push(zero_op);
        let st = stack(entries);
        assert!(AdjustStack::default().is_identity());
        assert!(st.is_identity());
        assert!(!st.is_empty());
        for c in [(0.0, 0.0, 0.0), (130.5, 100.25, 70.125), (255.0, 255.0, 255.0), (-3.0, 300.0, 17.3333)] {
            assert!(same_bits(st.apply(c), c), "{c:?}");
            for e in &st.entries {
                assert!(same_bits(e.apply(c), c));
            }
        }
    }

    #[test]
    fn levels_at_rest_is_bit_exact_directly() {
        // Protects: the scope's "(in 0..255, gamma 1, out 0..255) returns its
        // input bit-exactly", through Adjustment::apply with no caller-side skip.
        let a = levels(0.0, 255.0, 1.0, 0.0, 255.0);
        for i in 0..=2550 {
            let x = i as f64 / 10.0 + 1.0 / 3.0;
            assert_eq!(a.apply((x, x * 0.5, 255.0 - x)).0.to_bits(), x.to_bits());
        }
        // And the formula itself, not the skip, would NOT be bit-exact: this is
        // why identity must be control flow.
        let via_math = levels_channel(1.0 / 3.0 + 77.0, 0.0, 255.0, 1.0, 0.0, 255.0);
        assert!((via_math - (1.0 / 3.0 + 77.0)).abs() < 1e-12);
    }

    #[test]
    fn levels_literal_values() {
        // Protects: the Levels formula (clip, 1/gamma exponent, output range).
        // Reference: ref.py levels((20,110,230), 20,200,2.0,10,240).
        let got = levels(20.0, 200.0, 2.0, 10.0, 240.0).apply((20.0, 110.0, 230.0));
        close(got, (10.0, 172.63455967290594, 240.0), 1e-9);
        // gamma > 1 brightens a midtone; gamma < 1 darkens it.
        let mid = |g: f64| levels(0.0, 255.0, g, 0.0, 255.0).apply((100.0, 100.0, 100.0)).0;
        assert!(mid(2.0) > 100.0 && mid(0.5) < 100.0);
        // Inversion (out_black > out_white) is allowed; opacity 0.25 mix.
        // Reference: ref.py opacity line, mix((130,100,70), invert, 0.25).
        let mut inv = levels(0.0, 255.0, 1.0, 255.0, 0.0);
        inv.opacity = 0.25;
        close(inv.apply(PX), (128.75, 113.75, 98.75), 1e-9);
    }

    #[test]
    fn single_quantisation_chain_stays_float() {
        // Protects: the stack operates in float and never rounds between entries.
        // Reference: ref.py levels2 (second stage fed the unrounded first output).
        let a = levels(20.0, 200.0, 2.0, 10.0, 240.0);
        let b = levels(0.0, 255.0, 0.5, 0.0, 255.0);
        let x = (20.0, 110.0, 230.0);
        let chained = stack(vec![a.clone(), b.clone()]).apply(x);
        close(chained, (0.39215686274509803, 116.87329879787497, 225.88235294117646), 1e-9);
        // The intermediate really is fractional, and rounding it would have moved
        // the answer by far more than the tolerance.
        let mid = a.apply(x);
        assert!(mid.1.fract() != 0.0);
        let rounded = b.apply((mid.0.round(), mid.1.round(), mid.2.round()));
        assert!((rounded.1 - chained.1).abs() > 0.3, "{rounded:?} vs {chained:?}");
    }

    #[test]
    fn hsl_literal_values() {
        // Protects: OkLab matrices, the hue rotation, chroma scale and both
        // lightness branches. Reference: ref.py hsl(...) on (130,100,70).
        let cases: [(Adjustment, Rgb); 5] = [
            (hsl(90.0, 0.0, 0.0), (78.36422443061925, 116.95610657095114, 92.48791038888795)),
            (hsl(0.0, 0.5, 0.0), (140.76021165871418, 95.63630184271128, 46.72955548703234)),
            (hsl(0.0, 0.0, 0.25), (166.36488416514342, 134.78972420009316, 103.75944349247094)),
            (hsl(0.0, 0.0, -0.25), (91.54559737278372, 63.59258857643168, 34.32929653089564)),
            (hsl(90.0, 0.5, 0.2), (87.70525178741599, 150.267185819347, 112.08470860838243)),
        ];
        for (a, want) in cases {
            close(a.apply(PX), want, 1e-8);
        }
    }

    #[test]
    fn hsl_hue_360_returns_input_and_keeps_lightness() {
        // Protects: scope "HSL hue +360 returns its input within 1e-9" through
        // the real maths (+-360 is not neutral, so this is not a skip), and the
        // OkLCh property that a hue rotation does not move L.
        for h in [360.0, -360.0] {
            let a = hsl(h, 0.0, 0.0);
            assert!(!a.is_skipped());
            for c in [PX, (10.0, 200.0, 90.0), (255.0, 0.0, 128.0), (0.0, 0.0, 0.0)] {
                close(a.apply(c), c, 1e-9);
            }
        }
        let l0 = srgb_to_oklab(PX)[0];
        for h in [-170.0, 33.0, 90.0, 250.0] {
            let l1 = srgb_to_oklab(hsl(h, 0.0, 0.0).apply(PX))[0];
            assert!((l0 - l1).abs() < 1e-12, "hue {h}: {l0} vs {l1}");
        }
    }

    #[test]
    fn hsl_saturation_minus_one_is_grey() {
        // Protects: chroma scale 1+s reaching exactly zero chroma. Reference:
        // ref.py hsl_sat-1 (channels agree to ~2e-5 because the published
        // OkLab white point is not exactly neutral, see LINEAR_SRGB_TO_LMS).
        let g = hsl(0.0, -1.0, 0.0).apply(PX);
        close(g, (106.46571180054221, 106.46571628108074, 106.46572840906943), 1e-8);
        assert!((g.0 - g.2).abs() < 1e-4);
    }

    #[test]
    fn hsl_lightness_is_monotone_and_reaches_ends() {
        // Protects: lightness range property: more lightness never darkens, and
        // -1 / +1 drive L to 0 / 1.
        let mut prev = -1.0;
        for i in 0..=20 {
            let l = -1.0 + i as f64 * 0.1;
            let out = hsl(0.0, 0.0, l).apply(PX);
            let big_l = srgb_to_oklab(out)[0];
            assert!(big_l >= prev - 1e-12);
            prev = big_l;
        }
        assert!(srgb_to_oklab(hsl(0.0, 0.0, -1.0).apply(PX))[0].abs() < 1e-12);
        assert!((srgb_to_oklab(hsl(0.0, 0.0, 1.0).apply(PX))[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn oklab_round_trip_and_anchor_values() {
        // Protects: the matrix pair is an exact round trip, and the forward
        // transform lands on Ottosson's published anchors (white L=1, a~b~0;
        // sRGB red L = 0.62792 per the post's table, a = 0.22486, b = 0.12585).
        for c in [PX, (0.0, 0.0, 0.0), (255.0, 255.0, 255.0), (255.0, 0.0, 0.0), (1.0, 2.0, 250.0)] {
            close(oklab_to_srgb(srgb_to_oklab(c)), c, 1e-9);
        }
        let w = srgb_to_oklab((255.0, 255.0, 255.0));
        assert!((w[0] - 1.0).abs() < 1e-8 && w[1].abs() < 1e-8 && w[2].abs() < 1e-7);
        let r = srgb_to_oklab((255.0, 0.0, 0.0));
        assert!((r[0] - 0.6279553606).abs() < 1e-6 && (r[1] - 0.2248630684).abs() < 1e-6 && (r[2] - 0.1258462985).abs() < 1e-6);
    }

    #[test]
    fn curves_literal_values_and_endpoints() {
        // Protects: Fritsch-Carlson tangents, Hermite basis, flat extrapolation,
        // passing through control points. Reference: ref.py pchip() on
        // [(0,0),(64,32),(192,224),(255,255)].
        let pts = [(0.0, 0.0), (64.0, 32.0), (192.0, 224.0), (255.0, 255.0)];
        let a = curve(&pts);
        let xs = [0.0, 32.0, 64.0, 100.0, 128.0, 192.0, 230.0, 255.0];
        let want = [0.0, 12.0, 32.0, 80.36872209821429, 128.06349206349205, 224.0, 245.714095749999, 255.0];
        for (x, w) in xs.iter().zip(want) {
            close(a.apply((*x, *x, *x)), (w, w, w), 1e-9);
        }
        // Flat beyond the end points.
        let b = curve(&[(50.0, 80.0), (200.0, 120.0)]);
        close(b.apply((0.0, 10.0, 255.0)), (80.0, 80.0, 120.0), 1e-12);
        // A flat segment stays flat (monotone, no overshoot).
        let c = curve(&[(0.0, 0.0), (100.0, 100.0), (150.0, 100.0), (255.0, 255.0)]);
        close(c.apply((50.0, 125.0, 200.0)), (62.5, 100.0, 153.5579311089515), 1e-9);
    }

    #[test]
    fn curves_monotone_data_gives_monotone_curve_without_overshoot() {
        // Protects: the monotonicity property (the reason for Fritsch-Carlson)
        // on steep data that would overshoot a plain cubic, including the 9/3
        // limiter branch.
        let pts = [(0.0, 0.0), (10.0, 10.0), (20.0, 240.0), (30.0, 245.0), (255.0, 255.0)];
        let a = curve(&pts);
        let mut prev = f64::NEG_INFINITY;
        for i in 0..=2550 {
            let x = i as f64 / 10.0;
            let y = a.apply((x, x, x)).0;
            assert!(y >= prev - 1e-9, "x={x}");
            assert!((0.0..=255.0).contains(&y));
            prev = y;
        }
    }

    #[test]
    fn gradient_map_literal_values() {
        // Protects: luma weights, ramp sampling, the alpha mix over the original.
        // Reference: ref.py gmap() with stops (0,navy),(0.5,yellow),(1,red a=.5).
        let g = gmap();
        close(g.apply((130.0, 100.0, 70.0)), (208.424, 208.424, 23.37932549019608), 1e-9);
        close(g.apply((0.0, 0.0, 0.0)), (0.0, 0.0, 128.0), 1e-9);
        close(g.apply((255.0, 255.0, 255.0)), (255.0, 127.5, 127.49999999999997), 1e-9);
        close(g.apply((30.0, 60.0, 90.0)), (111.576, 111.576, 71.99322352941176), 1e-9);
        assert!(!g.kind.is_neutral());
    }

    #[test]
    fn stack_order_matters_and_opacity_mixes() {
        // Protects: entries run first to last; opacity is a lerp from the input.
        let a = levels(0.0, 255.0, 1.0, 40.0, 200.0);
        let b = hsl(0.0, 0.0, 0.25);
        let ab = stack(vec![a.clone(), b.clone()]).apply(PX);
        let ba = stack(vec![b.clone(), a.clone()]).apply(PX);
        assert!((ab.0 - ba.0).abs() > 0.1);
        close(ab, b.apply(a.apply(PX)), 0.0);
        let mut half = a.clone();
        half.opacity = 0.5;
        let full = a.apply(PX);
        close(half.apply(PX), ((PX.0 + full.0) / 2.0, (PX.1 + full.1) / 2.0, (PX.2 + full.2) / 2.0), 1e-12);
        // opacity exactly 1 returns the adjusted colour with no mix: over a
        // sweep, `c + (adj - c) * 1` differs from `adj` in the last bit for some
        // pixels, so a mix-at-1 mutant is caught here.
        for i in 0..2000 {
            let c = (i as f64 * 0.1273 + 0.01, 255.0 - i as f64 * 0.0931, (i as f64 * 0.0377) % 255.0);
            assert!(same_bits(a.apply(c), levels_channel_tuple(&a, c)), "{c:?}");
        }
    }

    #[test]
    fn curves_limiter_literal_values() {
        // Protects: the Fritsch-Carlson radius-3 limiter (threshold 9, scale 3).
        // Reference: ref.py-style pchip with the limiter, written independently
        // in Python. Set A is steep enough that the limiter fires at any
        // threshold and would overshoot (to ~165) without it; set B has
        // alpha^2 + beta^2 = 10.58, between 9 and 16, so only the true threshold
        // of 9 limits it.
        let a = curve(&[(0.0, 0.0), (1.0, 100.0), (11.0, 110.0), (255.0, 255.0)]);
        let xs = [0.5, 3.0, 6.0, 9.0, 50.0];
        let want = [62.12504670860829, 104.86437020710954, 108.6903473797703, 109.85927443899647, 118.12015996339416];
        for (x, w) in xs.iter().zip(want) {
            assert!((a.apply((*x, *x, *x)).0 - w).abs() < 1e-9, "A x={x}");
        }
        let b = curve(&[(0.0, 0.0), (1.0, 3.6), (2.0, 4.6), (3.0, 8.2)]);
        for (x, w) in [(1.25, 3.955123782208717), (1.5, 4.1), (1.75, 4.244876217791283)] {
            assert!((b.apply((x, x, x)).0 - w).abs() < 1e-9, "B x={x}");
        }
    }

    /// Levels by direct formula, for the opacity-1 bit check above.
    fn levels_channel_tuple(a: &Adjustment, c: Rgb) -> Rgb {
        match &a.kind {
            AdjustKind::Levels { in_black, in_white, gamma, out_black, out_white } => (
                levels_channel(c.0, *in_black, *in_white, *gamma, *out_black, *out_white),
                levels_channel(c.1, *in_black, *in_white, *gamma, *out_black, *out_white),
                levels_channel(c.2, *in_black, *in_white, *gamma, *out_black, *out_white),
            ),
            _ => unreachable!(),
        }
    }

    #[test]
    fn apply_is_deterministic() {
        // Protects: same input, same bits, every call and across clones.
        let st = stack(vec![levels(10.0, 240.0, 1.3, 5.0, 250.0), hsl(33.0, 0.2, 0.1), curve(&[(0.0, 0.0), (90.0, 70.0), (255.0, 255.0)]), gmap()]);
        let cl = st.clone();
        for c in [PX, (3.0, 250.0, 17.0)] {
            let a = st.apply(c);
            assert!(same_bits(a, st.apply(c)) && same_bits(a, cl.apply(c)));
        }
    }

    #[test]
    fn validation_refuses_bad_parameters_without_panicking() {
        // Protects: every range check refuses (including NaN and infinity) and
        // names the parameter; the boundary values are accepted.
        let nan = f64::NAN;
        let inf = f64::INFINITY;
        let bad: Vec<(Adjustment, &str)> = vec![
            (levels(-0.1, 255.0, 1.0, 0.0, 255.0), "in_black"),
            (levels(0.0, 255.1, 1.0, 0.0, 255.0), "in_white"),
            (levels(100.0, 100.0, 1.0, 0.0, 255.0), "in_black"),
            (levels(0.0, 255.0, 0.0999, 0.0, 255.0), "gamma"),
            (levels(0.0, 255.0, 10.001, 0.0, 255.0), "gamma"),
            (levels(0.0, 255.0, nan, 0.0, 255.0), "gamma"),
            (levels(0.0, 255.0, 1.0, -1.0, 255.0), "out_black"),
            (levels(0.0, 255.0, 1.0, 0.0, 256.0), "out_white"),
            (levels(nan, 255.0, 1.0, 0.0, 255.0), "in_black"),
            (levels(0.0, inf, 1.0, 0.0, 255.0), "in_white"),
            (curve(&[(0.0, 0.0)]), "points"),
            (curve(&[(0.0, 0.0), (0.0, 5.0)]), "points"),
            (curve(&[(0.0, 0.0), (50.0, 300.0)]), "points"),
            (curve(&[(0.0, 0.0), (nan, 5.0)]), "points"),
            (curve(&[(10.0, 0.0), (5.0, 5.0)]), "points"),
            (curve(&(0..17).map(|i| (i as f64 * 10.0, 5.0)).collect::<Vec<_>>()), "points"),
            (hsl(360.1, 0.0, 0.0), "hue_deg"),
            (hsl(0.0, 1.01, 0.0), "saturation"),
            (hsl(0.0, 0.0, -1.01), "lightness"),
            (hsl(nan, 0.0, 0.0), "hue_deg"),
            (hsl(0.0, inf, 0.0), "saturation"),
            (
                Adjustment::new(AdjustKind::GradientMap { ramp: serde_json::from_str(r#"{"stops":[]}"#).unwrap() }),
                "ramp",
            ),
        ];
        for (a, param) in bad {
            let e = a.validate().expect_err(param);
            assert_eq!(e.param, param, "{a:?}");
            // And apply on the invalid entry must not panic.
            let _ = a.apply(PX);
        }
        let mut op = hsl(10.0, 0.0, 0.0);
        for o in [-0.01, 1.01, nan, inf] {
            op.opacity = o;
            assert_eq!(op.validate().unwrap_err().param, "opacity");
        }
        for ok in [
            levels(0.0, 255.0, 0.1, 0.0, 255.0),
            levels(0.0, 255.0, 10.0, 255.0, 0.0),
            hsl(-360.0, -1.0, 1.0),
            hsl(360.0, 1.0, -1.0),
            curve(&(0..16).map(|i| (i as f64 * 17.0, i as f64 * 17.0)).collect::<Vec<_>>()),
            gmap(),
        ] {
            ok.validate().unwrap();
        }
        // Stack level: index is reported; too many entries refused.
        let st = stack(vec![hsl(0.0, 0.0, 0.0), hsl(999.0, 0.0, 0.0)]);
        let e = st.validate().unwrap_err();
        assert_eq!((e.entry, e.param), (Some(1), "hue_deg"));
        assert!(e.to_string().contains("adjustment 1"));
        assert!(stack(vec![hsl(1.0, 0.0, 0.0); 33]).validate().is_err());
        stack(vec![hsl(1.0, 0.0, 0.0); 32]).validate().unwrap();
    }

    /// A struct owning a stack the way ET-C2's holders will.
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Owner {
        name: String,
        #[serde(default, skip_serializing_if = "AdjustStack::is_empty")]
        adjust: AdjustStack,
    }

    #[test]
    fn serde_round_trip_and_empty_stack_omitted() {
        // Protects: saved form round-trips every kind losslessly, an empty stack
        // is omitted so an existing save re-serialises byte-identically, and a
        // document written before the field existed loads with an empty stack.
        let legacy = r#"{"name":"terrain"}"#;
        let o: Owner = serde_json::from_str(legacy).unwrap();
        assert!(o.adjust.is_empty());
        assert_eq!(serde_json::to_string(&o).unwrap(), legacy);

        let mut off = hsl(12.5, -0.25, 0.5);
        off.enabled = false;
        off.opacity = 0.75;
        let st = stack(vec![
            levels(5.0, 250.0, 1.25, 3.0, 240.0),
            curve(&[(0.0, 0.0), (60.0, 90.5), (255.0, 255.0)]),
            off,
            gmap(),
        ]);
        let json = serde_json::to_string(&st).unwrap();
        let back: AdjustStack = serde_json::from_str(&json).unwrap();
        assert_eq!(back, st);
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
        let o2 = Owner { name: "water".into(), adjust: st.clone() };
        let j2 = serde_json::to_string(&o2).unwrap();
        assert!(j2.contains("\"adjust\""));
        assert_eq!(serde_json::from_str::<Owner>(&j2).unwrap(), o2);
        // Defaults: a bare entry is enabled at full opacity.
        let bare: Adjustment = serde_json::from_str(r#"{"kind":{"hsl":{"hue_deg":10.0,"saturation":0.0,"lightness":0.0}}}"#).unwrap();
        assert!(bare.enabled && bare.opacity == 1.0);
    }

    #[test]
    fn from_json_validates_and_refuses_unknown_kinds() {
        // Protects: loading is not a bypass of validation, and garbage is an
        // error, never a panic or a silently-empty stack.
        let ok = r#"{"entries":[{"kind":{"hsl":{"hue_deg":10.0,"saturation":0.0,"lightness":0.0}}}]}"#;
        assert_eq!(AdjustStack::from_json(ok).unwrap().entries.len(), 1);
        let range = r#"{"entries":[{"kind":{"hsl":{"hue_deg":1000.0,"saturation":0.0,"lightness":0.0}}}]}"#;
        assert_eq!(AdjustStack::from_json(range).unwrap_err().param, "hue_deg");
        for junk in [r#"{"entries":[{"kind":{"sepia":{}}}]}"#, "nonsense", r#"{"entries":[{"kind":{"hsl":{}}}]}"#] {
            assert_eq!(AdjustStack::from_json(junk).unwrap_err().param, "document");
        }
    }
}
