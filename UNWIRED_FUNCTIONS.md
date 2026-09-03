# Presented but not wired — the standing table

> **Relationship to `cartalith-native/docs/STATUS.md`** (added 2026-08-31)
>
> **This file answers a question `STATUS.md` does not ask: "the product draws
> this control — is there anything behind it?"** It is per-control, not per
> milestone. Each row carries a `file:line` that was opened, the reason a user
> is shown, and a proposal sourced from the spec or from an existing engine
> binding. That granularity is the point, and it does not belong in a milestone
> ledger.
>
> **`STATUS.md` is authoritative for status.** This table's counts are a *cut*,
> dated and reproducible, not a live figure. If a row here says a binding is
> missing and the code says otherwise, **the code wins and the row is a
> defect** — that is this document's own primary failure mode, twice proven
> against itself (44 rows/12 wrong on the first re-cut, then two more rows
> the 2026-08-31 cut closed in-audit while still printing them as open).
>
> Nothing here is a schedule. What is *chosen* for the 18 Large rows is in
> `LARGE_ITEM_RULINGS.md`; what is *queued* is in `OUTSTANDING_WORK.md`, which
> deliberately counts this whole document as one row so the two cannot drift.

Owner, 2026-08-30: *"Make sure all menu's are created and all presented
functions are wired or the ones that have no code behind them get listed in a
table with a proper proposal (inferred from the menu name and highest probable
explanation/design spec of the named function.)"*

## This cut — 2026-09-01, against the uncommitted working tree

**Nothing described as closed below is committed.** `git diff --shortstat`
against `HEAD` at the time of this cut reads **124 files changed, 15 957
insertions(+), 10 481 deletions(-)** — the same uncommitted working tree the
2026-08-31 cut was written against, grown further. Every symbol this cut
cites was opened fresh today; none of the 2026-08-31 cut's file:line pairs
were trusted forward.

**Correction, 2026-09-02: no longer true.** That tree is committed now —
`4ec07f5` (urban milestones 8-15, the Vulkan/DX12 answer, `_civPlaceSmelting`/
`_civSaltAccess`) and `cff1edc` (crater/volcano/hillslope physics, urban
milestone 16-17, the Vault SAF provider, four right-dock contexts,
`_civPlaceTrade`, `deny.toml`) — so every row this document calls closed
above is real history on `main`'s ancestry, not staged work. `git status` at
this cut shows one uncommitted file, `crates/cartalith-civ/src/landmark.rs`
(+252/-16): a different lane's in-flight work threading the routed way
network into five new landmark kinds
(`road_junction`/`bridge_site`/`market_site`/`caravan_station`/`trade_depot`).
Checked and excluded: none of those five names, nor anything else in that
diff, appears in any row of this document.

**The headline change: 75 → 23 open rows, then 22 after a same-day second
pass, then 21 after a same-day third pass — and still 21 after a fourth pass
the next day (2026-09-02), which closed nothing but corrected two rows and
found one new dangerous-class entry.** The 2026-08-31 cut's own count —
stated honestly in its own text — was 75, not the 77 its header printed,
because two rows (State religion, `_refresh_phone_bar_lit()`) were already
closed in-audit when that cut was written. Re-opening every one of those 75
rows against today's tree:

| Tier | Rows | Open at the 2026-08-31 cut | Newly closed this cut | Open now |
|---|---:|---:|---:|---:|
| Trivial | 17 | 16 (State religion already closed) | **16** | **0** |
| Small | 25 | 24 (`_refresh_phone_bar_lit()` already closed) | **23** | **1** |
| Medium | 17 | 17 | **14** | **3** |
| Large | 18 | 18 | **1 fully** (1 half-built) | **17** |
| **Total** | **77** | **75** | **54** | **21** |

