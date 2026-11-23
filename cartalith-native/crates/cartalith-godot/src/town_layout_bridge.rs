//! Frozen town layouts in the save (`TOWN_LAYOUT_SAVE_SCOPE.md`, Rulings GZ and HA, 2026-11-22): the slot
//! `entities/town_layouts.json`, the freeze trigger, the serve path, drift checks and the re-lay.
//!
//! The owner's request (Ruling GZ): *"how we can save town layouts without it being graphics ... that way if a user
//! maps poi or pins a town that information won't be lost."* A town is a pure function of the world under it, the
//! place editor, twelve `urban_*` switches, the rules and the generator's code, and a rules change alone moved 64 of
//! 64 census towns (scope §3.3). So the first time a mark lands in a town (a point of interest, a label, an icon or a
//! saved measurement: Ruling HA's Q6 widened POIs to all four) the town is **frozen**: its drawn geometry is kept as
//! integer vector data (`cartalith_civ::town_freeze`) and served by `urban_layouts` in preference to generating it
//! again, so the map, the City Viewer and the planner see the town the pin was placed on.
//!
//! ## What it must never do
//!
//! - **Replace a frozen town silently** (scope §2 item 4). A difference is reported (`town_state`, `town_check`,
//!   the open report); only the user's explicit "Re-lay with current rules" changes it, and that is one undo step.
//! - **Delete a pin.** A pin that finds no matching feature after a re-lay stays exactly where it is, keeps its id,
//!   name and note, and is reported detached (Ruling HA, Q8).
//! - **Drop a row it cannot use.** A damaged row, an unknown encoding or a newer document is kept verbatim and
//!   written back; its town is generated this session (scope §7).
//! - **Touch an unfrozen town.** A town nobody pinned is exactly the computation it was: no row, no extra work.
//!
//! ## Undo
//!
//! The table is an `Arc<TownTable>` that every change replaces (copy on write), and every change happens inside
//! `WorldGen::anno_scope_multi`, which records the table's before and after on the same ledger row as the mark that
//! caused it (`annotation_undo::AnnoEntry::towns`). So undoing the placement that froze a town unfreezes it, and a
//! re-lay with all its moved pins, labels and icons undoes in one step.

use std::collections::BTreeMap;
use std::sync::Arc;

use godot::prelude::*;
use serde_json::{Map, Value, json};

use cartalith_civ::town_freeze::{self as tf, Anchor, FeatKind};
use cartalith_civ::urban_adapter::{UrbanLayout, Vec2 as UVec2};

use crate::annotation_undo::{AnnoSnap, AnnoStore};
use crate::WorldGen;

/// The slot (`SAVEFILE_COMPAT.md` §9.10).
pub(crate) const SLOT: &str = "entities/town_layouts.json";

/// The document version this build writes and reads; a greater one is kept verbatim (scope §7).
pub(crate) const DOC_VERSION: i64 = 1;

/// A mark within this many metres of a settlement's cell centre is in its town, the City Viewer's own
/// `POI_TOWN_RADIUS_M` (half the map's 1.7 km site box, `city_viewer_window.gd`), so the town a pin freezes is the
/// town the viewer shows it in. A frozen stretched town widens it to half its longer side.
pub(crate) const TOWN_RADIUS_M: f64 = 850.0;

/// The timeline chip shows when the settlement's population this year differs from the frozen one by this factor or
/// more either way (Ruling HA, Q3: one layout through the timeline, a chip at a 2x gap).
pub(crate) const POP_CHIP_RATIO: f64 = 2.0;

/// Where a pin sits on a frozen town, with the grid position it had when anchored (`at`, thousandths of a cell: the
/// conflict anchors' `anchor_at` precedent, scope §4.4) and whether the last re-lay left it detached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredAnchor {
    pub a: Anchor,
    pub at: (i64, i64),
    pub detached: bool,
}

/// One frozen town (a row of the slot).
#[derive(Clone)]
pub(crate) struct FrozenTown {
    /// `settlements[].id`, the stable id.
    pub id: u64,
    /// The row as stored. For a usable row everything **but** `anchors` (held in [`Self::anchors`]); for an unusable
    /// one, the whole row exactly as read.
    pub row: Value,
    /// The decoded layout, verified against `layout_hash`; `None` for a row kept verbatim.
    pub layout: Option<Arc<UrbanLayout>>,
    /// Why the row cannot be used (damaged, unknown encoding); `None` for a usable row.
    pub problem: Option<String>,
    /// POI id to anchor (scope §4.4: keyed by POI id; `pois.json` is unchanged).
    pub anchors: BTreeMap<u32, StoredAnchor>,
}

impl PartialEq for FrozenTown {
    /// The stored row and the anchors say everything; the decoded layout is a function of the row.
    fn eq(&self, o: &Self) -> bool {
        self.id == o.id && self.problem == o.problem && self.anchors == o.anchors && self.row == o.row
    }
}

/// The whole slot in memory.
#[derive(Clone, Default)]
pub(crate) struct TownTable {
    /// Rows by settlement id.
    pub rows: BTreeMap<u64, Arc<FrozenTown>>,
    /// Rows with no usable `id`, kept verbatim and written after the others (never dropped, scope §7).
    pub orphans: Vec<Value>,
    /// The whole document, kept verbatim when it was a newer `version` or not an object: written back untouched, and
    /// no town is frozen this session (there is no table to add to).
    pub verbatim: Option<String>,
}

impl std::fmt::Debug for TownTable {
    /// Counts only: a capital's geometry is hundreds of kilobytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TownTable({} rows, {} orphans, verbatim {})", self.rows.len(), self.orphans.len(), self.verbatim.is_some())
    }
}

impl PartialEq for TownTable {
    /// Rows compared by pointer first (a snapshot shares every unchanged row), then by content.
    fn eq(&self, o: &Self) -> bool {
        self.verbatim == o.verbatim
            && self.orphans == o.orphans
            && self.rows.len() == o.rows.len()
            && self.rows.iter().zip(&o.rows).all(|((ka, a), (kb, b))| ka == kb && (Arc::ptr_eq(a, b) || **a == **b))
    }
}

fn anchor_json(poi: u32, s: &StoredAnchor) -> Value {
    let mut m = Map::new();
    m.insert("poi".into(), json!(poi));
    m.insert("kind".into(), json!(s.a.kind.key()));
    if let Some(f) = s.a.fid {
        m.insert("fid".into(), json!(f));
    }
    m.insert("off".into(), json!([s.a.off.0, s.a.off.1]));
    m.insert("tag".into(), json!(s.a.tag));
    m.insert("at".into(), json!([s.at.0, s.at.1]));
    if let Some(u) = s.a.u {
        m.insert("u".into(), json!(u));
    }
    if s.detached {
        m.insert("det".into(), json!(1));
    }
    Value::Object(m)
}

fn anchor_from(v: &Value) -> Option<(u32, StoredAnchor)> {
    let poi = u32::try_from(v.get("poi")?.as_u64()?).ok()?;
    let kind = FeatKind::from_key(v.get("kind")?.as_str()?)?;
    let fid = match v.get("fid") {
        Some(f) => Some(usize::try_from(f.as_u64()?).ok()?),
        None => None,
    };
    let off = v.get("off")?.as_array()?;
    let at = v.get("at")?.as_array()?;
    Some((
        poi,
        StoredAnchor {
            a: Anchor {
                kind,
                fid,
                off: (off.first()?.as_i64()?, off.get(1)?.as_i64()?),
                tag: v.get("tag")?.as_str()?.to_string(),
                u: v.get("u").and_then(Value::as_i64),
            },
            at: (at.first()?.as_i64()?, at.get(1)?.as_i64()?),
            detached: v.get("det").and_then(Value::as_i64) == Some(1),
        },
    ))
}

