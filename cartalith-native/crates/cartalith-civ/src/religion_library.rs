//! Custom religions as **named variants** of the eight built-ins --
//! `FACTION_HUB_DESIGN.md` §6, phase **R1**.
//!
//! # What it is
//!
//! A custom religion is a record `{id, name, base_key, colour, notes}`
//! ([`ReligionDef`]) whose engine behaviour **is its `base_key`'s**: one of
//! the eight keys in [`crate::roster::CIV_RELIGIONS`]. A faction's stored
//! religion may then be `custom:<id>` ([`custom_religion_key`]) instead of a
//! built-in key. The user sees the custom name; the engine keeps reading the
//! base key.
//!
//! # The one rule this module exists to enforce
//!
//! **No engine consumer ever sees a raw `custom:` key.** `belief.rs`
//! (`belief_seed`, `compat`), the relations faith term and the religious
//! power axis (`faction_has_religion`) are all keyed by the eight built-ins,
//! and a `custom:3` reaching any of them would not fail loudly -- it would
//! quietly read as "an unknown religion", which those functions treat as no
//! faith at all. So every consumer resolves through
//! [`ReligionLibrary::engine_key`] first, at the boundary, and that function
//! can only return a built-in key or the caller's own non-custom input
//! (tested: `engine_key_never_returns_a_custom_key`).
//!
//! # A dangling id degrades honestly
//!
//! `custom:<id>` with no such record (a hand-edited archive, or an
//! `entities/religions.json` that was damaged or dropped) resolves to
//! [`ReligionChoice::Missing`]. The engine reads that as `"none"` -- no
//! religion, which is the design's stated degradation -- and
//! [`ReligionLibrary::display_name`] says **"Missing religion (custom:N)"**
//! rather than showing a built-in name that was never chosen. It never
//! panics and never maps to a plausible faith (`MISTAKES.md`: never encode
//! "no value" as a plausible value).
//!
//! # What it must never do
//!
//! - Never change generated output. A project with no custom religion never
//!   constructs a `custom:` key, and [`ReligionLibrary::engine_key`] passes
//!   every non-custom key through unchanged -- including an unknown one from
//!   an old or hand-edited archive, which reaches the engine exactly as it
//!   did before this module existed.
//! - Never reissue an id. `next_id` only goes up, and is saved, so a deleted
//!   religion's id cannot be handed to a new one and silently re-attach a
//!   stale reference (the [`crate::conflict::ConflictStore`] rule).
//! - Never carry real attributes. Phase **R2** (axes that drive coexistence
//!   and retention) is blocked on the researched expansion and an owner
//!   ruling; nothing here reads anything but `base_key`.
//!
//! Stateless helpers plus one small value type, like
//! [`crate::conflict::ConflictStore`]: the library lives on
//! `cartalith-godot`'s `CivData`, beside the roster that references it.

use crate::roster::{has_key, CIV_RELIGIONS};

/// The prefix a faction's stored religion carries when it names a custom
/// religion: `custom:<id>`. Chosen so it cannot collide with a built-in key
/// (every `CIV_RELIGIONS` key is `[a-z_]+`, with no colon).
pub const CUSTOM_RELIGION_PREFIX: &str = "custom:";

/// Longest custom religion name, in characters (not bytes). A labelled
/// judgement: long enough for "The Reformed Church of the Western Shore"
/// (39), short enough that a picker row stays one line on a phone.
pub const RELIGION_NAME_MAX_CHARS: usize = 48;

/// Longest custom religion notes, in characters. A labelled judgement: a few
/// paragraphs of free text, bounded so a pasted document cannot bloat
/// `entities/religions.json` unnoticed.
pub const RELIGION_NOTES_MAX_CHARS: usize = 4000;

/// Names a custom religion may not take besides the eight built-ins' own
/// labels and keys: the two the shell itself prints for a faction's
/// religion (`civilization_workspace.gd::_religion_label` reads `"none"` as
/// "No religion"; a dangling reference reads "Missing religion"). A custom
/// one wearing either would be indistinguishable from that state.
const RESERVED_NAMES: [&str; 2] = ["no religion", "missing religion"];

