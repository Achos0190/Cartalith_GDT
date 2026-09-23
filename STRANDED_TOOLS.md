# STRANDED_TOOLS.md — tools with an engine and no surface (closed 2026-08-19)

**What this is:** the 2026-08-18 report that found seven tools with a working,
golden-verified Rust engine and nowhere in the DCC design to invoke them, plus
one half-surfaced — and where each went when the design answered the next day.
**What it is not:** status, or a description of the shell today. It closed on
2026-08-19 and is history. Each tool's binding, arming, draft and rendering as
built is `UNIFIED_TOOL_PLAN.md`'s "Milestone F as built"; status is
`cartalith-native/docs/STATUS.md`.

Code comments and two user-facing strings cite this file by row number
(`rows 4-8`, `rows 10 and 12`, `row 11`), so the table keeps its numbering.

---

## 1. The finding (2026-08-18)

Written against `DCC_SHELL_SPEC.md` (sync 2026-08-18T23:05Z) and
`UNIFIED_TOOL_PLAN.md` milestones A–E. `DCC_CONTROL_INDEX.md` indexes the
design's controls against the engine; this report ran the comparison the other
way — engine capability against the design's controls — which is the direction
that shows what the design leaves out.

**The design had no tool palette.** The previous shell's tool rail (`main.gd`'s
`TOOL_GROUPS`, five groups) had been replaced by the domain rail, which selects
a *workspace*, not a tool, and nothing took over the tool rail's job. Of the
sixteen tools `UNIFIED_TOOL_PLAN.md` defines, the design gave a chooser only to
the six inside §5.2's Sculpt panel; three were viewport modes that need none;
**seven had a working engine and no surface**, and one was half-surfaced.

None of the stranded seven had a `cartalith-godot` binding either — the
GDExtension then exported 44 methods, none for sculpt, stamp, paint, label,
icon, measure or region export — so even the six *specified* Sculpt tools could
not be wired. That made the binding layer, `UNIFIED_TOOL_PLAN.md` Milestone F,
the critical path ahead of any surface.

**What the spec actually contained, per stranded tool** (the evidence, not an
assertion):

- **Biome paint** — `biome` appears four times: generation stage 09, a Sample
  readout, a surface-mode hotkey, and §7's rule that Cartography may not alter
  biome classification. All four are *views of* biome; none is a brush, though
  `paint.rs` has a whole override layer with its own golden suite.
