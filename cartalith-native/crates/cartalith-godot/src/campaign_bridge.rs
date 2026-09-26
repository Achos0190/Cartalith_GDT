//! Ruling AW -- the CARTO ▸ Conflict layer's data, over
//! [`cartalith_civ::campaign`] (`MILITARY_MANPOWER_SCOPE.md` §5).
//!
//! Its own file with a `#[godot_api(secondary)]` block, like
//! `conflict_bridge.rs`: one read-only query, no state of its own. It reads
//! the SP-4 store (`WorldGen::conflicts`), the recorded timeline and the map
//! scale; it writes nothing.

use cartalith_civ::campaign::{campaigns_at, edge_segment};
use godot::prelude::*;

use crate::WorldGen;
use crate::conflict_bridge::anchor_pos;

#[godot_api(secondary)]
impl WorldGen {
    /// Every conflict active in `year`, read for the Conflict layer. One row
    /// each, in creation order:
    ///
    /// - `id`, `name`, `kind`, `sides` (`PackedInt32Array`);
    /// - `territory_year` -- the recorded year whose claims are read (the
    ///   latest at or before `year`); `baseline_year` -- the one in force at
    ///   the conflict's start. Each **absent** when nothing is recorded that
    ///   early;
    /// - `siege` (Siege-kind only): `centre` (`Vector2`, grid cells),
    ///   `radius_cells` (absent without a map scale), and when known
    ///   `besieged_tid`, `besieged_name`, `garrison` (the besieged place's
    ///   MM-6 garrison in `year`'s reading, absent when it has none),
    ///   `defender`, `besiegers` (`PackedInt32Array`);
    /// - `front_segments` (`PackedVector2Array`, endpoint pairs in grid
    ///   coordinates) and `front_cell_count` -- **absent** when the territory
    ///   in force cannot be read, empty when there is no shared border;
    /// - `changed_cells` (`PackedInt32Array`, cell indices) with `changed_to`
    ///   (the new owner, same order) -- **absent** when either end cannot be
    ///   read.
    ///
    /// Empty before any `generate()`, and in any year no conflict is active.
    #[func]
    fn conflict_campaigns(&self, year: i64) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let km_per_cell = cartalith_spatial::cell_km(self.map_width_km, gw.max(1));
        let rows = campaigns_at(
            &civ.timeline,
            &self.conflicts.conflicts,
            |a| anchor_pos(civ, a),
            gw,
            gh,
            km_per_cell,
            year,
        );
        // MM-6: every garrison in this year's reading, computed at most once
        // per call and only when a siege names a place -- this query runs on
        // every cursor step.
        let garrisons = std::cell::OnceCell::new();
        rows.iter()
            .filter_map(|r| {
                let c = self.conflicts.get(r.conflict_id)?;
                let sides: PackedInt32Array = r.sides.iter().copied().collect();
                let mut d = vdict! {
                    "id" => r.conflict_id as i64,
                    "name" => c.name.as_str(),
                    "kind" => c.kind.key(),
                    "sides" => &sides,
                };
                if let Some(y) = r.territory_year {
                    d.set("territory_year", y);
                }
                if let Some(y) = r.baseline_year {
                    d.set("baseline_year", y);
                }
                if let Some(s) = &r.siege {
                    let mut sd = vdict! {
                        "centre" => Vector2::new(s.centre.0 as f32, s.centre.1 as f32),
                    };
                    if let Some(rc) = s.radius_cells {
                        sd.set("radius_cells", rc);
                    }
                    if let Some(t) = s.besieged_tid {
                        sd.set("besieged_tid", t as i64);
                        if let Some(n) = civ.settlements.iter().find(|p| p.tid == t) {
                            sd.set("besieged_name", n.name.as_str());
                        }
                        // MM-6: the besieged place's garrison in this year's
                        // reading -- the defenders' starting force. Reported,
                        // not drawn: the ring does not scale with it
                        // (`MILITARY_MANPOWER_SCOPE.md` §5.1).
                        let found = garrisons
                            .get_or_init(|| self.garrisons_by_tid_at(year))
                            .get(&t)
                            .copied();
                        if let Some(g) = found {
                            sd.set("garrison", g as i64);
                        }
                    }
                    if let Some(f) = s.defender {
                        sd.set("defender", f as i64);
                    }
                    if let Some(b) = &s.besiegers {
                        let b: PackedInt32Array = b.iter().copied().collect();
                        sd.set("besiegers", &b);
                    }
                    d.set("siege", &sd);
                }
                if let (Some(edges), Some(cells)) = (&r.front_edges, &r.front_cells) {
                    let mut seg = PackedVector2Array::new();
                    for &(a, b) in edges {
                        if let Some(((x0, y0), (x1, y1))) = edge_segment(a, b, gw) {
                            seg.push(Vector2::new(x0 as f32, y0 as f32));
                            seg.push(Vector2::new(x1 as f32, y1 as f32));
                        }
                    }
                    d.set("front_segments", &seg);
                    d.set("front_cell_count", cells.len() as i64);
                }
                if let Some(ch) = &r.changed {
                    let cells: PackedInt32Array = ch.iter().map(|c| c.cell as i32).collect();
                    let to: PackedInt32Array = ch.iter().map(|c| c.to).collect();
                    d.set("changed_cells", &cells);
                    d.set("changed_to", &to);
                }
                Some(d)
            })
            .collect()
    }
}
