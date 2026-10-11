//! A deterministic, streamable brush-stroke engine: pointer samples in, dabs
//! out (`EDITING_TOOLS_SCOPE.md` milestone ET-D1).
//!
//! **Generic machinery with no Cartalith semantics.** Like [`crate::paint`],
//! this module knows nothing of biomes, terrain or layers: a [`Sample`] is a
//! position, an optional pressure, an optional tilt and a timestamp; a [`Dab`]
//! is a position, a size multiplier, a flow multiplier and an angle. What a
//! dab *paints* is the consumer's business. ET-D2 wires it to the paint
//! bridge (`cartalith-godot`'s `paint_bridge.rs` stroke session); the
//! `per_sample` and `finish` additions are ET-D2's.
//!
//! # What it does, in the order it does it
//!
//! 1. **Smoothing** (optional "pulled string", off by default): the stroke
//!    point lags the pointer by at most [`StrokeConfig::smoothing`] cells.
//! 2. **Spacing**: the polyline of (smoothed) samples is walked, and a dab is
//!    emitted every `spacing x diameter` cells, where `diameter` is the
//!    *current* dab's diameter (`base_diameter x size_k`). The distance still
//!    owed to the next dab is carried in `f64` across segments **and across
//!    [`StrokeEngine::feed`] calls**, so the dab list does not depend on how
//!    the samples were chunked.
//! 3. **Dynamics**: size and flow are each driven by one [`Sensor`] through
//!    [`dynamic_value`]'s law. Angle follows the travel direction and/or a
//!    random jitter.
//! 4. **Scatter**: each source dab becomes `1..=16` copies offset
//!    perpendicular (and optionally along) the travel direction.
//! 5. **Symmetry**: every scatter copy is replicated by [`Symmetry`] about a
//!    centre and an axis angle. **Last, on purpose**: the mirrored dab copies
//!    the source's already-final `size_k`, `flow_k` and angle (reflected), so it
//!    is the exact mirror of its source. Drawing fresh jitter per mirrored
//!    copy would make the "mirror" a different dab.
//!
//! # Determinism
//!
//! All randomness is [`stroke_rng`], a pure function of
//! `(stroke seed, dab index, stream)` — no hidden state, so reordering
//! *which* dab asks first changes nothing, and replaying the same samples in
//! any chunking gives byte-equal dabs.
//!
//! # Provenance and its caveat
//!
//! The design follows Krita's and PhotoCraft's brush models (PhotoCraft:
//! `crates/paint/src/{dynamics,brush,symmetry}.rs`, MIT/Apache-2.0, read as
//! documentation only). **Every PhotoCraft figure below was "as read via
//! summary" in `EDITING_TOOLS_SCOPE.md` §2 — a summary of the page, not the
//! file's text — and has not been re-verified here.** No code was copied.
//! Constants without a citation are labelled judgements.
//!
//! # Never
//!
//! - Never encode a missing sensor reading as a plausible value: an absent
//!   pressure is `None`, and the dynamics step returns exactly `1.0` for it
//!   *by control flow* (see [`dynamic_value`]).
//! - Never draw randomness from anything but [`stroke_rng`].
//! - Never apply symmetry before dynamics/jitter.

use std::f64::consts::{PI, TAU};

// ----------------------------------------------------------------------------
// Constants
// ----------------------------------------------------------------------------

/// The default spacing, as a fraction of the dab diameter: the Paint Toolbar
/// artboard's `18 %` (a design value, `EDITING_TOOLS_SCOPE.md` ET-D1 table).
/// PhotoCraft's own default is `0.1` *(as read via summary)*.
pub const DEFAULT_SPACING: f64 = 0.18;

/// The smallest spacing fraction accepted. **Judgement**: below `1 %` a long
/// stroke emits a hundred dabs per diameter for no visible gain, and `0`
/// would never advance.
pub const MIN_SPACING: f64 = 0.01;

/// The largest spacing fraction accepted. **Judgement**: ten diameters apart
/// is already "dotted line"; a larger value is almost certainly a unit error.
pub const MAX_SPACING: f64 = 10.0;

/// The floor under the distance between two dabs, in cells. **Judgement**:
/// `size_k` may legitimately reach `0` (a dynamics `min` of `0` at zero
/// pressure) and `spacing x 0 = 0` would make the walk loop forever; a
/// thousandth of a cell is far below anything a grid can show.
pub const MIN_STEP: f64 = 1e-3;

/// The most scatter copies per source dab. PhotoCraft's cap *(as read via
/// summary)*, quoted in the scope.
pub const MAX_SCATTER_COUNT: u32 = 16;

/// The largest scatter offset in diameters (`= 1000 %`). PhotoCraft's maximum
/// *(as read via summary)*.
pub const MAX_SCATTER_AMOUNT: f64 = 10.0;

/// The most symmetry copies a radial mode may ask for. **Judgement**: a
/// mandala brush above 64 petals is unreadable at any brush size this app
/// draws, and the cap bounds the output per sample.
pub const MAX_RADIAL: u32 = 64;

/// `splitmix64`/`fmix64`-family golden-ratio constant that salts the seed so
/// that seed `0` does not hash to a degenerate chain (`fmix64(0) == 0`). The
/// same constant `paint.rs`'s `cell_dither` salts its key with.
const SEED_SALT: u64 = 0x9E37_79B9_7F4A_7C15;

/// Salt added to the stream id before it is mixed in, so stream `0` is not
/// the identity. **Judgement**: an arbitrary odd 64-bit constant (the
/// fractional digits of a well-known irrational would do equally well); only
/// its being non-zero and odd matters.
const STREAM_SALT: u64 = 0xD6E8_FEB8_6659_FD93;

/// RNG stream that drives the size dynamic's jitter.
const STREAM_SIZE: u32 = 1;
/// RNG stream that drives the flow dynamic's jitter.
const STREAM_FLOW: u32 = 2;
/// RNG stream that drives the angle jitter.
const STREAM_ANGLE: u32 = 3;
/// RNG stream that drives the scatter copy-count jitter.
const STREAM_SCATTER_COUNT: u32 = 4;
/// First RNG stream of the scatter offsets: copy `j` uses `BASE + 2 j` for the
/// perpendicular offset and `BASE + 2 j + 1` for the along-track one. 16
/// copies reach stream 47, clear of the streams above.
const STREAM_SCATTER_BASE: u32 = 16;

// ----------------------------------------------------------------------------
// Public types
// ----------------------------------------------------------------------------

/// One pointer reading.
///
/// Why `Option` on `pressure` and `tilt`: a mouse reports neither, and a pen
/// may report either. **Absent is `None`, never `1.0` or `0.0`** — see the
/// module's "Never" list. Pressure is expected in `[0, 1]` and tilt as the two
/// axis components in `[-1, 1]` (Godot's `InputEventMouseMotion.tilt`
/// convention, **not verified here**); out-of-range finite values are clamped
/// and non-finite ones are treated as absent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// Pointer x in cells (`f64`: the spacing test is to 1e-9 cells).
    pub x: f64,
    /// Pointer y in cells.
    pub y: f64,
    /// Pen pressure in `[0, 1]`, or `None` when the device did not report it.
    pub pressure: Option<f32>,
    /// Pen tilt `(x, y)` in `[-1, 1]`, or `None` when not reported.
    pub tilt: Option<(f32, f32)>,
    /// Event time in milliseconds on any monotonic clock; only differences
    /// are used (for the [`Sensor::Speed`] control).
    pub t_ms: u64,
}

