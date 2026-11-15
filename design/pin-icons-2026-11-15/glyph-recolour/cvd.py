"""Colour-vision-deficiency simulation and colour difference, for choosing and
checking the glyph set's category colours.

Simulation: Machado, Oliveira & Fernandes (2009) matrices at severity 1.0
(full deuteranopia / protanopia), applied in linear RGB.
Difference: CIE76 delta-E in CIELAB (D65). A labelled judgement: pairs under
about 12 are hard to tell apart as small coloured rims; 20 and up are easy.
"""
import numpy as np
DEUTAN = np.array([[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.011820, 0.042940, 0.968881]])
PROTAN = np.array([[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]])
def to_lin(c):
    c = np.asarray(c, np.float64) / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
def to_srgb(l):
    l = np.clip(l, 0, 1)
    return np.where(l <= 0.0031308, l * 12.92, 1.055 * l ** (1 / 2.4) - 0.055) * 255.0
def simulate(rgb, M):
    """rgb: (...,3) sRGB 0..255 -> simulated sRGB 0..255."""
    return to_srgb(to_lin(rgb) @ M.T)
def lab(rgb):
    l = to_lin(rgb)
    X = l @ np.array([0.4124, 0.3576, 0.1805]); Y = l @ np.array([0.2126, 0.7152, 0.0722]); Z = l @ np.array([0.0193, 0.1192, 0.9505])
    f = lambda t: np.where(t > 0.008856, np.cbrt(t), 7.787 * t + 16 / 116)
    fx, fy, fz = f(X / 0.95047), f(Y), f(Z / 1.08883)
    return np.stack([116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)], -1)
def dE(a, b): return float(np.linalg.norm(lab(a) - lab(b)))
def hexc(h): h = h.lstrip('#'); return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))
