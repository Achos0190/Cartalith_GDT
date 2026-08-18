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

---

## 1. The catalogue

Nine known divergences. The first five were found by earlier milestones; #6–#8
were being found by the `cartalith-urban` fork while this sweep ran; #9 is this
sweep's own.

| # | What differs | Where the JS-faithful helper lives |
|---|---|---|
| 1 | `Math.hypot` ≠ `f64::hypot` | `cartalith-urban::geom::js_hypot` (canonical), plus 4 copies |
| 2 | `Math.exp` ≠ `f64::exp` | `cartalith-urban::geom::js_exp` (only copy) |
| 3 | `Math.min`/`Math.max` propagate NaN; Rust's absorb it | `js_min`/`js_max` in `-urban::geom`, `-civ`, `-terrain::amplify` |
| 4 | `NaN` is falsy in JS (`p.pop||0`, `maxPop ? … : 0`) | `js_num_or_zero`/`js_truthy_num` (`cartalith-civ`) |
| 5 | Rounding modes: `Math.round`, `toFixed`, `Uint8ClampedArray` | `js_round` (6 crates), `js_fixed` (`-civ`), `js_to_fixed` (`-spatial::geo`), `u8_clamped` (`-terrain::tile_render`) |
| 6 | `Math.sin` ≠ `f64::sin` | `cartalith-urban::geom::js_sin` (in flight, fork) |
| 7 | `Math.cos` ≠ `f64::cos` | `cartalith-urban::geom::js_cos` (in flight, fork) |
| 8 | `Math.log` ≠ `f64::ln` | `cartalith-urban::geom::js_log` (in flight, fork) |
| 9 | **`Math.atan2` ≠ `f64::atan2`** | **none — nothing has been ported for it** |

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

Every disagreement is one ulp. That is not a reason to relax: divergence #1's
own history is a one-ulp `hypot` turning a four-node road graph into a
three-node one, because 11 m was a snap threshold.

`atan2` is the headline. It is the *largest* divergence in the workspace, it
has eight live call sites, and no `js_atan2` exists. See §4.4.

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

## 2. What this sweep changed

Two real bugs, both in `cartalith-spatial`, both proved with a test that fails
before the fix and passes after, and both with their expected values re-derived
from V8 rather than read off the new code.

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

---

## 3. Do the helpers agree with each other?

**No.** They were written independently in five crates, and there are three
real disagreements — all measured, none currently live.

### 3.1 `js_round` — one input, six crates

`cartalith-urban::geom::js_round` compares the fractional part. The other six
(`-assets::manual`, `-civ`, `-climate`, `-engine`, `-spatial::region`,
`-terrain`) all use `(x + 0.5).floor()`.

A sweep of 3 million random values plus every double within 3 ulp of every
half-integer in ±50 finds **exactly one** disagreeing input:

```
x = 0.49999999999999994        (the largest double below 0.5)
  (x + 0.5).floor()  = 1       because x + 0.5 rounds up to exactly 1.0
  V8 Math.round(x)   = 0
```

`cartalith-terrain`'s doc comment calls `(x + 0.5).floor()` "the standard exact
equivalent". It is not, and the urban copy's own doc says so. **Not fixed:**
one unreachable input against six cross-crate edits is the wrong trade while
three forks are active. **Do** use the fractional-part form when writing a new
one.

### 3.2 `js_hypot` — NaN and infinity

`cartalith-urban::geom::js_hypot` has a specification preamble; the copies in
`-assets::manual`, `-civ` and `-terrain::sculpt` do not:

| | V8 | urban | the other three |
|---|---|---|---|
| `hypot(NaN, 0)` | NaN | NaN | **0** |
| `hypot(∞, 3)` | ∞ | ∞ | **NaN** |
| `hypot(∞, NaN)` | ∞ | ∞ | **NaN** |

`hypot(NaN, 0)` → `0` is argument-order dependent: `hypot(0, NaN)` gives NaN in
all four. No live site can reach a NaN or infinite argument, so this is
recorded, not fixed. The new copy added in §2.1 has the preamble.

