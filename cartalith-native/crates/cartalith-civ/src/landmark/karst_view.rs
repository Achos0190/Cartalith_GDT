//! **GF-6's landmark reading of GF-5** (`GEOLOGY_FIRST_SCOPE.md` §4.11, §7
//! GF-6; owner Ruling BH): what the *cave* and *spring* detectors read about
//! dissolving and permeable rock, derived on demand from a generated world.
//!
//! GF-5 (`cartalith_erosion::karst`) stores nothing new on the world: its
//! spring line is "derived on demand, not stored, so it cannot disagree with
//! the column" (`GEOLOGY_FIRST_SCOPE.md` §7 "GF-5 measured and built"), and
//! which cells its dissolution pass lowered is not kept either. This module
//! derives both, from the world's own rock column and surface, with GF-5's
//! own functions, so there is one copy of each rule:
//!
//! - [`KarstView::dissolution_m`]: GF-5's dissolution pass
//!   (`karst_dissolution`, the engine's `k_k` and τ) **replayed on the
//!   finished surface and rainfall**, in metres per cell. A cave's domain is
//!   where this is above zero: soluble rock the karst model dissolves. It is
//!   the same replay GF-5's B5 measured (`cartalith-godot/tests/gf5_karst.rs`),
//!   because the pass's own input surface, inside `generate_terrain` after
//!   the threshold hillslope, is not observable from a `WorldState`. So it
//!   reads where the pass acts on the world **as finished** (later stages,
//!   such as the river carve and the post-carve climate refresh, have moved
//!   the surface and the rain since), not a record of what the pass did. The
//!   measured agreement between the two is in `GEOLOGY_FIRST_SCOPE.md` §7
//!   "GF-6 measured" (`tests/gf6_karst_landmarks.rs`).
//! - [`KarstView::spring_line`]: `spring_line_potential` itself, on the
//!   finished surface.
//!
//! **Only for a world the karst model ran on.** [`KarstView::for_world`]
//! refuses, with the reason, unless the world has a rock column and was
//! generated with `geology_processes` and `geology_karst` both on, the exact
//! gate `generate_terrain` runs GF-5 under (`RockContext::karst`). On any
//! other world (the app today: `geology_karst` ships off) there is no karst
//! fact to read, and the cave detector reports that instead of placing caves
//! at random on limestone, which is what its old `not_built` reason refused.
//! Must never produce a view for a world GF-5 did not run on, and never fill
//! an absent view with a plausible one (`MISTAKES.md`: never encode "no
//! value" as a plausible value).

use cartalith_engine::{WorldParams, WorldState};
use cartalith_erosion::karst::{karst_dissolution, spring_line_potential, KarstRock, KARST_DEPTH_M};
use cartalith_terrain::geology::{m_to_norm, Rock, NO_LAYER, R_EXPOSE_M};

/// Why a world has no [`KarstView`]. Each is a reason a reader is shown;
/// none is a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KarstAbsent {
    /// The world has no rock column; carries the column's own reason
    /// (`Geology::absent_reason`).
    NoColumn(&'static str),
    /// `geology_processes` was off, so no process read rock and the karst
    /// pass, which shares that gate, did not run.
    ProcessesOff,
    /// `geology_karst` was off (the app's default), so the karst pass did
    /// not run.
    KarstOff,
    /// The world's grid does not match its parameters, so the view would
    /// describe a different world.
    GridMismatch,
}

impl KarstAbsent {
    /// The sentence a reader is shown for a cave the pass did not place, or
    /// for a spring that kept the hydrology-only reading.
    pub fn reason(self) -> &'static str {
        match self {
            KarstAbsent::NoColumn(why) => why,
            KarstAbsent::ProcessesOff => {
                "This world was generated with the landform processes not reading rock, so the karst model did not run and nothing records where rock dissolves."
            }
            KarstAbsent::KarstOff => {
                "This world was generated with the karst model off, so nothing records where rock dissolves underground or where water meets an impermeable layer."
            }
            KarstAbsent::GridMismatch => "The world's grid does not match its parameters, so its rock column cannot be read.",
        }
    }
}

