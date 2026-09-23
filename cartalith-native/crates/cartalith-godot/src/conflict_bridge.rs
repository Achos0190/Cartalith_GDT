//! `STORY_PLANNING_SCOPE.md` SP-4 -- the conflict overlay's Godot surface
//! over [`cartalith_civ::conflict`].
//!
//! The store is `WorldGen::conflicts`. Its lifecycle, decided here because
//! Ruling AO left it open ("behaviour on the referenced entity's
//! deletion/regenerate needs deciding by whoever builds it"):
//!
//! - **A new world** (`absorb`: generate, import) **or a closed project**
//!   (`load_save`, which `project_open` runs before restoring) **empties the
//!   store**, exactly as it empties knowledge links and landmarks. `tid`s
//!   restart at 1 in a new world, so a kept anchor would silently re-bind to
//!   an unrelated town; and free geometry drawn over the previous world's
//!   grid means nothing over the new one.
//! - **Auto-populate and Clear places** (`civ_populate`, `civ_clear_places`)
//!   replace every settlement on the *same* grid, and reissue `tid`s. The
//!   conflicts are kept -- the terrain they were drawn over is unchanged --
//!   but every anchor is **detached** first, baking the shape in where it is
//!   drawn ([`ConflictStore::detach_all`]).
//! - **One settlement deleted** (`civ_delete_settlement`), or a province
//!   whose seed stops being a seed after a recompute: the anchor is kept and
//!   reported `anchor_resolved: false`; the shape stays where it was attached.
//!   If the `tid` resolves again, it re-attaches.
//!
//! Saved in `entities/conflicts.json` (engine-owned; `project_bridge.rs`).

use cartalith_civ::conflict::{Conflict, ConflictAnchor, ConflictError, ConflictKind, ConflictStore};
use godot::prelude::*;

use crate::{CivData, WorldGen};

/// Where `a` is now, in grid cells, or `None` when it no longer resolves.
pub(crate) fn anchor_pos(civ: &CivData, a: ConflictAnchor) -> Option<(f64, f64)> {
    let tid = match a {
        ConflictAnchor::Settlement(t) => t,
        ConflictAnchor::Province(t) => {
            // Still a province seed? Otherwise the province it named is gone.
            province_of_seed(civ, t)?;
            t
        }
    };
    civ.settlements
        .iter()
        .find(|s| s.tid == tid)
        .map(|s| (s.placement.x as f64, s.placement.y as f64))
}

fn province_of_seed(civ: &CivData, seed_tid: u64) -> Option<&cartalith_civ::Province> {
    civ.province_list
        .iter()
        .find(|p| civ.settlements.get(p.capital_settlement_index).is_some_and(|s| s.tid == seed_tid))
}

fn anchor_name(civ: &CivData, a: ConflictAnchor) -> Option<String> {
    match a {
        ConflictAnchor::Settlement(t) => civ.settlements.iter().find(|s| s.tid == t).map(|s| s.name.clone()),
        ConflictAnchor::Province(t) => province_of_seed(civ, t).map(|p| p.name.clone()),
    }
}

pub(crate) fn anchor_key(a: ConflictAnchor) -> (&'static str, u64) {
    match a {
        ConflictAnchor::Settlement(t) => ("settlement", t),
        ConflictAnchor::Province(t) => ("province", t),
    }
}

pub(crate) fn anchor_from_key(kind: &str, tid: u64) -> Option<ConflictAnchor> {
    match kind {
        "settlement" if tid > 0 => Some(ConflictAnchor::Settlement(tid)),
        "province" if tid > 0 => Some(ConflictAnchor::Province(tid)),
        _ => None,
    }
}

fn error_text(e: &ConflictError) -> String {
    match e {
        ConflictError::TooFewPoints { kind, need, got } => {
            format!("A {} needs at least {need} point(s); it has {got}.", kind.key())
        }
        ConflictError::EndBeforeStart => "The end year is before the start year.".into(),
        ConflictError::BadSide(f) => format!("Faction {f} cannot be a side."),
        ConflictError::NonFinitePoint => "A point is not a finite position.".into(),
        ConflictError::NoSuchConflict(id) => format!("No conflict #{id}."),
    }
}

