shader_type canvas_item;

// Deep-zoom tile compositing (`viewport_host.gd`'s `_build_lod_tile`).
//
// A tile's own texture is NOT a picture -- it is the relief-detail shade
// ratio `lod_bridge::synthesize_tile_rgba` computes: how much the procedural
// sub-cell detail changes the hillshade at that pixel, relative to the same
// ground with no detail added. The colour comes from `base_tex`, the very
// texture `map_view` shows (`EngineBridge.color_texture()`), sampled at the
// same ground position. That is what makes the deep-zoom view and the base
// map agree by construction rather than by two palettes happening to match --
// see `lod_bridge.rs`'s "What a tile actually contains" section for why the
// reference's own `_lodBuildTileRGBA` gate is what this reproduces.
//
// `filter_linear` on `base_tex` is the second half of this milestone's point:
// `viewport_host.gd`'s `_raster()` sets NEAREST on `map_view` (correct at fit
// zoom, where one cell is about one screen pixel), which is exactly the
// blocky single-cell squares the owner complained about once a cell covers
// many pixels. Sampling the same texture linearly here removes them without
// touching how the map draws at any zoom below the threshold.
uniform sampler2D base_tex : filter_linear, repeat_disable;

// This tile's footprint in `base_tex`'s UV space -- cell `[origin, origin+n]`
// over `[0, grid]`, matching the screen rect `_lod_tile_rect` gives the node.
uniform vec2 base_uv0;
uniform vec2 base_uv1;

// Must match `lod_bridge::SHADE_RATIO_MID` / `SHADE_RATIO_GAIN`.
const float SHADE_RATIO_MID = 128.0;
const float SHADE_RATIO_GAIN = 256.0;

void fragment() {
	float ratio = 1.0 + (texture(TEXTURE, UV).r * 255.0 - SHADE_RATIO_MID) / SHADE_RATIO_GAIN;
	vec4 base = texture(base_tex, mix(base_uv0, base_uv1, UV));
	COLOR = vec4(base.rgb * ratio, base.a);
}
