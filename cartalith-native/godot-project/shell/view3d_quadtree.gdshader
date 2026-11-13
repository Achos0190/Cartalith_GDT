// 3D-3's quadtree terrain shader (`3D_VIEWPORT_SCOPE.md` §5, milestone 3D-3;
// §3.2, §3.3, §3.5, §3.6, §3.11). Used by `view3d_quadtree.gd`'s one
// MultiMesh: every drawn quadtree node is one instance of one shared patch
// (VERTEX.xz in [0, 1]; VERTEX.y = -1 marks a skirt vertex, 0 the surface).
//
// What it does:
// - Computes the level `L_3d(x)` per vertex and per fragment, the 2D
//   formula with px-per-cell from the distance to the camera (§3.3):
//     ppc(x) = cell_m * f_px / max(cell_m, |x - camera|)   (perspective)
//     ppc    = ortho_ppc                                    (top-down)
//     L      = log2(max(1, gw * max(0.01, ppc) / 256))      (lod_bridge::continuous_level)
//            + smoothstep01(log2(ppc / ppc_entry)) * b(x)  (EF-9.4; BOOST_LEVELS = 1)
//   so a top-down orthographic frame at the 2D view's scale has 2D's `L` at
//   every pixel (bar B1). The distance's height is the level-0 relief
//   (`ref_h`) rather than the drawn surface, so `L` is one function of position
//   that every node evaluates alike; the selection's bound uses the same.
// - Heights and colour are FUNCTIONS OF POSITION, the same in every node:
//   with j = round(L) (clamped to the node's level and two above it),
//   value = mix(level j-1, level j, smoothstep01(L - j + 0.5)), exactly the
//   2D tile blend (`lod_tile.gdshader`, `continuous_lod::level_weight`). Two
//   nodes that touch therefore compute the same height at the same point.
// - Moves odd vertices toward the parent grid as the node's own weight falls
//   (CDLOD's morph), over the first `morph_band` of a level's band, so the
//   edge facing a coarser neighbour sits on that neighbour's vertices (§3.3).
// - Skirts: skirt vertices drop by the node's skirt depth (the instance
//   basis's y scale), a safety net where a neighbour is two levels away or a
//   tile is late (§3.5).
// - Flattens below sea level to the sea surface (§3.6; the relief already
//   carries it, `metres()` clamps the rest) and draws the drape unlit (§3.4's
//   map shading; the lit drape is 3D-4).
//
// What it must never do: write DEPTH (early-z, §3.11), or read a tile layer
// it was not given (a missing parent level falls back to the node's own).
shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_opaque;

// Vertex grids (`WorldGen.view3d_take_ready`'s `geo`), one layer per tile,
// `grid_n x grid_n` RF, texel (i, j) exactly at patch vertex (i, j).
uniform sampler2DArray geo : filter_linear, repeat_disable;
// The 2D tiles themselves, RGBA8 with mipmaps, one layer per tile.
uniform sampler2DArray col : source_color, filter_linear_mipmap, repeat_disable;
// The level-0 relief, for the distance's height only.
uniform sampler2D ref_h : filter_linear, repeat_disable;
uniform vec2 ref_px = vec2(256.0);
uniform sampler2D boost_tex : filter_nearest, repeat_disable;
uniform int boost_n = 0;
uniform float boost_on = 0.0;
uniform float grid_n = 65.0;
uniform vec2 col_px = vec2(256.0);
// Index space [0, 1]^2 in metres, and the camera's position in it.
uniform vec2 world_m = vec2(1.0);
uniform vec2 cam_idx = vec2(0.0);
uniform float cam_y = 0.0;
uniform float cell_m = 1.0;
uniform float gw = 2.0;
uniform float f_px = 800.0;
uniform float ortho_ppc = 0.0;
uniform float ppc_entry = 0.0;
uniform float sea_level = 0.42;
uniform float peak_m = 4000.0;
uniform float exag = 1.0;
uniform float morph_band = 0.5;
// Drape mip bias (labelled judgement, `view3d_quadtree.gd::DRAPE_BIAS`).
uniform float drape_bias = -0.5;
// 0 off; 1 tints each node by its level (§5's node-level debug view); 2 writes
// the per-pixel level j as grey j / 16 (bar B1's level-index measurement); 3
// paints skirt fragments pure green (the probe's measure of how much skirt
// shows, which is how much the morph leaves uncovered).
uniform int debug_levels = 0;

