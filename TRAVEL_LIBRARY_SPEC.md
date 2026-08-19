# Data ▸ Travel library — spec

> **This is an ADDITION to the original DCC GUI.** It is not part of `DCC_SHELL_SPEC.md`
> and does not exist in Cartalith Gen1 v2.10. Nothing in the original shell — menu bar,
> domain rail, docks, viewport, timeline — changes to accommodate it. It adds one entry
> to the existing **Data** menu and one window.

Mockups: `Journey Planner DCC.dc.html` — `2a` (menu item), `2b` (list + inspector).
A table-form alternative was reviewed and deleted.

The menu item opens the window directly (Data ▸ Travel library…, ⇧L). There is no
submenu; the definition types are tabs inside the window.

## 1 · Purpose and scope

An information layer, nothing more. It declares the **classifications and constraints**
the journey planner computes from, so a world can carry its own animals, vehicles,
vessels and party set-ups instead of mis-picking a stock entry that happens to be close.

Deliberately NOT in scope: art, icons, portraits, maps, charts, stat visualisation,
lore or descriptive text. Presentation stays at the density of the Data manager. Nothing
here is a workspace — no route is stored, no plan is computed in this window.

Everything defined here becomes a selectable option in the planner's party form and, for
animals and vehicles, in the per-stage override set.

## 2 · Placement

- **Data** (top menu bar) ▸ **Travel library** ▸ submenu.
- Submenu: Animals & mounts… · Vehicles… · Vessels… · Party set-ups… ·
  New from selected… · New blank definition… · Capture party from planner ·
  Validate constraints · Show usage in journeys · Import definitions .csv… ·
  Reset to stock definitions…
- Opens as its own window (like Data manager / Asset library), tabbed by definition type.
- Menu metrics, hairlines, mono labels and accent behaviour are taken verbatim from the
  shell's Assets menu. No new visual vocabulary is introduced.

## 3 · Fields

Stock entries are read-only; duplicate to edit. A field left unset is *incomplete*, not
zero — the planner falls back to the entry's declared substitute and flags the stage.

### 3.1 Animals & mounts

| group | fields |
| --- | --- |
| Classification | name · role (pack / mount / draft, multi) · substitutes for · size class · availability (global / regional, named region) |
| Capacity & speed | load capacity kg · draft pull kg towed · base speed km/h · sustainable hours/day · forced-pace cap × base |
| Sustenance | fodder need kg/day · water need L/day · grazing tolerance · waterless limit days |
| Terrain constraints | per-terrain multiplier or `blocked` (plains, steppe, forest, hills, mountain, marsh, desert, high pass, snowfield, river ford) |
| Requirements & prohibitions | yokeable to wheeled vehicles · requires road/track to tow · blocked by seasonal closures · carryable aboard a vessel · usable as a mount · handlers required per N head |
| Cost | upkeep sp/day/head |

### 3.2 Vehicles

class (wheeled / dragged) · load kg · draft head required (count + role) · speed ×
· road requirement (none / track / road) · off-road multiplier or `blocked` · ford
multiplier or `blocked` · carryable aboard a vessel.

### 3.3 Vessels

mode (river / sea, multi) · hold kg · crew required · base speed · water rating
(sheltered / coastal / open) with `blocked` beyond it · sailing window (daylight /
continuous) · portage-capable.

### 3.4 Party set-ups

One row = one preset of **party-form values only**: transport · group size · cargo kg ·
pace · hours/day · supplies carried · animal counts by species · vehicle counts by type ·
grazing · foraging · season defaults. No route, and applying a set-up leaves per-stage
overrides untouched. "Capture party from planner" writes the current form into a new row.

## 4 · Validation

The only interactive weight the window carries. Three states, shown in the list and per
entry: **ok**, **incomplete** (a constraint field is unset), **conflicting** (e.g. grazing
tolerance restricted to grassland while non-grassland terrain multipliers are non-zero).
Each entry also reports usage — how many saved journeys and set-ups reference it — and
warns that editing capacity, fodder or a constraint re-plans them.

## 5 · Why the constraint fields are mandatory

Blocked stages, the "faster mode available" advisory and the better-animal/vehicle
advisory in the planner are all derived from these same fields. An entry without them
would not merely look unfinished — it would plan silently wrong.
