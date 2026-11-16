extends RefCounted
## VE-6's template chooser (`VAULT_EDITOR_PROPOSAL.md` VE-6, audit `GUI_PLANS_AUDIT_2026-11-16.md`): the one control a new
## note's form uses to choose what it starts from. It lists the vault's templates (`vault_templates`, the folder the
## vault's own Obsidian / Templater settings name; Cartalith ships none) and a `(blank note)` choice, and shows a SHORT
## PREVIEW of the chosen one under the dropdown, so the user sees what they are about to copy before pressing Create.
## One implementation for the two forms that create a note for a world entity: the vault window's place panel
## (`VaultWindow._build_create`, Ruling FJ) and the Context panel's Note tab (`ContextNote._build_create`). The unresolved
## link's menu rows (`VaultWindow._create_items`) stay a menu: each row IS the choice.
##
## What it must never do: write anything. Reading a template to preview it is `vault_read_file`, the automatic
## half of the vault's read/write rule. Nothing here fills placeholders; the preview shows the template's own text
## (`{{title}}` and all) and says so, because the fill happens at Create, in `cartalith_vault::template::fill`.
##
## Phone, tablet and desktop share the build: the dropdown is `DccWidgets.choice` (touch-sized on a tablet), the preview a
## wrapping label with a width floor (an autowrap label measures its height at its current width, 1 px before the first
## layout, which would grow the window tall; the same floor `vault_window.gd` puts on its template-source line).

const DccWidgetsRef := preload("res://shell/dcc_widgets.gd")
const DccThemeRef := preload("res://shell/dcc_theme.gd")

## The dropdown's text for the blank choice (one string for the two forms, and for probes).
const BLANK_LABEL := "(blank note)"
## How much of a template the preview shows: lines, then characters. Labelled judgement: enough to recognise a template's
## headings and fields without turning the form into a document; a phone shows about this much above the Create button.
const PREVIEW_MAX_LINES := 8
const PREVIEW_MAX_CHARS := 360
## What a template with no text previews as (an unreadable file, or an empty one), never a made-up line.
const PREVIEW_EMPTY := "(empty template)"

## Builds the chooser into `parent`. `name_for_blank` is the entity's name the blank note's title line uses (`# Name`).
## `blank_first` puts `(blank note)` at the top (the Note tab's order, default blank) and else last (the vault window's order,
## default the first template). `blank_value` is what `on_change` receives for the blank choice (`""` on the Note tab,
## the window's own sentinel there); a template's value is its vault-relative path. `selected` is the value to start on.
## Returns `{option, preview, values}`: the dropdown, the preview label, and the values in dropdown order (for tests).
static func build(parent: Control, bridge, name_for_blank: String, selected: String, blank_value: String,
		blank_first: bool, on_change: Callable) -> Dictionary:
	var labels: Array = []
	var values: Array = []
	if blank_first:
		labels.append(BLANK_LABEL)
		values.append(blank_value)
	for t in bridge.vault_templates():
		var d: Dictionary = t
		labels.append(String(d.get("label", "?")))
		values.append(String(d.get("rel", "")))
	if not blank_first:
		labels.append(BLANK_LABEL)
		values.append(blank_value)
	var at := maxi(0, values.find(selected))
	var preview := DccThemeRef.label("", "text_ghost", DccThemeRef.FS_SMALL)
	preview.name = "TemplatePreview"
	preview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	preview.custom_minimum_size.x = 160
	var show := func(i: int) -> void:
		var v := String(values[i])
		preview.text = preview_for(bridge, v, name_for_blank, blank_value)
	var opt: OptionButton = DccWidgetsRef.choice(parent, "Template", labels, at,
		func(i: int):
			show.call(i)
			on_change.call(i, String(values[i])),
		"The vault's own templates, from the folder your Obsidian settings name -- or any .md whose path contains \"template\". "
		+ "Cartalith ships none; a blank note holds the title only. The text below is what will be copied.")
	opt.name = "NoteTemplate"
	parent.add_child(preview)
	show.call(at)
	return {"option": opt, "preview": preview, "values": values}

## The preview text for one choice: the blank note's own title line, or the template's first lines.
static func preview_for(bridge, value: String, name_for_blank: String, blank_value: String) -> String:
	if value == blank_value:
		return "# %s" % name_for_blank
	return clip(bridge.vault_read_file(value))

## A template's text cut for the preview: at most `PREVIEW_MAX_LINES` non-trailing lines and `PREVIEW_MAX_CHARS` characters,
## ending `...` when anything was cut; `PREVIEW_EMPTY` for a blank text. Carriage returns are dropped.
static func clip(raw: String) -> String:
	var text := raw.replace("\r", "").strip_edges()
	if text == "":
		return PREVIEW_EMPTY
	var lines := text.split("\n")
	var cut := lines.size() > PREVIEW_MAX_LINES
	var out := "\n".join(lines.slice(0, PREVIEW_MAX_LINES))
	if out.length() > PREVIEW_MAX_CHARS:
		out = out.substr(0, PREVIEW_MAX_CHARS)
		cut = true
	return out + ("\n..." if cut else "")
