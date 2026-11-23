"""Ruling HU: named literal mutants on lines whose effect the unit suite does not reach, run against
`_towncensus_probe.gd` on the world named for each. Each replaces one exact literal (it must occur exactly once),
rebuilds `cartalith-godot`, copies the dll into the mutant project copy (its mtime checked against the build), runs
the census and must turn it red (exit 1 with a `TOWN FAIL` line). The source is restored in a `finally` and its hash
checked. BUILD_ERROR is never KILLED.

  python -I census_mutants.py [name-prefix]     (CARGO_TARGET_DIR=C:/cw/hu_target; mutant project copy C:/cw/hu_mut)

Verdicts to stdout and census_mutants_result.txt beside this file; each census log to C:/cw/hu_mut/logs/."""
import os, pathlib, shutil, subprocess, sys, hashlib

ROOT = pathlib.Path("C:/cw/hu/cartalith-native")
TARGET = pathlib.Path(os.environ.get("CARGO_TARGET_DIR", "C:/cw/hu_target")) / "debug" / "cartalith_godot.dll"
MUT = pathlib.Path("C:/cw/hu_mut")
GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
U, C, G = "crates/cartalith-urban/src/", "crates/cartalith-civ/src/", "crates/cartalith-godot/src/"
OWNER = ["--seeds", "532187", "--gw", "2048", "--gh", "1311", "--params", "C:/cw/owner_repro/seed532187_params.txt", "--cell", "1478,1159", "--cell-min-housed", "90", "--max-quartered", "0", "--rows"]
W246 = ["--seeds", "246371", "--gw", "384", "--gh", "288", "--rows"]
WHOLE = "        let len: f64 = chain.iter().map(|&e| g.nodes[g.edges[e].a].pt().dist(g.nodes[g.edges[e].b].pt())).sum();"
WET = "        let len: f64 = chain.iter().map(|&e| { let (a, b) = (g.nodes[g.edges[e].a].pt(), g.nodes[g.edges[e].b].pt()); let k = (a.dist(b) / WET_STREET_STEP_M).ceil().max(1.0) as usize; a.dist(b) * (0..=k).filter(|&i| site.is_water(a.lerp(b, i as f64 / k as f64))).count() as f64 / (k + 1) as f64 }).sum();"
# (name, file, [(old, new), ...], census args)
M = [
    ("C1 the span over the water alone (Ruling HQ's measure), on the owner's world", U + "cleanup.rs", [(WHOLE, WET)], OWNER),
    ("C2 the square crossing lands before the centreline (unit survivor H8)", U + "cleanup.rs", [(".skip_while(|&x| x.dist(from) < d.len())", "")], W246),
    ("C3 each ferry landing a quarter of the way into the water (the landing gate's positive control)", U + "cleanup.rs", [("            g.ferries.push((a, b));", "            g.ferries.push((a.lerp(b, 0.25), b.lerp(a, 0.25)));")], OWNER),
]

def build():
    p = subprocess.run(["cargo", "build", "-p", "cartalith-godot"], cwd=ROOT, capture_output=True, text=True)
    return p.returncode == 0

def census(args, log):
    dll = MUT / "target/debug/cartalith_godot.dll"
    shutil.copy2(TARGET, dll)
    assert dll.stat().st_mtime >= TARGET.stat().st_mtime - 1 and dll.stat().st_size == TARGET.stat().st_size, "stale dll"
    p = subprocess.run([GODOT, "--headless", "--path", str(MUT / "godot-project"), "--script", "res://_towncensus_probe.gd", "--"] + args, capture_output=True, text=True, timeout=3600)
    out = p.stdout + p.stderr
    log.write_text(out)
    import re
    fails = re.findall(r"TOWN FAIL: (.*)", out)
    verdict = "KILLED" if p.returncode == 1 and fails else ("SURVIVED" if p.returncode == 0 else "ERROR (exit %d)" % p.returncode)
    return verdict + "".join("  | " + f[:150] for f in fails[:3])

(MUT / "logs").mkdir(parents=True, exist_ok=True)
res = []
only = sys.argv[1] if len(sys.argv) > 1 else ""
for name, f, reps, args in M:
    if not name.startswith(only): continue
    files = [(ROOT / f, reps)]
    saved = [(p, p.read_bytes()) for p, _ in files]
    try:
        ok = True
        for p, rs in files:
            t = p.read_bytes().decode("utf-8")
            crlf = "\r\n" in t   # a file the Windows tools saved with CRLF: the literals' line breaks follow it
            for old, new in rs:
                if crlf:
                    old, new = old.replace("\n", "\r\n"), new.replace("\n", "\r\n")
                if t.count(old) != 1:
                    ok = False; res.append((name, "LITERAL_NOT_ONCE (%d)" % t.count(old)))
                t = t.replace(old, new)
            p.write_bytes(t.encode("utf-8"))
        if ok:
            v = census(args, MUT / "logs" / (name.split()[0] + ".txt")) if build() else "BUILD_ERROR"
            res.append((name, v))
    finally:
        for p, b in saved:
            p.write_bytes(b)
            assert hashlib.sha256(p.read_bytes()).hexdigest() == hashlib.sha256(b).hexdigest()
    print("%-70s %s" % res[-1], flush=True)
build()   # leave the target holding the unmutated build
out = pathlib.Path(__file__).with_name("census_mutants_result.txt")
with out.open("a") as fh:
    fh.write("\n".join("%-70s %s" % r for r in res) + "\n")
