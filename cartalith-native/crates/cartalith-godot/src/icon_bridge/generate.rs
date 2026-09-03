//! CARTO ▸ Icons — the generated placement pass's Godot surface.
//!
//! Owner ruling 2026-09-02: *"Build, and add a sea-marks asset family"* — a
//! generated placement pass **plus** a fourth family in `cartalith-assets`, so
//! the ICONS panel's `SEA MARKS` chip and its *snap sea marks to coast* rule
//! stop being drawn-and-disabled and start doing something.
//!
//! The algorithm is next door in [`crate::icon_bridge`]
//! ([`IconEditor::generate`], [`PlacementFamily`], [`CoastSnap`]) and is
//! exercised without a Godot runtime; the coastline itself is in
//! `cartalith_assets::coast`. This file is only the `Variant` conversion over
//! them plus the one thing that cannot live in either: assembling **candidates**
//! out of `WorldGen`'s own fields. That is `label_bridge/generate.rs`'s split,
//! for its reason — a candidate sweep reads `self.civ`, `self.landmark_store`
//! and the height field, and none of those exist below this crate.
//!
//! ## Where a candidate comes from, per family
//!
//! Four families, four different sources, and no source is shared with
//! another — which is the point of the ruling. Each is something the engine
//! already computes; none of them is invented here.
//!
//! - **PLACES** — `civ.settlements`, one candidate per settlement, slot from
//!   its own [`SettlementKind`]. Nothing is placed before the civilisation
//!   layer has run.
//! - **TREES** — `cartalith_assets::place_map_icons_ruled`, the golden-verified
//!   ruled scatter engine, driven by the five `tree_*` slots' own
//!   [`preset_scatter_rule`]s. Needs the biome raster, so it needs `civ` too.
//! - **SEA MARKS** — a jittered-free coarse sweep of the **water**, every
//!   [`sea_mark_gap`] cells. Deliberately the open sea rather than the
//!   shoreline: if the sweep only ever picked coast cells, *"snap sea marks to
//!   coast"* would be a toggle with nothing to do. With the rule off, marks sit
//!   where the sweep found them; with it on, each is pulled to the nearest
//!   coast cell and one too far from any shore is dropped.
//! - **POI** — the landmark store, slot from the landmark's own kind (see
//!   [`poi_slot_for_landmark`], which maps three names and sends the rest to
//!   `other` rather than inventing forty correspondences).
//!
//! ## What is *not* here
//!
//! No new list, no `generated` flag: the pass appends into the one icon list
//! and is idempotent because its own output culls its own candidates —
//! [`IconEditor::generate`]'s doc says why that differs from the label pass.
//! So `icon_list`, `icon_delete` and `icon_clear_all` in `lib.rs` keep working
//! unchanged, and "undo a generated run" is the Clear-all that already exists.

use godot::prelude::*;

use cartalith_assets::manual::IconViewEnv;
use cartalith_assets::{
    icon_slot_for_item, place_map_icons_ruled, preset_scatter_rule, PlaceIconsRuledOpts,
    ScatterRule, PACK_POI_SLOTS, PACK_SEAMARK_SLOTS,
};
use cartalith_civ::labels::LabelRect;
use cartalith_civ::SettlementKind;

use super::{
    default_snap_radius, CoastSnap, IconCandidate, IconGenPlan, IconGenReport, PlacementFamily,
    TREE_SLOTS,
};
use crate::WorldGen;

/// The slot a settlement of this tier draws its pin from —
/// `PACK_SETTLEMENT_SLOTS`, which the reference's own comment says *"mirror the
/// civ layer's `CIV_SETTLEMENT_CLASSES` keys exactly"*, so five of the six
/// tiers are an identity.
///
/// [`SettlementKind::Metropolis`] is the exception and is mapped to `capital`:
/// it is this port's own v0.75 rank-5 promotion of a capital
/// (`civ_select_metropolises`) and the frozen pack vocabulary has no sixth
/// entry to grow. A pack cannot author a metropolis pin, so it gets the capital
/// one rather than nothing.
fn settlement_slot(kind: SettlementKind) -> &'static str {
    match kind {
        SettlementKind::Hamlet => "hamlet",
        SettlementKind::Village => "village",
        SettlementKind::Town => "town",
        SettlementKind::City => "city",
        SettlementKind::Capital | SettlementKind::Metropolis => "capital",
    }
}

