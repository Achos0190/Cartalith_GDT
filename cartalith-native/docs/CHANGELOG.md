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

## GPU layer integration milestone 9: flow accumulation, redesigned not ported (2026-08-17)

The owner's "do the algorithms for the GPU" directive, taken at the one row
`GPU_LAYER_INTEGRATION_SCOPE.md`'s own feasibility table had deferred
longest: flow accumulation, "poor GPU fit without a real algorithmic
redesign". This is that redesign — the first genuinely *sequential* CPU
algorithm in this pipeline to move to GPU, as opposed to the eight
per-cell/local-neighbourhood kernels milestones 1-8 built.

**Built:**

- `cartalith-gpu/shaders/gpu_flow.wgsl` — three entry points.
  `dir_main` computes the D8 steepest-descent receiver per cell (a literal
  transcript of `compute_flow`'s inner loop: same visiting order, same
  strict `>` first-max-wins tie-break, same world-wrap branch — and this
  kernel *does* implement world-wrap, unlike milestones 1-5's, since for
  flow direction it is one extra modulo rather than a structural change).
  `scatter_main`/`merge_main` are one **pointer-doubling** round.
- `GpuFlowContext`/`init_gpu_flow_with`/`dispatch_gpu_flow`/
  `flow_accumulation_gpu_with` in `cartalith-gpu`, on milestone 8's shared
  `GpuDevice` from the start (no standalone per-call-context version was
  ever written — milestone 6's mistake was not repeated).
- `examples/flow_downstream_settlements` — the milestone's real
  measurement harness (see below).
- Wiring in `cartalith-engine::generate_terrain` behind `p.use_gpu`, at all
  four `compute_flow` call sites, with per-stage CPU fallback and never a
  panic. `"flow"` joins `WorldState.gpu_stages_used`.

**The parallel formulation.** `compute_flow` sorts all `n` cells by
descending height, then walks that order pushing each cell's running total
into its receiver. Those are two separable things: flow *direction* is a
pure function of the height field (it never reads `acc`, so the ordering is
irrelevant to it — embarrassingly parallel), and the *accumulation* over
the resulting single-receiver forest is a subtree sum, which the
descending-height walk computes incidentally rather than fundamentally.
Subtree sums over a pointer forest parallelize by pointer doubling
(path doubling / dependency transfer, the shape Qin & Zhan 2012 and the
2016 RUSLE paper describe, and `HETEROGENEOUS_COMPUTE_RESEARCH.md` §48-49
summarizes): each round every cell delivers its current total to the cell
its pointer names, then re-points at that cell's pointer. Invariant after
round `k` — `acc[i]` is the seed sum of everything upstream of `i` within
`2^k` steps and `ptr[i]` is the cell `2^k` steps downstream — so
`ceil(log2(n))` rounds is a hard upper bound (22 at 2048×2048), against
the thousands a naive donor-gather-to-fixpoint would need and against the
global sort the CPU pays.

**Fixed-point accumulation, deliberately.** WGSL has no atomic float add.
A compare-exchange emulation would make the answer depend on which thread
wins each race — non-deterministic run to run, which every GPU milestone
here has had to rule out. `acc`/`delta` are `atomic<u32>` fixed point
instead: integer addition is exactly associative and commutative, so the
scatter is order-independent *and* bit-reproducible. The scale is the
largest power of two whose worst-case total still fits `u32`, derived per
call from the real seed total. Worth stating plainly: the GPU rounds each
seed **once** and is exact from then on, while the CPU rounds to `f32` on
**every** write (`acc[best] = (acc[best] as f64 + acc[i] as f64) as f32`,
thousands deep at a major outlet) — at large accumulations the GPU path is
arguably the more accurate of the two.

**Verified:**

- **Flow directions: 0 mismatches out of 262,144** at 512×512, both
  world-wrap modes, two roughness regimes. The `f64`-vs-`f32` near-tie risk
  this milestone expected to have to quantify did not materialize.
- **Accumulation vs. the real, untouched `cartalith_hydrology::compute_flow`**
  (`cartalith-hydrology` added as a `cartalith-gpu` dev-dependency —
  milestone 4's discipline; no noise dependency, so no §7c carve-out
  needed): **bit-exact (max_abs = 0.0)** for `use_rain=false`, the
  area-only seeding the pipeline's first call uses. For discharge seeding
  the error is pure seed quantization, with the *opposite* shape to the
  CPU's — worst at tiny accumulations built from many small seeds,
  shrinking as accumulation grows. On real generated worlds, at and above
  `river_flow_thresh` (the only regime any downstream consumer
  distinguishes): **1.3e-4 relative at 512², 3.3e-4 at 1024²**. New
  `FLOW_TOLERANCE` (1e-3) bounds that regime; `FLOW_ANY_CELL_TOLERANCE`
  (5e-3) is the loose guard over sub-threshold cells (worst 2.6e-3, on
  cells accumulating ~1.2 units).
- **Bit-reproducible run to run** — asserted, since it is the whole reason
  fixed point was chosen over a CAS loop.
- `cargo build --workspace`, `cargo test --workspace` (74 suites, 0
  failures, 0 modified tests — `cartalith-hydrology`'s own
  `golden_parity_flow.rs` untouched and passing, and `compute_flow` itself
  is byte-untouched), `cargo clippy -p cartalith-gpu -p cartalith-engine
  --all-targets` with no new warnings (the two `dead_code` ones on
  `dispatch_gpu_height`/`dispatch_gpu_resistance` predate this work —
  confirmed by checking the version at `HEAD`, not assumed).

**The measured downstream effect — the actual headline.** Flow
accumulation is the first GPU kernel here that is *not* a leaf
computation: it feeds rivers, which feed settlement suitability, which
feeds roads and territory. So "the numbers agree" was not a sufficient
answer, and the divergence was measured through to the civilisation layer,
holding terrain fixed (one real CPU-path world; both accumulations
computed over its own final height/rainfall fields, so the comparison
isolates flow accumulation rather than also swapping the noise/plate/
weather kernels the way flipping `use_gpu` would):

- **River network: zero difference.** `build_channels` +
  `strahler_from_receivers` on both accumulations, two roughness regimes —
  identical river-cell counts (2674/2674 and 6652/6652), 0 channel-mask
  cells differing, 0 channel receivers differing, 0 Strahler-order cells
  differing.
- **Settlements: zero difference.** The full `compute_civilisation`
  suitability chain run twice with every input identical except
  `flow_discharge`: the suitability raster differs only in its last `f32`
  digits (max 2.7e-6 at 512², 1.3e-5 at 1024²), and
  `find_settlement_seeds` returns **the same count and the same positions
  — 104/104 at 512², 125/125 at 1024², zero seeds moved.**

The divergence is real but lands entirely below the granularity anything
downstream resolves.

**Real timing.** Isolated kernel vs. the real CPU `compute_flow`, shared
device: 128² **0.20×** (GPU loses — the round count barely falls with grid
size, 14 vs 22, so a small grid pays nearly the same dispatch count over
far less work), 512² **4.6×**, 1024² **10.4×**, 2048² **15.5×** (31.5ms vs
488.9ms). End-to-end `generate_terrain`, same single-run benchmark
milestones 6/8 used: ratio moves 0.11×→0.16× at 128², 0.76×→0.83× at 512²,
1.14×→**1.36×** at 1024², and 0.98×→**1.74×** at 2048² — the largest
single-milestone shift this effort has produced, which makes sense given
`compute_flow` was ~490ms of pure CPU per call and is called up to four
times per generation. Absolute times moved more than ratios between the
two benchmark runs (CPU 2048² measured 4.86s here vs 5.83s in milestone
8's run with no code changed on that path), so the ratios are the
meaningful comparison — reported that way rather than quoting the flattering
absolute numbers.

**Two honest "shouldn't run on GPU" findings, from reading the real code
rather than the scope doc's summary of it:**

- **`build_water_bodies` (water-body priority-flood): half tractable, half
  a real research task — not attempted.** It is two algorithms back to
  back. The connected-components half has a real parallel formulation and
  the exact CPU answer is even reachable (component IDs are assigned in
  raster-scan discovery order, so its "largest, first wins on ties" is
  equivalent to "largest, smallest minimum cell index wins" — reproducible
  by label propagation or union-find by pointer jumping, the same machinery
  this milestone built). The depression-fill half is a global priority
  queue (`MinHeap`, classic Priority-Flood); its parallel formulations
  converge in O(longest ascending path) iterations, and unlike flow
  accumulation there is no pointer structure to double — the recurrence is
  a min-max over neighbours, not a sum over a tree. That is exactly the
  iterations-for-parallelism trade this milestone avoided. Measured for
  proportion: `build_water_bodies` costs **~92ms at 1024×1024**, an order
  of magnitude less than flow accumulation was costing.
- **`road_dijkstra` (Dijkstra/MST road networks): confirmed, stays on
  CPU** — agreeing with `HETEROGENEOUS_COMPUTE_RESEARCH.md` §53 but with
  two concrete reasons. Its `prev` array *is* the road geometry, and unlike
  an accumulation sum (order-independent once the receiver forest exists) a
  shortest-path predecessor is genuinely settle-order-dependent on ties, so
  every GPU alternative (Bellman-Ford, delta-stepping, fast marching) would
  visibly move roads. More decisively: it is already called many times over
  a small downsampled road grid, once per hub, at four call sites, all
  independent — and those call sites are still plain `.iter().map()`. The
  available parallelism here is **across sources on CPU** (Rayon would take
  them today), not within one traversal on GPU.

**Open / not attempted:** `compute_stress`'s gather reformulation and
orogeny's parallel-graph redesign (still deferred, milestones 5-6);
world-wrap for the milestone 1-5 kernels; averaging the timing benchmarks
to kill the single-run variance milestone 8 already flagged; per-pipeline
caching across repeated `generate_terrain` calls. `cartalith-gpu` now
carries a deliberate **dev-dependency cycle** (on `cartalith-engine` and
`cartalith-civ`, for the downstream-settlement example only) — cargo
permits cycles through dev-dependencies specifically, nothing in the
library depends on either, and the alternative was editing a crate three
other concurrent forks were working in.

## Phase 3 milestone 4: the atlas look — paper ground, forest stippling, plate border (2026-08-17)

Fourth `TERRAIN_APPEARANCE_SCOPE.md` pass, and the one that closes most of
`VISION.md`'s sequencing item 2. That item named four things still ahead
after milestone 3 — *"the paper/vellum ground, forest stippling,
hand-lettered glyphs and the physical border"*. Three of the four live in
`render.rs`'s raster and landed here; the fourth (hand-lettered settlement
glyphs) is drawn by `godot-project/map_overlay.gd`, not by this raster, and
was deliberately left alone.

**Built**, three independent stages on `TerrainAppearance`, each gated on
its own parameter:

- **Paper/vellum ground** (`paper_tone`/`apply_paper`, new
  `paper_strength`/`paper_tint`/`paper_grain`/`paper_mottle`/`paper_wash`).
  Applied in `cell_color` *after both* the land and sea branches, on
  purpose: an ocean that isn't on the same sheet as the land makes the map
  read as terrain art pasted onto a parchment background. Two composited
  parts, both luminance-preserving by construction — a parchment tint
  divided by its own Rec.709 luma (so it warms without dimming; a straight
  multiply by an off-white would cost ~10% luma everywhere and flatten
  exactly the relief legibility milestone 2 bought), and `paper_wash`, a
  pull toward a paper-coloured grey *of the same luminance* so chroma drops
  and nothing else does. Fibre is two fixed-cell-frequency coherent-noise
  octaves (one isotropic tooth, one stretched into laid lines) never finer
  than ~3 cells per feature; ageing is a separate sheet-scale mottle.
- **Forest stippling** (in `land_color`, `stipple_strength`/
  `stipple_scale_frac`). Weighted by `material_weights`' own `canopy`
  fraction — real data, not decorative noise laid wherever the image
  happens to look green — through a `smoothstep` gate (no hard biome
  borders), and applied as a **zero-mean** modulation so canopy gains
  texture without being net-darkened.
- **Physical plate border** (`apply_border`, `border_width_frac`/
  `border_ink`). A bare-paper margin carrying a thick and a thin neatline,
  ink density modulated along the rule by low-frequency coherent noise so
  it reads as drawn rather than as a CSS box. Widths floored in absolute
  cells so the frame survives the 512²–8192² range without the two rules
  merging. Pure presentation: it reads no world data at all.

None of the three touches `material_weights` or the 25 palettes — the same
rule milestones 2 and 3 both held to.

**Golden-parity: the same mechanism extended, not replaced.**
`js_reference()` gains `paper_strength: 0.0`, `stipple_strength: 0.0`,
`border_width_frac: 0.0`, and each stage **early-returns on its own zero**
rather than merely evaluating to an arithmetic no-op — `paper_tone` returns
before touching a single `vnoise`, the stipple block is inside an `if`,
`apply_border` returns its argument. That is exactly the discipline
`relief_lights <= 1` established in milestone 2 (a dedicated branch, so
parity can never drift on a float reassociation). `golden_parity_render.rs`
remains **completely unmodified** and both tests still pass at their
original `1e-4` tolerance with every expected value unchanged.

**Two real corrections caught by looking, not by the numbers** — milestone
3's lesson holding for a second milestone running:

1. The parchment tint on its own is only a hue rotation, and a side-by-side
   showed it reading far too weakly, leaving a digital-looking saturated
   ocean. `paper_wash` is what actually shifted the tonal feel: pigment
   soaked into a sheet is never as chromatic as an emitted colour, and that
   is the whole difference between a screen render and a printed plate.
2. The first stipple field read as a regular diagonal **halftone screen** —
   §30's "random texture noise" failure, and the same class of regression as
   milestone 2's AO speckle, found the same way (a 6× crop of the real dump,
   not a diff statistic). Fixed by rotating the sampling lattice ~34°,
   domain-warping it with a second coherent field, and flooring mark size at
   4 cells. Deterministic throughout (§27) — every stage is a pure function
   of the cell coordinates.

**Measured against §30's anti-list, terrain only** (2048², seed 12345, the
40-cell frame band excluded so the border doesn't skew terrain statistics;
"base" is milestone 3's look):

| | Classic base | Classic atlas | Archipelago base | Archipelago atlas |
|---|---|---|---|---|
| interior luma min | 42.4 | 41.0 | 34.6 | 33.8 |
| interior luma mean | 132.8 | **133.0** | 106.3 | **106.2** |
| interior luma sd | 31.32 | **31.89** | 27.66 | **28.30** |
| interior mean chroma | 59.7 | 51.96 | 70.3 | 51.96 |
| any-channel clipping | 0.70% | 0.73% | 0.03% | 0.03% |

Mean luma unchanged to a fraction of a level in both worlds; contrast
**rises** slightly rather than falling, so nothing was washed out; the luma
minimum drops 1.4 and 0.8 levels, entirely from paper grain, so no new black
valleys; terrain clipping unchanged.

**Cross-world honesty — and this time it runs opposite to milestones 2 and
3.** Both of those were strong on mountainous Classic and nearly invisible
on low-relief Archipelago because they keyed off relief and drainage. This
one is the reverse: the paper acts on the whole sheet and Archipelago is
mostly ocean, so it loses **26%** of its chroma against Classic's 13%, its
bright cyan sea becoming a muted teal-grey — the largest single visual
change either test world has seen in this phase. Worth recording: the two
worlds start 18% apart in mean chroma (59.7 vs 70.3) and land within 0.01 of
each other (51.960 vs 51.963), not by clamping (the ratios differ, 0.871 vs
0.739) but because a common printing medium is exactly what converges two
differently coloured subjects. Stippling mirrors it: 13.9% of Classic's
pixels touched vs 10.8% of Archipelago's, and it only really reads where
there is continuous canopy.

**Cost, honestly: not free.** 2048² render 598 → 915 ms (Classic) and 295 →
597 ms (Archipelago) — the paper is four extra `vnoise` calls on every pixel
including the ocean. A one-shot cost at generate time against a pipeline
that already takes far longer, so accepted rather than optimized, but a real
regression from milestone 2's "essentially free" and the obvious first
candidate if the render ever needs to be fast (the two sheet-scale mottle
octaves could be precomputed coarse and bilinearly sampled).

**One known limitation, found in the real app and not fixed here.** Two
systems draw *over* the finished raster and know nothing about the frame:
`lib.rs`'s river channel-mask tint and `map_overlay.gd`'s settlement/road
markers. In both test worlds a settlement at the extreme west edge puts its
marker partly on the plate margin. The fix (skip the overlay inside the
border band) belongs in those two files, outside this milestone's
`render.rs`-only scope and one of them owned by a concurrent fork this
session. Flagged rather than reached for.

**Verified.** `cargo build -p cartalith-godot` clean; `cargo test
--workspace` 383 passed / 0 failed with no expected value anywhere modified;
`cargo clippy -p cartalith-godot --all-targets` clean for this milestone's
files (the crate's sole remaining warning is the pre-existing
`needless_borrow` in `lib.rs`; `cartalith-gpu`/`cartalith-civ` warnings are
concurrent forks' and were confirmed unrelated by file and line — this pass
also cleared four `field_reassign_with_default` warnings the A/B harness had
accumulated); `godot4 --headless --quit main.tscn` clean load. Real windowed
app (2048², seed 12345, 40 settlements) generated and screenshotted for
**both** Classic and Archipelago: plate frame, parchment ground and canopy
texture all read correctly at the app's own display scale with the
settlement/road overlay on top. The controlled before/after is
`appearance_ab_dump.rs` run at the same 2048² the app uses — it now emits a
`noatlas`/`withatlas` isolation pair (milestones 2 and 3 held fixed) plus
`paperonly`/`stippleonly` dumps, since the three stages are independent and
a combined image cannot show which one carries a change.

## Phase 4 milestone 2: pack ZIP read/write (2026-08-17)

`ASSET_LIBRARY_SCOPE.md` milestone 2 of 7 — the reference's `unzipAny`
(line 12210) and `zipStore` (line 12009) in Rust terms, plus the entry
ordering its own exporter writes.

**Placement decided for real, not deferred again.** The scope doc left it
open between "`cartalith-assets` behind a feature" and "`cartalith-io`".
Reading `cartalith-io` first is what settled it: its entire zip surface is
`ZipArchive::new` + `by_name` + `read_to_end`, so there is no helper to
extract — the `zip` crate already *is* the shared piece that `unzipAny`/
`zipStore` were in the reference. Milestone 1's finding that packs use the
same writer as the world save is true, and precisely because it is true it
implies a shared *crate*, not shared code. Two further reasons pushed the
same way: `cartalith-io` is reading-only by explicit scope (`MVP_SCOPE.md`
point 12, `SAVEFILE_COMPAT.md`'s "Deferred"), and a pack writer there would
break that; and the dependency would point the wrong way, making every
consumer of the world-save loader drag in the asset vocabulary for a
subsystem that is optional by design.

So: **new `cartalith-assets::archive`, behind an on-by-default `zip`
feature**. `default-features = false` gives back exactly the archive-free
manifest model milestone 1 shipped — and that is *tested*
(`cargo test -p cartalith-assets --no-default-features`), not merely claimed.

**Ported, not delegated.** The container is the `zip` crate's job. What is a
real port is the reference's own policy around it, all of which a plain `zip`
call gets wrong by default:

- **`.png` is STORED, everything else DEFLATED.** A PNG is already internally
  DEFLATE-compressed; re-compressing it is wasted CPU for no size gain (the
  reference says so in its own comment).
- **Timestamps frozen at 1980-01-01 00:00:00** — `zipStore` hardcodes DOS date
  `0x0021`, time `0`, which makes exports byte-reproducible. `zip`'s own
  default is the wall clock, so this is set explicitly; there is a test that
  writes the same pack twice and compares bytes.
- **`pack.json` last**, after every image, matching the exporter's own append.
- **Names verbatim on read** — no wrapping-folder stripping, no backslash
  rewriting. Zipping the *folder* rather than its *contents* therefore fails
  with the reference's own "pack has no pack.json or pack.csv", which is the
  behaviour to keep: guessing a root would make every manifest path ambiguous.
  There is a test for it.
- **Directory entries survive** as zero-byte members (the reference walks the
  central directory and keeps what it finds; no manifest path ends in `/`).
- **An unrecognised method errors, worded as the reference words it** —
  `unsupported zip method 93 for pack.json`, obtained by reading the entry's
  metadata with `by_index_raw` before instantiating a decompressor, rather
  than letting the crate return a generic "unsupported archive".

Two deliberate non-ports, stated rather than smuggled: `zipStore`'s extra
"only if the compressed bytes actually came out smaller" fallback (a
browser-side size/`CompressionStream`-availability concern no reader can
observe), and `unzipStore` — `unzipAny`'s fallback for an archive with no
readable central directory, which answers `null` for every deflated entry.
That is a defence against a truncated `ArrayBuffer`, not a format variant;
`zip::ZipArchive` requires the central directory and errors cleanly without
it, which is the better answer.

**API**: `read_pack_entries` (→ `BTreeMap<String, Vec<u8>>`, the shape
`parse_pack_entries` already wanted), `read_pack` (entries + validated
manifest in one pass — `loadAssetPack` minus the image decode),
`write_pack_entries` (caller-ordered), and `write_pack` (manifest + images,
applying the exporter's own traversal order and appending `pack.json`).
`write_pack` errors on a manifest path the image map does not carry rather
than exporting a pack whose own parser would warn about every missing slot.

**Verified against a pack the reference itself exported — in both
directions.** The harness runs the reference's own
`PackManifestBuilder.build()` (line 26964) over its own `FAMILIES`/`AssetDB`
vocabulary and its own `zipStore()` headlessly under Node's
`vm.runInContext`, everything lifted verbatim by line range from the frozen
HTML — the same technique the 2026-08-15 "extraction harness upgrade" entry
established and `cartalith-io`'s `golden_parity_real_export.rs` uses for world
saves. Exactly two things in that run are not reference code, and the test
file says so up front: `renderToBlob` is a canvas rasteriser (replaced by a
real PNG encoder emitting genuine, valid PNGs at each family's own 512²/256²
bake size), and the three DOM inputs
`E('alPackName'|'alPackAuthor'|'alPackLicense')` are stubbed with real values.
Filenames, entry order, stored-vs-deflated, timestamps, the manifest's exact
JSON text and every CRC-32 are the reference's own output.

The result is checked in as `tests/fixtures/reference_pack.zip` (18 entries,
21 KB, covering all eight families including a custom set) plus that run's own
`unzipAny`/`parsePackManifest`/`packSummary` capture — so the comparison is
against what the reference saw, not against a re-read by this port's own
reader.

- **Read**: entries match name for name and CRC-32 for CRC-32 (the checksums
  the reference wrote into the archive, re-derived in the test by a ten-line
  port of the reference's own `crc32` rather than by adding a hash crate);
  `parse_pack_entries` reproduces the summary and the single warning; and
  `to_pack_json()` reproduces the exporter's `pack.json` **text byte for
  byte** — which incidentally confirms milestone 1 got `RawManifest`'s field
  order right (`textures`, `icons`, then only the non-empty
  `biomes`/`terrains`/`structures`/`custom`, because the reference
  pre-creates the first two in its object literal and adds the rest lazily).
- **Write**: `write_pack` reproduces the reference archive's entry order,
  per-entry compression method, CRC-32, uncompressed size and 1980 timestamps
  — and the bytes were fed back through the reference's own `unzipAny` +
  `parsePackManifest`, which read all 18 entries with byte-identical payloads,
  a byte-identical `pack.json`, and an identical summary and warning list.
  The two archives differ by **2 bytes in total**, first divergence at offset
  4 (version-needed-to-extract). Exact byte equality is not the bar and could
  not be: the one deflated entry is compressed by `miniz_oxide` here and by
  the browser's zlib there, and two conforming deflate encoders need not agree
  on a bit stream.

**Corrections this milestone made to `ASSET_LIBRARY_SCOPE.md`**: its §4 filed
ZIP read/write under "platform work, not a port", which is three-quarters
right — the container is, the export policy above is not, and the timestamp
case is one where the crate's default is actively wrong. Milestone 5 (the
Library model) is flagged to keep **both** the raw custom-set name and its
slug, confirmed by watching the real exporter emit `custom/naval/…` paths
under a `"Naval"` manifest key; losing either makes a round-trip lossy. And
`packSummary`'s "*N* custom icon(s)" counts slots, not variants — already
matched, noted because it reads like a bug.

**Verification**: `cargo build -p cartalith-assets` and `--workspace` clean;
`cargo test -p cartalith-assets` green with 14 new tests (4 golden-parity +
10 unit) and green again with `--no-default-features`, where the archive
module and its golden test compile out entirely; `cargo clippy -p cartalith-assets --all-targets`
clean for this milestone's files; `cargo test --workspace` 0 failures.
**Wired to nothing** — milestone 7 is where integration lives.

## Phase 4 milestone 3: scatter rules, with the v1.27 hardening ported as fixes (2026-08-17)

`ASSET_LIBRARY_SCOPE.md` milestone 3. New module `scatter` in
`cartalith-assets`: the `ScatterRule` model that decides *where* an asset gets
scattered on the map, its ten slot presets, and the hardened normalizer that
is the only way to build one out of a user-supplied project file. Still wired
to nothing — the placement engine that consumes these is milestone 4.

**Ported** (reference `Cartalith Gen1 v2.10.html` lines 6937-7039, 7088-7101
and 12171): `ScatterMode`/`SCATTER_MODES`, `ScatterRule` +
`Default` (`defaultScatterRule`), `preset_scatter_rule` with the ten
`SCATTER_RULE_PRESETS` inline, `scatter_rule_key`, `normalize_scatter_rule`,
`pick_weighted_variant`, `pick_icon_variant`, `current_scatter_rules`,
`autopopulate_scatter_rules`, and `ScatterRule::spacing_cells` (the relief
`spaceOf` helper, see below).

**The three v1.27 hardening fixes, re-derived for Rust rather than
transcribed.** Rules arrive from `assetlib/library.json` inside a
*user-supplied project `.zip`*, so every field is untrusted. v1.26 merged it
with the `+x||fallback` idiom, which lost a legitimate `0` (falsy in JS) and
let a `NaN` propagate instead of being rejected. `tests/hardening_v1_27.rs`
has one test per named failure, each reproducing the *downstream* arithmetic
inline (four lines, lifted from `placeMapIconsRuled`) so the test demonstrates
the failure it prevents rather than asserting a value:

1. **A `NaN` `density` scattering on every cell — still a real hazard here,
   by the opposite IEEE rule.** The JS predicate is
   `keep >= Math.min(1, density)`, and `Math.min(1, NaN)` is `NaN`, so nothing
   is ever rejected. Rust's `f64::min` *absorbs* NaN
   (`f64::min(1.0, NAN) == 1.0`) — but `keep` is a hash in `[0,1]`, so
   `keep >= 1.0` is false anyway and the corrupt rule still carpets the map.
   Same catastrophe, opposite mechanism. Rejecting non-finite input at the
   boundary closes both, and the same fix restores a deliberately-zero
   density (which v1.26's `||` idiom turned into 1, i.e. "place everywhere").
2. **A `NaN` `spacing` collapsing an O(1) neighbour test to O(n²) — real, and
   `f64::max` would have masked it.** `Math.ceil(W/NaN)||1` yields a 1×1
   bucket grid, so `fits()` degenerates from a nine-bucket lookup into a scan
   over every icon already placed. Rust's NaN-absorbing `f64::max` would
   rescue the derived-spacing path *by accident*; the explicit `is_finite`
   check is kept anyway, because an implicit dependency on an IEEE corner is
   precisely what this fix existed to remove — and fix 1 shows how little
   that intuition can be trusted.
3. **The `Object.assign` aliasing bug — structurally unreachable, and not for
   the reason one would guess.** It is *not* "Rust's ownership rules". The
   bug requires the defaults and the untrusted input to inhabit one mutable
   object (`Object.assign(base, r)` mutates `base` and returns it, so every
   `num(out.minSize, …, base.minSize)` fell back to the very `'x'` it was
   rejecting). Here they are different **types** — an owned `ScatterRule`
   with `f64` fields versus a `serde_json::Value` — so no merge-in-place is
   expressible, because a `"x"` can never be stored in the field it would
   have to corrupt. **No defensive code was written for it**; the test pins
   the reference's own probe case so a future refactor toward a "merge"
   helper fails loudly, plus a nothing-poisons-anything sweep over a record
   whose every field is garbage.

A fourth guarantee this port has and the reference cannot: `ScatterRule`
implements `Serialize` but **deliberately not `Deserialize`**, so the
hardening cannot be bypassed by a future caller reaching for
`serde_json::from_str` — `normalize_scatter_rule` is the only door in.

**Golden-verified against the real reference** (transient Node `vm` harness
over the frozen HTML, same technique as milestones 1-2, harness not checked
in). `pick_weighted_variant` is deterministic-hash-driven and diffs
**exactly**: an 11-case × 36-position sweep matched index for index,
including the three degenerate weightings that must fall through to
`pickIconVariant`'s untouched v1.25 hash. 37 `normalize_scatter_rule`
fixtures cover the JavaScript idioms a rewrite gets plausibly wrong
(`+"2.5"` vs `+"x"`, `0` falsy but `"no"` truthy, `Number.isFinite` not
coercing so `"4"` is dropped from a biome list while `5.5` is kept, `""` as
"unset" but `"   "` as 0, `"0x2"` as 2).

**One real bug the golden run caught on the first pass**: `density`'s
fallback is not symmetric with the other numeric fields. The reference merges
first and *then* runs `num(out.density,0,3,1)`, so an **absent** `density`
keeps the slot preset's own value (`cactus` stays 0.35) while a **rejected**
one lands on a literal `1`. Every other numeric field falls back to the
preset in both cases. Nothing but a golden run would have found it.

**Corrections to `ASSET_LIBRARY_SCOPE.md`, recorded there:**

- Milestone 4 is **not** "the first milestone with a cross-crate dependency"
  — milestone 3 is. `pickWeightedVariant` falls through to `pickIconVariant`,
  which is `hash`, so `cartalith-assets` now depends on `cartalith-noise`.
  Reimplementing that hash to preserve milestone 1's standalone property was
  the worse trade: `cartalith-noise::hash` carries two hard-won JS float
  subtleties that cost golden-test failures to find.
- `pickIconVariant` shipped here rather than in milestone 4 (three lines, and
  `pickWeightedVariant` cannot be golden-tested without it), as did
  `spaceOf`'s half of fix 2, as `ScatterRule::spacing_cells`. Leaving half of
  a named fix to a later milestone would have made it untestable here.
- `biomes` is `Vec<f64>`, not `Vec<i32>` — a consequence of
  `Number.isFinite` not coercing, so milestone 4's `biomeOk` compares against
  `biome[i] as f64`.
- Milestone 4's own two v1.27 fixes (most-specific-first priority sort,
  `requireWetland` ANDed with the biome test) confirmed to live inside
  `placeMapIconsRuled` and remain its own work.

**Verified:** `cargo build -p cartalith-assets` and `--workspace`,
`cargo test -p cartalith-assets` (24 new tests: 11 golden + 4 hardening + 9
unit), `cargo clippy -p cartalith-assets --all-targets` clean,
`cargo test --workspace` with no regressions.

## Phase 4 milestone 4: rule-driven icon placement, both v1.27 fixes confirmed structurally necessary (2026-08-17)

`ASSET_LIBRARY_SCOPE.md` milestone 4. New module `placement` in
`cartalith-assets`: `place_map_icons_ruled` (the reference's
`placeMapIconsRuled`, line 7194), `icon_slot_for_item` with the `TREE_SLOT`/
`SCATTER_SLOT` legacy fallback maps (7289-7300), and `sprite_draw_rect`
(12173). The first milestone in this crate with real golden-parity
*placement* surface: every random draw is `hash(x,y,seed±k)` on a cell
coordinate, so a port either lands icons on the identical cells with the
identical sizes, or it does not — no tolerance to argue about. Still wired to
nothing.

**The reference's legacy `placeMapIcons` body is out of scope, deliberately.**
`placeMapIconsRuled` is reached only when the caller passes non-empty
`opts.rules`; the untouched v1.25 hard-coded biome-switch path that runs
otherwise is not ported here — nothing in the milestone 4 scope calls for it,
and `current_scatter_rules` (milestone 3) already reproduces the empty-table
condition under which the reference would fall through to it. `iconSlotForItem`
is still ported in full, including its legacy `cat`/`kind` branches (the
`TREE_SLOT`/`SCATTER_SLOT` maps milestone 3's own corrections flagged as
needed here but not yet named): it is the one function that has to agree with
a legacy-shaped item's slot spelling even though this crate produces none.

**Both of milestone 4's own v1.27 fixes, ported and checked for whether they
transfer, the same scrutiny milestone 3 applied to its three** (one of which
it found structurally unreachable in Rust). Both of these do transfer —
they are real logic bugs, not JS-coercion artifacts:

1. **Most-specific-first priority sort** (reference lines 7250-7259). Before
   v1.27, a contested cell's winner was whichever rule happened to be
   inserted first in the array the caller built — which, since the table
   comes from iterating an object, meant "whichever order the user happened
   to add assets to the Library in." The fix sorts by `specificity` (fewest
   matching biomes = most specific; a wetland-requiring rule's contribution
   is offset below a non-wetland rule's; an empty biome list — "any land" —
   sorts last) before the first-match-wins loop runs. **Structurally
   necessary in Rust too**: nothing about ownership or types removes
   insertion-order dependence from a `Vec` any more than from a JS array.
2. **`requireWetland` ANDed with the biome test, not substituted for it**
   (reference line 7273). v1.26's scatter branch let `requireWetland` outright
   *replace* the biome test, so a rule with both a biome list and
   `requireWetland` ticked silently discarded the user's biome selection —
   any wetland cell matched, regardless of biome. **Structurally necessary in
   Rust too**: this is a predicate-logic defect, not a consequence of JS's
   type coercion or object-aliasing semantics (the two mechanisms behind two
   of `scatter.rs`'s three v1.27 fixes) — a straight transcription of the old
   "replace" logic reproduces the bug in any language.

Proven with a hand-traceable fixture rather than left to a broad sweep's
chance coverage: a 3×1 grid, `sea=-1` (every cell is land), `tGap=1`. That
last choice is the trick — `hash(*)` is always in `[0,1)`, so
`(hash(gx,gy,seed)*1)|0` is always `0`, meaning the scatter grid's jitter
degenerates to zero and `jx=gx, jy=gy` exactly for every cell (confirmed
against the real reference `hash`, not assumed). Cell 0 is wetland+grass,
cell 1 is dry+grass, cell 2 is wetland+shrub. Three rules — `wetland_grass`
(wetland AND grass), `narrow_biome` (grass only), `generic_land` (any land) —
inserted **least-specific first**, resolve to `wetland_grass` / `narrow_biome`
/ `generic_land` at the three cells across three seeds, and the outcome is
identical when the whole array is reversed. That third result is the fix 2
proof specifically: cell 2 is wetland (would have satisfied `requireWetland`
under the old OR/replace semantics) but its biome is wrong, so `wetland_grass`
must be rejected and the cell falls through to `generic_land`.

**Golden-verified against the real reference** (transient Node `vm` harness
over the frozen HTML, same technique as milestones 1-3, harness not checked
in). A synthetic 10×8 grid (single circular elevation peak, biome cycling
through `(x*3+y*5)%14`, wetland mask on `(x+y)%4==0`) run through an
eight-rule table across six sea/seed/density configurations matches
cell-for-cell, key-for-key, and size-for-size to 1e-9. One configuration
(`sea=0.2, tGap=2`) exercises every rule family in one run — both relief bands
sharing one bucket grid (including an unbounded `elevMin:null` relief rule),
three different scatter specificities picking different winners at different
cells, and `ghost_biome` (`biomes:[5.5]`) placing **nothing**, anywhere, in
any configuration — direct evidence that `biomeOk`'s `biome[i] as f64` cast
works: a non-integer rule biome is finite (so nothing rejects it at the
normalizer boundary) but simply never equals an integer `BIOME_INDEX`.

**Corrections to `ASSET_LIBRARY_SCOPE.md`, recorded there:** none found to
milestones 5-7's scope on this read; `TREE_SLOT`/`SCATTER_SLOT` were already
flagged as milestone 4's own remaining work by milestone 3's corrections, and
that is exactly where they landed.

**Verified:** `cargo build -p cartalith-assets` and `--workspace`,
`cargo test -p cartalith-assets` (23 new tests: 12 unit + 11 golden-parity placement +
`icon_slot_for_item` + `spriteDrawRect`, plus unit tests for the empty-grid
guard, the `t_gap=0` clamp, `biome_ok`, and the specificity ordering),
`cargo clippy -p cartalith-assets --all-targets` clean, `cargo test
--workspace` with no regressions.

## Phase 3 milestone 4 follow-up: overlays learn about the plate frame (2026-08-17)

Closes the one limitation milestone 4 flagged and deliberately did not
reach for. That milestone gave the raster a physical plate frame (bare-paper
margin plus a thick and a thin neatline); four things drawn *over* that
raster knew nothing about it, so anything near the sheet edge painted onto
what is supposed to read as blank paper. In the real app at 2048²/seed
12345/Classic, a capital's marker ring hung entirely off the plate, three
smaller markers sat on the cream margin, and roads ruled straight across it.

**The choice, since there were two real options and they are not
equivalent.** `TERRAIN_APPEARANCE_SCOPE.md`'s own milestone-4 section
records the reasoning in full; the short version is that *insetting* the
overlay coordinate space — remapping world cells into the plate interior —
is the wrong shape for this frame. The frame composites **over** the
finished raster's outermost cells; it does not shrink the map into a margin.
The terrain under the margin is covered, not moved, so an inset marker would
be displaced from the coastline and river it belongs to. Everything drawn
from world data is therefore handled at the *neatline*, not by remapping:

- **Linear features are clipped.** A road or sea lane that reaches the sheet
  edge genuinely continues past it, and cutting it at the neatline is what
  an atlas plate does.
- **Point symbols are placed or omitted, never sliced.** A settlement whose
  cell lies under the frame has no visible terrain beneath it at all, so its
  marker points at nothing; it is off-plate and is not drawn. One whose
  centre is inside keeps its position and lets the clip trim any overhang —
  which is the actual defect, markers landing *partly* on the margin.
- **Raster tints fade with the frame rather than being cut**, using the
  frame's own soft edge, so a river does not stop one cell short of where
  the wash does.

**Built.**

- `render.rs` gains two `pub fn`s and becomes the single source of the
  frame's geometry: `border_width_cells` (frame width in cells, `0.0` when
  disabled) and `border_cover` (how much of a cell the frame covers, `0.0`
  through the whole interior, ramping to `1.0` under the margin using the
  same `smoothstep` edge `apply_border` composites with). `apply_border` was
  refactored onto both rather than keeping its own copy of `0.014 * gw`.
- `lib.rs` — all three of this crate's over-raster products now fade by
  `1 - border_cover`: the river channel-mask tint in `build_color_texture`,
  the per-faction wash in `build_territory_texture`, and the line in
  `build_province_boundary_texture`. The last two were not in milestone 4's
  flagged list and were found while fixing it — territory is a solid wash
  over every owned cell, so any faction reaching the sheet edge coloured the
  margin outright.
- `WorldGen::get_border_inset_frac()` — the frame width as a **fraction of
  texture width**, the one new `#[func]`. A fraction rather than a cell
  count deliberately: `map_overlay.gd` works in screen pixels against a
  letterboxed texture, so a fraction survives the fit maths with no
  resolution knowledge on the GDScript side, and nothing hardcodes `0.014` a
  second time.
- `map_overlay.gd` — `_interior_rect()` alongside the existing
  `_displayed_rect()` (the frame is inset by the same cell count on all four
  sides and the fit scale is uniform, so one pixel inset serves both axes).
  `_draw` scissors the canvas item to that rect via
  `RenderingServer.canvas_item_set_clip`/`canvas_item_set_custom_rect` —
  one scissor for all four primitive types rather than hand-clipping
  circles, arcs, polylines and dashed lines separately. `Control` re-sets
  both from its own rect on every `NOTIFICATION_DRAW`, which fires
  immediately before `_draw()`, so the override lasts exactly one frame and
  needs no restore. Settlements whose centre falls outside the interior are
  skipped, in `_draw` **and** in `_gui_input`'s hit test on the same
  predicate — an invisible marker must not still fill the Inspector's "WHY
  HERE?" panel from a hover over blank paper. The hover card now clamps into
  the interior instead of the control bounds, so the scissor can never slice
  it.
- `main.gd` passes the new value through `set_civ_data`, whose extra
  parameter defaults to `0.0` (no frame → old behaviour exactly).

**No-op without a frame, by construction — the same gate every milestone-2/3/4
stage uses.** `border_cover` returns `0.0` everywhere when
`border_width_frac == 0.0` (`js_reference()`'s state), each raster call site
is written as `tinted + (plain - tinted) * cover` so `cover == 0.0` restores
the old value *bit-exactly* rather than to within an ulp, and
`_border_frac == 0.0` makes `_interior_rect()` return `_displayed_rect()`
unchanged and skips the scissor entirely. `golden_parity_render.rs` is still
completely unmodified and both tests still pass at their original `1e-4`
tolerance.

**Verified — by looking, since that is how the defect was found.**
`cargo build -p cartalith-godot` clean; `cargo test --workspace` 0 failures
with no expected value modified; `cargo clippy -p cartalith-godot
--all-targets` now fully clean (this pass also cleared the pre-existing
`needless_borrow` at `lib.rs:317` that milestone 4's entry recorded as the
crate's sole remaining warning); `godot4 --headless --quit main.tscn` clean
load. Real windowed app, controlled before/after: the same 2048²/seed
12345/Classic world generated twice, once with the fix stashed and once with
it applied, screenshotted with `PrintWindow` and cropped 4× at the west
edge. Counting overlay ink inside the frame band (the specific failing case,
not a general impression):

| | marker orange on margin | river-tint cyan on margin |
|---|---|---|
| before | 268 px | 67 px |
| after | **0 px** | **0 px** |

Differences between the two runs are confined to the frame band and to
within 4 px inside the neatline — nothing in the plate interior moved.
Archipelago (35 settlements, sea routes on) gives 0 px on both counts too,
and shows the two rules working together: a coastal capital whose centre is
just inside the interior is trimmed cleanly at the neatline while the sea
lanes are cut there. Territory fill, province boundaries, settlements,
roads, sea routes, hover card and the Inspector's causal-chain "WHY HERE?"
panel were all exercised on the real app after the change.

## Phase 4 milestone 5: the Library model, AssetDB/AssetCollections/AssetValidator (2026-08-17)

`ASSET_LIBRARY_SCOPE.md` milestone 5. New module `library` in
`cartalith-assets`: `AssetDB` (frozen-vocabulary bootstrap, custom-slot
add/rename/remove, lazy scatter-rule attach, item store), `AssetCollections`,
`run` (`AssetValidator.run()`), and the `assetlib/library.json` record shape
(`LibraryFile`/`SlotRecord`/`ItemRecord`, `parse_library_json`,
`AssetDB::to_library_json`/`apply_library_file`). Pure data management; no
images — every `LibraryItem` carries a caller-supplied `hash: String` rather
than one computed from pixels, which is what keeps the validator's
duplicate-image detection fully implementable without an image decoder.
Depends on milestones 1 (`Family`/`slug_id`) and 3 (`ScatterRule` and its
normalizer/presets/key function), both used directly. Still wired to
nothing.

**How this lines up with `SAVEFILE_COMPAT.md`'s existing cross-reference.**
That document already lists "an Asset Library payload" among the entries its
MVP reader ignores, noting "there is nothing in the port to deserialise them
into yet." `LibraryFile` is that something now: `{version, kind, pack:
{name,author,license}, collections:{name->[uid]}, slots:[{fam,id,name,meta,
items:[{img,name,t}],set?,rules?}]}`, field order matching a real
`_alExportEntries()` export exactly. `SAVEFILE_COMPAT.md` needed no
correction — `cartalith-io` still deserialises nothing here, by design (the
same "packs are optional, the save loader is not" reasoning milestone 2
already used to keep the dependency pointing the right way) — only a real,
tested shape now exists for a later milestone (6/7, or a future
`cartalith-io` extension) to read into.

**Two real corrections to this document's own §4, found by reading the
reference rather than assumed from milestone 1's framing:**

1. **Per-slot display *names* are not purely presentational.** §4 filed
   `mkSlots`'s `name`/`desc`/`code` columns as UI-only text; true for
   `desc`/`code`, false for `name` — `AssetValidator.run()`'s "Identical
   images" warning renders `slot.name`, confirmed by a golden run:
   `"Identical images: Mountain#1 = Hill#1"`, not `mountain#1 = hill#1`.
   Ported as `slot_title`, a 65-entry table across the six frozen families
   whose slots ever appear in that message.
2. **The Library's own `poi` vocabulary is ten slots, not `PACK_POI_SLOTS`'
   eight.** `AssetDB` bootstraps from the Asset Library's own `FAMILIES`
   table, which is a *different* constant than the pack-import vocabulary
   `parsePackManifest` validates against, and `FAMILIES[...].poi.slots`
   carries `lake`/`bridge` in addition. Both lists now exist
   (`LIBRARY_POI_SLOTS`, ten; `PACK_POI_SLOTS`, eight, unchanged) rather than
   one "fixing" the other — the same `lake`/`bridge`-authorable-but-never-
   loads inconsistency §1 already named, now visible in two constants
   instead of one.

**The id-slugging and uid-collision hardening the milestone asked for by
name — real, checked for rather than assumed, and ported with tests.**
`addCustomSlot` returns the *existing* slot on a uid collision rather than
creating a duplicate (`const existing=fam.slots.find(...); if(existing)
return existing;`); `renameCustomSlot` refuses a colliding rename outright,
keeping the *old* uid (`if(SLOT_REG[nuid]) return uid;`). Unlike v1.27's
three fixes, neither carries a version-tagged reference comment — reported
here as a finding, not a named historical fix — but both are real
defences against untrusted, free-form user text (a custom slot's `id` is
[`slug_id`] of whatever the author typed) colliding on one slug, which is a
real hazard for content editable outside the app rather than a
hypothetical. Pinned in `tests/hardening_asset_db.rs`, which also documents
a companion finding: two of `run`'s six checks — "Duplicate identifier" and
"Invalid filename id" — are structurally unreachable through this module's
own public API in *both* languages, for a reason that is not "Rust's type
system" (the same shape of surprise milestone 3's fix #3 found for the
`Object.assign` aliasing bug). Ported anyway, faithfully, as real
defence-in-depth. A third check, "Collection references a missing asset,"
*is* reachable, but only via `AssetCollections::from_map`'s deliberately
unchecked assignment (mirroring `AssetCollections.map=lib.collections||{}`
in `_alImportProject`) — `remove_custom_slot` already cleans up membership
before the validator could see a stale reference through ordinary editing.

**Golden-verified against the real reference** — `AssetValidator.run()`
turned out to be exactly the "strong golden-verification candidate" the
scope document suggested. A transient Node `vm` harness (same technique as
milestones 1-4, not checked in) ran the real `AssetDB`/`AssetCollections`/
`AssetValidator`/`_alExportEntries` on twelve constructed library states —
empty, one item, duplicate hashes across two and three slots, the
grass-splat hint present and absent, an empty custom slot, a stale
collection reference reached the one real way, and a "kitchen sink"
combining several warnings to pin the reference's exact warning order.
`to_library_json()`'s shape was checked the same way across five more
scenarios (pack fields, a bare frozen slot, a tagged-but-empty custom slot
kept alive by `fam.custom`, a tagged-but-empty frozen slot kept alive by its
tags, a frozen slot with neither excluded entirely, collections
round-tripping verbatim, and the whole-library-empty `None` case). Every
case matched on the first run.

**Deliberately not restored by this milestone**: `apply_library_file`
restores pack info, collections, and per-slot metadata/scatter rules from a
parsed `LibraryFile`, but not items — `SlotRecord.items` carries everything
a real reader has except pixels (`img` index, name, transform), left for
milestone 6 to pair with decoded `assetlib/img/<idx>.png` bytes and a real
`itemHash`. `normalizeScatterRule`-on-load happens eagerly during *parsing*
rather than at apply time, since a record's own `fam`/`id`/`set` are enough
to compute its rule key without touching the live registry.

56 new tests (23 unit + 32 golden-parity + 7 hardening — the hardening file's
scenarios deliberately overlap two of the golden file's, pinning the same
behaviour once as "matches the reference" and once as "and here is why it
matters").

**Corrections to milestones 6-7's scope:**

- Milestone 6's "`itemHash` duplicate detection" is already implemented
  (`duplicate_groups`/`slot_has_dupe`); milestone 6 only needs to supply a
  real hash string via `AssetDB::add_item`, not reimplement grouping.
- Milestone 6's "per-item transform" already has its data shape
  (`ItemTransform`); `fitToBottom` remains milestone 6's own pixel-dimension
  computation, but the field and its `library.json` round trip do not need
  redesigning.
- Milestone 6 needs to wire real item restoration into
  `AssetDB::apply_library_file` (or a wrapper around it): decode each
  `SlotRecord.items[].img`-indexed PNG, hash it, and call `AssetDB::add_item`
  with a `LibraryItem` built from the record's own `name`/`t`.

**Verified:** `cargo build -p cartalith-assets` and `--workspace`,
`cargo test -p cartalith-assets` (74 lib unit tests, 32 golden-parity, 7
hardening, plus milestones 1-4's existing suites unchanged),
`cargo clippy -p cartalith-assets --all-targets` clean, `cargo test
--workspace` 0 regressions across every crate.

## Phase 4 milestone 6: image handling, real pixels via the `image` crate (2026-08-17)

`ASSET_LIBRARY_SCOPE.md` milestone 6. New module `raster` in
`cartalith-assets` (no feature gate — `image`'s own `default-features =
false` + `png`-only already keeps its footprint small, and nothing in this
crate needs an image-free build the way `archive`'s callers might want a
zip-free one). First milestone that touches pixels.

**Narrower than the milestone's own original description, exactly as
milestone 5's own corrections already called it.** The transform *shape*
(`ItemTransform`) and the duplicate-detection *machinery*
(`duplicate_groups`/`slot_has_dupe`) both already existed. What this
milestone actually built: real PNG decode/encode (`decode_png`/
`encode_png`), a real content hash from decoded pixels (`item_hash`), the
transform math itself applied to pixels rather than merely represented
(`fit_to_bottom` mutates the transform; `render_item` is what actually
composites scale/pan onto a canvas), thumbnail and pack-export bake
(`render_item` again — the reference's own single shared function for
both, per `ThumbnailRenderer`'s own architecture comment: "shared render
core (thumbnails, inspector preview, export bake)"), `finalizePackTexture`'s
inverse means (`finalize_pack_texture_inv_mean`), and wiring decoded items
into library restoration (`AssetDB::apply_library_file_with_items`).

**Crate work (`image`) plus a thin port, per the scope doc's own framing.**
`image = "0.25.10"`, `default-features = false`, `features = ["png"]` —
every asset this crate reads or writes is a PNG (every pack entry, every
`assetlib/img/N.png` project entry), so the rest of `image`'s format zoo
(gif/jpeg/webp/tiff/avif/exr/…) and its rayon/simd extras are dead weight
never called here. Not present anywhere else in the workspace before this
milestone; `default-features = false` verified to actually drop the extra
codec dependencies (compared the dependency tree with and without the
override before committing to it).

**`itemHash`'s real algorithm, read before porting anything, and a real
compatibility decision rather than an assumption.** `itemHash(img,w,h)`
(reference line 26913) downsamples through `ctx.drawImage(img,0,0,32,32)`
on a canvas, then runs a stride-7 FNV-1a variant (offset basis
`0x811c9dc5`, prime `0x01000193`, 32-bit wrapping multiply) over the
resulting pixels, appending `-{w}x{h}` (the item's *original* dimensions,
not the thumbnail's). The hash constants and stride are ported verbatim.
**Not golden-verified against a captured browser hash — a real, checked
decision, not a gap**, for two reasons found by reading rather than
assumed:

1. **Never serialized, on either side.** `_alExportEntries` writes
   `{img,name,t}` per item (line 27890) — no `hash` field — and
   `_alImportProject` **recomputes** `hash:itemHash(img,w,h)` fresh after
   its own decode (line 27922) rather than reading one back from a file.
   No process ever compares its hash against another's.
   `crate::library::ItemRecord` already reflected this before this
   milestone ever named the reason — it shipped in milestone 5 with no
   `hash` field at all.
2. **Could not match even if it needed to.** `ctx.drawImage`'s resample
   kernel is implementation-defined per the HTML5 Canvas spec — two
   *browsers* need not agree on it, so "matches the reference" was never a
   coherent bar here.

`item_hash` is therefore real, deterministic content hashing (`image`'s
`Triangle` filter standing in for the browser's unspecified resample),
verified with real unit tests for the property that matters: same decoded
pixels in, same string out; different pixels or different original
dimensions, a different string out.

**`finalizePackTexture`'s "inverse means" — checked against the real
reference rather than assumed to be some reversed baking transform, per
the task's own instruction to verify exactly what before porting. The
literal reading holds**: the mean of each of R/G/B across every pixel,
clamped to never read below 1 (`Math.max(1,mean)`), then reciprocated.
Ported as `finalize_pack_texture_inv_mean(w,h,rgba) -> [f64;3]`. Pure
arithmetic, no DOM — unlike `item_hash`, this one **is** golden-verified
against the real reference (the same transient Node `vm` technique every
earlier milestone used), six fixtures matched exactly including `n==0` and
a mean-below-1-clamped case. `fit_to_bottom` is the milestone's other
DOM-free function, golden-verified alongside it with seven fixtures.

**`render_item` ports the reference's own shared render core**
(`drawItemOnly`/`renderItem`) as one function for thumbnail, inspector
preview, and pack-export bake alike, matching the reference's own
architecture. Geometry (position, size, source-over alpha compositing) is
exact; only the resampling kernel (`image`'s `CatmullRom`, standing in for
`imageSmoothingQuality:'high'`) is not reference-identical, for the same
reason `item_hash`'s is not.

**Why the split between golden-parity and real unit tests**: every prior
milestone's golden tests lift real reference functions into a headless
Node `vm.runInContext` sandbox, which has no `document`/`Canvas`/`Image`/
`Blob`. `finalizePackTexture` and `fitToBottom` are the only two functions
in this milestone's scope with no DOM dependency, so those two are
golden-verified; `item_hash`/`render_item`/`decode_png`/`encode_png` are
real unit tests, documented as such in `src/raster.rs`'s own module docs.

**`AssetDB::apply_library_file_with_items`**, the milestone-5-flagged
wrapper: calls `apply_library_file` (unchanged, still covered by its own
tests), then for each item whose PNG bytes the caller supplies (keyed by
`img` index — reading them out of a project `.zip` is the caller's job),
decodes, hashes for real, and `add_item`s a `LibraryItem` built from the
record's own `name`/`t`. A missing or undecodable image is skipped
silently without failing the rest — the reference's own
`try{...}catch(_){}` (line 27920-27923).

**A real non-port, named rather than silently skipped**:
`AssetImporter.importPackZip` (decoding a whole *external pack* into
`AssetDB`, distinct from restoring a previously-*exported project* via
`apply_library_file_with_items`) was not built — the driving task named
project restoration specifically, and every piece `importPackZip`'s
equivalent would compose already exists for whoever picks it up next
(`PackManifest`, `PackEntries`, this milestone's decode/hash/transform
functions). Not a correction to milestone 7, which does not need it.

15 new tests (10 raster unit + 3 library unit + 2 golden-parity). Still
wired to nothing.

**Corrections to milestone 7's scope: none.** Its boundary — renderer +
Godot integration, only-then UI, sprite-sheet slicer and Library page UI
both already out of scope — is unchanged by what this milestone's real
implementation surface turned out to be.

**Verified:** `cargo build -p cartalith-assets` and `--workspace`,
`cargo test -p cartalith-assets` (87 lib unit tests total — 74 carried over
from milestones 1-5 plus this milestone's 13 new ones, 10 in `raster` and 3
in `library`; plus the new `tests/golden_parity_raster.rs`, 2 tests; every
earlier milestone's golden/hardening suite unchanged), `cargo clippy -p
cartalith-assets --all-targets` clean, `cargo test --workspace` 0
regressions across every crate.

## Phase 4 milestone 7 — renderer + Godot integration, closing Phase 4 (2026-08-17)

`ASSET_LIBRARY_SCOPE.md`'s final milestone: sprite compositing and
ground-texture splat sampling, in a new `cartalith-godot::pack` module — the
first thing in the workspace to depend on `cartalith-assets` (its own doc
comment said "nothing depends on this yet" until now).

**Sprite compositing** (`composite_map_icons`, `drawMapIcons`'s own
Y-sorted painter's pass): a real scatter-rule table is built from a loaded
pack's manifest (`autopopulate_scatter_rules`); a `BIOME_INDEX` raster and a
wetland mask are derived at render time from the already-generated height/
temperature/rainfall fields — presentation-side computation, no new
world-generation data, same category `render.rs`'s own `material_weights`
already is (`cartalith_civ::classify_biome`, already golden-verified
elsewhere in the workspace, plus a `buildWetlandMask`-equivalent; one
honest simplification: every water cell is `BIOME_OCEAN`, since this port
has never built the lake/ocean flood-fill classifier `buildBiomeRaster`
uses, and none of the ten frozen icon presets target the lake index).
`place_map_icons_ruled` places the icons; each is then composited: a real
bilinear-sampled blit (hand-written, not a new `image`-crate dependency in
this crate — the icons involved are small enough that a manual sampler is
the smaller, sufficient tool) at `sprite_draw_rect`'s destination geometry
where the pack has real art for that slot, a real per-slot procedural glyph
fallback (`draw_icon_glyph`) otherwise — all ten `PACK_ICON_SLOTS` shapes
(mountain/hill/six tree kinds/cactus/boulder), "shrub" doubling as the
reference's own documented catch-all for an uncovered custom asset. Two
purely-decorative reference variants are dropped (the arid jagged hill
outline, the cold-mountain snow-cap) since the reference itself calls them
"procedural-fallback variety only" layered on an unconditional base
silhouette, which is what's ported.

**Ground-texture splat** (`land_color`'s new branch, `render.rs`): the six
`SPLAT_PAINT_SLOTS` channels, decoded and inverse-mean-baked at load time
(`finalize_pack_texture_inv_mean`, milestone 6's own function, wired to
something real for the first time), blended per-cell using the *exact*
`materialWeights` fractions and each material's own procedural ramp colour
`land_color` already computes — no new logic, splat is a read-only consumer
of both, exactly mirroring the reference's own `sp()` accumulator.

**The two Cartography "painted layers" are deliberately not implemented
this pass — a real scope finding, not an oversight.** Read literally
(reference lines 7898-7900, 12187-12196): `pBio`/`pTer` are per-cell
indices into `state.cartoPaint.biome`/`.terrain`, sparse arrays a manual
paint-brush tool populates (`paintBiome`/`paintSplat`/`paintTerrain` module
globals). This port has never ported that tool — there is no producer of a
painted-cell array anywhere in the workspace, and building one from scratch
is itself a real, separate UI+state effort this milestone's own "no GUI
controls" boundary rules out. Unlike splat (gated only by
`assetPack.texAny`) and icons (gated by `state.viz.icons`), the painted
layers are gated by a *third* piece of state this port has no producer for
at all — so `LoadedPack` parses `.biomes`/`.terrains` from the manifest but
never decodes or rasterises them. Named as the natural next item for
whoever ports the Cartography paint-brush tool.

**Two real reference defaults confirmed by reading, not assumed.**
`state.viz.icons` defaults `false` — icons are an opt-in `state.viz.*`
stretch feature like every other one `render.rs`'s own doc comment already
excludes, so a pack-less *or* icon-toggle-off render was always
bit-identical, and `current_scatter_rules` returning `None` whenever no
pack supplies real icon art is `composite_map_icons`'s own early return,
reproducing exactly that no-op. `state.viz.splat` defaults **`0.7`** — the
opposite shape, gated purely by `assetPack.texAny`, real and active the
instant a pack with real ground textures loads, no toggle involved at all.
Both are genuinely additive/opt-in rather than JS-parity-gated stretch
features (there is no pack-less version of "blend in a texture that
doesn't exist" to stay bit-identical with) — confirmed by
`golden_parity_render.rs` passing **unmodified** at its original `1e-4`
tolerance, since `RenderCtx.splat` stays `None` on that path (`with_splat`
is a builder method the test never calls).

**This port confirmed to ship no default asset pack** (nothing in
`godot-project/` bundles pack art), so real compositing has nothing to
composite by default, exactly as the milestone's own scope anticipated.
Real, permanent plumbing added for it rather than a throwaway stand-in:
`WorldGen::load_asset_pack(path) -> bool` (a native filesystem path, same
convention as `load_save`) and `WorldGen::has_asset_pack() -> bool` — real
`#[func]` API surface with **no GDScript call site anywhere**, dormant code
for a future importer or `GUI_SHELL_SCOPE.md` pass, not a GUI control.

**Verified three ways.** A new `cartalith-godot/tests/pack_compositing.rs`
loads the real `reference_pack.zip` fixture milestone 2 golden-verified
against the reference's own exporter (reused, not reinvented) and proves,
on a small synthetic world: real sprite art blits where a relief-mode
mountain places one; the procedural glyph fallback fires for a biome region
the fixture has no art for at all; and a pack whose manifest has no icon
slots places nothing at all — the same "keeps `placeMapIcons` on its
legacy/no-op path" condition `current_scatter_rules`'s own doc comment
names as what keeps a pack-less render bit-identical. Static: `cargo build
-p cartalith-godot` and `--workspace`, `cargo test --workspace` (0
regressions, `golden_parity_render.rs` unmodified and still passing at its
original tolerance, 3 new integration tests plus the untouched suite),
`cargo clippy -p cartalith-godot -p cartalith-assets --all-targets` clean
(the rasterizer's loose `bytes`/`gw`/`gh` argument triples were refactored
into a small `Canvas` struct along the way, both for clippy's
`too_many_arguments` and because it reads better), `godot4 --headless
--quit main.tscn` clean. Real windowed: launched the actual
`Godot_v4.7.1-stable_win64.exe`, generated a real 512² world, called
`load_asset_pack` against the real fixture (temporary `main.gd` debug calls
only, reverted before commit — the shipped diff carries no GDScript
changes at all), and saved the native output `Image` directly to disk
(`Image.save_png`) for full-resolution inspection rather than relying on a
scaled-down window screenshot, since a 512² world's icons are only a
handful of pixels wide on screen. **Confirmed by actually looking at it**:
a sharp-edged, flat-coloured rectangular block sits on land exactly where a
relief-mode mountain would place one — real pack sprite art, not a
procedural blend (which is always noisy/gradient, never a hard-edged
rectangle); a large irregular checkerboard region follows real
land-material boundaries rather than sitting in a fixed box — real
per-pixel splat sampling, not a sprite; and small soft-edged translucent
blobs appear elsewhere on plain terrain, consistent with the procedural
glyph fallback rendering where the fixture pack has no matching art.

**Phase 4 is genuinely complete — all seven milestones done.** Checked
honestly against `ASSET_LIBRARY_SCOPE.md` §8's own "done means", written
specifically to give this phase an operational finish line beyond
`ROADMAP.md`'s one-sentence description: a real `.zip` pack authored
outside the app can be imported, validated with the reference's own
warnings, and rendered onto the map — sprites where the pack carries them,
procedural art where it does not — with a pack-less render staying
bit-identical to today's. That bar is met. The one explicit carve-out in
that same sentence — the Library-*authoring* workspace — is not part of
Phase 4's own definition of done; it is `GUI_SHELL_SCOPE.md`'s own future
work, same as the Cartography paint-brush tool this milestone found and
named above.

**Files touched:** `cartalith-native/crates/cartalith-godot/Cargo.toml`
(new `cartalith-assets` dependency), `cartalith-native/crates/
cartalith-godot/src/pack.rs` (new), `cartalith-native/crates/cartalith-
godot/src/render.rs` (`SplatChannel`/`SplatTextures`, `splat_sample`,
`land_color`'s splat branch, `TerrainAppearance::splat_strength`,
`RenderCtx::with_splat`), `cartalith-native/crates/cartalith-godot/src/
lib.rs` (`WorldGen::load_asset_pack`/`has_asset_pack`, `seed`/`asset_pack`
fields, splat/icon wiring in `build_color_texture`), `cartalith-native/
crates/cartalith-godot/tests/pack_compositing.rs` (new). `godot-project/
main.gd` untouched in the final diff — the verification-only debug calls
described above were reverted before commit.

## DCC shell milestone 1: full structural replacement of the panel-browser shell (2026-08-18)

`DCC_SHELL_SCOPE.md`'s milestone 1, picked up mid-flight: a prior fork had
already done the real rebuild work in the working tree (`main.gd`/`main.tscn`/
`map_overlay.gd`, all uncommitted) before being cut off by an account-level API
error with no recoverable transcript. This pass's first job was assessing that
work rather than assuming either "nothing works" or "everything is done" —
`git diff` against all three files, a full read of `main.gd`, `main.tscn`, and
`map_overlay.gd`, then real verification, found the prior fork's work
**substantially complete and structurally sound**: all six regions from
`UI_SHELL_DESIGN.md`'s governing table built as real Godot scenes/Control
nodes, every currently-real control re-parented, the eight-menu bar content
change done correctly (not just a rename — Edit/Help genuinely new, Generate/
Simulate/Render/Assets restructured per the design doc's own table), the
click-to-pin Properties dock (`GUI_FEATURE_PARITY_SCOPE.md` Category-1 item
#10) and the three-way independent layer-toggle split (item #9) both real and
wired, File > Import asset pack wired to the real `load_asset_pack`/
`has_asset_pack` (item #1). `cargo build -p cartalith-godot` and
`cargo test --workspace` both passed clean on first attempt — the prior
fork's Rust-facing GDScript (function names, signal shapes, dictionary keys)
matched `lib.rs` exactly with zero corrections needed.

**One real gap found and fixed**: the status bar's own `StatusHintLabel` —
`UI_SHELL_DESIGN.md`'s "the active tool's modifier hints" slot — had no
`unique_name_in_owner` and was never touched by `main.gd`, so it stayed
hard-coded "no active tool" even after selecting a tool from the rail, while
the Tool Options Bar correctly showed the tool's name. Left alone, this would
have been a real inconsistency between two chrome regions describing the same
state, not merely an unfinished stretch feature — fixed by wiring
`_on_tool_selected` to set `%StatusHintLabel` honestly (`"RAISE / LOWER
selected -- no pass-buffer/commit/discard yet"`), matching the same
"visible, not hidden; honest, not silent" discipline every inert item in the
new menus already follows.

**Judgment call — tool-options-bar/status-bar honesty**: the prior fork had
already made the right call here, worth recording explicitly since the task
asked for it to be judged carefully. The Tool Options Bar shows no live
per-tool parameters at all — just the selected tool's name and one hint line
("no live tool parameters -- tool system not implemented yet
(DCC_SHELL_SCOPE.md milestone 2/3)") — rather than fabricating controls like
"RAISE / LOWER · commit pass" that would imply a working pass-buffer/commit/
discard model. That fabricated version would have been actively misleading
(matches the task's own framing of the risk); the shipped version is honestly
inert, consistent with every other not-yet-real surface in this shell (the
disabled Generate-menu stage items, the disabled Edit-menu Undo, etc.).

**Known pre-existing cosmetic issue, not touched**: unchecked `CheckBox`
nodes in the right dock (`Territory (faction fill)`, `Province boundaries`
before being checked) render with no visible checkbox glyph against
`theme/dark_theme.tres`'s dark fill — `checkbox_unchecked_color` is set but
Godot's `CheckBox` icon theme items are a separate mechanism this theme
resource doesn't populate. Confirmed functional regardless (clicking toggles
the layer correctly, screenshot-verified below) — this is a theme-icon gap
that predates this milestone (the theme resource itself isn't part of this
diff), not a DCC-shell structural defect, so it's noted here rather than
fixed as scope creep.

**Verification**: `cargo build -p cartalith-godot` and `cargo test
--workspace` both clean, 0 regressions (every crate's suite passing, e.g.
`cartalith-godot` 139/139, `cartalith-civ` 87/87). `godot4 --headless --quit
main.tscn` clean load, no script/parse errors. Real windowed-app screenshot
verification, end-to-end, on this session's real Windows desktop
(`Godot_v4.7.1-stable_win64.exe --path godot-project main.tscn`,
`PrintWindow`-based capture, synthetic `mouse_event`/`SetCursorPos`
automation, maximize/restore focus-forcing trick): File > New World opened
with all five real fields defaulted correctly (seed 12345, 2K, 800 km, sea
level 42%, Classic); Generate produced a real 2048×2048 world (seed 12345,
40 settlements) with terrain, settlement markers, roads, and sea routes all
rendering correctly through the new viewport; toggling Territory (faction
fill) and Province boundaries from the Layers dock rendered both overlays
correctly and independently of Settlements/Roads/Sea routes staying on;
hovering a settlement showed the on-canvas hover card and live Sample-dock
data simultaneously; clicking it pinned the same settlement's full causal
"WHY HERE?" chain into the Properties dock (`strong fresh water (1.00) →
strong gentle terrain (0.97) → strong terrain form (0.85)`, suitability,
river order/flow, distance to water, elevation, travel cost) and the pin
survived subsequent layer-toggle clicks, as designed; File > Open project
(.zip) opened the real save-file dialog rooted correctly and cancelled
cleanly without disturbing the generated world; Help > Credits opened the
real credits dialog with its full academic-principles text; selecting a
tool-rail icon (Raise / lower) updated the Tool Options Bar label, the
now-fixed status-bar hint, and the rail's own highlight together; switching
to the CIVILIZATION workspace tab correctly restyled the tab row and dimmed/
brightened tool-rail group emphasis without touching the viewport or the
still-selected tool's own highlight, per `UI_SHELL_DESIGN.md`'s "a tab
swaps... it never swaps the application, and never changes the map" rule.

**Files touched**: `cartalith-native/godot-project/main.gd` (status-bar hint
wiring, the only code change this pass added on top of the prior fork's
work — everything else was prior-fork work verified as-is),
`cartalith-native/godot-project/main.tscn` (`StatusHintLabel` gains
`unique_name_in_owner`), `cartalith-native/godot-project/map_overlay.gd`
(prior-fork work, verified unchanged). `DCC_SHELL_SCOPE.md` marked milestone
1 done.

## Generation parameters: the whole engine surface reachable from GDScript (2026-08-18)

Owner directive, verbatim: *"make all generation options active in the
current interface so that we have the same functional controls as the older
html version."* This is the Rust half — exposing every generation parameter
the engine already computes with, so the GUI can reach it. A sibling fork
built the Generate-menu dialogs against this API in parallel.

**The measured gap**: `cartalith-engine` defines eight parameter structs
(`TectonicParams`, `VolcanismParams`, `CraterParams`, `PlanetParams`,
`ClimateInputParams`, `StreamParams`, `WorldStructureParams`, `WorldParams`).
`cartalith-godot`'s `WorldGen` exposed **7** of their fields: `sea_level`,
four subsystem flags, and — only as five hardcoded named presets with no path
for raw values — the World-Structure block. Everything else was live in the
engine and unreachable. **58** are reachable now, covering all eight structs.

**Built:**

- `cartalith-godot/src/params.rs` — a flat, dotted-key parameter table
  (`"sea_level"`, `"tect.plates"`, `"climate.lat_n"`, mirroring the
  `WorldParams` field path exactly), 58 rows, each carrying `group`, `kind`,
  `min`/`max`/`step`, `label`, `unit`, `reference_control`, and a
  getter/setter function-pointer pair. Deliberately **`godot`-free** so it
  unit-tests under a plain `cargo test` with no engine process, the same way
  `render.rs` does (`cartalith-godot` is a `cdylib`).
- Six new `#[func]`s plus five read-only ones on `WorldGen`:
  `get_params()`, `get_param_defaults()`, `get_param_info()`,
  `get_param_groups()`, `set_params(Dictionary) -> {rejected, clamped}`,
  `reset_params()`, `get_gpu_stages_used()`, `get_seed()`,
  `get_villages_enabled()`, `apply_archetype(name)`, `get_archetypes()`.
- **Why one table rather than ~58 individual setters**: emitting a `#[func]`
  per field would make GDScript hardcode 58 names, ranges, steps and labels a
  second time — the exact duplication that lets a slider silently drift from
  the range the reference actually shipped. `get_param_info()` is the single
  source of truth, and the GUI builds its dialogs from it. Adding a parameter
  is one row in `params.rs` and no GDScript change at all.
- **Ranges are the reference's own**, converted through each control's real
  handler rather than invented: `alpha`'s slider is raw 0-100 mapping
  `v/100*1.2`, so the table carries `0.0..1.2` step `0.012`; `crat` is raw
  0-100 mapping `v*2`, so `0..200` step 2; and so on for the 47 parameters
  the reference genuinely exposed as user controls. The 11 it never exposed
  (`tect.lloyd`, `climate.ocean_hum`, `climate.bulk_evap`,
  `climate.current_k`, `climate.terrain_wind_deflection`, and — as *raw*
  values rather than presets — nothing else; the World-Structure five the
  reference does expose were simply unreachable in this port) are flagged
  with an empty `reference_control` rather than presented as parity —
  `DECISIONS.md` §7d: a superset is not a violation as long as the default
  reproduces reference behaviour.
- **`use_gpu` and `gpu_stages_used` surfaced** (`GUI_FEATURE_PARITY_SCOPE.md`
  Category-1 item 7, never wired before). `get_gpu_stages_used()` is
  read-only and deliberately not derivable from the flag: every stage falls
  back to CPU *individually* on failure (`HARDWARE_ACCELERATION.md` §27), so
  an empty array with `use_gpu` on means "asked for GPU, got none" — which
  the UI must be able to report honestly. Cleared on `load_save()`, since a
  loaded save was not generated by this process at all.
- **Raw World-Structure knobs reachable** (Category-1 item 8) plus
  `apply_archetype()`, which writes a preset into the *persistent* parameters
  so the five sliders then show real numbers and stay editable — the
  reference's own behaviour (its archetype segment sets the same five
  sliders, which remain live as "Custom" fine-tuning).
  `generate_world_structure()` deliberately keeps its original **one-call**
  semantics and leaves the stored parameters untouched, so `main.gd`'s
  existing pattern of alternating between it and `generate()` per menu
  selection keeps working unchanged.
- **Planet parameters reachable** (Category-1 item 6): `planet.g`,
  `planet.rotation_hours`, `planet.axial_tilt_deg` — all three real and live
  in climate, previously hardcoded for every generate.
- The three existing setters (`set_sea_level`, `set_experimental_flags`,
  `set_villages_enabled`) are unchanged in signature; the first two are now
  thin sugar writing into the same storage, so the two surfaces cannot
  disagree about a value.

**Invalid-value handling** — one policy, decided once, documented on
`params::set` and in `GENERATION_PARAMETERS.md`: unknown key → **rejected**
(and printed); wrong type → **rejected** (a `bool` parameter takes only a
real boolean, no truthy numbers; nothing is coerced from String/`null`);
NaN/±inf → **rejected** (clamping a NaN produces a NaN either way, and NaN
comparison differs between JS and Rust — one in the height field propagates
silently through every downstream stage); out of range → **clamped, applied,
and reported**; a fractional value for an int parameter → **rounded and
reported**. Clamping rather than rejecting out-of-range values is the real
choice here: every value feeds a kernel with no meaningful behaviour outside
its range, clamping matches `set_sea_level`'s own existing
`.clamp(0.0, 1.0)`, and it is *reported* in `set_params`' return, so a dialog
reads the stored value back rather than assuming its widget won.

**Verified:**

- `cargo test --workspace`: 83 test binaries, **0 regressions**, every
  golden-parity fixture unmodified. `WorldGen::params` starts at
  `WorldParams::defaults(0, 0, 0)` and `generate()` overwrites only
  `gw`/`gh`/`tect.seed`/`map_width_km`, so an untouched instance builds a
  byte-identical `WorldParams` to the one the old inline code built.
- `cargo clippy -p cartalith-engine -p cartalith-godot --all-targets`: clean.
- New `cartalith-godot/tests/params_mapping.rs`, 11 tests: every default
  round-trips through its own key and leaves `WorldParams` `PartialEq`-
  identical (the headline zero-behaviour-change check); every default lies
  inside its own advertised range; keys unique and groups contiguous;
  unknown-key rejection writes nothing; wrong-type rejection in both
  directions; non-finite rejection; out-of-range clamping; int rounding;
  partial updates touching only their named keys; all eight engine structs
  reachable; both Category-1 items reachable.
- `Godot_v4.7.1 --headless --quit main.tscn`: loads clean, extension
  initialises, and the sibling fork's `main.gd` reports
  `58 exposed by the engine, 2 deliberately excluded, 57 rows across the
  Generate menu` — the API consumed end to end, not merely compiled.

**Files touched**: `cartalith-native/crates/cartalith-godot/src/params.rs`
(new), `cartalith-native/crates/cartalith-godot/tests/params_mapping.rs`
(new), `cartalith-native/crates/cartalith-godot/src/lib.rs`,
`cartalith-native/crates/cartalith-engine/src/lib.rs` (`Clone`/`Debug`/
`PartialEq` derives on the eight parameter structs — no behaviour change),
`GENERATION_PARAMETERS.md` (new, repo root — the full per-parameter
inventory the GUI fork and every future pass works from).

**Still open**: the parameters the reference exposed for pipeline stages this
port has not ported at all (droplet/hillslope/velocity erosion, glacial,
coastal), the three structured-orogeny T5 knobs (`foldIntensity`/
`trenchDepth`/`faultBlock`, currently hardcoded inside `generate_terrain` to
the exact values the reference's own null-coalescing defaults produce), and
geoid/tides/seasons — each itemized with its reason in
`GENERATION_PARAMETERS.md`'s own "Parameters the reference exposed that this
port does not" section.

## Journey Planner milestone 4: consumption/resupply — and milestone 3 closed (2026-08-18)

Built **out of the numbered order on purpose**. Milestone 3's own
investigation found a real dependency inversion — its two stage calculators
need milestone 4's mass model — and `JOURNEY_PLANNER_SCOPE.md` now carries an
explicit build-order table at the head of its milestone breakdown. The
numbers stay as historical identifiers (they are referenced across this
changelog, `STATUS.md` and several commit messages); the table is the real
order.

**Ported (`cartalith-civ`, ~1,000 lines):**

- **The four real quick wins first**, as the scope doc suggested:
  `jp_human_water_rate` (v1.95's one per-person daily rate),
  `jp_human_water_carry_days`/`jp_animal_water_carry_days` (v1.84's
  desert-only carried-water reserve — outside an arid biome water is
  assumed collectable and contributes *zero* mass), and
  `jp_desert_tier_for_gap`.
- `jp_consumption_factors` (Pandolf terrain factors × a velocity-squared
  pace surcharge), `jp_foraging` (v1.81, food *and* water offsets),
  `jp_capacity` (the whole mass model: seasonal physiology, desert
  food/water multipliers, the phantom-draft-animal shortfall, and v1.83's
  saddlebag credit for a rider's own mount, which correctly refuses to
  double-count the "Lone courier" preset's already-declared horse), and
  `jp_assess_resupply` (v1.51's two named causes — a load problem and a
  water problem are fixed by different actions, so they cannot share one
  message).
- The hydrology/measurement helpers: `jp_water_reach_cells`,
  `jp_drinking_coarse_ease` (v1.101 Fix B — JP's own *uncapped* ease, where
  the map's `river_coarse_ease` is capped for cartographic reasons that have
  nothing to do with finding a spring), `jp_stage_dry_km`, and
  `jp_resupply_reach` (v1.51's headline audit finding: nothing had ever
  compared the resupply requirement with the settlements the route passes).
- The data milestones 2 and 3 each deliberately left out: `JP_BIOMES`'
  `water`/`forage`/`waterForage`/`grazing` columns (now one `jp_biome`
  record, with milestone 2's two-field lookup delegating to it rather than
  keeping a second copy of the same table), `JP_SEASONAL_ANIMAL`,
  `JP_SEASONAL_HUMAN`, `JP_DESERT_ANIMAL_MOD`, `JP_GRAZING`, `JP_SEASONS`,
  `JP_FORAGING`, `JP_TERRAIN_CONSUMPTION`, `JP_FORAGE_TERRAIN`, `JP_PACE`,
  `JP_INFRA`, `JP_ROUTE`, `JP_GROUP_CLASSES`, `JP_LAND_TRANSPORTS`,
  `JP_DESERT_WATER` and the vehicle/ration constants.
- **Milestone 3's two deferrals, which closes milestone 3**: `jp_calc_land`
  (hard feasibility blocks → speed chain → v1.51's supply/load/speed
  convergence loop → v1.63's and v1.67's two infeasibility cutoffs) and
  `jp_calc_water`. Both return `Result<_, JpBlocked>` rather than the
  reference's `{blocked:"…"}` sentinel object, so a blocked stage cannot be
  read as a computed one by accident. Their `formula` trace strings are
  deliberately not ported — pure presentation (`ARCHITECTURE.md`), and every
  value they print is a field on the returned struct.
- **Milestone 6's `jp_fmt_kg`**, needed here because both calculators format
  their overload/hold messages with it. The rest of milestone 6 is untouched.
- **Milestone 2's `_jpBestLandTransportForStage`**, checked against the real
  reference rather than assumed: its `eff` parameter is only ever a plan with
  per-stage overrides merged in, so `jp_calc_land` landing was genuinely all
  it needed. **Genuinely unblocked, ported.** Milestone 2's other three
  (`jp_auto_pick_transport`/`jp_auto_pick_vessel`/`_jp_best_package_for_stage`)
  re-read again and still blocked on milestone 5's plan shapes.

**The wildlife-richness question, resolved by investigation:** `jp_foraging`
reads the world's wildlife richness through `_jpWildlifeForageMod`, which
this port has never plumbed in. Checked against Phase 2's own ecology work
rather than concluded from the name: `build_npp` and
`build_carrying_capacity` are real and are *inputs* to the reference's
richness model, but they are not the same quantity — `richness` is a
per-ecoregion **species count** (`assignWildlife`'s `present.length`, a biome
species roster clipped by a species-area × energy × heterogeneity × latitude
curve), and the whole ecoregion-segmentation + roster subsystem behind it is
unported, on no Journey Planner milestone, and larger than this one. So the
input is genuinely new, and it is **caller-supplied**:
`jp_wildlife_forage_mod(region_richness, world_mean_richness)` and
`jp_world_mean_richness(&[Option<f64>])` are pure, and `JpStage` carries the
finished multiplier where the reference carries `mx`/`my`. Same precedent as
`civ_resource_trade_balance`'s caller-supplied means, and it preserves the
reference's own calibration anchor exactly: **1.0 means "no wildlife data",
and 1.0 is also what an exactly-average region produces**, so a port with no
ecoregion model behaves identically to the reference running on a world whose
wildlife layer was never built.

**Verified — golden, not hand arithmetic.** Reference lines 17297-19252 were
sliced out of `reference/Cartalith Gen1 v2.10.html` and evaluated in a bare
Node `vm.runInContext` with no DOM (the harness milestone 3 introduced,
extended: one contiguous slice with a block-comment balance check at the
boundaries, which is exactly the class of bug that bit milestone 3). Every
expected value in the 26 new tests is that run's output, including all eight
`jpCapacity` configurations field by field, all eleven `jpCalcLand` cases and
all seven `jpCalcWater` cases with their exact verdict and blocked-message
strings. `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ --lib`
(165 passed, 0 failed, 26 new), `cargo clippy -p cartalith-civ --all-targets`
(clean — the two remaining lib warnings are the same pre-existing ones
milestones 2 and 3 recorded), `cargo test --workspace` (0 regressions).

**Not wired to any caller** — no `#[func]`, no `compute_civilisation()`
integration, per the scope doc's own "Out of scope for all milestones": the
Journey Planner is interactive per-journey tooling whose real integration is
future GUI work.

**One new workspace dependency**: `cartalith-civ` now depends on
`cartalith-terrain`, for `river_coarse_ease` — `jp_stage_dry_km` divides the
map's own coarse ease back out to substitute JP's uncapped one, and
duplicating that function here would have been a second copy to drift from.

## DCC shell milestone 2: the Generate menu's real parameter dialogs (2026-08-18)

`UI_SHELL_DESIGN.md`'s Generate menu spec built for real — "the pipeline
stages in order [...] each opens its parameter dialog". The GUI half of the
owner's directive "make all generation options active in the current
interface so that we have the same functional controls as the older html
version"; the engine half is the previous entry ("Generation parameters: the
whole engine surface reachable from GDScript"), whose flat dotted-key API
this consumes.

**Built** — all in `godot-project/main.gd`; `main.tscn` is untouched, the
dialogs are constructed at runtime:

- **Six live stage dialogs** on the Generate menu — Tectonics, Volcanism,
  Erosion, Hydrology, Climate, Settlements — carrying **57 controls**, every
  one of them wired end to end from widget to `WorldParams` to the generated
  world. The remaining four stages (Glacial & coastal, Ecology,
  Infrastructure, Politics) stay visibly present and disabled, each with a
  tooltip naming the real reason: the engine has no parameters for them,
  because those passes are either unported or have no dials in either engine.
- **Nothing about a parameter is written twice.** Every range, step, label,
  unit and default is read at runtime from `WorldGen.get_param_info()` /
  `get_param_defaults()`, which the Rust side builds from `params.rs`'s
  `PARAMS` table. `main.gd` carries only what that table has no opinion
  about: which Generate-menu *stage* a parameter group belongs to, which rows
  are level-5 Advanced, and the prose. Adding a parameter stays one Rust row
  and no GDScript change, exactly as `params.rs`'s own doc comment intends.
- **The five-level disclosure grammar** (`design/Cartalith Menu Structure
  v2.dc.html`): menu bar (1) → Generate menu (2) → a stage's dialog (3) → a
  section per `params.rs` group (4) → that section's collapsed **ADVANCED**
  fold (5). Advanced membership follows a rule rather than taste: a
  parameter is Advanced if the reference itself buried it (its
  `<details class="adv">` *Physical coupling fields* block — flexure,
  heterogeneity, rock resistance) or if the reference never exposed it at all
  and this port surfaces it as a superset (`tect.lloyd`,
  `climate.current_k`/`ocean_hum`/`bulk_evap`,
  `climate.terrain_wind_deflection`).
- **Real reset**, two granularities: each dialog's *Reset this stage*, and
  Generate → *Reset all generation parameters* (which calls the engine's own
  `reset_params()` rather than replaying 58 values at it). Both restore
  `cartalith_engine::WorldParams::defaults` — the reference app's own `state`
  literal.
- **`set_params`' verdict is respected, not assumed.** It returns
  `{"clamped": [...], "rejected": [...]}`; when a key comes back in either
  list the row re-reads the engine's actual stored value instead of leaving
  the widget claiming a value the engine did not accept.
- **Six parameters are proxied, not duplicated.** `tect.dynamic_lithology`,
  `volc.provinces`, `climate.terrain_wind_deflection`, `climate.currents` and
  village seeding already had working controls in File > New World, pushed to
  the engine by `set_experimental_flags()`/`set_villages_enabled()`. Their
  stage rows drive those existing `CheckBox` nodes directly, so the two
  surfaces cannot disagree about one value. Verified in the app: toggling
  village seeding in Generate > Settlements flips the New World checkbox too.

**Two parameters deliberately excluded, each with its reason recorded in
`EXCLUDED_KEYS`**: `sea_level` (File > New World already owns it through
`set_sea_level()`; a second control for one value is worse than none), and
`use_gpu` (`GPU_LAYER_INTEGRATION_SCOPE.md`'s current milestone is still the
GPU-safe noise redesign, and per `DECISIONS.md` §7c the GPU path produces a
*different* world for the same seed — surfacing the switch now would expose
an incomplete path; `GUI_FEATURE_PARITY_SCOPE.md` Category-1 item #7,
deferred again here rather than silently dropped).

**Staleness — the honest answer, decided rather than faked.**
`UI_SHELL_DESIGN.md` says each stage "reports staleness". No staleness system
exists (`UNIFIED_TOOL_PLAN.md` milestone A, unbuilt), and more fundamentally
the engine is a **one-shot generator**: `generate_terrain` runs the whole
pipeline or none of it, so there is no per-stage incremental recompute for a
stage to be stale *relative to*. A per-stage "stale" pip would advertise
exactly the incremental pipeline that does not exist. So there are **no
per-stage staleness indicators**. Instead every dialog carries an honest
regenerate-to-apply affordance: a footer line that states plainly that the
whole world is regenerated and there is no per-stage recompute, a status-bar
note when a parameter has changed since the last generate, and a real
*Generate now* button whose own tooltip says it runs the same single full
pass File > New World's Generate runs.

**Ranges and labels — matched, and where they are not, said so.** Every
numeric range and step is the reference control's own, converted through the
reference's own `tparam`/`cparam`/`eparam`/`bind` mapping function (that
conversion lives in `params.rs`; `GENERATION_PARAMETERS.md` records the raw
slider range beside each row). Two honest deviations, both recorded:

- *Value-readout precision* is derived from each parameter's step (step >= 1
  → 0 dp, >= 0.1 → 1 dp, >= 0.01 → 2 dp, else 3 dp) rather than copying each
  reference span's own `toFixed`. It agrees with the reference everywhere
  except `Uplift spread`, which reads `18.0 px` here against the reference's
  `Math.round(...)+'px'` → `18px`. The step is 0.4, so a decimal is the more
  informative readout; noted rather than special-cased.
- *`flexure` and `hetero`* ship in the reference with a static HTML slider
  position that disagrees with the app's own `state` default. The reference
  overwrites both in `syncUI` (reference line 12656), so the `state` default
  is the real one — which is what `WorldParams::defaults` carries and what
  these dialogs show. A reference bug, not a port deviation.

**Verified — real windowed app, not just compiled.** `cargo build -p
cartalith-godot` clean; `cargo test --workspace` **563 tests across 83
binaries, 0 failures, 0 regressions**; `godot4 --headless --quit main.tscn`
clean load reporting `58 exposed by the engine, 2 deliberately excluded, 57
rows across the Generate menu`. Then the load-bearing check — the real
1920x1080 windowed app (`PrintWindow` + synthetic input, this session's
established technique), seed 12345 / 2048x2048 / 800 km / Classic throughout,
**one parameter changed at a time so attribution is unambiguous**:

| Changed | Struct | What the map actually did |
|---|---|---|
| `tect.plates` 14 → 40 | `TectonicParams` | Continent structure completely different — many more, smaller landmasses and inland seas at the same seed |
| `climate.equator_temp` 30 → 0 °C, `climate.pole_temp` −25 → −50 °C | `ClimateInputParams` | Coastlines identical, the whole world glaciated — biomes changed, terrain geometry preserved, exactly the expected decoupling |
| `volc.count` 20 → 100 | `VolcanismParams` | Extra volcanic cones on the same base terrain |
| `crater.count` 100 → 200 | `CraterParams` | Clear circular impact craters with rims where the previous render had plain ground |
| `river_density` x1.00 → x3.00 | `WorldParams` (top level) | Dense dendritic drainage networks across every landmass |

Also confirmed in the same session: *Reset this stage* restored Climate's
30 °C/−25 °C exactly; the staleness footer and status-bar note flipped on
change and cleared on generate; parameter tooltips surface the reference's
own element id (`Reference control #plates.`, `#tpo.`).

**Golden path re-verified, no regressions**: generation end-to-end from both
entry points (File > New World's Generate and a stage dialog's *Generate
now* — the same single function); all five map-overlay toggles including
Territory (faction fill) and Province boundaries; the causal-chain Inspector
on **hover** and **click-to-pin** (`Sevjuniana (Capital)`, population 19518,
`strong fresh water (1.00) → strong gentle terrain (0.97) → strong river
access (1.00)`, `Despite: weak flood risk (0.26)`, suitability 0.81, Strahler
4 · flow 2202), the pin surviving subsequent layer-toggle clicks; Help >
Credits; File > Open project's dialog.

**One layout trap worth recording**, since it cost a rebuild to find: an
autowrap `Label` with no width constraint reports a minimum height for
wrapping at its longest-word width — hundreds of lines for a paragraph. Three
of those in a dialog drove `AcceptDialog`'s `wrap_controls` to size the window
past the bottom of a 1080p screen, taking its own footer buttons with it.
Fixed by pinning the wrap width on every autowrap `Label`/`CheckBox` in these
dialogs (`_hint_label`, `_build_param_row`) and turning `wrap_controls` off so
the explicit `DIALOG_SIZE` holds and the `ScrollContainer` does the scrolling
it is there for. `ScrollContainer` itself was measured innocent — it reports a
12 px vertical minimum for 800 px of content.

**Still open**: the four parameterless stages stay inert until their passes
are ported (glacial, coastal) or gain dials (ecology, infrastructure,
politics); the three structured-orogeny knobs and the geoid/tides/seasons
sub-systems remain unexposed for the reasons `GENERATION_PARAMETERS.md`
records; `use_gpu` waits on the noise redesign. Light theme and responsive
breakpoints are still deferred (`DCC_SHELL_SCOPE.md`), as is any tool
functionality (`UNIFIED_TOOL_PLAN.md`). The pre-existing `dark_theme.tres`
issue where an unchecked `CheckBox` draws no glyph is unchanged by this pass
and still visible in these dialogs.

## Non-square maps: unlocking the aspect ratio the engine already had (2026-08-18)

Owner's standing complaint, and a real limitation: **the map is always
square, but nothing in the engine requires that.** This pass makes non-square
generation work end to end from the GDScript boundary, and verifies it.

**What the investigation actually found — the square-ness was in one file.**

`cartalith_engine::WorldParams` has always carried independent `pub gw` and
`pub gh`. More than that: **every golden-parity fixture in this workspace is
already non-square** — 14x11 and 16x12 across the whole `cartalith-civ`
battery and `golden_parity_carve.rs`, 24x18 and 20x14 in
`golden_parity_pipeline.rs`, 48x40 in `golden_parity_settlement_prereqs.rs`,
10x8 in `golden_parity_weather.rs`, 14x11 in the real `.zip` export fixture.
The engine, terrain, climate, hydrology, erosion and civ layers are therefore
*already JS-verified at non-square dimensions*, and have been since they were
ported. `cartalith-io`'s save loading was likewise already correct (its own
unit tests load 10x8 and 12x6, and `WorldGen::load_save` has always stored
both dimensions) — which is exactly why loading a real reference export never
hit the square restriction.

The restriction was two lines in `cartalith-godot/src/lib.rs`:
`call_params`'s `p.gh = gw;` and `absorb`'s `self.gh = gw as i32;` /
`compute_civilisation(&ws, gw, gw, …)`. `generate()` takes a single
`resolution`, so the boundary layer threw the capability away.

**The reference is never square, either.** `gridH(gw)` (reference HTML line
5049) is `round(gw * 0.5)` in world mode (2:1 equirectangular) and
`round(gw * 0.64)` in region mode (a 1.5625:1 frame), and the reference's
"Working resolution" segment (512 / 1K / 2K / 4K / 8K) sets the **width**
only. So this port's square default was never a parity match — it was an
artifact of a one-argument `generate()`. It stays the default here regardless,
because every golden fixture and every existing `main.gd` call is built on it.

**The API, and why it has this shape.**

Additive, square by default, fitted to the convention `88c15f0` established
(a flat dotted-key `Dictionary` for parameters, with `seed`/`resolution`/
`width_km` deliberately kept as `generate()` arguments):

- `generate_sized(seed, width_km, grid_w, grid_h)` and
  `generate_world_structure_sized(seed, width_km, grid_w, grid_h, archetype)`
  — the general entry points. `generate()`/`generate_world_structure()` are
  now exactly these with `grid_h = grid_w`, unchanged for callers.
- `reference_grid_height(grid_w, world) -> int` — the reference's own
  `gridH`, so a setup dialog can offer the shape the reference uses without
  hardcoding `0.64`/`0.5` in GDScript (`ARCHITECTURE.md`: "Godot computes
  nothing beyond layout").
- `get_map_width_km()` / `get_map_height_km()` — readouts.

Grid height is a **call argument, not a stored parameter**, for the same
reason `resolution` already is: it reallocates every field in the pipeline,
so it cannot honour the parameter table's contract ("set it, then generate as
many times as you like").

**`map_height_km` is derived, and deliberately has no setter.** Read
literally, every kilometre-to-cell conversion in this workspace goes through
one quotient — `map_width_km / gw` — and applies it isotropically:
`terrain_detail_k(gw, map_width_km)`, `river_flow_thresh(…, world_gw,
map_width_km)`, `civ_catchment_radius_cells(cat_km2, map_width_km, gw)`,
`suppression_radius_cells(spacing_km, gw, map_width_km)`. The engine's real,
already-shipped assumption is that **cells are square in kilometres**, so the
map's height in km is `map_width_km * gh / gw` and nothing else. An
independently-set height would silently contradict every distance, grade,
river threshold, catchment radius and settlement spacing the world was built
from — precisely the class of silent rescaling the reference cites when it
freezes `map_width_km` after creation. `get_map_height_km()` reports the
derived value; there is no `set_map_height_km`.

**Rendering: one real fix, and one thing that was already right.**

`render.rs` was audited per-pixel. Every index is `y * gw + x` with a genuine
`gh` bound; every resolution-derived radius (`smooth_sea_h`'s `gw/200`,
`build_ao`'s `ao_radius_frac * gw`, `build_hydro_wetness`, the stipple mark
spacing, `bio_jitter`'s and `land_color`'s noise frequencies) is keyed to `gw`
on *both* axes, which is isotropic and therefore correct — a feature is the
same size in cells whichever way the sheet runs. `box_v`, `sea_shade_from`,
`slope_at`, `aspect_factor`, `curvature_at` and `shade` all clamp against
`gh`. `vignette_at` and the edge haze normalize each axis by its own
dimension, matching the reference.

The one real problem was the plate frame. `border_width_cells` derives a
uniform margin in cells from `gw` (correct — a real plate margin *is*
uniform, and it is what keeps `get_border_inset_frac`'s "fraction of texture
width" contract exact), but `apply_border`/`border_cover` measure
`min(dx, dy)`: on a plate much wider than it is tall, that margin can exceed
half the height and cover the entire sheet, rendering blank paper. Now capped
at `0.25 * gh` — **only when `gh < gw`**, so every square and every tall grid
keeps byte-for-byte the width it had before. A guard that also fired on
square grids would have silently changed the frame at small square
resolutions, which is exactly the kind of drift this pass was told not to
introduce.

`pack.rs` needed no change: it already threads `gw`/`gh` into
`build_biome_and_wetland`, `place_map_icons_ruled` and the `Canvas`
rasterizer, all of which are dimension-independent.

**`map_overlay.gd`'s fit math was already correct — verified, not assumed,
and not touched.** `_displayed_rect()` computes `scale = min(size.x/gw,
size.y/gh)` and centres `Vector2(gw, gh) * scale`: a real aspect-preserving
fit, matching `MapView`'s own `stretch_mode = 5`
(`STRETCH_KEEP_ASPECT_CENTERED`). `_interior_rect` insets by
`border_frac * rect.size.x`, which is right for a non-square plate precisely
because the frame is a uniform cell count and the fit scale is uniform.
`_cell_to_screen` divides by `_gw`/`_gh` separately, and `main.gd` has always
passed `get_width()`/`get_height()` rather than one number. No GDScript
change was needed or made.

**Verified.**

- `cargo test --workspace`: 0 regressions, **every golden-parity fixture
  unmodified**, including `golden_parity_render.rs` (square, `js_reference()`)
  and the whole non-square civ battery.
- New `cartalith-engine/tests/non_square_pipeline.rs` (7 tests): the full
  pipeline at 256x128, 128x256, 250x150, the reference's own 256x164 region
  and 256x128 world shapes, a 512x32 case where resolution-derived blur radii
  exceed the shorter axis outright, and World Structure at 192x96. Every
  field allocated at `gw*gh`, all finite, height still normalized to `[0,1]`,
  no degenerate all-sea/all-land outcome.
- New `cartalith-godot/tests/nonsquare.rs` (7 tests): every cell of 192x96,
  96x192, 150x90 and 128x128 renders in range; world mode at 2:1; the plate
  frame is a uniform margin with a non-margin interior and never swallows the
  sheet; the border guard provably never fires on a square or tall grid;
  sprite compositing runs on non-square buffers in both orientations.
- The load-bearing one — **the image is the right shape, not merely the right
  size**: `rendered_water_still_lands_where_the_field_says_it_does` renders
  each shape under `js_reference()` and checks that blue-dominant pixels still
  coincide with `field[i] < sea_level` (freezing cells excluded, since
  `snow_glac` is blue-dominant too). A renderer that transposed axes, used the
  wrong stride or clamped `y` against `gw` would still emit `gw*gh` finite
  pixels but would decorrelate here. Agreement is required above 95%.
- Real PNGs of real non-square worlds, for eyeball verification, via
  `cargo test -p cartalith-godot --test nonsquare -- --ignored`:
  `target/nonsquare/{512x256,256x512,512x256_world,512x512}.png`.
- `cargo clippy -p cartalith-engine -p cartalith-godot --all-targets`: clean.
- `godot4 --headless --quit main.tscn`: clean load.

**Still open, deliberately.**

- **The setup dialog itself.** This pass is the Rust half only: the GUI is a
  follow-up (a sibling fork owns `main.gd`/`main.tscn` right now). What that
  dialog needs to call is `generate_sized`/`generate_world_structure_sized`,
  with `reference_grid_height()` for the default shape and
  `get_map_width_km()`/`get_map_height_km()` for the readout. The reference's
  own model is worth copying: a **width** resolution segment plus an extent
  (region/world) choice, with height derived — not two free spinboxes.
- **`cartalith-civ` was read but not edited** (a sibling fork is mid-milestone
  in it). Nothing was found that needs fixing: its whole public surface takes
  `gw, gh` pairs; its two width-only helpers (`civ_catchment_radius_cells`,
  `suppression_radius_cells`) are km-to-cell conversions that are *correct*
  under the square-cells rule; the seed-suppression radius `max(6, gw/22)`
  this port passes into `find_settlement_seeds` is `GW`-keyed in the
  reference too, on maps that are themselves non-square; and its golden
  fixtures are 14x11/16x12/48x40. Recorded here so the next reader does not
  re-derive it.
- **Extreme aspect ratios beyond roughly 16:1** are not a design target. They
  do not panic and are covered by a test (512x32), but a margin that is 4.5%
  of the height and a `min(gw,240)` weather grid only a couple of rows tall
  are degenerate rather than wrong.

## Unified tool plan milestone A — the `PassBuffer`/staleness core (2026-08-18)

`UNIFIED_TOOL_PLAN.md`'s foundation layer, the mechanism every tool in
milestones B-F shares. Shipped tested and **unwired** — no tool exists yet,
nothing in the pipeline consults it — the same "ship the primitive ahead of
the orchestration" precedent Phase 2 and the Journey Planner both used.

**Read first, designed after.** The reference's Sculpt editor (reference HTML
lines ~8837-9470) was read directly rather than through the plan's summary.
The plan's central claim held up: the reference already has a real
draft/commit/discard model, and it is the direct ancestor of the DCC shell's
"pass buffer" language, not invented UX. What reading added is the property
the plan never states — **a stamp holds no pixel data**. `{type, seed, pts,
g:{...}, f:{...}, hidden, _cx, _cy}` is a *recipe* (feature key, seed, the
captured stroke polyline in grid coordinates, two flat parameter bags, a
hide flag, a cached radial centroid), re-evaluated over its own padded
bounding box every time it is drawn or baked. That is why the draft can be
plain object state, JSON-snapshotted for undo, reordered, and thrown away
for free — and it is the reason this milestone could ship a small type
rather than a delta-buffer subsystem.

**Built.**

- `cartalith-spatial::pass` — `Stamp` (a trait, not a struct: `bounds()` +
  `apply(&self, dst, width, height)`, with an associated `Cell` type so an
  `f32` height stamp and a `u8` categorical-override disc both fit),
  `PassEntry<S>`, `PassBuffer<S>`, `CommitSummary`. Preview composites over
  a `&[Cell]` read of the field into a caller-owned scratch — the field is a
  shared reference, so "never mutates" is the borrow checker's guarantee, not
  a convention. Commit bakes the visible stack in order into the real field
  and marks every touched tile dirty **once**. Discard forgets. Plus the
  draft-scoped undo/redo stack ported from `sculptHistory`/`sculptRedoStack`
  (cap 30 = the reference's `SCULPT_HIST_MAX`) over all four structural
  edits the reference tracks: add, delete, reorder, hide.
- `cartalith-spatial::staleness` — `StageGraph`: a DAG of pipeline stages,
  each owning a `DirtyTracker`, with per-tile staleness computed lazily from
  version counters. No stage names, no Cartalith semantics.
- `cartalith-engine::staleness` — `PipelineStage` + `pipeline_stage_graph()`:
  the Cartalith stage names and edges. Placed here, not in the library crate,
  for the reason `cartalith-spatial`'s own `DirtyTracker` doc comment gives
  for refusing to bake in field names — the library stays generic, pipeline
  knowledge lives with the orchestrator that owns pipeline order.
- `cartalith-engine` now depends on `cartalith-spatial`: the workspace's
  **first** dependent on that crate. `LOD_TILING_BASE_SCOPE.md` built it
  standalone "for whenever a real large-world need actually triggers
  integration"; the trigger turned out to be the tool system, not LOD
  rendering, and that document is updated to say so.

**Design decisions worth the record.**

- **`DirtyTracker` needed no extension at all.** The plan predicted
  "necessary but not sufficient". Confirmed — but the remedy is pure
  composition, and not one method was added or changed. `mark_dirty` already
  *is* "my data changed at this tile, here is why, bump the version", the one
  primitive both editing and recomputation need.
- **Staleness needs two rules, not one.** Version comparison against direct
  upstreams — the plan's description — is not transitive: a height edit bumps
  only height's version, so climate, comparing against an unmoved hydrology,
  would report itself current. The graph adds "or an upstream is itself
  stale", evaluated recursively **at query time**. Deferral is intact
  (nothing is pushed downstream at commit; computing a flow change's
  downstream tile footprint is exactly the expensive query the plan refuses
  to run), and civ correctly reports stale after a terrain edit.
- **Deferral is structural.** `StageGraph` has no recompute hook of any kind
  — no closure, no callback, no trait object — and every query takes `&self`.
  It cannot recompute, rather than merely choosing not to. That is the code
  answer to the measured constraint (`CPU_MULTITHREADING_SCOPE.md`: terrain
  ~5.1s and terrain+civ ~7.07s at 2048², excluding climate/erosion/hydrology
  and civ's sequential stages) behind the mockup's "rivers · deferred".
- **The real chain has more edges than the plan's linear spine.** Verified
  against the real signature: `build_settlement_suitability` takes `field`
  (height) and `slope_n` directly, so civ depends on height and hydrology
  directly, not only through climate. Encoded. Erosion deliberately left out
  — it is two-way-coupled with climate (`ARCHITECTURE.md`'s known
  acyclicity pressure point), a cycle cannot be expressed here by
  construction, and picking an edge direction before a tool makes the
  question concrete would be guessing.
- **Two separate concerns, deliberately not merged.** A stage's dirty *flag*
  means "changed, presentation layer hasn't re-read it" — a re-upload marker,
  cleared by `acknowledge`. Staleness is computed purely from *versions*.
  Acknowledging never changes what is stale.

**Verified.**

- `cargo test -p cartalith-spatial`: 67 pass (24 before, 43 new).
  `cargo test -p cartalith-engine`: 21 pass including 5 new, golden-parity
  fixtures unmodified. `cargo clippy --all-targets` clean on both.
- The behaviours that actually matter, each a real test: a stroke previews
  without mutating the field; preview and commit produce identical results
  (the test that would catch the one-apply-two-destinations contract
  drifting); commit applies the whole stack in order and empties the draft;
  discard leaves the field **bit-identical**, compared as raw bit patterns so
  a `-0.0`/`0.0` or NaN-payload difference would still fail; one commit bumps
  each touched tile exactly once however many strokes touched it (the "undo
  granularity is one committed pass" rule, in code rather than left to
  callers); repeated commit/discard cycles bump versions only on commit;
  reordering the stack really changes the result (proved with a
  set-to-constant stamp — an add-only stamp would have hidden it); staleness
  marks exactly the right stages at exactly the right tiles, reports the
  most-upstream cause for a status line, and repeated querying changes no
  version and clears no staleness.
- `cargo test --workspace` could **not** be run to completion: `cartalith-civ`
  is mid-edit by a sibling fork and does not compile
  (`crates/cartalith-civ/src/lib.rs:8633`, a parenthesisation error in
  Journey Planner work, plus a `JpPlan` initializer missing five fields).
  Untouched here by instruction, and unrelated — nothing in this milestone
  references `cartalith-civ`. Every crate that does build was run
  individually instead: `-noise`, `-rng`, `-terrain`, `-climate`, `-erosion`,
  `-hydrology`, `-spatial`, `-engine`, `-io` all pass, 0 regressions.

**Still open, deliberately.**

- **The tools themselves** (milestones B-E) and shell wiring (F). This is the
  mechanism, nothing more.
- **The field-level undo snapshot at commit time.** The plan lists it under
  the shared editing model, but there is no undo stack anywhere in this port
  to snapshot into, and choosing its granularity before milestone B has a
  real committed edit to undo would be guessing. `PassBuffer::commit` returns
  the exact touched-tile list a tile-diff undo would need, so the seam is
  open rather than speculatively filled.
- **Tile-incremental recompute** of hydrology/climate/civ — none of those
  crates are tile-scoped today. `StageGraph` reports *which* tiles are stale;
  every stage still recomputes globally when asked. Unchanged from the plan's
  own deliberate deferral.

## Journey Planner milestone 5: route/stage derivation, in three sub-milestones (2026-08-18)

`JOURNEY_PLANNER_SCOPE.md` called milestone 5 "almost certainly the largest
single milestone in this whole plan", and it did not survive as one flat
pass. It is the real orchestration layer: what turns a drawn route polyline
into stages, and what runs milestones 3/4's stage calculators over them. The
work is recorded as the three sub-milestones the code actually falls into —
**5a world sampling**, **5b `_jpDeriveStages`**, **5c `_jpPlan`** — all three
shipped in this pass, so the split describes the work rather than a schedule,
but the boundaries are real (5c cannot be attempted before 5b, which cannot
be attempted before 5a).

**Ported (`cartalith-civ`, ~1,150 lines):**

- **5a** — `jp_road_cells` (+ `civ_walk_way_cells`), `jp_infra_context`,
  `jp_claimed_at`, `jp_stage_infra`, `jp_river_condition`,
  `jp_sea_condition`, `jp_coarse_idx`, `jp_stop_key`, `jp_mode_for_route`,
  `civ_transshipments`/`civ_transfer_overhead`, `civ_passed_settlements`,
  and the data behind them (`JP_INFRA_TIERS`, the v1.97 river-gradient and
  sea-condition bands, each rig's `neutral`/`span` derived from its own polar
  rather than written down).
- **5b** — `jp_derive_stages` and `JpDerivedStage`, plus the `JpWorld`
  borrowed context that replaces the reference's dozen globals.
- **5c** — `jp_plan`/`JpJourneyPlan`, `jp_effective_stage_plan` +
  `JpStageOverride`, `jp_ensure_plan`, `JpLegResult`/`JpLegCalc`,
  `JpTimelineDay`, `JpStop`, and five new `JpPlan` fields
  (`route_cond`/`infra`/`stage_overrides`/`season_drift`/`rest_cadence`)
  that reproduce the rest of `_jpEnsurePlan`'s default block.

**Four functions on no milestone list, needed here, ported rather than
stubbed.** The largest is a real gap this port had never noticed:
`_jpDeriveStages` samples `currentCartBiome()` **and** `currentCartTerrain()`
on every route point, and **neither the Cartalith biome paint layer nor the
terrain paint layer existed in this port at all** — the existing
`build_biome_raster` is the *climate* raster, a different vocabulary that
`cartalith-assets` already documents as distinct. `build_cart_biome`/
`build_cart_terrain`/`CART_BIOMES`/`CART_TERRAINS`/`jp_legacy_biome_of` are
ported here. One ordering detail was **checked rather than assumed**, and
would have silently mis-mapped every biome if it had not been:
`ELEV_TO_CART` is indexed by the reference's `BIOME_INDEX`, whose order puts
**shrub before savanna** — which is exactly this port's own `BIOME_*`
numbering, so the table transfers unchanged. The other three are
`_civTransshipments`/`_civTransferOverhead` (predicted by the scope doc —
`jp_journey_cost` takes the count and nothing produced it), `_civWalkWayCells`
and `_civPassedSettlements`.

**Three of this milestone's own listed functions are deliberately not Rust
functions**, each for a reason recorded in the scope doc rather than left as
a silent omission: `_jp_layovers` is a JS lazy-init idiom (a
`HashMap<String,i64>` needs none — shipped as the `JpLayovers` alias);
`_jp_settlements` is a *runtime* kind filter over the reference's one untyped
`state.places` array, and this port's settlements are already typed as
settlements, so building the `JpPlace` list **is** the filter; and
**`_jp_reroute_for_mode` is genuinely blocked** — its whole body is
`_civDijkstraPath(..., domain)`, and that function plus `_civWaterCostGrid`/
`_civMixedCostGrid` are unported, on no milestone in that document, and are
the interactive Route tool's own multi-modal pathfinder rather than anything
the Journey Planner owns. Its pure half, `jp_mode_for_route`, is ported.

**How the shapes resolved against milestone 4's `JpStage`** — the question
the scope doc wrote down in advance: `JpDerivedStage` **does** carry the
reference's `mx`/`my`, because they are a genuine map measurement made here;
`JpStage` correctly does not, because what the calculators consume is the
finished wildlife multiplier. `JpDerivedStage::to_stage(wildlife_forage_mod)`
bridges, and `jp_plan` takes a `&dyn Fn(f64,f64) -> f64` in exactly the
reference's `_jpWildlifeForageMod(mx,my)` position. **No change to `JpStage`
was needed** — milestone 4 got it right.

**Milestone 2's remaining functions.** `jp_auto_pick_vessel` shipped **here**,
because `_jpEnsurePlan` calls it on first plan creation and milestone 5 could
not be finished without it. The other two — `jp_auto_pick_transport` and
`_jp_best_package_for_stage` — are now **genuinely unblocked** (re-read
against what shipped, not assumed: the first needs `_jpEnsurePlan` +
`_jpDeriveStages` + `jpCapacity`-shaped arithmetic, all now real; the second
turns out to need only a stage and an `eff` *plan*, the same finding
milestone 4 made about `_jpBestLandTransportForStage`). Left to milestone 2's
own remainder rather than absorbed here, since nothing milestone 5 built
needs them.

**Two reference quirks reproduced as written**, recorded so nobody "fixes"
them: `_jpDeriveStages` falls back to `state.mapWidthKm||12000` while
`_jpInfraContext` two functions away uses `||800`; and `_jpRoadCells` keys its
map by JS string concatenation while `_civWalkWayCells` emits a way's first
and seam-break points **unrounded**, so those writes produce keys no integer
lookup can hit — reproduced by not recording a non-integral emission.

**Golden-verified against the real reference.** Eight line ranges were sliced
out of `reference/Cartalith Gen1 v2.10.html` — `riverCoarseEase`/
`terrainDetailK` (2641-2675), `classifyBiome` (5736-5743), `BIOME_KEYS`/
`BIOME_INDEX` (6796-6797), the cart paint layers (6810-6877), the whole
Journey Planner (17297-19419), `_jpModeForRoute` (20368-20379),
`_civPassedSettlements` (21154-21175) and `_civWalkWayCells` (21766-21777) —
and evaluated in a bare Node `vm.runInContext` with no DOM. **Milestone 4's
block-comment balance assertion was applied per slice and earned its keep
again**: it caught **three** genuine boundary errors (the `riverCoarseEase`,
cart-layer and `_civWalkWayCells` slices each ran one line into the following
comment block), and the JS parser caught a fourth (the Journey Planner slice
cut `_jpPlan`'s closing brace).

The world driven through it is synthetic but **exactly** reproducible: every
field is a closed form in `+ - * /` over exact values with no transcendental
anywhere, so the Rust test rebuilds the identical `f32` grids and only the
*outputs* are embedded. It is a real world for this layer: a 24x16 map with an
ocean margin, a lake, a mountain ridge, a river column, a highway, a
reference-road spur, claimed territory and five settlements, crossed by a
24-point route that derives into seven stages (2 sea, 1 river, 4 land), one
transshipment, a 41-day timeline and a genuinely unmet resupply requirement.

**Verified**: `cargo build -p cartalith-civ`; `cargo test -p cartalith-civ
--lib` (184 passed, 0 failed, **19 new**) — every expected value is the
reference run's output, including all seven stages field by field and the
whole `_jpPlan` roll-up (km, days, food/water/fodder, hazards,
ascent/descent, transshipment overhead, rest and total days, per-leg days and
speeds, the resupply-reach measurement, and the timeline's first/seventh/last
day with their camps); `cargo clippy -p cartalith-civ --all-targets` (the new
code adds no warnings; the remaining ones are pre-existing and unrelated —
two `needless_range_loop`s in `civ_sea_routes` and float-precision literals in
two older golden test files); `cargo test --workspace` (0 regressions).

**Not wired to any caller** — no `#[func]`, no `compute_civilisation()`
integration, per the scope doc's own "Out of scope for all milestones": the
Journey Planner is interactive per-journey tooling whose real integration is
future GUI work.

## Phase 5 milestone 1: urban morphology investigated; `cartalith-urban`'s RNG + geometry kernel (2026-08-18)

`ROADMAP.md`'s Phase 5 entry — "already a self-contained DOM-free engine […]
which suggests it ports cleanly into `cartalith-urban`, depending on
`cartalith-civ` for settlement context" — was written before anyone read the
code. Verified rather than inherited, the same way this project has already had
to correct the Journey Planner ("small" → ~70 functions), the Asset Library
("arbitrary named images" → a frozen ordered vocabulary) and territory
generation ("an algorithm" → no implementation at all). New scope doc:
`URBAN_MORPHOLOGY_SCOPE.md`.

**Three claims checked; two hold, one is wrong.**

- **"Self-contained DOM-free engine" — confirmed, unusually strongly.** Script
  block 4 (lines 28166-31104, 2,937 lines of body) is a single
  `const UME = (() => {…})()` IIFE. Grepping its range for `document`,
  `window`, `canvas`, `ctx.`, `getElementById`, `localStorage` and
  `requestAnimationFrame` returns **zero hits** — the sole match in the range is
  the word "context" inside a comment. It ends with
  `if(typeof module!=='undefined'&&module.exports)module.exports=UME;`, exposes
  fourteen internals via a `_test` export, and ships `hashModel(m)` (line
  31087), a stable FNV serialisation of graph/blocks/parcels/buildings which the
  reference's own comment labels "for determinism goldens". The reference's
  authors already ran this headlessly (`tests/run_um.sh`, named in the HTML
  comment above the block). Golden verification here did not have to be
  invented.
- **"Does not consume asset packs" — confirmed independently.** Phase 4's own
  milestone-1 investigation had recorded this; re-checked from scratch,
  `assetPack`/`AssetLibrary`/`AssetDB` are all zero hits in block 4. It emits
  geometry with **kind tags** (`b.kind`, `par.district`), never image
  references. Phase 4's finding stands unamended.
- **"Depending on `cartalith-civ` for settlement context" — wrong.** The engine
  takes **no civ types at all**. `generate(seed, opts)`'s entire input surface
  is a seed, numbers (`pop`, `epochs`, `settlementAge`), strings from fixed
  vocabularies (`culture`, `site`, `faith`, `civicStyle`, `wallStyle`),
  booleans (`walls`, `fortified`, `ruined`, `terrainAware`, `wallGenerations`),
  two plain rasters (`opts.water` = mask + distance transform + river
  centreline; `opts.terrain` = heightfield), and two point-list hooks
  (`routeEnds`, `primaryPaths`). The civ coupling is real but lives **one layer
  up**, in script block 2's `_um*` adapter (lines ~22036-22960, 28 functions,
  925 lines), which turns a settlement into that opts object. So
  **`cartalith-urban` depends on `cartalith-rng` and nothing else** — which is
  also what let this milestone be built and verified while `cartalith-civ` was
  mid-edit by a sibling fork.

**"Ports cleanly" is true of the boundary and false of the effort.** Measured:

| subsystem | functions | lines | milestones |
|---|---|---|---|
| Journey Planner | ~70 | ~3,100 | 6 |
| Asset Library | 19 top-level | ~2,250 | 7 |
| **urban morphology, engine** | **92** | **2,937** | **~13** |
| **plus the civ adapter** | **+28** | **+925** | **+2** |
| **Phase 5 total** | **120** | **~3,860** | **~17** |

and block 4 is denser per line than the Journey Planner — `buildWall` alone is
~190 lines of one algorithm, `grow` ~167, `buildBuildings` ~148, `applyStarFort`
~100. **Phase 5 is the largest single unported subsystem remaining.** What it
generates is correspondingly broad: A* primary routes over a slope-cost raster,
an epoch-loop organic growth model (or concentric radial rings for the Venus
culture), curtain walls and bastioned star forts, **planar face extraction** for
blocks, **vertex-bisector series platting** for parcels, building footprints by
grammar, districts, harbours, bridges, markets, faith sites, games, farmland and
a decay pass.

**RNG, proved rather than assumed** (the care Phase 2 milestone 9 took over
`_civRng`): block 4's own header says `mulberry32` is "intentionally NOT
redefined here … it falls through to the byte-identical module-scope copy
already in script block 1". Verified — no `mulberry32` anywhere in 28166-31104,
and block 1's line 2291 is the one `cartalith-rng` already golden-verifies. So
this is not merely the same algorithm under a different wrapper; it is the same
function. What is new is the **seed derivation**: `stream(seed,label)` =
`mulberry32((seed>>>0) ^ fnv1a(label))`, labelled substreams (`'site'`,
`'grow/e3'`, `'parcels/blk7'`) so each stage draws independently from one town
seed. `fnv1a` has no Gen1 equivalent and is ported here.

**Built — milestone 1, the foundation every later milestone reads:** new crate
`cartalith-urban` (no `gdext`, no civ, following `cartalith-spatial`/
`cartalith-assets`' standalone precedent), two modules.

- `rng.rs` — `fnv1a` (over UTF-16 code units, since JS `charCodeAt` is UTF-16),
  `stream`, and `Substream`'s `u`/`range`/`int`/`pick`/`norm`/`logn`/`chance`.
  Draw order is load-bearing and is pinned: `norm()` is Box-Muller and consumes
  **two** draws, and `pick` consumes a draw **even on an empty array** (the JS
  evaluates `f()` before indexing) — a port that short-circuited would
  desynchronise every later draw in the same substream.
- `geom.rs` — `js_hypot`, `Vec2` (with `Add`/`Sub`/`Mul<f64>`), `poly_area`,
  `poly_centroid`, `point_in_poly`, `seg_int`, `dist_pt_seg`,
  `poly_self_intersects`, `chaikin`, `simplify` (Douglas-Peucker), `ensure_ccw`,
  `inset_poly`, `clip_convex`, `convex_hull`.

**One real parity trap, found the hard way and worth the whole pass:**
`V.len`/`V.dist` are `Math.hypot`, and **V8's `Math.hypot` is not correctly
rounded**. ECMA-262 leaves it implementation-approximated; V8 scales by the
largest magnitude and Kahan-sums the squared ratios, so `Math.hypot(3,3)` =
4.2426406871192856585 while Rust's `f64::hypot(3,3)` = 4.2426406871192847703
(the correctly-rounded value) — one ulp apart on an input as ordinary as (3,3).
The **first** golden run of `dist_pt_seg` failed on exactly that case. Every
distance in this engine flows through `Math.hypot`, and many are threshold
comparisons — `attachPoint`'s 11 m snap, `rawEdge`'s 3.5 m minimum segment,
`nearestNode`'s search radius — where being *more* accurate than the reference
is the wrong answer (`cartalith-rust-conventions`: match the JS engine, do not
improve on it). `geom::js_hypot` reproduces V8's algorithm including the spec's
∞-before-NaN ordering, is golden-tested against twelve captured values, and
carries an explicit `assert_ne!` against `f64::hypot` so nobody "simplifies" it
back later. Left undetected it would have silently degraded every later
milestone's parity for reasons unrelated to correctness.

**Two reference behaviours pinned as behaviours, not fixed as bugs**, since
downstream code is tuned around both: `clipConvex` intersects against the clip
**segment** rather than the clip line, so clipping a shape that pokes past the
window's corners can collapse to empty (golden-asserted: a triangle clipped by
an overlapping square really does come back empty); and `insetPoly` returns
*nothing at all* — not a degenerate polygon — when the result's area is below 15
or it self-intersects at ≤60 vertices, which `buildBlocks` reads as "this block
cannot be built on".

**Verified — golden, not hand arithmetic.** The harness slices lines
**28167-31103 as one contiguous block**, plus line 2291 for `mulberry32`, and
evaluates them in a bare Node `vm.runInContext` with no DOM, with a
**block-comment balance assertion on both slice boundaries** — Journey Planner
milestone 4's design, adopted for the same reason (an unterminated block-comment
opener at a boundary silently swallows the rest of the slice; one contiguous
slice plus a balance assert removes the class), plus two assertions that the
slice really starts at the IIFE and ends at the export. 18 of the 19 tests are
that run's output, reached through the reference's own `_test`/public exports.
The nineteenth (`poly_self_intersects`) is a real unit test, documented as such
because that function is not on the `_test` export — the precedent territory,
provinces and `cartalith-spatial` set. The `norm`/`logn` values run through
`ln`/`cos`/`sqrt`/`exp`, the one family where V8's and Rust's libm need not
agree to the last bit; they were checked at **zero** tolerance and did agree, so
exact is what is asserted rather than an epsilon that would hide a future
divergence.

`cargo build -p cartalith-urban`, `cargo test -p cartalith-urban` (19 passed, 0
failed), `cargo clippy -p cartalith-urban --all-targets` (clean, no warnings).
`cargo build --workspace --exclude cartalith-godot` clean and
`cargo check -p cartalith-godot` clean — the full `--workspace` **build** stops
at `error: failed to remove file … cartalith_godot.dll — Access is denied`,
which is the Godot editor holding the DLL open, not a compile error and not
introduced here. Note the task's expected transient (`cartalith-civ` mid-edit,
not compiling) had already cleared: `cartalith-civ` builds.

**Not wired to anything** — no `#[func]`, no `compute_civilisation()`
integration, no GUI. Per `URBAN_MORPHOLOGY_SCOPE.md`'s own "Out of scope for
every milestone": urban morphology's real integration is a rendering decision
that does not exist yet, and the block-2/block-1 drawing code
(`_umDrawLayout`, `_umDrawLayoutPreview`, `_umLayoutAlpha`, the LOD hook near
line 15606) plus the browser-thread LRU/`setTimeout(…,0)` generation queue are
explicitly not port targets.

**Files touched**: `URBAN_MORPHOLOGY_SCOPE.md` (new),
`cartalith-native/crates/cartalith-urban/` (new crate: `Cargo.toml`,
`src/lib.rs`, `src/rng.rs`, `src/geom.rs`), `cartalith-native/docs/STATUS.md`,
`cartalith-native/docs/CHANGELOG.md`. The workspace `Cargo.toml` needs no edit
— `members = ["crates/*"]` picks the new crate up.

## Unified tool plan milestone B — the Sculpt-editor terrain port (2026-08-18)

`UNIFIED_TOOL_PLAN.md`'s largest single chunk, and the first real *tool*
engine in this port: the whole thirteen-feature Sculpt registry, its three
noise families, its stamp bbox/coverage/domain-warp pipeline, and its eight
presets — ported, golden-verified bit-exact against the reference, and wired
to milestone A's `PassBuffer` and to nothing else. Shipped **unwired**: no
Godot scene, `main.gd`, `main.tscn` or `cartalith-godot` file was touched
(sibling forks are live in `cartalith-godot` and `cartalith-civ`), the same
"primitive ahead of orchestration" precedent every prior milestone used.

**Where it landed, and why `cartalith-terrain`.**

- `crates/cartalith-terrain/src/sculpt.rs` (new) — the whole port.
- `crates/cartalith-terrain/tests/golden_parity_sculpt.rs` (new).

Milestone A split generic stack machinery into `cartalith-spatial` and
Cartalith *pipeline* knowledge into `cartalith-engine`. This is a third
category neither covers: **subsystem-domain math**. All thirteen features are
height-field formulas, `ARCHITECTURE.md`'s "one crate per subsystem" already
names `cartalith-terrain` as the crate that owns the height formula, and the
reference itself keeps `SCULPT_FEATURES` in script block 1 beside tectonics
rather than anywhere near its UI. A new `cartalith-sculpt` crate would have
bought a `Cargo.toml` and nothing else — no second consumer, no independent
test boundary (these tests need `cartalith-noise`, which terrain already
depends on), no dependency edge it would break. `cartalith-engine` would be
wrong for the mirror image of milestone A's reason: this is computation, and
*"`cartalith-engine` orchestrates; it does not compute"*. `cartalith-terrain`
gains a `cartalith-spatial` dependency — the workspace's second, after
milestone A's `cartalith-engine` edge.

**What the real registry turned out to be.** Thirteen entries, in
`Object.keys(SCULPT_FEATURES)` order: mountains, hills, ridge, plateau,
cliff, canyon, valley, river, lake, basin, coastline, volcano, freehand.
Eight `SCULPT_PRESETS` (Rolling Hills, Alps, Rockies, Badlands, Volcanic
Isle, Mesa, Karst, Glacial Valley), eight Freehand sub-modes (raise, lower,
smooth, cliff, ridge, canyon, mesa, volcano), eight shared globals
(`SCULPT_GLOBAL_DEF`: brushSize 32, hardness 0.5, intensity 1.0, noiseScale
5, octaves 5, persistence 0.5, lacunarity 2.0, edgeNoise 0.55), and 38
per-feature controls carrying their real min/max/step/default. All of it
ported as data — the control tuples included, because milestone F's tool
options bar and Properties panel need exactly those ranges and there is no
second source for them.

Three properties reading it added that the plan does not state:

1. **The registry's order is load-bearing.** A stamp's effective noise seed
   is `(stamp.seed ^ ((index + 1) * 1013)) >>> 0`, where `index` is the
   feature's position in the object literal. Reordering `FEATURE_KEYS`
   silently re-randomises every stamp in the file. The constant carries that
   warning, and a test pins the order.
2. **`edgeChar`/`edgeFreqMul` are per-feature registry data, not derived.**
   Thirteen hand-tuned pairs giving each landform its own domain-warped edge
   character — Coastline 1.5/0.55 (ragged, low-frequency), Mountains 1.4/1.5
   (tight, high-frequency), River 0.4/0.8 (nearly clean, because meander
   already supplies its shape), Cliff 0.6/0.45 (wanders like a fault trace).
3. **Volcano is the one feature that does not use `brushSize`.**
   `sculptStampRadius` special-cases it to its own `volcRadius` control,
   because its cone profile is defined in terms of that radius. Everything
   else — including Lake, the other radial feature — uses the brush.

**The brush model, concretely.** Coverage is
`smoothstep(0, 1, (R - dist) / feather)` with
`feather = max(floor, R * (1 - hardness))` — one falloff shape for all
thirteen, genuinely not user-selectable. `hardness` shapes the coverage,
`intensity` scales it into effect strength (`k = cov * intensity`); two
independent multipliers, which is why the mockup has two sliders. Then
**two** noise passes, not one: a domain warp displaces the *sample position*
before coverage is measured, so the silhouette moves; and a separate 3.4×
higher-frequency term roughens `cov` itself but only where `cov < 1`, so the
interior stays solid while only the rim breaks up. Both use `seed + 2100`;
the feature bodies' own `fbm`/`ridged`/`billow` use `seed`/`+700`/`+1400`.
Finally each feature returns a `mode` and a `val`: `add` → `h0 + k*val`,
`set` → `h0 + k*(val - h0)`. Which mode a feature uses is its defining trait
— Plateau being `set`-to-`max(h0, level)` is exactly why it never lowers
terrain and is a flatten/terrace tool rather than another raise brush.

**Determinism.** Every noise call goes through `cartalith-noise`'s
JS-matching `vnoise`/`hash`, never the GPU-safe PCG3D `gpu_vnoise`, and the
choice was checked against `DECISIONS.md` rather than assumed: §7's
golden-parity requirement covers any CPU path with a reference ancestor, and
§7a's principled-equivalence relaxation is scoped to GPU/optimized paths
specifically. A sculpt stamp has a reference ancestor and runs on the CPU, so
it must reproduce the reference bit-for-bit — and does.

**Golden verification — and a correction to the plan's own expectation.**
The plan predicted no golden path was available here: *"new-to-the-port
interactive behavior with no golden JS-array trace to diff against ...
verify per-feature `apply()` math ... rather than attempting stroke-sequence
parity."* That conflates two things. A *stroke sequence* is indeed not a
reproducible fixture — but a *stamp* is, and the reference stores one as
plain data (`{type, seed, pts, g, f}`). Constructing that object directly and
calling the real `sculptApplyStamp` under Node needs no pointer events, no
DOM, and no `generate()` run, because the reference itself marks the block
*"pure, DOM-free core"*. So milestone B got real golden parity: **23 cases,
every one bit-exact.**

Harness (transient, not checked in): Node `vm.runInContext` over four
contiguous line slices of the real file — 2292-2293 (`hash`/`vnoise`),
7568-7569 (`clamp01`/`smoothstep`), 8304 (`lerp`), 8821-9081 (the entire
Sculpt pure core) — each with a **block-comment balance assertion** and a
"starts at a top-level boundary" check, the technique Journey Planner
milestone 4 established. It earns its keep here: the 8821-9081 block both
opens and closes inside a long `/* ... */`, so an off-by-one at either end
would have spliced a comment open and silently swallowed code rather than
throwing a syntax error. One shim is disclosed in the test's own header:
`sculptDefaultParams` (line 9102) sits just past the pure core in the UI
half, so its three-line body is re-declared rather than widening the slice
into DOM-dependent code — it reads the registry's own control tuples, so the
defaults still come from the reference.

Cases: the twelve non-Freehand features, Freehand's eight sub-modes, the
"Alps" preset run through `Preset::apply` the way the UI runs it, and Lake's
commit-time `waterOnly` dry run. Each checked with an FNV-1a-64 fold over
every cell's raw `f32` bit pattern (so a one-ULP difference anywhere in 4096
cells fails), plus changed-cell count, six sampled cells as raw bits, and the
bounding box. Fixture field is a deliberately non-flat `f32` sawtooth — a
flat base would hide every `h0`-dependent branch, and River, Lake, Plateau,
Coastline and Mesa are all `h0`-dependent. Plus one cross-cutting test a
harness sharing the same copy-paste error would not catch: no two features
produce the same field at the same seed.

**Findings worth the record.**

- **`Math.pow`/`Math.exp` needed no tolerance**, despite this CHANGELOG's own
  earlier `1e-4` allowance for them. Every value is rounded to `f32` at
  exactly the point the JS `Float32Array` assignment rounds it, which absorbs
  the last-ULP `f64` disagreement between V8's fdlibm and the platform libm.
  The one razor-thin thing is the *fixture's own* base field: built in `f32`
  instead of `f64`-then-store, it shifts by an ULP and all 23 cases fail
  (which is how the first run failed, and a useful measure of the margin).
- **`Math.hypot` is not `sqrt(x*x+y*y)`** — V8 divides by the larger
  magnitude and Kahan-compensates the sum of squares. Ported as V8 computes
  it, then *measured*: swapping in the naive form still passes all 23 cases,
  because the `f32` store absorbs the difference too. Kept for fidelity and
  documented as explicitly **not** test-enforced, with the real risk named
  rather than implied — `nearest_on_stroke` picks a segment with a
  `dist < best` comparison, so one ULP can change which segment wins and with
  it the *sign* of `sd`, which Cliff and Canyon read directly.
- **Smooth also ignores `waterOnly`.** The plan flags the pre-loop snapshot
  (Freehand/Smooth is the one feature that bypasses the generic per-pixel
  path entirely, because a 4-neighbour blur cannot read stable neighbour
  state off a live-mutating buffer). It does not mention that the branch
  `return`s *before* the water-only check, so a smooth stamp would write
  height even on a water-only pass. Unreachable in practice — only Lake
  stamps are ever passed `waterOnly` — ported as-is rather than "fixed", with
  the reasoning at the site.
- **`sculptStampBBox` and `sculptApplyStamp` disagree about `feather`**: the
  bbox uses `max(2, rad*(1-hardness))` for every feature, `apply` uses
  `max(1.5, R*(1-hardness))` for non-radial ones. The bbox's floor is the
  larger, so the box always covers what `apply` writes. Harmless, and
  "fixing" it would change which tiles a stamp reports as touched. Ported
  verbatim.
- **One deliberate divergence, forced by milestone A's trait signature.** The
  reference reads `state.seaLevel` *live* at apply time, so moving the
  sea-level slider re-renders existing Plateau/Coastline drafts.
  `Stamp::apply` takes only a destination, so `sea_level` lives on the stamp,
  with `SculptStamp::with_sea_level()` as the explicit re-stamp. Same result,
  an explicit step instead of an implicit global read; only two of the
  thirteen features read it at all.
- **A known limitation carried over faithfully, not introduced.**
  `docs/SCULPT_EDITOR_INTEGRATION_PLAN.md` §6 left an open item — does the
  stroke-distance code handle world-mode equirectangular wraparound (a stroke
  crossing the antimeridian)? Reading the shipped `sculptNearestOnStroke`
  answers it: **no**, there is no wrap handling; the reference shipped
  without resolving its own open item. This port matches. Inventing wrap
  behaviour the reference never had would break parity for the common case to
  fix one nobody has hit.

**Verified.** 43 unit tests in `cartalith-terrain::sculpt` (bounds vs. writes
for all thirteen features, Plateau's never-lowers monotonicity, Cliff's
one-sided sign flip, Freehand raise/lower symmetry, the one-point-stroke
radial degeneration Freehand's tap-once modes rely on, River/Lake as the only
water writers, the water-only pass not double-carving, and four end-to-end
`PassBuffer` integrations including preview-equals-commit bit-for-bit and
discard leaving the field bit-identical) plus the 23 golden tests.
`cargo build -p cartalith-terrain`, `cargo test -p cartalith-terrain`
(43 + 23 + the crate's 11 pre-existing golden suites) and
`cargo clippy -p cartalith-terrain --all-targets` all clean.

`cargo test --workspace --exclude cartalith-godot` also ran clean this
session — the `cartalith-civ` build break the previous session recorded is
gone (that fork has landed). `cartalith-godot` was excluded only because its
`.dll` was locked by a running Godot editor; `cargo check -p cartalith-godot`
is clean, and nothing in this diff touches it.

**Not built, deliberately.** `sculptCommit`'s water hooks
(`enforceRiverChannels`, `enforceChannelDescent` + `riverMask`/`riverFloor`
locking, the lake→`lakeMask` deposit) are milestone C — though `apply_into`'s
`water`/`water_only` parameters, the primitive those hooks consume, are
ported and golden-verified here, because they are one branch inside the
function this milestone owns and splitting them out would have meant porting
`sculptApplyStamp` twice. Also not built: the "respect water mask" gate the
mockup shows for Raise/lower (the reference's Freehand has no water gate at
all — a real new feature, not a port), stroke capture and simplification
(`rdpSimplify`/`catmullRomSample` are input routing, Godot-side), the
`SCULPT_COLORS` overlay palette, and all shell wiring (milestone F).

**Files touched**: `cartalith-native/crates/cartalith-terrain/src/sculpt.rs`
(new), `cartalith-native/crates/cartalith-terrain/tests/golden_parity_sculpt.rs`
(new), `cartalith-native/crates/cartalith-terrain/src/lib.rs` (one `pub mod`
line), `cartalith-native/crates/cartalith-terrain/Cargo.toml`
(`cartalith-spatial` dependency, description), `UNIFIED_TOOL_PLAN.md`,
`cartalith-native/docs/STATUS.md`, `cartalith-native/docs/CHANGELOG.md`.

## DCC shell milestone 3: the World Setup dialog (2026-08-18)

Owner's own request, verbatim: *"maybe we should start thinking about a
proper base setup menu where we can pick map size, resolution, dimensions -
basically expanded from the current html version."* The GUI half of the
non-square work `22ae75b` landed in Rust. **No Rust changed this pass** — the
API it needs already existed (`generate_sized`,
`generate_world_structure_sized`, `reference_grid_height`,
`get_map_width_km`/`get_map_height_km`).

`UI_SHELL_DESIGN.md` puts "New world" in the File menu and rules that menu
items open **dialogs, never persistent side panels**, so this grows File ▸
New world — milestone 1's carry-over home for the generation controls — into
a real world-setup gate. Full design record and verification table:
`DCC_SHELL_SCOPE.md`'s milestone 3 section.

**Built:**

- A new first section in the New-world dialog, `MAP SIZE, RESOLUTION &
  DIMENSIONS`, built at runtime. Four rows in one grammar — **label · guided
  preset · exact value**: Extent (Region / Whole world), Map width km (six
  scale presets from Local 200 km to Planet 40 075 km, plus the reference's
  own free km entry), Resolution/columns (the reference's own
  512/1K/2K/4K/8K segment plus free entry 4–8192), Aspect/rows (2:1, 16:9,
  the reference's own 1.5625:1 region frame, 4:3, 1:1, 3:4, 9:16, Custom,
  plus a free row count). Every preset writes its free entry; typing any
  other value flips the preset back to *Custom*.
- A **live derived readout** under them — Grid (cells + total), Extent (km ×
  km), Cell size (km per cell), Aspect (ratio + landscape/portrait/square) —
  so picking 1K in region mode shows the real 1024 × 512 grid and the real km
  extent of both axes before anything is generated.
- Generation dispatched through `generate_sized()` /
  `generate_world_structure_sized()`, with the status bar, the top-right
  readout and the viewport scale bar all reporting the real shape.

**Three engine rules the design is built around** (`GENERATION_PARAMETERS.md`
"Map dimensions and aspect ratio"), not re-derived here:

1. **Cells are square in km.** Every km↔cell conversion in the workspace
   comes from the one quotient `map_width_km / gw` applied to both axes, so
   map height in km is derived (`width_km × gh / gw`). There is deliberately
   no height-in-km control — it is a readout, and the section's own header
   text states the reason rather than leaving the absence looking accidental.
2. **World mode is physically 2:1** (X wraps 360° of longitude, Y spans 180°
   of latitude). Choosing Whole world pins the aspect to 2:1, takes the row
   count from `reference_grid_height(gw, true)`, and disables the aspect and
   row controls **with the physical reason in prose directly above them**.
3. **Grid height is a call argument, not a stored parameter** — it
   reallocates every field in the pipeline, so it cannot honour the parameter
   table's "set once, generate many" contract.

**Nothing the engine owns is copied into GDScript**, the same discipline
milestone 2 set: both reference `gridH` factors (0.5 / 0.64) are asked of
`WorldGen.reference_grid_height()`, extent is stored through `set_params({
"world": …})`, and the post-generation summary reads `get_map_width_km()` /
`get_map_height_km()` back instead of echoing the request — so a
setup-readout/engine disagreement would be *visible*. The two scene-authored
controls the section needs (`%ResolutionInput`, `%WidthInput`) are
**re-parented** into the new rows rather than duplicated: one node per value.

**`world` now has two surfaces and one node.** It is a real generation
parameter the Generate ▸ Climate dialog legitimately shows, *and* a
creation-time shape decision the setup dialog owns. Rather than drop it from
one side it became a `PROXY_KEYS` entry onto the Extent control — the
mechanism milestone 2 already used for the four experimental flags, extended
here to handle an `OptionButton` (assigning `selected` emits no signal, so
the handler that reaches the engine is called explicitly). Verified live:
flipping the Climate checkbox moves the Extent selector, writes the engine
parameter, disables the aspect control and re-derives the grid to 2048 × 1024.

**Honest guidance instead of discovery-by-waiting.** Two conditional warnings
under the readout: 4K/8K grids are memory- and time-heavy on this port's
CPU-only pipeline (milestone 1's static hint, now conditional and covering
the row count too), and aspect ratios past ~16:1 are degenerate —
non-crashing, but the coarse weather grid loses almost all resolution across
the short axis and the plate frame swallows a large fraction of the sheet
(the finding the Rust non-square pass recorded).

**One real bug found in the existing dialog**: `%WidthInput`'s `max_value`
was 40 000 km, so "Earth's equator" (40 075 km) silently clamped. Raised to
100 000 with a step of 5. Found by the screenshot verification, not by
reading.

**Verified:**

- `cargo build -p cartalith-godot`: clean.
- `cargo test --workspace`: **719 tests across 88 binaries, 0 failures, 0
  regressions**. (Higher than milestone 2's 563 because sibling forks added
  `cartalith-urban` and a terrain sculpt module; `cartalith-civ` compiled
  fine despite the flagged possibility of mid-edit sibling state.)
- `godot4 --headless --quit main.tscn`: loads clean, warnings byte-identical
  to the pre-change baseline — checked by stashing the change and re-running,
  not assumed. (The two RID/ObjectDB lines are pre-existing.)
- **Real 1920×1080 windowed app**, driven through this dialog, each shape's
  setup readout compared against `get_map_width_km()`/`get_map_height_km()`
  after generating:

| Shape | Asked | Engine reported | Rendered |
|---|---|---|---|
| 2:1 landscape, Earth-like | 1024 × 512, 2 000 km | 1024 × 512, 2000 × 1000 km | correct 2:1 plate, not stretched |
| 3:4 portrait, Classic | 768 × 1024, 1 500 km | 768 × 1024, 1500 × 2000 km | correct portrait plate, polar snow at the north edge |
| Whole world, Earth-like | 1024 × 512, 40 000 km | 1024 × 512, 40000 × 20000 km | 2:1, **visible polar caps top and bottom**, sea lanes wrapping |
| 16:9, Archipelago | 640 × 360, 1 200 km | 640 × 360, 1200 × 675 km | correct 16:9 plate |

  Every readout matched the engine exactly. `map_overlay.gd` needed no
  change — its `_displayed_rect()` already fits with
  `min(size.x/gw, size.y/gh)`, so markers, roads and sea lanes land on the
  right pixels at any aspect.
- **Archetype dispatch re-verified** — the `a265b2b` bug, where World Shape
  silently never reached generation. Earth-like and Archipelago both
  dispatched through `generate_world_structure_sized` and produced their
  characteristic worlds with real settlement counts (36 and 25); the call's
  `bool` return is still surfaced as a visible failure rather than swallowed.
- **Golden path re-verified, no regressions**: generation from both entry
  points, all five overlay toggles (territory fill and province boundaries
  included), the causal-chain Inspector on hover **and** click-to-pin —
  driven through `map_overlay`'s own real hit test at a settlement's own
  pixel rather than a hand-made signal emit — all six Generate stage dialogs
  building, and Credits.

**Deferred, unchanged**: light theme, responsive breakpoints, all tool
functionality. Saving a *parameter set* as a named preset document is the
natural follow-up this milestone deliberately does not attempt.

## Journey Planner milestone 6 + milestone 2's remainder — the subsystem closes (2026-08-18)

The last two open milestones of `JOURNEY_PLANNER_SCOPE.md`, in the order its
build-order table set: **6 (verdict/reporting)** first, because it needed
milestone 5's plan output to verify against, then **2's remainder**, which
milestone 5 unblocked.

**Ported (`cartalith-civ`):**

- **Milestone 6** — `jp_verdict`/`JpVerdict` (v1.49's interpretive layer: the
  five-band read of a finished plan, with every contributing signal returned
  by name), `jp_confidence`/`JpConfidence` (the asymmetric honesty band on the
  day count), `jp_pack_range`/`JpPackRange` (the wagon-equation ceiling),
  `jp_fmt_days`, and `jp_risk` — the campaign-duration advisory milestone 5
  deliberately left here because it is a verdict string, not part of the
  roll-up. `jp_fmt_kg` had already shipped with milestone 4 and was not
  re-ported.
- **Milestone 2's remainder** — `jp_auto_pick_transport`/`JpAutoTransport` (the
  whole route's transport/animal/vehicle mix, including v1.48's analytically
  detected `fodderInfeasible` divergence and the Walking→Baggage Train
  auto-promote, which adds the one missing `_jpEnsurePlan` default,
  `JpPlan::auto_promote`) and `jp_best_package_for_stage`/`JpPackageFix` (v1.66's
  per-stage species+vehicle suggestion, on the same "measure, never silently
  apply" contract as `jp_best_land_transport_for_stage`).

Both reference functions build HTML hint strings; those are **not** ported —
presentation is Godot's (`ARCHITECTURE.md`), and every value the hints print
is a field on the returns above. `jp_auto_pick_transport` mutates the plan
exactly as the reference mutates `jn.plan`.

**A real bug found in a shared helper, by this milestone's own golden run.**
`js_fixed` — milestone 4's reproduction of JS `toFixed`'s round-half-*away
from zero* tie-break — was computing the tie by scaling: `(v*10^d +
0.5).floor()`. That **fabricates** ties. `61.5/30` is `2.0499999999999998`,
which JS renders `"2.0"`, but `2.0499999999999998 * 10` rounds to exactly
`20.5` in `f64` and the `+0.5` then carried it to `"2.1"`. `jp_fmt_days(61.5)`
was the case that caught it. Rewritten to decide the tie on the value's
**exact** decimal expansion — a double is a dyadic rational, so a genuine tie
at place `d+1` means the expansion ends in a 5 there — and to lean on Rust's
own `{:.N}` (already the correctly-rounded exact decimal) for everything else.
Verified against `Number.prototype.toFixed` on 30 cases including the pairs
that look identical and are not (`1.25` is a real tie, `2.05` is not) and
`jp_fmt_kg(1250.0)` = `"1.3 t"`, the tie that reaches a user-visible string.
No existing test changed its expected value.

**`_jp_reroute_for_mode` remains the one unported function, and the finding
was re-checked rather than inherited**: its whole body is
`_civDijkstraPath(s,e,domain)`, and that function with `_civWaterCostGrid` and
`_civMixedCostGrid` is the interactive Route tool's own multi-modal
pathfinder — unported, on no milestone in the scope doc, and a UI action
besides ("Re-route land-only"). Its pure half, `jp_mode_for_route`, shipped
with milestone 5. Nothing here invents a pathfinder to close it.

**Golden-verified against the real reference**, through milestone 5's own
harness and fixture: eight line ranges sliced out of `reference/Cartalith Gen1
v2.10.html` into a bare Node `vm.runInContext` with no DOM, each carrying the
**block-comment balance assertion** on its own boundaries. All eight balanced
first time — including the one that moved, the Journey Planner slice extended
from 17297-19419 to **17297-19532** to take v1.49's verdict layer with it. The
harness did surface one error of a *different* class, which the balance check
is not designed to catch and the JS parser could not either: the milestone-5
slice `2641-2675` starts one line **below** `TERRAIN_DETAIL_MAX_K`, which
`riverCoarseEase` reads, and `_jpDeriveStages` catches its own exceptions and
returns an empty stage list — so the whole world silently derived to zero
stages with no error anywhere. Found by instrumenting that `catch`; the slice
is now `2640-2675`.

The world, route and party are milestone 5's fixture unchanged, and reproduce
its values exactly (760.847480700888… km, 41.317750030325… days, seven
stages). The m5 route cannot reach every verdict band on its own — its
resupply requirement is genuinely unmet, which alone forces `severe` — so each
band probe edits exactly the signals `_jpVerdict` reads on a **real** plan, and
the harness made the identical edits to the identical fields.

**Verified**: `cargo build -p cartalith-civ`; `cargo test -p cartalith-civ`
(**194 passed, 0 failed, 10 new**) — every expected string and number is the
reference run's output, covering all five verdict levels and both Strained
texts, all fourteen contributing reasons, every `_jpConfidence` threshold from
both sides, `_jpPackRange` across species/grazing/desert, `jpFmtDays`' three
unit bands and its rounding edges, all four risk tiers, nine
`jpAutoPickTransport` configurations (counts, carts, wagons, promotion,
divergence) and thirteen `_jpBestPackageForStage` cases;
`cargo clippy -p cartalith-civ --all-targets` (the new code adds no warnings;
the remaining ones are the same pre-existing, unrelated set); `cargo test
--workspace` (0 regressions — `cartalith-urban`'s two failures are a sibling
fork's uncommitted work in the same tree, and that crate depends only on
`cartalith-rng`).

**Not wired to any caller** — no `#[func]`, no `compute_civilisation()`
integration. See `JOURNEY_PLANNER_SCOPE.md`'s closing status for what
integration would actually mean.

**Journey Planner engine work is complete.** All ~70 real `jp*`/`_jp*`
functions are ported bar `_jp_reroute_for_mode`; the seven UI-only ones the
scope doc names were never portable; what remains is the interactive GUI that
would give a player somewhere to type a journey into.

## Phase 5 milestone 2 — the planar street graph, the thing the whole engine stands on (2026-08-18)

`URBAN_MORPHOLOGY_SCOPE.md` milestone 2: all 15 functions of reference lines
**28363-28512** — `makeGraph`, `gKey`, `gridCellsForSeg`, `indexEdge`/
`unindexEdge`/`edgesNear` (the uniform-grid spatial index), `addNode`,
`nearestNode`, `rawEdge`, `splitEdge`, `attachPoint`, `addStreet`,
`addPolylineStreet`, `extractFaces`, `edgeBetween` — as
`cartalith-urban::graph`. Dependencies unchanged: `cartalith-rng` only. Wired to
nothing, per the standing discipline.

The plan said 28363-28513; `edgeBetween` ends at 28512 and `astar` begins at
28514, so the range was one line long. Small, but the kind of thing this port
checks rather than inherits.

**The planarity invariant lives here.** `addStreet` attaches both endpoints
(snap to a node within 11 m, else split the nearest edge within 9 m, else a new
node), splits every live edge the new segment crosses, promotes every existing
node within 2.5 m of the segment's interior to a junction, then chains the whole
ordered sequence. `extractFaces` — angularly-sorted half-edge traversal with
dead-end spur collapsing — is what makes town blocks possible at all.

### The index design, settled for the whole crate

Dense `Vec` with tombstones, ids never reused, exactly as the scope doc
predicted: `splitEdge` leaves the split edge in place and dead, and later
milestones walk `g.edges` by index filtering on `alive`. A slotmap would have
been the "better" Rust answer and would have changed the iteration order that
`extract_faces` and every `filter(e => e.alive)` pass depends on.

Two things the plan did not say, both verified rather than assumed:

- **`nextN`/`nextE` are not stored.** They are unconditionally `nodes.len()` and
  `edges.len()`, and the capture asserts that against the reference's own
  counters on all 19 scenarios rather than leaving it as a claim.
- **`gKey` does not survive as a function.** An `(i64, i64)` tuple key is the
  same partition as the reference's `cx + ':' + cy` string, and the grid map is
  only ever probed, never iterated, so no ordering is lost. 15 reference
  functions land as 14 Rust items.

`cls` stays a `&'static str` rather than becoming an enum: the reference
compares it by string in six places and `hashModel` serialises it verbatim, so
the string *is* the value — and an enum would have to guess now at classes later
milestones introduce (`'ringroad'` in 10, `'lane'` in 11, a variable in `grow`).

### Golden-verified through `_test`, and mutation-checked afterwards

`UME._test` reaches `makeGraph`, `addStreet` and `extractFaces`, and that turns
out to be enough for all fifteen, because the harness dumps the **entire graph
state** after each scripted scenario rather than return values alone: every node
with its adjacency, every edge including tombstoned ones, the uniform grid cell
by cell, and the extracted faces. `attachPoint`/`rawEdge`/`splitEdge`/
`nearestNode` live entirely inside `addStreet`; the index family's whole
observable effect *is* the grid. 19 scenarios, matching exactly — floats
emitted as JSON shortest-round-trip decimals so Rust parses each back to the
bit-identical `f64` and nothing is compared within a tolerance.

The scenarios include a **stress case driven by the reference's own exported
`stream`**, so 24 pseudo-random streets produce the identical input sequence on
both sides — making it a golden over `cartalith-urban::rng` and the graph at
once (94 edges, 12 faces).

**`hashModel()` turned out not to be usable here, correcting an assumption the
scope doc made.** It reads `m.graph`/`m.blocks`/`m.parcels`/`m.buildings` off a
finished `generate()` model and cannot be fed a partial subsystem; it becomes
reachable at **milestone 16**. The state dump is stricter anyway — `hashModel`
rounds coordinates to `Math.round(n.x*100)`.

**The goldens were then mutation-checked**, because a full-state dump can look
thorough and still be vacuous. Perturbing the 26 m index cell, the 0.7 cell
step, the 3×3 cell dilation, the 11 m node snap, the 9 m edge snap, both 3.5 m
guards, the 2.5 m node-promotion radius, the `[0.03, 0.97]` t clamp, the spur
collapse's stack rule, the outer-face tie-break's strict `>`, and swapping
`js_hypot` for `f64::hypot` each break at least one golden. **The first round
found two constants unexercised**, and two scenarios exist only because of it:
`clampT` (a 400 m street where the t clamp genuinely moves the split, from
x=10 to x=12 at the low end and 388.36 at the high end) and `hypotSnap*`.

### `js_hypot` earns its keep, visibly

Milestone 1 established that V8's `Math.hypot` is one ulp above the correctly
rounded value. Milestone 2 makes that **structural**: at
`dx = 7.778174593052022`, V8 gives `Math.hypot(dx, dx) == 11` exactly while
Rust's `f64::hypot` gives `10.999999999999998`. `attachPoint` snaps at strictly
under 11. So on that input the reference builds a **four-node** graph and a
port using `f64::hypot` builds a **three-node** one — a different graph, not a
differently-rounded one. Four `hypotSnap*` goldens straddle the boundary and a
named test asserts the arithmetic directly, so `js_hypot` cannot be
"simplified" away without failing first.

### The block-comment assertion, run as a negative control

It caught nothing this time — milestone 1's slice boundaries are unchanged and
correct. But deliberately shifting them found a genuine hole in the assertion
itself: a slice that *ends* inside a block comment is caught (unterminated
open), and one that starts three lines into the header comment is caught once an
**orphan-close counter** is added — but one that starts exactly *one* line late
is not, because the scanner treats an apostrophe at depth 0 as a string
delimiter and block 4's header comment contains the prose `"Gen1's globals"`,
which swallows the stray `*/`. The orphan-close counter is a real improvement
and is kept; the residual hole is covered by the two **structural** assertions
(the slice must contain the `UME` IIFE header and must end at
`module.exports = UME;`). Recorded plainly in the scope doc: the balance assert
is necessary, not sufficient.

### Findings that change how later milestones must be built

1. **Encapsulation, verified by grep across all 2,937 lines of block 4:**
   `cell`, `grid`, `nextE` and `nextN` are touched **only** by this milestone's
   functions. No later milestone reaches into the spatial index.
2. **`g._fromPaths` is a dynamic JS property and needs a real field.**
   `buildPrimariesFromPaths` sets it (line 28830, milestone 6) and
   `builtMassHull` reads it (line 29709, milestone 10) to discount the bare
   degree-2 vertices a resampled real road drags in. Milestone 2 deliberately
   did **not** add the field — nothing sets or reads it yet — and both later
   milestones' entries now say so, because skipping it over-encloses the
   enceinte along arterials exactly as the reference's own v1.01 note describes.
3. **The reference is internally inconsistent about one splice, and the port
   reproduces it.** `splitEdge` removes an edge from `a.adj` with an
   **unguarded** `splice(indexOf(e.id), 1)` — a miss would silently drop the
   *last* element, since JS `splice(-1,1)` does — while milestone 11's
   `_killEdge` guards the identical splice with `if (k >= 0)`. Unreachable given
   `rawEdge`'s invariant; reproduced rather than hardened, and flagged so
   milestone 11 does not unify them.
4. **`addStreet` leaves orphan nodes.** When both endpoints are fresh and every
   resulting link is then rejected by the 3.5 m minimum, the nodes stay in
   `g.nodes` with empty `adj`. Pinned by a golden (4 nodes, 1 edge). Later
   passes must keep filtering on live adjacency.
5. **The stable hit sort is a safety property, not a behavioural one — the tie
   is unreachable.** Two crossings at one `t` are the same point, so those edges
   already crossed and share a node whose half-edges the `1e-4` guard excludes.
   Two on-segment nodes at one `t` lie on one perpendicular within 2.5 m of the
   segment, hence within 5 m of each other, which the 11 m snap prevents. A
   crossing tied with a node sits at that node's own foot, ≤2.5 m away, which
   the 3.5 m split guard folds back. Established by trying to construct one and
   failing: the scenario built for the attempt is kept, renamed `nearParallel`,
   for what it does cover. Confirmed by mutation (an unstable sort changes no
   golden) and by a test that re-derives every hit parameter across all 19
   scenarios.
6. **Two constants in `addStreet` are redundant inside the engine's own site
   box.** The `1e-4` interior-crossing and `1e-3` node-parameter guards survive
   being loosened to `1e-9`: a hit at `t = 1e-4` is `1e-4·L` from an endpoint,
   so it only escapes the 3.5 m fold-back past `L > 35 km`, and the node guard
   past `L > 3.5 km`, against `SITE_WM`/`SITE_HM` of 1700 × 1250 m. Kept as
   written — they are the reference's, and they are what stops the degenerate
   case if the box ever grows. The two surviving mutations are reported as a
   finding, not hidden as a gap.
7. **`extractFaces`' guard arithmetic is subtler than it looks.** JS
   `while (guard++ < 20000)` leaves `guard` at 20001 when the bound stopped it,
   so the post-check `guard >= 20000` also discards a face that closed on step
   20000 exactly, and a traversal that hits the guard is **dropped**, not
   truncated. Reproduced as written.
8. **The outer-face tie-break is observable.** A closed loop with one dead-end
   spur yields exactly two faces of equal absolute area (±14400 on the golden),
   and the strict `>` makes the *lowest-indexed* one outer. `buildBlocks`
   (milestone 12) skips the outer face, so this is not cosmetic.

### Verification

`cargo build -p cartalith-urban`, `cargo test -p cartalith-urban` (26 tests, up
from 19), `cargo clippy -p cartalith-urban --all-targets` all clean.
`cargo build --workspace` hit the known Windows quirk — `failed to remove file
… cartalith_godot.dll — Access is denied`, a Godot editor instance holding the
DLL, not a compile error — so it was run as `--workspace --exclude
cartalith-godot` plus `cargo check -p cartalith-godot`, both clean. Stated
rather than reported as a clean workspace build that did not happen.

**Scope discipline held**: milestone 2 only. `astar` (milestone 3) is next door
in the reference and was not touched; `Graph::from_paths` was left for milestone
6 rather than added speculatively; nothing is wired into
`compute_civilisation()`, `cartalith-godot` or the GUI.

## Unified tool plan milestone C — the Water & ecology group (2026-08-18)

`UNIFIED_TOOL_PLAN.md`'s milestone C: River/water's special commit path and
the Cartography paint brush, both golden-verified against the real reference.
Shipped **unwired**, the same "primitive ahead of orchestration" precedent
milestones A and B used — no Godot scene, `main.gd` or `cartalith-godot` file
was touched. `UNIFIED_TOOL_PLAN.md` gained a "Milestone C as built" section
with the full findings; this entry is the summary.

**Built:**

- `cartalith-spatial/src/paint.rs` — `PaintStamp` (the hard-edged categorical
  disc, implementing milestone A's `Stamp` trait with `Cell = u8`) and
  `PaintLayer` (the lazily-allocated override grid, its nearest-neighbour
  sample, its per-cell merge, and `state.cartoPaint`'s sparse
  `[index, value, …]` persistence). Generic machinery, so it lives beside
  `PassBuffer` rather than in a domain crate — the module never learns what a
  biome is, only that `0` means unpainted. Milestone A's own `pass.rs` doc had
  already named this type in advance.
- `cartalith-hydrology/src/lib.rs` — `enforce_river_channels`, the
  deposition-refill clamp, three lines from `enforce_channel_descent` in this
  port as in the reference.
- `cartalith-engine/src/sculpt_commit.rs` — `WaterState`,
  `commit_sculpt_pass`, `SculptCommitSummary`: `sculptCommit`'s water hooks,
  composing `PassBuffer` (spatial), `SculptStamp` (terrain) and
  `enforce_channel_descent` (hydrology) without computing anything new.
- `cartalith-civ/src/lib.rs` — `apply_force_lake`, closing a gap this
  milestone itself opened (below).

**What the "special commit path" really is.** Read directly (reference lines
9318-9346) it is a fixed five-step sequence in which every step's ordering is
load-bearing: bake the whole stack → `enforceRiverChannels` (re-clamp cells
locked by an *earlier* commit, **after** the bake and **before** this batch's
carving, because a Mountains stamp painted across an old river would otherwise
bury it) → per river stamp, `enforceChannelDescent` + lock → Lake last, as a
`water_only` dry run against the already-final height → one `computeFlow`, one
`refreshClimate`. The first four steps are ported; the fifth deliberately is
not, because it is downstream whole-field recompute and milestone A's
`StageGraph` exists so it stays deferred. That line — steps 2-4 are *part of
the edit*, step 5 is *recompute* — is the plan's one real ambiguity, now
resolved.

**Three things reading the reference corrected in the plan (River/water):**

1. **`half_w` is the brush, not the discharge.** `carveRiverValleys` derives
   its channel half-width from Strahler order and a real-km scale;
   `sculptCommit` uses `max(1, brushSize*0.13)`. The right difference — a
   hand-painted river has no drainage area — but porting the generated
   formula would have silently changed every hand-painted river.
2. **`enforceChannelDescent` walks the stroke's own points and never
   resamples.** A two-point stroke locks **3 cells**; the same stroke as 23
   points locks **46**. That is a real constraint on milestone F: stroke
   capture must not decimate hard, or a river carves visibly and locks almost
   nothing, and later erosion refills it. Both fixtures ship because testing
   only the coarse stroke would barely have exercised the lock.
3. **A draft with no water stamps is bit-identical to a plain commit**, tested
   on raw `f32` bit patterns rather than assumed, so callers never choose
   between two commit functions.

**Three things reading the reference corrected in the plan (Biome paint).**
The plan's core claim holds — a separate override array, `0` = unpainted, a
hard land-only gate rather than a toggle — but:

1. **There are three paint layers, not one**: `paintBiome`, `paintSplat`
   (asset-pack ground textures) and `paintTerrain` (`CART_TERRAINS`), three
   peer arrays driven by one brush. One `PaintStamp` serves all three; a
   biome-shaped type would not have.
2. **The merge is two operations at two altitudes, and the plan describes the
   rarer one.** Per-cell replace happens in exactly one place, the Cartalith
   editor export. The *renderer* does not replace anything — `landColorCore`
   alpha-blends the painted index's palette colour over the fully shaded
   procedural colour at weight **0.60**, explicitly so hillshade/AO/haze still
   show through. And the audit the plan asked for has a clean answer: **no
   analysis consumer merges at all** — `buildEcoregions` and every Journey
   Planner `currentCartBiome()` reader take the unpainted classifier output.
   Painted overrides are presentation and export, never simulation input, so
   merging them into `classify_biome`'s callers as the plan's phrasing invites
   would have introduced behaviour the reference does not have.
3. **The gate is `wb[i] !== 0`, not `== 1`** — it excludes lakes as well as
   ocean, including above-sea-level ones. A port gating on ocean alone passes
   every ocean test and silently paints over lakes, so the golden fixture
   classifies its water band as `2`.

The rasters these overrides sit on are the ones Journey Planner milestone 5
ported (`build_cart_biome`/`build_cart_terrain`, 1-based `Vec<u8>`), which is
exactly `PaintLayer`'s shape — the two fit with no adapter.

**A gap this milestone opened and then closed.**
`cartalith_civ::build_water_bodies` had deliberately omitted the reference's
`forceLake` option, reasoning — correctly at the time — that *"no painting UI
exists in this port, so it would be an always-false input with no caller ever
setting it."* This milestone is the producer that reasoning was waiting for:
the Lake stamp's commit writes `lake_mask`, and `forceLake` is its only
consumer, the thing that makes a painted lake classify as a lake even when its
basin sits above sea level or is too arid to pool. Without it `lake_mask`
would have been dead output. It ships as `apply_force_lake`, a post-pass
rather than a new parameter — **bit-equivalent**, because `force` is the last
mutation the reference makes to `out` — which leaves `build_water_bodies`'
signature and every caller untouched, including the one in `cartalith-godot`
this milestone must not edit. `build_water_bodies`' own doc comment, which
still carried the expired reasoning, was corrected.

**One deliberate new affordance, flagged as new rather than as parity.**
`PaintStamp::mask` is an `Option`; `None` means no gate. The reference always
gates. This exists because `UI_SHELL_DESIGN.md`'s tool options bar shows a
"respect water mask" switch the reference has no equivalent for — the same
mockup-vs-reference gap milestone B recorded for Freehand raise/lower. The
Cartography constructor requires a mask and the ungated one is separately
named, so parity is the default and the addition is opt-in
(`DECISIONS.md` §7d).

**One open question left open.** The reference clears painted overrides on
terrain rebuild, but it only ever had one `generate()`. This port now has
incremental terrain edits, and whether a Sculpt commit that changes the
climate inputs under a painted cell should clear that cell has no reference
answer. `PaintLayer::clear` implements the reference-faithful floor and its
doc names the question; the deciding caller is the shell, which does not exist
yet.

**Golden-verified — 18 tests, every one bit-exact on the first run** (11 for
the water commit path, 7 for the paint brush), against the real reference in a
bare Node `vm.runInContext` harness. Six contiguous line slices — 2292-2293,
7568-7569, 8304, **8725-8745**, 8821-9081, **4758-4795** — each carrying a
**block-comment balance assertion** plus start- and end-of-slice top-level
boundary checks. Both new slices sit hard against comment boundaries.

The assertions earned their keep again, in the two different ways they can.
The end-of-slice check threw on `hash/vnoise` (a one-line function whose brace
is not at column 0) — a false positive, but exactly the class it exists to
surface, and fixing it properly rather than deleting it kept it useful for the
two tight new slices. The second failure is the one worth recording, because
the balance check is **not** designed to catch it and it produced *silently
empty* results rather than an error: the reference declares
`paintBiome`/`_paintLayer`/`_paintValue`/`_paintRadius` with `let`, which in a
`vm` script are lexical bindings, **not** properties of the context object, so
setting `ctx._paintRadius` from the host created a shadow the reference code
never read and `_paintAt` ran against defaults. Same class as Journey Planner
milestone 5's silently-empty stage list. Everything now drives `_paintAt` from
inside the context.

`sculptCommit`'s water-hook body is **transcribed, not sliced**, and that is
disclosed rather than implied: the function opens with `_sculptEditorActive()`
and closes with `computeFlow`/`refreshClimate`/`renderNow`/`sculptSyncUI`, all
DOM or whole-pipeline recompute, so lines 9320-9346 are copied verbatim with
`sculptStamps` as a parameter and those calls dropped.

Milestone B's two fixture findings held: the base field is built in `f64` and
rounded once at the `f32` store, and **no tolerance was needed anywhere** —
heights compare as raw `f32` bit patterns folded FNV-1a-64 over all 4096
cells, and paint layers are integers. One cross-check worth naming: the
`hidden_river_is_skipped` case reproduces milestone B's own `mountains` golden
hash exactly, independent evidence that the water hooks are genuinely inert
rather than merely usually harmless.

**Verified**: `cargo test` on every crate touched — 21 new unit tests in
`cartalith-spatial` (88 total, up from 67), 10 new in `cartalith-engine`, 2
new in `cartalith-hydrology`, 3 new in `cartalith-civ` (197 total) — plus the
18 golden tests; `cargo clippy --all-targets` clean on all four;
`cargo test --workspace` 0 regressions. `cargo build --workspace` hit the
known `cartalith_godot.dll — Access is denied` quirk (a Godot editor holds the
DLL open), so it was run as `--exclude cartalith-godot` plus
`cargo check -p cartalith-godot`, both clean; no `cartalith-godot` file was
edited.

**Not built, deliberately.** The tools' *interaction* halves — stroke and tap
capture, the layer/value/radius pickers — are input routing and belong to
milestone F. Also not built: the `biomes`/`terrains` pack-image decode, which
`cartalith-godot/src/pack.rs` skipped because *"there is no producer of a
painted-cell array anywhere in this workspace"* and which named "a future
milestone that ports the Cartography paint-brush tool" as the place to resume
— that producer now exists, but the consumer is a `cartalith-godot` render
change this milestone is scoped out of. The 0.60 painted-colour blend in
`land_color` is the same case.

## GUI parity Category-1 sweep: the world data browser and the performance readout (2026-08-18)

`GUI_FEATURE_PARITY_SCOPE.md`'s Category 1 — *"real backing exists, just
needs wiring"* — closed except for one row, which this pass demotes to
Category 2 rather than force-fitting.

### What the audit said, versus what was already true

That document was written before three forks landed on the same day, so
half its Category-1 table had already been closed by work that was not
about it. Re-verified against the code, not the doc:

| # | Item | State found |
|---|---|---|
| 1 | Import asset pack | **Already done** — DCC shell milestone 1, `File ▸ Import asset pack…` with a real `FileDialog` |
| 2 | Settlements table | Genuinely unwired — **done here** |
| 3 | Trade balance / Economy | Genuinely unwired — **done here** |
| 4 | Province list | Genuinely unwired — **done here** |
| 5 | Faction culture-terrain-fit | Genuinely unwired, and **not Category 1**: see below |
| 6 | Planet gravity / day length / axial tilt | **Already done** — the generation-parameter API (`88c15f0`) put `planet.g`/`planet.rotation_hours`/`planet.axial_tilt_deg` in the table, and the Generate stage dialogs (`a11c2d7`) render them as the Climate stage's PLANET section |
| 7 | GPU status / toggle | Readout half genuinely unwired — **done here**; toggle half stays deferred, see below |
| 8 | World Structure raw sliders | **Already done** — same two commits: `world_structure.*`'s five knobs are the Tectonics stage's WORLD STRUCTURE section, and `apply_archetype()` exists so a preset writes those same five sliders the way the reference's own archetype segment does |
| 9 | Layer granularity | **Already done** — DCC shell milestone 1, three independent toggles |
| 10 | Click-to-pin selection | **Already done** — DCC shell milestone 1, the Properties dock |

Item 5 is the honest correction. `civ_culture_terrain_fit` is real and
unit-tested, but its signature takes a per-faction `terrain_mix` and a
`world_mean_terrain` — and *nothing computes either*. `ECONOMY_SCOPE.md`
still lists that aggregation (`_civFactionAggregates`) as unstarted. So
this is not "a one-line `#[func]` mirroring `set_sea_level`"; it is a real
`cartalith-civ` milestone with a thin GUI on top. The audit itself hedged
it as "a half-step into Category 2"; this pass moves it there outright
rather than adding a `#[func]` with no argument to call it with.

### Built — no Rust changed

Every `#[func]` this needs has existed and been called by nothing:
`get_settlements()` (read, but only for map markers), `get_provinces()`,
`get_trade_balances()`, `get_gpu_stages_used()`. So this milestone is
`main.gd` only — `main.tscn` untouched, no crate touched, following the
pattern the three preceding forks converged on (dialogs built at runtime
from what the engine reports, so adding data stays a Rust-side change).

**World data browser** — `Simulate ▸ Economy…` and `Simulate ▸ Statistics…`,
one dialog with three tabs:

- **Settlements** — every settlement the world placed, sortable on any
  column and filterable by name. Closes the prior shell's own hint text
  ("a dedicated searchable/sortable table here is not yet built").
  Clicking a row pins that settlement in the Properties dock with the full
  causal "why here?" chain — the same chain clicking its map marker gives,
  so the table is a second door into the existing inspector rather than a
  parallel display of the same facts.
- **Provinces** — `get_provinces()`'s first consumer ever. Province
  *boundaries* have rendered since milestone 16; province *identity* (name,
  owning faction, and which settlement each was grown from) has never been
  shown. The capital column resolves `capital_settlement_index` against
  `get_settlements()`, so the two queries are visibly the same world.
- **Economy** — `get_trade_balances()`'s first consumer ever: per
  settlement, which of the 15 resource keys its hinterland exports and
  imports against the world mean.

**Performance readout** — `View ▸ Performance readout…`. Names all six
GPU-eligible stages and says GPU or CPU for each, from
`get_gpu_stages_used()`. That query is deliberately not derivable from the
`use_gpu` parameter: every stage falls back to CPU *individually* on any
GPU init or dispatch failure (`HARDWARE_ACCELERATION.md` §27), so "asked
for GPU, got none" is a state the UI has to be able to say out loud, and
the summary line says exactly that when it happens. Plus a runtime block
(FPS, adapter, threads, static/video memory) from Godot's own
`Performance`/`OS`/`RenderingServer` singletons — flagged in the dialog
itself as an addition this port has and the reference never did
(`DECISIONS.md` §7d), not as parity.

The `use_gpu` **toggle** stays deferred and visibly so: the checkbox is
present, disabled, and carries the reason.
`GPU_LAYER_INTEGRATION_SCOPE.md`'s current milestone is still the GPU-safe
noise redesign, so switching the path on would produce a different world
for the same seed (`DECISIONS.md` §7c). Same "visibly present, not hidden"
rule the inert menu items already follow.

### Placement, against a shell the audit predates

`GUI_FEATURE_PARITY_SCOPE.md` was written against the panel-browser shell
and recommends dock panels in places the DCC shell has none. Its
Category-1/2 findings are about the Rust engine and survive that; its
placement recommendations do not. `UI_SHELL_DESIGN.md` governs instead:
menu items open dialogs, never persistent side panels, and the right dock
is Layers/Properties/Sample only. Hence dialogs behind menu items whose
names come from that document's own Simulate list, not three new dock
panels. One dialog with a tab row rather than three near-identical
dialogs, so the sort/filter/fill machinery is written once; the tab row
reuses the flat-toggle-button pattern the workspace tabs already
established rather than a `TabContainer`, which has no `dark_theme.tres`
entry and would arrive in Godot's default chrome.

Nothing in the browser derives a fact: every value is a field the engine
returned, and GDScript formats, sorts and filters it (`ARCHITECTURE.md`).
The one exception is the summary line's row counts, which count what is on
screen.

### Verified

- `godot4 --headless --quit main.tscn`: clean load, byte-identical console
  output to the same run with `HEAD`'s `main.gd` swapped in (including the
  two pre-existing RID/ObjectDB exit warnings, checked rather than assumed
  to be mine).
- `cargo test --workspace` at `HEAD` in a clean worktree: **all green, 0
  failures**. The working tree could not be tested directly — a concurrent
  fork is mid-commit in `cartalith-civ` and `cartalith-godot/src/render.rs`
  (`civ_is_valid_terrain` not yet added, `land_color` grown a `lith`
  parameter its call site hasn't caught up to), so
  `cargo build -p cartalith-godot` fails on *their* uncommitted edit.
  Nothing in this milestone touches Rust, so the worktree baseline is the
  honest number.
- **Real windowed app, real world, real data.** Generated 512×328 at seed
  12345 (40 settlements, 9 provinces) and captured the running window with
  `PrintWindow`: the settlements tab shows 40 real rows with
  faction-coloured ids matching the map's own Okabe-Ito markers; sorting by
  population descending and filtering to "a" gives "39 of 40 rows" with the
  six capitals on top; provinces shows 9 rows each resolving to a real
  capital settlement name; economy shows real per-settlement export/import
  lists; the performance readout shows all six stages CPU with "0 of 6
  eligible stages ran on GPU — the whole pipeline ran on CPU, as
  configured", and a live 60 FPS / RX 7800 XT / 16 threads / 73 MiB runtime
  block.
- **Province rendering survived both shell rebuilds** — confirmed by
  screenshot, not by reading the scene file: the Layers dock's "Province
  boundaries" toggle draws `build_province_boundary_texture()`'s real
  boundary lines over the territory fill.
- All three new menu entry points driven by **real mouse clicks** on the
  running window, not by calling the handlers: `Simulate` opens with
  Economy…/Statistics… live among the greyed inert rows, clicking Economy
  opens the dialog on the right tab, and `View ▸ Performance readout…`
  opens its dialog. Explicit menu ids moved to 100+ so they cannot collide
  with the sequential ids `PopupMenu.add_item()` hands the inert rows
  around them — currently harmless since disabled items never emit, and not
  harmless the first time one is enabled.
- Two real defects found this way and fixed before commit: the performance
  dialog's footer hint sat *under* `AcceptDialog`'s own OK button at 620 px
  tall (the same fixed-height-versus-content tension the autowrap-`Label`
  trap comes from), and `Tree` centres column titles by default, which
  reads as a heading for the wrong column once a wide name column sits
  beside a narrow numeric one.
- Golden path re-checked in the same run: generation, all five layer
  toggles, hover sample, click-to-pin Properties, scale bar and status
  line all unchanged.

### Still open in Category 1

Nothing, except item 5 as re-classified above and item 7's toggle as
deferred above. `GUI_FEATURE_PARITY_SCOPE.md` is updated in the same commit
with its whole table re-baselined against the DCC shell, so the next pass
is not working from a map of a shell that no longer exists.

## Phase 5 milestone 3 — A\* over the site cost raster, and the golden suite that wasn't testing anything (2026-08-18)

`URBAN_MORPHOLOGY_SCOPE.md` milestone 3: `astar`, reference lines
**28514-28547**, as `cartalith-urban::astar`. Dependencies unchanged
(`cartalith-rng` only). Wired to nothing, per the standing discipline. 7 new
tests, 33 in the crate.

The plan said lines 28514-28556. `astar`'s last line is
`path.reverse();return path;}` at **28547**; 28548-28556 is a blank line plus
the *site model* header comments that belong to milestone 5's range. The scope
doc is corrected. (Milestone 5's own 28557-28742 is right — `shoreFromMask`
really does start at 28557.)

### What was ported

One function, ~34 dense lines: a hand-rolled binary heap, an 8-connected
neighbourhood with `Math.SQRT2` diagonals, trapezoidal edge costs
(`dl * 0.5 * (cost[i] + cost[ni])`), and a `0.9`-weighted Euclidean heuristic
measured in cells. Ported literally rather than swapped for `BinaryHeap`,
because the heap's tie-break is what makes the path reproducible: sift-up stops
on `<=` (an equal-`f` newcomer stays below its parent) and sift-down uses a
strict `<` (a tie prefers the left child). `BinaryHeap` has neither property.

Every distance goes through `geom::js_hypot`, not `f64::hypot`.

### The verification is the story

Seventeen scenarios were written by hand first and **all seventeen reproduced
the reference exactly on the first run**: degenerate 9x1 and 3x17 strips, both
rectangle orientations, a 500-cost wall with one gap, an infinite moat, a NaN
band and a NaN seal, a zero-cost field, start-equals-goal, adjacent and
diagonally-adjacent goals, two rasters filled by the reference's own exported
`stream`, and a sweep taking **every** cell of a 6x5 raster as the goal in turn.

Then fifteen mutations of the ported algorithm were run against them and **nine
survived**: the `0.9` weight, the `0.5` trapezoid factor, the `DIRS` order, all
three heap comparators, `js_hypot` vs `f64::hypot`, the `i == gi` early break,
and the dead `INFINITY` guard. Nine of the twelve behaviours that make this
function reproducible were untested, by a suite that looked thorough and passed.

**The cause generalises well past this milestone:** a *continuously-valued* cost
raster essentially never produces two frontier entries with exactly equal `f`,
so it cannot observe a tie-break at all. Only a *quantised* raster can. An
exhaustive search over ~800,000 (raster family x size x endpoint) combinations
found a discriminator for every survivor, and every tie-break discriminator came
from a quantised field — costs drawn from `{0.5, 1}`, `{1, 2}` or
`{1, 2, 3, 4}`. Eight such scenarios were added (`tiesHalf`, `tiesLeft`,
`tiesRight`, `tiesWide`, `tiesDiag`, `nearAdmissible`, `trapezoidal`,
`greedyTrap`), captured from the reference like every other. **Fourteen of
fifteen mutations now die.**

That regime is not artificial. `buildPrimaries` builds its raster as
`(1 + (slope*3.2)^2) * 8`, and slope over most of a site is flat — so the real
8 m cost field is *mostly constant* away from the river and the bridge band.
Ties are the normal case there, not the exotic one.

**The one surviving mutation is reported, not hidden.** Deleting
`if (g0[i] === Infinity) continue;` changes nothing, because the branch is
unreachable in the reference too: `g0[ni]` is written on the line before every
`push`, and `g0[si]` before the start's own push. The line is kept because it is
what the reference writes; a test asserts the invariant it rests on (no
relaxation ever writes a non-finite `g`) across the infinity and NaN scenarios,
rather than pretending to cover a dead branch.

### `js_hypot`, quantified

Milestone 1 found the V8 discrepancy; milestone 2 proved it changes graph
*topology*. Milestone 3 adds the frequency: over the 4,096 integer offsets a
64x64 raster produces, `js_hypot` and `f64::hypot` disagree on **1,398** —
better than a third, all by one ulp. It still took a 64x48 quantised raster
(`tiesWide`) to build a golden that notices, because one ulp only bites when it
makes or breaks an exact tie. Asserted directly as well as golden-enforced.

### What the reference's A\* actually is

Written down so nobody "fixes" it: the heuristic is `0.9 x` Euclidean distance
**in cells** while a step costs the trapezoidal mean of two *metres-scaled*
raster values (order 8-2000), so it is wildly under-weighted normally and
over-weighted wherever the raster is cheap; there is **no closed set** and no
stale-entry check, so cells are re-expanded; and `if (i === gi) break` stops on
the first *pop* of the goal, which under an inadmissible heuristic need not be
its cheapest path. The search is **reproducible, not optimal**, and the golden
path is the specification — a correctness-improving rewrite would move every
primary route, and with it every block, parcel and building grown against it.

### `null` comes only from non-finite cost

An 8-connected full grid has no unreachable cell, so `astar` returns `null` only
by arithmetic: an `Infinity` tentative cost fails `c < g0[ni]`, and a `NaN` one
fails it too — every NaN comparison is false in Rust exactly as in JS. Both
pinned by goldens (`moat`, `nanSeals`), and the NaN case is one of the few
places in this port where JS/Rust NaN agreement is load-bearing rather than
incidental (`cartalith-rust-conventions`).

### One deliberate divergence

An out-of-range `start`/`goal` **panics** here. The reference reads past its
typed arrays, gets `undefined`, and — since `undefined === Infinity` is false —
sails past its own guard into nonsense. Its only caller (`buildPrimaries`'
`toCell`) clamps to `[1, W-2] x [1, H-2]` first, so the branch is unreachable in
the engine; loud beats silent for something that cannot happen.

### Harness changes worth inheriting

Same contiguous 28167-31103 slice plus line 2291, same balance scan with
milestone 2's orphan-close counter, re-run as a negative control (the
one-line-late hole is confirmed still present and still covered). Two
improvements:

- The first structural assertion is tightened from "the slice *contains* the
  `UME` IIFE header" to "**the slice's first line is** block 4's header comment
  opening" — which catches the one-line-late case directly rather than by luck.
- A fourth assertion runs as a live negative control in the other direction:
  block 4 must **not** define `mulberry32`, since the whole reason line 2291 is
  spliced in is that it falls through to block 1.

The capture also refuses to write a file unless every path is non-empty, starts
at its start cell, ends at its goal cell, the two sealed scenarios really
returned `null`, and the capture exceeds 300 path cells — the explicit emptiness
gate three earlier subsystems in this project needed and did not have.

### One tooling trap

The first mutation run reported two **false** survivors. Cause: `cargo`'s
freshness check is mtime-based and a mutation written in the same second as the
previous build was silently not rebuilt; and one pattern (`dl * 0.5 *`) matched
inside the function's own **doc comment** before it matched the code, so
`String.replace`'s first-occurrence rule mutated prose and nothing else. Both
found by hand-checking a "survivor" and watching it die immediately. Any later
milestone that mutation-tests should stamp the file's mtime forward and anchor
its patterns on code that cannot appear in a comment.

### Corrections written forward

- **Milestone 6 must not "improve" the search.** `buildPrimaries` runs `astar`
  once per external route endpoint over a *copy* of the raster with already-used
  cells multiplied by `0.45`, so reinforcement is order-dependent on
  `site.routeEnds` and each run inherits the previous run's exact cell set. Any
  change to which cells a path occupies compounds across routes.
- **Milestone 6 owns the clamp.** This port's `astar` takes `(usize, usize)`
  cells and panics out of range; `toCell`'s `max(1, min(W-2, round(p.x/CS)))`
  must be reproduced at the call site.
- **Milestones 12 and 13 will hit the same coverage trap.** `buildBlocks` and
  `buildParcels` compare areas and lengths against thresholds; goldens built
  only on continuous random inputs will not exercise their tie-breaks either.
  Build at least one quantised or symmetric fixture per milestone from here on.

### Verified

- `cargo build -p cartalith-urban`, `cargo test -p cartalith-urban` (33 pass),
  `cargo clippy -p cartalith-urban --all-targets` — all clean.
- 25 golden scenarios plus a 30-goal sweep, every path compared exactly against
  the reference's own `UME._test.astar` output. No tolerances anywhere.
- Mutation-checked: 14 of 15 mutations killed, the survivor documented above.

## Unified tool plan milestone D — the Civilization group (2026-08-18)

`UNIFIED_TOOL_PLAN.md`'s milestone D: Place settlement's manual-insertion path,
Draw route/way's pathfinder and snap interaction, and Territory/faction's
override mechanism — all three golden-verified against the real reference.
Shipped **unwired**, the same "primitive ahead of orchestration" precedent
milestones A-C used: no Godot scene, `main.gd`, `main.tscn`, `render.rs` or any
`cartalith-godot` file was touched. `UNIFIED_TOOL_PLAN.md` gained a "Milestone D
as built" section with the full findings; this entry is the summary.

**Built:**

- `cartalith-civ/src/tools.rs` — the whole milestone. `merge_territory_paint`;
  `civ_place_pick_weight`/`civ_pick_place_at`/`civ_drop_place` (+ `DropPlace`)
  and the two zoom-scaled pick radii; `civ_nearest_on_way`/
  `civ_find_snap_target`/`civ_snap_point` (+ `SnapTarget`/`SnapKind`);
  `civ_dijkstra_path` with its three cost grids, the existing-way discount and
  the `reachable` flag (+ `RouteContext`/`RouteMode`/`WayRef`/`DijkstraPath`);
  `civ_join_dijkstra_segs`; `civ_commit_way` (+ `ManualWay`/`ManualWayType`).
- `cartalith-civ/tests/golden_parity_civ_tools.rs` — 16 golden tests.
- `cartalith-civ/src/lib.rs` — `TerrainValid` widened from a `bool` to the
  reference's four real modes, `js_hypot`, and a real bug fix in
  `civ_smooth_path` (below).

**The headline finding: `_civDijkstraPath` is not `road_dijkstra`.** The plan
said the pathing primitive was already ported and only the interaction, the
`ManualWay` struct and the unreachable-leg warning were new. Checked against the
reference: that is wrong, and the gap is most of the tool. `road_dijkstra` is the
reference's `roadDijkstra` — the bare single-source relaxation kernel over a
caller-supplied cost array, in script block 1, ~22 500 lines earlier.
`_civDijkstraPath` is one of its *callers* and calls it at exactly one line;
everything that makes a route a route is in the wrapper and was unported: three
cost grids (`_civLandCostGrid`/`_civWaterCostGrid`/`_civMixedCostGrid`, with
`_CIV_SEA_COST = 0.6` deliberately *below* the flat-land baseline), the
existing-way ×0.25 discount and its polyline rasterizer, settlement gravity,
reconstruction into world coordinates, wrap-aware smoothing with a mode-matched
terrain-validity repair, and the `reachable` flag.

**This unblocks the Journey Planner.** `JOURNEY_PLANNER_SCOPE.md`'s closeout
listed `_jpRerouteForMode` as the subsystem's one still-blocked function,
because its whole body is `_civDijkstraPath`. That function now exists, with all
three domains and the `reachable` flag `_jpRerouteForMode` exists to check.
`JOURNEY_PLANNER_SCOPE.md` is updated; what remains there is a three-line
transport→domain mapping and a UI action.

**Territory/faction is an addition, not parity — flagged as such.** The
reference has only a paint brush (`_civPaintTerritoryAt`); `PHASE2_SCOPE.md`'s
milestone-9 investigation already found it never had algorithmic territory
generation at all. This port does (`assign_territory`, `DECISIONS.md` §7b), so
the tool paints over a base the reference never had — a superset under
`DECISIONS.md` §7d. The brush itself needed **no new code**: milestone C's
`PaintStamp::ungated` *is* `_civPaintTerritoryAt` (the reference's own
`_paintAt` comment calls itself "a direct lift of `_civPaintTerritoryAt`'s
geometry"), exactly as milestone C predicted. `ungated` and not `new` because
`_civPaintTerritoryAt` has no land/water gate — a faction can own coastal water.
The whole new surface is a five-line `merge_territory_paint`.

**Three more things reading the reference corrected:**

1. **`_civCommitRoute` sits eighteen lines above `_civCommitWay`** and looks
   nearly identical, but routes `'mixed'` into `civJourneys` instead of
   `'land'`/`'water'` into `civWays`. The plan warns about conflating
   `_civOpenRouteEditor`; this is the closer trap, and porting it would let a
   hand-drawn road cut across a bay.
2. **The unreachable fallback is not a straight line from start to end.**
   `_civSmoothPath` splits runs at any `|Δx| > GW/2` jump — unconditionally,
   world mode or not — and the reconstruction's start→target-cell hop is such a
   jump, so the run holding the start is dropped and the drawn stub sits at the
   *target* end with the start absent. Pinned by a test so nobody "fixes" the
   port into disagreeing with the reference; the shell's warning must not
   promise a line between the user's two waypoints.
3. **`_civDropPlace` checks select-near-existing *before* the water refusal**,
   so a settlement whose terrain changed under it stays selectable.

**Two bugs found in already-shipped, already-golden-verified code**, both latent
because no prior fixture in this crate had a wrapped route, both fixed with every
pre-existing golden still passing:

1. **`civ_smooth_path` summed `km` across run boundaries.** The reference's
   `if(k > 0)` guard is per-run, deliberately excluding the seam jump a `brks`
   entry marks; the port used "if anything has been pushed". Milestone D's
   world-wrap fixture reported 876.8 km for a route the reference measures at
   136.6 km — one whole map width per seam crossing. Affected every consumer of
   a wrapped way's length (`civ_consolidate_and_smooth_ways`, `civ_sea_routes`).
2. **`Math.hypot` is now genuinely test-enforced.** Milestone B ported V8's
   compensated version and honestly recorded that its fixtures could not tell it
   from `sqrt(x²+y²)`, because an `f32` store absorbed the difference.
   `_civSmoothPath` accumulates `km` in `f64` across dozens of segments with no
   rounding step, so one ULP survives: `610.6390435628962` (Rust's `hypot`) vs
   `610.6390435628963` (the reference). `cartalith-civ` now has its own
   `js_hypot` across the route-geometry chain. The crate's other `.hypot()`
   sites are deliberately left alone — covered by their own passing goldens, and
   editing verified code on an unmeasured hunch is what this discipline exists
   to prevent.

**No `PassBuffer` anywhere, deliberately.** The plan predicted this for two of
the three tools; it held for all three. Placing a settlement is one atomic
append; the in-progress waypoint chain *is* Draw way's pass-buffer unit and
`civ_commit_way` takes it as a plain slice; Territory paint's staging is
milestone C's `PaintLayer`.

**Verified:**

- **16 golden-parity tests, every one bit-exact.** `km` compared as raw `f64`
  bit patterns, not with a tolerance; the territory raster as an FNV-1a-64 over
  all its bytes. No tolerance anywhere.
- Harness: Node `vm.runInContext` over **whole `<script>` blocks** (#1
  2084-14556, #2 14563-26720), asserted by their real `<script>`/`</script>`
  delimiters — a stronger boundary guarantee than milestones B/C's line slices.
  The balance/orphan-close checks ran anyway and fired twice, **both times
  wrongly**: nested template literals desynchronised a crude string skipper, and
  regex literals containing a bare `"` were read as string openers. Both fixed
  properly rather than deleted.
- **Emptiness assertions and real negative controls**, the class of failure that
  has now bitten three subsystems: every "should route" case asserted ≥ 2 points
  and `km > 0`, every "should not route" case asserted `reachable === false`,
  the brush asserted a nonzero painted count, the drop tool exactly one place,
  the unreachable commit a non-empty warning.
- The world under the tools was checked first: `field`, water bodies, biome
  raster and Strahler order all FNV-matched this port's own `generate_terrain`
  pipeline exactly, in both cases, as did the land/ocean/lake counts. Both
  fixtures carry real ocean, land and lakes; case 1 has 42 lake cells and an
  ocean connected only through the seam.
- Everything driven from **inside** the vm context (milestone C's lesson):
  `civWays`, `_civActiveFaction`, `_civTerRadius`, `_civWayWaypoints` and
  `civTerritory` are `let` bindings, not context properties.
- Six presentation-only functions were neutralised inside the context by
  reassignment — disclosed, but no tool body was transcribed or edited, and none
  of the six touches routing, placement or paint state.
- 28 new unit tests in `cartalith-civ` (225 total, up from 197).
  `cargo build`/`cargo test`/`cargo clippy --all-targets` clean on
  `cartalith-civ`. `cargo test --workspace`: 842 passing, 0 failures. (One
  transient `cartalith-engine` GPU-determinism failure under full-workspace
  parallelism cleared on rerun and in isolation; unrelated to this work.)

**Not built, deliberately:** the interaction halves (waypoint capture, the
Escape/commit keybinding, the shared active-faction quick-select, brush-radius
and way-type pickers, the snap on/off switch) — input routing, milestone F.
Also `_civCommitRoute`/`civJourneys`, `_civDropPOI` (no POI concept here),
`_civConnectPlaceToNetwork`, and provinces over a painted territory raster.

## Phase 3 milestone 5: geological material exposure + local contrast (2026-08-18)

`TERRAIN_APPEARANCE_SCOPE.md` milestone 5, from `TERRAIN_APPEARANCE_RESEARCH.md`
§12 and §18 — the two the previous three milestones each explicitly deferred.
Presentation only: nothing here touches heightmap, climate, hydrology, biome
classification, settlement generation, routes or seed.

**Why these two.** Together they answer §30's stated objective from opposite
directions — §12 puts *more real information* into the image, §18 makes
information already present easier to separate. Rejected with reasons: §16
multi-scale detail is largely already delivered by milestone 4's paper grain
and stipple; §17 colour vibrancy would pull against milestone 4's deliberate
13-26% chroma reduction two days after making it; §20's high-precision
pipeline has no consumer in this port yet; §21 GPU is its own milestone; §29
quality tiers need more stages to tier than exist.

**The §12 plumbing question, checked before committing.** The brief flagged
Journey Planner milestone 5's `build_cart_terrain`/`CART_TERRAINS` (commit
`dca5954`). Reading it, that is the wrong source — a party-movement *surface*
vocabulary (Paved Road, Open Plains, Hills...) derived from field/water/temp/
rain, i.e. from inputs `render.rs` already reads, so it would have added a
coarse re-classification rather than new physical information. The right
source is `cartalith_civ::build_lithology`: seven `LITH_KEYS` rock types built
from the **tectonic substrate** (`age_field`/`volcanic_field`/`crust_field`/
`resistance_field`), which the renderer genuinely could not derive. And the
plumbing already exists — `cartalith-godot` depends on `cartalith-civ` and
`lib.rs` already calls `build_lithology` for the soil chain — so this is one
more call in the file that already makes it, not new cross-crate wiring.

It matters more than it sounds. Over the Classic test world's land that
vocabulary is **shale 45%, metamorphic 33%, basalt 11%, sandstone 7%,
limestone 4%, granite 0.4%** — and granite is exactly what the ported climate
heuristic paints by default. The renderer had been showing one rock for a
world that has seven.

**Built — §12.** Five new rock palettes (`rock_basalt`/`rock_andesite`/
`rock_limestone`/`rock_shale`/`rock_metamorphic`; granite and sandstone
already existed), `litho_strength`, `litho_exposure`, and
`RenderCtx::with_lithology` — a **builder**, like `with_splat`, so
`golden_parity_render.rs` stays positionally valid and untouched. Two halves:
`rock_material_col` blends the reference's own `rock_col` toward the real
rock's palette (a blend, not a replacement — the heuristic still carries
surface character, the lithology supplies identity), and bedrock **shows
through thin soil** in `land_color`, gated on §12's own list (slope,
vegetation potential, effective moisture) and scaled by the cover fraction
that is not already rock or snow, so it is self-limiting and never bleeds
through an icecap. Neither touches `material_weights` — five milestones in,
the golden-verified fraction blend has still never been edited.

The lithology index is sampled through a **coherent positional jitter**
(`RenderCtx::litho_at`, ~10-cell wavelength) rather than straight:
`build_lithology` is categorical and single-pass, so a granite/limestone
contact sampled straight renders as a clean vector line — §30's "artificial
outlines" and "hard biome borders" at once. Jittering is this renderer's own
idiom, not a new one; it is what `bio_jitter` already does for the
reference's biome classification.

**Built — §18.** `apply_local_contrast`, the **first stage in `render.rs`
that is not per-pixel**, and necessarily so: "make neighbouring terrain
materials visually distinguishable" is a statement about a neighbourhood of
the *finished* colour, which does not exist until the raster does. It runs
over the output byte buffer in `lib.rs`, after the river tint and before the
icon pass; `cell_color`'s signature and behaviour are untouched. §18's three
constraints are met by construction, not by tuning: the response is
`d · exp(−(d/knee)²)` so gain **falls to zero** on strong edges (an unsharp
halo is an overshoot proportional to edge strength; here gain is inversely
related to it), the correction is **additive and equal on all three
channels** so chroma is provably unchanged, and the band is a ~20-cell blur
at 2048² rather than a 3×3 kernel. It fades out under the plate frame via
milestone 4's own `border_cover`.

**Two real corrections, caught by measuring and by looking — milestone 3's
lesson holding for the third milestone running.**

1. *The geology gate was written in raw slope units, and raw slope is
   resolution-dependent.* `slope_at` is a per-**cell** height difference, so
   the same mountain measures far shallower on a finer grid: median land
   slope over Classic is **0.00354 at 512² and 0.00054 at 2048²**, measured
   rather than assumed. The first `smoothstep(0.008, 0.050, slope)` therefore
   confined the whole stage to the steepest ~5% of land *at the resolution
   the app actually runs at*, while looking perfectly reasonable in source.
   Fixed by normalizing to `slope * gw`, this project's own convention for
   exactly this (`cartalith_civ::build_slope_field` stores `slopeAt(x,y)*GW`):
   Classic pixels moved by >3 levels/channel went **1.17% → 6.61%**. The
   reference's own `material_weights` normalizers inherit the same dependence
   and were left exactly alone — they are golden-verified.
2. *Local contrast as a plain high-pass amplified the sheet's own texture.*
   `luma − blur(luma)` sweeps in everything finer than the radius — here
   milestone 4's ~3-cell paper grain and the C¹ seams of the value-noise
   lattices under the mottle — and the first version produced a faint
   rectangular quilting across land and sea. That is §30's "random texture
   noise", the same failure class as milestone 2's AO speckle and milestone
   4's halftone stipple, found the same way: by looking at a downsampled real
   dump, not at a statistic. Fixed by making it a **band-pass** (subtract a
   small blur instead of the raw image) so the boosted band is the material
   scale and the sheet's texture passes through untouched. Benefit intact:
   luma sd 33.10 before the fix, 33.08 after.

**Measured against §30's anti-list** (2048², seed 12345, frame band excluded;
"base" is milestone 4's look):

| | Classic base | Classic m5 | Archipelago base | Archipelago m5 | Wide 2048x1024 base | Wide m5 |
|---|---|---|---|---|---|---|
| interior luma min | 41.0 | 38.7 | 33.8 | 26.9 | 45.4 | 39.4 |
| interior luma mean | 132.75 | **131.60** | 105.98 | **105.31** | 136.98 | **135.23** |
| interior luma sd | 31.94 | **32.85** | 28.34 | **28.98** | 27.28 | **28.80** |
| interior mean chroma | 51.80 | 51.24 | 51.84 | 51.81 | 52.49 | 51.24 |
| any-channel clipping | 0.78% | **0.67%** | 0.04% | 0.04% | 0.00% | 0.00% |

Contrast **rises** in all three worlds while mean luma falls about one level
and clipping *falls* — separation bought from the middle of the range, not by
pushing anything to black or white. The whole chroma movement belongs to
geology (rock palettes are less chromatic than the tan they replace): the
local-contrast-only dump measures 51.79 against a 51.80 base, luminance-only
by construction as claimed. Luma minimum drops 2-7 levels from local contrast
deepening the darkest concavity; 26.9/255 at worst is a deep shadow, not a
black valley.

Which stage carries what (pixels moved >3 levels/channel): geology 6.61% /
0.94% / 10.75% and local contrast 24.90% / 11.69% / 31.52% for Classic /
Archipelago / Wide. Within geology the two halves split 0.94% (rock palette)
to 5.29% (soil show-through) on Classic — the show-through carries most of
it, because at 2048² the reference's own rock *fraction* is small except near
summits, the same resolution-dependence finding above.

**Cross-world honesty.** Same direction as milestones 2 and 3, not milestone
4's inversion: geology is strong on mountainous Classic and the wide plate,
nearly absent on Archipelago (0.94%) — a low-relief fragmented world simply
has little steep thin-soiled ground for bedrock to show through, which is the
honest answer rather than a knob to force. Local contrast is substantial in
**all three**, because every world has material boundaries whether or not it
has mountains — which is exactly why it was worth pairing with a relief-keyed
effect.

**Golden parity: the same gating mechanism, extended a fourth time.**
`js_reference()` gains `litho_strength: 0.0`, `litho_exposure: 0.0`,
`local_contrast: 0.0`, and each stage early-returns on its own zero rather
than evaluating to a no-op — `rock_material_col` returns the reference's
`rock_col` before touching a palette, the show-through block is inside an
`if`, `apply_local_contrast` returns before allocating. §12 is additionally
off *by data*, since `with_lithology` is a builder the golden test never
calls. **`golden_parity_render.rs` is still completely unmodified and both
tests still pass at their original `1e-4` tolerance with every expected value
unchanged** — five milestones in, that file has never been edited.

One new non-`#[ignore]`d test guards the one thing `render.rs` cannot guard
itself: it is `#[path]`-included standalone by the golden test, so it spells
the rock-type order out as `LITHO_PALETTE_ORDER` rather than importing
`LITH_KEYS`. `appearance_ab_dump.rs` sees both crates and asserts they match,
so the duplicate is checked rather than hoped for.

**Non-square correctness** (commit `22ae75b` made these real). Every radius in
`render.rs` is keyed to `gw`, so a wide short plate is where a width-derived
blur radius can exceed the map's own height; the local-contrast radius is
capped against the short axis for that reason. A 2048x1024 world was added to
the A/B harness and carried through every measurement above, and the frame
band on it is **bit-identical** before and after — 0 of 168,896 pixels
changed, so `border_cover`'s fade is exact rather than approximate.

**Cost.** 2048² render 923 → 1110 ms (Classic, +20%), 607 → 752 ms
(Archipelago), 501 → 599 ms (Wide). Local contrast is three separable box
blurs plus one `exp` per pixel; geology is one extra `vnoise` pair and a
palette blend on land only, plus `build_lithology` (one neighbour-free
`par_iter` pass). Real-app `build_color_texture` end to end: 1442 / 1085 /
761 ms. One-shot at generate time.

**Verified.** `cargo check -p cartalith-godot --all-targets` clean; `cargo
build --release -p cartalith-godot` clean (the debug cdylib hit the known
`Access is denied` DLL lock from a running editor, so the debug DLL was built
and exercised in a detached worktree instead); `cargo test --workspace` **572
passed / 0 failed**, no expected value anywhere modified; `cargo clippy -p
cartalith-godot --all-targets` clean for this crate's own files (remaining
warnings are `cartalith-gpu`'s and `cartalith-civ`'s, confirmed unrelated by
file and line). `godot4 --headless --quit main.tscn` clean load. And the real
`build_color_texture` path — which the dump harness does *not* exercise,
since it calls `render.rs` directly — was run headlessly end to end for all
three worlds and produced correct PNGs with river tint, plate frame and
non-square aspect intact.

**Still open:** hand-lettered settlement glyphs (`map_overlay.gd`, not this
raster), §16 multi-scale detail as an explicit control set, §17 colour
vibrancy, §19 atmospheric/distance effects, §20 the high-precision display
pipeline, §21 the GPU rendering path, §29 quality tiers, the GUI editing
panel (`GUI_SHELL_SCOPE.md`), and milestone 1's elevation-ramp question.

## Phase 5 milestone 4 — generation rules and culture profiles, and the one line that would have built a different town (2026-08-18)

`URBAN_MORPHOLOGY_SCOPE.md` milestone 4: `CULTURE_PROFILES`, `resolveProfile`,
`DEFAULT_RULES`, `cloneRules`, `resolveRules`, `clamp`, `applyWildness`,
`applyPlotChaos` — reference lines **28193-28280** — as
`cartalith-urban::rules`. Dependencies unchanged (`cartalith-rng` only). Wired
to nothing. 10 new tests, 43 in the crate.

**The stated range was wrong at both ends, in opposite directions**, and this is
the third range in the plan to need correcting. The plan said 28212-28289. The
start was 13 lines late — 28212 is `resolveProfile`, so the range **excluded
`CULTURE_PROFILES` entirely**, the first item the milestone's own list names.
The end was 9 lines late — 28281 is blank and 28282-28289 is the `V` vector
helper object, which milestone 1 already shipped. Milestone 5's stated start
(28557, `shoreFromMask`) was checked as a side effect and is correct.

**The reason this milestone is not "just data".** `clamp` is
`Math.max(lo, Math.min(hi, v))`, and the obvious Rust transliteration
`lo.max(hi.min(v))` is wrong: JS `Math.min`/`Math.max` propagate NaN, Rust's
absorb it. A NaN wildness slider leaves eight NaN street fields in the
reference; the naive port lands **every clamped field on its own upper bound**
instead — a maximally-wild rule set that looks entirely plausible, feeds
straight into `grow` (milestone 7), and produces a town nobody can trace back to
a rounding rule. Same trap `cartalith-assets` milestone 3 hit from the opposite
direction. The port routes `clamp` through explicit `js_min`/`js_max` mirroring
the source expression; `wild_NaN`/`chaos_NaN` goldens pin it and a test carries
the `js_hypot`-style guard so the simplification fails loudly. One documented,
unreachable divergence remains, on signed zero — and it is exactly why two
mutations survive.

**Findings in the data.** `applyWildness` is **not idempotent**: ten of its
eleven assignments recompute from a hardcoded literal times `w`, but
`deadEndBias` accumulates off its own current value, walking 0.15 → 0.30 → 0.40
(capped) under repeated `w = 2` while nothing else moves. It also silently
overwrites custom values it never reads, and touches neither `settlement` nor
four named `street` fields. `profile.deadEndBias` **does not exist on either
live profile**, so milestone 11's
`clamp((profile.deadEndBias||0)+…, 0, 0.40)` always gets zero from the profile
side — the capture asserts that absence against the reference's own key list.
Four profile fields are read by nothing at all (`parcelPattern`, whose death the
reference documents itself; `orientation`; `civicAnchorLabel`; and
`defaultWalls`, about which **the reference's own provenance prose is stale** —
it claims the UI reads it, and v2.10 has zero reads anywhere). Nothing outside
block 4 uses any of this milestone's exports: the whole host app touches exactly
three names on `UME`. `resolveProfile` has a **prototype-chain hole** —
`resolveProfile('toString')` returns a *function*, `'__proto__'` returns
`Object.prototype`, all truthy, all past the `||` fallback — captured as the
reference's real behaviour with a golden asserting this port hardens all five to
`medieval`. `cloneRules` does not survive as a function (`Copy` is the deep
clone, milestone 2's `gKey` call) but is not quite one either: a NaN
round-trips to `null` through `JSON.stringify`, pinned, unreachable inside the
engine, and impossible to reproduce in a typed struct.

**Mutation testing: 120 mutations, 114 died, 4 survived, 2 killed by the
compiler.** Every numeric literal on a non-comment line perturbed one at a time
(84), plus 36 structural mutations across both clamp semantics, both comparators,
every `js_round` alternative, the `deadEndBias` accumulation, the `2-w`
inversions, both `meta` write-backs, `resolveRules`' merge, `resolveProfile`'s
fallback and arm order, and eleven profile-table values including the array's
own order. The two compiler-killed ones are array lengths. The four survivors:
`js_min`'s `<` → `<=` and `js_max`'s `>` → `>=`, both of which only differ on
`+0` vs `-0` (the documented unreachable divergence); and the `1.0`/`4.0` clamp
bounds inside `Math.round(clamp(2*c,1,4))`, which survive a `+0.01` perturbation
and **die** at `1.0 → 1.6`, `1.0 → 0.0`, `4.0 → 4.6` and `4.0 → 3.0` — shown by
graded perturbation rather than asserted.

**A fifth survivor was killed by adding scenarios, and it generalises.** The `2`
multiplier in `clamp(2*c,1,4)` survived the first round for the same reason:
`subdivisionCap` is a **quantised output**, and a rounded value cannot observe a
change to its inputs smaller than half its own step. Three scenarios were added
sitting just *below* the rounding boundaries the existing three sit exactly on
(`chaos_0p7475`/`chaos_1p2475`/`chaos_1p7475`), and it dies. This is milestone
3's lesson arriving from the other side — there a quantised *input* was needed
before a tie-break could be observed; here a quantised *output* hides a
constant. Both are the same fact: a golden can only test what its inputs let the
function express.

**A tooling trap worth more than the milestone.** The first combined mutation
sweep reported **34 survivors**. Every one died when re-run by hand; the
structural block alone killed 34 of 36; the full 120 killed 114. The sweep had
been reporting a stale binary from partway through. It was neither of milestone
3's two traps (the mtime stamp and comment-anchored patterns were both already
in place and both held), did not reproduce on replay, and most likely came from
a sibling fork building concurrently in the shared `target/`. The durable
lesson, now written into the scope doc's verification convention: **re-run every
survivor in isolation before reporting it** — a "did the tests actually run"
gate does not catch this, because a stale binary reports a perfectly healthy
`N passed`. Add the gate anyway; it catches the adjacent case of a filter that
silently matches nothing.

**Golden verification.** All eight items are on `UME`'s *public* export rather
than `_test`, so this is the first milestone in the subsystem needing no
indirection: 53 rule cases, both profiles field by field, 15 `resolveProfile`
ids. Rule sets are flattened into one canonical field order and compared **bit
for bit** via `to_bits`, so a NaN must be a NaN and a `-0` could not pass for a
`+0`; no tolerances. The capture asserts the reference's `DEFAULT_RULES` still
carries exactly that key set in exactly that order, asserts neither profile
defines `deadEndBias`, and refuses to write unless the output is populated,
right-shaped and actually varies. Every golden matched on the first run — which
is precisely why the mutation testing is the part that counts. The slice
harness is milestone 3's verbatim (contiguous 28167-31103 plus line 2291,
balance scan with the orphan-close counter, four structural assertions), re-run
as a negative control with one new row: a slice starting seven lines early and
swallowing the end of block 3 escapes the balance scan and is caught by the
first-line assertion, the same shape as milestone 2's documented residual hole.

**Verified:** `cargo build -p cartalith-urban`, `cargo test -p cartalith-urban`
(43 passed / 0 failed) and `cargo clippy -p cartalith-urban --all-targets` all
clean. `cargo build --workspace --exclude cartalith-godot` clean (the two
remaining warnings are `cartalith-gpu`'s, pre-existing and a sibling fork's).
`cargo fmt` deliberately not run — the crate and its siblings are already not
rustfmt-clean, so it would reformat other forks' files.

**Also:** `astar.rs`'s module header still carried the superseded 28514-28556
line range; corrected to 28514-28547 in line with the scope doc.

## Unified tool plan milestone E — the Annotation & measure group (2026-08-18)

`UNIFIED_TOOL_PLAN.md` milestone E: the last of the four tool-group engine
halves. Label, Icon stamp, Measure, and the compute/encoding core of Region
select/export — all golden-verified, all wired to nothing, same
"primitive ahead of orchestration" precedent as A-D. No Godot scene, `main.gd`,
`main.tscn`, `render.rs` or any `cartalith-godot` file was touched;
`cartalith-urban` (a sibling fork's milestone 4) was left alone.

**Built, across six crates** — placement argued from A-D's rule each time
(generic machinery → `cartalith-spatial`, pipeline knowledge →
`cartalith-engine`, subsystem-domain math → the owning crate):

- `cartalith-civ/src/labels.rs` — `MapLabel`, `arc_label_layout`, `label_box`,
  `label_hit_test`, `LabelEditSession`, and the three handle formulas. The
  reference's own `_civ`-prefixed family, beside the settlements, ways and
  territory this crate already owns.
- `cartalith-assets/src/manual.rs` — `ManualIcon`, `place_manual_icon`,
  `icon_brush_rule`, `icon_brush_stamp`, `icon_box`, `icon_hit_test`,
  `icon_resize_scale`. The manual half of the rule-driven placement this crate
  already ships, reading the same `ScatterRule` table.
- `cartalith-spatial/src/measure.rs` — the Measure tool. **An addition, not a
  port** (`DECISIONS.md` §7d): the reference has no measuring tool at all.
- `cartalith-spatial/src/region.rs` — `norm_region`, `tile_dims`,
  `FloatRegion`, `js_round`.
- `cartalith-terrain/src/amplify.rs` — `amplify_region`, `refine_tile`.
- `cartalith-io/src/tiles.rs` — `pack_height16`/`unpack_height16`,
  `TileManifest`, `build_tile_manifest`, `manifest_json`.
- `cartalith-engine/src/region_export.rs` — `export_region_tiles`.
  `cartalith-engine` gains a `cartalith-io` dependency, its first.

**Region select/export needed a split, and it is recorded.** The plan asked for
one if the item stayed large. It does, but not where the name suggests:
`exportRegionTiles` is four calls and a loop, and everything hard in it is
either pure geometry (shipped here, bit-exact) or a browser API (which cannot
be). **Milestone E2** is therefore entirely format-and-pixels: per-tile PNG
(`tilePngBytes`), `gzipBytes`, the `.zip` assembly, `exportGeoJSON` with its
raster-to-vector boundary tracer, and `regionNewWorldBtn`'s replace-the-world
path. Smaller than the plan feared, and named in full in
`UNIFIED_TOOL_PLAN.md`'s "Milestone E as built".

**Three places reading the reference corrected the plan.**

1. **The plan describes the wrong icon function.** It calls
   `_carIconBrushStamp` *"stamp mode (place one icon by hand at a clicked
   point)"*. It is not: there are **three** placement paths, and the manual
   half is two of them — a four-line click-to-place branch (9776-9784) *and* a
   dart-throwing blue-noise scatter **brush**, which the plan does not mention.
   The brush is deliberately unseeded (*"a brush stroke is an authoring ACTION
   ... re-painting the same spot should add new icons"*), so `icon_brush_stamp`
   takes its randomness as a parameter and the harness overrode `Math.random`
   inside the vm context with a matching LCG.
2. **`amplifyRegion` has a real division by zero.** `outH == 1` with `rh > 1`
   evaluates `0/0` and returns an all-`NaN` tile. Verified against the
   reference, ported as written, pinned by a golden — and it forced `js_min`/
   `js_max`, because `Math.min(1, NaN)` is `NaN` in JS while Rust's `f64::min`
   returns the other operand and would have hidden it.
3. **Measure really has zero precedent**, so it ships flagged as an addition
   with no golden test and cannot have one. Its km scale is not invented: it is
   the same `hypot(dx,dy) * map_width_km / gw` `civ_smooth_path` uses, compared
   as raw `f64` bits by a test.

**Golden verification — 49 tests, everything exact but two ULPs.** Node
`vm.runInContext` over whole `<script>` blocks #1 and #2, delimiters asserted
against the real tags (milestone D's technique). Two environment modifications,
both disclosed: a recording Canvas-2D stub (`drawArcLabel`/`_civLabelBox` need
one; no function body is transcribed) and the seeded `Math.random`. The
block-comment balance check **fired twice and was wrong both times** — a `}`
closing an arrow body inside a `${ }` substitution, then a regex-literal
skipper whose "does a value precede this `/`?" test matched only a single
identifier character — and both were fixed rather than deleted. The documented
apostrophe-in-prose blind spot showed up as a *symptom* of the first.

Shape assertions ran before any golden was written down (non-constant
amplifications, an exactly-zero tile seam delta, both branches of the arc
layout, hits *and* misses *and* an overlap in the label hit test, two
legitimately-empty brush runs as negative controls, every brushed icon on land
and in bounds). The fixture field is synthetic and **pure arithmetic** — no
`sin`/`cos`/`exp`, so libm cannot disagree about the input — with a quantised
`% 11` term and both land and water in quantity; both sides FNV-1a-64 it and
every golden file asserts that hash first.

The one inexactness, measured rather than assumed: a 36-glyph arc label matches
on 106 of 108 values and is **one ULP** out on two `dx` values, both from
`r * sin(theta)`, with `dy` and `rot` exact at the same glyphs — so `theta` is
bit-identical and the gap is purely V8's `Math.sin` against Rust's. The test
pins exactly which two, so it cannot grow. Nothing branches on a glyph
position. Everything else compares with **no tolerance anywhere**.

**Mutation testing — 89 mutations, 86 killed, 3 survivors, all three shown
equivalent.** 81 killed by a golden, 5 by a unit test. Both of milestone 3's
false-survivor traps were guarded: the runner asserts cargo actually recompiled
(a missing `Compiling` line is BROKEN, not a survivor), and mutations are
applied by a `sed` address that skips comment lines with a pre-check that the
needle occurs in code.

The first pass exposed **ten real fixture-shape gaps**, every one fixed by
adding a differently-*shaped* fixture rather than by weakening the mutation:
five in the region geometry (a fractional drag too small for the minimum to
stop masking `ceil`; no explicit minimum of 0 or 1; an aspect-1 case at a tile
size where the two branches happen to agree; a region and output not collapsed
together; a hit-test probe table whose only miss was far outside every box) and
five brush constants that no golden *could* have caught — a dart always lands
on an integer cell, so no dart-versus-dart fixture can see a spacing constant
of 3.0 versus 2.9, and `max(1.2, 3/sqrt(d))` reaches its floor only above a
density the reference's own slider cannot reach. Those five are now killed by
scripted-RNG unit tests that observe each constant on its own.

The three survivors are equivalent mutants with the algebra written out:
`base < sea` vs `<=` (the taken branch computes 0 at equality, which is the
else branch's value), `x + w > gw` vs `>=` (the clamp body is a no-op at
equality), and `js_round`'s half-up rule inside `region.rs` (its only caller
feeds it positive values, where the two roundings agree — the *other*
`js_round`, in `manual.rs`, does see negatives and is killed).

**Verified:** `cargo build` / `cargo test` / `cargo clippy --all-targets` clean
on all six crates' new code. `cargo test --workspace`: 1034 passing, 0
failures. `cargo build --workspace` hit the known
`cartalith_godot.dll — Access is denied` transient (a Godot editor holding the
DLL), so it was run as `--exclude cartalith-godot` plus
`cargo check -p cartalith-godot`, both clean. `cargo fmt` deliberately not run:
several crates are already not rustfmt-clean and it would reformat sibling
forks' files.

**Not built, deliberately:** every tool's interaction half (label drag/rotate/
arc capture, the icon gallery arm/disarm, the measure tool's click capture, the
region drag-rectangle and its dashed overlay) is input routing and belongs to
milestone F; milestone E2 in full; the *rendering* of labels and icons (a
`cartalith-godot` change this milestone is scoped out of); and persistence of
`state.labels`/`state.mapIcons`, since `SAVEFILE_COMPAT.md` is read-only here
and adding a writer is its own decision.

## Second real Android device pass — the numbers moved, and the phone UI is honestly unusable (2026-08-18)

Not a milestone. A **verification pass** on the same OnePlus 6T the
2026-08-17 pass used, re-run because an enormous amount had landed since and
none of it had been on hardware: the GUI replaced twice (panel-browser shell,
then the DCC editor shell, plus the Fable-5 declutter), 57 generation controls
plus a File ▸ New world dialog, independent `gw`/`gh` (non-square maps), four
new crates (`cartalith-spatial`, `-assets`, `-urban`, tool-system code), and
terrain appearance milestones 2-5's per-pixel work in `render.rs`. Full record
in `ANDROID_BUILD_SCOPE.md`'s new dated section.

**No code was changed.** Nothing crashed, so nothing needed fixing, and the
one layout problem found is a real deferred milestone that this pass
deliberately did not start.

**Build and install still work.** `cargo ndk -t arm64-v8a build -p
cartalith-godot` is clean against the grown workspace; the APK exports
(68 MB, debug-signed) and installs first try. One new step is now effectively
required: the debug `.so` has reached **400 MB** of debuginfo (`[profile.dev]`
carries `opt-level = 1` but leaves `debug = true`), and Godot stores `.so`
files uncompressed, so it was stripped with the NDK's `llvm-strip
--strip-debug` down to 18 MB before export. Noted rather than fixed — the
real fix is a dedicated Android profile, which is a decision, not a chore.

**Golden path runs on device, driven entirely by touch.** Generate → render →
Layers overlays (Territory faction fill, Province boundaries) → settlement
selection with the **WHY HERE causal-chain explainer** populating correctly →
tool rail switching the tool options bar to `RAISE / LOWER` → View ▸
Performance readout → a `Generate — Climate` slider dragged by swipe. No
`FATAL`, no `ANR`, no `lowmemorykiller`, empty crash buffer, 60 FPS throughout
(generation is on a background `Thread`). The Performance readout reports
`0 of 6 eligible stages ran on GPU` — the GPU path is correctly inert on
Android.

**Memory has grown materially, and it is measured, not guessed.** Same
`dumpsys meminfo` / `TOTAL PSS` method as the previous pass. Like-for-like at
512x512: **peak 283,326 KB → 395,756 KB (+40%)**, steady-state 271,290 KB →
316,200 KB (+17%). At the app's *own default* of 2048x1311 (2.68 M cells) the
phone reaches **894,968 KB peak (874 MB) over ~31 s** — it completes and
renders correctly, but that is a large fraction of a mid-range device's budget
with no progress indication. **No leak**: regenerating at 512x512 after the
2.68 M-cell world returned steady-state to 309,200 KB, marginally *below* the
same session's first 512x512 run.

Timing scales with cell count: 3.2 s at 131 k, 4.5 s at 262 k, 8-9 s at 466 k,
31 s at 2.68 M. The 512x512 figure is *faster* than the previous pass's ~7-9 s
— read as "not slower", since both are inferred from the memory trace rather
than an instrumented timer.

**Non-square maps work on device, all four shapes.** 512x512 1:1, 512x256 2:1
with Extent = **Whole world** (which correctly pins the aspect to 2:1 and
disables the Aspect control), 512x910 9:16 tall portrait, and 2048x1311
1.5625:1. Each aspect-fits the viewport correctly and reports the right cells
and kilometres in the header and status bar. No bug found.

**The phone UI: structurally intact, physically unusable by finger.** The
finding is more nuanced than expected and both halves matter.

What does *not* break: the app is orientation-locked to landscape (Godot's
default `display/window/handheld/orientation`; `project.godot` has no
`[display]` section), so the shell gets a **2340x1080** surface — *wider* than
the 1920x1080 it was designed at and exactly as tall. Nothing reflows, nothing
clips, all six regions hold their proportions, the right dock keeps its full
296 px, and **every runtime-built dialog fits inside 1080 and scrolls
internally** — the 1080p dialog overflow a sibling fork reported is **not**
reproduced here. This is a load-bearing accident: unlocking orientation before
the responsive milestone ships would hand the shell a 1080x2340 portrait
surface and break all of it.

What does break: absolute pixel sizes against a 403 dpi panel. In its
landscape configuration this display puts Android's 48 dp minimum touch target
at **94 physical pixels**. The shell offers 26-44. Menu bar 34 px (2.15 mm,
36% of minimum), left tool rail 44 px wide with ~35 px pitch (2.78 mm / 2.2 mm,
47%), Layers rows 32 px (34%), status bar 26 px (28%), dropdown rows ~22 px
(1.39 mm, 23%), slider grabbers ~12 px (0.76 mm, 13%). Body text is 10-13 px
against a 24 px (12 sp) minimum. A fingertip contact patch is 110-160 physical
pixels — one touch spans the menu bar plus the workspace tabs plus the tool
options bar, or five dropdown rows, or three Layers checkboxes.

Every interaction in this pass succeeded, and that is **not** evidence a person
can perform them: `adb shell input tap` is a zero-area point at a pixel
computed from a screenshot. What it does prove is that Android *event routing*
is sound — taps land on the right controls, swipes drive sliders, popups open
and dismiss. The interaction model works; the target geometry does not.
Verdict: drivable with a stylus or fingernail, effectively undrivable by
fingertip, and the dock/status bar/tool options text (0.45-0.8 mm cap heights)
is below normal acuity at arm's length. Worst regions in order: the left tool
rail, menu and dropdown popups, the status bar. Best behaved: the dialogs.

**Deliberately not fixed.** `DCC_SHELL_SCOPE.md` and `UI_SHELL_DESIGN.md` both
scope a real 393x852 phone layout (bottom tool bar, bottom-sheet tool options,
full-height panel sheets, 44-52 px targets) and both defer it. Building any of
it here would leave the half-migrated state this project has avoided
throughout. The measurements are recorded as the spec input for that
milestone, with one correction it will need: its own 44-52 px figures must be
read as *density-independent* pixels (~86-102 physical px on this device), not
raw Godot pixels — at raw pixels the new layout would be no better than the
current one.

## Phase 5 milestone 5 — the site model, and the second V8 libm that is not Rust's (2026-08-18)

`URBAN_MORPHOLOGY_SCOPE.md` milestone 5: `shoreFromMask`, `buildSite`,
`terrainSuitability` — reference lines **28549-28741** — as
`cartalith-urban::site`. Dependencies unchanged (`cartalith-rng` only). Wired to
nothing. 16 new tests, 59 in the crate.

`buildSite` is the input contract for everything downstream: it fixes the
1700 × 1250 m box, decides where the water is, and hands back the five field
closures (`height`, `slope`, `riverDist`, `isWater`, `bankSide`) that anchors,
routes, growth, walls, parcels and buildings all query. Nothing later in the
subsystem re-derives any of it.

**The stated range was wrong at both ends again** — four for four now. The plan
said 28557-28742: 28742 is blank (`terrainSuitability` ends at 28741), and 28557
is the first line of *code* but not of the milestone, since 28549-28556 are the
site-model archetype comment and `shoreFromMask`'s own v0.98 note — the block
milestone 3 identified as belonging here when it corrected its own range.

**`Math.exp` is the second V8 libm divergence, and it dwarfs the first.**
Milestone 1 found `Math.hypot`. This milestone's very first golden run failed on
`terrainSuitability` at one probe of one site, one ulp out, and the cause was
`exp`:

| | disagreements with V8 |
|---|---|
| `f64::exp` (the platform libm) | **20,721 of 240,000** random arguments |
| `geom::js_exp` (this milestone) | **0 of 240,000** |

V8 calls `base::ieee754::exp`, which is FDLIBM's `__ieee754_exp` — argument
reduction to `[-½ln2, ½ln2]`, a degree-5 polynomial correction and a `2^k` scale.
It is *less* accurate than a modern libm, and matching it rather than improving
on it is the whole of `cartalith-rust-conventions`' float rule. Ported beside
`js_hypot`, with the same `assert_ne!`-style guard: eight golden arguments on
which the platform `exp` gives a different answer, and a test that fails if it
ever stops doing so. One measured special case is reported rather than
explained — across 244,000 arguments (240,000 random, every half- and
quarter-integer to ±20, and `1.0` at ±1 and ±2 ulp) the two agree everywhere
**except at exactly `x == 1.0`**, where V8 returns the correctly-rounded `e` and
FDLIBM returns one ulp above it. Reproduced because it was measured; unreachable
from the site model, whose `exp` arguments are never positive.

**This retro-fixes milestone 1.** `rng::logn` is `median * Math.exp(sig *
norm())` and had been on `f64::exp`; its milestone-1 goldens passed, which means
they landed on values the two libms agree about — luck, not safety. It now goes
through `js_exp` and those goldens still pass, which is the check. `logn` has
five call sites in block 4 (29524 in `grow`, 30242/30288 in `buildParcels`,
30523-30524 in `buildBuildings`), so every frontage width, plot depth and
building dimension in the town is drawn through it; milestone 12 would have
found this against a far larger golden surface.

**Findings.** `buildSite` is two sites wearing one name and which is live is
decided **per field, not per site**, so the port carries `Option<WaterCtx>` /
`Option<TerrainCtx>` rather than the single source enum the plan proposed — an
enum would have to lie about the mixed cases the host produces. `kind` is **not
a closed vocabulary**: `kind || 'river'` defaults only the falsy case, every
unrecognised string falls through to the coastline branch while still being
returned verbatim, and milestone 9 compares `site.kind === 'coast'` directly —
so `kind` stays a `String`, the call milestone 2 made about `Edge::cls`.
`!!W.riverPath` is truthy for a path **too short to be a river**, making a site
river-like while its water geometry still comes from the mask. **A bay draws one
fewer number than a coast** (31 draws against 32 — it reuses its own indent
centre as the harbour), so their `routeEnds` diverge, and the two share a seed on
purpose. One mask is read **two different ways** (truthy in `shoreFromMask`,
`=== 1` in `isWater`), so a cell holding `2` is water to one and land to the
other. `shoreFromMask`'s principal axis can **collapse to `(0, 0)`** on an
isotropic point cloud, after which the sort is a no-op and the raster's own order
survives; and the fallback eigenvector is not exotic at all — a plain horizontal
shoreline takes it every time, it is simply invisible unless the shore has points
in two rows. Out of bounds is **`undefined`, not a panic**, reachable three ways,
and the port takes the deliberate divergence *the other way* from milestone 3's
`astar` — loud there because the case cannot happen, quiet here because it can.
`bankSide` **never returns 0**. The bridge index starts at `-1` and
`Math.max(0, bi)` is the only thing placing the bridge when no slope compares.
The three analytic hills are **drawn even when a real heightfield makes them
dead**, because twelve draws are twelve positions in the substream.
`waterPoly` is **empty on two of the four paths and read by nothing inside block
4** — it exists for the renderer.

**Golden verification.** None of the three functions is on `UME`'s public export
*or* its `_test` one — the first milestone here to reach neither — so the capture
adds them to the returned object with a single anchored replacement of the
`return {` line, asserted to match exactly once; the frozen reference is never
edited. The `vm` handoff needed one thing worth recording: `const UME = (() =>
{…})()` is a **lexical binding, not a property of the context's global object**,
so `ctx.UME` is `undefined` however well the slice ran — one of the three
silently-empty-output incidents this project has shipped, met head-on with an
explicit `globalThis.__UME = UME;` and an assertion. Rasters are emitted into the
golden file rather than rebuilt on the Rust side, so both sides provably run on
identical inputs. 19 shoreline scenarios and 36 site scenarios, each with **106
probes** of the five closures plus `terrainSuitability`, compared **bit for bit**
via `to_bits` with no tolerances anywhere — including `height` and `slope`, which
run through `exp` and `js_hypot`. The capture's emptiness/shape gate has
twenty-odd clauses (the tie fixture must really tie, a bay and a coast must
really differ, `atoll` must really take the coast branch under its own name, the
all-NaN slope field must really fall back to `river[0]`, a mask of 2s must really
read as land, …), and the Rust side mirrors the shape half as its own test so a
truncated `golden.rs` cannot make the suite vacuously pass. Every golden matched
on the first run **except the one probe that surfaced `Math.exp`**.

**Mutation-tested: 271 mutations, 240 died (2 at the type level), 31 survived**,
every survivor re-run in isolation per milestone 4's rule. Every numeric literal
on a non-comment line (207) plus 64 structural mutations covering every
`js_min`/`js_max`/`js_hypot`/`js_exp` call site, every comparator and tie-break,
both Chaikin passes, the draw order and count, all six `||` defaults, both mask
truthiness tests, the sort's stability and the bilinear term order. The survivors
are reported in the scope doc by class with the invariant each rests on: ten dead
stores, six equivalent by the surrounding arithmetic, two boundary tests whose
branches compute the same number, six guards against data the reference cannot
produce, four needing an exact tie a continuous field cannot make, and three
unobservable through Rust's stable sort — that last checked rather than assumed
(the stable sort reaches every ordering decision through its `Less` arm, so
downgrading `Greater` to `Equal` still returns a sorted result).

**The first sweep is the finding, though.** It left **46** survivors and almost
none were equivalent mutants — they were two specific fixture gaps. Every
hand-built water raster was uniform along one axis (`j >= 9 ? water : land`
makes column 0 and column 16 identical, so no `maskIdx` `i`-clamp mutation is
visible), and a fixed `[0.1, 0.5, 0.9]²` probe grid never once entered the
10-40 m band around the river where every threshold in this milestone lives.
Rebuilding the probes **out of the site's own polyline** — offsets straddling the
water band at three points along the centreline, and a ±0.25 m ladder either side
of (and exactly on) the real waterline at nine abscissae — plus a per-column
ripple in every mask took the count 46 → 35 → 31 over three rounds. Fifteen
constants were killed by fixtures rather than argued away, including several that
needed one built on purpose: a **seed scan** for a channel whose drift actually
saturates its upper clamp (no hand-picked seed does), an 18.85 m-per-segment
river whose quay walk accumulates 94.25 m in five steps — just under its own 95 m
stop — a two-row shoreline (a one-row one cannot show the fallback eigenvector,
because sorting a row-major list by *y* is the identity), the same cloud at 4 mm
cells to push the eigenvalue discriminant below 1, and a vertical shoreline so
the harbour search's reference *y* decides. **Milestone 3 asked for quantised
inputs and milestone 4 for just-below-a-boundary inputs; milestone 5 adds that a
geometric subsystem needs its fixtures derived from the geometry under test.**

**Corrections written forward**: every milestone from here must use
`geom::js_exp` for `Math.exp` (milestone 7's `logisticRamp` is the next direct
call site); milestone 6's `placeAnchors` can reach its literal market fallback,
because a landlocked site has neither a `bridgePt` nor a `harbour.pt`; milestone
9's `site.kind === 'coast'` is a string test an enum would have broken; milestone
10 must not read `site.waterPoly` as the town's water; and milestones 6-16's
stated ranges are all still unverified.

**Verified:** `cargo build -p cartalith-urban`, `cargo test -p cartalith-urban`
(59 passed / 0 failed) and `cargo clippy -p cartalith-urban --all-targets` all
clean. `cargo build --workspace --exclude cartalith-godot` clean (the two
remaining warnings are `cartalith-gpu`'s, pre-existing and a sibling fork's).
`cargo fmt` deliberately not run — the crate and its siblings are already not
rustfmt-clean, so it would reformat other forks' files.

**Also:** `js_min`/`js_max` moved from `rules` to `geom`, beside `js_hypot` and
now `js_exp` — the site model gave them a second set of call sites and `geom` is
where the "JS semantics, not Rust's" helpers belong. No behaviour change; the
milestone-4 tests that document them are untouched and still pass.

## Phase 2 milestone 20 — `_civFactionAggregates`, and the blocker it clears (2026-08-18)

The last unstarted piece of Phase 2's economy work (`ECONOMY_SCOPE.md`'s own
"real next milestones" item 3), taken now because it is a real blocker for
something already built: the GUI parity audit (`d84dfd0`) had to re-classify
`civ_culture_terrain_fit` from "just needs wiring" to genuinely blocked,
because it takes a per-faction `terrain_mix` and a `world_mean_terrain` that
**nothing computed**. That is `_civFactionAggregates`.

**Ported** (`cartalith-civ`, reference lines 23575 / 23566 / 23557 / 23553 /
22450):

- `civ_faction_aggregates` + `FactionAggregatesInput`/`FactionPlace`/
  `FactionAggregate`/`FactionAggregates`/`FactionPower`/`SectorOutput` — the
  one `O(GW·GH + nPlaces)` pass: per-faction population, territory km², food
  production capacity and surplus, trade volume, mean economic importance,
  fortified fraction, tax income, 15-key resource means, six-way sector
  output, craft share, the export/import/strategic lists, the capital pick,
  and v1.55's five-axis "Territory Fit" terrain mix — plus the world means of
  both the 15 resource keys and the 5 terrain axes.
- `civ_tax_rate` (`CIV_TAX_RATE`), `CIV_PRIMARY_SPECIALISATION`,
  `CIV_TERRAIN_MIX_KEYS`, `civ_ocean_dist_field` (`_civOceanDistField` — the
  coast axis needs an ocean-only chamfer DT, and this crate had `chamfer_dist`
  only as a private helper), and `js_min`/`js_max` (this crate did not have
  them; `cartalith-terrain` and `cartalith-urban` each carry their own).

**The design decisions, and what they were weighed against.**

- **The five-axis "power" composite is ported verbatim**, not simplified. It
  is the one place `ECONOMY_SCOPE.md` flagged as needing a real judgement
  call. The reference labels it honestly rather than dressing it up
  ("explicitly derived/heuristic, never presented as simulated"; `cultural`
  carries its own "population-proportional placeholder — no spread/
  assimilation model exists"). Simplifying it would have meant inventing a
  *different* heuristic with nothing to check it against — strictly worse
  than porting a disclosed one faithfully.
- **`CIV_MAX_TIER_RANK` is 5, not 4.** The reference's `maxRank` is taken
  over its *full* ten-entry `CIV_SETTLEMENT_CLASSES` table, whose top entry is
  `metropolis` at rank 5 — a tier this port does not model. Normalising by 4
  (this port's own highest tier, `capital`) would have inflated every
  faction's `capitalTierNorm`, and with it the military and political axes, by
  25%. Caught by reading the reference's `Math.max(1,...map(c=>c.rank))`
  rather than by assuming the port's own enum was the whole table; pinned by
  a unit test and killed by a mutation.
- **Four fields this port does not have are caller-supplied, not invented.**
  Verified by grep across every crate: `p.tradeVolume`,
  `p.economicImportance`, `p.specialisation` and `_umInferWalls(p)` have no
  producer anywhere here. They are fields on `FactionPlace`, and
  `FactionPlace::from_settlement` fills each with the value the reference
  itself computes when the field is absent (`||0`, `||'craft'`, `false`). The
  golden harness captured the reference's own `_umInferWalls` verdict per
  place and feeds the same booleans in, so `fortifiedFraction` and the
  military axis are genuinely tested rather than trivially zero on both sides.

- **One real JS-semantics trap, found by re-reading rather than by a test.**
  The reference guards every per-place number with `||0` and every
  divide-by-max with a truthiness check. **`NaN` is falsy in JS**, so a `NaN`
  population is absorbed *at the place*; a plain Rust read of the same `f64`
  would carry it forward and turn a faction's whole row into `NaN`s the
  reference never produces. Ported as `js_num_or_zero`/`js_truthy_num` with a
  unit test on the absorbed case. Disclosed consequence: because those
  coercions land first, no `NaN` can reach the power clamp through any
  caller-supplied field, so `js_min`/`js_max`'s NaN behaviour is proved by
  direct unit tests on them rather than through the aggregate.

**The resource-residency tension does not bind here** — checked against what
the code actually does, not inherited. `compute_civilisation()` still frees
the six unused resource fields immediately *before* `assign_territory`, so the
tension is live for any caller wanting the resource means. But the half of
`_civFactionAggregates` that unblocks `civ_culture_terrain_fit` — Territory
Fit — needs no resource field at all, and `resources` is an `Option` that
directly ports the reference's own nullable `pots` (every use guarded by
`if(pots)`), with its absent branch a real tested path. So the memory
decision stays with whoever adds a real caller, where it is exactly one line
(move that free below `assign_territory`), rather than being paid for now on
speculation.

**`civ_culture_terrain_fit` is now genuinely callable**, and the golden test
proves it rather than asserting it: it is invoked for all seven cultures ×
all seven factions in both fixtures, straight off the aggregate output, and
compared against the reference's own `_civCultureTerrainFit` over the same
aggregates (`common`/`imperial` correctly `None` on both sides).
`GUI_FEATURE_PARITY_SCOPE.md`'s item 5 is a wiring job again rather than a
blocked one.

**Verified.**

- **Golden-parity**, two cases, Node `vm.runInContext` over whole `<script>`
  blocks #1 (2084-14556) and #2 (14563-26720) asserted by their real
  delimiters, with the standing block-comment-balance check — which earned
  its keep twice by being **wrong**: a false "newline in regex" on
  `raw[i]/=cRange` (a `/` after `]` is division), then a false one inside
  `_jpPackRange`'s hint builder, where a `${...}` substitution was closed by
  the first `}` inside it, so an IIFE's `try{...}` ended the substitution
  early and the rest of the template literal was scanned as code. Fixed with
  a per-substitution brace-depth counter, not by deleting the check. Both
  blocks additionally compile through `new vm.Script(...)` first.
- **Six input hashes exact** (field, biome raster, lithology, water access,
  ocean distance transform, river mask) plus the territory raster. **Two are
  not, disclosed rather than papered over**: `tempField`/`rainField` differ
  from this port's by 1–3 f32 ULP in a minority of cells (case 0: 1
  temperature cell of 432, 178 rainfall cells, max relative 2.7e-7) — a
  **pre-existing** climate-chain property entirely upstream of this milestone
  — which propagates into carrying capacity, NPP, population density and the
  resource potentials. It changes nothing categorical (no river cell crosses
  the flow threshold, no biome class changes, no lithology class changes; all
  three hashes are exact). So density/flow are compared by land-cell sum at
  1e-6 relative, resource means at 1e-6, the two `Math.round`ed density sums
  to ±1, and everything else at 1e-9 or exactly.
- **Fixture shapes reach the edges on purpose**: a faction with neither
  territory nor settlements, one with territory but no settlement, one with
  exactly one settlement, a zero-population hamlet, an unmapped
  specialisation, an out-of-range faction id, and (both cases) a faction whose
  territory spans the x=0/x=gw−1 seam — case 1 additionally has settlements on
  both sides of it. Non-emptiness asserted explicitly.
- **15 unit tests** for what a golden from a real world cannot reach: `NaN`
  absorption at the place (`||0`) and NaN propagation through the power clamp
  (`js_min`/`js_max` vs `f64::min`/`f64::max`), the pre-world guard including
  the reference's own `worldMeanResource == {}` / zero-filled
  `worldMeanTerrain` asymmetry, a wrong-length territory raster, territory ids
  at or past the faction count, `Math.round`'s negative half, the
  elevation-denominator floor, the absent-resource path, the religion flag
  (every fresh world is all-`'none'`), the capital tie-break, the craft fold,
  `from_settlement`'s defaults, and `civ_ocean_dist_field`'s ocean-only vs.
  fallback distinction.
- **Mutation testing**: 58 mutations across the new
constants and branches, each applied to a unique **code-only** anchor
(checked to occur exactly once outside any comment line -- the
"pattern matched inside a comment" trap), each run **alone with a full
rebuild**, never as a combined sweep, because a stale binary reports a
healthy `N passed`. **56 killed.**

The first pass's six survivors were not six equivalent mutants. **Four were
real fixture gaps**, closed with new unit tests and then re-killed:

1. The religious axis's `0.7/0.3` weights were invisible because the only
   fixture exercising them had both normalisers saturating to 1, where
   `0.7+0.3` and `0.6+0.4` are the same number. Fixed with unequal
   populations.
2. The territory guard's **upper** bound (`f >= nF`) was never exercised --
   the synthetic raster only ever assigns valid ids. Fixed with a raster
   containing `nF` itself and a far-out id.
3. `Math.round` is round-half-**up** (toward +inf); Rust's `f64::round` is
   round-half-away-from-zero. They differ only on a negative half, and
   `foodSurplus` is the only rounded value here that can go negative. No
   generated world lands on an exact half; a unit test now does.
4. The `Math.max(1e-6, 1-sea)` elevation-denominator floor never activates
   at a real sea level. Fixed with a near-ceiling `sea` where it does.

**Two are genuine equivalent mutants**, and both were *proved* genuinely
tested with discriminating variants rather than accepted on assertion:
`coast <= 1.5 -> 1.6` cannot change anything, because a chamfer distance is
a sum of 1s and sqrt(2)s and `(1.5, 1.6]` is empty (`1.4` and `2.5` both
kill); `flow > thresh -> >=` cannot, because no accumulated discharge lands
exactly on the threshold (`x2` and `/2` both kill).

Two further mutations reported **stale anchors** rather than results --
caused by this milestone's own mid-sweep addition of the `||0` coercions,
which renamed the lines they targeted. Re-run against the corrected anchors;
both killed.
- `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ`,
  `cargo clippy -p cartalith-civ --all-targets` clean,
  `cargo test --workspace` 0 regressions.

**Not wired to any caller** — `compute_civilisation()` untouched, no `#[func]`,
no GDScript. All UI work is on hold (owner, 2026-08-18,
`DCC_SHELL_SCOPE.md`), and the standing "don't wire in what nothing calls"
rule applies to the engine side too.

## Unified tool plan milestone E2 — Region select/export's format-and-pixels half, plus GeoJSON (2026-08-18)

The other half of the split milestone E made honestly: per-tile PNG, gzip, the
`.zip` assembly, `exportGeoJSON` with its raster→vector boundary tracer, and
the non-UI core of `regionNewWorldBtn`. Milestone E predicted *"a smaller E2
than the plan feared"* and that held — every deferred item is done, and what
grew was the verification, not the code.

**Ported, and where it landed:**

- `cartalith-terrain/src/tile_render.rs` — `hypso` (8332), the `SEA`/`LAND`
  palettes (8330-8331), `lerp`/`mix` (8304-8305), the four v1.29 edge
  extrapolators (11606-11609), `renderHeightTileRGBA` (11610), and ECMA's
  `ToUint8Clamp`. A height ramp plus a normal-from-height shade is a height
  formula start to finish — milestone B's subsystem-domain category, and the
  same reason milestone E put `amplify_region`/`refine_tile` here. Touches no
  canvas and no encoder. 13 unit tests.
- `cartalith-spatial/src/geo.rs` — `_geoXY` (12491), `_geoTraceMaskRings`
  (12500), `_geoRingArea` (12526), `_geoPointInRing` (12527),
  `_geoMaskOutlineCoords` (12540), plus `js_to_fixed` and an `id_mask` helper.
  Every one operates on a binary mask over a grid plus a km scale and knows
  nothing about what the mask means — the reference proves the point by calling
  one shared helper from both the territory and the province exporter. 15 unit
  tests.
- `cartalith-io/src/gzip.rs` — `gzipBytes`/`gunzipBytes` (11582/11585) over
  `flate2`, beside `pack_height16`, which produces the bytes being compressed.
  6 unit tests.
- `cartalith-assets/src/archive.rs` — `zipStore` (12009) **generalised**; see
  below.
- `cartalith-engine/src/geojson.rs` — `exportGeoJSON` (12576),
  `_geoTerritoryFeature` (12557), `_geoProvinceFeature` (12569), and a
  `JSON.stringify`-exact writer. 9 unit tests.
- `cartalith-engine/src/region_export.rs` — `tilePngBytes` (11871, height
  branch), `exportRegionTiles`' gzip and PNG steps, the `refineBtn` handler's
  `.zip` assembly (13191) minus the download, and `extract_region_as_world`.
  18 unit tests.

**The zip and PNG conventions matched — because they are the same function.**
The reference has exactly one zip writer with three callers (the asset-pack
exporter, the project export, the region export). Rather than write a second,
`cartalith-assets::archive` grew a neutral `zip_store`/`zip_store_bytes` and
`write_pack_entries` became a one-line alias; `cartalith-engine` gained a
`cartalith-assets` dependency, which it needed anyway for `raster::encode_png`
(Phase 4's `image`-crate encoder, `default-features = false`, `png` only).
Milestone 2's two recorded conventions carried over unchanged: `.png` entries
STORED, every timestamp frozen at 1980-01-01.

**One convention milestone 2 had skipped turned out to be reachable.**
`zipStore` falls back to STORE whenever DEFLATE does not actually shrink the
entry. Milestone 2 read that as a browser size concern and deliberately did not
port it. Running the reference's own `zipStore` on a four-entry archive shaped
like a region export shows **three of four entries come back STORED** — the
`.png`, a 7-byte `params.json` whose deflate header costs more than it saves,
and an incompressible blob; only the height tile deflates. `deflate_helps` now
measures first and chooses second. `cartalith-assets`' existing tests were
unaffected (a real `pack.json` still deflates), and `archive.rs`' milestone-2
note is corrected rather than left contradicting the code.

**A STORE-only archive is byte-identical to the reference apart from two fields
no reader interprets** — the version-needed/made-by word (`zip` writes 1.0 for
a stored entry, the reference hardcodes 2.0) and the external file attributes
(`zip` stamps unix 0644, the reference writes 0). The golden normalises exactly
those and then demands every one of the other 172 bytes match. Deflated
entries, gzip streams and PNGs cannot match and were never going to
(`miniz_oxide` here, the browser's zlib and PNG encoder there); for those the
*decisions* are golden-verified, the *pixels* are golden-verified before
encoding, and the containers are verified by round trip in both directions.
Reproducibility survives: gzip's MTIME is pinned to 0, the zip's timestamps to
1980, so the same export twice is the same bytes.

**Four things the reference corrected:**

1. **`Uint8ClampedArray` is not a cast.** `out[p] = c[0]*s` stores a float, and
   `ToUint8Clamp` rounds **ties to even** after clamping and mapping NaN to 0.
   `c[0]*s` is fractional almost everywhere, so `as u8` would be wrong in
   roughly half of all pixels.
2. **`hypso` extrapolates past its own palette into negative channels.** The
   depth ramp is unclamped, so at `sea = 0.3` a `v` of `-0.1` returns
   `[-0.67, -10.67, -16.67]` — verified, not inferred, and pinned by a golden.
   Harmless only because the clamped store catches it.
3. **`Number.prototype.toFixed` rounds ties to the larger n**, where Rust's
   `{:.3}` rounds to even. Reachable, not theoretical: an 800 km map on a
   12 800-cell grid has `cellKm == 0.0625`, an exact tie at three decimals — JS
   says `0.063`, Rust says `0.062`.
4. **The tracer's JS `Map` semantics are observable.** Ring discovery follows
   insertion order, and the checkerboard pinch the reference says it *"doesn't
   disambiguate"* works by one cell's edge overwriting another's at the same
   key. From outside that is an **unclosed ring**, and `_geoRingArea`'s
   `i < len-1` then omits its closing segment. Reproduced exactly.

`tilePngBytes`' **biome** branch is deliberately not ported: it needs the whole
climate stack sampled off the coarse grid, which is a Phase 3 rendering concern.
The height renderer is the reference's own default and its own fallback.

**`regionNewWorldBtn` is a UI action with a real computational core.** The
button stays unported (all UI work is on hold — owner, 2026-08-18,
`DCC_SHELL_SCOPE.md`). `extract_region_as_world` is what it computes before it
mutates anything: `tileDims(sel,1,1,ts)` for the new grid,
`max(1, mapWidthKm * sel.w / GW)` against the **old** `GW`, and the amplified
field. The rest — `allocate()`, `refreshClimate()`, clearing the civ layer,
`confirm()`, `_setupOpen('calibrate')` — is orchestration over a live world the
shell owns, and is listed in the function's doc comment rather than half-built.
Two reference decisions are kept: it deliberately does **not** normalise, and
clearing civ data is the honest answer rather than a subtly-wrong remap.

**Verified — and one harness bug that looked exactly like a reference bug:**

- Node `vm.runInContext` over whole `<script>` blocks (#1 2084-14556, #2
  14563-26720), delimiters asserted against the real tags. The block-comment
  balance assertion ran on both and passed clean this time (1203 and 187 open
  comments), with milestone E's two skipper fixes still in place.
- Milestone E disclosed it never invoked `exportRegionTiles` itself. **E2
  did** — Node has `CompressionStream`, and `tilePngBytes` returns `null`
  headlessly exactly as the reference documents. The first real call disagreed
  with milestone E on the **fourth tile only**. Cause: with the DOM stubbed,
  block #1's boot code schedules a deferred first `generate()` on a timer, and
  the reference's `microtask()` is literally `setTimeout(r, 0)` — which
  `exportRegionTiles` awaits between tiles. The boot work fired between tile 3
  and tile 4 and overwrote `field` mid-loop. `amplifyRegion` called twice in a
  row is bit-identical; the harness was not. Fixed by making
  `requestAnimationFrame` inert and draining pending macrotasks before
  installing any fixture, after which all four tiles match milestone E's
  recorded hashes exactly — which **discharges milestone E's disclosure**: the
  assembly matches, not just its four primitives.
- **18 golden-parity tests + 61 unit tests, everything bit-exact with no
  tolerance anywhere**: `hypso` as raw `f64` bit patterns, six rasters as
  FNV-1a-64 over every byte plus their first and last twelve, both GeoJSON
  documents as whole strings (2136 and 924 characters), and a STORE-only zip as
  bytes.
- **The trig agrees.** `renderHeightTileRGBA` calls `Math.sin`/`Math.cos` on
  the sun azimuth and the byte-exact match holds across four azimuths (0, 45,
  200, 315) — not one lucky argument. Worth stating given Phase 5 milestone 5
  found `f64::exp` diverging from V8's on 20 721 of 240 000 arguments.
- **58 mutations, 54 killed, 4 survivors.** The first sweep started at 47/10
  and **six of those ten survivors were real fixture gaps**, each a constant no
  golden could have caught as first written: `_geoXY`'s three decimals (every
  fixture coordinate was a whole km or a clean `.5`); the tracer's
  `ring.length >= 4` filter in **both** directions; the shell/hole split's
  `area > 0`; `v < sea` in the shading branch (no pixel sat exactly at sea
  level); and `strahlerOrder`'s spelling (the GeoJSON world traced no river).
  Whether the degenerate ring shapes were even *reachable* was settled by brute
  force rather than argued — all 65 536 masks on a 4x4 grid through the
  reference's own tracer, which finds length-4 rings for 1 695 of them,
  length-3 rings for 8 760, and rings of area exactly zero. All six closed with
  reference-derived fixtures.
- The **four remaining survivors are equivalent mutants**, each with its
  algebra recorded in `UNIFIED_TOOL_PLAN.md`: the smallest-enclosing-shell
  tie-break (nested shells cannot have equal area), the sea ramp's `d < 0.5`
  (both branches return `SEA[1]` at exactly 0.5), V8's compensated `Math.hypot`
  versus the naive form (a ≤2-ULP difference cannot survive an 8-bit quantiser
  — milestone B's survivor for the same reason), and `tile_dims(sel,1,1,ts)`
  versus `(2,2,ts)` (the aspect ratio cancels when `cols == rows`; the
  asymmetric `(2,1)` control **is** killed). Every survivor was re-run in
  isolation, because a stale binary reports a healthy `N passed`.
- `cargo build --workspace`, `cargo test --workspace` (**1150 passing, 0
  failures**) and `cargo clippy --all-targets` on all five touched crates: all
  clean. The `cartalith_godot.dll` access-denied transient did not appear this
  run; the `cartalith-engine` GPU-determinism test failed once under full
  parallelism and passed on its own, the known transient.

**Not wired to any caller** — no `#[func]`, no GDScript, no Godot file touched.
All UI work is on hold, and the standing "don't wire in what nothing calls"
rule applies to the engine side too. The unified tool plan now has **only
milestone F** (shell wiring) left.

## Phase 3 milestone 6 — the GPU question answered by measurement, and §29 quality tiers (2026-08-18)

`TERRAIN_APPEARANCE_SCOPE.md` milestone 6. Research §21 (GPU path) and §29
(quality tiers), presentation-only throughout: nothing touches the heightmap,
climate, hydrology, biome classification, settlements, routes or the seed.

**The finding that shaped the milestone.** GPU compute *is* reachable — not
through Godot's renderer (`gl_compatibility` still cannot dispatch
`RenderingDevice` compute) but through the standalone `wgpu` instance
`cartalith-gpu` already owns; measured on this session's adapter at 2048²,
GPU-safe noise runs in 2.8 ms against 36.8 ms of single-thread CPU. But the
renderer was not GPU-bound: `build_color_texture`'s per-pixel loop had grown to
~1 s at 2048² **on one thread**, while every engine crate feeding it has been
Rayon-parallel since `CPU_MULTITHREADING_SCOPE.md` milestones 2-3. It was the
last O(gw·gh) serial loop in the workspace.

**Built — the parallel appearance pass.** `cartalith-godot` gains `rayon = "1"`
(the same declaration five sibling crates carry; nothing new enters the
dependency tree). `build_color_texture`'s loop is now `par_chunks_mut` over
rows with its body unchanged (river tint included); `apply_local_contrast`'s
luma build and correction loop are parallel; `box_h` — the horizontal half of
every separable box blur in `render.rs`, which also serves `build_ao`,
`build_hydro_wetness` and `smooth_sea_h` — is row-parallel, and the two
independent blurs inside `apply_local_contrast` run under a `rayon::join`.
`box_v` is deliberately left serial: it walks columns, which rayon cannot
express over a flat buffer without `unsafe`, and the transpose alternative
would double memory traffic and touch the JS-parity path.

Bit-identical rather than approximately so, checked three ways: a new test
comparing serial and parallel renders byte-for-byte at all four tiers; the A/B
harness `assert_eq!`ing both paths at 2048²; and a re-run of all 48 dumps after
the `box_h` change, diffed against the pre-change files — 48 of 48 identical.

`cell_color` alone: Classic 2048² **1040 → 125 ms (8.3×)**, Archipelago
**665 → 70 ms (9.5×)**, Wide 2048×1024 **583 → 61 ms (9.5×)**. End to end in
the real app (headless Godot, same DLL, same world at the app's own 2048×1311,
`RAYON_NUM_THREADS=1` versus unset): `build_color_texture` **955 → 293 ms,
3.3×**. The remainder is the lithology build, the `PackedByteArray` copy, the
`Image` construction and the serial `box_v` halves. The 955 ms single-thread
figure also confirms the baseline — milestone 5 published 1442 ms at 2048²,
which scales to ~924 ms at this resolution.

**Built — §29 quality tiers.** `QualityTier`
(`Performance`/`Balanced`/`Quality`/`Ultra`) with `for_tier`, `name`,
`from_name`, `ALL` and a free `recommended_quality_tier()`, surfaced as
`get_quality_tier`/`set_quality_tier`/`list_quality_tiers`/
`get_recommended_quality_tier` on `WorldGen`. `Quality` returns
`TerrainAppearance::default()` *unchanged and unreconstructed*, so the ladder
cannot drift from the look milestones 1-5 tuned — verified by test and by all
three 2048² tier dumps being byte-identical to that world's existing `after`
dump.

**The tier table is built from a measurement that contradicts §29's own
recipe.** A new `cost_table` in the A/B harness disables exactly one stage at a
time, best of three, at 2048². Local contrast costs 30-53 ms and the paper's
four `vnoise` calls ~6-18 ms; stipple, geology, hydrology, AO **and dropping
five of the six light directions** all sit at or below the noise floor. §29
prescribes "basic hillshade, no expensive AO" for its cheap tier, which assumes
raymarched AO and a shading pass per light — neither is what this renderer
does. Building to §29's text would have surrendered milestone 2's relief
legibility to buy nothing measurable, so the ladder drops stages in measured
cost order instead: Performance loses local contrast, paper fibre and mottle,
stipple and geology while keeping all six lights, AO and the hydrology tint;
Balanced is exactly Quality minus local contrast and paper mottle; Ultra raises
lights to ten, AO to 0.32 and local contrast to 0.62. Every tier keeps the
paper tint, the paper wash and the plate frame — the ladder drops texture,
never identity, and a test asserts it.

Cost of the ladder (parallel, 2048², local contrast included): Classic
**74/101/162/163 ms**, Archipelago **38/58/127/130 ms**, Wide
**40/53/88/89 ms**. Performance is 2.2-3.3× cheaper than Quality. Ultra costs
the same as Quality — an honest result, and why `recommended_quality_tier()`
never proposes it.

**Policy stayed with the owner.** `WorldGen` still starts at `Quality` on every
device; the recommendation function reads `available_parallelism()` (capping
Android one rung lower) and only *offers* a tier. The Android pass's
874 MB / ~31 s at 2048×1311 is the real consumer, and this milestone hands it
two independent levers without deciding the default.

**Golden-parity: the gating mechanism extended a fifth time.** `paper_tone`'s
fibre and mottle now each early-return on their own zero instead of sharing
`paper_strength`'s single gate — milestone 2's `relief_lights <= 1` rule
applied one level finer, and what makes a smooth-sheet Performance tier cost
nothing rather than computing four `vnoise` calls and multiplying by 0.
`js_reference()` needed no new fields (`paper_strength: 0.0` short-circuits
ahead of both), and `for_tier` is never on the parity path.
`golden_parity_render.rs` is **still completely unmodified** and both tests
still pass at their original `1e-4` tolerance. Six milestones in, that file has
never been edited.

**The §21 verdict, with the arithmetic.** A GPU appearance path would still win
on raw kernel time, but the ratio moved: appearance was ~955 ms of a ~6.5 s
generate+render (15%) this morning and is **293 ms of ~5.9 s (5%)** now, so a
perfect GPU port would save about 5% of the time to a new world — against a
WGSL port of `material_weights`, 25 palettes, the jittered micro-ramps, ten
`vnoise` call sites, the lithology jitter and the AO/hydrology tables, in
`f32`, producing a second renderer that diverges from the golden-verified one
under `DECISIONS.md` §7c and has to track every future appearance milestone.
Not started. If picked up later the beachhead is `apply_local_contrast`, not
`cell_color`: the largest single stage, a self-contained whole-raster pass that
reads no world fields, so one upload and one download and no material logic.

**One pre-existing artifact found by looking, deliberately not fixed.** The
full-sheet downsample shows rectangular blockiness in the open ocean — cells
~80 grid units across at 2048². It is present in the `js_reference` dump too,
and *more* visible there, because milestone 4's paper wash mutes it: the source
is `seaColorCore`'s own `n_low` value-noise sample at `25.6/gw`, whose lattice
seams show at that spacing. On §30's anti-list, inherited from the reference
HTML, and fixing it means deviating from the golden-verified path under §7d.
Recorded rather than changed inside a performance milestone.

**Anti-list, all four tiers × three worlds** (2048², frame band excluded):
contrast (`luma sd`) rises up the ladder in every world — Classic
31.48/31.35/32.79/33.06, Archipelago 27.73/27.76/28.93/29.01, Wide
26.82/26.82/28.60/29.13 — while clipping never rises with it (Classic *falls*,
0.78% → 0.68%) and chroma moves at most 1.5 out of ~52 across the entire
ladder. Luma minimum falls up the ladder (deeper concavities) but never below
26.8/255. Every tier differs from Quality in 16-53% of interior pixels, so none
is a placebo. One honest non-monotonicity: Balanced's `sd` on Classic sits
slightly below Performance's, because Balanced adds geology (rock palettes are
less contrasty than the tan they replace) while still lacking the
local-contrast pass that is what raises `sd`.

**Real crops, at the maximum-difference window** (256² integral-image search,
not a guessed location), 3×, all three worlds: Classic Performance keeps the
glacial tongue, shaded ridge flanks, coastal escarpment and settlement dot and
loses the sheet fibre, the sandstone warmth and the snow/rock crispness;
Archipelago is the clearest case for the stipple (smooth green wash versus real
clumped canopy); Wide landed on an impact crater whose rim reads as pale
limestone at Quality and uniform tan at Performance, with the plate frame
correct on all four sides at 2:1. Quality versus Ultra is barely separable even
at the maximum-difference window.

**Verified.** `cargo build -p cartalith-godot` clean (debug and release);
`cargo test --workspace` 1156 passed / 0 failed across 89 suites, no expected
value modified; `cargo clippy -p cartalith-godot --all-targets` clean with zero
warnings for this crate including its test targets (remaining workspace
warnings are `cartalith-gpu`'s and `cartalith-civ`'s, concurrent forks',
confirmed by file and line); `godot4 --headless --quit main.tscn` clean load,
exit 0. Eight new tests in `tests/appearance_tiers.rs` (synthetic 128×79 field,
no generator, 40 ms — belongs in the ordinary sweep). Mutation-tested: forcing
`paper_tone`'s mottle branch off and collapsing `Balanced` into `Quality` were
each introduced deliberately and each caught by the intended test.

## JS-semantics fidelity audit — the whole workspace, at last (2026-08-18)

Not a milestone: a verification pass. Five JS-vs-Rust semantic divergences had
been found over this port's life, each by whichever milestone tripped over it,
each *after* the code had passed golden tests, and nobody had ever swept all
fourteen crates for the rest of them. `JS_SEMANTICS_AUDIT.md` (new, repo root)
is that sweep and is written to be read *before* porting, not after a fixture
disagrees.

**Two real bugs found and fixed, both in `cartalith-spatial`, both proved with
a test that fails before and passes after.**

1. **`PaintStamp::apply` painted rim cells the reference skips.** `_paintAt`'s
   gate is `Math.hypot(dx,dy) > R`, and the module comment said the exact rim
   set depends on it — then used `f64::hypot`, which disagrees with V8 on
   **1 398 of the 4 096** integer offsets in `[0,64)²`. An exhaustive scan of
   every integer radius `1..=512` finds 25 radii where a *cell* actually
   changes, the first at `R = 125`: `35² + 120² = 125²`, so the true distance
   is exactly the radius, `f64::hypot` returns `125.0` and paints, and V8
   returns `125.00000000000001421` and skips. Eight cells per stamp. Not live
   (the reference's sliders cap the radius at 40 and 20), but `PaintStamp::new`
   takes an uncapped `f64`, and the invariant the module claimed — "for every
   integer radius … identical" — was simply false; the real one was "R < 125".
   Fixed by computing V8's `Math.hypot`.

2. **`js_to_fixed` rounded down on roughly one value in ten.** The
   `Number.prototype.toFixed` behind **every GeoJSON coordinate and way
   length**. `round_up = first > 5 || (tie && !neg)` carried two bugs: a first
   dropped digit of `5` with any nonzero tail rounded *down* (`9.051 -> 9.0`
   where V8 gives `9.1`; `286.4957967118851 -> 286.49` where V8 gives
   `286.50`), and a negative tie rounded toward zero (`-0.0625 -> -0.062`;
   ECMA-262 21.1.3.3 step 6 strips the sign *before* picking "the larger n", so
   V8 gives `-0.063`). Both collapse to one rule — round the magnitude, ties
   away from zero — so the fix is `round_up = first >= 5`.

   **`golden_parity_geojson.rs` calls this function on every feature it exports
   and could not see either bug**, and passes unmodified now: its world is
   600 km over 12 cells, so `cell_km` is exactly `50` and every coordinate it
   rounds is already an integer, and its one fractional value (`38.4567` km)
   has `6` as its first dropped digit — the branch that was right. A fixture
   chosen to cover every *feature type* covered no *rounding branch*. Worse, a
   unit test asserted `js_to_fixed(-0.0625, 3) == -0.062`, pinning the second
   bug: it had been written from a paraphrase of the spec rather than from
   `node`. That assertion is the only expected value this pass changed, and its
   replacement is V8's own output.

**One new divergence found, not yet ported — and it is the largest in the
workspace.** `Math.atan2` disagrees with `f64::atan2` on **22.98 %** of
arguments (200 000 samples against `node`), versus 9.52 % for `exp`, 3.40 % for
`ln` and 2.34 % for `sin`/`cos`. Eight live sites, no `js_atan2` anywhere. The
structural one is `cartalith-hydrology::build_channels`, whose
`0.5 + 0.5·cos(da)` steering factor differs from V8 on **12.97 %** of aspects
and feeds a `score > best_score` argmax that picks the cell a river flows into.
Recommended as the next thing to port, into the urban fork's FDLIBM block
beside the `js_sin`/`js_cos`/`js_log` it is currently adding.

**The helpers disagree with each other, in three measured ways, none live.**
`js_round`: six crates use `(x + 0.5).floor()`, which differs from V8 on
**exactly one** double, `0.49999999999999994` (`cartalith-terrain`'s comment
calling it "the standard exact equivalent" is wrong). `js_hypot`: three of the
four copies lack the spec preamble, so `hypot(∞, 3)` gives `NaN` instead of `∞`
and `hypot(NaN, 0)` gives `0` instead of `NaN`. `js_min`: `-terrain::amplify`
and `-urban`/`-civ` disagree on `min(+0, -0)`. All recorded rather than fixed —
six cross-crate edits for one unreachable input is the wrong trade with three
forks in flight.

**Recommended, deliberately not done: a `cartalith-jsmath` leaf crate.** Seven
copies of `js_hypot`, seven of `js_round`, three of `js_min`/`js_max`, two of
`toFixed`. A dependency-free crate below `-noise`/`-rng` is the only shape that
reaches all fourteen (`-urban` sees only `-rng`; `-assets` only `-io`/`-noise`),
and it does not disturb `ARCHITECTURE.md`'s one-way ordering. Blocked on the
urban fork, which is actively editing the file that would move.

**Reviewed and believed safe, with the invariant, so the next reader does not
re-derive it.** The D8 neighbour tables in `-erosion`/`-gpu`/`-hydrology` are
`hypot(dx,dy)` for `dx,dy ∈ {-1,0,1}` and are **bit-identical** to V8 on all
nine values. `f64::clamp` already propagates NaN exactly as
`Math.max(lo, Math.min(hi, x))` does, which is why divergence #3 has almost no
live surface left — only two hand-written `lo.max(hi.min(x))` sites remain, and
neither is NaN-reachable. Every `0/0` candidate feeding a clamp is guarded at
the *source* (`flow_max`'s `if raw > 0.0`, `slope_max`'s constant `4.0`). And
`build_npp`'s `exp` was measured rather than assumed: over **10 million**
temperature samples, swapping `f64::exp` for `js_exp` changed the stored `f32`
zero times, because an `f32` store is 10⁸ times coarser than the divergence.
The full site-by-site verdict list, including the sites that are *probably*
fine and cannot be proved so, is §4 of the audit.

**Verified.** `cargo test --workspace --exclude cartalith-godot`: **1131 passed
/ 0 failed** across 96 suites, against a **1128 / 0** baseline taken
immediately before — the delta is exactly the three tests added, and no
pre-existing golden moved. `cargo clippy -p cartalith-spatial --all-targets`
clean, zero warnings. Both fixes confirmed to fail before and pass after by
reverting the one-line change and re-running. `cartalith-godot` excluded for
the documented DLL-lock transient. `cargo fmt` not run. Nothing Godot-scene-side
was touched (UI hold, `DCC_SHELL_SCOPE.md`), and `cartalith-urban` and
`cartalith-godot/src/render.rs` were audited and reported on but not edited, so
the two active forks are untouched.

## `js_atan2`, and the river receiver it was picking wrong (2026-08-18)

Acts on `JS_SEMANTICS_AUDIT.md`'s recommendation #1, the one it called the
biggest measured gap in the workspace. **It was a live bug, not a latent one:
`build_channels` was steering rivers into the wrong cell.**

**What V8 actually runs.** `Math.atan2` does not reach the platform libm. V8
ships its own FDLIBM port in `src/base/ieee754.cc` — the same reason `js_exp`
exists — so the target is `__ieee754_atan2` and the `atan` it calls. Verified
by measurement rather than assumed. Two details decide whether a transcription
is right, and the second is not in the source most people would reach for:

- the specification preamble (both signed zeros, each infinity quadrant, NaN);
- **`m &= 1` in the `|y/x| > 2**60` branch** — the FreeBSD msun correction V8
  carries and the original 1993 Sun fdlibm does not. Without it the port
  disagrees with V8 on **777 of 240,000** arguments, all of them `x` tiny and
  negative, returning one ulp above `pi/2` where V8 returns `pi/2`. The first
  transcription *was* the 1993 source; the differential run against `node`
  pointed straight at the branch.

**Measured, to the standard `js_exp` set.** Over 240,000 arguments across four
bands (`[-1,1]`, height gradients `1e-8..1e-1`, coordinate deltas `±4096`,
mixed magnitudes `2^±40`), `f64::atan2` returns a different double from V8 on
**40,824**; `js_atan2` on **0**. Over the 1,089-pair cross product of 33
special values, `f64::atan2` differs on **42** and `js_atan2` on **0**.

**The bug, and why it is structural rather than a coincidence.**
`build_channels`'s receiver is a discrete argmax — the cell a river flows into
— so nothing absorbs a one-ulp difference the way an `f32` store does for the
audit's `exp` sites. A cell whose 3x3 is left-right symmetric has `gx == 0.0`
exactly, so `aspect = atan2(-gy, -0.0)` lands on the signed-zero branch at
exactly `-pi/2`; its two symmetric downhill diagonals then have **exactly
equal** `drop` and mathematically equal `|da|`, and the argmax is settled
purely by which of two last bits is larger. That is a ridge, a saddle or a
plateau edge — ordinary terrain.

The whole decision for a cell depends on exactly its 3x3 block, so the domain
was sampled directly: over **1,200,000** random blocks on a quantised height
lattice, `f64::atan2` picks a **different receiver from V8 on 84**; `js_atan2`
picks V8's on all 1,200,000. Every one of the 43 divergent blocks kept was then
re-run against `node` executing the reference's own channelization loop:
**V8 agreed with `js_atan2` on 43 of 43 and with `f64::atan2` on 0 of 43.**

**`sin`/`cos` deliberately not ported, and that is measured rather than
skipped.** They diverge from V8 too (2.34 % each) and sit in the same
expression, but the wrap `atan2(sin(da), cos(da))` only decides the outcome
when the two competing `da` are exact negatives, and `sin`/`cos` preserve that
antisymmetry exactly whatever their accuracy. Over **600,000** blocks in four
terrain regimes, `js_atan2` with Rust's own `sin`/`cos` agreed with V8 on every
single receiver. Porting them here would have been two hundred lines of
unreachable FDLIBM and a ninth copy site.

**Why no golden could have caught it — measured, not asserted.** All three
cases of `golden_parity_river.rs` pass **unmodified**. Instrumented, they
channelize **365 cells between them, and not one has `gx == 0.0` exactly, nor a
top-two score gap below `1e-15`**: their terrain is smooth and asymmetric, so
the precondition never arises. This is the audit's own recurring finding in its
sharpest form — a fixture that exercises the *feature* thoroughly and the
*branch* not at all, exactly as `golden_parity_geojson.rs` did for
`js_to_fixed`.

**Where it landed, and why not where the audit suggested.** The audit proposed
the urban fork's FDLIBM block. That fork is still live — 607 uncommitted lines
in `geom.rs`, its own `routes` golden red mid-edit — so per the audit's own
rule `js_atan2` went into `cartalith-hydrology` as a private `jsmath` module,
where the live bug was, and the `cartalith-jsmath` consolidation is
**re-recommended rather than performed**. That is honestly an eighth copy site
for the FDLIBM family, and the audit now records the cost: `-terrain:372` and
`-urban::graph:607` both still need `atan2` and neither can reach a private
module in `cartalith-hydrology`.

**The other seven sites, each with a verdict rather than a shrug:**

- **`-terrain:1864, 1865` (`poly_meta`) — safe, proved.** Its points come from
  `walk()`, one 8-connected cell at a time, so the arguments are always in
  `{-1,0,1}²` — and **all eight D8 directions are bit-identical between
  `f64::atan2` and V8** (verified directly). Belt and braces: `curvature`, the
  only thing the turning angle feeds, has no consumer in the workspace outside
  one unit-test assertion.
- **`-civ::labels:517` — safe, for a different reason worth stating.** Its
  input is a live mouse drag, not a seeded pipeline value, so there is no
  reproducible reference to diverge from; the output is continuous and crosses
  no threshold.
- **`-terrain:372` (plate circular mean) — reported, not changed, and this is
  the useful negative result.** It **cannot be fixed by `js_atan2` alone**,
  because the divergence enters upstream: over 2,000 synthetic plates at
  `gw = 512`, Rust's `sin`/`cos` produce a different `(Σ sin, Σ cos)` pair from
  V8's on **92** of them before `atan2` is called. Swapping in `js_atan2` alone
  moves the final `plate.x` from 98/2000 disagreeing to 7/2000 — an improvement
  that leaves the site *differently* wrong, which is worse than leaving it
  alone. Its quantised consumer (`js_round(plate.x)` → cell index) differs
  **0/2000**, but `-terrain:347` feeds the unrounded value into the next Lloyd
  iteration's nearest-plate argmin, which is the same hazard one step removed.
  Fix it in the same pass that lands `js_sin`/`js_cos`, not before.
- **`-urban::graph:607` — fork territory, audited not touched, and the next one
  to fix.** `ang` is the **sort key** for the half-edges around a node, which
  the face traversal walks. A one-ulp reorder of two edges leaving a node in
  nearly the same direction produces a different city block. Same argmax hazard
  in a different costume; recommended to the fork, which has already added
  `js_sin`/`js_cos`/`js_log` to the file where it belongs.

**Two of the three recorded helper disagreements fixed; the third left, with
the reason.**

- **`js_hypot`'s missing spec preamble — fixed in all three copies.** Cheap and
  safe in a way `js_round` is not: additive, one place per crate rather than
  six, and it can only change the result for an infinite or NaN argument, which
  no live site reaches — so no golden could move, and none did. In
  `cartalith-terrain` the guard went into the variadic `js_hypot_n`, which also
  covers `tile_render::js_hypot3` — a seventh entry point §3.2's four-way table
  had not listed, and one a fix applied only to the two-argument forms would
  have missed. Each crate gained a spec test with `node`-derived expectations.
- **`js_round`'s `(x + 0.5).floor()` — comment fixed, six implementations
  left.** `cartalith-terrain`'s claim that it is "the standard exact
  equivalent" is false and is now corrected in place, naming the one disagreeing
  input (`0.49999999999999994`) and pointing at the urban fractional-part form.
  Leaving a wrong claim in place is how the next reader "fixes" the correct copy
  to match the incorrect one. The implementations stay: one unreachable input
  against six cross-crate edits is still the wrong trade under an active fork.
- **`js_min`/`js_max` on signed zero — left.** Unobservable (no signed zero's
  sign is ever read), already documented in the urban copy.

**One expectation was wrong when first written, and the test caught it.** An
extra hand-added assertion claimed `atan2(-0.25, -0.0) == -pi`, reasoning from
the signed-zero rules; V8 gives `-pi/2`, because `x` being a zero of either
sign makes the answer `±pi/2` with the sign taken from `y` alone. Written from
reasoning instead of from `node` — the exact failure the audit's recommendation
#5 exists to prevent, caught because that recommendation had also produced the
habit of checking.

**Verified.** `cargo test --workspace --exclude cartalith-godot --exclude
cartalith-urban`: **1069 passed / 0 failed** across 94 suites, against a
**1062 / 0** baseline taken by reverting exactly the five files this pass
touched and re-running — the delta is **exactly the seven tests added**, and no
pre-existing golden moved. `cartalith-urban` is excluded from that pair because
the sibling fork was editing it during the run: its own uncommitted goldens went
from 1 red to 6 red between the baseline and the after-run, none in a crate this
pass touched (including it gives 1130 → 1132 against that moving failure count).
`cartalith-godot` excluded for the documented DLL-lock transient. `cargo clippy
-p cartalith-hydrology -p cartalith-assets -p cartalith-civ -p cartalith-terrain
--all-targets`: no warning or error in any line this pass wrote. The fix was
confirmed to fail before and pass after by reverting the three call sites —
receiver `8` before, `6` after, `6` from `node`. `cargo fmt` not run. Nothing
Godot-scene-side touched (UI hold, `DCC_SHELL_SCOPE.md`); `cartalith-urban` read
and reported on, never edited.

## Phase 5 milestone 6 — anchors and primary routes, and three more V8 libms that are not Rust's (2026-08-18)

`URBAN_MORPHOLOGY_SCOPE.md` milestone 6: `placeAnchors`, `buildPrimaries`,
`buildPrimariesFromPaths` — reference lines **28743-28833** — as
`cartalith-urban::routes`. Dependencies unchanged (`cartalith-rng` only). Wired
to nothing. 10 new tests, 69 in the crate.

This is the first milestone that produces a **real street graph end to end**:
`placeAnchors` picks the one point the whole town is organised around, and the
two `build_primaries*` functions lay the arterial backbone that milestone 7's
growth, milestone 10's enceinte and milestone 12's blocks all accrete onto. Its
golden is therefore a whole-subsystem artefact — market, provenance, every route
polyline, the entire resulting graph and a hash of the spatial index — rather
than a function's return value.

**The stated range was wrong again, five for five.** The plan said 28744-28843:
`buildPrimariesFromPaths` ends at **28833**, 28834 is blank, and 28835-28843 is
the *radial streets* header comment belonging to milestone 8; and 28743 is the
`/* ---------------- anchors ---------------- */` section header, which by the
convention milestones 4 and 5 settled belongs to the milestone it introduces.
Milestones 7-16 remain unverified, and milestone 8's stated start should be
28835, not 28844.

### `Math.sin`, `Math.cos` and `Math.log` are the third, fourth and fifth divergences

Milestone 1 found `Math.hypot` and milestone 5 found `Math.exp` — both **after**
a golden failed. This milestone measured *first*, before writing a line of
`placeAnchors`, which is why all 35 of its scenarios matched on the first run.

| over 80,214 arguments spanning every reachable reduction branch | disagreements with V8 |
|---|---|
| `f64::sin` (the platform libm) | **1,942** |
| `f64::cos` | **2,160** |
| `geom::js_sin` / `js_cos` | **0** / **0** |

| over 60,009 arguments across the whole normal range | |
|---|---|
| `f64::ln` | **1,647** |
| `geom::js_log` | **0** |

`Math.sin` and `Math.cos` are the **third and fourth most-used** functions in
block 4 — 27 and 26 call sites, behind only `Math.min`/`Math.max` — and
`placeAnchors` calls both on every one of its 400 candidate points. V8 calls
`base::ieee754::sin`/`cos`/`log`, FDLIBM's `__ieee754_*`: argument reduction mod
π/2 through `__ieee754_rem_pio2`, then one of two degree-6 kernel polynomials by
quadrant. Transliterated into `geom` beside `js_hypot` and `js_exp`.

**This retro-fixes milestone 1 a second time.** `rng::norm` is
`Math.sqrt(-2*Math.log(u1)) * Math.cos(2*PI*u2)` and had been on `f64::ln` and
`f64::cos` with a documented "they happen to agree" note. They do not agree in
general; milestone 1's goldens landed on values they agree about, the same luck
milestone 5 found in `logn`. `norm` is the highest-leverage function in the
subsystem — `logn` sits on top of it and draws every frontage width, plot depth
and building dimension in the town. The milestone-1 goldens pass unchanged
afterwards, which is the check. `Math.sqrt` needs no treatment: IEEE-754
mandates a correctly-rounded square root, so V8's and Rust's agree by
specification.

**One branch is deliberately not ported, and says so.** Above `2^19 * π/2`
(≈8.2e5) FDLIBM switches to Payne-Hanek reduction — `__kernel_rem_pio2`, a
hundred-odd lines of multi-precision integer arithmetic over a 66-word table of
2/π. Every trig argument in this subsystem is an angle: `range(-PI, 0)`,
`i/n * 2PI`, an `atan2` result, a bearing. None can leave `[-4π, 4π]`, so that
branch would be dead code with a real chance of being silently wrong.
`js_sin`/`js_cos` hand off to the platform libm above the threshold, a test
asserts they do, and the doc comment names it as the one input class not
reproduced.

**The rest of the libm bill, measured now so later milestones do not each
rediscover it.** `Math.atan2` disagrees with `f64::atan2` on **10,615 of 60,000**
arguments — 17.7%, the worst yet, with 7 call sites starting at milestone 8.
`Math.log10` disagrees on 960/60,000 (milestone 15), `Math.acos` on 544/60,000
(milestone 10). `Math.pow(x, 2)` was measured **bit-identical** to `x * x` on
60,000 arguments, so `buildPrimaries`' single `Math.pow` needs nothing at all.

### What the milestone actually does

`placeAnchors` sites the market by rejection sampling: 400 seeded candidates on
the landward half-circle around the break-of-bulk point (the bridge, else the
quay, else a literal fraction of the box), each scored against slope, distance
from that point and distance out of the flood band. `buildPrimaries`
**synthesises** the backbone — an 8 m cost raster whose cost is
`1 + (slope·3.2)²` plus water and bank terms, then `astar` from each external
route endpoint to the market, multiplying already-used cells by `0.45` so later
routes braid onto earlier ones. `buildPrimariesFromPaths` **injects** it
instead: the host hands over the real inter-settlement roads as metre offsets
from the settlement, and the town is grown around those; `generate()` prefers
this whenever `opts.primaryPaths` is non-empty.

### Findings

1. **Neither route builder draws a random number.** Both take a `seed` and
   neither reads it — verified by grep over both bodies and asserted from the
   other side by a test that runs each with a wildly different seed and requires
   a byte-identical graph. `placeAnchors` is the only RNG consumer and draws
   exactly **800** times, two per candidate, **before** any rejection test, so
   the sequence is independent of the site's shape.
2. **Both return values are dead.** `generate()` calls whichever builder applies
   for its effect on `g` and discards the routes (lines 31021-31022). Returned
   anyway, because they are what the reference returns and they make a far
   stricter golden than the graph alone.
3. **The two builders disagree about their own return shape.**
   `buildPrimaries` pushes `{pts, i}`; `buildPrimariesFromPaths` pushes `{pts}`
   with no `i`. Carried as `Route { pts, i: Option<usize> }` rather than erased.
4. **`riverthrough` shares `river`'s candidate band but not its preferred
   distance.** `dBand` tests both kinds and widens to `[60, 240]`; the score's
   `Math.abs(d - (kind === 'river' ? 120 : 100))` tests `'river'` alone. Two
   fixtures share seed 7 for exactly this; a set with one of the two kinds
   cannot see it.
5. **The market reference's third `||` arm is live**, as milestone 5 predicted:
   a landlocked site has no bridge *and* no quay, so the town centres on the
   literal `{Wm*0.52, Hm*0.42}`.
6. **`best === null` is reachable, and only on a small box** — and it is the one
   place in the subsystem that can put the market **outside the site box** (at
   150 × 150 it lands at y = −57, with no clamp anywhere).
7. **The 80 m margin is unobservable on the engine's own box.** At 1700 × 1250
   the reference point sits at (884, 525) and candidates reach 240 m, so no
   candidate is ever within 400 m of the margin. It takes a ~520 m box to make
   the constant do anything, which is what `midBox`/`midBoxRiver` are.
8. **`Math.max(0, rd − 260)` is dead on every site this engine can build.** The
   candidate is drawn at most 240 m from a reference point that lies *on* the
   water on every watered site, so `rd` cannot exceed the draw; and a landlocked
   site's dummy river at `(−1e4, −1e4)` makes the term a ~81-unit constant that
   shifts every score equally. A test asserts that invariant across all 35
   fixtures rather than asserting the dead branch.
9. **`buildPrimariesFromPaths`' final `sm.length < 2` guard cannot fire**, and
   its `path.length < 2` guard is redundant with the next one — but its
   `pts.length < 2` guard is **not**: a path whose second point is outside the
   box leaves exactly the market in `pts`, which without the guard survives as a
   degenerate two-identical-point street, adding a **node** and no edge.
10. **A metre offset added to a metre coordinate cannot express a one-ulp
    boundary.** Both boundary fixtures needed rebuilding for this:
    `(386.6 + 1.0000000000000002) − 386.6` is exactly `1.0`. `> 1` is straddled
    with 1 m and 1.25 m; the 6 m box tolerance with −5, −6 and −7. **Any
    boundary fixture built by offsetting a large coordinate has to clear that
    coordinate's own ulp, not the constant's** — which generalises milestone 4's
    just-below-a-boundary rule, and which milestone 17's adapter will hit
    because it produces exactly these offsets.
11. **`toCell`'s clamp absorbs the `Math.round` question.** JS rounds halves
    toward `+∞` and `f64::round` away from zero; a negative cell index clamps to
    `1` regardless. `geom::js_round` is written correctly anyway — and `rules`'
    private copy now routes through it, provably identical on its own `[1, 4]`
    domain — because the next caller may not clamp.
12. **The reinforcement's `Set` iteration order cannot matter** (disjoint
    indices, one multiply each), but the **route order** very much does: a test
    reverses `site.routeEnds` and requires the town to change, so the `0.45` can
    never be quietly neutralised.

### Golden verification

Same slice harness as milestones 3-5, verbatim: contiguous 28167-31103 plus line
2291, the balance scan with milestone 2's orphan-close counter, and the four
structural assertions including milestone 3's tightened first-line form and the
`mulberry32` negative control. None of the three functions is on `UME`'s public
export or its `_test` one, so the capture adds them — with `buildSite` and
`makeGraph`, which the fixtures need — by milestone 5's single anchored
replacement of the `return {` line, asserted to match exactly once, with the
explicit `globalThis.__UME` handoff and its assertion. The frozen reference file
is never touched.

38 scenarios, compared **bit for bit** with no tolerances anywhere. The spatial
index is pinned by the reference's **own** `fnv1a` over its own canonical grid
dump rather than cell by cell — milestone 2 golden-tested the index itself, and
restating 400-odd cells per scenario would have added 40,000 lines of golden for
no extra strength.

The capture's emptiness / shape gate refuses to write unless twenty-odd
conditions hold, each naming the fixture it protects: the 80 m margin rejects
>20 and admits >20 candidates on the mid-box fixtures and rejects **zero** on
the full-size one; `lastCandidateWins` really wins on candidate 399;
`shortDtWater` really admits >100 candidates and then scores every one `NaN`;
`tinyBox` really takes the `best === null` fallback and `landlocked3` really does
not; `bay` and `coast` really diverge on one seed while `atoll` and `coast`
really coincide; `nanCost` really produced no routes; `_fromPaths` agrees with
the route count everywhere and is false somewhere; the 1 m unshift boundary is
straddled both ways; the box-edge triple keeps 3 of its 4 points; and
`bendPath`'s Chaikin corners really separate `simplify(1.2)` from
`simplify(1.3)`. The Rust side mirrors the shape half as its own test, because
`zip` stops at the shorter side and a truncated `golden.rs` would otherwise pass
vacuously.

**Every golden matched on the first run** — all 38 scenarios, every one of the
four rounds of fixture work included.

### Mutation testing

**Five sweeps: 300 mutations / 98 survivors, then 300 / 79, then 306 / 73, then
306 / 74, and finally 306 mutations, 233 died, 73 survived.** Every survivor was
re-run in isolation and **not one false survivor appeared in any round** —
milestone 4's stale-binary problem, solved by giving the sweep its own
`CARGO_TARGET_DIR` instead of sharing one with the other forks.

**Six of the 306 are deliberate graded perturbations** — milestone 4's device for
a constant whose small change is absorbed — and **all six die**: the sea cost
`240 → 5`, both second-simplify tolerances `1.2 → 4.0`, `toCell`'s lower clamp
`1 → 3`, the margin `80 → 200` on all four sides at once, and the flood-band
penalty `260 → 20000`. Each says *this constant is tested; a 37% nudge is simply
below what the fixture can express*.

**One thing the round-4 sweep taught that no earlier milestone had hit: fixture
coverage is not monotonic when you *replace* a fixture rather than add one.**
Round 4 swapped a trig band whose reduced remainder was ~1e-9 for one whose
remainder is ~1e-13, gaining the third correction round and *losing* the kernels'
own `|x| < 2^-27` shortcut — two mutants a previous round had killed came back.
The survivor count went 73 → 74 on a round that was strictly meant to improve
things. Both bands are now present, which is the final 73.

##### The 19 survivors in `routes.rs`

| class | n | why it survives |
|---|---|---|
| the 80 m margin, three of its four sides | 3 | `marginWinner` is a scanned site whose *winning* candidate sits 80-110 m from **one** edge, so only that side's constant is observable; the other three would each need their own scanned site. The graded `80 → 200`, which moves all four at once, **dies** |
| the flood-band penalty's `0` and `260` | 2 | proven dead rather than argued: a candidate is drawn at most 240 m from a reference point that lies *on* the water, so `rd − 260` is never positive on a watered site; and a landlocked site's dummy river at `(−1e4, −1e4)` makes the term a ~81-unit constant that shifts every score equally. A test asserts that invariant across all 38 fixtures, and the graded `260 → 20000` **dies** |
| the `240` sea cost | 1 | a **barrier, not a cost**: any value large enough to make a water cell non-optimal produces the same path, so `240 → 328.93` cannot move one. The graded `240 → 5` **dies** |
| `toCell`'s two `1.0` lower clamps | 2 | the clamp's result is immediately `as usize`, so a change smaller than one whole cell truncates away — milestone 4's quantised-output pattern, third appearance. The graded `1 → 3` **dies** |
| five comparators that need an exact tie | 5 | the margin's `<` → `<=`, the flood band's `<` → `<=`, the score's `>` → `>=`, the bridge window's `<` → `<=`, the bank band's `<` → `<=`. Every one of those inputs is a continuous distance or score; **milestone 3's finding recurring**, and unlike there it cannot be closed by a quantised raster, because these are polyline distances and sums of RNG draws |
| `bs = −∞ → −1e308` | 1 | no reachable score is below `−1e308`; the initial value's only job is to lose to the first accepted candidate |
| `toCell`'s clamp **order** | 1 | `max(1, min(W−2, ·))` and `min(W−2, max(1, ·))` differ only when `W < 3`, i.e. a site box under 24 m |
| `js_round → f64::round` | 1 | they differ only on negative halves, and a negative cell index clamps to `1` either way. `js_round` is written correctly anyway, because the next caller may not clamp |
| `fromPaths`' `path.len() < 2` → `is_empty()` | 1 | a one-point path yields a one-point in-box run, which the *next* guard drops. That next guard is **not** redundant, and `pathsOnlyMarket` is the fixture that shows it |
| `rem_pio2`'s two round triggers, `16 → 17` and `49 → 50` | 2 | see below — both rounds are load-bearing and tested; what no fixture produces is an argument whose exponent gap is *exactly* 17 or *exactly* 50 |

##### The 54 survivors in the FDLIBM block

| class | n | why they survive |
|---|---|---|
| dead in **this port's** call path | 11 | `js_sin`/`js_cos` filter `|x| ≤ π/4` and Inf/NaN *before* calling `rem_pio2`, so its own early return and its own Inf/NaN branch are unreachable through the public API (8 mutants); `HUGE_ARG_HI` only decides where the platform hand-off starts (1); and the `ix == 0x3ff921fb` sub-branch needs `|x|` inside a 2.3e-8-wide window at π/2 (2) |
| `iy` is a flag, not a value | 5 | `kernel_sin`'s third argument is only ever tested `== 0`, so `1 → 2` is the same call at all four sites; and on the `|x| ≤ π/4` short path `y` is unused, so `0.0 → 1.0` is too |
| ±1-ulp threshold constants | 18 | every `0x…` comparison bound — the four `0x7fff_ffff` absolute-value masks, `0x3e40_0000`, `0x3fd3_3333`, `0x3fe9_0000`, `0x3fe9_21fb`, `0x4002_d97c`, `0x4139_21fb`, `0x7ff0_0000`, `0x0010_0000`, `0x6147a`, `0x6b851`. One ulp of a **high word** only changes behaviour for an argument sitting in that one-ulp window; 54,000 uniform draws never land in one |
| provably equivalent arithmetic | 13 | `0x95f64` is **even**, so the bit its mask can add to `i0` is one `hx` already carries and the `\|` is a no-op — checked by hand after the runner flagged it, because it looks like it should be catastrophic; `qx`'s `0x0020_0000` cancels in `a − (hz − …)`, which is exactly why FDLIBM may pick `0.28125` arbitrarily; `(x as i32) == 0 → == 1` never takes the tiny-x shortcut and the polynomial returns `x` (or `1.0`) anyway; `hx > 0 → > 1` and `hx < 0 → < 1` sit where `hx ∈ {0, 1}` is unreachable; and `js_log`'s five branch selectors pick between two algebraically identical final formulas |
| the staged reduction refines the **tail**, not the returned double | 7 | the four `y[0] → y[1]` index mutations in the medium branch, both `0x7ff` exponent masks, and one more trigger form. **Evidence, not assertion**: never running the second round (`i > 100000`) **dies**, always running the third (`i > −1`) **dies**, and always running the second (`i > −1`) **survives**. So both rounds are load-bearing and both are tested; FDLIBM's first round is already "good to 85 bit" against a 53-bit result, so running one round more than needed is free and running one fewer is not |

### Two tooling incidents worth carrying forward

**A dozen hand-picked rows cannot test a bit-twiddling port.** The first sweep
left **63 survivors inside `js_sin`/`js_cos`/`js_log` alone** — every reduction
threshold, every `y[0]`/`y[1]` slot, both correction-round triggers and the
whole `kernel_cos` `qx` split untested, by a golden table built exactly the way
`js_exp`'s and `js_hypot`'s were. Twelve rows cover twelve paths through a
branchy function, not its branches. The fix is four lines of golden: an FNV-1a
**hash** over every result for 54,000 sin arguments, 54,000 cos and 30,000 log,
with the arguments drawn by the reference's own `mulberry32` so both sides
provably evaluate the same points, and the bands chosen to enter each reduction
branch on purpose. It matched V8 on the first run and killed essentially all 63.
**Any later milestone that ports a libm function should start there** — and
milestones 8, 10 and 15 each need one.

**Two mutation runners on one target directory left a live mutation in the
source.** Round 2 was started twice by accident; the first was killed
mid-mutation, the second read the already-mutated file as its "original" and
faithfully restored it to that, and `routes.rs` carried `-(s * 5.61)` where the
reference has `-(s * 4)`. Nothing but the suite failing afterwards said so — a
per-edit `finally` restore is not enough, because it restores to whatever it
read. The runner now takes a **pristine snapshot before it writes anything**,
restores from that snapshot at the end, re-runs the suite as a post-sweep
baseline, and refuses to start while a lock file exists. Milestone 4's
stale-binary incident produced the "re-run every survivor in isolation" rule;
this is its sibling and the more dangerous of the two, because it corrupts the
**source** rather than the **report**.

(The private `CARGO_TARGET_DIR` this runner uses did solve the original problem:
**zero false survivors** across 600 mutations, where milestone 4's
shared-directory run reported 34.)

### Corrections written forward

1. Milestone 6's own range was **28743-28833**; milestone 8's should start at
   **28835**. Five ranges checked, five wrong — verify the rest before slicing.
2. Every milestone from here must use `geom::js_sin`/`js_cos` for
   `Math.sin`/`Math.cos`, exactly as it must use `js_exp`, `js_hypot` and
   `js_min`/`js_max`. Milestone 8's `buildRadialStreets` is trig-saturated.
3. **Milestone 8 needs a `js_atan2` and cannot borrow the one that now
   exists.** A sibling fork landed `cartalith-hydrology::jsmath::js_atan2` the
   same day; `cartalith-urban` depends on `cartalith-rng` only and must keep
   doing so, so milestone 8 either copies it into `geom` or the
   `cartalith-jsmath` leaf crate `JS_SEMANTICS_AUDIT.md` recommends finally gets
   built. Milestone 10 needs `js_acos` (0.9%) and milestone 15 `js_log10`
   (1.6%) on the same terms. Port all of them against a bulk hash golden, not a
   dozen rows.
4. `Graph::from_paths` exists now, and milestone 10's `builtMassHull` must read
   it or the enceinte over-encloses along arterials.
5. Milestone 16 inherits only the graph and `placeAnchors`' 800-draw substream;
   neither route builder touches the RNG and both return values are discarded.
6. The market can land **outside the site box** when every candidate is
   rejected. Milestones 7 and 10 measure everything from `anchors.market`.
7. **`extractFaces` still sorts half-edges with `f64::atan2`.** Milestone 2 wrote
   it before anyone had measured `Math.atan2`, and it is now the largest known
   divergence in the workspace. Not changed here — milestone 6's scope is the
   three route functions, and the `js_atan2` that landed the same day lives in
   `cartalith-hydrology`, which this crate must not depend on — but
   whoever lands `js_atan2` should sweep that call site, re-run milestone 2's 19
   graph scenarios and report the result. The "it only affects order" argument
   is the same one that was made for `hypot` before milestone 2 proved `hypot`
   changes graph *topology*.
8. **Fixture coverage is not monotonic when a fixture is *replaced* rather than
   added** — round 4's survivor count went *up*, 73 → 74, on a round meant to
   improve things, because swapping one trig band for a better one gave up the
   branch the old one reached. Add; do not substitute.

## `cartalith-jsmath` — one implementation each, and the last two `atan2` hazards closed (2026-08-18)

`JS_SEMANTICS_AUDIT.md` recommendation #2, carried out. The audit catalogued
eight distinct operations where Rust's standard library and V8 disagree about
what a floating-point expression means, plus the rounding-mode family — and by
the time it ran, the JS-faithful replacements had been written **independently
in five crates**: seven copies of `js_hypot`, seven of `js_round`, three of
`js_min`/`js_max`, two of `toFixed`, and one each of `js_exp`/`js_sin`/`js_cos`/
`js_log`/`js_atan2` that nothing outside their own crate could reach.

The recommendation was blocked only because `cartalith-urban` was mid-edit. That
fork landed (`6d242cf`), and the case had got worse in the meantime: milestone 6
established that `cartalith-urban` must keep `cartalith-rng` as its **only**
dependency, so it could not use the `js_atan2` that had landed in
`cartalith-hydrology` — and its milestone 8 needs `atan2`, which would have been
a **ninth** FDLIBM copy site.

### The crate

`crates/cartalith-jsmath`, two files, **no dependencies at all** — not on
another Cartalith crate, not on a third-party crate, and **not a dev-dependency
either**. `ARCHITECTURE.md` has dependencies running one way in pipeline order,
and a crate with none cannot create a cycle wherever it is added; that is the
only shape that reaches all fifteen, since `cartalith-urban` is allowed only
`-rng` and `cartalith-assets` only `-io`/`-noise`. The bulk goldens carry a
four-line inline `mulberry32` rather than borrowing `cartalith-rng`'s, precisely
so the leaf property is a fact about the manifest rather than a convention.

- `libm.rs` — the FDLIBM family V8 ships in `src/base/ieee754.cc` rather than
  taking from the platform: `js_exp`, `js_sin`, `js_cos`, `js_log`, `js_atan2`,
  and `js_atan`, which was private inside `cartalith-hydrology` and is public
  now because `Math.atan` is a JS function in its own right.
- `lib.rs` — `js_hypot`/`js_hypot3`/`js_hypot_n`, `js_min`/`js_max`,
  `js_round`, `js_num_or_zero`/`js_truthy_num`, `js_fixed`/`js_to_fixed`,
  `u8_clamped`.

**No call site had to change.** Where a module path was load-bearing —
`geom::js_hypot`, `sculpt::js_hypot`, `tile_render::u8_clamped`,
`spatial::geo::js_to_fixed` — it survives as a `pub use` re-export.

**What deliberately stayed put.** `cartalith-spatial::paint`'s exhaustive
rim-cell scans, `cartalith-assets`' `js_round_is_half_up_which_matters_at_the_left_edge`,
and `cartalith-civ`'s NaN-absorption tests: they test a *call site's* behaviour,
not the helper's. `-assets::scatter::js_number`, `-assets::manifest::js_parse_float`,
`-io::tiles::js_num` and `-urban::site::js_or` also stayed — they are JS
*coercions* over crate-specific types (`serde_json::Value`, `Option<f64>`,
strings), not floating-point semantics.

**`js_acos`/`js_log10` deliberately not added.** `cartalith-urban` milestones 10
and 15 will need them; adding them now would be two hundred lines of FDLIBM with
no caller and no golden.

### The three copy disagreements, resolved rather than recorded

1. **`js_round`.** Six crates used `(x + 0.5).floor()`, which differs from V8 on
   exactly one double, `0.49999999999999994`. Consolidated onto the
   fractional-part form, which is V8's answer there and everywhere.
   `cartalith-terrain`'s "the standard exact equivalent" comment went with the
   code it was wrong about. No golden moved.
2. **`js_hypot`.** The `js_atan2` fork had already restored the specification
   preamble to all three copies that had lost it, but through **five distinct
   compensated sums**. There is one now, `js_hypot_n`, with `js_hypot` and
   `js_hypot3` as wrappers, so the preamble cannot be lost from one entry point
   and kept in another. Its one behavioural difference: it takes signed
   arguments and `abs`es them itself, where `cartalith-terrain`'s form required
   pre-`abs`ed magnitudes — and `abs` of a magnitude is the identity, so nothing
   moved. Five identical spec tests collapsed into one.
3. **`js_min`/`js_max` on signed zero.** `Math.min(+0, -0)` is `-0` and
   `Math.max(+0, -0)` is `+0` in **either** argument order, because ECMA-262
   treats `-0` as strictly smaller than `+0` for these two functions alone. A
   plain `<` cannot see that, so which order a copy got right depended purely on
   how it was spelled: `-urban`/`-civ`'s `if b < a` and `-terrain`'s `if a < b`
   were each wrong in the opposite direction. **All three were wrong.** The one
   implementation has a four-line both-zeros arm and is V8's in both orders,
   with eight expectations read off `node` using `Object.is(x, -0)` and both
   superseded forms asserted failing.

### Two live hazards, fixed and proved

**`cartalith-urban::graph:607` — `extract_faces`' half-edge sort key.** `ang`
was `f64::atan2`, written at milestone 2 before the divergence was measured. It
is the sort key the face traversal walks, so one ulp reorders two edges leaving
a node and the traversal produces a different city block. Measured on the
coordinates the graph really produces — arbitrary `f64`, from `attach_point`'s
split points — `f64::atan2` returns a different double from V8 on **196,034 of
510,634** near-parallel edge deltas (38 %, higher than any figure in the audit
because these are small differences of large coordinates), and the two put the
pair in a **different order on 23,814 of them, 4.7 %**. Three quarters of those
are cases where Rust manufactures a difference out of what V8 computes as an
exact tie.

All **20** of milestone 2's golden scenarios — which compare the entire graph
state plus every extracted face against the reference's own `UME._test` output —
pass **unmodified** after the fix. That is the proof the change moved nothing
they can see, and the reason is measurable rather than lucky: they are built
from round coordinates whose incidences at a node are milliradians apart. What
they cannot see is pinned by a new test carrying five real delta pairs with the
bit patterns `node` v24.19.0 returns and V8's own comparison of each: **V8 agrees
with `js_atan2` on 5 of 5 and with `f64::atan2` on 0 of 5.**

**`cartalith-terrain:372` — `buildPlates`' world-wrap circular mean.** The audit
reported this and deliberately did *not* change it, because the divergence
enters **upstream** of `atan2` and `js_atan2` alone would leave the site
*differently* wrong. Its instruction was to fix all three together in the pass
that lands `js_sin`/`js_cos`. This is that pass, and the site became fixable.

Measured over 2,000 synthetic plates at `gw = 512`, arguments drawn by the
reference's own `mulberry32`:

| | disagrees with V8 |
|---|---|
| the `(Σ sin, Σ cos)` pair, before `atan2` is reached | **737 / 2000** |
| final `plate.x`, `f64::sin`/`cos`/`atan2` | **193 / 2000** |
| final `plate.x`, `js_atan2` only — the partial fix | **110 / 2000** |
| final `plate.x`, `js_sin` + `js_cos` + `js_atan2` | **0 / 2000** |

The third row is the audit's "differently wrong" claim turned into a measurement
in this repository: the partial fix is a real improvement that still leaves the
site disagreeing with V8 on one plate in eighteen. It matters because
`-terrain:347`'s `dx = x as f64 - plate.x` feeds the next Lloyd iteration's
nearest-plate argmin — the same discrete-decision hazard as the river receiver,
one iteration removed.

Both `world`-mode cases of `golden_parity_plates.rs` pass **unmodified**; their
grids are 6×5 and 7×6, so the circular mean is over a handful of cells and lands
on the same double either way. The *feature* golden passing while it cannot see
the *branch*, again — which is why the new test is a bulk FNV-1a hash of all
2,000 `plate.x` values per seed against `node`, with assertions that Rust's own
libm and the partial fix both produce a *different* hash so the rows cannot
quietly stop discriminating.

### Verification

- `cargo test --workspace --exclude cartalith-godot`: **1134 passed / 0 failed**
  across 99 suites, against an immediately-preceding baseline of **1138 / 0**
  across 96, taken before anything was touched. The delta of **-4** is fully
  accounted for: 8 tests **moved** out of `-urban`/`-hydrology` into
  `cartalith-jsmath`; 15 **duplicates** deleted (five crates had grown identical
  copies of the same three helper tests); `cartalith-jsmath` gained those 8 plus
  **8 new**; **3 new** landed at the two fixed call sites.
  `1138 − 8 − 15 + 16 + 3 = 1134`.
- **No existing golden expectation was modified anywhere.** Not one — including
  all 20 urban graph scenarios and both `world` cases of the plates golden.
- The moved tests passed on their **first run** in the new home, including the
  bulk FNV-1a goldens over 54,000 `sin`, 54,000 `cos` and 30,000 `log` results
  and the two `node`-derived `js_atan2` goldens. That is the check that the move
  was pure.
- `cargo clippy --all-targets` over all nine touched crates: **no warning in any
  line this pass wrote**; `cartalith-jsmath` is warning-free outright. What
  remains is what the audit's §6.1 already named — `excessive_precision` in
  `golden_parity_road_network.rs`, two loop-index warnings and a `matches!`
  suggestion in `cartalith-civ`, `cartalith-gpu`'s two dead-code warnings.
- **Mutation-tested: 440 mutants, 258 killed, 182 survived, 0 broken**, over the
  whole non-test body of both crate files, each in its own `cargo test` with a
  **private `CARGO_TARGET_DIR`** — milestone 6 found two runners sharing one
  target directory leaving a live mutation in the source. Snapshot before
  writing, restore after, a post-sweep byte-comparison of both files, and a
  post-sweep baseline: **GREEN**. Twenty survivors re-run in isolation:
  **20 of 20 reproduce, zero false survivors.** The survivor classes, each with
  the invariant it rests on:

  | count | class | why no argument can express it |
  |---|---|---|
  | 56 | a FDLIBM constant moved by **one ulp** | the perturbation reaches the result scaled by `z^k` and is 10⁻⁵–10⁻³ of the result's own ulp |
  | 55 | a comparison flipped on a boundary | the two forms differ only when the operands are exactly equal, and both arms then return the same double |
  | 36 | a reduction threshold bumped one representable step | fires only for the single argument whose high word *is* the threshold |
  | 24 | a guard dropped | Rust's saturating float→int casts and always-false NaN comparisons make it redundant (`u8_clamped`'s `return 0`/`return 255`, `js_round`'s non-finite arm) |
  | 11 | a slot or shift inside `rem_pio2`'s third correction round or the Payne-Hanek hand-off | branches the audit already documents as unreachable from any angle this engine produces |

  **The sweep paid for itself twice.** The first pass left **206** alive, and
  **101 of those were inside `js_exp` and `js_atan`/`js_atan2`** — the two
  functions that predated the hash technique and still had only a dozen
  hand-picked rows each. Adding the two bulk FNV-1a goldens they were missing
  (48,000 `exp` arguments over eight bands, 54,000 `atan2` arguments over nine
  ratio bands and all four sign quadrants, both matching V8 on the **first
  run**) took it to 182. The sweep also found **four real gaps** in this pass's
  own new code: both `toFixed` ports' non-finite guards were untested — dropping
  either leaves a `.expect()` that panics on an infinity — and every value in
  the `u8_clamped` table had an **even floor**, so inverting its round-down
  comparison fell through to the ties-to-even arm and produced the same answer.
  Writing the non-finite test then exposed a **real divergence**: `js_fixed`
  returned Rust's `inf`/`-inf` where JS spells them `Infinity`/`-Infinity`.
  Fixed, from `node`.
- `cargo fmt` was not run — several crates are not rustfmt-clean and it would
  create noise across the workspace. Nothing Godot-scene-side was touched (UI
  hold, owner, 2026-08-18).

### Two findings about the sweep itself, worth more than the numbers

**A mutation operator can manufacture its own survivors.** The first round
reported dozens, and they were an artefact twice over: it was mutating inside
`//` comments (`// atan(1.5)hi` → `// atan(1.6)hi` is not a mutation), and it
was bumping a float constant's **last written decimal digit** — but FDLIBM
writes its constants to 21 significant figures, three past what an `f64` can
hold, so the "mutant" parsed to the same double. Both fixed: mutate only the
code half of a line, and perturb a float by **one ulp** via its bit pattern.
The lesson generalises past this crate — a survivor is only evidence if the
mutant is genuinely a different program.

**The consolidation is exactly the shape of edit this project's own history says
to distrust**, so it was done as a pure move: the implementations were extracted
verbatim rather than re-derived, the tests moved with them unchanged, and the
only bodies actually rewritten were `js_hypot_n`'s signature and `js_min`/
`js_max`'s signed-zero arm — the two places where the copies disagreed and a
choice had to be made. Everything else is the same text in a different file.

## Phase 5 milestone 7 — organic growth, the function the whole subsystem is an accretion onto (2026-08-18)

`URBAN_MORPHOLOGY_SCOPE.md` milestone 7: `logisticRamp`,
`estimateCarryingCapacity`, `wallOccupancy`, `grow`, `supersedeWall` —
reference lines **29384-29630** — as `cartalith-urban::growth`. Dependencies
unchanged (`cartalith-jsmath`, `cartalith-rng`). Wired to nothing. 15 new
tests, 84 in the crate, 60 golden scenarios.

`grow` is the heart: an epoch loop that spends a population-derived
street-length budget on seeded candidate segments, branching off existing
frontages at near-perpendicular angles, with a decaying exploration share, a
market-distance density gradient, junction-angle and parallel-spacing
rejection, bridgehead rules for the far bank, and — behind an opt-in flag —
successive wall generations gated on real elapsed years. The scope document
predicted this would be the hardest milestone to land and that its golden would
have to be a **per-epoch** graph hash so a divergence localises to an epoch.
Both were right, and the per-epoch trace is what the harness is built around.

**Every golden matched on the first run** — the first 48, and the 12 the
mutation sweep's second round added later. Total street length,
the per-epoch trace, every node, every edge, every provenance string, the
spatial index, every `buildWall` call and every supersession record.

**The stated range understated the milestone by six lines at the start, and its
end was right — the first of the six whose end was.** `logisticRamp`'s body
starts at 29390, but 29384-29389 is its own doc comment (the one flagging
`k = 6.5` as tuned rather than measured), which by milestones 4/5/6's
convention belongs to the milestone it introduces. 29630 is exactly
`supersedeWall`'s closing brace. **Six ranges checked, six adjusted.**

### `buildWall` is milestone 10's, so the capture stubs it — on both sides

`grow` and `supersedeWall` both call `buildWall` (line 29748, 190 lines,
milestone 10). Rather than defer every wall branch for three milestones, it
arrives here as a `WallBuilder` trait object, and the golden capture **stubs the
reference's own `buildWall`** with a single anchored insertion into the sliced
text — asserted to match exactly once, with the frozen file never written to.
Both sides then run the same no-op recorder, and the fire epoch, the M-GRW-2b
age gate, the M-GRW-2a occupancy gate, the generation cap and the supersession
itself are all golden-verified now.

Said plainly, because it is the one place these goldens are not the whole
engine: a stubbed `buildWall` never writes `wallState.ring` and never advances
`wallState.epoch`, so the supersession fixtures have to **preset** a ring, and
the age gate is measured from the initial epoch for every generation instead of
being re-armed. Both are identical on both sides and therefore parity-neutral.
Milestone 10 should re-run all 60 with the real builder.

Two more functions had to come forward for the same reason: `ringCrossings`
(line 29631, milestone 10's first function, six lines) and `distToLine` (line
28971, milestone 9's first line, three lines). They live in `growth` now;
milestones 9 and 10 should read them rather than port them again.

### `WallState` carries only what this milestone touches

`buildWall` writes nine fields milestone 7 does not model (`waterWalls`,
`spurs`, `spansWater`, `style`, `prov`, `fort`, `centroid`, `terrainDeflected`,
`_waterClosure`) and `supersedeWall` copies six of them into its history
record. None is modelled here — exactly as milestone 2 left `Graph::_fromPaths`
out until milestone 6 became the milestone that set it. Guessing the shape of
`fort` from a function this milestone does not port is the running-ahead this
port avoids; leaving a documented hole is not. **Milestone 10 must add them to
`WallState` and to `WallGeneration`'s copy list in the same pass**, or the
history record is silently lossy and every structural test still passes.

### Findings

- **`kept` is dead.** `grow` pushes `made[0].id` into a local array that is
  never read, returned or exported. Omitted rather than reproduced.
- **The wet-crossing walk takes six samples, not five, and the last is the
  segment's own endpoint.** `for (let t = 0.15; t <= 1; t += 0.17)` gives
  `0.15`, `0.32`, `0.49`, `0.66`, `0.8300000000000001`, `1.0`. The *reasoned*
  answer — accumulation drifts, the sixth is `1.0000000000000002`, the walk
  stops at five — was wrong twice over. Read out of `node` instead, which is the
  standing rule, confirmed again. **And the accumulation turns out not to be
  load-bearing at these three constants**: `0.15 + k * 0.17` is bit-identical on
  all six. Recorded as a measurement so a later milestone changing the step
  re-measures rather than inheriting either belief.
- **A `NaN` slope does not reject** — `NaN > 0.34` is false — so an all-`NaN`
  heightfield stops nothing in the legalisation. What it poisons is
  `estimateCarryingCapacity`: the ring average is `NaN`, `clamp` returns `NaN`,
  and `maxR` is `NaN` for the whole run, which makes every `dM > maxR` test
  false and **removes the reach limit** rather than stopping growth.
- **`opts.rules || DEFAULT_RULES` is the raw table**, milestone 4's correction,
  now proved by golden rather than by reading: a run with no `opts.rules`
  produces a byte-identical town to one passing an explicit `DEFAULT_RULES`, and
  both the capture gate and the Rust shape gate assert it.
- **`primEdges` is captured once per epoch, before any street is placed**, so
  streets laid this epoch cannot anchor this epoch's ribbon suburbs. A "hoist
  the filter" refactor would silently invert that.
- **`wallState.generation || 1` reads a stored `0` as `1`**, and the
  `genGenerationZero` fixture reaches it.
- **`Math.max(3, Math.floor(epochs * 0.6))` needs three fixtures, not two**: at
  2 epochs nothing fires, at 3 and at 5 it fires at epoch 3 (the `max` arm and
  the `floor` arm), at 8 it fires at 4.
- **A harbour with a one-point quay is still a harbour**, and produces the
  no-harbour town: `distToLine` under two points is `Infinity`, so
  `Math.min(dM, Infinity + 35)` is `dM`. `harbourEmptyQuay`'s graph hash equals
  `coastTown`'s, asserted.
- **`grow` always enters from `generate()` with `ring: null` and a resolved rule
  set** — checked, because the first draft of this note said the opposite.
  `generate()`'s only pre-`grow` `buildWall` is in the **radial** branch, and
  that branch does not call `grow` at all. So the fire-epoch arm is always live
  in production and the preset-ring fixtures here are a superset of what
  `generate()` can reach.

### Golden verification

Same slice harness as milestones 3-6 verbatim (contiguous 28167-31103 plus line
2291, the balance scan with milestone 2's orphan-close counter, milestone 3's
four structural assertions), with **three** anchored text edits this time, each
asserted to match exactly once: the `return {` replacement exposing the five
functions, the `buildWall` stub, and the per-epoch observer inside `grow`'s
loop.

Bit for bit through `to_bits`, no tolerances. `graph_hash` is the reference's
own `fnv1a` over its own canonical dump of every node and edge with each double
as its exact 64 bits — a bit-for-bit statement about the whole graph, not a
tolerance in disguise. The explicit node/edge dump is redundant strictness kept
only for the scenarios under 170 edges so a failure is readable, the same trade
milestone 6 made for the spatial index one scale up; it took `golden.rs` from
785 KB to 244 KB. `prov_hash` is a second `fnv1a` over every edge's provenance
string, which pins the Exploration/Densification split, the epoch stamp, and the
ring road's interpolated `Math.round(fillFraction * 100)`.

The capture's shape gate names the fixture behind each of its twenty-odd
conditions and refuses to write otherwise; the Rust side mirrors it, because
`zip` stops at the shorter side.

### Two rounds of fixtures lost to milestone 5's rule, in two disguises

**The terrain rasters were in metres.** `site.height` reads
`opts.terrain.grid` **raw** and `site.slope` multiplies a per-metre central
difference by **900**, so a grid holding 40-95 m of elevation produces slopes of
2 to 204 and `slope > 0.34` rejected every candidate on every raster-backed
site. Fifteen fixtures grew nothing; the only two that worked were the two with
no terrain raster at all. **Any raster-backed fixture in any later milestone
must use a normalised heightfield** — this will hit milestones 10, 13 and 15.

**A hand-drawn ring can never be 80% full.** The M-GRW-2a gate needs
`fillFraction >= 0.8` **and** `exteriorCount >= max(10, interior * 0.15)`, which
is the whole point of the metric. Ellipses topped out at 0.44; scaling them
about the market swept 0.30-0.80 and never passed 0.58, because a convex hull of
a real town's interior nodes does not fill an ellipse. Then the first
hull-derived attempt failed the *other* half: the hull of the whole built mass
at epoch 3 reaches the box edges along the primaries, so inflating it 8%
enclosed the finished town and left `exteriorCount` at **zero**. What works is
the hull of the built mass at epoch 3 restricted to 260 m of the market and
inflated 6% — roughly what `buildWall` itself constructs.

### Round 2: twelve fixtures, and seven survivors turned into assertions

Milestone 6's *add, do not substitute* rule, applied to a survivor list. Twelve
new scenarios were built for constants the first sweep left standing, and every
one **also matched the reference on the first run**: a closed square of four
**exactly-38 m** edges with no degree-1 node (so neither the mid-edge tap nor
the dead-end continuation can fire, and `<` and `<=` are different towns); two
small boxes where the four `40 m` box margins actually reject; a quay 40 m off
the market so `distToLine + 35` is really the smaller term; `160` years over 8
epochs, where `120 / 20` is **exactly 6.0** and the age gate's `>=` and `>`
differ by one epoch; a rule gap of `262.5` against the absent-`settlementAge`
default, where `262.5 / (300/8)` is **exactly 7.0**; a falsy `settlementAge: 0`
that must produce the byte-identical town to an absent one; a
`settlementAge: 0.5` with a 1-year gap, the only setting where
`Math.max(1, …)`'s floor decides anything inside 8 epochs; an extramural share
of `0.8` that blocks, which is what says the test multiplies the **interior**
count; a **scanned** ring radius (592 m) whose first supersession happens with
an exterior count of **exactly 10**, against a `max(10, …)` floor pinned by a
share of 0; the same circuit wound the other way; and a **two-point** `landArc`
between the one-point arc that lays no road and the long one that does.

Seven more were dealt with the other way. A proof does not *kill* a mutant — a
test asserting a constant cannot matter still passes when the constant changes —
so those are still counted as survivors. What changed is that each now rests on
an executable statement rather than a paragraph: the carrying-capacity clamp is
dead by construction (`terrainSuitability` is a product of two `[0, 1]` factors,
asserted over 720 probes across every site in the file); no node ever holds a
dead edge in `adj`, so `wallOccupancy`'s `alive` filter cannot bite until
milestone 11's `_killEdge` exists; the junction-angle double wrap is undone by
the `min(dd, π − dd)` that follows it, measured over 200,000 arguments; the
twelve `2π·i/12` ring angles are ones V8 and the platform agree on — asserted
together with >100 disagreements in 40,000 arbitrary angles, so the survivor
cannot be read as a licence anywhere else; a zero-area ring cannot contain a
node; `convexHull`'s winding never varies, while a caller-supplied ring's does;
and both non-`wallGenerations` fallbacks are assigned only when they are not
read.

### Mutation testing

Every numeric literal on a non-comment, non-string line of `growth.rs` (96),
plus **118 hand-written structural mutations** covering every draw and its
order, every comparator and tie-break, both `||` fallbacks, the epoch loop's
two origin branches, the reach and bank tests, the ribbon-suburb rule, the
demand gradient, every legalisation guard, the wet walk, the wall-permeability
loop, the parallel-spacing loop, the street class and width, the provenance
strings, all four arms of the wall episode, every field of the supersession
record, and both helpers borrowed forward. Patterns are validated to match
**exactly once in real code** before the sweep starts, numeric replacements are
made by `(line, column)`, comment and string text is stripped before scanning,
the runner takes a **pristine snapshot before it writes anything** and restores
from that, holds a lock file, runs on a **private `CARGO_TARGET_DIR`**, and
re-runs the suite as a post-sweep baseline.

**Two sweeps: 214 mutations / 51 survivors, then — after twelve new fixtures
and seven new assertions — 214 mutations, 176 died, 38 survived.** Every
survivor was re-run in isolation and **not one false survivor appeared in either
round**, the third milestone running for which the private target directory has
held.

**Eleven of the 214 are deliberate graded perturbations** — milestone 4's device
for a constant whose small change is absorbed — and **all eleven die**: `k`
`6.5 → 30`, the mid-edge minimum `38 → 300`, the junction minimum `18 → 400`,
the slope limit `0.34 → 0.001`, the gate radius `20 → 4000`, the tapped-frontage
skip `1.5 → 500`, the parallel-angle limit `0.5 → 3.2`, the exploration band
`+140 → +5`, the ribbon-suburb radius `90 → 2`, the interior-node floor
`8 → 400`, and the try budget `2600 → 12`. Each says *this constant is tested; a
37% nudge is simply below what the fixture can express.*

#### The 38 survivors, by the invariant each rests on

| class | n | why they survive |
|---|---|---|
| **an exact tie on a continuous value** | 13 | `len < budget`, the bridgehead distance and probability, the 90 m ribbon radius, `h.u > 1e-3`, `h.t > 0.03`, `h.t < hitT`, the 18 m junction minimum, the junction-angle limit, the 0.34 slope limit, the 20 m gate radius, the parallel spacing, and `fillFraction >= 0.8`. **Milestone 3's finding recurring**, and here it cannot be closed the way milestone 3 closed it: every one of these inputs is a polyline distance, an angle, a hull-area ratio or a raw `mulberry32` draw, none of which a quantised raster can pin. Where the boundary *was* integer arithmetic — the age gate, the extramural floor, the 38 m minimum — round 2 built the fixture and the mutant died |
| **proved dead or a no-op, with an executable assertion** | 11 | both carrying-capacity clamp bounds; `wallArea > 0` twice (a zero-area ring contains no node, so the `interior >= 8` beside it can never hold); `ccFactor`'s `: 1` and `yearsPerEpoch`'s `: 0` (assigned only when `wallGenerations` is off, read only when it is on); the probe-ring rotation `i → i+1` (twelve evenly spaced angles are the same twelve points); `js_cos → f64::cos` (V8 and the platform agree on all twelve of *these* angles, asserted together with >100 disagreements over arbitrary ones so it cannot be read as a licence elsewhere); the `alive` filter on `adj` (no node ever holds a dead edge until milestone 11's `_killEdge`); `abs` on the hull area (`convexHull`'s winding never varies — the `abs` on the **ring** does matter, and `genRingReversed` shows it); and the junction-angle double wrap (undone by the `min(dd, π − dd)` that follows it, measured over 200,000 arguments) |
| **an exact integer count no town produced** | 4 | `interior.len() >= 8` in both directions and `hull.len() >= 3` in both. Quantised and therefore closable in principle — it needs a circuit containing *exactly* eight built interior nodes, or one whose interior hull has *exactly* three vertices, while still passing the fill and extramural gates. None of the 60 towns lands there, and unlike the 38 m edge these cannot be constructed by hand: the counts are outputs of the growth loop, not inputs to it |
| **a bound no reachable value approaches** | 5 | `tries < 2600` → 2601 (a 2,601st attempt after 2,600 failures still places nothing); `h.u < 1 − 1e-3` widened twice (`segInt` only ever returns `u ∈ [0, 1]`, so raising the ceiling admits nothing); the wet walk's start `0.15 → 0.3155` (no fixture has a segment wet *only* in that opening slice); and `fmt_js_int`'s `n > 0` sign test, which needs an infinite `fillFraction` |
| **three of the four 40 m box margins** | 2 | the small-box fixtures made growth bind against one edge and killed that side; the other two need their own site whose *growth* is bounded by that specific edge. **Milestone 6's 80 m margin finding recurring exactly** — a margin is invisible until the candidates it removes were going to be kept |
| **provably equivalent rewrites** | 3 | the tapped-frontage skip `1.5 → 2.165` (the frontage sits at ~0 and every other edge is far past either value); `edgesNear(midp, midp) → edgesNear(O, B)` (a superset of cells, but the `d < 24` test measured from `midp` rejects every extra one); and `arc.length > 1 → > 0` (a one-point polyline yields no consecutive pair, so `addPolylineStreet` lays nothing either way) |

### Verification

`cargo build -p cartalith-urban`, `cargo test -p cartalith-urban` (84 passed, 0
failed) and `cargo clippy -p cartalith-urban --all-targets` all clean.
`cargo test --workspace --exclude cartalith-godot` against a baseline taken
before the milestone, with the delta accounted for. `cargo fmt` not run. No
Godot file touched (UI hold). Nothing wired into `compute_civilisation()`,
`cartalith-godot` or the GUI.

## Journey Planner — the Godot boundary (`JOURNEY_PLANNER_SCOPE.md` closing-status steps 1, 2 and 4, 2026-08-19)

`JOURNEY_PLANNER_SCOPE.md` closed on 2026-08-18 with the engine done and the
feature not: 65 of the reference's 74 `jp*`/`_jp*` functions ported and
golden-tested in `cartalith-civ`, and **zero `#[func]`s for any of them**. Its
own closing section listed five things making it real would need, in order.
Three are now built. Nothing here is new modelling — the algorithms were already
golden-verified, and this pass adds none.

### What was exposed

One new `#[godot_api(secondary)]` block in `cartalith-godot/src/lib.rs`:

- **`jp_options() -> Dictionary`** — every dropdown vocabulary, keyed by the same
  field names `jp_compute` accepts, so a form is built by walking it rather than
  by hard-coding a second copy of the vocabulary in GDScript. `route_cond` nests
  one level deeper (`{land, river, sea}`) because a "Maintained" road condition
  cannot describe a sea leg and `_jpDeriveStages` rejects it when it tries;
  `reference` carries the terrain/biome/category/animal tables a *results* panel
  needs to label what came back. Pure — callable before `generate()`.
- **`jp_default_plan() -> Dictionary`** — `JpPlan::default()` (the reference's
  own `_jpEnsurePlan` default block) flat, 28 keys plus `party_fields`.
- **`jp_compute(request: Dictionary) -> Dictionary`** — `jp_plan` → `jp_verdict`
  → `jp_confidence`. `request` takes `route` (an int index into the committed
  routes) or `points` (a `PackedVector2Array`), plus optional `plan`,
  `stage_overrides` (`{stage_index: {field: value}}`) and `layovers`
  (`{stop_key: days}`, keyed by the `key` each returned stop carries).

Plus two in the existing INFRA block — **`route_count()`** and
**`route_get(index)`** (`{points, brks, km, mode, unreachable_legs}`).

And five wrappers in `godot-project/shell/engine_bridge.gd`, `has_method()`-guarded
like the other ~94. **No workspace script calls any of them** — the party form
and results panel are a separate, deliberate follow-up (steps 3 and 5).

### The route-getter gap was real

`UNIFIED_TOOL_PLAN.md` milestone F's own note — *"there is no getter for the
manual-ways list itself yet, deliberately out of this milestone's exact scope"* —
still held: `route_commit()` returned an index into a list nothing could read
back. `route_get`/`route_count` close it for routes, and `jp_compute`'s `route`
key is the first consumer. It is preferred over `points` precisely because it
reads the route's own `f64` grid coordinates; `PackedVector2Array` is `f32`, so a
route round-tripped through Godot is a rounded copy of itself.

### `JpWorld` needed no new pipeline state

The scope document claimed every field it borrows was already computed by this
port. That checked out. `field`/`temperature`/`rainfall`/`flow_discharge` come
from `WorldState`; `water_bodies`/`territory`/`ways`/`settlements` from
`CivData`; `peak_m` from `WorldParams`; `flow_thresh` from the same
`cartalith_hydrology::river_flow_thresh` call `compute_civilisation` already
makes. Its prediction about the two exceptions also held exactly:
`build_cart_biome`/`build_cart_terrain` still have no pipeline-stage caller, so
`journey_bridge::JourneyWorld` computes them per call from rasters that already
exist, alongside `jp_road_cells`. **No generation stage was added.**

### Three inputs are empty on purpose, and say so

- **`ocean_field`/`wind_field` are `None`.** This port's climate stage computes
  the ocean-current vector field *inside* `cartalith_climate::ocean_sst_anomaly`
  and discards it — nothing in `WorldState` retains a `u`/`v` pair at any
  resolution, so there is no `currentOceanField()`/`currentWindField()` to hand
  over. `None` is `jp_sea_condition`'s own supported input: a sea leg reads its
  structural condition and skips the wind/current term rather than reading an
  invented one. A third entry for the scope document's "quality ceilings" list.
- **`road_cells` sees the generated way network only.** `jp_road_cells` takes
  `&[Way]`; hand-drawn ways are `tools::ManualWay`, whose `Ancient` variant
  `jp_road_cells` has no branch for — the reference's `_jpRoadCells` *does*
  (`'ancient' → ["Dirt Track","Deteriorated"]`), because its one `civWays` array
  holds both kinds. Widening `jp_road_cells` is a `cartalith-civ` change against
  golden-tested code; reported, not approximated.
- **`road_edges` is empty.** `build_road_network`'s `RoadEdge` list is not
  retained by `compute_civilisation` and has no live equivalent to the
  reference's `state.roads.edges`.

`wildlife_forage_mod` is `|_, _| 1.0` — the reference's own answer on a world
with no wildlife layer, already disclosed by the scope document.

### One reference behaviour preserved rather than "fixed"

`jp_claimed_at` tests `territory[i] >= 0`, and this port's `assign_territory`
uses `0` for unowned — so every cell reads as claimed. That is exactly what the
reference does: its `civTerritory` is a `Uint8Array`, so `>= 0` is likewise
always true. `civ.territory` is passed through unchanged. Remapping it here would
have been a silent divergence from a golden-verified consumer, dressed up as a
bug fix.

### Where the work actually was

Not the plumbing. `JpJourneyPlan` is `stages` + `results` + `timeline` + `stops`
plus ~25 scalars, and each `JpLegResult` carries a whole second `JpPlan` (the
*effective* plan that leg was computed under, which season drift and the
per-stage vessel fallback can both have altered) alongside its own land or water
calculation and that calculation's own `JpCapacity`/`JpResupply`. Nine helper
functions in `lib.rs`, following `icon_dict`/`label_dict`'s existing `vdict!`
pattern at each nesting depth. The scope document's guess that step 4 would be
*"small once 1-3 exist"* was half right.

`journey_bridge.rs` itself is **`godot`-free**, the same isolation
`sculpt_bridge`/`civ_tools_bridge`/`infra_tools_bridge` established: it owns the
plan/party form parser and its inverse, the `JourneyWorld` buffers and the option
tables; `lib.rs` owns the `Variant` conversion and the flattening.

### Verification

- 28 new plain-Rust tests in `journey_bridge.rs` — no Godot runtime, run by
  `cargo test -p cartalith-godot`'s ordinary unit pass. The form parse, the
  flatten/reparse round trip (both a fully-populated plan and the default),
  per-stage overrides, and the `Int`/`Num` acceptance rule.
- **One recogniser test per option table, pinned against the engine's own
  lookup.** This is the failure mode worth the tests: a dropdown offering a key
  the engine does not recognise never errors — it falls through to `?? 1.0` and
  reports a plausible number computed from the wrong row. Five of the tables
  (`JP_PACE`, `JP_GRAZING`, `JP_FORAGING`, `JP_REST_CADENCES`, `JP_ROUTE`) are
  `match` arms rather than `pub const`s, so a transcription typo is otherwise
  invisible. Where the lookup is private (`jp_grazing`) the test proves
  recognition indirectly and says how: an unrecognised key collapses onto the
  same fodder fraction as the first key, so three *distinct* fodder masses is
  exactly the proof.
- One end-to-end test that the assembled `JourneyWorld` really drives `jp_plan`
  over synthetic rasters of the shapes `WorldGen` holds — a real 14-cell land
  traverse with both endpoint settlements appearing in `stops` — rather than
  merely producing non-empty buffers.
- `cargo build -p cartalith-godot` clean; `cargo clippy -p cartalith-godot
  --all-targets` clean for the new code. **`cargo clean -p cartalith-godot`
  followed by a full rebuild**, then `cargo test -p cartalith-godot`: 153 unit
  tests + 31 integration tests, 0 failed — the stale-`.rlib` failure mode this
  project has documented, ruled out rather than assumed away.
- Godot 4.7.1 headless boot of `godot-project` clean, extension initialised. A
  scripted headless smoke run confirmed all five methods register and work end to
  end: generate → `route_begin`/`route_append_stop`/`route_commit` → `route_get`
  (1157 km, 35 points, `land`) → `jp_compute` (11 stages, 3 stops, 24.0 travel
  days, verdict `severe`, confidence band 25.2-39.8 days), with a deliberately
  bogus plan key correctly reported in `rejected` and the per-leg `eff` season
  reflecting the request. The smoke script was removed after the run.
- No golden tests added, and none needed: the `jp_*` functions underneath are
  already golden-verified and this pass adds no algorithm.
- `reference/Cartalith Gen1 v2.10.html` untouched. No file under
  `godot-project/shell/workspaces/`, `map_overlay.gd`, `tool_overlay.gd` or
  `right_dock.gd` touched — the UI hold and the deliberate step-3/step-5
  boundary both respected.

## Sample panel + Layers popover — §6's twelve dashed fields, and no new retention (2026-08-19)

**All twelve of §6's dashed Sample fields are live, and none of them needed a
byte of new retention.** `right_dock.gd`'s `MISSING_SAMPLE_FIELDS` listed
twelve readouts (slope, aspect, plate + type, boundary + distance, resistance,
lithology, temperature, precipitation, drainage, biome, soil, control) that
read `—` always, each with "no per-cell query" against a `WorldGen` that
exported no field sampler. `sample_bridge.rs` is that sampler. Elevation — the
thirteenth, §6's large accent readout — was dashed too and is now metres above
sea level.

**Nothing was added to `WorldGen`, `WorldState` or `CivData`.** Every reading
is either a raster generation already keeps or is derived from those at the
one queried cell:

| Field | Source | Cost per query |
|---|---|---|
| elevation | `WorldState::field` + `metersPerUnit()`'s own anchoring | O(1) |
| slope, aspect | central difference of `field` at the cell | O(1) |
| plate + type | `plate_id`, oceanic/continental from `crust_field`'s sign | O(1) |
| boundary + type | `boundary_mask` + `boundary_type` | O(1) |
| boundary distance | ring search over `boundary_mask`, capped at 96 cells | O(d²) |
| resistance, temperature, precipitation, drainage | the same-named `WorldState` fields | O(1) |
| river order | `WorldState::stream_order` | O(1) |
| lithology, soil | `build_lithology`/`build_soil_fertility` **called on one-element slices** | O(1) |
| biome | `CivData::water_bodies` + `classify_biome(t, m)` | O(1) |
| control | `CivData::territory` | O(1) |

**One prior comment was wrong and is corrected in place, not deleted
quietly.** The Biome row claimed `explain_settlement()`'s doc comment meant
"retaining the rasters for arbitrary-cell queries would cost hundreds of MB at
production resolutions." That doc comment is about the *suitability* rasters
(coast SDF, river order, travel cost, the weighted terms), which genuinely are
computed and dropped inside `compute_civilisation`. Biome is not one of them:
`build_water_bodies`' classification has been retained on `CivData` since the
Settlement tool needed snap-to-water, and `classify_biome` is a pure
two-argument function over two rasters `WorldState` already holds. Nothing in
`MEMORY_OPTIMIZATION_SCOPE.md`'s budget had to move.

**Lithology and soil are derived without copying a single formula.** Both
`build_lithology` and `build_soil_fertility` are strictly per-cell (the
lithology port's own doc comment: *"Pure, single-pass, no neighbour reads"*),
so they are called on one-element slices — bit-identical to indexing the
full-grid result, with none of their golden-tested branches restated in
`cartalith-godot`. `one_cell_lithology_and_soil_match_the_full_grid` asserts
that equality at every cell of a 16×12 fixture.

**Aspect is new work and says so.** The reference's `aspectFactor` (line 7590)
is a shading scalar — a signed north-south derivative flipped by hemisphere —
not a compass bearing. The Sample panel's Aspect is the standard GIS downslope
azimuth off the same central difference. **No parity claim is made for it**,
and the first implementation was 180° out (it reported the *uphill* bearing);
`aspect_points_downhill` caught that, which is why the test exists.

**New `#[func]` surface** (`lib.rs`, one new `#[godot_api(secondary)]` block):

| method | what it is |
|---|---|
| `sample_cell(gx, gy) -> Dictionary` | every §6 field for one cell in **one** call — `on_cursor_sampled` fires on every mouse-motion event, and sixteen per-field getters would be sixteen boundary crossings per motion. Keys whose backing data genuinely is not there are **omitted, never zero-filled**. `{}` for an out-of-grid cell rather than clamping to an edge and reporting a neighbour's readings. |
| `debug_layers() -> Array` | the popover's grouped menu in the reference's own `LAYER_GROUPS` order, each row carrying `available` and its legend swatches. |
| `build_debug_texture(view) -> ImageTexture` | one field view as a grid-sized RGBA texture. **Nothing is cached** — caching all seventeen would be ~270 MB at 2048² — so re-picking re-derives. |

**Debug views: 18, in the reference's own six groups.** Base (no overlay,
elevation), Climate (temperature, rainfall), Tectonics (plates, boundaries,
tectonic type, stress, crust age, **resistance**), Hydrology (river flow,
Strahler order), Surface (biomes, terrain, lithology, soil fertility,
**slope**, **aspect**), Civilization (political control). Every ramp that
exists in the reference is ported from its own debug-overlay pixel loop
(lines 8470-8530) and palette constants — `tempColor`, `rainColor`,
`divColor`, `hsl`, `hypso`, `LITH_COLS`, `BTYPE_COLS`, `CART_BIOME_COLS`,
`CART_TERRAIN_COLS` — pinned by `ported_palettes_match_the_reference`. The
four bold ones have **no reference counterpart** (the reference's base map
*is* elevation, and it never drew slope, aspect or resistance); their ramps
are this port's own and each row's hint says so.

**The Layers popover is real.** `layers_popover.gd` (new), opened by §9's
layers button. It used to emit a signal that `app.gd` answered by selecting
the Cartography domain — a stand-in for exactly this. Nothing that stand-in
reached is removed: `cartography_workspace.gd`'s Visible-layers toggles and
`ViewportHost.set_layer_visible()` are untouched, still on the rail, and the
popover's footer points at them. The popover carries the grouped picker, the
active view's legend, and the reference's own `#dbgOpacity` slider blending
the field raster over the base map. Like the reference's, it stays open across
picks. A view whose one input this world lacks (Strahler without river
extraction, biomes/terrain/control on a loaded save) comes back
`available: false` and is drawn greyed with the reason in its tooltip, rather
than offered and then silently doing nothing.

**`available` is O(1), deliberately.** The first version answered it by
building each raster and seeing whether it worked, which at 2048² would have
derived seventeen full-grid rasters every time the popover opened.
`layer_available` reads which *inputs* exist instead, and
`available_matches_debug_raster` pins the cheap answer against the expensive
one across both civ/no-civ and both rivers/no-rivers fixtures so they cannot
disagree.

**Nothing was left open for want of retention** — no field required a raster
this pipeline computes and discards, so there is no disclosed cost estimate to
report and no `DECISIONS.md`-level change to raise.

**Tests**: 15 new plain-Rust tests in `sample_bridge.rs` (169 unit tests pass
after `cargo clean -p cartalith-godot` + full rebuild), clippy clean for the
new code. Two headless smoke runs against a real generated world: a sampler
run over six cells plus a found plate-boundary cell, asserting plausible
ranges rather than merely non-crashing (elevation in [0,1], metres in
±12 km, temperature in [−90, 70] °C, precipitation and soil in [0,1], slope
in [0, 90]°, aspect in [0, 360), a named lithology, a real plate id, biome and
control present) and all 18 views drawing at the right size and not as a flat
colour; and a full-app run that generated a 192² world, drove
`on_cursor_sampled` and read every Sample row back live (`-124 m · ocean`,
`Slope 2.1° · n 3.67`, `Aspect SW 233°`, `Plate + type 10 · oceanic`,
`Boundary + distance transform (shear) · on it`, …), confirmed every row
resets to `—` when the cursor leaves the map, built the popover's 19 rows,
picked four views, checked the legend followed, and confirmed the layers
button opens the popover. Headless Godot 4.7.1 boot clean.

**Still open, deliberately**: `DCC_SHELL_SPEC.md` §6's *Layers* right-dock
context (the ordered list with per-layer opacity bars and blend modes, nested
children under Terrain) is a different thing from this canvas popover and is
not built; §7's Layer-properties/ramp-editor panes still have no
`TerrainAppearance` binding behind them, unchanged by this pass.

## Journey Planner — distance-spine takeover, replacing the AcceptDialog (`JOURNEY_PLANNER_SPEC.md`, 2026-08-19)

Same day, second pass. The party form + results window built earlier
(`journey_planner_window.gd`, `extends AcceptDialog`) was real and working —
route picker, the full plan form seeded from `jp_default_plan()`, per-stage
overrides, a results panel — but a popup modal over the map, exactly the
layout the owner's new mockup (`design/Journey Planner DCC.dc.html`, screen
`1a`) replaces. `DCC_SHELL_SPEC.md` §4.5.4 got a same-day correction note
recording the architecture: Journey is a third INFRA tool (alongside Way,
Route), armed via the rail-foot context slot, with **no map click/drag
gesture** — arming it swaps the entire INFRA viewport region (map, both
docks, tool options bar) for the mockup's distance-spine planner in place,
rather than drawing an overlay on the map the way Way/Route do, and unlike
Travel library (a real separate window, later work, not touched here).

**`journey_planner_window.gd` is deleted.** Its field-binding and results-
rendering logic was carried forward into the new `journey_planner_view.gd`
(not rewritten from scratch) — the route picker, the `_choice_field`/
`_number_field`/`_toggle_field`/`_route_cond_field` vocabulary, the
`stage_overrides` request shape, and the verdict/confidence/totals/stage/
stops/timeline rendering all reappear here, re-laid-out into the mockup's
regions rather than re-derived.

**Architecture**: one new class, `JourneyPlannerView`, builds three region
roots once at `setup()` time and hides them — `_left_panel` appended to
`app.left_dock_body` (journeys list + the party form, grouped exactly as the
mockup: Traveler / Season & weather / Carriage / Route conditions), a
full-rect `_center_panel` appended to `app.viewport_content` next to
`app.viewport` (route map, terrain profile, stops strip, stage inspector,
stage matrix), and a delegate into `right_dock.gd`'s existing dispatch (a new
`CTX_JOURNEY`, mirroring the `CTX_SCULPT` precedent already there, so
`right_dock_body`'s single owner is never fought over). Visibility is
recomputed off two signals that already existed, `app.tool_armed` and
`app.workspace_changed`, as `armed_tool == "journey" && active_domain() ==
"infrastructure"` — not a single stored flag — so switching domains away and
back while Journey stays armed (the "one tool armed at a time, globally" rule
every other tool already lives under) restores the swap instead of leaving
stale chrome. Verified by a scripted headless run, described below.

**What's real, traced to a live call**:

- Route map and terrain-profile stage bands: `route_get()`'s own points,
  sliced per `plan.stages[i].{i0, i1}` — the exact index range `jp_plan`
  derived that stage over — coloured by category (land/river/sea/blocked),
  not a decorative SVG curve. Settlement markers are `plan.stops[]`'s real
  x/y, fit to the same bounding-box transform.
- The elevation sparkline: `plan.profile`'s real 0-1 normalised samples,
  drawn for the first time — the AcceptDialog pass reported the array's
  presence and stopped (`plan.day_fracs`/`plan.results[i].eff` were the
  disclosed leftovers too, but the profile was the one worth closing once
  the view was being rebuilt anyway).
- Stops-strip x-position: the nearest route point's cumulative chord length
  over the route's total chord length — exact for position purposes, since
  `map_width_km` is uniform across the grid, so chord-length fraction and
  km fraction are the same number.
- Stage inspector's 15 override fields (§6) and the stage matrix's editable
  mode/pace/hours columns both write into `jp_compute`'s real
  `stage_overrides` map and trigger a real recompute.
- Results panel: verdict/confidence/Time/Load/Supply reach/Vessels all read
  real `jp_compute` fields; a stage-table CSV export goes to the OS
  clipboard (no file-writer exists to save it to disk, same gap as every
  other save action in this shell).

**What's disclosed rather than faked** (checked against `journey_bridge.rs`
and `cartalith-civ`, not guessed):

- **Carriage Auto mode** has no Rust port of the reference's own
  `jpAutoPickTransport` (reference HTML ~line 19617) — no auto-carriage
  function exists anywhere in `cartalith-civ`. Selecting Auto disables
  editing the animal/vehicle counts and states the gap; it does not compute
  a plausible-looking pick.
- **Party presets** (`JP_PRESETS`, reference-JS-only, no `jp_presets()`
  binding) and **re-route-for-mode** (`_jpRerouteForMode`, same gap) are
  both present in the tool options bar, disabled, with the reason stated.
- **Cost** results group: `jp_journey_plan_dict`'s full field list (checked
  line by line against `lib.rs`) carries no monetary figures at all — no
  food/fodder/wages/tolls/upkeep sums exist to show.
- **⇧-drag spine trim** is deferred, not faked — `jp_compute` has no request
  field a trim gesture could feed; click-to-select and ⌥-click-isolate are
  both real and implemented.
- **Calculation trace** (`⧉`) is a disabled stub, matching this shell's own
  precedent for every genuinely-unbuilt window.

**One field-count question resolved, not guessed at**: `JOURNEY_PLANNER_SPEC
.md` §5 says "all 26, unchanged from v2.10" party fields. The live
`jp_default_plan()` call returns 28 real plan fields (`plan_to_pairs`' own
28-entry list, already correctly documented in `STATUS.md`'s Journey Planner
Godot boundary section as "28 keys + `party_fields`") grouped into the
mockup's own four left-dock sections — Traveler, Season & weather, Carriage,
Route conditions. The mockup itself (`design/Journey Planner DCC.dc.html`
lines 238-297, read literally rather than from the prose spec) has **no
fifth "Stops" group in the left dock at all** — Stops is the separate 32px
centre strip §3's own region table already lists on its own row. The spec's
prose undercount is corrected here rather than propagated into the build.

**Wiring**: `Data ▸ Journey planner… ⇧J` (`menus.gd`, new
`ID_JOURNEY_PLANNER`) and the INFRA dock's own Logistics "Open Journey
Planner" button (`infrastructure_workspace.gd`, call site unchanged) both
resolve to `app.open_journey_planner() -> journey_planner_view.open() ->
app.arm_tool("journey")`. The mockup's own "rail-foot slot" phrasing is
honoured as the tool's context readout (`app.set_rail_foot("JOURNEY")`,
reusing `DccShell`'s existing shared `rail_foot` Label) rather than built out
as a second independently-clickable target — `dcc_shell.gd`'s `rail_foot` is
shared across every domain's context text, and making only INFRA's foot cell
clickable would be a shared-base-class change for a capability the dock
button already provides. Recorded as a real, disclosed deviation from a
literal reading of "tool-foot slot" as its own entry point, not a silent
skip.

**Also added**: three colour tokens (`warn` #e0a840, `block` #b55950, `water`
#7d9dae) to `DccTheme`, read off the mockup's own inline styles — the shell
had `accent`/`stale` but nothing for "strained"/"blocked"/"water leg" before
this feature needed them, in both palettes so light mode (not yet designed
for this feature, §10's own "still to build" list) never hits `DccTheme.c()`'s
unknown-token error. A `tool_journey` glyph and three text symbols
(`blocked`/`warn_tri`/`bolt`, the mockup's own ⛔/⚠/⚡) in `DccIcons`.

**Verified**: headless boot clean (`--headless --path . --quit`, zero parse
or runtime errors) after fixing two real Variant-typing issues the strict
warnings-as-errors build caught (`Callable.call()`'s return is always
`Variant`; two `_draw()` sites needed an explicit `: Vector2` annotation
rather than `:=`). A scripted smoke run (written to `_journey_test.gd`,
exercised, then deleted — not committed, matching this port's convention of
not leaving one-off harnesses behind) generated a small world, committed a
real two-point route, armed the tool and confirmed both docks plus the
centre region swapped while `app.viewport` hid, confirmed a real
`jp_compute` result (14 derived stages, `ok: true`), selected a stage and
applied a real `pace` override with a confirmed recompute, disarmed and
confirmed full restoration (map visible again, left panel hidden), then
armed again while on the `world` domain (confirmed the view correctly stayed
hidden) and switched back to `infrastructure` (confirmed it reappeared
without re-arming) — every step passed.

**Not attempted this pass, disclosed rather than silently skipped**: light
theme (`JOURNEY_PLANNER_SPEC.md` §10's own "still to build" list), the 2560
tablet breakpoint, and a blocked-stage inspector state visually distinct from
the `block`-token colouring already applied throughout the stage inspector,
matrix, profile band and route map.

## Timeline milestone 1 — the `_civSettlementPopulation` dependency chain, shared tier tables, stable ids (`TIMELINE_SCOPE.md` milestone 1, 2026-08-19)

The shared prerequisite both later Timeline/collapse-recovery milestones
(proximity graph, the actual collapse/recovery stepper, the snapshot data
model) are blocked on. `TIMELINE_SCOPE.md` itself is the freshly-written
scoping pass this milestone follows; its §9 "Decisions" section resolved
three open questions in-flight (recorded there and repeated here per this
port's own discipline of not letting a design choice live in only one
document).

**Built** — a new `cartalith-civ::timeline` module:

- **The population-ceiling chain** (reference lines 23313-23512):
  `subsistenceModeAt`/`agrarianDensityKm2` (the land-use-mode density
  bands), `currentAgrarianDensity` (the per-cell field, normalised onto the
  pre-v1.31 `Σ K×AGRARIAN_MAX_KM2` basis — ported without the reference's
  own per-world cache, matching this crate's existing stateless-field-
  builder convention), `_civCatchmentDensityMean`/`_civCatchmentPop`
  (world-wrap-aware disc-mean over a settlement's catchment), the
  `_CIV_SURPLUS_FRACTION`/`_CIV_TRADE_K` tables, and `_civSettlementPopulation`
  itself. Built on the two pieces `TIMELINE_SCOPE.md` §3 identified as
  already-real and reusable: `build_carrying_capacity` (the `K` field) and
  `civ_catchment_km2`/`civ_catchment_radius_cells`.
  - **One parameter dropped, not silently**: the reference's own `K`
    argument to `_civCatchmentPop` only feeds its dead fallback branch
    (`typeof currentAgrarianDensity==='function'` is always true — a
    hoisted top-level function declaration — so the `K`-based fallback
    path can never execute). `civ_catchment_pop` takes `dens: &[f32]`
    directly instead; `K` still shapes the answer, just one level removed
    (it's what `civ_current_agrarian_density` builds `dens` from).
- **The shared tier tables**: `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR`/
  `_civTierForPopulation`, and `_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME` as
  a `RecoveryPhase` enum (ported because milestone 1's own scope bullet
  names them as shared with `_civApplyRecovery`, even though nothing
  constructs a `RecoveryPhase` yet — see the `_civApplyRecovery` decision
  below).
- **A stable id (`tid`)** on `NamedSettlement`/`Way` (`cartalith-civ/src/
  lib.rs`), plus `civ_assign_tid`/`civ_resync_next_tid` in the new module.

**Three decisions made in-flight** (`TIMELINE_SCOPE.md` §9), recorded here
per this port's discipline of never letting a design choice live in only
one document:

- **Metropolis tier: capped at `Capital`.** The reference's ported tier
  table has six entries (`metropolis` highest, floor 150000); this port's
  `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR` have five, stopping at `Capital` —
  `SettlementKind` gets no `Metropolis` variant in this pass.
  `_civSelectMetropolises` (the promotion pass that would produce one) is a
  separate, still-unported gap (`PHASE2_SCOPE.md`), tracked there, not
  invented here. The cap needs no special-casing: `civ_tier_for_population`
  walks the order high-to-low, and `Capital`'s own floor (30000) is still
  the first satisfied entry for a population the reference would have
  called `metropolis` (verified up to 5,000,000 in the golden test below).
- **`_civApplyRecovery`: out of scope.** The v0.82 static/instant recovery
  pass (`_civIterativeAutoWorld`'s auto-populate "Recovery phase" dropdown)
  is adjacent — it shares the tier tables with this milestone's own
  population chain, which is why `RecoveryPhase` is ported here — but
  porting the pass itself is left for a future `PHASE2_SCOPE.md` addendum,
  not bundled in.
- **Stable id (`tid`) design: eager assignment, not the reference's lazy
  first-touch.** The reference's own `_civAssignTid` only ever stamps an
  object's `tid` the first time something touches it (empirically,
  `civSnapshotSave`, milestone 4's own territory). This port assigns
  eagerly instead — at settlement-placement/road-generation time
  (`compute_civilisation`, `cartalith-godot/src/lib.rs`) and at every later
  manual-insertion point (`civ_tools_bridge::drop_settlement`) — because
  `cartalith-civ` is stateless (`ARCHITECTURE.md`) and the reference's lazy
  trigger has no clean pure-function home here, while "assign it when the
  object is created" does. `0` is the "unassigned" sentinel (`tid==null`'s
  Rust analogue); `civ_assign_tid` is idempotent, so a later touch-point
  (milestone 4's snapshot save, if it wants one) is safe to add without
  double-assigning. The counter (`next_tid`) lives on `CivData`
  (`cartalith-godot`) — the one place this port's civ state is actually
  mutable — with `civ_resync_next_tid` (milestone-1 scope: scans live
  settlements/ways only, not yet timeline snapshot history, which doesn't
  exist until milestone 4) as the pure rescan `_civResyncNextTid` mirrors.
  New design, not a reference algorithm to golden-match (`DECISIONS.md`
  §7a "principled equivalence").

**Every construction site of `NamedSettlement`/`Way` updated**, grepped
across the whole workspace (`cartalith-civ`'s own `lib.rs`/`tools.rs` and
every `tests/` fixture, `cartalith-godot`'s `civ_tools_bridge.rs`/
`infra_tools_bridge.rs`/`journey_bridge.rs`/`lib.rs`) — production sites get
`tid: 0` at construction (assigned for real one step later, at the
`compute_civilisation`/`drop_settlement` boundary); test fixtures that don't
exercise tid semantics get `tid: 0` and stay that way.

**Golden-verified against the real reference**: a Node `vm.runInContext`
harness (transient, not checked in) sliced the exact verified line ranges
(23313-23512, 24614-24618) and ran them with `currentCarryingCapacity`/
`currentWaterAccess`/`buildBiomeRaster`/`currentAgrarianDensity` stubbed to
return hand-picked arrays — legitimate because every function under test is
"pure over the supplied per-cell field" per its own reference doc comment,
so feeding a known-good input directly is `PARITY_TESTING.md`'s own "one
test per pipeline stage" guidance, not a shortcut around it. 9 tests, 25
individual golden comparisons, real reference numbers (`cartalith-civ/
tests/golden_parity_settlement_population.rs`):

- `subsistenceModeAt`/`agrarianDensityKm2`: 11 cases across all four
  biome-excluded codes (ocean/ice/tundra/desert) and both boundaries of
  every mode transition (e.g. `k=0.45,water=0.35,rain=0.249999` → short
  fallow density `17.55`, one float below the `rain=0.25` annual-
  cultivation threshold's `72.0`), plus a `NaN` k case (`0.0`, matching
  JS's `K||0`).
- `currentAgrarianDensity`: a 3-cell mixed land/sea fixture — reference
  answer `[210.07957458496094, 9.920424461364746, 0]` (`Float32Array` in
  the reference, so already f32-rounded) — plus an all-sea fixture
  confirming the `rawSum>0?refSum/rawSum:1` fallback, not a divide by zero.
- `_civCatchmentDensityMean`: a sea-cell-excluded disc mean (`11.75`), a
  world-wrap-vs-not pair on the same fixture (`4.8` vs `4.25`), and an
  all-sea zero.
- `_civCatchmentPop`/`_civSettlementPopulation`: all five `SettlementKind`s
  on a uniform-density fixture where every kind's catchment radius floors
  to 1 cell — isolating the per-tier catchment-area scaling
  (hamlet 60 → capital 14000) and the surplus/trade-concentration formula
  at `normB=0` and `normB=1` (e.g. capital: `1540` → `4466`) — plus a
  `NaN normB` / all-sea zero case.
- `_civTierForPopulation`: every floor boundary from `hamlet` through
  `capital`, plus the two rows (150000, 5000000) where the reference says
  `"metropolis"` and this port's capped table says `Capital` — the
  documented divergence point, asserted rather than silently matched.

**Verified**: `cargo build -p cartalith-godot` (the cdylib, not just `cargo
test`) and a headless Godot 4.7.1 boot (`--headless --path godot-project
--quit`) both clean. `cargo test -p cartalith-civ -p cartalith-godot`: all
passing (291 + 9 new golden tests in `cartalith-civ`, 170 + 4 tid-focused
tests in `cartalith-godot`, including a new `drop_settlement` test pinning
that a hand-placed settlement gets a real monotonically-increasing tid and
that re-selecting an existing settlement never advances the counter).
Clippy clean on the new code (one pre-existing-style `1 * gw` readability
warning left as-is, matching this crate's own established grid-indexing
convention elsewhere).

**Out of scope, per `TIMELINE_SCOPE.md`**: the proximity graph/betweenness
centrality (milestone 2), the collapse/recovery step functions (milestone
3), the snapshot data model/orchestrator (milestone 4), the Godot boundary
(milestone 5), and UI playback controls (milestone 6) — none of that is
touched here.

## DCC shell: Storage locations, Recent worlds, Data manager window (`DCC_SHELL_SPEC.md` §2.1/§2.4/§2.5/§9, 2026-08-19)

The owner asked specifically about the original DCC shell design's file/
folder-browsing menus and wanted them built now. `menus.gd`'s own honesty
convention (`_live()` wired to real behaviour, `_todo()` disabled with the
reason attached, never enabled-and-inert, never silently omitted) is
preserved exactly — this pass flips several long-standing `_todo()` items
live because they now have real behaviour, and leaves every genuine gap
disclosed with a reason verified against the actual code rather than
inherited from an older comment.

**Built:**

- **`shell/dcc_settings.gd` (`DccSettings`)** — a new persistence layer, the
  first thing in this shell that writes to `user://` (confirmed nothing
  existed by grepping `ConfigFile`/`user://`/`OS.get_user_data_dir` across
  `shell/` first). One `ConfigFile` at `user://cartalith_settings.cfg` holds:
  - The four storage roots from §2.1 (projects, tile atlas cache, asset
    packs, exports), defaulting to `OS.get_user_data_dir()`-relative paths
    (`Worlds`, `Cache/atlas`, `Packs`, `Exports`) rather than §2.1's own
    literal `~/Cartalith/Worlds` etc. — that prose is macOS-flavored (`~` as
    a home directory) and does not hold on Windows, where
    `get_user_data_dir()` is already the cross-platform-correct answer. Read
    as directive intent (four separate, sensible, per-purpose roots), not
    literal paths to reproduce, and said so in the file's own header.
  - The recent-projects list, capped at 10 (§2.1: "last 10 projects"),
    de-duplicating a re-opened path by moving it to the front rather than
    appending a second copy.
- **File ▸ Storage locations** (was `_todo`, "not configurable yet") — now
  opens a real read-only `AcceptDialog` listing the four current root paths.
- **File ▸ Change locations…** (new item; §2.1 lists it but it wasn't in the
  menu at all before this pass) — a modal with one `FileDialog`
  (`FILE_MODE_OPEN_DIR`) per root; picking a folder writes back to
  `DccSettings` immediately, no separate confirm step, and the readout
  updates in place. §2.1's own "moving the atlas root invalidates the cache"
  is handled by disclosure rather than by inventing cache logic: no tile
  atlas/cache concept exists anywhere in this port yet (Preferences ▸ Tiled
  LOD is itself still `_todo`), so the dialog says exactly that next to the
  atlas-cache row instead of pretending to invalidate something that isn't
  built.
- **File ▸ Show project on disk** (was `_todo`, "requires a project path") —
  now real. `DccApp` gained `current_project_path: String`, set by a new
  `_load_project()` helper that both `open_project_picker()`'s file-dialog
  callback and the new `open_recent_project()` funnel through, so there is
  exactly one place that remembers "what's open" and one place that updates
  the recent list. Reveals the folder via `OS.shell_show_in_file_manager`
  (Godot 4.4+, confirmed present in this project's 4.7.1) with an
  `OS.shell_open("file://...")` fallback for an older GDExtension build.
  Disabled with a tooltip until a project has actually been opened this
  session.
- **Preferences ▸ Application ▸ Storage locations…** (§2.5 lists this row;
  it wasn't in `_preferences()` at all before this pass) — added as a `_live`
  item that opens the exact same dialog `File ▸ Storage locations` does,
  matching §2.5's own "Same modal as File."
- **Data ▸ Recent worlds** (was `_todo`, "no project registry yet") — a real
  submenu, rebuilt on every `about_to_popup` (unlike the fixed-content
  `_quality_popup`/theme submenus already in this file, the recent list
  changes between menu opens, so the cached-once pattern those use would go
  stale). Each entry shows the file name with the full path as its tooltip
  and calls `open_recent_project()` on click.
- **`shell/data_manager_window.gd` (`DataManagerWindow`)** — §9's Data
  manager window, which did not exist in any form before this pass (the old
  comment in `menus.gd` said so plainly: "the whole menu is honest about
  that"). An `AcceptDialog` matching `world_data_window.gd`/
  `performance_window.gd`'s own construction convention, titled
  `⧉ DATA MANAGER`, subtitle "import · export · sources · conversion ·
  validation" verbatim. Structure per §9: a routes rail (the five §2.4
  groups, each with its listed sub-items as real rail buttons) and a route
  pane that rebuilds its content — breadcrumb plus body — on selection.
  `menus.gd`'s five Data-menu group items (Import/Export/Sources/Conversion/
  Validation) now open this window scoped to that group's first route via a
  new `DccApp.open_data_manager(group)`, instead of being disabled at the
  menu level.

  Route by route, what's real and what's disclosed:

  - **Import ▸ World Data (.zip · fields)** — real. Opens the exact same
    `.zip` project picker `File ▸ Open project…` already uses
    (`_host.open_project_picker()`), not a second implementation.
  - **Import ▸ Assets** — real as a routing shortcut, per §2.4's own table
    ("Assets (routes to the Assets menu)"): calls
    `_host.open_asset_pack_picker()` directly.
  - **Export ▸ World Data** — disclosed gap, **re-verified against the crate
    directly this pass** rather than trusted from an older comment:
    `cartalith-io` reads `.zip` saves (`load_save`) but its only
    `zip::ZipWriter` call lives inside its own `#[cfg(test)]` fixture builder
    (`tests::build_test_zip`), not production code. Confirmed by reading
    `cartalith-io/src/lib.rs` directly. A save writer is a separate, larger
    piece of work and stayed out of scope, per this dispatch's own
    instruction not to add Rust while a concurrent pass owned
    `cartalith-civ`/`cartalith-godot`.
  - **Import ▸ Maps/Heightmaps (PNG · TIFF)/GIS · GeoJSON**, **Export ▸
    Maps (image · tiles)**, **Export ▸ GIS / GeoJSON**, **Export ▸ Assets
    (pack .zip)**, **Sources** (External/Connected/Registry), **Conversion**
    (Coordinate systems/Format/Data transformation), **Validation** (Check
    data/Repair · Normalize) — all disclosed gaps, each with its own reason
    checked against the real workspace rather than assumed: no image/
    heightmap/GeoJSON import path exists anywhere (grepped `cartalith-io`,
    `cartalith-spatial`, `cartalith-assets`); no tile-pyramid/GIS export
    assembler exists even though `cartalith-terrain::tile_render` already
    draws per-tile PNGs for the unrelated Region-select tool; Export ▸ Assets
    routes to Assets ▸ Asset pack ▸ Build ▸ Export pack .zip…, which is
    itself `_todo` (needs the still-unbuilt asset-library window); no source
    registry exists; no CRS/format conversion exists (the engine works in
    one flat km projection throughout); and `load_save()` returns a plain
    bool with no warning collection anywhere a "Check data" count could read
    from.
  - The rail's own foot shows the real exports-root path
    (`DccSettings.storage_root("exports")`) and states plainly that no
    export has run yet, rather than inventing the `14:02 · 62 MB`-style
    placeholder §9's own mockup prose shows.

- `Data ▸ World data tables…` untouched — `world_data_window.gd`'s own doc
  comment already draws the line this pass respects: that window is the
  settlement/province/economy table browser, §9's related-but-distinct
  sibling, not the Data manager window this pass built.

**Not built, and why** (every one of these was `_todo` before this pass and
stays `_todo`, unchanged): Save project/Save as…/Autosave/Revert to last
save/Close project (all need a save writer or a project lifecycle, neither
exists); everything under Edit; the asset-library window and everything that
depends on it. None of these were in scope for this dispatch.

**Constraint honoured**: no Rust file touched. A separate, concurrent pass
was editing `cartalith-civ`/`cartalith-godot` for a stable-id field at the
same time (`git status` showed `cartalith-civ/src/timeline.rs` modified by
that work, not this one, throughout). Everything above is real against the
existing bridge surface — `bridge.load_save`, `open_asset_pack_picker` — with
no new `#[func]` needed anywhere.

**Verified**: every new/modified `.gd` file parses. A first `--headless
--path . --quit` boot failed with "Identifier ... not declared in the
current scope" for the two new `class_name` scripts (`DataManagerWindow`,
`DccSettings`) — a `--headless --path . --import` rescan (which regenerates
the global script-class cache) fixed it; worth remembering for the next new
`class_name` file added to this project, since it is not obvious from the
error message alone. A scripted, discarded smoke scene (`_smoke_data_mgr.gd`/
`.tscn`, deleted after this pass) exercised: the storage-root read/write
round-trip; `open_storage_locations`/`open_change_locations` opened and
closed back-to-back without a stale exclusive-window conflict (Godot raises
one if a second modal opens while an earlier one is still visible — the
harness had to close each dialog itself between calls, which is exactly what
a real user does by clicking OK, not a bug in the shipped code); recent-
projects dedup (re-opening an already-present path moves it to front,
`item_count` stays correct, asserted in-test); the Data manager window
opened on all five groups and every one of its 15 routes clicked in turn,
confirming the breadcrumb and pane content match each route's `kind`
(`live`/`route`/`gap`); `show_project_on_disk` no-ops cleanly with no
project open. The smoke run wrote real fake recent-project paths into the
actual `user://cartalith_settings.cfg` — noticed and the file deleted
afterward so a genuine future session starts clean rather than seeing test
junk in its own recent-worlds list. Headless Godot 4.7.1 boot
(`--headless --path godot-project --quit`) clean with the smoke files
removed, confirmed as the final step.

## Timeline milestone 2 — the proximity graph and Brandes betweenness, genuinely new Rust (`TIMELINE_SCOPE.md` milestone 2, 2026-08-19)

Fully self-contained per the scope doc — a places array + `cellKm` in,
adjacency/betweenness out, no dependency on milestone 1's population-ceiling
chain despite sharing `cartalith-civ::timeline`. Confirmed before writing
anything: no Brandes betweenness-centrality implementation existed anywhere
in this workspace (`_civNetworkMetrics`, the reference's other user of the
same algorithm, is not ported either) — this is new Rust, not a port of
something already half-there.

**Built** — two new functions in `cartalith-civ::timeline`:

- **`civ_proximity_adjacency`** (`_civProximityAdjacency`, reference lines
  24672-24683): a symmetric k-nearest-neighbour graph among settlement
  positions, real km via `cell_km`, world-wrap aware on the X seam. Takes
  bare `(x, y)` pairs rather than a `NamedSettlement`/domain struct — the
  reference itself only ever reads `.x`/`.y` off each place, and this
  matches the crate's existing "just positions" idiom
  (`civ_passed_settlements`'s `pts: &[(f64,f64)]`, `jp_resupply_reach`'s
  `pts`). `world_wrap`/`gw` are caller-supplied parameters (mirroring
  `civ_catchment_density_mean`'s own `world_wrap` from milestone 1 and
  `civ_passed_settlements`'s `world`), not read from a global, since
  `cartalith-civ` is stateless. Distance uses `js_hypot`/`js_min`
  (`cartalith-jsmath`) call-for-call against the reference's
  `Math.hypot`/`Math.min` — new code with no existing golden coverage to
  weigh against changing it, unlike the crate's other `.hypot()` sites
  (`lib.rs` ~5094-5098, left alone on that same reasoning in reverse).
  Returns one sorted, deduplicated neighbour list per node — a `Vec` playing
  the reference's `Set`'s dedup role; the sort doesn't change downstream
  betweenness, since Brandes' shortest-path counts are sums over the edge
  set, not order-sensitive on how a node's own list is arranged.
- **`civ_betweenness_from_adjacency`** (`_civBetweennessFromAdjacency`,
  reference lines 24687-24709): textbook Brandes (2001) unweighted
  betweenness centrality — the reference's own comment calls it "the same
  algorithm `_civNetworkMetrics` uses," confirmed by reading the ported
  lines directly rather than trusting the scope doc's summary; it is
  standard BFS-plus-dependency-accumulation, not a simplified/approximate
  variant. Returns **raw, un-normalised** betweenness, summed over both
  directions of every pair with no divide-by-2 for the graph being
  undirected — ported exactly as the reference computes it, not "corrected"
  to the more common convention. One parameter dropped, not silently: the
  reference's own `(n, adj)` signature carries `n` only because
  `_civCollapseStep` (milestone 3, unported) happens to have `settlements
  .length` sitting in scope when it calls this — `n` is always `adj.length`
  at the only real call site, so the port takes `adj: &[Vec<usize>]` alone
  and reads `adj.len()`, the same "drop a provably-redundant parameter"
  move milestone 1 already made for `_civCatchmentPop`'s dead `K` fallback.

**World-wrap distance**: grepped this crate first per the task's own
instruction rather than inventing a helper — `civ_passed_settlements`
(`lib.rs` ~9174) already has the exact pattern (`dx.min(gw as f64 - dx)`
gated by a caller-supplied `world: bool`), reused here as
`js_min(dx, gw - dx)` gated by `world_wrap`, matching the reference's own
`!!state.world` semantics call-for-call rather than the crate's several
always-on wrap sites (`lib.rs` lines 8414/8477/9473, which don't gate on a
flag because their own callers are always in a wrap-relevant context).

**Golden-verified against the real reference**
(`cartalith-civ/tests/golden_parity_timeline_graph.rs`, 6 tests): a Node
`vm.runInContext` harness (transient, not checked in, same convention as
milestone 1's own) sliced lines 24672-24709 verbatim into a context stubbed
with `state={world:false}`/`GW=<fixture width>`, ran every fixture below,
and every adjacency list and betweenness number below is the reference's
own output, not hand-computed and not re-derived independently except where
stated:

- **A 3-node hand-checkable path** (points at x=0,10,20, k=1, maxKm=15):
  adjacency `[1]/[0,2]/[1]`, raw betweenness `[0,2,0]` — also independently
  hand-derived via Brandes' own recurrence (not just asserted against the
  harness), confirming the "no divide-by-2" reading of the reference is
  correct and not a misreading of the ported lines.
- **A 5-node chain** (x=0,5,10,15,20, k=2): the path 0-1-2-3-4 with no
  shortcuts, betweenness `[0,0,8,0,0]` — the centre carries the whole load.
- **A world-wrap pair** (GW=100, points at x=2 and x=98, maxKm=20, k=1):
  empty graph without wrap (raw distance 96 exceeds 20), the pair becomes
  each other's sole neighbour with wrap on (wrap distance `min(96,4)=4`) —
  proving `world_wrap` changes which edges exist, not just that the flag is
  threaded through.
- **A world-wrap 4-cycle** (GW=100, points at x=0,25,50,75, k=2, maxKm=30):
  without wrap, the same positions form the path 0-1-2-3 (betweenness
  `[0,4,4,0]`); with wrap, the seam edge (75→0, wrap distance 25) closes a
  clean 4-cycle (betweenness ties at `[1,1,1,1]`, each node sitting on
  exactly one of the two tied shortest routes between the "opposite" pairs).
- **An 8-settlement real-scale fixture** (512x328 grid, 800 km width →
  `cellKm=1.5625`, the engine's own default extent; `maxLinkKm =
  cellKm*GW*0.5`, the reference's own default from `_civCollapseStep` line
  24794, reused here as the realistic default even though milestone 3 that
  formula belongs to isn't ported yet): `k=4` (the reference's own default)
  gives one connected graph where a bridging settlement and a cluster hub
  both carry the graph's entire betweenness load (`9` each, everyone else
  `0`); `k=2` on the identical positions splits into two disconnected
  components — exercising Brandes across multiple components in one call,
  confirming no node ever accrues cross-component betweenness.

**Verified**: `cargo build -p cartalith-godot` (the cdylib) and a headless
Godot 4.7.1 boot (`--headless --path godot-project --quit`) both clean.
`cargo test -p cartalith-civ`: all passing (21 `timeline` unit tests + 6 new
golden tests in `golden_parity_timeline_graph.rs`, 0 regressions elsewhere
in the crate). Clippy clean on every line touched by this milestone
(`cargo clippy -p cartalith-civ --no-deps -- -D warnings`, filtered to this
milestone's own line ranges) — two pre-existing `needless_range_loop`
findings in `lib.rs` (unrelated code this milestone never touches) and the
one pre-existing `1 * gw` readability finding milestone 1's own entry
already logged are all outside this milestone's additions, left as-is.

**Out of scope, per `TIMELINE_SCOPE.md`**: the collapse/recovery step
functions (milestone 3, which is what actually calls
`civ_proximity_adjacency`/`civ_betweenness_from_adjacency` for real), the
snapshot data model/orchestrator (milestone 4), the Godot boundary
(milestone 5), and UI playback controls (milestone 6) — none of that is
touched here. Nothing in this workspace calls either new function yet.

## Asset library window — closing the GUI gap `ASSET_LIBRARY_SCOPE.md` left open (`DCC_SHELL_SPEC.md` §2.3/§8, 2026-08-19)

`ASSET_LIBRARY_SCOPE.md` marked Phase 4's engine side (`cartalith-assets`)
complete on 2026-08-17 but explicitly carved the authoring UI out as later
work. `menus.gd`'s `_assets()` had every item except "Import asset pack
.zip…" disabled with "the asset-library window is not built yet." This pass
builds that window (`shell/asset_library_window.gd`, `AssetLibraryWindow`,
~830 lines, one file per this codebase's convention for a window this size)
and wires `⧉ Asset library` (⇧A) / `▦ Sprite sheet slicer` to it.

**A real discrepancy found reading the crate, not the mockup**: §8 describes
"24 families... Settlements, Terrain, Cartography, plus Collections."
`cartalith-assets/src/slots.rs` + `library.rs` define **eight** —
`ASSET_LIBRARY_SCOPE.md` §1 already said so ("eight families, seven of them
closed vocabularies") when Phase 4 shipped; this pass re-confirmed it by
reading both files directly and by a headless smoke run that opened every
one of the eight and asserted each grid populates with the real frozen slot
count (textures 7, biomes 15, terrains 13, icons 10, settlement 9, trait 7,
poi 10 — the Library's own 10-slot list, not the pack-import 8 — custom
0/open, 71 frozen slots total). §8's 24-family rail is the mockup's own
finer subdivision with no Rust type behind it; the window's family rail
lists the real eight, grouped the way the crate itself groups them
(`Family::is_texture()`, the `structures.*` trio: settlement/trait/poi).

**What's real vs. disclosed gap, and why it's mostly gap**:
`cartalith-godot/src/lib.rs` was grepped for every `#[func]` this pass.
Exactly two touch assets — `load_asset_pack(path) -> bool` and
`has_asset_pack() -> bool`. `pack.rs`'s `LoadedPack` (milestone 7) decodes
real pixels but only inside the render path, with no `#[func]` of its own —
there is no live `AssetDB` on the Godot side of the boundary at all.
Concretely:

- **Real**: the family list and each family's frozen slot ids (verbatim
  from `slots.rs`/`library.rs`'s invariant constants); each family's
  anchor/bake-size/variant metadata (`Family::anchor()`/`size()`/
  `is_multi()`); search and sort over that real list; multi-select in the
  slot grid (⇧-range, ⌘/Ctrl-add — client-side UI state, real interaction);
  the zoom control; the inspector's preview-background swatches
  (presentation only, the spec's own five swatch values); Import asset pack
  .zip… (the same `bridge.load_asset_pack` path the Assets menu already
  had); the pack-loaded status line (`bridge.has_asset_pack()`); the
  sprite-sheet slicer's image load, dimension readout, and its columns/
  rows/margin/spacing grid overlay (Godot's own `Image` loader plus
  arithmetic, no engine call needed) plus a real (sampled, honestly labelled
  "not an exact pixel scan") non-empty-cell count.
- **Disclosed gap** (a `_gap_button`/`_gap_kv_row` with a real reason as its
  tooltip, `menus.gd`'s own `_todo()` convention extended to a window body):
  per-slot fill state and thumbnails — the slot grid draws every slot as a
  checkerboard on principle, never guessing empty vs. filled; item variants,
  per-item scale, tags, pack metadata (name/author/license); batch edit
  (Tag/Collect/Rename/Duplicate/Delete); Validate/Clear library; Apply to
  map/Export pack .zip (no in-memory library session exists anywhere to
  compile or export — `load_asset_pack` loads a pack from disk for
  rendering only); the slicer's actual slice operation, its trim/skip
  toggles, and assign-to-family/fill-from (`raster.rs`/`manifest.rs`/
  `archive.rs` checked directly: whole-image decode/encode only, no
  sheet-splitting function anywhere in the crate).

**`menus.gd`**: `⧉ Asset library`/`▦ Sprite sheet slicer` promoted `_live`.
`Icon families ▸`/`Texture sets ▸` are now real submenus (split by
`Family::is_texture()`, matching the crate's own split) that open the
window scoped to one family — real, since scoping which family the rail
selects needs no engine query. `Apply library to map`/`Clear library…` stay
`_todo`, their reasons updated from "requires the window" (now built) to
the real finding above. `Import image…` stays `_todo`: the window exists,
but landing a loose image in an Unassigned-imports custom slot needs
`AssetDB::addCustomSlot`, which isn't exposed.

**Constraint honoured**: no Rust file touched (`git diff --stat --
cartalith-native/crates` empty at the end of this pass) — two other agents
were concurrently editing `cartalith-civ`/`cartalith-godot` Timeline work at
the same time. Everything above is real against the existing bridge surface
(`bridge.load_asset_pack`, `bridge.has_asset_pack`, `bridge.world_loaded`)
with no new `#[func]` needed anywhere; where a query genuinely doesn't
exist, it's disclosed rather than invented (no per-slot fill state was
approximated, no fabricated thumbnails, no fake pack metadata).

**Verified**: `asset_library_window.gd` parses under a headless `--import`
rescan (two bugs the first `--quit` boot caught and this pass fixed:
`DccWidgets.spacer()` doesn't exist — the real static is `DccTheme.spacer()`
— and an untyped `Dictionary.get()` result tripped this project's
warnings-as-errors inferred-Variant-type check, fixed by typing the
intermediate variable explicitly). A scripted, discarded smoke run
(`_smoke_asset_lib.gd`/`.tscn`, deleted after, along with the `.uid` file
Godot generates for a new script — noticed and removed so a real future
session starts clean) instantiated a real `DccApp`, opened the asset
library window, selected all eight families in turn and asserted each
grid's real child count against the frozen slot count above, exercised
single-click and ⇧-range multi-select on the icons family (confirmed the
selection dictionary held exactly the expected two uids after each step),
opened the sprite-sheet slicer modal, and read `has_asset_pack()` — all
without error. Headless Godot 4.7.1 boot (`--headless --path godot-project
--quit`) clean with the smoke files removed, confirmed as the final step.

## Timeline milestone 3 — the collapse and recovery step functions, the mechanistic core of the v0.85 stepper (`TIMELINE_SCOPE.md` milestone 3, 2026-08-19)

Depends on milestones 1 (population-ceiling chain, tier tables, stable
`tid`) and 2 (proximity graph, Brandes betweenness) — both already landed in
`cartalith-civ::timeline`. Per the scope doc's own framing, this is "the
core of the subsystem and the highest-value golden-parity target" because
it is fully deterministic (no RNG anywhere in the block, confirmed by
reading all five functions directly, not assumed from the reference's own
comment on a sibling function).

**Built** — five new functions plus a settlement-only place type in
`cartalith-civ::timeline`:

- **`CollapsePlace`**: `tid`/`x`/`y`/`kind`/`pop`/`fortified`/`ruins`. NOT
  `NamedSettlement` — the reference's `places` array mixes settlements with
  non-settlement POIs (filtered by `p.category==='settlement'` at the top
  of `_civCollapseStep`), and this port's `NamedSettlement` has no
  `traits`/`ruins` fields to carry the stepper's own new surface (a
  persistent `'fortified'` trait, a `ruins` flag). Rather than bolt two
  dead fields onto a struct every other subsystem in the crate also
  constructs, this milestone defines its own type — the same decoupling
  precedent milestone 2 set for `civ_proximity_adjacency`'s bare `(f64,
  f64)` positions. Because this port's place type is settlements-only,
  `civ_collapse_step`/`civ_recovery_growth_step` skip the reference's
  `p.category==='settlement'` filter-and-reassemble dance over a mixed
  array entirely — a disclosed structural simplification (every input
  entry already is a settlement, output preserves input order with
  failed/abandoned entries dropped, which is what the reference's own
  reassembly produces for the settlement subset), not a behavior change.
- **`CollapseCharacter`** (`_CIV_COLLAPSE_CHAR_WEIGHTS`/
  `_CIV_COLLAPSE_MIGRATION_BIAS`, reference lines 24653-24663): a closed
  Rust enum (`Trade`/`Disease`/`Conflict`/`Mixed`) in place of the
  reference's string-keyed lookup with a `mixed` fallback for an
  unrecognised key — the fallback is unreachable once the type only admits
  four real values.
- **`civ_settlement_stress`** (`_civSettlementStress`, 24713-24723):
  per-settlement stress in `[0,1]`, blending trade-dependency loss `L`
  (needs a caller-supplied `baseline_norm_b: Option<&HashMap<u64,f64>>` —
  `None`/absent, or a near-zero baseline, both give `L=0`, matching the
  reference's own "no loss to measure yet" comment), density/connectivity
  exposure `D` (half normalised betweenness, half population rank), and
  undefended-violence exposure `V` (`0.3` fortified, `1.0` not — fortified
  meaning the explicit trait OR currently an exchange tier, capped at
  `Capital` per milestone 1's metropolis decision), by the active
  character's weight triple.
- **`civ_mortality_migration_rates`** (`_civMortalityMigrationRates`,
  24726-24731): stress × severity × character → this step's ANNUAL
  excess-mortality fraction `m` and out-migration fraction `g`, using the
  rate-ceiling constants (`CIV_COLLAPSE_MAX_MORTALITY=0.15`,
  `CIV_COLLAPSE_MAX_MIGRATION=0.25`) and the character's migration bias.
- **`civ_gravity_migrate`** (`_civGravityMigrate`, 24738-24778):
  Zipf/Ravenstein (1946/1885) gravity-model migration redistribution —
  each origin's migrant pool split across every other place proportional
  to `headroom × fortifiedBonus / distance^β` (`CIV_MIGRATE_BETA=1.5`), up
  to 4 saturation-aware passes (a destination that caps mid-split has its
  clipped remainder re-offered to the still-open ones on the next pass),
  system-wide overflow becoming unplaced transit/diaspora loss.
  `cap_field`/`places` share an implicit same-length, same-index contract
  (documented, not re-validated — an index mismatch is a caller bug).
- **`civ_collapse_step`** (`_civCollapseStep`, 24785-24848): one
  `step_years`-long collapse step. Rebuilds the proximity graph +
  betweenness from `places`' own positions this step (milestone 2's
  functions, deliberately decoupled from any stale `ways` index),
  computes stress/mortality/migration per settlement, redistributes
  migrants via `civ_gravity_migrate`, re-derives tiers (see the demote-only
  finding below), drops anything under `CIV_ABANDON_FLOOR=20`. Returns the
  new places array, `{died, migrated, unplaced, failed}` stats, and
  `norm_b_by_tid` (every INPUT settlement's normalised betweenness this
  step, threaded forward as the next step's stress baseline — the reason
  milestone 1's stable `tid` field exists).
- **`civ_recovery_growth_step`** (`_civRecoveryGrowthStep`, 24852-24870):
  one `step_years`-long logistic (Verhulst 1838) regrowth step toward each
  settlement's own-kind catchment ceiling (`civ_settlement_population`,
  milestone 1), compounding `rate` internally `step_years` times (logistic
  growth isn't linear in time), re-deriving tiers upward (see the
  promote-only finding below).

**A real algorithmic surprise, verified against the actual reference lines
rather than trusted from `TIMELINE_SCOPE.md`'s own summary**: the scope doc
describes both step functions as "re-deriv[ing] tiers" without stating
direction. Reading the reference directly settles it exactly:
`_civCollapseStep` line 24826 computes `demoted =
_CIV_TIER_ORDER.indexOf(newKind) > _CIV_TIER_ORDER.indexOf(p.kind)` and
**only** updates `p.kind` inside the `demoted` branch — so a collapse step
can never promote a settlement's tier upward within that same step, even
if its post-mortality/migration population would nominally clear a higher
tier's floor. `_civRecoveryGrowthStep` (line 24863) is the exact mirror —
`promoted` gates the only branch that updates `q.kind`, so recovery can
never demote. Both are now named, tested invariants in `timeline.rs`'s own
unit tests
(`collapse_step_never_promotes_even_if_population_would_clear_a_higher_floor`
/ `recovery_growth_step_never_demotes_even_if_population_would_clear_a_lower_floor`),
not an assumption carried forward from the scope doc.

**`fortified` is sticky, `ruins` is not**: on demotion, a former exchange-
tier (`City`/`Capital`) nucleus gains both `ruins=true` and
`fortified=true` (reference: `p.ruins=true; ...traits.push('fortified')`);
on promotion back into an exchange tier, `ruins` clears (reference:
`delete q.ruins`) but `fortified` never does — the reference never removes
a trait once added, matching real-world "the old fortifications are still
there." Golden-verified both directions: a `ruins`+`fortified` Town with
high enough local density (its OWN Town-kind catchment ceiling clearing the
City floor) promotes into City over 100 years of 5%/yr regrowth, clearing
`ruins` while keeping `fortified`; the same shape at lower density promotes
only into Town — not an exchange tier — and `ruins` correctly stays set.

**The reference's dead `_K`-null fallback branches dropped**, extending
milestone 1's already-logged precedent (`civ_catchment_pop`'s dropped dead
`K` parameter): both `_civCollapseStep` (line 24802) and
`_civRecoveryGrowthStep` (line 24854) guard `currentCarryingCapacity` with
`typeof===  'function'`, which is always true in the real app (a hoisted
top-level declaration) — so the `_K?...:...` false branch
(`(p.pop||0)*1.05` / `(q.pop||1)*2`) never executes. This port's step
functions always compute the capacity-grounded ceiling via
`civ_settlement_population`; supplying real `dens`/`field` arrays is the
caller's responsibility, same contract milestone 1 already placed on that
function's own callers.

**Golden-verified against the real reference**
(`cartalith-civ/tests/golden_parity_timeline_collapse.rs`, 9 tests): a Node
`vm.runInContext` harness (transient, not checked in, same convention as
milestones 1-2) sliced the milestone-1 population-ceiling chain (lines
23407-23512) and the whole v0.85 stepper block (24614-24870) verbatim into
a context stubbed with `state`/`GW`/`GH`/`field` and
`currentAgrarianDensity`/`currentCarryingCapacity` returning caller-supplied
arrays. Fixtures shaped to reach real branches, per this project's own
working rule:

- **The abandonment floor, exactly**: a single isolated settlement (no
  destination to migrate to, so every migrant becomes unplaced diaspora
  loss) at three starting populations chosen so the post-step population
  lands at 19 (one below `CIV_ABANDON_FLOOR=20` — abandoned, `failed=1`),
  20 (exactly at the floor — survives, the check is strictly `<`), and 21
  (one above — survives).
- **A fortified-vs-unfortified pair at equal distance (400km) and equal
  headroom (500)**, isolating `civ_gravity_migrate` alone: the fortified
  destination receives exactly `1.5x` what the unfortified one does
  (`received[1]/received[2] == 1.5`, not merely "more") — proving
  `CIV_FORTIFIED_BONUS` actually changes destination weighting, in a
  single saturation-free pass so the ratio is clean.
- **The gravity model's multi-pass saturation genuinely engaging**: a near
  destination (headroom 50) and a far one (headroom 2000) — a single
  proportional pass would over-allocate to the near one, so the algorithm
  must cap it and re-offer the clipped remainder to the far one on a later
  pass (`received == [50, 950]`, `unplaced == 0`); a second fixture with
  both destinations' combined headroom (150) below the migrant pool (1000)
  proves the unplaced/diaspora-loss statistic actually accumulates what
  neither can absorb (`received == [50, 100]`, `unplaced == 850`).
- **All four collapse characters on one HUB/DENSE/UNDEFENDED/FORTRESS
  fixture** — at the raw-stress level first, with a caller-supplied
  synthetic `baseline_norm_b` (disclosed as such — not derived from an
  actual prior simulated step, which is milestone 4's orchestrator's job;
  `civ_settlement_stress`'s real signature takes an arbitrary map, so this
  is a legitimate test of that signature) giving only HUB a real `L`
  trade-loss term: trade ranks HUB most-stressed (0.951), disease
  *inverts* the ranking entirely — HUB drops to second-lowest (0.318) while
  the genuinely dense/connected settlements become most stressed (0.600,
  0.617) — the design doc's own central claim (trade and disease archetypes
  point opposite directions), proven in real numbers, not asserted. Then
  end-to-end through `civ_collapse_step` on the identical fixture (severity
  0.5, `t=0`, no baseline — `L=0` for everyone, isolating `D`/`V`):
  `failed` counts of **0** (trade — nobody fails when there's no loss yet
  to measure), **1** (disease — only the high-centrality bridge),
  **2** (conflict — both unfortified low-D settlements, sparing the
  fortified one despite its identical low-D profile), **1** (mixed, same
  survivor set as disease but different exact numbers) — with exact
  `died`/`migrated`/`unplaced` stats and exact surviving tids/populations
  for each character.
- **Recovery promoting a `ruins`-flagged settlement into an exchange
  tier**, contrasted with one promoting into a non-exchange tier, as
  detailed above.

Plus 7 new unit tests in `timeline.rs` itself (character weights each sum
to 1; an unassigned `tid=0` never performs a baseline lookup even when a
populated map is supplied — mirrors the reference's own `place.tid!=null`
guard; `civ_collapse_step` on an empty places array is a true no-op;
`civ_gravity_migrate` is a true no-op when nobody migrates; both the
demote-only and promote-only invariants above).

**Verified**: `cargo build -p cartalith-godot` (the cdylib) and a headless
Godot 4.7.1 boot (`--headless --path godot-project --quit`) both clean
(exit 0, no errors). `cargo test -p cartalith-civ`: all passing (303 lib
tests including 28 `timeline` unit tests, 9 new golden tests in
`golden_parity_timeline_collapse.rs`, 0 regressions elsewhere in the
crate). Clippy clean (`cargo clippy -p cartalith-civ --all-targets`) — two
`neg_cmp_op_on_partial_ord` findings in `civ_gravity_migrate` kept and
narrowly `#[allow]`ed with a comment rather than rewritten: `!(remaining >
0.0)` is not the same as `remaining <= 0.0` when `remaining` can be NaN
(the reference's own `!(remaining>0)` falsy-check is NaN-inclusive, and a
naive `<=` rewrite would silently drop that reading), per
`cartalith-rust-conventions`'s own NaN-comparison-awareness rule. One
collateral-damage note: an initial `cargo fmt -p cartalith-civ` run
reformatted every file in the crate (6,000+ lines of churn in `lib.rs`
alone, from a repo that evidently doesn't keep the whole crate
`rustfmt`-clean) — reverted everything outside `timeline.rs`/this
milestone's own new test file before committing, so this commit's diff is
exactly this milestone's own work.

**Out of scope, per `TIMELINE_SCOPE.md`**: `_civSimulateTimeline` (the pure
orchestrator) and `_civRunCollapseSimulation` (the impure wiring) —
milestone 4's job, not this one's. The snapshot data model (milestone 4),
the Godot boundary (milestone 5), and UI playback controls (milestone 6)
are equally untouched. Nothing in this workspace calls any of this
milestone's five new functions yet.

## Timeline milestone 4 — snapshot data model + orchestrator (`TIMELINE_SCOPE.md` milestone 4, 2026-08-19)

Two pieces, split exactly along the crate boundary `TIMELINE_SCOPE.md` §5
predicted: the pure orchestrator and snapshot/diff logic in
`cartalith-civ::timeline`, and the real mutable state (a `Vec` of year
snapshots plus the active-year cursor) on `CivData` in
`cartalith-godot/src/lib.rs`, with thin methods calling into the pure
functions — `cartalith-civ` stays stateless (`ARCHITECTURE.md`), matching
`journey_bridge.rs`'s own established precedent for a Godot-side crate
owning mutable state over an engine crate's pure functions.

**`civ_assign_tid`/`civ_resync_next_tid` were already half-satisfied by
milestone 1** — exactly as the task brief predicted checking for.
Milestone 1 already built both as pure functions in `timeline.rs` (eager
assignment at placement time, not the reference's lazy first-touch — logged
there already). What milestone 1 *couldn't* build yet was the timeline-
history half of `_civResyncNextTid` (reference lines 20569-20571 scan every
`civTimeline` entry's own `places`/`ways`, not just the live arrays) —
`TimelineSnapshot` didn't exist until this milestone. Rather than widen
`civ_resync_next_tid`'s signature (real callers/tests already depend on its
two-argument shape, and a fresh `compute_civilisation` run legitimately has
no timeline to scan), this milestone adds a sibling,
**`civ_resync_next_tid_with_timeline`**, folding in every snapshot's
settlements/ways on top of the same live-array scan. No caller wires it in
yet (nothing in this port reloads/rebuilds settlement lists out from under
the counter today — the same gap milestone 1's own doc comment already
flagged) — it exists ready for whichever future pass adds save-format
persistence or a "resync after external edit" path.

**Built in `cartalith-civ::timeline`** (pure, stateless, takes/returns
explicit values — no crate-level mutable state):

- **`TimelineSnapshot { year, territory, settlements, ways }`** — one
  recorded year. Deliberately NOT the reference's own `{...p}` loosely-typed
  spread, and deliberately missing `provinces`/`trade_balances`/
  `explanations` — confirmed by reading `civSnapshotSave` directly (lines
  20596-20604) that the reference's own snapshot never captured them either.
  `territory` is a dense `Vec<i32>` clone, not the reference's sparse
  `[i, factionId, ...]` pair encoding (reference line 20598) — that
  encoding exists to shrink the reference's own save-file payload, a
  concern this in-memory struct doesn't share (`TIMELINE_SCOPE.md` §9
  defers persistence entirely); disclosed simplification, not a silent one.
- **`civ_year_diff`** (`_civYearDiff`, 20580-20595): diffs a year's
  snapshot against the chronologically-previous recorded one by tid set —
  `present`/`removed`/`added`, as `BTreeSet<u64>`. No memoization cache
  (the reference's own `_civYearDiffCacheYear`/`_civYearDiffCache` needs an
  explicit invalidation call at five separate sites to stay correct; a
  `BTreeSet` diff over two small vecs is cheap enough not to need one) — a
  caller wanting memoization can add it at the `CivData` boundary.
- **`civ_snapshot_save`**/**`civ_snapshot_load`** (`civSnapshotSave`/
  `civSnapshotLoad`, 20596-20614): upsert-and-sort into a `&mut
  Vec<TimelineSnapshot>`; restore territory only, filling `0` first
  (reference: `terr.fill(0)`) — never touches settlements/ways, the
  reference's own emphasis (lines 20559-20561) that those stay the single
  always-current, always-editable arrays.
- **`civ_simulate_timeline`** (`_civSimulateTimeline`, 24875-24892): the
  pure orchestrator. Runs `opts.steps` collapse-or-recovery steps from a
  starting `CollapsePlace` array (milestone 3's `civ_collapse_step`/
  `civ_recovery_growth_step`), returning one `TimelineStepSnapshot{places,
  stats}` per step. Never touches any timeline/live state — `SimulateMode`/
  `SimulateWorldParams`/`SimulateTimelineOpts` bundle the reference's own
  `opts` bag plus the seven `dens`/`field`/`gw`/`gh`/`sea`/`world_wrap`/
  `map_width_km` world-sampling parameters both step functions need, since
  those are stable for a whole simulation run rather than re-derived per
  step. `baseline_norm_b` is captured ONLY at step `t==0` and reused
  unchanged for every later step — read directly off the reference's
  `if(t===0) baselineNormB=r.normBByTid...` (conditioned on `t` inside the
  loop, not reassigned every iteration), a detail easy to get wrong by
  assuming each step re-baselines against its own predecessor.

**Built on `CivData` in `cartalith-godot/src/lib.rs`** — two new fields
(`timeline: Vec<TimelineSnapshot>`, `year: i64`, both reset to
empty/`0` in `compute_civilisation`, matching the reference's own
`generate()` wrapper clearing `civTerritory`/`civTimeline`/`civYear` on
every fresh procedural generation) and an `impl CivData` block of plain
methods — no `#[func]`, no `Variant`, no godot-visible surface, per this
milestone's own explicit scope:

- **`civ_goto_year`** (`civGotoYear`, 20615-20617, minus the UI rebuild
  call — milestone 6): sets the cursor, restores territory via
  `civ_snapshot_load`.
- **`civ_add_year`** (`civAddYear`, 20618-20634): the empty-timeline seed
  case (reference's own v0.62 fix, avoiding a phantom "0 AD" entry at the
  init `civYear=0`), the "snapshot the currently-active year from live
  state first, so it's never lost" step, the "don't clobber an already-
  recorded year" bail, and the "carry forward from the nearest earlier
  recorded year" case — all four read directly off the reference, not
  inferred from the scope doc's summary.
- **`civ_remove_year`** (`civRemoveYear`, 20635-20641, minus the UI
  rebuild): falls back to the earliest remaining year (`self.timeline`
  stays sorted by construction, so `.first()` suffices), or year `0` if
  none remain (reference: `next?next.year:0`).
- **`civ_year_diff`**: thin passthrough to the pure function above.

**A deliberate deviation, logged per `TIMELINE_SCOPE.md` §9's own
in-flight decision, not silently matched**: `civ_add_year` caps recorded
years at `TIMELINE_MAX_YEARS = 2000` (a no-op past the cap — the
currently-active year's live state is already safely snapshotted before
the check runs in every code path, so refusing to grow further never loses
data). The reference stores an unbounded `civTimeline` with no eviction.

**Golden-verified against the real reference**
(`cartalith-civ/tests/golden_parity_timeline_orchestrator.rs`, 4 tests): a
Node `vm.runInContext` harness (transient, not checked in, same convention
as milestones 1-3) sliced the population-ceiling chain (23407-23434 +
23461-23512, skipping the real cached `currentAgrarianDensity` body in
between since the harness stubs that function directly) and the v0.85
stepper block **plus the orchestrator itself** (24614-24892) into a context
stubbed the same way milestone 3's own harness was. One early false start
worth recording: the first harness run produced all-zero stats for every
step — `_civCollapseStep` filters `places` down to
`p.category==='settlement'` before doing anything, and the harness's first
place fixtures didn't set that field, so every step silently ran on zero
settlements (`n=0`, early-return branch). Caught by inspecting the
suspiciously-uniform zero output, not by a passing-but-wrong test — exactly
the "watch for silently-empty golden output" failure mode this repo's own
root `CLAUDE.md` names.

Fixtures, reusing (not re-deriving) milestone 3's own already-verified
HUB/DENSE/UNDEFENDED/FORTRESS base places and ruins+fortified-Town recovery
fixture, since this milestone's job is proving the ORCHESTRATOR's
step-to-step wiring, not the step math again:

- **Collapse, `mixed` character, 3 steps**: proves `baseline_norm_b`
  threading — step 0's `died`/`unplaced`/`failed` (365/368/1) match
  milestone 3's own single-step golden numbers for this exact fixture
  exactly, and steps 1-2's own `died`/`migrated`/`unplaced`/`failed`
  (122/17/107/0, then 49/21/30/0) come from the SAME t=0 baseline reused,
  not re-captured each step, which the harness's real output confirms.
- **Collapse, `trade` character, 2 steps, a different severity (0.8)**: an
  independent second configuration.
- **Recovery, 2 steps of 50 years each**: final pop/kind (6211/City) match
  `golden_parity_timeline_collapse.rs`'s own single-100-year-step number
  for the identical starting fixture *exactly* — confirming the
  orchestrator's step-to-step `cur=r.places` chaining is equivalent to
  running the same total duration in one step, not an assumption.
- **`opts.steps` omitted (`0`)**: clamps to exactly 1 step
  (`Math.max(1,opts.steps||1)`), matching milestone 3's own `conflict`
  character single-step numbers verbatim.

Plus 6 new unit tests in `timeline.rs` (snapshot save upserts and re-sorts;
snapshot load restores territory only and fills `0` for an unrecorded
year; year-diff tid-vs-name disambiguation — `TIMELINE_SCOPE.md` §7 success
criterion 3's own named case, a settlement that disappears and a
same-name/same-position DIFFERENT settlement that appears in its place,
tid correctly showing removed+added rather than "persisted"; year-diff
against the earliest recorded year has no previous, so every present tid
reads as "added" — read directly off the reference's `tidsOf(null)`
behavior, not assumed empty; year-diff for an unrecorded year is empty;
`civ_resync_next_tid_with_timeline` folding in snapshot history the
milestone-1 version is blind to) and 8 new unit tests in
`cartalith-godot/src/lib.rs`'s own `civ_timeline_tests` module covering
`TIMELINE_SCOPE.md` §7 success criterion 2 directly: adding a year never
loses the currently-active year's live edits, `civ_goto_year` never
mutates settlements/ways (a fixture where the live array is deliberately
diverged from BOTH recorded snapshots, so a bug that restored settlements
from a snapshot would be caught even though the two snapshots'
`settlements` happen to agree with each other), and `civ_remove_year`
falls back to the earliest remaining year or `0`.

**Verified**: `cargo build -p cartalith-godot` (the cdylib) and a headless
Godot 4.7.1 boot (`--headless --path godot-project --quit`) both clean
(exit 0, no errors, extension loads). `cargo test -p cartalith-civ`: 309
lib tests (up from 303; +6 this milestone) plus the new 4-test golden file,
0 regressions. `cargo test -p cartalith-godot`: 178 lib tests (up from
170; +8 this milestone), 0 regressions. `cargo clippy -p cartalith-civ -p
cartalith-godot --all-targets`: clean — every warning shown is pre-existing
in files this milestone didn't touch (`golden_parity_road_network.rs`
excessive-precision literals, two pre-existing `identity_op` findings in
test index arithmetic, two pre-existing `too_many_arguments` findings).

**Out of scope, per `TIMELINE_SCOPE.md`**: the Godot boundary
(`timeline_bridge.rs`, `#[func]` surface — milestone 5) and UI playback
controls (milestone 6) are both untouched; nothing in this workspace calls
`civ_add_year`/`civ_remove_year`/`civ_year_diff`/`civ_simulate_timeline`
yet outside this milestone's own tests. Save-format persistence of
`civTimeline`/`civYear` remains deferred, as `TIMELINE_SCOPE.md` §9 already
recorded.

## Timeline milestone 5 — the Godot boundary (`TIMELINE_SCOPE.md` milestone 5, 2026-08-19)

New `cartalith-godot/src/timeline_bridge.rs` module, godot-free (`cargo test
-p cartalith-godot --lib` runs its 11 tests with no Godot runtime), following
`journey_bridge.rs`'s exact precedent: the module owns the sim-panel request
parser and the impure wiring; `lib.rs` owns the thin `Variant`<->Rust
conversion and the `#[func]` surface, in a new `#[godot_api(secondary)]`
block (`WorldGen` has a `Base<RefCounted>` field, so only the crate's first
`#[godot_api] impl WorldGen` block may omit `secondary`).

**`#[func]` surface added to `WorldGen`** (7 methods — the 5 the task brief
named, plus 2 small getters milestone 6 will want):

- **`civ_add_year(year)`/`civ_goto_year(year)`/`civ_remove_year(year)`** —
  thin wrappers over `CivData`'s already-built milestone-4 methods. No new
  logic; a no-op before any `generate()`.
- **`get_civ_year()`**/**`get_civ_timeline_years()`** — the active cursor and
  the sorted list of recorded years, so a future dock can build the pill
  list/slider without a per-year round trip.
- **`civ_year_diff(year)`** — `{"present"/"removed"/"added":
  PackedInt64Array}` of tids, thin passthrough to `CivData::civ_year_diff`
  (milestone 4).
- **`civ_run_collapse_simulation(request)`** — the one real new wiring this
  milestone adds: `timeline_bridge::run_collapse_simulation`, a straight port
  of `_civRunCollapseSimulation`'s impure half (reference lines 24896-24950).

**Request/response shape** (`timeline_bridge::CollapseSimRequest`, parsed
from a flat `Dictionary` the same way `journey_bridge::plan_from_pairs`
parses the Journey Planner's form — a partial request is legal, an unknown or
wrong-typed key is reported in `rejected` rather than silently ignored):
`mode` (`"collapse"`/`"recovery"`), `character`
(`"mixed"`/`"trade"`/`"disease"`/`"conflict"`), `severity`/`rate` (already
real units — `[0,1]`/fraction-per-year — not raw 0-100/1-30 slider ticks,
since this port has no slider yet to divide; a future milestone-6 UI does
that division on its own side), `start_year`/`duration`/`step_years`, and
`confirm_overwrite`.

**The warn-before-overwrite case** (reference lines 24910-24911, a blocking
`confirm()` dialog): this boundary can't block on one, so instead of asking a
question mid-call, a first call whose simulated years would land on
already-recorded entries returns `{"ok": false, "needs_confirm": true,
"clobber_years": [...]}` **without writing anything** — the caller re-sends
the identical request with `confirm_overwrite: true` to proceed. Checked
first: this port has no prior "confirm-before-overwrite" case to match
(`grep -i confirm` over every bridge module turned up only unrelated hits —
label-editor confirm/cancel, a doc-comment "confirm the extension loaded") —
so this is new design, following the same "a response field the caller
checks" shape `jp_compute`'s own `rejected` array already establishes for
"something needs the caller's attention, not a hard failure."

**The anchor/carry-forward behavior, verified against the real reference
before trusting the task brief's own summary of it** (reference lines
24915-24925): the currently-active year is snapshotted from live state first
(never lost), then a "before" frame is written at the simulation's own
`start_year` **only if none exists there yet** — and because that write
always happens before the anchor search runs, `anchor=civTimeline.filter(y
<= startYear)...[0]` always resolves to exactly the `start_year` entry.
Territory/ways for every simulated year come from that one anchor entry,
copied unchanged — "collapse doesn't redraw political borders," confirmed
verbatim in the reference's own comment, not just the task brief's
paraphrase of it. One real, testable consequence: if `start_year` already
carries a manually-authored (or previously-simulated) territory, a new
simulation run preserves it rather than silently replacing it with whatever
the live grid currently shows — the whole point of the guard, and the thing
`territory_and_ways_carry_forward_unchanged_from_the_nearest_prior_entry`
actually exercises (an earlier draft of that test picked an anchor year
*before* `start_year`, which is unreachable in practice: the write above
guarantees an entry sits at `start_year` itself before the search runs, so
`anchor` can only ever be "the pre-existing start-year entry" or "the live
state just captured there" — caught by the test itself failing, not assumed
correct).

**A disclosed, out-of-scope gap found while wiring this up**:
`CollapsePlace` (milestone 3) carries `fortified`/`ruins`, but
`TimelineSnapshot` stores `Vec<NamedSettlement>` (milestone 4), and
`NamedSettlement` (Phase 2, predating Timeline) has neither field. Extending
`NamedSettlement` would ripple into every other subsystem that constructs one
— real work, and explicitly out of this milestone's own scope ("do NOT touch
milestones 1-4's already-committed functions"). `fortified`/`ruins` stay
correctly threaded through every step *within* one simulation run (the
orchestrator chains `Vec<CollapsePlace>`, never touching `NamedSettlement`
until the final per-step write), but do not survive into what gets stored
for later scrubbing/redisplay. Inert today — nothing reads it, milestone 6
isn't built — flagged in `timeline_bridge.rs`'s own module doc for whichever
future milestone extends `NamedSettlement`/`TimelineSnapshot` to close it.

**One additive change to already-committed code, not a refactor of it**:
`CivData` gains a `dens: Vec<f32>` field (`currentAgrarianDensity()`'s
per-cell output, `cartalith_civ::timeline::civ_current_agrarian_density`),
computed once in `compute_civilisation` from `carrying_cap`/`water_access`/
`biome` — locals that function already builds for suitability scoring — and
retained past its own return, exactly the "it was already real,
already-computed data this function held anyway" reasoning `water_bodies`
was kept for. Without it, `civ_run_collapse_simulation` would have had to
re-run the soil/water-access/biome sub-pipeline on every simulate call.
Milestone 1-4 functions/methods themselves are untouched — only `CivData`'s
struct (already extended by milestones 1 and 4) and `compute_civilisation`
(Phase 2 infrastructure, not a Timeline milestone deliverable) gained one
more field/computation, the same kind of growth those milestones already
did.

**GDScript**: all 7 methods wired into `godot-project/shell/engine_bridge.gd`
with the `has_method()` guard this project uses everywhere for
GDExtension-version-skew safety, ready for milestone 6 to consume — no UI
built here.

**Verified**: `cargo test -p cartalith-godot --lib` — 189 lib tests (up from
178; +11 this milestone), 0 regressions, no Godot runtime involved for any of
them. `cargo test -p cartalith-civ --lib` — 309 tests, 0 regressions (this
milestone calls but does not modify `cartalith-civ`). `cargo build -p
cartalith-godot` (the cdylib, not just `cargo test`) and a headless Godot
4.7.1 boot (`--headless --path godot-project --quit`) both clean. `godot
--headless --check-only --script shell/engine_bridge.gd` — the GDScript
addition parses with no errors. `cargo clippy -p cartalith-godot
--all-targets`: clean — every warning shown is pre-existing in files this
milestone didn't touch.

**Out of scope, per `TIMELINE_SCOPE.md`**: milestone 6 (UI playback
controls) is untouched — nothing in the actual Godot shell calls any of
these 7 methods yet. Save-format persistence of `civTimeline`/`civYear`
remains deferred, as `TIMELINE_SCOPE.md` §9 already recorded. The
`fortified`/`ruins` snapshot gap above is disclosed, not fixed, in this pass.

## Timeline milestone 6 — UI playback controls, the shell (`TIMELINE_SCOPE.md` milestone 6, 2026-08-19)

Closes `TIMELINE_SCOPE.md`'s milestone list. GDScript only — no Rust file
touched (a separate, concurrent, unrelated pass had `cartalith-civ/src/
lib.rs` and `cartalith-godot/src/lib.rs` modified-but-uncommitted in the
working tree throughout this dispatch; left untouched and not staged into
this milestone's commit).

**Where it lives, and why not where the brief's own precedent pointed**: a
new sixth `DccWidgets.category()` — "Timeline" — in `civilization_workspace
.gd`'s left dock, alongside the file's existing Settlements/Population/
Economy/Politics/Culture categories, rather than a new `right_dock.gd` CTX_*
context (`CTX_SCULPT`/`CTX_JOURNEY` were the dispatch's own suggested
precedent). Both of those are driven by an actual map TOOL arming
(`app.tool_armed`) tied to viewport interaction; Timeline has no map click of
its own — add year / goto year / run simulation are pure state edits, the
same shape this file's own Settlements/Population/Politics categories
already are (click a row, act, done) — so `DccWidgets.category()`, this
file's own established vocabulary (used five times before this milestone),
is the correctly-scoped precedent instead. Also deliberately **not** wired
into `dcc_shell.gd`'s own `timeline_bar`/`timeline_row` — the empty bottom
strip `DCC_CONTROL_INDEX.md` §10 reserves and shows for civilization/
infrastructure (`app.gd`'s `_on_workspace_changed`) — per `TIMELINE_SCOPE.md`
§4's own instruction to default to a dedicated panel rather than risk
building into the still-undecided six-toggle continuous-simulation region;
that bar is one fixed-height row with no room for everything below, and §10
never says whether its own scrub track means this discrete `civTimeline` or
the other feature. Left untouched.

**Built, all six of the brief's own numbered pieces**:

1. **Years pill row + Add year** — one pill per `get_civ_timeline_years()`
   entry, `_civFormatYear` ported verbatim (`-1200 → "1200 BC"`, `450 →
   "450 AD"`) as `_tl_format_year`; clicking a pill calls `civ_goto_year`,
   its own ✕ calls `civ_remove_year`. A year-value `SpinBox` (default `100`,
   the reference's own `#civTlYear` default) plus an Add year button calls
   `civ_add_year`.
2. **Scrub track** — real time-scale, matching the reference's v0.91
   behavior verified against the reference directly (`_civWireYearSlider`,
   lines 26451-26474): min/max are the actual lowest/highest recorded years,
   not a snapshot-count index, and dragging snaps to the nearest recorded
   year every tick (`_tl_nearest_year`), with a full pill-row/active-year
   refresh on release (`drag_ended`) rather than every tick, mirroring the
   reference's own `oninput`-snaps/`onchange`-rebuilds split.
3. **Playback transport** — Play/Pause via a real `Timer` (`wait_time =
   1.2`, the reference's own 1200ms interval verbatim) advancing to the next
   recorded year and auto-stopping at the end (`_civTlStartPlay`/
   `_civTlStopPlay`, lines 26424-26440, ported behavior-for-behavior), plus
   a Step button (this milestone's own addition — the brief asked for
   Play/Pause/**Step**, which the reference's markup doesn't have as a
   separate control) that advances one recorded year without arming the
   timer.
4. **The three filter checkboxes** — exist-only/ghost/highlight toggle real
   `bool` state and the panel calls `civ_year_diff()` for a live present/
   removed/added tid-set count readout underneath them (verified against a
   real multi-year, multi-settlement fixture in the headless smoke run
   below — real, non-zero, changing numbers). **They do not filter/ghost/
   highlight individual settlement pins on the map** — a real, disclosed
   gap, not a faked one: `get_settlements()` (`lib.rs`) carries no `tid`
   field, even though `NamedSettlement` gained one at the Rust level in
   milestone 1 (confirmed by reading `get_settlements()`'s own `#[func]`
   doc comment and its dict-building body directly — no `"tid"` key
   anywhere in `lib.rs`, `map_overlay.gd`'s `_settlements` array carries no
   tid either). Nothing on the Godot side can therefore tell which drawn pin
   any of `civ_year_diff()`'s tids refers to; matching by name/position
   would be exactly the kind of fake disambiguation `TIMELINE_SCOPE.md`'s
   own module doc warns tid exists to avoid (a same-named settlement can be
   a genuinely different object). Closing this needs a Rust-side change —
   `get_settlements()` (and/or a new per-year snapshot-settlements getter,
   since `civ_goto_year` deliberately never touches live `settlements`
   either) exposing `tid` — explicitly out of this GDScript-only milestone's
   own constraint ("no Rust files"). Stated in-product (a note under the
   checkboxes) and here, not silently worked around.
5. **The collapse-simulation form** — Mode (collapse/recovery) · Character
   (mixed/trade/disease/conflict, collapse-only) · Severity (0-100%,
   collapse-only) or Regrowth rate (0.1-3.0%/yr, recovery-only, reproducing
   the reference's own tenths-of-percent slider granularity in percent units
   directly rather than porting its raw 1-30 tick scale) · Start year ·
   Duration · Step years · a primary Simulate button calling
   `civ_run_collapse_simulation`, and an output note reproducing the
   reference's own `civSimOut` wording per mode (died/migrated/unplaced/
   failed for collapse, settlements-remain-and-regrow for recovery). The
   `needs_confirm`/`clobber_years` response is handled with a real
   `ConfirmationDialog` (title, the reference's own overwrite-count/year-list
   wording, an "Overwrite" OK button, Cancel/close as No) added to `app` —
   this shell's own `AcceptDialog`-on-`app`-root convention
   (`app.gd`'s `open_storage_locations`/`open_credits`, `cartography_
   workspace.gd`'s `_prompt_label_name`), extended to `ConfirmationDialog`
   since none of those needed a real two-way choice before. Confirming
   re-sends the identical request with `confirm_overwrite: true`.
6. **Gating verified against the reference, not assumed**: `_civBuildExplore
   TimelineUI` (line 26481) hides the slider/playback row unless
   `civTimeline.length>1`; the scrub and playback sections here both early-
   return under the same `years.size() < 2` guard, and the years section
   itself (Add year) never gates — matching the reference's own comment at
   line 1885-1887 ("adding the first year has to start somewhere").

**Territory-view refresh, found while wiring, not in the brief's own
checklist**: every call that moves the timeline cursor (`civ_goto_year`, via
a pill click, a scrub drag, Step, a playback tick, or the end-of-simulation
`civ_goto_year(end_year)` `civ_run_collapse_simulation` performs internally)
reloads the engine's `territory` grid but does not itself touch the rendered
texture. `_tl_refresh_territory_view()` writes `app.viewport.territory_view
.texture = bridge.territory_texture()` after every one of those calls — the
same direct-field-write-without-`ViewportHost.refresh()`'s-camera-reset
pattern `_commit_territory()`/`_refresh_civ_data()` in this same file already
use, for the same reason (a camera snap on every scrub tick would be
disorienting).

**Verified**: every modified file parses (`godot --headless --check-only
--script` conceptually covered by the full boot below, since this pass
touched only one file). A first headless boot (`--headless --path
godot-project --quit`) was clean before any new code, confirming the
baseline. After the change: a second `--headless --path godot-project
--import` rescan (no new `class_name` added this pass, so not strictly
required, but run anyway per this project's own "worth remembering" note)
and a further `--headless --path godot-project --quit` boot, both clean, no
parse or registration errors. **A scripted, discarded smoke scene**
(`_smoke_timeline.gd`/`.tscn`, deleted after this pass, same precedent as
the Data-manager and Journey-planner milestones' own smoke runs) instanced
the real `app.tscn`, generated a real 160×160/300 km world, and drove the
new code paths directly (the same handlers the buttons call, not simulated
clicks): `civ_add_year(50)` then `civ_add_year(150)` — years list becomes
`[50, 150]`, active year tracks each add; `civ_goto_year(50)` — active year
updates, pill row builds 10 children with no error; `civ_year_diff(150)` —
returned real non-zero counts (`141 present / 0 removed / 0 added`, a real
settlement roster existing by year 150 with nothing to compare against
before it); `Step` — advances to 150; a collapse simulation (character
conflict, severity 0.8, start 150, duration 20, step 10) against the real
generated settlements — real output: `"Simulated 2 steps (10 yr each), 150
AD -> 170 AD. 9 settlements remain. 57111 died, 15086 migrated (53494 lost
in transit/diaspora), 125 settlements failed/abandoned."`, and two new
timeline years (160, 170) appended; **re-running the identical request
correctly returned `needs_confirm` and left `_tl_sim_out` unchanged** (the
real `ConfirmationDialog` was constructed and popped with no error headless
— exercising that codepath, not just the data layer), then **re-running with
`confirm_overwrite: true` succeeded and overwrote in place**; Play/Stop
toggled `_tl_playing` correctly; `civ_remove_year(50)` left `[150, 160,
170]`. No crash anywhere in the sequence. Smoke files deleted before this
commit — `git status` shows only `civilization_workspace.gd` plus this
milestone's doc updates from this pass.

**Out of scope, per `TIMELINE_SCOPE.md` §4/§6**: the DCC shell's own
six-toggle continuous-simulation Timeline region (`DCC_CONTROL_INDEX.md`
§10's Climate/Population/Economy/Politics/Infrastructure/Warfare toggles)
and Warfare — untouched, still the owner's open product decision.
Save-format persistence of `civTimeline`/`civYear` remains deferred per §9.
The settlement-pin tid gap (item 4 above) is a real, disclosed, Rust-side
follow-up, not fixed here.

## Travel Library milestone 1 — data model, stock content, validation, and real `jp_plan` wiring (`TRAVEL_LIBRARY_SPEC.md`, 2026-08-19)

The Rust half of `Data ▸ Travel library…`: a genuinely new, owner-supplied
addition to the DCC shell (`TRAVEL_LIBRARY_SPEC.md`'s own opening line —
"not part of `DCC_SHELL_SPEC.md` and does not exist in Cartalith Gen1
v2.10"). No golden-parity target exists for any of it. The paired GDScript
window (`design/Journey Planner DCC.dc.html` sections `2a`/`2b`) is a
separate, later dispatch; nothing GDScript-visible is added here by design.

**Where it lives, and why:** split exactly along `ARCHITECTURE.md`'s
stateless-`cartalith-civ`/stateful-`cartalith-godot` line, the same split
every other subsystem in this port uses.

- `cartalith-civ/src/travel_library.rs` (new, godot-free, `pub mod
  travel_library` off the crate root): the four §3 definition types
  (`AnimalDef`/`VehicleDef`/`VesselDef`/`PartyPreset`), §4's three-state
  `ValidationState` (ok/incomplete/conflicting) with a `validate_*` function
  per type, the stock content, and the §6 resolver-building functions. Pure
  data and pure functions — no mutable store, matching `cartalith-assets`'
  slot/library split conceptually but keeping the *mutable* half out of this
  crate per this port's own rule (`ARCHITECTURE.md`), since
  `jp_capacity_ex`/`jp_calc_land_ex`/`jp_plan_ex` (below) need to be
  computable from this data with no Godot runtime.
- `cartalith-godot/src/travel_bridge.rs` (new, also godot-free —
  `journey_bridge.rs`'s exact isolation pattern, no `#[func]`s this
  dispatch): `TravelEntry` + a generic `EntrySet<T>` (stock bootstrap, add/
  duplicate/edit/delete, `reset_to_stock`) wrapping all four definition
  types identically rather than four hand-written CRUD blocks, `TravelLibrary`
  bundling the four sets plus fresh-id generation, per-entry validation
  passthroughs, usage tracking, and `animal_overrides()` — the map
  `cartalith_civ::travel_library::animal_resolver_fns` turns into the two
  closures `jp_plan_ex` consumes.

**The four field lists, as implemented** (§3): Animals & mounts —
classification (name, role\[multi\], substitutes-for, size class,
availability), capacity & speed (load capacity kg, draft pull kg, base speed
km/h, sustainable hours/day, forced-pace cap), sustenance (fodder kg/day,
water L/day, grazing tolerance, waterless limit days), a ten-row terrain
table (`TL_TERRAIN_KEYS`: Plains/Steppe/Forest/Hills/Mountain/Marsh/Desert/
High Pass/Snowfield/River Ford, each a multiplier or `blocked`), requirements
& prohibitions (yokeable/requires-road/seasonal-closure-blocked/carryable-
aboard-vessel/usable-as-mount/handlers-per-N-head), cost (upkeep sp/day/head).
Vehicles — class, load kg, draft head required (count + role), speed
multiplier, road requirement, off-road and ford (each multiplier-or-blocked),
carryable aboard vessel. Vessels — mode\[multi\], hold kg, crew required,
base speed, water rating (sheltered/coastal/open), sailing window
(daylight/continuous), portage capable. Party set-ups — the party-form-only
subset of `JpPlan`/`JpParty` reused directly rather than re-invented
(transport, mount animal, vessel, hours, pace, season, supply days, carry
food, grazing, foraging, and all ten party counts), with
`PartyPreset::from_jp_plan`/`apply_to` being the spec's own "Capture party
from planner" and "apply a set-up leaves per-stage overrides untouched".

**Stock data, and what's borrowed vs. new**: 7 animals (Donkey/Mule/Camel/
Horse/Ox/Yak/Reindeer, per §3.1's own mockup examples). The first four
mirror `cartalith_civ::jp_animal_stats`'s own golden-tested `cap_kg`/
`food_kg_day`/`water_l_day`/`mounted_speed_kmh` figures exactly, and their
terrain rows mirror `jp_animal_terrain_mod`'s built-in per-species overrides
— existing golden data, not invented a second time. Ox/Yak/Reindeer are
genuinely new content: plausible, internally-consistent draft-animal figures
(ox: slow/huge pull/cheap keep; yak: high-altitude affinity; reindeer: fast
on snow, minimal fodder from lichen grazing) grounded in common domain
knowledge, not academic citation, per this milestone's own framing for new
design. 5 vehicles (Handcart/Cart/Wagon/Sledge/Travois) mirror `jp_capacity`'s
own `JP_CART_CAP`/`JP_WAGON_CAP`/`JP_SLED_CAP`/`JP_TRAVOIS_CAP`/draft-head
constants exactly. 11 vessels mirror `jp_ship_stats`'s full roster
(`speed_kmh`/`cargo_kg`/`crew` exactly; `water_rating` derived from its
`river`/`sea`/`open_sea` flags; `sailing_window`/`portage_capable` are new
fields with no engine equivalent, a plausible oared-vs-sailing split). 2
stock party presets (Light Pack Column, Heavy Wagon Caravan) span the party
form's own extremes.

**Validation** (§4): `Incomplete` always takes priority over `Conflicting` —
checking a conflict over data that isn't even fully present would itself be
a guess (unit-tested explicitly:
`incomplete_takes_priority_over_a_conflict_that_would_otherwise_fire`). Two
mechanically-checkable conflict rules for `AnimalDef`: §4's own worked
example (grazing tolerance restricted to grassland while a non-grassland
terrain row still carries a real, non-zero, non-blocked multiplier), and a
new one this field list implies (`roles` claims `Mount` but `usable_as_mount`
disagrees, or the reverse). One each for `VehicleDef` (`road_requirement ==
None` but `off_road == Blocked` — a vehicle that needs no road yet cannot
move without one) and `VesselDef` (`water_rating == Sheltered` but `modes`
claims `Sea`). Every stock entry validates `Ok` (unit-tested per type), so
duplicating one starts clean, exactly as the Asset Library's own stock
entries do.

**The terrain-vocabulary seam, found while wiring**: the spec's ten-category
terrain table (§3.1) is coarser than and not identical to the engine's real
per-stage terrain strings (`CART_TERRAINS`, 13 keys including two road
surfaces). `tl_terrain_key_for_engine` is the documented, one-way mapping:
`"Paved Road"`/`"Dirt Track"` are deliberately excluded (matching
`jp_animal_terrain_mod`'s own built-in convention — no species entry ever
overrides either road surface), `"Ruins / Debris"` has no ten-category row,
and `"Steppe"`/`"River Ford"` are two spec rows with no engine terrain that
maps back to them (the engine has no distinct grassland surface, and a river
ford is a crossing-count hazard, not a `terrain` string) — both fields are
still stored and validated, only their consumption is inert.

**The engine wiring — real, not decorative:** `jp_capacity`/`jp_calc_land`/
`jp_plan` each gained an `_ex` sibling (`jp_capacity_ex`/`jp_calc_land_ex`/
`jp_plan_ex`) taking an `Option<&JpAnimalResolver>` — two closures
(`stats`/`terrain_mod`) that return `None` for "no override, use the
built-in table". `resolve_animal_stats`/`resolve_animal_terrain_mod` are the
one central fallback point, so a partially-incomplete override (say,
`load_capacity_kg` unset) degrades field-by-field to the built-in figure
rather than an all-or-nothing swap. The original three functions are now
one-line wrappers passing `None` — **confirmed byte-for-byte unchanged**:
the full existing Journey Planner test suite (`cargo test -p cartalith-civ`,
`cargo test -p cartalith-godot`) passes unmodified, 0 regressions.
`jp_calc_land_ex`'s terrain block gained one real new branch: a `blocked`
terrain for the pace-setting animal now returns a hard `JpBlocked`, the same
shape as the existing wheeled-vehicle/mount-terrain checks, not merely a
very small multiplier.

Two `travel_bridge.rs` integration tests prove the whole chain
(`TravelLibrary::animal_overrides` → `animal_resolver_fns` →
`JpAnimalResolver` → `jp_plan_ex`) against a real, synthetic-but-real land
world: `a_custom_animal_override_changes_a_computed_journey` — a custom
donkey (500 kg capacity vs. stock 80 kg, 1.5 km/h vs. stock 4.0 km/h) makes
`jp_plan_ex`'s `days` strictly increase and `avg_km_day` strictly decrease
versus the identical call with `None`; `a_blocked_terrain_override_actually_
blocks_the_stage` — marking the route's own terrain `blocked` on a custom
donkey makes `jp_plan_ex` return a journey with `blocked_idx.is_some()`.

**Disclosed, named gaps, not approximated:**

1. **Only the four built-in party-form species can override anything.**
   `AnimalDef::species_key` is `Some("donkey"|"mule"|"camel"|"horse")` for an
   entry representing one of `JP_ANIMAL_KEYS`, `None` otherwise. `JpParty` is
   a fixed four-field struct with no generic animal-count map, so the stock
   Ox/Yak/Reindeer entries (and any wholly new custom species) are real,
   validated, inspectable data with **no live engine hook** — a genuinely
   larger change to a golden-tested type, correctly out of this milestone's
   scope.
2. **Vehicles and vessels are data-only.** `jp_capacity`'s cart/wagon/sled/
   travois mass constants and `jp_ship_stats`' vessel table are still the
   fixed built-ins; no `animal_resolver_fns`-shaped bridge exists for either.
3. **No `#[func]` boundary exists yet, by design.** `cartalith-godot/src/
   lib.rs`'s `WorldGen`/`jp_compute` are untouched — no `travel_library`
   field, no GDScript-visible surface. The shape a later dispatch adds is
   documented in `travel_bridge.rs`'s own module doc: build
   `TravelLibrary::animal_overrides()` → `animal_resolver_fns` →
   `JpAnimalResolver`, pass `Some(&resolver)` to `jp_plan_ex` in place of
   today's `jp_plan` at the `jp_compute` call site.
4. **"Saved journeys" do not exist as a referenceable, persistent thing
   anywhere in this port**, checked rather than assumed:
   `WorldGen.infra.routes`/`route_get` are drawn polylines with no attached
   party plan, and `jp_compute` computes and returns a plan without storing
   it. §4's "how many saved journeys ... reference it" usage count is
   therefore always `0` — `TravelLibrary::animal_usage_in_journeys` says so
   explicitly rather than inventing a count. Party-set-up usage tracking
   (`animal_usage_in_presets`) *is* real, since presets are the library's
   own stored rows.

**A stale-info check, not assumed from this dispatch's own brief**: the
brief that started this milestone described `JOURNEY_PLANNER_SCOPE.md` as
"6 of 10 transport modes built, 4 correctly deferred on unbuilt
dependencies" and asked whether any of those four were about to close
because of the Travel Library. Re-reading that document directly (not
trusted from the brief) shows this description is **already stale**: its
own milestone 2 section opens with "**Its last two landed with milestone
6's pass**... fully complete 2026-08-18" — `jp_auto_pick_transport`,
`jp_auto_pick_vessel`, `jp_best_land_transport_for_stage` and
`jp_best_package_for_stage` are **all four already ported and
golden-verified**, one day before this milestone started, unrelated to
Travel Library work. There is nothing left to close on that front, and
nothing this milestone did closes anything either way — none of the four
consume `AnimalDef`/`VehicleDef`/`VesselDef` or the new resolver. Recorded
here so a future session reads the current top-of-section note rather than
the original per-milestone write-up further down the same document, which
is exactly the trap `JOURNEY_PLANNER_SCOPE.md`'s own structure (a live
correction note followed by an intentionally unedited original) warns
against silently falling into.

**Verified**: `cargo build -p cartalith-godot` (the cdylib, not just `cargo
test`) and a headless Godot 4.7.1 boot (`--headless --path godot-project
--quit`) both clean, no parse/registration errors, `Initialize godot-rust`
in the log. `cargo test -p cartalith-civ`/`cargo test -p cartalith-godot`
(full workspace suites, not just `--lib`) both 0 failures. 18 new
`cartalith-civ` lib tests (327 total, +18 from 309), 13 new `cartalith-godot`
lib tests (202 total, +13 from 189). `cargo clippy --all-targets` on both
crates: clean on every line this milestone touched (one `type_complexity`
lint on `animal_resolver_fns`' return type fixed with two named type
aliases; the one pre-existing warning elsewhere in `cartalith-civ` predates
this milestone and was left alone). `cargo fmt` was **not** run
project-wide this pass — the workspace has no `rustfmt.toml`, and a default
100-column pass reformatted dozens of unrelated files across both crates
(other agents' in-flight work included); the two new files were kept at
their as-written formatting and every edit to `lib.rs`/`travel_library.rs`/
`travel_bridge.rs` in this milestone is a clean, minimal diff against
`HEAD` (94 lines in `cartalith-civ/src/lib.rs`, 1 line in
`cartalith-godot/src/lib.rs`, both new files untouched by anyone else) —
worth a note in `README.md`'s working discipline for the next session that
runs `cargo fmt` blind.

## DCC shell GUI audit — the class of bug `f274d13` found once, hunted across the whole shell (2026-08-19)

Owner request, verbatim: a full audit of the GUI now that the §4.5 tool
palette, Journey Planner, Timeline, and the Data manager/Asset Library
windows had all landed in rapid succession this session, specifically for
the shape of bug `f274d13` (`git show f274d13`) already found and fixed once
— `File ▸ Storage locations` and `Change locations…` opening two dialogs
that showed the same four rows redundantly. Read `DCC_SHELL_SPEC.md` and
`DCC_CONTROL_INDEX.md` in full as ground truth, then every menu/window/
workspace file under `godot-project/shell/` (`menus.gd`, `app.gd`,
`dcc_shell.gd`, `dcc_settings.gd`, `right_dock.gd`, `world_data_window.gd`,
`data_manager_window.gd`, `asset_library_window.gd`,
`journey_planner_view.gd`, `layers_popover.gd`, `dcc_widgets.gd`,
`dcc_theme.gd`, `dcc_icons.gd`, `global_tools.gd`, `new_world_dialog.gd`,
`performance_window.gd`, and every file under `workspaces/`) — roughly
12,000 lines, cross-referenced against the spec's own prose section by
section. `viewport_host.gd`/`map_overlay.gd` were explicitly out of scope: a
concurrent agent was fixing two unrelated rendering bugs in exactly those
two files during this pass.

**The overwhelming finding: this shell is unusually self-consistent.** Every
file already follows `menus.gd`'s own honesty convention (a control with no
engine behind it ships disabled, with an accurate tooltip, never silently
inert), `DccTheme`/`DccWidgets` are used uniformly with no ad hoc
`Label.new()`/hardcoded-colour drift found anywhere outside the two
`RichTextLabel`s that genuinely need BBCode, and dozens of disclosed-gap
tooltips were individually re-verified against the real Rust source this
pass (grepping `cartalith-godot/src/lib.rs` for the `#[func]` each one
claims is missing) and found still accurate. Six real findings survived that
bar, all fixed:

1. **Right dock chrome title hardcoded `"LAYERS"`** (`dcc_shell.gd`'s
   `_build_right_dock`) regardless of which context `right_dock.gd` actually
   dispatches — Sample, Settlement, Route, River, Faction, Measure, Region
   select, Stamp stack, Journey. None of those is ever "Layers" (the Layers
   popover is a wholly separate canvas control, `layers_popover.gd`); the
   mockup's own "Layers" screen was this dock's *pictured default state*,
   not its permanent chrome label, and every other context already drew its
   own real section header one scroll-step below the stale one. Fixed with
   a new `DccShell.right_dock_title` field and `set_right_dock_title()`
   method, kept in sync by `RightDock._rebuild()`'s own new
   `CTX_TITLES`/`_current_title()` (which also correctly falls back to
   "Sample" for `CTX_SETTLEMENT`/`CTX_JOURNEY`'s own no-data fallback paths,
   not just the happy path).
2. **`Assets ▸ ⧉ Asset library` / `⧉ Sprite sheet slicer` used the wrong
   glyph.** `menus.gd` prefixed both with `DccIcons.SYMBOLS["panels"]` (▤) —
   the phone app-bar's own "Panels" button glyph, reused here by mistake —
   instead of §2.3's own literal `⧉` "opens a dedicated window" marker,
   which every window this shell actually opens already carries in its own
   title (`"⧉ ASSET LIBRARY"`, `"⧉ DATA MANAGER"`). `DccIcons.PATHS` does
   carry a drawn `"window"` glyph matching the concept, but nothing in this
   codebase ever calls `PopupMenu.add_icon_item` — every menu glyph
   elsewhere is plain Unicode text — so the fix follows that established
   convention (`"⧉ Asset library"`, `"⧉ Sprite sheet slicer (▦)"`) rather
   than introducing a new one for two menu rows.
3. **`data_manager_window.gd`'s `export_assets` route reason was stale.**
   It named `Assets ▸ Asset pack ▸ Build ▸ Export pack .zip…` — a submenu
   path that was never actually built into `menus.gd` (only Icon families ▸/
   Texture sets ▸ exist there) — and claimed "it needs the asset-library
   window, which is not built," true when that sentence was presumably
   written, false since `asset_library_window.gd` shipped earlier this
   session with its own real (and honestly disabled, for a still-real
   reason) Export pack .zip button. Corrected to name that real button.
4. **`PerformanceWindow` was built, wired into `app.gd`'s `_ready()`, and
   completely unreachable** — `DccApp.open_performance()` existed and
   nothing anywhere called it, and the window itself was a bare "Being
   ported from main.gd's performance dialog" placeholder despite real,
   already-exposed data sitting unused directly beside it:
   `EngineBridge.gpu_stages_used()` (backed by the real `#[func]`
   `get_gpu_stages_used`), `quality_tier()`/`quality_tiers()`/
   `recommended_quality_tier()` (four real `#[func]`s already driving
   `menus.gd`'s own Render quality submenu), and Godot's
   `OS.get_static_memory_usage()` (the same source the menu bar's own
   `top_mem` readout already uses). Rebuilt with real content from all
   three, and wired to a new, real `Preferences ▸ Memory ▸ Working set…`
   item — closing part of finding 6 below at the same time.
5. **`Data ▸ World data tables…` was the one live, enabled menu item in
   this shell that opened a window delivering nothing** — also a bare
   "Being ported from main.gd's world-data dialog" placeholder, but unlike
   `PerformanceWindow` this one had a real, non-`_todo()`'d entry point
   already pointed at it, which is exactly the "enabled and silently inert"
   shape `menus.gd`'s own header comment says this shell must never ship.
   Rebuilt as three real, filterable tables (Settlements/Provinces/
   Economy) reading `bridge.settlements()`/`provinces()`/`trade_balances()`
   directly — the identical real data `civilization_workspace.gd`'s own
   Settlements/Politics/Economy categories already read and cap at a top-N
   summary; this window is the uncapped, name-filterable view those
   categories point at, not a second implementation of the same query.
6. **Preferences ▸ Memory was missing two of its own three items.** §2.5
   names Undo history, Working set and Clear caches; only the first ever
   made it into `menus.gd`, not even as disabled placeholders for the other
   two. Fixed alongside finding 4: Working set is now a real, live item;
   Clear caches is now an honest `_todo()` (no atlas/field cache exists yet
   to clear).

**Findings considered and not changed, with why:** the combined
`_todo(p, "Tiled LOD · tile size · atlas cache", ...)` single menu item
(spec lists Tiled LOD / Tile size · LOD levels / Atlas cache / Chunk debug
overlay as four rows) reads as a deliberate compression of four
identically-blocked items into one disabled line rather than clutter, not a
coverage gap — left alone. The unused `ID_PREF_UNITS_KM`/`ID_PREF_UNITS_MI`
constants in `menus.gd` (declared, never referenced — Units is a single
`_todo()`'d item instead) are dead code, not a user-facing bug; left alone
rather than risk widening this pass into a km/mi feature build. §4.5.4's
"road / track / trail / bridge" way-type vocabulary in the spec disagrees
with the engine's real `ManualWayType` (road/track/sea_lane/ancient) —
already disclosed accurately, in detail, by `infrastructure_workspace.gd`'s
own doc comment (checked against `infra_tools_bridge::parse_way_type`
directly); already correct, not a finding.

**Verified**: every modified file re-read after editing. A headless Godot
4.7.1 boot (`--headless --path godot-project --quit`) clean — "Initialize
godot-rust" in the log, no parse or registration errors. A scripted drive
(a temporary `_audit_check.tscn`/`.gd` pair, not committed) booted the real
shell, generated a small world, and exercised all six fixes directly rather
than trusting a read-through: the right-dock title read `SAMPLE`, then
`SETTLEMENT` on `on_settlement_selected`, then `SAMPLE` again on
deselection; `WorldDataWindow` built 127/9/128 real rows across its three
tabs against that world; `PerformanceWindow` opened with real content both
from its own `open()` and via the new Preferences menu dispatch path; and
`DataManagerWindow`'s corrected `export_assets` reason string read back
exactly as written, confirming the dictionary edit took. No Rust file
touched anywhere in this pass.

