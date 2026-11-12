# OUTSTANDING_WORK.md second half, reconciled against the code, 2026-11-12 (B)

A dated cut, not state: `cartalith-native/docs/STATUS.md` is the state. This file records what re-opening each row of
`OUTSTANDING_WORK.md` section 2.7 through the end found at origin/main `d3873d70`. The first half (through 2.6) is the other
audit's. Line numbers below are that commit's and will drift; the row title is the key.

**How strong the evidence is.** DONE and STALE verdicts were re-opened at the named symbols. PARTLY and OPEN verdicts say what
was opened; where a row's own text carries a dated verified claim that I did not re-run (device passes, probe counts), the
evidence column says "per row". Nothing was marked done on a row's say-so.

## Counts (101 table rows in 2.7 to 3.3)

| Verdict | Rows |
|---|---|
| CLOSED-ALREADY (struck before this audit) | 26 |
| DONE (closed by this audit) | 2 |
| STALE (superseded, closed with a pointer) | 1 |
| PARTLY (trimmed or annotated) | 35 |
| OPEN | 37 |

Sections 4 to 7 are covered after the table.

## Rows closed or corrected by this audit

* **Governments G1** (3.1) is DONE: `roster.rs` `CIV_GOVERNMENTS` (17 entries, six core and two optional appended past
  `CIV_GOVERNMENTS_REFERENCE_LEN`), `manpower.rs` `GOVERNMENT_EXTRACTION`, `CITIZEN_MODERNISATION`,
  `SOLDIER_UPKEEP_BY_BRACKET`, `GOVERNMENT_LEVY_BONUS` (commit `61fc1585`), the test
  `the_khanate_levy_bonus_is_derived_from_its_era_band`, `g1_baseline.rs::g1_0_baseline_columns`,
  `faction_roster_window.gd::_government_choice` and `_gov_probe.gd`; `STATUS.md`'s Governments ledger lists G1-0 to G1-5 done.
* **Vault note list `private` badge** (3.1) is DONE: `vault_privacy.gd`, and `vault_window.gd` draws the badge on the browse tree's
  note rows; `STATUS.md` "VE-9 finished ... the private badge (Ruling DP)". The row said "Not built".
* **"Generation menu cannot be found on the phone"** (2.7) is STALE: its premise (`PHONE_TABS` reading MAP / GENERATE / PLAN / MORE)
  is gone. Ruling EH item 1 retired the GENERATE column; `PHONE_TABS` is now MAP, the domain tab, STORY, MENU.
* **Defects found while annotating** (2.8) trimmed: `_sort_by_pop`, `_BULLET_RE`, the `tool_fan` back button and the `app.gd`
  doc block are fixed; five defects remain and are listed.
* **Implement the mobile menus** (3.3) corrected: MS-6 (`3b04e444`) and MS-7 (`263a7b26`, with the 1 180 dp whole menu bar) are built;
  MS-11's quick drawer and "Share project" are not.
* **GUI replacement stages 3, 5, 6, 7** (2.10): the premise holds in one direction only. `STATUS.md`'s RP table still says "not
  started", but menus, the Context panel and tool-options bar, `phone_menu.gd` and the class-swap re-presentation of windows all
  exist. The row is PARTLY (spec conformance unmeasured). **`STATUS.md`'s RP-S3/S5/S6/S7 cells are the stale party and were not
  edited here** (other agents hold that file).
* **Tablet canvas rows** (2.7) annotated: Rulings EG and EM send every touch window under 900 dp to the phone GUI, so the gap
  now concerns tablets of 900 dp and over (`W_DOCK_TABLET` is still a flat 400, `dcc_theme.gd`).
* **Touch-drag `MOUSE_FILTER_STOP` row** (2.7) annotated with the central touch-scroll fix.
* **Section 6.8** first two bullets are resolved: `ROADMAP.md` no longer contains "all seven milestones" or the "revisit when a
  concrete need appears" wording.
* **Orphan text** after the first 2.7 row (the tail of a row closed 2026-09-21) is labelled as such; it had lost its header.

## Row by row

