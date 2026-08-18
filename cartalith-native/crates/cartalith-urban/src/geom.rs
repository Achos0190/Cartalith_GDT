//! The vector / polygon geometry kernel — reference lines 28286-28362
//! (`V`, `polyArea` … `clipConvex`) plus `convexHull` (line 29639).
//!
//! Every function here is reachable from the reference's own `UME._test`
//! export, which is what makes this milestone golden-verifiable rather than
//! hand-checked. Ported literally: the operation order inside each expression
//! is the reference's, because these feed `insetPoly`'s area/self-intersection
//! guards and `buildBlocks`' accept/reject thresholds, where a last-bit
//! difference flips a block from kept to dropped.
//!
//! Two behaviours here look like bugs and are not being "fixed" — they are what
//! the reference does, they are pinned by golden tests, and downstream code is
//! tuned around them:
//!
//! - [`clip_convex`] intersects the subject edge against the clip *segment*,
//!   not the clip *line*. A subject vertex outside the clip polygon's extent
//!   therefore produces no crossing point at all, so clipping a shape that
//!   pokes past the clip window's corners can collapse to empty rather than
//!   returning the true intersection. `buildParcels`/`buildBuildings` only ever
//!   clip shapes that already sit inside their block, so it never bites there.
//! - [`inset_poly`] returns `None` — not a degenerate polygon — when the result
//!   has area below 15, or when it self-intersects and has at most 60 vertices.
//!   Those are the reference's own two rejection gates and `buildBlocks` reads
//!   the `None` as "this block cannot be built on".

