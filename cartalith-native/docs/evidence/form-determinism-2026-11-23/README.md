# Form determinism, 2026-11-23

The `OUTSTANDING_WORK.md` row "Same request, different world by form" (shell batch B, `4fcf0d94`): one New World request
gave 28 generated labels on the desktop and 27 on the tablet and phone.

## How it was measured

`cartalith-native/tools/form_determinism.py all <out>` runs `godot-project/tools/form_determinism_drive.gd` in the real shell
nine times. It covers three forms (desktop 1600x900; tablet 1700x1000 `--force-touch`; phone 450x975 `--force-touch`) under three
GPU settings:

- the default, with no stored choice;
- the stored choice CPU, `[gpu] enabled=0`;
- the stored choice GPU, `[gpu] enabled=1`.

Each run uses a fresh scratch APPDATA and its own `CARTALITH_CONFIG_PATH`, so nothing carries over from one form to the next.

Each run goes through File > New world and sets seed 4242 and a 256x160 grid. It clicks Create for real: the footer button on
desktop and tablet, and §6.7's CREATE WORLD on the phone. When the run finishes, including the auto-populate passes, it dumps:

- the dialog's request;
- the auto-populate plan;
- the full parameter table (156 keys) at boot and after the run;
- MD5 hashes of the heightmap (16-bit PNG at 2048), the rivers (`get_rivers(0)`), the settlements (`get_settlements()`) and the
  generated labels.

`form_determinism.py check` diffs those dumps against the desktop's. The DLL was built from `2405895c` (this tree). All runs
were on this PC (RX 7800 XT).

## Result

| configuration | parameters that differ (desktop vs tablet, phone) | height MD5 desktop / tablet / phone | rivers | settlements MD5 |
|---|---|---|---|---|
| default | `use_gpu` only: desktop `true`, tablet and phone `false` | `59766e4a…` / `c604568f…` / `c604568f…` | 597 / 596 / 596 | equal (`537c4466…`) |
| CPU stored on all three | none | `c604568f…` on all three | 596 | equal |
| GPU stored on all three | none | `59766e4a…` on all three | 597 | equal |

With the compute path pinned, the three forms are byte-identical in every dump: request, plan, parameters before and after, and
all four hashes. The only form-dependent input is the boot default of `use_gpu`.

## The cause

`EngineBridge._ready` sets `use_gpu` from `default_gpu_on()` when the user has stored no choice. That function returns false for
touch-class devices, which includes `--force-touch`. This is **Ruling EY** (owner, 2026-11-14, "CPU on phones/tablets"), and
**Ruling EZ** item 2 (2026-11-15) re-confirmed it: *"User preference remains here with the current default behaviour."*

The GPU path makes a different world from the CPU path, because warp and heterogeneity use the GPU-safe noise (§7c), and that is
deliberate (§7a, principled equivalence). So the desktop's world differs from the touch forms' world on the same PC.

**No code was changed.** The difference comes from a ruled default, and changing it would be a silent deviation from an owner
ruling.

## What the GPU path changes (default desktop vs default tablet, same seed)

- **Height:** 687 744 of 2 621 440 export pixels differ (26 %). The largest difference is 2 135 of 65 535 (3.3 % of the range),
  and the mean over all pixels is 1.45. The two exports are `default/desktop_gpu_height.png` and `default/tablet_cpu_height.png`.
- **Rivers:** 597 against 596.
- **Labels:** 3 of 54 generated labels differ, all of them landmarks (Ford, Gorge, Ridge, Waterfall), which follow the terrain.
- **Settlements:** identical.

GPU runs are repeatable on this machine: the default desktop run and all three GPU-stored runs give the same hash.

`PERFORMANCE_BENCHMARKS.md` §12 records that the GPU world also differs between devices: warp, heterogeneity and weather hash
differently on the PC, the 6T and the Pad 3. The PC GPU is 1.5 to 25 times faster per kernel, but no faster end to end (§12.6).

## The rule, and the options (owner question)

"A seed gives the same world on every form and every device" holds today only on the CPU path, which is the touch default.

- **A. Keep Ruling EY.** The PC defaults to the GPU, so a PC world and a tablet world from one seed differ unless the user stores
  CPU.
- **B. Default the CPU everywhere.** One seed then gives one world on every form and device. The PC loses nothing measurable end
  to end (§12.6), and GPU stays a user choice in Preferences.
- **C. Finish Ruling EZ item 3.** Keep only device-identical kernels on the GPU path. That makes the GPU world the same across
  devices, but it remains a different world from the CPU path.
- **D. Record the compute path in saves and shares.** A frozen town or shared seed then names its path.

`TOWN_LAYOUT_SAVE_SCOPE.md` TL-5 depends on this choice.

## Test and mutant

- `form_determinism.py all` passes:
  - `default` must differ in `use_gpu` alone, asserted as literals (desktop `true`, tablet `false`, phone `false`);
  - `cpu` and `gpu` must be byte-identical across the three forms.

  Log: `all.log`.
- `form_determinism.py mutant` replaces the exact literal
  `param_set("use_gpu", default_gpu_on() if stored < 0 else stored == 1)` with `param_set("use_gpu", default_gpu_on())` in the
  scratch copy, so the boot ignores the stored choice. The CPU-stored desktop then generates on the GPU, the check goes red, and
  the result is **MUTANT KILLED** (`mutant.log`, `mutant_cpu/`). The file is restored in a `finally` and compared afterwards.

## Seen in passing, not fixed

On the phone form, `chronos_items.gd::_start_scan` (line 802) called `WorldGen.timelines_scan()` while the generate worker
held the engine. The result was `Gd<T>::bind() failed, already bound` and `SCRIPT ERROR: Bug: Invalid call error code 1337`. It
appeared in 4 of the 7 phone runs that generated, under all three GPU settings, and in none of the desktop or tablet runs. It did
not change any dump.
