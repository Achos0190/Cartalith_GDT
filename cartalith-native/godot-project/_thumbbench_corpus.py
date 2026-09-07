"""Build a realistic gallery of saves + an independent golden thumbnail.

Independent of the GDScript: the ramp is re-derived here from
cartalith-terrain/src/tile_render.rs's SEA/LAND constants, and the archive is
the minimal conforming writer SAVEFILE_COMPAT.md 6.1 describes.
"""
import io, json, os, shutil, struct, sys, zipfile
import numpy as np

ROOT = r'C:\Users\Vincent\AppData\Roaming\Godot\app_userdata\Cartalith Terrain Generator\Worlds'
OUT = os.path.dirname(os.path.abspath(__file__))   # goldens sit beside the probe

SEA = np.array([[10, 28, 46], [26, 86, 140], [70, 140, 196]], dtype=np.float32)
LAND_AT = np.array([0.0, 0.18, 0.38, 0.58, 0.78, 1.0], dtype=np.float32)
LAND = np.array([[47, 122, 68], [111, 154, 58], [201, 178, 74],
                 [150, 112, 72], [140, 140, 140], [248, 248, 250]], dtype=np.float32)
TW, TH = 96, 72


def f32(x):
    return np.float32(x)


def hypso(v, sea):
    """float32 arithmetic, mirroring GDScript's Color (32-bit components)."""
    v, sea = f32(v), f32(sea)
    if v < sea:
        d = f32(0.0) if sea <= 0 else f32((sea - v) / sea)
        if d < f32(0.5):
            a, b, t = SEA[2], SEA[1], f32(d / f32(0.5))
        else:
            a, b, t = SEA[1], SEA[0], f32((d - f32(0.5)) / f32(0.5))
        # Color.lerp in normalised space, as the GDScript does.
        an, bn = (a / f32(255.0)).astype(np.float32), (b / f32(255.0)).astype(np.float32)
        return np.clip(an + (bn - an) * t, 0.0, 1.0).astype(np.float32)
    r = f32(0.0) if (f32(1.0) - sea) <= 0 else f32((v - sea) / (f32(1.0) - sea))
    for i in range(len(LAND_AT) - 1):
        if r <= LAND_AT[i + 1]:
            span = f32(LAND_AT[i + 1] - LAND_AT[i])
            t = f32((r - LAND_AT[i]) / (f32(1.0) if span == 0 else span))
            an = (LAND[i] / f32(255.0)).astype(np.float32)
            bn = (LAND[i + 1] / f32(255.0)).astype(np.float32)
            return np.clip(an + (bn - an) * t, 0.0, 1.0).astype(np.float32)
    return (LAND[-1] / f32(255.0)).astype(np.float32)


def golden(hm, gw, gh, sea):
    """The expected 96x72 RGB8, nearest-sampled, then Godot's truncating store."""
    px = np.zeros((TH, TW, 3), dtype=np.uint8)
    for ty in range(TH):
        sy = min(int(float(ty) * gh / TH), gh - 1)
        for tx in range(TW):
            sx = min(int(float(tx) * gw / TW), gw - 1)
            c = hypso(hm[sy * gw + sx], sea)
            px[ty, tx] = np.clip((c * np.float32(255.0)).astype(np.float64), 0, 255).astype(np.uint8)
    return px


def field(gw, gh, seed):
    """A smooth, continent-shaped [0,1] field: compresses like a real raster."""
    rng = np.random.default_rng(seed)
    yy, xx = np.mgrid[0:gh, 0:gw]
    u, v = xx / gw, yy / gh
    f = np.zeros((gh, gw), dtype=np.float64)
    for k, amp in ((1, 1.0), (2, 0.5), (4, 0.25), (8, 0.12), (16, 0.06)):
        pu, pv = rng.random() * 6.283, rng.random() * 6.283
        f += amp * np.sin(2 * np.pi * k * u + pu) * np.cos(2 * np.pi * k * v + pv)
    f -= 1.4 * ((u - 0.5) ** 2 + (v - 0.5) ** 2)          # a central landmass
    f = (f - f.min()) / (f.max() - f.min())
    return f.astype(np.float32).ravel()


