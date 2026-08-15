# Status quicklist

A living checklist, not a narrative — read this first, before `CHANGELOG.md`,
to know what's done vs. open without re-reading the whole history each
session. Update it in the same commit as whatever changes its answer.
`CHANGELOG.md` stays the detailed record of *how*; this is only *what/done?*.

Last updated: 2026-08-16.

## MVP_SCOPE.md — "done means all seven"

| # | Criterion | Status |
|---|---|---|
| 1 | Height/temp/rain/flow match golden data | **Done.** Every pipeline stage golden-verified bit-exact/tight-tolerance against the real JS engine: tectonics/orogeny (graph-driven T1-T5), volcanism+provinces, climate (temp/wind/rain), ocean currents, terrain wind deflection, erosion, hydrology, world-structure archetypes, full carve pipeline. Nothing left pinned to a stale default. |
| 2 | Recognisable 2D map render | **In progress.** Previous renderer was a placeholder elevation-only tint (own doc comment called it "deliberately not attempted" biome colouring). A fork was dispatched 2026-08-16 to port the reference's real default-settings material/biome color model (`materialWeights` + palette ramps + sea gradient + hillshade) — check `CHANGELOG.md`'s latest entry for the outcome. |
| 3 | Windows `.exe` builds + owner has run it | **Partly done.** Phase 0 walking-skeleton confirmed on real Windows (`ping()` round-trip). The current full MVP UI (seed/resolution/generate/load-save, real rendering) has *not* been separately confirmed running on Windows by the owner — worth a fresh screenshot/run once criterion 2's rendering lands, now that this session has real desktop access on the owner's machine. |
| 4 | Android `.apk` builds + owner has installed/run | **Apk builds and packages, confirmed.** Install+run on *real hardware* is not reachable from this environment — investigated via emulator, root-caused as a SwiftShader/emulator limitation, not our code (see `CHANGELOG.md`'s Android emulator entries). Per the `/goal` set 2026-08-16, this is **no longer a hard requirement** — testing via Godot editor/headless and local Android Studio is sufficient for now. |
| 5 | Map width scales feature size | **Done** — a consequence of criterion 1's parity, verified via the world-structure archetype port. |
| 6 | Changelog entry per milestone | **Ongoing** — `CHANGELOG.md` has an entry for every milestone so far; keep this up. |
| 7 | Opens a real HTML-app `.zip`, renders it, checked against the HTML app's own output | **Done (2026-08-16).** `cartalith-io::load_save` verified bit-exact against a real export produced by running the actual, unmodified reference engine (not just its own synthetic round-trip tests): `crates/cartalith-io/tests/golden_parity_real_export.rs` against `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`. See `CHANGELOG.md`'s "cartalith-io verified against a real HTML-app export" entry for the harness technique (including a genuine `generate()`-name-collision gotcha found along the way). |

## ROADMAP.md phases

| Phase | Status |
|---|---|
| 0 — Walking skeleton | **Done.** Triangle/button/`ping()` confirmed on Windows and Android (build+package; Android run-on-device is the one open half, see criterion 4 above). |
| 1 — Terrain MVP | **6.5/7** per the table above. Two "easy to forget" Phase-1 closeout items from `ROADMAP.md` are **not started**: a credits screen, and a licence audit of the crates pulled in (`PROVENANCE.md`). |
| 2 — Civilisation layer | Not started. Out of scope until raised. |
| 3 — Rendering and 3D | Not formally started. Note: the criterion-2 rendering fork (above) ports the reference's *default-settings* material model only — real biome colours, real hillshade — explicitly excluding every `state.viz.*`-gated stretch feature (splat texturing, geology microtexture, NPR "Painter" styles, AO/SVF/shadows, multi-sun). Those remain genuine Phase 3 scope, not done by this pass. |
| 4 — Asset Library | Not started. |
| 5 — Urban morphology | Not started. |

## Known-open items (not owner-blocked, just not done yet)

- Credits screen (Phase 1 closeout, `ROADMAP.md`).
- Crate licence audit (Phase 1 closeout, `PROVENANCE.md`).
- Criterion 7's real-export verification (in progress, see table above).
- `.claude/skills/ui-ux-pro-max/` — installed but never reviewed (bundles third-party Python scripts; flagged as a security concern when installed, not yet resolved).

## Owner-only items

- Criterion 4's full sense (installed and run on the owner's *actual phone*) — softened by the 2026-08-16 `/goal`, no longer blocking.
- Nothing else currently requires the owner specifically; this session has real Windows desktop + `godot4` CLI access, which closes most of what earlier sessions couldn't do themselves.
