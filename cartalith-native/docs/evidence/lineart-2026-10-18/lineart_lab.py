"""Line-art -> sprite method lab (throwaway, 2026-10-18). Python 3.11 + numpy/scipy/Pillow.

Synthetic, deterministic samples drawn as vector strokes in normalised coordinates, so the
ground truth can be rendered at ANY output size. Each method starts from the 1024^2 "scan"
an author would import, and is judged at the on-map sprite heights 40/160/640/2560 px
(x1/x4/x16/x64 of a 2048-wide grid's s=1 icon: dh = (2048/110)*2.2 = 40.96 px).
"""
import io, json, math, sys, time
import numpy as np
from PIL import Image, ImageDraw, ImageFilter
from scipy import ndimage as ndi

OUT = sys.argv[1]
SRC = 1024
SCALES = [40, 160, 640, 2560]
BG = np.array([118, 140, 92], float)       # a mid map green to composite over
INK = np.array([34, 28, 22], float)        # the palette ink colour methods recolour to

# ------------------------------------------------------------------ samples (vector truth)
def wob(rng, pts, amp):
    pts = np.array(pts, float)
    # densify then perturb with a smooth random walk: "hand-drawn"
    dense = []
    for a, b in zip(pts[:-1], pts[1:]):
        n = max(2, int(np.hypot(*(b - a)) / 0.01))
        for t in np.linspace(0, 1, n, endpoint=False):
            dense.append(a + (b - a) * t)
    dense.append(pts[-1])
    d = np.array(dense)
    walk = np.cumsum(rng.normal(0, amp, d.shape), axis=0)
    walk -= np.linspace(walk[0], walk[-1], len(walk))     # pin the ends
    return d + walk * 0.35

def pine(rng):
    S = []
    w = 0.0060
    S.append((wob(rng, [(0.5, 0.95), (0.5, 0.12)], 0.0015), w))
    for i in range(9):
        y = 0.22 + i * 0.075
        half = 0.06 + i * 0.033
        S.append((wob(rng, [(0.5, y - 0.03), (0.5 - half, y + 0.05)], 0.0015), w))
        S.append((wob(rng, [(0.5, y - 0.03), (0.5 + half, y + 0.05)], 0.0015), w))
    S.append((wob(rng, [(0.42, 0.95), (0.58, 0.95)], 0.001), w))
    return S, []

def castle(rng):
    S, F = [], []
    w = 0.0080
    S.append((wob(rng, [(0.2, 0.92), (0.2, 0.45), (0.8, 0.45), (0.8, 0.92), (0.2, 0.92)], 0.001), w))
    for x0 in (0.12, 0.70):   # towers
        S.append((wob(rng, [(x0, 0.92), (x0, 0.25), (x0 + 0.18, 0.25), (x0 + 0.18, 0.92)], 0.001), w))
        for k in range(3):   # crenellations
            xa = x0 + k * 0.065
            S.append((wob(rng, [(xa, 0.25), (xa, 0.19), (xa + 0.035, 0.19), (xa + 0.035, 0.25)], 0.0006), w))
        for k in range(2):   # windows
            S.append((wob(rng, [(x0 + 0.07, 0.36 + k * 0.17), (x0 + 0.07, 0.42 + k * 0.17),
                                (x0 + 0.11, 0.42 + k * 0.17), (x0 + 0.11, 0.36 + k * 0.17), (x0 + 0.07, 0.36 + k * 0.17)], 0.0004), w * 0.8))
    F.append([(0.44, 0.92), (0.44, 0.74), (0.5, 0.68), (0.56, 0.74), (0.56, 0.92)])   # a solid door fill
    for k in range(5):   # wall stones (thin)
        S.append((wob(rng, [(0.30 + 0.08 * k, 0.55), (0.34 + 0.08 * k, 0.55)], 0.0005), w * 0.6))
    return S, F

