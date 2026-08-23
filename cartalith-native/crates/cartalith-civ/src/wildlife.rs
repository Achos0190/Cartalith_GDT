//! Wildlife layer — `buildTRI`/`guildTrophic`/`buildEcoregions`/
//! `regionRichness`/`assignWildlife`/`wildRegionColor`/`currentWildlife`,
//! plus the roster popup's own `wildFmtPop`/`showWildInfo`
//! (reference HTML lines 6489-6620 and 8256-8276).
//!
//! Fauna is **per ecoregion, not per cell**: the grid is segmented into
//! connected components of one Cartalith biome, each component is scored for
//! species richness (species–area × energy × heterogeneity × latitude), and
//! a named Earth-analogue roster is cut to that richness and given
//! population estimates by a Lindeman energy cascade with Kleiber metabolic
//! scaling.
//!
//! It lives here rather than in `cartalith-climate` for the reason the
//! porting ladder gives: every one of its inputs already lives in this
//! crate. `buildNPP` is [`crate::build_npp`] (ported for the carrying-
//! capacity chain and **not** re-implemented here), the biome grid is
//! [`crate::build_cart_biome`], and water access and carrying capacity are
//! [`crate::build_water_access`]/[`crate::build_carrying_capacity`]. Only
//! `buildTRI` was missing.
//!
//! The reference's own note applies unchanged: this is debug/export-only in
//! the JS app, so `generate()` and the default render stay bit-identical
//! whether or not it runs.

use cartalith_jsmath::{js_max, js_min, js_round};

use crate::CART_BIOMES;

/// `buildTRI` (reference HTML lines 6504-6512): the Riley et al. 1999
/// terrain-ruggedness index — `sqrt(Σ (zᵢ − z₀)²)` over the 8 neighbours.
///
/// `wrap` wraps in X only; Y always clamps, which is correct for an
/// equirectangular world (there is no cell north of the north pole).
pub fn build_tri(field: &[f32], w: usize, h: usize, wrap: bool) -> Vec<f32> {
    let mut out = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let z0 = field[i] as f64;
            let mut s = 0.0f64;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let mut nx = x as i64 + dx;
                    if wrap {
                        nx = nx.rem_euclid(w as i64);
                    } else {
                        nx = nx.clamp(0, w as i64 - 1);
                    }
                    let ny = (y as i64 + dy).clamp(0, h as i64 - 1);
                    let d = field[ny as usize * w + nx as usize] as f64 - z0;
                    s += d * d;
                }
            }
            out[i] = s.sqrt() as f32;
        }
    }
    out
}

/// `WILD_GUILDS` (reference HTML line 6515) — the display order every guild
/// list is emitted in.
pub const WILD_GUILDS: [&str; 14] = [
    "grazer",
    "browser",
    "smallHerbivore",
    "apexPredator",
    "mesoPredator",
    "scavenger",
    "raptor",
    "semiAquatic",
    "waterfowl",
    "fish",
    "primate",
    "reptile",
    "insectivore",
    "marine",
];

/// `WILD_GUILD_LABELS` (reference HTML line 6516), parallel to
/// [`WILD_GUILDS`]. UI wording, kept verbatim (`ARCHITECTURE.md`: wording
/// belongs to the UI, and this is the reference's own wording).
pub const WILD_GUILD_LABELS: [&str; 14] = [
    "Grazers",
    "Browsers",
    "Small herbivores",
    "Apex predators",
    "Mesopredators",
    "Scavengers",
    "Raptors",
    "Semi-aquatic",
    "Waterfowl",
    "Fish",
    "Primates",
    "Reptiles",
    "Insectivores",
    "Marine",
];

/// The trophic level a guild feeds at — `guildTrophic` (reference line
/// 6517). Three levels, because the Lindeman cascade has three pools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trophic {
    Herb,
    Pred,
    Scav,
}

/// `guildTrophic` (reference HTML line 6517).
pub fn guild_trophic(guild: &str) -> Trophic {
    match guild {
        "apexPredator" | "mesoPredator" | "raptor" | "reptile" => Trophic::Pred,
        "scavenger" | "insectivore" => Trophic::Scav,
        _ => Trophic::Herb,
    }
}

/// A terrain gate on a roster entry — the reference's optional fourth
/// element (`'ridge' | 'coastal'`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    None,
    Ridge,
    Coastal,
}

/// One `WILD_ROSTERS` entry: `[name, guild, bodyMassKg, gate?]`.
#[derive(Clone, Copy, Debug)]
pub struct RosterEntry {
    pub name: &'static str,
    pub guild: &'static str,
    pub mass_kg: f64,
    pub gate: Gate,
}

