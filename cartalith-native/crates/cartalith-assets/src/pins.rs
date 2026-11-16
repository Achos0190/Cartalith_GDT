//! The **Pins** category of the asset library (Ruling FP, `PIN_PALETTE_SCOPE.md` "Custom pins").
//!
//! A custom pin is not a new family: it is a slot of the existing open [`crate::Family::Custom`] vocabulary, in the one
//! reserved set [`PINS_SET`], so it is stored, listed, imported, sliced, exported and merged by the machinery every other
//! custom icon already uses, and a pack that carries one needs nothing a v2.10 reader must understand (it sees a custom set
//! it ignores). What a custom *icon* lacks and a pin needs is a colour and a name that survives a round trip, which are:
//!
//! - the **name** is the slot's display name (`LibrarySlot::name`), the colour is [`crate::library::SlotMeta::color`];
//! - in a pack both travel in the additive top-level `pins` section of `pack.json`
//!   (`{"<slot id>": {"name": "...", "color": "#rrggbb"}}`, [`crate::manifest::RawPin`]), keyed by the slot's id in
//!   `custom.Pins`. A pack with no pins writes no such key, so every pack written before this existed is byte-identical.
//!
//! Must never: let a colour that is not `#rrggbb` or a name that is empty reach a library or a pack; both are refused.

/// The reserved custom set a pin lives in. Case-sensitive, as every custom set name is (`AssetDB::add_custom_slot`).
pub const PINS_SET: &str = "Pins";

/// The longest pin name kept, in characters; equal to `cartalith_civ::poi::CUSTOM_NAME_MAX` (the engine's copy, which a
/// `cartalith-godot` test compares with this one: the two crates do not depend on each other).
pub const PIN_NAME_MAX: usize = 40;

/// The colour a pin shows when it has none: a neutral grey, never a colour that would pass for a category's (a labelled
/// judgement; the palette's eight category colours are saturated).
pub const PIN_DEFAULT_COLOUR: &str = "#8a8f98";

/// Whether `c` is `#rrggbb` (either case). The same rule as `cartalith_civ::poi::is_pin_colour`.
pub fn is_pin_colour(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

/// `raw` trimmed and cut to [`PIN_NAME_MAX`] characters, or `None` when nothing is left (a pin always has a name).
pub fn clean_pin_name(raw: &str) -> Option<String> {
    let t: String = raw.trim().chars().take(PIN_NAME_MAX).collect();
    let t = t.trim().to_string();
    (!t.is_empty()).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the colour and name rules, on literals.
    #[test]
    fn colours_and_names_are_validated_not_repaired() {
        assert!(is_pin_colour("#8a8f98") && is_pin_colour("#FFFFFF") && is_pin_colour(PIN_DEFAULT_COLOUR));
        for bad in ["", "8a8f98", "#8a8f9", "#8a8f988", "#8a8f9g", "rgb(1,2,3)", " #8a8f98"] {
            assert!(!is_pin_colour(bad), "{bad:?}");
        }
        assert_eq!(clean_pin_name("  Beehive \n").as_deref(), Some("Beehive"));
        assert_eq!(clean_pin_name("   "), None);
        assert_eq!(clean_pin_name(&"x".repeat(60)).unwrap().chars().count(), 40);
    }
}
