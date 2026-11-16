# GUI plans audit, 2026-11-16

**Findings only; a dated cut.** Requested by the owner: *"I have the feeling a lot is already done, verify how old the
current GUI is."* Seven plans, every milestone, each verified at the symbol in the shell `.gd` files or the Rust crates,
not from `STATUS.md`'s say-so. Tree at `1a6800d2` (2026-11-16). No product code was changed. `STATUS.md` stays the
source of truth for progress; this file is evidence and will age.

## The answer

**The owner's feeling is right.** Of 65 milestones across the seven plans (PC 11, MS 11, VE 10, CQ 9, SM 7, CM 7,
AP 10), **56 are DONE, 7 are PARTLY, 1 is OPEN and 1 is SUPERSEDED in part** (the count at the audit; PC-6 moved to DONE the same day, below). What remains is mostly **device passes on
the 6T and the Pad** and a list of named leftovers, not unbuilt milestones.

**How old is the GUI.** `shell/dcc_shell.gd` first landed 2026-08-18 (`2a814163`, "the eight regions, seven menus, five
domains"). Every milestone of these plans landed between **2026-09-26** (CM-1…CM-6) and **2026-11-15** (CQ-5). The PC
shell (PC-1…PC-11) is **2026-11-07 to 2026-11-08**, the mobile shell MS-1…MS-9 and MS-11 is **2026-11-07 to 2026-11-12**,
the vault editor VE-0…VE-9 is **2026-11-08 to 2026-11-09**. `shell/` was touched on 2026-11-16 itself. So the current GUI
is at most nine days old in the parts these plans cover; it is not the 2026-08 shell. The 380 ms hold, 520 dp rule and
other design figures are labelled judgements awaiting glass, not unbuilt features.

**Method and its limits.** For each milestone: read its definition and bar in the plan; read its `STATUS.md` entry;
grep the named symbol in `cartalith-native/godot-project/shell/*.gd` or `cartalith-native/crates/**.rs` and confirm it
exists and is wired (I read call sites for the load-bearing ones, for example `civ_goto_year(` has one writer, the
proposal's PC-2 grep bar, and `on_settlement_selected(` has none outside `select`). Dates and commits are from
`git log --grep` on the milestone ID. I **did not re-run probes** (no Godot session was started); a "DONE" verdict means
the symbol exists and `STATUS.md` records the probe passing, not that I re-ran it. Device column: from `STATUS.md`
evidence only; "owner" means the owner's own report, "windowed" means a desktop-hosted probe with synthetic touches.

Verdicts: **DONE**, **PARTLY** (what is missing is named), **OPEN**, **SUPERSEDED**.

---

## 1. `PC_SHELL_PROPOSAL.md` (PC-1…PC-11): 10 DONE, 1 PARTLY at the audit; 11 DONE after the 2026-11-16 leftover pass (PC-6 except its file slot's Rust registration)

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| PC-1 One selection | DONE | `DccApp.select()`, `selection`, `select_back/forward`, `current_faction()` in `app.gd`; the bar's grep `right_dock_ctrl.on_settlement_selected(` returns 0 | `5ea29148` 2026-11-07 | desktop probe only |
| PC-2 One year | DONE | `DccShell.tl_set_year(year, exact)` in `dcc_shell.gd`; `civ_goto_year(` is called only there (grep) | `bec4f064` 2026-11-07 | desktop only |
| PC-3 Strip at program scope | DONE | `app.gd::_on_workspace_changed` shows the strip in all three domains; `tl_filter`, `tl_record_year`, `tl_go_recorded`. **Update 2026-11-16:** the collapsed line's slider (`TimelineCollapsedSlider`, desktop; the tablet's too since Ruling FO item 5) and one undo step for + Record year (`year_record_history.gd`) are built | `55312a12` 2026-11-08 (`ae60ed86` conflict row) | no |
| PC-4 Context panel: header, pin, Inspect | DONE | `right_dock.gd` (`RightDock`, `show_subject`, `on_selection`); pin and FOLLOWING/PINNED. **Update 2026-11-16:** Inspect's edit mode (`_build_settlement_edit_rows`), roads (`roads_section`) and F7 (`DccApp.conflict_goto_start`) are built; the founding year is not (no engine field, owner question) | `2f2af9e9` 2026-11-08 | desktop only |
| PC-5 Note tab | DONE | `shell/context_note.gd`; the vault window's one editor; `_context_tab_ready()` in `menus.gd`; private line live 2026-11-14 | `1d0bc3e9` 2026-11-08 | desktop/tablet windowed |
| PC-6 Ask tab | **DONE except the file slot (2026-11-16)** | `RightDock.show_tab()`, `LlmAssistantWindow.present_in()`, `shown_block` (Rust `cartalith_llm::prompt`). **Built 2026-11-16** (`_subject_note`, `_toggle_note_chip`, `_SUBJECT_TASKS`, `_mentions_in`, `_follow_citation`, `DccApp.ask_about`, `ContextBroker.ask_rows`, `conv_save` + `ConvSaveCard`, `conversation_document`); **still missing: the world-file slot's registration in `cartalith_io::DOCUMENT_SLOTS` (Rust)**, so the document is built but written nowhere yet | `1f526394` 2026-11-08 | desktop only |
| PC-7 Non-exclusive context windows | DONE | `shell/context_window.gd` (`applies()`, `arrived()`), `context_nav.gd` Back/Forward trail, `popout_host.gd` (`context_module_window` joined `LIVE_KEYS` 2026-11-16) | `de1568cc` 2026-11-08; leftovers 2026-11-14, Ctrl+[ / Ctrl+] 2026-11-15 | desktop only |
| PC-8 Planner in its own window | DONE | `app.gd::planner_in_window()`, `open_journey_planner()`, `planner_window.gd`, `JourneyPlannerView._open_window` | `9d23b027` 2026-11-08 | planner on monitor 2 is owner-run, unmeasured |
| PC-9 Menus and keys | DONE | `menus.gd` (`ID_SHARE_PROJECT` is MS-4's; Records, Generate world in File, `_context_tab_ready`); Escape-deselects and desktop search 2026-11-08; four of five 3-level paths flattened `5c1d178f` | `922c7bb2` 2026-11-07 | n/a (desktop) |
| PC-10 Ten modules | DONE | `context_modules.gd`, `context_module_window.gd`, `DccApp.context_module_registry()` / `context_panel_modules()` / `set_context_module_shown()`; `CTX_MODULES` in `menus.gd` (ten rows) | `8253037b` 2026-11-08 | desktop/tablet windowed |
| PC-11 Tool-options bar holds tool settings only | DONE | `app.gd::_tool_options_generate()` contains 0 `DccWidgets.action(` calls (counted) | `922c7bb2` 2026-11-07 | n/a |

**Remaining leftovers after the 2026-11-16 pass (`STATUS.md` has the evidence):** the world-file slot's Rust registration
(S); the founding year in AT YEAR (needs an engine field; owner); PC-8's route-name disagreement M2 and the window floor
wider than the map at 1 600 px (both owner decisions, not guessed); a module's own key (S); the tablet's collapsed-strip
slider (owner: the touch strip is 34 px). Built 2026-11-16: PC-6's six items, PC-4's edit mode, roads and F7, PC-3's slider
and undo, PC-10's module Pop out, PC-5's Note Pop out and note citations.

## 2. `MOBILE_SHELL_PROPOSAL.md` (MS-1…MS-11): 8 DONE, 2 PARTLY, 1 OPEN

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| MS-1 dp classes, live re-class, UI scale | DONE (code) | `DccTheme.classify()`, `width_class_of()`, `DccShell.quiescent_blockers()`, `DccApp._remount_composition()`, `represent_capture()` in 28 windows. S0 6T density measured. **Open: B7, B9's export half, B10's other probes, S4's device rows.** The 600 dp width line is SUPERSEDED by Ruling EG (900 dp shortest side) | S1 `ecd023ab` 2026-11-07 … S4.6 `73605b4c` 2026-11-12 | 6T: density (`ae802d3b`) and an adb swipe smoke; Pad 3 booted the phone GUI by a stored 200 % UI scale, trap fixed 2026-11-09 |
| MS-2 Bottom bar, domain tab | DONE | `DccShell.PHONE_TABS`, `DOMAIN_TAB`, `_pick_phone_tab`, `phone_domain_fan.gd`; three dots added 2026-11-13 | `d0045b68` 2026-11-08 | 6T: owner report 2026-11-13 fixed dots and switches; timings (380 ms hold, 48-unit swipe) await MS-10 |
| MS-3 Domain sheet from dock builders | **PARTLY** | `_mount_dock_in_sheet()`, WORLD ALL segment, CIVIL > Journeys row, `MapBody`. **Missing:** node chips and tool palette atop the domain sheet; MAP still selects CARTOGRAPHY; North up, grid, scale-bar toggles; the bar "count 0 reachable only through Window ▸ Left dock" is recorded for CIVIL/CARTO/WORLD but not as a full walk | `2d029770`, `82234124` 2026-11-08; MAP body 2026-11-13 | 6T: owner report 2026-11-13 |
| MS-4 MENU mirrors the eight menus | DONE | `phone_menu.gd::_fill_root`, `_fill_popup`; `_ms4_probe` green again 2026-11-14 | `9bae6952` 2026-11-08 | desktop-dumped menus compared; 6T walk partial |
| MS-5 Tool strip, fan, context card, map control row | DONE | `tool_strip.gd`, `tool_fan.gd`, `dcc_shell.gd` control row; only the card's `param` rows on the phone are unbuilt (optional) | `a2200e5b` 2026-11-08, `c302a4ed`, `8fcab5fc`, `e6f3cfbd` 2026-11-09 | windowed only: "no Android build" in all four steps |
| MS-6 STORY tab | DONE | STORY sheet of four segments over the shown band's rows (`3b04e444`); `phone_menu.gd::open_story`, `_fill_story_rows`. Left: a year in TIME's subtitle, live `journey A to B` | `3b04e444` 2026-11-09 | windowed |
| MS-7 Tablet docking, collapse, readouts | DONE (2026-11-16) | `DccTheme.tablet_dock_plan()`, `tablet_bar_whole()`, `tablet_readouts_shown()`, `_set_dock_floating()`, `_left_pin`/`_right_pin`, `_apply_tablet_readouts()`; `_ms7_probe.gd`. The pin and the readouts are built | `263a7b26` 2026-11-09 | Pad 3 sizes windowed; no glass pass |
| MS-8 New World AUTO-POPULATE | DONE | `auto_populate.gd`, `new_world_dialog.gd`; switches remove what Generate made when off (2026-11-13). Open edge: generated label list view-dependent (OUTSTANDING row) | `99566ee5` 2026-11-12 | **6T, owner: "seems to work perfectly"** (2026-11-12) |
| MS-9 Search runs every command | DONE | `CommandIndex.run_search`, `search_block`, `row_state`; `DccShell._run_search_command` | `763448c3` 2026-11-12 | 6T: phone search timing recorded (`ad613410`) |
| MS-10 Device passes: 6T, tablet, 4K window | **OPEN** | none by definition (device work). Partial 6T evidence is listed on MS-1, 2, 3, 8, 9 only | n/a | not done |
| MS-11 Phone quick drawer | DONE | `quick_drawer.gd` (`setup`, `_set_open`, `_on_grip_input`), `quick_bar_phone.gd::drawer()`; an OUTSTANDING note of the same day saying "no drawer symbol" is stale | `eef64483` 2026-11-12 | windowed (`_ms11_probe` 103/103 at the 6T size); glass pass pending |

## 3. `VAULT_EDITOR_PROPOSAL.md` (VE-0…VE-9): 8 DONE, 2 PARTLY

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| VE-0 Measure on 4.7: highlighter, hover under touch, Android folder access on the 6T | **PARTLY** | `_ve0_measure_probe.gd` answered the Godot questions (colour only for `SyntaxHighlighter`); **the 6T's folder access was not measurable** | `bec744b5` 2026-11-08 | **no** |
| VE-1 Links follow, return, peek | DONE | `meta_clicked` in `markdown_editor.gd` and `vault_window.gd` | `bec744b5` 2026-11-08 | no |
| VE-2 `[[` suggestions, auto-pair | DONE | suggestion picker in `markdown_editor.gd`; Rust `notelinks` suggestions | `5b7f7947` 2026-11-08 | no |
| VE-3 Write colouring, Split, smart keys | DONE | `markdown_highlighter.gd`; Split from 760 dp | `2dd5159d` 2026-11-08 | no |
| VE-4 Autosave, conflict banner, session undo, Revert | DONE | "Revert to opened" in `markdown_editor.gd`/`vault_window.gd`; `vault_compare.gd`. SAF write atomicity unmeasured | `875b8542` 2026-11-08; Compare `3ad51623` 2026-11-13 | no (SAF side open) |
| VE-5 Backlinks, mentions, outline, properties, tags | DONE | `vault_side_panel.gd`; `LIST_CAP` routes | `996cb307` 2026-11-08 | no |
| VE-6 Rename with link rewriting, create-from-link | DONE | `vault_window.gd::request_rename_of`; Rust `plan_rename` in `cartalith-vault`; SAF `move_file` (`vault_saf.rs`, off-device) | `7e565d1d` 2026-11-08; SAF move `75e8947e` | no |
| VE-7 Tables, images, embeds, tasks, chronos, callouts | DONE | Preview in `markdown_editor.gd` (chronos timeline, callouts) | `7d4c0761` 2026-11-08 | no |
| VE-8 Phone keyboard row, Open ›, Done | **PARTLY** | built (keyboard row, `Open ›`, Done, long-press variants); **the bar (row above the keyboard on the 6T, both orientations) was never run on glass** | `d91f9a5e` 2026-11-08 | **no** |
| VE-9 Landmark notes, notes scope in Find, private badge, caps | DONE | `PlaceSearch._add_landmarks`, `vault_privacy.gd`, `_sync_private_notice`; caps routed to `LIST_CAP` (the bar's `grep 2000` was closed 2026-11-13) | `923cb657` 2026-11-08, `df36f8d1` 2026-11-09 | no |

Leftovers: a drawn chronos track (M, no specification; the browse preview's embeds built 2026-11-16; pictures by bare name and Android pictures built 2026-11-16, the Android read untested on a device);
VE-5 end-of-Preview copy (built 2026-11-16, with the tablet form); VE-6 a fuller template chooser (S); probes that call the 17 new
`vault_*` functions (S); the 6T rows for VE-0, VE-4 (SAF) and VE-8 (S, owner-run, needs `adb run-as` or a connected
folder, which `vault_connect_saf` does not yet give).

## 4. `CUSTOM_QUICK_BAR_SCOPE.md` (CQ-0…CQ-8): 7 DONE, 1 PARTLY, 1 SUPERSEDED (in part)

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| CQ-0 Pin identity | DONE | `CommandIndex` `pin_key`, `pinnable`, `why_not` | `5012914f` 2026-10-25 | n/a |
| CQ-1 Store | DONE | `DccSettings` `[quick_bar]` (`_clean_pins`, `_scan_pins`) | `3b28d2a8` 2026-10-25 | n/a |
| CQ-2 Desktop cluster | DONE | `quick_bar.gd`, `app.gd::_install_tool_palette_bar` | `3b28d2a8` 2026-10-25 | n/a |
| CQ-3 Customise window | DONE | `quick_bar_window.gd`, `quick_bar_menu_pin.gd` | `c5da8d82` 2026-10-25 | n/a |
| CQ-4 Ring dock and shared sub-rings | DONE | `quick_bar_ring.gd`, `context_broker.gd::ring_collect` | `80bae4f9` 2026-10-26 | n/a |
| CQ-5 Tablet | **PARTLY** | tablet form built and probed windowed (`_quickbar_tablet_probe`); **hold, slide into a dock chip, lift; 44 dp on a thumb; card beside the ring in portrait are not on glass** | `58c180da` 2026-11-15 | **windowed only**; owner's Pad check pending |
| CQ-6 Phone | SUPERSEDED (in part) | the MAP-sheet chip row was replaced by MS-11's drawer (Ruling DY Q10-Q12); `All…` sheet, MORE ▸ Quick, search star and the Customise phone layout stand; long-press-to-reorder was never built; the 6T bars not run | `a2e680cd` 2026-10-26; replaced by `eef64483` 2026-11-12 | no |
| CQ-7 Assistant chip | DONE | `assistant_chip.gd::AssistantChip.decide` | `31a5f6e9` 2026-10-26 | n/a |
| CQ-8 Per-domain visibility, toggle pins, tool pins | DONE | `DccSettings._domains_valid`, `_domains_sorted`; `_cq8_probe` | `23045a8c` 2026-11-08, `f2a1e9ad` 2026-11-09 | n/a |

Deferred inside CQ-8 by its own text, each needing an owner answer: layout export/import; replacing a diagonal; pinning
a domain's own tools (M each, unscheduled).

## 5. `STORY_MENUS_PROPOSAL.md` (SM-1…SM-7): 7 DONE

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| SM-1 Data to World with bands, Places, Timeline rows | DONE | `menus.gd` Records ▸ RECORDS ▸ Places…, Timeline (the World menu was renamed **Records** later by Ruling DY PQ6, `922c7bb2`) | `95f28d90` 2026-10-18 | no |
| SM-2 Story menu | DONE | `ID_STORY_TIMELINE` 723, `ID_STORY_POLITICAL` 724, `ID_STORY_JOURNEY_PLANNER` in `menus.gd` | `e4c5e252` 2026-10-18 | no |
| SM-3 File ▸ Export ▸ with Player / DM presets | DONE | `export_dialog.gd` presets (`"player": {"title": "Player map"…}`) | `3f27e383` 2026-10-18 | no |
| SM-4 Context-card story verbs | DONE | `civilization_workspace.gd` `STORY_CHIP_JOURNEY`, card rows *Add event to timeline here*, *Plan journey from/to here* | `81c8e818` 2026-10-18 | no |
| SM-5 Places without notes | DONE | `app.gd`/`menus.gd`/`vault_window.gd` `places_without_notes` (now a signpost, PC-9) | `56671218` 2026-10-18 | no |
| SM-6 Saved journeys and Conflicts windows | DONE | `saved_journeys_window.gd`, `conflicts_window.gd` | `56671218` 2026-10-18 | no |
| SM-7 Session mode | DONE | `menus.gd` Window ▸ Session mode (`_session_gate_row`) | `fb6127f8` 2026-10-18 | no |

The proposal's §8 later rows (a per-item DM-only mark, free timeline events, print and VTT writers, CIVIL ▸ Settlements)
stay unscheduled by Ruling EA item 7 ("None for now"); they are not milestones.

## 6. `MAP_CONTEXT_SCOPE.md` (CM-1…CM-7 and forks F1-F5): 6 DONE, 1 PARTLY

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| CM-1 Request and broker | DONE | `context_broker.gd`, `context_requested` | `881ca87d` 2026-09-26 | n/a |
| CM-2 Context card | DONE | `context_card.gd`; Menu key and Shift+F10 `1ca95c54` | `a5e44b34` 2026-09-26 | n/a |
| CM-3 Desktop ring | DONE | `radial_ring.gd` | `0ea4c2d1` 2026-09-26 | n/a |
| CM-4 Tablet hold | **PARTLY** | hold, slide-to-select, handedness built; **the bar's "pen barrel button verified on the owner's device" is open**, and research since says the Stylo 2 has none | `564270d7` 2026-09-26 | **no** |
| CM-5 Phone pin and peek | DONE | long-press `ba015748` (480 ms, F1 = Ruling AX); long-press fixed on the 6T 2026-11-15 | `ba015748` 2026-09-26 | **6T: reproduced and fixed 2026-11-15** |
| CM-6 Phone thumb fan | DONE | `tool_fan.gd` | `de2d4daa` 2026-09-27 | no |
| CM-7 New picks and verbs | DONE | landmark, route, stamp, way, river rows | `45c87339`, `885601d2`, `3bfbe0f4` 2026-09-27 | n/a |

Forks: F1 (480 ms, drop pin and open verbs), F2 (build the fan), F3 (global cardinals), F4 (one *Settlement actions in
CIVIL ›* row, `cartography_workspace.gd`) are answered by Ruling AX and built. F5 was "later"; Ruling DH Q3 lifted it
for the ring dock and shared sub-rings (CQ-4, built); **the tablet edge sliders (§7.3) and per-slot ring customisation
beyond CQ-4's tab remain unbuilt and unscheduled** (M).

## 7. `ASSET_PACK_TOOLS_PROPOSAL.md` (AP-1…AP-10): 10 DONE (two with recorded bar misses)

| Milestone | Verdict | Evidence symbol | Landed | Device? |
|---|---|---|---|---|
| AP-1 Open a pack into the library | DONE | `as_import_pack` (`engine_bridge.gd`, `asset_library_window.gd`) | `3c0c32dd` 2026-10-18 | no |
| AP-2 Pack report | DONE | `cartalith-assets/src/pack_report.rs::pack_report` | `ea23ed45` 2026-10-18 | no |
| AP-3 Settlement/POI/sea-mark/custom art on the map | DONE | trait-badge sprite path | `b1abc0a5` 2026-10-19 | no |
| AP-4 Seam checker, make-seamless | DONE | `cartalith-assets/src/seamless.rs` | `b1abc0a5` 2026-10-19 | no |
| AP-5 Feature icons at screen resolution | DONE, bars (b) and (c) missed | `set_feature_icon_source`, `feature_icon_draw_list`, `PACK_FEATURE_ICONS := true`, `composite_placed_features`. Line art meets ≤ 1 px; painted sprites miss ≤ 2 px (3.16 at ×16) and the 1 % ring bar (8.1 %); **owner accepted faithful (Ruling CS item 1)** | `7837c1a5` 2026-10-19 | no |
| AP-6 Near-duplicate finder | DONE | `cartalith-assets/src/near_dupes.rs` | `88d9f4a2` 2026-10-19 | no |
| AP-7 Refine verbs | DONE | `refine.rs` (`harmonise` …), `asset_library_window.gd::_refine_*` | `88d9f4a2` 2026-10-19 | no |
| AP-8 Folder import adapters | DONE | `asset_bridge.rs::import_folder` | `2abc824d` 2026-10-19 | **Android untested** |
| AP-9 Diff, merge, favourites, recent | DONE | `diff_pack`, `merge_pack`; favourites in `asset_library_window.gd`/`dcc_settings.gd` | `2abc824d` 2026-10-19 | no |
| AP-10 Line-art conversion | DONE, one recorded 0.001 IoU miss | `cartalith-assets/src/lineart.rs` | `e62ce7ec` 2026-10-19 (approved `66afcd65`) | no; owner scans untested |

Leftovers carried from `OUTSTANDING_WORK.md` (not re-opened here beyond the symbols above): ground candidates matching
no slot wait in a custom set with no move-to-slot verb (S); `.wonderdraft_symbols` unparsed, format unverified (S);
`pack_report` / `near_dupes` / `lineart::working_image` bake straight (S); `tests/pack_compositing.rs`'s "forces `shrub`"
claim is false (S); Android folder import on glass (S, device).

---

## Summary of the remaining work

| Plan | Done / partly / open / superseded | Open items (size) |
|---|---|---|
| PC shell | 11 / 0 / 0 / 0 (2026-11-16) | the world-file slot's Rust registration (**S**); the founding year (Ruling FO item 1, a Rust change). Built 2026-11-16 by Ruling FO: the one route name (M2) and the tablet's collapsed slider; the planner window floor was accepted |
| Mobile shell | 8 / 2 / 1 / 0 | **MS-10 device passes** on the 6T, a tablet and a 4K window (**M**, owner-run); MS-3 node chips + tool palette atop the domain sheet and MAP's own tool domain (**S-M**, NOT built 2026-11-16: the proposal gives only a sentence, "the node's chips, the domain's tool palette, then every category", and the dock's accordion headers already are the chips; what is undecided is whether a second chip strip duplicates them, and which tools the palette holds per domain, the rail having tools on desktop only; an owner design call); MS-1 B7/B9/B10 device and probe rows (**S**); delete the unreachable `QuickBarPhoneList` sheet (**S**); MS-5 card `param` rows, only if the owner wants them (**S**) |
| Vault editor | 8 / 2 / 0 / 0 | the 6T rows for VE-0, VE-4 (SAF) and VE-8 (**S**, owner-run); the 6T pass for VE-7's Android picture read (built 2026-11-16, untested on a device; **S**), drawn chronos track (**M**, no specification); 17-function probes (**S**). Built 2026-11-16: VE-5's end-of-Preview copy and the browse preview's embeds |
| Custom quick bar | 7 / 1 / 0 / 1 | CQ-5 Pad pass (**S**, owner-run); CQ-6 long-press reorder and 6T bars (**S**); CQ-8's three deferrals, each needing an owner answer (**M**); ring parts overhanging a dock panel not clickable, phone tool cell 3.5-5 px high (**S**) |
| Story menus | 7 / 0 / 0 / 0 | nothing scheduled; §8 later rows unscheduled by Ruling EA item 7 (each **M**, blocked on a ruling) |
| Map context | 6 / 1 / 0 / 0 | CM-4 device check of the pen barrel button and haptics (**S**, owner-run); F5's tablet edge sliders and further ring customisation (**M**, unscheduled) |
| Asset pack tools | 10 / 0 / 0 / 0 | the five small leftovers above (**S** each), Android folder import on glass (**S**, device) |

**By size.** Genuinely unbuilt design work: about **3 M** items (VE-7's remaining edges,
CQ-8 deferrals, F5 sliders) plus MS-3's S-M. **Device-bound work** (owner on the 6T or Pad, no code): MS-10, CQ-5, CM-4,
VE-0/4/8, AP-8, MS-1 B7. Everything else is S.

## Findings: what the documents got wrong

1. `OUTSTANDING_WORK.md`, the *Implement the mobile menus* row, still said MS-11's quick drawer was "not built (no
   drawer symbol in `quick_bar_phone*.gd`)". `shell/quick_drawer.gd` exists and `eef64483` (2026-11-12) built it.
   Corrected.
2. `OUTSTANDING_WORK.md`, the *Custom quick bar* row, ended "Next: CQ-5 (tablet touch and pen)". CQ-5 was built
   2026-11-15 (`58c180da`). Corrected.
3. `OUTSTANDING_WORK.md`, the *Asset pack refinement tools* row, carried the history of the original "Nothing is built"
   text inside a very long cell; it already says AP-1…AP-10 are built. Left, with a one-line pointer to this audit.
4. `STATUS.md`'s 2026-10-31 *Mobile shell proposal* entry says "MS-2…MS-10 are unscheduled", and its *MS-1: being built*
   paragraph says "S4.5 and S4.6 not built". Both were true on their dates and are false now; correction notes added
   in place. Its *quick bar and the cartwheel* entry ("nothing built") is likewise dated and was left as history.
5. No milestone listed as built in `STATUS.md` was found missing from the code. The reverse (built but listed open)
   happened twice, items 1 and 2.

## Post-audit changes the same day (added 2026-11-16 by `DOCS_AUDIT_2026-11-16.md`)

This audit is a dated cut; these landed or were ruled after it and contradict rows above, which are left as written:

- **CQ-6** (section 4 table, "long-press-to-reorder was never built"; the Custom quick bar row of "Summary of the remaining work"): the long-press
  reorder was built in `3f7ea940` (`_cq6reorder_probe.gd`, `DccSettings.quick_bar_move_visible_to`).
- **VE-6** "a fuller template chooser (S)": built in `3f7ea940` (`_ve6tpl_probe.gd`). **VE-5** tablet form: built in
  `8dd94baf`; the end-of-Preview copy and the 17-function probes remain.
- **MS-3** "NOT built ... needs an owner design call": the call was made by Ruling FL item 1 (a tool palette only, no node
  chips); **built 2026-11-16** (see the next entry).
- **CQ-8**'s three deferrals "each needing an owner answer": answered by Ruling FL item 2 (all three wanted) and **built the same day**.
- **MS-3** (Ruling FL item 1) **built 2026-11-16**: the phone domain sheet's tool palette (`DccApp._sync_phone_tool_palette`, from the one `_domain_tools` registry; no node chips). The MS-3 row, the Mobile shell row and the CQ-8 rows above read as written; `STATUS.md`, "Ruling FL items 1 and 2 built", is the answer.
- **Ruling FO items 2 and 5, VE-5's end-of-Preview copy and VE-7's browse-preview embeds** **built 2026-11-16**: the PC shell row's route name (M2) and tablet slider, the Vault editor row's copy and browse embeds. The rows above read as written; `STATUS.md`, "Ruling FO items 2 and 5", is the answer. VE-7's other two edges (bare-name and Android pictures) were built later the same day in the Rust provider (`STATUS.md`, "VE-7 picture edges and the conversation slot"); the Android read is untested on a device. The drawn chronos track stays open and unspecified.
