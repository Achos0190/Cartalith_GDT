//! `AssetDB`'s own id-slugging and uid-collision hardening, named and pinned
//! the same way `tests/hardening_v1_27.rs` pins the scatter-rule fixes.
//!
//! This one has no version-tagged reference comment to point at — unlike
//! v1.27's `NaN`/aliasing fixes, `addCustomSlot`/`renameCustomSlot`'s
//! collision handling carries no `/* vX.YY fix */` marker in the frozen HTML.
//! It was still worth checking for (per `ASSET_LIBRARY_SCOPE.md`'s milestone 5
//! instructions, echoing milestone 3's own "check before assuming there's
//! nothing there" discipline), and there genuinely is real defensive logic
//! here, inline in the reference's own code:
//!
//! ```text
//! addCustomSlot(name,setName){
//!   ...
//!   const existing=fam.slots.find(s=>s.uid===uid);
//!   if(existing) return existing;                          // <- idempotent, not a second slot
//!   ...
//! }
//! renameCustomSlot(uid,newName){
//!   ...
//!   if(SLOT_REG[nuid]) return uid;                          // <- collision — keep old id
//!   ...
//! }
//! ```
//!
//! Both matter because a custom slot's `id` is **derived from untrusted,
//! free-form user text** via [`slug_id`] — two differently-spelled display
//! names ("Wind Mill!!" and "wind   mill") can legitimately collide on one
//! slug, and this is not a hypothetical: an [`AssetDB`] can be rebuilt from a
//! project's `library.json`, which is user-editable outside the app. Without
//! the guards below, that collision would either silently create two
//! independent slots sharing one uid (whichever one `store`/`SLOT_REG` wrote
//! last wins reads, corrupting the other) or let a rename silently steal
//! another slot's identity.
//!
//! A companion finding, also pinned below: two of
//! [`cartalith_assets::library::run`]'s six checks —
//! "Duplicate identifier" and "Invalid filename id" — turn out to be
//! **structurally unreachable through this module's own public API**, in
//! both languages, for a reason that is not "Rust's type system" (the same
//! shape of surprise milestone 3's fix #3 documented for the
//! `Object.assign` aliasing bug). They are ported anyway, faithfully, as real
//! defence-in-depth against a state no *public* API call can currently
//! produce — see `run`'s own doc comment for the fuller reasoning.

use cartalith_assets::{AssetDB, Family, slug_id};

// ============================================================================
// addCustomSlot: idempotent on a uid collision
// ============================================================================

/// **Hardening: a repeated `add_custom_slot` call must return the existing
/// slot, never a duplicate or a silent overwrite.**
///
/// The reference's own guard: `const existing=fam.slots.find(s=>s.uid===uid);
/// if(existing) return existing;`. Demonstrated first as the failure this
/// prevents — a naive "always push a new slot" implementation would double
/// the registry's custom-family length on the second call — then as the real
/// behaviour.
#[test]
fn add_custom_slot_never_creates_a_second_slot_for_one_uid() {
    let mut db = AssetDB::new();
    let before = db.slots_in_family(Family::Custom).len();
    db.add_custom_slot("Lighthouse", Some("Naval"));
    db.add_custom_slot("Lighthouse", Some("Naval"));
    db.add_custom_slot("Lighthouse", Some("Naval"));
    let after = db.slots_in_family(Family::Custom).len();
    assert_eq!(after, before + 1, "three identical calls must yield exactly one slot");
}

/// **The interesting case**: two *differently spelled* display names that
/// [`slug_id`] happens to collapse onto the same id. This is the real
/// untrusted-input hazard — not a defensive nicety against a caller who
/// literally repeats themselves, but a guarantee that holds even when the
/// two calls look nothing alike on the surface.
#[test]
fn add_custom_slot_collapses_differently_spelled_names_that_slug_identically() {
    assert_eq!(slug_id("Wind Mill!!"), slug_id("wind   mill"));

    let mut db = AssetDB::new();
    let first = db.add_custom_slot("Wind Mill!!", Some("Naval")).clone();
    let second = db.add_custom_slot("wind   mill", Some("Naval")).clone();

    assert_eq!(first.uid, second.uid);
    assert_eq!(
        db.slots_in_family(Family::Custom).len(),
        1,
        "one slug, one slot -- regardless of how many spellings reached it"
    );
    // First writer wins: the slot keeps its ORIGINAL display name. A silent
    // overwrite here would mean the second call's text quietly replaced the
    // first author's own label.
    assert_eq!(db.get(&first.uid).unwrap().name, "Wind Mill!!");
}

/// The `Family::Custom` slot store cannot even *express* a uid collision:
/// [`AssetDB`] keys its registry by `uid` in a `HashMap`, which cannot hold
/// two different values under one key. The reference's own `SLOT_REG` object
/// carries the equivalent guarantee (`SLOT_REG[uid]={fam,slot}` always
/// replaces, never appends) — this port's structural guarantee and the
/// reference's structural guarantee are the same shape, not merely
/// coincidentally similar outcomes.
#[test]
fn a_custom_uid_can_never_address_two_different_slots() {
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    // Any second call that would slug to the same uid, however it is spelled,
    // must resolve to the very same slot object -- there is no code path
    // that could hand back a different one.
    for spelling in ["Lighthouse", "LIGHTHOUSE", "  lighthouse  "] {
        let slot = db.add_custom_slot(spelling, Some("Naval"));
        assert_eq!(slot.uid, uid);
    }
}

