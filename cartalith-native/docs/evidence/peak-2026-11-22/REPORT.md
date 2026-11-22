# F29 peak altitude from tectonics (Ruling GL), 2026-11-22

Harness: `crates/cartalith-engine/tests/peak_f29.rs`, `measure_f29` (ignored; run `cargo test -p cartalith-engine --test peak_f29 --release -- --include-ignored --nocapture`). 384x256 grid, 14 plates, drift 1.0, 3 000 km unless stated.

Law: `peak = 2000 + 7000 (1 - e^(-E/3)) (1 - e^(-W/1000))` m. E = 90th percentile of the closing relative plate speed over plate-boundary cell edges, 2 cm/yr per velocity unit. Anchors (heights and rates recalled, to be confirmed): Himalaya 8 849 m at ~4-5 cm/yr (Molnar & Tapponnier 1975), Andes 6 961 m at 7-8 cm/yr (Somoza 1998), Alps 4 808 m at <1 cm/yr and old, Appalachians 2 037 m, Urals 1 895 m; summit ceiling ~9-10 km (Dalmayrac & Molnar 1981; England & Houseman 1989), glacial buzzsaw (Egholm et al. 2009). The constants are labelled judgements; 2 cm/yr per unit was calibrated by measurement (5 cm/yr gave 7-8 km everywhere and 50 % of land below 0 C).

## Derived peak (m), per seed and setting

| setting | 483920 | 912345 | 246371 |
|---|---|---|---|
| 8 plates, drift 1, 3 000 km | 5727 | 5674 | 6472 |
| 14 plates, drift 0.5 | 4160 | 3689 | 4640 |
| 14 plates, drift 1, 800 km | 4097 | 3709 | 4453 |
| 14 plates, drift 1 | 5618 | 4949 | 6233 |
| 14 plates, drift 2 | 7268 | 6591 | 7772 |
| 30 plates, drift 1 | 5974 | 5635 | 5992 |
| 14 plates, drift 1, 12 000 km | 5808 | 5104 | 6454 |

Monotone in drift (asserted for 6 drifts in `derived_peak_is_deterministic_and_grid_independent_and_responds_to_velocity`) and in map width; zero drift gives 2 000 m. Resolution does not move it (256-wide proxy).

## Before (4 000 m) and after, shares of land, 3 000 km

| seed | peak | T<0 C | T<5 C | T<-10 C | above 0.65 snowline |
|---|---|---|---|---|---|
| 483920 | 4000 / 5618 | 14.3 / 29.0 % | 25.4 / 48.4 % | 2.9 / 9.8 % | 22.8 / 22.8 % |
| 912345 | 4000 / 4949 | 12.0 / 23.6 % | 28.9 / 41.3 % | 0.9 / 2.2 % | 16.3 / 16.2 % |
| 246371 | 4000 / 6233 | 16.6 / 27.7 % | 24.4 / 42.4 % | 0.5 / 13.1 % | 8.1 / 8.0 % |

The snowline is a fraction of the above-sea range (`glacial_snowline` 0.65), so it does not move; the temperature lapse reads metres, so the cold area roughly doubles. Coasts, land share (79.8, 62.2, 71.3 %) and the height field are byte-identical; the 2D frames differ only in snow and vegetation tint and in what civ places on colder ground.

Frames (all looked at): `{483920,246371}_{off,on}_00_2d_before.png` (map) and `_01_entry_orbit.png` (3D, true scale), 1 200 km, 512x384; peak 4 000 vs 4 676 m (483920) and 4 000 vs 4 998 m (246371).

Mutants (Python, exact literal, restored in `finally`, run serially; engine `peak_f29` release plus godot `params_mapping peak_auto`): 11 of 11 killed: each of the five constants, the percentile index (9/10 to 5/10), the closing-speed sign, `resolve_peak`'s condition inverted, `peak_auto` true in `WorldParams::defaults`, false in `params::defaults`, true in `LOADS_WHEN_ABSENT`.
