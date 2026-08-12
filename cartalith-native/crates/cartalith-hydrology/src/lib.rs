//! flow accumulation, river network, channel width
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

/// Descending-height, ascending-index-on-tie comparison — the ordering
/// `_flowRadixSortDesc()` (reference HTML lines 4846-4861) guarantees.
/// The JS implementation is a radix sort operating on IEEE-754 bit
/// patterns (an order-preserving float→uint key, inverted for descending
/// order); the *algorithm* is a correctness-equivalent substitution
/// target per `PROVENANCE.md` (flow accumulation is downstream of the
/// heightmap pixels — only the ordering guarantee matters for parity,
/// not the sort implementation), but its **quirk carries over**: JS
/// explicitly normalizes `-0.0`'s sort key to match `+0.0`
/// (`if(b===0x80000000) b=0`), which `f32::total_cmp` does not do on its
/// own (`total_cmp` treats `-0.0 < +0.0`) — normalized here before
/// comparing.
fn flow_cmp_desc(a: f32, b: f32) -> std::cmp::Ordering {
    let na = if a == 0.0 { 0.0f32 } else { a };
    let nb = if b == 0.0 { 0.0f32 } else { b };
    nb.total_cmp(&na)
}

/// `computeFlow()` (reference HTML lines 4862-4890): D8 steepest-descent
/// flow accumulation. Processes cells in descending-height order so that
/// by the time a cell is visited, every upstream contribution has already
/// accumulated into it (classic drainage-order trick) — each cell then
/// pushes its full accumulated flow to its single steepest downhill
/// neighbor.
///
/// `useRain=true` seeds discharge from rainfall (mean-normalized, floor
/// `0.05`) rather than bare cell count — rivers accumulate runoff, not
/// area (Whipple & Tucker 1999). `useRain=false` (the `acc.fill(1)`
/// path) is the area-only seeding `computeFlow()`'s first call in
/// `generate()` uses, before climate exists yet.
///
/// `acc[best]+=acc[i]` is the same "multiple cells can write to one
/// target within a pass" trap `erode_thermal`'s `delta[j]+=` needed —
/// a downhill cell can receive accumulated flow from several different
/// upstream cells, each JS write rounding to `f32` individually. Kept as
/// per-write rounding here too, not an `f64` accumulator.
pub fn compute_flow(gw: usize, gh: usize, field: &[f32], rain: Option<&[f32]>, use_rain: bool, world: bool) -> Vec<f32> {
    let n = gw * gh;
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| flow_cmp_desc(field[a], field[b]).then(a.cmp(&b)));

    let mut acc = vec![0f32; n];
    if use_rain {
        let rain = rain.expect("rain field required when use_rain is true");
        let mut sm = 0.0f64;
        for i in 0..n {
            let r = (rain[i] as f64).max(0.05);
            acc[i] = r as f32;
            sm += r;
        }
        let k = n as f64 / sm.max(1e-6);
        for v in &mut acc {
            *v = (*v as f64 * k) as f32;
        }
    } else {
        acc.fill(1.0);
    }

    // D8[(dy+1)*3+(dx+1)] = hypot(dx, dy) for dx,dy in {-1,0,1}; center
    // (index 4) is unused (the main loop always skips dx=dy=0) but kept
    // for a direct match to the reference's own indexing scheme.
    let mut d8 = [0f64; 9];
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
        }
    }

    for &i in &order {
        let x = (i % gw) as i64;
        let y = (i / gw) as i64;
        let h = field[i] as f64;
        let mut best: i64 = -1;
        let mut best_drop = 0.0f64;
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let mut nx = x + dx;
                let ny = y + dy;
                if world {
                    nx = ((nx % gw as i64) + gw as i64) % gw as i64;
                } else if nx < 0 || nx >= gw as i64 {
                    continue;
                }
                if ny < 0 || ny >= gh as i64 {
                    continue;
                }
                let j = ny * gw as i64 + nx;
                let drop = (h - field[j as usize] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
                if drop > best_drop {
                    best_drop = drop;
                    best = j;
                }
            }
        }
        if best >= 0 {
            let best = best as usize;
            acc[best] = (acc[best] as f64 + acc[i] as f64) as f32;
        }
    }

    acc
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }
}
