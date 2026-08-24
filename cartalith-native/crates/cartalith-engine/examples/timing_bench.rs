// Real wall-clock generate_terrain timing at this project's established
// sizes (128/512/1024/2048) -- first written for
// CPU_MULTITHREADING_SCOPE.md's first pass, kept as a reusable tool for
// future CPU/GPU timing passes (this project's own discipline: real
// numbers, not assumed ones, per every GPU_LAYER_INTEGRATION_SCOPE.md
// and CPU_MULTITHREADING_SCOPE.md milestone). `cargo run --release
// --example timing_bench -p cartalith-engine`.
use cartalith_engine::{generate_terrain, WorldParams};
use std::time::Instant;

fn main() {
    for &size in &[128usize, 512, 1024, 2048] {
        let p = WorldParams::defaults(size, size, 12345);
        // Warm up once (first call pays any one-time cost); then take the
        // best of 3 timed runs to reduce OS scheduling noise.
        let _ = generate_terrain(&p);
        let mut best = f64::INFINITY;
        for _ in 0..3 {
            let start = Instant::now();
            let _ = generate_terrain(&p);
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed < best {
                best = elapsed;
            }
        }
        println!("{size}x{size}: {best:.4}s");
    }
}
