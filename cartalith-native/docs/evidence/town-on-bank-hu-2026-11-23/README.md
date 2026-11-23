# Ruling HU: Ruling HQ's answers (2026-11-23)

The state, the causes and the bar table are in `cartalith-native/docs/STATUS.md` ("Ruling HU"). This folder holds
the evidence that entry was read from.

| File | What it is |
|---|---|
| `census_before_*.txt` | `_towncensus_probe.gd` on Ruling HQ's build (the dll of `origin/main` at `4317673c`). `384` is seeds 483920, 912345, 246371 and 83744 at 384 x 288 (800 km, villages on), `--max-quartered 6`; `2048` is seed 483920 at 2 048 x 1 536, `--max-quartered 2`; `owner` is seed 532187 at 2 048 x 1 311 from `C:/cw/owner_repro/seed532187_params.txt`, with his town (cell 1478, 1159) printed in both states, `--cell-min-housed 90 --max-quartered 0`. These ran on Ruling HQ's probe, without this ruling's two gates; their longest bridge reads 885 m at 2 048 and 404 m on the owner's world, which the first gate would fail. |
| `census_after_*.txt` | The same three runs on the final tree, with this ruling's two new gates: a bridge longer than 400 m with its approaches, and a ferry shorter than half its town's river. |
| `bridge_fate.py`, `fate_*.txt` | Each of Ruling HN's bridge towns (HN's census is Ruling HQ's `census_before_*.txt`) in HQ's build and in this one: a bridge, a ferry or nothing, with its housing; then every town whose crossing changed kind between HQ and HU. Bars: no town loses its crossing; a ferry town houses at least its HN figure less 5 points. |
| `bars.py`, `bars.txt` | The bar table per world, before and after. |
| `mutants.py`, `mutants_result.txt` | Literal mutants on the unit suite. |
| `census_mutants.py`, `census_mutants_result.txt` | Mutants on lines the unit suite does not reach, against the census on the world named for each. |
| `drive_mutants.py`, `drive_mutants_result.txt` | Mutants on the markers' job wiring, against `tools/marker_async_drive.gd` in the real shell. |
| `marker_timing_before.log`, `marker_timing_after.log` | `tools/marker_async_drive.gd` (desktop, real shell, seed 483920 at 384, 236 settlements): per case (a live sea-level move, a sculpt stroke committed, opening a save, a parameter write) the tables built on the main thread and on the job, the shell's marker call right after the change, and the longest frame in the next four seconds. BEFORE is Ruling HQ's dll with this ruling's shell (the engine has no job, so the shell takes the old path); its FAIL lines are the drive's mutant. Another lane's Godot was running during both. |
| `combined.png`, `combine.py` | The frames. Rows: the owner's town, 83744 #85 and 246371 #205 (the two hamlets), 83744 #3 and 912345 #13 (two of the crossings HQ lost). Each town has two rows, both with `civ.bank_sites` on: BEFORE (HQ's build) and AFTER. Columns: the map at x16 and x64, the City Viewer at its opening frame and at x4. |
| `run_drive.py` | The launcher of every windowed run. It holds `C:/cw/.win_lock` only while the window is open, with an owner file written at once, under a scratch APPDATA and `CARTALITH_CONFIG_PATH`. |
| `frames_*.log` | `tools/town_bank_drive.gd`'s checks for each framed run. "A press at #3's / #85's drawn point finds it" fails on 83744 in **both** builds; this ruling did not cause it. |
| `workspace_test.txt` | The summary of `cargo test --workspace --no-fail-fast` on the final tree after the last rebase. |

Each script's header says how to run it again. A project copy made from a fresh worktree needs the owner's
`*.import` files and `.godot` folder copied in (`MISTAKES.md`).
