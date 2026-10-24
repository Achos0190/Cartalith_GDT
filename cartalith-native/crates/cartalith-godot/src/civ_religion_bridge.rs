//! The custom-religion library's `#[func]` surface -- `FACTION_HUB_DESIGN.md`
//! §6, phase **R1** ("named variants").
//!
//! A custom religion is a [`ReligionDef`] in `CivData::religions`; a faction
//! picks one by storing `custom:<id>` as its religion
//! (`civ_set_faction_field(f, "religion", "custom:3")`). Its engine behaviour
//! is its `base_key`'s: every engine reader resolves through
//! `ReligionLibrary::engine_key` (`FactionRoster::engine_religions` /
//! `has_religion_flags`). **Since FH-R2 R2-3 (2026-10-24)** a custom religion
//! may also override its base's axes ([`axes_from_dict`], the `axes` field of
//! `civ_edit_religion`); the belief layer reads the exclusivity and
//! membership overrides in the holding faction's territory
//! (`CivData::belief_regime`), and nothing else reads any override. An
//! unedited custom religion still changes no generated output.
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
use cartalith_civ::religion_profile::{
    religion_profile, AxisOverrides, AxisValue, Exclusivity, Institution, Locus, Membership, WorshipObject,
};
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

/// The editor's names for axes 1-5, in display order: `(field key, label,
/// read by the simulation)`. The third member is what the shell's note says
/// ("belief spread reads exclusivity and membership; the rest are recorded"),
/// so the words cannot drift from `AxisOverrides::touches_belief`.
pub(crate) const AXIS_FIELDS: [(&str, &str, bool); 5] = [
    ("object", "Object of worship", false),
    ("exclusivity", "Exclusivity", true),
    ("membership", "Membership", true),
    ("locus", "Locus", false),
    ("institution", "Institution", false),
];

/// `{axis: value_key}` for the overridden axes of `o` (absent = "as base").
pub(crate) fn axes_dict(o: &AxisOverrides) -> VarDictionary {
    let mut d = VarDictionary::new();
    if let Some(v) = o.object {
        d.set("object", v.key());
    }
    if let Some(v) = o.exclusivity {
        d.set("exclusivity", v.key());
    }
    if let Some(v) = o.membership {
        d.set("membership", v.key());
    }
    if let Some(v) = o.locus {
        d.set("locus", v.key());
    }
    if let Some(v) = o.institution {
        d.set("institution", v.key());
    }
    d
}

/// One axis value from the shell: `""` is "as base" (`Ok(None)`); a known key
/// is `Ok(Some)`; anything else is refused with the axis named.
fn axis_from<T: AxisValue>(axis: &str, v: &str) -> Result<Option<T>, String> {
    if v.is_empty() {
        return Ok(None);
    }
    T::from_key(v).map(Some).ok_or_else(|| format!("\"{v}\" is not a value of {axis}."))
}

/// The shell's `axes` dictionary as overrides. Replaces wholesale: an axis the
/// dictionary does not name is "as base", so the editor always sends all five
/// (an empty string for "as base"). An unknown axis name, a non-text value or
/// an unknown value key refuses the whole edit.
pub(crate) fn axes_from_dict(d: &VarDictionary) -> Result<AxisOverrides, String> {
    let mut o = AxisOverrides::default();
    for (k, v) in d.iter_shared() {
        let k = k.to_string();
        let v = v.try_to::<GString>().map_err(|_| format!("Belief \"{k}\" must be text."))?.to_string();
        match k.as_str() {
            "object" => o.object = axis_from::<WorshipObject>(&k, &v)?,
            "exclusivity" => o.exclusivity = axis_from::<Exclusivity>(&k, &v)?,
            "membership" => o.membership = axis_from::<Membership>(&k, &v)?,
            "locus" => o.locus = axis_from::<Locus>(&k, &v)?,
            "institution" => o.institution = axis_from::<Institution>(&k, &v)?,
            _ => return Err(format!("\"{k}\" is not a religion belief.")),
        }
    }
    Ok(o)
}

