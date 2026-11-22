//! Finding F26 under owner Ruling GL (2026-11-22): the shell's view of
//! city-states and their spheres of influence (`cartalith_civ::city_states`
//! holds the model), the promotion of a sphere to a faction, and its undo.
//!
//! **Derived on read, never generated.** Behind `CivParams::city_states`
//! (on in `params::defaults`, off in `WorldParams::defaults` and for a save
//! without the key), the spheres are computed from the claim grid on screen,
//! the settlements of the year those claims belong to, and the territory cost
//! `compute_civilisation` swept the claims over. `compute_civilisation` never
//! reads the switch, so no generated output, golden or FH-R2/G1 baseline moves.
//!
//! **The year.** Ruling AT keeps `CivData::territory` at the recorded year
//! the timeline last loaded (`CivData::territory_year`), and a timeline jump
//! never touches the live settlements. So the spheres read that year's
//! recorded settlements (`TimelineSnapshot::settlements`, with that year's
//! populations) when a recorded year's claims are on screen, and the live
//! settlements otherwise: as settlements grow and shrink over the
//! collapse/recovery timeline, the spheres follow per recorded year with no
//! stored state. Alignment is keyed by `tid` for the same reason.
//!
//! **The cost.** Territory is swept over `build_travel_cost` (Ruling GO at
//! strength 0) or over Ruling GO's friction with its river edge cost. The
//! friction reads the civ pass's river, biome and corridor rasters, which
//! `compute_civilisation` frees; [`TerritoryCost::build`] rebuilds them from
//! the world state exactly as that pass does, once, and `WorldGen` caches the
//! result keyed on every input (see [`TerritoryCost::key`]). The cache is
//! resident only while the switch is on and spheres have been asked for:
//! 4 bytes a cell for the road cost, 5 for the friction (13.4 MB on the
//! default 2048 x 1311 grid), and none at all with the switch off.
//!
//! What it must never do: compute spheres with the switch off, take a
//! faction's land or settlement into a sphere, or promote without leaving an
//! undo step.

use std::collections::HashMap;
use std::rc::Rc;

use cartalith_civ::border_terrain::{BorderTerrain, Friction};
use cartalith_civ::city_states::Spheres;
use cartalith_civ::{NamedSettlement, SettlementKind};

use godot::prelude::*;

use crate::civ_tools_bridge::CivTools;
use crate::sample_bridge::hash_bytes;
use crate::{CivData, WorldGen};

/// The cost the territory sweep ran over, rebuilt on demand.
pub(crate) enum TerritoryCost {
    /// `build_travel_cost`, the §7b sweep (Ruling GO at strength 0).
    Road(Vec<f32>),
    /// Ruling GO's friction, read through its own per-step edge cost.
    Friction(Friction),
}

/// `i32` as bytes for an in-process cache key only (never persisted).
fn i32_bytes(v: &[i32]) -> &[u8] {
    // SAFETY: `i32` has no padding and no invalid bit patterns.
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), std::mem::size_of_val(v)) }
}

/// Folds a list of words into a key.
fn fold(seed: u64, words: &[u64]) -> u64 {
    let b: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    hash_bytes(seed, &b)
}

/// The inputs [`TerritoryCost::build`] reads beside the world state.
pub(crate) struct CostInputs<'a> {
    pub ws: &'a cartalith_engine::WorldState,
    /// `CivData::water_bodies`, the classification the civ pass built its biome from.
    pub water_bodies: &'a [u8],
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub map_width_km: f64,
    pub river_density: f64,
    pub civ: &'a cartalith_engine::CivParams,
}

