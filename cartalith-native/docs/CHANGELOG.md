# Changelog

One entry per milestone: what was ported or built, how it was verified, and
what is still open (`README.md`'s working discipline).

## Phase 0 — Walking skeleton (2026-08-11)

No engine logic. Proves the toolchain end to end before porting any formula
(`ROADMAP.md`, `TOOLCHAIN.md`).

**Built:**

- Cargo workspace at `cartalith-native/` with the nine crates
  `ARCHITECTURE.md` specifies (`cartalith-noise/-rng/-terrain/-climate/
  -erosion/-hydrology/-engine/-io/-godot`). Eight are empty stubs — one
  trivial test each, no logic — matching Phase 0's "no engine logic yet."
  `cartalith-godot` is the only one that does anything: it depends on
  `gdext` (crate `godot` 0.5.5, `api-4-7` feature) and exposes a
  `WalkingSkeleton : Node` with a `ping() -> String` method.
- `godot-project/` — a minimal Godot 4.7.1 project (`project.godot`,
  `main.tscn`, `main.gd`, `cartalith.gdextension`) with a triangle, a
  button, a label, and the `WalkingSkeleton` node, per the "triangle,
  button, printed line" bar in `ROADMAP.md`.

**Verified — actually ran, not just compiled:**

- `cargo build -p cartalith-godot` (native Linux, debug): compiles.
- `godot4 --headless --import .`: **loads** the extension —
  `Initialize godot-rust (API v4.7.stable.official, runtime
  v4.7.1.stable.official, safeguards strict)` in the log.
- `godot4 --headless --quit main.tscn`: runs the scene one frame and exits
  0, with `WalkingSkeleton::ready()`'s `godot_print!` line in the output.
  This is the headless engine runtime, not the graphical editor — no
  display is available in this session, so the editor UI, the button
  click, and the on-screen triangle are unverified (`DECISIONS.md` §5).
- Windows: `x86_64-pc-windows-gnu` build (mingw-w64, `TOOLCHAIN.md`'s
  documented fallback — `cargo-xwin`'s MSVC route is covered below) →
  `godot4 --headless --export-debug "Windows Desktop" ...` and
  `--export-release` both produced a real PE32+ `Cartalith.exe` with the
  extension `.dll` alongside it. Compiles and packages only; nobody has
  run it on Windows (`DECISIONS.md` §5, `MVP_SCOPE.md` criterion 3).
- Android: **not verified.** `cargo ndk build` fails at the first step —
  no NDK is installed, and none of this session's available channels can
  install one (below). Everything short of that step is in place: Rust
  Android targets installed, the `android.*` lines in
  `cartalith.gdextension`, and an `Android` preset in
  `export_presets.cfg`.

**Toolchain installed this session:**

- Rust targets: `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`,
  `aarch64-linux-android`, `armv7-linux-androideabi`,
  `x86_64-linux-android`, `i686-linux-android`.
- `cargo-xwin` 0.23, `cargo-ndk` 4.1.2 (installed; `cargo-xwin` cannot
  complete a build here — see below).
- Godot 4.7.1-stable (latest stable 4.x as of this session, superseding
  the "verify before pinning" placeholder in `DECISIONS.md` §9) — editor
  binary and both export-template sets, from GitHub Releases.
- `gcc-mingw-w64-x86-64` / `g++-mingw-w64-x86-64` via apt, for the
  Windows-GNU fallback.

## Open / blocked

**Android SDK and NDK could not be installed in this session — two
independent official channels are both blocked by this session's network
policy, confirmed rather than assumed:**

1. `dl.google.com` (the sole official host for the SDK command-line tools
   and the NDK) — direct `curl` and the Ubuntu `google-android-*-installer`
   packages both fail identically: `Proxy tunneling failed: Forbidden`.
2. The `barichello/godot-ci` Docker image (which bundles an
   officially-sourced SDK+NDK) — the registry API itself
   (`registry-1.docker.io`) is reachable, but blob downloads redirect to a
   CDN host that the proxy also rejects with 403.

Separately: Godot 4.7.1 pins **NDK 29.0.14206865**
(`platform/android/detect.py`), well past the newest version
(`google-android-ndk-r26c-installer`) Ubuntu's apt packaging offers — so
even a working apt mirror would not have supplied the right version.
`export-pipeline.md`'s "23.2.8568313 at the time of writing" is stale;
noted there.

**This is the named Phase 0 risk landing, just not the anticipated shape.**
`TOOLCHAIN.md` and `REFERENCES.md` flagged gdext's Android support itself
as the risk (experimental, "documentation and tooling still lacking"). What
actually blocked Phase 0 here is this session's network egress policy, not
gdext — the Rust/Godot/Android integration itself is untested in either
direction. Whoever runs Phase 0 next with SDK/NDK access should expect the
*next* failure, if any, to be the one the docs anticipated.

**To finish Android Phase 0** (from a session or machine with SDK/NDK
access): install the SDK platform tools + NDK 29.0.14206865, set
`ANDROID_SDK_ROOT` and `ANDROID_NDK_HOME`, then:

```bash
cargo ndk -t arm64-v8a build -p cartalith-godot
godot4 --headless --export-debug "Android" builds/android/Cartalith.apk
```

`export_presets.cfg`'s Android preset currently builds `arm64` only
(matches real devices, per `TOOLCHAIN.md`); flip the other `arch/*` keys
on once armv7/x86 emulator coverage is wanted.

**`cargo-xwin`'s MSVC route is also blocked**, same category: its
first-run CRT/SDK fetch hits `aka.ms`, which this session's proxy also
rejects. Windows Phase 0 succeeded anyway via the documented mingw-w64
fallback (`TOOLCHAIN.md`), so this did not block the milestone — noted so
the next session doesn't re-diagnose it. `x86_64-pc-windows-gnu` produces a
working extension DLL; whether MSVC is worth pursuing later (matching
users' ABI expectations exactly, per `DECISIONS.md`) is open.

**Not started:** everything past the walking skeleton — no engine crate has
logic yet (Phase 1, `MVP_SCOPE.md`).

## Phase 0 follow-up — owner verification on real Windows (2026-08-11)

The owner opened `godot-project/` in the Godot 4.7.1 editor on their own
Windows machine: the scene rendered (triangle, label, button), which
`ROADMAP.md`'s bar asks for and this session's headless run couldn't show.

**Found a real bug in the process — `.gdextension` pointed at the wrong
Windows path.** The checked-in manifest had
`windows.debug.x86_64 = "res://../target/x86_64-pc-windows-gnu/debug/..."`,
because that is where *this session's* mingw-w64 cross-build (the
`cargo-xwin`/MSVC fallback, see above) happened to land. A native `cargo
build` on real Windows uses the host MSVC toolchain by default and needs no
`--target` flag, so cargo drops the DLL straight in `target\debug\` — no
triple subdirectory — and Godot correctly reported
`GDExtension dynamic library not found`. Fixed to
`res://../target/debug/cartalith_godot.dll` (and `.../release/...`),
matching a plain `cargo build -p cartalith-godot [--release]` on native
Windows — the path a real contributor actually hits, not the one this
session's sandbox-only cross-compile happened to produce. The `linux.*`
and `android.*` entries were already written this way; only `windows.*`
had the leftover sandbox path.

Once the owner rebuilds with the fixed manifest and re-runs the scene, the
outstanding check is clicking "Ping Rust" and confirming
`cartalith-godot: pong` appears — the actual GDExtension method-call
round-trip, still unconfirmed as of this entry.

## Phase 0 follow-up — owner tried real Android hardware (2026-08-11)

The owner opened the project folder directly in the **Godot Editor Android
app** on a real device (Adreno 750). Useful data even though it failed:

```
OpenGL API OpenGL ES 3.2 ... Compatibility - Using Device: Qualcomm - Adreno (TM) 750
ERROR: Can't open dynamic library: .../target/aarch64-linux-android/debug/libcartalith_godot.so
ERROR: Can't open GDExtension dynamic library: 'res://cartalith.gdextension'.
```

**Godot's own Android runtime works fine on real hardware** — it reached
renderer init before failing on the one thing this session's blocked NDK
never let it build: the `.so` itself doesn't exist yet. This is the exact
gap already logged above (`cargo ndk build` fails with no NDK installed),
not a new problem. Narrows the real remaining Android risk to "does the
gdext cross-compile + on-device load work," not anything about Godot's
Android integration generally.

Also notable: running a project directly from the Godot Editor Android app
is a lighter-weight on-device test than a full `.apk` export/sideload
cycle — worth using once `cargo ndk -t arm64-v8a build -p cartalith-godot`
produces the `.so` on a machine with NDK access. `MVP_SCOPE.md` criterion 4
still wants the actual `.apk` built and sideloaded, so this is a good
interim signal, not a substitute for that.

## Phase 0 close-out — real Windows + Android, on real hardware (2026-08-12)

The owner set up Rust, Godot 4.7.1, and the Android SDK/NDK directly on
their own Windows PC (mirroring `TOOLCHAIN.md`, none of which the cloud
sandbox could reach), then had this session continue there directly —
real tool access to the actual machine, not the copy-paste relay the
earlier entries describe.

**Windows: confirmed.** Rebuilt with the fixed `.gdextension` manifest,
opened a fresh editor session, clicked "Ping Rust" — `cartalith-godot:
pong` appeared. The GDExtension method-call round-trip works, closing the
one gap the previous entry left open.

**Android: confirmed further than before, one more real gap found and
fixed.**

- `cargo ndk -t arm64-v8a build -p cartalith-godot` succeeded. `file` on
  the result confirms `ELF 64-bit ... ARM aarch64 ... built by NDK r29
  (14206865)` — exactly the version Godot 4.7.1 pins
  (`platform/android/detect.py`), not a newer or older one.
- `godot4 --headless --export-debug "Android" builds/android/Cartalith.apk`
  first failed: `ETC2/ASTC texture compression is required for Android
  export`. This is a real, generically-required Godot Android export
  setting that had never been set — nothing about this project's assets
  in particular. Fixed by adding
  `textures/vram_compression/import_etc2_astc=true` to
  `godot-project/project.godot`. Re-running the export then succeeded: a
  real signed debug `.apk` was built and Godot's own verification step
  passed.

**Still open:** `MVP_SCOPE.md` criterion 4 wants the `.apk` **installed
and run** on the device, not just built. This session confirmed the build
and packaging pipeline end-to-end on real hardware — sideloading and
launching it is the next, separate step.

With this, Phase 0's two remaining checkboxes from the original walking
skeleton (`ROADMAP.md`) are both closed on real hardware: Godot loads and
runs the gdext extension, and Windows + Android both build and package.
Phase 1 (`MVP_SCOPE.md`) can start.

## Phase 1 — mulberry32 ported and golden-verified (2026-08-12)

First engine logic in `cartalith-rng`, per `PARITY_TESTING.md`'s explicit
ordering: **port the RNG first, alone, before anything depends on it.**

- Located the reference implementation in `reference/Cartalith Gen1
  v2.10.html` (the `/* ===== noise ===== */` section) and confirmed via
  call-site grep that it's the one MVP scope actually needs — tectonics,
  volcanism, and crater placement all seed from it directly
  (`state.tect.seed`, `state.tect.seed^0x5bf03635`,
  `state.tect.seed^0x27d4eb2f`). The civ-layer `_civRng` elsewhere in the
  file is a different, near-identical implementation used only by
  out-of-MVP-scope code — not ported.
- Extracted golden output by running the **actual JS function under real
  Node.js** (v24.19.0), not hand-derived: 9 seeds (including the two real
  XOR-derived seeds volcanism/craters actually use) × 8 values each = 72
  values, in `cartalith-rng/tests/golden_parity.rs`.
- Ported to `Mulberry32` (`cartalith-rng/src/lib.rs`) on `u32` throughout —
  JS's `Math.imul`/`^`/`>>>` are bit-identical regardless of signed vs.
  unsigned interpretation, so no `i32` split was needed. Division by
  `2^32` is exact in `f64` (power-of-two divisor, integer numerator), so
  the golden test asserts **bit-for-bit equality**, not a tolerance — the
  right bar for pure integer arithmetic, per `PARITY_TESTING.md`'s "give
  each field its own tolerance and record the reasoning."
- `cargo test -p cartalith-rng`: all 72 golden values match exactly.
  `cargo clippy -p cartalith-rng --all-targets`: clean.

**Next**: `hash`, `vnoise`, `fbm`, `ridged` — same section of the reference
file, same golden-extraction method, per `PARITY_TESTING.md`'s stated
order ("port the RNG first, and the noise second").

## Phase 1 — hash/vnoise/fbm/ridged ported, one real sign bug caught (2026-08-12)

`cartalith-noise`, continuing directly from `mulberry32`. Golden data again
extracted by running the actual JS (reference HTML lines 2292-2295) under
Node.js — 234 `hash` cases (13 x-coords x 3 y-coords x 6 seeds, including
negative coordinates) + 108 noise cases (12 float x-coords x 3 y-coords x
3 seeds, each checking `vnoise`/`fbm`/`ridged` together) = 342 values,
generated straight into a Rust test file rather than hand-transcribed
JSON, to rule out transcription error at this volume.

**First golden-test failure of the port, and exactly the kind
`PARITY_TESTING.md` warns about.** `hash`'s middle step
(`h=(h^(h>>>13))*1274126177`) doesn't use `Math.imul` — the product can
reach ~2^61, past `f64`'s exact 53-bit range, so JS's own float64
rounding is genuine, load-bearing behavior here (kept, not "fixed" — see
`cartalith-noise/src/lib.rs`'s doc comment). Getting *that* part right
the first time, the port still initially failed on `hash(0, 7,
668265263)` and others: JS's `^` operator returns a **signed** int32, and
that signed value — not its unsigned bit pattern — is what the following
plain `*` multiplies. The first draft carried the unsigned value forward
instead, silently flipping the sign of the multiplicand (and therefore
the whole result) for roughly half of all outputs. Root-caused by
comparing a step-by-step Node trace against the same steps reasoned
through in Rust, not by loosening the assertion — the discipline the
project's own skill (`cartalith-porting-discipline`) names explicitly:
"a mismatch means re-read the JS... not widen the tolerance."

Fixed with one re-interpretation cast (`h2_bits as i32`) before the
multiply; all 342 values then matched bit-for-bit.

- `cargo test -p cartalith-noise`: all 342 golden values match exactly.
  `cargo clippy -p cartalith-noise --all-targets`: clean.

Also ported `pvnoise`/`pfbm`/`pridged` — the `state.world`-wrap periodic
siblings (x lattice coordinate wraps mod `pX` so noise tiles on a
cylinder) — in the same pass, since they're the same reference-file
section and the same one caveat (JS `%` keeps the dividend's sign; the
`((xi%pX)+pX)%pX` double-mod pattern needs replicating exactly, not
swapped for Rust's `rem_euclid`). All 90 additional golden values matched
on the first attempt — the `hash` sign fix above was the only real trap
in this whole section.

- `cargo test -p cartalith-noise`: all 432 golden values (342 + 90) match
  exactly. `cargo clippy -p cartalith-noise --all-targets`: clean.

**Next**: `buildTectonicSubstrate()` — the first real pipeline stage
(`MVP_SCOPE.md` point 1), and the first one built on top of `mulberry32`
+ this noise module rather than tested in isolation.

## Phase 1 — computeWarp ported (2026-08-12)

`cartalith-terrain::compute_warp` — the first stage of `buildTectonicSubstrate`
(reference HTML lines 2621-2735: `computeWarpPrep`/`fillWarpRows`/`warpParams`
combined into one function, since JS's caching split across those three is a
perf detail orthogonal to output values). Domain-warped fbm: sample `fbm`
twice for a (qx, qy) offset, then sample again shifted by `4*(qx,qy)` for the
final displacement — classic Inigo Quilez-style warp.

4 golden cases (small grids, 5x5 to 8x6, covering `world` true/false and
the below-threshold `None` case) generated the same way as the noise
primitives, all matching bit-for-bit on the first attempt — the earlier
noise work paying off, since this stage is "just" `fbm`/`pfbm` composed
per-cell.

One thing worth being explicit about: `warpX`/`warpY` are `Float32Array`
in JS, so every stored cell is rounded from `f64` to `f32` at the point of
assignment — not just at the end. `compute_warp` returns `Vec<f32>` and
casts at the same point (`(wx * 2.0 * amp) as f32`), which reproduces
that rounding rather than losing it to a final bulk conversion; later
stages that read this field (`assignPlates`'s `ax=warpX?x+warpX[i]:x`)
see the same already-rounded value JS would.

- `cargo test -p cartalith-terrain`: 4/4 golden cases exact.
  `cargo clippy -p cartalith-terrain --all-targets`: clean (added
  `#![allow(clippy::excessive_precision)]` to the generated golden test
  files — clippy flags full-round-trip-precision float literals as
  "excessive," which is exactly the precision golden fixtures need).

**Next**: `buildPlates()` — plate initialisation (positions, velocities,
crust type), feeding `assignPlates`'s JFA Voronoi.

## Phase 1 — buildPlates ported, one more real precision trap found (2026-08-12)

`cartalith-terrain::build_plates` (reference HTML lines 2740-2766): seeds
`n` plates via `mulberry32`, then relaxes their positions toward their
own Voronoi-cell centroids for a configurable number of Lloyd iterations
(brute-force nearest-plate per cell — its own separate cost from
`assignPlates`'s JFA, which only replaces the *final* per-pixel
assignment). World-wrap mode uses a circular mean (`atan2` of summed
sin/cos) for x so a plate straddling the map seam isn't pulled toward the
middle; also supports the world-structure crust reclassification pass
(reads a continentality field, itself still unported — the parameter
shape is ready for it).

**Found the trap by reading the JS spec before porting, not by a failing
test this time**: `Math.round` in JS rounds ties toward `+Infinity`
(`Math.round(-0.5) === 0`), but Rust's `f64::round()` rounds ties away
from zero (`(-0.5_f64).round() == -1.0`) — genuinely different functions
for negative half-integer inputs. `buildPlates`'s world-wrap math
(`dx-Math.round(dx/GW)*GW`) and its world-structure reclassification
(`Math.round(plates[p].x)`) both depend on the JS behavior specifically.
Wrote a `js_round` helper (`(x + 0.5).floor()` — the standard exact
equivalent) instead of reaching for `.round()`, and made sure the golden
cases actually exercise negative `dx` under world-wrap so a wrong
rounding choice would have been caught, not just missed by luck.

5 golden cases (world true/false, 0-2 Lloyd iterations, one exercising
world-structure reclassification against a synthetic continentality
field), all bit-for-bit exact — including the `atan2`/circular-mean path,
so Rust's and V8's `atan2` agree exactly for these inputs (not guaranteed
in general for transcendental functions, but confirmed here rather than
assumed).

- `cargo test -p cartalith-terrain`: 5/5 golden cases exact (9 total with
  `compute_warp`'s). `cargo clippy -p cartalith-terrain --all-targets`:
  clean.

**Next**: `assignPlates()` — the JFA Voronoi rasterisation that turns
`plates[]` into a per-pixel `plateId` field, the first stage whose output
every later tectonic step (`computeStress`, `computeFlexure`, `ageField`,
...) actually reads.

## Phase 1 — assignPlates ported, one real infinite-loop bug caught pre-test (2026-08-12)

`cartalith-terrain::assign_plates` (reference HTML lines 2771-2810): the
Jump Flood Algorithm Voronoi rasterization — O(N log N) instead of the
brute-force O(N × plates) `build_plates`'s own Lloyd step still uses.
Every later tectonic stage (`computeStress`, `computeFlexure`, `ageField`,
...) reads this function's output, not `build_plates`'s directly.

**A real bug caught before ever running the test**, not by one: JS's
`for(let step=maxStep>>1; step>=1; step>>=1)` doesn't step through a
range — it visits exactly three fixed offsets per axis (`-step, 0,
+step`), never anything between. A first-draft port used a manual `while
dx <= step { ...; dx += step }` translation and shadowed the outer
step-halving variable with an inner per-iteration copy that was never
written back — the outer loop's own `step` never actually halved, an
infinite loop that would have hung the first test run rather than failed
it. Caught on a re-read before running anything, and replaced with the
much simpler (and correct-by-construction) `for &dy in &[-step, 0,
step]` — since the JS loop only ever has three iterations, there was no
reason to reach for a while-loop translation that needed manual increment
bookkeeping in four different early-continue branches in the first place.

Two precision details preserved deliberately, not simplified away:
- `bestD2` is a `Float32Array` in JS — the running best squared distance
  rounds to `f32` on every write, and later comparisons read that
  *rounded* value back. `best_d2: Vec<f32>` reproduces the rounding point
  exactly; an `f64` accumulator would occasionally pick a different
  winner on a near-tie.
- The world-wrap distance correction reuses `js_round`
  (`buildPlates`'s own trap, `Math.round` ties toward `+Infinity`).

4 golden cases (world true/false × warp-displaced/undisplaced), all
matching bit-for-bit on the first successful run (after the loop-shape
fix above).

- `cargo test -p cartalith-terrain`: 4/4 golden cases exact (13 total
  across the crate's three golden suites). `cargo clippy -p
  cartalith-terrain --all-targets`: clean.

**Next**: `computeStress()` — boundary classification and convergence/
shear stress accumulation, the first stage to read `plateId` rather than
`plates[]` directly.

## Phase 1 — gaussBlur ported (2026-08-12)

`cartalith-terrain::gauss_blur` plus its `box_h`/`box_v` internals
(reference HTML lines 2511-2515) — three-pass box blur approximating a
Gaussian, the shared smoothing primitive `computeStress`,
`computeFlexure`, and the base-crust blur all lean on next. CPU path
only: the JS original tries a GPU path first
(`GPU.enabled && GPU.blurOK`), but that's unavailable headless, and JS
itself already falls back to the exact CPU code ported here when it is —
so parity only needs this one branch, not a GPU-vs-CPU tolerance.

Two things preserved rather than "cleaned up": the running sum in
`box_h`/`box_v` stays `f64` throughout the sliding-window accumulation
(matching JS, where `acc` is a plain number even though it's summing
`Float32Array` reads) and only rounds to `f32` at the point of writing
`dst` — an `f32` accumulator would drift over a wide blur radius. And
`r<1` is a real early return (an unmodified copy), not an optimization
worth skipping in the port.

4 golden cases (wrap on/off, one below the `r<1` threshold), all
bit-for-bit exact on the first attempt.

- `cargo test -p cartalith-terrain`: 4/4 golden cases exact (17 total
  across the crate's four golden suites). `cargo clippy -p
  cartalith-terrain --all-targets`: clean.

**Next**: `computeStress()` itself, now that its one missing dependency
(`gaussBlur`) is in place.

## Phase 1 — computeStress ported, two double-rounding traps caught pre-test (2026-08-12)

`cartalith-terrain::compute_stress` + `classify_boundary` (reference HTML
lines 2818-2848): walks each cell's right/down neighbors (plus, under
world-wrap, the row-wrap neighbor at the right edge), and where two
different plates meet, accumulates convergence (`C`, boundary-normal) and
shear (`S`, boundary-tangent) stress from the plates' relative velocity,
classifying each boundary cell by its dominant interaction.

**Two more precision traps, both caught by reasoning through JS's typed-
array semantics before running anything** — the pattern this port keeps
hitting, per `cartalith-porting-discipline`'s emphasis on reading before
porting rather than translating syntax and hoping:

1. `raw[i]+=C` writes straight into a `Float32Array`. A first-draft
   `raw[i] += c as f32` truncates `c` to `f32` *before* adding — but JS
   promotes the existing `f32` value to `f64`, adds the full-precision
   `f64` `C`, and rounds only once at the end. Truncating first
   double-rounds, which can disagree with JS by a ULP. Fixed to
   `raw[i] = (raw[i] as f64 + c) as f32`.
2. `mx`/`ms` (the per-field normalization max) are plain JS variables —
   `f64` — even though every value feeding them is read from a
   `Float32Array`. The final `stressField[i]/=mx` divides in `f64` before
   rounding back to `f32` on store. An `f32` accumulator for `mx` would
   make that division happen in `f32` precision instead — a different
   operation, not just a different variable type.

Neither trap was found by a failing golden test — both were caught by
comparing the port line-by-line against JS's actual promotion/rounding
rules before running `cargo test` at all. The golden tests (3 cases,
covering `boundary_mask`/`boundary_type`/`stress_field`/`shear_field`
together) passed on the first real attempt, which is exactly the outcome
that discipline is supposed to produce — the alternative is discovering
these the slow way, one red test at a time.

- `cargo test -p cartalith-terrain`: 3/3 golden cases exact (20 total
  across the crate's five golden suites). `cargo clippy -p
  cartalith-terrain --all-targets`: clean.

**Next**: `computeFlexure()` — needs `boundaryMask` + `stressField`, both
now available.

## Phase 1 — computeFlexure, computeHeterogeneity, computeResistance ported (2026-08-12)

Three more `buildTectonicSubstrate` stages, all straightforward given the
primitives already in place — no new precision traps found, just careful
translation:

- `compute_flexure` (reference HTML lines 3105-3111): seeds from
  `stressField` at boundary cells only, blurs at 3x the normal radius
  (flexural wavelength >> stress wavelength), normalizes by max
  magnitude. Reuses `gauss_blur` directly.
- `terrain_detail_k` (near line 2636) + `compute_heterogeneity` (lines
  3117-3125): low-frequency noise modulated by tectonic age — old stable
  cratons show more internal basement diversity than young near-boundary
  crust. `terrain_detail_k` eases relief-noise frequency only once the
  map's real cell size drops below the app's own 800km/2048px reference
  (a no-op at or above it), the same "measure the real km scale, don't
  assume resolution implies scale" reasoning `PROVENANCE.md` flags this
  whole `terrainDetailK` family for.
- `compute_resistance` (lines 3132-3139): erosion resistance from crust
  type (continental base = harder) and age (older = more resistant) —
  the one later `streamPowerErode` will read to spatially modulate
  erodibility.

5 golden cases across the three functions, all bit-for-bit exact on the
first attempt.

- `cargo test -p cartalith-terrain`: 5/5 golden cases exact (25 total
  across the crate's six golden suites). `cargo clippy -p
  cartalith-terrain --all-targets`: clean (one `#[allow(too_many_arguments)]`
  on `compute_heterogeneity` — JS groups those params into an object only
  to share `fillHeteroRows` with a Web-Worker-pool path this port doesn't
  have; a bespoke struct here would exist solely to satisfy the lint).

**`buildTectonicSubstrate`'s remaining pieces**: `generateContinentalityField`
(world-structure archetypes, deferred with `MVP_SCOPE.md`'s own blessing
until archetypes are actually being wired up) and the T1-T5 orogeny
machinery (`buildOrogenyField`, opt-in via `state.tect.tectonicGraph`,
substantial on its own — boundary polyline graphs, thinning, per-type
structured features). Everything needed for the **default, non-orogeny,
non-archetype** path — the one Phase 1's own golden-parity harness will
actually run first — is now in place: `compute_warp`, `build_plates`,
`assign_plates`, `compute_stress`, `compute_flexure`,
`compute_heterogeneity`, `compute_resistance`.

**Next**: the height formula itself (`MVP_SCOPE.md` point 2) — the first
stage that actually turns these fields into a heightmap, and the natural
point to also wire `cartalith-engine`'s orchestration so these nine
functions run as one real pipeline instead of nine independently-tested
islands.

## Phase 1 — height formula + normalize ported (2026-08-12)

`cartalith-terrain::compute_height` + `normalize_field` (reference HTML
lines 2335-2344 and 4930-4935) — `MVP_SCOPE.md` points 2 and 3, and
literally **the formula**: `0.5 + A*(0.40*bs + 0.50*T) + Fwt*flex +
Hwt*hetero + B*N*(0.25+0.75*rug)`, transcribed term-for-term and weight-
for-weight rather than reformulated, per `DECISIONS.md` §7's "reproduce
it; do not improve it." `HeightParams` bundles the six tectonic tuning
knobs (`alpha`/`beta`/`age`/`flexure`/`hetero`/`ridged`) into one struct —
unlike `compute_heterogeneity`'s params, this grouping mirrors a real
conceptual unit (the formula's own weights), not a JS worker-pool-sharing
artifact, so it earns the struct rather than an `#[allow]`.

3 golden cases spanning the branches that actually change behavior:
world-wrap off/ridged-off, world-wrap on/ridged on/warp-displaced, and a
third adding `oro` (orogeny) input — even though `buildOrogenyField`
itself isn't ported yet, `compute_height`'s own `oro` branch
(`T=oro?oro[i]+Math.min(sf,0):sf`) needed covering now rather than left
implicitly untested until whenever orogeny lands. All three matched
bit-for-bit on the first attempt — the accumulated discipline from the
last several functions (know the JS promotion/rounding rules before
writing the port, not after a test fails) held here too.

`normalize_field` is CPU-path-only (same `GPU.enabled` fallback pattern
as `gaussBlur`) and reproduces JS's `mx-mn||1` flat-field guard (a zero
range divides by `1`, not `NaN`) explicitly rather than trusting Rust's
own zero-division behavior to coincidentally match.

- `cargo test -p cartalith-terrain`: 3/3 golden cases exact (28 total
  across the crate's seven golden suites). `cargo clippy -p
  cartalith-terrain --all-targets`: clean.

With this, every piece of `MVP_SCOPE.md`'s first two milestones — the
default tectonic substrate and the height formula — golden-verifies in
isolation. **Next**: wire `cartalith-engine`'s orchestration so these ten
functions actually run as one pipeline against a real seed, rather than
ten independently-tested islands — the natural point to also extract a
true end-to-end golden fixture (seed → full field) from the live JS
`generate()`, per `PARITY_TESTING.md`'s own recommended harness shape.

## Phase 1 — volcanism (simple mode) + craters ported (2026-08-12)

`cartalith-terrain::stamp_volcanoes_simple` + `stamp_craters` (reference
HTML lines 3466-3576) — `MVP_SCOPE.md` point 4, "the point-feature
placement and carving passes."

**A scope note, not a silent gap**: `state.volc.provinces` defaults to
`true` in the JS engine, which routes through `stampVolcanoesProvinces` —
a larger clustering algorithm (arc/rift/hotspot chains along boundary
type, spacing-based candidate selection with a Fisher-Yates shuffle) that
isn't ported yet. `stampVolcanoesSimple` (boundary-biased scatter) is the
one actually ported here, because it's the shared foundation
(`stampOneVolcano`/`placeSizedVolcano`) both modes build on, and because
verifying it in isolation first is the same "one piece at a time" reason
every other function in this port has gone this way. Provinces mode is
tracked as follow-up work, not dropped.

**A precision trap distinct from the ones found so far — clamp order,
not accumulation order.** Both `stampOneVolcano` and `stampOneCrater`
follow a JS `field[i]+=delta; if(field[i]>1)... else if(field[i]<0)...`
pattern. The natural-looking Rust translation — sum the delta(s) in
`f64`, clamp that `f64` sum, round once — can disagree with JS right at
the boundary: a sum just under `1.0` in `f64` can round *up* past `1.0`
when narrowed to `f32` (or the reverse), and JS's clamp check reads the
*already-rounded* `f32` value, not the pre-rounding sum. Fixed by adding
`add_rounded()`, a small helper that rounds to `f32` immediately on every
individual `+=` site (`stampOneCrater` has three per cell — bowl, rim,
basin — each its own rounding step in JS, not summed together), with the
clamp check reading back the stored, already-rounded value afterward —
matching JS's actual two-step process instead of the arithmetically
"nicer" one-step version that happens to be wrong at the edges.

Also worth naming since it's easy to get backwards: `placeSizedVolcano`'s
bounds check happens *before* any RNG draw, so an out-of-bounds
placement consumes zero random numbers rather than a partial draw — and
`stampVolcanoesSimple`'s `placeSizedVolcano(cx,cy,rng,rng()*v.age)` call
evaluates the age argument (one more RNG draw) *before* entering the
function body, ahead of the `r`/`hM`/`radKm` draws inside it. Every
placement function in this port is a chain where one wrong draw shifts
every subsequent one, so call-order details like these matter as much as
the arithmetic itself.

4 golden cases (2 volcano scatters, 2 crater fields including a
large/basin-crater path), all bit-for-bit exact on the first attempt —
the clamp-order and call-order issues above were both caught by reading
the JS before writing Rust, not by a failing test.

- `cargo test -p cartalith-terrain`: 4/4 golden cases exact (32 total
  across the crate's eight golden suites). `cargo clippy -p
  cartalith-terrain --all-targets`: clean.

**Next**: `stampVolcanoesProvinces` (the actual default volcanism path)
and world-structure archetypes (`generateContinentalityField`,
`applyWorldStructureSeaLevel`), or move on to climate — open which to
prioritize.

## Phase 1 — temperature ported to new cartalith-climate crate (2026-08-12)

`cartalith-climate::compute_temperature` (reference HTML lines 4951-5153)
— `MVP_SCOPE.md` point 6's temperature half. First code in this crate;
first crate boundary crossed since `cartalith-terrain`. Chose temperature
over continuing volcanism/archetypes because it's needed before wind or
rainfall can be ported at all, and moving one pipeline stage further
along felt more valuable right now than deepening one already-working
stage.

Latitude-band base temperature (grounded in axial tilt + rotation via
`insolationContrastK`/`rotationContrastK` — North & Coakley 1979's P2
energy-balance approximation, `docs/research/solar-energy-budget.md`),
cooled by altitude above sea level scaled by gravity (lapse rate), then
optionally relaxed toward a colder cryosphere equilibrium wherever ice
forms.

**Deferred, matching this port's established pattern**: `geoidField`
(`buildGeoid`, a per-cell sea-level offset from a separate planet-geoid
subsystem) isn't ported. `state.planet.geoid.enabled` defaults to
`false`, where JS's own `geoAt()` always returns `0` regardless — so
`compute_temperature`'s `geo_field: Option<&[f32]>` passed as `None`
matches the app's own default path exactly, not an approximation of it.

**One more per-pass rounding trap, same family as `computeStress`'s and
the volcanism/crater clamp-order one, caught before running any test**:
`applyCryosphereAlbedo` relaxes over 6 passes, each reading and writing
the same `Float32Array` in place — so each pass's write rounds to `f32`
*before* the next pass's `smoothstep` reads it back, not a full-precision
`f64` value carried between passes. A first draft used an `f64` working
array for the whole 6-pass loop, rounding only once at the end; fixed to
keep the working array `f32` throughout, exactly mirroring JS's real
storage. `base` (the pre-loop snapshot) is captured once and never
re-rounded, matching JS's own `Float32Array` copy semantics.

Also kept `!(k > 0.0)` rather than the clippy-preferred `k <= 0.0` for
the albedo no-op check, with a comment explaining why: JS's `!(k>0)` is
`true` for `NaN` (since `NaN>0` is `false`), matching this project's
NaN-as-off convention (`cartalith-rust-conventions`) — `k <= 0.0` would
flip that, since `NaN<=0.0` is *also* `false` in IEEE754.

3 golden cases (region mode, world-wrap mode with a non-Earth tilt/
rotation, and one exercising the albedo relaxation path), all bit-for-bit
exact on the first attempt.

- `cargo test -p cartalith-climate`: 3/3 golden cases exact.
  `cargo clippy -p cartalith-climate --all-targets`: clean.

**Next**: wind (`buildWind`/`simulateWeather`) and rainfall — the larger
half of `MVP_SCOPE.md` point 6, and the point real river/lake formation
(hydrology) will eventually depend on.

## Phase 1 — wind + rainfall ported; first genuine tolerance-based golden test (2026-08-12)

`cartalith-climate::simulate_weather` + `build_wind` (reference HTML
lines 5299-5719) — the rest of `MVP_SCOPE.md` point 6. Prevailing
latitude-band winds (band count from `circulationCells`, itself scaled by
planet rotation/size/gravity), an optional pressure-gradient + Coriolis
perturbation from a smoothed temperature proxy, then `simulateWeather`'s
iterative loop: evaporate over sea, advect moisture along wind
(semi-Lagrangian backtrace via `bilC`), precipitate on orographic lift +
convective excess, normalized against the 82nd-percentile land rainfall.

**Deferred, same pattern as every prior deferral in this port — tracked,
not silently dropped**: ocean-current SST folding (`state.climate.currents`,
which `MVP_SCOPE.md` explicitly names a stretch goal), terrain wind
deflection (`buildWind`'s `opts.elev`/`deflectFlow` branch — grouped with
ocean currents by that same `MVP_SCOPE.md` sentence, so deferred under
the same explicit permission), and world-structure continental-interior
dryness. `geoidField` is omitted too, matching `compute_temperature`'s
own reasoning.

**The first genuinely tolerance-based golden test in this port, not an
exact-equality one — found by bisection, not assumed.** Two of three
golden cases initially failed by tiny (~1e-7) amounts after the fact that
`build_wind`'s pressure/Coriolis step calls `.hypot()`. Rather than
assume "floating-point noise" and loosen the assertion — exactly what
`PARITY_TESTING.md` and this port's own skill
(`cartalith-porting-discipline`) warn against — the actual `wx`/`wy`
output was compared **bit-for-bit** (`f32::to_bits()`) against a parallel
Node run. They differ from JS by exactly 1 ULP at a handful of cells,
*immediately* after `build_wind` returns, before the iteration loop's 5
passes ever run — proving the divergence is real and located, not an
accumulating translation bug. `Math.hypot` is one of the ECMAScript Math
functions ECMA-262 explicitly permits engines to only
"implementation-approximate," unlike `+`/`-`/`*`/`/`, which are exactly
specified — so a 1-ULP disagreement between V8's and Rust's `hypot` is
expected and unavoidable, not a defect in either. `golden_parity_weather.rs`
now asserts within `1e-5` absolute tolerance (documented inline, ~100x
the observed drift) — every other golden suite in this port stays exact.

- `cargo test -p cartalith-climate`: 6/6 golden cases pass (3 temperature
  exact, 3 weather within tolerance). `cargo clippy -p cartalith-climate
  --all-targets`: clean.

With temperature, wind, and rainfall all in place, `MVP_SCOPE.md` point 6
(climate) is done for the default path. **Next**: erosion (`MVP_SCOPE.md`
point 7) — droplet, stream-power, thermal — or wiring `cartalith-engine`'s
orchestration now that tectonics + height + climate can run as one real
pipeline; open which to prioritize.

## Phase 1 — droplet erosion ported (new cartalith-erosion crate) (2026-08-12)

`cartalith-erosion::droplet_kernel` (reference HTML lines 3584-3616) —
the first piece of `MVP_SCOPE.md` point 7. Particle-based hydraulic
erosion: each droplet spawns (rain-weighted rejection sampling when
climate coupling is on), then follows the inertia-blended downhill
gradient for up to `max_lifetime` steps, eroding or depositing based on
carrying capacity vs. current sediment load, gaining/losing speed via a
simplified energy-conservation term scaled by planet gravity.

The original JS is explicitly "self-contained by design" (its own
comment: no module globals, since it's shipped to a Web Worker via
`Function.prototype.toString()`) — which made it a clean first erosion
stage to port, since the whole simulation genuinely is captured by one
function's arguments, no hidden state to track down.

**Two more round-then-clamp precision sites, same family as
`cartalith-terrain`'s volcano/crater stamps** — caught by pattern-matching
against the already-established trap, not rediscovered from scratch:
`scrape()`'s per-cell `field[i]-=amount; if(field[i]<0)field[i]=0;`
needed the clamp applied to the *rounded* `f32` value, not the
pre-rounding `f64` delta (same helper pattern as `add_rounded` in
`cartalith-terrain`, inlined here). `deposit()`, by contrast, genuinely
has no clamping in the original — erosion's own `field[i]<0/>1` bound
happens once, later, in `erodeFinish` (not yet ported), so `deposit`
here correctly doesn't clamp either.

Also preserved exactly: the do-while spawn-rejection loop's short-circuit
evaluation order (`ck>0 && rng()>... && ++tries<16` — the second and
third RNG-consuming/counting steps only happen when the earlier ones are
true, so an early exit consumes fewer random draws, which would shift
every subsequent droplet's entire trajectory if translated wrong).

2 golden cases (climate-uncoupled and rain-coupled with rejection
sampling active), both bit-for-bit exact on the first attempt — including
through a `.hypot()` call (`len=Math.hypot(dx,dy)`), which didn't trigger
the 1-ULP `Math.hypot` divergence `simulate_weather`'s golden test hit;
worth remembering as a latent, not fully eliminated, risk if a future
seed/config combination does hit it there.

- `cargo test -p cartalith-erosion`: 2/2 golden cases exact.
  `cargo clippy -p cartalith-erosion --all-targets`: clean.

**Not yet ported**: `erodeThermal` (thermal erosion — talus-angle-driven
diffusion), `streamPowerKernel`/`streamPowerErode` (the other named
erosion mode, incision as a function of drainage area and slope — needs
flow accumulation from hydrology first), `hillslopeDiffuseCPU`, and
`isostaticRebound`/`erodeFinish`'s clamp-and-recompute tail. `droplet_kernel`
alone is a real, usable erosion stage, not a stub — but `MVP_SCOPE.md`
point 7 names all three (droplet, stream-power, thermal) explicitly.

**Next**: thermal erosion (simpler, no flow-accumulation dependency) or
`cartalith-hydrology`'s flow accumulation (which stream-power erosion
needs anyway) — open which to prioritize.

## Phase 1 — thermal erosion ported (2026-08-12)

`cartalith-erosion::erode_thermal` (reference HTML lines 3856-3865,
`erodeThermalCPU` — CPU path only, GPU unavailable headless). Talus-
angle diffusion: any cell steeper than `talus` relative to a 4-connected
neighbor sheds the excess height, split proportionally among however
many neighbors are over-steep, repeated for `passes` iterations.

**A new variant of the round-then-clamp family, not just a repeat**:
`delta` is a fresh `Float32Array` *every pass*, and unlike every prior
example of this trap in the port so far (all of which were multiple
writes to the *same* cell), here a single downhill neighbor cell can
receive `+=` contributions from *several different uphill cells* within
one pass — each one JS rounds to `f32` individually as it's added, not
batched. Kept `delta` as `Vec<f32>` throughout rather than accumulating
in `f64` and rounding once, the same reasoning as `compute_stress`'s
`raw[i]+=` and `stamp_one_crater`'s three-site accumulation, just
distributed across cells instead of terms.

2 golden cases (default talus, and a much looser one to exercise more
redistribution activity), both bit-for-bit exact on the first attempt.

- `cargo test -p cartalith-erosion`: 4/4 golden cases exact across the
  crate's two suites. `cargo clippy -p cartalith-erosion --all-targets`:
  clean.

**Remaining in `MVP_SCOPE.md` point 7**: `streamPowerKernel`/
`streamPowerErode` — needs flow accumulation from hydrology first, so
it's naturally blocked until `cartalith-hydrology` exists — plus
`hillslopeDiffuseCPU` and `isostaticRebound`/`erodeFinish`'s tail.

**Next**: `cartalith-hydrology`'s flow accumulation — both unblocks
stream-power erosion and is `MVP_SCOPE.md` point 8 in its own right.

## Phase 1 — flow accumulation ported (new cartalith-hydrology crate) (2026-08-12)

`cartalith-hydrology::compute_flow` (reference HTML lines 4846-4890) —
`MVP_SCOPE.md` point 8's first piece: D8 steepest-descent flow
accumulation, seeded either uniformly (bare cell count) or by rainfall
(discharge — Whipple & Tucker 1999, mean-normalized with a `0.05` floor).

**A deliberate algorithm substitution, not a full port — and a
documented one, per `PROVENANCE.md`'s own rule.** The JS original
(`_flowRadixSortDesc`) sorts cells by a hand-rolled radix sort operating
on IEEE-754 bit patterns re-mapped into an order-preserving `u32` key.
Flow accumulation is downstream of the heightmap pixels
(`PROVENANCE.md` §2's line: "hand-port anything upstream of the
heightmap; take a crate for anything downstream of the pixels"), so only
the *ordering guarantee* — descending height, ties broken by ascending
original index — matters for parity, not the sort algorithm producing
it. Replaced with `Vec::sort_by` and an explicit tie-breaking comparator,
which is simpler and doesn't need reimplementing a bit-trick radix sort.

One quirk *did* need carrying over rather than assumed away: JS
explicitly normalizes `-0.0`'s sort key to equal `+0.0`'s
(`if(b===0x80000000) b=0`) before building the order-preserving key.
Rust's `f32::total_cmp` does *not* do this on its own — it defines
`-0.0 < +0.0`, a real total ordering, just a different one than JS's
here. `flow_cmp_desc` normalizes both operands to canonical `+0.0`
before comparing, so a `-0.0`/`+0.0` tie still falls through to the
index-based tiebreak exactly as it would in JS, rather than being
silently ordered by sign.

Same `acc[best]+=acc[i]` multi-writer-per-pass trap `erode_thermal`'s
`delta[j]+=` already established the pattern for: a downhill cell can
receive accumulated flow from several different upstream cells within
the same pass, each rounding to `f32` individually — kept as per-write
rounding, not an `f64` accumulator.

3 golden cases (area-seeded, rain-seeded + world-wrap, rain-seeded
region), all bit-for-bit exact on the first attempt.

- `cargo test -p cartalith-hydrology`: 3/3 golden cases exact.
  `cargo clippy -p cartalith-hydrology --all-targets`: clean.

This also unblocks `streamPowerErode` (needs `computeFlow`'s discharge
output), the piece `MVP_SCOPE.md` point 7 (erosion) was still missing.

**Next**: `streamPowerKernel`/`streamPowerErode` (now unblocked), or
river network extraction (Strahler ordering, polyline tracing,
real-km-aware channel width — the rest of `MVP_SCOPE.md` point 8); open
which to prioritize.

## Phase 1 — streamPowerKernel ported: MVP_SCOPE.md point 7 complete (2026-08-12)

`cartalith-erosion::stream_power_kernel` (reference HTML lines
4082-4194) — the third and last erosion mode `MVP_SCOPE.md` point 7
names, and the largest single function ported in this session: priority-
flood depression fill, multiple-flow-direction drainage area (Freeman
1991), and implicit stream-power incision (Braun & Willett 2013) with an
optional sediment-deposition pass.

**The `MinHeap` was hand-ported field-for-field, deliberately not
substituted for `std::collections::BinaryHeap`** — the one place in this
whole port so far where `PROVENANCE.md`'s algorithm table names the
*exact reason* not to take the easy crate substitute: "Priority-flood
depression fill (Barnes-style) — hand-port, carefully. Equal-priority
pop order decides the fill tie-break and therefore lake shape." A
generic binary heap would very likely produce a *valid* flood-fill, just
not the *same* one on ties — silently different lake shapes and channel
placement. Ported the same array-backed sift-up/sift-down comparison and
swap order line-for-line instead.

Three more precision/ordering details preserved deliberately, all
recognized from patterns this port has already established:
- `Cc` (implicit-incision coefficient) is a `Float64Array` in JS —
  genuinely full `f64` precision, computed once and reused across every
  iteration, never rounded through `f32`. Kept as `Vec<f64>`, not
  downcast.
- `area[j]+=...` and `sed[r]+=sed[i]` are both the same multi-writer-
  per-pass trap `erode_thermal`'s `delta[j]+=` and `compute_flow`'s
  `acc[best]+=` already established — a single target cell can receive
  contributions from several different source cells within one pass,
  each JS write rounding to `f32` individually.
- The sediment-deposition block's two conditional adjustments to
  `fld[i]`/`sed[i]` are sequential statements, not simultaneous — the
  second condition reads back the *already-rounded* result the first one
  just wrote (matters when both conditions fire on the same cell in the
  same pass).

2 golden cases (default/no-deposition, and a climate-coupled config with
deposition enabled — the more complex code path), both bit-for-bit exact
on the first attempt despite this being the largest and most intricate
single function ported so far. That outcome is the real payoff of the
accumulated discipline from every earlier function in this port: read
the JS's exact promotion/rounding/ordering rules before writing the
Rust, not after a red test.

- `cargo test -p cartalith-erosion`: 6/6 golden cases exact across the
  crate's three suites. `cargo clippy -p cartalith-erosion
  --all-targets`: clean.

**With this, `MVP_SCOPE.md` point 7 (erosion) is complete** — droplet,
thermal, and stream-power all ported and golden-verified. `eroFinish`'s
tail (`isostaticRebound`, `enforceRiverChannels`, dynamic-lithology
recompute) remains unported, but the three named erosion *mechanisms*
are done.

**Next**: river network extraction (Strahler ordering, polyline tracing,
real-km-aware channel width — the rest of `MVP_SCOPE.md` point 8), or
wiring `cartalith-engine`'s orchestration now that tectonics, height,
climate, and erosion can all run as one real pipeline; open which to
prioritize.

## Phase 1 — channelization + Strahler ordering ported (2026-08-12)

`cartalith-hydrology::build_channels` + `strahler_from_receivers`
(reference HTML lines 4454-4522), plus `river_flow_thresh` and
`river_coarse_ease` (the latter added to `cartalith-terrain`, alongside
its sibling `terrain_detail_k` — same "scale-invariant terrain" family,
`docs/research/scale-invariant-terrain.md`). `MVP_SCOPE.md`'s own
"Strahler ordering" bullet, specifically — not the whole of
`buildRiverNetwork()`.

**Scoped deliberately, not the full function**: `buildRiverNetwork()`
also stamps channel width/depth/intensity into render-ready fields and
(separately) traces polylines for the renderer. Neither changes network
*topology* — which cells channelize, and the Strahler order of each —
so this pass ported exactly what `strahler_from_receivers` needs and
nothing past it, deferred the rest explicitly.

**A real subtlety worth naming**: the single-receiver tree isn't picked
by raw steepest-D8. Following Tarboton (1997), each channel cell
computes a continuous gradient *aspect* (`atan2(-gy,-gx)`) and picks
whichever of its 8 neighbors best combines downhill drop with alignment
to that true aspect (`drop*(0.5+0.5*cos(Δθ))`), falling back to plain
steepest-descent only when no neighbor is well-aligned. This removes the
45°/90° staircase bias a pure-D8 tree would carry into traced polylines
later — a real algorithmic choice in the original, not an
implementation detail, so it needed porting exactly (the angular-
distance wrapping via `atan2(sin(Δθ),cos(Δθ))` in particular — a common
idiom for "shortest signed angular distance," easy to get subtly wrong
translating between languages' `atan2` argument order, which is the same
`(y, x)` in both JS and Rust here so no transposition risk, but worth
flagging as the kind of detail that's easy to transpose without
noticing).

`riverFlowThresh`'s two-width-parameter shape was also worth preserving
exactly rather than simplifying: the reference's own comment states the
divisor terms (`terrainDetailK`/`riverCoarseEase`) must read the
*world's* own grid width and real km extent, never the grid actually
being classified — otherwise an LOD tile's threshold would be a tile-
local mis-estimate rather than anchored to the real world's detail
level. `river_flow_thresh` keeps `world_gw` and `gw` as separate
parameters for this reason, even though they're always equal on the MVP
path (no tiled LOD yet).

3 golden cases (region, world-wrap, and a higher-density config that
also exercises `channelThreshold`'s density-reshaping `dexp` term), all
bit-for-bit exact on the first attempt — including the D∞-aspect
receiver routing and the Strahler ordering's tributary-counting logic.

- `cargo test -p cartalith-hydrology`: 6/6 golden cases exact across the
  crate's two suites. `cargo clippy -p cartalith-hydrology
  --all-targets` and `cargo clippy -p cartalith-terrain --all-targets`:
  both clean.

**Remaining in `MVP_SCOPE.md` point 8**: channel width/depth/intensity
stamping and polyline tracing (`buildRiverNetwork`'s other half),
`carveRiverValleys` (which actually cuts channels into the height
field), `enforceRiverChannels`.

**Next**: wiring `cartalith-engine`'s orchestration — tectonics, height,
climate, and both erosion + channel-topology hydrology can now run as
one real pipeline against an actual seed, which is also the natural
point to extract a true end-to-end golden fixture from the live JS
`generate()`, per `PARITY_TESTING.md`'s own recommended harness shape;
or continue deeper into hydrology (width/depth stamping, valley
carving). Open which to prioritize.

## Phase 1 — `cartalith-engine` wired, first true end-to-end fixture (2026-08-12)

`generate_terrain()` (reference HTML `generate()` lines 3339-3391 + its
`buildTectonicSubstrate` prefix, lines 3396-3462) — the sync, no-worker-
pool path, since this port has no browser worker pool. Runs every
already-ported subsystem in the JS engine's own order, seed through to
river-network *topology*: warp → plates → assign → stress → flexure →
base-blur → age → heterogeneity → resistance → height → normalize →
volcanism/craters + clamp → flow(area) → temperature → weather →
moisture correctors → flow(discharge) → channelize → Strahler. Stops
one step before `carveRiverValleys()` (needs river polyline tracing +
channel width, neither ported yet).

Two gaps found and closed while wiring, both real ported functions that
existed only as *inputs* other functions expected, never themselves
produced:

- **`build_age_field`** (`cartalith-terrain`) — `distanceToBoundary()` +
  its immediate `ageField` normalize (ref lines 2860-2879, 2779-2783). A
  two-pass chamfer distance transform; every prior heterogeneity/
  resistance/height golden test had been handed a pre-made `age_field`
  fixture, so this gap was invisible until something had to actually
  *produce* one. First attempt used a pure-`f32` accumulator inside the
  per-cell `min` chain and failed 5 of 80 cells by 1 ULP — the same
  double-rounding trap `compute_stress` already has a comment for
  (JS's `v` stays `f64` across a cell's whole chain of `Math.min`
  comparisons; only the final `d[idx]=v` rounds once). Fixed by keeping
  the accumulator `f64` per-cell, matching the existing pattern instead
  of rediscovering it. 3 golden cases, exact after the fix.
- **`apply_climate_moisture_correctors`** (`cartalith-climate`) — ref
  lines 5188-5225. Unconditional in `refreshClimate()`, unlike
  `applyOceanCurrents`/`computeSeasons` (both opt-in, off by default,
  still not ported): coastal-proximity rain boost, river-corridor
  moisture boost, ITCZ/subtropical-dry-belt sharpening — three
  sequential in-place passes over `rainField`. 2 golden cases (region,
  world-wrap), exact on the first attempt.

**Deliberately not reproduced, and why** (full reasoning in
`generate_terrain`'s own doc comment): World-Structure archetypes and
graph-driven orogeny are both off at the JS engine's own defaults, so
omitting them is bit-identical, not an approximation, at those
defaults. `stampVolcanoesProvinces` (JS default) vs `stampVolcanoesSimple`
(only one ported) is a previously-logged deviation, not a new one.
Ocean-current SST folding is taken as deferred per `MVP_SCOPE.md`'s own
stretch-goal permission, despite `state.climate.currents` defaulting
`true` — consistent with `simulate_weather`'s pre-existing deferral of
the same mechanism (and of `buildWind`'s terrain-deflection branch,
which the same v1.78 comment says is "no longer a toggle" in the JS
source, so this is a real, disclosed divergence from the literal JS
default, not a stale toggle assumption).

**First true end-to-end golden fixture**: extracted the tectonic-
substrate-through-flow(area) segment verbatim from the reference HTML
(no stripping needed — nothing in this segment is deferred) into a
Node harness declaring the same module globals the JS functions
themselves read/write, then ran `generate_terrain` against the same
seed/grid/params and asserted exact equality on ten fields (`field`,
`plate_id`, `boundary_mask`, `stress_field`, `flexure_field`,
`age_field`, `heterogeneity_field`, `resistance_field`,
`volcanic_field`, `impact_field`, `flow_area`). 2 cases (region,
world-wrap), 24×18 and 20×14, both bit-for-bit exact. This is the
first test in the port that exercises *wiring* — whether
`generate_terrain` threads each subsystem's output into the next
subsystem's input correctly — rather than any one subsystem in
isolation; every function it calls already has its own golden test,
but none of those catch an argument passed in the wrong order or a
field threaded from the wrong stage.

**Not re-verified end-to-end**: climate (temperature/weather/moisture
correctors) and river-network topology past `flow(area)`. Both are
already golden-tested in isolation by their own crates' test suites.
Extending the end-to-end fixture through them would require hand-
stripping `buildWind`'s terrain-deflection branch and the ocean-current
SST fold from the JS harness copy to match this port's own documented
deferrals of both — real transcription risk for coverage the existing
per-function tests already provide. Flagged explicitly, not silently
assumed.

- `cargo test --workspace`: every suite green, including the new
  `cartalith-engine` tests (1 smoke test + 2 end-to-end golden cases)
  and the two new `cartalith-terrain`/`cartalith-climate` golden
  suites above (3 + 2 cases). `cargo clippy --workspace --all-targets`:
  clean.

**Next**: river polyline tracing + real-km-aware channel width, then
`carveRiverValleys`'s stream-power tail (the rest of `MVP_SCOPE.md`
point 8) — both needed before `generate_terrain` can reach the same
point `generate()` does; or `cartalith-io` (reading a real `.zip` save
per `SAVEFILE_COMPAT.md`, which also doubles as independent golden
data per its own MVP entry); or basic 2D rendering in `cartalith-godot`
now that a real field exists to draw. Open which to prioritize.

## Phase 1 — `carveRiverValleys` wired: `generate_terrain` now matches a default `generate()` (2026-08-12)

Closed the gap the previous entry's "Next" note named. `generate_terrain`
now runs `carveRiverValleys()` (reference HTML lines 8761-8789) whenever
`carve_rivers` is on (the JS default) — light discharge-weighted
stream-power pass → isostatic rebound → river-network topology on the
now-lightly-eroded field → vector channel carve (`enforceChannelDescent`,
cutting through rises so every carved valley actually descends to its
outlet) → final `flow(discharge)` + `refreshClimate()` against the carved
field. This is the same point a fresh default `generate()` call itself
leaves `field`/`tempField`/`rainField`/`flowField` at — not an
approximation of it.

Three new functions ported to close real, previously-unaddressed gaps:

- **`isostatic_rebound`** (`cartalith-erosion`) — erosional unloading
  returns as broad flexural uplift (England & Molnar 1990), one-sided
  (only net removal rebounds). 2 golden cases, exact.
- **`trace_river_polylines`** (`cartalith-hydrology`) — walks each
  channel cell's receiver downstream from every un-donored source,
  main-stems-first so trunks trace as long contiguous polylines rather
  than being fragmented by a tributary claiming shared cells first.
  1 golden case (a confluence, two headwater branches into one trunk),
  exact.
- **`enforce_channel_descent`** (`cartalith-hydrology`) — stamps a
  parabolic cross-section along a polyline whose centreline is forced to
  descend monotonically, cutting through any rises so the carved valley
  drains to its outlet; returns the carved cells so the caller can lock
  them. 1 golden case (a deliberate mid-path rise), exact — the "cut
  through rises" behavior is directly visible in the fixture. Also added
  `river_width_scale_k` (`cartalith-hydrology`, trivial) — the real-km
  channel-width scale `MVP_SCOPE.md` point 8 names.

**A real restructuring, not just an addition**: the previous entry's
`generate_terrain` computed river-network topology (`build_channels` +
`strahler_from_receivers`) as a standalone final step after
`flow(discharge)`. That doesn't match `generate()`'s actual call graph —
`buildRiverNetwork` (topology's real JS source) is *only* ever called
from inside `carveRiverValleys`, on the field *after* the light
stream-power pass, not on the pre-erosion field. Wiring the real carve
step meant moving topology computation to where JS actually computes it,
not just appending carving after the old topology call. `channels`/
`stream_order`/`river_mask`/`river_floor` are now `Option`s on
`WorldState`, `None` when `carve_rivers` is off — matching JS, where none
of that data exists at all outside `carveRiverValleys`.

**Also added**: `resist` to `TectonicParams` (streamParams()'s
erodibility-resistance weight — a real `state.tect` field nothing had
read yet) and a new `StreamParams` struct (`state.stream`'s
uplift/k/iters/deposit/climateK — `cycles` omitted, only read by the
manual "Stream evolve" tool, not `carveRiverValleys`).

**Deliberately not reproduced in this pass** (all previously logged,
none new): `recomputeResistanceAfterErosion` (`state.tect.dynamicLithology`
default off), `enforceRiverChannels` (always a no-op on any *fresh*
`generate()` — `riverMask` only ever gets populated by a prior carve or
manual brushing, both of which start empty). River-network render/export
helpers (`splitRiverPolylines`, `riverSinuAmp`/`riverSinuosity`,
`buildFeatureRegistry`, `buildRiverNetwork`'s own width/intensity/depth
stamping loop) are all render- or export-time concerns per the
reference's own comment on `splitRiverPolylines` — `carveRiverValleys`
computes its own simpler per-polyline half-width directly and never
reaches that loop.

**Second true end-to-end golden fixture**, extending the first
(tectonic-substrate-through-flow(area)) all the way through the full
carve tail: extracted `streamPowerKernel`, `isostaticRebound`, the
channelization+Strahler topology, `traceRiverPolylines`,
`enforceChannelDescent`, and `carveRiverValleys`'s own orchestration
verbatim into the Node harness (`buildWind`/`simulateWeather` keep the
same elev/currents omissions `simulate_weather`'s own golden tests
already use, for a fair comparison of what's actually ported). 2 cases,
asserted against `generate_terrain`'s final `field`/`temperature`/
`rainfall`/`flow_discharge`/`river_mask`.

**Tolerance, not exact equality, for this one** — unlike the first
fixture. A pipeline chaining this many independent `Math.hypot` call
sites (`computeStress`, `streamPowerKernel`'s D8 table,
`enforceChannelDescent`, `buildWind`, `simulateWeather`'s advection)
accumulates the same 1-ULP JS/Rust divergence `golden_parity_weather.rs`
already documents from more than one site, rather than the zero or one
site earlier, shorter fixtures hit. `river_mask` (a discrete carve
decision) stayed *exact* in both cases despite the underlying floats
being off by ~1e-7 — itself evidence the divergence is float noise, not
a wrong decision.

- `cargo test --workspace`: all green, including 2 new golden suites
  (`cartalith-erosion`: 2 cases: `cartalith-hydrology`: 3 cases across
  a new suite) and the new `cartalith-engine` end-to-end carve fixture
  (2 cases) plus an updated smoke-test pair (one asserting
  `carve_rivers:true` produces topology, one asserting `carve_rivers:
  false` produces `None`). `cargo clippy --workspace --all-targets`:
  clean.

**Remaining for full `generate()` parity**: World-Structure archetypes
(`MVP_SCOPE.md` point 5 — the continentality field + sea-level
re-anchoring, both currently no-ops since WS defaults disabled),
graph-driven orogeny (WS-gated, same reason), `stampVolcanoesProvinces`,
ocean-current SST folding + terrain wind deflection, seasons
(`computeSeasons`), dynamic lithology. All previously logged, all
no-ops or off at the JS engine's own defaults except the three genuine
stretch-goal deferrals (`MVP_SCOPE.md`'s own permission).

**Next**: `cartalith-io` (reading a real `.zip` save per
`SAVEFILE_COMPAT.md` — also doubles as independent golden data per its
own MVP entry, and a real save exercises paths this port's own
synthetic fixtures can't); basic 2D rendering in `cartalith-godot` now
that a real, carved field exists to draw; or World-Structure archetypes
(`MVP_SCOPE.md` point 5). Open which to prioritize.

## Phase 1 — `cartalith-io` reads a save's terrain fields (2026-08-12)

`MVP_SCOPE.md` point 12: **reading only** — `load_save()` opens a
`.zip`, pulls `params.json`'s `GW`/`GH`/`state.tect.seed`/
`state.mapWidthKm`/`state.seaLevel`/`state.world`, and the six
terrain-field entries `SAVEFILE_COMPAT.md`'s table names
(`heightmap.f32`, `temperature.f32`, `rainfall.f32`,
`volcanic_field.f32`, `impact_field.f32`, `strahler_order.bin`). Chosen
approach 1 from `SAVEFILE_COMPAT.md`'s own two options — parse
`params.json` as `serde_json::Value` and pull only what this port's
pipeline reads, rather than a struct modeling the whole (large, mostly
civ/UI) `state` object.

Takes `Read + Seek` rather than a filesystem path — `zip::ZipArchive`
accepts a `File`, an in-memory `Cursor<Vec<u8>>`, or anything else that
implements both, so the reader isn't tied to disk I/O and is trivially
testable in-memory.

**Ignores unknown zip entries; never errors on them** — a real export
carries far more than this reader wants (biome/lithology rasters, civ
data, a baked atlas, `map.png`, a README), and
`SAVEFILE_COMPAT.md` calls that out explicitly as normal, not
corruption. Errors are typed (`LoadError`) rather than panics —
`MissingEntry`/`MissingField` name exactly what's missing, `Zip`/`Io`/
`Json` wrap the underlying library errors.

**No real HTML-app export available to test against in this
environment** (no browser to produce one) — `SAVEFILE_COMPAT.md` itself
names confirming against a real export as one of the first things to
do, and that's still genuinely unverified. Tested instead against a
synthetic `.zip` this crate's own tests build (via the `zip` crate's
writer, STORE method, matching pre-v1.90 saves) with the exact entry
names/layout `SAVEFILE_COMPAT.md` documents, including a deliberately
unknown entry to exercise the "ignore, don't error" path, and a
missing-entry case to exercise the error path. This proves the reading
logic itself is correct — byte layout, JSON field paths, entry
handling — but is **not** a substitute for real-export testing, and
is flagged as such rather than silently treated as equivalent.
`SAVEFILE_COMPAT.md`'s own "doubles as golden data" framing for a real
export remains unrealized until one is available.

3 tests (round-trip on a region config, round-trip on a world config,
missing-entry error path), all passing. `cargo test --workspace`:
green. `cargo clippy --workspace --all-targets`: clean. New
dependencies: `zip` (with `deflate`), `serde` (`derive`), `serde_json`.

**Next**: a real HTML-app `.zip` export to close the verification gap
above (owner-provided, since this environment can't produce one);
`cartalith-godot`'s basic 2D rendering (color + hillshade) now that
both a generated field and a loaded save's field exist to draw; or
World-Structure archetypes (`MVP_SCOPE.md` point 5 — the one remaining
MVP-listed subsystem with no port started at all). Open which to
prioritize.

## Phase 1 — basic 2D rendering + minimal UI: the first visible map (2026-08-12)

`MVP_SCOPE.md` points 10-11, closing `cartalith-godot`'s Phase 0
placeholder gap. A new `WorldGen` GDExtension class
(`ARCHITECTURE.md`'s own named API surface: "a `WorldGen` with
`generate(seed, width_km, resolution)` and accessors returning
fields") wraps `cartalith_engine::generate_terrain`, and a new
`build_color_texture()` method turns the result into a Godot
`ImageTexture` — the first point in this port where a generated world
is actually visible, not just numerically verified.

**Square grid only** (`gw == gh`, from a single `resolution` parameter)
— the reference HTML's own aspect-from-image/`resW` handling is
UI-layer scope this port hasn't built. `build_color_texture()`
deliberately does not attempt colour parity with the reference
renderer: `MVP_SCOPE.md` point 10 explicitly excludes "multi-octave
grain, NPR styles, splat textures, or LOD pyramid," and a from-scratch
simplified renderer (sea-level split, three-stop hypsometric land
ramp, bathymetric water ramp, analytic hillshade from the height
gradient, a blue tint on `carve_rivers`-produced channel cells) is
enough to satisfy `MVP_SCOPE.md`'s own "done" checklist point 2: "land
and water distinct, biome colouring plausible, rivers visible."

Minimal UI (`main.tscn`/`main.gd`): seed/resolution/map-width `SpinBox`
inputs, a Generate button, a `TextureRect`. `ARCHITECTURE.md`'s own
constraint held to exactly: "Godot computes nothing beyond layout" —
`main.gd` only reads input values and calls into `WorldGen`, no
numeric logic of its own. The Phase 0 `WalkingSkeleton` ping UI (its
job — proving the extension loads and survives Windows/Android export
— already done and logged) is retired from the visible scene; the node
itself stays instantiated so its `ready()` print remains a cheap
"extension actually loaded" canary.

**Verified headlessly, not visually** — this environment has no
interactive display to open the Godot editor in. Confirmed instead via
`godot --headless -s smoke_test.gd`: `WorldGen.generate()` runs the
full pipeline, `build_color_texture()` returns a non-null texture of
the requested size, and the rendered image — saved to a PNG and
actually inspected — shows a plausible small landmass with distinct
water/land colouring and visible shading, not a blank or garbage
image. This is real evidence the rendering path works end-to-end, but
it is **not** the same as the owner opening the project in the Godot
editor and confirming it looks right interactively — flagged
explicitly, matching this port's established practice for anything a
headless session can't fully confirm (`DECISIONS.md` §5,
`cartalith-porting-discipline`'s own "flag what can't be verified"
rule, already applied to the Windows/Android export checks).
`smoke_test.gd` is kept in the repo as a cheap, repeatable headless
regression check — not a substitute for the owner's own look.

- `cargo build -p cartalith-godot`: clean. `cargo test --workspace` /
  `cargo clippy --workspace --all-targets`: unaffected, still green —
  `cartalith-godot` has no `cargo test`-visible tests of its own (GDScript
  isn't exercised by `cargo test`), verification is the headless Godot
  run above.

**Remaining for MVP_SCOPE.md's "done means all seven"**: criterion 7
(open a real `.zip` and render it against the HTML app's own output for
the same file) needs the still-missing real export (see the previous
entry) *and* wiring `cartalith-io::load_save`'s output through
`build_color_texture`'s same rendering path (not yet done — today's
work only wires the `generate_terrain` path). Criteria 3/4 (Windows/
Android, owner-run) were already confirmed in Phase 0.

**Next**: wire `cartalith-io::load_save` into `WorldGen` (a second
`load()` entry point next to `generate()`, reusing the same
`build_color_texture()` renderer) so criterion 7 is one step away from
just needing a real export; World-Structure archetypes
(`MVP_SCOPE.md` point 5); or the owner opening the project in the
Godot editor to close the visual-verification gap this entry flags.
Open which to prioritize.

## Phase 1 — World-Structure archetypes (2026-08-12)

`MVP_SCOPE.md` point 5, the last MVP-listed subsystem with no port at
all until now. `generate_continentality_field` and
`apply_world_structure_sea_level` (`cartalith-terrain`, reference HTML
lines 2556-2589 and 2603-2617) — a coarse-grid percentile-normalized
noise field for continentality/fragmentation shape, and a histogram
re-anchor of sea level against the *actual* generated field so an
archetype's promised land fraction holds regardless of how its own
`tectonicEnergy`/`oceanDepth` reshaped the height distribution
independently (the exact v1.25 bug the reference's own comment
describes — Archipelago/Volcanic rendering *more* land than Classic
despite promising less). 5 golden cases (3 continentality-field
configs, 2 sea-level re-anchors), all bit-for-bit exact on the first
attempt. `build_plates`'s own continentality-reclassification branch
(`WorldStructure<'a>`) was already ported in an earlier pass and just
needed a real caller.

`cartalith-engine::generate_terrain` now derives `tect.plates`/
`tect.vel`/`volc.count` from the archetype's own params exactly as
`deriveFromWorldStructure()` does (reference HTML lines 2528-2538)
whenever `world_structure.enabled` — these three become **entirely
archetype-controlled**, not independently configurable, matching JS.
`WorldParams` takes the five raw archetype knobs directly
(continentality/fragmentation/tectonicEnergy/oceanDepth/hotspotDensity)
rather than modeling `ARCHETYPES`' named presets — a caller wanting
"Archipelago" passes that preset's own numbers.

**One real, deliberate deviation, not a no-op-at-defaults case like
every other item on this port's deferred list**: JS's
`deriveFromWorldStructure()` always sets `state.tect.tectonicGraph=true`
alongside the plates/vel/volc.count derivation — turning on
graph-driven orogeny (`buildOrogenyField`, T2+T3: boundary-polyline-
graph-driven fold/trench/fault-block landforms along each margin,
replacing the older convergent-stress "blob" uplift). This port has not
ported `buildOrogenyField` — a large separate subsystem — so `oro`
stays `None` even when World-Structure is enabled. A World-Structure
world generated here gets the right continentality shape and land
fraction (both real, verified, load-bearing effects) but the older blob
uplift instead of JS's structured per-margin orogeny. Flagged
explicitly in `generate_terrain`'s own doc comment, not silently
approximated — `foldIntensity`/`trenchDepth` (orogeny-only tuning
knobs) are correspondingly not modeled at all.

Added `WorldState::sea_level` — the sea level actually used for a given
generation, which callers that classify land vs. ocean must read
instead of `WorldParams::sea_level` once World-Structure can move it.
Caught this was needed while writing a wiring test that measures land
fraction: `cartalith-godot::WorldGen` was reading `p.sea_level`
directly (harmless today, since nothing yet exposes `world_structure`
to the GDScript UI, but silently wrong the moment it is) — fixed to
read `ws.sea_level` instead.

A new `cartalith-engine` wiring test (not a golden-parity test — the
underlying formulas are already verified by `cartalith-terrain`'s own
golden suite) generates an Archipelago- and a Supercontinent-configured
world at the same seed/grid and asserts Archipelago's land fraction is
smaller — the actual, user-visible reason `apply_world_structure_sea_level`
exists, per its own doc comment.

- `cargo test --workspace`: all green, including 5 new
  `cartalith-terrain` golden cases and the new `cartalith-engine`
  wiring test. `cargo clippy --workspace --all-targets`: clean.

**Remaining, all previously logged**: graph-driven orogeny (now the
one deviation actually reachable by enabling a real feature, not just
"off at JS's own default"), `stampVolcanoesProvinces`, ocean currents,
terrain wind deflection, seasons, dynamic lithology — the stretch-goal
deferrals `MVP_SCOPE.md` itself sanctions, plus the two owner-only
items already flagged (a real `.zip` export, eyes on the Godot editor).

**Next**: wire `cartalith-io::load_save` into `WorldGen`
(`MVP_SCOPE.md`'s "done means all seven" criterion 7's other half);
expose `world_structure` to the GDScript UI now that the engine side
is real; or graph-driven orogeny, the largest remaining unported
subsystem. Open which to prioritize.

## Phase 1 — dynamic lithology (L4 exhumation hardening) ported and wired (2026-08-13)

`recomputeResistanceAfterErosion()` (reference HTML line 3144, doc
comment at lines 3140-3143): where erosion has carved deeply
(`pre[i] - post[i]` large, i.e. net exhumation), resistance climbs
toward a basement maximum of 1.0 so the *next* erosion pass bites less
there — differential erosion producing benches/inselbergs/hard sills.
Pure per-cell formula, no RNG or convolution involved, so ported and
golden-verified with a hand-derived unit test directly against the JS
arithmetic (`cartalith-erosion`'s own test module) rather than an HTML
extraction harness — nothing in `min(1, resist[i] + k*ex)` is
order-sensitive the way `gauss_blur` or noise are.

Wired into `cartalith-engine::generate_terrain`'s `carveRiverValleys`
step, mirroring JS's own call site (`eroFinish`) exactly: called right
after `isostatic_rebound`, gated on a new `TectonicParams::
dynamic_lithology` field (default `false`, matching `state.tect.
dynamicLithology`'s own JS default) with `k` fixed at 6.0 — JS's own
built-in default when `eroFinish` calls it with no `opts`. Off by
default, so this pipeline stays bit-identical to before unless a
caller opts in; `resistance_field` is now declared `mut` to allow it.

- `cargo test --workspace`: all green, including the new
  `recompute_resistance_matches_js_formula` unit test.
  `cargo clippy --workspace --all-targets`: clean.

**Remaining, all previously logged**: graph-driven orogeny,
`stampVolcanoesProvinces`, ocean currents, terrain wind deflection,
seasons — the stretch-goal deferrals `MVP_SCOPE.md` itself sanctions,
plus the two owner-only items already flagged (a real `.zip` export,
eyes on the Godot editor). Dynamic lithology is no longer on this
list.

**Next**: same open choice as last entry — `cartalith-io::load_save`
into `WorldGen`, `world_structure` exposed to the GDScript UI, or
graph-driven orogeny.

## Phase 1 — `cartalith-io::load_save` wired into `WorldGen`: MVP_SCOPE.md criterion 7 (2026-08-13)

`WorldGen` (`cartalith-godot`) now carries a `WorldSource` enum
(`Generated(Box<WorldState>)` from `generate()`, or `Loaded(Box<SaveData>)`
from a new `load_save(path)` method) instead of a bare `Option<WorldState>`
— a loaded save only has the terrain fields `SAVEFILE_COMPAT.md` documents
(no plate/stress/flexure substrate; those were never part of the save
format), so it's a distinct variant rather than a partially-fake
`WorldState`. `build_color_texture` reads `(field, channel_mask)` through a
small match on the source instead of touching `WorldState` fields directly,
so the existing renderer (`MVP_SCOPE.md` point 10, already logged as
deliberately not the reference's full renderer) needed no changes beyond
that indirection. Channel overlay for a loaded save reuses
`strahler_order` (`u8`, `0` = non-channel) directly — same semantics as
`ChannelResult::chan` from a fresh `generate()`, so one `!= 0` check covers
both sources.

`load_save(path: GString) -> bool` opens `path` as a plain
`std::fs::File` (`cartalith_io::load_save` only needs `Read + Seek`, so no
Godot `FileAccess` involvement) and returns `false` — printing the error,
leaving the previous `source` untouched — on any open/parse failure,
matching `generate()`'s own fail-quietly-check-the-console shape. `path` is
a native OS path, which is what a GDScript `FileDialog` in filesystem-access
mode returns; `res://`/`user://` virtual paths are not handled and aren't
needed for this criterion.

`main.tscn`/`main.gd`: added a "Load Save (.zip)..." button and a
`FileDialog` (`access = FILESYSTEM`, `*.zip` filter) — picking a file calls
`WorldGen.load_save` then `build_color_texture`, mirroring the Generate
button's own status-label pattern.

- `cargo build/test/clippy --workspace`: all green/clean, including the
  boxed `WorldSource` variants (clippy's `large_enum_variant` on the
  first pass — `WorldState` at 488 bytes vs. `SaveData` at 184 — fixed by
  boxing both).
- **Not verified in this environment**: no `godot4` CLI available this
  session, so the scene/extension change is unverified against a real
  Godot import or a real HTML-app `.zip` export — only "compiles and the
  Rust-side unit tests pass" is confirmed
  (`cartalith-porting-discipline`'s own carve-out). Criterion 7 needs a
  real save file and a side-by-side comparison against what the HTML app
  shows for it — owner-only until then.

**Remaining, all previously logged**: graph-driven orogeny,
`stampVolcanoesProvinces`, ocean currents, terrain wind deflection,
seasons, `world_structure` not yet exposed to the GDScript UI — the
stretch-goal deferrals `MVP_SCOPE.md` itself sanctions, plus the
owner-only items above and eyes on the Godot editor.

**Next**: expose `world_structure` to the GDScript UI now that both the
generate and load paths are real; or graph-driven orogeny, the largest
remaining unported subsystem.

## Phase 1 — World-Structure archetypes exposed to the GDScript UI (2026-08-13)

The engine side (`cartalith-engine::WorldParams::world_structure`) has
taken raw archetype knobs since the earlier World-Structure port; nothing
in the UI could reach it. `WorldGen` (`cartalith-godot`) gained
`generate_world_structure(seed, width_km, resolution, archetype)`, a
second entry point alongside `generate()` (kept as the plain,
World-Structure-disabled "Classic" path rather than overloading one method
with an optional/empty archetype string). A `match` on the archetype name
holds the five-preset `ARCHETYPES` table (reference HTML lines 2521-2526:
earth/supercontinent/archipelago/volcanic/rift, each a
`(continentality, fragmentation, tectonicEnergy, oceanDepth,
hotspotDensity)` tuple) verbatim — the name→knobs lookup lives here, in
the Rust boundary layer, not in GDScript
(`ARCHITECTURE.md`: "Godot computes nothing beyond layout. Anything you
could get numerically wrong belongs in Rust."). An unrecognized name
prints to console and returns `false` rather than silently falling back to
Classic.

`main.tscn`/`main.gd`: added a "World shape" `OptionButton`
(Classic/Earth/Supercontinent/Archipelago/Volcanic/Rift) above the
Generate button. `WORLD_SHAPES` maps its selected index to the archetype
string `generate_world_structure` expects; index 0 (Classic) routes to the
existing plain `generate()` call instead. The status label now also shows
the chosen shape after a successful generate.

- `cargo build/test/clippy --workspace`: all green/clean.
- **Not verified in this environment**: same `godot4` CLI carve-out as the
  previous entry — compiles, but the dropdown/generate flow hasn't been
  clicked through in a real editor or export this session.

**Remaining, all previously logged**: graph-driven orogeny (now the
*only* real numerical deviation left when an archetype is selected — see
the earlier World-Structure entry's own flag on `tectonicGraph`/
`buildOrogenyField`), `stampVolcanoesProvinces`, ocean currents, terrain
wind deflection, seasons — the stretch-goal deferrals `MVP_SCOPE.md`
itself sanctions, plus the owner-only items (Godot editor / device
verification) already flagged.

**Next**: graph-driven orogeny — the largest remaining unported subsystem,
and now the one piece of the World-Structure UI that doesn't match the
reference's actual landform shaping yet; or one of the smaller climate
deferrals (`stampVolcanoesProvinces`, ocean currents, wind deflection,
seasons).

## Phase 1 — `stampVolcanoesProvinces` ported, not yet the default (2026-08-13)

`classifyBoundaries`/`placeProvinceVolcanoes`/`stampVolcanoesProvinces`
(reference HTML lines 3507-3556) — the JS default (`state.volc.provinces:
true`) volcanism mode: clusters volcanoes into a handful of provinces
(75% arc/subduction along convergent boundaries, 15% rift along divergent
ones, 10% age-progressive hotspot chains along plate drift) instead of
`stamp_volcanoes_simple`'s uniform boundary dusting. New
`cartalith-terrain` functions, all three ported line-for-line including
RNG draw order — the tricky part: several call sites in the reference draw
multiple `rng()` values within one JS expression (e.g.
`placeSizedVolcano(x+(rng()*2-1)*6, y+(rng()*2-1)*6, rng, rng()*age)`,
three draws in one argument list). Rust can't borrow `rng: &mut Mulberry32`
mutably twice within one call expression the way that reads, so each draw
became its own `let` in the same left-to-right order JS's argument
evaluation would hit it — same numbers, different syntax.

**Ported, but `cartalith_engine::VolcanismParams::provinces` defaults to
`false`, not JS's own `true`** — a real, deliberate deviation, flagged
rather than silently taken (`cartalith-porting-discipline`'s own standard
for this). This environment has no JS runtime
(`PARITY_TESTING.md`'s extraction procedure needs one to run the reference
HTML and read back real numbers), so there's no way to extract golden
fixtures for a multi-branch, RNG-order-sensitive placement algorithm —
unlike `recompute_resistance_after_erosion` earlier this phase, a
hand-derived unit test isn't a credible substitute here. Shipping this as
the pipeline's silent default without golden verification would be
exactly the "looks reasonable" bar this project's discipline rejects. The
function is fully reachable (`p.volc.provinces = true`) for anyone who
wants to opt in and verify it independently; `golden_parity_carve.rs`'s
existing verified fixtures (which predate this port and implicitly assumed
simple-mode volcanism) stay valid at the `false` default.

Added `stamp_volcanoes_provinces_is_deterministic`
(`cartalith-terrain/tests/golden_parity_volc_craters.rs`) — explicitly
**not** a golden-parity test: same-seed-produces-same-output, output
actually differs from the untouched baseline, and stays within `[0,1]`.
Catches a future refactor breaking determinism or silently no-op'ing the
function; does not catch a wrong-vs-JS formula. That gap is real and
stays open until someone runs the reference HTML in a JS-capable
environment and extracts fixtures the way `PARITY_TESTING.md` describes.

- `cargo build/test/clippy --workspace`: all green/clean.
  `#[allow(clippy::approx_constant)]` needed on one site — the reference's
  own `6.283` literal (not `TAU`) for a per-province angle draw, kept
  exact per `cartalith-rust-conventions`.

**Remaining, all previously logged**: graph-driven orogeny, ocean
currents, terrain wind deflection, seasons — plus, new to this entry,
golden-verifying `stamp_volcanoes_provinces` itself once a JS runtime is
available, which also unblocks flipping `VolcanismParams::provinces` to
match JS's own default.

**Next**: graph-driven orogeny remains the largest unported subsystem; the
smaller climate deferrals (ocean currents, wind deflection, seasons) are
comparably-sized alternatives. Golden-verifying `stamp_volcanoes_provinces`
is worth flagging to the owner specifically — it's done except for the one
thing this environment structurally cannot do.

## Phase 1 — `deflectFlow` + terrain wind deflection ported, not yet the default (2026-08-13)

`deflect_flow` (reference HTML lines 5315-5357) — the generic flow-
deflection primitive `buildWind`'s terrain coupling and (not yet ported)
`computeOceanCurrent`'s coastline coupling both build on: the component of
a vector field pointing INTO a rising `block` field is reduced and
redirected tangentially along the block field's local contour, iterated
16 times with light blending so deflection propagates upstream of a ridge
rather than only appearing on top of it, then a gap/strait acceleration
pass from the block field's own Laplacian. New `cartalith-climate` public
function, reusable once ocean currents are tackled.

Wired into `build_wind`'s own `opts.elev` branch (reference HTML lines
5521-5535): mountains block/split flow, gaps/straits accelerate it,
followed by an elevation-band damping term (thin high-altitude air slows
near-surface flow). `simulate_weather` now always has the coarse elevation
array (`eh`) this needs on hand — it already computed one for temperature
lapse — so wiring it through was just a new `elev: Option<(&[f32], f64)>`
parameter on `build_wind` and a new `WeatherParams::terrain_wind_deflection`
gate.

**Ported, but defaults to `false`, not JS's own unconditional-since-v1.78
default** — same reasoning as `stampVolcanoesProvinces` two entries back,
and arguably a bigger deal here: this is a 16-iteration algorithm that
reshapes wind everywhere terrain exists, which cascades into every
downstream term in `simulate_weather` (evaporation, advection, orographic
rain). No JS runtime in this environment means no golden fixtures to
verify any of that cascade against, and this environment has no way to
confirm a subtle sign error or off-by-one in the 16-iteration loop
wouldn't silently pass every existing test while producing wrong rainfall
everywhere. Reachable via `p.climate.terrain_wind_deflection` for anyone
who wants to opt in and verify independently.

Added `deflect_flow_regression.rs` (`cartalith-climate/tests/`) —
explicitly **not** golden-parity: determinism across repeat runs, a
synthetic ridge test confirming flow actually bends upstream of a block
(and stays near-untouched far from one) as a basic physical sanity check,
and a `strength: 0, gap_k: 0` near-identity case. None of this confirms
the numbers match JS.

- `cargo build/test/clippy --workspace`: all green/clean. One
  `clippy::manual_clamp` fix (`.max(a).min(b)` → `.clamp(a,b)`, same
  values, no behavior change).
- `golden_parity_weather.rs`'s three existing fixtures needed a new
  `terrain_wind_deflection: false` field added to their `WeatherParams`
  literals — no other change, since `false` reproduces this port's
  pre-existing (undeflected) behavior exactly.

**Remaining, all previously logged**: graph-driven orogeny, ocean-current
SST folding (now blocked on `deflect_flow`'s own golden verification too,
since `computeOceanCurrent` reuses it), seasons — plus golden-verifying
`stamp_volcanoes_provinces` and now `deflect_flow`/terrain wind deflection,
both once a JS runtime is available.

**Next**: graph-driven orogeny remains the largest unported subsystem;
seasons is the smallest remaining climate deferral. Ocean-current SST
folding is now more tractable than before (its own `computeOceanCurrent`
reuses this entry's `deflect_flow`), but still needs coastline-distance
scanning and gyre logic on top. Golden-verifying the two RNG/iteration-
heavy algorithms ported this session (`stamp_volcanoes_provinces`,
`deflect_flow`) against real JS output is worth flagging to the owner as
its own piece of work — both are functionally complete and blocked purely
on this environment lacking a JS runtime.

## Phase 1 — experimental subsystem toggles exposed to the GDScript UI (2026-08-13)

The three formulas ported this session but kept off by default
(dynamic lithology, clustered volcanism/provinces, terrain wind
deflection) had no way to reach the UI at all — the only way to exercise
them was a hand-edited `WorldParams`. `WorldGen` gained
`set_experimental_flags(dynamic_lithology, volc_provinces,
terrain_wind_deflection)`, applied by both `generate()` and
`generate_world_structure()` before calling `generate_terrain`; `main.tscn`
gained an "Experimental (unverified vs. the HTML app)" section with one
checkbox per flag, read at Generate-press time.

**Why now, and why exposed at all despite being unverified**: this
environment structurally cannot golden-verify these three (`stamp_volcanoes_
provinces`, `deflect_flow`/wind deflection, `recompute_resistance_after_
erosion`'s gate) against real JS — no JS runtime here to run the reference
HTML. But the *owner's* machine can run that reference HTML directly. Real
UI checkboxes turn "someone needs to write a golden-fixture-extraction
harness" into "toggle this on, generate the same seed in both the Godot
build and the HTML app, and look" — a much lower bar, and the fastest
realistic path to closing this port's actual outstanding verification gap.
Labeled "unverified" explicitly, not left implicit.

- `cargo build/test/clippy --workspace`: all green/clean.
- Also fixed two stale doc comments in `cartalith-godot::WorldGen`
  (`generate`/`generate_world_structure`) still saying World-Structure
  archetypes were "not yet exposed to this UI" — they have been since two
  entries back.
- **Not verified in this environment**: same `godot4` CLI carve-out as
  every Godot-side change this session — compiles, but the three new
  checkboxes haven't been clicked in a real editor or export.

**Remaining, all previously logged**: graph-driven orogeny, ocean-current
SST folding, seasons — plus golden-verifying the three experimental flags
now exposed, which is squarely an owner task from here (needs the actual
HTML app to compare against, not just a JS runtime to run it headless).

**Next**: graph-driven orogeny or seasons for further subsystem ports;
otherwise this phase's remaining open items are largely owner-side
(Windows/Android device verification per `MVP_SCOPE.md` criteria 3-4, a
real Godot editor pass, and now side-by-side experimental-flag comparison
against the HTML app).

## Phase 1 — ocean-current SST folding ported, not yet the default (2026-08-13)

The largest of this phase's subsystem ports. Three new `cartalith-climate`
functions:

- `compute_ocean_current` (reference HTML `computeOceanCurrent`, lines
  5368-5462): a genuine 2D ocean-current vector field — Ekman-rotated
  (~25° right of wind in the N hemisphere, left in the S) from the
  terrain-deflected wind, run through `deflect_flow` again against a HARD
  coastline, a continental-shelf friction term, and a western-
  intensification heuristic (subtropical gyres pile transport on a
  basin's western edge — a distance-to-coast proxy, not a solved
  beta-plane model, disclosed as such in both the reference and here).
- `ocean_sst_anomaly` (reference HTML `oceanSSTAnomaly`, lines 5246-5268):
  builds a synthetic zonal-mean wind field via `build_wind`, runs it
  through `compute_ocean_current`, and derives a coarse-grid SST anomaly
  (poleward currents warm, equatorward currents cold-upwell) clamped to
  ±8°C and blurred.
- `apply_ocean_currents` (reference HTML `applyOceanCurrents`, lines
  5270-5288): the post-hoc half — folds the anomaly directly into ocean
  `temperature`, and (coast-proximity-weighted) into nearby land
  `temperature`/`rainfall`.

Wired at both places JS reads `state.climate.currents`:
`simulate_weather`'s own loop 2 (folds the anomaly into `tc`/`sst_evap`
*before* `build_wind` runs, closing the currents→SST→pressure→wind→
rainfall loop — new `WeatherParams::currents`/`current_k` fields) and
`generate_terrain`'s two `refreshClimate()`-equivalent points, right after
`apply_climate_moisture_correctors` (new `ClimateInputParams::currents`/
`current_k`, matching reference HTML line 8783's
`computeFlow(true); refreshClimate();` inside `carveRiverValleys`).

**Ported, but defaults to `false`, not JS's own `true`** — JS ships ocean
currents ON by default ("cheap, integrated into the weather sim before
buildWind"); this port keeps the same off-by-default posture as
`stampVolcanoesProvinces`/terrain wind deflection and for the same
reason: no JS runtime in this environment to golden-verify a
`deflect_flow`-based algorithm with its own additional unverified
heuristic (western intensification) layered on top. Reachable via
`p.climate.currents` (engine) and `p.currents` (`WeatherParams`
directly).

Added `ocean_current_regression.rs` — explicitly **not** golden-parity:
determinism across repeat runs, zero-on-land invariants, and an SST
anomaly bounds check (`[-8,8]`). None of this confirms the numbers match
JS.

Extended `cartalith-godot::WorldGen::set_experimental_flags` to a fourth
flag (`ocean_currents`) and added a matching "Ocean-current SST folding"
checkbox to `main.tscn`'s experimental section, same reasoning as the
other three — the owner's machine can run the real HTML app and close
this port's actual verification gap; this dev environment cannot.

- `cargo build/test/clippy --workspace`: all green/clean. Two more
  `clippy::needless_range_loop` allows (multi-array-indexed row scans,
  same precedent as `cartalith-erosion`) and one more `manual_clamp` fix.
- `golden_parity_weather.rs`'s three fixtures needed `currents: false,
  current_k: 1.0` added to their `WeatherParams` literals — no behavior
  change, `false` reproduces this port's pre-existing behavior exactly.

**Remaining, all previously logged**: graph-driven orogeny, seasons — plus
golden-verifying all four experimental flags now exposed (dynamic
lithology's gate, `stamp_volcanoes_provinces`, terrain wind deflection,
ocean currents), squarely an owner task needing the actual HTML app to
compare against.

**Next**: graph-driven orogeny is the last large unported subsystem;
seasons the last smaller one. Beyond that, this phase's open items are
owner-side: Windows/Android device verification (`MVP_SCOPE.md` criteria
3-4), a real Godot editor pass, and side-by-side experimental-flag
comparison against the HTML app.

## Phase 1 — graph-driven orogeny, part 1: T1 boundary polyline graph (2026-08-13)

Graph-driven orogeny (`state.tect.tectonicGraph`) is this port's largest
remaining subsystem — split at the reference's own T1/T2+T3 boundary
rather than ported in one pass. This entry is T1 only: turning the
per-cell boundary mask into vector polylines T2+3 can grow features
along.

- `thin_mask` (reference HTML `thinMask`, lines 2888-2909): Zhang-Suen
  thinning, reducing a (possibly 2-cell-thick) boundary mask to a 1-pixel
  skeleton.
- `trace_boundaries` (reference HTML `traceBoundaries`, lines 2921-2952):
  walks the thinned skeleton into polylines — chains between nodes
  (degree != 2: endpoints, junctions) and pure loops (all degree-2, no
  node) traced separately. Returns both the polylines (with
  `poly_meta`'s arc-length/curvature/closed-loop metadata,
  `_polyMeta` at lines 2910-2920) and the junction list.

**Genuinely golden-verifiable this time, unlike the last three ports**:
this stage is pure topology/integer logic (no RNG, no float-precision-
sensitive iteration) — a `thin_mask`/`trace_boundaries` unit test IS a
credible parity check, not a "looks reasonable" substitute, because the
algorithm is small and deterministic enough to hand-trace exactly.
Verified straight-line tracing by hand (a 5-cell line stays a fixed point
under thinning — endpoints are always kept, and interior cells have
exactly 2 opposite 1-neighbors, which is 2 separate ring transitions, not
the 1 Zhang-Suen requires for deletion — then traces to one 5-point
chain with zero curvature) and a 2-cell mask's own real quirk: JS's
`traceBoundaries` never marks a walk's *starting* cell visited, so a
direct edge between two nodes gets recorded twice, once per endpoint —
ported as-is and locked in by its own test, not silently deduplicated
(`cartalith-porting-discipline`: that would be an improvement over JS,
which needs a logged decision, not a silent fix made while porting). A
first attempt at a "pure loop, no junctions" hand-built test case (a 3x3
ring) turned out to be wrong by hand-calculation — 8-connected corner
cells on a 1-cell-thick ring are diagonally adjacent to the *other* edge
meeting at that corner, creating real degree>=3 junctions there, which
is correct algorithm behavior, not a bug; dropped that test rather than
assert a wrong expectation, since constructing a genuinely junction-free
loop by hand needs a much larger, gently-curved shape impractical to
hand-verify pixel-by-pixel.

- `cargo build/test/clippy --workspace`: all green/clean, 8 new
  `cartalith-terrain` unit tests.

**Remaining**: T2+T3 (`buildOrogenyField`, the per-boundary-type
signed-distance-field kernel stamping — collision fold ripples,
subduction trench+arc, rift graben+shoulders) and wiring into
`generate_terrain` behind `world_structure`'s existing `tectonicGraph`
deviation flag. Once wired, this also closes that flag's own logged gap
(World-Structure worlds currently get the older convergent-stress "blob"
uplift instead of structured per-margin orogeny). Seasons and the
owner-side items from the previous entry are still open too.

**Next**: T2+T3 — the actual landform-shaping kernels this graph feeds.

## Phase 1 — graph-driven orogeny, part 2: T2+T3 kernels ported and golden-verified (2026-08-15)

**Node.js is now available in this environment** (installed this session,
v24.19.0, matching the version every prior golden extraction in this
CHANGELOG already cites) — the "no JS runtime here" caveat several earlier
entries logged for deferring golden verification
(`stampVolcanoesProvinces`, ocean-current SST folding, terrain wind
deflection) no longer applies to future sessions on this machine.

`tag_boundary_types` (`currentBoundaryGraph`'s per-polyline dominant-type
majority vote, reference HTML lines 2957-2962, inlined in JS rather than
its own function) and `build_orogeny_field`/`smooth_orogeny`
(`buildOrogenyField`/`smoothOrogeny`, T2+T3, reference lines 2981-3080) —
the per-boundary-type signed-distance-field kernels (collision multi-ridge
fold, subduction/arc trench+arc, rift graben+shoulders+optional
Basin-and-Range fault blocks, transform shear-driven fault valley) that
`buildOrogenyField`'s own header comment describes, stamped along each
typed margin polyline and combined by `|max|` so junctions don't
double-stack.

**Golden-verified**: extracted by running the actual JS `buildOrogenyField`/
`smoothOrogeny` under real Node.js (a `vm`-sandboxed load of every
`<script>` block in the reference HTML, with permissive stubs for
DOM/timer APIs so top-level init code doesn't throw or hang the process --
sanity-checked first against `mulberry32`'s already-golden values before
trusting it for anything new) against a synthetic 20x14 grid, one polyline
per boundary type, deterministic non-random stress/crust/shear fields.
5 cases (collision, subductionOC, arcOO, rift with `faultBlockK>0` to
exercise the Basin-and-Range branch, transform with a real shear field),
2 also checked through `smooth_orogeny`. All bit-for-bit exact.

**Two real precision-order bugs caught by the first test run, both the
same class**: JS declares `dist`/`side` as `Float32Array` inside
`buildOrogenyField` — every store narrows to f32, so a later read is the
*rounded* value widened back to f64, not the full-precision f64 that
produced it. A first pass at this port kept both as `Vec<f64>` (no
narrowing until the final `U` store), and separately narrowed the final
`|max|`-combine comparison's `v` to f32 *before* comparing magnitudes
instead of after, matching neither of JS's two separate narrowing points.
Both fixed to narrow exactly where JS's own `Float32Array` assignments do
and nowhere else (`cartalith-rust-conventions`: match JS's float precision,
don't improve on it) — the bit-exact test results above are after both
fixes; the first attempt failed at roughly the 7th significant digit,
small enough to look like "close enough" and easy to wave through without
a golden test catching it.

`tag_boundary_types` is intentionally **not** golden-tested via Node: it
depends on `boundaryMask`/`boundaryType`/`GW`/`GH` globals declared `let`
at the reference HTML's top level, which never attach to a Node `vm`
context object (a real, load-bearing quirk of that extraction technique,
not a bug in the harness) — small enough, pure counting + argmax over a
fixed 6-entry array, to hand-verify instead, same class of "small enough
to hand-trace exactly" T1's `thin_mask`/`trace_boundaries` tests already
used. Covers a clean majority and JS's strict-`>` tie-break behaviour
(lower boundary-type id wins a tie, not the last one scanned).

- `cargo test --workspace`: all green, including this entry's new
  `golden_parity_orogeny.rs` (5 tests) and one new `cartalith-terrain`
  unit test. `cargo clippy --workspace --all-targets`: clean.

**Remaining, unchanged from the previous entry**: wiring into
`generate_terrain` behind `world_structure`'s existing `tectonicGraph`
deviation flag. Everything the kernels themselves need already exists on
the tectonic-substrate result (`boundary_mask`, `boundary_type`,
`stress_field`, `shear_field`) — the one missing piece is a
`plate_crust`-equivalent helper (JS `plateCrust()`, reference HTML: builds
a per-cell `plates[plateId[i]].base` array) to feed `build_orogeny_field`'s
ocean-side vote. Deliberately left as its own step rather than folded into
this commit, matching this CHANGELOG's own established rhythm of "ported,
not yet wired" as a separate unit from "wired in" (`stampVolcanoesProvinces`,
`deflectFlow`, ocean currents all did this too) — easier to review, and a
wiring bug can't be confused for a kernel bug if they land separately.

**Next**: `plate_crust` helper, then wire `tag_boundary_types` +
`build_orogeny_field` + `smooth_orogeny` into `generate_terrain`'s
World-Structure section, closing the gap that section's own doc comment
already flags.

## Phase 1 — graph-driven orogeny wired in: T2+T3 gap closed (2026-08-15)

No `plate_crust` helper needed after all — `generate_terrain` already
computes `base_raw` (`plate_id.iter().map(|&pid| plates[pid].base as f32)`)
for `base_field`'s own gaussian blur, which is exactly JS's `plateCrust()`
output before the smoothing step JS never applies to it either. Reused
directly rather than adding a duplicate helper computing the same array a
second way.

Also found, while tracing where `oro` needed to land, that `compute_height`
**already** had an `oro: Option<&[f32]>` parameter matching JS's
`T=oro?oro[i]+Math.min(sf,0):sf` exactly (`fillHeightRows`) — always called
with `None` at the one call site. So the height-formula combination logic
was already correct and already golden-covered by every earlier
`compute_height` test (they all pass `oro: None`, exercising the `else`
arm); wiring this stage down to "compute a real `oro` and pass `Some(&oro)`
instead of `None`" rather than also porting a combination formula.

Wired into `generate_terrain`, gated on `p.world_structure.enabled` (the
only trigger this port models, matching JS's own only caller of
`tectonicGraph=true`): `trace_boundaries` on `stress.boundary_mask` →
`tag_boundary_types` using `stress.boundary_type` → `build_orogeny_field`
(stress + `base_raw` as crust + `stress.shear_field`) → `smooth_orogeny` →
fed to `compute_height` as `Some(&oro)`. `foldK`/`trenchK`/`faultBlockK`
hardcoded to JS's own null-coalescing defaults (`0.16`/`1.0`/`0`) since
`foldIntensity`/`trenchDepth`/`faultBlock` aren't exposed as configurable
params anywhere in this port yet — noted in the module doc comment, not
silently assumed.

- `cargo test --workspace`: all green, no golden test's expected values
  changed (`oro` stays `None` unless `world_structure.enabled`, so every
  non-World-Structure golden fixture is untouched by construction).
  `generate_terrain_world_structure_shapes_land_fraction`
  (`cartalith-engine`) now exercises the real orogeny path for the first
  time — previously `world_structure.enabled` still left `oro` at `None`
  regardless — and still passes, including its Archipelago-has-less-land-
  than-Supercontinent assertion. `cargo clippy --workspace --all-targets`:
  clean (one `doc_lazy_continuation` false-positive from a doc-comment line
  starting with `+ ` that rustdoc's markdown parser read as a list bullet
  — reworded, not suppressed).

This closes the last item `MVP_SCOPE.md` point 1 (World-Structure
archetypes) and the module's own doc comment were flagging as a real,
deliberate deviation from JS rather than a no-op-at-defaults case.
World-Structure worlds generated by this port now get JS's actual
structured per-margin orogeny, not the older convergent-stress "blob"
uplift.

**Remaining**: seasons and the owner-side items already logged (a real
`.zip` export, eyes on the Godot editor) are the only items left on this
CHANGELOG's own running "remaining" list. `stampVolcanoesProvinces`,
ocean-current SST folding, and terrain wind deflection are ported but
still off-by-default pending golden verification — no longer blocked on
"no JS runtime here" (Node is installed now, see the previous entry), just
not yet done.

## Phase 1 — extraction harness upgrade + stampVolcanoesProvinces golden-verified (2026-08-15)

**A real harness upgrade, not just one more subsystem.** The orogeny
extraction technique (a Node `vm` sandbox with DOM/timer stubs) only
worked because `buildOrogenyField`/`smoothOrogeny` happen to be pure --
every input as an explicit parameter, per their own doc comments.
`stampVolcanoesProvinces` and its own helpers (`placeSizedVolcano`,
`placeProvinceVolcanoes`, `classifyBoundaries`) are **not** pure -- they
read `GW`/`GH`/`state`/`field`/`boundaryMask`/`stressField`/`plateId`/
`plates` directly as globals, which the previous harness couldn't reach at
all: `vm.runInContext` attaches top-level `function`/`var` bindings to the
sandbox's context object, but **not** top-level `let`/`const` bindings --
they live in an unreachable script-scope environment instead, a genuine
`vm` quirk, not a bug in the harness itself. The reference HTML declares
`GW`, `state`, `boundaryMask`, etc. with `let`/`const` at the top level.

Fixed by rewriting *only zero-indent* (genuinely top-level, not
function-body-local) `let`/`const` to `var` before `vm` execution -- a
regex on the concatenated `<script>` source, not a real parser, but this
file's own convention of never indenting top-level declarations makes it
safe: block-scoped/closure-captured locals inside functions (where
let-vs-var actually changes semantics) are never zero-indent, so they're
untouched.

**This unlocked something bigger than reaching a few more globals**: with
`GW`/`GH`/`state`/`allocate` all reachable, it's now possible to set a
small grid (20x14, matching this project's own stage-test convention),
call the actual unmodified top-level `generate()`, and let the real
pipeline populate every field from a real run -- rather than hand-building
synthetic inputs per function the way the orogeny extraction had to.
`Worker` was left `undefined` rather than stubbed: this port's own
architecture only targets the sync/no-worker-pool fallback (no browser
worker pool in Rust), and the reference app's own `typeof Worker==='undefined'`
feature-detection is exactly how it's meant to reach that same path --
stubbing `Worker` would make the app *think* workers are available and
hang waiting on a `postMessage` callback that never fires. (Also worth
recording: the reference app's actual default grid is 2048x1311 -- the
full sync pipeline at that size is why an early attempt at this looked
like a hang and wasn't; it just wasn't going to finish in this session.
Set `GW`/`GH` small and call `allocate()` before `generate()`.)

**`stampVolcanoesProvinces` verified by monkey-patching it**, not by
calling it in isolation with hand-built inputs: replaced the sandbox's own
`stampVolcanoesProvinces` with a wrapper that snapshots `field`/
`boundaryMask`/`stressField`/`plateId`/`plates` immediately before calling
the real original function, then snapshots `field`/`volcanicField`
immediately after -- so every input the golden test uses is something the
real `generate()` pipeline genuinely produced at seed 42, not a
hand-constructed approximation of what it might produce. Fed those exact
captured values into `stamp_volcanoes_provinces`
(`golden_parity_volc_provinces.rs`) and asserted against the captured
output. **Bit-exact on the first attempt** -- no precision-order bug this
time, unlike orogeny's two.

- `cargo test --workspace`: all green. `cargo clippy --workspace
  --all-targets`: clean.

**Deliberately not flipping `WorldParams::defaults`'s `volc.provinces` to
match JS's own `true` default**, even though the function itself is now
golden-verified: the height field it produces feeds every downstream
stage, and `golden_parity_carve.rs`/`golden_parity_pipeline.rs`'s existing
fixtures were captured against the `stamp_volcanoes_simple` path this
default currently selects. Flipping the default would silently invalidate
those without also re-extracting them -- a real, separate unit of work,
not a side effect of verifying one function. Noted in
`WorldParams::defaults`'s own doc comment rather than done quietly.

**This harness upgrade is the more valuable output of this entry** --
ocean-current SST folding and terrain wind deflection (both real,
non-pure, global-reading JS functions, same class as
`stampVolcanoesProvinces`) can use the exact same monkey-patch-and-capture
technique next, and so can anything else this port ever needs to verify
against a real `generate()` run, not just a hand-built synthetic case.

**Next**: ocean-current SST folding or terrain wind deflection, same
technique. Eventually: re-extract `golden_parity_carve.rs`/
`golden_parity_pipeline.rs` fixtures with `volc.provinces: true` and flip
that default to match JS for real, once enough of the "ported but
off-by-default" list is cleared that it's worth doing once instead of
per-subsystem.

## Phase 1 — terrain wind deflection golden-verified (2026-08-15)

`deflect_flow` (`cartalith_climate`) turned out to be pure -- every input
explicit, same as `build_orogeny_field` -- so this didn't need the
let-to-var harness upgrade at all, just the original orogeny-style
extraction: synthetic 16x12 `u0`/`v0`/`block0` fields (a sine/cosine flow
crossing a diagonal ridge), called `deflectFlow` directly under Node with
three cases (default opts, world-wrap, custom knobs). **Bit-exact on the
first attempt**, all three cases.

`build_wind`'s own wiring around `deflect_flow` (the `block` field's
`land`/`mtn` terms, the hardcoded `DeflectFlowParams` constants, the
elevation-band damping combine after) was checked line-for-line against
reference HTML lines 5521-5535 rather than golden-extracted separately --
`build_wind` itself isn't pure (reads several fields as part of a larger
weather-simulation loop this port hasn't fully isolated for Node
extraction yet), but this one block is small and mechanical enough that
direct comparison is credible, the same reasoning `compute_height`'s
`oro` combination formula got when the orogeny wiring landed.

- `cargo test --workspace`: all green (`golden_parity_deflect_flow.rs`, 3
  new tests). `cargo clippy --workspace --all-targets`: clean.

**Still `false` by default**, same reasoning as `stampVolcanoesProvinces`:
verifying the function doesn't make flipping the pipeline-wide default
free, since it'd invalidate `golden_parity_weather.rs` and everything
built on `simulate_weather` without also re-extracting those.

**Remaining on the "ported but off-by-default" list**: only ocean-current
SST folding now, and unlike the other two, it isn't just a
fixture-invalidation-caution case -- `compute_ocean_current`'s
western-intensification heuristic hasn't been checked against JS at all
yet, kernel-level or otherwise (see `WeatherParams::currents`'s own doc
comment).

**Next**: ocean-current SST folding — the last item on this list, and the
only one that's genuinely unverified at the kernel level, not just
withheld from the pipeline default.

## Phase 1 — ocean-current SST folding golden-verified: the last "unverified" item closed (2026-08-15)

`compute_ocean_current` turned out pure too (all inputs explicit) — same
orogeny-style extraction as `deflect_flow`, two cases (World mode with
wrap, Region mode with the western-intensification heuristic off).
**Bit-exact on the first attempt, including the heuristic itself** — its
own doc comment already discloses it as "a distance-to-coast proxy, NOT a
solved beta-plane model," and this confirms that disclosed proxy is
actually ported correctly, not merely disclosed as an approximation while
secretly also being wrong.

`ocean_sst_anomaly`/`apply_ocean_currents` are the two remaining pieces,
and JS's versions of both read several globals directly
(`state.climate`/`GW`/`GH`/`field`/`geoidField`) where this port's own
signatures parameterize everything — extracting them via Node would need
the generate()-driving technique `stampVolcanoesProvinces` used. Checked
by direct line-for-line comparison instead (reference HTML lines
5246-5288): both are short, and every function they call
(`build_wind`, `compute_ocean_current`, `gauss_blur`) is now independently
golden-verified, so comparison is credible here the same way it was for
`compute_height`'s `oro` combination and `build_wind`'s own terrain-deflect
wiring. One deliberate, already-disclosed gap found: JS reads
`field[i]-geoAt(i)` where this port reads plain `field[i]` — correct at
`state.planet.geoid.enabled`'s default `false` (geoid correction is zero
there), same reasoning `compute_temperature` already documents for the
same omission.

- `cargo test --workspace`: all green (`golden_parity_ocean_current.rs`, 2
  new tests). `cargo clippy --workspace --all-targets`: clean.

**This closes the "ported but off-by-default" list entirely.**
`stampVolcanoesProvinces`, terrain wind deflection, and ocean-current SST
folding are all now verified — the first at full pipeline fidelity
(monkey-patched into a real `generate()` run), the other two at the
kernel level plus direct-comparison wiring checks. All three remain
`false` in `WorldParams::defaults`/`WeatherParams`, deliberately: each
would change a field every downstream stage reads, and the existing
`golden_parity_carve.rs`/`golden_parity_pipeline.rs`/
`golden_parity_weather.rs` fixtures were captured with all three off.
Flipping any of them for real is its own future unit of work — re-extract
those fixtures with the new defaults, not a quiet side effect of
verifying the functions themselves.

**Next**: with the pipeline itself now fully ported and every previously
"unverified" stretch subsystem closed, the remaining open items are the
owner-only ones already logged repeatedly in this CHANGELOG (a real
`.zip` export, eyes on the Godot editor and a real device) — and, if
pursued, the bigger fixture-re-extraction pass to flip the three
now-verified defaults to match JS for real. Everything past that is
Phase 2+ (`ROADMAP.md`): civilisation, urban morphology, asset library —
out of this port's current scope, not merely undone.

## Phase 1 — volc.provinces flipped to match JS; Godot UI defaults follow (2026-08-15)

Started the fixture-re-extraction pass the previous entry flagged as
future work, scoped to just `volc.provinces` (the cleanest of the three —
`golden_parity_pipeline.rs` is bit-exact and stops before climate, so it's
the only fixture this flip actually touches; `terrain_wind_deflection`/
`currents` both feed climate and would also touch
`golden_parity_carve.rs`/`golden_parity_weather.rs`, left for a dedicated
pass rather than folded in here).

`WorldParams::defaults`'s `volc.provinces` is now `true`, matching JS.
`golden_parity_pipeline.rs` re-captured by monkey-patching `computeFlow`
to snapshot every field it checks immediately after its first (area) call
then aborting before climate/carve ever run — same technique
`golden_parity_volc_provinces.rs` used, not a hand-stripped
re-derivation. `plate_id`/`boundary_mask`/`stress_field`/`flexure_field`/
`age_field`/`heterogeneity_field`/`resistance_field` came back
bit-identical to the previous fixture, as expected (all computed before
volcanism runs); only `field`/`volcanic_field`/`impact_field`
(volcanism's own writes) and `flow_area` (reads the post-volcanism field)
actually changed.

`golden_parity_carve.rs` still assumes the old default (it also implicitly
assumes `terrain_wind_deflection`/`currents` off, so re-extracting it
belongs with flipping those too) — pinned with an explicit
`p.volc.provinces = false` override rather than left to silently break,
with a comment explaining why it's pinned and what unpins it.

**Also flipped `cartalith-godot::WorldGen`'s own defaults** (independent
of `WorldParams::defaults` — every `WorldGen::generate()`/
`generate_world_structure()` call already overrides all four experimental
flags explicitly, so this doesn't touch any golden fixture): `volc_provinces`/
`terrain_wind_deflection`/`ocean_currents` now default `true`
(`dynamic_lithology` stays `false` — that's JS's own real default, not an
unverified-so-off case like the other three). Updated the Godot UI to
match: the three checkboxes now start checked, the section header changed
from "EXPERIMENTAL FEATURES" to "ADVANCED FEATURES", and the hint label no
longer claims they're unverified, since they aren't anymore. This is the
most user-visible form of closing the gap — a fresh `WorldGen` now
produces output matching the real HTML app's actual defaults, not the
conservative unverified-so-off state this port carried through most of
Phase 1.

- `cargo test --workspace`: all green, including the re-extracted
  `golden_parity_pipeline.rs` and the pinned `golden_parity_carve.rs`.
  `cargo clippy --workspace --all-targets`: clean. `godot4 --headless
  --import .` / `--quit main.tscn`: clean, extension still loads.

**Remaining**: re-extract `golden_parity_carve.rs`/
`golden_parity_weather.rs` with `terrain_wind_deflection`/`currents` also
flipped to `true` in `WorldParams::defaults`/`WeatherParams` — the last
piece of making the pipeline-level defaults fully match JS, not just the
Godot-facing `WorldGen` wrapper. Both fixtures are tolerance-based
(`golden_parity_carve.rs` already documents why: accumulated `Math.hypot`
noise across a long pipeline) and touch climate/erosion, so this is a
bigger, more careful pass than today's — genuinely separate work, not
mechanically the same as this entry.

## Phase 1 — terrain_wind_deflection/currents flipped to match JS (2026-08-15)

Finished what the previous entry deferred. `WorldParams::defaults`'s
`climate.terrain_wind_deflection`/`climate.currents` are now `true`,
matching JS (wind deflection unconditional since v1.78; `state.climate.currents`
defaults `true`).

Added a new `simulate_weather` test case first
(`golden_parity_weather.rs::simulate_weather_currents_case`) to prove the
`terrain_wind_deflection: true, currents: true` wiring together, end to
end, before touching any default: captured by driving the real reference
`generate()` under Node (`state.climate.currents = true`, `wIters = 5` for
speed) and monkey-patching `simulateWeather` to snapshot its real inputs
(`field` plus every `state.climate`/`state.planet` value the JS function
reads) immediately before the call, and `rainField` immediately after --
same technique `golden_parity_volc_provinces.rs` established. One real
naming trap caught along the way: JS's actual fields are `windDir` (not
`windDirDeg`) and a `windMode` string (`'auto'`/`'manual'`), not a
`windManual` boolean directly -- the port's own `wind_manual`/`wind_dir_deg`
params are a derived, cleaner shape than JS's own state layout, not a
literal field-for-field mirror. **Passed at the existing `1e-5` tolerance
on the first attempt.**

Flipping the actual `WorldParams`/`WeatherParams` defaults touches only
`golden_parity_carve.rs` (already pinned, now to all three flags with an
updated comment explaining why and what it's still owed) -- audited every
other `WorldParams::defaults` caller first (`golden_parity_pipeline.rs`
doesn't reach the climate stage at all; `cartalith-godot::WorldGen`
already overrides all four explicitly at every call site, same as before).

- `cargo test --workspace`: all green, including the new weather test
  case. `cargo clippy --workspace --all-targets`: clean. `godot4
  --headless --import .` / `--quit main.tscn`: clean.

**This is the pipeline-level default flip fully done.** `WorldParams::defaults`
now matches JS's real defaults for all three subsystems this port spent
today verifying. The one remaining piece is exactly what it's been since
the volc.provinces entry: re-extract `golden_parity_carve.rs` itself from
scratch with all three flags on (it's still pinned to the old,
all-three-off state) -- a genuinely separate, more careful pass given its
own tolerance-based, full-pipeline-through-carved-rivers scope, not
mechanically the same as adding one new isolated test case the way today's
`currents_case` was.

## Phase 1 — golden_parity_carve.rs re-extracted: the fixture-flip pass is done (2026-08-15)

Closed the one remaining gap. `carveRiverValleys()`'s own step 3
(reference HTML line 8784: `computeFlow(true); refreshClimate();`) meant
this capture needed no monkey-patch or early-abort at all, unlike every
other fixture regenerated today -- carving is the last stage that touches
any field this test checks, so the real reference `generate()` could just
run to completion under Node and `field`/`tempField`/`rainField`/
`flowField`/`riverMask` read directly off the sandbox afterward.

Both cases (`gw=14 gh=11 seed=24601 world=false`,
`gw=16 gh=12 seed=314159 world=true`) passed at the existing tolerance
(`1e-4` atol+rtol, `river_mask` exact) **on the first attempt** -- the
full pipeline (tectonics through carved rivers, with clustered volcanism,
terrain-deflected wind, and ocean-current-coupled climate all genuinely
active together for the first time in any test) matches JS end to end.
No test-body overrides needed anymore either: this is what a plain
`WorldParams::defaults` run actually produces now.

- `cargo test --workspace`: all green. `cargo clippy --workspace
  --all-targets`: clean. `godot4 --headless --quit main.tscn`: clean.

**This closes the fixture-re-extraction arc started three entries ago.**
Every golden test in this port now reflects `WorldParams::defaults`'s
real, JS-matching values -- no fixture anywhere is still pinned to a
stale pre-flip default. Combined with the earlier entries: graph-driven
orogeny ported and wired, all three previously-"unverified" stretch
subsystems golden-verified, and now every fixture that touches any of
them re-extracted against their real defaults rather than left assuming
the conservative all-off state this port carried through most of Phase 1.

**Where this leaves `MVP_SCOPE.md`'s "done means all seven" checklist**:
six of seven are satisfied. The seventh (`.apk` installed and run on a
real device) needs the owner's own hardware -- nothing left in engine
scope blocks it. Past that, remaining work is Phase 2+ (`ROADMAP.md`):
civilisation, urban morphology, asset library -- out of this port's
current scope until raised and scoped properly, not merely undone.

## Phase 0 close-out, part 2 — the Android `.apk` builds and packages on real hardware (2026-08-15)

This session had real `ANDROID_NDK_HOME`/`ANDROID_SDK_ROOT` access on the
owner's own Windows machine (set up earlier this session, NDK
`29.0.14206865` -- exactly what Godot 4.7.1 pins) -- something no earlier
session had. Worth actually using rather than leaving the Android side at
"blocked" from the original cloud sandbox.

`cargo ndk -t arm64-v8a build -p cartalith-godot`: succeeded, produced a
real `target/aarch64-linux-android/debug/libcartalith_godot.so` (220MB).
`godot4 --headless --export-debug "Android" builds/android/Cartalith.apk`:
succeeded -- added every resource, aligned, signed with the debug
keystore, verified. Confirmed both `lib/arm64-v8a/libgodot_android.so`
(Godot's own runtime) and `lib/arm64-v8a/libcartalith_godot.so` (this
port's extension) are genuinely inside the packaged `.apk`, not just that
the export command exited 0 -- same "unzip and look" discipline
`TOOLCHAIN.md` already prescribes for exactly this kind of claim.

**Confirmed: builds and packages. Not confirmed: installed and run on a
real device** -- that's the one half of `MVP_SCOPE.md` criterion 4 no
session can do from a terminal, `DECISIONS.md` §5's own carve-out. The
owner already reached this exact wall once before this session (via the
Godot Editor Android app, missing `.so` at the time) -- with a real
`.apk` in hand now, sideloading it directly is simpler than repeating
that path.

**This closes every part of Phase 0/Phase 1 reachable without the
owner's phone in hand.** Both platforms now build and package
end-to-end; Windows has owner confirmation already (`ping()` round-trip,
earlier in this CHANGELOG), Android is one sideload away from the same.

## Android emulator investigation — a real, reproducible boot stall, but SwiftShader's, not ours (2026-08-15)

Tried the AVD emulator ("OP12", arm64, API level matching the export
preset) as the closest technically-reachable proxy for "install and run"
without the owner's own phone. Installed cleanly (`adb install`), process
launches and survives (`pidof` stays populated, `ActivityTaskManager:
Displayed ... +2s14ms`), no crash. A screenshot (`adb shell screencap`)
shows a flat gray screen -- no triangle, no theme, no controls, nothing.

Root-caused, not just observed. Cleared logcat and did a fresh
force-stop/relaunch twice to rule out stale-buffer artifacts -- both
captures are identical and short (29 lines under the `godot` tag,
consistently). The full sequence, every time:

1. `Godot Engine v4.7.1.stable.official...`
2. `WARNING: Failed to load cached shader, recompiling.`
3. `ERROR: SceneShaderGLES3: Program linking failed: Fragment shader
   active uniforms exceed GL_MAX_FRAGMENT_UNIFORM_VECTORS (261)`
4. `OpenGL API OpenGL ES 3.0 (OpenGL ES 3.0 SwiftShader 4.0.0.1) -
   Compatibility - ... (Google SwiftShader)`
5. Two more `CanvasShaderGLES3: Program linking failed` (same uniform-limit
   error), then **nothing further, ever** -- not another log line of any
   kind, across two independent clean captures.

`SceneShaderGLES3`/`CanvasShaderGLES3` are Godot's own built-in engine
shaders, not this port's -- they'd fail identically for a stock, unmodified
Godot project on this same emulator. SwiftShader (the emulator's
CPU-software GL implementation, not a real GPU) reports a
`GL_MAX_FRAGMENT_UNIFORM_VECTORS` of 261, which is below what Godot's
Compatibility renderer needs to link its own default shaders. Real GPU
drivers don't carry this ceiling (typical minimum is 1024+ per the GLES3
spec) -- this is a software-renderer-specific limitation, matching exactly
the "real GPU rendering ... cannot be confirmed from a headless/cloud
session" carve-out `DECISIONS.md` §5 and the `cartalith-porting-discipline`
skill already document, just encountered here via an emulator instead of a
sandbox.

**What this does and doesn't tell us about criterion 4.** Godot loads
GDExtensions during its own early boot (`Main::setup2()`), which Phase 0's
`--headless --import` entry (above) confirmed logs `Initialize godot-rust
(...)` on success. That line never appears in either capture -- but neither
does any explicit extension-load failure (`Can't open dynamic library`,
`GDExtension dynamic library not found`), which Godot logs just as loudly
on the paths we've hit before (the Windows `.gdextension` path bug,
earlier this CHANGELOG). The honest read: the boot sequence appears to
stall inside shader/splash-screen setup before it reaches a point this
capture can distinguish "extension loaded silently" from "extension never
attempted." **This emulator run cannot confirm or deny whether
`libcartalith_godot.so` loads on Android** -- it only confirms the `.apk`
installs, launches, and doesn't crash. That was already known from the
packaging entry above.

**Not pursued further**: forcing a different rendering driver/backend to
work around SwiftShader's shader-linking ceiling, since any such change
would be diagnosing the emulator's software renderer, not this port, and
the real target (owner's physical device, real GPU) doesn't share this
constraint. `MVP_SCOPE.md` criterion 4 remains exactly where the prior
entry left it: one sideload away, on real hardware, not reachable from
this environment.

One correction to the read above: this AVD (`emulator-5554`,
`sdk_gphone16k_x86_64`) is an **x86_64** system image, not arm64 --
`ro.product.cpu.abi` confirms it, despite the `.apk` shipping only
`lib/arm64-v8a/*.so`. It runs our arm64 libraries through
`ro.dalvik.vm.native.bridge=libndk_translation.so` (confirmed present at
`/system/lib64/libndk_translation.so`), Android Studio's standard ARM
translation layer for x86_64 images with `abilist=x86_64,arm64-v8a`. This
doesn't overturn the SceneShaderGLES3/SwiftShader root cause above (that's
a pure GL-driver issue, unrelated to CPU architecture), but it does mean
the emulator adds a second, unaudited layer of indirection
(binary-translated native calls) between our `.so` and the OS that a real
device wouldn't have -- one more reason this environment's result can't
stand in for actual hardware, independent of the rendering finding.
Checked for a physical device on this machine's USB first
(`adb devices -l`) in case one was already connected and sideloading could
happen directly from here; none was.

## `cartalith-io` verified against a real HTML-app export: `MVP_SCOPE.md` criterion 7 closed (2026-08-16)

The one half of criterion 7 no prior session could reach: `load_save` had
only ever been checked against synthetic fixtures built by hand
(`SaveData`'s own doc comment called this out explicitly as "not a
substitute for testing against a real export"). No browser was ever
available in any session -- solved the same way this port solves every
other "no browser" gap: the reference's own `exportZip()` (line ~12418) is
pure-data JS, reachable via the same Node `vm.runInContext` harness this
project has used all session, run against the real, unmodified reference
engine.

**A real, previously-undocumented gotcha found along the way**: the
reference file declares **two** top-level `function generate(...)` in the
same global scope -- the real terrain engine's `async function generate()`
(line 3339, no arguments) and the unrelated urban-morphology block's own
`function generate(seed,opts)` (line 30931, block 4). Both are plain
non-module `<script>` tags (four of them, confirmed via `grep -n
"</script>"`: lines 14557/26721/28162/31104), so they share one global
object -- the later declaration silently wins the binding after all four
scripts load, meaning a harness that concatenates the whole file and calls
bare `generate()` would silently run the WRONG function. Sidestepped by
extracting only script tag #1's content (lines 2084-14556), which contains
the entire terrain/climate/hydrology engine plus `exportZip`/
`buildGridFields`/`serializeState` and none of the urban block, so the name
collision never loads at all. Worth remembering for any future extraction
that naively slices "the whole `<script>` region" instead of a specific tag.

**Harness upgrade, reusable next time**: the previous harness's DOM/timer
stubs were enough for `generate()`'s own pipeline but not for a real
export, since this file's top-level script also wires a large amount of UI
(event listeners, `querySelectorAll` chains, `bind()` calls) that isn't
worth stubbing method-by-method. Replaced with a single permissive
Proxy-based fake DOM element: any read of an unset property returns
another instance of itself (both callable AND carrying element-like
methods, since call sites use the same expression either way --
`el.querySelectorAll(...).forEach(...)` calls it, `document.documentElement`
reads it as a sub-object), so arbitrary UI wiring silently no-ops instead
of needing a hand-maintained stub list. `window` set to the sandbox object
itself (browser semantics: `window === globalThis`).

**Verified real, not synthetic.** Extracted the 7 entries
`cartalith_io::load_save` actually reads (`params.json`, `heightmap.f32`,
`temperature.f32`, `rainfall.f32`, `volcanic_field.f32`, `impact_field.f32`,
`strahler_order.bin` -- confirmed by reading `SaveData`'s fields and
`load_save`'s own `read_entry` calls directly, not assumed) from a real
`generate()` run at `gw=14 gh=11 seed=24601 world=false` -- deliberately
the same config as `golden_parity_carve.rs`'s case 0, which let
`field[0..5]` from this independent extraction be cross-checked directly
against that fixture's `expected_field[0..5]`: **exact match**, real
evidence this harness reconstruction is faithful and not a coincidentally-
similar but differently-configured run. Zipped via PowerShell
`Compress-Archive` (native platform tool, no new npm dependency for a
one-shot extraction script -- `SAVEFILE_COMPAT.md` already confirms the
`zip` crate reads both STORE and DEFLATE) after confirming the produced
archive's entry names are flat, not nested under a folder (a real
`Compress-Archive` gotcha, checked directly via
`[System.IO.Compression.ZipFile]::OpenRead`, not assumed safe).

New fixtures: `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`
(the real export) and `..._captured.json` (the same values captured
directly from the JS sandbox's own typed arrays at export time,
independently of the `.zip`'s own bytes -- comparing against this rather
than re-reading the `.zip` back is what makes this an actual loader check,
not a test of `load_save` against itself). New test
`crates/cartalith-io/tests/golden_parity_real_export.rs`: bit-exact
equality (not tolerance -- reading a raw little-endian `Float32Array` dump
back is a lossless byte reinterpretation, not a second computation), plus a
sanity check that `volcanic_field`/`strahler_order` carry real non-zero
variation rather than placeholder zeros. **Passed on the first attempt.**

- `cargo test -p cartalith-io`: all green (3 existing + 1 new).
  `cargo clippy -p cartalith-io --all-targets`: clean. `cargo test
  --workspace`: everything outside `cartalith-godot` green; a concurrent
  session was mid-edit on `cartalith-godot`'s own rendering port and its
  `golden_parity_render.rs` was failing at the time -- not this entry's
  scope, left untouched, see that crate's own CHANGELOG entry.

**This closes `MVP_SCOPE.md` criterion 7 for real** -- not "compiles and
the Rust-side unit tests pass" (the prior state, per this file's own
2026-08-13 entry) but an actual real-export round-trip, bit-exact,
verified. `STATUS.md` updated to match.

## Godot UI reskin via ui-ux-pro-max: dark technical-dashboard palette + grouped settings (2026-08-16)

The owner asked to use the `ui-ux-pro-max` skill (installed earlier this
session, security-reviewed just before this pass -- `scripts/*.py` checked
for `exec`/`eval`/`subprocess`/network calls; the only matches were an
offline data-validator (`validate_data.py`) and its own test suite, not
the runtime search/design-system path) to migrate and improve on the
reference HTML's interface design for `godot-project/`.

`python scripts/search.py "... dashboard tool" --design-system --density 8
--variance 4` (a first attempt with plainer keywords matched the skill's
marketing-landing pattern -- hero/CTA/footer -- wrong shape for a dense
control panel; retried per the skill's own protocol with dashboard-shaped
keywords) returned a dark "technical dashboard" system: background
`#0F172A`, card `#1B2336`, slate primary/secondary `#1E293B`/`#334155`,
accent `#22C55E`, foreground `#F8FAFC`, muted-foreground `#94A3B8`, border
`#475569`, destructive `#EF4444`, focus ring `#FFFFFF`. A follow-up `ux`
domain search (`"slider checkbox touch target size"`) confirmed the
44px-minimum/8px-gap touch rules the previous session's responsive pass
had already applied -- verified, not re-derived.

**Scope decision, made explicitly rather than by default:** the reference
HTML's real control inventory (grepped every top-level `bind(...)` call)
is overwhelmingly Phase-3/civ-layer material this port doesn't implement
-- AO/SVF/shadows/curvature-shading/geology-microtexture/wetness/splat/
ridged-relief, all four SDF coast/river/biome sliders, all nine "Painter"
NPR style sliders, civ icon/way scale, territory/way opacity, planet
rotation/axial-tilt/geoid/tides, erosion evolve-cycle/velocity knobs.
Adding UI for any of these would be a dead control that does nothing --
worse UX than the control not existing. "Migrate the UI" was read as:
apply the reference's level of visual/UX polish to the real, engine-backed
control surface, not port every DOM widget that happens to exist.

**What changed:**
- `theme/app_theme.tres` rebuilt on the new palette (was a warm amber
  accent from an earlier session's own from-scratch pass -- not wrong,
  just not what the design-system search actually recommends for this
  product category). Added explicit `styles/focus` StyleBoxFlat overrides
  for `Button`/`OptionButton`/`CheckBox`/`PrimaryButton` (a white 2px
  outline with `expand_margin`) -- the previous theme only had a focus
  style on `LineEdit`, so keyboard-only navigation had no visible focus
  indicator on any button/checkbox/dropdown, a real pre-delivery-checklist
  gap (`references/pro-rules.md` accessibility section) that predates this
  pass, not introduced by it.
- `main.tscn`: split the single "World Parameters" card into two --
  "World Parameters" (seed/resolution/width) and "World Structure"
  (archetype dropdown, now with its own explanatory hint pulled from
  `MVP_SCOPE.md` point 5's own re-anchoring language) -- matching how the
  reference conceptually separates these, not how the MVP UI happened to
  ship them flattened together. "Advanced Features" card kept, hint text
  tightened. Small accent-colored header dot added (a `ColorRect`, not an
  icon font/emoji -- `pro-rules.md`'s "no emoji as structural icons" rule
  applies even to a decorative mark). All existing `unique_name_in_owner`
  refs (`%SeedInput` etc.) preserved exactly, so `main.gd` needed zero
  changes -- Godot's `%Name` lookup is scene-wide, not path-dependent.
- Manually checked text/background contrast pairs against WCAG 4.5:1
  (foreground-on-background, muted-foreground-on-card, accent-on-card,
  navy-on-accent-button): all clear 6:1 or better. `cargo`/`godot4`
  verification is **not** run this pass -- `cartalith-godot`'s own Rust
  side is mid-edit by a concurrent session (its `render.rs` port), so a
  build/run right now would test that work's in-progress state, not this
  one's. `.tscn`/`.tres` internal consistency (every referenced
  `SubResource`/`ExtResource` id declared, no dangling unique-name refs)
  checked by manual review only -- real engine verification is deferred
  to whoever runs `godot4 --headless --quit main.tscn` once the rendering
  port lands.

**Deliberately deferred, not forgotten:**
- Typography: the design-system match was Fira Sans/Fira Code ("dashboard,
  data, analytics, technical, precise"). Sourcing and OFL-license-checking
  real font files wasn't done this pass (no verified way to fetch and
  confirm a binary asset's license from here in one shot) -- theme keeps
  Godot's built-in default font. Follow-up.
- `MVP_SCOPE.md` point 9 (sea level) is real in-scope terrain parameter
  the reference exposes (`bind('sea', ...)`) that this UI still doesn't
  -- flagged, not added, since exposing it needs a new `#[func]` on
  `WorldGen` and `cartalith-godot/src/lib.rs` is the concurrent rendering
  fork's file this pass explicitly avoided touching.
- Every gated Phase-3 viz control listed above -- explicitly out of scope
  until the rendering they'd control actually exists.

`STATUS.md` updated: the `ui-ux-pro-max` "installed but never reviewed"
open item is now reviewed (see above); this UI pass itself noted under
Phase 1.

## Real biome/hillshade rendering replaces the MVP placeholder tint (2026-08-16)

Closes the `build_color_texture` gap its own old doc comment flagged: "a
simplified stand-in for the reference HTML's own biome colouring,
deliberately not attempted here." Ported the reference renderer's real
default-settings material synthesis instead -- `materialWeights` (the
snow/rock/sand/wetland/canopy/grass fraction model, reference HTML
7655-7707), the six climate-selected material colour ramps, the ecotone
noise jitter, the multi-scale (macro+meso+micro) hillshade, the `bioBlend`
grey-desaturation blend, the edge haze fade, and `seaColorCore`'s
depth-banded/temperature-tinted water with sea-ice and surf-line -- all new
in `cartalith-godot/src/render.rs`. Deliberately excluded: every
`state.viz.*`-gated stretch feature (splat texturing, geology microtexture,
NPR "Painter" styles, AO/SVF/shadow fields, coast/river SDF tinting, the
vector river overlay), all off at JS's own defaults, so omitting them
changes nothing about the *default* view -- `render.rs`'s own doc comment
has the full list and why each is out.

Two real bugs caught by golden verification, not code review:
- **Missing final `ao * vignette` multiply** (reference HTML 7959-7960) --
  sits right after the entire gated "Painter" NPR block but is itself
  unconditional, easy to miss on a read that stops at "the core." Corner
  cells were rendering ~40% too bright without it (a golden test cell
  caught a 184-vs-108 mismatch immediately).
- **`seaColor` reads smoothed bathymetry, not the raw field** -- JS's real
  default (`state.mode==='biome'`) always builds `_seaH`/`_seaShade`
  (`smoothSeaH`/`seaShadeFrom`, two separable box-blur passes over the
  heightmap + a hillshade of the result) before shading water, "so the
  seas were reading blocky" per the reference's own v0.063 comment. Not a
  stretch feature -- it's what a default `generate()` + real app session
  actually produces on screen, confirmed by extracting golden data through
  an actual `await generate()` run (which triggers a real render pass) and
  finding `seaColor(...)` disagreed with a naive `seaColorCore(...)` call
  using the raw field. Ported `smoothSeaH`/`seaShadeFrom` (new `box_h`/
  `box_v` helpers in `render.rs`) rather than working around it.

**Golden-verified** (`golden_parity_render.rs`, new): two fixtures, both
running the real reference `generate()` under Node and calling
`isWater(v) ? seaColor(...) : surfaceColor(...)` per cell directly --
`GW=GH=10`/seed 24601/`world=false` (41/100 cells water, exercises both
colour paths) and `GW=GH=12`/seed 314159/`world=true` (exercises
`slope_at`'s X-wrap path the first fixture never touches). Both pass at
`1e-4` per-channel tolerance (Math.pow/exp/hypot vs. Rust's f64
equivalents can differ by a handful of ULPs through `materialWeights`'
two-pass canopy closure -- not bit-exact by construction, unlike the
kernel-level golden tests elsewhere in this port, but far tighter than
anything visually perceptible).

`WorldGen` gained `world`/`lat_n`/`lat_s` fields (`latAt`'s inputs) set
from `p.world`/`p.climate.lat_n`/`.lat_s` on `generate()`/
`generate_world_structure()`; a loaded save has no stored latitude band
(`SAVEFILE_COMPAT.md`), so it falls back to JS's own literal `climate`
defaults (55/5), same as `WorldParams::defaults` does.

- `cargo build -p cartalith-godot` / `cargo check --workspace --all-targets`:
  clean. `cargo clippy --workspace --all-targets`: clean. `cargo test
  --workspace`: all green, including both new render fixtures. `godot4
  --headless --quit main.tscn`: clean, extension loads
  (`Initialize godot-rust ...`).

**What this doesn't reach**: pixel-for-pixel parity with every one of the
reference's opt-in visual stretch features (still Phase 3, `ROADMAP.md`),
and the river-network vector overlay (`drawRiverWays`) -- the existing
simple channel-mask blue tint stays as its stand-in until that subsystem
is ported.

## Real Windows hands-on verification: two real bugs caught, and a theme swap to the reference's own light palette (2026-08-16)

With the rendering port, UI reskin, and criterion-7 work all landed, ran
the actual full MVP UI on this session's real Windows desktop -- not
`godot4 --headless`, the genuine windowed app (`Godot_v4.7.1-stable_win64.exe
--path godot-project main.tscn`), screenshotted via `PrintWindow` (occlusion-
independent, unlike `CopyFromScreen` which the first attempt used and which
captured the wrong window entirely), and driven with real synthetic mouse
clicks at the button's actual screen coordinates. This is `MVP_SCOPE.md`
criterion 3's other half, previously only satisfied by the Phase 0
walking-skeleton's `ping()` round-trip, not the real terrain-generating UI.

**Two real bugs found by this, neither visible from reading the code:**

1. **World-Structure dropdown showed no text at all.** Root cause: the
   `.tscn`'s hand-authored `item_0/text`..`item_5/text` properties on
   `WorldShapeInput` (added by the same-day UI reskin pass) are missing the
   `id`/`icon` sub-fields Godot's own editor normally serialises alongside
   `text` -- hand-typed directly into the text-format scene file, they
   deserialise as broken/blank entries. First fix attempt made this *worse*
   -- adding a `_ready()` populate-via-script loop on top of the already-
   present (broken) scene items produced literal empty rows before the
   real options in the dropdown popup, exactly as the owner reported by
   screenshot. Correct fix: delete the malformed scene-authored items
   entirely and populate purely from `main.gd`'s existing
   `WORLD_SHAPES`/new `WORLD_SHAPE_LABELS` arrays (`_ready()`), the single
   source of truth the script already had -- plus `world_shape_input.
   selected = 0` so a Classic default actually displays instead of
   OptionButton's `-1` (which GDScript's negative array indexing had been
   silently resolving to `WORLD_SHAPES`'s *last* entry, `"rift"`, not
   `"Classic"`, every time the JS-parity-verified default flags implied it
   should be Classic).
2. **Window title still read "Cartalith (walking skeleton) (DEBUG)"**
   -- `project.godot`'s `config/name` was never updated past Phase 0. Now
   `"Cartalith Terrain Generator"`.

**Theme swap, per explicit owner feedback ("I like the light colorscheme
better from the html")**: the reference HTML actually defaults to a *dark*
palette (`:root{ --bg:#101218; ... }`) -- what the owner meant is its real,
built-in `:root[data-theme="light"]` alternate theme (line 271, a parchment
palette the reference itself ported from the older V1.915 editor's "Light"
option: `--bg:#efe7d6 --panel:#fbf5e9 --panel2:#e7ddc9 --line:#d3c8b0
--ink:#2a2015 --dim:#6d5f47 --accent:#b07f3f --accent2:#3f6f9e
--warn:#b04a4a`). Replaced the earlier same-day `ui-ux-pro-max` dark
"technical dashboard" match (`#0F172A`/`#22C55E`) with a literal port of
this real palette into `theme/app_theme.tres`, plus the two hardcoded
non-Theme colours in `main.tscn` (`Background` `ColorRect`, header
`AccentDot`). `accent2` (blue) is used only for keyboard-focus rings, kept
deliberately distinct from `accent` (warm brown, used for primary/active
styling) so "this is focused" and "this is the active/primary control"
stay visually different signals. The reference's own comment on this palette
is worth carrying forward: only UI chrome restyles; the rendered map's
colours are JS ramps (now `render.rs`'s ported equivalent), never CSS/Theme
-- confirmed by regenerating the same seed under both themes and observing
identical map pixels, only the surrounding chrome changed.

Both fixes (and the theme swap) verified by the same real-window
screenshot+click loop, not just re-read: dropdown now shows "Classic" and
the status label correctly reports the archetype
(`"128x128, seed 12345, 800 km, Classic"`); title bar confirmed; light
theme confirmed end-to-end including a real `Generate` click producing the
same terrain under the new chrome.

- `cargo test --workspace`: all green (no Rust touched by this entry, but
  confirmed rather than assumed). `godot4 --headless --quit main.tscn`:
  clean, extension loads.

## GPU-compute pilot: `wgpu` viable as a hardware path, not viable for this specific formula (2026-08-16)

`GPU_COMPUTE_PILOT_SCOPE.md` (repo root) scoped the first milestone of the
owner-supplied `HARDWARE_ACCELERATION.md` architecture: prove or disprove a
standalone `wgpu` compute path on real hardware, using exactly one kernel
(`cartalith_noise::vnoise`) as the test case. New crate `cartalith-gpu`
(`wgpu` 30.0.0, `pollster` 1.0.1, `bytemuck` 1.25.2 -- versions verified
against crates.io/docs.rs at implementation time, not assumed, per
`HARDWARE_ACCELERATION.md` §36's own warning that `wgpu`'s API moves
between majors). No `gdext` dependency; builds and tests with plain `cargo
test -p cartalith-gpu`, no Godot involvement (`ARCHITECTURE.md`'s rule
intact).

**The hardware path itself: works cleanly.** `Instance`/`Adapter`/`Device`
creation, adapter inspection, conservative limits
(`Limits::downlevel_defaults().using_resolution(...)`, not
`Limits::unlimited()`), shader compilation, buffer/bind-group/pipeline
setup, dispatch, and readback all function correctly on this session's
real hardware: AMD Radeon RX 7800 XT, Vulkan backend, discrete GPU
(`gpu_context_creates_on_this_hardware`).

**The specific formula: does not survive an f32 GPU port, and the reason
is precise, not vague.** `cartalith_noise::hash`'s own doc comment already
warned its middle product reaches ~2^61 -- past `f64`'s own exact-integer
range (2^53), meaning even the trusted CPU reference relies on `f64`'s
specific rounding behaviour at that magnitude. Porting the same formula to
WGSL's `f32` (24-bit mantissa) loses far more precision at that magnitude,
and WGSL's `f32`->`u32` conversion for out-of-range floats is
implementation-defined/saturating, not the wrap-on-truncate Rust's `(x as
i64) as u32` guarantees -- both effects compound at *every* `hash` call
(the ~2^61 regime is the norm here, not an edge case). Measured, not
theorised: at 128x128 (`f32_hash_diverges_from_cpu_reference`),
**16384/16384 cells** (100%) exceed even a loose `1e-4` tolerance, max
absolute difference `0.93` on a `[0,1]`-ranged output -- categorically
wrong, not "close but imprecise." `self_test` (the actual `HARDWARE_
ACCELERATION.md` §9 gate, not a separate throwaway check) correctly
reports FAIL, and `vnoise_grid`'s public API correctly refuses the GPU
path and falls back to CPU as a result -- the gating logic itself is
proven correct by this failure, not just the happy path.

**A secondary experiment, and a genuinely interesting dead end.**
`wgpu::Features::SHADER_F64` *is* reported present on this adapter (Vulkan
exposes `shaderFloat64` on this GPU) -- raising the obvious question of
whether an `f64`-arithmetic WGSL kernel could close the gap entirely.
It cannot be tried at all, for a reason worth recording precisely: naga
(wgpu 30's WGSL front end) does not implement `enable f64;` -- its
`EnableExtensions` type lists `f16`/`wgpu_int16`/ray-tracing/mesh-shader
extensions but no `f64` entry, confirmed by reading naga's own source
(`front/wgsl/parse/directive/enable_extension.rs`) and reproduced live
(`f64_wgsl_is_not_implemented_by_naga_even_though_the_gpu_feature_exists`,
caught cleanly via `push_error_scope`/`pop_error_scope` rather than left
to panic). The GPU and the `wgpu::Features` API both expose the
capability; the WGSL shader language, as wgpu 30 compiles it, has no
syntax to use it. (A raw-SPIR-V shader source could bypass WGSL and use
`f64` directly -- a real door, deliberately not opened: hand-authoring or
generating SPIR-V is well outside this pilot's "port the formula, don't
reformulate the toolchain" scope, per `GPU_COMPUTE_PILOT_SCOPE.md`.)

**Real timing numbers** (`measured_gpu_vs_cpu_timing`, GPU dispatch+readback
vs. single-thread CPU, this hardware, after a warm-up dispatch to exclude
one-time pipeline/driver JIT cost):

| Field size | Cells | GPU (dispatch+readback) | CPU (single-thread) | CPU/GPU ratio |
|---|---|---|---|---|
| 128x128 | 16,384 | 941.8µs | 186.7µs | **0.20x -- GPU loses** |
| 512x512 | 262,144 | 685.9µs | 3.058ms | 4.46x |
| 1024x1024 | 1,048,576 | 778.2µs | 12.180ms | 15.65x |
| 2048x2048 | 4,194,304 | 2.488ms | 48.631ms | 19.55x |

The classic, expected shape: dispatch/readback overhead dominates and
loses at small sizes, GPU wins increasingly at scale -- exactly
`HARDWARE_ACCELERATION.md` §6's own framing ("small operations should
remain on the CPU when that is demonstrably faster"). These numbers are
real data for judging *future* GPU-compute candidates that don't share
`hash`'s f64-precision dependency, not a verdict on this specific kernel
(which isn't deployable regardless of how fast it runs, since it's wrong).

**CPU fallback**: actually exercised, not just present
(`gpu_fallback_path_matches_cpu_reference` forces the no-GPU branch via
`ctx: None` and asserts the result is bit-identical to the already-trusted
CPU path).

- `cargo build -p cartalith-gpu`, `cargo test -p cartalith-gpu` (7/7 pass),
  `cargo clippy -p cartalith-gpu --all-targets` (clean): all green.
  `cargo test --workspace` / `cargo build --workspace`: no regressions
  elsewhere.

**`GPU_COMPUTE_PILOT_SCOPE.md`'s own question, answered honestly**: is
`wgpu` a viable, correctness-preserving path on this project's actual
hardware, for embarrassingly-parallel work? **Yes, the hardware/API path
itself is solid.** Is *this* kernel a candidate for GPU deployment?
**No** -- not a wgpu limitation, a precision limitation specific to a
hash formula whose CPU reference deliberately depends on `f64`
magnitude-dependent rounding to match the JS engine
(`cartalith-rust-conventions`: match precision, don't improve it -- the
same discipline that makes this formula hard to port is the discipline
that makes the CPU version correct in the first place). Nothing outside
`GPU_COMPUTE_PILOT_SCOPE.md`'s "In scope" list was implemented; the
`ComputeTier` classifier, diagnostics panel, telemetry system, tiled
compute, and every other §-numbered item on the "Out of scope" table
remain untouched, as scoped. Whether other candidate subsystems
(hillshade/AO synthesis, biome classification -- pure functions of
already-computed fields, no `hash`-style huge-integer arithmetic) fare
differently is the natural next question, not answered by this pilot.

## Phase 2 milestone 1: affordance fields foundation (lithology, soil fertility, water access) (2026-08-16)

First real Phase 2 (civilisation layer) work, scoped in `PHASE2_SCOPE.md`
after tracing `currentSettlementSuitability`'s (the "v1.30 one function"
`ROADMAP.md` flags) real dependency chain and finding it several
milestones away, not a starting point -- the reference's own history
(v0.104 comment, line ~5824) already drew this exact boundary: *"this
lands lithology -> soil -> water access; resources + carrying-capacity +
settlement suitability are the v0.105-0.106 follow-ups."* This entry ships
that same first slice.

**New crate `cartalith-civ`** (`crates/cartalith-civ/`), zero `gdext`
dependency, depends on `cartalith-engine` (for `WorldState`) and
`cartalith-hydrology` (for the shared `river_flow_thresh`). Resolves the
placement tension `PHASE2_SCOPE.md` flagged (reference treats these as
block-1/terrain functions, `ROADMAP.md` names a new `cartalith-civ` crate)
by having the new crate depend on already-computed terrain/climate output
without modifying it -- matches both.

**Ported** (reference HTML lines 5835/5852/5866):
- `buildLithology` -> `build_lithology`: categorical 7-type rock
  classification (granite/basalt/andesite/limestone/sandstone/shale/
  metamorphic) from crust sign, volcanic intensity, resistance, and
  rain-conditioned elevation band. Pure, single-pass, no neighbour reads.
  The reference signature also takes an unused `hetero` parameter (dead in
  the original too) -- omitted here, a no-op restructuring
  (`cartalith-porting-discipline`: "internal restructuring that preserves
  output: proceed").
- `buildSoilFertility` -> `build_soil_fertility`: Jenny (1941) pedological
  interaction -- temperature bell x moisture x lithology-weatherability x
  slope-shedding x age-development. Needed its own `slope_at` -- a
  deliberate small duplicate of `cartalith-godot/src/render.rs`'s existing
  copy (same reference function, `slopeAt`, line 7584) rather than a
  cross-crate extraction for one ~10-line pure function; `render.rs` can't
  be a dependency (it's the `gdext` boundary crate).
- `buildWaterAccess` -> `build_water_access`: exponential distance decay
  from rivers/coast, via a new `chamfer_dist` (reference's `chamferDist`,
  line 7423) -- a two-pass raster chamfer distance transform. `d` stays
  `f32` throughout (matching the reference's own `Float32Array`), with
  every per-cell store narrowed from an `f64` accumulation -- the
  intermediate truncation genuinely participates in the result (each
  cell's neighbours read back the already-truncated value), not just a
  final-output rounding, so this needed the same "accumulate at f64,
  narrow only at store, per-cell" discipline the orogeny fix used earlier
  this session, applied to every single raster-scan step rather than once
  at the end.

**A real gap found and fixed along the way, not part of this milestone's
own scope**: `WorldState` never retained `plateCrust()`'s Rust equivalent
(`base_raw`, the raw per-cell plate base `buildLithology`'s `crust`
parameter needs) past `generate_terrain` -- it was a local variable,
computed and used for orogeny/height, then dropped. Added as a new
`WorldState.crust_field` (pure addition, `cartalith-engine` owns
`WorldState` per the porting-discipline ladder, no existing field's
numeric output touched).

**Golden-verified** against a real reference `generate()` run via the
established Node `vm.runInContext` harness (transient, rebuilt for this
task, not checked in), reusing `golden_parity_carve.rs`'s exact two fixture
configs (`gw=14 gh=11 seed=24601 world=false`, `gw=16 gh=12 seed=314159
world=true`, `w_iters=12`) so this doubles as a cross-check that both
extractions agree on the same underlying run (confirmed: `sea_level`
matched exactly in both). Lithology asserted bit-exact (`assert_eq!` on
the `Vec<u8>` -- it's a categorical classification, any mismatch would be
a real bug); soil fertility and water access at `1e-4` atol+rtol, matching
`golden_parity_carve.rs`'s own convention. **Both cases passed on the
first attempt.**

- `cargo test -p cartalith-civ` (8 tests: 6 unit + 2 golden), `cargo
  clippy -p cartalith-civ --all-targets`: clean (one real `if_same_then_
  else` lint fixed by merging the two source conditions with `||`, since
  the reference's own `if(fld[i]<sea) src[i]=1; else if(flow[i]>thr)
  src[i]=1;` has genuinely identical bodies). `cargo test --workspace` /
  `cargo build --workspace`: no regressions elsewhere.

**Where this leaves Phase 2**: milestone 1 of an unknown-but-large number
still needed before civilisation-layer feature parity. `currentSettlement
Suitability` itself still needs resource potentials, carrying capacity,
route corridors, landmass quality, coast SDF, and water-body
classification -- none of which exist yet, all explicitly deferred by
`PHASE2_SCOPE.md`'s own "Out of scope" table. Factions, territory, roads,
provinces, economy, and the Journey Planner remain untouched.

## Phase 2 milestone 2 -- water-body classification (2026-08-16)

`PHASE2_SCOPE.md`'s milestone 2: `buildWaterBodies` (reference HTML line
5753) ported to `cartalith-civ` as `build_water_bodies`. Two real
algorithms, not one -- a connected-components flood fill (largest below-sea
component = ocean, every other below-sea component = lake) and a
priority-flood depression fill (Barnes-style min-heap) for above-sea
pooled lakes, gated on local rainfall. `PROVENANCE.md` already flagged
this exact algorithm ("hand-port, carefully: equal-priority pop order
decides the fill tie-break and therefore lake shape") -- the reference's
own hand-rolled array-backed `MinHeap` was ported index-for-index and
comparison-for-comparison (`<=` sift-up break, `<` sift-down child
selection), not swapped for `std::collections::BinaryHeap`, since that
crate's tie-break behaviour on equal priorities is not guaranteed to match.

**A real, root-caused harness bug, not a fixture mismatch.** The first
extraction attempt produced field values wildly different from
`golden_parity_carve.rs`'s own `expected_field` -- and, worse, genuinely
*nondeterministic* across separate process runs of the identical harness
script with the identical intended seed. Root cause: the reference's own
`state` literal defaults `tect.seed` to `(Math.random()*99999)|0` at
script-load time (line 2264) -- the real per-generation seed lives at
`state.tect.seed`, not a top-level `state.seed`. The harness had been
setting `state.seed` (a field nothing reads), leaving the actual generation
seeded by whatever `Math.random()` produced that process launch. Confirmed
by running the extraction twice and diffing (different results both
times), then fixed by setting `state.tect.seed` (matching
`WorldParams.tect.seed` on the Rust side) and re-verifying determinism
across two more runs before trusting the data.
`cartalith-porting-discipline`'s own rule held here: a red/wrong result
means re-read and root-cause, not adjust a tolerance to paper over it --
there was no tolerance question at all, the bug was entirely in the
harness's own state setup. Also needed an explicit `allocate()` call (the
reference's own auto-boot sequence, stripped from the harness to control
`generate()` invocation directly, was the only caller of `allocate()` for
a fresh resolution -- omitting it left `riverMask`/`riverFloor` null,
crashing `carveRiverValleys`) and a correction to `GH`: the reference
derives grid height from width via a fixed aspect ratio (`gridH(gw) =
round(gw*0.64)`, giving `GH=9` for `GW=14`), but this port's
`WorldParams::defaults(gw, gh, seed)` takes `gw`/`gh` as independent
parameters -- the harness must set `GH` directly to match the Rust
fixture's `gh=11`, not derive it from `gridH()`.

**Golden-verified** against the same two fixture configs as milestone 1
and `golden_parity_carve.rs` (`gw=14 gh=11 seed=24601 world=false`,
`gw=16 gh=12 seed=314159 world=true`), with `field[0..5]` matching
`golden_parity_carve.rs`'s own `expected_field[0..5]` exactly once the
seed-field bug was fixed -- real cross-validation the harness reconstruction
is faithful. Case 0 exercises the 0/1 (land/ocean) path with no pooled
lakes; case 1 exercises all three classes (127 land, 13 ocean, 52 lake)
including the x-wrap connected-components/priority-flood path.
Classification asserted bit-exact (categorical `u8`); fill-level at `1e-4`
atol+rtol, matching this workspace's convention. **Both cases passed on
the first attempt once the harness was actually correct.**

- `cargo test -p cartalith-civ` (14 tests: 10 unit + 4 golden), `cargo
  clippy -p cartalith-civ --all-targets`: clean. `cargo test --workspace` /
  `cargo build --workspace`: no regressions elsewhere.

**Scope discipline**: nothing from `PHASE2_SCOPE.md`'s "Out of scope"
table was touched. Biome classification (`classifyBiome`/`buildBiomeRaster`)
reads this milestone's output but stays its own milestone 3, not bundled
in just because it's the natural next step.

**Where this leaves Phase 2**: milestone 2 of 2 done so far. Biome
classification is milestone 3; resource potentials, carrying capacity,
population density, settlement suitability, factions, territory, roads,
provinces, economy, and the Journey Planner remain untouched.

## Phase 2 milestone 3 — biome classification (2026-08-16)

Ported `classifyBiome` (reference HTML line 5736 -- pure temperature/
moisture -> one of 12 climate-biome categories, threshold order preserved
exactly) and `buildBiomeRaster` (line 6798 -- applies it per-cell, with
milestone 2's water-body classification overriding climate for ocean/lake
cells) to `cartalith-civ`. `BIOME_KEYS`/`BIOME_INDEX` (lines 6796-6797)
ported as named `u8` constants (`BIOME_ICE`..`BIOME_TROP_WET`, plus
`BIOME_OCEAN`=0 and `BIOME_LAKE`=13, matching the reference's own index
values) rather than a Rust enum, for consistency with milestone 1's
`build_lithology` convention (plain numeric codes, not an enum).

Extraction harness called the reference's own `buildBiomeRaster()`
directly post-`generate()` rather than hand-composing
`classifyBiome`+water-body logic in the harness -- exercises the exact
composition production JS code uses, not a parallel reimplementation of
it. Cross-checked two independent ways before trusting the data: (1)
`field[0..5]` matched `golden_parity_waterbodies.rs`'s already-verified
`expected_fill[0..5]` exactly for both cases (confirming the harness
correctly applies milestone 2's own root-caused seeding fix --
`state.tect.seed`, not `state.seed`); (2) each case's biome category
counts summed exactly to that same file's known ocean/lake/land totals
(case 0: 75 ocean + 79 land; case 1: 13 ocean + 52 lake + 127 land) -- a
biome raster with a real classification bug would not reproduce those
totals by coincidence. Both golden cases passed bit-exact on the first
attempt.

- `cargo test -p cartalith-civ` (19 tests: 15 unit + 4 golden -- 6 new
  unit tests cover every `classifyBiome` threshold branch, plus one
  covering `buildBiomeRaster`'s water-override precedence), `cargo
  clippy -p cartalith-civ --all-targets`: clean. `cargo test --workspace`
  / `cargo build --workspace`: no regressions elsewhere.

**Scope discipline**: `buildCartBiome` (reference line 6817 -- a
*different*, denser 15-category Cartalith editor-bridge biome-paint
auto-fill) confirmed out of scope: it feeds a paint-layer export/editor
bridge (`CART_BIOMES`/`CART_BIOME_COLS`) with no consumer anywhere in
this port (no painting UI, no Cartalith editor integration exists) --
same reasoning `PHASE2_SCOPE.md` already used to defer other
speculative-UI-adjacent work. Not implemented.

**What's confirmed still missing before milestone 4 (resources/carrying
capacity/population density) is fully reachable**: `boundary_type` and
`shear_field` (needed by `buildResourcePotentials`) already exist in
`cartalith-terrain`'s tectonic-substrate output from earlier this
session's orogeny work -- check whether they're retained on `WorldState`
or need the same treatment `crust_field` did in milestone 1 (computed but
discarded past `generate_terrain`). `buildCarryingCapacity` needs only
already-real inputs (soil, water access, biome, temp, field) plus
`buildWetlandMask` (small, not yet ported). Population density
additionally needs `buildNPP`/`currentNPP` (net primary productivity,
reference line 6613) -- **does not exist in this port yet**, a real gap
milestone 4 will need to close, not assumed away.

**Where this leaves Phase 2**: milestone 3 of (at least) 4 done. Resource
potentials, carrying capacity, population density are milestone 4;
settlement suitability, factions, territory, roads, provinces, economy,
and the Journey Planner remain untouched and further out.

## Phase 2 milestone 4 -- carrying capacity, NPP, population density (2026-08-16)

`buildResourcePotentials` split out into its own milestone 5 after
checking real size (~108 lines, 9 resource-type scoring rules -- see
`PHASE2_SCOPE.md`'s milestone 4/5 split). This milestone ports the three
smaller, already-reachable functions instead: `buildCarryingCapacity`
(reference line 6238), `buildNPP` (line 6497), and
`estimateRegionalDensityKm2` (line 6217), plus their small dependencies
(`biomeDensityResidual`/`biomeIntensifyEligible` lookups,
`WETLAND_DENSITY_RESIDUAL`/`WETLAND_INTENSIFY_ELIGIBLE`,
`buildWetlandMask` line 6839, `foragerFloorKm2`).

**A real semantic gotcha caught while porting, not by review alone**:
the reference's `bM=(bK&&biome) ? (1-bK+bK*resid) : 1` is a genuine
short-circuit, not a weighted blend that happens to reach 1 at `bK=0`. A
first instinct (`1.0 - biome_k + biome_k * resid` unconditionally) would
have been numerically identical at `biome_k=0.0` but silently *wrong*
the moment a caller passes `biome=None` with `biome_k>0.0` -- the
reference's own condition requires *both* `bK` truthy *and* `biome`
present, and this port's `build_carrying_capacity` reproduces that exact
gate (`if biome_k != 0.0 && biome.is_some()`), not just the arithmetic
that happens to match at the reference's own real default.

**Golden verification**: harness called the reference's own
`buildCarryingCapacity`/`buildNPP`/`estimateRegionalDensityKm2` directly
through `currentSoil()`/`currentWaterAccess()`/`buildBiomeRaster()`, the
exact production composition, not a hand-assembled reimplementation. Hit
a real harness gap along the way: `generate()` assumes `field`/`GW`/`GH`
and every subsystem field are already allocated by a prior `allocate()`
call (normally triggered by UI resolution-change handlers this harness
bypasses entirely) -- without it, `buildWaterBodies`'s `filled.set(fld)`
throws `RangeError: offset is out of bounds` because `field` is still
sized from the sandbox's own initial (wrong) grid dimensions. Fixed by
calling `allocate()` explicitly after setting `GW`/`GH`, before
`generate()`. Cross-checked before trusting the extraction: this harness's
own `field[0..5]` matched both `cartalith-engine/tests/golden_parity_carve.rs`'s
`expected_field` *and* `cartalith-io`'s real-export fixture
(`real_export_seed24601_captured.json`) exactly across the full 154-value
array for case 0 -- three independently-built harnesses across three
different sessions/milestones all agreeing is strong evidence this
extraction technique is sound, not just this one run. Determinism
reconfirmed by running case 0 twice and diffing (the milestone-2 lesson
applied cleanly from the start this time).

`build_carrying_capacity`/`build_npp`/`estimate_regional_density_km2` all
`1e-4` absolute+relative tolerance (continuous `f32` fields), matching
`golden_parity_affordance.rs`'s existing convention. Both fixture cases
passed on the first attempt.

- `cargo test/clippy -p cartalith-civ`: clean, 25 unit tests + 8 golden
  tests. `cargo test --workspace`/`cargo build --workspace`: no
  regressions.

**Scope discipline**: `buildResourcePotentials` (milestone 5) not
touched. Confirmed for milestone 5's benefit: `WorldState`
(`cartalith-engine/src/lib.rs`) has no `boundary_type`/`shear_field`
fields -- they exist only inside a local `stress` struct computed
mid-pipeline in `generate_terrain` and are discarded past it, the exact
same situation `crust_field` was in before milestone 1's fix. Milestone 5
will need the equivalent retention fix before it can start.

**Where this leaves Phase 2**: milestone 4 of (at least) 5 done. Resource
potentials is milestone 5; settlement suitability, factions, territory,
roads, provinces, economy, and the Journey Planner remain untouched and
further out.

## Phase 2 milestone 5 — resource potentials (2026-08-16)

`buildResourcePotentials` (reference HTML lines 6085-6172) ported to
`cartalith-civ`: 15 `[0,1]` geological-potential fields (copper, tin, iron,
gold, salt, timber, lead, silver, clay, buildstone, flint, obsidian, gems,
sulfur, alum) from lithology x boundary type x shear x crustal age x
volcanism x flow x rain x biome. Computed over the FULL map (submerged
cells included, matching the reference's own v0.86 fix for a
sea-slider-dependent blank layer), then thinned by a rank-based scarcity
cut (`applyResourceScarcity`/`resourceScarcityCut`) applied AFTER the
geology so it can only remove deposits, never invent them.

**`WorldState`'s first real gap this milestone needed, not optional**:
`boundary_type`/`shear_field` (`cartalith-terrain`'s `StressResult`) were
computed for T2+T3 orogeny but discarded past `generate_terrain`, the same
situation `crust_field` was in before milestone 1's fix. Added both as
retained `WorldState` fields (`cartalith-engine/src/lib.rs`) -- a pure
additive change, `cargo build --workspace` confirmed nothing else broke.

**Golden verification**: same Node `vm` harness technique, reconfirmed
deterministic (ran each case twice, diffed) and cross-checked against
`golden_parity_carve.rs`'s own `expected_field[0..5]` -- exact match both
cases, same as every prior milestone. All 15 fields at `1e-4`
absolute+relative tolerance, matching this crate's existing convention.
**Both fixture cases passed on the first attempt.**

Production default matched exactly, not assumed: `currentResourcePotentials()`
(reference line ~6452) passes no explicit `scarcity`/`scarcityLegacy`, so
these fixtures use `scarcity=true, scarcity_legacy=false` -- the original
six (copper/tin/iron/gold/salt/timber) are genuinely NOT scarcity-thinned
by default, only the nine v1.31 additions are. Verified with a dedicated
unit test (`build_resource_potentials_scarcity_default_spares_legacy_six`)
that would fail if this were backwards.

- `cargo test/clippy -p cartalith-civ`: clean, 37 unit tests + 10 golden
  tests. `cargo test --workspace`/`cargo build --workspace`/`cargo clippy
  --workspace --all-targets`: no regressions.

**Scope discipline**: settlement suitability, factions, territory, roads,
provinces, economy, and the Journey Planner all remain untouched. Nothing
outside `buildResourcePotentials` itself and the `WorldState` retention
fix it required was implemented.

**Where this leaves Phase 2**: 5 of (at least) 5 milestones scoped so far
are done -- every affordance field the reference's own v0.104-v0.106
history names (lithology, soil, water access, water bodies, biome
classification, carrying capacity, NPP, population density, resource
potentials) is now golden-verified in `cartalith-civ`. Settlement
suitability (`currentSettlementSuitability`, the "v1.30 one function") is
the next real milestone -- still needs route corridors, landmass quality,
and coast SDF (none built yet, per milestone 1's original dependency
trace) on top of everything now real. Factions, territory, provinces,
economy, and the Journey Planner remain untouched and further out.

## Phase 2 milestone 6 — settlement-suitability prerequisites: route corridors, landmass quality, coast SDF (2026-08-16)

Ports the last three affordance fields `currentSettlementSuitability()`'s
real `ctx` needs before settlement suitability itself becomes reachable:
`buildRouteCorridors` (reference line 5903), `buildLandmassQuality` (line
5970), `buildCoastSDF` (line 7462, always via the JFA/Euclidean backend
`{euclid:true}` -- the only path this port's production caller actually
uses).

**A real harness bug found and root-caused before trusting any of this
milestone's data.** The first extraction attempt reproduced `field[0..5]`
close to but not bit-identical to `golden_parity_carve.rs`'s fixture
(~1e-5 off) -- small enough to look like float noise, but this project's
own discipline doesn't accept "close" as a harness cross-check. Root
cause: that fixture was captured with `p.climate.w_iters=12` (an explicit
speed override, not the real default `70`), so wind/rain convergence --
and the carved `field` downstream -- genuinely differs at the reference's
own literal default. Setting `state.climate.wIters=12` before extraction
reproduced the fixture exactly. A parameter mismatch, not a wrong-seed or
wrong-code-path bug this time.

**Three real subtleties caught during porting, not just formula
transcription:**
- `currentSlopeField()` (raw `slopeAt(x,y)`, reference line 5661) is
  distinct from `currentSoil()`'s own inline `slopeAt(x,y)*GW` convention
  `build_slope_field` (milestone 1) already provides -- reusing the wrong
  one would silently double- or under-scale `buildRouteCorridors`'s cost
  field. Added `build_raw_slope_field` as the correct, separate input.
- `buildLandmassQuality`'s flood fill is **8-neighbour** (diagonals
  included) -- deliberately different from `build_water_bodies`'s
  (milestone 2) 4-neighbour below-sea fill. Component-labelling order
  doesn't affect the final partition the way the priority-flood heap's pop
  order decided lake shape, so this port's own stack-based traversal
  doesn't need to replicate the reference's flat-array stack mechanics
  index-for-index, only the connectivity rule.
- `buildCoastSDF` dispatches between a chamfer fallback and a true-Euclidean
  Jump Flooding Algorithm (`jfaDist`, Rong & Tan 2006) depending on
  `opts.euclid`; the only real caller in this port's scope always passes
  `{euclid:true}`, so `jfaDist` (log2(N) halving passes, each cell
  propagating its nearest seed cell's coordinate from 8 neighbours at the
  current step size) was ported, not the simpler chamfer path.

**Golden verification**: two cases reuse this crate's established fixture
configs (matching `golden_parity_carve.rs`/`golden_parity_resource_
potentials.rs`); a third, larger case (`gw=48 gh=40 seed=777 world=false`)
was added specifically because both established fixtures' tiny grids
(154/192 cells) genuinely produce **zero** nonzero corridor cells from the
real reference engine -- confirmed real (`buildRouteCorridors`'s own
comment: "SPARSE, like every other opportunity term", `CORRIDOR_KNEE=0.45`
is a strict threshold), not a bug, but an all-zero fixture would pass even
with an inverted min/max in the flanking-barrier logic. The larger case
(203/1920 cells nonzero) genuinely exercises that branch. `1e-4` tolerance
for the two continuous fields (corridors, coast SDF); landmass-quality
component count asserted exact (an integer). All three cases passed on the
first attempt.

- `cargo test/clippy -p cartalith-civ --all-targets`: clean (44 unit +
  13 golden tests). `cargo test/build/clippy --workspace`: no regressions.

**Milestone 7 scoping**: `currentSettlementSuitability`/`findSettlementSeeds`
themselves still need river network order (`_riverNet.order`, from
`buildRiverNetwork`, reference line 4494) on top of everything this
milestone lands. Checked, not assumed away -- and the finding is more
nuanced than "missing": `cartalith-hydrology::strahler_from_receivers`
(the pure Strahler solver `buildRiverNetwork` itself calls, reference line
4454) is already ported, and `WorldState.stream_order` is already
populated by it when `carve_rivers` is on (`cartalith-engine`, feeding
`ch.recv`/`ch.chan` from `build_channels`'s carve-pipeline channel
computation into the same ordering rule). **Not yet verified**: whether
`build_channels`'s receiver/channel logic is a semantic match for
`buildRiverNetwork`'s own independent channelization (it recomputes `recv`/
`chan` itself via a slope-area threshold + Tarboton-aspect receiver
selection, rather than necessarily reusing whatever the carve pipeline's
own channel computation does) -- if it matches, `ws.stream_order` may
already answer settlement suitability's `riverOrder` term directly with no
further porting; if the two channelization approaches differ, milestone 7
needs its own `buildRiverNetwork`-equivalent (reusing `strahler_from_
receivers`, just fed a different `recv`/`chan`). Milestone 7's own first
step is resolving this, not assuming either answer.

## Phase 2 milestone 7 -- settlement suitability / seed-finding: the "v1.30 one function" (2026-08-16)

The function `ROADMAP.md` originally named as this phase's landmark.
`buildSettlementSuitability`/`findSettlementSeeds` (reference lines
6319/6418) ported to `cartalith-civ`, golden-verified bit-close against
the real reference engine.

**River-network question resolved (milestone 6's open item).**
`cartalith_hydrology::build_channels` IS already a line-for-line port of
`buildRiverNetwork`'s channelization loop -- its own doc comment cites the
exact reference lines (4503-4522). The algorithm was never the problem.
The real finding: `WorldState.stream_order` is computed too early in the
pipeline for this specific caller. The reference's own
`carveRiverValleys()` explicitly nulls `_riverNet` at its very last line
(8783), so `currentSettlementSuitability()` always rebuilds the river
network fresh on the FINAL, post-carve `field`/`flowField` the next time
anything asks for it -- never reusing whatever was computed mid-carve.
`WorldState.stream_order` is computed at an earlier point in
`generate_terrain` (before the channel-lock stamp that follows it), so
it's stale for this one caller even though it remains correct for its own
original purpose. Fixed with `fresh_river_order()`, a thin wrapper that
reuses `build_channels`/`strahler_from_receivers` directly on
`ws.field`/`ws.flow_discharge` -- no second receiver-tree implementation
needed, just the right inputs.

**A real gap closed along the way**: `buildFloodField` (reference line
5634, TWI + discharge + lowland-proximity) had no port anywhere in this
crate. `buildSettlementSuitability`'s `ctx.flood` genuinely reads it in
production (unlike some other `ctx` fields that stay `null` in this port
for lack of an upstream source) -- ported as `build_flood_field`. No
geoid field exists in this port, so `field[i]-geoAt(i)` becomes just
`field[i]`, the same `geo: None` pattern `build_water_bodies` already
established for the same absence.

**A genuine threshold ambiguity found and resolved, not glossed over.**
First golden extraction attempt used `{thresh: SETTLE_SEED_THRESH}` (0.42)
-- matching what the reference's *interactive advisory debug view* passes
(lines 8461/11517) -- and found a real mismatch: 6 seeds instead of 5,
even though the suitability field itself was already bit-identical to the
fixture. Investigated rather than dismissed as noise: the reference
actually has two different real call sites with different thresholds. The
`settlement_seeds.json` **export** (line 12445:
`findSettlementSeeds(currentSettlementSuitability(),GW,GH)`, no opts) uses
the function's own bare default, `0.65` -- and since no interactive debug
view exists anywhere in this port, that export path is the only headless,
non-interactive real production caller and this port's own closest
analog. Re-extracted at `0.65`; both fixtures passed exactly.
`SETTLE_SEED_THRESH` (0.42) stays ported as a named `pub const` in
`cartalith-civ` for if/when an interactive advisory view is ever built
here -- just not what this milestone's golden fixture exercises.

Both cases (`gw=14 gh=11 seed=24601 world=false`, `gw=16 gh=12
seed=314159 world=true`) reuse the crate's standing fixture configs
(`w_iters=12`, matching every prior milestone). `field[0..5]` cross-checked
exactly against `golden_parity_carve.rs` before trusting either
extraction; determinism confirmed by running case 0 twice and diffing
byte-identical JSON. Suitability at `1e-4` tolerance (this crate's
convention); seeds checked by exact `(x, y, score)` triples in
score-descending order, not just count -- both passed after the threshold
fix, without needing to touch the underlying formula at all.

- `cargo test/clippy -p cartalith-civ --all-targets`: clean. `cargo
  test/build/clippy --workspace`: no regressions.

**Confirmed still missing before milestone 8** (factions/territory/
provinces/economy, block 2 proper): everything, genuinely -- no faction,
territory, culture, or economy logic exists anywhere in this port yet.
Milestone 8 needs its own scoping pass once picked up, the same way every
milestone before it did.

## Phase 2 milestone 8: settlement placement + faction assignment (2026-08-16)

`_civIterativeAutoWorld` (reference line ~25336, block 2's real
"auto-populate" entry point) mixes pure algorithm with direct DOM reads
(`document.getElementById('civNCap')` etc.) and `alert()` calls on failure
paths. Neither belongs in a pure Rust crate, so this milestone ports only
the deterministic core it calls, stopping before population/naming
(`_civSettleName`/`_civBasePopForKind`, culture/economy -- milestone 9+):

- **Land-component labelling** -- a fresh 4-connected flood fill over
  land cells (world-wrap aware), matching `_civIterativeAutoWorld`'s own
  inline pass exactly. **Deliberately not a reuse of
  `build_landmass_quality`'s flood fill** -- that one is 8-connected
  (diagonals), a different algorithm for a different purpose (area+
  capacity scoring, not per-cell landmass membership for factions).
  Reusing it here would have silently changed which cells count as "the
  same landmass." A unit test (`label_land_components_separates_
  diagonal_only_touching_islands`) pins exactly this distinction: two
  land cells touching only at a corner stay separate components under
  4-connectivity, where the 8-connected fill would merge them.
- **`_civSnapLand`/`_civSnapCoast`/`_civIsCoastal`** (reference lines
  ~20747/20841/20917) -- snap a suitability-maximum seed onto real dry
  land (Chebyshev-ring spiral search, no world-wrap -- a real reference
  quirk, `_civSnapCoast` DOES wrap, `_civSnapLand` does not, preserved
  exactly rather than "fixed" for consistency), then onto the shore when
  near the sea (harbour towns sit ON the water), plus ocean-port
  detection (`_civIsCoastal` always x-wraps unconditionally regardless of
  `state.world` -- another real, preserved quirk, not normalized away).
  `_civSnapLand` needed `_civLakeFlooded` (reference line 5737, a small
  sub-cell-flood-band check reading `WaterBodies::fill_level`, milestone
  2's own output) -- ported alongside it.
- **`_civAssignLandmassFactions`** (reference line ~25022) -- the genuinely
  intricate one: capacity-weighted seat apportionment across landmasses
  (iterative, capped by each landmass's own candidate count), concrete
  faction-id assignment, and for any landmass earning multiple seats, a
  suitability+spacing capital-seeding loop (5 attempts, halving minimum
  separation each attempt, falling back to top-suitability-regardless-of-
  spacing if the search never finds enough) followed by nearest-seed
  assignment for every other candidate on that landmass. Fully
  deterministic (fixed iteration over ascending-sorted landmass ids, no
  RNG) -- ported line-for-line, not reimplemented from the docstring's
  summary of what it does.
- **Settlement tier classification** (capital/city/town/village/hamlet by
  rank -- the `isCapital`/`isCity`/`isTown`/`isVillage` cascade inline in
  `_civIterativeAutoWorld`, ~lines 25409-25421) -- small, ported alongside
  the rest since it has no DOM dependency either.

`CIV_FACTIONS` (reference line 14568): confirmed `_civAssignLandmassFactions`
only ever reads `.length` from it (for `factionCount`) -- the 7-entry
roster (`'Unclaimed'` + 6 real factions, so `factionCount=6`) is
presentation data (names, colours) with zero algorithmic content this
milestone's output depends on. A plain `faction_count: i32` parameter is
sufficient; the full roster was not ported.

**Golden verification.** The reference's own candidate-building loop is
inline in `_civIterativeAutoWorld`, not a standalone callable -- extracted
by injecting a small harness-only function into the same `vm` context that
mirrors that loop verbatim (calling the reference's own `_civSnapLand`/
`_civSnapCoast`/`_civIsCoastal`/`_civAssignLandmassFactions` internally,
not a reimplementation) rather than trying to call a DOM-coupled function
directly. `state.tect.seed` (not `state.seed`) set correctly from the
start this time -- no fresh harness-seeding bug to rediscover.
`field[0..5]` matched `golden_parity_carve.rs`'s trusted values exactly on
the first extraction for both cases, and the extracted `seeds` matched
`golden_parity_settlement_suitability.rs`'s own already-verified seed list
exactly, before any of this milestone's own new data was trusted.

**Both fixtures genuinely exercise the multi-capital (K>1 seats) branch**
of `_civAssignLandmassFactions`, not a degenerate always-single-seat case
-- checked, not assumed, the same discipline milestone 6 established when
it found its own small fixtures produced all-zero output for a different
field. Case 0's candidates split across 2 landmasses (2 candidates on one,
1 on the other); with `factionCount=6`, the 2-candidate landmass earns a
second seat and exercises the spacing loop. Case 1's 5 candidates all land
on a single world-wrapped landmass, which earns seats up to its full
candidate count (5), so every candidate becomes its own capital via the
same branch. No third, larger fixture was needed.

All output (faction id, capital flag, settlement tier, coastal flag) is
categorical/discrete -- checked bit-exact per place, matching every other
categorical output in this crate. Both fixture cases passed on the first
attempt.

- `cargo test/clippy -p cartalith-civ --all-targets`: clean (36 unit +
  15 golden tests). `cargo test/build/clippy --workspace`: no regressions.

**Confirmed still missing before milestone 9** (territory/province
generation, `_civGenerateProvinces`/`getCivTerritory` -- the natural next
target): everything specific to territory/provinces, genuinely -- this
milestone reaches faction *assignment* (which settlement belongs to which
polity), not territory *shape* (the geographic boundary a faction
controls). Milestone 9 needs its own investigation pass into what
`_civGenerateProvinces` actually depends on before it can be scoped, the
same way this milestone's own scoping investigated `_civIterativeAutoWorld`
before committing to a boundary.

## Phase 2 milestone 9 -- settlement population + naming, and a dead-code seed quirk worth knowing about (2026-08-16)

Milestone 8's own investigation found territory/provinces genuinely
unreachable (no auto-generation path exists anywhere in the reference --
interactive paint + save/load only), so this milestone picked the other
candidate with a clean boundary and real inputs: `_civBasePopForKind`
(reference line ~23433, a trivial lookup) and `_civSettleName` (line
~20717, RNG-driven syllable-combination naming), the rest of
`_civIterativeAutoWorld`'s own `places.map(...)` closure milestone 8
stopped short of.

**A genuinely important, verified quirk, not assumed**: `_civSettleName`'s
RNG seed comes from `_civRng((state.seed||12345)*31337+999)` (reference
line 25339) -- but `state.seed` (distinct from `state.tect.seed`, the real
per-world terrain seed) is **never assigned anywhere in the reference
file**. Verified two ways: grepping every `.seed=` write site (the only
matches are unrelated -- `_sculptCtx.seed`, `opts.seed` for erosion
droplets, an export-metadata field that itself reads `state.tect.seed`),
and confirming live at runtime that `state.seed` reads `undefined`
immediately after a real `generate()` call, for both fixture configs. So
`state.seed||12345` always evaluates to the literal `12345` -- the civ-
naming RNG stream is seeded IDENTICALLY for every world ever generated,
regardless of that world's actual terrain seed. Combined with
`_civAssignLandmassFactions` (milestone 8, consumes no RNG) cycling
faction ids from `1` in ascending landmass order, this produces a real,
fully-explained, non-coincidental result: both fixtures' rank-1 settlement
(faction 1 in both) gets the exact same generated name
("Sevjuniana"), and this chains forward through every same-rank,
same-faction pair (case 0 and case 1's first three settlements, factions
1-3, produced byte-identical names pairwise; case 1's settlements 4-5,
unique factions, are unique). Documented in full in
`golden_parity_settlement_naming.rs`'s own module doc comment so this
doesn't read as a bug to whoever encounters it next.

`_civRng`'s generator body is `mulberry32` in disguise -- proved by hand
(XOR/OR commutativity + `ToInt32`'s idempotence under modular reduction
means `_civRng`'s never-explicitly-wrapped state is numerically identical
at every step to `mulberry32`'s explicitly `|0`-wrapped one, for any
realistic call count) rather than assumed, so this reuses
`cartalith_rng::Mulberry32` directly instead of a second hand-rolled
generator -- the only real difference is `_civRng`'s own seed-derivation
wrapper (`(seed>>>0)||1`), ported as `civ_name_rng()`.

**Two real bugs caught in the extraction harness itself, not in the Rust
port** -- worth recording since both would otherwise look like Rust bugs
to whoever debugs a similar mismatch later:
1. The reference file has **four** `<script>` blocks, not the one/two
   earlier milestones' own doc comments assumed. `_civRng`/`_civSettleName`/
   `_civAssignLandmassFactions` all live in block #2 (lines 14562-26721),
   requiring blocks #1+#2 concatenated. Worse: block #2 itself contains a
   comment discussing "this file's three sequential `<script>` tags" in
   prose -- a naive regex scan for the literal text `<script>` counts that
   occurrence too, miscounting every block after it. Fixed by using the
   real line numbers from a direct `grep -n` on the file instead of
   re-deriving boundaries by text search.
2. `SettlementPlacement.suit` (milestone 8, already correct) carries the
   settlement's ORIGINAL SEED score straight through unchanged after
   snapping (reference line 25398: `suit:s.score`, not a fresh lookup at
   the snapped position) -- milestone 8's own golden test never checked
   `.suit` itself, so this went unverified until this milestone's
   population formula (which reads `.suit` directly) exposed a harness bug
   that re-sampled `suit[]` at the wrong (snapped) position. Correct names
   with wrong population values was the exact signal that narrowed this to
   a `suit`-extraction bug, not an RNG/naming bug, before touching any Rust
   code. Fixed by mirroring the reference's real candidate-building loop
   (`_civSnapLand`/`_civSnapCoast`/land-component DFS) directly in the
   harness.

**Golden verification**: both fixture cases, names checked by exact string
equality (RNG-driven, no rounding ambiguity once the stream is confirmed
in sync), population checked as an exact `u32` (a `round()` of a
continuous formula removes sub-integer ambiguity). Both passed on the
first attempt after the two harness bugs above were fixed --
`field[0..5]` and `suit` per settlement cross-checked against
`golden_parity_carve.rs`/`golden_parity_settlement_suitability.rs`'s own
already-trusted values before any new data was accepted.

- `cargo test/clippy -p cartalith-civ --all-targets`: clean (36 unit + 20
  golden tests, counted directly not estimated). `cargo test/build/clippy
  --workspace`: no regressions.

**Confirmed still missing before milestone 10** (territory/provinces):
unchanged from milestone 8's own finding -- no auto-generation path exists
anywhere in the reference, purely interactive paint + save/load. Remains
blocked on a real design decision (raised with the owner), not scoped
further here.

## Phase 2 milestone 11: road network algorithm -- `buildTravelCost`/`roadDijkstra`/`buildRoadNetwork` (2026-08-16)

Ported `buildTravelCost` (reference line 3257), `roadDijkstra` (3275), and
`buildRoadNetwork` (3316) to `cartalith-civ`. Investigated (per
`PHASE2_SCOPE.md` milestone 11) before starting: `_civSeedVillages` reads
`ways` via `_civRoadProximityQuery`, genuinely blocked on a real road
network existing, not a false blocker; roads themselves turned out
reachable now, since these three functions are pure block-1 code (the
reference's own comment on `buildRoadNetwork` says so outright: "Pure").

**Crate placement, decided and documented rather than defaulted**: these
functions live in block 1 of the reference (well before the civ block),
a real signal weighed against `ARCHITECTURE.md`'s own text -- "later
subsystems (civ, urban morphology, assets) arrive as new crates depending
on `cartalith-engine`'s public types" naming `cartalith-civ` for exactly
this phase (`ROADMAP.md`). Road connectivity between settlements is
conceptually civ-layer regardless of which reference script block defined
the pure function first, and a new crate would duplicate `cartalith-civ`'s
existing zero-`gdext`/`WorldState`-read-only shape for no real benefit.
Landed in `cartalith-civ`.

**A genuine precision-regime distinction caught by reading the reference's
own comments, not by pattern-matching this crate's existing `MinHeap`**:
`roadDijkstra`'s own local heap is a DIFFERENT precision regime from
`build_water_bodies`'s (milestone 2) `f32`-priority `MinHeap`. The
reference's v1.89 comment confirms a `Float32Array`-backed heap was tried
here and measured WORSE (reverted) -- `roadDijkstra` deliberately keeps a
plain (untyped, therefore `f64`) JS array heap, matching the v0.70 comment's
documented "Float64 push priorities vs Float32 `dist` array" mismatch
(the actual fix for a real historical bug: a lazy heap pushing duplicate
entries that round-compare-equal at `f32` precision could re-push without
bound). Reusing the crate's existing `f32` heap here would have silently
diverged from the reference. Added a distinct `DijkstraHeap` (`f64`
priorities) rather than generalising the existing one -- same sift-up/
sift-down comparison operators (`<=`/`<`), different priority type,
genuinely a different heap instance per the reference's own design.

**Deliberately narrower than the reference's full API, with reasons
recorded, not silently dropped**: only the scalar single-source case of
`roadDijkstra` is ported (the reference's own v1.71 multi-source array
variant has no in-scope caller -- every call site this milestone reaches
passes a scalar source); the `edgeCost` v1.98 optional directional-cost
callback is omitted (no call site in scope passes one, and the reference's
own comment confirms every such call site is bit-identical to the
unconditional path ported here).

**Golden verification**: built a fresh Node harness needing only block 1
(2084-14556) -- unlike milestone 9, no civ-block code is needed here.
Reused milestone 9's own already-verified settlement `(x,y)` pairs
directly as `places` rather than re-deriving the civ pipeline in JS
(`build_road_network` only reads `.x`/`.y`, matching the reference's own
`buildRoadNetwork`). Stripped the reference's trailing "v0.67: boot"
auto-generate call from the extracted source (`GW=state.resW; GH=gridH(GW);
allocate(); withBusy('generating…',generate);`) -- left in place, it would
auto-run `generate()` with wrong parameters before the harness driver gets
a chance to set `state.tect.seed`/`GW`/`GH`, the same class of pitfall
milestone 9 hit with the auto-boot's `allocate()` call.
`field[0..5]` cross-checked against `golden_parity_carve.rs`'s trusted
values before trusting the extraction -- matched exactly, both cases.
Both fixture cases passed at `1e-4` (cost field) / bit-exact (edge
topology, cell-index paths) on the first attempt.

**Real terrain data exercised the "unreachable place" branch, not a
synthetic-only test**: case0's 3 places produce only ONE MST edge, not the
two a fully-connected 3-node MST would have -- place index 1 sits on a
landmass the cost-distance search from place 0 never reaches in this
generated world, so the Prim loop's `bu===Infinity` guard correctly
breaks early. Confirmed by a synthetic unit test covering the same branch
deliberately, then found again for real by the golden fixture.

- `cargo test/clippy -p cartalith-civ --all-targets`: clean (47 unit + 22
  golden tests). `cargo test/build/clippy --workspace`: no regressions.

**Investigated for milestone 12, not implemented**: does the civ
auto-populate flow (`_civIterativeAutoWorld`) ever call `buildRoadNetwork`
to connect its own auto-placed settlements? **No** -- grepped every call
site of `buildRoadNetwork`/`buildTravelCost` in the reference: the only
caller is `buildRoadsOp()` (line 4816), which reads `state.places` (the
*manual*, user-clicked marker list, a distinct tool from the civ
auto-populate settlements milestones 8-9 built). The civ auto-populate
flow's own road system is a **different, larger** algorithm entirely:
`civWays` (line 14758, genuinely "auto-generated," unlike `civTerritory`)
is built by `_civHierarchicalNetwork` (land routes) + `_civMstRoutes`
(sea routes, port-to-port) + `_civPreferSeaRoutes` (chooses land vs. sea
per edge by cost, preserving connectivity) -- none of which this milestone
ported or read in depth. `_civSeedVillages`'s `ways` parameter is
`civWays`, not `buildRoadNetwork`'s output -- **milestone 11's work does
not unblock village seeding**; that needs `_civHierarchicalNetwork` (a
separate, larger port) first. What milestone 11 *does* provide, real and
useful on its own: the algorithm behind the manual "Generate Roads" tool
(`buildRoadsOp`), reachable as a future Godot-UI feature independent of
the civ auto-populate pipeline. Recording this precisely so milestone 12
isn't scoped on the wrong assumption.

## UI/UX catch-up: Phase 2's civilisation layer is now visible (2026-08-16)

Every Phase 2 milestone through 11 (affordance fields, water bodies,
biomes, carrying capacity, resources, route corridors/landmass/coast-SDF,
settlement suitability, faction assignment, population/naming, roads)
landed with zero wiring to `cartalith-godot` and zero visual
representation -- the map view showed terrain only. Requested explicitly
by the owner this session ("with every milestone and phase the GUI and UX
should be updated as well... use a separate agent") after Phase 2 reached
9+ golden-verified milestones with nothing to show for it on screen. This
entry establishes that pattern going forward, not just a one-off fix.

**Part 1 -- chained the civ pipeline end-to-end for the first time.**
`compute_civilisation()` (`cartalith-godot/src/lib.rs`, new) calls the
full Phase 2 chain in dependency order (mirroring
`cartalith-civ/tests/golden_parity_settlement_placement.rs`'s own
`compute_placements` helper, the canonical reference for call order):
water bodies -> biome -> lithology/soil -> water access/carrying capacity
-> resources -> route corridors/landmass quality/coast SDF -> settlement
suitability -> seeds -> land-component labelling -> snap-to-land/coast ->
faction assignment -> settlement placement -> naming/population -> travel
cost -> road network. Runs automatically at the end of `generate()`/
`generate_world_structure()` (on the same background `Thread` `main.gd`
already uses -- no new threading code needed, civ computation is just
more Rust work inside a call that was already off-thread), stored as
`Option<CivData>` on `WorldGen`; `None` for a loaded save (`SAVEFILE_COMPAT.md`
doesn't store the substrate fields -- `crust_field`/`boundary_type`/
`shear_field`/`age_field` -- this pipeline needs) or before the first
`generate()`.

New `#[func]` accessors: `get_settlements() -> Array<VarDictionary>` (x/y/
name/population/kind/faction/capital/coastal per settlement -- `VarDictionary`
not `Dictionary<K,V>`, since gdext 0.5.5's `Dictionary` is now a *typed*
homogeneous-value container and these values are heterogeneous) and
`get_roads() -> Array<PackedVector2Array>` (one array of grid-cell
`(x,y)` points per MST edge, the real terrain-following path, not a
straight line between endpoints).

**Honest scope note on which road algorithm this uses**: `build_road_network`
(milestone 11) is, per that milestone's own investigation, the algorithm
behind the reference's *manual* "Generate Roads" tool (`buildRoadsOp`,
user-clicked markers) -- not the civ auto-populate's own road system
(`civWays`, built by the separate, unported `_civHierarchicalNetwork`+
`_civMstRoutes`). This wiring deliberately reuses the one already-ported,
golden-verified pathfinding algorithm to connect the *auto-placed* civ
settlements instead, since it's the same underlying MST-over-cost-distance
technique and produces a real, useful result now rather than leaving roads
absent until `_civHierarchicalNetwork` is ported. A deliberate adaptation,
not a literal port of what the reference does at this call site -- worth
knowing if `_civHierarchicalNetwork` lands later and this wiring should be
swapped to the "real" civ road system.

**Part 2 -- new Godot-side visualisation**, matching the existing light
"parchment" theme (no new visual language introduced):
- `main.tscn` gains a "MAP LAYERS" settings card (same card pattern as
  World Structure/Advanced Features) with a "Show settlements & roads"
  checkbox, default on, plus a new `MapOverlay` `Control` node as a
  sibling of `%MapView` under the same `MarginContainer` (`MapMargin`)
  -- `MarginContainer` lays out every child to the same rect, so the two
  overlap automatically with no manual rect-syncing needed.
- `map_overlay.gd` (new): reproduces `%MapView`'s own
  `STRETCH_KEEP_ASPECT_CENTERED` fit/letterbox math (`_displayed_rect()`)
  so grid-cell coordinates land on the real pixels the terrain texture
  occupies, at any window size. Draws road polylines, then settlement
  markers sized by tier (capitals get a ring, standing out from hamlets)
  and coloured by faction using the Okabe-Ito colourblind-safe qualitative
  palette (6 hues, matching `CIV_FACTION_COUNT` exactly) -- chosen
  deliberately independent of the UI's own parchment theme, since this is
  data-driven map content (which faction owns this settlement) the same
  way the terrain renderer's biome colours are theme-independent.
  `_gui_input` hit-tests mouse position against marker screen positions
  (tier-radius + a small hit-test pad) for hover; a floating info card
  (name, tier, population, formatted `17.7k`-style) draws near the cursor,
  styled to match the parchment theme's card/border colours.
- `main.gd`: fetches `get_settlements()`/`get_roads()` right after
  `build_color_texture()` in `_on_generate_done`, hands them to the
  overlay, and appends a live settlement count to the status label. Clears
  the overlay (empty arrays) in `_on_save_file_selected` since a loaded
  save carries no civ data. The layer-visibility checkbox toggles
  `MapOverlay.visible` directly.

**Verified hands-on, not just headless** (this session's own established
discipline -- two real UI bugs earlier were only caught this way): ran the
actual windowed app, generated a real 512x512 world, confirmed on screen
-- 20 settlements rendered with visibly distinct per-faction colours,
capitals clearly larger with a ring, roads following real terrain (not
straight lines) between markers, and hovering a marker produced a correct
info card (`"Torvtorgskaltorvbay (Capital)" / "Population 17.7k"`, a real
RNG-generated name from milestone 9's culture tables). `godot4 --headless
--quit main.tscn`: clean, extension loads.

**Verification**: `cargo build -p cartalith-godot`, `cargo clippy -p
cartalith-godot --all-targets`: clean, no warnings. `cargo build
--workspace`: clean.

**What this doesn't cover**: territory/border rendering (Phase 2
milestone not yet implemented -- nothing to show), the full interactive
civ editor (faction management, label editing, painting), village
markers (village seeding itself still blocked on `civWays`, per above).

## GPU-safe noise redesign: the pipeline's actual first GPU milestone (2026-08-16)

`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1 -- the first real code
written under `DECISIONS.md` §7a's "principled equivalence" carve-out.
The GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`) found
`cartalith_noise::hash`'s JS-matching output depends on IEEE-754
*double*-precision rounding at an intermediate magnitude (~2^61) --
unrepresentable in `f32`, and WGSL has no working `f64` support on this
toolchain regardless of hardware feature support (`naga` doesn't
implement `enable f64;`). Since `hash`/`vnoise` feed nearly the entire
terrain substrate (domain warp, crustal heterogeneity, the height
formula's fractal terms), this was the actual blocker for GPU-accelerating
anything upstream, not one item among many.

**Built, in `cartalith-noise`**: `gpu_hash(x, y, s) -> u32` and
`gpu_vnoise(x, y, s) -> f32` -- a **deliberate redesign**, not a patched
port. Construction: single-round PCG3D (Mark Jarzynski & Marc Olano,
"Hash Functions for GPU Rendering," *Journal of Computer Graphics
Techniques*, vol. 9, no. 3, 2020, jcgt.org/published/0009/03/02/), a hash
designed specifically for GPU shaders. Every operation is pure `u32`
wrapping arithmetic (multiply/add/xor/shift) until the final `u32 -> f32`
conversion, which is a fully-specified IEEE-754 round-to-nearest
operation on both platforms -- unlike `hash`'s problem (`f32 -> u32` for
out-of-range values is implementation-defined/saturating), this direction
has no platform-dependent behaviour to worry about. `hash`/`vnoise`
themselves are completely untouched -- every existing golden-parity test
across the workspace that depends on them for exact JS matching passes
unmodified (`cargo test --workspace`, confirmed before and after).

**Built, in `cartalith-gpu`**: `shaders/gpu_noise.wgsl`, mirroring the
Rust side operation-for-operation, plus `init_gpu_safe_noise()` and
`gpu_safe_noise_grid_cpu()` (reuses the pilot's existing `dispatch_gpu` --
it was already generic over which shader/pipeline the `GpuContext` was
built with, so no dispatch-path changes were needed).

**Verification -- CPU vs. GPU, not vs. JS** (`DECISIONS.md` §7a: this pair
was never required to match JS, only each other): at 512x512 on this
session's real hardware (AMD Radeon RX 7800 XT, Vulkan), **0/262144
cells exceeded a `1e-5` tolerance; max absolute difference was
1.28e-6** -- effectively exact, with a comfortable margin. Set as a real
`#[test]` assertion (`gpu_safe_noise_matches_cpu_reference_at_real_field_
size`), not just logged, since this pair is *expected* to agree by
construction, unlike the old pilot's documented-as-failing comparison.

**Real timing, measured fresh** (the pilot's own 4.46x/15.65x/19.55x
numbers were for the non-portable `hash` kernel and don't carry over --
this hash does more work per call, two full PCG3D mixing rounds):

| Size | GPU dispatch+readback | CPU (single-thread) | Ratio |
|---|---|---|---|
| 128x128 | 1.31ms | 135µs | 0.10x (GPU loses, dispatch overhead) |
| 512x512 | 870µs | 2.48ms | 2.85x |
| 1024x1024 | 900µs | 9.35ms | 10.39x |
| 2048x2048 | 2.88ms | 34.37ms | 11.94x |

Real, legitimate speedups at the sizes that matter (this port's real
default is 2048, per the resolution-control fix earlier this session).

- `cargo test -p cartalith-noise -p cartalith-gpu`: clean, all new tests
  pass (CPU/GPU determinism, output-range sanity, lattice-boundary
  continuity, the real-field-size correctness gate, timing). `cargo
  clippy -p cartalith-noise -p cartalith-gpu --all-targets`: clean, zero
  warnings. `cargo test --workspace`: all green, zero failures, confirming
  the existing JS-matching noise tests are genuinely untouched.
- **Not independently confirmed this pass**: `cargo build --workspace`
  hit a transient file-lock (`cartalith_godot.dll`, "Access is denied")
  -- environmental, not a code regression: a concurrent session pass had
  just run the real windowed app (see the UI/UX entry immediately above),
  which was still holding the DLL open. `cartalith-noise`/`cartalith-gpu`
  build and test clean in isolation, and `cargo test --workspace`
  (a separate, already-clean compilation) covers the same ground without
  hitting the lock.

**What this doesn't do**: wire the new noise into any actual pipeline
stage. This milestone is the primitive alone, verified standalone --
`GPU_LAYER_INTEGRATION_SCOPE.md`'s own scope boundary. The real next
GPU milestone is domain warp / crustal heterogeneity / the height
formula -- the first actual pipeline stage that can now move to GPU,
scoped separately once reachable.

## Phase 2 milestone 12 — civ auto-populate road network topology (2026-08-16)

`PHASE2_SCOPE.md` milestone 12: `_civHierarchicalNetwork` (reference HTML
line ~21526) plus its direct helpers -- the real dependency
`_civSeedVillages` needs (`civWays`), confirmed by reading every real call
site of it and of milestone 11's `build_road_network`: the auto-populate
flow (`_civIterativeAutoWorld`, lines ~25581-25680) calls
`_civHierarchicalNetwork(places,{})` with empty opts (no `existingWays`)
and never calls `_civPreferSeaRoutes` at all -- that function is only used
by the separate `_civAutoRoutes` (manual-tool-adjacent) caller.
`build_road_network` (milestone 11) is a different, simpler system used
only by the *manual* "Generate Roads" tool.

**Real scope finding, not assumed**: `_civHierarchicalNetwork` turned out
substantially larger than milestone 11 estimated when this milestone was
first scoped -- THREE passes (Prim MST, min-degree-fill by settlement
tier, Floyd-Warshall shortcut-detour-relief), not two, plus a
corridor-consolidation + Catmull-Rom-smoothing + road-class/name-emission
step (reference lines ~21670-21739) that turns raw edges into pretty,
deduplicated polylines for rendering. **Split the scope here**: ported the
raw three-pass topology (`civ_hierarchical_network_topology`, new in
`cartalith-civ`) -- what `_civSeedVillages`'s `_civRoadProximityQuery`
needs functionally (distance to nearest road cell), even unsmoothed.
Corridor consolidation/smoothing/classification is real, separate work
(needs `_civSmoothPath`, `_civTerrainValidTest`, road-class assignment --
none read or ported) deferred to its own future milestone rather than
rushed under budget pressure.

**Built**: `civ_biome_friction`, `civ_navigable_river_discount`,
`civ_routing_grid`, `civ_enhanced_travel_cost`,
`civ_apply_settlement_gravity`, `civ_snap_finite`, `civ_trace_path`,
`civ_hierarchical_network_topology` in `cartalith-civ`. Reuses
`road_dijkstra` (milestone 11) directly -- its scalar single-source,
no-`edgeCost` signature already matches every call this milestone makes.
A real bug caught before it shipped: `river_flow_thresh` needs the real
per-world `map_width_km`, not a hardcoded `800.0` default -- would have
silently diverged for any non-default map width; threaded through as a
real parameter instead.

**Golden verification**: fresh Node harness, blocks #1 (2083-14556) + #2
(14562-26720) concatenated (per `golden_parity_settlement_naming.rs`'s own
documented block boundaries). `_civHierarchicalNetwork` only *returns* the
post-consolidation `ways` -- the raw `allEdges` this port's topology
matches was captured by instrumenting the extracted source, inserting a
capture statement immediately before the reference's own `/* Classify,
CONSOLIDATE and smooth. */` comment. Settlement inputs reused directly
from `golden_parity_settlement_naming.rs`'s own already-verified
`(x,y,faction,kind)` fixture rather than re-derived. `field[0..5]`
cross-checked against a direct `generate_terrain` call before trusting
the extraction.

Both fixture cases are real, meaningful edge cases, not synthetic ones:
case0 (3 capitals) has one settlement genuinely **unreachable** from the
other two over the terrain-cost grid (`degree_of=[1,0,1]`, a single MST
edge, min-degree-fill correctly finds no finite-cost candidates rather
than looping or panicking); case1 (5 capitals, each requiring tier degree
5) exercises the fill pass hitting its natural ceiling instead of the
requirement -- every place reaches degree 4 (the maximum possible with 4
other places), the network becomes the complete graph K5 (10 edges), and
pass 3 (shortcut-detour-relief) correctly finds nothing left to add.
Edge topology and usage counts checked exactly (integers/categorical, no
float tolerance needed) -- both cases passed on the first real attempt
after fixing the `river_flow_thresh` parameter bug above.

- `cargo test/clippy -p cartalith-civ --all-targets`: clean (2 new golden
  tests, zero new clippy warnings -- the file's other pre-existing
  excessive-float-precision warnings are milestone 11's own test file,
  untouched here). `cargo test --workspace`/`cargo build --workspace`: no
  regressions.

**Milestone 13+ not yet scoped**: sea routes (`_civMstRoutes` with
`isSea=true`) -- confirmed to have real, separate new dependencies
(current/wind-costed sea edges via `_civSeaTimeEdgeCost`, sea-lane
augmentation beyond the MST tree, `_civSmoothPath` Catmull-Rom smoothing)
not shared with this milestone's land network at all. Corridor
consolidation/smoothing for the land network (deferred above) is a
separate, likely-smaller follow-up once `_civSmoothPath` exists for sea
routes to use too -- worth doing once, not twice.

## Phase 2 milestone 10 -- territory assignment: cost-distance Voronoi, population-weighted (2026-08-16)

`PHASE2_SCOPE.md` milestone 10 / `DECISIONS.md` §7b. The first Phase 2
milestone with **no JS reference to port at all** -- the reference has no
algorithmic territory generation whatsoever, only a hand-painted brush
tool and save/load restoration of an already-painted raster. This is the
owner's own design decision (§7b), implemented here, not a discovery made
during porting.

**Algorithm**: for every capital settlement (milestone 8's `capital`
flag), run `road_dijkstra` (milestone 11, reused directly -- the same
private function `build_road_network` already calls, made available in
this module) from that capital's cell over `build_travel_cost`'s real
terrain-cost field. Each cell's *effective* distance is its raw
cost-distance divided by `territory_weight(pop) = 1 + ln(1 + pop/pop_ref)`
(§7b's own suggested form), so a more populous capital's territory reaches
farther for the same terrain cost. Each land cell's owner is the faction
of whichever capital reaches it at the lowest effective distance; a
multi-capital faction's territory is the union of every one of its
capitals' independently-competing zones. Cells no capital's Dijkstra tree
ever reaches (water, or a genuinely disconnected landmass) stay unowned
(faction `0`) -- the same unreachability mechanism `build_travel_cost`'s
water-impassable convention already gives `build_road_network`, no
separate sea-level check needed.

**`pop_ref` choice, documented not arbitrary**: `15000.0`, exactly
`civ_base_pop_for_kind(SettlementKind::Capital)` -- the reference
population scale for a capital before suitability/RNG variance. A real
capital's actual population (`name_and_populate_settlements`:
`base*(0.7+suit*0.8)*(0.8+rng*0.4)`) ranges roughly 8,400-27,000;
anchoring `pop_ref` at the base value keeps the weight spread well-behaved
across that real range (`w` from ~1.41 at the low end to ~2.10 at the high
end) instead of picking a number with no connection to this port's actual
population scale.

**Verification, per §7a/§7b's own stated standard**: no golden-parity test
is possible (nothing to diff against). Eight unit tests instead, covering
what a JS diff would have covered by construction: a capital's own cell is
always self-owned (trivially, distance 0 always wins); on a flat,
fully-passable strip with two equal-population capitals, ownership splits
at the geometric midpoint (the classic unweighted-Voronoi boundary,
confirming the weight function is inert at equal population); with one
capital at 100,000 and a rival at 5,000 on the same strip, the geometric
midpoint flips to the larger capital -- the actual population-weighting
behaviour, not just present but *measured* to move the boundary the
expected direction; unreachable cells (an impassable barrier cell)
correctly stay unowned rather than defaulting to whichever capital was
processed first; a non-capital settlement projects no territory of its
own; a two-capital faction's territory is confirmed as the union of both
zones, not just the first one checked.

Deliberately not attempted this pass: rendering territory as a real colour
overlay in the Godot map view (the UI-per-milestone process this session
established) -- would need a new `cartalith-godot` binding plus
`map_overlay.gd` wiring, genuine scope beyond this milestone's own
`cartalith-civ`-only diff. Flagged as the natural next UI/UX-catch-up
target, not silently skipped.

- `cargo test -p cartalith-civ`: clean, 8 new unit tests (50 total in the
  crate's own unit-test binary). `cargo clippy -p cartalith-civ
  --all-targets`: clean (one real `erasing_op` lint caught in a test's
  own `owner[0*5+0]` index expression, fixed to `owner[0]` -- a
  leftover-from-drafting mistake, not a design issue). `cargo test
  --workspace`/`cargo build --workspace`: no regressions.

**Milestone 13/14 unchanged from milestone 12's own scoping**: sea routes
and road corridor consolidation/smoothing, per the entry above.
`_civGenerateProvinces` (sub-partitioning owned territory into
per-settlement provinces) is now genuinely reachable for the first time
-- territory itself exists -- but not scoped or attempted here.

## GPU layer integration milestone 2 -- domain warp + crustal heterogeneity on GPU (2026-08-16)

`compute_warp`/`compute_heterogeneity` (`cartalith-terrain/src/lib.rs`)
ported to GPU (`cartalith-gpu`), building on milestone 1's `gpu_hash`/
`gpu_vnoise` (`cartalith-noise`). Non-`world` branch only (no `pfbm`
periodic-noise equivalent yet) -- deliberately deferred, see
`GPU_LAYER_INTEGRATION_SCOPE.md`.

**Built**: `cartalith_noise::gpu_fbm` (a 6-octave combinator over
`gpu_vnoise`, all-`f32`, same octave-combining shape as the JS-matching
`fbm`). `cartalith-gpu` gains `gpu_warp.wgsl`/`gpu_heterogeneity.wgsl`,
`init_gpu_warp`/`init_gpu_heterogeneity`, `dispatch_gpu_warp`/
`dispatch_gpu_heterogeneity`, and CPU reference twins
(`gpu_warp_grid_cpu`/`gpu_heterogeneity_grid_cpu`). `init_gpu_with`
refactored to take the bind-group layout as a parameter (`WarpParams`
needs 2 storage outputs, `HeteroParams` needs 3 storage inputs + 1
output, neither fits the pilot's original single-uniform/single-storage
layout) -- the three existing call sites (`init_gpu`/`init_gpu_f64`/
`init_gpu_safe_noise`) pass the original layout unchanged, so this is a
pure extension, not a behaviour change to anything already verified.

**A real, measured, structural finding, not a bug**: `gpu_heterogeneity`
(one `gpu_fbm` call per cell) matches its CPU twin within
`GPU_SAFE_NOISE_TOLERANCE` (`1e-5`) exactly -- 0/262144 cells at 512x512,
max diff ~6e-7, confirming `gpu_fbm` itself carries no new precision gap.
`gpu_warp` (which chains TWO nested `gpu_fbm` evaluations -- `qx`/`qy`
first, then `wx`/`wy` sampled at a position computed from `qx`/`qy`)
diverged by up to 1.18e-4 at the same tolerance: sub-epsilon residual
float-scheduling differences (the same FMA-contraction-scale category the
pilot's own tolerance comment already named) in the first evaluation
become a coordinate perturbation feeding a second, full 6-octave
evaluation, which amplifies them. Given a real, isolated cause (proven by
comparing against the passing single-evaluation case, not assumed), added
`WARP_TOLERANCE = 2e-4` -- set just above the actually-measured max, not
loosened further, matching `PARITY_TESTING.md`'s rule applied here to a
GPU/GPU pair instead of a JS/Rust one. At that tolerance: 0/524288 cells
(both axes) exceed it, both `gpu_warp`/`gpu_heterogeneity` deterministic
across repeated runs, all output finite and within the expected physical
range.

**Real timing** (128/512/1024/2048, same honest methodology as milestone
1 -- the pilot's own numbers don't carry over, these functions do 4x/1x
as much per-cell work respectively):

| Size | `gpu_warp` (24 `gpu_vnoise` calls/cell) | `gpu_heterogeneity` (6 calls/cell) |
|---|---|---|
| 128² | 3.46x | 0.87x (GPU loses, dispatch overhead) |
| 512² | 46.18x | 7.32x |
| 1024² | 79.22x | 15.54x |
| 2048² | 80.37x | 16.74x |

`gpu_warp`'s ratios are dramatically higher than milestone 1's bare-noise
kernel (up to 80x vs ~12x) -- makes sense: CPU cost scales with total
octave-call count (24 vs 6 vs milestone 1's 1), while GPU dispatch/
readback overhead stays roughly fixed, so costlier per-cell kernels see
proportionally larger GPU wins.

**Qualitative check** (`DECISIONS.md` §7a, "judged by looking at it"): a
grayscale PGM of a real 256x256 GPU `warp_x` field is written to a temp
path by `gpu_warp_debug_image_written_for_visual_check` for manual visual
inspection (no banding/lattice-artifact check automated -- this is a
by-eye test, the PGM is the deliverable).

**A real, flaky, pre-existing-class issue found, not introduced**:
`cargo test -p cartalith-gpu` alone (default thread-parallel) hit one
`STATUS_ACCESS_VIOLATION` crash partway through -- not attributable to
any specific test or assertion. `--test-threads=1` reproduces cleanly
(18/18 pass) every time, and a full `cargo test --workspace` run (where
cargo runs each crate's test binary as a separate process rather than
maximally parallelizing within one) also passed clean. Read as GPU-driver-
level resource contention from several tests creating/tearing down
`wgpu` devices concurrently within one process -- a real fragility this
crate's growing GPU-context-per-test count made more likely to surface,
worth knowing about, not silently worked around by weakening a test.

- `cargo build -p cartalith-terrain -p cartalith-gpu`: clean.
  `cargo test -p cartalith-noise -p cartalith-gpu`: 18/18 (`cartalith-gpu`,
  serial or as part of the full workspace run) + existing `cartalith-noise`
  tests, all pass. `cargo clippy -p cartalith-terrain -p cartalith-gpu
  -p cartalith-noise --all-targets`: clean (two expected `dead_code`
  warnings on functions only reachable via `#[cfg(test)]`, same class the
  pilot's own `dispatch_gpu`/`vnoise_grid_cpu` already have in a
  non-test build -- not a real issue). `cargo test --workspace`/`cargo
  build --workspace`: no regressions, `cartalith-terrain`'s existing
  `compute_warp`/`compute_heterogeneity` golden-parity tests untouched
  and passing (confirms the CPU functions were genuinely not modified).

**Milestone 3 (not scoped here)**: `compute_height` itself is the next
candidate, per `GPU_LAYER_INTEGRATION_SCOPE.md`'s own note -- its real
upstream dependency chain (boundary stress, flexure, orogeny, JFA Voronoi
plate assignment) needs the same investigation-before-scoping pass every
milestone in this session has had, not assumed reachable.

## GPU layer integration milestone 3 -- the height formula (`compute_height`) on GPU (2026-08-16)

`compute_height` (`cartalith-terrain/src/lib.rs:1001`) ported to GPU
(`cartalith-gpu`), treating its upstream input fields (`base_field`,
`stress`, `flex`, `hetero`, `age`, `warp_x`/`warp_y`, `oro`) as opaque
GPU buffers -- this milestone deliberately does NOT attempt plate
assignment/stress/flexure/orogeny's own GPU portability, per
`GPU_LAYER_INTEGRATION_SCOPE.md`'s own scope. Non-`world` branch only,
matching milestones 1-2's own deferral.

**Built**: `cartalith_noise::gpu_ridged` (6-octave ridged multifractal
over `gpu_vnoise`, same fold-and-square transform as the JS-matching
`ridged`, all-`f32` -- the noise-combinator gap milestone 2 anticipated).
`cartalith-gpu` gains `gpu_height.wgsl`, `init_gpu_height`,
`dispatch_gpu_height`, and a CPU reference twin (`gpu_height_grid_cpu`).
`init_gpu_with` gained an automatic `max_storage_buffers_per_shader_stage`
bump, derived from each kernel's own bind-group layout (counting its
`Storage`-typed entries) rather than a hand-picked number -- this
kernel's 9 storage buffers (8 inputs + 1 output) exceed
`downlevel_defaults()`'s conservative baseline, and milestone 2's own
3-call-site extension pattern (add a parameter, existing calls
unaffected) wasn't itself enough here since the *limit*, not just the
*layout*, needed adjusting. Self-contained, backward-compatible: the
existing 4 call sites (`init_gpu`/`init_gpu_f64`/`init_gpu_safe_noise`/
`init_gpu_warp`/`init_gpu_heterogeneity`) are unaffected and scales
automatically for any future kernel's own buffer count.

**`oro`'s absence changes the formula, not just an additive no-op** --
unlike `warp_x`/`warp_y` (zero-filled when absent, matching
`.map_or(0.0, ...)`'s CPU behaviour exactly), `compute_height`'s `t =
match oro { Some(o) => o[i] + stress.min(0.0), None => stress }` is a
genuine branch. The shader takes an explicit `has_oro: u32` param and a
`select()`; a dedicated regression test
(`gpu_height_has_oro_true_changes_the_formula`, distinctly different oro
data vs. `has_oro=false`) proves the branch is genuinely wired, not
silently ignored either way -- the kind of thing a naive all-buffers-are-
optional-and-zero-filled port would have gotten wrong.

**Verification, no golden-parity possible (`DECISIONS.md` §7c)**: both
`ridged=false` and `ridged=true` verified against `gpu_height_grid_cpu`
at 512x512 with 5 distinct, non-trivial synthetic input fields (so a
mis-wired buffer binding -- e.g. `stress` accidentally reading `flex`'s
buffer -- would show up as a wrong-shaped result, not pass by
coincidence): **0/262144 mismatches, max observed absolute difference
1.19e-7** -- essentially `f32`'s own machine epsilon. This kernel has
only ONE noise evaluation per cell (unlike `compute_warp`'s two nested
ones), the same shape as milestone 2's clean `gpu_heterogeneity` result,
not its compounding `gpu_warp` one -- given its own `HEIGHT_TOLERANCE`
(`= GPU_SAFE_NOISE_TOLERANCE`, the tightest this crate uses) rather than
reusing the looser `WARP_TOLERANCE` a first guess might have borrowed
without checking what was actually measured. Deterministic across
repeated runs, all output finite and within the expected physical range.
A debug PGM of a real GPU height field written for by-eye inspection.

**Real timing** (single-threaded CPU vs. GPU dispatch+readback, same
honest methodology as milestones 1-2):

| Size | `gpu_height` (1 `gpu_fbm`/`gpu_ridged` call/cell + 8 buffer reads) |
|---|---|
| 128² | 0.86x (GPU loses, dispatch overhead) |
| 512² | 5.17x |
| 1024² | 8.13x |
| 2048² | 4.84x |

The drop from 1024² to 2048² is reported as measured, not smoothed over
-- a plausible cause (this kernel reads 8 input buffers vs. `gpu_warp`/
`gpu_heterogeneity`'s 2-4, so it may be memory-bandwidth-bound rather
than compute-bound at scale) is not yet investigated; worth a look if
this kernel's throughput matters later, not chased down here.

- `cargo test -p cartalith-noise -p cartalith-terrain -p cartalith-gpu`:
  all pass (23/23 in `cartalith-gpu`, includes 5 new height tests; serial
  or as part of a full workspace run, same known concurrent-GPU-context
  flake milestone 2 already documented). `cargo clippy -p cartalith-noise
  -p cartalith-terrain -p cartalith-gpu --all-targets`: clean (one real
  `clippy::type_complexity` warning on a new test helper's 5-tuple return
  type, fixed with a named type alias; the pre-existing `dead_code`
  warnings on test-only dispatch functions are the same known class
  milestone 2 already documented, not new). `cargo test --workspace`/
  `cargo build --workspace`: no regressions, `cartalith-terrain`'s
  existing `compute_height` golden-parity tests untouched and passing
  (confirms the CPU function was genuinely not modified).

**Also fixed**: `GPU_LAYER_INTEGRATION_SCOPE.md` had picked up a doc-merge
artifact -- milestone 2's own "Done" completion note had been misplaced
under milestone 3's heading (a concurrent-edit collision from earlier in
this session), leaving milestone 2's section without its completion
record and a stale duplicate "milestone 3: not yet scoped" section
alongside the real one. Corrected in place as part of this entry's own
doc update, not left standing.

**Milestone 4 (not scoped here)**: plate assignment/stress/flexure/
orogeny's own GPU portability is the natural next candidate -- this
milestone deliberately treated them as opaque buffers and did not
investigate them. One correction already on record
(`GPU_LAYER_INTEGRATION_SCOPE.md`): plate assignment uses JFA (Jump
Flooding Algorithm), which is specifically designed to parallelize well
on GPU, unlike the genuinely poor-fit graph/sequential algorithms
(flow accumulation, priority-flood, Dijkstra/MST) -- a hypothesis worth
checking, not yet a finding. Real investigation of `cartalith-terrain`'s
actual plate-assignment/stress/flexure/orogeny code is milestone 4's
first step, the same discipline every milestone here has had.

## Phase 2 milestone 15 -- village seeding (`_civSeedVillages`)

Ported `_civSeedVillages` (reference HTML line ~25164) plus its direct
helpers `_civVillageAcceptProb` (~25159) and a milestone-12-topology-
adapted `_civRoadProximityQuery` (~25127) to `cartalith-civ`. Confirmed
reachable independent of milestones 13/14 (sea routes, corridor
consolidation/smoothing): `_civSeedVillages` only needs road-proximity
*distance*, which milestone 12's raw, unsmoothed
`civ_hierarchical_network_topology` edges already provide -- smoothing is
a rendering concern, not a functional one for this pass.

**RNG stream threading, a real gap closed before this milestone could
start**: the reference shares ONE `rng` closure across the whole
`_civIterativeAutoWorld` flow -- settlement placement, naming, THEN
village seeding draw from one continuous `mulberry32` sequence, not
independent streams. `name_and_populate_settlements` (milestone 9)
previously created and discarded its own `civ_name_rng()` internally with
no way to continue its ending state. Added
`name_and_populate_settlements_with_rng` (threading an external
`&mut Mulberry32`) alongside the existing zero-arg function, which now
delegates to it -- purely additive, the original signature/behaviour and
its existing golden test are untouched.

**Road-proximity coordinate adaptation**: `HierarchicalNetworkResult`'s
edge paths live in `civ_hierarchical_network_topology`'s own DOWNSAMPLED
routing grid (`rw`x`rh`, scaled by `sc`), not full-grid coordinates like
the reference's own already-full-grid `ways`/`.pts`. `RoadProximityIndex`
converts each path cell back via `(cx+0.5)/sc` -- the identical mapping
`buildRoadsOp` itself uses to turn a routing-grid path back into world
coordinates -- and inserts every raw per-cell path point directly (no
2-cell segment interpolation, since milestone 12's raw path is already
denser than the reference's own coarser polyline sampling needs).

**A real threshold-consistency question investigated, not assumed**:
`_civSeedVillages`'s own `suitHi=SETTLE_SEED_THRESH` is unambiguous
(`0.42`, the reference's literal module constant) -- but milestones 7-9's
existing golden tests all seed their candidate lists at `0.65`
(`find_settlement_seeds(..., 0.65, ...)`), not `SETTLE_SEED_THRESH`.
Traced the reference's own default `_civIterativeAutoWorld` call
(`thresh:wantCounts?0.35:SETTLE_SEED_THRESH`) and confirmed a headless
harness with no DOM elements makes `wantCounts` always `null` (every
`document.getElementById` lookup returns `null` -&gt; falsy), so the real
default path uses `0.42`, not `0.65`. This is **not a bug in milestones
7-9** (`find_settlement_seeds`/`place_settlements`/
`name_and_populate_settlements` are pure functions, correctly verified
bit-exact for whichever threshold their own tests fed them) -- it is a
**pipeline-orchestration question**: whatever calls these functions to
build the REAL base-settlement candidate list (`cartalith-godot`'s
`compute_civilisation()`, built by the UI/UX pass) should pass `0.42`
(`SETTLE_SEED_THRESH`) to match `_civIterativeAutoWorld`'s real default
behaviour, not `0.65` (that value's real origin is a *different* call
site -- the standalone `settlement_seeds.json` export's own bare-default
fallback). **Flagged here for whoever next touches `cartalith-godot`'s
orchestration -- not fixed in this pass, out of `cartalith-civ`'s own
scope.**

**Golden verification**: fresh Node harness (blocks #1 2084-14552,
trimmed before the trailing `GW=state.resW;...generate()` auto-invoke
that would otherwise run at default resolution on load, + block #2
14563-26720). Two real gotchas this harness hit and fixed, neither
previously documented by a sibling fixture: (1) the permissive DOM-stub
Proxy needed explicit `Symbol.toPrimitive`/`valueOf`/`toString` handlers
-- an auto-vivified stub property failing numeric coercion
(`navigator.maxTouchPoints&gt;1`) crashed load entirely; (2)
`window.addEventListener` needed a real no-op function on the sandbox
object itself, since `window===sandbox` in this harness rather than
another stub layer.

Deliberately fully synthetic inputs (uniform `field=0.9`/`seaLevel=0.1`
so every cell is land, avoiding the real risk of hand-picking candidate
coordinates that might land underwater in an actual generated world) --
same standard `golden_parity_hierarchical_network.rs`'s own settlement
inputs already established: hand-constructed but verified against the
REAL reference function, not a reimplementation. Two well-separated
suitability hotspots (`0.5`, comfortably above `VILLAGE_SUIT_THRESH`
so `suitProb` clamps to `1.0` and the accept roll is deterministic
regardless of RNG position), no road edges (`suitProb` alone must carry
acceptance). **Passed bit-exact on the first attempt**, including
RNG-derived village names (`"Nashzafwell"`/`"Dagrkartor"`) and
nearest-capital faction inheritance. A second, targeted extraction
independently confirmed the road-proximity coordinate-conversion formula:
computed `_civVillageAcceptProb` at the reference's own real distance
matched a hand-calculated `exp(-0.7071.../4)` to 15 significant figures
(`0.8379668855787558`), the same geometry `RoadProximityIndex`'s
`(cx+0.5)/sc` conversion produces by construction.

**Toggle decision**: the reference gates this whole feature behind
`_civVillages` (default OFF). `civ_seed_villages` itself is a standalone,
nothing-calls-it-automatically function within `cartalith-civ` -- already
opt-in by construction. Whether `cartalith-godot`'s orchestration should
call it (and whether that should be user-facing, matching the reference's
own default-off gating) is that crate's own decision, out of this
milestone's scope -- flagged, not resolved here.

`cargo test -p cartalith-civ`: 60 lib tests + all golden fixtures green
(10 new unit tests covering `civ_village_accept_prob`'s formula
boundaries/monotonicity, `RoadProximityIndex`'s empty/populated cases,
`suppression_radius_cells`, spacing-rejection, and the village cap).
`cargo clippy -p cartalith-civ --all-targets`: clean, zero new warnings.
`cargo test --workspace`/`cargo build --workspace`: no regressions.

**Not implemented, milestones 13/14's own scope**: sea routes
(`_civMstRoutes`), corridor consolidation/Catmull-Rom smoothing/road
classification -- unaffected by this milestone, which only needed raw
topology.

## GPU layer integration milestone 4: `gauss_blur` + `compute_resistance` on GPU -- genuine three-way JS/CPU/GPU parity (2026-08-16)

`GPU_LAYER_INTEGRATION_SCOPE.md`'s milestone 4. Unlike milestones 1-3
(GPU-safe noise, domain warp, height formula -- all noise-driven, all
verified only GPU-vs-CPU-twin per `DECISIONS.md` §7c since the JS-matching
noise isn't GPU-portable at all), `gauss_blur` and `compute_resistance`
touch no noise. Traced `generate_terrain`'s real call chain first (milestone
3's own "out of scope, investigate for milestone 5" list): both are used
directly, `gauss_blur` twice (`base_field`, and inside `compute_flexure`).

**Headline result, investigated rather than assumed**: both kernels reach
genuine three-way JS/CPU/GPU parity -- verified directly against the real,
untouched `cartalith_terrain::gauss_blur`/`compute_resistance` (a new
`cartalith-terrain` dev-dependency in `cartalith-gpu`'s own `Cargo.toml`,
test-only, no runtime dependency), not a GPU-specific CPU twin.

- `gauss_blur`: the real concern going in was real -- CPU accumulates its
  sliding-window sum in `f64` (rounding to `f32` only on write); WGSL has
  no `f64` at all, so the GPU kernel does a direct per-cell window sum in
  `f32` throughout (a different, GPU-native evaluation order for the same
  box-filter definition, not a running-sum port -- that optimization isn't
  even expressible on this toolchain). Measured at 512x512 across three
  radius/wrap configurations (including a 48-cell radius, 97 summed
  values per output cell): worst observed divergence `7.15e-7`, essentially
  `f32` machine epsilon. `BLUR_TOLERANCE = 2e-6`.
- `compute_resistance`: trivial per-cell formula (`min(crustal*0.6 +
  age*0.4, 1.0)`), no accumulation. Measured at 512x512: worst observed
  divergence `5.96e-8`. `RESISTANCE_TOLERANCE = 5e-7` (a ~8x margin,
  matching this crate's convention for a stable FMA-contraction-scale
  residual rather than the thinner margin a first pass gave it).

**`compute_flexure`, checked not assumed**: a thin wrapper (mask
`stress_field` by `boundary_mask`, `gauss_blur` at 3x radius, max-abs
normalize) -- not ported this pass. The trivial mask-select and the
normalize both follow the same "cheap CPU post/pre-process around the real
GPU-accelerated workload" pattern milestone 2's heterogeneity normalize
already established; `gpu_gauss_blur` itself is the piece that needed real
verification, and now exists for a future pass to wire `compute_flexure`
around.

**Real, honest timing** (128/512/1024/2048, single-threaded CPU, real
wgpu dispatch+readback):

| Kernel | 128² | 512² | 1024² | 2048² |
|---|---|---|---|---|
| `gauss_blur` | 0.03x (GPU loses) | 3.33x | 16.89x | 20.49x |
| `compute_resistance` | 0.00x | 0.12x | 0.38x | 0.38x |

`compute_resistance` **loses to CPU at every tested size, including
2048²** -- reported plainly, not hidden. Its formula is so trivial (one
multiply-add-min, one array lookup) that GPU dispatch/readback overhead
never amortizes against it, exactly the case `HARDWARE_ACCELERATION.md`
§6 already warns about ("small operations should remain on the CPU when
demonstrably faster"). `crustal_per_plate` (a `plates[k].base.max(0.0)`
array, `num_plates` long) is precomputed on CPU once per call -- cheap,
not the workload being measured.

**Where the code goes**: `cartalith-gpu`, two new WGSL files
(`gpu_gauss_blur.wgsl` with two entry points `box_h_main`/`box_v_main`
sharing one bind-group layout, `gpu_resistance.wgsl`), a new
`GpuBlurContext` (two pipelines, one device/queue -- `gauss_blur`'s
three-pass horizontal-then-vertical structure needs both kernels able to
read what the other just wrote, which the existing single-pipeline
`GpuContext` can't express; a dedicated init function duplicating
`init_gpu_with`'s adapter/device/queue setup rather than generalizing that
shared helper for its one caller needing two entry points). `cartalith-
terrain`'s `gauss_blur`/`compute_resistance`/`compute_flexure` untouched,
same rule every GPU milestone has followed for the CPU reference pipeline.

7 new tests (`gpu_gauss_blur_matches_real_cpu_gauss_blur`,
`_matches_gpu_shaped_cpu_twin`, `_r_below_one_is_unmodified_copy`,
`_deterministic_across_runs`; `gpu_compute_resistance_matches_real_cpu_
compute_resistance`, `_deterministic_across_runs`; `measured_gpu_blur_
and_resistance_timing`). `cargo test -p cartalith-gpu`: 30/30 pass.
`cargo test --workspace`/`cargo build --workspace`/`cargo clippy -p
cartalith-terrain -p cartalith-gpu --all-targets`: clean, no regressions.

**Milestone 5, not scoped, deliberately left open**: `build_age_field`
(a real two-pass chamfer distance transform, sequential sweep dependency
-- confirmed a poor GPU fit, not assumed, milestone 3's own note), plate
assignment/`build_plates` (JFA-based, flagged as a plausible good fit
several milestones ago, still not investigated), `compute_stress`, and
orogeny's graph-tracing (`trace_boundaries`/`tag_boundary_types`/
`build_orogeny_field`) all remain genuinely uninvestigated -- this
milestone didn't get to any of them, say so plainly rather than implying
otherwise.

## GPU layer integration milestone 5 -- plate assignment (JFA), GPU beats brute-force exactly (2026-08-16)

`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 5. Read `assign_plates`
(`cartalith-terrain/src/lib.rs:400`) and `compute_stress` (line 657) in
full before scoping, confirming milestone 3's hypothesis on the former and
finding the latter genuinely harder than a same-shape sibling.

**`assign_plates` confirmed a textbook Jump Flooding Algorithm** -- but a
specific variant: **in-place mutation**, not double-buffered. Its
`while step_u >= 1` loop scans row-major and updates `nearest[i]`/
`best_d2[i]` directly, so a cell processed later in the same pass can see
another cell's update from *earlier in that same pass*, not just the
previous pass's frozen state. That's a real, order-dependent algorithm
variant -- not an implementation detail like `gauss_blur`'s running-sum
(milestone 4), which computed the *same* mathematical result a different
way. In-place JFA and double-buffered JFA are both valid completions of
the jump-flood algorithm, but they can converge to different specific
answers in ambiguous/boundary cases. This kernel implements the
**standard double-buffered variant** (`gpu_jfa_plates.wgsl`) -- the
textbook, race-free GPU formulation JFA is actually known for -- and does
**not** attempt to reproduce the CPU function's in-place answer
cell-for-cell. Verified against brute-force exact-nearest-plate ground
truth instead of the CPU function directly, per the scope doc's own
instruction to investigate which framing fits rather than assume either.

**`compute_stress` confirmed genuinely harder, deferred to its own future
milestone**: its main loop is a *scatter*, not a per-cell-independent
kernel -- each boundary cell writes accumulated stress to both itself
and its neighbour in the same iteration (`raw[i]` and `raw[j]`), a real
cross-thread write hazard on GPU (multiple invocations could target the
same output cell simultaneously). WGSL's core spec doesn't cover atomic
`f32` add on this toolchain. A real port needs reformulating as a
*gather* (each output cell reads whether its neighbours would have
pushed a contribution onto it) -- which changes summation order and
needs its own floating-point re-verification, not just a translation.
Not bundled into this milestone.

**Built**: `gpu_jfa_plates.wgsl` (`main`, double-buffered, dispatched
`log2(max(w,h))`-ish times per call with alternating in/out bind groups,
all within one encoder -- same "many passes, one submit" shape
`dispatch_gpu_gauss_blur` established, generalized from a fixed 3x2 to
`steps.len()` distinct-parameter passes). `init_gpu_jfa_plates`/
`dispatch_gpu_assign_plates` in `cartalith-gpu` (single-pipeline
`GpuContext`, not a `GpuBlurContext` -- JFA has one shader entry point,
unlike blur's `box_h`/`box_v` pair). Seeding (plate home cells at
distance 0) and the fallback fill for any cell JFA never reaches both run
on CPU, mirroring `assign_plates`'s own structure -- neither is worth its
own GPU kernel (seeding is O(plate count); the fallback was empirically
zero-cost in every test run here). `brute_force_nearest_plate` (public):
the ground-truth reference neither JFA variant is trying to match exactly,
used to characterize both variants' real approximation error.
`cartalith_terrain::assign_plates`/`compute_stress` untouched.

**Verification -- three-way comparison, not GPU-vs-CPU-twin**: GPU JFA vs.
brute-force truth, CPU (in-place) JFA vs. brute-force truth, and GPU vs.
CPU directly, all measured together across three configurations (512x512
at 14 and 40 plates, 1024x768 at 22 plates). Result: **GPU JFA matched
brute-force ground truth exactly, 0 mismatches, in every configuration
tested.** CPU's in-place JFA had a consistent, tiny real approximation
error (1-2 cells out of 262,144-786,432, i.e. ~0.0003-0.0004%) against the
same ground truth -- expected and correct JFA behaviour (a well-known
property of the algorithm at boundary/equidistant cells), not a bug in
either variant. GPU-vs-CPU direct mismatches exactly track CPU's own
deviation from truth, confirming GPU isn't introducing new error, it's
simply more exact than the CPU in-place variant on this test suite.
Determinism confirmed (same input twice, byte-identical GPU output).

**Real timing** (128/512/1024/2048, 24 plates): GPU wins even at 128x128
(1.63x) -- unlike every single-pass kernel in milestones 1-4, which all
*lost* at that size to dispatch overhead. JFA's `log2(size)`-pass
structure means real compute work happens even on a small grid, so there's
more for the GPU to amortize its fixed dispatch overhead against. Scaling
up: 11.50x at 512x512, 18.22x at 1024x1024, 15.65x at 2048x2048 (a real,
reported-not-smoothed-over dip from 1024x1024, matching the same
unexplained-but-honestly-reported pattern milestone 3's height kernel
found at the same transition).

**Verification**: `cargo test -p cartalith-gpu`: 33/33 pass (3 new).
`cargo test --workspace`/`cargo build --workspace`: clean, no
regressions. `cargo clippy -p cartalith-terrain -p cartalith-gpu
--all-targets`: clean (only the same pre-existing "never used outside
tests" warnings every prior GPU milestone already has).

**Milestone 6, not investigated this pass**: `compute_stress` (deferred
above, needs the gather reformulation), `flex`'s full body beyond
milestone 4's blur, orogeny's graph-tracing
(`trace_boundaries`/`tag_boundary_types`/`build_orogeny_field`, still
genuinely unread), `build_age_field` (confirmed poor fit, milestone 4).
Orogeny is the natural next candidate to actually read, per this
milestone's own scope note -- not investigated this pass, flagged
honestly rather than guessed at.

## Phase 2 milestone 14 -- corridor consolidation + path smoothing (2026-08-16)

Ported the consolidation/classify/smooth/name tail of
`_civHierarchicalNetwork` (reference HTML lines ~21670-21739) plus its
three helpers `rdpSimplify`/`catmullRomSample` (lines 8701/8790) and
`_civSmoothPath`/`_civTerrainValidTest`/`_civNearestValidPt` (lines
21892/21843/21872) to `cartalith-civ`, as
`civ_consolidate_and_smooth_ways`. This is `PHASE2_SCOPE.md` milestone
14, deferred from milestone 12 on purpose -- milestone 12 shipped the raw
MST-family topology (what road-proximity queries need), this ships the
presentation layer on top of it (what actually gets drawn).

**What the step actually does, confirmed against the reference rather
than assumed from the original one-sentence brief**: milestone 12's
`civ_hierarchical_network_topology` produces raw edges as lists of
downsampled-routing-grid cell indices, several of which legitimately
overlap (two settlements' shortest paths sharing a trunk corridor). This
milestone (a) sorts all edges by peak per-cell usage count descending,
(b) walks them busiest-first, each edge claiming only the sub-runs of its
own path not already claimed by a busier edge (plus one already-claimed
connector cell at each cut, so strokes still join at junctions) -- so a
shared trunk renders once, under the busiest edge's classification, not
once per edge that uses it, (c) classifies each edge by its own peak
usage (`highway`>=8, `regional`>=5, `road`>=3, else `track`) and
auto-names it from its endpoint settlements
(`"PlaceA → PlaceB"`/one name/empty), (d) converts each claimed run's
cell indices back to full-grid coordinates and Catmull-Rom-smooths it
(RDP-simplify at `eps=1.5` then chord-length-parameterized spline
sampling at `step=3`, both matching the reference's own generic
polyline-smoothing helpers bit-for-bit in algorithm shape), with a
terrain-validity repair pass (any smoothed+rounded point landing in water
gets pulled to the nearest land cell via bounded expanding-box search)
and float-precision endpoint restoration (a run's own first/last point is
always the exact un-rounded coordinate it started from, not smoothing
output), and (e) an edge with no unclaimed cells left at all still emits
a hidden 2-point straight-line way (so the network graph stays complete
for anything querying by `aIdx`/`bIdx`, even though nothing draws it).
A final endpoint-snap pass pulls each visible way's own start/end point
onto its edge's real settlement position if within a bounded threshold
(`(max(6, 4/sc) capped at (GW/30)*0.45))^2`), since consolidation can
leave a visible run starting a routing-cell or two short of the pin.

`_civTerrainValidTest` was ported narrowed, not in general form: the
reference's real function takes a `kind` (`'land'`/`'ocean'`/etc.) plus
optional sea-lane-allowance `opts`, but this network's only real call
site is `_civTerrainValidTest('land')` with no `opts` -- so
`civ_is_valid_land` implements exactly that one collapsed case (valid iff
not water, against milestone 2's real water-body classification), not
the general dispatcher. Flagged explicitly for milestone 13: the ocean
mode (`kind==='ocean'`, valid iff water-body class 1 specifically,
excluding lakes) is a different, not-yet-ported case that milestone 13's
sea routes will need.

**Golden verification**: fresh Node harness, blocks #1 (2084-14556) +
#2 (14563-26720) concatenated into one `vm` context. **A small but real
correction to a previously-recorded line-range convention**: an earlier
milestone's own CHANGELOG entry states these ranges as
"2083-14556"/"14562-26720" -- literally slicing at those numbers includes
the `<script>` tag itself as the first extracted line (confirmed by
direct inspection: line 2083 is the tag, real code starts 2084; line
14562 is block 2's tag, real code starts 14563) and throws
`SyntaxError: Unexpected token '<'`. Both prior milestones' own
extractions evidently used the code-only start already (their harnesses
otherwise couldn't have run) -- this was a transcription slip in that one
CHANGELOG sentence, not a real bug in prior extractions; recorded here so
the next fork copying line ranges from a CHANGELOG entry uses the correct
ones. `state.tect.seed` (not the dead `state.seed`), `allocate()` with
zero arguments, `state.climate.wIters=12` (this crate's established
speed-override convention, matching every sibling fixture, not the real
70). `_civHierarchicalNetwork(places, {})` was called directly rather
than instrumented -- unlike milestone 12, this function already *returns*
the post-consolidation `ways` array natively.

Both test cases reuse already-verified upstream fixtures rather than
re-deriving anything: settlement `(x,y,faction,name,pop)` tuples from
`golden_parity_settlement_naming.rs`'s own case0/case1, topology
confirmed against `golden_parity_hierarchical_network.rs`'s own edge
counts (case0: 1 edge; case1: K5, 10 edges) by calling
`civ_hierarchical_network_topology` fresh inside the new test (cheap,
deterministic, already proven correct -- not hand-transcribed). `field[0]`
cross-checked against both cases' already-trusted Rust-side values before
trusting the extraction, per this crate's own established discipline:
matched within ~9e-6 (`0.8640562...` vs `0.8640472...` for case0,
similarly for case1) -- comfortably inside this crate's `1e-4`
convention, and consistent with ordinary JS-vs-Rust cross-language float
noise (`PARITY_TESTING.md`: exact bit-identity across languages isn't the
target), not a harness setup bug.

New test file `tests/golden_parity_road_consolidation.rs`, two cases:
- **Case 0** (region, 1 edge, `gw=14 gh=11 seed=24601`): a genuine
  short-segment Catmull-Rom oversampling quirk, not a synthetic corner
  case -- the 2-cell path `[35,34]` produces a 3-point smoothed output
  `[(7,2),(7,2),(6,2)]` where the interpolated midpoint (6.5,2) rounds
  (via this project's `Math.round`-equivalent, which rounds .5 up) back
  onto the run's own start point, which then also gets endpoint-precision-
  restored to the identical exact coordinate -- a real duplicate point in
  legitimate output, traced and confirmed by hand, not treated as a bug
  to paper over.
- **Case 1** (world-wrap, K5, `gw=16 gh=12 seed=314159`): 10 ways, a mix
  of 3 visible smoothed polylines (2 highway, exercising real corridor
  sharing -- e.g. the Orenelywash-Ghalbahrghaltazdune and
  Ghalbahrghaltazdune-Hurngarngarnhaskcairn edges both routing through
  the same claimed cells the busiest edge already smoothed) and 7 hidden
  straight-line ways (edges whose entire path was already claimed by a
  busier edge), 5 highway + 2 regional by classification.

Point coordinates checked at this crate's established `1e-4` tolerance;
`km`/name/type/`aIdx`/`bIdx`/`hidden`/point-count checked exactly.
Both cases pass bit-for-bit against the real extraction on the first
attempt.

**Verification**: `cargo test -p cartalith-civ --all-targets`: all pass
(2 new, no regressions in the other 8 golden-parity test files). `cargo
clippy -p cartalith-civ --all-targets`: clean except the same 27
pre-existing "float has excessive precision" warnings milestone 12's own
test file already had (none in the new code). `cargo test --workspace` /
`cargo build --workspace`: clean, no regressions.

**Milestone 13, investigated for scope, not started**: `_civMstRoutes`
(reference line 21240) shares `_civSmoothPath` and the overall
Dijkstra-then-MST shape with milestone 12/this milestone, but is a real,
separately-scoped algorithm -- not a same-shape sibling to reskin. Its
cost grid marks land `Infinity` (genuinely impassable for sea routing,
not merely expensive -- a v1.xx fix note in the reference explains a
finite-but-expensive land cost let Dijkstra cut across jagged downsampled
coastline pixels, which Catmull-Rom smoothing then exaggerated into
visible nonsensical loops), pathing is costed by real current/wind fields
via `_civSeaTimeEdgeCost` (not yet read in any detail), and a v0.73
sea-lane augmentation pass adds each port's single nearest sea-reachable
port as a direct lane beyond the bare MST tree (capped at 1.15x the MST's
own longest edge), so short coastal hops don't detour through the tree's
spine. The one piece of milestone 13 this pass concretely unblocks:
`_civSmoothPath` itself is real and ported, reusable as-is by whatever
ports `_civMstRoutes` -- only the validity-predicate needs to change from
this milestone's land-only `civ_is_valid_land` to an ocean-only
equivalent (`kind==='ocean'`: valid iff water-body class 1 specifically,
excluding lakes -- a straightforward variant, not a new algorithm).
Economy, culture, and territory sub-partitioning
(`_civGenerateProvinces`) remain untouched, out of scope for this pass as
directed.

## UI/UX catch-up: territory + villages (2026-08-16)

Second UI/UX-catch-up pass this session (the first wired settlements/
factions/roads into `map_overlay.gd`; the ongoing practice per the
owner's own words: "keep an agent in parallel for the gui"). Two Phase 2
milestones had landed with real output and zero visual representation:
milestone 10 (territory assignment, `assign_territory`) — its own
implementing pass explicitly flagged it was never rendered or even
looked at — and milestone 15 (village seeding, `civ_seed_villages`),
which wasn't called from `cartalith-godot` at all yet.

**Part 1 — wired both into the real pipeline** (`cartalith-godot/src/
lib.rs`). `compute_civilisation()` gained a `villages_enabled: bool`
parameter (threaded from a new `WorldGen.villages` field, default
`false` matching the reference's real `_civVillages` default). When
enabled: one `Mulberry32` instance is created via `civ_name_rng()` and
threaded through *both* `name_and_populate_settlements_with_rng` and
`civ_seed_villages` — a single continuous stream, not two independent
RNGs, per `civ_seed_villages`'s own doc comment requirement (a bug in an
early draft of this edit: an accidental duplicate `civ_name_rng()` call
created a second, desynced RNG instance; caught and fixed during
self-review before verification, never shipped). `civ_routing_grid`
(the `routing_rw`/`routing_sc` grid `civ_seed_villages` needs) is
`private` in `cartalith-civ` — rather than widen that crate's public API
while a concurrent fork was actively editing the same file (milestone 14
landed mid-session, see below), its trivial formula (`rw =
gw.min(384)`, `sc = rw/gw`) is replicated locally in `cartalith-godot`
instead. Seeded villages come back as `VillageSettlement { x, y, name,
faction }` and are merged into the same `Vec<NamedSettlement>` the map
overlay already draws, tagged `SettlementKind::Hamlet`, `pop: 0`
(villages don't carry a population figure in the reference either).
Territory is unconditional (not gated) — `assign_territory` is called
every generation regardless of the `villages` toggle, reusing the
already-computed `cost` field from the road network (one Dijkstra per
capital, cheap), and there's no reference default to match since the
reference has no algorithmic territory generation at all
(`DECISIONS.md` §7b).

**Part 2 — rendered both.** Territory: per-cell `Vec<i32>` grid data is
too large for `map_overlay.gd`'s per-marker `_draw()` calls (the pattern
settlements/roads use) — instead, a new `build_territory_texture()`
(mirrors the existing `build_color_texture()` pattern) turns it into an
RGBA8 `ImageTexture`: the same 6-hue Okabe-Ito palette settlement
markers already use, at alpha 82/255 (~0.32) so terrain/biome colour
still reads through, transparent for unowned cells (water or
unreachable from any capital). A new `TerritoryView` `TextureRect` sits
in `main.tscn` between `MapView` (terrain) and `MapOverlay` (settlement/
road vector draw) for correct z-order, wired to a new default-OFF "Show
territory (faction colour fill)" checkbox in the existing "Map Layers"
card. Villages needed no new rendering code at all — merging them into
the existing settlement list means `map_overlay.gd`'s established
tier-based marker/hover pattern already draws and labels them as
`Hamlet`-tier markers, gated by a new default-OFF "Village seeding
(Phase 2, additive hamlets)" checkbox under "Advanced Features".

**Verified — real windowed app, not just headless.** `cargo build -p
cartalith-godot`, `cargo clippy -p cartalith-godot --all-targets`, and
`cargo build --workspace` all clean (only pre-existing warnings from
concurrent GPU-integration work elsewhere in the workspace, none from
these changes). Launched the actual windowed MVP UI on this session's
real Windows desktop (PID/hwnd captured, `PrintWindow`-based screenshot
technique per this session's established method), scrolled the settings
panel with synthetic mouse-wheel events to reveal the two new
checkboxes (a real PowerShell gotcha hit and fixed along the way: the
`mouse_event` P/Invoke signature's `dwData` parameter must be declared
signed `int`, not `uint` — a negative wheel-delta value throws a type-
conversion error against an unsigned parameter type), checked both,
generated a real 512×512 world (seed 12345, 800 km, Classic), and
screenshotted the result. **Confirmed by actually looking at it**:
territory renders as four plausible, contiguous colour-filled regions
(orange/blue/yellow/teal-green) that follow the landmasses and stop
cleanly at coastlines and open water — not noise, not a uniform tint —
and village seeding visibly densifies the settlement layer: dense
clusters of small hamlet-tier dots surround each capital well beyond
what base settlement placement alone produces, with the status label
confirming 240 total settlements for this run.

**Scope discipline held**: sea routes (13), road consolidation/
smoothing rendering (14 — its data-layer function `
civ_consolidate_and_smooth_ways` landed mid-session from a concurrent
fork, confirmed via `git log` to touch only `cartalith-civ` and its own
docs, not `cartalith-godot` or `godot-project/` — the map correctly
still renders milestone 12's raw topology, since wiring 14 in was never
part of either fork's scope), the full interactive civ editor, and
territory sub-partitioning into provinces all stayed untouched, exactly
as directed.

**Files touched**: `cartalith-native/crates/cartalith-godot/src/lib.rs`
(`CivData.territory` field, `compute_civilisation()` signature +
villages/territory wiring, `WorldGen.villages` field +
`set_villages_enabled()`, `build_territory_texture()`),
`cartalith-native/godot-project/main.tscn` (`TerritoryLayerCheck`,
`VillagesCheck`, `TerritoryView`), `cartalith-native/godot-project/
main.gd` (onready refs, toggle wiring, `set_villages_enabled()` call,
`build_territory_texture()` call, clearing `territory_view.texture` on
save-load). `cartalith-civ` untouched — deliberately, both to stay
disjoint from the concurrent milestone-14 fork and because no new
public API was needed there.

## UI/UX catch-up: wire milestone 14's smoothed roads into the map (2026-08-16)

Closes the gap the previous UI/UX pass explicitly flagged: milestone 14
(`civ_consolidate_and_smooth_ways`) landed with real Catmull-Rom-smoothed,
classified, named road polylines, but `compute_civilisation()` still
built its `roads` field from `build_road_network` — not even milestone
12's own raw topology, let alone milestone 14's smoothed output. That
function is `buildRoadNetwork`, the reference's *manual*-placement-tool
algorithm, used as a stand-in for the real auto-populate road system
before that system existed at all (an even earlier gap than the
milestone-14-only framing this task started from).

**Fixed the real chain**: `civ_hierarchical_network_topology` (milestone
12) now builds the actual auto-populate topology from `placements`, then
— after naming/village-seeding, since `civ_consolidate_and_smooth_ways`
needs named settlements for its `pa.name`/`pb.name` endpoint naming —
`civ_consolidate_and_smooth_ways` (milestone 14) turns it into the
smoothed `Way` list the map now renders. `civ_seed_villages` also now
reads `topology.edges` (the real network) instead of the old manual-tool
stand-in's edges, so village road-proximity is against the right network
too, not just the map's rendering.

**`CivData.roads: Vec<RoadEdge>` → `CivData.ways: Vec<Way>`.** `get_roads()`
now returns `Array<VarDictionary>` (`points`, `brks`, `way_type`, `name`)
instead of raw `Array<PackedVector2Array>` cell-index paths — `points`
are already continuous, smoothed, full-resolution coordinates, not grid
cell indices, so `map_overlay.gd` needed a distinct `_point_to_screen`
(no `+0.5` cell-centering) alongside the existing `_cell_to_screen`
(settlement markers, which *are* still cell-index-based) — using the
wrong one would have shifted every road by half a cell. Hidden ways (an
edge fully consolidated away into a busier neighbour — real, expected
behaviour, not a bug) are filtered out entirely rather than drawn as
degenerate 2-point stubs. `brks` (real internal gaps where two disjoint
consolidated runs share one `Way`) are honoured by splitting into
separate `draw_polyline` calls per run — drawing straight through a break
would render a phantom line across a real discontinuity. Road width now
varies by `way_type` (`highway` 2.6px down to `track` 1.1px), the same
"tier implies visual weight" principle already applied to settlement
markers.

**Screenshot-verified** (real windowed app, 512×512, seed 12345, Classic,
40 settlements): roads now render as visibly smooth, continuous curves
following terrain between settlements, a clear, dramatic change from the
straight/jagged MST-approximation look the previous pipeline produced —
confirmed by eye, not just "the code compiles."

**Real gotcha caught before it shipped**: the first edit attempt passed
an empty settlement slice to `civ_consolidate_and_smooth_ways` (available
before naming ran) — silently wrong, not a crash: every edge's `a_idx`/
`b_idx` bounds-check (`a >= n || b >= n`) would have failed against an
empty list, producing **zero ways** rather than an error. Caught by
reading the function's own indexing logic before running it, not by
observing a blank map after the fact.

**Verification**: `cargo build -p cartalith-godot`, `cargo test
--workspace`, `cargo clippy --workspace --all-targets` all clean.
`godot4 --headless --quit main.tscn` clean, extension loads.

**Files touched**: `cartalith-native/crates/cartalith-godot/src/lib.rs`
(`CivData.ways` replacing `.roads`, `compute_civilisation()`'s road-chain
reordering, `get_roads()`'s new `Dictionary`-per-way shape),
`cartalith-native/godot-project/map_overlay.gd` (`_point_to_screen`,
`_draw_way_segment`, `ROAD_WIDTH_BY_TYPE`, break-aware road drawing).
`cartalith-civ`/`cartalith-terrain`/`cartalith-gpu` untouched — stayed
disjoint from concurrently-running GPU-integration and Phase 2 forks.

## GPU layer integration milestone 6 -- first real partial-GPU pipeline integration (2026-08-16)

`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6. Every prior GPU milestone
(1-5) built and verified a **standalone** kernel — none was ever called
from `generate_terrain` (`cartalith-engine/src/lib.rs:418`) itself.
Generating a map has been CPU-only this whole time not because GPU
wasn't working, but because nothing wired it in. This milestone is that
wiring.

**A real gap found before it could be closed**: milestones 2/4/5's own
`dispatch_gpu_warp`/`dispatch_gpu_heterogeneity`/`dispatch_gpu_gauss_
blur`/`dispatch_gpu_assign_plates` were all private to `cartalith-gpu` —
`init_gpu_*()` was public but the actual dispatch functions weren't, so
no other crate could reach them regardless of any flag. Added four new
public wrappers (`warp_grid_gpu`/`heterogeneity_grid_gpu`/`gauss_blur_
grid_gpu`/`assign_plates_grid_gpu`), each `init_gpu_X().ok()?` then
dispatch, returning `Option` — `None` means "GPU unavailable right now,"
never a panic (`HARDWARE_ACCELERATION.md` §27).

**Built**: `WorldParams.use_gpu: bool` (default `false`) and
`WorldState.gpu_stages_used: Vec<String>` (which stages actually ran on
GPU this call — a caller isn't left guessing which path executed).
`generate_terrain` gained a `p.use_gpu` branch running domain warp,
crustal heterogeneity, plate assignment, and flexure/base-field blur on
GPU, with per-stage fallback to the exact CPU function on any `None`
(including, for plate assignment, any `-1`/unassigned cell in the GPU
result — treated as a failed dispatch, not cast-and-corrupt). Domain
warp and heterogeneity gate on `p.use_gpu && !world` specifically —
milestone 2 never added world-wrap support to those two kernels, so
`world=true` always takes CPU regardless of the flag.
`compute_flexure`'s own three-step body (mask, blur, normalize) is
inlined into the GPU branch so only its blur step routes through GPU;
the `use_gpu=false` branch keeps calling the real, untouched
`compute_flexure` directly. `compute_stress`, `build_age_field`, and
orogeny stayed CPU-only, as scoped (confirmed poor GPU fits in
milestones 4/5/6's own orogeny investigation). `cartalith-terrain`'s
reference functions are byte-untouched.

**CPU path unchanged — the headline requirement**: `cargo test
--workspace` 100% green, every existing golden-parity test for
`generate_terrain` and every downstream field (climate, erosion,
hydrology, all of Phase 2) passes unmodified. `use_gpu: false` is
`WorldParams::defaults()`'s value, so no pre-existing call site needed
touching.

**GPU path verification**: two new `cartalith-engine` tests —
determinism (same seed, `use_gpu=true`, run twice, byte-identical
`field` and `gpu_stages_used`) and statistical sanity (no NaN/Inf,
`field` still in `[0,1]`, not degenerate-flat, every `gpu_stages_used`
name is one of the four this milestone actually wired) plus a shape-
parity test confirming `use_gpu=true`/`false` produce identically-
shaped `WorldState`s even though values differ (`DECISIONS.md` §7c: GPU
noise is a structurally different hash, not a tolerance-close port, so
the two are genuinely different valid worlds for the same seed, not
compared value-for-value). Visual render comparison not attempted this
pass — no windowed Godot session available in this environment.

**Real timing — end-to-end, not isolated kernel dispatch**: each of the
four GPU wrappers creates its own fresh `GpuContext` per call (documented
tradeoff, fine for one-shot batch generation), so `generate_terrain
(use_gpu=true)` pays roughly four device-creation overheads every call,
not once. Measured (release build): 128×128 GPU ~16× **slower** (1.44s
vs 88ms), 512×512 ~2.4× slower (1.46s vs 594ms), 1024×1024 still slower
but closing (2.32s vs 1.82s), 2048×2048 GPU finally wins, modestly
(6.03s vs 7.20s, 1.19×). Reported honestly including the loss: at every
size this pilot ships at by default, GPU is slower, dominated by ~1.3-
1.4s of near-flat fixed per-call context-creation overhead that the
individual kernels' own much larger standalone wins (up to 80× for warp,
~18-20× for blur/JFA, milestones 2/4/5) can't outrun until the grid is
large enough. Context reuse/caching across the four stages is the
single highest-leverage next optimization — not attempted here, flagged
in `GPU_LAYER_INTEGRATION_SCOPE.md` rather than glossed over.

**Verification**: `cargo build --workspace`, `cargo test --workspace`
(0 regressions), `cargo clippy --workspace --all-targets` clean — one
real new warning in this milestone's own inlined `compute_flexure`
masking loop (`needless_range_loop`), fixed with `zip` over an index
range; everything else pre-existing.

**Milestone 7, investigated not built**: read `simulate_weather`
(`cartalith-climate/src/lib.rs:963`) in full, resolving this scope
doc's own flagged uncertainty about its wind/rain loop's cross-cell
coupling. Finding: genuinely GPU-feasible — each iteration's three
per-cell passes (evaporation, semi-Lagrangian advection via bilinear
*gather* from the previous iteration's frozen field, precipitation
deposit) are all gather-shaped like JFA/blur, not `compute_stress`'s
scatter hazard. `build_wind` itself (called once, not per-iteration) is
also per-cell independent and already calls `gauss_blur`, unused on GPU
here. Not bundled into this pass — real future scoping work (kernel
count, per-iteration dispatch overhead repeating this milestone's own
context-creation lesson) still needed, not assumed complete by the
investigation alone.

**Files touched**: `cartalith-native/crates/cartalith-gpu/src/lib.rs`
(four new public wrappers), `cartalith-native/crates/cartalith-engine/
Cargo.toml` (new `cartalith-gpu` dependency), `cartalith-native/crates/
cartalith-engine/src/lib.rs` (`WorldParams.use_gpu`, `WorldState.
gpu_stages_used`, `generate_terrain`'s new branch, four new tests).
`cartalith-terrain` untouched.

## Phase 2 milestone 13 -- sea routes (2026-08-16)

Ported `_civMstRoutes(ports, true)` (reference HTML line 21240, `isSea`
branch only) to `cartalith-civ` as `civ_sea_routes`. This is
`PHASE2_SCOPE.md` milestone 13, the port-to-port sea-lane MST that runs
alongside milestone 12's land network in the real auto-populate flow
(`_civIterativeAutoWorld`, reference line ~25680: `if(ports.length>=2)
ways.push(..._civMstRoutes(ports,true))`, pushed unconditionally, NOT
gated behind `_civAutoRoutes`'s land-vs-sea cost comparison -- that
comparison belongs to a separate manual "Auto routes" tool, confirmed
out of scope by reading `_civAutoRoutes` itself).

**Scope confirmed by reading the reference directly, not assumed from
the milestone brief**: the `isSea=false` land branch of `_civMstRoutes`
has no confirmed real caller (`_civHierarchicalNetwork`/milestone 12 is
what the actual land network uses) and is not ported. `_civSeaTimeEdgeCost`
(v1.98 current/wind-costed sea-lane pricing) is also not ported -- its
real inputs, the ocean-current and wind u/v vector fields, are computed
internally by `apply_ocean_currents`/`deflect_flow` but never retained
on `WorldState` past that internal use (only the resulting SST/rainfall
corrections survive). The reference's own code degrades gracefully when
these fields are unavailable (`if(!oceanF&&!windF) return null` ->
caller falls back to `roadDijkstra`'s default uniform arithmetic-cost
step), so this port takes that same documented fallback rather than
adding new `WorldState` plumbing outside this milestone's scope -- a
real, flagged follow-up (wind/current-aware sea-lane costing), not a
silently-dropped feature.

**What it does**: builds a downsampled cost grid where navigable open
ocean (`water_bodies==1`) costs 1 and everything else -- land, lakes,
inland seas -- is genuinely `Infinity` (not merely expensive; the
reference's own fix-history comment explains why: a finite land cost let
Dijkstra cut across jagged downsampled coastline pixels when that was
cheaper than the long way around, and Catmull-Rom smoothing then
exaggerated those land-cutting zigzags into visible nonsensical loops).
Snaps each port to the nearest navigable-ocean cell (radius 10 --
deliberately wider than milestone 12/14's own radius-6 `civ_snap_finite`
calls on a different cost grid, matching the reference exactly, not
"fixed" into false consistency), runs Dijkstra from every port, builds a
Prim's MST over the pairwise distances, then applies the v0.73 nearest-
port sea-lane augmentation (each port's single nearest sea-reachable
port becomes a direct lane too, capped at 1.15x the MST's own longest
hop, so two neighbouring coastal towns linked only via a long detour
through the tree's spine also get the short direct economic hop).
Reconstructs each edge's path from the Dijkstra `prev` tree and smooths
it with the same Catmull-Rom pipeline milestone 14 already ported
(`civ_smooth_path`), in ocean validity mode.

**Four existing helpers generalized to land/ocean modes** rather than
duplicated: `civ_snap_finite` (added a `max_r` parameter, was hardcoded
to milestone 14's `1..=6`), `civ_is_valid_land` renamed
`civ_is_valid_terrain` (added an `is_sea` branch alongside the existing
land check, matching `_civTerrainValidTest('land'|'ocean')`),
`civ_nearest_valid_pt` and `civ_smooth_path` (both threaded the same
`is_sea` flag through to their internal validity checks). All four
existing call sites (milestone 12/14's land-only uses) updated to pass
their previous fixed values explicitly -- a surgical parameter addition,
not a closure/trait abstraction, per this project's smallest-diff
discipline.

**Golden verification**: fresh Node `vm` harness (reference HTML blocks
2084-14556 + 14563-26720), reusing `golden_parity_road_consolidation.rs`'s
own case0/case1 fixtures (already-verified coastal settlements at two
grids with genuine mixed land/ocean/lake geography: case0 79 land / 75
ocean / 0 lake of 154 cells at gw14×gh11 world=false; case1 127 land / 13
ocean / 52 lake of 192 cells at gw16×gh12 world=true). A real harness bug
caught before trusting extraction: `generate()` is `async`, and a bare
unawaited call left `field` at its default-zero fill and
`currentWaterBodies()` reporting 100% ocean -- fixed by awaiting it
properly and cross-checking `field[0]` plus land/ocean/lake cell counts
against already-trusted fixtures before extracting `_civMstRoutes`
output. Both cases (2 routes for case0's 3 ports, 4 routes for case1's 5
ports) matched the Rust port's output exactly on the first run --
`cargo test -p cartalith-civ --test golden_parity_sea_routes` passes
both. Two of case1's four routes carry `km:0` despite having real points
-- confirmed a genuine reference behavior by reading `_civSmoothPath`
directly (it accumulates `km` over the *rounded* sample points before
its own final step restores full-precision endpoints, so a short
diagonal hop whose only interior sample rounds to coincide with the
pre-restore rounded start point contributes zero distance), not a
harness bug.

**Verification**: `cargo build --workspace`, `cargo test --workspace`
(0 regressions), `cargo clippy -p cartalith-civ --all-targets` clean for
the new code (all reported warnings pre-existing, in other test files).

**Files touched**: `cartalith-native/crates/cartalith-civ/src/lib.rs`
(`SeaRoute` struct, `civ_sea_routes`, the four generalized helpers and
their updated call sites), `cartalith-native/crates/cartalith-civ/tests/
golden_parity_sea_routes.rs` (new).

## Memory optimization — `ResourcePotentials` unused-field fix (2026-08-16)

Prompted directly by the owner: generating a map "uses a ton of memory."
Investigated per `MEMORY_OPTIMIZATION_SCOPE.md` with real hands-on
measurement before and after, not assumption (`cartalith-porting-
discipline`'s own working rule).

**Confirmed dominant contributor**: `ResourcePotentials`
(`cartalith-civ`, Phase 2 milestone 5) computes and holds all 15
`Vec<f32>` fields simultaneously (~240 MB at 2048x2048), but
`build_settlement_suitability`'s mineral term only ever reads the 9-key
`SUIT_RESOURCE_KEYS` subset (copper/tin/iron/gold/salt/timber/lead/
silver/gems). Grepped the whole workspace for the other 6 field names
(`clay`/`buildstone`/`flint`/`obsidian`/`sulfur`/`alum`) -- the only
other reference found was a test-only variable inside
`cartalith-civ`'s own test suite. Confirmed via NLL-lifetime analysis of
`compute_civilisation()` that these six unused fields' backing arrays
(~96 MB combined) stay alive for the full ~40-line span from
computation through `build_settlement_suitability`, as one contributor
among the roughly 436 MB of `compute_civilisation()`-owned arrays alive
simultaneously at that point in the pipeline -- the single largest
confirmed contributor (over 50% of that local peak).

**Fix**: `cartalith-godot/src/lib.rs`'s `compute_civilisation()` --
`let resources = ...` changed to `let mut resources = ...`, and the six
unused fields' `Vec`s are reset to empty (`Vec::new()`) immediately
after `build_resource_potentials` returns, freeing their backing
allocations well before `resources` would otherwise be dropped at
function exit. No signature changes to `build_resource_potentials` or
`ResourcePotentials` itself -- the scope doc correctly flagged
restructuring the struct/builder as real, unjustified-without-
confirmation complexity, so this is the smallest fix that captures the
confirmed saving.

**Real before/after measurement** (Windows, real windowed app,
`PrintWindow`/`mouse_event` automation, `Get-Process` sampled every
~1.5s during generation, 2048x2048, seed 12345, Classic, 800 km, Phase 2
civ layer + rendering all active -- same technique the scope doc's own
baseline used):

| State | Before | After (run 1) | After (run 2) |
|---|---|---|---|
| Idle baseline | ~288-300 MB | 288.5 MB | -- |
| **Peak during generation** | **~1,445-1,653 MB** | **1,501.8 MB** | **1,434.5 MB** |
| Steady-state after completion | ~689-691 MB | 678.0 MB | 679.9 MB |

Both post-fix peaks land at or below the pre-fix range's own floor, and
steady-state dropped by ~10-12 MB in both runs -- a real, honest, but
modest improvement, not a dramatic one: the confirmed ~96 MB saving is a
real fraction of the ~1.1-1.3 GB total transient peak above baseline,
not its majority. **No persistent leak**, re-confirmed: two consecutive
generations' steady-state (678.0 MB, 679.9 MB) stayed flat, matching the
pre-fix finding.

**Not chased in this pass** (per the scope doc's own out-of-scope list):
the remaining ~1.3-1.4 GB of transient peak above steady-state is
mostly `cartalith-terrain`/`-climate`/`-erosion`/`-hydrology`'s own ~96
full-grid allocations plus `SuitabilityCtx`'s ~10 simultaneously-alive
field references, neither instrumented stage-by-stage in this pass;
GPU memory (separate pool); resolution-range UI policy (product
decision, not this investigation's call).

**Verification**: `cargo build -p cartalith-godot` clean (only
pre-existing unrelated warnings in `cartalith-gpu`), `cargo test -p
cartalith-civ` passes, `cargo clippy -p cartalith-civ -p
cartalith-godot --all-targets` clean for the new code (the one
`needless_borrow` warning at `cartalith-godot/src/lib.rs:253` is
pre-existing, from the earlier village-seeding wiring, unrelated to
this change), `cargo test --workspace` (0 regressions), `godot4
--headless --quit main.tscn` (clean load and exit).

**Files touched**: `cartalith-native/crates/cartalith-godot/src/lib.rs`
(`compute_civilisation()`), `MEMORY_OPTIMIZATION_SCOPE.md` (marked
done with the real numbers above).

## CPU multithreading milestone 1: Rayon-parallelize `cartalith-terrain`'s per-cell functions (2026-08-16)

`CPU_MULTITHREADING_SCOPE.md`'s first pass, prompted directly by the
owner ("multithreading support for cpus... doesn't seem to fully use
the cpu"). Unlike every GPU milestone this session, this needs no
`DECISIONS.md` §7a carve-out at all: parallelizing an existing per-cell
loop with Rayon doesn't change what gets computed, only which of this
machine's 16 logical cores computes which independent cell and in what
order -- for any function shaped `output[i] = f(input, i)` with zero
cross-cell read/write dependency, that preserves golden-parity output
**exactly**, bit-for-bit, not within a tolerance.

Added `rayon = "1"` to `cartalith-terrain/Cargo.toml` (same
direct-version-string convention `cartalith-gpu`'s `Cargo.toml` already
uses -- no `[workspace.dependencies]` table exists in this workspace to
join). Parallelized, each verified independent-per-cell/row/column by
reading the function fully before touching it:

- `compute_warp` -- rows are independent (`warp_x[i]`/`warp_y[i]`
  depend only on `x, y`, never another cell); parallelized with
  `par_chunks_mut(gw)` zipped across the two output fields.
- `compute_heterogeneity` -- same per-row shape; parallelized the
  fbm-heavy loop, left the trailing max-find/rescale passes sequential
  (a single O(n) scan, not the bottleneck, and touching a reduction adds
  verification risk for no measurable gain).
- `compute_height` -- same per-row shape; the most fbm/pridged-heavy
  per-cell loop in the crate (up to 5 octaves per cell), so this is
  where most of the real speedup below comes from.
- `compute_resistance` -- flat per-index loop (`resistance[i] =
  f(plate_id[i], age_field[i])`), parallelized directly with
  `par_iter_mut().enumerate()`.
- `gauss_blur`'s `box_h`/`box_v` (both private helpers, exercised by
  every crate that calls `gauss_blur`, e.g. `compute_stress`/
  `compute_flexure`/`build_orogeny_field`): `box_h` rows are
  contiguous in `dst`'s row-major layout, so `par_chunks_mut(w)` zipped
  with `src.par_chunks(w)` was a direct fit. `box_v`'s columns are
  *not* contiguous (`dst[y*w+x]` for fixed `x`, varying `y` is a
  strided write) -- rather than reach for `unsafe` to split disjoint
  strided slices (a real, common pattern elsewhere, but unjustified
  complexity when a simpler option exists), each column is computed in
  parallel into a column-major scratch buffer (`par_chunks_mut(h)`,
  each chunk genuinely contiguous there), then scattered into `dst` in
  one cheap sequential O(w*h) pass -- memory-bound, negligible next to
  the O(w*h*(2r+1))-shaped sliding-window work it replaces.

**Left sequential, matching `GPU_LAYER_INTEGRATION_SCOPE.md`'s own
"poor fit" catalogue for the identical underlying reason** (real
cross-cell/sequential state, not "hasn't been tried"): `build_plates`'s
Lloyd relaxation (per-cell writes reduce into shared per-plate
accumulators -- a genuine reduction, not an independent write),
`assign_plates`'s JFA (already GPU-verified as an iterative
pass-based algorithm), `compute_stress` (confirmed scatter-write
hazard -- two cells' boundary write to each other, Rayon has the
identical race problem plain threads would), `build_age_field`
(sequential two-pass chamfer distance transform), orogeny's
graph-tracing. All correctly out of scope per the doc, none forced.

**Golden-parity verification -- the headline check**: every existing
test for the touched functions passes completely unmodified, at
existing tolerances, with zero changes to any test file:
`golden_parity_blur.rs` (4/4), `golden_parity_flex_hetero_resist.rs`
(5/5, covers `compute_heterogeneity`/`compute_resistance`/
`compute_flexure`, which itself calls `gauss_blur`),
`golden_parity_height.rs` (3/3), `golden_parity_stress.rs` (3/3,
exercises `compute_stress`'s own `gauss_blur` calls),
`golden_parity_orogeny.rs` (5/5, exercises `build_age_field`'s
downstream consumers). Full `cargo test --workspace` -- every crate,
including `cartalith-engine`'s `golden_parity_pipeline.rs`/
`golden_parity_carve.rs` (the full pipeline, transitively exercising
every touched function together) and `cartalith-gpu`'s dev-only
CPU-vs-GPU cross-verification tests against the now-parallel
`cartalith-terrain` functions -- 0 failures, 0 modified tests. This is
the real, load-bearing evidence that parallel execution order doesn't
perturb any of these functions' floating-point results, not an
assumption.

**Real timing** (`cargo run --release --example timing_bench -p
cartalith-engine`, this session's real 16-logical-core machine, best of
3 timed runs after 1 warmup, `WorldParams::defaults` at each size, seed
12345):

| Size | Before | After | Speedup |
|---|---|---|---|
| 128x128 | 0.0973s | 0.0936s | ~1.04x |
| 512x512 | 0.6019s | 0.4859s | ~1.24x |
| 1024x1024 | 1.8328s | 1.3143s | ~1.39x |
| 2048x2048 | 7.0670s | 5.1071s | ~1.38x |

Honest reporting, not a hoped-for number: nowhere near a theoretical
16x, and that's expected, not a bug -- Amdahl's law. This pass touched
5 functions in one crate; the rest of `generate_terrain`'s pipeline
(plate seeding/Lloyd relaxation, JFA plate assignment, `compute_stress`,
`build_age_field`, all of `cartalith-climate`/`-erosion`/`-hydrology`,
river carving) is still fully sequential and sets the real ceiling
measured here. 128x128's near-1x result is expected too -- Rayon's own
per-call thread-pool dispatch overhead is a larger fraction of a
128x128 grid's total work than of a 2048x2048 grid's, so the smallest
size shows the least benefit (visible, not hidden, per this project's
own honest-reporting standard).

**Before-measurement method**: `git stash push -- Cargo.lock
crates/cartalith-terrain/Cargo.toml crates/cartalith-terrain/src/lib.rs`
to get a real pre-change build (not a mental estimate), ran the same
release-mode bench binary, then `git stash pop` to restore. Both
before and after used the identical bench harness and machine state.

**New tool, kept (not scope creep)**: `cartalith-engine/examples/
timing_bench.rs` -- this project's own discipline for every GPU/CPU
milestone is real measured numbers, not assumed ones
(`GPU_LAYER_INTEGRATION_SCOPE.md`'s and `CPU_MULTITHREADING_SCOPE.md`'s
own repeated language). A reusable `cargo run --release --example
timing_bench -p cartalith-engine` is a small, direct fit for that
standing need, not a one-off throwaway.

**Verification**: `cargo build -p cartalith-terrain`, `cargo test -p
cartalith-terrain` (all pre-existing tests, unmodified), `cargo clippy
-p cartalith-terrain --all-targets` clean (four `needless_range_loop`
warnings resolved with `#[allow]` + a one-line reason, matching this
workspace's own existing convention in `cartalith-civ`/`-climate`/
`-erosion` for loops where the index variable drives more than one
array), `cargo build --workspace` clean, `cargo test --workspace`
(every test, every crate, 0 failures, 0 modified).

**Out of scope for this pass, explicitly deferred, not forgotten**
(per the scope doc): `cartalith-civ` (two other forks were concurrently
active there when this pass started), `cartalith-climate`/`-erosion`/
`-hydrology` (each needs its own per-function independence read before
touching, same discipline this pass followed for `cartalith-terrain`),
GPU milestone 6's own flagged next step (`GpuContext` reuse across
stages, currently ~1.3-1.4s fixed overhead per call), the
integrated-GPU-alongside-dedicated idea (`HARDWARE_ACCELERATION.md`).

**Files touched**: `cartalith-native/Cargo.lock`,
`cartalith-native/crates/cartalith-terrain/Cargo.toml`,
`cartalith-native/crates/cartalith-terrain/src/lib.rs`
(`compute_warp`/`compute_heterogeneity`/`compute_height`/
`compute_resistance`/`box_h`/`box_v`), new
`cartalith-native/crates/cartalith-engine/examples/timing_bench.rs`,
`CPU_MULTITHREADING_SCOPE.md` (marked first pass done with the real
numbers above), `cartalith-native/docs/STATUS.md`.

## Real Android device pass: builds, installs, launches — blocked at the lock screen (2026-08-17)

First real on-device Android test this project has had, now that a real
device (OnePlus 6T, Android 14) was connected and authorized. Full record
in `ANDROID_BUILD_SCOPE.md` (new, repo root); summary here.

**Toolchain check, not toolchain setup**: every piece `TOOLCHAIN.md`
flagged as "the single highest-risk item" — `aarch64-linux-android`
rustup target, `cargo-ndk`, the NDK, `gdext`'s Android library paths in
`cartalith.gdextension`, the `"Android"` preset in `export_presets.cfg`
— was already correctly installed and wired from earlier work. Nothing
needed fixing. The existing `builds/android/Cartalith.apk` was just stale
(dated 2026-08-15, before all of that day's Phase 2/GPU/memory/threading
work) and debug-only.

**Build**: `cargo ndk -t arm64-v8a build --release -p cartalith-godot`
(2m38s, clean) produced a current release `.so`. `godot4 --headless
--export-release "Android"` failed as expected — no release keystore
exists yet (`TOOLCHAIN.md`'s own "No keystore yet, debug signing is
enough to sideload" note). Rebuilt the debug `.so` too, then `godot4
--headless --export-debug "Android"` succeeded, signed with Godot's own
debug keystore. (Note: `godot4` resolves via a WinGet shim visible to
PowerShell but not to Git Bash's `PATH` — this pass's Godot invocations
had to go through the `PowerShell` tool.)

**Install + launch**: `adb install -r` succeeded first try. Logcat
confirms a genuine successful engine start on real hardware — the
GDExtension loaded, Godot's native layer initialized, and a real OpenGL
ES 3.2 context was created against the device's actual Adreno 630 GPU
(`renderer: gl_compatibility`, `Using Device: Qualcomm - Adreno (TM)
630`). No crash, no ANR, no `gdext` error anywhere in the process's log.
Launch/idle memory via `adb shell dumpsys meminfo`: 151,982 KB PSS total,
78,244 KB private dirty.

**Real blocker, investigated not assumed**: a screenshot taken shortly
after launch came back solid black. Traced it, not shrugged at it —
`dumpsys power`/`dumpsys deviceidle` showed the screen had locked
(`mScreenLocked=true`), and Godot's own logcat showed why: `OnPause`→
`OnStop` fired ~140ms after `OnResume`, then `BufferQueue has been
abandoned`/`eglSwapBuffers failed: EGL_BAD_SURFACE` — the render surface
was torn down mid-init when the screen locked under it. Woke the screen
(`input keyevent KEYCODE_WAKEUP`, confirmed `mWakefulness=Awake`) and
tried `wm dismiss-keyguard` plus a manual swipe — keyguard stayed up.
`adb shell locksettings get-disabled` returned `false`, confirming this
device has a real, enabled lock credential (PIN/pattern/biometric), not
a bare swipe lock — `wm dismiss-keyguard` only works against "None"/
"Swipe" security, exactly the no-op observed. A repeat screenshot came
back byte-identical to the first: Android intentionally blanks
`screencap` output behind a secure keyguard, a real OS security
behavior. **This is a physical-access requirement, not a code or
toolchain gap** — guessing or forcing past a real lock credential was
never appropriate to attempt.

**Reached**: build/install/launch/engine-init/GPU-context-creation, all
confirmed real, on real hardware, for the first time. **Not reached**:
driving the golden path (Generate button, confirming the render, on-
device memory during generation, ANR/responsiveness under Android's
stricter watchdog) — needs the owner to physically unlock the phone
first, or run the already-installed APK themselves. Nothing else in the
repo needs to change before that — the build path itself is proven.

**Files touched**: new `ANDROID_BUILD_SCOPE.md` (repo root),
`cartalith-native/docs/STATUS.md` (criterion 4, Phase 1 row, Owner-only
items). No production code changed — the toolchain and export
configuration were already correct. Build artifacts (`.so`, `.apk`)
stay gitignored, not committed, per existing convention.

## Real Android device pass, part 2: golden path actually driven (2026-08-17, same day)

The owner unlocked the phone mid-session. Re-checked immediately rather
than assuming: `adb devices` still showed it, and a fresh screencap came
back a real 1.26MB image (vs. the earlier blanked 15KB ones) —
`dumpsys window` confirmed `isKeyguardShowing=false`.

The app (same process, alive backgrounded since the earlier launch) was
foregrounded and was **already showing a fully rendered world** — real
biome/hillshade terrain, rivers, faction-coloured settlements, the road
network, at the UI's own default params (512×512, seed 12345, 800km,
Classic, 40 settlements) — confirming the on-device renderer itself
works, not just that the engine initializes.

Tapped **Generate** for real (`adb shell input tap`) and sampled `adb
shell dumpsys meminfo` roughly every second through the run:

**Peak PSS during generation: ~283,326 KB (~277 MB)** at 512×512,
settling to a flat ~271,290 KB steady-state across four consecutive
samples afterward — no runaway growth, matching the "real peak, no
persistent leak" shape `MEMORY_OPTIMIZATION_SCOPE.md` already found on
Windows at a much larger 2048×2048 (not directly size-comparable, but
the same qualitative pattern). Generation completed in ~7-9s wall-clock
— slower than this session's own 512×512 desktop timing-bench numbers
(sub-second on a 16-thread machine), expected given a phone SoC's far
fewer/slower cores plus the not-yet-multithreaded Phase 2 civ layer
running in the mix.

A second screenshot taken as memory plateaued showed the **identical
rendered map** — same seed, same terrain/settlements/roads — real
confirmation the full pipeline (terrain → climate → erosion → hydrology
→ Phase 2 civ → render) ran to completion on-device and re-rendered
correctly, not just redrew stale state. **No ANR, no crash, no hang**:
`adb logcat` across the whole window showed no `ANR`/`FATAL`/`crash`
lines from this app.

**MVP criterion 4 is now fully closed** — both halves (build+install
and actually running the golden path) confirmed on real hardware with
real numbers.

**Files touched**: `ANDROID_BUILD_SCOPE.md` (blocker section replaced
with the real golden-path result and numbers), `cartalith-native/docs/
STATUS.md` (criterion 4, Phase 1 row, Owner-only items, all marked
fully done).

## Wire sea routes (Phase 2 milestone 13) into the Godot renderer (2026-08-17)

`civ_sea_routes` landed last session but was never reachable from
GDScript — `compute_civilisation()` computed land roads only. Wired the
full chain: `CivData` gained a `sea_routes: Vec<cartalith_civ::SeaRoute>`
field, `compute_civilisation()` calls `civ_sea_routes(&ports, ...)`
alongside the existing road build, and `cartalith-godot` exposes a new
`get_sea_routes() -> Array[Dictionary]` (`{points, brks, name}` — same
shape as `get_roads()` minus `way_type`, since `SeaRoute` carries no
highway/regional/road/track tier). `main.gd` fetches it after `generate()`
and hands it to `map_overlay.set_civ_data()` alongside settlements/roads.

**Rendering style**: reference HTML (line ~15511) draws sea lanes as a
solid dark-navy underlayer plus a lighter dashed overlay, distinct from
land roads' own solid-brown styling — `map_overlay.gd` reproduces this
(`SEA_ROUTE_UNDERLAY`/`SEA_ROUTE_DASH_COLOR` consts, `_draw_sea_route_segment`).
The dash overlay is walked manually (`_draw_dashed_polyline`), not one
`draw_dashed_line` call per vertex pair — a smoothed route's points are
only a few px apart, shorter than one dash+gap cycle, so restarting dash
phase at every vertex made every segment land inside the "on" portion
and render solid, not dashed. Carrying the dash phase continuously across
vertices fixed this — caught by real-app screenshot verification, not
assumed correct from reading the code.

**Real crash found and fixed by that same verification pass**: generating
an Archipelago-shape world (512×512, guaranteed multiple sea routes)
crashed the real windowed app outright — `godot.log` showed a GDScript
backtrace through `_draw_dashed_polyline` ending in a Godot-engine
`FATAL: Index p_index = 26906976 out of bounds` inside its internal
`Vector<T>::operator[]`, i.e. the renderer's own draw-command buffer grew
to ~27 million entries before overflowing. Root cause: `step := minf(
remaining_in_state, seg_len - traveled)` is mathematically positive
whenever the loop runs, but `phase` accumulates additively across every
vertex of a long route, and float drift can land `cycle_pos` close enough
to a `dash_len`/`period` boundary that the subtraction rounds to exactly
`0.0` — `step` then never advances `traveled`, so the `while traveled <
seg_len` loop spins forever, calling `draw_line` on a zero-length segment
each pass until the buffer overflows. Fixed by flooring `step` to a
sub-pixel epsilon (`0.001`) so every iteration guarantees forward
progress; the resulting overshoot is visually invisible. Re-ran the exact
same Archipelago/512 generation after the fix: no crash, dashed sea
routes render correctly and visibly distinct from land roads (confirmed
by cropping/zooming the real screenshot on the water between coastal
settlements).

**Verified**: `cargo build -p cartalith-civ -p cartalith-godot`, `cargo
test --workspace` (0 regressions), `cargo clippy -p cartalith-civ -p
cartalith-godot --all-targets` (clean for the new code; pre-existing
unrelated warnings elsewhere untouched), `godot4 --headless --quit
main.tscn` (clean load). Real windowed-app screenshot verification as
described above, including reproducing and then re-verifying the crash
fix on the identical config that originally crashed.

A debug-only string (`[DEBUG coastal=%d sea_routes=%d]`) had been left in
`main.gd`'s status label by the in-progress work that produced this
wiring — removed before commit; it was never meant to ship.

**Files touched**: `cartalith-native/crates/cartalith-godot/src/lib.rs`
(`sea_routes` field, wiring, `get_sea_routes()`), `cartalith-native/
godot-project/main.gd` (fetch + pass through, debug string removed),
`cartalith-native/godot-project/map_overlay.gd` (rendering + the crash
fix), this file, `docs/STATUS.md`.

## CPU multithreading milestone 2: Rayon-parallelize cartalith-civ (2026-08-17)

Follow-up to milestone 1 (`1faa16a`, `cartalith-terrain`) -- unblocked
once the two concurrent forks `CPU_MULTITHREADING_SCOPE.md` named as
the reason `cartalith-civ` was deferred (sea routes, memory
investigation) both landed (`71da1d5`, `62b9b51`).

Added `rayon = "1"` to `cartalith-civ/Cargo.toml` (same convention
milestone 1 used). Read every named candidate function's full body
before touching it, same discipline as milestone 1. Parallelized 16
functions confirmed genuinely `output[i] = f(input, i)` or a
fixed-radius read of an already-frozen buffer: `build_lithology`,
`build_slope_field`, `build_soil_fertility`, `build_water_access`'s two
per-cell passes, `build_biome_raster`, `build_wetland_mask`,
`build_carrying_capacity`, `build_npp`, `estimate_regional_density_
km2`, `build_resource_potentials`'s 15-field main loop, `apply_
resource_scarcity`, `build_raw_slope_field`, `build_route_corridors`
(both passes), `build_landmass_quality`'s final per-cell fold,
`build_flood_field`, `build_settlement_suitability`, `build_travel_
cost`, and `assign_territory`'s inner per-capital cell loop.

`build_resource_potentials`'s 15 simultaneous output fields (the same
ones `MEMORY_OPTIMIZATION_SCOPE.md` flagged for their memory
footprint) don't zip cleanly as 15 separate `par_iter_mut()` slices, so
the per-cell math computes into one `[f32; 15]` per cell in parallel,
then a cheap sequential pass scatters the 15 values into their named
output `Vec`s -- plain data movement, negligible next to the branchy
geology math it follows.

`apply_resource_scarcity` needed real care, not a direct `par_iter_
mut` swap: it ranks all non-zero land-cell values by a global sort to
find a keep-threshold. Parallelized the value collection (`into_par_
iter().filter_map().collect()` -- order-preserving, and irrelevant
anyway since the result is sorted immediately after), the land-cell
count, `par_sort_unstable_by` (safe despite "unstable": the threshold
depends only on the VALUE at rank `keep-1`, never on which physical
duplicate of a tied value ends up there), and the final per-cell
threshold-application loop.

**Left sequential, and why** (same bar as every prior GPU/CPU pass this
session): `chamfer_dist` (two-pass raster scan, each cell reads its own
predecessor from the SAME pass -- a genuine wavefront dependency);
`jfa_dist`/`build_coast_sdf` (iterative Jump Flooding, already
GPU-verified as iterative); `build_water_bodies` (priority-flood);
`label_land_components` and `build_landmass_quality`'s own flood-fill
(connected components); `road_dijkstra`/`build_road_network`/`civ_
hierarchical_network_topology`/`civ_sea_routes`/`civ_consolidate_and_
smooth_ways` (graph/Dijkstra/MST); `assign_landmass_factions`/`place_
settlements`/`civ_seed_villages`/naming (RNG-stream order matters, not
grid-shaped anyway); `fresh_river_order` (delegates entirely to
`cartalith-hydrology`, outside this crate).

**Golden-parity verification, exact as required**: `cargo build -p
cartalith-civ` clean. `cargo test -p cartalith-civ`: every existing
golden-parity suite passes completely unmodified at existing
tolerances (resource potentials, settlement suitability/placement/
naming, village seeding, hierarchical network, road network/
consolidation, sea routes, water bodies, carrying capacity/NPP/
density, settlement prereqs). `cargo clippy -p cartalith-civ
--all-targets` clean for this pass's own code (two pre-existing,
unrelated warnings confirmed by line number: a `needless_range_loop`
note in `civ_sea_routes`, untouched this pass, and a test-fixture
`excessive_precision` note). Full `cargo test --workspace`: 68 test
suites, 0 failures, 0 modified tests -- every other crate (including
`cartalith-godot` and `cartalith-gpu`'s own cross-verification tests)
unaffected. `cargo build --workspace` clean.

**Real timing**: `compute_civilisation()` itself can't be benchmarked
directly from outside `cartalith-godot` -- it's a private `fn` in the
one crate `ARCHITECTURE.md` restricts to `cdylib`-only, no `rlib`
target to link a bench binary against. A new `cartalith-civ/examples/
timing_bench.rs` instead chains this crate's own real per-cell
pipeline in the same order `golden_parity_settlement_naming.rs`'s
`compute_named_settlements` test helper already established, fed with
real `generate_terrain` output (not synthetic data) -- the real
upstream half of what `compute_civilisation()` runs. Measured via
`git stash` of this pass's own changes for a true sequential baseline
from the identical benchmark, then restored (16-core machine, best of
3, seed 12345):

| Size | Before | After | Speedup |
|---|---|---|---|
| 128x128 | 0.0074s | 0.0075s | ~0.99x |
| 512x512 | 0.1399s | 0.1044s | ~1.34x |
| 1024x1024 | 0.6615s | 0.4340s | ~1.52x |
| 2048x2048 | 3.5568s | 1.9625s | ~1.81x |

Better-scaling than milestone 1's own terrain result (~1.38x at
2048x2048) -- more, and larger, genuinely independent per-cell work in
this crate (`build_resource_potentials`'s 15 fields, `build_
settlement_suitability`'s large branchy body) than `cartalith-terrain`'s
five functions had. Combined with milestone 1's own number, a full
`generate_terrain` + this crate's civ layer at 2048x2048 goes from
`7.0670s + 3.5568s = 10.62s` sequential to `5.1071s + 1.9625s = 7.07s`
parallelized -- roughly a third off real wall-clock time for the two
subsystems parallelized so far, honestly reported, not the theoretical
16x (`chamfer_dist`/`jfa_dist`/flood-fill/priority-flood all stay
sequential and set the real ceiling, same reasoning as milestone 1).

**Files touched**: `cartalith-native/crates/cartalith-civ/Cargo.toml`
(`rayon` dependency), `cartalith-native/crates/cartalith-civ/src/lib.rs`
(the 16 functions above), `cartalith-native/crates/cartalith-civ/
examples/timing_bench.rs` (new), `CPU_MULTITHREADING_SCOPE.md`, this
file, `docs/STATUS.md`.

## Phase 1 closeout: credits screen + crate license audit (2026-08-17)

`ROADMAP.md`/`PROVENANCE.md` both name two items as part of Phase 1's own
definition of done, easy to forget because neither is visible until
someone looks: a credits screen (carrying forward the reference HTML's
own `#creditsModal` attribution, which "dropping in the rewrite would
quietly withdraw") and a crate license audit ("release-blocking...
neither is visible until someone looks"). Both sat open in `STATUS.md`'s
known-open-items list since early in the port. Picked up now.

**License audit**: installed `cargo-license` and ran it against the
whole workspace with `--all-features`. Real result, not assumed clean:

- **~190 of ~200 total dependencies** (including transitive) are
  permissively licensed — MIT, Apache-2.0, BSD-2-Clause, Zlib, ISC,
  Unlicense, CC0-1.0, or 0BSD, individually or dual/tri-licensed. Every
  core dependency this port actually relies on falls here: `rayon`
  (CPU parallelism), `wgpu`/`naga` (GPU compute), `serde`/`serde_json`,
  `zip`/`flate2`/`crc32fast` (the save format), `glam`, and all nine of
  this project's own crates.
- **`godot`/`gdext`** (`godot`, `godot-core`, `godot-ffi`,
  `godot-macros`, `godot-codegen`, `godot-cell`, `godot-bindings`,
  `gdextension-api` — 8 crates) are **MPL-2.0** (Mozilla Public License
  2.0), the one weak-copyleft entry in the whole tree. Flagged, not
  hidden: MPL-2.0's copyleft is file-level and applies to modifications
  of MPL-licensed files themselves — this project depends on `gdext`
  unmodified, as the Rust↔Godot binding the entire port is built on, not
  as vendored/modified source.
- **`libbz2-rs-sys`** (pulled in transitively via `zip`'s optional bzip2
  support) reports its license as the literal string `bzip2-1.0.6` —
  the original bzip2 license (Julian Seward), a permissive BSD-style
  license, not a standard SPDX identifier `cargo-license` recognized by
  name.
- **No GPL, LGPL, AGPL, or other strong-copyleft dependency anywhere**
  in the workspace, across all features.

**Credits screen**: a new `godot-project/credits.gd` (`AcceptDialog`,
wired into `main.tscn` as a `CreditsDialog` node) reachable via a new
header "ⓘ" button (`CreditsButton`, `main.tscn`'s `HeaderRow`) —
mirroring the reference HTML's own header-ⓘ `#creditsModal` affordance.
Content, in a scrollable `RichTextLabel` (BBCode): the reference's own
attribution (`reference/Cartalith Gen1 v2.10.html`'s `#creditsModal`,
line ~2043 — programming/code sources studied, academic principles for
terrain/tectonics/climate and civilization/population, condensed from
the original), plus a new section this port owns on top of that: the
license-audit findings above, in the same "studied, not copied, and
here's exactly what we depend on" spirit.

**Verified**: `cargo build --workspace`, `cargo test --workspace` (0
regressions — pure GDScript/scene change, no Rust logic touched),
`godot4 --headless --quit main.tscn` clean load. Real windowed-app
screenshot verification (`PrintWindow`/`mouse_event`, minimize/restore
focus trick): clicked the real ⓘ button at its real on-screen position,
confirmed the dialog opens with the attribution section visible, then
scrolled and confirmed the license-audit section renders in full down to
its closing note — not a placeholder, not assumed from reading the code.

**Files touched**: `cartalith-native/godot-project/main.tscn`
(`CreditsButton`, `CreditsDialog` + its `ScrollContainer`/
`RichTextLabel`), new `cartalith-native/godot-project/credits.gd`,
`cartalith-native/godot-project/main.gd` (button wiring), `docs/
STATUS.md` (Phase 1 row, known-open-items list).

## GPU layer integration milestone 8: context reuse across `generate_terrain`'s stages (2026-08-17)

Milestone 6's own flagged next optimization, picked up directly: each of
its five GPU dispatches (warp, heterogeneity, plate assignment, and two
separate `gauss_blur_grid_gpu` calls) independently paid `instance.
request_adapter`/`adapter.request_device` (~1.3-1.4s each, flat
regardless of grid size — the dominant cost below 2048×2048).

**New `cartalith-gpu` API**: `GpuDevice` (adapter+device+queue, no
pipeline) + `init_gpu_shared_device()`. Confirmed (not assumed) `wgpu::
Device`/`wgpu::Queue` are cheap `Clone` handles by reading `wgpu`
30.0.0's own source (`#[derive(Debug, Clone)]`, Arc-backed via
`dispatch::DispatchDevice`/`DispatchQueue`) before relying on it. Each
of the four reused kernels gained an `init_gpu_X_with(gpu: &GpuDevice)`
pipeline builder, and the four milestone-6 wrappers gained `_with`
siblings (`warp_grid_gpu_with`/`heterogeneity_grid_gpu_with`/
`gauss_blur_grid_gpu_with`/`assign_plates_grid_gpu_with`) that build a
pipeline on an existing device instead of requesting a new one —
infallible past device creation, so no more per-stage `Option`/`.ok()?`.
The original standalone `init_gpu_X()`/`X_grid_gpu()` functions are
byte-untouched; every milestone 1-6 test calling them directly still
exercises the identical code path. `REUSED_STAGE_MAX_STORAGE_BUFFERS =
8` (JFA's own bind-group size) sizes the shared device's limits up
front, since `wgpu` limits can't be raised after device creation.

`generate_terrain` now calls `init_gpu_shared_device()` once (behind `if
p.use_gpu`) and every GPU call site uses `gpu_device.as_ref().map(|gpu|
..._with(gpu, ...))` — a `None` still falls through to the exact same
CPU fallback as before, just from one failure point instead of five.

**CPU path unchanged, confirmed not assumed**: `cargo test --workspace`
— 0 failures, every golden-parity test unmodified, which is only
possible if `use_gpu=false`'s output stayed byte-identical.

**Real timing** (same benchmark as milestone 6, release build, single
run per size):

| Size | Before (M6) | After (M8) | Ratio before | Ratio after |
|---|---|---|---|---|
| 128×128 | 1.44s | 813ms | 0.06× | 0.11× |
| 512×512 | 1.46s | 689ms | 0.41× | 0.76× |
| 1024×1024 | 2.32s | 1.39s | 0.78× | **1.14× — new win** |
| 2048×2048 | 6.03s | 5.92s | 1.19× | 0.98× |

**GPU now beats CPU starting at 1024×1024** (previously only
2048×2048, and only by 19%) — a real crossover moved down a full size
tier. Reported honestly: 2048×2048's own ratio dipped from a 19% win to
essentially even between the two runs — almost certainly single-run
variance (CPU time alone moved from 7.20s to 5.83s with zero code
changed on that path), not a regression, and the benchmark's own doc
comment already flags "not averaged" as a known limitation. Not
re-run to chase a better number.

**Verified**: `cargo build --workspace`, `cargo test --workspace` (0
failures), `cargo clippy -p cartalith-gpu -p cartalith-engine
--all-targets` (clean; one new `too_many_arguments` warning on
`heterogeneity_grid_gpu_with`, fixed with the same `#[allow]`
convention already used ~35 times elsewhere in this workspace).

**Not attempted**: pipeline caching *across* repeated `generate_terrain`
calls (this milestone only shares the device *within* one call);
averaging the timing benchmark to reduce single-run noise; GPU
milestone 7 (climate), untouched.

**Files touched**: `cartalith-native/crates/cartalith-gpu/src/lib.rs`
(`GpuDevice`, `init_gpu_shared_device`, four `_with` pipeline builders,
four `_with` wrapper functions), `cartalith-native/crates/
cartalith-engine/src/lib.rs` (`generate_terrain`'s five GPU call sites),
`GPU_LAYER_INTEGRATION_SCOPE.md` (milestone 8 section), this file,
`docs/STATUS.md`.

## New crate `cartalith-spatial`: standalone tiling/spatial-index base, not integrated (2026-08-17)

Prompted by the owner, directly: "LOD and zoom etc might be out of scope for
the base, but they're still goals in this project. The base should be
present before integration." Given three concrete scope options, the owner
chose "data structures + dirty-region/versioning scaffolding" — build the
foundational data structures now, real and unit-tested, but touch nothing in
the live generation/rendering pipeline. Full reasoning, scope boundary, and
what stays deferred: `LOD_TILING_BASE_SCOPE.md` (repo root). The research
this responds to (`TERRAIN_ARCHITECTURE_RESEARCH.md`, also filed this
session) describes a much larger real-time camera/LOD/streaming/painting
architecture that doesn't fit Cartalith's current one-shot static-generation
product shape at all — this crate is deliberately the narrow, safe slice of
that research: reusable data structures with zero opinion on Cartalith
semantics, sitting completely unreferenced by any other crate.

**Built**, all in one new crate `cartalith-native/crates/cartalith-spatial`
(no `gdext` dependency, per `ARCHITECTURE.md`'s crate-boundary rule):

- **`TiledField<T>`** — wraps a flat, row-major `Vec<T>` (the same
  Structure-of-Arrays layout `WorldState`/`CivData` already use) with
  zero-copy `whole()`/`row()`/`column()`/`region()`/`tile()` views, both
  read-only and mutable. `tile_size` is a constructor parameter, not a
  hardcoded constant — no real workload exists yet to benchmark 64 vs. 128
  vs. 256 against (`TERRAIN_ARCHITECTURE_RESEARCH.md` §31 says exactly this).
  `column()` returns a lazy iterator rather than a slice since a column
  isn't contiguous in row-major storage (stride = width) — a real, not
  hypothetical, distinction from `row()`'s genuine `&[T]` slice.
- **`QuadTree<T>`** — packed (`Vec<Node<T>>`, integer child indices via a
  `NO_CHILD` sentinel, never `Box<Node>`/pointers), built bottom-up from a
  flat field with a caller-supplied `leaf_max` and a caller-supplied
  `flags_of` closure for per-node aggregate flags — the crate assigns no
  meaning to any bit, deliberately (no `has_river: bool` baked into a
  library crate with no real caller to say what "has river" should mean
  yet). Handles non-power-of-two/odd dimensions by omitting zero-width or
  zero-height quadrants during the split rather than special-casing them.
  `query_region_counted` returns both the matching leaves and how many nodes
  were actually visited — the real proof that bounds-rejection skips whole
  subtrees rather than merely returning the right answer by brute force.
- **`DirtyTracker`** — per-tile dirty flag with a caller-supplied reason
  string (not Cartalith's specific `HEIGHT_DIRTY`/`BIOME_DIRTY` field-
  dependency semantics from the research doc — that dependency graph has no
  real caller yet either) plus a monotonic `u64` version counter that only
  `mark_dirty` bumps; `clear_dirty` acknowledges without incrementing, since
  clearing isn't itself a data change.
- `serde` `Serialize`/`Deserialize` on all three (reusing the exact
  `serde = "1.0.229"` version `cartalith-io` already pins, not a new
  version choice) — round-trip tested via `serde_json` (dev-dependency
  only), not wired to any actual disk-paging system.

**Verified**: 24 real unit tests, not compile-only coverage — tile-boundary
correctness including the off-by-one edge-tile case (world dimensions not an
exact multiple of `tile_size`), a mutable view's writes landing in the
correct backing-array cells, quadtree aggregate min/max matching the real
source data, non-power-of-two dimension handling (5×3, doesn't panic, every
cell covered by exactly one leaf), and — the test that actually proves the
point rather than assuming it — a 64×64/leaf_max-4 tree (a real multi-level
structure, `>100` nodes) queried with a 1×1 region visiting `< len()/4`
nodes, plus a predicate-search test using a genuinely partial query region
(not the whole field, which would reject nothing) to find one specific cell
among 1024 by descending only into leaves whose own min/max aggregate
couldn't rule it out. `cargo build/test/clippy -p cartalith-spatial` all
clean. Full `cargo test --workspace` clean (one `generate_terrain_gpu_path_
is_deterministic_and_valid` failure on the first run reproduced the
already-documented pre-existing GPU-driver flakiness under parallel test
scheduling — passed in isolation and on a full clean re-run; this crate has
no GPU code and nothing depends on it, so it cannot be the cause).

**Confirmed untouched, deliberately**: `cartalith-engine`, `cartalith-
terrain`, `cartalith-climate`, `cartalith-erosion`, `cartalith-hydrology`,
`cartalith-civ`, `cartalith-godot`, every `.gd`/`.tscn` file — nothing else
in the workspace references `cartalith-spatial` at all. It exists purely so
that whenever Phase 3 (3D) or a real large-world need actually triggers LOD/
tiling integration, that work starts from a tested foundation instead of a
green field.

**Files touched**: new `cartalith-native/crates/cartalith-spatial/`
(`Cargo.toml`, `src/lib.rs`), `LOD_TILING_BASE_SCOPE.md` (marked done), this
file, `docs/STATUS.md`.

## GPU layer integration milestone 7: climate's wind/rain loop -- real loss even with milestone 8's fix (2026-08-17)

Built `simulate_weather`'s inner loop on GPU (`gpu_weather.wgsl`: `evap_main`
/`advect_main`/`deposit_main`, evaporation+boundary-reset fused into one
dispatch, advection and deposit each their own since WGSL has no
cross-workgroup barrier mid-dispatch), using milestone 8's shared-`GpuDevice`
pattern from the very start rather than repeating milestone 6's original
per-call-context mistake.

**Real refactor first**: extracted `simulate_weather`'s previously-inline
setup into `pub fn build_weather_grid` (returns a new `WeatherGrid`) and the
post-loop teardown into `pub fn finish_weather_grid`, both in
`cartalith-climate`, with `simulate_weather` itself now calling them --
pure extraction, zero behavior change (`golden_parity_weather.rs`'s four
cases pass exactly as before). This keeps `cartalith-climate` itself free of
any `cartalith-gpu` dependency, matching every other subsystem crate's
convention -- the branching/dispatching logic lives entirely in
`cartalith-engine`, which already depends on both.

**Correctness**: no noise dependency in this function at all, verified
directly against the real, untouched `cartalith_climate::simulate_weather`
(milestone 4's GPU-vs-real-CPU discipline), at the real production default
`iters=70`. Max abs diff `1.79e-7` -- essentially f32 machine epsilon; 70
iterations of gather/advect/deposit did not compound meaningfully (bounded,
non-chaotic arithmetic, unlike nested noise evaluations). `WEATHER_TOLERANCE
= 1e-5`, ~50x headroom over the measured value.

**Real timing, the honest finding**: unlike every prior GPU-wired stage,
this kernel's own working set is capped (`ww = min(gw, 240)`) and doesn't
grow with map resolution once `gw >= 240` -- every resolution preset this
port offers. Measured at the kernel's real production working size (240x240
coarse grid, 70 iterations, sourced from a real 2048x2048 map): **GPU =
23.8ms, CPU = 22.2ms, ratio 0.93x -- GPU loses**, even with milestone 8's
shared-device fix applied from the start. 210 total dispatches (70 iters x
3 passes) against a 57,600-cell working set is too little per-dispatch work
to amortize even the small remaining fixed per-dispatch overhead once
context-creation is no longer the dominant cost. Joins `compute_resistance`
(milestone 4, 0.38x) as a second confirmed case of a GPU-verified kernel
that shouldn't actually run on GPU -- for a different structural reason
(dispatch-count-dominated, not formula-triviality-dominated).

**Wired anyway**, behind `p.use_gpu`, both `simulate_weather` call sites in
`generate_terrain` (initial pass + post-river-carve recompute) -- `"weather"`
joins the known `gpu_stages_used` entries, per-stage CPU fallback preserved,
for architectural consistency even though this stage is expected to keep
losing regardless of map size.

**Real pre-existing bug found and fixed, unrelated to this milestone's own
scope**: `cartalith-civ/examples/timing_bench.rs` (CPU-multithreading
milestone 2) and `cartalith-engine/examples/timing_bench.rs`
(CPU-multithreading milestone 1) collided at the identical
`target/debug/examples/timing_bench.exe` output path -- broke `cargo test
--workspace`/`cargo build --workspace --examples` outright for anyone, not
just this task. Fixed by renaming `cartalith-civ`'s to
`civ_timing_bench.rs` (its own scope doc's `-p cartalith-civ`-qualified
`cargo run --example` commands updated to match; `cartalith-engine`'s stays
unchanged, having existed one commit earlier).

**Verified**: `cargo build --workspace`, `cargo test --workspace` (70
suites, 0 failures, 0 modified tests -- including `golden_parity_weather.rs`
unchanged after the extraction), `cargo clippy -p cartalith-gpu -p
cartalith-climate -p cartalith-engine --all-targets` clean.

**Files touched**: `cartalith-native/crates/cartalith-gpu/shaders/
gpu_weather.wgsl` (new), `cartalith-gpu/src/lib.rs` (`WeatherParams`,
`GpuWeatherContext`, `init_gpu_weather_with`, `dispatch_gpu_weather`,
`simulate_weather_loop_gpu_with`, two new tests), `cartalith-gpu/Cargo.toml`
(`cartalith-climate` dev-dependency), `cartalith-climate/src/lib.rs`
(`WeatherGrid`, `build_weather_grid`, `finish_weather_grid`,
`simulate_weather` refactored to call them), `cartalith-engine/src/lib.rs`
(both `simulate_weather` call sites GPU-gated, `gpu_stages_used` allowlist),
`cartalith-civ/examples/civ_timing_bench.rs` (renamed from
`timing_bench.rs`), `CPU_MULTITHREADING_SCOPE.md`, `GPU_LAYER_INTEGRATION_
SCOPE.md`, this file, `docs/STATUS.md`.

## CPU multithreading milestone 3: Rayon-parallelize climate/erosion/hydrology (2026-08-17)

Covers `CPU_MULTITHREADING_SCOPE.md`'s own "natural follow-up" from
milestone 1 (`1faa16a`, `cartalith-terrain`) and milestone 2 (`d938afb`,
`cartalith-civ`): `cartalith-climate`, `cartalith-erosion`,
`cartalith-hydrology`. Read every candidate function's body in full
before touching it, checking each against known hazard categories
(flow accumulation, priority-flood, scatter-write, per-particle
sequential state, floating-point summation-order sensitivity) rather
than assuming safety from a function's name.

**`cartalith-climate`**: the deepest pass — most of the crate genuinely
parallelizes. `compute_temperature`, `apply_cryosphere_albedo` (parallel
within each of 6 passes), `blur_coarse` (both passes are direct 3-tap
convolutions, no running-sum unlike `cartalith-terrain::gauss_blur`'s
box_h/box_v — simpler to parallelize), `deflect_flow` (parallel within
each iteration), `build_wind` (its pressure-gradient max found via
`reduce(f64::max)` — exact, since max is order-independent unlike a
sum), `compute_ocean_current` (its western-intensification pass stays
row-parallel, each row keeping its own sequential west-distance scan —
the same "per-row independent, within-row sequential" shape
`gauss_blur`'s box_h uses), `ocean_sst_anomaly`, `apply_ocean_currents`,
`apply_climate_moisture_correctors`, and `simulate_weather`'s `iters`
loop (all three per-iteration passes — evaporation, semi-Lagrangian
advection, precipitation — parallel within one iteration, `iters` itself
sequential; confirms `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7's own
"gather-shaped" finding for this exact loop applies equally to the CPU
path).

**`cartalith-erosion`**: mixed, the flagged hazards confirmed real.
Parallelized: `erode_thermal`'s final clamp (not its `delta` computation
— see below), `stream_power_kernel`'s `u`/`u_max` normalization, its
`rcv`/`rdist` receiver computation, its `cc` computation (`order[k]`'s
indirection turned out not to matter — the computation depends only on
the resulting index `i`, and `order` visits every index exactly once,
so iterating `i` directly in parallel is the identical computation
without a scatter-via-collect step), its final clamp; `isostatic_
rebound`'s `d`-field fill and combine (`any` via `.par_iter().any()`, a
boolean OR — order-independent); `recompute_resistance_after_erosion`.
Confirmed unsafe, not assumed: `droplet_kernel` (genuine per-droplet
sequential state, verified by reading the function, not taken on the
scope doc's own leading hypothesis alone); `erode_thermal`'s `delta`
computation (scatters into up to 4 neighbours' `delta[j]` in the same
pass — the same cross-cell hazard `compute_stress` has); `stream_power_
kernel`'s `area` flow-accumulation pass and its entire main `p.iters`
loop (a genuine donor-receiver wavefront *within* one iteration, per
the code's own receivers-before-donors comment). `ss` (a running sum
gating a branch) deliberately left sequential — summation order affects
rounding, unlike a max, and a parallel reduction could rarely flip which
side of `ss < 1e-3` it lands on.

**`cartalith-hydrology`**: confirmed mostly sequential, matching the
scope doc's own leading hypothesis. `compute_flow` (flow accumulation)
stays fully sequential — its own doc comment already named the
`acc[best]+=acc[i]` scatter hazard before this pass started; only its
rain-rescale loop parallelizes. `strahler_from_receivers`, `trace_
river_polylines`, `enforce_channel_descent` all confirmed genuinely
sequential and, separately, not grid-sized (channel-cell-count or
source-count sized — small real payoff even if theoretically safe). The
one real win: `build_channels`'s main channelization loop — genuinely
per-cell, parallelized by row.

**Golden-parity verification, exact as required**: every existing test
in all three crates passes completely unmodified — every golden-parity
suite (temperature/weather/ocean_current/deflect_flow/moisture_
correctors for climate; droplet/thermal/streampower/rebound for erosion;
flow/river/polylines for hydrology). `cargo clippy --all-targets` clean
on all three, zero new warnings. Full `cargo test --workspace`/`cargo
build --workspace`: 0 failures, 0 modified tests, every other crate
(including the concurrently-landed GPU weather milestone 7 work above)
unaffected.

**Real timing** (`timing_bench`, 16-core machine, seed 12345). Measured
via a temporary `git worktree` at the last clean commit rather than
`git stash` — a concurrent fork's own uncommitted GPU-weather extraction
lived in this same `cartalith-climate/src/lib.rs` file, and stashing the
whole file would have reverted their in-progress work too:

| Size | Before | After | Speedup |
|---|---|---|---|
| 128x128 | 0.1049s | 0.0797s | ~1.32x |
| 512x512 | 0.5222s | 0.3363s | ~1.55x |
| 1024x1024 | 1.4109s | 1.1230s | ~1.26x |
| 2048x2048 | 5.1970s | 4.7815s | ~1.09x |

Real, honest, and unusually better-scaling at smaller sizes than at
2048x2048 for this session's own timing results — plausibly climate's
coarse weather grid (capped `min(gw,240)`) keeps the `iters` loop's own
per-cell work roughly constant past a certain full-resolution size,
while erosion/hydrology's full-resolution passes keep growing, shrinking
the parallelized fraction of total work as gw/gh grows. Not chased
further this pass — a real candidate if a future pass revisits this
crate. Combined with milestones 1/2's already-measured terrain+civ
speedups, this is the third and (for now) final subsystem this session's
CPU-multithreading effort covers.

**Files touched**: `cartalith-native/crates/cartalith-climate/{Cargo.toml,
src/lib.rs}`, `cartalith-erosion/{Cargo.toml,src/lib.rs}`,
`cartalith-hydrology/{Cargo.toml,src/lib.rs}`, `CPU_MULTITHREADING_
SCOPE.md`, this file, `docs/STATUS.md`.

## Phase 2 milestone 16: provinces (`_civGenerateProvinces`) (2026-08-17)

Resolved a blocker recorded, not assumed stale, since the milestone-9
investigation that found it: the reference's `civTerritory` (this
function's real input) has zero programmatic producer anywhere in the JS
— only an interactive paint tool and a save/load deserializer ever write
it. Milestone 10's own `assign_territory` (`DECISIONS.md` §7b), built for
an unrelated reason (the port needed *a* territory system since the
reference never had one), turned out to produce the exact same per-cell
shape this function needs (`Vec<i32>` faction id, `0` = unowned). Verified
by reading the real reference source directly before porting, not by
trusting the earlier note's own summary — confirmed compatible.

`civ_generate_provinces` (`cartalith-civ`): a settlement-seeded Voronoi
partition of each faction's own territory, restricted to same-faction
seeds. Seeds are `Capital`/`City`-tier settlements (this port's five-tier
`SettlementKind` reduces the reference's rank>=3 filter cleanly, since
metropolis/university/industrial were never ported into `SettlementKind`
at all — nothing else rank>=3 could mean here). No city-tier seed in a
faction falls back to its single highest-population settlement. A faction
that owns territory but placed zero settlements gets no province.

No JS reference to golden-verify the province step against (same reason
as territory itself, §7b) — verified by 5 real unit tests: multi-seed
Voronoi split, single-fallback-seed case, a province never claims a cell
outside its own faction's territory, a faction with territory but no
settlements stays unassigned, every reachable owned cell partitions into
a real province with no gaps.

Wired into `cartalith-godot`'s `CivData`/`compute_civilisation()`
(`provinces: Vec<i32>`, `province_list: Vec<Province>`) with two new
`#[func]`s: `get_provinces()` and `build_province_boundary_texture()` (a
boundary-line overlay, not a per-province fill — a real per-province
colour palette is a UI/UX design decision this task didn't make, since
province count isn't bounded the way `CIV_FACTION_COUNT` is).
**Deliberately not wired into `main.gd`/`map_overlay.gd`** — no new
toggle, no new `TextureRect` — left for a dedicated UI/UX pass per this
port's own standing practice, rather than improvising scene-tree changes
inside a data-porting task. Both new methods verified against real
generated data via a temporary (uncommitted) headless GDScript: 7
provinces at seed 12345/512²/Classic, a real non-empty 512×512 boundary
texture (2,262 boundary pixels), no crash — the same real-invocation
discipline this session's sea-routes crash was caught by, applied here
even with no permanent UI yet to screenshot.

**Verified**: `cargo test -p cartalith-civ` (5 new tests, 64 total, 0
failed), `cargo test --workspace` (0 regressions), `cargo clippy -p
cartalith-civ -p cartalith-godot --all-targets` clean, `godot4 --headless
--quit main.tscn` clean load, plus the real headless functional check
above.

**Files touched**: `cartalith-native/crates/cartalith-civ/src/lib.rs`
(`Province`, `civ_generate_provinces`, 5 tests), `cartalith-godot/
src/lib.rs` (`CivData` fields, wiring, `get_provinces()`,
`build_province_boundary_texture()`), `PHASE2_SCOPE.md`, this file,
`docs/STATUS.md`.

## UI/UX catch-up: render province boundaries (2026-08-17)

Follow-up to the province-generation pass above, which deliberately left
rendering out: a per-province *fill* colour needs a real UI/UX decision
(province count is unbounded, unlike `CIV_FACTION_COUNT`'s fixed
Okabe-Ito palette), so `build_province_boundary_texture()` was shipped
as a boundary-only texture instead -- a thin ink-toned line wherever two
orthogonally-adjacent cells belong to different provinces, transparent
everywhere else. That sidesteps the palette problem entirely: no palette
needed for a line.

Wired it the same way `build_territory_texture()` already was -- a new
`ProvinceBoundaryView` `TextureRect` sibling of `TerritoryView` (drawn on
top of it, same `expand_mode`/`stretch_mode`/`mouse_filter` properties,
default `visible = false`), a matching `ProvinceLayerCheck` checkbox in
`MapLayersCard` right after `TerritoryLayerCheck` (default OFF, same
style), and the same set-on-generate/clear-on-load-save pattern
`territory_view.texture` already follows in `main.gd`. `get_provinces()`
turned out to expose no cell-to-province lookup (`get_province_id_at`,
named in its own doc comment, isn't actually a real `#[func]`) --
skipped wiring a hover card for it rather than inventing a query that
isn't there.

**Verification**: `cargo test --workspace` (0 regressions), `cargo
build -p cartalith-godot` clean, `godot4 --headless --quit` clean load.
Real windowed-app screenshot verification (Classic, seed 12345, 2048x2048,
40 settlements) confirmed the layer toggles render correctly with visible
checkmarks and the map updates -- but a thin 1px boundary line proved
genuinely hard to distinguish by eye from roads/coastline linework at
normal zoom in a static screenshot, so screenshot inspection alone
wasn't conclusive. Followed up with a direct, objective check instead: a
temporary headless script (deleted after use, not committed) called
`generate()`/`get_provinces()`/`build_province_boundary_texture()`
directly and counted real non-transparent pixels -- **7 provinces,
2,262 boundary pixels on a real 512x512 texture** (the same figure the
province-generation pass's own earlier headless check reported,
confirming reproducible real data, not a fluke), `build_territory_texture()`
also confirmed present. Combined with the GDScript wiring being
mechanically identical to `territory_view`'s already screenshot-verified
pattern, this is real, working data reaching a real, working render path.

**Files touched**: `cartalith-native/godot-project/main.tscn`
(`ProvinceLayerCheck`, `ProvinceBoundaryView`), `main.gd` (wiring),
`docs/CHANGELOG.md`, `docs/STATUS.md`.

## Phase 2 milestone 17: economy investigated, `civ_resource_trade_balance` ported (2026-08-17)

Investigated "economy" and the "Journey Planner" for real, both repeatedly
named but never read — full reasoning in `ECONOMY_SCOPE.md` (repo root).
Found two separate, both genuinely large subsystems, not one: the Journey
Planner (`jp*`/`_jp*`, reference lines ~17300-20400, ~70 functions covering
transport-mode selection, physical travel cost, consumption/resupply,
seasonal closures, multi-stage route derivation) confirms `ROADMAP.md`'s own
"consider it a sub-phase" warning as accurate — comparable in size to this
port's entire civ-layer effort to date, not attempted here. The
faction/settlement economy layer (`_civFactionAggregates`, ~165 lines,
reference line 23575; `_civPlaceTrade` and its dependency cluster,
reference line 24459) is smaller but still real — a `Vec` output feeding
Factions/Settlements/Economy/Statistics display pages, explicitly
"NOT new simulation" per the reference's own header comment.

**Ported**: `civ_resource_trade_balance` (`cartalith-civ`) — direct port of
`_civResourceTradeBalance` (reference line 24175, v1.33's unification of two
drifted copies). Given a settlement's or faction's own catchment-mean
resource values and the world mean for the same 15 `CIV_RESOURCE_KEYS`,
classifies each as an export (well above world average, past both a ratio
and an absolute floor) or an import (well below average, and only for
resources in `CIV_CONSUMED_RESOURCES` — a locally-scarce resource nobody
consumes, like `gems`, is never an import). Fully self-contained: operates
on caller-supplied means, no new upstream field dependency.

Kept a real JS-parity subtlety clippy flagged: the reference's
`!(world>0.002)` (not `world<=0.002`) matters for NaN inputs — `!(NaN>x)` is
`true` in both languages, `NaN<=x` is `false` only in Rust. Kept the
JS-matching form with a documented `#[allow(clippy::neg_cmp_op_on_partial_ord)]`
rather than silently changing NaN behavior to satisfy the lint.

Chose real unit tests over a Node-harness golden extraction for this one
function specifically — small, pure, branch-complete, no RNG/iteration-order
to get subtly wrong, the same category `PARITY_TESTING.md`'s own discipline
treats real unit tests as a legitimate stand-in for (same precedent as
milestone 10's territory / the provinces work). Seven tests: empty inputs,
the absent-worldwide branch's absolute floor, ratio-clears-but-floor-fails
(a real branch-order case), a genuine export, import correctly gated to
`CONSUMED_RESOURCES` only, missing-key-as-zero fallback, full 15-key order.

**Real tension found, not resolved**: the full trade layer's own resource-mean
aggregation needs all 15 `CIV_RESOURCE_KEYS` resident, but this session's own
memory-optimization pass (commit `62b9b51`) frees 6 of them
(clay/buildstone/flint/obsidian/sulfur/alum) right after
`build_resource_potentials` returns, since nothing consumed them at the time.
Flagged in `ECONOMY_SCOPE.md` for whoever ports the full orchestration next —
not silently reverted here.

**Not wired anywhere** — `civ_resource_trade_balance` has no real caller yet
in this port (the broader `_civPlaceTrade` orchestration that would call it
doesn't exist), so it isn't exposed to Godot. Wiring a function nothing calls
into the API surface would repeat the exact "technically done, practically
inert" trap this document's own milestone 9 note already flagged once for
territory before provinces gave it a real caller.

**Verified**: `cargo test -p cartalith-civ --lib trade_balance` (7 new, all
passing), `cargo test --workspace` (0 regressions), `cargo clippy -p
cartalith-civ --all-targets` clean.

**Files touched**: `cartalith-civ/src/lib.rs` (`civ_resource_trade_balance`,
`CIV_RESOURCE_KEYS`, `CIV_CONSUMED_RESOURCES`, `TradeBalance`, 7 tests), new
`ECONOMY_SCOPE.md` (repo root), `PHASE2_SCOPE.md`, `docs/STATUS.md`.

## Sea level as a user-adjustable Godot control (2026-08-17)

Closed a known-open item from the UI reskin (`MVP_SCOPE.md` point 9 /
reference `state.seaLevel`): `cartalith-engine::WorldParams.sea_level` was
already a real `[0,1]` parameter (default `0.42`, matching the reference's
own default), but nothing in `cartalith-godot`/the Godot UI ever set it --
every generation silently used the hardcoded default.

**Rust side**: `WorldGen` gained a `sea_level_input: f64` field (distinct
from the existing `sea_level` field, which tracks the *effective*
post-generation value the renderer reads -- `WorldState.sea_level`, not
`p.sea_level`, since World-Structure archetypes re-anchor it) and a new
`set_sea_level(&mut self, sea_level: f64)` `#[func]`, clamped to `[0,1]`.
Wired into both `generate()` and `generate_world_structure()` via
`p.sea_level = self.sea_level_input`.

**Real, documented interaction, not a new limitation**: `generate_world_
structure()` always sets `world_structure.enabled = true`, and
`apply_world_structure_sea_level` unconditionally re-anchors sea level from
the selected archetype's own land-fraction target when enabled --
overriding whatever `p.sea_level` was set to. The manual sea-level input
therefore only has a real effect under the Classic world shape (`generate()`,
no archetype). A new `SeaLevelHint` label in `main.tscn` says so plainly,
matching this UI's existing hint-label convention (`ResolutionHint`).

**GDScript side**: new `Sea level` `SpinBox` in the `WORLD PARAMETERS` card
(0-100%, matching the reference's own `#seaV` slider convention -- `bind(
'sea', e => state.seaLevel = +e.target.value/100 ...)`), `main.gd` converts
to the `[0,1]` fraction `set_sea_level` expects before calling it alongside
the existing `set_experimental_flags`/`set_villages_enabled` calls.

**Screenshot-verified, not just wired**: seed 12345, 512x512, Classic. At
the default 42% sea level, the generated map showed roughly half ocean/half
land with the usual coastline shape. Changing only the sea-level input to
15% and regenerating with the same seed produced a dramatically different
result -- most of the ocean became land, only a small lake/river feature
remained, and settlement positions shifted accordingly (they snap to real
land/coast, which moved). Confirms the control has a real, substantial
effect on generation, not just a cosmetic one.

**Real pre-existing bug found while tracing this, not fixed here (out of
scope for this task, flagged in `docs/STATUS.md`'s Known-open items)**:
`main.gd`'s `_on_generate_pressed` calls `generate()`/`generate_world_
structure()` synchronously on the main thread first (wasted, blocking work
-- the result is thrown away), then `_generate_worker` (whose result is
what's actually displayed) unconditionally calls plain `generate()`, never
`generate_world_structure()`, regardless of the selected World Shape.
**Every World Shape archetype selection currently has no effect on the
displayed map** -- generation always runs the Classic path. This did not
block verifying sea level (Classic is exactly the path sea level's manual
input takes effect on, per the interaction above), but is a real,
independent bug a future pass needs to fix.

**Verified**: `cargo build -p cartalith-godot`, `cargo test --workspace`
(0 regressions; one `cartalith-engine` GPU-determinism test failed once
under parallel scheduling, reproduced and confirmed as the already-
documented pre-existing GPU-driver flakiness by re-running in isolation
with `--test-threads=1`, where it passed clean -- unrelated to this
change, which touches no GPU code), `godot4 --headless --quit main.tscn`
clean load, real windowed-app screenshot verification as described above.

**Files touched**: `cartalith-godot/src/lib.rs` (`sea_level_input` field,
`set_sea_level`, wiring into both generate paths), `main.tscn` (`SeaLevelRow`/
`SeaLevelInput`/`SeaLevelHint`), `main.gd` (`sea_level_input` ref, wiring
into `_on_generate_pressed`), `docs/STATUS.md` (item closed, new bug
flagged), this file.

## Fix: World Shape archetype selection had no effect on generation (2026-08-17)

The bug the sea-level pass above flagged (not fixed there): every World
Shape dropdown selection (Earth-like/Supercontinent/Archipelago/Volcanic/
Rift) silently generated the Classic map regardless. Independently
re-confirmed before fixing, per this project's own discipline -- read both
Rust entry points (`WorldGen::generate`/`generate_world_structure` in
`cartalith-godot/src/lib.rs`) and found the real mechanism is worse than
"never called": `_on_generate_pressed` *did* call `generate_world_structure()`
when an archetype was selected, but **synchronously on the main thread**
(freezing the UI for the full generation, defeating the whole point of the
background-thread design this file's own doc comment describes), and then
`_generate_worker` (started immediately after, on the background thread)
**unconditionally re-ran plain `generate()`**, overwriting `world_gen`'s
state with the archetype-free result before `_on_generate_done` ever read
it. Both entry points are equally expensive full `generate_terrain()`
calls mutating the same `self` state on the Rust side -- confirmed by
reading `generate()`/`generate_world_structure()` side by side.

**Fix**: moved the `archetype.is_empty() ? generate() : generate_world_
structure()` branch into `_generate_worker` itself, making it the one and
only call site, on the background thread where it belongs. `archetype` is
now bound into the worker via `Thread.start(...).bind(...)`; `_on_generate_
done` gained an `ok: bool` parameter (previously checked before the thread
even started) to still handle the defensive "unknown archetype string"
case, now surfaced after the worker completes instead of before it starts.

**Real before/after screenshot proof, not just a code-path claim**: same
seed (12345), same resolution (512x512), same map width (800km) --
Classic produced one large landmass, 40 settlements; Archipelago produced
scattered small islands across mostly open ocean, 33 settlements.
Dramatically different structure, not just different noise detail --
exactly the world-structure archetype's own land-fraction/fragmentation
parameters taking real effect for the first time. Before this fix, both
screenshots would have been byte-identical regardless of World Shape
selection.

**Verified**: `cargo build -p cartalith-godot` clean (no Rust changes
needed -- this was purely a GDScript dispatch bug), `cargo test --workspace`
(71+ suites, 0 failures, 0 modified tests), `godot4 --headless --quit
main.tscn` clean load, real windowed-app screenshot verification with the
Classic-vs-Archipelago comparison described above.

**Files touched**: `main.gd` (`_on_generate_pressed`/`_generate_worker`/
`_on_generate_done`), `docs/STATUS.md` (Known-open item closed), this file.

## Phase 3 milestone 1: `TerrainAppearance` abstraction in the renderer (2026-08-17)

Turns `TERRAIN_APPEARANCE_RESEARCH.md` (owner-supplied) into Phase 3's first
real milestone (`TERRAIN_APPEARANCE_SCOPE.md`) — a behavior-preserving
refactor of `crates/cartalith-godot/src/render.rs`'s colour logic, no
visual change.

**Real audit finding, correcting the milestone's own initial assumption**:
`render.rs` has no elevation-keyed colour *breakpoint ramp* anywhere,
despite the research doc's MapTiler-style mental model (`0m → green, 300m
→ yellow-green, ...`). Colour comes from `material_weights()` — a
continuous blend over temperature/moisture/slope/relative-elevation/
aspect/curvature producing six material fractions, each contributing
colour via a noise-jittered 3-stop micro-ramp (`ramp3`, selected by a
per-pixel texture-variety value from coherent noise, not by elevation).
Relative elevation is one continuous input among several, not a lookup
axis — so there were no "hardcoded elevation bands" to re-encode as a
ramp, contrary to the milestone's own original plan.

**Built instead, the honest version**: the 25 material/water 3-stop
palettes (`W_ABYSS`...`MANGROVE`) plus `EXAG`/`SUN_AZ_DEG`/`BIO_BLEND` —
previously 26 free module-level consts — are now one owned
`TerrainAppearance` struct with a `Default` impl reproducing every value
exactly. Threaded through every colour-selector function
(`grass_col`/`forest_col`/`sand_col`/`rock_col`/`snow_col`/`wetland_col`/
`sea_color_core`/`land_color`/`sea_shade_from`/`RenderCtx::shade`), all
previously reading bare consts, now reading `&TerrainAppearance`.
`RenderCtx` owns one `TerrainAppearance` (built via `Default` inside
`RenderCtx::new`), so `RenderCtx::new`'s and `cell_color`'s public
signatures — and therefore `golden_parity_render.rs` — needed **zero
modification**. Not wired to any UI/`#[func]` yet, matching
`cartalith-spatial`'s own "standalone but real" precedent.

**Verified**: `golden_parity_render.rs`'s two tests
(`cell_color_matches_js_surface_and_sea`, `cell_color_matches_js_world_wrap`)
pass byte-identical, test file completely unmodified — the headline check
for a pure refactor. `cargo build -p cartalith-godot` clean, `cargo clippy
-p cartalith-godot --all-targets` clean (no new warnings), `cargo test
--workspace` 0 regressions, `godot4 --headless --quit` clean load. Real
windowed-app screenshot (seed 12345, Classic, 2048², 40 settlements)
confirms correct rendering — biome colours, hillshade, settlements, roads,
sea routes all visible, matching this session's prior screenshots at the
same settings.

**Files touched**: `crates/cartalith-godot/src/render.rs`,
`TERRAIN_APPEARANCE_SCOPE.md` (milestone 1 marked done with the real
finding), `docs/STATUS.md`, this file.

## Fix: province boundary lines were illegible at normal zoom (2026-08-17)

`build_province_boundary_texture()` (commit `f1afafb`) was functionally
correct — real, verified per-cell data — but real screenshot testing found
its output illegible: a literal single-cell-wide line drawn into a
full-grid-resolution (e.g. 2048×2048) RGBA texture becomes sub-pixel once
`TextureRect` downscales it to fit a typical viewport width, anti-aliasing
it into a faint smudge indistinguishable from roads or coastline. This
was a rendering-resolution problem, not a data problem — the boundary
data itself was already correct.

**Fix, in `cartalith-godot`'s `build_province_boundary_texture()` only**
(`map_overlay.gd` has no province-drawing code at all — provinces render
as a texture overlay, same mechanism as territory, not custom `_draw()`):
two-pass approach — pass 1 detects boundaries symmetrically (checks all
four neighbours, not just +x/+y, so a boundary is a property of the edge
rather than of scan order), pass 2 dilates by one cell (3×3 neighbourhood)
for a real ~3px stroke at source resolution instead of 1px. Alpha nudged
modestly (200→235 of 255) — not to fully opaque, since the goal is a
legible subdivision line, not a competing top-level feature.

**Real before/after screenshot** (seed 12345, Classic, 512×512, 40
settlements, both territory and province layers enabled): boundaries now
read as clean, bold lines cleanly subdividing each faction's territory
(visible splitting the orange and teal/green factions into two provinces
each), clearly distinct from roads' thin brown lines, at normal
(non-zoomed, non-cropped) view — the exact case the original fix's
screenshot testing found illegible.

**Verified**: `cargo build -p cartalith-godot` clean, `cargo test
--workspace` 0 regressions (91 test binaries, all green), `godot4
--headless --quit main.tscn` clean load.

**Files touched**: `crates/cartalith-godot/src/lib.rs`
(`build_province_boundary_texture`), `docs/STATUS.md`, this file.

## GUI shell redesign milestone 1: full 6-region professional-editor shell (2026-08-17)

Owner-supplied design import (`claude_design` MCP, project "UI mockups
planning") — `design/Cartalith GUI.dc.html` (multi-breakpoint/theme mockup)
and `design/cartalith-menu-structure.md` (the handoff spec). Two owner
decisions grounded this pass, both recorded in `GUI_SHELL_SCOPE.md`: target
this Godot port (the menu-structure doc's own "re-parent existing `#id`s"
notes describe the JS reference app's DOM, a different, frozen file this
repo's own `CLAUDE.md` forbids editing, in a different repository); and
build the full shell structure now, wiring only what has real engine
backing, leaving everything else visibly present but honestly inert.

Rebuilt `main.tscn`/`main.gd` from the prior single-panel MVP layout into
the mockup's full structure: a top bar with 7 domain menus (Project/World/
Generate/Simulate/Map/Assets/View), a 4-group workspace navigator (World/
Civilization/Infrastructure/Cartography, ~20 subject rows generated from a
data table, not hand-authored per-node), a second panel that swaps content
with the navigator selection, a mode bar (WORLD/EDIT/ANALYSIS/SIMULATION/
CARTOGRAPHIC/DEBUG — only WORLD active, the rest real but `disabled`), the
existing map viewport, a right context inspector (new), and a bottom
timeline bar with transport/speed/simulation-layer controls (all real nodes,
all `disabled` — no time-simulation engine exists to drive them).

**Zero Rust changes** — confirmed by `cargo build -p cartalith-godot`
needing no new `#[func]`s; every real control (seed/resolution/width/sea
level/world shape/the four experimental flags/villages/the three
map-overlay toggles/load-save/credits) was re-parented, not rewritten.
Godot's `%UniqueName` lookup resolves by name regardless of tree position,
so `main.gd`'s existing `@onready var x = %Name` references needed no
changes for any of these — the whole rewrite stayed mechanical for the
working parts.

**Real feature-inventory corrections** (verified against the actual
`cartalith-godot` `#[func]` list, not assumed from the mockup): no live
CPU/GPU/memory readout exists — the top-bar readout shows real generation
status instead of a fabricated number; no per-cell inspector query
(elevation/slope/etc. at an arbitrary cursor position) exists — the
Inspector's "no selection" state says so honestly rather than being built
against fields that don't exist. What *does* have real backing: settlement
hover data, already computed by `map_overlay.gd` for its own on-canvas
hover card — added a new `settlement_hovered` signal so the Inspector panel
shows the same real data (name/population/faction/coastal/capital) without
duplicating the hit-test logic.

**Judgment calls, disclosed rather than silently made**: menu items with
real backing are actions (Generate World, New seed, Open project, Credits),
not live-editable parameters — `PopupMenu` doesn't support embedded
SpinBox/slider controls well, so parameter editing stays in the
navigator-driven second panel. The menu-structure doc's full multi-hundred-
item inventory (individual generation-stage sliders, most of which don't
exist as separate Rust tunables beyond the 4 experimental flags already
exposed) was populated representatively, not transcribed exhaustively — the
shell *structure* was this milestone's goal, not every leaf item. Panel
widths deviate from the mockup's exact 206/238/272px where the existing
cards needed more room to stay readable (360px second panel). The prior
`Stage`/`ControlsPanel` width-based responsive fallback was removed, not
preserved — the new 5-region layout has no structural equivalent, and a
real responsive redesign is its own deferred milestone; narrow windows will
look cramped, not stacked, until then. The new shell chrome uses inline
dark `StyleBoxFlat` styling (not a new Theme resource); re-parented input
controls still render with `app_theme.tres`'s light-parchment chrome on the
new dark background — a real, known, flagged visual seam pending the
deferred light/dark theme-toggle milestone.

**Verified**: `cargo build -p cartalith-godot` clean, `cargo test
--workspace` 0 failures. `godot4 --headless --quit main.tscn` clean load.
Real windowed-app screenshot verification, end-to-end, through the new
shell (this session's established `PrintWindow`/`mouse_event`/minimize-
restore-focus technique): generation (seed 12345, Classic, 2048²) completed
and rendered correctly with real terrain/settlements/roads/sea routes;
top-bar readout and status label both updated with real data; the
`CARTOGRAPHY > Layers` navigator swap correctly showed the three real
overlay toggles; hovering a settlement updated the new Inspector panel with
real data, confirmed against the existing on-canvas hover card showing the
same settlement; the Credits dialog opened and rendered correctly.

**Deferred, exactly as scoped**: light theme, panel collapse/rails, all
three responsive breakpoints, terrain appearance's actual editing GUI.

**Files touched**: `godot-project/main.tscn` (full rebuild),
`godot-project/main.gd` (menu/navigator/inspector logic added, old
`Stage`/`ControlsPanel` responsive code removed), `godot-project/
map_overlay.gd` (`settlement_hovered` signal), `GUI_SHELL_SCOPE.md`,
`docs/STATUS.md`, this file. New `design/` directory holds the imported
mockup HTML and menu-structure spec verbatim.

## Phase 2's remaining outstanding points: economy wired, culture investigated, Journey Planner started (2026-08-17)

Three-part continuation of milestone 17's economy investigation, all real
findings from reading the actual reference source, not assumed.

**Economy wiring, and the memory tension resolved.** `civ_resource_trade_
balance` (ported last pass, unwired) now has real callers.
`_civFactionAggregates`/`_civPlaceResourceContext` were grepped directly to
confirm the earlier "needs all 15 `CIV_RESOURCE_KEYS`" tension is real, not
assumable-away — both genuinely read every key. A compounding finding not in
the original write-up: `_civFactionAggregates`'s per-faction resource-mean
approach also needs `territory`, which isn't computed until much later in
`compute_civilisation()`, so a simple reorder wouldn't have worked. The fix
used `_civPlaceTrade`'s own settlement-catchment approach instead
(`_civPlaceResourceContext`, reference line 24567 — a fixed-radius disc scan
around a settlement, no territory needed). New in `cartalith-civ`:
`civ_world_mean_resources` (the one territory-independent piece of
`_civFactionAggregates`, extracted standalone since `_civPlaceTrade`'s own
`worldMean` argument reuses this exact value per the reference),
`civ_catchment_km2`/`civ_catchment_radius_cells` (`_CIV_CATCHMENT_KM2`/
`_civCatchmentRadiusCells`, reference lines 23407/23481), `civ_place_
resource_context` (the disc scan itself). 8 new tests, including one
verifying the disc-scan rejection/world-wrap/ocean-exclusion behavior
directly, not just the happy path. `compute_civilisation()`'s free of the
six otherwise-unused resource fields moved from immediately after
`build_resource_potentials` to right after settlements are finalized (before
`territory`) — the trade-balance computation runs in between, needing the
full 15-key vocabulary. Real, bounded, measured tradeoff: these six fields
(~96 MB at 2048×2048) now stay resident through settlement placement/roads/
naming instead of being freed immediately, but steady-state after
`compute_civilisation()` returns is unaffected either way. New
`get_trade_balances()` `#[func]` in `cartalith-godot`, same order/index as
`get_settlements()`, `exports`/`imports` as `PackedStringArray`s.

**Culture beyond naming, investigated for real.** Grepped the reference for
every culture-related computation beyond the already-ported syllable/suffix
naming tables (milestone 9). Confirmed Government/Religion/Ag-technology are
genuinely UI-only categorical pills with zero derived computation (the
reference's own v1.57 comment says so directly). But one real thing exists:
`_civCultureTerrainFit` (reference line 23748, v1.55) — does a faction's
territory terrain-mix match what its culture is thematically associated with
(highland↔hills, desert↔arid, riverlands↔river, sylvan↔forest,
maritime↔coast), a match/typical/mismatch verdict relative to the world
mean. `common`/`imperial` (identity-flavored) deliberately get no verdict.
Ported as `civ_culture_terrain_fit`, 7 tests covering every verdict band and
both zero-world-mean edge cases (a genuinely-present resource with an absent
world baseline reads as a fabricated "match" per the reference's own
branch; a genuinely-absent resource with an absent baseline reads as
"typical," not "match" — the two zero-mean cases the reference's own ratio
formula deliberately treats differently). Not wired — its real inputs
(per-faction terrain-mix fractions) are part of the still-unstarted
`_civFactionAggregates` territory aggregation. Also found and correctly
excluded: an entirely unrelated, much larger "culture profiles" system at
reference lines 28193+ (`docs/07-culture-architecture.md`, urban-morphology
city-layout patterns — Organic Growth, Islamic, Byzantine, etc.) — this
belongs to `ROADMAP.md` Phase 5 (Urban Morphology, block 4, not started),
not Phase 2.

**Journey Planner, milestone 1 of a new multi-milestone plan
(`JOURNEY_PLANNER_SCOPE.md`).** Read the real reference source (lines
~17300-20400) to find the two categories of its ~70 functions that need no
route/plan/vessel context object at all: physical-modeling primitives
(`jp_fatigue`, `jp_load_penalty` plus `JP_LOAD_INVALID_RATIO`'s v1.63
infeasible-stage fix, `jp_surface_gain`, `jp_can_use_wheels`) and the
reference's own "v1.52: four items each deferred across three versions"
cluster — `jp_season_at` (season drift over long journeys), `jp_rest_days`
(travel-day/calendar-day split, the Andean-caravan-ethnoarchaeology-sourced
rest cadence), `jp_seasonal_closure` (mountain passes shut in Winter),
`jp_sea_closure` (the *Mare Clausum* analogue — open-water shipping closes
in Winter, coastal cabotage doesn't). All four closure/scheduling functions
are real, historically-sourced fixes per the reference's own extensive
comments, ported faithfully. 22 tests. Not wired — the real route/plan
orchestration (`JOURNEY_PLANNER_SCOPE.md`'s milestones 2-6) remains large,
real, unstarted future work; milestone 5 (route/stage derivation) alone is
flagged as likely the single largest remaining milestone in this whole plan.

**Verified**: `cargo build -p cartalith-civ -p cartalith-godot`,
`cargo test --workspace` (70 suites, 0 failures, 0 modified tests — the
headline check for both the economy-wiring refactor and every new pure
function), `cargo clippy -p cartalith-civ -p cartalith-godot` clean (two
new deliberate `#[allow(clippy::neg_cmp_op_on_partial_ord)]`s, matching
`civ_resource_trade_balance`'s own established NaN-preserving precedent —
not lint-driven rewrites), `godot4 --headless --quit` clean load.

**Files touched**: `cartalith-native/crates/cartalith-civ/src/lib.rs` (new
functions + 37 new tests), `cartalith-native/crates/cartalith-godot/src/
lib.rs` (`trade_balances` field, `get_trade_balances()`, the resource-free
reordering), `ECONOMY_SCOPE.md`, `PHASE2_SCOPE.md`, new
`JOURNEY_PLANNER_SCOPE.md`, `docs/STATUS.md`, this file.

## Causal-chain explainer: "why is this settlement here?" (2026-08-17)

`VISION.md`'s sequencing item 1, and the one idea in the owner's vision
render that this engine was already positioned to answer honestly. The
render annotates the map with chains like `mountain range -> watershed ->
river -> fertile valley -> settlements -> road network -> trade corridor ->
political importance`. Those aren't decorative: every link is a field this
port already computes and golden-verifies. This makes that derivable
causality visible.

**What it decomposes.** `build_settlement_suitability` *is* the answer to
"why here?" -- it sums thirteen weighted terms (carrying capacity, water
access, gentle slope, terrain form, coastal access, river, lake, minerals,
route corridor, farmland, buildable ground, minus flood risk and islet
isolation) and squashes the result through a sigmoid. New
`cartalith_civ::explain_settlement_suitability` returns that same sum
broken into its parts: each term's raw `value`, its `weight` (signed --
the two penalties are genuinely negative), and its `contribution`
(`weight * value`), sorted most-decisive-first, plus the pre-sigmoid `z`
and the two real early-return exclusions (`below_sea_level`, `water_body`).

**It is provably the real arithmetic, not a lookalike.**
`explanation_reconstructs_real_suitability` runs both functions over an
entire synthetic field and asserts the explainer's `score` equals
`build_settlement_suitability`'s output at *every* cell, and that each
cell's terms sum to its own `z`. A second test does the same for the
no-context weight set. Editing one function's arithmetic without the other
fails the build -- which is the point of having it. (7 new tests total,
also covering exclusion reasons, sort order, penalty signs, and that the
mineral term really does ignore the six non-ore resources.)

**A real design correction, made deliberately.** The obvious API would be
`explain_cell(x, y)` for an arbitrary cell. That is not what shipped, for a
measured reason: every raster the decomposition needs (soil, water access,
carrying capacity, coast SDF, river order, flow, corridor, landmass, flood,
slope, resources) is a local of `compute_civilisation` and dies at its end.
`CivData` retains none of them. Answering for arbitrary cells later would
mean holding all twelve -- hundreds of MB at 2048x2048, straight back into
what `MEMORY_OPTIMIZATION_SCOPE.md` spent real measurement escaping. So the
explanation is computed per-settlement, inside that function, while the
rasters are alive: ~40 records instead of ~4.2M cells, covering the question
actually being asked. `WorldGen.explain_settlement(index)` is keyed by
settlement index accordingly.

**Verified against the terrain, not just against itself.** A temporary
headless script (not committed) generated a real 512x512 world and checked
all 40 settlements: every one of the 10 sitting within 5 cells of water
carried a coastal bonus, every one of the 30 beyond it carried none (the
coast term is a 5-cell falloff), 29 carried a river term consistent with
their Strahler order, and no settlement violated either relation. Real
windowed-app hover screenshots confirmed the Inspector renders it live.

**Real finding surfaced by that cross-check**: a settlement can honestly
read `Coastal: yes` while earning *zero* coastal bonus. Two different,
deliberately different notions exist in the reference -- the `coastal` flag
uses a `max(6, GW/60)`-cell radius (34 cells at 2048, port eligibility),
while the suitability bonus uses a 5-cell falloff. Not a bug in either;
the Inspector now says "Distance to water" rather than "Coast" so the two
lines stop reading as a contradiction.

**Wording lives in GDScript, facts in Rust.** The `#[func]` returns numbers
and stable keys only; `main.gd` owns the phrasing, and qualifies each term
by its own raw reading ("weak farmland (0.31)", not a flattering label) so
a settlement placed on mediocre ground reads as such. `map_overlay.gd`'s
`settlement_hovered` signal gained the settlement index so the Inspector can
ask without re-running any hit test.

**Verified**: `cargo build -p cartalith-godot`, `cargo test --workspace`
(70 suites, 0 failures), `cargo clippy -p cartalith-civ -p cartalith-godot
--all-targets` clean, `godot4 --headless --quit` clean load, plus the
headless terrain cross-check and windowed screenshots above.

**Files touched**: `cartalith-native/crates/cartalith-civ/src/lib.rs`
(`SuitTerm`/`SuitExplanation`/`explain_settlement_suitability` + 7 tests),
`cartalith-native/crates/cartalith-godot/src/lib.rs`
(`SettlementExplanation`, `CivData.explanations`, `explain_settlement()`),
`cartalith-native/godot-project/main.gd` (Inspector "WHY HERE?" section),
`cartalith-native/godot-project/map_overlay.gd` (index on the hover
signal), `VISION.md`, `docs/STATUS.md`, this file.

## Phase 3 milestone 2: multidirectional hillshade + ambient occlusion (2026-08-17)

`TERRAIN_APPEARANCE_SCOPE.md` milestone 2. Milestone 1 was zero-visual-change
groundwork; this is the pass where the default render actually gets better.
Prompted by the owner's vision render (`VISION.md`), whose own gap assessment
names atlas-quality rendering the largest purely-visual gap between today and
the target.

**Two improvements, chosen because they touch only the lighting term.**
`TERRAIN_APPEARANCE_RESEARCH.md` lists 15 phases; this did §14
(multidirectional hillshade) and §15 (ambient occlusion) properly instead of
four badly. Neither touches `material_weights` or the 25 palettes — the part
golden-verified against JS, and the part §32 warns is easiest to improve for
one terrain type while wrecking another.

They're also complementary, which is why either alone would have been worse:
multi-light reveals ridgelines running *parallel* to the single NW sun
(structurally invisible under one light), but adding lights lifts shadows and
flattens depth — the classic multidirectional failure mode. AO puts that
depth back from the terrain's own concavity instead of from light direction.

- **Multidirectional hillshade** — `shade` computes the surface normal once
  and dots it against a precomputed weighted light table (6 lights evenly
  spaced from `sun_az_deg`, weight `((1+cos θ)/2)^p`, primary NW sun still
  dominant at 43%). Each light is clamped at the horizon *before* weighting.
  The light curve's ambient floor and gain became parameters
  (`relief_ambient`/`relief_gain`) because multi-light compresses the shade
  range upward and the reference's `0.45` floor would wash the image out.
- **Ambient occlusion** — `build_ao`, a two-scale cavity map: compare each
  cell against a blurred copy of the heightfield; sitting below the local
  mean means sitting in a hollow. Uses the box blur already in this file.
  `ao` had been a hardcoded `1.0` in `land_color` since the renderer landed.

**The AO normalization is what makes it survive §32.** Each scale is
normalized by its own RMS *over land cells only*, so occlusion is measured
against each world's own relief statistics. A fixed threshold would give a
low-relief world no AO and crush an alpine one — precisely the failure §32
names. Pure function of the heightfield, so §27 determinism holds.

**Golden parity kept exact — not re-baselined, not loosened.** New
`TerrainAppearance::js_reference()` reproduces the pre-milestone renderer
bit-for-bit: `relief_lights: 1` takes a dedicated early-return branch in
`shade` (so parity can't drift on a float reassociation) and
`ao_strength: 0.0` skips the AO precompute, leaving the `1.0` the code
previously hardcoded. `golden_parity_render.rs` now builds its context via
the new `RenderCtx::with_appearance(..., js_reference())` — **both tests pass
at their original `1e-4` tolerance with every expected value unchanged**; the
only edit is which appearance the context uses.

This reading of `DECISIONS.md` §7a is deliberate: §7a's carve-out is scoped to
paths where JS parity is *impractical* (GPU/`f32`/`naga`), and it states that
the CPU rendering port "stays golden-verified against the JS engine and that
work is not being discarded or devalued". A deliberate visual improvement
isn't an impractical one, so the reference path stays tested — which also
satisfies research doc §1.5's "preserve the current renderer as a
fallback/reference implementation" literally.

**New A/B harness** — `tests/appearance_ab_dump.rs` (`#[ignore]`d, run with
`--ignored`) renders one generated world through both appearances and dumps
raw RGB for Classic and Archipelago. Research doc §1.6's "deterministic A/B
comparison rendering"; it exists because app screenshots can't isolate the
renderer from the rest of the app.

**Real before/after**, from both the deterministic dump and the real windowed
app (2048², seed 12345, Classic, 40 settlements, identical params both runs):
drainage networks, ridge/valley structure and coastal escarpments become
legible where the single-sun render was a flat tan wash. Measured against
§30's anti-list rather than eyeballed:

| | Classic before | Classic after | Archipelago before | Archipelago after |
|---|---|---|---|---|
| min luma | 39.4 | **39.4** | 31.6 | **31.6** |
| mean luma | 133.3 | 128.8 | 108.7 | 108.0 |

Identical minima prove no new darkest pixel — no black valleys (AO darkens
concavities only, floored at `1 - ao_strength`). Mean luma barely moves, so
contrast is redistributed rather than the image dimmed. Archipelago is the
§32 case: the low-relief world gains definition without being crushed or
going monochromatic.

**One real regression caught by looking, not by reading.** A 3× zoom of the
dump showed speckle on flat plains — the fine AO radius resolved to 1 cell at
512², close enough to the raw field that the cavity signal picked up per-cell
heightfield noise ("random texture noise", also on §30's anti-list). Floored
both radii (`r_fine = (r_broad/3).max(2)`) and re-verified.

**Cost is essentially nil**: 512² render 45→45 ms (Classic), 20→19 ms
(Archipelago). The normal is computed once and reused across all six lights,
so multi-light adds only dot products; AO is a one-time O(n) separable blur
plus a per-pixel lookup.

**Verified**: `cargo build -p cartalith-godot` clean; `cargo test --workspace`
71 suites, 0 failures, 0 modified expectations; `cargo clippy -p
cartalith-godot --all-targets` clean for this milestone's files (the one
remaining warning is a pre-existing `needless_borrow` in `lib.rs`); `godot4
--headless --quit main.tscn` clean.

**Files touched**: `cartalith-native/crates/cartalith-godot/src/render.rs`,
`cartalith-native/crates/cartalith-godot/tests/golden_parity_render.rs` (two
constructor calls only), new
`cartalith-native/crates/cartalith-godot/tests/appearance_ab_dump.rs`,
`TERRAIN_APPEARANCE_SCOPE.md`, `VISION.md`, `docs/STATUS.md`, this file.

## GUI shell cleanup: remove top-bar/navigator menu duplication (2026-08-17)

Owner-flagged directly after using the shell: *"There should be no double
menus in the upper bar that are present in the left [nav]."* Audited every
top-bar menu item (`main.gd`'s `_build_menus()`) against the navigator's
`NAV_GROUPS` inventory for real duplication (same label *and* destination),
not superficial word overlap between conceptually distinct surfaces
(`design/cartalith-menu-structure.md`'s own rule: menus hold operations,
the navigator holds subjects).

Found one real, flagrant duplicate: the Map menu's "Layers" item did
nothing but jump to `CARTOGRAPHY > Layers` — the exact same panel the nav's
own "Layers" subject already opens, identical label, identical destination,
zero distinct content. Removed the item and the now-dead
`_on_map_menu_id` handler (the Map menu's remaining three items are all
`disabled`, so nothing was left listening for a click that could never
fire).

Considered and deliberately left alone: the top-bar "Assets" domain menu
vs. the CARTOGRAPHY nav's "Assets" subject (both inert placeholders
representing genuinely different real-design scopes — global asset-library
management vs. per-map asset usage — and removing either would trade real
mockup fidelity for a speculative fix with no concrete content yet on
either side to disambiguate against), and the Generate menu's numbered
pipeline-stage items that share a bare word with unrelated nav subjects
("08 Ecology", "09 Settlements", "11 Politics") — an ordered process list
reads as a genuinely different kind of thing from a subject browser, not a
copy of it.

**Verified**: `cargo build -p cartalith-godot` clean (0 new Rust, pure
GDScript), `cargo test --workspace` unaffected, `godot4 --headless --quit`
clean load. Real windowed-app screenshot verification, maximized
(1696×1018): confirmed the Map menu shows only its three real items with
the nav's own unduplicated "Layers" below it; re-ran the full golden path
(seed 12345, Classic, 2048², Generate → real terrain/settlements/roads/sea
routes) and the causal-chain Inspector (hover → real "WHY HERE?" chain)
both still work correctly through the cleaned-up shell; the Layers panel
(now the sole entry point for that content) still functions for all three
overlay toggles.

**Files touched**: `cartalith-native/godot-project/main.gd`,
`GUI_SHELL_SCOPE.md`, `docs/STATUS.md`, this file.

## App icon wired for Windows and Android (2026-08-17)

Owner supplied a real icon design (`design/app-icon.png` — the layered-map
motif from `VISION.md`'s own "physical sheets" idea, with a "C" wordmark)
and asked for it wired into both platform build targets, not just dropped in
as a file.

Generated the full real asset set from the one 1254×1254 source (Pillow,
installed fresh for this task — no other workspace tooling did image
processing before now): bbox-cropped to the actual opaque content, re-centred
on a square canvas so every derived size shares one consistent frame.

- `godot-project/icon.png` (256×256) — the project/editor icon, wired via a
  new `config/icon` in `project.godot` (previously unset, falling back to
  Godot's own default logo).
- `godot-project/icon.ico` (16/32/48/64/128/256, multi-resolution) — Windows
  export's `application/icon` and `application/console_wrapper_icon`
  (`export_presets.cfg`, both previously empty strings).
- Android launcher set (`godot-project/icons/`): `android_main_192.png`
  (legacy icon, content at ~88% of frame — safety margin for launchers that
  still circle-crop), `android_adaptive_foreground_432.png` (content scaled
  to ~62% of the 432px canvas — Android's adaptive-icon mask safe zone,
  roughly the inner 66%; full-bleed content would get clipped by circular/
  squircle/rounded-square launcher masks), `android_adaptive_background_432.png`
  (flat fill — sampled the icon's own dominant colour by histogramming the
  bottom third of the source rather than a single-pixel guess, which first
  landed on an anti-aliased edge pixel; the real base-plate tone is
  `rgb(0,48,96)`), `android_adaptive_monochrome_432.png` (white silhouette
  from the foreground's own alpha channel, for Android 13+ themed icons).
  Wired into `export_presets.cfg`'s four `launcher_icons/*` fields
  (previously empty strings).

**Verified**: `godot4 --headless --quit main.tscn` clean load. Real
windowed-app screenshot (`PrintWindow`, this session's established
technique) confirms the title-bar icon is genuinely the new design, not
Godot's default — cropped and zoomed the exact icon region to check, not
assumed from the config change alone. Export-time icon baking (the actual
`.exe`/`.apk` icon, as opposed to the debug-run title bar) not re-verified
by a fresh export in this pass — the config wiring is real and points at
real files, following the same `application/icon`/`launcher_icons/*` fields
this project's Windows/Android builds already use, but a full export was not
re-run just for this cosmetic change.

**Files touched**: `design/app-icon.png` (owner-supplied), `godot-project/
icon.png`, `godot-project/icon.ico`, `godot-project/icons/android_*.png`
(4 files, new), `godot-project/project.godot`, `godot-project/
export_presets.cfg`, this file, `docs/STATUS.md`.

## GUI shell second workflow re-audit: Layers made a permanent panel (2026-08-17)

Owner asked to re-check the shell against the design mockup/menu-structure
docs once more. Re-reading `design/Cartalith GUI.dc.html`'s own `1a`/`4a`
reference screens against the running shell surfaced a real structural
mismatch, not a cosmetic one: the mockup's Layers panel is a permanent third
column beside the workspace navigator, always visible regardless of which
nav subject is active — the first shell pass (`5d44c6b`) had instead made
Layers a destination the navigator *swapped to* (`CARTOGRAPHY:Layers`),
collapsing two of the mockup's always-visible regions into one slot.

Restructured `main.tscn`: `LayersContent` (settlement/territory/province
toggles) moved out of the swappable `SecondPanel` into a new, permanent
`LayersPanel` sitting between it and the viewport — a real fifth column,
matching the mockup's own region count. `CARTOGRAPHY:Layers` now correctly
explains itself ("Layer visibility is always available in the LAYERS panel
to the right...") rather than either duplicating the real panel or showing
the generic "not wired yet" placeholder, which would have been actively
misleading for a subject whose content genuinely does exist, just not
behind that click.

**Verified**: `cargo build -p cartalith-godot`, `cargo test --workspace`
(0 regressions), `godot4 --headless --quit main.tscn` clean load. Real
windowed-app screenshots, before and after the restructure, and a real
end-to-end generation (seed 12345, Classic, 2048×2048, 40 settlements)
through the new layout — Layers panel, its three toggles, and the map all
render correctly together.

**Files touched**: `cartalith-native/godot-project/main.tscn`,
`cartalith-native/godot-project/main.gd`, `GUI_SHELL_SCOPE.md`, this file.

## Phase 3 milestone 3: hydrology-based colour tint (2026-08-17)

Third `TERRAIN_APPEARANCE_SCOPE.md` pass — a new `hydro_wet_strength`/
`hydro_wet_radius_frac` pair on `TerrainAppearance`, applied at
`land_color`'s final tonal stage (alongside AO/vignette, never touching the
golden-verified `material_weights` blend). `build_hydro_wetness` reuses the
existing `flow` field (already threaded through `RenderCtx`, so this needed
zero `lib.rs` changes), log-compresses and min-max normalizes it the same
way `build_ao` already does for the same §32 reason (a fixed threshold
would flatter one world and wash out another), keeps only the top of the
range via `smoothstep`, and blurs it into a soft halo around channels. The
result is a subtle cool/dark pull in `land_color`'s final tone near high
flow accumulation — an ambient "there's real drainage near here" cue, not a
repaint of rivers into the raster (the vector river overlay is untouched,
per `TERRAIN_APPEARANCE_RESEARCH.md` §13's own explicit boundary).

**Golden-parity**: `js_reference()` sets `hydro_wet_strength: 0.0`, which
skips the precompute entirely and leaves the term a true no-op — both
`golden_parity_render.rs` tests pass at their original `1e-4` tolerance,
file unmodified except the appearance-construction call already changed by
milestone 2.

**A real tuning pass, disclosed rather than hidden**: the first parameter
guess (strength 0.20, activation threshold 0.72–0.97, radius 0.004×gw)
passed every build/test/clippy check and produced a diff shaped correctly
like real river networks under 15× amplification — but a side-by-side crop
at *actual* strength showed nothing perceptible: 0.4% of pixels changed by
a mean of 2.5 (out of a possible 765). Caught by looking at the real crop,
not by trusting the diff statistics or the amplified visualization.
Retuned to 0.38 / 0.55–0.88 / 0.006 and re-verified the same way — 2.19% of
Classic's pixels now change, and the crop centred on the single most-changed
pixel (found programmatically, not eyeballed) shows a real, deliberately
subtle cooling along the actual valley floor.

**Honest cross-world result**: Classic — visible at its strongest point.
Archipelago (low-relief, fragmented, less continuous drainage) — only 0.75%
of pixels touched, essentially imperceptible even at its own strongest
pixel. Not a bug — there's simply less major flow accumulation on a world
shaped like that, the same honest shape milestone 2's own AO finding on
Archipelago already had. Both worlds: luma minimum identical before/after
(no new black valleys — the anti-list's own explicit warning), no banding
or haloing observed in either crop.

**Verified**: `cargo build -p cartalith-godot` clean, `cargo test
--workspace` 0 regressions, `cargo clippy -p cartalith-godot --all-targets`
clean for this milestone's files (three pre-existing warnings elsewhere,
confirmed unrelated by file/line — two in `cartalith-civ` from concurrently-
landing work, one pre-existing `needless_borrow` in `lib.rs`), `godot4
--headless --quit main.tscn` clean. Primary before/after comparison used the
deterministic `appearance_ab_dump.rs` harness (extended with an isolation
pair holding milestone 2's relief/AO fixed so this milestone's own delta is
measured independently), following milestone 2's own established finding
that windowed UI automation was unreliable this session — one real
end-to-end windowed-app run (seed 12345, Classic, 2048×2048, 40 settlements)
confirmed correct generation and rendering with no crash or visual
corruption, not a repeated multi-shot visual comparison.

**Note on shared build environment**: `cargo build` hit a transient
`Access is denied` on `cartalith_godot.dll` several times mid-task — a
concurrent fork's own windowed Godot instance had the DLL loaded. Resolved
by polling/retrying rather than force-closing anything, except for one
necessary `Stop-Process` sweep of all Godot instances immediately before
this milestone's own single real-app screenshot check, to guarantee a clean
window handle — flagged here in case a concurrent fork's own screenshot
verification was interrupted by that.

**Files touched**: `cartalith-native/crates/cartalith-godot/src/render.rs`,
`cartalith-native/crates/cartalith-godot/tests/appearance_ab_dump.rs`,
`TERRAIN_APPEARANCE_SCOPE.md`, `VISION.md`, `docs/STATUS.md`, this file.

## Journey Planner milestone 2: transport mode selection (2026-08-17)

`JOURNEY_PLANNER_SCOPE.md`'s next milestone. Real finding: 4 of the 10
originally-listed functions (`jpAutoPickTransport`, `jpAutoPickVessel`,
`_jpBestLandTransportForStage`, `_jpBestPackageForStage`) turned out to have
a genuine dependency on milestone 5's route/stage derivation (or milestone
3's `jpCalcLand`) once the real reference code was read line-by-line — not
assumed from the scope doc's own earlier guess. Left unported, re-flagged
under their real dependency milestone rather than forced.

The other six shipped, given caller-supplied stage lists instead of the
full JS `plan` object: `jp_best_animal_for_context`, `jp_pick_species_for_route`
(includes the v1.50 bottleneck-veto logic — a route mostly plains with one
real mountain-pass stretch switches the whole route's animal choice),
`jp_resolve_mount`, `jp_vessel_matrix`, `jp_vessel_fits`, `jp_auto_stage_vessel`,
plus their real data tables (`JP_ANIMALS`, `JP_ANIMAL_TERRAIN_OVERRIDE`,
`JP_TERRAIN` land/river/sea, `JP_SHIPS`, `JP_VESSEL_PREFERENCE`,
`JP_WATER_WINDOW`) and supporting pure functions (`jp_animal_terrain_mod`,
`jp_water_window`, `jp_vessel_water_block`, `jp_vessel_day_km`).

**The biome-mapping question this scope doc worried about turned out to
already be answered by the reference itself.** `jpLegacyBiomeOf` (reference
line 18310) already maps `classifyBiome(T,M)`'s output keys onto
`JP_BIOMES`' legacy names — and those keys are the exact same climate-biome
scheme this port's own `classify_biome` golden-verifies against, confirmed
by reading both side by side. Ported as `jp_biome_key(biome_id, temp_c)`, a
direct transcription of the reference's own fallback table (including the
desert/temperature split), not an invented mapping.

15 new unit tests, no golden harness needed (same precedent as milestone
1 and `civ_resource_trade_balance`/`civ_culture_terrain_fit` — small, pure,
branch-complete functions). A hand-computed vessel-speed test (Cog on
Coastal Waters: 10 × 11 × 0.60 = 66.0 km/day) and a real bottleneck-veto
test are the two that exercise the least-trivial arithmetic.

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ
--lib` (127 passed, 0 failed, 15 new), `cargo clippy -p cartalith-civ --lib`
clean (fixed one real `collapsible_if` in the new `jp_vessel_matrix` code;
two pre-existing, unrelated warnings elsewhere untouched), `cargo test
--workspace` (0 regressions). Not wired to any caller, per this doc's own
established "ship the primitive ahead of the orchestration" precedent and
`JOURNEY_PLANNER_SCOPE.md`'s explicit "out of scope for all milestones"
section (Journey Planner is real interactive per-journey tooling, a future
GUI feature, not something auto-wired for every settlement pair).

**Files touched**: `cartalith-native/crates/cartalith-civ/src/lib.rs`,
`JOURNEY_PLANNER_SCOPE.md`, `docs/STATUS.md`, this file.

## GUI decluttering pass: real target IA, real dark theme (2026-08-17)

`GUI_SHELL_SCOPE.md`'s milestone-1 shell had two real reference-fidelity
violations and one real visual-consistency bug, all owner-flagged after a
design-lead agent researched the reference app, the shell as it stood, and
the mockup and produced a concrete target IA. Implemented in full — real
menu/panel restructuring, real control relocation, real dark restyling, no
new engine functionality anywhere.

**Navigator**: `INFRASTRUCTURE` (Roads/Rivers/Ports/Trade/Logistics — zero
reference grounding) replaced wholesale with `EXPLORE` (Tools/Timeline/
Info/Journeys/Journey Planner — the reference's real second mode).
`CARTOGRAPHY:Layers` removed as a nav subject (it and `LayersPanel` were
two surfaces for one thing, per the reference app's own admission); the
freed 5th CARTOGRAPHY slot now holds `Paint`, the reference's real brush
bucket, previously homeless. `WORLD:Resources` (zero grounding) replaced
with `WORLD:Sculpt` (the reference's real 4th Generate branch). CIVILIZATION
subjects renamed to the reference's real 5 (`Population`→`Factions`,
`Politics`→`Generation`, `Culture`→`Statistics`); Roads/ways controls that
were stranded in the deleted `INFRASTRUCTURE` group conceptually now live
in `Generation`'s Step 2, matching the reference. CARTOGRAPHY subjects
renamed to the reference's real 5 (`Styling`→`Map Style`, `Assets`→`Icons`,
`Export`→`Map View`). 18 of 20 non-Overview subjects now carry a specific
reference-grounded honest placeholder instead of one generic string.

**Top bar**: `ProjectMenu`'s invented `New world.../Save project` deleted,
replaced with honest Import/Export groups. `GenerateMenu`'s fabricated flat
11-stage pipeline — the single largest piece of invented structure in the
prior shell — replaced with the reference's real Civilization Step 1→2→3
sequence. `SimulateMenu`/`MapMenu`/`ViewMenu` renamed to real
reference-grounded items (same slot counts). `AssetsMenu` converted
`MenuButton`→`Button` (a mode-switch toggle in the reference, not a
dropdown). Added a `ThemeToggleButton` to the global header (reference has
one, mockup specifies it, it was simply absent) — stays `disabled`, the
light-theme milestone itself is still deferred.

**Real bug fixed**: `FooterVBox` (Generate/Load Save/Status) had no
visibility gating at all — it persisted, visible, across all 20 nav
subjects instead of `WORLD:Overview` alone. Fixed.

**Visual consistency, the largest real fix**: authored a real dark
`Theme` resource (`theme/dark_theme.tres`) from the exact token values
already scattered as inline literals throughout `main.tscn` — surface
`#0d0e0f`, text `#c8cbcd`, accent `#e0a34a`, etc. — with real styles and
explicit `disabled` states for `Button`/`CheckBox`/`OptionButton`/
`SpinBox`/`LineEdit`/`HSlider`/`FoldableContainer`. Assigned as both
`Main`'s theme and the project-wide default (retiring `app_theme.tres`,
the MVP's light-parchment theme, from the live path without deleting it),
and directly on `CreditsDialog` (Window nodes don't inherit Control-tree
themes — confirmed real: Credits was a fully unstyled default-grey dialog
before, legibly dark-themed after, though its background panel hue stayed
Godot's own default grey — a smaller, flagged, real remaining gap).
`theme_type_variation = &"SettingsCard"` retired: the three light-parchment
cards (`WorldParamsCard`/`WorldStructureCard`/`AdvancedCard`) sitting on
the dark shell — the single most visible inconsistency in the prior
shell — flattened into plain sectioned `VBoxContainer`s with `HSeparator`
dividers; `AdvancedCard` became a Godot 4.4+ `FoldableContainer`, collapsed
by default, matching the reference's own `<details>` pattern.
`map_overlay.gd`'s settlement hover-card literals recolored from cream/
brown to the same dark tokens.

**Verified**: `cargo build -p cartalith-godot` clean (0 new Rust — pure
GDScript/scene/theme work), `cargo test --workspace` 0 failures, `godot4
--headless --quit main.tscn` clean load re-checked after each incremental
restructuring step. Real windowed-app before/after screenshots — the
*before* shot came from genuinely running the old shell (`git stash` of
every changed file, screenshot, `git stash pop` to restore), not memory.
Full golden path reconfirmed through the restructured shell: Generate
(seed 12345, 2048×2048, 800 km, Classic → 40 real settlements), territory/
province-boundary toggles, settlement hover (dark on-canvas card *and* the
causal "WHY HERE?" Inspector chain, still correct — `strong fresh water
(0.93) → strong gentle terrain (0.99) → weak fertile land (0.34)`,
`Suitability 0.80`), Credits (now dark/legible), Load-Save (opens, browses
the real filesystem). `ProjectMenu`/`GenerateMenu`/`ViewMenu` popup content
and the `CIVILIZATION:Settlements`/`CARTOGRAPHY:Paint` honest placeholders
(including the `FooterVBox` fix, directly visible) all confirmed against
the plan's own tables.

One real drift from the design-lead plan's own assumption, found and
corrected rather than silently followed: the plan's §3 described
consolidating a pre-existing "debug/analysis layer picker (30 views) +
opacity" into `LayersPanel`. No such picker has ever existed in this
codebase — checked, not assumed — `LayersPanel` was already the one
honest surface. Nothing to consolidate; not fabricated to match the plan's
text.

**Files touched**: `cartalith-native/godot-project/main.tscn`, `main.gd`,
`map_overlay.gd`, `project.godot`, new `theme/dark_theme.tres`,
`GUI_SHELL_SCOPE.md`, `docs/STATUS.md`, this file.

## Journey Planner milestone 3: physical travel cost (2026-08-17)

`JOURNEY_PLANNER_SCOPE.md`'s next milestone. **The biggest real finding is a
dependency-ordering error in that scope doc itself**, found by reading the
reference rather than trusting the plan: milestone 3 is listed *before*
milestone 4, and it needs to be the other way round. `jpCalcLand` (reference
line 18912) calls `jpCapacity` (18177), `jpForaging` (18156),
`jpAssessResupply` (18231) and `_jpDesertTierForGap` (18727); `jpCalcWater`
(19124) calls `jpAssessResupply` and `jpHumanWaterRate` (17626). Every one of
those is on milestone 4's own list, and they are not thin shims —
`jpCapacity` is the whole seasonal-physiology/draft-shortfall/mount-saddlebag
mass model, and `jpForaging` reaches through `_jpWildlifeForageMod` into the
world's wildlife-richness field, real world context never plumbed into the
Journey Planner. So `jp_calc_land`/`jp_calc_water` stay unported and are
re-flagged under milestone 4, on exactly the discipline milestone 2 used for
its own four deferrals. The scope doc is corrected accordingly.

Two more of the eleven listed functions (`jp_water_window`,
`jp_animal_terrain_mod`) had already shipped with milestone 2, which needed
them for its own work. Not re-ported.

**The seven that shipped**, all self-contained given a caller-supplied
party/leg summary instead of the full JS `plan`/`jn` object: `jp_train_pace`
(the slowest-carrier rule — wheels, then travois, then pack animals, porters
last), `jp_sail_factor` (v1.97's rig-class sail polar: zero in the no-go
zone, peak on a beam-to-broad reach, falling off again dead downwind — not a
cosine of wind angle), `jp_wx_weighted` + `jp_weather_factor` (season×biome
probability-weighted weather, blending the pace animal's own affinity, plus
v1.44's forced-condition override), `jp_column_length_km` +
`jp_column_factor` (v1.51's road-capacity damping — the fix for bigger
parties coming out monotonically *faster* in v1.50), and `jp_journey_cost`.
Supporting data ported alongside: `JP_TRAIN_PACE`, `JP_RIG`/`JP_SHIP_RIG`,
`JP_WEATHER`, `JP_ANIMAL_WEATHER_OVERRIDE`, `JP_FILES_BY_TERRAIN` and the
column-spacing constants, `JP_COST_*`, plus one small shared `JpParty`
struct.

**`JP_BIOMES[...].weather` — the table the scope doc flagged as "not yet
identified as ported or not" — was NOT ported**, checked rather than
assumed: milestone 2 deliberately narrowed its `JP_BIOMES` port to the two
fields `jpBestAnimalForContext` reads and said so in its own doc comment. The
weather distributions (12 biomes × 4 seasons × 5 conditions) are ported here
with the two functions that consume them. The remaining columns
(`water`/`forage`/`waterForage`/`grazing`) stay unported and belong to
milestone 4.

**`jp_journey_cost` turned out portable**, confirmed by reading its real
signature rather than assuming (the scope doc warned it might need milestone
2's transport selection to have run). The reference's own comment calls it
"pure over the plan object — no globals, no DOM", and that held: it touches a
five-field per-leg summary (`cat`/`st.km`/`days`/`crew`/`blocked`), one
`claimedFrac` per stage, the trip totals and the party. Ported with a
`JourneyLeg` input struct narrowed to exactly those fields — which is also
the shape `jp_calc_land`/`jp_calc_water` will produce once milestone 4
unblocks them.

**Milestone 2's four deferrals: none resolved**, re-checked by reading each
again rather than inferred. `_jpBestLandTransportForStage` calls `jpCalcLand`
in its inner loop, and `jpCalcLand` did not land — so it stays blocked, now
behind milestone 4 rather than milestone 3. `jpAutoPickTransport`/
`jpAutoPickVessel` still open with `_jpEnsurePlan`+`_jpDeriveStages`
(milestone 5); `_jpBestPackageForStage` still takes an `_jpEffectiveStagePlan`
-shaped argument.

**Golden-verified against the real reference**, not hand arithmetic — the
first Journey Planner milestone to use a harness rather than pure unit tests,
because the weather blend is a 48-cell five-term float sum where hand
arithmetic would be the weak link. The reference's own source lines for all
seven functions and their tables were sliced out of `reference/Cartalith Gen1
v2.10.html` by line range and run in a bare Node `vm.runInContext` with no
DOM (the same technique Phase 2 used throughout, applied to functions pure
enough not to need a generated world to drive them). Every expected value in
the 12 new tests is that run's output: all 48 `jpWxWeighted` cells as a
block, the sail polar's five control points plus interpolation and
angle-folding (−90°/270°/400° all fold correctly), and two full
`jpJourneyCost` breakdowns. One real harness bug caught before anything was
trusted: an unterminated block comment at a slice boundary was swallowing the
following slice.

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ
--lib` (139 passed, 0 failed, 12 new), `cargo clippy -p cartalith-civ
--all-targets` (two real findings in the new code fixed — a `manual_clamp`
and an `inconsistent_digit_grouping`; the lib is back to the same two
pre-existing unrelated warnings milestone 2 recorded, and the new test code
adds none), `cargo test --workspace` (0 regressions). Three sibling forks had
uncommitted work in the shared tree at the time, including a half-created
`cartalith-assets` crate whose missing `src/lib.rs` broke workspace manifest
loading; rather than edit a shared `Cargo.toml` out from under them,
verification ran against a scratch mirror of the workspace with that one
in-progress crate omitted. **Not wired to any caller** — no `#[func]`, no
`compute_civilisation()` integration, per `JOURNEY_PLANNER_SCOPE.md`'s own
"out of scope for all milestones" section.

**Files touched**: `cartalith-native/crates/cartalith-civ/src/lib.rs`,
`JOURNEY_PLANNER_SCOPE.md`, `cartalith-native/docs/STATUS.md`, this file.

## Phase 4 started: Asset Library investigated, milestone 1 (pack manifest) (2026-08-17)

`ROADMAP.md`'s Phase 4 is one sentence ("Block 3, the sprite and texture pack
system") plus a "Confirm before starting" note, which the owner's own
direction to continue "until you've finished phase 4" satisfies. This entry
covers the investigation that sentence deferred and the first real milestone
built on it. Full findings in the new `ASSET_LIBRARY_SCOPE.md`.

**What the Asset Library really is**, read out of `reference/Cartalith Gen1
v2.10.html` rather than out of the two pre-implementation design docs in
`docs/` (where those disagree with the shipped code, the code won and the
disagreement is recorded):

- **An asset is not an arbitrary named image.** It is one PNG bound to one
  slot in a frozen, ordered vocabulary the engine already knows how to draw --
  8 families, 7 of them closed (7 splat channels, 15 biome grounds, 13 terrain
  grounds, 10 feature icons, 9 settlement pins, 7 trait overlays, 8 POI
  markers) plus one open-vocabulary `custom` family of user-named icons in
  user-named sets. Slots hold 1..N variants, picked deterministically by
  position hash so a ridge of forty peaks is not forty copies of one drawing.
- **Order is load-bearing twice over**: the biome/terrain lists are index-
  aligned 1:1 with the frozen `CART_BIOMES`/`CART_TERRAINS` paint vocabularies,
  and the structure lists mirror `CIV_SETTLEMENT_CLASSES`/`CIV_POI_TYPES`/
  `CIV_TRAITS` key for key.
- **An asset pack is a real serialization format**, not a proposal -- a plain
  PKZIP written by the same `zipStore()` the world save uses, with a
  `pack.json` (schema 1, or the schema-2 superset) or a real `pack.csv`
  alternative over that frozen vocabulary. Unknown keys are dropped *with a
  warning*, never rejected, so parsing can only fail on a missing or malformed
  manifest -- never on content. **The manifest, not the folder layout, is the
  source of truth**; `textures/`, `icons/` etc. are only what the exporter
  happens to write.
- **A second, different format also exists**: `_alExportEntries`/
  `_alImportProject` embed the *editable* Library (per-slot metadata, tags,
  collections, per-item transforms, scatter rules) into a project `.zip` as
  `assetlib/library.json` + `assetlib/img/N.png`. That is the "Asset Library
  payload" `SAVEFILE_COMPAT.md` already lists among entries the MVP reader
  ignores. The live `assetPack` global is deliberately never serialized into
  `params.json` (the reference's transient-UI invariant 6).
- **The renderer really does draw pack sprites**, and has for many versions --
  the vector glyphs are the fallback, not the other way round. `placeMapIcons`
  (with a v1.26 rule-driven engine behind it) decides where, `iconSlotForItem`
  resolves the slot, `pickWeightedVariant` picks the variant, `drawMapIcons`
  composites in one Y-sorted painter's pass, bottom-anchored via
  `spriteDrawRect`. Phase 5's urban morphology does **not** consume packs
  (checked: block 4 has no `assetPack` reference).

**How big Phase 4 actually is -- stated plainly rather than understated**: block
3 (the Asset Library page) is ~1,439 lines, block 1's asset regions ~800 more,
plus block 2's consumers -- **~2,250+ lines against the Journey Planner's
~3,100**. But where the Journey Planner was ~70 functions of dense portable
modelling, this is ~600-800 lines of portable logic wrapped in 1,000+ lines of
editor UI (the sprite-sheet slicer modal alone is ~408 lines, almost entirely
canvas/pointer interaction) and a platform layer of image/ZIP handling that is
crate work, not porting. A real sub-phase; scoped into seven milestones.

**Milestone 1 shipped: new crate `cartalith-assets`** -- the pack *manifest*:
data model, parser, validation warnings, schema-2 serialization. Deliberately
the piece with no images, no archive, no renderer and no UI in it, and the
piece every later milestone is defined against. Zero `gdext`, zero dependency
on any other Cartalith crate -- the standalone shape `cartalith-spatial` set.

- `slots.rs` -- all seven frozen vocabularies verbatim, plus a `Family` enum
  carrying each family's manifest section, export directory, bake size,
  opacity, anchor and multi-variant flag (the reference's own `FAMILIES`
  metadata), `Family::asset_path` (the exporter's path convention) and
  `slug_id`.
- `manifest.rs` -- `RawManifest` (as authored, key order preserved) and
  `PackManifest` (validated), `parse_pack_csv`, `parse_pack_manifest`,
  `parse_pack_entries`, `pack_summary`, `to_raw`/`to_pack_json`,
  `referenced_files`, and a `PackError` whose `NoManifest` message is the
  reference's own thrown string.
- `ordered_map.rs` -- a ~40-line insertion-ordered map. Not incidental: the
  reference emits its unknown-slot warnings by iterating the author's own
  objects, and JavaScript iterates string keys in insertion order, so **warning
  order is a function of how the pack was written**. `BTreeMap` would sort it
  away, and serde_json's `preserve_order` feature would have leaked into
  `cartalith-io` through workspace feature unification -- so a small local type
  instead of a shared behaviour change.

**Golden-verified against the real reference** -- a real execution path exists
for this logic, so it was used rather than stood in for. A transient Node `vm`
harness (the same technique Phase 2 used throughout) slices
`parsePackCsv`/`parsePackManifest`/`packSummary` and their six `PACK_*_SLOTS`
vocabularies out of the frozen HTML by line range and runs them on five
fixtures; every expected value in `tests/golden_parity_pack_manifest.rs` is
that run's output verbatim. All five cases matched on the first run.

The fixtures deliberately target what a rewrite gets *plausibly* wrong rather
than the happy path: a missing texture file; an unknown texture slot; an
unknown biome slot that is really a *terrain* slot (dropped even though its
file is present); one missing icon variant (slot survives) versus every variant
missing (slot dropped whole, not left empty); a bare string standing in for a
one-element variant list; an unknown settlement slot; a missing custom-set
variant that kills the slot but not the set; CSV variant ordering as a *stable*
sort with unnumbered rows pushed to the end (an unstable sort would silently
reorder them); JSON winning over CSV when both are present; an empty-string
path counting as a missing file because the reference's `has` is a truthiness
test; and the exact wording and ordering of all nine resulting warnings.

Two real reference details found by reading and preserved rather than tidied:
the CSV path drops unknown slots **silently** while the JSON path warns, and
the pack-import `poi` vocabulary has 8 slots while the Asset Library's own has
10 (`lake`/`bridge` have no engine POI kind, so they can be authored and
exported but never load -- the reference documents this and shrugs).

**Verified**: `cargo build -p cartalith-assets` and `cargo build --workspace`,
`cargo test -p cartalith-assets` (28 tests -- 18 unit, 9 golden, 1 doctest -- 0
failures), `cargo clippy -p cartalith-assets --all-targets` clean (one real
`collapsible_match` in the CSV dispatch fixed by moving all four arms onto
match guards), `cargo fmt`. `cargo test --workspace --exclude cartalith-gpu`:
0 regressions. `cartalith-gpu` was excluded because a sibling fork's
in-progress GPU flow-accumulation work had its lib tests mid-edit and
non-compiling in the shared tree at the time -- unrelated to this crate, which
nothing depends on. Nothing Godot-side was touched, so no scene load was
needed. **Not wired to anything** -- same "don't wire in what nothing calls"
discipline as `cartalith-spatial` and every unwired Phase 2 primitive.

**Files touched**: `cartalith-native/crates/cartalith-assets/` (new),
`ASSET_LIBRARY_SCOPE.md` (new), `cartalith-native/docs/STATUS.md`, this file.
The workspace `Cargo.toml` needed no edit -- its `members = ["crates/*"]` glob
picks the new crate up automatically.
