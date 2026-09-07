"""Mutate every constant the thumbnail path introduces, in both directions."""
import io, os, shutil, subprocess, sys

GD = r'C:\Users\Vincent\Cartalith_GDT\cartalith-native\godot-project\shell\open_project_dialog.gd'
PROJ = r'C:\Users\Vincent\Cartalith_GDT\cartalith-native\godot-project'
GODOT = r'C:\Users\Vincent\Desktop\Godot_v4.7.1-stable_win64_console.exe'
PRISTINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'pristine.gd')

M = [
    # (label, find, replace)
    ("THUMB_W 96->95", "const THUMB_W := 96", "const THUMB_W := 95"),
    ("THUMB_W 96->97", "const THUMB_W := 96", "const THUMB_W := 97"),
    ("THUMB_H 72->71", "const THUMB_H := 72", "const THUMB_H := 71"),
    ("THUMB_H 72->73", "const THUMB_H := 72", "const THUMB_H := 73"),
    ("SEA[0].r 10->4", "Color(10 / 255.0, 28 / 255.0, 46 / 255.0)", "Color(4 / 255.0, 28 / 255.0, 46 / 255.0)"),
    ("SEA[0].r 10->16", "Color(10 / 255.0, 28 / 255.0, 46 / 255.0)", "Color(16 / 255.0, 28 / 255.0, 46 / 255.0)"),
    ("SEA[1].g 86->80", "Color(26 / 255.0, 86 / 255.0, 140 / 255.0)", "Color(26 / 255.0, 80 / 255.0, 140 / 255.0)"),
    ("SEA[1].g 86->92", "Color(26 / 255.0, 86 / 255.0, 140 / 255.0)", "Color(26 / 255.0, 92 / 255.0, 140 / 255.0)"),
    ("SEA[2].b 196->190", "Color(70 / 255.0, 140 / 255.0, 196 / 255.0)", "Color(70 / 255.0, 140 / 255.0, 190 / 255.0)"),
    ("SEA[2].b 196->202", "Color(70 / 255.0, 140 / 255.0, 196 / 255.0)", "Color(70 / 255.0, 140 / 255.0, 202 / 255.0)"),
    ("LAND_AT .38->.32", "[0.0, 0.18, 0.38, 0.58, 0.78, 1.0]", "[0.0, 0.18, 0.32, 0.58, 0.78, 1.0]"),
    ("LAND_AT .38->.44", "[0.0, 0.18, 0.38, 0.58, 0.78, 1.0]", "[0.0, 0.18, 0.44, 0.58, 0.78, 1.0]"),
    ("LAND_AT .18->.12", "[0.0, 0.18, 0.38, 0.58, 0.78, 1.0]", "[0.0, 0.12, 0.38, 0.58, 0.78, 1.0]"),
    ("LAND_AT .18->.24", "[0.0, 0.18, 0.38, 0.58, 0.78, 1.0]", "[0.0, 0.24, 0.38, 0.58, 0.78, 1.0]"),
    ("LAND[0].g 122->116", "Color(47 / 255.0, 122 / 255.0, 68 / 255.0)", "Color(47 / 255.0, 116 / 255.0, 68 / 255.0)"),
    ("LAND[0].g 122->128", "Color(47 / 255.0, 122 / 255.0, 68 / 255.0)", "Color(47 / 255.0, 128 / 255.0, 68 / 255.0)"),
    ("LAND[2].r 201->195", "Color(201 / 255.0, 178 / 255.0, 74 / 255.0)", "Color(195 / 255.0, 178 / 255.0, 74 / 255.0)"),
    ("LAND[2].r 201->207", "Color(201 / 255.0, 178 / 255.0, 74 / 255.0)", "Color(207 / 255.0, 178 / 255.0, 74 / 255.0)"),
    ("LAND[3].b 72->66", "Color(150 / 255.0, 112 / 255.0, 72 / 255.0)", "Color(150 / 255.0, 112 / 255.0, 66 / 255.0)"),
    ("LAND[3].b 72->78", "Color(150 / 255.0, 112 / 255.0, 72 / 255.0)", "Color(150 / 255.0, 112 / 255.0, 78 / 255.0)"),
    ("LAND[4] 140->134", "Color(140 / 255.0, 140 / 255.0, 140 / 255.0)", "Color(134 / 255.0, 140 / 255.0, 140 / 255.0)"),
    ("LAND[4] 140->146", "Color(140 / 255.0, 140 / 255.0, 140 / 255.0)", "Color(146 / 255.0, 140 / 255.0, 140 / 255.0)"),
    ("LAND[5].b 250->244", "Color(248 / 255.0, 248 / 255.0, 250 / 255.0)", "Color(248 / 255.0, 248 / 255.0, 244 / 255.0)"),
    ("LAND[5].b 250->254", "Color(248 / 255.0, 248 / 255.0, 250 / 255.0)", "Color(248 / 255.0, 248 / 255.0, 254 / 255.0)"),
    ("depth split .5->.45", "if d < 0.5 \\", "if d < 0.45 \\"),
    ("depth split .5->.55", "if d < 0.5 \\", "if d < 0.55 \\"),
    ("shallow div .5->.45", "HYPSO_SEA[1], d / 0.5)", "HYPSO_SEA[1], d / 0.45)"),
    ("shallow div .5->.55", "HYPSO_SEA[1], d / 0.5)", "HYPSO_SEA[1], d / 0.55)"),
    ("deep div .5->.45", "(d - 0.5) / 0.5)", "(d - 0.5) / 0.45)"),
    ("deep div .5->.55", "(d - 0.5) / 0.5)", "(d - 0.5) / 0.55)"),
    ("byte stride 4->3", "bytes.decode_float((row + sx) * 4)", "bytes.decode_float((row + sx) * 3)"),
    ("byte stride 4->5", "bytes.decode_float((row + sx) * 4)", "bytes.decode_float((row + sx) * 5)"),
    ("len guard 4->3", "if bytes.size() != gw * gh * 4:", "if bytes.size() != gw * gh * 3:"),
    ("len guard 4->5", "if bytes.size() != gw * gh * 4:", "if bytes.size() != gw * gh * 5:"),
    ("len guard dropped", "if bytes.size() != gw * gh * 4:", "if bytes.size() < 0:"),
    ("sea range lo 0->gt0", "or not (sea >= 0.0 and sea <= 1.0) \\", "or not (sea > 0.0 and sea <= 1.0) \\"),
    ("sea range hi 1->2", "or not (sea >= 0.0 and sea <= 1.0) \\", "or not (sea >= 0.0 and sea <= 2.0) \\"),
    ("sea range dropped", "or not (sea >= 0.0 and sea <= 1.0) \\", "or false \\"),
    ("grid guard 1->0", "if gw < 1 or gh < 1 or entry", "if gw < 0 or gh < 0 or entry"),
    ("grid guard 1->2", "if gw < 1 or gh < 1 or entry", "if gw < 2 or gh < 2 or entry"),
    ("cache key: no mtime", "FileAccess.get_modified_time(path)]", "0]"),
    ("cache key: no path", 'path.sha256_text().substr(0, 16)', '"kkkkkkkkkkkkkkkk"'),
    ("row stride dropped", "var row: int = sy * gw", "var row: int = sy * gh"),
    ("x clamp gw-1->gw-2", "mini(int(float(tx) * gw / THUMB_W), gw - 1)", "mini(int(float(tx) * gw / THUMB_W), gw - 2)"),
    ("y clamp gh-1->gh-2", "mini(int(float(ty) * gh / THUMB_H), gh - 1)", "mini(int(float(ty) * gh / THUMB_H), gh - 2)"),
    ("y clamp gh-1->gh", "mini(int(float(ty) * gh / THUMB_H), gh - 1)", "mini(int(float(ty) * gh / THUMB_H), gh)"),
]


