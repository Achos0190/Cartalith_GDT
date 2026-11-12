//! **M8 residual, physical and economic kinds** — Delta, Ancient forest, Salt works and
//! Glacial feature (`LANDMARK_GENERATION_SCOPE.md` M8, "the kind-table residual").
//!
//! Each of the four used to ship `buildable: false` because a *measured input* was missing
//! (a deposition record, a forest age, coastal salt, a record of where ice carved). This file
//! does not invent that input. For each it reads the closest physical fact the engine really
//! holds, says in the causal chain what it read, and says here what it is **not**:
//!
//! | Kind | Reads | Is not |
//! |---|---|---|
//! | Delta | a river mouth's discharge and the low coastal plain around it | a record of how much sediment settled |
//! | Ancient forest | forest biome that is wide, large and untouched by settlement or road | a forest *age* |
//! | Salt works | a flat, sheltered shore on a dry biome | a map of coastal salt |
//! | Glacial feature | a hollow or a trough inside the glacial pass's own eroding domain | a per-cell record of ice carving |
//!
//! ## §31 categories
//!
//! - **Category A:** the chamfer distance transform ([`chamfer_km`], Borgefors 1986) and the
//!   connected-component area of a forest patch.
//! - **Category B:** each detector's *rule*, cited at its constant: Galloway 1975 and Syvitski &
//!   Saito 2007 (deltas), Potapov et al. 2008 (intact forest landscapes), McGranahan et al. 2007
//!   (the 10 m low-elevation coastal zone), the reference's own glacial kernel (its activation
//!   test).
//! - **Category C:** every weight, and every threshold whose source line says "judgement".
//!
//! ## What none of these may do
//!
//! Return a candidate for want of data. Each detector returns `None` (the funnel's `NoTerrain`)
//! when the raster it needs is absent, never a plausible default (`MISTAKES.md`: never encode
//! "no value" as a plausible value).

use super::*;

// ---------------------------------------------------------------------------
// Delta
// ---------------------------------------------------------------------------

/// **Category B.** The window, in km, over which a river mouth's low coastal plain is measured.
/// Delta plains run from a few km (small rivers) to hundreds (the Nile); 12 km is the smallest
/// scale that still reads at this engine's 4-6 km cells. A labelled judgement.
const DELTA_PLAIN_KM: f64 = 12.0;
/// **Compute budget**, as [`SCALE_MAX_CELLS`] (40): the plain window is clamped to this many cells. It was 8,
/// which shrank the 12 km window to 3 km on the owner's 0.4 km cells and placed no delta there.
const DELTA_PLAIN_MAX_CELLS: i64 = 40;
/// **Category B, and measured.** A cell is "coastal plain" when its height is at most the router's
/// own marshland margin above the sea, [`crate::river_class::DELTA_LOWLAND_DH`] (0.06 field
/// units), the "low-lying" test `civ_swamp_penalty` and the river classification's "River Delta"
/// reach already use, so the landmark and the router cannot disagree about what is low.
///
/// **Why not the 10 m low-elevation coastal zone** (McGranahan, Balk & Anderson 2007), which is
/// the real-world definition and was the first choice: measured on three 256 x 192 worlds (seeds
/// 24601, 483920, 912345; 3.1 km cells) the share of land within 12 km of a large river mouth
/// that stands within 10 m of the sea is 0.00 at the median (maximum 0.12), because the engine's
/// coasts rise tens of metres inside one cell; with the router's margin it is 0.57 / 0.70 / 0.71
/// at the median and spans 0.00 to 1.00, which is a test that discriminates.
const DELTA_PLAIN_MARGIN: f64 = crate::river_class::DELTA_LOWLAND_DH;
/// **Category C.** At least this share of the land in the plain window must be coastal plain.
/// A delta is a plain the river builds *into* the water, so a mouth inside a steep fjord wall
/// is not one. A labelled judgement chosen against the distribution above (median 0.57-0.71, so
/// half the land is a demanding but reachable bar); the count it admits is in
/// (`STATUS.md`).
const DELTA_MIN_PLAIN_FRACTION: f64 = 0.50;
/// **Category C.** Sum-of-terms weights: discharge (sediment supply, Syvitski & Saito 2007's
/// first control), plain extent (accommodation space, Galloway 1975), settlement access.
const DELTA_TERMS: [(&str, f64); 3] =
    [("discharge", 0.45), ("plain extent", 0.35), ("settlement access", 0.20)];

