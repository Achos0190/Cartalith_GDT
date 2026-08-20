//! Labelled deterministic RNG substreams — reference lines 28178-28190.
//!
//! ```js
//! function fnv1a(str){let h=0x811c9dc5;for(let i=0;i<str.length;i++){h^=str.charCodeAt(i);h=Math.imul(h,0x01000193);}return h>>>0;}
//! function stream(seed,label){
//!   const f=mulberry32((seed>>>0)^fnv1a(label));
//!   ...
//! }
//! ```
//!
//! **The RNG question, checked rather than assumed** (the same care Phase 2
//! milestone 9 took over `_civRng`): block 4's own header comment says
//! `mulberry32` is "intentionally NOT redefined here [...] it falls through to
//! the byte-identical module-scope copy already in script block 1". So the
//! generator is literally the one `cartalith-rng` already golden-verifies —
//! not merely the same algorithm under a different wrapper. What is new here is
//! the **seed derivation**: `mulberry32(seed ^ fnv1a(label))`, a labelled
//! substream so that (say) `'grow/e3'` draws an independent sequence from
//! `'parcels/blk7'` off one town seed. `fnv1a` has no Gen1 equivalent and is
//! ported here.
//!
//! `charCodeAt` yields UTF-16 code units, so this port hashes
//! `str::encode_utf16()` rather than bytes — identical for the ASCII labels the
//! engine actually uses, and correct rather than accidentally-correct if a
//! label ever carries a non-ASCII character.
//!
//! `Math.imul` is a 32-bit wrapping multiply, so the FNV round is
//! `wrapping_mul` on `u32`.

use cartalith_rng::Mulberry32;

