extends SyntaxHighlighter
## Write-mode colouring for the vault's Markdown editor (VE-3, `VAULT_EDITOR_PROPOSAL.md` s.3.1, s.5).
##
## Loaded by `markdown_editor.gd` with `preload()`, deliberately without a `class_name` (a new global
## class needs `godot --headless --import`, which strips `project.godot`'s `;` comments;
## `MISTAKES.md`).
##
## ## What it does, and what it cannot
##
## It colours the SOURCE by column: headings, dimmed markers, links, tags, code, emphasis, quotes,
## the front matter. **A `SyntaxHighlighter` honours colour only** -- `_ve0_measure_probe.gd` Q1 found
## that a `font_size` or `font` key leaves the line's width and height unchanged -- so there is no
## live bold, italic or larger heading in Write; Preview and Split are where those are drawn
## (s.3.1's stated substitute). Because a colour change moves no glyph, the caret and the colours
## cannot drift apart at any width or wrap: the text is laid out once, by the `TextEdit`.
##
## ## Where each rule lives
##
## - **Links come from the Rust crate** (`link_source`, the editor's hook onto `vault_link_spans`), so
##   GDScript keeps no second link parser (s.5). Everything else here is lexical colouring of a line
##   (a bold marker, a `#tag`, a fence), which the crate does not model and does not need to.
## - **Line state** (front matter, fenced code) needs more than one line: `line_kinds` is one pure
##   pass over the lines, and `_get_line_syntax_highlighting` re-reads it whenever the `TextEdit`'s
##   edit `get_version()` moves.
## - **Colours are theme tokens, then held to a floor** (`palette`): text roles meet `TEXT_FLOOR`
##   (WCAG AA, 4.5:1) against the field ground and the dimmed marker meets `MARKER_FLOOR`.
##   `legible` nudges a token toward `text_bright` just far enough to meet its floor, in whichever
##   theme is active, so no role can be unreadable in the light palette (where `accent` measures
##   3.93:1 on the field and `good` 4.39:1).
##
## Never writes a file, never edits the text, and holds no vault knowledge beyond the two hooks.

## Role of a run. Integers so a line's roles are a flat array; the order is not significant.
const BODY := 0
const MARKER := 1
const HEADING := 2
const EMPH := 3
const CODE := 4
const LINK := 5
const TAG := 6
const HIGHLIGHT := 7
const STRIKE := 8
const QUOTE := 9
const FRONT := 10
const TASK := 11
const MATCH := 12

## Line kinds from `line_kinds`: ordinary Markdown, front matter (delimiters included), a fence's
## opening or closing line, and the lines between a fence's.
const K_NORMAL := 0
const K_FRONT := 1
const K_FENCE := 2
const K_CODE := 3

## Contrast floors against the field ground. 4.5:1 is WCAG 2.1 AA for text. 3:1 for the dimmed marker
## is a labelled judgement (WCAG 1.4.11's non-text floor): a marker is decoration the reader needs to
## see but not to read, and `text_dim` measures 4.25:1 in the light palette. Neither
## `VAULT_EDITOR_PROPOSAL.md` nor `DESIGN_HANDOFF.md` states a numeric bar, so these are the
## project's own, recorded here.
const TEXT_FLOOR := 4.5
const MARKER_FLOOR := 3.0

## Host hook: `link_source(line: String) -> Array` of `{start, end, syntax, external}` (character
## offsets into the line), `vault_link_spans`' shape. Without it no link is coloured.
var link_source: Callable = Callable()
## Host hook: `match_source(line_text: String) -> Array` of `Vector2i(from, to)` columns of the
## find bar's matches in that line. Without it nothing is marked.
var match_source: Callable = Callable()

var _palette: Dictionary = {}
var _kinds := PackedInt32Array()
var _kinds_version := -1

static var _re: Dictionary = {}


func _init() -> void:
	_palette = palette()


## Rebuilds the colours from the current theme. The editor calls this when it is built; a theme flip
## rebuilds the vault window, which builds a new editor.
func refresh_palette() -> void:
	_palette = palette()
	clear_highlighting_cache()


## The colour the highlighter uses for `role`, as an editor probe or a contrast check reads it.
func color_of(role: int) -> Color:
	return _palette.get(role, _palette[BODY])


