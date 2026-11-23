# What Cartalith can do

A feature list for the native app (`cartalith-native/`: a Rust engine inside a Godot shell). It describes what the app
does **today**. It does not cover the legacy HTML version or any plans.

> **A dated cut, 2026-11-23.** Progress is recorded in `cartalith-native/docs/STATUS.md` and nowhere else. Where this
> list and `STATUS.md` disagree, `STATUS.md` wins and this file is stale. Every feature below was checked at its symbol
> in the code (a menu row, an engine function the shell calls, a shell file, or a switch default). The appendix names
> the symbols that were opened.

**How to read the platform tags.** **All** means desktop, tablet and phone. The phone's MENU tab is built from the
desktop menu bar row for row, and the tablet's short bar reaches the same menus through its ☰ overflow, so a feature
reached from a menu is on all three forms unless a gate in the code says otherwise. Features in the domain docks reach
the phone through its domain tab. The exceptions found are tagged.
**On** means the feature is on in a new world. **Off by default** means a setting the user turns on. **Behind a switch,
off** means it is built but switched off in the app.

The app has three working domains, chosen from the left rail on desktop and tablet and from the domain tab on the phone:
**WORLD** (terrain, water, climate, ecology), **CIVIL** (settlements, factions, routes, journeys) and **CARTO** (how the
map looks). The menus are File, Edit, Assets, Records, Story, Preferences, Window and Help.

---

## What sets Cartalith apart

- **Worlds built from physical processes.** Plates move and collide, then volcanoes, impact craters, erosion, glaciers,
  rivers, lakes, climate and biomes follow from them in a ten-stage pipeline. Nothing is painted on.
- **Towns that match the map's rivers and coasts.** A river town stands on the bank of the river the map draws, takes
  that river's width, stretches along it and bridges or ferries across it. Harbour towns lay out to their water.
- **Town plans frozen into the save.** A town you care about keeps its street plan in the project file. It is not
  re-generated when the rules improve, unless you choose to re-lay it.
- **The same world on every device.** The same seed and settings give the same world on desktop, tablet and phone. The
  check is `tools/form_determinism.py`, run on the default CPU path. `STATUS.md` also records identical results on the
  PC, a OnePlus 6T and a OnePlus Pad 3; those device runs were not repeated for this list. GPU generation is opt-in, and
  it gives a different world from the CPU path.
- **A Markdown vault wired into the map.** Notes link to places, factions and landmarks. Dated entries in your notes
  appear on the map's timeline.
- **An optional assistant that can run on your own machine.** It is off until you turn it on. It works with local
  servers such as LM Studio, Ollama or llama.cpp, or with a cloud provider. It can search your notes and the world, and
  it can propose edits that you review.
- **A whole civilisation simulated over time.** Factions, borders, economies, armies, religions and a collapse-and-recovery
  timeline, all derived from the terrain they sit on.

---

## WORLD

### Creating a world
- **New world** (File ▸ New world…): choose a seed, an archetype (Earth-like, Supercontinent, Archipelago, Volcanic,
  Rift), a size from Local 200 km to Planet 40 075 km, a shape from 2:1 to tall portrait, and a resolution from 512 to
  8K. A land-and-sea preview shows before you create. *All.*
- **Auto-populate**: a new world can come with settlements, routes, landmarks and names already placed. Each can be
  turned off. *All; on.*
- **Generation pipeline** (WORLD ▸ Generate): every parameter is grouped by stage: Planet, Geology, Hydrology, Climate
  and Ecology. Edit and re-run, or reset one stage (Edit ▸ Reset one stage). *All.*
- **New world from selection**: make a new, more detailed world from a region of the current one. *All.*
- **Import a heightmap** (PNG), as elevation with tectonics inferred from it. *All.*
- **Centre the world**: rotates a whole-world map so its seam falls in open ocean. *All.*

