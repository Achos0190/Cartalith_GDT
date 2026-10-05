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
// **RIM-1/3/6 (2026-10-04, Ruling BU): the river is painted here too.** With
// `river_on` the same pass paints every river as water in the map's own
// pixels, from `river_field` (`WorldGen::river_field_texture`: per texel the
// distance to the nearest drawn centreline, that river's half-width and its
// order-1 weight) and the styled colour `river_color`
// (`WorldGen::river_color_texture`, the map with every river at full
// coverage): coverage is RV-2's own stroke law (`river_px_width` and the
// one-pixel fringe: half-width in screen pixels `max(hw * ppc * wm * rw, 0.5)`,
// alpha falling linearly to zero over one more pixel, the order-1 de-emphasis
// on both) and is multiplied by one minus the shore's water coverage, so a
// river meets a lake or the sea with no seam and is never drawn on water.
// Just outside a river's edge a lighter, greener floodplain tint fades over a
// band that scales with the channel (`river_field.rs` `FLOODPLAIN_*`).
// `river_field.rs::coverage` is this law mirrored, pinned against the stroke.
// **RIM-5**: the order-1 share's alpha also takes `o1_distance_fade(ppc)` (a
// smoothstep from `O1_GONE_PPC` to `O1_FADE_FULL_PPC` pixels per cell, 1 from
// `O1_FADE_FULL_PPC` up), so headwater streams thin out of a zoomed-out map
// without popping. RIM-7 (2026-10-05): the deep-zoom tiles and the export
// paint the river by this same law in Rust (`river_field::pixel_paint`,
// `PaintSource::raster`) -- keep the two in step; the mirror tests read this
// file.
// **RIM-2**: with `river_bank` above zero a one-pixel line in `river_bank_color`
// (the style's river ink) is drawn on the river's visible edge, `river_field.rs`
// `bank_coverage` mirrored: centred on the 50 % point of the fringe, inward only
// as far as the river has a body (a 1 px stream takes no ink on its bed),
// outward only as far as the field's valid band less a margin (it thins to
// nothing at a zoomed-out view rather than ending on a step), times the same
// order-1 alpha (RIM-5: no orphan outline on a faded stream) and plate-frame
// weight as the water. `river_bank` 0 -- every preset with no opinion -- runs
// none of it, so the render is what it was, bar a measured 1-LSB single-pixel residue
// in 5 of 32 probe frames (cause unproven, probably driver rounding after this recompile).
//
// Must never invent a colour: every output is a mix of texels of this
// texture, `river_color` (itself a map render), or -- the one exception, RIM-2 --
// `river_bank_color`, the preset's own ungraded river ink. Must never move a cell centre across the shore (the field's clamp
// guarantees it). Off (`shore_on` false) it is the plain nearest-filtered
// texture, which is what a binary without the field, and the reference look
// (`TerrainAppearance::smooth_shores` false), draw.
#include "res://shell/shore_field.gdshaderinc"

uniform sampler2D shore_field : filter_nearest, repeat_disable;
uniform bool shore_on = false;

// RIM: the river's distance field (RGBA half float, `gw * k` by `gh * k`
// texels, k 1 or 2: R distance in cells to the nearest centreline, G its
// half-width in cells, B the order-1 weight, A the plate-frame weight) and its colour map, both read
// at the pixel's own UV with hardware linear filtering. `river_on` follows
// the Rivers layer switch and the field being present; `river_width` is the
// preset's multiplier the field was built for (`river_paint_params`).
uniform sampler2D river_field : filter_linear, repeat_disable;
uniform sampler2D river_color : filter_linear, repeat_disable;
uniform bool river_on = false;
uniform float river_width = 1.0;
// 0 draws no floodplain tint; 1 is the full tint below.
uniform float floodplain_strength = 1.0;
// RIM-2: the bank outline's opacity (0 = off, the default and what every preset
// with no opinion hands over, `WorldGen::river_paint_params`) and its ink, the
// preset's `river_ink_r/g/b` as 0..1. The ink is NOT passed through the style's
// grade, as the river's own colour map is: a preset that grades heavily
// (`grade_temperature`, exposure) should keep `river_bank` small or off.
uniform float river_bank = 0.0;
uniform vec3 river_bank_color = vec3(0.0);

// The order-1 de-emphasis, `river_stroke::o1_deemphasis`: `zk` is pixels per
// cell, clamped to 1 below; faded out by 8 (the reference's `zk = 8`).
// x: width multiplier, y: alpha multiplier.
vec2 o1_deemphasis(float zk) {
	float de = clamp(1.0 - (max(zk, 1.0) - 1.0) / 7.0, 0.0, 1.0);
	return vec2(0.55 + 0.45 * (1.0 - de), 0.4 + 0.6 * (1.0 - de));
}

// RIM-5, `river_stroke::o1_distance_fade`: 0 at or below `O1_GONE_PPC` pixels
// per cell, 1 at or above `O1_FADE_FULL_PPC`, a smoothstep between. Labelled
// judgement, tuned on the owner's world; keep both in step with the Rust.
const float O1_GONE_PPC = 0.2;
const float O1_FADE_FULL_PPC = 0.8;
float o1_distance_fade(float ppc) {
	return smoothstep(O1_GONE_PPC, O1_FADE_FULL_PPC, ppc);
}

