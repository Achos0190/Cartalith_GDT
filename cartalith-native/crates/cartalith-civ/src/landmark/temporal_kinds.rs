//! **M9's temporal half** (`LANDMARK_GENERATION_SCOPE.md` M9; research §20 and §25): the four
//! Historical kinds that were refused for want of a present-year read or a persisted record, and
//! the per-landmark *phase* a landmark is in at the Timeline's year.
//!
//! | Kind | Reads | Chain (research §20) |
//! |---|---|---|
//! | Historic battlefield | a drawn battle that ended before the present year | regional conflict → battle → remembered |
//! | Ruin | a settlement whose latest collapse record flags it in ruins, not walled | settlement → decline → ruination |
//! | Destroyed fortress | the same, walled (`fortified`) | settlement → fortification → conflict/decline → ruination |
//! | Historic crossing | a river crossing a former way used and no live way uses | road → river → crossing no longer travelled |
//!
//! All four read the **recorded** past ([`LandmarkHistory`], [`BattleMark`]); none reads outcome text
//! (`Conflict::outcome` stays free text by `STORY_PLANNING_SCOPE.md` §5) and none invents a
//! destruction. Ruin and Destroyed fortress need the collapse flags `project_bridge` persists since
//! 2026-11-12; a world with no collapse record has no ruins to read, and says `NoTerrain`.
//!
//! ## Why a kind is chosen by the year, and what that costs
//!
//! Battlefield and Historic battlefield are one record in two eras: a battle still going, or yet to
//! come, is a Battlefield, and one that ended before [`LandmarkHistory::now_year`] is a Historic
//! battlefield. The run is made at one year, so the choice is as of that year; **a landmark's key
//! is `kind@x,y`, so a battle that crosses the line between two runs changes key**. The
//! bridge carries a user-given name across the pair (`landmark_set_user_name`'s alias rule), so a
//! name the person typed survives; the generated name is derived and simply follows the new kind.
//!
//! ## §31 categories
//!
//! Category B: the chains are research §20's, and the ruin flags are the collapse model's own.
//! Category C: every weight below, and Historic crossing's *definition*, which research §29 leaves
//! open and `LANDMARK_GENERATION_SCOPE.md` called a guess (see [`pool_historic_crossing`] for what
//! was chosen and why it is labelled rather than cited).

use super::*;

/// Which era of a drawn battle a pool takes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum BattleEra {
    /// Not yet ended at the present year (or no present year is known): Battlefield.
    Current,
    /// Ended before the present year: Historic battlefield.
    Past,
}

/// The era of `b` at `now`. With **no** present year (a caller with no Timeline) every battle is
/// [`BattleEra::Current`], which is exactly how Battlefield behaved before the split, so a caller
/// predating the year changes nothing. A battle with no end year is still going.
pub(super) fn era_of(b: &BattleMark, now: Option<i64>) -> BattleEra {
    match (now, b.end_year) {
        (Some(n), Some(e)) if e < n => BattleEra::Past,
        _ => BattleEra::Current,
    }
}

