// AP-5 (Ruling CQ Q3; Ruling CR's SDF path): the scattered feature icons,
// drawn by `map_overlay.gd`'s feature-icon layer (`_rebuild_feature_icons`)
// at screen resolution instead of baked into the grid raster. One material
// for the whole layer, so the painter's order between line-art and painted
// sprites is the order the layer draws them in.
//
// Two paths, chosen per sprite by the engine (`pack::feature_texture`) and
// signalled through the integer part of UV (never through COLOR, which the
// canvas modulate scales):
//
//   UV.x < 1.5   PAINTED. The texture is premultiplied and carries its own
//                area-filtered mip chain (`pack::feature_raster_texture`);
//                the layer's LINEAR_WITH_MIPMAPS filter reads it, and the
//                colour is divided back out, so a transparent texel's RGB
//                never darkens an edge. Plain filtering: magnified past its
//                256² source a painted sprite's soft edge widens with the
//                zoom (measured and reported in STATUS.md, AP-5).
//
//   UV.x >= 1.5  LINE ART (one ink). The texture's alpha is AP-10's signed
//                distance field (`lineart::Sdf`: 128 is the edge, one level
//                is SDF_SPREAD_TEXELS / 127 of that mip level's own texels;
//                RGB is the ink). UV.x = 2 + 2k + u with k the sprite's
//                2nd-percentile stroke width in top-level texels times 8,
//                and UV.y = 2m + v with m the deepest mip level that holds a
//                real distance level. The level read is the largest with at
//                most 4 texels per screen pixel (`lineart::MAX_TEXELS_PER_PIXEL`),
//                and alpha = clamp(d_px + delta + 0.5): a one-pixel ramp on
//                the contour, lifted by Ruling CR's stroke floor delta =
//                max(0, (1 - stroke_px) / 2) so no stroke draws thinner than
//                1 px (`lineart::floor_offset`, line weight 1.0). The same
//                formula as `lineart::render_coverage`, evaluated per screen
//                pixel instead of per bake texel.
//
// Must never be given anything but that layer's sprites: the UV code would
// read any other draw's UVs as parameters.
shader_type canvas_item;

// `lineart::SDF_SPREAD_TEXELS` (8.0) over the quantiser's 127 steps.
const float SDF_LEVEL_TEXELS = 8.0 / 127.0;
// `lineart::MAX_TEXELS_PER_PIXEL`.
const float MAX_TEXELS_PER_PIXEL = 4.0;
// Ruling CR Q-L1's default line weight at map scale, in screen pixels.
const float LINE_WEIGHT_PX = 1.0;

varying vec4 v_col;

void vertex() {
	v_col = COLOR;
}

void fragment() {
	vec2 tsz = 1.0 / TEXTURE_PIXEL_SIZE;
	if (UV.x >= 1.5) {
		// Decode with a quarter-unit margin (+0.25 on a stride of 2): a helper
		// invocation just outside the quad sees u or v slightly below 0, and a
		// bare floor() would decode the neighbouring code there.
		float k = floor(UV.x * 0.5 + 0.25) - 1.0;
		float max_lod = floor(UV.y * 0.5 + 0.25);
		vec2 uv = vec2(UV.x - 2.0 * (k + 1.0), UV.y - 2.0 * max_lod);
		// Top-level texels per screen pixel, the larger axis. From the RAW UV,
		// never the decoded one: the codes are constant across a quad, so the
		// derivatives are the same, and a decoded value can jump between a
		// pixel and its helper neighbour at the quad's edge (measured: every
		// line-art quad drew its own outline before this).
		vec2 t = UV * tsz;
		float tpp = max(max(length(dFdx(t)), length(dFdy(t))), 1e-6);
		float lod = clamp(ceil(log2(max(tpp / MAX_TEXELS_PER_PIXEL, 1.0))), 0.0, max_lod);
		vec4 s = textureLod(TEXTURE, uv, lod);
		float d_px = (s.a * 255.0 - 128.0) * SDF_LEVEL_TEXELS * exp2(lod) / tpp;
		float stroke_px = (k / 8.0) / tpp;
		float delta = max(0.0, (LINE_WEIGHT_PX - stroke_px) * 0.5);
		float a = clamp(d_px + delta + 0.5, 0.0, 1.0);
		COLOR = vec4(s.rgb * v_col.rgb, a * v_col.a);
	} else {
		vec4 p = texture(TEXTURE, UV);
		vec3 rgb = p.a > 0.0 ? p.rgb / p.a : vec3(0.0);
		COLOR = vec4(rgb, p.a) * v_col;
	}
}
