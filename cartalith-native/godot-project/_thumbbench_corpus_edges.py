"""The edge-case half of the corpus. Run after _thumbbench_corpus.py.

Every archive here exists to kill a specific mutant: absent and
out-of-range sea_level, absent and zero grid, the smallest grid section 7
allows, and a world exactly the tile's own 96x72 -- the only grid where
the sample index reaches gw-1/gh-1, which is what makes the axis clamps
testable at all.
"""
import json, os, zipfile
import numpy as np
ROOT = r'C:\Users\Vincent\AppData\Roaming\Godot\app_userdata\Cartalith Terrain Generator\Worlds'
hm = np.linspace(0, 1, 128 * 96, dtype=np.float32)

def w(name, world):
    p = os.path.join(ROOT, name)
    with zipfile.ZipFile(p, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr("project.json", json.dumps(
            {"format": "cartalith-project", "format_version": 1, "world": world}))
        z.writestr("rasters/heightmap.f32", hm.tobytes())
    print(name, os.path.getsize(p))

# sea_level absent -- a MUST that section 7 says a reader refuses.
w("_thumbbench_nosea.zip", {"grid_width": 128, "grid_height": 96, "seed": 1})
# sea_level out of [0,1].
w("_thumbbench_badsea.zip", {"grid_width": 128, "grid_height": 96, "sea_level": 1.7, "seed": 2})
import json, os, zipfile
import numpy as np
ROOT = r'C:\Users\Vincent\AppData\Roaming\Godot\app_userdata\Cartalith Terrain Generator\Worlds'

def w(name, world, gw, gh, raster=None):
    p = os.path.join(ROOT, name)
    if raster is None:
        raster = np.linspace(0, 1, max(gw * gh, 1), dtype=np.float32).tobytes()
    with zipfile.ZipFile(p, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr("project.json", json.dumps(
            {"format": "cartalith-project", "format_version": 1, "world": world}))
        z.writestr("rasters/heightmap.f32", raster)
    print(name, os.path.getsize(p))

# Exactly the tile's own size: the one grid where the sample index reaches
# gw-1 / gh-1, so the two axis clamps are load-bearing rather than dead.
w("_thumbbench_exact.zip", {"grid_width": 96, "grid_height": 72,
                            "sea_level": 0.42, "seed": 3}, 96, 72)
# The smallest grid section 7 allows (integer >= 1). Must render, not refuse.
w("_thumbbench_one.zip", {"grid_width": 1, "grid_height": 1,
                          "sea_level": 0.42, "seed": 4}, 1, 1)
# grid_width absent -- a MUST; section 7: "Refuse. ... No raster in the archive
# can be validated without them."
w("_thumbbench_nogrid.zip", {"grid_height": 96, "sea_level": 0.42, "seed": 5}, 0, 96, b"")
# grid_width 0 -- section 7: "Zero or negative is refused too."
w("_thumbbench_zerogrid.zip", {"grid_width": 0, "grid_height": 96,
                               "sea_level": 0.42, "seed": 6}, 0, 96, b"")

# The 96x72 world's golden, computed by the same outside ramp the main corpus
# uses, so the probe compares against something that is not the GDScript.
_here = os.path.dirname(os.path.abspath(__file__))
exec(open(os.path.join(_here, "_thumbbench_corpus.py"), encoding="utf-8").read().split("def main()")[0])
_z = zipfile.ZipFile(os.path.join(ROOT, "_thumbbench_exact.zip"))
_hm = np.frombuffer(_z.read("rasters/heightmap.f32"), dtype="<f4")
open(os.path.join(_here, "_thumbbench_exact.raw"), "wb").write(golden(_hm, 96, 72, 0.42).tobytes())
print("_thumbbench_exact.raw written")
