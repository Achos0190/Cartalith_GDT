# Sea lanes end on their port's marker (owner report 2026-11-23)

The owner, after a clean generation: *"the attachment from routes (sea and land) to the settlements when you zoom
in."* This folder is the sea half. The state and the numbers are in `cartalith-native/docs/STATUS.md` ("Sea lanes end
on their port's marker"); this folder holds the evidence that entry was read from.

| File | What it is |
|---|---|
| `run_drive.py` | The launcher of every windowed run: `tools/sea_lane_ends_drive.gd` in the real shell on one form, holding `C:/cw/.win_lock` only while the window is open, with an owner file written at once, under a scratch APPDATA and `CARTALITH_CONFIG_PATH`. |
| `drive_after_{desktop,tablet,phone}.log` | The drive on the final shell: seed 51203 at 2 048 x 1 311, 800 km, made by `EngineBridge.generate` with no world open (the app's parameter table). Per zoom (x1, x4, x16, x64), the gap in window pixels from the end the overlay drew each lane with to its port's marker, and the same for the engine's own end (the stroke before this change). |
| `drive_after_seed483920_{desktop,phone}.log`, `drive_after_seed912345_{desktop,phone}.log` | The same drive on two more seeds (2 048 x 1 311, app defaults): gap and spur 0.00 px at every zoom, no crossing at any zoom. 912345 has one lane (two ends), so its precondition line ("at least 4 ends") reads FAIL there; its measurements stand. |
| `drive_before_nopin_desktop.log`, `drive_after_nopin_desktop.log` | The two pin-hidden runs the frames' first two blocks come from; the BEFORE run is the drive going red with the attachment removed. |
| `mutants.py`, `mutants_result.txt` | Six literal mutants of `map_overlay.gd`'s attachment and cut, each run through the drive on the desktop form; all KILLED. |
| `frames/`, `combined.png`, `combine.py` | Desktop, two lane ends at x1, x4, x16 and x64, each crop centred on the marker: BEFORE with the pins hidden (the attachment replaced by `pass` in the project copy for that run, restored after), AFTER with the pins hidden (`SL_HIDE_PINS=1`), and AFTER as the app draws it. With the pins drawn the change is mostly under the pin's head; hidden, the BEFORE stroke stops at the cell corner or runs past the town and back, and the AFTER stroke ends on the town's market badge. |
| `census_sea_fork_bar_before.txt`, `census_sea_fork_bar_after.txt` | `sea_site_census::sea_lane_census` on its ten worlds before and after the fork bar was tightened (`FORK_PART_KM`); only `thread_km` moves. The `owner51203` rows are the owner's world from his parameter list (`SEA_CENSUS_PARAMS`, not committed). |
| `owner51203_strait_lanes_on_off.png`, `strait.py`, `find_thread.py` | The owner's world from the census dump (data frame: the generator's ocean and land, one colour per lane, ports ringed): `civ.sea_lanes` on (left) and off (right) at the port (204, 1192), where the tightened bar's 26.95 km comes from; `find_thread.py` lists the side-by-side stretches per lane pair, `strait.py` draws a box of dumps side by side. |
| `census_mutants.py`, `census_mutants_result.txt` | Literal mutants of the tightened fork bar, judged by the census on the owner's world. |
| `sweep_sea_census_v26.11.22-002_6f2d5080.txt`, `sweep_sea_census_v26.11.23-001_0b3ba6cd.txt` | The regression sweep: the same census harness built at the two APKs' commits. Every row, and every dumped lane polyline, is identical. |
