# Ruling HM: river towns on the drawn river's bank (2026-11-23)

The state and the bar table are in `cartalith-native/docs/STATUS.md` ("River towns stand on the drawn river's bank").
This folder holds what that entry was read from.

| File | What it is |
|---|---|
| `census_384.txt` | `_towncensus_probe.gd` on seeds 483920, 912345, 246371 and 83744 at 384 x 288 (800 km, villages on): every settlement laid out with `civ.bank_sites` off, then on. One summary line per state; `--rows` lists each town off a bar or whose housing fell by a fifth. |
| `census_2048.txt` | The same on seed 483920 at 2 048 x 1 536. |
| `census_owner_532187.txt` | The same on the owner's world (seed 532187, 2 048 x 1 311, the parameters of `C:/cw/owner_repro/seed532187_params.txt`, which is read in place and not copied here); his town is settlement #1 at cell 1478, 1159, printed in both states. |
| `drive_desktop.log`, `drive_tablet.log`, `drive_phone.log` | `tools/town_bank_drive.gd` in the real shell on each form (seed 483920 as the app generates it, which is not the headless census world: the app's generate sets its own sea level). Seven numbered checks, every line `ok` or `FAIL`. |
| `drive_owner_desktop.log`, `drive_owner_phone.log` | The same drive on the owner's world, framing his town. |
| `drive_mutate_off.log` | The drive's own mutant (`HM_MUTATE=off`: the "after" state left off). Must fail; it fails five checks. |
| `drive_mutate_shell.log` | A shell mutant in a scratch copy: `engine_bridge.gd` no longer emits `urban_params_changed` for `civ.bank_sites`. Must fail check 5; it does. |
| `combined.png` | The frames, desktop: six river towns of the app's seed 483920 and the owner's town, one row per town and state (BEFORE = switch off, AFTER = on); columns: the map at each zoom framed, then the City Viewer at its opening frame and at x4. |
| `combined_tablet_phone.png` | The owner's town on the phone, then six towns on the tablet (1700 x 1000) and on the phone (450 x 975). **Not the same six as the desktop's**: the same indices were asked for, but the touch forms' world has 236 settlements where the desktop's has 239, so an index names another town there (#3 is Abedomarmarch on the tablet and Valcorforum on the desktop). Every one is a river town the ruling moved; only the desktop sheet has the town whose market's bank flipped (#209), and only one such town, not two: this world has one away from the map's edge. |
| `mutants.py`, `mutants_result.txt` | The Rust literal mutants and their verdicts. |
| `run_drive.py` | The launcher used for every windowed run: it holds `C:/cw/.win_lock` with an owner file for the run only. |
| `combine.py` | Builds the combined PNGs from the crops a drive saved. |
| `workspace_test.txt` | The summary of `cargo test --workspace --no-fail-fast` on the final tree. |
| `drive_final_quick.log` | One desktop drive on the dll rebuilt after the last rebase. |
| `drive_reopen.log` | The drive with `HM_REOPEN=1`: the world saved with the switch on and reopened; the framed town is laid out as before the save (pin, market, box, every building) and the engine still gives it its point. |
| `bankframes_quick.log` | Ruling HG's pixel probe (`_bankframes_probe.tscn --quick`) run again on the marker's new rule: no marker point in the river's water. |

How to run them again is in each script's header. The individual crops are not kept: the combined images are the record.
