# JavaScript-semantics audit

A workspace-wide sweep for places where this port silently computes something
different from `reference/Cartalith Gen1 v2.10.html` because Rust's standard
library and V8 disagree about what a floating-point operation means.

**Read this before porting anything.** Five such divergences had been found by
2026-08-18, each by whichever milestone happened to trip over it, each *after*
the code had already passed golden tests, and at least two of them
retroactively invalidated work that had been called verified. Nobody had ever
swept the whole workspace. This document is that sweep, plus the catalogue a
future porter should consult first rather than rediscovering an entry the hard
way.

Audit run 2026-08-18 against all fourteen crates. Every measurement below was
taken against **V8 itself** (`node` v24.19.0) or against an exhaustive scan —
none of it is reasoning from a specification alone.

**Follow-up pass, same day:** recommendation #1 was acted on. `js_atan2` is
ported and `build_channels` is fixed; §2.3, §4.4 and §6 carry the result.

**Second follow-up, same day: recommendation #2 is done.** Every helper in this
catalogue now has exactly **one** implementation, in a new dependency-free leaf
crate, `cartalith-jsmath`. All three copy disagreements in §3 are resolved
rather than recorded, and the two remaining live hazards §4.4 named — the urban
half-edge sort key and `-terrain:372`'s plate circular mean — are both fixed and
proved. §2.4, §2.5, §3, §4.4, §5 and §6.2 carry the result.

The document is kept current rather than superseded, because it is meant to be
read *before* porting, not after.

---

## 1. The catalogue

Nine rows: **eight distinct operations** — `hypot`, `exp`, `atan2`, `sin`,
`cos`, `ln`, NaN-in-`min`/`max`, and NaN-falsiness — plus the rounding-mode
family, which is one row because its four members share one cause. The first
five were found by earlier milestones; #6–#8 were being found by the
`cartalith-urban` fork while this sweep ran; #9 is this sweep's own.

**Every one of them now has exactly one implementation, and it is in
`cartalith-jsmath`.** Where a crate's own module path was load-bearing (`geom::
js_hypot`, `sculpt::js_hypot`, `tile_render::u8_clamped`) it survives as a
`pub use` re-export, so no call site had to move.

| # | What differs | The one JS-faithful helper |
|---|---|---|
| 1 | `Math.hypot` ≠ `f64::hypot` | `cartalith-jsmath::js_hypot` / `js_hypot3` / `js_hypot_n` (was 7 copies, 5 distinct sums) |
| 2 | `Math.exp` ≠ `f64::exp` | `cartalith-jsmath::js_exp` |
| 3 | `Math.min`/`Math.max` propagate NaN; Rust's absorb it | `cartalith-jsmath::js_min` / `js_max` (was 3 copies, §3.3) |
| 4 | `NaN` is falsy in JS (`p.pop||0`, `maxPop ? … : 0`) | `cartalith-jsmath::js_num_or_zero` / `js_truthy_num` |
| 5 | Rounding modes: `Math.round`, `toFixed`, `Uint8ClampedArray` | `cartalith-jsmath::js_round` (was 7 copies, §3.1), `js_fixed`, `js_to_fixed`, `u8_clamped` |
| 6 | `Math.sin` ≠ `f64::sin` | `cartalith-jsmath::js_sin` |
| 7 | `Math.cos` ≠ `f64::cos` | `cartalith-jsmath::js_cos` |
| 8 | `Math.log` ≠ `f64::ln` | `cartalith-jsmath::js_log` |
| 9 | `Math.atan2` ≠ `f64::atan2` | `cartalith-jsmath::js_atan2` (plus `js_atan`, now public) |

### 1.1 How far apart they actually are

200 000 random arguments per function, drawn in the ranges this engine really
uses, compared bit for bit against V8:

| operation | Rust disagrees with V8 on |
|---|---|
| `atan2` | **45 970 / 200 000 — 22.98 %** |
| `exp` | 19 030 / 200 000 — 9.52 % |
| `ln` | 6 801 / 200 000 — 3.40 % |
| `cos` | 4 680 / 200 000 — 2.34 % |
| `sin` | 4 671 / 200 000 — 2.34 % |
| `pow` | 79 / 200 000 — 0.04 % |
| `sqrt` | **0** — IEEE-754 mandates correct rounding, so this one is safe forever |
| `js_exp` (the ported FDLIBM) | **0 / 200 000** |
| `js_atan2` (the ported FDLIBM) | **0 / 240 000** (follow-up pass, §2.3) |

Every disagreement is one ulp. That is not a reason to relax: divergence #1's
own history is a one-ulp `hypot` turning a four-node road graph into a
three-node one, because 11 m was a snap threshold.

`atan2` is the headline. It is the *largest* divergence in the workspace and it
had eight live call sites with no `js_atan2` anywhere. It now has one, reachable
from every crate. See §2.3, §2.4, §2.5 and §4.4.

**A note on the two `atan2` percentages in this document.** The sweep's own
draw gave 22.98 % over 200 000 arguments; the follow-up pass's independent
draw gave **17.01 % over 240 000** (40 824 disagreements), across four bands —
`[-1,1]`, height gradients at `1e-8..1e-1`, coordinate deltas at `±4096`, and
mixed magnitudes at `2^±40`. Both numbers are real; they differ because the
divergence rate depends on the argument distribution, and neither sample is
"the" answer. The conclusion is the same either way, and the figure that
actually settled the question is the one below it: the ported function
disagrees with V8 on **zero** arguments in either sample.

### 1.2 What is safe by construction

Worth knowing so nobody spends a milestone on it:

- **`sqrt`, `abs`, `floor`, `ceil`, `trunc`, and `+ - * /`** are all IEEE-754
  correctly-rounded or exact. Rust and V8 cannot differ. `.sqrt()` needs no
  helper and never will.
- **`f64::clamp(lo, hi)` already matches JS.** `NaN < lo` and `NaN > hi` are
  both false, so `clamp` returns NaN — exactly what
  `Math.max(lo, Math.min(hi, x))` does. Divergence #3 is a hazard of the
  *hand-written* `lo.max(hi.min(x))` idiom, not of `.clamp()`. This is why the
  sweep found so few live #3 sites: most of the workspace already uses
  `.clamp`.
  **One caveat:** `f64::clamp` *panics* if `lo > hi` or if either bound is NaN,
  where JS would not. Any `.clamp()` whose bounds are computed rather than
  literal is a panic risk, and a panic crossing the gdext boundary takes the
  Godot process down (`cartalith-rust-conventions`).

---

## 2. What this sweep and its two follow-up passes changed

Five real divergences fixed: two found by the sweep itself (both in
`cartalith-spatial`, §2.1 and §2.2), one by the `js_atan2` follow-up
(`cartalith-hydrology`, §2.3), and two by the consolidation pass — the
`cartalith-urban` half-edge sort key (§2.4) and `cartalith-terrain`'s plate
circular mean (§2.5), which are the two live hazards §4.4 had named and left.

Each is proved with a test that fails before the fix and passes after, or — for
the two whose effect no existing fixture can express — with a bulk differential
against `node` at the exact seam. Every expected value is re-derived from V8
rather than read off the new code.

### 2.1 `PaintStamp::apply` painted rim cells the reference skips

`cartalith-spatial/src/paint.rs`. The brush gate is `_paintAt`'s
`if(Math.hypot(dx,dy)>R) continue`, and the module comment correctly said that
"the exact set of rim cells" depends on it — then computed it with
`f64::hypot`.

`f64::hypot` and V8's disagree on **1 398 of the 4 096** integer offsets
`(dx, dy) ∈ [0, 64)²`. Almost all of that is invisible, because only the boolean
survives and the two values straddle `R` only when the true distance is exactly
`R` — which needs a Pythagorean triple. An exhaustive scan of every integer
radius `1..=512` finds 25 radii where a cell actually changes, the first at:

```
R = 125, cells (±35, ±120) and (±120, ±35)        35² + 120² = 125²
  true value                            125
  f64::hypot(35, 120)                   125                     -> not > R -> painted
  V8 Math.hypot(35, 120)                125.00000000000001421   -> > R     -> SKIPPED