/// Whether `a` still indexes a feature of `l` (an out-of-range `fid` is dropped on load, scope §7).
fn anchor_in_range(l: &UrbanLayout, a: &Anchor) -> bool {
    tf::anchor_point(l, a).is_some()
}

impl FrozenTown {
    /// The row as written: the stored members plus the anchors of POIs that still exist (an anchor whose pin is gone
    /// is dropped, scope §7), sorted keys.
    fn to_row(&self, live_pois: &std::collections::HashSet<u32>) -> Value {
        if self.layout.is_none() {
            return self.row.clone();
        }
        let mut r = self.row.clone();
        let anchors: Vec<Value> =
            self.anchors.iter().filter(|(id, _)| live_pois.contains(id)).map(|(id, a)| anchor_json(*id, a)).collect();
        if let Some(o) = r.as_object_mut() {
            if anchors.is_empty() {
                o.remove("anchors");
            } else {
                o.insert("anchors".into(), Value::Array(anchors));
            }
        }
        r
    }

    /// The frozen year and population (`frozen` member), when recorded.
    pub(crate) fn frozen_at(&self) -> (Option<i64>, Option<i64>) {
        let f = self.row.get("frozen");
        (f.and_then(|f| f.get("year")).and_then(Value::as_i64), f.and_then(|f| f.get("pop")).and_then(Value::as_i64))
    }

    /// The recorded rules version.
    pub(crate) fn rules_version(&self) -> Option<i64> {
        self.row.get("recipe").and_then(|r| r.get("rules_version")).and_then(Value::as_i64)
    }

    /// The recorded switch set (absent members read as off, scope §4.1).
    pub(crate) fn switches(&self) -> BTreeMap<String, bool> {
        self.row
            .get("recipe")
            .and_then(|r| r.get("switches"))
            .and_then(Value::as_object)
            .map(|o| o.iter().filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b))).collect())
            .unwrap_or_default()
    }
}

/// Builds a usable row: `id`, `frozen` (year and population), `recipe`, `enc`, `layout_hash`, `geom` and `manual` when
/// the user froze it by hand; anchors are held beside.
fn make_row(id: u64, year: i64, pop: i64, recipe: Value, geom: &Value, manual: bool) -> Value {
    let mut m = Map::new();
    m.insert("id".into(), json!(id));
    m.insert("frozen".into(), json!({ "year": year, "pop": pop }));
    m.insert("recipe".into(), recipe);
    m.insert("enc".into(), json!(tf::ENC));
    m.insert("layout_hash".into(), json!(tf::layout_hash(geom)));
    m.insert("geom".into(), geom.clone());
    if manual {
        m.insert("manual".into(), json!(1));
    }
    Value::Object(m)
}

/// Reads one row: usable when its `enc` is known, its geometry decodes and its stored hash is the hash of that
/// geometry. Otherwise kept verbatim with the reason (scope §7's damage table).
fn read_row(v: &Value, live_pois: &std::collections::HashSet<u32>, dropped: &mut usize) -> Option<FrozenTown> {
    let id = v.get("id")?.as_u64()?;
    let problem = (|| -> Result<Arc<UrbanLayout>, String> {
        let enc = v.get("enc").and_then(Value::as_str).ok_or("no enc")?;
        if enc != tf::ENC {
            return Err(format!("uses a newer layout encoding ({enc})"));
        }
        let geom = v.get("geom").ok_or("no geometry")?;
        let stored = v.get("layout_hash").and_then(Value::as_str).ok_or("no layout_hash")?;
        if tf::layout_hash(geom) != stored {
            return Err("its layout_hash does not match its geometry".into());
        }
        v.get("recipe").and_then(Value::as_object).ok_or("no recipe")?;
        Ok(Arc::new(tf::decode(geom)?))
    })();
    Some(match problem {
        Ok(layout) => {
            let mut row = v.clone();
            let mut anchors = BTreeMap::new();
            if let Some(o) = row.as_object_mut() {
                for a in o.remove("anchors").as_ref().and_then(Value::as_array).into_iter().flatten() {
                    match anchor_from(a) {
                        Some((poi, s)) if live_pois.contains(&poi) && anchor_in_range(&layout, &s.a) => {
                            anchors.insert(poi, s);
                        }
                        _ => *dropped += 1,
                    }
                }
            }
            FrozenTown { id, row, layout: Some(layout), problem: None, anchors }
        }
        Err(e) => FrozenTown { id, row: v.clone(), layout: None, problem: Some(e), anchors: BTreeMap::new() },
    })
}

/// The slot read back (scope §7). `text` is the document's verbatim text and `doc` its parsed value; `live_pois` the
/// ids of the POIs this archive restored. Every unusable part is kept verbatim and named in `warnings`.
pub(crate) fn restore(text: Option<&str>, doc: Option<&Value>, live_pois: &[u32], warnings: &mut Vec<String>) -> TownTable {
    let mut t = TownTable::default();
    let (Some(text), Some(doc)) = (text, doc) else { return t };
    let version = doc.get("version").and_then(Value::as_i64);
    let towns = doc.get("towns").and_then(Value::as_array);
    let (Some(v), Some(towns)) = (version, towns) else {
        warnings.push(format!("{SLOT}: not a town-layout document -- kept as it is; every town is drawn from the rules"));
        t.verbatim = Some(text.to_string());
        return t;
    };
    if v > DOC_VERSION {
        warnings.push(format!("{SLOT}: version {v} is newer than this build reads ({DOC_VERSION}) -- kept as it is; every town is drawn from the rules"));
        t.verbatim = Some(text.to_string());
        return t;
    }
    let live: std::collections::HashSet<u32> = live_pois.iter().copied().collect();
    let mut dropped = 0usize;
    for row in towns {
        match read_row(row, &live, &mut dropped) {
            Some(f) => {
                if let Some(p) = &f.problem {
                    warnings.push(format!("{SLOT}: frozen town #{} {p} -- kept as it is and drawn from the rules until you freeze it again", f.id));
                }
                if t.rows.contains_key(&f.id) {
                    warnings.push(format!("{SLOT}: frozen town #{} appears twice -- the second row is kept as it is", f.id));
                    t.orphans.push(row.clone());
                } else {
                    t.rows.insert(f.id, Arc::new(f));
                }
            }
            None => {
                warnings.push(format!("{SLOT}: a frozen town row has no id -- kept as it is"));
                t.orphans.push(row.clone());
            }
        }
    }
    if dropped > 0 {
        warnings.push(format!("{SLOT}: {dropped} pin anchors dropped (their point of interest is gone, or the anchor named a feature the town does not have)"));
    }
    t
}