// RIM-2, `river_field.rs`: the bank outline's full-strength width in screen
// pixels, the margin it keeps from the field's valid band in cells, and that
// band's guaranteed least (`river_field::BAND_CELLS`). Labelled judgement for
// the first two (see the Rust); keep all three in step with it -- a test reads
// them from here.
const float BANK_WIDTH_PX = 1.0;
const float BANK_BAND_MARGIN_CELLS = 1.0;
const float FIELD_BAND_CELLS = 3.5;

// The floodplain tint: a lift toward a lighter, greener land, **labelled
// judgement** tuned on the owner's world (before/after sheets, 2026-10-04):
// enough to read as a flat valley floor beside the water, not a second colour.
vec3 floodplain(vec3 land, float w) {
	return mix(land, land * vec3(0.96, 1.03, 0.94) + vec3(0.012, 0.034, 0.004), clamp(w, 0.0, 1.0));
}

// The vertex colour (the `TextureRect`'s modulate), carried so a written
// `COLOR` is modulated the way Godot's default fragment would have.
varying vec4 v_mod;

void vertex() {
	v_mod = COLOR;
}

void fragment() {
	ivec2 sz = textureSize(TEXTURE, 0);
	ivec2 fsz = textureSize(river_field, 0);
	// A field of another size describes another grid (a probe's swapped
	// texture, a world mid-rebuild): leave the pixel alone. The river field is
	// the map's grid at 1 or 2 texels a cell.
	bool shore = shore_on && textureSize(shore_field, 0) == sz;
	bool paint = river_on && shore && fsz.x >= sz.x && fsz.x % sz.x == 0 && fsz.y == (fsz.x / sz.x) * sz.y;
	// Everything that differentiates is evaluated before any divergent branch:
	// `fwidth` needs its neighbours to have run the same code.
	float s = shore ? shore_value(shore_field, UV) : 0.0;
	float cov = shore ? shore_cover(s, fwidth(s)) : 0.0;
	float ppc = 1.0 / max(fwidth(UV.x * float(sz.x)), 1e-6);
	vec4 rf = paint ? texture(river_field, UV) : vec4(64.0, 0.0, 0.0, 0.0);
	vec3 rcol = paint ? texture(river_color, UV).rgb : vec3(0.0);

	// The river's coverage of this pixel, RV-2's stroke law, and the
	// floodplain's weight. Both are zero with no river in reach (the field's
	// sentinel is 64 cells).
	float rcov = 0.0;
	float fp = 0.0;
	float bnk = 0.0;
	if (paint) {
		float o1 = clamp(rf.b, 0.0, 1.0);
		vec2 dm = o1_deemphasis(ppc);
		float wm = mix(1.0, dm.x, o1);
		float am = mix(1.0, dm.y * o1_distance_fade(ppc), o1);
		float hw_px = max(rf.g * ppc * wm * river_width, 0.5);
		// rf.a: the plate frame's weight (`river_field.rs`, `1 - border_cover`)
		// -- the river never lies on the bare-paper margin the raster baked.
		rcov = clamp(hw_px + 1.0 - rf.r * ppc, 0.0, 1.0) * am * rf.a;
		float edge = rf.g * river_width;
		float e = rf.r - edge;
		fp = (1.0 - smoothstep(0.0, 1.0 + edge, e)) * (1.0 - o1) * smoothstep(0.35, 1.0, edge) * floodplain_strength * rf.a;
		// RIM-2, `river_field::bank_coverage`: a line centred on the visible
		// edge (`be` pixels outside it), reaching in only as far as the river
		// has a body (a 1 px stream: not at all) and out only as far as the
		// field's valid band less a margin. Off (0) runs none of this.
		if (river_bank > 0.0) {
			float be = rf.r * ppc - (hw_px + 0.5);
			float reach = 0.5 * BANK_WIDTH_PX + 0.5;
			float room = max((rf.g * river_width + FIELD_BAND_CELLS - BANK_BAND_MARGIN_CELLS) * ppc - hw_px - 0.5, 0.0);
			float ring = be < 0.0 ? clamp(min(reach, max(hw_px - 0.5, 0.0)) + be, 0.0, 1.0) : clamp(min(reach, room) - be, 0.0, 1.0);
			bnk = ring * clamp(river_bank, 0.0, 1.0) * am * rf.a;
		}
	}

	if (shore) {
		ivec2 hi = sz - ivec2(1, 1);
		vec2 p = UV * vec2(sz) - 0.5;
		vec2 fl = floor(p);
		vec2 f = p - fl;
		ivec2 a = clamp(ivec2(fl), ivec2(0, 0), hi);
		ivec2 b = clamp(ivec2(fl) + ivec2(1, 1), ivec2(0, 0), hi);
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
		// Only a pixel between the two classes is redrawn by the shore.
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
			// The river lies on the land side only, under the water: the land
			// colour takes the floodplain and the river, then the shore's
			// coverage mixes the water over it.
			vec3 land = mix(floodplain(cl.rgb, fp), rcol, rcov);
			if (bnk > 0.0) {
				land = mix(land, river_bank_color, bnk);
			}
			COLOR = mix(vec4(land, cl.a), cw, cov) * v_mod;
		} else if (!any_w && paint) {
			// All land: the pixel is the nearest texel (what the TextureRect
			// drew), with the floodplain and river over it.
			vec4 t = COLOR;
			vec3 lnd = mix(floodplain(t.rgb, fp), rcol * v_mod.rgb, rcov);
			if (bnk > 0.0) {
				lnd = mix(lnd, river_bank_color * v_mod.rgb, bnk);
			}
			COLOR = vec4(lnd, t.a);
		}
	}
}
