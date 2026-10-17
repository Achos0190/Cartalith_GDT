//! GF-5: karst dissolution, permeability-reduced runoff and the spring line
//! (`GEOLOGY_FIRST_SCOPE.md` §4.6, §4.12's karst row, §7 GF-5; owner Rulings
//! BH and BJ).
//!
//! Three pure functions over the GF-1 rock column, read through §2.5's
//! exposure rule:
//!
//! - [`karst_dissolution`] lowers land whose exposed rock is soluble by
//!   `min(k_k·σ·rain·τ, soluble thickness)` (§4.12's budget expression), and
//!   never below sea level.
//! - [`effective_runoff`] returns `rain·(1 − φ·P(exposed))`, the runoff the
//!   channel network's flow accumulation reads (§4.6: "`rain_eff` is used for
//!   the channel network's flow").
//! - [`spring_line_potential`] marks where a permeable cap over an
//!   impermeable substrate has its contact at the surface (§4.6's spring
//!   line; §4.11's *spring* landmark reads it at GF-6).
//!
//! The engine runs the first two only with `WorldParams::geology_karst` on (and
//! the rock gate, `geology_processes` with a column); off, no statement here is
//! reached, so every other world is unchanged by control flow.
//!
//! Must never read rock the column does not hold (a cell holding no valid rock
//! panics, as every GF kernel does: a mismatched column describes a different
//! world, and a fallback would be a plausible value standing in for none), and
//! must never lower a cell that is not soluble at the surface: B5 asserts that
//! exactly, with a mutation dropping σ.

use cartalith_terrain::geology::{GeologyColumn, Permeability, Rock};
use rayon::prelude::*;

/// §4.6's `k_k`, as metres of dissolution at unit rain on exposed soluble
/// rock, over one default world's model time (τ = 1). **Judgement** (§4.6:
/// "`k_k` and `φ` are judgements, tuned by B5 and B8"), set from
/// GF-5's measurement (`cartalith-godot/tests/gf5_karst.rs`, `gf5_bars`;
/// `GEOLOGY_FIRST_SCOPE.md` §7 "GF-5 measured"). The rate being proportional
/// to runoff is Ford & Williams (2007), not verified at the source here; the
/// magnitude is not from any source.
///
/// Chosen after measuring 10, 15 and 20 m at φ = 0.5 (2048 x 1311, 800 km,
/// five seeds): 20 m failed B8 on seed 71077345 (1-3-cell lakes 27 -> 34
/// against a bar of 33.75); 15 m passes B8 on all five and B5's depression
/// half on the four seeds where it is measurable; 10 m also passes B8 but
/// leaves B5's depression half unmeasurable on two seeds (fewer than 100
/// cells). So 15 is the largest value tried that keeps B8.
pub const KARST_DEPTH_M: f64 = 15.0;

/// §4.6's `φ`: the share of rain a fully permeable outcrop (`P = 1`) takes
/// underground, so it does not join the channel network's surface flow.
/// **Judgement**, tuned by B8 like [`KARST_DEPTH_M`]. That permeability lowers
/// surface runoff and so drainage density is Carlston (1963), as §2.3 cites
/// it; not verified at the source here. Only 0 and 0.5 were measured: the
/// runoff term alone at 0.5 (`KARST_DEPTH_M` 0) kept B8 on the three seeds
/// tried (24601, 71077345, 12345). 0.5 is not swept further.
pub const PERMEABLE_RUNOFF_LOSS: f64 = 0.5;

/// §2.3's permeability class as the `P` in §4.6's runoff formula, 0 to 1.
///
/// **Judgement**: the classes are ordinal (§2.3 reads them from Freeze &
/// Cherry 1979 Table 2.2, cited and not verified at GF-1), and these numbers
/// only space them evenly from very low (0) to high (1). `Variable`
/// (unconsolidated material: sand high, clay low, §2.3) is **0**: the model
/// does not know which it is, so it claims no runoff loss there rather than
/// inventing one. That keeps the alluvium on valley floors, where most
/// exposed `Unconsolidated` sits, draining as before.
pub fn permeability_fraction(p: Permeability) -> f64 {
    match p {
        Permeability::VeryLow => 0.0,
        Permeability::Low => 0.25,
        Permeability::Moderate => 0.5,
        Permeability::High => 1.0,
        Permeability::Variable => 0.0,
    }
}

/// The shared inputs of the three functions.
pub struct KarstRock<'a> {
    /// The GF-1 column. Read only; the caller accounts the field change
    /// (`crate::account_regolith`, losses strip regolith first, §4.9).
    pub column: &'a GeologyColumn,
    /// §2.5's `R_EXPOSE` in normalised height units.
    pub r_expose: f32,
    /// The world's sea level, normalised. Land is `field > sea`.
    pub sea: f64,
}