/// §29's **Delta** (Physical, Regional) — where a large river meets standing water across a low
/// plain.
///
/// **Domain:** a land channel cell whose receiver is not land (ocean or lake), so one candidate
/// per river mouth. **Constraints:** the mouth's flow is at least
/// [`river_class::DELTA_FLOW_MULT`] times the channel threshold (the router's own marshland
/// rule, reused rather than re-chosen, so the router's "River Delta" reach and this landmark
/// cannot disagree about which rivers are large), and at least
/// [`DELTA_MIN_PLAIN_FRACTION`] of the land in the plain window is within
/// [`DELTA_PLAIN_MARGIN`] of the sea.
///
/// **Not a sediment budget.** Erosion carries sediment and lays it down, but nothing keeps how
/// much settled at each mouth, so wave and tide reworking (Galloway's triangle) is not read.
/// This is a delta *site*: the place a delta can build, not proof one did.
pub(super) fn pool_delta(c: &Ctx<'_>) -> Option<Pool> {
    let chan = c.chan?;
    let flow = c.flow?;
    let recv = c.recv?;
    c.water?;
    let gw = c.inp.gw;
    let r = c.cells(DELTA_PLAIN_KM, DELTA_PLAIN_MAX_CELLS);
    let min_flow = c.flow_thresh * crate::river_class::DELTA_FLOW_MULT;
    let has_settle = !c.inp.settlements.is_empty();
    let mut p = Pool::new();
    let (mut t_q, mut t_p, mut t_s) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        if chan[i] == 0 || !c.is_land(i) {
            continue;
        }
        let rv = recv[i];
        if rv < 0 || rv as usize >= c.n || c.is_land(rv as usize) {
            continue;
        }
        let into_lake = !c.is_ocean(rv as usize);
        let q = flow[i] as f64;
        if !(q >= min_flow) {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let (mut land, mut plain) = (0usize, 0usize);
        for dy in -r..=r {
            for dx in -r..=r {
                if let Some(j) = c.nb(x, y, dx, dy).filter(|j| c.is_land(*j)) {
                    land += 1;
                    if c.inp.field[j] as f64 <= c.inp.sea_level + DELTA_PLAIN_MARGIN {
                        plain += 1;
                    }
                }
            }
        }
        let frac = if land > 0 { plain as f64 / land as f64 } else { 0.0 };
        if !(frac >= DELTA_MIN_PLAIN_FRACTION) {
            p.rejected_constraint += 1;
            continue;
        }
        let mut facts = vec![
            format!(
                "a river mouth into {}, carrying {} cells of flow ({:.0} times the channel threshold)",
                if into_lake { "a lake" } else { "the sea" },
                thousands(q),
                q / c.flow_thresh
            ),
            format!(
                "{} of the land within {} is low coastal plain (within {} of the sea)",
                fmt_pct(frac),
                fmt_km(r as f64 * c.cell_km),
                fmt_m(DELTA_PLAIN_MARGIN * c.inp.mpu())
            ),
        ];
        if has_settle {
            facts.push(format!("nearest settlement {}", fmt_km(c.nearest_settlement_km(x, y))));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_q.push(q.ln_1p() as f32);
        t_p.push(frac as f32);
        t_s.push(c.influence(x, y) as f32);
    }
    p.terms = vec![
        (DELTA_TERMS[0].0, DELTA_TERMS[0].1, t_q),
        (DELTA_TERMS[1].0, DELTA_TERMS[1].1, t_p),
    ];
    if has_settle {
        p.terms.push((DELTA_TERMS[2].0, DELTA_TERMS[2].1, t_s));
    }
    Some(p)
}

// ---------------------------------------------------------------------------
// Ancient forest
// ---------------------------------------------------------------------------

/// **Category B.** The biomes that are forest: boreal, conifer, temperate forest, temperate
/// rainforest, tropical rainforest. Dry tropical forest and savanna are excluded because the
/// engine's `tropDry` is a seasonal scrub-forest mix whose canopy is not continuous.
pub(super) const FOREST_BIOMES: [u8; 5] = [
    crate::BIOME_BOREAL,
    crate::BIOME_CONIFER,
    crate::BIOME_TEMP_FOREST,
    crate::BIOME_TEMP_RAIN,
    crate::BIOME_TROP_WET,
];
/// **Category B.** Minimum width of an intact forest landscape: 10 km (Potapov et al. 2008,
/// *Ecology and Society* 13(2):51, the Intact Forest Landscapes criteria). A cell qualifies at
/// half that from the nearest edge.
const ANCIENT_FOREST_MIN_WIDTH_KM: f64 = 10.0;
/// **Category B.** Minimum area of an intact forest landscape: 500 km² (50 000 ha), the same
/// source.
const ANCIENT_FOREST_MIN_AREA_KM2: f64 = 500.0;
/// **Category C.** The buffer kept around a settlement or road before forest counts as untouched,
/// in km. Potapov's criteria exclude a 1 km zone around settlements; 2 km is used here because
/// this engine's settlement is a cell of 3-6 km, so one cell of margin is the smallest honest
/// buffer. A labelled judgement.
const ANCIENT_FOREST_BUFFER_KM: f64 = 2.0;
/// **Category C.** Sum-of-terms weights: depth from the edge, patch area, remoteness.
const ANCIENT_FOREST_TERMS: [(&str, f64); 3] =
    [("depth from the edge", 0.45), ("patch area", 0.35), ("remoteness", 0.20)];

/// The chamfer distance transform of Borgefors (1986), 8-neighbour (weights 1 and √2), in km:
/// the distance from each cell to the nearest `source` cell. **Category A.** Swept forward and
/// backward until nothing changes (at most [`CHAMFER_MAX_SWEEPS`] pairs; the seam of a wrapping
/// world needs a second pass). Returns `f32::INFINITY` where there is no source anywhere,
/// never `0`, so "no source" cannot read as "touching one".
pub(super) fn chamfer_km(c: &Ctx<'_>, source: &[bool]) -> Vec<f32> {
    let (gw, gh) = (c.inp.gw, c.inp.gh);
    let mut d: Vec<f32> = source.iter().map(|s| if *s { 0.0 } else { f32::INFINITY }).collect();
    let diag = std::f32::consts::SQRT_2;
    let fwd: [(i64, i64, f32); 4] = [(-1, 0, 1.0), (-1, -1, diag), (0, -1, 1.0), (1, -1, diag)];
    let bwd: [(i64, i64, f32); 4] = [(1, 0, 1.0), (1, 1, diag), (0, 1, 1.0), (-1, 1, diag)];
    for _ in 0..CHAMFER_MAX_SWEEPS {
        let mut changed = false;
        for pass in 0..2 {
            let (mask, rows): (&[(i64, i64, f32); 4], Vec<usize>) =
                if pass == 0 { (&fwd, (0..gh).collect()) } else { (&bwd, (0..gh).rev().collect()) };
            for y in rows {
                let cols: Vec<usize> = if pass == 0 { (0..gw).collect() } else { (0..gw).rev().collect() };
                for x in cols {
                    let i = y * gw + x;
                    let mut best = d[i];
                    for (dx, dy, w) in mask {
                        if let Some(j) = c.nb(x, y, *dx, *dy) {
                            let v = d[j] + w;
                            if v < best {
                                best = v;
                            }
                        }
                    }
                    if best < d[i] {
                        d[i] = best;
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    let ck = c.cell_km as f32;
    d.iter_mut().for_each(|v| *v *= ck);
    d
}

/// Sweeps of [`chamfer_km`]. A world's seam needs two; four is margin. A compute bound.
const CHAMFER_MAX_SWEEPS: usize = 4;

/// §29's **Ancient forest** (Physical; **Regional**, see below) — forest that is wide, large and
/// untouched.
///
/// **No age is modelled**, and that is the old `not_built` reason, still true. What the world
/// does hold is whether a forest is *intact*: the Intact Forest Landscapes criteria of Potapov et
/// al. (2008) call a forest intact when it is at least 500 km², at least 10 km wide, and clear of
/// settlement and road. Old-growth survives in exactly such ground, because clearing, logging
/// and fire follow access. The causal chain says "intact", never "old".
///
/// **Domain:** forest-biome land whose chamfer distance to the nearest open ground, water,
/// settlement or way is at least half [`ANCIENT_FOREST_MIN_WIDTH_KM`] and a local maximum of that
/// distance (the patch's heart), in a 4-connected patch of at least
/// [`ANCIENT_FOREST_MIN_AREA_KM2`]. Needs the biome raster and a non-empty settlement list: with
/// no settlements there is no record of human presence at all, and untouched would be a claim.
///
/// **Class is Regional**, not the research table's Continental. A 200 km exclusion against every
/// other landmark (cross-type competition) would let this kind place only where nothing else
/// is, and none on a populated world; a 500 km² patch is a regional feature.
pub(super) fn pool_ancient_forest(c: &Ctx<'_>) -> Option<Pool> {
    let biome = c.biome?;
    if c.inp.settlements.is_empty() || c.cell_km <= 0.0 {
        return None;
    }
    let (gw, gh) = (c.inp.gw, c.inp.gh);
    let forest: Vec<bool> = biome.iter().map(|b| FOREST_BIOMES.contains(b)).collect();
    // Everything that ends an intact landscape: open ground, water, and human presence.
    let buffer_cells = (ANCIENT_FOREST_BUFFER_KM / c.cell_km).ceil().max(1.0);
    let mut human = vec![false; c.n];
    for s in c.inp.settlements {
        for dy in -(buffer_cells as i64)..=(buffer_cells as i64) {
            for dx in -(buffer_cells as i64)..=(buffer_cells as i64) {
                if let Some(j) = c.nb(s.x.min(gw - 1), s.y.min(gh - 1), dx, dy) {
                    human[j] = true;
                }
            }
        }
    }
    if let Some(w) = c.ways.as_ref() {
        for i in 0..c.n {
            if w.ways_here[i] > 0 {
                human[i] = true;
            }
        }
    }
    let intact: Vec<bool> = (0..c.n).map(|i| forest[i] && !human[i]).collect();
    let src: Vec<bool> = intact.iter().map(|v| !*v).collect();
    let depth = chamfer_km(c, &src);
    // 4-connected patch areas over the intact mask.
    let mut patch_km2 = vec![0f32; c.n];
    let mut seen = vec![false; c.n];
    let mut stack: Vec<usize> = Vec::new();
    let mut members: Vec<usize> = Vec::new();
    let cell_area = c.cell_km * c.cell_km;
    for s in 0..c.n {
        if seen[s] || !intact[s] {
            continue;
        }
        seen[s] = true;
        stack.clear();
        stack.push(s);
        members.clear();
        while let Some(i) = stack.pop() {
            members.push(i);
            let (x, y) = (i % gw, i / gw);
            for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                if let Some(j) = c.nb(x, y, dx, dy).filter(|j| !seen[*j] && intact[*j]) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        let area = (members.len() as f64 * cell_area) as f32;
        for m in &members {
            patch_km2[*m] = area;
        }
    }
    let half_width = ANCIENT_FOREST_MIN_WIDTH_KM * 0.5;
    let mut p = Pool::new();
    let (mut t_d, mut t_a, mut t_r) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        if !intact[i] || !depth[i].is_finite() {
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        // The patch's heart: the farthest cell from any edge among its eight neighbours.
        let heart = [(-1i64, -1i64), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
            .iter()
            .all(|(dx, dy)| c.nb(x, y, *dx, *dy).is_none_or(|j| depth[j] <= depth[i]));
        if !heart {
            continue;
        }
        if (depth[i] as f64) < half_width || (patch_km2[i] as f64) < ANCIENT_FOREST_MIN_AREA_KM2 {
            p.rejected_constraint += 1;
            continue;
        }
        let near = c.nearest_settlement_km(x, y);
        let b = biome[i];
        let name = match b {
            crate::BIOME_BOREAL => "boreal forest",
            crate::BIOME_CONIFER => "conifer forest",
            crate::BIOME_TEMP_FOREST => "temperate forest",
            crate::BIOME_TEMP_RAIN => "temperate rainforest",
            _ => "tropical rainforest",
        };
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                format!(
                    "{} unbroken by clearing, road or settlement: {} from the nearest edge",
                    name,
                    fmt_km(depth[i] as f64)
                ),
                format!("one patch of {} km2", thousands(patch_km2[i] as f64)),
                format!("nearest settlement {}", fmt_km(near)),
            ],
        });
        t_d.push(depth[i]);
        t_a.push(patch_km2[i]);
        t_r.push(near as f32);
    }
    p.terms = vec![
        (ANCIENT_FOREST_TERMS[0].0, ANCIENT_FOREST_TERMS[0].1, t_d),
        (ANCIENT_FOREST_TERMS[1].0, ANCIENT_FOREST_TERMS[1].1, t_a),
        (ANCIENT_FOREST_TERMS[2].0, ANCIENT_FOREST_TERMS[2].1, t_r),
    ];
    Some(p)
}

// ---------------------------------------------------------------------------
// Salt works
// ---------------------------------------------------------------------------

/// **Category B.** Solar salt is made where evaporation outruns rainfall on a flat, sheltered,
/// low shore (salinas, sabkhas, lagoon pans). Biomes whose climate has that balance, with a rank
/// for how dry: desert, tropical dry, shrub. A labelled judgement on the engine's biome classes.
pub(super) const SALT_BIOMES_FOR_TEST: [u8; 3] = [crate::BIOME_DESERT, crate::BIOME_TROP_DRY, crate::BIOME_SHRUB];
const SALT_BIOME_DRYNESS: [(u8, f64); 3] =
    [(crate::BIOME_DESERT, 1.0), (crate::BIOME_TROP_DRY, 0.7), (crate::BIOME_SHRUB, 0.6)];
/// **Category C, and measured.** A pan needs a shore you can level, so the harbour's own limit
/// ([`HARBOUR_MAX_GRADIENT`], 0.06) is reused rather than a second number. The first choice, 0.02 (a
/// 1 degree shore) with a 10 m elevation (McGranahan, Balk & Anderson 2007's low-elevation coastal
/// zone), refused 85 % and 70 % of the dry coastal cells on a 256 x 192 census world and on the owner's
/// 2048 x 1311 world (0 placed there): this engine's coasts rise to a median 62 m (3.1 km cells) and 25 m
/// (0.4 km cells) in the first cell, so a real-world pan's flatness cannot be asked of a coastal cell.
/// With 0.06 and 30 m, 96 of 448 and 176 of 1 557 dry coastal cells pass (21 % and 11 %), which is
/// still a demanding test.
const SALT_MAX_GRADIENT: f64 = HARBOUR_MAX_GRADIENT;
/// **Category C, and measured** (see [`SALT_MAX_GRADIENT`]): the height of the pan above the sea, in
/// metres. 30 m is three times the real-world 10 m low-elevation coastal zone, which a cell of 0.4-3 km
/// cannot meet at its first land cell.
const SALT_MAX_ELEV_M: f64 = 30.0;
/// **Category C.** Sum-of-terms weights: dryness, shelter, level shore, settlement access.
const SALT_TERMS: [(&str, f64); 4] =
    [("dryness", 0.35), ("shelter", 0.30), ("level shore", 0.15), ("settlement access", 0.20)];

/// §29's **Salt works** (Economic, Local) — a solar salt pan on a dry, flat, sheltered shore.
///
/// **Domain:** coastal land (ocean in its eight-neighbourhood) on a [`SALT_BIOME_DRYNESS`]
/// biome. **Constraints:** within [`SALT_MAX_ELEV_M`] of the sea, gradient at most
/// [`SALT_MAX_GRADIENT`], and the water enclosed at least as much as a harbour's
/// ([`HARBOUR_MIN_SHELTER`] of the approach is land: a lagoon, not open surf).
///
/// **Not a salt map.** Mine already places rock-salt workings from the salt resource field; this
/// kind is the *coastal* half the old reason said the engine had no map for, and it reads climate
/// and shore shape instead.
pub(super) fn pool_salt_works(c: &Ctx<'_>) -> Option<Pool> {
    let biome = c.biome?;
    c.water?;
    if c.d.slope.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let r = c.cells(HARBOUR_SHELTER_KM, HARBOUR_SHELTER_MAX_CELLS).max(2);
    let has_settle = !c.inp.settlements.is_empty();
    let mut p = Pool::new();
    let (mut t_d, mut t_sh, mut t_l, mut t_s) = (vec![], vec![], vec![], vec![]);
    for i in 0..c.n {
        if !c.is_land(i) {
            continue;
        }
        let Some(&(_, dry)) = SALT_BIOME_DRYNESS.iter().find(|(b, _)| *b == biome[i]) else { continue };
        let (x, y) = (i % gw, i / gw);
        let coastal = [(-1i64, 0i64), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]
            .iter()
            .any(|(dx, dy)| c.nb(x, y, *dx, *dy).is_some_and(|j| c.is_ocean(j)));
        if !coastal {
            continue;
        }
        let (land_frac, _) = approach(c, x, y, r);
        let grad = c.gradient(i);
        if c.elev_m(i) > SALT_MAX_ELEV_M || grad > SALT_MAX_GRADIENT || land_frac < HARBOUR_MIN_SHELTER {
            p.rejected_constraint += 1;
            continue;
        }
        let mut facts = vec![
            format!("a dry coast ({}), where evaporation outruns rain", biome_name(biome[i])),
            format!("a flat shore at {} above the sea, falling at {}", fmt_m(c.elev_m(i).max(0.0)), fmt_pct(grad)),
            format!("{} of the approach is land: sheltered water", fmt_pct(land_frac)),
        ];
        if has_settle {
            facts.push(format!("nearest settlement {}", fmt_km(c.nearest_settlement_km(x, y))));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_d.push(dry as f32);
        t_sh.push(land_frac as f32);
        t_l.push(-grad as f32);
        t_s.push(c.influence(x, y) as f32);
    }
    p.terms = vec![
        (SALT_TERMS[0].0, SALT_TERMS[0].1, t_d),
        (SALT_TERMS[1].0, SALT_TERMS[1].1, t_sh),
        (SALT_TERMS[2].0, SALT_TERMS[2].1, t_l),
    ];
    if has_settle {
        p.terms.push((SALT_TERMS[3].0, SALT_TERMS[3].1, t_s));
    }
    Some(p)
}

/// The land share and mean ocean depth (m, positive) of the square window of radius `r` around a
/// cell: the harbour's "approach" measure, one body for [`pool_salt_works`] and the harbour
/// detector, so the two cannot disagree about shelter. The depth is `0.0` with no ocean in the
/// window; callers that need to tell that from a depth of zero test the land share.
pub(super) fn approach(c: &Ctx<'_>, x: usize, y: usize, r: i64) -> (f64, f64) {
    let (mut land_n, mut total, mut depth_acc, mut depth_n) = (0usize, 0usize, 0f64, 0usize);
    for dy in -r..=r {
        for dx in -r..=r {
            if let Some(j) = c.nb(x, y, dx, dy) {
                total += 1;
                if c.is_land(j) {
                    land_n += 1;
                } else if c.is_ocean(j) {
                    depth_acc += -c.elev_m(j);
                    depth_n += 1;
                }
            }
        }
    }
    let shelter = if total > 0 { land_n as f64 / total as f64 } else { 0.0 };
    (shelter, if depth_n > 0 { depth_acc / depth_n as f64 } else { 0.0 })
}

/// The engine's biome class as a lower-case phrase for a causal chain.
pub(super) fn biome_name(b: u8) -> &'static str {
    match b {
        crate::BIOME_DESERT => "desert",
        crate::BIOME_TROP_DRY => "dry tropics",
        crate::BIOME_SHRUB => "shrubland",
        crate::BIOME_SAVANNA => "savanna",
        crate::BIOME_GRASS => "grassland",
        crate::BIOME_TUNDRA => "tundra",
        crate::BIOME_ICE => "ice",
        _ => "biome",
    }
}

// ---------------------------------------------------------------------------
// Glacial feature
// ---------------------------------------------------------------------------

/// **Category C.** A cirque is a hollow at least this deep below its broad surroundings (broad
/// TPI, metres). A labelled judgement, as [`GORGE_MAX_TPI_M`] is for gorges.
const GLACIAL_CIRQUE_MAX_TPI_M: f64 = -60.0;
/// **Category C.** Minimum local relief round a cirque, metres: a headwall, not a dimple.
const GLACIAL_MIN_RELIEF_M: f64 = 150.0;
/// **The reference's own number.** `glacialKernel` over-deepens a cell "where discharge is under
/// 100", the head of the network, which is where cirques form; a trough is a channel at or above
/// it (`cartalith_erosion::passes`, `glacial_core`).
const GLACIAL_HEAD_FLOW: f64 = 100.0;
/// **Category C.** Sum-of-terms weights: hollow depth, relief, cold, height above the snowline.
const GLACIAL_TERMS: [(&str, f64); 4] =
    [("hollow depth", 0.35), ("relief", 0.25), ("cold", 0.20), ("above the snowline", 0.20)];

/// §29's **Glacial feature** (Physical, Regional) — a cirque or a glacial trough inside ground the
/// glacial pass actually works.
///
/// The old refusal was measured and stands where it was measured: the Landform layer's cirque
/// class is a cold-concave shape test that the glacial pass barely moves (over 99 % of its cells
/// are the same with the pass off and on). A hollow is therefore **not** evidence of ice.
/// What this reads instead is the pass's *own activation test* — `glacial_core` erodes a cell
/// only where it is above the snowline `sea + (1 − sea)·snowline` **and** below freezing — and
/// requires the hollow to sit inside that domain, on a world generated with the pass on
/// ([`LandmarkInputs::glacial_snow_el`] is `Some` only then). Inside the domain the pass
/// removed material from every such cell, so a hollow there has ice in its history by the
/// engine's own rule.
///
/// **Not a carve record.** The test reads the final field, and the pass lowers ground, so a valley
/// it cut below the snowline is outside the domain and is missed; the kind under-reports
/// troughs rather than inventing them. **Domain:** land in the glacial domain that is either a
/// cirque (broad TPI at most [`GLACIAL_CIRQUE_MAX_TPI_M`], relief at least
/// [`GLACIAL_MIN_RELIEF_M`], and not the channel's trunk) or a trough (a channel cell carrying at
/// least [`GLACIAL_HEAD_FLOW`] cells with relief at least [`GLACIAL_MIN_RELIEF_M`]).
pub(super) fn pool_glacial(c: &Ctx<'_>) -> Option<Pool> {
    let snow_el = c.inp.glacial_snow_el.filter(|v| v.is_finite())?;
    let temp = c.temp?;
    if c.d.tpi_broad.is_empty() || c.d.hmax.is_empty() {
        return None;
    }
    let flow = c.flow;
    let chan = c.chan;
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_h, mut t_r, mut t_c, mut t_a) = (vec![], vec![], vec![], vec![]);
    for i in 0..c.n {
        if !c.is_land(i) || !(temp[i] < 0.0) || (c.inp.field[i] as f64) < snow_el {
            continue;
        }
        let tpi = c.inp.dh_m(c.d.tpi_broad(i) as f64);
        let relief = c.relief_m(i);
        let q = flow.map(|f| f[i] as f64).unwrap_or(0.0);
        let on_channel = chan.is_some_and(|ch| ch[i] != 0);
        let cirque = tpi <= GLACIAL_CIRQUE_MAX_TPI_M && !(on_channel && q >= GLACIAL_HEAD_FLOW);
        let trough = on_channel && q >= GLACIAL_HEAD_FLOW && tpi < 0.0;
        if !(cirque || trough) {
            continue;
        }
        if relief < GLACIAL_MIN_RELIEF_M {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let above = (c.inp.field[i] as f64 - snow_el) * c.inp.mpu();
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                format!(
                    "{} inside the ground the glacial pass erodes: {} above the snowline at {:.0} °C",
                    if cirque { "a cirque, a hollow worn at the head of the ice" } else { "a trough the ice followed down a channel" },
                    fmt_m(above),
                    temp[i]
                ),
                format!("{} below its surroundings, with {} of relief round it", fmt_m(-tpi), fmt_m(relief)),
            ],
        });
        t_h.push(-tpi as f32);
        t_r.push(relief as f32);
        t_c.push(-temp[i]);
        t_a.push(above as f32);
    }
    p.terms = vec![
        (GLACIAL_TERMS[0].0, GLACIAL_TERMS[0].1, t_h),
        (GLACIAL_TERMS[1].0, GLACIAL_TERMS[1].1, t_r),
        (GLACIAL_TERMS[2].0, GLACIAL_TERMS[2].1, t_c),
        (GLACIAL_TERMS[3].0, GLACIAL_TERMS[3].1, t_a),
    ];
    Some(p)
}

#[cfg(test)]
#[path = "natural_kinds_tests.rs"]
mod tests;
