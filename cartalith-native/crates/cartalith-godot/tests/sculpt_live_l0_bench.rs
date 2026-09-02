//! `SCULPT_LIVE_SCOPE.md` milestone **L0** — measure, don't optimise.
//!
//! Every scheduling decision in that scope document rests on numbers this
//! file exists to produce: whether `RenderCtx::with_appearance`'s three
//! whole-grid precomputes (`smooth_sea_h`/`build_ao`/`build_hydro_wetness`)
//! really dominate `build_sculpt_preview_texture` (L1's premise), whether
//! GPU flow accumulation is already fast enough at full resolution for L2
//! to skip a proxy LOD, and whether deferring L3 (erosion/climate/civ) is
//! still the right call once climate refresh has a real number next to it.
//!
//! ## Why this lives here, and how it reaches private `render.rs` code
//!
//! `cartalith-godot` is `cdylib`-only (`ARCHITECTURE.md`) — no `rlib` target
//! exists to link an external bench binary against, the same constraint
//! `CPU_MULTITHREADING_SCOPE.md`'s own civ timing bench ran into. This
//! crate's existing tests solve it with `#[path]`, compiling `render.rs`
//! fresh into the test binary itself (`golden_parity_render.rs`,
//! `appearance_ab_dump.rs`, `pack_compositing.rs`, `nonsquare.rs`,
//! `appearance_tiers.rs` all already do this) — followed here rather than
//! invented. `smooth_sea_h`/`build_ao`/`build_hydro_wetness` were bumped
//! from private to `pub(crate)` in `render.rs` to be reachable from this
//! module at all (see their own doc comments) — a visibility-only change,
//! zero behaviour difference, verified by the full existing test suite
//! passing unmodified.
//!
//! `commit_sculpt_pass`'s own four internal steps (stack bake,
//! `enforce_river_channels`, per-river `enforce_channel_descent`, lake
//! deposit) needed no such change: every one of those was already `pub` in
//! `cartalith-spatial`/`cartalith-hydrology`/`cartalith-terrain::sculpt`, so
//! this file just calls them directly, in the same order
//! `cartalith-engine/src/sculpt_commit.rs` does, to break the total down
//! rather than only measuring it end to end.
//!
//! ## What this file does NOT measure, and why
//!
//! **Texture upload.** `Image::create_from_data`/`ImageTexture::
//! create_from_image` cross the GDExtension FFI boundary into a running
//! Godot engine process — calling them here would abort or panic outside
//! one, and standing one up would mean editing `godot-project/**`, off
//! limits for this milestone. The per-pixel colour loop below already
//! includes the byte-packing that feeds `Image::create_from_data` (`lib.rs`'s
//! own loop fuses colour computation and packing in one pass, so there is no
//! separate "packing" stage to isolate); what remains unmeasured is only the
//! engine-side copy into a GPU-visible `Image`/texture resource. That copy
//! moves `gw*gh*3` bytes (3 MB at 1024², 12 MB at 2048²) through a single
//! `Image::create_from_data` call plus one `ImageTexture::create_from_image`
//! upload — bounded well under a millisecond to a few milliseconds on any
//! GPU capable of running this engine at all, and reported as a bound, not a
//! measurement, in `## L0 as measured`.
//!
//! **A GPU path for the stamp/preview/commit stages.** There isn't one.
//! `SculptStamp::apply`, `smooth_sea_h`, `build_ao`, `build_hydro_wetness`
//! and `commit_sculpt_pass` are all CPU-only today, so `use_gpu` has no
//! effect on any of them — confirmed by reading every one of their bodies,
//! not assumed. The `use_gpu` axis the task asks for is real only for
//! `compute_flow` and the climate refresh, which is itself informative: it
//! means L1 has no GPU decision to make, only L2 does.
//!
//! `#[ignore]` by default (real `generate_terrain` calls at up to 2048²,
//! seconds each) — run explicitly:
//! ```text
//! cargo test --release -p cartalith-godot --test sculpt_live_l0_bench -- --ignored --nocapture
//! ```

use std::time::Instant;

use rayon::prelude::*;

#[path = "../src/render.rs"]
mod render;

