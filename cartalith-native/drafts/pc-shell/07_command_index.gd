## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8, "Search keeps every moved title" (milestone PC-9), and `MISTAKES.md`,
## "Move a command off the menu bar": the index is built by walking the live menu bar, so a title that
## stops being a menu row stops being findable, silently. Every such title gets an `EXTRAS` row in the
## same change that moves it.
##
## Target: `shell/command_index.gd` (`CommandIndex`). Depends on nothing. **Lands in the same change
## as `06_menus_help_assets.gd`** (which removes "Import image…") **and `02_menus_world.gd`'s EDIT W5**
## (which renames the tablet's "Run pipeline"); the other two aliases switch on by themselves when PC-4
## and PC-5 rename their rows (see `alias` below), so no later change has to remember them.
## Applies to: every form factor (the index is built on all three).
##
## Which titles leave the bar, measured by TITLE (not by `search()`, which also matches blurbs):
##   "Import image"          leaves Assets now (06). No other menu row has the title. ALIAS.
##   "Notes for this place"  becomes "Note for the selection" from PC-5 (04). A search for "notes" no
##                           longer reaches it by title ("Note for" does not contain "notes"). ALIAS.
##   "Right dock"            becomes "Context panel" from PC-4 (05), on the phone too since Ruling DX
##                           PQ13. "dock" then finds only "Left dock". ALIAS.
##   "Run pipeline"          the narrow tablet's World row, renamed "Generate world" by Ruling DX PQ11
##                           (02, EDIT W5). "pipeline" then matches no title. ALIAS (the ruling asks for it).
##   "Political history of this place", "Assistant panel", Help's "Keyboard shortcuts": renamed, but
##                           the words a searcher types ("political history", "assistant", "keyboard")
##                           still match the new titles. No alias.
##   World's "Timeline", Story's "Places without notes": the same titles stay on the bar (Story ▸
##                           Timeline, World ▸ Places without notes…). No alias.
##
## What to do. Each EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   EDIT X1  `EXTRAS`: after the `Theme` row (the line beginning
##                {"title": "Theme", "blurb": "Dark, Light or Follow system.
##            ) and before the closing `]`, insert the four rows in the block marked below.
##   EDIT X2  append the paragraph marked below to the end of `EXTRAS`' doc block, directly above
##            `const EXTRAS: Array = [`.
##   EDIT X3  `build()`: directly after the line
##                for e in EXTRAS:
##            insert the two lines
##                if bool(e.get("alias", false)) and _has_menu_title(String(e.get("title", ""))):
##                    continue
##            (two tabs, then three).
##   ADD      `_has_menu_title()` below, after `build()`.
##
## Verify: the parse check on `res://shell/command_index.gd`. `_idxfind_probe.gd` builds the index with
## no app (`build(null, null)`), so every alias row appears there: add `"Import image": 1,
## "Notes for this place": 1, "Right dock": 1, "Run pipeline": 1` to its `want` table, and keep
## "Journey planner": 0 and "Theme": 1. With the app: a search for "import image" lists the alias; before
## PC-5 a search for "notes for this place" lists the menu row only, not the alias beside it; on a tablet,
## a search for "run pipeline" lists the alias, which names World ▸ Generate world.

## ── EDIT X1: the four rows to insert into `EXTRAS`, after the Theme row ──────────────────────────────
	{"title": "Import image", "blurb": "Moved: Assets ▸ ⧉ Asset library (Shift+A; ☰ on a tablet) opens the library, and its own Import image… in the rail foot imports into the focused slot.", "group": "Assets", "alias": true},
	{"title": "Notes for this place", "blurb": "Now Story ▸ Note for the selection (Ctrl+Shift+N): the Context panel's Note tab, for a place or anything else selected.", "group": "Story", "alias": true},
	{"title": "Right dock", "blurb": "Now the Context panel: Window ▸ Context panel (☰ ▸ Window on a tablet; on a phone, the selection sheet) shows or hides it, and the submenu under it opens Inspect, Note and Ask (Ctrl+1, Ctrl+2, Ctrl+3).", "group": "Window", "alias": true},
	{"title": "Run pipeline", "blurb": "Now World ▸ Generate world (Ctrl+R) on every form factor: the same command under one label (Ruling DX).", "group": "World", "alias": true},
## ── end of EDIT X1 ──────────────────────────────────────────────────────────────────────────────────

## ── EDIT X2: append to `EXTRAS`' doc block ────────────────────────────────────────────────────────
## **Four ALIAS rows added by PC-9 (`PC_SHELL_PROPOSAL.md` §3.8), and the `alias`
## key that came with them.** "Import image" left Assets (it opened the Asset
## library under a second name). "Notes for this place" becomes "Note for the
## selection" when the Context panel gains its Note tab (PC-5), and "Right dock"
## becomes "Context panel" when the panel exists (PC-4; on the phone too, where
## it names the selection sheet, Ruling DX PQ13). "Run pipeline" became "Generate
## world" on the narrow tablet (Ruling DX PQ11: one label on every form factor).
## After each, a search for the old words finds nothing by title -- the defect
## this table is for. `alias: true` makes `build()` skip the row while a live
## menu row still carries that exact title (before the change that renames it),
## so a pointer never sits beside the row it points away from. Renames whose old words still match the
## new title ("political history", "assistant", "keyboard") and titles still on
## the bar elsewhere ("Timeline", "Places without notes") need no alias. Counted
## by TITLE, never by `search()`, which matches blurbs (`MISTAKES.md`, "Move a
## command off the menu bar").
## ── end of EDIT X2 ──────────────────────────────────────────────────────────────────────────────────

## Whether a menu row titled `title` is in the index right now, compared the way `_walk_popup()` stores
## titles (the ellipsis dropped, edges stripped) and without regard to case. Counts `menu` and
## `readout` rows, and a `_todo` row too (it is a menu row whose command is not built): any of them
## means the title is still on the bar, so an alias for it would sit beside it.
##
## Must run after `_add_menu_commands()`, which is the only thing that adds menu rows; `build()` calls
## it in the `EXTRAS` loop, after that walk. Reads the index only.
func _has_menu_title(title: String) -> bool:
	var want := title.replace("…", "").strip_edges().to_lower()
	for r in _rows:
		var kind := String(r.get("kind", ""))
		if (kind == "menu" or kind == "readout") and String(r.get("title", "")).to_lower() == want:
			return true
	return false
