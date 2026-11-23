# Ruling HQ: Ruling HN's answers (2026-11-23)

The state, the causes and the bar table are in `cartalith-native/docs/STATUS.md` ("Ruling HQ"). This folder holds
the evidence that entry was read from.

| File | What it is |
|---|---|
| `census_before_*.txt` | This ruling's `_towncensus_probe.gd` on Ruling HN's build (the dll of `origin/main` at `676fbad2`; every new key read with a default, so its span over the water reads as the whole chain). The runs FAIL where HN missed: two wall runs over a river on 246371, one over the sea at 2 048, the owner's town under 90 %. `384` covers seeds 483920, 912345, 246371 and 83744 at 384 x 288 (800 km, villages on). `2048` is seed 483920 at 2 048 x 1 536. `owner` is seed 532187 at 2 048 x 1 311 from `C:/cw/owner_repro/seed532187_params.txt`, with his town (cell 1478, 1159) printed in both states. |
| `census_after_*.txt` | The same three runs on the final tree, with this ruling's columns: ferries, a bridge's span over the water, walled towns with no street within 150 m of their market (the gate for HN's U18), and towns housing over 125 %. The runs gate `--max-quartered` (6, 2, 0) and the owner's also `--cell-min-housed 90`. |
| `bridge_fate.py`, `bridge_fate.txt` | Each of HN's bridge towns in HQ: still a bridge, a ferry, or lost, with its housing in both. |
| `mutants.py`, `mutants_result.txt` | The literal mutants on the unit suite: 21 mutants, 19 killed by the unit suite. N2 and N5, the tries, are killed by the census instead. |
| `census_mutants.py`, `census_mutants_result.txt` | The mutants on wiring lines that no unit fixture reaches, run against the census on the world named for each. These are HN's four survivors (U14, U16, U18 and U19) plus this ruling's wiring. The script rebuilds the dll per mutant and checks its mtime. W3 survived and its code was removed. W4 is a reading, not a grade (see the result file). |
| `marker_cost.txt` | `tools/marker_cost_drive.gd` in the real shell, desktop: where the markers' table is built after a generate, and the cost of the first marker query after a live sea-level move. Each is run twice: after the change, and with `HQ_MUTATE=nowarm` (the render warm-up off, which is the state before this change and the drive's mutant). |
| `combined.png`, `combine.py` | The frames. Rows: the owner's town, 246371 #8 at 384 (the wall spike), 483920 #4 at 2 048 (the wall over the sea) and 483920 #9 at 384 (a ferry town). Each town has two rows, both with `civ.bank_sites` on: BEFORE (HN's build) and AFTER. Columns: the map at x16 and x64, then the City Viewer at its opening frame and at x4. **246371 #8's map crops show nothing useful**: the town sits on the map's west edge, and the crop shows the frame's margin and the side panel. Its City Viewer crops do show the change, the wall spike into the river before and the wall on the shore after. The frames predate the last two changes (the adapter's wall routing removed, and the dead-end landing); the four framed towns' census rows are identical before and after them. The 384 towns were made by the engine call (`HM_PARAMS` with an empty file), so their indices are the census's. `EngineBridge.generate` makes another world from the same seed (`MISTAKES.md`, the preflight row on an owner's parameter list). |
| `run_drive.py` | The launcher of every windowed run. It holds `C:/cw/.win_lock` with an owner file for the run only, under a scratch APPDATA and `CARTALITH_CONFIG_PATH`. |
| `frames_*.log` | `tools/town_bank_drive.gd`'s checks for each framed run. "A press at #8's drawn point finds it" fails on 246371 #8 in **both** builds; this ruling did not cause it. |
| `workspace_test.txt` | The summary of `cargo test --workspace --no-fail-fast` on the final tree after the last rebase. |

Each script's header says how to run it again. A project copy made from a fresh worktree needs the owner's
`*.import` files copied in (`MISTAKES.md`).
