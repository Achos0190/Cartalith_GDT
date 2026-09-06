# Round 3 — five undesigned surfaces, designed 2026-09-06

**Source of truth is the Claude Design project, not this file.** Project
`067f80e7-dbb7-4492-8e69-96aaa8050a4d`, canvas
`Cartalith Undesigned Surfaces.dc.html` (§1a–§1e), with a per-screen control
inventory beside it in `docs/DESIGN_<ID>_*.md`. This file exists so a lane can be
briefed without re-fetching, and so the findings below are not rediscovered.

Owner-supplied, 2026-09-06: *"Here's part of the missing elements."* Batch 1 of 3
of the 14 surfaces in `SCREENS_NEEDED.md`.

| § | Screen | Register | Spec file in the design project |
|---|---|---|---|
| 1a | Find on map | ED-05 | `docs/DESIGN_ED-05_FIND_ON_MAP.md` |
| 1b | Data manager ▸ Checks | DM-10, DM-11 | `docs/DESIGN_DM-10_VALIDATION.md` |
| 1c | Keyboard shortcuts editor | HE-02, PR-16 | `docs/DESIGN_HE-02_SHORTCUTS.md` |
| 1d | Save layout as… | WI-01 | `docs/DESIGN_WI-01_SAVE_LAYOUT.md` |
| 1e | Rail expansion | SH-01 | `docs/DESIGN_SH-01_RAIL_EXPANSION.md` |

**These were drawn against the shell that exists**, not against the research
alone — each inventory has an "Exists today" column read off this repository at
`main`. Where the engine cannot back a field it is **drawn dashed with the
code's own reason**, which is the discipline this project asks of its own work.
Treat a dashed field as a finding, not as a gap to fill with something plausible.

## What each screen costs, in the designs' own terms

**1e Rail expansion (SH-01) — cheapest; every part already exists.** Expanding
reveals labels and subtitles for the same items and never new ones (Blender /
Photoshop, the register's own proposal). `DccShell.DOMAINS` already carries
`label` and `subtitle` and already uses them as the button tooltip;
`ROLE["w_rail_expanded"]` is `[200, 264]`; `set_rail_expanded`/`is_rail_expanded`
and the rotated chevron all ship. **This is a new *use* of existing data, not new
data.** Two departures flagged for an owner call, with the code named as the
newer party: rail labels are the theme's sans rather than tracked mono (because
`dcc_shell.gd` says so where it builds them), and the expanded rail reads
`label` — World / Civilization / Cartography — not the `rail` token. One layout
problem, drawn rather than argued: Civilization's subtitle is 70 characters and
wraps to three lines at 200 px, so rows are unequal; the canvas exposes a
`railSubtitle` prop (`wrap`/`clip`) so the alternative can be *seen*. **No phone
surface, deliberately** — the app bar's subtitle slot already carries the world
name and seed, and displacing that to explain a domain the user just tapped is a
bad trade.

