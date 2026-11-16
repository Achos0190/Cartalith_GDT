"""Mutation run for the low-relief fade: exact literal replace (must occur exactly once), build errors
separate from survivors, the file restored in a `finally`, hash checked against the pre-run value."""
import hashlib, subprocess, sys, os
R = "C:/cw/fgleft/cartalith-native"
REN = R + "/crates/cartalith-godot/src/render.rs"
T = ["cargo", "test", "-p", "cartalith-godot", "--test", "canopy_and_mottle"]
M = [
    ("far_lo", "pub const LAND_GRAIN_FAR_KM_PX: (f64, f64) = (0.5, 2.0);", "pub const LAND_GRAIN_FAR_KM_PX: (f64, f64) = (0.4, 2.0);"),
    ("far_hi", "pub const LAND_GRAIN_FAR_KM_PX: (f64, f64) = (0.5, 2.0);", "pub const LAND_GRAIN_FAR_KM_PX: (f64, f64) = (0.5, 3.0);"),
    ("flat_lo", "pub const LAND_GRAIN_FLAT_SLOPE: (f64, f64) = (0.002, 0.010);", "pub const LAND_GRAIN_FLAT_SLOPE: (f64, f64) = (0.003, 0.010);"),
    ("flat_hi", "pub const LAND_GRAIN_FLAT_SLOPE: (f64, f64) = (0.002, 0.010);", "pub const LAND_GRAIN_FLAT_SLOPE: (f64, f64) = (0.002, 0.012);"),
    ("keep", "pub const LAND_GRAIN_FLAT_KEEP: f64 = 0.25;", "pub const LAND_GRAIN_FLAT_KEEP: f64 = 0.3;"),
    ("no_far", "1.0 - far * flat * (1.0 - LAND_GRAIN_FLAT_KEEP)", "1.0 - flat * (1.0 - LAND_GRAIN_FLAT_KEEP)"),
    ("no_flat", "1.0 - far * flat * (1.0 - LAND_GRAIN_FLAT_KEEP)", "1.0 - far * (1.0 - LAND_GRAIN_FLAT_KEEP)"),
    ("nan_guard", "    if !slope.is_finite() {\n        return 1.0;\n    }\n    let far", "    if false {\n        return 1.0;\n    }\n    let far"),
    ("far_zero_exit", "    if far <= 0.0 {\n        return 1.0;\n    }", "    if far < 0.0 {\n        return 1.0;\n    }"),
    ("lattice_no_keep", "(f2, smoothstep(px, 2.0 * px, 1.0 / f2) * keep)", "(f2, smoothstep(px, 2.0 * px, 1.0 / f2))"),
    ("uncapped_keep", "    if (1.0 / f) * gs.km_per_cell <= LAND_GRAIN_MAX_KM {\n        return (f, 1.0);\n    }\n    let f2 = gs.km_per_cell / LAND_GRAIN_MAX_KM;\n    let px = gs.cells_per_px * LAND_GRAIN_NYQ_PX;\n    (f2, smoothstep(px, 2.0 * px, 1.0 / f2) * keep)", "    if (1.0 / f) * gs.km_per_cell <= LAND_GRAIN_MAX_KM {\n        return (f, keep);\n    }\n    let f2 = gs.km_per_cell / LAND_GRAIN_MAX_KM;\n    let px = gs.cells_per_px * LAND_GRAIN_NYQ_PX;\n    (f2, smoothstep(px, 2.0 * px, 1.0 / f2) * keep)"),
    ("keep_none", "let keep = grain.filter(|gs| gs.relief_fade).map_or(1.0, |gs| grain_relief_keep(gs, slope));", "let keep = 1.0;"),
    ("raster_fades", "grain.filter(|gs| gs.relief_fade).map_or(1.0, |gs| grain_relief_keep(gs, slope))", "grain.map_or(1.0, |gs| grain_relief_keep(gs, slope))"),
    ("raster_site_true", "ctx.grain_scale(1.0, false));", "ctx.grain_scale(1.0, true));"),
    ("tile_site_false", "let grain_k = ctx.grain_scale(cx.max(cy), true);", "let grain_k = ctx.grain_scale(cx.max(cy), false);"),
    ("export_site_false", "ctx.grain_scale(step.0.max(step.1), true),", "ctx.grain_scale(step.0.max(step.1), false),"),
    ("land_color_inf", "land_grain_at(appearance, x, y, gw, grain, slope);", "land_grain_at(appearance, x, y, gw, grain, f64::INFINITY);"),
    ("wrapper_flat", "land_grain_at(a, x, y, gw, grain, f64::INFINITY)\n}", "land_grain_at(a, x, y, gw, grain, 0.0)\n}"),
    ("lattice_wrapper", "grain_lattice_relief(f, gs, 1.0)\n}", "grain_lattice_relief(f, gs, 0.5)\n}"),
]
only = sys.argv[1:]
h = lambda p: hashlib.sha256(open(p, "rb").read()).hexdigest()
pre = h(REN)
env = dict(os.environ, CARGO_TARGET_DIR="C:/cw/fgleft_target2")
res = []
for name, old, new in M:
    if only and name not in only:
        continue
    src = open(REN, "rb").read()
    o, n = old.encode(), new.encode()
    if src.count(o) != 1:
        o, n = old.replace("\n", "\r\n").encode(), new.replace("\n", "\r\n").encode()
    assert src.count(o) == 1, f"{name}: literal occurs {src.count(o)} times"
    try:
        open(REN, "wb").write(src.replace(o, n))
        r = subprocess.run(T, cwd=R, env=env, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if "error[" in out or "error: could not compile" in out:
            v = "BUILD_ERROR"
        elif r.returncode != 0:
            v = "KILLED"
        else:
            v = "SURVIVED"
    finally:
        open(REN, "wb").write(src)
    res.append((name, v))
    print(name, v, flush=True)
assert h(REN) == pre, "render.rs was not restored"
print("RESTORED OK;", sum(v == "KILLED" for _, v in res), "of", len(res), "killed")
