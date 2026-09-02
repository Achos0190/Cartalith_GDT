shader_type canvas_item;

// Animated water (`water_anim_layer.gd`, `GUI_GAP_REGISTER.md` RN-01) --
// the reference's `waterAnimFrame` (reference HTML lines 8672-8690) done on
// the GPU instead of in a per-frame `ImageData` loop.
//
// ## What is the same, and what deliberately is not
//
// The *model* is the reference's, constant for constant: a two-stream
// crossfade (`flowMapPhases(t, 2.4)`, line 8326) advecting a value-noise
// field backwards along the flow direction at `SCALE = 6` cells per unit
// phase and `F = 0.22` cycles per cell, thresholded at `rip - 0.5`, scaled by
// the channel intensity, `* 240` and capped at alpha 150 over the colour
// (235, 245, 255).
//
// What differs is where it runs and which noise it runs on. The reference
// rebuilds a `GW x GH` RGBA buffer on the CPU every animation frame and
// `putImageData`s it -- at this port's 2048x1311 that is 2.7 million pixels
// of JavaScript per frame, which is exactly why the reference caps itself at
// `GW*GH <= 400000` (line 8670) and refuses to animate a full-size world at
// all. A fragment shader has no such ceiling, so **this port drops that cap**
// and animates at any resolution.
//
// The noise is a standard smoothstep-interpolated integer-hash value noise
// rather than `cartalith_noise::vnoise` bit-for-bit. `vnoise`'s hash is
// `>>>`-based 32-bit integer arithmetic, which GLSL ES cannot reproduce
// exactly across drivers, and this is a shimmer: `DECISIONS.md` §7a's
// principled-equivalence bar ("the same visual character, not the same
// bytes") is the one that applies, the same call `wind_fx_layer.gd` made
// when it replaced the reference's `destination-out` canvas fade.
//
// Nothing here reads a `wgpu` device or a `RenderingDevice`: this is an
// ordinary `ShaderMaterial` on an ordinary `ColorRect`, which is what keeps
// it clear of the GL-Compatibility/`wgpu` hazard class recorded in
// `cartalith-gpu/src/multi.rs`.

// `sample_bridge.rs`'s `waterfx` raster: R/G = unit downhill direction
// encoded as `v * 0.5 + 0.5`, A = channel intensity. NEAREST because R and G
// are a decoded vector, not a colour -- filtering them across a channel edge
// would smear the direction into the bank.
uniform sampler2D flow_tex : filter_nearest, repeat_disable;

// Grid size in cells, so the noise lattice is in *cell* space exactly as the
// reference's is (it samples `vnoise(x*F, y*F)` on raw grid coordinates).
uniform vec2 grid_size = vec2(1024.0, 1024.0);

// Seconds since the layer started. Fed from GDScript rather than read from
// `TIME` so pausing the effect actually freezes it instead of resuming
// somewhere else.
uniform float anim_time = 0.0;

// The reference's own constants (line 8683).
const float SCALE = 6.0;
const float F = 0.22;
const float PERIOD = 2.4;
const float MAX_ALPHA = 150.0 / 255.0;

float hash21(vec2 p) {
	// Deterministic, driver-independent: pure float arithmetic, no integer
	// shifts. The exact constants are the usual sin-fract pair; what matters
	// is that it is stable per lattice point, which is all value noise asks.
	return fract(sin(dot(floor(p), vec2(127.1, 311.7))) * 43758.5453123);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = p - i;
	// `vnoise`'s own interpolant (`xf*xf*(3-2*xf)`), matched exactly -- the
	// hash differs, the shaping does not.
	vec2 u = f * f * (3.0 - 2.0 * f);
	float a = hash21(i);
	float b = hash21(i + vec2(1.0, 0.0));
	float c = hash21(i + vec2(0.0, 1.0));
	float d = hash21(i + vec2(1.0, 1.0));
	return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

void fragment() {
	vec4 flow = texture(flow_tex, UV);
	float s = flow.a;
	if (s <= 0.02) {          // the reference's own skip threshold (line 8684)
		COLOR = vec4(0.0);
	} else {
		vec2 dir = flow.rg * 2.0 - 1.0;
		vec2 cell = UV * grid_size;

		// `flowMapPhases(t, 2.4)` (line 8326-8329), verbatim.
		float f0 = fract(anim_time / PERIOD);
		float f1 = fract(f0 + 0.5);
		float blend = abs(2.0 * f0 - 1.0);

		float rip = (1.0 - blend) * vnoise((cell - dir * f0 * SCALE) * F)
			+ blend * vnoise((cell - dir * f1 * SCALE) * F);

		float a = max(0.0, rip - 0.5) * s * (240.0 / 255.0);
		COLOR = vec4(235.0 / 255.0, 245.0 / 255.0, 255.0 / 255.0, min(MAX_ALPHA, a));
	}
}
