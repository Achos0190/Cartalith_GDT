# F26 under Ruling GL: city-states and spheres of influence (2026-11-22)

- `measure.txt`: `cargo test -p cartalith-godot --lib city_states_measure -- --ignored --nocapture`
  (`city_state_bridge/tests.rs`). Seeds 483920, 912345, 246371 at grid widths 384 and 2048, the
  app's `params::defaults` (800 km), three worlds each: every faction deleted (`factions 0`), the
  generated six (`factions 6`), the sixth deleted (`factions 5`). Per world: loose places and
  their populations, city-states, sphere cells, aligned places, violations (sphere cells on claimed
  land plus faction settlements aligned: must be 0) and a promotion plus undo of the largest sphere.
  Constants picked from it: `CITY_STATE_MIN_POP` 5 000 (towns measured at most 2 609, cities at
  least 7 316) and `SPHERE_REACH_FRAC` 1/10 (1/12 and 1/8 measured at 384 too; see its doc).
- `drive_desktop.txt`: the drive's log, 18 checks, 0 failed, on the tree rebased over F27.
- `frames/`: the real desktop shell at 1600x900 (scratch APPDATA, window captures only), driven by
  `godot-project/tools/city_states_drive.gd` through `tools/fennara_drive.py --shell desktop`, seed
  483920 then 912345 at 512x328:
  - `zero_faction_spheres.png`: all six factions deleted; each city-state's sphere washed lighter in
    its own colour and outlined (the dark lines), no faction wash anywhere.
  - `inspect_city_state.png`: Inspect on the city-state Sevjuniana (8 622 people), `Sphere: City-state · 37 aligned` and
    **Make this sphere a faction**.
  - `inspect_aligned.png`: a village inside it, `Aligned to Sevjuniana` (a link).
  - `card_city_state.png`: the real right-click card on the city-state: subtitle `City-state · 37
    aligned`, row **Make this sphere a faction** (the check reads inside the card popup, not the screen).
  - `promoted.png`: after the real click on Inspect's action: one faction, the city-state its capital,
    38 places in it, the sphere now a nation's wash and pins in the same colour (top left: Sevjuniana sits
    on the map's west edge, so it cannot be centred).
  - `undone.png`: after Edit > Undo: zero factions, the city-state and its sphere back.
  - `factions_spheres.png`: seed 912345 with five factions (the sixth deleted): the loose capital's
    sphere (pink, top right) beside the nations, on unclaimed land only.