fn result(ok: bool, id: u64, error: &str) -> VarDictionary {
    let mut d = vdict! { "ok" => ok, "id" => id as i64 };
    if !error.is_empty() {
        d.set("error", error);
    }
    d
}

/// Applies `fields` to `c`. Anchor changes are returned rather than applied,
/// because resolving one needs the civ layer. `Err` names the bad field.
fn apply_fields(c: &mut Conflict, fields: &VarDictionary) -> Result<Option<Option<ConflictAnchor>>, String> {
    let get_s = |k: &str| fields.get(k).and_then(|v| v.try_to::<GString>().ok()).map(|g| g.to_string());
    let get_i = |k: &str| fields.get(k).and_then(|v| v.try_to::<i64>().ok());
    if let Some(n) = get_s("name") {
        c.name = n.trim().to_string();
    }
    if let Some(k) = get_s("kind") {
        c.kind = ConflictKind::from_key(&k).ok_or_else(|| format!("Unknown conflict kind '{k}'."))?;
    }
    if let Some(y) = get_i("start_year") {
        c.start_year = y;
    }
    if fields.get("ongoing").and_then(|v| v.try_to::<bool>().ok()) == Some(true) {
        c.end_year = None;
    } else if let Some(y) = get_i("end_year") {
        c.end_year = Some(y);
    }
    if let Some(o) = get_s("outcome") {
        c.outcome = o.trim().to_string();
    }
    if let Some(v) = fields.get("sides") {
        let sides: Vec<i32> = if let Ok(p) = v.try_to::<PackedInt32Array>() {
            p.to_vec()
        } else if let Ok(a) = v.try_to::<VarArray>() {
            a.iter_shared().filter_map(|x| x.try_to::<i64>().ok()).map(|x| x as i32).collect()
        } else {
            return Err("'sides' must be an array of faction indices.".into());
        };
        c.sides = sides;
    }
    if let Some(v) = fields.get("points") {
        let p = v.try_to::<PackedVector2Array>().map_err(|_| "'points' must be a PackedVector2Array.".to_string())?;
        c.points = p.as_slice().iter().map(|v| (f64::from(v.x), f64::from(v.y))).collect();
    }
    Ok(match get_s("anchor_kind") {
        None => None,
        Some(k) if k == "none" || k.is_empty() => Some(None),
        Some(k) => {
            let tid = get_i("anchor_tid").unwrap_or(0).max(0) as u64;
            Some(Some(anchor_from_key(&k, tid).ok_or_else(|| format!("Bad anchor '{k}' #{tid}."))?))
        }
    })
}

impl WorldGen {
    fn conflict_anchor_now(&self, a: ConflictAnchor) -> Option<(f64, f64)> {
        self.civ.as_ref().and_then(|civ| anchor_pos(civ, a))
    }

    /// `apply_fields` then the anchor change, resolved against the live civ
    /// layer. Attaching to a place that does not resolve is refused -- only a
    /// place that exists now can be attached to.
    fn conflict_apply(&self, c: &mut Conflict, fields: &VarDictionary) -> Result<(), String> {
        let before = c.resolved_points(c.anchor.and_then(|a| self.conflict_anchor_now(a)));
        let had_points = fields.contains_key("points");
        let anchor_change = apply_fields(c, fields)?;
        let drawn = if had_points { c.points.clone() } else { before };
        if let Some(sides_bad) = c.sides.iter().find(|&&f| {
            !self.civ.as_ref().is_some_and(|civ| civ.faction_roster.is_assignable(f))
        }) {
            return Err(format!("Faction {sides_bad} is not a real faction in this world."));
        }
        match anchor_change {
            None => {
                // Newly drawn geometry under an existing, resolving anchor:
                // re-base it on where the anchor is now. (Unresolved: the new
                // points are already stored and draw as given.)
                if let Some(a) = c.anchor.filter(|_| had_points)
                    && let Some(at) = self.conflict_anchor_now(a)
                {
                    c.set_anchor(Some(a), at, drawn);
                }
            }
            Some(None) => c.set_anchor(None, (0.0, 0.0), drawn),
            Some(Some(a)) => {
                let at = self.conflict_anchor_now(a).ok_or_else(|| {
                    let (k, t) = anchor_key(a);
                    format!("No {k} #{t} in this world to attach to.")
                })?;
                c.set_anchor(Some(a), at, drawn);
            }
        }
        Ok(())
    }

