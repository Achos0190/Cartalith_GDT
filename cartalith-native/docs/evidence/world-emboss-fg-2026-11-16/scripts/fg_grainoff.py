"""Build a scratch DLL whose land jitter is off (every lattice at its mean), for bar B8, then restore.
usage: python fg_grainoff.py <repo cartalith-native dir> <target dir> <out dll>"""
import subprocess, sys, os, shutil, hashlib
R, T, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
p = R + "/crates/cartalith-godot/src/render.rs"
src = open(p, "rb").read()
h0 = hashlib.sha256(src).hexdigest()
o = b"    if a.land_grain_warp <= 0.0 {\n        return (vnoise(x * 0.06, y * 0.06, 11)"
if src.count(o) != 1:
    o = o.replace(b"\n", b"\r\n")
assert src.count(o) == 1
n = o.replace(b"    if a.land_grain_warp <= 0.0 {", b"    if a.land_grain_warp > 0.0 { return (0.5, 0.5, 0.5); }\n    if a.land_grain_warp <= 0.0 {", 1)
try:
    open(p, "wb").write(src.replace(o, n))
    r = subprocess.run(["cargo", "build", "-p", "cartalith-godot"], cwd=R, env=dict(os.environ, CARGO_TARGET_DIR=T), capture_output=True, text=True)
    print(r.stderr[-400:])
    shutil.copy(T + "/debug/cartalith_godot.dll", OUT)
finally:
    open(p, "wb").write(src)
assert hashlib.sha256(open(p, "rb").read()).hexdigest() == h0
print("restored")
