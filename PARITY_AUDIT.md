# Parity audit — the port's progress claims against the legacy checklist and the real code

**Pass 1, 2026-08-23.** First cross-reference of this repository's own
progress/status documents against (a) `reference/FUNCTION_INDEX.md` as rewritten
the same day (commit `d0667c5` — a full end-to-end analyst read of the frozen
v2.10 file: ~300 user-facing controls in Part 0, all 1094 top-level functions in
Part 1) and (b) the current state of `cartalith-native/`, read and built rather
than taken on a document's word.

**Re-run this as the port continues.** Every finding below is dated to this
pass. The freshest claims are the least audited ones, so a re-run is worth most
immediately after a batch of milestones lands — which is exactly when this pass
found the drift it found.

**This is a findings-only report.** A subsystem that came out clean gets one
line saying so; it does not get its checklist restated. Nothing was "fixed" by
editing an older document to agree with a newer one — contradictions are
reported for the owner to rule on. The one mechanical edit made is disclosed in
§7.

---

## 1 · Summary

### What was reviewed

| Input | Size |
|---|---|
| `reference/FUNCTION_INDEX.md` Part 0 | ~300 controls in 14 groups, 15 dynamic-builder families, 9 keyboard-shortcut rows, 18 canvas interactions |
| `reference/FUNCTION_INDEX.md` Part 1 | 1094 functions in **70 subsystem clusters** across 4 script blocks (633 / 350 / 19 / 92) |
| Progress documents | `STATUS.md` (4 489 lines), `CHANGELOG.md` (17 821), `FUNCTIONAL_CONTRACT.md`, `GUI_GAP_REGISTER.md` (2 027), `ROADMAP.md`, `README.md`, `CLAUDE.md` |
| Scope documents | `MVP_SCOPE` · `PHASE2_SCOPE` · `JOURNEY_PLANNER_SCOPE` · `ECONOMY_SCOPE` · `ASSET_LIBRARY_SCOPE` · `URBAN_MORPHOLOGY_SCOPE` · `TERRAIN_APPEARANCE_SCOPE` · `UNIFIED_TOOL_PLAN` · `GPU_LAYER_INTEGRATION_SCOPE` · `CPU_MULTITHREADING_SCOPE` · `MEMORY_OPTIMIZATION_SCOPE` · `LOD_TILING_BASE_SCOPE` · `ANDROID_BUILD_SCOPE` · `TIMELINE_SCOPE` · `DCC_SHELL_SCOPE` |
| Code | 15 crates, **205 `#[func]` bindings** on `WorldGen`, 24 GDScript files under `godot-project/shell/` |

### Build and test state at audit time

- `cargo build -p cartalith-godot` — **clean** (2 dead-code warnings in `cartalith-gpu`, pre-existing).
- `cargo test --workspace` — **65 suites, 1 136 passed, 0 failed, 4 ignored.**
- Working tree was **dirty from concurrent agents** (see §6); findings were taken
  against the working tree, and where that matters it is said.

### Status counts

Counted at the level of Part 1's own **70 subsystem clusters**, because a
per-function tally would imply a precision this pass cannot honestly claim.
Function-level figures are estimates and labelled as such.

| Class | Clusters | Approx. functions | Notes |
|---|---:|---:|---|
| **(a)** verified equivalent — implementation located in current code | **43** | ~690 | The whole generation pipeline, civ layer, Journey Planner, timeline, asset library, sculpt, region export, LOD tiles |
| **(b)** implemented with a disclosed divergence | **6** | ~45 | All six already disclosed somewhere; §2 lists where |
| **(c)** claimed done, **not verifiable from the repository** | **0 at function level**, but a whole class of claims (§4) | — | No document named a specific ported Rust function that turned out not to exist. The unverifiable claims are all *visual/on-device* ones, by construction |
| **(d)** genuinely not done | **21** | ~360 | Of which **14 user-facing surfaces have no disclosure anywhere** — §5, the most valuable list here |

**The headline positive.** Every code-level claim spot-checked in these
documents held up. `civ_faction_territory_stats`, `jp_journey_cost`,
`explain_settlement`, `as_collections`, `import_heightmap`,
`set_metropolis_enabled`/`set_recovery_phase`, `region_export_tiles`,
`multi::compute_instance`, `get_settlements()`'s `tid`, the Fira font files,
`place_settlements_with_water_edge_snap`'s live call site — all located, all
real. **The drift this audit found is in the *status tables*, not in the code.**

### The contradictions, in priority order