/// `Math.hypot(x, y)` as V8 actually computes it — **not** `f64::hypot`.
///
/// This is a real, measured discrepancy, not a theoretical one. ECMA-262
/// specifies `Math.hypot` only as "implementation-approximated", and V8
/// implements it by scaling to the largest magnitude and Kahan-summing the
/// squared ratios: `max * sqrt(Σ (vᵢ/max)²)`. Rust's `f64::hypot` is a
/// different, more accurate algorithm. On inputs as ordinary as `(3, 3)` they
/// disagree by one ulp:
///
/// | | value |
/// |---|---|
/// | true 3√2 | 4.242640687119285146… |
/// | Rust `f64::hypot(3,3)` | 4.2426406871192847703 (correctly rounded) |
/// | V8 `Math.hypot(3,3)` | 4.2426406871192856585 (1 ulp high) |
///
/// It surfaced immediately: the first golden run of `dist_pt_seg` failed on
/// exactly that case. Every distance in this engine — `V.len`, `V.dist`,
/// `nearestNode`'s radius test, `attachPoint`'s 11 m snap, `rawEdge`'s 3.5 m
/// minimum — flows through it, and those are threshold comparisons where being
/// *more* accurate than the reference is the wrong answer
/// (`cartalith-rust-conventions`: match the JS engine, do not improve on it).
/// So V8's algorithm is reproduced here rather than delegated.
///
/// The infinity/NaN ordering is the spec's (`Math.hypot(∞, NaN)` is `∞`, not
/// `NaN`), which the scaling loop on its own would get wrong.
pub fn js_hypot(x: f64, y: f64) -> f64 {
    if x.is_infinite() || y.is_infinite() {
        return f64::INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    let (ax, ay) = (x.abs(), y.abs());
    let max = if ay > ax { ay } else { ax };
    if max == 0.0 {
        return 0.0;
    }
    // Kahan-compensated Σ (vᵢ/max)², in V8's own argument order.
    let mut sum = 0.0f64;
    let mut compensation = 0.0f64;
    for v in [ax, ay] {
        let n = v / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

/// `Math.min(a, b)`, with JS semantics rather than Rust's.
///
/// The difference that matters: **JS propagates NaN, Rust absorbs it.**
/// `Math.min(0.70, NaN)` is `NaN`; `f64::min(0.70, NaN)` is `0.70`.
///
/// **One documented divergence, on signed zero.** `Math.min(+0, -0)` is `-0`
/// and `Math.max(+0, -0)` is `+0`; this returns whichever argument the `<`
/// comparison happens to land on, since `-0.0 < 0.0` is false. Only two of
/// `applyWildness`/`applyPlotChaos`'s eleven clamps have a zero bound
/// (`pierceChance` and `deadEndBias`, both `lo = 0`), and neither can reach a
/// `-0` argument: `0.10 * (2 - w)` is `-0` only if `2 - w` is `-0`, which
/// subtraction of two finite doubles never produces, and
/// `deadEndBias + (w - 1) * 0.15` is `+0` at `w == 1`. So the divergence is
/// unreachable, and handling it would be four lines of code for a case no
/// caller can construct. Recorded, not coded around.
///
/// Milestone 5 gave these a second set of call sites (`buildSite`'s raster
/// index clamps and channel-drift bounds, `terrainSuitability`'s flood score),
/// which is why they live here beside [`js_hypot`] rather than inside
/// `rules` where milestone 4 first needed them.
pub fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if b < a {
        b
    } else {
        a
    }
}

/// `Math.max(a, b)`, with JS semantics. See [`js_min`] for the NaN rule and the
/// signed-zero divergence.
pub fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if b > a {
        b
    } else {
        a
    }
}

/// `Math.exp(x)`, with **V8's** result rather than the platform libm's.
///
/// The same finding as [`js_hypot`], arriving at milestone 5 and measured the
/// same way. Every height, every slope falloff and every log-normal draw in
/// this subsystem runs through `Math.exp`, and Rust's `f64::exp` delegates to
/// the platform's libm, which is **not** the function V8 calls:
///
/// | | disagreements with V8 |
/// |---|---|
/// | `f64::exp` (MSVC CRT) | **20,721 of 240,000** random arguments, all by one ulp |
/// | this function | **0 of 240,000** |
///
/// V8 calls `base::ieee754::exp`, which is FDLIBM's `__ieee754_exp` — argument
/// reduction to `[-0.5 ln2, 0.5 ln2]`, a degree-5 polynomial for the
/// `x - x*c` correction, and a `2^k` scale — transliterated below with its
/// integer bit twiddling intact. It is *less* accurate than a good modern libm
/// (it promises under one ulp, not correct rounding), and that is exactly the
/// point: `cartalith-rust-conventions`' rule is to match the JS engine, not to
/// improve on it. Milestone 5's very first golden run failed on a one-ulp
/// `exp`, the same way milestone 1's first `dist_pt_seg` run failed on a
/// one-ulp `hypot`.
///
/// **One measured special case.** Across 244,000 arguments — 240,000 random,
/// plus every half- and quarter-integer to +-20, plus `1.0` at +-1 and +-2 ulp
/// — V8 and FDLIBM agree everywhere except at exactly `x == 1.0`, where V8
/// returns the correctly-rounded `e` and FDLIBM returns one ulp above it.
/// Reproduced here because it was measured, not because its cause is known. It
/// is unreachable from the site model, whose `exp` arguments are all
/// `-(d^2)/(2*sigma^2)` and therefore never positive.
// The constants below are FDLIBM's own source text, quoted digit for digit so
// this can be diffed against `e_exp.c` by eye. Clippy would have them shortened
// to the same doubles written differently, and `INVLN2` replaced by
// `f64::consts::LOG2_E` -- both are the same value and both would destroy that
// property, which is the only defence this function has against a silent edit.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
pub fn js_exp(x: f64) -> f64 {
    const O_THRESHOLD: f64 = 7.09782712893383973096e+02;
    const U_THRESHOLD: f64 = -7.45133219101941108420e+02;
    const LN2HI: [f64; 2] = [6.93147180369123816490e-01, -6.93147180369123816490e-01];
    const LN2LO: [f64; 2] = [1.90821492927058770002e-10, -1.90821492927058770002e-10];
    const INVLN2: f64 = 1.44269504088896338700e+00;
    const HALF: [f64; 2] = [0.5, -0.5];
    const P1: f64 = 1.66666666666666019037e-01;
    const P2: f64 = -2.77777777770155933842e-03;
    const P3: f64 = 6.61375632143793436117e-05;
    const P4: f64 = -1.65339022054652515390e-06;
    const P5: f64 = 4.13813679705723846039e-08;
    const TWOM1000: f64 = 9.33263618503218878990e-302;
    const HUGE: f64 = 1.0e300;

    // The measured divergence above. See the doc comment.
    if x == 1.0 {
        return std::f64::consts::E;
    }

    let bits = x.to_bits();
    let hx0 = (bits >> 32) as u32;
    let xsb = ((hx0 >> 31) & 1) as usize; // sign bit of x
    let hx = hx0 & 0x7fff_ffff; // high word of |x|

    let mut k: i32 = 0;
    let mut hi = 0.0f64;
    let mut lo = 0.0f64;
    let mut x = x;

    // filter out non-finite argument
    if hx >= 0x4086_2e42 {
        // |x| >= 709.78...
        if hx >= 0x7ff0_0000 {
            let lx = bits as u32;
            if ((hx & 0x000f_ffff) | lx) != 0 {
                return x + x; // NaN
            }
            return if xsb == 0 { x } else { 0.0 }; // exp(+-inf) = {inf, 0}
        }
        if x > O_THRESHOLD {
            return HUGE * HUGE; // overflow
        }
        if x < U_THRESHOLD {
            return TWOM1000 * TWOM1000; // underflow
        }
    }

    // argument reduction
    if hx > 0x3fd6_2e42 {
        // |x| > 0.5 ln2
        if hx < 0x3ff0_a2b2 {
            // ... and |x| < 1.5 ln2
            hi = x - LN2HI[xsb];
            lo = LN2LO[xsb];
            k = 1 - xsb as i32 - xsb as i32;
        } else {
            k = (INVLN2 * x + HALF[xsb]) as i32;
            let t = k as f64;
            hi = x - t * LN2HI[0]; // t*ln2HI is exact here
            lo = t * LN2LO[0];
        }
        x = hi - lo;
    } else if hx < 0x3e30_0000 {
        // |x| < 2^-28
        if HUGE + x > 1.0 {
            return 1.0 + x; // trigger inexact
        }
    } else {
        k = 0;
    }

    // x is now in the primary range
    let t = x * x;
    let twopk = if k >= -1021 {
        f64::from_bits(((0x3ff + k) as u64) << 52)
    } else {
        f64::from_bits(((0x3ff + k + 1000) as u64) << 52)
    };
    let c = x - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    if k == 0 {
        return 1.0 - ((x * c) / (c - 2.0) - x);
    }
    let y = 1.0 - ((lo - (x * c) / (2.0 - c)) - hi);
    if k >= -1021 {
        if k == 1024 {
            // the one k whose 2^k word would overflow to infinity; scale twice
            return y * 2.0 * f64::from_bits(0x7fe0_0000_0000_0000);
        }
        y * twopk
    } else {
        y * twopk * TWOM1000
    }
}

/// `V` (reference line 28286): a plain 2-D point/vector. `f64` throughout —
/// these are JS `Number`s and the engine's thresholds are tuned against their
/// exact values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

/// Convenience constructor; `Vec2::new(x, y)` reads better than a struct
/// literal at the density these algorithms are written at.
impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    /// `V.len` — `Math.hypot`, via [`js_hypot`].
    pub fn len(self) -> f64 {
        js_hypot(self.x, self.y)
    }
    /// `V.dist` — `Math.hypot`, via [`js_hypot`].
    pub fn dist(self, b: Vec2) -> f64 {
        js_hypot(self.x - b.x, self.y - b.y)
    }
    /// `V.norm` — divides by `hypot||1`, so the zero vector maps to itself
    /// rather than to NaN. Kept exactly, since several call sites rely on it.
    /// (JS `||` treats NaN as falsy too, so a NaN length also becomes 1.)
    pub fn norm(self) -> Vec2 {
        let h = js_hypot(self.x, self.y);
        let l = if h == 0.0 || h.is_nan() { 1.0 } else { h };
        Vec2::new(self.x / l, self.y / l)
    }
    /// `V.dot`
    pub fn dot(self, b: Vec2) -> f64 {
        self.x * b.x + self.y * b.y
    }
    /// `V.cross` — the 2-D scalar cross product.
    pub fn cross(self, b: Vec2) -> f64 {
        self.x * b.y - self.y * b.x
    }
    /// `V.lerp`
    pub fn lerp(self, b: Vec2, t: f64) -> Vec2 {
        Vec2::new(self.x + (b.x - self.x) * t, self.y + (b.y - self.y) * t)
    }
    /// `V.rot90`
    pub fn rot90(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
}

/// `V.add` — as the operator, so the dense ported geometry below reads as
/// arithmetic rather than as method chains.
impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, b: Vec2) -> Vec2 {
        Vec2::new(self.x + b.x, self.y + b.y)
    }
}
/// `V.sub`
impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, b: Vec2) -> Vec2 {
        Vec2::new(self.x - b.x, self.y - b.y)
    }
}
/// `V.mul` — vector times scalar.
impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}