## Brings the line kinds up to date. Called by the editor after an edit and after the find query changes.
## It clears no cache, deliberately: with the explicit clears removed, lines below a newly typed fence and
## lines whose find hits changed were still redrawn in their new colours (`_ve3_probe.gd`, two survivors
## of the literal mutants), so the `TextEdit` is not serving stale colours for either.
func refresh() -> void:
	var te := get_text_edit()
	if te != null:
		_sync_kinds(te)


## The kind (`K_*`) of line `line`: `K_NORMAL` for an ordinary Markdown line. The editor's smart keys
## use it to leave fences and the front matter alone.
func kind_at(line: int) -> int:
	var te := get_text_edit()
	if te == null:
		return K_NORMAL
	_sync_kinds(te)
	return _kinds[line] if line >= 0 and line < _kinds.size() else K_NORMAL


## Re-reads the line kinds when the text has changed since they were read. No cache handling is needed
## here: the `TextEdit` drops the cached colours of the edited line and every line below it itself
## (measured: with an explicit clear removed, a line below a newly typed fence still turned to code
## colour in the probe), so a fence opened above recolours what follows.
func _sync_kinds(te: TextEdit) -> void:
	if _kinds_version == te.get_version() and _kinds.size() == te.get_line_count():
		return
	var lines := PackedStringArray()
	for i in te.get_line_count():
		lines.append(te.get_line(i))
	_kinds = line_kinds(lines)
	_kinds_version = te.get_version()


func _get_line_syntax_highlighting(line: int) -> Dictionary:
	var te := get_text_edit()
	if te == null or line < 0 or line >= te.get_line_count():
		return {}
	_sync_kinds(te)
	var text := te.get_line(line)
	var links: Array = []
	var kind: int = _kinds[line] if line < _kinds.size() else K_NORMAL
	if kind == K_NORMAL and link_source.is_valid() and text.find("[") >= 0:
		links = link_source.call(text)
	var matches: Array = []
	if match_source.is_valid() and not text.is_empty():
		matches = match_source.call(text)
	var roles := line_roles(text, kind, links, matches)
	var out := {}
	var prev := -1
	for i in roles.size():
		if roles[i] != prev:
			out[i] = {"color": color_of(roles[i])}
			prev = roles[i]
	if out.is_empty():
		out[0] = {"color": color_of(BODY)}
	return out


# -- Pure ---------------------------------------------------------------------

## The kind of every line (`K_*`): front matter is the block opened by a `---` on line 0 and closed
## by the next `---` or `...`; a fence is opened and closed by a line starting with three backticks or
## three tildes (up to three spaces of indent). A fence the text never closes runs to the end, as in
## CommonMark. A labelled simplification: the closing fence is any fence line, not one of the same
## character and at least the same length.
static func line_kinds(lines: PackedStringArray) -> PackedInt32Array:
	var out := PackedInt32Array()
	out.resize(lines.size())
	var i := 0
	if lines.size() > 0 and lines[0].strip_edges(false, true) == "---":
		var j := 1
		while j < lines.size() and lines[j].strip_edges(false, true) not in ["---", "..."]:
			j += 1
		if j < lines.size():
			for k in range(0, j + 1):
				out[k] = K_FRONT
			i = j + 1
	var in_fence := false
	while i < lines.size():
		var l := lines[i]
		var t := l.lstrip(" ")
		var fence := (l.length() - t.length()) <= 3 and (t.begins_with("```") or t.begins_with("~~~"))
		if fence:
			out[i] = K_FENCE
			in_fence = not in_fence
		else:
			out[i] = K_CODE if in_fence else K_NORMAL
		i += 1
	return out