def write_tree(path, gw, gh, sea, seed, hm=None):
    hm = field(gw, gh, seed) if hm is None else hm
    manifest = {"format": "cartalith-project", "format_version": 1,
                "generator": "thumbbench", "world": {
                    "grid_width": gw, "grid_height": gh, "wrap_x": False,
                    "map_width_km": 900.0, "sea_level": sea, "seed": seed,
                    "origin": "gen"}}
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr("project.json", json.dumps(manifest))
        z.writestr("params.json", json.dumps({"GW": gw, "GH": gh,
                                              "state": {"seaLevel": sea, "tect": {"seed": seed}}}))
        z.writestr("rasters/heightmap.f32", hm.tobytes())
    return hm


def main():
    os.makedirs(ROOT, exist_ok=True)
    for f in os.listdir(ROOT):
        if f.startswith("_thumbbench"):
            os.remove(os.path.join(ROOT, f))

    made = []
    # Six big worlds -- the case the brief costs at ~10 MB of raster per tile.
    for i in range(6):
        p = os.path.join(ROOT, "_thumbbench_big%d.zip" % i)
        write_tree(p, 2048, 1311, 0.42, 40000 + i)
        made.append(p)
    # Two mid worlds.
    for i in range(2):
        p = os.path.join(ROOT, "_thumbbench_mid%d.zip" % i)
        write_tree(p, 512, 512, 0.55, 700 + i)
        made.append(p)

    # The golden world: written last so its heightmap is in hand.
    gp = os.path.join(ROOT, "_thumbbench_golden.zip")
    hm = write_tree(gp, 384, 256, 0.42, 24601)
    made.append(gp)
    g = golden(hm, 384, 256, 0.42)
    with open(os.path.join(OUT, "_thumbbench_golden.raw"), "wb") as fh:
        fh.write(g.tobytes())

    # Degenerate but conforming sea levels: 7 allows both hypso guards.
    write_tree(os.path.join(ROOT, "_thumbbench_sea0.zip"), 128, 96, 0.0, 11)
    write_tree(os.path.join(ROOT, "_thumbbench_sea1.zip"), 128, 96, 1.0, 12)
    made += [os.path.join(ROOT, "_thumbbench_sea0.zip"), os.path.join(ROOT, "_thumbbench_sea1.zip")]

    # A real engine-written save, copied in unchanged.
    real = os.path.join(os.path.dirname(ROOT), "__diagreview_project__.zip")
    shutil.copy(real, os.path.join(ROOT, "_thumbbench_real.zip"))
    made.append(os.path.join(ROOT, "_thumbbench_real.zip"))

    # A flat (section 15) archive, likewise real.
    flat = os.path.join(os.path.dirname(ROOT), "_dbg.zip")
    shutil.copy(flat, os.path.join(ROOT, "_thumbbench_flat.zip"))
    made.append(os.path.join(ROOT, "_thumbbench_flat.zip"))

    # Damaged: the manifest says 256x256, the raster is short (section 8.1 refuse).
    dp = os.path.join(ROOT, "_thumbbench_damaged.zip")
    with zipfile.ZipFile(dp, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr("project.json", json.dumps({"format": "cartalith-project", "format_version": 1,
                                               "world": {"grid_width": 256, "grid_height": 256,
                                                         "sea_level": 0.42, "seed": 1}}))
        z.writestr("rasters/heightmap.f32", b"\0" * 1024)
    made.append(dp)

    # Foreign: a .zip with neither manifest.
    fp = os.path.join(ROOT, "_thumbbench_foreign.zip")
    with zipfile.ZipFile(fp, 'w') as z:
        z.writestr("readme.txt", "not a world")
    made.append(fp)

    for p in sorted(made):
        print("%9d  %s" % (os.path.getsize(p), os.path.basename(p)))
    print("total tiles:", len(made))


main()
