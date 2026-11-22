---
name: cartalith-playtest
description: >
  Playtest, probe and bug-hunt the Cartalith Godot shell the way this project does it: windowed probe
  scenes under a scratch APPDATA, one run per form (desktop 1600x900, tablet 1280x800, phone 393x852 or
  1080x2340 at density 2.8125), real input events, framebuffer frames that are opened and looked at, and
  a mutant that must turn the probe red. Use when asked to playtest, click through, bug-hunt, verify a
  GUI change in the real app, or write a `_*_probe.gd`. Not for Rust-only changes (cargo test) and not
  for headless pixel checks (they pass vacuously).
---

# Cartalith playtest

Design: `TOOLING_PLAN.md` (M3). Read `MISTAKES.md`'s preflight first; its rows on pixels, windows,
`.dll` freshness and the window lock apply to every step below.

## Before any run
1. `cargo build -p cartalith-godot` if any `.rs` changed. A probe loads `target/debug/cartalith_godot.dll`, and `cargo test` does not rebuild it (a stale dll tests the previous engine). Compare mtimes and say so.
2. Parse-check every `.gd` touched plus `shell/app.gd`: `Godot_v4.7.2-stable_win64_console.exe --headless --check-only --script <f>`; grade on stderr "Parse Error" / "Failed to load script", not the exit code. A probe with a parse error hangs while holding the lock.
3. Copy the project to a scratch dir with its own `config/name`; run with `APPDATA=<empty dir>` so the owner's settings, worlds and Vault are never read, and restore any setting the probe changes on every exit path.

## The run
- **Windowed, never `--headless`** for anything pixel-based (framebuffer and `ImageTexture.update()` are no-ops there). Take `C:/cw/.win_lock` with `mkdir`; release it only if you made it, chained with `&&`, never after `;`. Hard `timeout` always. Headless runs need no lock.
- **Forms**, one run each: desktop `--resolution 1600x900`; tablet `--resolution 1280x800 -- --force-touch`; phone `--resolution 393x852 -- --force-touch` (or `-- --vp 1080x2340 --density 2.8125 --force-touch`). The form is decided at boot, so each is its own launch.
- Probe args come after `--`: `--shots <dir> --tag <name> --seed <n> --vp WxH --density D --force-touch --mutate <kind>`; unknown args abort with exit 2. Exit 0 held, 1 a check failed, 2 could not run.
- **Real events**: press the button through its `pressed`, or send `InputEventMouse*`/`InputEventKey` through the viewport; a handler called directly is not a click. Fennara's `ctx.click_button`/`tap_key` do this (`cartalith-native/tools/fennara_drive.py`).
- **Frames**: save with `--shots`, then OPEN them (Read the PNG) at every zoom the change touches (x1/x4/x16/x64: zoom gains detail is a hard requirement). A frame is the renderer's answer, not engine state: dump the data before naming a cause.
- **Assertions**: thresholds are palette-bound (this machine boots light); force the palette, measure a control state, and make every measurement carry a positive control (an empty diff is a failure, never a pass). Assert literals, not constants read back.
- **Mutant**: every probe has a `--mutate` mode, or a Python exact-literal mutation (occurs once, BUILD_ERROR separate from SURVIVED, restore in `finally`). A probe that passes its own mutant is a decoration.
- A failure while other lanes run: re-run once before concluding the code broke.

## Bug-hunt order
Walk menus (`_fgui2_walk_probe.gd` presses every window-opening command), then each form, then odd sizes; check every window for overflow past the screen edge (`get_combined_minimum_size().x` against the screen). Report: form, steps, the frame path, the symbol, the check that now goes red.

## Driving the real shell without writing a probe (TP-2/TP-3, built 2026-11-22; full usage in `TOOLING_PLAN.md`)
```
python -I cartalith-native/tools/fennara_drive.py <godot 4.7.2 console exe> --shell desktop|tablet|phone res://tools/my_script.gd        --dll <target>/debug/cartalith_godot.dll --out <dir>
```
It copies `godot-project` to a scratch root (`C:/cw/drive_scratch`: own `config/name`, empty `APPDATA`; delete it after a `.rs` or
dll change), runs the real shell at that form (desktop 1600x900; tablet 1700x1000 `--force-touch --density=1.0`; phone 450x975
`--force-touch --density 1.094`), holds `C:/cw/.win_lock` only while the window is open (waits if another lane has it), kills only its
own process tree, and exits 0 / 1 on a `FAIL` line. Your script (`extends RefCounted`, `func run(ctx)`, in `godot-project/tools/`)
uses `DriveCtx`: `dump_ui(ctx, contains)` (every visible control: path, class, text, rect, enabled, tooltip, plus open popup rows),
`click(ctx, pos)`, `type_text(ctx, node, s)` (real key events), `menu_open(ctx, "File", "New world")` (bar menu, phone MENU tab, or
`"ring"`), `rect_of(control)` (click space: never `get_global_rect()` inside a dialog), and Fennara's `ctx.capture(label)` for the
frame to Read. Boot: desktop/tablet show a modal welcome card ("Continue without a world" first); the phone shows the worlds list
("+ NEW WORLD" then CANCEL). `drive_proof.gd` is the worked example on all three forms; `drive_mutant.py` shows how a helper is
mutation-tested. Parse-check your script headless in the scratch root before spending a lock turn. Popup rows are chosen with Down and
Enter keys, not a mouse click; the ring path is untested. The older form, `fennara_drive.py <godot> <project copy> res://script.gd -- <args>`,
still runs a script against any project copy you made yourself.