varying vec3 v_rel;
varying vec2 v_u;
varying flat vec4 v_inst;
// 0 on the surface, rising to 1 down a skirt (debug mode 3 only reads it).
varying float v_skirt;

float metres(float h) {
	return max((h - sea_level) / max(1.0 - sea_level, 1e-6) * peak_m, 0.0) * exag;
}

// `BoostField::sample`, as `lod_tile.gdshader::boost_at`.
float boost_at(vec2 u) {
	if (boost_n <= 0) {
		return 0.0;
	}
	vec2 p = u * float(boost_n) - 0.5;
	vec2 i0 = floor(p);
	vec2 f = p - i0;
	ivec2 m = ivec2(boost_n - 1);
	ivec2 a = clamp(ivec2(i0), ivec2(0), m);
	ivec2 b = clamp(ivec2(i0) + ivec2(1), ivec2(0), m);
	float v00 = texelFetch(boost_tex, ivec2(a.x, a.y), 0).r;
	float v10 = texelFetch(boost_tex, ivec2(b.x, a.y), 0).r;
	float v01 = texelFetch(boost_tex, ivec2(a.x, b.y), 0).r;
	float v11 = texelFetch(boost_tex, ivec2(b.x, b.y), 0).r;
	return mix(mix(v00, v10, f.x), mix(v01, v11, f.x), f.y);
}

// `L_3d` at camera-relative ground point `rel_xz` (index-space `idx`).
float level_at(vec2 rel_xz, vec2 idx, out float yr) {
	yr = metres(texture(ref_h, (0.5 + clamp(idx, 0.0, 1.0) * (ref_px - 1.0)) / ref_px).r) - cam_y;
	float ppc = ortho_ppc;
	if (ppc <= 0.0) {
		ppc = cell_m * f_px / max(cell_m, length(vec3(rel_xz.x, yr, rel_xz.y)));
	}
	float lam = log2(max(1.0, gw * max(0.01, ppc) / 256.0));
	if (boost_on > 0.5 && ppc_entry > 0.0) {
		lam += smoothstep(0.0, 1.0, log2(ppc / ppc_entry)) * clamp(boost_at(idx), 0.0, 1.0);
	}
	return lam;
}

// Patch coordinate `u` of the node -> the coordinate in the tile `m` levels up
// (0 own, 1 parent, 2 grandparent), from the node's child-position bits.
vec2 up(vec2 u, int m, vec4 bits) {
	if (m >= 1) {
		u = (u + bits.xy) * 0.5;
	}
	if (m >= 2) {
		u = (u + bits.zw) * 0.5;
	}
	return u;
}

float layer_of(vec4 inst, int m) {
	return m == 0 ? inst.x : (m == 1 ? inst.y : inst.z);
}

vec4 bits_of(float pk) {
	float b = floor(pk / 16.0);
	return vec4(mod(b, 2.0), mod(floor(b / 2.0), 2.0), mod(floor(b / 4.0), 2.0), mod(floor(b / 8.0), 2.0));
}

float geo_at(vec2 u, int m, vec4 inst, vec4 bits) {
	vec2 t = up(u, m, bits);
	return textureLod(geo, vec3((0.5 + t * (grid_n - 1.0)) / grid_n, layer_of(inst, m)), 0.0).r;
}

// The level j to draw at `L` in a node of level k with `have` levels above it
// resident (0..2), and how far up j is (m = k - j).
int up_for(float L, float k, int have) {
	float j = clamp(floor(L + 0.5), max(k - float(have), 0.0), k);
	return int(k - j);
}