def mountain(rng):
    S = []
    w = 0.0080
    ridge = [(0.06, 0.9), (0.32, 0.42), (0.42, 0.52), (0.55, 0.18), (0.78, 0.6), (0.94, 0.9)]
    S.append((wob(rng, ridge, 0.0012), w))
    # hatching on the shadow flank: thin parallel strokes, spacing 0.02 (20 px at 1024)
    for i in range(14):
        t = i / 14
        x = 0.56 + t * 0.34
        ytop = 0.19 + t * 0.66 + 0.04
        S.append((wob(rng, [(x, ytop), (x - 0.05, min(0.9, ytop + 0.16))], 0.0006), 0.0040))
    S.append((wob(rng, [(0.55, 0.18), (0.52, 0.5), (0.58, 0.9)], 0.0012), w * 0.8))
    return S, []

SAMPLES = {"pine": pine, "castle": castle, "mountain": mountain, "scan": castle}

def render_truth(strokes, fills, H, floor_px=0.0):
    ss = int(min(16, max(2, round(4096 / H))))
    N = H * ss
    im = Image.new("L", (N, N), 0)
    dr = ImageDraw.Draw(im)
    for pts, w in strokes:
        wp = max(w * H, floor_px) * ss
        p = [(x * N, y * N) for x, y in pts]
        dr.line(p, fill=255, width=max(1, int(round(wp))), joint="curve")
        r = wp / 2
        for x, y in (p[0], p[-1]):
            dr.ellipse([x - r, y - r, x + r, y + r], fill=255)
    for poly in fills:
        dr.polygon([(x * N, y * N) for x, y in poly], fill=255)
    a = np.asarray(im, float) / 255.0
    return a.reshape(H, ss, H, ss).mean(axis=(1, 3))

def make_scan(name, strokes, fills, rng):
    cov = render_truth(strokes, fills, SRC)
    if name == "scan":
        # ink starvation: strokes break up in places (the "broken line" input)
        starve = ndi.gaussian_filter(rng.random((SRC, SRC)), 6)
        starve = (starve - starve.min()) / (starve.max() - starve.min())
        cov = cov * np.clip(0.35 + 1.1 * starve, 0, 1)
        cov = ndi.gaussian_filter(cov, 0.9)                     # scanner blur
        paper = np.array([238, 228, 205], float)
        yy, xx = np.mgrid[0:SRC, 0:SRC] / SRC
        light = 1.0 - 0.28 * ((xx - 0.2) ** 2 + (yy - 0.3) ** 2)   # uneven illumination
        img = (paper[None, None] * light[..., None]) * (1 - cov[..., None]) + INK[None, None] * 0.6 * cov[..., None]
        img += rng.normal(0, 6, img.shape)                           # paper grain
        for _ in range(60):                                          # dust
            cy, cx = rng.integers(0, SRC, 2)
            r = rng.integers(1, 3)
            img[max(0, cy - r):cy + r, max(0, cx - r):cx + r] *= 0.45
        q = 35
    else:
        img = 255.0 * (1 - cov[..., None]) + np.array([20, 20, 20.0]) * cov[..., None]
        img = np.repeat(img, 1, axis=2) if img.shape[2] == 3 else img
        q = 80 if name == "pine" else None   # pine as an AI/JPEG export; others clean PNG
    img = np.clip(img, 0, 255).astype(np.uint8)
    if img.shape[2] == 1:
        img = np.repeat(img, 3, axis=2)
    pil = Image.fromarray(img, "RGB")
    if q:
        b = io.BytesIO(); pil.save(b, "JPEG", quality=q); pil = Image.open(io.BytesIO(b.getvalue())).convert("RGB")
    return np.asarray(pil, float)

def lum(rgb):
    return rgb[..., 0] * 0.2126 + rgb[..., 1] * 0.7152 + rgb[..., 2] * 0.0722

# ------------------------------------------------------------------ resampling helpers
def resize_f(a, W, H, flt):
    return np.asarray(Image.fromarray(a.astype(np.float32), "F").resize((W, H), flt), float)

