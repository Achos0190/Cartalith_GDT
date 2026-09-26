extends Node
## Committed verification harness for `OUTSTANDING_WORK.md`'s "Record
## approved Vault Browser mockup + close its remaining gaps" row.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultbrowsegaps_probe.tscn
##
## Drives the real app, the real `VaultWindow.open_browse()` two-pane
## browser, against a real folder of real Markdown files and a real
## generated world, and asserts the gaps the row named as closed this batch.
## Kept as a separate file from the pre-existing `_vaultbrowse_probe.gd`
## (which this file's own author briefly and wrongly overwrote before
## restoring it from `git show HEAD:...` -- that probe covers the raw-text
## preview+edit panel and its hash guard, a different slice of the same
## window; this one is scoped to the mockup-gap row instead of duplicating
## or replacing it):
##
##   1. **Path-keyed backlinks/unlinked mentions** (`vault_file_backlinks`,
##      `vault_file_mentions`) — a positive control (a file with one real
##      backlink and one real unlinked mention) against a negative control
##      (an unrelated file with neither), so the count is proven to come from
##      the fixture and not from a constant that always reads the same.
##   2. **"Centre on map"** — enabled for a settlement note whose frontmatter
##      names a real `tid` in the generated world, and disabled with a stated
##      reason for a note whose frontmatter names no settlement.
##   3. **The excerpt's ATX heading-marker strip** — a note whose first line
##      is a heading must show the bare text, never the raw `#` marker.
##   4. **"Open to edit" as the browse-mode primary button** — distinct from
##      `_build_attach`'s own "Preview & edit this note…", which is unchanged.
##
## Committed like every probe in this folder (`STATUS.md`'s F8 row).

const SEED := 483920

var _app: Node
var _bridge
var _root := ""
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VAULTBROWSEGAPS  OK  %s" % label)
	else:
		_fails.append(label)
		print("VAULTBROWSEGAPS  !!  %s   %s" % [label, detail])


