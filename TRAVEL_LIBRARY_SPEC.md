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

## 6 · Build status (2026-08-20: the planner's party form now offers the library)

**§1's own promise — "everything defined here becomes a selectable option in
the planner's party form" — is now true for animals, and stated in-UI where it
is not.** The `#[func]` boundary and the `2a`/`2b` window landed the day before
(§6a below, unchanged and still accurate); what this pass added is the last
connecting piece: the Journey Planner's own party form reading the live library
and its choice reaching a computed plan.

### What is now selectable, and where

- **Four per-species *animal definition* pickers** (Carriage ▸ "ANIMAL
  DEFINITIONS · TRAVEL LIBRARY"), one each for donkey/mule/camel/horse. Each
  lists every library entry that resolves to that species — stock first, then
  custom in add order — with custom rows tagged `· custom` (the `2b` mockup's
  own `custom · …` mono treatment, accent-coloured when selected) and ⚠/⚠⚠
  carrying §4's validation state through.
- **The Mount picker** is the same list filtered to §3.1's `usable as a mount`,
  labelled `<species> › <entry>`. One choice sets both facts it implies: the
  engine's `mount_animal` species key *and* that species' definition slot.
- **The Vessel picker** lists every library vessel, and **disables** the ones
  with no engine counterpart with the reason on the item itself
  (`— no engine hook`). `jp_ship_stats` is still a fixed built-in table; a
  custom vessel is real, validated data with no resolver. Stated where a user
  meets it rather than omitted.
- **Party set-ups** (§3.4, gap-register JP-02) in the tool-options bar: a
  `set-up` dropdown over `tl_list("preset")` — stock and captured alike, custom
  tagged — plus `capture party…`, which writes the current form into a new row
  through `tl_capture_preset_from_plan`. Applying assigns only the twenty keys
  `tl_get("preset", id)` returns (`PRESET_FIELD_KEYS`, `PartyPreset::apply_to`'s
  own inverse, so there is no second translation table to drift), and leaves
  per-stage overrides untouched exactly as §3.4 requires.

### How the choice reaches the engine

`jp_compute` gained one request key, `animal_entries` — `{species_key:
entry_id}` — which routes through the new
`travel_bridge::TravelLibrary::animal_overrides_selected` into the resolver
`jp_plan_ex` already consumed. Three properties are pinned by tests rather than
asserted here:

- **An absent key changes nothing.** An empty selection reproduces
  `animal_overrides()` exactly, so
  `regression_stock_only_travel_library_matches_pre_dispatch_jp_plan` still
  holds byte for byte against the plain `jp_plan` this replaced.
- **Naming a *stock* entry means "no override"** — the built-in table —
  which is deliberately not the same as leaving the slot unnamed. Verified
  live: the identical journey computes `31.6792` days both ways.
- **A selection that cannot be honoured is rejected, not silently ignored**
  (unknown species, unknown id, or an entry that resolves to no slot), landing
  in `jp_compute`'s own `rejected` array.

`TravelLibrary::animal_species_slot` is the single place that decides which of
the four built-in species an entry may occupy: its own `species_key`, else the
one its `substitutes_for` chain reaches (bounded by the store's size, so a
user-typed cycle terminates). `tl_list("animal")`/`tl_get("animal", …)` expose
it as `species_slot`, alongside `usable_as_mount`, so the form costs one call
rather than one per entry.

### Real numbers, not a claim

Headless drive against a generated 96×96 world, a 1082.32 km route, a Baggage
Train of 6 with 900 kg cargo and 12 mules:

| Mule slot occupied by | days | avg km/day |
|---|---:|---:|
| stock **Mule** (baseline, no `animal_entries`) | 31.6792 | 42.1475 |
| stock **Mule**, named explicitly | 31.6792 | 42.1475 |
| custom **Kharen dray-mule** (260 kg cap, 9 kg fodder, 34 L water) | 31.1925 | 42.9617 |
| custom **Kharen dray-ox**, from blank, `substitutes for = mule` (300 kg, own ten-row terrain table) | 48.4610 | 27.4275 |

And a Mounted Rider party of 4, where the entry's own `base speed km/h` is the
pace-setter rather than `JP_TRAIN_PACE`'s constant:

| Mount | days | avg km/day |
|---|---:|---:|
| stock **Horse** (6.0 km/h) | 32.8385 | 40.3270 |
| custom **Kharen courser** (9.0 km/h, 150 kg) | 18.5708 | 69.5093 |

A custom entry whose ten terrain rows are all `blocked` still hard-blocks the
stage through the selection path, exactly as it did through the implicit one.

### `JpParty` was re-examined and deliberately NOT widened

This dispatch was asked to check first whether widening `JpParty` to a generic
animal-count map had become bounded now that the `_ex` resolver refactor exists.
It has not, and the reason is a **spec** gap rather than a mechanical one:

1. **The data to drive a new species does not exist.** `jp_capacity_ex` reads
   `jp_seasonal_animal(season, key)` — sixteen `(cap, food, water)` rows, four
   seasons × four species — and `jp_desert_animal_mod(key)`'s desert
   food/water pair, for every species it sums. §3.1's field list carries
   **neither**. A wholly new species would silently take the neutral `1.0`
   fallbacks on both, which is precisely §5's own "would not merely look
   unfinished — it would plan silently wrong". Closing that means adding
   fourteen fields per animal to §3.1, an owner-facing spec change, not a type
   widening.
2. **The refactor centralised stat *lookup*, not count *enumeration*.**
   `resolve_animal_stats`/`resolve_animal_terrain_mod` genuinely do resolve any
   key. Everything that walks the *counts* is still fixed-four: `JpParty`'s four
   fields, `JpStageOverride`'s four more, `jp_capacity_ex`'s `counts` closure
   plus its explicitly order-pinned `JP_ANIMAL_KEYS` summation ("which fixes the
   float summation order") and its hardcoded four-term capacity sum,
   `pack_animals()`, `jp_best_animal_for_context`'s `JP_ANIMAL_KEYS` scan, and
   `journey_bridge`'s flatten/unflatten pair.
3. **Three golden-tested signatures return `&'static str`** —
   `JpPlan::resolve_mount`, `jp_resolve_mount`, `jp_best_animal_for_context` —
   and a user-created species id is not one. Widening forces `String` (or a
   borrow tied to the plan) through all three and every caller.
4. **The reference is itself fixed-four** (`JP_ANIMAL_KEYS`), so there is no
   golden target for any of it: widening is a deliberate deviation to disclose
   under `DECISIONS.md` §7, not a port.

So the **substitutes-for path** is what shipped, and it is genuinely useful: a
from-blank "Kharen dray-ox" that declares `substitutes for = mule` occupies the
mule slot with **its own** capacity, speed, fodder, water and ten-row terrain
table (the 48.4610-day row above). What it still borrows from the substituted
species is exactly what §3.1 has no fields for — seasonal physiology and the
desert multipliers — and the party form says so, by name, in the note under the
animal-definition pickers.

### Still honestly not live

- **Wholly-new species with no substitute** — the stock Ox/Yak/Reindeer, and
  every from-blank custom animal until its owner fills "Substitutes for" in —
  are not offered at all. They are **named in the party form**, with the one
  edit that fixes them, rather than silently omitted from the dropdowns.
- **Vehicles and vessels remain data-only.** No resolver equivalent to
  `animal_resolver_fns` exists for `jp_capacity`'s cart/wagon/sled/travois
  constants or `jp_ship_stats`' vessel table. The Vessel picker lists them and
  disables the unhooked ones with the reason; the vehicle counts are still
  plain `JpParty` spinners.
- **§4's "saved journeys" usage count is still always `0`** — no persistent,
  referenceable saved journey exists in this port at all. Party-set-up usage is
  real.

## 6a · Build status (2026-08-19, the `#[func]` boundary and the window)

**The whole spec is now real, engine to Godot to GDScript.** The gap this
section used to describe -- no `#[func]` boundary, no window -- is closed:

- **`cartalith-godot/src/lib.rs`'s `WorldGen` now carries a live
  `travel_library: travel_bridge::TravelLibrary` field**, bootstrapped with
  stock content in `init()` and, deliberately, **not reset by `absorb()`** on
  a re-generate -- it is user-editable project state, not civ-generation
  output, so it persists across `generate()`/`generate_world_structure()`
  the same way `asset_pack`/`quality` already do.
- **A full `#[func]` CRUD+query surface** (`lib.rs`'s Travel Library
  `#[godot_api(secondary)]` block): `tl_counts`, `tl_list`, `tl_get`,
  `tl_duplicate`, `tl_add_blank`, `tl_delete`, `tl_reset_to_stock`,
  `tl_edit`, `tl_capture_preset_from_plan` -- one dispatch over
  `kind: "animal"|"vehicle"|"vessel"|"preset"` for all four §3 types rather
  than four times the surface. The thin `Variant`<->Rust flattening lives in
  `lib.rs`; every real CRUD/validation/usage call underneath is
  `travel_bridge.rs`'s own, unchanged by this pass. `travel_bridge.rs`
  gained the `Variant`-shaped field-pairs layer this boundary needed
  (`animal_to_pairs`/`animal_apply_pairs` and the vehicle/vessel/preset
  siblings), reusing `journey_bridge::JpValue`/`jp_pairs_dict`/
  `jp_dict_to_pairs` rather than inventing a second flattening convention.
- **`jp_compute` is wired live**, not just proven in a Rust-internal test:
  it now builds a `JpAnimalResolver` from
  `self.travel_library.animal_overrides()` via
  `cartalith_civ::travel_library::animal_resolver_fns` and calls
  `jp_plan_ex(..., Some(&resolver))` unconditionally, in place of the old
  `jp_plan` call. A stock-only library is provably identical to the old
  behaviour (`resolve_animal_stats`/`resolve_animal_terrain_mod` fall back
  to the built-in table exactly as if `animals` were `None`) --
  `travel_bridge.rs`'s own
  `regression_stock_only_travel_library_matches_pre_dispatch_jp_plan` test
  asserts full structural equality (`assert_eq!`) between `jp_plan(...)` and
  the new call chain over a fresh, untouched library, not merely "close
  enough".
- **The `2a`/`2b` window is built**:
  `godot-project/shell/travel_library_window.gd`, wired at `Data ▸ ⧉ Travel
  library… (⇧L)` (`menus.gd`, `app.gd`) -- own popup window (not an
  in-shell takeover, per the mockup's own "⇧L · own window" annotation),
  tabbed by definition type, each tab a Custom/Stock entries rail (filter,
  ＋ new blank / ⧉ duplicate / ✕ delete) plus a grouped field inspector
  (exactly §3's own group names per type) with save/duplicate/revert and
  ok/incomplete/conflicting validation banners styled off `DccTheme`'s
  `warn`/`water`/`block` tokens (the mockup's own `#e0a840`/`#7d9dae`/
  `#b55950`, already-named shell-wide tokens, not re-hardcoded here).
  Edits are staged locally and committed with "save definition", matching
  the mockup's own footer exactly. The inspector says plainly, per entry,
  when a definition has no live computational effect yet (see below) rather
  than implying it already changes a plan.

**Still honestly not live**, unchanged from before this pass and explicitly
out of its scope (`GUI_GAP_REGISTER.md` JP-02/IN-06, marked "unblocked, not
yet wired"):

- The Journey Planner's own party form does not yet *offer* a custom Travel
  Library entry as a selectable Transport/mount option -- creating and
  validating one is real; picking it in the planner's own dropdown is the
  next dispatch (a different file, `journey_planner_view.gd`, was mid-edit
  by another concurrent pass during this one and deliberately left
  untouched).
- Only the four built-in party-form species (donkey/mule/camel/horse) can
  override a computed journey at all -- a wholly new species (the stock
  Ox/Yak/Reindeer) and every vehicle/vessel definition remain real,
  validated, inspectable data with no live engine hook, said plainly in the
  window's own inspector note rather than approximated.
- §4's "saved journeys" usage count is still honestly always `0` (no
  persistent, referenceable saved journey exists in this port at all).

## 6b · Build status (Rust half only, 2026-08-19, superseded by §6/§6a above)

The data model, stock content, CRUD and validation described above are real, in
`cartalith-native/crates/cartalith-civ/src/travel_library.rs` (data shapes, §4
validation, stock content, the resolver-building functions) and
`cartalith-native/crates/cartalith-godot/src/travel_bridge.rs` (the mutable
stock-plus-custom store, usage tracking). Full record —
`cartalith-native/docs/CHANGELOG.md`'s "Travel Library milestone 1" entry,
`STATUS.md`'s matching section.

**Real, and wired into an actual computed plan**: a custom Travel Library entry
overriding one of the four built-in party-form species (donkey/mule/camel/horse) —
duplicate the stock entry, edit `load_capacity_kg`/`base_speed_kmh`/`fodder_need_kg_day`/
`water_need_l_day`/a terrain row — changes `jp_plan`'s computed `days`/`avg_km_day`,
and a terrain marked `blocked` on that entry's own ten-row table hard-blocks that stage,
exactly §5's own claim. Proved by two integration tests, not merely round-tripped data.

**Not yet wired, disclosed rather than approximated:**

- **No GUI exists yet.** This spec's `2a`/`2b` window (menu item, list + inspector) is
  unbuilt — a separate, later dispatch, against the Rust surface above.
- **No `#[func]` boundary exists yet either.** `cartalith-godot/src/lib.rs`'s
  `WorldGen`/`jp_compute` do not hold a `TravelLibrary` and do not read one. The exact
  shape the GUI dispatch needs to add — `TravelLibrary::animal_overrides()` →
  `cartalith_civ::travel_library::animal_resolver_fns` → `JpAnimalResolver` → pass
  `Some(&resolver)` to `jp_plan_ex` in place of today's `jp_plan` — is documented in
  `travel_bridge.rs`'s own module doc.
- **Only the four built-in species can override anything.** A wholly new species (the
  stock Ox/Yak/Reindeer §3.1 itself names as mockup examples) has no `JpParty` slot to
  occupy — that struct is four fixed fields, not a generic animal-count map — so those
  three stock entries are real, validated, inspectable data with no live engine effect.
  Widening `JpParty`/`JpPlan` to a generic shape is real, larger work against
  golden-tested types, correctly left for a future milestone.
- **Vehicles and vessels are data-only.** §3.2/§3.3's field lists, stock content and §4
  validation are all real; no resolver equivalent to the animal one exists yet for
  `jp_capacity`'s cart/wagon/sled/travois constants or `jp_ship_stats`' vessel table.
- **§4's "usage in saved journeys" is honestly always `0`.** No persistent,
  referenceable "saved journey" exists anywhere in this port — `route_get`/
  `WorldGen.infra.routes` are drawn polylines with no attached party plan, and
  `jp_compute` computes and returns a plan without storing it. Party-set-up usage
  *is* real (`TravelLibrary::animal_usage_in_presets`), since presets are the
  library's own stored rows.
