extends RefCounted
class_name DiagnosticReviewDialog

## The review panel in front of `Help ▸ Save diagnostic report`.
##
## **Why a panel exists at all, and why it is not the button the canvas drew.**
## `LARGE_ITEM_RULINGS.md`'s "Build" ruling on `Report an issue` is *"Replace
## with a local diagnostic dump ... **No endpoint required.**"* The round-3
## canvas drew an `OPEN TRACKER` button that opens a browser, which contradicts
## the ruling and is not built here -- an owner decision is newer than any
## canvas. What the canvas got right is everything else on that artboard: a
## pre-filled report is a **claim about what the app knows**, so the user sees
## exactly what is about to be written before it is written, and the canvas's
## own note ("COPY DETAILS is the row that earns its place: it works with no
## network") is why `Copy` sits beside `Save` rather than being an afterthought.
##
## Nothing here opens a URL and nothing is posted. The user attaches the file.
##
## **The table is `DiagnosticReport.manifest()` drawn directly.** It is not a
## description of the report -- it is the array `DiagnosticReport.build_text()`
## writes, so a row cannot appear here and be missing from the file, and the
## file's own `== WHAT THIS FILE CONTAINS ==` block is the same list again.
## `_diagreview_probe.gd` asserts that correspondence label by label.
##
## Vocabulary: `DccWidgets.modal_card()` with `modal_inset()`/`modal_stat()`
## rows, which is the shell's existing composition for "a block of KEY .....
## value lines the user is being asked to confirm", and `modal_stat_absent()`
## for a value this build cannot supply -- that helper's whole reason for
## existing is that a plausible figure in one of these slots is worse than a
## dash, which is exactly this panel's problem.

## Wider than `DccWidgets.MODAL_W` (340), because each row carries three
## things: a label, the value's first line, and the symbol the value was read
## from. **Measured at this width rather than argued for:**
## `_diagreview_probe.gd` walks the drawn `from …` labels and reports the worst
## `get_line_count()`; at 520, 1600x1000 pointer, it is **1** -- every source
## string, including the longest (`Engine.get_version_info() · OS.get_name() ·
## Time.get_datetime_string_from_system()`), sits on one line. The probe fails
## the run if any of them ever wraps past two. No claim is made about 340: it
## was not measured, and the number that matters is the one this ships at.
const CARD_W := 520

