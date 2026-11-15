"""Ruling FG mutation run: exact literal replace (must occur exactly once), build errors separate from
survivors, every file restored in a `finally`, and hashes checked against the pre-run value."""
import hashlib, subprocess, sys, os
R = "C:/cw/fgfix/cartalith-native"
AMP = R + "/crates/cartalith-terrain/src/amplify.rs"
REN = R + "/crates/cartalith-godot/src/render.rs"
LOD = R + "/crates/cartalith-godot/src/lod_bridge.rs"
T_AMP = ["cargo", "test", "-p", "cartalith-terrain", "--lib", "amplify"]
T_LOD = ["cargo", "test", "-p", "cartalith-godot", "--lib", "lod_bridge"]
T_MOT = ["cargo", "test", "-p", "cartalith-godot", "--test", "canopy_and_mottle"]
M = [
    ("gain", AMP, "pub const ZOOM_DETAIL_KM_RELIEF_GAIN: f64 = 11.0;", "pub const ZOOM_DETAIL_KM_RELIEF_GAIN: f64 = 10.0;", T_AMP),
    ("floor", AMP, "pub const ZOOM_DETAIL_KM_RELIEF_FLOOR: f64 = 0.00153;", "pub const ZOOM_DETAIL_KM_RELIEF_FLOOR: f64 = 0.0016;", T_AMP),
    ("max_oct", AMP, "pub const ZOOM_DETAIL_KM_MAX_OCTAVES: u32 = 14;", "pub const ZOOM_DETAIL_KM_MAX_OCTAVES: u32 = 8;", T_AMP),
    ("cap_frac", AMP, "pub const ZOOM_DETAIL_KM_CAP_FRAC: f64 = 0.1;", "pub const ZOOM_DETAIL_KM_CAP_FRAC: f64 = 0.2;", T_AMP),
    ("no_curvature", AMP, "    js_hypot(js_hypot(gx, gy), lap)\n", "    js_hypot(gx, gy) + 0.0 * lap\n", T_AMP),
    ("no_floor", AMP, "rho * js_max(relief, km.relief_floor);", "rho * relief;", T_AMP),
    ("no_softcap", AMP, "if cap.is_finite() && cap > 0.0 { cap * (x / cap).tanh() } else { x }", "x", T_AMP),
    ("no_shape_weight", AMP, "sum += n * (lam / l).powf(h_exp) * w;", "sum += n * w;", T_AMP),
    ("no_rough", AMP, "let rho = rough.map_or(1.0, |f| f(cx, cy));", "let rho = rough.map_or(1.0, |_f| 1.0);", T_AMP),
    ("no_shore", AMP, "soft_cap(sum * amp, km.cap) * shore", "soft_cap(sum * amp, km.cap)", T_AMP),
    ("lap_sign", AMP, "(l + r + u + d) * 0.25 - c);", "(l + r + u + d) * 0.5 - c);", T_AMP),
    ("depth_no_margin", LOD, "let z = f64::from(z_base()) + margin + (l / DETAIL_FLOOR_KM).log2();", "let z = f64::from(z_base()) + (l / DETAIL_FLOOR_KM).log2();", T_LOD),
    ("ceiling17", LOD, "pub const MAX_LEVEL: i32 = 16;", "pub const MAX_LEVEL: i32 = 17;", T_LOD),
    ("floor_km", LOD, "pub const DETAIL_FLOOR_KM: f64 = 0.015;", "pub const DETAIL_FLOOR_KM: f64 = 0.02;", T_LOD),
    ("no_base_floor", LOD, "(z.ceil() as i32).clamp(BASE_MAX_LEVEL, MAX_LEVEL)", "(z.ceil() as i32).clamp(0, MAX_LEVEL)", T_LOD),
    ("tier_flat", LOD, "    tier_max + (map_max - BASE_MAX_LEVEL)\n", "    tier_max + 0 * (map_max - BASE_MAX_LEVEL)\n", T_LOD),
    ("to_ignores_cap", LOD, "Some(max_level.clamp(0, MAX_LEVEL)))", "Some(BASE_MAX_LEVEL))", T_LOD),
    ("grain_cap25", REN, "pub const LAND_GRAIN_MAX_KM: f64 = 20.0;", "pub const LAND_GRAIN_MAX_KM: f64 = 25.0;", T_MOT),
    ("grain_no_fade", REN, "    (f2, smoothstep(px, 2.0 * px, 1.0 / f2))\n", "    (f2, 1.0 + 0.0 * smoothstep(px, 2.0 * px, 1.0 / f2))\n", T_MOT),
    ("grain_no_cap", REN, "if (1.0 / f) * gs.km_per_cell <= LAND_GRAIN_MAX_KM {", "if true || (1.0 / f) * gs.km_per_cell <= LAND_GRAIN_MAX_KM {", T_MOT),
    ("grain_fade_uncapped", REN, "        return (f, 1.0);\n    }\n    let f2 = gs.km_per_cell", "        return (f, smoothstep(gs.cells_per_px * LAND_GRAIN_NYQ_PX, 2.0 * gs.cells_per_px * LAND_GRAIN_NYQ_PX, 1.0 / f));\n    }\n    let f2 = gs.km_per_cell", T_MOT),
    ("warp_uncapped", REN, "Some(gs) if LAND_WARP_CELLS * gs.km_per_cell > LAND_GRAIN_MAX_KM => LAND_GRAIN_MAX_KM / gs.km_per_cell,", "Some(_gs) if false => 1.0,", T_MOT),
    ("fade_always_formula", REN, "    if vis >= 1.0 { n } else { 0.5 + (n - 0.5) * vis }\n", "    0.5 + (n - 0.5) * vis\n", T_MOT),
    ("rho_cold", REN, "pub const DETAIL_RHO_COLD: f64 = 0.8;", "pub const DETAIL_RHO_COLD: f64 = 1.0;", T_MOT),
    ("rho_sand", REN, "pub const DETAIL_RHO_SAND: f64 = 0.25;", "pub const DETAIL_RHO_SAND: f64 = 0.3;", T_MOT),
    ("rho_forest_ramp", REN, "let forest = smoothstep(0.25, 0.55, m);", "let forest = smoothstep(0.25, 0.65, m);", T_MOT),
]
only = sys.argv[1:]
h = lambda p: hashlib.sha256(open(p, "rb").read()).hexdigest()
files = {AMP, REN, LOD}
pre = {f: h(f) for f in files}
env = dict(os.environ, CARGO_TARGET_DIR="C:/cw/fgfix_target")
res = []
for name, f, old, new, cmd in M:
    if only and name not in only:
        continue
    src = open(f, "rb").read()
    o, n = old.encode(), new.encode()
    # Files may be CRLF on disk: match either form.
    if src.count(o) != 1:
        o, n = old.replace("\n", "\r\n").encode(), new.replace("\n", "\r\n").encode()
    assert src.count(o) == 1, f"{name}: literal occurs {src.count(o)} times"
    try:
        open(f, "wb").write(src.replace(o, n))
        r = subprocess.run(cmd, cwd=R, env=env, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if "error[" in out or "error: could not compile" in out:
            v = "BUILD_ERROR"
        elif r.returncode != 0:
            v = "KILLED"
        else:
            v = "SURVIVED"
    finally:
        open(f, "wb").write(src)
    res.append((name, v))
    print(name, v, flush=True)
post = {f: h(f) for f in files}
assert post == pre, "a file was not restored"
print("RESTORED OK;", sum(v == "KILLED" for _, v in res), "of", len(res), "killed")
