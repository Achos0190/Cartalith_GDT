# Outstanding work

> **A router, not a status record.** Each line says what is left, who owns it, and the symbol to check.
> **`cartalith-native/docs/STATUS.md` wins on any status**; a line here that disagrees with it is a bug in this file.
> History, evidence, struck rows, count logs and the contradiction log are in `OUTSTANDING_WORK_ARCHIVE_2026-11-22.md`
> (old "`OUTSTANDING_WORK.md` §N" citations resolve there). Re-open a line at its symbol before scheduling it
> (`MISTAKES.md` preflight). When a line is done, delete it here and record it in `STATUS.md`. No "done" column.

Rebuilt from the archive on 2026-11-22 at the owner's request (*"Maybe only keep the things that really need to be
done on the list"*). "(verify)" marks a line that is probably done but not proven at its symbol.

## Next, in order (owner)

1. **Parallel streets: PS-5 with the back-street tier and the water gate (Ruling HD)** (fix E built and the water gate a no-go, scope section 17; PS-3 no-go `4d540795`, PS-4 no-go `68c45f38`; Rulings GY and HB). PS-5 carries the riverside bar and the needles; Q26 and Q27 answered (Ruling HD: the water gate is rebuilt inside PS-5; E's 2 048 misses accepted). Owner `PARALLEL_STREETS_SCOPE.md` sections 14-17; switch `urban_parallel_streets` (off).
2. **River follow-ups:** RR-T1 and RR-T2's town side are built and pushed (`urban_river_smooth`). Left: (a) the harbour-front sea extension (Ruling HC item 3): smooth the `to_sea` river of a river-mouth town too, same switch; (b) RR-T2 in the app: a world with two or more stream towns to measure quay and port counts and frame them (seed 483920 has none: its 20 undrawn river towns are merged `parallel_of` runs, not streams), and "not river-front typed"; (c) re-run `_rivertown_probe.gd` with `--mutate off` (not yet run). Owner `RIVER_REALISM_SCOPE.md` RR-T1, RR-T2. (c) the 20 towns per world on a merged `parallel_of` run keep the raw path at river width while the map draws the same water on the parent run 1-2 cells away: bind them to the parent drawn run (coordinator, 2026-11-22); (d) the probe's `--mutate off` run and the re-run of its edited ONE CURVE bar.
2a. **Ruling HN (HM ships; owner 2026-11-23):** close HM's misses (housed p10 on 83744 and the owner's 532187; the last markers across from their market; walls and streets over the drawn stroke); an older world offers once to turn `civ.bank_sites` on (frozen towns untouched); town lake and sea edges match the map's drawn water; the river look's slider never moves towns. Agent's leftovers: (a) the marker reads the town's point, not the laid-out market, so 4 of 1 230 census towns still have marker and market on opposite banks (a market-only path through `cartalith_urban::generate` would close it); (b) housed p10 fell on two of six worlds and a handful of towns lost most of their housing (a river's end, a single crossing road, capitals short of bank land); (c) lake and sea edges against what the map draws are unmeasured; (d) a saved world loads the switch off: decide whether opening an old world should offer it; (e) `town_layout_bridge::town_at` still measures a town's reach from its cell, so a pin dropped on the far side of a town moved several hundred metres may not attach to it; (f) `town_origin` runs for every settlement on each zoom step, untimed.
3. **Builds only when the owner asks** (owner 2026-11-23). Devices and the D: folder are on v.26.11.22-002; `version.txt` already reads v.26.11.23-001. Waiting for a device: TL-5's arm64 re-lay and TR-7's trace file.
4. **The small open rows** (each under its area below): F1 measure-to-journey, the trace on a device (TR-7), Ruling GB's leftover towns, name-style
   leftovers.
5. **The menu and function placement audit** (owner, 2026-11-17; Sonnet walks, Fable verifies): map every function to
   every home on desktop, tablet and phone, and report doubles, misplacements and dead entries. Last before the refactor.
6. **The codebase refactor** (`CODEBASE_REFACTOR_PROPOSAL.md`; inventory `CODEBASE_INVENTORY.md`). It absorbs the
   serial generation floor (Ruling FL item 3, see Performance).
7. **World-scale terrain** (`WORLD_SCALE_TERRAIN_PLAN.md`; parked until the refactor). It absorbs ZR's no-go decision
   and FG's open bars (see Terrain).
   - Ruling GV (cut-out modes and provenance; folded into the plan, parked with it): a `region_source` manifest, a generation-rules version and field digest in every save, cell units in ground units, a warning on Generate over a region world, Regenerate region from source (plan §9). Free B (Re-simulate region) dropped by the owner.

After the whole list: the **performance pass across devices** (owner, 2026-11-08), then the LLM line (priority order:
GUI, then engine, then the LLM interface).

## Open work by area

### Timeline and story
- `generate` right after opening a saved project prints "Bug: Invalid call error code 1337" (generation completes; found 2026-11-22 by the HF probe, `3b3fa4a9`; not investigated).
- Junction hamlets in the live list carry founding years 100-300; moving the cursor back and re-snapshotting there gives that year those hamlets. `adopt_junction_hamlets`.

### Towns and urban layout
- Ruling GB leftovers: 483920 #8 (59 %), #11 (65 %), 912345 #0 (68 %) at 2 048 under 70 % housed; 246371 #0 thins
  east; #11's unbuilt wedge; over-housing (912345 #30, census #2); wall/hull max 1.25; the 1.6 cap binding on 246371 #0. `urban_growth_water_widen`, `urban_town_bridge_search`.
- Ruling FK leftovers: harbours still over 23 m from water (483920 #5 at 384, #10 at 2 048); hamlet 154:136 about 400 m inland. `urban_harbour_front`, `civ.port_reach`.
- Settlement #6 with `urban_riverbank` on: growth gathers at the box edge (river port far from the town). `grow`'s `d_m`.
- City Viewer: a bay town's smoothed shore has no shallow band of its own. `urban_shore.gd`.
- Ruling EX: `um_site_profile`'s half-cell river distance (golden-pinned; needs a re-baseline ruling); the Map view's suitability layer shows the frontage term off. `sample_bridge.rs`.
- Town pins: the painted icon set has no `temple`; a landmark hidden by Ruling FC still suppresses its generated icon at its cell. `PinMarker.KIND_ICON`.
- `map_overlay.gd`'s `_town_half_km` comment still says "planned box"; `forget_urban_layout(index)` does not re-read the boxes.
- Landmark M8: six kinds still `buildable: false` (saddle, river crossing, temple, pilgrimage site, tomb, monument; the four cultural ones need designs). `cartalith-civ/src/landmark.rs` `kinds()`.
- Landmark M9 remainder (verify). `LANDMARK_GENERATION_SCOPE.md`.
- `ALIGNMENT_AUDIT.md` D29/H gaps: economy district fills (oreyard, fishery, sawyard, granary, warehouse), generated-but-undrawn urban layers, region export omits `params.json`, `snow_aspect_c` has no control.

### Rivers, water and terrain generation
- River combs after Ruling EV: the stale-status fix (snapshot `WorldGen::stages` in `restore_new_world_rules`, restore in `revert_new_world_rules`) and the comb follow-ups. `DELTA_GEOMETRY_RESEARCH.md` §14.6; `comb_host`.
- Ruling EV: basin capture affects 0.05-0.09 % of land (two candidate designs, `DELTA_GEOMETRY_RESEARCH.md` §14.8); small knots where rivers meet.
- `tile::tile_rivers` still uses the reference ease. `cartalith-hydrology`.
- River realism RR milestones not yet built (verify against `STATUS.md`). `RIVER_REALISM_SCOPE.md`.
- Lake balance: the owner's 51203 window not re-measured; removed share 93/65/65 % vs harness 74-90 % unexplained; never looked at windowed. `apply_lake_balance`.
- Multi-ridge orogenic belts (v2.36 multi-sheet belt, v2.35 junction fix): scheduled (Ruling AP), not built; scope into batches first. `RC_ENGINE_CHANGES.md` §6b.1-2; `cartalith-terrain` (no multi-sheet field exists). Fit-scale mountain breadth waits on it.
- GF-9 switch-on: re-measure every switch-on bar on today's app worlds, switch on what passes (Ruling EA item 5); GF-8's Rock drafts saved with it. `GEOLOGY_FIRST_SCOPE.md` §5.2, §7; `geology_processes`.
- Landslip and pinnacle landforms (Ruling BG): write the scope first. No scope document exists.
- Volcano gully spacing by type, depth capped by the cone (Ruling CO item 2; measure first). `EDIFICE_GULLY_SPACING_CELLS` (still 4 cells).
- Edifice radius floor at Earth width (Ruling CO item 4; measure first, then options to the owner). `stamp_one_volcano` `rad_cells.max(2.0)`.
- Volcano leftovers: supervolcano caldera lake an exact ellipse at x4; maars rare from site refusals. `VOLCANO_APPEARANCE_RESEARCH.md`.
- Peak law: re-fit the three constants only if the owner wants taller defaults. `cartalith_engine::derived_peak_m`.
- Ruling FF leftover: road crossings on low-flow map river cells cost nothing (gate on order instead moves every road; owner call). `civ_river_crossing_cost`.

### Rendering, 3D and export
- 3D-4 to 3D-8 (lit drape, sea and atmosphere first). `3D_VIEWPORT_SCOPE.md`; `shell/view3d*.gd`. 3D-3's CDLOD morph has no measurable effect (keep or drop is the owner's).
- Render target gap list: the open gaps. `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md`.
- Ruling FG bars B1, B4, B5 (x4-x32) need real sub-cell landforms; goes with world-scale terrain.
- Snow aspect term: acceptance measure (LOD-D4 bar 1b) not met and found biased. `TerrainAppearance::snow_aspect_c`.
- Deep-zoom diamond lattice: the owner's re-check on a real world; candidate (c) only if residue remains. `relief_lowpass`.
- Export: a windowed probe that ET-C3's overlay stack reaches the exported overlay pixels. `shell/export_overlay.gd`.
- Export: an owner look at `EXPORT_URBAN_MIN_BOX_PX` (75 px: no town plan at any width on a 1 200 km world); Android tile/frame measurement. `map_overlay.gd`; `EXPORT_SCOPE.md`.
- ET-C2: the timing and the river-bank seam. `render.rs` `finish_rgb`.
- Curves: the end-to-end live-drag timing in Godot. `adjust.rs`.
- ET-D2/D3: pen samples before the `is_pen` latch lose pressure; `PaintEditor` has no shell-facing undo `#[func]`. `paint_bridge.rs`.
- ET-D4: time a 200-waypoint drag commit on a 2048 world; `redo_labels` can undercount. `infra_tools_bridge.rs`.
- Colour paint: the §4 local-contrast halo measurement. `COLOUR_PAINT_SCOPE.md`.
- Colour editing: phone/touch scaling of the new picker rows; the 298x624 popup's buttons at the window edge; `ramp_strength` default 0 (owner call). `adjust_panel.gd`.
- Zoom residual: about 8 frames over 16.7 ms per notch run, one ~27 ms first-notch frame (verify). `viewport_host.gd`.
- Raisz pictograms, mappa-mundi icons, Ordnance Survey symbology (Ruling BI): each needs a scope first.

### Civilisation, routes, planner and military
- F1: Measure > export to Journey turned 897 km into 206 km; not reproduced, needs the owner's Measure mode and points. `right_dock.gd::_on_plan_journey`, `WorldGen::measure_result`.
- R8: seed 707's ~0.1-day sea route, not reproduced in Rust; print `route_get(i).points` in the shell. `journey_planner_view.gd`.
- Military: on industrial rows a high-state-capacity standing army above its band (Ruling DL Q2, measure first). `manpower.rs`; `ECOLOGICAL_ARMY_TERM_SCOPE.md` §10.7.
- Military: DL Q8, the `improvedAgrarian` levy below its band (measure first). `ECOLOGICAL_ARMY_TERM_SCOPE.md` §10.8.
- FH-R2: re-measure M2's cost bar on today's tree by M2's own method (Ruling EA item 4). `FH_R2_PLAN.md` §6.
- G1 caveat: `world_loaded` from an asset pack or fjord carve restarts an open Way/Route draft; File > Open path unprobed. `_redraft_after_world_change`.
- Context card: faction rows only when the click hits no object; `_ctxpicks` fails 2 checks. `civilization_workspace.gd`.

### Desktop, tablet and phone shell
- **Ruling HO:** the tablet starts on the full-screen worlds grid (`scrPicker`, `TABLET_UI_SPEC.md`) instead of the modal welcome card.
- Phone: the undo chip (bottom-left of `_phone_content_gap`) overlaps the tool strip's tool chip at 450x975 whenever any step exists (frame `docs/evidence/shell-batch-b-2026-11-23/after/undo_chip_year.png`: it covers the "IN" of INSPECT); it also does not refresh on a planner stop press (`stops_*` has no change signal). `dcc_shell.gd::_build_phone_undo_chip`, `tool_strip.gd`.
- Touch scroll, still blocking after shell batch B (measured, `tools/shell_batch_b_census_drive.gd`, tablet 1700x1000): a plain `LineEdit` in the Vault window (a drag that starts on it scrolls nothing); `ramp_bar.gd` (owns the drag on purpose); `HSlider`s and the phone's window `SpinBox`es were not on screen to measure in any state visited. Not measured at all: any window the census did not open.
- Owner question, PC vs touch world from one seed: the only form-dependent input is the boot default of `use_gpu` (`EngineBridge.default_gpu_on`: desktop `true`, touch `false`, Ruling EY and Ruling EZ item 2), and the GPU path makes another world (seed 4242 at 256x160: 26 % of height-export pixels differ, at most 3.3 % of the range; 597 vs 596 rivers; 3 of 54 labels; settlements equal). With the stored choice the same, the three forms are byte-identical. Options A-D (keep EY, CPU default everywhere, EZ item 3, record the path in saves) are in `cartalith-native/docs/evidence/form-determinism-2026-11-23/README.md`; TL-5 depends on the answer. Check: `cartalith-native/tools/form_determinism.py all`.
- Phone: `chronos_items.gd::_start_scan` calls `WorldGen.timelines_scan()` while a generate worker holds the engine (`Gd<T>::bind() failed, already bound`, `SCRIPT ERROR ... 1337`), in 4 of 7 phone Creates measured 2026-11-23 and none on desktop or tablet; no dump changed. Same evidence directory.
- Phone: `+`/`-` fine-adjust on sliders and the Splat/Biomes probes (Ruling EH item 1 follow-ups; owner decision pending).
- Phone 2026-10-26 finding, device only: SwiftKey IME duplication when editing the Save-as name with `adb` key events (unverified with a human typist). `dcc_shell.gd`.
- GUI replacement stages 3, 5, 6 and 7: re-open each spec against the shell and close or narrow (verify; RP-S6 superseded). `design/dcc-environment-2026-08-31/spec/`.
- BUILD_ANSWERS §3: Data manager screenshots for the owner's approval. `data_manager_window.gd`.
- BUILD_ANSWERS §4: content descriptions, dynamic type, the 48 dp sweep, screenshots.
- `DCC_SHELL_SPEC.md` §11: the 51 stale citations (as parity rows).
- Map context: F5 ring slot replacement (later). `MAP_CONTEXT_SCOPE.md`.
- Point Godot's remote debugger at the Android build to read the node tree off the device (owner suggestion). `ANDROID_BUILD_SCOPE.md`.
- UI copy pass, remaining surfaces: listed at the end of `cartalith-native/docs/evidence/ui-copy-2026-10-26/COPY_CHANGES.md`.

### Vault and save format
- Vault window editor toolbar "clipped at the right edge at 1600x900" (found 2026-11-22): NOT reproduced on 2026-11-22d. `tools/vault_toolbar_drive.gd` (desktop, tablet; the phone toolbar rides on the keyboard) finds every button inside the editor on the vault window (it wraps: an `HFlowContainer`) and docked in the Note tab; frames in `docs/evidence/small-fixes-2026-11-22d/`. Re-open if the clip is seen again, with the state (note, mode, window size).
- Ruling GZ's device leg (TL-5): freeze the 64 census towns on x86-64, re-lay them on the 6T or the Pad with the next APK, expect 64 of 64 equal or "same within rounding". `TOWN_LAYOUT_SAVE_SCOPE.md` §8.
- VE-4 SAF side: write atomicity and read-only behaviour; an outside-change check on focus. `vault_saf.rs`.
- Ruling FH: the save prompt for refined levels and the disk-cap setting (parked with world-scale terrain, since ZR is a no-go).

### Assistant (LLM) — after GUI and engine
- `llm.first_byte` has no probe assertion of its own: it was checked end to end by running `_llmpanel_probe` with `CARTALITH_TRACE=info` and grepping the trace file (10 records, no prompt, key or note text), but renaming it leaves the probes green.
- Trace on a device (TR-7, `TOOLING_PLAN.md`): the crate and `cartalith-godot` build for Android, but the file under the app directory pulled with `adb` is unproven (no device was used).
- Live runs on tablet and phone, Describe-place prose, the hosted wire. `LLM_ASSISTANT_SCOPE.md`.
- Region edits: a multi-zoom look at each stamp, a ridge chord across strokes, a per-cell cost, a quality bar for real models; the noise operation. `LLM_ASSISTANT_SCOPE.md` LLM-7, s.5h.
- Undo-and-regenerate: hand edits dropped by Apply's discard question are not restored.
- Rust request builder has no private-note check of its own (the shell is the one gate). `llm_stream.rs`.
- Action catalogue: review the `unreviewed` rows; a name-to-settlement lookup; AC-6 (a forcing harness first). `ASSISTANT_ACTIONS_PROPOSAL.md`.
- Model list: record real list replies (OpenAI, OpenRouter, Gemini, vLLM); embedding filter; a tools mark. `models.rs`.
- Profiles: phone quick-bar/ring switch; per-profile `rag_pre`/`rag_tools`. `llm_profiles.gd`.
- Grounded prose: the third-of-runs dead Send click; empty 25 km neighbour facts without villages. `place_facts.rs`.
- Retrieval RK-8 to RK-11 live runs (owner-run model passes). `RAG_KNOWLEDGE_BASE_RESEARCH.md` §8.4.
- Consent: a real LM Studio, a remote host and a device. `llm_consent.gd`.

### Performance, GPU and memory
- Serial generation floor (priority floods, in-place jump floods, D8): goes into the refactor (Ruling FL item 3); each needs a ruling and re-baseline. `PERFORMANCE_BENCHMARKS.md` §12.6.
- Pad 3: the `RAYON_NUM_THREADS=1` column; in-app timing of `absorb()`, `warm_render_caches()` and first draw (`bridge.last_generate_ms`).
- GPU: the Erode op's thermal pass not checked across devices; the integrated Radeon not re-run at 16k. `erode_op`.
- 16k/32k: 32k not completed (commit limit); the unbroken-out 150 s of Tectonics; GPU at 32k (2 GiB limit). `PERFORMANCE_BENCHMARKS.md` §13.
- Stream power on GPU: SP-G0 measure first. `GPU_STREAM_POWER_SCOPE.md`.
- `build_water_bodies` on GPU (large, research). `GPU_LAYER_INTEGRATION_SCOPE.md`.
- Tiled GPU compute beyond `gpu_warp`'s band split. `HARDWARE_ACCELERATION.md` §18.
- EF-9: confirm what in `ef9_bridge.rs` is dead before removing; EF-9.5 has no function yet.
- Memory: lower the suitability plateau only if the handset peak says so. `MEMORY_OPTIMIZATION_SCOPE.md`.
- Search field lag of 10-40 s on the 6T (not reproduced on desktop): time a keystroke on the device.

### Discipline, tests and documents
- Ruling BK: annotate existing code for a human auditor (large; batch log in the archive). `find_gaps.py`.
- Defects found while annotating: `LoadError::OutOfRange`'s literal newline and the rest. `STATUS.md` "Rust annotation pass".
- `site::tests::golden_build_site` fails in release (a NaN's sign bit). `crates/cartalith-urban/src/site/tests.rs`.
- Reference: v2.71's function index and drift file (the file is in `reference/`; neither exists). `RC_ENGINE_CHANGES.md`.
- Test hygiene: the staging file `MISTAKES_CANDIDATES_2026-10-26.md` remains.
- Gate: `cargo test --workspace` after any Rust change (newest entry in `STATUS.md`); probes are separate.

### Device passes (owner-run)
- MS-10 on the 6T, a tablet and a 4K window; MS-1 B7, B9 export half, B10. `MOBILE_SHELL_PROPOSAL.md`.
- VE-0, VE-4 (SAF), VE-8 (toolbar over the real keyboard, both orientations), VE-7's Android picture read; Vault milestone 4 (`vault_saf_android.gd` never run).
- CQ-5 on the Pad; CQ-6 6T bars; CQ-8's Android share sheet and picker; CM-4 pen barrel and haptics; F5b edge sliders on the Pad.
- Pen: SP-0, SP-2 to SP-7 and the Tool fan target (`STYLO2_PEN_PROPOSAL.md` §11); the Stylo 2's pressure, tilt, hover and palm through the Stylus tester; SH-2's `disable_scroll_deadzone` (measure first); ET-D5 brush controls on real pen and touch.
- The VIBRATE permission on the 6T (`adb logcat | grep -i "VIBRATE permission"`); the rest of SH-0 (`wm density`, `getevent -lt`, `dumpsys input`).
- Welcome idle-mode fix, menu-drag fix, central touch-scroll fix and the two scroll reports on the handset.
- The 3D gate on a slow device (3D-0(c), 3D-7); CP-4 colour paint on glass; E5 export dialog on a phone; AP-8 folder import; EF-9 timing on the 6T.
- Pop-outs on monitor 2 (planner, dialogs, OS Ctrl+Z/Y/S). `POPOUT_WINDOWS_SCOPE.md` §12, §14.
- Idle power on the device; the ~9.4 s boot stall (needs a profileable build); the generation peak on the handset; R1's Godot hunk; six features never driven since 2026-08-24; phone landscape re-capture.
- The phone glass checks left by 2026-11-13..17 work (coach tips, card buttons, memory readout, scale bar, long-press pin, asset library persistence, peek sheet tap-through).
- Three-platform design-conformance verification with Fable, when all GUI work is done (owner, 2026-09-05). `DESIGN_HANDOFF.md`.

## Blocked

| Item | Blocked on |
|---|---|
| Zoom refinement ZR-1 on; Ruling FH's save prompt and disk cap | owner decision after ZR-0's no-go (close ZR, or scope a coarse-level landform process); folded into world-scale terrain |
| World-scale terrain, FG's B1/B4/B5 | the refactor (owner order) |
| Serial generation floor | the refactor (Ruling FL item 3) |
| GF-8's Rock drafts in `drafts/paint.json` | the GF-9 switch-on (Ruling EA item 6) |
| Fit-scale mountain breadth | multi-ridge orogenic belts |
| `cartalith-gpu` device throughput tests; one `cargo test --workspace` run without the low-memory guard | the owner enlarging the page file |
| Press `.icc` loader | a real press profile (Ruling CF item 3) |
| §47 hi-DPI at `_phone_scale` 3.664 | a OnePlus 12 on the bench |
| GPU §21 thermal/mobile scheduling | an Android GPU compute path (none exists) |
| Pad 3 felt relief | the Pad has no vibration motor; OnePlus pen SDK or sound/visual cues (`STYLUS_HAPTICS_RESEARCH.md` Q10, Q11) |
| Every device pass above | the owner attaching the 6T and the Pad |

## Held by the owner (not work until the owner says)

- HTML working copy stage 2 (phone and tablet shells). `html/README.md` — no HTML work for now (owner, 2026-11-08).
- HTML stage 3 (Context modules, planner window). `html/README.md`.
- HTML versions v2.74 to v2.77 by Ruling EF (fix pass, speed pass, three ports, Measure). `html/audits/v2.72-2026-11-01/`.
- Moving each HTML version into `Cartalith_RC` (owner's own step).
- HTML stage-1 findings: hidden `#assetsPanel` in Assets mode; the place editor over the timeline strip at 1 366 x 768.
- Haptic feedback SH-7, SH-8, SH-12 and the Q13 pen-motor experiment (parked 2026-11-02).
- SH-5's Rust half (`tilt_size`/`tilt_min`) and sculpt's inner hover ring (Ruling EE scope; not scheduled).
- Performance telemetry (Ruling AZ: parked).
- Cartalith-specific small LLM, Qwen3.5-4B (Ruling EP: after the rest of Cartalith); SL-0 onward; MiniMind idea. `SMALL_LLM_ADDON_RESEARCH.md`.
- FH-R2 S3b (R2-5) and S7 world-instanced religions (unscheduled, Ruling EA). `FH_R2_PLAN.md`.
- Story menus §8 later rows (Ruling EA item 7: "None for now"). `STORY_MENUS_PROPOSAL.md` §8.
- Deep plots (`urban_burgage`): off; Q5-Q8 default drop. `TOWN_GENERATOR_JUDGEMENT.md`.
- River stretch variants beyond Ruling FS's shipped setting. `urban_river_stretch`.

## Owner questions still owed

Checked against `LARGE_ITEM_RULINGS.md` through Ruling HJ (2026-11-22); numbering kept from the rebuild, answered items removed.

1. **Zoom refinement after ZR-0's no-go:** close ZR, or scope a coarse-level landform process? (`ZOOM_REFINEMENT_SCOPE.md` §8.)
2. **Deep plots Q5-Q8** (`TOWN_GENERATOR_JUDGEMENT.md`): default drop.
3. **Ruling GB's 1.6 cap** on 246371 #0: keep or raise.
5. **Ruling FC's reading** of "the map view" as the City Viewer: confirm.
6. **3D-3's CDLOD morph:** keep or drop.
7. **Ruling EH item 1 follow-ups:** phone slider `+`/`-` fine-adjust.
9. **`ramp_strength` default 0:** keep (ramp edits invisible until Colour relief is raised).
11. **16k/32k:** raise the page file, or run 32k only after tiling.
12. **Small LLM Q-SL-1 to Q-SL-10** (held until after the rest).
13. **Peak law re-fit:** only if taller defaults are wanted.
