//! The custom-religion library's `#[func]` surface -- `FACTION_HUB_DESIGN.md`
//! §6, phase **R1** ("named variants").
//!
//! A custom religion is a [`ReligionDef`] in `CivData::religions`; a faction
//! picks one by storing `custom:<id>` as its religion
//! (`civ_set_faction_field(f, "religion", "custom:3")`). Its engine behaviour
//! is its `base_key`'s: every engine reader resolves through
//! `ReligionLibrary::engine_key` (`FactionRoster::engine_religions` /
//! `has_religion_flags`), so this module adds names, colours and notes and
//! **changes no generated output**. The shell must say so in words; R2 (real
//! attributes) is not built.
//!
//! The four funcs follow the roster's `civ_*` naming. Each mutation validates
//! before it writes ([`ReligionLibrary`]'s rule) and answers with a
//! dictionary carrying `ok` and, on refusal, a plain-English `error` -- the
//! shell shows that text rather than inventing its own reason. Before any
//! `generate()` there is no civilisation layer and every call refuses.
//!
//! The logic sits in the pure `CivData` methods below so `cargo test` can
//! reach it; a `#[func]` cannot run without a live engine (`lib.rs`'s
//! `civ_merge_tests` states why).

use godot::prelude::*;

use cartalith_civ::religion_library::{custom_religion_key, ReligionDef, ReligionError};
use cartalith_civ::roster::CIV_RELIGIONS;

use crate::{CivData, WorldGen};

/// Answer for a call made before any world exists. Not a library error: there
/// is no library yet, because there is no roster for it to serve.
const NO_CIV: &str = "Generate a world first: custom religions belong to its factions.";

/// A colour channel from GDScript, or `None` outside `0..=255`. Refused rather
/// than clamped: a clamped 300 would store a colour nobody chose.
fn channel(v: i64) -> Option<u8> {
    u8::try_from(v).ok()
}

/// `(r, g, b)` from three GDScript ints, or the refusal text.
fn colour_from(r: i64, g: i64, b: i64) -> Result<(u8, u8, u8), String> {
    match (channel(r), channel(g), channel(b)) {
        (Some(r), Some(g), Some(b)) => Ok((r, g, b)),
        _ => Err("A colour channel must be a whole number from 0 to 255.".to_string()),
    }
}

/// A religion id from GDScript. Ids are `u32` from `1`; anything else names
/// no religion, which the library reports in its own words.
fn id_from(id: i64) -> u32 {
    // 0 is never issued, so an out-of-range id maps to "no such religion".
    u32::try_from(id).unwrap_or(0)
}

impl CivData {
    /// Deletes custom religion `id`, refusing while any roster row holds it.
    /// Pure, so the refusal is unit-testable.
    pub(crate) fn religion_delete(&mut self, id: u32) -> Result<ReligionDef, ReligionError> {
        let CivData { religions, faction_roster, .. } = self;
        religions.delete(id, faction_roster.0.iter().map(|e| e.religion.as_str()))
    }

    /// Roster indices whose stored religion is `custom:<id>`.
    pub(crate) fn religion_users(&self, id: u32) -> Vec<usize> {
        let key = custom_religion_key(id);
        self.faction_roster
            .0
            .iter()
            .enumerate()
            .filter(|(_, e)| e.religion == key)
            .map(|(i, _)| i)
            .collect()
    }
}

