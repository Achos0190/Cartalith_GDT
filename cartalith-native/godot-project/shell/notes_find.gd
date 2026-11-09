extends RefCounted
## The map's Find `n ` scope: the vault's notes (VE-9, `VAULT_EDITOR_PROPOSAL.md` s.7).
##
## `DccShell._search_result_set()` calls `rows()` when the typed scope is `n`; it returns the same dictionary shape
## the place scopes do (`rows`, `total`, `query`) plus two optional keys: `note` (shown INSTEAD of a list, for a
## state where there is nothing to search: no vault, no query) and `footer` (shown after the rows or after "No
## matches", for the qualifier a list must not hide).
##
## **What it must never do.** Present a names-only or capped search as complete (the footer says which, from the
## engine's own `indexed`, `truncated` and the three-character floor, the same three facts the vault window's
## search states); walk the whole vault on a keystroke (`vault_search` reads at most its own budget, and the
## shell runs this only when the field changes); read a note's text into a row (the row carries the engine's
## excerpt only); open or write anything (a press is `DccShell._select_search_hit`, which opens the note in the
## vault window); hide a private note (Ruling DP: it is listed, drawn as private).
##
## Preloaded by `dcc_shell.gd` (`NotesFind`), no `class_name`, so it needs no `--headless --import` pass.

## A label for a note row's kind line; the rows' `entity` is `note`.
const KIND := "note"


## The scope's result set for `query` (already stripped of the `n ` token). `shell` is the `DccShell` (it carries
## `bridge` through `_find_engine_bridge()` and the vault window through `vault_window` on the app).
static func rows(shell, bridge, query: String) -> Dictionary:
	var out := {"query": query, "rows": [], "total": 0}
	if bridge == null or not bool(bridge.vault_info().get("bound", false)):
		out["note"] = "No vault folder is connected, so there are no notes to search. Connect one under Records ▸ Markdown vault."
		return out
	if query == "":
		out["note"] = "Type a note's name, or words from inside one, after n . Names are searched always; the text inside notes once the content index has been built."
		return out
	var r: Dictionary = bridge.vault_search(query, 30, 0)
	if not bool(r.get("ok", false)):
		out["note"] = "Notes: %s" % String(r.get("error", "the search was refused"))
		return out
	var vw = shell.get("vault_window")
	var hits: Array = r.get("hits", [])
	var rels: Array = []
	for h in hits:
		rels.append(String((h as Dictionary).get("rel", "")))
	if vw != null:
		vw.privacy().decide(rels)
	var rows: Array = []
	for h in hits:
		var d: Dictionary = h
		var rel := String(d.get("rel", ""))
		var base := rel.get_file().trim_suffix(".md")
		var sub := rel
		var ex := String(d.get("excerpt", "")).strip_edges()
		if ex != "" and not bool(d.get("in_name", false)):
			sub += " · " + ex
		var name := base
		var priv := ""
		if vw != null and vw.privacy().is_private(rel):
			name += vw.privacy().SUFFIX
			priv = vw.privacy().reason(rel)
		var row := {"name": name, "kind": KIND, "subtitle": sub, "entity": "note", "rel": rel}
		if priv != "":
			row["private"] = priv
		rows.append(row)
	out["rows"] = rows
	out["total"] = rows.size()
	var scanned := int(r.get("scanned", 0))
	if not bool(r.get("indexed", false)):
		out["footer"] = "Names only: this vault has no content index, so the inside of a note was not looked at. Build it in Records ▸ Markdown vault."
	elif query.length() < 3:
		out["footer"] = "Names only: a query under three characters is not confirmed against the text of a note. Add a letter to search inside the notes too."
	elif bool(r.get("truncated", false)):
		out["footer"] = "Cut short: there are more matches, and notes past the scan limit (%d opened) were not read. Narrow the query." % scanned
	return out
