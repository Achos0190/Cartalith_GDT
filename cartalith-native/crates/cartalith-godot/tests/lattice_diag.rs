//! SCRATCH diagnosis harness for the "diamond lattice" row. Not a regression test.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/render.rs"]
mod render;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

fn app_world(seed: i32, gw: usize, gh: usize, km: f64, edit: impl Fn(&mut WorldParams)) -> (WorldParams, WorldState) {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = km;
    p.sea_level = 0.42;
    edit(&mut p);
    let ws = generate_terrain(&p);
    (p, ws)
}

/// Fraction of the 3x3-highpass energy that sits at the exact (pi,pi) Nyquist checker,
/// measured per 16x16 block and reported as the max block and the region mean of |block mean|.
fn nyq(f: &[f64], gw: usize, x0: usize, y0: usize, w: usize, h: usize) -> (f64, f64, f64) {
    let at = |x: usize, y: usize| f[y * gw + x];
    let bs = 16;
    let (mut best, mut sum, mut n) = (0.0f64, 0.0f64, 0usize);
    let (mut rms_all, mut cnt) = (0.0f64, 0usize);
    let mut by = y0 + 1;
    while by + bs < y0 + h - 1 {
        let mut bx = x0 + 1;
        while bx + bs < x0 + w - 1 {
            let mut acc = 0.0;
            for y in by..by + bs {
                for x in bx..bx + bs {
                    let m = (at(x - 1, y - 1) + at(x, y - 1) + at(x + 1, y - 1) + at(x - 1, y) + at(x, y) + at(x + 1, y) + at(x - 1, y + 1) + at(x, y + 1) + at(x + 1, y + 1)) / 9.0;
                    let hp = at(x, y) - m;
                    let s = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
                    acc += hp * s;
                    rms_all += hp * hp;
                    cnt += 1;
                }
            }
            let a = (acc / (bs * bs) as f64).abs();
            best = best.max(a);
            sum += a;
            n += 1;
            bx += bs;
        }
        by += bs;
    }
    (best, sum / n.max(1) as f64, (rms_all / cnt.max(1) as f64).sqrt())
}

fn lum(ctx: &render::RenderCtx, gw: usize, gh: usize) -> Vec<f64> {
    let mut v = vec![0.0; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(ctx, x, y);
            v[y * gw + x] = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        }
    }
    v
}

#[test]
#[ignore]
fn diag_lattice() {
    let (gw, gh) = (1024usize, 768usize);
    println!("defaults use_gpu={}", params::defaults().use_gpu);
    let (p, ws) = app_world(483920, gw, gh, 1200.0, |p| { if std::env::var("LAT_GPU").is_ok() { p.use_gpu = true; } });
    println!("field[398,668] = {} (GUI sample_cell said 0.59100025892258)", ws.field[668 * gw + 398]);
    let f: Vec<f64> = ws.field.iter().map(|&v| v as f64).collect();
    for (name, x0, y0, w, h) in [("roi 380..480 x 640..740", 380, 640, 100, 100), ("whole land-ish 0..1024x0..768", 0, 0, gw, gh)] {
        let (mx, mean, rms) = nyq(&f, gw, x0, y0, w, h);
        println!("ELEV  {name}: nyq block max {mx:.3e} mean {mean:.3e} hp_rms {rms:.3e}");
    }
    let a = render::TerrainAppearance::default();
    let ctx = render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a);
    if let Ok(dir) = std::env::var("LAT_OUT") {
        let (x0, y0, w, h) = (380usize, 640usize, 100usize, 100usize);
        let mut buf = Vec::new();
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let (r, g, b) = render::cell_color(&ctx, x, y);
                buf.extend_from_slice(&[(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
            }
        }
        std::fs::write(format!("{dir}/roi_{w}x{h}.rgb"), buf).unwrap();
        let mut fb = Vec::new();
        for y in y0..y0 + h { for x in x0..x0 + w { fb.extend_from_slice(&ws.field[y * gw + x].to_le_bytes()); } }
        std::fs::write(format!("{dir}/roi_field.f32"), fb).unwrap();
    }
    let l = lum(&ctx, gw, gh);
    for (name, x0, y0, w, h) in [("roi", 380, 640, 100, 100), ("all", 0, 0, gw, gh)] {
        let (mx, mean, rms) = nyq(&l, gw, x0, y0, w, h);
        println!("LUM   {name}: nyq block max {mx:.3e} mean {mean:.3e} hp_rms {rms:.3e}");
    }
}

/// 3x3 highpass rms over land cells and the Q4 checker fraction (1/16 for white noise, 1 for a pure checker).
fn hf_stats(f: &[f64], land: &[bool], gw: usize, gh: usize) -> (f64, f64) {
    let at = |x: usize, y: usize| f[y * gw + x];
    let mut hp = vec![0.0f64; gw * gh];
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let m = (at(x - 1, y - 1) + at(x, y - 1) + at(x + 1, y - 1) + at(x - 1, y) + at(x, y) + at(x + 1, y) + at(x - 1, y + 1) + at(x, y + 1) + at(x + 1, y + 1)) / 9.0;
            hp[y * gw + x] = at(x, y) - m;
        }
    }
    let (mut e, mut n) = (0.0, 0usize);
    let (mut eq, mut nq) = (0.0, 0usize);
    let mut by = 1;
    while by + 4 < gh - 1 {
        let mut bx = 1;
        while bx + 4 < gw - 1 {
            let mut all_land = true;
            let (mut acc, mut sq) = (0.0, 0.0);
            for y in by..by + 4 {
                for x in bx..bx + 4 {
                    if !land[y * gw + x] { all_land = false; }
                    let s = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
                    acc += hp[y * gw + x] * s;
                    sq += hp[y * gw + x] * hp[y * gw + x];
                }
            }
            if all_land { e += sq; n += 16; eq += (acc / 16.0) * (acc / 16.0); nq += 1; }
            bx += 4;
        }
        by += 4;
    }
    let rms = (e / n.max(1) as f64).sqrt();
    let q4 = (eq / nq.max(1) as f64) / (e / n.max(1) as f64).max(1e-30);
    (rms, q4)
}

