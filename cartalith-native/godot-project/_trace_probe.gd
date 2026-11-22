extends Node
## TOOLING_PLAN.md M1 (TR-3/TR-4): switches the trace on through the real extension class, generates a
## world through `WorldGen.generate_sized`, sends a shell event, and checks the log file holds the ten
## stage spans under one `gen::run`, the bridge span, the shell event, and none of a planted note.
## Args after `--`: `--size WxH` (default 1024x512), `--mutate off` (never starts the trace; must fail).
## Exit 0 held, 1 a check failed.

const NOTE := "The queen hides the silver sword beneath the chapel"

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var mutate := "off" in args
	var w := 1024
	var h := 512
	var i := args.find("--size")
	if i >= 0 and i + 1 < args.size():
		var p := args[i + 1].split("x")
		w = int(p[0])
		h = int(p[1])
	var dir := ProjectSettings.globalize_path("user://logs")
	var path := ""
	if not mutate:
		path = Trace.start("info", dir)
	var fails: Array[String] = []
	if not mutate and path == "":
		fails.append("Trace.start refused")
	var wg := WorldGen.new()
	var t0 := Time.get_ticks_msec()
	wg.generate_sized(12345, 200.0, w, h)
	print("generate_sized ", w, "x", h, " took ", Time.get_ticks_msec() - t0, " ms")
	Trace.event("probe.generated", float(Time.get_ticks_msec() - t0), 1)
	if Trace.event(NOTE, 0.0, 0):
		fails.append("a sentence was accepted as an op name")
	Trace.flush()
	Trace.stop()
	var text := FileAccess.get_file_as_string(path) if path != "" else ""
	print("trace file: ", path, " bytes=", text.length())
	for n in ["gen::run", "gen::planet", "gen::tectonics", "gen::volcanism", "gen::erosion", "gen::hydrology", "gen::climate", "gen::ecology_biomes", "gen::resources_soils", "bridge::generate_sized"]:
		if not text.contains("\"name\":\"" + n + "\""):
			fails.append("missing span " + n)
	if not text.contains("\"op\":\"probe.generated\""):
		fails.append("missing shell event")
	if text.contains("queen") or text.contains("sword"):
		fails.append("note text reached the log")
	for f in fails:
		printerr("FAIL: ", f)
	print("PASS" if fails.is_empty() else "FAILED")
	get_tree().quit(0 if fails.is_empty() else 1)
