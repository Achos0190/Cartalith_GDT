# Ruling GQ: the peak law re-fitted, 2026-11-22

Harness: `crates/cartalith-engine/tests/peak_f29.rs` (`measure_f29`, `sweep_cold_share`, both ignored; `cargo test -p cartalith-engine --test peak_f29 --release -- --include-ignored --nocapture`). 384x256 grid, 14 plates unless stated.

## Fit
Land share below 0 C against a fixed peak (3 000 km, drift 1), `sweep_cold_share`:

| peak m | 483920 | 912345 | 246371 |
|---|---|---|---|
| 3500 | 11.6 | 7.4 | 13.3 |
| 4000 | 14.3 | 12.0 | 16.6 |
| 4500 | 16.9 | 17.7 | 19.1 |
| 5000 | 20.9 | 24.2 | 21.5 |
| 5500 | 27.8 | 30.1 | 24.2 |

At 4.5-5 km a seed necessarily sits 3-10 points above its 4 000 m share: the lapse reads metres, so 4.5 km cannot also keep today's cold share. Constants were chosen by a grid search over (scale, shape, width scale) against both a 4.4-5.0 km default at 800 and 3 000 km, drift 2 at 3 000 km at 7 km or more, and the smallest mean distance from today's per-seed cold share (script not kept; the table above is its input). Law: `peak = 2000 + 7000 (1 - e^(-(E/3.8)^1.25)) (1 - e^(-W/400))` m, `PEAK_CM_PER_UNIT` unchanged at 2.

## Derived peak (m), F29 law in brackets for 3 000 km

| setting | 483920 | 912345 | 246371 |
|---|---|---|---|
| drift 0.5, 3000 km | 3445 (4160) | 3037 (3689) | 3903 (4640) |
| drift 1, 800 km | 4561 | 3920 | 5208 |
| drift 1, 3000 km | 4960 (5618) | 4219 (4949) | 5708 (6233) |
| drift 2, 800 km | 6417 | 5610 | 7048 |
| drift 2, 3000 km | 7105 (7268) | 6172 (6591) | 7835 (7772) |
| drift 3, 800 km | 7363 | 6710 | 7745 |
| 30 plates, drift 1, 3000 km | 5386 | 4980 | 5408 |

Default (drift 1): mean 4.56 km at 800 km, 4.96 km at 3 000 km. Strong collisions reach 7-7.8 km (drift 2, 3 000 km; drift 3, 800 km); the curve saturates toward 9 km, it is not clamped. Monotone in drift and width (asserted).

## Shares of land, 4 000 m (before) / derived (after)

| seed, km | peak | T<0 C | T<5 C | T<-10 C | above snowline |
|---|---|---|---|---|---|
| 483920, 800 | 4000 / 4561 | 15.6 / 18.6 | 26.5 / 35.1 | 3.8 / 6.7 | 24.0 / 23.9 |
| 912345, 800 | 4000 / 3920 | 12.0 / 11.2 | 28.7 / 27.4 | 0.8 / 0.8 | 15.9 / 15.9 |
| 246371, 800 | 4000 / 5208 | 16.6 / 22.6 | 24.5 / 31.4 | 0.5 / 4.7 | 8.8 / 8.6 |
| 483920, 3000 | 4000 / 4960 | 14.3 / 20.5 | 25.4 / 41.1 | 2.9 / 7.3 | 22.8 / 22.8 |
| 912345, 3000 | 4000 / 4219 | 12.0 / 14.4 | 28.9 / 31.8 | 0.9 / 1.1 | 16.3 / 16.3 |
| 246371, 3000 | 4000 / 5708 | 16.6 / 25.3 | 24.4 / 34.8 | 0.5 / 8.3 | 8.1 / 8.1 |

F29 had 29.0 / 23.6 / 27.7 % at 3 000 km. 246371 stays the outlier because its plates close fastest (E 3.0 cm/yr against 1.8 and 2.4). Coasts, land share and the height field are byte-identical.

## Frames (looked at), 800 km, 512x384, `gq/`
`{483920,246371}_{off,on}_{2d,3d}.png`: peak 4 000 / 4 581 m (483920) and 4 000 / 5 039 m (246371; the probe's 512x384 grid gives slightly different plates from the 384x256 harness). Same terrain; more snow on the ranges, taller 3D relief, civ placement shifts a little. One check of ten in the first probe run failed (the 3D probe's own, on the 483920 off arm, not reproduced on the other three runs); the frames were looked at.

## Undo
`undo_group.gd` `write_param` records `peak_auto` as a member before `peak_m` (undo writes in reverse), so one Edit > Undo of an assistant-typed peak restores both. `_ac3_probe.gd` asserts `peak_auto` true, false after Apply, true after Undo.

Mutants (serial, restored in finally, no residue): PEAK_SCALE_CM 3.8 to 3.0, PEAK_SHAPE 1.25 to 1.0, PEAK_SIZE_KM 400 to 1000, PEAK_CM_PER_UNIT 2 to 2.5 and `.powf(PEAK_SHAPE)` to `.powf(1.0)`: 5 of 5 killed by `peak_f29`; removing the `peak_auto` member in `undo_group.gd` write_param: killed by `_ac3_probe` (FAIL, 'the SAME undo restored peak_auto (false)'); restored probe PASS 50 checks.
