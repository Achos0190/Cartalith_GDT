class_name VaultCompare
extends RefCounted
## VE-4's Compare (`VAULT_EDITOR_PROPOSAL.md` s.7a): the pure half. A line diff of the editor's text
## ("Mine") against the disk version ("Theirs") that says which lines of each side are not shared.
##
## It must never write anything and never read the vault; the window feeds it two strings. The view and the
## three choices live in `vault_window.gd` (`_open_compare`), which reuses `_on_banner_choice`: there is
## no second save path.
##
## Method: a longest-common-subsequence over whole lines. The table is O(n*m), so `MAX_CELLS` bounds it;
## the figure is a labelled judgement (4 million cells is about 16 MB as a packed int array, a note of
## 2 000 lines against itself), not a measurement. Above it the fallback marks a line changed when its
## text occurs nowhere on the other side (a multiset-free approximation, order ignored), and `exact` is
## false so the view says so rather than pretending.

const MAX_CELLS := 4_000_000


## Returns `{"mine": PackedInt32Array, "theirs": PackedInt32Array, "exact": bool}`: the 0-based indices of
## the lines on each side that are NOT part of the common subsequence.
static func changed_lines(mine: PackedStringArray, theirs: PackedStringArray) -> Dictionary:
	var n := mine.size()
	var m := theirs.size()
	var a_out := PackedInt32Array()
	var b_out := PackedInt32Array()
	if n * m > MAX_CELLS:
		var in_b := {}
		for s in theirs:
			in_b[s] = true
		var in_a := {}
		for s in mine:
			in_a[s] = true
		for i in n:
			if not in_b.has(mine[i]):
				a_out.append(i)
		for j in m:
			if not in_a.has(theirs[j]):
				b_out.append(j)
		return {"mine": a_out, "theirs": b_out, "exact": false}
	var w := m + 1
	var t := PackedInt32Array()
	t.resize((n + 1) * w)
	for i in range(n - 1, -1, -1):
		for j in range(m - 1, -1, -1):
			if mine[i] == theirs[j]:
				t[i * w + j] = t[(i + 1) * w + j + 1] + 1
			else:
				t[i * w + j] = maxi(t[(i + 1) * w + j], t[i * w + j + 1])
	var i2 := 0
	var j2 := 0
	while i2 < n and j2 < m:
		if mine[i2] == theirs[j2]:
			i2 += 1
			j2 += 1
		elif t[(i2 + 1) * w + j2] >= t[i2 * w + j2 + 1]:
			a_out.append(i2)
			i2 += 1
		else:
			b_out.append(j2)
			j2 += 1
	while i2 < n:
		a_out.append(i2)
		i2 += 1
	while j2 < m:
		b_out.append(j2)
		j2 += 1
	return {"mine": a_out, "theirs": b_out, "exact": true}
