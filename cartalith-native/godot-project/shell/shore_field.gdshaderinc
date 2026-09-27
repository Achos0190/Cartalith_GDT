// **The smooth shoreline, shared** (RV-4, 2026-09-27: "Coastlines and lake
// shores are pixel-stepped"). Included by the two shaders that must agree on
// where the water is at fit zoom: `map_shore.gdshader` (the base map, which
// draws the water) and `river_under_water.gdshader` (the river stroke, which
// hides under it).
//
// `shore_field` is `WorldGen::shore_field_texture`: one float per cell,
// `render::shore_field`'s signed water depth -- the water surface minus the
// ground, in height units, positive on every cell the map classifies as water
// and negative on every cell it classifies as land. The shoreline is where its
// bilinear interpolation between cell centres crosses zero: for the sea that
// is the height field's own contour at sea level (the line
// `cartalith_terrain::vector::trace_coastline` traces), for an above-sea lake
// the contour at its pooled surface, so it is smooth wherever the terrain is.
//
// Two properties every caller relies on, both from the field's clamp and not
// from this file: a cell centre the map classifies as water reads water here
// and a land one reads land (classification never changes), and a point whose
// four surrounding cell centres are all water reads water (a convex mix of
// positives) -- the region `river_stroke.rs` carries every river mouth into.
//
// Must never be sampled with hardware filtering: the bilinear below is the
// definition, fetched texel by texel so both shaders compute the same number.

// Bilinear `shore_field` at texture UV `uv` (texel centres at `(i + 0.5) /
// size`, the same UV the map's `TextureRect` and the stroke's mesh use),
// clamped at the grid's edge.
float shore_value(sampler2D fld, vec2 uv) {
	ivec2 sz = textureSize(fld, 0);
	vec2 p = uv * vec2(sz) - 0.5;
	vec2 fl = floor(p);
	vec2 f = p - fl;
	ivec2 hi = sz - ivec2(1, 1);
	ivec2 a = clamp(ivec2(fl), ivec2(0, 0), hi);
	ivec2 b = clamp(ivec2(fl) + ivec2(1, 1), ivec2(0, 0), hi);
	float s00 = texelFetch(fld, ivec2(a.x, a.y), 0).r;
	float s10 = texelFetch(fld, ivec2(b.x, a.y), 0).r;
	float s01 = texelFetch(fld, ivec2(a.x, b.y), 0).r;
	float s11 = texelFetch(fld, ivec2(b.x, b.y), 0).r;
	return mix(mix(s00, s10, f.x), mix(s01, s11, f.x), f.y);
}

// How much of this pixel the water covers, 0..1, from the field's value `s`
// and its screen-space change `w` (`fwidth(s)`, taken by the caller in uniform
// control flow): the signed distance to the zero crossing in pixels, plus a
// half. `w == 0` (a flat field) degenerates to a hard step at zero.
float shore_cover(float s, float w) {
	return clamp(0.5 + s / max(w, 1e-12), 0.0, 1.0);
}