/// The POI slot a landmark of `kind` draws from.
///
/// `cartalith_civ::landmark::kinds()` is a 40-plus vocabulary and
/// [`PACK_POI_SLOTS`] is eight, so most of it has no counterpart — and
/// `other` is in the frozen list for exactly that. The rule is: an identity
/// where the two vocabularies already agree on a word, three renames where they
/// clearly mean the same thing, `other` for everything else. Inventing a
/// mapping for the remaining thirty-odd is the mistake this whole lane exists
/// to undo.
fn poi_slot_for_landmark(kind: &str) -> &'static str {
    if let Some(s) = PACK_POI_SLOTS.iter().find(|s| **s == kind) {
        return s;
    }
    match kind {
        "peak" => "mountain_peak",
        "ancient_forest" => "named_forest",
        "ruins" | "ruined_settlement" => "ruin",
        _ => "other",
    }
}

/// The sea-mark sweep's step, in cells: `max(8, gw / 32)`.
///
/// Deliberately the same figure as [`default_snap_radius`]: a sweep coarser
/// than the snap radius would leave stretches of coastline with no sample close
/// enough to reach them, and a finer one just hands the spacing culler more
/// candidates to throw away — at the cost of a ring search each.
pub fn sea_mark_gap(gw: usize) -> usize {
    (gw / 32).max(8)
}

fn as_f64(d: &VarDictionary, key: &str) -> Option<f64> {
    d.get(key).and_then(|v| v.try_to::<f64>().ok()).filter(|f| f.is_finite())
}

fn as_bool(d: &VarDictionary, key: &str, default: bool) -> bool {
    d.get(key).and_then(|v| v.try_to::<bool>().ok()).unwrap_or(default)
}

fn report_dict(r: &IconGenReport, elapsed_ms: i64) -> VarDictionary {
    vdict! {
        "ok" => true,
        "placed" => r.placed as i64,
        "culled_spacing" => r.culled_spacing as i64,
        "culled_label" => r.culled_label as i64,
        "off_coast" => r.off_coast as i64,
        "unknown_slot" => r.unknown_slot as i64,
        "snapped" => r.snapped as i64,
        "elapsed_ms" => elapsed_ms,
    }
}

fn refused(reason: &str) -> VarDictionary {
    vdict! {
        "ok" => false,
        "reason" => reason,
        "placed" => 0i64,
        "culled_spacing" => 0i64,
        "culled_label" => 0i64,
        "off_coast" => 0i64,
        "unknown_slot" => 0i64,
        "snapped" => 0i64,
        "elapsed_ms" => 0i64,
    }
}

impl WorldGen {
    /// The height field of whatever world is loaded, generated or restored.
    fn icon_gen_field(&self) -> Option<&[f32]> {
        match self.source.as_ref()? {
            crate::WorldSource::Generated(ws) => Some(&ws.field),
            crate::WorldSource::Loaded(save) => Some(&save.fields.heightmap),
        }
    }