void vertex() {
	vec4 inst = INSTANCE_CUSTOM;
	float k = mod(inst.w, 16.0);
	vec4 bits = bits_of(inst.w);
	int have = inst.z >= 0.0 ? 2 : (inst.y >= 0.0 ? 1 : 0);
	vec2 p = VERTEX.xz;
	bool skirt = VERTEX.y < -0.5;
	float skirt_m = length(MODEL_MATRIX[1].xyz);
	// Camera-relative ground position: MODEL_MATRIX is the node's footprint
	// minus the camera, subtracted in 64-bit floats by the script (§3.11).
	vec2 rel = (MODEL_MATRIX * vec4(p.x, 0.0, p.y, 1.0)).xz;
	float yr;
	float L0 = level_at(rel, cam_idx + rel / world_m, yr);
	// CDLOD's morph: odd grid vertices slide to the lower even neighbour as
	// the node's own weight falls, so at weight 0 the patch is its parent's grid.
	float wk = smoothstep(0.0, 1.0, (L0 - k + 0.5) / morph_band);
	vec2 gi = floor(p * (grid_n - 1.0) + 0.5);
	vec2 odd = mod(gi, 2.0);
	p -= odd / (grid_n - 1.0) * (1.0 - wk);
	rel = (MODEL_MATRIX * vec4(p.x, 0.0, p.y, 1.0)).xz;
	vec2 idx = cam_idx + rel / world_m;
	float L = level_at(rel, idx, yr);
	int m = up_for(L, k, have);
	float j = k - float(m);
	float w = smoothstep(0.0, 1.0, L - j + 0.5);
	float hj = geo_at(p, m, inst, bits);
	float hp = (j > 0.0 && m < have) ? geo_at(p, m + 1, inst, bits) : hj;
	float y = metres(mix(hp, hj, w)) - cam_y;
	if (skirt) {
		y -= skirt_m;
	}
	v_rel = vec3(rel.x, yr, rel.y);
	v_skirt = skirt ? 1.0 : 0.0;
	v_u = p;
	v_inst = inst;
	VERTEX = vec3(p.x, y / max(skirt_m, 1e-6), p.y);
}

vec4 col_at(vec2 u, int m, vec4 inst, vec4 bits) {
	vec2 t = up(u, m, bits);
	vec3 uvl = vec3((0.5 + t * (col_px - 1.0)) / col_px, layer_of(inst, m));
	// Top-down draws the tiles as 2D does, unmipped (bar B1): with EF-9's
	// boost a pixel can draw a level whose texels are minified up to 2.8x, which
	// 2D's sprites sample at full resolution, and a mip there measured 4 % less
	// detail than 2D at x16 and x64. Perspective keeps the mips, so grazing
	// ground is filtered rather than sparkling.
	if (ortho_ppc > 0.0) {
		return textureLod(col, uvl, 0.0);
	}
	return texture(col, uvl, drape_bias);
}

void fragment() {
	vec4 inst = v_inst;
	float k = mod(inst.w, 16.0);
	vec4 bits = bits_of(inst.w);
	int have = inst.z >= 0.0 ? 2 : (inst.y >= 0.0 ? 1 : 0);
	vec2 idx = cam_idx + v_rel.xz / world_m;
	float yr;
	float L = level_at(v_rel.xz, idx, yr);
	int m = up_for(L, k, have);
	float j = k - float(m);
	float w = smoothstep(0.0, 1.0, L - j + 0.5);
	vec4 cj = col_at(v_u, m, inst, bits);
	vec4 cp = (j > 0.0 && m < have) ? col_at(v_u, m + 1, inst, bits) : cj;
	// The 2D river rule (`lod_tile.gdshader`): where this level's tile holds
	// river, the mix leans to it by its coverage.
	float ww = mix(w, 1.0, 1.0 - cj.a);
	vec3 c = mix(cp.rgb, cj.rgb, ww);
	if (debug_levels == 1) {
		vec3 hue = 0.5 + 0.5 * cos(6.2832 * (k / 7.0 + vec3(0.0, 0.33, 0.67)));
		vec2 e = min(v_u, 1.0 - v_u);
		c = mix(c, hue, min(e.x, e.y) < 0.01 ? 0.9 : 0.45);
	} else if (debug_levels == 2) {
		c = vec3((j + 0.5) / 16.0);
	} else if (debug_levels == 3 && v_skirt > 0.001) {
		c = vec3(0.0, 1.0, 0.0);
	}
	ALBEDO = c;
}
