//! The mutable civ **roster** state — `PARITY_AUDIT.md` §5 items 2, 3, 7, 9
//! and 10, and `GUI_GAP_REGISTER.md` CV-07/MS-13/ED-03.
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `civ_tools_bridge.rs` and `sculpt_bridge.rs` argue for: `lib.rs` owns
//! the thin `Variant` conversion and the `#[func]` surface, this module
//! owns the state and the pure helpers, with its own `#[cfg(test)]` suite
//! exercised by `cargo test -p cartalith-godot`.
//!
//! ## Why the roster lives here and not in `cartalith-civ`
//!
//! Because `cartalith-civ` is stateless (`ARCHITECTURE.md`), exactly as
//! `next_tid` and `CivTools::territory_base` already are. The reference
//! keeps five parallel arrays — `CIV_FACTIONS`, `civFactionNames`,
//! `civFactionCulture`, `civFactionReligion`, `civFactionGovernment`,
//! `civFactionAgTech` — all mutated in place by `_civAddFaction` /
//! `_civRemoveFaction` (reference lines 14644-14672). One [`FactionEntry`]
//! per index replaces the five arrays; the *vocabularies* those fields are
//! constrained to are the reference's own tables in
//! `cartalith_civ::roster`, read from there rather than transcribed.
//!
//! ## Why place edits live here too, keyed by `tid`
//!
//! `_civPopulatePlaceEditor` (reference 16694) edits nine things. Four —
//! name, kind, faction, population — are real `NamedSettlement` fields and
//! are written straight onto it. The other five — specialisation,
//! traits, history, an `umAge` override and an `umWalls` override — have no
//! field on `NamedSettlement` at all, and adding them would touch every one
//! of the ~15 places that struct is constructed (mostly tests in three
//! crates) for data the engine does not read.
//!
//! So they go in [`PlaceExtras`], a side table keyed by **`tid`**, not by
//! index: an index is invalidated by the very delete this module exists to
//! add, while `tid` is the stable identity `TIMELINE_SCOPE.md` milestone 1
//! already built for exactly this reason.
//!
//! **What that costs, stated rather than hidden:** an edited
//! `specialisation` does *not* reach `civ_faction_aggregates`' sector
//! output, even though `FactionPlace::specialisation` is a field it reads.
//! Wiring it would change already-golden economy numbers on a user edit,
//! which is a decision, not a detail — see `GUI_GAP_REGISTER.md` ED-03.
//! An edited `umWalls`/`umAge` likewise reaches nothing: their only
//! consumers are urban-morphology functions (`_umWallSpec`, `_umInferAge`)
//! that milestones 8-17 have not ported.

use std::collections::HashMap;

use cartalith_civ::NamedSettlement;
use cartalith_civ::roster::{
    AG_TECH_LEVELS, CIV_FACTION_BASE, CIV_GOVERNMENTS, CIV_RELIGIONS, CIV_SPECIALISATIONS,
    CIV_TRAITS, civ_faction_color, has_key,
};

/// One roster row. Index 0 is the reference's fixed "Unclaimed" entry and
/// is never removable.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionEntry {
    /// `civFactionNames[i]`.
    pub name: String,
    /// `civFactionCulture[i]` — a `cartalith_civ::CIV_CULTURES` key.
    pub culture: String,
    /// `civFactionReligion[i]`, default `"none"`.
    pub religion: String,
    /// `civFactionGovernment[i]`; the reference seeds index 0 `"none"` and
    /// every other index `"monarchy"` (line 14805).
    pub government: String,
    /// `civFactionAgTech[i]`, default `"traditionalAgrarian"`.
    pub ag_tech: String,
    /// `CIV_FACTIONS[i][1]` for the base seven, `_civFactionColor(i)` past
    /// them — the reference's own rule, so an appended faction stays
    /// visually distinct without a colour picker.
    pub color: (u8, u8, u8),
}

