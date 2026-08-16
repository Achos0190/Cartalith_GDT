# Decision log

Every major choice, what it beat, and why. Read the reasoning before revisiting
one — each was narrowed from real alternatives, not taken by default.

| # | Decision | Beat |
|---|---|---|
| 1 | Native rewrite | wrapping the HTML app (Tauri/Electron/Capacitor) |
| 2 | Rust engine | C/C++ |
| 3 | Godot shell via `gdext` | pure Rust (bevy, wgpu+winit) |
| 4 | 2D only for v1 | 2D and 3D together |
| 5 | Cloud session builds | local PC, or a hybrid |
| 6 | Personal/hobby bar | store distribution |
| 7 | Golden-value parity | independent correctness |
| 8 | Code in a new repository | code beside these docs |
| 9 | Godot version pinned at setup | a version pinned here |

## 1. Native rewrite, not a wrapped web app

Wrapping would cost almost nothing and reuse the whole tested engine. It was
rejected on the stated goal, not on merit: a wrapper still runs the same
Canvas2D/WebGL JS inside a webview, which is an installable icon rather than
different performance — and Android WebView GPU and touch behaviour is a known
source of quirks this project already works around.

If the goal were "ship an exe and apk quickly," wrapping would be the right call.
It is lower risk and far less work.

## 2. Rust, not C/C++

Performance is a wash — both compile through LLVM, neither is inherently faster
when written competently. Three things decided it:

- **Memory safety where it matters most here.** The engine manipulates large flat
  arrays across many sequential passes, exactly where C/C++ produces silent
  out-of-bounds and aliasing bugs. The HTML CHANGELOG records hitting this class
  repeatedly — edge-clamping errors, off-by-one indexing, antimeridian wraparound,
  NaN propagation — and catching it only through runtime testing, because JS
  offers no compile-time guarantee. Rust catches much of it at compile time, or
  panics at the faulty line rather than returning a wrong answer three passes on.
- **One toolchain, three targets.** `cargo` cross-compiles to Windows, Android
  (`cargo-ndk`), and WebAssembly, so a future browser build could share the engine
  core. C/C++ reaches WASM through Emscripten with clunkier tooling.
- **Mature Godot bindings.** `gdext` gives Rust GDExtension access without a
  hand-written C ABI bridge.

**The tradeoff, stated plainly:** the borrow checker fights the pattern this
generator leans on — passes mutating large shared arrays in place. Expect real
up-front thought about ownership (arena patterns, `&mut [f32]` threaded through
functions, `rayon` where Web Workers were) rather than mechanical translation.
A one-time cost during the port, not an ongoing tax.

## 3. Godot shell, not pure Rust

Godot solves for free what is not this project's differentiator: windowing, input
including touch, a UI system, and one-click export for Windows and Android —
which carries the NDK, Gradle, and signing plumbing a pure-Rust stack would have
to build by hand. The value lives in generation and simulation, which is what
moves to Rust.

## 4. 2D only for v1

3D means a second rendering pipeline — mesh generation, camera, lighting — on top
of porting and verifying a generation engine. Cutting it keeps the first milestone
achievable. `ROADMAP.md` Phase 3 brings it back.

## 5. Cloud session builds; owner verifies on hardware

Godot cross-exports to Windows and Android from Linux given the right SDKs and
templates, so building needs no Windows machine.

What a container cannot do is run the `.exe` on Windows, install and touch-test
the `.apk`, or confirm real GPU behaviour. Build here, verify there — the same
carve-out the HTML project already documents for WebGL, Workers, canvas, and
touch. **Every milestone states which half it achieved.**

## 6. Personal/hobby distribution

Builds the owner runs: a plain `.exe`, a sideloaded `.apk`. No signing
certificates, store listings, or policy compliance yet. This can change later at
no architectural cost.

## 7. Golden-value parity, not independent correctness

The JS engine carries 200+ versions of measured, owner-verified correctness —
scale-invariant terrain detail, river formation, climate coupling, settlement
placement. Re-deriving that by feel would discard it silently.

Parity makes the existing engine ground truth the port is checked against
mechanically. `PARITY_TESTING.md` covers how, and why exact bit-identity — the
standard the HTML project holds itself to JS-to-JS — is not achievable across
languages.

## 7a. Principled equivalence for GPU/optimized paths (owner decision, 2026-08-16)

§7 above still governs the CPU reference pipeline — every subsystem ported
this session (tectonics, climate, erosion, hydrology, rendering, real HTML
export round-trip) stays golden-verified against the JS engine and that
work is not being discarded or devalued.

What changes: for GPU-accelerated or otherwise re-optimized paths, exact or
tolerance-bound numerical matching against JS is **not** a requirement when
it becomes impractical. The GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`,
`cartalith-gpu`) hit exactly this wall — `hash()`'s JS semantics depend on
IEEE-754 *double*-precision rounding at an intermediate magnitude (~2^61)
that exceeds `f32`'s useful range entirely, and WGSL has no working `f64`
support on this toolchain (`naga` doesn't implement `enable f64;`). Owner's
own framing: "rust, godot and wgpu are inherently a different type of code
language" — cross-hardware GPU determinism is a categorically different,
harder problem than the cross-*language*-on-CPU tolerance §7 already
accepted, and insisting on JS-array-diffable output from a GPU path is not
worth blocking real optimization over.

The replacement bar, for any path where JS-parity genuinely can't be
tested 1:1: implement the same **academic principles and generation flow**
the reference embodies (the actual algorithm/model being approximated, not
an arbitrary reinvention — `PROVENANCE.md`'s citation list still describes
*why* a formula looks the way it does even when its exact digits are no
longer being chased), and judge the result by whether it reaches an
**equal-or-better visual/qualitative outcome**, not by array diffing.
"Same seed reproduces the same world" (this port's own determinism
contract) still holds *within* whichever path (CPU or GPU) actually ran —
this is about JS-cross-checkability, not about abandoning determinism
inside the Rust/wgpu implementation itself.

This reopens a redesigned, GPU-native hash/noise function as a real option
— the GPU pilot's "not viable" verdict was specifically about reproducing
JS's exact rounding, not about GPU noise generation being impossible in
principle. A GPU-safe hash is legitimate future work; scope it properly
(same discipline as every other milestone this session — a scope doc, not
an improvised inline rewrite) rather than retrofitting it into whichever
crate happens to be open at the time.

## 8. Documentation here, code in a new repository

`Cartalith_RC` has strict conventions of its own — single HTML file, version per
file, a test and hash discipline tied to that file — that a Cargo workspace does
not fit. A separate repository avoids conflicts over `.gitignore` scope, CI
assumptions, and `CLAUDE.md` ownership.

These documents are written to be copied wholesale into that repository as its
seed documentation. Nothing here depends on staying in `Cartalith_RC`.

## 9. Godot version pinned at setup, not here

Install the latest stable Godot 4.x available when the repository is created, and
record the exact version in its own toolchain notes.

This document was written against a knowledge cutoff. A patch version hardcoded
here risks being stale — missing bugfixes, or naming a release no longer
recommended — by the time anyone executes the plan. Same discipline the HTML
project enforces about re-measuring rather than trusting a stale assumption.
