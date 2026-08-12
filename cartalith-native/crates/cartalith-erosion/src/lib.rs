//! droplet, stream-power, thermal erosion
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

use cartalith_rng::Mulberry32;

struct HGrad {
    height: f64,
    grad_x: f64,
    grad_y: f64,
}

/// `hgrad()` (reference HTML, inside `dropletKernel`): bilinear height and
/// gradient at a fractional grid position.
fn hgrad(fld: &[f32], w: usize, px: f64, py: f64) -> HGrad {
    let nx = px as usize;
    let ny = py as usize;
    let fx = px - nx as f64;
    let fy = py - ny as f64;
    let i = ny * w + nx;
    let h00 = fld[i] as f64;
    let h10 = fld[i + 1] as f64;
    let h01 = fld[i + w] as f64;
    let h11 = fld[i + w + 1] as f64;
    let grad_x = (h10 - h00) * (1.0 - fy) + (h11 - h01) * fy;
    let grad_y = (h01 - h00) * (1.0 - fx) + (h11 - h10) * fx;
    let height = h00 * (1.0 - fx) * (1.0 - fy) + h10 * fx * (1.0 - fy) + h01 * (1.0 - fx) * fy + h11 * fx * fy;
    HGrad { height, grad_x, grad_y }
}

/// `deposit()`: bilinear-splat sediment into the (up to) four cells
/// surrounding a fractional position. No clamping — unlike `scrape`, JS's
/// `deposit` never bounds the result (erosion's own `field[i]<0/>1` clamp
/// happens once, later, in `erodeFinish`).
#[allow(clippy::too_many_arguments)]
fn deposit(fld: &mut [f32], w: usize, h: usize, nx: usize, ny: usize, fx: f64, fy: f64, amount: f64) {
    let i = ny * w + nx;
    fld[i] = (fld[i] as f64 + amount * (1.0 - fx) * (1.0 - fy)) as f32;
    if nx + 1 < w {
        fld[i + 1] = (fld[i + 1] as f64 + amount * fx * (1.0 - fy)) as f32;
    }
    if ny + 1 < h {
        fld[i + w] = (fld[i + w] as f64 + amount * (1.0 - fx) * fy) as f32;
    }
    if nx + 1 < w && ny + 1 < h {
        fld[i + w + 1] = (fld[i + w + 1] as f64 + amount * fx * fy) as f32;
    }
}

/// `scrape()`: subtracts a weighted amount across the circular brush
/// kernel, clamped at `0`. Same round-then-clamp order as every other
/// `Float32Array` clamp site in this port (`cartalith-terrain`'s
/// `stamp_one_volcano`/`stamp_one_crater`): round to `f32` first, then
/// clamp against *that* rounded value, not the pre-rounding `f64` delta.
#[allow(clippy::too_many_arguments)]
fn scrape(fld: &mut [f32], w: usize, h: usize, b_dx: &[i32], b_dy: &[i32], b_w: &[f64], cx: i64, cy: i64, amount: f64) {
    for k in 0..b_dx.len() {
        let x = cx + b_dx[k] as i64;
        let y = cy + b_dy[k] as i64;
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            continue;
        }
        let i = y as usize * w + x as usize;
        let stored = (fld[i] as f64 - amount * b_w[k]) as f32;
        fld[i] = if (stored as f64) < 0.0 { 0.0 } else { stored };
    }
}

/// The erosion tuning knobs `dropletParams()` bundles (reference HTML
/// line 3889) — `state.erosion`'s own fields, plus the derived `ck`
/// (climate-coupling strength) and `seed`.
pub struct DropletParams {
    pub droplets: i32,
    pub inertia: f64,
    pub capacity: f64,
    pub min_slope: f64,
    pub deposit: f64,
    pub erode: f64,
    pub evaporate: f64,
    pub gravity: f64,
    pub g: f64,
    pub max_lifetime: i32,
    pub init_speed: f64,
    pub init_water: f64,
    pub radius: i32,
    pub ck: f64,
    pub seed: u32,
}