/// Walks a way's control polyline at [`WAY_STEP_CELLS`] (the sampling [`WayGrid::build`] uses),
/// returning the cell of each sample with its distance along the course in cells, and the total
/// length in cells. The short way round the seam of a wrapping world is the real segment. One body
/// for Ancient road and Historic crossing, so the two cannot disagree about where a former way ran.
pub(super) fn course_samples(c: &Ctx<'_>, w: &crate::Way) -> (Vec<(usize, f64)>, f64) {
    let (gw, gh) = (c.inp.gw, c.inp.gh);
    let mut samples: Vec<(usize, f64)> = Vec::new();
    let mut along = 0f64;
    for pair in w.pts.windows(2) {
        let (ax, ay) = pair[0];
        let (bx, by) = pair[1];
        let mut dx = bx - ax;
        let dy = by - ay;
        if c.inp.world {
            let gwf = gw as f64;
            if dx > gwf * 0.5 {
                dx -= gwf;
            } else if dx < -gwf * 0.5 {
                dx += gwf;
            }
        }
        let len = (dx * dx + dy * dy).sqrt();
        if !len.is_finite() {
            continue;
        }
        let steps = ((len / WAY_STEP_CELLS).ceil() as i64).clamp(1, WAY_MAX_STEPS);
        for k in 0..=steps {
            let t = k as f64 / steps as f64;
            let (x, y) = (ax + dx * t, ay + dy * t);
            if !x.is_finite() || !(y >= 0.0) || y.floor() >= gh as f64 {
                continue;
            }
            let mut xx = x.floor() as i64;
            if c.inp.world {
                xx = xx.rem_euclid(gw as i64);
            } else if xx < 0 || xx >= gw as i64 {
                continue;
            }
            samples.push((y.floor() as usize * gw + xx as usize, along + len * t));
        }
        along += len;
    }
    (samples, along)
}


/// **Category C.** Ruin's terms: the size the place had. Destroyed fortress adds how commanding
/// its ground is.
const RUIN_TERMS: [(&str, f64); 1] = [("former size", 1.0)];

/// How far from a ruined fortress a recorded battle still counts as part of its story, in km.
/// **Category C**, a labelled judgement: one to two days' march, the scale at which a field
/// action and a fortress are one campaign.
const FORTRESS_BATTLE_KM: f64 = 50.0;

/// §29's **Ruin** (Historical, Regional) and **Destroyed fortress** (Historical, Regional): a
/// settlement a collapse record left in ruins ([`LandmarkHistory::ruins`]), the unwalled and the
/// walled.
///
/// **Domain:** each [`RuinedSite`] whose `fortified` matches `fortress`. **Constraints**, each a
/// constraint reject: the cell is on the grid and land (a site the sea has since taken is gone). A
/// ruined place that a settlement still occupies is **kept**, and the chain says it stands in
/// ruins: ruination, not absence, is this kind's whole point, and absence is Abandoned
/// settlement's. For a fortress the chain also names the nearest recorded battle within
/// [`FORTRESS_BATTLE_KM`] whose period reached the site's last record, as the *conflict* link of
/// research §20, only where one exists; it never writes a cause the record does not hold.
pub(super) fn pool_ruin(c: &Ctx<'_>, fortress: bool) -> Option<Pool> {
    let h = c.inp.history?;
    let rows: Vec<&RuinedSite> = h.ruins.iter().filter(|r| r.fortified == fortress).collect();
    if rows.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let mut t_pop: Vec<f32> = Vec::new();
    for r in rows {
        let s = &r.site;
        if s.x >= gw || s.y >= c.inp.gh {
            p.rejected_constraint += 1;
            continue;
        }
        let i = s.y * gw + s.x;
        if !c.is_land(i) {
            p.rejected_constraint += 1;
            continue;
        }
        let name = if s.name.trim().is_empty() { "an unnamed settlement" } else { s.name.as_str() };
        let mut facts = vec![
            format!(
                "{} was recorded {} in {}",
                name,
                if fortress { "walled and in ruins" } else { "in ruins" },
                s.last_year
            ),
            format!("population {} at that record", thousands(s.population)),
            if r.still_live {
                "a settlement still stands on the site, in decline".to_string()
            } else {
                "no settlement stands on the site now".to_string()
            },
        ];
        if fortress {
            if let Some((b, km)) = nearest_battle(c, s.x, s.y, s.last_year) {
                let bn = if b.name.trim().is_empty() { "an unnamed battle" } else { b.name.as_str() };
                facts.push(format!("{} was fought {} away, in {}", bn, fmt_km(km), b.start_year));
            }
        }
        p.cands.push(Cand { i, x: s.x, y: s.y, facts });
        t_pop.push(s.population as f32);
    }
    p.terms.push((RUIN_TERMS[0].0, RUIN_TERMS[0].1, t_pop));
    Some(p)
}