/// `fnv1a` (reference line 28178) — FNV-1a over the label's UTF-16 code units.
pub fn fnv1a(label: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for u in label.encode_utf16() {
        h ^= u32::from(u);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// One labelled substream — `stream(seed, label)` (reference line 28179).
///
/// The JS returns an object of closures over a single `mulberry32` draw
/// function; every helper below advances that one shared generator, so **call
/// order is load-bearing** exactly as it is in the reference.
pub struct Substream {
    inner: Mulberry32,
}

/// `stream(seed,label)` — `mulberry32((seed>>>0) ^ fnv1a(label))`.
pub fn stream(seed: u32, label: &str) -> Substream {
    Substream { inner: Mulberry32::new(seed ^ fnv1a(label)) }
}

impl Substream {
    /// `r.u()` — the raw `mulberry32` draw in `[0, 1)`.
    pub fn u(&mut self) -> f64 {
        self.inner.next_f64()
    }

    /// `r.range(a,b)` — `a+(b-a)*f()`. Written in that exact form (not
    /// `lerp`, not FMA) because reordering it would change the last bits.
    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        let f = self.u();
        a + (b - a) * f
    }

    /// `r.int(a,b)` — `a+Math.floor(f()*(b-a+1))`, inclusive of both ends.
    pub fn int(&mut self, a: i64, b: i64) -> i64 {
        let f = self.u();
        a + (f * ((b - a + 1) as f64)).floor() as i64
    }

    /// The index `r.pick(arr)` would select: `Math.floor(f()*arr.length)`.
    ///
    /// Exposed as an index rather than an element so callers can pick out of
    /// any collection without this crate borrowing it. An empty collection
    /// yields `None` (JS would index `undefined`) — but the draw is still
    /// consumed first, because the JS evaluates `f()` before indexing and a
    /// port that skipped it would desynchronise the shared generator.
    pub fn pick_index(&mut self, len: usize) -> Option<usize> {
        let f = self.u();
        if len == 0 {
            return None;
        }
        Some((f * (len as f64)).floor() as usize)
    }

    /// `r.pick(arr)` over a slice.
    pub fn pick<'a, T>(&mut self, arr: &'a [T]) -> Option<&'a T> {
        self.pick_index(arr.len()).map(|i| &arr[i])
    }

    /// `r.norm()` — Box-Muller, **two draws per call**, `u1` clamped up to
    /// `1e-12` before the log. The reference's own comment flags this as
    /// same-engine deterministic.
    ///
    /// Through [`crate::geom::js_log`] and [`crate::geom::js_cos`], not
    /// `f64::ln` / `f64::cos`. Milestone 1 shipped this on the platform libm
    /// with a documented "they happen to agree" note; milestone 6 measured the
    /// disagreement — **1,647 of 60,009** arguments for `ln` and **2,160 of
    /// 80,214** for `cos` — and this is the highest-leverage call site in the
    /// subsystem, since [`Self::logn`] runs on top of it and draws every
    /// frontage width, plot depth and building dimension in the town. The
    /// milestone-1 goldens still pass, which is the check that this is a fix
    /// and not a change. `sqrt` needs no such treatment: IEEE-754 mandates a
    /// correctly-rounded square root, so V8's and Rust's agree by
    /// specification.
    pub fn norm(&mut self) -> f64 {
        let mut u1 = self.u();
        if u1 < 1e-12 {
            u1 = 1e-12;
        }
        let u2 = self.u();
        (-2.0 * crate::geom::js_log(u1)).sqrt()
            * crate::geom::js_cos(2.0 * std::f64::consts::PI * u2)
    }

    /// `r.logn(median,sig)` — `median*Math.exp(sig*r.norm())`.
    ///
    /// Through [`crate::geom::js_exp`], not `f64::exp`: milestone 5 measured
    /// the platform libm disagreeing with V8 on **20,721 of 240,000** random
    /// arguments. Milestone 1's goldens here happened to fall on values the two
    /// agree about, which is luck, not safety — `buildParcels` (milestone 12)
    /// draws every frontage width and plot depth through this call.
    pub fn logn(&mut self, median: f64, sig: f64) -> f64 {
        median * crate::geom::js_exp(sig * self.norm())
    }

    /// `r.chance(p)` — `f() < p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.u() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every expected value below is the reference's own output, captured by
    /// slicing block 4 (lines 28167-31103) out of the frozen HTML as **one
    /// contiguous block** — plus line 2291, `mulberry32`, which block 4
    /// deliberately does not define — and running it under Node's bare
    /// `vm.runInContext` with no DOM, with a block-comment balance assertion on
    /// both slice boundaries. That assertion is Journey Planner milestone 4's
    /// design, adopted here for the same reason: an unterminated `/*` at a
    /// slice boundary silently swallows the rest of the slice, and the balance
    /// check removes the whole class.
    ///
    /// Everything is compared **exactly**, including `norm`/`logn`. Those run
    /// through `log`/`cos`/`exp`, all three of which V8 computes with FDLIBM
    /// and the platform does not — so all three go through
    /// [`crate::geom::js_log`], [`crate::geom::js_cos`] and
    /// [`crate::geom::js_exp`] rather than `f64`'s. Milestone 1 asserted
    /// exactness here on the platform libm and it passed, which was luck: the
    /// values below happen to be ones the two agree about. Milestones 5 and 6
    /// replaced the luck with the right function, and these goldens passing
    /// unchanged afterwards is the check on that.
    fn eq_exact(a: f64, b: f64) {
        assert_eq!(a, b);
    }

    #[test]
    fn golden_fnv1a() {
        // reference: UME.fnv1a(label)
        for (label, want) in [
            ("", 2166136261u32),
            ("site", 932310606),
            ("anchors", 592852805),
            ("grow/e1", 2820215005),
            ("grow/e8", 2669216434),
            ("parcels/blk0", 2567477463),
            ("plaza", 3504834741),
            ("wall", 2804296981),
            ("buildings", 2366751346),
            ("a", 3826002220),
            ("Cartalith", 106998565),
            ("blk12", 1475724291),
        ] {
            assert_eq!(fnv1a(label), want, "fnv1a({label:?})");
        }
    }

    #[test]
    fn golden_stream_raw_draws() {
        // reference: const r=UME.stream(seed,label); r.u() x8
        let cases: [(u32, &str, [f64; 8]); 6] = [
            (0, "site", [
                0.6914413259364665, 0.7049627169035375, 0.2130293461959809, 0.4871823317371309,
                0.2513629775494337, 0.06259308592416346, 0.8516745162196457, 0.20995264616794884,
            ]),
            (1, "site", [
                0.6378936148248613, 0.5494151888415217, 0.7164829955436289, 0.7719443573150784,
                0.6765064685605466, 0.353879984235391, 0.6837638353463262, 0.7769494773820043,
            ]),
            (24601, "anchors", [
                0.6799897165037692, 0.4719015774317086, 0.5714706724975258, 0.2915750911924988,
                0.6476951758377254, 0.28165796119719744, 0.4264061104040593, 0.9736984998453408,
            ]),
            (0xDEAD_BEEF, "grow/e3", [
                0.7057416231837124, 0.6982163537759334, 0.6548164251726121, 0.19638055004179478,
                0.01001868536695838, 0.05670433375053108, 0.5116014182567596, 0.9704403632786125,
            ]),
            (7, "", [
                0.12754531647078693, 0.38559746788814664, 0.882713150465861, 0.9036114742048085,
                0.6980974909383804, 0.5457241979893297, 0.9258443301077932, 0.9757276917807758,
            ]),
            (4294967295, "parcels/blk0", [
                0.4172972086817026, 0.9679008240345865, 0.05334676126949489, 0.6028361127246171,
                0.10438563604839146, 0.5414658021181822, 0.4660848423372954, 0.6565632955171168,
            ]),
        ];
        for (seed, label, want) in cases {
            let mut r = stream(seed, label);
            for (k, w) in want.iter().enumerate() {
                assert_eq!(r.u(), *w, "stream({seed},{label:?}) draw {k}");
            }
        }
    }

    #[test]
    fn golden_stream_helpers() {
        // reference: one stream(12345,'ops') driven through each helper in
        // this order — the shared generator makes the ORDER part of the golden.
        let mut r = stream(12345, "ops");
        for w in [
            4.211377810395788, 1.4987015617080033, 0.2666817402932793, -1.873793491453398,
            -0.7433067164383829, 2.027579218149185,
        ] {
            assert_eq!(r.range(-3.5, 7.25), w);
        }
        for w in [2i64, 4, 6, 6, 6, 2] {
            assert_eq!(r.int(1, 6), w);
        }
        let arr = ["a", "b", "c", "d", "e"];
        for w in [3usize, 4, 3, 1, 1, 1] {
            assert_eq!(r.pick_index(arr.len()), Some(w));
        }
        for w in [
            0.7194811092149691, -1.0787318479801071, -0.6235400038093816, -0.27552107805101733,
            -0.5932317847657164, 0.22172583226003054,
        ] {
            eq_exact(r.norm(), w);
        }
        for w in [
            19.08009791574942, 17.82269979596583, 23.787387692650107, 12.955974614748477,
            19.156887781038392, 16.517651333446334,
        ] {
            eq_exact(r.logn(22.0, 0.28), w);
        }
        for w in [true, false, false, true, false, false] {
            assert_eq!(r.chance(0.35), w);
        }
    }

    #[test]
    fn golden_logn_at_a_real_call_site() {
        // reference: buildParcels' own substream shape,
        // stream((fnv1a(blk.id)^0)>>>0,'parcels/'+blk.id), driven through logn
        // as buildParcels drives it — pins the Box-Muller draw pairing against
        // a call site the engine really has, not just a synthetic one.
        let mut r = stream(fnv1a("blk7"), "parcels/blk7");
        for w in [
            39.39784913136483, 20.438528410799016, 31.163337459954445, 31.013435141033312,
            22.22370932877332,
        ] {
            eq_exact(r.logn(30.0, 0.28), w);
        }
    }

    #[test]
    fn pick_on_empty_is_none_but_still_consumes_a_draw() {
        // JS `arr[Math.floor(f()*0)]` is `undefined`; this port says so in the
        // type. `f()` is evaluated regardless, so the generator advances —
        // a port that short-circuited would desynchronise every later draw.
        let mut r = stream(1, "x");
        assert_eq!(r.pick::<u8>(&[]), None);
        let mut plain = stream(1, "x");
        plain.u();
        assert_eq!(r.u(), plain.u());
    }
}
