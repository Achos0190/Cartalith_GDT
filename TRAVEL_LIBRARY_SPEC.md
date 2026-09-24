# Data ▸ Travel library — spec

> **Port note — not part of the vendored spec.** Everything from here to the end
> of §5 is the owner's design spec for this window, imported verbatim on
> 2026-08-19 (`c634110`) with `design/Journey Planner DCC.dc.html` artboards
> `2a`/`2b`, and is not edited apart from the one marked port note in §2. §6 is
> this port's own: how the spec is realised and where the build departs from
> it. Neither is status — that is `cartalith-native/docs/STATUS.md`'s. The
> engine side of the planner this library feeds is `JOURNEY_PLANNER_SCOPE.md`.

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

> **Port note.** Built as the single row this spec's opening paragraph and the
> `2a` artboard describe: `Data ▸ Travel library… ⇧L` opens the window directly
> and there is no submenu (`menus.gd::_data`). Where each listed action went is
> in §6.

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

## 6 · Port notes — how the spec is realised

*This port's section, not the vendored spec.* It replaces the dated build
records that stood here (2026-08-19, -19 and -20); `git log --
TRAVEL_LIBRARY_SPEC.md` has them.

### 6.1 Where it lives

- **Data model** — `cartalith-civ/src/travel_library.rs`: the four §3 types,
  §4 validation, the stock content, and the pure resolver builders
  (`animal_resolver_fns`, `vessel_resolver_fn`). There is no reference
  counterpart and so no golden target; where a stock figure could be grounded in
  an existing golden-tested constant (the four `JP_ANIMAL_KEYS` species,
  `jp_capacity`'s vehicle masses, `jp_ship_stats`) it is.
- **Store** — `cartalith-godot/src/travel_bridge.rs::TravelLibrary`, held on
  `WorldGen`, bootstrapped with stock content at `init()`. It is user-editable
  project state, so a regenerate does not reset it (like the asset pack), and
  the project archive saves its custom half as `library/travel.json`
  (`SAVEFILE_COMPAT.md`; stock entries are rebuilt identically each launch, so
  they are not stored).
- **Boundary** — one `#[func]` dispatch over `kind: "animal"|"vehicle"|
  "vessel"|"preset"` rather than four surfaces: `tl_counts`, `tl_list`,
  `tl_get`, `tl_duplicate`, `tl_add_blank`, `tl_delete`, `tl_reset_to_stock`,
  `tl_edit`, `tl_capture_preset_from_plan`. The field-pairs layer reuses
  `journey_bridge`'s `JpValue` flattening rather than inventing a second one.
- **Window** — `shell/travel_library_window.gd`, its own popup (the mockup's
  "⇧L · own window"), tabbed by definition type. Each tab is a Custom/Stock rail
  (filter, ＋ new blank, ⧉ duplicate, ✕ delete) plus an inspector grouped
  exactly as §3 groups each type. Edits are staged and committed with "save
  definition" (save / duplicate / revert, as `2b`'s footer). The §4 banners use
  the shell's existing `warn`/`water`/`block` tokens — the mockup's own
  `#e0a840`/`#7d9dae`/`#b55950`.

**§2's submenu actions**: *New from selected* is ⧉ duplicate; *New blank*,
*delete* and *Reset to stock* are window actions; *Validate constraints* and
*Show usage in journeys* are shown per entry rather than invoked; *Capture party
from planner* is the planner's `capture party…` (below). *Import definitions
.csv…* is the rail's ⇪ button: `travel_library_window.gd::import_csv` reads a
file picked in a `FileDialog` into the tab on screen — row 1 names columns by
`tl_get`'s field keys, each later row becomes a custom entry through
`tl_add_blank` + `tl_edit`, and refusals are reported by line and key
(`c1e0a2a`, 2026-09-24; this paragraph said no action imported `.csv` until
then).

### 6.2 How a definition reaches a computed journey

**Animals.** `jp_compute` takes `animal_entries` — `{species_key: entry_id}`
— which `TravelLibrary::animal_overrides_selected` turns into the override map
`animal_resolver_fns` builds a `JpAnimalResolver` from, for `jp_plan_ex`. Three
properties are pinned by tests:

- **An absent key changes nothing.** An empty selection reproduces
  `animal_overrides()` exactly, and a stock-only library reproduces the plain
  `jp_plan` with full structural equality
  (`regression_stock_only_travel_library_matches_pre_dispatch_jp_plan`).
- **Naming a *stock* entry means "no override"** — the built-in table — which is
  deliberately not the same as leaving the slot unnamed.
- **A selection that cannot be honoured is rejected, not ignored** (unknown
  species, unknown id, or an entry that resolves to no slot), into
  `jp_compute`'s `rejected` array.

`TravelLibrary::animal_species_slot` is the one place that decides which of the
four built-in species an entry may occupy: its own `species_key`, else the one
its `substitutes_for` chain reaches — bounded by the store's size, so a
user-typed cycle terminates. `tl_list`/`tl_get` expose it as `species_slot`,
beside `usable_as_mount`.

In the party form: four per-species **animal definition** pickers (Carriage ▸
"ANIMAL DEFINITIONS · TRAVEL LIBRARY"), each listing every entry that resolves
to that species, stock first, custom tagged `· custom` and carrying §4's
validation marks. The **Mount** picker is the same list filtered to §3.1's
*usable as a mount*, labelled `<species> › <entry>`; one choice sets both the
engine's `mount_animal` and that species' definition slot. The selection is
journey-wide: a per-stage override (`JpStageOverride`) changes counts, the
mount species and the vessel name, not which definition fills a species slot —
a departure from §1's "and, for animals and vehicles, in the per-stage override
set".

**Vessels.** `TravelLibrary::vessel_overrides()` → `vessel_resolver_fn` →
`JpVesselResolver`, handed to `jp_plan_full` and resolved by **name**, so a
custom hull's speed, hold, crew and water rating drive the water legs. An entry
still missing one of those four is declined by the resolver — it falls back to
the built-in table rather than sail a zero-hold hull — and the planner's Vessel
picker draws it disabled. §3.3 has no per-water-type blacklist field
(`ShipStats::invalid_water`), so a custom vessel is constrained by mode and
water rating only; and §3.3's `sailing window` is not coupled to the engine's
`jp_water_window`, a property of the water type (`JOURNEY_PLANNER_SCOPE.md`,
"The Travel Library and the party form").

**Vehicles are data only.** No resolver exists for `jp_capacity`'s
cart/wagon/sled/travois constants; the party form's vehicle counts are plain
`JpParty` counts, and the window says so on every vehicle.

**Party set-ups** (§3.4). The planner's tool-options bar carries a `set-up`
dropdown over `tl_list("preset")` (stock and captured, custom tagged) and
`capture party…` through `tl_capture_preset_from_plan`. Applying one assigns
only the twenty keys `tl_get("preset", id)` returns — `PRESET_FIELD_KEYS`,
`PartyPreset::apply_to`'s own inverse, so there is no second translation table
to drift — and leaves per-stage overrides untouched, as §3.4 requires. This
replaces the reference's JS-only `JP_PRESETS` with the strictly larger thing:
stock set-ups *and* every set-up a user captures.

**Usage** (§4). A saved journey (story planning's `Journey`,
`STORY_PLANNING_SCOPE.md` SP-1) names a party preset, so a preset's usage counts
journeys as well as set-ups (`InfraTools::preset_usage_in_journeys`). A journey
names no animal, so an animal's journey usage is `0` in the engine by
construction (`TravelLibrary::animal_usage_in_journeys`).

### 6.3 Why `JpParty` stays four fixed species

Only the four built-in species (donkey/mule/camel/horse) can occupy a party
slot. Widening `JpParty` to a generic animal-count map was examined and
declined, for a **spec** reason rather than a mechanical one:

1. **The data to drive a new species does not exist.** `jp_capacity_ex` reads
   `jp_seasonal_animal(season, key)` — sixteen `(cap, food, water)` rows, four
   seasons × four species — and `jp_desert_animal_mod(key)`'s desert food/water
   pair, for every species it sums. §3.1 carries **neither**. A wholly new
   species would silently take the neutral `1.0` fallbacks on both — precisely
   §5's "would plan silently wrong". Closing that means adding fourteen fields
   per animal to §3.1, an owner-facing spec change, not a type widening.
2. **The resolver centralised stat *lookup*, not count *enumeration*.**
   `resolve_animal_stats`/`resolve_animal_terrain_mod` resolve any key, but
   everything that walks the *counts* is fixed-four: `JpParty`'s fields,
   `JpStageOverride`'s, `jp_capacity_ex`'s `counts` closure with its
   order-pinned `JP_ANIMAL_KEYS` summation ("which fixes the float summation
   order") and four-term capacity sum, `pack_animals()`,
   `jp_best_animal_for_context`'s scan, and `journey_bridge`'s flatten/unflatten.
3. **Three golden-tested signatures return `&'static str`** —
   `JpPlan::resolve_mount`, `jp_resolve_mount`, `jp_best_animal_for_context` —
   and a user-created species id is not one.
4. **The reference is itself fixed-four** (`JP_ANIMAL_KEYS`), so none of it has
   a golden target: widening is a deliberate deviation to disclose under
   `DECISIONS.md` §7, not a port.

What ships is the **substitutes-for path**: an entry that declares
`substitutes for = <one of the four>` occupies that slot with **its own**
capacity, speed, fodder, water and ten-row terrain table. What it still borrows
from the substituted species is exactly what §3.1 has no fields for — seasonal
physiology and the desert multipliers — and the party form says so by name. A
wholly new species with no substitute (the stock Ox/Yak/Reindeer, and every
from-blank entry until "Substitutes for" is filled in) is not offered in the
party form; the form names it and the one edit that fixes it rather than
silently omitting it.

### 6.4 Measured: a custom animal really re-plans a journey (2026-08-20)

Headless drive against a generated 96×96 world, a 1 082.32 km route, a Baggage
Train of 6 with 900 kg cargo and 12 mules:

| Mule slot occupied by | days | avg km/day |
|---|---:|---:|
| stock **Mule** (baseline, no `animal_entries`) | 31.6792 | 42.1475 |
| stock **Mule**, named explicitly | 31.6792 | 42.1475 |
| custom **Kharen dray-mule** (260 kg cap, 9 kg fodder, 34 L water) | 31.1925 | 42.9617 |
| custom **Kharen dray-ox**, from blank, `substitutes for = mule` (300 kg, own ten-row terrain table) | 48.4610 | 27.4275 |

And a Mounted Rider party of 4, where the entry's own `base speed km/h` sets the
pace rather than `JP_TRAIN_PACE`'s constant:

| Mount | days | avg km/day |
|---|---:|---:|
| stock **Horse** (6.0 km/h) | 32.8385 | 40.3270 |
| custom **Kharen courser** (9.0 km/h, 150 kg) | 18.5708 | 69.5093 |

A custom entry whose ten terrain rows are all `blocked` still hard-blocks the
stage through the selection path, as it does through the implicit one.
