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

Each run goes through File > New world and sets seed 4242 and a 256x160 grid, and checks that the request carries them. It clicks Create for real: the footer button on
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

With the compute path pinned, every dump is identical across the three forms: request, plan, parameters before and after, and
all four hashes. (The height hash is of a 16-bit export at 2048 px, so "identical" is at that resolution; the other three are of
the engine's own lists.) The only form-dependent input is the boot default of `use_gpu`.

## The cause

`EngineBridge._ready` sets `use_gpu` from `default_gpu_on()` when the user has stored no choice. That function returns false for
touch-class devices, which includes `--force-touch`. This is **Ruling EY** (owner, 2026-11-14, "CPU on phones/tablets"), and
**Ruling EZ** item 2 (2026-11-15) re-confirmed it: *"User preference remains here with the current default behaviour."*

**What `use_gpu` changes in a generate today** (read at the symbols): only `cartalith_engine::GPU_PATH_KERNELS`, which are plate
assignment and the plate-base blur (`generate_terrain_inner`, the two `if p.use_gpu` branches). Stress and flexure stay on the CPU
on both paths. Erosion and the depression fills do not run on the GPU in a generate. The only GPU erosion kernel is thermal, in
`erode_op`, and that is the Erode tool (`erode_bridge.rs`), not generation. GPU stream-power erosion is a research track only
(Ruling AZ). Ruling EZ item 3's narrowing has landed (`PERFORMANCE_BENCHMARKS.md` §12.8): the two kernels give the same bits on
the PC, the 6T and the Pad 3, and the GPU path's digest is the same on all three.

The GPU path is still a different world from the CPU path. §12's per-kernel table (CPU inputs) finds plate assignment unlike the
CPU in 4 of 2 684 928 cells, and the blur unlike it by at most 3.0e-7 (f32 against the CPU's arithmetic). Those differences seed
everything downstream. `DECISIONS.md` §7a (owner, 2026-08-16) puts GPU paths under principled equivalence rather than parity with
the CPU, so this is allowed. The desktop's world therefore differs from the touch forms' world on the same PC.

**No code was changed.** The difference comes from a ruled default, and changing it would be a silent deviation from an owner
ruling.

## What the GPU path changes (default desktop vs default tablet, same seed)

- **Height:** 687 744 of 2 621 440 export pixels differ (26 %). The largest difference is 2 135 of 65 535 (3.3 % of the range),
  and the mean over all pixels is 1.45. The two exports are `default/desktop_gpu_height.png` and `default/tablet_cpu_height.png`.
- **Rivers:** 597 against 596.
- **Labels:** 3 of 54 generated labels differ, all of them landmarks (Ford, Gorge, Ridge, Waterfall), which follow the terrain.
- **Settlements:** identical (with the auto-populate passes Create runs).

GPU runs are repeatable on this machine: the default desktop run and all three GPU-stored runs give the same hash.

**The original symptom.** The batch B row's 28 against 27 labels came from `tools/shell_batch_b_labels_drive.gd`. It generates
through `EngineBridge.generate` without the auto-populate passes, so its settlements and labels differ from this drive's.
Re-run on the desktop with that drive, 2026-11-23: with the stored CPU choice the desktop gives **27 labels, hash 2798146760**, which
is batch B's tablet and phone list exactly. On its default (GPU) the desktop gives 28 labels, hash 4096503529. Logs:
`batch_b_labels_desktop_cpu.log` and `batch_b_labels_desktop_default.log`.

## The rule, and the options (owner question)

On every device measured, a seed gives one world per compute path: one for the CPU path and one for the GPU path (§12.8). The
form chooses the path only through Ruling EY's default. "Same world on every form and device" therefore holds when the path is the
same, and fails between a PC on its default and a touch device on its default.

- **A. Keep Ruling EY.** A PC world and a tablet world from one seed differ unless the user stores the same choice on both.
- **B. Default the CPU everywhere.** One seed then gives one world on every form and device. On the PC, §12.8 established no
  end-to-end speed difference between the paths, so the PC loses nothing measured. GPU stays a user choice in Preferences.
- **C. Default the GPU everywhere.** Also one world everywhere, because the GPU path is device-identical on the three devices
  measured. The touch devices would leave Ruling EY's CPU default. On the 6T, §12.8 measured the GPU path 3 % faster; the Pad 3
  showed no established difference.
- **D. Record the compute path in saves and shares.** A frozen town or a shared seed then names its path, and the opener uses it.

`TOWN_LAYOUT_SAVE_SCOPE.md` TL-5 depends on this choice.

## Test and mutant

- `form_determinism.py all` passes:
  - `default` must differ in `use_gpu` alone, asserted as literals (desktop `true`, tablet `false`, phone `false`);
  - `cpu` and `gpu` must give identical dumps across the three forms.

  Log: `all.log`.
- `form_determinism.py mutant` replaces the exact literal
  `param_set("use_gpu", default_gpu_on() if stored < 0 else stored == 1)` with `param_set("use_gpu", default_gpu_on())` in the
  scratch copy, so the boot ignores the stored choice. The CPU-stored desktop then generates on the GPU, the check goes red, and
  the result is **MUTANT KILLED** (`mutant.log`, `mutant_cpu/`). The file is restored in a `finally` and compared afterwards.

## Seen in passing, not fixed

On the phone form, `chronos_items.gd::_start_scan` (line 802) called `WorldGen.timelines_scan()` while the generate worker
held the engine. The result was `Gd<T>::bind() failed, already bound` and `SCRIPT ERROR: Bug: Invalid call error code 1337`. It
appeared in 5 of the 11 phone runs that generated, under all three GPU settings, and in none of the desktop or tablet runs. It did
not change any dump.
