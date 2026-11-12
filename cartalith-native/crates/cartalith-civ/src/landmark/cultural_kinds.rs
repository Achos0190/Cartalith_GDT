//! **M9's cultural half** — Sacred mountain, Sacred grove, Shrine and Ceremonial site
//! (`LANDMARK_GENERATION_SCOPE.md` M9; research §19 and §26).
//!
//! Research §26's rule is the whole design: *"A physical landmark should be separated from its
//! cultural interpretation … This avoids hardcoding cultural meaning into geography. The physical
//! world remains consistent while interpretation varies between cultures."* So none of these
//! kinds has a terrain detector of its own. Each **reinterprets a physical kind's candidates**
//! (a peak, an intact forest, a spring, a rock formation) and keeps one only inside the territory
//! of a faction whose faith reads that feature as sacred. The same peak in a land whose faith does
//! not hold high places sacred is simply a Peak.
//!
//! | Kind | Reinterprets | The faction's faith must |
//! |---|---|---|
//! | Sacred mountain | Peak | have the hills as its terrain domain ([`crate::belief::religion_domain`]): Sky Pantheon |
//! | Sacred grove | Ancient forest | have the forest as its domain (Old Gods) or be Spirit Ways, whose own row says "a sacred grove" |
//! | Shrine | Spring | build the rite `shrine` ([`crate::religion_profile::religion_rite`]): the place-bound folk polytheisms |
//! | Ceremonial site | Rock formation | be bound to a place or to a lineage (`Locus::Place` or `Locus::Household`) |
//!
//! Every one of those tables is **owner-approved content** (Ruling CJ's profiles, Ruling CY's rite
//! table, the belief module's domains), read, not re-chosen here. Where a faction has no religion,
//! or a religion with no row, it reads nothing sacred: `none` is silence, never a plausible default.
//!
//! ## §31 categories
//!
//! Category C: the pairing of a faith's profile with a physical feature is an engineering
//! synthesis, not an established formula, and research §19's `F_cultural` term is **binary**
//! here (it gates; it does not weigh), because the engine holds a faith per faction and no measure
//! of how devout. Each pairing is labelled where it is made. The base pool's own Category A/B terms
//! (research §19's prominence, visibility, isolation) rank the survivors unchanged.
//!
//! ## What they may do that no other kind does
//!
//! **Sit on the very cell of the physical landmark they reinterpret.** [`Pool::overlay`] exempts
//! them from the shared spacing field (they keep their own kind's spacing), because §26 says one
//! feature carries two readings. They never block anything: an overlay is not entered into the
//! shared field.

use super::*;
use crate::religion_profile::{religion_profile, religion_rite, Locus};

/// One faction's faith, as much of it as this pass reads. **Index = faction id** in
/// [`LandmarkInputs::faiths`] (the roster's own numbering, `0` the unclaimed entry).
#[derive(Clone, Debug, PartialEq)]
pub struct FactionFaith {
    /// The faction's name, for the causal chain.
    pub faction_name: String,
    /// The **engine** religion key (`ReligionLibrary::engine_key`: a custom religion already
    /// resolved to its base), `"none"` for no religion. A key with no profile reads nothing.
    pub religion: String,
    /// The religion's display name (a custom religion's own), for the causal chain.
    pub religion_label: String,
}

/// The physical feature a cultural kind reinterprets, and the reading a faith must hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reading {
    /// Peak → Sacred mountain.
    HighPlaces,
    /// Ancient forest → Sacred grove.
    Grove,
    /// Spring → Shrine.
    Shrine,
    /// Rock formation → Ceremonial site.
    Ceremonial,
}

impl Reading {
    /// Why `faith` reads this feature as sacred, or `None` if it does not. A pure function of the
    /// owner-approved tables, so it is unit-tested against each row.
    pub(super) fn reads(self, faith: &FactionFaith) -> Option<String> {
        let key = faith.religion.as_str();
        if key.is_empty() || key == "none" {
            return None;
        }
        let who = &faith.religion_label;
        match self {
            Reading::HighPlaces => (crate::belief::religion_domain(key) == Some("hills"))
                .then(|| format!("{who} holds high places sacred (its terrain is the hills)")),
            Reading::Grove => (crate::belief::religion_domain(key) == Some("forest") || key == "spirit_ways")
                .then(|| format!("{who} holds groves sacred")),
            Reading::Shrine => (religion_rite(key) == Some("shrine"))
                .then(|| format!("{who}, a place-bound folk faith, raises small shrines")),
            Reading::Ceremonial => religion_profile(key)
                .filter(|p| p.locus.iter().any(|l| matches!(l, Locus::Place | Locus::Household)))
                .map(|_| format!("{who} is bound to place or lineage")),
        }
    }
}

/// Keeps the candidates of `base` whose territory's faith reads the feature as sacred, as an
/// **overlay** pool (see the module doc). Returns `None` (`NoTerrain`) when the territory raster,
/// the faith list or the base pool is absent: with no map of who owns the ground there is no
/// interpretation to make, and the physical kind's own candidates must not be relabelled sacred
/// for want of data.
///
/// A candidate on unowned ground (territory `0`), or in a faction whose faith does not read it,
/// is a constraint reject: it is a perfectly good physical feature that this world's peoples do
/// not hold sacred. The chain gains the reading as its first fact (cause first).
pub(super) fn pool_interpreted(c: &Ctx<'_>, base: Option<Pool>, reading: Reading) -> Option<Pool> {
    let territory = c.territory?;
    if c.inp.faiths.is_empty() {
        return None;
    }
    let base = base?;
    let mut out = Pool::new();
    out.overlay = true;
    out.rejected_constraint = base.rejected_constraint;
    let mut keep: Vec<usize> = Vec::new();
    for (k, cand) in base.cands.into_iter().enumerate() {
        let owner = territory[cand.i];
        let faith = (owner > 0).then(|| c.inp.faiths.get(owner as usize)).flatten();
        let Some((faith, why)) = faith.and_then(|f| reading.reads(f).map(|w| (f, w))) else {
            out.rejected_constraint += 1;
            continue;
        };
        let mut facts = vec![format!("in the land of {}: {}", faith.faction_name, why)];
        facts.extend(cand.facts);
        out.cands.push(Cand { i: cand.i, x: cand.x, y: cand.y, facts });
        keep.push(k);
    }
    let n_base = base.terms.first().map(|t| t.2.len());
    out.terms = base
        .terms
        .into_iter()
        // A term whose length is not the base pool's candidate count cannot be filtered by
        // index; dropping it (the sum renormalises) is honest, indexing past its end is not.
        .filter(|(_, _, v)| Some(v.len()) == n_base)
        .map(|(l, w, v)| (l, w, keep.iter().map(|&k| v[k]).collect()))
        .collect();
    Some(out)
}