/// `polyArea` (line 28290) — signed shoelace area, positive for CCW.
pub fn poly_area(p: &[Vec2]) -> f64 {
    let n = p.len();
    let mut s = 0.0;
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        s += a.x * b.y - b.x * a.y;
    }
    s / 2.0
}

/// `polyCentroid` (line 28291) — area-weighted centroid, falling back to the
/// vertex mean when the signed area is degenerate (`|2A| < 1e-9`).
pub fn poly_centroid(p: &[Vec2]) -> Vec2 {
    let n = p.len();
    let (mut sx, mut sy, mut sa) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        let c = a.x * b.y - b.x * a.y;
        sa += c;
        sx += (a.x + b.x) * c;
        sy += (a.y + b.y) * c;
    }
    if sa.abs() < 1e-9 {
        let (mut mx, mut my) = (0.0, 0.0);
        for q in p {
            mx += q.x;
            my += q.y;
        }
        return Vec2::new(mx / n as f64, my / n as f64);
    }
    Vec2::new(sx / (3.0 * sa), sy / (3.0 * sa))
}

/// `pointInPoly` (line 28295) — the standard crossing-number test, with the
/// reference's exact half-open edge convention (`(yi>pt.y) !== (yj>pt.y)`).
pub fn point_in_poly(pt: Vec2, p: &[Vec2]) -> bool {
    let mut inside = false;
    let n = p.len();
    if n == 0 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (p[i].x, p[i].y);
        let (xj, yj) = (p[j].x, p[j].y);
        if ((yi > pt.y) != (yj > pt.y)) && (pt.x < (xj - xi) * (pt.y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// What [`seg_int`] returns: the two parameters and the crossing point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegHit {
    pub t: f64,
    pub u: f64,
    pub pt: Vec2,
}

/// `segInt` (line 28298) — **segment**-segment intersection, `None` when the
/// denominator is under `1e-12` (parallel/collinear, including the overlapping
/// collinear case) or when either parameter falls outside `[-1e-9, 1+1e-9]`.
pub fn seg_int(p: Vec2, p2: Vec2, q: Vec2, q2: Vec2) -> Option<SegHit> {
    let r = Vec2::new(p2.x - p.x, p2.y - p.y);
    let s = Vec2::new(q2.x - q.x, q2.y - q.y);
    let den = r.cross(s);
    if den.abs() < 1e-12 {
        return None;
    }
    let qp = Vec2::new(q.x - p.x, q.y - p.y);
    let t = qp.cross(s) / den;
    let u = qp.cross(r) / den;
    if !(-1e-9..=1.0 + 1e-9).contains(&t) || !(-1e-9..=1.0 + 1e-9).contains(&u) {
        return None;
    }
    Some(SegHit { t, u, pt: Vec2::new(p.x + t * r.x, p.y + t * r.y) })
}

/// `distPtSeg` (line 28305) — point-to-segment distance, degenerate segments
/// falling back to point-to-point.
pub fn dist_pt_seg(pt: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let l2 = ab.x * ab.x + ab.y * ab.y;
    if l2 < 1e-12 {
        return pt.dist(a);
    }
    let mut t = ((pt.x - a.x) * ab.x + (pt.y - a.y) * ab.y) / l2;
    t = t.clamp(0.0, 1.0);
    pt.dist(Vec2::new(a.x + ab.x * t, a.y + ab.y * t))
}

/// `polySelfIntersects` (line 28309) — O(n²) all-pairs edge test skipping
/// adjacent pairs and the wrap-around pair. Quadratic on purpose: its only
/// caller ([`inset_poly`]) gates it behind a 60-vertex cap.
pub fn poly_self_intersects(p: &[Vec2]) -> bool {
    let n = p.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if i.abs_diff(j) <= 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if seg_int(p[i], p[(i + 1) % n], p[j], p[(j + 1) % n]).is_some() {
                return true;
            }
        }
    }
    false
}