def run():
    p = subprocess.run([GODOT, "--headless", "--script", "_thumbbench_probe.gd"],
                       cwd=PROJ, capture_output=True, text=True, timeout=300)
    out = p.stdout + p.stderr
    for line in out.splitlines():
        if line.strip().endswith("failure(s)"):
            return int(line.strip().split()[0]), out
    return -1, out


def check():
    p = subprocess.run([GODOT, "--headless", "--check-only", "--script", "shell/open_project_dialog.gd"],
                       cwd=PROJ, capture_output=True, text=True, timeout=180)
    return p.returncode


def main():
    shutil.copy(GD, PRISTINE)
    base = io.open(PRISTINE, encoding='utf-8').read()
    n, _ = run()
    print("baseline: %d failure(s)" % n)
    if n != 0:
        print("!! baseline is not green; stopping")
        return
    survivors = []
    try:
        for label, a, b in M:
            if base.count(a) != 1:
                print("%-24s  SKIP (find matches %d times)" % (label, base.count(a)))
                survivors.append(label + " [unmatched]")
                continue
            io.open(GD, 'w', encoding='utf-8', newline='\n').write(base.replace(a, b, 1))
            rc = check()
            if rc != 0:
                print("%-24s  parse error -> counted as killed" % label)
                continue
            k, out = run()
            state = "KILLED  (%d failing)" % k if k > 0 else ("SURVIVED" if k == 0 else "CRASHED -> killed")
            print("%-24s  %s" % (label, state))
            if k == 0:
                survivors.append(label)
    finally:
        shutil.copy(PRISTINE, GD)
    print("\nsurvivors: %d" % len(survivors))
    for s in survivors:
        print("  -", s)
    print("restored; check rc =", check())


main()
