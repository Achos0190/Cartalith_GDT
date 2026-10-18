//! **GF-6's landmark reading of the rock column** (`GEOLOGY_FIRST_SCOPE.md`
//! §4.7, §4.11, §7 GF-6; owner Ruling BH): what the *rock formation* and
//! *waterfall* detectors read about the rock at the surface, derived on
//! demand from a generated world.
//!
//! - [`RockView::strength`]: §2.3's normalised strength `s` of the rock each
//!   cell exposes on the finished surface (`GeologyColumn::exposed`, §2.5's
//!   rule, regolith included). *Rock formation* reads it in place of the
//!   older `resistance_field` (§4.11: "`ROCK_FORMATION_MIN_RESISTANCE` reads
//!   **s** instead of `resistance_field`"). Available on any world with a
//!   column, the app's included: the column is built before any erosion
//!   (§2.1), so the rock at the surface is a fact of the world whether or not
//!   a landform process read it.
//! - [`RockView::knickpoints`]: `cartalith_erosion::carve_width::knickpoints`
//!   on the finished surface and channel tree: where a river leaves a
//!   stronger cap for its weaker substrate (§4.7). *Waterfall* reads it.
//!   **Only for a world whose landform processes read rock**
//!   (`geology_processes` on): there the erosion that cuts weak rock back
//!   faster than strong rock is what makes a step at the contact. On any
//!   other world the contact is only where a rock-blind surface happens to
//!   cross it, so no knickpoint is claimed, and the reason is kept instead
//!   ([`RockView::knick_absent`]).
//!
//! [`RockView::for_world`] refuses, with the reason, when the world has no
//! column. Must never produce a view for a world without one, and never fill
//! an absent field with a plausible one (`MISTAKES.md`: never encode "no
//! value" as a plausible value).

use cartalith_engine::{WorldParams, WorldState};
use cartalith_erosion::carve_width::knickpoints;
use cartalith_terrain::geology::{m_to_norm, Rock, R_EXPOSE_M};

/// Why a world has no [`RockView`]. Each is a reason a reader is shown;
/// none is a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RockAbsent {
    /// The world has no rock column; carries the column's own reason
    /// (`Geology::absent_reason`).
    NoColumn(&'static str),
    /// The world's grid does not match its parameters, so the view would
    /// describe a different world.
    GridMismatch,
}

impl RockAbsent {
    /// The sentence a reader is shown for a rock formation the pass did not
    /// place.
    pub fn reason(self) -> &'static str {
        match self {
            RockAbsent::NoColumn(why) => why,
            RockAbsent::GridMismatch => "The world's grid does not match its parameters, so its rock column cannot be read.",
        }
    }
}

/// Why a [`RockView`] carries no knickpoints.
pub const KNICK_PROCESSES_OFF: &str =
    "This world was generated with the landform processes not reading rock, so no river was cut back faster in weak rock than in strong, and no waterfall is read from a rock contact.";
/// Why a [`RockView`] carries no knickpoints: the world has no channel tree.
pub const KNICK_NO_CHANNELS: &str = "This world has no channel network, so no river crosses a rock contact.";

/// The rock at the surface of one finished world, as the landmark pass reads
/// it. Built by [`RockView::for_world`]; every vector is `gw * gh` long.
#[derive(Clone, Debug)]
pub struct RockView {
    pub gw: usize,
    pub gh: usize,
    /// §2.3's `s` of the exposed rock per cell.
    pub strength: Vec<f32>,
    /// The exposed rock per cell (a [`Rock`] index), for the facts a
    /// placement states.
    pub exposed: Vec<u8>,
    /// §4.7's knickpoints, or `None` with [`Self::knick_absent`] saying why.
    pub knickpoints: Option<Vec<bool>>,
    /// The reason [`Self::knickpoints`] is `None`; `None` when it is present.
    pub knick_absent: Option<&'static str>,
}

impl RockView {
    /// Derives the view for `ws`, which must have been generated from `p`.
    /// Refuses (module doc) when the world has no column.
    pub fn for_world(ws: &WorldState, p: &WorldParams) -> Result<RockView, RockAbsent> {
        let column = ws
            .geology
            .column()
            .ok_or_else(|| RockAbsent::NoColumn(ws.geology.absent_reason().unwrap_or("This world has no rock column.")))?;
        let (gw, gh) = (p.gw, p.gh);
        let n = gw * gh;
        if n == 0 || ws.field.len() != n || column.len() != n {
            return Err(RockAbsent::GridMismatch);
        }
        let sea = ws.sea_level;
        // The engine's own conversion (`generate_terrain`'s `RockContext`).
        let r_expose = m_to_norm(R_EXPOSE_M, sea, p.peak_m) as f32;
        let exposed: Vec<u8> = (0..n)
            .map(|i| {
                column
                    .exposed(i, ws.field[i], r_expose)
                    .unwrap_or_else(|| panic!("GF-6: cell {i} of the rock column holds no valid rock")) as u8
            })
            .collect();
        let strength = exposed.iter().map(|&r| Rock::from_u8(r).expect("just read from the column").props().s).collect();
        let (knickpoints, knick_absent) = match (p.geology_processes, ws.channels.as_ref()) {
            (false, _) => (None, Some(KNICK_PROCESSES_OFF)),
            (true, Some(ch)) if ch.chan.len() == n && ch.recv.len() == n => {
                (Some(knickpoints(&ws.field, &ch.chan, &ch.recv, column, sea, r_expose)), None)
            }
            (true, _) => (None, Some(KNICK_NO_CHANNELS)),
        };
        Ok(RockView { gw, gh, strength, exposed, knickpoints, knick_absent })
    }

    /// Whether every vector is `n` long, so a detector can index it.
    pub(crate) fn fits(&self, n: usize) -> bool {
        n > 0
            && self.gw * self.gh == n
            && self.strength.len() == n
            && self.exposed.len() == n
            && self.knickpoints.as_ref().is_none_or(|k| k.len() == n)
    }

    /// The exposed rock's name at `i` (§2.2's names, `RockProps::name`).
    pub fn exposed_name(&self, i: usize) -> &'static str {
        Rock::from_u8(self.exposed[i]).map(|r| r.props().name).unwrap_or("a rock the column does not name")
    }
}