```

(`node -e "Math.hypot(35,120) > 125"` → `true`.)

**Was it live?** No — the reference's own sliders cap `_paintRadius` at 40 and
`_civTerRadius` at 20, and no radius below 125 differs. But `PaintStamp::new`
takes an uncapped `f64`, and the invariant the module *claimed* ("for every
integer radius … the two are identical") is false. Fixed by computing V8's
`Math.hypot`, which removes the need for any invariant.

Tests added: `rim_cells_on_a_pythagorean_triple_follow_v8_not_rust_hypot`
(fails before, passes after) and
`below_radius_125_the_two_hypots_agree_on_every_cell` (an exhaustive
`1..=124` × all-offsets proof that no existing golden could have moved).

### 2.2 `js_to_fixed` rounded down on roughly one value in ten

`cartalith-spatial/src/geo.rs`, the `Number.prototype.toFixed` used for **every
GeoJSON coordinate and every way length**. One expression,
`round_up = first > 5 || (tie && !neg)`, carried two bugs.

**Bug A — the serious one.** A first dropped digit of `5` followed by any
nonzero digit rounded *down*. That is not a last-place nicety; it is a whole
unit in the last kept place, and it fires whenever the first dropped digit is
`5`, which is roughly one value in ten.

```
js_to_fixed(9.051, 1)              = 9.0      V8: "9.1"
js_to_fixed(286.4957967118851, 2)  = 286.49   V8: "286.50"
```

**Bug B.** A tie on a negative value rounded toward zero. ECMA-262 21.1.3.3
step 6 sets `s` to `"-"` and `x` to `-x` *before* picking "the larger n", so
the choice is made on the magnitude and a tie goes away from zero on both
signs. `js_to_fixed(-0.0625, 3)` gave `-0.062`; V8 gives `-0.063`.

Both collapse to one rule — round the magnitude to nearest, ties away from
zero — so the fix is `round_up = first >= 5`, with no special case for the tie
and none for the sign.

**Why no test caught it, and why one test asserted the bug.**
`golden_parity_geojson.rs` reaches this function on every feature it exports.
Its world is 600 km over 12 cells, so `cell_km` is exactly `50` and every
coordinate it rounds is already an integer; its one deliberately-fractional
value, a way of `38.4567` km, has `6` as its first dropped digit — the branch
that was correct. A golden fixture chosen to exercise every *feature type* did
not exercise a single *rounding branch*. Separately, a unit test asserted
`js_to_fixed(-0.0625, 3) == -0.062`, pinning bug B in place: a test is only as
right as the reference reading behind it, and that one had reasoned from a
paraphrase of the specification instead of running `node`.

That expectation is the only one this audit changed, and the replacement is
V8's own output. `golden_parity_geojson.rs` itself passes **unmodified**, which
is the proof that the bug was invisible to it.

Test added: `js_to_fixed_matches_v8_on_every_rounding_branch` — 32 cases
covering first-dropped-digit `<5`, `=5` with a tail, exact tie, `>5`, full
carry, both signs, `d = 0..3`, and the `cell_km` values a non-power-of-two map
really produces. Every expectation is `+v.toFixed(d)` read off `node`.

Two of those cases look like counterexamples and are not: `0.12345` rounds to
`0.123` and `0.15` to `0.1`, because the nearest doubles to those decimals are
*below* them (`0.1234499999…`, `0.1499999999…`). Neither is a tie. That is
exactly why the rule has to be read off the exact binary expansion rather than
the decimal literal, and why the earlier `(v * 10^d + 0.5).floor()` form in
`cartalith-civ` had to be rewritten when it was found *fabricating* ties.

For confidence that nothing else is wrong there: 60 000 randomised differential
cases against V8 now agree exactly, for both this function and
`cartalith-civ::js_fixed`.

### 2.3 `build_channels` steered rivers into the wrong cell

`cartalith-hydrology/src/lib.rs`, the follow-up pass. This is recommendation
#1, acted on.

**What V8 actually runs.** `Math.atan2` does *not* reach the platform libm.
V8 ships its own FDLIBM port in `src/base/ieee754.cc` — the same reason
`js_exp` exists — so the target is `__ieee754_atan2` plus the `atan` it calls.
Two details decide whether a transcription is right:

* the specification preamble (both signed zeros, each infinity quadrant, NaN);
* **`m &= 1` in the `|y/x| > 2**60` branch.** That line is the FreeBSD msun
  correction V8 carries and the original 1993 Sun fdlibm does not. Without it
  the port disagrees with V8 on **777 of 240 000** arguments — every one of
  them `x` tiny and negative, `y` large — returning one ulp above `pi/2` where
  V8 returns `pi/2` exactly. It was found by measurement, not by reading: the
  first transcription was the 1993 source and the differential run pointed
  straight at the branch.

**Measured.** Over 240 000 arguments in the four bands named in §1.1,
`f64::atan2` returns a different double from V8 on **40 824**; `js_atan2`
returns a different double on **0**. Over the 1 089-pair cross product of 33
special values (both zeros, both infinities, NaN, subnormals, `f64::MAX/MIN`,
and each fdlibm reduction boundary), `f64::atan2` differs on **42** and
`js_atan2` on **0**.

**The bug was live, and the mechanism is structural rather than a coincidence.**
`build_channels`'s receiver is a discrete argmax — the cell a river flows into
— so nothing absorbs a one-ulp difference the way an `f32` store does for the
`exp` sites in §4.2. The reachable case:

1. a cell whose 3x3 is left-right symmetric has `gx == 0.0` **exactly**;
2. so `aspect = atan2(-gy, -0.0)` lands on the signed-zero branch and comes out
   at exactly `-pi/2`;
3. its two symmetric downhill diagonals then have **exactly equal** `drop`, and
   `|wrap(atan2(dy,dx) - aspect)|` is mathematically `3*pi/4` for both;
4. so the argmax is settled purely by which of two last bits is larger, and
   `f64::atan2` settles it differently from V8. `score > best_score` is strict,
   so the tie goes to whichever neighbour the loop reached first.

That configuration is a ridge, saddle or plateau edge — ordinary terrain, not a
contrivance. The whole decision for a cell depends on exactly its 3x3 block, so
the domain can be sampled directly. Over **1 200 000** random 3x3 blocks on a
quantised height lattice, `f64::atan2` picks a **different receiver from V8 on
84**; `js_atan2` picks V8's on all 1 200 000. Every one of the 43 divergent
blocks the search kept was then re-checked against `node` running the
reference's own channelization loop: **V8 agreed with `js_atan2` on 43 of 43
and with `f64::atan2` on 0 of 43.** The port was picking the wrong cell.

**`sin`/`cos` are deliberately *not* ported here, and that is a measured
decision, not an omission.** They diverge from V8 too (2.34 % each) and they
sit in the same expression. But the wrap `atan2(sin(da), cos(da))` only decides
the outcome when the two competing `da` are exact negatives of each other, and
`sin`/`cos` preserve that antisymmetry exactly whatever their accuracy — so the
divergence cannot reach the argmax. Over **600 000** blocks spanning four
terrain regimes (uniform random, quantised lattice, near-flat plateau, and
perturbed-symmetric), `js_atan2` with Rust's own `sin`/`cos` agreed with V8 on
**every single receiver**. Porting `js_sin`/`js_cos` into this crate would have
been two hundred lines of unreachable FDLIBM and a ninth copy site. (The
consolidation later made all three reachable from everywhere at no copy cost,
which is what let §2.5 close `-terrain:372` — but the reasoning here still
holds for this call site, whose receiver is unaffected either way.)

**Why no existing golden could have caught it.** All three cases in
`golden_parity_river.rs` pass **unmodified**, which is the proof rather than
the excuse — but the reason is measurable. Instrumented, the three fixtures
channelize **365 cells between them, and not one has `gx == 0.0` exactly, nor
a top-two score gap below `1e-15`.** Their terrain is smooth and asymmetric, so
the precondition never arises. This is §4's recurring pattern in its sharpest
form yet: a golden that exercises the *feature* thoroughly and the *branch* not
at all. It is the same shape as §2.2's `js_to_fixed`, where a fixture chosen to
cover every feature type covered no rounding branch.

Tests added, all three in `cartalith-hydrology` (the first two moved to
`cartalith-jsmath` with the function, unchanged):
`js_atan2_matches_v8_on_every_branch` (44 arguments crossing every fdlibm
reduction interval, both `|y/x|` extremes and the `m &= 1` branch),
`js_atan2_matches_v8_on_the_spec_pinned_edge_cases` (the 26 cases ECMA-262
21.3.2.8 names plus the seven NaN combinations), and
`build_channels_receiver_follows_v8_not_rust_atan2` — two 3x3 grids, one on
ordinary generated-terrain `f32` values and one on exactly-representable ones,
which return receiver `8` before the fix and `6` after. V8 returns `6`.

One expectation in that second test was **wrong when first written** and the
test caught it: an extra hand-added assertion claimed
`atan2(-0.25, -0.0) == -pi`, reasoning from the signed-zero rules, where V8
gives `-pi/2` (`x` is a zero of either sign, so the answer is `±pi/2` and the
sign comes from `y` alone). It was written from reasoning instead of from
`node` — exactly the failure §5's recommendation #5 exists to prevent, caught
this time because the recommendation had also produced the habit of running
`node` before believing the fix.

### 2.4 `extract_faces` sorted half-edges by a key V8 computes differently

`cartalith-urban/src/graph.rs:607`, the second follow-up pass. This is the
hazard §4.4 named and left to the fork.

`ang` is `atan2` of an edge's endpoint delta, stored on a half-edge and used as
the **sort key** for the half-edges around a node — which is exactly what
`extract_faces`' traversal walks to pick the next edge. `sort_by` is stable, so
an exact tie keeps insertion order, but a one-ulp difference reorders two edges
leaving a node in nearly the same direction and the traversal then produces a
different city block.

**Measured, on the arguments this graph really produces.** Over 510 634
near-parallel edge-delta pairs on the arbitrary `f64` coordinates
`attach_point`'s split points and `buildPrimaries`' street ends actually have,
`f64::atan2` returns a different double from V8 on **196 034** — 38 %, higher
than any figure in §1.1 because the deltas are small differences of large
coordinates. More to the point, the two implementations put the pair in a
**different order on 23 814 of them, 4.7 %.** Three quarters of those are cases
where Rust manufactures a difference out of what V8 computes as an exact tie,
which the stable sort would otherwise have resolved by insertion order.

**Was it live?** Not in any fixture. All 20 of milestone 2's golden scenarios —
which compare the entire graph state plus every extracted face against the
reference's own `UME._test` output — pass **unmodified** after the fix, and the
reason is measurable rather than lucky: they are built from round coordinates
whose incidences at a node are milliradians apart, so no last bit can reorder
them. That is §4's recurring pattern again, and the fixtures being unable to see
the change is the proof that the change is safe, not an excuse for not making
it.

Test added: `graph::tests::the_half_edge_sort_key_orders_like_v8_not_like_rust`
— five real delta pairs from the search above, each with the bit pattern `node`
v24.19.0 returns for `Math.atan2(dy, dx)` and V8's own comparison of the two.
V8 agrees with `js_atan2` on 5 of 5 and with `f64::atan2` on 0 of 5.

### 2.5 `buildPlates`' circular mean, which `js_atan2` alone could not fix

`cartalith-terrain/src/lib.rs:372`, the second follow-up pass. §4.4 reported
this site and deliberately did **not** change it; this is that decision being
carried out on its own terms.

The site is `buildPlates`' `world`-mode circular mean of a plate's member cells'
x positions, `atan2(Σ sin θ, Σ cos θ)` scaled by `gw`. The audit's finding was
that **the divergence enters upstream of `atan2`**, so porting `js_atan2` alone
would leave the site *differently* wrong — worse than leaving it alone, because
the next reader would believe it had been handled. With `js_sin` and `js_cos`
now in the same crate, all three move together.

**Measured, 2 000 synthetic plates at `gw = 512`, arguments drawn by the
reference's own `mulberry32` so both sides evaluate the same points:**

| | disagrees with V8 |
|---|---|
| the `(Σ sin, Σ cos)` pair, before `atan2` is reached | **737 / 2 000** |
| final `plate.x`, `f64::sin`/`cos`/`atan2` | **193 / 2 000** |
| final `plate.x`, `js_atan2` only (the partial fix) | **110 / 2 000** |
| final `plate.x`, `js_sin` + `js_cos` + `js_atan2` | **0 / 2 000** |

The third row is the audit's "differently wrong" claim, now a measurement in
this repository rather than a sentence in a document: the partial fix is a real
improvement and still leaves the site disagreeing with V8 on one plate in
eighteen.

It matters because of `-terrain:347`: the next Lloyd iteration's
`dx = x as f64 - plate.x` feeds a nearest-plate argmin, structurally the same
discrete-decision hazard as §2.3's river receiver, one iteration removed.
(`-terrain:385` quantises through `js_round` and was measured safe either way.)

**Both `world`-mode cases of `golden_parity_plates.rs` pass unmodified.** Their
grids are 6×5 and 7×6, so their circular means are over a handful of cells and
land on the same double either way — the *feature* golden, again passing because
it cannot see the *branch*.

Test added: `cartalith-terrain/tests/golden_parity_plate_circular_mean.rs` —
one FNV-1a hash of all 2 000 `plate.x` values per seed, over two seeds, against
`node` v24.19.0 running the identical loop; plus assertions that Rust's own
libm and the `js_atan2`-only partial both produce a *different* hash, so the
rows cannot quietly stop discriminating.

---

## 3. Do the helpers agree with each other?

**They did not, and all three disagreements are now settled** — by there being
one implementation. What follows is kept as the record of what each copy did and
which answer won, because the next reader's temptation will be to "simplify" the
surviving form back into one of the wrong ones.

### 3.1 `js_round` — one input, six crates → **resolved, fractional-part form**

`cartalith-urban::geom::js_round` compared the fractional part. The other six
(`-assets::manual`, `-civ`, `-climate`, `-engine`, `-spatial::region`,
`-terrain`) all used `(x + 0.5).floor()`.

A sweep of 3 million random values plus every double within 3 ulp of every
half-integer in ±50 finds **exactly one** disagreeing input:

```
x = 0.49999999999999994        (the largest double below 0.5)
  (x + 0.5).floor()  = 1       because x + 0.5 rounds up to exactly 1.0
  V8 Math.round(x)   = 0