def bilinear_point(img, H):
    """pack.rs::sample_bilinear at destination pixel centres (the port's blit_sprite)."""
    h, w = img.shape[:2]
    u = (np.arange(H) + 0.5) / H
    x = np.maximum(u * w - 0.5, 0); y = np.maximum(u * h - 0.5, 0)
    x0 = np.floor(x).astype(int); y0 = np.floor(y).astype(int)
    x1 = np.minimum(x0 + 1, w - 1); y1 = np.minimum(y0 + 1, h - 1)
    fx = (x - x0)[None, :, None]; fy = (y - y0)[:, None, None]
    p = lambda yy, xx: img[yy][:, xx]
    top = p(y0, x0) * (1 - fx) + p(y0, x1) * fx
    bot = p(y1, x0) * (1 - fx) + p(y1, x1) * fx
    return top * (1 - fy) + bot * fy

def nearest_mag(img, z):
    return np.repeat(np.repeat(img, z, axis=0), z, axis=1)

# ------------------------------------------------------------------ methods
def legacy(scan, H):
    """Reference/port today: chroma key (corner colour, tol 40) -> render_item 256^2 CatmullRom
    (straight alpha, channels independent) -> blit at 40 px (bilinear point sample) -> the base
    raster is shown NEAREST (viewport_host.gd::_raster), so zoom z magnifies by z."""
    rgba = np.dstack([scan, np.full(scan.shape[:2], 255.0)])
    key = scan[2, 2]
    d2 = ((scan - key) ** 2).sum(-1)
    rgba[..., 3][d2 <= 40 * 40] = 0
    bake = np.dstack([resize_f(rgba[..., c], 256, 256, Image.BICUBIC) for c in range(4)])
    bake = np.clip(bake, 0, 255)
    small = bilinear_point(bake, 40)
    big = nearest_mag(small, H // 40)
    return big[..., :3], big[..., 3] / 255.0

def normalise_paper(scan):
    L = lum(scan)
    # local paper estimate: the window must exceed the largest ink feature (a 25 px window
    # read the castle's 123 px solid door as "paper" and hollowed it); illumination is low
    # frequency, so ~15 % of the sprite is safe
    w = int(0.15 * L.shape[0]) | 1
    B = ndi.uniform_filter(ndi.maximum_filter(L, w), w)
    return np.clip(L / np.maximum(B, 1), 0, 1.2)

def c2a_alpha(scan):
    """Colour-to-alpha against the LOCAL paper (PhotoCraft color_to_alpha.rs formula, with
    the key replaced by a per-pixel paper estimate), then its transparency/opacity remap."""
    Ln = normalise_paper(scan)
    ink = np.percentile(Ln, 0.5)
    a = np.clip((1 - Ln) / max(1e-3, 1 - ink), 0, 1)
    t0, t1 = 0.10, 0.80
    return np.clip((a - t0) / (t1 - t0), 0, 1)

def coverage_raster(scan, H, store=SRC):
    a = c2a_alpha(scan)
    if store != SRC:
        a = resize_f(a, store, store, Image.BOX)
    flt = Image.BOX if H <= store else Image.BICUBIC
    out = np.clip(resize_f(a, H, H, flt), 0, 1)
    return np.broadcast_to(INK, (H, H, 3)), out

def sauvola(L, win=31, k=0.2, R=0.5):
    m = ndi.uniform_filter(L, win); s = np.sqrt(np.maximum(ndi.uniform_filter(L * L, win) - m * m, 0))
    return L < m * (1 + k * (s / R - 1))

def bradley(L, win=63, t=0.15):
    return L < ndi.uniform_filter(L, win) * (1 - t)

def otsu(L):
    h, e = np.histogram(L, 256, (0, 1.2)); c = (e[:-1] + e[1:]) / 2
    w0 = np.cumsum(h); w1 = w0[-1] - w0; m0 = np.cumsum(h * c) / np.maximum(w0, 1)
    m1 = (np.sum(h * c) - np.cumsum(h * c)) / np.maximum(w1, 1)
    return L < c[np.argmax(w0 * w1 * (m0 - m1) ** 2)]

def clean_mask(scan, close=1, speck=24, pin=12, method="otsu"):
    Ln = normalise_paper(scan)
    m = {"sauvola": sauvola, "bradley": bradley, "otsu": otsu}[method](Ln)
    m &= Ln < 0.9          # never call clean paper ink (Sauvola's flat-region guard)
    lab, n = ndi.label(m, np.ones((3, 3)))
    sizes = ndi.sum(m, lab, range(1, n + 1))
    m = np.isin(lab, 1 + np.flatnonzero(sizes >= speck))
    if close:
        m = ndi.binary_closing(m, np.ones((2 * close + 1,) * 2), border_value=0)
    hl, hn = ndi.label(~m)
    hs = ndi.sum(~m, hl, range(1, hn + 1))
    m |= np.isin(hl, 1 + np.flatnonzero(hs < pin))
    return m

def sdf_from_mask(m, R, spread=8.0):
    """Signed distance (positive inside) measured at 2x the mask with a smoothed contour,
    resampled to R^2 and quantised to u8 with +-spread texels of range."""
    up = ndi.zoom(ndi.gaussian_filter(m.astype(float), 0.8), 2, order=1) > 0.5
    d = (ndi.distance_transform_edt(up) - ndi.distance_transform_edt(~up)) / 2.0   # in mask px
    n = m.shape[0]
    c = (np.arange(R) + 0.5) * (2 * n) / R - 0.5
    yy, xx = np.meshgrid(c, c, indexing="ij")
    dR = ndi.map_coordinates(d, [yy, xx], order=1) * R / n                      # in texels
    return np.clip(np.round(128 + dR * 127 / spread), 0, 255).astype(np.uint8)

def render_sdf(q, H, delta_out=0.0, spread=8.0):
    R = q.shape[0]
    d = (q.astype(float) - 128) * spread / 127
    c = (np.arange(H) + 0.5) * R / H - 0.5
    yy, xx = np.meshgrid(c, c, indexing="ij")
    dout = ndi.map_coordinates(d, [yy, xx], order=1, mode="nearest") * H / R
    return np.clip(dout + delta_out + 0.5, 0, 1)

def mat_field(m, R, spread=8.0):
    """Medial-axis field: per texel, distance to the nearest centreline pixel and that
    centreline pixel's stroke radius (edt of the mask at the skeleton). Two u8 channels."""
    sk = zhang_suen(m)
    em = ndi.distance_transform_edt(m)
    d, inds = ndi.distance_transform_edt(~sk, return_indices=True)
    rad = np.maximum(em[inds[0], inds[1]] - 0.5, 0)
    n = m.shape[0]
    c = (np.arange(R) + 0.5) * n / R - 0.5
    yy, xx = np.meshgrid(c, c, indexing="ij")
    dR = ndi.map_coordinates(d, [yy, xx], order=1) * R / n
    rR = ndi.map_coordinates(rad, [yy, xx], order=1) * R / n
    qd = np.clip(np.round(dR * 255 / spread), 0, 255).astype(np.uint8)
    qr = np.clip(np.round(rR * 255 / spread), 0, 255).astype(np.uint8)
    return np.dstack([qd, qr])

def render_mat(q, H, rmin_out=0.5, spread=8.0):
    R = q.shape[0]
    c = (np.arange(H) + 0.5) * R / H - 0.5
    yy, xx = np.meshgrid(c, c, indexing="ij")
    k = H / R * spread / 255
    d = ndi.map_coordinates(q[..., 0].astype(float), [yy, xx], order=1, mode="nearest") * k
    r = ndi.map_coordinates(q[..., 1].astype(float), [yy, xx], order=1, mode="nearest") * k
    return np.clip(np.maximum(r, rmin_out) - d + 0.5, 0, 1)

def gap_field(m, R, spread_px=64.0):
    """Local width of the paper gap next to each texel: 2 x the background distance at the
    nearest background-ridge pixel. Lets the render-time floor stop before it closes a gap."""
    eb = ndi.distance_transform_edt(~m)
    ridge = (eb >= ndi.maximum_filter(eb, 3)) & ~m & (eb > 0)
    _, inds = ndi.distance_transform_edt(~ridge, return_indices=True)
    G = 2 * eb[inds[0], inds[1]]
    n = m.shape[0]
    c = (np.arange(R) + 0.5) * n / R - 0.5
    yy, xx = np.meshgrid(c, c, indexing="ij")
    GR = ndi.map_coordinates(G, [yy, xx], order=1) * R / n
    return np.clip(np.round(GR * 255 / spread_px), 0, 255).astype(np.uint8)

def render_sdf_gap(q, g, H, delta_out, spread=8.0, spread_px=64.0):
    R = q.shape[0]
    d = (q.astype(float) - 128) * spread / 127
    c = (np.arange(H) + 0.5) * R / H - 0.5
    yy, xx = np.meshgrid(c, c, indexing="ij")
    dout = ndi.map_coordinates(d, [yy, xx], order=1, mode="nearest") * H / R
    Rg = g.shape[0]
    cg = (np.arange(H) + 0.5) * Rg / H - 0.5
    yg, xg = np.meshgrid(cg, cg, indexing="ij")
    gout = ndi.map_coordinates(g.astype(float) * spread_px / 255, [yg, xg], order=1, mode="nearest") * H / Rg
    de = np.clip(np.minimum(delta_out, (gout - 1.0) / 2), 0, None)
    return np.clip(dout + de + 0.5, 0, 1)

def ridge_widths(m):
    """Stroke widths along the medial ridge: 2*edt-1 at local maxima of the distance map."""
    if m.sum() == 0:
        return np.array([0.0])
    e = ndi.distance_transform_edt(m)
    mx = ndi.maximum_filter(e, 3)
    r = (e >= mx) & m & (e > 0)
    return 2 * e[r] - 1

def floor_delta(m, H, wmin=1.0):
    w = ridge_widths(m) * H / m.shape[0]
    p = np.percentile(w, 2)
    return max(0.0, (wmin - p) / 2), float(np.median(w))

def zhang_suen(m):
    img = m.copy().astype(np.uint8)
    while True:
        changed = False
        for step in (0, 1):
            P = np.pad(img, 1)
            n = [P[:-2, 1:-1], P[:-2, 2:], P[1:-1, 2:], P[2:, 2:], P[2:, 1:-1], P[2:, :-2], P[1:-1, :-2], P[:-2, :-2]]
            B = sum(n)
            seq = n + [n[0]]
            A = sum(((seq[i] == 0) & (seq[i + 1] == 1)).astype(np.uint8) for i in range(8))
            if step == 0:
                c = (n[0] * n[2] * n[4] == 0) & (n[2] * n[4] * n[6] == 0)
            else:
                c = (n[0] * n[2] * n[6] == 0) & (n[0] * n[4] * n[6] == 0)
            rm = (img == 1) & (B >= 2) & (B <= 6) & (A == 1) & c
            if rm.any():
                img[rm] = 0; changed = True
        if not changed:
            return img.astype(bool)

# ------------------------------------------------------------------ metrics
def metrics(O, T, rgb=None):
    Ob, Tb = O > 0.5, T > 0.5
    st = np.ones((3, 3))
    comp = lambda b: ndi.label(b, st)[1]
    holes = lambda b: ndi.label(~b)[1] - 1
    inter = (Ob & Tb).sum(); uni = (Ob | Tb).sum()
    wO = ridge_widths(Ob)
    per = (Ob & ~ndi.binary_erosion(Ob)).sum()
    mid = ((O > 0.1) & (O < 0.9)).sum()
    return dict(comp=comp(Ob), comp_T=comp(Tb), holes=holes(Ob), holes_T=holes(Tb),
                iou=round(inter / max(uni, 1), 3),
                solidity=round(float(O[Tb].mean()) if Tb.any() else 0, 3),
                w_p5=round(float(np.percentile(wO, 5)), 2), w_med=round(float(np.median(wO)), 2),
                edge_px=(round(mid / per, 2) if per > 0 else None))

def effective(rgb, a):
    """Composite over the map green; effective ink = darkening toward INK; halo = pixels the
    sprite LIGHTENS by > 20 luma (paper fringe kept opaque)."""
    out = rgb * a[..., None] + BG * (1 - a[..., None])
    lb, li, lo = lum(BG[None, None]), lum(INK[None, None]), lum(out)
    e = np.clip((lb - lo) / (lb - li), 0, 1)
    halo = float(((lo - lb) > 20).mean())
    return out, e, halo

FOCUS = {"pine": (0.42, 0.30), "castle": (0.12, 0.17), "mountain": (0.60, 0.30), "scan": (0.12, 0.17)}
def crop(img, name, H, n=200):
    img = np.clip(img, 0, 255).astype(np.uint8)
    if H <= n:
        return img
    fx, fy = FOCUS[name]
    x0 = min(H - n, int(fx * H)); y0 = min(H - n, int(fy * H))
    return img[y0:y0 + n, x0:x0 + n]

# ------------------------------------------------------------------ run
def main():
    res = {"binarise": {}, "methods": {}, "sdf_res": {}, "sizes": {}, "times_ms": {}}
    vis = {}
    for si, (name, fn) in enumerate(SAMPLES.items()):
        rng = np.random.default_rng(1000 + si)
        S, F = fn(rng)
        scan = make_scan(name, S, F, np.random.default_rng(77 + si))
        Image.fromarray(scan.astype(np.uint8)).save(f"{OUT}/src_{name}.png")
        T1024 = render_truth(S, F, SRC)
        # (b)/(c) binarisation at source resolution
        tb = T1024 > 0.5
        rb = {}
        L = lum(scan) / 255
        key = scan[2, 2]
        rb["chroma_tol40"] = ((scan - key) ** 2).sum(-1) > 1600
        rb["otsu_raw"] = otsu(L)
        Ln = normalise_paper(scan)
        rb["otsu_flat"] = otsu(Ln)
        rb["bradley_flat"] = bradley(Ln) & (Ln < 0.9)
        rb["sauvola_flat"] = sauvola(Ln) & (Ln < 0.9)
        rb["sauvola_clean"] = clean_mask(scan, close=0, method="sauvola")
        rb["otsu_flat_clean"] = clean_mask(scan, close=0)
        rb["otsu_flat_clean_close1"] = clean_mask(scan, close=1)
        rb["otsu_flat_clean_close2"] = clean_mask(scan, close=2)
        bres = {}
        for k, m in rb.items():
            bres[k] = dict(iou=round((m & tb).sum() / max((m | tb).sum(), 1), 3), comp=ndi.label(m, np.ones((3, 3)))[1],
                           holes=ndi.label(~m, np.ones((3, 3)))[1] - 1)
        bres["truth"] = dict(iou=1.0, comp=ndi.label(tb, np.ones((3, 3)))[1], holes=ndi.label(~tb, np.ones((3, 3)))[1] - 1)
        res["binarise"][name] = bres
        if name == "scan":
            vis["binarise"] = rb
        # cleaned mask (close only for the broken scan; clean art must not be closed: hatching)
        t0 = time.perf_counter()
        m = clean_mask(scan, close=1 if name == "scan" else 0)
        t_clean = (time.perf_counter() - t0) * 1000
        t0 = time.perf_counter(); q128 = sdf_from_mask(m, 128); t_sdf = (time.perf_counter() - t0) * 1000
        q256 = sdf_from_mask(m, 256); q512 = sdf_from_mask(m, 512)
        t0 = time.perf_counter(); mat128 = mat_field(m, 128); t_sk = (time.perf_counter() - t0) * 1000
        mat256 = mat_field(m, 256)
        g128 = gap_field(m, 128)
        res["times_ms"][name] = dict(clean=round(t_clean), sdf128=round(t_sdf), mat128=round(t_sk))
        # file sizes of what each method stores
        def png_bytes(arr, mode):
            b = io.BytesIO(); Image.fromarray(arr, mode).save(b, "PNG", optimize=True); return len(b.getvalue())
        rgba = np.dstack([scan, np.where(((scan - key) ** 2).sum(-1) <= 1600, 0, 255)]).astype(np.uint8)
        bake = Image.fromarray(rgba, "RGBA").resize((256, 256), Image.BICUBIC)
        b = io.BytesIO(); bake.save(b, "PNG", optimize=True)
        cov = (c2a_alpha(scan) * 255).round().astype(np.uint8)
        res["sizes"][name] = dict(legacy_rgba256=len(b.getvalue()), coverage_L1024=png_bytes(cov, "L"),
                                  coverage_L512=png_bytes(np.asarray(Image.fromarray(cov).resize((512, 512), Image.BOX)), "L"),
                                  sdf_L64=png_bytes(sdf_from_mask(m, 64), "L"), sdf_L128=png_bytes(q128, "L"),
                                  sdf_L256=png_bytes(q256, "L"), sdf_L512=png_bytes(q512, "L"),
                                  mat_LA128=png_bytes(mat128, "LA"), mat_LA256=png_bytes(mat256, "LA"))
        mres = {}
        vis[name] = {}
        for H in SCALES:
            T = render_truth(S, F, H, floor_px=1.0)      # target: the art with every stroke >= 1 px
            Traw = render_truth(S, F, H)
            row = {}
            outs = {}
            tt = {}
            t0 = time.perf_counter(); rgb, a = legacy(scan, H); tt["legacy"] = time.perf_counter() - t0
            outs["legacy"] = (rgb, a)
            t0 = time.perf_counter(); outs["coverage"] = coverage_raster(scan, H); tt["coverage"] = time.perf_counter() - t0
            outs["coverage512"] = coverage_raster(scan, H, store=512)
            d, _ = floor_delta(m, H)
            inkH = np.broadcast_to(INK, (H, H, 3))
            outs["sdf128_floor"] = (inkH, render_sdf(q128, H, d))
            outs["sdf256_floor"] = (inkH, render_sdf(q256, H, d))
            t0 = time.perf_counter(); outs["sdf512_floor"] = (inkH, render_sdf(q512, H, d)); tt["sdf512"] = time.perf_counter() - t0
            outs["sdf_gapfloor"] = (inkH, render_sdf_gap(q128 if H <= 40 else q512, g128, H, d))
            t0 = time.perf_counter(); outs["mat256"] = (inkH, render_mat(mat256, H)); tt["mat256"] = time.perf_counter() - t0
            for k, (rgb, a) in outs.items():
                comp_img, e, halo = effective(np.asarray(rgb, float), a)
                mm = metrics(e, T)
                mm["halo"] = round(halo, 4)
                mm["mass"] = round(float(e.sum() / max(Traw.sum(), 1e-9)), 2)
                mm["iou_raw"] = round(((e > 0.5) & (Traw > 0.5)).sum() / max(((e > 0.5) | (Traw > 0.5)).sum(), 1), 3)
                row[k] = mm
                vis[name].setdefault(k, {})[H] = crop(comp_img, name, H)
            row["_floor_delta_out_px"] = round(d, 3)
            row["_render_ms"] = {k: round(v * 1000, 1) for k, v in tt.items()}
            mres[H] = row
            if name in ("castle", "mountain") and H == 2560:
                sr = {}
                for R in (64, 128, 256):
                    q = sdf_from_mask(m, R)
                    a = render_sdf(q, H, 0.0)
                    sr[R] = metrics(a, render_truth(S, F, H, floor_px=1.0))
                    vis.setdefault("sdfres_" + name, {})[R] = crop(effective(np.broadcast_to(INK, (H, H, 3)), a)[0], name, H)
                res["sdf_res"][name] = sr
        res["methods"][name] = mres
    json.dump(res, open(f"{OUT}/results.json", "w"), indent=1, default=int)
    np.save(f"{OUT}/vis.npy", vis, allow_pickle=True)

if __name__ == "__main__":
    main()
