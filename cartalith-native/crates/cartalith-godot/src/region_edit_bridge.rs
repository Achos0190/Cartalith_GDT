//! The engine half of the assistant's edit actions (`LLM_ASSISTANT_SCOPE.md` s.5c, milestone
//! LLM-7; owner Rulings DD Q10 and DQ): describe a region the user painted or boxed, preview an edit
//! of it on a COPY, and apply it as one undo step. The maths is `cartalith_terrain::region_edit`;
//! this file is the Godot conversion and the world bookkeeping.
//!
//! # The three `#[func]`s
//!
//! * [`WorldGen::region_describe`] reads. A spec `{rect?: [x, y, w, h], dabs?: [[cx, cy, r], ...]}`
//!   (grid cells) becomes a mask, its id and the facts the model is told: bounding box in cells and
//!   km, cell count, run-length rows, min/mean/max height in metres, land fraction, mean slope.
//!   Nothing is written; a failure is `{ok: false, reason}`, never a default region.
//! * [`WorldGen::region_edit_preview`] reads. Runs the operations on a copy of the region's window
//!   (the SAME function apply uses) and reports before/after statistics, the cells that crossed sea
//!   level, the cells a clamp stopped, a paint conflict, and two small RGB thumbnails.
//! * [`WorldGen::region_edit_apply`] writes, as exactly the bookkeeping a sculpt commit does and
//!   nothing else: the bake check, a height undo snapshot, the redo tail dropped, one ledger row of
//!   kind `height` (so `undo_group.gd` collects it as a group member), the field written inside the
//!   mask only, and hydrology and climate re-run through the staleness graph for the touched tiles
//!   (`mark_and_recompute`, as `sculpt_commit` does). It never generates and never erodes.
//!
//! # What apply refuses, each with a reason
//!
//! No generated world; a baked (finalised) world; a region or an operation list that breaks a limit
//! (the SAME constants the model-facing validator carries; `tests/region_edit_limits.rs` pins that
//! they are equal); a region that overlaps committed colour paint (Ruling AS clears paint under a
//! sculpt and the undo cannot bring it back, so the assistant refuses rather than destroy work
//! the undo cannot restore; the owner can clear the paint first. Owner question E7).
//!
//! MUST NEVER: write a cell outside the mask; write before every check has passed; leave the undo
//! snapshot pushed when a check fails after it (the push is the LAST fallible step's successor).

use godot::prelude::*;

use cartalith_engine::staleness::PipelineStage;
use cartalith_terrain::region_edit::{self as re, EditOp, Op, RegionMask, RegionSpec};
use cartalith_terrain::sculpt::Falloff;

use crate::undo_group_bridge::fnv1a;
use crate::{undo, WorldGen, WorldSource};

/// Thumbnail long side in pixels (a labelled judgement: two of them sit side by side in a 560 px
/// window and 96 px is large enough to see a ridge).
const THUMB_SIDE: usize = 96;

/// Read a Godot number out of a `Variant` (int or float), `None` for anything else (a string is
/// never read as a number).
fn num(v: &Variant) -> Option<f64> {
    match v.get_type() {
        VariantType::FLOAT => Some(v.to::<f64>()),
        VariantType::INT => Some(v.to::<i64>() as f64),
        _ => None,
    }
}

fn fail(reason: impl Into<String>) -> VarDictionary {
    let r: String = reason.into();
    vdict! { "ok" => false, "reason" => r.as_str() }
}

/// `{rect: [x, y, w, h]?, dabs: [[x, y, r], ...]?}` into a [`RegionSpec`]. Strict: a wrong shape is
/// refused with words.
fn spec_from(d: &VarDictionary) -> Result<RegionSpec, String> {
    let mut spec = RegionSpec::default();
    if let Some(r) = d.get("rect") {
        let a = r.try_to::<VarArray>().map_err(|_| "the box is not a list of four numbers".to_string())?;
        let n: Vec<f64> = a.iter_shared().filter_map(|v| num(&v)).collect();
        if a.len() != 4 || n.len() != 4 {
            return Err("the box needs four numbers: x, y, width, height".into());
        }
        spec.rect = Some([n[0], n[1], n[2], n[3]]);
    }
    if let Some(ds) = d.get("dabs") {
        let a = ds.try_to::<VarArray>().map_err(|_| "the paint dabs are not a list".to_string())?;
        if a.len() > re::MAX_DABS {
            return Err(format!("the region is made of {} paint dabs; at most {} are accepted", a.len(), re::MAX_DABS));
        }
        for v in a.iter_shared() {
            let t = v.try_to::<VarArray>().map_err(|_| "a paint dab is not a list of three numbers".to_string())?;
            let n: Vec<f64> = t.iter_shared().filter_map(|x| num(&x)).collect();
            if t.len() != 3 || n.len() != 3 {
                return Err("a paint dab needs three numbers: x, y, radius".into());
            }
            spec.dabs.push([n[0], n[1], n[2]]);
        }
    }
    Ok(spec)
}