### Terrain, geology and volcanoes
- **Plate tectonics**: plates, collision and subduction belts, island arcs, rifts and transform faults. Mountain heights
  come from the tectonics, with ridged relief in the mountain belts. *On.*
- **Rock types**: a lithology model gives every cell one of seven rock types, viewable as a map layer. *On.* The rock-aware
  processes (erosion that reads the rock, karst, rock-dependent valleys) are built *behind a switch, off*.
- **Volcanoes by type**: stratocones, scoria cones, lava domes, calderas, maars, a possible supervolcano and flood-basalt
  plateaus. Summit craters are sized from real proportions, active craters stay dry, and lava that reaches the sea builds
  new land. *On.*
- **Impact craters**, placed by a size-frequency law over the world's area. *On.*
- **Erosion**: stream-power carving runs in every world, and glacial carving of U-shaped valleys is on by default. Hillslope,
  momentum, coastal, sediment-fill and tidal-flat passes are *off by default*. Droplet erosion and fjord carving run on
  demand over a finished map. *All.*

### Water
- **Rivers and lakes** with drainage across the whole surface. Water that reaches a hollow carries on to the sea, and
  rivers wander across flat ground instead of running in straight grid lines. *On.*
- **Lakes that obey a water balance**: a basin holds a lake only where inflow beats evaporation; otherwise it is a dry
  playa. *On.*
- **Named rivers**: rivers and other unnamed features get names in the world's naming style. *On.*

### Climate and ecology
- **Climate**: temperature, rainfall, prevailing winds and ocean currents, with Köppen climate classes. Wind and current
  views animate. *All.*
- **Biomes and terrain classes** (15 biome and 13 terrain classes), soil fertility, plant productivity, wildlife,
  wind-throw risk, and resources (copper, tin, iron, gold, salt and timber). *All.*

### Sculpting by hand (WORLD ▸ Sculpt)
- **Sculpt brush and feature stamps**: raise, lower, smooth, and stamp mountains, hills, ridges, plateaus, cliffs,
  canyons, valleys, rivers, lakes, basins, coastlines and volcanoes. Includes mirror and radial symmetry; drainage and
  climate update after each edit. *All.*
- **Biome & Terrain Paint** (B): paint biome and terrain classes by hand. *All.*
- **Pen pressure and tilt** shape the brush where a stylus is present (Preferences ▸ Stylus). *All, with a pen.*
- **Brush edge sliders**: radius and strength on the screen edge while a brush is armed. *Tablet only.*

### Reading the world
- **Data overlays** (38), grouped Base, Climate, Tectonics, Hydrology, Surface and Civilization. They include
  elevation, plates, crust age, geoid and tides previews, rock resistance, river flow, stream order, flood risk, fjord
  mask, landforms, slope, aspect, local relief, settlement suitability, route corridors and travel cost. *All.*
- **Measure tools**: distance, bearing, area, radius, cross-section and vertical difference, plus point sampling. *All.*
- **World data tables** and the field browser (Records ▸ World data tables…). *All.*

---

## CIVIL

### Factions, nations and borders
- **Factions** (Records ▸ Factions…): a roster where you add, remove and rename factions. Each has a colour, a currency,
  a culture, one of the government forms and a religion, plus its relations with the others. A world may have no
  factions at all. *All.*
- **Territory**: claims spread over the land with terrain friction, so borders follow ridges, canyons and rivers. A
  drawn border line marks them. Paint or lasso territory by hand (T, ⇧T). *All; on.*
- **Provinces** divide each nation along the same border logic. *All; on.*
- **City-states**: a large settlement of no faction is a city-state with its own sphere of influence, and you can promote
  a sphere to a faction. *All; on.*
- **Moving a place between factions**, with a preview of the land that would change hands. *All.*

### Settlements and towns
- **Settlements** sited by suitability: water, terrain, river crossings, junctions and harbours. Place, edit, rename,
  re-roll names, set founding years, walls and traits, or delete (Settlement tool, S). Settlements of no faction keep
  their own garrison. *All.*
