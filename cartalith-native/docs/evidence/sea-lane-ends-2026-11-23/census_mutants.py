"""Literal mutants of the sea-lane census's tightened fork bar (`sea_site_census.rs`, `FORK_PART_KM`), each judged by the
census on the owner's world (`SEA_CENSUS_ONLY=owner51203`, `SEA_CENSUS_PARAMS` = his parameter list, not committed).

  python -I census_mutants.py PARAMS_FILE

Exact literal replace (occurs exactly once), BUILD_ERROR kept apart from SURVIVED, the file restored in a `finally` and
its hash checked. KILLED = the two owner51203 rows differ from the unmutated run's."""
import hashlib, os, pathlib, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
F = ROOT / "crates" / "cartalith-godot" / "src" / "sea_site_census.rs"
MUTANTS = [
    ("part distance 5 km -> 1000 km", "const FORK_PART_KM: f64 = 5.0;", "const FORK_PART_KM: f64 = 1000.0;"),
    ("parted test -> any separation", "q.0 == li && far > hi &&", "q.0 == li && far > 0.0 &&"),
    ("fork needs touch AND part -> OR", "let fork = touch && samples", "let fork = touch || samples"),
]
env = dict(os.environ, SEA_CENSUS_ONLY="owner51203", SEA_CENSUS_PARAMS=sys.argv[1])
cmd = ["cargo", "test", "--release", "-p", "cartalith-godot", "--lib", "sea_site_census::sea_lane_census", "--",
       "--ignored", "--nocapture", "--test-threads", "1"]

def rows():
    r = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        return None
    return [l.split(" (")[0] for l in r.stdout.splitlines() if l.startswith("SEA owner51203")]

orig = F.read_bytes(); h0 = hashlib.sha256(orig).hexdigest()
base = rows()
print("BASE", *base, sep="\n  ")
try:
    for name, a, b in MUTANTS:
        t = orig.decode("utf-8")
        assert t.count(a) == 1, (name, t.count(a))
        F.write_bytes(t.replace(a, b).encode("utf-8"))
        got = rows()
        v = "BUILD_ERROR" if got is None else ("SURVIVED" if got == base else "KILLED")
        print("%-36s %s" % (name, v), *(got or []), sep="\n  ", flush=True)
finally:
    F.write_bytes(orig)
assert hashlib.sha256(F.read_bytes()).hexdigest() == h0
print("restored", h0[:12])