/// `chaikin` (line 28314) — one corner-cutting subdivision pass at the ¼ / ¾
/// parameters. Open polylines keep their endpoints; closed ones wrap.
pub fn chaikin(pts: &[Vec2], closed: bool) -> Vec<Vec2> {
    let n = pts.len();
    if n == 0 {
        return Vec::new();
    }
    let lim = if closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(2 * lim + 2);
    if !closed {
        out.push(pts[0]);
    }
    for i in 0..lim {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        out.push(a.lerp(b, 0.25));
        out.push(a.lerp(b, 0.75));
    }
    if !closed {
        out.push(pts[n - 1]);
    }
    out
}

/// `simplify` (line 28321) — iterative Douglas-Peucker. Fewer than three
/// points passes through unchanged.
pub fn simplify(pts: &[Vec2], tol: f64) -> Vec<Vec2> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![0u8; pts.len()];
    keep[0] = 1;
    let last = pts.len() - 1;
    keep[last] = 1;
    let mut stack = vec![(0usize, last)];
    while let Some((a, b)) = stack.pop() {
        let mut mx = -1.0f64;
        let mut mi = usize::MAX;
        for i in (a + 1)..b {
            let d = dist_pt_seg(pts[i], pts[a], pts[b]);
            if d > mx {
                mx = d;
                mi = i;
            }
        }
        if mx > tol {
            keep[mi] = 1;
            stack.push((a, mi));
            stack.push((mi, b));
        }
    }
    pts.iter().enumerate().filter(|(i, _)| keep[*i] == 1).map(|(_, p)| *p).collect()
}

/// `ensureCCW` (line 28330) — reverse if the shoelace area is negative, so the
/// interior lies to the LEFT of every directed edge.
pub fn ensure_ccw(p: &[Vec2]) -> Vec<Vec2> {
    if poly_area(p) < 0.0 {
        let mut v = p.to_vec();
        v.reverse();
        v
    } else {
        p.to_vec()
    }
}

/// `insetPoly` (line 28332) — per-edge inward offset with miter joins.
/// `dists[i]` applies to edge `i -> i+1`, and a short `dists` clamps to its
/// last entry. CCW input expected.
///
/// Returns `None` on the reference's own three rejections: fewer than three
/// vertices, a result whose signed area is below 15, or (at 60 vertices or
/// fewer) a result that self-intersects.
pub fn inset_poly(poly: &[Vec2], dists: &[f64]) -> Option<Vec<Vec2>> {
    let n = poly.len();
    if n < 3 {
        return None;
    }
    // `Math.max(...dists)` — used twice below as the miter-runaway clamp.
    let dmax = dists.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut lines: Vec<(Vec2, Vec2)> = Vec::with_capacity(n); // (point, direction)
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let d = (b - a).norm();
        let nl = Vec2::new(-d.y, d.x); // inward (left) normal for CCW
        let off = dists[i.min(dists.len() - 1)];
        lines.push((a + nl * off, d));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let l1 = lines[(i + n - 1) % n];
        let l2 = lines[i];
        let den = l1.1.cross(l2.1);
        let pt = if den.abs() < 1e-9 {
            l2.0
        } else {
            let t = (l2.0 - l1.0).cross(l2.1) / den;
            let mut pt = l1.0 + l1.1 * t;
            if pt.dist(poly[i]) > 4.0 * dmax + 6.0 {
                pt = poly[i] + (pt - poly[i]).norm() * (dmax * 1.8);
            }
            pt
        };
        out.push(pt);
    }
    if poly_area(&out) < 15.0 {
        return None;
    }
    if out.len() <= 60 && poly_self_intersects(&out) {
        return None;
    }
    Some(out)
}

