extends Node
## **`DccTheme._elide_labels` no longer grows without limit on desktop.**
##
## `OUTSTANDING_WORK.md` (found by the wf53 verifier): the registry that lets a
## live tablet rotation re-toggle a header's `clip_text` pruned freed entries
## only inside `set_portrait()`'s own walk -- which a desktop session never
## calls with a new value, since `_landscape` never flips there. Measured:
## 104 entries at a desktop boot, 1 054 after 150 domain switches and 50
## right-dock rebuilds, 951 of them already-freed dead weight.
##
## The fix (`dcc_theme.gd::header()`) registers a label only `if is_tablet()`.
## `is_tablet()` is decided once at boot and never revisited
## (`dcc_shell.gd::_ready()`), so a desktop or phone label can never have a
## later `set_portrait()` call change its clip state -- there is no live event
## left that could ever reach it -- and appending it bought nothing but
## growth.
##
## This probe drives `DccTheme`'s static API directly rather than booting
## `app.tscn`: `DccTheme extends RefCounted` and every symbol touched here
## (`header`, `set_touch`, `set_phone`, `set_portrait`, `is_tablet`) is pure
## GDScript with no dependency on the compiled `cartalith_godot` extension, so
## a stale `.dll` cannot make this probe report a false PASS (`MISTAKES.md`'s
## stale-binary rule does not apply to it, which is deliberately why it is
## shaped this way rather than as a churn drive through `dcc_shell.select_domain()`
## -- that would also load a `.tscn` the concurrent CIVIL rail workflow is
## mid-editing, for a fact this probe does not need the real shell to check).
##
## The real shell IS covered separately: `_elidehead_probe.gd` boots
## `app.tscn`, walks the live WORLD dock's actual header Labels, and confirms
## none of them ends up in `_elide_labels` on desktop while every one still
## carries the correct (un-elided) clip state -- the two probes together are
## the unit-plus-integration pair, not a substitute for each other.
##
## No arguments; always run headless with no viewport:
##   godot --headless _elidechurn_probe.tscn
##
## A SCRIPT ERROR must never read PASS. `_ready()` never calls `quit(0)`
## except at its own single final line, after every phase's checks, so a
## crash anywhere above leaves the process hung/errored rather than emitting a
## success exit code -- the harness sees a non-zero/aborted run either way.

var _fail := 0

func _log(s: String) -> void:
	print("[elidechurn] %s" % s)

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

## `Label.free()` is immediate (unlike `queue_free()`, which defers to end of
## frame) and needs no SceneTree membership -- neither of these labels is ever
## added as a child of anything, matching how `_apply_elide()`'s own contract
## only cares about the Label object, not its parent. Using an immediate
## `free()` here removes the one piece of frame-timing ambiguity a
## `queue_free()`-based churn probe would carry (MISTAKES.md: re-run a
## suspicious probe failure once before trusting it -- this shape has nothing
## to be suspicious of instead).
func _free_all(labels: Array) -> void:
	for l in labels:
		if is_instance_valid(l):
			l.free()

