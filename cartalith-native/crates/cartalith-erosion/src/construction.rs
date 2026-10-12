//! GF-11: rock-aware height construction (`GEOLOGY_FIRST_SCOPE.md` §4.14,
//! §4.16, §7 GF-11; owner Rulings BM, BN and BY).
//!
//! **What it does.** A rock-blind budget of denudation, in metres, is spent
//! through each land cell's rock column at each unit's own erodibility: a
//! cap is lowered slowly while it survives, and once the budget breaches it
//! the whole budget is spent at the substrate's rate. The breach line is
//! where the surface steps down, which is the scarp. Then the depressions
//! that differential lowering opened are filled back to their spill points
//! from the ocean, and the fill becomes regolith.
//!
//! **Why.** Ruling BM asked for landforms that track rock; GF-10 measured
//! that this construction makes the scarps and that the uplift-driven stage
//! erases them (§5.10). Ruling BY (2026-10-10) ships construction alone and
//! builds no stage (GF-12). GF-11's pre-registered rule (v3, the harness's
//! `GF11_RULE`) passed G1-G4 on the measured arm, and its cost bar G5 passed
//! once the owner relaxed it to 1.25 x (Ruling CA item 1, 2026-10-11;
//! measured 1.211 and 1.187).
//!
//! **This is the measured arm, transcribed.** Every expression here is the
//! harness's `uplift_shape`, `budget_m`, `construct` (step rule) and
//! `flood4` (`cartalith-godot/tests/gf0_geology_harness.rs`), in the same
//! order, in the same types, so the built stage reproduces the prototype the
//! bars judged bit for bit. `gf0_geology_harness.rs`'s
//! `gf11_built_stage_reproduces_the_measured_arm` asserts that on whole
//! worlds. The harness keeps its own copy on purpose: it is the independent
//! implementation the built stage is checked against.
//!
//! **What it must never do.** Write `contact` (construction is erosion in the
//! past, and erosion does not move a contact, §4.14); touch a cell that was
//! not land on the structural surface; raise a cell above its structural
//! height; leave a new closed depression it can still fill; or run without a
//! column or with an invented `D₁` (it skips with a reason instead).

use cartalith_terrain::geology::{m_to_norm, Construction, ConstructionSkip, GeologyColumn, Rock, R_EXPOSE_M};

/// §4.15: weight of the flexural bulge's positive part in the uplift shape.
/// **Judgement** (scope), the value GF-10 and GF-11 measured.
pub const A_PHI: f64 = 0.5;
/// Ruling BN (§4.15 as amended): weight of the rift term (shoulders up, axis
/// down). **Judgement**, set equal to [`A_PHI`]: both are the flexural
/// response of the same lithosphere to a boundary load.
pub const A_RIFT: f64 = 0.5;
/// §4.15: the continental background term, as a share of `U₀`, on cells
/// whose crust value is non-negative. **Judgement** (scope: `U_bg = 0.1·U₀`).
pub const U_BG: f64 = 0.1;
/// §4.14: lower clamp on the budget weight `Ū / mean_land(Ū)`. **Judgement**
/// (scope): no land is exempt from denudation; cratons strip their covers too.
pub const W_MIN: f64 = 0.25;
/// §4.14: upper clamp on the budget weight. **Judgement** (scope): no belt
/// gets an unbounded budget.
pub const W_MAX: f64 = 4.0;
/// `build_water_bodies`' `lake_depth` (`cartalith_civ::LAKE_DEPTH`, 0.004
/// normalised): a fill deeper than this is a lake. The report's
/// `new_pits` counts land cells whose depression is deeper than on the
/// structural surface by more than it (B14: must be 0).
pub const PIN_DEPTH: f64 = 0.004;
/// The fill loop's tolerance, normalised units: an excess below it is float
/// rounding, not a pit. 1e-6 is 7 mm at a 4 000 m peak and 0.42 sea
/// (arithmetic: 1e-6 × 4000 / 0.58). GF-11's rule v3 judges G4 to it.
pub const FILL_TOL: f64 = 1e-6;
/// The fill loop's pass cap: a guard against a loop that never settles,
/// never a result. Whatever it leaves is counted in
/// [`ConstructionReport::new_pits`], so a world that hits it says so. GF-11's
/// measured worlds settled well inside it (the harness prints the count).
/// Mutating it to 1 survives every test (2026-10-12): on every fixture and
/// test world one raise pass leaves no excess, so the second pass only
/// confirms; the cap binds only on a world that needs more, which none
/// tested does.
pub const FILL_PASS_CAP: usize = 64;

