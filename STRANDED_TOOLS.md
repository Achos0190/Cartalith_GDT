# Stranded tools — engines built, no surface in the DCC design

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

That makes the binding layer the real critical path, ahead of any of the
proposals above. `UNIFIED_TOOL_PLAN.md` calls this Milestone F ("Shell wiring")
and it is the only lettered milestone still outstanding.

## Recommendation

1. Build the shell to the spec, exactly — nothing invented (in progress).
2. Do Milestone F: bind sculpt, stamps, paint, labels, icons, measure and region
   export to Godot. This unblocks rows 4–8 *and* every proposal above.
3. Bring this table to the design project and let the rows above be specified
   properly, rather than improvised in code.