/// One stamp the engine asks the consumer to make.
///
/// All fields are `f64` so the symmetry tests can hold 1e-12 and so equality
/// is bit-for-bit comparable across chunkings. `size_k` and `flow_k` are
/// multipliers on the consumer's own brush size and flow, **not** absolute
/// values: the engine never learns what the consumer's size means beyond the
/// `base_diameter` it spaces by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Dab centre x in cells.
    pub x: f64,
    /// Dab centre y in cells.
    pub y: f64,
    /// Size multiplier in `[min, 1]` (exactly `1.0` when the dynamic is idle).
    pub size_k: f64,
    /// Flow multiplier in `[min, 1]` (exactly `1.0` when the dynamic is idle).
    pub flow_k: f64,
    /// Tip angle in radians, not wrapped to any range.
    pub angle: f64,
}

/// What drives a dynamic (size or flow).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sensor {
    /// No sensor: the control is a constant `1.0` by design, so only the
    /// jitter term acts. **Distinct from a sensor whose reading is absent.**
    Off,
    /// Pen pressure. Absent reading means the dynamic is idle (`1.0`).
    Pressure,
    /// Pen elevation: `1` upright, `0` flat, from the tilt magnitude.
    /// Absent tilt means the dynamic is idle.
    TiltElevation,
    /// Pointer speed against [`StrokeConfig::speed_ref`], `0` still and `1` at
    /// or above the reference. Absent for the first sample or a non-advancing
    /// clock, which means the dynamic is idle.
    Speed,
    /// Fades from `1` at the stroke start to `0` after
    /// [`StrokeConfig::fade_distance`] cells of path. Always present.
    Fade,
}

/// One dynamic's settings: which sensor, the floor, and the jitter.
///
/// The law is PhotoCraft's `(min + (1 - min) x control) x (1 - jitter x rnd)`,
/// clamped to `min` *(as read via summary)*; see [`dynamic_value`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dynamic {
    /// The driving sensor.
    pub sensor: Sensor,
    /// The floor in `[0, 1]`: the value at `control = 0`, and the clamp.
    pub min: f64,
    /// The jitter in `[0, 1]`: fraction of the value a random draw may remove.
    pub jitter: f64,
}

impl Dynamic {
    /// The idle dynamic: no sensor, no floor, no jitter, always exactly `1.0`.
    pub const OFF: Dynamic = Dynamic { sensor: Sensor::Off, min: 0.0, jitter: 0.0 };
}

/// Scatter settings. Defaults (count 1, amount 0) emit exactly the source dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scatter {
    /// Copies per source dab, `1..=`[`MAX_SCATTER_COUNT`].
    pub count: u32,
    /// Fraction in `[0, 1]` of the count a random draw may remove (never below
    /// one copy).
    pub count_jitter: f64,
    /// Offset range in dab diameters, `0..=`[`MAX_SCATTER_AMOUNT`]: each copy
    /// moves by a uniform draw in `[-amount, +amount]` diameters.
    pub amount: f64,
    /// Also offset *along* the travel direction, not just perpendicular.
    pub both_axes: bool,
}

impl Scatter {
    /// Scatter off: one copy, no offset.
    pub const OFF: Scatter =
        Scatter { count: 1, count_jitter: 0.0, amount: 0.0, both_axes: false };
}

/// How a source dab is replicated about a centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symmetry {
    /// No replication.
    None,
    /// Mirror across the vertical axis (rotated by the axis angle): 2 dabs.
    Vertical,
    /// Mirror across the horizontal axis (rotated by the axis angle): 2 dabs.
    Horizontal,
    /// Both mirrors and their composition, the point reflection: 4 dabs.
    /// *Scope wording is "dual (point)"; four copies is this module's reading
    /// of it and is flagged for the owner.*
    Dual,
    /// `n` copies rotated by `2 pi k / n` about the centre, the source being
    /// copy 0: exactly `n` dabs, `1..=`[`MAX_RADIAL`]. The axis angle has no
    /// effect on a radial pattern (it only rotates mirror axes).
    Radial(u32),
}

/// Everything that shapes a stroke. Build with [`StrokeConfig::new`] and
/// override fields; [`StrokeEngine::new`] validates it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeConfig {
    /// The stroke's RNG seed; the same seed and samples give the same dabs.
    pub seed: u64,
    /// The unscaled dab diameter in cells (`> 0`), which spacing is a
    /// fraction of.
    pub base_diameter: f64,
    /// Spacing as a fraction of the current dab diameter,
    /// [`MIN_SPACING`]`..=`[`MAX_SPACING`]; default [`DEFAULT_SPACING`].
    pub spacing: f64,
    /// Spacing off (ET-D2): when `true`, [`StrokeConfig::spacing`] is ignored and every
    /// accepted sample yields exactly one source dab at the (smoothed) sample position,
    /// the first included. This is the "one dab per pointer sample" path the paint tool
    /// had before the engine existed; it exists so a caller can keep that picture while
    /// still using the engine's dynamics, scatter and symmetry. Default `false`.
    pub per_sample: bool,
    /// Pulled-string length in cells; `0` (the default) means smoothing is
    /// off and the stroke follows the raw samples. PhotoCraft's default is
    /// 100 px *(as read via summary)*; here it is cells and off, a judgement.
    pub smoothing: f64,
    /// The size dynamic.
    pub size: Dynamic,
    /// The flow dynamic.
    pub flow: Dynamic,
    /// Speed, in cells per second, that maps to `control = 1` for
    /// [`Sensor::Speed`] (`> 0`). **Judgement**: 1000 cells/s is a brisk
    /// stroke across a typical map.
    pub speed_ref: f64,
    /// Path length in cells over which [`Sensor::Fade`] runs `1 -> 0`
    /// (`> 0`). **Judgement**: 100 cells.
    pub fade_distance: f64,
    /// Tip angle added to everything else, radians.
    pub base_angle: f64,
    /// Add the travel direction to the angle once one is known.
    pub follow_direction: bool,
    /// Angle jitter in `[0, 1]`: the angle moves by up to `jitter x pi` either
    /// way. **Judgement**: `1` is a fully random tip orientation (+/- half a
    /// turn).
    pub angle_jitter: f64,
    /// Scatter settings.
    pub scatter: Scatter,
    /// Symmetry mode.
    pub symmetry: Symmetry,
    /// Symmetry centre x in cells.
    pub symmetry_cx: f64,
    /// Symmetry centre y in cells.
    pub symmetry_cy: f64,
    /// Symmetry axis rotation in radians (mirror modes only).
    pub symmetry_angle: f64,
}

impl StrokeConfig {
    /// A config with every optional behaviour off: spacing [`DEFAULT_SPACING`],
    /// no smoothing, idle dynamics, no scatter, no symmetry.
    ///
    /// `seed` keys the RNG; `base_diameter` is the dab diameter in cells.
    pub fn new(seed: u64, base_diameter: f64) -> Self {
        StrokeConfig {
            seed,
            base_diameter,
            spacing: DEFAULT_SPACING,
            per_sample: false,
            smoothing: 0.0,
            size: Dynamic::OFF,
            flow: Dynamic::OFF,
            speed_ref: 1000.0,
            fade_distance: 100.0,
            base_angle: 0.0,
            follow_direction: false,
            angle_jitter: 0.0,
            scatter: Scatter::OFF,
            symmetry: Symmetry::None,
            symmetry_cx: 0.0,
            symmetry_cy: 0.0,
            symmetry_angle: 0.0,
        }
    }
}

