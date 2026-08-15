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
