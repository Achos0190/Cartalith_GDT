# Presented but not wired — the standing table

> **Relationship to `cartalith-native/docs/STATUS.md`** (added 2026-08-31)
>
> **This file answers a question `STATUS.md` does not ask: "the product draws
> this control — is there anything behind it?"** It is per-control, not per
> milestone. Each row carries a `file:line` that was opened, the reason a user
> is shown, and a proposal sourced from the spec or from an existing engine
> binding. That granularity is the point, and it does not belong in a milestone
> ledger.
>
> **`STATUS.md` is authoritative for status.** This table's counts are a *cut*,
> dated and reproducible, not a live figure. If a row here says a binding is
> missing and the code says otherwise, **the code wins and the row is a
> defect** — that is this document's own primary failure mode, now **five** times
> proven against itself (44 rows / 12 wrong on the first re-cut; two rows the
> 2026-08-31 cut closed in-audit while still printing them as open; the first
> 2026-09-03 cut, where **fourteen of the seventeen open Large rows were already
> built**; and two found by the second 2026-09-03 pass — a "correction" to the
> Trivial table that **reversed an accurate note** and asserted the opposite of
> the comment it cited, and a provenance claim that dated a false string to
> 2026-08-25 and called it "never revisited" when `git log -S` puts an edit to
> that same note on 2026-09-03. **A wrong correction and a false exoneration are
> both worse than the original defect, because each looks freshly checked.**
>
> Nothing here is a schedule. What is *chosen* for the 18 Large rows is in
> `LARGE_ITEM_RULINGS.md`; what is *queued* is in `OUTSTANDING_WORK.md`, which
> deliberately counts this whole document as one row so the two cannot drift.

Owner, 2026-08-30: *"Make sure all menu's are created and all presented
functions are wired or the ones that have no code behind them get listed in a
table with a proper proposal (inferred from the menu name and highest probable
explanation/design spec of the named function.)"*

## This cut — 2026-09-03 (second pass), against `HEAD` `0bba2f9` plus the working tree

**Method, and it is the whole method.** Every open row and every
dangerous-class entry is re-opened **at its cited symbol**, not at its line
number, and judged against the code. Nothing is carried forward on trust. Where
a line number has drifted, the symbol is found first and the citation rewritten
from it. `git status` at both passes shows an uncommitted working tree with
other lanes editing `shell/*.gd`; every symbol below was read from that tree.

**The second pass added one rule, and it is the one that found b-7, b-8 and
b-9.** Re-opening the *rows* is necessary and not sufficient, because the rows
are the claims someone already doubted. The claims nobody doubted are the
sentences sitting quietly beside working controls. The query that reaches them:

```
grep -rn "cartalith[-_]" --include=*.gd shell/ | grep -iE "no |not |never |missing|absent"
```

**40 hits**, run from `cartalith-native/godot-project/` at this cut. Three are
false, and two of those three assert that an entire ported, golden-tested crate
module does not exist. (The count is the command's, not a recollection: an
earlier draft of this paragraph said 26, which was a *narrower* pipeline's
output quoted against the wider command printed here.)

### Line numbers drifted while this cut was being written

**Not a caveat — a measurement, and the strongest argument this file has for its
own "find the symbol, not the line" rule.** Two other lanes were editing
`shell/*.gd` and `cartalith-godot/src/*.rs` throughout. Every citation was
re-resolved at the symbol a second time at close of pass, and these had moved
**within the same session**:

| Symbol | Cited early in this pass | At close of pass | Moved |
|---|---:|---:|---:|
| `civilization_workspace.gd` b-9 string | `:5257` | **`:5405`** | 148 |
| `_build_relationships()` | `:4918` | **`:5066`** | 148 |
| `render.rs::apply_color_space` | `:5493` | **`:5734`** | 241 |
| `lib.rs::build_paint_preview_texture` | `:8926` | **`:8936`** | 10 |
| `lib.rs::paint_set_brush` | `:8873` | **`:8883`** | 10 |
| `lib.rs::get_rivers` / `::river_at` | `:7132` / `:7159` | **`:7138` / `:7165`** | 6 |

Every §b citation was re-resolved once more at close of pass. **b-1 … b-8 all
held exactly** — `rivers_note()` `:801` / drawn `world_workspace.gd:522`;
`cartography_workspace.gd:508` and `:643`; `layers_popover.gd:77`;
`place_editor_window.gd:570`; `tool_bar.gd:605`/`:609`;
`world_workspace.gd:159`; `performance_window.gd:140` — because those files were
untouched by the other lanes. `map_overlay.gd` **was** edited and its three
citations still held at `:523`/`:554`/`:555-556`. Only b-9's file moved.

**A reader applying these fixes should grep the quoted string rather than jump
to the line.** The numbers above were true at close of pass over an uncommitted
tree, and the table immediately above is the evidence that "true at close of
pass" has a half-life measured in hours here.

**The headline of the first 2026-09-03 pass was 21 open rows → 6.** This
second pass re-opened all six at their symbols and **all six are still open**:
nothing closed, and nothing was found mis-scheduled. The finding is elsewhere.

**The dangerous class went 6 → 9.** Three new entries, each found by opening a
claim that names a Rust symbol and then opening the symbol. One stale source
comment closed. And **two of this document's own claims were wrong and are
corrected below** — a "correction" that reversed an accurate note, and a
provenance claim that made a defect look older and less culpable than it is.

| Tier | Open at the 2026-09-02 cut | Closed this cut | Open now |
|---|---:|---:|---:|
| Trivial | 0 | — | **0** |
| Small | 1 | 0 | **1** |
| Medium | 3 | 1 | **2** |
| Large | 17 | 14 | **3** |
| **Total** | **21** | **15** | **6** |

**Second pass, same day.** All six were re-opened at their symbols again and
**none closed**: `label_glyph_layout` (still no bridge handle in `map_overlay.gd`,
still two font-size models), `statusMid` (still Owner question 2), the bounded
previews (still declined at both builders' own docs), Cut/Copy/Paste/Select all
(still no clipboard), the 3D viewport (still no `Camera3D`/`MeshInstance3D` in
`shell/` or any `.tscn` — the only repo hits are the third-party
`addons/godot_ai/` handlers, which are not this shell), and `region_new_world`
(still no caller outside three probes). A cut that closes nothing is the
expected result of running two cuts in one day; it is recorded rather than
padded.

**Why the first pass closed so many at once.** The 2026-09-02 cut closed nothing and said so. Between
it and this one, `45b368d` and `0bba2f9` landed the label-class model and the
collision culler, the river entity, the colour-space pipeline, the icon
placement pass and its density brush, the five civilisation authoring
operations, the CPU thread pool, the units switch, the rebindable-shortcut
store, the diagnostic report, the landmark reject list, the settlement
diagnostics binding and the right dock's append model — **thirteen of the
fourteen closures are one of those.** This is `LARGE_ITEM_RULINGS.md`
executed, not a re-count.

### The dangerous class went the other way: 1 real entry → 6 → 9

That is the finding of both 2026-09-03 passes, not the closures. Five of the
first six are false *because* of a closure above: **a capability landing turns
the sentence explaining its absence into a lie, and nothing in the build catches
that.**

- `tools/audit_wiring.py` structurally cannot see it — the binding *is* called;
  it is the prose that lies. It found none of the nine.
- `cargo test --workspace` cannot see it — the strings are GDScript literals
  and Rust hint text, and no test asserts on either. **Measured, not asserted:
  the suite is `2 992 passed; 0 failed; 25 ignored` with all nine standing.**

Two of the first six are the **exact shape** of the defect the 2026-09-02 cut
found on the Settlement diagnostics overlay: a control blaming urban milestones
that have shipped. One more repeats a claim that the file it cites as agreeing
with it has already corrected in its own source.

**What separates the three the second pass added is age, and it was measured
rather than assumed.** The first draft of this paragraph asserted that b-7 and
b-8 "were never true" — that the crates predated the strings denying them.
`git log -S` refutes it, and the real answer is sharper:

| Entry | The false string was written | The thing it denies landed | False for |
|---|---|---|---:|
| b-7 (Köppen) | `7f5e54c`, 2026-08-18 | `koppen.rs`, `b7a46a7`, 2026-08-23 | **16 days** |
| b-8 (GPU devices) | `595582d`, 2026-08-19 | `multi.rs`, `0de790a`, 2026-08-20 | **14 days** |
| b-1 … b-6 | various | `45b368d` / `0bba2f9`, 2026-09-03 | hours to days |

So all nine share one mechanism — the sentence was true when written and a
landing falsified it — and the difference is only how long nobody looked. **b-8
was falsified the day after it was written.** That changes the remedy: watching
each wave's diff for closures catches the six young ones and neither of the two
that have been lying for a fortnight. The old files have to be swept too, which
is what the `grep "cartalith[-_]"` query above is for.

---

## Trivial — 0 open (17 of 17 closed)

Unchanged from the 2026-09-01 cut and not re-walked in full. **Two closure
descriptions in that cut are now wrong and are corrected here**, both found in
contact with other work rather than by a sweep:

| Row | The 2026-09-01 closure note said | What the code says now |
|---|---|---|
| State religion reads `—` | "reads `roster.get("religion", "")`, prints `"none"` as a real answer rather than dashing" | **RETRACTED — the 2026-09-01 note was accurate and the first 2026-09-03 pass wrongly marked it stale.** Re-opened at `_build_faction_details()`'s religion field, `right_dock.gd:1853-1860`: `rel := String(roster.get("religion", "")).strip_edges()`, printed as `rel.capitalize() if rel != "" else "—"`. `"none"` is non-empty, so it prints as *None* — exactly what the note said. The dash is reserved for a **missing key**, and the comment at `:1857-1859` says so in as many words: *"`"none"` is a real answer from `cartalith-civ`'s own vocabulary, not an absence … so it prints rather than dashing."* The retracted correction asserted the opposite of that comment while citing its line range. **A wrong correction is the same defect class this document exists to catch, committed by this document.** |
| `pack.rs` declines painted layers | "stating precisely what's still missing (decoding `biomes`/`terrains` pixels — a real, separate, correctly-deferred job)" | **That job is done.** `pack.rs:24-30` now lists the painted layers among the milestone's built surfaces; `LoadedPack::biomes`/`::terrains` decode to `render::GroundTile` and `render.rs`'s paint blend consumes them instead of taking the flat-swatch branch unconditionally. |

---

## Small — 1 open (24 of 25 closed)

### Open

| Item | Where (`file:line`) | Current state | Size |
|---|---|---|---|
| `label_glyph_layout` is re-implemented in GDScript | `map_overlay.gd:515-554` (the constants block and its own explanation); binding `EngineBridge.label_glyph_layout` at **`engine_bridge.gd:3211`** (drifted from `:2554`) | **Unchanged in substance, re-verified at the symbol.** The three constants are still pinned and named (`ARC_STRAIGHT_THRESHOLD := 0.01` at `:554`, checked against `labels.rs`); `map_overlay.gd` still contains no call to `label_glyph_layout`, so the full fix is still undone, for the two reasons its own comment gives (no bridge handle in this file; a px-per-cell font model that differs from `labels::label_font_size`). The named cost — `label_box_at`/`label_handles` sizing off the engine's font size while the drawn glyph sizes off this file's — still stands. **Three stale citations, all in this row's own cited location**, and the count grew this pass. (1) `map_overlay.gd:523` cites the binding as *"`engine_bridge.gd:2469 func label_glyph_layout`"*; it is at **`:3211`**, 742 lines away. (2) `:554` cites `ARC_STRAIGHT_THRESHOLD` as `labels.rs:150`; it is **`labels.rs:164`** (`pub const ARC_STRAIGHT_THRESHOLD: f64 = 0.01;`). (3) `:555` and `:556` both cite `labels.rs:176` for the radius floor and the spread divisor; `arc_label_layout` opens at **`labels.rs:182`** and the straight-line branch reads `ARC_STRAIGHT_THRESHOLD` at `:184`. The *values* are all still correct — `0.01`, `1.2`, `2.2` — so nothing here is wrong about the code; a comment block written expressly so `grep` would find the Rust/GDScript pair now points `grep` at three wrong places. **A comment written to stop a citation drifting has drifted in every citation it carries.** | small |

### Closed

Unchanged from the 2026-09-01 cut (24 rows), not reproduced here.

---

## Medium — 2 open (15 of 17 closed)

### Open

| Item | Where (`file:line`) | Current state | Size |
|---|---|---|---|
| `statusMid` composite | `app.gd:739-805` (drifted from `:739-791`); slot reserved at `dcc_shell.gd:3399-3402` (drifted from `:3231-3237`) | **Unchanged, ~90% built, one field genuinely blocked.** Stage name, pass duration and autosave state are live at `_refresh_status_mid()` (`app.gd:767`). `repaint NN ms` is still deliberately absent and the comment at `:747-754` still owes the reader why: this shell composites through `ViewportHost` + `map_overlay.gd` + live overlays with no single-pass timer. `grep -rn repaint shell/*.gd` still finds prose and nothing else. **Still blocked on Owner question 2**, which is now the only owner question blocking a row. | medium |
| Previews re-upload the whole texture | `cartalith-spatial/src/pass.rs:193,199` (`touched_tiles`/`touched_bounds`, **unmoved** — the bare `pass.rs` of earlier cuts is ambiguous, this workspace has one `pass.rs` and it is in `cartalith-spatial`, not `cartalith-godot`); consumers `lib.rs:8033` (`build_sculpt_preview_texture`; `:6128-6179` two cuts ago, `:8027` at the first 2026-09-03 pass) and `:8936` (`build_paint_preview_texture`; `:7000-7023`, then `:8926`) | **Unchanged, and still honestly declined rather than skipped.** Both builders re-opened. `build_sculpt_preview_texture`'s doc still argues a bounded preview needs `render.rs`'s AO/wetness/sea passes reworked over a caller-supplied window — code `golden_parity_render.rs` pins bit-for-bit — and that restricting only the final pixel loop would be "a cosmetic optimisation reported as a real one". `build_paint_preview_texture`'s doc (`:8914`) still argues paint does not need this at all, having no derived whole-grid rasters underneath. Correctly left for a dedicated live-preview pass. | medium |

### Newly closed by the first 2026-09-03 pass

| Item | Where (current) | How it closed |
|---|---|---|
| The right dock does not follow the armed tool (`rdExtraMode()`) | `right_dock.gd:47-100` (the doc), `:1036` (`_tool_section()`), `:1051` (`_append_tool()`) | **Closed by an owner ruling and its implementation, 2026-09-03**: *"Selection wins; the tool appends a section."* The four tool sections shipped first as `CTX_*` constants with `_dispatch()` arms, which made arming a tool **replace** the dock — measured in a booted app: "selecting a settlement then arming Territory: `title=Territory`, `settlement name SURVIVED=false`". They are now `TOOL_PAINT`/`TOOL_STOPS`/`TOOL_ANNO`/`TOOL_TERR` **section ids**, appended by `_append_tool()` *after* `_dispatch()` draws the selection, and **not stored** — which one is drawn is derived from `app.armed_tool` plus the domain on every rebuild, which *is* `rdMode4()`'s own fall-through table rather than a second copy of it. The file carries an explicit "do not give any of these a `CTX_` name again" warning naming each of the three things that caused the takeover. **This also answers Owner question 1**, the last thing this row was formally waiting on. |

### Closed at earlier cuts

The other 14 Medium rows are unchanged from the 2026-09-01 cut.

---

## Large — 3 open (15 of 18 closed)

`LARGE_ITEM_RULINGS.md` recorded owner decisions for all 18 on 2026-08-31 and
said plainly that none was in flight that day. **That is no longer the state of
this tier.** Fourteen closed this cut; with Paint brush falloff (2026-09-01)
and Saved measurements (2026-09-03), fifteen of the eighteen are built.

### Open

| Item | Where (`file:line`) | Ruling | State |
|---|---|---|---|
| `Cut` · `Copy` · `Paste` · `Select all` | `menus.gd:771` (Cut), `:776` (Copy), `:777` (Paste), `:808` (Select all) | Ruled: selection sets → clipboard → commands, **in that order**. | **Open, and step one is now done.** Re-opened at all four `_todo` call sites. The engine holds a real selection **set** per entity kind (`selection.rs`); `icon_select_all`/`label_select_all`/`sculpt_select_all_stamps` are bound and wrapped; Ctrl-click adds and Shift-click takes a range on the canvas. **Step two, the clipboard, does not exist** — nothing can serialise an icon or a label into a buffer, hold it across a world change, or paste it back. All four reasons were rewritten 2026-09-03 against exactly this state and verified true at the code, including the retirement of the old Select-all reason (*"every selection holds exactly one item"*), which the multi-selection model made false. Beyond the clipboard, the row still needs the "scoped to the active layer" dispatch that `DccApp.clear_selection()` already has a shape for. | large |
| The 3D viewport | `menus.gd:1887` (`3D viewport defaults`, drifted from `:1756-1757`); also `:1863` (`Anti-aliasing · anisotropy`) | Ruled: **deferred**, research first. | **Open by ruling, correctly untouched.** `3D_TERRAIN_RENDER_RESEARCH.md` (1 530 lines) exists; three commissioned questions parked. Re-checked repo-wide this cut: no `Camera3D` or `MeshInstance3D` anywhere in `shell/` or in any `.tscn`. Both dependent `_todo` reasons ("there is no 3D viewport") are true. | large |
| `Region ▸ New world from selection` | engine `ops_bridge.rs:317` (`region_new_world`), `:425` (`region_new_world_error`); wrapper `engine_bridge.gd:2989`; **no UI caller** | Ruled: a scoped parity pass, kept separate from GUI work. | **Half closed, and this row's own stated reason has gone false.** The 2026-09-02 cut said *"`extract_region_as_world` still has no `#[func]` and no menu row; `ops_bridge.rs`'s own doc still lists it first among ported-and-unexposed capability."* Two of those three clauses are now false: `#[func] region_new_world` exists (threaded, primitives-in/`bool`-out, refusal reason read back on the main thread), and `ops_bridge.rs:28` now reads **"`extract_region_as_world` is wired, here"**. `EngineBridge.region_new_world()` wraps it with the same `generation_started`/`generation_finished` contract as `import_heightmap()`. **Only the last clause survives: no control anywhere calls it.** A repo-wide search for callers outside `engine_bridge.gd` finds **three** probes — `_verify_region_probe.gd:28,42,61,100`, `_verify_region2_probe.gd:31,58,62` and `_worldswap_probe.gd:132` (the earlier cut named two of the three) — and nothing in `shell/`; `grep -n '"Region"' menus.gd` is empty, so there is no `Region` menu to hang the row on. **This is the cheapest remaining Large row by a wide margin** — one menu row plus the confirm dialog the wrapper's own doc says a caller owes (it destroys the civ layer, labels, icons, ways, routes, paint and sculpt drafts). | large |

### Closed by the first 2026-09-03 pass

Re-checked, not re-listed on trust: four of the fifteen were re-opened at their
symbols by the second pass and all four hold — `apply_color_space`
(**`render.rs:5493`**, not `:5476` as first written; `list_color_spaces`
`lib.rs:6241`, `set_color_space` `:6261`), `DccSettings.shortcut_binding`
(`dcc_settings.gd:567`) with `SHORTCUT_CONTEXT_MENU` at `:557`,
`LandmarkRun::rejects` (`landmark.rs:798`) with `LandmarkRejectReason` at `:549`
and `engine_bridge.gd:4542`, and the right dock's append model
(`_tool_section()` `right_dock.gd:1036`, `_append_tool()` `:1051`, called from
`:851`, the four `TOOL_*` ids at `:89-100`, and the "do not give any of these a
`CTX_` name again" warning at `:69-70`). The measurements slot's four symbols
are exact as cited (`:2110`, `:2388`, `:2405`, `:2545`).

| Item | Where (current) | How it closed |
|---|---|---|
| **CARTO ▸ Labels: the whole panel** | `labels.rs:649` (`LabelClass`), `:660` (`LABEL_CLASSES`), `LABEL_TYPOGRAPHY_DEFAULTS`; UI `cartography_workspace.gd:1579` (`_build_label_classes`), `:1646-1653` (three live dials), `:1852` (real count summary); bridge `engine_bridge.gd:3069` (`label_class_table`), `:3083` (`labels_generated_counts`) | **All three ruled steps built.** (1) `MapLabel::class` is a real field with a `from_key`/`default` contract for archives written before it existed. (2) A generated labelling pass (`generate_labels`) emits per-class placements in drawing order, so a continental name draws under a settlement name. (3) `LabelTypography` carries size/halo/tracking per class, served over the boundary by `label_class_table()` — with the GDScript array demoted to "the fallback now, not the authority" for an older cdylib, and the Rust side holding the test that pins the two identical. The panel's counts come from `labels_generated_counts()` and its three sliders are live, their domains taken from the engine's own ranges so a value the engine would clamp is unreachable on the dial that sends it. |
| **Label collision culling** | `labels.rs` (`LabelRect`, `LabelCullMetrics`, `label_cull_rect`, `LabelClassCount::suppressed`); UI toggle `cartography_workspace.gd:1660`, sent at `:1832` as `"cull": {"on": …, "advance_ratio": …}`, counted at `:1852` | **Built with the labelling pass, exactly as the ruling sequenced it.** The toggle is a live `DccWidgets.toggle` whose callback re-runs the pass. The one number the engine cannot know — mean glyph advance as a fraction of font size — is measured off the shell's own font by `_label_advance_ratio()` and sent with every run, and the fact that boxes are therefore *estimated* is stated to the user in the panel's own note rather than dressed up as a measurement. Suppression is by rank; hand-placed labels are never suppressed. |
| **CARTO ▸ Icons: generated placement** | `icon_bridge/generate.rs` (`IconEditor::generate`, `PlacementFamily`, `CoastSnap`, `sea_mark_gap`); UI `cartography_workspace.gd:1896` (`_build_icon_placement`), `:2007` (`_run_icon_placement`), live sliders `:1945`/`:1952`, three live rules `:1960-1970` | **Built, including the fourth family the ruling ordered.** `PlacementFamily` is `Places`/`Poi`/`Trees`/**`SeaMarks`** — the sea-marks family answering Owner question 4, which the previous cut recorded as answered-but-unbuilt. Both sliders are live (the `_dead_slider` fallback survives only for a cdylib without the binding), all three placement rules are live, `Place this family` runs the pass, and four **disjoint** rejection counters mean a run that placed nothing still says which wall it hit. **Note for the next reader:** `ICON_FAMILIES` (`cartography_workspace.gd:126`) is still three entries and that is correct — it is the *manual* icon vocabulary, a different list from `PlacementFamily`'s four. The 2026-09-02 cut's "the sea-marks family is not in `ICON_FAMILIES`" was checking the wrong list. |
| **The manual-icon tool** | `icon_bridge/brush.rs`; bindings `engine_bridge.gd:2230` (`icon_brush_set`), `:2239` (`icon_brush`), `:2248` (`icon_brush_stamp`); UI `cartography_workspace.gd:943` (`_build_icon_brush_controls`), arming `:984-986`, drag `:1059-1060`, release `:1073-1074` | **Built as `UNIFIED_TOOL_PLAN.md` milestone E.** The 2026-09-02 cut's check — "no icon-brush arming anywhere in `shell/`" — is now false: a `Brush` toggle plus radius and density sliders, drawn only against a cdylib carrying the binding, with the brush's three settings re-sent on every re-arm because `absorb()` rebuilds the engine's editor on every generate and a brush set before a regenerate would otherwise silently revert to `IconBrush::default()` while the UI still showed the user's numbers. |
| **The river entity** | `lib.rs:7138` (`get_rivers(min_order)`), `:7165` (`river_at(gx, gy, radius_cells, min_order)`), `:2377` (`river_dict`); UI `right_dock.gd:412` (selection wired), `:466` (`_on_map_clicked_river`), `:607` (`show_river`), `:1598-1740` (the context) | **Both halves of the ruling built: one binding plus viewport hit-testing.** The dock's own comment records the closure honestly — *"every clause of it is now false: `WorldGen::get_rivers(min_order)` returns the entities and `WorldGen::river_at(…)` picks one"*. `river_at` exists separately from `get_rivers` deliberately, so a click does not trace the network twice. Seven fields are live; the two remaining Actions are disabled with **narrower, re-derived** reasons (an edited polyline has nowhere to be written back into the flow field; catchment decomposition needs labelled basins, the same absence `landmark.rs` records for its confluence rule), and the third Action was **removed rather than re-labelled with a new pretext**, because all three things it promised are now rows above it. This is the model the two files in the dangerous class below should be corrected against. |
| **Civilisation authoring operations** | `lib.rs:4975` (`civ_populate`), `:5088` (`civ_clear_places`), `:5139` (`civ_clear_territory`); ways `civ_clear_ways`; UI `civilization_workspace.gd:1474`/`1504`/`1526`/`1645`, `infrastructure_workspace.gd:1132` (`Generate roads`), `:1147` (`Clear ways & journeys`); params `params.rs:492-531` | **All five re-entrant operations built, plus the civ `PARAMS` group.** The 2026-09-02 cut called this "the single largest CIVIL gap, unchanged" and re-checked that no such symbols existed anywhere in `cartalith-godot`. All five exist now. The `PARAMS` group is derived from `group == "civ"` (`params.rs:831`) rather than a second hardcoded key list, so it cannot drift from the specs. Each destructive op confirms first and states what it destroys. |
| **Settlement diagnostics overlay** | `urban_bridge.rs::settlement_diagnostics`; bridge `engine_bridge.gd:838`; UI `civilization_workspace.gd:1688` (`_build_settlement_diagnostics`), cards at `:1719-1812` | **Closed, and its dangerous-class §b tooltip closed with it.** The 2026-09-02 cut found this control's tooltip blaming urban milestones 9/10/13 that had shipped, and named the corrected, narrower blocker: *"add a lightweight `#[func]` over the ported pure functions."* That is exactly what landed. The card handles its two absent-value cases explicitly rather than defaulting them (`"— wall rung not in this build's diagnostics"`, `"Harbour — scale not in this build's diagnostics"`) — `MISTAKES.md`'s omit-the-key rule applied at the UI edge — and the `river_order == 0` with `has_river` true case is called out as `um_site_profile`'s own deliberate behaviour rather than smoothed over. |
| **Colour management** | `render.rs:5734` (`apply_color_space` — `:5476` at the first pass, `:5493` mid-way through this one, `:5734` at its close), run as the last stage of `build_color_texture` at `lib.rs:6724`; `lib.rs:6241` (`list_color_spaces`), `:6247` (`get_color_space`), `:6261` (`set_color_space`), field at `:2906`; UI `render_workspace.gd:1412` (`_build_color_management`), `:1452` (`_on_color_space`), `:1552` (`_sync_color_space`); menu `menus.gd:1886` | **Built, and the `_todo` it replaced was false in every clause the moment it shipped** — the menu's own comment says so: it read *"the renderer is sRGB-only end to end: `render.rs` writes 8-bit sRGB bytes and nothing carries a colour space through to the texture"*, and both halves became untrue in the same session. **Two devices, not the spec's three**, argued rather than quietly dropped: sRGB and Display P3 are display devices and linear is a *working* space, unshippable at 8 bits (Godot's own `srgb_to_linear` doc). The control lives in CARTO ▸ COLOURS as a per-session display setting, and `_sync_color_space()` exists because `release_world()` re-initialises the field to `Srgb`. `menus.gd:1886` is a **`_signpost`, not a `_todo`**, deliberately, so `command_index.gd` does not count a shipped feature as missing. |
| **Rebindable keyboard shortcuts** | store `dcc_settings.gd:557` (`SHORTCUT_CONTEXT_MENU`), `:567` (`shortcut_binding`), `:579` (`set_shortcut_binding`), `:587` (`clear_shortcut_binding`); applied at build `menus.gd:443-445`; predicates `:451`/`:456`; editor `shortcuts_dialog.gd:154` (`open_editable`), `:359` (`_commit_capture`), `:391` (`_conflict_within_menu`), `:400`/`:411` (reset one / all); menu `menus.gd:2075` | **Built exactly as ruled — "a binding table in `DccSettings`, applied over the menu accelerators at build time. Per-context, not flat."** Conflict detection is scoped within a menu per the ruling, and still applies the change while naming the other row. **One class, opened two ways** (`open()` read-only for Help, `open_editable()` for Preferences), because `GUI_GAP_REGISTER.md` §7.9 calls two separate dialogs a bug. The table is not written down anywhere — it is walked off the live `MenuBar`, so it cannot disagree with the app — and the four accelerators with no menu row behind them (the Layers digits, space-to-pan, Escape, Delete) are declared separately with the file that owns each and never grow a rebind control. |
| **`Units` (km / mi)** | `menus.gd:2055-2057` (three radios), `:2062` (submenu); `dcc_units.gd` (`DccUnits`, `KM_PER_MI`, `KM_PER_NMI`, `to_unit`/`suffix`/`format`/`format_area`/`format_adaptive`/`format_thousands`); consumers `right_dock.gd`, `viewport_host.gd`, `world_workspace.gd`, `menus.gd` | **Built, with nautical miles as the third unit the ruling added.** Canonical storage stays km; `DccUnits` converts only what a user reads. The measure panel routes every reading through it (`right_dock.gd:2044,2055,2073,2085,2094,2191-2192`), and the one place metres are deliberately kept rather than converted is argued at its own call site (`:2011`). **See dangerous class §b-5** — one tooltip still tells the user this preference does not exist. |
| **CPU worker threads** | `cartalith-engine/src/lib.rs:922` (`rayon::ThreadPoolBuilder`); `cartalith-godot/src/lib.rs:4254` (`cpu_logical_core_count`), `:4272` (`cpu_thread_count_active`), `:4292` (`set_cpu_thread_count`); UI `menus.gd:2394` (`_build_cpu_threads_menu`), `:2416` (submenu), `:2428` (`_refresh_cpu_threads_menu`) | **Built as a configurable pool.** The 2026-09-02 cut's check — "`ThreadPoolBuilder` appears nowhere in the workspace outside one benchmark printout" — is now false. The ladder is the spec's own (automatic / 1 / quarter / half / cores − 4 / all), deduplicated because rungs collide on a small machine. **The row says when a choice is not live, because usually it is not**: Rayon's global pool builds once per process, implicitly, before a menu can be opened, so `set_cpu_thread_count()` returns whether the request took effect and the readout prints the **measured running count beside the stored preference** — "12 chosen, 16 running, next start" is visible rather than implied. Its own first version returned `true` from a call that changed nothing, and that is recorded at the call site. The `_todo` still at `menus.gd:1831` is the build-conditional fallback for a cdylib without the binding, not an open row. |
| **`Report an issue`** | `diagnostic_report.gd` (`DiagnosticReport.write()`); menu `menus.gd:3892` and its tooltip at `:3893` | **Built as ruled — replaced with a local diagnostic dump, no endpoint.** Writes generation info, missing bindings, project format version, GPU state and the last error to a text file under the same storage root the Data Manager's exports already use, then reveals it. Three of the five readouts are **reused rather than re-implemented** (`GenInfoDialog._dump_text()`, `EngineBridge.missing_bindings()`, `EngineBridge.project_format_version()` — all three built as Trivial rows in the 2026-09-01 pass, which the 2026-09-02 cut noted were "sitting right there to be reused"). The two that did not exist are built: GPU state from the accessors Preferences already uses, and a last-error retention (`EngineBridge.note_error()`/`.last_error()`) that nothing in the codebase had. The tooltip says plainly that nothing is sent anywhere. |
| **Landmark funnel: crowding and rejected candidates** | `landmark.rs:798` (`LandmarkRun::rejects`), `LandmarkReject`; bridge `engine_bridge.gd:4542` (`landmark_rejects`); UI `civilization_workspace.gd:3910-3915` (both chips), `:3978-3994`, `:4033` (`_lm_show_rejects`), `:4396` (`_lm_rejects`) | **The second half built; the row is now fully closed.** Crowding was already live. What was missing — "no coordinates, counts only" — is now `LandmarkReject` carrying score and position, surfaced by two live chips off **one** `landmark_rejects()` pull, and by a `landmark_rejects` diagnostic map layer the chip turns on before closing the popover so the map is the thing. The disabled state is now the honest one (`No rejects to show`, with its own reason) rather than a permanently-dead chip. |
| **Saved measurements + CSV** | `right_dock.gd:2110` (`_build_measure_actions`), `:2388` (`measurements_document`), `:2405` (`measurements_document_text`), `:2545` (`restore_measurements_document`); slot `annotations/measurements.json` in `cartalith_io::DOCUMENT_SLOTS` | **Closed 2026-09-03, carried forward and re-verified at the symbols.** The fifth save slot the ruling called for, caller-owned, riding the same document dictionary the engine's four ride rather than a second mechanism. Verified end to end by `_measurestore_probe.tscn` (24 checks, 0 fails). |
| **Paint brush falloff** | `paint.rs:143-144,180,199,219,249,299-319`; `paint_bridge.rs:466`; UI `world_workspace.gd:2103,2105`; `DECISIONS.md` §7k | **Closed 2026-09-01, and its one disclosed residual is now closed too.** The doc comment that still said hardness/softness are "never consumed" has been fixed: `paint_set_brush` is `lib.rs:8883` and its doc at `:8863-8866` now reads *"consumed since `DECISIONS.md` §7k"*. Nothing is left owed on this row. |

---

## The dangerous class — 9 entries, all real

A disabled row with an honest tooltip costs a user nothing. These cost them
trust. **b-1 … b-6 were all found on the first 2026-09-03 pass** and all six
are still present, re-verified at the string this pass. **b-7, b-8 and b-9 are
new**, found by the one method that works here: take a sentence that names a
Rust symbol or a crate and asserts its absence, then open the symbol.

The 2026-09-02 cut's one real entry (Settlement diagnostics) is closed; its one
kept-for-contrast non-defect (the CARTO Labels `Class` dropdown) is **obsolete**
— that panel is live now, so there is no frozen half for it to be contrasted
against, and the entry is retired rather than restated.

**All nine are prose fixes in files this lane does not own.** Each is written
precisely enough to apply in one edit.

**Why nine and not six, stated as a method rather than a score.** Two of the
three new ones (b-7, b-8) say a *whole subsystem* is unported or nonexistent
while a golden-tested port of it is reachable from a live control in the same
application, and the third (b-9) is the b-4 shape — a true conclusion resting on
a reason that has become false. None was findable by grepping for a disabled
control: **b-7 and b-9 are notes beside things that work, and b-8 sits in a
window that is otherwise entirely live.** The productive query is the one printed at the top of this cut — 40 hits, of
which three were false. `tools/audit_wiring.py` found none of them, for the
reason this section has now given four cuts running: every `#[func]` involved
*is* called.

### (a) Drawn ENABLED, does nothing meaningful — 0

Re-checked: the six closures of previous cuts hold. `tool_bar.gd`'s duplicate
Hardness slider is still deleted; `world_workspace.gd`'s pair is still the one
surviving control and still consumed.

### (b) The stated reason is FALSE — 9

| # | Where | What it says | Why it is false | Severity |
|---|---|---|---|---|
| **b-1** | `infrastructure_workspace.gd:801-809` (`rivers_note()`), drawn to the user at `world_workspace.gd:522` (WORLD ▸ Hydrology) | *"No river ENTITY is exposed to Godot … What does not cross is a channel run aggregated into one river with a name, a length and a mouth: **there is no get_rivers() and no way to select one**, so v3's per-reach rows … have no entity to hang on. Same finding the right dock's River context reports, field by field."* | False in three clauses. `get_rivers(min_order)` is `lib.rs:7138` and returns aggregated entities carrying `km` and a polyline; `river_at()` is `:7165`; selection is wired at `right_dock.gd:412,466`. **And the file it cites as agreeing has already corrected itself** — `right_dock.gd:1601-1604` states in its own source that "every clause of it is now false". `infrastructure_workspace.gd:158` declares this note the **single owner** of the disclosure, so this one wrong string is the only thing a user reads. | **highest** |
| **b-2** | `cartography_workspace.gd:506-512` (the `DccWidgets.note(sec,` call at `:506`, string body `:507-512` — the earlier cut's `-513` overran by a line) | *"Show rivers in biome view (#showRivers) and Rivers as ways: both are reference RENDER filters over a river network **that never crosses the GDExtension boundary** — cartalith-hydrology computes it internally and only the finished raster comes out (**there is no get_rivers()**)."* | Same falsehood, second site. The network crosses the boundary and `get_rivers()` exists. The trailing sentence — "Same entity gap the Rivers subject and the right dock's River context both already report" — compounds it by pointing at two places that no longer report any such gap. | high |
| **b-3** | `layers_popover.gd:77-79` (the `"popdensity"` key at `:77`, string `:78-79`; the transcription comment naming its Rust source is `:75-76`) **and** `sample_bridge.rs:724` (the `LAYER_GROUPS` hint the popover transcribes) | *"Never available: a missing composite. **No regional population-density estimator exists in this engine.**"* | False. `cartalith_civ::estimate_regional_density_km2` is `cartalith-civ/src/lib.rs:1061`, golden-tested (`golden_parity_carrying_capacity.rs:130`), and **already reached from a shipped `#[func]`** — `ops_bridge.rs:169`, inside `civ_regional_population()`, which builds the full per-cell `dens` field and then integrates it away to return a world total. The estimator exists; what is missing is a path that keeps the field instead of discarding it. Two files carry one string and the Rust one is the source, so **both must change together**. | high |
| **b-4** | `place_editor_window.gd:569-572` (the `DccWidgets.note(sec,` call at `:569`, string `:570-572` — **drifted 3 lines** since the earlier cut wrote `:572-574`, which now lands on the closing paren and past it) | *"Both overrides are stored and neither is consumed: their only readers are `_umInferAge`/`_umWallSpec` in the urban-morphology layer, **which milestones 8-17 have not ported** (URBAN_MORPHOLOGY_SCOPE.md)."* | **The exact shape the 2026-09-02 cut caught on Settlement diagnostics, in a different file.** Both are ported: `cartalith_civ::urban_adapter::um_infer_age` and `cartalith_civ::military::um_wall_spec` (`military.rs:102`), the latter with ~15 assertions pinning its rungs; `age_override` is even *read* there (`military.rs:113`). The **conclusion is still true** — neither override is consumed — but for a completely different reason: `urban_adapter.rs:1620,1626` hardcodes `walls_override: None, age_override: None` because the adapter has no per-settlement override source crossing the boundary (`:1141` says as much). A reworded reason that is still false is worse than the original; this one must be **replaced**, not softened. | high |
| **b-5** | `tool_bar.gd:609`, under the label at **`:605`** (the `_bar_hint(row, "— 4 canvas options unbuilt",` call; the earlier cut's `:606` is one line low) | *"units ▸ km: the canvas itself says this inherits the app-wide unit switch (the reference's `_setUnits`, line 13722); **no such preference exists in this shell yet, so every reading is km**."* | False in both clauses. `Preferences ▸ Units` is live with km / mi / **nmi** (`menus.gd:2055-2062`), and the measure readouts this hover sits above route through `DccUnits` (`right_dock.gd:2044,2055,2073,2085,2094,2191-2192`). It is now **three** unbuilt canvas options, not four — and the count is in the label, so **`:605` changes with the string at `:609`**; fixing one without the other leaves the hover disagreeing with its own heading. | medium |
| **b-6** | `cartography_workspace.gd:642-649` (the `DccWidgets.note(gaps,` call at `:642`, string `:643-649`; the earlier cut's `643-650` is one line low at both ends) | *"Declutter budget · still not built. **Label and icon collision is not resolved anywhere — overlapping annotation simply overlaps.**"* | False since `45b368d`/`0bba2f9`. Label-on-label collision is resolved by `label_cull_rect` and reported as `%d drawn · %d culled` (`cartography_workspace.gd:1852`); icon-on-label collision is resolved by the placement pass's `avoid label boxes` rule, whose own tooltip 1 300 lines below reads *"Measured with the same culler the labelling pass uses"* (`:1962`). **The earlier cut said this note was "written 2026-08-25 (`61ebb00`) and never revisited". That is false, and the truth is worse.** `git log -S` on the sentence itself returns `61ebb00` (2026-08-25) for its introduction, but `git log -S "2026-09-03 raster stack did not do"` on the *same note* returns **`0bba2f9` (2026-09-03 13:07)** — the note was edited on the day the culler landed, a CA-04 clause was added to it, and the false first sentence was left standing directly above the new true one. It was not overlooked; it was read past. The *rest* of the note (a declutter budget needs a per-layer zoom range, which needs the annotation overlays to be stack rows) is still true — `cartography_workspace.gd:486-500` argues the same split honestly and currently — and should survive the edit. | medium |
| **b-7** | `world_workspace.gd:159` (the `STAGES` row for **08 Climate**, its `"gap"` value), drawn to the user by `_build_stage_meta()` at `world_workspace.gd:1031-1032` | *"Seasons and Köppen-Geiger classification **are not ported**."* | False, and it names a whole ported crate module. `cartalith-climate/src/koppen.rs` opens *"Seasons and Köppen–Geiger classification — `computeTempInto`/`computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor`"* and exports all five (`compute_seasons` `:286`, `classify_koppen` `:132`, `build_koppen` `:232`, `koppen_color` `:71`, `compute_temp_into` `:97`). It is golden-tested — `crates/cartalith-climate/tests/golden_parity_koppen.rs`, including `build_koppen_matches_the_reference_over_its_own_seasonal_fields` (`:123`) and three mutation-shaped `assert_ne!`s at `:223-228`. **And it already crosses the boundary into a live control**: `sample_bridge.rs:143` imports `compute_seasons`/`koppen_color`, `:607` registers the `"koppen"` layer under Climate with its own hint, and `:1080-1085` builds its Peel-et-al. legend — so a user can draw the Köppen field from Layers today while this row tells them it does not exist. **The aggravating detail is one line up.** The Erosion row's `"gap"` immediately above (`:151`) is a 2026-08-30 correction that opens *"it was stale on six of its seven claims"* — the same table was audited, the Climate row was not re-checked, and it is now the stale one. **Provenance, measured:** the string is `7f5e54c` (2026-08-18); `koppen.rs` is `b7a46a7` (2026-08-23). True for five days, false for sixteen. What is arguably true and should replace it: seasons are computed **on demand when the layer is picked**, not during stage 08, and no `PARAMS` dial exposes `KoppenParams`. | **highest** |
| **b-8** | `performance_window.gd:140` (a `DccWidgets.note` at the foot of the Performance window body) | *"Devices, multi-GPU mode and VRAM budget: see Preferences ▸ Performance — **no per-device enumeration exists in cartalith-gpu** (GPU_LAYER_INTEGRATION_SCOPE.md)."* | False. `cartalith_gpu::enumerate_devices` is `multi.rs:378`, and that module's own capability table at `:13` reads *"Devices \| **real** — [`enumerate_devices`] lists every physical GPU with name, type, backend and limits, and [`set_preferences`] picks which one(s) dispatch runs on."* It is bound as `WorldGen::gpu_enumerate_devices` (`lib.rs:3998`), documented as the source `menus.gd::_active_backend` reads (`lib.rs:4173`), and used by `compute_config_bench.rs:92,174`. **Two files in this shell make the same claim and only one of them is right**: `menus.gd:1826` says *"This GDExtension build predates the multi-GPU API (`WorldGen.gpu_enumerate_devices` is missing)"* — a **build-conditional**, which is the true statement and the one this document already files under "not gaps". `performance_window.gd` states the same condition as a permanent absence in the engine. **Provenance, measured:** the string is `595582d` (2026-08-19); `multi.rs` is `0de790a` (2026-08-20) — *"Multi-GPU: enumeration, device selection, split tiles, VRAM budget"*. It was falsified **the next day** and has stood for fourteen. Replace with `menus.gd:1826`'s wording, or drop the clause and keep the pointer. | high |
| **b-9** | `civilization_workspace.gd:5405`, inside `_build_politics_gaps()` (`:5399`, its `DccWidgets.note` at `:5401-5406`) — **re-cited at close of pass: this was `:5257` two hours earlier in the same session, see "Line numbers drifted while this cut was being written" below** | *"A recorded year snapshots which settlements exist and who holds which cell; it records no relation between two factions, **because cartalith-civ has no such relation to record at any year**."* | The b-4 shape: the conclusion is true, the reason is false. `cartalith-civ/src/relations.rs` is CV-26's own module — the register ID this very note cites — and it exists precisely to create the edge the note says does not exist: *"the register's structural objection was the real one … so this module creates that edge"* (`:6-10`). `FactionRelation` is `:79`, `civ_faction_relations` is `:224`, and it is bound at `civ_military_bridge.rs:624`. **Three surfaces in this app already draw it**, one of them 330 lines above this note in the same file: `_build_relationships()` (`civilization_workspace.gd:5066`, pulling `bridge.civ_faction_relations()` at `:5071`), the right dock's RL-01 faction relations (`right_dock.gd:1861`, `_build_faction_relations(body)`), and the culture note at `civilization_workspace.gd:2224`. **The same file already words this correctly** at `:5120-5126`: *"Treaties · vassalage · diplomacy actions · change over time · needs a decision"*, closing with *"The standing between every pair is derived and live above."* That is the replacement text, and `relations.rs:14-22` is its source — a derived, recomputed relation with **no stored state and no transition over time**. The true gap is that nothing snapshots a relation **at a year**, not that no relation exists. | medium |

### Stale source comments — not user-visible, so not §b, but the same defect class

**4 → 5 open, 1 closed.** The `map_overlay.gd` constants block accounts for
three of the five on its own, which is the point of listing them separately:
they are all in the comment that exists to keep a Rust/GDScript pair findable.

| Where | What it says | Why it is false |
|---|---|---|
| `cartography_workspace.gd:1436-1442` | *"The fourth is still absent, deliberately … **so the toggle stays disabled** and now says the narrower true thing."* | The toggle is live at `:1660`, and that build site is itself annotated "**Live.**". Written 2026-09-02, when it was true; the culler landed 2026-09-03. |
| `cartography_workspace.gd:2221-2223` | *"It sets the halo and tracking it draws with … and its priority **once the collision culler lands**."* | It landed. |
| `map_overlay.gd:523` | *"`EngineBridge.label_glyph_layout`, `engine_bridge.gd:2469 func label_glyph_layout`"* | It is at `engine_bridge.gd:3211`. This is the open Small row's own cited location. |
| `map_overlay.gd:554` | `ARC_STRAIGHT_THRESHOLD := 0.01 ## labels.rs:150` | **New this pass.** `pub const ARC_STRAIGHT_THRESHOLD` is `labels.rs:164`. The value is right; the pointer is 14 lines off. |
| `map_overlay.gd:555-556` | `ARC_RADIUS_FLOOR_K` / `ARC_SPREAD_DIVISOR`, both `## labels.rs:176` | **New this pass.** `arc_label_layout` opens at `labels.rs:182`. Both values are right. Three citations in one block, written expressly so `grep` would find the pair, and all three now miss. |
| ~~`lib.rs:6704-6705`~~ | ~~`paint_set_brush`'s doc says hardness/softness are "never consumed"~~ | **CLOSED this pass.** `paint_set_brush` moved to `lib.rs:8883` and its doc at `:8863-8866` now reads *"`hardness`/`softness` (0..1, **consumed since `DECISIONS.md` §7k** — a deterministic probability-threshold band feathers the disc's edge; `1.0`/`0.0` is bit-identical to the old hard disc)"*. Disclosed 2026-09-01, carried two cuts, fixed. |

### (c) Reason true, but the presentation misleads — 0 remain

All nine closures of the 2026-09-01 cut hold. Two improved further on the first
2026-09-03 pass and are worth naming as the pattern all nine §b entries should
be fixed against — with `render_workspace.gd:550-568` (in the verified-TRUE list
below) as the third and best worked example, because it is the only one that
*re-derived* what was left instead of trimming the old sentence:

- **The right dock's River Actions** went from three disabled buttons to two,
  because the third promised three things that are now rows above it. It was
  **removed rather than re-labelled with a new pretext**, and the file says so.
- **The CARTO Labels `Class` dropdown** stopped being a disclosed non-defect by
  becoming genuinely live: it now re-seats three live dials rather than three
  inert ones, and the re-entrancy that creates is written down at
  `_sync_label_class()` rather than defended with a guard nothing else uses.

---

## Not gaps — recorded so they are not re-listed next time

**Build-conditional `_todo`s, not permanent rows.** Re-verified, and the list
**grew**: `menus.gd:1826,1827,1831,1837,1838` (multi-GPU devices/mode, CPU
worker threads, VRAM budget, VRAM fallback), `:1230` (no landmark vocabulary),
`:2102` (Try the GPU again), `:3048,3055,3090` (atlas export/import/cap),
`:3631` (Forget layout with none saved), `:3865` (Documentation with no
`res://` beside the build), `:712` (Redo on a cdylib predating the binding),
and `journey_planner_view.gd:456,1457,2624`. **Any table that lists these flat
is wrong** — each is enabled the moment the native library is rebuilt or the
condition changes. `menus.gd:1831` in particular is *not* the CPU worker
threads row; that row is built at `:2416`.

**Presented, permanently inert, and correct.** `Follow system`
(`menus.gd:1889-1893`); `Alternate frames` / `Reduce working res`, enforced
engine-side; the Working set / VRAM estimate readouts, indexed `"readout"`.

**Deliberate omissions, argued in code.** CIVIL POI, WORLD palette
Sculpt/Freehand, `Data ▸ Conversion` (the owner's 2026-08-20 removal still
holds). Not exhaustively re-walked this cut.

**Verified-TRUE declines, found while sweeping for false ones.** Recorded
because a sweep that reports only its hits reads as though everything it
touched was wrong:

- `civilization_workspace.gd:1967-1971` — "Province-level … has no binding".
  True: `get_provinces()` (`lib.rs:7207`) carries name, faction and
  `capital_settlement_index`, and no settlement belongs to a province in
  anything crossing the boundary.
- `civilization_workspace.gd:5243-5246` — "the OLD snapshot's settlement data,
  which no `#[func]` exposes yet". True: `civ_year_diff()`
  (`lib.rs:13385-13388`) returns three `PackedInt64Array`s of tids, nothing else.
- `right_dock.gd:1290-1294` — "no `#[func]` evaluates the cost surface
  pointwise". True; and `:1296-1300`'s "no row-slice `#[func]`" is true too.
- `place_editor_window.gd:299` — "`assign_territory` runs inside `generate()`
  and no `#[func]` re-runs it". True: no recalculate-territories `#[func]`
  exists.
- `asset_library_window.gd:2815` — "removing one has no binding yet". True.
- `menus.gd:3028` / `world_workspace.gd:897` — "nothing reads the cache at draw
  time yet". True: no atlas lookup on the draw path.
- `render_workspace.gd:1437-1441` — "Godot's compatibility renderer does no
  colour management and there is no hook to convert a colour on its way to the
  screen." True **and correctly scoped**: this is about the overlay `Control`s
  and the interface chrome, not the engine's raster, which `apply_color_space`
  *does* re-encode. Easy to misread as contradicting the colour-management
  closure; it does not.
- `layers_popover.gd`'s `oro`, `velo` and `siteprofile` gap sentences: all
  three true. `siteprofile` is the *raster composite*, still with no Rust
  equivalent beyond its two inputs; the settlement-diagnostics **card** that
  landed this cut is a different surface and does not close it. **Its
  `popdensity` sentence, sitting between them, is b-3.** Four sentences from
  one constant, three right and one wrong, is why this sweep opens every symbol
  rather than sampling.

**Added by the second pass, 2026-09-03.** Each was opened because it names a
crate or a Rust symbol and asserts an absence — the query that produced b-7,
b-8 and b-9. These are the ones that held:

- `data_manager_window.gd:129` — *"No GeoJSON import path exists.
  `cartalith-engine::geojson` is write-only."* True: the module's whole public
  surface is `stringify` (`geojson.rs:72`), `territory_feature` (`:244`),
  `province_feature` (`:269`), `export_geojson` (`:295`) and
  `feature_collection` (`:301`). Nothing parses.
- `render_workspace.gd:557-568` — *"Not bound, because the engine has no such
  stage: minor channels, season blend, and the river-band and biome-blend legs
  of the reference's three SDF layers."* True, and **it is the model the nine
  §b entries should be fixed against**: the comment above it (`:550-556`)
  records that three of the stages it used to list shipped on 2026-09-03, says
  so in `MISTAKES.md`'s own vocabulary, and then *re-checks what is left rather
  than inheriting it* — `minorStreams` and `season` return zero hits in
  `render.rs`, and `buildRiverSDF`/`buildBiomeBoundaryDist` are named as
  unported by `render.rs`'s own module doc (`:30`, `:37`) and again at `:1542`.
  Confirmed independently this pass.
- `place_editor_window.gd:440` — *"the reference's `_civApplyFoodShedCeilings`
  is not ported, so nothing here shrinks the settlement to fit."* True:
  `civ_food_shed` (`trade.rs:732`) computes the ceiling and the only mention of
  `_civApplyFoodShedCeilings` in the workspace is `trade.rs:727`, a doc comment
  citing it for a threading decision. No port, no caller.
- `civilization_workspace.gd:4972-4978` and `infrastructure_workspace.gd:590-596`
  — the two `needs a decision` notes. Both true, both correctly scoped, and
  both say what *is* live in the sentence after the gap. `relations.rs:18-22`
  is the Rust side agreeing in its own words.
- `world_workspace.gd:132` — the Geoid/tides row. True, and unusually careful:
  it separates a missing sub-system from a *present* one whose enable is
  another toggle, and names the exact thing that is unexposed
  (`PlanetParams` carries no moon roster).

---

## Owner questions

1. **Does `rdExtraMode()` replace the right dock's selection contexts, or sit
   beside them?** **ANSWERED and executed, 2026-09-03.** Owner ruling:
   *"Selection wins; the tool appends a section."* Implemented as
   `_append_tool()` over four `TOOL_*` section ids derived from
   `app.armed_tool`. No longer an open question or an open row.
2. **What should `statusMid`'s `repaint NN ms` measure?** **Still open**, and
   now the only owner question blocking a row.
3. **Should the WORLD left-dock A/B switch come back?** **Still open.** No
   `ldSwitch`/`ldSwA`/`ldSwB` in `world_workspace.gd`.
4. **How do the design's four icon placement families map onto the engine's
   three?** **ANSWERED and now BUILT.** A fourth family was created:
   `PlacementFamily::SeaMarks`, with its own `snap sea marks to coast` rule and
   a `sea_mark_gap()` spacing. Closed with the CARTO Icons row.
5. **Paint falloff: bind it, or delete the sliders?** **Answered and built**
   (2026-09-01). Not an open row.
6. **Should a committed sculpt stamp re-evaluate when sea level moves?**
   **Still open.** `SculptStamp::with_sea_level` still has no caller.
7. **Are the four unwritten save slots deliberate or an oversight?**
   **Answered and fully executed**, plus the fifth slot
   (`annotations/measurements.json`).
8. **Is `init_gpu_f64` kept or deleted?** **Still open.**
9. **Is the phone app bar's `☰`/`▤` pair now stale?** **Still warranted for an
   owner look**, unchanged from the 2026-09-02 cut: the shell cites
   `DCC_SHELL_SPEC.md` §13 as authority for keeping them, and that same file
   marks the cited passage superseded at `:1014-1017`.
10. **`--good` and `--accH`.** **Still open**, unchanged.

### Left undetermined by this pass — work someone must do, not decisions to take

Every item below needs a run, a capture or a device. None can be settled by
reading code, which is all this cut did.

- Whether the CARTO Labels and Icons panels read correctly now that they are
  **live rather than inert**. The `_mark_inert()` dimming and `_dead_slider()`
  fallbacks still exist for a cdylib without the bindings, and nobody has
  looked at either state since the panels went live. Needs a capture — and per
  `MISTAKES.md` that capture must **force** its palette and refuse to run
  otherwise, because this machine boots light.
- Whether the phone's measure strip / label bar / way card count as missing.
  Needs a handset run.
- The 44 dp vs 48 dp target sweep. Needs a measured run on a device.
- Whether `sculpt_stroke_point` can reject a point the shell already appended.
  **Unchanged** — `sculpt_stroke_point_count()` is still a plain counter.
- Whether the `right_dock.gd` River context is reachable. **No longer
  undetermined, and no longer unreachable** — `_on_map_clicked_river()`
  (`right_dock.gd:466`) is wired to a real viewport click through `river_at()`.
  The comment that said it had "no live trigger today" is gone, correctly.
- Landscape composition beyond the sheet handle. Not re-checked this cut.
- Whether any `_todo` reason cites a `PARITY_AUDIT.md` section number that has
  since moved. Not re-checked at either 2026-09-03 pass.
- **Whether the same false-absence sweep finds more in the files the
  `cartalith[-_]` query cannot reach.** It only catches a sentence that names a
  crate or a Rust path. A sentence that says *"there is no way to …"* without
  naming a symbol is invisible to it, and b-1's *"there is no `get_rivers()`"*
  only surfaced because it happened to name the function. The 26-hit query is a
  floor on this defect class, not a census — and b-8, false for fourteen days in
  a window nobody re-swept, is the evidence that the floor is well below the
  real count.
- Whether the rest of `STAGES`' `"gap"` strings hold. There are **ten**
  (`world_workspace.gd:132, 136, 140, 144, 147, 151, 155, 159, 163, 167`; `:147`
  is empty). All ten were *read* this pass and **three were run down to their
  Rust**: `:151` (Erosion — a 2026-08-30 self-correction, still true), `:159`
  (Climate — **b-7**), and `:132` (Planet/geoid — true). The other six were read
  and found plausible, which is not the same as verified: `:144`'s claim that
  `generate_terrain` hardcodes `0.16 / 1.0 / 0` for `foldI`/`trenchD`/`faultB`
  is a **pinned-constant claim in prose** and is exactly the kind this project
  has been wrong about before. Someone should open `OrogenyParams`' call site.
