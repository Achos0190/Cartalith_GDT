# Status quicklist

A living checklist, not a narrative — read this first, before `CHANGELOG.md`,
to know what's done vs. open without re-reading the whole history each
session. Update it in the same commit as whatever changes its answer.
`CHANGELOG.md` stays the detailed record of *how*; this is only *what/done?*.

Last updated: 2026-08-16 (post rendering-port/UI-reskin/criterion-7 landing).

## MVP_SCOPE.md — "done means all seven"

| # | Criterion | Status |
|---|---|---|
| 1 | Height/temp/rain/flow match golden data | **Done.** Every pipeline stage golden-verified bit-exact/tight-tolerance against the real JS engine: tectonics/orogeny (graph-driven T1-T5), volcanism+provinces, climate (temp/wind/rain), ocean currents, terrain wind deflection, erosion, hydrology, world-structure archetypes, full carve pipeline. Nothing left pinned to a stale default. |
| — | UI/UX (not one of the seven, but part of the `/goal` "feature and graphic parity" directive) | **Reskinned 2026-08-16.** `theme/app_theme.tres`/`main.tscn` rebuilt on a `ui-ux-pro-max` dark-dashboard design system (real palette + grouped World Parameters/World Structure/Advanced cards + visible keyboard-focus states). Deferred: real Fira font files (license-unverified, kept Godot's default font), and `MVP_SCOPE.md` point 9 (sea level) still isn't user-adjustable from Godot — needs a new `#[func]` binding, not done this pass. See `CHANGELOG.md`'s "Godot UI reskin via ui-ux-pro-max" entry. |
| 2 | Recognisable 2D map render | **Done (2026-08-16).** Replaced the placeholder elevation-only tint with the reference's real default-settings biome/hillshade renderer (`crates/cartalith-godot/src/render.rs`, new): `materialWeights` (snow/rock/sand/wetland/canopy/grass), the six climate-selected colour ramps, multi-scale hillshade, `bioBlend` desaturation, edge haze, and `seaColorCore` (smoothed-bathymetry depth/temperature banding — confirmed this is JS's real default, not a stretch feature). Two real bugs caught by golden verification, not by read-through: a missing final `ao*vignette` multiply (~40% too bright at corners) and sea colour needing the smoothed, not raw, depth field. Golden-verified against two real `generate()` runs at `1e-4` tolerance (`golden_parity_render.rs`). Deliberately excludes every `state.viz.*`-gated stretch feature (splat texturing, geology, NPR "Painter" styles, AO/SVF/shadow, SDF tinting) — all off at JS's own defaults; that's genuine Phase 3 scope, see below. |
| 3 | Windows `.exe` builds + owner has run it | **Partly done.** Phase 0 walking-skeleton confirmed on real Windows (`ping()` round-trip). Now that criterion 2's real rendering and the UI reskin have both landed, the current full MVP UI (seed/resolution/generate/load-save, real biome-coloured map, reskinned theme) has *not yet* been screenshotted/run as a whole on this session's real Windows desktop access — worth doing before calling this fully closed. |
| 4 | Android `.apk` builds + owner has installed/run | **Apk builds and packages, confirmed.** Install+run on *real hardware* is not reachable from this environment — investigated via emulator, root-caused as a SwiftShader/emulator limitation, not our code (see `CHANGELOG.md`'s Android emulator entries). Per the `/goal` set 2026-08-16, this is **no longer a hard requirement** — testing via Godot editor/headless and local Android Studio is sufficient for now. |
| 5 | Map width scales feature size | **Done** — a consequence of criterion 1's parity, verified via the world-structure archetype port. |
| 6 | Changelog entry per milestone | **Ongoing** — `CHANGELOG.md` has an entry for every milestone so far; keep this up. |
| 7 | Opens a real HTML-app `.zip`, renders it, checked against the HTML app's own output | **Done (2026-08-16).** `cartalith-io::load_save` verified bit-exact against a real export produced by running the actual, unmodified reference engine (not just its own synthetic round-trip tests): `crates/cartalith-io/tests/golden_parity_real_export.rs` against `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`. See `CHANGELOG.md`'s "cartalith-io verified against a real HTML-app export" entry for the harness technique (including a genuine `generate()`-name-collision gotcha found along the way). |

## ROADMAP.md phases

| Phase | Status |
|---|---|
| 0 — Walking skeleton | **Done.** Triangle/button/`ping()` confirmed on Windows and Android (build+package; Android run-on-device is the one open half, see criterion 4 above). |
| 1 — Terrain MVP | **6/7 solid, 1 partial.** Criteria 1/2/5/6/7 done, criterion 4 softened by the 2026-08-16 `/goal` (no longer a hard requirement), criterion 3 needs a fresh whole-app run/screenshot on Windows (see above). Two "easy to forget" Phase-1 closeout items from `ROADMAP.md` are **not started**: a credits screen, and a licence audit of the crates pulled in (`PROVENANCE.md`). |
| 2 — Civilisation layer | Not started. Out of scope until raised. |
| 3 — Rendering and 3D | Not formally started. Two things to remember when it does: **(a)** criterion 2's renderer (above) ports the reference's *default-settings* material model only — real biome colours, real hillshade — explicitly excluding every `state.viz.*`-gated stretch feature (splat texturing, geology microtexture, NPR "Painter" styles, AO/SVF/shadows, multi-sun, SDF coast/river/biome tinting). Wiring any of those in is genuine Phase 3 work. **(b)** When that work lands, re-invoke `ui-ux-pro-max` for the UI side rather than bolting raw sliders onto the newly-exposed params — keep it consistent with the 2026-08-16 dark-dashboard design system. **(c)** GPU compute shaders were researched 2026-08-16 (prompted by `godot-demo-projects/compute/heightmap`) and found not applicable *right now*: `project.godot` uses the `gl_compatibility` renderer, which doesn't support `RenderingDevice` compute dispatch at all (engine-level constraint, already documented in `.claude/skills/godot-shell/SKILL.md`). If Phase 3 revisits the renderer for other reasons (3D terrain drape, particles), GPU-accelerated presentation-layer work becomes reachable as a side effect — that's the point to reconsider it, not before, and not for core generation (which must stay CPU-Rust for golden-parity reproducibility regardless of renderer). |
| 4 — Asset Library | Not started. |
| 5 — Urban morphology | Not started. |

## Known-open items (not owner-blocked, just not done yet)

- Credits screen (Phase 1 closeout, `ROADMAP.md`).
- Crate licence audit (Phase 1 closeout, `PROVENANCE.md`).
- Real Fira Sans/Fira Code font files for the UI theme (design-system match found the pairing; sourcing + OFL-license verification deferred).
- Sea level as a user-adjustable Godot control (`MVP_SCOPE.md` point 9 — real terrain scope, just not wired to a `#[func]` yet).

## Owner-only items

- Criterion 4's full sense (installed and run on the owner's *actual phone*) — softened by the 2026-08-16 `/goal`, no longer blocking.
- Nothing else currently requires the owner specifically; this session has real Windows desktop + `godot4` CLI access, which closes most of what earlier sessions couldn't do themselves.
