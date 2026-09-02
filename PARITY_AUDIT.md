# Parity audit — the port's progress claims against the legacy checklist and the real code

> **Passes so far.** [Pass 1 — 2026-08-23](#1--summary) (§1-§8) ·
> [Pass 2 — 2026-08-24](#pass-2--2026-08-24) (§9-§14) ·
> [Pass 3 — 2026-08-25](#pass-3--2026-08-25) (§15-§21). Each pass keeps its own
> findings, dated; nothing from an earlier pass is rewritten, so the three read
> as a trail rather than a snapshot. **Pass 3 carries the standing "what is
> actually left" list** — [§20](#20--what-is-actually-left).

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
| ~~Bake + tile pyramid + persistent atlas + finalize (`bakeAllTiles`/`atlas*`/`setFinalized`/`applyFinalizedUI`)~~ | ~50 | **DONE 2026-08-24.** `cartalith-spatial/src/pyramid.rs` (the addressing), `cartalith-terrain`'s `add_zoom_detail` (the progressive octaves), `cartalith-io/src/atlas.rs` (keys, chunk encode/decode, the portable manifest, and a **filesystem** `AtlasStore` where the reference has IndexedDB), `cartalith-engine/src/bake.rs` (`pyramid_tile`, `bakeAllTiles`/`bakeVisibleTiles`, the `World/` archive both ways, `FinalizeLock`) and `cartalith-godot/src/bake_bridge.rs` + 14 `#[func]`s. 16 golden tests, every one matching first run. Measured on a 2048×1311 world at 1024 px: depth 3 = 85 chunks, 1.64 s, 234 MiB; a deep-zoom read is within one `rg16` LSB of live synthesis. **Two deliberate non-ports, stated in `pyramid_tile`'s own doc:** the reference's `coarseFlow` burn-in and its `featureDetailPass`/`tileErode` extras, both off by default there | WW-01/PR-10/PR-12/SH-07 closed; register S4/S5 re-corrected |
| Global heightmap undo (`pushUndo`/`undoLast`/`updateUndoUI`) | 3 | absent. Draft-scoped undo/redo **is** real (`cartalith-spatial/src/pass.rs:278`, `sculpt_undo`/`sculpt_redo`) | ED-01/ED-02/PR-11 |
| ~~NPR "Painter" styles, waves, animated water, multi-sun as *controls*~~ | 10 styles + 3 toggles | **DONE (2026-08-23).** Literal per-pixel ports (`render::apply_npr`/`apply_waves`/`multi_sun_from_normal`/`coast_distance`) with a golden suite of their own (`golden_parity_npr.rs`), bound as `WorldGen::get_npr`/`set_npr`, live in the RENDER dock. Animated water is a Godot `ShaderMaterial` overlay instead (per-frame, so principled-equivalent rather than golden — `DECISIONS.md` §7a) | `GUI_GAP_REGISTER.md` RN-02 |
| The remaining **rendering-advanced** toggles as *controls* | parchment, surface texture, sky view factor, ridge crests, ridged relief, slope rock, geology materials, cast shadows, curvature shading, minor channels, wetness, season, SDF coast/river/biome | **split, 2026-08-24.** The four with a real engine stage behind them are now live controls — **parchment** (`paper_strength`), **geology materials** (`litho_strength`/`litho_exposure`), **wetness** (`hydro_wet_strength`, live but see CA-11) and **ambient occlusion** — along with the Map view four (relief exaggeration / sun azimuth / sun elevation / relief↔biome), the reference's five **Map style presets**, and eleven port-only appearance fields the reference never had. **Four more landed later the same day** — surface texture (`tex_strength`), ridge crests (`crest_strength`), ridged relief (`ridged_strength`) and curvature shading (`curve_shade`), all literal ports in the reference's own pipeline slots, all live controls. **The remaining six have no engine stage at all** and are unported rather than unbound: sky view factor, slope rock, cast shadows, minor channels, season blend, SDF coast/river/biome | `GUI_GAP_REGISTER.md` RN-03 (closed), CA-01/PR-09 (closed), CA-11 (new) |
| `exportZip` + `serializeState` + channel atlas + f32 layer previews | ~20 | **partly present (2026-08-23)** | `serializeState`'s parameter half and `exportZip`'s seven terrain entries are both real now (`cartalith_io::write_save`, FI-01 closed). Of the *bake* half, `map.png`/tiles and the channel atlas are real as of 2026-08-24 (`WorldGen::export_raster_png`/`export_channel_atlas`, Data manager ▸ Export ▸ World Data) — as **loose files**, not yet assembled into one archive. What stays absent is the f32 layer previews, the civ/UI payload, and the assembly step that would put params, layers, raster and atlas into a single `.zip`; §5 item 14 |

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
| 14 | **Export controls in the header bar**, itemised | `bakeRes` (2K/4K/8K), `bakeTiles`, `chanAtlasChk`, `layersPreviewChk` | **Three of four built 2026-08-24; the fourth is disclosed in the UI.** The 2026-08-24 re-scope above was right about the split and right about the size. (a) `bakeRes`/`bakeTiles` are the **export raster** and are real: `render::bake_pixel`/`bake_rect` run the whole material path at a *fractional* grid position, reached by widening `land_color`/`apply_npr`/`paper_tone`/`apply_border`/`bio_jitter`/`splat_sample` to `f64` coordinates and adding six fractional twins on `RenderCtx` (the reference has two of them itself, `curvatureAtF`/`aspectFactorF`). `tests/bake_raster.rs` pins the property that makes it parity-safe: at every integer cell the bake path and `cell_color` produce the **same exported byte** on a small fixture, and agree to `f32` rounding at scale (the prologue is stored `f32` because the reference's own is a `Float32Array`, so a dozen or so bytes of 8,060,928 come back one level off at 2048x1312 — measured live, and `the_integer_identity_is_f32_tight_not_bit_exact_at_scale` states the bound). `golden_parity_render.rs`' own verification carries over. **The export also composites the river-channel tint** `build_color_texture` draws: a first live comparison at grid resolution came back 291,815 bytes different, all of them rivers, because the tint reads a mask `RenderCtx` does not carry — it is now applied inside `bake_rect`'s pixel loop, before quantization, since a pass over the finished bytes cannot be bit-identical to the screen's single rounding. `WorldGen::export_raster_png` is the binding and Data manager ▸ Export ▸ World Data is the caller. (b) `chanAtlasChk` is real: `cartalith_engine::channel_atlas` (`_chanEnc`/`packRGB8`/`unpackRGB8`/`channelAtlasGroups`/`channelAtlasManifest`/`channelAtlasEntries`), bound as `WorldGen::export_channel_atlas`. (c) `layersPreviewChk` is **still absent** — it belongs with `exportZip`'s f32 layer blobs, which this route does not write either; both are drawn disabled in the pane with that reason |

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

---

# Pass 2 — 2026-08-24

**The re-run §8.4 asked for.** Thirty-three commits landed between pass 1
(`a4fe3b7`) and `3d167eb`, in roughly thirty hours of concurrent-agent work —
far past the "batch of three or more milestones" trigger. This pass reads that
batch against the same progress documents, verifies the four owner
architecture decisions taken inside it, spot-checks the batch's unusually high
number of disclosed *git-index sweep* incidents, and builds and tests the
fully-merged tree rather than trusting any single agent's own green run.

**Same rules as pass 1.** Findings only — a subsystem that came out clean gets
one line. Purely mechanical staleness was fixed and every fix is disclosed in
§13; genuine judgment calls are reported for the owner, not resolved.

---

## 9 · Summary

### What landed, and what was checked

| Input | Size |
|---|---|
| Commits reviewed | **33** (`git log a4fe3b7..HEAD`), every message body read in full |
| New/changed engine modules | `erosion/passes.rs` · `climate/{geoid,tides,koppen,windthrow}.rs` · `civ/{wildlife,roster,urban_adapter}.rs` · `terrain/{center,fjord,landform}.rs` · `engine/center.rs` · `spatial/measure.rs` · `io/save.rs` · `godot/{undo,measure_bridge,urban_bridge,civ_roster_bridge,geojson_bridge}.rs` |
| New shell files | `tool_bar.gd` · `section_strip.gd` · `phone_menu.gd` · `place_editor_window.gd` · `faction_roster_window.gd` · `faction_banner.gd` · `city_viewer_window.gd` · `urban_layout_draw.gd` · `resource_overlay.gd` · `gen_info_dialog.gd` · `water_anim_layer.gd` + `water_anim.gdshader` |
| Progress documents re-read | `STATUS.md` · `CHANGELOG.md` · `GUI_GAP_REGISTER.md` (3 028 lines, up from 2 027) · `FUNCTIONAL_CONTRACT.md` · `ROADMAP.md` · `README.md` · `CLAUDE.md` · `DECISIONS.md` · `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` |
| Sweep incidents spot-checked | **6**, each traced with `git log -S` to the commit that actually carries the content |

### Build and test state at audit time

Run against the fully-merged `3d167eb`, from `cartalith-native/`:

- `cargo build --workspace` — **clean**, exit 0. Two `dead_code` warnings in
  `cartalith-gpu` (`dispatch_gpu_height`, `dispatch_gpu_resistance`) — the same
  two pass 1 recorded as pre-existing. No new warning anywhere.
- `cargo test --workspace` — **129 test binaries, 1 891 passed, 0 failed,
  6 ignored.** Independently re-derived from the raw `test result:` lines, and
  it matches `3d167eb`'s own claim (1 891 / 129) to the number.
- Working tree: clean of tracked changes. Untracked are the vendored
  `ui-ux-pro-max` skill and **eleven `_*_shot.gd`/`.tscn` harness scenes** —
  which is pass 1 §4's own recommendation happening in practice, but only in
  working trees. See F12.

**Growth since pass 1**, measured not quoted:

| Metric | Pass 1 | Pass 2 |
|---|---:|---:|
| `#[func]` bindings on `WorldGen` | 205 | **294** |
| Crates | 15 | 15 |
| `GUI_GAP_REGISTER.md` lines / sections | 2 027 / 15 | **3 028 / 22** |
| Distinct gap IDs in register tables | ~123 claimed | **203** |
| `params.rs` parameter specs | — | **82** |
| `cartalith-urban` consumers | **0** | **1** (`cartalith-civ`) |

### The headline: pass 1's own list was worked, and worked well

Of pass 1 §5's **fourteen reference surfaces with no disclosure anywhere**,
**twelve are now built**, one is disclosed as a disabled control with a real
reason, and one remains open:

| Pass 1 §5 item | Now |
|---|---|
| 1 Town layouts + City Viewer | **built** — `be2d5f7`, `city_viewer_window.gd` + `urban_layout_draw.gd` |
| 2 Right-click context menu | **built** — `e63d5d9`, five of the reference's six ops |
| 3 Settlement editing and deletion (*"a live usability hole"*) | **built** — `place_editor_window.gd`, `civ_delete_settlement` |
| 4 Delete key | **built** — `app.gd`'s `KEY_DELETE` branch |
| 5 `resOverlay` | **built** — `0f8ad05`, *and this audit's own reading of it corrected*: `res` is resolution, not resources; it is a perf HUD, not a resource-potential readout |
| 6 Generation-info dump | **built** — `gen_info_dialog.gd` |
| 7 "Land sustains ≈ N" | **built** — `_civAgrarianRegionalTotal` ported, golden-tested |
| 8 Wildlife roster popup | already closed at pass 1 |
| 9 Faction Roster + editors | **built** — `faction_roster_window.gd`, `civ_roster_bridge.rs` |
| 10 Procedural faction banners | **built** — `faction_banner.gd`, a real `_draw()` port of the Curve2D composition |
| 11 Layer hotkeys diverge from the reference | **still undisclosed against the reference** — see F5 |
| 12 `civBiomeKChk` | **built** — a `CivOptions` flag in File ▸ New world ▸ Generation |
| 13 `civDiagnosticsChk` | **disclosed**, as a disabled control stating the real blocker (it draws `_um*` data) |
| 14 Bake / tile-pyramid header controls | **still open, and re-scoped 2026-08-24** — the 2026-08-23 scoping was wrong: the tile pyramid and the export raster are two different systems that share a verb. See the row itself |

Pass 1's **C1** (`CLAUDE.md`'s UI hold) was fixed in `CLAUDE.md` — and nowhere
else; see F1. **C2** (`FUNCTIONAL_CONTRACT.md`) was fixed in full by `3182364`
and **has already gone stale again**; see F2. **C3** (urban morphology in
neither master inventory) is fully closed: capability 13 exists in
`FUNCTIONAL_CONTRACT.md`, §6.16 and UM-01/02/03 exist in the register, and the
crate has a real consumer. **C4** (`STATUS.md`'s five stale boxes) was fixed by
`8feba29` — and a *different* stale cluster remains; see F6. **C5** (the
register's O-table) is fully closed. **C6** is half-closed: `README.md`'s
Phase 5 row was fixed in pass 1; `CLAUDE.md`'s crate count was not, until now.

### Findings, in priority order

| # | Finding | Severity |
|---|---|---|
| **F1** | **Pass 1's top-severity C1 was only one-third fixed.** `c8ac8fb` corrected `CLAUDE.md`; `README.md` L30 and `ROADMAP.md` L67 kept asserting *"All UI work is currently on hold"* for six more days — and `README.md` is the file `CLAUDE.md` itself sends every session to read first | **highest** — **fixed mechanically, §13** |
| **F2** | **`FUNCTIONAL_CONTRACT.md` went stale again within 24 hours of being corrected**, on at least six rows this same batch closed. It is the named master capability inventory, and its own "Disagreements with prior docs" section was rewritten the day before to say the disagreements were resolved | **high** — reported, not fixed |
| **F3** | **`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 was updated for Q3 and Q4 and not for Q1, Q2 or Q5** — Q1 and Q2 still read as unanswered requests for authorisation, though both were authorised, built, measured and (for Q1) recorded as `DECISIONS.md` §7f. **Q5 — "new coupling is affordable as opt-in parameters" — is recorded nowhere as a standing decision** | **high** |
| **F4** | Q3's own resolution text points future wiring at `GUI_GAP_REGISTER.md` **MS-06**. MS-06 is *"Auto-populate world"*. The real tracking rows are **SG-01…SG-03** | medium |
| **F5** | Pass 1 §5 item 11 (**layer hotkeys `1–8` vs the reference's `0 B T F S W R`**) is still disclosed only against `DCC_SHELL_SPEC.md`, never against the reference. It is the last §7d-shaped decision in that list that nobody has made explicitly | medium |
| **F6** | **`STATUS.md` carries a second stale-checkbox cluster** — its 2026-08-20 register-pass entry still leaves *"The (A) list, remaining 13 entries"* and *"Nine omissions"* unchecked, naming the light theme, JP-13, JP-14, O4/O5/O7/O8 and the Travel library window, **all of which the register marks done**. C4/C5 recurring, from the other side | medium |
| **F7** | **The register's §3 headline counts are further from true than pass 1 found them** — still *"123 catalogued gap entries · (A) 17 (B) 71 (C) 23 (D) 12"* against **203 distinct IDs** across 22 sections. Pass 1's recommendation #2 was not acted on, and `STATUS.md` restates the same figures | medium |
| **F8** | **One new gap this batch found was disclosed only in `CHANGELOG.md`, never registered**: `DccWidgets.note()`'s 240 px `custom_minimum_size.x` puts a ~272 px floor under any dock context that draws a note, above `DccTheme.W_RIGHT_DOCK_MIN` (260) — so the right dock cannot be dragged to its own documented minimum. Every other "still open" item this batch left *is* registered | medium |
| **F9** | The register cites **`params.rs`'s "58 entries"** in three places; there are **82** (`+21` erosion, `+10` GeoJSON/tides, and others). SG-03's whole cost estimate rests on that number | low |
| **F10** | `README.md`'s Phase 2 row still read *"65 of the reference's 74 `jp*` functions … 1 blocked"*; `JOURNEY_PLANNER_SCOPE.md` records **66 of 74** with the blocked line empty | low — **fixed mechanically, §13** |
| **F11** | `CLAUDE.md` still said **"14 crates"**; there are 15. Reported by pass 1 §7 and not picked up by `c8ac8fb`, which edited the same file | low — **fixed mechanically, §13** |
| **F12** | Eleven `_*_shot.gd`/`.tscn` verification harnesses exist **untracked in the working tree** and are cited by name in commit messages and `CHANGELOG.md` as evidence. Pass 1 §4 recommended exactly this artefact — as a **committed** one. As it stands the citations point at files a fresh clone does not have | low, but it is pass 1 §4's recommendation half-taken |

---

## 10 · The disclosed sweep incidents — verified, not taken on trust

This batch had an unusually high rate of concurrent-agent git-index collisions:
several commits staged whole shared files and swept in a sibling agent's
in-flight work, or had their own work swept away. Every one was disclosed in a
commit message as *"content verified intact, just under the wrong commit
message."* **Six were independently re-checked here**, each by tracing the
claimed content to the commit that actually carries it with `git log -S`, and
by confirming it is present and correct at `HEAD`.

| # | Disclosure | Verdict |
|---|---|---|
| 1 | `b7a46a7` (geoid/tides/Köppen/wildlife): *"Docs for this pass … landed in `1f7c295`, which swept them from the shared index"* | **True and intact.** `git log -S 'cartalith-civ/src/wildlife.rs' -- docs/CHANGELOG.md` → `1f7c295`. DV-04/DV-06/DV-07/DV-11 and the new WL-01 are all present in the register with their golden-test counts, and every claimed module exists |
| 2 | `1f7c295` (erosion passes): *"The four shared narrative documents also carry two concurrent agents' in-flight entries (the NPR block and the geoid/tides/Köppen/wildlife cluster)"* | **True and intact.** `git log -S 'golden_parity_npr.rs' -- docs/CHANGELOG.md` → `1f7c295`, i.e. the NPR entry really did land under the erosion commit, exactly as stated. `golden_parity_npr.rs` and `render.rs`'s NPR block both exist and pass |
| 3 | `e63d5d9` (civ interaction): *"`GUI_GAP_REGISTER.md` §18 and the CV-01/CV-07/MS-13/ED-03/UM-03 row edits … were swept into `be2d5f7`"* | **True and intact.** `git log -S '## 18 · The civ-interaction surface' -- GUI_GAP_REGISTER.md` → `be2d5f7`. §18 is present in full at `HEAD`, with §18.1's five closed rows, §18.2's POI re-check and §18.3's ED-03a…d + UM-03 residuals |
| 4 | `5f9d5f9` (Journey Planner): *"`GUI_GAP_REGISTER.md` rows for these were swept into a concurrent agent commit; the content is theirs to the letter"* | **True and intact.** The JP-01 and JP-04 closure text traces to `75a9269` (the four-clusters commit). All seven rows — JP-01/03/04/05/07/09 and IN-06 — are present, closed, and describe code that exists |
| 5 | `b0b095f`: *"The previous commit added `geojson_bridge.rs` but not the `mod` line that compiles it, because `lib.rs` was mid-edit by a concurrent change"* | **True and repaired.** `mod geojson_bridge;` is `lib.rs:23`; `export_geojson` is a real `#[func]`; the workspace builds. This is the one sweep that actually **broke** something, and it was caught and fixed inside the same hour |
| 6 | `92699da`: doc-comment mojibake and a comment landed above the wrong function, both *"from a merge"* | **True and repaired.** `grep` for `Â§`/`â€`/`Ã©` across `crates/cartalith-godot/src/*.rs` returns nothing |

**Nothing was lost.** A file-existence sweep over every module and shell script
named in the batch's commit messages — 31 paths — found **zero missing**, and
every new Rust module is registered in `lib.rs`'s `mod` list. The disclosure
convention held: in this batch it was accurate every time it was used, and the
one real casualty announced itself.

---

## 11 · The four owner architecture decisions

`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` (`5b0f2aa`) closed with six open
questions. Four were answered by the owner during this batch.

### Q3 — the staleness graph's consumer · **REAL, not a stub** ✅

`cartalith_engine::staleness::recompute_stale(&mut StageGraph, &WorldParams,
&mut WorldState) -> RecomputeReport` at `crates/cartalith-engine/src/staleness.rs:205`
is a working function, not a placeholder: it gates on `any_stale(hydro) ||
any_stale(clim)`, calls the real `refresh_climate` over `ws.field` writing
`ws.temperature`/`ws.rainfall`/`ws.flow_discharge`, then bumps hydrology's
version *before* climate's with a load-bearing comment explaining the order.
It is wired at four call sites in `cartalith-godot/src/lib.rs` (`sculpt_commit`,
`carve_fjords`, `paint_commit`, and the new `#[func] recompute_stale_stages()`),
not merely exported. Its own no-op guard (`n == 0` or a length mismatch) returns
`RecomputeReport::default()` rather than panicking across the gdext boundary,
which is the right shape.

### Q4 — erosion inside the height stage · **matches the real structure** ✅

`pipeline_stage_graph` is still exactly four acyclic nodes —
`Height` (no upstreams) → `Hydrology` → `Climate` → `Civ` — with **no `Erosion`
node, no new edge and no new stage kind**, which is what candidate (a) means
concretely. The decision is not just prose: `staleness.rs:311` is
`the_owners_erosion_decision_keeps_the_graph_at_four_acyclic_stages`, a real
test asserting the graph shape, so reversing it has to be argued rather than
drifted into. The module doc carries the reasoning. **Verified against the file,
not the commit message.**

### Q5 — opt-in coupling as a standing pattern · **NOT recorded anywhere** ❌

This is F3's sharp end. `DECISIONS.md` runs to §7f and has **no section, and no
occurrence of "opt-in coupling", "vegetation", "Cordonnier" or "feedback"** at
all. The research document's §4 item 5 still reads as an **open question**
(*"Does the owner want more feedback anywhere, or less?"*), unstruck, with the
opt-in framing offered as an observation rather than recorded as an approved
pattern. If the owner has approved "new coupling ships as an opt-in parameter,
default off, reproducing reference behaviour when off" as the standing rule,
**that decision currently exists only in a chat transcript.** `ErosionPassParams`
demonstrates the pattern and `DECISIONS.md` §7d authorises the *superset* shape
it uses — but §7d is about behaviour-vs-implementation, not about coupling, and
a future agent reading `DECISIONS.md` will not find the rule. **Recommended: a
`DECISIONS.md` §7g.** Not written here, because inventing the owner's wording
for a decision is exactly what this audit is not for.

### Q6 — the civ-crate move · **correctly deferred, and nothing did it** ✅

`compute_civilisation` is still `crates/cartalith-godot/src/lib.rs:616` — the
only definition in the workspace. No stray copy appeared in `cartalith-engine`
or `cartalith-civ`. Register row SG-02 depends on this being true and states it
correctly.

### The research document's own open-questions section · **partly updated** ⚠️

The prior agent's claim to have updated §4 holds for **Q3 and Q4 only**, both
struck through with a full IMPLEMENTED note. **Q1 and Q2 were left untouched**
and still read as requests for permission —

- Q1 (*"Is a measured experiment on the apparently-dead pre-carve
  `compute_flow` authorised?"*) — **it was**: `ac1de2d` skips it when carving
  runs, proved bit-identical over six fixtures, measured at 8.9-9.9 %, and
  disclosed as **`DECISIONS.md` §7f**. The question is answered in another file
  and not marked answered in its own.
- Q2 (*"Is the radix-sort substitution worth a measurement pass?"*) — **it was
  done**: `ae260cd` ports `_flowRadixSortDesc`, 11.08× on the sort, 20.0 % off a
  2048² generation, element-identical across twelve fixtures.

And F4: Q3's resolution text says the future UI wiring is *"tracked as
`GUI_GAP_REGISTER.md` MS-06"*. MS-06 is **Auto-populate world**. The staleness
UI is **SG-01** (the indicator), **SG-02** (a Recompute-now control) and
**SG-03** (`param_set` marking the graph) — all three real rows in the
register's new §21, all correctly open.

---

## 12 · New gaps this batch introduced — registration check

Several agents left explicit "still open"/"deliberately not built" items in
their own reports. Each was checked for real registration rather than a mention
in a commit message. **All but one are registered.**

| Item, as its own agent stated it | Registered? |
|---|---|
| An edited **specialisation** does not reach `civ_faction_aggregates`' sector output | ✅ **ED-03a**, with the real reason (feeding user edits in would move already-golden economy numbers on an interactive edit) |
| **age**/**walls** stored and consumed by nothing; the seven **traits** never drawn | ✅ **ED-03b**, **ED-03c** |
| A place edit or delete does **not** recompute provinces / trade / roads / territory / explanations | ✅ **ED-03d**, and stated in `civ_delete_settlement`'s doc comment *and* the delete dialog's own text |
| Dock content does not drag-to-scroll on phone | ✅ **PH-05** (§22) |
| `new_world_dialog.gd` / `browse_dialog.gd` still desktop-sized on phone | ✅ **PH-06** (§22), and also still open in `STATUS.md`'s phone section |
| **SG-02** (civ-rebuild recompute binding) and **SG-03** (per-parameter staleness marking) | ✅ both, §21, each with the design decision that blocks it |
| `ViewportHost._zoom_at()` pivots against the wrong origin, measured at (412, 70) | ✅ **SH-11**, classified (A), open |
| No `way_set_name` / `way_delete` for a committed manual way | ✅ **MS-09**, updated in the same pass that closed IN-02 |
| GeoJSON **import**, CRS anywhere, the untested sea-lane export path | ✅ DM-03 / DM-07, and `FUNCTIONAL_CONTRACT.md`'s absent list |
| Journey persistence across sessions (JP-06 / JP-08) | ✅ both, with a *"Ceiling update"* re-stating what actually remains now that the writer exists |
| Urban milestones 8-17, and the six unported `_um*` functions | ✅ §6.16 / UM-01…03, `URBAN_MORPHOLOGY_SCOPE.md`, and the adapter's own module header |
| **`DccWidgets.note()`'s 240 px minimum still fights the right dock's 260 px** | ❌ **`CHANGELOG.md` only.** No register row. **F8** |
| *"The left dock and the workspace panels were not audited for the same fault"* | ❌ same entry, same gap — an explicitly-unaudited surface with no row |

`GUI_GAP_REGISTER.md` is, again, the best-maintained document in this
repository: it gained seven whole sections (§16-§22) and ~80 rows in this batch
and every one of them is real, dated and evidenced. F8 is one row's worth of
omission against that; it is listed because the pattern this audit exists to
catch is exactly "disclosed in a narrative document, never registered."

---

## 13 · Edits made by this pass

**Four, all purely mechanical, all disclosed.** Each aligns a stale statement
with a fact already recorded elsewhere in the repository; none resolves a
judgment call.

1. **`README.md` L30-32** — *"All UI work is currently on hold"* → the hold is
   lifted, mirroring the correction `c8ac8fb` had already made in `CLAUDE.md`
   and the verbatim owner instruction at the top of `DCC_SHELL_SCOPE.md`. This
   is pass 1's **C1**, whose highest-severity framing was *"it tells a fresh
   session not to do the work the owner has been asking for"* — and `README.md`
   is the file `CLAUDE.md` sends every session to first.
2. **`ROADMAP.md` Phase 3** — the same sentence, the same correction.
3. **`CLAUDE.md` Contents table** — *"the Cargo workspace (14 crates)"* → **15**
   (`ls cartalith-native/crates` → 15; `README.md` already said "Fifteen").
   Reported by pass 1 §7 and missed by the commit that edited this same file.
4. **`README.md` Phase 2 row** — *"65 of the reference's 74 `jp*` functions …
   1 blocked"* → **66 of 74, nothing blocked**, per
   `JOURNEY_PLANNER_SCOPE.md`'s own 2026-08-23 update section and
   `cartalith_civ::jp_reroute_for_mode` existing in code.

**Deliberately not edited, reported instead:**

- **`FUNCTIONAL_CONTRACT.md` (F2).** Six-plus rows, and the fix is a
  whole-document pass of the kind `3182364` already did once. Specifically:

  | Row / list entry | Says | Is |
  |---|---|---|
  | `Map rendering (NPR/geology/SDF toggles)` | Absent | NPR Painter block built — ten styles, coastal waves, animated water, multi-sun (`65f0f85`, `golden_parity_npr.rs`, live in the RENDER dock). The geology/SDF half is still genuinely absent |
  | `Save (write)` | Absent | `cartalith_io::write_save` + `WorldGen::save_project`; File ▸ Save / Save as / Autosave / Revert / Close all real (`749d3b5`, FI-01 closed) |
  | `Undo` | Draft-scoped real; global still absent | Global undo built — `cartalith-godot/src/undo.rs`, Edit ▸ Undo (Ctrl+Z), Preferences ▸ Memory ▸ Undo history (`11fe6cc`, ED-01/PR-11) |
  | absent-list: *"Save writing: `.zip` save is entirely unbuilt (read-only so far)"* | — | false |
  | absent-list: *"NPR Painter styles … unscoped"* | — | the Painter styles specifically are built |
  | absent-list: *"Global (generation-parameter) undo: absent"* | — | built |
  | absent-list: *"Journey Planner's individually-registered gaps"* (six named) | — | five of the six closed by `5f9d5f9`; only journey save/registry remains, and only partly |
  | absent-list: *"Settlement/place editing and deletion, and the map's right-click context menu"* | — | both built by `e63d5d9` |
  | `Hydrology/climate/ecology live tuning` | Absent | **judgment call** — `recompute_stale` now makes a committed edit reach hydrology and climate without a full regenerate, which is part of this row. "Live tuning" from a *dial* is still absent (SG-03). Left for the owner to phrase |

- **`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 (F3, F4).** Q1 and Q2
  need striking through with their real outcomes, Q3's MS-06 reference needs to
  become SG-01…SG-03, and Q5 needs either a `DECISIONS.md` §7g or an explicit
  "still open". All four are the research author's own to write; a third party
  restating an owner decision is how a decision loses its provenance.
- **`STATUS.md`'s second stale-checkbox cluster (F6)** and its restatement of
  the register's stale headline counts.
- **`GUI_GAP_REGISTER.md` §3's counts (F7)** and the `params.rs` "58 entries"
  figure (F9). Re-deriving the A/B/C/D split is a classification pass, not an
  arithmetic one — it needs a judgment per row about whether a design now
  exists, which is the register author's call.
- **F5**, the layer-hotkey divergence: a §7d-shaped decision that has still not
  been made, and this audit is not the place to make it.
- **F8**, the unregistered `note()` floor: one register row, but writing it
  means choosing its class and its cost estimate.

---

## 14 · What a third pass should do differently

1. **Audit `FUNCTIONAL_CONTRACT.md` first, not last.** It has now gone stale
   twice, and both times for the same structural reason: it is a *summary*
   table that no feature commit is obliged to touch, unlike
   `GUI_GAP_REGISTER.md`, which every agent updates because it is where their
   own row lives. Either give each capability row an owning register ID, or
   stop treating it as authoritative.
2. **Walk Part 0 control-by-control.** Pass 1 recommended this, pass 2 did not
   do it either — this pass followed the batch rather than the checklist,
   which was the right call given 33 commits, but the recommendation stands
   and is now two passes old.
3. **Re-derive the register's §3 counts** — pass 1's recommendation #2,
   unactioned, and the drift has roughly doubled (123 → 203).
4. **Commit the `_*_shot` harnesses** (F12). Eleven of them exist, they are
   cited as evidence by name in `CHANGELOG.md`, and a fresh clone does not have
   them. Pass 1 §4 asked for exactly this; the harnesses got written and then
   left outside the repository by the *"screenshots are not source"*
   convention, which was never meant to cover the driver.
5. **Re-run after the next batch.** The trigger held: this pass found twelve of
   pass 1's fourteen undisclosed surfaces closed, and *also* found the master
   inventory stale again within a day. Both facts are the same fact — the port
   is moving faster than its summary tables, and only a periodic sweep notices.

---

# Pass 3 — 2026-08-25

**The re-run §14.5 asked for.** Eighty-two commits landed between pass 2
(`3d167eb`) and `10f7c9c` — the v3 left-rail restructure, the Markdown vault
subsystem, IN-13 trade flows, VA-01 vault backlinks, ED-02 the undo ledger,
CV-23 contested-borders influence, CV-25/CV-26 military and faction relations,
a four-output military manpower model, and a long run of rendering and
conformance fixes. Well past the "batch of three or more milestones" trigger,
and more than twice pass 2's batch.

**This pass was run against a specific instruction**, and it is the right one:
*do not trust a register entry's stated reason.* The session that produced this
batch found **seven** items registered as "no backing capability" that were
already built. So every item reported below as outstanding was checked against
the real crates and against `reference/FUNCTION_INDEX.md` before being written
down, and the ones whose engine half already exists are sorted separately —
they are a different and much cheaper kind of work.

**Same rules as passes 1 and 2.** Findings only. Purely mechanical staleness
was fixed and every fix is disclosed in §21; genuine judgment calls are
reported for the owner, not resolved.

---

## 15 · Summary

### What landed, and what was checked

| Input | Size |
|---|---|
| Commits reviewed | **82** (`git log 3d167eb..HEAD`) |
| New crate | `cartalith-vault` — the sixteenth, and the only one that is not engine work |
| New engine/bridge modules | `civ/{trade,military,manpower,relations,territory_influence}.rs` · `civ_continents` · `godot/{civ_trade_bridge,civ_military_bridge,vault_bridge,bake_bridge,export_raster,undo}.rs` · `engine/bake.rs` · `urban/{plaza,blocks}` |
| Progress documents re-read | `STATUS.md` · `GUI_GAP_REGISTER.md` (**6 771** lines, up from 3 028) · `FUNCTIONAL_CONTRACT.md` · `ROADMAP.md` · `README.md` · `CLAUDE.md` · `DECISIONS.md` · `URBAN_MORPHOLOGY_SCOPE.md` · `JOURNEY_PLANNER_SCOPE.md` · `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` |
| Register sections added since pass 2 | **§23-§44** — twenty-two, against pass 2's seven |

### Build and test state at audit time

Run against `10f7c9c`, from `cartalith-native/`:

- `cargo check --workspace --all-targets` — **clean, exit 0.** No warnings
  surfaced. The two `dead_code` warnings in `cartalith-gpu` that passes 1 and
  2 both recorded as pre-existing no longer appear.
- `cargo test --workspace --no-fail-fast` — **138 test binaries, 2 203 passed,
  8 ignored.** The suite count matches the last recorded full run exactly.
- **One test fails intermittently. See F1** — it is the only red anywhere in
  this pass, and it is not a stale-document finding.
- Working tree: **clean of tracked changes.** Untracked are the vendored
  `ui-ux-pro-max` skill and **152 verification-harness files**. See F8.

**Growth since pass 2**, measured not quoted:

| Metric | Pass 1 | Pass 2 | Pass 3 |
|---|---:|---:|---:|
| `#[func]` bindings in `cartalith-godot` | 205 | 294 | **402** |
| Crates | 15 | 15 | **16** |
| GDScript files under `godot-project/shell/` | 24 | — | **43** |
| `GUI_GAP_REGISTER.md` lines / sections | 2 027 / 15 | 3 028 / 22 | **6 771 / 44** |
| Distinct gap IDs, by the register's own method | ~123 claimed | 215 recounted | **260** |
| `cartalith-urban` Rust lines (incl. tests) | — | — | **18 568** |
| `STATUS.md` size | 4 489 lines | — | **7 499 lines / 689 KB** |

### The headline: the anti-pattern is being caught faster than it is created

Pass 1 found fourteen undisclosed surfaces; pass 2 found twelve of them built.
This pass found something better: **the batch caught its own instances.**
Register §39 checked all fifteen of §37's new IDs against the crates before
writing code and found **four already built** (WW-14 ecology productivity,
CV-21 faction colour, CA-19 the biome legend, WW-15 the CRS declaration), and
§40/§42/§43 found four more (the reference military model, IN-13's bipartite
match and union-find, and two inert tables). That is the discipline working.

**What this pass adds is that the same anti-pattern has migrated into the
narrative documents.** Every one of §16's five findings is a *document* that
describes as missing something the code has. The register is now the
best-maintained document in the repository by a wide margin; the summary
tables that no feature commit is obliged to touch are where the drift lives —
which is precisely pass 2's §14.1 prediction, holding for the third pass
running.

### Findings, in priority order

| # | Finding | Severity |
|---|---|---|
| **F1** | **`generate_terrain_gpu_path_is_deterministic_and_valid` fails intermittently** — 2 of 6 full-workspace runs, 0 of 6 in isolation. The two runs differ by ~1 ulp of `f32`, and the assertion is `assert_eq!` on the whole field. This is the only failing thing in the port, and it may be **testing a bar the owner has already relaxed** | **highest** — reported, needs a decision |
| **F2** | **Urban morphology milestones 8a and 12 shipped on 2026-08-24 and three documents still said "milestones 1-7 done, 8-17 open"** — `ROADMAP.md`, `README.md` and `FUNCTIONAL_CONTRACT.md`, the last of which named **blocks/parcels** in its absent-entirely list a day after milestone 12 built it | **high** — **fixed mechanically, §21** |
| **F3** | **Pass 2's F10 recurring in the document pass 2 did not fix.** `ROADMAP.md` Phase 2 still read *"65 of the reference's 74 `jp*` functions … 1 blocked on a Route-tool pathfinder since ported"*. Same shape as pass 2's own F1: a correction applied to `README.md` and not to `ROADMAP.md`'s copy of the identical sentence | medium — **fixed mechanically, §21** |
| **F4** | **`FUNCTIONAL_CONTRACT.md` has gone stale for the third consecutive pass**, and one entry was **stale on the day pass 2 wrote it**: pass 2 corrected the SG-03 row as a "judgment call" on 2026-08-24, and SG-03 closed on 2026-08-24. Also wrong: the persistent atlas/bake bullet, and ambient occlusion listed as an unscoped absent toggle while shipping with two live sliders | **high** — **fixed mechanically, §21** |
| **F5** | **Three register rows in §6 state a reason the engine has since falsified** — RD-01 (defensibility), RD-02 (settlement↔route association) and RD-07 (state religion). All three are "already built, just unwired", the exact class §39 exists to catch, sitting in the oldest part of the register where §37-§44's re-checks never reached | **high** — reported (register is concurrent territory) |
| **F6** | **The register's §3 headline count is stale again** — 215 against **260** by the section's own disclosed method, its prefix list omits **five** prefixes now in use (`MR`, `TO`, `MN`, `VA`, `KV`), and it says *"six `### PH-0N ·` entries"* where there are **eleven**. Pass 1 rec #2, pass 2 F7, pass 3 F6 | medium — reported |
| **F7** | **`STATUS.md` has stopped being a quicklist.** 7 499 lines / **689 KB** — 41 % of `CHANGELOG.md`'s size — while its own first line reads *"A living checklist, not a narrative"* and `CLAUDE.md` calls it the authoritative status and sends every session to it before anything else. A single "Last updated" entry now runs to ~2 KB of prose | medium — structural, reported |
| **F8** | **Pass 2 rec #4 unactioned, and the drift is 6.5× worse.** Pass 2 found 11 untracked `_*_shot`/`_*_probe` harnesses; there are now **72 `.gd` + 71 `.tscn` + 9 `.png`**, cited by name as evidence throughout `CHANGELOG.md` and the register, and absent from a fresh clone | medium |
| **F9** | **`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 has gone further wrong, not less.** Pass 2's F3 is half-closed (Q5 now has `DECISIONS.md` §7g ✅); Q1 and Q2 still read as unanswered authorisation requests though both are built (`DECISIONS.md` §7f; `flow_sort_desc`). Pass 2's F4 is unactioned **and Q3's text now carries three wrong sentences**, not one | medium |
| **F10** | **Pass 1 §5 item 11 / pass 2 F5 is now three passes old**: the layer hotkeys `1–8` are still disclosed only against `DCC_SHELL_SPEC.md` §10, never against the reference's own `LAYER_HOTKEYS` (`0 B T F S W R`). `grep` finds `LAYER_HOTKEYS` nowhere in the register or the spec | low, but it is the last unmade §7d decision from pass 1's list |

**Closed since pass 2, verified rather than assumed:** pass 2's F6
(`STATUS.md`'s second stale-checkbox cluster — 7 unchecked boxes remain, none
of them the named ones), F8 (`DccWidgets.note()`'s 240 px floor — registered
as **SH-12** *and fixed*, 240 → 190, including the left-dock/workspace half
pass 2 flagged as unaudited), and F3's Q5 half (`DECISIONS.md` §7g now
exists). **Pass 1's C1 has not regressed**: `CLAUDE.md`, `README.md` and
`ROADMAP.md` all now state the hold as lifted, and `CLAUDE.md`'s crate count
reads 16.

---

## 16 · The already-built sweep — five documents describing capability that exists

The instruction this pass was given. Each row below was checked against the
crates, not against the document that claims it.

### F5a · RD-01 — Settlement ▸ Defensibility

**The row says** (`GUI_GAP_REGISTER.md` §6): *"(B) small — the reference itself
treats it as a UI-only categorical."* **The shell says**
(`right_dock.gd:597-600`), drawing a literal `—`:

> `explain_settlement()`'s suitability terms have no defensibility axis --
> gentle_slope/terrain_form are the closest inputs but the engine doesn't
> label either one defensibility.

**What is true.** `cartalith_civ::civ_place_defensibility`
(`cartalith-civ/src/military.rs:166`) is a golden-verified port of the
reference's own `_civPlaceDefensibility` (reference line 23802), landed by
register §40. `civ_military_bridge.rs:168` computes it per place and `:599`
exposes it by name; `civilization_workspace.gd:1595` already **sorts walled
places by it**. The dock note is correct about `explain_settlement` and wrong
about the engine — it names the one function that does not carry the axis, in
a build where another one does.

**Real remaining scope:** point the field at `civ_military_summary()` instead
of `explain_settlement()`. Not a port.

### F5b · RD-02 — Settlement ▸ Routes

**The row says:** *"(B) small — needs a settlement↔way association the engine
does not model."* **The shell says:** *"Roads and sea routes carry no
settlement index … nothing associates a route with this settlement."*

**What is true.** IN-13 built exactly that association.
`cartalith-civ/src/trade.rs`'s `WayRouter` (`:550-639`) runs a real Dijkstra
over the way graph and caches `prev_way` **per source settlement**, then walks
it back edge by edge in `accumulate()`; `TradeFlow` carries `from` and `to` as
settlement indices, and `way_load` is per way in `get_roads()` order. The
association is computed for every one of the verification world's 624 flows —
and then aggregated away into a scalar per way.

**Real remaining scope:** retain or re-walk the already-cached path; the
per-source cache makes the second walk nearly free. That is a much smaller
item than "the engine does not model it", which is what the row currently
tells a future session.

### F5c · RD-07 — Faction ▸ State religion

**The row says:** *"(B) wrapper — `civ_faction_aggregates` is golden-verified
and unexposed."*

**What is true.** `civ_faction_aggregates` is **exposed** — it is called
inside a live `#[func]` at `cartalith-godot/src/lib.rs:10987` (the Faction
Roster's "Territory fit" panel) and again at `civ_military_bridge.rs:262`.
More than that, the religion input is **already fed from the roster** in both
places (`faction_has_religion: Some(&has_religion)`, `lib.rs:10983` and
`civ_military_bridge.rs:254`), and `religious` is a real computed power axis
that feeds `overall` (`cartalith-civ/src/lib.rs:2917-2937`).

**Real remaining scope:** surface one field the O(cells) pass already
computes. The row's "wrapper" class is right; its reason is not, and the
reason is what makes it look like a binding still has to be written.

### F5d · `FUNCTIONAL_CONTRACT.md` — three absent-list entries that are built

| Entry | What is true |
|---|---|
| *"Dial/slider-triggered live re-tuning (SG-03): a moved parameter still marks nothing stale"* | **SG-03 closed 2026-08-24**, the same day pass 2 wrote this bullet. `set_params`' own doc comment reads *"**Marks the staleness graph** (`GUI_GAP_REGISTER.md` SG-03) for the 25 keys that have a live-apply path"*, and the register's §21 SG-03 row reads CLOSED |
| *"Persistent LOD tile atlas/cache, and the bake/finalize-lock: nothing persists it to disk yet"* | `cartalith_engine::bake`'s `AtlasStore` writes chunks under a real disk root with a manifest, and imports an atlas back out of a zip (`atlasImportEntries`, reference 10910). `bake_bridge.rs` carries `BakeState::root` and the finalize lock. Three commits: `e0dfa44`, `948e15a`, `3689d7b` |
| *"AO/SVF/shadow toggles … presentation-only, unscoped"* | **Ambient occlusion is `render.rs`'s milestone 2 and ships**, with two live sliders in `render_workspace.gd` (`ao_strength`, `ao_radius_frac`) whose tooltip calls one *"the reference's own Ambient occlusion slider"*. SVF/cast-shadow and SDF genuinely remain on `render.rs`'s own excluded list; AO does not |

### F5e · Three documents on urban morphology

`URBAN_MORPHOLOGY_SCOPE.md`'s own milestone headings record **milestone 12
(blocks and parcels) done 2026-08-24** and **milestone 8a (the plaza) done
2026-08-24** — the latter is commit `5ea486c`, *"The town has a market square,
because buildPlaza is ported"*. `ROADMAP.md`, `README.md` and
`FUNCTIONAL_CONTRACT.md` all still said "1-7 done, 8-17 open", and
`FUNCTIONAL_CONTRACT.md`'s absent list named **blocks/parcels** among the
things not built.

---

## 17 · `ROADMAP.md` phase by phase, verified against code

| Phase | Documented | Verified | Verdict |
|---|---|---|---|
| **0** — Walking skeleton | done | — | **done** |
| **1** — Terrain MVP | done | — | **done** |
| **2** — Civilisation layer | done, "65 of 74, 1 blocked" | `cartalith_civ::jp_reroute_for_mode` exists (`lib.rs:11897`); `JOURNEY_PLANNER_SCOPE.md` §1069 records 66 of 74 | **done** — the count was stale, **F3** |
| **3** — Rendering and 3D | partial; "the 3D drape is not started" | No 3D anywhere: `PR-06` (AA/anisotropy) and `PR-08` (3D viewport defaults) are both open and both gated on it; `DECISIONS.md` §4 defers it | **accurate.** The 2D half is far past "milestones 1-6" now (NPR block, colour grade, bake/atlas, deep zoom); the 3D half is **deliberately deferred**, not merely unstarted |
| **4** — Asset Library | done | — | **done** |
| **5** — Urban morphology | "1-7 done; 8-17 open" | 1-7, 17a, **8a**, **12** done; 15 of 20 `_um*` ported | **started-and-incomplete**, and further along than documented — **F2** |

**Nothing in `ROADMAP.md` is genuinely unstarted except the 3D drape**, and
that is an owner decision with its own `DECISIONS.md` section rather than a
gap. The "Not a phase: LOD and large worlds" section has been overtaken —
deep-zoom tiling, the quadtree base, the persistent atlas and a measured
performance comparison all exist; the section still reads *"Revisit when a
concrete need appears rather than building it speculatively."* That is a
**judgment call about whether the section should be retired**, and it is
reported rather than edited.

---

## 18 · The register's open and narrowed entries — spot-checked, not taken on trust

**§37-§44's own re-checks hold.** Every closure claim spot-checked here was
real: `civ_place_defensibility` and `um_wall_spec`/`um_infer_walls` (§40),
`trade.rs`'s union-find `RoadComponents` and bipartite match (§42),
`territory_influence` (§41), `build_npp` and `wildlife` (§39), `civ_continents`
(§35). The batch's headline claim — that four of §37's fifteen were already
built — is accurate, and so is the sharper one, that §37's *stated reason* was
factually wrong for CV-21, WW-14, WW-15 and CA-16.

**The six narrowed remainders' blockers were checked and are real and current:**

| ID | Chip | Blocker verified |
|---|---|---|
| **CV-23** | blocked on | Historical occupation is timeline work; `territory_influence` computes the present-year field on demand and retains nothing. Real |
| **CV-24** | needs a decision | `dcc_shell.gd`'s `timeline_bar` is a single `Control` from `_build_timeline()` with a fixed-height contribution (`:2407`). A shell-frame change, correctly classified (C) |
| **CV-25** | needs a decision | Per-settlement garrisons, campaigns, unit movement, combat. **In flux — `MILITARY_MANPOWER_SCOPE.md` is a concurrent agent's territory and was not audited here** |
| **CV-26** | needs a decision | Treaties, vassalage, change over time. `relations` exists as one derived value per faction pair; there is no year axis on it. Real |
| **CA-18** | blocked on | `grep` for label collision / declutter budget across the whole shell and `cartalith-godot`: **zero hits.** Real |
| **CA-19** | costs a re-baseline | `CART_BIOME_COLS` is a compile-time `const` in `render.rs`, and `paint_bridge.rs:851` asserts exact RGB values against it. Making it writable really would move golden expectations `DECISIONS.md` §7a protects. **The most precisely-stated blocker in the register** |

**Other still-open §6 rows verified as genuinely absent** — no engine half,
reason accurate: **CV-01** the POI tool (`civ_tools_bridge.rs:26` states it in
the module doc: *"Territory only — no `civ_drop_poi`, no fabricated POI record
type"*) · **GeoJSON import** (`DM-03`; zero hits for any import path, and it
was explicitly out of scope when export was built) · **JP-06/JP-08** journey
persistence (`cartalith-io/src/save.rs` names them in its header and writes no
journey entry) · **PR-03** CPU worker threads (no `ThreadPoolBuilder` call
anywhere in the workspace) · **FI-06** project name field (no
`project_name`/`world_name` on `WorldGen` or `SaveData`) · **RD-05/IN-01**
rivers-as-entities — `trace_river_polylines` and `split_river_polylines` are
real and golden-verified, but a polyline has no id, name or catchment, so the
row's framing is **correct**. That last one is worth naming explicitly: it is
the one place in this pass where a "but the polylines exist" argument looked
like it would falsify a reason and did not.

---

## 19 · Cross-document contradictions

**Resolved by this pass, mechanically** — F2, F3, F4. See §21.

**Reported, not resolved:**

1. **The register's §3 counts (F6).** Pass 1 asked, pass 2 recounted to 215
   and disclosed a reproducible method, and the method now yields **260**
   (249 table-row IDs + 11 `### PH-` headings). §3's own prefix list is
   missing `MR`, `TO`, `MN`, `VA` and `KV`, all in use, and its "six `PH-0N`
   entries" is eleven. **The arithmetic half is mechanical and the A/B/C/D
   split is not**, which is why pass 2 declined; this pass also declines, for
   a second reason — `GUI_GAP_REGISTER.md` is a concurrent agent's territory
   this session, and this repository's own history of git-index sweeps
   (pass 2 §10, six incidents in one batch) makes an unnecessary edit to a
   shared 6 771-line file a poor trade for one corrected number.

2. **`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 (F9).** Q1 and Q2 read
   as open authorisation requests; both are built (`DECISIONS.md` §7f;
   `cartalith_hydrology::flow_sort_desc`, tested element-identical to the
   comparison sort it replaced). Q3's resolution text now carries **three**
   wrong sentences where pass 2 found one:
   - *"the future wiring is tracked as `GUI_GAP_REGISTER.md` MS-06"* — MS-06
     is *Auto-populate world*; the rows are SG-01…SG-03 (pass 2's F4, still
     unactioned);
   - *"**No UI was built**; the hold stands"* — the hold was lifted the same
     day it was called, and SG-01/SG-03's UI was subsequently built;
   - *"`param_set` is deliberately still unwired"* — **SG-03 closed
     2026-08-24**; `set_params` marks the graph.

   Pass 2's standing reason for not editing this document holds and is
   repeated here: *a third party restating an owner decision is how a
   decision loses its provenance.* It is the research author's to write.

3. **`ROADMAP.md`'s "Not a phase: LOD and large worlds"** now describes as
   speculative a subsystem that shipped (§17). Retiring or rewriting the
   section is a judgment call about the document's shape, not a staleness fix.

4. **`STATUS.md`'s purpose (F7).** Not a contradiction between documents but
   between a document and itself: `CLAUDE.md` says *"Authoritative status is
   `cartalith-native/docs/STATUS.md`. Read it before starting work"*, and the
   file is 689 KB. A session that follows that instruction literally spends
   its budget on exactly the thing the quicklist was created to prevent.

---

## 20 · What is actually left

The owner-actionable list, sorted by kind. Everything here was verified
against the crates during this pass.

### Needs an owner decision

| Item | The decision |
|---|---|
| **F1 — the flaky GPU determinism test** | `generate_terrain_gpu_path_is_deterministic_and_valid` asserts `assert_eq!` on the whole `f32` field across two GPU runs. It fails ~1 run in 3 under full-workspace parallel load, by ~1 ulp. The owner has already ruled that **GPU paths are held to principled equivalence, not bit-identity** (`DECISIONS.md` §7a's GPU door). Is bit-identity still the right bar *for determinism between two runs of the same path*, or should the assertion become a tolerance? Answering it either fixes the test or promotes the flake to a real bug |
| **CV-25** — per-settlement garrisons, campaigns, unit movement, combat | *In flux; a concurrent agent holds it* |
| **CV-26** — treaties, vassalage, diplomatic change over time | What a treaty *is* here, and whether relations get a year axis |
| **WW-15** — reprojection | The CRS is declared; nothing reprojects. Needs a target projection before it is scopeable |
| **CA-19** — a *writable* biome colour table | Buildable today; **costs a golden re-baseline** `DECISIONS.md` §7a protects |
| **IN-13's remainder** — prices, tariffs, caravans as entities, trade over time | Each needs a decision about what a currency is in this world. Already stated on screen |
| **F10** — layer hotkeys `1–8` vs the reference's `0 B T F S W R` | A §7d-shaped decision, unmade for three passes |
| **CA-12** — the Icon tool is inert until an asset pack is imported, and none ships | Registered as "(B) small, but an owner call — not taken here" |

### Already built, just unwired — the cheapest work available

| Item | What exists | What is missing |
|---|---|---|
| **RD-01** Settlement ▸ Defensibility | `civ_place_defensibility`, golden-verified; exposed via `civ_military_summary()`; already sorted on in `civilization_workspace.gd` | One dock field pointed at the right binding |
| **RD-02** Settlement ▸ Routes | `WayRouter`'s per-source Dijkstra + `prev_way` cache; `TradeFlow.from`/`.to` | Retain or re-walk the cached path instead of aggregating it into `way_load` |
| **RD-07** Faction ▸ State religion | `civ_faction_aggregates` live in two `#[func]`s; `faction_has_religion` already fed from the roster; `religious` a real power axis | One surfaced field |
| **CV-02** Culture ▸ Profiles | `civ_default_culture` already called inside `get_factions()` | One `get_cultures()` binding |
| **F8** the harnesses | 72 `.gd` + 71 `.tscn` drivers exist and work | `git add` — they are cited as evidence and a fresh clone has none |

### Real work, small

`FI-06` project name field · `CV-05` territory "respect coastlines" ·
`JP-10`/`JP-11` foraging offset and load speed penalty · `IN-03` per-waypoint
way/route undo · `WW-06` paint hardness/softness · `WW-08` min stream order ·
`CA-06` label letter-spacing and anchor · `PR-03` CPU worker threads (one
`ThreadPoolBuilder` call; the *default* is owner policy) · `CV-01` the POI
tool (one `civ_drop_poi` mirroring `civ_drop_settlement`; the `poi` asset
family already carries the 10-slot vocabulary) · `SH-06`'s `→` draft-stamp
suffix.

### Real work, large

| Item | Honest current scope |
|---|---|
| **Urban morphology, milestones 8, 9, 10, 11, 13, 14, 15, 16** | **~42 of block 4's 92 functions**, plus 5 of the 20 `_um*` adapter functions (`_umHarbourScale`, `_umPt`, `_umSiteProfile`, `_umOreBearing`, `_umCacheKey`). Milestone 10 (fortification, 9 fns / ~407 lines) is the largest single one; milestone 16 (`generate` + `hashModel`) is the payoff, because it unlocks a **whole-subsystem golden** the reference wrote for exactly that purpose. **Scope estimate, revised honestly:** the original "~3,860 lines" counts *JS*. Milestones 1-7 + 8a + 12 produced **18 568 lines of Rust with tests** for roughly half the function count, so the remainder is plausibly of the same order again — not a phase-sized push, and the scope document has said so since 2026-08-18 |
| **The 3D drape** | Genuinely not started, and **deliberately deferred** (`DECISIONS.md` §4). `PR-06` and `PR-08` are both gated behind it. Phase 3's other half has run far ahead of this entry |
| **ED-03/ED-04** cut/copy/paste/select-all | No clipboard model for any entity. Three separate pieces, not one selection abstraction |
| **RD-05 / IN-01** rivers as entities | Rivers are a per-cell network; polylines exist but carry no id, name or catchment. Reason verified accurate |
| **CA-03 / CA-04 / RD-10** layer compositing | Needs the single colour pass to become separable outputs — an architecture change `GUI_FEATURE_PARITY_SCOPE.md` Category 3 already recommended |
| **GeoJSON import** (`DM-03`) | Absent, and explicitly out of scope when export was built |
| **CV-24** the year scrubber as program scope | A shell-frame change; `TIMELINE_SCOPE.md` §4's standing instruction is to design the panel first rather than guess the region |

### Deliberately deferred

3D drape (`DECISIONS.md` §4) · store distribution (§6) · a WASM target (§2) ·
an Obsidian plugin (owner: *"a wish, deferred outright"*) · POIs and "regions"
as vault entity kinds (owner-excluded; recorded as unsatisfiable rather than
faked) · `compute_civilisation`'s crate move (owner 2026-08-24: *"not a
priority right now"*) · `WW-10`, `WW-11`, `CV-06`, `CV-09`, `IN-04`, `RD-04`,
`RD-12`, `AS-14`, `AS-15`, `CA-10`, `SH-02`, `SH-04`, `SH-08` — all class (D),
engine truth or owner decision, not gaps · `DM-07`/`DM-08`/`DM-09`, resolved
by deletion (owner, 2026-08-20).

---

## 21 · Edits made by this pass

**Seven, all purely mechanical, all disclosed.** Each aligns a stale statement
with a fact already recorded elsewhere in the repository or provable from the
code; none resolves a judgment call.

1. **`ROADMAP.md` Phase 2** — *"65 of the reference's 74 `jp*` functions …
   1 blocked"* → **66, nothing blocked**, per `JOURNEY_PLANNER_SCOPE.md`
   §"the count reaches 66 of 74" and `cartalith_civ::jp_reroute_for_mode`
   existing at `lib.rs:11897`. This is pass 2's F10, fixed in `README.md`
   then and missed here — the same one-of-two-copies failure as pass 2's F1.
2. **`ROADMAP.md` Phase 5** — *"8-17 open"* → milestones **8a and 12 done
   2026-08-24**, the open list enumerated, and the authority repointed from
   `STATUS.md` to `URBAN_MORPHOLOGY_SCOPE.md`'s own milestone headings, which
   is where the fact actually lives.
3. **`README.md` Phase 5 row** — same correction.
4. **`FUNCTIONAL_CONTRACT.md` urban capability row** — same correction.
5. **`FUNCTIONAL_CONTRACT.md` absent-entirely list, urban bullet** — rewritten
   to the real remainder (~42 block-4 functions + 5 `_um*`), and
   **blocks/parcels removed from the absent list**, since milestone 12 built
   it the day before that bullet was written.
6. **`FUNCTIONAL_CONTRACT.md`, three more entries** — SG-03 struck (closed
   2026-08-24; `set_params`' own doc comment names it); the persistent
   atlas/bake bullet struck (`cartalith_engine::bake::AtlasStore` writes to
   disk); ambient occlusion removed from the AO/SVF/shadow/SDF bullet (it
   ships, with two live sliders). The coverage table's matching rows for live
   tuning and for the tile pyramid were corrected with them.
7. **This file's header** — pass 3 added to the nav block, and pass 2's range
   corrected from §9-§13 to §9-§14.

**Deliberately not edited, reported instead:**

- **`GUI_GAP_REGISTER.md` §3's counts (F6)** and **RD-01/RD-02/RD-07's stale
  reasons (F5)**. Two reasons, both standing: the A/B/C/D split is a
  classification pass rather than an arithmetic one (pass 2's own reasoning),
  **and the register is a concurrent agent's territory this session**. Every
  fact needed to make all four edits is in §16 and §19 above.
- **`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 (F9).** Pass 2's reason
  holds verbatim: the owner decisions are the research author's to record.
- **`STATUS.md` (F7).** Its size is a structural problem, and shrinking it is
  an editorial decision about what a quicklist is for — not a staleness fix.
- **`ROADMAP.md`'s "Not a phase: LOD and large worlds"** — retiring a section
  is a judgment call.
- **F1**, the flaky test. Changing an assertion's bar is exactly the kind of
  deviation `CLAUDE.md` says to raise rather than make quietly.
- **Anything under `MILITARY_MANPOWER_SCOPE.md`, `cartalith-civ/src/
  manpower.rs` or the CIVIL ▸ Military surfaces** — a concurrent agent held
  them for the whole of this pass. Register §43/§44 were read for context and
  are reported as *in flux*, not audited.

---

## 22 · What a fourth pass should do differently

1. **Start from `git log`, then the register's newest sections, then the
   summary tables — in that order.** This pass did, and it is why §16 exists:
   the register had already self-corrected, and the drift had all migrated to
   the documents no feature commit is obliged to touch. Pass 2's §14.1
   recommendation is now three-for-three and should be a standing rule rather
   than a recommendation.
2. **Give `FUNCTIONAL_CONTRACT.md` an owning register ID per row, or demote
   it.** Pass 2 proposed this and it was not done; the document has now gone
   stale three passes running, and once *on the day it was corrected*. It is
   the only document in this repository with that record.
3. **Sweep §6 of the register the way §39 swept §37.** F5 found three rows
   whose reason the engine has falsified, all of them in the register's oldest
   section, which §37-§44's re-checks never reached. §6 has ~150 rows and has
   not been re-checked against the crates since it was written. **That sweep
   is the single highest-value thing a fourth pass can do**, and §39 has
   already proved the method works.
4. **Decide F1 before it trains people to ignore a red test.** An
   intermittently-failing test, in a repository whose own discipline note
   reads *"a stale binary reports a healthy N passed"*, is corrosive out of
   all proportion to its size.
5. **Commit the harnesses (F8), or write down that they are deliberately
   ephemeral.** Asked by pass 1 §4, asked again by pass 2 rec #4, and the
   count has gone 11 → 152. One of the two answers is right; neither of them
   is "keep citing files a clone does not have."
6. **Walk Part 0 control-by-control.** Pass 1 recommended it, pass 2 did not
   do it, pass 3 did not do it. Three passes old. It is the one input that
   would find gaps the register has never registered at all — which is the
   only category none of these three passes can rule out.

## 23 · Pass 5 — the wiring audit, run mechanically instead of by eye

Owner, 2026-08-26: *"Do a full audit and check if everything is wired
correctly, like you said this is the 4th time already."*

Fair. Every previous find of this class — something ported, golden-tested,
recorded as done, and called by nothing — was found **by accident, mid-task,
by eye**. Four passes of this document did the same by reading. So this pass
did not read anything first: it asked the question mechanically, then verified
each hit by hand.

**The harness is checked in this time**, at
`cartalith-native/tools/audit_wiring.py` — §22 rec #5 asked for exactly that,
after passes 1 and 2 both asked and were ignored. `python
cartalith-native/tools/audit_wiring.py`. It answers four questions:

| | question | result |
|---|---|---|
| A | `pub fn` with no caller outside test code | **65 of 1 167** |
| B | `#[func]` no `.gd` file names | **17 of 361** |
| C | names the shell asks `world_gen` for that do not exist | **0** |
| D | `engine_bridge.gd` funcs nothing calls | **10 of 337** |

It flags; it does not judge. A hit is one of four things and only reading
tells you which: a superseded wrapper kept on purpose, a disclosed deliberate
non-use, ordinary accessor surface, or a real gap. Everything below was read.

### C is empty, and that is the one number worth celebrating

Every method the shell asks `world_gen` for exists. That class — a stale
`_has("...")` guard silently disabling a feature forever, which is exactly how
`save_project`-versus-`project_save` disabled File ▸ Save until 2026-08-25 —
is now empty. It is also the only one of the four that *can* be empty, and the
only one worth turning into a red test later.

### F9 · Owner-requested, engine-complete, UI-absent — the vault

Four `#[func]`s, zero references in any `.gd` file. This is the 2026-08-25
direction, verbatim: *"Search for it in the vault etc. ... When the user ever
wants to change this or add information to the markdown file it's an explicit
action with a prompt confirmation (the prompt should have **an option to
confirm always**)."*

| built | reached by the shell |
|---|---|
| `vault_search` (`vault_bridge.rs:177`) — indexed + scanned search with excerpts | **no.** There is no search field in `vault_window.gd` at all |
| `vault_write_prefs` / `vault_set_write_pref` / `vault_prefs_json` / `vault_restore_prefs` (284/297/306/314) — the "confirm always" preference, with its own doc comment headed *"the owner's 2026-08-25 direction"* | **no.** `_preview_dialog` (`vault_window.gd:503`) builds the confirmation with a title, a note and a read-only preview — and no checkbox. Nothing reads or writes a preference |
| `vault_file_data` / `vault_entity_data` / `vault_link_data` (215/243/263) — a note's frontmatter and fields, *"what a search result or an attach dialog shows before the user commits"* | **no** |
| `vault_remove_block` (588) | **no.** A written block cannot be taken back from the UI |

The engine half is complete, documented and tested. The confirmation dialog
the owner asked for exists; the *"confirm always"* half of the same sentence
does not, and neither does the search the same message opens with.

### F10 · `project_save_with_documents` — unwired **and** asymmetric

`entities/journeys.json` is a registered, non-engine-owned document slot.
`project_save_with_documents` (`project_bridge.rs:1017`) is the documented
channel for *"project state GDScript owns and the engine does not model"*.
Nothing calls it.

But wiring it alone would be worse than leaving it: **there is no reader.**
`project_bridge.rs` exposes six `#[func]`s and none of them hands a
caller-owned document back after `project_open`. Writing journeys today
produces a file the shell can never load.

> **Correction, 2026-08-26 — the paragraph above is wrong, and it is this
> pass's own error, not an inherited one.** `project_open` *already* built a
> `documents` dictionary of every non-engine-owned slot the archive carried
> (`project_bridge.rs:1471-1478` as of `cd02490`). The audit read the `#[func]`
> list, saw no reader-shaped name in it, and concluded there was no reader —
> the exact failure mode the harness exists to prevent, committed by hand in
> the prose beside it.
>
> The real defect was narrower and differently shaped: that dictionary held
> `serde_json::to_string` of the **coerced `serde_json::Value`**, which is not
> the document that was saved. `Value`'s object map is a `BTreeMap`, so members
> come back sorted; whitespace is gone; and §14.2's integer coercion has
> already rewritten `1.0` to `1` inside it. So the honest statement is *"the
> shell could load them but not get them back unchanged"* — still worth fixing,
> and fixed in `afc2d57`, but not the blocker this section claimed.
>
> What survives unchanged: the writer had no caller, `entities/journeys.json`
> was and is unwritten, and the `JOURNEY_PLANNER_SCOPE.md` paragraph below is
> still stale. Only the severity and the cause were wrong.

This also makes `JOURNEY_PLANNER_SCOPE.md` stale in a specific way. It says
the saved-journeys list is in-session only because it *"needed FI-01's `.zip`
save-**writer**"*, and then that *"what is still missing is narrower: a way for
GDScript-owned state to reach the save's `state` object"*. The writer exists
and the channel exists. What is missing is the **read** side, which that
paragraph does not mention.

### F11 · The reference's `erode()` op is ported at kernel level and never assembled

Reference line **3898** — `dropletParams` at 3889, `erodeFinish` at 3892:
`erode()` is `dropletKernel(...)` → `erodeFinish` → `erodeThermal(p.thermalPasses)`
→ clamp → `isostaticRebound`.

> **Two corrections, 2026-08-26, both this pass's own.** This section first said
> "line 3894", which is the clamp *inside* `erodeFinish`, not the op — checked
> against the reference and fixed above. This repository's own working rule is
> *"verify a scope document's line ranges against the real reference before
> slicing"*, learned from four consecutive urban milestones; the audit that
> re-states that rule broke it in the same document.
>
> The second correction matters more, because it changes the fix. The paragraph
> below says the two kernels *"appear nowhere outside their own crate"*, which
> reads as an oversight — someone forgot a call. They are **unreachable**:
> `cartalith-godot` does not depend on `cartalith-erosion` at all (only
> `cartalith-erosion` itself and `cartalith-engine` name it in a `Cargo.toml`),
> and `cartalith-engine`'s `use` at `lib.rs:125` re-exports none of it, so the
> kernels are *unnameable* from the bridge crate. That is a dependency gap, not
> a missing call, and it is why this was the largest item: the fix is a new
> `pub fn` in `cartalith-engine` (which already has the dependency), not a line
> in a bridge.

`cartalith-erosion` has all three, with golden-parity coverage
(`golden_parity_droplet`, `_thermal`, `_rebound` — `CPU_MULTITHREADING_SCOPE.md`
line 399 names them). `cartalith-engine` imports **`isostatic_rebound` only**
(`lib.rs:125`); `droplet_kernel` and `erode_thermal` appear nowhere outside
their own crate and its tests, and no `droplets` / `thermal_passes` parameter
exists on `WorldParams`.

No document discloses this as deferred. `ARCHITECTURE.md:39` lists the crate as
*"droplet, stream-power, thermal"* without qualification. This is the largest
single item the audit found: a named generation op the HTML app has, ported,
tested, and unreachable.

### F12 · The Journey Planner's wildlife modifier is switched off on worlds that have wildlife

`jp_compute` passes `&|_, _| 1.0` as the forage modifier, over a comment
reading *"the reference's own answer on a world whose wildlife layer was never
built"* (`lib.rs:8790`).

That comment is stale. `cartalith-civ/src/wildlife.rs` exists, is
golden-tested (`golden_parity_wildlife.rs`), carries `Ecoregion::richness` and
`Ecoregions::region_id`, and `wildlife_region_at` is a live `#[func]` the shell
already calls. `jp_world_mean_richness` and `jp_wildlife_forage_mod` — the two
functions that turn exactly those two fields into the modifier — are ported,
tested, and called by nothing.

Not a one-line fix, and the audit should say so: `sample_bridge::
wildlife_regions` rebuilds the cart-biome, NPP, TRI and lithology fields, and
`jp_compute` runs on every party-form keystroke. It needs a cache before it
needs a call site.

### F13 · Reference features ported, exposed nowhere

Each verified individually: zero references in `cartalith-godot` and zero in
`godot-project/**.gd`.

| function | what it is |
|---|---|
| `extract_region_as_world` (`region_export.rs:277`) | a selected region promoted to its own world |
| `apply_force_lake` (`civ/lib.rs:715`) | the force-lake post-pass |
| `estimate_regional_density_km2` (`civ/lib.rs:1020`) | real regional population density, additive to carrying capacity |
| `icon_brush_rule` / `icon_brush_stamp` (`assets/manual.rs:189/211`) | the icon brush |
| `jp_pack_range` (`civ/lib.rs:13657`) | **the v1.48 fix itself** — the fodder ceiling stated *before* the user configures past it. Without it the port reproduces the pre-v1.48 behaviour the reference wrote this function to end |
| `jp_risk` (`civ/lib.rs:13725`) | the campaign-duration advisory |
| `jp_vessel_matrix` (`civ/lib.rs:8470`) | every vessel × every water type — *"what is actually fast HERE"* |
| `arc_label_line_width` (`civ/labels.rs:194`) | arc-label line width |
| `to_library_json` / `drop_collection` / `apply_library_file_with_items` / `referenced_files` / `slot_paths` / `filled_count` (`cartalith-assets`) | asset-library operations |
| `build_tide_field` / `refresh_geoid` | **probably benign** — `compute_tide_field` and `current_geoid_preview` are the wired siblings; listed for completeness, not as findings |

### F14 · Low severity, but real

* **Three getters nothing reads.** `get_villages_enabled`,
  `get_metropolis_enabled`, `get_recovery_phase` are registered; the setters
  are wired and the shell keeps its own copy of each toggle. Nothing
  re-reads engine state, so shell and engine can disagree after a load — the
  same shape as the bug `route_get`'s `mode` had until 2026-08-26.
* **`lod_max_level` is never read.** The shell calls `lod_level_for_zoom` and
  `lod_tiles_per_axis` but nothing clamps a requested level against the
  engine's own ceiling.
* **Ten `engine_bridge.gd` wrappers nothing calls**, including
  `missing_bindings` (the diagnostic that would have surfaced F9 and F10 in
  the UI), `param_groups` / `param_default`, `vault_read_file`,
  `label_glyph_layout`, `civ_has_faction_colors`,
  `get_biome_k_enabled` / `set_biome_k_enabled`, `heightmap_grid_size`,
  `sculpt_stroke_point_count`.

### Classified as **not** gaps

Read and dismissed, so a sixth pass does not re-litigate them:

* **Superseded wrappers, kept deliberately:** `place_settlements` (→
  `_with_water_edge_snap`, and its own comment says it is *"kept as an exact,
  unchanged"* original), `name_and_populate_settlements` (→ `_with_rng`),
  `jp_plan` (→ `jp_plan_full`), `jp_capacity` (→ `jp_capacity_ex`),
  `jp_calc_water` (→ `jp_calc_water_ex`), `build_road_network` (→
  `civ_hierarchical_network_topology`).
* **Disclosed deliberate non-use:** `civ_zoom_pick_r` — `lib.rs:5903` states
  in full why the bridge uses `civ_place_pick_radius` without zoom scaling.
* **Accessor surface on data structures:** all 18 in `cartalith-spatial`
  (`row_mut`, `tile_mut`, `is_dirty`, `query_region`, …), both in
  `cartalith-io`, `clip_convex`, `with_sea_level`, `unpack_rgb8`.
* **GPU variants behind a feature/probe path:** all 9 in `cartalith-gpu`.

### What this pass changed

Nothing in the crates. It added the harness and this section. Every finding
above is a decision for the owner about what to build next, and F10 in
particular is a decision about **what to build first** — the document reader,
before anything writes documents.

### What a sixth pass should do differently

1. **Re-run the harness before reading anything.** It took under a second and
   found in one pass what four reading passes found one at a time.
2. **Make question C a red test.** It is the only one of the four that is
   legitimately zero, and it is the class that has actually shipped broken.
3. **Do not turn A or B into tests without an allowlist**, and do not write
   the allowlist — 60-odd entries is a second place for the truth to rot.
   A hit list a human triages is the right shape for these two.
4. **F11 first among the build items.** It is a whole named op from the
   reference, not a readout.

### F15 · File ▸ Save wrote the tree; File ▸ Open read it as flat, and dropped everything else

**Found 2026-08-26, while wiring F10, and it is the most serious defect this
audit produced.** Measured, not inferred (`_openpath_probe.gd`):

```
  settlements before save: 8
  project_save: ok=true entries=18
  load_save (what the shell calls): ok=true settlements=0
  project_open (never called by the shell): ok=true restored=["civ","appearance"] settlements=8
```

`EngineBridge.save_project` was repointed at `project_save` — the tree writer —
on 2026-08-25. `EngineBridge.load_save` was never repointed: it still called
`WorldGen::load_save`, which is `cartalith_io::load_save`, the **flat HTML-format
reader**. So every project saved by this build and reopened by this build came
back with its terrain intact and its **entire civilisation layer, appearance and
every document silently gone** — `ok == true`, no warning, nothing in the
console. The user would discover it as "my settlements vanished", one session
later, with no way to tell which of the two operations lost them.

`project_open` reads *both* layouts — verified in the same probe: it opens a
flat export and reports `layout == "flat"` — so the fix is one call, not a
format sniff, and it satisfies the owner's own rule (read the old format and
the new one, write only the new one). `_savereopen_probe.gd` now drives the
round trip through the real `EngineBridge`: 8 settlements out, 8 back, a
caller-owned document byte-identical including an integer above 2^31, and the
old flat archive still opening.

### F16 · The harness that found §23 had this exact blind spot

**Question B counted every `.gd` file, including probes.** `project_open` was
named by `_savetree_probe.gd` and by nothing else, so it scored as *reached*
while F15 sat underneath it. A capability reached only by a probe is not wired:
it is a capability with a test and no product behind it, which is the precise
thing this harness exists to find.

Fixed: question B now scans **shell files only**, and flags a hit that a probe
names with `<-- named ONLY by a probe`, because that is a stronger signal than
silence — it means someone built the thing, proved it works, and never
connected it.

The corrected question B immediately returned two more:

* `recompute_stale_stages` — the shell **reads** staleness (`stale_stages()`,
  on a one-second timer, displayed in the status rail) and never calls the
  function that settles it. Every other recompute path reaches
  `mark_and_recompute` as a side effect of some other op, so the indicator does
  clear eventually; what is missing is the explicit "recompute now" the
  reference offers. Open.
* `atlas_is_covered` — open, unexamined.

**This is the fifth distinct instance of the same defect class in one day**, and
the first four were found by eye. The lesson §23 already drew stands, with one
correction: the harness was right to exist and wrong in its first form, and it
took a real data-loss bug to expose the flaw. Re-run it after wiring work, not
only before.