/// Everything construction reads besides the field and the column: the
/// pipeline's own values at the geology stage.
pub struct ConstructionInputs<'a> {
    pub gw: usize,
    pub gh: usize,
    /// East-west wrap (a whole-world map).
    pub world: bool,
    /// Sea level, normalised.
    pub sea: f64,
    /// Peak altitude, metres: with `sea`, the metres-per-unit scale.
    pub peak_m: f64,
    /// `tect.blur_r`, cells: the stress wavelength; the flexure, the rift
    /// term's wide blur and the budget's smoothing use three times it.
    pub blur_r: f64,
    /// GF-7's geological age τ, after `GeoClock`'s clamp: scales the budget
    /// linearly (§4.12, §4.16).
    pub tau: f64,
    /// §4.1's rock contrast `c` (`tect.resist`): `κ^c` per rock. At 0 every
    /// rock lowers alike, the rock-blind control.
    pub contrast: f64,
    /// `stress_field`, convergence positive.
    pub stress: &'a [f32],
    /// `boundary_mask`: non-zero on plate-boundary cells.
    pub boundary_mask: &'a [u8],
    /// `boundary_type` (`cartalith_terrain::btype`).
    pub boundary_type: &'a [u8],
    /// The crust field (`WorldState::crust_field`); `>= 0` is continental.
    pub crust: &'a [f32],
}

/// What one construction did, for the harness, the tests and a future
/// readout. Counts are cells; metres are metres.
#[derive(Clone, Debug, PartialEq)]
pub struct ConstructionReport {
    /// §4.14's `D₁`: the median land cap thickness, metres.
    pub d1_m: f64,
    /// Boundary cells typed rift that seed the rift term.
    pub rift_source_cells: usize,
    /// Land cells of the structural surface.
    pub land: usize,
    /// Land cells with a two-layer column.
    pub two_layer_land: usize,
    /// Two-layer land cells whose budget breached the cap.
    pub breached: usize,
    /// Land cells left unlowered because their exposed rock is andesite or
    /// tuff (§9 Q17: an arc edifice's relief is the stamper's, and young).
    pub arc_exempt: usize,
    /// Fill passes run, including the last one that found nothing.
    pub fill_passes: usize,
    /// Cells the fill raised at least once.
    pub raised_cells: usize,
    /// Structural land now at or below sea (Ruling BN's bays and inlets).
    pub land_to_sea: usize,
    /// Structural land the old "land stays land" floor (1 m above sea) would
    /// have caught, before the fill: reported so Q14's reversal is visible.
    pub would_floor: usize,
    /// Land cells whose depression is still deeper than on the structural
    /// surface by more than [`PIN_DEPTH`] (B14: must be 0).
    pub new_pits: usize,
}

/// The map's real outlets: every cell on a non-wrapping edge (on a world
/// map, only the polar rows). A drop off the map edge is an outlet in every
/// routing this engine does, so a depression is measured against it too.
pub fn boundary_cells(gw: usize, gh: usize, world: bool) -> Vec<bool> {
    (0..gw * gh)
        .map(|i| {
            let (x, y) = (i % gw, i / gw);
            y == 0 || y + 1 == gh || (!world && (x == 0 || x + 1 == gw))
        })
        .collect()
}

/// `f32` under `total_cmp`, so the priority flood's heap order is defined
/// for every bit pattern.
#[derive(Clone, Copy, PartialEq)]
struct OrdF32(f32);
impl Eq for OrdF32 {}
impl PartialOrd for OrdF32 {
    /// The total order's `partial_cmp`.
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for OrdF32 {
    /// `f32::total_cmp`: NaN and signed zeros order deterministically.
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&o.0)
    }
}

/// A priority-flood fill over **4-neighbours**, seeded at `seed`: each
/// cell's value is the lowest level at which water standing in it could
/// leave for a seed, so `fill − z` is the depression depth. The value is the
/// unique spill level, so it does not depend on the heap's tie order.
///
/// Why 4-neighbours: `build_water_bodies` joins below-sea water 4-connected,
/// so a bay this fill leaves open is one the classifier calls ocean; an
/// 8-neighbour fill would pass a diagonal-only mouth the classifier calls a
/// lake (§4.14, the coast bullet). Must never be used as a routing surface
/// (the engine's is `build_routing_surface`: 8-neighbour, with an ε tilt).
pub fn flood4(z: &[f32], seed: &[bool], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let n = gw * gh;
    let mut f = z.to_vec();
    let mut done = seed.to_vec();
    let mut heap = BinaryHeap::new();
    for i in 0..n {
        if seed[i] {
            heap.push(Reverse((OrdF32(f[i]), i)));
        }
    }
    while let Some(Reverse((OrdF32(fi), i))) = heap.pop() {
        let (x, y) = (i % gw, i / gw);
        let mut nb = [usize::MAX; 4];
        if x > 0 {
            nb[0] = i - 1;
        } else if world {
            nb[0] = i + gw - 1;
        }
        if x + 1 < gw {
            nb[1] = i + 1;
        } else if world {
            nb[1] = i + 1 - gw;
        }
        if y > 0 {
            nb[2] = i - gw;
        }
        if y + 1 < gh {
            nb[3] = i + gw;
        }
        for j in nb {
            if j == usize::MAX || done[j] {
                continue;
            }
            done[j] = true;
            if f[j] < fi {
                f[j] = fi;
            }
            heap.push(Reverse((OrdF32(f[j]), j)));
        }
    }
    f
}