/// The nearest drawn battle within [`FORTRESS_BATTLE_KM`] of a cell whose period had begun by
/// `year` (a battle that starts after the site's last record cannot have ended it), with its
/// distance in km. Ties: the earlier in the list, deterministically.
fn nearest_battle<'a>(c: &Ctx<'a>, x: usize, y: usize, year: i64) -> Option<(&'a BattleMark, f64)> {
    let mut best: Option<(&BattleMark, f64)> = None;
    for b in c.inp.battles {
        if !b.x.is_finite() || !b.y.is_finite() || b.start_year > year.saturating_add(BATTLE_AFTER_YEARS) {
            continue;
        }
        let mut dx = b.x - x as f64;
        if c.inp.world {
            let gw = c.inp.gw as f64;
            if dx > gw * 0.5 {
                dx -= gw;
            } else if dx < -gw * 0.5 {
                dx += gw;
            }
        }
        let d = (dx * dx + (b.y - y as f64).powi(2)).sqrt() * c.cell_km;
        if d <= FORTRESS_BATTLE_KM && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((b, d));
        }
    }
    best
}

/// A battle may begin this many years after a site's last record and still be its story: a
/// collapse is recorded in steps, and the fall may be recorded a step before the war that caused
/// it is dated. **Category C**, a labelled judgement.
const BATTLE_AFTER_YEARS: i64 = 25;

/// **Category C.** Historic crossing's terms: the road's class, the river's size, and a battle
/// nearby.
const HISTORIC_CROSSING_TERMS: [(&str, f64); 3] =
    [("former class", 0.40), ("river size", 0.30), ("battle nearby", 0.30)];

/// How near a recorded battle must be to a crossing to add to it, in km. **Category C.**
const CROSSING_BATTLE_KM: f64 = 10.0;

/// §29's **Historic crossing** (Historical, Local) — a river crossing that a road once used and
/// no live road uses.
///
/// Research §29 names this kind and says nothing of what makes a crossing *historic*; the scope
/// called any choice a guess. The choice made here is stated so it can be overruled: **a crossing is
/// historic when a recorded way used it and the world no longer carries that way there**. That is
/// the one reading the engine can check (the Timeline remembers the way; the live network does
/// not), it separates this kind from Ford and Bridge site (which mark crossings of the *live*
/// network), and it ranks by the three things that made a crossing matter in the record: how
/// important the road was, how big the river was, and whether a battle was fought within
/// [`CROSSING_BATTLE_KM`]. Years in use are not read: recorded years are sparse snapshots.
///
/// **Domain:** per former way ([`LandmarkHistory::ways`]), each run of consecutive course samples
/// on a channel cell is one crossing, placed on the middle sample. **Constraints:** land, and not
/// on a live way. Needs the channel mask and flow; `None` otherwise.
pub(super) fn pool_historic_crossing(c: &Ctx<'_>) -> Option<Pool> {
    let h = c.inp.history?;
    let chan = c.chan?;
    let flow = c.flow?;
    if h.ways.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_rank, mut t_flow, mut t_battle) = (vec![], vec![], vec![]);
    for fw in &h.ways {
        let w = &fw.way;
        let Some(rank) = way_rank(crate::way_class::WayClass::from(w.way_type)) else {
            continue;
        };
        let (samples, _) = course_samples(c, w);
        let mut run: Vec<usize> = Vec::new();
        let mut close = |run: &mut Vec<usize>, p: &mut Pool| {
            if run.is_empty() {
                return;
            }
            let i = run[run.len() / 2];
            run.clear();
            let on_live = c.ways.as_ref().is_some_and(|g| g.ways_here[i] > 0);
            if !c.is_land(i) || on_live {
                p.rejected_constraint += 1;
                return;
            }
            let (x, y) = (i % gw, i / gw);
            let q = flow[i] as f64;
            let battles = c
                .inp
                .battles
                .iter()
                .filter(|b| {
                    b.x.is_finite()
                        && b.y.is_finite()
                        && ((b.x - x as f64).powi(2) + (b.y - y as f64).powi(2)).sqrt() * c.cell_km <= CROSSING_BATTLE_KM
                })
                .count();
            let name = if w.name.trim().is_empty() { String::new() } else { format!(" ({})", w.name) };
            let mut facts = vec![
                format!("a {}{} crossed this river, last recorded in {}", rank_label(rank), name, fw.last_year),
                format!("the river carries {} cells of flow here", thousands(q)),
                "no road on the present network crosses here".to_string(),
            ];
            if battles > 0 {
                facts.push(format!("{} recorded battle(s) within {}", battles, fmt_km(CROSSING_BATTLE_KM)));
            }
            p.cands.push(Cand { i, x, y, facts });
            t_rank.push(rank as f32);
            t_flow.push(q.ln_1p() as f32);
            t_battle.push(battles as f32);
        };
        for (i, _) in &samples {
            if chan[*i] != 0 && c.is_land(*i) {
                // One crossing is one unbroken run of channel cells; a repeated cell (the course
                // revisiting a sample) is not a second crossing.
                if run.last() != Some(i) {
                    run.push(*i);
                }
            } else {
                close(&mut run, &mut p);
            }
        }
        close(&mut run, &mut p);
    }
    p.terms = vec![
        (HISTORIC_CROSSING_TERMS[0].0, HISTORIC_CROSSING_TERMS[0].1, t_rank),
        (HISTORIC_CROSSING_TERMS[1].0, HISTORIC_CROSSING_TERMS[1].1, t_flow),
        (HISTORIC_CROSSING_TERMS[2].0, HISTORIC_CROSSING_TERMS[2].1, t_battle),
    ];
    Some(p)
}

