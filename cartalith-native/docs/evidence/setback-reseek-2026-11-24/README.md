# Ruling HZ: the setback guard seeks its town's crossings first (2026-11-24)

The change, the cause and the bars are in `cartalith-native/docs/STATUS.md` ("Ruling HZ") and `DECISIONS.md` section 7p.
BEFORE is `origin/main` at `3772af5c`; AFTER is this build. Both with `civ.bank_sites` on and both setback switches on.

| File | What it is |
|---|---|
| `before_30.png`, `after_30.png`, `combined.png` | 483920 #30 (the capital at cell 1063,198) at 2 048 x 1 311, plotted by `plot_town.py` from `_towncensus_probe.gd --dump` (`town_debug_json`): market red, bridges yellow, gates white, wall orange. BEFORE the guard refused the set-back layout (one crossing fewer) and the town stands where the unshifted layout puts it, market 318 m from the sea; AFTER the market is at the water, the bridge stays (a gate was added where its road meets the wall), housed 16 614 against 15 075 of 17 171. |
| `plot_town.py` | Copied from `town-on-bank-hn-2026-11-23/`. Run `python -X utf8 -I`: the dumps hold UTF-8. |
| `run_frames.py` | The HS drive runner, kept for reference. **Its City Viewer frames are not evidence here**: in the shell's world #30 is served as the frozen layout (`#30 froze with the switch off`, `served unchanged`), so BEFORE and AFTER were byte-identical files; the plots above are the dump of the live layout. |
| `census_after_*.txt`, `status.txt` | `_towncensus_probe.gd` on the AFTER build: 483920 at 2 048 x 1 536, the owner's 532187, the four 384 worlds. Every run exited 0. Against `coastal-setback-2026-11-23/merged/census_*.txt` the only differences are timings (and the GPU loader warnings). |
| `road_after_*.txt`, `status_roads.txt` | `_roadattach_probe.gd` on the same worlds. |
| `cs0_after.txt` | `town_river_census::cs0_hazard_census` over `cs0_probe.gd`'s dump of 483920 at 2 048 x 1 311: coastal markets set back without a reason 3 (`#7 232 m`, `#166 838 m`, `#236 1030 m`; was 4 with #30 at 318 m), guard refusals 0 (was 1). The river's 9 are as before. |
| `mutants_result.txt` | The named literal mutants: unit tests (`setback/tests.rs`) and end to end (#30's dump). |
| `workspace_test.txt`, `baselines.txt` | `cargo test --workspace --no-fail-fast` and the 12 ignored `fh_r2_baseline`/`g1_baseline` tests on the final tree. |

## The cause, at its symbols

The set-back layout of #30 has no crossing: not because a road join was lost (`routes::dry_join` joins the host's roads to the new
market and the host's crossing road is in the graph before the sweep) but because `cleanup::clear_fort_zone_keeping`
kills every street that passes the wall away from a gate. The crossing road stood 33 m from the wall's nearest gate and
the sweep's keep radius for a primary is 31 m. `setback::gate_bridges` adds the gate; `setback::reseek_crossing` lays a
street across for a town with none in its own network (it acts on other towns' layouts; on #30 the crossing existed).