impl TerritoryCost {
    /// Every input the cost reads, as one key: the height field, the
    /// classification, sea level, wrap, the friction strength and the river
    /// network's own inputs (flow, density, width and the three switches that
    /// choose which network the civ pass reads). Any of them moving moves the
    /// key.
    pub(crate) fn key(i: &CostInputs) -> u64 {
        let mut k = hash_bytes(0x6369_7479, crate::story_bridge::f32_bytes(&i.ws.field));
        k = hash_bytes(k, crate::story_bridge::f32_bytes(&i.ws.flow_discharge));
        k = hash_bytes(k, i.water_bodies);
        fold(
            k,
            &[
                i.gw as u64,
                i.gh as u64,
                u64::from(i.world),
                i.ws.sea_level.to_bits(),
                i.civ.border_friction.to_bits(),
                i.map_width_km.to_bits(),
                i.river_density.to_bits(),
                u64::from(i.civ.map_rivers) | u64::from(i.civ.river_front) << 1 | u64::from(i.civ.site_map_terms) << 2,
            ],
        )
    }

    /// The cost `compute_civilisation`'s territory ran over, rebuilt the way
    /// that pass builds it: the road cost at strength 0, else the friction
    /// over the same biome, corridor and river rasters (each built by the call
    /// the pass makes, in `lib.rs::compute_civilisation`).
    pub(crate) fn build(i: &CostInputs) -> Self {
        let (ws, gw, gh, sea) = (i.ws, i.gw, i.gh, i.ws.sea_level);
        let cost = cartalith_civ::build_travel_cost(&ws.field, gw, gh, sea);
        if i.civ.border_friction <= 0.0 {
            return TerritoryCost::Road(cost);
        }
        let biome = cartalith_civ::build_biome_raster(i.water_bodies, &ws.temperature, &ws.rainfall);
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, i.map_width_km);
        let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, i.world);
        let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, i.world, flow_thresh);
        drop(raw_slope);
        let map_net = if i.civ.river_front || i.civ.map_rivers || i.civ.site_map_terms { crate::map_river_network(ws, gw, gh) } else { None };
        let (river_order, _) = crate::civ_river_network(ws, gw, gh, sea, i.world, i.river_density, i.map_width_km, i.civ.map_rivers, map_net.as_ref());
        let t = BorderTerrain {
            field: &ws.field,
            river_order: &river_order,
            biome: &biome,
            corridors: &corridors,
            flow: &ws.flow_discharge,
            flow_thresh,
            gw,
            gh,
            sea,
            world: i.world,
        };
        TerritoryCost::Friction(cartalith_civ::border_terrain::territory_friction(&cost, &t, i.civ.border_friction))
    }

    /// The spheres over this cost.
    pub(crate) fn spheres(&self, settlements: &[NamedSettlement], owner: &[i32], gw: usize, gh: usize, world: bool) -> Spheres {
        match self {
            TerritoryCost::Road(c) => cartalith_civ::city_states::spheres(settlements, owner, c, None, gw, gh, world),
            TerritoryCost::Friction(f) => {
                let edge = |a: usize, b: usize, dx: isize, dy: isize| f.edge(a, b, dx, dy);
                cartalith_civ::city_states::spheres(settlements, owner, &f.node, Some(&edge), gw, gh, world)
            }
        }
    }
}

/// One city-state as the shell reads it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Seat {
    pub tid: u64,
    pub name: String,
    pub pop: u32,
    /// The sphere's id in [`SphereView::cell`].
    pub id: u32,
    pub cells: usize,
    /// The tids aligned to it, itself excluded, in settlement order.
    pub members: Vec<u64>,
    /// `cartalith_civ::roster::civ_faction_color(tid)`: the roster's one rule
    /// for an appended faction's colour, keyed by the seat's stable tid so a
    /// sphere keeps its colour as the roster changes, and the faction a
    /// promotion makes keeps it too.
    pub rgb: (u8, u8, u8),
}

/// The spheres of the state on screen, keyed by tid.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SphereView {
    /// The recorded year whose settlements were read, or `None` for the live ones.
    pub year: Option<i64>,
    pub seats: Vec<Seat>,
    /// Per cell, `0` or a [`Seat::id`].
    pub cell: Vec<u32>,
    /// Member tid to seat tid (a seat maps to itself).
    pub aligned: HashMap<u64, u64>,
}

impl SphereView {
    /// The seat a tid belongs to, by tid.
    pub(crate) fn seat_of(&self, tid: u64) -> Option<&Seat> {
        let s = *self.aligned.get(&tid)?;
        self.seats.iter().find(|x| x.tid == s)
    }
}

