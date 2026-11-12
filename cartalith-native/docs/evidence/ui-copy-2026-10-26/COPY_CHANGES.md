# UI copy pass, 2026-10-26

Before/after of the user-facing strings changed in this pass. Classification: SHORTEN (same meaning, fewer words), REMOVE (developer note deleted), REWORD (plain language). Detail that a user needs to make a safe choice was kept; long technical explanation lives in `LLM_ASSISTANT_SCOPE.md` s.6 and the code comments.

Rust (`cartalith-llm/src/error.rs`, `Display` of `LlmError`, shown in the panel): transport error -> "Could not reach the server"; provider error (HTTP n) -> "The server returned an error (code n)"; invalid API key/request/base URL -> "The API key / request / server address is not valid"; length cap -> "The reply was cut short by the reply length limit". The key is still scrubbed before display (unchanged `Scrubbed` type; tests `error_body_echoing_the_key_is_scrubbed` etc. pass).

| Surface | Before | After |
|---|---|---|
| Assistant settings | note": "The master switch. Off means nothing is sent. | note": "Off means nothing is ever sent. |
| Assistant settings | note": "Prose for a place, its story, or a Vault note, from Story ▸ Notes ▸ Assistant panel. Notes are written only on your click. | note": "Draft text for a place, its story or a note. Nothing is saved until you click. |
| Assistant settings | note": "Generation-setting proposals and view or layer commands (LLM-6): the model proposes, you review the card and press Apply; one Apply is one undo step and never regenerates. Also carries questions about the world and your notes. | note": "The assistant suggests settings, view and layer changes. You review each one and press Apply; one undo step reverses it. |
| Assistant settings | unavailable": "Not available in this build yet: edit actions (LLM-7) are not built. Grouped undo, which they need, exists. | unavailable": "Coming later. |
| Assistant settings | note": "This computer and your local network.", | note": "Models on this computer or your network.", |
| Assistant settings | note_phone": "This phone and your local network, such as a model server on your computer. | note_phone": "Models on this phone or your network. |
| Assistant settings | note": "Hosted endpoints on the internet. | note": "Models run by a company on the internet. |
| Assistant settings | note": "Needed for setting proposals and for questions: the model is shown the current value of each setting it asks about, and can read whole-world figures (sea level, land fraction, elevation percentiles, mean rainfall and temperature, productivity) and up to 8 single map cells. Reading changes nothing. | note": "Lets the assistant see your current settings and world-wide figures such as sea level and rainfall. It only reads. |
| Assistant settings | note": "Excerpts of the notes that match your request, never a whole note unasked and never a note you marked private (dm or private in its properties, the tag dm-only or secret, or a DM, Private or Secrets folder). | note": "Short excerpts of notes that match your request. Notes marked private, dm-only or secret, or kept in a DM, Private or Secrets folder, are never sent. |
| Assistant settings | note": "The facts the app holds about this world that match your request: names, populations, who owns what, trade partners, faiths, history. World data has no private marker: everything in it is sent as found. | note": "World facts that match your request (names, populations, trade, faiths, history). World data has no private marker, so everything found is sent. |
| Assistant settings | Search before asking (automatic: on for a local model) | Search first (automatic for local models) |
| Assistant settings | Search before asking (always) | Search first (always) |
| Assistant settings | Do not search before asking | Do not search first |
| Assistant settings | Optional. Everything here is off until you turn it on, and with it off nothing is sent anywhere. " + 		"A world generates the same with or without a model. | Optional. Everything is off until you turn it on. Worlds generate the same with or without an assistant. |
| Assistant settings | Nothing is sent unless ticked, per source. Whole notes, the whole Vault, the save file, file paths and the machine name are never sent unasked. | Only ticked sources are sent. Whole notes, your whole Vault, save files, file paths and your computer name are never sent unasked. |
| Assistant settings | Search is used only when a box above is ticked. The app can search your request first and hand the model the best five excerpts (needed by models that never call a tool), and/or let the model search and read sources itself. | Used only when a search box above is ticked. Cartalith can search for you first, or let the assistant search itself. |
| Assistant settings | Automatic searches your request before the model sees it when the model is on this computer or your network, and not for a hosted model unless you allow it below. | Automatic searches first for models on this computer or network, and for hosted models only if you allow it below. |
| Assistant settings | note": "Offers the model three read-only tools (search, read one source, list what is connected to it). At most 12 lookups per question; each result is at most 12 000 bytes. | note": "The assistant can look things up itself, read-only, up to 12 times per question. |
| Assistant settings | A hosted model is run by a third party. Excerpts of your notes and your world's data are sent to it only if you allow it here, per endpoint. | Hosted models are run by a third party. Excerpts of your notes and world are sent only if you allow it, per service. |
| Assistant settings | A provider only fills in defaults. Your address, model and key always win; the model is free text. | A provider fills in defaults. Your own address, model and key always win. |
| Assistant settings | Where requests go. http:// is allowed only for this computer or your network; a remote address must be https://. | Where requests go. Use http:// only for this computer or your network; other addresses need https://. |
| Assistant settings | The models this server offers, asked when this window opens and when the address, key or a switch changes. Pick \"Type a name…\" to enter one by hand. | The models this server offers. Pick \"Type a name…\" to enter one yourself. |
| Assistant settings | Type a model id exactly as your provider expects it. Always available, whatever the list says. | Type the model name exactly as your provider spells it. |
| Assistant settings | Where the key is read from. Cartalith never writes a key to a file. | Where the key comes from. Cartalith never saves a key to a file. |
| Assistant settings | The NAME of an environment variable that holds the key. Set it before starting Cartalith. | The name of the environment variable that holds the key. Set it before starting Cartalith. |
| Assistant settings | Stored in the OS credential store, never in a file, under this profile's own entry. It is cleared from this field when you press Save and is never shown again. | Kept in your system credential store, never in a file. The field clears when you press Save and the key is never shown again. |
| Assistant settings | Automatic switches only when the model is on this computer. Always switches whatever the address says (for Docker, WSL, a tunnel). Never leaves your GPU choice alone. | Automatic switches only for models on this computer. Always switches for any address (Docker, WSL, tunnels). Never leaves your GPU alone. |
| Assistant settings | Shows how many tokens a session has used. It is a readout, not a limit. | How many tokens this session has used. Not a limit. |
| Assistant settings | Optional. When the tokens servers have reported this session reach this number, Send is refused until you raise it, set it to 0, or reset the counter. 0 means no cap, which is the default. | Optional. Send stops when this session reaches this many tokens. 0 means no cap. |
| Assistant settings | No spend limit is enforced unless you set a cap above. A cap counts only what servers report, so a server that reports nothing is not counted. A hosted provider bills you directly. | No spend limit is enforced unless you set a cap above. A cap counts only what the server reports. A hosted provider bills you directly. |
| Assistant settings | A profile is a whole set-up: provider, address, model, thinking, what is sent, reply cap, retries and (for a local model) the GPU choice. Switching applies all of it at once. The master switch, Commands, Edit and the Local / Remote provider switches belong to no profile and never change. A key is never part of a profile; each profile keeps its own in the OS credential store. | A profile is a saved set-up: provider, address, model and what is sent. Switching applies all of it. The master switch, Commands and the Local / Remote switches are not part of a profile. Each profile keeps its own key in the credential store. |
| Assistant settings | 1 to 40 characters. A new profile starts empty with nothing sent; pick a provider and a model, then save a key for it if it needs one. | 1 to 40 characters. A new profile starts empty: pick a provider and a model, then save a key if it needs one. |
| Assistant settings | Off asks a reasoning model not to think first, on every request, using the field the provider is known to take (LM Studio and Ollama reasoning_effort none; llama.cpp and vLLM the template switch). A provider with no known field is sent nothing. | Off asks a reasoning model not to think first. Providers that cannot do this are left alone. |
| Assistant settings | Pins the longest reply this profile asks for. 0 uses each task's own cap (about a page of prose; longer for a place entry). From 16 to 8192. | The longest reply to ask for, 16 to 8192. 0 lets each task choose. |
| Assistant settings | After a transient failure (a busy server, a dropped connection before any text) the app can try once more. Never more than once. | After a brief failure, such as a busy server, try once more. Never more than once. |
| Assistant settings | Not needed: this endpoint is on this computer or your network. | Not needed: this model is on this computer or your network. |
| Assistant settings | This build has no assistant support. Update the Cartalith library to use it. | The assistant is not available in this version of Cartalith. |
| Assistant panel / quick bar / pop-out | Commands proposes generation-setting changes and view changes (layers, style preset, landmark types, centring and zooming, opening a window) for you to review, and answers questions from read-only lookups (and from your notes and world data when ticked in Settings); nothing changes until you press Apply. It never generates, saves, closes, exports or deletes. Edit actions arrive in a later release. | Suggests setting and view changes and answers questions about your world. Nothing changes until you press Apply. It never generates, saves, exports or deletes. |
| Assistant panel / quick bar / pop-out | The model's proposal could not be used (%s). It was told why and asked once more. | The suggestion could not be used (%s). Asking the assistant to try again. |
| Assistant panel / quick bar / pop-out | The model asked for setting groups that do not exist; it was told the valid ones. | The assistant asked for settings that do not exist; asking again. |
| Assistant panel / quick bar / pop-out |  If the model could not pick setting groups, press Choose groups, tick up to %d and send again. |  If it could not find the right settings, press Choose groups, tick up to %d and send again. |
| Assistant panel / quick bar / pop-out | Tick \"Send: generation parameters and summary statistics\" in Settings first (a proposal or a question shows the model the current settings and figures), or tick \"Send: Vault notes found by search\" or \"Send: world data found by search\" to ask about your notes and world. | In Settings, tick \"Send: generation parameters and summary statistics\" to use this, or tick a search source (Vault notes or world data) to ask about your notes and world. |
| Assistant panel / quick bar / pop-out | Ask for a settings change (a drier climate), a view change (hide the roads, centre on a cell, switch the style) or a question about the settings or the world… | Ask for a settings change (a drier climate), a view change (hide the roads) or a question about the world… |
| Assistant settings |  Needs the master switch. |  Turn on Assistant enabled first. |
| Assistant settings | The master switch, Commands and the Local / Remote switches are not part of a profile. | Assistant enabled, Commands and the Local / Remote switches are not part of a profile. |
| Assistant profile switch | Switch the whole assistant set-up at once: provider, address, model, thinking, what is sent, reply cap and retries. The master switch, Commands and Edit are not part of a profile and never change. | Switch the whole assistant set-up at once: provider, address, model and what is sent. Assistant enabled and Commands are not part of a profile. |
| Quick bar window / pop-out button | The chip appears under the cartwheel and at the start of the bar once the assistant is enabled and a provider is set. This switch hides it; it is not one of your pins. | Shows the Assistant button once the assistant is ready. Turn off to hide it. |
| Quick bar window / pop-out button | The Assistant chip is hidden: it needs the assistant switched on in Preferences, a provider and a model, and this switch on. | The Assistant chip is hidden: turn on the assistant in Settings, choose a provider and model, and turn this on. |
| Quick bar window / pop-out button | Moves this panel into its own operating-system window, for a second monitor. Press again to dock it back. Your place in it (tab, selection, scroll) is kept. | Opens this panel in its own window, for a second monitor. Press again to dock it back. |
| Quick bar window / pop-out button | Up to %d chips under the cartwheel, beside its eight slots (never replacing one). A quick flick never selects them. | Up to %d buttons under the cartwheel, beside its eight slots. |
| Quick bar window / pop-out button | Pins go after a diagonal's built-in items (a sub-ring places at most %d). Single-tool diagonals cannot share. | Pins go after a diagonal's own items (at most %d). Single-tool diagonals cannot take pins. |
| Consent card | for a view proposal: which map layers are on, the style preset names, the landmark types this world has, the grid size and zoom range, and the names of the windows it may open (a fixed list of read-only ones); a proposal changes nothing until you press Apply | for a view suggestion: which map layers are on, the style names, the landmark types, the grid size and zoom range, and the names of a few windows it may open; nothing changes until you press Apply |
| Consent card | for a lookup in %s: the words searched and the excerpts found, which may include names and notes you typed. Before the model answers, the app may send the best %d (up to %d characters in all); the model may ask for up to %d more lookups of at most %d bytes each. Every answer lists the sources sent. | for a lookup in %s: the words searched and the excerpts found, which may include names and notes you typed. Up to %d excerpts (%d characters) may go first, and the assistant may look up to %d more times (%d bytes each). Every answer lists the sources sent. |
| Consent card | for a question: any setting's value and default, whole-world summary figures (sea level, land fraction, elevation percentiles, mean rainfall and temperature, productivity) and up to 8 single map cells (elevation, slope, climate, biome); never a map image | for a question: setting values, world-wide figures (sea level, land fraction, rainfall, temperature) and up to 8 map cells (elevation, slope, climate, biome); never a map image |
| Asset library, line art, export, world/civilization workspaces, command index | Each writes a new variant and leaves the original as it was (Ruling CQ Q7). | Each writes a new variant and leaves the original as it was. |
| Asset library, line art, export, world/civilization workspaces, command index |  (Ruling CQ Q4). No paper colour | . No paper colour |
| Asset library, line art, export, world/civilization workspaces, command index |  (Ruling CR Q-L1). 1 px keeps | . 1 px keeps |
| Asset library, line art, export, world/civilization workspaces, command index |  Only scans need it (Ruling CR). |  Only scans need it. |
| Asset library, line art, export, world/civilization workspaces, command index | Planned journey routes. Not in Ruling AZ's v1 list; off by default. | Planned journey routes. Off by default. |
| Asset library, line art, export, world/civilization workspaces, command index |  (Ruling AZ: towns by default). |  (towns by default). |
| Asset library, line art, export, world/civilization workspaces, command index | Painting biome and terrain classes by hand, over the classified fields. Ruling L decision 2: this is the brush's one home, and Biome paint (B) in the tool palette is the way to arm it. | Paint biome and terrain classes by hand. Use Biome paint (B) in the tool palette to start. |
| Asset library, line art, export, world/civilization workspaces, command index | Unit movement and battles that resolve themselves: Ruling AW did not ask for them, and nothing here moves a force or decides a fight. | Units do not move and battles are not simulated; nothing here decides a fight. |
| Asset library, line art, export, world/civilization workspaces, command index | (RELIGION_DIFFUSION_SCOPE.md milestone 1).  |  |
| Asset library, line art, export, world/civilization workspaces, command index | No registered handler yet: an EXTRAS row is a search answer, not a callable command (CUSTOM_QUICK_BAR_SCOPE.md s.3.3; CQ-0 measured it). | This is a search result, not a command, so it cannot be pinned. |
| Asset library, line art, export, world/civilization workspaces, command index |  (CQ-8). | . |

