"""Form determinism check (2026-11-23): one seed, the same New World request, on desktop, tablet and phone.

Runs `godot-project/tools/form_determinism_drive.gd` (a real File > New world, a real click on Create) once per form
in the real shell, each with a FRESH scratch APPDATA so no setting leaks from one form into the next, and diffs the
dumps: the request, the auto-populate plan, the full parameter table at boot and after the run, and the digests of the
heightmap, the rivers, the settlements and the generated labels.

Why the compute path is pinned by the stored choice: `use_gpu` is the one parameter whose boot default depends on the
form, by owner decision (Ruling EY, reaffirmed by Ruling EZ item 2: touch devices default to the CPU, the PC to the
GPU; `EngineBridge.default_gpu_on`), and the GPU path makes a different world from the CPU path. With the stored
explicit choice (`[gpu] enabled`, what Preferences > GPU acceleration writes) the same on every form, the worlds must
be byte-identical; at the default they must differ in `use_gpu` alone, true on the desktop and false on touch.

  GODOT_EXE=<console exe> [CARTALITH_DLL=<dll>] [FORMDET_ROOT=<scratch>] python -I form_determinism.py <command>:
  python -I form_determinism.py run TAG GPU FORM [OUT]         GPU is 0, 1 or "-" (the default); FORM desktop|tablet|phone
  python -I form_determinism.py check DIR equal|ruled_default    exit 0 when the dumps in DIR meet the expectation
  python -I form_determinism.py all OUT                         default, CPU-pinned and GPU-pinned on all three forms
  python -I form_determinism.py mutant OUT                      the boot ignores the stored choice: CPU-pinned must FAIL
                                                                (exit 0 killed, 1 survived, 2 broken: a run that did not dump)

Window lock (MISTAKES.md): C:/cw/.win_lock is taken with mkdir and an owner file written at once; it is removed only
when this process made it, and held only while Godot runs. Never `--headless` (the shell needs a window).
"""
import json, os, pathlib, shutil, subprocess, sys, time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fennara_drive as fd  # noqa: E402  (the launcher this repository already has)

GODOT = os.environ.get("GODOT_EXE", "")   # the Godot 4.7.2 console exe; required (`run` refuses without it)
NATIVE = pathlib.Path(__file__).resolve().parents[1]
DLL = os.environ.get("CARTALITH_DLL", str(NATIVE / "target" / "debug" / "cartalith_godot.dll"))
ROOT = pathlib.Path(os.environ.get("FORMDET_ROOT", "C:/cw/formdet_scratch"))
FORMS = ("desktop", "tablet", "phone")
## The boot line of `EngineBridge._ready` the mutant breaks: the stored explicit choice is ignored, so a PC whose user
## chose the CPU generates on the GPU anyway. Exact literal, must occur once.
MUTANT_OLD = 'param_set("use_gpu", default_gpu_on() if stored < 0 else stored == 1)'
MUTANT_NEW = 'param_set("use_gpu", default_gpu_on())'


def run(tag, gpu, form, out):
    """One form's run. True only when the run completed, no check failed, and it wrote a fresh dump."""
    if not GODOT:
        sys.exit("set GODOT_EXE to the Godot 4.7.2 console exe")
    dump = pathlib.Path(out) / tag / f"{form}.json"
    dump.unlink(missing_ok=True)   # a stale dump from an earlier run must never be read as this one's
    proj = fd.scratch_shell(DLL, ROOT)
    app, loc = ROOT / f"appdata_{tag}_{form}", ROOT / f"localappdata_{tag}_{form}"
    cfg = app / "cartalith_settings.cfg"
    for v in (app, loc):
        shutil.rmtree(v, ignore_errors=True)
        v.mkdir(parents=True)
    if gpu in ("0", "1"):
        cfg.write_text("[gpu]\n\nenabled=%s\n" % gpu)
    env = {"APPDATA": str(app), "LOCALAPPDATA": str(loc), "CARTALITH_CONFIG_PATH": str(cfg),
           "FORMDET_OUT": str(pathlib.Path(out) / tag)}
    if not (proj / ".godot").exists():
        subprocess.run([GODOT, "--headless", "--path", str(proj), "--import"], capture_output=True, timeout=1800, env=dict(os.environ, **env))
    lock, owner = fd.LOCK, fd.LOCK / "owner_form_determinism.txt"
    while True:
        try:
            lock.mkdir()
            owner.write_text(f"form_determinism.py pid {os.getpid()} {tag} {form}\n")
            break
        except FileExistsError:
            time.sleep(5)
    try:
        st, lines = fd.drive(GODOT, proj, "res://tools/form_determinism_drive.gd", "res://shell/app.tscn",
                             fd.FORMS[form], 300, env, pathlib.Path(out) / tag / f"cap_{form}")
    finally:
        owner.unlink()
        lock.rmdir()
    print(f"== {tag} {form}: {st.get('status')}")
    print("\n".join(lines))
    return st.get("status") == "completed" and not any(l.startswith("FAIL") for l in lines) and dump.exists()