use cartalith_climate::{apply_climate_moisture_correctors, compute_temperature, simulate_weather, ClimateParams, WeatherParams};
use cartalith_engine::sculpt_commit::{commit_sculpt_pass, WaterState};
use cartalith_engine::{generate_terrain, WorldParams};
use cartalith_hydrology::{compute_flow, enforce_channel_descent, enforce_river_channels};
use cartalith_spatial::{DirtyTracker, PassBuffer, Stamp};
use cartalith_terrain::sculpt::{Feature, Point, SculptStamp};

const SEED: i32 = 12345;
const SIZES: [usize; 3] = [512, 1024, 2048];
const RUNS: usize = 5;

// ---- timing plumbing ----

#[derive(Clone, Copy)]
struct Stats {
    min_ms: f64,
    mean_ms: f64,
    max_ms: f64,
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:8.2} ms  (mean {:8.2}, max {:8.2}, n={RUNS})", self.min_ms, self.mean_ms, self.max_ms)
    }
}

/// One untimed warm-up run, then `RUNS` timed runs, each against a *fresh*
/// fixture `setup` builds -- required here because every operation this file
/// times mutates its own input (a stamp writes into a field, a commit clears
/// a draft), so reusing one fixture across iterations would silently change
/// what later iterations measure. Reports min/mean/max rather than a single
/// sample: `appearance_ab_dump.rs`'s own `cost_table` found a single
/// wall-clock sample at these sizes noisy enough to produce a *negative*
/// marginal cost, and the GPU scope separately flagged single-run variance
/// as a real problem this project has already been burned by once.
fn time_runs<S>(mut setup: impl FnMut() -> S, mut op: impl FnMut(S)) -> Stats {
    {
        let s = setup();
        op(s);
    }
    let mut samples = [0f64; RUNS];
    for sample in &mut samples {
        let s = setup();
        let t0 = Instant::now();
        op(s);
        *sample = t0.elapsed().as_secs_f64() * 1000.0;
    }
    let min_ms = samples.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ms = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean_ms = samples.iter().sum::<f64>() / RUNS as f64;
    Stats { min_ms, mean_ms, max_ms }
}

// ---- fixtures ----

/// A stroke of `n+1` points spanning `length` cells, centred on `(cx, cy)`,
/// dense enough that `enforce_channel_descent` (which walks the stroke's own
/// points and does not resample) carves realistically -- the same density
/// `cartalith-engine/src/sculpt_commit.rs`'s own test fixtures use.
fn stroke(cx: f64, cy: f64, length: f64, n: usize) -> Vec<Point> {
    (0..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            Point::new(cx - length / 2.0 + t * length, cy)
        })
        .collect()
}

fn typical_stroke(gw: usize, gh: usize) -> Vec<Point> {
    stroke(gw as f64 / 2.0, gh as f64 / 2.0, 300.0, 20)
}

fn large_stroke(gw: usize, gh: usize) -> Vec<Point> {
    let length = 0.9 * gw.min(gh) as f64;
    stroke(gw as f64 / 2.0, gh as f64 / 2.0, length, 60)
}

fn mountains_stamp(points: Vec<Point>, brush_size: f64, sea_level: f64) -> SculptStamp {
    let mut s = SculptStamp::new(Feature::Mountains, 42, points, sea_level);
    s.globals.brush_size = brush_size;
    s
}

// ---- B: SculptStamp::apply() ----

fn bench_stamp_apply(field: &[f32], gw: usize, gh: usize, sea_level: f64) {
    let typical = mountains_stamp(typical_stroke(gw, gh), 64.0, sea_level);
    let s = time_runs(|| field.to_vec(), |mut f| typical.apply(&mut f, gw, gh));
    println!("  stamp apply, typical (64px brush, 300px stroke): {s}");

    let large = mountains_stamp(large_stroke(gw, gh), 200.0, sea_level);
    let s = time_runs(|| field.to_vec(), |mut f| large.apply(&mut f, gw, gh));
    println!("  stamp apply, large   (200px brush, {}px stroke): {s}", (0.9 * gw.min(gh) as f64) as i64);
}

// ---- C: build_sculpt_preview_texture, broken down ----