// ---------------------------------------------------------------------------
// Phase: a landmark's temporal state (research §25)
// ---------------------------------------------------------------------------

/// Research §25's lifecycle, restricted to the stages this engine holds evidence for. The
/// research's *Discovered*, *Named* and *Rediscovered* are not derivable from the world (naming is
/// authored, and is carried by the landmark's user-given name, Ruling EL); *Utilized*, *Developed*,
/// *Monumentalized*, *Abandoned* and *Ruined* are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandmarkPhase {
    /// A natural feature with no recorded human use within [`PHASE_USE_KM`].
    Natural,
    /// A natural feature a settlement now lives within [`PHASE_USE_KM`] of.
    Utilised,
    /// A site people built or work: a fort, harbour, mine, market and the like.
    Developed,
    /// A place set apart by what people did there: a battlefield, a shrine, a monument.
    Monumentalised,
    /// A site once lived on or travelled, now empty or unused.
    Abandoned,
    /// A settlement the record leaves in ruins.
    Ruined,
}

impl LandmarkPhase {
    /// The lower-case token the shell receives and prints.
    pub fn key(self) -> &'static str {
        match self {
            LandmarkPhase::Natural => "natural",
            LandmarkPhase::Utilised => "utilised",
            LandmarkPhase::Developed => "developed",
            LandmarkPhase::Monumentalised => "monumentalised",
            LandmarkPhase::Abandoned => "abandoned",
            LandmarkPhase::Ruined => "ruined",
        }
    }
}

/// How near a live settlement must be for a natural landmark to be *used*, in km. **Category C**:
/// a day's round trip on foot at 5 km/h is about 20 km out and back, so 10 km is the radius a
/// settlement draws water, stone or timber from without camping. A labelled judgement.
pub const PHASE_USE_KM: f64 = 10.0;

/// Kinds people built or work (Developed). **Category C**: a vocabulary decision.
const DEVELOPED_KINDS: [&str; 14] = [
    "fort",
    "watchtower",
    "fortified_pass",
    "fortified_crossing",
    "bridge_site",
    "road_junction",
    "market_site",
    "caravan_station",
    "trade_depot",
    "mine",
    "quarry",
    "resource_extraction_site",
    "salt_works",
    "harbour",
];