/// `clipConvex` (line 28351) — Sutherland-Hodgman against a convex CCW clip
/// polygon. See this module's header for the segment-vs-line caveat, which is
/// reference behaviour and is pinned by a golden test.
pub fn clip_convex(subject: &[Vec2], clip: &[Vec2]) -> Vec<Vec2> {
    let mut out = subject.to_vec();
    let m = clip.len();
    let mut i = 0;
    while i < m && !out.is_empty() {
        let a = clip[i];
        let b = clip[(i + 1) % m];
        let inp = std::mem::take(&mut out);
        let inside = |p: Vec2| (b - a).cross(p - a) >= -1e-9;
        let ln = inp.len();
        for j in 0..ln {
            let cur = inp[j];
            let prv = inp[(j + ln - 1) % ln];
            let (ci, pi) = (inside(cur), inside(prv));
            if ci {
                if !pi && let Some(h) = seg_int(prv, cur, a, b) {
                    out.push(h.pt);
                }
                out.push(cur);
            } else if pi && let Some(h) = seg_int(prv, cur, a, b) {
                out.push(h.pt);
            }
        }
        i += 1;
    }
    out
}

/// `convexHull` (line 29639) — monotone chain. Collinear points are dropped
/// (`<= 0` on the cross product), and the sort is by x then y, stable, exactly
/// as JS's own stable `Array.prototype.sort` with that comparator.
pub fn convex_hull(pts: &[Vec2]) -> Vec<Vec2> {
    let mut p = pts.to_vec();
    // `(a,b)=>a.x-b.x||a.y-b.y`: JS treats a NaN comparator result as 0, i.e.
    // "keep the existing order" under a stable sort — `unwrap_or(Equal)` on a
    // stable `sort_by` is exactly that, not a papered-over NaN.
    p.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .filter(|o| o.is_ne())
            .unwrap_or_else(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });
    if p.len() < 3 {
        return p;
    }
    let cr = |o: Vec2, a: Vec2, b: Vec2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut lo: Vec<Vec2> = Vec::new();
    for q in &p {
        while lo.len() >= 2 && cr(lo[lo.len() - 2], lo[lo.len() - 1], *q) <= 0.0 {
            lo.pop();
        }
        lo.push(*q);
    }
    let mut up: Vec<Vec2> = Vec::new();
    for q in p.iter().rev() {
        while up.len() >= 2 && cr(up[up.len() - 2], up[up.len() - 1], *q) <= 0.0 {
            up.pop();
        }
        up.push(*q);
    }
    lo.pop();
    up.pop();
    lo.extend(up);
    lo
}

#[cfg(test)]
mod tests {
    use super::*;

    /// See `rng.rs`'s own test-module comment for how these expected values
    /// were captured: block 4 sliced contiguously out of the frozen reference,
    /// balance-asserted at both boundaries, run under Node `vm.runInContext`,
    /// every function reached through the reference's own `UME._test` export.
    /// Every value below is compared **exactly** — none of this module's
    /// arithmetic passes through a transcendental function whose last bit could
    /// legitimately differ between V8 and Rust.
    fn p(x: f64, y: f64) -> Vec2 {
        Vec2::new(x, y)
    }
    fn pts(v: &[(f64, f64)]) -> Vec<Vec2> {
        v.iter().map(|&(x, y)| p(x, y)).collect()
    }
    fn flat(v: &[Vec2]) -> Vec<(f64, f64)> {
        v.iter().map(|q| (q.x, q.y)).collect()
    }
    fn square() -> Vec<Vec2> {
        pts(&[(0., 0.), (10., 0.), (10., 10.), (0., 10.)])
    }
    fn cw_square() -> Vec<Vec2> {
        let mut v = square();
        v.reverse();
        v
    }
    fn tri() -> Vec<Vec2> {
        pts(&[(0., 0.), (8., 0.), (0., 6.)])
    }
    fn l_shape() -> Vec<Vec2> {
        pts(&[(0., 0.), (12., 0.), (12., 4.), (5., 4.), (5., 11.), (0., 11.)])
    }

