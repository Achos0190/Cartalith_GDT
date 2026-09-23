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
//! An edited `umWalls`/`umAge`, by contrast, **does** reach the engine, and
//! this paragraph said the opposite until 2026-09-03. Its reason -- "their
//! only consumers are urban-morphology functions (`_umWallSpec`,
//! `_umInferAge`) that milestones 8-17 have not ported" -- stopped being
//! true when those milestones landed. Both are read today by
//! `civ_military_bridge::defences` (`um_wall_spec`/`um_infer_walls` per
//! settlement) and by `urban_bridge::urban_layouts` and
//! `::settlement_diagnostics`, the last of which hands them to
//! `cartalith_civ::urban_adapter::settlement_layout_with` as
//! `PlaceOverrides::walls_override`/`age_override`, so an ED-03 edit changes
//! the generated town.

use std::collections::HashMap;

use cartalith_civ::NamedSettlement;
use cartalith_civ::roster::{
    AG_TECH_LEVELS, CIV_FACTION_BASE, CIV_GOVERNMENTS, CIV_RELIGIONS, CIV_SPECIALISATIONS,
    CIV_TRAITS, civ_default_government, civ_default_religion, civ_faction_color, has_key,
};

/// One roster row. Index 0 is the reference's fixed "Unclaimed" entry and
/// is never removable.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionEntry {
    /// `civFactionNames[i]`.
    pub name: String,
    /// `civFactionCulture[i]` — a `cartalith_civ::CIV_CULTURES` key.
    pub culture: String,
    /// `civFactionReligion[i]`. The reference always seeds `"none"`; this
    /// port instead assigns a real default via
    /// [`cartalith_civ::roster::civ_default_religion`] for every index but
    /// `0` ("Unclaimed" stays `"none"`), authorised 2026-09-23 — see
    /// [`FactionEntry::default_for`].
    pub religion: String,
    /// `civFactionGovernment[i]`. The reference seeds index 0 `"none"` and
    /// every other index `"monarchy"` (line 14805); this port instead
    /// spreads a real default across [`CIV_GOVERNMENTS`] via
    /// [`cartalith_civ::roster::civ_default_government`] for every index but
    /// `0`, authorised 2026-09-23 — see [`FactionEntry::default_for`].
    pub government: String,
    /// `civFactionAgTech[i]`, default `"traditionalAgrarian"`.
    pub ag_tech: String,
    /// `CIV_FACTIONS[i][1]` for the base seven, `_civFactionColor(i)` past
    /// them — the reference's own rule, so an appended faction stays
    /// visually distinct without a colour picker.
    pub color: (u8, u8, u8),
    /// The user's own identity colour for this faction, or `None` for
    /// "whatever the palette rule gives it" — `GUI_GAP_REGISTER.md`
    /// **CV-21**, and v3's CIVIL-owns-the-colour half.
    ///
    /// **Why this is a second field rather than a write into [`color`]
    /// above.** `color` is the *reference's* table, and this port does not
    /// render in it: `lib.rs`'s `FACTION_RGB` (Okabe-Ito, colourblind-safe)
    /// is what territory and the political-control field actually draw,
    /// a divergence that predates this module and is disclosed at both
    /// ends. Overwriting `color` would silently make the reference table
    /// the render palette for edited factions and not for unedited ones —
    /// two rules in one roster. So the override is its own field, the
    /// render rule is untouched at rest, and `None` is exactly today's
    /// behaviour.
    pub color_override: Option<(u8, u8, u8)>,
    /// IN-13 tariffs this faction levies **as importer**, keyed by the
    /// exporting faction's id: rate `0..=1` (Ruling AE,
    /// `LARGE_ITEM_RULINGS.md`). Its own relationship field, deliberately
    /// not `civ_faction_relations`' score, which is documented as not
    /// diplomacy. Stored on the importer's row because a tariff is the
    /// importer's policy; `(a, b)` and `(b, a)` are independent.
    ///
    /// Empty is the default and the untariffed match. A row is never kept
    /// at `0.0` — [`FactionRoster::set_tariff`] removes it — so an
    /// unedited roster saves exactly as it did before this field existed.
    pub tariffs: std::collections::BTreeMap<usize, f64>,
}

