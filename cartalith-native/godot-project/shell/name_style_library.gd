extends RefCounted

## Ruling FZ (owner, 2026-11-17): the **per-device library of imported name styles**, the place every world draws from.
##
## One CSV per style in `user://name_styles/` (`style_<id>.csv`, the id being the engine's content hash of the style's
## canonical CSV, so importing the same file twice is one style). It is the same arrangement as the asset library's
## per-device store (`user://asset_library`, Ruling FR): the user's own, usable in every world, never inside a world file.
## **Each world saves a copy of the styles it uses** (`entities/name_styles.json`, engine side), so a world opens and
## names the same on a device that does not have the style.
##
## What this must never do: write a world, touch a built-in culture (they are read-only and live in the engine), or repair
## a CSV. A file that does not parse is skipped here and listed by `Records ▸ Name styles…` with the engine's own row
## errors.
##
## Preloaded (`const NameStyleLibrary := preload(...)`) rather than a `class_name`, so adding it needs no
## `--headless --import` pass (CLAUDE.md: that pass strips `project.godot`'s `;` comments).

const DIR := "user://name_styles"

## The folder, created on demand.
static func ensure_dir() -> void:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(DIR))

## `[{key, id, name, csv}]` for every library file that parses (a style with row errors is kept: the canonical CSV the
## engine returned has none). Sorted by name, then id, so the list is stable. Registers each with the engine so a roster
## or a preview can name it by key.
static func entries(bridge) -> Array:
	var out: Array = []
	if bridge == null:
		return out
	ensure_dir()
	var d := DirAccess.open(DIR)
	if d == null:
		return out
	for f in d.get_files():
		if not f.begins_with("style_") or not f.ends_with(".csv"):
			continue
		var text := FileAccess.get_file_as_string(DIR.path_join(f))
		var r: Dictionary = bridge.ns_parse(text, true)
		if not bool(r.get("ok", false)):
			continue
		out.append({"key": String(r["key"]), "id": String(r["key"]).trim_prefix("style:"),
			"name": String(r["name"]), "csv": String(r["csv"])})
	out.sort_custom(func(a, b): return [String(a["name"]).to_lower(), a["id"]] < [String(b["name"]).to_lower(), b["id"]])
	return out

## Adds a parsed style (`bridge.ns_parse(csv, true)` result with `ok`) to the library as its canonical CSV. Returns the key.
static func add(parsed: Dictionary) -> String:
	ensure_dir()
	var key := String(parsed["key"])
	var f := FileAccess.open(DIR.path_join("style_%s.csv" % key.trim_prefix("style:")), FileAccess.WRITE)
	if f != null:
		f.store_string(String(parsed["csv"]))
		f.close()
	return key

## Removes a style's file. `false` when there was none.
static func remove(key: String) -> bool:
	var p := DIR.path_join("style_%s.csv" % key.trim_prefix("style:"))
	if not FileAccess.file_exists(p):
		return false
	return DirAccess.remove_absolute(ProjectSettings.globalize_path(p)) == OK

## The CSV text with its `Name style,<name>` row renamed (a duplicate gets its own name; the id follows the content).
static func with_name(csv: String, new_name: String) -> String:
	var lines := csv.split("\n")
	for i in lines.size():
		if lines[i].to_lower().begins_with("name style,"):
			var n := new_name.replace("\"", "\"\"")
			lines[i] = "Name style,\"%s\"" % n if (new_name.contains(",") or new_name.contains("\"")) else "Name style,%s" % new_name
			break
	return "\n".join(lines)

## Registers every library style with the engine (so the roster can hold one) and returns the entries.
static func register_all(bridge) -> Array:
	return entries(bridge)