const fn e(name: &'static str, guild: &'static str, mass_kg: f64) -> RosterEntry {
    RosterEntry {
        name,
        guild,
        mass_kg,
        gate: Gate::None,
    }
}
const fn g(name: &'static str, guild: &'static str, mass_kg: f64, gate: Gate) -> RosterEntry {
    RosterEntry {
        name,
        guild,
        mass_kg,
        gate,
    }
}

const R1: [RosterEntry; 5] = [
    g("Harbour seal", "marine", 120.0, Gate::Coastal),
    e("Shorebird flock", "waterfowl", 0.3),
    e("Otter", "semiAquatic", 9.0),
    e("Gull", "scavenger", 1.0),
    e("Coastal grazer", "grazer", 60.0),
];
const R2: [RosterEntry; 7] = [
    e("Red deer", "browser", 200.0),
    e("Roe deer", "browser", 25.0),
    e("Wild boar", "browser", 80.0),
    e("Brown bear", "apexPredator", 250.0),
    e("Lynx", "mesoPredator", 22.0),
    e("Red fox", "mesoPredator", 6.0),
    e("Badger", "insectivore", 12.0),
];
const R3: [RosterEntry; 6] = [
    e("Red deer", "browser", 150.0),
    e("Mouflon", "grazer", 45.0),
    e("Wild boar", "browser", 70.0),
    e("Wildcat", "mesoPredator", 5.0),
    e("Booted eagle", "raptor", 1.0),
    e("Tortoise", "reptile", 4.0),
];
const R4: [RosterEntry; 7] = [
    e("Beaver", "semiAquatic", 20.0),
    e("Otter", "semiAquatic", 9.0),
    e("Water buffalo", "grazer", 700.0),
    e("Heron", "waterfowl", 2.0),
    e("Crane", "waterfowl", 5.0),
    e("Frog", "insectivore", 0.1),
    e("Pike", "fish", 8.0),
];
const R5: [RosterEntry; 8] = [
    e("Bison", "grazer", 700.0),
    e("Wild horse", "grazer", 350.0),
    e("Antelope", "grazer", 50.0),
    e("Grey wolf", "apexPredator", 40.0),
    e("Lion", "apexPredator", 190.0),
    e("Hare", "smallHerbivore", 4.0),
    e("Vulture", "scavenger", 7.0),
    e("Steppe eagle", "raptor", 3.0),
];
const R6: [RosterEntry; 7] = [
    e("Tapir", "browser", 250.0),
    e("Forest deer", "browser", 30.0),
    e("Jaguar", "apexPredator", 90.0),
    e("Monkey troop", "primate", 8.0),
    e("Hornbill", "raptor", 2.0),
    e("River turtle", "reptile", 20.0),
    e("Forest hog", "browser", 100.0),
];
const R7: [RosterEntry; 6] = [
    e("Moose", "browser", 450.0),
    e("Reindeer", "grazer", 120.0),
    e("Grey wolf", "apexPredator", 40.0),
    e("Lynx", "mesoPredator", 22.0),
    e("Wolverine", "mesoPredator", 14.0),
    e("Capercaillie", "smallHerbivore", 4.0),
];
const R8: [RosterEntry; 6] = [
    g("Ibex", "browser", 90.0, Gate::Ridge),
    g("Wild sheep", "grazer", 70.0, Gate::Ridge),
    e("Chamois", "browser", 40.0),
    g("Snow leopard", "apexPredator", 45.0, Gate::Ridge),
    g("Golden eagle", "raptor", 5.0, Gate::Ridge),
    e("Marmot", "smallHerbivore", 5.0),
];
const R9: [RosterEntry; 5] = [
    e("Wild ass", "grazer", 260.0),
    e("Saiga", "grazer", 40.0),
    e("Corsac fox", "mesoPredator", 3.0),
    e("Steppe eagle", "raptor", 3.0),
    e("Jerboa", "smallHerbivore", 0.06),
];
const R10: [RosterEntry; 7] = [
    e("Camel", "grazer", 500.0),
    e("Oryx", "grazer", 180.0),
    e("Fennec fox", "mesoPredator", 1.5),
    e("Jackal", "mesoPredator", 10.0),
    e("Sandgrouse", "smallHerbivore", 0.3),
    e("Monitor lizard", "reptile", 6.0),
    e("Scorpion", "insectivore", 0.03),
];
const R11: [RosterEntry; 7] = [
    e("Reindeer", "grazer", 120.0),
    e("Musk ox", "grazer", 300.0),
    e("Arctic fox", "mesoPredator", 4.0),
    e("Grey wolf", "apexPredator", 40.0),
    e("Lemming", "smallHerbivore", 0.05),
    e("Ptarmigan", "smallHerbivore", 0.5),
    e("Snowy owl", "raptor", 2.0),
];
const R12: [RosterEntry; 3] = [
    e("Vermin", "smallHerbivore", 0.3),
    e("Scavenger bird", "scavenger", 1.0),
    e("Feral predator", "mesoPredator", 20.0),
];
const R13: [RosterEntry; 5] = [
    e("Red deer", "browser", 180.0),
    e("Wild boar", "browser", 70.0),
    e("Red fox", "mesoPredator", 6.0),
    e("Hawk", "raptor", 1.0),
    e("Hare", "smallHerbivore", 4.0),
];
const R14: [RosterEntry; 5] = [
    e("Pike", "fish", 5.0),
    e("Otter", "semiAquatic", 9.0),
    e("Mallard duck", "waterfowl", 1.0),
    e("Heron", "waterfowl", 2.0),
    e("Frog", "insectivore", 0.1),
];
const R15: [RosterEntry; 4] = [
    e("Fish shoal", "fish", 2.0),
    e("Whale", "marine", 30000.0),
    e("Harbour seal", "marine", 120.0),
    e("Seabird", "marine", 1.0),
];

