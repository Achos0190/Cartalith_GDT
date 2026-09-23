# SCREENS_NEEDED.md — the undesigned-surfaces brief, and where its answers are

**What this is:** the brief that asked Claude Design (2026-09-06) for 14 GUI
surfaces that had no artboard anywhere — not in the DCC shell spec, not in a
subsystem spec, not in the mockups. **What it is not:** a live list. Every
surface below now has a design (the owner's three round-3 canvases) or a ruling
that it needs none. Whether each is *built* is `cartalith-native/docs/STATUS.md`'s
answer, not this file's.

The comparable-application research each entry was briefed with lives whole in
`GUI_GAP_REGISTER.md` §7 — read it there. This file's earlier copy of that
research was a lossy extract, cut off mid-sentence in five entries, and is
dropped rather than repaired (history: `git log -- SCREENS_NEEDED.md`).

## The fourteen, and where each answer lives

Round 3 batches 1 and 3 are indexed in `design/ROUND3_INDEX.md`, with three of
batch 1's boards redrawn in `design/round3-corrected/`. **Batch 2 is not indexed
anywhere in this repository**; it is assigned below by elimination (the three
canvases covered exactly the fourteen), and its canvas lives only in the owner's
Claude Design project. Owner rulings are newer than any canvas and win where
they disagree.

| # | Surface | Register | Research | Design | Rulings that bind it |
|---|---|---|---|---|---|
| 1 | Find on map | ED-05 | §7.2 | round 3 §1a; `round3-corrected/FindOnMap.dc.html` | — |
| 2 | Data manager ▸ Validation (drawn as *Checks*) | DM-10, DM-11 | §7.5 | round 3 §1b | — |
| 3 | Colour management | PR-07 | §7.6 | round 3 batch 3 | `LARGE_ITEM_RULINGS.md` "Colour management → Build it"; Ruling AN |
| 4 | Tiled LOD, tile size, atlas cache | PR-10 | §7.7 | round 3 batch 3 | Rulings U and X |
| 5 | Units | PR-15 | §7.8 | round 3 batch 3 | "`Units` (km / mi) → Build, and add nautical miles" |
| 6 | Keyboard shortcuts editor | HE-02, PR-16 | §7.9 | round 3 §1c | "Rebindable keyboard shortcuts → per-context table with conflict detection" |
| 7 | Save layout as… | WI-01 | §7.10 | round 3 §1d; `round3-corrected/SaveLayout.dc.html` | — |
| 8 | Documentation and Report an issue | HE-01, HE-03 | §7.11 | round 3 batch 3 | "`Report an issue` → replace with a local diagnostic dump" — the canvas's `OPEN TRACKER` loses to it |
| 9 | Calculation trace window | JP-05 | §7.12 | round 3 batch 2 | — (§7.12: an inline group, not a `⧉` window) |
| 10 | Sculpt: brush shape, stroke & grid, actions | WW-03, WW-04, WW-05 | §7.13 | round 3 batch 2 | — |
| 11 | Label font role | CA-07 | §7.14 | round 3 batch 2 | — |
| 12 | Style presets | CA-08 | §7.15 | round 3 batch 2 | — |
| 13 | Layer list search; Blocks / Verticality | CA-09 | §7.16 | **none needed** | Search reuses the Find-on-map vocabulary; the footer tabs were ruled **not built** (`GUI_GAP_REGISTER.md` §7.16, reading C) — both names describe controls the shell already has |
| 14 | Rail expansion | SH-01 | §7.17 | round 3 §1e; `round3-corrected/Rail.dc.html` | — |

Three surfaces were excluded from the fourteen because a design already existed:
the undo-history panel (ED-02, PR-11 — round 1's History artboard), Data manager
▸ Sources (DM-06 — round 2's DataManager artboard), and Data manager ▸
Conversion (DM-07…09 — removed by owner decision 2026-08-20, so the canvas that
still draws it is the stale party).

## Rules for the next undesigned-surface brief

Kept because each exists for a reason that already cost something here.
Breakpoints and what to hand back are `DESIGN_HANDOFF.md` §10's; tokens resolve
in `cartalith-native/godot-project/shell/dcc_theme.gd`, not in a canvas.

1. **Newer canvas wins; an owner decision is newer than any canvas.** Where no
   canvas exists, derive from the DCC canvases' own vocabulary
   (`DCC_SHELL_SCOPE.md`).
2. **Touch floor is 44 dp** (`DccTheme.PHONE_TAP_MIN`), multiplied by the
   phone's `_phone_scale` — 115 physical px at a scale of 2.62. It is a floor on
   the target, both axes.
3. **Never draw a fabricated value, and never a fabricated reason.** A field
   with no engine source is drawn dashed with what is missing. A dash's reason
   is a claim: three shipped with false causes and all three had to be
   corrected.
4. **Say what backs every field.** A design that names a number the engine
   cannot produce becomes a build that fabricates one. Round 3's own "Exists
   today" columns failed against the code seven times — twice calling shipped
   work unbuilt, twice dashing a live field — so check them at the symbol.
5. **Draw both palettes.** The shell ships light and dark.