/// The settlements the spheres read (see the module doc's "The year") and
/// the recorded year they came from.
pub(crate) fn sphere_settlements(civ: &CivData) -> (&[NamedSettlement], Option<i64>) {
    if let Some(y) = civ.territory_year {
        if let Some(s) = civ.timeline.iter().find(|s| s.year == y) {
            return (&s.settlements, Some(y));
        }
    }
    (&civ.settlements, None)
}

/// The key of a sphere result over `cost_key`: the claim grid and every
/// settlement field the model reads (tid, cell, population, faction).
pub(crate) fn view_key(cost_key: u64, settlements: &[NamedSettlement], owner: &[i32], year: Option<i64>) -> u64 {
    let k = hash_bytes(cost_key, i32_bytes(owner));
    let mut w = vec![year.map_or(u64::MAX, |y| y as u64)];
    for s in settlements {
        w.extend([s.tid, s.placement.x as u64, s.placement.y as u64, u64::from(s.pop), s.placement.faction as u64]);
    }
    fold(k, &w)
}

/// The view of [`Spheres`] computed over `settlements` (the shell's form).
pub(crate) fn view_of(sp: &Spheres, settlements: &[NamedSettlement], year: Option<i64>) -> SphereView {
    let mut cells = vec![0usize; sp.seats.len()];
    for &c in &sp.cell {
        if c != 0 {
            cells[c as usize - 1] += 1;
        }
    }
    let seats = sp
        .seats
        .iter()
        .enumerate()
        .map(|(k, &si)| {
            let s = &settlements[si];
            Seat {
                tid: s.tid,
                name: s.name.clone(),
                pop: s.pop,
                id: k as u32 + 1,
                cells: cells[k],
                members: sp.members_of(si).iter().map(|&m| settlements[m].tid).collect(),
                rgb: cartalith_civ::roster::civ_faction_color(s.tid as usize),
            }
        })
        .collect();
    let aligned = sp
        .aligned
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.map(|seat| (settlements[i].tid, settlements[seat].tid)))
        .collect();
    SphereView { year, seats, cell: sp.cell.clone(), aligned }
}

/// The cache `WorldGen` holds: the rebuilt cost and the last view.
#[derive(Default)]
pub(crate) struct SphereCache {
    pub cost: Option<(u64, TerritoryCost)>,
    pub view: Option<(u64, Rc<SphereView>)>,
}

/// What a promotion changed, held so it can be undone. `members` and `cells`
/// carry the values from before the promotion.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PromoteUndo {
    /// The ledger row's seq (0 until recorded).
    pub seq: u64,
    /// The faction id the promotion appended.
    pub faction: i32,
    /// Its name, part of the liveness test.
    pub name: String,
    /// `(tid, faction, capital, kind)` of every settlement it moved.
    pub members: Vec<(u64, i32, bool, SettlementKind)>,
    /// `(cell, paint)` of every cell it claimed. Each was unclaimed (`0`)
    /// before, so the claim grid needs no record: the roster's `remove_last`
    /// puts every cell of the removed faction back to `0` (a mutant restoring
    /// it here survived, 2026-11-22, for that reason, and was deleted).
    pub cells: Vec<(usize, u8)>,
    pub label: String,
}

/// Why a promotion was refused, as the stable token the shell reads and a sentence.
pub(crate) fn refusal(key: &'static str) -> (&'static str, &'static str) {
    match key {
        "off" => (key, "City-states are off for this world (Preferences > Generation rules: City-states)."),
        "no_world" => (key, "Generate a world first."),
        "not_a_seat" => (key, "That place is not a city-state with a sphere of influence."),
        "no_claim_grid" => (key, "This world has no claim grid to write the new faction's land into."),
        _ => ("paint_limit", "The territory paint holds faction ids up to 255; remove a faction first."),
    }
}