impl FactionEntry {
    /// The reference's own defaults for index `i`, whether it is one of the
    /// seven base rows or an appended one.
    pub fn default_for(i: usize) -> Self {
        let (name, color) = match CIV_FACTION_BASE.get(i) {
            Some(&(n, c)) => (n.to_string(), c),
            None => (format!("Faction {i}"), civ_faction_color(i)),
        };
        FactionEntry {
            name,
            culture: cartalith_civ::civ_default_culture(i as i32).key.to_string(),
            religion: "none".to_string(),
            government: if i == 0 { "none" } else { "monarchy" }.to_string(),
            ag_tech: "traditionalAgrarian".to_string(),
            color,
        }
    }
}

/// The whole roster, index 0 = "Unclaimed". `len() - 1` is the reference's
/// `CIV_FACTIONS.length - 1`, i.e. the number of real assignable factions.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionRoster(pub Vec<FactionEntry>);

impl FactionRoster {
    /// A roster of `count` real factions plus Unclaimed at index 0.
    pub fn seeded(count: usize) -> Self {
        FactionRoster((0..=count).map(FactionEntry::default_for).collect())
    }

    /// Real assignable factions (`1..=n`), excluding Unclaimed.
    pub fn count(&self) -> usize {
        self.0.len().saturating_sub(1)
    }

    /// `_civAddFaction` (reference 14644): append one at the next index
    /// with the reference's own defaults. Returns its id.
    pub fn add(&mut self) -> usize {
        let i = self.0.len();
        self.0.push(FactionEntry::default_for(i));
        i
    }

    /// `_civRemoveFaction` (reference 14657), including its two real side
    /// effects: nothing may be left pointing at the removed index, so every
    /// settlement and every territory cell using it reverts to Unclaimed
    /// (`0`) rather than dangling.
    ///
    /// Refuses (returns `false`, changing nothing) at the reference's own
    /// floor — `CIV_FACTIONS.length <= 2`, i.e. Unclaimed plus one real
    /// faction.
    ///
    /// The reference also splices the removed index out of its
    /// `mapFilter.factionsOff` visibility set; this port's per-faction map
    /// filtering does not exist yet, so there is no equivalent set to
    /// clean, and none is fabricated.
    pub fn remove_last(
        &mut self,
        settlements: &mut [NamedSettlement],
        territory: &mut [i32],
    ) -> bool {
        if self.0.len() <= 2 {
            return false;
        }
        let idx = (self.0.len() - 1) as i32;
        for s in settlements.iter_mut() {
            if s.placement.faction == idx {
                s.placement.faction = 0;
            }
        }
        for t in territory.iter_mut() {
            if *t == idx {
                *t = 0;
            }
        }
        self.0.pop();
        true
    }

    /// `1..=count()` — a real, assignable faction id.
    pub fn is_assignable(&self, fid: i32) -> bool {
        fid >= 1 && (fid as usize) < self.0.len()
    }

    /// Writes one editable field. Returns `false` (changing nothing) for an
    /// unknown faction, an unknown field key, or a value outside that
    /// field's own reference vocabulary — a typo from GDScript is rejected,
    /// never stored. `name` is free text and only rejected when blank
    /// (trimmed), matching the reference's own `oninput` which stores
    /// whatever is typed but never has an empty roster label to fall back
    /// to.
    pub fn set_field(&mut self, fid: usize, key: &str, value: &str) -> bool {
        let Some(entry) = self.0.get_mut(fid) else {
            return false;
        };
        match key {
            "name" => {
                if value.trim().is_empty() {
                    return false;
                }
                entry.name = value.to_string();
            }
            "culture" => {
                if !cartalith_civ::CIV_CULTURES.iter().any(|c| c.key == value) {
                    return false;
                }
                entry.culture = value.to_string();
            }
            "religion" => {
                if !has_key(&CIV_RELIGIONS, value) {
                    return false;
                }
                entry.religion = value.to_string();
            }
            "government" => {
                if !has_key(&CIV_GOVERNMENTS, value) {
                    return false;
                }
                entry.government = value.to_string();
            }
            "ag_tech" => {
                if !AG_TECH_LEVELS.iter().any(|t| t.key == value) {
                    return false;
                }
                entry.ag_tech = value.to_string();
            }
            _ => return false,
        }
        true
    }

