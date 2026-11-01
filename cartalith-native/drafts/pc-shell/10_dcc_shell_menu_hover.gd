## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8, "Desktop menus open as side-by-side columns" (milestone PC-9). Three
## sentences, read against the shipped bar:
##   - "An open submenu is a second column beside its parent, so the whole path stays visible": what a
##     Godot `PopupMenu` submenu already does (it opens beside its row and the parent stays open). No
##     change. Not checked on screen in this session.
##   - "Rows are one line, with the accelerator on the right": `PopupMenu`'s own layout. No change.
##   - "Moving across the bar while a menu is open switches menus": NOT today's behaviour.
##     `MenuButton.switch_on_hover` defaults to false and nothing in `shell/` sets it
##     (`grep -rn switch_on_hover shell/` finds nothing at a38e0ac). This draft sets it.
##
## Target: `shell/dcc_shell.gd`, `add_menu()`. Depends on nothing; independent of drafts 01-09.
## Applies to: the PC only. A touch screen has no hover to switch on, so `_touch` (the tablet and the
## phone, set before the menus are built) leaves it off; the phone's bar is a hidden model anyway.
## Forms: one.
##
## What to do. The EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   EDIT D1  `add_menu()`: directly after the line
##                mb.focus_mode = Control.FOCUS_NONE
##            insert the three lines below (one tab of indentation).
##
## Verify: the parse check on `res://shell/dcc_shell.gd` and `res://shell/app.gd`. By hand on the PC:
## open File, move the pointer across Edit, Assets, World: each opens as the pointer reaches it, and
## the previous one closes. On a tablet build (`--force-touch`) nothing changes.

## ── EDIT D1: insert after `mb.focus_mode = Control.FOCUS_NONE` in `add_menu()` ───────────────────
	## PC-9 (`PC_SHELL_PROPOSAL.md` §3.8): with one menu open, moving the pointer across the bar
	## opens the menu under it, as every desktop menu bar does. Off on touch: there is no hover.
	mb.switch_on_hover = not _touch
## ── end of EDIT D1 ──────────────────────────────────────────────────────────────────────────────────
