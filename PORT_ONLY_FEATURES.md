# Features and refinements the port has that the legacy HTML does not

Compiled 2026-09-13 against the frozen legacy app, `reference/Cartalith Gen1 v2.11.html`.

**What counts.** Only two kinds of item are listed:

- **New**: nothing like it exists in the legacy file.
- **Improved**: the legacy file has a lesser version, and the row says what changed.

A straight port of a legacy function is not listed, even if its code was rewritten.

**How each row was checked.** Three read-only research passes (engine, GUI shell, rendering/urban/assets) opened the implementing symbol in this repository. Each confirmed it is live code, not a comment or a stub. Each then searched the v2.11 file for the concept under its likely names. A scope document's claim was used only as a lead. Where a claim turned out wrong, the row says so.

**This is a list, not a status file.** Where something stands today is `cartalith-native/docs/STATUS.md`'s answer. The status column below records what the check saw on 2026-09-13, so re-verify it before relying on it.

---

## 1. World generation and simulation (Rust engine)

| Feature | Kind | What it does | Where in the code | Status 2026-09-13 |
|---|---|---|---|---|
| Military manpower model | New | Standing army, field army, emergency mobilisation and a war-duration ladder, derived from five drivers: agricultural surplus, fiscal extraction, professionalisation, logistics and carrying capacity. The legacy file has no army-size model at all | `cartalith-civ/src/manpower.rs::civ_military_manpower`; bridged in `civ_military_bridge.rs` | Shipped |
| Causal landmark generation | New | 49 landmark kinds sited from terrain, hydrology and the route network (bridges, market sites, caravan stations, trade depots, road junctions…). A funnel reports candidates per kind and which cap bound. The legacy "landmark" is only a hand-placed POI icon | `cartalith-civ/src/landmark.rs::generate`, `LandmarkFunnel`; `landmark_run` in `cartalith-godot/src/lib.rs` | Shipped |
| Continents as entities | New | Landmasses are ranked by area, named, and given an id and boundary so notes can link to them. The legacy file computes only a per-cell continentality field | `cartalith-civ/src/lib.rs::civ_continents`; `EntityKind::Continent` in `cartalith-vault` | Shipped |
| Multi-good trade matching | Improved | The legacy supply match and road gate handle food only. Here they cover all fifteen resource goods, driven by the good-reach figure the legacy file computed only for display | `cartalith-civ/src/trade.rs` | Shipped |
| Physical crater model | New | Optional crater count and size by area density, plus wear over a stated surface age in millions of years | `params.rs` rows `crater.physical_model`, `crater.surface_age_myr`; `cartalith_terrain::crater_degradation_tau` | Shipped, off by default |
| Erosion passes as generation parameters | Improved | Velocity erosion, glacial carving, coastal processes and hillslope diffusion are one-off manual buttons in the legacy file. Here they are saved, toggleable parameters that run inside generation | `cartalith-godot/src/params.rs` (`passes.*`) | Shipped; the passes default to off (`WorldParams::defaults`) |
| GPU compute across generation | New | wgpu compute for noise, domain warp, crustal heterogeneity, the height formula, plate assignment, weather and flow accumulation, with CPU fallback. The legacy WebGL2 path covers only eight post-process shaders (erosion, diffusion, blur, temperature, coastal) | `cartalith-gpu/src/lib.rs`; dispatched from `cartalith-engine::generate_terrain` | Shipped |
| Pipeline-wide multithreading | Improved | Per-cell loops across terrain, climate, erosion, hydrology and civilisation run in parallel with Rayon. The legacy file uses Web Workers for two erosion kernels only | `par_iter` throughout `cartalith-climate`, `-civ`, `-erosion`, `-engine`, `-terrain`, `-hydrology` | Shipped |
| Lower generation memory peak | Improved | The previous world is freed before the next is generated, dead grids are removed, and resource potentials are stored in blocks. Measured about 519 MiB lower at peak | `cartalith-godot/src/lib.rs::release_world`; `RESOURCE_BLOCK` in `cartalith-civ` | Partial: R1–R3 done, R4–R8 not started |
| Non-destructive tiled sculpt storage | Improved | Sculpt commit, discard, undo and redo work over a tiled buffer with dirty-region tracking, instead of whole-canvas snapshots | `cartalith-spatial/src/pass.rs::PassBuffer`, `DirtyTracker`; `sculpt_bridge.rs` | Shipped |

