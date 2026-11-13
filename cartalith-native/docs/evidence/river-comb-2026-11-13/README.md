# River combs, 2026-11-13 (owner Ruling EV)

The owner, with two phone screenshots of seed 51203 (OnePlus 6T, APK from `da0dbbbf`, the river flow view and the
relief view at zoom 788 %): *"Still see some heavy combing:"*, then *"resolve it"*. `DELTA_GEOMETRY_RESEARCH.md`
section 14 has the cause, the fix and the measurement; this folder holds the frames and the logs. A verifier's
findings on the first pass were worked the same day (14.8, 14.9, and `STATUS.md`'s follow-up entry): the frames, the
composites and the `probe_*` / `*_comb.json` / `rules_*` files here are from that second pass, with the lake-shore fix.

## The three arms

Every frame is the owner's world rebuilt in the app from the World info dump the phone printed
(`crates/cartalith-godot/tests/fixtures/owner_world_51203_params.txt`, 123 rows; seed 51203, 2048 x 1024, 40 075 km,
river density 2.5, world structure on, GPU on), centred on cell (165, 256), the middle of the owner's screenshots:

- **owner**: as the phone made it: rough routing, the lake water balance and the shared road trunks OFF (an older
  save had been opened in that session, and New World generated from the table it left);
- **before**: what a correct New World made before Ruling EV: rough routing, the lake balance and the trunks on,
  basin routing off;
- **after**: Ruling EV: the same, with basin routing on (what New World makes now).

## Files

- `comb_seed51203_z01.00.png`, `_z04.00`, `_z07.88` (the owner's zoom), `_z16.00`, `_z64.00`: per zoom, the river flow
  view (top row) and the relief view (bottom row), columns owner / before / after, each panel the app's map view at
  0.75 scale. Lakes are drawn with the second pass's shore fix (a lake the water balance made is contoured by depth;
  before it, dry-country balanced lakes drew cell-stepped from x7.88).
- `owner_composite_combing_fixed.png`: the owner's view at 788 %, as made (left) and as a New World makes it now
  (right), full scale.
- `<arm>_comb.json`: `WorldGen.river_comb` for each arm (the comb and straight-run measure on the traced network, on
  the part of it under a drawn lake (`traced_lake`) and on what Ruling EN's law draws at each zoom, over the whole map
  and over the screenshots' window of cells 35..295 x 50..463), with the settlement and road counts.
- `probe_<arm>.log`: the `_rivercomb_probe.tscn` runs (windowed), every parameter read back after the run, and the
  shore-field check on the two balanced arms.
- `measurement.log`: `tests/river_basin_routing_measurement.rs` (release, alone, `use_gpu` off) on seeds 51203, 246371,
  483920 and 912345, the first pass.
- `measurement_gpu_51203.log`: the same harness with `BR_GPU=1` on 51203 (the owner's world as the app makes it), with
  lake cells under Ruling CC's area floor and inside the owner's window; and the two fill tilts compared on its final
  field.
- `measurement_tilts.log`: `floor_tilt_against_breadth_on_the_final_field` on the four worlds (`use_gpu` off): cells
  each tilt raises that the other does not, and land cells that drain to another seed.
- `mutants.log` (first pass) and `mutants_followup.log` (second pass): literal mutants, Rust and GDScript.
- `rules_desk.log`, `rules_phone.log`, `rules_tablet.log`: `_newworld_rules_probe.tscn` (windowed) on the desktop
  shell, the phone form and the tablet form; `rules_base.log`: the same probe on `fcda4d3d`'s shell (the first pass),
  where its new legs fail.
- `screening_four_tilts_owner_window.png`: the screening that chose the tilt, from a scratch harness that was not
  kept (the carve-off ground of seed 51203, rough routing on, the owner's window; tan is depression-filled ground):
  the breadth-first fill, a chamfer tilt alone, a terrain-ordered fill alone, and both (built). Looked at, not
  measured.

## How to reproduce

```text
godot --path . res://_rivercomb_probe.tscn -- --out DIR --params ../crates/cartalith-godot/tests/fixtures/owner_world_51203_params.txt \
    --zooms 1,4,7.88,16,64 --tag after --set rough_routing=true,lake_balance=true,civ.merge_ways=true,basin_routing=true
cargo test --release -p cartalith-godot --test river_basin_routing_measurement -- --ignored --nocapture --test-threads=1
BR_GPU=1 BR_WORLDS=51203 cargo test --release -p cartalith-godot --test river_basin_routing_measurement -- --ignored --nocapture --test-threads=1
```

The `owner` arm is `--set basin_routing=false` (the dump predates the key); `before` is the `after` line with
`basin_routing=false`. Each probe run took the `C:/cw/.win_lock` window lock for that run only.