### 3.3 `js_min`/`js_max` — signed zero

`Math.min(+0, -0)` is `-0` and `Math.max(+0, -0)` is `+0`.
`cartalith-terrain::amplify` gets `min` right where `-urban`/`-civ` get it
wrong; all three get `max` wrong in one of the two argument orders.
Unobservable in this engine (a signed zero's sign is never read), and already
documented in the urban copy. Recorded.

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

### 4.4 `f64::atan2` — 8 sites, the largest unswept divergence

Nothing has been ported for `Math.atan2`, and it disagrees with V8 more than
any other function here (**22.98 %**).

The structural one is **`cartalith-hydrology::build_channels`** (lines 241,
270, 271):

```rust
let aspect = (-gy).atan2(-gx);
let mut da = (dy as f64).atan2(dx as f64) - aspect;
da = da.sin().atan2(da.cos()).abs();
let score = drop * (0.5 + 0.5 * da.cos());
if score > best_score { best_score = score; best = j; }
```

`best` is the cell a river flows into — a discrete argmax, not a shaded value.
Measured against V8 over 100 000 random aspects:

- `atan2(dy, dx)` for the eight D8 directions: **0 / 8 differ** (those are
  exact, so the divergence enters only through `aspect` and the `sin`/`cos`/
  `atan2` wrap chain);
- the aspect factor `0.5 + 0.5·cos(da)` as a whole:
  **12 969 / 100 000 — 12.97 % differ.**

So one in eight of this function's steering weights is one ulp off V8. The
argmax flips only when two neighbours' scores land within one ulp of each
other, which is rare — but when it does, the river takes a different cell and
everything downstream of it moves. This is the same shape as divergence #1's
node-graph failure, one probability class quieter.

Other sites: `-civ::labels:517` (a label bearing in degrees — cosmetic),
`-terrain:372` (a plate seed position, then scaled by `gw` — value only),
`-terrain:1864, 1865` (a turn angle inside a length accumulation), and
`-urban::graph:607` (fork territory; `ang` is stored on an edge, and since the
urban fork is currently porting `js_sin`/`js_cos`/`js_log`, that is the natural
place for a `js_atan2` to land).

**Recommendation:** port `js_atan2` next, into the urban fork's FDLIBM block
where `js_sin`/`js_cos`/`js_log` already are, rather than opening a seventh
copy site. Then re-verify `build_channels` specifically.

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

**1. Port `js_atan2`, and re-verify `build_channels` after.** §4.4. It is the
biggest measured gap in the workspace and it sits on a discrete argmax.

**2. Consolidate the helpers into a `cartalith-jsmath` leaf crate — later, not
now.** There are 7 copies of `js_hypot`, 7 of `js_round`, 3 of
`js_min`/`js_max`, 2 of `toFixed`, and one each of
`js_exp`/`js_sin`/`js_cos`/`js_log` that nothing outside `cartalith-urban` can
reach. §3 shows the copies have already drifted apart in three measurable ways,
and §2.2 shows that when two independent ports of one conversion exist, one of
them is wrong.

`ARCHITECTURE.md` says dependencies run one way, in pipeline order. A crate
with *no* dependencies, sitting below `cartalith-noise` and `cartalith-rng`,
does not disturb that ordering, and it is the only shape that reaches all
fourteen: `cartalith-urban` depends only on `-rng`, and `cartalith-assets` only
on `-io`/`-noise`, so neither can see `cartalith-spatial`.

**Do not do it yet.** `cartalith-urban::geom` is the file the urban fork is
actively adding `js_sin`/`js_cos`/`js_log` to; moving it now guarantees a
collision. The move is mechanical and safe once the three forks have landed,
and it should be one commit that only relocates code — no behaviour change, so
every golden must pass untouched.

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

Measurement harnesses (the V8 dumps, the exhaustive scans, the differential
runs) were scratch programs, not checked in — the same convention the
golden-fixture extraction harnesses follow. Every number in this document is
reproducible from the description beside it.