```

`cartalith-terrain`'s doc comment called `(x + 0.5).floor()` "the standard
exact equivalent". It is not, and the urban copy's own doc says so.

**First follow-up pass: the false comment was fixed, the six implementations
were not** — one unreachable input against six cross-crate edits was the wrong
trade while a fork was active.

**Second follow-up pass: resolved.** There is one `js_round`, in
`cartalith-jsmath`, and it is the fractional-part form. The six additive copies
are gone, and with them `cartalith-terrain`'s "standard exact equivalent"
comment. No golden moved — which is what §1's own reasoning predicted, since
`0.49999999999999994` is not reachable from any call site, but it is now a
measurement (the whole workspace suite, unmodified) rather than an argument.

### 3.2 `js_hypot` — NaN and infinity → **resolved, one compensated sum**

`cartalith-urban::geom::js_hypot` had a specification preamble; the copies in
`-assets::manual`, `-civ` and `-terrain::sculpt` did not:

| | V8 | urban | the other three |
|---|---|---|---|
| `hypot(NaN, 0)` | NaN | NaN | **0** |
| `hypot(∞, 3)` | ∞ | ∞ | **NaN** |
| `hypot(∞, NaN)` | ∞ | ∞ | **NaN** |

`hypot(NaN, 0)` → `0` is argument-order dependent: `hypot(0, NaN)` gives NaN in
all four. No live site can reach a NaN or infinite argument. The new copy added
in §2.1 has the preamble.

**Follow-up pass: fixed in all three.** This one *is* a cheap, safe fix, unlike
§3.1's: it is additive, it is one place per crate rather than six, and it can
only change the result for an argument that is infinite or NaN — inputs no live
site reaches, so no golden can move (and none did). In `cartalith-terrain` the
guard went into the variadic `js_hypot_n`, so the three-argument form
`renderHeightTileRGBA` uses is covered by the same code. Each crate gained a
`js_hypot_follows_the_spec_on_infinity_and_nan` test whose expectations are
`node`'s output, so a future copy-paste cannot lose the preamble silently
again.

All seven `js_hypot`-family entry points then agreed with V8 on infinity and
NaN, but through **five distinct implementations** of the compensated sum
(`-assets::manual`, `-civ`, `-spatial::paint`, `-terrain::sculpt::js_hypot_n`,
`-urban::geom`) plus two thin wrappers (`-terrain::sculpt::js_hypot`,
`-terrain::tile_render::js_hypot3`). `js_hypot3` is worth naming: the sweep's §5
count of "seven copies" included it, but §3.2's four-way table did not, and it
would have been missed by a fix applied only to the two-argument forms.

**Second follow-up pass: resolved.** There is now **one** compensated sum,
`cartalith-jsmath::js_hypot_n`, with `js_hypot` and `js_hypot3` as wrappers over
it, so the preamble cannot be lost from one entry point and kept in another. The
one behavioural difference in the move: `js_hypot_n` takes its arguments with
their signs and `abs`es them itself, where `cartalith-terrain`'s form required
pre-`abs`ed magnitudes. Its callers still pass magnitudes and `abs` of a
magnitude is the identity, so nothing moved. The five identical
`js_hypot_follows_the_spec_on_infinity_and_nan` tests collapsed into one.

### 3.3 `js_min`/`js_max` — signed zero → **resolved, V8's answer pinned**

`Math.min(+0, -0)` is `-0` and `Math.max(+0, -0)` is `+0`, in **either**
argument order — for these two functions alone, `-0` counts as strictly smaller
than `+0`. A plain `<`
cannot see that, because `-0.0 < 0.0` is false — so which order a copy got right
depended purely on how it was spelled. `cartalith-urban`/`-civ`'s
`if b < a { b } else { a }` answered `min(-0, +0)` correctly and `min(+0, -0)`
wrongly; `cartalith-terrain::amplify`'s `if a < b { a } else { b }` did the
reverse. All three were wrong in one direction.

**Second follow-up pass: resolved.** The single implementation adds a four-line
both-zeros arm and is V8's in both orders. Still unobservable in this engine — no
live site reads the sign of a zero — but an implementation that is right in one
argument order and wrong in the other is not a thing to keep once there is only
one of it. `js_min_max_pick_the_v8_signed_zero_in_either_argument_order` pins
eight cases read off `node` with `Object.is(x, -0)`, and asserts both superseded
forms failing the order each got wrong, so the rows cannot quietly stop
discriminating.

### 3.4 `toFixed` — two implementations, and one was wrong

`cartalith-civ::js_fixed` (returns a `String`) and
`cartalith-spatial::geo::js_to_fixed` (returns the `+`-coerced number) are
independent ports of the same conversion. `js_fixed` is correct — 0
disagreements in 60 000 differential cases. `js_to_fixed` was not; see §2.2.
They now agree.

Note that `js_fixed` was itself rewritten once, after milestone 6 found its
earlier form fabricating ties. Two independent ports of `toFixed`, two
different bugs, both found late. It is the most error-prone conversion in this
catalogue.

**Second follow-up pass: both moved, and they now check each other.** They are
kept as two functions because they return different things — `js_fixed` the
string `toFixed` produces, `js_to_fixed` the number `+x.toFixed(d)` coerces to —
but they live in one file, and
`the_two_to_fixed_ports_agree_with_each_other_everywhere` runs them against each
other over 200 000 cases across five bands (including one built specifically to
land on and beside decimal ties, which is the branch both bugs lived in), plus
every named row from §2.2 and milestone 6's fabricated tie. Either bug would have
been caught on the first run of that test.

---

## 4. Every site reviewed, and the verdict on it

Counts are live code, excluding doc comments and test bodies.

### 4.1 `f64::hypot` — 44 sites

| verdict | sites |
|---|---|
| **fixed** | 1 (`-spatial::paint`, §2.1) |
| **safe, provable invariant** | 6 |
| **safe, threshold is continuous** | 3 |
| **1-ulp value divergence, absorbed by an `f32` store** | ~20 |
| **1-ulp value divergence, survives in `f64` — reported, not changed** | ~14 |

**Safe with a hard invariant — do not "fix" these.** The D8 neighbour-distance
tables at `-erosion:292`, `-gpu:4090`, `-hydrology:78` and `-hydrology:196`
build `hypot(dx, dy)` for `dx, dy ∈ {-1, 0, 1}`. All nine values are
**bit-identical** between `f64::hypot` and V8 (verified exhaustively): the
Kahan sum is exact on `0`, `1` and `2`, so both return exactly `1` and the
correctly-rounded `√2`. Also safe: `cartalith-spatial::paint`'s own new
exhaustive test, and `-civ::tools:1196`, which is a test helper.

**Safe because the threshold is continuous.** `-terrain:1418`
(`<= rad_prov`), `-terrain:1464` (`>= sp`) and `-terrain:1714`
(`< (br + rad_cells) * 0.8`) compare a distance against a value drawn from
`rng.next_f64()`. A flip needs an exact tie against a continuously-distributed
threshold. Worth naming that these are *structurally* dangerous even so:
`-terrain:1418` sets `cand.len()`, and `cand` is then Fisher-Yates shuffled, so
one changed candidate shifts the entire downstream RNG stream. Low probability,
total consequence.

**Reported, not changed.** `cartalith-civ` milestone D already recorded the
policy this audit endorses: its route-geometry chain uses `js_hypot`, and its
slope-gradient and Journey-Planner sites (`-civ:106, 3863, 4178, 8483, 8546,
9128, 9143, 9542` and `-civ::tools:537`) were "deliberately left alone: covered
by their own passing golden tests, and changing them here would be an
unmeasured edit to verified code". The same applies to `-climate`'s eight
wind/gradient magnitudes, `-terrain`'s remaining ten, `-erosion:176`, and
`-hydrology:233`/`:459`. Most store through `f32` (see §4.2's arithmetic).

**Fork territory, audited not touched.** `-godot::render:964, 1127, 1134, 1174,
1609` and `-godot::pack:126`. `pack:126` and `render:1127` are the same
`slope_at` expression as `-civ:106`, triplicated. None is a ported-from-JS
threshold; all feed shading. No action recommended beyond noting the
triplication.

### 4.2 `f64::exp` — 23 sites

None fixed. The measured reason:

`cartalith-civ::build_npp` computes `3000 / (1 + exp(1.315 - 0.119 * t))` and
stores through `f32`. Over **10 million** temperature samples spanning
−40 °C…60 °C at 10⁻⁵ steps, swapping `f64::exp` for `js_exp` changed the stored
`f32` **zero times**. An `f32` store has ~6 × 10⁻⁸ relative precision and the
divergence is ~2.2 × 10⁻¹⁶ relative, so a visible hit needs the `f64` value to
sit within one part in 2.7 × 10⁸ of an `f32` rounding boundary. That is not
*never* — over a 2048² map it works out to roughly one cell every few hundred
generations — but it is far below the level at which an unmeasured edit to
golden-verified code is the right move.

Sites, all in this category: `-civ:134, 136, 228, 756, 786, 787, 1076, 2629,
2905, 4858`; `-climate:186`; `-terrain:1078, 2027, 2225, 2226` and
`-terrain::sculpt:1407, 1571`; `-gpu:2467` (a GPU path, held to the
principled-equivalence bar per the 2026-08-16 owner note, so out of scope by
definition); `-godot::render:1367, 1369, 1373, 1376, 1934` (fork territory,
shading).

**The exception worth flagging:** `-civ:4858`'s `road_prob` and `-civ:2629`/
`:2905`'s logistic scores feed comparisons and accept/reject gates rather than
an `f32` raster. They are the `exp` sites most likely to become the next
divergence #2 — check them first if a civ fixture ever disagrees.

### 4.3 `.min()`/`.max()` on floats — 206 sites

**No live divergence found.** The reasons, in order of how much of the surface
each covers:

1. **`.clamp()` is already JS-faithful** (§1.2), and the workspace
   overwhelmingly uses it. The dangerous hand-written `lo.max(hi.min(x))` idiom
   appears at only **two** live sites (`-civ:5536`, `-terrain:1405`), neither
   reachable by NaN.
2. **The NaN sources were already guarded** — at the source, not at the clamp.
   `-civ:1052` sets `flow_max = if raw > 0.0 { raw } else { 1.0 }`, so
   `ln(1 + f) / ln(1 + flow_max * 0.05)` can never be `0/0` on a map with no
   river flow. `-civ:2467`/`:2735` make `slope_max` the constant `4.0`.
   `-climate`'s latitude denominators are `(wh.max(1) as f64 - 1.0).max(1.0)`.
   `cartalith-civ` already carries `js_num_or_zero`/`js_truthy_num` for
   divergence #4 on the faction-aggregate path.
3. **The remainder are one-sided clamps on values that cannot be NaN**:
   `.max(0.0)` (69), `.min(1.0)` (47), `.max(1.0)` (33), applied to grid
   dimensions, RNG draws and `f32` field reads.

**Believed safe, cannot prove — the honest list.** These absorb a NaN where JS
would propagate it, and the argument for safety is "the field never contains a
NaN", which is a whole-pipeline property no test asserts:

- `cartalith-erosion:206` — `(speed² + (−dh)·g·grav).max(0.0).sqrt()`, the
  reference's `Math.sqrt(Math.max(0, …))`. A NaN in the height field would
  surface as `0`, not as a NaN.
- `-erosion:498, 662, 679, 687, 743`; `-civ:1068, 2482, 2617, 2741, 9399,
  10078`; `-climate:497` — the same shape.
- `-civ:1051` — `fold(0.0, |m, v| m.max(v))` as a running maximum. Rust's
  `f64::max` absorbs a NaN element, and so does the reference's own `if(v>mx)`
  loop, so this one actually matches. Listed because it *looks* like a hit and
  the next reader will check it otherwise.

**Recommendation:** rather than sprinkling `js_min`/`js_max` across 200 sites,
add one debug-only assertion that the pipeline's output fields are NaN-free.
That converts the whole class from "believed safe" to "checked", at one site.

### 4.4 `f64::atan2` — 8 sites, swept

`Math.atan2` disagrees with V8 more than any other function here. `js_atan2`
now exists in `cartalith-jsmath`, reachable from every crate, and every site has
a verdict.

| site | verdict |
|---|---|
| `-hydrology:241, 270, 271` (`build_channels`) | **fixed** — §2.3 |
| `-terrain:1864, 1865` (`poly_meta`) | **safe, proved** |
| `-civ::labels:517` (`label_rotate_deg`) | **safe, no reproducible reference** |
| `-terrain:372` (plate circular mean) | **fixed** — §2.5, with `js_sin`/`js_cos` |
| `-urban::graph:607` (`ang`) | **fixed** — §2.4 |

**All eight sites are now closed** — five fixed (`build_channels`'s three, plus
`-terrain:372` and `-urban::graph:607`) and three safe with a stated invariant
(`poly_meta`'s two and `label_rotate_deg`). Nothing in this table is "reported,
not changed" any more.

**`-terrain:1864, 1865` — safe with a hard invariant, do not "fix" this one.**
`poly_meta`'s turning angle takes `atan2` of differences between consecutive
polyline points, and those points come from `walk()`, which advances one
8-connected cell at a time. The arguments are therefore always in
`{-1, 0, 1}²`, and **all eight D8 directions are bit-identical between
`f64::atan2` and V8** (verified directly, not assumed — the same shape as the
D8 `hypot` tables in §4.1). Belt and braces: `curvature`, the only value the
turning angle feeds, has no consumer anywhere in the workspace outside one
`assert_eq!` in a terrain unit test.

**`-civ::labels:517` — safe, but for a different reason worth stating.**
`label_rotate_deg` converts a live pointer position into a label's rotation in
degrees. Its input is a mouse drag, not a seeded pipeline value, so there is no
reproducible reference for it to diverge *from*; two runs of the reference
itself would not agree either. The output is continuous, feeds a canvas
rotation, and crosses no threshold. Cosmetic, as the sweep first called it, and
now with the invariant attached.

**`-terrain:372` — the reason it was left, and what closing it took.**
This is `buildPlates`'s circular mean of a plate's member-cell x positions in
world-wrap mode: `atan2(Σ sin θ, Σ cos θ)`, scaled by `gw`. **It cannot be made
V8-faithful by porting `js_atan2`, because the divergence enters upstream of
`atan2`.** Measured over 2 000 synthetic plates at `gw = 512`: Rust's `sin`/
`cos` produce a **different `(Σ sin, Σ cos)` pair from V8's on 92 of them**,
before `atan2` is ever called. Swapping in `js_atan2` alone moves the final
`plate.x` from 98/2000 disagreeing to 7/2000 — an improvement that leaves the
site *differently* wrong, which is worse than leaving it alone, since it would
make the next reader believe the site had been handled.

Its downstream is worth recording precisely, because it is not uniformly safe:

* `-terrain:385` quantises through `js_round(plate.x).clamp(0, gw-1)`. Over the
  same 2 000 plates the resulting **cell index differs from V8 zero times**.
  Safe.
* `-terrain:347` does **not** quantise — `dx = x as f64 - plate.x` feeds the
  next Lloyd relaxation iteration's nearest-plate argmin. That is structurally
  the same hazard as §2.3, one iteration removed.

So: fix this in the same pass that lands `js_sin`/`js_cos`, not before, and fix
all three together.

**Done — §2.5.** All three are `cartalith-jsmath`'s, all three moved together,
and the site now matches V8 on 2 000 of 2 000 synthetic plates across two seeds.
The re-measurement with a larger member count per plate is worth recording
beside the original figures: 737/2000 of the `(Σ sin, Σ cos)` pairs differ,
193/2000 of the final `plate.x` values do with Rust's own libm, and **110/2000
still do with `js_atan2` alone** — the partial fix is a real improvement that
leaves the site wrong on one plate in eighteen, which is exactly why it was
refused.

**`-urban::graph:607` — fixed, §2.4.** `ang` is `atan2` of an edge's endpoint
delta, stored on a half-edge and used as the **sort key** for the half-edges
around a node, which is what the face traversal walks. `sort_by` is stable, so
an exact tie keeps insertion order — but a one-ulp difference reorders two edges
leaving a node in nearly the same direction, and the face traversal then produces
a different city block. That is the §2.3 argmax hazard in a different costume,
and this document's #1 history said it would be found the hard way if it were not
found deliberately.

The follow-up pass audited it and left it, because the urban fork was live in
that crate. That fork has landed, and the fix went in with the consolidation.
Measured on the coordinates the graph really produces: `f64::atan2` and V8
disagree on **38 %** of edge deltas and put a near-parallel pair in a
**different order 4.7 %** of the time. All 20 golden scenarios pass unmodified,
and the ordering itself is pinned against `node` by a new test.

### 4.5 `f64::round` — 47 sites

`Math.round` is half-up (toward +∞); `f64::round` is half-away-from-zero. They
differ **only on exact negative halves**. Every site was checked for a negative
argument:

- **`cartalith-assets::raster:249, 250`** (`dx`/`dy` from `pan_x`/`pan_y`) is
  the only site whose argument is genuinely, routinely negative — and it is
  **not** a divergence: the reference has no `Math.round` there at all.
  `drawItemOnly` passes floats straight to `ctx.drawImage`, and this port
  rounds only because `image::imageops::overlay` takes integers. Already
  documented in that module as a non-reference-identical resampling step.
- `-civ:8478, 8479` and `-gpu:2176` can see a negative, and each is immediately
  `.clamp(0, …)`ed, where `-1` and `0` land on the same answer. Safe.
- Every other site rounds a width, radius, count, population, distance or
  percentage — non-negative by construction.

`-godot::params:383` (`v = v.round()`, a parameter snap) and `-godot::render:1219,
1220` (jitter offsets, which can go negative near an edge) are fork territory.
Reported: `render:1219/1220` is the one place in the workspace where an
unexamined negative `.round()` could bite.

### 4.6 Float → integer, and `as u8` — 26 sites

**`as u8` truncates**; a `Uint8ClampedArray` store rounds ties to even.
`cartalith-terrain::tile_render::u8_clamped` is the correct conversion, and its
module doc already records that a naive `as u8` costs a whole level.

- `-terrain::tile_render` uses `u8_clamped` correctly throughout. No action.
- `-godot::lib:1408–1410, 1477, 1751` and `-godot::pack:211–213` truncate — but
  none is a `Uint8ClampedArray` port. They are this port's own display code
  (`border_cover`, the faction wash, the plate frame), with no reference to
  match, and `pack.rs` already applies `.round()` first. Fork territory; no
  action recommended.
- `-gpu:3140, 3428` are debug dumps.
- `-spatial::paint:330` and `-terrain:1943, 2017` cast values that are already
  integral.

**Float → int casts generally.** Rust `as i64` truncates toward zero and
saturates; `Math.floor` goes toward −∞ and JS `|0` wraps modulo 2³². Every
negative-capable site the sweep found is protected by an immediately following
`.max(0)` or `.clamp(0, …)`, where truncation and floor agree
(`-terrain:1623–1625`, `-hydrology:444, 445`, `-civ:8478, 8479`). Reviewed as a
class; no live divergence.

---

## 5. Recommendations

**1. ~~Port `js_atan2`, and re-verify `build_channels` after.~~ DONE
(2026-08-18).** §2.3. It was a live bug, not a latent one: the argmax was
picking the wrong cell. **What is left of this recommendation** is
`-urban::graph:607`, the half-edge sort key (§4.4), which belongs to the urban
fork beside the `js_sin`/`js_cos`/`js_log` it has already added; and
`-terrain:372`, which needs `js_sin`/`js_cos` before `js_atan2` can help it.

**2. ~~Consolidate the helpers into a `cartalith-jsmath` leaf crate.~~ DONE
(2026-08-18).** There were 7 copies of `js_hypot`, 7 of `js_round`, 3 of
`js_min`/`js_max`, 2 of `toFixed`, and one each of
`js_exp`/`js_sin`/`js_cos`/`js_log` that nothing outside `cartalith-urban` could
reach. §3 showed the copies had already drifted apart in three measurable ways,
and §2.2 showed that when two independent ports of one conversion exist, one of
them is wrong.

`ARCHITECTURE.md` says dependencies run one way, in pipeline order. A crate
with *no* dependencies, sitting below `cartalith-noise` and `cartalith-rng`,
does not disturb that ordering, and it is the only shape that reaches all
fifteen: `cartalith-urban` depends only on `-rng`, and `cartalith-assets` only
on `-io`/`-noise`, so neither can see `cartalith-spatial`. `cartalith-jsmath`
was created with an empty `[dependencies]` and **no dev-dependency either** —
its bulk goldens carry a four-line inline `mulberry32` rather than borrowing
`cartalith-rng`'s, so the leaf property is a fact about the manifest rather than
a convention.

**The block was `cartalith-urban` being mid-edit**, with 607 uncommitted lines
in `geom.rs`, `js_sin`/`js_cos`/`js_log` freshly added, and its own `routes`
golden red. That fork landed as `6d242cf`, and the consolidation followed.

**What actually moved.** Every helper this document names, plus `js_atan`, which
was private inside `cartalith-hydrology` and is public now because `Math.atan`
is a JS function in its own right. Where a module path was load-bearing —
`geom::js_hypot`, `sculpt::js_hypot`, `tile_render::u8_clamped`,
`spatial::geo::js_to_fixed` — it survives as a `pub use` re-export, so **no call
site had to change**. The `cartalith-urban` tests moved with the code unchanged,
including the bulk FNV-1a goldens over 54 000 `sin`, 54 000 `cos` and 30 000
`log` results, and they passed on the first run in their new home, which is the
check that the move was pure.

**What deliberately did not move.** `cartalith-spatial::paint`'s exhaustive rim-
cell scans, `cartalith-assets`' `js_round_is_half_up_which_matters_at_the_left_edge`,
and `cartalith-civ`'s NaN-absorption tests all stayed where they are: they test a
*call site's* behaviour, not the helper's, which is the line this consolidation
drew. `cartalith-assets::scatter::js_number`, `-assets::manifest::js_parse_float`,
`-io::tiles::js_num` and `-urban::site::js_or` also stayed — they are JS
*coercions* over crate-specific types (`serde_json::Value`, `Option<f64>`,
strings), not floating-point semantics, and pulling them into a math crate would
have been the wrong boundary.

**Not done, deliberately: `js_acos` and `js_log10`.** `cartalith-urban`
milestones 10 and 15 are known to need them. Adding them now would be two
hundred lines of FDLIBM with no caller and no golden — exactly the dead code
this project avoids. The crate is where they go when a milestone needs them.

**The follow-up pass made the case stronger, and paid the cost it predicted.**
`js_atan2` went into `cartalith-hydrology` as a private module — an **eighth**
copy site for the FDLIBM family — because that is where the live bug was and
because opening a cross-crate refactor under an active sibling is the one thing
this recommendation says not to do. That is the right call for one pass and the
wrong steady state: `-terrain:372` and `-urban::graph:607` both still need
`atan2`, and neither can reach `cartalith-hydrology`'s private module, so the
next porter faces a ninth and tenth copy or a dependency edge that
`ARCHITECTURE.md` would not want. When the crate is created, `js_atan2` and its
`js_atan` should move into it with the rest — the tests move with them
unchanged, which is the check that the move was pure.

**3. Assert NaN-freedom once instead of clamping 200 times.** §4.3.

**4. Fixtures must exercise branches, not just features.** Both bugs in §2
survived because the fixtures were chosen for *coverage of feature types* and
happened to use round numbers. `golden_parity_geojson.rs` calls `js_to_fixed`
on every feature it exports, on a grid of exactly 50 km per cell, so it never
once rounded a fractional coordinate. When porting a numeric conversion, pick
fixture inputs that hit each rounding branch — including a negative one.

**5. Read the reference by running it.** The `-0.062` expectation in §2.2 was
written from a paraphrase of ECMA-262 and asserted the bug for two milestones.
`node -e "(-0.0625).toFixed(3)"` takes a second and is never wrong.

---

## 6. Verification

- `cargo test --workspace --exclude cartalith-godot`: **1131 passed, 0 failed**
  across 96 test binaries. The baseline immediately before this audit was
  **1128 passed, 0 failed** across 96; the delta is the three tests added here.
  (`cartalith-godot` is excluded because a running Godot editor holds
  `cartalith_godot.dll` — the documented transient, not a defect.)
- `cargo clippy -p cartalith-spatial --all-targets`: clean, zero warnings.
- **No existing golden expectation was modified.** The one changed expectation
  is a `cartalith-spatial` unit-test assertion that encoded bug B of §2.2, and
  its replacement is V8's own output.
- Both fixes were confirmed to fail before and pass after, by reverting the
  one-line change and re-running the new test.
- `cargo fmt` was not run.

### 6.1 The follow-up pass (`js_atan2`, same day)

- `cargo test --workspace --exclude cartalith-godot --exclude cartalith-urban`:
  **1069 passed / 0 failed** across 94 suites, against a **1062 / 0** baseline
  taken by reverting exactly the five files this pass touched and re-running.
  The delta is **exactly the seven tests added** and nothing else moved.
- `cartalith-urban` is excluded from that pair because the sibling fork was
  editing it *during* the run — its own uncommitted `routes`/`geom`/`rng`
  goldens went from 1 red to 6 red between the baseline and the after-run,
  none of them in a crate this pass touched. Including it
  (`--exclude cartalith-godot` only, `--no-fail-fast`) gives 1130 → 1132 with
  that moving failure count, which is why the clean comparison excludes it.
  `cartalith-godot` is excluded for the documented DLL-lock transient.
- `cargo clippy -p cartalith-hydrology -p cartalith-assets -p cartalith-civ
  -p cartalith-terrain --all-targets`: **no warning or error in any line this
  pass wrote.** (The four crates are not warning-free overall — pre-existing
  `excessive_precision` in `golden_parity_road_network.rs` and two loop-index
  warnings in `cartalith-civ` — but none is in changed code.)
- **No existing golden expectation was modified**, including all three cases of
  `golden_parity_river.rs`, which pass unmodified precisely *because* they
  cannot observe the bug (§2.3).
- The `build_channels` fix was confirmed to fail before and pass after by
  reverting the three call sites and re-running: receiver `8` before, `6`
  after, and `6` from `node`.
- `cargo fmt` was not run. Nothing Godot-scene-side was touched (UI hold,
  `DCC_SHELL_SCOPE.md`). `cartalith-urban` was read and reported on, never
  edited.

### 6.2 The consolidation pass (`cartalith-jsmath`, same day)

- `cargo test --workspace --exclude cartalith-godot`: **1134 passed / 0 failed**
  across 99 suites, against an immediately-preceding baseline of **1138 / 0**
  across 96. The delta of **-4** is fully accounted for and nothing else moved:
  8 tests **moved** out of `-urban`/`-hydrology` into `cartalith-jsmath`; 15
  **duplicates** were deleted, because five crates had grown identical copies of
  the same three helper tests; `cartalith-jsmath` gained those 8 plus **8 new**
  ones; and **3 new** tests landed at the two fixed call sites. `1138 - 8 - 15 +
  16 + 3 = 1134`. The three new suites are `cartalith-jsmath`'s unit tests, its
  (empty) doc-tests, and `golden_parity_plate_circular_mean.rs`.
- **No existing golden expectation was modified anywhere.** Not one. That
  includes all 20 `cartalith-urban` graph scenarios and both `world`-mode cases
  of `golden_parity_plates.rs`, which is the proof that §2.4's and §2.5's fixes
  moved nothing those fixtures can see.
- `cargo clippy --all-targets` over all nine touched crates
  (`-jsmath -urban -hydrology -terrain -civ -spatial -assets -engine -climate`):
  **no warning in any line this pass wrote.** The warnings that remain are the
  ones §6.1 already named — `excessive_precision` in
  `golden_parity_road_network.rs`, two loop-index warnings and a `matches!`
  suggestion in `cartalith-civ`, and `cartalith-gpu`'s two dead-code
  warnings — none in changed code. `cartalith-jsmath` itself is warning-free.
- **Mutation-tested: 440 mutants, 258 killed, 182 survived, 0 broken.** See
  §6.3.
- `cargo fmt` was not run. Nothing Godot-scene-side was touched (UI hold,
  2026-08-18).


### 6.3 The mutation sweep, and what it found

Run over the whole non-test body of both `cartalith-jsmath` files, each mutant
in its own `cargo test` with a **private `CARGO_TARGET_DIR`** — `cartalith-urban`
milestone 6 found two runners sharing one target directory silently leaving a
live mutation in the source. Snapshot before writing, restore after, a
post-sweep byte-comparison of both files, and a post-sweep baseline: **GREEN**.

**440 mutants, 258 killed, 182 survived, 0 broken.** Twenty survivors re-run in
isolation: **20 of 20 reproduce, zero false survivors.**

| count | survivor class | the invariant it rests on |
|---|---|---|
| 56 | a FDLIBM constant moved by **one ulp** | it reaches the result scaled by `z^k` and is 10⁻⁵–10⁻³ of the result's own ulp |
| 55 | a comparison flipped on its boundary | the forms differ only when the operands are exactly equal, and both arms then return the same double |
| 36 | a reduction threshold bumped one representable step | fires only for the single argument whose high word *is* the threshold |
| 24 | a guard dropped | Rust's saturating float→int casts and always-false NaN comparisons make it redundant |
| 11 | a slot or shift inside `rem_pio2`'s third correction round, or the Payne-Hanek hand-off | branches no angle this engine produces can reach |

**Three findings, in descending order of how far they generalise.**

**1. A mutation operator can manufacture its own survivors.** The first round
reported dozens that were artefacts twice over: it was mutating inside `//`
comments (`// atan(1.5)hi` → `// atan(1.6)hi` is not a mutation), and it was
bumping a float constant's **last written decimal digit** — but FDLIBM writes
its constants to 21 significant figures, three past what an `f64` holds, so the
"mutant" parsed to the same double. Both fixed: mutate only the code half of a
line, and perturb a float by one ulp through its bit pattern. A survivor is
evidence only if the mutant is genuinely a different program.