impl KarstRock<'_> {
    /// The exposed rock at cell `i` for surface `z`; panics on a cell
    /// holding no valid rock (module doc).
    fn exposed(&self, i: usize, z: f32) -> Rock {
        self.column
            .exposed(i, z, self.r_expose)
            .unwrap_or_else(|| panic!("GF-5: cell {i} of the rock column holds no valid rock (rock_top {})", self.column.rock_top[i]))
    }
}

/// §4.6's dissolution with §4.12's budget: on every land cell whose exposed
/// rock is soluble (σ = 1, limestone), lower the surface by
/// `min(depth·rain, soluble thickness)`, where `depth` is `k_k·τ` in
/// normalised units (the caller converts [`KARST_DEPTH_M`] with
/// `geology::m_to_norm` and multiplies by τ).
///
/// - **Soluble thickness** (§4.12: "bounded by the column"): when the
///   soluble rock is the cap of a two-layer column, the height of the
///   surface above the contact, so the pass dissolves the cap at most down
///   to the insoluble substrate it exposes; otherwise unbounded (limestone
///   with no unit recorded beneath it, or limestone exposed as the
///   substrate under a breached cap).
/// - **Base level**: never below `sea`. **Judgement**, with a physical
///   reason: dissolution needs water to drain through the rock, and at sea
///   level the water table stops it; it also keeps the pass from turning
///   land into sea, which B8's hard guard (no ocean on river paths) forbids.
///   A cell already at or below `sea` is not land and is not touched.
/// - `rain` is read as it is, with no clamp: the rate is proportional to
///   runoff (§4.6).
///
/// Returns the number of cells lowered. Must never lower a cell whose
/// exposed rock is insoluble (B5, exact).
pub fn karst_dissolution(field: &mut [f32], rain: &[f32], depth: f64, rock: &KarstRock) -> usize {
    assert_eq!(field.len(), rain.len(), "GF-5: field and rain differ in length");
    assert_eq!(field.len(), rock.column.len(), "GF-5: field and column differ in length");
    // The lowest height the pass may leave: the first f32 strictly above
    // `sea`, so a dissolved cell stays land (`field > sea`) after rounding.
    let mut floor = rock.sea as f32;
    while floor as f64 <= rock.sea {
        floor = floor.next_up();
    }
    field
        .par_iter_mut()
        .enumerate()
        .map(|(i, z)| {
            let z0 = *z as f64;
            if z0 <= rock.sea {
                return 0usize;
            }
            let exposed = rock.exposed(i, *z);
            if !exposed.props().soluble {
                return 0;
            }
            let mut dz = depth * rain[i].max(0.0) as f64;
            // The cap's own thickness above an insoluble substrate.
            if let Some((sub, c)) = rock.column.substrate(i)
                && Some(exposed) == rock.column.top(i)
                && *z >= c
                && !sub.props().soluble
            {
                dz = dz.min(z0 - c as f64);
            }
            if dz <= 0.0 {
                return 0;
            }
            // Base level: never at or below sea (the doc's judgement).
            let next = ((z0 - dz) as f32).max(floor);
            if next < *z {
                *z = next;
                1
            } else {
                0
            }
        })
        .sum()
}

/// §4.6's runoff: `rain·(1 − φ·P(exposed))` on land, `rain` unchanged on
/// cells at or below `sea` (the sea has no outcrop). `phi` is
/// [`PERMEABLE_RUNOFF_LOSS`] in the engine.
///
/// Must never raise a cell's runoff above its rain: `P` and `phi` are both in
/// `[0, 1]`, which the function asserts for `phi`.
pub fn effective_runoff(field: &[f32], rain: &[f32], phi: f64, rock: &KarstRock) -> Vec<f32> {
    assert!((0.0..=1.0).contains(&phi), "GF-5: phi {phi} outside [0, 1]");
    assert_eq!(field.len(), rain.len(), "GF-5: field and rain differ in length");
    (0..field.len())
        .into_par_iter()
        .map(|i| {
            if field[i] as f64 <= rock.sea {
                return rain[i];
            }
            let p = permeability_fraction(rock.exposed(i, field[i]).props().permeability);
            (rain[i] as f64 * (1.0 - phi * p)) as f32
        })
        .collect()
}

/// Whether `top` over `sub` is §4.6's spring-line pair: a permeable cap
/// (`P(top) ≥ 0.5`, moderate or high) over an impermeable substrate
/// (`P(sub) ≤ 0.25`, low or very low). The thresholds split §2.3's classes
/// at the middle; **judgement**. `Variable` reads as 0 ([`permeability_fraction`])
/// and so is never the cap.
pub fn is_spring_pair(top: Rock, sub: Rock) -> bool {
    permeability_fraction(top.props().permeability) >= 0.5 && permeability_fraction(sub.props().permeability) <= 0.25
}