/// Turns the sphere of seat `seat_tid` into a new faction: appends one to the
/// roster (named after the seat, in the sphere's colour), moves the seat (made
/// its capital) and every aligned settlement into it, claims the sphere's
/// cells as committed Territory paint (so the land survives a Recompute, as a
/// polity move's does), and rebuilds the provinces. Returns what it changed.
///
/// Refuses, changing nothing, when the seat is not in `view`, when there is
/// no claim grid or no Territory tool state, or past the paint's u8 ids.
pub(crate) fn promote(
    civ: &mut CivData,
    tools: Option<&mut CivTools>,
    view: &SphereView,
    seat_tid: u64,
    gw: usize,
    gh: usize,
) -> Result<PromoteUndo, &'static str> {
    let seat = view.seats.iter().find(|s| s.tid == seat_tid).ok_or("not_a_seat")?;
    let n = gw * gh;
    let tools = tools.ok_or("no_claim_grid")?;
    if civ.territory.len() != n || view.cell.len() != n || tools.territory_base.len() != n {
        return Err("no_claim_grid");
    }
    let id = civ.faction_roster.0.len();
    let id_u8 = u8::try_from(id).map_err(|_| "paint_limit")?;
    let added = civ.faction_roster.add();
    debug_assert_eq!(added, id);
    let entry = &mut civ.faction_roster.0[id];
    entry.name = seat.name.clone();
    entry.color_override = Some(seat.rgb);
    let fid = id as i32;
    let mut members = Vec::new();
    for s in civ.settlements.iter_mut() {
        let is_seat = s.tid == seat_tid;
        // Only a place still loose: a faction's settlement is never taken.
        if s.placement.faction != 0 || !(is_seat || seat.members.contains(&s.tid)) {
            continue;
        }
        members.push((s.tid, s.placement.faction, s.placement.capital, s.placement.kind));
        s.placement.faction = fid;
        if is_seat {
            s.placement.capital = true;
            if !matches!(s.placement.kind, SettlementKind::Metropolis | SettlementKind::Capital) {
                s.placement.kind = SettlementKind::Capital;
            }
        }
    }
    let mut cells = Vec::new();
    for c in 0..n {
        // Only cells still unclaimed: the view may be one year's, and the claim grid is what is written.
        if view.cell[c] != seat.id || civ.territory[c] != 0 {
            continue;
        }
        let old_paint = tools.territory_paint.cells().map_or(0, |p| p[c]);
        cells.push((c, old_paint));
        tools.territory_paint.cells_mut(n)[c] = id_u8;
        civ.territory[c] = fid;
    }
    crate::civ_reprovince(civ, gw, gh);
    let label = format!("Make {} a faction", seat.name);
    Ok(PromoteUndo { seq: 0, faction: fid, name: seat.name.clone(), members, cells, label })
}

/// Whether `u` can still be undone: its faction is still the roster's last,
/// under the same name, and every settlement it moved is still in it.
pub(crate) fn promote_live(civ: &CivData, u: &PromoteUndo) -> bool {
    let f = u.faction as usize;
    civ.faction_roster.0.len() == f + 1
        && civ.faction_roster.0[f].name == u.name
        && u.members.iter().all(|(tid, ..)| civ.settlements.iter().any(|s| s.tid == *tid && s.placement.faction == u.faction))
}

/// Reverts `u` (the caller has checked [`promote_live`]): the cells' paint,
/// the settlements' faction, capital flag and class, then the faction itself
/// (the roster's `remove_last`, which returns its cells to unclaimed), and the
/// provinces.
pub(crate) fn unpromote(civ: &mut CivData, tools: Option<&mut CivTools>, u: &PromoteUndo, gw: usize, gh: usize) {
    let n = gw * gh;
    if let Some(t) = tools {
        for &(c, paint) in &u.cells {
            if t.territory_paint.cells().is_some() {
                t.territory_paint.cells_mut(n)[c] = paint;
            }
        }
    }
    for &(tid, f, cap, kind) in &u.members {
        if let Some(s) = civ.settlements.iter_mut().find(|s| s.tid == tid) {
            s.placement.faction = f;
            s.placement.capital = cap;
            s.placement.kind = kind;
        }
    }
    let CivData { faction_roster, settlements, territory, .. } = civ;
    faction_roster.remove_last(settlements, territory);
    crate::civ_reprovince(civ, gw, gh);
}

