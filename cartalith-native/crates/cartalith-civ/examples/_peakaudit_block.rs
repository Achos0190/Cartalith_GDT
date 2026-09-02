//! `_peakaudit_block` — costing probe for one proposed change. **Throwaway.**
//!
//! `build_resource_potentials` collects its per-cell results into one
//! `Vec<[f32; 15]>` (60 B/cell) before scattering them into the 15 named
//! output `Vec`s, and that intermediate is the single largest transient in
//! the whole pipeline (`MEMORY_OPTIMIZATION_SCOPE.md`'s generation-peak
//! audit measured it at 153.63 MiB at 2048x1311).
//!
//! The proposed fix is to run the same `par_iter` in fixed-size blocks,
//! reusing one small buffer. This probe measures **what that costs in ms**,
//! with the same rayon dispatch shape and the same scatter, over the same
//! cell count — a stand-in kernel rather than the real 15-resource maths,
//! because the question is dispatch and allocation overhead, not arithmetic.
//!
//! ```text
//! cargo run --release -p cartalith-civ --example _peakaudit_block -- <n> [block]
//! ```

use rayon::prelude::*;
use std::time::Instant;

#[inline]
fn kernel(i: usize) -> [f32; 15] {
    let x = i as f32 * 1.000_001;
    let mut o = [0f32; 15];
    for (k, v) in o.iter_mut().enumerate() {
        *v = (x * (k as f32 + 1.0)).sin();
    }
    o
}

fn scatter(src: &[[f32; 15]], base: usize, outs: &mut [Vec<f32>; 15]) {
    for (j, cell) in src.iter().enumerate() {
        for k in 0..15 {
            outs[k][base + j] = cell[k];
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(2_684_928);
    let block: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1 << 16);
    println!("n = {n} cells, block = {block} cells ({:.2} MiB buffer vs {:.2} MiB monolithic)",
        (block * 60) as f64 / 1048576.0, (n * 60) as f64 / 1048576.0);

    for rep in 0..3 {
        // --- monolithic: today's shape -------------------------------------
        let mut outs: [Vec<f32>; 15] = std::array::from_fn(|_| vec![0f32; n]);
        let t = Instant::now();
        let per_cell: Vec<[f32; 15]> = (0..n).into_par_iter().map(kernel).collect();
        let t_collect = t.elapsed();
        scatter(&per_cell, 0, &mut outs);
        let t_mono = t.elapsed();
        drop(per_cell);
        let sum_mono: f64 = outs.iter().map(|v| v[n / 2] as f64).sum();
        drop(outs);

        // --- blocked: the proposal -----------------------------------------
        let mut outs: [Vec<f32>; 15] = std::array::from_fn(|_| vec![0f32; n]);
        let mut buf: Vec<[f32; 15]> = Vec::with_capacity(block);
        let t = Instant::now();
        let mut start = 0usize;
        while start < n {
            let end = (start + block).min(n);
            (start..end).into_par_iter().map(kernel).collect_into_vec(&mut buf);
            scatter(&buf, start, &mut outs);
            start = end;
        }
        let t_block = t.elapsed();
        let sum_block: f64 = outs.iter().map(|v| v[n / 2] as f64).sum();

        println!(
            "rep {rep}: monolithic {:7.1} ms (collect {:7.1} ms)   blocked {:7.1} ms   delta {:+7.1} ms   identical={}",
            t_mono.as_secs_f64() * 1000.0,
            t_collect.as_secs_f64() * 1000.0,
            t_block.as_secs_f64() * 1000.0,
            (t_block.as_secs_f64() - t_mono.as_secs_f64()) * 1000.0,
            (sum_mono - sum_block).abs() < 1e-12
        );
    }
}
