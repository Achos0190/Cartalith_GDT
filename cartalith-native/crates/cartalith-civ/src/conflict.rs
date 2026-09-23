//! `STORY_PLANNING_SCOPE.md` SP-4 -- the conflict overlay's entity.
//!
//! A drawn conflict: a front, an arrow, a siege or a battle, with a name, a
//! year range, the factions involved and an authored outcome. No reference
//! ancestor (`DECISIONS.md` §7d, divergence by addition), and **no combat
//! resolution** (§5): nothing here decides who wins. [`side_manpower`] only
//! *reads* `manpower.rs`'s answer for each side.
//!
//! ## The optional anchor (Ruling AO)
//!
//! A conflict may reference a settlement (by `tid`) or a province (by its
//! seed settlement's `tid` -- see [`ConflictAnchor`] for why not its id).
//! The geometry is stored as it was drawn, together with the anchor's
//! position at the moment it was attached ([`Conflict::anchor_at`]); drawing
//! it is [`Conflict::resolved_points`], which translates the whole shape by
//! however far the anchor has since moved. So moving the place moves the
//! conflict, and nothing is rewritten on a move.
//!
//! **An anchor that no longer resolves** (the settlement was deleted, the
//! province vanished in a recompute) is kept, not cleared: the shape draws
//! where it was attached and the caller reports the anchor as missing. If
//! the id resolves again (an undo restoring the settlement), it re-attaches
//! with nothing to repair. Deleting the reference silently would lose the
//! author's link; re-binding it to something else would be worse.
//!
//! Year handling is a plain `i64` year, the Timeline cursor's own unit.
//! SP-2's day-level calendar (Ruling AO, "Grain") is not needed to answer
//! "is this conflict active in the cursor's year", which is SP-4's only
//! question of time.

use crate::manpower::Manpower;

/// Which of §4's four drawn things this is. Fronts and arrows are
/// polylines; sieges and battles are single-point markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    Front,
    Arrow,
    Siege,
    Battle,
}

impl ConflictKind {
    pub const ALL: [ConflictKind; 4] = [Self::Front, Self::Arrow, Self::Siege, Self::Battle];

    pub fn key(self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Arrow => "arrow",
            Self::Siege => "siege",
            Self::Battle => "battle",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.key() == k)
    }

    /// Points the shape needs before it can be committed: a line needs two
    /// ends, a marker one position.
    pub fn min_points(self) -> usize {
        match self {
            Self::Front | Self::Arrow => 2,
            Self::Siege | Self::Battle => 1,
        }
    }
}

/// What a conflict is attached to, **keyed on a settlement `tid` in both
/// cases**.
///
/// `Province` carries the `tid` of the province's seed settlement
/// (`Province::capital_settlement_index`'s settlement), not `Province::id`:
/// that id is a positional counter `civ_generate_provinces` re-issues on
/// every run (faction order, then settlement order), so a recompute that
/// adds or drops one settlement renumbers every province after it. A `tid`
/// survives a recompute; a province whose seed is no longer a seed simply
/// stops resolving, which is reported rather than re-bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictAnchor {
    Settlement(u64),
    Province(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub id: u64,
    pub name: String,
    pub kind: ConflictKind,
    pub start_year: i64,
    /// `None` = still going; open-ended to the right.
    pub end_year: Option<i64>,
    /// Faction indices (`1..`, the roster's own numbering; `0` is Unclaimed
    /// and is refused by [`ConflictStore::add`]/[`ConflictStore::replace`]).
    pub sides: Vec<i32>,
    /// Free text, authored. Empty = not stated. Deliberately not an enum: a
    /// closed vocabulary of outcomes is the start of a resolution model,
    /// which §5 rules out.
    pub outcome: String,
    /// Grid-cell coordinates as drawn.
    pub points: Vec<(f64, f64)>,
    pub anchor: Option<ConflictAnchor>,
    /// The anchor's position when it was attached. Meaningless without
    /// `anchor`.
    pub anchor_at: (f64, f64),
}

impl Conflict {
    /// Active in `year` if `start_year <= year <= end_year` (inclusive both
    /// ends; an open end is active from its start onwards).
    pub fn active_in(&self, year: i64) -> bool {
        year >= self.start_year && self.end_year.is_none_or(|e| year <= e)
    }