/// A sphere's wash alpha as a share of the faction wash's
/// (`WorldGen::territory_opacity`). **Owner, 2026-11-22:** *"Can't really see
/// the city state borders, can you change the opacity in the layer to 100%"* --
/// so 1.0: a sphere washes at the full Territory layer opacity, the same as a
/// faction's land (the first build's 0.75 read too faint on yellow/green
/// terrain). The Territory layer's own opacity still scales both together.
pub(crate) const SPHERE_ALPHA_SCALE: f64 = 1.0;

/// The History row's reason once a promotion can no longer be undone.
pub(crate) const STALE_REASON: &str = "the faction it made has changed since (a later faction was added, it was renamed or deleted, or a place left it)";

impl WorldGen {
    /// The spheres of the state on screen, or `None` with the switch off,
    /// without a generated world or civilisation layer, or without a whole
    /// claim grid. Cached: the cost on [`TerritoryCost::key`], the view on
    /// [`view_key`] over it, so a timeline jump, an edit or a promotion
    /// recomputes and a repeated read does not.
    pub(crate) fn sphere_view(&self) -> Option<Rc<SphereView>> {
        if !self.params.civ.city_states {
            return None;
        }
        let Some(crate::WorldSource::Generated(ws)) = self.source.as_ref() else { return None };
        let civ = self.civ.as_ref()?;
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || civ.territory.len() != gw * gh || civ.water_bodies.len() != gw * gh {
            return None;
        }
        let inputs = CostInputs {
            ws,
            water_bodies: &civ.water_bodies,
            gw,
            gh,
            world: self.world,
            map_width_km: self.map_width_km,
            river_density: self.params.river_density,
            civ: &self.params.civ,
        };
        let mut cache = self.sphere_cache.borrow_mut();
        let ck = TerritoryCost::key(&inputs);
        if cache.cost.as_ref().is_none_or(|(k, _)| *k != ck) {
            cache.view = None;
            cache.cost = Some((ck, TerritoryCost::build(&inputs)));
        }
        let (settlements, year) = sphere_settlements(civ);
        let vk = view_key(ck, settlements, &civ.territory, year);
        if let Some((k, v)) = cache.view.as_ref() {
            if *k == vk {
                return Some(v.clone());
            }
        }
        let cost = &cache.cost.as_ref().expect("built above").1;
        let v = Rc::new(view_of(&cost.spheres(settlements, &civ.territory, gw, gh, self.world), settlements, year));
        cache.view = Some((vk, v.clone()));
        Some(v)
    }

    /// Reverts the newest live sphere promotion and returns its label.
    pub(crate) fn undo_sphere_step(&mut self) -> Option<String> {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let civ = self.civ.as_mut()?;
        if !self.sphere_undo.last().is_some_and(|u| promote_live(civ, u)) {
            return None;
        }
        let u = self.sphere_undo.pop()?;
        unpromote(civ, self.civ_tools.as_mut(), &u, gw, gh);
        self.ledger.remove_seq(u.seq);
        self.civ_dirty = true;
        Some(u.label)
    }