impl FactionEntry {
    /// Index `i`'s defaults, whether it is one of the seven base rows or an
    /// appended one.
    ///
    /// **Wired 2026-09-23** (owner: "Should this go live? — Yes, wire it in
    /// now"), replacing the reference's own uniform `religion: "none"` /
    /// `government: "monarchy"` seed for index >= 1 with
    /// [`cartalith_civ::roster::civ_default_religion`] and
    /// [`cartalith_civ::roster::civ_default_government`] — both pure,
    /// RNG-free functions of `culture`/`i` that were built and golden-tested
    /// against `belief::compat` before this call was added (see their own
    /// doc comments in `cartalith_civ::roster` for the culture-fit and
    /// index-cycling rules). Index `0` ("Unclaimed") is unaffected: it stays
    /// `"none"`/`"none"`, the one case neither function is asked about.
    ///
    /// This is a real generated-output change, not a flavour one —
    /// `CIV_GOVERNMENTS`'s own doc comment on this crate's `cartalith-civ`
    /// side explains why (`manpower::GOVERNMENT_EXTRACTION`/`CITIZEN_SHARE`
    /// and `belief::faction_has_religion` both key off these fields).
    pub fn default_for(i: usize) -> Self {
        let (name, color) = match CIV_FACTION_BASE.get(i) {
            Some(&(n, c)) => (n.to_string(), c),
            None => (format!("Faction {i}"), civ_faction_color(i)),
        };
        let culture = cartalith_civ::civ_default_culture(i as i32).key.to_string();
        let (religion, government) = if i == 0 {
            ("none".to_string(), "none".to_string())
        } else {
            (civ_default_religion(&culture, i).to_string(), civ_default_government(i).to_string())
        };
        FactionEntry {
            name,
            culture,
            religion,
            government,
            ag_tech: "traditionalAgrarian".to_string(),
            color,
            color_override: None,
            tariffs: Default::default(),
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
        // A tariff on the removed faction's goods would otherwise outlive
        // it and silently re-apply to whatever `add()` later puts at `idx`.
        for e in self.0.iter_mut() {
            e.tariffs.remove(&(idx as usize));
        }
        true
    }

    /// Sets the rate `importer` levies on goods from `exporter`. Refuses
    /// (returning `false`, changing nothing) an unknown faction on either
    /// side, Unclaimed as importer (it has no government to levy with), a
    /// faction taxing itself, and a rate outside `0..=1` or `NaN`. Rate
    /// `0.0` removes the row, so "no tariff" has exactly one encoding.
    pub fn set_tariff(&mut self, importer: usize, exporter: usize, rate: f64) -> bool {
        let n = self.0.len();
        if importer == 0 || importer >= n || exporter >= n || importer == exporter {
            return false;
        }
        if !(0.0..=1.0).contains(&rate) {
            return false;
        }
        let row = &mut self.0[importer].tariffs;
        if rate == 0.0 {
            row.remove(&exporter);
        } else {
            row.insert(exporter, rate);
        }
        true
    }

    /// Every nonzero tariff, as the engine's sparse rows.
    pub fn tariff_rows(&self) -> Vec<cartalith_civ::trade::Tariff> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(i, e)| {
                e.tariffs.iter().map(move |(&x, &rate)| cartalith_civ::trade::Tariff {
                    importer: i as i32,
                    exporter: x as i32,
                    rate,
                })
            })
            .collect()
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

    /// Sets (`Some`) or clears (`None`) faction `fid`'s identity colour —
    /// `GUI_GAP_REGISTER.md` **CV-21**. Returns `false`, changing nothing,
    /// for an unknown faction and for index 0: "Unclaimed" is not a faction
    /// and nothing renders it, so a colour for it would be a control with no
    /// output.
    pub fn set_color(&mut self, fid: usize, color: Option<(u8, u8, u8)>) -> bool {
        if fid == 0 {
            return false;
        }
        let Some(entry) = self.0.get_mut(fid) else {
            return false;
        };
        entry.color_override = color;
        true
    }

    /// Whether any faction carries a user colour. The renderer's own cheap
    /// "is this world still on the default palette?" question, and what the
    /// dock's *Reset all* row gates on.
    pub fn any_color_override(&self) -> bool {
        self.0.iter().any(|e| e.color_override.is_some())
    }

    /// `civFactionReligion[f] !== 'none'` per faction — the one roster field
    /// `cartalith_civ::civ_faction_aggregates` actually reads
    /// (`FactionAggregatesInput::faction_has_religion`). Built here so a
    /// caller that wires that function up does not re-derive the rule.
    pub fn has_religion_flags(&self) -> Vec<bool> {
        self.0.iter().map(|e| e.religion != "none").collect()
    }

