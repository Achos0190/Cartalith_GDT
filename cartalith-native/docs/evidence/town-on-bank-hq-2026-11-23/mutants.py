"""Ruling HQ: named literal mutants on the unit suite. Each replaces one exact literal (it must occur exactly once),
runs the tests named for it, and must turn them red. The file is restored in a `finally` and its hash checked. A
mutant that does not build is BUILD_ERROR, never KILLED. The wiring mutants (generate_pass, the adapter, the warm-up)
are `census_mutants.py`'s.

  python -I mutants.py [name-prefix]        (set CARGO_TARGET_DIR first)

Verdicts go to stdout and to mutants_result.txt beside this file."""
import pathlib, subprocess, sys, hashlib

ROOT = pathlib.Path("C:/cw/hq/cartalith-native")
U, C, G = "crates/cartalith-urban/src/", "crates/cartalith-civ/src/", "crates/cartalith-godot/src/"
# (name, file, old, new, crate, test filter)
M = [
    ("N1 BANK_SITES_FAR_BANK_MAX_RATIO 2.0 to 1.0", U + "riverbank.rs", "pub const BANK_SITES_FAR_BANK_MAX_RATIO: f64 = 2.0;", "pub const BANK_SITES_FAR_BANK_MAX_RATIO: f64 = 1.0;", "cartalith-urban", "far_bank"),
    ("N1b BANK_SITES_FAR_BANK_MAX_RATIO 2.0 to 20.0", U + "riverbank.rs", "pub const BANK_SITES_FAR_BANK_MAX_RATIO: f64 = 2.0;", "pub const BANK_SITES_FAR_BANK_MAX_RATIO: f64 = 20.0;", "cartalith-urban", "far_bank"),
    ("N2 BANK_SITES_FAR_BANK_TRIES 40 000 to 2 600", U + "growth.rs", "pub const BANK_SITES_FAR_BANK_TRIES: i32 = 40_000;", "pub const BANK_SITES_FAR_BANK_TRIES: i32 = 2_600;", "cartalith-urban", "far_bank"),
    ("N3 far_bank_max_ratio's switch inverted", U + "riverbank.rs", "    if bank_sites { BANK_SITES_FAR_BANK_MAX_RATIO } else { FAR_BANK_MAX_RATIO }", "    if !bank_sites { BANK_SITES_FAR_BANK_MAX_RATIO } else { FAR_BANK_MAX_RATIO }", "cartalith-urban", "far_bank"),
    ("N4 second_pass reads Ruling FB's cap for every town", U + "riverbank.rs", "        let max = far_bank_max_ratio(t.bank_sites);", "        let max = far_bank_max_ratio(false);", "cartalith-urban", "second_pass_asks"),
    ("N5 grow_far_bank's tries ignore the switch", U + "growth.rs", "    let max_tries = if bank_sites { BANK_SITES_FAR_BANK_TRIES } else { FAR_BANK_TRIES };", "    let max_tries = FAR_BANK_TRIES;", "cartalith-urban", "far_bank"),
    ("N6 BRIDGE_MAX_SPAN_M 400 to 4 000", U + "cleanup.rs", "pub const BRIDGE_MAX_SPAN_M: f64 = 400.0;", "pub const BRIDGE_MAX_SPAN_M: f64 = 4000.0;", "cartalith-urban", "a_crossing_longer"),
    ("N6b BRIDGE_MAX_SPAN_M 400 to 250", U + "cleanup.rs", "pub const BRIDGE_MAX_SPAN_M: f64 = 400.0;", "pub const BRIDGE_MAX_SPAN_M: f64 = 250.0;", "cartalith-urban", "a_crossing_longer"),
    ("N7 the prune does not keep a ferry's landings", U + "cleanup.rs", " || c.is_some_and(|c| landed.contains(&c));", ";", "cartalith-urban", "a_crossing_longer"),
    ("N8 the span is whole street lengths, not the water's share", U + "cleanup.rs", "                a.dist(b) * (0..=k).filter(|&i| site.is_water(a.lerp(b, i as f64 / k as f64))).count() as f64 / (k + 1) as f64", "                a.dist(b) + 0.0 * (0..=k).filter(|&i| site.is_water(a.lerp(b, i as f64 / k as f64))).count() as f64 / (k + 1) as f64", "cartalith-urban", "a_crossing_longer"),
    ("N9 a landing at the street's dry node, not the water's edge", U + "cleanup.rs", ".take_while(|&x| !site.is_water(x)).last().unwrap_or(p);", ".take_while(|_| false).last().unwrap_or(p);", "cartalith-urban", "a_crossing_longer"),
    ("N9b a landing only where a street leaves the chain", U + "cleanup.rs", "                if slot.is_none_or(|(r, _)| rank < r) {", "                if rank == 0 && slot.is_none_or(|(r, _)| rank < r) {", "cartalith-urban", "a_crossing_longer"),
    ("N10 no ferry detail is emitted", U + "generate.rs", "details.extend(g.ferries.iter().enumerate()", "details.extend(g.ferries.iter().take(0).enumerate()", "cartalith-civ", "too_wide_to_bridge"),
    ("N11 the spike's base never goes", U + "wallhug.rs", "            for base in [before, after] {", "            for base in [before, after].into_iter().take(0) {", "cartalith-urban", "a_wall_spike"),
    ("N12 SHORE_ROUTE_CLEAR_M 6 to 0", U + "wallhug.rs", "pub const SHORE_ROUTE_CLEAR_M: f64 = 6.0;", "pub const SHORE_ROUTE_CLEAR_M: f64 = 0.0;", "cartalith-urban", "a_wall_spike"),
    ("N13 a gated chord is routed too", U + "wallhug.rs", " || w.iter().any(|&p| water_gates.iter().any(|&g| g.dist(p) <= WATER_GATE_KEEP_M)) {", " {", "cartalith-urban", "a_wall_spike"),
    ("N14 SHORE_ROUTE_MARGIN_M 200 to 0", U + "wallhug.rs", "pub const SHORE_ROUTE_MARGIN_M: f64 = 200.0;", "pub const SHORE_ROUTE_MARGIN_M: f64 = 0.0;", "cartalith-urban", "a_wall_spike"),
    ("N15 SHORE_ROUTE_CELL_M 8 to 80", U + "wallhug.rs", "pub const SHORE_ROUTE_CELL_M: f64 = 8.0;", "pub const SHORE_ROUTE_CELL_M: f64 = 80.0;", "cartalith-urban", "a_wall_spike"),
    ("N16 the route is not pulled straight", U + "wallhug.rs", "let j = (i + 1..route.len()).rev().find(|&j| wet_on(route[i], route[j]).is_empty()).unwrap_or(i + 1);", "let j = i + 1;", "cartalith-urban", "a_wall_spike"),
    ("N17 a self-crossing route is returned", U + "wallhug.rs", "    (moved && !ring_self_crosses(&out)).then_some(out)", "    moved.then_some(out)", "cartalith-urban", "a_wall_spike"),
    ("N18 URBAN_RULES_VERSION 4 to 5", C + "town_freeze.rs", "pub const URBAN_RULES_VERSION: u32 = 4;", "pub const URBAN_RULES_VERSION: u32 = 5;", "cartalith-civ", "the_rules_version"),
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