/// One custom religion. Engine behaviour is `base_key`'s, nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReligionDef {
    /// Stable, never reused within a project ([`ReligionLibrary::next_id`]).
    /// The faction's stored key is `custom:<id>`.
    pub id: u32,
    /// Display name, trimmed, non-empty, at most [`RELIGION_NAME_MAX_CHARS`]
    /// characters, unique case-insensitively against the built-ins and the
    /// other custom religions.
    pub name: String,
    /// One of the eight `CIV_RELIGIONS` keys. What the engine reads.
    pub base_key: String,
    /// The swatch the shell draws it with. Display only: no engine reader.
    pub colour: (u8, u8, u8),
    /// Free text, at most [`RELIGION_NOTES_MAX_CHARS`] characters.
    pub notes: String,
}

/// Why a library edit was refused. Every refusal changes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReligionError {
    /// The name is empty once trimmed.
    NameEmpty,
    /// The name is longer than [`RELIGION_NAME_MAX_CHARS`].
    NameTooLong,
    /// The name holds a control character (a newline, a tab).
    NameControlChar,
    /// The name equals (case-insensitively) a built-in religion, a reserved
    /// display string, or another custom religion -- named here.
    NameTaken(String),
    /// `base_key` is not one of the eight built-ins.
    BaseUnknown(String),
    /// The notes are longer than [`RELIGION_NOTES_MAX_CHARS`].
    NotesTooLong,
    /// No custom religion has this id.
    NoSuchReligion(u32),
    /// Delete refused: these factions (roster indices) still hold it.
    InUse(Vec<usize>),
    /// `u32::MAX` ids issued; none left that was never used.
    IdsExhausted,
}

impl std::fmt::Display for ReligionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameEmpty => write!(f, "A religion needs a name."),
            Self::NameTooLong => {
                write!(f, "That name is longer than {RELIGION_NAME_MAX_CHARS} characters.")
            }
            Self::NameControlChar => write!(f, "A name cannot contain line breaks or tabs."),
            Self::NameTaken(other) => write!(f, "\"{other}\" already uses that name."),
            Self::BaseUnknown(k) => write!(f, "\"{k}\" is not one of the eight built-in religions."),
            Self::NotesTooLong => {
                write!(f, "Notes are longer than {RELIGION_NOTES_MAX_CHARS} characters.")
            }
            Self::NoSuchReligion(id) => write!(f, "There is no custom religion {id}."),
            Self::InUse(fs) => write!(
                f,
                "{} faction{} still hold{} this religion; give {} another first.",
                fs.len(),
                if fs.len() == 1 { "" } else { "s" },
                if fs.len() == 1 { "s" } else { "" },
                if fs.len() == 1 { "it" } else { "them" }
            ),
            Self::IdsExhausted => write!(f, "No unused religion id is left in this project."),
        }
    }
}