/// Normalised autocorrelation of the 3x3 highpass at lag 3 along (1,0), (0,1), (1,1), (1,-1), over land cells.
/// A pure 45-degree hatch scores high on the diagonal pair and low on the axis pair; isotropic noise scores ~0 on all.
fn dir_corr(f: &[f64], land: &[bool], gw: usize, gh: usize) -> [f64; 4] {
    let at = |x: usize, y: usize| f[y * gw + x];
    let mut hp = vec![0.0f64; gw * gh];
    for y in 1..gh - 1 { for x in 1..gw - 1 {
        let mut m = 0.0; for dy in 0..3 { for dx in 0..3 { m += at(x + dx - 1, y + dy - 1); } }
        hp[y * gw + x] = at(x, y) - m / 9.0;
    } }
    let lag = 3i64;
    let dirs = [(lag, 0i64), (0, lag), (lag, lag), (lag, -lag)];
    let mut out = [0.0; 4];
    for (k, (dx, dy)) in dirs.iter().enumerate() {
        let (mut num, mut den) = (0.0, 0.0);
        for y in 8..gh - 8 { for x in 8..gw - 8 {
            let (x2, y2) = ((x as i64 + dx) as usize, (y as i64 + dy) as usize);
            if !land[y * gw + x] || !land[y2 * gw + x2] { continue; }
            num += hp[y * gw + x] * hp[y2 * gw + x2];
            den += hp[y * gw + x] * hp[y * gw + x];
        } }
        out[k] = num / den.max(1e-30);
    }
    out
}

#[test]
#[ignore]
fn diag_variants() {
    let (gw, gh) = (1024usize, 768usize);
    type Edit = Box<dyn Fn(&mut WorldParams)>;
    let variants: Vec<(&str, Edit)> = vec![
        ("app defaults", Box::new(|_| {})),
        ("carve_rivers=false", Box::new(|p| p.carve_rivers = false)),
        ("stream k=0 uplift=0", Box::new(|p| { p.stream.k = 0.0; p.stream.uplift = 0.0; })),
        ("integrate_drainage=false", Box::new(|p| p.integrate_drainage = false)),
        ("glacial=false", Box::new(|p| p.passes.glacial = false)),
        ("geology_model=false", Box::new(|p| p.geology_model = false)),
        ("engine defaults (parity)", Box::new(|p| { *p = WorldParams::defaults(p.gw, p.gh, p.tect.seed); p.map_width_km = 1200.0; p.sea_level = 0.42; })),
    ];
    for (name, ed) in variants {
        let (p, ws) = app_world(483920, gw, gh, 1200.0, |p| { p.use_gpu = false; ed(p); });
        let land: Vec<bool> = ws.field.iter().map(|&v| v as f64 > ws.sea_level as f64 + 0.01).collect();
        let f: Vec<f64> = ws.field.iter().map(|&v| v as f64).collect();
        let (er, eq) = hf_stats(&f, &land, gw, gh);
        let a = render::TerrainAppearance::default();
        let ctx = render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a);
        let l = lum(&ctx, gw, gh);
        let (lr, lq) = hf_stats(&l, &land, gw, gh);
        if let Ok(dir) = std::env::var("LAT_OUT") {
            let (x0, y0, w, h) = (340usize, 620usize, 160usize, 120usize);
            let mut buf = Vec::new();
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    let (r, g, b) = render::cell_color(&ctx, x, y);
                    buf.extend_from_slice(&[(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
                }
            }
            let tag: String = name.chars().filter(|c| c.is_alphanumeric()).collect();
            std::fs::write(format!("{dir}/var_{tag}.rgb"), buf).unwrap();
            let mut eb = Vec::new();
            for y in y0..y0 + h { for x in x0..x0 + w { eb.extend_from_slice(&ws.field[y * gw + x].to_le_bytes()); } }
            std::fs::write(format!("{dir}/var_{tag}.f32"), eb).unwrap();
        }
        let (ec, lc) = (dir_corr(&f, &land, gw, gh), dir_corr(&l, &land, gw, gh));
        println!("DIRCORR {name:28} ELEV x/y/d+/d- {:.3} {:.3} {:.3} {:.3}  LUM {:.3} {:.3} {:.3} {:.3}", ec[0], ec[1], ec[2], ec[3], lc[0], lc[1], lc[2], lc[3]);
        println!("VARIANT {name:28} ELEV hp_rms {er:.3e} Q4 {eq:.3}   LUM hp_rms {lr:.3e} Q4 {lq:.3}");
    }
}