    /// The shape to draw, given where the anchor is now (`None` when there is
    /// no anchor or it no longer resolves -- the shape then stays where it
    /// was drawn).
    pub fn resolved_points(&self, anchor_now: Option<(f64, f64)>) -> Vec<(f64, f64)> {
        match (self.anchor, anchor_now) {
            (Some(_), Some((ax, ay))) => {
                let (dx, dy) = (ax - self.anchor_at.0, ay - self.anchor_at.1);
                self.points.iter().map(|&(x, y)| (x + dx, y + dy)).collect()
            }
            _ => self.points.clone(),
        }
    }

    /// Attach to `anchor`, currently at `at`, keeping the shape where it is
    /// drawn now (`drawn_now` is [`Self::resolved_points`] under the old
    /// anchor). `None` detaches and bakes the drawn position in.
    pub fn set_anchor(
        &mut self,
        anchor: Option<ConflictAnchor>,
        at: (f64, f64),
        drawn_now: Vec<(f64, f64)>,
    ) {
        self.points = drawn_now;
        self.anchor = anchor;
        self.anchor_at = if anchor.is_some() { at } else { (0.0, 0.0) };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictError {
    TooFewPoints { kind: ConflictKind, need: usize, got: usize },
    EndBeforeStart,
    BadSide(i32),
    NonFinitePoint,
    NoSuchConflict(u64),
}

/// Every conflict of one world. `next_id` never goes back, so an id is
/// never reissued after a delete.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConflictStore {
    pub next_id: u64,
    pub conflicts: Vec<Conflict>,
}

impl ConflictStore {
    pub fn new() -> Self {
        Self { next_id: 1, conflicts: Vec::new() }
    }

    fn validate(c: &Conflict) -> Result<(), ConflictError> {
        let need = c.kind.min_points();
        if c.points.len() < need {
            return Err(ConflictError::TooFewPoints { kind: c.kind, need, got: c.points.len() });
        }
        if c.points.iter().any(|p| !p.0.is_finite() || !p.1.is_finite()) {
            return Err(ConflictError::NonFinitePoint);
        }
        if c.end_year.is_some_and(|e| e < c.start_year) {
            return Err(ConflictError::EndBeforeStart);
        }
        if let Some(&s) = c.sides.iter().find(|&&s| s <= 0) {
            return Err(ConflictError::BadSide(s));
        }
        Ok(())
    }

    /// A marker kind keeps only its first point; a faction named twice is
    /// one side (first mention wins, so side order is the author's).
    fn normalise(c: &mut Conflict) {
        if c.kind.min_points() == 1 {
            c.points.truncate(1);
        }
        let mut seen = Vec::with_capacity(c.sides.len());
        c.sides.retain(|s| {
            let fresh = !seen.contains(s);
            seen.push(*s);
            fresh
        });
    }

    /// Adds `c` under a fresh id (whatever `c.id` held is ignored) and
    /// returns it.
    pub fn add(&mut self, mut c: Conflict) -> Result<u64, ConflictError> {
        Self::validate(&c)?;
        Self::normalise(&mut c);
        c.id = self.next_id.max(1);
        self.next_id = c.id + 1;
        self.conflicts.push(c);
        Ok(self.next_id - 1)
    }

    /// Replaces the conflict with `c.id`, validated the same way.
    pub fn replace(&mut self, mut c: Conflict) -> Result<(), ConflictError> {
        Self::validate(&c)?;
        Self::normalise(&mut c);
        let slot = self.get_mut(c.id).ok_or(ConflictError::NoSuchConflict(c.id))?;
        *slot = c;
        Ok(())
    }