- **Seed villages** into an existing world, with undo. *All.*
- **Town layouts and the City Viewer**: every settlement has a generated street plan with houses, a market, walls and
  bridges, seen full size in the City Viewer with marker cards and notes. *All.*
- **River and harbour towns**: towns stand on the drawn river's bank and use its width, stretch along their river, reach
  across with bridges (ferries on wide water), and harbour towns face their sea. Villages grow as street villages and
  walls hug the built-up area. *On.*
- **Frozen towns**: freeze a town from the City Viewer, or by placing a point of interest, label or measurement inside
  it. Its plan is then kept in the save. Records ▸ Check frozen towns… and Re-lay all frozen towns… manage them. *All.*
- **Landmarks**: over 40 kinds placed where the terrain and history put them: peaks, gorges, waterfalls, mountain passes,
  fords, harbours, mines, forts, shrines, ruins and more. They have their own types window (Assets ▸ Landmark
  types…). *All.*

### Routes, trade and travel
- **Roads and ways**: generated road networks, plus routed or freehand ways drawn by hand (W) and committed routes
  (⇧R). *All.*
- **Sea lanes and ports** between coastal settlements. *All; on.*
- **Economy**: each faction's economy and currency, trade flows between settlements, food sheds, salt sources and iron
  smelting, shown in the place editor and the faction roster. *All.*
- **Journey planner** (Story ▸ Journey planner): plan a trip by traveller, season, carriage, route and stops. It covers
  river legs, vessels and resupply, and it can save journeys and keep a travel library. *All.*

### Armies, belief and history
- **Military manpower**: standing, field and emergency armies per faction, derived from population, technology and how
  much the land can feed. *All.*
- **Religions**: create custom religions, then run a diffusion model that spreads faith from settlement to settlement
  over the years. *All.*
- **Timeline and collapse/recovery**: record years, run a collapse-and-recovery simulation, and step the map through
  time. Political history of a place and faction history are in the Story menu. *All.*
- **Conflicts and campaigns** (Story ▸ Conflicts…, Draw a conflict): drawn wars with sides and their manpower. *All.*
- **Name styles from CSV** (Records ▸ Name styles…): your own naming lists for places. *All.*

---

## CARTO

### Map looks
- **Style presets**: Natural Vibrant, Antique, Ink, Watercolor, Print, Village, Atlas, Imhof relief, Blueprint, Ink wash,
  Woodcut, Vintage atlas, Nautical, Night, Cel / Toon, Tanaka relief and Satellite relief, each in Dark, Light or Pastel.
  *All.*
- **Hand-drawn effects** to mix in: watercolour, contour lines, ink linework, hachure, posterize, engraving, stipple,
  sepia, risograph and pointillism. *All.*
- **Relief and light**: sun direction, multiple suns, relief exaggeration and lighting presets. *All.*
- **Colours**: edit the elevation colour ramp and the biome colours, with per-layer adjustment stacks (levels, curves,
  hue, saturation, lightness) and a shared look undo history. *All.*
- **Colour paint**: paint colours or materials (lava, embers, ice glow, toxic glow) onto the map as strokes or filled
  areas, without destroying the relief underneath. *All.*

### Layers, labels and pins
- **Layers**: settlements, ways and routes, sea routes, town layouts at deep zoom, landmarks, conflicts, provinces and
  territory, each switchable. *All.*
- **Labels** (L) and **icons** (I), placed by hand or generated with the world's names. *All.*
- **Pin palette and points of interest**: place any pin kind by hand from a searchable palette, including custom pins
  from your asset library. Marker style can be pins, circles, icon pins or painted pins. *All.*

### Assets
- **Asset library** (Assets menu): import asset packs (.zip) or folders of art, slice sprite sheets, compare packs, manage
  icon families and texture sets, and apply the library to the map. The library persists on each device. *All.*

