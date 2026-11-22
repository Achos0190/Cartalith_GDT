# Town layouts in the save — scope for Ruling GZ (2026-11-22)

Defines, does not track: status is `cartalith-native/docs/STATUS.md`'s. Scope written 2026-11-22; **build not started.**
Source ruling: `LARGE_ITEM_RULINGS.md` "Ruling GZ" (owner: *"how we can save town layouts without it being graphics (I'm think a hash
string or something like that) that way if a user maps poi or pins a town that information won't be lost."*). Documentation and
measurement only: no product code was changed; the measurement harness was scratch and is not in the tree.

**Tags.** Every claim is tagged **[sym]** read at a symbol (names given, no line numbers), **[meas]** measured by me on 2026-11-22
(method in §3.4), or **[inf]** inference, to be proved by the milestone that needs it.

## 0. The answer in one paragraph

A town layout is a pure function of a large, partly hidden input set (the seed, the settlement's population and place-editor fields,
twelve `urban_*` switches, the rules table, the **world's own rasters and roads around the town**, and the generator's code). Every one of
those moves independently, and a **rules change alone moved 64 of 64 census towns** [meas]. A hash of the recipe cannot bring a town back
once the code has moved, so the ruling's design stands: a town that carries a user's pin **freezes** into compact integer vector data. The
**recipe** (with a **rules version**) says how it was made, the **fingerprint** says whether today's generator still makes the same
town, and **anchors** say what each pin sits on so it can follow after an explicit re-lay. A frozen town of median size costs about 4 KB
and the largest capital about 216 KB in the save [meas]. The one place the measurement disagrees with the ruling's wording is the codec:
Zstandard saves 3.4 % over deflate on this data [meas] and would need an exception to `SAVEFILE_COMPAT.md` §3.3, so §4.3 recommends
deflated JSON and asks (Q1).

## 1. Inventory — what a layout is and what it depends on

### 1.1 Where it is made [sym]

- `cartalith_urban::generate` (module `generate`) is the reference's `generate()`: 29 stages plus this port's wall lots, citadel and
  courtyard rings. It returns a `Town`. `hash_model(&Town)` is the reference's own whole-model comparator.
- `cartalith_civ::urban_adapter` is the only door in. `um_place_context_with(&UrbanWorld, &NamedSettlement, &[Way], &PlaceOverrides)`
  builds an `UrbanContext`; `run_layout(&UrbanContext, Option<&Rules>)` (and `run_layout_choosing` / `run_layout_as` for Ruling FQ's
  stretched river towns, which may lay a town out up to three times) calls `generate` and projects the `Town` into `UrbanLayout`.
- `cartalith-godot` `urban_bridge.rs`: `WorldGen::urban_layouts(indices)` builds the context through `place_overrides` and `own_rules`,
  lays the town out through `stretch_layout` and crosses it with `layout_dict`. It keeps no layout cache (only the stretch verdicts,
  `stretch_choices`, keyed by `stretch_choice_key`). The map (`map_overlay.gd`, cached by index) and the City Viewer
  (`shell/city_viewer_window.gd`) both read **this one call**, so a frozen town served from this one place is served everywhere.
- A town is generated on demand per settlement, never as a generation stage (`cartalith-urban` crate header). Opening a save never lays
  a town out.

### 1.2 Every input [sym]

