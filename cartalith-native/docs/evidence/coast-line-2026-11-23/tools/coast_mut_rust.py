"""Exact-literal mutants of coast_polylines; each must turn `cargo test -p cartalith-civ --lib coast_` red. Restores in finally."""
import subprocess, pathlib, os, hashlib
P = pathlib.Path("C:/cw/coast/cartalith-native/crates/cartalith-civ/src/border_terrain.rs")
orig = P.read_bytes()
h0 = hashlib.sha256(orig).hexdigest()
NL = chr(13) + chr(10)
M = [
 ("R1 crossing at the midpoint, not the interpolated zero", "let t = (v0 / (v0 - v1)) as f32;", "let t = 0.5f32; let _ = (v0, v1);"),
 ("R2 no Newton step on the midpoint (a chord across the hyperbola)", "for _ in 0..2 {\n                    let s =", "for _ in 0..0 {\n                    let s ="),
 ("R3 saddle paired by the opposite centre sign", "if (a + b + c + d > 0.0) == (a > 0.0) { vec![(et, er), (el, eb)] }", "if (a + b + c + d > 0.0) != (a > 0.0) { vec![(et, er), (el, eb)] }"),
 ("R4 run floor 16 -> 1 (a perpendicular border cuts the coast)", "const COAST_RUN_MIN: usize = 16;", "const COAST_RUN_MIN: usize = 1;"),
 ("R5 run floor 16 -> 400 (a parallel border cuts nothing)", "const COAST_RUN_MIN: usize = 16;", "const COAST_RUN_MIN: usize = 400;"),
 ("R6 near distance 1.25 -> 0.5 (a border one cell inland is not near)", "const COAST_BORDER_NEAR: f32 = 1.25;", "const COAST_BORDER_NEAR: f32 = 0.5;"),
 ("R7 hidden mask always 0", "r.0 |= flags[i];", "r.0 |= 0 * flags[i];"),
 ("R8 the vertical edge's interpolation swaps x and y", "if vert { (i as f32 + 0.5, j as f32 + t + 0.5) }", "if vert { (i as f32 + t + 0.5, j as f32 + 0.5) }"),
 ("R9 the half-cell offset of the cell centres dropped", "(i as f32 + t + 0.5, j as f32 + 0.5) }", "(i as f32 + t, j as f32 + 0.5) }"),
 ("R10 coarse strides all 1 (no decimation)", "pub const COAST_STRIDES: [usize; 3] = [4, 8, 16];", "pub const COAST_STRIDES: [usize; 3] = [1, 1, 1];"),
 ("R11 the coarsest level is only 8", "pub const COAST_STRIDES: [usize; 3] = [4, 8, 16];", "pub const COAST_STRIDES: [usize; 3] = [4, 8, 8];"),
]
res = []
try:
    for name, old, new in M:
        s = orig.decode("utf-8")
        old = old.replace(chr(10), NL); new = new.replace(chr(10), NL)
        assert s.count(old) == 1, (name, s.count(old))
        P.write_bytes(s.replace(old, new).encode("utf-8"))
        r = subprocess.run(["cargo", "test", "-p", "cartalith-civ", "--lib", "coast_"], cwd="C:/cw/coast/cartalith-native", env=dict(os.environ, CARGO_TARGET_DIR="C:/cw/coast_target"), capture_output=True, text=True)
        out = r.stdout + r.stderr
        if "error[" in out or "could not compile" in out:
            res.append((name, "BUILD_ERROR"))
        elif r.returncode != 0:
            res.append((name, "KILLED"))
        else:
            res.append((name, "SURVIVED"))
        print(res[-1], flush=True)
finally:
    P.write_bytes(orig)
assert hashlib.sha256(P.read_bytes()).hexdigest() == h0
print("restored; residue check ok")