### Output
- **Export** (File ▸ Export): player map and DM/full map, a map image as PNG, BigTIFF or JPEG (sizes offered from 2 048 to 32 768 px wide), tile
  sets (XYZ, TMS or WMTS), and GeoJSON of the whole world. *All.*
- **3D view** (Window ▸ 3D view): a navigable 3D terrain view. Each device measures what it can draw and sets a tier
  (Off, Basic, Standard or Full), which you can override. *All, as the device allows.*
- **Deep zoom**: terrain detail follows the viewed scale through tiled detail and an atlas cache (Preferences). *All.*

---

## Records and the Vault

- **Data manager** (Records ▸ Data manager): one window for import, export and validation (heightmaps, GeoJSON, world
  data, tiles). *All.*
- **Places and Places without notes**: list every settlement, or only the ones you have not written about yet. *All.*
- **Markdown vault** (Records ▸ Markdown vault…, Browse & edit a note…): connect a folder of Markdown notes. *All; on
  Android through the system folder picker.*
- **Note editor**: write and preview, follow `[[links]]` with suggestions over notes, headings and map items, backlinks
  and mentions, rename with link rewriting, templates, tags, properties, task lists and embedded images. *All.*
- **Notes tied to the map**: attach a note to a place, faction, landmark, label or icon; open it from the map's context
  card (Story ▸ Note for the selection). Notes can be marked private. *All.*
- **Chronos timelines**: dated events in your notes appear on the map's year strip with founding, conflict and journey
  events. The strip has four levels, from a minimal bar to a filterable table and a consolidated timeline document. Vault
  timelines are offered for linking or import from a passive badge, never automatically. *All.*

---

## The Assistant

- **Optional language-model assistant** (Story ▸ Assistant panel…, Preferences ▸ Assistant…). *Off by default; nothing
  is sent while it is off.* *All.*
- **Local or cloud**: presets for Ollama, LM Studio, llama.cpp, vLLM, OpenRouter, OpenAI, Anthropic and Gemini (through
  an OpenAI-compatible endpoint). Local and remote use are allowed separately. *All.*
- **Three permission levels** to turn on separately: answer questions, run commands, and make edits. Edits include
  terrain sculpts and colour paint inside a region you paint. Each edit arrives as a card you apply, and it can be undone.
  *All.*
- **Knowledge search** over your vault and the world's own data. Notes marked private are never sent. *All.*
- **Keys kept in the system's secure store**: Windows Credential Manager on the PC, the Android Keystore on phones and
  tablets. *Desktop and Android.*
- **Ask about the selection** (Story menu): ask about whatever is selected on the map. *All.*

---

## Platform

### Forms and controls
- **Three shells from one app**: desktop, tablet and phone layouts, chosen from the window's size and re-chosen live when
  it changes. *All.*
- **Context card**: right-click (or long-press) a map item for its actions. *All; a sheet on the phone.*
- **Cartwheel**: an 8-slot tool ring at the pointer (Q, or hold). *Desktop and tablet.*
- **Context panel**: a right dock with Inspect, Note and Ask plus modules you arrange as tabs. *All; a sheet on the phone.*
- **Quick bar**: pin any command to the top bar (Preferences ▸ Quick bar…). *All.*
- **Find on map** (Edit ▸ Find on map…) finds places, and a `.` prefix searches commands. Keyboard shortcuts can be
  rebound, and Help has a gesture reference. *Shortcuts: desktop; the rest: all.*
- **Panels in their own windows**, for a second monitor (Preferences ▸ Open panels in their own window). *Desktop only;
  off by default.*
- **Theme** (dark, light or follow the system), **units**, **UI scale** (100-200 %), **handedness** on touch forms,
  saved layouts and session mode (Window menu). *All.*
- **Undo** with an undo history window and a memory budget you can set. *All.*

### Saves and files
- **Project files** (.zip) holding the world, civilisation, art, frozen towns and vault links, with compressed rasters.
  Save, save as, revert, recent worlds, autosave on an interval and share. *All.*