    /// `civFactionReligion[f] !== 'none'` per faction — the one roster field
    /// `cartalith_civ::civ_faction_aggregates` actually reads
    /// (`FactionAggregatesInput::faction_has_religion`). Built here so a
    /// caller that wires that function up does not re-derive the rule.
    pub fn has_religion_flags(&self) -> Vec<bool> {
        self.0.iter().map(|e| e.religion != "none").collect()
    }
}

/// The five place-editor fields `NamedSettlement` has no room for. See this
/// module's own doc comment for why they live beside it rather than on it,
/// and what that costs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlaceExtras {
    /// `p.specialisation`, a `CIV_SPECIALISATIONS` key. Empty means the
    /// reference's own `'none'` default.
    pub specialisation: String,
    /// `p.traits`, `CIV_TRAITS` keys, insertion-ordered like the
    /// reference's own array (it `push`es and `splice`s, never sorts).
    pub traits: Vec<String>,
    /// `p.history` — free text, `v0.62`.
    pub history: String,
    /// `p.umAge`: `None` = the reference's "auto (infer from population)".
    /// Clamped `30..=1000` exactly as the reference's own `oninput` does.
    pub age: Option<u32>,
    /// `p.umWalls`: `None` = the reference's indeterminate "auto" state,
    /// which its own checkbox visualises and cannot be returned to by
    /// clicking.
    pub walls: Option<bool>,
}

/// Every place edit, keyed by the settlement's stable `tid`.
#[derive(Debug, Clone, Default)]
pub struct PlaceExtrasTable(pub HashMap<u64, PlaceExtras>);

impl PlaceExtrasTable {
    pub fn get(&self, tid: u64) -> PlaceExtras {
        self.0.get(&tid).cloned().unwrap_or_default()
    }

    /// Toggles one trait key on/off, mirroring the reference's own
    /// `indexOf`/`splice`/`push`. Returns `false` for an unknown key.
    pub fn toggle_trait(&mut self, tid: u64, key: &str) -> bool {
        if !CIV_TRAITS.iter().any(|&(k, _, _)| k == key) {
            return false;
        }
        let e = self.0.entry(tid).or_default();
        match e.traits.iter().position(|t| t == key) {
            Some(i) => {
                e.traits.remove(i);
            }
            None => e.traits.push(key.to_string()),
        }
        true
    }

    /// Returns `false` for an unknown specialisation key, changing nothing.
    pub fn set_specialisation(&mut self, tid: u64, key: &str) -> bool {
        if !has_key(&CIV_SPECIALISATIONS, key) {
            return false;
        }
        self.0.entry(tid).or_default().specialisation = key.to_string();
        true
    }

    pub fn set_history(&mut self, tid: u64, text: &str) {
        self.0.entry(tid).or_default().history = text.to_string();
    }

    /// `age < 0` means "back to auto" (`p.umAge = null`); anything else is
    /// clamped to the reference's own `30..=1000`.
    pub fn set_age(&mut self, tid: u64, age: i64) {
        self.0.entry(tid).or_default().age = if age < 0 {
            None
        } else {
            Some(age.clamp(30, 1000) as u32)
        };
    }

    /// `walls < 0` means "back to auto" (`p.umWalls = null`) — an option the
    /// reference's own native checkbox cannot offer, added here because a
    /// `Variant` boundary can carry the third state a DOM checkbox cannot.
    pub fn set_walls(&mut self, tid: u64, walls: i64) {
        self.0.entry(tid).or_default().walls = match walls {
            w if w < 0 => None,
            0 => Some(false),
            _ => Some(true),
        };
    }