/// The document's text to save, or `None` when no town is frozen (so a world with none writes no member and
/// re-saves byte for byte, scope §7 MUST). Compact canonical JSON (sorted keys, integers), so a load then save is
/// byte-identical; compact because `insert_doc`'s pretty printing puts every integer of a town on its own line.
pub(crate) fn save_text(t: &TownTable, live_pois: &[u32]) -> Option<String> {
    if let Some(v) = &t.verbatim {
        return Some(v.clone());
    }
    if t.rows.is_empty() && t.orphans.is_empty() {
        return None;
    }
    let live: std::collections::HashSet<u32> = live_pois.iter().copied().collect();
    let mut towns: Vec<Value> = t.rows.values().map(|r| r.to_row(&live)).collect();
    towns.extend(t.orphans.iter().cloned());
    Some(tf::canonical(&json!({ "version": DOC_VERSION, "towns": towns })))
}

/// The recipe of a town (scope §4.1): everything needed to ask the generator for the same town again, as plain
/// values, plus `site_hash` (the world under it) and the rules version. `e` is the settlement's place-editor row.
pub(crate) fn recipe(
    ctx: &cartalith_civ::urban_adapter::UrbanContext,
    o: &cartalith_civ::urban_adapter::PlaceOverrides,
    river_kind_map: bool,
    s: &cartalith_civ::NamedSettlement,
    e: &crate::civ_roster_bridge::PlaceExtras,
    rules: Option<&cartalith_civ::urban_adapter::Rules>,
) -> Value {
    let mut m = Map::new();
    m.insert("rules_version".into(), json!(tf::URBAN_RULES_VERSION));
    m.insert("switches".into(), json!(tf::switch_record(o, river_kind_map)));
    m.insert("seed".into(), json!(ctx.seed));
    m.insert("cell".into(), json!([s.placement.x, s.placement.y]));
    m.insert("pop".into(), json!(ctx.pop.round() as i64));
    m.insert("site_kind".into(), json!(ctx.site_kind));
    m.insert("wall_style".into(), json!(ctx.wall_style));
    m.insert("walls".into(), json!(ctx.walls));
    m.insert("fortified".into(), json!(ctx.fortified));
    m.insert("culture".into(), json!(e.culture));
    m.insert("faith".into(), json!(ctx.faith.as_deref().unwrap_or("")));
    m.insert("specialisation".into(), json!(e.specialisation));
    m.insert("variant".into(), json!(e.variant));
    if let Some(a) = e.age {
        m.insert("age_override".into(), json!(a));
    }
    if let Some(w) = e.walls {
        m.insert("walls_override".into(), json!(w));
    }
    let mut rv = Map::new();
    rv.insert("preset".into(), json!(e.rules_preset));
    if let Some(r) = rules {
        let table: Map<String, Value> =
            crate::urban_bridge::rules_pairs(r).iter().map(|(k, v)| ((*k).to_string(), json!(tf::hex16(v.to_bits())))).collect();
        rv.insert("table".into(), Value::Object(table));
    }
    m.insert("rules".into(), Value::Object(rv));
    m.insert("site_hash".into(), json!(tf::hex16(tf::site_hash(ctx))));
    Value::Object(m)
}

/// The cheap tier of scope §4.2: why a recorded recipe and today's differ, in words, without laying the town out.
/// Empty when they agree. Population and the inferred age are left to the timeline chip (they move with the year).
pub(crate) fn cheap_reasons(recorded: &Value, now: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let rv = |v: &Value| v.get("rules_version").and_then(Value::as_i64);
    if rv(recorded) != rv(now) {
        out.push(format!("drawn by rules v{} (current v{})", rv(recorded).unwrap_or(0), rv(now).unwrap_or(0)));
    }
    let sw = |v: &Value| -> BTreeMap<String, bool> {
        v.get("switches").and_then(Value::as_object).map(|o| o.iter().filter_map(|(k, b)| b.as_bool().map(|b| (k.clone(), b))).collect()).unwrap_or_default()
    };
    let (a, b) = (sw(recorded), sw(now));
    for k in tf::switch_diff(&a, &b) {
        let on = b.get(&k).copied().unwrap_or(false);
        out.push(format!("{} is now {}", k.trim_start_matches("urban_").replace('_', " "), if on { "on" } else { "off" }));
    }
    if recorded.get("site_hash") != now.get("site_hash") {
        out.push("the ground or roads under this town changed since it was drawn".into());
    }
    let fields = ["seed", "cell", "site_kind", "wall_style", "walls", "fortified", "culture", "faith", "specialisation", "variant", "age_override", "walls_override", "rules"];
    let moved: Vec<&str> = fields.iter().copied().filter(|f| recorded.get(*f) != now.get(*f)).collect();
    if !moved.is_empty() {
        out.push(format!("the town's settings changed ({})", moved.join(", ")));
    }
    out
}

/// A report row for a pin a re-lay could not re-attach.
#[derive(Debug, Clone)]
pub(crate) struct Detached {
    pub what: &'static str,
    pub kind: FeatKind,
    pub name: String,
}

/// What a re-lay did, for its report (scope §4.4 step 5, §6).
#[derive(Debug, Clone, Default)]
pub(crate) struct RelayReport {
    pub attached: usize,
    pub ground: usize,
    pub detached: Vec<Detached>,
    /// Share of the old town's buildings with no building of the new town within 2 m of its centre.
    pub changed_share: f64,
}

/// What a re-lay does with one pin (scope §4.4): re-attached to a feature of the new town (and so moved to it),
/// kept at its absolute place because it stood on open ground, or left exactly where it is and reported detached.
/// Every pin gets exactly one outcome, each with its anchor on the **new** town: none is ever dropped.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Relay {
    Attached(Anchor, UVec2),
    Ground(Anchor),
    Detached(Anchor),
}

/// The re-lay of one pin anchored by `a` on `old`, standing at local point `here_new` of the new town `new` (its
/// unmoved place in the new town's frame): [`Relay`]. A re-attach whose anchor point cannot be placed reads as
/// detached, so a pin never moves to a guessed point.
pub(crate) fn relay_one(old: &UrbanLayout, a: &Anchor, new: &UrbanLayout, here_new: UVec2) -> Relay {
    match tf::reattach(old, a, new) {
        Some(na) => match tf::anchor_point(new, &na) {
            Some(p) => Relay::Attached(na, p),
            None => Relay::Detached(tf::choose_anchor(new, here_new)),
        },
        None if a.kind == FeatKind::Ground => Relay::Ground(tf::choose_anchor(new, here_new)),
        None => Relay::Detached(tf::choose_anchor(new, here_new)),
    }
}