    /// One family's candidates, in a deterministic order.
    ///
    /// Order matters and is not incidental: the pass takes candidates
    /// first-come and culls what collides, so this order is what decides which
    /// of two neighbours survives. Every source here iterates its own stored
    /// order (settlements as placed, landmarks as ranked, the sweep row by
    /// row), all of which are already deterministic per world.
    fn icon_candidates(&self, family: PlacementFamily) -> Vec<IconCandidate> {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return Vec::new();
        }
        match family {
            PlacementFamily::Places => self
                .civ
                .as_ref()
                .map(|c| {
                    c.settlements
                        .iter()
                        .map(|s| IconCandidate {
                            x: s.placement.x as f64,
                            y: s.placement.y as f64,
                            slot: settlement_slot(s.placement.kind).to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default(),

            PlacementFamily::Poi => self
                .landmark_store
                .last
                .as_ref()
                .map(|r| {
                    r.landmarks
                        .iter()
                        .map(|l| IconCandidate {
                            x: l.x as f64,
                            y: l.y as f64,
                            slot: poi_slot_for_landmark(&l.kind).to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default(),

            PlacementFamily::Trees => {
                // The ruled scatter engine, with the five tree slots' own
                // preset rules — the same rules `AssetDB::new` bootstraps a
                // library's `icons` slots with. Needs the biome raster, which
                // needs the civilisation layer's water bodies.
                let (Some(ws), Some(civ)) = (
                    match self.source.as_ref() {
                        Some(crate::WorldSource::Generated(ws)) => Some(ws),
                        _ => None,
                    },
                    self.civ.as_ref(),
                ) else {
                    return Vec::new();
                };
                let biome = cartalith_civ::build_biome_raster(
                    &civ.water_bodies,
                    &ws.temperature,
                    &ws.rainfall,
                );
                let rules: Vec<(&str, ScatterRule)> =
                    TREE_SLOTS.iter().map(|s| (*s, preset_scatter_rule(s))).collect();
                let refs: Vec<(&str, &ScatterRule)> =
                    rules.iter().map(|(k, r)| (*k, r)).collect();
                let fld64: Vec<f64> = ws.field.iter().map(|&v| v as f64).collect();
                let mut opts = PlaceIconsRuledOpts::new(gw, &refs);
                opts.sea = self.sea_level;
                opts.seed = self.seed;
                place_map_icons_ruled(&fld64, Some(&biome), gw, gh, &opts)
                    .into_iter()
                    .map(|it| IconCandidate {
                        x: it.x as f64,
                        y: it.y as f64,
                        slot: icon_slot_for_item(&it),
                    })
                    .collect()
            }

            PlacementFamily::SeaMarks => {
                let Some(field) = self.icon_gen_field() else { return Vec::new() };
                let sea = self.sea_level;
                let gap = sea_mark_gap(gw);
                let mut out = Vec::new();
                for y in (0..gh).step_by(gap) {
                    for x in (0..gw).step_by(gap) {
                        if !cartalith_assets::coast::is_water(field, gw, gh, sea, x as i64, y as i64)
                        {
                            continue;
                        }
                        // Which of the eight marks this cell carries — the same
                        // positional hash `pick_weighted_variant` uses to choose
                        // a sprite, so two runs over one world agree and two
                        // adjacent marks differ.
                        let v = cartalith_assets::pick_icon_variant(
                            x as i32,
                            y as i32,
                            self.seed,
                            PACK_SEAMARK_SLOTS.len(),
                        );
                        out.push(IconCandidate {
                            x: x as f64,
                            y: y as f64,
                            slot: PACK_SEAMARK_SLOTS[v].to_string(),
                        });
                    }
                }
                out
            }
        }
    }

    /// Every hand-placed label's box, measured with the live typography table —
    /// `label_bridge::LabelBridge::reserved_rects`' own measurement, plus the
    /// generated labels, which are just as much on the map.
    ///
    /// Empty when there is no label bridge yet. Reading it here rather than
    /// asking the caller for rectangles is deliberate: the shell would have to
    /// re-measure what the engine already knows, and the two measurements would
    /// drift the first time a class's tracking changed.
    fn icon_reserved_label_rects(&self) -> Vec<LabelRect> {
        let Some(lb) = self.labels.as_ref() else { return Vec::new() };
        let m = lb.gen_settings.cull.unwrap_or_default();
        lb.render_order()
            .filter(|(l, _)| !l.name.is_empty())
            .map(|(l, _)| {
                cartalith_civ::labels::label_cull_rect(l, lb.typography[l.class.index()].tracking, &m)
            })
            .collect()
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The four placement families the ICONS panel draws chips for, each with
    /// the vocabulary it places from and how much of it the loaded pack fills.
    ///
    /// Available before any `generate()` — it is a design table, not world
    /// data. Rows carry `key` (the design's own id, e.g. `"SEA MARKS"`),
    /// `family` (the asset family behind it), `slots` (how many slots that
    /// vocabulary has) and `filled` (how many of them the loaded pack has art
    /// for; `0` with no pack, which is not an error — every slot falls back to
    /// its procedural glyph, the format's own per-slot rule).
    ///
    /// This replaces the panel's transcribed `FAM` table. The design's own
    /// figures were `[filled, total]` pairs describing *its* art, and the panel
    /// had to label them "(design figures)" because nothing in the engine could
    /// answer the question; now something can.
    #[func]
    fn icon_placement_families(&self) -> Array<VarDictionary> {
        PlacementFamily::ALL
            .into_iter()
            .map(|f| {
                let slots = f.slots();
                let fam = f.icon_family().pack_family();
                let filled = self.asset_pack.as_ref().map_or(0, |p| {
                    slots.iter().filter(|s| p.manifest.slot_paths(fam, s).is_some()).count()
                });
                vdict! {
                    "key" => f.key(),
                    "family" => f.icon_family().key(),
                    "slots" => slots.len() as i64,
                    "filled" => filled as i64,
                }
            })
            .collect()
    }

    /// Run the generated placement pass for one family and append what survives
    /// to the icon list.
    ///
    /// `options`, every key optional:
    /// - `family` — one of [`PlacementFamily::key`]; defaults to `"PLACES"`.
    /// - `scale` — per-instance size, clamped to the same bounds `icon_arm`
    ///   uses.
    /// - `min_spacing` — minimum centre-to-centre separation **in grid cells**.
    /// - `avoid_labels` / `enforce_spacing` / `snap_coast` — the panel's three
    ///   rule toggles, defaulting to the design's own initial state (the first
    ///   two on, the snap off — `cartalith-dcc-parts.js:398`).
    ///
    /// Returns `{ok, placed, culled_spacing, culled_label, off_coast,
    /// unknown_slot, snapped, elapsed_ms}`, or `{ok: false, reason}` before any
    /// `generate()`. **Every counter is a real number**, and `placed: 0` with a
    /// non-zero `off_coast` is a different and more useful answer than a
    /// refusal: it says the sweep found sea but no shore within reach.
    ///
    /// Running it twice over one world places nothing the second time — see
    /// [`IconEditor::generate`].
    #[func]
    fn icon_generate(&mut self, options: VarDictionary) -> VarDictionary {
        if self.icons.is_none() {
            return refused(
                "Automatic placement needs a generated world: it places on settlements, biomes, \
                 landmarks and coastline, none of which exist before generate().",
            );
        }
        let key = options
            .get("family")
            .and_then(|v| v.try_to::<GString>().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| PlacementFamily::Places.key().to_string());
        let Some(family) = PlacementFamily::from_key(&key) else {
            return refused(&format!("Unknown placement family: {key}"));
        };

        let plan = IconGenPlan {
            family,
            scale: as_f64(&options, "scale").unwrap_or(1.0),
            min_spacing: as_f64(&options, "min_spacing").unwrap_or(0.0),
            avoid_labels: as_bool(&options, "avoid_labels", true),
            enforce_spacing: as_bool(&options, "enforce_spacing", true),
        };

        let t0 = std::time::Instant::now();
        // Everything read off `self` is gathered before the icon editor is
        // borrowed mutably -- `label_bridge/generate.rs`'s own "sweep, then
        // place" ordering, for the same borrow reason.
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let candidates = self.icon_candidates(family);
        let reserved = if plan.avoid_labels { self.icon_reserved_label_rects() } else { Vec::new() };
        let want_snap = as_bool(&options, "snap_coast", false) && family == PlacementFamily::SeaMarks;
        let sea = self.sea_level;
        let env = IconViewEnv { grid_w: gw, ..Default::default() };

        // The height field is **borrowed**, not copied. `icon_gen_field` takes
        // `&self` and so would hold the whole struct borrowed against
        // `self.icons.as_mut()` below; matching on `self.source` here borrows
        // one field, and the borrow checker allows that beside a mutable borrow
        // of a different one. The alternative was `to_vec()`, which on a 4096²
        // world is a 64 MB copy per button press.
        let field: Option<&[f32]> = if want_snap {
            match self.source.as_ref() {
                Some(crate::WorldSource::Generated(ws)) => Some(&ws.field),
                Some(crate::WorldSource::Loaded(save)) => Some(&save.fields.heightmap),
                None => None,
            }
        } else {
            None
        };
        let snap = field.map(|f| CoastSnap { field: f, gw, gh, sea, max_r: default_snap_radius(gw) });

        let editor = self.icons.as_mut().expect("checked above");
        let report = editor.generate(&plan, &candidates, &env, &reserved, snap.as_ref());
        report_dict(&report, t0.elapsed().as_millis() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_slots_are_the_frozen_vocabulary_and_metropolis_borrows_capitals() {
        for kind in [
            SettlementKind::Hamlet,
            SettlementKind::Village,
            SettlementKind::Town,
            SettlementKind::City,
            SettlementKind::Capital,
            SettlementKind::Metropolis,
        ] {
            let slot = settlement_slot(kind);
            assert!(
                cartalith_assets::PACK_SETTLEMENT_SLOTS.contains(&slot),
                "{slot} is not in the frozen settlement vocabulary"
            );
        }
        assert_eq!(settlement_slot(SettlementKind::Metropolis), "capital");
        assert_eq!(settlement_slot(SettlementKind::Hamlet), "hamlet");
    }

    #[test]
    fn every_landmark_kind_resolves_to_a_real_poi_slot() {
        // The 40-plus landmark vocabulary against the frozen eight: no panic,
        // no invented slot, and the ones with no counterpart land on `other`
        // rather than on whichever slot happened to sound similar.
        for spec in cartalith_civ::landmark::kinds() {
            let slot = poi_slot_for_landmark(spec.key);
            assert!(PACK_POI_SLOTS.contains(&slot), "{} -> {slot} is not a POI slot", spec.key);
        }
        assert_eq!(poi_slot_for_landmark("peak"), "mountain_peak");
        assert_eq!(poi_slot_for_landmark("cave"), "cave");
        assert_eq!(poi_slot_for_landmark("ancient_forest"), "named_forest");
        assert_eq!(poi_slot_for_landmark("mountain_pass"), "other");
        assert_eq!(poi_slot_for_landmark(""), "other");
    }

    #[test]
    fn the_sweep_step_never_outruns_the_snap_radius() {
        // A sweep coarser than the search radius leaves coastline no sample can
        // reach. Asserted across the resolutions this port actually ships.
        for gw in [64usize, 256, 512, 1024, 2048, 4096] {
            assert!(
                sea_mark_gap(gw) as i64 <= default_snap_radius(gw),
                "gw {gw}: gap {} outruns radius {}",
                sea_mark_gap(gw),
                default_snap_radius(gw)
            );
        }
    }

    #[test]
    fn every_candidate_slot_a_family_can_produce_is_one_that_family_accepts() {
        // The pass counts a foreign slot as `unknown_slot` rather than placing
        // it, so a source that emitted one would silently place nothing. Pin
        // the two hand-written mappings against the families they feed.
        for kind in [SettlementKind::Hamlet, SettlementKind::Capital, SettlementKind::Metropolis] {
            assert!(PlacementFamily::Places.slots().contains(&settlement_slot(kind)));
        }
        for spec in cartalith_civ::landmark::kinds() {
            assert!(PlacementFamily::Poi.slots().contains(&poi_slot_for_landmark(spec.key)));
        }
        for s in PACK_SEAMARK_SLOTS {
            assert!(PlacementFamily::SeaMarks.slots().contains(&s));
        }
        // And the ruled engine's own slot spelling for the five tree rules.
        for s in TREE_SLOTS {
            assert!(PlacementFamily::Trees.slots().contains(&s));
        }
    }
}