## The role of every character of `text`. `links` are `vault_link_spans` rows for this line,
## `matches` are `Vector2i(from, to)` find hits. Roles are painted in order -- the block structure
## first, then emphasis, then code (which overrides, so `**` inside backticks is code), then links,
## then find matches -- so a later rule wins.
static func line_roles(text: String, kind: int, links: Array = [], matches: Array = []) -> PackedInt32Array:
	var n := text.length()
	var r := PackedInt32Array()
	r.resize(n)
	if kind == K_FRONT:
		var d := text.strip_edges() in ["---", "..."]
		for i in n:
			r[i] = MARKER if d else FRONT
		return _paint_matches(r, matches)
	if kind == K_CODE:
		for i in n:
			r[i] = CODE
		return _paint_matches(r, matches)
	if kind == K_FENCE:
		for i in n:
			r[i] = MARKER
		return _paint_matches(r, matches)
	var body_from := 0
	var heading := false
	var quote := false
	var m: RegExMatch = _rx("heading").search(text)
	if m != null:
		_paint(r, m.get_start(1), m.get_end(2), MARKER)
		_paint(r, m.get_end(2), n, HEADING)
		heading = true
		body_from = m.get_end(2)
	elif _rx("hr").search(text) != null:
		_paint(r, 0, n, MARKER)
		return _paint_matches(r, matches)
	else:
		m = _rx("quote").search(text)
		if m != null:
			_paint(r, m.get_start(1), m.get_end(1), MARKER)
			_paint(r, m.get_end(1), n, QUOTE)
			quote = true
			body_from = m.get_end(1)
		var rest := text.substr(body_from)
		var lm: RegExMatch = _rx("list").search(rest)
		if lm != null:
			_paint(r, body_from + lm.get_start(1), body_from + lm.get_end(1), MARKER)
			body_from += lm.get_end(0)
			var tm: RegExMatch = _rx("task").search(text.substr(body_from))
			if tm != null:
				_paint(r, body_from + tm.get_start(0), body_from + tm.get_end(0), TASK)
		if quote:
			var cm: RegExMatch = _rx("callout").search(text.substr(body_from))
			if cm != null:
				_paint(r, body_from + cm.get_start(0), body_from + cm.get_end(0), HIGHLIGHT)
	# Emphasis, strike and highlight. In a heading the text keeps the heading colour (`soft`) and only
	# the markers dim.
	_inline_pair(r, text, "bold", EMPH, heading)
	_inline_pair(r, text, "italic_star", EMPH, heading)
	_inline_pair(r, text, "italic_under", EMPH, heading)
	_inline_pair(r, text, "strike", STRIKE, false)
	_inline_pair(r, text, "mark", HIGHLIGHT, false)
	for tm2 in _rx("tag").search_all(text):
		_paint(r, tm2.get_start(0), tm2.get_end(0), TAG)
	for cm2 in _rx("code").search_all(text):
		_paint(r, cm2.get_start(0), cm2.get_end(0), CODE)
		_paint(r, cm2.get_start(1), cm2.get_end(1), MARKER)
		_paint(r, cm2.get_start(3), cm2.get_end(3), MARKER)
	for ln in links:
		var s := int((ln as Dictionary).get("start", 0))
		var e := int((ln as Dictionary).get("end", 0))
		_paint(r, s, e, LINK)
		var syntax := String((ln as Dictionary).get("syntax", ""))
		if syntax == "wiki" or syntax == "embed":
			var open := 3 if syntax == "embed" else 2
			_paint(r, s, s + open, MARKER)
			_paint(r, e - 2, e, MARKER)
	return _paint_matches(r, matches)


static func _paint_matches(r: PackedInt32Array, matches: Array) -> PackedInt32Array:
	for mt in matches:
		_paint(r, (mt as Vector2i).x, (mt as Vector2i).y, MATCH)
	return r


## Sets `role` on `[from, to)`, clamped to the line.
static func _paint(r: PackedInt32Array, from: int, to: int, role: int) -> void:
	for i in range(maxi(from, 0), mini(to, r.size())):
		r[i] = role


## Paints one paired construct: the inner text `role` (unless `soft`, when only plain body text
## changes), each of the two delimiters `MARKER`. The regexes capture the opener, the inner text and
## the closer as groups 1, 2 and 3.
static func _inline_pair(r: PackedInt32Array, text: String, key: String, role: int, soft: bool) -> void:
	for m in _rx(key).search_all(text):
		_paint(r, m.get_start(1), m.get_end(1), MARKER)
		_paint(r, m.get_start(3), m.get_end(3), MARKER)
		for i in range(m.get_start(2), m.get_end(2)):
			if not soft or r[i] == BODY:
				r[i] = role


