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
