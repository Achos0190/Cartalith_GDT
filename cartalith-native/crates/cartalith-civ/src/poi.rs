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
//! ## Kind names (Ruling FD, `PIN_PALETTE_SCOPE.md` PP-2)
//!
//! A [`Poi`] stores its kind as a **name** (`Poi::kind`, a `String`): any icon name the pin palette offers
//! (`icons/pins/{glyph,painted}/<name>.png`: `tavern`, `capital`, `waterfall`, ...), not one of a fixed enum. The thirteen
//! [`PoiKind`]s below (twelve, plus `inn` since 2026-11-15) are now only the *legacy* list: every one of their keys is a valid name, so an old save's `kind`
//! loads as the same string, unchanged. The engine cannot read the shell's icon files at run time, so [`KIND_NAMES`] is a
//! copy of the union of both icon sets' file names (a test compares it with the folders, so a new icon without a row, or
//! a row without a file, fails the build), plus the legacy keys; a name outside both is **refused**, never turned into `other`, the same rule the
//! legacy kind key always had.
//!
//! A **free-text kind** is a `free_label` the user typed ("Beehives") with a mark: the kind name is `other` (a question
//! mark) or `notice` (an exclamation mark), the user's choice ([`FREE_MARKS`]). The label is shown by the card and Find
//! in place of the kind's words; it never replaces the icon, which stays that mark.
//!
//! ## The legacy kind list
//!
//! [`PoiKind`] is the owner's list (tavern, inn, shop, smithy, temple, shrine, guild, gate, well, ruin, camp, mine, other).
//! Each has a stable wire `key()` (saved), a display `label()` and the pin pictogram the shell draws (`shell/pin_marker.gd`,
//! `KIND_PICTOGRAM`). **`other` is the stated fallback, never a default for an unreadable saved key**: a saved key this build
//! does not know is refused by [`PoiKind::from_key`] and the row is dropped with a warning, not turned into `other`.