## Compiled once. Each pattern is documented where it is built.
static func _rx(key: String) -> RegEx:
	if _re.is_empty():
		var pats := {
			# `# ` .. `###### `: group 1 the hashes, group 2 the spaces after them.
			"heading": "^ {0,3}(#{1,6})([ \\t]+|$)",
			# Three or more of one rule character, spaces allowed between.
			"hr": "^ {0,3}([-*_])([ \\t]*\\1){2,}[ \\t]*$",
			# Block-quote markers `>`, `> >`; group 1 includes the space after each.
			"quote": "^( {0,3}(?:>[ \\t]?)+)",
			# A bullet or number marker; group 1 is the marker itself, group 0 includes the spaces.
			"list": "^([ \\t]*(?:[-*+]|\\d{1,9}[.)]))[ \\t]+",
			"task": "^\\[[ xX]\\]",
			"callout": "^\\[![^\\]]+\\][+-]?",
			# `**x**` / `__x__`: opener, inner text that does not start or end in a space, closer.
			"bold": "(\\*\\*|__)(?=\\S)(.+?)(?<=\\S)(\\1)",
			"italic_star": "(?<![*\\w])(\\*)(?![\\s*])(.+?)(?<![\\s*])(\\*)(?![*\\w])",
			"italic_under": "(?<![_\\w])(_)(?![\\s_])(.+?)(?<![\\s_])(_)(?![_\\w])",
			"strike": "(~~)(?=\\S)(.+?)(?<=\\S)(~~)",
			"mark": "(==)(?=\\S)(.+?)(?<=\\S)(==)",
			# Obsidian: a tag may not be all digits and holds no spaces.
			"tag": "(?<![\\w/&#])#(?=[A-Za-z0-9_/-]*[A-Za-z_/-])[A-Za-z0-9_/-]+",
			# A code span: opener run, text, matching closer run (groups 1, 2, 3).
			"code": "(?<!`)(`+)(?!`)(.+?)(?<!`)(\\1)(?!`)",
		}
		for k in pats:
			var rx := RegEx.new()
			rx.compile(pats[k])
			_re[k] = rx
	return _re[key]


# -- Colours ------------------------------------------------------------------

## Role -> colour for the active theme: tokens, held to their floor against the field ground
## (`sunken`, what `DccTheme.field_box` fills). See the file header for why `legible` exists.
static func palette() -> Dictionary:
	var ground := DccTheme.c("sunken")
	var t := func(token: String) -> Color: return legible(DccTheme.c(token), ground, TEXT_FLOOR)
	return {
		BODY: t.call("text"),
		MARKER: legible(DccTheme.c("text_dim"), ground, MARKER_FLOOR),
		HEADING: t.call("accent"),
		EMPH: t.call("text_bright"),
		CODE: t.call("block"),
		LINK: t.call("water"),
		TAG: t.call("good"),
		HIGHLIGHT: t.call("warn"),
		STRIKE: t.call("text_dim"),
		QUOTE: t.call("text_secondary"),
		FRONT: t.call("text_dim"),
		TASK: t.call("accent"),
		MATCH: t.call("warn"),
	}


## `fg` itself when it already meets `floor` against `ground`, otherwise the nearest blend toward
## `text_bright` (in 5% steps) that does. `text_bright` is 14:1 or better in both palettes, so the
## loop always ends; it stops at the first step that passes so the hue is kept as far as possible.
static func legible(fg: Color, ground: Color, floor_ratio: float) -> Color:
	var target := DccTheme.c("text_bright")
	var c := Color(fg.r, fg.g, fg.b, 1.0)
	var step := 0
	while contrast(c, ground) < floor_ratio and step < 20:
		step += 1
		c = Color(fg.r, fg.g, fg.b, 1.0).lerp(target, 0.05 * step)
	return c


## WCAG 2.1 contrast ratio of two opaque colours, 1.0 to 21.0.
static func contrast(a: Color, b: Color) -> float:
	var la := _luminance(a)
	var lb := _luminance(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)


static func _luminance(c: Color) -> float:
	var f := func(v: float) -> float:
		return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)
	return 0.2126 * f.call(c.r) + 0.7152 * f.call(c.g) + 0.0722 * f.call(c.b)
