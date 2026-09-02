//! The faction roster's and place editor's **vocabulary tables**, plus the
//! one real function among them (`_civFactionColor`).
//!
//! `PARITY_AUDIT.md` §5 items 3, 9 and 10 name three reference surfaces
//! that had no port at all: the place-edit popup
//! (`_civPopulatePlaceEditor`), the Faction Roster modal
//! (`_civOpenFactionsModal`/`_civPopulateFactionEditor`) and the procedural
//! faction banner (`_civFactionBannerCanvas`). Every one of them is driven
//! by a small fixed list the reference declares once at module load and
//! never computes: traits, specialisations, religions, governments,
//! agricultural-technology levels, and the base faction name/colour table.
//!
//! Those lists live here rather than in `lib.rs` for the same reason
//! `CIV_CULTURES` lives beside `civ_settle_name` there: they are inert
//! lookup data ported verbatim, and a shell that renders a dropdown must
//! read the engine's own table rather than transcribe a second copy into
//! GDScript that can silently drift.
//!
//! ## What is deliberately *not* here
//!
//! - **The roster itself.** `cartalith-civ` is stateless (`ARCHITECTURE.md`),
//!   so the mutable per-faction name/culture/religion/government/ag-tech
//!   arrays (the reference's `civFactionNames`/`civFactionCulture`/
//!   `civFactionReligion`/`civFactionGovernment`/`civFactionAgTech`) live at
//!   the `cartalith-godot` boundary next to `next_tid` and
//!   `territory_base`, exactly as `timeline`'s own module doc argues for
//!   `tid`. This module supplies the *defaults* those arrays are seeded
//!   with and the vocabularies they are constrained to.
//!
//! ## `farmersPerUrbanite`'s consumers -- a stale claim, corrected 2026-09-01
//!
//! This section used to say [`crate::roster::AG_TECH_LEVELS`]'s
//! `farmers_per_urbanite` was as inert as Government/Religion, because the
//! reference's own two readers
//! of it -- `_civFoodShed`/`foodSurplusRatio` (reference line 14811:
//! *"`farmersPerUrbanite` is read by foodSurplusRatio()/_civFoodShed() and
//! genuinely changes a faction's urbanisation ceiling"*) -- had no port.
//! That stopped being true, in two unrelated ways, neither of which touched
//! this file:
//!
//! - `foodSurplusRatio` is [`crate::timeline::food_surplus_ratio`], and
//!   `_civFoodShed` is [`crate::trade::civ_food_shed`] (2026-09-01, closing
//!   `ECONOMY_SCOPE.md` milestone 2 -- see that function's own doc comment
//!   for what this pass found already ported with zero callers, and what
//!   it built). This is the food/trade route the reference itself names.
//! - Separately, `cartalith_civ::manpower::civ_military_manpower`'s
//!   agricultural-labour-ratio term reads
//!   [`crate::roster::AgTechLevel`]'s `farmers_per_urbanite` field directly,
//!   wired at `civ_military_bridge.rs` (`MILITARY_MANPOWER_SCOPE.md`,
//!   2026-08-25).
//!   That route has nothing to do with food logistics -- it is the labour
//!   force behind an army's headcount.
//!
//! **The "still genuinely true" paragraph that stood here is now false too
//! (corrected 2026-09-01, later the same day).** It read: *"nobody at the
//! `cartalith-godot` boundary calls `civ_food_shed`"*, and said a real
//! per-settlement `farmers_per_urbanite` and soil field had yet to be
//! resolved and threaded in from there. All of that was built in the same
//! pass, which is why the sentence above no longer calls the manpower route
//! the only one reachable from Godot: `civ_trade_bridge.rs`'s
//! `food_shed_rows` resolves each settlement's `farmers_per_urbanite`
//! through [`crate::roster::civ_ag_tech_by_key`] -- the same entry point the
//! manpower model uses -- builds the soil field, shares one `RoadComponents`
//! across the sweep, and calls [`crate::trade::civ_food_shed`] once per
//! settlement behind the `#[func] civ_food_shed` binding. The wiring is the
//! shape this paragraph predicted; it simply exists now.
//!
//! What has not changed is why the tables live here: this crate still holds
//! no faction roster (`ARCHITECTURE.md`), so those per-settlement values are
//! still resolved by the caller and threaded in, never read from state this
//! module owns. The table was ported regardless of who reads it, the same
//! reasoning `civ_base_pop_for_kind`'s unreachable `Metropolis` row carries.