func _ready() -> void:
	## Fresh process: `DccTheme`'s static vars are still at their declared
	## defaults (`_touch=false`, `_phone_mode=false`, `_portrait=false`,
	## `_elide_labels=[]`). Set explicitly anyway so the probe does not depend
	## on that being true if a default is ever changed.
	DccTheme.set_touch(false)
	DccTheme.set_phone(false)
	_check(not DccTheme.is_tablet(), "fresh process starts is_tablet()==false")
	_check(DccTheme._elide_labels.size() == 0, "fresh process starts with an empty registry")

	# -- Desktop phase ---------------------------------------------------------
	_log("-- desktop phase (is_tablet()=false) --")

	var boot_batch: Array = []
	for i in 104:
		boot_batch.append(DccTheme.header("Section %d" % i, "§", true))
	var size_after_boot := DccTheme._elide_labels.size()
	_log("  registry size after a 104-header desktop boot: %d" % size_after_boot)
	_check(size_after_boot == 0,
		"a desktop boot registers NOTHING (104 headers built, registry stays empty)")
	## The clip state itself is unaffected by the registration change --
	## `_apply_elide()` still runs unconditionally. Sampled here before any
	## label is freed.
	var sample: Label = boot_batch[0]
	_check(sample.clip_text == false,
		"a desktop header's clip_text is false at construction (unchanged)")
	_check(sample.text_overrun_behavior == TextServer.OVERRUN_NO_TRIMMING,
		"a desktop header's overrun behaviour is OVERRUN_NO_TRIMMING (unchanged)")

	var live_batch := boot_batch
	## 150 domain switches, each tearing down the previous batch's headers and
	## building a fresh set -- the shape `OUTSTANDING_WORK.md` measured against.
	for switch_i in 150:
		_free_all(live_batch)
		live_batch = []
		for i in 6:
			live_batch.append(DccTheme.header("Domain %d.%d" % [switch_i, i], "§", true))
	## 50 right-dock rebuilds, same shape, smaller batch (`_append_layers` /
	## `_build_sample`-sized: a couple of section headers per rebuild).
	for rebuild_i in 50:
		_free_all(live_batch)
		live_batch = []
		for i in 2:
			live_batch.append(DccTheme.header("RightDock %d.%d" % [rebuild_i, i], "§", true))

	var size_after_churn := DccTheme._elide_labels.size()
	_log("  registry size after 150 domain switches + 50 right-dock rebuilds: %d"
		% size_after_churn)
	_check(size_after_churn == 0,
		"the registry is STILL empty after the full churn run (was 1 054/951-dead pre-fix)")

	_free_all(live_batch)
	_check(DccTheme._elide_labels.size() == 0, "the registry is empty after final cleanup")

	# -- Tablet phase: prove the fix does not touch tablet's own behaviour -----
	_log("-- tablet phase (is_tablet()=true) --")
	DccTheme.set_touch(true)
	DccTheme.set_phone(false)
	_check(DccTheme.is_tablet(), "tablet phase actually reaches is_tablet()==true")

	var tablet_batch: Array = []
	for i in 10:
		tablet_batch.append(DccTheme.header("Tablet %d" % i, "§", true))
	_check(DccTheme._elide_labels.size() == 10,
		"tablet headers ARE registered (10 built, 10 in the registry)")
	for l in tablet_batch:
		_check(l.clip_text == false,
			"a tablet header is un-elided at construction while landscape")

	## Live rotation into portrait: `set_portrait()`'s own walk must still
	## apply to every still-valid entry, unchanged by this fix.
	DccTheme.set_portrait(true)
	for l in tablet_batch:
		_check(l.clip_text == true,
			"rotating into portrait live-elides an existing tablet header")
		_check(l.text_overrun_behavior == TextServer.OVERRUN_TRIM_ELLIPSIS,
			"...with OVERRUN_TRIM_ELLIPSIS")

	## Free 6 of the 10 (simulate a partial panel teardown), rotate back, and
	## confirm `set_portrait()`'s prune-on-rotation still removes exactly the
	## freed entries -- the mechanism this fix leaves untouched.
	for i in range(6):
		tablet_batch[i].free()
	DccTheme.set_portrait(false)
	_check(DccTheme._elide_labels.size() == 4,
		"prune-on-rotation still drops exactly the 6 freed tablet headers")
	for i in range(6, 10):
		var l: Label = tablet_batch[i]
		_check(l.clip_text == false,
			"the 4 surviving tablet headers un-elide on the rotation back")
	for i in range(6, 10):
		tablet_batch[i].free()
	DccTheme.set_portrait(true)
	_check(DccTheme._elide_labels.size() == 0,
		"the registry is empty again after the last tablet header frees and rotates")

	_log("RESULT fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