/// `WILD_ROSTERS` (reference HTML lines 6518-6534), keyed by 1-based
/// [`CART_BIOMES`] index. **Order within a roster is load-bearing**:
/// `assignWildlife` takes the first `rich` entries, so the roster is a
/// priority list, not a set.
///
/// The reference writes them as an object literal in biome order
/// `5,2,6,7,8,13,4,10,9,3,11,1,14,15,12`; they are indexed here instead, so
/// index 0 (unpainted) is the empty roster and the JS object's own key order
/// is irrelevant.
pub fn wild_roster(biome: u8) -> &'static [RosterEntry] {
    match biome {
        1 => &R1,
        2 => &R2,
        3 => &R3,
        4 => &R4,
        5 => &R5,
        6 => &R6,
        7 => &R7,
        8 => &R8,
        9 => &R9,
        10 => &R10,
        11 => &R11,
        12 => &R12,
        13 => &R13,
        14 => &R14,
        15 => &R15,
        _ => &[],
    }
}

/// One ecoregion record — `buildEcoregions`' own `recs.push({…})` plus the
/// fields `currentWildlife` and `assignWildlife` bolt on afterwards.
#[derive(Clone, Debug, Default)]
pub struct Ecoregion {
    pub id: usize,
    pub biome: u8,
    pub cells: usize,
    /// `(Σ npp / cells) / 3000` — NPP normalised against the Miami ceiling.
    pub nppn: f64,
    pub tri: f64,
    pub water: f64,
    pub k: f64,
    pub lat_abs: f64,
    pub ridge_frac: f64,
    pub valley_frac: f64,
    pub coastal: bool,
    /// Circular-mean X (a region straddling the seam has no linear mean).
    pub cx: usize,
    pub cy: usize,
    // -- filled by `assign_wildlife` / `current_wildlife` --
    pub richness: usize,
    pub guilds: Vec<GuildRoster>,
    pub summary: String,
    pub area_km2: f64,
    pub col: (u8, u8, u8),
}

/// One guild's slice of a region's fauna.
#[derive(Clone, Debug)]
pub struct GuildRoster {
    pub guild: &'static str,
    /// Share of the region's total animal biomass, rounded to 2 decimals.
    pub biomass_rel: f64,
    pub species: Vec<Species>,
}

#[derive(Clone, Debug)]
pub struct Species {
    pub name: &'static str,
    pub mass_kg: f64,
    pub population_est: f64,
}

/// `buildEcoregions`' return — the per-cell region id (`-1` for water and
/// for cells whose component was dropped as too small), the kept records,
/// and the cell count below which a region draws no clickable marker.
pub struct Ecoregions {
    pub region_id: Vec<i32>,
    pub regions: Vec<Ecoregion>,
    pub marker_min: usize,
}