**1a Find on map (ED-05) — the engine half is built.** `PlaceSearch` ships with
`build()`, `size()`, `all()`, `search(q)`, and `CommandIndex.search(q)` backs the
`.` scope. What is owed is the **status-bar locator field** (250 × 17) and a
`⌘F`/`Ctrl+K` accelerator on the existing disabled `Edit ▸ Find on map…` row.
Ranking is already built and is **deliberately substring, not fuzzy**: three
bands, each alphabetical — prefix of the name · elsewhere in the name · only in
subtitle or kind — and an empty query returns the whole index in build order, so
the empty state is the index rather than a blank panel. Band headers are a
**design addition** computable client-side; the engine returns one flat array.
Scopes `s` `f` `lb` `r` `.` all resolve today. **`l` for landmarks is drawn
dashed on the code's own argument** — `icon_list()` carries `family/slot/set/
scale`, four closed vocabularies and no identifying name, so a row reading
"pine / forest" is not a place anyone typed a name to find; shipping a scope that
always returns nothing is worse than an absent one.

**1d Save layout as… (WI-01) — and it fixes a live defect.** `Window ▸ Layouts`,
`ID_WIN_SAVE_LAYOUT = 602`, `_capture_layout()`/`_apply_layout()` and
`DccSettings.layouts()/save_layout()/delete` all exist. **`dcc_shell.gd` records
that both guards were dead — neither name was ever declared — so
`Save layout as…` silently stored nothing.** The design assumes that fixed. Owed:
the modal (name well, collision warning, REPLACE/SAVE), a "what this saves"
readout, and **four seeded layouts** that do not exist yet. The register's four
tasks are not in conflict with three domains, because a layout is not a domain —
the snapshot carries `domain` *and* `mode`: Generate = WORLD/pipeline, Sculpt =
WORLD/sculpt, Cartography = CARTO/style, Journey = CIVIL/planner. **Confirm the
five region flags in `_capture_layout()` against the code before drawing them** —
the design lists frame §4's five and says so.

**1b Data manager ▸ Checks (DM-10) — the finding is the screen.** **Cartalith
cannot ship QGIS's geometry validator, because there is no geometry-validity
entry point over map layers.** `poly_self_intersects` exists but is internal to
`cartalith-urban`, uncalled across the boundary, and its subject is city blocks
rather than territory or coastline; in `cartalith-spatial/src/geo.rs` **an
unclosed ring is normal output**, documented and asserted in
`golden_parity_geo.rs`, so reporting it as an error would be *wrong* rather than
merely absent; ring orientation is checked nowhere. What the engine does validate
is **definitions** — `validate_animal` / `validate_vehicle` / `validate_vessel`
(`Ok` · `Incomplete(missing)` · `Conflicting(conflicts)`), `validate_party_preset`
(no `Conflicting`, per its own doc) and `AssetLibrarySession::validate() ->
Vec<String>`. So the screen is a `checks` route over those five, with **Map
geometry drawn dashed**. `LOCATE` is drawn **present and disabled** — no row here
is a placed thing — so the table already has the right shape the day a positional
validator exists. `FIX SELECTED` is **not** built: the engine has validators
only, nothing that changes state.

**1c Keyboard shortcuts editor (HE-02) — the largest, and the cost is stated.**
`ShortcutsDialog` already **generates** its table by walking the live `MenuBar` at
open and reading `get_item_accelerator`, recursing into submenus, asserting
non-emptiness. **That generation is the property the file exists for: a generated
table cannot disagree with the app.** Editing introduces a second source, so
overrides need a store *and* must be applied back through
`set_item_accelerator` at menu build or an edit vanishes on next launch. New
work: a `Keyboard` pane in Preferences, a **chord-capture well (a genuinely new
widget — nothing in `dcc_widgets.gd` reads a key combination)**, a conflict panel,
`ASSIGN ANYWAY`, `RESET ALL`, an override count. **The chord column is
deliberately dashed** for File/Edit/Assets/Data/Window/Help: drawing plausible
chords would create exactly the second copy of the truth the dialog exists to
prevent. Only two are named because they are verifiable — `Ctrl+Shift+P` read
from the code, and `Ctrl+K` *proposed* here for Find on map. Conflicts follow
**Blender, not Photoshop**: a duplicate is never blocked; the panel lists every
other binding and the user decides. Phone is read-only **by decision** — an
override there would apply to a menu bar the phone shell does not draw. Surfaces
an open decision rather than settling it: `PARITY_AUDIT.md` §20 F10, layer
hotkeys `1–8` as shipped against the reference's `0 B T F S W R`, unmade for
three passes.

## Build order, and why

**1e → 1a → 1d → 1b → 1c.** Rail expansion first because every part exists and it
is a use rather than a build; Find on map next because its engine half ships;
Save layout third because it repairs a defect that silently loses user data;
Checks fourth because it needs the five validators surfaced across the boundary;
Shortcuts last because it is the only one that needs a new widget and a new
persisted store.

---

## Batch 3 — `Cartalith Undesigned Surfaces 3.dc.html`, checked 2026-09-06

PR-07 colour · PR-10 tiles & LOD · PR-15 units · HE-01/HE-03 docs & report.
**The strongest of the three canvases**: three of its four sections argue for
*less* surface than the register asks for, and the argument is the work. Two
premises still fail, and one recommendation collides with a standing ruling.

**PR-07 — the central premise is false, and it inverts the recommendation.**
The canvas says *"Display Device is sRGB and nothing else"* and draws a locked
readout *"because the list has no second entry"*. There is a second entry:
`render::ColorSpace` is `Srgb` **and `DisplayP3`** (DCI-P3 primaries, D65,
sRGB's transfer function), `set_color_space` is bound and forwarded through
`engine_bridge.gd`, and `render_workspace.gd` already wires a picker to it.
**Measured 2026-09-06 by the export verifier: Display P3 moves the viewport
raster by up to 23 byte levels.** So the axis is real, user-selectable, and
already has a home. The honest screen is not "there is nothing to choose" — it
is that **the picker affects the screen and deliberately does NOT affect
exports**: `encode_png_rgb8` writes no ICC profile, so applying the transform
would produce an untagged P3 file that everything colour-managed reads as sRGB.
That is a fact a user needs told, and no surface tells them today.

**HE-03 — the owner already ruled, and the canvas draws the opposite.**
`LARGE_ITEM_RULINGS.md`: *"`Report an issue` → **Replace with a local diagnostic
dump**. Rename to a save-diagnostic-report action writing generation info,
missing bindings, project format version, GPU state and the last error to a file
the user attaches themselves. **No endpoint required.**"* The canvas draws an
`OPEN TRACKER` button that opens a browser. **An owner decision is newer than any
canvas.** The canvas half-arrives at the right answer on its own — its note says
*"COPY DETAILS is the row that earns its place: it works with no network"* — and
its **"what will be attached" review list is exactly right** and should be kept:
a pre-filled report is a claim about what the app knows, so the user must see it
before it leaves. Build the ruling's version with the canvas's review panel.

**PR-10 — the route is real, the row is disabled.** `Preferences ▸ Tiles & LOD`
exists with `Tile size` (256/512/1024 — **three values, not the canvas's four**),
`LOD levels` and `Chunk debug overlay`. `Atlas cache ▸ Size cap · GB` exists as a
**`_todo` row**, not a live one. So the canvas is right that the destination
exists and wrong that the control does. Its split — Preferences keeps what
changes drawing now, tiled *output* becomes a named build definition under
`Data ▸ Export` — is sound and matches ruling 28's own division. **Needs the
owner to confirm that destination before the build dialog is drawn.**

**PR-15 — no toggle exists in the shell, which settles it toward Reading 3.**
The canvas says *"a Cartalith canvas already draws a working km/mi toggle in the
menu bar"* and calls it a contradiction with a kilometre-only engine. Searched
the shell: **there is no km/mi toggle anywhere in `shell/*.gd`.** So the
contradiction is between a *design canvas* and the engine, not inside the code —
which makes **Reading 3 (the toggle was drawn by mistake; remove it from the
canvas)** the cheapest answer and the only one needing no build work. The
canvas's own core argument stands and is worth keeping either way: **units belong
in the file, not the preferences**, because a world is shared and must not read
in kilometres for its author and miles for everyone they send it to.

### What is implementable now, and what is not

| § | State | Next step |
|---|---|---|
| HE-01/HE-03 | **Ready** — the ruling settles it and the canvas supplies the review panel | Build the diagnostic dump. `menus.gd` is held by a running lane |
| PR-07 | **Redraw** — premise false | Draw the screen-vs-export distinction instead |
| PR-10 | **Owner** — confirm `Data ▸ Export` as the destination | Then draw the build dialog and manager |
| PR-15 | **Owner** — Reading 3 recommended on the evidence | Then remove the toggle from the canvas |
