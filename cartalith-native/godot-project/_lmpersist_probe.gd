extends SceneTree

## Owner ruling 10 (2026-09-06) — landmarks persist in `entities/landmarks.json`.
##
##   Godot_v4.7.1-stable_win64_console --headless --path . \
##       --script res://_lmpersist_probe.gd
##
## Headless and `extends SceneTree`, the shape `_savetree_probe.gd` already
## uses for this exact question: nothing here rasterises, and the thing under
## test is `project_save` / `project_open`, not a drawn panel. `WorldGen.new()`
## rather than a shell boot for the same reason — the shell adds a renderer
## dependency and no coverage.
##
## What the Rust unit tests cannot reach, and this does: the `#[func]` path.
## `project_open` restores `landmark_store.last` *after* `load_save` has
## invalidated it, using `self.seed` and `self.gw` that `load_save` set a few
## dozen lines earlier — an ordering nothing in `cargo test` exercises,
## because `WorldGen` is a cdylib `GodotClass` and cannot be constructed there.

func _init() -> void:
	var fails := 0

	var wg: WorldGen = WorldGen.new()
	for m in ["landmark_run", "landmarks", "landmark_funnels", "landmark_last_run",
			"project_save", "project_open"]:
		if not wg.has_method(m):
			print("FAIL: %s absent — the probe cannot run" % m)
			quit(1)
			return

	wg.generate_sized(24601, 640.0, 192, 144)
	if not wg.landmark_run():
		print("FAIL: landmark_run refused -> ", wg.landmark_last_run().get("error", "?"))
		quit(1)
		return

	var before: Array = wg.landmarks()
	var funnels_before: Array = wg.landmark_funnels()
	var run_before: Dictionary = wg.landmark_last_run()
	print("  placed before save: ", before.size(), "  funnels: ", funnels_before.size())
	if before.size() < 20:
		print("  FAIL: only %d placed; too thin to prove anything" % before.size())
		fails += 1

	## The identity under test, composed exactly as `Landmark::key()` does.
	var keys_before := {}
	for d: Dictionary in before:
		keys_before["%s@%d,%d" % [d["kind"], int(d["x"]), int(d["y"])]] = d
	if keys_before.size() != before.size():
		print("  FAIL: kind@x,y is not unique across %d placements (%d distinct)"
			% [before.size(), keys_before.size()])
		fails += 1

	var path := OS.get_user_data_dir().path_join("_lmpersist_probe.zip")
	var r: Dictionary = wg.project_save(path)
	if not bool(r.get("ok", false)):
		print("  FAIL: save -> ", r.get("error", "?"))
		quit(1)
		return

	## The slot is written, and it is written into the tree layout.
	var zr := ZIPReader.new()
	if zr.open(path) != OK:
		print("  FAIL: archive did not open")
		quit(1)
		return
	var names: PackedStringArray = zr.get_files()
	var doc := ""
	if "entities/landmarks.json" in names:
		doc = zr.read_file("entities/landmarks.json").get_string_from_utf8()
	zr.close()
	if doc.is_empty():
		print("  FAIL: entities/landmarks.json is absent or empty")
		quit(1)
		return
	var parsed: Variant = JSON.parse_string(doc)
	if typeof(parsed) != TYPE_DICTIONARY:
		print("  FAIL: entities/landmarks.json is not an object")
		quit(1)
		return
	var obj: Dictionary = parsed
	print("  document: ", doc.length(), " bytes, members ", obj.keys())
	if not obj.has("results"):
		print("  FAIL: no `results` member — the run was not written")
		fails += 1
	elif int((obj["results"] as Dictionary).get("landmarks", []).size()) != before.size():
		print("  FAIL: document carries %d placements, the run had %d"
			% [int((obj["results"] as Dictionary)["landmarks"].size()), before.size()])
		fails += 1
	## §14.1 and `LandmarkDto`'s own list of what is deliberately absent.
	for absent in ["\"seed\"", "\"class\"", "\"id\""]:
		if absent in doc:
			print("  FAIL: %s reached the document; it is derived, not stored" % absent)
			fails += 1

	## The reopen, in a fresh engine object — the state under test is what
	## `project_open` puts back, not what this one still happens to hold.
	var wg2: WorldGen = WorldGen.new()
	var o: Dictionary = wg2.project_open(path)
	if not bool(o.get("ok", false)):
		print("  FAIL: open -> ", o.get("error", "?"))
		quit(1)
		return
	var restored: Array = o.get("restored", [])
	print("  restored=", restored)
	if not ("landmarks" in restored):
		print("  FAIL: `landmarks` is not in `restored`")
		fails += 1

	var after: Array = wg2.landmarks()
	print("  placed after reopen: ", after.size())
	if after.size() != before.size():
		print("  FAIL: %d placements went in, %d came back" % [before.size(), after.size()])
		fails += 1

	## Every key survives, and each row's non-float members are unchanged.
	var missing := 0
	var changed := 0
	for i in range(after.size()):
		var d: Dictionary = after[i]
		var key := "%s@%d,%d" % [d["kind"], int(d["x"]), int(d["y"])]
		if not keys_before.has(key):
			missing += 1
			continue
		var was: Dictionary = keys_before[key]
		if d["causal"] != was["causal"]:
			changed += 1
		if int(d["id"]) != i + 1:
			print("  FAIL: row %d has id %d — the id is a position and is re-derived" % [i, int(d["id"])])
			fails += 1
	if missing > 0:
		print("  FAIL: %d restored landmarks carry a key the saved run did not" % missing)
		fails += 1
	if changed > 0:
		print("  FAIL: %d causal chains changed across the round trip" % changed)
		fails += 1

	## The funnels come back in `kinds()` order with their counters, so the
	## dock's "why fewer than I asked for" line is as real as it was.
	var funnels_after: Array = wg2.landmark_funnels()
	if funnels_after.size() != funnels_before.size():
		print("  FAIL: %d funnels went in, %d came back"
			% [funnels_before.size(), funnels_after.size()])
		fails += 1
	else:
		var moved := 0
		var unrecorded := 0
		for i in range(funnels_after.size()):
			var a: Dictionary = funnels_after[i]
			var b: Dictionary = funnels_before[i]
			for k in ["kind", "candidates", "rejected_constraint", "rejected_score",
					"rejected_spacing", "rejected_cap", "cap", "placed", "limit"]:
				if a.get(k) != b.get(k):
					moved += 1
					if moved <= 3:
						print("    funnel[%d] %s: %s -> %s" % [i, k, b.get(k), a.get(k)])
			if String(a.get("limit", "")) == "unrecorded":
				unrecorded += 1
		if moved > 0:
			print("  FAIL: %d funnel members changed across the round trip" % moved)
			fails += 1
		if unrecorded > 0:
			print("  FAIL: %d funnels came back `unrecorded` from a measured run" % unrecorded)
			fails += 1

	## `landmark_last_run()` reports a real run for a reopened project. Its
	## own `else` branch returns `ok=false` with "No landmark pass has run for
	## this world." whenever `landmark_store.last` is `None`, and `load_save`
	## clears it on every open — so without the restore this is the answer the
	## code gives, by construction rather than by measurement.
	var run_after: Dictionary = wg2.landmark_last_run()
	print("  last_run after reopen: ok=", run_after.get("ok"), " placed=", run_after.get("placed"))
	if not bool(run_after.get("ok", false)):
		print("  FAIL: last_run says the pass has not run -> ", run_after.get("error", "?"))
		fails += 1
	if int(run_after.get("placed", -1)) != int(run_before.get("placed", -2)):
		print("  FAIL: placed %s -> %s" % [run_before.get("placed"), run_after.get("placed")])
		fails += 1

	## The rejects are deliberately not carried, and that is stated rather
	## than assumed: an empty list here must coexist with funnels that still
	## report real rejections, or the omission would be silently lossy.
	if wg2.has_method("landmark_rejects"):
		var rej: Array = wg2.landmark_rejects()
		var total_rejected := 0
		for f: Dictionary in funnels_after:
			total_rejected += int(f.get("rejected_constraint", 0)) + int(f.get("rejected_score", 0)) \
				+ int(f.get("rejected_spacing", 0)) + int(f.get("rejected_cap", 0))
		print("  rejects after reopen: ", rej.size(), " (funnels still report ", total_rejected, ")")
		if not rej.is_empty():
			print("  FAIL: the reject list is not written, so it must come back empty")
			fails += 1
		if total_rejected == 0:
			print("  FAIL: the funnels report no rejections, so the omission is untested here")
			fails += 1

	## Save the reopened project again: a second cycle must not lose the run.
	var path2 := OS.get_user_data_dir().path_join("_lmpersist_probe2.zip")
	var r2: Dictionary = wg2.project_save(path2)
	if not bool(r2.get("ok", false)):
		print("  FAIL: second save -> ", r2.get("error", "?"))
		fails += 1
	else:
		var wg3: WorldGen = WorldGen.new()
		var o3: Dictionary = wg3.project_open(path2)
		var third: Array = wg3.landmarks() if bool(o3.get("ok", false)) else []
		print("  placed after a second cycle: ", third.size())
		if third.size() != before.size():
			print("  FAIL: the second save/open cycle lost placements (%d -> %d)"
				% [before.size(), third.size()])
			fails += 1

	if fails == 0:
		print("PASS: %d landmarks and %d funnels survive save, reopen and a second cycle"
			% [before.size(), funnels_after.size()])
	else:
		print("FAIL: %d checks failed" % fails)
	quit(1 if fails > 0 else 0)