fn bench_preview(ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool) {
    let appearance = render::TerrainAppearance::default();
    let flow = ws.flow_discharge.as_slice();

    let s = time_runs(|| (), |_| { render::smooth_sea_h(&ws.field, gw, gh, world); });
    println!("  smooth_sea_h:                {s}");

    let s = time_runs(|| (), |_| { render::build_ao(&ws.field, gw, gh, ws.sea_level, world, &appearance); });
    println!("  build_ao:                    {s}");

    let s = time_runs(|| (), |_| { render::build_hydro_wetness(Some(flow), gw, gh, world, &appearance); });
    println!("  build_hydro_wetness:         {s}");

    let s = time_runs(
        || render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(flow), gw, gh, ws.sea_level, world, 55.0, 5.0, appearance.clone()),
        |ctx| {
            let mut bytes = vec![0u8; gw * gh * 3];
            bytes.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
                for x in 0..gw {
                    let (r, g, b) = render::cell_color(&ctx, x, y);
                    let o = x * 3;
                    row[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                    row[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                    row[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
                }
            });
        },
    );
    println!("  per-pixel colour loop:       {s}  (matches build_sculpt_preview_texture's own loop, RenderCtx construction excluded)");

    // The constructor call itself must be INSIDE `op`, not `setup` -- an
    // earlier version of this file put it in `setup` by mistake, which
    // timed an empty closure and silently reported near-zero. Caught by
    // sanity-checking against the sum of the three precomputes above before
    // trusting the number (`with_appearance` calls exactly those three plus
    // the much cheaper `sea_shade_from`/`build_lights`, so the ctor total
    // must be close to their sum, not orders of magnitude under it).
    let s = time_runs(
        || (),
        |_| {
            let ctx = render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(flow), gw, gh, ws.sea_level, world, 55.0, 5.0, appearance.clone());
            std::hint::black_box(&ctx);
        },
    );
    println!("  with_appearance (full ctor): {s}  (sum of the three precomputes above plus sea_shade_from/build_lights)");

    println!("  texture upload:              NOT MEASURABLE outside a live Godot process (see module doc); bounded well under a few ms by memory bandwidth on {} bytes", gw * gh * 3);
}

// ---- D: commit_sculpt_pass, broken down ----

fn commit_fixture(gw: usize, gh: usize, ws: &cartalith_engine::WorldState) -> (PassBuffer<SculptStamp>, Vec<f32>, WaterState) {
    let mut buf = PassBuffer::new(gw, gh, 64);
    buf.push(mountains_stamp(typical_stroke(gw, gh), 64.0, ws.sea_level));
    let mut river = SculptStamp::new(Feature::River, 7, typical_stroke(gw, gh), ws.sea_level);
    river.globals.brush_size = 40.0;
    buf.push(river);
    let mut lake = SculptStamp::new(Feature::Lake, 9, vec![Point::new(gw as f64 * 0.5, gh as f64 * 0.5)], ws.sea_level);
    lake.globals.brush_size = 30.0;
    buf.push(lake);

    let field = ws.field.clone();

    // A realistic "earlier commit already locked a channel" fixture --
    // `cartalith-engine/src/sculpt_commit.rs`'s own
    // `an_earlier_lock_is_reclamped_before_new_carving` test uses the same
    // technique. Scaled to ~40% of width so `enforce_river_channels` has
    // real, size-proportional work to do.
    let n = gw * gh;
    let mut water = WaterState::new(n);
    let half = ((gw as f64 * 0.4) as usize) / 2;
    let (cx, cy) = (gw / 2, gh / 2);
    for x in cx.saturating_sub(half)..(cx + half).min(gw) {
        let i = cy * gw + x;
        water.river_mask[i] = 1;
        water.river_floor[i] = (ws.sea_level - 0.05) as f32;
    }
    water.river_any = true;

    (buf, field, water)
}

