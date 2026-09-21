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
// **This is LOD-D3's morph partner for a tile that has no live parent tile**
// -- the scope's *"the entry level's parent is `map_view.texture` sampled
// linearly"*. It was declared and wired by LOD-D2 with no consumer, reserved
// in as many words for this milestone; the UV mapping `_build_lod_tile()`
// derives for it has been exercised since, which is the whole reason it was
// kept rather than re-derived here against a camera transform whose errors
// read as seams rather than as bugs.
//
// `filter_linear` is load-bearing and is not what the screen would do on its
// own: `map_view`'s own sprite draws with `TEXTURE_FILTER_NEAREST`
// (`_raster()`), so letting the base show through from underneath would blend
// a deep-zoom tile against visible coarse-cell squares. Sampled here it is
// smooth, which is what a parent is supposed to look like.
uniform sampler2D base_tex : filter_linear, repeat_disable;

// **LOD-D3's morph parameter `t`**, from `WorldGen::lod_morph` -- see
// `lod_bridge::morph_for_zoom`, which is `lod_level_for_zoom`'s own expression
// without the rounding. `0.0` draw the parent, `1.0` draw this tile.
//
// Not computed here, and not in GDScript either: the fade and the level switch
// have to agree about where the level boundary is, and two copies of one
// expression is exactly how they would come to disagree. The engine answers
// both questions from one place.
//
// Defaults to `1.0` so a material whose parameter was never set draws its tile
// whole. A `0.0` default would make a missed `set_shader_parameter` look like
// an empty layer, which is the failure mode hardest to attribute.
uniform float morph : hint_range(0.0, 1.0) = 1.0;

// Whether a parent TILE is drawn underneath this one (`viewport_host.gd` keeps
// the level above alive and orders it behind -- LOD-D3's parent fallback), as
// opposed to nothing but the base map.
//
// Set the same way for **every tile of a level at once**, never per tile:
// `_refresh_lod_blend()` turns it on only when the whole parent set is live,
// because two children side by side in different modes would meet along their
// shared edge with the base map on one side of it and a parent tile on the
// other -- a seam of half the tile-versus-raster difference, mid-fade, that
// moves as the parent set fills in. Its own note there carries the reasoning.
//
// It selects which of the two blends below is correct, and they are not
// interchangeable:
//
//   * with a parent underneath, this fragment must leave the parent visible,
//     so it emits `morph` as its ALPHA and the hardware blend does the mix --
//     `tile*t + parent*(1-t)`, which is the scope's `mix(parent_sample,
//     tile_sample, t)` with the parent supplied by the node below instead of
//     by a second sampler. One texture sample, which is the milestone's own
//     per-fragment budget.
//   * with no parent, the blend partner is `base_tex`, which is sampled here
//     rather than drawn underneath (see its own note), so the mix happens in
//     the shader and the fragment stays opaque.
//
// Emitting alpha in the second case too would blend against `map_view`'s
// NEAREST-filtered sprite and reintroduce coarse-cell squares under a fading
// tile; sampling `base_tex` in the first case would blend against the base map
// where a much finer parent tile is available, which is a visible loss of
// detail at every level change.
uniform bool under_parent = false;

void fragment() {
	vec4 tile = texture(TEXTURE, UV);
	if (under_parent) {
		COLOR = vec4(tile.rgb, morph * tile.a);
	} else {
		vec4 base = texture(base_tex, mix(base_uv0, base_uv1, UV));
		COLOR = vec4(mix(base.rgb, tile.rgb, morph * tile.a), 1.0);
	}
}