/// The re-lay of every mark of a town (scope §4.4), pure: each `(mark, anchor on old, its unmoved place in new's local
/// frame, its grid point as stored)` gets exactly one outcome, `(mark, Some(new local point) when it moves, its anchor on
/// new, detached, its grid point)`, in the same order, and the report counts them. `name` names a mark for the report.
/// **The count out equals the count in**: no mark is ever dropped (TL-4's "0 pins lost", asserted by
/// `relay_keeps_every_mark`).
#[allow(clippy::type_complexity)]
pub(crate) fn plan_moves(
    old: &UrbanLayout,
    new: &UrbanLayout,
    marks: Vec<(MarkRef, Anchor, UVec2, (i64, i64))>,
    name: impl Fn(MarkRef) -> (&'static str, String),
) -> (Vec<(MarkRef, Option<UVec2>, Anchor, bool, (i64, i64))>, RelayReport) {
    let mut rep = RelayReport::default();
    let mut out = Vec::with_capacity(marks.len());
    for (m, a, here_new, here) in marks {
        match relay_one(old, &a, new, here_new) {
            Relay::Attached(na, p) => {
                rep.attached += 1;
                out.push((m, Some(p), na, false, here));
            }
            Relay::Ground(na) => {
                rep.ground += 1;
                out.push((m, None, na, false, here));
            }
            Relay::Detached(na) => {
                let (what, n) = name(m);
                rep.detached.push(Detached { what, kind: a.kind, name: n });
                // Stays exactly where it is, re-anchored to whatever it now stands on (scope §4.4 step 5).
                out.push((m, None, na, true, here));
            }
        }
    }
    (out, rep)
}

/// One mark of any of the four kinds inside a town: what it is, and where.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MarkRef {
    Poi(u32),
    Label(usize),
    Icon(usize),
}

/// [`WorldGen::town_at`]'s centre (traced frame, cells) and longer box side (metres) for an unfrozen town at cell
/// `cell` laid round `origin`: the origin (the adapter's frame, so plus a half) with the box Ruling HM grows for it,
/// else the cell centre with the reference's box.
fn town_centre(cell: (usize, usize), origin: Option<cartalith_civ::urban_adapter::BankOrigin>) -> ((f64, f64), f64) {
    let c = origin.map_or((cell.0 as f64 + 0.5, cell.1 as f64 + 0.5), |b| (b.at.0 + 0.5, b.at.1 + 0.5));
    let long = cartalith_civ::urban_adapter::um_keep_river_box(origin).map_or(cartalith_civ::urban_adapter::SITE_WM, |b| b.0.max(b.1));
    (c, long)
}

impl WorldGen {
    /// Metres per grid cell (`world_crs`' `cell_km` times 1000); `0` before a world.
    fn town_cell_m(&self) -> f64 {
        if self.gw > 0 && self.map_width_km > 0.0 { self.map_width_km * 1000.0 / f64::from(self.gw) } else { 0.0 }
    }

    /// Grid point `(gx, gy)` in layout `l`'s local metres, the town drawn at settlement cell `(sx, sy)` (the map's own
    /// transform inverted, `city_viewer_window.gd::_poi_pins`).
    fn town_local(&self, l: &UrbanLayout, sx: usize, sy: usize, gx: f64, gy: f64) -> UVec2 {
        let mpc = self.town_cell_m();
        tf::from_town(l, UVec2 { x: (gx - sx as f64 - 0.5) * mpc, y: (gy - sy as f64 - 0.5) * mpc })
    }

    /// [`Self::town_local`] inverted: a local point of `l` as a grid point.
    pub(crate) fn town_grid(&self, l: &UrbanLayout, sx: usize, sy: usize, p: UVec2) -> (f64, f64) {
        let mpc = self.town_cell_m().max(1e-9);
        let t = tf::to_town(l, p);
        (sx as f64 + 0.5 + t.x / mpc, sy as f64 + 0.5 + t.y / mpc)
    }