/// A built-in religion's own axes as `{axis: value_key}` (locus joined by
/// `+`, e.g. `portable+civic`), omitting every axis its profile leaves
/// unclassified -- what the editor shows as "as base (...)". Empty for `none`.
pub(crate) fn base_axes_dict(key: &str) -> VarDictionary {
    let mut d = VarDictionary::new();
    let Some(p) = religion_profile(key) else { return d };
    if let Some(v) = p.object {
        d.set("object", v.key());
    }
    d.set("exclusivity", p.exclusivity.key());
    if let Some(v) = p.membership {
        d.set("membership", v.key());
    }
    if !p.locus.is_empty() {
        let l: Vec<&str> = p.locus.iter().map(|v| v.key()).collect();
        d.set("locus", l.join("+").as_str());
    }
    if let Some(v) = p.institution {
        d.set("institution", v.key());
    }
    d
}

/// `[{key, label}]` for one axis' values, in `ALL` order.
fn axis_values<T: AxisValue>() -> Array<VarDictionary> {
    T::ALL.iter().map(|v| vdict! { "key" => v.key(), "label" => v.label() }).collect()
}

impl CivData {
    /// FH-R2 R2-3's edit, all-or-nothing across both halves: the R1 members
    /// (`edit`) and, when `axes` is `Some`, the overrides (`set_axes`), applied
    /// to a copy and kept only if both succeed. So a base change to `none`
    /// together with new overrides, or overrides on a `none` base, write
    /// nothing at all.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn religion_edit(
        &mut self,
        id: u32,
        name: Option<&str>,
        base: Option<&str>,
        colour: Option<(u8, u8, u8)>,
        notes: Option<&str>,
        axes: Option<AxisOverrides>,
    ) -> Result<(), ReligionError> {
        let mut next = self.religions.clone();
        // Overrides first when they clear: a base change to `none` together
        // with "every belief back to as base" is a legal edit.
        if let Some(a) = axes.as_ref().filter(|a| a.is_empty()) {
            next.set_axes(id, a.clone())?;
        }
        next.edit(id, name, base, colour, notes)?;
        if let Some(a) = axes.filter(|a| !a.is_empty()) {
            next.set_axes(id, a)?;
        }
        self.religions = next;
        Ok(())
    }

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
    /// notes, used_by, axes}` -- `key` is the `custom:<id>` a faction stores,
    /// `base_label` the built-in's `CIV_RELIGIONS` label (what it behaves
    /// as), `used_by` the faction ids holding it (a delete is refused while
    /// it is non-empty), `axes` its FH-R2 R2-3 overrides as `{axis: value}`
    /// with only the overridden axes present (empty = behaves as its base).
    /// Empty before any `generate()`.
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
                    "axes" => &axes_dict(&d.axes),
                }
            })
            .collect()
    }

    /// Creates a custom religion behaving as `base_key` (one of the
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
    /// `notes` (Strings), `color_r`/`color_g`/`color_b` (all three
    /// together) and, since FH-R2 R2-3, `axes`: a Dictionary of the five
    /// beliefs (`object`, `exclusivity`, `membership`, `locus`,
    /// `institution`), each a value key from `civ_religion_axes_vocabulary()`
    /// or `""` for "as base", replacing the overrides wholesale. An unknown
    /// key, a wrongly typed value or a partial colour is refused.
    /// All-or-nothing across every member: `{ok: true}` or
    /// `{ok: false, error}` with nothing written. A new `base_key` changes
    /// what every faction holding it behaves as, and so invalidates a belief
    /// run seeded from the old one (`CivData::belief_key` covers the resolved
    /// column); an axis edit applies from the next run.
    #[func]
    fn civ_edit_religion(&mut self, id: i64, fields: VarDictionary) -> VarDictionary {
        let Some(civ) = self.civ.as_mut() else { return refused(NO_CIV) };
        const KNOWN: [&str; 7] = ["name", "base_key", "notes", "color_r", "color_g", "color_b", "axes"];
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
            let axes = match fields.get("axes") {
                None => None,
                Some(v) => {
                    let d = v.try_to::<VarDictionary>().map_err(|_| "\"axes\" must be a dictionary.".to_string())?;
                    Some(axes_from_dict(&d)?)
                }
            };
            Ok((name, base, notes, colour, axes))
        })();
        let (name, base, notes, colour, axes) = match parsed {
            Ok(p) => p,
            Err(e) => return refused(e),
        };
        match civ.religion_edit(id_from(id), name.as_deref(), base.as_deref(), colour, notes.as_deref(), axes) {
            Ok(()) => vdict! { "ok" => true },
            Err(e) => refused(e.to_string()),
        }
    }

    /// FH-R2 R2-3's editor vocabulary: one row per axis 1-5, in display
    /// order, as `{key, label, read, values: [{key, label}]}`. `read` is
    /// whether the simulation reads the axis today (exclusivity and
    /// membership, the belief layer); the other three are recorded and shown
    /// only, and the shell says so rather than implying they do something.
    #[func]
    fn civ_religion_axes_vocabulary(&self) -> Array<VarDictionary> {
        AXIS_FIELDS
            .iter()
            .map(|&(key, label, read)| {
                let values = match key {
                    "object" => axis_values::<WorshipObject>(),
                    "exclusivity" => axis_values::<Exclusivity>(),
                    "membership" => axis_values::<Membership>(),
                    "locus" => axis_values::<Locus>(),
                    _ => axis_values::<Institution>(),
                };
                vdict! { "key" => key, "label" => label, "read" => read, "values" => &values }
            })
            .collect()
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

/// Tests of how a custom religion's name reaches the UI (`custom_religion_name`).
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

/// Tests of R2-3's bridge logic that need no live engine: the all-or-nothing
/// edit across R1 members and axes.
#[cfg(test)]
mod axes_tests {
    use super::*;

    /// Protects: `religion_edit` is all-or-nothing across both halves. A base
    /// change to `none` with new overrides writes nothing (the name in the
    /// same call included); a base change to `none` that also clears every
    /// override is legal; overrides alone are written.
    #[test]
    fn an_edit_with_axes_is_all_or_nothing() {
        let mut civ = crate::civ_timeline_tests::empty_civ(Vec::new(), Vec::new());
        let id = civ.religions.create("Tide", "sea_lords", (1, 2, 3), "").unwrap();
        let birth = AxisOverrides { membership: Some(Membership::Birth), ..Default::default() };
        civ.religion_edit(id, None, None, None, None, Some(birth.clone())).unwrap();
        assert_eq!(civ.religions.get(id).unwrap().axes, birth);
        let before = civ.religions.clone();
        assert_eq!(
            civ.religion_edit(id, Some("Renamed"), Some("none"), None, None, Some(birth.clone())),
            Err(ReligionError::AxesOnNoReligion)
        );
        assert_eq!(civ.religions, before, "nothing written");
        assert_eq!(civ.religion_edit(id, None, Some("none"), None, None, None), Err(ReligionError::AxesOnNoReligion));
        // The R1 half would succeed alone (a fresh religion has no overrides
        // to block a base change), the axes half fails: nothing is written.
        let fresh = civ.religions.create("Fresh", "sun_cult", (1, 1, 1), "").unwrap();
        let before = civ.religions.clone();
        assert_eq!(
            civ.religion_edit(fresh, Some("Fresher"), Some("none"), None, None, Some(birth.clone())),
            Err(ReligionError::AxesOnNoReligion)
        );
        assert_eq!(civ.religions, before, "the successful R1 half is not kept either");
        civ.religion_edit(id, None, Some("none"), None, None, Some(AxisOverrides::default())).unwrap();
        let d = civ.religions.get(id).unwrap();
        assert_eq!((d.base_key.as_str(), d.axes.is_empty()), ("none", true));
    }
}

/// Tests of `CivData::belief_regime`, R2-3's bridge from the roster to the
/// belief step.
#[cfg(test)]
mod regime_tests {
    use crate::civ_timeline_tests::{empty_civ, mk_settlement};
    use cartalith_civ::religion_library::custom_religion_key;
    use cartalith_civ::religion_profile::{AxisOverrides, Locus, Membership};

    /// Protects: the regime maps a faction's settlements to a variant table
    /// only when that faction holds a custom religion overriding a
    /// belief-read axis; an unedited custom religion, a recorded-only
    /// override and a built-in all leave the regime uniform. Mutation check:
    /// a builder that never maps (`per_faction` all `None`) fails the second
    /// half.
    #[test]
    fn the_regime_maps_only_a_belief_reading_override() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha"), mk_settlement(2, 9, "Beta")], vec![]);
        civ.settlements[1].placement.faction = 2;
        assert!(civ.belief_regime().is_uniform(), "built-ins only");
        let id = civ.religions.create("Tide", "sea_lords", (1, 2, 3), "").unwrap();
        {
            let crate::CivData { faction_roster, religions, .. } = &mut civ;
            assert!(faction_roster.set_religion(1, &custom_religion_key(id), religions));
        }
        assert!(civ.belief_regime().is_uniform(), "unedited custom religion");
        civ.religions
            .set_axes(id, AxisOverrides { locus: Some(Locus::Portable), ..Default::default() })
            .unwrap();
        assert!(civ.belief_regime().is_uniform(), "recorded-only override");
        civ.religions
            .set_axes(id, AxisOverrides { membership: Some(Membership::Birth), ..Default::default() })
            .unwrap();
        let r = civ.belief_regime();
        assert_eq!(r.terms.len(), 1);
        assert_eq!(r.of, vec![Some(0), None], "faction 1's settlement only");
    }

    /// Protects: R2-6's wiring. The default policy (plural) and a policy on a
    /// faction with no state religion leave the regime uniform; a real policy
    /// maps only that faction's settlements, and stacks over a custom
    /// variant's table when both apply (the table carries both the variant's
    /// retention for its own slot and the policy's minority multiplier).
    /// `set_field` refuses Unclaimed and an unknown key. Mutation check: a
    /// builder that ignores the policy fails the second assertion.
    #[test]
    fn a_policy_maps_its_faction_and_plural_maps_nothing() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha"), mk_settlement(2, 9, "Beta")], vec![]);
        civ.settlements[1].placement.faction = 2;
        assert_eq!(civ.faction_roster.0[1].religious_policy, "plural");
        assert!(civ.belief_regime().is_uniform(), "plural everywhere");
        assert!(civ.faction_roster.set_field(1, "religious_policy", "persecuting"));
        let r = civ.belief_regime();
        assert_eq!(r.of, vec![Some(0), None]);
        assert!(!civ.faction_roster.set_field(0, "religious_policy", "persecuting"), "Unclaimed has no ruler");
        assert!(!civ.faction_roster.set_field(1, "religious_policy", "theocratic"));
        assert_eq!(civ.faction_roster.0[1].religious_policy, "persecuting", "a refusal writes nothing");
        // No state religion: nothing to establish.
        assert!(civ.faction_roster.set_field(1, "religion", "none"));
        assert!(civ.belief_regime().is_uniform());
        // Variant plus policy: one table carrying both.
        let id = civ.religions.create("Tide", "sea_lords", (1, 2, 3), "").unwrap();
        {
            let crate::CivData { faction_roster, religions, .. } = &mut civ;
            assert!(faction_roster.set_religion(1, &custom_religion_key(id), religions));
        }
        civ.religions
            .set_axes(id, AxisOverrides { membership: Some(Membership::Birth), ..Default::default() })
            .unwrap();
        let r = civ.belief_regime();
        let t = &r.terms[r.of[0].unwrap()];
        let sea = cartalith_civ::belief::religion_index("sea_lords").unwrap();
        let one = cartalith_civ::belief::religion_index("one_god").unwrap();
        assert_eq!(t.retention(sea), 0.75, "the variant's own slot keeps its birth retention");
        assert_eq!(t.conversion(one), 0.1, "and a minority takes the persecuting multiplier");
        // An unknown key (a newer archive's) reads as no policy.
        civ.faction_roster.0[1].religious_policy = "future_policy".to_string();
        let r = civ.belief_regime();
        assert_eq!(r.terms[r.of[0].unwrap()].conversion(one), 1.0);
    }
}