/// Kinds set apart by what people did or believed there (Monumentalised). **Category C.**
const MONUMENTAL_KINDS: [&str; 9] = [
    "battlefield",
    "battlefield_historic",
    "border_marker",
    "shrine",
    "temple",
    "sacred_grove",
    "sacred_mountain",
    "ceremonial_site",
    "pilgrimage_site",
];

/// What [`landmark_phase`] reads. Borrowed, and assembled by the caller from the live world, so the
/// phase is **as of the Timeline year in force** and moves with it (a settlement that does not
/// exist at year 100 does at year 300).
#[derive(Clone, Debug)]
pub struct PhaseInputs<'a> {
    pub gw: usize,
    /// Whole-grid cells across wrap in x.
    pub world: bool,
    /// Real km per cell.
    pub cell_km: f64,
    /// The live settlements. **Empty means no civilisation layer**, and the phase is then `None`.
    pub settlements: &'a [LandmarkSite],
}

/// A landmark's phase and the plain-language reason for it, or `None` when the world has no
/// settlement at all (there is no record of human presence either way, so *Natural* would be a
/// claim, not a reading).
///
/// Evaluated in order: a kind that *is* the ruin or the abandonment says so; then monumental,
/// then developed; then a natural feature is *Utilised* if a settlement lives within
/// [`PHASE_USE_KM`] and *Natural* otherwise.
pub fn landmark_phase(lm: &Landmark, inp: &PhaseInputs<'_>) -> Option<(LandmarkPhase, String)> {
    if inp.settlements.is_empty() || inp.gw == 0 || !(inp.cell_km > 0.0) {
        return None;
    }
    let kind = lm.kind.as_str();
    let by_kind = |p: LandmarkPhase, why: &str| Some((p, why.to_string()));
    match kind {
        "ruin" | "destroyed_fortress" => return by_kind(LandmarkPhase::Ruined, "the record leaves this place in ruins"),
        "abandoned_settlement" | "ancient_road" | "historic_crossing" => {
            return by_kind(LandmarkPhase::Abandoned, "a place the record shows people used and left")
        }
        _ => {}
    }
    if MONUMENTAL_KINDS.contains(&kind) {
        return by_kind(LandmarkPhase::Monumentalised, "set apart by what people did or believed here");
    }
    if DEVELOPED_KINDS.contains(&kind) {
        return by_kind(LandmarkPhase::Developed, "a site people built or work");
    }
    let mut best = f64::INFINITY;
    for s in inp.settlements {
        let mut dx = s.x as f64 - lm.x as f64;
        if inp.world {
            let gw = inp.gw as f64;
            if dx > gw * 0.5 {
                dx -= gw;
            } else if dx < -gw * 0.5 {
                dx += gw;
            }
        }
        let d = (dx * dx + (s.y as f64 - lm.y as f64).powi(2)).sqrt() * inp.cell_km;
        if d < best {
            best = d;
        }
    }
    if best <= PHASE_USE_KM {
        Some((LandmarkPhase::Utilised, format!("a settlement lives {} away", fmt_km(best))))
    } else {
        Some((LandmarkPhase::Natural, format!("no settlement within {}", fmt_km(PHASE_USE_KM))))
    }
}


/// The key a landmark would have had in the **other era** of its record: a `battlefield` at
/// `(x, y)` becomes a `battlefield_historic` there when its battle ends, and back if the year
/// moves. `None` for every other kind. A landmark's key is `kind@x,y`, so without this a name the
/// person typed would be orphaned the moment a re-run changed the era; the bridge reads the alias
/// when a key has no name of its own and clears it with the key.
pub fn era_alias_key(kind: &str, x: usize, y: usize) -> Option<String> {
    match kind {
        "battlefield" => Some(format!("battlefield_historic@{x},{y}")),
        "battlefield_historic" => Some(format!("battlefield@{x},{y}")),
        _ => None,
    }
}
