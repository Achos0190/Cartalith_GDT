"""Ruling HU: named literal mutants on the markers' job wiring, which no unit test reaches, run against
`tools/marker_async_drive.gd` in the real shell (desktop). Each replaces one exact literal (it must occur exactly once)
and must turn the drive red (a FAIL line, exit 1). A Rust mutant rebuilds `cartalith-godot` and copies the dll into
the project copy (its mtime and size checked); a GDScript mutant edits the project copy's file. Every file is
restored in a `finally` and its hash checked; the dll is rebuilt unmutated at the end.

  python -I drive_mutants.py [name-prefix]     (CARGO_TARGET_DIR=C:/cw/hu_target; project copy C:/cw/hu_dev)

Verdicts to stdout and drive_mutants_result.txt beside this file."""
import hashlib, os, pathlib, shutil, subprocess, sys

HERE = pathlib.Path(__file__).parent
ROOT = pathlib.Path("C:/cw/hu/cartalith-native")
DEV = pathlib.Path("C:/cw/hu_dev")
TARGET = pathlib.Path(os.environ.get("CARGO_TARGET_DIR", "C:/cw/hu_target")) / "debug" / "cartalith_godot.dll"
M = [
    ("W1 the markers read the table in place (the job never used)", ROOT / "crates/cartalith-godot/src/bank_site.rs",
     "let markets = if self.markers_async.get() { self.market_points_marker() } else { self.market_points() };", "let markets = self.market_points();", True),
    ("W2 the shell never turns the job on", DEV / "godot-project/shell/engine_bridge.gd",
     "\t\tworld_gen.set_markers_async(true)\n", "\t\tworld_gen.set_markers_async(false)\n", False),
    ("W3 a landed table is never re-asked for", DEV / "godot-project/shell/engine_bridge.gd",
     "\tif state == 1:\n\t\tbank_params_changed.emit()\n", "\tif state == 1:\n\t\tpass\n", False),
]

def build():
    return subprocess.run(["cargo", "build", "-p", "cartalith-godot"], cwd=ROOT, capture_output=True, text=True).returncode == 0

def install():
    dll = DEV / "target/debug/cartalith_godot.dll"
    shutil.copy2(TARGET, dll)
    assert dll.stat().st_size == TARGET.stat().st_size and dll.stat().st_mtime >= TARGET.stat().st_mtime - 1, "stale dll"

def drive(tag):
    out = pathlib.Path("C:/cw/hu_ev/drive_mut_" + tag)
    out.mkdir(parents=True, exist_ok=True)
    p = subprocess.run([sys.executable, "-I", str(HERE / "run_drive.py"), "desktop", str(out), "900", "res://tools/marker_async_drive.gd"],
                       capture_output=True, text=True, env=dict(os.environ, HU_ROOT=str(DEV)), timeout=1800)
    (out / "log.txt").write_text(p.stdout + p.stderr)
    fails = [l for l in p.stdout.splitlines() if l.startswith("FAIL")]
    return ("KILLED" if p.returncode == 1 and fails else ("SURVIVED" if p.returncode == 0 else "ERROR (exit %d)" % p.returncode)), fails

res = []
only = sys.argv[1] if len(sys.argv) > 1 else ""
for name, path, old, new, rust in M:
    if not name.startswith(only): continue
    src = path.read_bytes(); h = hashlib.sha256(src).hexdigest(); text = src.decode("utf-8")
    if "\r\n" in text:
        old, new = old.replace("\n", "\r\n"), new.replace("\n", "\r\n")
    if text.count(old) != 1:
        res.append((name, "LITERAL_NOT_ONCE (%d)" % text.count(old), [])); print(res[-1], flush=True); continue
    try:
        path.write_bytes(text.replace(old, new).encode("utf-8"))
        if rust and not build():
            v, fails = "BUILD_ERROR", []
        else:
            if rust: install()
            v, fails = drive(name.split()[0])
    finally:
        path.write_bytes(src)
    assert hashlib.sha256(path.read_bytes()).hexdigest() == h, "restore failed for %s" % path
    if rust:   # the next mutant must run on the unmutated engine
        assert build(), "the unmutated build failed"; install()
    res.append((name, v, fails)); print("%-62s %s" % (name, v), *fails[:3], sep="\n   ", flush=True)
with (HERE / "drive_mutants_result.txt").open("a") as fh:
    for name, v, fails in res:
        fh.write("%-62s %s\n" % (name, v) + "".join("   %s\n" % f for f in fails[:4]))