    /// The settlement index whose town grid point `(gx, gy)` is in: the nearest settlement whose town's centre is
    /// within its radius. `None` outside every town.
    ///
    /// **The town's centre is where the town is drawn, not the cell** (Ruling HN leftover (e), 2026-11-23): a frozen
    /// town's saved box centre carried onto the map by its own pin, else the point an unfrozen town is laid round
    /// (`bank_site::town_origin`, Ruling HM), else the cell centre. Measured from the cell, a pin dropped on the far
    /// side of a town moved several hundred metres to its river's bank was outside the cell's radius and attached
    /// to nothing (the widest move in the HM census is 1 745 m against the 850 m radius). The radius is
    /// [`TOWN_RADIUS_M`], widened to half the longer side of a larger box: a frozen layout's own, or the box
    /// Ruling HM grows for a town on the drawn edge (`urban_adapter::um_keep_river_box`).
    pub(crate) fn town_at(&self, gx: f64, gy: f64) -> Option<usize> {
        let civ = self.civ.as_ref()?;
        let mpc = self.town_cell_m();
        if mpc <= 0.0 {
            return None;
        }
        civ.settlements
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let (c, long) = match self.town_layouts.rows.get(&s.tid).and_then(|f| f.layout.as_ref()) {
                    Some(l) => (self.town_grid(l, s.placement.x, s.placement.y, UVec2 { x: l.wm / 2.0, y: l.hm / 2.0 }), l.wm.max(l.hm)),
                    None => town_centre((s.placement.x, s.placement.y), self.town_origin(s)),
                };
                let d = (gx - c.0).hypot(gy - c.1) * mpc;
                (d <= TOWN_RADIUS_M.max(long / 2.0)).then_some((i, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
            .map(|(i, _)| i)
    }

    /// Lays the towns of `indices` out **now, from the current inputs** (never from a frozen row), with their recipe:
    /// what a freeze and a re-lay both start from.
    fn town_fresh(&self, indices: &[usize]) -> Vec<(usize, Option<(UrbanLayout, Value)>)> {
        self.town_batch(indices, |_, s, ctx, rules, o, kind_map| {
            let e = self.civ.as_ref().map(|c| c.place_extras.get(s.tid)).unwrap_or_default();
            self.stretch_layout(ctx, rules).map(|l| (l, recipe(ctx, o, kind_map, s, &e, rules)))
        })
    }

    /// Today's recipe for settlement `idx` without laying it out (the cheap tier: builds the context only).
    fn town_recipe_now(&self, idx: usize) -> Option<Value> {
        self.town_batch(&[idx], |_, s, ctx, rules, o, kind_map| {
            let e = self.civ.as_ref().map(|c| c.place_extras.get(s.tid)).unwrap_or_default();
            recipe(ctx, o, kind_map, s, &e, rules)
        })
        .pop()
        .map(|(_, r)| r)
    }

    /// Installs a table (undo, redo, load, every change) and counts the change for the shell (`town_layouts_rev`).
    pub(crate) fn town_install(&mut self, t: Arc<TownTable>) {
        if !Arc::ptr_eq(&t, &self.town_layouts) {
            self.town_layouts = t;
            self.town_rev = self.town_rev.wrapping_add(1);
        }
    }

    /// The table, writable: clones it if a snapshot shares it. Does **not** count the change: only a change to what
    /// is drawn (a freeze, a re-lay, an install) moves `town_rev`, since the shell answers a move by forgetting every
    /// town it drew, and an anchor update draws nothing new.
    fn towns_mut(&mut self) -> &mut TownTable {
        Arc::make_mut(&mut self.town_layouts)
    }

    /// Every POI, label and icon inside the town of settlement `idx`, with its grid point.
    fn town_marks(&self, idx: usize) -> Vec<(MarkRef, f64, f64)> {
        let mut out = Vec::new();
        for p in &self.pois {
            if self.town_at(p.x, p.y) == Some(idx) {
                out.push((MarkRef::Poi(p.id), p.x, p.y));
            }
        }
        if let Some(l) = self.labels.as_ref() {
            for (i, lb) in l.labels.iter().enumerate() {
                if self.town_at(lb.x, lb.y) == Some(idx) {
                    out.push((MarkRef::Label(i), lb.x, lb.y));
                }
            }
        }
        if let Some(ic) = self.icons.as_ref() {
            for (i, c) in ic.icons.iter().enumerate() {
                if self.town_at(c.x, c.y) == Some(idx) {
                    out.push((MarkRef::Icon(i), c.x, c.y));
                }
            }
        }
        out
    }

    /// Freezes the town of settlement `idx` from its current inputs (scope §5) and anchors every POI already inside it.
    /// `false` when it is already frozen, when the slot is held verbatim (a newer document: nothing can be added), when
    /// its row is damaged (never replaced by an automatic placement, scope §7; `force` is the user's explicit freeze),
    /// or when the town has no layout (a settlement in open water).
    pub(crate) fn town_freeze_index(&mut self, idx: usize, manual: bool, force: bool) -> bool {
        let Some(civ) = self.civ.as_ref() else { return false };
        let Some(s) = civ.settlements.get(idx) else { return false };
        let (tid, (sx, sy), pop, year) = (s.tid, (s.placement.x, s.placement.y), i64::from(s.pop), civ.year);
        if self.town_layouts.verbatim.is_some() {
            return false;
        }
        if let Some(r) = self.town_layouts.rows.get(&tid) {
            if r.problem.is_none() || !force {
                return false;
            }
        }
        let Some((_, Some((layout, recipe)))) = self.town_fresh(&[idx]).pop() else { return false };
        let geom = tf::encode(&layout);
        // The served layout is the decoded one, so what is drawn after the freeze is exactly what a reopened save draws.
        let Ok(decoded) = tf::decode(&geom) else { return false };
        let row = make_row(tid, year, pop, recipe, &geom, manual);
        let mut town = FrozenTown { id: tid, row, layout: Some(Arc::new(decoded)), problem: None, anchors: BTreeMap::new() };
        let l = Arc::clone(town.layout.as_ref().expect("just set"));
        for p in &self.pois {
            if self.town_at(p.x, p.y) == Some(idx) {
                let a = tf::choose_anchor(&l, self.town_local(&l, sx, sy, p.x, p.y));
                town.anchors.insert(p.id, StoredAnchor { a, at: grid_at(p.x, p.y), detached: false });
            }
        }
        self.towns_mut().rows.insert(tid, Arc::new(town));
        self.town_rev = self.town_rev.wrapping_add(1);
        true
    }

    /// TL-3's trigger, called by `anno_scope_multi` for every store an edit changed: each POI, label or icon that is new
    /// or moved in `post` against `pre` freezes the town it landed in (any settlement, Ruling HA Q2) or, in a town
    /// already frozen, anchors on its frozen layout. A POI moved out of a town loses that town's anchor. Never runs for
    /// undo, redo or a load (they do not go through the scope), nor inside a re-lay.
    pub(crate) fn town_on_marks(&mut self, pre: &AnnoSnap, post: &AnnoSnap) {
        let mut placed: Vec<(Option<u32>, f64, f64)> = Vec::new();
        match (pre, post) {
            (AnnoSnap::Pois(a), AnnoSnap::Pois(b)) => {
                for p in b {
                    if !a.iter().any(|q| q.id == p.id && q.x == p.x && q.y == p.y) {
                        placed.push((Some(p.id), p.x, p.y));
                    }
                }
            }
            (AnnoSnap::Labels(a, _), AnnoSnap::Labels(b, _)) => {
                placed.extend(b.iter().filter(|l| !a.iter().any(|q| q.x == l.x && q.y == l.y)).map(|l| (None, l.x, l.y)));
            }
            (AnnoSnap::Icons(a), AnnoSnap::Icons(b)) => {
                placed.extend(b.iter().filter(|l| !a.iter().any(|q| q.x == l.x && q.y == l.y)).map(|l| (None, l.x, l.y)));
            }
            _ => return,
        }
        for (poi, gx, gy) in placed {
            if let Some(id) = poi {
                // Moved: drop its anchor in any other town first.
                let holders: Vec<u64> = self.town_layouts.rows.iter().filter(|(_, r)| r.anchors.contains_key(&id)).map(|(k, _)| *k).collect();
                for k in holders {
                    if let Some(r) = self.towns_mut().rows.get_mut(&k) {
                        Arc::make_mut(r).anchors.remove(&id);
                    }
                }
            }
            let Some(idx) = self.town_at(gx, gy) else { continue };
            let Some((tid, sx, sy)) = self.civ.as_ref().and_then(|c| c.settlements.get(idx)).map(|s| (s.tid, s.placement.x, s.placement.y)) else { continue };
            if !self.town_layouts.rows.contains_key(&tid) {
                self.town_freeze_index(idx, false, false);
                continue; // the freeze anchored every POI inside, this one included
            }
            let Some(id) = poi else { continue };
            let Some(l) = self.town_layouts.rows.get(&tid).and_then(|r| r.layout.clone()) else { continue };
            let a = tf::choose_anchor(&l, self.town_local(&l, sx, sy, gx, gy));
            if let Some(r) = self.towns_mut().rows.get_mut(&tid) {
                Arc::make_mut(r).anchors.insert(id, StoredAnchor { a, at: grid_at(gx, gy), detached: false });
            }
        }
    }

    /// The frozen town of settlement `idx` (id `tid`) as `urban_layouts` serves it: `layout_dict` of the frozen layout
    /// plus the frozen keys. `None` when the town is not frozen or its row is not usable (then it is generated).
    pub(crate) fn town_frozen_dict(&self, idx: usize, tid: u64) -> Option<VarDictionary> {
        let r = self.town_layouts.rows.get(&tid)?;
        let l = r.layout.as_ref()?;
        let mut d = crate::urban_bridge::layout_dict(idx as i64, l);
        d.set("frozen", true);
        let (year, pop) = r.frozen_at();
        if let Some(y) = year {
            d.set("frozen_year", y);
        }
        if let Some(p) = pop {
            d.set("frozen_pop", p);
        }
        if let Some(v) = r.rules_version() {
            d.set("frozen_rules_version", v);
        }
        Some(d)
    }

    /// The re-lay of settlement `idx` (scope §4.4): today's layout and recipe, the moves every mark inside the town
    /// would make, and the report. Pure: changes nothing. `None` when the town is not frozen and usable or no longer
    /// lays out.
    #[allow(clippy::type_complexity)]
    fn town_relay_plan(&self, idx: usize) -> Option<(Arc<UrbanLayout>, Arc<UrbanLayout>, Value, Vec<(MarkRef, Option<(f64, f64)>, StoredAnchor)>, RelayReport)> {
        let civ = self.civ.as_ref()?;
        let s = civ.settlements.get(idx)?;
        let (sx, sy) = (s.placement.x, s.placement.y);
        let old = self.town_layouts.rows.get(&s.tid)?.layout.clone()?;
        let (_, Some((fresh, recipe))) = self.town_fresh(&[idx]).pop()? else { return None };
        let new = Arc::new(tf::decode(&tf::encode(&fresh)).ok()?);
        let stored = &self.town_layouts.rows.get(&s.tid)?.anchors;
        let mut marks = Vec::new();
        for (m, gx, gy) in self.town_marks(idx) {
            // A POI's stored anchor, while the pin is still where it was anchored; otherwise its anchor on the frozen
            // layout is computed from where it stands (labels, icons and a moved pin: Ruling HA's Q6).
            let here = grid_at(gx, gy);
            let a = match m {
                MarkRef::Poi(id) => stored.get(&id).filter(|s| s.at == here).map(|s| s.a.clone()),
                _ => None,
            }
            .unwrap_or_else(|| tf::choose_anchor(&old, self.town_local(&old, sx, sy, gx, gy)));
            marks.push((m, a, self.town_local(&new, sx, sy, gx, gy), here));
        }
        let (planned, mut rep) = plan_moves(&old, &new, marks, |m| self.mark_name(m));
        let moves = planned
            .into_iter()
            .map(|(m, to, a, detached, here)| match to {
                Some(p) => {
                    let (x, y) = self.town_grid(&new, sx, sy, p);
                    (m, Some((x, y)), StoredAnchor { a, at: grid_at(x, y), detached })
                }
                None => (m, None, StoredAnchor { a, at: here, detached }),
            })
            .collect();
        rep.changed_share = changed_share(&old, &new);
        Some((old, new, recipe, moves, rep))
    }

    /// What a report calls mark `m`: a POI's name (else its kind), a label's text, an icon's slot, and its noun.
    fn mark_name(&self, m: MarkRef) -> (&'static str, String) {
        match m {
            MarkRef::Poi(id) => ("point of interest", self.pois.iter().find(|p| p.id == id).map(|p| if p.name.is_empty() { p.kind.clone() } else { p.name.clone() }).unwrap_or_default()),
            MarkRef::Label(i) => ("label", self.labels.as_ref().and_then(|l| l.labels.get(i)).map(|l| l.name.clone()).unwrap_or_default()),
            MarkRef::Icon(i) => ("icon", self.icons.as_ref().and_then(|c| c.icons.get(i)).map(|c| c.slot.clone()).unwrap_or_default()),
        }
    }

    /// Applies a re-lay plan inside one undo step (the caller's scope): the new frozen row, the moved marks and
    /// their anchors. A mark moved off the grid stays where it was.
    fn town_relay_commit(&mut self, idx: usize, new: &Arc<UrbanLayout>, recipe: Value, moves: Vec<(MarkRef, Option<(f64, f64)>, StoredAnchor)>) {
        let Some((tid, pop, year)) = self.civ.as_ref().and_then(|c| c.settlements.get(idx).map(|s| (s.tid, i64::from(s.pop), c.year))) else { return };
        let (gw, gh) = (f64::from(self.gw), f64::from(self.gh));
        let on_grid = |(x, y): (f64, f64)| x.is_finite() && y.is_finite() && x >= 0.0 && y >= 0.0 && x < gw && y < gh;
        let manual = self.town_layouts.rows.get(&tid).is_some_and(|r| r.row.get("manual").is_some());
        let geom = tf::encode(new);
        let mut town = FrozenTown { id: tid, row: make_row(tid, year, pop, recipe, &geom, manual), layout: Some(Arc::clone(new)), problem: None, anchors: BTreeMap::new() };
        for (m, to, a) in moves {
            let to = to.filter(|p| on_grid(*p));
            match m {
                MarkRef::Poi(id) => {
                    if let (Some((x, y)), Some(p)) = (to, self.pois.iter_mut().find(|p| p.id == id)) {
                        p.x = x;
                        p.y = y;
                    }
                    town.anchors.insert(id, a);
                }
                MarkRef::Label(i) => {
                    if let (Some((x, y)), Some(l)) = (to, self.labels.as_mut().and_then(|l| l.labels.get_mut(i))) {
                        l.x = x;
                        l.y = y;
                    }
                }
                MarkRef::Icon(i) => {
                    if let (Some((x, y)), Some(c)) = (to, self.icons.as_mut().and_then(|c| c.icons.get_mut(i))) {
                        c.x = x;
                        c.y = y;
                    }
                }
            }
        }
        self.towns_mut().rows.insert(tid, Arc::new(town));
        self.town_rev = self.town_rev.wrapping_add(1);
    }

    /// The live POI ids (what an anchor may name).
    pub(crate) fn town_live_pois(&self) -> Vec<u32> {
        self.pois.iter().map(|p| p.id).collect()
    }

    /// One line for the open-project report (scope §6): how many frozen towns were drawn by older rules or with
    /// switches that differ from today's parameters. Judged without laying anything out (the recorded rules version
    /// and switch set against today's; the ground under a town is checked when it is opened). `None` when every
    /// frozen town is current.
    pub(crate) fn town_open_report(&self) -> Option<String> {
        let now = self.town_param_switches();
        let n = self
            .town_layouts
            .rows
            .values()
            .filter(|r| r.layout.is_some())
            .filter(|r| r.rules_version() != Some(i64::from(tf::URBAN_RULES_VERSION)) || !tf::switch_diff(&r.switches(), &now).is_empty())
            .count();
        (n > 0).then(|| format!("{n} frozen town{} {} drawn by older rules or other town switches (open the town in the City Viewer to re-lay)", if n == 1 { "" } else { "s" }, if n == 1 { "was" } else { "were" }))
    }

    /// Today's switch set from the parameters (the open report's cheap check; a layout records what was in effect).
    fn town_param_switches(&self) -> BTreeMap<String, bool> {
        let p = &self.params;
        [
            ("urban_wall_hug", p.urban_wall_hug),
            ("urban_riverbank", p.urban_riverbank),
            ("urban_street_village", p.urban_street_village),
            ("urban_market_scale", p.urban_market_scale),
            ("urban_harbour_front", p.urban_harbour_front),
            ("urban_river_stretch", p.urban_river_stretch),
            ("urban_burgage", p.urban_burgage),
            ("urban_market_road_bank", p.urban_market_road_bank),
            ("urban_growth_water_widen", p.urban_growth_water_widen),
            ("urban_town_bridge_search", p.urban_town_bridge_search),
            ("urban_coastal_setback", p.urban_coastal_setback),
            ("urban_floodplain_setback", p.urban_floodplain_setback),
            ("urban_parallel_streets", p.urban_parallel_streets),
            ("urban_river_smooth", p.urban_river_smooth),
            ("urban_river_kind_map", p.urban_river_kind_map),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
    }

    /// The drift check of one town (scope §4.2, TL-5): the cheap tier's reasons, then a fresh layout's stored form
    /// against the frozen one: `"unchanged"`, `"rounding"` (same town within rounding) or `"changed"`.
    fn town_drift(&self, idx: usize) -> Option<(Vec<String>, &'static str)> {
        let s = self.civ.as_ref()?.settlements.get(idx)?;
        let r = self.town_layouts.rows.get(&s.tid)?;
        let old = r.layout.as_ref()?;
        let (_, Some((fresh, recipe))) = self.town_fresh(&[idx]).pop()? else { return None };
        let reasons = cheap_reasons(r.row.get("recipe").unwrap_or(&Value::Null), &recipe);
        let (a, b) = (tf::encode(old), tf::encode(&fresh));
        let verdict = if tf::layout_hash(&a) == tf::layout_hash(&b) {
            "unchanged"
        } else if tf::within_rounding(&a, &b) {
            "rounding"
        } else {
            "changed"
        };
        Some((reasons, verdict))
    }

    /// The display name of settlement `idx`.
    fn town_name(&self, idx: usize) -> String {
        self.civ.as_ref().and_then(|c| c.settlements.get(idx)).map(|s| s.name.clone()).unwrap_or_default()
    }

    /// The settlement index of every usable frozen row whose settlement exists, in id order; and how many rows have no
    /// settlement (kept verbatim, Ruling HA Q11).
    fn town_frozen_indices(&self) -> (Vec<usize>, usize) {
        let Some(civ) = self.civ.as_ref() else { return (Vec::new(), self.town_layouts.rows.len()) };
        let mut out = Vec::new();
        let mut gone = 0;
        for (tid, r) in &self.town_layouts.rows {
            match civ.settlements.iter().position(|s| s.tid == *tid) {
                Some(i) if r.layout.is_some() => out.push(i),
                Some(_) => {}
                None => gone += 1,
            }
        }
        (out, gone)
    }
}

/// A grid point as thousandths of a cell (the anchors' integer `at`).
fn grid_at(x: f64, y: f64) -> (i64, i64) {
    ((x * 1000.0).round() as i64, (y * 1000.0).round() as i64)
}

/// The share of `old`'s buildings with no building of `new` whose centre lies within 2 m of its own, in the town
/// frame (scope §5's "changed share of buildings"). `ponytail:` O(n x m); a capital is 5 587 x 5 587 point tests,
/// tens of milliseconds, once per preview.
fn changed_share(old: &UrbanLayout, new: &UrbanLayout) -> f64 {
    if old.buildings.is_empty() {
        return 0.0;
    }
    let c = |l: &UrbanLayout, p: &[UVec2]| {
        let n = p.len().max(1) as f64;
        tf::to_town(l, UVec2 { x: p.iter().map(|q| q.x).sum::<f64>() / n, y: p.iter().map(|q| q.y).sum::<f64>() / n })
    };
    let nc: Vec<UVec2> = new.buildings.iter().map(|b| c(new, &b.poly)).collect();
    let moved = old
        .buildings
        .iter()
        .filter(|b| {
            let p = c(old, &b.poly);
            !nc.iter().any(|q| (q.x - p.x).hypot(q.y - p.y) <= 2.0)
        })
        .count();
    moved as f64 / old.buildings.len() as f64
}

fn report_dict(rep: &RelayReport) -> VarDictionary {
    let det: Array<VarDictionary> = rep
        .detached
        .iter()
        .map(|d| {
            let mut x = VarDictionary::new();
            x.set("what", d.what);
            x.set("kind", d.kind.key());
            x.set("name", d.name.as_str());
            x
        })
        .collect();
    let mut d = VarDictionary::new();
    d.set("attached", rep.attached as i64);
    d.set("ground", rep.ground as i64);
    d.set("detached", &det);
    d.set("detached_count", rep.detached.len() as i64);
    d.set("changed_share", rep.changed_share);
    d
}

#[godot_api(secondary)]
impl WorldGen {
    /// The frozen-town state of settlement `index` for the City Viewer's badge, banner and chip (scope §6, TL-6/TL-7):
    /// `{frozen}` plus, for a frozen town, `manual`, `year`, `pop` (when frozen), `pop_now` (the settlement this year),
    /// `chip` (true when they differ by more than 2x either way, Ruling HA Q3), `rules_version`, `rules_version_now`,
    /// `anchors`, `detached` and `damaged` (the reason, for a row kept verbatim). With `check` it also builds today's
    /// context (no layout) and adds `reasons`, the cheap tier's words (scope §4.2). Empty for an index out of range.
    #[func]
    fn town_state(&self, index: i64, check: bool) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(civ) = self.civ.as_ref() else { return d };
        let Some(idx) = usize::try_from(index).ok().filter(|i| *i < civ.settlements.len()) else { return d };
        let s = &civ.settlements[idx];
        let Some(r) = self.town_layouts.rows.get(&s.tid) else {
            d.set("frozen", false);
            return d;
        };
        d.set("frozen", r.layout.is_some());
        if let Some(p) = &r.problem {
            d.set("damaged", p.as_str());
            return d;
        }
        d.set("manual", r.row.get("manual").is_some());
        let (year, pop) = r.frozen_at();
        if let Some(y) = year {
            d.set("year", y);
        }
        // The settlement in the year shown: the cursor year's recorded snapshot when it holds it (the timeline and
        // the recovery simulation record per-year populations), else the live list.
        let now = civ.timeline.iter().find(|t| t.year == civ.year).and_then(|t| t.settlements.iter().find(|x| x.tid == s.tid)).map_or(s.pop, |x| x.pop);
        d.set("pop_now", i64::from(now));
        if let Some(p) = pop {
            d.set("pop", p);
            d.set("chip", pop_chip(p, i64::from(now)));
        }
        if let Some(v) = r.rules_version() {
            d.set("rules_version", v);
        }
        d.set("rules_version_now", i64::from(tf::URBAN_RULES_VERSION));
        d.set("anchors", r.anchors.len() as i64);
        d.set("detached", r.anchors.values().filter(|a| a.detached).count() as i64);
        if check {
            if let Some(now) = self.town_recipe_now(idx) {
                let reasons: PackedStringArray = cheap_reasons(r.row.get("recipe").unwrap_or(&Value::Null), &now).iter().map(GString::from).collect();
                d.set("reasons", &reasons);
            }
        }
        d
    }

    /// "Freeze this town" (Ruling HA Q10): freezes settlement `index`'s town with no pin, from its current inputs, as
    /// one undo step. Also the user's explicit re-freeze of a damaged row (scope §7). `false` when it is already
    /// frozen, cannot be laid out, or the slot is held verbatim.
    #[func]
    fn town_freeze(&mut self, index: i64) -> bool {
        let Ok(idx) = usize::try_from(index) else { return false };
        let label = format!("Freeze town {}", self.town_name(idx));
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| s.town_freeze_index(idx, true, true))
    }

    /// Freezes the towns any of `points` (grid coordinates) stand in, as one undo step: a saved measurement inside a
    /// town anchors it too (Ruling HA Q6). Returns how many towns were frozen now.
    #[func]
    fn town_freeze_points(&mut self, points: PackedVector2Array) -> i64 {
        let idxs: std::collections::BTreeSet<usize> = points.as_slice().iter().filter_map(|p| self.town_at(f64::from(p.x), f64::from(p.y))).collect();
        self.anno_scope(AnnoStore::Pois, "Freeze town under a measurement".to_string(), None, None, |s| {
            idxs.iter().filter(|i| s.town_freeze_index(**i, false, false)).count() as i64
        })
    }

    /// Settlement index of the town grid point `(gx, gy)` stands in, or `-1`.
    #[func]
    fn town_at_point(&self, gx: f64, gy: f64) -> i64 {
        self.town_at(gx, gy).map_or(-1, |i| i as i64)
    }

    /// The drift check of settlement `index`'s frozen town (scope §4.2, TL-5): `{ok, verdict, reasons}`, `verdict`
    /// one of `unchanged`, `rounding` (same town within rounding) or `changed`. Lays the town out once (0.1-2.4 s on
    /// a capital); never on open. `{ok: false}` for a town not frozen.
    #[func]
    fn town_check(&self, index: i64) -> VarDictionary {
        let mut d = VarDictionary::new();
        match usize::try_from(index).ok().and_then(|i| self.town_drift(i)) {
            Some((reasons, verdict)) => {
                d.set("ok", true);
                d.set("verdict", verdict);
                let r: PackedStringArray = reasons.iter().map(GString::from).collect();
                d.set("reasons", &r);
            }
            None => d.set("ok", false),
        }
        d
    }

    /// "Check frozen towns" (Ruling HA Q7): the drift check of every frozen town, `{count, unchanged, rounding,
    /// changed, damaged, gone, rows: [{index, name, verdict, reasons}]}`. `gone` counts rows whose settlement no
    /// longer exists (kept, Q11).
    #[func]
    fn towns_check_all(&self) -> VarDictionary {
        let (idxs, gone) = self.town_frozen_indices();
        let mut counts = [0i64; 3];
        let rows: Array<VarDictionary> = idxs
            .iter()
            .filter_map(|i| self.town_drift(*i).map(|(r, v)| (*i, r, v)))
            .map(|(i, reasons, v)| {
                counts[match v {
                    "unchanged" => 0,
                    "rounding" => 1,
                    _ => 2,
                }] += 1;
                let mut x = VarDictionary::new();
                x.set("index", i as i64);
                x.set("name", self.town_name(i).as_str());
                x.set("verdict", v);
                let r: PackedStringArray = reasons.iter().map(GString::from).collect();
                x.set("reasons", &r);
                x
            })
            .collect();
        let mut d = VarDictionary::new();
        d.set("count", self.town_layouts.rows.len() as i64);
        d.set("unchanged", counts[0]);
        d.set("rounding", counts[1]);
        d.set("changed", counts[2]);
        d.set("damaged", self.town_layouts.rows.values().filter(|r| r.problem.is_some()).count() as i64);
        d.set("gone", gone as i64);
        d.set("rows", &rows);
        d
    }

    /// The before/after of "Re-lay with current rules" for settlement `index` (scope §5, §6): `{ok, before, after,
    /// report}`, `before` the frozen town and `after` today's, both as `urban_layouts` entries, and `report` the
    /// re-attach result (`attached`, `ground`, `detached` rows, `changed_share`). Changes nothing; the plan is kept for
    /// [`Self::town_relay_apply`] and [`Self::town_relay_map_points`].
    #[func]
    fn town_relay_preview(&self, index: i64) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(idx) = usize::try_from(index).ok() else { return d };
        let Some((old, new, _, _, rep)) = self.town_relay_plan(idx) else {
            d.set("ok", false);
            return d;
        };
        d.set("ok", true);
        d.set("before", &crate::urban_bridge::layout_dict(index, &old));
        d.set("after", &crate::urban_bridge::layout_dict(index, &new));
        d.set("report", &report_dict(&rep));
        d
    }