/// GF-5's karst facts about one finished world, as the landmark pass reads
/// them. Built by [`KarstView::for_world`]; every vector is `gw * gh` long.
#[derive(Clone, Debug)]
pub struct KarstView {
    pub gw: usize,
    pub gh: usize,
    /// Metres GF-5's dissolution pass lowers each cell by when replayed on
    /// the finished surface and rain (module doc). `0.0` is a real answer:
    /// the pass does not act there (insoluble rock, sea, no rain, or a cap
    /// already at its insoluble contact).
    pub dissolution_m: Vec<f32>,
    /// GF-5's spring line on the finished surface.
    pub spring_line: Vec<bool>,
    /// The exposed rock per cell (a [`Rock`] index), for the facts a
    /// placement states.
    pub exposed: Vec<u8>,
    /// The column's substrate per cell, or `NO_LAYER`, for the spring facts.
    pub substrate: Vec<u8>,
}

impl KarstView {
    /// Derives the view for `ws`, which must have been generated from `p`.
    /// Refuses (module doc) unless the karst pass ran on it.
    pub fn for_world(ws: &WorldState, p: &WorldParams) -> Result<KarstView, KarstAbsent> {
        let column = ws.geology.column().ok_or_else(|| {
            KarstAbsent::NoColumn(ws.geology.absent_reason().unwrap_or("This world has no rock column."))
        })?;
        // The engine's gate, in its order: the column, then the rock gate
        // (`rock_on = geology_processes && column`), then GF-5's switch.
        if !p.geology_processes {
            return Err(KarstAbsent::ProcessesOff);
        }
        if !p.geology_karst {
            return Err(KarstAbsent::KarstOff);
        }
        let (gw, gh) = (p.gw, p.gh);
        let n = gw * gh;
        if n == 0 || ws.field.len() != n || ws.rainfall.len() != n || column.len() != n {
            return Err(KarstAbsent::GridMismatch);
        }
        let sea = ws.sea_level;
        // The engine's own conversions (`generate_terrain`'s `RockContext`
        // and `RockContext::karst`): R_EXPOSE and k_k in normalised units, k_k
        // scaled by the clock under the same rock gate.
        let r_expose = m_to_norm(R_EXPOSE_M, sea, p.peak_m) as f32;
        let depth = m_to_norm(KARST_DEPTH_M, sea, p.peak_m)
            * cartalith_engine::geo_clock::GeoClock::new(true, p.geo_age).karst_budget_factor();
        let rock = KarstRock { column, r_expose, sea };
        let mut lowered = ws.field.as_ref().clone();
        karst_dissolution(&mut lowered, &ws.rainfall, depth, &rock);
        // Normalised height to metres: `m_to_norm`'s inverse, the anchoring
        // `LandmarkInputs::mpu` uses too.
        let mpu = if sea < 1.0 { p.peak_m / (1.0 - sea) } else { 0.0 };
        let dissolution_m = ws.field.iter().zip(&lowered).map(|(a, b)| ((*a as f64 - *b as f64) * mpu) as f32).collect();
        let spring_line = spring_line_potential(&ws.field, gw, gh, p.world, &rock);
        let exposed = (0..n)
            .map(|i| {
                column
                    .exposed(i, ws.field[i], r_expose)
                    .unwrap_or_else(|| panic!("GF-6: cell {i} of the rock column holds no valid rock")) as u8
            })
            .collect();
        Ok(KarstView { gw, gh, dissolution_m, spring_line, exposed, substrate: column.rock_sub.clone() })
    }

    /// Whether every vector is `n` long, so a detector can index it.
    pub(crate) fn fits(&self, n: usize) -> bool {
        n > 0
            && self.gw * self.gh == n
            && self.dissolution_m.len() == n
            && self.spring_line.len() == n
            && self.exposed.len() == n
            && self.substrate.len() == n
    }

    /// The exposed rock's name at `i` (§2.2's names, `RockProps::name`).
    pub(crate) fn exposed_name(&self, i: usize) -> &'static str {
        Rock::from_u8(self.exposed[i]).map(|r| r.props().name).unwrap_or("a rock the column does not name")
    }

    /// The substrate's name at `i`, or `None` for a single-layer column.
    pub(crate) fn substrate_name(&self, i: usize) -> Option<&'static str> {
        (self.substrate[i] != NO_LAYER).then(|| Rock::from_u8(self.substrate[i]).map(|r| r.props().name)).flatten()
    }
}