/// `dropletKernel()` (reference HTML lines 3584-3616) — particle-based
/// hydraulic erosion. Self-contained by design in the original (no module
/// globals; ships to a Web Worker via `toString()`), which made it a
/// natural first erosion stage to port: the whole simulation is captured
/// by this one function's arguments.
///
/// Each droplet: spawns (rain-weighted rejection sampling when `ck>0`),
/// then for up to `max_lifetime` steps follows the inertia-blended
/// downhill gradient, erodes or deposits based on carrying capacity
/// versus current sediment load, and gains/loses speed via a simplified
/// energy-conservation term (`speed² += -dH·gravity·g`) scaled by planet
/// gravity. Mutates `fld` in place.
///
/// `rain` must be `Some` whenever `ck>0` — mirrors the JS caller's own
/// invariant (`erode()`: `ck>0?rainField:null`), not independently
/// re-validated here.
pub fn droplet_kernel(fld: &mut [f32], rain: Option<&[f32]>, w: usize, h: usize, p: &DropletParams) {
    let mut rng = Mulberry32::new(p.seed ^ 0x9e3779b9);

    let r = p.radius;
    let mut b_dx: Vec<i32> = Vec::new();
    let mut b_dy: Vec<i32> = Vec::new();
    let mut b_w: Vec<f64> = Vec::new();
    {
        let mut sum = 0.0f64;
        let mut w_raw: Vec<f64> = Vec::new();
        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = (dx * dx + dy * dy) as f64;
                if d2 <= (r * r) as f64 {
                    let wt = 1.0 - d2.sqrt() / r as f64;
                    b_dx.push(dx);
                    b_dy.push(dy);
                    w_raw.push(wt);
                    sum += wt;
                }
            }
        }
        for v in w_raw {
            b_w.push(v / sum);
        }
    }

    let ck = p.ck;

    for _ in 0..p.droplets {
        let (mut px, mut py);
        let mut tries = 0i32;
        loop {
            px = rng.next_f64() * (w as f64 - 1.0);
            py = rng.next_f64() * (h as f64 - 1.0);
            if ck > 0.0 {
                let rf = rain.expect("rain field required when ck > 0")[py as usize * w + px as usize] as f64;
                if rng.next_f64() > 0.15 + 0.85 * rf {
                    tries += 1;
                    if tries < 16 {
                        continue;
                    }
                }
            }
            break;
        }

        let mut dx = 0.0f64;
        let mut dy = 0.0f64;
        let mut speed = p.init_speed;
        let mut water = p.init_water;
        let mut sed = 0.0f64;

        for _ in 0..p.max_lifetime {
            let nx = px as usize;
            let ny = py as usize;
            let fx = px - nx as f64;
            let fy = py - ny as f64;
            let hg = hgrad(fld, w, px, py);

            dx = dx * p.inertia - hg.grad_x * (1.0 - p.inertia);
            dy = dy * p.inertia - hg.grad_y * (1.0 - p.inertia);
            let len = dx.hypot(dy);
            if len < 1e-6 {
                break;
            }
            dx /= len;
            dy /= len;
            px += dx;
            py += dy;
            if px < 0.0 || py < 0.0 || px >= w as f64 - 1.0 || py >= h as f64 - 1.0 {
                break;
            }

            let d_h = hgrad(fld, w, px, py).height - hg.height;
            let cap = (-d_h).max(p.min_slope) * speed * water * p.capacity;

            if sed > cap || d_h > 0.0 {
                let dep = if d_h > 0.0 { d_h.min(sed) } else { (sed - cap) * p.deposit };
                sed -= dep;
                deposit(fld, w, h, nx, ny, fx, fy, dep);
            } else {
                let rf = if ck > 0.0 {
                    rain.expect("rain field required when ck > 0")[ny * w + nx] as f64
                } else {
                    1.0
                };
                let ero = ((cap - sed) * p.erode * (1.0 + ck * rf)).min(-d_h);
                sed += ero;
                scrape(fld, w, h, &b_dx, &b_dy, &b_w, nx as i64, ny as i64, ero);
            }

            speed = (speed * speed + (-d_h) * p.gravity * p.g).max(0.0).sqrt();
            water *= 1.0 - p.evaporate;
            if water < 1e-3 {
                break;
            }
        }
    }
}

/// `erodeThermalCPU()` (reference HTML lines 3856-3865), CPU path only —
/// GPU is unavailable headless and JS falls back to this exact code when
/// it is. Talus-angle-driven diffusion: any cell steeper than `talus`
/// relative to a 4-connected neighbor sheds the excess, split
/// proportionally among however many neighbors are over-steep.
///
/// `delta` stays `f32` throughout, matching JS's fresh `Float32Array`
/// each pass — a downhill neighbor cell can receive `+=` contributions
/// from *several* different uphill cells within one pass, and JS rounds
/// each one individually, not once at the end. An `f64` accumulator for
/// `delta` would occasionally disagree with a genuinely multi-contributor
/// cell — the same trap `stamp_one_crater`'s three-site `field[i]+=` and
/// `compute_stress`'s `raw[i]+=` both needed `add_rounded`-style handling
/// for, just with the accumulation spread across *different cells* within
/// one pass here instead of multiple terms at *one* cell.
pub fn erode_thermal(fld: &mut [f32], w: usize, h: usize, passes: i32, talus: f64) {
    for _ in 0..passes {
        let mut delta = vec![0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let hh = fld[i] as f64;
                let mut excess = 0.0f64;
                let mut nb: Vec<(usize, f64)> = Vec::new();
                let ne: [(i64, i64); 4] = [
                    (x as i64 - 1, y as i64),
                    (x as i64 + 1, y as i64),
                    (x as i64, y as i64 - 1),
                    (x as i64, y as i64 + 1),
                ];
                for &(nx, ny) in &ne {
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    let dh = hh - fld[ny * w + nx] as f64;
                    if dh > talus {
                        nb.push((ny * w + nx, dh - talus));
                        excess += dh - talus;
                    }
                }
                if excess > 0.0 {
                    let move_amt = hh.min(excess * 0.5 * 0.25);
                    delta[i] = (delta[i] as f64 - move_amt) as f32;
                    for &(j, e) in &nb {
                        delta[j] = (delta[j] as f64 + move_amt * (e / excess)) as f32;
                    }
                }
            }
        }
        for i in 0..w * h {
            let stored = (fld[i] as f64 + delta[i] as f64) as f32;
            fld[i] = if (stored as f64) < 0.0 {
                0.0
            } else if (stored as f64) > 1.0 {
                1.0
            } else {
                stored
            };
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }
}
