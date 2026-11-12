// 3D-2's terrain shader (`3D_VIEWPORT_SCOPE.md` §5, milestone 3D-2; §3.1,
// §3.4, §3.6, §3.11). Used by `view3d_terrain.gd` for its two meshes, the
// near window and the whole-world backdrop; both are the same 257 x 257
// vertex unit patch (VERTEX.xz in [0, 1]) placed by the node's transform.
//
// What it does:
// - Displaces each vertex by the relief texture (normalised [0, 1] heights from
//   `WorldGen.view3d_tile()`, the very heights the 2D tile was coloured from)
//   turned into metres by `sample_bridge::FieldRefs::elevation_m`'s law,
//   `(h - sea) / (1 - sea) * peak_m`, times the view's exaggeration.
// - Samples the relief at `h_lod`, the mip level whose texel matches one
//   vertex step: a box low-pass, so geometry carries the silhouette and the
//   drape carries the detail (§3.2; §8.3's candidate (a), because the raw
//   deep-zoom heights displaced vertex for vertex read as needles).
// - Flattens everything below sea level to the sea surface (§3.6's default).
// - Draws the drape unlit (§3.4's "map shading", Basic's mode): the 2D tile
//   already bakes its hillshade, and a light on top would shade it twice.
//
// What it must never do: write DEPTH (it would disable early-z, §3.11), or
// change anything the 2D map draws (it only reads textures 2D also reads).
shader_type spatial;
render_mode unshaded, cull_back, depth_draw_opaque;

uniform sampler2D hmap : filter_linear_mipmap, repeat_disable;
uniform sampler2D cmap : source_color, filter_linear_mipmap, repeat_disable;
// Patch coordinate -> texture coordinate, as (scale.xy, offset.xy):
// uv = VERTEX.xz * xform.xy + xform.zw. Set by `view3d_terrain.gd`, which
// knows each texture's own pixel-to-sample layout.
uniform vec4 h_xform = vec4(1.0, 1.0, 0.0, 0.0);
uniform vec4 c_xform = vec4(1.0, 1.0, 0.0, 0.0);
uniform float h_lod = 0.0;
uniform float sea_level = 0.42;
uniform float peak_m = 4000.0;
uniform float exag = 1.0;
// Metres subtracted from every height: 0 for the window; the backdrop's sink,
// so the window always draws over the ground the backdrop holds under its rim.
uniform float sink_m = 0.0;
// The backdrop's hole, in patch coordinates (x0, y0, x1, y1): fragments inside
// are discarded because the window draws that ground. x0 > x1 means no hole.
uniform vec4 hole = vec4(1.0, 1.0, 0.0, 0.0);
// The window's rim blend: within `rim` of the patch edge (patch units) the
// height eases toward the backdrop's own relief, unsunk, sampled from `hfar`
// through `hf_xform` (patch -> backdrop uv), so at the edge the window meets
// the backdrop `sink_m` above it and wins the depth test (bar B5).
// rim = 0 turns it off (the backdrop itself).
uniform sampler2D hfar : filter_linear_mipmap, repeat_disable;
uniform vec4 hf_xform = vec4(1.0, 1.0, 0.0, 0.0);
uniform float hf_lod = 0.0;
uniform float rim = 0.0;

varying vec2 v_patch;

float metres(float h) {
	return max((h - sea_level) / max(1.0 - sea_level, 1e-6) * peak_m, 0.0) * exag;
}

void vertex() {
	vec2 p = VERTEX.xz;
	v_patch = p;
	float y = metres(textureLod(hmap, p * h_xform.xy + h_xform.zw, h_lod).r) - sink_m;
	if (rim > 0.0) {
		vec2 d = min(p, 1.0 - p);
		float e = smoothstep(0.0, rim, min(d.x, d.y));
		float yf = metres(textureLod(hfar, p * hf_xform.xy + hf_xform.zw, hf_lod).r);
		y = mix(yf, y, e);
	}
	VERTEX.y = y;
}

void fragment() {
	if (hole.x <= hole.z && v_patch.x > hole.x && v_patch.x < hole.z && v_patch.y > hole.y && v_patch.y < hole.w) {
		discard;
	}
	ALBEDO = texture(cmap, v_patch * c_xform.xy + c_xform.zw).rgb;
}