/// §4.6's spring line: `true` on every land cell of a two-layer column whose
/// pair [`is_spring_pair`], which exposes its **cap**, and which has a
/// 4-neighbour of a two-layer column that is land and exposes its
/// **substrate**. That is where the contact outcrops, so water moving down
/// through the cap meets the impermeable unit at the surface. It is the
/// same breach-line definition as B4's cap edge (§5.2), restricted to the
/// permeable-over-impermeable pairs. `x` wraps when `world`.
///
/// Derived from the column and the surface, so it is computed when asked
/// for rather than stored: it cannot disagree with the column it reads.
pub fn spring_line_potential(field: &[f32], gw: usize, gh: usize, world: bool, rock: &KarstRock) -> Vec<bool> {
    assert_eq!(field.len(), gw * gh, "GF-5: field is not gw x gh");
    let land = |i: usize| field[i] as f64 > rock.sea;
    let exposes_sub = |i: usize| -> bool {
        match rock.column.substrate(i) {
            Some((sub, _)) => land(i) && rock.exposed(i, field[i]) == sub && Some(sub) != rock.column.top(i),
            None => false,
        }
    };
    (0..gw * gh)
        .into_par_iter()
        .map(|i| {
            let (Some(top), Some((sub, _))) = (rock.column.top(i), rock.column.substrate(i)) else {
                return false;
            };
            if !land(i) || !is_spring_pair(top, sub) || rock.exposed(i, field[i]) != top {
                return false;
            }
            let (x, y) = (i % gw, i / gw);
            let mut nb = [None; 4];
            if x > 0 {
                nb[0] = Some(i - 1);
            } else if world {
                nb[0] = Some(i + gw - 1);
            }
            if x + 1 < gw {
                nb[1] = Some(i + 1);
            } else if world {
                nb[1] = Some(i + 1 - gw);
            }
            if y > 0 {
                nb[2] = Some(i - gw);
            }
            if y + 1 < gh {
                nb[3] = Some(i + gw);
            }
            nb.into_iter().flatten().any(exposes_sub)
        })
        .collect()
}

