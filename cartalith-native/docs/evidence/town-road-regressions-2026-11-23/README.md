# Roads meet their towns (owner report 2026-11-23)

The owner's report on v.26.11.23-001: *"routes seem to miss settlements and connect to points they no longer sit"*, then
(coordinator, after a clean generation) *"the attachment from routes (sea and land) to the settlements when you zoom in"*.
The state, the cause and the bar table are in `cartalith-native/docs/STATUS.md` ("Roads meet their towns"). This folder
holds the evidence that entry was read from.

| File | What it is |
|---|---|
| `bars.txt`, `bars.py` | The bar table, before (the build of `e74e7312`, `civ.bank_sites` on) and after (this change, on), from `_roadattach_probe.gd`'s summary lines on the six census worlds (483920, 912345, 246371 and 83744 at 384 x 288; 483920 at 2 048 x 1 536; the owner's 532187 at 2 048 x 1 311 from `C:/cw/owner_repro/seed532187_params.txt`) and seed 51203 at 4 096 x 2 048 at the app's defaults (the owner's Pad world size). |
| `probe_before_*.txt`, `probe_after_*.txt` | The probe's full output per world (`--rows`). The "before" runs read the town's river line for crossings (that build has no `drawn_river_crosses`), so their "made" column is a different measure. |
| `drive_after_4096.txt`, `drive_after_384.txt` | `tools/road_attach_drive.gd` in the real shell (desktop 1 600 x 900): at x4 every named road end is drawn within 4 px of its marker; at x16 and x64 within 4 px of its town's entry as the overlay draws the plan. |
| `census_clean_4096.txt` | `_towncensus_probe.gd` on 51203 at 4 096 x 2 048, app defaults: the clean-world record for the walls and pieces part of the report. |
| `census_owner_mix_*.txt` | The same census on the owner's world file `seed51203_params.txt` (`civ.bank_sites` on with `urban_river_smooth` and three other urban switches off): `owner` as sent, `B` with `urban_river_smooth` on, `D` with the nine switches at the app's defaults. |
| `mutants.py`, `mutants_result.txt` | Literal mutants: six on the unit suite, three built into the dll against the probe on 483920 at 384. All killed. |
| `drive_mutant.py`, `drive_mutant_result.txt` | The overlay's mutant (a named end not drawn at its marker below the town layer) against the drive. |
| `workspace_test.txt`, `baseline_test.txt` | `cargo test --workspace --no-fail-fast` and the 12 ignored `fh_r2_baseline`/`g1_baseline` tests on the final tree. |
| `combined.png`, `combine.py` | The frames: per town, BEFORE (`e74e7312`, switch on; its frame script takes no City Viewer, the grey cell) and AFTER (switch on), map x4, x16, x64 and the City Viewer. 51203 at 4 096 (#24, #130, #14, #1) and 483920 at 384 (#19, #0, #10; the drive's #1 at 384 stands at the map's edge and its crops are off the map, so it is left out). The single frames were not kept in the tree (11 MB). |

How to run: the probe, `Godot_v4.7.2-stable_win64_console.exe --headless --path <project copy> --script
res://_roadattach_probe.gd -- --seeds 483920 --rows` (its header lists the arguments); the drive through
`cartalith-native/tools/fennara_drive.py` with `RA_OUT`, `RA_SEED`, `RA_GW`, `RA_GH`, `RA_TOWNS` (its header). A project
copy made from a fresh worktree needs the owner's `*.import` files and `.godot` folder copied in (`MISTAKES.md`).