    #[test]
    fn golden_js_hypot_differs_from_rust_hypot() {
        // Values captured from the same Node run. The (3,3) row is the one
        // that broke `dist_pt_seg` before `js_hypot` existed, and the assert
        // below states plainly that Rust's own `hypot` gives a DIFFERENT
        // answer — so nobody "simplifies" this back to `f64::hypot` later.
        for (x, y, want) in [
            (3.0, 3.0, 4.242640687119286),
            (3.0, 4.0, 5.0),
            (0.1, 0.2, 0.223606797749979),
            (1e-300, 1e-300, 1.4142135623730952e-300),
            (1e300, 1e300, 1.4142135623730952e300),
            (0.0, 0.0, 0.0),
            (0.0, 5.0, 5.0),
            (-7.0, 0.0, 7.0),
            (1.0, 1.0, std::f64::consts::SQRT_2),
            (2.5, 7.25, 7.668930825088984),
            (123.456, 789.012, 798.612121170221),
            (1e-320, 0.0, 1e-320),
        ] {
            assert_eq!(js_hypot(x, y), want, "js_hypot({x}, {y})");
        }
        assert_ne!(js_hypot(3.0, 3.0), 3.0f64.hypot(3.0));
        // Math.hypot(±∞, NaN) is ∞ per spec, ahead of the NaN rule.
        assert!(js_hypot(f64::INFINITY, f64::NAN).is_infinite());
        assert!(js_hypot(f64::NAN, 1.0).is_nan());
    }

    #[test]
    fn golden_js_exp_differs_from_rust_exp() {
        // Captured from the same Node run. Every row below is one where the
        // platform `f64::exp` gives a DIFFERENT answer, which the assert at the
        // end states plainly -- the same device `js_hypot` carries, for the same
        // reason: this is not an accuracy improvement waiting to be made.
        const CASES: [(f64, f64); 8] = [
            (-2.449164366815239, 0.08636572642093979),
            (-1.529806137084961, 0.21657764962095738),
            (-2.3961539268493652, 0.0910675329974808),
            (-1.8253226280212402, 0.16116563918983867),
            (-2.2492117881774902, 0.10548233422645462),
            (-1.816183090209961, 0.16264537037235),
            (-1.0032005310058594, 0.366703913772948),
            (-1.1515216827392578, 0.3161553150757277),
        ];
        let mut differ = 0;
        for (x, want) in CASES {
            assert_eq!(js_exp(x), want, "js_exp({x})");
            if x.exp() != want {
                differ += 1;
            }
        }
        assert_eq!(differ, CASES.len(), "these rows exist to discriminate; f64::exp now agrees on some");

        // and the rest of the domain, including the special case measured at
        // exactly 1.0 and the reduction-branch boundaries
        for (x, want) in [
            (0.0, 1.0),
            (-0.0, 1.0),
            (1.0, std::f64::consts::E),
            (-1.0, 0.36787944117144233),
            (2.0, 7.38905609893065),
            (0.5, 1.6487212707001282),
            (1e-30, 1.0),
            (709.78, 1.7928227943945155e308),
            (-745.2, 0.0),
        ] {
            assert_eq!(js_exp(x), want, "js_exp({x})");
        }
        assert!(js_exp(709.79).is_infinite());
        assert!(js_exp(f64::INFINITY).is_infinite());
        assert_eq!(js_exp(f64::NEG_INFINITY), 0.0);
        assert!(js_exp(f64::NAN).is_nan());
    }

    #[test]
    fn golden_poly_area() {
        assert_eq!(poly_area(&square()), 100.0);
        assert_eq!(poly_area(&cw_square()), -100.0);
        assert_eq!(poly_area(&tri()), 24.0);
        assert_eq!(poly_area(&l_shape()), 83.0);
        // collinear -> exactly zero, and a self-crossing bowtie's lobes cancel
        assert_eq!(poly_area(&pts(&[(1., 1.), (3., 1.), (5., 1.)])), 0.0);
        assert_eq!(poly_area(&pts(&[(0., 0.), (10., 10.), (10., 0.), (0., 10.)])), 0.0);
    }

    #[test]
    fn golden_poly_centroid() {
        assert_eq!(flat(&[poly_centroid(&square())]), [(5.0, 5.0)]);
        assert_eq!(flat(&[poly_centroid(&cw_square())]), [(5.0, 5.0)]);
        assert_eq!(flat(&[poly_centroid(&tri())]), [(2.6666666666666665, 2.0)]);
        assert_eq!(
            flat(&[poly_centroid(&l_shape())]),
            [(4.524096385542169, 4.319277108433735)]
        );
        // degenerate (zero signed area) -> vertex mean, the reference's fallback
        assert_eq!(flat(&[poly_centroid(&pts(&[(1., 1.), (3., 1.), (5., 1.)]))]), [(3.0, 1.0)]);
    }

    #[test]
    fn golden_point_in_poly() {
        // includes the reference's asymmetric edge convention: the (0,0) corner
        // and the x=10 edge of the same square answer differently.
        for (pt, poly, want) in [
            (p(5., 5.), square(), true),
            (p(-1., 5.), square(), false),
            (p(0., 0.), square(), true),
            (p(10., 5.), square(), false),
            (p(2., 2.), tri(), true),
            (p(7., 5.), tri(), false),
            (p(3., 8.), l_shape(), true),
            (p(8., 8.), l_shape(), false),
        ] {
            assert_eq!(point_in_poly(pt, &poly), want, "point_in_poly({pt:?})");
        }
    }