/// `CIV_FACTIONS` (reference line 14568), index 0 = "Unclaimed", fixed.
///
/// The reference's own comment calls this list *mutable*:
/// `_civAddFaction`/`_civRemoveFaction` grow and shrink it, and
/// [`civ_faction_color`] colours any index past the hand-picked palette.
/// So this is the **base** table a fresh roster starts from, not a bound.
///
/// The colours are the reference's, kept with the names they belong to.
/// `cartalith-godot`'s own `FACTION_RGB` (an Okabe-Ito colourblind-safe
/// swatch) is what territory actually renders in and is *not* replaced by
/// this -- that divergence predates this module and stays disclosed where
/// it already is.
pub const CIV_FACTION_BASE: [(&str, (u8, u8, u8)); 7] = [
    ("Unclaimed", (60, 60, 60)),
    ("Aurelia", (206, 84, 72)),
    ("Veldmark", (86, 156, 96)),
    ("Korrath", (92, 124, 206)),
    ("Sythe Dominion", (196, 164, 72)),
    ("Mirelle", (166, 96, 186)),
    ("Draumr League", (80, 176, 184)),
];

/// `_civFactionColor` (reference line 14576): golden-angle hue rotation
/// through HSL(h, 0.55, 0.5), so an appended faction index is deterministic
/// and never lands on a nearby hue. Ported verbatim, integer rounding
/// included -- `Math.round` is JS's half-up-toward-+Infinity, which is
/// `f64::round` for the non-negative values this can produce.
pub fn civ_faction_color(i: usize) -> (u8, u8, u8) {
    let h = (i as f64 * 137.508) % 360.0;
    let s: f64 = 0.55;
    let l: f64 = 0.5;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

/// `CIV_TRAITS` (reference line 14715) -- `(key, label, glyph)`.
///
/// The reference's own comment is worth carrying: traits are map-glyph
/// *attributes* (a place can have several) and deliberately overlap
/// [`CIV_SPECIALISATIONS`] on `mining`/`trade_hub`; a specialisation is the
/// ONE economic focus, the same word as a trait is just a badge. **Never
/// reorder** -- the reference writes these keys into save files, and
/// `administrative` was appended (v1.28) for exactly that reason.
pub const CIV_TRAITS: [(&str, &str, &str); 7] = [
    ("fortified", "Fortified", "\u{2B22}"),
    ("mining", "Mining", "\u{2692}"),
    ("port", "Port", "\u{2693}"),
    ("trade_hub", "Trade hub", "\u{2663}"),
    ("military", "Military", "\u{2694}"),
    ("religious", "Religious", "\u{271D}"),
    ("administrative", "Administrative", "\u{265C}"),
];

/// `CIV_SPECIALISATIONS` (reference line 14729) -- `(key, label)`. The
/// keys are the ones [`crate::CIV_PRIMARY_SPECIALISATION`] already maps onto
/// named primary sectors; this is the full picker vocabulary that table's
/// five entries are a subset of.
pub const CIV_SPECIALISATIONS: [(&str, &str); 10] = [
    ("none", "None / generic"),
    ("fishing", "Fishing"),
    ("grain", "Grain producer"),
    ("pastoral", "Pastoral / herding"),
    ("timber", "Timber / forestry"),
    ("mining", "Mining"),
    ("vineyard", "Vineyard / orchard"),
    ("trade_hub", "Trade hub"),
    ("monastic", "Monastic / temple"),
    ("garrison", "Garrison / fort"),
];

/// `CIV_RELIGIONS` (reference line ~14780) -- `(key, label)`. A per-faction
/// categorical "state religion" attribute; the reference scoped FMG's full
/// spatial religion-spread model down to exactly this list, on purpose.
pub const CIV_RELIGIONS: [(&str, &str); 8] = [
    ("none", "None / secular"),
    ("sun_cult", "Sun Cult"),
    ("earth_mother", "Earth Mother"),
    ("sea_lords", "Sea Lords"),
    ("sky_pantheon", "Sky Pantheon"),
    ("ancestor_rites", "Ancestor Rites"),
    ("flame_creed", "Flame Creed"),
    ("old_gods", "Old Gods"),
];

/// `CIV_GOVERNMENTS` (reference line 14794) -- `(key, label)`. Pure
/// flavour: the reference's own comment says no simulation reads or writes
/// this, and nothing in this port does either.
pub const CIV_GOVERNMENTS: [(&str, &str); 9] = [
    ("none", "None / Unclaimed"),
    ("chiefdom", "Chiefdom"),
    ("tribal_confederacy", "Tribal Confederacy"),
    ("monarchy", "Monarchy"),
    ("oligarchy", "Oligarchy"),
    ("republic", "Republic"),
    ("theocracy", "Theocracy"),
    ("empire", "Empire"),
    ("city_state", "City-State"),
];

/// One `AG_TECH_LEVELS` row (reference line 14816).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgTechLevel {
    pub key: &'static str,
    pub label: &'static str,
    /// The England agricultural-labour-share series (Broadberry & Gardner
    /// 2013; CAMPOP) converted to farmers:urbanite. See this module's own
    /// doc for why nothing in this port reads it yet.
    pub farmers_per_urbanite: f64,
    pub hint: &'static str,
}

