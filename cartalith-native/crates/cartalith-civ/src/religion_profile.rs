//! FH-R2 milestone R2-1's **`ReligionProfile` table**: each built-in religion's
//! position on the structural axes of the owner-requested research document
//! `RELIGION_GOVERNMENT_EXPANSION_RESEARCH.md` ("Real-world grounding: the
//! structural axes", axes 1-5), plus the design canvas's TYPE column
//! (organized / folk / cult / heresy, `Religion.dc.html`).
//!
//! **What it is.** A parallel table keyed by [`crate::roster::CIV_RELIGIONS`]
//! key, in the same shape as `belief::CIV_RELIGION_DOMAIN` and
//! [`crate::CIV_CULTURE_TERRAIN_KEY`]: data beside the vocabulary rather than
//! fields on it, so the vocabulary's `(key, label)` tuples (what saves store and
//! what every reader indexes) do not change shape.
//!
//! **Where each value comes from (Ruling BK).** Approved by the owner in
//! Ruling CJ (`LARGE_ITEM_RULINGS.md`, 2026-10-18), which approved
//! `FH_R2_PLAN.md` Q1 at its default:
//!
//! - **The reference's seven religions** (`sun_cult` .. `old_gods`): object,
//!   exclusivity, locus and TYPE are the research document's draft table "The
//!   existing 8, classified", transcribed; `flame_creed` is a **dualist,
//!   exclusive-leaning** religion, Q1's default, which Ruling CJ named. That
//!   draft gives **no membership mode and no institutional form** for any of
//!   the seven, so both are `None` here ("not classified"), never a guessed
//!   value: an owner authoring pass supplies them or nobody does.
//! - **The seven R2-1 religions** (`one_god` .. `god_king`): exclusivity,
//!   membership, locus and institution are read from the research document's
//!   "7 new entries" table's **Structure** column (written there as
//!   `exclusivity · membership · locus · institution`, e.g. `one_god`'s
//!   "exclusive · universal · portable+civic · hierarchical clergy"), and the
//!   object of worship from its **Type** column. A part the row does not state
//!   is `None`. Their **TYPE** is not in the research document; it is derived
//!   here by one stated rule, labelled on each row: *organized* where the row
//!   names a clergy, priesthood, monastery or scholarly institution, *cult*
//!   where its own Type text says cult, *folk* where it names no institution
//!   beyond a specialist; `None` where the row names none at all
//!   (`covenant_folk`). That derivation is this pass's, for the owner to
//!   overturn row by row.
//! - **`none`** has no row: secularism is not a religion with a structure.
//!   [`religion_profile`] returns `None` for it, and every reader must treat
//!   that as "no profile", not as a profile of defaults.
//!
//! **What reads it.** `belief.rs`'s FH-R2 R2-2 terms (retention from
//! [`Membership`], competition from [`Exclusivity`] and [`Membership`]) and the
//! shell's Religion table (TYPE, through `civ_religion_vocabulary`). Object,
//! locus and institution are carried for FH-R2's later stages (S3's site
//! threshold and siting rule, `FH_R2_PLAN.md` §2.2) and are read by nothing in
//! the simulation today.
//!
//! **What it must never do.** Never gain a row value with no source named in
//! its `basis`; never fill a `None` with a plausible default (`MISTAKES.md`,
//! "Return, marshal or default a value that can be absent"); never be reordered
//! or keyed by index (it is looked up by key); never carry a terrain verdict
//! (that is `belief::CIV_RELIGION_DOMAIN`'s job, and the seven R2-1 religions
//! have none by their own source's word).

/// Axis 1, "object of worship" (research document: Tylor, Max Müller).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorshipObject {
    Monotheism,
    /// Henotheism / monolatry: one god supreme among others.
    Henotheism,
    Polytheism,
    Dualism,
    Animism,
    AncestorVeneration,
    /// Non-theistic soteriology (the research document's Buddhism/Jainism row).
    NonTheistic,
    RulerCult,
}