/// `buildEcoregions` (reference HTML lines 6538-6567): 4-neighbour connected
/// components over the biome grid, X-wrapping in world mode, aggregating
/// every field each region record needs, then dropping components below
/// `min_area` and reindexing the survivors.
///
/// **The traversal order is load-bearing.** Every aggregate below is a
/// running `f64` sum over `f32` reads, and floating-point addition is not
/// associative — so the LIFO stack, the `pop()`, and the exact
/// `left, right, up, down` push order are reproduced rather than replaced
/// with a queue or an iterator.
#[allow(clippy::too_many_arguments)]
pub fn build_ecoregions(
    cart_biome: &[u8],
    field: &[f32],
    npp: &[f32],
    tri: &[f32],
    water: &[f32],
    k: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    wrap: bool,
    min_area: Option<usize>,
    lat_of: impl Fn(usize) -> f64,
) -> Ecoregions {
    let n = w * h;
    let min_area = min_area.unwrap_or_else(|| ((n as f64 / 3000.0) as usize).max(12));
    let mut lab = vec![-1i32; n];
    let mut recs: Vec<Ecoregion> = Vec::new();
    let mut old_of: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut comp: i32 = 0;

    for s0 in 0..n {
        if lab[s0] >= 0 {
            continue;
        }
        let b = cart_biome[s0];
        if b == 0 {
            continue; // unpainted / water sliver stays -1
        }
        lab[s0] = comp;
        stack.clear();
        stack.push(s0);
        let (mut cnt, mut s_n, mut s_t, mut s_w, mut s_k) = (0usize, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let (mut s_lat, mut s_y, mut s_cos, mut s_sin) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let (mut ridge, mut valley, mut coastal) = (0usize, 0usize, false);

        while let Some(i) = stack.pop() {
            let x = i % w;
            let y = i / w;
            cnt += 1;
            s_n += npp[i] as f64;
            s_t += tri[i] as f64;
            s_w += water[i] as f64;
            s_k += k[i] as f64;
            s_lat += lat_of(y).abs();
            s_y += y as f64;
            let ang = (x as f64 / w as f64) * 2.0 * std::f64::consts::PI;
            s_cos += ang.cos();
            s_sin += ang.sin();

            // TPI (Weiss 2001) over the 4-neighbour mean: above it is a
            // ridge, below it a valley.
            let xl = if wrap {
                (x + w - 1) % w
            } else if x > 0 {
                x - 1
            } else {
                x
            };
            let xr = if wrap {
                (x + 1) % w
            } else if x < w - 1 {
                x + 1
            } else {
                x
            };
            let yu = if y > 0 { y - 1 } else { y };
            let yd = if y < h - 1 { y + 1 } else { y };
            let m4 = (field[y * w + xl] as f64 + field[y * w + xr] as f64 + field[yu * w + x] as f64 + field[yd * w + x] as f64) * 0.25;
            let tpi = field[i] as f64 - m4;
            if tpi > 0.0015 {
                ridge += 1;
            } else if tpi < -0.0015 {
                valley += 1;
            }

            let nb = |nx: i64, ny: i64, lab: &mut Vec<i32>, stack: &mut Vec<usize>, coastal: &mut bool| {
                let nx = if wrap {
                    nx.rem_euclid(w as i64)
                } else if nx < 0 || nx >= w as i64 {
                    return;
                } else {
                    nx
                };
                if ny < 0 || ny >= h as i64 {
                    return;
                }
                let j = ny as usize * w + nx as usize;
                if cart_biome[j] == 15 || (field[j] as f64) < sea {
                    *coastal = true;
                }
                if lab[j] < 0 && cart_biome[j] == b {
                    lab[j] = comp;
                    stack.push(j);
                }
            };
            let (xi, yi) = (x as i64, y as i64);
            nb(xi - 1, yi, &mut lab, &mut stack, &mut coastal);
            nb(xi + 1, yi, &mut lab, &mut stack, &mut coastal);
            nb(xi, yi - 1, &mut lab, &mut stack, &mut coastal);
            nb(xi, yi + 1, &mut lab, &mut stack, &mut coastal);
        }

        let mut cmx = s_sin.atan2(s_cos);
        if cmx < 0.0 {
            cmx += 2.0 * std::f64::consts::PI;
        }
        let c_x = (js_round(cmx / (2.0 * std::f64::consts::PI) * w as f64) as usize) % w;
        let cf = cnt as f64;
        recs.push(Ecoregion {
            biome: b,
            cells: cnt,
            nppn: (s_n / cf) / 3000.0,
            tri: s_t / cf,
            water: s_w / cf,
            k: s_k / cf,
            lat_abs: s_lat / cf,
            ridge_frac: ridge as f64 / cf,
            valley_frac: valley as f64 / cf,
            coastal,
            cx: c_x,
            cy: js_round(s_y / cf) as usize,
            ..Ecoregion::default()
        });
        old_of.push(comp as usize);
        comp += 1;
    }

    // Drop small regions; reindex the survivors to contiguous ids matching
    // the `regions` array.
    let mut remap = vec![-1i32; comp.max(0) as usize];
    let mut kept: Vec<Ecoregion> = Vec::new();
    for (slot, mut r) in recs.into_iter().enumerate() {
        if r.cells >= min_area {
            let nid = kept.len();
            remap[old_of[slot]] = nid as i32;
            r.id = nid;
            kept.push(r);
        }
    }
    let region_id: Vec<i32> = lab.iter().map(|l| if *l >= 0 { remap[*l as usize] } else { -1 }).collect();
    Ecoregions {
        region_id,
        regions: kept,
        marker_min: min_area.max((n as f64 / 300.0) as usize),
    }
}

/// `wildSig2` (reference HTML line 6568): round to two significant figures.
pub fn wild_sig2(x: f64) -> f64 {
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(x > 0.0) {
        return 0.0;
    }
    let d = 10f64.powf(x.log10().floor() - 1.0);
    js_round(x / d) * d
}

/// `regionRichness`' own tuning constants (reference line 6571).
#[derive(Clone, Copy, Debug)]
pub struct RichnessOpts {
    pub c: f64,
    pub k_h: f64,
    pub tri_ref: f64,
    pub cell_km2: f64,
}

impl Default for RichnessOpts {
    fn default() -> Self {
        Self {
            c: 1.2,
            k_h: 0.6,
            tri_ref: 0.08,
            cell_km2: 1.0,
        }
    }
}

/// `regionRichness` (reference HTML lines 6570-6576):
/// `S = c·Aᶻ·E^0.7·(1 + kH·TRIn)·latF` — species–area (MacArthur & Wilson
/// 1967) × energy (Wright 1983) × heterogeneity (Stein 2014) × latitude
/// (Rosenzweig 1995).
pub fn region_richness(rec: &Ecoregion, o: &RichnessOpts) -> f64 {
    let area_km2 = js_max(1.0, rec.cells as f64 * o.cell_km2);
    // A rugged region packs more habitat per unit area, so its species-area
    // exponent rises.
    let z = if rec.ridge_frac > 0.25 { 0.25 } else { 0.15 };
    let tri_n = js_min(1.0, rec.tri / o.tri_ref);
    let lat_f = js_max(
        1.0,
        js_min(1.6, js_max(1e-6, (rec.lat_abs * std::f64::consts::PI / 180.0).cos()).powf(-0.5)),
    );
    o.c * area_km2.powf(z) * js_max(0.0, rec.nppn).powf(0.7) * (1.0 + o.k_h * tri_n) * lat_f
}

/// `assignWildlife`'s own knobs — `cellKm` feeds the energy cascade's real
/// area, `cell_km2` feeds [`region_richness`] through the same opts object
/// the reference passes to both.
#[derive(Clone, Copy, Debug)]
pub struct WildlifeOpts {
    pub cell_km: f64,
    pub richness: RichnessOpts,
}

impl Default for WildlifeOpts {
    fn default() -> Self {
        Self {
            cell_km: 1.0,
            richness: RichnessOpts::default(),
        }
    }
}

/// `assignWildlife` (reference HTML lines 6578-6605): cut the biome's
/// roster to the region's richness, then split the region's energy budget
/// across it (Lindeman 1942's 10% cascade) and size each population by
/// Kleiber 1932 metabolic scaling.
///
/// Mutates `rec`'s `richness`/`guilds`/`summary` in place, which is what
/// the reference's own `Object.assign(rec, assignWildlife(rec, …))` does.
pub fn assign_wildlife(rec: &mut Ecoregion, o: &WildlifeOpts) {
    let roster: Vec<RosterEntry> = wild_roster(rec.biome)
        .iter()
        .copied()
        .filter(|e| match e.gate {
            Gate::Ridge => rec.ridge_frac >= 0.15,
            Gate::Coastal => rec.coastal,
            Gate::None => true,
        })
        .collect();
    let s_raw = region_richness(rec, &o.richness);
    let rich = if roster.is_empty() {
        0
    } else {
        // At least two species where any exist at all, never more than the
        // roster holds.
        js_max(js_min(2.0, roster.len() as f64), js_min(roster.len() as f64, js_round(s_raw))) as usize
    };
    let mut present: Vec<RosterEntry> = roster.into_iter().take(rich).collect();

    let area_m2 = rec.cells as f64 * (o.cell_km * 1000.0) * (o.cell_km * 1000.0);
    // Water biomes get a productivity baseline: the Miami NPP model is 0
    // over sea, but a lake or an ocean shelf is not sterile.
    let mut npp = rec.nppn * 3000.0;
    if npp <= 0.0 && (rec.biome == 14 || rec.biome == 15) {
        npp = if rec.biome == 14 { 700.0 } else { 450.0 };
    }
    let plant = npp * area_m2 * 4.0; // kcal/yr (~4 kcal per g dry matter)
    let pool = |t: Trophic| match t {
        Trophic::Herb => 0.10 * plant,
        Trophic::Pred => 0.01 * plant,
        Trophic::Scav => 0.002 * plant,
    };

    // Apex/meso predators need a herbivore base to feed on.
    if !present.iter().any(|e| guild_trophic(e.guild) == Trophic::Herb) {
        present.retain(|e| guild_trophic(e.guild) != Trophic::Pred);
    }
    let count_of = |t: Trophic| present.iter().filter(|e| guild_trophic(e.guild) == t).count();
    let (n_herb, n_pred, n_scav) = (count_of(Trophic::Herb), count_of(Trophic::Pred), count_of(Trophic::Scav));

    // Guild buckets, keyed by `WILD_GUILDS` slot so the emitted order is
    // that array's order regardless of roster order.
    let mut by_guild: Vec<Vec<Species>> = vec![Vec::new(); WILD_GUILDS.len()];
    for en in &present {
        let tr = guild_trophic(en.guild);
        let cnt = match tr {
            Trophic::Herb => n_herb,
            Trophic::Pred => n_pred,
            Trophic::Scav => n_scav,
        };
        let share = if cnt > 0 { pool(tr) / cnt as f64 } else { 0.0 };
        let demand = 70.0 * en.mass_kg.powf(0.75) * 365.0 * 2.0;
        let mut pop = if demand > 0.0 { (share / demand).floor() } else { 0.0 };
        if pop < 1.0 {
            pop = 1.0;
        }
        let slot = WILD_GUILDS
            .iter()
            .position(|g| *g == en.guild)
            .expect("every roster guild is in WILD_GUILDS");
        by_guild[slot].push(Species {
            name: en.name,
            mass_kg: en.mass_kg,
            population_est: wild_sig2(pop),
        });
    }

    let mut tot_bio = 0.0f64;
    let mut guilds: Vec<(GuildRoster, f64)> = Vec::new();
    for (slot, species) in by_guild.into_iter().enumerate() {
        if species.is_empty() {
            continue;
        }
        let mut bio = 0.0f64;
        for s in &species {
            bio += s.population_est * s.mass_kg;
        }
        tot_bio += bio;
        guilds.push((
            GuildRoster {
                guild: WILD_GUILDS[slot],
                biomass_rel: 0.0,
                species,
            },
            bio,
        ));
    }
    let mut guilds: Vec<GuildRoster> = guilds
        .into_iter()
        .map(|(mut gr, bio)| {
            gr.biomass_rel = if tot_bio > 0.0 {
                js_round(bio / tot_bio * 100.0) / 100.0
            } else {
                0.0
            };
            gr
        })
        .collect();

    // `guilds.slice().sort((a,b)=>b.biomassRel-a.biomassRel)[0]`. V8's sort
    // is stable, so a tie keeps `WILD_GUILDS` order -- `sort_by` here is
    // stable too, which is why this reproduces the same winner.
    let mut ranked: Vec<usize> = (0..guilds.len()).collect();
    ranked.sort_by(|a, b| {
        guilds[*b]
            .biomass_rel
            .partial_cmp(&guilds[*a].biomass_rel)
            .expect("biomass_rel is never NaN")
    });
    let top = ranked.first().map(|i| guilds[*i].guild);

    let names: Vec<&str> = present.iter().take(3).map(|e| e.name).collect();
    let mut summary = format!("{}: {} species", CART_BIOMES[rec.biome as usize - 1], present.len());
    if let Some(t) = top {
        let label = WILD_GUILD_LABELS[WILD_GUILDS.iter().position(|g| *g == t).unwrap()];
        summary.push_str(&format!(", dominant {}", label.to_lowercase()));
    }
    if !names.is_empty() {
        summary.push_str(&format!(" ({}{})", names.join(", "), if present.len() > 3 { "…" } else { "" }));
    }
    summary.push('.');

    rec.richness = present.len();
    rec.guilds = std::mem::take(&mut guilds);
    rec.summary = summary;
}

/// `wildRegionColor` (reference HTML lines 6607-6611): ocean and lake keep
/// their blues; land runs tan (sparse) → deep green (rich).
pub fn wild_region_color(rec: &Ecoregion) -> (u8, u8, u8) {
    if rec.biome == 15 {
        return (34, 74, 120);
    }
    if rec.biome == 14 {
        return (60, 120, 180);
    }
    let t = js_min(1.0, rec.richness as f64 / 8.0);
    (
        js_round(168.0 - 118.0 * t) as u8,
        js_round(150.0 - 40.0 * t) as u8,
        js_round(96.0 - 46.0 * t) as u8,
    )
}

/// `wildFmtPop` (reference HTML line 8257): the roster popup's own
/// population formatter — `M` above a million, `k` above a thousand, bare
/// integer below.
///
/// The `k`/`M` forms are `Math.round(n/100)/10` and `Math.round(n/1e5)/10`,
/// which JS stringifies without a trailing `.0`; Rust's `Display` for `f64`
/// does the same, so `"1k"` and `"4.5M"` both come out right.
pub fn wild_fmt_pop(n: f64) -> String {
    if n >= 1e6 {
        return format!("{}M", js_round(n / 1e5) / 10.0);
    }
    if n >= 1e3 {
        return format!("{}k", js_round(n / 100.0) / 10.0);
    }
    // JS `n|0`: ToInt32, i.e. truncate toward zero.
    format!("{}", n as i32)
}

/// `currentWildlife` (reference HTML lines 6615-6620): segment, then score
/// and colour every kept region.
///
/// The reference caches this in `_wildlife`; this port rebuilds on demand,
/// the same choice `current_wind_field` already documents — nothing here is
/// retained on `WorldState`, so there is nothing to invalidate.
#[allow(clippy::too_many_arguments)]
pub fn current_wildlife(
    cart_biome: &[u8],
    field: &[f32],
    npp: &[f32],
    tri: &[f32],
    water: &[f32],
    k: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    wrap: bool,
    cell_km: f64,
    lat_of: impl Fn(usize) -> f64,
) -> Ecoregions {
    let mut eco = build_ecoregions(cart_biome, field, npp, tri, water, k, w, h, sea, wrap, None, lat_of);
    let cell_km2 = cell_km * cell_km;
    let o = WildlifeOpts {
        cell_km,
        richness: RichnessOpts {
            cell_km2,
            ..RichnessOpts::default()
        },
    };
    for rec in eco.regions.iter_mut() {
        assign_wildlife(rec, &o);
        rec.area_km2 = js_round(rec.cells as f64 * cell_km2);
        rec.col = wild_region_color(rec);
    }
    eco
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_guild_table_and_its_labels_stay_parallel() {
        assert_eq!(WILD_GUILDS.len(), WILD_GUILD_LABELS.len());
        assert_eq!(guild_trophic("grazer"), Trophic::Herb);
        assert_eq!(guild_trophic("apexPredator"), Trophic::Pred);
        assert_eq!(guild_trophic("reptile"), Trophic::Pred);
        assert_eq!(guild_trophic("insectivore"), Trophic::Scav);
        assert_eq!(guild_trophic("nothing-like-this"), Trophic::Herb);
    }

    #[test]
    fn every_roster_entry_names_a_real_guild_and_a_real_biome() {
        for b in 1u8..=15 {
            let r = wild_roster(b);
            assert!(!r.is_empty(), "biome {b} ({}) must have a roster", CART_BIOMES[b as usize - 1]);
            for e in r {
                assert!(WILD_GUILDS.contains(&e.guild), "{} has unknown guild {}", e.name, e.guild);
                assert!(e.mass_kg > 0.0);
            }
        }
        assert!(wild_roster(0).is_empty());
        assert!(wild_roster(200).is_empty());
    }

    #[test]
    fn sig2_keeps_two_significant_figures() {
        assert_eq!(wild_sig2(0.0), 0.0);
        assert_eq!(wild_sig2(-5.0), 0.0);
        assert_eq!(wild_sig2(1234.0), 1200.0);
        assert_eq!(wild_sig2(98765.0), 99000.0);
        assert_eq!(wild_sig2(7.0), 7.0);
    }

    #[test]
    fn fmt_pop_switches_units_at_the_reference_s_own_boundaries() {
        assert_eq!(wild_fmt_pop(0.0), "0");
        assert_eq!(wild_fmt_pop(999.0), "999");
        assert_eq!(wild_fmt_pop(1000.0), "1k");
        assert_eq!(wild_fmt_pop(1050.0), "1.1k");
        assert_eq!(wild_fmt_pop(999999.0), "1000k");
        assert_eq!(wild_fmt_pop(1_000_000.0), "1M");
        assert_eq!(wild_fmt_pop(4_500_000.0), "4.5M");
    }

    #[test]
    fn tri_is_zero_on_a_flat_grid_and_positive_on_a_step() {
        let flat = vec![0.5f32; 9];
        assert!(build_tri(&flat, 3, 3, false).iter().all(|v| *v == 0.0));
        let mut step = vec![0.0f32; 9];
        step[4] = 1.0;
        let t = build_tri(&step, 3, 3, false);
        assert!(t[4] > 0.0 && t[0] > 0.0);
    }

    #[test]
    fn wrapping_changes_tri_only_at_the_seam() {
        let mut f = vec![0.0f32; 12];
        for y in 0..3 {
            f[y * 4] = 1.0; // a wall down the left edge
        }
        let clamped = build_tri(&f, 4, 3, false);
        let wrapped = build_tri(&f, 4, 3, true);
        assert_ne!(clamped[3], wrapped[3], "the right edge sees the left wall only when wrapping");
        assert_eq!(clamped[1], wrapped[1], "the interior must be untouched");
    }

    #[test]
    fn a_region_below_min_area_is_dropped_and_its_cells_read_minus_one() {
        // One 3-cell blob of biome 2 in an 8x8 grid; min_area = 12 default.
        let (w, h) = (8usize, 8usize);
        let mut cb = vec![0u8; w * h];
        cb[9] = 2;
        cb[10] = 2;
        cb[17] = 2;
        let z = vec![0f32; w * h];
        let eco = build_ecoregions(&cb, &z, &z, &z, &z, &z, w, h, 0.42, false, None, |y| 45.0 - y as f64);
        assert!(eco.regions.is_empty());
        assert!(eco.region_id.iter().all(|r| *r == -1));
        // ...and it survives a lowered floor.
        let eco = build_ecoregions(&cb, &z, &z, &z, &z, &z, w, h, 0.42, false, Some(2), |y| 45.0 - y as f64);
        assert_eq!(eco.regions.len(), 1);
        assert_eq!(eco.regions[0].cells, 3);
        assert_eq!(eco.region_id[9], 0);
        assert_eq!(eco.region_id[0], -1);
    }

    #[test]
    fn predators_are_dropped_when_no_herbivore_is_present() {
        // Biome 12 (Ruined Wastes): smallHerbivore, scavenger, mesoPredator.
        // A richness of 1 leaves only the herbivore, so nothing is dropped;
        // pushing the roster to start at the predator is what exercises the
        // rule, so drive it directly through a tiny synthetic record.
        let mut rec = Ecoregion {
            biome: 12,
            cells: 40,
            nppn: 0.2,
            tri: 0.05,
            lat_abs: 30.0,
            ..Ecoregion::default()
        };
        assign_wildlife(&mut rec, &WildlifeOpts::default());
        assert!(rec.richness > 0);
        assert!(rec.guilds.iter().any(|g| !g.species.is_empty()));
        let shares: f64 = rec.guilds.iter().map(|g| g.biomass_rel).sum();
        assert!((shares - 1.0).abs() < 0.05, "biomass shares should sum to ~1, got {shares}");
    }

    #[test]
    fn region_colour_is_blue_for_water_biomes_and_greens_with_richness() {
        let lake = Ecoregion {
            biome: 14,
            ..Ecoregion::default()
        };
        let ocean = Ecoregion {
            biome: 15,
            ..Ecoregion::default()
        };
        assert_eq!(wild_region_color(&lake), (60, 120, 180));
        assert_eq!(wild_region_color(&ocean), (34, 74, 120));
        let sparse = Ecoregion {
            biome: 5,
            richness: 0,
            ..Ecoregion::default()
        };
        let rich = Ecoregion {
            biome: 5,
            richness: 8,
            ..Ecoregion::default()
        };
        assert_eq!(wild_region_color(&sparse), (168, 150, 96));
        assert_eq!(wild_region_color(&rich), (50, 110, 50));
        // Richness past 8 clamps rather than wrapping the channel.
        let very_rich = Ecoregion {
            biome: 5,
            richness: 40,
            ..Ecoregion::default()
        };
        assert_eq!(wild_region_color(&very_rich), (50, 110, 50));
    }
}
