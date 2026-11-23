# CS-0: today's placing against the coastal setback research (2026-11-23, owner Ruling HQ)

The tables and the verdict are in `COASTAL_SETBACK_RESEARCH.md` section 7. This folder holds what they were read from.
Measured on `676fbad2`, before Ruling HQ's build. Measurement only: no product code changed.

| File | What it is |
|---|---|
| `cs0_probe.gd` | The layout side. Headless, run from a project copy (`--script res://cs0_probe.gd`); one JSON line per settlement: the census row (`WorldGen.town_bank_census`) and the town as `WorldGen.town_debug_json` gives it on an 8 m lattice (the map's drawn land, sea, lake and river stroke). `--params` an owner's list, `--app` the world `EngineBridge.generate` makes. The dumps (6-30 MB each) are not kept; the commands below make them again. |
| `census/probe_*.log` | The probe's own lines per world (settlements, timings). |
| `census/census_*.txt` | `cs0_hazard_census`'s output per world (`cartalith-godot/src/town_river_census.rs`, `#[ignore]`d): the identity checks (cell heights and the drawn sea against the probe's), the classes, the `H_SURGE` / `H_FLOOD` sweeps, distances and relief. |
| `rows/rows_<seed>_<gw>.tsv` | One row per coastal or river town: class, distances (market, quay, quay to any water, nearest street or building), the reasons at the waterfront point and at the market, exposure, relief, whether the heights and the cell fields split in the cap, the rule's core distance, the market cell's and the settlement cell's flood, landform and wetland, and Ruling HM's origin offset. `app_rows_483920_384.tsv` is the app world. |
| `summary.txt`, `cs0_summary.py` | The per-world tables, the bar baselines (from the probe rows), the make-up of (a) and (d), where the river (c) reason comes from, the go/no-go inputs and the FK row's towns. `python -I cs0_summary.py <dumps> <rows dir>`. |
| `hfa/harbour_front_audit_*.txt` | The old census (`harbour_front_audit`, the box's raster water, no HM/HN), today, at 384 x 288 (three seeds) and 2 048 x 1 311 (483920), villages on: the FK leftovers in their original measure, and the sea lanes. |
| `timing.txt` | `CS0_TIME=1`: the flood, wetland and landform builders, median of 5, two separate processes, run alone. |
| `run_frames.py`, `combine.py`, `combined.png`, `frames/` | The frames: `tools/town_bank_drive.gd` in the real shell (desktop 1600 x 900, scratch APPDATA, `C:/cw/.win_lock` held with an owner file for the run), towns by cell, the map at x4 and x16 and the City Viewer, `civ.bank_sites` on. The 2 048 run used `HM_PARAMS` pointing at a file with no parameter in it, so the drive made its world by `generate_sized` like the probe (the drive found the same indices: 3, 34, 30, 7, 166). `combined.png`: two towns of each of (a), (c), (d). |

## Running it again

```
# 1. dll (release, copied to <project copy>/../target/debug/cartalith_godot.dll) and a project copy with the owner's *.import and .godot
CARGO_TARGET_DIR=<t> cargo build --release -p cartalith-godot
# 2. the probe, per world (scratch APPDATA, LOCALAPPDATA and CARTALITH_CONFIG_PATH)
godot --headless --path <copy> --script res://cs0_probe.gd -- --seed 483920 --gw 384 --gh 288 --out w483920_384.jsonl
godot ... -- --seed 483920 --gw 2048 --gh 1311 --out w483920_2048.jsonl
godot ... -- --seed 532187 --gw 2048 --gh 1311 --params C:/cw/owner_repro/seed532187_params.txt --out w532187_2048.jsonl
godot ... -- --seed 483920 --gw 384 --gh 288 --app --out app483920_384.jsonl
# 3. the terrain side, per dump (CS0_PARAMS only for the owner's world; CS0_TIME=1 for the timings)
CS0_DUMP=w483920_384.jsonl CS0_OUT=<rows dir> <test exe> cs0_hazard_census --ignored --nocapture --test-threads 1
# 4. the old census
TOWN_RIVER_GW=384 TOWN_RIVER_GH=288 TOWN_RIVER_VILLAGES=1 TOWN_RIVER_LIST=1 <test exe> harbour_front_audit --ignored --nocapture --test-threads 1
# 5. frames (takes the window lock itself)
python -I run_frames.py <out> 1500 HM_SEED=483920 HM_GW=2048 HM_GH=1311 HM_PARAMS=<a file with no key> HM_CELLS=1520:293,1404:968,1063:198,1323:81,1456:298 HM_MAP_ZOOMS=4,16
```
