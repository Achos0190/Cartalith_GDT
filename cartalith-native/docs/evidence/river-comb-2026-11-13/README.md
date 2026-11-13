# River combs, 2026-11-13 (owner Ruling EV)

The owner, with two phone screenshots of seed 51203 (OnePlus 6T, APK from `da0dbbbf`, the river flow view and the
relief view at zoom 788 %): *"Still see some heavy combing:"*, then *"resolve it"*. `DELTA_GEOMETRY_RESEARCH.md`
section 14 has the cause, the fix and the measurement; this folder holds the frames and the logs.

## The three arms

Every frame is the owner's world rebuilt in the app from the World info dump the phone printed
(`crates/cartalith-godot/tests/fixtures/owner_world_51203_params.txt`, 123 rows; seed 51203, 2048 x 1024, 40 075 km,
river density 2.5, world structure on), centred on cell (165, 256), the middle of the owner's screenshots:

- **owner**: as the phone made it: rough routing, the lake water balance and the shared road trunks OFF (an older
  save had been opened in that session, and New World generated from the table it left);
- **before**: what a correct New World made before Ruling EV: rough routing, the lake balance and the trunks on,
  basin routing off;
- **after**: Ruling EV: the same, with basin routing on (what New World makes now).

## Files

- `comb_seed51203_z01.00.png`, `_z04.00`, `_z07.88` (the owner's zoom), `_z16.00`, `_z64.00`: per zoom, the river flow
  view (top row) and the relief view (bottom row), columns owner / before / after, each panel the app's map view at
  0.75 scale.
- `owner_composite_combing_fixed.png`: the owner's view at 788 %, as made (left) and as a New World makes it now
  (right), full scale.
- `<arm>_comb.json`: `WorldGen.river_comb` for each arm (the comb and straight-run measure on the traced network and
  on what Ruling EN's law draws at each zoom, over the whole map and over the screenshots' window of cells
  35..295 x 50..463), with the settlement and road counts.
- `probe_<arm>.log`: the `_rivercomb_probe.tscn` runs (windowed), every parameter read back after the run.
- `measurement.log`: `tests/river_basin_routing_measurement.rs` (release, alone) on seeds 51203, 246371, 483920 and
  912345.
- `rules_desk.log`, `rules_phone.log`: `_newworld_rules_probe.tscn` (windowed), the New World restore on the desktop
  shell and the phone form.
- `screening_four_tilts_owner_window.png`: the screening that chose the tilt, from a scratch harness that was not
  kept (the carve-off ground of seed 51203, rough routing on, the owner's window; tan is depression-filled ground):
  the breadth-first fill, a chamfer tilt alone, a terrain-ordered fill alone, and both (built). Looked at, not
  measured.

## How to reproduce

```text
godot --path . res://_rivercomb_probe.tscn -- --out DIR --params ../crates/cartalith-godot/tests/fixtures/owner_world_51203_params.txt \
    --zooms 1,4,7.88,16,64 --tag after --set rough_routing=true,lake_balance=true,civ.merge_ways=true,basin_routing=true
cargo test --release -p cartalith-godot --test river_basin_routing_measurement -- --ignored --nocapture --test-threads=1
```

The `owner` arm is `--set basin_routing=false` (the dump predates the key); `before` is the `after` line with
`basin_routing=false`. Each probe run took the `C:/cw/.win_lock` window lock for that run only.
