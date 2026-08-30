//! The Landmark Generation dock's thin `Variant`-adjacent helpers over
//! `cartalith_civ::landmark` — `LANDMARK_UI_DESIGN.md` §9's wiring table.
//!
//! ## Almost everything here already lives in `cartalith_civ::landmark`
//!
//! That module's own doc states why: *"the panel must not own the
//! vocabulary, or the 49 keys, ranges and labels get hardcoded a second time
//! on the GDScript side and drift."* [`cartalith_civ::landmark::LandmarkStore`]
//! already bundles the settings a panel writes and the last run's result it
//! reads; [`cartalith_civ::landmark::LandmarkSettings`] already has
//! `Default`, `cap`/`is_armed`/`set_cap`/`set_armed`/`radius_km`; every enum
//! (`LandmarkClass`, `LandmarkFamily`, `LandmarkLimit`) already has its own
//! `as_str()`. `WorldGen` in `lib.rs` holds exactly one new field —
//! `landmark_store: cartalith_civ::landmark::LandmarkStore` — and the
//! `#[func]` surface there converts it to/from `Variant` types directly
//! against that crate's own accessors, not against anything reinvented here.
//!
//! ## What this module actually adds
//!
//! Two things the crate deliberately does **not** provide:
//!
//! 1. **Unknown-key rejection.** `LandmarkSettings::set_cap`/`set_armed`
//!    insert unconditionally — reasonable for a crate that trusts its
//!    caller, wrong for a boundary that takes a raw `String` off the wire.
//!    A typo'd key from GDScript must be refused, not silently turned into a
//!    fiftieth landmark type. [`set_cap`]/[`set_armed`] add that check via
//!    [`cartalith_civ::landmark::kind_spec`], which already exists for
//!    exactly this kind of lookup.
//! 2. **`class_key` string -> [`LandmarkClass`].** The crate has
//!    `LandmarkClass::as_str()`/`LandmarkClass::all()` but no reverse
//!    lookup (it never needs one internally); `landmark_set_class_radius`'s
//!    `class_key: String` argument does. [`class_from_key`] is that lookup,
//!    built from the same `as_str()`/`all()` pair so it can never disagree
//!    with them.
//!
//! Crowding is clamped to `[0, 3]`, not the `LANDMARK_UI_DESIGN.md` §4.1
//! dial copy's `0.25×..2.00×` — `LandmarkSettings::crowding`'s own doc
//! comment states the field's real valid range as `0..3` (`ScatterRule::
//! density`'s own range, "Higher packs tighter"), which is what this bridge
//! defers to; the UI's dial travel is a separate, later decision that can
//! narrow the *slider*, not the *value the engine accepts*.
//!
//! ## Style
//!
//! Deliberately **free of any `godot` dependency**, the isolation
//! `civ_roster_bridge.rs` and `params.rs` both argue for. `lib.rs` owns the
//! `WorldGen` field, the `Variant`/Dictionary conversion, the `LandmarkInputs`
//! assembly, and the `#[func]` surface itself.

use cartalith_civ::landmark::{kind_spec, LandmarkClass, LandmarkSettings};

/// [`LandmarkSettings::crowding`]'s own documented range.
pub const CROWDING_MIN: f64 = 0.0;
pub const CROWDING_MAX: f64 = 3.0;

/// Writes one type's cap. Returns `false`, changing nothing, for a key
/// [`cartalith_civ::landmark::kinds`] does not carry — a typo from GDScript
/// is rejected, never silently turned into a fiftieth type. Floored at zero;
/// negative caps have no meaning.
pub fn set_cap(settings: &mut LandmarkSettings, key: &str, v: i64) -> bool {
    if kind_spec(key).is_none() {
        return false;
    }
    settings.set_cap(key, v.max(0) as u32);
    true
}

/// Arms/disarms one type. Same unknown-key rejection as [`set_cap`].
pub fn set_armed(settings: &mut LandmarkSettings, key: &str, on: bool) -> bool {
    if kind_spec(key).is_none() {
        return false;
    }
    settings.set_armed(key, on);
    true
}

/// Always succeeds — clamped to [`CROWDING_MIN`]..[`CROWDING_MAX`] (and to
/// `1.0`, the field's own default, for a non-finite input — the same
/// NaN-hazard discipline `LandmarkSettings::radius_km`'s own doc comment
/// states for the same field). There is no value this can reject.
pub fn set_crowding(settings: &mut LandmarkSettings, v: f64) -> bool {
    settings.crowding = if v.is_finite() { v.clamp(CROWDING_MIN, CROWDING_MAX) } else { 1.0 };
    true
}

/// The reverse of [`LandmarkClass::as_str`] — see this module's own doc for
/// why the crate does not carry this lookup itself.
fn class_from_key(key: &str) -> Option<LandmarkClass> {
    LandmarkClass::all().into_iter().find(|c| c.as_str() == key)
}