    /// Where each grid point of `points` would move under the re-lay of settlement `index` (the shell's saved
    /// measurements, which the engine does not hold, Ruling HA Q6): the same re-attach as a pin, a point that finds no
    /// match staying put. Empty when the town is not frozen.
    #[func]
    fn town_relay_map_points(&self, index: i64, points: PackedVector2Array) -> PackedVector2Array {
        let Some(idx) = usize::try_from(index).ok() else { return PackedVector2Array::new() };
        let Some((sx, sy)) = self.civ.as_ref().and_then(|c| c.settlements.get(idx)).map(|s| (s.placement.x, s.placement.y)) else { return PackedVector2Array::new() };
        let Some((old, new, _, _, _)) = self.town_relay_plan(idx) else { return PackedVector2Array::new() };
        points
            .as_slice()
            .iter()
            .map(|p| {
                let (gx, gy) = (f64::from(p.x), f64::from(p.y));
                let a = tf::choose_anchor(&old, self.town_local(&old, sx, sy, gx, gy));
                tf::reattach(&old, &a, &new)
                    .and_then(|na| tf::anchor_point(&new, &na))
                    .map_or(*p, |q| {
                        let (x, y) = self.town_grid(&new, sx, sy, q);
                        Vector2::new(x as f32, y as f32)
                    })
            })
            .collect()
    }