/// Axis 2, exclusivity (research document: Jan Assmann's "Mosaic
/// distinction"). The axis the research document calls "the key pluralism
/// axis".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exclusivity {
    /// Belonging rules out other belonging.
    Exclusive,
    /// The research document's "exclusive-leaning" (`flame_creed`, Ruling CJ).
    ExclusiveLeaning,
    /// "Exclusive for monks, additive for laity" (`the_path`'s own row).
    Mixed,
    /// Additive / cumulative: worship many, cults stack.
    Additive,
}

impl Exclusivity {
    /// Whether the religion draws a boundary around its members: `Exclusive`
    /// or `ExclusiveLeaning`. `Mixed` is not counted, because its laity is
    /// additive by its own row's word; it is not counted as additive either
    /// ([`Self::is_additive`]), so it sits between the two.
    pub fn is_exclusive(self) -> bool {
        matches!(self, Exclusivity::Exclusive | Exclusivity::ExclusiveLeaning)
    }

    /// Whether the religion is additive (cults stack). Only `Additive`.
    pub fn is_additive(self) -> bool {
        matches!(self, Exclusivity::Additive)
    }
}

/// Axis 3, membership mode (research document: "world" vs "ethnic" religion,
/// Weber).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    /// Birth / ethnic: one is born into it, and it does not proselytise.
    Birth,
    /// Universal / missionary: anyone may join and members are sought.
    Universal,
    /// Initiatory: entered by initiation into a voluntary group.
    Initiatory,
    /// Civic: belonging follows the city or the state.
    Civic,
}

/// Axis 4, locus (research document: Jonathan Z. Smith, *Here, There, and
/// Anywhere*, 2003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locus {
    /// Household / lineage.
    Household,
    Place,
    /// Civic / state.
    Civic,
    /// Portable ("anywhere"): travels with its people.
    Portable,
}

/// Axis 5, institutional form (research document: Weber/Troeltsch;
/// Whitehouse, *Modes of Religiosity*, 2004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Institution {
    HierarchicalClergy,
    TemplePriesthood,
    Monastic,
    Congregational,
    /// A specialist such as a shaman, with no temple.
    Specialist,
}

/// The design canvas's TYPE column (`Religion.dc.html`: organized / folk /
/// cult / heresy). `Heresy` is in the canvas's vocabulary but no built-in
/// religion is one: the research document calls a heresy "an *event* -- a
/// schism spawning a new instance -- not an archetype" (S7, not built).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReligionType {
    Organized,
    Folk,
    Cult,
    Heresy,
}

impl ReligionType {
    /// The lower-case word the canvas prints, which is also the key the shell
    /// receives.
    pub fn key(self) -> &'static str {
        match self {
            ReligionType::Organized => "organized",
            ReligionType::Folk => "folk",
            ReligionType::Cult => "cult",
            ReligionType::Heresy => "heresy",
        }
    }
}

/// One built-in religion's structural profile. Every `Option` field is
/// `None` where the source gives no value (see the module doc); a reader must
/// handle that case rather than substitute a default.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReligionProfile {
    /// A [`crate::roster::CIV_RELIGIONS`] key other than `none`.
    pub key: &'static str,
    /// Axis 1. `None` where the source names no object (`mysteries`).
    pub object: Option<WorshipObject>,
    /// Axis 2. Every row has one: both source tables state it.
    pub exclusivity: Exclusivity,
    /// Axis 3. `None` for the reference's seven (the approved draft has no
    /// membership column).
    pub membership: Option<Membership>,
    /// Axis 4, possibly two values (`one_god`'s "portable+civic"); empty where
    /// the source names none.
    pub locus: &'static [Locus],
    /// Axis 5. `None` where the source names none.
    pub institution: Option<Institution>,
    /// The canvas's TYPE. `None` where the source names no institution and the
    /// derivation rule (module doc) therefore has nothing to read.
    pub kind: Option<ReligionType>,
    /// Where this row's values come from, in words. Non-empty, test-enforced.
    pub basis: &'static str,
}

