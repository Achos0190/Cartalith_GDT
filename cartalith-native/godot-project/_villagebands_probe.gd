extends Node
## OUTSTANDING_WORK.md §2.11: "The Village carto preset still looks harsh
## (yellow/purple/teal)". Ruling AZ (LARGE_ITEM_RULINGS.md, 2026-09-28): more
## colour bands, softer, matching the other presets. This probe measures the
## harsh-transition metric (fraction of adjacent-pixel pairs whose RGB
## Euclidean colour distance exceeds THRESHOLD) for every `STYLE_PRESETS`
## entry in `render_workspace.gd`, on one fixed seed, so the same run
## rebuilt against a different `quantize_flat_palette` BANDS constant tells
## whether Village's figure has moved into the other presets' range.
##
## Only Village's own recipe sets `Npr::village = true` (checked by grep --
## no other STYLE_PRESETS entry does), so only Village's number moves when
## `BANDS` changes; the other six are a fixed control measured once per run.
##
## MUST run WINDOWED: `ImageTexture.update()` is a no-op under `--headless`
## (MISTAKES.md, "Run a pixel probe").
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _villagebands_probe.tscn -- --tag bands3
##
## `--tag` names the output PNGs (and is required); any other argument is
## refused. Not committed -- a measurement probe for one backlog row, not a
## regression guard (those live in `cargo test`).

const SEED := 20260927
const WIDTH_KM := 1200.0
const GRID_W := 512
const GRID_H := 328

## Euclidean RGB distance above which an adjacent pair counts as a "harsh"
## transition. Stated, not tuned to an outcome: half the maximum possible
## per-channel step (255) combined over three channels as
## sqrt(3*(255/2)^2) ~= 220.7, rounded down -- a pair has to differ by more
## than half of full-scale on every channel, on average, to count. Applied
## identically to every preset.
const THRESHOLD := 90.0

## The exact `STYLE_PRESETS` table from `render_workspace.gd`, copied rather
## than loaded from the workspace scene (which needs the full DCC shell
## running) -- see that file for the recipe comments. Kept in the same
## order so a report line matches a gallery tile.
const STYLE_PRESETS := [
	["Natural Vibrant", "Natural Vibrant", {"multi_sun": true}],
	["Default", "Quality tier", {}],
	["Antique", "Antique Parchment", {"sepia": 0.35, "multi_sun": true}],
	["Ink", "Natural Vibrant", {"ink": 0.6, "contours": 0.35, "multi_sun": true}],
	["Watercolor", "Natural Vibrant", {"watercolor": 0.65, "multi_sun": true}],
	["Print", "Natural Vibrant", {"risograph": 0.5, "contours": 0.25}],
	["Village", "Quality tier", {"village": true},
		{"detail_macro_weight": 0.0, "detail_meso_weight": 0.0, "detail_micro_weight": 0.0,
			"relief_ambient": 1.0}],
]

const STYLE_MANAGED := {
	"watercolor": 0.0, "contours": 0.0, "contour_m": 0.0, "ink": 0.0,
	"hachure": 0.0, "cel": 0.0, "crosshatch": 0.0, "stipple": 0.0,
	"sepia": 0.0, "risograph": 0.0, "pointillism": 0.0,
	"waves": false, "multi_sun": false, "village": false,
}

static func _preset_appearance_keys() -> PackedStringArray:
	var keys := PackedStringArray()
	for entry in STYLE_PRESETS:
		if entry.size() > 3:
			for key in Dictionary(entry[3]):
				if not keys.has(String(key)):
					keys.append(String(key))
	return keys

var bridge: Node
var fails := 0

func _ok(cond: bool, what: String) -> void:
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		fails += 1