    /// **Ruling V** (`LARGE_ITEM_RULINGS.md`, 2026-09-21): a GeoJSON import
    /// feature naming a faction this roster doesn't have creates it, rather
    /// than being remapped to the nearest existing name or dropped into
    /// "unclaimed." This is the resolve-or-create step both `geojson_apply`
    /// call sites (settlement, territory) share.
    ///
    /// Matching is **exact**, after trimming — never fuzzy, never
    /// case-folded. The ruling explicitly rules out a fuzzy remap, and a
    /// case-fold is the same hazard at a smaller scale (an importer that
    /// meant two distinct factions named "Aurelia" and "AURELIA" would
    /// silently lose one). `name` blank (after trimming) resolves to
    /// faction `0` ("Unclaimed") and creates nothing — the reference's own
    /// "no faction on this feature" case, not an unknown name to invent a
    /// faction for.
    ///
    /// Returns `(fid, Some(name))` when a new faction was appended — the
    /// imported name verbatim, so a caller can report exactly what was
    /// created — or `(fid, None)` when an existing row matched (including
    /// the blank/"Unclaimed" case, `fid == 0`).
    ///
    /// A created faction gets [`FactionEntry::default_for`]'s ordinary
    /// defaults (culture/religion/government/ag-tech, and a colour via
    /// [`civ_faction_color`] at its new index — guaranteed distinct from
    /// every earlier index by that function's own golden-angle hue
    /// rotation) with only `name` overridden to the imported string. Its
    /// territory is whatever the caller paints afterward; nothing here
    /// claims a cell.
    pub fn find_or_create_by_name(&mut self, name: &str) -> (usize, Option<String>) {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unclaimed") {
            return (0, None);
        }
        if let Some(i) = self.0.iter().position(|e| e.name == trimmed) {
            return (i, None);
        }
        let i = self.0.len();
        self.0.push(FactionEntry { name: trimmed.to_string(), ..FactionEntry::default_for(i) });
        (i, Some(trimmed.to_string()))
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
        assert_eq!(r.0[0].religion, "none", "Unclaimed is untouched by the 2026-09-23 wiring");
        assert_eq!(r.0[1].name, "Aurelia");
        // The reference's own uniform "monarchy" is gone (wired 2026-09-23,
        // `MISTAKES.md` "Change generated output"): `civ_default_government`
        // walks `faction_index - 1` mod the eight non-"none" forms, so index
        // 1 lands on the first of them, "chiefdom".
        assert_eq!(r.0[1].government, "chiefdom");
        // `civ_default_culture(1)` is "imperial", one of the two unthemed
        // cultures, so `civ_default_religion` cycles the seven-way tie by
        // `faction_index`: index 1 -> the second candidate, "earth_mother".
        assert_eq!(r.0[1].religion, "earth_mother");
        assert_eq!(r.0[6].name, "Draumr League");
        // Culture follows `_civDefaultCulture(i)` == CIV_CULTURES[i % 7].
        assert_eq!(r.0[0].culture, "common");
        assert_eq!(r.0[1].culture, "imperial");
    }

    /// The wiring boundary itself, built and authorised 2026-09-23: every
    /// entry point onto [`FactionEntry::default_for`] -- `seeded`, `add`,
    /// and (via [`FactionRoster::find_or_create_by_name`], its own test
    /// above) GeoJSON import -- must hand a genuinely new faction a real
    /// religion and government, never the reference's uniform
    /// `"none"`/`"monarchy"`, and never touch index `0`.
    #[test]
    fn a_freshly_added_faction_gets_a_real_non_uniform_religion_and_government() {
        let mut r = FactionRoster::seeded(0); // just Unclaimed
        assert_eq!(r.0[0].religion, "none");
        assert_eq!(r.0[0].government, "none");

        let id1 = r.add(); // index 1, culture "imperial" -- unthemed
        let id2 = r.add(); // index 2, culture "highland" -- themed
        assert_eq!((id1, id2), (1, 2));

        for &i in &[id1, id2] {
            assert_ne!(r.0[i].religion, "none", "index {i} must get a real religion");
            assert_ne!(r.0[i].government, "none", "index {i} must get a real government");
            assert!(has_key(&CIV_RELIGIONS, &r.0[i].religion));
            assert!(has_key(&CIV_GOVERNMENTS, &r.0[i].government));
        }
        // Not a collapse onto one value regardless of culture: the themed
        // index 2 ("highland") is forced onto "sky_pantheon" by
        // `belief::compat`'s bijection, which the unthemed index 1
        // ("imperial") cannot land on by construction (`civ_default_religion`
        // only returns a themed culture's forced pick for that culture).
        assert_eq!(r.0[2].religion, "sky_pantheon");
        assert_ne!(r.0[1].religion, r.0[2].religion);
        // Government has no culture correlation, but must still vary with
        // faction index rather than repeating the old uniform "monarchy".
        assert_ne!(r.0[1].government, r.0[2].government);

        // Unclaimed stays exactly as it was.
        assert_eq!(r.0[0].religion, "none");
        assert_eq!(r.0[0].government, "none");
    }

    /// `GUI_GAP_REGISTER.md` **CV-21**. The two things worth pinning are
    /// that a fresh roster is *empty* of overrides (so the render palette is
    /// bit-identical to what it was before this field existed) and that
    /// Unclaimed refuses one.
    #[test]
    fn an_identity_colour_is_opt_in_and_reversible() {
        let mut r = FactionRoster::seeded(6);
        assert!(!r.any_color_override(), "a fresh roster renders on the palette rule, unchanged");
        assert!(r.0.iter().all(|e| e.color_override.is_none()));

        assert!(r.set_color(2, Some((10, 20, 30))));
        assert_eq!(r.0[2].color_override, Some((10, 20, 30)));
        assert!(r.any_color_override());
        // The reference table underneath is untouched -- it is not the
        // render palette and this does not make it one.
        assert_eq!(r.0[2].color, CIV_FACTION_BASE[2].1);

        assert!(r.set_color(2, None), "clearing is the same call");
        assert!(!r.any_color_override());

        assert!(!r.set_color(0, Some((1, 2, 3))), "Unclaimed is not a faction");
        assert_eq!(r.0[0].color_override, None);
        assert!(!r.set_color(99, Some((1, 2, 3))), "unknown faction");
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
    fn set_tariff_validates_and_zero_removes_the_row() {
        let mut r = FactionRoster::seeded(3);
        assert!(r.tariff_rows().is_empty(), "a fresh roster levies nothing");
        assert!(r.set_tariff(2, 1, 0.25));
        assert!(r.set_tariff(2, 0, 0.5), "goods from Unclaimed land may be taxed");
        assert!(!r.set_tariff(0, 1, 0.25), "Unclaimed has no government to levy with");
        assert!(!r.set_tariff(2, 2, 0.25), "a faction cannot tax itself");
        assert!(!r.set_tariff(4, 1, 0.25), "unknown importer");
        assert!(!r.set_tariff(2, 4, 0.25), "unknown exporter");
        assert!(!r.set_tariff(2, 1, 1.5));
        assert!(!r.set_tariff(2, 1, -0.1));
        assert!(!r.set_tariff(2, 1, f64::NAN));
        assert_eq!(r.0[2].tariffs.get(&1), Some(&0.25), "refusals changed nothing");
        let rows = r.tariff_rows();
        assert_eq!(rows.len(), 2);
        assert!(rows.contains(&cartalith_civ::trade::Tariff { importer: 2, exporter: 1, rate: 0.25 }));
        assert!(r.set_tariff(2, 1, 0.0));
        assert!(r.set_tariff(2, 0, 0.0));
        assert!(r.0[2].tariffs.is_empty(), "rate 0 is the absence of a row, not a stored 0");
        assert_eq!(r, FactionRoster::seeded(3));
    }

    #[test]
    fn removing_a_faction_drops_every_tariff_on_its_goods() {
        let mut r = FactionRoster::seeded(3);
        assert!(r.set_tariff(1, 3, 0.4));
        assert!(r.set_tariff(2, 3, 0.4));
        assert!(r.set_tariff(1, 2, 0.1));
        let (mut s, mut t): (Vec<NamedSettlement>, Vec<i32>) = (vec![], vec![]);
        assert!(r.remove_last(&mut s, &mut t));
        let _ = r.add(); // a new faction at index 3 must not inherit them
        let rows = r.tariff_rows();
        assert_eq!(rows, vec![cartalith_civ::trade::Tariff { importer: 1, exporter: 2, rate: 0.1 }]);
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

    /// Ruling V (2026-09-21): the core case this method exists for. Seeded
    /// past the 7-row `CIV_FACTION_BASE` table so the appended index
    /// exercises `civ_faction_color`'s hue-rotation branch, not the base
    /// table's own hand-picked colours — `FactionEntry::default_for`'s own
    /// rule, unchanged by this method (only `name` is overridden).
    #[test]
    fn find_or_create_by_name_makes_a_new_faction_for_an_unknown_import_name() {
        let mut r = FactionRoster::seeded(7);
        let (fid, created) = r.find_or_create_by_name("Whitestone Confederacy");
        assert_eq!(fid, 8, "appended past Unclaimed + the seeded 7");
        assert_eq!(created, Some("Whitestone Confederacy".to_string()));
        assert_eq!(r.0[8].name, "Whitestone Confederacy");
        assert_eq!(r.count(), 8);
        // Its colour is the ordinary appended-index rule, distinct by
        // construction, and it starts with the ordinary defaults --
        // `civ_default_government`/`civ_default_religion`, wired
        // 2026-09-23, not the reference's uniform "monarchy"/"none". Index
        // 8's culture is `civ_default_culture(8)` == CIV_CULTURES[8 % 7] ==
        // "imperial" (unthemed), the same tie as index 1, and
        // `civ_default_government(8)` walks to the eighth (last) non-"none"
        // form.
        assert_eq!(r.0[8].color, civ_faction_color(8));
        assert_eq!(r.0[8].government, "city_state");
        assert_eq!(r.0[8].religion, "earth_mother");

        // A second feature naming the SAME faction matches, not duplicates.
        let (fid2, created2) = r.find_or_create_by_name("Whitestone Confederacy");
        assert_eq!(fid2, 8);
        assert_eq!(created2, None, "an existing exact match creates nothing");
        assert_eq!(r.count(), 8, "no duplicate appended");
    }

    #[test]
    fn find_or_create_by_name_matches_an_existing_faction_exactly_not_fuzzily() {
        let mut r = FactionRoster::seeded(6);
        let (fid, created) = r.find_or_create_by_name("Aurelia");
        assert_eq!(fid, 1, "matches CIV_FACTION_BASE[1] exactly");
        assert_eq!(created, None);
        assert_eq!(r.count(), 6, "no faction appended for an existing name");

        // A near-miss is NOT fuzzy-matched -- it creates its own faction,
        // exactly the "do not remap by fuzzy name match" half of the ruling.
        let (fid2, created2) = r.find_or_create_by_name("Aurelia ");
        assert_eq!(fid2, 1, "trimmed, so the trailing space is not a mismatch");
        assert_eq!(created2, None);
        let (fid3, created3) = r.find_or_create_by_name("Aurelian Remnant");
        assert_ne!(fid3, 1);
        assert!(created3.is_some(), "a genuinely different name is a genuinely different faction");
        assert_eq!(r.count(), 7);
    }

    #[test]
    fn find_or_create_by_name_resolves_blank_and_unclaimed_without_creating() {
        let mut r = FactionRoster::seeded(3);
        for input in ["", "   ", "Unclaimed", "unclaimed", " UNCLAIMED "] {
            let (fid, created) = r.find_or_create_by_name(input);
            assert_eq!(fid, 0, "{input:?}");
            assert_eq!(created, None, "{input:?}");
        }
        assert_eq!(r.count(), 3, "none of the above appended a faction");
    }

    #[test]
    fn has_religion_flags_track_the_one_field_aggregates_read() {
        let mut r = FactionRoster::seeded(2);
        // Wired 2026-09-23: every real faction (index >= 1) now defaults to
        // a non-"none" religion (`civ_default_religion`) -- only Unclaimed
        // stays "none". An explicit hand edit to "none" must still clear the
        // flag, and a later edit away from it must still set it -- the flag
        // tracks the field's current value, not whether it was ever touched.
        assert_eq!(r.has_religion_flags(), vec![false, true, true]);
        r.set_field(1, "religion", "none");
        assert_eq!(r.has_religion_flags(), vec![false, false, true]);
        r.set_field(2, "religion", "old_gods");
        assert_eq!(r.has_religion_flags(), vec![false, false, true]);
        r.set_field(1, "religion", "sea_lords");
        assert_eq!(r.has_religion_flags(), vec![false, true, true]);
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
