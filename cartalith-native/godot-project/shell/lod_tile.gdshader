shader_type canvas_item;

// Deep-zoom tile compositing (`viewport_host.gd`'s `_build_lod_tile`).
//
// **A tile is a picture again, since `LOD_DETAIL_SCOPE.md` LOD-D2.** From
// 2026-08-23 to 2026-09-21 this shader decoded a relief-detail shade ratio --
// one byte per pixel, `128` meaning "the sub-cell detail changes nothing
// here" -- and multiplied it into `base_tex`, the very texture `map_view`
// shows. That made the deep-zoom view and the base map agree by construction,
// and it also capped what a deeper level could ever show: a scalar multiplier
// can redistribute the colours the base raster already has at that pixel, and
// it cannot add a material boundary, a crest stroke or a river band that the
// base raster does not have at all. Measured, which is why it changed: detail
// per screen pixel fell from 0.0220 at zoom 1 to 0.0017 at zoom 40.
//
// `lod_bridge::synthesize_tile_rgba` now returns `renderBiomeTileRGBA`'s own
// output (`render::render_biome_tile_rgba`, LOD-D1's port, golden-verified
// byte-identical against the frozen reference) -- the same coloriser the map
// itself runs, evaluated at tile resolution. So the fragment is the tile,
// and the agreement that the shade ratio bought structurally is now bought by
// the two paths running the same function over the same fields.
//
// `TEXTURE` is sampled with the filter `_build_lod_tile` sets on the sprite
// (`TEXTURE_FILTER_LINEAR`), not `NEAREST` -- the whole reason this milestone
// exists is to stop showing blocky single-cell squares at deep zoom.

// This tile's footprint in `base_tex`'s UV space -- cell `[origin, origin+n]`
// over `[0, grid]`, matching the screen rect `_lod_tile_rect` gives the node.
uniform vec2 base_uv0;
uniform vec2 base_uv1;

// The base map, sampled at the same ground position as this fragment.
//
// **Read only where the tile is transparent**, which is nowhere today:
// `render_biome_tile_rgba` writes alpha `255` at every pixel, so `mix` below
// resolves to the tile and this sampler contributes nothing to a shipped
// frame. It is kept, wired and correct rather than deleted for one stated
// reason: LOD-D3 ("Continuous transitions: parent fallback and a colour-space
// morph") needs exactly this -- a second colour to blend toward, at the same
// ground position, with the UV mapping already derived and already exercised.
// Deleting it would mean re-deriving `base_uv0`/`base_uv1` in the milestone
// that needs them most, against a camera transform that is easy to get subtly
// wrong and whose error looks like a seam rather than like a bug.
//
// Stated as an unused input rather than dressed up as a fallback: nothing in
// this build can produce a partially transparent tile.
uniform sampler2D base_tex : filter_linear, repeat_disable;

void fragment() {
	vec4 tile = texture(TEXTURE, UV);
	vec4 base = texture(base_tex, mix(base_uv0, base_uv1, UV));
	COLOR = vec4(mix(base.rgb, tile.rgb, tile.a), 1.0);
}
