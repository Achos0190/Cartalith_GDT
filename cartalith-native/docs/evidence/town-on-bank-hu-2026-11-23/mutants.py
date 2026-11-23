"""Ruling HU: named literal mutants on the unit suite. Each replaces one exact literal (it must occur exactly once),
runs the tests named for it, and must turn them red. The file is restored in a `finally` and its hash checked. A
mutant that does not build is BUILD_ERROR, never KILLED. The wiring mutants (the markers' job in the real shell) are
`drive_mutants.py`'s.

  python -I mutants.py [name-prefix]        (set CARGO_TARGET_DIR first)

Verdicts go to stdout and to mutants_result.txt beside this file."""
import pathlib, subprocess, sys, hashlib

ROOT = pathlib.Path("C:/cw/hu/cartalith-native")
U, C, G = "crates/cartalith-urban/src/", "crates/cartalith-civ/src/", "crates/cartalith-godot/src/"
WHOLE = "        let len: f64 = chain.iter().map(|&e| g.nodes[g.edges[e].a].pt().dist(g.nodes[g.edges[e].b].pt())).sum();"
WET = "        let len: f64 = chain.iter().map(|&e| { let (a, b) = (g.nodes[g.edges[e].a].pt(), g.nodes[g.edges[e].b].pt()); let k = (a.dist(b) / WET_STREET_STEP_M).ceil().max(1.0) as usize; a.dist(b) * (0..=k).filter(|&i| site.is_water(a.lerp(b, i as f64 / k as f64))).count() as f64 / (k + 1) as f64 }).sum();"
# (name, file, old, new, crate, test filter)
M = [
    ("H1 the span over the water alone (Ruling HQ's measure)", U + "cleanup.rs", WHOLE, WET, "cartalith-urban", "a_crossing_longer"),
    ("H2 BRIDGE_MAX_SPAN_M 400 to 4 000", U + "cleanup.rs", "pub const BRIDGE_MAX_SPAN_M: f64 = 400.0;", "pub const BRIDGE_MAX_SPAN_M: f64 = 4000.0;", "cartalith-urban", "a_crossing_longer"),
    ("H3 BRIDGE_MAX_SPAN_M 400 to 350", U + "cleanup.rs", "pub const BRIDGE_MAX_SPAN_M: f64 = 400.0;", "pub const BRIDGE_MAX_SPAN_M: f64 = 350.0;", "cartalith-urban", "a_crossing_longer"),
    ("H4 a ferried street's dry part is never laid again", U + "cleanup.rs", "    for s in &stubs {", "    for s in stubs.iter().take(0) {", "cartalith-urban", "a_crossing_longer"),
    ("H5 STUB_MIN_M 4 to 4 000", U + "cleanup.rs", "pub const STUB_MIN_M: f64 = 4.0;", "pub const STUB_MIN_M: f64 = 4000.0;", "cartalith-urban", "a_crossing_longer"),
    ("H6 FERRY_REACH_RIVERS 2 to 0", U + "cleanup.rs", "pub const FERRY_REACH_RIVERS: f64 = 2.0;", "pub const FERRY_REACH_RIVERS: f64 = 0.0;", "cartalith-urban", "a_ferry_whose_road"),
    ("H7 the square crossing is never tried", U + "cleanup.rs", "            && site.river.len() >= 2\n", "            && site.river.len() >= 2000\n", "cartalith-urban", "a_ferry_whose_road"),
    ("H9 the line is carried on from any wet node, not only a dead end", U + "cleanup.rs", "if !site.is_water(p) || g.nodes[n].adj.iter().any(|&j| j != e && g.edges[j].alive) {", "if !site.is_water(p) {", "cartalith-urban", "a_ferry_whose_road"),
    ("H10 URBAN_RULES_VERSION 5 to 6", C + "town_freeze.rs", "pub const URBAN_RULES_VERSION: u32 = 5;", "pub const URBAN_RULES_VERSION: u32 = 6;", "cartalith-civ", "the_rules_version"),
]

def run(crate, filt):
    p = subprocess.run(["cargo", "test", "-p", crate, "--lib", "--", filt], cwd=ROOT, capture_output=True, text=True)
    out = p.stdout + p.stderr
    if "error[" in out or "could not compile" in out: return "BUILD_ERROR"
    if "test result: FAILED" in out: return "KILLED"
    if "test result: ok" in out and " 0 passed" not in out: return "SURVIVED"
    return "NO_TESTS_RAN"

res = []
only = sys.argv[1] if len(sys.argv) > 1 else ""
for name, f, old, new, crate, filt in M:
    if not name.startswith(only): continue
    path = ROOT / f
    src = path.read_bytes(); h = hashlib.sha256(src).hexdigest()
    text = src.decode("utf-8")
    if chr(13) + chr(10) in text:   # a file saved with CRLF: the literals' line breaks follow it
        old, new = old.replace(chr(10), chr(13) + chr(10)), new.replace(chr(10), chr(13) + chr(10))
    if text.count(old) != 1:
        res.append((name, "LITERAL_NOT_ONCE (%d)" % text.count(old))); print(res[-1], flush=True); continue
    try:
        path.write_bytes(text.replace(old, new).encode("utf-8"))
        v = run(crate, filt)
    finally:
        path.write_bytes(src)
    assert hashlib.sha256(path.read_bytes()).hexdigest() == h, "restore failed for " + f
    res.append((name, v)); print("%-70s %s" % res[-1], flush=True)
out = pathlib.Path(__file__).with_name("mutants_result.txt")
out.write_text("\n".join("%-70s %s" % r for r in res) + "\n")