    #[test]
    fn golden_seg_int() {
        assert_eq!(
            seg_int(p(0., 0.), p(10., 10.), p(0., 10.), p(10., 0.)),
            Some(SegHit { t: 0.5, u: 0.5, pt: p(5., 5.) })
        );
        assert_eq!(seg_int(p(0., 0.), p(10., 0.), p(0., 5.), p(10., 5.)), None); // parallel
        assert_eq!(
            seg_int(p(0., 0.), p(10., 0.), p(5., 0.), p(5., 5.)),
            Some(SegHit { t: 0.5, u: 0.0, pt: p(5., 0.) })
        ); // T-junction: u lands exactly on 0
        assert_eq!(seg_int(p(0., 0.), p(4., 0.), p(6., 0.), p(10., 0.)), None); // collinear
        assert_eq!(seg_int(p(0., 0.), p(1., 1.), p(2., 2.), p(3., 3.)), None); // collinear, apart
        let h = seg_int(p(0., 0.), p(3., 7.), p(1., 6.), p(9., 2.)).expect("oblique crossing");
        assert_eq!(h.t, 0.7647058823529411);
        assert_eq!(h.u, 0.16176470588235295);
        assert_eq!((h.pt.x, h.pt.y), (2.2941176470588234, 5.352941176470588));
    }

    #[test]
    fn golden_dist_pt_seg() {
        assert_eq!(dist_pt_seg(p(5., 5.), p(0., 0.), p(10., 0.)), 5.0);
        assert_eq!(dist_pt_seg(p(-3., 4.), p(0., 0.), p(10., 0.)), 5.0); // clamped to t=0
        assert_eq!(dist_pt_seg(p(13., 4.), p(0., 0.), p(10., 0.)), 5.0); // clamped to t=1
        assert_eq!(dist_pt_seg(p(2., 2.), p(1., 1.), p(1., 1.)), std::f64::consts::SQRT_2); // degenerate
        assert_eq!(dist_pt_seg(p(3., 9.), p(0., 0.), p(6., 6.)), 4.242640687119286);
    }

    #[test]
    fn golden_ensure_ccw() {
        let want = flat(&square());
        assert_eq!(flat(&ensure_ccw(&square())), want);
        assert_eq!(flat(&ensure_ccw(&cw_square())), want);
    }

    #[test]
    fn golden_chaikin() {
        assert_eq!(
            flat(&chaikin(&tri(), false)),
            [(0., 0.), (2., 0.), (6., 0.), (6., 1.5), (2., 4.5), (0., 6.)]
        );
        assert_eq!(
            flat(&chaikin(&square(), true)),
            [
                (2.5, 0.), (7.5, 0.), (10., 2.5), (10., 7.5),
                (7.5, 10.), (2.5, 10.), (0., 7.5), (0., 2.5)
            ]
        );
        assert_eq!(
            flat(&chaikin(&l_shape(), true)),
            [
                (3., 0.), (9., 0.), (12., 1.), (12., 3.), (10.25, 4.), (6.75, 4.),
                (5., 5.75), (5., 9.25), (3.75, 11.), (1.25, 11.), (0., 8.25), (0., 2.75)
            ]
        );
        // buildSite's own idiom: chaikin(chaikin(pts,false),false)
        let once = chaikin(&pts(&[(0., 0.), (3., 5.), (9., 1.), (12., 7.)]), false);
        assert_eq!(
            flat(&chaikin(&once, false)),
            [
                (0., 0.), (0.1875, 0.3125), (0.5625, 0.9375), (1.125, 1.875), (1.875, 3.125),
                (2.8125, 3.8125), (3.9375, 3.9375), (5.25, 3.5), (6.75, 2.5), (8.0625, 2.125),
                (9.1875, 2.375), (10.125, 3.25), (10.875, 4.75), (11.4375, 5.875),
                (11.8125, 6.625), (12., 7.)
            ]
        );
    }

    #[test]
    fn golden_simplify() {
        let line: Vec<Vec2> =
            (0..=20).map(|i| p(i as f64 * 5.0, (i as f64 / 3.0).sin() * 9.0)).collect();
        assert_eq!(
            flat(&simplify(&line, 0.5)),
            [
                (0., 0.), (15., 7.573238863271069), (25., 8.958671619765884),
                (35., 6.507772935644921), (60., -6.811222457771354), (70., -8.990594253881355),
                (80., -7.31996452410822), (100., 3.3673610751409795)
            ]
        );
        assert_eq!(
            flat(&simplify(&line, 3.0)),
            [
                (0., 0.), (25., 8.958671619765884), (70., -8.990594253881355),
                (100., 3.3673610751409795)
            ]
        );
        assert_eq!(flat(&simplify(&line, 30.0)), [(0., 0.), (100., 3.3673610751409795)]);
        assert_eq!(flat(&simplify(&pts(&[(0., 0.), (1., 1.)]), 1.0)), [(0., 0.), (1., 1.)]);
    }