/// Writes one class's radius. Returns `false` for a `class_key` outside
/// `"continental"`/`"regional"`/`"local"`/`"cultural"`. Floored at zero km
/// (and at a non-finite input) — a negative or NaN separation has no
/// meaning and would corrupt [`LandmarkSettings::radius_km`]'s own division.
pub fn set_class_radius(settings: &mut LandmarkSettings, class_key: &str, km: f64) -> bool {
    let Some(class) = class_from_key(class_key) else { return false };
    settings.class_radius_km[class.index()] = if km.is_finite() { km.max(0.0) } else { 0.0 };
    true
}

/// Always succeeds — a plain boolean has nothing to reject.
pub fn set_cross_competition(settings: &mut LandmarkSettings, on: bool) -> bool {
    settings.cross_type_competition = on;
    true
}

/// [`cartalith_civ::landmark::LandmarkInputs::settlements`]'s own element —
/// this pass reads three numbers off a settlement, not the whole record
/// (see that field's own doc comment). Pure mapping, kept here so
/// `landmark_run()` in `lib.rs` reads as assembly rather than conversion.
pub fn settlement_to_site(s: &cartalith_civ::NamedSettlement) -> cartalith_civ::landmark::LandmarkSite {
    cartalith_civ::landmark::LandmarkSite {
        x: s.placement.x,
        y: s.placement.y,
        population: s.pop as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_cap_rejects_an_unknown_key_without_creating_it() {
        let mut s = LandmarkSettings::default();
        let n = s.caps.len();
        assert!(!set_cap(&mut s, "not_a_real_kind", 5), "an unknown key must be refused");
        assert_eq!(s.caps.len(), n, "a rejected key changes nothing, including its own count");
    }

    #[test]
    fn set_cap_writes_a_known_key_and_floors_negative_at_zero() {
        let mut s = LandmarkSettings::default();
        let key = s.caps.keys().next().cloned().expect("kinds() must not be empty");
        assert!(set_cap(&mut s, &key, 12));
        assert_eq!(s.caps[&key], 12);
        assert!(set_cap(&mut s, &key, -5), "a known key is still accepted, just clamped");
        assert_eq!(s.caps[&key], 0);
    }

    #[test]
    fn set_armed_rejects_an_unknown_key_and_writes_a_known_one() {
        let mut s = LandmarkSettings::default();
        assert!(!set_armed(&mut s, "not_a_real_kind", true));
        let key = s.armed.keys().next().cloned().expect("kinds() must not be empty");
        assert!(set_armed(&mut s, &key, false));
        assert_eq!(s.armed[&key], false);
        assert!(set_armed(&mut s, &key, true));
        assert_eq!(s.armed[&key], true);
    }

    #[test]
    fn crowding_clamps_to_the_documented_range_and_survives_non_finite_input() {
        let mut s = LandmarkSettings::default();
        assert!(set_crowding(&mut s, 10.0));
        assert_eq!(s.crowding, CROWDING_MAX);
        assert!(set_crowding(&mut s, -1.0));
        assert_eq!(s.crowding, CROWDING_MIN);
        assert!(set_crowding(&mut s, 1.5));
        assert_eq!(s.crowding, 1.5);
        assert!(set_crowding(&mut s, f64::NAN));
        assert_eq!(s.crowding, 1.0, "a non-finite write falls back to the field's own default");
    }

    #[test]
    fn class_radius_rejects_an_unknown_class_and_writes_the_right_slot() {
        let mut s = LandmarkSettings::default();
        let before = s.class_radius_km;
        assert!(!set_class_radius(&mut s, "continentalx", 10.0));
        assert_eq!(s.class_radius_km, before, "a rejected class changes nothing");
        assert!(set_class_radius(&mut s, "local", 9.5));
        assert_eq!(s.class_radius_km[LandmarkClass::Local.index()], 9.5);
        assert!(set_class_radius(&mut s, "continental", -3.0), "still accepted, just floored");
        assert_eq!(s.class_radius_km[LandmarkClass::Continental.index()], 0.0);
    }

    #[test]
    fn cross_competition_always_succeeds() {
        let mut s = LandmarkSettings::default();
        assert!(set_cross_competition(&mut s, true));
        assert!(s.cross_type_competition);
        assert!(set_cross_competition(&mut s, false));
        assert!(!s.cross_type_competition);
    }

    #[test]
    fn settlement_to_site_maps_position_and_population() {
        use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};
        let s = NamedSettlement {
            tid: 1,
            placement: SettlementPlacement {
                x: 7,
                y: 9,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: SettlementKind::Town,
                coastal: false,
            },
            name: "Test".to_string(),
            pop: 4200,
        };
        let site = settlement_to_site(&s);
        assert_eq!(site.x, 7);
        assert_eq!(site.y, 9);
        assert_eq!(site.population, 4200.0);
    }
}
