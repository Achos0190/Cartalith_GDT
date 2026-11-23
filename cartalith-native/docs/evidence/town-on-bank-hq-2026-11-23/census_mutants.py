"""Ruling HQ: named literal mutants on WIRING lines no unit fixture reaches (Ruling HN's four survivors, U14 U16 U18
U19, and this ruling's own wiring), run against `_towncensus_probe.gd`. Each replaces one exact literal (it must occur
exactly once), rebuilds `cartalith-godot`, copies the dll into the mutant project copy (its mtime checked against the
build), runs the census on the world named for it, and must turn it red (exit 1 with a `TOWN FAIL` line). The source
is restored in a `finally` and its hash checked. BUILD_ERROR is never KILLED.

  python -I census_mutants.py [name-prefix]     (CARGO_TARGET_DIR=C:/cw/hq_target; mutant project copy C:/cw/hq_mut)

Verdicts to stdout and census_mutants_result.txt beside this file; each census log to C:/cw/hq_mut/logs/."""
import os, pathlib, shutil, subprocess, sys, hashlib

ROOT = pathlib.Path("C:/cw/hq/cartalith-native")
TARGET = pathlib.Path(os.environ.get("CARGO_TARGET_DIR", "C:/cw/hq_target")) / "debug" / "cartalith_godot.dll"
MUT = pathlib.Path("C:/cw/hq_mut")
GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
U, C, G = "crates/cartalith-urban/src/", "crates/cartalith-civ/src/", "crates/cartalith-godot/src/"
OWNER = ["--seeds", "532187", "--gw", "2048", "--gh", "1311", "--params", "C:/cw/owner_repro/seed532187_params.txt", "--cell", "1478,1159", "--cell-min-housed", "90", "--max-quartered", "0", "--rows"]
W384 = ["--seeds", "483920,912345,246371,83744", "--gw", "384", "--gh", "288", "--max-quartered", "6", "--rows"]
W246 = ["--seeds", "246371", "--gw", "384", "--gh", "288", "--rows"]
W2048 = ["--seeds", "483920", "--gw", "2048", "--gh", "1536", "--max-quartered", "2", "--rows"]
# (name, file, [(old, new), ...], census args)
M = [
    ("U14 the wet street sweep is never called", U + "generate.rs", [("    if opts.bank_sites {\n        crate::cleanup::sweep_wet_streets(&site, &mut g, anchors.market);", "    if false {\n        crate::cleanup::sweep_wet_streets(&site, &mut g, anchors.market);")], OWNER),
    ("U16 ground past a river's end is never given the market's bank", U + "generate.rs", [("    if opts.bank_sites {\n        site.bank_ref = Some(anchors.market);", "    if false {\n        site.bank_ref = Some(anchors.market);")], W384),
    ("U18 the rampart sweep is never given the market", U + "generate.rs", [("opts.bank_sites.then_some(anchors.market));", "None);")], OWNER),
    ("U19 a house off the box is never swept", U + "generate.rs", [("|| (opts.bank_sites && buildings[i].poly.iter().any(|&v| site.off_box(v)))", "|| (false && buildings[i].poly.iter().any(|&v| site.off_box(v)))")], W384),
    ("N2 BANK_SITES_FAR_BANK_TRIES 40 000 to 2 600 (survives the unit suite)", U + "growth.rs", [("pub const BANK_SITES_FAR_BANK_TRIES: i32 = 40_000;", "pub const BANK_SITES_FAR_BANK_TRIES: i32 = 2_600;")], OWNER),
    ("N5 grow_far_bank's tries ignore the switch (survives the unit suite)", U + "growth.rs", [("    let max_tries = if bank_sites { BANK_SITES_FAR_BANK_TRIES } else { FAR_BANK_TRIES };", "    let max_tries = FAR_BANK_TRIES;")], OWNER),
    ("W1 grow_far_bank is never told the switch", U + "generate.rs", [("epochs, share, far_centre, opts.bank_sites);", "epochs, share, far_centre, false);")], OWNER),
    ("W2 the generator's wall is never routed round the water", U + "wallhug.rs", [("    let routed = ring_round_water(r0.as_deref().unwrap_or(ring), &gates, &|p| site.is_water(p));", "    let routed: Option<Vec<Vec2>> = None;")], W2048),
    # W3 (the adapter's wall never routed round the strokes) SURVIVED on 246371 at 384 cells in the first run: the
    # generator's routing already left no wall over a drawn river, so the adapter's call was removed, not kept unproven.
    # W4 is read, not graded: with U18 the gate must count towns; with the constant 15 000 m as well it must count none.
    ("W4 census: MARKET_STREET_NEAR_M 150 to 15 000, with U18", G + "town_measures.rs", [("pub(crate) const MARKET_STREET_NEAR_M: f64 = 150.0;", "pub(crate) const MARKET_STREET_NEAR_M: f64 = 15000.0;")], OWNER),
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
    near = [int(x) for x in re.findall(r"TOWN   on : .*no street near the market (\d+)", out)]
    tag = " (no street near the market: %s)" % near
    return tag + " " + ("KILLED" if p.returncode == 1 and "TOWN FAIL" in out else ("SURVIVED" if p.returncode == 0 else "ERROR (exit %d)" % p.returncode))

(MUT / "logs").mkdir(parents=True, exist_ok=True)
res = []
only = sys.argv[1] if len(sys.argv) > 1 else ""
for name, f, reps, args in M:
    if not name.startswith(only): continue
    files = [(ROOT / f, reps)]
    if name.startswith("W4"):   # with U18, so the gate it measures has a town to see
        files.append((ROOT / U / "generate.rs", [("opts.bank_sites.then_some(anchors.market));", "None);")]))
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
