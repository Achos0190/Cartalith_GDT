# Ruling HS: the coastal and floodplain setback, built (2026-11-23)

The rule, the sweep and the bar table are in `COASTAL_SETBACK_RESEARCH.md` section 8 and `cartalith-native/docs/STATUS.md`
("Ruling HS"). BEFORE is `origin/main` at `94064bc5`; AFTER is this build. Both with `civ.bank_sites` on.

| File | What it is |
|---|---|
| `summary.txt` | Everything the tables were read from in one place: towns laid out differently (coarse grids: 0), the hard-gate bars, the crossings, the census exit codes and the `HS` lines. |
| `census/before_*.txt`, `census/after_*.txt` | `_towncensus_probe.gd` (Rulings HM to HU's census and gates): `384` the four seeds at 384 x 288 (`--max-quartered 6`); `2048` seed 483920 at 2 048 x 1 536 (`--max-quartered 2 --rows`); `owner` seed 532187 at 2 048 x 1 311 from `C:/cw/owner_repro/seed532187_params.txt` (`--cell 1478,1159 --cell-min-housed 90 --max-quartered 0`). Every after run exited 0 (its gates). |
| `cs0/before_*.txt`, `cs0/after_*.txt` | CS-0's census (`town_river_census::cs0_hazard_census`, `#[ignore]`d) over `cs0_probe.gd`'s dumps of each world, now with the `HS` lines (records, set back without a reason, bodies past their setback, guard refusals) and the input-1 sweep at 4, 6 and 8 m. |
| `cs0/sweep_h6_*.txt`, `cs0/sweep_h8_*.txt` | The same census over the product run at `H_SURGE` 6 and 8 m (a sweep-only environment override, removed before the commit). |
| `crossings.py` | Per town: every crossing (bridge or ferry) before kept after, and its housing. |
| `cmp_dumps.py` | Per town: whether two builds lay it out identically (market, buildings, streets). |
| `mutants.py`, `mutants_result.txt` | 21 named literal mutants on the constants and comparisons, against the unit tests: all killed. |
| `run_census.sh`, `run_after2.sh`, `cs0.sh` | How the census runs were made (scratch APPDATA; headless, no window lock). |
| `run_frames.py`, `combine.py`, `combined.png`, `frames/` | The frames: `tools/town_bank_drive.gd` in the real shell (desktop 1600 x 900, scratch APPDATA and `CARTALITH_CONFIG_PATH`, `C:/cw/.win_lock` held with an owner file only while the window was open), the map at x4 and x16 and the City Viewer, the `on` crops. `combined.png`: a row a town, BEFORE then AFTER; the last row is one town at 4 and at 8 m. Every run's checks passed but `after_h8.log`'s Ruling HM positive control ("a framed town laid round the drawn edge"), which its two coastal hamlets cannot meet; it says nothing of this rule. |
| `workspace_test.txt`, `baselines.txt` | `cargo test --workspace --no-fail-fast` and the 12 ignored `fh_r2_baseline`/`g1_baseline` tests on the final tree. |

A project copy made from a fresh worktree needs the owner's `*.import` files and `.godot` folder copied in (`MISTAKES.md`).