## 2. Data, files and interchange

| Feature | Kind | What it does | Where in the code | Status 2026-09-13 |
|---|---|---|---|---|
| Markdown Vault, read and write | Improved | Linked Markdown notes for continents, provinces and settlements, edited in the app. The engine handles section editing with byte-identical round-trips, a machine block, backlinks, templates, and atomic writes with conflict detection. The legacy file only displays links from a `vault.json` it cannot write | `cartalith-vault/src/*`; 52 entry points in `vault_bridge.rs`; `shell/vault_window.gd` | Shipped |
| GeoJSON import and inspection | New | Reads a GeoJSON FeatureCollection (this app's export or a foreign one) and reports features, layers, geometry types, bounds and provenance, refusing bad features with a reason. The legacy file only exports | `cartalith-io/src/geojson_import.rs::parse_geojson`; `geojson_bridge.rs::geojson_inspect` | Engine and bridge shipped; not checked as a GUI action |
| New project-archive slots | New | The save format gains `entities/journeys.json`, `entities/landmarks.json`, `annotations/measurements.json`, `library/assets.json`, `library/travel.json`, `drafts/paint.json` and `drafts/sculpt.json` | `cartalith-io/src/project.rs::DOCUMENT_SLOTS` | Each slot is as real as its feature |
| Autosave | New | Writes a backup beside the open project every N minutes once the world has changed; on/off and interval are settings | `shell/dcc_settings.gd` (`autosave_*`); `app.gd::_setup_autosave`, `_autosave_tick` | Shipped, on by default |
| Recent projects | New | The last ten projects, remembered between sessions | `shell/dcc_settings.gd::recent_projects`, `remember_project` | Shipped |
| Local diagnostic report | New | Help ▸ Save diagnostic report shows every row first, then writes one local text file (generation info, missing bindings, format version, GPU state, last error). Nothing is sent anywhere | `shell/diagnostic_report.gd`; `diagnostic_review_dialog.gd` | Shipped |
| Data Manager window | Improved | Import, export, sources and validation in one navigable window with status and estimates, replacing five separate File-menu buttons | `shell/data_manager_window.gd` | Shipped |
| Persisted LOD tiles in the save | New | The archive can carry the deep-zoom tile pyramid with a staleness key, replacing the browser-only IndexedDB atlas | `cartalith-io/src/project.rs::LodTiles` | **Built, not wired**: nothing populates it yet |

## 3. Interface and platforms

| Feature | Kind | What it does | Where in the code | Status 2026-09-13 |
|---|---|---|---|---|
| Android app | New | An installable APK with haptic feedback, the system back gesture stepping back one level at a time, and layout inside the real safe area | `shell/dcc_shell.gd::_haptic`, the `NOTIFICATION_WM_GO_BACK_REQUEST` handler; `app.gd::set_safe_insets` | Shipped; runs on a OnePlus 6T |
| Desktop, tablet and phone layouts | Improved | A real three-form-factor system with touch targets of at least 44 px on touch devices. The legacy file has one desktop layout plus a hamburger, a zoom pad and a joystick for narrow screens | `shell/dcc_theme.gd` (`is_touch`, `is_tablet`, `is_phone`, touch tokens); `phone_menu.gd` | Shipped |
| Command search | New | One search box over every generation parameter, menu command and extra action; unavailable rows say why | `shell/command_index.gd` | Shipped |
| Rebindable keyboard shortcuts | New | Preferences ▸ Keyboard shortcuts reassigns accelerators, shows conflicts and restores defaults. Help ▸ Keyboard shortcuts lists them from the live menu bar | `shell/shortcuts_dialog.gd`; `menus.gd::_bind_accelerator` | Shipped |
| Find on map | New | One search over settlements, factions, labels, roads and sea routes that pans to the result. The legacy search covers the settlement table only | `shell/place_search.gd`; Edit ▸ Find on map… | Shipped |
| Measure tools | New | Distance, bearing, area, radius, cross-section and vertical difference, with an elevation-profile strip. Distances are wrap-aware and use the map's km-per-cell scale | `cartalith-spatial/src/measure.rs`; `measure_bridge.rs`; `shell/global_tools.gd::MEASURE_MODES`, `section_strip.gd` | Shipped |
| Undo history and redo | Improved | A browsable list of committed steps with a working redo. The legacy file keeps five undo steps with no redo | `app.gd::open_undo_history`, `right_dock.gd::show_history`; `EngineBridge.redo_available` | Shipped |
| Unified tool bar | Improved | Sculpt, paint and measure from one bar in any domain, calling the same engine functions as the domain panels. In the legacy file these live in separate tabs | `shell/tool_bar.gd` | Shipped; flatten and noise brushes and water or lithology paint are absent because the engine has no such modes |
| Memory and GPU readouts | New | Working set against physical memory, render-memory monitors, and GPU VRAM budget, visible in the menu and top bar | `menus.gd::_refresh_working_set_row`; `app.gd` `top_mem` | Shipped |
| Coach marks | New | Two one-time hints on first phone use (the bottom tabs and the tool-sheet handle), remembered once seen | `app.gd::_maybe_show_coach_marks` | Shipped, phone only |
| Setup is not a lock | Improved | The first-run dialog can be dismissed, leaving a usable empty app. The legacy setup wizard cannot be skipped | `app.gd::_open_welcome_when_drawn`; phone uses `phone_project_picker.gd` | Shipped |
| Failed-open recovery | Improved | A project that fails to open leaves a status hint and a toast, and keeps the open dialog (desktop) or picker (phone) up to retry. The legacy file shows one `alert()` | `app.gd::_load_project` (`last_open_refusal`) | Shipped on desktop; the phone half was not re-read at its code |

## 4. Map rendering and appearance

| Feature | Kind | What it does | Where in the code | Status 2026-09-13 |
|---|---|---|---|---|
| Flow-based wetness halo | New | Cools and darkens land near high-flow channels as a blurred halo, separate from the ported wetness multiply | `render.rs::build_hydro_wetness`, `hydro_wet_strength` | Shipped, reached through look presets |
| Local contrast | New | A band-pass pass that separates material edges; the legacy file has no neighbourhood contrast stage | `render.rs::apply_local_contrast` | Shipped, via look presets |
| Forest stippling from canopy | New | Stipple driven by the actual canopy fraction, distinct from the ported pen-stipple effect | `render.rs::land_color` (stipple block) | Shipped, via look presets |
| Plate border and neatline | New | A physical map border with ink colour; overlays clip to it | `render.rs::apply_border`; `map_overlay.gd` | Shipped |
| Editable elevation colour ramp | New | Elevation-keyed colour breakpoints with presets | `render.rs::ElevationRamp`, `RAMP_PRESETS`; `lib.rs::set_ramp_mode` | Shipped, off by default |
| Reorderable raster layer stack | New | Terrain, hillshade and colour-relief layers as data: order, blend, opacity and visibility | `render.rs::LayerStack`; `lib.rs::get_layer_stack`, `set_layer_stack` | Shipped |
| Quality tiers | New | Performance, Balanced, Quality and Ultra, with a recommended tier | `render.rs::QualityTier`, `recommended_quality_tier`; `lib.rs::set_quality_tier` | Shipped |
| Display P3 colour space | New | Wide-gamut output option | `render.rs::ColorSpace::DisplayP3`; `lib.rs::get_color_space` | Shipped; little uses it |
| Colour grade | New | Exposure, contrast, saturation, temperature, split toning and gamma, weighted by biome, elevation, moisture and geology | `render.rs::apply_color_grade`, `build_grade_influence` | **Engine only**: no bridge function exposes it |
| Two-scale ambient occlusion | Improved | Fine and broad occlusion, each normalised to the world's own land RMS. The legacy file has single-radius, unnormalised AO. **`render.rs`'s comment saying "the reference has no AO" is wrong**: the legacy `buildAOField` and its `aoR` slider exist | `render.rs::build_ao` | Shipped |
| Configurable multi-sun relief | Improved | The legacy on/off multi-sun lighting becomes a light count, directionality, ambient floor and gain | `render.rs` (`relief_lights`, `multi_sun_from_normal`) | Shipped |
| Rock exposure from real lithology | Improved | Rock colour from the seven lithology classes on by default, with bedrock showing through thin soil. The legacy equivalent is behind two rarely enabled flags | `render.rs::rock_material_col`, `litho_strength` | Shipped |
| Richer paper ground | Improved | Warped grain and mottle with a luminance-preserving wash. The legacy parchment effect needs an asset-pack texture | `render.rs::paper_tone`, `paper_wash` | Shipped |
| Route-corridor and travel-cost map views | Improved | Two fields the legacy file computes only as internal scoring inputs become visible analysis layers | `cartalith-godot/src/sample_bridge.rs` (`corridor`, `travel_cost`) | Shipped |
| Interactive deep-zoom tiles | Improved | Camera-driven tile synthesis with added fBm/ridged detail when zooming in. **It is not sharper in colour than the legacy viewer yet**: the legacy per-tile colour renderer `renderBiomeTileRGBA` is unported (milestones LOD-D1 and D2 in `LOD_DETAIL_SCOPE.md`) | `lod_bridge.rs::lod_synthesize_tile`; `cartalith_terrain::amplify::amplify_region` | Shipped, with that limit |

## 5. Urban layouts

| Feature | Kind | What it does | Where in the code | Status 2026-09-13 |
|---|---|---|---|---|
| Round towers on curtain walls | New | Towers drawn at every vertex of a plain wall circuit. The legacy file's only towers are harbour chain towers | `shell/urban_layout_draw.gd` (`WALL_TOWER`) | Shipped 2026-09-13 |
| Deep-zoom-safe ground fills | Improved | Block and parcel fills fall back to a triangle fan where Godot's triangulation fails at deep zoom (42 failures → 0 on the repro town). The legacy canvas never triangulated, so it never had the failure | `urban_layout_draw.gd::_fill_ground_polygon` | Shipped |

---

## Checked and not listed

These were candidates, and each turned out to exist in the legacy file already, or not to exist in this port:

- **The Journey Planner** is a port. The legacy file has about 70 `jp*` functions.
- **Heightmap import** ports `loadImage` and `inferTectonics`.
- **GeoJSON export** ports `exportGeoJSON`; only import is new.
- **The asset-pack slicer and pack format** are ports of the legacy `SpriteSheetImporter` and `pack.json`.
- **The fortification ladder** (`um_wall_spec`, `um_infer_walls`) is a port.
- **The deep-zoom pyramid and chunk atlas** port `pyramidTile` and `atlasPutMeta`.
- **Crest lines, curvature shading, sky-view factor, cast shadows, the SDF coast and river bands, and all ten painter styles** are ports; `render.rs` cites their legacy lines.
- **Look presets** are the legacy style-bundle mechanism, reapplied.
- **The dark/light theme toggle** is legacy.
- **The Region tool** is legacy.
- **Commit/Discard for drafts** is legacy, per tool.
- **Generation-parameter undo** is a close port of `pushUndo`/`undoLast`.
- **Brush shapes, stroke and grid options, and sculpt actions** are drawn in a design canvas but have no engine and are not in the legacy file (`GUI_GAP_REGISTER.md` WW-03 to WW-05).
- **Citadels** are ruled for (Ruling I) but not built: zero hits in `cartalith-urban`.
- **Story planning beyond SP-1** is not built. The saved Journey lives in GDScript.
- **The owner's re-sorted CIVIL left rail** (Ruling L) is held, uncommitted, pending verification, so it is not listed.