Also: `This build has no assistant support[. Update the Cartalith library]` -> `The assistant is not available in this version of Cartalith.` (13 places in llm_assistant_window, llm_models, llm_profiles, llm_run, engine_bridge). Phone Gestures page: the `Confirmed in this build.` suffix and the `Each row above names the symbol...` developer note removed; `Not in this build` heading -> `Not available`.

## Counts

- Assistant settings: 39
- Assistant panel / quick bar / pop-out: 6
- Assistant profile switch: 1
- Quick bar window / pop-out button: 5
- Consent card: 3
- Asset library, line art, export, world/civilization workspaces, command index: 11

## Left for the owner / next pass

- `Not in this build` on pinned quick-bar commands (4 files, many probes) and the `... on this binary` engine-bridge errors (about 60) are still developer wording; they are probe-pinned or non-visible in normal use.
- Long place-facts, knowledge and view-preview texts in `llm_assistant_window.gd` (the `This will send:` preview) are safety text and were left whole.
- Strings LLM-7 was adding were not touched.
- Screenshots: see this folder if present.

## Verification state

- cargo test -p cartalith-llm: all green (two literals in `llm1_bars.rs` updated to the new exact text).
- Windowed probes green: `_llmset_probe` (272 ok, 0 fail), `_llmpanel_probe` (93), `_quickbar_assistant_probe` (107). Probe literals changed only for `Coming later.` and `Turn on Assistant enabled first.` (exact-match assertions).
- NOT run (budget): `_quickbar_window_probe`, `_llmview_probe`, `_llmknow_probe`, `_llmmodels_probe`, `_llmprofiles_probe`, `_quickbar_*_probe`, `_popout_roster_probe`, `_phoneui_probe`, `_alignment_probe`, the phone 393x852 clipping check, and the before/after screenshots. Strings changed outside the Assistant surfaces (asset library, line art, export, workspaces, command index, popout tooltip, phone gestures page) are unverified by probes.


