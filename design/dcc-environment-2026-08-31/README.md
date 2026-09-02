# DCC Environment + Android, imported 2026-08-31

Owner-supplied, imported over the design MCP from the Claude Design project
**UI mockups planning** (`067f80e7-dbb7-4492-8e69-96aaa8050a4d`), with the
instruction: **"Replace the current GUI, do not upgrade. Fully replace."**

| File | What it is |
|---|---|
| `Cartalith DCC Environment.dc.html` | Windows / tablet. Interactive: drag pans, wheel or pinch zooms. Four frames — `WINDOWS 1920`, `LAPTOP 1366`, `TABLET 2560`, `TABLET PORTRAIT` |
| `Cartalith Android.dc.html` | Phone. Interactive: drag, pinch, rotate, long-press, edge-swipe. Four devices — 360, 412, 480, `TABLET 800` |

Both import `landmark-glyphs.js` and `support.js` from the design project.

## The headline change

**The domain rail goes from five to three.** `WORLD · CIVIL · CARTO`, with a
node tree under each header rather than five flat domains:

| Domain | Nodes |
|---|---|
| WORLD | Generation pipeline · Sculpt |
| CIVIL | Landmarks · Factions & settlements · Ways & routes · Journey planner |
| CARTO | Layers & style · Labels · Icons · Terrain appearance |

So **INFRA folds into CIVIL** ("Ways & routes") and **RENDER folds into CARTO**
("Terrain appearance"). That is an information-architecture change, not a
reskin, and it reaches the rail, every workspace, `select_domain_category` and
its call sites, the phone's domain cells and the command index.

The seven menus are unchanged: File · Edit · Assets · Data · Preferences ·
Window · Help.

## `landmark-glyphs.js` is this repo's own glyph set, round-tripped

The 49 glyphs it carries are the ones drawn here on 2026-08-30 and committed to
`shell/dcc_icons.gd` — same path data, including the re-cut `road_junction`,
`spring` and `pilgrimage_site`. Nothing to re-import: the design was built on
top of them.

It draws its own `Cliff`, `Lake` and `Volcanic feature` rather than reusing the
sculpt-feature glyphs, but the paths are near-identical re-scalings, so the
reuse decision in `DccIcons.LANDMARK_REUSE` stands.

## Scope, stated rather than assumed

These two files specify the **shell frame and its information architecture** —
menu bar, rail, docks, tool options, viewport furniture, status, timeline, and
the whole phone shell.

They do **not** specify the dedicated windows: Faction roster, Place editor,
City viewer, Data manager, Vault, Travel library, Asset library. Those keep
their implementations and are re-pointed at the new frame — replacing them would
mean inventing them, which is worse than keeping what is built and wired.