    fn conflict_dict(&self, c: &Conflict) -> VarDictionary {
        let now = c.anchor.and_then(|a| self.conflict_anchor_now(a));
        let pts: PackedVector2Array =
            c.resolved_points(now).iter().map(|&(x, y)| Vector2::new(x as f32, y as f32)).collect();
        let sides: PackedInt32Array = c.sides.iter().copied().collect();
        let year = self.civ.as_ref().map_or(0, |civ| civ.year);
        let mut d = vdict! {
            "id" => c.id as i64,
            "name" => c.name.as_str(),
            "kind" => c.kind.key(),
            "start_year" => c.start_year,
            "sides" => &sides,
            "outcome" => c.outcome.as_str(),
            "points" => &pts,
            "active" => c.active_in(year),
        };
        // An open-ended conflict has no end year: omitted, never a sentinel.
        if let Some(e) = c.end_year {
            d.set("end_year", e);
        }
        if let Some(a) = c.anchor {
            let (k, t) = anchor_key(a);
            d.set("anchor_kind", k);
            d.set("anchor_tid", t as i64);
            d.set("anchor_resolved", now.is_some());
            if let Some(n) = self.civ.as_ref().and_then(|civ| anchor_name(civ, a)) {
                d.set("anchor_name", n.as_str());
            }
        }
        d
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The four drawable kinds, in tool order: `front`, `arrow` (polylines,
    /// two points or more), `siege`, `battle` (one-point markers).
    #[func]
    fn conflict_kinds() -> PackedStringArray {
        ConflictKind::ALL.iter().map(|k| GString::from(k.key())).collect()
    }

    /// Adds a conflict. `fields`: `name`, `kind`, `start_year`, `end_year`
    /// (or `ongoing: true`), `sides` (faction indices), `outcome`, `points`
    /// (`PackedVector2Array`, grid cells), and optionally `anchor_kind`
    /// (`"settlement"`/`"province"`/`"none"`) with `anchor_tid` (the
    /// settlement's `tid`, or the province's seed settlement's `tid`).
    /// Returns `{ok, id, error?}`.
    #[func]
    fn conflict_add(&mut self, fields: VarDictionary) -> VarDictionary {
        let mut c = Conflict {
            id: 0,
            name: String::new(),
            kind: ConflictKind::Front,
            start_year: self.civ.as_ref().map_or(0, |civ| civ.year),
            end_year: None,
            sides: Vec::new(),
            outcome: String::new(),
            points: Vec::new(),
            anchor: None,
            anchor_at: (0.0, 0.0),
        };
        if let Err(e) = self.conflict_apply(&mut c, &fields) {
            return result(false, 0, &e);
        }
        match self.conflicts.add(c) {
            Ok(id) => result(true, id, ""),
            Err(e) => result(false, 0, &error_text(&e)),
        }
    }

    /// Changes the fields present in `fields` (same keys as
    /// [`Self::conflict_add`]); absent keys are left alone. Nothing changes
    /// on a refusal. Returns `{ok, id, error?}`.
    #[func]
    fn conflict_update(&mut self, id: i64, fields: VarDictionary) -> VarDictionary {
        let id = id.max(0) as u64;
        let Some(mut c) = self.conflicts.get(id).cloned() else {
            return result(false, id, &error_text(&ConflictError::NoSuchConflict(id)));
        };
        if let Err(e) = self.conflict_apply(&mut c, &fields) {
            return result(false, id, &e);
        }
        match self.conflicts.replace(c) {
            Ok(()) => result(true, id, ""),
            Err(e) => result(false, id, &error_text(&e)),
        }
    }

    #[func]
    fn conflict_delete(&mut self, id: i64) -> bool {
        self.conflicts.remove(id.max(0) as u64)
    }

    /// Every conflict, in creation order. Each: `id`, `name`, `kind`,
    /// `start_year`, `end_year` (absent when ongoing), `sides`
    /// (`PackedInt32Array`), `outcome`, `points` (**resolved**: translated by
    /// however far the anchor has moved), `active` (in the Timeline cursor's
    /// year, `get_civ_year()`), and when anchored `anchor_kind`, `anchor_tid`,
    /// `anchor_resolved`, and `anchor_name` (absent when unresolved).
    #[func]
    fn conflict_list(&self) -> Array<VarDictionary> {
        self.conflicts.conflicts.iter().map(|c| self.conflict_dict(c)).collect()
    }

    /// One conflict as [`Self::conflict_list`] describes it; empty for an
    /// unknown id.
    #[func]
    fn conflict_get(&self, id: i64) -> VarDictionary {
        self.conflicts.get(id.max(0) as u64).map_or_else(VarDictionary::new, |c| self.conflict_dict(c))
    }

    /// "What conflicts happened here" (Ruling AO): ids of the conflicts
    /// attached to settlement `tid` (`kind == "settlement"`) or to the
    /// province seeded by settlement `tid` (`kind == "province"`).
    #[func]
    fn conflicts_attached_to(&self, kind: GString, tid: i64) -> PackedInt64Array {
        anchor_from_key(&kind.to_string(), tid.max(0) as u64)
            .map_or_else(Vec::new, |a| self.conflicts.attached_to(a))
            .into_iter()
            .map(|i| i as i64)
            .collect()
    }

    /// SP-4's "reads the numbers": one row per side, in side order --
    /// `faction`, `name`, and when the world has a manpower row for it,
    /// `standing_army`, `professional_core`, `field_army`,
    /// `emergency_mobilization`, `field_duration_days`,
    /// `emergency_duration_days`, `era`. The figures are
    /// `cartalith_civ::manpower`'s, through the same pass CIVIL ▸ Military
    /// shows (`manpower_by_faction`), so the two can never disagree.
    ///
    /// **They are the world as it stands, not the conflict's year.** The
    /// model reads the live settlement list and roster; `civ_goto_year`
    /// swaps only territory. So scrubbing to a conflict's year changes the
    /// territory term and nothing else -- stated in the readout, not hidden.
    /// A side with no row carries no manpower keys (absent, never zero).
    #[func]
    fn conflict_sides_manpower(&self, id: i64) -> Array<VarDictionary> {
        let Some(c) = self.conflicts.get(id.max(0) as u64) else { return Array::new() };
        let rows = self.manpower_by_faction();
        cartalith_civ::conflict::side_manpower(&c.sides, &rows)
            .iter()
            .map(|s| {
                let name = self
                    .civ
                    .as_ref()
                    .and_then(|civ| civ.faction_roster.0.get(s.faction as usize))
                    .map_or_else(String::new, |e| e.name.clone());
                let mut d = vdict! { "faction" => s.faction as i64, "name" => name.as_str() };
                if let Some(m) = s.manpower {
                    d.set("standing_army", m.standing_army);
                    d.set("professional_core", m.professional_core);
                    d.set("field_army", m.field_army);
                    d.set("emergency_mobilization", m.emergency_mobilization);
                    d.set("field_duration_days", m.field_duration_days);
                    d.set("emergency_duration_days", m.emergency_duration_days);
                    d.set("era", m.era_band.name);
                }
                d
            })
            .collect()
    }
}

/// Used by the passes that reissue every `tid` (see this module's doc).
pub(crate) fn detach_all(store: &mut ConflictStore, civ: Option<&CivData>) {
    store.detach_all(|a| civ.and_then(|c| anchor_pos(c, a)));
}