fn bench_commit(ws: &cartalith_engine::WorldState, gw: usize, gh: usize) {
    // (1) stack bake alone.
    let s = time_runs(
        || {
            let (buf, field, _water) = commit_fixture(gw, gh, ws);
            (buf, field)
        },
        |(mut buf, mut field)| {
            let mut tracker = DirtyTracker::new(buf.tile_count());
            buf.commit(&mut field, &mut tracker, "bench");
        },
    );
    println!("  stack bake (3 stamps):       {s}");

    // (2) enforce_river_channels alone, against the baked field and the
    // pre-locked channel the fixture set up.
    let s = time_runs(
        || {
            let (mut buf, mut field, water) = commit_fixture(gw, gh, ws);
            let mut tracker = DirtyTracker::new(buf.tile_count());
            buf.commit(&mut field, &mut tracker, "bench");
            (field, water)
        },
        |(mut field, water)| {
            enforce_river_channels(&mut field, &water.river_mask, &water.river_floor);
        },
    );
    println!("  enforce_river_channels:      {s}");

    // (3) per-river enforce_channel_descent alone (one river stamp).
    let river_pts: Vec<(f64, f64)> = typical_stroke(gw, gh).iter().map(|p| (p.x, p.y)).collect();
    let sea_level = ws.sea_level;
    let s = time_runs(
        || {
            let (mut buf, mut field, water) = commit_fixture(gw, gh, ws);
            let mut tracker = DirtyTracker::new(buf.tile_count());
            buf.commit(&mut field, &mut tracker, "bench");
            enforce_river_channels(&mut field, &water.river_mask, &water.river_floor);
            field
        },
        |mut field| {
            enforce_channel_descent(&mut field, gw, gh, &river_pts, sea_level, 1.0f64.max(40.0 * 0.13), 0.0006);
        },
    );
    println!("  enforce_channel_descent (1 river): {s}");

    // (4) lake deposit alone (water_only dry run).
    let lake = {
        let mut l = SculptStamp::new(Feature::Lake, 9, vec![Point::new(gw as f64 * 0.5, gh as f64 * 0.5)], sea_level);
        l.globals.brush_size = 30.0;
        l
    };
    let s = time_runs(
        || {
            let (mut buf, mut field, water) = commit_fixture(gw, gh, ws);
            let mut tracker = DirtyTracker::new(buf.tile_count());
            buf.commit(&mut field, &mut tracker, "bench");
            enforce_river_channels(&mut field, &water.river_mask, &water.river_floor);
            // Realistic ordering: the lake step reads the height AFTER this
            // batch's own river carving too (step 3 in `commit_sculpt_pass`),
            // not just after the bake + reclamp.
            enforce_channel_descent(&mut field, gw, gh, &river_pts, sea_level, 1.0f64.max(40.0 * 0.13), 0.0006);
            let n = gw * gh;
            (field, vec![-1f32; n])
        },
        |(mut field, mut surface)| {
            // `water_only = true` never writes `field` (see `apply_into`'s
            // own doc comment) -- passed `&mut` only because the signature
            // requires it, not cloned, so this measures exactly what
            // `commit_sculpt_pass`'s own lake step pays.
            lake.apply_into(&mut field, Some(&mut surface), gw, gh, true);
        },
    );
    println!("  lake deposit (water_only):   {s}");

    // (5) end to end, for a sanity cross-check against the sum of 1-4.
    let s = time_runs(
        || commit_fixture(gw, gh, ws),
        |(mut buf, mut field, mut water)| {
            let mut tracker = DirtyTracker::new(buf.tile_count());
            commit_sculpt_pass(&mut buf, &mut field, &mut water, &mut tracker, "bench", sea_level);
        },
    );
    println!("  commit_sculpt_pass (end to end): {s}");
}

// ---- E/F: downstream stages L2/L3 would have to pay ----

fn climate_ctx(p: &WorldParams, world: bool) -> (ClimateParams, WeatherParams, f64) {
    let sea_level = p.sea_level;
    let climate_params = ClimateParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        g: p.planet.g,
        sea_level,
        peak_m: p.peak_m,
        albedo_k: p.climate.albedo_k,
    };
    let weather_params = WeatherParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        sea_level,
        peak_m: p.peak_m,
        wind_manual: p.climate.wind_manual,
        wind_dir_deg: p.climate.wind_dir_deg,
        press_k: p.climate.press_k,
        ocean_hum: p.climate.ocean_hum,
        evap: p.climate.evap,
        ocean: p.climate.ocean,
        rain_k: p.climate.rain_k,
        rain_dep: p.climate.rain_dep,
        bulk_evap: p.climate.bulk_evap,
        terrain_wind_deflection: p.climate.terrain_wind_deflection,
        currents: p.climate.currents,
        current_k: p.climate.current_k,
    };
    (climate_params, weather_params, sea_level)
}

