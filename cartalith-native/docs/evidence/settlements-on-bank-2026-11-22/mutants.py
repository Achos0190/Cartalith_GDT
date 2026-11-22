"""Ruling HG literal mutants. Exact literal replace, occurs exactly once, BUILD_ERROR apart from SURVIVED, restore in finally."""
import subprocess, sys, pathlib, hashlib, os
ROOT = pathlib.Path("C:/cw/hg2/cartalith-native")
BANK = ROOT / "crates/cartalith-godot/src/bank_site.rs"
ENV = dict(os.environ, CARGO_TARGET_DIR="C:/cw/hg2_target")
LIB = ["cargo", "test", "-p", "cartalith-godot", "--lib", "bank_site"]
MUTANTS = [
    ("M1 the gap dropped from the clearance", BANK, "    let c = half + q.gap;\n", "    let c = half;\n", LIB),
    ("M1b the gap dropped from the covering test", BANK, "let depth = half + self.gap - hit.d;", "let depth = half - hit.d;", LIB),
    ("M2 MARKET_FIRST_BANK reversed", BANK, "pub const MARKET_FIRST_BANK: (f64, f64) = (-1.0, -0.13);", "pub const MARKET_FIRST_BANK: (f64, f64) = (1.0, 0.13);", LIB),
    ("M3 dry ground not required", BANK, "(q.dry)(p.0, p.1) && q.covering(p).is_none()", "q.covering(p).is_none()", LIB),
    ("M4 other strokes not cleared", BANK, "(q.dry)(p.0, p.1) && q.covering(p).is_none()", "(q.dry)(p.0, p.1)", LIB),
    ("M5 a hidden river counts as drawn", BANK, "(alpha >= crate::river_stroke::PICK_MIN_VISIBILITY)", "(alpha >= 0.0)", LIB),
    ("M6 half the stroke read as a quarter", BANK, "f64::from(w_px) / 2.0 / f64::from(ppc)", "f64::from(w_px) / 4.0 / f64::from(ppc)", LIB),
    ("M7 SEARCH_REACH 2.0 -> 0.5", BANK, "pub const SEARCH_REACH: f64 = 2.0;", "pub const SEARCH_REACH: f64 = 0.5;", LIB),
    ("M8 SEARCH_DIRS 16 -> 2", BANK, "pub const SEARCH_DIRS: usize = 16;", "pub const SEARCH_DIRS: usize = 2;", LIB),
    ("M9 CLEAR_EPS 1e-7 -> -1e-3", BANK, "pub const CLEAR_EPS: f64 = 1e-7;", "pub const CLEAR_EPS: f64 = -1e-3;", LIB),
    ("M10 ON_LINE_EPS 1e-6 -> 1.0", BANK, "pub const ON_LINE_EPS: f64 = 1e-6;", "pub const ON_LINE_EPS: f64 = 1.0;", LIB),
    ("M11 the pixel floor left out of the search reach", BANK, "let reach = self.gap + self.wide.max(floor);", "let reach = self.gap + self.wide;", LIB),
    ("M15 no second try without the gap", BANK, "bank_point_at_gap(q, centre, prefer).or_else(|| if q.gap > 0.0 { bank_point_at_gap(&BankQuery { gap: 0.0, ..*q }, centre, prefer) } else { None })", "bank_point_at_gap(q, centre, prefer)", LIB),
    ("M12 app default off", ROOT / "crates/cartalith-godot/src/params.rs", "    p.civ.bank_sites = true;\n", "    p.civ.bank_sites = false;\n", ["cargo", "test", "-p", "cartalith-godot", "--lib", "a_new_world_after_opening_a_pre_ruling_er_archive"]),
    ("M13 engine default on", ROOT / "crates/cartalith-engine/src/lib.rs", "                bank_sites: false,\n", "                bank_sites: true,\n", ["cargo", "test", "-p", "cartalith-godot", "--test", "params_mapping"]),
    ("M14 a save without the key loads on", ROOT / "crates/cartalith-godot/src/params.rs", '("civ.bank_sites", Value::Bool(false)),', '("civ.bank_sites", Value::Bool(true)),', ["cargo", "test", "-p", "cartalith-godot", "--lib", "a_new_world_after_opening_a_pre_ruling_er_archive"]),
]
only = sys.argv[1:]
for name, path, old, new, cmd in MUTANTS:
    if only and not any(name.startswith(o + " ") for o in only):
        continue
    src = path.read_bytes()
    text = src.decode("utf-8")
    assert text.count(old) == 1, (name, text.count(old))
    try:
        path.write_bytes(text.replace(old, new).encode("utf-8"))
        r = subprocess.run(cmd, cwd=ROOT, env=ENV, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if "error[" in out or "error: could not compile" in out:
            verdict = "BUILD_ERROR"
        elif r.returncode != 0 and "test result: FAILED" in out:
            failed = [l.split()[1] for l in out.splitlines() if l.startswith("test ") and l.rstrip().endswith("FAILED")]
            verdict = "KILLED by " + ", ".join(sorted(set(failed)))
        elif "test result: ok" in out and " 0 passed" not in out.split("test result: ok")[1][:40]:
            verdict = "SURVIVED"
        else:
            verdict = "UNCLEAR rc=%d" % r.returncode
        print(name, "->", verdict, flush=True)
    finally:
        path.write_bytes(src)
        assert hashlib.sha256(path.read_bytes()).hexdigest() == hashlib.sha256(src).hexdigest()
print("done; residue check:", subprocess.run(["git", "status", "--short"], cwd=ROOT, capture_output=True, text=True).stdout.count("MUTANT"))