---

# Second pass, 2026-11-12

Same rules (plain, short, user-facing, no internal jargon, no build or binary talk). Deferred this pass because other agents were editing them: `llm_*.gd` and the assistant window, the command index and its search UI, `new_world_dialog.gd`, `auto_populate.gd`, `workspaces/world_workspace.gd`, the phone quick drawer, `quick_bar*.gd`, `phone_menu.gd`, the vault editor files (`markdown_editor.gd`, `vault_*`), and the river context card. That deferral covers the quick bar's `Not in this build` wording (`quick_bar.gd`, `quick_bar_phone.gd`, `quick_bar_ring.gd` and their pinned probes) and `phone_menu.gd`'s copy of the gesture-help tip, which still reads "The touch vocabulary this build has".

Also changed, not in the table: 39 engine-bridge errors of the form `<function> not available on this binary` (all in `engine_bridge.gd`) became `This action is not available in this version of Cartalith.`, and the asset library's `Not available on this binary.` became `Not available in this version of Cartalith.`. `_gsw_probe.gd` pinned four of the bridge strings and was re-pointed to the new text; `_presetgal_probe.gd` pinned `not in this build` (now `not available`); `_lanedfoodshed_probe.gd` pinned the symbol and milestone names in the place editor's dash text and now pins `returned nothing` and `older version of Cartalith`, which still tells the held-pass branch from the absent-pass branch.

