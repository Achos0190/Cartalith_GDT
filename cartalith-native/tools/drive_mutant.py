"""Mutants for the TP-3 helpers (TOOLING_PLAN.md M2): each must turn `drive_proof.gd` RED on the desktop form.

  python -I drive_mutant.py GODOT DLL OUT_DIR

Exact-literal replacement in `godot-project/tools/drive_ctx.gd`: the literal must occur exactly once, the original is
restored in a `finally`, and a run that cannot start (exit 2, no status) is reported as BUILD_ERROR, never as KILLED.
Baseline first: a proof that is not green unmutated proves nothing. Exit 0 = baseline green and every mutant killed.
"""
import pathlib, subprocess, sys

HERE = pathlib.Path(__file__).resolve().parent
CTX = HERE.parents[0] / "godot-project" / "tools" / "drive_ctx.gd"
MUTANTS = {
    # type_text drops its last character: the seed read-back from dump_ui must disagree
    "type_text_drops_last": ("	for ch in s:\n		var code", "	for ch in s.substr(0, maxi(0, s.length() - 1)):\n		var code"),
    # the menu row is never activated (Escape instead of Enter): the screen the proof expects never opens
    "popup_row_not_activated": ("	await tap_key(ctx, KEY_ENTER)\n", "	await tap_key(ctx, KEY_ESCAPE)\n"),
    # rects inside an embedded window lose the window offset and scale: the welcome click misses
    "rect_offset_dropped": ("return c.get_screen_transform() * Rect2(Vector2.ZERO, c.size)", "return c.get_global_rect()"),
}


def run(godot, dll, out, tag):
    r = subprocess.run([sys.executable, "-I", str(HERE / "fennara_drive.py"), godot, "--shell", "desktop", "res://tools/drive_proof.gd",
                        "--dll", dll, "--out", f"{out}/{tag}"], capture_output=True, text=True, encoding="utf-8", errors="replace")
    return r.returncode, r.stdout


if __name__ == "__main__":
    godot, dll, out = sys.argv[1:4]
    rc, txt = run(godot, dll, out, "baseline")
    print("baseline rc", rc); print(txt[-600:])
    assert rc == 0, "baseline is not green: the mutants prove nothing"
    orig = CTX.read_text(encoding="utf-8", newline="")
    bad = 0
    try:
        for name, (a, b) in MUTANTS.items():
            if "\r\n" in orig:   # a Windows checkout (autocrlf) has CRLF; the literals above are written with LF
                a, b = a.replace("\n", "\r\n"), b.replace("\n", "\r\n")
            assert orig.count(a) == 1, f"{name}: literal must occur once, found {orig.count(a)}"
            CTX.write_text(orig.replace(a, b), encoding="utf-8", newline="")
            rc, txt = run(godot, dll, out, name)
            verdict = "KILLED" if rc == 1 else ("BUILD_ERROR" if rc == 2 else "SURVIVED")
            bad += verdict != "KILLED"
            print(name, verdict, "| failing lines:", [l for l in txt.splitlines() if "FAIL" in l][:2])
    finally:
        CTX.write_text(orig, encoding="utf-8", newline="")
    sys.exit(1 if bad else 0)
