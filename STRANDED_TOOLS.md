# Stranded tools — RESOLVED by the design's §4.5 Tool palette

> **Status: closed, 2026-08-19.** This document reported that seven tools had a
> working, golden-verified Rust engine and nowhere in the DCC design to invoke
> them, plus one half-surfaced. The owner took it to the design project, and the
> revision imported at sync 2026-08-19T00:20Z adds **§4.5 Tool palette** — which
> gives every one of them a home, adds a tool the report had missed, and splits
> one row into the two tools the reference actually keeps separate.
>
> | Was stranded | Now lives at |
> |---|---|
> | Measure | §4.5.1 global tool, key `M` |
> | Region select / export | §4.5.1 global tool, key `R` — *"the marquee §9's export route was missing"*, two views of one rect |
> | Biome paint | §4.5.2 WORLD tool, key `B` — moved out of Cartography, because §7's presentation-only rule forbids it there |
> | Place settlement | §4.5.3 CIVIL tool, key `S` |
> | Territory paint | §4.5.3 CIVIL tool, key `T` |
> | Draw route / way | §4.5.4 INFRA — **split into two**, Way (`W`) and Route (`⇧R`), because v2.10 keeps `draw_way` and `route` separate |
> | Label | §4.5.5 CARTO tool, key `L` |
> | Icon stamp | §4.5.5 CARTO tool, key `I` — the library arms it, the tool places it |
>
> Also added, and absent from this report because the report worked from the
> engine's tool list rather than the reference's: **POI** (§4.5.3, key `P`,
> `_civDropPOI`) — a separate record type from a settlement. And **Inspect**
> (`V`) is named as the thing that makes every §6 inspector reachable at all,
> which the previous revision left implicit.
>
> §12 gained the twelve matching glyphs, so the no-emoji rule still covers the
> whole product. §3 now states that every left dock opens with the TOOLS block
> and that the armed tool survives a workspace switch.
>
> **The engine question this raised is still open**, and is the real remaining
> work: none of these tools has a `cartalith-godot` binding. Sculpt now does
> (34 methods, 2026-08-19), which is the template for the rest. Milestone F is
> partially done, not done.
>
> **Update, 2026-09-01: that engine question is closed, and the paragraph
> above is stale.** It described 2026-08-19, the day Sculpt's binding landed
> and nothing else's had yet. By 2026-08-25 the rest had: every one of the
> seven STRANDED rows and the one half-stranded row below now has a real
> `cartalith-godot` binding behind a real tool-rail control, arming, storing
> and rendering it — enumerated tool by tool, with file:line citations, in
> `UNIFIED_TOOL_PLAN.md`'s new **"Milestone F as built"** section. Nobody
> corrected this document when that happened, which is the exact defect
> `OUTSTANDING_WORK.md` §1 ("Milestone F's own closeout") was opened to fix.
>
> In brief, so a reader who stops here still gets the truth: **Milestone F is
> done.** The "Status of all sixteen" table below is left exactly as written
> on 2026-08-18 — it is the record of the investigation that found the gap,
> and every one of its "Bound to Godot: ✗" and "State: STRANDED" marks is
> now wrong, dated evidence of a state that no longer holds, not a live
> status. Same for "What blocks all of it regardless"'s claim that the
> GDExtension "exports 44 methods... and not one sculpt, stamp, paint,
> label, icon, measure or region-export method" — false as of this update —
> and for "Recommendation" item 2, which is done. The one exception the
> table did not anticipate: the design's own later revision added a
> seventeenth tool, **POI**, which this port still declines to bind — by a
> Milestone D decision older than Milestone F itself, not an omission (see
> `UNIFIED_TOOL_PLAN.md`'s new section for the three-file citation trail).
> And one small, honestly-drawn loose end survives: Region select's corner
> handles are drawn but a drag-resize is not wired to them (same section,
> "One honest residual") — dragging a fresh marquee still reaches the whole
> export loop correctly, only the handle shortcut does not.
>
> The original report follows unchanged, as the record of how the gap was found.

---

> Written 2026-08-18 against `DCC_SHELL_SPEC.md` (sync 2026-08-18T23:05Z) and
> `UNIFIED_TOOL_PLAN.md` milestones A–E. Companion to `DCC_CONTROL_INDEX.md`,
> which indexes the design's controls against engine capability; this document
> runs the comparison the other way — engine capability against the design's
> controls — and finds the shortfall that direction hides.

The instruction was *"replace the current GUI and replace it in full by the DCC
version including all its wiring and functionality."* That is buildable for
everything the design specifies. It is not buildable for tools the design does
not specify, and there are eleven of those in one state or another. This
document records them rather than inventing UI for them.

## The finding in one sentence

**The DCC design has no tool palette.** Of the sixteen tools `UNIFIED_TOOL_PLAN.md`
defines and milestones B–E built, the design gives a chooser to exactly the six
that live inside §5.2's Sculpt panel; three more are viewport modes that need no
chooser; and **seven have a working Rust engine and nowhere in the shell to
invoke it** — plus one that is half-surfaced.

The previous shell had a tool rail (`main.gd`'s `TOOL_GROUPS`, five groups).
This revision removed it and replaced it with the domain rail, which selects a
*workspace*, not a tool. Nothing took over the job the tool rail was doing.

## Status of all sixteen

Engine column cites the file that actually implements it. "Bound" means a
`cartalith-godot` GDExtension method exists — **none of the stranded seven have
one**, so each is two layers away from usable, not one.

> **This table is dated 2026-08-18 and preserved as written.** Every `✗` and
> every `STRANDED` below describes that day, not today: as of 2026-09-01
> every row is bound (rows 1-3 correctly still show no binding — they never
> needed one) except **POI**, a seventeenth tool the design added later and
> this port still declines to bind by design. See `UNIFIED_TOOL_PLAN.md`'s
> "Milestone F as built" for the current, verified state of each row and why
> this one stayed stale for two weeks.

| # | Tool | Engine (built, tested) | Bound to Godot | Design surface | State |
|---|---|---|---|---|---|
| 1 | Select / inspect (`V`) | selection is the shell's own | n/a | §6 "contents follow the selection" | **Mode, no chooser needed** |
| 2 | Pan (`H`) | viewport navigation | n/a | implicit | **Mode, no chooser needed** |
| 3 | Point sample (`I`) | all fields exist in `WorldState` | partial | §6 Sample context, 16 fields | **Readout surfaced, no tool** |
| 4 | Raise / lower (`B`) | `sculpt.rs` Freehand `raise`/`lower` | ✗ | §4 default context, §5.2 | **Specified** |
| 5 | Smooth (`S`) | `sculpt.rs` Freehand `smooth` | ✗ | §5.2 sub-mode | **Specified** |
| 6 | Flatten / terrace (`F`) | `sculpt.rs` `Feature::Plateau` | ✗ | §5.2 feature | **Specified** |
| 7 | Stamp (landform library) | `sculpt.rs` 13 features, `SculptStamp` | ✗ | §5.2 + §6 stamp stack | **Specified** |
| 8 | River / water (`R`) | `sculpt.rs` river + lake, `sculpt_commit.rs` | ✗ | §5.2 + §6 River inspector | **Specified** |
| 9 | **Biome paint (`P`)** | `cartalith-spatial/src/paint.rs`, `PaintStamp` | ✗ | none | **STRANDED** |
| 10 | **Place settlement** | `cartalith-civ/src/tools.rs` `civ_drop_place`, `civ_pick_place_at` | ✗ | none | **STRANDED** |
| 11 | **Draw route / way** | `tools.rs` `ManualWay`, `RouteContext`, `DijkstraPath` | ✗ | none | **STRANDED** |
| 12 | **Territory / faction paint** | `tools.rs` `merge_territory_paint` | ✗ | none | **STRANDED** |
| 13 | **Label (`T`)** | `cartalith-civ/src/labels.rs`, 886 lines | ✗ | layer visibility only | **STRANDED** |
| 14 | **Icon stamp** | `cartalith-assets/src/manual.rs`, `place_manual_icon` | ✗ | library arms it; nothing places it | **STRANDED** |
| 15 | **Measure (`M`)** | `cartalith-spatial/src/measure.rs` | ✗ | none — zero mentions in the spec | **STRANDED** |
| 16 | Region select / export | `cartalith-engine/src/region_export.rs`, 565 lines | ✗ | export route exists (§9); on-map selection does not | **Half-stranded** |

Seven fully stranded, one half, three chooser-less modes — eleven tools touched
by the gap, which is the number the control index reported.

## Evidence, per stranded tool

Not assertions — what the spec actually contains.

**Biome paint.** `biome` appears four times in the spec: generation stage 09, a
Sample readout field, a viewport surface-mode hotkey (`Biome 2`), and §7's
prohibition on Cartography altering biome classification. All four are *views
of* biome. None is a brush. The engine has a whole override layer built for this
(`paint.rs`, with its own golden-parity suite) and a documented rule that
painting biome does **not** mark height, hydrology or climate stale — a rule
with nothing to govern.

**Place settlement.** The CIVIL left dock is specified as *"Settlements,
population, economy, politics, culture"* and the right dock as a Settlement
inspector. Both read. `civ_drop_place` writes, and has no caller.

**Draw route / way.** INFRA's dock is *"Roads, rivers, ports, trade, logistics"*;
§6's Route inspector carries *"stages, vessels, cost trace, per-stage overrides,
daily stages"* — an inspector for routes that exist. `ManualWay` and the four
`RouteMode` variants make new ones.

**Territory / faction paint.** `territory` appears once in the entire spec, as a
field inside the Faction inspector. Cartography lists a `Political (off)` layer.
`merge_territory_paint` has no surface at all.

**Label.** The richest orphan: 886 lines covering arc layout along a path, font
scaling by zoom, hit boxes, drag handles and resize. The spec gives labels a
Cartography *layer* row (visibility, opacity) and lets Edit ▸ Cut/Copy/Delete
operate on *"labels, icons, places, stamps"* — the verbs exist, the noun cannot
be created. There is no way to author a label.

**Icon stamp.** Half the pipeline is specified: §2.3 and §8 give the asset
library, 24 icon families, and pack handling. `manual.rs` supplies the other
half — arming an icon, placing it at a cursor, hit-testing it, resizing it by
handle. Nothing in the shell connects the two.

**Measure.** The word does not appear in the spec in this sense at all
(both matches are the English verb). `measure.rs` ships `measure`,
`measure_path` and `cell_km`.

**Region select / export.** §9's Data manager has a real export route with tile
scheme, zoom range, CRS and *world bounds* — but bounds as a typed field, not a
marquee dragged on the map. `export_region_tiles` and `extract_region_as_world`
both take a rectangle somebody has to choose.

## What I would expect on the UI

Not built, and not to be built until you say so — this is the proposal the
report exists to let you accept or reject. Every row respects the design's own
grammar: tools live where the domain that owns them lives, the tool options bar
carries their frequently-changed values, and the right dock inspects what is
selected.

| Tool | Where it belongs | Chooser | Tool options bar row | Right dock | Non-destructive? |
|---|---|---|---|---|---|
| Biome paint | WORLD ▸ Sculpt panel, a second tab beside the 13 features | Feature-grid sibling: `PAINT` | `PAINT · BIOME` · biome swatch · radius (cells, default 6) · hardness · ✓ Commit · Discard | Painted-cell count, biome legend, override-layer toggle | Yes — `PaintStamp` draft, commits like a sculpt pass, marks **only** ecology stale |
| Place settlement | CIVIL ▸ left dock, an `EDIT` section above the settlement list | Section action: `＋ Place settlement` | `CIVIL · PLACE` · kind (5 tiers) · faction · snap-to-water toggle · pick radius | Settlement inspector, live, on the placed marker | No pass buffer needed — a place is one record; Undo covers it |
| Draw route / way | INFRA ▸ left dock, `EDIT` section | Section action: `＋ Draw route` | `INFRA · ROUTE` · way type (highway/regional/road/track) · mode (freehand / snap / Dijkstra) · ↶ ↷ · ✓ Commit | Route inspector with the cost trace it already specifies | Draft polyline until committed |
| Territory paint | CIVIL ▸ left dock, `EDIT` section | Section action: `Territory brush` | `CIVIL · TERRITORY` · faction swatch · radius · add / subtract | Faction inspector (already specified) | Yes — `merge_territory_paint` is already a merge, so it is a draft by construction |
| Label | CARTO ▸ Layer properties for `Labels & annotation`, plus a canvas tool | Layer-scoped action: `＋ Add label` | `CARTO · LABEL` · text field · size mode · arc on/off · anchor | Selected-label properties: text, size, arc curvature, handles | Presentation-only — §7 forbids it marking any stage stale, which is correct here |
| Icon stamp | Asset library window arms it (already specified); the map places it | Armed-icon indicator in the tool options bar | `ASSETS · ICON` · armed family + variant · scale · rotation · scatter-rule toggle | Placed-icon properties: family, variant, scale, rotation | Presentation-only |
| Measure | Viewport-level, available in every domain | Status-bar toggle or `M`; §12's rule set has no measure glyph yet, so one is owed | `MEASURE` · mode (segment / path) · units (km) · running total · ✕ Clear | Measurement readout: segment lengths, total, bearing | Nothing to commit — ephemeral overlay |
| Region select / export | Viewport marquee that fills §9's `world bounds` | Drag with `R`, or `Select region` in the Data manager route pane | `REGION` · x/y/w/h in cells and km · lock aspect · Use as export bounds | Region summary: extent, cell count, estimated tiles | Selection only; the export route already owns the write |
| Point sample | Already the default context | none needed — §6's Sample **is** the tool | (no row) | Already specified, 16 fields | n/a |

Two consequences worth stating before anyone builds this:

1. **The four `EDIT` sections above are new structure, not a re-skin.** §3 says
   the CIVIL and INFRA docks hold data; giving them creation affordances changes
   what those docks are for. That is a design decision, and it is yours.
2. **Measure needs a glyph.** §12 fixes the icon rules and enumerates every
   drawn glyph; a measure tool adds the first one the design has not specified.

## What blocks all of it regardless

Every stranded tool needs a `cartalith-godot` binding before any surface can
call it. The GDExtension currently exports 44 methods — generation, parameters,
textures, settlements, roads, sea routes, provinces, trade, quality tiers — and
**not one** sculpt, stamp, paint, label, icon, measure or region-export method.
So even the six *specified* Sculpt tools (rows 4–8) cannot be wired today: §5.2
is fully designed and fully implemented in Rust, with nothing in between.

> **False as of 2026-09-01.** The 44-method count and the "not one... method"
> claim describe 2026-08-18. Sculpt, Paint, Label, Icon, Measure and Region
> export all have `#[func]` bindings now — 34 for Sculpt alone, the same
> figure this document's own top block already named on 2026-08-19 — each
> enumerated with file:line citations in `UNIFIED_TOOL_PLAN.md`'s "Milestone
> F as built".

That makes the binding layer the real critical path, ahead of any of the
proposals above. `UNIFIED_TOOL_PLAN.md` calls this Milestone F ("Shell wiring")
and it is the only lettered milestone still outstanding.

> **Also stale**: Milestone F shipped 2026-08-18 through 2026-08-25 and is no
> longer outstanding. It was the *last* lettered milestone, not because one
> remained after it, but because there is no Milestone G.

## Recommendation

1. Build the shell to the spec, exactly — nothing invented (in progress).
2. Do Milestone F: bind sculpt, stamps, paint, labels, icons, measure and region
   export to Godot. This unblocks rows 4–8 *and* every proposal above.
   **Done, 2026-08-18 through 2026-08-25** — see `UNIFIED_TOOL_PLAN.md`'s
   "Milestone F as built" for the tool-by-tool evidence.
3. Bring this table to the design project and let the rows above be specified
   properly, rather than improvised in code. **Also done** — the top of this
   very document records it: the design's §4.5 Tool palette, imported at sync
   2026-08-19T00:20Z, is that response, giving every one of these tools the
   home this recommendation asked for.
