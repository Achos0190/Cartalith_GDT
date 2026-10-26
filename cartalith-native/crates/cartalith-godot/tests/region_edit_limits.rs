//! LLM-7's engine bars that need no Godot (`LLM_ASSISTANT_SCOPE.md` s.5c): the model-facing limits
//! and the engine's limits are the same numbers, and on real generated worlds (three seeds) a
//! region described, validated and applied behaves as the card promises.
//!
//! `cartalith_llm` cannot depend on `cartalith_terrain` (its one-dependency test), so the limits
//! exist twice; this file is the only thing that stops them drifting. The world half mirrors what
//! `region_edit_bridge.rs` does with a `WorldGen` (which a unit test cannot build): rasterise,
//! measure, hand the numbers to the validator, run the validated operations, and check the result
//! against a recount written differently.

use cartalith_engine::{WorldParams, generate_terrain};
use cartalith_llm::edit_actions::{self as ea, EditCatalogue};
use cartalith_terrain::region_edit as re;
use cartalith_terrain::sculpt::Falloff;
use serde_json::{Value, json};

/// Protects "the validator and the engine call enforce the SAME limits". MUTATION: change a
/// constant on one side only and the matching assertion goes red.
#[test]
fn the_model_facing_limits_equal_the_engine_limits() {
    assert_eq!(ea::MAX_EDIT_OPS, re::MAX_EDIT_OPS);
    assert_eq!(ea::MAX_AMOUNT_M, re::MAX_AMOUNT_M);
    assert_eq!(ea::MAX_TOTAL_AMOUNT_M, re::MAX_TOTAL_AMOUNT_M);
    assert_eq!(ea::MAX_SMOOTH_PASSES, re::MAX_SMOOTH_PASSES);
    assert_eq!(ea::STRENGTH_RANGE, re::STRENGTH_RANGE);
    assert_eq!(ea::MAX_REGION_CELLS, re::MAX_REGION_CELLS);
    assert_eq!(ea::MIN_REGION_CELLS, re::MIN_REGION_CELLS);
    let engine_falloffs: Vec<&str> = Falloff::ALL.iter().map(|f| f.key()).collect();
    assert_eq!(ea::FALLOFFS.to_vec(), engine_falloffs, "the model's ramp names are the sculpt tools' own");
}

fn world(seed: i32) -> (Vec<f32>, usize, usize, f64, f64) {
    let (gw, gh) = (128usize, 80usize);
    let mut p = WorldParams::defaults(gw, gh, seed);
    p.use_gpu = false;
    let peak = p.peak_m;
    let ws = generate_terrain(&p);
    (ws.field.to_vec(), gw, gh, ws.sea_level, peak)
}

/// The facts JSON the bridge builds, from the same engine calls.
fn facts(mask: &re::RegionMask, field: &[f32], gw: usize, gh: usize, sea: f64, peak: f64) -> Value {
    let s = re::stats(mask, field, gw, gh, sea, peak, Some(2.0)).unwrap();
    let (step, rows, approx) = re::run_rows(mask, 24, 6);
    let rows_json: Vec<Value> = rows.iter().map(|(y, runs)| json!([y, runs.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>()])).collect();
    json!({
        "world": {"w": gw, "h": gh, "peak_m": peak},
        "region": {"id": "region_0123456789ab", "shape": "painted", "bbox": [mask.x0, mask.y0, mask.w, mask.h], "cells": mask.count,
            "clipped": mask.clipped, "row_step": step, "approximate": approx, "rows": rows_json,
            "ground": {"min_m": s.min_m, "mean_m": s.mean_m, "max_m": s.max_m, "land_fraction": s.land_fraction,
                       "mean_slope_m_per_km": s.mean_slope_m_per_km}}
    })
}