fn bench_flow_and_climate(p: &WorldParams, ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool, gpu: Option<&cartalith_gpu::GpuDevice>) {
    let field = &ws.field;

    // compute_flow, CPU.
    let s = time_runs(|| (), |_| { compute_flow(gw, gh, field, None, false, world); });
    println!("  compute_flow, CPU:           {s}");

    // compute_flow, GPU -- one warm GpuFlowContext, reused across runs
    // (mirrors generate_terrain's own reuse pattern, `init_gpu_flow_with`
    // once + `dispatch_gpu_flow` per accumulation, not a fresh
    // adapter/device/shader handshake per stroke).
    if let Some(gpu) = gpu {
        let ctx = cartalith_gpu::init_gpu_flow_with(gpu);
        let s = time_runs(|| (), |_| { cartalith_gpu::dispatch_gpu_flow(&ctx, gw, gh, field, None, false, world); });
        println!("  compute_flow, GPU (warm ctx):{s}");
    } else {
        println!("  compute_flow, GPU:           skipped, no wgpu adapter available on this machine");
    }

    let (climate_params, weather_params, sea_level) = climate_ctx(p, world);
    let flow_area = compute_flow(gw, gh, field, None, false, world);

    // Climate refresh, CPU: compute_temperature + simulate_weather +
    // apply_climate_moisture_correctors, the exact chain and order
    // `generate_terrain` runs (`ocean currents` stays off, matching
    // `WorldParams::defaults`).
    let s = time_runs(
        || (),
        |_| {
            let mut temperature = compute_temperature(gw, gh, field, None, &climate_params);
            let mut rainfall = simulate_weather(gw, gh, field, p.climate.w_iters, 0.0, &weather_params);
            apply_climate_moisture_correctors(gw, gh, field, &flow_area, &mut rainfall, sea_level, world, p.climate.lat_n, p.climate.lat_s, p.climate.zonal_k);
            std::hint::black_box(&mut temperature);
        },
    );
    println!("  climate refresh, CPU:        {s}  (temperature + simulate_weather + moisture correctors)");

    if let Some(gpu) = gpu {
        let s = time_runs(
            || (),
            |_| {
                let mut temperature = compute_temperature(gw, gh, field, None, &climate_params);
                let grid = cartalith_climate::build_weather_grid(gw, gh, field, 0.0, &weather_params);
                let (_w, rain) = cartalith_gpu::simulate_weather_loop_gpu_with(
                    gpu, &grid.eh, &grid.tc, &grid.sst_evap, &grid.wx, &grid.wy, &grid.w_init, grid.ww as u32, grid.wh as u32, p.climate.w_iters, grid.sea as f32, grid.ocean_hum as f32,
                    grid.evap as f32, grid.ocean as f32, grid.rain_k as f32, grid.dry as f32, grid.step as f32, grid.bulk_evap, grid.wrap_x,
                ).expect("weather loop must complete on this device");
                let mut rainfall = cartalith_climate::finish_weather_grid(&grid.eh, rain, grid.ww, grid.wh, grid.wrap_x, grid.sea, gw, gh);
                apply_climate_moisture_correctors(gw, gh, field, &flow_area, &mut rainfall, sea_level, world, p.climate.lat_n, p.climate.lat_s, p.climate.zonal_k);
                std::hint::black_box(&mut temperature);
            },
        );
        println!("  climate refresh, GPU:        {s}  (temperature stays CPU -- no GPU path exists for it; weather on GPU)");
    } else {
        println!("  climate refresh, GPU:        skipped, no wgpu adapter available on this machine");
    }
}

#[test]
#[ignore = "real generate_terrain calls up to 2048^2, several seconds; run explicitly with --ignored --nocapture"]
fn l0_measure() {
    let gpu = cartalith_gpu::init_gpu_shared_device().ok();
    match &gpu {
        Some(g) => println!("GPU adapter: {} ({:?}, {:?})", g.adapter_name, g.adapter_backend, g.device_type),
        None => println!("No wgpu adapter available -- GPU rows will be skipped and reported as such."),
    }

    for &n in &SIZES {
        println!("\n==== {n}x{n} ====");
        let p = WorldParams::defaults(n, n, SEED);
        let world = p.world;
        let t0 = Instant::now();
        let ws = generate_terrain(&p);
        println!("  (fixture: generate_terrain took {:.0} ms, not part of any reported figure)", t0.elapsed().as_secs_f64() * 1000.0);

        println!(" -- B: SculptStamp::apply() --");
        bench_stamp_apply(&ws.field, n, n, ws.sea_level);

        println!(" -- C: build_sculpt_preview_texture, broken down --");
        bench_preview(&ws, n, n, world);

        println!(" -- D: commit_sculpt_pass, broken down --");
        bench_commit(&ws, n, n);

        println!(" -- E/F: downstream stages a commit does not run today --");
        bench_flow_and_climate(&p, &ws, n, n, world, gpu.as_ref());
    }
}
