extends Node
## Two world-replacement identity defects, both pre-existing, both fixed
## 2026-09-03. Run:
##
##   godot --headless --path . _worldswap_probe.tscn
##
## A1 -- `import_heightmap` replaces the world by calling `absorb()` and
## nothing else (it cannot call `release_world()` first: an unreadable file
## must leave the previous world untouched). `release_world()` was the only
## path clearing `vault.store.links`/`.snapshots`, so knowledge links and map
## snapshots taken against the OUTGOING world survived into the imported one,
## pointing at settlements that no longer existed -- and
## `project_save_with_documents` would then have written them into the
## imported world's `vault.json`. The clear now lives in `absorb()`, which is
## the funnel all four replacement paths share.
##
## A2 -- `bake_bridge::world_key_signature` hashed only generation *inputs*.
## An imported heightmap, a region resample and a generated world at the same
## parameter tuple therefore shared one atlas namespace, and one would read
## another's baked tiles. The signature now carries how the field was
## produced.
##
## Rust-side, `bake_bridge.rs`'s own suite pins the signature. What only this
## probe can reach is whether the *bridge* wires it: a `#[func]` running
## against a real `WorldGen` over a real replacement.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## A link store with one link and one snapshot filed against world A, plus a
## device vault binding that must NOT be dropped by a world replacement.
## Injected through `vault_restore_state` rather than built with
## `vault_attach`/`vault_snapshot`, so the probe needs no vault on disk and no
## render pass -- what is under test is the clear, not how the entries got in.
const STORE_A := """{
  "version": 1,
  "vaults": [{"id": "v1", "display_name": "Probe vault"}],
  "links": [{
    "link_id": "L1",
    "entity_kind": "settlement",
    "entity_id": 1,
    "entity_label": "Aldermoor",
    "vault_id": "v1",
    "relative_path": "places/aldermoor.md",
    "selection": {"type": "whole_document"},
    "imported_text": "# Aldermoor\\n"
  }],
  "snapshots": {"settlement:1|local": "maps/aldermoor-local.png"}
}"""

func _seed_store(wg: Object) -> bool:
	if not wg.vault_restore_state(STORE_A):
		print("  [FATAL] vault_restore_state refused the fixture -- the store schema moved")
		return false
	return true

func _snapshot_count(wg: Object) -> int:
	var j: Variant = JSON.parse_string(String(wg.vault_state_json()))
	if typeof(j) != TYPE_DICTIONARY:
		return -1
	return (j.get("snapshots", {}) as Dictionary).size()

