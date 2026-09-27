shader_type canvas_item;

// **Water above rivers, in the base view** (owner, 2026-09-27: "the ocean
// texture should be drawn above the river graphic"). The material of the
// canvas item `map_overlay.gd::_draw_rivers` draws the river stroke into --
// that stroke and nothing else.
//
// The stroke's UV is its own ground position in the map texture
// (`WorldGen::river_view_mesh`), so `water_mask` sampled at UV reads the
// cell under this fragment. The mask is `WorldGen::river_water_mask`: 255 on
// every cell the map texture draws as water. The base map's `TextureRect`
// draws that texture NEAREST (`viewport_host.gd::_raster`), so sampling the
// mask NEAREST at the same UV answers exactly the question "is this screen
// pixel drawn as water?", and discarding there leaves the water on top --
// flush with the coast the map itself draws, with no second, smoother
// shoreline that would disagree with it.
//
// Must never discard anything but water: every land pixel keeps the
// stroke's own colour and antialiasing (`COLOR`, unwritten, is the vertex
// colour times the river colour texture, Godot's default).
uniform sampler2D water_mask : filter_nearest, repeat_disable;

// The stroke's UV is laid out over `map_overlay.gd`'s fit rect; the base map's
// `TextureRect` draws over a rect Godot truncates to whole pixels, up to a
// pixel off it (`map_overlay.gd::_map_texture_rect`). `mask_map` takes the
// one to the other -- `uv * zw + xy` -- so the mask is read at the texel the
// map itself shows at this pixel. Identity by default (no offset, unit scale).
uniform vec4 mask_map = vec4(0.0, 0.0, 1.0, 1.0);

void fragment() {
	// 0.5: the mask is 0 or 1 (255), and nearest sampling never mixes the two.
	if (texture(water_mask, UV * mask_map.zw + mask_map.xy).r > 0.5) {
		discard;
	}
}
