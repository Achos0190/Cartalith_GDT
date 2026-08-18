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

// ---------------------------------------------------------------------------
// The JS-semantics math these algorithms are written against
// ---------------------------------------------------------------------------
//
// `js_hypot`, `js_min`, `js_max`, `js_round`, `js_exp`, `js_sin`, `js_cos` and
// `js_log` were written here, milestone by milestone, as each measurement
// found the next V8 libm that Rust's standard library does not reproduce. They
// now live in `cartalith-jsmath`, the workspace's one dependency-free leaf
// crate, together with the `js_atan2` `cartalith-hydrology` had grown
// independently and the copies five other crates had grown of the rest
// (`JS_SEMANTICS_AUDIT.md` recommendation #2).
//
// Re-exported rather than merely imported, so that `geom::js_hypot` keeps
// meaning what it meant to every call site in this crate, and so the FDLIBM
// goldens that moved out with the code cannot be mistaken for tests this
// module still owns. `cartalith-jsmath` has no dependencies of its own, so
// this crate still sees exactly `cartalith-rng` and nothing else in the
// pipeline (milestone 6's rule).
pub use cartalith_jsmath::{
    js_atan2, js_cos, js_exp, js_hypot, js_log, js_max, js_min, js_round, js_sin,
};

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