fn falloff_from(key: &str) -> Option<Falloff> {
    Falloff::ALL.iter().copied().find(|f| f.key() == key)
}

/// `[{operation, amount_m?, strength?, passes?, falloff, softness}, ...]` into engine operations.
/// Strict and typed: an unknown operation or falloff, a missing or non-number field is a refusal.
fn ops_from(a: &VarArray) -> Result<Vec<EditOp>, String> {
    let mut out = Vec::new();
    for v in a.iter_shared() {
        let d = v.try_to::<VarDictionary>().map_err(|_| "an operation is not a dictionary".to_string())?;
        let get = |k: &str| d.get(k).and_then(|x| num(&x));
        let key = d.get("operation").and_then(|x| x.try_to::<GString>().ok()).map(|g| g.to_string()).unwrap_or_default();
        let op = match key.as_str() {
            "raise" => Op::Raise { amount_m: get("amount_m").ok_or("raise needs amount_m")? },
            "lower" => Op::Lower { amount_m: get("amount_m").ok_or("lower needs amount_m")? },
            "smooth" => {
                let p = get("passes").ok_or("smooth needs passes")?;
                if p.fract() != 0.0 || !(0.0..=1000.0).contains(&p) {
                    return Err("passes must be a whole number".into());
                }
                Op::Smooth { strength: get("strength").ok_or("smooth needs strength")?, passes: p as u32 }
            }
            other => return Err(format!("unknown operation {other}")),
        };
        let fk = d.get("falloff").and_then(|x| x.try_to::<GString>().ok()).map(|g| g.to_string()).ok_or("an operation needs a falloff")?;
        let falloff = falloff_from(&fk).ok_or_else(|| format!("unknown falloff {fk}"))?;
        out.push(EditOp { op, falloff, softness: get("softness").ok_or("an operation needs a softness")? });
    }
    Ok(out)
}

/// The region's id: `region_` and 12 hex digits of the FNV-1a of its bounding box and cells, so the
/// same painted shape is the same id and a different one is not. (The id names a SHAPE; Apply
/// re-reads the ground, so a stale id cannot apply old heights.)
fn region_id(mask: &RegionMask) -> String {
    let mut bytes = Vec::with_capacity(32 + mask.cells.len());
    for v in [mask.x0, mask.y0, mask.w, mask.h] {
        bytes.extend_from_slice(&(v as u64).to_le_bytes());
    }
    bytes.extend(mask.cells.iter().map(|c| u8::from(*c)));
    format!("region_{:012x}", fnv1a(&bytes) & 0xffff_ffff_ffff)
}

fn stats_dict(s: &re::RegionStats) -> VarDictionary {
    let mut g = vdict! { "min_m" => s.min_m, "mean_m" => s.mean_m, "max_m" => s.max_m, "land_fraction" => s.land_fraction };
    if let Some(sl) = s.mean_slope_m_per_km {
        g.set("mean_slope_m_per_km", sl);
    }
    g
}

