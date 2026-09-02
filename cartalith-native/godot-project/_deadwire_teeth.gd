extends Node
## THROWAWAY. Negative control for `_deadwire_probe.gd`'s disabled-control
## rules. A probe that passes proves nothing unless its failure path still
## fires, so this hands `_audit()` a synthetic tree with one control of each
## shape and checks the verdict is what the rules say it must be.

func _pressed_a() -> void:
	pass


func _pressed_b() -> void:
	pass


func _ready() -> void:
	var probe = load("res://_deadwire_probe.gd").new()

	var box := VBoxContainer.new()
	add_child(box)

	## A: disabled, no tooltip, wired to nothing -> rule 2, DEAD-SILENT.
	var a := Button.new()
	a.text = "A dead+silent"
	a.disabled = true
	box.add_child(a)

	## B: disabled, no tooltip, wired to a real handler, never enabled in any
	## pass -> rule 3, NEVER-ENABLED. This is the hard-set `disabled = true`
	## case the assignment names.
	var b := Button.new()
	b.text = "B wired but hard-disabled"
	b.disabled = true
	b.pressed.connect(_pressed_b)
	box.add_child(b)

	## C: disabled here, enabled in the second pass -> the state-driven shape
	## the six real controls have. Must be acquitted.
	var c := Button.new()
	c.text = "C gated"
	c.disabled = true
	c.pressed.connect(_pressed_a)
	box.add_child(c)

	## D: disabled with a reason -> the `_todo()` contract, never reported.
	var d := Button.new()
	d.text = "D has a reason"
	d.disabled = true
	d.tooltip_text = "Not in this build."
	box.add_child(d)

	await get_tree().process_frame
	probe._audit("Synthetic", box)
	## Second state: C's gate opens, nothing else moves. Note that C's LABEL
	## also changes, to prove identity is the handler and not the text.
	c.disabled = false
	c.text = "C gated (open)"
	await get_tree().process_frame
	probe._audit("Synthetic", box)
	probe._verdict()

	var expect := 2   ## A and B, and only those
	print("TEETH  fail=%d expected=%d  %s"
		% [probe._fail, expect, "PASS" if probe._fail == expect else "FAIL"])
	get_tree().quit(0 if probe._fail == expect else 1)
