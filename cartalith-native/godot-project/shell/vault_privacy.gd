extends RefCounted
## The private badge's cache (Ruling DP; VE-9): which vault notes are private, so every list and link that
## shows a note can say so beside its name.
##
## **The rule is not here.** `cartalith_vault::private::private_marker` decides (folder, frontmatter, tags,
## fail-closed on malformed frontmatter); this file asks the bridge (`vault_private_marks`) and remembers the
## answer. Nothing in this file reads a note or guesses.
##
## **Three states, never two.** A note is private (a reason is held), public (decided, no reason) or
## *undecided* (not asked yet, past the read budget, or unreadable). Undecided draws **no** badge and is not
## cached as public: "not looked at" is not "not private" (`MISTAKES.md`, no value as a plausible value).
## `pending()` says which of a list are still undecided so a long list can be finished across frames.
##
## **Must never:** write a note; keep note text (only the bridge's fixed-vocabulary reason is held, so a
## reason can be shown and logged without leaking the note); badge a world entity (Ruling DN K3: world data has
## no private marker).
##
## Preloaded by `vault_window.gd` (`Privacy`); no `class_name`, so it needs no `--headless --import` pass.

## The word every list draws after a private note's name. Plain text on purpose: it renders in every font and
## a probe can read it.
const WORD := "private"
## What follows the name: `Name  ·  private`.
const SUFFIX := "  ·  private"

## `EngineBridge`.
var bridge
## rel -> reason, for the notes decided private.
var _marks: Dictionary = {}
## rel -> true, for every note whose answer is final (private or public).
var _decided: Dictionary = {}


func _init(b) -> void:
	bridge = b


## Forgets every answer (the vault changed under us: a refresh, another vault, a rebuilt window).
func clear() -> void:
	_marks.clear()
	_decided.clear()


## Forgets one note's answer (it was saved; its frontmatter may have changed).
func forget(rel: String) -> void:
	_marks.erase(rel)
	_decided.erase(rel)


## Asks the bridge about the notes in `rels` that are still undecided, at most `budget` reads (0 takes the
## bridge's own). Returns how many it decided. Safe with no vault or no API: nothing is decided and nothing is
## badged.
func decide(rels: Array, budget: int = 0) -> int:
	var ask := PackedStringArray()
	for r in rels:
		var rel := String(r)
		if rel != "" and not _decided.has(rel):
			ask.append(rel)
	if ask.is_empty():
		return 0
	var got: Dictionary = bridge.vault_private_marks(ask, budget)
	if not bool(got.get("ok", false)):
		return 0
	var marks: Dictionary = got.get("marks", {})
	for rel in marks:
		_marks[String(rel)] = String(marks[rel])
	var n := 0
	for rel in got.get("decided", PackedStringArray()):
		_decided[String(rel)] = true
		n += 1
	return n


## Of `rels`, the ones with no final answer yet.
func pending(rels: Array) -> Array:
	var out: Array = []
	for r in rels:
		if not _decided.has(String(r)):
			out.append(String(r))
	return out


## Whether `rel` is private. `false` for an undecided note: ask `is_decided` to tell the two apart.
func is_private(rel: String) -> bool:
	return _marks.has(rel)


## Whether `rel`'s answer is final.
func is_decided(rel: String) -> bool:
	return _decided.has(rel)


## Why `rel` is private, in the crate's fixed words ("frontmatter private: true"), or `""`.
func reason(rel: String) -> String:
	return String(_marks.get(rel, ""))


## `SUFFIX` for a private note, `""` otherwise (public and undecided alike).
func suffix(rel: String) -> String:
	return SUFFIX if _marks.has(rel) else ""


## Decides `rels` (a short list: a menu, a card, a page of results) and returns `name_text` with the badge
## appended when `rel` is private.
func badged(rel: String, name_text: String) -> String:
	if not _decided.has(rel):
		decide([rel])
	return name_text + suffix(rel)