    pub fn get(&self, id: u64) -> Option<&Conflict> {
        self.conflicts.iter().find(|c| c.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Conflict> {
        self.conflicts.iter_mut().find(|c| c.id == id)
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let before = self.conflicts.len();
        self.conflicts.retain(|c| c.id != id);
        self.conflicts.len() != before
    }

    /// "What conflicts happened here": ids of every conflict attached to
    /// `anchor`, in store order.
    pub fn attached_to(&self, anchor: ConflictAnchor) -> Vec<u64> {
        self.conflicts.iter().filter(|c| c.anchor == Some(anchor)).map(|c| c.id).collect()
    }

    /// Converts every settlement/province anchor to free geometry at its
    /// current drawn position. For the passes that reissue `tid`s and
    /// province ids from scratch over the same grid (`civ_populate`), where
    /// keeping the id would silently re-bind it to an unrelated place.
    /// `resolve` is the pre-pass anchor position lookup.
    pub fn detach_all(&mut self, resolve: impl Fn(ConflictAnchor) -> Option<(f64, f64)>) {
        for c in &mut self.conflicts {
            if let Some(a) = c.anchor {
                let drawn = c.resolved_points(resolve(a));
                c.set_anchor(None, (0.0, 0.0), drawn);
            }
        }
    }
}

/// One side of a conflict, read. `manpower` is `None` when the world has no
/// row for that faction (no civ layer, or a faction index the roster no
/// longer has) -- absent, never a zero army.
#[derive(Debug, Clone, PartialEq)]
pub struct SideReading<'a> {
    pub faction: i32,
    pub manpower: Option<&'a Manpower>,
}

/// Each side's row out of `by_faction`, which is indexed `faction - 1` --
/// the order `civ_military_manpower_world` returns for factions `1..n`.
pub fn side_manpower<'a>(sides: &[i32], by_faction: &'a [Manpower]) -> Vec<SideReading<'a>> {
    sides
        .iter()
        .map(|&f| SideReading {
            faction: f,
            manpower: usize::try_from(f).ok().and_then(|i| i.checked_sub(1)).and_then(|i| by_faction.get(i)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manpower::{civ_military_manpower, civ_military_manpower_world, ManpowerInput};

    fn front(points: Vec<(f64, f64)>) -> Conflict {
        Conflict {
            id: 0,
            name: "War of the Two Rivers".into(),
            kind: ConflictKind::Front,
            start_year: 100,
            end_year: Some(104),
            sides: vec![1, 2],
            outcome: String::new(),
            points,
            anchor: None,
            anchor_at: (0.0, 0.0),
        }
    }

    #[test]
    fn construction_validates_and_issues_fresh_ids() {
        let mut s = ConflictStore::new();
        assert_eq!(s.add(front(vec![(1.0, 1.0), (5.0, 2.0)])), Ok(1));
        assert_eq!(s.add(front(vec![(1.0, 1.0), (5.0, 2.0)])), Ok(2));
        assert_eq!(
            s.add(front(vec![(1.0, 1.0)])),
            Err(ConflictError::TooFewPoints { kind: ConflictKind::Front, need: 2, got: 1 })
        );
        let mut bad = front(vec![(0.0, 0.0), (1.0, 1.0)]);
        bad.end_year = Some(99);
        assert_eq!(s.add(bad), Err(ConflictError::EndBeforeStart));
        // A one-year battle ends the year it starts: legal.
        let mut one_year = front(vec![(0.0, 0.0), (1.0, 1.0)]);
        one_year.end_year = Some(100);
        assert!(s.add(one_year).is_ok());
        let mut unclaimed = front(vec![(0.0, 0.0), (1.0, 1.0)]);
        unclaimed.sides = vec![1, 0];
        assert_eq!(s.add(unclaimed), Err(ConflictError::BadSide(0)));
        assert_eq!(
            s.add(front(vec![(f64::NAN, 0.0), (1.0, 1.0)])),
            Err(ConflictError::NonFinitePoint)
        );
        // A delete never lets an id be reissued.
        assert!(s.remove(2));
        assert!(!s.remove(2));
        assert_eq!(s.add(front(vec![(1.0, 1.0), (5.0, 2.0)])), Ok(4));
    }

    #[test]
    fn a_marker_keeps_one_point_and_a_repeated_side_is_one_side() {
        let mut s = ConflictStore::new();
        let mut siege = front(vec![(3.0, 4.0), (9.0, 9.0)]);
        siege.kind = ConflictKind::Siege;
        siege.sides = vec![2, 1, 2];
        let id = s.add(siege).unwrap();
        assert_eq!(s.get(id).unwrap().points, vec![(3.0, 4.0)]);
        assert_eq!(s.get(id).unwrap().sides, vec![2, 1]);
    }

    #[test]
    fn active_range_is_inclusive_and_open_ended() {
        let c = front(vec![(0.0, 0.0), (1.0, 1.0)]);
        assert!(!c.active_in(99));
        assert!(c.active_in(100));
        assert!(c.active_in(104));
        assert!(!c.active_in(105));
        let open = Conflict { end_year: None, ..c };
        assert!(open.active_in(10_000));
        assert!(!open.active_in(99));
    }

    #[test]
    fn moving_the_anchor_moves_the_shape_and_a_missing_one_leaves_it() {
        let mut c = front(vec![(10.0, 10.0), (14.0, 12.0)]);
        c.set_anchor(Some(ConflictAnchor::Settlement(7)), (11.0, 11.0), c.points.clone());
        // Settlement 7 moved from (11,11) to (21,8).
        assert_eq!(c.resolved_points(Some((21.0, 8.0))), vec![(20.0, 7.0), (24.0, 9.0)]);
        // Settlement 7 deleted: drawn where it was attached, anchor kept.
        assert_eq!(c.resolved_points(None), vec![(10.0, 10.0), (14.0, 12.0)]);
        assert_eq!(c.anchor, Some(ConflictAnchor::Settlement(7)));
        // Detaching bakes the current drawn position in.
        let drawn = c.resolved_points(Some((21.0, 8.0)));
        c.set_anchor(None, (0.0, 0.0), drawn);
        assert_eq!(c.resolved_points(Some((99.0, 99.0))), vec![(20.0, 7.0), (24.0, 9.0)]);
    }

    #[test]
    fn attached_to_answers_what_happened_here_by_id_not_name() {
        let mut s = ConflictStore::new();
        let mut a = front(vec![(0.0, 0.0), (1.0, 1.0)]);
        a.anchor = Some(ConflictAnchor::Settlement(3));
        let mut b = a.clone();
        b.anchor = Some(ConflictAnchor::Province(3));
        let mut c = a.clone();
        c.anchor = Some(ConflictAnchor::Settlement(4));
        let ia = s.add(a).unwrap();
        let ib = s.add(b).unwrap();
        s.add(c).unwrap();
        s.add(front(vec![(0.0, 0.0), (1.0, 1.0)])).unwrap();
        assert_eq!(s.attached_to(ConflictAnchor::Settlement(3)), vec![ia]);
        // Settlement 3 and province 3 are different places.
        assert_eq!(s.attached_to(ConflictAnchor::Province(3)), vec![ib]);
    }

    #[test]
    fn detach_all_keeps_every_shape_where_it_is_drawn() {
        let mut s = ConflictStore::new();
        let mut a = front(vec![(0.0, 0.0), (2.0, 0.0)]);
        a.set_anchor(Some(ConflictAnchor::Settlement(5)), (1.0, 1.0), a.points.clone());
        let id = s.add(a).unwrap();
        s.detach_all(|_| Some((4.0, 1.0)));
        let c = s.get(id).unwrap();
        assert_eq!(c.anchor, None);
        assert_eq!(c.points, vec![(3.0, 0.0), (5.0, 0.0)]);
    }

    /// The manpower read picks each side's own row out of the world pass, and
    /// that row is the model's answer for that faction's inputs -- checked
    /// against an independent single-faction call, not against the world
    /// vector it was indexed from.
    #[test]
    fn sides_read_their_own_faction_from_the_real_manpower_model() {
        let base = ManpowerInput {
            nucleated_pop: 250_000.0,
            farmers_per_urbanite: 3.0,
            land_capacity: 1_000_000.0,
            government: "monarchy",
            capital_road_reach: 0.4,
            road_density: 0.2,
            navigable_share: 0.1,
            sea_share: 0.0,
        };
        let strong = ManpowerInput {
            nucleated_pop: 600_000.0,
            capital_road_reach: 0.9,
            road_density: 0.8,
            land_capacity: 2_400_000.0,
            ..base
        };
        let inputs = [base, strong];
        let rows = civ_military_manpower_world(&inputs);
        let read = side_manpower(&[2, 1, 9, 0, -3], &rows);
        assert_eq!(read.len(), 5);
        assert_eq!(read[0].faction, 2);
        // Both factions' land-per-head equals the world's here (1M / 1M and
        // 2.4M / 2.4M people), so the world pass's normalisation is exactly
        // `land / people`, and each row must equal a direct single-faction
        // call with that land.
        let direct_strong = civ_military_manpower(&strong);
        let got = read[0].manpower.unwrap();
        assert!((got.standing_army - direct_strong.standing_army).abs() < 1e-6);
        assert!((got.field_army - direct_strong.field_army).abs() < 1e-6);
        assert!(got.standing_army > read[1].manpower.unwrap().standing_army);
        // No row for faction 9, 0 or a negative index: absent, not zero.
        assert!(read[2].manpower.is_none());
        assert!(read[3].manpower.is_none());
        assert!(read[4].manpower.is_none());
        // And the two sides really are different rows.
        assert_ne!(read[0].manpower, read[1].manpower);
    }

    #[test]
    fn kind_keys_round_trip() {
        for k in ConflictKind::ALL {
            assert_eq!(ConflictKind::from_key(k.key()), Some(k));
        }
        assert_eq!(ConflictKind::from_key("skirmish"), None);
    }
}