func _vault_count(wg: Object) -> int:
	var j: Variant = JSON.parse_string(String(wg.vault_state_json()))
	if typeof(j) != TYPE_DICTIONARY:
		return -1
	return (j.get("vaults", []) as Array).size()

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	## World A, and a heightmap on disk to import over it. Small grids: this
	## probe is about identity, not fidelity.
	var wg: Object = ClassDB.instantiate("WorldGen")
	wg.generate_sized(24601, 640.0, 192, 96)
	var png := OS.get_user_data_dir() + "/_worldswap.png"
	var ex: Dictionary = wg.export_heightmap_png(png, 2048)
	if not bool(ex.get("ok", false)):
		print("[FATAL] could not write the probe heightmap: ", ex.get("error", ""))
		get_tree().quit(1); return
	print("[SETUP] world A 192x96 seed 24601; heightmap at ", png)

	print("\n=== A1: the fixture really is in the store ===")
	if not _seed_store(wg):
		get_tree().quit(1); return
	_ok("one link against world A", (wg.vault_all_links() as Array).size(), 1)
	_ok("one snapshot against world A", _snapshot_count(wg), 1)
	_ok("one device vault binding", _vault_count(wg), 1)

	print("\n=== A1: import_heightmap does not carry them into world B ===")
	## The defect path exactly: `absorb` without `release_world`.
	_ok("the import succeeded", wg.import_heightmap(png, 777, 400.0, 128), true)
	_ok("links did not survive the replacement", (wg.vault_all_links() as Array).size(), 0)
	_ok("snapshots did not survive it either", _snapshot_count(wg), 0)
	## The half that must NOT be cleared: `vaults` is the device's binding
	## registry, not world state. Clearing the whole store would unbind the
	## user's vault on every import -- a fix worse than the defect.
	_ok("the device vault binding survived", _vault_count(wg), 1)

	print("\n=== A1: the other three replacement paths agree ===")
	## `generate_sized` and `generate_world_structure_sized` reach the clear
	## through `release_world` AND `absorb`; asserting them here is what keeps
	## the three paths one answer rather than three.
	var wg2: Object = ClassDB.instantiate("WorldGen")
	wg2.generate_sized(24601, 640.0, 192, 96)
	if not _seed_store(wg2): get_tree().quit(1); return
	wg2.generate_sized(31337, 640.0, 192, 96)
	_ok("generate_sized clears links", (wg2.vault_all_links() as Array).size(), 0)
	_ok("generate_sized clears snapshots", _snapshot_count(wg2), 0)
	_ok("generate_sized keeps the vault binding", _vault_count(wg2), 1)

	var wg3: Object = ClassDB.instantiate("WorldGen")
	wg3.generate_sized(24601, 640.0, 192, 96)
	if not _seed_store(wg3): get_tree().quit(1); return
	_ok("the archetype generate ran",
		wg3.generate_world_structure_sized(31337, 640.0, 192, 96, "archipelago"), true)
	_ok("generate_world_structure_sized clears links",
		(wg3.vault_all_links() as Array).size(), 0)
	_ok("generate_world_structure_sized clears snapshots", _snapshot_count(wg3), 0)

	var wg4: Object = ClassDB.instantiate("WorldGen")
	wg4.generate_sized(24601, 640.0, 192, 96)
	if not _seed_store(wg4): get_tree().quit(1); return
	## `region_set` takes CELLS, not fractions.
	wg4.region_set(40.0, 20.0, 96.0, 48.0)
	var r4: bool = wg4.region_new_world(256, false, 0.0, 0.0)
	if not r4:
		print("  reason: ", wg4.region_new_world_error())
	_ok("the region resample ran", r4, true)
	_ok("region_new_world clears links", (wg4.vault_all_links() as Array).size(), 0)
	_ok("region_new_world clears snapshots", _snapshot_count(wg4), 0)

	print("\n=== A2: three origins at ONE parameter tuple are three atlases ===")
	## A *controlled* collision, not three worlds that happen to differ.
	## Every element of the world key except the origin is pinned equal here
	## by construction, and each one is asserted rather than assumed:
	##
	##   grid          160 x 80 -- asserted on all three below
	##   seed          4242     -- passed to the generate and the import,
	##                             inherited by the resample from its parent
	##   map_width_km  512.0    -- passed; a full-extent resample keeps it
	##   sea_level     default  -- `region_as_new_world` sets `p.sea_level =
	##                             opts.sea`, which is the parent's own
	##   wrap + params defaults -- none of the three touches `set_params`
	##
	## Before the origin element, all three hashed to one string.
	var a: Object = ClassDB.instantiate("WorldGen")
	a.generate_sized(4242, 512.0, 160, 80)
	var ref := OS.get_user_data_dir() + "/_worldswap_ref.png"
	## 2048 wide over a 2:1 grid is a 2048x1024 image (512 is refused --
	## the exporter offers 2048/4096/8192), so the import's own
	## `GH = max(80, round(GW / aspect))` lands back on 160x80.
	var ex2: Dictionary = a.export_heightmap_png(ref, 2048)
	_ok("the reference heightmap was written", ex2.get("ok", false), true)
	_ok("it is the 2:1 image the import needs", ex2.get("height"), 1024)
	var gen_key := String(a.atlas_world_key())

	var b: Object = ClassDB.instantiate("WorldGen")
	_ok("the same-shape import ran", b.import_heightmap(ref, 4242, 512.0, 160), true)
	_ok("the import landed on the same grid",
		Vector2i(b.get_width(), b.get_height()), Vector2i(160, 80))
	_ok("on the same seed", b.get_seed(), 4242)
	_ok("and the same map width", b.get_map_width_km(), 512.0)
	var import_key := String(b.atlas_world_key())

	print("  info generated key = ", gen_key, "   imported key = ", import_key)
	_ok("neither key is empty", gen_key != "" and import_key != "", true)
	_ok("an import does not read a generated world's atlas namespace",
		gen_key != import_key, true)

	## Full-extent marquee at a tile size equal to the parent's own width, so
	## the resample comes back at the parent's grid, width and sea level --
	## the same tuple again, from the third door.
	var c: Object = ClassDB.instantiate("WorldGen")
	c.generate_sized(4242, 512.0, 160, 80)
	c.region_set(0.0, 0.0, 160.0, 80.0)
	var rc: bool = c.region_new_world(160, false, 0.0, 0.0)
	if not rc:
		print("  reason: ", c.region_new_world_error())
	_ok("the region resample for the key test ran", rc, true)
	_ok("the resample landed on the same grid",
		Vector2i(c.get_width(), c.get_height()), Vector2i(160, 80))
	_ok("on the same seed", c.get_seed(), 4242)
	_ok("and the same map width", c.get_map_width_km(), 512.0)
	var region_key := String(c.atlas_world_key())
	print("  info resample key = ", region_key)
	_ok("a resample does not read a generated world's atlas namespace",
		region_key != gen_key, true)
	_ok("a resample and an import are two namespaces", region_key != import_key, true)

	print("\n=== A2: the key is still stable for everything it was stable for ===")
	## The invalidation rule has to keep working in both directions, or the
	## discriminator has bought a collision fix by breaking the cache.
	var d: Object = ClassDB.instantiate("WorldGen")
	d.generate_sized(4242, 512.0, 160, 80)
	_ok("two identical generates share one key", String(d.atlas_world_key()), gen_key)
	d.generate_sized(4243, 512.0, 160, 80)
	_ok("a changed seed still changes the key",
		String(d.atlas_world_key()) != gen_key, true)

	## And the round trip the origin element must not break: save the
	## generated world, load it back, and the key has to be the one its atlas
	## was baked under. `load_save` is the one replacement path that does not
	## go through `absorb`, and it reports ORIGIN_GENERATED for exactly this.
	var zip := OS.get_user_data_dir() + "/_worldswap.zip"
	var e: Object = ClassDB.instantiate("WorldGen")
	e.generate_sized(4242, 512.0, 160, 80)
	_ok("the save was written", e.save_project(zip), true)
	var f: Object = ClassDB.instantiate("WorldGen")
	_ok("the save was read back", f.load_save(zip), true)
	_ok("reopening a generated world keeps its atlas addressable",
		String(f.atlas_world_key()), gen_key)

	DirAccess.remove_absolute(png)
	DirAccess.remove_absolute(ref)
	DirAccess.remove_absolute(zip)
	print("\n_worldswap_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