/// The structural surface's ocean: the cells `cartalith_civ::build_water_bodies`
/// classifies as ocean (class 1), which is what the measured arm seeded its
/// fill from. Below-sea cells joined 4-connected (east-west wrapped on a world
/// map). On a bounded map every component touching a real edge is ocean
/// (Ruling Q), and if none does, the largest is (the classifier's fallback);
/// on a world map the single largest component is ocean (Ruling T). Ties go
/// to the component found first in index order, as the classifier's
/// `largest_component` breaks them.
///
/// Why a copy and not a call: `cartalith-civ` depends on this crate's
/// consumer (`cartalith-engine`), so the engine cannot call it.
/// `gf0_geology_harness.rs`'s `gf11_structural_ocean_is_the_classifiers`
/// holds the two equal on real worlds, both kinds of map. Must never be
/// changed apart from `build_water_bodies`' ocean rule.
pub fn structural_ocean(field: &[f32], gw: usize, gh: usize, sea: f64, world: bool) -> Vec<bool> {
    let n = gw * gh;
    let mut lab = vec![usize::MAX; n];
    let (mut sizes, mut touches) = (Vec::new(), Vec::new());
    let mut stack = Vec::new();
    for s in 0..n {
        if lab[s] != usize::MAX || (field[s] as f64) >= sea {
            continue;
        }
        let comp = sizes.len();
        lab[s] = comp;
        stack.push(s);
        let (mut cnt, mut edge) = (0usize, false);
        while let Some(i) = stack.pop() {
            cnt += 1;
            let (x, y) = (i % gw, i / gw);
            if (!world && (x == 0 || x + 1 == gw)) || y == 0 || y + 1 == gh {
                edge = true;
            }
            let w = if x > 0 { Some(i - 1) } else if world { Some(i + gw - 1) } else { None };
            let e = if x + 1 < gw { Some(i + 1) } else if world { Some(i + 1 - gw) } else { None };
            let up = (y > 0).then(|| i - gw);
            let dn = (y + 1 < gh).then(|| i + gw);
            for j in [w, e, up, dn].into_iter().flatten() {
                if lab[j] == usize::MAX && (field[j] as f64) < sea {
                    lab[j] = comp;
                    stack.push(j);
                }
            }
        }
        sizes.push(cnt);
        touches.push(edge);
    }
    // First strictly-larger wins: the classifier's tie rule.
    let largest = || {
        let (mut best, mut size) = (None, 0usize);
        for (c, &sz) in sizes.iter().enumerate() {
            if sz > size {
                size = sz;
                best = Some(c);
            }
        }
        best
    };
    let mut is_ocean = vec![false; sizes.len()];
    if world {
        if let Some(c) = largest() {
            is_ocean[c] = true;
        }
    } else {
        is_ocean.clone_from(&touches);
        if !is_ocean.iter().any(|&t| t) {
            if let Some(c) = largest() {
                is_ocean[c] = true;
            }
        }
    }
    (0..n).map(|i| lab[i] != usize::MAX && is_ocean[lab[i]]).collect()
}

/// §4.15's uplift field without its scale, `U(i) = U₀ · shape(i)`:
///
/// `shape = max(σ, 0) + A_PHI·max(φ, 0) + A_RIFT·R + U_BG·[crust ≥ 0]`
///
/// with `σ` the stress field, `φ` `compute_flexure`, and `R` Ruling BN's rift
/// term: the divergence `max(−σ, 0)` on rift-typed boundary cells, blurred at
/// `3·blur_r` minus the same blurred at `blur_r`, over its largest magnitude
/// (a mass-preserving difference: negative on the axis, positive on the
/// shoulders). Returns the shape and the number of rift source cells.
///
/// The orogeny term (`a_o·max(oro, 0)`) is absent: it exists only with
/// world-structure on, where [`construct_relief`]'s caller skips
/// ([`ConstructionSkip::WorldStructureOn`]). Must never be read as a rate:
/// `U₀` cancels from construction's budget and is never solved here.
pub fn uplift_shape(inp: &ConstructionInputs) -> (Vec<f64>, usize) {
    let (gw, gh, world) = (inp.gw, inp.gh, inp.world);
    let n = gw * gh;
    let phi = cartalith_terrain::compute_flexure(gw, gh, inp.boundary_mask, inp.stress, inp.blur_r, world);
    let src: Vec<f32> = (0..n)
        .map(|i| {
            if inp.boundary_mask[i] != 0 && inp.boundary_type[i] == cartalith_terrain::btype::RIFT {
                (-inp.stress[i]).max(0.0)
            } else {
                0.0
            }
        })
        .collect();
    let rift_src_cells = src.iter().filter(|&&v| v > 0.0).count();
    let narrow = cartalith_terrain::gauss_blur(&src, inp.blur_r, gw, gh, world);
    let wide = cartalith_terrain::gauss_blur(&src, 3.0 * inp.blur_r, gw, gh, world);
    let dog: Vec<f64> = (0..n).map(|i| wide[i] as f64 - narrow[i] as f64).collect();
    // 1e-12 floor: with no rift the difference is all zero and R must be 0,
    // not 0/0. Any real rift's magnitude is far above it.
    let mx = dog.iter().fold(1e-12f64, |m, v| m.max(v.abs()));
    let shape = (0..n)
        .map(|i| {
            (inp.stress[i] as f64).max(0.0) + A_PHI * (phi[i] as f64).max(0.0) + A_RIFT * dog[i] / mx + if inp.crust[i] >= 0.0 { U_BG } else { 0.0 }
        })
        .collect();
    (shape, rift_src_cells)
}