## Two lines per row, not three columns: `modal_stat()` already owns the
## `KEY ..... value` shape the whole shell uses for a block of figures being
## confirmed, and the source goes on its own line underneath rather than
## competing with the value for the same horizontal space. The value is what
## the user is reviewing; the source is what lets them check it.
static func open(app: Node, bridge: EngineBridge) -> void:
	var opts := DiagnosticReport.default_options()
	var card := DccWidgets.modal_card(app, "Save diagnostic report",
		DccWidgets.MODAL_CONFIRM, CARD_W)
	var body: VBoxContainer = card["body"]

	DccWidgets.modal_prose(body,
		"This writes one text file to disk and opens the folder it is in. "
		+ "Nothing is sent anywhere and no tracker is opened — attach the file "
		+ "to a report yourself. Everything it will contain is listed below.")

	## The two toggles the canvas draws, and it is right about both defaults.
	## Seed and parameters on, because a world is reproducible from its seed and
	## a report without them can rarely be acted on; project path off, because
	## it is a filesystem path and on Windows it embeds the account name. Both
	## rebuild the table rather than annotating it, so the panel always shows
	## the report as it stands right now.
	var table := VBoxContainer.new()
	table.add_theme_constant_override("separation", 0)

	DccWidgets.toggle(body, "Include the seed and generation parameters", true,
		func(v: bool):
			opts[DiagnosticReport.OPT_SEED_PARAMS] = v
			_fill(table, app, bridge, opts),
		"On by default: a world is reproducible from its seed, so without this the report cannot say which world went wrong.")
	DccWidgets.toggle(body, "Include the project file path", false,
		func(v: bool):
			opts[DiagnosticReport.OPT_PROJECT_PATH] = v
			_fill(table, app, bridge, opts),
		"Off by default: it is a filesystem path, and on Windows it normally embeds your account name.")

	body.add_child(table)
	_fill(table, app, bridge, opts)

	## **What the toggles do not guarantee.** Turning a row off drops its
	## section; it does not scrub the string from the file, because the log tail
	## is verbatim application output and nothing filters it. Measured, not
	## supposed: `_diagreview_probe.gd` turned the seed row off and found the
	## seed still in the file, inside the log, put there by a print. Filtering
	## the log would be guessing at what to redact and would cost the log its
	## point, so both surfaces say so instead.
	DccWidgets.modal_foot(body,
		"Turning a row off drops its section — it does not scrub the value from "
		+ "the file. The log tail is verbatim app output and is not filtered by "
		+ "these choices, and a captured error names the file it failed on. The "
		+ "written file repeats this at the top.")

	## `modal_choices()` owns Esc, Enter, the ✕ and the dismiss-then-act order,
	## so the two answers go through it. Copy is added afterwards and moved to
	## the middle: `modal_actions()` is `ALIGNMENT_END`, so appending would put
	## it rightmost, and the rightmost slot belongs to the action the card is
	## named for.
	var choices := DccWidgets.modal_choices(card, {
		"cancel": "Cancel",
		"safe": {"text": "Save report", "on": func(): _save(app, bridge, opts)},
	})
	var row: HBoxContainer = choices["row"]
	var copy := DccWidgets.modal_quiet(row, "Copy to clipboard",
		func(): _copy(app, bridge, opts))
	row.move_child(copy, 1)

	DccWidgets.modal_present(card, app)

## Rebuilt whole on every toggle. The rows are cheap (the expensive one is the
## parameter dump, which is a dictionary read and a sort) and rebuilding is the
## only shape that cannot drift: there is no per-row "now hide yourself" path
## to get wrong, and what is on screen is always a fresh `manifest()`.
static func _fill(table: VBoxContainer, app: Node, bridge: EngineBridge,
		opts: Dictionary) -> void:
	for c in table.get_children():
		table.remove_child(c)
		c.queue_free()
	var inset := DccWidgets.modal_inset(table)
	var rows := DiagnosticReport.manifest(app, bridge, opts)
	var included := 0
	for r in rows:
		if bool(r["included"]):
			included += 1
			DccWidgets.modal_stat(inset, String(r["label"]), String(r["preview"]))
		else:
			## The dash and its reason, never an empty value and never a
			## plausible one. `modal_stat_absent()` puts `why` on the row's
			## tooltip, so the reason travels with the dash.
			DccWidgets.modal_stat_absent(inset, String(r["label"]), String(r["why"]))
		## **Where the value came from, under every row including the dashed
		## ones.** A dashed row's source is the symbol that was asked and could
		## not answer, which is the more useful half of the dash.
		var src := DccTheme.mono_label("from " + String(r["source"]),
			"text_ghost", DccTheme.FS_MICRO)
		src.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		inset.add_child(src)
	DccWidgets.modal_foot(table, "%d of %d values will be written."
		% [included, rows.size()])

static func _save(app: Node, bridge: EngineBridge, opts: Dictionary) -> void:
	DiagnosticReport.write(app, bridge, opts)

## Works with no network and on a device with no file manager -- the canvas's
## own reason for keeping it. Exactly the bytes `write()` would have put on
## disk, from the same `build_text()` call, so a pasted report and a saved one
## are the same artefact.
static func _copy(app: Node, bridge: EngineBridge, opts: Dictionary) -> void:
	var text := DiagnosticReport.build_text(app, bridge, opts)
	DisplayServer.clipboard_set(text)
	if app != null and app.has_method("set_status"):
		app.set_status("hint", "diagnostic report copied to clipboard (%d characters)"
			% text.length(), "text_dim")
