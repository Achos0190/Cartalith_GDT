"""Ruling HS: named literal mutants on the setback's constants and comparisons, each against the unit tests that
protect it. Exact-literal replace (asserted to occur exactly once), BUILD_ERROR kept apart from SURVIVED, the file
restored in a `finally`.

  python -I mutants.py <cartalith-native dir> <CARGO_TARGET_DIR>
"""
import os, subprocess, sys

root, target = sys.argv[1], sys.argv[2]
U = "crates/cartalith-urban/src/setback.rs"
C = "crates/cartalith-civ/src/setback.rs"
G = "crates/cartalith-urban/src/generate.rs"
UT = ["cargo", "test", "-p", "cartalith-urban", "--lib", "setback"]
CT = ["cargo", "test", "-p", "cartalith-civ", "--lib", "setback::"]
FT = ["cargo", "test", "-p", "cartalith-civ", "--lib", "town_freeze"]
MUTANTS = [
    ("COAST_FRONT_M 70 -> 100", U, "pub const COAST_FRONT_M: f64 = 70.0;", "pub const COAST_FRONT_M: f64 = 100.0;", UT),
    ("COAST_CAP_M 600 -> 700", U, "pub const COAST_CAP_M: f64 = 600.0;", "pub const COAST_CAP_M: f64 = 700.0;", UT),
    ("RIVER_CAP_M 500 -> 550", U, "pub const RIVER_CAP_M: f64 = 500.0;", "pub const RIVER_CAP_M: f64 = 550.0;", UT),
    ("H_FLOOD_M 2 -> 3", U, "pub const H_FLOOD_M: f64 = 2.0;", "pub const H_FLOOD_M: f64 = 3.0;", UT),
    ("WALK_STEP_M 11 -> 22", U, "pub const WALK_STEP_M: f64 = 11.0;", "pub const WALK_STEP_M: f64 = 22.0;", UT),
    ("cap <= -> <", U, "!(f64::from(sc.sea_m[k]) <= COAST_CAP_M)", "!(f64::from(sc.sea_m[k]) < COAST_CAP_M)", UT),
    ("tie prefers lower ground", U, "(d == bd && h > bh)", "(d == bd && h < bh)", UT),
    ("walk ignores wetland", U, " && sc.bits[kp] & WETLAND == 0", "", UT),
    ("market never comes to the front", U, "if mc.dist(q) > COAST_FRONT_M {", "if false {", UT),
    ("walk terrace >= -> >", U, "if f64::from(sc.ground_m[kp]) - chan >= H_FLOOD_M", "if f64::from(sc.ground_m[kp]) - chan > H_FLOOD_M", UT),
    ("FINE_MAX_CELL_M 625 -> 700", C, "pub const FINE_MAX_CELL_M: f64 = 625.0;", "pub const FINE_MAX_CELL_M: f64 = 700.0;", CT),
    ("H_SURGE_M 4 -> 3", C, "pub const H_SURGE_M: f64 = 4.0;", "pub const H_SURGE_M: f64 = 3.0;", CT),
    ("D_SURGE_M 100 -> 90", C, "pub const D_SURGE_M: f64 = 100.0;", "pub const D_SURGE_M: f64 = 90.0;", CT),
    ("E_EXPOSED 0.55 -> 0.85", C, "pub const E_EXPOSED: f32 = 0.55;", "pub const E_EXPOSED: f32 = 0.85;", CT),
    ("D_CLIFF_M 50 -> 40", C, "pub const D_CLIFF_M: f64 = 50.0;", "pub const D_CLIFF_M: f64 = 40.0;", CT),
    ("H_CLIFF_M 15 -> 16", C, "pub const H_CLIFF_M: f64 = 15.0;", "pub const H_CLIFF_M: f64 = 16.0;", CT),
    ("FLOOD_SAFE 0.55 -> 0.6", C, "pub const FLOOD_SAFE: f32 = 0.55;", "pub const FLOOD_SAFE: f32 = 0.6;", CT),
    ("LF_CLIFF 1 -> 2", C, "pub const LF_CLIFF: u8 = 1;", "pub const LF_CLIFF: u8 = 2;", CT),
    ("LF_FLOODPLAIN 6 -> 5", C, "pub const LF_FLOODPLAIN: u8 = 6;", "pub const LF_FLOODPLAIN: u8 = 5;", CT),
    ("surge on a sheltered shore", C, "if l.exposed && l.sea_m <= D_SURGE_M", "if l.sea_m <= D_SURGE_M", CT),
    ("housing guard always keeps the rule", G, "if town.pop >= HARBOUR_HOUSING_KEEP * reference.pop.min(reference.pop_target) && crossings(&town) >= crossings(&reference) {", "if crossings(&town) >= crossings(&reference) {", FT),
]
env = dict(os.environ, CARGO_TARGET_DIR=target)
killed = 0
for name, rel, old, new, cmd in MUTANTS:
    path = os.path.join(root, rel)
    src = open(path, encoding="utf-8").read()
    assert src.count(old) == 1, f"{name}: the literal occurs {src.count(old)} times"
    try:
        open(path, "w", encoding="utf-8").write(src.replace(old, new))
        r = subprocess.run(cmd, cwd=root, env=env, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if "error[" in out or "could not compile" in out and "test result" not in out:
            verdict = "BUILD_ERROR"
        elif r.returncode != 0 and "FAILED" in out:
            verdict = "KILLED"
            killed += 1
        elif r.returncode == 0:
            verdict = "SURVIVED"
        else:
            verdict = "OTHER(%d)" % r.returncode
    finally:
        open(path, "w", encoding="utf-8").write(src)
    print(f"{verdict:12} {name}", flush=True)
print(f"{killed} of {len(MUTANTS)} killed")