/// The arithmetic mean, `None` when empty (never NaN or 0).
fn mean(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        None
    } else {
        Some(v.iter().sum::<f64>() / v.len() as f64)
    }
}

/// The linearly interpolated median of an unsorted sample, `None` when
/// empty: the harness's `quantile(v, 0.5)`, the rule `D₁` was measured by.
fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let pos = 0.5 * (s.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    Some(s[lo] + (s[hi] - s[lo]) * (pos - lo as f64))
}

/// §4.14's pre-registered `D₁`: the median, over land cells of the
/// structural surface with a two-layer column, of the cap thickness
/// `(z₀ − contact)` in metres. `None` when there is no such cell: the rule
/// has no sample, and no number stands in for it.
pub fn median_cap_thickness_m(field: &[f32], column: &GeologyColumn, sea: f64, peak_m: f64) -> Option<f64> {
    let mpu = peak_m / (1.0 - sea);
    let h: Vec<f64> = (0..field.len())
        .filter_map(|i| match column.substrate(i) {
            Some((_, c)) if field[i] as f64 > sea => Some((field[i] as f64 - c as f64) * mpu),
            _ => None,
        })
        .collect();
    median(&h)
}

/// §4.14's budget in metres per land cell of `field` (0 elsewhere):
/// `B = τ·D₁·clamp(Ū / mean_land(Ū), W_MIN, W_MAX)`, `Ū` the uplift shape
/// blurred at `3·blur_r` so the budget varies over the plate scale and the
/// rock map supplies all the short-wavelength contrast. `None` when `field`
/// has no land (the land mean does not exist).
pub fn construction_budget_m(field: &[f32], shape: &[f64], d1_m: f64, inp: &ConstructionInputs) -> Option<Vec<f64>> {
    let (gw, gh, world) = (inp.gw, inp.gh, inp.world);
    let n = gw * gh;
    let sea = inp.sea;
    let shape_f: Vec<f32> = shape.iter().map(|&v| v as f32).collect();
    let ubar = cartalith_terrain::gauss_blur(&shape_f, 3.0 * inp.blur_r, gw, gh, world);
    let land: Vec<f64> = (0..n).filter(|&i| field[i] as f64 > sea).map(|i| ubar[i] as f64).collect();
    let m = mean(&land)?;
    Some((0..n).map(|i| if field[i] as f64 > sea { inp.tau * d1_m * (ubar[i] as f64 / m).clamp(W_MIN, W_MAX) } else { 0.0 }).collect())
}

/// GF-11's stage: lowers `field` (the structural surface on entry) by
/// §4.14's construction, fills what it opened, adds the fill to the
/// column's regolith, and records the structural surface on the column.
///
/// Returns the report, or the reason it did not run; on a skip the field is
/// untouched and the column records [`Construction::Skipped`].
///
/// Must be called once per generation, after the geology stage and before
/// the priming flow (§4.16's order), on the column that stage derived from
/// this same `field`.
pub fn construct_relief(field: &mut [f32], column: &mut GeologyColumn, inp: &ConstructionInputs) -> Result<ConstructionReport, ConstructionSkip> {
    let n = inp.gw * inp.gh;
    assert!(
        field.len() == n && column.len() == n && inp.stress.len() == n && inp.boundary_mask.len() == n && inp.boundary_type.len() == n && inp.crust.len() == n,
        "construct_relief: every input must be {n} cells ({}x{})",
        inp.gw,
        inp.gh
    );
    let skip = |column: &mut GeologyColumn, why: ConstructionSkip| {
        column.construction = Construction::Skipped(why);
        Err(why)
    };
    if !field.iter().any(|&z| z as f64 > inp.sea) {
        return skip(column, ConstructionSkip::NoLand);
    }
    let Some(d1_m) = median_cap_thickness_m(field, column, inp.sea, inp.peak_m) else {
        return skip(column, ConstructionSkip::NoTwoLayerLand);
    };
    let (shape, rift_source_cells) = uplift_shape(inp);
    let ocean0 = structural_ocean(field, inp.gw, inp.gh, inp.sea, inp.world);
    let budget = construction_budget_m(field, &shape, d1_m, inp).expect("land was checked above");
    let mut report = construct_with_budget(field, column, inp, &budget, &ocean0);
    report.d1_m = d1_m;
    report.rift_source_cells = rift_source_cells;
    Ok(report)
}