/// The refusal dictionary every mutation returns on failure.
fn refused(error: impl Into<String>) -> VarDictionary {
    vdict! { "ok" => false, "error" => error.into() }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The project's custom religions, in creation order, as
    /// `{id, key, name, base_key, base_label, color_r, color_g, color_b,
    /// notes, used_by}` -- `key` is the `custom:<id>` a faction stores,
    /// `base_label` the built-in's `CIV_RELIGIONS` label (what it behaves
    /// as), `used_by` the faction ids holding it (a delete is refused while
    /// it is non-empty). Empty before any `generate()`.
    #[func]
    fn civ_custom_religions(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.religions
            .defs()
            .iter()
            .map(|d| {
                let base_label = CIV_RELIGIONS
                    .iter()
                    .find(|(k, _)| *k == d.base_key)
                    .map_or(d.base_key.as_str(), |(_, l)| *l);
                let used_by: PackedInt32Array =
                    civ.religion_users(d.id).into_iter().map(|i| i as i32).collect();
                vdict! {
                    "id" => d.id as i64,
                    "key" => custom_religion_key(d.id),
                    "name" => d.name.as_str(),
                    "base_key" => d.base_key.as_str(),
                    "base_label" => base_label,
                    "color_r" => d.colour.0 as i64,
                    "color_g" => d.colour.1 as i64,
                    "color_b" => d.colour.2 as i64,
                    "notes" => d.notes.as_str(),
                    "used_by" => &used_by,
                }
            })
            .collect()
    }

    /// Creates a custom religion behaving as `base_key` (one of the eight
    /// built-ins). `{ok: true, id, key}` or `{ok: false, error}`; the name is
    /// trimmed and must be unique (case-insensitive) against the built-ins and
    /// the other custom religions. Assigning it to a faction is a separate
    /// `civ_set_faction_field(f, "religion", key)`.
    #[func]
    fn civ_add_religion(
        &mut self,
        name: GString,
        base_key: GString,
        color_r: i64,
        color_g: i64,
        color_b: i64,
        notes: GString,
    ) -> VarDictionary {
        let Some(civ) = self.civ.as_mut() else { return refused(NO_CIV) };
        let colour = match colour_from(color_r, color_g, color_b) {
            Ok(c) => c,
            Err(e) => return refused(e),
        };
        match civ.religions.create(&name.to_string(), &base_key.to_string(), colour, &notes.to_string()) {
            Ok(id) => vdict! { "ok" => true, "id" => id as i64, "key" => custom_religion_key(id) },
            Err(e) => refused(e.to_string()),
        }
    }

    /// Edits custom religion `id`. `fields` may carry `name`, `base_key`,
    /// `notes` (Strings) and `color_r`/`color_g`/`color_b` (all three
    /// together); an unknown key, a wrongly typed value or a partial colour is
    /// refused. All-or-nothing: `{ok: true}` or `{ok: false, error}` with
    /// nothing written. A new `base_key` changes what every faction holding
    /// it behaves as, and so invalidates a belief run seeded from the old one
    /// (`CivData::belief_key` covers the resolved column).
    #[func]
    fn civ_edit_religion(&mut self, id: i64, fields: VarDictionary) -> VarDictionary {
        let Some(civ) = self.civ.as_mut() else { return refused(NO_CIV) };
        const KNOWN: [&str; 6] = ["name", "base_key", "notes", "color_r", "color_g", "color_b"];
        for (k, _) in fields.iter_shared() {
            let k = k.to_string();
            if !KNOWN.contains(&k.as_str()) {
                return refused(format!("\"{k}\" is not a religion field."));
            }
        }
        let get_s = |k: &str| -> Result<Option<String>, String> {
            match fields.get(k) {
                None => Ok(None),
                Some(v) => v
                    .try_to::<GString>()
                    .map(|g| Some(g.to_string()))
                    .map_err(|_| format!("\"{k}\" must be text.")),
            }
        };
        let get_i = |k: &str| -> Result<Option<i64>, String> {
            match fields.get(k) {
                None => Ok(None),
                Some(v) => v.try_to::<i64>().map(Some).map_err(|_| format!("\"{k}\" must be a whole number.")),
            }
        };
        let parsed = (|| -> Result<_, String> {
            let name = get_s("name")?;
            let base = get_s("base_key")?;
            let notes = get_s("notes")?;
            let colour = match (get_i("color_r")?, get_i("color_g")?, get_i("color_b")?) {
                (None, None, None) => None,
                (Some(r), Some(g), Some(b)) => Some(colour_from(r, g, b)?),
                _ => return Err("A colour needs all three channels.".to_string()),
            };
            Ok((name, base, notes, colour))
        })();
        let (name, base, notes, colour) = match parsed {
            Ok(p) => p,
            Err(e) => return refused(e),
        };
        match civ.religions.edit(id_from(id), name.as_deref(), base.as_deref(), colour, notes.as_deref()) {
            Ok(()) => vdict! { "ok" => true },
            Err(e) => refused(e.to_string()),
        }
    }

    /// Deletes custom religion `id` -- **refused while any faction holds
    /// it**, with `used_by` naming those faction ids so the shell can say
    /// who. `{ok: true}` or `{ok: false, error, used_by?}`. The id is never
    /// issued again in this project.
    #[func]
    fn civ_delete_religion(&mut self, id: i64) -> VarDictionary {
        let Some(civ) = self.civ.as_mut() else { return refused(NO_CIV) };
        match civ.religion_delete(id_from(id)) {
            Ok(_) => vdict! { "ok" => true },
            Err(e) => {
                let mut d = refused(e.to_string());
                if let ReligionError::InUse(users) = &e {
                    let ids: PackedInt32Array = users.iter().map(|&i| i as i32).collect();
                    d.set("used_by", &ids);
                }
                d
            }
        }
    }
}

