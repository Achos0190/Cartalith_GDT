# `docs/` — the **source project's** documentation, not this port's

Everything in this directory belongs to **Cartalith Gen1, the single-file HTML
application** (`reference/Cartalith Gen1 v2.10.html`). It was copied here as
provenance, exactly as `PROVENANCE.md` and the root `CLAUDE.md` instruct:
constants without reachable derivations get "cleaned up" by someone who cannot
see why they hold.

**It is reference material. It is not this port's design.** Nothing here
describes decisions made for the Rust/Godot port, and nothing here is
maintained as the port evolves. Read it to understand *why the JS engine does
what it does*; never as a statement of what this repository is building.

## ⚠ Two filenames collide with the port's own documents

| This directory | Repo root | Same name, different document |
|---|---|---|
| `docs/UNIFIED_TOOL_PLAN.md` | `UNIFIED_TOOL_PLAN.md` | **Unrelated.** Here: the HTML app's own 2026 plan to merge `elevation_foundation_v0.036.html` and `Cartalith_V1.914.html` into one file. At root: this port's tool-system milestone plan (A-F), written from the reference's Sculpt editor. |
| `docs/ROADMAP.md` | `ROADMAP.md` | **Unrelated.** Here: the HTML app's own priority-ordered roadmap. At root: this port's phase plan (Phase 0-5). |

When any instruction, scope document or agent brief in this repository names
`UNIFIED_TOOL_PLAN.md` or `ROADMAP.md` **without a directory prefix, it means
the one at the repository root.** `UI_SHELL_DESIGN.md` (imported verbatim from
the design project) refers to `docs/UNIFIED_TOOL_PLAN.md` because the design
team followed a `docs/`-rooted convention this repository does not use — that
discrepancy is recorded in the root `UNIFIED_TOOL_PLAN.md`'s own header.

## Where the port's own documentation lives

| Location | Contents |
|---|---|
| Repository root | Every scope, decision and plan document for the port (`DECISIONS.md`, `ARCHITECTURE.md`, `ROADMAP.md`, the `*_SCOPE.md` family, `VISION.md`, `FUNCTIONAL_CONTRACT.md`, …). `README.md` is the index. |
| `cartalith-native/docs/` | The port's living `CHANGELOG.md` and `STATUS.md`. |
| `design/` | Owner-supplied UI mockups and handoff specs, imported verbatim. |
| `reference/` | The frozen `Cartalith Gen1 v2.10.html` snapshot and its generated `FUNCTION_INDEX.md`. |

## What's here

- `HANDOFF.md` (350 KB) — the HTML project's own session hand-off. The single
  richest explanation of that codebase's structure and invariants.
- `ROADMAP.md` — the HTML project's roadmap (see collision note above).
- `UNIFIED_TOOL_PLAN.md` — the HTML project's merge plan (see collision note).
- `SCULPT_EDITOR_INTEGRATION_PLAN.md` — real prior art; the port's tool-system
  milestone B cited it while porting the Sculpt editor's 13 landform features.
- `AFFORDANCE_FIELD_PLAN.md`, `ASSET_PACK_FORMAT.md`,
  `ASSET_PACK_INTEGRATION.md`, `ATLAS_ARCHITECTURE.md`,
  `BIOME_AND_VISUALS_PLAN.md`, `DEEP_MERGE_PLAN.md`,
  `GENERATOR_PARAMETERS.md`, `LOD_PYRAMID_PLAN.md`,
  `WORLD_CENTRIC_ARCHITECTURE.md`, `WORLD_REGIONAL_TILING_PLAN.md` — the HTML
  project's subsystem designs.
- `SESSION_LOG_2026-06-10.md`, `SESSION_LOG_2026-06-11.md` — its session logs.
- `research/` — the academic sources `PROVENANCE.md` requires be kept
  reachable.

**A caution these documents share with the frozen HTML itself**: they describe
the source app as of when they were written. Where one disagrees with
`reference/Cartalith Gen1 v2.10.html`, the code wins — the Asset Library
investigation found exactly that and recorded it (`ASSET_LIBRARY_SCOPE.md`).
