"""Literal mutants for the 2026-11-28 shell leftovers. Each mutates ONE exact literal in the worktree (occurs exactly once;
BUILD_ERROR is kept apart from SURVIVED), runs the named drive on one form, and the drive must print a FAIL line. The file is
restored from its own bytes in a `finally` and its hash is compared with the pre-run value.
  python -I mutants.py <name> <form>
"""
import hashlib, os, pathlib, subprocess, sys
W = pathlib.Path("C:/cw/shellc/cartalith-native/godot-project")
M = {
 "pinch_guard_off": ("shell/viewport_host.gd", "if sig == _last_magnify_sig:", "if false and sig == _last_magnify_sig:", "res://tools/pinch_drive.gd"),
 "undrawn_keys_empty": ("map_overlay.gd", 'if lb.has("key") and _label_hidden(lb):', 'if lb.has("key") and false:', "res://tools/labels_hit_drive.gd"),
 "coast_arm_no_rebuild": ("shell/viewport_host.gd", "if province_view.texture != null and _engine_readable():", "if false and _engine_readable():", "res://tools/coast_raster_drive.gd"),
 "refresh_ignores_coast": ("shell/viewport_host.gd", '_bridge.province_boundary_texture(overlay.layer_visible("coast"))', "_bridge.province_boundary_texture(false)", "res://tools/coast_raster_drive.gd"),
}
name, form = sys.argv[1], sys.argv[2]
rel, old, new, script = M[name]
p = W / rel
orig = p.read_bytes()
h0 = hashlib.sha256(orig).hexdigest()
text = orig.decode("utf-8")
assert text.count(old) == 1, f"BUILD_ERROR: {old!r} occurs {text.count(old)} times"
try:
    p.write_bytes(text.replace(old, new).encode("utf-8"))
    r = subprocess.run([sys.executable, "-I", "C:/cw/shellc_run.py", form, script, f"mut_{name}_{form}"], capture_output=True, text=True, encoding="utf-8", errors="replace", env=dict(os.environ, SC_TIMEOUT="400"))
    out = r.stdout
    killed = any(l.startswith("FAIL") for l in out.splitlines())
    ran = "RESULT fails=" in out
    print(f"MUTANT {name} on {form}: {'KILLED' if killed else ('SURVIVED' if ran else 'BUILD_ERROR/NO RESULT')}")
    print("\n".join(l[:200] for l in out.splitlines() if l.startswith("FAIL")))
finally:
    p.write_bytes(orig)
    assert hashlib.sha256(p.read_bytes()).hexdigest() == h0, "RESTORE FAILED"