/// The custom religion's own name for a stored religion key, or `None` unless
/// the key is a `custom:<id>` that resolves in `religions`.
///
/// The one definition of "does this faction hold a *named* religion" that the
/// display surfaces share (the map hover card's `ruler_religion_name`, the
/// GeoJSON `religionName`): a built-in, a legacy unknown key and a dangling
/// reference are all `None`, so a surface never prints a base name, an empty
/// string or `"Missing religion"` as though it were a custom name. Display
/// only; it must never feed a simulation input (engine readers resolve
/// through `ReligionLibrary::engine_key`).
pub(crate) fn custom_religion_name<'a>(
    religions: &'a cartalith_civ::religion_library::ReligionLibrary,
    stored: &'a str,
) -> Option<&'a str> {
    match religions.choice(stored) {
        cartalith_civ::religion_library::ReligionChoice::Custom(d) => Some(d.name.as_str()),
        _ => None,
    }
}

impl CivData {
    /// The custom religion's own name for faction `faction`'s stored religion
    /// (`FactionRoster` row index), or `None` for a built-in, a dangling
    /// reference, faction `0` with no custom choice, or an out-of-range index.
    /// What `get_settlements` reports as `ruler_religion_name` so the hover
    /// card can name the ruler's faith as the user named it.
    pub(crate) fn ruler_custom_religion_name(&self, faction: i32) -> Option<&str> {
        let e = usize::try_from(faction).ok().and_then(|i| self.faction_roster.0.get(i))?;
        custom_religion_name(&self.religions, e.religion.as_str())
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;
    use cartalith_civ::religion_library::ReligionLibrary;

    /// Protects: `custom_religion_name` answers only for a resolving
    /// `custom:<id>` -- never the base's name, a built-in's label or a
    /// "Missing religion" string. Mutation check: returning the base key for
    /// `Custom` (the old fall-back-to-base display) turns the first assertion
    /// red.
    #[test]
    fn only_a_resolving_custom_key_has_a_custom_name() {
        let mut lib = ReligionLibrary::new();
        let id = lib.create("Church of the Tide", "sea_lords", (1, 2, 3), "").unwrap();
        let key = custom_religion_key(id);
        assert_eq!(custom_religion_name(&lib, &key), Some("Church of the Tide"));
        assert_eq!(custom_religion_name(&lib, "sea_lords"), None);
        assert_eq!(custom_religion_name(&lib, "none"), None);
        assert_eq!(custom_religion_name(&lib, "custom:99"), None);
        assert_eq!(custom_religion_name(&lib, "custom:x"), None);
    }
}