| Group | Inputs | Where read |
|---|---|---|
| Identity | `seed = floor(hash(x as i32, y as i32, world_seed) * 4294967295)` as `u32`; `x, y` the settlement cell; a `variant` XORs into it | `um_place_context_with` |
| Settlement | `pop` (clamped to 400..20000 for generation), `settlement_age` (`age_override` or `um_infer_age(pop)`), `site_kind` | `UrbanContext` |
| Place editor | `specialisation`, `fortified` trait, `umWalls` override, `umAge`, `culture` (Ruling J town plan), `rules_preset`, `variant` | `PlaceOverrides`, saved in `settlements[].extras` and `place_extras` |
| Faith | the ruling faction's religion rite (`civ.town_faith`) | `PlaceOverrides::faith` |
| Walls | `wall_style` from `um_wall_spec`, `walls` | `UrbanContext` |
| **World, around the town** | the `field` (relief), `flow`, `water_bodies`, Strahler `order`, the traced river polylines, sea level; these become `UmWater` (a run-length water mask, river path, sea fraction) and `TerrainCtx`; and **the roads**: `route_ends`, `primary_paths` from `civ.ways` | `UrbanWorld`, `um_water_ctx_with`, `um_terrain_ctx`, `um_route_ends`, `um_primary_paths_reach` |
| Switches (app defaults from `cartalith_godot::params::defaults`) | `urban_wall_hug`, `urban_riverbank`, `urban_street_village`, `urban_market_scale`, `urban_harbour_front`, `urban_river_stretch`, `urban_burgage`, `urban_market_road_bank`, `urban_growth_water_widen`, `urban_town_bridge_search`, `urban_parallel_streets`, and `urban_river_kind_map` (decides the site kind through `UrbanWorld`) | `place_overrides`, `UrbanWorld::river_kind_from_map` |
| Rules | a `Rules` table (street 14 f64, parcels 3, settlement 5, meta 2) or `None` = `DEFAULT_RULES`; per-town `rules_preset` ("" follows the world's active rules, `default`, or a named preset such as `market_town`) | `own_rules`, `Rules::to_patch` |
| Stretch | `site_box`, `river_stretch` plan and the `plain` fallback context (Ruling FQ), and the verdict `StretchChoice` | `stretch_choice_key` |
| **Code** | the generator itself: every constant and stage. Not a stored input anywhere today | n/a |

Two consequences drive the whole design.

1. **The world under the town is an input.** A sculpt, a regenerate of the world, a changed river or a moved road changes a town with
   no change to any town setting. A recipe of "seed, pop, type" cannot describe a town; the recipe must carry a digest of the
   world-derived context (§4.1).
2. **The generator's code is an input that nothing records.** Today the only guard is the goldens, which pin the reference-default
   switches; the twelve app-on switches and the port's own stages are pinned by no stored version. Two sibling builds are changing it
   now (parallel streets PS-4, river smoothing inside layouts) [sym: the coordinator's brief].

### 1.3 What a layout consists of [sym: `UrbanLayout`, `layout_dict`]

Everything below is output the renderer draws; a frozen town must reproduce all of it, because the shell draws from `layout_dict`'s
keys only.

| Part | Fields |
|---|---|
| Frame | `wm, hm` (box metres), `market`, `pin` (where the settlement's map position falls; the City Viewer and the map anchor on it), `orient`, `site_kind`, `market_prov` |
| Streets | `edges`: segments `(a, b, cls, w)` with cls one of `lane, street, quay, ringroad, primary` |
| Ground | `blocks` (polygons) and `block_plaza`; `plaza` (polygon, centre); `markets` (named squares) |
| Lots and houses | `parcels` (quad, `edge_cls`, `tone`, `district`), `buildings` (polygon, `ridge`, `kind`, `district`, `courtyard`), `building_tone`, `building_intramural` |
| Walls and gates | `wall` (`ring`, `gates` with `water` flag, `spurs`, `land_arc`, `water_walls`, `history` of superseded rings, `style`, `fort`), `citadel`, `wall_hug` outcome, `wall_spec` |
| Water | `water_poly`, `river`, `river_w`, `water_mask_runs`, `bridge_pt`, `bridges`, `ford`, `harbour` (quay, piers, mole, defence), `harbour_pt` |
| Outside the walls | `farmland` and `details` (wells, trees, fences, racks, booms, canal; each a point, segment or polygon with `kind`, `rr`, `orchard`), `faith_sites` (rite, polygons) |
| Scalars | `street_len`, `target_len`, `max_rf`, `pop_target`, `pop`, `settlement_age`, `uses_real_water`, `uses_real_terrain`, `route_ends`; the PS-1 `parallel` plan (diagnostic) |

**Identifiers today [sym]:** buildings carry `bld<n>` and parcels a string id, counted across one run and **not stable across runs or
rules**; edges have no id (a position in the list); gates are positions in `wall.gates`; the plaza is one value. Inside a frozen town
every one of these becomes stable because the arrays are stored in order (§4.4).

**Typical size** (counts, 64 census towns, §3.4) [meas]: hamlet median 145 buildings and 36 street segments; village median 199
buildings; town median 632 buildings and 132-347 segments; city 2 874; capital 4 292 (up to 5 587 buildings, 1 547 segments, 5 161
parcels). Generation time for the same towns, release build on a machine with other builds running [meas, noisy]: hamlet median 108 ms,
town 264 ms, city 1.75 s, capital 2.4 s (max 5.4 s).

### 1.4 What already exists for fingerprints, and why neither is reusable [sym]

- `hash_model` (FNV-1a over centimetre-rounded edges, nodes, blocks, parcels and buildings) is golden-verified against the reference but
  **does not read walls, plaza, water, details, farmland, ridges or districts of buildings**, so a wall-only change leaves it unmoved.
- `stretch_choice_key` hashes the `Debug` text of `GenOpts` with `std`'s `DefaultHasher`. It is a good in-process cache key (derived
  from the definition, so no input list to keep in step, the rule in `MISTAKES.md`) and **must never be saved**: the hasher's algorithm
  is unspecified across Rust versions and `Debug` text changes when any field is added, so every saved town would read as changed.
- `ZR_RULES_VERSION` (`ZOOM_REFINEMENT_SCOPE.md` §4.3) is the pattern for the rules version: a literal beside a golden fixture hash.
- Conflict anchors (`project_bridge.rs` `ConflictAnchorDto`: kind, `tid`, and `anchor_at`, where the anchor stood when attached) are the
  precedent for "bind to an id and remember the position".

### 1.5 How pins and POIs sit in a town today [sym]

- A user POI is stored in **absolute continuous grid coordinates** (`annotations/pois.json`, `SAVEFILE_COMPAT.md` §11.2a: `id`
  never reissued, `x, y`, `kind`, `kind_name`, `name`). It binds to nothing in the layout. The City Viewer maps it into the town by the
  map's own transform inverted (`_poi_pins`: `anchor + R(-orient) * (grid point - cell - 0.5) * metres_per_cell`, within
  `POI_TOWN_RADIUS_M` = 850 m of the settlement cell).
- So a pin never moves when a rules change redraws the town; **the town moves under it** (a tavern pin ends up in a street or a field).
  That is the owner's loss. The pin's vault note binds to the POI `id` (§13.3.4, `poi`), which survives; nothing binds a pin to a house.
- Every write goes through `poi_bridge.rs`: `poi_add`, `poi_add_free`, `poi_move` (and the CARTO pin tool, the assistant and any import
  call these). That is the engine-level place to hook a freeze, not the shell.
- The City Viewer's existing **Regenerate** button is **not** "regenerate with current rules": it bumps `place_extras.variant`, which
  re-keys the seed and draws a different town (`_regenerate`, `_set_plan`). The new action must be named differently (§6).
- There is no geometry editing of a town anywhere [sym]: the viewer's writes are the Town plan (culture, rule set, variant) and place
  editor fields, all of which are **recipe** inputs.

## 2. Design in outline (approved by Ruling GZ; this document specifies it)

1. A town is **frozen** the first time a pin or POI is placed in it (or an explicit "Freeze this town"). A frozen town stores its recipe,
   fingerprint, geometry and the anchors of the pins on it.
2. A town never touched stays a recipe-less computation, exactly as today (nothing to lose, nothing to store).
3. A frozen town is served from the save by `urban_layouts`, so the map, the City Viewer and the planner see the same town the pin was
   placed on, whatever the rules, switches or world have become since.
4. **Nothing replaces a frozen town silently.** A difference is reported; the only way to change it is the user's explicit re-lay.

## 3. Measurement

### 3.4 Method [meas]

Scratch test in a worktree of `origin/main` at `cc68e2ad` (not committed, deleted): worlds generated through the app's parameters
(`fh_r2_baseline::generate_world`, `params::defaults()`), seeds **483920 and 912345**, 800 km, **384 columns**, six factions; every
settlement (64 towns: 36 hamlets, 9 villages, 6 towns, 3 cities, 10 capitals) laid out with the app's switches on
(`PlaceOverrides` as `place_overrides` builds it), release build, `run_layout`. One machine, other builds running, so timings are
noisy. Not measured: the 2 048-column worlds (town sizes do not depend on the grid; their count does), a third seed, Android, decode.

### 3.1 Encoded size per town [meas]

Geometry quantised to **0.1 m** integers; street graph as a node table plus edge index pairs; polygons delta-coded with zigzag varints;
string attributes as table indices. Encoded: edges, blocks, parcels (with district, tone, class), buildings (with kind, courtyard, tone,
ridge), wall (ring, gates, spurs, land arc, water walls, history), plaza, water polygon, river, route ends, water mask runs, markets,
farmland and details, faith-site polygons, harbour, citadel. **Not encoded:** the fort bastions, bridge and ford points, scalars and
provenance strings (inference: under 2 KB; the real encoder in TL-1 re-measures).

| Town (census) | Buildings | Raw varint | Binary + Zstd-3 (method 93) | **JSON delta ints, deflate (§3.3 as it stands)** | Binary + deflate | Zstd in base64 in JSON, deflate |
|---|---|---|---|---|---|---|
| **Small:** village, pop 477, coast | 33 | 2.2 KB | 1.4 KB | **1.1 KB** | 1.4 KB | 1.4 KB |
| **Median by raw size:** hamlet, pop 167 | 162 | 9.5 KB | 5.3 KB | **3.9 KB** | 5.0 KB | 5.3 KB |
| Town, median of 6 | 632 | 32 KB | 24 KB | **23 KB** (22.6) | 23 KB | 24 KB |
| City, median of 3 | 2 874 | 153 KB | 109 KB | **116 KB** | 116 KB | 109 KB |
| **Largest:** capital, pop 24 647 | 5 587 | 273 KB | 201 KB | **216 KB** | 216 KB | 202 KB |

Medians by class: hamlet 4.9 KB (Zstd), capital 157 KB. **All 64 towns frozen at once: 2.32 MB (Zstd), 2.40 MB (JSON deflate), 2.44 MB
(binary deflate).** For scale, a format-3 save's six core rasters are about 14-79 MB (`SAVEFILE_COMPAT.md` §18.7).

Findings: (a) varint deltas barely compress further (273 KB to 201 KB for the capital), so the geometry is already dense; (b) **Zstd
buys 3.4 % over deflate in total (7 % on the capital)** and the `json delta ints + deflate` form is within 4 % of it; (c) absolute
integer arrays in JSON cost 20-25 % more than delta arrays after deflate. Reductions not measured [inf]: buildings are
near-rectangles (centre, size, angle is 6 numbers against 8), and parcels share corners with neighbours (a shared vertex table); either
could take roughly a further 20-30 %; TL-1 decides whether the complexity is worth it.

Encode time 0.02 ms (village) to 3.2 ms (capital) [meas]; decode is the same order [inf].

### 3.2 Does the same town come out twice? [meas]

Same process, same context, laid out twice: the canonical bytes were identical for **64 of 64 towns**. Across machines and
architectures it is **not measured** and is the largest unknown (§9, R1).

### 3.3 How much does a rules change move? [meas]

Same 64 contexts re-laid out with the `market_town` rules preset: **64 of 64 changed**. With `urban_parallel_streets` on: **19 of 64
changed** (the rest are villages, hamlets and radial towns the switch does not reach). A single rules or switch decision moves
essentially every town; this is why the pins need a frozen town and not merely a recipe.

### 3.5 Re-attach by nearest same-kind feature [meas, pooled]

For each town, up to 20 pins per feature kind were placed on the default layout (evenly spaced), the town was re-laid out under each of
the two changes above, and each pin was moved to the nearest feature of the **same kind and same tag** (building: kind and district;
plot: district; street: class; gate: water flag). Share within 10 / 25 / 50 m, 8 994 pins pooled:

| Kind | Pins | 10 m | **25 m** | 50 m |
|---|---|---|---|---|
| Building | 2 782 | 70 % | **83 %** | 88 % |
| Plot | 2 944 | 77 % | **86 %** | 91 % |
| Street (segment midpoint) | 2 950 | 69 % | **82 %** | 89 % |
| Plaza | 128 | 93 % | **93 %** | 93 % |
| **Gate** | 190 | 14 % | **24 %** | 35 % |

**Read these as an upper-pooled baseline, not a bar:** the pool mixes the 64 `market_town` re-lays with the 19 parallel-streets re-lays,
and a town the second change leaves identical re-attaches trivially (the split is TL-4's first job). **Gates fail the nearest rule**: a
gate is where the wall meets a road, and when the road moves the gate moves a long way. §4.4 therefore re-attaches a gate by bearing
from the market (inference, to be measured).

## 4. The four pieces

### 4.1 Recipe and rules version

**The recipe** is everything needed to *ask the generator for the same town again*, stored as plain values (never as a hash alone):

```
recipe = {
  "rules_version": <int>,                    // URBAN_RULES_VERSION when frozen (below)
  "switches": { "urban_riverbank": true, ... },   // the 13 listed in §1.2, name -> bool, keys sorted
  "seed": <u32>, "cell": [x, y], "pop": <clamped f64 as integer>, "age": <years>,
  "site_kind": "...", "wall_style": "...", "walls": <bool>, "fortified": <bool>,
  "culture": "...", "faith": "...", "specialisation": "...",
  "rules": "<preset id>" | { "street": {...}, "parcels": {...}, "settlement": {...}, "meta": {...} },
  "variant": <int>,
  "site_hash": "<16 hex>"                     // a digest of the world-derived context, below
}
```

**Rules version.** Two parts, both stored, compared separately so the report can say *what* differs.

- **`URBAN_RULES_VERSION`**: an integer constant in `cartalith-civ`'s `urban_adapter` (the one door; it sees both the adapter's and the
  generator's behaviour), bumped by hand whenever a change moves a generated layout under any switch setting. It is guarded the way
  `ZR_RULES_VERSION` is: a **fixture-hash test** lays out a fixed set of contexts (six synthetic ones spanning the site kinds, the size
  tiers, a walled and an unwalled town and a stretched river town, **with the app's defaults on**) and asserts a committed literal
  hash of their canonical bytes beside the version literal. Any behavioural change moves the hash, the test goes red, and the fix is to
  bump both. Mutating the version alone also goes red. Starts at `1`.
- **The switch set**: each `urban_*` name with the value the town was made with. **A switch absent from a recipe reads as the value it
  had before the switch existed (off)**, so a later build that adds a switch (PS-4's, river smoothing's) does not make every old frozen
  town "differ"; it differs only if the new switch is on **and** reaches that town. A switch whose default flips is reported by name.

**Why not derive the version from a hash of the code or the switch table alone.** A hash of the switch set misses a changed constant in
`rules.rs` or a stage; a hash of source text moves on every comment (`MISTAKES.md`: a change-detector that fires on everything is
ignored). The fixture hash is the one instrument that fires on exactly "output moved".

**`site_hash`** is FNV-1a 64 over a **specified canonical byte stream** of the world-derived context: the box and `site_box`, the water
context's mask runs and river path, the terrain context's grid, `route_ends`, `primary_paths`, each coordinate as `round(v * 10)`. It is
**not** the `Debug` text of `GenOpts` (§1.4) and carries its own version byte. It answers "has the world under the town moved since
it was drawn" independently of "have the rules moved".

### 4.2 Fingerprint

- **Hash.** FNV-1a 64 (lowercase 16 hex), the function `cartalith_urban::generate` already uses for `hash_model` and the one
  `SAVEFILE_COMPAT.md` §13.3.4 uses for landmark ids, so there is one hash function in the format. It detects accident, not tampering;
  at tens of towns a 64-bit collision is not a practical risk.
- **What is hashed.** The **canonical byte stream of the frozen geometry** (§4.3), after quantisation, in the stored order: streets,
  blocks, parcels, buildings, wall, plaza, water, markets, farmland and details, faith sites, harbour, citadel, in that order, with the
  attributes. Hashing the stored form (not the floats) is what makes the stored hash checkable by a reader that never runs the generator.
- **Two uses.**
  1. **Integrity.** On open, the stored geometry is decoded and re-hashed; a mismatch means the row is damaged (§7).
  2. **Drift.** To learn whether today's generator still makes this town, lay it out from the recipe (one town, 0.1-2.4 s) and compare
     its canonical hash with the stored one. Equal: "unchanged". Different: the town is reported as drawn by older rules and a
     before/after is offered. This runs **on demand** (opening the town in the City Viewer, or the bulk check, Q7), **never on open**:
     64 towns would be about 2.3 s of main-thread work [meas, summed generation time is higher with stretch retries].
- **Cheap tier first.** Before generating, compare `rules_version`, the switch set and `site_hash` (no generation needed): any difference
  already says *why* ("rules v1 to v3", "urban_parallel_streets is now on", "the ground under the town was edited").
- **Quantisation boundary [inf].** A coordinate within 0.05 m of a rounding boundary could round differently on another architecture
  (a fused multiply-add, a libm difference: `JS_SEMANTICS_AUDIT.md`). So an unequal hash is not yet "different": compare the two decoded
  town structures (same counts, every vertex within 0.15 m) and report **"same town within rounding"** when they pass. Only a failure
  of that test is a mismatch.
- **On a mismatch the user is told and the layout is never silently replaced.** The frozen layout keeps being served; the City Viewer
  shows the banner of §6; the open-project report names the count ("3 frozen towns were drawn by older rules"). Nothing re-lays unless
  the user presses the action.

### 4.3 Frozen geometry encoding

- **Integer coordinates at a 0.1 m quantum**, in the layout's local metres (`0..wm`, `0..hm`), plus `wm, hm, market, pin` and `orient`.
  `orient` is stored as its exact `f64` bit pattern (hex), because it rotates the whole town about the pin and a rounded angle would walk
  every vertex. *Why 0.1 m:* the generator's own comparator rounds to a centimetre (`hash_model`), a house is 4-15 m, the smallest
  feature drawn is a bollard or well; a 0.05 m worst-case error is under a pixel at the deepest City Viewer zoom [inf: confirm at
  ×64 in TL-1's visual check]. A coarser quantum (0.25 m) would shave bytes but not change the order of magnitude, so it is not worth
  the visible risk.
- **Vertex lists are delta-coded**: the first vertex absolute, each next as `(dx, dy)` integers from the previous. **Streets are a node
  table plus index pairs** (a street graph shares its nodes), which also gives each street segment a stable id: its index.
- **Attributes** (`district`, building `kind`, edge class, detail kind) are small integer indices into a string table stored once in the
  row, so renaming a district string in a later build cannot silently relabel a frozen town: an unknown string is read from the stored
  table, never from the current build's.
- **In the JSON slot** the geometry is nested arrays of integers (`"edges": [[...], ...]`), written by the project writer in sorted-key,
  integer-only form, so it is **method 8 (deflate) like every other JSON document** and a plain zip tool still lists and reads it
  (`SAVEFILE_COMPAT.md` §3.3). Size under this form is the **JSON column** of §3.1.
- **The Zstandard question (Q1).** Ruling GZ says *"integer coordinates, Zstandard"*. Format 3 allows method 93 **only** for `rasters/`
  and `history/territory/` (§3.3), and measurement says it saves 3.4 % here. Two honest options: **(A, recommended)** keep the slot
  JSON and deflate, no change to §3.3; **(B)** write each frozen town as a binary entry `entities/town_layouts/<id>.bin` at method 93,
  which needs §3.3's exception widened, a second reader, and gives up the zip-tool readability. The default is A; B costs a rule change
  for 3.4 %.
- **Raster rules do not apply.** None of §8.5's row delta or §8.2's byte-plane shuffle (they act on grid payloads). The delta here is
  geometric.
- **Versioning.** Each row carries `"enc": "q10d1"` (quantum 10 per metre, delta, version 1). A reader that does not know an `enc`
  keeps the row opaque (§7), never guesses.

### 4.4 Pin anchors

**What a pin binds to.** For each POI inside a town's box, the anchor records:

```
{ "poi": <poi id>,                  // pois.json id, never reissued; the key
  "kind": "building|plot|street|gate|plaza|market|bridge|ground",
  "fid":  <index in the frozen array, or null for plaza/ground>,
  "off":  [du, dv],                 // 0.1 m integers, offset from the feature's anchor point in the feature's own axes
  "tag":  "<kind>|<district>" ,     // the feature's class, for same-kind matching
  "at":   [gx, gy]                  // grid position when anchored (conflict anchors' anchor_at precedent)
  "u":    <0..1000>                 // street only: position along the segment, per mille
}
```

- **Choosing the feature at freeze or at pin placement:** the smallest containing building; else the containing plot; else the nearest
  street segment within 6 m (with `u`); else a gate within 12 m; else the plaza or a market square if inside; else `ground` (offset from
  the market, never re-attaching, kept at its absolute place).
- **Ids stay stable within a frozen town** because every array is stored in order and the anchor stores the index, not the
  `bld<n>` string. Re-freezing the same town (an explicit Freeze after a re-lay) issues new indices and rewrites the anchors in the same
  step, in one undo group.
- **Stored where.** In `town_layouts.json`, under the town, **keyed by POI id**; `pois.json` is unchanged (the pin does not know it has
  an anchor; the saved bytes of every older world stay identical). A POI deleted later leaves an anchor with no pin: the reader drops
  it and counts it; a pin with no anchor in a frozen town gets one the next time the town is frozen (it is placed on a frozen town, so
  it anchors at placement).
- **Which marks** (Q6): POIs only. Labels, icons and measurements keep absolute positions as today. Generated (derived) town pins are
  recomputed from the frozen layout, so they cannot drift.

**Re-attach after an explicit re-lay.** The new layout `L'` is made with the current rules; for each anchor, in order:

1. Take `L'` features of the **same kind and the same `tag`**.
2. The expected point is the **old feature's anchor point** in the **town frame** (grid offset from the settlement cell, i.e. the map's
   own transform), so a market that moved does not shift the search.
3. Choose the nearest within the **bar**: building 25 m, plot 25 m, street 15 m (keeping `u` along the new segment), plaza 60 m, bridge
   20 m. Ties go to the lower index. **Gates match by bearing from the market within 15 degrees, then by nearest** (§3.5 shows the
   distance rule fails them; inference, to be measured in TL-4).
4. The POI is moved to `feature point + off` (rotated into the new feature's axes).
5. **A pin that cannot re-attach stays exactly where it is, keeps its id, name, kind and note link, and is marked `detached`**: listed
   in the re-lay report ("18 of 430 pins did not find a matching building and were left in place"), drawn with the existing
   missing-icon style outline, and re-anchored to whatever it now stands on (plot, street or ground) so it is again a normal anchored
   pin. **No pin is deleted by a re-lay.** The whole re-lay (layout, moved pins, anchor rows) is one undo step.

## 5. Triggers

| Event | What happens |
|---|---|
| A POI is added or moved inside a town's box (any caller: the CARTO tool, the assistant, import; hooked in `poi_bridge.rs`) | If the town is not frozen: lay it out now, freeze it (recipe, hash, geometry), anchor the pin. If frozen: anchor the pin on the frozen layout. One undo group with the placement. |
| "Freeze this town" (viewer menu) | Same, with no pin. |
| A place-editor or Town-plan edit to a frozen town (culture, rule set, walls, age, specialisation, variant) | **Never silently applies.** The shell asks: "This town is frozen. Re-lay it with the new settings? Pins will follow." Yes runs the re-lay; No leaves the setting recorded but the town frozen (Q5). The existing **Regenerate** (new variant) becomes "New variant" and goes through the same question. |
| **Re-lay with current rules** (the new action; unfreezes, lays out from the recipe's *current* inputs, re-attaches, refreezes) | The City Viewer shows **before and after** (a toggle and a swipe between the frozen and the new plan, the changed share of buildings, the re-attach report); Apply or Cancel. Apply is one undo step. Cancel changes nothing. |
| The world under the town changes (sculpt, regenerate, road or river edit; `site_hash` differs) | The frozen town is kept. The viewer banner says "the ground or roads under this town changed since it was drawn"; Re-lay is offered. Never automatic (Q4). |
| The rules or an `urban_*` switch changes (`urban_params_changed`) | Unfrozen towns re-lay as today; frozen towns stay. The banner names the difference by the cheap tier of §4.2. |
| A frozen town's settlement **grows or shrinks** in the timeline or recovery simulation | See below. |
| Undo of the placement that froze the town | The town unfreezes (the row is removed) only if nothing else is anchored on it. |

**Frozen town and the timeline** [sym]: the City Viewer reads the town "at the year shown": the class and population come from the
snapshot `civ_goto_year` loaded, and the window re-reads on `timeline_changed` (PC-7). A frozen town would otherwise draw one layout
for a settlement of 300 and again for the same settlement at 20 000.

- **Default (Q3):** the frozen layout wins in every year in which the settlement exists. That is the only way a pin placed at one year
  stays attached through the others. The year strip and the viewer show a **"drawn at pop 2 207, year 412; this year pop 31 000"
  chip** when the settlement's population differs from the frozen one by more than 2x either way, with Re-lay as the action. A
  settlement not yet founded or already destroyed is not drawn, as today, and its frozen row waits.
- **The owner question this raises:** should a frozen town *size up* (several frozen layouts per town, one per size tier, chosen by
  the year) or stay one layout? One layout is the cheap and predictable answer; tiered layouts triple the bytes and need a rule for which
  tier a pin belongs to. Default: one layout, with the chip. Per-tier freezing is a later row if the owner wants it.
- **A recovery simulation that changes class** (a city collapsing to a town) does not re-lay a frozen town; the chip says so.

## 6. Forms — what the user sees

The existing surfaces are the City Viewer (`city_viewer_window.gd`, one window with desktop, tablet and phone forms through
`DccTheme`), the map's town layer, the open-project report and the command index.

| Surface | Desktop | Tablet | Phone (393 dp and 1080x2340 at density 2.8125) |
|---|---|---|---|
| Frozen badge | A chip in the viewer header: "Frozen · year 412 · pop 2 207 · rules v3 (current v5)" with a lock glyph | Same, in the header row | A lock chip under the title; the detail text is the chip's tap sheet. 44 dp target |
| Re-lay with current rules | Header button beside Look; opens the before/after | Same | In the viewer's overflow menu and as a primary button in the chip's sheet; the before/after is a full-screen toggle (tap to flip, no swipe needed) |
| Mismatch banner | A one-line non-modal bar above the plan: "Rules changed since this town was drawn: parallel streets is now on. [Re-lay] [Keep]" | Same | A banner above the plan that wraps to two lines; the two buttons stack, 44 dp |
| Detached pins | After a re-lay, a list in the report and a "Detached" section in the viewer's pin list | Same | Same list in a bottom sheet |
| Open report | One line in the open-project warnings: "3 frozen towns were drawn by older rules (open Records > Towns)" | Same | Same line; no modal |
| Bulk | Command index entries "Check frozen towns" and "Re-lay all frozen towns" (Q7) with a count report | Same | Same entries in the command sheet |
| Map | **No new glyph** (Q9): a frozen town looks as it always did | Same | Same |

New buttons obey the project's measured rules: the phone's minimum 44 dp, no `clip_text` on a label that sits beside an expanding
sibling (`MISTAKES.md`), and the phone/tablet class chosen by the window's dp size (Ruling EG). Every string is a labelled judgement
until the owner sees a frame.

## 7. Save rules (`entities/town_layouts.json`)

**Registration** [sym]: one line in `cartalith-io` `DOCUMENT_SLOTS`, a `SLOT_` constant and an engine-owned restore beside
`name_styles` in `project_bridge.rs` (`ENGINE_OWNED_SLOTS`), and a section in `SAVEFILE_COMPAT.md` (new §9.10, with §5's tree and §13.3.4's
note that anchors do not change `poi` ids). **No `format_version` bump**: the member is additive and format 3 already opens a
new document slot as unknown.

```json
{
  "version": 1,
  "towns": [
    {
      "id": 12,                         // settlements[].id, the stable id (the conflict anchors' tid)
      "frozen": { "year": 412, "pop": 2207 },
      "recipe": { "rules_version": 3, "switches": {"urban_riverbank": true}, "seed": 2918471, "cell": [104, 57], "pop": 2207, "site_hash": "9c1e..." },
      "enc": "q10d1",
      "layout_hash": "a41f33b0c2d9e671",
      "geom": { "wm": 17000, "hm": 12500, "orient": "0000000000000000", "pin": [8500, 6250], "strings": {"district": ["market"]}, "nodes": [...], "edges": [...], "blocks": [...], "parcels": [...], "buildings": [...], "wall": {...}, "water": {...}, "details": [...] },
      "anchors": [ { "poi": 4, "kind": "building", "fid": 211, "off": [12, -8], "tag": "tavern|market", "at": [104.62, 57.31] } ]
    }
  ]
}
```

**MUST**
- A writer MUST write the member only when at least one town is frozen, so a world with none re-serialises byte for byte.
- A writer MUST write rows in ascending `id`, integer-only geometry, sorted keys, so a load then save **round-trips byte for byte**.
- A writer MUST write `layout_hash` over the canonical stored form (§4.2) and MUST NOT write a row whose hash it did not compute from that form.
- A reader MUST serve a readable frozen row in preference to generating, and MUST NOT replace it with a generated layout without the user's
  explicit action (§5).
- A reader MUST keep, report and write back a row it cannot use (below); it MUST NOT drop it.
- A reader MUST treat a switch absent from `recipe.switches` as off (§4.1).

**MAY**
- A reader MAY decode lazily (on first use of the town), as long as it verified the row's hash before serving it.
- A writer MAY omit `anchors` when empty and `frozen` when unknown.

**Reading, by the damage ladder (`SAVEFILE_COMPAT.md` §6.4a)**

| Damage | Result |
|---|---|
| Slot absent (every older save) | No frozen towns; every town is generated as today; pins stay at their absolute positions. First pin freezes. No warning. |
| Document is not parseable or `version` is greater than known | The whole slot is **kept verbatim** and written back untouched; towns are generated; one warning names the entry. |
| A row with an unknown `enc` | **Kept verbatim**, town generated, warning "frozen town #12 uses a newer layout encoding". |
| A row whose `layout_hash` does not match its decoded geometry, or a missing mandatory member, or an out-of-range index | **Kept verbatim**, town generated this session, warning "frozen town #12 is damaged; it is drawn from the rules until you freeze it again". The row is replaced only when the user freezes that town again, never by an automatic pin placement (the pin placement asks first). |
| An anchor whose `poi` no longer exists | The anchor is dropped and counted; the town is unaffected. |
| An anchor with a `fid` out of range | The anchor is dropped; the pin keeps its absolute place and is reported as detached. |

**Older build, newer save.** An older build ignores the slot (§6.3, §14.3), draws generated towns and keeps every pin at its absolute
position; **a resave from the older build drops the slot**, so the freezing is lost, never the pins or their notes. (A format-3 save is
already unreadable by the HTML app and by builds before Ruling GX.)

**Round-trip test.** Freeze a census town, save, load, save again: the two `town_layouts.json` byte strings are equal; a mutant that
reorders rows, quantises a different quantum or drops `orient` bits turns it red (TL-2).

## 8. Milestones

Every milestone obeys Ruling BK (annotated code) and `MISTAKES.md`'s rows for pinned constants (assert literals, mutate each), format
changes (grep every reader outside the owning crate: the shell's `ZIPReader` reads some archives itself) and "cargo test cannot see a
broken shell" (probes, windowed, a mutant that turns each red). Bars are numeric; those marked * rest on §3 numbers and carry a
15-25 % margin; the rest are new.

| # | Milestone | Bars |
|---|---|---|
| **TL-0** | `URBAN_RULES_VERSION`, the switch-set record, `site_hash`, the fixture-hash test. Lands **after** parallel streets PS-4 and river smoothing so its first literal is their output | 6 fixture hashes pinned as literals; changing one rules constant turns it red; changing the version alone turns it red; `site_hash` moves when a water-mask run, a terrain cell, a route end or a primary-path point moves (4 mutants) |
| **TL-1** | The canonical codec: `UrbanLayout` to and from the frozen form; the fingerprint | decode(encode(l)) re-encodes byte-identical on all 64 census towns; max vertex error at most 0.05 m (asserted); `layout_dict` of a decoded town equals the generated one on every key the shell reads (key-by-key, counts and 0.1 m); size, JSON delta ints deflated: hamlet median **at most 5 KB***, town median at most 28 KB*, **capital maximum at most 260 KB***, all 64 towns at most 2.9 MB* (measured 3.9 KB, 23 KB, 216 KB, 2.40 MB); decode of the largest capital at most 20 ms (to be measured; generation is 2 400 ms); visual check at ×1, ×4, ×16, ×64 against the generated town |
| **TL-2** | The slot: `DOCUMENT_SLOTS`, writer, restore, report, `SAVEFILE_COMPAT.md` §9.10 | byte-identical round trip on 5 frozen census towns; an older save (built from a real prior-format file with `git show`, not an empty fixture) opens, resolves and re-saves byte-identically; each row of §7's damage table has a test and a kept-verbatim write-back test; a save with no frozen town writes no member |
| **TL-3** | The freeze trigger in `poi_bridge.rs` and the serve path in `urban_layouts`; the viewer badge | a pin placed through each of 4 callers (CARTO tool, assistant, import, POI move) freezes exactly its town; a frozen town's `urban_layouts` dictionary is identical after the rules preset changes; **a frozen town loads in at most 10 % of its generation time** (capital at most 240 ms, measured generation 2 440 ms median); an unfrozen town costs 0 extra work; windowed probe on desktop, tablet and phone |
| **TL-4** | Anchors and the re-lay: choose, store, re-attach, detach, undo | **0 pins lost across a rules change** (asserted over all pins of all 64 towns under both changes of §3.3, counting detached pins as present); re-attach within bar: building and plot at least 80 %*, street at least 78 %*, plaza at least 90 %*, gates (by bearing) at least 70 % (proposed, unmeasured); the re-lay then undo restores the layout and every pin to the byte; the split of §3.5 by change is reported first |
| **TL-5** | Drift detection and cross-architecture determinism | a frozen town re-laid with unchanged rules gives a **byte-identical layout** on 64 of 64 towns (measured same-process 64 of 64); the cheap tier names the cause for each of 4 injected differences (rules version, a switch, the world, a place-editor field); **the same 64 towns frozen on x86-64 and re-laid on Android arm64 report 64 of 64 equal or "same within rounding"**, the exact-equal count reported |
| **TL-6** | Forms and the report: badge, banner, before/after, bulk commands, open report line | probes at desktop 1600x900, tablet 1280x800 and phone 393x852 and 1080x2340 at 2.8125, real input events, frames opened and looked at; every new control at least 44 dp on the phone; contrast pair computed for the chip and banner; a mutant per probe |
| **TL-7** | The timeline chip and the recovery simulation | a frozen town's pins keep their anchors across 5 scrubbed years; the chip shows at a 2x population difference and not at 1.9x; nothing re-lays on a year move |
| **TL-8** | Docs and rules: `SAVEFILE_COMPAT.md`, `DECISIONS.md` §7p note (frozen towns are outside the §7p switch story: their drawing is stored data), `MISTAKES.md` entries for anything met, `STATUS.md` | the prose of every touched comment matches the code (the row in `MISTAKES.md`); `cargo test --workspace` and a parse check of every `.gd` touched plus `shell/app.gd` |

TL-0 and TL-1 are independent of the UI and can be built by two lanes; TL-3 and TL-4 are one lane (they share the engine hook); TL-6 is
the UI lane and needs TL-3.

## 9. Risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | **Cross-architecture determinism**: a frozen layout from x86 re-lays differently on arm64 (FMA, libm) and every town reads "changed" | The "same within rounding" test (§4.2); TL-5's cross-architecture bar; the fingerprint is of the stored form, so a frozen town is never *re-derived* on open |
| R2 | **Frozen towns never improve**: after a user pins a few towns the owner's later town fixes (PS-5, river smoothing) do not reach them | The badge and banner; the bulk "Re-lay all frozen towns" (Q7); the before/after is cheap and undoable |
| R3 | **Too many frozen towns** from pins placed at world zoom near villages | A hamlet costs about 4 KB and a capital at most 216 KB [meas]; all 64 would be 2.4 MB, 3 to 17 % of a format-3 save's rasters; no cap needed. Q2 |
| R4 | **The stretch verdict** (Ruling FQ) depends on three layouts per town and on a cache | A frozen town stores the layout that was drawn, so the verdict is moot while frozen; on re-lay the cache key is recomputed from the new context |
| R5 | A change to `generate` lands **without** a version bump | The fixture-hash test; a change that moves only a switch-gated path is covered by the switch record |
| R6 | A coordinator-visible race: PS-4 and river smoothing move default output while TL-0 is being written | TL-0 lands after both; its first literals are their output |
| R7 | The frozen form cannot reproduce a field the shell later starts to read | `layout_dict`'s key set is asserted against the frozen decode (TL-1); a new key in `layout_dict` without a frozen counterpart turns the test red |
| R8 | **Pin placed while a town is mid-generation** (stretched towns lay out up to three times, seconds on a capital, §1.3) | The freeze runs the same `stretch_layout` path; the placement waits on it behind the busy overlay, as other generation does; measured generation up to 5.4 s |
| R9 | Undo history and freeze interplay (undo of a placement that froze the town) | §5's last row; TL-4's undo bar |
| R10 | Measurement thinness: two seeds, 64 towns at 384 columns, noisy timings, pooled re-attach baseline | Every bar marked * is re-measured on a wider set (3 seeds and a 2 048 world) by TL-1 and TL-4 before it is pinned |

## 10. Owner questions, with defaults

| # | Question | Default (what is built if you do not answer) |
|---|---|---|
| **Q1** | Codec: your ruling says Zstandard. Measured saving is 3.4 % (7 % on the largest capital) and needs §3.3 widened for a binary entry. Keep deflated JSON? | **Deflated JSON, integer delta arrays; §3.3 unchanged** |
| **Q2** | Freeze on a pin in **any** settlement, villages and hamlets included? | **Yes, any settlement** (about 4 KB each) |
| **Q3** | A frozen town through the timeline: one layout in every year (with a chip at a 2x population gap), or one frozen layout per size tier? | **One layout, with the chip** |
| **Q4** | World edited under a frozen town (sculpt, roads, rivers): report only, or offer to re-lay? | **Report with a Re-lay offer; never automatic** |
| **Q5** | Place-editor and Town-plan edits to a frozen town: ask before re-laying? Rename today's **Regenerate** to **New variant**? | **Ask first; rename it** |
| **Q6** | Anchors for POIs only, or also hand-placed labels, icons and measurements inside a town? | **POIs only** |
| **Q7** | Commands "Check frozen towns" and "Re-lay all frozen towns" (with the pooled before/after report)? | **Both, in the command index and the viewer menu** |
| **Q8** | A pin that finds no matching feature: stay in place and be listed (detached), or be deleted? | **Stay and be listed; nothing is deleted** |
| **Q9** | A lock glyph on frozen towns on the map? | **No; the City Viewer only** |
| **Q10** | A manual "Freeze this town" with no pin, for a town you simply want kept? | **Yes** |
| **Q11** | A hand-placed settlement the world later regenerates away (its `id` gone): keep its frozen row verbatim (so a restored world finds it) or drop it with a warning? | **Keep verbatim and report** |