/// The construction proper, on a given budget and structural ocean: the
/// layer-through lowering (step breach rule), the ocean-seeded fill, the
/// regolith write and the record. Split from [`construct_relief`] so a
/// fixture can fix the budget and the ocean and check the arithmetic against
/// literals. `d1_m` and `rift_source_cells` are left 0 in the report: the
/// caller that derived the budget fills them.
///
/// For each land cell `i` of the structural surface `z₀`, with `B` the
/// budget, `k = κ^c`, and `h = z₀ − contact` in metres:
///
/// ```text
/// single layer:              D = B·k_top
/// cap survives (B·k_t ≤ h):  D = B·k_top
/// cap breached:              D = B·k_sub      (the step rule; §4.14, GF-10)
/// arc edifice (exposed andesite or tuff, §9 Q17): D = 0
/// z = z₀ − D
/// ```
///
/// Why the step rule: it is §4.14's construction and the one GF-11's
/// pre-registered rule measured; the continuous rule `h + (B − h/k_t)·k_s`
/// is continuous at the breach line and so makes no scarp by arithmetic
/// (GF-10 measured both, §5.10).
///
/// Then the fill (Ruling BN's coast): both surfaces are filled from the
/// structural ocean and the map's outlets ([`flood4`]); wherever the lowered
/// surface's depression is deeper than the structural surface's by more than
/// [`FILL_TOL`], the cell is raised by the excess, and the raise is added to
/// `regolith` (a basin differential erosion opens is a basin sediment
/// fills). Repeated until nothing is raised, or [`FILL_PASS_CAP`]. Existing
/// depressions keep their depth, so an existing lake bed may end below sea
/// as a lake (Ruling BY item 3). There is no floor at sea level: land ends
/// below sea only where the fill leaves it, i.e. where it opens onto the
/// ocean.
pub fn construct_with_budget(field: &mut [f32], column: &mut GeologyColumn, inp: &ConstructionInputs, budget: &[f64], ocean0: &[bool]) -> ConstructionReport {
    let (gw, gh, world) = (inp.gw, inp.gh, inp.world);
    let n = gw * gh;
    let sea = inp.sea;
    let mpu = inp.peak_m / (1.0 - sea);
    let r_expose = m_to_norm(R_EXPOSE_M, sea, inp.peak_m) as f32;
    let z0: Vec<f32> = field.to_vec();
    let land0: Vec<bool> = (0..n).map(|i| z0[i] as f64 > sea).collect();
    let kc = |r: Rock| (r.props().kappa as f64).powf(inp.contrast);
    let (mut breached, mut two, mut arc, mut land) = (0usize, 0usize, 0usize, 0usize);
    for i in 0..n {
        if !land0[i] {
            continue;
        }
        land += 1;
        let (Some(top), Some(exposed)) = (column.top(i), column.exposed(i, z0[i], r_expose)) else {
            // A cell with no valid top rock is a malformed column; `build_geology`
            // never writes one. Left unlowered rather than given a made-up κ.
            continue;
        };
        if matches!(exposed, Rock::Andesite | Rock::Tuff) {
            arc += 1;
            continue;
        }
        let b = budget[i];
        let kt = kc(top);
        let d = match column.substrate(i) {
            None => b * kt,
            Some((sub, contact)) => {
                two += 1;
                let h = (z0[i] as f64 - contact as f64) * mpu;
                if b * kt <= h {
                    b * kt
                } else {
                    breached += 1;
                    b * kc(sub)
                }
            }
        };
        field[i] = (z0[i] as f64 - d / mpu) as f32;
    }
    let would_floor = (0..n).filter(|&i| land0[i] && (field[i] as f64) < sea + 1.0 / mpu).count();
    let edge = boundary_cells(gw, gh, world);
    let seed: Vec<bool> = (0..n).map(|i| ocean0[i] || edge[i]).collect();
    let f0 = flood4(&z0, &seed, gw, gh, world);
    let d_old: Vec<f64> = (0..n).map(|i| f0[i] as f64 - z0[i] as f64).collect();
    let mut passes = 0usize;
    let mut raised = vec![false; n];
    loop {
        let f = flood4(field, &seed, gw, gh, world);
        let mut any = false;
        for i in 0..n {
            if !land0[i] {
                continue;
            }
            let excess = (f[i] as f64 - field[i] as f64) - d_old[i];
            if excess > FILL_TOL {
                let before = field[i];
                field[i] = (field[i] as f64 + excess) as f32;
                // §4.14: the raise is regolith, exactly what was added.
                column.regolith[i] = (column.regolith[i] as f64 + (field[i] as f64 - before as f64)) as f32;
                raised[i] = true;
                any = true;
            }
        }
        passes += 1;
        if !any || passes >= FILL_PASS_CAP {
            break;
        }
    }
    let f = flood4(field, &seed, gw, gh, world);
    let new_pits = (0..n).filter(|&i| land0[i] && (f[i] as f64 - field[i] as f64) - d_old[i] > PIN_DEPTH).count();
    let land_to_sea = (0..n).filter(|&i| land0[i] && (field[i] as f64) <= sea).count();
    column.construction = Construction::Built { structural_surface: z0 };
    ConstructionReport {
        d1_m: 0.0,
        rift_source_cells: 0,
        land,
        two_layer_land: two,
        breached,
        arc_exempt: arc,
        fill_passes: passes,
        raised_cells: raised.iter().filter(|&&r| r).count(),
        land_to_sea,
        would_floor,
        new_pits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_terrain::geology::NO_LAYER;

    /// The fixture of the harness's `b14_construction_keeps_the_column_and_makes_no_inland_pits`,
    /// rebuilt here against the production code: 96 × 64, a sea strip at
    /// x < 8, granite rising gently inland, a sandstone-over-shale basin
    /// (20 m cap seaward of x = 34, 400 m landward) and an inland 10 m-cap
    /// patch ringed by granite whose breach would be a pit below sea.
    struct Fix {
        gw: usize,
        gh: usize,
        z0: Vec<f32>,
        col: GeologyColumn,
        contact: Vec<f32>,
        sea: f64,
        mpu: f64,
    }

    fn fixture() -> Fix {
        let (gw, gh) = (96usize, 64usize);
        let n = gw * gh;
        let sea = 0.42f64;
        let peak = 4000.0;
        let mpu = peak / (1.0 - sea);
        let mut top = vec![Rock::Granite as u8; n];
        let mut sub = vec![NO_LAYER; n];
        let mut contact = vec![f32::NAN; n];
        let mut z0 = vec![0f32; n];
        for i in 0..n {
            let (x, y) = (i % gw, i / gw);
            z0[i] = if x < 8 { 0.40 } else { (sea + 0.0005 + 0.0002 * x as f64) as f32 };
            let cap_m = if (8..60).contains(&x) && (16..48).contains(&y) {
                Some(if x < 34 { 20.0 } else { 400.0 })
            } else if (62..69).contains(&x) && (52..60).contains(&y) {
                Some(10.0)
            } else {
                None
            };
            if let Some(h) = cap_m {
                top[i] = Rock::Sandstone as u8;
                sub[i] = Rock::Shale as u8;
                contact[i] = (z0[i] as f64 - h / mpu) as f32;
            }
        }
        let col = GeologyColumn {
            rock_top: top,
            rock_sub: sub,
            contact: contact.clone(),
            regolith: vec![0.0; n],
            volcanic_setting: vec![0; n],
            construction: Construction::Off,
        };
        Fix { gw, gh, z0, col, contact, sea, mpu }
    }

    fn inputs<'a>(f: &Fix, zeros: &'a [f32], mask: &'a [u8], crust: &'a [f32], c: f64) -> ConstructionInputs<'a> {
        ConstructionInputs {
            gw: f.gw,
            gh: f.gh,
            world: false,
            sea: f.sea,
            peak_m: 4000.0,
            blur_r: 18.0,
            tau: 1.0,
            contrast: c,
            stress: zeros,
            boundary_mask: mask,
            boundary_type: mask,
            crust,
        }
    }

    fn bits(a: &[f32], b: &[f32]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
    }

    /// Protects: B14 on the built stage (§5.2, rule v3's G4), against a
    /// fixture whose answer is known, with a flat 60 m budget: contact
    /// bit-identical; at c = 0 every non-raised land cell is `z₀ − 60 m`
    /// exactly; the breach rule's two branches against literals (a surviving
    /// 400 m sandstone cap lowers 60·0.8^0.5 = 53.666 m; a breached 20 m cap
    /// 60·2.5^0.5 = 94.868 m, §2.3's κ); regolith rose by exactly each raise;
    /// the inland breach was filled (no new pit); no land went below sea
    /// without opening onto the ocean; and the coastal breach opened a bay.
    /// Each of the scope's three mutations (drop the fill, drop the regolith
    /// write, drop the breach branch) turns an assertion here red.
    #[test]
    fn b14_on_the_built_stage() {
        for c in [0.0, 0.5] {
            let mut f = fixture();
            let n = f.gw * f.gh;
            let (zeros, mask, crust) = (vec![0f32; n], vec![0u8; n], vec![1f32; n]);
            let inp = inputs(&f, &zeros, &mask, &crust, c);
            let ocean0: Vec<bool> = (0..n).map(|i| (f.z0[i] as f64) < f.sea).collect();
            let budget: Vec<f64> = (0..n).map(|i| if f.z0[i] as f64 > f.sea { 60.0 } else { 0.0 }).collect();
            let mut z = f.z0.clone();
            let r = construct_with_budget(&mut z, &mut f.col, &inp, &budget, &ocean0);
            assert!(bits(&f.col.contact, &f.contact), "B14: construction wrote the contact (c {c})");
            assert_eq!(r.new_pits, 0, "B14: a new closed depression survived (c {c})");
            assert!((0..n).all(|i| z[i] <= f.z0[i]), "no cell may end above its structural height (c {c})");
            match &f.col.construction {
                Construction::Built { structural_surface } => assert!(bits(structural_surface, &f.z0), "the record must be the structural surface"),
                other => panic!("construction must record Built, got {other:?}"),
            }
            let low = |x: usize, y: usize| (f.z0[y * f.gw + x] as f64 - z[y * f.gw + x] as f64) * f.mpu;
            if c == 0.0 {
                // Every land cell the fill did not raise is z₀ − 60 m, bit for
                // bit: at c = 0 every κ^c is 1, the rock-blind control.
                let miss = (0..n)
                    .filter(|&i| f.z0[i] as f64 > f.sea && f.col.regolith[i] == 0.0)
                    .filter(|&i| z[i].to_bits() != ((f.z0[i] as f64 - 60.0 / f.mpu) as f32).to_bits())
                    .count();
                assert_eq!(miss, 0, "at c = 0 the unfilled surface must be z0 - B exactly");
            } else {
                assert!((low(45, 30) - 53.666).abs() < 0.01, "a surviving cap lowers by B*k_t; got {}", low(45, 30));
                // x = 20 is near the sea strip; its basin opens as a bay, so it
                // is not refilled and keeps the breached lowering.
                assert!((low(20, 30) - 94.868).abs() < 0.01, "a breached cap lowers by B*k_s; got {}", low(20, 30));
                assert!(r.breached > 0 && r.land_to_sea > 0, "the coastal breach must open a bay at c = 0.5");
                assert!(r.raised_cells > 0, "the inland breach must be filled at c = 0.5");
                // The inland patch (x 62..69, y 52..60) breached to ~2 m below
                // sea and was refilled to its spill: above sea again, and its
                // regolith is exactly its raise over z0 - 94.868 m.
                let i = 56 * f.gw + 65;
                assert!(z[i] as f64 > f.sea, "the inland pit must be refilled above sea; got {}", z[i]);
                let unfilled = f.z0[i] as f64 - 60.0 * 2.5f64.sqrt() / f.mpu;
                assert!((f.col.regolith[i] as f64 - (z[i] as f64 - unfilled)).abs() < 1e-6, "regolith must equal the fill raise exactly");
                // Ruling BN: below-sea land must be joined to the ocean.
                let oc = structural_ocean(&z, f.gw, f.gh, f.sea, false);
                let stranded = (0..n).filter(|&k| f.z0[k] as f64 > f.sea && (z[k] as f64) < f.sea && !oc[k]).count();
                assert_eq!(stranded, 0, "Ruling BN: land below sea must open onto the ocean");
            }
        }
    }

    /// Protects: `construct_relief`'s two skips -- no land, and no two-layer
    /// land -- leave the field untouched and record the reason, rather than
    /// lowering by an invented `D₁`.
    #[test]
    fn construction_skips_with_a_reason_and_touches_nothing() {
        let mut f = fixture();
        let n = f.gw * f.gh;
        let (zeros, mask, crust) = (vec![0f32; n], vec![0u8; n], vec![1f32; n]);
        let inp = inputs(&f, &zeros, &mask, &crust, 0.5);
        // No two-layer land: make every column single-layer.
        f.col.rock_sub = vec![NO_LAYER; n];
        f.col.contact = vec![f32::NAN; n];
        let mut z = f.z0.clone();
        assert_eq!(construct_relief(&mut z, &mut f.col, &inp), Err(ConstructionSkip::NoTwoLayerLand));
        assert!(bits(&z, &f.z0));
        assert_eq!(f.col.construction, Construction::Skipped(ConstructionSkip::NoTwoLayerLand));
        // No land at all.
        let mut sunk = vec![0.1f32; n];
        assert_eq!(construct_relief(&mut sunk, &mut f.col, &inp), Err(ConstructionSkip::NoLand));
        assert!(sunk.iter().all(|&v| v == 0.1));
        assert_eq!(f.col.construction, Construction::Skipped(ConstructionSkip::NoLand));
    }

    /// Protects: `D₁` is the linearly interpolated median of the land cap
    /// thicknesses in metres (the rule GF-11 measured), against literals: the
    /// fixture's caps are 20 m (x 8..34), 400 m (x 34..60) over 32 rows and
    /// 10 m (7 × 8 cells): 832 + 832 + 56 = 1 720 samples, so the median is the
    /// mean of the 860th and 861st sorted values, both 20 m.
    #[test]
    fn d1_is_the_median_land_cap_thickness() {
        let f = fixture();
        let d1 = median_cap_thickness_m(&f.z0, &f.col, f.sea, 4000.0).expect("two-layer land");
        assert!((d1 - 20.0).abs() < 1e-3, "D1 must be 20 m on the fixture; got {d1}");
        assert_eq!(median(&[3.0, 1.0, 2.0, 10.0]), Some(2.5), "an even sample interpolates");
        assert_eq!(median(&[]), None, "an empty sample has no median");
    }

    /// Protects: the budget's formula and clamps against literals: with a
    /// uniform shape the weight is 1 and B = τ·D₁ on land and 0 at sea; with a
    /// shape 100× the land mean on one patch the weight clamps at W_MAX = 4,
    /// and a near-zero shape clamps at W_MIN = 0.25.
    #[test]
    fn the_budget_is_tau_d1_times_the_clamped_weight() {
        let f = fixture();
        let n = f.gw * f.gh;
        let (zeros, mask, crust) = (vec![0f32; n], vec![0u8; n], vec![1f32; n]);
        let mut inp = inputs(&f, &zeros, &mask, &crust, 0.5);
        inp.tau = 2.0;
        let b = construction_budget_m(&f.z0, &vec![1.0; n], 30.0, &inp).unwrap();
        let land = (0..n).find(|&i| f.z0[i] as f64 > f.sea).unwrap();
        assert!((b[land] - 60.0).abs() < 1e-4, "uniform shape: B = tau * D1 = 60 m; got {}", b[land]);
        assert_eq!(b[0], 0.0, "a sea cell has no budget");
        inp.tau = 1.0;
        inp.blur_r = 0.5; // keep the patch's blur local
        let mut shape = vec![1e-6f64; n];
        for y in 20..44 {
            for x in 60..90 {
                shape[y * f.gw + x] = 1.0;
            }
        }
        let b = construction_budget_m(&f.z0, &shape, 10.0, &inp).unwrap();
        assert!((b[32 * f.gw + 75] - 40.0).abs() < 1e-6, "a hot patch clamps at W_MAX = 4: 40 m; got {}", b[32 * f.gw + 75]);
        assert!((b[5 * f.gw + 20] - 2.5).abs() < 1e-6, "a cold cell clamps at W_MIN = 0.25: 2.5 m; got {}", b[5 * f.gw + 20]);
        assert_eq!(construction_budget_m(&vec![0.1; n], &shape, 10.0, &inp), None, "no land: no mean, no budget");
    }

    /// Protects: the uplift shape's terms and weights against literals on a
    /// grid with no boundary and no rift: every term but the background is 0,
    /// so the shape is U_BG = 0.1 on continental crust and 0 on oceanic.
    /// Then a positive stress cell adds max(σ, 0) exactly.
    #[test]
    fn the_uplift_shape_reads_its_terms() {
        let (gw, gh) = (40usize, 20usize);
        let n = gw * gh;
        let mut stress = vec![0f32; n];
        let mask = vec![0u8; n];
        let crust: Vec<f32> = (0..n).map(|i| if i % gw < 20 { 1.0 } else { -1.0 }).collect();
        stress[10 * gw + 5] = 0.75;
        let inp = ConstructionInputs {
            gw,
            gh,
            world: false,
            sea: 0.42,
            peak_m: 4000.0,
            blur_r: 2.0,
            tau: 1.0,
            contrast: 0.5,
            stress: &stress,
            boundary_mask: &mask,
            boundary_type: &mask,
            crust: &crust,
        };
        let (s, rift) = uplift_shape(&inp);
        assert_eq!(rift, 0);
        assert!((s[0] - 0.1).abs() < 1e-12, "continental background is 0.1; got {}", s[0]);
        assert_eq!(s[30], 0.0, "oceanic crust gets no background");
        assert!((s[10 * gw + 5] - 0.85).abs() < 1e-6, "stress adds max(sigma, 0): 0.75 + 0.1; got {}", s[10 * gw + 5]);
    }

    /// Protects: `structural_ocean`'s rule on hand-built grids: on a bounded
    /// map an edge-touching below-sea component is ocean and an enclosed one
    /// is not; with no edge-touching component the largest is (the fallback);
    /// on a world map only the single largest is ocean, and a component that
    /// wraps east-west is joined across the seam.
    #[test]
    fn the_structural_ocean_follows_the_classifiers_rule() {
        let (gw, gh) = (12usize, 8usize);
        let n = gw * gh;
        let sea = 0.5;
        let mut z = vec![0.9f32; n];
        // Edge-touching sea at x = 0, rows 2..6.
        for y in 2..6 {
            z[y * gw] = 0.1;
        }
        // Enclosed 2x2 basin at (5..7, 3..5).
        for y in 3..5 {
            for x in 5..7 {
                z[y * gw + x] = 0.1;
            }
        }
        let o = structural_ocean(&z, gw, gh, sea, false);
        assert!(o[3 * gw] && !o[3 * gw + 5], "edge sea is ocean, the enclosed basin is not");
        // Only enclosed basins: the larger wins.
        let mut z2 = vec![0.9f32; n];
        z2[3 * gw + 3] = 0.1;
        for y in 3..5 {
            for x in 6..8 {
                z2[y * gw + x] = 0.1;
            }
        }
        let o2 = structural_ocean(&z2, gw, gh, sea, false);
        assert!(o2[3 * gw + 6] && !o2[3 * gw + 3], "no edge sea: the largest component is ocean");
        // World map: a strip touching x = 0 and x = gw-1 joins across the seam
        // (4 cells), beating an interior 3-cell component.
        let mut z3 = vec![0.9f32; n];
        z3[4 * gw] = 0.1;
        z3[4 * gw + 1] = 0.1;
        z3[4 * gw + gw - 1] = 0.1;
        z3[4 * gw + gw - 2] = 0.1;
        for x in 5..8 {
            z3[2 * gw + x] = 0.1;
        }
        // And a 2-cell component on the polar row: it touches a real edge, so
        // the bounded rule would call it ocean; Ruling T's single-largest rule
        // must not.
        z3[3] = 0.1;
        z3[4] = 0.1;
        let o3 = structural_ocean(&z3, gw, gh, sea, true);
        assert!(o3[4 * gw] && o3[4 * gw + gw - 1] && !o3[2 * gw + 6], "world map: the wrapped strip is the one ocean");
        assert!(!o3[3], "world map: a smaller polar component is not ocean (Ruling T)");
    }
}