- **Place settlement** — the CIVIL docks read (*"Settlements, population,
  economy, politics, culture"*; a Settlement inspector). `civ_drop_place`
  writes, and had no caller.
- **Draw route / way** — §6's Route inspector inspects routes that exist;
  `ManualWay` and the four `RouteMode` variants make new ones.
- **Territory / faction paint** — `territory` appears once, as a Faction
  inspector field. `merge_territory_paint` had no surface at all.
- **Label** — a Cartography *layer* row (visibility, opacity), and Edit ▸
  Cut/Copy/Delete operating on *"labels, icons, places, stamps"*: **the verbs
  exist, the noun cannot be created.** `labels.rs` (arc layout, zoom scaling,
  hit boxes, handles, resize) had no authoring path.
- **Icon stamp** — §2.3 and §8 specify the asset library, the icon families and
  pack handling; `manual.rs` supplies arming, placing, hit-testing and
  resizing. Nothing connected the two halves.
- **Measure** — zero mentions in this sense (both matches are the English
  verb). `measure.rs` ships `measure`, `measure_path` and `cell_km`.
- **Region select / export** — §9's export route takes *world bounds* as a
  typed field; `export_region_tiles` and `extract_region_as_world` both need a
  rectangle somebody has to draw.

## 2. The sixteen tools, and where each went

The design revision imported at sync 2026-08-19T00:20Z added **§4.5 Tool
palette**, which gave every row a home. The "2026-08-18" column is the
report's finding, kept as evidence; none of it describes the shell today.

| # | Tool | Engine (2026-08-18) | 2026-08-18 | Home in §4.5 (2026-08-19) |
|---|---|---|---|---|
| 1 | Select / inspect (`V`) | the shell's own selection | mode, no chooser needed | §4.5.1 Inspect (`V`) — named as what makes every §6 inspector reachable |
| 2 | Pan (`H`) | viewport navigation | mode, no chooser needed | §4.5.1 Pan, a legend: always available |
| 3 | Point sample (`I` then; `I` is Icon now) | every field exists in `WorldState` | readout surfaced (§6 Sample), no tool | §6 Sample — a context, correctly not a tool |
| 4 | Raise / lower | `sculpt.rs` Freehand `raise`/`lower` | specified (§5.2), unbound | §5.2 Sculpt panel |
| 5 | Smooth | `sculpt.rs` Freehand `smooth` | specified, unbound | §5.2 |
| 6 | Flatten / terrace | `sculpt.rs` `Feature::Plateau` | specified, unbound | §5.2 |
| 7 | Stamp (landform library) | `sculpt.rs`, 13 features, `SculptStamp` | specified, unbound | §5.2 + §6 stamp stack |
| 8 | River / water | `sculpt.rs` river + lake, `sculpt_commit.rs` | specified, unbound | §5.2 + §6 River inspector |
| 9 | Biome paint | `cartalith-spatial/src/paint.rs`, `PaintStamp` | **stranded** | §4.5.2 WORLD, `B` — moved out of Cartography, because §7's presentation-only rule forbids it there |
| 10 | Place settlement | `cartalith-civ/src/tools.rs` `civ_drop_place`, `civ_pick_place_at` | **stranded** | §4.5.3 CIVIL, `S` |
| 11 | Draw route / way | `tools.rs` `ManualWay`, `RouteContext`, `DijkstraPath` | **stranded** | §4.5.4 INFRA, **split into two**: Way (`W`) and Route (`⇧R`), because v2.10 keeps `draw_way` and `route` separate |
| 12 | Territory / faction paint | `tools.rs` `merge_territory_paint` | **stranded** | §4.5.3 CIVIL, `T` |
| 13 | Label | `cartalith-civ/src/labels.rs` | **stranded** | §4.5.5 CARTO, `L` |
| 14 | Icon stamp | `cartalith-assets/src/manual.rs`, `place_manual_icon` | **stranded** | §4.5.5 CARTO, `I` — the library arms it, the tool places it |
| 15 | Measure | `cartalith-spatial/src/measure.rs` | **stranded** | §4.5.1 global, `M` |
| 16 | Region select / export | `cartalith-engine/src/region_export.rs` | half-stranded: export route (§9), no on-map selection | §4.5.1 global, `R` — *"the marquee §9's export route was missing"*, two views of one rect |

§4.5 also added **POI** (§4.5.3, `P`, `_civDropPOI`) — a separate record type
from a settlement, missed here because the report worked from the engine's tool
list rather than the reference's. §12 gained twelve matching glyphs, so the
no-emoji rule still covers the whole product.

## 3. After the resolution — pointers, not status

- **Milestone F** bound rows 4-16 (2026-08-18 → 2026-08-25). `UNIFIED_TOOL_PLAN.md`
  "Milestone F as built" walks every row, including how Measure (six modes) and
  Region select (one marquee → readout → export loop) outgrew §4 below, and one
  recorded residual on Region select's corner handles.
- **POI** has no binding by a Milestone D decision older than Milestone F:
  `_civDropPOI` has no Rust counterpart, and `civ_tools_bridge.rs`'s module doc
  says POI "is not a ported concept". `civilization_workspace.gd::_build_tools`
  omits the button rather than drawing it dead.
- **Where the palette is drawn has moved twice since §4.5; the keys have not.**
  The INFRA domain merged into CIVIL on 2026-08-20, so Way and Route arm from
  CIVIL's tool set (`civilization_workspace.gd::_build_tools`). And **Ruling AK**
  (`LARGE_ITEM_RULINGS.md`, 2026-09-23) moved the palette out of each left
  dock's TOOLS block into an always-visible top bar on desktop and tablet
  (`DccApp._install_tool_palette_bar()`, global cells in
  `DccWidgets.GLOBAL_TOOL_ENTRIES`) — an owner decision over the canvas, which
  still draws the dock block. The phone keeps its own arrangement.

## 4. The proposal — "What I would expect on the UI" (2026-08-18)

Superseded the next day by §4.5, which the owner commissioned from this report.
Kept in summary because `UNIFIED_TOOL_PLAN.md` compares what shipped against
it. Every proposal followed the design's own grammar — a tool lives with the
domain that owns it, the tool options bar carries its frequently changed values,
the right dock inspects what is selected:

| Tool | Proposed home | Options bar | Draft? |
|---|---|---|---|
| Biome paint | WORLD ▸ Sculpt panel, a `PAINT` sibling of the 13 features | biome swatch · radius (default 6) · hardness · Commit / Discard | yes — a `PaintStamp` draft that marks **only** ecology stale |
| Place settlement | CIVIL left dock, a new `EDIT` section | kind (5 tiers) · faction · snap to water · pick radius | no — one record; Undo covers it |
| Draw route / way | INFRA left dock, `EDIT` section | way type · mode (freehand / snap / Dijkstra) · undo/redo · Commit | a polyline until committed |
| Territory paint | CIVIL left dock, `EDIT` section | faction swatch · radius · add / subtract | yes — `merge_territory_paint` is a merge by construction |
| Label | CARTO ▸ `Labels & annotation` layer, plus a canvas tool | text · size mode · arc · anchor | presentation-only; §7 forbids it marking any stage stale |
| Icon stamp | armed by the asset library, placed on the map | family + variant · scale · rotation · scatter rule | presentation-only |
| Measure | viewport-level, every domain, `M` | segment / path · km · running total · Clear | nothing to commit — an ephemeral overlay |
| Region select | a viewport marquee filling §9's `world bounds` | x/y/w/h in cells and km · lock aspect · use as export bounds | a selection; the export route owns the write |

It flagged two consequences, and §4.5 answered both: the `EDIT` sections would
have been new structure (docks that held data gaining creation affordances — a
design decision), which §4.5 avoided with a separate palette; and Measure needed
the first glyph §12 had not specified, which §4.5's twelve new glyphs supplied.
The full 2026-08-18 text is in history: `git log -- STRANDED_TOOLS.md`.