/// A rejected [`StrokeConfig`]. The text names the offending field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeError {
    /// The named field is out of range or not finite.
    Invalid(&'static str),
}

// ----------------------------------------------------------------------------
// Counter RNG and the dynamics law
// ----------------------------------------------------------------------------

/// MurmurHash3's 64-bit finalizer (`fmix64`, public domain) — the same mixer
/// `paint.rs`'s `cell_dither` uses. Private: only [`stroke_rng`] needs it.
fn fmix64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// A uniform draw in `[0, 1)` that is a pure function of
/// `(seed, dab_index, stream)`.
///
/// Why a counter RNG and not a stateful one: the engine must give the same
/// dabs however the samples are chunked, and a stateful generator's output
/// depends on the order of draws. Each argument is mixed in its own `fmix64`
/// round, so `(idx, stream)` pairs do not alias. **Never** replace this with a
/// sequential generator, and never key a draw by anything but these three
/// values (in particular not by a symmetry-copy index: see the module doc).
pub fn stroke_rng(seed: u64, dab_index: u64, stream: u32) -> f64 {
    let a = fmix64(seed ^ SEED_SALT);
    let b = fmix64(a ^ dab_index);
    let c = fmix64(b ^ (stream as u64).wrapping_add(STREAM_SALT));
    (c >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// The dynamics law: `(min + (1 - min) x control) x (1 - jitter x rnd)`,
/// clamped to `min` (PhotoCraft, *as read via summary*).
///
/// `control` is the sensor's reading in `[0, 1]`, or `None` if the sensor has
/// nothing to say. **`None` returns exactly `1.0` by early return**, before
/// any arithmetic, so "no pressure" can never masquerade as a pressure of
/// some default: a test that puts `min`, `jitter` and `rnd` at values where
/// any substituted control would differ pins this. [`Sensor::Off`] is handled
/// by the caller passing `Some(1.0)`: that is a *designed* constant, not an
/// absent reading. `rnd` is always supplied (the caller draws it whether or
/// not `jitter` is zero) so the RNG use does not depend on settings.
pub fn dynamic_value(min: f64, jitter: f64, control: Option<f64>, rnd: f64) -> f64 {
    let Some(control) = control else {
        return 1.0;
    };
    let v = (min + (1.0 - min) * control) * (1.0 - jitter * rnd);
    v.max(min)
}

// ----------------------------------------------------------------------------
// The engine
// ----------------------------------------------------------------------------

/// A sample after cleaning and smoothing, in the engine's own working types.
#[derive(Clone, Copy, Debug)]
struct Pt {
    x: f64,
    y: f64,
    pressure: Option<f64>,
    tilt: Option<(f64, f64)>,
    t_ms: u64,
}

/// What the sensors read at one dab: every field is `None` when unknown.
#[derive(Clone, Copy, Debug)]
struct Readings {
    pressure: Option<f64>,
    tilt: Option<(f64, f64)>,
    /// Cells per millisecond.
    speed: Option<f64>,
    /// Path length from the stroke start to this dab, cells.
    path: f64,
}

/// The streaming engine for one stroke.
///
/// Create one per stroke, [`feed`](StrokeEngine::feed) it samples in any
/// chunking, and read the dabs it appends. All carried state (the owed
/// distance, the smoothing point, the dab counter, the last sample) lives
/// here, which is exactly what makes chunking irrelevant. It never draws and
/// never allocates beyond the caller's `out` vector.
#[derive(Clone, Debug)]
pub struct StrokeEngine {
    cfg: StrokeConfig,
    /// The previous cleaned sample, `None` before the first.
    prev: Option<Pt>,
    /// Distance in cells still owed before the next dab, carried in `f64`
    /// across segments and calls. Always `> 0` between dabs.
    until_next: f64,
    /// Source dabs emitted so far: the RNG's `dab_index`.
    dab_index: u64,
    /// Path length walked so far, cells.
    path: f64,
    /// Unit travel direction of the last non-degenerate segment, `None`
    /// until one exists.
    dir: Option<(f64, f64)>,
    /// The pulled-string point, `None` before the first sample or when
    /// smoothing is off.
    string: Option<(f64, f64)>,
    /// The last raw (unsmoothed) pointer position, for [`StrokeEngine::finish`].
    last_raw: Option<(f64, f64)>,
}

/// `true` when the value is finite and within `[lo, hi]`.
fn in_range(v: f64, lo: f64, hi: f64) -> bool {
    v.is_finite() && v >= lo && v <= hi
}

impl StrokeEngine {
    /// Validate `cfg` and start a stroke.
    ///
    /// Why validate rather than clamp: a clamped nonsense value would be a
    /// plausible-looking brush that nobody asked for. A bad config is a
    /// programming error the caller should see.
    pub fn new(cfg: StrokeConfig) -> Result<Self, StrokeError> {
        let bad = StrokeError::Invalid;
        if !(cfg.base_diameter.is_finite() && cfg.base_diameter > 0.0) {
            return Err(bad("base_diameter"));
        }
        if !in_range(cfg.spacing, MIN_SPACING, MAX_SPACING) {
            return Err(bad("spacing"));
        }
        if !(cfg.smoothing.is_finite() && cfg.smoothing >= 0.0) {
            return Err(bad("smoothing"));
        }
        for (d, name) in [(cfg.size, "size"), (cfg.flow, "flow")] {
            if !in_range(d.min, 0.0, 1.0) || !in_range(d.jitter, 0.0, 1.0) {
                return Err(bad(name));
            }
        }
        if !(cfg.speed_ref.is_finite() && cfg.speed_ref > 0.0) {
            return Err(bad("speed_ref"));
        }
        if !(cfg.fade_distance.is_finite() && cfg.fade_distance > 0.0) {
            return Err(bad("fade_distance"));
        }
        if !cfg.base_angle.is_finite() || !in_range(cfg.angle_jitter, 0.0, 1.0) {
            return Err(bad("angle"));
        }
        let s = cfg.scatter;
        if s.count == 0
            || s.count > MAX_SCATTER_COUNT
            || !in_range(s.count_jitter, 0.0, 1.0)
            || !in_range(s.amount, 0.0, MAX_SCATTER_AMOUNT)
        {
            return Err(bad("scatter"));
        }
        if let Symmetry::Radial(n) = cfg.symmetry
            && (n == 0 || n > MAX_RADIAL)
        {
            return Err(bad("symmetry"));
        }
        if !(cfg.symmetry_cx.is_finite()
            && cfg.symmetry_cy.is_finite()
            && cfg.symmetry_angle.is_finite())
        {
            return Err(bad("symmetry_centre"));
        }
        Ok(StrokeEngine {
            cfg,
            prev: None,
            until_next: 0.0,
            dab_index: 0,
            path: 0.0,
            dir: None,
            string: None,
            last_raw: None,
        })
    }

    /// Feed `samples` and append the dabs they complete to `out`.
    ///
    /// Equivalent to calling [`feed_one`](Self::feed_one) for each sample, so
    /// splitting a sample list across calls anywhere yields byte-equal dabs.
    pub fn feed(&mut self, samples: &[Sample], out: &mut Vec<Dab>) {
        for s in samples {
            self.feed_one(*s, out);
        }
    }

    /// Feed one sample. A sample with a non-finite position is ignored whole
    /// (there is no position to place a dab at, and inventing one would be a
    /// plausible value for "no value").
    pub fn feed_one(&mut self, s: Sample, out: &mut Vec<Dab>) {
        if !(s.x.is_finite() && s.y.is_finite()) {
            return;
        }
        self.last_raw = Some((s.x, s.y));
        let (x, y) = self.smooth(s.x, s.y);
        let cur = Pt {
            x,
            y,
            pressure: s.pressure.filter(|p| p.is_finite()).map(|p| (p as f64).clamp(0.0, 1.0)),
            tilt: s.tilt.and_then(|(a, b)| {
                (a.is_finite() && b.is_finite())
                    .then(|| ((a as f64).clamp(-1.0, 1.0), (b as f64).clamp(-1.0, 1.0)))
            }),
            t_ms: s.t_ms,
        };
        match self.prev {
            None => {
                // The first sample always stamps: a click must leave a dab.
                let r = Readings { pressure: cur.pressure, tilt: cur.tilt, speed: None, path: 0.0 };
                let size_k = self.emit(cur.x, cur.y, r, out);
                self.until_next = self.step_for(size_k);
            }
            Some(prev) if self.cfg.per_sample => self.sample_dab(prev, cur, out),
            Some(prev) => self.walk(prev, cur, out),
        }
        self.prev = Some(cur);
    }

    /// End the stroke: with smoothing on, the pulled-string point stops short of
    /// the last pointer position, so walk the remaining distance to it (ET-D2;
    /// ET-D1 had no end-of-stroke hook). Appends the dabs that tail completes.
    ///
    /// A no-op when smoothing is off (the string never lags: by control flow, no
    /// arithmetic), when nothing was fed, and when called twice (the second call
    /// finds the string already at the pointer). The tail reuses the last
    /// sample's pressure and tilt and its timestamp, so the `Speed` sensor reads
    /// absent on it (a zero-length time step) rather than inventing a speed.
    pub fn finish(&mut self, out: &mut Vec<Dab>) {
        if self.cfg.smoothing <= 0.0 {
            return;
        }
        let (Some(prev), Some((rx, ry))) = (self.prev, self.last_raw) else {
            return;
        };
        if (prev.x, prev.y) == (rx, ry) {
            return;
        }
        let end = Pt { x: rx, y: ry, ..prev };
        if self.cfg.per_sample {
            self.sample_dab(prev, end, out);
        } else {
            self.walk(prev, end, out);
        }
        self.string = Some((rx, ry));
        self.prev = Some(end);
    }

    /// The `per_sample` counterpart of [`walk`](Self::walk): one source dab at
    /// `cur`, whatever the distance from `prev` (zero included), keeping the
    /// travel direction and path length that `walk` would.
    fn sample_dab(&mut self, prev: Pt, cur: Pt, out: &mut Vec<Dab>) {
        let (dx, dy) = (cur.x - prev.x, cur.y - prev.y);
        let len = dx.hypot(dy);
        if len > 0.0 {
            self.dir = Some((dx / len, dy / len));
        }
        let speed = cur.t_ms.checked_sub(prev.t_ms).filter(|dt| *dt > 0).map(|dt| len / dt as f64);
        self.path += len;
        let r = Readings { pressure: cur.pressure, tilt: cur.tilt, speed, path: self.path };
        self.emit(cur.x, cur.y, r, out);
    }

    /// The pulled-string point for a raw pointer position.
    ///
    /// With `smoothing == 0` this returns the raw position by control flow
    /// (no arithmetic), so smoothing-off strokes are exactly the raw ones.
    fn smooth(&mut self, x: f64, y: f64) -> (f64, f64) {
        let len = self.cfg.smoothing;
        if len <= 0.0 {
            return (x, y);
        }
        let p = match self.string {
            None => (x, y),
            Some((sx, sy)) => {
                let (dx, dy) = (x - sx, y - sy);
                let d = dx.hypot(dy);
                if d > len {
                    let k = (d - len) / d;
                    (sx + dx * k, sy + dy * k)
                } else {
                    (sx, sy)
                }
            }
        };
        self.string = Some(p);
        p
    }

    /// Distance to the next dab after one of size multiplier `size_k`.
    fn step_for(&self, size_k: f64) -> f64 {
        (self.cfg.spacing * self.cfg.base_diameter * size_k).max(MIN_STEP)
    }

    /// Walk the segment `prev -> cur`, emitting a dab each time the owed
    /// distance is paid off, and carry what is left.
    fn walk(&mut self, prev: Pt, cur: Pt, out: &mut Vec<Dab>) {
        let (dx, dy) = (cur.x - prev.x, cur.y - prev.y);
        let len = dx.hypot(dy);
        if len == 0.0 {
            // No movement: nothing owed is paid, and the direction is kept.
            return;
        }
        self.dir = Some((dx / len, dy / len));
        let speed = cur
            .t_ms
            .checked_sub(prev.t_ms)
            .filter(|dt| *dt > 0)
            .map(|dt| len / dt as f64);
        // Sensors interpolate only when both ends have a reading; a missing
        // end leaves the whole segment's reading absent.
        let pressure_ends = prev.pressure.zip(cur.pressure);
        let tilt_ends = prev.tilt.zip(cur.tilt);
        let path0 = self.path;
        let mut travelled = 0.0_f64;
        while self.until_next <= len - travelled {
            travelled += self.until_next;
            let t = travelled / len;
            let r = Readings {
                pressure: pressure_ends.map(|(a, b)| a + (b - a) * t),
                tilt: tilt_ends.map(|((ax, ay), (bx, by))| (ax + (bx - ax) * t, ay + (by - ay) * t)),
                speed,
                path: path0 + travelled,
            };
            let size_k = self.emit(prev.x + dx * t, prev.y + dy * t, r, out);
            self.until_next = self.step_for(size_k);
        }
        let remaining = len - travelled;
        self.until_next -= remaining;
        self.path += len;
    }

    /// The control value in `[0, 1]` for `sensor` at readings `r`; `None`
    /// when the sensor has no reading (see [`Sensor`]). `Off` is `Some(1.0)`.
    fn control(&self, sensor: Sensor, r: &Readings) -> Option<f64> {
        match sensor {
            Sensor::Off => Some(1.0),
            Sensor::Pressure => r.pressure,
            Sensor::TiltElevation => r.tilt.map(|(tx, ty)| 1.0 - tx.hypot(ty).min(1.0)),
            // cells/ms -> cells/s, against the reference.
            Sensor::Speed => r.speed.map(|v| (v * 1000.0 / self.cfg.speed_ref).clamp(0.0, 1.0)),
            Sensor::Fade => Some(1.0 - (r.path / self.cfg.fade_distance).clamp(0.0, 1.0)),
        }
    }

    /// Make one source dab at `(x, y)`: dynamics, then scatter, then
    /// symmetry, in that order. Returns the source's `size_k` (which sets the
    /// next spacing step) and advances the dab counter.
    fn emit(&mut self, x: f64, y: f64, r: Readings, out: &mut Vec<Dab>) -> f64 {
        let cfg = self.cfg;
        let idx = self.dab_index;
        self.dab_index += 1;

        // 1. Dynamics and jitter: all randomness for this dab is drawn here,
        //    once, keyed by the source dab only.
        let size_rnd = stroke_rng(cfg.seed, idx, STREAM_SIZE);
        let flow_rnd = stroke_rng(cfg.seed, idx, STREAM_FLOW);
        let size_k = dynamic_value(cfg.size.min, cfg.size.jitter, self.control(cfg.size.sensor, &r), size_rnd);
        let flow_k = dynamic_value(cfg.flow.min, cfg.flow.jitter, self.control(cfg.flow.sensor, &r), flow_rnd);
        let mut angle = cfg.base_angle;
        if cfg.follow_direction
            && let Some((ux, uy)) = self.dir
        {
            angle += uy.atan2(ux);
        }
        if cfg.angle_jitter > 0.0 {
            let rnd = stroke_rng(cfg.seed, idx, STREAM_ANGLE);
            angle += cfg.angle_jitter * (2.0 * rnd - 1.0) * PI;
        }
        let diameter = cfg.base_diameter * size_k;

        // 2. Scatter. With no travel direction yet (the first dab of a stroke)
        //    the world x-axis stands in for it: a labelled judgement, because
        //    inventing no direction at all would lose the first scatter.
        let (ux, uy) = self.dir.unwrap_or((1.0, 0.0));
        let (px, py) = (-uy, ux);
        let sc = cfg.scatter;
        let copies = if sc.count_jitter > 0.0 {
            let rnd = stroke_rng(cfg.seed, idx, STREAM_SCATTER_COUNT);
            let n = (sc.count as f64 * (1.0 - sc.count_jitter * rnd)).round() as u32;
            n.clamp(1, sc.count)
        } else {
            sc.count
        };
        for j in 0..copies {
            let (mut cx, mut cy) = (x, y);
            if sc.amount > 0.0 {
                let s = STREAM_SCATTER_BASE + 2 * j;
                let off = (2.0 * stroke_rng(cfg.seed, idx, s) - 1.0) * sc.amount * diameter;
                cx += px * off;
                cy += py * off;
                if sc.both_axes {
                    let al = (2.0 * stroke_rng(cfg.seed, idx, s + 1) - 1.0) * sc.amount * diameter;
                    cx += ux * al;
                    cy += uy * al;
                }
            }
            // 3. Symmetry, last: every copy reuses the final size, flow and
            //    angle, reflected; nothing random is drawn below this line.
            for k in 0..self.symmetry_count() {
                let (mx, my, ma) = self.symmetry_copy(k, cx, cy, angle);
                let sk = size_k;
                out.push(Dab { x: mx, y: my, size_k: sk, flow_k, angle: ma });
            }
        }
        size_k
    }

    /// How many dabs [`Symmetry`] makes from one.
    fn symmetry_count(&self) -> u32 {
        match self.cfg.symmetry {
            Symmetry::None => 1,
            Symmetry::Vertical | Symmetry::Horizontal => 2,
            Symmetry::Dual => 4,
            Symmetry::Radial(n) => n,
        }
    }

    /// Copy `k` of a dab at `(x, y)` with `angle`; copy 0 is always the
    /// untouched source (returned by branch, not arithmetic, so it is exact).
    ///
    /// A mirror across a line through the centre at direction angle `phi`
    /// reflects position as `2 (v.u) u - v` and the tip angle as
    /// `2 phi - angle`. The vertical axis at rotation 0 runs along `+y`
    /// (`phi = a + pi/2`); the horizontal along `+x` (`phi = a`).
    fn symmetry_copy(&self, k: u32, x: f64, y: f64, angle: f64) -> (f64, f64, f64) {
        if k == 0 {
            return (x, y, angle);
        }
        let c = self.cfg;
        let a = c.symmetry_angle;
        let mirror = |phi: f64| {
            let (vx, vy) = (x - c.symmetry_cx, y - c.symmetry_cy);
            let (ux, uy) = (phi.cos(), phi.sin());
            let d = 2.0 * (vx * ux + vy * uy);
            (c.symmetry_cx + d * ux - vx, c.symmetry_cy + d * uy - vy, 2.0 * phi - angle)
        };
        match c.symmetry {
            Symmetry::None => (x, y, angle),
            Symmetry::Vertical => mirror(a + PI / 2.0),
            Symmetry::Horizontal => mirror(a),
            Symmetry::Dual => match k {
                1 => mirror(a + PI / 2.0),
                2 => mirror(a),
                _ => (2.0 * c.symmetry_cx - x, 2.0 * c.symmetry_cy - y, angle + PI),
            },
            Symmetry::Radial(n) => {
                let th = TAU * k as f64 / n as f64;
                let (s, co) = th.sin_cos();
                let (vx, vy) = (x - c.symmetry_cx, y - c.symmetry_cy);
                (
                    c.symmetry_cx + vx * co - vy * s,
                    c.symmetry_cy + vx * s + vy * co,
                    angle + th,
                )
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A bare sample: no pressure, no tilt, time = index in ms.
    fn at(x: f64, y: f64, i: u64) -> Sample {
        Sample { x, y, pressure: None, tilt: None, t_ms: i }
    }

    /// Run `samples` through a fresh engine in one chunk.
    fn run(cfg: StrokeConfig, samples: &[Sample]) -> Vec<Dab> {
        let mut e = StrokeEngine::new(cfg).unwrap();
        let mut out = Vec::new();
        e.feed(samples, &mut out);
        out
    }

    /// Bit-level equality of dab lists (so `-0.0 != 0.0` and NaN is visible).
    fn bits(d: &[Dab]) -> Vec<[u64; 5]> {
        d.iter()
            .map(|d| [d.x.to_bits(), d.y.to_bits(), d.size_k.to_bits(), d.flow_k.to_bits(), d.angle.to_bits()])
            .collect()
    }

    /// Protects: the counter RNG's exact output, against values computed by an
    /// independent Python `fmix64` chain (not by this module), so a changed
    /// salt, mixing order or stream key cannot hide behind self-agreement; and
    /// that seed, index and stream each change the draw.
    #[test]
    fn rng_matches_independent_python_literals() {
        assert_eq!(stroke_rng(42, 0, 1), 0.6897749719681896);
        assert_eq!(stroke_rng(42, 0, 2), 0.8999664376090855);
        assert_eq!(stroke_rng(42, 1, 1), 0.013677799243152045);
        assert_eq!(stroke_rng(7, 0, 1), 0.9886330880543397);
        assert_eq!(stroke_rng(42, 1, 16), 0.1322989795564895);
        let base = stroke_rng(42, 1, 1);
        assert_ne!(base, stroke_rng(43, 1, 1));
        assert_ne!(base, stroke_rng(42, 2, 1));
        assert_ne!(base, stroke_rng(42, 1, 2));
        for i in 0..1000 {
            let v = stroke_rng(0, i, 0);
            assert!((0.0..1.0).contains(&v));
        }
    }

    /// Protects: spacing of `s x diameter` (here `0.18 x 10 = 1.8`, the literal
    /// asserted, not recomputed from the constants) between consecutive dab
    /// centres to 1e-9 cells, on three sample densities from far finer than
    /// the step to far coarser — which is what the carried remainder is for.
    /// Mutation: dropping the carry (the `until_next -= remaining` line).
    #[test]
    fn spacing_is_exact_whatever_the_sample_density() {
        for pitch in [0.37_f64, 5.0, 23.0] {
            let n = (400.0 / pitch) as u64;
            let samples: Vec<Sample> =
                (0..=n).map(|i| at(i as f64 * pitch, 0.0, i)).collect();
            let dabs = run(StrokeConfig::new(1, 10.0), &samples);
            assert!(dabs.len() > 100, "pitch {pitch}: only {} dabs", dabs.len());
            assert_eq!(dabs[0].x, 0.0);
            for w in dabs.windows(2) {
                let gap = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
                assert!((gap - 1.8).abs() < 1e-9, "pitch {pitch}: gap {gap}");
            }
        }
    }

    /// Protects: the pre-change contrast. Samples every 3 diameters leave a
    /// 3-diameter gap if each sample stamps once (today's dispatch, computed
    /// here from the input positions alone); the engine leaves none above
    /// `s x diameter`.
    #[test]
    fn engine_closes_the_three_diameter_gaps_one_dab_per_sample_leaves() {
        let d = 10.0;
        let samples: Vec<Sample> = (0..20).map(|i| at(i as f64 * 3.0 * d, 5.0, i)).collect();
        // Pre-change: one dab per sample, so the gap is the sample pitch.
        let pre: f64 = samples.windows(2).map(|w| w[1].x - w[0].x).fold(0.0, f64::max);
        assert_eq!(pre, 30.0, "pre-change contrast: three diameters");
        let dabs = run(StrokeConfig::new(1, d), &samples);
        let post = dabs.windows(2).map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y)).fold(0.0, f64::max);
        assert!(post <= 1.8 + 1e-9, "engine gap {post}");
        // 19 pitches of 30 = 570 cells at 1.8: floor(316.67) + 1 = 317 dabs.
        assert_eq!(dabs.len(), 317);
    }

    /// Protects: float accumulation over a long stroke (`Risks` in ET-D1).
    /// 10 000 irregularly spaced samples on a diagonal; dab `k` must land at
    /// `k x 1.8` along it (closed form, literal step) within 1e-9, and the
    /// count must equal `floor(L / 1.8) + 1` with `L` summed in integers.
    #[test]
    fn ten_thousand_samples_match_the_closed_form() {
        let (ux, uy) = (0.6, 0.8);
        // Pitch 0.5 + 0.1 * (i % 7) cells, summed in tenths of a cell.
        let mut tenths = 0u64;
        let mut samples = vec![at(0.0, 0.0, 0)];
        for i in 1..10_000u64 {
            tenths += 5 + (i % 7);
            let s = tenths as f64 / 10.0;
            samples.push(at(s * ux, s * uy, i));
        }
        let dabs = run(StrokeConfig::new(1, 10.0), &samples);
        let total = tenths as f64 / 10.0;
        assert_eq!(dabs.len(), (total / 1.8).floor() as usize + 1);
        for (k, d) in dabs.iter().enumerate() {
            let s = k as f64 * 1.8;
            assert!((d.x - s * ux).abs() < 1e-9 && (d.y - s * uy).abs() < 1e-9, "dab {k}: ({}, {})", d.x, d.y);
        }
    }

    /// A busy config exercising every stage, for the chunking test.
    fn busy() -> StrokeConfig {
        let mut c = StrokeConfig::new(99, 8.0);
        c.smoothing = 3.0;
        c.size = Dynamic { sensor: Sensor::Pressure, min: 0.3, jitter: 0.4 };
        c.flow = Dynamic { sensor: Sensor::Speed, min: 0.1, jitter: 0.2 };
        c.angle_jitter = 0.5;
        c.follow_direction = true;
        c.scatter = Scatter { count: 4, count_jitter: 0.5, amount: 1.5, both_axes: true };
        c.symmetry = Symmetry::Radial(3);
        c
    }

    /// 200 deterministic curved samples with a mix of present and absent
    /// pressure/tilt and uneven timing.
    fn curve200() -> Vec<Sample> {
        (0..200u64)
            .map(|i| {
                let t = i as f64;
                Sample {
                    x: 20.0 + 0.9 * t + 6.0 * (t * 0.21).sin(),
                    y: 40.0 + 5.0 * (t * 0.13).cos() * (1.0 + t * 0.01),
                    pressure: if i % 9 == 4 { None } else { Some(((i * 37) % 100) as f32 / 100.0) },
                    tilt: if i % 5 == 0 { None } else { Some((0.3, -0.2)) },
                    t_ms: i * 8 + (i % 3),
                }
            })
            .collect()
    }

    /// Protects: streamability. The same 200 samples fed as 1, 7 and 200
    /// chunks (and an uneven split) give byte-equal dabs, with smoothing,
    /// dynamics, jitter, scatter and symmetry all on. Also asserts the list
    /// is non-empty and long enough to mean something.
    /// (Unaffected by swapping RNG stream ids: both runs swap alike.)
    #[test]
    fn chunking_does_not_change_the_dabs() {
        let samples = curve200();
        let whole = run(busy(), &samples);
        assert!(whole.len() > 200, "{} dabs", whole.len());
        for chunk in [7usize, 200, 1, 33] {
            let mut e = StrokeEngine::new(busy()).unwrap();
            let mut out = Vec::new();
            for c in samples.chunks(chunk) {
                e.feed(c, &mut out);
            }
            assert_eq!(bits(&out), bits(&whole), "chunk size {chunk}");
        }
    }

    /// Protects: an absent pressure gives `size_k` and `flow_k` of exactly
    /// 1.0 *by control flow*. The settings are chosen so that every plausible
    /// substitute differs: with `min 0.2`, jitter 0.5, a default control of
    /// 0.5 would give 0.6 x (1 - 0.5 rnd) and a default of 1.0 would still be
    /// jittered below 1.0 for these seeds. Present pressure in the same
    /// config must differ from 1.0, so the sensor is genuinely live.
    #[test]
    fn absent_pressure_is_exactly_one_by_control_flow() {
        let mut c = StrokeConfig::new(42, 10.0);
        c.size = Dynamic { sensor: Sensor::Pressure, min: 0.2, jitter: 0.5 };
        c.flow = Dynamic { sensor: Sensor::Pressure, min: 0.2, jitter: 0.5 };
        let line = |p: Option<f32>| -> Vec<Sample> {
            (0..30u64).map(|i| Sample { x: i as f64 * 4.0, y: 0.0, pressure: p, tilt: None, t_ms: i }).collect()
        };
        let absent = run(c, &line(None));
        assert!(absent.len() > 20);
        for d in &absent {
            assert_eq!(d.size_k.to_bits(), 1.0f64.to_bits());
            assert_eq!(d.flow_k.to_bits(), 1.0f64.to_bits());
        }
        let present = run(c, &line(Some(0.5)));
        assert!(present.iter().all(|d| d.size_k < 0.99 && d.flow_k < 0.99));
        // The sensor off is a different thing: jitter alone still acts.
        c.size.sensor = Sensor::Off;
        let off = run(c, &line(None));
        assert!(off.iter().any(|d| d.size_k < 0.99));
    }

    /// Protects: the dynamics law against hand-computed literals, using an
    /// RNG value from the independent Python chain: `(0.2 + 0.8 x 0.5) x
    /// (1 - 0.5 x 0.6897749719681896)`, and the clamp to `min` when jitter
    /// pushes the value below the floor.
    #[test]
    fn dynamics_law_and_clamp_match_literals() {
        let mut c = StrokeConfig::new(42, 10.0);
        c.size = Dynamic { sensor: Sensor::Pressure, min: 0.2, jitter: 0.5 };
        c.flow = Dynamic { sensor: Sensor::Pressure, min: 0.2, jitter: 0.0 };
        let one = Sample { x: 0.0, y: 0.0, pressure: Some(0.5), tilt: None, t_ms: 0 };
        let d = run(c, &[one])[0];
        assert!((d.size_k - 0.39306750840954312).abs() < 1e-12, "{}", d.size_k);
        assert!((d.flow_k - 0.6).abs() < 1e-12, "{}", d.flow_k);
        // Pressure 0, jitter 1: 0.5 x (1 - 0.9) = 0.05 would be < min 0.5.
        c.size = Dynamic { sensor: Sensor::Pressure, min: 0.5, jitter: 1.0 };
        let zero = Sample { pressure: Some(0.0), ..one };
        let d = run(c, &[zero])[0];
        assert_eq!(d.size_k, 0.5);
    }

    /// Protects: scatter's offset law and RNG keying: for seed 42 the second
    /// dab (index 1, at x = 1.8, travel along +x) is offset perpendicular by
    /// `(2 x 0.1322989795564895 - 1) x 1.0 x 10 = -7.35402040887021` cells,
    /// the draw taken from the independent Python chain at stream 16. Mutation:
    /// changing the scatter stream base.
    #[test]
    fn scatter_offset_matches_pinned_draw() {
        let mut c = StrokeConfig::new(42, 10.0);
        c.scatter = Scatter { count: 1, count_jitter: 0.0, amount: 1.0, both_axes: false };
        let dabs = run(c, &[at(0.0, 0.0, 0), at(100.0, 0.0, 1)]);
        assert!((dabs[1].x - 1.8).abs() < 1e-9);
        assert!((dabs[1].y - -7.35402040887021).abs() < 1e-9, "{}", dabs[1].y);
        // Dab 0 had no direction: world x stands in, so the offset is in y.
        assert_eq!(dabs[0].x, 0.0);
        assert!(dabs[0].y.abs() <= 10.0 && dabs[0].y != 0.0);
    }

    /// Protects: scatter count and the 16 cap are enforced, and count jitter
    /// never goes below one copy.
    #[test]
    fn scatter_counts_are_bounded() {
        let mut c = StrokeConfig::new(5, 10.0);
        c.scatter = Scatter { count: 16, count_jitter: 0.0, amount: 0.0, both_axes: false };
        assert_eq!(run(c, &[at(0.0, 0.0, 0)]).len(), 16);
        c.scatter.count = 17;
        assert_eq!(StrokeEngine::new(c).unwrap_err(), StrokeError::Invalid("scatter"));
        c.scatter = Scatter { count: 3, count_jitter: 1.0, amount: 0.0, both_axes: false };
        for seed in 0..50 {
            c.seed = seed;
            let n = run(c, &[at(0.0, 0.0, 0)]).len();
            assert!((1..=3).contains(&n));
        }
    }

    /// Reflect point `p` across the line through `c` with direction angle
    /// `phi`, written independently of the engine (vector form, no angle
    /// doubling).
    fn reflect(p: (f64, f64), c: (f64, f64), phi: f64) -> (f64, f64) {
        let u = (phi.cos(), phi.sin());
        let v = (p.0 - c.0, p.1 - c.1);
        let dot = v.0 * u.0 + v.1 * u.1;
        (c.0 + 2.0 * dot * u.0 - v.0, c.1 + 2.0 * dot * u.1 - v.1)
    }

    /// Difference of two angles wrapped to `[-pi, pi]`.
    fn ang_diff(a: f64, b: f64) -> f64 {
        (a - b + PI).rem_euclid(TAU) - PI
    }

    /// Protects: symmetry makes the exact mirror (1e-12) of every source dab
    /// for each mode, with size jitter, flow jitter, angle jitter and scatter
    /// all on, so a mirrored copy that drew its own jitter would differ in
    /// `size_k`/`flow_k`/`angle` and fail. N-fold radial yields exactly N
    /// copies. Mutation: jitter drawn after the mirror, per copy.
    #[test]
    fn symmetry_is_the_exact_mirror_of_each_source() {
        let (cx, cy, a) = (30.0, -12.0, 0.4);
        let modes: [(Symmetry, usize); 5] = [
            (Symmetry::None, 1),
            (Symmetry::Vertical, 2),
            (Symmetry::Horizontal, 2),
            (Symmetry::Dual, 4),
            (Symmetry::Radial(5), 5),
        ];
        for (mode, n) in modes {
            let mut c = StrokeConfig::new(7, 6.0);
            c.size = Dynamic { sensor: Sensor::Off, min: 0.1, jitter: 0.8 };
            c.flow = Dynamic { sensor: Sensor::Off, min: 0.1, jitter: 0.8 };
            c.angle_jitter = 0.9;
            c.scatter = Scatter { count: 3, count_jitter: 0.0, amount: 0.7, both_axes: true };
            c.symmetry = mode;
            c.symmetry_cx = cx;
            c.symmetry_cy = cy;
            c.symmetry_angle = a;
            let samples: Vec<Sample> =
                (0..40u64).map(|i| at(i as f64 * 2.5, 3.0 + (i as f64 * 0.3).sin() * 4.0, i)).collect();
            let dabs = run(c, &samples);
            assert_eq!(dabs.len() % n, 0, "{mode:?}");
            assert!(dabs.len() >= 3 * n * 10, "{mode:?}: {} dabs", dabs.len());
            for g in dabs.chunks(n) {
                let s = g[0];
                for (k, d) in g.iter().enumerate() {
                    // size and flow are the source's, bit for bit.
                    assert_eq!(d.size_k.to_bits(), s.size_k.to_bits(), "{mode:?} copy {k}");
                    assert_eq!(d.flow_k.to_bits(), s.flow_k.to_bits(), "{mode:?} copy {k}");
                    let (ex, ey, ea) = match (mode, k) {
                        (_, 0) => (s.x, s.y, s.angle),
                        (Symmetry::Vertical, 1) | (Symmetry::Dual, 1) => {
                            let phi = a + PI / 2.0;
                            let p = reflect((s.x, s.y), (cx, cy), phi);
                            (p.0, p.1, 2.0 * phi - s.angle)
                        }
                        (Symmetry::Horizontal, 1) | (Symmetry::Dual, 2) => {
                            let p = reflect((s.x, s.y), (cx, cy), a);
                            (p.0, p.1, 2.0 * a - s.angle)
                        }
                        (Symmetry::Dual, 3) => (2.0 * cx - s.x, 2.0 * cy - s.y, s.angle + PI),
                        (Symmetry::Radial(m), k) => {
                            let th = TAU * k as f64 / m as f64;
                            let (vx, vy) = (s.x - cx, s.y - cy);
                            (
                                cx + vx * th.cos() - vy * th.sin(),
                                cy + vx * th.sin() + vy * th.cos(),
                                s.angle + th,
                            )
                        }
                        other => panic!("unexpected {other:?}"),
                    };
                    assert!((d.x - ex).abs() < 1e-12 && (d.y - ey).abs() < 1e-12, "{mode:?} copy {k}");
                    assert!(ang_diff(d.angle, ea).abs() < 1e-12, "{mode:?} copy {k} angle");
                }
            }
        }
    }

    /// Protects: pulled-string smoothing. A string of 5 cells behind a
    /// pointer that jumps from 0 to 10 leaves the stroke point at exactly 5;
    /// with smoothing off (the default) the same input reaches 10. The
    /// string never overshoots a pointer inside its length.
    #[test]
    fn smoothing_lags_by_the_string_length_and_is_off_by_default() {
        let s = [at(0.0, 0.0, 0), at(10.0, 0.0, 1)];
        let off = run(StrokeConfig::new(1, 10.0), &s);
        assert_eq!(off.last().unwrap().x, 10.0 - (10.0 % 1.8));
        let mut c = StrokeConfig::new(1, 10.0);
        c.smoothing = 5.0;
        let on = run(c, &s);
        // Stroke point ends at 5.0: dabs at 0, 1.8, 3.6 (next would be 5.4).
        assert!((on.last().unwrap().x - 3.6).abs() < 1e-9, "{}", on.last().unwrap().x);
        let near = run(c, &[at(0.0, 0.0, 0), at(3.0, 0.0, 1)]);
        assert_eq!(near.len(), 1, "a pointer inside the string does not move it");
    }

    /// Protects: speed from timestamps (absent for the first dab, and for a
    /// stalled clock) and the fade sensor. Speed absent leaves the dynamic
    /// idle (1.0); a slow stroke gives a low control, a fast one a high one;
    /// fade runs 1 -> 0 along the path.
    #[test]
    fn speed_and_fade_sensors() {
        let mut c = StrokeConfig::new(1, 10.0);
        c.size = Dynamic { sensor: Sensor::Speed, min: 0.0, jitter: 0.0 };
        // 4 cells per ms = 4000 cells/s against ref 1000 -> control 1.
        let fast: Vec<Sample> = (0..10u64).map(|i| at(i as f64 * 4.0, 0.0, i)).collect();
        assert!(run(c, &fast)[1..].iter().all(|d| (d.size_k - 1.0).abs() < 1e-12));
        // 0.1 cell per ms = 100 cells/s -> control 0.1.
        let slow: Vec<Sample> = (0..100u64).map(|i| at(i as f64 * 0.5, 0.0, i * 5)).collect();
        assert!(run(c, &slow)[1..].iter().all(|d| (d.size_k - 0.1).abs() < 1e-12));
        // The very first dab has no speed: idle, exactly 1.0.
        assert_eq!(run(c, &slow)[0].size_k, 1.0);
        // Stalled clock (same t_ms): speed absent, idle.
        let stalled: Vec<Sample> = (0..10u64).map(|i| at(i as f64 * 4.0, 0.0, 7)).collect();
        assert!(run(c, &stalled).iter().all(|d| d.size_k == 1.0));
        // Fade: 100 cells to zero; dab at path 50 is 0.5.
        c.size = Dynamic { sensor: Sensor::Fade, min: 0.0, jitter: 0.0 };
        let line = [at(0.0, 0.0, 0), at(200.0, 0.0, 1)];
        let f = run(c, &line);
        assert_eq!(f[0].size_k, 1.0);
        let half = f.iter().find(|d| d.x > 50.0 - 0.1 * 10.0).unwrap();
        assert!(half.size_k < 0.6 && half.size_k > 0.0);
        assert_eq!(f.last().unwrap().size_k, 0.0);
    }

    /// Protects: rejection of nonsense configs and tolerance of nonsense
    /// samples (non-finite positions ignored, non-finite pressure absent).
    #[test]
    fn invalid_config_is_rejected_and_bad_samples_are_ignored() {
        assert!(StrokeEngine::new(StrokeConfig::new(1, 0.0)).is_err());
        assert!(StrokeEngine::new(StrokeConfig::new(1, f64::NAN)).is_err());
        let mut c = StrokeConfig::new(1, 5.0);
        c.spacing = 0.0;
        assert_eq!(StrokeEngine::new(c).unwrap_err(), StrokeError::Invalid("spacing"));
        c.spacing = 0.2;
        c.symmetry = Symmetry::Radial(0);
        assert_eq!(StrokeEngine::new(c).unwrap_err(), StrokeError::Invalid("symmetry"));
        let mut c = StrokeConfig::new(1, 5.0);
        c.size = Dynamic { sensor: Sensor::Pressure, min: 0.5, jitter: 0.0 };
        let nan = Sample { x: 1.0, y: 1.0, pressure: Some(f32::NAN), tilt: None, t_ms: 0 };
        let d = run(c, &[at(f64::NAN, 0.0, 0), nan]);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].size_k, 1.0, "non-finite pressure is absent");
    }

    /// Protects: a zero `min` with zero pressure (size 0) cannot hang the
    /// walk: the step floor guarantees progress.
    #[test]
    fn zero_size_still_advances() {
        let mut c = StrokeConfig::new(1, 10.0);
        c.size = Dynamic { sensor: Sensor::Pressure, min: 0.0, jitter: 0.0 };
        let s: Vec<Sample> = (0..3u64)
            .map(|i| Sample { x: i as f64, y: 0.0, pressure: Some(0.0), tilt: None, t_ms: i })
            .collect();
        let d = run(c, &s);
        assert!(d.len() >= 1000 && d.len() <= 2002, "{}", d.len());
    }

    /// Protects: `finish` (ET-D2) walking the smoothing tail to the pointer. A string of 5 behind a pointer that jumped 0 to 10 stops at 5 (pinned by the test above); after `finish` the last dab lies on the stroke line past 5 and no further than 10, and the dab spacing is still `spacing x diameter` (1.8) across the join. A second `finish`, a smoothing-off `finish` and a `finish` on an empty engine add nothing.
    #[test]
    fn finish_flushes_the_smoothing_tail_once_and_only_with_smoothing() {
        let s = [at(0.0, 0.0, 0), at(10.0, 0.0, 1)];
        let mut c = StrokeConfig::new(1, 10.0);
        c.smoothing = 5.0;
        let mut e = StrokeEngine::new(c).unwrap();
        let mut out = Vec::new();
        e.feed(&s, &mut out);
        let before = out.len();
        assert!(out.last().unwrap().x <= 5.0, "premise: the string stops at 5");
        e.finish(&mut out);
        assert!(out.len() > before, "the tail adds dabs");
        let last = out.last().unwrap();
        assert!(last.x > 5.0 && last.x <= 10.0 && last.y == 0.0, "last dab at x = {}", last.x);
        // 1.8 spacing is carried across the join: dabs at 0, 1.8, 3.6, ... so the
        // last dab at or before 10 is 9.0.
        assert!((last.x - 9.0).abs() < 1e-9, "x = {}", last.x);
        let n = out.len();
        e.finish(&mut out);
        assert_eq!(out.len(), n, "a second finish is a no-op");

        let mut off = StrokeEngine::new(StrokeConfig::new(1, 10.0)).unwrap();
        let mut o = Vec::new();
        off.feed(&s, &mut o);
        let m = o.len();
        off.finish(&mut o);
        assert_eq!(o.len(), m, "smoothing off: nothing to flush");
        let mut empty = StrokeEngine::new(c).unwrap();
        empty.finish(&mut o);
        assert_eq!(o.len(), m);
    }

    /// Protects: `per_sample` (ET-D2): one dab per accepted sample at its own position, spacing ignored, repeated positions included, the first sample included; and exactly `1.0` size for an absent pressure.
    #[test]
    fn per_sample_emits_one_dab_per_sample_and_ignores_spacing() {
        let mut c = StrokeConfig::new(1, 10.0);
        c.per_sample = true;
        let s = [at(0.0, 0.0, 0), at(30.0, 0.0, 1), at(30.0, 0.0, 2), at(31.0, 4.0, 3)];
        let d = run(c, &s);
        let xs: Vec<(f64, f64)> = d.iter().map(|d| (d.x, d.y)).collect();
        assert_eq!(xs, vec![(0.0, 0.0), (30.0, 0.0), (30.0, 0.0), (31.0, 4.0)]);
        assert!(d.iter().all(|d| d.size_k == 1.0), "absent pressure: exactly the base size");
    }
}
