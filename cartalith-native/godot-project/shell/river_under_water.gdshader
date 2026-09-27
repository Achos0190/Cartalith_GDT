shader_type canvas_item;

// **Water above rivers, in the base view** (owner, 2026-09-27: "the ocean
// texture should be drawn above the river graphic"). The material of the
// canvas item `map_overlay.gd::_draw_rivers` draws the river stroke into --
// that stroke and nothing else.
//
// The stroke's UV is its own ground position in the map texture
// (`WorldGen::river_view_mesh`), so a field sampled at UV reads the ground
// under this fragment.
//
// **With the smooth shoreline (RV-4, 2026-09-27)** -- `shore_on`, whenever the
// base map draws one (`map_shore.gdshader`): the stroke reads the SAME shore
// field at the same point through the same function
// (`shore_field.gdshaderinc`), and fades out exactly as the map's water fades
// in -- its alpha times one minus the water's coverage of this pixel,
// discarded where the water covers it fully. So the water sits above the
// river along the smooth coast the map draws, with no river pixel on the
// water and no gap between the river and the water. The river mouths are
// carried into the region where all four surrounding cell centres are water
// (`river_stroke.rs`), which this field reads as water whatever its values.
//
// **Without it** (`shore_on` false: an older binary, or the reference look,
// `TerrainAppearance::smooth_shores` false) the map is the plain nearest-filtered
// texture, and the stroke discards where `water_mask`
// (`WorldGen::river_water_mask`, 255 on every cell the map draws as water),
// sampled NEAREST like the map, says water: flush with the cell coast.
//
// Must never discard or fade anything but water: every land pixel keeps the
// stroke's own colour and antialiasing (`COLOR`, unwritten there, is the vertex
// colour times the river colour texture, Godot's default).
#include "res://shell/shore_field.gdshaderinc"

uniform sampler2D water_mask : filter_nearest, repeat_disable;
uniform sampler2D shore_field : filter_nearest, repeat_disable;
uniform bool shore_on = false;

// The stroke's UV is laid out over `map_overlay.gd`'s fit rect; the base map's
// `TextureRect` draws over a rect Godot truncates to whole pixels, up to a
// pixel off it (`map_overlay.gd::_map_texture_rect`). `mask_map` takes the
// one to the other -- `uv * zw + xy` -- so the mask is read at the texel the
// map itself shows at this pixel. Identity by default (no offset, unit scale).
uniform vec4 mask_map = vec4(0.0, 0.0, 1.0, 1.0);

void fragment() {
	vec2 muv = UV * mask_map.zw + mask_map.xy;
	if (shore_on) {
		// `fwidth` before the branch below, in uniform control flow.
		float s = shore_value(shore_field, muv);
		float cov = shore_cover(s, fwidth(s));
		// 1.0: the water is opaque here, nothing of the river may show.
		if (cov >= 1.0) {
			discard;
		}
		COLOR.a *= 1.0 - cov;
	} else if (texture(water_mask, muv).r > 0.5) {
		// 0.5: the mask is 0 or 1 (255), and nearest sampling never mixes the two.
		discard;
	}
}