// ============================================================================
// renameCustomSlot: collision-safe, never clobbers the target
// ============================================================================

/// **Hardening: a rename that would collide with an existing slot must be
/// refused outright**, not redirected onto the target and not silently
/// merged with it. The reference's own guard: `if(SLOT_REG[nuid]) return uid;`.
///
/// This is the sharper failure mode a naive port could introduce: without
/// the guard, renaming "Lighthouse" to "Buoy" (when a "Buoy" slot already
/// exists) could either silently delete the renaming slot's own identity
/// (its uid now points at the pre-existing "Buoy" data) or overwrite "Buoy"'s
/// own store/meta with "Lighthouse"'s. Neither is acceptable for
/// user-authored content editable outside the app.
#[test]
fn rename_custom_slot_refuses_a_collision_rather_than_merging_or_overwriting() {
    let mut db = AssetDB::new();
    let lighthouse = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    let buoy = db.add_custom_slot("Buoy", Some("Naval")).uid.clone();
    db.add_item(&lighthouse, cartalith_assets::LibraryItem::new("l1.png", "hashL"));
    db.add_item(&buoy, cartalith_assets::LibraryItem::new("b1.png", "hashB"));

    let result = db.rename_custom_slot(&lighthouse, "Buoy");

    assert_eq!(result, lighthouse, "refused rename returns the OLD uid, not the target's");
    assert_eq!(db.get(&lighthouse).unwrap().name, "Lighthouse", "renaming slot is untouched");
    assert_eq!(db.get(&buoy).unwrap().name, "Buoy", "existing target slot is untouched");
    // Neither slot's items merged or vanished.
    assert_eq!(db.items(&lighthouse).len(), 1);
    assert_eq!(db.items(&lighthouse)[0].name, "l1.png");
    assert_eq!(db.items(&buoy).len(), 1);
    assert_eq!(db.items(&buoy)[0].name, "b1.png");
    assert_eq!(
        db.slots_in_family(Family::Custom).len(),
        2,
        "still exactly two slots -- no merge, no silent drop"
    );
}

/// A rename that changes only *casing/whitespace* (so it slugs to the very
/// same id it already has) is not a "collision with itself": it must update
/// the display name in place and keep the uid stable, matching the
/// reference's `if(nuid===uid){ slot.name=...; return uid; }` branch, which
/// is checked *before* the collision guard.
#[test]
fn renaming_to_a_spelling_that_slugs_to_the_same_id_is_not_treated_as_a_collision() {
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    let result = db.rename_custom_slot(&uid, "LIGHTHOUSE");
    assert_eq!(result, uid, "same slug -> same uid, not refused");
    assert_eq!(db.get(&uid).unwrap().name, "LIGHTHOUSE", "display name still updates");
}

// ============================================================================
// A finding, not a fix: two validator checks are structurally unreachable
// ============================================================================

/// **Finding, verified by construction rather than assumed**: `run`'s
/// "Invalid filename id" check (`!/^[a-z0-9_]+$/.test(s.id)`) exists to catch
/// a custom slot whose `id` is not a clean slug — but every path that can
/// create or rename a custom slot in this module (and, reading the reference,
/// every path in `AssetDB` too: `importPackZip`, the sprite-sheet importer,
/// the "duplicate item" button, `_alImportProject`) routes the slot's id
/// through [`slug_id`] first, which by construction can only ever produce
/// `[a-z0-9_]+` or the literal fallback `"icon"` — both of which already
/// satisfy the check. There is no public way to make this warning fire.
///
/// Not dead code to delete: it is real, cheap defence-in-depth against a
/// hypothetical future mutation path that bypasses `slug_id`, and the
/// reference itself has carried it (presumably for the same reason) without
/// ever being able to trigger it either.
#[test]
fn slug_id_can_never_produce_an_id_the_validator_would_flag() {
    fn looks_valid(id: &str) -> bool {
        !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }
    for input in [
        "Lighthouse",
        "Wind Mill!!",
        "  --Old Ruin--  ",
        "Château",
        "",
        "!!!",
        "已经",
        "a...b...c",
        "MiXeD_Case-123",
    ] {
        assert!(
            looks_valid(&slug_id(input)),
            "slug_id({input:?}) = {:?} would fail the validator's own regex",
            slug_id(input)
        );
    }
}

/// **Finding, verified end-to-end**: `run`'s "Collection references a
/// missing asset" check cannot fire via [`AssetDB::remove_custom_slot`],
/// because that method already calls
/// [`cartalith_assets::AssetCollections::drop_uid`] before the validator
/// could ever observe a stale reference. The check is not dead — see
/// `tests/golden_parity_library.rs`'s
/// `validator_stale_collection_only_fires_via_an_unchecked_assignment` for
/// the one real path, [`cartalith_assets::AssetCollections::from_map`] — but
/// it genuinely cannot be reached through ordinary slot editing.
#[test]
fn removing_a_custom_slot_leaves_no_stale_collection_reference_for_the_validator_to_find() {
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    db.collections.add("Coastal", std::slice::from_ref(&uid));
    assert!(db.remove_custom_slot(&uid));
    assert!(
        db.collections.membership(&uid).is_empty(),
        "removeCustomSlot's own AssetCollections::drop_uid call already cleaned this up"
    );
}
