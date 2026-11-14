//! User-placed points of interest -- `CITY_MAP_LOOK_SCOPE.md` CL-11, Ruling EW item 4
//! (*"Yes, user-placed POIs"*, owner, 2026-11-13), which **reverses CV-01**, the earlier decision that POIs are not a ported
//! concept.
//!
//! A [`Poi`] is a tavern, a shop, a smithy... that the **user** places at a map point and names. It is not generated and no
//! engine pass reads it: nothing here may feed generation, settlement siting, the economy or any golden. It exists to be
//! drawn as a teardrop pin, picked, moved, renamed, deleted (all undoable through the annotation ledger) and bound to a
//! vault note.
//!
//! ## Identity: a stored id, not a position
//!
//! A label and an icon are identified by their cell (`label_entity_id`), so a move has to re-point their note. A POI instead
//! carries a stable `id` issued by the store when it is placed and **never reissued** (`WorldGen::poi_next_id` only grows and
//! is saved), so a move or a rename leaves its note bound, two POIs may stand on one cell, and a deleted POI's note link is an
//! absent entity that no later POI can inherit.
//!
//! ## The kind list
//!
//! [`PoiKind`] is the owner's list (tavern or inn, shop, smithy, temple, shrine, guild, gate, well, ruin, camp, mine, other).
//! Each has a stable wire `key()` (saved), a display `label()` and the pin pictogram the shell draws (`shell/pin_marker.gd`,
//! `KIND_PICTOGRAM`). **`other` is the stated fallback, never a default for an unreadable saved key**: a saved key this build
//! does not know is refused by [`PoiKind::from_key`] and the row is dropped with a warning, not turned into `other`.

/// The kinds of point of interest the Place tool offers, in the order the picker lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoiKind {
    /// A tavern or inn (pictogram: a mug).
    Tavern,
    /// A shop or market stall.
    Shop,
    /// A smithy or forge.
    Smithy,
    /// A temple.
    Temple,
    /// A shrine.
    Shrine,
    /// A guildhall.
    Guild,
    /// A gate.
    Gate,
    /// A well.
    Well,
    /// A ruin.
    Ruin,
    /// A camp.
    Camp,
    /// A mine.
    Mine,
    /// Anything else; drawn with the generic round dot.
    Other,
}

impl PoiKind {
    /// Every kind in the picker's order.
    pub const ALL: [PoiKind; 12] = [
        PoiKind::Tavern,
        PoiKind::Shop,
        PoiKind::Smithy,
        PoiKind::Temple,
        PoiKind::Shrine,
        PoiKind::Guild,
        PoiKind::Gate,
        PoiKind::Well,
        PoiKind::Ruin,
        PoiKind::Camp,
        PoiKind::Mine,
        PoiKind::Other,
    ];

    /// The saved key (`annotations/pois.json`, the shell's kind argument). Stable: renaming one orphans saved POIs.
    pub fn key(self) -> &'static str {
        match self {
            PoiKind::Tavern => "tavern",
            PoiKind::Shop => "shop",
            PoiKind::Smithy => "smithy",
            PoiKind::Temple => "temple",
            PoiKind::Shrine => "shrine",
            PoiKind::Guild => "guild",
            PoiKind::Gate => "gate",
            PoiKind::Well => "well",
            PoiKind::Ruin => "ruin",
            PoiKind::Camp => "camp",
            PoiKind::Mine => "mine",
            PoiKind::Other => "other",
        }
    }

    /// The words the picker and a default name use ("Tavern or inn", not the key).
    pub fn label(self) -> &'static str {
        match self {
            PoiKind::Tavern => "Tavern or inn",
            PoiKind::Shop => "Shop",
            PoiKind::Smithy => "Smithy",
            PoiKind::Temple => "Temple",
            PoiKind::Shrine => "Shrine",
            PoiKind::Guild => "Guild",
            PoiKind::Gate => "Gate",
            PoiKind::Well => "Well",
            PoiKind::Ruin => "Ruin",
            PoiKind::Camp => "Camp",
            PoiKind::Mine => "Mine",
            PoiKind::Other => "Other",
        }
    }

    /// The kind a saved or passed key names, or `None` for a key this build does not know (**never `Other`**: an unknown key
    /// is a refusal, not a guess, so a newer file's kind is reported and not silently relabelled).
    pub fn from_key(key: &str) -> Option<PoiKind> {
        PoiKind::ALL.iter().copied().find(|k| k.key() == key)
    }
}

/// One user-placed point of interest.
#[derive(Debug, Clone, PartialEq)]
pub struct Poi {
    /// The stable id the store issued (`>= 1`; `0` is never issued, so a missing id cannot read as the first POI).
    pub id: u32,
    /// Grid x in continuous cell units (a click's coordinate, as a hand-placed icon's is).
    pub x: f64,
    /// Grid y, as [`Poi::x`].
    pub y: f64,
    /// What kind of place it is.
    pub kind: PoiKind,
    /// The name the user gave it; may be empty (an unnamed POI is shown by its kind, never by an invented name).
    pub name: String,
}

/// The next id after `pois` for a store whose counter is `next_id`: the larger of the counter and one past the largest
/// stored id, so a counter that lags a loaded list (an older or hand-edited file) can never reissue a live id. `None` when
/// the id space is exhausted (a stored id of `u32::MAX`): the caller refuses the placement rather than wrapping onto a live
/// id. The store stores `result + 1` as its new counter.
pub fn issue_id(pois: &[Poi], next_id: u32) -> Option<u32> {
    let past_max = match pois.iter().map(|p| p.id).max() {
        None => 1,
        Some(m) => m.checked_add(1)?,
    };
    Some(next_id.max(past_max).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> Poi {
        Poi { id, x: 1.0, y: 2.0, kind: PoiKind::Well, name: String::new() }
    }

    /// Protects: the kind list is the owner's twelve, every key round-trips and is unique, and an unknown key is a refusal
    /// (never `Other`).
    #[test]
    fn kinds_round_trip_and_an_unknown_key_is_refused() {
        assert_eq!(PoiKind::ALL.len(), 12);
        let keys: Vec<&str> = PoiKind::ALL.iter().map(|k| k.key()).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 12, "keys must be unique: {keys:?}");
        for k in PoiKind::ALL {
            assert_eq!(PoiKind::from_key(k.key()), Some(k));
        }
        // The owner's list, literally and in order: a renamed key would orphan every saved POI of that kind.
        assert_eq!(keys, ["tavern", "shop", "smithy", "temple", "shrine", "guild", "gate", "well", "ruin", "camp", "mine", "other"]);
        assert_eq!(PoiKind::from_key("tavern"), Some(PoiKind::Tavern));
        assert_eq!(PoiKind::from_key("brothel"), None);
        assert_eq!(PoiKind::from_key(""), None);
    }

    /// Protects: an id is never reissued and never `0`, even when the counter lags the list.
    #[test]
    fn issue_id_never_reuses_a_live_id_or_zero() {
        assert_eq!(issue_id(&[], 0), Some(1));
        assert_eq!(issue_id(&[], 1), Some(1));
        assert_eq!(issue_id(&[p(1), p(2)], 1), Some(3), "a counter behind the list is lifted past the largest id");
        assert_eq!(issue_id(&[p(1)], 7), Some(7), "a counter ahead of the list wins: a deleted id stays retired");
        assert_eq!(issue_id(&[p(u32::MAX)], 1), None, "an exhausted id space refuses rather than reissuing a live id");
    }
}