/// The kinds of point of interest the Place tool offers, in the order the picker lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoiKind {
    /// A tavern (pictogram: a mug).
    Tavern,
    /// An inn (pictogram: the half-timbered house with a hanging sign). **Added 2026-11-15** (`PIN_PALETTE_SCOPE.md` PP-7,
    /// owner: *"The one that is now there for tavern can de reused for an inn."*), after `cf430ea4` had made `inn` a valid
    /// kind name; it is placed beside [`PoiKind::Tavern`] so the picker keeps the two together. Its **wire form is unchanged**
    /// (see [`PoiKind::is_original_wire_key`]).
    Inn,
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
    pub const ALL: [PoiKind; 13] = [
        PoiKind::Tavern,
        PoiKind::Inn,
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
            PoiKind::Inn => "inn",
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

    /// The words the picker and a default name use ("Tavern", not the key).
    pub fn label(self) -> &'static str {
        match self {
            PoiKind::Tavern => "Tavern",
            PoiKind::Inn => "Inn",
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

    /// Whether `key` is one of the **twelve keys saves have always written** in `kind` (every legacy kind except
    /// [`PoiKind::Inn`]). The save writer puts only these in `kind`; an inn keeps the form `cf430ea4` gave it
    /// (`kind: "other"`, `kind_name: "inn"`), so a build from before this change still opens the row (as a question mark)
    /// instead of dropping it as an unknown key, and a save holding an inn is byte-identical to what the old path wrote.
    /// The reader accepts either form (`kind_name` wins), so nothing already saved changes meaning.
    pub fn is_original_wire_key(key: &str) -> bool {
        key != "inn" && PoiKind::from_key(key).is_some()
    }
}

/// Every kind name a pin may carry: the union of the file names in `godot-project/icons/pins/glyph/` and
/// `.../painted/` (62: `rockfall` is glyph-only, `hazard` and `inn` painted-only). Sorted. The legacy `shop` is not here
/// (no file); `is_valid_kind_name` accepts it separately. A test
/// (`kind_names_match_the_icon_folders`) reads both folders and fails when this list and the folders differ, so a pin
/// kind cannot exist without a picture or the reverse.
pub const KIND_NAMES: [&str; 62] = [
    "ancient_road", "battlefield", "border_marker", "bridge", "camp",
    "capital", "caravan_station", "castle", "cave", "ceremonial_site",
    "city", "cliff", "delta", "desert", "farmland",
    "ford", "forest", "fort", "fortified_crossing", "fortified_pass",
    "gate", "glacier", "gorge", "guild", "hamlet",
    "harbour", "hazard", "historic_crossing", "inn", "lake",
    "market", "mine", "monument", "mountain", "mountain_pass",
    "notice", "other", "portage", "quarry", "ridge",
    "river_confluence", "road_junction", "rock_formation", "rockfall", "ruin",
    "sacred_grove", "sacred_mountain", "salt_works", "shrine", "smithy",
    "spring", "tavern", "temple", "tomb", "town",
    "trade_depot", "village", "volcano", "watchtower", "waterfall",
    "well", "wildlife",
];

/// The longest free-text kind label stored, in characters. A labelled judgement: a place's kind in a few words.
pub const FREE_LABEL_MAX: usize = 60;

/// The two marks a free-text kind may draw: the question mark (`other`) or the exclamation mark (`notice`), the user's
/// choice (Ruling FD item 2). Any other name with a free label is refused.
pub const FREE_MARKS: [&str; 2] = ["other", "notice"];

/// Whether `name` is one of [`KIND_NAMES`] **or a legacy [`PoiKind::key`]** (`shop` has no icon file of its own, the
/// shell draws it as `market`, but a saved `shop` must keep loading). Exact, case-sensitive: `Tavern` is not `tavern`.
pub fn is_valid_kind_name(name: &str) -> bool {
    KIND_NAMES.contains(&name) || PoiKind::from_key(name).is_some() || is_custom_kind_id(name)
}

// ---------------------------------------------------------------------------------------------------------------------
// Custom pins (Ruling FP, `PIN_PALETTE_SCOPE.md` "Custom pins"): the user's own icons as placeable kinds.
// ---------------------------------------------------------------------------------------------------------------------

/// The prefix every custom kind id carries. **A hyphen never occurs in a built-in kind name** (all are `[a-z_]`), so a custom
/// id can never equal a built-in or a legacy key; a test walks both lists to keep that true.
pub const CUSTOM_KIND_PREFIX: &str = "custom-";

/// The longest custom pin name kept, in characters (a labelled judgement: the card and palette tooltip are one line).
pub const CUSTOM_NAME_MAX: usize = 40;

/// Whether `id` has the shape of a custom kind id: `custom-<slug>-<8 lower-case hex>`, the slug 1 to 32 characters of
/// `[a-z0-9_]`. **Shape only**: whether this world has a picture for it is [`CustomPin`]'s registry's business, and a
/// well-formed id with no picture is *kept* (drawn as the missing-icon marker), never turned into a built-in.
pub fn is_custom_kind_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix(CUSTOM_KIND_PREFIX) else { return false };
    let Some((slug, hash)) = rest.rsplit_once('-') else { return false };
    (1..=32).contains(&slug.len())
        && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        && hash.len() == 8
        && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Whether `c` is a colour the pin can carry: `#rrggbb`, lower- or upper-case hex. Anything else is refused, not repaired.
pub fn is_pin_colour(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

/// One custom pin a world uses: the kind id (content-addressed by the library, so an edited pin is a *new* id and a placed
/// pin keeps the picture it was placed with), the user's name and colour, and the icon as PNG bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPin {
    /// A [`is_custom_kind_id`] id.
    pub id: String,
    /// The user's name for the pin (at most [`CUSTOM_NAME_MAX`] characters, never empty).
    pub name: String,
    /// `#rrggbb` ([`is_pin_colour`]).
    pub color: String,
    /// The icon, a PNG (the shell draws it with its bottom-centre at the pin's point).
    pub png: Vec<u8>,
}

/// The longest icon a world keeps, in bytes: 128 x 128 RGBA PNGs are a few tens of kilobytes, so 256 KiB is generous and a
/// hostile save cannot make the project archive large through this slot (a labelled judgement).
pub const CUSTOM_PNG_MAX: usize = 256 * 1024;

/// Standard base64 (RFC 4648, padded). The world saves a custom pin's icon in `annotations/pois.json`, a JSON document, and
/// no base64 crate is in this workspace's tree; this is the 30 lines it would have cost to add one, tested both ways.
pub fn b64_encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

/// The inverse of [`b64_encode`]; `None` for anything that is not canonical padded base64 (a wrong length, a stray byte).
pub fn b64_decode(text: &str) -> Option<Vec<u8>> {
    let b = text.as_bytes();
    if b.len() % 4 != 0 {
        return None;
    }
    let val = |c: u8| -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some(u32::from(c - b'A')),
            b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
            b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let mut out = Vec::with_capacity(b.len() / 4 * 3);
    for (i, q) in b.chunks(4).enumerate() {
        let last = i == b.len() / 4 - 1;
        let pad = q.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return None;
        }
        let mut n = 0u32;
        for (k, &c) in q.iter().enumerate() {
            n = (n << 6) | if k >= 4 - pad { if c == b'=' { 0 } else { return None } } else { val(c)? };
        }
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Some(out)
}

/// The words for a kind name: the legacy [`PoiKind::label`] for the thirteen ("Tavern", "Inn"), else the name with `_`
/// turned to spaces and its first letter capitalised ("Mountain pass"). Never invents: an empty name gives an empty string.
pub fn kind_label_for(name: &str) -> String {
    if is_custom_kind_id(name) {
        // The words live in the world's own registry (`CustomPin::name`); the engine's callers that hold it pass those
        // instead. Never derive words from the slug: it is an id, and the user's name may differ from it.
        return "Custom pin".to_string();
    }
    if let Some(k) = PoiKind::from_key(name) {
        return k.label().to_string();
    }
    let spaced = name.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Trims a free label and caps it at [`FREE_LABEL_MAX`] characters; `None` for a label that is empty after trimming (an
/// empty string is not a label).
pub fn clean_free_label(raw: &str) -> Option<String> {
    let t: String = raw.trim().chars().take(FREE_LABEL_MAX).collect();
    let t = t.trim().to_string();
    (!t.is_empty()).then_some(t)
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
    /// What kind of place it is: a kind **name** (see [`is_valid_kind_name`]; a legacy [`PoiKind::key`] is one). Names the
    /// pin's icon.
    pub kind: String,
    /// The user's own words for the kind ("Beehives"), or empty. Non-empty only with a [`FREE_MARKS`] kind; empty is "none"
    /// (a free label is never an empty string).
    pub free_label: String,
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
        Poi { id, x: 1.0, y: 2.0, kind: "well".into(), free_label: String::new(), name: String::new() }
    }

    /// Protects: the kind list is the owner's twelve plus `inn`, every key round-trips and is unique, and an unknown key is a refusal
    /// (never `Other`).
    #[test]
    fn kinds_round_trip_and_an_unknown_key_is_refused() {
        assert_eq!(PoiKind::ALL.len(), 13);
        let keys: Vec<&str> = PoiKind::ALL.iter().map(|k| k.key()).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 13, "keys must be unique: {keys:?}");
        for k in PoiKind::ALL {
            assert_eq!(PoiKind::from_key(k.key()), Some(k));
        }
        // The owner's list, literally and in order: a renamed key would orphan every saved POI of that kind.
        assert_eq!(keys, ["tavern", "inn", "shop", "smithy", "temple", "shrine", "guild", "gate", "well", "ruin", "camp", "mine", "other"]);
        assert_eq!(PoiKind::from_key("inn"), Some(PoiKind::Inn));
        assert_eq!(PoiKind::from_key("tavern"), Some(PoiKind::Tavern));
        assert_eq!(PoiKind::from_key("brothel"), None);
        assert_eq!(PoiKind::from_key(""), None);
    }

    /// Protects: the name list is exactly the union of the two icon folders (a new icon needs a row, a row needs a file),
    /// is sorted and unique, and holds all legacy keys so an old save's kind is always a valid name.
    #[test]
    fn kind_names_match_the_icon_folders() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot-project/icons/pins");
        let mut files = std::collections::BTreeSet::new();
        for set in ["glyph", "painted"] {
            let dir = root.join(set);
            let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
            for e in entries {
                let name = e.unwrap().file_name().to_string_lossy().into_owned();
                if let Some(stem) = name.strip_suffix(".png") {
                    files.insert(stem.to_string());
                }
            }
        }
        assert_eq!(files.len(), 62, "the sets changed: update KIND_NAMES and the shell's PinPalette table");
        let listed: Vec<String> = KIND_NAMES.iter().map(|s| s.to_string()).collect();
        let on_disk: Vec<String> = files.into_iter().collect();
        assert_eq!(listed, on_disk);
        for k in PoiKind::ALL {
            assert!(is_valid_kind_name(k.key()), "legacy key {} must stay a valid name", k.key());
        }
    }

    /// Protects: names are matched exactly (no case folding, no trimming, no guessing), the words for a name, and the free
    /// label's trimming and cap.
    /// Protects (Ruling FP): the custom id shape. Literals only: a well-formed id passes and every malformed one is refused,
    /// and no built-in or legacy kind name is mistaken for one (the collision rule).
    #[test]
    fn custom_ids_have_one_shape_and_never_equal_a_builtin() {
        for ok in ["custom-beehive-0a1b2c3d", "custom-a-00000000", "custom-my_pin_2-ffffffff"] {
            assert!(is_custom_kind_id(ok) && is_valid_kind_name(ok), "{ok}");
        }
        for bad in ["custom-", "custom--0a1b2c3d", "custom-x-0A1B2C3D", "custom-x-0a1b2c3", "custom-x-0a1b2c3de", "custom-X-0a1b2c3d",
            "Custom-x-0a1b2c3d", "custom-x y-0a1b2c3d", "custom-x-0a1b2c3g", "x-0a1b2c3d", "", &format!("custom-{}-0a1b2c3d", "a".repeat(33))] {
            assert!(!is_custom_kind_id(bad), "{bad:?}");
            assert!(!is_valid_kind_name(bad), "{bad:?}");
        }
        for k in KIND_NAMES {
            assert!(!is_custom_kind_id(k) && !k.contains('-'), "{k}");
        }
        assert!(is_pin_colour("#c0a040") && is_pin_colour("#C0A040"));
        for bad in ["c0a040", "#c0a04", "#c0a0400", "#gggggg", "", "red"] {
            assert!(!is_pin_colour(bad), "{bad:?}");
        }
    }

    /// Protects: the base64 helpers against the RFC 4648 vectors, and the refusals.
    #[test]
    fn base64_matches_the_rfc_vectors_and_refuses_noise() {
        for (raw, enc) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="), ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")] {
            assert_eq!(b64_encode(raw.as_bytes()), enc);
            assert_eq!(b64_decode(enc).as_deref(), Some(raw.as_bytes()));
        }
        let all: Vec<u8> = (0..=255u8).collect();
        assert_eq!(b64_decode(&b64_encode(&all)), Some(all));
        for bad in ["Zg=", "Zg==Zg==", "Z===", "Zm9*", "Zm 9v", "=Zg="] {
            assert_eq!(b64_decode(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn kind_names_labels_and_free_labels() {
        assert!(is_valid_kind_name("mountain_pass") && is_valid_kind_name("inn") && is_valid_kind_name("notice"));
        for bad in ["", "Tavern", "mountain pass", " tavern", "brothel", "../x"] {
            assert!(!is_valid_kind_name(bad), "{bad:?}");
        }
        assert_eq!(kind_label_for("tavern"), "Tavern", "a legacy key keeps its words");
        assert_eq!(kind_label_for("inn"), "Inn", "an inn has its own words, not the tavern's");
        assert_eq!(kind_label_for("mountain_pass"), "Mountain pass");
        assert_eq!(kind_label_for(""), "");
        assert_eq!(clean_free_label("  Beehives \n"), Some("Beehives".to_string()));
        assert_eq!(clean_free_label("   "), None);
        assert_eq!(clean_free_label(&"b".repeat(100)).unwrap().chars().count(), FREE_LABEL_MAX);
        assert_eq!(FREE_MARKS, ["other", "notice"]);
    }

    /// Protects: exactly the twelve keys saves always wrote are `is_original_wire_key`; `inn` is a legacy kind but not one.
    #[test]
    fn only_the_original_twelve_keys_are_wire_keys() {
        let wire: Vec<&str> = PoiKind::ALL.iter().map(|k| k.key()).filter(|k| PoiKind::is_original_wire_key(k)).collect();
        assert_eq!(wire, ["tavern", "shop", "smithy", "temple", "shrine", "guild", "gate", "well", "ruin", "camp", "mine", "other"]);
        assert!(!PoiKind::is_original_wire_key("inn") && !PoiKind::is_original_wire_key("waterfall") && !PoiKind::is_original_wire_key(""));
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