- **Opens older saves**, including flat saves from the HTML era with their settlements, labels and icons. *All.*

### Installing and running
- **Windows installer**: a per-user setup with a Start-menu shortcut and an uninstaller; newer versions upgrade in
  place. *Windows.*
- **Android build** (APK) for phones and tablets. *Android.*
- **Multithreaded generation**, with the worker-thread count in Preferences. *All.*
- **GPU acceleration**, with a choice of graphics device, in Preferences. *Off by default on every device.*
- **Diagnostics**: a diagnostics overlay, a diagnostic trace (*off by default*), and a diagnostic report to save
  (Help ▸ Save diagnostic report…). *All.*

---

## Unverified (left out of the list above)

- **Platform tags** come from gates in the code. No device or windowed run was made for this list.
- **Stylo 2 pen link** (Preferences ▸ Stylus ▸ Pen link, touch forms). The menu exists, but it needs an Android plugin
  (`cartalith-native/android/pen-plugin/`) that is not under the Godot project's `addons/`. Whether a shipped APK carries
  it was not checked, and the code treats the plugin as absent when it is missing.
- **Haptics.** Not checked in this pass.
- **Multi-GPU modes** (split tiles, alternate frames) and the VRAM budget. The rows are built, but what they do was not
  checked in this pass.
- **Map image import** (Data manager ▸ Import ▸ Maps), **external sources** and **validation checks** show as gaps in
  the code itself, so they are left out. The same applies to the Print sheet (PDF) and VTT battle map rows in the
  Export menu.

## Not yet

- **Parallel streets (PS-5)**: regular, parallel street blocks for planned towns. Built behind `urban_parallel_streets`,
  off; the back-street tier and the water gate are next.
- **Coastal setback**: markets come to the water unless surge, cliffs or marsh give a reason to stand back (CS-1 onward).
- **World-scale terrain**: 16k-32k tiled worlds with region cut-outs, parked until after the refactor.
- **The codebase refactor**: the planned restructuring of the code, scheduled after the current list.

---

## Appendix: what was opened

Opened in the worktree at `origin/main` `0ccfd3a7`, 2026-11-23. Paths are relative to `cartalith-native/`.

- **Menus and forms.** `godot-project/shell/menus.gd`: `build()` (the eight-menu bar and the tablet short bar),
  `_overflow`, `_file`, `_build_file_export`, `_data`, `_story`, `_preferences` (the GPU and thread rows,
  `PopoutHost.supported()`, the `is_touch()` gates), `_window` with `_build_view3d_rows`, and `_help`.
  `phone_menu.gd` header (MENU repeats the desktop menus row for row). `dcc_shell.gd` `DOMAINS`, `RAIL_NODES`,
  `PHONE_TABS`. `command_index.gd` `EXTRAS`.
- **Switch defaults.** `crates/cartalith-godot/src/params.rs` `defaults()` (every `p.* = true` line, plus
  `geology_processes`, `relief_model`, `geology_karst` and `geology_carve` false); `params.rs` `LOADS_WHEN_ABSENT`
  (`urban_parallel_streets` false). `crates/cartalith-engine/src/lib.rs` `WorldParams::defaults` (pass booleans false,
  `urban_parallel_streets: false`). `godot-project/shell/engine_bridge.gd` `default_gpu_on()` (returns false).
  `crates/cartalith-llm/src/settings.rs` (`enabled: false`, `defaults_are_off_and_empty`).
