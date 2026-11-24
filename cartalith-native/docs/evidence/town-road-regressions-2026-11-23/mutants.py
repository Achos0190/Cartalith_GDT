"""Literal mutants for the road-attachment fix (owner report 2026-11-23).

  python -I mutants.py WORKTREE TARGET_DIR SCRATCH_PROJECT_ROOT [unit|probe|all]

`unit`: each mutant against `cargo test -p cartalith-godot --lib bank_site`. `probe`: each mutant built into the dll and
copied into SCRATCH_PROJECT_ROOT/target/debug, then `_roadattach_probe.gd` on seed 483920 at 384 x 288 (headless) must
exit 1. Exact literal replace that must occur once; BUILD_ERROR is reported apart from SURVIVED; the file is restored in a
`finally` and its hash checked against the pre-run value."""
import hashlib, os, pathlib, shutil, subprocess, sys

wt, target, scratch = pathlib.Path(sys.argv[1]), sys.argv[2], pathlib.Path(sys.argv[3])
which = sys.argv[4] if len(sys.argv) > 4 else "all"
BS = "cartalith-native/crates/cartalith-godot/src/bank_site.rs"
LIB = "cartalith-native/crates/cartalith-godot/src/lib.rs"
GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
UNIT = [
    ("R1 the ramp's slope", BS, "(2.0 - t / full).clamp(0.0, 1.0)", "(3.0 - t / full).clamp(0.0, 1.0)"),
    ("R2 the short road's quarter", BS, "let full = full.min(len / 4.0);", "let full = full.min(len / 2.0);"),
    ("R3 the drawn end makes no crossing", BS, "    if count(&out) <= own {", "    if true {"),
    ("R4 the straddling segment", BS, "let segs = 0..(last + 1).min(moved.len() - 1);", "let segs = 0..last.min(moved.len() - 1);"),
    ("R5 the landing is dry", BS, "            if !wet(x) {", "            if wet(x) {"),
    ("R6 the end is the market", BS, "        out[0] = market;", "        out[0] = tail;"),
]
PROBE = [
    ("P1 the half-cell frame", BS, "line.iter().map(|p| (p.0 + 0.5, p.1 + 0.5)).collect();", "line.iter().map(|p| (p.0, p.1)).collect();"),
    ("P2 the town's bank move", BS, ".map(|s| self.town_origin(s).map_or((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5), |b| (b.at.0 + 0.5, b.at.1 + 0.5)))", ".map(|s| (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5))"),
    ("P3 the named town is its tid", LIB, "d.set(*k, s.tid as i64);", "d.set(*k, -1i64);"),
]

def run(cmd, cwd=None, timeout=3600):
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout, env=dict(os.environ, CARGO_TARGET_DIR=target))
    return p.returncode, p.stdout + p.stderr

def one(name, rel, old, new, probe):
    f = wt / rel
    src = f.read_bytes()
    h0 = hashlib.sha256(src).hexdigest()
    text = src.decode("utf-8")
    assert text.count(old) == 1, "%s: the literal occurs %d times" % (name, text.count(old))
    try:
        f.write_bytes(text.replace(old, new).encode("utf-8"))
        if not probe:
            rc, out = run(["cargo", "test", "-p", "cartalith-godot", "--lib", "bank_site"], cwd=wt / "cartalith-native")
            if "error[" in out or "could not compile" in out:
                return "BUILD_ERROR"
            return "KILLED" if rc != 0 else "SURVIVED"
        rc, out = run(["cargo", "build", "-p", "cartalith-godot"], cwd=wt / "cartalith-native")
        if rc != 0:
            return "BUILD_ERROR"
        shutil.copy(pathlib.Path(target) / "debug/cartalith_godot.dll", scratch / "target/debug/cartalith_godot.dll")
        p = subprocess.run([GODOT, "--headless", "--path", str(scratch / "godot-project"), "--script", "res://_roadattach_probe.gd", "--", "--seeds", "483920"],
                           capture_output=True, text=True, timeout=1800, env=dict(os.environ, APPDATA=str(scratch / "appdata_mut"), CARTALITH_CONFIG_PATH=str(scratch / "appdata_mut/c.cfg")))
        fails = [l for l in p.stdout.splitlines() if l.startswith("ROAD FAIL")]
        return ("KILLED (%s)" % fails[0][:120]) if p.returncode == 1 and fails else "SURVIVED (exit %d)" % p.returncode
    finally:
        f.write_bytes(src)
        assert hashlib.sha256(f.read_bytes()).hexdigest() == h0, "%s: restore failed" % rel

if __name__ == "__main__":
    os.makedirs(scratch / "appdata_mut", exist_ok=True)
    for (name, rel, old, new) in (UNIT if which in ("unit", "all") else []):
        print(name, "->", one(name, rel, old, new, False), flush=True)
    for (name, rel, old, new) in (PROBE if which in ("probe", "all") else []):
        print(name, "->", one(name, rel, old, new, True), flush=True)
    if which in ("probe", "all"):
        run(["cargo", "build", "-p", "cartalith-godot"], cwd=wt / "cartalith-native")
        shutil.copy(pathlib.Path(target) / "debug/cartalith_godot.dll", scratch / "target/debug/cartalith_godot.dll")
        print("restored the unmutated dll into", scratch)