    /// The sphere dictionary of one seat (`civ_spheres`' rows).
    fn seat_dict(&self, s: &Seat) -> VarDictionary {
        let civ = self.civ.as_ref();
        let index_of = |tid: u64| civ.and_then(|c| c.settlements.iter().position(|x| x.tid == tid)).map_or(-1, |i| i as i64);
        let members: PackedInt64Array = s.members.iter().map(|&t| index_of(t)).filter(|&i| i >= 0).collect();
        let cell_km = if self.gw > 0 { self.map_width_km / self.gw as f64 } else { 0.0 };
        vdict! {
            "tid" => s.tid as i64,
            "index" => index_of(s.tid),
            "name" => s.name.as_str(),
            "population" => s.pop as i64,
            "cells" => s.cells as i64,
            "km2" => s.cells as f64 * cell_km * cell_km,
            "members" => &members,
            "color" => Color::from_rgba8(s.rgb.0, s.rgb.1, s.rgb.2, 255),
        }
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// F26 / Ruling GL: every city-state of the state on screen and its
    /// sphere. `{}` with `civ.city_states` off or without a civilisation
    /// layer and a claim grid; otherwise `{year, seats, aligned}`: `year` only
    /// when a recorded year's settlements were read (omitted for the live
    /// ones), `seats` one row per city-state (`tid`, `index` into
    /// `get_settlements()` or `-1` when the place is not live, `name`,
    /// `population`, `cells`, `km2`, `members` as live indices, `color`), and
    /// `aligned` the number of settlements aligned to some seat, seats
    /// excluded.
    #[func]
    fn civ_spheres(&self) -> VarDictionary {
        let Some(v) = self.sphere_view() else { return VarDictionary::new() };
        let seats: Array<VarDictionary> = v.seats.iter().map(|s| self.seat_dict(s)).collect();
        let aligned = v.seats.iter().map(|s| s.members.len() as i64).sum::<i64>();
        let mut d = vdict! { "seats" => &seats, "aligned" => aligned };
        if let Some(y) = v.year {
            d.set("year", y);
        }
        d
    }

    /// F26: settlement `index`'s place in a sphere. `{}` when it is in none
    /// (or the switch is off); otherwise `role` (`"seat"` for a city-state,
    /// `"aligned"` for a place aligned to one), `seat` (the seat's row, as
    /// `civ_spheres`' `seats`), and `year` when a recorded year's settlements
    /// were read.
    #[func]
    fn civ_settlement_sphere(&self, index: i64) -> VarDictionary {
        let Some(v) = self.sphere_view() else { return VarDictionary::new() };
        let Some(tid) = self.civ.as_ref().and_then(|c| usize::try_from(index).ok().and_then(|i| c.settlements.get(i))).map(|s| s.tid) else {
            return VarDictionary::new();
        };
        let Some(seat) = v.seat_of(tid) else { return VarDictionary::new() };
        let mut d = vdict! { "role" => if seat.tid == tid { "seat" } else { "aligned" }, "seat" => &self.seat_dict(seat) };
        if let Some(y) = v.year {
            d.set("year", y);
        }
        d
    }

    /// F26 / Ruling GL item 4: **"Make this sphere a faction"**. Settlement
    /// `index` must be a city-state with a sphere; its sphere becomes a new
    /// faction through the roster ([`promote`]). Returns `{ok: true, faction,
    /// name, members, cells}` or `{ok: false, reason, message}` (`reason` one
    /// of `off`, `no_world`, `not_a_seat`, `no_claim_grid`, `paint_limit`).
    /// One undo step (`EntryKind::SpherePromote`): Edit > Undo removes the
    /// faction and puts every place and cell back. Marks the civ layer dirty,
    /// as a polity move does (routes, trade and the economy derive from faction).
    #[func]
    fn civ_promote_sphere(&mut self, index: i64) -> VarDictionary {
        let fail = |k: &'static str| {
            let (key, msg) = refusal(k);
            vdict! { "ok" => false, "reason" => key, "message" => msg }
        };
        if !self.params.civ.city_states {
            return fail("off");
        }
        let Some(view) = self.sphere_view() else { return fail("no_world") };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let Some(civ) = self.civ.as_mut() else { return fail("no_world") };
        let Some(tid) = usize::try_from(index).ok().and_then(|i| civ.settlements.get(i)).map(|s| s.tid) else {
            return fail("not_a_seat");
        };
        let mut u = match promote(civ, self.civ_tools.as_mut(), &view, tid, gw, gh) {
            Ok(u) => u,
            Err(k) => return fail(k),
        };
        self.civ_dirty = true;
        u.seq = self.ledger.record("civ", u.label.clone(), format!("{} places, {} cells", u.members.len(), u.cells.len()), crate::undo::EntryKind::SpherePromote);
        let out = vdict! {
            "ok" => true,
            "faction" => u.faction as i64,
            "name" => u.name.as_str(),
            "members" => u.members.len() as i64,
            "cells" => u.cells.len() as i64,
        };
        self.sphere_undo.push(u);
        out
    }
}

#[cfg(test)]
mod tests;