#[godot_api(secondary)]
impl WorldGen {
    /// Describe the region `spec` over the open world, for the assistant (read-only). Answers
    /// `{ok: true, id, shape, bbox: [x, y, w, h], cells, clipped, rows, row_step, approximate,
    /// ground?, area_km2?, bbox_km?, world: {w, h, peak_m}}`, or `{ok: false, reason}`. Absent
    /// figures are omitted: no `ground` without a readable height field, no km figures without a map
    /// width. `rows` is `[[y, [[x0, x1], ...]], ...]` (see `region_edit::run_rows`).
    #[func]
    fn region_describe(&self, spec: VarDictionary) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return fail("No generated world is open.") };
        let spec = match spec_from(&spec) {
            Ok(s) => s,
            Err(w) => return fail(w),
        };
        let mask = match re::rasterise(&spec, f.gw, f.gh) {
            Ok(m) => m,
            Err(e) => return fail(e.to_string()),
        };
        let cell_km = (f.map_width_km > 0.0 && f.gw > 0).then(|| f.map_width_km / f.gw as f64);
        let mut d = vdict! {
            "ok" => true,
            "id" => region_id(&mask).as_str(),
            "shape" => if spec.dabs.is_empty() { "box" } else { "painted" },
            "cells" => mask.count as i64,
            "clipped" => mask.clipped,
        };
        d.set("bbox", &VarArray::from(&[(mask.x0 as i64).to_variant(), (mask.y0 as i64).to_variant(), (mask.w as i64).to_variant(), (mask.h as i64).to_variant()]));
        let (step, rows, approx) = re::run_rows(&mask, 24, 6);
        let mut rows_out = VarArray::new();
        for (y, runs) in &rows {
            let mut rs = VarArray::new();
            for (a, b) in runs {
                rs.push(&VarArray::from(&[(*a as i64).to_variant(), (*b as i64).to_variant()]).to_variant());
            }
            rows_out.push(&VarArray::from(&[(*y as i64).to_variant(), rs.to_variant()]).to_variant());
        }
        d.set("rows", &rows_out);
        d.set("row_step", step as i64);
        d.set("approximate", approx);
        if let Some(s) = re::stats(&mask, f.field, f.gw, f.gh, f.sea_level, f.peak_m, cell_km) {
            d.set("ground", &stats_dict(&s));
        }
        if let Some(km) = cell_km {
            d.set("area_km2", mask.count as f64 * km * km);
            d.set(
                "bbox_km",
                &VarArray::from(&[
                    (mask.x0 as f64 * km).to_variant(),
                    (mask.y0 as f64 * km).to_variant(),
                    ((mask.x0 + mask.w) as f64 * km).to_variant(),
                    ((mask.y0 + mask.h) as f64 * km).to_variant(),
                ]),
            );
        }
        d.set("world", &vdict! { "w" => f.gw as i64, "h" => f.gh as i64, "peak_m" => f.peak_m });
        d
    }

    /// Preview `ops` on a COPY of the region's window (read-only; the world does not change).
    /// Answers `{ok: true, changed, max_up_m, max_down_m, became_sea, became_land, capped, floored, limited,
    /// paint_cells, before: {min_m, mean_m, max_m}, after: {..}, mean_change_m, thumb_w, thumb_h,
    /// thumb_before, thumb_after}` (the thumbnails are RGB8 `PackedByteArray`s on one shared height
    /// scale), or `{ok: false, reason}`. The operations pass the same limits Apply enforces.
    #[func]
    fn region_edit_preview(&self, spec: VarDictionary, ops: VarArray) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return fail("No generated world is open.") };
        let (mask, ops) = match (|| -> Result<(RegionMask, Vec<EditOp>), String> {
            let spec = spec_from(&spec)?;
            let mask = re::rasterise(&spec, f.gw, f.gh).map_err(|e| e.to_string())?;
            let ops = ops_from(&ops)?;
            re::check_ops(&ops).map_err(|e| e.to_string())?;
            Ok((mask, ops))
        })() {
            Ok(t) => t,
            Err(w) => return fail(w),
        };
        let before_win = re::window(&mask, f.field, f.gw, f.gh);
        let mut after_win = before_win.clone();
        let rep = match re::apply_ops(&mut after_win, &mask, &ops, f.sea_level, f.peak_m) {
            Ok(r) => r,
            Err(e) => return fail(e.to_string()),
        };
        let mut after_field = f.field.to_vec();
        re::write_back(&after_win, &mask, &mut after_field, f.gw);
        let cell_km = (f.map_width_km > 0.0 && f.gw > 0).then(|| f.map_width_km / f.gw as f64);
        let (Some(sb), Some(sa)) = (
            re::stats(&mask, f.field, f.gw, f.gh, f.sea_level, f.peak_m, cell_km),
            re::stats(&mask, &after_field, f.gw, f.gh, f.sea_level, f.peak_m, cell_km),
        ) else {
            return fail("The region's heights could not be measured.");
        };
        let (lo, hi) = before_win.data.iter().chain(after_win.data.iter()).fold((f32::INFINITY, f32::NEG_INFINITY), |(l, h), v| (l.min(*v), h.max(*v)));
        let (tw, th, tb) = re::thumbnail(&before_win, &mask, f.sea_level, lo, hi, THUMB_SIDE);
        let (_, _, ta) = re::thumbnail(&after_win, &mask, f.sea_level, lo, hi, THUMB_SIDE);
        let paint = self.paint_cells_under(&mask);
        let mut d = vdict! {
            "ok" => true,
            "changed" => rep.changed as i64,
            "max_up_m" => rep.max_up_m,
            "max_down_m" => rep.max_down_m,
            "became_sea" => rep.became_sea as i64,
            "became_land" => rep.became_land as i64,
            "capped" => rep.capped as i64,
            "floored" => rep.floored as i64,
            "limited" => rep.limited as i64,
            "paint_cells" => paint as i64,
            "mean_change_m" => sa.mean_m - sb.mean_m,
            "thumb_w" => tw as i64,
            "thumb_h" => th as i64,
        };
        d.set("before", &stats_dict(&sb));
        d.set("after", &stats_dict(&sa));
        d.set("thumb_before", &PackedByteArray::from(tb.as_slice()));
        d.set("thumb_after", &PackedByteArray::from(ta.as_slice()));
        d
    }

    /// Apply `ops` to the region as ONE undo step (see the module doc). `label` is the ledger row's
    /// label. Answers `{ok: true, changed, recomputed, still_stale}` or `{ok: false, reason}`; on a
    /// refusal nothing was written and nothing was pushed.
    #[func]
    fn region_edit_apply(&mut self, spec: VarDictionary, ops: VarArray, label: GString) -> VarDictionary {
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::HeightEdit) {
            return fail(format!("{msg}"));
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let (sea, peak) = (self.sea_level, self.params.peak_m);
        let parsed = (|| -> Result<(RegionMask, Vec<EditOp>), String> {
            let spec = spec_from(&spec)?;
            let mask = re::rasterise(&spec, gw, gh).map_err(|e| e.to_string())?;
            let ops = ops_from(&ops)?;
            re::check_ops(&ops).map_err(|e| e.to_string())?;
            Ok((mask, ops))
        })();
        let (mask, ops) = match parsed {
            Ok(t) => t,
            Err(w) => return fail(w),
        };
        let paint = self.paint_cells_under(&mask);
        if paint > 0 {
            return fail(format!(
                "{paint} painted cells lie under the region. An edit would clear them and Undo could not bring them back, so the assistant does not edit there: clear the paint first, or paint a different region."
            ));
        }
        let label = label.to_string();
        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else { return fail("No generated world is open.") };
        if ws.field.len() != gw * gh {
            return fail("No generated world is open.");
        }
        // Compute on a window copy FIRST: every fallible step runs before the undo snapshot is taken.
        let mut win = re::window(&mask, &ws.field, gw, gh);
        let rep = match re::apply_ops(&mut win, &mask, &ops, sea, peak) {
            Ok(r) => r,
            Err(e) => return fail(e.to_string()),
        };
        if rep.changed == 0 {
            return fail("The edit would not change any cell, so nothing was applied.");
        }
        self.undo.push("Assistant edit", &ws.field);
        self.redo.clear();
        self.ledger.record("height", if label.is_empty() { "Assistant edit".to_owned() } else { label }, format!("{} cells", rep.changed), undo::EntryKind::HeightSnapshot);
        re::write_back(&win, &mask, std::sync::Arc::make_mut(&mut ws.field).as_mut_slice(), gw);
        let tiles = re::tiles_touched(&mask, gw);
        let (recomputed, still_stale) = self.mark_and_recompute(PipelineStage::Height, tiles, "assistant_edit");
        vdict! { "ok" => true, "changed" => rep.changed as i64, "recomputed" => &recomputed, "still_stale" => &still_stale }
    }
}

impl WorldGen {
    /// Committed colour-override paint cells under the region (zero without paint). Read-only.
    fn paint_cells_under(&self, mask: &RegionMask) -> usize {
        let Some(p) = self.paint.as_ref() else { return 0 };
        if !p.has_committed_paint() {
            return 0;
        }
        let n = (self.gw.max(0) as usize) * (self.gh.max(0) as usize);
        let gw = self.gw.max(0) as usize;
        let mut footprint = vec![false; n];
        for row in 0..mask.h {
            for col in 0..mask.w {
                if mask.cells[row * mask.w + col] {
                    footprint[(mask.y0 + row) * gw + mask.x0 + col] = true;
                }
            }
        }
        p.count_cells_under(n, &footprint)
    }
}
