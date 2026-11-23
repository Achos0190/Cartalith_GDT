# CPU by default on every device, and the chronos strip's engine reads during a generate (2026-11-23)

Ruling HP (`LARGE_ITEM_RULINGS.md`): the compute path defaults to CPU on every device. Tree: origin/main `cbc49e73` plus this change,
debug dll built from it. Windowed, real shell, a fresh scratch APPDATA per run (`tools/form_determinism.py`), window lock held only while Godot ran.

## The change
- `EngineBridge.default_gpu_on()` returns `false` everywhere. A stored `Preferences > GPU acceleration` choice still wins (`_ready`: `default_gpu_on() if stored < 0 else stored == 1`).
- A save carries `use_gpu` in its parameters: `params::save_state` writes it and `apply_saved_state` restores it. Pinned by the new
  `use_gpu_round_trips_through_a_save_and_defaults_off` (`crates/cartalith-godot/tests/params_mapping.rs`), so an old save made on the GPU reopens on it.
- `tools/form_determinism.py` and its drive: the default is now asserted identical on all three forms, with `use_gpu` False as a literal.
- Prose: `GENERATION_PARAMETERS.md`, `DECISIONS.md` section 7a (a 2026-11-23 paragraph that cites section 7p), comments in `engine_bridge.gd`, `dcc_settings.gd`, `menus.gd`, `new_world_dialog.gd`, `lib.rs`, `_gpudefault_probe.gd`.

## Hashes per form (`all.log`; seed 4242, 256x160; desktop 1600x900, tablet 1700x1000 `--force-touch`, phone 450x975 `--force-touch`)
| setting | use_gpu at boot | height png md5 | rivers | settlements md5 | labels md5 |
|---|---|---|---|---|---|
| default, all three forms identical | false | `c604568f6f78e31c187f0ae5f374c13c` | 596 | `537c44662c9c4906b2bf26e6655c1c4d` | `73c6f909aac7534928afb1a38d1ab764` |
| stored CPU, all three identical | false | the same | 596 | the same | the same |
| stored GPU, all three identical | true | `59766e4a6570577830ca26cb68d1967a` | 597 | the same | `d84eef3c7f7f62fb53afcbca8f50d668` |
`all` ends PASS three times (default, cpu, gpu). The default now equals the CPU world, which before this change only touch devices produced.

## Mutants
- `python tools/form_determinism.py mutant OUT default` (`mutant_default.log`): `default_gpu_on()` true on the desktop. The desktop boots on the GPU
  (`59766e4a...`), the default check goes red (`use_gpu` and the height and label hashes differ), MUTANT KILLED.
- `mutant OUT stored` (the older mutant: the boot ignores a stored choice) is unchanged and was killed on 2026-11-23's earlier evidence.
- Race mutant, below: KILLED.

## The race: the chronos strip read the engine while a generate worker held it
Same error as `form-determinism-2026-11-23/README.md`: `Gd<T>::bind() failed, already bound`, `SCRIPT ERROR: Bug: Invalid call error code 1337`.
- `chronos_items.gd::_start_scan` (the vault scan) and `_refresh` (`timeline_items`) now return while `bridge.generating`, and run again from the bridge's
  own `generation_finished` (connected `CONNECT_ONE_SHOT | CONNECT_DEFERRED`; every `generating = false` is followed by that signal). `dcc_shell.gd::tl_override`
  (read by the strip's `tl_range` every frame) returns `{}` while generating.
- Statistical: 12 phone Creates on the final tree, 0 with a SCRIPT ERROR or "already bound" (`race_fix.log`; an earlier 12 of 12 clean runs before the second guard, `race/fix*`). The old tree showed it in 5 of 11. The same loop with the `_start_scan` guard removed
  was 0 of 12 (`race_mutant.log`: the timer rarely lands in the worker's window on a quiet machine), so that loop cannot tell a fix from none.
- Deterministic: `godot-project/tools/chronos_scan_race_drive.gd` (phone) generates a world, starts a second 1024x640 generate and calls `_start_scan` on the
  live strip while it runs. Fixed tree: exit 0 (`scanrace_fix.log`). Guard removed (literal `if false and ...`): the 1337 error, exit 1 (`scanrace_mutant.log`), MUTANT KILLED.
- The earlier OUTSTANDING row "generate right after opening a saved project prints error code 1337": `tools/tlrange_drive.gd` on desktop reproduced it at `step A`
  (`tlrange_desktop.log`). Its `game.log` showed the same panic from two more readers, `tl_override` (16 times) and `timeline_items` via `chronos_items.gd::_refresh`
  (2), not only `_start_scan`. After the three guards the same drive exits 0 with no 1337. So it was the same class of cause, and it is fixed.
