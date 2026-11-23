# Ruling HN: a town's water is the map's water (2026-11-23)

The state, the causes and the bar table are in `cartalith-native/docs/STATUS.md` ("A town's water is the map's
water"). This folder holds what that entry was read from.

| File | What it is |
|---|---|
| `census_before_*.txt` | `_towncensus_probe.gd` (this ruling's version: lines sampled every 4 m, bridges and gated walls counted apart, the map's lake and sea, four zooms) on the build Ruling HM shipped plus the new measures only: `384` is seeds 483920, 912345, 246371 and 83744 at 384 x 288 (800 km, villages on), `2048` seed 483920 at 2 048 x 1 536, `owner` seed 532187 at 2 048 x 1 311 from `C:/cw/owner_repro/seed532187_params.txt` (read in place), his town printed in both states. Each run lays every settlement out with `civ.bank_sites` off, then on. |
| `census_after_*.txt` | The same three runs on the final tree before the last rebase (onto `631f530d`, which touched no town or river code); after it the owner's world was run again on the rebuilt dll and every `TOWN` line is the same. `--rows` lists each town off a bar or whose housing fell by a fifth; "least housed" names the eight lowest river towns. |
| `timing.txt`, `slider.txt` | `--time` (the marker path, run alone) and `--slider` (the river look's width from 1 to 2.5: no town moves, the markers do). |
| `mutants.py`, `mutants_result.txt` | The Rust literal mutants and their verdicts: 35 killed, and four survivors on wiring lines, named. |
| `run_drive.py` | The launcher of every windowed run: it holds `C:/cw/.win_lock` with an owner file for the run only, under a scratch APPDATA. |
| `write_older_save.gd` | How the older world of the offer drive was made: run headless in a project copy that loads the **last binary before the switch existed** (`4e273cb2~1`, built in a separate worktree), it generates seed 483920 at 384 x 288 and saves it. The save (2.4 MB) is not kept here; the two older archives already in `crates/cartalith-godot/tests/fixtures/` back the unit test, and neither can show the card (no complete substrate, so no town can be laid out). |
| `drive_offer_desktop.log`, `_tablet.log`, `_phone.log` | `tools/bank_offer_drive.gd` in the real shell on each form: the card, Keep, never again (the same file and a renamed copy), Update with a frozen town, and a new save. |
| `drive_offer_mutant.log` | A shell mutant in the scratch copy (the recorded answer is not read): "and is not asked again" fails. The renamed copy's check does not fail with it, although the card is shown again there: the drive finds the card by its node name, and the new card is renamed while the old one is still queued to be freed. A weakness of that one check, not of the shell. |
| `offer_desktop.png`, `offer_tablet.png`, `offer_phone.png` | The card on each form, as the drive saw it. |
| `drive_frames_*.log` | `tools/town_bank_drive.gd` on the towns framed: `before_*` with Ruling HM's build (`origin/main` at `6122c43b`, in a second project copy), `after_*` with this one. The before runs fail their one-bank check on the two flipped towns, which is the miss this ruling closes. |
| `combined.png` | The frames: the owner's town, two towns whose marker stood across the river from their market (532187 #5, 246371 #103) and two sea towns (246371 #2 and #6). Two rows a town, both with the switch on: BEFORE (Ruling HM's build) and AFTER; columns: the map at x16 and x64, the City Viewer at its opening frame and at x4. **The two flipped towns show no flip in these crops**: their marker was across the river from the market at x1 and x4, where the distance is under a pixel; at x16 and x64 both builds put it on the market's bank. What the crops do show is the town itself before and after. The lake columns of the census are 0 in both states, so the lake edge is covered by the unit test against the tiles' water mask, not by a frame. |
| `combine.py`, `plot_town.py` | Build `combined.png`; plot a `--dump` of one town over the map's water (how every cause in the status entry was found). |
| `plots/` | The plots the causes were read from: the owner's town before (houses, no streets on the market's bank) and after, and the towns named in the status entry. |
| `workspace_test.txt` | The summary of `cargo test --workspace --no-fail-fast` on the final tree. |

How to run them again is in each script's header. A project copy made from a fresh worktree needs the owner's
`*.import` files copied in (`MISTAKES.md`).