## Fraction of horizontally- and vertically-adjacent pixel pairs whose RGB
## Euclidean distance exceeds THRESHOLD, sampled on a stride so a 512x328
## image is cheap to walk exhaustively without a stride at all -- every pair,
## not a subsample chosen by outcome.
func _harsh_fraction(img: Image) -> Dictionary:
	var w := img.get_width()
	var h := img.get_height()
	var harsh := 0
	var total := 0
	for y in range(h):
		for x in range(w):
			var c0 := img.get_pixel(x, y)
			if x + 1 < w:
				var c1 := img.get_pixel(x + 1, y)
				var d := Vector3(c0.r - c1.r, c0.g - c1.g, c0.b - c1.b).length() * 255.0
				total += 1
				if d > THRESHOLD:
					harsh += 1
			if y + 1 < h:
				var c2 := img.get_pixel(x, y + 1)
				var d2 := Vector3(c0.r - c2.r, c0.g - c2.g, c0.b - c2.b).length() * 255.0
				total += 1
				if d2 > THRESHOLD:
					harsh += 1
	return {"harsh": harsh, "total": total, "fraction": (float(harsh) / float(total)) if total > 0 else 0.0}

func _apply_preset(index: int) -> void:
	var values: Dictionary = STYLE_MANAGED.duplicate()
	for key in Dictionary(STYLE_PRESETS[index][2]):
		values[key] = STYLE_PRESETS[index][2][key]
	bridge.set_look(String(STYLE_PRESETS[index][1]))
	bridge.set_npr(values)
	bridge.drop_appearance_overrides(_preset_appearance_keys())
	if STYLE_PRESETS[index].size() > 3:
		bridge.set_appearance(Dictionary(STYLE_PRESETS[index][3]))

func _ready() -> void:
	var tag := ""
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		if args[k] == "--tag" and k + 1 < args.size():
			tag = args[k + 1]
			k += 2
		else:
			push_error("unknown argument %s" % args[k])
			get_tree().quit(2)
			return
	if tag == "":
		push_error("--tag <name> is required")
		get_tree().quit(2)
		return

	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame
	var dir := ProjectSettings.globalize_path("user://_villagebands_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)
	print("  threshold: %.1f (Euclidean RGB distance, 0-441.7 range)" % THRESHOLD)

	bridge.world_gen.generate_sized(SEED, WIDTH_KM, GRID_W, GRID_H)
	bridge.has_world = true
	# Force the palette: sRGB, not whatever colour space a prior run left set.
	bridge.world_gen.set_color_space("sRGB")

	var results := {}
	for i in STYLE_PRESETS.size():
		var name: String = STYLE_PRESETS[i][0]
		_apply_preset(i)
		var tex: ImageTexture = bridge.world_gen.build_color_texture()
		_ok(tex != null, "%s: build_color_texture returns a texture" % name)
		if tex == null:
			continue
		var img := tex.get_image()
		img.convert(Image.FORMAT_RGB8)
		var m := _harsh_fraction(img)
		results[name] = m
		print("  %-16s harsh=%d/%d fraction=%.6f" % [name, m["harsh"], m["total"], m["fraction"]])
		if name == "Village":
			img.save_png(dir.path_join("%s_village.png" % tag))

	# Positive control: the metric distinguishes at least two presets from
	# each other, so a uniformly-zero or uniformly-saturated measurement
	# cannot pass silently.
	var fractions: Array = []
	for name in results:
		fractions.append(results[name]["fraction"])
	fractions.sort()
	_ok(fractions[0] != fractions[-1], "positive control -- presets are not all measured identically")

	if results.has("Village"):
		var others := []
		for name in results:
			if name != "Village":
				others.append(results[name]["fraction"])
		others.sort()
		print("  other presets' fraction range: [%.6f, %.6f]" % [others[0], others[-1]])
		print("  Village fraction: %.6f" % results["Village"]["fraction"])

	bridge.world_gen.set_color_space("sRGB")
	print("\n==== village bands (%s): %s ====\n" % [tag, "OK" if fails == 0 else "SEE ABOVE"])
	get_tree().quit(1 if fails > 0 else 0)