/// What a faction's stored religion string means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReligionChoice<'a> {
    /// Not a `custom:` key: one of the eight built-ins, or -- from an old or
    /// hand-edited archive -- an unknown key, which is passed through exactly
    /// as it was before custom religions existed.
    Builtin(&'a str),
    /// A `custom:<id>` that names a record in the library.
    Custom(&'a ReligionDef),
    /// A `custom:` key that names nothing (deleted outside the app, damaged
    /// document, or not a number). Carries the raw key for display.
    Missing(&'a str),
}

/// `custom:<id>` for `id`.
pub fn custom_religion_key(id: u32) -> String {
    format!("{CUSTOM_RELIGION_PREFIX}{id}")
}

/// The id in a `custom:<id>` key: ASCII digits only after the prefix, no
/// sign, no whitespace. `None` for a built-in key and for a malformed custom
/// one (which [`ReligionLibrary::choice`] then reports as `Missing`).
pub fn parse_custom_religion_key(key: &str) -> Option<u32> {
    let digits = key.strip_prefix(CUSTOM_RELIGION_PREFIX)?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Lower-cased, trimmed, inner whitespace collapsed to one space: the form
/// two names are compared in, so "Sun  cult" collides with "Sun Cult".
fn fold_name(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The custom-religion library of one project. `next_id` starts at `1` and
/// never goes back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReligionLibrary {
    next_id: u32,
    defs: Vec<ReligionDef>,
}

impl Default for ReligionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl ReligionLibrary {
    /// An empty library; the first religion created gets id `1`.
    pub fn new() -> Self {
        Self { next_id: 1, defs: Vec::new() }
    }

    /// Rebuilds a library from a saved document's rows, tolerating damage
    /// row by row (`SAVEFILE_COMPAT.md` §6.4a rung 3): a row with id `0`, a
    /// duplicate id, an unknown `base_key`, or a name that fails
    /// [`Self::validate_name`] against the rows already kept is dropped, and
    /// the rest open. `next_id` is raised past every kept id and never below
    /// `1` (§9.1's "next_id never trails its data"). Returns the library and
    /// how many rows were dropped, so the caller can say so.
    ///
    /// A faction that held a dropped row's id then reads as `Missing` --
    /// honest, and visible in the shell -- rather than being re-pointed at a
    /// guess.
    pub fn from_saved(next_id: u32, rows: Vec<ReligionDef>) -> (Self, usize) {
        let mut lib = Self::new();
        let mut dropped = 0;
        for mut row in rows {
            let ok_id = row.id != 0 && lib.get(row.id).is_none();
            let ok_base = has_key(&CIV_RELIGIONS, &row.base_key);
            let name = lib.validate_name(&row.name, None);
            match (ok_id, ok_base, name) {
                (true, true, Ok(name)) => {
                    row.name = name;
                    // Over-long notes are trimmed rather than costing the row:
                    // the notes are the least load-bearing member.
                    if row.notes.chars().count() > RELIGION_NOTES_MAX_CHARS {
                        row.notes = row.notes.chars().take(RELIGION_NOTES_MAX_CHARS).collect();
                    }
                    lib.defs.push(row);
                }
                _ => dropped += 1,
            }
        }
        let past_max = lib.defs.iter().map(|d| d.id.saturating_add(1)).max().unwrap_or(1);
        lib.next_id = next_id.max(past_max).max(1);
        (lib, dropped)
    }

    /// Every custom religion, in creation order.
    pub fn defs(&self) -> &[ReligionDef] {
        &self.defs
    }

    /// The id the next [`Self::create`] will issue.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// `true` while no religion has ever been created in this project: the
    /// state in which `entities/religions.json` is not written, so an
    /// untouched project saves exactly the archive it did before R1.
    pub fn is_pristine(&self) -> bool {
        self.next_id <= 1 && self.defs.is_empty()
    }

    /// The record with this id.
    pub fn get(&self, id: u32) -> Option<&ReligionDef> {
        self.defs.iter().find(|d| d.id == id)
    }

    /// What a faction's stored religion string means. See [`ReligionChoice`].
    pub fn choice<'a>(&'a self, key: &'a str) -> ReligionChoice<'a> {
        if !key.starts_with(CUSTOM_RELIGION_PREFIX) {
            return ReligionChoice::Builtin(key);
        }
        match parse_custom_religion_key(key).and_then(|id| self.get(id)) {
            Some(d) => ReligionChoice::Custom(d),
            None => ReligionChoice::Missing(key),
        }
    }

    /// **The boundary resolver.** The key every engine consumer must read in
    /// place of a faction's stored religion: a built-in (or legacy unknown)
    /// key unchanged; a custom religion's `base_key`; and `"none"` for a
    /// dangling `custom:` key (no religion -- the shell labels it "Missing
    /// religion"). Never returns a string starting with
    /// [`CUSTOM_RELIGION_PREFIX`]: a `Custom` record's `base_key` is checked
    /// against `CIV_RELIGIONS` on create, edit and load.
    pub fn engine_key<'a>(&'a self, key: &'a str) -> &'a str {
        match self.choice(key) {
            ReligionChoice::Builtin(k) => k,
            ReligionChoice::Custom(d) => d.base_key.as_str(),
            ReligionChoice::Missing(_) => "none",
        }
    }

    /// The name a person reads for a stored religion: a built-in's
    /// `CIV_RELIGIONS` label (an unknown legacy key prints as itself), a
    /// custom religion's own name, or `"Missing religion (custom:N)"`.
    pub fn display_name(&self, key: &str) -> String {
        match self.choice(key) {
            ReligionChoice::Builtin(k) => CIV_RELIGIONS
                .iter()
                .find(|(bk, _)| *bk == k)
                .map_or_else(|| k.to_string(), |(_, label)| (*label).to_string()),
            ReligionChoice::Custom(d) => d.name.clone(),
            ReligionChoice::Missing(raw) => format!("Missing religion ({raw})"),
        }
    }

    /// Checks a proposed name and returns it trimmed. `except` is the id
    /// being renamed, which may keep its own name. Unique case-insensitively
    /// (whitespace folded) against every built-in label, every built-in key
    /// read with spaces for underscores, [`RESERVED_NAMES`], anything that
    /// looks like a `custom:` key, and every other custom religion.
    pub fn validate_name(&self, name: &str, except: Option<u32>) -> Result<String, ReligionError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(ReligionError::NameEmpty);
        }
        if trimmed.chars().any(char::is_control) {
            return Err(ReligionError::NameControlChar);
        }
        if trimmed.chars().count() > RELIGION_NAME_MAX_CHARS {
            return Err(ReligionError::NameTooLong);
        }
        let folded = fold_name(trimmed);
        if folded.starts_with(CUSTOM_RELIGION_PREFIX) {
            return Err(ReligionError::NameTaken(trimmed.to_string()));
        }
        for (key, label) in CIV_RELIGIONS.iter() {
            if folded == fold_name(label) || folded == fold_name(&key.replace('_', " ")) {
                return Err(ReligionError::NameTaken((*label).to_string()));
            }
        }
        if let Some(r) = RESERVED_NAMES.iter().find(|r| folded == **r) {
            return Err(ReligionError::NameTaken((*r).to_string()));
        }
        if let Some(other) =
            self.defs.iter().find(|d| Some(d.id) != except && fold_name(&d.name) == folded)
        {
            return Err(ReligionError::NameTaken(other.name.clone()));
        }
        Ok(trimmed.to_string())
    }

    fn validate_base(base_key: &str) -> Result<(), ReligionError> {
        if has_key(&CIV_RELIGIONS, base_key) {
            Ok(())
        } else {
            Err(ReligionError::BaseUnknown(base_key.to_string()))
        }
    }

    fn validate_notes(notes: &str) -> Result<(), ReligionError> {
        if notes.chars().count() > RELIGION_NOTES_MAX_CHARS {
            Err(ReligionError::NotesTooLong)
        } else {
            Ok(())
        }
    }

    /// Creates a religion and returns its new id. Validates everything
    /// before storing anything.
    pub fn create(
        &mut self,
        name: &str,
        base_key: &str,
        colour: (u8, u8, u8),
        notes: &str,
    ) -> Result<u32, ReligionError> {
        let name = self.validate_name(name, None)?;
        Self::validate_base(base_key)?;
        Self::validate_notes(notes)?;
        let id = self.next_id;
        if id == u32::MAX {
            return Err(ReligionError::IdsExhausted);
        }
        self.next_id = id + 1;
        self.defs.push(ReligionDef {
            id,
            name,
            base_key: base_key.to_string(),
            colour,
            notes: notes.to_string(),
        });
        Ok(id)
    }

    /// Edits the members given as `Some`, all-or-nothing: every proposed
    /// member is validated before any is written.
    pub fn edit(
        &mut self,
        id: u32,
        name: Option<&str>,
        base_key: Option<&str>,
        colour: Option<(u8, u8, u8)>,
        notes: Option<&str>,
    ) -> Result<(), ReligionError> {
        if self.get(id).is_none() {
            return Err(ReligionError::NoSuchReligion(id));
        }
        let name = name.map(|n| self.validate_name(n, Some(id))).transpose()?;
        if let Some(b) = base_key {
            Self::validate_base(b)?;
        }
        if let Some(n) = notes {
            Self::validate_notes(n)?;
        }
        let d = self.defs.iter_mut().find(|d| d.id == id).expect("checked above");
        if let Some(n) = name {
            d.name = n;
        }
        if let Some(b) = base_key {
            d.base_key = b.to_string();
        }
        if let Some(c) = colour {
            d.colour = c;
        }
        if let Some(n) = notes {
            d.notes = n.to_string();
        }
        Ok(())
    }

    /// Deletes religion `id`, refusing while any faction holds it.
    /// `faction_choices` is every roster row's stored religion, in roster
    /// order; the refusal names the indices that hold it. `next_id` is not
    /// touched, so the id is never issued again.
    pub fn delete<'k>(
        &mut self,
        id: u32,
        faction_choices: impl IntoIterator<Item = &'k str>,
    ) -> Result<ReligionDef, ReligionError> {
        let pos = self
            .defs
            .iter()
            .position(|d| d.id == id)
            .ok_or(ReligionError::NoSuchReligion(id))?;
        let users: Vec<usize> = faction_choices
            .into_iter()
            .enumerate()
            .filter(|(_, k)| parse_custom_religion_key(k) == Some(id))
            .map(|(i, _)| i)
            .collect();
        if !users.is_empty() {
            return Err(ReligionError::InUse(users));
        }
        Ok(self.defs.remove(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib_with_one() -> (ReligionLibrary, u32) {
        let mut lib = ReligionLibrary::new();
        let id = lib.create("Church of the Tide", "sea_lords", (10, 20, 30), "notes").unwrap();
        (lib, id)
    }

    /// Protects: the custom-key round trip, and that a malformed key is
    /// never read as an id (so it lands in `Missing`, not on a real record).
    #[test]
    fn custom_keys_parse_strictly() {
        assert_eq!(custom_religion_key(7), "custom:7");
        assert_eq!(parse_custom_religion_key("custom:7"), Some(7));
        for bad in ["custom:", "custom:+7", "custom: 7", "custom:7a", "custom:-1", "sun_cult", "Custom:7"] {
            assert_eq!(parse_custom_religion_key(bad), None, "{bad}");
        }
    }

    /// Protects: the boundary rule -- a custom religion resolves to its base
    /// key, a dangling one to `"none"`, and a built-in or legacy unknown key
    /// passes through unchanged (so pre-R1 projects behave byte-identically).
    #[test]
    fn engine_key_resolves_at_the_boundary() {
        let (lib, id) = lib_with_one();
        let key = custom_religion_key(id);
        assert_eq!(lib.engine_key(&key), "sea_lords");
        assert_eq!(lib.engine_key("custom:99"), "none");
        assert_eq!(lib.engine_key("custom:x"), "none");
        assert_eq!(lib.engine_key("sun_cult"), "sun_cult");
        assert_eq!(lib.engine_key("zen_legacy"), "zen_legacy", "legacy unknown keys pass through");
    }

    /// Protects: the one rule of this module -- whatever is stored, and
    /// whatever the library holds, the engine never receives a `custom:` key.
    #[test]
    fn engine_key_never_returns_a_custom_key() {
        let (mut lib, _) = lib_with_one();
        lib.create("Hearth Path", "none", (0, 0, 0), "").unwrap();
        let mut probes: Vec<String> = (0..6).map(custom_religion_key).collect();
        probes.extend(CIV_RELIGIONS.iter().map(|(k, _)| k.to_string()));
        probes.extend(["custom:", "custom:abc", "custom:4294967295"].map(String::from));
        for p in &probes {
            assert!(!lib.engine_key(p).starts_with(CUSTOM_RELIGION_PREFIX), "{p}");
        }
    }

    /// Protects: a dangling reference is labelled as missing, never shown as
    /// a built-in name the user did not choose.
    #[test]
    fn display_names_are_honest() {
        let (lib, id) = lib_with_one();
        assert_eq!(lib.display_name(&custom_religion_key(id)), "Church of the Tide");
        assert_eq!(lib.display_name("custom:42"), "Missing religion (custom:42)");
        assert_eq!(lib.display_name("sun_cult"), "Sun Cult");
        assert_eq!(lib.display_name("zen_legacy"), "zen_legacy");
    }

    /// Protects: name validation -- trimmed, non-empty, bounded, no control
    /// characters, unique case-insensitively against built-in labels, keys,
    /// reserved display strings and other customs; a rename may keep its own.
    #[test]
    fn names_are_validated() {
        let (mut lib, id) = lib_with_one();
        assert_eq!(lib.validate_name("  Hearth  ", None), Ok("Hearth".to_string()));
        assert_eq!(lib.validate_name("   ", None), Err(ReligionError::NameEmpty));
        assert_eq!(lib.validate_name("a\nb", None), Err(ReligionError::NameControlChar));
        assert_eq!(lib.validate_name(&"x".repeat(48), None).map(|s| s.len()), Ok(48));
        assert_eq!(lib.validate_name(&"x".repeat(49), None), Err(ReligionError::NameTooLong));
        for taken in ["sun cult", "SUN  CULT", "Old gods", "none / SECULAR", "sea lords", "No Religion", "missing religion", "custom:4"] {
            assert!(matches!(lib.validate_name(taken, None), Err(ReligionError::NameTaken(_))), "{taken}");
        }
        assert!(matches!(lib.validate_name("church of the TIDE", None), Err(ReligionError::NameTaken(_))));
        assert_eq!(lib.validate_name("Church of the Tide", Some(id)), Ok("Church of the Tide".to_string()));
        assert!(lib.create("Church of the Tide", "sun_cult", (0, 0, 0), "").is_err());
    }

    /// Protects: base key and notes validation, and that a refused edit is
    /// all-or-nothing (a valid name in the same call is not written).
    #[test]
    fn create_and_edit_validate_before_writing() {
        let (mut lib, id) = lib_with_one();
        assert_eq!(lib.create("X", "cargo_cult", (0, 0, 0), ""), Err(ReligionError::BaseUnknown("cargo_cult".into())));
        assert_eq!(lib.create("X", "custom:1", (0, 0, 0), ""), Err(ReligionError::BaseUnknown("custom:1".into())));
        assert_eq!(lib.create("X", "sun_cult", (0, 0, 0), &"n".repeat(4001)), Err(ReligionError::NotesTooLong));
        let before = lib.clone();
        assert!(lib.edit(id, Some("Renamed"), Some("nope"), None, None).is_err());
        assert_eq!(lib, before, "nothing written on refusal");
        lib.edit(id, Some(" Renamed "), Some("old_gods"), Some((1, 2, 3)), Some("n2")).unwrap();
        let d = lib.get(id).unwrap();
        assert_eq!((d.name.as_str(), d.base_key.as_str(), d.colour, d.notes.as_str()), ("Renamed", "old_gods", (1, 2, 3), "n2"));
        assert_eq!(lib.edit(99, None, None, None, None), Err(ReligionError::NoSuchReligion(99)));
    }

    /// Protects: delete is refused while any faction holds the religion,
    /// naming who; and an id is never reissued after a delete.
    #[test]
    fn delete_refuses_in_use_and_ids_are_never_reused() {
        let (mut lib, id) = lib_with_one();
        let key = custom_religion_key(id);
        let roster = ["none", key.as_str(), "sun_cult", key.as_str()];
        assert_eq!(lib.delete(id, roster), Err(ReligionError::InUse(vec![1, 3])));
        assert!(lib.get(id).is_some());
        lib.delete(id, ["none", "sun_cult"]).unwrap();
        assert!(lib.get(id).is_none());
        let next = lib.create("Second", "sun_cult", (0, 0, 0), "").unwrap();
        assert_eq!(next, 2, "id 1 is never issued again");
        assert_eq!(lib.delete(1, ["none"]), Err(ReligionError::NoSuchReligion(1)));
    }

    /// Protects: the load rule -- damaged rows cost themselves only, and
    /// `next_id` never trails the data (nor falls below a saved value that
    /// records deleted ids).
    #[test]
    fn from_saved_tolerates_bad_rows_and_keeps_next_id_ahead() {
        let row = |id, name: &str, base: &str| ReligionDef {
            id,
            name: name.into(),
            base_key: base.into(),
            colour: (1, 1, 1),
            notes: String::new(),
        };
        let (lib, dropped) = ReligionLibrary::from_saved(
            0,
            vec![row(3, "A", "sun_cult"), row(3, "B", "sun_cult"), row(0, "C", "sun_cult"), row(5, "D", "zen"), row(6, "Sun Cult", "old_gods"), row(7, "E", "old_gods")],
        );
        assert_eq!(dropped, 4);
        assert_eq!(lib.defs().iter().map(|d| d.id).collect::<Vec<_>>(), vec![3, 7]);
        assert_eq!(lib.next_id(), 8);
        let (lib, _) = ReligionLibrary::from_saved(20, vec![row(3, "A", "sun_cult")]);
        assert_eq!(lib.next_id(), 20, "a saved next_id past deleted ids is kept");
        let (empty, _) = ReligionLibrary::from_saved(0, vec![]);
        assert!(empty.is_pristine());
    }
}