/// Pins GF-5's three functions on hand-built columns against literal
/// heights, never against their own constants.
#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_terrain::geology::{Construction, NO_LAYER};

    /// A column of `n` cells, every cell `top` over `sub` (or single-layer
    /// when `sub` is `None`) at contact `c`, bare rock.
    fn column(tops: &[Rock], subs: &[Option<(Rock, f32)>]) -> GeologyColumn {
        GeologyColumn {
            rock_top: tops.iter().map(|&r| r as u8).collect(),
            rock_sub: subs.iter().map(|s| s.map_or(NO_LAYER, |(r, _)| r as u8)).collect(),
            contact: subs.iter().map(|s| s.map_or(f32::NAN, |(_, c)| c)).collect(),
            regolith: vec![0.0; tops.len()],
            volcanic_setting: vec![0; tops.len()],
            construction: Construction::Off,
        }
    }

    /// Protects B5's exact half: only soluble exposed rock is lowered, by
    /// `depth·rain`, and an insoluble cell under the same rain is untouched.
    #[test]
    fn only_soluble_rock_dissolves_by_depth_times_rain() {
        let col = column(&[Rock::Limestone, Rock::Granite, Rock::Sandstone, Rock::Limestone], &[None; 4]);
        let rock = KarstRock { column: &col, r_expose: 0.01, sea: 0.4 };
        let mut f = vec![0.8f32, 0.8, 0.8, 0.8];
        let n = karst_dissolution(&mut f, &[0.5, 0.5, 0.5, 1.0], 0.02, &rock);
        assert_eq!(n, 2);
        assert_eq!(f[1], 0.8, "granite must not dissolve");
        assert_eq!(f[2], 0.8, "sandstone must not dissolve");
        assert!((f[0] - 0.79).abs() < 1e-6, "0.8 - 0.02 x 0.5 = 0.79, got {}", f[0]);
        assert!((f[3] - 0.78).abs() < 1e-6, "0.8 - 0.02 x 1.0 = 0.78, got {}", f[3]);
    }

    /// Protects §4.12's column bound and the base level: a limestone cap
    /// dissolves only down to its insoluble contact, never below sea, and a
    /// sea cell is not touched.
    #[test]
    fn dissolution_stops_at_the_contact_and_at_sea() {
        let col = column(
            &[Rock::Limestone, Rock::Limestone, Rock::Limestone],
            &[Some((Rock::Shale, 0.795)), None, None],
        );
        // Sea 0.42: `0.42f32` is below 0.42 in f64, so a floor of `sea as
        // f32` would leave the cell sea; only the first f32 above it is land.
        let rock = KarstRock { column: &col, r_expose: 0.01, sea: 0.42 };
        let mut f = vec![0.8f32, 0.425, 0.41];
        karst_dissolution(&mut f, &[1.0, 1.0, 1.0], 0.05, &rock);
        assert!((f[0] - 0.795).abs() < 1e-6, "cap stops at its contact 0.795, got {}", f[0]);
        assert!(f[1] as f64 > 0.42 && (f[1] - 0.42).abs() < 1e-6, "stops just above sea 0.42, got {}", f[1]);
        assert_eq!(f[2], 0.41, "a sea cell is untouched");
        // Once the cap is consumed the substrate is exposed: shale, insoluble.
        let before = f[0];
        karst_dissolution(&mut f, &[1.0, 1.0, 1.0], 0.05, &rock);
        assert!(f[0] <= before + 1e-7 && f[0] >= 0.795 - 1e-6, "a consumed cap does not keep dissolving: {}", f[0]);
    }

    /// Protects the regolith gate: limestone under more than `R_EXPOSE` of
    /// regolith exposes unconsolidated material, which is insoluble.
    #[test]
    fn buried_limestone_does_not_dissolve() {
        let mut col = column(&[Rock::Limestone], &[None]);
        col.regolith[0] = 0.02;
        let rock = KarstRock { column: &col, r_expose: 0.01, sea: 0.4 };
        let mut f = vec![0.8f32];
        assert_eq!(karst_dissolution(&mut f, &[1.0], 0.05, &rock), 0);
        assert_eq!(f[0], 0.8);
    }

    /// Protects §4.6's runoff formula at literal values: high-permeability
    /// limestone loses half its rain at φ = 0.5, very-low shale none,
    /// moderate sandstone a quarter; the sea keeps its rain.
    #[test]
    fn runoff_falls_with_permeability() {
        let col = column(&[Rock::Limestone, Rock::Shale, Rock::Sandstone, Rock::Granite], &[None; 4]);
        let rock = KarstRock { column: &col, r_expose: 0.01, sea: 0.4 };
        let r = effective_runoff(&[0.8, 0.8, 0.8, 0.3], &[1.0, 1.0, 1.0, 1.0], 0.5, &rock);
        assert_eq!(r, vec![0.5, 1.0, 0.75, 1.0]);
        let r0 = effective_runoff(&[0.8, 0.8, 0.8, 0.8], &[0.6, 0.6, 0.6, 0.6], 0.0, &rock);
        assert_eq!(r0, vec![0.6; 4], "phi 0 is the rain unchanged");
    }

    /// Protects the ordinal spacing and the `Variable` rule (no claim: 0).
    #[test]
    fn permeability_fractions_are_the_judged_literals() {
        assert_eq!(permeability_fraction(Permeability::VeryLow), 0.0);
        assert_eq!(permeability_fraction(Permeability::Low), 0.25);
        assert_eq!(permeability_fraction(Permeability::Moderate), 0.5);
        assert_eq!(permeability_fraction(Permeability::High), 1.0);
        assert_eq!(permeability_fraction(Permeability::Variable), 0.0);
    }

    /// Protects the spring line's definition: a permeable cap exposing its
    /// cap beside a cell exposing the impermeable substrate is marked; the
    /// substrate cell, a granite-capped pair and an all-cap row are not.
    #[test]
    fn the_spring_line_is_the_permeable_cap_beside_its_exposed_substrate() {
        // Row of 4: limestone over shale everywhere, contact 0.6; cells 0-1
        // stand above it (cap exposed), 2-3 below it (shale exposed).
        let s = Some((Rock::Shale, 0.6f32));
        let col = column(&[Rock::Limestone; 4], &[s; 4]);
        let rock = KarstRock { column: &col, r_expose: 0.01, sea: 0.4 };
        let line = spring_line_potential(&[0.7, 0.7, 0.5, 0.5], 4, 1, false, &rock);
        assert_eq!(line, vec![false, true, false, false]);
        // Wrapped: cell 0's west neighbour is cell 3, which exposes shale.
        let wrapped = spring_line_potential(&[0.7, 0.7, 0.5, 0.5], 4, 1, true, &rock);
        assert_eq!(wrapped, vec![true, true, false, false]);
        // Granite (low permeability) over shale is not a spring pair.
        let g = column(&[Rock::Granite; 4], &[s; 4]);
        let rock_g = KarstRock { column: &g, r_expose: 0.01, sea: 0.4 };
        assert_eq!(spring_line_potential(&[0.7, 0.7, 0.5, 0.5], 4, 1, false, &rock_g), vec![false; 4]);
        // No exposed substrate anywhere: no line.
        assert_eq!(spring_line_potential(&[0.7; 4], 4, 1, false, &rock), vec![false; 4]);
        assert!(is_spring_pair(Rock::PlateauBasalt, Rock::Shale));
        assert!(!is_spring_pair(Rock::Shale, Rock::Limestone));
    }
}