| # | Contradiction | Severity |
|---|---|---|
| **C1** | `CLAUDE.md` and `README.md` both say **"all UI work is on hold"** and cite `DCC_SHELL_SCOPE.md`'s top notice — which says **"✅ THE HOLD IS LIFTED — BUILD IT"**. Eleven UI-building commits have landed since. `ROADMAP.md` Phase 3 carries a third copy | **highest** — it is the auto-loading instruction file, and it tells a fresh session not to do the work the owner has been asking for |
| **C2** | `FUNCTIONAL_CONTRACT.md`'s summary coverage table is stale on **10 of its 22 rows**, and its capability-4 body says the Timeline layer has "zero of it" built. The doc's own closing line reads "Disagreements with prior docs: None found" | **high** — it is the master capability inventory future parity work is told to check itself against |
| **C3** | **Urban morphology appears in neither master inventory.** `FUNCTIONAL_CONTRACT.md` has no capability row for it; `GUI_GAP_REGISTER.md` has zero occurrences of "urban", "city viewer", "town layout" or "morphology" in 2 027 lines — for what `README.md`/`STATUS.md` themselves call "the largest single unported subsystem" | **high** |
| **C4** | `STATUS.md`'s own **"Known-open items"** and four other `- [ ]` boxes contradict `STATUS.md`'s own header: five are closed in fact, verified in code | medium |
| **C5** | `GUI_GAP_REGISTER.md` §5's omission table lists **O4, O5, O7, O8 as open**, while §6.6/§6.9 of the same file mark their counterparts (PR-14, WI-02/WI-03, JP-13, JP-14) **done 2026-08-19** | medium |
| **C6** | `README.md` says Phase 5 is at "milestones 1-5 of ~17"; `STATUS.md` and `URBAN_MORPHOLOGY_SCOPE.md` both say **1-7**. `CLAUDE.md` says "14 crates"; there are **15** | low (one fixed, §7) |

---

## 2 · Divergences — implemented, but not the reference's behaviour (class b)

All six are already disclosed. Listed so a reader of `FUNCTION_INDEX.md` Part 0
walking the checklist does not mistake them for parity.

| Reference behaviour | Port | Where it is disclosed |
|---|---|---|
| Territory auto-assignment `_civAutoPolity` (all-settlement-seeded, unweighted, reach-capped) | Cost-distance Voronoi, capital-seeded, population-weighted | `DECISIONS.md` §7b (owner-closed 2026-08-19); `FUNCTIONAL_CONTRACT.md` cap. 4 |
| Wind/ocean streak trail via `destination-out` canvas fade | Per-particle 12-position history redrawn under `0.86^k` decay | `STATUS.md` header 2026-08-23; `GUI_GAP_REGISTER.md` SH-09 |
| Slicer: second pixel toggle is chroma-key only | Chroma-key **plus** a port-side *Trim transparent edges* the reference has no equivalent for | `cartalith-assets/src/slicer.rs` module docs; `GUI_GAP_REGISTER.md` AS-10 |
| Slicer target: flat target-slot dropdown | Reframed as *Assign to family / Fill from*, composed from the reference's own three targets | AS-11 |
| Way types `road/track/trail/bridge` (spec) | Engine enum `road/track/sea_lane/ancient` | IN-05, resolved in the engine's favour |
| Layer hotkeys `0 B T F S W R` (`LAYER_HOTKEYS`) | Digits **1–8** over `LAYER_GROUPS`' build order | `layers_popover.gd:46-76` — **disclosed against `DCC_SHELL_SPEC.md`, never against the reference.** See §5 item 11 |

Two more that read as divergences and are not:
`asset_library_window.gd`'s **8-family rail vs the reference's 24** (AS-16, owner
decision) and **`Anchor` as a family property rather than per-slot** (AS-15,
engine truth). Both correctly classified (D) in the register.

---

## 3 · Per-subsystem findings

Only items that are **not** a clean "(a) verified equivalent". Subsystems that
came out clean are named at the end of this section rather than expanded.

### 3.1 World generation, terrain, erosion, hydrology (Part 1 block 1)

Core is clean: plates/JFA/boundary classification/stress/orogeny/flexure/
heterogeneity/resistance, the master height formula, volcanism/craters, droplet
and stream-power and thermal erosion, priority-flood + MFD flow, Strahler,
channel widths, river polylines and sinuosity, world-structure archetypes and
the histogram sea-level re-anchor — all located in `cartalith-terrain`,
`cartalith-erosion`, `cartalith-hydrology`, `cartalith-engine`, all with
`golden_parity_*` suites. Tectonic inversion for imported DEMs
(`cartalith-terrain/src/infer.rs` + `cartalith-engine/src/import.rs`) is
bit-exact and now live at `Data ▸ Import ▸ Heightmaps (PNG)`.

**Not done (all with existing disclosure):**