| Line | Verdict | Row | Evidence |
|---|---|---|---|
| 3344 | OPEN | **Environment: the user-level `JAVA_HOME` and PATH still name the removed `jdk-21.0.12.8-hotspot`** | re-opened; no code found that closes it |
| 3395 | OPEN | **Three-platform design-conformance verification — owner request, 2026-09-05** | re-opened; no code found that closes it |
| 3396 | CLOSED-ALREADY | ~~**`File ▸ Recent worlds` leaves show a filename where the canvas shows the world**~~ — **CLOSED 2026-09-21 ( | struck in the source row; its own text carries the evidence |
| 3397 | CLOSED-ALREADY | ~~**Rulings 28/29’s "size shown at save time" has no home in the shell**~~ — **CLOSED 2026-09-22 (verified), o | struck in the source row; its own text carries the evidence |
| 3398 | PARTLY | **THE PHONE SHELL MUST MATCH `Cartalith Android.dc.html` 100% FAITHFULLY — owner, 2026-09-07** | phone shell rebuilt by MS-1..MS-6 (STATUS headings); 100% conformance to the canvas is unmeasured; still the owner's glass check |
| 3399 | PARTLY | **`ColorPickerButton` ×8 and 123-155 other controls eat the drag by `MOUSE_FILTER_STOP`** | central touch-scroll fix (`dcc_widgets.gd` touch_passive/touch_list/touch_popup; STATUS 2026-11-08); ColorPickerButton triage not stated; handset unconfirmed |
| 3400 | OPEN | **Point Godot’s remote debugger at the ANDROID build and read the node tree off the device** | re-opened; no code found that closes it |
| 3401 | PARTLY | **THE TABLET CANVAS HAS NEVER BEEN IMPLEMENTED AGAINST — it is a DIFFERENT SHELL, not a scaled one** | narrowed by Rulings EG/EM (under 900 dp = phone GUI); `W_DOCK_TABLET` still 400 at `dcc_theme.gd:690`; MS-7 520 dp rule (`TABLET_MAP_MIN_DP`) |
| 3402 | OPEN | **The tablet palette is a STRUCTURAL change, not three numbers — and one contrast pair drops below 3:1** | re-opened; no code found that closes it |
| 3403 | OPEN | **§11: the fills are restored; the 51 stale citations are what is left** | re-opened; no code found that closes it |
| 3404 | PARTLY | **The app never idles: it redraws at 60 fps with nothing changing, and that is the power draw** — **FIXED 2026 | `app.gd:1496` sets `OS.low_processor_usage_mode`; on-device re-measure still owed; Ruling EM welcome idle fix |
| 3405 | PARTLY | **TABLET PORTRAIT: five Planet sliders are 290 px in a 232 px dock — the canvas answers with a 2-COLUMN GRID,  | third attempt committed 2026-09-13 per row text; two residues named in the row (Climate 244, rotate-to-portrait Terrain 239) not re-measured |
| 3406 | CLOSED-ALREADY | ~~**Implement the owner's re-sorted PC left rail**~~ — **CLOSED 2026-09-21 (verified) — Ruling L complete, all | struck in the source row; its own text carries the evidence |
| 3407 | CLOSED-ALREADY | ~~**149 of 200 addon villages have no road — CORRECTED 2026-09-23: this is a real port gap, not a design quest | struck in the source row; its own text carries the evidence |
| 3408 | PARTLY | **OWNER-REPORTED: the tablet still shows the PC layout — the spec PARKED adoption behind a decision the owner  | rail/menu work committed 2026-09-13 per row text; landscape 320 docks, palette, `scrPlanner` open; EG narrows scope |
| 3409 | OPEN | **The `--generate` leg of the capture probes crashes the GPU — three tablet figures are unconfirmed** | re-opened; no code found that closes it |
| 3410 | PARTLY | **Every generate still blocks the main thread: ~1.5 s at 2048 wide, ~5.8 s at 4096 (was 2.3 s and 10.5 s); 819 | `warm_render_caches` / `drawn_water_digest` landed `ef35265a` per row; 8192 not re-measured |
| 3411 | PARTLY | **Paint layers: show the original (as-generated) raster on an opacity slider while painting terrain/biome** —  | `paint_original.gd` exists; STATUS 'Phone Paint reachable'; the sculpt-snapshot half needs Rust, unchanged |
| 3412 | CLOSED-ALREADY | ~~**Phone: the Paint tool bar cannot be reached by thumb**~~ — closed 2026-10-04; narrowed. **Superseded 2026- | struck in the source row; its own text carries the evidence |
| 3413 | OPEN | **Zoom residual: about 8 frames over 16.7 ms per notch run remain; one unexplained ~27 ms frame on the first n | re-opened; no code found that closes it |
| 3414 | CLOSED-ALREADY | ~~**Zoom is clamped at the top of its range while the badge keeps advancing**~~ - **CLOSED 2026-09-21 (verifie | struck in the source row; its own text carries the evidence |
| 3415 | OPEN | **Boot has an unattributed ~9.4 s main-thread CPU stall between engine start and the painted welcome screen** | re-opened; no code found that closes it |
| 3416 | STALE | **OWNER-REPORTED ON GLASS: the generation menu cannot be found on the phone** | PHONE_TABS now MAP/domain/STORY/MENU in `dcc_shell.gd`; Ruling EH item 1 and STATUS 'Ruling EH item 1 built' |
| 3417 | CLOSED-ALREADY | ~~**METHOD: the on-glass method has now been RUN once — it works, and it is calibrated**~~ — **CLOSED 2026-10- | struck in the source row; its own text carries the evidence |
| 3418 | OPEN | Six features never driven on device since the 2026-08-24 USB disconnect — paint visibility, save/undo, the deb | re-opened; no code found that closes it |
| 3419 | OPEN | **Measure the reduced generation peak on the handset** | re-opened; no code found that closes it |
| 3420 | OPEN | **Lower the suitability plateau further** (memory, optional) | re-opened; no code found that closes it |
| 3421 | CLOSED-ALREADY | ~~The left-panel sheet retains its scroll offset across close/reopen and will not scroll back up — **NEITHER H | struck in the source row; its own text carries the evidence |
| 3422 | OPEN | Exercise **R1**'s Godot-side hunk inside a running Godot process on the handset | re-opened; no code found that closes it |
| 3435 | OPEN | **Annotate the existing code for a human auditor** — Ruling BK | re-opened; no code found that closes it |
| 3436 | PARTLY | **Re-freeze the reference and regenerate `FUNCTION_INDEX.md` in the same pass** - ~~*BLOCKED 2026-10-10: the v | `reference/` has the v2.71 file but no function index or drift file for it (listing checked) |
| 3437 | CLOSED-ALREADY | ~~**Found-bug sweep 2026-10-11 — batch R (Rust) CLOSED (R1 to R7 and R9 fixed, R8 not reproduced); batch G (sh | struck in the source row; its own text carries the evidence |
| 3438 | PARTLY | **Defects found while annotating (Ruling BK), not fixed** | OutOfRange newline, `landmark_dict_tests` #[ignore], glacier self-assert, `export_session.rs:37` allow(dead_code), thumbs-not-repositioned all still present; `_sort_by_pop`, `_BULLET_RE`, tool_fan back, app.gd doc block fixed |
| 3462 | OPEN | **Build multi-ridge orogenic belts: `buildOrogenyField`'s multi-sheet belt (v2.36) and its junction fix (v2.35 | re-opened; no code found that closes it |
| 3481 | PARTLY | **Colour editing follow-ups from the 2026-10-10 paint QA, and the look-history gaps** (~~toolbar undo/redo squ | closed halves cited in row (`look_history.gd`, `can_undo_any`); Painter/NPR sliders and layer stack not verified |
| 3482 | PARTLY | **The phone undo chip answers only `bridge.can_undo()`, and CARTO is reachable on the phone only through MORE, | half closed 2026-10-12 per row; CARTO phone route is a design decision |
| 3483 | CLOSED-ALREADY | ~~**G5: the ET-D4 probe fails its check (a) at the default 1600x900 (75/76) because the two-line Way bar moves | struck in the source row; its own text carries the evidence |
| 3484 | CLOSED-ALREADY | ~~**G6: `_TRACE_TERMS` and similar shell rows left over after the planner work**~~ | struck in the source row; its own text carries the evidence |
| 3485 | PARTLY | **Engine calls still reachable mid-generation outside the arm, undo, stamp and tool-bar paths were not swept** | gates swept per row (`_gsw_probe`); remainder per row not re-run |
| 3486 | CLOSED-ALREADY | ~~**`adjust_panel.gd`'s header comment still says `refresh_undo_rows` calls `undo_stats()` unguarded**~~ | struck in the source row; its own text carries the evidence |
| 3487 | OPEN | **Saved measurements may already hold a bogus 0.0-degree bearing and nothing migrates them** | re-opened; no code found that closes it |
| 3488 | OPEN | **G1 caveats: `world_loaded` also fires for an asset pack or a fjord carve, so those restart an open Way or Ro | re-opened; no code found that closes it |
| 3489 | PARTLY | **Tablet density for the batch G / arming-sweep probes: only the existing density probes were run (2026-10-12) | closed halves per row; `_lookstate` 3 failures unexplained |
| 3490 | OPEN | **Probes leak persisted settings into the shared user-data dir (2026-10-18)** — *hermetic pass done for the pr | re-opened; no code found that closes it |
| 3491 | OPEN | **Paint, Sculpt and Way brush controls (ET-D5) are verified only with synthetic input: no real pen or touch ha | re-opened; no code found that closes it |
| 3492 | PARTLY | **Landmark filter and zoom-scaled landmark marks — built 2026-10-19; both owner questions answered by Ruling C | `landmark_filter_panel.gd` exists; Ruling CV answered both questions; the CIVIL > Landmarks dock link is the small residue |
| 3493 | PARTLY | **GUI replacement stages 3, 5, 6 and 7 (menus, right dock and furniture, phone, the nine windows) — `STATUS.md | STATUS RP table (RP-S3/S5/S6/S7 'not started') is stale; symbols exist for each stage (menus.gd, PC-4/PC-10/PC-11, phone_menu.gd, MS-1 S4.5); spec conformance unmeasured |
| 3494 | PARTLY | **Pop-out windows: panels (planner, civilisation windows) on a second monitor** (owner question 2026-10-26: *" | PW-0..PW-5 built (STATUS headings PW-4, PW-5; `popout_host.gd`, `planner_window.gd`); owner-run two-monitor and 6T checks remain |
| 3517 | CLOSED-ALREADY | ~~**A saved journey keeps only its party preset** — Ruling AR: keep the full plan~~ | struck in the source row; its own text carries the evidence |
| 3518 | CLOSED-ALREADY | ~~**Records the code contradicts — STATUS, OUTSTANDING, scope, contract, rulings, architecture**~~ — **CLOSED  | struck in the source row; its own text carries the evidence |
| 3519 | CLOSED-ALREADY | ~~**Record each year's institutions (ag-tech, government) and place overrides in the timeline**~~ | struck in the source row; its own text carries the evidence |
| 3520 | CLOSED-ALREADY | ~~**LOD-D7: deep-zoom tiles for info and debug views**~~ — Ruling AV — **PARTLY CLOSED 2026-10-06: eight views | struck in the source row; its own text carries the evidence |
| 3521 | OPEN | **Smaller gaps the audit found with no row** | re-opened; no code found that closes it |
| 3551 | CLOSED-ALREADY | ~~**Consolidated Faction hub (FH-0 to FH-9, FH-R1)**~~ - **CLOSED 2026-10-11 (verified at the `STATUS.md` FH-0 | struck in the source row; its own text carries the evidence |
| 3552 | CLOSED-ALREADY | ~~**CM-2 follow-ups: the faction rows are absent when the right-click lands on an object; `_ctxpicks` has 2 fa | struck in the source row; its own text carries the evidence |
| 3553 | CLOSED-ALREADY | ~~**The status bar lets a long hint sentence widen the whole shell past a 1600 px window**~~ | struck in the source row; its own text carries the evidence |
| 3554 | OPEN | **CM-4 device check: the pen barrel button and haptics on the owner's tablet** | re-opened; no code found that closes it |
| 3555 | PARTLY | **Custom quick bar: user-pinned commands in the top bar and under the cartwheel, plus a conditional Assistant  | CQ-0..CQ-4, CQ-6, CQ-7, CQ-8 built per row and STATUS; `quick_bar*.gd`; CQ-5 (tablet touch and pen) open |
| 3561 | PARTLY | **Geology first: lithology, layering and age drive every landform process** — Ruling BH; *Ruling BJ: 11 rock t | GF-0..GF-8 built, processes gated off; switch-on is the 3571 row |
| 3562 | OPEN | **Landslip and pinnacle landforms (the Storr / Trotternish)** — Ruling BG — *builds on Ruling BH's geology mod | re-opened; no code found that closes it |
| 3563 | OPEN | **Coastal and lake-shore river combs that remain after Ruling BV** — option 2, draw-side thinning — **BUILT 20 | re-opened; no code found that closes it |
| 3564 | PARTLY | **Fixed-size labels: region default, legacy-import inversion, small leftovers** - follow-ups to Ruling AP | items 1-4 closed per row; item 5 (hit-box staleness after a zoom notch) not re-checked |
| 3565 | PARTLY | **Mountain relief — H1/H2 built 2026-10-19 (`tect.orogen_relief`), kept on by Ruling CV; the three visible def | H1/H2 built per row; fit-scale breadth awaits the multi-ridge row 3462 |
| 3566 | OPEN | **Type-dependent gully spacing and a gully depth capped by the cone** (filed 2026-10-18, Ruling CO item 2; rep | re-opened; no code found that closes it |
| 3567 | OPEN | **Coarse-scale edifice radius floor: at Earth width every volcano is clamped to 2 cells (about 39 km)** (filed | re-opened; no code found that closes it |
| 3568 | PARTLY | **Volcano colour and form by type, age and activity — VA-0…VA-4 and VA-7 built 2026-10-24 (Ruling CW), VA-5 on | VA-0..VA-7 built per row and STATUS; cartographic looks open |
| 3569 | CLOSED-ALREADY | ~~**Terrain-following lava flows: steps 2 and 3**~~ (both built 2026-10-25 under owner Ruling DB items 1 and 2 | struck in the source row; its own text carries the evidence |
| 3570 | CLOSED-ALREADY | ~~**Volcano summits flattened at the soft ceiling**~~ (the "caldera lake cut by a straight edge" on 71077345,  | struck in the source row; its own text carries the evidence |
| 3571 | OPEN | **GF-9's switch-on: re-measure every switch-on bar on today's app worlds, then switch on what passes** (Ruling | re-opened; no code found that closes it |
| 3605 | CLOSED-ALREADY | ~~§20 — the high-precision display pipeline~~ — **CLOSED 2026-09-23 (verified independently)** | struck in the source row; its own text carries the evidence |
| 3606 | CLOSED-ALREADY | ~~**The Village carto preset still looks harsh (yellow/purple/teal)**~~ — **CLOSED 2026-09-28: five flat bands | struck in the source row; its own text carries the evidence |
| 3607 | PARTLY | **Optional LLM assistant through a user-supplied API or local model** (owner question 2026-10-25: *"what if we | LLM-0..LLM-12 shell and crate files exist (`llm_*.gd`, `cartalith-llm`); phone-side follow-ups listed in row remain |
| 3608 | OPEN | **Model list dropdown follow-ups** (LLM-12; owner request 2026-10-26) | re-opened; no code found that closes it |
| 3609 | OPEN | **Assistant profiles follow-ups** (Ruling DQ; owner request 2026-10-26) | re-opened; no code found that closes it |
| 3610 | DONE | **Vault note list: a visible `private` badge** (Ruling DP, optional) | `vault_privacy.gd`; `vault_window.gd` private-badge block over the browse tree rows; STATUS 'VE-9 finished' |
| 3611 | OPEN | **Grounded place prose follow-ups** (LLM-11; owner rule 2026-10-26) | re-opened; no code found that closes it |
| 3612 | OPEN | **Small Cartalith-tailored LLM as a separate add-on, no Ollama** (owner idea 2026-10-26) | re-opened; no code found that closes it |
| 3613 | PARTLY | **Edit actions over a painted region: follow-ups** (LLM-7; owner idea 2026-10-25, scheduled 2026-10-26, Ruling | `region_edit.rs`, `llm_region.gd`, `llm_edit.gd` exist; remaining ops per row |
| 3614 | PARTLY | **Retrieval knowledge base: the assistant answers from the Vault AND the world's own data, with WORLD DATA / V | `cartalith-knowledge` crate, RK-0..RK-7 per row; RK-8..RK-11 open |
| 3615 | PARTLY | **FH-R2 — religion expansion S1 to S7: R2-0 to R2-3 and R2-6 built (Rulings CG, CJ; R2-3 and R2-6 at Q6's and  | R2-0..R2-3, R2-6 built per row; R2-5 unscheduled; cost re-measure owed |
| 3616 | DONE | **G1 — government forms: six core plus two optional, with authored manpower rows** (`FH_R2_PLAN.md` Q9; kept o | `roster.rs` CIV_GOVERNMENTS; `manpower.rs` GOVERNMENT_EXTRACTION/CITIZEN_MODERNISATION/SOLDIER_UPKEEP_BY_BRACKET/GOVERNMENT_LEVY_BONUS; test `the_khanate_levy_bonus_is_derived_from_its_era_band`; `g1_baseline.rs::g1_0_baseline_columns`; `faction_roster_window.gd::_government_choice`; `_gov_probe.gd`; commit 61fc1585; STATUS Governments ledger G1-0..G1-5 |
| 3617 | CLOSED-ALREADY | ~~**Reference reading of the credited edge, `(cx+0.5)/sc`, NOT adopted: owner question**~~ — **CLOSED 2026-11- | struck in the source row; its own text carries the evidence |
| 3618 | CLOSED-ALREADY | ~~**River Transport: gate navigability on discharge as well as Strahler order? (owner question), and the unmea | struck in the source row; its own text carries the evidence |
| 3619 | OPEN | **GF-9, the geology re-baseline and scrub — ~~parked until the owner switches the geology processes on~~ no lo | re-opened; no code found that closes it |
| 3620 | OPEN | **GF-8's three Rock draft layers are not saved in `drafts/paint.json`** | re-opened; no code found that closes it |
| 3621 | OPEN | **Story menus: the proposal's §8 later rows** | re-opened; no code found that closes it |
| 3627 | CLOSED-ALREADY | ~~**LOD-D4 · Ice and snow from fields that already exist**~~ (Ruling K, step 2) — **CLOSED 2026-09-21 (verifie | struck in the source row; its own text carries the evidence |
| 3647 | OPEN | **BUILD_ANSWERS §3** — the Data-manager window ~~and 13 of 24 asset families~~ are absent from the new Environ | re-opened; no code found that closes it |
| 3648 | PARTLY | **BUILD_ANSWERS §4** — ~~phone generation-failure and storage-full states are undesigned;~~ content descriptio | `_save_blocked_by_space`, `_report_failure` per row; content descriptions, dynamic type, 48 dp sweep open |
| 3649 | PARTLY | **DS-13** — the phone viewport control column (zoom/pan/navpad) redesign | items 1-6 built (STATUS MS-5 headings); item 7 (3D camera) is 3D-2's |
| 3650 | PARTLY | **Phone findings of 2026-10-26 that were observed and not fixed** | item (1) fixed per row; items 2-5 unverified on glass |
| 3651 | PARTLY | **Phone touch-scroll: the central fix (owner, 2026-10-26)** | built 2026-11-08 (STATUS 'Central touch-scroll fix built'); handset confirmation open; Asset library survey rows |
| 3652 | OPEN | Observe the §13 phone **landscape** composition on the device | re-opened; no code found that closes it |
| 3653 | OPEN | **§47** — hi-DPI blur confirmed only to `_phone_scale` 2.748, not the owner's 3.664 | re-opened; no code found that closes it |
| 3654 | OPEN | **GPU §21** — thermal / mobile-adaptive GPU scheduling | re-opened; no code found that closes it |
| 3655 | OPEN | Vault **milestone 4** — device pass verifying the Android SAF provider (folder picker, persisted grant, revoca | re-opened; no code found that closes it |
| 3656 | PARTLY | **PC menus and keys: what is left after PC-9 and PC-11** — the drafts (01–11) are in `shell/` since 2026-11-07 | PC-9/PC-11 and follow-ups built (STATUS headings 2026-11-07/08); re-run probes after later milestones |
| 3657 | PARTLY | **Implement the mobile menus** — owner, 2026-11-01: *"It should be outstanding work that these newly designed  | MS-2, MS-4, MS-6 and MS-7 built (STATUS headings; commits 3b04e444, 263a7b26); MS-11 quick drawer not built; no 'Share project' action in `phone_menu.gd` or `menus.gd` |
| 3658 | PARTLY | **PC shell: one selection, one year, the Context panel (Inspect · Note · Ask and ten modules), windows that fo | PC-1..PC-11 built per STATUS headings; the PC-5/6/7/8/10 leftovers listed in the row |
| 3659 | PARTLY | **The assistant's action catalogue: every action reachable without flooding the context** — design approved an | AC-1..AC-5 and AC-7 built per row; AC-6 unsupported by live numbers |
| 3660 | PARTLY | **Vault editor and linking to a regular Markdown editor's level** — proposal of 2026-10-31; owner questions VQ | VE-0..VE-9 built (STATUS headings); SAF move, glass pass open |

## Sections 4 to 7

* **Section 4 (owner decisions).** Items 1 to 11 are struck as answered (Ruling EA and earlier); rulings after EA (EB to ET)
  answer none of items 12 to 15 (grep of `LARGE_ITEM_RULINGS.md` for the press `.icc`, telemetry and page-file items finds only
  the earlier deferrals). Verdict: items 12 (press `.icc`), 13 (telemetry) and 15 (page file) OPEN, owner-held; item 14 (attach
  the 6T and tablet) OPEN, owner-held, and the 6T was used 2026-11-08 for an adb swipe smoke (`STATUS.md`), so it is attached at
  times. The 2026-10-11 block below is history.
* **Section 5 (declined and shelved).** Not gaps by definition. One entry was stale and already says so in its own text: "The 3D
  viewport, and all 3D work" (approved by Ruling CS item 4; `view3d.gd`, `view3d_gate.gd` exist, `STATUS.md` has 3D-1). No edit.
* **Section 6.** 6.1 to 6.7 and 6.9 are struck as closed. 6.8 first two bullets resolved (above); the urban `_um*` bullet is
  marked resolved in its own text; the remaining bullets are historical counts.
* **Section 7 (what this document does not cover).** Descriptive; no row to close. Its "no surveyor ran `cargo test`" statement
  holds for this audit too: **no tests were run** (no code was changed and the task was a documentation reconciliation).

## The biggest truly open items

1. **Multi-ridge orogenic belts** (2.9; `cartalith-terrain` has no multi-sheet fields) and the fit-scale mountain breadth that waits on them.
2. **GF-9's switch-on re-measure** (2.13) and everything gated behind it (GF-8's Rock drafts, GF-4's coastal flip, GF-9's scrub).
3. **Landslip and pinnacle landforms** (Ruling BG): no scope document exists yet; "scope it first" is the whole next step.
4. **Volcano edifice rows**: gully spacing keyed in km (`EDIFICE_GULLY_SPACING_CELLS` is still 4 cells, cap 48) and the 2-cell radius floor (`rad_cells.max(2.0)` in `stamp_one_volcano`).
5. **The v2.71 reference function index and drift file** (2.8): `reference/` holds the file but no index or drift map.
6. **Device passes owed on the 6T and the Pad**: touch scroll, boot stall (~9.4 s, unattributed), idle power, SAF vault, CQ-6 and CQ-5, MS-10.
7. **MS-11 phone quick drawer, "Share project", and the tablet shell for 900 dp and over** (`W_DOCK_TABLET`, palette, `scrPlanner`).
8. **Ruling BK annotation pass** over existing code (2.8) and the five residual annotation-time defects.