Left as they were on purpose: `Diplomacy: not built` and the faction emblem `Not built` lines (several probes pin them and the words are plain), the `Not built yet:` heads of the Context panel rows (plain; only the milestone and document references were removed), the menu `_todo` vocabulary, and long safety text.

| Surface | Before | After |
|---|---|---|
| Engine bridge errors | "this build has no adjustment stacks" | "Adjustments are not available in this version of Cartalith." |
| Engine bridge errors | "this build has no bake_all()" | "Baking is not available in this version of Cartalith." |
| Engine bridge errors | "this build has no bake_visible()" | "Baking is not available in this version of Cartalith." |
| Engine bridge errors | "this build of the engine has no heightmap export" | "Heightmap export is not available in this version of Cartalith." |
| Engine bridge errors | "this build of the engine has no raster export" | "Image export is not available in this version of Cartalith." |
| Engine bridge errors | "this build of the engine cannot embed a %s profile -- rebuild cartalith-godot" % color_space | "This version of Cartalith cannot embed a %s colour profile." % color_space |
| Engine bridge errors | "this build of the engine has no %s -- rebuild cartalith-godot" % method | "This version of Cartalith does not offer %s." % method |
| Engine bridge errors | "this build has no atlas_import_zip()" | "Atlas import is not available in this version of Cartalith." |
| Engine bridge errors | "journey_set_plan is not exposed by this build's GDExtension binary -- rebuild cartalith-godot." | "The journey planner is not available in this version of Cartalith." |
| Engine bridge errors | "this build's GDExtension predates apply_geojson_document -- rebuild cartalith-godot" | "GeoJSON import is not available in this version of Cartalith." |
| App status lines | "this build's engine has no heightmap import" | "heightmap import is not available in this version of Cartalith" |
| App status lines | "%d travel-library field(s) in this archive are not recognised by this build" | "%d travel-library field(s) in this archive are not recognised by this version of Cartalith" |
| App status lines | or in a shape this build could not read, and were left as they are in the file" | or in a shape this version of Cartalith could not read, and were left as they are in the file" |
| App status lines | "Labels & overlays adjustments were not saved: this build of the engine has no slot for them in the project file." | "Labels & overlays adjustments were not saved: this version of Cartalith cannot store them in the project file." |
| App status lines | "The map style variant was not saved: this build of the engine has no slot for it in the project file." | "The map style variant was not saved: this version of Cartalith cannot store it in the project file." |
| App status lines | "this build has no region resample binding" | "region resampling is not available in this version of Cartalith" |
| App status lines | "this build of the app can't read the world's seed" | "the world's seed is not available in this version of Cartalith" |
| App status lines | "this build cannot open a pack in the library" | "opening a pack in the library is not available in this version of Cartalith" |
| Asset library | "this build of the app can't read collection membership" | "collection membership is not available in this version of Cartalith" |
| Asset library | "this build of the app can't read the item total" | "the item total is not available in this version of Cartalith" |
| Asset library | "this build of the app can't read the collection count" | "the collection count is not available in this version of Cartalith" |
| Asset library | Nine families: the reference engine's eight, frozen, plus this port's own Sea marks -- not the design mockup's 24. The mockup subdivides more finely (splitting, for example, \"Feature icons\" into \"Trees & cover\" / \"Rock & scree\"); nothing in this build dr | Nine families, including Sea marks. Capacity and fill counts shown here are both real. |
| Asset library | Real: drag one or more selected tiles onto a Collections-rail row. Dropping a file from outside the app straight onto a slot is not wired up yet -- that needs the drop point matched against the grid, a real but unscheduled piece of work, not an impossibility.  | Drag one or more selected tiles onto a Collections row. Dropping a file from outside the app onto a slot does not work yet; use Import image… instead. |
| Asset library | Crops each cell to its content. Note: the reference slicer has no trim -- this is a port-side addition (DCC_SHELL_SPEC.md §8), using the reference's own alpha>8 threshold so it agrees with Skip empty cells about what content is. | Crops each cell to its content, using the same transparency threshold as Skip empty cells. |
| Asset library | Where the cells land. The first four are the reference's own targets; \"a family, slot by slot\" is DCC_SHELL_SPEC.md §8's instead. | Where the cells land. The last option fills a family slot by slot. |
| Data manager | No source registry exists anywhere in the workspace. §9 designs no pane for this route either -- GUI_GAP_REGISTER.md DM-06 is classed (C), needing a design before it can need code. | There is no source registry yet, so this route has nothing to show. |
| Data manager | Same -- no source registry exists. The canvas's `1` badge on this row is mockup data, not a count this build could produce, so no badge is drawn. | There is no source registry yet, so no count is shown. |
| Data manager | No CRS handling exists anywhere in the workspace -- reprojection was the substance of the Conversion group the owner deleted on 2026-08-20 (GUI_GAP_REGISTER.md §7.4), and no import or export path has carried a projection since. | Cartalith does not reproject: no import or export carries a coordinate system. |
| Data manager | Not built yet is the export half -- obsidian:// links in exported tiles and note links in exported GeoJSON -- plus two-way sync, which is deliberately left out of this first version. The three checkboxes below are those three items. | Not available yet: obsidian:// links in exported tiles, note links in exported GeoJSON and two-way sync. The three checkboxes below are those three items. |
| Data manager | Not built. cartalith-vault deliberately generates no obsidian:// URLs (its own module doc: \"no obsidian:// scheme here, no wikilink generation\"), cartalith-engine::geojson writes no note property, and two-way sync is MARKDOWN_VAULT_INTEGRATION.md §33's expli | Not available yet. Linking an entity to a note works today, in Data ▸ Markdown vault. |
| Data manager | This route writes loose files, not one project .zip. The save writer (cartalith_io::write_save, FI-01) and these two rasters are both real now; assembling exportZip's full archive -- params.json, the f32 layer blobs, map.png, the atlas and features.json in one | This route writes loose files, not one project .zip. Use File ▸ Save for the full project archive. |
| Data manager | Four areas, not §9's five. The Conversion group (Coordinate Systems / Format Conversion / Data Transformation) was deleted on the owner's 2026-08-20 decision -- GUI_GAP_REGISTER.md §7.4 found no serious GIS application carries a top-level Conversion route, bec | Four areas: Import, Export, Sources and Vault. Reprojection belongs to the import or export step that reads the file. |
| Data manager | binding missing in this build | not available in this version of Cartalith |
| Data manager | export failed — no world is loaded, or this build's GDExtension predates export_geojson | export failed — no world is loaded, or GeoJSON export is not available in this version of Cartalith |
| Data manager | This window cannot tell which absence that is: a loaded .zip save retains no channel topology at all (SAVEFILE_COMPAT.md), a generation may still be in flight, and a generated world can simply have no river reaching the exported order. The exported file carrie | There are no rivers to export. A loaded .zip save keeps no river channels, a generation may still be running, or the world has no river of that order. The exported file carries no river either way. |
| Data manager | "Connect any folder of .md files -- Obsidian is one such folder, and nothing requires it -- and settlements | "Connect any folder of Markdown (.md) notes -- Obsidian is one such folder, and nothing requires it -- and settlements |
| Other windows (adjust_panel.gd) | These adjustments last for this session only: this build's project file has no place to store them yet, so saving does not keep them. | These adjustments last for this session only; saving does not keep them. |
| Other windows (context_modules.gd) | A landmark has no dated history in this build: its note's dated entries are not indexed (the chronos index reads settlement and faction notes only), and the engine records no event for a landmark. | A landmark has no dated history: dated entries in its note are not on the timeline, which reads settlement and faction notes only. |
| Other windows (dcc_shell.gd) | Placed landmarks are named now (Ruling EI: type plus nearest place, as the vault links them) but are not in this index yet. Link to one from a note with [[ instead. | Placed landmarks are named by type and nearest place, but are not in this index yet. Link to one from a note with [[ instead. |
| Other windows (layers_popover.gd) | No field views: this build's engine has no debug_layers() binding. | No field views are available in this version of Cartalith. |
| Other windows (cartography_workspace.gd) | No field views: this build's engine has no debug_layers() binding. | No field views are available in this version of Cartalith. |
| Other windows (right_dock.gd) | No colour-ramp editor: this build's engine has no set_color_ramp() binding. | No colour-ramp editor is available in this version of Cartalith. |
| Other windows (tool_bar.gd) | The canvas's draggable A/B handles are not built -- re-clicking is the same two clicks and needs no on-canvas hit test of its own. | Click the map again to move point A or B. |
| Other windows (rock_paint_ui.gd) | const SAVE_SHORT_OLD := "not saved with the project by this build of the engine" | const SAVE_SHORT_OLD := "not saved with the project by this version of Cartalith" |
| Travel library | No live effect yet: this is not one of the four built-in party-form species (donkey/mule/camel/horse), and its Substitutes for field reaches none of them. It is real, validated, inspectable data, but the party form's JpParty shape has no slot for a new species | No effect on journeys yet: this is not one of the four built-in party species (donkey, mule, camel, horse), and its Substitutes for field names none of them. Set Substitutes for to one of the four, or duplicate one of them, and it will affect computed journeys |
| Travel library | No live effect yet: vehicles are real, validated, inspectable data, but no resolver hook exists for jp_capacity's vehicle constants -- there is no vehicle_overrides() beside vessel_overrides(), so nothing a vehicle carries reaches a computed journey (TRAVEL_LI | No effect on journeys yet: vehicles are saved and checked, but the journey planner does not use them. |
| Faction roster | This faction's stored government is not one this build knows; it is kept as saved. Pick a form to replace it. | This faction's government is not one this version of Cartalith knows; it is kept as saved. Pick a form to replace it. |
| Faction roster | No profile for the culture \"%s\": this build's engine has no get_cultures() row for it. | No profile for the culture \"%s\". |
| Faction roster | Not built: a faction emblem (GUI_GAP_REGISTER.md CV-21). The banner is procedural -- a  | Not built: a faction emblem. The banner is drawn from the faction's id and colour -- a  |
| Faction roster | + "port of _civFactionBannerCanvas' own composition, driven by the faction id and its " | + "fixed composition, driven by the faction id and its " |
| Faction roster | + "colour -- and there is no image slot, no charge vocabulary and no asset-library " | + "colour -- and there is no way to attach an image or an asset-library " |
| Faction roster | + "binding for an authored one.").name = "EmblemNotBuilt" | + "piece as an authored emblem.").name = "EmblemNotBuilt" |
| Menus and Help | It arrives with the one shared selection (PC_SHELL_PROPOSAL.md §3.1, milestone PC-1). | This comes later, with one shared selection. |
| Menus and Help | No virtual-tabletop format exists in this build: no grid calibration, wall or light data is computed or written. | There is no virtual-tabletop export: no grid calibration, wall or light data is written. |
| Menus and Help | text += " — not in this build" | text += " — not available" |
| Menus and Help | tip = "This build of the engine has no conflict_campaigns binding." | tip = "Conflict campaigns are not available in this version of Cartalith." |
| Menus and Help | v3's frontmatter mapping and sync direction | the frontmatter mapping and sync direction |
| Menus and Help | "(GUI_GAP_REGISTER.md VA-01) and an Attach-or-create-a-note picker onto any " | "and an Attach-or-create-a-note picker onto any " |
| Menus and Help | templates -- Cartalith ships none of its own (GUI_GAP_REGISTER.md VA-02). | templates -- Cartalith ships none of its own. |
| Menus and Help | so the palm does not cover the card (MAP_CONTEXT_SCOPE.md §7.1). No effect | so the palm does not cover the card. No effect |
| Menus and Help | Ruling EM: what a pen's pressure does on this tool. | What a pen's pressure does on this tool. |
| Menus and Help | until the phone layout reflows below 412 dp (MOBILE_SHELL_PROPOSAL.md MS-2 to MS-6). | until the phone layout reflows below 412 dp. |
| Menus and Help | "Same -- this build predates the multi-GPU API." | "Not available in this version of Cartalith." |
| Menus and Help | The app's own default, 3.4x -- what this build renders with when nothing is stored here. It is not one of the three rungs on purpose -- rounding the shipped default onto the nearest one would change every untouched install's render the moment this preference e | The app's own default, 3.4x, used when nothing is stored here. It is not one of the three choices on purpose, so existing installs render as before. |
| Menus and Help | "No atlas in this build" | "No atlas available" |
| Menus and Help | No recommendation is available for this machine on this build. | No recommendation is available for this machine. |
| Menus and Help | which is not a rail node this build has. | which is not a place the rail can show. |
| Menus and Help | Not built yet: the right dock shows one thing at a time and has no tabs. Inspect, Note and Ask arrive when it becomes the Context panel (PC_SHELL_PROPOSAL.md §3.3, milestone PC-4). | Not built yet: the right dock shows one thing at a time and has no tabs. Inspect, Note and Ask come later, with the Context panel. |
| Menus and Help | Not built yet: the Note tab arrives with milestone PC-5 (PC_SHELL_PROPOSAL.md §3.4). Meanwhile | Not built yet: the Note tab comes later. Meanwhile |
| Menus and Help | Not built yet: the Ask tab arrives with milestone PC-6 (PC_SHELL_PROPOSAL.md §3.5). Meanwhile | Not built yet: the Ask tab comes later. Meanwhile |
| Menus and Help | Pinning arrives with the Context panel (PC_SHELL_PROPOSAL.md §3.3, milestone PC-4). | Pinning comes later, with the Context panel. |
| Menus and Help | It arrives with the Context panel (PC_SHELL_PROPOSAL.md §3.3, milestone PC-4). | This comes later, with the Context panel. |
| Menus and Help | arrives as a tab of the Context panel (PC_SHELL_PROPOSAL.md §3.3, Ruling DY; milestone PC-10). %s | comes later, as a tab of the Context panel. %s |
| Menus and Help | it arrives with one selection (milestone PC-1), and bookmarks are new. | it comes later, with one shared selection, and bookmarks are new. |
| Menus and Help | It arrives with the Context panel (PC_SHELL_PROPOSAL.md §3.3, Ruling DY; milestone PC-4). | This comes later, with the Context panel. |
| Menus and Help | The touch vocabulary this build has: every gesture the map and the sheets answer to, each checked against the code that answers it. | Every touch gesture the map and the sheets answer to. |
| Menus and Help | No in-app documentation exists, and SS2.7 names no URL to open instead. The reference is the repository's own documents (README.md, DECISIONS.md, ARCHITECTURE.md and the scope documents), which are not present beside this build -- an exported build ships res:/ | There is no in-app manual. The documents live in the project folder, which an installed copy does not carry; this row works when Cartalith runs from the project folder. |
| Menus and Help | Opens the repository's documents in the OS file manager: %s. There is no in-app manual, and SS2.7 names no URL -- this is the reference CLAUDE.md itself calls the reading entry point. | Opens the project's documents in your file manager: %s. There is no in-app manual. |
| Menus and Help | Shows you generation info, missing bindings, the project format version, GPU state, the last error this session saw and the tail of the log — each with the symbol it was read from — and writes the ones you keep to a text file. | Shows generation info, the project format version, GPU state, the last error this session saw and the end of the log, and writes the ones you keep to a text file. |
| Menus and Help | this build has no as_set_pack_info() | pack info cannot be saved in this version of Cartalith |
| Menus and Help | _host.set_status("hint", "this build has no visible_grid_rect() — refine needs it", "text_dim") | _host.set_status("hint", "refine is not available in this version of Cartalith", "text_dim") |
| Render workspace | return "%s -- not in this build" % look | return "%s -- not available" % look |
| Render workspace | is not in this build's picker. | is not available in the picker. |
| Civilization workspace | — wall rung not in this build's diagnostics | — wall rung not available |
| Civilization workspace | Harbour — scale not in this build's diagnostics | Harbour — scale not available |
| Diagnostic report | unavailable -- this build predates the multi-GPU API (WorldGen.gpu_enumerate_devices/gpu_set_multi_mode missing). | unavailable -- multi-GPU is not supported in this version of Cartalith. |
| Diagnostic report | unavailable -- this build predates gpu_last_backend(). | unavailable -- the GPU backend of the last run is not reported in this version of Cartalith. |
| Journey planner | jp_options / jp_default_plan / jp_compute / route_count / route_get are not exposed by this build's GDExtension binary -- rebuild cartalith-godot. | The journey planner is not available in this version of Cartalith. |
| Journey planner | jp_options / jp_default_plan / jp_compute / route_count / route_get are not exposed by this build. | The journey planner is not available in this version of Cartalith. |
| Journey planner | jp_reroute is not exposed by this build's GDExtension binary -- rebuild cartalith-godot. | Rerouting is not available in this version of Cartalith. |
| Journey planner | Auto ran but reported no outcome this build understands | Auto ran but reported no outcome Cartalith understands |
| Journey planner | Writes the current party form into a new Travel Library party set-up (TRAVEL_LIBRARY_SPEC.md §3.4). | Writes the current party form into a new Travel Library party set-up. |
| Place editor | ported concept (GUI_GAP_REGISTER.md CV-01);  | ported concept;  |
| Other windows (right_dock.gd) | Authored, in your words. Nothing here decides who wins (STORY_PLANNING_SCOPE.md §5). | Authored, in your words. Nothing here decides who wins. |
| Other windows (right_dock.gd) | so every cell reads bare rock in this build. | so every cell reads bare rock for now. |
| Other windows (colour_paint_tool.gd) | This build's engine has no colour-paint binding (colour_paint_info), so the tool cannot paint. | Colour paint is not available in this version of Cartalith, so the tool cannot paint. |
| Other windows (colour_paint_tool.gd) | Colour paint: no binding. | Colour paint: not available in this version of Cartalith. |
| Data manager | Same -- rebuild cartalith-godot for WorldGen::export_channel_atlas. | Not available in this version of Cartalith. |
| Data manager | Throws the backlink index away and reads the folder again (vault_rebuild_backlinks, then vault_refresh_backlinks). | Throws the backlink index away and reads the folder again. |
| Data manager | Drops this device's binding (vault_disconnect). The links themselves survive | Disconnects this device from the vault folder. The links themselves survive |
| Faction roster | v3's own split: which colour a faction *is* belongs here, how heavily the wash is laid on belongs to CARTO. | Which colour a faction is belongs here; how heavily the wash is laid on belongs to CARTO. |
| Faction roster | Centre the map on it and open its editor -- the reference's own sublist row action. | Centre the map on it and open its editor. |
| Faction roster | This build's engine has no custom-religion binding. Rebuild the native library before treating it as a missing feature. | Custom religions are not available in this version of Cartalith. |
| Engine bridge errors | This build's engine has no custom-religion binding. | Custom religions are not available in this version of Cartalith. |
| Journey planner | This build's GDExtension binary returns no trace -- rebuild cartalith-godot. | No route trace is available in this version of Cartalith. |
| Other windows (open_project_dialog.gd) | Shared projects are not a concept in this port | Shared projects are not available |
| Place editor | "Food shed: — no row for this settlement. civ_food_shed() " | "Food shed: — no row for this settlement. " |
| Place editor | + "returned nothing, which is what an engine build older than ECONOMY_SCOPE.md " | + "The engine returned nothing for it, as an older version of " |
| Place editor | + "milestone 2 does.") | + "Cartalith would.") |
| Place editor | "Smelting: — no row for this settlement. civ_place_smelting() " | "Smelting: — no row for this settlement. " |
| Place editor | "Salt: — no row for this settlement. civ_salt_access() " | "Salt: — no row for this settlement. " |
| Place editor | + "returned nothing, the same defensive case civ_food_shed()'s own reader states " | + "The engine returned nothing for it, as an older version of " |
| Place editor | + "above for an engine build without the binding.") | + "Cartalith would.") |
| Place editor | No trait vocabulary -- the engine build is older than civ_trait_vocabulary(). | No trait list is available in this version of Cartalith. |
| Other windows (settlement_types_window.gd) | No trait vocabulary -- the engine build is older than civ_trait_vocabulary(). | No trait list is available in this version of Cartalith. |
| Other windows (settlement_types_window.gd) | No specialisation vocabulary -- the engine build is older than civ_specialisation_vocabulary(). | No specialisation list is available in this version of Cartalith. |
| Asset library | Tags are added in batch; removing one has no binding yet. | Tags are added in batch; removing one is not available yet. |

## Counts (second pass)

- 117 edit entries across 25 shell files (some replace more than one identical occurrence), plus 39 bridge errors and 1 asset-library line, so 160 or so user-visible strings. By surface: engine bridge 11 entries plus the 39, menus and Help 29, data manager 14, faction roster 9, place editor 9, asset library 8, app status lines 8, journey planner 6, other windows 20.