| Reference cluster | Functions | Port state | Disclosed at |
|---|---|---|---|
| Landmass centering (`bestEmptyColumn`/`shiftGridX`/`featherSeamX`/`centerLandmasses`) | 4 | absent; grep `center_landmass` across `crates/*/src` → 0 hits | `GUI_GAP_REGISTER.md` MS-01, disabled button in the WORLD tool-options bar |
| Fjords (`buildFjordMask`/`carveFjords`/`currentFjordMask`/`carveFjordsOp`) | 4 | absent | `sample_bridge.rs:613`'s gap-view list |
| ~~Velocity erosion (Mei virtual pipes) + coastal + glacial + hillslope~~ | ~16 | **DONE (2026-08-23).** Bit-exact kernels in `cartalith-erosion/src/passes.rs` (26 golden tests, 98/115 mutants killed), run as **default-off generation parameters** — `cartalith_engine::ErosionPassParams`, 21 `params.rs` rows. Not the reference's run *buttons*: `DECISIONS.md` §7d, decided at `GUI_GAP_REGISTER.md` §19. **Droplet** is the one of the five still parameter-less | WW-02, tabulated in `GENERATION_PARAMETERS.md` |
| ~~Evolve coupled + sediment routing/deposition~~ | 4 | **DONE (2026-08-23).** Both are orchestration and are transcribed into the same pass block; `evolveCoupled` needed one genuinely new engine function, now written — `cartalith_engine::refresh_climate`, the reference's `computeFlow(true); refreshClimate();` tail over a changed surface | MS-04 / MS-05, §19 |
| ~~Tidal flats~~ | 1 | **DONE (2026-08-24).** The kernel was ported and golden-tested a day earlier; what was missing was the tidal-range field, which `cartalith-climate/src/tides.rs` now produces. `passes.tidal_flats` + `passes.tidal_k` are the **seventh** erosion pass, running last (the reference's own source order), and the toggle doubles as the tides enable — it builds the field from the finished surface right before the kernel reads it, exactly as `refreshTides()` does there. Measured: 19.58 % of every water cell accreted, water only and upward only | MS-05 closes; `GUI_GAP_REGISTER.md` §19.5 |
| ~~Geoid (`buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview`)~~ | 4 | **DONE (2026-08-23)** — `cartalith-climate/src/geoid.rs`, 7 golden tests. The Geoid debug view previews it at the reference's own default amplitude, exactly as `currentGeoidPreview` does while the toggle is off | `GUI_GAP_REGISTER.md` DV-06; WW-07's *parameters* stay open |
| ~~Tides (`tidalForcing`/`computeTideField`/`buildTideField`/`refreshTides`/`currentTideField`)~~ | 5 | **DONE (2026-08-23)** — `cartalith-climate/src/tides.rs`, 6 golden tests. Green's-law shelf amplification and coastal funnelling both verified against a captured reference world | DV-07; WW-07's *parameters* stay open |
| ~~Seasons + Köppen (`computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor`…)~~ | 7 | **DONE (2026-08-23)** — `cartalith-climate/src/koppen.rs`, 6 golden tests. The frozen 30-key order and the Peel et al. palette are verbatim; the classifier is bit-exact against the reference's own captured seasonal fields. The *rain* half inherits `simulate_weather`'s three already-disclosed deferrals, which the module doc states rather than hides | DV-04; WW-09's *parameters* stay open |
| ~~Wildlife + ecoregions (`buildTRI`/`guildTrophic`/`buildEcoregions`/`assignWildlife`/`regionRichness`/`wildRegionColor`/`currentWildlife`)~~ | ~9 (`buildNPP` **is** ported and is consumed, not re-implemented) | **DONE (2026-08-23)** — `cartalith-civ/src/wildlife.rs`, 8 golden tests. In `cartalith-civ` because every input it reads already lives there. The roster click popup came with it — see §5 item 8 | DV-11 (new), and the new register row WL-01 |
| Windthrow (`buildWindThrowField`/`currentWindThrowField`) | 2 | absent | `sample_bridge.rs:601` |
| Landform classification (`buildLandformField`/`currentLandform`) | 2 | absent | `sample_bridge.rs:613` |
| 3D drape (`_m4*`/`_cam3d*`/`_v3d*`/`enter3D`/`exit3D`/`drawSoft`) | ~18 | absent by decision | `DECISIONS.md` §4, PR-06/PR-08 |
| Bake + tile pyramid + IndexedDB atlas + finalize (`bakeAllTiles`/`atlas*`/`setFinalized`/`applyFinalizedUI`) | ~50 | LOD **tile synthesis** is live (`lod_bridge.rs`, `lod_synthesize_tile`/`lod_tile_cells`); the *persistent atlas*, the bake and the finalize-lock are absent | WW-01, PR-10, register S4/S5 |
| Global heightmap undo (`pushUndo`/`undoLast`/`updateUndoUI`) | 3 | absent. Draft-scoped undo/redo **is** real (`cartalith-spatial/src/pass.rs:278`, `sculpt_undo`/`sculpt_redo`) | ED-01/ED-02/PR-11 |
| ~~NPR "Painter" styles, waves, animated water, multi-sun as *controls*~~ | 10 styles + 3 toggles | **DONE (2026-08-23).** Literal per-pixel ports (`render::apply_npr`/`apply_waves`/`multi_sun_from_normal`/`coast_distance`) with a golden suite of their own (`golden_parity_npr.rs`), bound as `WorldGen::get_npr`/`set_npr`, live in the RENDER dock. Animated water is a Godot `ShaderMaterial` overlay instead (per-frame, so principled-equivalent rather than golden — `DECISIONS.md` §7a) | `GUI_GAP_REGISTER.md` RN-02 |
| The remaining **rendering-advanced** toggles as *controls* | parchment, surface texture, sky view factor, ridge crests, ridged relief, slope rock, geology materials, cast shadows, curvature shading, minor channels, wetness, season, SDF coast/river/biome | still absent as controls; several of the underlying effects exist under other names in `TerrainAppearance` (paper, geology, local contrast) but are tier-driven, not exposed | `render_workspace.gd`'s own trimmed disclosure; register RN-01 |
| `exportZip` + `serializeState` + channel atlas + f32 layer previews | ~20 | absent | FI-01/DM-04 ("no save writer") — but the four *header-bar controls* behind it are not itemised; §5 item 14 |

**GeoJSON export was the one genuinely mis-stated row — CLOSED 2026-08-24.**
`cartalith-engine/src/geojson.rs` existed with `golden_parity_geojson.rs`
beside it — nine reference functions ported and verified — but had **no
`#[func]` binding**, which `data_manager_window.gd` and register DM-03 both
described accurately ("(B) wrapper — one `#[func]` plus assembling a
`GeoJsonWorld`"), while `FUNCTIONAL_CONTRACT.md` called it **"Absent"** in both
its body and its summary table. The wrapper is now
`crates/cartalith-godot/src/geojson_bridge.rs` and its caller is the Data
manager's Export ▸ GIS / GeoJSON route; the contract's two rows are corrected.
One reference function had to be ported alongside it —
`cartalith_hydrology::split_river_polylines` (`splitRiverPolylines`, reference
4596), without which a wrapped receiver chain exports as one `LineString`
drawn back across the whole map. Verified in the real app: 305,646 B, 511
features across five layers. `GUI_GAP_REGISTER.md` §20.

### 3.2 Civilisation layer (Part 1 block 2)

Clean: settlement suitability/seeding/placement (with the v1.36 water-edge snap
now on the live path, `crates/cartalith-godot/src/lib.rs:671`), faction
assignment, naming, population and the food-shed ceiling chain, territory,
provinces, villages, the hierarchical road network, sea routes, corridor
consolidation, path smoothing, faction aggregates, culture-terrain fit,
metropolis promotion and the static recovery phase, the whole timeline/collapse
simulation, and Brandes betweenness.

**Not done:**

| Reference surface | Port state | Disclosure |
|---|---|---|
| POI drop, POI pins, POI list (`_civDropPOI`/`_civDrawPoiPin`/`_civRenderPoiList`) | absent; `civ_tools_bridge.rs` says POI "is not a ported concept" | CV-01, and the tool is *omitted rather than built inert* deliberately |
| Add/remove faction, persistent faction identity | absent; `CIV_FACTION_COUNT` is a constant | CV-07 / MS-13 |
| Faction Roster modal — `_civOpenFactionsModal`, `_civPopulateFactionEditor` (culture/religion/government/ag-tech editors, terrain-fit verdicts, settlement sublist), `_civFactionBannerCanvas` | absent. The right dock's Faction context (RD-06/RD-08) is a **read-only summary**, not the editor | **only add/remove is registered (CV-07); the editor and the banners are undisclosed** — §5 item 9 |
| Place edit popup — `_civPopulatePlaceEditor` (name, kind, faction, pop, specialisation, traits, history, walls override, **delete**) | absent. `civ_drop_settlement` creates; **nothing edits or deletes a settlement** | ED-03 frames this as a clipboard/selection gap. **The missing editor itself is undisclosed** — §5 item 3 |
| Right-click context menu `_civCtxShow` (Edit / Move-viewer-to / Delete nearest place / Drop settlement / Drop POI / Info here) | absent. `PopupMenu` appears only in `menus.gd`/`dcc_shell.gd`; no `MOUSE_BUTTON_RIGHT` handler on the map | **undisclosed anywhere** — §5 item 2 |
| `_civAgrarianRegionalTotal` → `civPopEstimateOut` ("Land sustains ≈ N") | absent; no `agrarian_regional_total` in `cartalith-civ` | **undisclosed** — §5 item 7 |
| `civBiomeKChk` (biome carrying-capacity residual toggle) | function present in `cartalith-civ`, **no `#[func]`** | **undisclosed** |
| `civDiagnosticsChk` (placement-diagnostics overlay) | absent | **undisclosed** |
| Committed manual ways never reach the map or a list | was real; `get_roads()` read `civ.ways` only | **IN-02, CLOSED 2026-08-24** — both getters now append `InfraTools::ways` tagged `manual: true` (sea lanes via `get_sea_routes()`), matching the reference's one-flat-`civWays` arrangement; the map repaints on commit and CIVIL ▸ Roads ▸ Hand-drawn lists them. The "routes" half was never a gap — `route_count`/`route_get` predate this |
| Whole-network operations absorbed by `generate()` — auto-populate, clear places/routes, generate roads, recalculate/clear territory, generate provinces | absent as *operations*; the results exist | MS-06…MS-12, all now disabled controls with real reasons |

### 3.3 Journey Planner

Engine-complete (65 of the reference's 74 `jp*`; 6 UI-only, 2 JS idioms, 1
formerly blocked and since ported), with a real Godot takeover view, the Travel
Library behind it, party set-ups, the timeline band and blocked-stage inline
resolutions. `FUNCTIONAL_CONTRACT.md` still carries **"3 of 6 milestones"** — see
C2.

Remaining, all registered: `jpAutoPickTransport`/`_jpRerouteForMode` (JP-01/JP-03),
`jp_journey_cost` ported-but-never-called (JP-04, "the single cheapest (B) in the
register"), the calculation-trace window (JP-05), journey save/registry
(JP-06/JP-08), ⇧-drag spine trim (JP-07), vessel sailing window and a vessel
resolver (JP-09/IN-06).

### 3.4 Urban morphology (Part 1 block 4 + block 2's `_um*` adapter)

**The largest genuine gap, and the least disclosed one.**

- `cartalith-urban` is real and good: 4 516 lines of `src`, milestones 1–7 done
  (RNG substreams, geometry kernel with `js_hypot`, planar street graph, A\* over
  the cost raster, generation rules + culture profiles, the site model, anchors
  and primary routes, organic growth), every module carrying its own `tests/golden.rs`.
- **It has zero consumers.** `grep -rn 'cartalith-urban' crates/*/Cargo.toml`
  returns only its own manifest. `cartalith-godot`'s dependency list does not
  include it. Nothing in `godot-project/` mentions it. The scope document says
  "Not wired to anything" and is correct.
- Milestones 8–17 remain: radial streets/plaza/waterway, water infrastructure,
  fortification, graph cleanup, blocks and parcels, districts and buildings,
  amenities, hinterland/decay/details/metrics, `generate()` orchestration +
  `hashModel`, and the 28-function civ adapter — **~45 of block 4's 92 functions
  plus all 28 adapter functions.**
- **User-facing surfaces with no disclosure in any progress document**:
  `civUrbanLayoutsChk` (draw town layouts at deep zoom), `cityViewerModal` with
  `cvCanvas`/`cvCloseBtn`/`cvLegend`/`cvInfoPanel` and its own wheel-zoom and
  drag-pan, `peCityPreview` (the layout thumbnail inside the place popup) and
  `peCityOpen` (its launcher). See C3.

### 3.5 Asset library

Clean, and the freshest claims verified: `as_collections` is in the `#[func]`
list; `_get_drag_data`/`_can_drop_data`/`_drop_data` are real in
`asset_library_window.gd:250,307`; the slicer is a golden-verified port of
`SpriteSheetImporter` including the half-gutter finding. Open and honestly
registered: "Unassigned imports" (AS-12), per-interior-line dragging and
cell-scoped slicing (AS-17), `as_set_item_transform` (AS-07).

### 3.6 Shell, chrome and session features

Clean: menus, domain rail (three domains since `42547d9` — `dcc_shell.gd:50`
confirms), in-shell file dialogs replacing every stock `FileDialog`, welcome
mode reproducing the reference's own three-button `#onboard` intro, theme
including light and follow-system, credits, scale bar, busy overlay, legend,
Layers popover with 18 live debug views and 11 honestly-unavailable ones, per-class
settlement and by-way-type filters, phone chrome, Fira Sans wired as
`dark_theme.tres`'s `default_font`.

Not done, undisclosed: the resource-inspection overlay (`resOverlay`, Shift+D),
the generation-info parameter dump (`genInfoBtn`/`generationInfoText`), and
Delete-key deletion of a selected place (no `KEY_DELETE` anywhere under
`godot-project/`). Space-hold pan **is** implemented (`viewport_host.gd:379`).

### 3.7 Clean subsystems, named not expanded

Noise primitives · world structure · plates/orogeny/crust · master pipeline and
volcanism · droplet/thermal/stream-power erosion · flow and rivers and features ·
paint layers · planetary units · atmosphere and ocean · biomes and water bodies ·
the affordance stack · tectonic inversion · the Cartalith biome/terrain bridge
and RLE · asset scatter rules and rule-driven map icons · distance fields and SDF ·
material rendering core · the renderer · river channel enforcement · sculpt
editor · region tile export · ZIP/asset-pack read · load and parameter wiring ·
view management · the civ layer's draw helpers, snapping, routing cost model,
auto-routing and network synthesis, population and food model, faction
aggregates, collapse/recovery, auto-world synthesis and state persistence.

---

## 4 · Class (c): claims that cannot be verified from the repository

**No document was found claiming a specific Rust function was ported that turned
out not to exist.** That is worth stating plainly — it is the failure mode this
audit was commissioned to look for, and it did not occur.

What *cannot* be re-verified is a real and growing class, by construction:

1. **Every "verified by looking" claim.** The visual sweep, the Asset library and
   Data manager rebuilds, the wind/ocean streak measurements (0.134/0.052 mean
   frame diff, 57-60 fps), the Devices-menu 479-driver run, the Android device
   passes. All rest on harness scripts and screenshots the project's own
   convention deliberately does **not** commit ("screenshots are not source").
   The claims are specific, internally consistent and stated with unusual
   candour about what was *not* verified — but a later session cannot reproduce
   them without redoing the work.
2. **`region_export_tiles` writing "33 entries, `tiles/index.json` present"** —
   the function and its golden suite are real; the archive round-trip is a
   headless-drive claim.
3. **The wind/ocean animation** is `HEAD` (`bdb6065`) but its own files were
   mid-edit in the working tree at audit time (§6), so the committed state and
   the tree disagreed. Verified against the tree.

**Recommendation, not a finding:** the honesty convention that keeps screenshots
out of the repo is right, but it means the visual-verification claims accumulate
with no residue. A committed, runnable harness scene (as opposed to a committed
screenshot) would make these reproducible without changing the convention.

---

## 5 · Class (d): reference surfaces with **no disclosure anywhere**

These are the rows a reader of `FUNCTION_INDEX.md` Part 0 would find no answer
for in `GUI_GAP_REGISTER.md`, `FUNCTIONAL_CONTRACT.md` or `STATUS.md`. The
register's own §13 audit was run against `design/Cartalith Menu Structure
v2.dc.html` (202 rows), not against the reference source — which is precisely
where these fell through.

| # | Reference surface | `#id` / function | Why it matters |
|---|---|---|---|
| 1 | **Town layouts on the map**, and the whole **City Viewer** modal | `civUrbanLayoutsChk`, `cityViewerModal`, `_cvDrawCity`, `_cvZoomAt`, `peCityPreview`, `peCityOpen` | The visible half of the largest unported subsystem |
| 2 | **Right-click context menu** on the map | `_civCtxShow` — 6 operations | The reference's only path to Move-viewer-to and Delete-nearest-place |
| 3 | **Settlement / POI editing and deletion** | `placeEditPopup`, `_civPopulatePlaceEditor` | `civ_drop_settlement` creates and nothing edits or removes; a user can add a settlement they can never fix or undo |
| 4 | **Delete key deletes the selected place** | block 2 keydown, line 26096 | No `KEY_DELETE` handler exists |
| 5 | **Resource-potential inspection overlay** | `resOverlay`, `updateResOverlay`/`toggleResOverlay`, Shift+D | A live analysis readout, distinct from the Resources debug *layer*, which is built |
| 6 | **Generation-info parameter dump** | `genInfoBtn`, `generationInfoText` | Bug-report affordance; cheap, and the port has `get_params()` already |
| 7 | **"Land sustains ≈ N" modelled-population readout** | `civPopEstimateOut`, `_civAgrarianRegionalTotal` | The only world-level population sanity figure the reference shows |
| ~~8~~ | ~~**Wildlife roster click popup**~~ | `showWildInfo`, `wildFmtPop` | **CLOSED 2026-08-23.** Ecoregions are ported, and the popup came with them: a click on the map while the Wildlife view is drawn calls `WorldGen::wildlife_region_at` (the reference's own `max(8, GW/40)` marker hit radius) and the RIGHT dock renders `showWildInfo` field for field. `wild_fmt_pop` stays engine-side, so the `~4.5M` wording has exactly one implementation. Registered as **WL-01**, so it is no longer a class-(d) row |
| 9 | **Faction Roster modal and its editors** | `civOpenFactionsBtn`, `_civPopulateFactionEditor`, `_civFactionBannerCanvas` | CV-07 registers add/remove only; culture/religion/government/ag-tech editing, terrain-fit verdicts and procedural banners are unregistered |
| 10 | **Procedural faction banners** | `_civFactionBannerCanvas` | (as above) |
| 11 | **Layer hotkeys diverge from the reference** | `LAYER_HOTKEYS` = `0 B T F S W R` vs the port's digits `1–8` | Disclosed against the *spec*, never against the *reference* — a §7d-shaped decision nobody made explicitly |
| 12 | **Biome carrying-capacity residual toggle** | `civBiomeKChk` | Function exists in `cartalith-civ`; no `#[func]` |
| 13 | **Placement-diagnostics overlay** | `civDiagnosticsChk` | Absent |
| 14 | **Export controls in the header bar**, itemised | `bakeRes` (2K/4K/8K), `bakeTiles`, `chanAtlasChk`, `layersPreviewChk` | All four fold into "no save writer" (FI-01/DM-04), but the channel atlas and the f32 layer previews are separate capabilities that no row names |

Items 1–3 are the substantive ones. Item 3 in particular is a **live usability
hole**, not just an inventory gap.

---

## 6 · Concurrent work in flight during this pass

`git status` at the start of this pass showed two other agents mid-edit
(`lod_bridge.rs`, `tile_render.rs`, `viewport_host.gd`, an untracked
`lod_tile.gdshader`; `wind_fx_layer.gd` staged-deleted **and** present as
untracked; `GUI_GAP_REGISTER.md`, `STATUS.md`, `CHANGELOG.md` all `MM`).
**Both landed before this document was committed**, as `4d266de` and `71f1fc8`.
Their claims were re-checked against the settled tree:

- **`4d266de` — Zoom-LOD: the deep-zoom tiles were the reference's Relief view.**
  Verified: `cartalith-terrain/src/tile_render.rs:247` `shade_tile`,
  `godot-project/shell/lod_tile.gdshader:33-35` multiplying the encoded ratio into
  `map_view`'s own texture, `viewport_host.gd:118,774` gating deep zoom on
  `LOD_AUTO_ZOOM = 2.2` as well as px-per-cell — the reference's own
  `LOD_AUTO_SCALE`. The root-cause account (only `renderHeightTileRGBA` was
  ported, and `_lodBuildTileRGBA` picks between it and `renderBiomeTileRGBA` on
  view mode) matches `FUNCTION_INDEX.md` Part 1's own entries at reference lines
  11144/11610/11629. `cargo build -p cartalith-godot` clean and
  `cargo test -p cartalith-terrain -p cartalith-godot` 455/455 after the merge.
- **`bdb6065` / `71f1fc8`** — wind/ocean streak overlay and the Android-phone
  canvas re-sync. The code-level halves verified (§3.6, §2); their
  *visual* halves fall in the class-(c) bucket of §4 like every other
  looked-at-it claim.

**Because `GUI_GAP_REGISTER.md`, `STATUS.md` and `CHANGELOG.md` were dirty for
most of this pass, no edit was made to any of them** — including the mechanical
staleness fixes §7 would otherwise have applied to `STATUS.md`. They are
reported below instead. Note that `4d266de` added 76 lines to `STATUS.md`
*after* this pass read it; the five stale `- [ ]` boxes listed in §7 were
re-confirmed against the post-merge file.

---

## 7 · Edits made by this pass

**One**, disclosed in full:

- `README.md` — the phase table's Phase 5 row read *"milestones 1-5 of ~17"*.
  Changed to **1-7**, matching `cartalith-native/docs/STATUS.md`'s Phase 5
  section ("Milestones 1-7 done (2026-08-18)"), `URBAN_MORPHOLOGY_SCOPE.md`'s own
  milestone-6 and milestone-7 headings (both `**done** (2026-08-18)`), and the
  code (`crates/cartalith-urban/src/routes.rs`, `src/growth.rs`, each with a
  `tests/golden.rs`). Purely mechanical; no open sub-items on either milestone.

**Not edited, reported instead** — each is a real staleness but touches a file a
concurrent agent held, or is a judgment call for the owner:

| Where | What it says | What is true | Evidence |
|---|---|---|---|
| `CLAUDE.md` Constraints; `README.md` L30-32; `ROADMAP.md` Phase 3 | "All UI work is on hold (owner, 2026-08-18)" | The hold was lifted the same day | `DCC_SHELL_SCOPE.md` L3-19 |
| `CLAUDE.md` Contents table | "the Cargo workspace (14 crates)" | 15 | `ls cartalith-native/crates` |
| `STATUS.md` "Known-open items" | "`get_settlements()` carries no `tid`" | It does | `crates/cartalith-godot/src/lib.rs:2670` |
| `STATUS.md` "Known-open items" | "Real Fira Sans/Fira Code font files … deferred" | Sourced, OFL-licensed and wired | `godot-project/fonts/`, `theme/dark_theme.tres:3` |
| `STATUS.md:4550` `- [ ]` | "`lib.rs` (~line 639) still calls the old `place_settlements`" | It calls `place_settlements_with_water_edge_snap` | `crates/cartalith-godot/src/lib.rs:671` (commit `24d3c12`) |
| `STATUS.md:3503` `- [ ]` | "`main.tscn` + `main.gd` … are still in the project" | Deleted | commit `788053b`; both paths absent |
| `STATUS.md:1076` `- [ ]` | "Port `js_atan2` — nothing ported" | Ported | `crates/cartalith-jsmath/src/libm.rs:645`, consumed at `cartalith-hydrology/src/lib.rs:279` |
| `STATUS.md:1080` `- [ ]` | "`cartalith-jsmath` leaf crate. Blocked on the urban fork" | The crate exists; **10** crates depend on it | `crates/cartalith-jsmath/`, `grep -l cartalith-jsmath crates/*/Cargo.toml` |
| `GUI_GAP_REGISTER.md` §5 | O4, O5, O7, O8 listed as open omissions | §6.6/§6.9 of the same file mark PR-14, WI-02, WI-03, JP-13, JP-14 **done 2026-08-19** | same file, two sections |
| `GUI_GAP_REGISTER.md` §3 | 123 entries, (A)17/(B)71/(C)23/(D)12 | ~14 rows have since moved to done; the register says so itself and never re-derived | §3's own "Stale as of 2026-08-20" note |
| `FUNCTIONAL_CONTRACT.md` | 10 summary rows + capability 4's body | see C2, expanded below | — |

### C2 in detail — `FUNCTIONAL_CONTRACT.md`'s coverage table

The document carries two staleness addenda already and still directs readers to
"treat status cells here as of 2026-08-18 morning" — but it remains the named
master inventory, so the rows are worth listing against what is now true.

| Row | Says | Is |
|---|---|---|
| Terrain editing (Sculpt) | Absent, scoped | Built — `UNIFIED_TOOL_PLAN.md` milestone B; `cartalith-terrain/src/sculpt.rs`, `sculpt_bridge.rs`, 34 `#[func]`s, draft undo/redo |
| Journey Planner | 3/6 milestones | 6/6, engine-complete |
| Tile pyramid / LOD / region export | Absent, foundation built | LOD tile synthesis live (`lod_bridge.rs`); region export golden-verified **and** wired to a real route (DM-13) |
| Labels/annotation (manual tools) | Absent, scoped | `label_bridge.rs`, `icon_bridge.rs`, `paint_bridge.rs`, `measure_*` all bound and live |
| Asset library (authoring UI) | Absent | Built, then rebuilt against the design canvas |
| Import (GeoJSON/heightmap) | Absent | Heightmap import + tectonic inversion done and bit-exact. GeoJSON *import* still absent |
| Export (tiles/image) | Absent | Tile export live end to end |
| Save (write) | Absent | Still true |
| Undo | Absent | Draft-scoped undo/redo is real; only **global** undo is absent |
| Analysis-field switching | Ambiguous, unresolved | Resolved — 18 live views + 11 with stated reasons (`sample_bridge.rs`) |
| Capability 4 body | "Absent: the Timeline/collapse-recovery simulation layer — zero of it exists in this port" | Fully built — `cartalith-civ/src/timeline.rs`, `timeline_bridge.rs`, `civ_add_year`/`civ_goto_year`/`civ_year_diff`/`civ_run_collapse_simulation` |
| Closing section | "Disagreements with prior docs: None found." | Eleven, above |

**No urban-morphology row exists at all** — see C3.

---

## 8 · What a second pass should do differently

1. **Walk Part 0 against the shell control-by-control**, not cluster-by-cluster.
   This pass sampled Part 0 and swept Part 1; the fourteen undisclosed surfaces
   in §5 all came from Part 0 rows, which suggests the yield there is higher.
2. **Re-derive `GUI_GAP_REGISTER.md` §3's counts.** The register is the best
   document in this repository and its headline table is the one part of it that
   is knowingly wrong.
3. **Give urban morphology a row in `FUNCTIONAL_CONTRACT.md` and a section in
   `GUI_GAP_REGISTER.md`** before the next milestone batch, so its UI debt starts
   accruing visibly rather than arriving all at once at milestone 17.
4. **Re-run after any batch of three or more milestones.** Both stale-open-item
   clusters found here (`STATUS.md`'s five, `FUNCTIONAL_CONTRACT.md`'s eleven)
   were created by exactly that: work landing faster than the summary tables that
   describe it.