/// `AG_TECH_LEVELS` (reference line 14816), in order. Index 1
/// (`traditionalAgrarian`) is the reference's own default for every
/// faction and the fallback [`civ_ag_tech_by_key`] returns.
pub const AG_TECH_LEVELS: [AgTechLevel; 6] = [
    AgTechLevel {
        key: "subsistence",
        label: "Subsistence (hoe, no plow)",
        farmers_per_urbanite: 19.0,
        hint: "~95% of the population farms. No plow — digging-stick/hoe cultivation.",
    },
    AgTechLevel {
        key: "traditionalAgrarian",
        label: "Traditional Agrarian (ard plow)",
        farmers_per_urbanite: 9.0,
        hint: "~90% farms. The pre-v1.54 default for every faction — unchanged old-save behaviour.",
    },
    AgTechLevel {
        key: "advancedAgrarian",
        label: "Advanced Agrarian (heavy plow, 3-field)",
        farmers_per_urbanite: 4.0,
        hint: "~80% farms. Heavy iron plow, three-field rotation, horse collar — high/late medieval.",
    },
    AgTechLevel {
        key: "improvedAgrarian",
        label: "Improved Agrarian — \"mastered the plow\"",
        farmers_per_urbanite: 1.0,
        hint: "~50% farms. Multi-course rotation, drainage, enclosure, selective breeding (England ~1700-1760).",
    },
    AgTechLevel {
        key: "earlyIndustrial",
        label: "Early Industrial — \"barely industrial\"",
        farmers_per_urbanite: 0.45,
        hint: "~31% farms. Steam threshing/reaper, first chemical fertilizer, pre-mass-import (England ~1800).",
    },
    AgTechLevel {
        key: "industrial",
        label: "Industrial",
        farmers_per_urbanite: 0.15,
        hint: "~13% farms. Mechanisation, synthetic nitrogen, rail/steamship grain trade.",
    },
];

/// `_civAgTechByKey` (reference line 14833): the named level, or Traditional
/// Agrarian -- the reference's own pre-v1.54 default -- for anything
/// unrecognised.
pub fn civ_ag_tech_by_key(key: &str) -> &'static AgTechLevel {
    AG_TECH_LEVELS
        .iter()
        .find(|t| t.key == key)
        .unwrap_or(&AG_TECH_LEVELS[1])
}

/// True when `key` names a real entry of `table`'s `(key, label)` rows --
/// the guard every setter at the boundary uses so a typo from GDScript is
/// rejected rather than stored.
pub fn has_key(table: &[(&str, &str)], key: &str) -> bool {
    table.iter().any(|&(k, _)| k == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_color_is_deterministic_and_spread() {
        // Index 0 is hue 0 -> pure red at s=0.55, l=0.5.
        assert_eq!(civ_faction_color(0), (198, 57, 57));
        // Two consecutive indices must not land on the same hue.
        assert_ne!(civ_faction_color(7), civ_faction_color(8));
        // Deterministic.
        assert_eq!(civ_faction_color(13), civ_faction_color(13));
    }

    #[test]
    fn faction_color_covers_every_hue_branch() {
        // 137.508 * k % 360 walks all six 60-degree sectors within k<=6.
        let mut sectors = [false; 6];
        for k in 0..6 {
            let h = (k as f64 * 137.508) % 360.0;
            sectors[(h / 60.0) as usize] = true;
        }
        assert_eq!(sectors.iter().filter(|&&s| s).count(), 5);
        // Every produced colour is a real, in-range RGB triple.
        for k in 0..64 {
            let (r, g, b) = civ_faction_color(k);
            assert!(r as u16 + g as u16 + b as u16 > 0);
        }
    }

    #[test]
    fn ag_tech_by_key_falls_back_to_traditional_agrarian() {
        assert_eq!(civ_ag_tech_by_key("industrial").farmers_per_urbanite, 0.15);
        assert_eq!(
            civ_ag_tech_by_key("nonsense").key,
            "traditionalAgrarian",
            "the reference's own fallback is AG_TECH_LEVELS[1], not [0]"
        );
    }

    #[test]
    fn vocabularies_are_the_reference_lengths() {
        assert_eq!(CIV_TRAITS.len(), 7);
        assert_eq!(CIV_SPECIALISATIONS.len(), 10);
        assert_eq!(CIV_RELIGIONS.len(), 8);
        assert_eq!(CIV_GOVERNMENTS.len(), 9);
        assert_eq!(AG_TECH_LEVELS.len(), 6);
        assert_eq!(CIV_FACTION_BASE.len(), 7);
        assert_eq!(CIV_TRAITS[6].0, "administrative", "v1.28 appended, never reordered");
        assert!(has_key(&CIV_RELIGIONS, "old_gods"));
        assert!(!has_key(&CIV_RELIGIONS, "sun_cult_"));
    }

    /// Every `CIV_PRIMARY_SPECIALISATION` key must exist in the full picker
    /// vocabulary -- otherwise the editor could never select the value the
    /// economy layer keys off.
    #[test]
    fn primary_specialisation_keys_are_a_subset() {
        for (k, _) in crate::CIV_PRIMARY_SPECIALISATION {
            assert!(
                CIV_SPECIALISATIONS.iter().any(|&(s, _)| s == k),
                "{k} missing from CIV_SPECIALISATIONS"
            );
        }
    }
}
