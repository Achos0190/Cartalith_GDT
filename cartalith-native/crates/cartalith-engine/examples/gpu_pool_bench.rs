//! `use_gpu` generation time with and without `cartalith-gpu`'s buffer pool
//! (`pool.rs`, `OUTSTANDING_WORK.md` §2.6, Ruling AZ).
//!
//! ```text
//! cargo run --release -p cartalith-engine --example gpu_pool_bench -- <size> <reps>
//! ```
//!
//! Toggles the pool on the process-wide cached device -- the very one
//! `generate_terrain` borrows, since every clone of a `GpuDevice` shares one
//! pool. Runs in blocks rather than alternating per generation, because turning
//! the pool off releases it: a pooled generation is only meaningful after a
//! pooled one, which is the "kept between generations" case being measured.
//! Each block starts with one discarded primer run; blocks go pooled, unpooled,
//! pooled, unpooled so a slow drift in the machine lands on both sides.
//!
//! Run it alone. Timings taken while anything else compiles or renders are not
//! measurements (`MISTAKES.md`).

use std::time::Instant;

use cartalith_engine::{WorldParams, generate_terrain};

fn stats(v: &mut [f64]) -> (f64, f64, f64) {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let med = if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 };
    (med, v[0], v[n - 1])
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let size: usize = a.first().and_then(|s| s.parse().ok()).unwrap_or(1024);
    let reps: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);

    cartalith_gpu::set_preferences(cartalith_gpu::GpuPreferences::default());
    let set = cartalith_gpu::init_gpu_device_set().expect("a GPU is required for this benchmark");
    let pool = set.primary().buffer_pool();
    println!("# device {:?}, size {size}², {reps} timed runs per block, 2 blocks per mode", set.primary().adapter_name);

    let mut p = WorldParams::defaults(size, size, 12345);
    p.use_gpu = true;

    let mut times: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut per_gen: [Vec<(u64, u64, u64)>; 2] = [Vec::new(), Vec::new()];
    for block in 0..4 {
        let pooled = block % 2 == 0;
        pool.set_enabled(pooled);
        let ws = generate_terrain(&p); // primer, discarded
        assert!(!ws.gpu_stages_used.is_empty(), "the primer ran no GPU stage -- nothing to measure");
        for _ in 0..reps {
            let before = pool.stats();
            let t = Instant::now();
            let ws = generate_terrain(&p);
            let ms = t.elapsed().as_secs_f64() * 1e3;
            let after = pool.stats();
            std::hint::black_box(&ws.field);
            times[usize::from(!pooled)].push(ms);
            per_gen[usize::from(!pooled)].push((
                after.misses - before.misses,
                after.hits - before.hits,
                after.fresh_bytes - before.fresh_bytes,
            ));
            println!("block{block} pooled={pooled} {ms:.1}ms stages={}", ws.gpu_stages_used.len());
        }
        if pooled {
            let s = pool.stats();
            println!("# after block{block}: retained {} buffers, {} MiB of a {} MiB cap", s.retained_buffers, s.retained_bytes >> 20, s.cap_bytes >> 20);
        }
    }

    for (i, label) in ["pooled", "unpooled"].iter().enumerate() {
        let (med, lo, hi) = stats(&mut times[i]);
        let g = &per_gen[i];
        let (mi, hi_, fb) = g[g.len() - 1];
        println!(
            "RESULT size={size} {label}: median {med:.1} ms ({lo:.1}..{hi:.1}), n={} | last generation: {mi} buffers allocated ({} MiB), {hi_} served from pool",
            times[i].len(),
            fb >> 20
        );
    }
    let (pm, _, _) = stats(&mut times[0]);
    let (um, _, _) = stats(&mut times[1]);
    println!("RESULT size={size} median delta (unpooled - pooled): {:.1} ms ({:+.2}%)", um - pm, 100.0 * (um - pm) / um);
}