    /// Drops the row for a deleted settlement so the table cannot grow
    /// unboundedly across a session of place edits and deletes.
    pub fn forget(&mut self, tid: u64) {
        self.0.remove(&tid);
    }
}

/// `_civPopulatePlaceEditor`'s Delete button (reference 16776-16784), minus
/// its `confirm()` — the shell owns the confirmation dialog, the same split
/// `timeline_bridge::run_collapse_simulation`'s `needs_confirm` already
/// uses. Returns the deleted settlement's `tid`, or `None` for an
/// out-of-range index.
pub fn delete_settlement(settlements: &mut Vec<NamedSettlement>, index: usize) -> Option<u64> {
    if index >= settlements.len() {
        return None;
    }
    Some(settlements.remove(index).tid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::{SettlementKind, SettlementPlacement};

    fn settlement(tid: u64, faction: i32) -> NamedSettlement {
        NamedSettlement {
            tid,
            placement: SettlementPlacement {
                x: 1,
                y: 1,
                suit: 0.5,
                faction,
                capital: false,
                kind: SettlementKind::Town,
                coastal: false,
            },
            name: "T".to_string(),
            pop: 100,
        }
    }

    #[test]
    fn seeded_roster_matches_the_reference_base_table() {
        let r = FactionRoster::seeded(6);
        assert_eq!(r.count(), 6);
        assert_eq!(r.0[0].name, "Unclaimed");
        assert_eq!(r.0[0].government, "none");
        assert_eq!(r.0[1].name, "Aurelia");
        assert_eq!(r.0[1].government, "monarchy");
        assert_eq!(r.0[6].name, "Draumr League");
        // Culture follows `_civDefaultCulture(i)` == CIV_CULTURES[i % 7].
        assert_eq!(r.0[0].culture, "common");
        assert_eq!(r.0[1].culture, "imperial");
    }

    #[test]
    fn add_faction_appends_with_a_generated_name_and_colour() {
        let mut r = FactionRoster::seeded(6);
        let id = r.add();
        assert_eq!(id, 7);
        assert_eq!(r.count(), 7);
        assert_eq!(r.0[7].name, "Faction 7");
        assert_eq!(r.0[7].color, civ_faction_color(7));
        // `_civDefaultCulture(7)` wraps to index 0 -- CIV_CULTURES has 7 entries.
        assert_eq!(r.0[7].culture, "common");
        assert!(r.is_assignable(7));
        assert!(!r.is_assignable(8));
        assert!(!r.is_assignable(0), "Unclaimed is never assignable");
    }

    #[test]
    fn remove_faction_reverts_its_settlements_and_territory_to_unclaimed() {
        let mut r = FactionRoster::seeded(6);
        let mut s = vec![settlement(1, 6), settlement(2, 3)];
        let mut terr = vec![6, 3, 0, 6];
        assert!(r.remove_last(&mut s, &mut terr));
        assert_eq!(r.count(), 5);
        assert_eq!(s[0].placement.faction, 0, "the removed faction's settlement reverts");
        assert_eq!(s[1].placement.faction, 3, "an untouched faction stays");
        assert_eq!(terr, vec![0, 3, 0, 0]);
    }

    #[test]
    fn remove_faction_refuses_at_the_reference_floor() {
        let mut r = FactionRoster::seeded(1);
        let mut s: Vec<NamedSettlement> = vec![];
        let mut t: Vec<i32> = vec![];
        assert!(!r.remove_last(&mut s, &mut t), "Unclaimed + 1 is the floor");
        assert_eq!(r.count(), 1);
        let mut r2 = FactionRoster::seeded(2);
        assert!(r2.remove_last(&mut s, &mut t));
        assert!(!r2.remove_last(&mut s, &mut t));
    }

    #[test]
    fn set_field_rejects_values_outside_the_reference_vocabulary() {
        let mut r = FactionRoster::seeded(2);
        assert!(r.set_field(1, "religion", "sea_lords"));
        assert_eq!(r.0[1].religion, "sea_lords");
        assert!(!r.set_field(1, "religion", "cargo_cult"));
        assert_eq!(r.0[1].religion, "sea_lords", "a rejected value changes nothing");
        assert!(r.set_field(1, "government", "republic"));
        assert!(!r.set_field(1, "government", "technocracy"));
        assert!(r.set_field(1, "culture", "maritime"));
        assert!(!r.set_field(1, "culture", "atlantean"));
        assert!(r.set_field(1, "ag_tech", "earlyIndustrial"));
        assert!(!r.set_field(1, "ag_tech", "fusion"));
        assert!(r.set_field(1, "name", "Thalassa"));
        assert!(!r.set_field(1, "name", "   "), "blank names are refused");
        assert_eq!(r.0[1].name, "Thalassa");
        assert!(!r.set_field(1, "colour", "red"), "unknown field key");
        assert!(!r.set_field(99, "name", "X"), "unknown faction");
    }

    #[test]
    fn has_religion_flags_track_the_one_field_aggregates_read() {
        let mut r = FactionRoster::seeded(2);
        assert_eq!(r.has_religion_flags(), vec![false, false, false]);
        r.set_field(2, "religion", "old_gods");
        assert_eq!(r.has_religion_flags(), vec![false, false, true]);
    }

    #[test]
    fn traits_toggle_on_and_off_and_reject_unknown_keys() {
        let mut t = PlaceExtrasTable::default();
        assert!(t.toggle_trait(7, "port"));
        assert!(t.toggle_trait(7, "mining"));
        assert_eq!(t.get(7).traits, vec!["port", "mining"], "insertion order, not sorted");
        assert!(t.toggle_trait(7, "port"));
        assert_eq!(t.get(7).traits, vec!["mining"]);
        assert!(!t.toggle_trait(7, "haunted"));
        assert_eq!(t.get(7).traits, vec!["mining"], "a rejected key changes nothing");
    }

    #[test]
    fn extras_defaults_and_clamps_match_the_reference() {
        let mut t = PlaceExtrasTable::default();
        assert_eq!(t.get(1), PlaceExtras::default(), "unknown tid reads as defaults");
        assert!(t.set_specialisation(1, "vineyard"));
        assert!(!t.set_specialisation(1, "cheese"));
        assert_eq!(t.get(1).specialisation, "vineyard");
        t.set_age(1, 5);
        assert_eq!(t.get(1).age, Some(30), "clamped to the reference's own 30 floor");
        t.set_age(1, 5000);
        assert_eq!(t.get(1).age, Some(1000), "clamped to the reference's own 1000 ceiling");
        t.set_age(1, -1);
        assert_eq!(t.get(1).age, None, "negative means back to auto");
        t.set_walls(1, 1);
        assert_eq!(t.get(1).walls, Some(true));
        t.set_walls(1, 0);
        assert_eq!(t.get(1).walls, Some(false));
        t.set_walls(1, -1);
        assert_eq!(t.get(1).walls, None);
        t.set_history(1, "Founded in fire.");
        assert_eq!(t.get(1).history, "Founded in fire.");
    }

    #[test]
    fn delete_settlement_removes_exactly_one_and_reports_its_tid() {
        let mut s = vec![settlement(11, 1), settlement(22, 2), settlement(33, 3)];
        assert_eq!(delete_settlement(&mut s, 1), Some(22));
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].tid, 11);
        assert_eq!(s[1].tid, 33);
        assert_eq!(delete_settlement(&mut s, 9), None, "out of range deletes nothing");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn extras_are_forgotten_with_their_settlement() {
        let mut t = PlaceExtrasTable::default();
        t.set_history(42, "gone");
        t.forget(42);
        assert_eq!(t.get(42).history, "");
    }
}