**2. Recommendation #4 again, from the other direction.** After that fix the
sweep left **206** alive and **101 of them were inside `js_exp` and
`js_atan`/`js_atan2`** — precisely the two functions that arrived before the
bulk-hash technique existed and still had only a dozen hand-picked rows each.
This is `cartalith-urban` milestone 6's "63 survivors" result reproduced exactly,
on different functions, two milestones later. Both got the golden they were
missing: an FNV-1a hash over 48,000 `Math.exp` results across eight bands (the
`2^-28` shortcut, inside `0.5 ln2`, across both reduction forms, the ordinary
range, both overflow and underflow thresholds, the subnormal rescale, and a band
hugging `1.0` where V8 and FDLIBM genuinely disagree) and one over 54,000
`Math.atan2` results across nine `|y/x|` bands — one per fdlibm `atan` reduction
interval, plus both `2^60` magnitude shortcuts including the `m &= 1` branch —
each draw also picking a scale and one of the four sign quadrants. **Both
matched V8 on the first run**, and the survivor count fell to 182.

**3. It found four real gaps in this pass's own new tests, and one real bug.**
Both `toFixed` ports' non-finite guards were untested — dropping either leaves a
`.expect()` that panics on an infinity, and nothing noticed, because
`js_to_fixed_passes_non_finite_through` had been dropped as a duplicate when it
was not one. Every value in the `u8_clamped` table had an **even floor**, so
inverting its round-down comparison fell through to the ties-to-even arm and
produced the same answer; `1.2` and `3.1` kill it. And writing the non-finite
test exposed a **real divergence** in code that had been sitting in
`cartalith-civ` unnoticed: `js_fixed` returned Rust's `inf`/`-inf` where
`Number.prototype.toFixed` gives `"Infinity"`/`"-Infinity"`. Unreachable from
any live caller, and fixed anyway — a JS-semantics helper that does not have JS's
semantics is the thing this crate exists to stop.

Measurement harnesses (the V8 dumps, the exhaustive scans, the differential
runs, the 3x3 block search, the near-parallel edge-delta search, the mutation
runner) were scratch programs, not checked in — the same convention the
golden-fixture extraction harnesses follow. Every number in this document is
reproducible from the description beside it.