func _write(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _texts(n: Node, out: Array) -> Array:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children():
		_texts(c, out)
	return out


func _find(n: Node, cls) -> Node:
	if is_instance_of(n, cls):
		return n
	for c in n.get_children():
		var r := _find(c, cls)
		if r != null:
			return r
	return null


## Every `Button` in the tree, so the probe can find "Centre on map" itself
## (not just its text) and read `.disabled`/`.tooltip_text` off the real node.
func _buttons(n: Node, out: Array) -> Array:
	if n is Button:
		out.append(n)
	for c in n.get_children():
		_buttons(c, out)
	return out


func _find_button(root: Node, text: String) -> Button:
	for b in _buttons(root, []):
		if String((b as Button).text) == text:
			return b as Button
	return null


func _generate() -> void:
	_bridge.generate({
		"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	await get_tree().process_frame


func _ready() -> void:
	## A clean device profile: an earlier probe run in this same Godot user
	## profile (`_vault_probe.gd`, `_vaultunlink_probe.gd`) can leave a bound
	## vault and a built index in `user://markdown_vault.json`, which the app
	## restores on boot -- by design (`_vaultunlink_probe.gd`'s own §1). That
	## is a different vault than this probe's fixture, so start from nothing.
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(0.8).timeout
	_bridge = _app.bridge
	await _generate()

	var settlements: Array = _bridge.settlements()
	_ok("world: settlements exist to centre on", settlements.size() > 0)
	if settlements.is_empty():
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))

	# -- a real vault on disk, with a real backlink and a real unlinked mention
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultbrowsegaps-probe"
	DirAccess.make_dir_recursive_absolute(_root + "/Settlements")
	DirAccess.make_dir_recursive_absolute(_root + "/Events")
	_write(_root + "/Settlements/Target.md", "---\ntype: settlement\ntid: %d\n---\n\n# Target\n\nDownstream barge terminus.\n" % tid)
	_write(_root + "/Chronicle.md", "The lords of [[Target]] held the ford.\n")
	_write(_root + "/Journal.md", "Rode through Target before the thaw and slept badly.\n")
	_write(_root + "/Events/Unrelated.md", "---\ntype: event\nyear: 12\n---\n\nNothing to do with any of it.\n")

	var conn: Dictionary = _bridge.vault_connect(_root, "BrowseProbeVault")
	_ok("connect: a real folder binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	## `_vaultunlink_probe.gd` already proves the index survives a relaunch by
	## design (its own §1/§2), which means a fresh app boot in this same
	## user profile can restore an *earlier probe's* index for an *earlier
	## probe's* vault. Rebuilding throws that away deterministically before
	## this probe asserts anything about "no index yet", rather than this
	## probe racing whatever a previous run left in `user://markdown_vault.json`.
	_bridge.vault_rebuild_backlinks()
	var pre_stats: Dictionary = _bridge.vault_backlink_stats()
	_ok("index: not built right after a rebuild, never a false zero",
		not bool(pre_stats.get("built", false)), str(pre_stats))
	_ok("backlinks: empty with no index (path-keyed, no entity needed)",
		_bridge.vault_file_backlinks("Settlements/Target.md").is_empty())

	var refreshed: Dictionary = _bridge.vault_refresh_backlinks(500)
	_ok("index: refresh succeeds", bool(refreshed.get("ok", false)), String(refreshed.get("error", "")))

	# -- the engine call, directly: positive control vs. negative control ----
	var back: Array = _bridge.vault_file_backlinks("Settlements/Target.md")
	_ok("backlinks: exactly one real backlink found", back.size() == 1, str(back))
	if back.size() == 1:
		var bd: Dictionary = back[0]
		_ok("backlinks: it is Chronicle.md, by wikilink", String(bd.get("rel", "")) == "Chronicle.md"
			and String(bd.get("form", "")) == "wiki", str(bd))
	var mentions: Array = _bridge.vault_file_mentions("Settlements/Target.md", 8)
	_ok("mentions: exactly one unlinked mention found", mentions.size() == 1, str(mentions))
	if mentions.size() == 1:
		_ok("mentions: it is Journal.md", String((mentions[0] as Dictionary).get("rel", "")) == "Journal.md", str(mentions))

	## Negative control: the unrelated event note has neither a backlink nor
	## a mention. Proves the count above is measuring the fixture, not
	## returning a constant that reads the same for every file.
	var no_back: Array = _bridge.vault_file_backlinks("Events/Unrelated.md")
	var no_mentions: Array = _bridge.vault_file_mentions("Events/Unrelated.md", 8)
	_ok("negative control: the unrelated note has no backlinks", no_back.is_empty(), str(no_back))
	_ok("negative control: the unrelated note has no mentions", no_mentions.is_empty(), str(no_mentions))

	# -- the same, read off the real drawn panel, not just the bridge --------
	_app.open_vault_browse()
	await get_tree().process_frame
	var vw := _find(_app, VaultWindow)
	vw._pick_file = "Settlements/Target.md"
	vw._rebuild()
	await get_tree().process_frame
	var vw_text := "\n".join(_texts(vw, []))
	_ok("panel: the backlink/mention line is drawn with the real counts",
		vw_text.find("1 backlink") >= 0 and vw_text.find("1 unlinked mention") >= 0, vw_text)
	_ok("panel: the backlink source is named", vw_text.find("Chronicle.md") >= 0, vw_text)
	_ok("panel: the mention source is named", vw_text.find("Journal.md") >= 0, vw_text)

	## Target.md's own first line is a heading ("# Target"), so its excerpt
	## exercises the ATX-marker strip directly: the drawn text must carry the
	## bare word, never the raw "# " marker (`OUTSTANDING_WORK.md`'s "the
	## excerpt shows raw `## ` markers" gap).
	_ok("panel: the excerpt strips the ATX heading marker", vw_text.find("# Target") < 0
		and vw_text.find("Target") >= 0, vw_text)

	## "Open to edit" is the mockup's own primary button, replacing "Preview
	## & edit this note…" in browse mode only (`_build_attach`'s own call
	## keeps the quieter text button, unchanged -- see `_vaultbrowse_probe.gd`
	## for that regression).
	var open_btn := _find_button(vw, "Open to edit")
	_ok("panel: the browse preview draws \"Open to edit\", not the Attach-mode label",
		open_btn != null and _find_button(vw, "Preview & edit this note…") == null)

	## Positive control: "Centre on map" enabled for a settlement note whose
	## frontmatter names a real tid.
	var centre := _find_button(vw, "Centre on map")
	_ok("panel: Centre on map is drawn", centre != null)
	if centre != null:
		_ok("panel: Centre on map is enabled for a real settlement's note", not centre.disabled, centre.tooltip_text)
		centre.pressed.emit()
		await get_tree().process_frame
		_ok("panel: pressing it does not crash the shell", is_instance_valid(vw))

	## Negative control: the same button, disabled with a stated reason, for
	## a note whose frontmatter names no settlement.
	vw._pick_file = "Events/Unrelated.md"
	vw._browse_path = ""
	vw._rebuild()
	await get_tree().process_frame
	var centre2 := _find_button(vw, "Centre on map")
	_ok("panel: Centre on map is disabled for a non-settlement note", centre2 != null and centre2.disabled,
		"" if centre2 == null else str(centre2.disabled))
	_ok("panel: the disabled reason names the actual frontmatter type", centre2 != null
		and centre2.tooltip_text.find("event") >= 0, "" if centre2 == null else centre2.tooltip_text)

	_finish()


func _finish() -> void:
	if _fails.is_empty():
		print("VAULTBROWSEGAPS  ALL CHECKS PASSED")
	else:
		print("VAULTBROWSEGAPS  %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