- **WORLD.** `new_world_dialog.gd` `SIZE_PRESETS`, `ASPECT_PRESETS`, `RESOLUTION_PRESETS`, the archetype names and the
  preview; `auto_populate.gd` `ORDER`, `DEFAULT_ON`; `workspaces/world_workspace.gd` `STAGES`, `CATEGORIES`, the tools
  block, `RING_UPLIFT_KEYS`; `tool_bar.gd` `SYM_LABELS`, `WAY_MODES`; `crates/cartalith-terrain/src/sculpt.rs` `Feature`,
  `FreehandMode`; `lib.rs` `carve_fjords`, `import_heightmap`; `app.gd` `CENTRE_TOOLTIP`, `_import_heightmap_at`;
  `sample_bridge.rs` `LAYER_GROUPS` (entries marked "Not available" were excluded); `wind_fx_layer.gd`;
  `global_tools.gd` measure tools; `RING_CARVE_KEYS`; `cartalith-civ/src/landmark.rs` `kinds()` (only `buildable: true`
  kinds named).
- **CIVIL.** `workspaces/civilization_workspace.gd` and `infrastructure_workspace.gd` categories and tools blocks; shell
  callers of `civ_government_vocabulary`, `civ_add_religion`, `civ_belief_run`, `civ_run_collapse_simulation`,
  `civ_military_summary`, `civ_faction_economy`, `civ_trade_flows`, `civ_food_shed`, `civ_polity_reassign`,
  `civ_promote_sphere`, `civ_settlement_sphere` (`context_card.gd`), `civ_seed_villages_now`, `civ_territory_paint_at`,
  `conflict_add`, `conflict_campaigns`, `civ_set_town_plan`, `town_freeze`; `viewport_host.gd` (`border_lines`,
  `province_lines`); `city_state_bridge.rs` header and `lib.rs`'s sphere wash; `town_layout_bridge.rs` `town_freeze`
  docs; `city_viewer_window.gd` Frozen town section; `place_editor_window.gd` trade notes.
- **CARTO.** `workspaces/cartography_workspace.gd` tools block, categories and `LIVE_LAYERS`;
  `workspaces/render_workspace.gd` `STYLES`, `STYLE_PRESETS`, `STYLE_VARIANTS`; `colour_paint_tool.gd`; `pin_palette.gd`
  (custom pins); `adjust_panel.gd`; `look_history.gd`; `menus.gd` `_build_marker_style_menu`;
  `crates/cartalith-godot/src/export_options.rs` `FORMAT_NAMES`; `export_raster.rs` `BAKE_WIDTHS`;
  `data_manager_window.gd` `ROUTES` (kind `live` only); `view3d_gate.gd` `TIER_NAMES`.
- **Records and Vault.** `crates/cartalith-godot/src/vault_bridge.rs` (the `vault_*` functions) and their shell callers
  (`vault_rename`, `vault_suggest`, `vault_toggle_task`, `vault_read_image` in `vault_window.gd`; `vault_file_backlinks`,
  `vault_tag_index`, `vault_note_properties` in `vault_side_panel.gd`; `vault_private_marks` in `vault_privacy.gd`); `vault_saf_android.gd`;
  `chronos_items.gd` and `chronos_prompt.gd` headers, mounted by `app.gd`.
- **Assistant.** `crates/cartalith-llm/src/presets.rs` preset ids; `settings.rs` `Capability`;
  `crates/cartalith-godot/src/llm_keystore.rs` and `llm_keystore_android.rs`; `knowledge_bridge.rs`.
- **Platform.** `crates/cartalith-io/src/project.rs` `PROJECT_FORMAT_VERSION` (3); `legacy_import.rs`;
  `installer/cartalith.iss`; `godot-project/export_presets.cfg` (Windows Desktop and Android); `tools/form_determinism.py`
  and `godot-project/tools/form_determinism_drive.gd`; `context_broker.gd` (the ring is not built on the phone),
  `menus.gd` `_build_map_surface_rows`, `dcc_shell.gd` `_ensure_command_index`, `engine_bridge.gd` `gpu_api`. `docs/STATUS.md` entries "CPU by default on every device", "Form
  determinism", "Windows installer", "Ruling GX and Ruling GW" and the Pad 3 timing entry (the cross-device digests were
  read there, not re-run).
- **Method and its limit.** The platform tags come from the menu mirroring and the gates listed above. This pass ran no
  device or windowed probe.