/// On three real worlds: a painted region is rasterised and measured; the validator, given only
/// those numbers, accepts a ridge request and refuses an over-ceiling one; the validated operations
/// are applied through the engine function; and an independent recount shows (a) no cell outside the
/// region moved, (b) the largest rise is at most the stated metres, (c) a raise never lowers a cell
/// and a lower never raises one, and (d) the same operations give the same bytes twice.
#[test]
fn a_described_validated_applied_region_behaves_as_the_card_promises_on_three_seeds() {
    for seed in [9137, 24601, 71077345] {
        let (field, gw, gh, sea, peak) = world(seed);
        // The candidate whose highest cell is lowest, so the raise below has headroom under the world's
        // ceiling on every seed (the ceiling rule itself is exercised right after).
        let candidates = [
            re::RegionSpec { rect: Some([30.0, 20.0, 40.0, 30.0]), dabs: vec![[90.0, 40.0, 10.0]] },
            re::RegionSpec { rect: Some([2.0, 2.0, 40.0, 30.0]), dabs: vec![[100.0, 60.0, 10.0]] },
            re::RegionSpec { rect: Some([60.0, 45.0, 40.0, 30.0]), dabs: vec![[20.0, 60.0, 10.0]] },
            re::RegionSpec { rect: Some([80.0, 2.0, 40.0, 30.0]), dabs: vec![[10.0, 50.0, 10.0]] },
        ];
        let max_of = |s: &re::RegionSpec| {
            let m = re::rasterise(s, gw, gh).unwrap();
            re::stats(&m, &field, gw, gh, sea, peak, None).unwrap().max_m
        };
        let spec = candidates.iter().min_by(|a, b| max_of(a).total_cmp(&max_of(b))).unwrap().clone();
        let mask = re::rasterise(&spec, gw, gh).unwrap();
        assert!(mask.count > 1000, "seed {seed}: the fixture must hold a real region, not a sliver");
        let f = facts(&mask, &field, gw, gh, sea, peak);
        let cat = EditCatalogue::from_json(&f).unwrap();
        assert!(cat.is_offered(), "seed {seed}: {:?}", cat.refusal());
        let id = "region_0123456789ab";
        let max_m = re::stats(&mask, &field, gw, gh, sea, peak, None).unwrap().max_m;
        let amt = (peak - max_m).floor().min(300.0);
        assert!(amt >= 50.0, "seed {seed}: the fixture needs 50 m of headroom (highest cell {max_m} of {peak})");
        let ok = json!({"edits": [{"operation": "raise", "region_id": id, "amount_m": amt, "falloff": "linear", "softness": 1.0},
                                   {"operation": "smooth", "region_id": id, "strength": 0.5, "passes": 2}]});
        let v = cat.validate(&ok);
        assert!(v.structural.is_none() && v.rejected.is_empty() && v.valid.len() == 2, "seed {seed}: {v:?}");
        // The ceiling rule: raising by 500 m a region whose highest cell is within 500 m of the peak is refused.
        let too_much = (peak - max_m).floor() + 1.0;
        if too_much <= re::MAX_AMOUNT_M {
            let bad = json!({"edits": [{"operation": "raise", "region_id": id, "amount_m": too_much}]});
            assert_eq!(cat.validate(&bad).rejected[0].rule.id(), "R9", "seed {seed}: raise {too_much} on max {max_m} of {peak}");
        }
        // Map the validated rows to engine operations exactly as the bridge does.
        let ops: Vec<re::EditOp> = v
            .valid
            .iter()
            .map(|c| {
                let falloff = Falloff::ALL.iter().copied().find(|f| f.key() == c.falloff).unwrap();
                let op = match c.operation {
                    "raise" => re::Op::Raise { amount_m: c.amount_m.unwrap() },
                    "lower" => re::Op::Lower { amount_m: c.amount_m.unwrap() },
                    _ => re::Op::Smooth { strength: c.strength.unwrap(), passes: c.passes.unwrap() },
                };
                re::EditOp { op, falloff, softness: c.softness }
            })
            .collect();
        let run = |ops: &[re::EditOp]| {
            let mut w = re::window(&mask, &field, gw, gh);
            let rep = re::apply_ops(&mut w, &mask, ops, sea, peak).unwrap();
            let mut after = field.clone();
            re::write_back(&w, &mask, &mut after, gw);
            (after, rep)
        };
        let (after, rep) = run(&ops);
        let (again, _) = run(&ops);
        assert_eq!(after, again, "seed {seed}: deterministic");
        let mut changed = 0usize;
        for y in 0..gh {
            for x in 0..gw {
                let inside = x >= mask.x0 && x < mask.x0 + mask.w && y >= mask.y0 && y < mask.y0 + mask.h && mask.cells[(y - mask.y0) * mask.w + (x - mask.x0)];
                let i = y * gw + x;
                if !inside {
                    assert_eq!(after[i], field[i], "seed {seed}: ({x},{y}) outside the region moved");
                } else if after[i] != field[i] {
                    changed += 1;
                }
            }
        }
        assert_eq!(changed, rep.changed, "seed {seed}");
        assert!(changed > 0, "seed {seed}");
        // A pure raise (no smoothing) never lowers a cell and never exceeds the stated metres.
        let (up, rep_up) = run(&ops[..1]);
        let m_per_unit = peak / (1.0 - sea);
        for i in 0..field.len() {
            assert!(up[i] >= field[i], "seed {seed}: a raise lowered cell {i}");
            assert!((up[i] as f64 - field[i] as f64) * m_per_unit <= amt + 0.01, "seed {seed}: cell {i} rose past {amt} m");
        }
        assert!(rep_up.max_up_m <= amt + 0.01 && rep_up.max_up_m > amt * 0.3, "seed {seed}: {rep_up:?}");
        // A lower never raises a cell.
        let low = [re::EditOp { op: re::Op::Lower { amount_m: 200.0 }, falloff: Falloff::Smooth, softness: 0.5 }];
        let (down, _) = run(&low);
        assert!((0..field.len()).all(|i| down[i] <= field[i]), "seed {seed}: a lower raised a cell");
    }
}