    #[test]
    fn golden_inset_poly() {
        assert_eq!(
            flat(&inset_poly(&square(), &[3., 3., 3., 3.]).expect("uniform inset")),
            [(3., 3.), (7., 3.), (7., 7.), (3., 7.)]
        );
        assert_eq!(
            flat(&inset_poly(&square(), &[1., 2., 3., 4.]).expect("per-edge inset")),
            [(4., 1.), (8., 1.), (8., 7.), (4., 7.)]
        );
        // short `dists` clamps to its last entry, so one value insets uniformly
        assert_eq!(
            flat(&inset_poly(&square(), &[2.]).expect("clamped dists")),
            [(2., 2.), (8., 2.), (8., 8.), (2., 8.)]
        );
        // the reference's three rejections, all `None`:
        assert!(inset_poly(&square(), &[6., 6., 6., 6.]).is_none()); // inverted
        assert!(inset_poly(&ensure_ccw(&tri()), &[1., 1., 1.]).is_none()); // area < 15
        assert!(inset_poly(&ensure_ccw(&l_shape()), &[2.; 6]).is_none()); // reflex corner
        assert!(inset_poly(&pts(&[(0., 0.), (1., 0.)]), &[1., 1.]).is_none()); // n < 3
    }

    #[test]
    fn golden_clip_convex() {
        // The segment-vs-line caveat, pinned: a square overlapping the clip
        // window's corner yields the reference's own partial result, and a
        // triangle poking outside collapses to EMPTY rather than clipping.
        assert_eq!(
            flat(&clip_convex(&square(), &pts(&[(4., 4.), (14., 4.), (14., 14.), (4., 14.)]))),
            [(4., 7.6), (10., 4.), (10., 10.), (4., 10.)]
        );
        assert!(
            clip_convex(&ensure_ccw(&tri()), &pts(&[(1., 1.), (6., 1.), (6., 6.), (1., 6.)]))
                .is_empty()
        );
        assert!(
            clip_convex(&square(), &pts(&[(50., 50.), (60., 50.), (60., 60.), (50., 60.)]))
                .is_empty()
        );
        // a fully-contained subject passes through untouched
        assert_eq!(
            flat(&clip_convex(&pts(&[(3., 3.), (6., 3.), (6., 6.), (3., 6.)]), &square())),
            [(3., 3.), (6., 3.), (6., 6.), (3., 6.)]
        );
    }

    #[test]
    fn golden_convex_hull() {
        assert_eq!(
            flat(&convex_hull(&pts(&[
                (0., 0.), (5., 1.), (10., 0.), (10., 10.),
                (5., 5.), (0., 10.), (2., 6.), (8., 3.)
            ]))),
            [(0., 0.), (10., 0.), (10., 10.), (0., 10.)]
        );
        // collinear input collapses to its two extremes
        assert_eq!(
            flat(&convex_hull(&pts(&[(0., 0.), (1., 1.), (2., 2.), (3., 3.)]))),
            [(0., 0.), (3., 3.)]
        );
        // fewer than three points short-circuits, returning the SORTED input
        assert_eq!(flat(&convex_hull(&pts(&[(1., 2.), (3., 4.)]))), [(1., 2.), (3., 4.)]);
        assert_eq!(
            flat(&convex_hull(&pts(&[(0., 0.), (0., 0.), (4., 0.), (4., 4.), (0., 4.), (2., 2.)]))),
            [(0., 0.), (4., 0.), (4., 4.), (0., 4.)]
        );
    }

    #[test]
    fn poly_self_intersects_matches_its_only_caller_s_needs() {
        // No golden path: `polySelfIntersects` is not on the reference's
        // `_test` export, so this is a real unit test of the ported logic,
        // documented as such (same precedent as territory/provinces).
        assert!(!poly_self_intersects(&square()));
        assert!(!poly_self_intersects(&l_shape()));
        assert!(poly_self_intersects(&pts(&[(0., 0.), (10., 10.), (10., 0.), (0., 10.)])));
        // adjacent edges share a vertex and must not count as a crossing
        assert!(!poly_self_intersects(&tri()));
    }

    #[test]
    fn vector_helpers_hold_the_reference_s_zero_length_convention() {
        // `V.norm` divides by `hypot||1`, so the zero vector maps to itself
        // instead of to NaN — several call sites depend on that, so it is
        // asserted rather than left implicit.
        let z = Vec2::default().norm();
        assert_eq!((z.x, z.y), (0.0, 0.0));
        assert_eq!(p(3., 4.).len(), 5.0);
        assert_eq!(p(1., 0.).rot90(), p(0., 1.));
        assert_eq!(p(1., 2.).cross(p(3., 4.)), -2.0);
        assert_eq!(p(1., 2.).dot(p(3., 4.)), 11.0);
        assert_eq!(p(0., 0.).lerp(p(10., 20.), 0.25), p(2.5, 5.0));
    }
}