/// The profile table: one row per built-in religion except `none`, in
/// [`crate::roster::CIV_RELIGIONS`] order (order is a readability choice;
/// lookups are by key).
pub const RELIGION_PROFILES: [ReligionProfile; 14] = [
    ReligionProfile {
        key: "sun_cult",
        object: Some(WorshipObject::Henotheism),
        exclusivity: Exclusivity::Additive,
        membership: None,
        locus: &[Locus::Civic],
        institution: None,
        kind: Some(ReligionType::Organized),
        basis: "Research draft 'The existing 8, classified': henotheism, additive, state, organized \
                (its confidence: soft -- could be monolatry). Approved, Ruling CJ.",
    },
    ReligionProfile {
        key: "earth_mother",
        object: Some(WorshipObject::Polytheism),
        exclusivity: Exclusivity::Additive,
        membership: None,
        locus: &[Locus::Place],
        institution: None,
        kind: Some(ReligionType::Folk),
        basis: "Research draft: goddess-centred polytheism, additive, place, folk (medium). \
                Approved, Ruling CJ.",
    },
    ReligionProfile {
        key: "sea_lords",
        object: Some(WorshipObject::Polytheism),
        exclusivity: Exclusivity::Additive,
        membership: None,
        locus: &[Locus::Place],
        institution: None,
        kind: Some(ReligionType::Folk),
        basis: "Research draft: polytheism, additive, place (coast), folk (medium). Approved, Ruling CJ.",
    },
    ReligionProfile {
        key: "sky_pantheon",
        object: Some(WorshipObject::Polytheism),
        exclusivity: Exclusivity::Additive,
        membership: None,
        locus: &[Locus::Civic],
        institution: None,
        kind: Some(ReligionType::Organized),
        basis: "Research draft: civic pantheon (a polytheism), additive, civic, organized (strong). \
                Approved, Ruling CJ.",
    },
    ReligionProfile {
        key: "ancestor_rites",
        object: Some(WorshipObject::AncestorVeneration),
        exclusivity: Exclusivity::Additive,
        membership: None,
        locus: &[Locus::Household],
        institution: None,
        kind: Some(ReligionType::Folk),
        basis: "Research draft: ancestor veneration, additive, household/lineage, folk (strong). \
                Approved, Ruling CJ.",
    },
    ReligionProfile {
        key: "flame_creed",
        object: Some(WorshipObject::Dualism),
        exclusivity: Exclusivity::ExclusiveLeaning,
        membership: None,
        locus: &[Locus::Civic],
        institution: None,
        kind: Some(ReligionType::Organized),
        basis: "Research draft: dualism (fire temples) or confessional monotheism, exclusive-leaning, \
                civic, organized -- the draft's softest call; FH_R2_PLAN.md Q1's default, dualist and \
                exclusive-leaning (distinct from one_god), approved by Ruling CJ.",
    },
    ReligionProfile {
        key: "old_gods",
        object: Some(WorshipObject::Polytheism),
        exclusivity: Exclusivity::Additive,
        membership: None,
        locus: &[Locus::Place],
        institution: None,
        kind: Some(ReligionType::Folk),
        basis: "Research draft: pre-institutional polytheism, additive, place, folk (medium). \
                Approved, Ruling CJ.",
    },
    ReligionProfile {
        key: "one_god",
        object: Some(WorshipObject::Monotheism),
        exclusivity: Exclusivity::Exclusive,
        membership: Some(Membership::Universal),
        locus: &[Locus::Portable, Locus::Civic],
        institution: Some(Institution::HierarchicalClergy),
        kind: Some(ReligionType::Organized),
        basis: "Research '7 new entries': universal monotheism; 'exclusive · universal · portable+civic · \
                hierarchical clergy'. TYPE organized derived (names a clergy).",
    },
    ReligionProfile {
        key: "revealed_law",
        object: Some(WorshipObject::Monotheism),
        exclusivity: Exclusivity::Exclusive,
        membership: Some(Membership::Universal),
        locus: &[Locus::Portable],
        institution: Some(Institution::Congregational),
        kind: Some(ReligionType::Organized),
        basis: "Research '7 new entries': universal monotheism, legal-scriptural; 'exclusive · universal · \
                portable · congregational-scholarly'. TYPE organized derived (names a scholarly institution).",
    },
    ReligionProfile {
        key: "covenant_folk",
        object: Some(WorshipObject::Monotheism),
        exclusivity: Exclusivity::Exclusive,
        membership: Some(Membership::Birth),
        locus: &[Locus::Portable],
        institution: None,
        kind: None,
        basis: "Research '7 new entries': ethnic monotheism; 'exclusive · birth, non-proselytising · \
                portable/diaspora'. The row names no institution, so no institution and no TYPE.",
    },
    ReligionProfile {
        key: "the_path",
        object: Some(WorshipObject::NonTheistic),
        exclusivity: Exclusivity::Mixed,
        membership: Some(Membership::Universal),
        locus: &[],
        institution: Some(Institution::Monastic),
        kind: Some(ReligionType::Organized),
        basis: "Research '7 new entries': non-theistic soteriology; 'exclusive for monks, additive for laity · \
                universal · monastery'. No locus stated. TYPE organized derived (names a monastery).",
    },
    ReligionProfile {
        key: "mysteries",
        object: None,
        exclusivity: Exclusivity::Additive,
        membership: Some(Membership::Initiatory),
        locus: &[],
        institution: None,
        kind: Some(ReligionType::Cult),
        basis: "Research '7 new entries': initiatory salvation cult; 'additive · initiatory · small \
                cult-group'. No object of worship or locus stated; 'small cult-group' is none of axis 5's \
                forms, so no institution. TYPE cult derived (its own Type text says cult).",
    },
    ReligionProfile {
        key: "spirit_ways",
        object: Some(WorshipObject::Animism),
        exclusivity: Exclusivity::Additive,
        membership: Some(Membership::Birth),
        locus: &[Locus::Place],
        institution: Some(Institution::Specialist),
        kind: Some(ReligionType::Folk),
        basis: "Research '7 new entries': animism + shamanism; 'additive · birth-place · specialist \
                (shaman), no temples' (birth-place read as birth membership, place locus). TYPE folk \
                derived (a specialist, no institution).",
    },
    ReligionProfile {
        key: "god_king",
        object: Some(WorshipObject::RulerCult),
        exclusivity: Exclusivity::Additive,
        membership: Some(Membership::Civic),
        locus: &[Locus::Civic],
        institution: Some(Institution::TemplePriesthood),
        kind: Some(ReligionType::Organized),
        basis: "Research '7 new entries': ruler cult / sacral kingship; 'additive (a loyalty rite) · civic · \
                state priesthood'. Its one word 'civic' is read on both axes that carry it (membership and \
                locus); 'state priesthood' as temple priesthood. TYPE organized derived (names a priesthood).",
    },
];