def check(d, expect):
    d = pathlib.Path(d)
    dumps = {f: json.load(open(d / f"{f}.json")) for f in FORMS if (d / f"{f}.json").exists()}
    ok = len(dumps) == 3
    if not ok:
        print(f"FAIL only {sorted(dumps)} dumped in {d}")
    for f, x in dumps.items():
        print(f, x["canvas"], "use_gpu", x["params_boot"].get("use_gpu"), x["hash"])
        if x["hash"]["settlements"] < 1 or x["hash"]["labels"] < 10 or len(x["params_after"]) < 100:
            ok = False
            print(f"FAIL {f}: an empty world or table diffs equal everywhere (control)")
    if expect == "ruled_default":
        ## Literals, not the default read back: Ruling EY's desktop GPU and touch CPU.
        for f, want in (("desktop", True), ("tablet", False), ("phone", False)):
            if f in dumps and dumps[f]["params_boot"].get("use_gpu") is not want:
                ok = False
                print(f"FAIL {f}: use_gpu at boot {dumps[f]['params_boot'].get('use_gpu')}, Ruling EY says {want}")
    for f in FORMS[1:]:
        if "desktop" not in dumps or f not in dumps:
            continue
        a, b = dumps["desktop"], dumps[f]
        for k in ("request", "auto_populate_plan", "params_boot", "params_after", "hash"):
            if isinstance(a[k], dict):
                diff = {x: (a[k].get(x), b[k].get(x)) for x in sorted(set(a[k]) | set(b[k])) if a[k].get(x) != b[k].get(x)}
            else:
                diff = {} if a[k] == b[k] else {"value": (a[k], b[k])}
            if expect == "ruled_default" and k.startswith("params"):
                bad = {x: v for x, v in diff.items() if x != "use_gpu"}
            elif expect == "ruled_default":
                bad = {}   # the GPU world differs from the CPU world; only the parameter routing is checked here
            else:
                bad = diff
            if bad:
                ok = False
            print(f"{'FAIL' if bad else 'ok  '} desktop vs {f} {k}: {diff if diff else 'equal'}")
    print("PASS" if ok else "FAIL")
    return ok


def main(a):
    if a[0] == "run":
        return 0 if run(a[1], a[2], a[3], a[4] if len(a) > 4 else "C:/cw/formdet_out") else 1
    if a[0] == "check":
        return 0 if check(a[1], a[2]) else 1
    if a[0] == "all":
        out = a[1]
        ran = [run(tag, gpu, f, out) for tag, gpu in (("default", "-"), ("cpu", "0"), ("gpu", "1")) for f in FORMS]
        if not all(ran):
            print("FAIL a run did not complete or dump")
            return 1
        r = [check(f"{out}/default", "ruled_default"), check(f"{out}/cpu", "equal"), check(f"{out}/gpu", "equal")]
        return 0 if all(r) else 1
    if a[0] == "mutant":
        out = a[1]
        p = fd.scratch_shell(DLL, ROOT) / "shell" / "engine_bridge.gd"
        src = p.read_text(encoding="utf-8")
        assert src.count(MUTANT_OLD) == 1, "mutant literal not found exactly once"
        try:
            p.write_text(src.replace(MUTANT_OLD, MUTANT_NEW), encoding="utf-8", newline="\n")
            ran = [run("mutant_cpu", "0", f, out) for f in FORMS]
        finally:
            p.write_text(src, encoding="utf-8", newline="\n")
        assert p.read_text(encoding="utf-8") == src
        if not all(ran):
            print("MUTANT BROKEN: a run did not complete or dump (neither killed nor survived)")
            return 2
        survived = check(f"{out}/mutant_cpu", "equal")
        ## Killed for the right reason only: the CPU-stored desktop booted on the GPU.
        gpu_desktop = json.load(open(f"{out}/mutant_cpu/desktop.json"))["params_boot"].get("use_gpu") is True
        if not survived and not gpu_desktop:
            print("MUTANT BROKEN: the check went red, but not because the desktop ignored its stored CPU choice")
            return 2
        print("MUTANT SURVIVED" if survived else "MUTANT KILLED")
        return 1 if survived else 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