> **Second pass, same day (2026-09-01).** The table above already reflects
> it: **Paint brush falloff** — the single row this whole cut named as
> highest-severity — was independently re-verified against the code after
> landing (`paint.rs`, `paint_bridge.rs`, `world_workspace.gd`, `tool_bar.gd`,
> `DECISIONS.md` §7k all opened fresh; `cargo test -p cartalith-spatial --lib`
> 148/148, `--test golden_parity_paint` 7/7, `cargo test -p cartalith-godot
> --lib` 409/409 after a fresh `cargo build -p cartalith-godot` — the dll was
> stale against exactly this row's own files, this project's own recorded
> hazard, caught rather than repeated) and moved from open to closed below.
>
> **Third pass, same day (2026-09-01).** One more Medium row closed:
> "Manual road tool / `road_edges` never retained" — independently
> re-verified against the code (`cartalith-godot/src/lib.rs:188,1434`,
> `crates/cartalith-civ/src/lib.rs:11778`, `journey_bridge.rs`'s module doc)
> as part of reconciling `OUTSTANDING_WORK.md` §2.3's journey/route cluster;
> see the Closed table below for the evidence. The table above reflects it;
> the "Open now" column elsewhere is the first cut's, unchanged.
>
> **Fourth pass, next day (2026-09-02), against HEAD `cff1edc`.** Re-opened
> all 21 open rows plus the dangerous-class entry at their cited symbols,
> against a tree where the previous cut's own giant diff is now committed
> history (see the correction above). **Zero rows closed outright — the
> count holds at 21 (1/3/17).** Two things changed: **(1)** the right-dock
> Medium row is narrower than stated — `cff1edc` landed four more
> tool-arm-driven dock contexts (paint/stops/anno/territory), so most tools
> now push dock content on arming by hand-written precedent, just not
> through a general mechanism; see that row and Owner question 1, both
> updated below. **(2)** the Settlement diagnostics overlay Large row's own
> tooltip went from true to false in exactly the way §b is built to catch:
> it still cites "urban milestones 9, 10 and 13" as its blocker, and all
> three shipped in `4ec07f5`. **1 new dangerous-class row, not carried
> forward from any prior cut** — see the dangerous class section and its
> own Large-table entry, both updated below. Several `file:line` citations
> across every tier had drifted (a moved binding at `engine_bridge.gd:2554`,
> not `:2486`; five workspace scripts now live under `shell/workspaces/`
> rather than `shell/` — cited here by basename only, per this document's
> own convention, so that move costs nothing) and are refreshed in place
> without a separate log; none changed a row's substance.

Every trivial and small row is closed except one (`label_glyph_layout`, still
partly open — see below); every medium row is closed except three, each with
a stated, verified reason; of the eighteen large rows, seventeen remain
exactly what `LARGE_ITEM_RULINGS.md` says they are — **ruled on, not built**
— with one partial exception (the landmark funnel's crowding parameter, which
is live), and **one, Paint brush falloff, is now fully built and verified.**

**The dangerous class shrank from 25 entries to 3, then to 1.** All 9
false-reason rows (§b) are now true. All 9 misleading-presentation rows (§c)
are now honestly presented. Of the 7 enabled-and-inert rows (§a), 4 were
fixed outright in the first cut, 1 was already a deliberately-disclosed
non-defect kept for contrast, and the remaining 2 — both symptoms of Paint
brush falloff — closed in the same-day second pass, verified against the
code rather than the report. **0 genuinely dangerous rows remain.**

**Addendum, 2026-09-02: no longer current.** Re-verifying all 21 open rows
against `cff1edc` found one new false reason that did not exist at the
2026-09-01 cut, because the milestones it now falsely blames had not landed
yet at that point. **1 genuinely dangerous row stands, not 0** — see the
dangerous class section below.

**Method.** Every row below was re-opened at its cited symbol, not its old
line number — line numbers had already drifted inside single days on this
tree before this cut started (see `right_dock.gd`'s own three-times-wrong
`get_factions()` citation, corrected and disclosed in the code itself,
`right_dock.gd:999-1017`). Closed rows are logged compactly, with the
evidence that closed them; open rows keep the full proposal.

---

## Trivial — 0 open (17 of 17 closed)

Every trivial row from the 2026-08-31 cut is fixed in the working tree.

| Item | Where (current) | How it closed |
|---|---|---|
| State religion reads `—` | `right_dock.gd:1095-1103` | Already closed pre-2026-08-31; still correct — reads `roster.get("religion", "")`, prints `"none"` as a real answer rather than dashing. |
| Auto-populate world tooltip cites a stale param count | `civilization_workspace.gd:1362-1364` | Tooltip rewritten; no entry count is stated at all any more, so there is nothing left to go stale. |
| Faction comment cites `get_factions()` at a stale line | `right_dock.gd:999-1017` | Now `lib.rs:6559` (verified), with the citation's own history of drift (3442 → 6225 → 6295) disclosed in the comment itself. |
| "No landmark types" — unreachable dead branch, false reason | `menus.gd:1109-1127` | Reworded to a true reason (a build predating `landmark_kinds()`), per the proposal's own fallback option. |
| "Not available for this world" on permanently-unavailable layers | `layers_popover.gd:383-394` | Split into two sentences: `GAP_LAYERS` rows get a per-id reason from the engine's own table; the seven conditional refusals keep "for this world". |
| `layers_popover.gd` "eleven permanent engine gaps" | `layers_popover.gd:114-118` | Now reads "Eleven ... on two different grounds. **Four** are the permanent engine gaps ... other **seven** ...". |
| WORLD gap note: tides "no cartalith-engine equivalent yet" | `world_workspace.gd:79-84` | Split: geoid stays true (still nothing); tides corrected ("ported and live, just not under this stage"). |
| First-run coach mark names a bar that no longer exists | `dcc_shell.gd:5579` | Rewritten against `PHONE_TABS`: "MAP · GENERATE · PLAN switch tasks here — MORE reaches everything else." |
| `--hero`/`--hero2` tokens unconsumed | `dcc_theme.gd:1162,1170`; `right_dock.gd:1910-1918` | `DccTheme.hero()`/`hero2()` now read `role_px("fs_hero"/"fs_hero_2")`; the elevation readout routes through `DccTheme.hero()`. |
| `--popW` token unconsumed | `layers_popover.gd:182-203,316-318` | Consumed at every non-phone density via `role_px("w_popover")`; phone deliberately sizes to the full-screen sheet instead (disclosed, not a gap). |
| `pack.rs` declines painted layers citing a producer that now exists | `pack.rs:1-72` | Module doc rewritten in full, naming `PaintEditor`/`get_paint_layers`/`with_paint` and stating precisely what's still missing (decoding `biomes`/`terrains` pixels — a real, separate, correctly-deferred job, never itself a numbered row). |
| Saved look picker's `choice()` is a no-op | `render_workspace.gd:859-861` | Tooltip now names the destination: "Picking one only selects it -- press Load look below to apply it." |
| Icon-placement fill line reads as live data | `cartography_workspace.gd:1401-1402` | Tagged "(design figures)" in the string itself, as proposed. |
| No project format version anywhere in the UI | `gen_info_dialog.gd:98-103`; `open_project_dialog.gd:754-802` | Printed in Generation info… (build's own version) and on the Open dialog's tile caption (the *save's* own version — the more useful of the two, with a comment explaining why they must differ). |
| Missing-binding fingerprint has no readout | `gen_info_dialog.gd:104-109` | One line: `Bindings missing: none` or the list. |
| Disabled radio rows lose their radio mark on phone | `phone_menu.gd:848-860` | Mark drawn first, then dimmed — radio-only, so a `_switch()` pill still reads as inert rather than operable. |
| Landscape sheet handle drawn and inert | `dcc_shell.gd:6023-6028` | Hidden outright in landscape rather than left visible-and-dead; coach mark #2 is skipped there too. |

---

## Small — 1 open (24 of 25 closed)

### Open

| Item | Where (`file:line`) | Current state | Size |
|---|---|---|---|
| `label_glyph_layout` is re-implemented in GDScript | `map_overlay.gd:450-492`; binding `engine_bridge.gd:2554` (drifted from `:2486`, re-verified 2026-09-02; `map_overlay.gd`'s own citation is unmoved) | **Partly closed, and re-scoped honestly rather than left stale.** The minimum the 2026-08-31 proposal allowed for — pin the three constants with a comment naming `labels.rs` — is done: `ARC_STRAIGHT_THRESHOLD`, the `1.2` radius floor and the `2.2` spread divisor are now named constants, checked against `labels.rs:150,176`. The full fix (call the binding once per label) is still not done, and the comment explains why in more depth than before: `map_overlay.gd` has no bridge handle to call through (a `viewport_host.gd` decision, not this file's), **and** a second, previously-unstated reason — this file's own px-per-cell font-sizing model differs from the engine's `label_font_size`, so swapping the loop for the binding would silently reshape every arched label, not just relocate arithmetic. **New, named 2026-09-01:** this unreconciled pair already costs something observable — `label_box_at`/`label_handles` (the resize/rotate/arc drag handles `cartography_workspace.gd` hit-tests) size themselves off the engine's `label_font_size`, while the glyph the user sees is sized by this file's own model. The fix direction is named (feed this file's px-per-cell into `LabelViewEnv` so the two models produce the same number, then delete the local copy) but is explicitly left undone rather than half-done. | small |

### Closed

| Item | Where (current) | How it closed |
|---|---|---|
| Biome-K toggle never re-read from the engine | `new_world_dialog.gd:253-254,504-506` | CheckBox retained in `biome_k_check`; `_sync_from_engine()` (on `about_to_popup`) reads `get_biome_k_enabled()` and `set_pressed_no_signal`s it. |
| Theme choice does not survive a relaunch | `dcc_settings.gd:361-372`; `menus.gd:315,328,3148` | `DccSettings.theme_mode()`/`set_theme_mode()` added; restored at boot, written on every change. |
| `Window ▸ Status bar` no-op on phone | `app.gd:2727,2753-2755`; `dcc_shell.gd:2117-2122` | Phone branch routes through `set_status_region_shown()`/`is_status_region_shown()` instead of a hidden node's `visible`. |
| `Window ▸ Left dock`/`Right dock`/`Reset layout` desync phone sheets | `app.gd:2727-2755` | All three phone-routed through `_set_sheet_open()`; `Reset layout` on phone now closes both sheets instead of showing both. |
| `Window ▸ Domain rail` is a one-way door on phone | `dcc_shell.gd:3831-3841,3888` | `_rail_region` now points at `_phone_bar_dests` — the three domain cells only; MORE stays reachable. |
| `_refresh_phone_bar_lit()` has never run | `dcc_shell.gd` | Already deleted pre-2026-08-31 (function, both dead fields, all six call sites) — confirmed still gone; `_refresh_phone_tabs()` is the one live path. |
| Selecting CIVIL leaves MORE lit | `dcc_shell.gd:2555-2568` | `_select_domain()` now calls `_refresh_phone_tabs()` at its one choke point. |
| `CommandIndex` indexes readouts as unavailable commands | `command_index.gd:51-61,153-189` | Third state added: `kind: "readout"`, `available: true`. Only `_todo()` mints an unavailable command now. |
| No per-slider "reset to default" | `world_workspace.gd:1063-1112,1246-1252` | `param_default(key)` now backs both `ERODE_DEFAULTS`' seeding and a per-row reset. |
| Stage group names hardcoded with no check | `world_workspace.gd:264-290` | `_assert_stage_groups()` checks every `STAGES` group against `param_groups()` before the dock is built. |
| No `Reset generation parameters` command | `menus.gd:747,813-841,888,935-937` | `Edit ▸ Reset generation parameters` (all) and `Edit ▸ Reset one stage` (the `keys` overload) both live. |
| Heightmap import commits without showing the working grid | `new_world_dialog.gd:397-424` | Prints `"2048 × 1311 → working grid 1024 × 656"` before Import, exactly as proposed. |
| Menu bar is missing `↶ ↷ ◐` | `dcc_shell.gd:990-1038` | All three built: undo/redo squares plus the theme square, wired through the same `apply_theme()`/`rebuild_theme()` pair as Preferences. |
| Fourth global tool `pan` is not drawn | `dcc_widgets.gd:913-919` | `GLOBAL_TOOL_ENTRIES` now has 4 entries; `pan` carries the verbatim spec tooltip and no arm callback. |
| GPU readback failure bans the session with no way back | `menus.gd:1910-1942,3044-3058` | `Preferences ▸ Performance ▸ Try the GPU again` live, gated on `gpu_clear_readback_failures`/`gpu_readback_failed`. |
| The place tool does not hit-test through the engine | `map_overlay.gd:1946-1969` | **Closed as a reasoned, disclosed divergence rather than built as literally proposed.** Investigated and dated 2026-09-01: `civ_pick_place_at` is a grid-space engine pick that can return an off-screen or hidden settlement; the shell's own screen-space hit-test is argued as the more correct rule for a pointer, at the cost of one tie-break (nearer-wins instead of rank-weighted) that is named and left as a known, minor divergence rather than silently dropped. |
| Safe-area insets are mock constants | `dcc_shell.gd:842-878` | `DisplayServer.get_display_safe_area()` now read; every inset is `max(real, mock)`. |
| No haptics anywhere | `dcc_shell.gd:890-914`; call sites `app.gd:117`, `dcc_shell.gd:4290,5138,5738,5741,5747` | `_haptic(kind)` built with the full spec table (`sample/detent/tool_arm/verdict/back/blocked`). Wired at the three sites the proposal named (detent, tool arm, back). Minor residual: `sample`/`verdict`/`blocked` are defined in the table but have no caller yet. |
| No relief-exaggeration default in Preferences | `dcc_settings.gd:375-405`; `menus.gd:1962-2014` | `Preferences ▸ Graphics ▸ relief exaggeration default` live, feeding `New world…`'s existing slider. |
| Landmark viewshed note states the superseded weighting | `civilization_workspace.gd:2156-2167,2274-2278` | Both the panel note and every affected row's tooltip now read the owner's `0.6 × prominence + 0.4 × visible land area inside 30 km` formula. |
| Two computed analysis fields have no debug view | `sample_bridge.rs:147,703,708` | `local_relief`/`tpi_multiscale` both added to the view registry with the proposed wording. |
| Timeline-aware tid reseed is never called | `lib.rs:3676`; `project_bridge.rs:1108-1111` | `civ_resync_next_tid_with_timeline` now called from both production sites, not just tests. |
| Generation and save failures never surface on phone | `app.gd:1780,1905-1907,1945-1947,2084-2088` | `_report_failure()` routes every one of the three through `_show_phone_toast()`. |
| Bottom-docked controls do not ride above the IME | `dcc_shell.gd:920-942,5861-5862,5937,6099-6103` | `DisplayServer.virtual_keyboard_get_height()` polled phone-side; `_phone_kb_height` added to every bottom inset, docked and floating. |

---

## Medium — 3 open (14 of 17 closed)

### Open

| Item | Where (`file:line`) | Current state | Size |
|---|---|---|---|
| `statusMid` composite | `app.gd:739-791` (drifted from `:698-720`); `dcc_shell.gd:3231-3237` (unmoved) | **~90% built, one field genuinely blocked.** Stage name (last `resolved` stage), pass duration and autosave state are all live and correct. `repaint NN ms` remains deliberately absent — the code says so at its own call site — because this shell composites through `ViewportHost` + `map_overlay.gd` + overlays with no single-pass timer to read, and the prototype's one-canvas-pass model doesn't transfer. **Still blocked on Owner question 2.** | medium |
| The right dock does not follow the armed tool (`rdExtraMode()`) | `right_dock.gd:688-712` (`_dispatch()`); `:506-572` (the per-tool `show_*`/`leave_*_context()` pairs) | **Narrower than stated, re-checked 2026-09-02.** The structural claim still holds: `_dispatch()` keys off one `_context` field, with no general "follow the armed tool" mechanism. But `cff1edc` landed four more tool-arm-driven contexts (`show_paint`, `show_stops`, `show_anno`, `show_territory` — each called from its owning workspace's own tool-armed handler) alongside the pre-existing Sculpt/Journey pair, so most of `UNIFIED_TOOL_PLAN.md`'s ten tools now push dock content on arming. Each call site hand-decides whether that overrides a live selection (`leave_sculpt_context()`/`leave_paint_context()` reset only from their own context, documented as deliberate). That is a real, working answer to the practical worry Owner question 1 raised — arming any of these four tools does override a selected settlement, and nothing has broken — just precedent rather than the generic `rdExtraMode()` binding this row asked for. **Owner question 1 itself is still open**: no ruling records "tool wins, per call site" as the adopted model, so the next tool added still has to guess which way to fall. | medium |
| Previews re-upload the whole texture | `pass.rs:193,199` (unmoved); consumers `lib.rs:6128-6179` (sculpt, drifted from `:5895-5925`), `:7000-7023` (paint) | **Investigated and honestly declined for this pass, not silently skipped. Re-verified 2026-09-02, unchanged in substance.** `build_sculpt_preview_texture`'s own doc explains in detail why a bounded preview needs `render.rs`'s AO/wetness/sea passes reworked to run over a caller-supplied window — real surgery on code `golden_parity_render.rs` pins bit-for-bit — and says explicitly that restricting only the final pixel loop "would shrink the returned image without touching the dominant cost, which would be a cosmetic optimisation reported as a real one." **New: `build_paint_preview_texture`'s own doc argues paint doesn't need this at all** — its preview is a flat per-cell colour lookup with no derived whole-grid rasters underneath (no AO/wetness/hillshade), so the cost a bounded variant would save is "negligible" there, unlike sculpt's. Correctly left for a dedicated live-preview pass rather than half-built here. | medium |

### Closed

| Item | Where (current) | How it closed |
|---|---|---|
| The expanded timeline strip | `dcc_shell.gd:3000-3020,3054-3100`; layer toggles `:3067-3075` | Fully built: content-driven height (collapsed/expanded), year cursor through `civ_goto_year`, speed pills `×1/×10/×100`, footer, and the six `TL_LAYERS` toggles with `BUILD_ANSWERS.md`'s verbatim note. One residual noted in the code itself: `civilization_workspace.gd`'s own year pills should connect to `timeline_changed` too, and don't yet. |
| Six simulation layer toggles | `dcc_shell.gd:3067-3075` | Built in the same pass as the strip (Climate/Population/Economy/Politics/Infrastructure/Warfare), with the required "no layer renders yet" note. |
| Sample fields do not read `—` when their stage is stale | `right_dock.gd:215-320,600-613` | `_stale_now()` (1 s cache) + `_stale_reason(label, stale)` gate every sample field individually, not just a whole-bar message. |
| Phone undo-history popover | `dcc_shell.gd:5119-5213` | Built to spec: 520 ms hold, `EDIT HISTORY · TAP TO ROLL BACK` header, rows newest-first capped at 6, reusing the desktop's existing multi-step `undo_ledger`/`undo_revert_to` — no new binding needed, as the original row predicted. |
| Phone sim strip | `dcc_shell.gd:444,3496-3498,5351-5475`; `app.gd:1337-1340` | Built, sharing the one `timeline_changed` signal with the desktop strip so the two cannot diverge. |
| App-bar `⋮` overflow | `dcc_shell.gd:3627-3654,4675-4682` | Built exactly to the 2026-08-31 canvas: Save project (+ `savedAt`), Theme (+ live label), Close world. |
| `drafts/paint.json` and `drafts/sculpt.json` slots | `project_bridge.rs:148-151,1926-2000`; `app.gd:2040-2071` | Built, and better than proposed: `project_engine_built_documents()` assembles all four engine slots; `_project_documents()` merges them with the shell's own `entities/journeys.json`; **both** `_write_project()` and `_autosave_tick()` now call it — fixing a real bug where autosave used to write an empty document map (no journeys, no paint, no sculpt draft, no libraries) even though a manual save didn't. |
| `library/assets.json` and `library/travel.json` slots | same as above; `project_bridge.rs:2346-2484` | Built in the same pass as the two rows above — all four slots share one mechanism. |
| Global `Redo` | `lib.rs:12582-12841,13582-13742`; `engine_bridge.gd:1614-1638`; `menus.gd:627-645,766-777,934`; `dcc_shell.gd:1114-1182` | **Fully built**, engine to menu to menu-bar square: `RedoTail`, `redo_available`/`redo_label`/`redo_last` `#[func]`s with a dedicated `global_redo_tests` module (truncation-on-new-operation, budget eviction, etc.), wrapped in `engine_bridge.gd`, wired live in `Edit ▸ Redo` and the `↷` menu-bar square, both correctly falling back to a true stale-build reason when the binding is absent. |
| No content descriptions, no dynamic type | `dcc_shell.gd:774-823` (23 `accessibility_*` sites) | Both halves built. `accessibility_name`/`_description` set on every glyph-only control (the menu-bar squares, the four phone app-bar cells). Dynamic type: `_os_text_scale()` derives Android's real font-scale setting from `screen_get_scale()/screen_get_dpi()` (checked against a full `ClassDB` sweep that found no direct accessor anywhere in this Godot build), clamped 0.85–1.6 for layout safety, and every phone label goes through it. Explicitly a no-op off Android, disclosed as such. |
| No storage-full handling | `app.gd:1976-2000,2081-2088`; `engine_bridge.gd:1153-1156`; `lib.rs:7903` | `disk_free_bytes()` bound; `_save_blocked_by_space()` refuses with a real message before a doomed write, treating an unknown answer (`-1`) as non-blocking rather than refusing every save on an older build. |
| `Save layout as…` | `dcc_settings.gd:428+`; `menus.gd:3246-3401` | Fully built: named snapshots of the five region toggles, active domain/mode, rail expansion, and (since 2026-09-01) the phone tool sheet's detent. `Reset layout` is the built-in first entry. |
| Atlas cache `Size cap · GB` | `menus.gd:2811-2880` | `atlas_evict_to(bytes)` bound and used; the cap ladder mirrors Performance ▸ VRAM budget as the old reason predicted it would. |
| Manual road tool / `road_edges` never retained | `journey_bridge.rs`, module doc | **The claim was false, not the row's title stale-open.** `CivData::road_edges` (`cartalith-godot/src/lib.rs:188`) genuinely retains `civ_hierarchical_network_topology`'s output (a different producer than the never-called `build_road_network` this row named) and both `jp_road_cells` call sites already used it — only the module doc said otherwise, now corrected. `jp_road_cells` also gained a `manual_ways: &[tools::ManualWay]` parameter this pass, so hand-drawn roads (the "manual" half of this row's own title) now reach it too. No literal "manual road tool" control exists anywhere in the shell (`grep -rni "manual road|road tool" godot-project/shell` — zero hits); the title referred to hand-drawn-road data reaching the planner, not a named button. |

---

## Large — 17 open, 1 fully closed (1 half-built)

`LARGE_ITEM_RULINGS.md` recorded owner decisions for all 18 on 2026-08-31 and
said plainly that none of them was in flight that day — the work in flight
was the 59 trivial/small/medium rows. **That remains true today for 16 of the
18.** Two exceptions:

| Item | Where (`file:line`) | Proposal / ruling | State |
|---|---|---|---|
| **Paint brush falloff** | `cartalith-spatial/src/paint.rs:143-320`; `paint_bridge.rs:38-65,219-466`; UI `world_workspace.gd:2098-2106`; `tool_bar.gd:433-442`; `DECISIONS.md` §7k | Ruled: **bind it** — a deliberate, disclosed divergence from the reference, recorded in `DECISIONS.md` when it lands. | **Fully built and verified, second pass 2026-09-01.** `PaintStamp::hardness`/`softness` (`paint.rs:143-144`), the `with_falloff` builder (`:180`), `feather_width` (`:199`), `passes_falloff` (`:219`) and `cell_dither` (`:249`) all exist and are exercised by `Stamp::apply` (`:299-319`). `paint_bridge.rs::stroke_at` (`:466`) calls `.with_falloff(self.brush.hardness, self.brush.softness)` on every dab. `DECISIONS.md` §7k records the divergence as ruled. The duplicate slider is resolved: `tool_bar.gd:433-442` deletes its copy in favour of `world_workspace.gd`'s, which carries both Hardness (`:2103`) and Softness (`:2105`) with tooltips naming the real mechanism. Verified independently, not carried from the report: `cargo test -p cartalith-spatial --lib` (148/148, 4 new), `--test golden_parity_paint` (7/7, unchanged — the hard-disc reference case is bit-identical), `cargo test -p cartalith-godot --lib` (409/409, 6 pre-existing ignores, 3 new tests) after a fresh `cargo build -p cartalith-godot` (the dll was stale against exactly these files), `cargo test -p cartalith-civ --lib` (513/513, proving the territory brush's separate `PaintStamp::ungated` use is unaffected), `cargo check --workspace` clean, and `--headless --check-only` clean on all four touched `.gd`/`.rs`-adjacent scripts. **One disclosed residual, not this row's to close:** `lib.rs:6704-6705`'s `paint_set_brush` doc comment still says hardness/softness are "never consumed" — confirmed still present, now stale, owned by whoever next touches `lib.rs`. |
| **Landmark funnel: crowding and rejected candidates** | `landmark.rs:475-524,580-646` (unmoved — the uncommitted way-network landmark-kind work in this file inserts only after line 646); UI `civilization_workspace.gd:2183-2192,2728-2740` (drifted from `:2059-2068,2589-2607`) | Ruled **both halves**: a crowding parameter on the placement pass, plus a rejected-candidate coordinate list and an overlay layer. | **Half built, re-verified 2026-09-02 — unchanged.** `LandmarkSettings::crowding` is fully live: clamped `[0.05, 3.0]`, scales every class's exclusion radius, tested (`crowding_higher_packs_tighter`, NaN/zero guarded), and driven by a real, wired `Crowding` slider (0.25×–2.00×) with a live km-clear readout — not a `_todo`. **Not built:** `LandmarkFunnel` still carries counts only (`candidates, rejected_constraint, rejected_score, rejected_spacing, rejected_cap, cap, placed`), no coordinates. "Lower crowding to fit" and "Show rejected" stay disabled `Callable()` chips with accurate, current reasons naming exactly what's missing. |

The other 16, re-verified rather than assumed unchanged:

| Item | Where (`file:line`) | Proposal / ruling | Size |
|---|---|---|---|
| `Cut` · `Copy` · `Paste` · `Select all` | `menus.gd:686-691,707-710` (drifted from `:686-708,717-721`) | Ruled: selection sets → clipboard → commands, in that order. Still `_todo`; still three unrelated single-`i64` selections (`icon_get_selected`, `label_get_selected`), still no clipboard model. Reasons verified current and true, 2026-09-02. | large |
| CARTO ▸ Labels: the whole panel | `cartography_workspace.gd:1217-1338` (drifted from `:1118-1294`, and the file moved under `shell/workspaces/`) | Ruled: all three steps (label_class field; a generated labelling pass; a per-class typography record). No `label_class` symbol exists anywhere in `crates/` (re-checked repo-wide, 2026-09-02). Panel still reads counts as `--`, sliders still `_dead_slider`. | large |
| Label collision culling | `cartography_workspace.gd:1285-1298` (drifted from `:1249-1255`) | Ruled: build with the labelling pass, not standalone. Toggle still drawn checked-and-disabled with the true reason ("label boxes are never measured against each other"). | large |
| CARTO ▸ Icons: generated placement | `cartography_workspace.gd:1359-1428` (drifted from `:1377-1412`) | Ruled: build, plus a new sea-marks asset family (answers owner question 4). No generated placement pass exists; `_icon_placement_family` is still read-only; the sea-marks family is not in `ICON_FAMILIES` (still 3: settlement/feature/poi). | large |
| The river entity | `right_dock.gd:1072-1122` (drifted from `:930-987`) | Ruled: one `get_rivers()` binding plus viewport hit-testing. Still zero `get_rivers`/`river_*` in `cartalith-godot` (re-checked 2026-09-02); still unreachable (nothing in the viewport can select a river — see "Left undetermined" below). The dock's own note and all seven dashed fields were rewritten 2026-09-01 for accuracy (correcting an over-claim that *no* river data crosses the boundary — per-cell Strahler order and discharge do), but the entity itself, and therefore this row, is unchanged. | large |
| Civilisation authoring operations | `civilization_workspace.gd:1041-1043,1397-1404` (drifted from `:1362-1378`); `infrastructure_workspace.gd:1115-1122` (roads) | Ruled: five re-entrant `#[func]`s over an existing world, plus a civ `PARAMS` group. All five (`civ_clear_territory`, `civ_populate`/Auto-populate, `civ_clear_places`, road generate/clear) remain `func(): pass` + disabled; re-checked 2026-09-02, no such symbols exist anywhere in `cartalith-godot`. **The single largest CIVIL gap, unchanged** — landing urban milestones 8-17 (`4ec07f5`/`cff1edc`) touched none of these five; that work is settlement *layout*, not settlement *placement/clearing*. | large |
| Settlement diagnostics overlay | `civilization_workspace.gd:1405-1415` (drifted from `:1368-1378`) | Ruled: surface the data now — no Cargo edit needed, `cartalith-urban` is already reached through `urban_adapter`. **Re-verified 2026-09-02: the control is unchanged (`func(): pass`, disabled), but its own tooltip is now the stale item — see the dangerous class below.** It still reads "blocked on urban milestones 9, 10 and 13" and "`_umSiteProfile` is unported because its own consumers are unbuilt." Both are false at HEAD `cff1edc`: `water.rs`/`fortify.rs`/`districts.rs` (m9/10/13) are built (STATUS.md UM-9/UM-10/UM-13 "done", 693/1288/1307 lines), and `urban_adapter.rs`'s own doc table (`:41-42`) now calls `um_site_profile`/`um_harbour_scale` **"ported,"** naming three of `_umSiteProfile`'s four one-time blockers as resolved — only a Settlement Inspector still doesn't exist. **The real blocker moved, it didn't close**: zero `#[func]` anywhere exposes `um_site_profile`/`um_harbour_scale`/a per-settlement wall rung (checked fresh); the one urban binding that does cross the boundary, `urban_layouts()` (`urban_bridge.rs:320`), returns a whole generated Town — walls, districts, markets, farmland — for the City Viewer, not this control's cheap three-line card. Settlements still carry no `specialisation` (unchanged, `urban_adapter.rs:1472`), and the third line's own gate, `_umModelCache`, stays out of scope for every milestone by the adapter's own design (`:45`) — unrelated to any milestone number. So the *ruling* — surface what exists — still has not been executed, and the honest next step is now "add a lightweight `#[func]` over the ported pure functions," not "wait for milestones 9/10/13." | large |
| The 3D viewport | `menus.gd:1756-1757` (drifted from `:1754-1756`) | Ruled: **deferred**, research first. `3D_TERRAIN_RENDER_RESEARCH.md` (1 530 lines) exists, complete, per `LARGE_ITEM_RULINGS.md`; three commissioned questions parked. No 3D work scheduled — correctly still absent (`grep` for `Camera3D`/`MeshInstance3D` in `shell/` and `.tscn` files, re-run 2026-09-02: nothing). | large |
| Colour management | `menus.gd:1754-1755` (unmoved) | Ruled: **build it**, behind an sRGB-identical default or a deliberate re-baseline. Still `_todo`, same reasoning as before the ruling ("The renderer is sRGB-only end to end... A three-row radio that always resolves to sRGB is exactly the enabled-and-inert row this menu forbids"). No colour-space symbol anywhere in `lib.rs`. | large |
| `Region ▸ New world from selection` | `ops_bridge.rs:1-28` (unmoved) | Ruled: a scoped parity pass, kept separate from GUI work. `extract_region_as_world` still has no `#[func]` and no menu row; `ops_bridge.rs`'s own doc still lists it first among ported-and-unexposed capability. Correctly untouched (re-checked 2026-09-02). | large |
| Saved measurements + CSV | `right_dock.gd`, `_build_measure_actions` and the "Saved measurements, on disk" section | **Closed 2026-09-03.** The fifth slot the ruling called for exists: `annotations/measurements.json`, registered in `cartalith_io::DOCUMENT_SLOTS` and caller-owned (`project_bridge.rs`'s partition test names it among six callers, not five). Written by `RightDock.measurements_document()` through `app.gd::_project_documents()` — the same dictionary the engine's four ride — and restored by `restore_measurements_document()` from `_restore_project_documents()`. `Save measurement`, the list, per-entry recall/drop, `Clear all` and `Copy saved as CSV` are all live; the CSV is canonical km/km²/m/deg. Verified end to end on a live world by `_measurestore_probe.tscn` (24 checks, 0 fails), which saves a project, reopens it and asserts both readings and every clicked point come back. | large |
| The manual-icon tool | `manual.rs` (`icon_brush_rule:189`, `icon_brush_stamp:211`); `ops_bridge.rs:1-8` | Ruled: schedule separately as `UNIFIED_TOOL_PLAN.md` Milestone E. Still correctly untouched — no icon-brush arming anywhere in `shell/` (repo-wide check, re-run 2026-09-02). | large |
| Rebindable keyboard shortcuts | `menus.gd:1902-1907` | Ruled: a per-context table in `DccSettings` with conflict detection. `_todo` unchanged; `Help ▸ Keyboard shortcuts…` is still read-only; no per-context store exists. | large |
| `Units` (km / mi) | `menus.gd:1899-1900` (unmoved) | Ruled: build, plus nautical miles as a third unit. `_todo` unchanged; still km-only at all five call sites the reason names. | large |
| CPU worker threads | `menus.gd:1720-1721` | Ruled: build a configurable pool. `_todo` unchanged; `ThreadPoolBuilder` still appears nowhere in the workspace outside one benchmark printout (re-checked 2026-09-02). | large |
| `Report an issue` | `menus.gd:3554-3555` | Ruled: replace with a local diagnostic-dump action (no endpoint needed). Still the original `_todo`, unchanged — even though three of the five readouts the replacement would bundle (format version, missing-bindings fingerprint, generation info) were built in the 2026-09-01 pass as Trivial rows and are sitting right there to be reused. | large |

---

## The dangerous class — 2 entries: 1 non-defect kept for contrast, 1 real

A disabled row with an honest tooltip costs a user nothing. These cost them
trust. **Re-verified at the code, not carried forward.** All 9 false-reason
rows and all 9 misleading-presentation rows from the 2026-08-31 cut are still
closed, re-checked rather than assumed. Of the 7 enabled-and-inert rows, 6
closed (4 in the first cut, 2 — both Paint falloff symptoms — in the same-day
second pass) and **1 was never a defect** and is kept in the table for
contrast.

**1 real, newly found this cut (2026-09-02) — not carried forward from any
prior cut, because it did not exist yet.** Settlement diagnostics overlay's
own tooltip (`civilization_workspace.gd:1405-1415`) names "urban milestones
9, 10 and 13" and an unported `_umSiteProfile` as its blocker. Both are false
at HEAD `cff1edc`: all three milestones shipped (`water.rs`/`fortify.rs`/
`districts.rs`, landed `4ec07f5`), and `urban_adapter.rs`'s own doc table
now marks `um_site_profile`/`um_harbour_scale` **"ported."** The control
itself stays correctly disabled — see its own Large-table entry above for
the corrected, narrower blocker (no `#[func]` exposes either function; the
only urban binding, `urban_layouts()`, builds a whole Town, not this
control's three-line card) — but the words a user reads blame milestones
that no longer block anything. See §(b) below.

### (a) Drawn ENABLED, does nothing meaningful — 0 remain (6 closed, 1 kept as a documented non-defect)

| Item | Where | What actually happens | Severity |
|---|---|---|---|
| ~~**Paint `Hardness`** — two copies on screen at once~~ | `world_workspace.gd:2103`; `tool_bar.gd:433` (deleted) | **Closed, second pass 2026-09-01.** Now consumed: `paint_bridge.rs::stroke_at` calls `PaintStamp::with_falloff(hardness, softness)` on every dab (`paint.rs:180,219,299-319`), verified by a direct search and by test (`hardness=0.4 must paint a different set than the hard disc`). The duplicate is resolved too — `tool_bar.gd`'s copy is deleted outright, not hidden; `world_workspace.gd`'s is the one surviving control. | — |
| ~~**Paint `Softness`**~~ | `world_workspace.gd:2105` | **Closed, second pass 2026-09-01.** Same fix as Hardness above — both feed the one `feather_width()` softening amount and both are genuinely read. Tooltip now describes the real mechanism rather than disclosing non-consumption. | — |
| ~~Biome-K checkbox reads the wrong state~~ | `new_world_dialog.gd:504-506` | **Closed.** `_sync_from_engine()` now reads `get_biome_k_enabled()` on every `about_to_popup`. | — |
| ~~`Preferences ▸ Theme` does not persist~~ | `dcc_settings.gd:361-372` | **Closed.** Persisted through `DccSettings.theme_mode()`/`set_theme_mode()`. | — |
| ~~`Window ▸ Status bar` on phone~~ | `dcc_shell.gd:2117-2122` | **Closed.** Routed through `set_status_region_shown()`. | — |
| ~~`Window ▸ Left/Right dock` and `Reset layout` on phone~~ | `app.gd:2727-2755` | **Closed.** Routed through `_set_sheet_open()`; `Reset layout` now closes both sheets on phone. | — |
| CARTO ▸ Labels `Class` dropdown | `cartography_workspace.gd:1271-1276` (drifted from `:1191-1216`, re-verified 2026-09-02) | **Unchanged, and still not a defect.** Live and panel-local: repaints the class list, the title, and re-seats the three still-inert sliders on that class's design defaults. Writes nothing any renderer reads, and its own tooltip discloses that. Kept in the table for the same reason as last cut — it is the one control in this panel that costs the engine nothing and was deliberately left live so the frozen half reads as intentional. | low, disclosed |

### (b) The stated reason is FALSE — 1 found (9 of 9 carried-forward still closed)

Every row carried forward from the 2026-08-31 cut still has a true reason,
re-verified at the code rather than assumed: **State religion** (closed
pre-cut, unchanged — reads the `roster` dict directly); **`pack.rs` painted
layers** (module doc rewritten, names the real producer and the real
remaining gap); **`_refresh_phone_bar_lit()`** (deleted, pre-cut, confirmed
still gone); **`Window ▸ Domain rail` on phone** (`_rail_region` now points
at exactly the three domain cells, not every `PHONE_TABS` cell); **the
first-run coach mark** (rewritten against `PHONE_TABS`); **`No landmark
types`** (reworded to a true stale-build reason); **the hero readout**
(`fs_hero`/`fs_hero_2` are now consumed by `DccTheme.hero()`/`hero2()`);
**tides** (split from geoid, correctly marked live); **the app-bar `⋮`**
(built to the 2026-08-31 canvas's `hMenu` — Save project / Theme / Close
world).

**Newly found, 2026-09-02: Settlement diagnostics overlay.** Its tooltip
(`civilization_workspace.gd:1405-1415`) reads "blocked on urban milestones
9, 10 and 13" and "`_umSiteProfile` is unported because its own consumers
are unbuilt." Both are now false, not merely stale-flavoured: `water.rs`
(m9, 693 lines), `fortify.rs` (m10, 1 288 lines) and `districts.rs` (m13,
1 307 lines) are all built and committed (`4ec07f5`; `STATUS.md` UM-9/
UM-10/UM-13 all "done"), and `urban_adapter.rs`'s own module-doc table
(`:41-42`) calls `um_site_profile`/`um_harbour_scale` **"ported"** outright,
naming three of `_umSiteProfile`'s four one-time blockers as now built —
only a Settlement Inspector still doesn't exist. The control itself is
unaffected: still `func(): pass`, still correctly disabled, since
(re-checked fresh, not carried from any prior claim) zero `#[func]` anywhere
exposes either function, and the one urban binding that does cross the
boundary, `urban_layouts()`, builds a whole generated Town rather than this
control's three-line card. So the disabling is still right — only the words
explaining it are wrong. Not fixed here (`civilization_workspace.gd` is not
this lane's file); recorded so the next pass corrects the string instead of
re-deriving why it's disabled.

### (c) Reason true, but the presentation misleads — 0 remain (9 of 9 closed)

Every row here is now honestly presented: `CommandIndex` readouts carry a
`"readout"` kind rather than reading as a dead command; the `GAP_LAYERS`
"Not available for this world" self-contradiction is split into two
sentences; the "eleven `GAP_LAYERS`" count now correctly reads
four-plus-seven; the `params.rs` entry-count claim was deleted rather than
updated to a new number that would just go stale again; the icon fill line
is tagged `(design figures)`; the viewshed note carries the owner's current
formula; the landscape drag handle is hidden rather than left visible-and-
inert; the disabled phone radios keep their mark; and — closed this cut,
found by re-checking rather than assumed fixed alongside its siblings — **the
three river Actions** (`right_dock.gd:1103-1130`, drifted from `:966-987`, re-verified 2026-09-02) each now carry their own
specific tooltip (Hydrology / Edit geometry / Analyse catchment, each naming
a different missing binding) instead of one seven-word sentence repeated
three times.

### The §4 disclosure scorecard — fully closed

`BUILD_ANSWERS.md` §4's four "disclose rather than build" items are now all
disclosed where a user can see them, closing the gap the 2026-08-31 cut found
in all four:

| §4 ruling | Built as ruled? | Disclosed? |
|---|---|---|
| Map canvas stays dark in light theme | Yes, by omission (unchanged) | Unverified this cut — not re-opened; carried forward from 2026-08-31 as the one un-rechecked cell in this table. |
| Rotation with a sheet open in landscape | Yes | **Yes, closed this cut** — the handle is now hidden in landscape (Trivial row above), not merely inert-and-visible. |
| Generation-failure and storage-full states | **Yes, closed this cut** — both built (Small + Medium rows above) | **Yes** — both route through `_show_phone_toast()`. |
| Content descriptions and dynamic type | **Yes, closed this cut** (Medium row above) | **Yes** — `accessibility_*` set, `_os_text_scale()` disclosed and clamped. |

---

## Not gaps — recorded so they are not re-listed next time

**Build-conditional `_todo`s, not permanent rows.** Structurally unchanged and
re-verified: the five GPU/Documentation rows are still gated behind `if
_bridge.gpu_api:` (`menus.gd:1714,1722`) and `_docs_dir() == ""`, both true on
every current build. **Any table that lists these flat is still wrong.**

**Presented, permanently inert, and correct.** `Follow system`
(`menus.gd:1889-1893`, disabled only where `DisplayServer.is_dark_mode_supported()`
is false — unchanged); `Alternate frames`/`Reduce working res`, still enforced
engine-side via `gpu_set_multi_mode`/`gpu_set_vram_fallback`
(`menus.gd:2194,2282`); the Working set / VRAM estimate readouts (now
correctly indexed as `"readout"`, not `"available: false"` — see the dangerous
class §c closure above, which improved this category rather than changing its
verdict).

**Deliberate omissions, argued in code.** Re-verified where cited above (CIVIL
POI, WORLD palette Sculpt/Freehand, `Data ▸ Conversion` — confirmed still
absent from `menus.gd`, the owner's 2026-08-20 removal still holds, no
re-add). Not exhaustively re-walked this cut; nothing found in contact with
this material contradicts the previous cut's entries.

**Superseded twins, `audit_wiring.py` false positive, declines verified
true, and the prose "Not built" sections** — carried forward from the
2026-08-31 cut. None of the code paths this cut touched intersects them, and
spot-checking a sample (`Undo depth 1-50`'s substitution note,
`civ_zoom_pick_r`'s disclosure) found no drift.

---

## Owner questions

1. **Does `rdExtraMode()` replace the right dock's selection contexts, or sit
   beside them?** **Still formally open, narrower after `cff1edc`
   (2026-09-02).** Four more tools (paint/stops/anno/territory) now push dock
   content on arming, each by its own hand-written call-site judgment about
   overriding a live selection — sitting beside, in practice, for most of
   `UNIFIED_TOOL_PLAN.md`'s ten tools now. No ruling has generalised that
   pattern into policy, so it is precedent rather than an answer the next
   tool can rely on. See the right-dock Medium row above.
2. **What should `statusMid`'s `repaint NN ms` measure?** **Still open.**
   Still the one field blocking full closure of the `statusMid` Medium row.
3. **Should the WORLD left-dock A/B switch come back?** **Still open.** No
   `ldSwitch`/`ldSwA`/`ldSwB` anywhere in `world_workspace.gd` — the labels
   this would need are still in the design's truncated tail.
4. **How do the design's four icon placement families map onto the engine's
   three?** **Answered** by `LARGE_ITEM_RULINGS.md`: they don't map — a
   fourth (sea marks) is created. **Not yet built** — still part of the open
   CARTO Icons Large row.
5. **Paint falloff: bind it, or delete the sliders?** **Answered**: bind it,
   as a recorded divergence. **Built and verified, second pass 2026-09-01** —
   see the Large section above. No longer an open row.
6. **Should a committed sculpt stamp re-evaluate when sea level moves?**
   **Still open.** `SculptStamp::with_sea_level` (`sculpt.rs:1076`) still has
   no caller anywhere in the workspace — re-confirmed this cut by a
   repo-wide search, not just the definition site.
7. **Are the four unwritten save slots deliberate or an oversight?**
   **Answered and fully executed.** All four original slots
   (`drafts/paint.json`, `drafts/sculpt.json`, `library/assets.json`,
   `library/travel.json`) are built and wired into both save and autosave
   (Medium, closed, above), and the *fifth* the ruling called for landed
   2026-09-03: `annotations/measurements.json`, caller-owned, riding the same
   channel rather than a second mechanism (Large, closed, above).
8. **Is `init_gpu_f64` kept or deleted?** **Still open.** Unchanged;
   `GPU_COMPUTE_PILOT_SCOPE.md` still records no disposition.
9. **Is the phone app bar's `☰`/`▤` pair now stale?** **Not simply resolved
   — re-examine.** The shell now draws `☰ / title+seed / ⌕ / ▤ / ⋮` on the
   phone app bar (`dcc_shell.gd:3655-3723`), citing `DCC_SHELL_SPEC.md` §13
   as authority ("the phone app bar is '☰ (domain drawer), title + seed, ▤
   (panels), ⋯ (overflow menu)'. So the fix and the spec agree",
   `dcc_shell.gd:3711-3713`). **That citation is to a passage the same spec
   file marks superseded** (`DCC_SHELL_SPEC.md:1014-1017`: "~~The app bar...
   ☰... ▤...~~ **Superseded**: 56 dp; ▤ and ⋯ moved to the bottom nav"), and
   the result still does not match the 2026-08-31 Android canvas this
   question originally cited (`[world pill] · ⌕ · ⋮`, no `☰`, no `▤`). The
   `⌕` and `⋮` cells are independently justified and correctly built (real
   destinations, guarded existence checks); `☰` and `▤` are not wrong, but
   the authority cited for keeping them is questionable on its own terms.
   Under `CLAUDE.md`'s "the newer canvas wins" rule, an owner look at
   whether the 2026-08-31 canvas's simpler bar should win instead is still
   warranted — this was not settled by this cut, only clarified.
10. **`--good` and `--accH`.** **Still open**, unchanged — both still
    declared-and-unused in the prototype itself per `dcc_theme.gd`'s own
    comments, so a shell with no consumer may be fidelity rather than a gap.

### Left undetermined by this pass — work someone must do, not decisions to take

- Whether the disabled CARTO Labels/Icons panels read as inert against the
  light theme's ground. Not re-checked this cut (needs a light-theme capture,
  not a code read).
- Whether the phone's measure strip / label bar / way card count as missing.
  Not re-checked this cut (needs a handset run).
- The 44 dp vs 48 dp target sweep. Not re-checked this cut (needs a measured
  run on a device).
- Whether `sculpt_stroke_point` can reject a point the shell already
  appended. **Unchanged** — `sculpt_stroke_point_count()`
  (`engine_bridge.gd:1465-1468`) is still a plain counter with no divergence
  check; `_sculpt_stroke_points` still lives at `world_workspace.gd:234` (the
  line number moved; the shape did not).
- Whether the `right_dock.gd` River context is reachable at all. **No longer
  undetermined — confirmed unreachable.** The code's own comment now states
  it plainly: "No `get_rivers()` exists and nothing in the viewport can
  select one ... this context has no live trigger today"
  (`right_dock.gd:1074-1075`, drifted from `:932-936`, re-verified
  2026-09-02 — same wording, same conclusion). Implemented anyway, for the
  same reason it was before — `_dispatch()` stays complete and honest rather
  than silently dropping the branch.
- Landscape composition beyond the sheet handle. Not re-checked this cut.
- Whether any `_todo` reason cites a `PARITY_AUDIT.md` section number that
  has since moved. Not re-checked this cut.