/// The profile for a built-in religion key, or `None` for `none`, a custom
/// religion's `custom:<id>` choice (resolve it to its base first), or any key
/// the vocabulary does not hold.
pub fn religion_profile(key: &str) -> Option<&'static ReligionProfile> {
    RELIGION_PROFILES.iter().find(|p| p.key == key)
}

/// Tests for the profile table: coverage against the vocabulary, the approved
/// literals, and the "no value is never a default" rule.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::roster::CIV_RELIGIONS;

    /// Protects: the table and the vocabulary cannot drift. Every built-in key
    /// but `none` has exactly one row, `none` has none, and no row names a key
    /// the vocabulary lacks.
    #[test]
    fn every_religion_but_none_has_exactly_one_profile() {
        for (k, _) in CIV_RELIGIONS.iter() {
            let n = RELIGION_PROFILES.iter().filter(|p| p.key == *k).count();
            assert_eq!(n, if *k == "none" { 0 } else { 1 }, "{k}");
        }
        assert_eq!(RELIGION_PROFILES.len(), 14, "fifteen religions minus `none`");
        assert!(religion_profile("none").is_none());
        assert!(religion_profile("custom:3").is_none());
        for p in RELIGION_PROFILES.iter() {
            assert!(!p.basis.trim().is_empty(), "{} has no basis", p.key);
        }
    }

    /// Protects: the classification Ruling CJ approved, pinned as literals
    /// (not read back from the table): `flame_creed` is dualist and
    /// exclusive-leaning, the reference's other six are additive, and the
    /// draft's TYPE column.
    #[test]
    fn the_approved_classification_of_the_reference_seven_is_pinned() {
        let flame = religion_profile("flame_creed").unwrap();
        assert_eq!(flame.object, Some(WorshipObject::Dualism));
        assert_eq!(flame.exclusivity, Exclusivity::ExclusiveLeaning);
        let want = [
            ("sun_cult", "organized"),
            ("earth_mother", "folk"),
            ("sea_lords", "folk"),
            ("sky_pantheon", "organized"),
            ("ancestor_rites", "folk"),
            ("flame_creed", "organized"),
            ("old_gods", "folk"),
        ];
        for (k, t) in want {
            let p = religion_profile(k).unwrap();
            assert_eq!(p.kind.map(ReligionType::key), Some(t), "{k}");
            if k != "flame_creed" {
                assert_eq!(p.exclusivity, Exclusivity::Additive, "{k}");
            }
        }
    }

    /// Protects: "never encode no value as a plausible value". The approved
    /// draft carries no membership mode or institution for the reference's
    /// seven, so both must stay `None` -- a later pass that fills one in must
    /// change this test and say where the value came from.
    #[test]
    fn the_reference_seven_carry_no_membership_or_institution() {
        for (k, _) in CIV_RELIGIONS[1..crate::roster::CIV_RELIGIONS_REFERENCE_LEN].iter() {
            let p = religion_profile(k).unwrap();
            assert_eq!(p.membership, None, "{k}");
            assert_eq!(p.institution, None, "{k}");
        }
    }

    /// Protects: the seven R2-1 rows read the research document's Structure
    /// column, pinned as literals for the two axes the belief model reads.
    #[test]
    fn the_appended_seven_carry_their_structure_column() {
        let want = [
            ("one_god", Exclusivity::Exclusive, Some(Membership::Universal)),
            ("revealed_law", Exclusivity::Exclusive, Some(Membership::Universal)),
            ("covenant_folk", Exclusivity::Exclusive, Some(Membership::Birth)),
            ("the_path", Exclusivity::Mixed, Some(Membership::Universal)),
            ("mysteries", Exclusivity::Additive, Some(Membership::Initiatory)),
            ("spirit_ways", Exclusivity::Additive, Some(Membership::Birth)),
            ("god_king", Exclusivity::Additive, Some(Membership::Civic)),
        ];
        for (k, e, m) in want {
            let p = religion_profile(k).unwrap();
            assert_eq!((p.exclusivity, p.membership), (e, m), "{k}");
        }
        assert_eq!(religion_profile("covenant_folk").unwrap().kind, None);
        assert_eq!(religion_profile("mysteries").unwrap().kind, Some(ReligionType::Cult));
    }

    /// Protects: the two derived predicates on exclusivity, including `Mixed`
    /// sitting on neither side.
    #[test]
    fn exclusivity_predicates_place_mixed_between() {
        assert!(Exclusivity::Exclusive.is_exclusive() && Exclusivity::ExclusiveLeaning.is_exclusive());
        assert!(!Exclusivity::Mixed.is_exclusive() && !Exclusivity::Mixed.is_additive());
        assert!(Exclusivity::Additive.is_additive() && !Exclusivity::Additive.is_exclusive());
    }
}
