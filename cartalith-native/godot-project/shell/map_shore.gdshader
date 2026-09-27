shader_type canvas_item;

// **The base map's smooth shoreline** (RV-4 and "Coastlines and lake shores
// are pixel-stepped", 2026-09-27). The material of `viewport_host.gd`'s
// `map_view` -- the base map's `TextureRect`, which draws the map texture
// NEAREST so that land keeps its crisp cells.
//
// What it does: at a pixel whose four surrounding cell centres are all water
// or all land it changes nothing (the colour is the nearest texel, exactly
// what the `TextureRect` drew before this material existed). At a pixel
// between a water centre and a land centre it asks the shared shore field
// (`shore_field.gdshaderinc`) which side of the smooth shoreline the pixel is
// on, takes that side's colour from the NEAREST of the four texels on that
// side, and blends the two sides by the pixel's water coverage, so the
// coast is antialiased at sub-cell precision instead of following cell edges.
//
// Why a shader and not a finer texture: the shoreline's precision is then the
// screen's own at every zoom below the deep-zoom switch (x2.2), for one float
// per cell (`WorldGen::shore_field_texture`), and the map texture's bytes --
// every style preset, the paper, the grade, the colour space -- are untouched:
// only which of two neighbouring texels a shore pixel shows changes.
//
// Must never invent a colour: every output is a mix of texels of this
// texture. Must never move a cell centre across the shore (the field's clamp
// guarantees it). Off (`shore_on` false) it is the plain nearest-filtered
// texture, which is what a binary without the field, and the reference look
// (`TerrainAppearance::smooth_shores` false), draw.
#include "res://shell/shore_field.gdshaderinc"

uniform sampler2D shore_field : filter_nearest, repeat_disable;
uniform bool shore_on = false;

// The vertex colour (the `TextureRect`'s modulate), carried so a written
// `COLOR` is modulated the way Godot's default fragment would have.
varying vec4 v_mod;

void vertex() {
	v_mod = COLOR;
}

void fragment() {
	// A field of another size describes another grid (a probe's swapped
	// texture, a world mid-rebuild): leave the pixel alone.
	if (shore_on && textureSize(shore_field, 0) == textureSize(TEXTURE, 0)) {
		ivec2 sz = textureSize(TEXTURE, 0);
		vec2 p = UV * vec2(sz) - 0.5;
		vec2 fl = floor(p);
		vec2 f = p - fl;
		ivec2 hi = sz - ivec2(1, 1);
		ivec2 a = clamp(ivec2(fl), ivec2(0, 0), hi);
		ivec2 b = clamp(ivec2(fl) + ivec2(1, 1), ivec2(0, 0), hi);
		// Evaluated before any divergent branch: `fwidth` needs its
		// neighbours to have run the same code.
		float s = shore_value(shore_field, UV);
		float cov = shore_cover(s, fwidth(s));
		ivec2 c0 = ivec2(a.x, a.y);
		ivec2 c1 = ivec2(b.x, a.y);
		ivec2 c2 = ivec2(a.x, b.y);
		ivec2 c3 = ivec2(b.x, b.y);
		bool w0 = texelFetch(shore_field, c0, 0).r > 0.0;
		bool w1 = texelFetch(shore_field, c1, 0).r > 0.0;
		bool w2 = texelFetch(shore_field, c2, 0).r > 0.0;
		bool w3 = texelFetch(shore_field, c3, 0).r > 0.0;
		bool any_w = w0 || w1 || w2 || w3;
		bool any_l = !(w0 && w1 && w2 && w3);
		// Only a pixel between the two classes is redrawn.
		if (any_w && any_l) {
			// Squared distance from this pixel to each centre, in cells; the
			// nearest of each class supplies that side's colour. Later corners
			// win ties (`<=`), as nearest filtering rounds a half up.
			float d0 = dot(f, f);
			float d1 = dot(f - vec2(1.0, 0.0), f - vec2(1.0, 0.0));
			float d2 = dot(f - vec2(0.0, 1.0), f - vec2(0.0, 1.0));
			float d3 = dot(f - vec2(1.0, 1.0), f - vec2(1.0, 1.0));
			vec4 t0 = texelFetch(TEXTURE, c0, 0);
			vec4 t1 = texelFetch(TEXTURE, c1, 0);
			vec4 t2 = texelFetch(TEXTURE, c2, 0);
			vec4 t3 = texelFetch(TEXTURE, c3, 0);
			float bw = 1e9;
			float bl = 1e9;
			vec4 cw = t0;
			vec4 cl = t0;
			if (w0) { if (d0 <= bw) { bw = d0; cw = t0; } } else { if (d0 <= bl) { bl = d0; cl = t0; } }
			if (w1) { if (d1 <= bw) { bw = d1; cw = t1; } } else { if (d1 <= bl) { bl = d1; cl = t1; } }
			if (w2) { if (d2 <= bw) { bw = d2; cw = t2; } } else { if (d2 <= bl) { bl = d2; cl = t2; } }
			if (w3) { if (d3 <= bw) { bw = d3; cw = t3; } } else { if (d3 <= bl) { bl = d3; cl = t3; } }
			COLOR = mix(cl, cw, cov) * v_mod;
		}
	}
}