    /// "Re-lay with current rules" (scope §5): unfreezes settlement `index`'s town, lays it out from today's inputs,
    /// re-attaches every POI, label and icon inside it, and freezes it again, **as one undo step**. A pin that finds no
    /// match stays exactly where it is and is listed detached; nothing is deleted. Returns the report (as
    /// [`Self::town_relay_preview`]'s) with `ok`.
    #[func]
    fn town_relay_apply(&mut self, index: i64) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(idx) = usize::try_from(index).ok() else { return d };
        let Some((_, new, recipe, moves, rep)) = self.town_relay_plan(idx) else {
            d.set("ok", false);
            return d;
        };
        let label = format!("Re-lay town {} with current rules", self.town_name(idx));
        self.town_relaying = true;
        self.anno_scope_multi(&[AnnoStore::Pois, AnnoStore::Labels, AnnoStore::Icons], label, None, None, |s| s.town_relay_commit(idx, &new, recipe, moves));
        self.town_relaying = false;
        d = report_dict(&rep);
        d.set("ok", true);
        d
    }

    /// "Re-lay all frozen towns" (Ruling HA Q7): every usable frozen town whose settlement exists, re-laid with
    /// today's rules as **one** undo step. `{ok, towns, attached, ground, detached_count, detached}` pooled.
    #[func]
    fn towns_relay_all(&mut self) -> VarDictionary {
        let (idxs, _) = self.town_frozen_indices();
        let plans: Vec<_> = idxs.iter().filter_map(|i| self.town_relay_plan(*i).map(|p| (*i, p))).collect();
        let mut pooled = RelayReport::default();
        self.town_relaying = true;
        let n = plans.len();
        self.anno_scope_multi(&[AnnoStore::Pois, AnnoStore::Labels, AnnoStore::Icons], format!("Re-lay {n} frozen towns"), None, None, |s| {
            for (i, (_, new, recipe, moves, rep)) in plans {
                s.town_relay_commit(i, &new, recipe, moves);
                pooled.attached += rep.attached;
                pooled.ground += rep.ground;
                pooled.detached.extend(rep.detached);
            }
        });
        self.town_relaying = false;
        let mut d = report_dict(&pooled);
        d.set("ok", true);
        d.set("towns", n as i64);
        d
    }

    /// A counter that moves whenever the frozen-town table changes (a freeze, a re-lay, an undo or redo of one, a load),
    /// so the shell knows to forget the towns it drew (`engine_bridge.gd` polls it).
    #[func]
    fn town_layouts_rev(&self) -> i64 {
        self.town_rev as i64
    }

    /// How many towns are frozen (usable rows).
    #[func]
    fn town_frozen_count(&self) -> i64 {
        self.town_layouts.rows.values().filter(|r| r.layout.is_some()).count() as i64
    }
}

/// Ruling HA Q3's chip: the frozen population and this year's differ by [`POP_CHIP_RATIO`] or more either way
/// (TL-7's bar: it shows at 2x and not at 1.9x).
/// A zero on either side is a gap of its own (a settlement not yet grown): it shows.
pub(crate) fn pop_chip(frozen: i64, now: i64) -> bool {
    let (a, b) = (frozen.max(0) as f64, now.max(0) as f64);
    if a == 0.0 || b == 0.0 {
        return a != b;
    }
    a / b >= POP_CHIP_RATIO || b / a >= POP_CHIP_RATIO
}

#[cfg(test)]
mod tests;
